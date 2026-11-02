// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The rich face's blocks: one tool call is one card (approval badge, terminal and diff
//! folded in), reasoning is a one-line disclosure, the user speaks in a bubble.
//! Translated from bboxy450-term lab/web/harness.js and lab.css (the web reference the
//! parity script compares against). https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use crate::protocol::*;
use bbt_ui::{self as ui, components as c, modules::seat_row::named};
use bbt_ui_model::draw::{Color as DrawColor, LineCap, LineJoin, PathCommand as P, PathStroke};
use bbt_ui_model::layout::Axis;
use std::sync::Arc;

pub type Rgba = [f32; 4];
/// Theme inks, resolved once per row. Tints mix toward these like CSS color-mix.
#[derive(Clone, Copy)]
pub struct Ink {
    pub fg: Rgba,
    pub dim: Rgba,
    pub surface: Rgba,
    pub bg: Rgba,
    pub seat: Rgba,
    pub ok: Rgba,
    pub bad: Rgba,
    pub ask: Rgba,
}
pub fn hex(s: &str) -> Rgba {
    DrawColor::hex(s).0
}
pub fn ink(ctx: ui::Context) -> Ink {
    let p = ctx.theme.palette();
    Ink {
        fg: hex(p.fg),
        dim: hex(p.fg_dim),
        surface: hex(p.surface),
        bg: hex(p.bg),
        seat: hex(p.accent),
        ok: hex("#2EE6A8"),
        bad: hex("#FF5D5D"),
        ask: hex("#FFC53D"),
    }
}
/// color-mix(in srgb, a t, b)
pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    [0, 1, 2, 3].map(|i| a[i] * t + b[i] * (1. - t))
}
pub fn alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], c[3] * a]
}
pub fn paint(c: Rgba) -> ui::Paint {
    ui::Paint::solid(ui::Color::Rgba(c.map(|v| v.clamp(0., 1.))))
}
const TERM_BG: &str = "#07080c";
pub fn kind_color(kind: &str) -> Rgba {
    hex(match kind {
        "read" => "#58D7FF",
        "search" => "#B07BFF",
        "execute" => "#2EE6A8",
        "edit" => "#FFC53D",
        "fetch" => "#4FD1FF",
        "delete" => "#FF6B6B",
        "move" => "#F2D16B",
        _ => "#FF4FA3",
    })
}

// ---------- primitives ----------
pub fn text(value: impl Into<String>, size: f32, weight: u16, color: Rgba, mono: bool) -> c::Text {
    let mut t = c::Text::new(
        value,
        ui::Text {
            step: ui::TypeStep::Body,
            size: Some(size),
            weight,
            paint: paint(color),
            mono,
            line_height: 1.5,
            ..Default::default()
        },
    );
    t.wrap = true;
    t
}
pub fn line(
    value: impl Into<String>,
    size: f32,
    weight: u16,
    color: Rgba,
    mono: bool,
) -> c::Element {
    let mut t = text(value, size, weight, color, mono);
    t.wrap = false;
    t.ellipsis = true;
    c::Element::Text(t)
}
pub fn runs(
    parts: Vec<(String, Rgba, u16, bool)>,
    size: f32,
    mono: bool,
    wrap: bool,
) -> c::Element {
    let mut t = text("", size, 400, [0.; 4], mono);
    t.runs = parts
        .into_iter()
        .filter(|(s, ..)| !s.is_empty())
        .map(|(value, color, weight, italic)| c::TextRun {
            value,
            style: ui::Text {
                step: ui::TypeStep::Body,
                size: Some(size),
                weight,
                paint: paint(color),
                mono,
                italic,
                line_height: 1.5,
                ..Default::default()
            },
        })
        .collect();
    if t.runs.is_empty() {
        t.runs.push(c::TextRun {
            value: " ".into(),
            style: ui::Text {
                size: Some(size),
                mono,
                ..Default::default()
            },
        });
    }
    t.wrap = wrap;
    t.preserve_whitespace = true;
    c::Element::Text(t)
}
pub struct Frame {
    pub fill: Rgba,
    pub border: Option<Rgba>,
    pub radius: [f32; 4],
    pub padding: [f32; 4],
    pub gap: f32,
    pub row: bool,
    /// rows stretch children to the row height (rules beside text)
    pub stretch: bool,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            fill: [0.; 4],
            border: None,
            radius: [0.; 4],
            padding: [0.; 4],
            gap: 0.,
            row: false,
            stretch: false,
        }
    }
}
pub fn frame(f: Frame, children: Vec<c::Item>) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.material = Some(ui::Material::Flat);
    style.overrides.fill = Some(paint(f.fill));
    style.overrides.radii = Some(f.radius.map(ui::Length::Px));
    style.overrides.padding = Some(f.padding.map(ui::Length::Px));
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(if f.border.is_some() { 1. } else { 0. }),
        paint: paint(f.border.unwrap_or([0.; 4])),
        outside: false,
    });
    style.overrides.shadow = None;
    let mut b = c::Box::new(style, children);
    b.direction = if f.row { Axis::Row } else { Axis::Column };
    b.gap = ui::Length::Px(f.gap);
    b.align = if f.row && !f.stretch {
        c::Alignment::Center
    } else {
        c::Alignment::Stretch
    };
    c::Element::Box(b)
}
pub fn hstack(children: Vec<c::Item>, gap: f32) -> c::Element {
    frame(
        Frame {
            gap,
            row: true,
            ..Default::default()
        },
        children,
    )
}
pub fn vstack(children: Vec<c::Item>, gap: f32) -> c::Element {
    frame(
        Frame {
            gap,
            ..Default::default()
        },
        children,
    )
}
pub fn fill(e: c::Element) -> c::Item {
    c::Item {
        element: e,
        extent: c::Extent::Content {
            grow: 1.,
            shrink: 1.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    }
}
pub fn fixed(e: c::Element) -> c::Item {
    c::Item {
        element: e,
        extent: c::Extent::Content {
            grow: 0.,
            shrink: 0.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    }
}
fn spacer() -> c::Item {
    c::Item {
        element: c::Element::Spacer,
        extent: c::Extent::Fill,
    }
}
pub fn pill(label: &str, fg: Rgba, bg: Rgba, size: f32) -> c::Element {
    frame(
        Frame {
            fill: bg,
            radius: [999.; 4],
            padding: [1., 7., 1., 7.],
            row: true,
            ..Default::default()
        },
        vec![fixed(line(label, size, 600, fg, false))],
    )
}

// ---------- icons: 16-unit strokes, translated from harness.js ICON ----------
type Poly = Vec<[f32; 2]>;
fn circle(cx: f32, cy: f32, r: f32) -> Vec<P> {
    let k = 0.552_284_8 * r;
    vec![
        P::Move([cx + r, cy]),
        P::Cubic([[cx + r, cy + k], [cx + k, cy + r], [cx, cy + r]]),
        P::Cubic([[cx - k, cy + r], [cx - r, cy + k], [cx - r, cy]]),
        P::Cubic([[cx - r, cy - k], [cx - k, cy - r], [cx, cy - r]]),
        P::Cubic([[cx + k, cy - r], [cx + r, cy - k], [cx + r, cy]]),
    ]
}
fn polys(lines: &[Poly]) -> Vec<P> {
    lines
        .iter()
        .flat_map(|l| {
            l.iter()
                .enumerate()
                .map(|(i, p)| if i == 0 { P::Move(*p) } else { P::Line(*p) })
        })
        .collect()
}
pub fn icon(name: &str, color: Rgba, size: f32, weight: f32) -> c::Element {
    let cmds: Vec<P> = match name {
        "read" => polys(&[
            vec![
                [4., 1.75],
                [9.5, 1.75],
                [12.5, 4.75],
                [12.5, 14.25],
                [4., 14.25],
                [4., 1.75],
            ],
            vec![[9.5, 1.75], [9.5, 4.75], [12.5, 4.75]],
            vec![[6., 8.], [10.5, 8.]],
            vec![[6., 10.5], [10.5, 10.5]],
        ]),
        "search" => {
            let mut v = circle(7., 7., 4.25);
            v.extend(polys(&[vec![[10.25, 10.25], [14., 14.]]]));
            v
        }
        "execute" => polys(&[
            vec![
                [1.75, 2.75],
                [14.25, 2.75],
                [14.25, 13.25],
                [1.75, 13.25],
                [1.75, 2.75],
            ],
            vec![[4.5, 6.5], [6.5, 8.25], [4.5, 10.]],
            vec![[8.5, 10.5], [11.5, 10.5]],
        ]),
        "edit" => polys(&[
            vec![
                [10.5, 2.5],
                [13.5, 5.5],
                [6., 13.],
                [3., 13.],
                [3., 10.],
                [10.5, 2.5],
            ],
            vec![[9., 4.], [12., 7.]],
        ]),
        "think" => polys(&[
            vec![[8., 1.75], [8., 3.75]],
            vec![[8., 12.25], [8., 14.25]],
            vec![[1.75, 8.], [3.75, 8.]],
            vec![[12.25, 8.], [14.25, 8.]],
            vec![[3.6, 3.6], [5., 5.]],
            vec![[11., 11.], [12.4, 12.4]],
            vec![[3.6, 12.4], [5., 11.]],
            vec![[11., 5.], [12.4, 3.6]],
        ]),
        "check" => polys(&[vec![[3.5, 8.5], [6.5, 11.5], [12.5, 4.5]]]),
        "x" => polys(&[
            vec![[4.5, 4.5], [11.5, 11.5]],
            vec![[11.5, 4.5], [4.5, 11.5]],
        ]),
        "shield" => polys(&[
            vec![
                [8., 1.75],
                [13.25, 4.],
                [13.25, 8.],
                [11.5, 12.2],
                [8., 14.25],
                [4.5, 12.2],
                [2.75, 8.],
                [2.75, 4.],
                [8., 1.75],
            ],
            vec![[5.75, 8.], [7.35, 9.6], [10.25, 6.5]],
        ]),
        "shieldx" => polys(&[
            vec![
                [8., 1.75],
                [13.25, 4.],
                [13.25, 8.],
                [11.5, 12.2],
                [8., 14.25],
                [4.5, 12.2],
                [2.75, 8.],
                [2.75, 4.],
                [8., 1.75],
            ],
            vec![[6.25, 6.25], [9.75, 9.75]],
            vec![[9.75, 6.25], [6.25, 9.75]],
        ]),
        "chev" => polys(&[vec![[6., 4.], [10., 8.], [6., 12.]]]),
        "send" => polys(&[
            vec![[8., 13.], [8., 3.]],
            vec![[3.5, 7.5], [8., 3.], [12.5, 7.5]],
        ]),
        // web harness.js icons: plug (ACP agents, /profiles) and stop (/interrupt)
        "plug" => polys(&[
            vec![[6., 2.], [6., 5.]],
            vec![[10., 2.], [10., 5.]],
            vec![
                [4.5, 5.],
                [11.5, 5.],
                [11.5, 7.5],
                [10.5, 9.9],
                [8., 11.],
                [5.5, 9.9],
                [4.5, 7.5],
                [4.5, 5.],
            ],
            vec![[8., 11.], [8., 14.]],
        ]),
        "stop" => polys(&[vec![
            [4.5, 4.5],
            [11.5, 4.5],
            [11.5, 11.5],
            [4.5, 11.5],
            [4.5, 4.5],
        ]]),
        "branch" => {
            let mut v = circle(4.5, 3.5, 1.5);
            v.extend(circle(4.5, 12.5, 1.5));
            v.extend(circle(11.5, 5.5, 1.5));
            v.extend(polys(&[
                vec![[4.5, 5.], [4.5, 11.]],
                vec![[11.5, 7.], [9., 9.5], [4.5, 11.]],
            ]));
            v
        }
        _ => polys(&[vec![
            [8., 2.],
            [9.6, 6.4],
            [14., 8.],
            [9.6, 9.6],
            [8., 14.],
            [6.4, 9.6],
            [2., 8.],
            [6.4, 6.4],
            [8., 2.],
        ]]),
    };
    let k = size / 16.;
    let scaled: Vec<P> = cmds
        .into_iter()
        .map(|c| match c {
            P::Move(p) => P::Move([p[0] * k, p[1] * k]),
            P::Line(p) => P::Line([p[0] * k, p[1] * k]),
            P::Cubic(ps) => P::Cubic(ps.map(|p| [p[0] * k, p[1] * k])),
        })
        .collect();
    c::Element::Path(c::Path {
        size: [size, size],
        points: Arc::from(scaled),
        stroke: PathStroke {
            width: weight * k,
            cap: LineCap::Round,
            join: LineJoin::Round,
            ..Default::default()
        },
        paint: paint(color),
        fill: None,
        opacity: 1.,
    })
}
/// A centred icon in a square tile (tool kind, status disc).
pub fn tile(icon_el: c::Element, side: f32, radius: f32, bg: Rgba) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.material = Some(ui::Material::Flat);
    style.overrides.fill = Some(paint(bg));
    style.overrides.radii = Some([ui::Length::Px(radius); 4]);
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(0.),
        paint: paint([0.; 4]),
        outside: false,
    });
    style.overrides.width = Some(ui::Length::Px(side));
    style.overrides.height = Some(ui::Length::Px(side));
    style.overrides.padding = Some([ui::Length::Px(0.); 4]);
    let mut b = c::Box::new(style, vec![spacer(), fixed(icon_el), spacer()]);
    b.direction = Axis::Row;
    b.align = c::Alignment::Center;
    c::Element::Box(b)
}

// ---------- syntax: the web highlighter's token classes ----------
const KW: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "if", "else", "for", "in", "return",
    "match", "self", "Self", "super", "crate", "true", "false", "const", "static", "enum", "while",
    "loop", "as", "where", "async", "await", "move", "trait", "type", "unsafe", "ref", "dyn",
    "function", "var", "import", "export", "from", "new", "class", "def", "None", "null",
];
pub fn syntax(line: &str, ink: &Ink) -> Vec<(String, Rgba, u16, bool)> {
    let tint = |h: &str, t: f32| mix(hex(h), ink.fg, t);
    let mut out: Vec<(String, Rgba, u16, bool)> = vec![];
    let push = |out: &mut Vec<(String, Rgba, u16, bool)>, s: &str, c: Rgba, italic: bool| {
        if let Some(last) = out.last_mut()
            && last.1 == c
            && last.3 == italic
        {
            last.0.push_str(s);
        } else {
            out.push((s.to_owned(), c, 400, italic));
        }
    };
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let rest = &line[i..];
        let (len, color, italic) = if rest.starts_with("//") {
            (rest.len(), ink.dim, true)
        } else if rest.starts_with("#[") || rest.starts_with("#![") {
            (
                rest.find(']').map_or(rest.len(), |e| e + 1),
                tint("#D4A017", 0.75),
                false,
            )
        } else if b[i] == b'"' {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            ((j + 1).min(b.len()) - i, tint("#2EC46A", 0.7), false)
        } else if b[i] == b'\''
            && b.len() > i + 2
            && (b[i + 2] == b'\'' || (b[i + 1] == b'\\' && b.len() > i + 3))
        {
            (
                if b[i + 1] == b'\\' { 4 } else { 3 },
                tint("#2EC46A", 0.7),
                false,
            )
        } else if b[i].is_ascii_digit() {
            let n = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
                .unwrap_or(rest.len());
            (n, tint("#FF8A2B", 0.75), false)
        } else if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let n = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let word = &rest[..n];
            if rest[n..].starts_with('!') {
                (n + 1, tint("#1FA8E0", 0.75), false)
            } else if KW.contains(&word) {
                (n, tint("#FF4FA3", 0.72), false)
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                (n, tint("#9B5CFF", 0.72), false)
            } else {
                (n, ink.fg, false)
            }
        } else {
            (rest.chars().next().map_or(1, char::len_utf8), ink.fg, false)
        };
        let len = len.max(1).min(rest.len());
        push(&mut out, &rest[..len], color, italic);
        i += len;
    }
    out
}

// ---------- diffs: a minimal per-hunk diff from any unified text ----------
// The core's edit diff can mark a whole file -/+; both sides are present, so an LCS over
// each hunk recovers the real change. Translated from harness.js lcsOps.
pub struct DiffRow {
    pub op: char,
    pub old: Option<usize>,
    pub new: Option<usize>,
    pub text: String,
}
pub fn minimal_diff(unified: &str) -> Vec<Vec<DiffRow>> {
    let mut hunks: Vec<(usize, usize, Vec<String>, Vec<String>)> = vec![];
    for l in unified.lines() {
        if l.starts_with("--- ") || l.starts_with("+++ ") || l.starts_with('\\') {
            continue;
        }
        if let Some(h) = l.strip_prefix("@@ -") {
            let mut parts = h.split(' ');
            let a = parts
                .next()
                .and_then(|s| s.split(',').next()?.parse().ok())
                .unwrap_or(1);
            let b = parts
                .next()
                .and_then(|s| s.trim_start_matches('+').split(',').next()?.parse().ok())
                .unwrap_or(1);
            hunks.push((a, b, vec![], vec![]));
            continue;
        }
        let Some(h) = hunks.last_mut() else { continue };
        match l.chars().next() {
            Some('-') => h.2.push(l[1..].to_owned()),
            Some('+') => h.3.push(l[1..].to_owned()),
            _ => {
                let t = l.get(1..).unwrap_or("").to_owned();
                h.2.push(t.clone());
                h.3.push(t);
            }
        }
    }
    hunks
        .into_iter()
        .map(|(a0, b0, old, new)| {
            let (n, m) = (old.len(), new.len());
            let mut rows = vec![];
            if n * m > 4_000_000 {
                rows.extend(old.iter().map(|t| ('-', t)));
                rows.extend(new.iter().map(|t| ('+', t)));
            } else {
                let mut dp = vec![vec![0u32; m + 1]; n + 1];
                for i in (0..n).rev() {
                    for j in (0..m).rev() {
                        dp[i][j] = if old[i] == new[j] {
                            dp[i + 1][j + 1] + 1
                        } else {
                            dp[i + 1][j].max(dp[i][j + 1])
                        };
                    }
                }
                let (mut i, mut j) = (0, 0);
                while i < n && j < m {
                    if old[i] == new[j] {
                        rows.push((' ', &old[i]));
                        i += 1;
                        j += 1;
                    } else if dp[i + 1][j] >= dp[i][j + 1] {
                        rows.push(('-', &old[i]));
                        i += 1;
                    } else {
                        rows.push(('+', &new[j]));
                        j += 1;
                    }
                }
                rows.extend(old[i..].iter().map(|t| ('-', t)));
                rows.extend(new[j..].iter().map(|t| ('+', t)));
            }
            let (mut a, mut b) = (a0, b0);
            rows.into_iter()
                .map(|(op, t)| {
                    let r = DiffRow {
                        op,
                        old: (op != '+').then_some(a),
                        new: (op != '-').then_some(b),
                        text: t.clone(),
                    };
                    if op != '+' {
                        a += 1;
                    }
                    if op != '-' {
                        b += 1;
                    }
                    r
                })
                .collect()
        })
        .collect()
}
pub fn diff_stat(unified: &str) -> (usize, usize) {
    minimal_diff(unified)
        .iter()
        .flatten()
        .fold((0, 0), |(a, d), r| match r.op {
            '+' => (a + 1, d),
            '-' => (a, d + 1),
            _ => (a, d),
        })
}
fn gutter(n: Option<usize>, ink: &Ink) -> c::Item {
    c::Item {
        element: runs(
            vec![(
                n.map_or(String::new(), |n| n.to_string()),
                alpha(ink.dim, 0.55),
                400,
                false,
            )],
            12.,
            true,
            false,
        ),
        extent: c::Extent::Fixed(30.),
    }
}
fn code_frame(rows: Vec<c::Item>, ink: &Ink) -> c::Element {
    frame(
        Frame {
            fill: alpha(ink.bg, 0.55),
            border: Some(alpha(ink.fg, 0.10)),
            radius: [8.; 4],
            padding: [4., 0., 4., 0.],
            ..Default::default()
        },
        rows,
    )
}
pub fn diff_view(key: &str, unified: &str, ink: &Ink) -> c::Element {
    const CTX: usize = 2;
    let mut out: Vec<c::Item> = vec![];
    for rows in minimal_diff(unified) {
        let near: Vec<bool> = (0..rows.len())
            .map(|i| {
                rows[i.saturating_sub(CTX)..(i + CTX + 1).min(rows.len())]
                    .iter()
                    .any(|r| r.op != ' ')
            })
            .collect();
        let mut skipped = 0;
        let fold = |out: &mut Vec<c::Item>, skipped: &mut usize| {
            if *skipped > 0 {
                out.push(
                    frame(
                        Frame {
                            fill: alpha(ink.fg, 0.04),
                            padding: [1., 0., 1., 64.],
                            ..Default::default()
                        },
                        vec![
                            line(
                                format!(
                                    "⋯ {} unchanged line{}",
                                    skipped,
                                    if *skipped == 1 { "" } else { "s" }
                                ),
                                11.,
                                400,
                                ink.dim,
                                false,
                            )
                            .into(),
                        ],
                    )
                    .into(),
                );
                *skipped = 0;
            }
        };
        for (i, r) in rows.iter().enumerate() {
            if !near[i] {
                skipped += 1;
                continue;
            }
            fold(&mut out, &mut skipped);
            let (bg, mark, mark_c) = match r.op {
                '+' => (alpha(ink.ok, 0.13), "+", ink.ok),
                '-' => (alpha(ink.bad, 0.12), "−", ink.bad),
                _ => ([0.; 4], " ", ink.dim),
            };
            let mut parts = vec![(format!("{mark} "), mark_c, 400, false)];
            parts.extend(syntax(&r.text, ink));
            out.push(
                named(
                    format!("{key}/line/{}", out.len()),
                    frame(
                        Frame {
                            fill: bg,
                            row: true,
                            stretch: true,
                            ..Default::default()
                        },
                        vec![
                            fixed(bar(3., if r.op == ' ' { [0.; 4] } else { mark_c }))
                                .with_width(3.),
                            gutter(r.old, ink),
                            gutter(r.new, ink),
                            fill(runs(parts, 12., true, false)),
                        ],
                    ),
                )
                .into(),
            );
        }
        fold(&mut out, &mut skipped);
    }
    named(key.to_owned(), code_frame(out, ink))
}
trait Width {
    fn with_width(self, w: f32) -> c::Item;
}
impl Width for c::Item {
    fn with_width(mut self, w: f32) -> c::Item {
        self.extent = c::Extent::Fixed(w);
        self
    }
}
pub fn source_view(numbered: &str, limit: usize, ink: &Ink) -> (c::Element, usize) {
    let rows: Vec<(usize, &str)> = numbered
        .lines()
        .enumerate()
        .map(|(i, l)| match l.split_once(": ") {
            Some((n, t)) if n.parse::<usize>().is_ok() => (n.parse().unwrap(), t),
            _ => match l.strip_suffix(':').and_then(|n| n.parse().ok()) {
                Some(n) => (n, ""),
                None => (i + 1, l),
            },
        })
        .collect();
    let total = rows.len();
    let items = rows
        .into_iter()
        .take(limit)
        .map(|(n, t)| {
            hstack(
                vec![
                    gutter(Some(n), ink),
                    fill(runs(syntax(t, ink), 12., true, false)),
                ],
                8.,
            )
            .into()
        })
        .collect();
    (code_frame(items, ink), total)
}
/// Test/compiler output coloured by verdict, as a person scans it.
pub fn term_output(out: &str, ink: &Ink) -> Vec<c::Item> {
    let ok = hex("#5BE08C");
    let bad = hex("#FF7A7A");
    const SHOWN: usize = 14;
    let lines: Vec<&str> = out.trim_matches('\n').lines().collect();
    let more = lines.len().saturating_sub(SHOWN + 2);
    let mut items: Vec<c::Item> = lines[..if more > 0 { SHOWN } else { lines.len() }]
        .iter()
        .map(|l| {
            let t = l.trim_start();
            let color = if l.ends_with("... ok")
                || t.starts_with("test result: ok")
                || t.starts_with("Finished")
            {
                ok
            } else if l.contains("FAILED")
                || l.contains("panicked")
                || t.starts_with("error")
                || t.starts_with("failures:")
            {
                bad
            } else if t.starts_with("warning") || t.starts_with("note:") {
                hex("#FFC53D")
            } else if t.starts_with("running ") || t.starts_with("Compiling") {
                hex("#7d8597")
            } else {
                hex("#c9cfdc")
            };
            runs(vec![((*l).to_owned(), color, 400, false)], 12., true, true).into()
        })
        .collect();
    if more > 0 {
        items.push(
            runs(
                vec![(
                    format!("⋯ {more} more lines"),
                    alpha(ink.dim, 0.8),
                    400,
                    true,
                )],
                11.5,
                false,
                false,
            )
            .into(),
        );
    }
    items
}
pub fn duration(ms: u64) -> String {
    match ms {
        0..1000 => format!("{}ms", ms.max(1)),
        1000..10000 => format!("{:.1}s", ms as f64 / 1000.),
        _ => format!("{}s", ms / 1000),
    }
}
pub struct Receipt {
    pub ok: bool,
    pub reason: String,
    pub took: Option<u64>,
    pub rounds: usize,
    pub tools: usize,
    pub edits: usize,
    pub tokens: Option<(u64, u64)>,
}
pub fn receipt(r: Receipt, ink: &Ink) -> c::Element {
    let c = if r.ok { ink.ok } else { ink.bad };
    let mut kids = vec![
        fixed(tile(
            icon(if r.ok { "check" } else { "x" }, ink.surface, 11., 2.4),
            18.,
            9.,
            c,
        )),
        fixed(line(
            if r.ok { "Done".to_owned() } else { r.reason },
            12.5,
            650,
            ink.fg,
            false,
        )),
    ];
    let mut facts = vec![];
    if let Some(t) = r.took {
        facts.push(duration(t));
    }
    facts.push(format!(
        "{} round{}",
        r.rounds,
        if r.rounds == 1 { "" } else { "s" }
    ));
    facts.push(format!(
        "{} tool{}",
        r.tools,
        if r.tools == 1 { "" } else { "s" }
    ));
    if r.edits > 0 {
        facts.push(format!(
            "{} edit{}",
            r.edits,
            if r.edits == 1 { "" } else { "s" }
        ));
    }
    if let Some((i, o)) = r.tokens.filter(|t| t.0 > 0) {
        facts.push(format!("{} in · {} out", kilo(i), kilo(o)));
    }
    kids.extend(
        facts
            .into_iter()
            .map(|t| fixed(line(t, 11.5, 400, ink.dim, false))),
    );
    frame(
        Frame {
            fill: alpha(c, 0.08),
            border: Some(alpha(c, 0.25)),
            radius: [11.; 4],
            padding: [8., 12., 8., 12.],
            gap: 12.,
            row: true,
            ..Default::default()
        },
        kids,
    )
}
pub fn kilo(n: u64) -> String {
    match n {
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1e6),
        n if n >= 10_000 => format!("{}k", n / 1000),
        n if n >= 1000 => format!("{:.1}k", n as f64 / 1e3),
        n => n.to_string(),
    }
}
pub fn terminal(
    key: &str,
    command: &str,
    output: Option<Vec<c::Item>>,
    exit: Option<i32>,
    ink: &Ink,
) -> c::Element {
    let mut rows: Vec<c::Item> = vec![
        frame(
            Frame {
                fill: alpha(hex("#ffffff"), 0.035),
                padding: [7., 10., 7., 10.],
                gap: 8.,
                row: true,
                ..Default::default()
            },
            vec![
                fixed(runs(
                    vec![("$".into(), ink.seat, 700, false)],
                    12.,
                    true,
                    false,
                )),
                fill(runs(
                    vec![(command.to_owned(), hex("#ffffff"), 400, false)],
                    12.,
                    true,
                    true,
                )),
            ],
        )
        .into(),
    ];
    let output = match (output, exit) {
        // the exit status is part of the terminal's own record, readable beside its output
        (out, Some(code)) => {
            let mut out = out.unwrap_or_default();
            out.push(
                runs(
                    vec![(
                        format!("exit {code}"),
                        if code == 0 {
                            hex("#5BE08C")
                        } else {
                            hex("#FF7A7A")
                        },
                        600,
                        false,
                    )],
                    11.,
                    true,
                    false,
                )
                .into(),
            );
            Some(out)
        }
        (out, None) => out,
    };
    if let Some(out) = output {
        rows.push(
            named(
                format!("{key}/output"),
                frame(
                    Frame {
                        padding: [7., 10., 7., 10.],
                        ..Default::default()
                    },
                    out,
                ),
            )
            .into(),
        );
    }
    named(
        key.to_owned(),
        frame(
            Frame {
                fill: hex(TERM_BG),
                border: Some(alpha(hex("#ffffff"), 0.07)),
                radius: [8.; 4],
                ..Default::default()
            },
            rows,
        ),
    )
}

// ---------- tool calls ----------
pub struct Tool<'a> {
    pub verb: String,
    pub target: String,
    pub mono: bool,
    pub kind: &'static str,
    pub args: serde_json::Value,
    pub out: String,
    pub name: &'a str,
}
pub fn describe<'a>(
    name: &'a str,
    tool_kind: ToolKind,
    input_raw: &str,
    output: &[OutputBlock],
    cwd: &str,
) -> Tool<'a> {
    let args: serde_json::Value =
        serde_json::from_str(input_raw).unwrap_or(serde_json::Value::Null);
    let s = |k: &str| {
        args.get(k)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_owned()
    };
    let short = |p: String| {
        p.strip_prefix(&format!("{cwd}/"))
            .map(str::to_owned)
            .unwrap_or(p)
    };
    let out = output
        .iter()
        .filter_map(|b| match b {
            OutputBlock::Text { text } | OutputBlock::Error { text } => Some(text.clone()),
            OutputBlock::Diff { unified, .. } => Some(unified.clone()),
            OutputBlock::Json { value } => Some(value.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let kind = match tool_kind {
        ToolKind::Read => "read",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Edit => "edit",
        ToolKind::Fetch => "fetch",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        _ => "other",
    };
    let (verb, target, mono, kind) = match name {
        "read" => ("Read".into(), short(s("path")), false, "read"),
        "grep" => ("Search".into(), s("pattern"), true, "search"),
        "glob" => ("Find".into(), s("pattern"), true, "search"),
        "bash" => ("Run".into(), s("command"), true, "execute"),
        "exec" => {
            let mut v = vec![s("program")];
            if let Some(a) = args.get("args").and_then(serde_json::Value::as_array) {
                v.extend(a.iter().filter_map(|x| x.as_str().map(str::to_owned)));
            }
            ("Run".into(), v.join(" "), true, "execute")
        }
        "edit" | "apply_patch" => ("Edit".into(), short(s("path")), false, "edit"),
        "write" => ("Write".into(), short(s("path")), false, "edit"),
        n if n.starts_with("mcp__") => {
            let mut p = n.splitn(3, "__").skip(1);
            let server = p.next().unwrap_or("").to_owned();
            (
                p.next().unwrap_or(n).replace('_', " "),
                server,
                false,
                "other",
            )
        }
        n => (
            if n.is_empty() {
                "Tool".into()
            } else {
                n.to_owned()
            },
            String::new(),
            false,
            kind,
        ),
    };
    Tool {
        verb,
        target,
        mono,
        kind,
        args,
        out,
        name,
    }
}
#[derive(Clone, Copy)]
pub enum Status {
    Running,
    Asking,
    Done,
    Failed,
    Denied,
}
pub fn status_disc(st: &Status, ink: &Ink) -> c::Element {
    match st {
        Status::Running => runs(
            vec![
                ("running ".into(), ink.dim, 500, false),
                ("•••".into(), ink.seat, 700, false),
            ],
            11.,
            false,
            false,
        ),
        Status::Asking => icon("shield", ink.ask, 14., 1.6),
        Status::Done => tile(icon("check", ink.surface, 11., 2.4), 18., 9., ink.ok),
        Status::Failed | Status::Denied => tile(icon("x", ink.surface, 11., 2.4), 18., 9., ink.bad),
    }
}
pub fn badge(
    decision: Option<&ApprovalDecision>,
    by: Option<&Resolver>,
    ink: &Ink,
) -> Option<c::Element> {
    let (name, label, c) = match decision? {
        ApprovalDecision::Deny { .. } => (
            "shieldx",
            if matches!(by, Some(Resolver::Unattended)) {
                "denied · no approver"
            } else {
                "denied"
            },
            ink.bad,
        ),
        ApprovalDecision::AllowAlways => ("shield", "always", ink.ok),
        _ => (
            "shield",
            if matches!(by, Some(Resolver::Rule { .. })) {
                "auto"
            } else {
                "allowed"
            },
            ink.ok,
        ),
    };
    Some(frame(
        Frame {
            fill: alpha(c, 0.13),
            radius: [999.; 4],
            padding: [1., 7., 1., 5.],
            gap: 4.,
            row: true,
            ..Default::default()
        },
        vec![
            fixed(icon(name, mix(c, ink.fg, 0.85), 12., 1.5)),
            fixed(line(label, 10.5, 600, mix(c, ink.fg, 0.85), false)),
        ],
    ))
}
pub struct Head {
    pub key: String,
    pub tool_kind: &'static str,
    pub verb: String,
    pub target: String,
    pub mono: bool,
    pub stat: Option<(usize, usize)>,
    pub meta: Option<(String, Option<bool>)>,
    pub badge: Option<c::Element>,
    pub dur: Option<String>,
    pub status: Status,
}
pub fn head(h: Head, ink: &Ink) -> c::Element {
    let kc = kind_color(h.tool_kind);
    let mut kids = vec![
        fixed(tile(
            icon(h.tool_kind, kc, 14., 1.5),
            24.,
            7.,
            alpha(kc, 0.16),
        )),
        fixed(line(h.verb, 13., 650, ink.fg, false)),
        fill(line(
            h.target,
            if h.mono { 12. } else { 13. },
            400,
            alpha(ink.fg, 0.82),
            h.mono,
        )),
    ];
    if let Some((a, d)) = h.stat {
        kids.push(fixed(runs(
            vec![
                (format!("+{a} "), ink.ok, 600, false),
                (format!("−{d}"), ink.bad, 600, false),
            ],
            11.5,
            true,
            false,
        )));
    }
    if let Some((m, good)) = h.meta {
        let c = match good {
            Some(true) => ink.ok,
            Some(false) => ink.bad,
            None => ink.dim,
        };
        kids.push(fixed(line(m, 11.5, 400, c, false)));
    }
    if let Some(b) = h.badge {
        kids.push(fixed(b));
    }
    if let Some(d) = h.dur {
        kids.push(fixed(line(d, 11., 400, ink.dim, false)));
    }
    let status_key = h.key.replacen("toolhead/", "tool/", 1) + "/status";
    kids.push(fixed(named(status_key, status_disc(&h.status, ink))));
    named(
        h.key,
        frame(
            Frame {
                padding: [7., 10., 7., 10.],
                gap: 8.,
                row: true,
                ..Default::default()
            },
            kids,
        ),
    )
}
pub fn card(status: &Status, children: Vec<c::Item>, ink: &Ink) -> c::Element {
    let (fill_c, border) = match status {
        Status::Failed | Status::Denied => (alpha(ink.bad, 0.06), alpha(ink.bad, 0.40)),
        Status::Asking => (alpha(ink.fg, 0.04), alpha(ink.ask, 0.60)),
        Status::Running => (alpha(ink.fg, 0.04), alpha(ink.seat, 0.70)),
        _ => (alpha(ink.fg, 0.04), alpha(ink.fg, 0.10)),
    };
    frame(
        Frame {
            fill: fill_c,
            border: Some(border),
            radius: [11.; 4],
            ..Default::default()
        },
        children,
    )
}
pub fn body(children: Vec<c::Item>) -> c::Item {
    frame(
        Frame {
            padding: [0., 10., 10., 10.],
            gap: 6.,
            ..Default::default()
        },
        children,
    )
    .into()
}
pub fn hits(out: &str, pattern: &str, cwd: &str, ink: &Ink) -> c::Element {
    let re = regex::Regex::new(pattern).ok();
    let rows = out
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut parts = l.splitn(3, ':');
            let (f, n, t) = (
                parts.next().unwrap_or(""),
                parts.next().unwrap_or(""),
                parts.next().unwrap_or(""),
            );
            let f = f.strip_prefix(&format!("{cwd}/")).unwrap_or(f);
            let mut spans = vec![
                (f.to_owned(), ink.fg, 400, false),
                (format!(":{n}  "), ink.seat, 400, false),
            ];
            let t = t.trim();
            let mut at = 0;
            if let Some(re) = &re {
                for m in re.find_iter(t) {
                    spans.push((t[at..m.start()].to_owned(), ink.dim, 400, false));
                    spans.push((m.as_str().to_owned(), ink.fg, 700, false));
                    at = m.end();
                }
            }
            spans.push((t[at..].to_owned(), ink.dim, 400, false));
            frame(
                Frame {
                    fill: alpha(ink.fg, 0.04),
                    radius: [6.; 4],
                    padding: [3., 8., 3., 8.],
                    ..Default::default()
                },
                vec![runs(spans, 12., true, false).into()],
            )
            .into()
        })
        .collect();
    vstack(rows, 2.)
}

// ---------- conversation blocks ----------
pub fn user_bubble(text_value: &str, ink: &Ink) -> c::Element {
    let bubble = frame(
        Frame {
            fill: mix(ink.seat, ink.surface, 0.2),
            border: Some(alpha(ink.seat, 0.35)),
            radius: [14., 14., 4., 14.],
            padding: [9., 13., 9., 13.],
            gap: 2.,
            ..Default::default()
        },
        vec![
            line("YOU", 10.5, 600, mix(ink.seat, ink.fg, 0.7), false).into(),
            c::Element::Text(text(text_value, 13.5, 400, ink.fg, false)).into(),
        ],
    );
    hstack(
        vec![
            spacer(),
            c::Item {
                element: bubble,
                extent: c::Extent::Content {
                    grow: 0.,
                    shrink: 1.,
                    min: 0.,
                    max: None,
                    max_fraction: Some(0.86),
                },
            },
        ],
        0.,
    )
}
pub fn thought_row(
    key: &str,
    live: bool,
    took: Option<u64>,
    gist: &str,
    open: bool,
    ink: &Ink,
) -> c::Element {
    let label = match took {
        _ if live => "Thinking".to_owned(),
        Some(ms) if ms >= 100 => format!("Thought for {}", duration(ms)),
        _ => "Thought".to_owned(),
    };
    named(
        key.to_owned(),
        hstack(
            vec![
                fixed(icon(
                    "think",
                    if live { ink.seat } else { ink.dim },
                    14.,
                    1.5,
                )),
                fixed(line(label, 12.5, 650, ink.fg, false)),
                fill(line(gist, 12.5, 400, alpha(ink.dim, 0.85), false)),
                fixed(icon(
                    if open { "x" } else { "chev" },
                    alpha(ink.dim, 0.7),
                    12.,
                    1.5,
                )),
            ],
            7.,
        ),
    )
}
pub fn thought_body(child: c::Element, ink: &Ink) -> c::Element {
    let gap = 12.;
    frame(
        Frame {
            gap,
            row: true,
            stretch: true,
            ..Default::default()
        },
        vec![
            fixed(bar(2., alpha(ink.seat, 0.45))).with_width(2.),
            fill(child),
        ],
    )
}
pub fn notice(text_value: &str, level: &Level, ink: &Ink) -> c::Element {
    let color = match level {
        Level::Warn => ink.ask,
        Level::Error => ink.bad,
        _ => ink.dim,
    };
    hstack(
        vec![
            fixed(dot(6., color)),
            fill(line(text_value, 11.5, 400, ink.dim, false)),
        ],
        7.,
    )
}

/// The session bar: mark, name, profile chip, folder chip, live phase, and the three
/// controls the window binds by name (jump, interrupt, stop).
pub fn header(
    profile: Option<String>,
    folder: Option<String>,
    status: &str,
    live: bool,
    menu_open: bool,
    ink: &Ink,
) -> c::Element {
    let chip = |kids: Vec<c::Item>| {
        frame(
            Frame {
                fill: alpha(ink.fg, 0.05),
                radius: [999.; 4],
                padding: [1., 8., 1., 8.],
                gap: 5.,
                row: true,
                ..Default::default()
            },
            kids,
        )
    };
    let mut kids = vec![
        fixed(tile(
            runs(
                vec![("b".into(), ink.surface, 800, false)],
                12.,
                false,
                false,
            ),
            20.,
            6.,
            ink.seat,
        )),
        fixed(line("bboxy", 13., 700, ink.fg, false)),
    ];
    if let Some(p) = profile {
        // web .hx-model: the profile chip opens the switcher
        kids.push(fixed(named(
            "profile",
            frame(
                Frame {
                    fill: if menu_open {
                        alpha(ink.seat, 0.1)
                    } else {
                        alpha(ink.fg, 0.05)
                    },
                    border: Some(if menu_open {
                        alpha(ink.seat, 0.5)
                    } else {
                        [0.; 4]
                    }),
                    radius: [999.; 4],
                    padding: [1., 8., 1., 8.],
                    gap: 5.,
                    row: true,
                    ..Default::default()
                },
                vec![
                    fixed(dot(6., ink.seat)),
                    fixed(line(
                        p,
                        11.5,
                        400,
                        if menu_open { ink.fg } else { ink.dim },
                        false,
                    )),
                    fixed(down_chevron(alpha(ink.dim, 0.7))),
                ],
            ),
        )));
    }
    if let Some(f) = folder {
        kids.push(fixed(chip(vec![
            fixed(icon("branch", ink.dim, 12., 1.5)),
            fixed(line(f, 11.5, 400, ink.dim, false)),
        ])));
    }
    kids.push(spacer());
    kids.push(fixed(dot(7., if live { ink.seat } else { ink.dim })));
    kids.push(fixed(line(
        status.to_owned(),
        11.5,
        400,
        if live { ink.fg } else { ink.dim },
        false,
    )));
    for (key, label) in [
        ("jump", "Latest"),
        ("interrupt", "Interrupt"),
        ("stop", "Stop"),
    ] {
        kids.push(fixed(named(
            key,
            frame(
                Frame {
                    radius: [6.; 4],
                    padding: [2., 6., 2., 6.],
                    ..Default::default()
                },
                vec![line(label, 11.5, 500, alpha(ink.dim, 0.85), false).into()],
            ),
        )));
    }
    frame(
        Frame {
            fill: alpha(ink.seat, 0.06),
            padding: [8., 12., 8., 12.],
            gap: 8.,
            row: true,
            ..Default::default()
        },
        kids,
    )
}

/// Composer states from harness-spec.md (Composer table): the session phase decides `run`
/// and `ask`; otherwise the draft and focus decide `typing`, `focus` or `idle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerState {
    Idle,
    Focus,
    Typing,
    Run,
    Ask,
}
impl ComposerState {
    pub fn of(live: bool, ask: bool, text: bool, focused: bool) -> Self {
        if ask {
            Self::Ask
        } else if live {
            Self::Run
        } else if text {
            Self::Typing
        } else if focused {
            Self::Focus
        } else {
            Self::Idle
        }
    }
}

/// The composer: one rounded box with the draft, two quiet chips, a state hint and the round
/// send button. Translated from harness.js composer() and lab.css .hx-comp/.hx-box/.hx-send
/// (bboxy450-term lab/web, harness-spec.md; https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md).
/// Node names keep the toolkit composer's hit contract (`composer`, `composer/text`,
/// `composer/body-viewport`, `composer/send`, `composer/attach`, `composer/priority`,
/// `composer/address`), so `composer::Regions` still drives clicks, the caret and the IME.
pub fn composer(
    view: &ui::modules::composer::Composer,
    live: bool,
    ask: bool,
    focused: bool,
    ctx: ui::Context,
    width: f32,
    m: &dyn c::Measure,
) -> Result<c::Element, String> {
    use ComposerState as S;
    use bbt_ui::widgets::text_area::TextArea;
    use bbt_ui_model::layout::Rect;
    const LINE: f32 = 20.;
    const MAX_LINES: f32 = 8.;
    let ink = ink(ctx);
    let has_text = !view.text.is_empty();
    let state = ComposerState::of(live, ask, has_text, focused);
    let box_w = (width - 24.).clamp(1., 860.);
    let side = ((width - box_w) / 2.).max(0.);
    let inner = (box_w - 22.).max(1.);
    let empty = !has_text && view.edit.as_ref().is_none_or(|e| e.preedit.is_none());
    let ph_ink = if state == S::Ask {
        alpha(ink.ask, 0.8)
    } else {
        alpha(ink.dim, 0.7)
    };
    let mut body = TextArea {
        value: view.text.clone(),
        placeholder: match state {
            S::Run => "Queue a follow-up…",
            S::Ask => "Answer the approval above",
            _ => "Ask bboxy, @ a file, / a command",
        }
        .into(),
        style: ui::Text {
            size: Some(13.),
            paint: paint(if empty { ph_ink } else { ink.fg }),
            line_height: LINE / 13.,
            ..Default::default()
        },
        edit: (view.focus && focused).then(|| view.edit.clone().unwrap_or_default()),
        min_lines: 1,
        wrap: true,
        name: "composer/text".into(),
    }
    .element(inner, ctx, m)?;
    if empty {
        tint_placeholder(&mut body, paint(ph_ink));
    }
    // Grow with the draft up to MAX_LINES, then scroll so the caret stays in view.
    let measured = body.layout(Rect::new(0., 0., inner, 0.), ctx, m)?;
    let height = measured.rect.h.clamp(LINE, LINE * MAX_LINES);
    fn caret(n: &c::Layout) -> Option<Rect> {
        if matches!(&n.kind, c::LayoutKind::Named(k) if k == "composer/text/caret") {
            return Some(n.rect);
        }
        n.children.iter().find_map(caret)
    }
    let bottom = caret(&measured).map_or(measured.rect.h, |r| r.bottom());
    let offset = view
        .body_scroll
        .unwrap_or((bottom - height).max(0.))
        .clamp(0., (measured.rect.h - height).max(0.));
    let viewport = named(
        "composer/body-viewport",
        c::Element::ScrollView(c::ScrollView {
            viewport: [inner, height],
            offset: [0., offset],
            content_width: Some(inner),
            content: Box::new(body),
        }),
    );
    let chip = |key: &str, label: &str| {
        named(
            key,
            frame(
                Frame {
                    fill: alpha(ink.fg, 0.07),
                    border: Some(alpha(ink.fg, 0.10)),
                    radius: [999.; 4],
                    padding: [0., 6., 0., 6.],
                    row: true,
                    ..Default::default()
                },
                vec![fixed(line(label, 10.5, 400, ink.dim, false))],
            ),
        )
    };
    let mut tools = vec![
        fixed(chip("composer/attach", "@ context")),
        fixed(chip("composer/priority", "queue")),
        spacer(),
    ];
    let hint = match state {
        S::Run if has_text => "⏎ queue · esc interrupt",
        S::Run => "esc interrupt",
        S::Typing => "⏎ send · ⇧⏎ newline",
        S::Ask => "1 allow · 2 always · 3 deny",
        S::Idle | S::Focus => "",
    };
    if !hint.is_empty() {
        let color = if state == S::Ask {
            ink.ask
        } else {
            alpha(ink.dim, 0.7)
        };
        tools.push(fixed(named(
            "composer/hint",
            line(hint, 10.5, 400, color, false),
        )));
    }
    // .hx-send: a 26 px circle. Empty: card-hi with a dim arrow; with text: seat with an
    // on-accent arrow and a glow; running and empty: a stop square on a hairline inset.
    let stop = state == S::Run && !has_text;
    let on_accent = hex("#FFFFFF");
    let mut send = frame(
        Frame {
            fill: if has_text {
                ink.seat
            } else {
                alpha(ink.fg, 0.07)
            },
            border: stop.then_some(alpha(ink.fg, 0.10)),
            radius: [13.; 4],
            padding: [if stop { 7. } else { 6. }; 4],
            ..Default::default()
        },
        vec![fixed(if stop {
            square(12., 6., 1.1, ink.fg)
        } else {
            icon("send", if has_text { on_accent } else { ink.dim }, 14., 2.2)
        })],
    );
    if let c::Element::Box(b) = &mut send {
        // Fixed 26 px whatever the inset border does to the padding box.
        b.style.overrides.width = Some(ui::Length::Px(26.));
        b.style.overrides.height = Some(ui::Length::Px(26.));
        if stop {
            b.style.overrides.padding = Some([ui::Length::Px(6.); 4]);
        }
        if has_text {
            b.style.overrides.shadow = Some(ui::Shadow::Custom {
                blur: 11.,
                color: alpha(ink.seat, 0.55),
            });
        }
    }
    tools.push(fixed(named("composer/send", send)));
    let mut kids = vec![];
    if let Some(n) = view.notice.as_ref().filter(|n| n.kind != "sent") {
        kids.push(fixed(named(
            "composer/notice",
            line(n.text.clone(), 11.5, 400, ink.ask, false),
        )));
    }
    // Zero-height address slot: routing stays in the draft model, the row is not shown.
    kids.push(c::Item {
        element: named("composer/address", c::Element::Spacer),
        extent: c::Extent::Fixed(0.),
    });
    kids.push(fixed(viewport));
    // The tools row sits 6 px under the draft (.hx-tools margin-top).
    kids.push(fixed(frame(
        Frame {
            padding: [6., 0., 0., 0.],
            gap: 6.,
            row: true,
            ..Default::default()
        },
        tools,
    )));
    let border = if state == S::Ask {
        alpha(ink.ask, 0.45)
    } else if focused {
        alpha(ink.seat, 0.55)
    } else {
        alpha(ink.fg, 0.10)
    };
    let mut panel = frame(
        Frame {
            fill: alpha(ink.fg, if focused { 0.05 } else { 0.04 }),
            border: Some(border),
            radius: [13.; 4],
            padding: [8., 10., 6., 10.],
            ..Default::default()
        },
        kids,
    );
    if let c::Element::Box(b) = &mut panel {
        b.gap = ui::Length::Px(0.);
        if focused {
            // Flat fallback: a solid seat-14 band 3 px outside the box, no blur.
            b.style.overrides.rings = Some(vec![ui::Border {
                width: ui::Length::Px(3.),
                paint: paint(alpha(ink.seat, 0.14)),
                outside: true,
            }]);
        }
    }
    if state == S::Run {
        // Running sweep, static flat fallback: a 2 px seat-55 line along the top edge,
        // inset by the corner radius as the web's overflow clip does.
        let h = panel.layout(Rect::new(0., 0., box_w, 0.), ctx, m)?.rect.h;
        let w = (box_w - 26.).max(0.);
        panel = c::Element::Layers {
            base: Box::new(panel),
            overlays: vec![c::Placed {
                content: named(
                    "composer/sweep",
                    c::Element::Rects(c::Rects {
                        size: [w, 2.],
                        rects: Arc::from([(
                            Rect::new(0., 0., w, 2.),
                            paint(alpha(ink.seat, 0.55)),
                        )]),
                        repeat: None,
                    }),
                ),
                left: 13.,
                bottom: h - 3.,
            }],
        };
    }
    Ok(frame(
        Frame {
            padding: [8., side, 12., side],
            ..Default::default()
        },
        vec![fixed(named("composer", panel))],
    ))
}
/// A filled rounded square centred in a `box_side` path (the web `stop` glyph).
fn square(box_side: f32, side: f32, r: f32, color: Rgba) -> c::Element {
    let (a, b) = ((box_side - side) / 2., (box_side + side) / 2.);
    let k = 0.552_284_8 * r;
    let mut path = vec![
        P::Move([a + r, a]),
        P::Line([b - r, a]),
        P::Cubic([[b - r + k, a], [b, a + r - k], [b, a + r]]),
        P::Line([b, b - r]),
        P::Cubic([[b, b - r + k], [b - r + k, b], [b - r, b]]),
        P::Line([a + r, b]),
        P::Cubic([[a + r - k, b], [a, b - r + k], [a, b - r]]),
        P::Line([a, a + r]),
        P::Cubic([[a, a + r - k], [a + r - k, a], [a + r, a]]),
    ];
    path.shrink_to_fit();
    c::Element::Path(c::Path {
        size: [box_side, box_side],
        points: Arc::from(path),
        stroke: PathStroke {
            width: 0.,
            ..Default::default()
        },
        paint: paint(color),
        fill: Some(paint(color)),
        opacity: 1.,
    })
}
/// TextArea paints its placeholder in the theme's muted ink; the web dims it further.
fn tint_placeholder(e: &mut c::Element, p: ui::Paint) {
    match e {
        c::Element::Text(t) => t.runs.iter_mut().for_each(|r| r.style.paint = p),
        c::Element::Named { content, .. } => tint_placeholder(content, p),
        c::Element::Layers { base, overlays } => {
            tint_placeholder(base, p);
            overlays
                .iter_mut()
                .for_each(|o| tint_placeholder(&mut o.content, p));
        }
        c::Element::Box(b) => b
            .children
            .iter_mut()
            .for_each(|i| tint_placeholder(&mut i.element, p)),
        _ => {}
    }
}

/// The web's rotated `chev` icon: a small down chevron.
fn down_chevron(color: Rgba) -> c::Element {
    c::Element::Path(c::Path {
        size: [10., 10.],
        points: Arc::from([
            P::Move([2.5, 3.75]),
            P::Line([5., 6.25]),
            P::Line([7.5, 3.75]),
        ]),
        stroke: PathStroke {
            width: 1.5,
            cap: LineCap::Round,
            join: LineJoin::Round,
            ..Default::default()
        },
        paint: paint(color),
        fill: None,
        opacity: 1.,
    })
}
/// A filled dot: a shape, not a painted box (it has no content slot).
pub fn dot(side: f32, color: Rgba) -> c::Element {
    let mut path = circle(side / 2., side / 2., side / 2.);
    path.shrink_to_fit();
    c::Element::Path(c::Path {
        size: [side, side],
        points: Arc::from(path),
        stroke: PathStroke {
            width: 0.,
            ..Default::default()
        },
        paint: paint(color),
        fill: Some(paint(color)),
        opacity: 1.,
    })
}
/// A vertical rule that stretches with its row.
pub fn bar(w: f32, color: Rgba) -> c::Element {
    c::Element::Rects(c::Rects {
        size: [w, 1.],
        rects: Arc::from([(bbt_ui_model::layout::Rect::new(0., 0., w, 1.), paint(color))]),
        repeat: Some((Axis::Column, 1.)),
    })
}
