// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Incremental bounded transcript law projection, selected-source byte validation.
//! Streaming non-fork subset of protocol/laws.rs. C1 rejects native/fork headers.
use super::c1_wire as w;
use crate::protocol::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
type OpenItem = (Option<TurnId>, ItemKind, Option<ItemId>, Option<ItemHead>);
#[derive(Default)]
pub struct Transcript {
    head: u64,
    epoch: u32,
    session: Option<SessionId>,
    queued: BTreeSet<TurnId>,
    started: BTreeSet<TurnId>,
    ended: BTreeSet<TurnId>,
    open: BTreeMap<ItemId, OpenItem>,
    items: BTreeSet<ItemId>,
    calls: BTreeSet<String>,
    closed: bool,
    endings: Vec<Seq>,
    open_turns: BTreeMap<TurnId, Seq>,
    clipped: Seq,
    replaced: Seq,
}
impl Transcript {
    /// One bounded owner entry for paced success/error settlement.
    pub(crate) fn drain_step(&mut self) -> bool {
        if self.queued.pop_first().is_some()
            || self.started.pop_first().is_some()
            || self.ended.pop_first().is_some()
            || self.open.pop_first().is_some()
            || self.items.pop_first().is_some()
            || self.calls.pop_first().is_some()
            || self.endings.pop().is_some()
            || self.open_turns.pop_first().is_some()
        {
            return true;
        }
        false
    }
    pub fn observe(&mut self, e: &Envelope) -> Result<(), laws::LawViolation> {
        let i = self.head as usize;
        let fail = |law, detail: &str| laws::LawViolation {
            line: i + 1,
            law,
            detail: detail.into(),
        };
        if !e.v.is_readable() {
            return Err(fail("version", "unsupported major"));
        }
        if let Some(id) = self.session {
            if id != e.session {
                return Err(fail("log-is-truth", "session changed"));
            }
        } else {
            self.session = Some(e.session);
        }
        if e.epoch.0 < self.epoch {
            return Err(fail("log-is-truth", "epoch decreased"));
        }
        self.epoch = e.epoch.0;
        if let Some(seq) = e.seq {
            if self.closed {
                return Err(fail("log-is-truth", "durable event after SessionClosed"));
            }
            if seq.0 != self.head + 1 {
                return Err(fail("log-is-truth", "durable gap or duplicate"));
            }
            self.head = seq.0;
        }
        if e.at.0 != self.head {
            return Err(fail("log-is-truth", "at is not the durable head"));
        }
        match &e.event {
            Event::SessionStarted { meta } => {
                if self.head != 1 || meta.session != e.session {
                    return Err(fail(
                        "log-is-truth",
                        "SessionStarted must be seq 1 of this session",
                    ));
                }
            }
            Event::TurnQueued { turn } => {
                if !self.queued.insert(*turn) {
                    return Err(fail("one-ending", "turn queued twice"));
                }
            }
            Event::TurnStarted { turn, origin, .. } => {
                // Agent turns have no UserTurn admission (and no TurnQueued).
                // Include them in the same accepted-turn/end accounting.
                if origin.as_ref() == Some(&TurnOrigin::Agent) {
                    self.queued.insert(*turn);
                }
                if !self.queued.contains(turn)
                    || self.ended.contains(turn)
                    || !self.started.insert(*turn)
                {
                    return Err(fail("one-ending", "invalid turn start"));
                }
                if let Some(seq) = e.seq {
                    self.open_turns.insert(*turn, seq);
                }
            }
            Event::TurnEnded { turn, reason, .. } => {
                if matches!(
                    reason,
                    StopReason::Error {
                        class: ErrorClass::Cancelled,
                        ..
                    }
                ) {
                    return Err(fail(
                        "one-ending",
                        "cancelled provider error is a Cancelled ending",
                    ));
                }
                if let Some(seq) = e.seq {
                    self.endings.push(seq);
                }
                self.open_turns.remove(turn);
                if !self.queued.contains(turn) || !self.ended.insert(*turn) {
                    return Err(fail("one-ending", "orphan or duplicate ending"));
                }
                if self
                    .open
                    .values()
                    .any(|(owner, _, _, _)| owner.as_ref() == Some(turn))
                {
                    return Err(fail("tool-pairing", "open item at turn end"));
                }
            }
            Event::ItemStarted {
                turn,
                item,
                kind,
                parent,
                head,
            } => {
                if let Some(t) = turn
                    && (!self.started.contains(t) || self.ended.contains(t))
                {
                    return Err(fail("one-ending", "item outside running turn"));
                }
                if !self.items.insert(*item) {
                    return Err(fail("one-ending", "item id reused"));
                }
                if parent.is_some_and(|p| !self.open.contains_key(&p)) {
                    return Err(fail("one-ending", "missing open parent"));
                }
                if e.v.major >= 1 || e.v.minor >= 2 {
                    let valid = matches!(head, Some(ItemHead::Unknown(_)))
                        || matches!(
                            (kind, head),
                            (ItemKind::ToolCall, Some(ItemHead::ToolCall { .. }))
                                | (ItemKind::Terminal, Some(ItemHead::Terminal { .. }))
                        )
                        || (!matches!(kind, ItemKind::ToolCall | ItemKind::Terminal)
                            && head.is_none());
                    if !valid {
                        return Err(fail("item-identity", "missing or mismatched durable head"));
                    }
                }
                self.open
                    .insert(*item, (*turn, kind.clone(), *parent, head.clone()));
            }
            Event::ItemCompleted { item, body, status } => {
                let boundary = match body {
                    Item::ContextTrim { clipped_through }
                        if matches!(status, ItemStatus::Completed) =>
                    {
                        Some((clipped_through, &mut self.clipped))
                    }
                    Item::Compaction {
                        replaced_through, ..
                    } if matches!(status, ItemStatus::Completed) => {
                        Some((replaced_through, &mut self.replaced))
                    }
                    _ => None,
                };
                if let Some((boundary, previous)) = boundary {
                    if *boundary < *previous {
                        return Err(fail("context-boundary", "reduction boundary decreased"));
                    }
                    let first = self.open_turns.values().min().copied();
                    let frontier = self
                        .endings
                        .iter()
                        .rev()
                        .copied()
                        .find(|s| first.is_none_or(|start| *s < start))
                        .unwrap_or(Seq(0));
                    if *boundary > frontier {
                        return Err(fail(
                            "context-boundary",
                            "reduction exceeds completed frontier",
                        ));
                    }
                    *previous = *boundary;
                }

                let Some((_, kind, _, head)) = self.open.remove(item) else {
                    return Err(fail("one-ending", "orphan or duplicate item completion"));
                };
                if body.kind() != kind && !kind.is_unknown() && !body.kind().is_unknown() {
                    return Err(fail("one-ending", "item kind changed"));
                }
                if self
                    .open
                    .values()
                    .any(|(_, _, parent, _)| parent.as_ref() == Some(item))
                {
                    return Err(fail("one-ending", "parent completed before child"));
                }
                if head.is_some()
                    && !matches!(head, Some(ItemHead::Unknown(_)))
                    && !body.kind().is_unknown()
                    && body.head() != head
                {
                    return Err(fail(
                        "item-identity",
                        "body identity differs from durable head",
                    ));
                }
                if matches!(body, Item::Terminal { live: Some(_), .. }) {
                    return Err(fail(
                        "item-identity",
                        "completed terminal contains live output",
                    ));
                }
                if let Item::ToolCall {
                    call_id, output, ..
                } = body
                    && (!self.calls.insert(call_id.clone()) || output.is_empty())
                {
                    return Err(fail("tool-pairing", "duplicate call id or absent result"));
                }
            }
            Event::ItemDelta { item, .. } | Event::ItemSnapshot { item, .. } => {
                if !self.open.contains_key(item) {
                    return Err(fail("one-ending", "delta/snapshot for closed item"));
                }
            }
            Event::SessionClosed => self.closed = true,
            Event::Lagged { .. } | Event::Reset { .. } => {
                return Err(fail("log-is-truth", "subscription control is not last"));
            }
            _ => {}
        }

        Ok(())
    }
    pub fn finish(&self) -> io::Result<()> {
        if !self.closed || self.queued != self.ended || !self.open.is_empty() {
            Err(io::Error::other("unfinished transcript"))
        } else {
            Ok(())
        }
    }
}
fn input_unknown(block: &InputBlock) -> bool {
    matches!(block, InputBlock::Unknown(_))
}
fn decision_unknown(d: &ApprovalDecision) -> bool {
    matches!(d, ApprovalDecision::Unknown(_))
}
fn plan_unknown(entries: &[PlanEntry]) -> bool {
    entries
        .iter()
        .any(|e| e.status.is_unknown() || e.priority.as_ref().is_some_and(Priority::is_unknown))
}
fn output_unknown(block: &OutputBlock) -> bool {
    matches!(
        block,
        OutputBlock::Unknown(_)
            | OutputBlock::Diff {
                change: Some(ChangeKind::Unknown(_)),
                ..
            }
    )
}
fn usage_unknown(usage: &Usage) -> bool {
    usage.scope.as_ref().is_some_and(UsageScope::is_unknown)
        || usage.backend.as_ref().is_some_and(|b| b.kind.is_unknown())
        || usage.source.as_ref().is_some_and(NumberSource::is_unknown)
        || usage.cost.as_ref().is_some_and(|c| c.source.is_unknown())
}
fn item_unknown(item: &Item) -> bool {
    match item {
        Item::Unknown(_) => true,
        Item::UserMessage { blocks } => blocks.iter().any(input_unknown),
        Item::ToolCall {
            tool_kind, output, ..
        } => tool_kind.is_unknown() || output.iter().any(output_unknown),
        Item::FileChange { change, .. } => matches!(change, ChangeKind::Unknown(_)),
        Item::Plan { entries } => plan_unknown(entries),
        Item::Approval {
            subject,
            options,
            decision,
            by,
        } => {
            subject.effect.is_unknown()
                || options.iter().any(ApprovalDecisionKind::is_unknown)
                || decision.as_ref().is_some_and(decision_unknown)
                || matches!(by, Some(Resolver::Unknown(_)))
        }
        Item::Notice { level, .. } => level.is_unknown(),
        Item::SystemMessage { role, .. } => role.is_unknown(),
        Item::Usage { usage } => usage_unknown(usage),
        Item::MemoryWrite { op, evidence, .. } => {
            op.is_unknown() || evidence.iter().any(EvidenceRef::is_unknown)
        }
        Item::MemoryState { state, refs, .. } => {
            state.is_unknown() || refs.iter().any(EvidenceRef::is_unknown)
        }
        Item::Trial {
            episodes,
            controls,
            verdict,
            ..
        } => {
            verdict.is_unknown()
                || episodes.iter().chain(controls).any(|episode| {
                    episode.origin.is_unknown()
                        || episode.runs.iter().any(|run| {
                            run.arm.is_unknown() || run.usage.as_ref().is_some_and(usage_unknown)
                        })
                })
        }
        Item::Adoption { sink, .. } => sink.kind.is_unknown(),
        Item::Receipt {
            kind,
            outcome,
            sink,
            ..
        } => {
            kind.is_unknown()
                || outcome.is_unknown()
                || sink.as_ref().is_some_and(|s| s.kind.is_unknown())
        }
        Item::Recall { sink, basis, .. } => {
            basis.is_unknown() || sink.as_ref().is_some_and(|s| s.kind.is_unknown())
        }
        _ => false,
    }
}
fn event_unknown(event: &Event) -> bool {
    match event {
        Event::Unknown(_) => true,
        Event::SessionStarted { meta } => meta.dream.as_ref().is_some_and(|dream| {
            dream.phase.is_unknown() || dream.trigger.is_unknown() || dream.scope.level.is_unknown()
        }),
        Event::ItemStarted { kind, head, .. } => {
            kind.is_unknown()
                || matches!(
                    head,
                    Some(
                        ItemHead::Unknown(_)
                            | ItemHead::ToolCall {
                                tool_kind: ToolKind::Unknown(_),
                                ..
                            }
                    )
                )
        }
        Event::ItemCompleted { status, body, .. } => {
            matches!(status, ItemStatus::Unknown(_)) || item_unknown(body)
        }
        Event::ItemSnapshot { partial, .. } => item_unknown(partial),
        Event::ItemDelta { delta, .. } => match delta {
            Delta::Unknown(_)
            | Delta::Output {
                stream: Stream::Unknown(_),
                ..
            } => true,
            Delta::Plan { entries } => plan_unknown(entries),
            _ => false,
        },
        Event::TurnEnded { reason, usage, .. } => {
            usage.as_ref().is_some_and(usage_unknown)
                || matches!(
                    reason,
                    StopReason::Unknown(_)
                        | StopReason::Error {
                            class: ErrorClass::Unknown(_),
                            ..
                        }
                )
        }
        Event::TurnStarted { origin, .. } => matches!(origin, Some(TurnOrigin::Unknown(_))),
        Event::OpRejected { reason, .. } => matches!(reason, Rejection::Unknown(_)),
        Event::Status { phase } => matches!(phase, Phase::Unknown(_)),
        _ => false,
    }
}

pub fn validate_event(e: &Envelope, source: &w::Source, index: u64) -> Result<(), w::Code> {
    if e.v.major != 1 || e.v.minor > 3 || event_unknown(&e.event) {
        return Err(w::Code::UnsupportedIr);
    }
    if e.session.to_string() != source.session.0
        || e.seq != Some(Seq(index))
        || e.at != Seq(index)
        || e.epoch.0 == 0
        || e.ts_ms > w::MAX_SAFE
        || !e.event.is_durable()
    {
        return Err(w::Code::MalformedIr);
    }
    if let Event::SessionStarted { meta } = &e.event {
        if index != 1
            || meta.session != e.session
            || meta.protocol.major != 1
            || meta.protocol.minor > 3
            || meta.source.is_some()
            || meta.fork.is_some()
        {
            return Err(w::Code::UnsupportedIr);
        }
        // Canonical binding is retained in metadata; never discover cwd/repo or invoke Git.
        if meta.vcs.as_ref().and_then(|v| v.repo.as_deref()) != Some(source.repo.0.as_str()) {
            return Err(w::Code::SourceMismatch);
        }
    } else if index == 1 {
        return Err(w::Code::MalformedIr);
    }
    Ok(())
}
/// Visit exactly typed blob carriers, never arbitrary JSON keys or extension data.
pub fn blobs(event: &Event) -> Result<Vec<&BlobRef>, w::Code> {
    let mut out = vec![];
    let mut push = |b| {
        out.try_reserve(1).map_err(|_| w::Code::ResourceLimit)?;
        out.push(b);
        Ok::<(), w::Code>(())
    };
    let item = match event {
        Event::ItemCompleted { body, .. } => Some(body),
        Event::ItemSnapshot { partial, .. } => Some(partial),
        _ => None,
    };
    if let Some(item) = item {
        match item {
            Item::UserMessage { blocks } => {
                for b in blocks {
                    match b {
                        InputBlock::Image { blob } | InputBlock::Audio { blob } => push(blob)?,
                        InputBlock::Resource {
                            blob: Some(blob), ..
                        } => push(blob)?,
                        _ => {}
                    }
                }
            }
            Item::ToolCall { output, spill, .. } => {
                if let Some(b) = spill {
                    push(b)?
                }
                for b in output {
                    match b {
                        OutputBlock::Image { blob } | OutputBlock::Audio { blob } => push(blob)?,
                        OutputBlock::Resource {
                            blob: Some(blob), ..
                        } => push(blob)?,
                        _ => {}
                    }
                }
            }
            Item::Terminal {
                output: Some(b), ..
            }
            | Item::Image { blob: b, .. }
            | Item::Compaction { summary: b, .. }
            | Item::Native { blob: Some(b), .. }
            | Item::Receipt { blob: Some(b), .. } => push(b)?,
            _ => {}
        }
    }
    Ok(out)
}
