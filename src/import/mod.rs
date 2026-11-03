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
//! adds; `finish` closes whatever is still open. Unmapped records remain Native;
//! unconsumed mapped fields accompany their item in the provider extension.
pub mod edge;
pub mod reproject;
mod spill;
pub mod watch;
pub(crate) use spill::append_projected;

use crate::protocol::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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

pub struct Projected {
    pub event: Event,
    pub native: Option<crate::protocol::Native>,
    pub ext: BTreeMap<String, Value>,
}
pub fn record_type(record: &Value) -> String {
    let kind = record
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("record");
    match record.pointer("/payload/type").and_then(Value::as_str) {
        Some(sub) => format!("{kind}/{sub}"),
        None => kind.to_owned(),
    }
}
pub fn native_identity(record: &Value) -> Option<crate::protocol::Native> {
    let id = record
        .get("uuid")
        .or_else(|| record.get("id"))
        .or_else(|| record.pointer("/payload/id"))
        .or_else(|| record.pointer("/payload/call_id"))?
        .as_str()?;
    Some(crate::protocol::Native {
        id: id.into(),
        parent: record
            .get("parentUuid")
            .or_else(|| record.get("parent_id"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        ts_ms: record
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(rfc3339_ms),
    })
}

/// The streaming projection of one native session.
pub struct Projector {
    native: Native,
    turn: Option<TurnId>,
    tools: BTreeMap<String, OpenTool>,
    usage: Option<Usage>,
    usage_overflowed: bool,
    claude_messages: BTreeSet<String>,
    codex_total: Option<Usage>,
    native_version: Option<String>,
    context_window: Option<u64>,
    /// The native session's working directory, once a record names it.
    pub cwd: Option<String>,
    pub model: Option<String>,
    /// The native clock of the last record that carried one (ms since the epoch).
    pub ts_ms: Option<u64>,
    #[cfg(feature = "test-fixtures")]
    restore_drop: bool,
}
impl Projector {
    pub fn new(native: Native) -> Self {
        Self {
            native,
            turn: None,
            tools: BTreeMap::new(),
            usage: None,
            usage_overflowed: false,
            claude_messages: BTreeSet::new(),
            codex_total: None,
            native_version: None,
            context_window: None,
            cwd: None,
            model: None,
            ts_ms: None,
            #[cfg(feature = "test-fixtures")]
            restore_drop: false,
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
        if !out
            .iter()
            .any(|e| matches!(e, Event::ItemStarted { .. } | Event::ItemCompleted { .. }))
        {
            self.native_record(record, &mut out);
        }
        out
    }
    pub fn project(&mut self, record: &Value) -> Vec<Projected> {
        let events = self.push(record);
        let complete = events
            .iter()
            .position(|e| matches!(e, Event::ItemCompleted { .. }))
            .unwrap_or(0);
        let raw_native = events.iter().any(|e| {
            matches!(
                e,
                Event::ItemCompleted {
                    body: Item::Native { .. },
                    ..
                }
            )
        });
        let extra = edge::remainder(self.native, record);
        events
            .into_iter()
            .enumerate()
            .map(|(i, event)| {
                let identity = if matches!(
                    &event,
                    Event::ItemStarted { .. } | Event::ItemCompleted { .. }
                ) {
                    native_identity(record)
                } else {
                    None
                };
                Projected {
                    event,
                    native: identity,
                    ext: if i == complete
                        && !raw_native
                        && extra.as_object().is_some_and(|o| !o.is_empty())
                    {
                        BTreeMap::from([(self.native.label().to_owned(), extra.clone())])
                    } else {
                        BTreeMap::new()
                    },
                }
            })
            .collect()
    }
    fn native_record(&self, record: &Value, out: &mut Vec<Event>) {
        #[cfg(feature = "test-fixtures")]
        if self.restore_drop {
            return;
        }
        if out.iter().any(|e| matches!(e, Event::ItemCompleted { body: Item::Native { raw, .. }, .. } if raw == record)) { return; }
        let item = ItemId::new();
        out.push(Event::ItemStarted {
            turn: self.turn,
            item,
            kind: ItemKind::Native,
            parent: None,
            head: None,
        });
        out.push(Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: Item::Native {
                provider: self.native.label().into(),
                record_type: record_type(record),
                raw: record.clone(),
                blob: None,
            },
        });
    }
    #[cfg(feature = "test-fixtures")]
    #[doc(hidden)]
    pub fn restore_drop_control(&mut self) {
        self.restore_drop = true;
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
        out.push(Event::TurnStarted {
            turn,
            model: self.model.clone(),
        });
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
            usage: self.usage.take(),
        });
        self.usage_overflowed = false;
    }
    fn set_model(&mut self, model: &str, out: &mut Vec<Event>) {
        if self.model.as_deref() != Some(model) {
            self.model = Some(model.to_owned());
            out.push(Event::ProfileChanged {
                profile: self.native.label().into(),
                model: model.to_owned(),
            });
        }
    }
    fn record_usage(&mut self, usage: Usage, out: &mut Vec<Event>) {
        if !self.usage_overflowed {
            self.usage = match self.usage.take() {
                Some(total) => add_usage(&total, &usage),
                None => Some(usage.clone()),
            };
            self.usage_overflowed = self.usage.is_none();
            if let Some(total) = &mut self.usage {
                total.scope = Some(UsageScope::Turn);
                total.source = Some(NumberSource::Derived);
                total.request_id = None;
            }
        }
        self.record(Item::Usage { usage }, out);
    }
    fn usage_backend(&self) -> Backend {
        Backend {
            kind: BackendKind::Import,
            name: self.native.label().into(),
            version: self.native_version.clone(),
        }
    }
    fn claude_usage(&mut self, r: &Value, message: &Value, out: &mut Vec<Event>) {
        let Some(u) = message.get("usage") else {
            return;
        };
        let Some(mut usage) = parse_usage(u, "cache_read_input_tokens", None) else {
            return;
        };
        // Anthropic reports uncached input separately; our input includes all cache tiers,
        // just as the native Anthropic provider does. Optional absent counts stay unknown.
        if !usage.unreported.iter().any(|n| n == "input_tokens") {
            let Some(input) = usage
                .input_tokens
                .checked_add(usage.cached_input_tokens.unwrap_or(0))
                .and_then(|n| n.checked_add(usage.cache_creation_input_tokens.unwrap_or(0)))
            else {
                return;
            };
            usage.input_tokens = input;
        }
        usage.scope = Some(UsageScope::Call);
        usage.backend = Some(Box::new(self.usage_backend()));
        usage.model = message
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_owned);
        usage.request_id = r
            .get("requestId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let mut keys: Vec<String> = message
            .get("id")
            .and_then(Value::as_str)
            .map(|s| format!("message:{s}"))
            .into_iter()
            .chain(
                r.get("requestId")
                    .and_then(Value::as_str)
                    .map(|s| format!("request:{s}")),
            )
            .collect();
        if keys.is_empty() {
            keys.extend(
                r.get("uuid")
                    .and_then(Value::as_str)
                    .map(|s| format!("record:{s}")),
            );
        }
        let duplicate = keys.iter().any(|key| self.claude_messages.contains(key));
        self.claude_messages.extend(keys);
        if duplicate {
            return;
        }
        self.record_usage(usage, out);
    }
    fn codex_usage(&mut self, p: &Value, out: &mut Vec<Event>) {
        let Some(info) = p.get("info").filter(|i| i.is_object()) else {
            return;
        };
        let total = info
            .get("total_token_usage")
            .and_then(|u| parse_usage(u, "cached_input_tokens", Some("reasoning_output_tokens")));
        let repeated = total.is_some() && total == self.codex_total;
        let window = info.get("model_context_window").and_then(Value::as_u64);
        // Only last_token_usage identifies a reported model call. A cumulative
        // snapshot is session scope, never an invented call or turn total (§4a).
        if !repeated
            && let Some(mut usage) = info.get("last_token_usage").and_then(|u| {
                parse_usage(u, "cached_input_tokens", Some("reasoning_output_tokens"))
            })
        {
            usage.scope = Some(UsageScope::Call);
            usage.backend = Some(Box::new(self.usage_backend()));
            usage.model = self.model.clone();
            usage.request_id = p
                .get("request_id")
                .and_then(Value::as_str)
                .map(str::to_owned);
            self.record_usage(usage, out);
        }
        let session = total.clone().or_else(|| {
            window.map(|_| Usage {
                source: Some(NumberSource::Reported),
                unreported: vec!["input_tokens".into(), "output_tokens".into()],
                ..Usage::default()
            })
        });
        if ((total.is_some() && !repeated) || window != self.context_window)
            && let Some(mut usage) = session
        {
            usage.scope = Some(UsageScope::Session);
            usage.backend = Some(Box::new(self.usage_backend()));
            // Cumulative counters can span models; the current model is not a
            // billed identity for all preceding calls.
            usage.context_window = window;
            let item = ItemId::new();
            out.push(Event::ItemStarted {
                turn: self.turn,
                item,
                kind: ItemKind::Usage,
                parent: None,
                head: None,
            });
            out.push(Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::Usage { usage },
            });
        }
        if total.is_some() {
            self.codex_total = total;
        }
        self.context_window = window;
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
                tool_kind: t.kind.clone(),
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
                code: None,
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
            self.native_record(r, out);
            return;
        }
        if let Some(version) = r.get("version").and_then(Value::as_str) {
            self.native_version = Some(version.to_owned());
        }
        if let Some(cwd) = r.get("cwd").and_then(Value::as_str) {
            self.cwd.get_or_insert_with(|| cwd.to_owned());
        }
        let kind = r.get("type").and_then(Value::as_str).unwrap_or("");
        let Some(message) = r.get("message") else {
            self.native_record(r, out);
            return;
        };
        if let Some(model) = message.get("model").and_then(Value::as_str) {
            self.set_model(model, out);
        }
        match kind {
            "user" => {
                if r.get("isMeta").and_then(Value::as_bool) == Some(true) {
                    self.native_record(r, out);
                    return;
                }
                match message.get("content") {
                    Some(Value::String(text)) => self.user_prompt(text, r, out),
                    Some(Value::Array(blocks)) => {
                        let mut prompt = vec![];
                        for b in blocks {
                            match b.get("type").and_then(Value::as_str) {
                                Some("tool_result") => {
                                    let id =
                                        b.get("tool_use_id").and_then(Value::as_str).unwrap_or("");
                                    let failed =
                                        b.get("is_error").and_then(Value::as_bool) == Some(true);
                                    if !self.tools.contains_key(id) {
                                        self.native_record(r, out);
                                    }
                                    self.tool_end(id, content_text(b.get("content")), failed, out);
                                }
                                Some("text") => prompt.push(
                                    b.get("text")
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_owned(),
                                ),
                                _ => self.native_record(r, out),
                            }
                        }
                        if !prompt.is_empty() {
                            self.user_prompt(&prompt.join("\n"), r, out);
                        }
                    }
                    _ => self.native_record(r, out),
                }
            }
            "assistant" => {
                if !message
                    .get("content")
                    .and_then(Value::as_array)
                    .is_some_and(|b| !b.is_empty())
                {
                    self.native_record(r, out);
                }
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
                            } else {
                                self.native_record(r, out);
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
                            } else {
                                self.native_record(r, out);
                            }
                        }
                        Some("tool_use") => {
                            let id = b.get("id").and_then(Value::as_str).unwrap_or("");
                            let name = b.get("name").and_then(Value::as_str).unwrap_or("tool");
                            let input = b.get("input").map(Value::to_string).unwrap_or_default();
                            if self.tools.contains_key(id) {
                                self.native_record(r, out);
                            }
                            self.tool_start(id, name, input, out);
                        }
                        _ => self.native_record(r, out),
                    }
                }
                self.claude_usage(r, message, out);
                if message.get("stop_reason").and_then(Value::as_str) == Some("end_turn")
                    && self.tools.is_empty()
                {
                    self.end_turn(StopReason::EndTurn, out);
                }
            }
            _ => self.native_record(r, out),
        }
    }
    fn user_prompt(&mut self, text: &str, record: &Value, out: &mut Vec<Event>) {
        // harness bookkeeping that Claude Code stores as user text is not a prompt
        let t = text.trim_start();
        if t.is_empty()
            || t.starts_with("<command-")
            || t.starts_with("<local-command")
            || t.starts_with("<system-reminder>")
        {
            self.native_record(record, out);
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
                if let Some(version) = p.get("cli_version").and_then(Value::as_str) {
                    self.native_version = Some(version.to_owned());
                }
                if let Some(cwd) = p.get("cwd").and_then(Value::as_str) {
                    self.cwd = Some(cwd.to_owned());
                }
            }
            "turn_context" => {
                if let Some(model) = p.get("model").and_then(Value::as_str) {
                    self.set_model(model, out);
                }
            }
            "event_msg" => match ptype {
                "task_started" => {
                    self.end_turn(StopReason::EndTurn, out);
                    self.begin_turn(out);
                }
                "task_complete" => self.end_turn(StopReason::EndTurn, out),
                "turn_aborted" => self.end_turn(StopReason::Interrupted, out),
                "token_count" => self.codex_usage(&p, out),
                _ => self.native_record(r, out),
            },
            "response_item" => match ptype {
                "message" => {
                    if p.get("content")
                        .and_then(Value::as_array)
                        .is_some_and(|blocks| {
                            blocks.iter().any(|b| {
                                !matches!(
                                    b.get("type").and_then(Value::as_str),
                                    Some("input_text" | "output_text" | "text")
                                )
                            })
                        })
                    {
                        self.native_record(r, out);
                    }
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
                        _ => self.native_record(r, out),
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
                    if self.tools.contains_key(id) {
                        self.native_record(r, out);
                    }
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
                _ => self.native_record(r, out),
            },
            _ => self.native_record(r, out),
        }
    }
}

/// §4a: required-but-unreported counts are named placeholders, never fabricated facts.
fn parse_usage(u: &Value, cached: &str, reasoning: Option<&str>) -> Option<Usage> {
    let input = u.get("input_tokens").and_then(Value::as_u64);
    let output = u.get("output_tokens").and_then(Value::as_u64);
    let cached = u.get(cached).and_then(Value::as_u64);
    let creation = u
        .get("cache_creation_input_tokens")
        .or_else(|| u.get("cache_write_input_tokens"))
        .and_then(Value::as_u64);
    let reasoning = reasoning
        .and_then(|key| u.get(key))
        .and_then(Value::as_u64)
        .or_else(|| {
            u.pointer("/output_tokens_details/thinking_tokens")
                .and_then(Value::as_u64)
        });
    if [input, output, cached, creation, reasoning]
        .iter()
        .all(Option::is_none)
    {
        return None;
    }
    let mut unreported = vec![];
    if input.is_none() {
        unreported.push("input_tokens".into());
    }
    if output.is_none() {
        unreported.push("output_tokens".into());
    }
    Some(Usage {
        input_tokens: input.unwrap_or(0),
        output_tokens: output.unwrap_or(0),
        cached_input_tokens: cached,
        cache_creation_input_tokens: creation,
        reasoning_tokens: reasoning,
        source: Some(NumberSource::Reported),
        unreported,
        ..Usage::default()
    })
}
fn add_usage(a: &Usage, b: &Usage) -> Option<Usage> {
    let optional = |a: Option<u64>, b: Option<u64>| a.zip(b).and_then(|(a, b)| a.checked_add(b));
    let unreported: Vec<String> = a
        .unreported
        .iter()
        .chain(&b.unreported)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut backend = a.backend.clone();
    if let Some(backend) = &mut backend
        && b.backend.as_ref().and_then(|b| b.version.as_ref()) != backend.version.as_ref()
    {
        backend.version = None;
    }
    Some(Usage {
        input_tokens: if unreported.iter().any(|n| n == "input_tokens") {
            0
        } else {
            a.input_tokens.checked_add(b.input_tokens)?
        },
        output_tokens: if unreported.iter().any(|n| n == "output_tokens") {
            0
        } else {
            a.output_tokens.checked_add(b.output_tokens)?
        },
        cached_input_tokens: optional(a.cached_input_tokens, b.cached_input_tokens),
        cache_creation_input_tokens: optional(
            a.cache_creation_input_tokens,
            b.cache_creation_input_tokens,
        ),
        reasoning_tokens: optional(a.reasoning_tokens, b.reasoning_tokens),
        scope: Some(UsageScope::Turn),
        source: Some(NumberSource::Derived),
        backend,
        model: if a.model == b.model {
            a.model.clone()
        } else {
            None
        },
        unreported,
        ..Usage::default()
    })
}

fn tool_body(t: OpenTool, output: Vec<OutputBlock>) -> Item {
    Item::ToolCall {
        title: None,
        server: None,
        raw_output: None,
        call_id: t.call_id,
        name: t.name,
        tool_kind: t.kind.clone(),
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
    let redactor = crate::redact::Set::default();
    crate::capability::register(&redactor).await;
    import_file_with_redactor(native, path, sessions, redactor).await
}
pub async fn import_file_with_redactor(
    native: Option<Native>,
    path: &std::path::Path,
    sessions: std::path::PathBuf,
    redactor: crate::redact::Set,
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
        let new = p.project(r);
        let at = p.ts_ms.or(ts).unwrap_or_else(crate::store::now_ms);
        events.extend(new.into_iter().map(|e| (e, at)));
    }
    let at = p.ts_ms.unwrap_or_else(crate::store::now_ms);
    events.extend(p.finish(StopReason::EndTurn).into_iter().map(|event| {
        (
            Projected {
                event,
                native: None,
                ext: BTreeMap::new(),
            },
            at,
        )
    }));
    let meta = SessionMeta {
        source: Some(Source {
            harness: native.label().into(),
            native_id: records.iter().find_map(|r| {
                r.get("sessionId")
                    .or_else(|| r.pointer("/payload/id"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            }),
            version: p.native_version.clone(),
            parser_revision: Some(2),
        }),
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
        crate::store::Store::open(sessions.clone(), meta, Some(OpId::new()), redactor.clone())
            .await
            .map_err(|e| e.to_string())?;
    store
        .append(
            Event::ProfileChanged {
                profile: native.label().into(),
                model: native.label().into(),
            },
            None,
        )
        .await
        .map_err(|e| e.to_string())?;
    for (e, at) in events {
        spill::append_projected(&mut store, e.event, None, at, e.native, e.ext)
            .await
            .map_err(|e| e.to_string())?;
    }
    store
        .append(Event::SessionClosed, None)
        .await
        .map_err(|e| e.to_string())?;
    drop(crate::frontends::ir_scan::on_close(
        sessions.join(format!("{session}.jsonl")),
        redactor,
        |text| eprintln!("Info: {text}"),
    ));
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
