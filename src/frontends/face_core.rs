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
/// Python's durable handle spans several MCP calls. Keep its presentation as one
/// command with indexed output pages, so repeated polls neither duplicate nor
/// erase output. Project-owned; retain the policy with extracted projections:
/// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PythonCell {
    pub source: Option<String>,
    pub execution_id: Option<String>,
    pub status: Option<String>,
    pub exit_code: Option<i64>,
    pub emitted: Option<serde_json::Value>,
    pub output: std::collections::BTreeMap<usize, String>,
    /// Output indices whose chunk came from stderr (`"stream":"stderr"`).
    pub stderr: std::collections::BTreeSet<usize>,
    pub error: Option<String>,
    pub continued: bool,
}
impl PythonCell {
    pub fn merge(&mut self, next: Self) {
        if self.execution_id != next.execution_id {
            return;
        }
        // Concurrent polls are ordered by call start in the transcript. A later
        // row may therefore contain an older running snapshot than this one.
        let terminal = |status: Option<&str>| {
            status.is_some_and(|s| !matches!(s, "starting" | "running" | "cancelling"))
        };
        let stale = terminal(self.status.as_deref()) && !terminal(next.status.as_deref());
        for (index, text) in next.output {
            if stale && self.output.contains_key(&index) {
                continue;
            }
            self.output.insert(index, text);
            if next.stderr.contains(&index) {
                self.stderr.insert(index);
            } else {
                self.stderr.remove(&index);
            }
        }
        if next.status.is_some() && !stale {
            self.status = next.status;
            self.exit_code = next.exit_code;
            self.error = next.error;
        }
        if let Some(emitted) = next.emitted {
            let count = |value: &serde_json::Value| value.as_array().map_or(0, Vec::len);
            let previous = self.emitted.as_ref().map_or(0, count);
            if self.emitted.is_none()
                || count(&emitted) > previous
                || (count(&emitted) == previous && !stale)
            {
                self.emitted = Some(emitted);
            }
        }
    }
    pub fn preview(&self) -> String {
        self.emitted
            .as_ref()
            .filter(|v| v.as_array().is_none_or(|a| !a.is_empty()))
            .map(|value| format!("emit {}", raw_tool_output(value)))
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(120)
            .collect()
    }
    /// A long execution id as `head…tail`; the full id stays in the open card.
    pub fn short_id(id: &str) -> String {
        let chars: Vec<char> = id.chars().collect();
        if chars.len() <= 14 {
            return id.to_owned();
        }
        let head: String = chars[..8].iter().collect();
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{head}\u{2026}{tail}")
    }
    /// The failure as `(code, message)`: the server's `{code, message}` object
    /// when it is one, else the retained text itself.
    pub fn error_parts(&self) -> Option<(Option<String>, String)> {
        let raw = self.error.as_ref()?;
        let value: serde_json::Value = serde_json::from_str(raw).unwrap_or_default();
        let field = |k: &str| value.get(k).and_then(serde_json::Value::as_str);
        Some(match (field("code"), field("message")) {
            (code, Some(message)) => (code.map(str::to_owned), message.to_owned()),
            (Some(code), None) => (Some(code.to_owned()), String::new()),
            _ => (None, raw.clone()),
        })
    }
    pub fn text(&self) -> String {
        let mut parts = Vec::new();
        if let Some(emitted) = &self.emitted
            && emitted.as_array().is_none_or(|a| !a.is_empty())
        {
            parts.push(format!("Emitted\n{}", raw_tool_output(emitted)));
        }
        let output = self.output.values().cloned().collect::<String>();
        if !output.is_empty() {
            parts.push(output);
        }
        if let Some(error) = &self.error {
            parts.push(error.clone());
        }
        parts.join("\n\n")
    }
}
pub fn python_cell(
    name: &str,
    input_raw: &str,
    output: &[OutputBlock],
    raw: Option<&serde_json::Value>,
) -> Option<PythonCell> {
    use serde_json::Value;
    let tool = mcp_tool(name).map_or(name, |(_, tool)| tool);
    if !matches!(tool, "python_execute" | "python_execution_get") {
        return None;
    }
    fn result(value: &Value) -> Option<Value> {
        if let Some(value) = value.get("structuredContent")
            && let Some(result) = result(value)
        {
            return Some(result);
        }
        if value.get("execution_id").is_some()
            || value.get("status").is_some()
            || value.get("error").is_some()
        {
            return Some(value.clone());
        }
        if let Some(text) = value.as_str()
            && let Ok(value) = serde_json::from_str::<Value>(text)
        {
            return result(&value);
        }
        value
            .get("content")
            .and_then(Value::as_array)
            .and_then(|content| {
                content
                    .iter()
                    .find_map(|block| block.get("text").and_then(result))
            })
    }
    let args: Value = serde_json::from_str(input_raw).unwrap_or(Value::Null);
    let cursor = args["cursor"].as_str().unwrap_or("0");
    // These pages contain exact nested-result bytes, not execution output.
    // Preserve their ordinary MCP card, including its base64 recovery payload.
    if tool == "python_execution_get" && cursor.starts_with("child:") {
        return None;
    }
    let value = raw
        .and_then(result)
        .or_else(|| {
            output.iter().find_map(|block| match block {
                OutputBlock::Json { value } => result(value),
                OutputBlock::Text { text } | OutputBlock::Error { text } => {
                    result(&Value::String(unfence(text)))
                }
                _ => None,
            })
        })
        .unwrap_or(Value::Null);
    let offset = cursor.parse::<usize>().unwrap_or(0);
    let chunks = value["output"].as_array().cloned().unwrap_or_default();
    Some(PythonCell {
        stderr: chunks
            .iter()
            .enumerate()
            .filter(|(_, chunk)| chunk["stream"] == "stderr")
            .map(|(i, _)| offset + i)
            .collect(),
        source: args["source"].as_str().map(str::to_owned),
        execution_id: value["execution_id"]
            .as_str()
            .or_else(|| args["execution_id"].as_str())
            .map(str::to_owned),
        status: value["status"].as_str().map(str::to_owned),
        exit_code: value["exit_code"].as_i64(),
        emitted: value.get("emitted").cloned(),
        output: value["output"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(i, chunk)| {
                (
                    offset + i,
                    chunk["text"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| {
                            "[binary output; recover with the execution cursor]".into()
                        }),
                )
            })
            .collect(),
        error: value
            .get("error")
            .filter(|v| !v.is_null())
            .map(raw_tool_output),
        continued: tool == "python_execution_get",
    })
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
/// `(server, tool)` of an MCP call in either harness spelling.
fn mcp_tool(name: &str) -> Option<(&str, &str)> {
    name.strip_prefix("mcp__")
        .and_then(|r| r.split_once("__"))
        .or_else(|| name.strip_prefix("mcp.").and_then(|r| r.split_once('.')))
        .filter(|(s, t)| !s.is_empty() && !t.is_empty())
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
            OutputBlock::Json { value } => Some(raw_tool_output(value)),
            OutputBlock::Audio { .. }
            | OutputBlock::Resource { .. }
            | OutputBlock::Terminal { .. }
            | OutputBlock::Citations { .. } => Some(b.display_text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut out = unfence(&out);
    let python = python_cell(name, input_raw, output, None);
    if let Some(cell) = &python {
        out = cell.text();
    }
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
        // Claude's tool loader: name what it loaded, not the transport query
        "ToolSearch" => (
            "Load tools".into(),
            s("query")
                .trim_start_matches("select:")
                .split(',')
                .map(|t| mcp_tool(t.trim()).map_or(t.trim(), |(_, tool)| tool))
                .collect::<Vec<_>>()
                .join(", "),
            false,
            "other",
        ),
        _ if let Some(cell) = &python => (
            if cell.continued {
                "Continue Python"
            } else {
                "Run Python"
            }
            .into(),
            cell.source
                .clone()
                .or_else(|| cell.execution_id.clone())
                .unwrap_or_default(),
            true,
            "execute",
        ),
        // one heading for every harness: Claude `mcp__s__t`, Codex `mcp.s.t` both read
        // `s.t`, then the call's first short string argument (a path, id or query)
        n if let Some((server, tool)) = mcp_tool(n) => {
            // Only identifying arguments, never arbitrary payload values.
            let hint = [
                "path", "locator", "query", "task", "seat_id", "message_id", "swarm_id", "id",
                "name", "url",
            ]
            .iter()
            .find_map(|k| args.get(*k).and_then(serde_json::Value::as_str))
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_default()
            .to_owned();
            (format!("{server}.{tool}"), short(hint), true, "other")
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

/// Old ACP producers inserted an empty placeholder beside genuine raw results.
/// Keep replay compatibility while never treating those sentinels as content.
pub fn placeholder_output(output: &[OutputBlock]) -> bool {
    output.is_empty() || output.iter().all(|block| matches!(block,
        OutputBlock::Text { text } if matches!(text.trim(), "" | "(no output)" | "(the agent did not report a result)")))
}
/// Render MCP's text/structured results without exposing binary payloads or
/// transport configuration. The upstream redactor still owns secret values.
pub fn raw_tool_output(value: &serde_json::Value) -> String {
    use serde_json::Value;
    fn safe(value: &Value) -> Value {
        match value {
            Value::Object(object) => Value::Object(
                object
                    .iter()
                    .map(|(key, value)| {
                        let compact = key.to_ascii_lowercase().replace(['_', '-'], "");
                        let value = if matches!(
                            compact.as_str(),
                            "blob"
                                | "mcpservers"
                                | "env"
                                | "headers"
                                | "authorization"
                                | "capability"
                                | "apikey"
                                | "token"
                                | "password"
                        ) || (compact == "data"
                            && (object.contains_key("mimeType")
                                || object
                                    .get("type")
                                    .and_then(Value::as_str)
                                    .is_some_and(|t| matches!(t, "image" | "audio" | "blob"))))
                        {
                            Value::String("[payload omitted]".into())
                        } else {
                            safe(value)
                        };
                        (key.clone(), value)
                    })
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.iter().map(safe).collect()),
            _ => value.clone(),
        }
    }
    if value.is_null() {
        return String::new();
    }
    if let Some(text) = value.as_str() {
        return text.to_owned();
    }
    let mut parts = vec![];
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        for block in content {
            match block.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        parts.push(text.to_owned());
                    }
                }
                Some("image" | "audio") => parts.push(format!(
                    "{} · {} · binary preview unavailable",
                    block["type"].as_str().unwrap_or("media"),
                    block
                        .get("mimeType")
                        .and_then(Value::as_str)
                        .unwrap_or("unreported MIME")
                )),
                _ => parts.push(serde_json::to_string_pretty(&safe(block)).unwrap_or_default()),
            }
        }
        if let Some(structured) = value.get("structuredContent") {
            parts.push(serde_json::to_string_pretty(&safe(structured)).unwrap_or_default());
        }
        if !parts.is_empty() {
            return parts.join("\n\n");
        }
    }
    serde_json::to_string_pretty(&safe(value)).unwrap_or_default()
}

#[cfg(test)]
mod gui_polish_output_tests {
    use super::*;
    #[test]
    fn python_execution_cell_model_keeps_source_status_preview_and_pages() {
        use serde_json::json;
        for name in [
            "python_execute",
            "mcp__bboxy__python_execute",
            "mcp.bboxy.python_execute",
        ] {
            let mut cell = python_cell(name, r#"{"source":"print('head')\nemit(1)"}"#, &[], Some(&json!({
                "structuredContent":{"execution_id":"run-1","status":"running","output":[{"text":"head\n"}],"emitted":[1]}
            }))).unwrap();
            assert_eq!(cell.source.as_deref(), Some("print('head')\nemit(1)"));
            assert_eq!(cell.status.as_deref(), Some("running"));
            assert_eq!(cell.preview(), "emit [ 1 ]");
            let page = python_cell("python_execution_get", r#"{"execution_id":"run-1","cursor":"1"}"#, &[
                OutputBlock::Text { text: json!({"execution_id":"run-1","status":"completed","exit_code":0,"output":[{"text":"tail\n"}],"emitted":[1,2]}).to_string() }
            ], None).unwrap();
            assert!(page.continued);
            assert_eq!(page.execution_id.as_deref(), Some("run-1"));
            cell.merge(page.clone());
            cell.merge(page);
            assert_eq!(cell.status.as_deref(), Some("completed"));
            assert_eq!(cell.exit_code, Some(0));
            assert_eq!(
                cell.output.values().cloned().collect::<String>(),
                "head\ntail\n"
            );
            assert!(cell.text().contains("2"));
            let stale = python_cell("python_execution_get", r#"{"execution_id":"run-1","cursor":"0"}"#, &[], Some(&json!({
                "execution_id":"run-1","status":"running","output":[{"text":"stale-head"}],"emitted":[1]
            }))).unwrap();
            cell.merge(stale.clone());
            assert_eq!(cell.status.as_deref(), Some("completed"));
            assert_eq!(cell.exit_code, Some(0));
            assert_eq!(cell.emitted, Some(json!([1, 2])));
            assert_eq!(
                cell.output.values().cloned().collect::<String>(),
                "head\ntail\n"
            );
            cell.status = Some("failed".into());
            cell.exit_code = Some(1);
            cell.error = Some("retained terminal error".into());
            cell.merge(stale);
            assert_eq!(cell.status.as_deref(), Some("failed"));
            assert_eq!(cell.exit_code, Some(1));
            assert_eq!(cell.error.as_deref(), Some("retained terminal error"));
            assert!(
                python_cell(
                    "python_execution_get",
                    r#"{"execution_id":"run-1","cursor":"child:0:0"}"#,
                    &[],
                    Some(&json!({"execution_id":"run-1","status":"unknown"})),
                )
                .is_none(),
                "child bytes stay in an ordinary MCP result card"
            );
            let child = describe(
                "mcp__bboxy__python_execution_get",
                ToolKind::Other,
                r#"{"execution_id":"run-1","cursor":"child:0:0"}"#,
                &[OutputBlock::Json {
                    value: json!({"data_base64":"Ynl0ZXM=","encoding":"base64-json"}),
                }],
                "/",
            );
            assert!(child.out.contains("Ynl0ZXM="));
            let refused = python_cell(
                "python_execution_get",
                r#"{"execution_id":"run-1"}"#,
                &[],
                Some(&json!({"execution_id":"run-1","error":{"code":"execution_context_changed"}})),
            )
            .unwrap();
            assert!(refused.status.is_none());
            assert!(refused.text().contains("execution_context_changed"));
            assert_eq!(
                refused.error_parts(),
                Some((Some("execution_context_changed".into()), String::new()))
            );
            let mixed = python_cell(name, r#"{"source":"x"}"#, &[], Some(&json!({
                "execution_id":"01M6XXWN1CY9A0746DJHQMGDPH","status":"failed","exit_code":1,
                "output":[{"stream":"stdout","text":"ok\n"},{"stream":"stderr","text":"boom\n"}],
                "error":{"code":"python_failed","message":"Python exited unsuccessfully"}
            }))).unwrap();
            assert_eq!(mixed.stderr.iter().copied().collect::<Vec<_>>(), vec![1]);
            assert_eq!(
                mixed.error_parts(),
                Some((Some("python_failed".into()), "Python exited unsuccessfully".into()))
            );
            assert_eq!(PythonCell::short_id("01M6XXWN1CY9A0746DJHQMGDPH"), "01M6XXWN\u{2026}GDPH");
            assert_eq!(PythonCell::short_id("run-1"), "run-1");
            let mut later = mixed.clone();
            later.merge(python_cell(name, r#"{"source":"x"}"#, &[], Some(&json!({
                "execution_id":"01M6XXWN1CY9A0746DJHQMGDPH","status":"failed","exit_code":1,
                "output":[{"stream":"stdout","text":"ok\n"},{"stream":"stdout","text":"replaced\n"}]
            }))).unwrap());
            assert!(later.stderr.is_empty(), "a replaced chunk takes its new stream");
        }
        assert!(python_cell("mcp__bboxy__unrelated", "{}", &[], None).is_none());
    }
    #[test]
    fn mcp_headings_match_across_harnesses() {
        for name in ["mcp__bboxy__message_ack", "mcp.bboxy.message_ack"] {
            let t = describe(name, ToolKind::Other, "{}", &[], "/");
            assert_eq!(
                (t.verb.as_str(), t.target.as_str()),
                ("bboxy.message_ack", "")
            );
        }
        let t = describe(
            "mcp__bboxy__knowledge_fetch",
            ToolKind::Other,
            r#"{"op_id":"01M6","locator":"tasks/sdk.md"}"#,
            &[],
            "/",
        );
        assert_eq!(
            (t.verb.as_str(), t.target.as_str()),
            ("bboxy.knowledge_fetch", "tasks/sdk.md")
        );
        let t = describe(
            "ToolSearch",
            ToolKind::Other,
            r#"{"query":"select:mcp__bboxy__orient,Read"}"#,
            &[],
            "/",
        );
        assert_eq!(
            (t.verb.as_str(), t.target.as_str()),
            ("Load tools", "orient, Read")
        );
    }
    #[test]
    fn gui_polish_result_formatter_keeps_structured_data_and_resource_links() {
        let result = raw_tool_output(&serde_json::json!({"content": [
            {"type":"text","text":"# Available result"},
            {"type":"image","mimeType":"image/png","data":"private-image-bytes"},
            {"type":"resource_link","uri":"https://example.test/artifact","name":"artifact"}
        ], "structuredContent": {"data":{"answer":42},"env":{"KEY":"private-env"},"mcpServers":{"private":"relay"}}}));
        for needle in [
            "Available result",
            "42",
            "https://example.test/artifact",
            "image/png",
        ] {
            assert!(result.contains(needle));
        }
        for secret in ["private-image-bytes", "private-env", "relay"] {
            assert!(!result.contains(secret));
        }
        assert!(placeholder_output(&[OutputBlock::Text {
            text: "(no output)".into()
        }]));
        assert!(!placeholder_output(&[OutputBlock::Error {
            text: "real error".into()
        }]));
    }
}
