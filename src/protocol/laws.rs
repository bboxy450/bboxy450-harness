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
    for (i, e) in events.iter().enumerate() {
        let fail = |law, detail: &str| LawViolation {
            line: i + 1,
            law,
            detail: detail.into(),
        };
        if e.v.major != PROTOCOL.major {
            return Err(fail("version", "unsupported major"));
        }
        if let Some(id) = session {
            if id != e.session {
                return Err(fail("log-is-truth", "session changed"));
            }
        } else {
            session = Some(e.session);
        }
        if e.epoch.0 < epoch {
            return Err(fail("log-is-truth", "epoch decreased"));
        }
        epoch = e.epoch.0;
        if e.seq.is_some() != e.event.is_durable() && !matches!(e.event, Event::Unknown) {
            return Err(fail("log-is-truth", "durability does not match event"));
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
            Event::TurnStarted { turn } => {
                if !queued.contains(turn) || ended.contains(turn) || !started.insert(*turn) {
                    return Err(fail("one-ending", "invalid turn start"));
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
                if e.v.minor >= 2 {
                    let valid = matches!(head, Some(ItemHead::Unknown))
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
                open.insert(*item, (*turn, *kind, *parent, head.clone()));
            }
            Event::ItemCompleted { item, body, .. } => {
                let Some((_, kind, _, head)) = open.remove(item) else {
                    return Err(fail("one-ending", "orphan or duplicate item completion"));
                };
                if body.kind() != kind
                    && kind != ItemKind::Unknown
                    && body.kind() != ItemKind::Unknown
                {
                    return Err(fail("one-ending", "item kind changed"));
                }
                if open
                    .values()
                    .any(|(_, _, parent, _)| parent.as_ref() == Some(item))
                {
                    return Err(fail("one-ending", "parent completed before child"));
                }
                if head.is_some()
                    && head != Some(ItemHead::Unknown)
                    && body.kind() != ItemKind::Unknown
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
