// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Pure helpers shared by every face (the GUI cards and the seat's terminal face): what a
//! tool call is about, a minimal diff, highlighter token classes, terminal verdicts.
//! No UI toolkit here, so the seat builds without the GUI feature.
//! Translated from bboxy450-term lab/web/harness.js (describe, lcsOps, highlight, termOut).
//! https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use crate::protocol::*;

/// Tool kind colours (the web reference's `.hx-ki` palette), as hex.
pub fn kind_hex(kind: &str) -> &'static str {
    match kind {
        "read" => "#58D7FF",
        "search" => "#B07BFF",
        "execute" => "#2EE6A8",
        "edit" => "#FFC53D",
        "fetch" => "#4FD1FF",
        "delete" => "#FF6B6B",
        "move" => "#F2D16B",
        _ => "#FF4FA3",
    }
}

// ---------- syntax: the web highlighter's token classes ----------
pub const KW: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "if", "else", "for", "in", "return",
    "match", "self", "Self", "super", "crate", "true", "false", "const", "static", "enum", "while",
    "loop", "as", "where", "async", "await", "move", "trait", "type", "unsafe", "ref", "dyn",
    "function", "var", "import", "export", "from", "new", "class", "def", "None", "null",
];
/// A highlighter token class (harness.js `highlight`: c, a, s, n, m, k, ty, plain).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    Plain,
    Comment,
    Attr,
    Str,
    Num,
    Macro,
    Keyword,
    Type,
}
/// Splits one source line into runs of one token class; adjacent runs of a class merge.
pub fn tokens(line: &str) -> Vec<(String, Tok)> {
    let mut out: Vec<(String, Tok)> = vec![];
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let rest = &line[i..];
        let (len, tok) = if rest.starts_with("//") {
            (rest.len(), Tok::Comment)
        } else if rest.starts_with("#[") || rest.starts_with("#![") {
            (rest.find(']').map_or(rest.len(), |e| e + 1), Tok::Attr)
        } else if b[i] == b'"' {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            ((j + 1).min(b.len()) - i, Tok::Str)
        } else if b[i] == b'\''
            && b.len() > i + 2
            && (b[i + 2] == b'\'' || (b[i + 1] == b'\\' && b.len() > i + 3))
        {
            (if b[i + 1] == b'\\' { 4 } else { 3 }, Tok::Str)
        } else if b[i].is_ascii_digit() {
            let n = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
                .unwrap_or(rest.len());
            (n, Tok::Num)
        } else if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let n = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let word = &rest[..n];
            if rest[n..].starts_with('!') {
                (n + 1, Tok::Macro)
            } else if KW.contains(&word) {
                (n, Tok::Keyword)
            } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                (n, Tok::Type)
            } else {
                (n, Tok::Plain)
            }
        } else {
            (rest.chars().next().map_or(1, char::len_utf8), Tok::Plain)
        };
        let mut len = len.max(1).min(rest.len());
        while !rest.is_char_boundary(len) {
            len += 1;
        }
        match out.last_mut() {
            Some(last) if last.1 == tok => last.0.push_str(&rest[..len]),
            _ => out.push((rest[..len].to_owned(), tok)),
        }
        i += len;
    }
    out
}

// ---------- terminal output: the verdict a person scans for ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Bad,
    Warn,
    Dim,
    Plain,
}
/// Test/compiler output line class (harness.js `termOut`).
pub fn verdict(l: &str) -> Verdict {
    let t = l.trim_start();
    if l.ends_with("... ok") || t.starts_with("test result: ok") || t.starts_with("Finished") {
        Verdict::Ok
    } else if l.contains("FAILED")
        || l.contains("panicked")
        || t.starts_with("error")
        || t.starts_with("failures:")
    {
        Verdict::Bad
    } else if t.starts_with("warning") || t.starts_with("note:") {
        Verdict::Warn
    } else if t.starts_with("running ") || t.starts_with("Compiling") {
        Verdict::Dim
    } else {
        Verdict::Plain
    }
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

pub fn duration(ms: u64) -> String {
    match ms {
        0..1000 => format!("{}ms", ms.max(1)),
        1000..10000 => format!("{:.1}s", ms as f64 / 1000.),
        _ => format!("{}s", ms / 1000),
    }
}
pub fn kilo(n: u64) -> String {
    match n {
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1e6),
        n if n >= 10_000 => format!("{}k", n / 1000),
        n if n >= 1000 => format!("{:.1}k", n as f64 / 1e3),
        n => n.to_string(),
    }
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
    /// an agent tool named by its command line whose output only the agent's terminal saw
    pub quiet: bool,
    /// where a Search or Find looked (harness.js `where`): the head reads `in <path>`
    pub place: Option<String>,
}
/// One grep hit (harness.js `hits`: `/^(.*?):(\d+):(.*)$/`): the file, its line and the
/// matched text; a line without `:N:` is a file alone.
pub fn hit_line(l: &str) -> (&str, Option<&str>, &str) {
    let b = l.as_bytes();
    let mut i = 0;
    while let Some(off) = l[i..].find(':') {
        let colon = i + off;
        let digits = b[colon + 1..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .count();
        if digits > 0 && b.get(colon + 1 + digits) == Some(&b':') {
            let end = colon + 1 + digits;
            return (&l[..colon], Some(&l[colon + 1..end]), &l[end + 1..]);
        }
        i = colon + 1;
    }
    (l, None, "")
}
/// The hit text split at the pattern's matches (`<mark>` on the web): (span, matched).
pub fn hit_spans(text: &str, re: Option<&regex::Regex>) -> Vec<(String, bool)> {
    let mut spans = vec![];
    let mut at = 0;
    if let Some(re) = re {
        for m in re.find_iter(text) {
            if m.is_empty() {
                continue;
            }
            if m.start() > at {
                spans.push((text[at..m.start()].to_owned(), false));
            }
            spans.push((m.as_str().to_owned(), true));
            at = m.end();
        }
    }
    if at < text.len() {
        spans.push((text[at..].to_owned(), false));
    }
    spans
}
/// ACP agents fence their output (```console … ```) or leave a placeholder for output only
/// their own terminal saw (harness-spec.md, "ACP-agent sessions: fold rules native must match").
fn unfence(out: &str) -> String {
    let inner = out
        .strip_prefix("```")
        .and_then(|rest| rest.split_once('\n'))
        .filter(|(info, _)| {
            info.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        })
        .and_then(|(_, body)| body.strip_suffix("```"))
        .map(|body| body.strip_suffix('\n').unwrap_or(body));
    let out = inner.unwrap_or(out);
    let placeholder = (out.starts_with("[terminal ") && out.ends_with(']') && !out.contains('\n'))
        || out == "(no output)";
    if placeholder {
        String::new()
    } else {
        out.to_owned()
    }
}
/// "ls -la": a command line, not a phrase ("List files")
fn command_line(name: &str) -> bool {
    let word = |w: &str, first: bool| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
            && (first || w.starts_with('-') || !w.chars().any(char::is_uppercase))
    };
    let mut parts = name.split_whitespace();
    parts.next().is_some_and(|w| word(w, true))
        && name.split_whitespace().count() > 1
        && parts.all(|w| word(w, false))
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
            OutputBlock::Audio { .. }
            | OutputBlock::Resource { .. }
            | OutputBlock::Terminal { .. }
            | OutputBlock::Citations { .. } => Some(b.display_text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let out = unfence(&out);
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
        "Terminal" => ("Run".into(), s("command"), true, "execute"),
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
        n if kind == "execute" && s("command").is_empty() && command_line(n) => {
            ("Run".into(), n.to_owned(), true, kind)
        }
        n => (
            if n.is_empty() {
                "Tool".into()
            } else {
                n.to_owned()
            },
            if s("command").is_empty() {
                short(s("path"))
            } else {
                s("command")
            },
            !s("command").is_empty(),
            kind,
        ),
    };
    let quiet = kind == "execute" && verb == "Run" && name == target && out.is_empty();
    let place =
        (matches!(name, "grep" | "glob") && !s("path").is_empty()).then(|| short(s("path")));
    Tool {
        verb,
        target,
        mono,
        kind,
        args,
        out,
        name,
        quiet,
        place,
    }
}
