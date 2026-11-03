// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The seat's terminal face: append-only lines with SGR colour and OSC 8 links, no cursor
//! addressing, no redraw (ICD embed-seat §2). Same structure as the rich face: the user
//! speaks in a highlighted block, agent markdown is rendered for the terminal, reasoning is
//! one "Thought for" line, one tool call is one card with its approval, terminal and file
//! change folded in, and a turn ends with a receipt.
//! Blocks print in item-start order, each once it is complete; agent text streams line by
//! line, so every prefix still prints exactly once. Model and tool text can never inject
//! terminal controls: every control character is dropped before styling is applied.
//! Structure translated from bboxy450-term lab/web/harness.js (the web reference).
//! https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use super::face_core::{self as core, Tok, Verdict};
use crate::protocol::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// Trusted OSC 7501 metadata, kept separate from the escape-stripping transcript.
/// Text fields use term's base64 grammar and UTF-8 byte limits.
#[derive(Default)]
pub struct StateReporter {
    title: String,
    turns: BTreeSet<TurnId>,
    permissions: BTreeMap<ItemId, String>,
    error: Option<String>,
}
impl StateReporter {
    pub fn report(&self) -> Vec<u8> {
        let (state, detail) = if let Some(error) = &self.error {
            ("error", Some(error.as_str()))
        } else if let Some(subject) = self.permissions.values().next() {
            ("blocked", Some(subject.as_str()))
        } else if self.turns.is_empty() {
            ("idle", None)
        } else {
            ("working", None)
        };
        let mut text = format!("\x1b]7501;state={state}");
        if !self.title.is_empty() {
            text.push_str(&format!(":title={}", encode_text(&self.title, 192)));
        }
        if let Some(detail) = detail {
            text.push_str(&format!(":msg={}", encode_text(detail, 2048)));
        }
        if state == "blocked" {
            text.push_str(":kind=permission");
        }
        text.push('\x07');
        text.into_bytes()
    }
    pub fn apply(&mut self, event: &Event) -> Option<Vec<u8>> {
        let before = self.report();
        let force = matches!(event, Event::TurnStarted { .. } | Event::TurnEnded { .. });
        match event {
            Event::ProfileChanged { profile, model } => self.title = format!("{profile} · {model}"),
            Event::TurnStarted { turn, .. } => {
                self.turns.insert(*turn);
                self.error = None;
            }
            Event::TurnEnded { turn, reason, .. } => {
                self.turns.remove(turn);
                self.error = match reason {
                    StopReason::Error { message, .. } => Some(message.clone()),
                    _ => None,
                };
            }
            Event::ItemSnapshot {
                item,
                partial:
                    Item::Approval {
                        subject,
                        decision: None,
                        ..
                    },
            } => {
                self.permissions.insert(*item, subject.summary.clone());
            }
            Event::Status {
                phase: Phase::Approval { item },
            } => {
                self.permissions.entry(*item).or_default();
            }
            Event::ItemCompleted {
                item,
                body: Item::Approval { .. },
                ..
            } => {
                self.permissions.remove(item);
            }
            _ => {}
        }
        let after = self.report();
        (force || before != after).then_some(after)
    }
}
fn encode_text(text: &str, limit: usize) -> String {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    STANDARD.encode(&text.as_bytes()[..end])
}

pub const STATE_PROBE: &[u8] = b"\x1b]7501;?\x07";
pub enum InputAction {
    Line(String),
    Probe,
    Echo(Vec<u8>),
    Eof,
}
/// A probe may split at any read boundary or interrupt a partially typed line.
/// Only the exact probe is consumed; ordinary escape bytes retain their input meaning.
#[derive(Default)]
pub struct InputFilter {
    line: Vec<u8>,
    probe: usize,
}
impl InputFilter {
    pub fn interrupt(&mut self) {
        self.line.clear();
        self.probe = 0;
    }
    pub fn feed(&mut self, bytes: &[u8], interactive: bool) -> Vec<InputAction> {
        let mut actions = vec![];
        for &byte in bytes {
            if byte == STATE_PROBE[self.probe] {
                self.probe += 1;
                if self.probe == STATE_PROBE.len() {
                    self.probe = 0;
                    actions.push(InputAction::Probe);
                }
                continue;
            }
            let prefix = self.probe;
            self.probe = 0;
            for &byte in &STATE_PROBE[..prefix] {
                self.ordinary(byte, interactive, &mut actions);
            }
            if byte == STATE_PROBE[0] {
                self.probe = 1;
            } else {
                self.ordinary(byte, interactive, &mut actions);
            }
        }
        actions
    }
    fn ordinary(&mut self, byte: u8, interactive: bool, actions: &mut Vec<InputAction>) {
        match byte {
            b'\n' => {
                if self.line.last() == Some(&b'\r') {
                    self.line.pop();
                }
                actions.push(InputAction::Line(
                    String::from_utf8_lossy(&self.line).into_owned(),
                ));
                self.line.clear();
                if interactive {
                    actions.push(InputAction::Echo(vec![b'\n']));
                }
            }
            4 if interactive => {
                if self.line.is_empty() {
                    actions.push(InputAction::Eof);
                }
            }
            8 | 127 if interactive => {
                if let Some(mut last) = self.line.pop() {
                    while last & 0xc0 == 0x80 {
                        let Some(byte) = self.line.pop() else {
                            break;
                        };
                        last = byte;
                    }
                    actions.push(InputAction::Echo(b"\x08 \x08".to_vec()));
                }
            }
            _ => {
                self.line.push(byte);
                if interactive && (byte >= 0x20 && byte != 0x7f || byte == b'\t') {
                    actions.push(InputAction::Echo(vec![byte]));
                }
            }
        }
    }
    pub fn finish(&mut self) -> Vec<InputAction> {
        self.line.extend_from_slice(&STATE_PROBE[..self.probe]);
        self.probe = 0;
        let mut actions = vec![];
        if !self.line.is_empty() {
            actions.push(InputAction::Line(
                String::from_utf8_lossy(&self.line).into_owned(),
            ));
            self.line.clear();
        }
        actions.push(InputAction::Eof);
        actions
    }
}

/// Noncanonical input makes a probe readable without Enter. ISIG stays enabled
/// for Ctrl-C; this face supplies line echo/editing and restores termios on exit.
pub struct TerminalInput(libc::termios);
impl TerminalInput {
    pub fn enable() -> std::io::Result<Self> {
        let mut original = std::mem::MaybeUninit::uninit();
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, original.as_mut_ptr()) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let original = unsafe { original.assume_init() };
        let mut input = original;
        input.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ECHONL);
        input.c_cc[libc::VMIN] = 1;
        input.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &input) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(original))
    }
}
impl Drop for TerminalInput {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.0);
        }
    }
}

/// How many output lines a card body shows before "… N more lines".
const BODY: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    None,
    Ansi256,
    Truecolor,
}
#[derive(Clone, Debug)]
pub struct Style {
    pub color: ColorMode,
    /// Columns; 0 asks the terminal (TIOCGWINSZ, then COLUMNS) before every block.
    pub width: usize,
    /// The terminal already echoed the user's line, so a user block starts on a fresh line.
    pub echo: bool,
}
impl Default for Style {
    /// Deterministic plain text at 100 columns: goldens and pipes.
    fn default() -> Self {
        Self {
            color: ColorMode::None,
            width: 100,
            echo: false,
        }
    }
}
impl Style {
    /// Truecolor when COLORTERM says so, else 256 colours; none for NO_COLOR, TERM=dumb or
    /// a non-terminal stdout (FORCE_COLOR/CLICOLOR_FORCE override that last one).
    pub fn detect(echo: bool) -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let tty = unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1;
        let forced = var("FORCE_COLOR").is_some_and(|v| v != "0")
            || var("CLICOLOR_FORCE").is_some_and(|v| v != "0");
        let color = if var("NO_COLOR").is_some()
            || var("TERM").as_deref() == Some("dumb")
            || !(tty || forced)
        {
            ColorMode::None
        } else if matches!(var("COLORTERM").as_deref(), Some("truecolor" | "24bit")) {
            ColorMode::Truecolor
        } else {
            ColorMode::Ansi256
        };
        Self {
            color,
            width: 0,
            echo,
        }
    }
}
/// The terminal's width: TIOCGWINSZ on stdout, then COLUMNS, then 100.
pub fn terminal_width() -> usize {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col > 0
    {
        return ws.ws_col as usize;
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.trim().parse().ok())
        .filter(|&c: &usize| c > 0)
        .unwrap_or(100)
}

// ---------- palette (the web reference's dark tokens) ----------
type Rgb = (u8, u8, u8);
const fn rgb(h: u32) -> Rgb {
    ((h >> 16) as u8, (h >> 8) as u8, h as u8)
}
fn hex(s: &str) -> Rgb {
    rgb(u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0xFFFFFF))
}
fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let m = |x: u8, y: u8| (x as f32 * t + y as f32 * (1. - t)).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}
const FG: Rgb = rgb(0xE6E9EF);
const BRIGHT: Rgb = rgb(0xFFFFFF);
const DIM: Rgb = rgb(0x7D8597);
const FAINT: Rgb = rgb(0x4A5163);
const RAIL: Rgb = rgb(0x3F475A);
const SEAT: Rgb = rgb(0x2EE6A8);
const OK: Rgb = rgb(0x2EE6A8);
const BAD: Rgb = rgb(0xFF5D5D);
const ASK: Rgb = rgb(0xFFC53D);
const TERM_OK: Rgb = rgb(0x5BE08C);
const TERM_BAD: Rgb = rgb(0xFF7A7A);
const USER_BG: Rgb = rgb(0x1C2130);
const CODE_BG: Rgb = rgb(0x141720);
const CHIP_BG: Rgb = rgb(0x262B38);
const ADD_BG: Rgb = rgb(0x0F2A1E);
const DEL_BG: Rgb = rgb(0x33161A);
const THINK: Rgb = rgb(0xB07BFF);
fn tok_color(t: Tok) -> (Rgb, bool) {
    let tint = |h: u32, t: f32| mix(rgb(h), FG, t);
    match t {
        Tok::Comment => (DIM, true),
        Tok::Attr => (tint(0xD4A017, 0.75), false),
        Tok::Str => (tint(0x2EC46A, 0.7), false),
        Tok::Num => (tint(0xFF8A2B, 0.75), false),
        Tok::Macro => (tint(0x1FA8E0, 0.75), false),
        Tok::Keyword => (tint(0xFF4FA3, 0.72), false),
        Tok::Type => (tint(0x9B5CFF, 0.72), false),
        Tok::Plain => (FG, false),
    }
}

// ---------- spans and lines ----------
#[derive(Clone, Default)]
struct Sp {
    t: String,
    fg: Option<Rgb>,
    bg: Option<Rgb>,
    bold: bool,
    ital: bool,
    link: Option<String>,
}
fn sp(t: impl Into<String>) -> Sp {
    Sp {
        t: clean(&t.into()),
        ..Default::default()
    }
}
impl Sp {
    fn fg(mut self, c: Rgb) -> Self {
        self.fg = Some(c);
        self
    }
    fn bg(mut self, c: Rgb) -> Self {
        self.bg = Some(c);
        self
    }
    fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    fn ital(mut self) -> Self {
        self.ital = true;
        self
    }
    fn same(&self, o: &Sp) -> bool {
        (self.fg, self.bg, self.bold, self.ital, &self.link)
            == (o.fg, o.bg, o.bold, o.ital, &o.link)
    }
}
type Line = Vec<Sp>;
/// Tabs become spaces; every other control character (C0, DEL, C1, ESC) is dropped.
fn clean(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\t' => out.push_str("    "),
            c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {}
            c => out.push(c),
        }
    }
    out
}
fn cw(c: char) -> usize {
    let u = c as u32;
    if (0x300..0x370).contains(&u)
        || (0x200B..=0x200F).contains(&u)
        || (0xFE00..=0xFE0F).contains(&u)
        || (0x20D0..0x2100).contains(&u) && u < 0x20F0
    {
        0
    } else if (0x1100..=0x115F).contains(&u)
        || (0x2E80..=0xA4CF).contains(&u) && u != 0x303F
        || (0xAC00..=0xD7A3).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0xFE30..=0xFE4F).contains(&u)
        || (0xFF00..=0xFF60).contains(&u)
        || (0xFFE0..=0xFFE6).contains(&u)
        || (0x1F300..=0x1F64F).contains(&u)
        || (0x1F900..=0x1F9FF).contains(&u)
        || (0x20000..=0x3FFFD).contains(&u)
    {
        2
    } else {
        1
    }
}
fn sw(s: &str) -> usize {
    s.chars().map(cw).sum()
}
fn lw(l: &[Sp]) -> usize {
    l.iter().map(|s| sw(&s.t)).sum()
}
/// Cut a line to `width` columns, ending in "…" when anything was dropped.
fn fit(l: Line, width: usize) -> Line {
    if lw(&l) <= width {
        return l;
    }
    let mut out: Line = vec![];
    let mut used = 0;
    let room = width.saturating_sub(1);
    'outer: for s in l {
        let mut t = String::new();
        for c in s.t.chars() {
            if used + cw(c) > room {
                let mut s2 = s.clone();
                s2.t = t;
                out.push(s2);
                break 'outer;
            }
            used += cw(c);
            t.push(c);
        }
        let mut s2 = s;
        s2.t = t;
        out.push(s2);
    }
    let style = out.last().cloned().unwrap_or_default();
    out.push(Sp {
        t: "…".into(),
        link: None,
        ..style
    });
    out
}
/// Word-wrap spans to `width`, with a first-line prefix and a continuation prefix.
fn wrap(spans: Line, width: usize, first: Line, rest: Line) -> Vec<Line> {
    // pieces: a word with its trailing spaces, carrying its span's style
    let mut pieces: Vec<Sp> = vec![];
    for s in spans {
        let mut cur = String::new();
        let mut in_space = false;
        for c in s.t.chars() {
            if c == ' ' {
                in_space = true;
            } else if in_space {
                pieces.push(Sp {
                    t: std::mem::take(&mut cur),
                    ..s.clone()
                });
                in_space = false;
            }
            cur.push(c);
        }
        if !cur.is_empty() {
            pieces.push(Sp { t: cur, ..s });
        }
    }
    let mut lines: Vec<Line> = vec![];
    let mut line = first.clone();
    let mut start = lw(&first);
    let mut used = start;
    let mut fresh = true;
    let push = |line: &mut Line, p: Sp| match line.last_mut() {
        Some(last) if last.same(&p) => last.t.push_str(&p.t),
        _ => line.push(p),
    };
    for mut p in pieces {
        if fresh {
            let t = p.t.trim_start_matches(' ').to_owned();
            if t.is_empty() {
                continue;
            }
            p.t = t;
        }
        let pw = sw(p.t.trim_end_matches(' '));
        if used + pw > width && !fresh {
            trim_end(&mut line);
            lines.push(std::mem::replace(&mut line, rest.clone()));
            start = lw(&rest);
            used = start;
            p.t = p.t.trim_start_matches(' ').to_owned();
        }
        // a word wider than the line is split by columns
        while used + sw(p.t.trim_end_matches(' ')) > width && width > start {
            let mut head = String::new();
            let mut w = used;
            let mut cut = 0;
            for (i, c) in p.t.char_indices() {
                if w + cw(c) > width {
                    cut = i;
                    break;
                }
                w += cw(c);
                head.push(c);
            }
            if head.is_empty() {
                break;
            }
            push(
                &mut line,
                Sp {
                    t: head,
                    ..p.clone()
                },
            );
            lines.push(std::mem::replace(&mut line, rest.clone()));
            used = start;
            p.t = p.t[cut..].to_owned();
        }
        used += sw(&p.t);
        fresh = false;
        push(&mut line, p);
    }
    trim_end(&mut line);
    if !fresh || lines.is_empty() {
        lines.push(line);
    }
    lines
}
fn trim_end(l: &mut Line) {
    while let Some(last) = l.last_mut() {
        let t = last.t.trim_end_matches(' ').len();
        if t == 0 && last.bg.is_none() {
            l.pop();
        } else {
            if last.bg.is_none() {
                last.t.truncate(t);
            }
            break;
        }
    }
}
/// Split one line into rows of exactly `width` columns (code: no word boundaries).
fn hard_wrap(spans: Line, width: usize, prefix: Line, cont: Line) -> Vec<Line> {
    let mut lines = vec![];
    let mut line = prefix.clone();
    let start = lw(&cont);
    let mut used = lw(&prefix);
    for s in spans {
        let mut t = String::new();
        for c in s.t.chars() {
            if used + cw(c) > width && used > start {
                line.push(Sp {
                    t: std::mem::take(&mut t),
                    ..s.clone()
                });
                lines.push(std::mem::replace(&mut line, cont.clone()));
                used = start;
            }
            used += cw(c);
            t.push(c);
        }
        line.push(Sp { t, ..s });
    }
    lines.push(line);
    lines
}

// ---------- the painter ----------
struct Pen {
    mode: ColorMode,
    width: usize,
}
impl Pen {
    fn color(&self, c: Rgb, bg: bool) -> String {
        let base = if bg { 48 } else { 38 };
        match self.mode {
            ColorMode::Truecolor => format!(";{base};2;{};{};{}", c.0, c.1, c.2),
            ColorMode::Ansi256 => format!(";{base};5;{}", ansi256(c)),
            ColorMode::None => String::new(),
        }
    }
    /// One output row. `fill` paints the rest of the row (and unpainted spans) as a block.
    fn row(&self, l: &[Sp], fill: Option<Rgb>) -> String {
        let mut out = String::new();
        if self.mode == ColorMode::None {
            for s in l {
                match &s.link {
                    Some(url) if !s.t.contains(url.as_str()) => {
                        out.push_str(&s.t);
                        out.push_str(&format!(" ({url})"));
                    }
                    _ => out.push_str(&s.t),
                }
            }
            let t = out.trim_end().len();
            out.truncate(t);
            out.push('\n');
            return out;
        }
        for s in l {
            if s.t.is_empty() {
                continue;
            }
            let mut sgr = String::from("\x1b[0");
            if s.bold {
                sgr.push_str(";1");
            }
            if s.ital {
                sgr.push_str(";3");
            }
            if let Some(c) = s.fg {
                sgr.push_str(&self.color(c, false));
            }
            if let Some(c) = s.bg.or(fill) {
                sgr.push_str(&self.color(c, true));
            }
            sgr.push('m');
            out.push_str(&sgr);
            if let Some(url) = &s.link {
                out.push_str(&format!(
                    "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                    clean(url),
                    s.t
                ));
            } else {
                out.push_str(&s.t);
            }
        }
        if let Some(c) = fill {
            let pad = self.width.saturating_sub(lw(l));
            if pad > 0 {
                out.push_str(&format!(
                    "\x1b[0{}m{}",
                    self.color(c, true),
                    " ".repeat(pad)
                ));
            }
        }
        out.push_str("\x1b[0m\n");
        out
    }
}
/// Nearest xterm-256 colour: the 6×6×6 cube or the grey ramp, whichever is closer.
fn ansi256(c: Rgb) -> u8 {
    const L: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let level = |v: u8| {
        (0..6)
            .min_by_key(|&i| (L[i] as i32 - v as i32).abs())
            .unwrap_or(0)
    };
    let (r, g, b) = (level(c.0), level(c.1), level(c.2));
    let dist = |x: (u8, u8, u8)| {
        let f = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
        f(x.0, c.0) + f(x.1, c.1) + f(x.2, c.2)
    };
    let avg = (c.0 as u32 + c.1 as u32 + c.2 as u32) / 3;
    let gi = (avg.saturating_sub(3) / 10).min(23) as u8;
    let gv = 8 + gi * 10;
    if dist((gv, gv, gv)) < dist((L[r], L[g], L[b])) {
        232 + gi
    } else {
        16 + 36 * r as u8 + 6 * g as u8 + b as u8
    }
}

// ---------- markdown for the terminal ----------
#[derive(Default)]
struct Md {
    code: Option<String>,
    blank: bool,
}
/// Inline markdown: `code`, **bold**, *italic*, [text](url).
fn inline(src: &str, base: &Sp, plain: bool) -> Line {
    let mut out: Line = vec![];
    let mut cur = String::new();
    let mut bold = false;
    let mut ital = false;
    let style = |bold: bool, ital: bool| {
        let mut s = base.clone();
        s.bold |= bold;
        s.ital |= ital;
        if bold && s.fg.is_none() {
            s.fg = Some(BRIGHT);
        }
        s
    };
    let flush = |out: &mut Line, cur: &mut String, bold: bool, ital: bool| {
        if !cur.is_empty() {
            out.push(Sp {
                t: clean(&std::mem::take(cur)),
                ..style(bold, ital)
            });
        }
    };
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < c.len() {
        let rest: String = c[i..].iter().collect();
        if c[i] == '`'
            && let Some(end) = c[i + 1..].iter().position(|&x| x == '`')
        {
            flush(&mut out, &mut cur, bold, ital);
            let code: String = c[i + 1..i + 1 + end].iter().collect();
            out.push(if plain {
                sp(format!("`{code}`"))
            } else {
                sp(code).fg(BRIGHT).bg(CHIP_BG)
            });
            i += end + 2;
            continue;
        }
        if rest.starts_with("**") && (bold || rest[2..].contains("**")) {
            flush(&mut out, &mut cur, bold, ital);
            bold = !bold;
            i += 2;
            continue;
        }
        if c[i] == '*'
            && (ital
                || (c.get(i + 1).is_some_and(|n| !n.is_whitespace())
                    && c[i + 1..].contains(&'*')
                    && (i == 0 || !c[i - 1].is_alphanumeric())))
        {
            flush(&mut out, &mut cur, bold, ital);
            ital = !ital;
            i += 1;
            continue;
        }
        if c[i] == '['
            && let Some(close) = rest.find("](")
            && let Some(end) = rest[close..].find(')')
        {
            let label = &rest[1..close];
            let url = &rest[close + 2..close + end];
            if !label.contains(']')
                && (url.starts_with("http://") || url.starts_with("https://"))
                && !url.contains(' ')
            {
                flush(&mut out, &mut cur, bold, ital);
                let mut s = style(bold, ital);
                s.t = clean(label);
                s.fg = Some(rgb(0x58D7FF));
                s.link = Some(url.to_owned());
                out.push(s);
                i += rest[..close + end + 1].chars().count();
                continue;
            }
        }
        cur.push(c[i]);
        i += 1;
    }
    flush(&mut out, &mut cur, bold, ital);
    out
}
impl Md {
    fn line(&mut self, src: &str, pen: &Pen) -> String {
        let w = pen.width;
        let plain = pen.mode == ColorMode::None;
        let mut out = String::new();
        let trimmed = src.trim_start();
        if let Some(fence) = trimmed.strip_prefix("```") {
            if self.code.take().is_some() {
                if !plain {
                    out += &pen.row(&[sp("  ")], Some(CODE_BG));
                }
            } else {
                let lang = fence.trim().split(' ').next().unwrap_or("").to_owned();
                out += &if plain {
                    pen.row(&[sp("  ```"), sp(lang.clone())], None)
                } else {
                    pen.row(&[sp("  "), sp(lang.clone()).fg(DIM).ital()], Some(CODE_BG))
                };
                self.code = Some(lang);
            }
            self.blank = false;
            return out;
        }
        if self.code.is_some() {
            let runs: Line = core::tokens(&clean(src))
                .into_iter()
                .map(|(t, k)| {
                    let (c, i) = tok_color(k);
                    let s = Sp {
                        t,
                        ..Default::default()
                    }
                    .fg(c);
                    if i { s.ital() } else { s }
                })
                .collect();
            for l in hard_wrap(
                runs,
                w.saturating_sub(1),
                vec![sp("    ")],
                vec![sp("    ")],
            ) {
                out += &pen.row(&l, (!plain).then_some(CODE_BG));
            }
            self.blank = false;
            return out;
        }
        if trimmed.is_empty() {
            if !self.blank {
                out.push('\n');
            }
            self.blank = true;
            return out;
        }
        self.blank = false;
        let base = Sp::default();
        let heading = trimmed
            .find(|c| c != '#')
            .filter(|&n| (1..=4).contains(&n) && trimmed[n..].starts_with(' '));
        if let Some(level) = heading {
            let mut text = inline(trimmed[level + 1..].trim(), &base, plain);
            for s in &mut text {
                s.bold = true;
                if s.bg.is_none() {
                    s.fg = Some(if level <= 2 { SEAT } else { BRIGHT });
                }
            }
            let mark = if plain {
                sp(format!("{} ", "#".repeat(level)))
            } else {
                sp(if level == 1 { "▍" } else { "" }).fg(SEAT)
            };
            for l in wrap(text, w, vec![sp("  "), mark], vec![sp("  ")]) {
                out += &pen.row(&l, None);
            }
            return out;
        }
        let indent = (src.len() - trimmed.len()) / 2;
        let pad = "  ".repeat(indent + 1);
        let bullet = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "));
        let ordered = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .filter(|&n| n > 0 && n < 4)
            .filter(|&n| trimmed[n..].starts_with(". ") || trimmed[n..].starts_with(") "))
            .map(|n| (&trimmed[..n], &trimmed[n + 2..]));
        if let Some(item) = bullet {
            let (mark, item) = if let Some(rest) = item.strip_prefix("[ ] ") {
                ("☐ ", rest)
            } else if let Some(rest) = item
                .strip_prefix("[x] ")
                .or_else(|| item.strip_prefix("[X] "))
            {
                ("☑ ", rest)
            } else {
                (if indent == 0 { "• " } else { "◦ " }, item)
            };
            for l in wrap(
                inline(item, &base, plain),
                w,
                vec![sp(pad.clone()), sp(mark).fg(SEAT)],
                vec![sp(format!("{pad}  "))],
            ) {
                out += &pen.row(&l, None);
            }
            return out;
        }
        if let Some((n, item)) = ordered {
            let mark = format!("{n}. ");
            let cont = format!("{pad}{}", " ".repeat(mark.len()));
            for l in wrap(
                inline(item, &base, plain),
                w,
                vec![sp(pad), sp(mark).fg(SEAT)],
                vec![sp(cont)],
            ) {
                out += &pen.row(&l, None);
            }
            return out;
        }
        if let Some(q) = trimmed.strip_prefix('>') {
            let q = q.strip_prefix(' ').unwrap_or(q);
            let mut base = base.clone();
            base.ital = true;
            base.fg = Some(DIM);
            for l in wrap(
                inline(q, &base, plain),
                w,
                vec![sp("  "), sp("▎ ").fg(FAINT)],
                vec![sp("  "), sp("▎ ").fg(FAINT)],
            ) {
                out += &pen.row(&l, None);
            }
            return out;
        }
        if trimmed.len() >= 3
            && trimmed.chars().all(|c| matches!(c, '-' | '*' | '_' | ' '))
            && trimmed.chars().filter(|c| *c != ' ').count() >= 3
        {
            let n = w.saturating_sub(5).min(60);
            out += &pen.row(&[sp("  "), sp("─".repeat(n)).fg(FAINT)], None);
            return out;
        }
        for l in wrap(
            inline(trimmed, &base, plain),
            w,
            vec![sp("  ")],
            vec![sp("  ")],
        ) {
            out += &pen.row(&l, None);
        }
        out
    }
}

// ---------- the projection ----------
#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    None,
    User,
    Prose,
    Card,
    Receipt,
}
struct Entry {
    kind: ItemKind,
    turn: Option<TurnId>,
    parent: Option<ItemId>,
    head: Option<ItemHead>,
    t0: u64,
    t1: Option<u64>,
    body: Option<Item>,
    status: Option<ItemStatus>,
    /// streamed text: agent text, reasoning, or terminal output
    text: String,
}
#[derive(Default)]
struct Streaming {
    emitted: String,
    md: Md,
}
#[derive(Default)]
struct TurnStat {
    t0: Option<u64>,
    rounds: usize,
    tools: usize,
    edits: usize,
}

/// Tracks emitted prefixes and open items. The bytes for a given envelope sequence are a
/// pure function of that sequence and the style, shared by live streams and replay.
pub struct LineProjection {
    style: Style,
    cwd: String,
    items: HashMap<ItemId, Entry>,
    queue: VecDeque<ItemId>,
    streams: HashMap<ItemId, Streaming>,
    kids: HashMap<ItemId, Vec<ItemId>>,
    printed: HashSet<ItemId>,
    turns: HashMap<TurnId, TurnStat>,
    running: bool,
    at_prompt: bool,
    family: Family,
    /// the last block was more than one row (a card with a body)
    tall: bool,
    /// exec cards already opened: their header printed, output streaming under the rail
    open: HashMap<ItemId, Open>,
}
/// A running command's card: output lines shown so far and lines held back.
#[derive(Default)]
struct Open {
    taken: usize,
    shown: usize,
    hidden: usize,
}
impl Default for LineProjection {
    fn default() -> Self {
        Self::new(Style::default())
    }
}
impl LineProjection {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            cwd: String::new(),
            items: HashMap::new(),
            queue: VecDeque::new(),
            streams: HashMap::new(),
            kids: HashMap::new(),
            printed: HashSet::new(),
            turns: HashMap::new(),
            running: false,
            at_prompt: false,
            family: Family::None,
            tall: false,
            open: HashMap::new(),
        }
    }
    fn pen(&self) -> Pen {
        let width = if self.style.width == 0 {
            terminal_width()
        } else {
            self.style.width
        };
        // stop one column short: writing the last column makes many terminals wrap early
        Pen {
            mode: self.style.color,
            width: width.clamp(24, 400) - 1,
        }
    }

    pub fn render(&mut self, envelope: &Envelope) -> Vec<u8> {
        let mut out = String::new();
        let ts = envelope.ts_ms;
        match &envelope.event {
            Event::SessionStarted { meta } => {
                self.cwd = meta.cwd.to_string_lossy().into_owned();
                let pen = self.pen();
                let l = vec![
                    sp("session ").fg(DIM),
                    sp(meta.session.to_string()).fg(FG),
                    sp(" · ").fg(FAINT),
                    sp(meta.profile.as_deref().unwrap_or("unconfigured"))
                        .fg(BRIGHT)
                        .bold(),
                ];
                let rows: String = wrap(l, pen.width, vec![], vec![sp("  ")])
                    .iter()
                    .map(|l| pen.row(l, None))
                    .collect();
                self.block(&mut out, Family::None, false, rows);
            }
            Event::ProfileChanged { profile, model } => {
                let pen = self.pen();
                let l = match acp_model(model) {
                    // an ACP agent: its round tile, `profile · model`, then the ACP tag
                    Some((agent, m)) => {
                        let (mark, color) = agent_mark(&agent);
                        let mut l = vec![
                            sp("  "),
                            sp(format!(" {mark} ")).fg(BRIGHT).bg(color).bold(),
                            sp(" "),
                            sp(profile.clone()).fg(BRIGHT).bold(),
                        ];
                        if !m.is_empty() {
                            l.push(sp(format!(" · {m}")).fg(DIM));
                        }
                        l.push(sp(" "));
                        l.push(sp(" ACP ").fg(mix(SEAT, FG, 0.85)).bg(rgb(0x16313C)).bold());
                        l
                    }
                    None => vec![
                        sp("  ● ").fg(SEAT),
                        sp(profile.clone()).fg(BRIGHT).bold(),
                        sp(" · ").fg(FAINT),
                        sp(model.clone()).fg(DIM),
                    ],
                };
                self.block(
                    &mut out,
                    Family::Card,
                    false,
                    pen.row(&fit(l, pen.width), None),
                );
            }
            Event::TurnStarted { turn, .. } => {
                self.running = true;
                self.turns.entry(*turn).or_default().t0 = Some(ts);
            }
            Event::ItemStarted {
                turn,
                item,
                kind,
                parent,
                head,
            } => {
                if let Some(t) = turn.and_then(|t| self.turns.get_mut(&t)) {
                    match kind {
                        ItemKind::Reasoning => t.rounds += 1,
                        ItemKind::ToolCall => t.tools += 1,
                        _ => {}
                    }
                }
                self.items.insert(
                    *item,
                    Entry {
                        kind: kind.clone(),
                        turn: *turn,
                        parent: *parent,
                        head: head.clone(),
                        t0: ts,
                        t1: None,
                        body: None,
                        status: None,
                        text: String::new(),
                    },
                );
                let foldable = matches!(kind, ItemKind::Terminal | ItemKind::FileChange)
                    && parent.is_some_and(|p| self.is_open_tool(&p));
                if foldable {
                    self.kids.entry(parent.unwrap()).or_default().push(*item);
                    self.pump(&mut out);
                } else if matches!(
                    kind,
                    ItemKind::UserMessage
                        | ItemKind::AgentMessage
                        | ItemKind::Reasoning
                        | ItemKind::ToolCall
                        | ItemKind::Plan
                        | ItemKind::Notice
                        | ItemKind::Terminal
                        | ItemKind::FileChange
                        | ItemKind::Command
                        | ItemKind::Hook
                        | ItemKind::SystemMessage
                        | ItemKind::Native
                        | ItemKind::Subagent
                        | ItemKind::Usage
                ) || kind.is_unknown()
                {
                    self.queue.push_back(*item);
                }
            }
            Event::ItemDelta { item, delta } => {
                let Some(e) = self.items.get_mut(item) else {
                    return vec![];
                };
                match delta {
                    Delta::Text { append }
                    | Delta::Reasoning { append }
                    | Delta::Output { append, .. } => e.text.push_str(append),
                    _ => {}
                }
                let (kind, parent) = (e.kind.clone(), e.parent);
                if let (ItemKind::Terminal, Some(tool), Delta::Output { append, .. }) =
                    (kind.clone(), parent, delta)
                    && let Some(t) = self.items.get_mut(&tool)
                    && t.kind == ItemKind::ToolCall
                {
                    t.text.push_str(append);
                }
                if kind == ItemKind::AgentMessage {
                    self.unblock_for(item, &mut out);
                }
                self.pump(&mut out);
            }
            Event::ItemSnapshot { item, partial } => {
                if let Some(e) = self.items.get_mut(item)
                    && let Item::AgentMessage { markdown } = partial
                {
                    e.text = markdown.clone();
                    self.unblock_for(item, &mut out);
                    self.pump(&mut out);
                }
            }
            Event::ItemCompleted { item, status, body } => {
                self.complete(*item, status, body, ts, &mut out);
            }
            Event::ContextChanged {
                mode,
                permission_mode,
                config,
            } => {
                self.notice_line(
                    &mut out,
                    &format!(
                        "Context: {} · {} · {} settings",
                        mode.as_deref().unwrap_or("unreported"),
                        permission_mode.as_deref().unwrap_or("unreported"),
                        config.as_ref().map_or(0, |c| c.len())
                    ),
                    &Level::Info,
                );
            }
            Event::TurnEnded {
                turn,
                reason,
                usage,
            } => {
                while let Some(id) = self.queue.pop_front() {
                    self.print_item(id, &mut out);
                }
                let stat = self.turns.remove(turn).unwrap_or_default();
                let pen = self.pen();
                let row = receipt(&pen, &stat, ts, reason, usage.as_ref());
                self.running = false;
                self.block(&mut out, Family::Receipt, false, row);
                self.prompt(&mut out);
            }
            Event::OpRejected { reason, .. } => {
                let pen = self.pen();
                let l = vec![
                    sp("  ✗ ").fg(BAD),
                    sp(format!("rejected: {reason:?}")).fg(BAD),
                ];
                self.block(&mut out, Family::Card, false, pen.row(&l, None));
            }
            Event::SessionRecovered {
                closed_items,
                closed_turns,
                ..
            } => {
                let t = format!("recovered {closed_items} items, {closed_turns} turns");
                self.notice_line(&mut out, &t, &Level::Info);
            }
            Event::Lagged { resume_after } => {
                let t = format!("transcript lagged; reconnect after {}", resume_after.0);
                self.notice_line(&mut out, &t, &Level::Warn);
            }
            Event::Reset { head, .. } => {
                let t = format!("transcript reset to {}", head.0);
                self.notice_line(&mut out, &t, &Level::Warn);
            }
            Event::Unknown(raw) => self.notice_line(
                &mut out,
                &format!("Unsupported event: {}", unknown_tag(raw)),
                &Level::Info,
            ),
            _ => {}
        }
        if !self.running && !self.at_prompt && !out.is_empty() {
            self.prompt(&mut out);
        }
        out.into_bytes()
    }

    fn prompt(&mut self, out: &mut String) {
        out.push_str(&if self.style.color == ColorMode::None {
            "› ".to_owned()
        } else {
            format!("\x1b[0;1{}m› \x1b[0m", self.pen().color(SEAT, false))
        });
        self.at_prompt = true;
        self.family = Family::None;
    }
    /// Emit one block, with a blank line between prose and cards, and leave the prompt line.
    fn block(&mut self, out: &mut String, fam: Family, user: bool, text: String) {
        if text.is_empty() {
            return;
        }
        if self.at_prompt {
            if !(user && self.style.echo) {
                out.push('\n');
            }
            self.at_prompt = false;
        }
        let gap = self.family != Family::None
            && (fam != self.family || fam == Family::User || (fam == Family::Card && self.tall));
        if gap || (fam == Family::User && user && self.family != Family::None) {
            out.push('\n');
        }
        self.tall = text.trim_end_matches('\n').contains('\n');
        out.push_str(&text);
        self.family = fam;
    }
    fn is_open_tool(&self, id: &ItemId) -> bool {
        self.items
            .get(id)
            .is_some_and(|e| e.kind == ItemKind::ToolCall && e.body.is_none())
            && !self.printed.contains(id)
    }
    /// Agent text streams while an earlier reasoning item is still open: show the thought
    /// now, with what is known, so the text is not held back.
    fn unblock_for(&mut self, item: &ItemId, out: &mut String) {
        while let Some(head) = self.queue.front().copied() {
            if head == *item {
                break;
            }
            let Some(e) = self.items.get(&head) else {
                self.queue.pop_front();
                continue;
            };
            if e.kind == ItemKind::Reasoning && e.body.is_none() {
                self.queue.pop_front();
                self.print_item(head, out);
            } else {
                break;
            }
        }
    }
    fn complete(
        &mut self,
        item: ItemId,
        status: &ItemStatus,
        body: &Item,
        ts: u64,
        out: &mut String,
    ) {
        if self.printed.contains(&item) {
            return;
        }
        let e = self.items.entry(item).or_insert_with(|| Entry {
            kind: kind_of(body),
            turn: None,
            parent: None,
            head: None,
            t0: ts,
            t1: None,
            body: None,
            status: None,
            text: String::new(),
        });
        e.t1 = Some(ts);
        e.status = Some(status.clone());
        e.body = Some(body.clone());
        let (turn, parent) = (e.turn, e.parent);
        match body {
            Item::Approval {
                subject, decision, ..
            } => {
                let tool = Some(subject.tool_call).filter(|t| self.is_open_tool(t));
                if decision.is_none() {
                    // an open question prints now: it is waiting for someone
                    let text = self.approval_card(item);
                    self.block(out, Family::Card, false, text);
                    self.printed.insert(item);
                } else if let Some(tool) = tool.or(parent.filter(|p| self.is_open_tool(p))) {
                    self.kids.entry(tool).or_default().push(item);
                } else {
                    self.queue.push_back(item);
                }
            }
            Item::FileChange { .. } => {
                if let Some(t) = turn.and_then(|t| self.turns.get_mut(&t)) {
                    t.edits += 1;
                }
            }
            _ => {}
        }
        if !self.queue.contains(&item) && kind_of(body) == ItemKind::Notice {
            self.queue.push_back(item);
        }
        self.pump(out);
    }
    /// Print every complete block at the head of the queue; stream the head's agent text.
    fn pump(&mut self, out: &mut String) {
        while let Some(head) = self.queue.front().copied() {
            let Some(e) = self.items.get(&head) else {
                self.queue.pop_front();
                continue;
            };
            if e.body.is_some() {
                self.queue.pop_front();
                self.print_item(head, out);
            } else if e.kind == ItemKind::AgentMessage {
                self.stream(head, false, out);
                break;
            } else {
                if e.kind == ItemKind::ToolCall
                    && (self.kid(&head, ItemKind::Terminal).is_some() || !e.text.is_empty())
                {
                    self.run_card(head, false, out);
                }
                break;
            }
        }
    }
    /// Agent markdown, line by line: complete lines print once; `done` flushes the rest.
    fn stream(&mut self, id: ItemId, done: bool, out: &mut String) {
        let pen = self.pen();
        let text = match self.items.get(&id).and_then(|e| e.body.as_ref()) {
            Some(Item::AgentMessage { markdown }) => markdown.clone(),
            _ => self
                .items
                .get(&id)
                .map(|e| e.text.clone())
                .unwrap_or_default(),
        };
        let mut st = self.streams.remove(&id).unwrap_or_default();
        let mut buf = String::new();
        if !text.starts_with(&st.emitted) {
            buf += &pen.row(&[sp("  ↻ stream update").fg(DIM).ital()], None);
            st = Streaming::default();
        }
        let new = &text[st.emitted.len()..];
        let take = if done {
            new.trim_end().len()
        } else {
            new.rfind('\n').map_or(0, |i| i + 1)
        };
        let first = st.emitted.is_empty();
        for l in new[..take].split_inclusive('\n') {
            let l = l.strip_suffix('\n').unwrap_or(l);
            if first && buf.is_empty() && l.trim().is_empty() {
                continue;
            }
            buf += &st.md.line(l, &pen);
        }
        st.emitted.push_str(&new[..take]);
        if done {
            if st.md.code.is_some() && pen.mode != ColorMode::None {
                buf += &pen.row(&[sp("  ")], Some(CODE_BG));
            }
            let t = buf.trim_end_matches('\n').len();
            buf.truncate(t);
            if !buf.is_empty() {
                buf.push('\n');
            }
        } else {
            self.streams.insert(id, st);
        }
        if !buf.is_empty() {
            if first && self.family == Family::Prose {
                buf.insert(0, '\n');
            }
            self.block(out, Family::Prose, false, buf);
        }
    }
    fn notice_line(&mut self, out: &mut String, text: &str, level: &Level) {
        let pen = self.pen();
        let (mark, color) = match level {
            Level::Warn => ("● ", ASK),
            Level::Error => ("✗ ", BAD),
            _ => ("· ", DIM),
        };
        let mut s = sp(text).fg(if matches!(level, Level::Warn) {
            DIM
        } else {
            color
        });
        if matches!(level, Level::Info | Level::Unknown(_)) {
            s = s.ital();
        }
        let mut rows = String::new();
        for l in wrap(
            vec![s],
            pen.width,
            vec![sp("  "), sp(mark).fg(color)],
            vec![sp("    ")],
        ) {
            rows += &pen.row(&l, None);
        }
        self.block(out, Family::Card, false, rows);
    }
    fn print_item(&mut self, id: ItemId, out: &mut String) {
        if !self.printed.insert(id) {
            return;
        }
        let Some(e) = self.items.get(&id) else { return };
        let pen = self.pen();
        match (e.kind.clone(), e.body.clone()) {
            (ItemKind::UserMessage, Some(Item::UserMessage { blocks })) => {
                let text = blocks
                    .iter()
                    .map(InputBlock::display_text)
                    .collect::<Vec<_>>()
                    .join("\n");
                let plain = pen.mode == ColorMode::None;
                let mut rows = String::new();
                for (i, para) in text.lines().enumerate() {
                    let first = if i == 0 {
                        vec![sp(" ❯ ").fg(SEAT).bold()]
                    } else {
                        vec![sp("   ")]
                    };
                    let base = Sp {
                        fg: Some(BRIGHT),
                        ..Default::default()
                    };
                    for l in wrap(
                        inline(para, &base, plain),
                        pen.width,
                        first,
                        vec![sp("   ")],
                    ) {
                        rows += &pen.row(&l, (!plain).then_some(USER_BG));
                    }
                }
                self.block(out, Family::User, true, rows);
            }
            (ItemKind::AgentMessage, _) => self.stream(id, true, out),
            (ItemKind::Reasoning, body) => {
                let text = match &body {
                    Some(Item::Reasoning { summary, raw, .. }) if !summary.trim().is_empty() => {
                        summary.clone()
                    }
                    Some(Item::Reasoning { raw: Some(raw), .. }) => raw.clone(),
                    _ => e.text.clone(),
                };
                let took = e.t1.map(|t| t.saturating_sub(e.t0));
                let mut l = vec![sp("  ✻ ").fg(THINK)];
                match took {
                    Some(t) if t >= 100 => {
                        l.push(sp("Thought for ").fg(DIM));
                        l.push(sp(core::duration(t)).fg(FG).bold());
                    }
                    Some(_) => l.push(sp("Thought").fg(DIM).bold()),
                    None => l.push(sp("Thinking").fg(DIM).bold()),
                }
                let gist = gist(&text);
                if !gist.is_empty() {
                    l.push(sp(" · ").fg(FAINT));
                    l.push(sp(gist).fg(DIM).ital());
                }
                let row = pen.row(&fit(l, pen.width), None);
                self.block(out, Family::Card, false, row);
            }
            (ItemKind::ToolCall, _) if self.open.contains_key(&id) => {
                self.run_card(id, true, out);
            }
            (ItemKind::ToolCall, _) => {
                let text = self.tool_card(id, &pen);
                self.block(out, Family::Card, false, text);
            }
            (ItemKind::Terminal, body) => {
                let (command, exit, live) = match &body {
                    Some(Item::Terminal {
                        command,
                        exit,
                        live,
                        ..
                    }) => (
                        command.clone(),
                        *exit,
                        live.clone().unwrap_or_else(|| e.text.clone()),
                    ),
                    _ => match &e.head {
                        Some(ItemHead::Terminal { command, .. }) => {
                            (command.clone(), None, e.text.clone())
                        }
                        _ => (String::new(), None, e.text.clone()),
                    },
                };
                let command = command
                    .strip_prefix("bash -c ")
                    .unwrap_or(&command)
                    .to_owned();
                let st = status_of(e.status.as_ref(), exit);
                let meta =
                    exit.map(|c| (format!("exit {c}"), if c == 0 { TERM_OK } else { TERM_BAD }));
                let head = card_head(
                    &pen,
                    Head {
                        kind: "execute",
                        verb: "Run",
                        target: &command,
                        mono: true,
                        meta,
                        took: e.t1.map(|t| t.saturating_sub(e.t0)),
                        ..Default::default()
                    },
                    st,
                );
                let body = term_body(&pen, &command, &live, st);
                self.block(out, Family::Card, false, head + &body);
            }
            (ItemKind::FileChange, Some(Item::FileChange { path, unified, .. })) => {
                let p = self.short(&path.to_string_lossy());
                let stat = core::diff_stat(&unified);
                let head = card_head(
                    &pen,
                    Head {
                        kind: "edit",
                        verb: "Changed",
                        target: &p,
                        stat: Some(stat),
                        ..Default::default()
                    },
                    St::Done,
                );
                let body = diff_body(&pen, &unified, St::Done);
                self.block(out, Family::Card, false, head + &body);
            }
            (ItemKind::Approval, Some(_)) => {
                let text = self.approval_card(id);
                self.block(out, Family::Card, false, text);
            }
            (ItemKind::Plan, Some(Item::Plan { entries })) => {
                let done = entries
                    .iter()
                    .filter(|p| p.status == PlanStatus::Completed)
                    .count();
                let mut rows = pen.row(
                    &[
                        sp("  ☰ ").fg(SEAT),
                        sp("Plan").fg(BRIGHT).bold(),
                        sp(format!("  {done}/{}", entries.len())).fg(DIM),
                    ],
                    None,
                );
                for p in entries {
                    let (mark, c, dim) = match p.status {
                        PlanStatus::Completed => ("✓ ", OK, true),
                        PlanStatus::InProgress => ("◐ ", ASK, false),
                        PlanStatus::Cancelled => ("✕ ", DIM, true),
                        _ => ("○ ", DIM, false),
                    };
                    let mut t = sp(p.text.clone());
                    t = if dim { t.fg(DIM) } else { t.fg(FG) };
                    for l in wrap(
                        vec![t],
                        pen.width,
                        vec![sp("  │ ").fg(RAIL), sp(mark).fg(c)],
                        vec![sp("  │   ").fg(RAIL)],
                    ) {
                        rows += &pen.row(&l, None);
                    }
                }
                self.block(out, Family::Card, false, rows);
            }
            (ItemKind::Notice, Some(Item::Notice { level, text })) => {
                self.notice_line(out, &text, &level);
            }
            (_, Some(body)) if body.extension_text().is_some() => {
                self.notice_line(out, &body.extension_text().unwrap(), &Level::Info);
            }
            (
                ItemKind::Subagent,
                Some(Item::Subagent {
                    session,
                    name,
                    call,
                }),
            ) if name.is_some() || call.is_some() => {
                self.notice_line(
                    out,
                    &format!("Subagent {} · {session}", name.as_deref().unwrap_or("")),
                    &Level::Info,
                );
            }
            (ItemKind::Usage, Some(Item::Usage { usage })) if usage.scope.is_some() => {
                self.notice_line(
                    out,
                    &format!(
                        "Usage: {} · context {}/{}",
                        usage
                            .numbers()
                            .into_iter()
                            .map(|(k, v)| format!("{k}={v}"))
                            .collect::<Vec<_>>()
                            .join(" · "),
                        usage
                            .context_used
                            .map_or_else(|| "unreported".into(), |v| v.to_string()),
                        usage
                            .context_window
                            .map_or_else(|| "unreported".into(), |v| v.to_string())
                    ),
                    &Level::Info,
                );
            }
            (_, Some(Item::Unknown(raw))) => self.notice_line(
                out,
                &format!("Unsupported item: {}", unknown_tag(&raw)),
                &Level::Info,
            ),
            (ItemKind::Unknown(raw), _) => self.notice_line(
                out,
                &format!("Unsupported item: {}", unknown_tag(&raw)),
                &Level::Info,
            ),
            _ => {}
        }
    }
    fn short(&self, p: &str) -> String {
        if !self.cwd.is_empty()
            && let Some(rest) = p.strip_prefix(&format!("{}/", self.cwd))
        {
            return rest.to_owned();
        }
        p.to_owned()
    }
    fn kid(&self, tool: &ItemId, kind: ItemKind) -> Option<&Entry> {
        self.kids
            .get(tool)?
            .iter()
            .filter_map(|k| self.items.get(k))
            .find(|k| k.kind == kind)
    }
    fn approval_card(&self, id: ItemId) -> String {
        let pen = self.pen();
        let Some(Item::Approval {
            subject,
            options,
            decision,
            by,
        }) = self.items.get(&id).and_then(|e| e.body.clone())
        else {
            return String::new();
        };
        let effect = format!("{:?}", subject.effect).to_lowercase();
        let what = subject.command.clone().unwrap_or(subject.summary.clone());
        let what = describe_subject(&what);
        let mut l = vec![
            sp("  ⚠ ").fg(ASK),
            sp("Allow ").fg(FG),
            sp(effect).fg(ASK).bold(),
            sp("? ").fg(FG),
        ];
        l.push(if pen.mode == ColorMode::None {
            sp(format!("`{what}`"))
        } else {
            sp(what).fg(BRIGHT).bg(CHIP_BG)
        });
        let mut rows = String::new();
        if let Some(d) = &decision {
            let (badge, c) = badge_of(d, by.as_ref());
            l[0] = sp("  ⛨ ").fg(c);
            l.push(sp("  "));
            l.push(sp(badge).fg(c));
            rows += &pen.row(&fit(l, pen.width), None);
        } else {
            rows += &pen.row(&fit(l, pen.width), None);
            let opts = options
                .iter()
                .map(|o| match o {
                    ApprovalDecisionKind::Allow => "Allow",
                    ApprovalDecisionKind::AllowAlways => "Always allow",
                    ApprovalDecisionKind::Deny => "Deny",
                    ApprovalDecisionKind::Unknown(_) => "?",
                })
                .collect::<Vec<_>>()
                .join(" · ");
            rows += &pen.row(
                &[
                    sp("  │ ").fg(ASK),
                    sp("needs approval: ").fg(ASK),
                    sp(opts).fg(DIM),
                ],
                None,
            );
        }
        rows
    }
    /// A command that is still running prints as it goes: the header once, `$ command`,
    /// output lines as they complete, and at the end a footer with exit, badge and time.
    fn run_card(&mut self, id: ItemId, done: bool, out: &mut String) {
        let pen = self.pen();
        let Some(e) = self.items.get(&id) else { return };
        let (name, kind, input) = match (&e.body, &e.head) {
            (
                Some(Item::ToolCall {
                    name,
                    tool_kind,
                    input_raw,
                    ..
                }),
                _,
            ) => (name.clone(), tool_kind.clone(), input_raw.clone()),
            (
                _,
                Some(ItemHead::ToolCall {
                    name, tool_kind, ..
                }),
            ) => (name.clone(), tool_kind.clone(), String::new()),
            _ => ("tool".into(), ToolKind::Execute, String::new()),
        };
        let command = match self
            .kid(&id, ItemKind::Terminal)
            .and_then(|t| t.head.clone())
        {
            Some(ItemHead::Terminal { command, .. }) => command
                .strip_prefix("bash -c ")
                .unwrap_or(&command)
                .to_owned(),
            _ => {
                let d = core::describe(&name, kind.clone(), &input, &[], &self.cwd);
                if d.target.is_empty() {
                    name.clone()
                } else {
                    d.target
                }
            }
        };
        let mut text = String::new();
        let continuing = self.open.contains_key(&id);
        if !continuing {
            text += &card_head(
                &pen,
                Head {
                    kind: "execute",
                    verb: "Run",
                    target: &command,
                    mono: true,
                    open: true,
                    ..Default::default()
                },
                St::Running,
            );
            for l in wrap(
                vec![sp(command.replace('\n', " ")).fg(BRIGHT)],
                pen.width,
                vec![rail(St::Running), sp("$ ").fg(SEAT).bold()],
                vec![rail(St::Running), sp("  ")],
            ) {
                text += &pen.row(&l, None);
            }
            self.open.insert(id, Open::default());
        }
        // output: streamed deltas, else what the tool returned once it is done
        let streamed = self
            .items
            .get(&id)
            .map(|e| e.text.clone())
            .unwrap_or_default();
        let source = if streamed.is_empty() && done {
            let e = &self.items[&id];
            let mut d = match &e.body {
                Some(Item::ToolCall { output, .. }) => {
                    core::describe(&name, kind.clone(), &input, output, &self.cwd).out
                }
                _ => String::new(),
            };
            if d.is_empty() {
                d = match self
                    .kid(&id, ItemKind::Terminal)
                    .and_then(|t| t.body.clone())
                {
                    Some(Item::Terminal { live: Some(l), .. }) => l,
                    _ => String::new(),
                };
            }
            d.lines()
                .filter(|l| !l.trim_start().starts_with("```"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            streamed
        };
        let st = self.open.get_mut(&id).unwrap();
        let rest = &source[st.taken.min(source.len())..];
        let take = if done {
            rest.len()
        } else {
            rest.rfind('\n').map_or(0, |i| i + 1)
        };
        let fresh = &rest[..take];
        st.taken += take;
        for l in fresh.lines() {
            if st.shown == 0 && l.trim().is_empty() {
                continue;
            }
            if st.shown >= BODY {
                st.hidden += 1;
                continue;
            }
            st.shown += 1;
            let c = match core::verdict(l) {
                Verdict::Ok => TERM_OK,
                Verdict::Bad => TERM_BAD,
                Verdict::Warn => ASK,
                Verdict::Dim => DIM,
                Verdict::Plain => rgb(0xC9CFDC),
            };
            text += &pen.row(&fit(vec![rail(St::Running), sp(l).fg(c)], pen.width), None);
        }
        if done {
            let hidden = st.hidden;
            self.open.remove(&id);
            let e = &self.items[&id];
            let exit = self
                .kid(&id, ItemKind::Terminal)
                .and_then(|t| match &t.body {
                    Some(Item::Terminal { exit, .. }) => *exit,
                    _ => None,
                });
            let approval = self
                .kid(&id, ItemKind::Approval)
                .and_then(|a| match &a.body {
                    Some(Item::Approval { decision, by, .. }) => {
                        decision.clone().map(|d| (d, by.clone()))
                    }
                    _ => None,
                });
            let st = match (&approval, status_of(e.status.as_ref(), None)) {
                (Some((ApprovalDecision::Deny { .. }, _)), _) => St::Denied,
                (_, s) => s,
            };
            text += &more(&pen, hidden, St::Running);
            let mut right: Line = vec![];
            let sep = |r: &mut Line| {
                if !r.is_empty() {
                    r.push(sp(" · ").fg(FAINT));
                }
            };
            if let Some(c) = exit {
                right.push(sp(format!("exit {c}")).fg(if c == 0 { TERM_OK } else { TERM_BAD }));
            }
            if let Some((d, by)) = &approval {
                let (b, c) = badge_of(d, by.as_ref());
                sep(&mut right);
                right.push(sp(b).fg(c));
            }
            if let Some(t1) = e.t1 {
                sep(&mut right);
                right.push(sp(core::duration(t1.saturating_sub(e.t0))).fg(DIM));
            }
            let (mark, mc) = mark_of(st);
            right.push(sp(format!("  {mark}")).fg(mc).bold());
            let left = vec![sp("  ╰").fg(RAIL)];
            let pad = pen.width.saturating_sub(lw(&left) + lw(&right) + 1);
            let mut l = left;
            l.push(sp(format!("{} ", "─".repeat(pad))).fg(RAIL));
            l.extend(right);
            text += &pen.row(&fit(l, pen.width), None);
        }
        if continuing {
            // the rest of an open card: no gap between its parts
            out.push_str(&text);
        } else {
            self.block(out, Family::Card, false, text);
        }
        self.tall = true;
    }
    fn tool_card(&self, id: ItemId, pen: &Pen) -> String {
        let Some(e) = self.items.get(&id) else {
            return String::new();
        };
        let (name, kind, input, output) = match &e.body {
            Some(Item::ToolCall {
                name,
                tool_kind,
                input_raw,
                output,
                ..
            }) => (
                name.clone(),
                tool_kind.clone(),
                input_raw.clone(),
                output.clone(),
            ),
            _ => match &e.head {
                Some(ItemHead::ToolCall {
                    name, tool_kind, ..
                }) => (name.clone(), tool_kind.clone(), String::new(), vec![]),
                _ => ("tool".into(), ToolKind::Other, String::new(), vec![]),
            },
        };
        let mut d = core::describe(&name, kind.clone(), &input, &output, &self.cwd);
        let known = matches!(
            name.as_str(),
            "read" | "grep" | "glob" | "bash" | "exec" | "edit" | "apply_patch" | "write"
        ) || name.starts_with("mcp__");
        if d.target.is_empty() && !known {
            // agents name tools freely ("Terminal", "ls -la"): find the subject in the args
            let arg = |k: &str| d.args.get(k).and_then(|v| v.as_str()).map(str::to_owned);
            if let Some(c) = arg("command").or_else(|| arg("cmd")) {
                d.verb = "Run".into();
                d.target = c;
                d.mono = true;
                d.kind = "execute";
            } else if let Some(p) = arg("path").or_else(|| arg("file_path")) {
                d.target = self.short(&p);
            } else if d.kind == "execute" && name.contains(' ') {
                d.verb = "Run".into();
                d.target = name.clone();
                d.mono = true;
            } else if let Some(Item::ToolCall { locations, .. }) = &e.body
                && let Some(l) = locations.first()
            {
                d.target = self.short(&l.path.to_string_lossy());
            }
        }
        // agent output often arrives as a fenced block: the card is the frame already
        d.out = d
            .out
            .lines()
            .filter(|l| !l.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n");
        // placeholders an agent sends instead of output count as none
        let t = d.out.trim();
        if t == "(no output)" || (t.starts_with("[terminal ") && t.ends_with(']')) {
            d.out.clear();
        }
        let approval = self
            .kid(&id, ItemKind::Approval)
            .and_then(|a| match &a.body {
                Some(Item::Approval { decision, by, .. }) => {
                    decision.clone().map(|d| (d, by.clone()))
                }
                _ => None,
            });
        let denied = matches!(approval, Some((ApprovalDecision::Deny { .. }, _)));
        let term = self.kid(&id, ItemKind::Terminal);
        let change = self.kid(&id, ItemKind::FileChange);
        let exit = term.and_then(|t| match &t.body {
            Some(Item::Terminal { exit, .. }) => *exit,
            _ => None,
        });
        let st = if denied {
            St::Denied
        } else if e.body.is_none() {
            St::Running
        } else {
            status_of(e.status.as_ref(), None)
        };
        let lines = d.out.lines().filter(|l| !l.trim().is_empty()).count();
        let mut meta = None;
        if d.kind == "execute"
            && let Some(c) = exit
        {
            meta = Some((format!("exit {c}"), if c == 0 { TERM_OK } else { TERM_BAD }));
        } else if d.verb == "Read" && lines > 0 && st == St::Done {
            meta = Some((
                format!("{lines} line{}", if lines == 1 { "" } else { "s" }),
                DIM,
            ));
        } else if (d.verb == "Search" || d.verb == "Find") && st == St::Done {
            meta = Some((
                format!("{lines} hit{}", if lines == 1 { "" } else { "s" }),
                DIM,
            ));
        }
        let unified = match change.and_then(|c| c.body.as_ref()) {
            Some(Item::FileChange { unified, .. }) => Some(unified.clone()),
            _ if d.kind == "edit" && d.out.starts_with("---") => Some(d.out.clone()),
            _ => None,
        };
        let stat = unified.as_deref().map(core::diff_stat);
        let badge = approval.as_ref().map(|(d, by)| badge_of(d, by.as_ref()));
        let mut target = d.target.clone();
        let mut chip = None;
        if name.starts_with("mcp__") {
            chip = Some("mcp");
        }
        if d.verb == "Search"
            && let Some(p) = d.args.get("path").and_then(|v| v.as_str())
        {
            target = format!("{target}  in {}", self.short(p));
        }
        let head = card_head(
            pen,
            Head {
                kind: d.kind,
                verb: &d.verb,
                target: &target,
                mono: d.mono,
                chip,
                stat,
                meta,
                badge,
                took: e.t1.map(|t| t.saturating_sub(e.t0)),
                open: false,
            },
            st,
        );
        let failed_reason = match e.status.as_ref() {
            Some(ItemStatus::Failed { reason, .. }) => Some(reason.clone()),
            _ => None,
        };
        let body = if denied {
            let reason = match &approval {
                Some((ApprovalDecision::Deny { reason: Some(r) }, _)) => r.clone(),
                _ => failed_reason.unwrap_or_else(|| "denied".into()),
            };
            err_body(pen, &reason)
        } else if d.kind == "execute" {
            let live = if !d.out.is_empty() {
                d.out.clone()
            } else {
                term.map(|t| match &t.body {
                    Some(Item::Terminal { live: Some(l), .. }) => l.clone(),
                    _ => t.text.clone(),
                })
                .unwrap_or_default()
            };
            let mut body = term_body(pen, &d.target, &live, st);
            if live.trim().is_empty() && e.body.is_some() {
                let note = if name.contains(' ') {
                    "output stayed in the agent's own terminal"
                } else {
                    "no output"
                };
                body += &pen.row(&[rail(st), sp(note).fg(DIM).ital()], None);
            }
            body
        } else if d.kind == "edit" {
            match &unified {
                Some(u) => diff_body(pen, u, st),
                None if st == St::Failed => err_body(
                    pen,
                    if d.out.is_empty() {
                        failed_reason.as_deref().unwrap_or("failed")
                    } else {
                        &d.out
                    },
                ),
                None => out_body(pen, &d.out),
            }
        } else if st == St::Failed {
            err_body(
                pen,
                if d.out.trim().is_empty() {
                    failed_reason.as_deref().unwrap_or("failed")
                } else {
                    &d.out
                },
            )
        } else if d.verb == "Search" && lines > 0 && lines <= 6 {
            hits_body(
                pen,
                &d.out,
                d.args.get("pattern").and_then(|v| v.as_str()),
                &self.cwd,
            )
        } else {
            String::new()
        };
        let mut extras = output
            .iter()
            .filter(|block| {
                matches!(
                    block,
                    OutputBlock::Audio { .. }
                        | OutputBlock::Resource { .. }
                        | OutputBlock::Terminal { .. }
                        | OutputBlock::Citations { .. }
                )
            })
            .map(OutputBlock::display_text)
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(Item::ToolCall {
            title,
            server,
            raw_output,
            ..
        }) = &e.body
        {
            let caption = [title.as_deref(), server.as_deref()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · ");
            if !caption.is_empty() {
                extras = format!("{caption}\n{extras}");
            }
            if output.is_empty()
                && let Some(value) = raw_output
            {
                extras.push_str(&value.to_string());
            }
        }
        head + &body + &out_body(pen, &extras)
    }
}

/// `acp:<command> · <model>` (an ACP session's model string) as (command, model).
fn acp_model(model: &str) -> Option<(String, String)> {
    let rest = model.strip_prefix("acp:")?;
    let (agent, model) = rest.split_once('·').unwrap_or((rest, ""));
    Some((agent.trim().to_owned(), model.trim().to_owned()))
}
/// An agent's tile mark and colour by its command (harness.js AGENTS).
fn agent_mark(command: &str) -> (String, Rgb) {
    let c = command.split_whitespace().next().unwrap_or("");
    let (m, h) = match c {
        "claude-agent-acp" => ("C", 0xD97757),
        "codex-acp" => ("Cx", 0x10A37F),
        "opencode" => ("oc", 0x5B6178),
        "pi-acp" => ("π", 0xFF4FA3),
        "omp" => ("ω", 0xB07BFF),
        "goose" => ("G", 0xC98A1B),
        _ => {
            let f = c
                .chars()
                .next()
                .map_or("·".into(), |f| f.to_uppercase().to_string());
            return (f, rgb(0x2EE6A8));
        }
    };
    (m.into(), rgb(h))
}
fn kind_of(body: &Item) -> ItemKind {
    body.kind()
}
/// The first sentence of a thought, on one line.
fn gist(text: &str) -> String {
    let first = text.trim().lines().next().unwrap_or("");
    let flat = first
        .split_whitespace()
        .filter(|w| !w.chars().all(|c| c == '#'))
        .collect::<Vec<_>>()
        .join(" ")
        .replace("**", "")
        .replace("__", "")
        .replace('`', "");
    let mut end = flat.len();
    for (i, c) in flat.char_indices() {
        if matches!(c, '.' | '!' | '?') && flat[i + 1..].starts_with(' ') {
            end = i + 1;
            break;
        }
    }
    flat[..end].to_owned()
}
/// An approval subject like `mcp__bboxy__orient {}` reads as its tool, never raw JSON.
fn describe_subject(s: &str) -> String {
    let (name, rest) = s.split_once(' ').unwrap_or((s, ""));
    if rest.trim_start().starts_with('{') {
        let d = core::describe(name, ToolKind::Other, rest, &[], "");
        if d.target.is_empty() {
            d.verb
        } else {
            format!("{} {}", d.verb, d.target)
        }
    } else {
        s.to_owned()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    Running,
    /// interrupted or cancelled: neutral, not red
    Stopped,
    Done,
    Failed,
    Denied,
}
fn status_of(s: Option<&ItemStatus>, exit: Option<i32>) -> St {
    match s {
        None => St::Running,
        Some(ItemStatus::Completed) if exit.is_none_or(|c| c == 0) => St::Done,
        Some(ItemStatus::Interrupted | ItemStatus::Cancelled) => St::Stopped,
        _ => St::Failed,
    }
}
fn badge_of(d: &ApprovalDecision, by: Option<&Resolver>) -> (String, Rgb) {
    let unattended = matches!(by, Some(Resolver::Unattended));
    let rule = matches!(by, Some(Resolver::Rule { .. }));
    match d {
        ApprovalDecision::Deny { .. } if unattended => ("⛨ denied · no approver".into(), BAD),
        ApprovalDecision::Deny { .. } => ("⛨ denied".into(), BAD),
        _ if rule => ("⛨ auto".into(), DIM),
        ApprovalDecision::AllowAlways => ("⛨ always".into(), OK),
        _ => ("⛨ allowed".into(), OK),
    }
}
fn glyph(kind: &str) -> &'static str {
    match kind {
        "read" => "▤",
        "search" => "◎",
        "execute" => "▶",
        "edit" => "✎",
        "delete" => "✕",
        "move" => "⇢",
        "fetch" => "◍",
        "think" => "✻",
        _ => "◆",
    }
}
/// The card header: glyph chip, verb, target … right-aligned stat, meta, badge, time, mark.
#[derive(Default)]
struct Head<'a> {
    kind: &'a str,
    verb: &'a str,
    target: &'a str,
    mono: bool,
    chip: Option<&'a str>,
    stat: Option<(usize, usize)>,
    meta: Option<(String, Rgb)>,
    badge: Option<(String, Rgb)>,
    took: Option<u64>,
    /// an open card: status and time arrive in its footer
    open: bool,
}
fn card_head(pen: &Pen, h: Head, st: St) -> String {
    let Head {
        kind,
        verb,
        target,
        mono,
        chip,
        stat,
        meta,
        badge,
        took,
        open,
    } = h;
    let kc = hex(core::kind_hex(kind));
    let mut left = vec![
        sp("  "),
        sp(format!(" {} ", glyph(kind)))
            .fg(kc)
            .bg(mix(kc, rgb(0x0B0D12), 0.2)),
        sp(" "),
        sp(verb).fg(BRIGHT).bold(),
    ];
    if let Some(c) = chip {
        left.push(sp(" "));
        left.push(sp(format!(" {c} ")).fg(DIM).bg(CHIP_BG));
    }
    let target = target.lines().next().unwrap_or("").trim().to_owned();
    if !target.is_empty() {
        left.push(sp(" "));
        left.push(if mono {
            sp(target).fg(rgb(0xC9CFDC))
        } else {
            sp(target).fg(FG)
        });
    }
    let mut right: Line = vec![];
    let gap = |r: &mut Line| {
        if !r.is_empty() {
            r.push(sp(" · ").fg(FAINT));
        }
    };
    if let Some((a, d)) = stat {
        right.push(sp(format!("+{a}")).fg(TERM_OK));
        right.push(sp(" "));
        right.push(sp(format!("−{d}")).fg(TERM_BAD));
    }
    if let Some((m, c)) = meta {
        gap(&mut right);
        right.push(sp(m).fg(c));
    }
    if let Some((b, c)) = badge {
        gap(&mut right);
        right.push(sp(b).fg(c));
    }
    if let Some(t) = took {
        gap(&mut right);
        right.push(sp(core::duration(t)).fg(DIM));
    }
    if !open {
        let (mark, mc) = mark_of(st);
        right.push(sp(format!("  {mark}")).fg(mc).bold());
    }
    let rw = lw(&right);
    let room = pen.width.saturating_sub(rw + 1);
    let left = fit(left, room);
    let pad = pen.width.saturating_sub(lw(&left) + rw);
    let mut l = left;
    l.push(sp(" ".repeat(pad)));
    l.extend(right);
    pen.row(&l, None)
}
fn mark_of(st: St) -> (&'static str, Rgb) {
    match st {
        St::Running => ("◌", DIM),
        St::Done => ("✓", OK),
        St::Failed => ("✗", BAD),
        St::Denied => ("⊘", BAD),
        St::Stopped => ("■", DIM),
    }
}
fn rail(st: St) -> Sp {
    sp("  │ ").fg(match st {
        St::Failed | St::Denied => mix(BAD, rgb(0x0B0D12), 0.45),
        _ => RAIL,
    })
}
fn more(pen: &Pen, n: usize, st: St) -> String {
    if n == 0 {
        return String::new();
    }
    pen.row(
        &[
            rail(st),
            sp(format!("… {n} more line{}", if n == 1 { "" } else { "s" }))
                .fg(DIM)
                .ital(),
        ],
        None,
    )
}
fn err_body(pen: &Pen, text: &str) -> String {
    let lines: Vec<&str> = text.trim_matches('\n').lines().collect();
    let mut rows = String::new();
    let mut shown = 0;
    for l in lines.iter().take(BODY) {
        for w in wrap(
            vec![sp(*l).fg(TERM_BAD)],
            pen.width,
            vec![rail(St::Failed)],
            vec![rail(St::Failed)],
        ) {
            rows += &pen.row(&w, None);
        }
        shown += 1;
    }
    rows + &more(pen, lines.len() - shown, St::Failed)
}
fn out_body(pen: &Pen, text: &str) -> String {
    let lines: Vec<&str> = text.trim_matches('\n').lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let mut rows = String::new();
    for l in lines.iter().take(BODY) {
        rows += &pen.row(
            &fit(vec![rail(St::Done), sp(*l).fg(rgb(0xC9CFDC))], pen.width),
            None,
        );
    }
    rows + &more(pen, lines.len().saturating_sub(BODY), St::Done)
}
/// `$ command`, then the output coloured by verdict.
fn term_body(pen: &Pen, command: &str, out: &str, st: St) -> String {
    let mut rows = String::new();
    if !command.trim().is_empty() {
        for l in wrap(
            vec![sp(command.replace('\n', " ")).fg(BRIGHT)],
            pen.width,
            vec![rail(st), sp("$ ").fg(SEAT).bold()],
            vec![rail(st), sp("  ")],
        ) {
            rows += &pen.row(&l, None);
        }
    }
    let lines: Vec<&str> = out.trim_matches('\n').lines().collect();
    for l in lines.iter().take(BODY) {
        let c = match core::verdict(l) {
            Verdict::Ok => TERM_OK,
            Verdict::Bad => TERM_BAD,
            Verdict::Warn => ASK,
            Verdict::Dim => DIM,
            Verdict::Plain => rgb(0xC9CFDC),
        };
        rows += &pen.row(&fit(vec![rail(st), sp(*l).fg(c)], pen.width), None);
    }
    rows + &more(pen, lines.len().saturating_sub(BODY), st)
}
/// The minimal diff, two context lines around each change, unchanged runs folded.
fn diff_body(pen: &Pen, unified: &str, st: St) -> String {
    let plain = pen.mode == ColorMode::None;
    let mut rows: Vec<String> = vec![];
    for hunk in core::minimal_diff(unified) {
        let near: Vec<bool> = (0..hunk.len())
            .map(|i| {
                hunk[i.saturating_sub(2)..(i + 3).min(hunk.len())]
                    .iter()
                    .any(|r| r.op != ' ')
            })
            .collect();
        let mut skipped = 0;
        let fold = |rows: &mut Vec<String>, skipped: &mut usize| {
            if *skipped > 0 {
                rows.push(
                    pen.row(
                        &[
                            rail(st),
                            sp(format!(
                                "     ⋯ {} unchanged line{}",
                                skipped,
                                if *skipped == 1 { "" } else { "s" }
                            ))
                            .fg(DIM)
                            .ital(),
                        ],
                        None,
                    ),
                );
                *skipped = 0;
            }
        };
        for (i, r) in hunk.iter().enumerate() {
            if !near[i] {
                skipped += 1;
                continue;
            }
            fold(&mut rows, &mut skipped);
            let n = r.new.or(r.old).map_or(String::new(), |n| n.to_string());
            let (sign, sc, bg) = match r.op {
                '+' => ("+", TERM_OK, Some(ADD_BG)),
                '-' => ("−", TERM_BAD, Some(DEL_BG)),
                _ => (" ", DIM, None),
            };
            let mut l = vec![rail(st), sp(format!("{n:>4} ")).fg(FAINT)];
            let mut s = sp(sign).fg(sc).bold();
            if let Some(b) = bg {
                s = s.bg(b);
            }
            l.push(s);
            for (t, k) in core::tokens(&clean(&r.text)) {
                let (c, it) = tok_color(k);
                let mut s = Sp {
                    t,
                    ..Default::default()
                }
                .fg(if r.op == ' ' { mix(c, DIM, 0.6) } else { c });
                s.ital = it;
                if let Some(b) = bg {
                    s = s.bg(b);
                }
                l.push(s);
            }
            let mut l = fit(l, pen.width);
            if let Some(b) = bg
                && !plain
            {
                let pad = pen.width.saturating_sub(lw(&l));
                l.push(sp(" ".repeat(pad)).bg(b));
            }
            rows.push(pen.row(&l, None));
        }
        fold(&mut rows, &mut skipped);
    }
    let hidden = rows.len().saturating_sub(BODY);
    let mut s: String = rows.into_iter().take(BODY).collect();
    s += &more(pen, hidden, st);
    s
}
/// Search hits: `file:line  text`, the pattern marked.
fn hits_body(pen: &Pen, out: &str, pattern: Option<&str>, cwd: &str) -> String {
    let re = pattern.and_then(|p| regex::Regex::new(p).ok());
    let mut rows = String::new();
    for l in out.lines().filter(|l| !l.trim().is_empty()).take(BODY) {
        let mut parts = l.splitn(3, ':');
        let (f, n, x) = (parts.next(), parts.next(), parts.next());
        let mut line = vec![rail(St::Done)];
        match (f, n.filter(|n| n.parse::<u32>().is_ok()), x) {
            (Some(f), Some(n), Some(x)) => {
                let f = f.strip_prefix(&format!("{cwd}/")).unwrap_or(f);
                line.push(sp(f).fg(rgb(0x58D7FF)));
                line.push(sp(format!(":{n}  ")).fg(FAINT));
                let x = x.trim();
                let mut at = 0;
                if let Some(re) = &re {
                    for m in re.find_iter(x) {
                        if m.start() > at {
                            line.push(sp(&x[at..m.start()]).fg(rgb(0xC9CFDC)));
                        }
                        if !m.as_str().is_empty() {
                            line.push(sp(m.as_str()).fg(ASK).bold());
                        }
                        at = m.end();
                    }
                }
                line.push(sp(&x[at..]).fg(rgb(0xC9CFDC)));
            }
            _ => line.push(sp(l).fg(rgb(0xC9CFDC))),
        }
        rows += &pen.row(&fit(line, pen.width), None);
    }
    rows
}
fn receipt(pen: &Pen, t: &TurnStat, ts: u64, reason: &StopReason, usage: Option<&Usage>) -> String {
    // harness-spec.md "Turn receipt": ok, soft (stopped by someone) or bad, said in words
    let mut message = None;
    let (ok, soft, label) = match reason {
        StopReason::EndTurn => (true, false, "Done"),
        StopReason::Cancelled => (false, true, "Cancelled"),
        StopReason::Interrupted => (false, true, "Interrupted"),
        StopReason::MaxTokens => (false, false, "Hit the output limit"),
        StopReason::MaxRounds => (false, false, "Stopped at the round limit"),
        StopReason::Error { class, message: m } => {
            message = Some(m.clone()).filter(|m| !m.trim().is_empty());
            (
                false,
                false,
                match class {
                    ErrorClass::Transient => "Connection dropped",
                    ErrorClass::RateLimited => "Rate limited",
                    ErrorClass::Auth => "Not signed in",
                    ErrorClass::BadRequest => "Request refused",
                    ErrorClass::Malformed => "Unreadable reply",
                    ErrorClass::MidStream => "Reply cut off",
                    ErrorClass::Config => "Profile misconfigured",
                    ErrorClass::Store => "Could not save the session",
                    ErrorClass::Tool => "Tool failed",
                    ErrorClass::Cancelled => "Cancelled",
                    _ => "Harness error",
                },
            )
        }
        StopReason::Unknown(_) => (false, false, "Ended"),
    };
    let c = if ok {
        OK
    } else if soft {
        DIM
    } else {
        BAD
    };
    let mut facts = vec![];
    if let Some(t0) = t.t0 {
        facts.push(core::duration(ts.saturating_sub(t0)));
    }
    let plural = |n: usize, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
    if t.rounds > 0 {
        facts.push(plural(t.rounds, "round"));
    }
    if t.tools > 0 {
        facts.push(plural(t.tools, "tool"));
    }
    if t.edits > 0 {
        facts.push(plural(t.edits, "edit"));
    }
    if let Some(u) = usage.filter(|u| u.input_tokens > 0) {
        facts.push(format!(
            "{} in · {} out",
            core::kilo(u.input_tokens),
            core::kilo(u.output_tokens)
        ));
    }
    // one span: "✓ Done" stays a literal a watcher can find, coloured or not
    let mut l = vec![
        sp(format!(
            "{} {label}",
            if ok {
                "✓"
            } else if soft {
                "■"
            } else {
                "✗"
            }
        ))
        .fg(c)
        .bold(),
    ];
    for f in facts {
        l.push(sp(" · ").fg(FAINT));
        l.push(sp(f).fg(DIM));
    }
    let mut rows = String::new();
    for w in wrap(l, pen.width, vec![sp("  ")], vec![sp("    ")]) {
        rows += &pen.row(&w, None);
    }
    if let Some(m) = message {
        for w in wrap(
            vec![sp(m).fg(mix(BAD, FG, 0.8))],
            pen.width,
            vec![sp("    ")],
            vec![sp("    ")],
        ) {
            rows += &pen.row(&w, None);
        }
    }
    rows
}
