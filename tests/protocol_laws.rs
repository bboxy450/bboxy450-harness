// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::{laws::check_transcript, *};
fn fixture() -> Vec<Envelope> {
    include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
fn expect_violation(events: &[Envelope], law: &str, detail: &str) {
    let violation = check_transcript(events).unwrap_err();
    assert_eq!((violation.law, violation.detail.as_str()), (law, detail));
}
#[test]
fn rejects_double_end_gap_unpaired_and_kind_change() {
    let good = fixture();
    check_transcript(&good).unwrap();
    let mut bad = good.clone();
    let mut e = bad.last().unwrap().clone();
    e.seq = Some(Seq(e.seq.unwrap().0 + 1));
    e.at = e.seq.unwrap();
    bad.push(e);
    expect_violation(&bad, "one-ending", "orphan or duplicate ending");
    let mut bad = good.clone();
    bad.remove(1);
    expect_violation(&bad, "log-is-truth", "durable gap or duplicate");
    let mut bad = good.clone();
    let tool = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::ToolCall { .. },
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemCompleted {
        body: Item::ToolCall { output, .. },
        ..
    } = &mut tool.event
    {
        output.clear();
    }
    expect_violation(&bad, "tool-pairing", "duplicate call id or absent result");
    let mut bad = good.clone();
    let e = bad
        .iter_mut()
        .find(|e| matches!(e.event, Event::ItemCompleted { .. }))
        .unwrap();
    if let Event::ItemCompleted { body, .. } = &mut e.event {
        *body = Item::Plan { entries: vec![] };
    }
    expect_violation(&bad, "one-ending", "item kind changed");
}

#[test]
fn durable_identity_and_live_completion_rules_are_specific() {
    let good = fixture();
    let mut bad = good.clone();
    let start = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::ToolCall,
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemStarted { head, .. } = &mut start.event {
        *head = None;
    }
    expect_violation(&bad, "item-identity", "missing or mismatched durable head");
    let mut bad = good.clone();
    let start = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::ToolCall,
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemStarted { head, .. } = &mut start.event {
        *head = Some(ItemHead::Terminal {
            terminal: "wrong-kind".into(),
            command: "".into(),
        });
    }
    expect_violation(&bad, "item-identity", "missing or mismatched durable head");
    let mut bad = good.clone();
    let complete = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::ToolCall { .. },
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemCompleted {
        body: Item::ToolCall { call_id, .. },
        ..
    } = &mut complete.event
    {
        *call_id = "changed-identity".into();
    }
    expect_violation(
        &bad,
        "item-identity",
        "body identity differs from durable head",
    );
    let mut bad = good;
    let complete = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::Terminal { .. },
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemCompleted {
        body: Item::Terminal { live, .. },
        ..
    } = &mut complete.event
    {
        *live = Some("illegal completed tail".into());
    }
    expect_violation(
        &bad,
        "item-identity",
        "completed terminal contains live output",
    );
}

#[test]
fn queued_cancellation_does_not_authorize_reducing_a_running_turn() {
    let mut events = fixture();
    let at = events
        .iter()
        .position(|e| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::ContextTrim,
                    ..
                }
            )
        })
        .unwrap();
    let turn = TurnId::new();
    let mut queued = events[at].clone();
    queued.event = Event::TurnQueued { turn };
    let mut ended = events
        .iter()
        .find(|e| matches!(e.event, Event::TurnEnded { .. }))
        .unwrap()
        .clone();
    if let Event::TurnEnded {
        turn: owner,
        reason,
        ..
    } = &mut ended.event
    {
        *owner = turn;
        *reason = StopReason::Cancelled;
    }
    events.splice(at..at, [queued, ended]);
    let mut head = 0;
    for event in &mut events {
        if event.seq.is_some() {
            head += 1;
            event.seq = Some(Seq(head));
        }
        event.at = Seq(head);
    }
    check_transcript(&events).unwrap();
    let cancelled_at = events[at + 1].seq.unwrap();
    let trim = events
        .iter_mut()
        .find_map(|e| match &mut e.event {
            Event::ItemCompleted {
                body: Item::ContextTrim { clipped_through },
                ..
            } => Some(clipped_through),
            _ => None,
        })
        .unwrap();
    *trim = cancelled_at;
    expect_violation(
        &events,
        "context-boundary",
        "reduction exceeds completed frontier",
    );
}
