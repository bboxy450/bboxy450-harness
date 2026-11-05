// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawViolation {
    pub line: usize,
    pub law: &'static str,
    pub detail: String,
}
impl std::fmt::Display for LawViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}: {}", self.line, self.law, self.detail)
    }
}
impl std::error::Error for LawViolation {}
/// Validate a complete session transcript, including recovered epochs and
/// optional final subscription control. Unknown variants still participate in seq.
pub fn check_transcript(events: &[Envelope]) -> Result<(), LawViolation> {
    let fork_at = events.first().and_then(|e| match &e.event {
        Event::SessionStarted { meta } => meta.fork.map(|(_, at)| at),
        _ => None,
    });
    let mut head = 0;
    let mut epoch = 0;
    let mut session = None;
    let mut queued = BTreeSet::new();
    let mut started = BTreeSet::new();
    let mut ended = BTreeSet::new();
    type OpenItem = (Option<TurnId>, ItemKind, Option<ItemId>, Option<ItemHead>);
    let mut open: BTreeMap<ItemId, OpenItem> = BTreeMap::new();
    let mut items = BTreeSet::new();
    let mut calls = BTreeSet::new();
    let mut closed = false;
    let mut endings = Vec::new();
    let mut open_turns = BTreeMap::new();
    let mut clipped = Seq(0);
    let mut replaced = Seq(0);
    for (i, e) in events.iter().enumerate() {
        let fail = |law, detail: &str| LawViolation {
            line: i + 1,
            law,
            detail: detail.into(),
        };
        if !e.v.is_readable() {
            return Err(fail("version", "unsupported major"));
        }
        let inherited =
            fork_at.is_some_and(|at| e.seq.is_some_and(|seq| seq > Seq(1) && seq <= at));
        if let Some(id) = session {
            if id != e.session && !inherited {
                return Err(fail("log-is-truth", "session changed"));
            }
        } else {
            session = Some(e.session);
        }
        if e.epoch.0 < epoch && !inherited {
            return Err(fail("log-is-truth", "epoch decreased"));
        }
        if !inherited {
            epoch = e.epoch.0;
        }
        if let Some(seq) = e.seq {
            if closed {
                return Err(fail("log-is-truth", "durable event after SessionClosed"));
            }
            if seq.0 != head + 1 {
                return Err(fail("log-is-truth", "durable gap or duplicate"));
            }
            head = seq.0;
        }
        if e.at.0 != head {
            return Err(fail("log-is-truth", "at is not the durable head"));
        }
        match &e.event {
            Event::SessionStarted { meta } => {
                if head != 1 || meta.session != e.session {
                    return Err(fail(
                        "log-is-truth",
                        "SessionStarted must be seq 1 of this session",
                    ));
                }
            }
            Event::TurnQueued { turn } => {
                if !queued.insert(*turn) {
                    return Err(fail("one-ending", "turn queued twice"));
                }
            }
            Event::TurnStarted { turn, origin, .. } => {
                // Agent turns have no UserTurn admission (and no TurnQueued).
                // Include them in the same accepted-turn/end accounting.
                if origin.as_ref() == Some(&TurnOrigin::Agent) {
                    queued.insert(*turn);
                }
                if !queued.contains(turn) || ended.contains(turn) || !started.insert(*turn) {
                    return Err(fail("one-ending", "invalid turn start"));
                }
                if let Some(seq) = e.seq {
                    open_turns.insert(*turn, seq);
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
                    endings.push(seq);
                }
                open_turns.remove(turn);
                if !queued.contains(turn) || !ended.insert(*turn) {
                    return Err(fail("one-ending", "orphan or duplicate ending"));
                }
                if open
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
                    && (!started.contains(t) || ended.contains(t))
                {
                    return Err(fail("one-ending", "item outside running turn"));
                }
                if !items.insert(*item) {
                    return Err(fail("one-ending", "item id reused"));
                }
                if parent.is_some_and(|p| !open.contains_key(&p)) {
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
                open.insert(*item, (*turn, kind.clone(), *parent, head.clone()));
            }
            Event::ItemCompleted { item, body, status } => {
                let boundary = match body {
                    Item::ContextTrim { clipped_through }
                        if matches!(status, ItemStatus::Completed) =>
                    {
                        Some((clipped_through, &mut clipped))
                    }
                    Item::Compaction {
                        replaced_through, ..
                    } if matches!(status, ItemStatus::Completed) => {
                        Some((replaced_through, &mut replaced))
                    }
                    _ => None,
                };
                if let Some((boundary, previous)) = boundary {
                    if *boundary < *previous {
                        return Err(fail("context-boundary", "reduction boundary decreased"));
                    }
                    let first = open_turns.values().min().copied();
                    let frontier = endings
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

                let Some((_, kind, _, head)) = open.remove(item) else {
                    return Err(fail("one-ending", "orphan or duplicate item completion"));
                };
                if body.kind() != kind && !kind.is_unknown() && !body.kind().is_unknown() {
                    return Err(fail("one-ending", "item kind changed"));
                }
                if open
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
                    && (!calls.insert(call_id.clone()) || output.is_empty())
                {
                    return Err(fail("tool-pairing", "duplicate call id or absent result"));
                }
            }
            Event::ItemDelta { item, .. } | Event::ItemSnapshot { item, .. } => {
                if !open.contains_key(item) {
                    return Err(fail("one-ending", "delta/snapshot for closed item"));
                }
            }
            Event::SessionClosed => closed = true,
            Event::Lagged { .. } | Event::Reset { .. } if i + 1 != events.len() => {
                return Err(fail("log-is-truth", "subscription control is not last"));
            }
            _ => {}
        }
    }
    if queued != ended {
        return Err(LawViolation {
            line: events.len(),
            law: "one-ending",
            detail: "unfinished turn".into(),
        });
    }
    if !open.is_empty() {
        return Err(LawViolation {
            line: events.len(),
            law: "one-ending",
            detail: "unfinished item".into(),
        });
    }
    Ok(())
}

/// Stricter validation for our own writers. Reader tolerance is not authorship.
pub fn check_authored(events: &[Envelope]) -> Result<(), LawViolation> {
    check_transcript(events)?;
    for (i, e) in events.iter().enumerate() {
        if e.v.major == 1 && e.v.minor >= 1 {
            let usage = match &e.event {
                Event::TurnEnded {
                    usage: Some(usage), ..
                }
                | Event::ItemCompleted {
                    body: Item::Usage { usage },
                    ..
                }
                | Event::ItemSnapshot {
                    partial: Item::Usage { usage },
                    ..
                } => Some(usage),
                _ => None,
            };
            if let Some(usage) = usage
                && let Err(detail) = check_usage(usage)
            {
                return Err(LawViolation {
                    line: i + 1,
                    law: "usage-provenance",
                    detail: detail.into(),
                });
            }
            if let Event::ItemCompleted {
                body:
                    Item::Reasoning {
                        provider_opaque: Some(value),
                        ..
                    },
                ..
            } = &e.event
                && !(value["provider"].is_string()
                    && value["form"].is_string()
                    && value.get("data").is_some())
            {
                return Err(LawViolation {
                    line: i + 1,
                    law: "sealed-reasoning",
                    detail: "opaque reasoning needs provider, form and data".into(),
                });
            }
        }
        if event_unknown(&e.event) {
            return Err(LawViolation {
                line: i + 1,
                law: "authorship",
                detail: "writer emitted an Unknown variant".into(),
            });
        }
        if e.v.major == 1 && e.v.minor >= 3 {
            check_dream_structure(&e.event).map_err(|detail| LawViolation {
                line: i + 1,
                law: "dream-structure",
                detail: detail.into(),
            })?;
        }
        if e.seq.is_some() != e.event.is_durable() {
            return Err(LawViolation {
                line: i + 1,
                law: "log-is-truth",
                detail: "authored durability does not match event".into(),
            });
        }
    }
    Ok(())
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

fn check_usage(usage: &Usage) -> Result<(), &'static str> {
    if usage.scope.is_none() || usage.backend.is_none() || usage.source.is_none() {
        return Err("usage needs scope, backend and source");
    }
    if usage.backend.as_ref().is_some_and(|b| b.name.is_empty()) {
        return Err("usage backend name is empty");
    }
    let mut named = std::collections::BTreeSet::new();
    for key in &usage.unreported {
        let value = match key.as_str() {
            "input_tokens" => usage.input_tokens,
            "output_tokens" => usage.output_tokens,
            _ => return Err("unreported only names required counters"),
        };
        if value != 0 || !named.insert(key) {
            return Err("an unreported required counter must be a unique zero placeholder");
        }
    }
    if let Some(cost) = &usage.cost {
        if cost.currency.len() != 3 || !cost.currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err("cost currency must be ISO 4217");
        }
        if cost.currency == "USD" && usage.cost_micros_usd != Some(cost.amount_micros) {
            return Err("USD cost must retain the legacy mirror");
        }
        if usage
            .backend
            .as_ref()
            .is_some_and(|b| b.kind == BackendKind::Engine)
            && cost.price_source.as_ref().is_none_or(String::is_empty)
        {
            return Err("engine cost needs price_source");
        }
    }
    Ok(())
}

// Presence/default checks only. Resolving hashes, commits, publication timelines,
// qualifying trials and per-harness projections belongs to the dream auditor.
fn check_dream_structure(event: &Event) -> Result<(), &'static str> {
    fn pin(pin: &RecordPin, committed: bool) -> bool {
        !pin.root.is_empty()
            && !pin.path.is_empty()
            && !pin.digest.is_empty()
            && (!committed || pin.commit.as_ref().is_some_and(|s| !s.is_empty()))
    }
    fn evidence(reference: &EvidenceRef) -> bool {
        match reference {
            EvidenceRef::Event {
                session,
                seq,
                digest,
            } => *session != empty_session() && seq.0 != 0 && !digest.is_empty(),
            EvidenceRef::Record {
                root,
                id,
                path,
                commit,
                digest,
            } => [root, id, path, commit, digest]
                .iter()
                .all(|s| !s.is_empty()),
            EvidenceRef::Unknown(_) => false,
        }
    }
    if let Event::SessionStarted { meta } = event
        && let Some(dream) = &meta.dream
    {
        let trigger = match &dream.trigger {
            Trigger::SessionClosed { session } => *session != empty_session(),
            Trigger::Quota {
                provider,
                resets_at,
            } => !provider.is_empty() && *resets_at != 0,
            Trigger::Operator { .. } => true,
            Trigger::Dream { entry } => !entry.is_empty(),
            Trigger::Schedule { name } => !name.is_empty(),
            Trigger::Unknown(_) => false,
        };
        if !trigger
            || dream.scope.repos.is_empty()
            || dream.scope.repos.iter().any(String::is_empty)
        {
            return Err("dream metadata has missing trigger or scope fields");
        }
    }
    let Event::ItemCompleted {
        body,
        status: ItemStatus::Completed,
        ..
    } = event
    else {
        return Ok(());
    };
    let valid = match body {
        Item::MemoryWrite {
            entry,
            op,
            record,
            evidence: refs,
            draft,
        } => {
            !entry.is_empty()
                && pin(record, *op != MemoryOp::Pending)
                && !refs.is_empty()
                && refs.iter().all(evidence)
                && (*op != MemoryOp::Pending || draft.is_some())
        }
        Item::MemoryState {
            entry,
            state,
            harness,
            refs,
            ..
        } => {
            let per_harness = matches!(
                state,
                EntryState::Tested
                    | EntryState::Adopted
                    | EntryState::Rejected
                    | EntryState::Retired
            );
            !entry.is_empty()
                && refs.iter().all(evidence)
                && if per_harness {
                    harness.as_ref().is_some_and(|h| !h.is_empty())
                } else {
                    harness.is_none()
                }
        }
        Item::Trial {
            entry,
            harness,
            thresholds,
            episodes,
            controls,
            ..
        } => {
            for episode in episodes.iter().chain(controls) {
                if !evidence(&episode.origin)
                    || episode.runs.iter().any(|r| r.session == empty_session())
                {
                    return Err("trial episode has missing source or run identity");
                }
                for run in &episode.runs {
                    if let Some(usage) = &run.usage {
                        check_usage(usage)?;
                    }
                }
            }
            !entry.is_empty() && !harness.is_empty() && !thresholds.is_empty()
        }
        Item::Adoption {
            entry,
            harness,
            sink,
            block_digest,
            record,
        } => {
            !entry.is_empty()
                && !harness.is_empty()
                && !sink.path.is_empty()
                && !block_digest.is_empty()
                && record.as_ref().is_some_and(|p| pin(p, true))
        }
        Item::Recall {
            entry,
            sink,
            basis,
            record,
            block_digest,
        } => {
            let pinned = matches!(basis, RecallBasis::Reported | RecallBasis::Cited);
            !entry.is_empty()
                && sink.as_ref().is_none_or(|s| !s.path.is_empty())
                && record.as_ref().is_none_or(|p| pin(p, pinned))
                && block_digest.as_ref().is_none_or(|d| !d.is_empty())
                && (!pinned || (record.is_some() && block_digest.is_some()))
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err("dream item has missing/default identity, revision pin, draft or harness fields")
    }
}
