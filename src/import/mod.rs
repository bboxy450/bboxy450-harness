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
/// Revision of the shared native Claude Code/Codex Projector mapping.
pub const PARSER_REVISION: u32 = 3;

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

pub const MATERIALIZER_REVISION: u32 = 1;
/// SHA256(ASCII "bboxy450-native-id-v1", NUL, namespace[32], domain_len_u64be,
/// domain_utf8 || ordinal_u64be), first 16 bytes as an existing ULID-shaped ID.
pub fn materialized_id(namespace: &[u8; 32], domain: &str, ordinal: u64) -> ulid::Ulid {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"bboxy450-native-id-v1");
    hash.update([0u8]);
    hash.update(namespace);
    hash.update((domain.len() as u64).to_be_bytes());
    hash.update(domain.as_bytes());
    hash.update(ordinal.to_be_bytes());
    let digest = hash.finalize();
    ulid::Ulid::from(u128::from_be_bytes(
        digest[..16].try_into().expect("SHA256 width"),
    ))
}
struct DeterministicIds {
    namespace: [u8; 32],
    ordinals: BTreeMap<&'static str, u64>,
}
impl DeterministicIds {
    fn next(&mut self, domain: &'static str) -> ulid::Ulid {
        let ordinal = self.ordinals.entry(domain).or_default();
        let id = materialized_id(&self.namespace, domain, *ordinal);
        *ordinal += 1;
        id
    }
}

/// The streaming projection of one native session.
pub struct Projector {
    deterministic_ids: Option<DeterministicIds>,
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
    /// First available native observations; absent fields stay absent.
    pub vcs: Option<Vcs>,
    pub model: Option<String>,
    /// The native clock of the last record that carried one (ms since the epoch).
    pub ts_ms: Option<u64>,
    #[cfg(feature = "test-fixtures")]
    restore_drop: bool,
}
impl Projector {
    pub fn new(native: Native) -> Self {
        Self {
            deterministic_ids: None,
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
            vcs: None,
            model: None,
            ts_ms: None,
            #[cfg(feature = "test-fixtures")]
            restore_drop: false,
        }
    }
    fn deterministic(native: Native, namespace: [u8; 32]) -> Self {
        let mut p = Self::new(native);
        p.deterministic_ids = Some(DeterministicIds {
            namespace,
            ordinals: BTreeMap::new(),
        });
        p
    }
    fn item_id(&mut self) -> ItemId {
        self.deterministic_ids
            .as_mut()
            .map(|ids| ItemId(ids.next("item")))
            .unwrap_or_default()
    }
    fn turn_id(&mut self) -> TurnId {
        self.deterministic_ids
            .as_mut()
            .map(|ids| TurnId(ids.next("turn")))
            .unwrap_or_default()
    }
    fn generated_call_id(&mut self) -> String {
        self.deterministic_ids
            .as_mut()
            .map(|ids| ids.next("call").to_string())
            .unwrap_or_else(|| ItemId::new().to_string())
    }
    pub fn push(&mut self, record: &Value) -> Vec<Event> {
        if let Some(observed) = edge::vcs(self.native, record) {
            let vcs = self.vcs.get_or_insert_with(Vcs::default);
            vcs.repo = vcs.repo.take().or(observed.repo);
            vcs.branch = vcs.branch.take().or(observed.branch);
            vcs.commit = vcs.commit.take().or(observed.commit);
        }
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
    fn native_record(&mut self, record: &Value, out: &mut Vec<Event>) {
        #[cfg(feature = "test-fixtures")]
        if self.restore_drop {
            return;
        }
        let record = edge::sanitized_record(self.native, record);
        if out.iter().any(|e| matches!(e, Event::ItemCompleted { body: Item::Native { raw, .. }, .. } if raw == &record)) { return; }
        let item = self.item_id();
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
                record_type: record_type(&record),
                raw: record,
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
        let turn = self.turn_id();
        out.push(Event::TurnQueued { turn });
        out.push(Event::TurnStarted {
            turn,
            model: self.model.clone(),
            origin: None,
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
            let item = self.item_id();
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
        let item = self.item_id();
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
            item: self.item_id(),
            call_id: if call_id.is_empty() {
                self.generated_call_id()
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
        vcs: p.vcs.clone(),
        dream: None,
        source: Some(Source {
            harness: native.label().into(),
            native_id: records.iter().find_map(|r| {
                r.get("sessionId")
                    .or_else(|| r.pointer("/payload/id"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            }),
            version: p.native_version.clone(),
            parser_revision: Some(PARSER_REVISION),
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

// Opt-in deterministic native custody, d20261103136344 (revision 1).
// Existing import/watch/reproject use Projector::new and unchanged default clocks.
// Origin/policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub const MAX_NATIVE_SNAPSHOT_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_NATIVE_RECORDS: usize = 10_000;
const MAX_NATIVE_CUSTODY_BYTES: u64 = 128 * 1024 * 1024;
pub struct NativeSource {
    file: std::fs::File,
    path: std::path::PathBuf,
    identity: [u64; 7],
}
pub struct NativeSnapshot {
    pub native: Native,
    pub namespace: [u8; 32],
    pub input_digest: String,
    records: Vec<Value>,
}
fn native_identity_stat(meta: &std::fs::Metadata) -> [u64; 7] {
    use std::os::unix::fs::MetadataExt;
    [
        meta.dev(),
        meta.ino(),
        meta.len(),
        meta.mtime() as u64,
        meta.mtime_nsec() as u64,
        meta.ctime() as u64,
        meta.ctime_nsec() as u64,
    ]
}
pub async fn open_native_source(path: std::path::PathBuf) -> std::io::Result<NativeSource> {
    crate::runtime::blocking(move || {
        use std::{io, os::unix::fs::OpenOptionsExt};
        crate::runtime::assert_off_rt_worker();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(&path)?;
        let meta = file.metadata()?;
        if !meta.is_file() || meta.len() > MAX_NATIVE_SNAPSHOT_BYTES {
            return Err(io::Error::other(
                "native snapshot is not a bounded regular file",
            ));
        }
        Ok(NativeSource {
            file,
            path,
            identity: native_identity_stat(&meta),
        })
    })
    .await
}
pub async fn snapshot_native(
    mut source: NativeSource,
    set: crate::redact::Set,
) -> std::io::Result<NativeSnapshot> {
    crate::runtime::blocking(move || {
        use std::io::{self,Read};use sha2::{Digest,Sha256};crate::runtime::assert_off_rt_worker();
        let mut bytes=zeroize::Zeroizing::new(Vec::new());source.file.by_ref().take(MAX_NATIVE_SNAPSHOT_BYTES+1).read_to_end(&mut bytes)?;
        let after=source.file.metadata()?;let pathmeta=std::fs::symlink_metadata(&source.path)?;
        if bytes.len() as u64>MAX_NATIVE_SNAPSHOT_BYTES||!pathmeta.is_file()||source.identity!=native_identity_stat(&after)||source.identity!=native_identity_stat(&pathmeta)||after.len()!=bytes.len() as u64 {
            return Err(io::Error::other("native snapshot changed during capture"));
        }
        let text=std::str::from_utf8(&bytes).map_err(|_|io::Error::other("native snapshot is not UTF-8"))?;
        let mut records=Vec::new();
        for line in text.lines().filter(|line|!line.trim().is_empty()) {
            if records.len()>=MAX_NATIVE_RECORDS{return Err(io::Error::other("native snapshot record bound exceeded"));}
            records.push(serde_json::from_str::<Value>(line).map_err(|_|io::Error::other("malformed or incomplete native record"))?);
        }
        let native=records.iter().take(8).find_map(Native::sniff).ok_or_else(||io::Error::other("unrecognized native snapshot"))?;
        let mut records=records.into_iter().map(|r|set.value(edge::sanitized_record(native,&r))).collect::<Vec<_>>();
        // Hash and project the same canonical sequence, including object order.
        for record in &mut records {record.sort_all_objects();}
        let mut canonical=serde_json::to_value(&records)?;canonical.sort_all_objects();
        let input_digest=format!("{:x}",Sha256::digest(serde_json::to_vec(&canonical)?));
        let mut versioned=serde_json::json!({"native":native.label(),"parser_revision":PARSER_REVISION,"materializer_revision":MATERIALIZER_REVISION,
            "protocol":PROTOCOL,"writer_package":env!("CARGO_PKG_VERSION"),"input_digest":input_digest});versioned.sort_all_objects();
        // Fixed encoding: domain bytes including NUL, u64be byte length, canonical JSON.
        let encoded=serde_json::to_vec(&versioned)?;let mut hash=Sha256::new();hash.update(b"bboxy450-native-namespace-v1");hash.update([0u8]);hash.update((encoded.len() as u64).to_be_bytes());hash.update(encoded);
        Ok(NativeSnapshot{native,namespace:hash.finalize().into(),input_digest,records})
    }).await
}
struct NativeProjection {
    meta: SessionMeta,
    events: Vec<(Projected, u64)>,
    ext: BTreeMap<String, Value>,
    first: u64,
    last: u64,
}
fn project_native(snapshot: NativeSnapshot) -> NativeProjection {
    crate::runtime::assert_off_rt_worker();
    let NativeSnapshot {
        native,
        namespace,
        input_digest,
        records,
    } = snapshot;
    let source_id = records.iter().find_map(|r| {
        r.get("sessionId")
            .or_else(|| r.pointer("/payload/id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    });
    let clock = |r: &Value| {
        r.get("timestamp")
            .and_then(Value::as_str)
            .and_then(rfc3339_ms)
            .unwrap_or(0)
    };
    let first = records.first().map(clock).unwrap_or(0);
    let last = records.last().map(clock).unwrap_or(0);
    let mut p = Projector::deterministic(native, namespace);
    let mut events = Vec::new();
    for record in records {
        // A missing record clock is unknown zero, never an invented historical time.
        p.ts_ms = None;
        let projected = p.project(&record);
        let at = p.ts_ms.unwrap_or(0);
        events.extend(projected.into_iter().map(|event| (event, at)));
    }
    events.extend(p.finish(StopReason::Interrupted).into_iter().map(|event| {
        (
            Projected {
                event,
                native: None,
                ext: BTreeMap::new(),
            },
            last,
        )
    }));
    let meta = SessionMeta {
        session: SessionId(materialized_id(&namespace, "session", 0)),
        cwd: p.cwd.unwrap_or_else(|| "/".into()).into(),
        vcs: p.vcs,
        dream: None,
        source: Some(Source {
            harness: native.label().into(),
            native_id: source_id,
            version: p.native_version,
            parser_revision: Some(PARSER_REVISION),
        }),
        profile: Some(native.label().into()),
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: first,
    };
    let ext = BTreeMap::from([(
        "native_materialization".into(),
        serde_json::json!({"namespace":namespace.iter().map(|b|format!("{b:02x}")).collect::<String>(),"input_digest":input_digest,
        "materializer_revision":MATERIALIZER_REVISION,"parser_revision":PARSER_REVISION,"writer_package":env!("CARGO_PKG_VERSION"),"closure":"snapshot_eof","native_process_completion":"unproven"}),
    )]);
    NativeProjection {
        meta,
        events,
        ext,
        first,
        last,
    }
}
struct NativeBundle {
    log: Vec<u8>,
    blobs: BTreeMap<String, Vec<u8>>,
}
/// A private complete stage still owns its nonblocking namespace claim. Only
/// publish() can expose custody; dropping/crashing a stage advertises no pins.
pub struct NativeStage {
    pub session: SessionId,
    pub namespace: String,
    pub staging: std::path::PathBuf,
    sessions: std::path::PathBuf,
    bundle: NativeBundle,
    _claim: crate::dream::d0::Claim,
}
fn native_directory(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    if !std::fs::symlink_metadata(path)?.is_dir() {
        return Err(std::io::Error::other("unsafe native custody directory"));
    }
    Ok(())
}
fn read_native_regular(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::{
        io::{self, Read},
        os::unix::fs::OpenOptionsExt,
    };
    let f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    if !f.metadata()?.is_file() || f.metadata()?.len() > MAX_NATIVE_CUSTODY_BYTES {
        return Err(io::Error::other("unsafe native custody file"));
    }
    let mut bytes = Vec::new();
    f.take(MAX_NATIVE_CUSTODY_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_NATIVE_CUSTODY_BYTES {
        return Err(io::Error::other("native custody size bound exceeded"));
    }
    Ok(bytes)
}
fn native_bundle(root: &std::path::Path, session: SessionId) -> std::io::Result<NativeBundle> {
    use sha2::{Digest, Sha256};
    use std::io;
    crate::runtime::assert_off_rt_worker();
    let log = read_native_regular(&crate::store::log_path(root, session))?;
    if !log.ends_with(b"\n") {
        return Err(io::Error::other("incomplete native custody log"));
    }
    let events = log
        .split_inclusive(|b| *b == b'\n')
        .map(|line| {
            Envelope::from_line(std::str::from_utf8(line).map_err(io::Error::other)?)
                .map_err(io::Error::other)
        })
        .collect::<io::Result<Vec<_>>>()?;
    if !matches!(events.last().map(|e| &e.event), Some(Event::SessionClosed))
        || events
            .iter()
            .any(|e| e.session != session || e.epoch != Epoch(1))
    {
        return Err(io::Error::other("native custody identity/closure mismatch"));
    }
    crate::protocol::laws::check_authored(&events)
        .map_err(|_| io::Error::other("invalid authored native custody"))?;
    let mut blobs = BTreeMap::new();
    let directory = root.join(session.to_string()).join("blobs");
    if directory.exists() {
        if !std::fs::symlink_metadata(&directory)?.is_dir() {
            return Err(io::Error::other("unsafe native blob directory"));
        }
        for file in std::fs::read_dir(&directory)? {
            let file = file?;
            let name = file
                .file_name()
                .into_string()
                .map_err(|_| io::Error::other("invalid native blob name"))?;
            let bytes = read_native_regular(&file.path())?;
            if name.len() != 64
                || !name.bytes().all(|c| c.is_ascii_hexdigit())
                || format!("{:x}", Sha256::digest(&bytes)) != name
            {
                return Err(io::Error::other("invalid native blob hash"));
            }
            blobs.insert(name, bytes);
        }
    }
    // Typed blob positions and shared spill preview markers must all resolve.
    fn check(value: &Value, blobs: &BTreeMap<String, Vec<u8>>) -> io::Result<()> {
        if let Value::Object(o) = value {
            if o.len() == 3
                && o.contains_key("sha256")
                && o.contains_key("len")
                && o.contains_key("mime")
            {
                let b: BlobRef = serde_json::from_value(value.clone())?;
                if blobs
                    .get(&b.sha256)
                    .is_none_or(|bytes| bytes.len() as u64 != b.len)
                {
                    return Err(io::Error::other("native blob reference does not resolve"));
                }
                return Ok(());
            }
            for (key, v) in o {
                if key != "raw" {
                    check(v, blobs)?;
                }
            }
        } else if let Value::Array(a) = value {
            for v in a {
                check(v, blobs)?;
            }
        } else if let Some(text) = value.as_str() {
            static MARKER: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
            for c in MARKER
                .get_or_init(|| {
                    regex::Regex::new(r"\[full content: blob ([0-9a-f]{64}) \(([0-9]+) bytes\)\]")
                        .unwrap()
                })
                .captures_iter(text)
            {
                let len = c[2].parse::<usize>().map_err(io::Error::other)?;
                if blobs.get(&c[1]).is_none_or(|bytes| bytes.len() != len) {
                    return Err(io::Error::other("native preview blob does not resolve"));
                }
            }
        }
        Ok(())
    }
    for e in &events {
        if let Event::ItemCompleted { body, .. } = &e.event {
            check(&serde_json::to_value(body)?, &blobs)?;
        }
        for value in e.ext.values() {
            if value.get("$preview").is_some()
                && let Some(blob) = value.get("$blob")
            {
                check(blob, &blobs)?;
            }
        }
    }
    Ok(NativeBundle { log, blobs })
}
pub async fn stage_native(
    snapshot: NativeSnapshot,
    sessions: std::path::PathBuf,
    redactor: crate::redact::Set,
) -> std::io::Result<Option<NativeStage>> {
    use std::io;
    let namespace = snapshot
        .namespace
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let claim_root = sessions.join(".native-custody");
    let name = namespace.clone();
    let session_root = sessions.clone();
    let Some(claim) = crate::runtime::blocking(move || {
        native_directory(&session_root)?;
        crate::dream::d0::Claim::acquire(&claim_root, &name)
    })
    .await?
    else {
        return Ok(None);
    };
    let projection = crate::runtime::blocking(move || project_native(snapshot)).await;
    let session = projection.meta.session;
    let root = sessions.clone();
    let staging = crate::runtime::blocking(move || {
        use std::os::unix::fs::DirBuilderExt;
        crate::runtime::assert_off_rt_worker();
        native_directory(&root)?;
        let path = root.join(format!(".native-stage-{}", ulid::Ulid::new()));
        std::fs::DirBuilder::new().mode(0o700).create(&path)?;
        Ok::<_, io::Error>(path)
    })
    .await?;
    let mut store = crate::store::Store::open_import_staging(
        staging.clone(),
        projection.meta,
        redactor,
        projection.first,
        projection.ext,
    )
    .await?;
    let label = store.events[0].event.clone();
    let Event::SessionStarted { meta } = label else {
        return Err(io::Error::other("missing native initial envelope"));
    };
    let native = meta
        .source
        .as_ref()
        .ok_or_else(|| io::Error::other("missing native source"))?
        .harness
        .clone();
    store
        .append_at(
            Event::ProfileChanged {
                profile: native.clone(),
                model: native,
            },
            None,
            projection.first,
        )
        .await?;
    for (event, at) in projection.events {
        spill::append_projected_staging(&mut store, event.event, at, event.native, event.ext)
            .await?;
        tokio::task::yield_now().await;
    }
    store
        .append_at(Event::SessionClosed, None, projection.last)
        .await?;
    drop(store);
    let path = staging.clone();
    let bundle = crate::runtime::blocking(move || native_bundle(&path, session)).await?;
    Ok(Some(NativeStage {
        session,
        namespace,
        staging,
        sessions,
        bundle,
        _claim: claim,
    }))
}
impl NativeStage {
    pub async fn publish(self) -> std::io::Result<std::path::PathBuf> {
        crate::runtime::blocking(move || {
            use std::io;
            crate::runtime::assert_off_rt_worker();
            let target = crate::store::log_path(&self.sessions, self.session);
            let validate = || -> io::Result<()> {
                let actual = native_bundle(&self.sessions, self.session)?;
                if actual.log != self.bundle.log
                    || self
                        .bundle
                        .blobs
                        .iter()
                        .any(|(name, bytes)| actual.blobs.get(name) != Some(bytes))
                {
                    return Err(io::Error::other("conflicting native custody bytes"));
                }
                Ok(())
            };
            match std::fs::symlink_metadata(&target) {
                Ok(_) => {
                    validate()?;
                    let _ = std::fs::remove_dir_all(&self.staging);
                    return Ok(target);
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            let session_dir = self.sessions.join(self.session.to_string());
            native_directory(&session_dir)?;
            let blobs = session_dir.join("blobs");
            native_directory(&blobs)?;
            for (name, bytes) in &self.bundle.blobs {
                let source = self
                    .staging
                    .join(self.session.to_string())
                    .join("blobs")
                    .join(name);
                let destination = blobs.join(name);
                match crate::store::publish_no_replace(&source, &destination) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e),
                }
                if read_native_regular(&destination)? != *bytes {
                    return Err(io::Error::other("conflicting native custody blob"));
                }
            }
            crate::runtime::sync_all(&std::fs::File::open(&blobs)?)?;
            crate::runtime::sync_all(&std::fs::File::open(&session_dir)?)?;
            let source = crate::store::log_path(&self.staging, self.session);
            match crate::store::publish_no_replace(&source, &target) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
            crate::runtime::sync_all(&std::fs::File::open(&self.sessions)?)?;
            validate()?;
            let _ = std::fs::remove_dir_all(&self.staging);
            Ok(target)
        })
        .await
    }
}
pub async fn materialize_native(
    path: std::path::PathBuf,
    sessions: std::path::PathBuf,
    redactor: crate::redact::Set,
) -> std::io::Result<Option<std::path::PathBuf>> {
    let snapshot = snapshot_native(open_native_source(path).await?, redactor.clone()).await?;
    match stage_native(snapshot, sessions, redactor).await? {
        Some(stage) => stage.publish().await.map(Some),
        None => Ok(None),
    }
}
