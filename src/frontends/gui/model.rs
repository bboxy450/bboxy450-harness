// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Pure, retained projection of the protocol. No UI or runtime dependency.
//!
//! A subscription is ordered and delivers each ephemeral envelope once. The wire
//! has no delta ID: identical adjacent text deltas can be legitimate, and cannot
//! be deduplicated by content. Durable seqs are deduplicated defensively. The
//! bridge owns exactly one subscription and publishes complete projection values,
//! so replacing/coalescing UI notifications cannot replay an individual delta.
use crate::protocol::*;
use im::{OrdMap, Vector};

#[derive(Clone, Debug, PartialEq)]
pub struct ViewItem {
    pub id: ItemId,
    pub turn: Option<TurnId>,
    pub parent: Option<ItemId>,
    pub started: Seq,
    pub completed: Option<Seq>,
    pub kind: ItemKind,
    pub body: Item,
    pub status: Option<ItemStatus>,
    /// Transient terminal output; the completed body's blob is authoritative.
    pub terminal_tail: String,
    /// Envelope clock at start and completion, for durations only.
    pub started_ms: u64,
    pub completed_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TurnState {
    Queued,
    Running,
    Ended {
        reason: StopReason,
        usage: Option<Box<Usage>>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    Resubscribe { after: Option<(Epoch, Seq)> },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Transcript {
    pub session: Option<SessionId>,
    pub epoch: Epoch,
    pub cursor: Seq,
    pub meta: Option<SessionMeta>,
    pub profile: Option<(String, String)>,
    pub context: Option<String>,
    pub agent_config: Option<AgentConfig>,
    pub items: OrdMap<ItemId, ViewItem>,
    pub order: Vector<ItemId>,
    pub turns: OrdMap<TurnId, TurnState>,
    pub turn_origins: OrdMap<TurnId, TurnOrigin>,
    pub phase: Phase,
    pub closed: bool,
    pub unknown: u64,
    pub last_unknown_tag: Option<String>,
    pub duplicate_durable: u64,
}
impl Default for Transcript {
    fn default() -> Self {
        Self {
            session: None,
            epoch: Epoch(0),
            cursor: Seq(0),
            meta: None,
            profile: None,
            context: None,
            agent_config: None,
            items: OrdMap::new(),
            order: Vector::new(),
            turns: OrdMap::new(),
            turn_origins: OrdMap::new(),
            phase: Phase::Idle,
            closed: false,
            unknown: 0,
            last_unknown_tag: None,
            duplicate_durable: 0,
        }
    }
}
impl Transcript {
    pub fn hello(&mut self, session: SessionId, epoch: Epoch) {
        if self.session.is_some_and(|s| s != session) {
            *self = Self::default();
        }
        self.session = Some(session);
        self.epoch = epoch;
    }
    pub fn apply(&mut self, e: &Envelope) -> Effect {
        // v0.2 is additive. Read its optional head/live fields through the wire
        // representation so this consumer builds both before and after the core
        // v0.2 enum change, without defining a second protocol or changing it.
        let extension = match &e.event {
            Event::ItemStarted { .. }
            | Event::ItemSnapshot {
                partial: Item::Terminal { .. },
                ..
            } => serde_json::to_value(&e.event).ok(),
            _ => None,
        };
        self.reduce(e, extension.as_ref())
    }
    /// Preserve additive v0.2 fields during fixture replay, including when the
    /// linked core's older decoder does not yet retain those fields itself.
    pub fn apply_wire(&mut self, wire: &serde_json::Value) -> Result<Effect, serde_json::Error> {
        let mut envelope: Envelope = serde_json::from_value(wire.clone())?;
        if let Some(meta) = &self.meta {
            envelope.project_into(meta);
        }
        Ok(self.reduce(&envelope, wire.get("event")))
    }
    fn reduce(&mut self, e: &Envelope, wire: Option<&serde_json::Value>) -> Effect {
        if !e.v.is_readable() || self.session.is_some_and(|s| s != e.session) {
            return Effect::None;
        }
        self.session = Some(e.session);
        if let Event::Reset { epoch, .. } = e.event {
            if epoch < self.epoch {
                return Effect::None;
            }
            *self = Self {
                session: Some(e.session),
                epoch,
                ..Self::default()
            };
            return Effect::Resubscribe { after: None };
        }
        if let Some(seq) = e.seq {
            if seq <= self.cursor {
                self.duplicate_durable += 1;
                return Effect::None;
            }
            self.cursor = seq;
            self.epoch = self.epoch.max(e.epoch);
        } else if e.epoch != self.epoch {
            return Effect::None;
        }
        match &e.event {
            Event::InputCapabilities { .. } => {}
            Event::SessionStarted { meta } => self.meta = Some(meta.clone()),
            Event::ProfileChanged { profile, model } => {
                self.profile = Some((profile.clone(), model.clone()))
            }
            Event::TurnQueued { turn } => {
                self.turns.insert(*turn, TurnState::Queued);
            }
            Event::TurnStarted { turn, origin, .. } => {
                self.turns.insert(*turn, TurnState::Running);
                self.turn_origins
                    .insert(*turn, origin.clone().unwrap_or(TurnOrigin::User));
            }
            Event::TurnEnded {
                turn,
                reason,
                usage,
            } => {
                self.turns.insert(
                    *turn,
                    TurnState::Ended {
                        reason: reason.clone(),
                        usage: usage.clone().map(Box::new),
                    },
                );
            }
            Event::ItemStarted {
                turn,
                item,
                kind,
                parent,
                ..
            } => {
                // IDs are never reused. A reconnect cannot reopen a completed item.
                if !self.items.contains_key(item) {
                    self.order.push_back(*item);
                    let mut body = empty_body(kind.clone(), *item);
                    if let Some(head) = wire.and_then(|v| v.get("head")) {
                        apply_head(&mut body, head);
                    }
                    self.items.insert(
                        *item,
                        ViewItem {
                            id: *item,
                            turn: *turn,
                            parent: *parent,
                            started: e.seq.unwrap_or(e.at),
                            completed: None,
                            kind: kind.clone(),
                            body,
                            status: None,
                            terminal_tail: String::new(),
                            started_ms: e.ts_ms,
                            completed_ms: None,
                        },
                    );
                }
            }
            Event::ItemCompleted { item, status, body } => {
                if let Some(row) = self.items.get_mut(item) {
                    row.body = body.clone();
                    row.status = Some(status.clone());
                    row.completed = e.seq;
                    row.completed_ms = Some(e.ts_ms);
                    if !matches!(row.body, Item::ToolCall { .. }) {
                        row.terminal_tail.clear();
                    }
                }
            }
            Event::ItemDelta { item, delta } => {
                if let Some(row) = self.open_at(*item, e.at) {
                    if let (Item::Terminal { .. }, Delta::Output { append, .. }) =
                        (&row.body, delta)
                    {
                        append_tail(&mut row.terminal_tail, append);
                        row.body.apply(delta);
                    } else {
                        row.body.apply(delta);
                    }
                }
            }
            Event::ItemSnapshot { item, partial } => {
                if let Some(row) = self.open_at(*item, e.at)
                    && partial.kind() == row.kind
                {
                    row.body = partial.clone();
                    if !matches!(partial, Item::ToolCall { .. }) {
                        row.terminal_tail.clear();
                    }
                    if matches!(partial, Item::Terminal { .. })
                        && let Some(live) = wire
                            .and_then(|v| v.pointer("/partial/live"))
                            .and_then(serde_json::Value::as_str)
                    {
                        append_tail(&mut row.terminal_tail, live);
                    }
                }
            }
            Event::Status { phase } => self.phase = phase.clone(),
            Event::Lagged { resume_after } => {
                return Effect::Resubscribe {
                    after: Some((self.epoch, *resume_after)),
                };
            }
            Event::SessionClosed => {
                self.closed = true;
                self.phase = Phase::Closed;
            }
            Event::Unknown(raw) => {
                self.unknown += 1;
                self.last_unknown_tag = Some(unknown_tag(raw).into());
            }
            Event::ContextChanged {
                mode,
                permission_mode,
                config,
            } => {
                if let Some(value) = config.as_ref().and_then(|c| c.get("acp_controls")) {
                    self.agent_config = serde_json::from_value(value.clone()).ok();
                    return Effect::None;
                }
                self.context = Some(format!(
                    "Context: {} · {}{}",
                    mode.as_deref().unwrap_or("unreported"),
                    permission_mode.as_deref().unwrap_or("unreported"),
                    config
                        .as_ref()
                        .map(|c| format!(" · {} settings", c.len()))
                        .unwrap_or_default()
                ))
            }
            Event::SessionRecovered { .. } => {
                self.agent_config = None;
            }
            Event::OpRejected {
                reason: Rejection::InvalidInput { detail },
                ..
            } if detail.starts_with("Agent configuration") => {
                if let Some(c) = &mut self.agent_config {
                    c.status = detail.clone();
                }
            }
            Event::OpRejected { .. } | Event::Reset { .. } => {}
        }
        if let Some(delta) = e
            .ext
            .get("acp")
            .and_then(|v| v.pointer("/_meta/terminal_output_delta"))
            && let (Some(terminal), Some(data)) = (
                delta.get("terminal_id").and_then(serde_json::Value::as_str),
                delta.get("data").and_then(serde_json::Value::as_str),
            )
        {
            let item = self.order.iter().rev().find(|id| self.items.get(id).is_some_and(|row| matches!(&row.body, Item::ToolCall { call_id, .. } if call_id == terminal))).copied();
            if let Some(row) = item.and_then(|id| self.items.get_mut(&id)) {
                append_tail(&mut row.terminal_tail, data);
            }
        }
        Effect::None
    }
    fn open_at(&mut self, item: ItemId, at: Seq) -> Option<&mut ViewItem> {
        self.items
            .get_mut(&item)
            .filter(|row| row.status.is_none() && at >= row.started)
    }
}
fn empty_body(kind: ItemKind, id: ItemId) -> Item {
    // Reuse the protocol's kind-complete constructor, stripping recovery-only
    // placeholders. Starting an item is not an interrupted/failed completion.
    let mut body = Item::interrupted(kind, id);
    match &mut body {
        Item::ToolCall {
            name,
            input_raw,
            output,
            ..
        } => {
            name.clear();
            input_raw.clear();
            output.clear();
        }
        Item::Approval {
            subject,
            decision,
            by,
            options,
        } => {
            subject.summary = "Approval pending".into();
            *decision = None;
            *by = None;
            // The ItemSnapshot supplies the actual subject and offered options.
            options.clear();
        }
        Item::Notice { text, .. } => text.clear(),
        _ => {}
    }
    body
}

fn apply_head(body: &mut Item, head: &serde_json::Value) {
    let text = |field: &str| {
        head.get(field)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    };
    match body {
        Item::ToolCall {
            call_id,
            name,
            tool_kind,
            ..
        } if head.get("kind").and_then(serde_json::Value::as_str) == Some("tool_call") => {
            if let Some(id) = text("call_id") {
                *call_id = id;
            }
            if let Some(value) = text("name") {
                *name = value;
            }
            if let Some(kind) = head
                .get("tool_kind")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
            {
                *tool_kind = kind;
            }
        }
        Item::Terminal {
            terminal, command, ..
        } if head.get("kind").and_then(serde_json::Value::as_str) == Some("terminal") => {
            if let Some(value) = text("terminal") {
                *terminal = value;
            }
            if let Some(value) = text("command") {
                *command = value;
            }
        }
        _ => {}
    }
}
fn append_tail(tail: &mut String, append: &str) {
    const LIMIT: usize = 64 * 1024;
    tail.push_str(append);
    if tail.len() > LIMIT {
        let mut start = tail.len() - (LIMIT - '…'.len_utf8());
        while !tail.is_char_boundary(start) {
            start += 1;
        }
        *tail = format!("…{}", &tail[start..]);
    }
}

#[cfg(test)]
mod terminal_metadata_tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn acp_terminal_metadata_survives_completion_and_duplicate_replay() {
        let body = json!({"kind":"tool_call","call_id":"exec-list","name":"List files","tool_kind":"read","input_raw":"","output":[{"type":"json","value":{"exit_code":0}}],"locations":[],"spill":null,"raw_output":{"exit_code":0}});
        let envelope = |seq: u64, event: Value, data: Option<&str>| {
            json!({"v":{"major":1,"minor":2},"session":"00000000000000000000000001","epoch":1,"seq":seq,"at":seq,"ts_ms":seq,"cause":null,
                "ext":data.map(|text|json!({"acp":{"_meta":{"terminal_output_delta":{"terminal_id":"exec-list","data":text}}}})).unwrap_or_else(||json!({})),"event":event})
        };
        let item = "00000000000000000000000002";
        let start = envelope(
            1,
            json!({"type":"item_started","turn":null,"item":item,"kind":"tool_call","parent":null,"head":{"kind":"tool_call","call_id":"exec-list","name":"List files","tool_kind":"read"}}),
            None,
        );
        let first: Envelope = serde_json::from_value(start.clone()).unwrap();
        let mut model = Transcript::default();
        model.hello(first.session, first.epoch);
        model.apply_wire(&start).unwrap();
        model
            .apply_wire(&envelope(
                2,
                json!({"type":"item_snapshot","item":item,"partial":body.clone()}),
                Some("one.rs\n"),
            ))
            .unwrap();
        let complete = envelope(
            3,
            json!({"type":"item_completed","item":item,"status":{"status":"completed"},"body":body}),
            Some("two.rs\n"),
        );
        model.apply_wire(&complete).unwrap();
        model.apply_wire(&complete).unwrap();
        let id: ItemId = serde_json::from_value(json!(item)).unwrap();
        assert_eq!(model.items[&id].terminal_tail, "one.rs\ntwo.rs\n");
        assert_eq!(model.duplicate_durable, 1);
    }
}
