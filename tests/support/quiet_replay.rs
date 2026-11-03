// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Synthetic raw ACP chunks interleaved with a real, frozen replay. No provider or login.
use bboxy450_harness::protocol::*;

pub fn resequence(events: &mut [Envelope]) {
    let mut head = 0;
    for e in events {
        e.raw = None;
        e.seq = if e.event.is_durable() {
            head += 1;
            Some(Seq(head))
        } else {
            None
        };
        e.at = Seq(head);
    }
}

pub fn replay(chunks: usize) -> Vec<Envelope> {
    let mut original: Vec<Envelope> = include_str!("../fixtures/parity/acp-claude-acp.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    // Two adjacent visible calls must keep their shared rail even with raw chunks
    // between them (origin/main 55c0080). Keep the frozen ACP fixture unchanged.
    let at = original
        .iter()
        .position(|e| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::ToolCall,
                    ..
                }
            )
        })
        .unwrap();
    let Event::ItemStarted { turn, .. } = original[at].event else {
        unreachable!()
    };
    let item = ItemId(ulid::Ulid::from(1u128 << 116));
    let body = Item::ToolCall {
        call_id: "quiet-parallel-read".into(),
        name: "read".into(),
        tool_kind: ToolKind::Read,
        input_raw: "{\"path\":\"note.txt\"}".into(),
        output: vec![OutputBlock::Text {
            text: "fixture contents".into(),
        }],
        locations: vec![],
        spill: None,
        title: None,
        server: None,
        raw_output: None,
    };
    let mut start = original[at].clone();
    start.event = Event::ItemStarted {
        turn,
        item,
        kind: ItemKind::ToolCall,
        parent: None,
        head: body.head(),
    };
    let mut complete = start.clone();
    complete.event = Event::ItemCompleted {
        item,
        status: ItemStatus::Completed,
        body,
    };
    original.splice(at..at, [start, complete]);
    let slots = original.len() - 1;
    let mut events = vec![];
    let mut turn = None;
    let mut next = 0u128;
    for (i, e) in original.into_iter().enumerate() {
        match e.event {
            Event::TurnStarted { turn: t, .. } => turn = Some(t),
            Event::TurnEnded { .. } => turn = None,
            _ => {}
        }
        events.push(e.clone());
        if i == slots {
            continue;
        }
        let count = chunks / slots + usize::from(i < chunks % slots);
        for _ in 0..count {
            let item = ItemId(ulid::Ulid::from((1u128 << 120) + next));
            next += 1;
            for event in [
                Event::ItemStarted {
                    turn,
                    item,
                    kind: ItemKind::Native,
                    parent: None,
                    head: None,
                },
                Event::ItemDelta {
                    item,
                    delta: Delta::Text {
                        append: "RAW_CHUNK_MUST_STAY_IN_LOG".into(),
                    },
                },
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::Native {
                        provider: "acp:claude-agent-acp".into(),
                        record_type: "agent_message_chunk".into(),
                        raw: serde_json::json!({"text":"RAW_CHUNK_MUST_STAY_IN_LOG"}),
                        blob: None,
                    },
                },
            ] {
                let mut native = e.clone();
                // Edge metadata timestamps must not change a visible receipt's duration.
                native.ts_ms += 90_000;
                native.event = event;
                events.push(native);
            }
        }
    }
    resequence(&mut events);
    laws::check_transcript(&events).unwrap();
    events
}

pub fn without_native(events: &[Envelope]) -> Vec<Envelope> {
    let native: std::collections::HashSet<_> = events
        .iter()
        .filter_map(|e| match e.event {
            Event::ItemStarted {
                item,
                kind: ItemKind::Native,
                ..
            } => Some(item),
            _ => None,
        })
        .collect();
    let mut events: Vec<_> = events
        .iter()
        .filter(|e| match e.event {
            Event::ItemStarted { item, .. }
            | Event::ItemDelta { item, .. }
            | Event::ItemSnapshot { item, .. }
            | Event::ItemCompleted { item, .. } => !native.contains(&item),
            _ => true,
        })
        .cloned()
        .collect();
    resequence(&mut events);
    laws::check_transcript(&events).unwrap();
    events
}

pub fn notices() -> Vec<Envelope> {
    let mut events = replay(0);
    events.truncate(1);
    let base = events[0].clone();
    for (i, body) in [
        Item::Command {
            name: "help".into(),
            args: "ARGUMENTS_STAY_IN_LOG".into(),
        },
        Item::Hook {
            event: "pre_tool".into(),
            name: "fixture-hook".into(),
            outcome: "passed".into(),
            output: Some("HOOK_OUTPUT_STAYS_IN_LOG".into()),
        },
        Item::SystemMessage {
            role: SystemRole::Developer,
            text: "Session instructions".into(),
        },
        Item::Compaction {
            summary: BlobRef {
                sha256: "0".repeat(64),
                len: 0,
                mime: "text/plain".into(),
            },
            replaced_through: Seq(0),
        },
        Item::Unknown(
            serde_json::json!({"kind":"future_renderer", "raw":"RAW_PAYLOAD_STAYS_IN_LOG"}),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let item = ItemId(ulid::Ulid::from(9000 + i as u128));
        for event in [
            Event::ItemStarted {
                turn: None,
                item,
                kind: body.kind(),
                parent: None,
                head: None,
            },
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body,
            },
        ] {
            let mut e = base.clone();
            e.event = event;
            events.push(e);
        }
    }
    resequence(&mut events);
    laws::check_transcript(&events).unwrap();
    events
}
