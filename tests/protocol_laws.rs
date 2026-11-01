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
#[test]
fn rejects_double_end_gap_unpaired_and_kind_change() {
    let good = fixture();
    check_transcript(&good).unwrap();
    let mut bad = good.clone();
    let mut e = bad.last().unwrap().clone();
    e.seq = Some(Seq(e.seq.unwrap().0 + 1));
    e.at = e.seq.unwrap();
    bad.push(e);
    assert_eq!(check_transcript(&bad).unwrap_err().law, "one-ending");
    let mut bad = good.clone();
    bad.remove(1);
    assert_eq!(check_transcript(&bad).unwrap_err().law, "log-is-truth");
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
    assert_eq!(check_transcript(&bad).unwrap_err().law, "tool-pairing");
    let mut bad = good.clone();
    let e = bad
        .iter_mut()
        .find(|e| matches!(e.event, Event::ItemCompleted { .. }))
        .unwrap();
    if let Event::ItemCompleted { body, .. } = &mut e.event {
        *body = Item::Plan { entries: vec![] };
    }
    assert_eq!(check_transcript(&bad).unwrap_err().law, "one-ending");
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
    assert_eq!(check_transcript(&bad).unwrap_err().law, "item-identity");
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
    assert_eq!(check_transcript(&bad).unwrap_err().law, "item-identity");
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
    assert_eq!(check_transcript(&bad).unwrap_err().law, "item-identity");
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
    assert_eq!(check_transcript(&bad).unwrap_err().law, "item-identity");
}
