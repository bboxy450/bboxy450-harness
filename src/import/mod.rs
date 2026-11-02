// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Native harness logs into our log format. Our log is the center: Claude Code and Codex
//! session files project, line by line, into the same `Event`s the engine and the ACP
//! backend write, so every face renders them and switching/handoff work on them.
//!
//! A projector is pure and streaming: `push` one native JSONL record, get the events it
//! adds; `finish` closes whatever is still open. Records it does not understand (metadata,
//! API bookkeeping, subagent side chains) add nothing. The result passes check_transcript.
pub mod watch;

use crate::protocol::*;
use serde_json::Value;
use std::collections::BTreeMap;

/// Which native format a file holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Native {
    ClaudeCode,
    Codex,
}
impl Native {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "claude" | "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }
    /// Guess from the first records of a file.
    pub fn sniff(first: &Value) -> Option<Self> {
        if first.get("payload").is_some() && first.get("type").is_some() {
            Some(Self::Codex)
        } else if first.get("sessionId").is_some()
            || first.get("parentUuid").is_some()
            || first.get("leafUuid").is_some()
        {
            Some(Self::ClaudeCode)
        } else {
            None
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
        }
    }
}

struct OpenTool {
    item: ItemId,
    call_id: String,
    name: String,
    kind: ToolKind,
    input: String,
}

/// The streaming projection of one native session.
pub struct Projector {
    native: Native,
    turn: Option<TurnId>,
    tools: BTreeMap<String, OpenTool>,
    /// The native session's working directory, once a record names it.
    pub cwd: Option<String>,
    pub model: Option<String>,
    /// The native clock of the last record that carried one (ms since the epoch).
    pub ts_ms: Option<u64>,
}
impl Projector {
    pub fn new(native: Native) -> Self {
        Self {
            native,
            turn: None,
            tools: BTreeMap::new(),
            cwd: None,
            model: None,
            ts_ms: None,
        }
    }
    pub fn push(&mut self, record: &Value) -> Vec<Event> {
        if let Some(ts) = record
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(rfc3339_ms)
        {
            self.ts_ms = Some(ts);
        }
        let mut out = vec![];
        match self.native {
            Native::ClaudeCode => self.claude(record, &mut out),
            Native::Codex => self.codex(record, &mut out),
        }
        out
    }
    /// Close the open turn (a file that ends mid-turn ends it Interrupted).
    pub fn finish(&mut self, reason: StopReason) -> Vec<Event> {
        let mut out = vec![];
        self.end_turn(reason, &mut out);
        out
    }

    // ---------- shared ----------
    fn begin_turn(&mut self, out: &mut Vec<Event>) -> TurnId {
        if let Some(t) = self.turn {
            return t;
        }
        let turn = TurnId::new();
        out.push(Event::TurnQueued { turn });
        out.push(Event::TurnStarted { turn });
        self.turn = Some(turn);
        turn
    }
    fn end_turn(&mut self, reason: StopReason, out: &mut Vec<Event>) {
        let Some(turn) = self.turn.take() else { return };
        for (_, t) in std::mem::take(&mut self.tools) {
            out.push(Event::ItemCompleted {
                item: t.item,
                status: ItemStatus::Interrupted,
                body: tool_body(
                    t,
                    vec![OutputBlock::Text {
                        text: "(no result in the log)".into(),
                    }],
                ),
            });
        }
        out.push(Event::TurnEnded {
            turn,
            reason,
            usage: None,
        });
    }
    fn record(&mut self, body: Item, out: &mut Vec<Event>) {
        let turn = self.begin_turn(out);
        let item = ItemId::new();
        out.push(Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        });
        out.push(Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body,
        });
    }
    fn tool_start(&mut self, call_id: &str, name: &str, input: String, out: &mut Vec<Event>) {
        let turn = self.begin_turn(out);
        if self.tools.contains_key(call_id) {
            return;
        }
        let t = OpenTool {
            item: ItemId::new(),
            call_id: if call_id.is_empty() {
                ItemId::new().to_string()
            } else {
                call_id.to_owned()
            },
            name: name.to_owned(),
            kind: tool_kind_for(name),
            input,
        };
        out.push(Event::ItemStarted {
            turn: Some(turn),
            item: t.item,
            kind: ItemKind::ToolCall,
            parent: None,
            head: Some(ItemHead::ToolCall {
                call_id: t.call_id.clone(),
                name: t.name.clone(),
                tool_kind: t.kind,
            }),
        });
        self.tools.insert(call_id.to_owned(), t);
    }
    fn tool_end(&mut self, call_id: &str, text: String, failed: bool, out: &mut Vec<Event>) {
        let Some(t) = self.tools.remove(call_id) else {
            return;
        };
        let output = vec![if failed {
            OutputBlock::Error { text }
        } else {
            OutputBlock::Text {
                text: if text.is_empty() {
                    "(no output)".into()
                } else {
                    text
                },
            }
        }];
        let status = if failed {
            ItemStatus::Failed {
                reason: "tool failed".into(),
            }
        } else {
            ItemStatus::Completed
        };
        out.push(Event::ItemCompleted {
            item: t.item,
            status,
            body: tool_body(t, output),
        });
    }

    // ---------- Claude Code: ~/.claude/projects/<project>/<session>.jsonl ----------
    fn claude(&mut self, r: &Value, out: &mut Vec<Event>) {
        if r.get("isSidechain").and_then(Value::as_bool) == Some(true) {
            return; // a subagent's own conversation
        }
        if let Some(cwd) = r.get("cwd").and_then(Value::as_str) {
            self.cwd.get_or_insert_with(|| cwd.to_owned());
        }
        let kind = r.get("type").and_then(Value::as_str).unwrap_or("");
        let Some(message) = r.get("message") else {
            return;
        };
        if let Some(model) = message.get("model").and_then(Value::as_str) {
            self.model = Some(model.to_owned());
        }
        match kind {
            "user" => {
                if r.get("isMeta").and_then(Value::as_bool) == Some(true) {
                    return;
                }
                match message.get("content") {
                    Some(Value::String(text)) => self.user_prompt(text, out),
                    Some(Value::Array(blocks)) => {
                        let mut prompt = vec![];
                        for b in blocks {
                            match b.get("type").and_then(Value::as_str) {
                                Some("tool_result") => {
                                    let id =
                                        b.get("tool_use_id").and_then(Value::as_str).unwrap_or("");
                                    let failed =
                                        b.get("is_error").and_then(Value::as_bool) == Some(true);
                                    self.tool_end(id, content_text(b.get("content")), failed, out);
                                }
                                Some("text") => prompt.push(
                                    b.get("text")
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_owned(),
                                ),
                                _ => {}
                            }
                        }
                        if !prompt.is_empty() {
                            self.user_prompt(&prompt.join("\n"), out);
                        }
                    }
                    _ => {}
                }
            }
            "assistant" => {
                for b in message
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    match b.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            let text = b.get("text").and_then(Value::as_str).unwrap_or("");
                            if !text.trim().is_empty() {
                                self.record(
                                    Item::AgentMessage {
                                        markdown: text.to_owned(),
                                    },
                                    out,
                                );
                            }
                        }
                        Some("thinking") => {
                            let text = b.get("thinking").and_then(Value::as_str).unwrap_or("");
                            if !text.trim().is_empty() {
                                self.record(
                                    Item::Reasoning {
                                        summary: text.to_owned(),
                                        raw: None,
                                        provider_opaque: None,
                                    },
                                    out,
                                );
                            }
                        }
                        Some("tool_use") => {
                            let id = b.get("id").and_then(Value::as_str).unwrap_or("");
                            let name = b.get("name").and_then(Value::as_str).unwrap_or("tool");
                            let input = b.get("input").map(Value::to_string).unwrap_or_default();
                            self.tool_start(id, name, input, out);
                        }
                        _ => {}
                    }
                }
                if message.get("stop_reason").and_then(Value::as_str) == Some("end_turn")
                    && self.tools.is_empty()
                {
                    self.end_turn(StopReason::EndTurn, out);
                }
            }
            _ => {}
        }
    }
    fn user_prompt(&mut self, text: &str, out: &mut Vec<Event>) {
        // harness bookkeeping that Claude Code stores as user text is not a prompt
        let t = text.trim_start();
        if t.is_empty()
            || t.starts_with("<command-")
            || t.starts_with("<local-command")
            || t.starts_with("<system-reminder>")
        {
            return;
        }
        self.end_turn(StopReason::EndTurn, out);
        self.record(
            Item::UserMessage {
                blocks: vec![InputBlock::Text {
                    text: text.to_owned(),
                }],
            },
            out,
        );
    }

    // ---------- Codex: ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl ----------
    fn codex(&mut self, r: &Value, out: &mut Vec<Event>) {
        let p = r.get("payload").cloned().unwrap_or(Value::Null);
        let ptype = p.get("type").and_then(Value::as_str).unwrap_or("");
        match r.get("type").and_then(Value::as_str).unwrap_or("") {
            "session_meta" => {
                if let Some(cwd) = p.get("cwd").and_then(Value::as_str) {
                    self.cwd = Some(cwd.to_owned());
                }
            }
            "turn_context" => {
                if let Some(model) = p.get("model").and_then(Value::as_str) {
                    self.model = Some(model.to_owned());
                }
            }
            "event_msg" => match ptype {
                "task_started" => {
                    self.end_turn(StopReason::EndTurn, out);
                    self.begin_turn(out);
                }
                "task_complete" => self.end_turn(StopReason::EndTurn, out),
                "turn_aborted" => self.end_turn(StopReason::Interrupted, out),
                _ => {}
            },
            "response_item" => match ptype {
                "message" => {
                    let role = p.get("role").and_then(Value::as_str).unwrap_or("");
                    let text = p
                        .get("content")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|c| c.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let t = text.trim_start();
                    match role {
                        "user" if !t.is_empty() && !t.starts_with('<') => self.record(
                            Item::UserMessage {
                                blocks: vec![InputBlock::Text { text }],
                            },
                            out,
                        ),
                        "assistant" if !t.is_empty() => {
                            self.record(Item::AgentMessage { markdown: text }, out)
                        }
                        _ => {}
                    }
                }
                "reasoning" => {
                    let text = p
                        .get("summary")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|s| s.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !text.trim().is_empty() {
                        self.record(
                            Item::Reasoning {
                                summary: text,
                                raw: None,
                                provider_opaque: None,
                            },
                            out,
                        );
                    }
                }
                "function_call" | "custom_tool_call" | "local_shell_call" => {
                    let id = p.get("call_id").and_then(Value::as_str).unwrap_or("");
                    let name = p.get("name").and_then(Value::as_str).unwrap_or("shell");
                    let input = p
                        .get("arguments")
                        .or_else(|| p.get("input"))
                        .or_else(|| p.get("action"))
                        .map(|v| {
                            v.as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| v.to_string())
                        })
                        .unwrap_or_default();
                    self.tool_start(id, name, input, out);
                }
                "function_call_output" | "custom_tool_call_output" | "local_shell_call_output" => {
                    let id = p.get("call_id").and_then(Value::as_str).unwrap_or("");
                    let output = p.get("output").map(|o| match o {
                        Value::String(s) => s.clone(),
                        other => other
                            .get("content")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                            .unwrap_or_else(|| other.to_string()),
                    });
                    self.tool_end(id, output.unwrap_or_default(), false, out);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn tool_body(t: OpenTool, output: Vec<OutputBlock>) -> Item {
    Item::ToolCall {
        call_id: t.call_id,
        name: t.name,
        tool_kind: t.kind,
        input_raw: t.input,
        output,
        locations: vec![],
        spill: None,
    }
}
fn content_text(c: Option<&Value>) -> String {
    match c {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|p| p.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}
/// A harness tool name → our tool kind, for the card's icon and body.
pub fn tool_kind_for(name: &str) -> ToolKind {
    let n = name.to_ascii_lowercase();
    match n.as_str() {
        "read" | "notebookread" | "view" => ToolKind::Read,
        "edit" | "multiedit" | "write" | "notebookedit" | "apply_patch" => ToolKind::Edit,
        "bash" | "shell" | "exec_command" | "local_shell" | "terminal" => ToolKind::Execute,
        "grep" | "glob" | "ls" | "search" => ToolKind::Search,
        "webfetch" | "websearch" | "fetch" => ToolKind::Fetch,
        _ if n.starts_with("mcp__") => ToolKind::Other,
        _ => ToolKind::Other,
    }
}

/// Project a whole native file into a durable log under `sessions`; returns its session.
pub async fn import_file(
    native: Option<Native>,
    path: &std::path::Path,
    sessions: std::path::PathBuf,
) -> Result<SessionId, String> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let records: Vec<Value> = text
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let native = native
        .or_else(|| records.iter().take(8).find_map(Native::sniff))
        .ok_or("unrecognised native log: pass claude or codex")?;
    let mut p = Projector::new(native);
    let mut events = vec![];
    for r in &records {
        let ts = p.ts_ms;
        let new = p.push(r);
        let at = p.ts_ms.or(ts).unwrap_or_else(crate::store::now_ms);
        events.extend(new.into_iter().map(|e| (e, at)));
    }
    let at = p.ts_ms.unwrap_or_else(crate::store::now_ms);
    events.extend(p.finish(StopReason::EndTurn).into_iter().map(|e| (e, at)));
    let meta = SessionMeta {
        session: SessionId::new(),
        cwd: p.cwd.clone().unwrap_or_else(|| "/".into()).into(),
        profile: Some(native.label().into()),
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: crate::store::now_ms(),
    };
    let session = meta.session;
    let mut store =
        crate::store::Store::open(sessions, meta, Some(OpId::new()), Default::default())
            .await
            .map_err(|e| e.to_string())?;
    store
        .append(
            Event::ProfileChanged {
                profile: native.label().into(),
                model: p.model.clone().unwrap_or_else(|| native.label().into()),
            },
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    for (e, at) in events {
        store
            .append_at(e, None, at)
            .await
            .map_err(|e| e.to_string())?;
    }
    store
        .append(Event::SessionClosed, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(session)
}

/// `2026-11-02T20:59:17.085Z` (or an offset) → milliseconds since the Unix epoch.
pub fn rfc3339_ms(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || (b[10] != b'T' && b[10] != b' ') {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, sec) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let mut rest = &s[19..];
    let mut ms = 0i64;
    if let Some(frac) = rest.strip_prefix('.') {
        let digits: String = frac.chars().take_while(char::is_ascii_digit).collect();
        rest = &frac[digits.len()..];
        ms = format!("{digits:0<3}")[..3].parse().ok()?;
    }
    let offset = match rest {
        "" | "Z" | "z" => 0,
        o if o.len() == 6 => {
            let sign = if o.starts_with('-') { -1 } else { 1 };
            sign * (o[1..3].parse::<i64>().ok()? * 3600 + o[4..6].parse::<i64>().ok()? * 60)
        }
        _ => return None,
    };
    // days from civil (Howard Hinnant)
    let (yy, m) = if mo <= 2 {
        (y - 1, mo + 9)
    } else {
        (y, mo - 3)
    };
    let era = yy.div_euclid(400);
    let yoe = yy - era * 400;
    let doy = (153 * m + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let secs = days * 86_400 + h * 3600 + mi * 60 + sec - offset;
    u64::try_from(secs * 1000 + ms).ok()
}
