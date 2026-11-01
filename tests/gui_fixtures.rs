// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

use bboxy450_harness::{
    frontends::gui::{
        bridge::read_replay,
        model::{Effect, Transcript, TurnState},
    },
    protocol::*,
};
use proptest::prelude::*;
use std::path::Path;
fn fixture(name: &str) -> Vec<Envelope> {
    std::fs::read_to_string(Path::new("tests/fixtures/protocol").join(name))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn started(item: ItemId, kind: ItemKind) -> Event {
    let head = match kind {
        ItemKind::ToolCall => {
            serde_json::json!({"kind":"tool_call","call_id":item.to_string(),"name":"fixture","tool_kind":"other"})
        }
        ItemKind::Terminal => {
            serde_json::json!({"kind":"terminal","terminal":item.to_string(),"command":"fixture"})
        }
        _ => serde_json::Value::Null,
    };
    serde_json::from_value(serde_json::json!({"type":"item_started","item":item,"turn":null,"parent":null,"kind":kind,"head":head})).unwrap()
}
fn envelope(event: Event, seq: Option<u64>, epoch: u32, at: u64) -> Envelope {
    static BASE: std::sync::OnceLock<Envelope> = std::sync::OnceLock::new();
    let mut e = BASE
        .get_or_init(|| fixture("all-items.jsonl").remove(0))
        .clone();
    e.event = event;
    e.seq = seq.map(Seq);
    e.epoch = Epoch(epoch);
    e.at = Seq(at);
    e
}
#[tokio::test]
async fn every_fixture_passes_laws_and_projects() {
    let mut count = 0;
    for entry in std::fs::read_dir("tests/fixtures/protocol").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|v| v != "jsonl") {
            continue;
        }
        let events: Vec<Envelope> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        bboxy450_harness::protocol::laws::check_transcript(&events).unwrap();
        let model = read_replay(path.clone()).await.unwrap();
        assert!(!model.items.is_empty(), "{}", path.display());
        assert_eq!(model.duplicate_durable, 0);
        assert!(model.items.values().all(|v| v.status.is_some()));
        assert!(
            model
                .turns
                .values()
                .all(|v| matches!(v, TurnState::Ended { .. }))
        );
        count += 1;
    }
    assert_eq!(count, 5);
}
#[test]
fn completion_wins_and_durable_replay_cannot_reopen_or_duplicate() {
    let events = fixture("all-items.jsonl");
    let mut model = Transcript::default();
    for e in &events {
        model.apply(e);
    }
    let snapshot = model.items.clone();
    let order = model.order.clone();
    for e in &events {
        model.apply(e);
    }
    assert_eq!(model.items, snapshot);
    assert_eq!(model.order, order);
    assert_eq!(
        model.duplicate_durable,
        events.iter().filter(|e| e.seq.is_some()).count() as u64
    );
}
#[test]
fn snapshots_replace_ordered_tail_late_chunks_and_aba_epochs_are_retired() {
    let item = ItemId::new();
    let mut m = Transcript::default();
    m.apply(&envelope(
        started(item, ItemKind::AgentMessage),
        Some(1),
        1,
        1,
    ));
    let delta = envelope(
        Event::ItemDelta {
            item,
            delta: Delta::Text {
                append: "ha".into(),
            },
        },
        None,
        1,
        1,
    );
    m.apply(&delta);
    m.apply(&delta); // identical text is two legitimate chunks
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: "haha".into()
        }
    );
    m.apply(&envelope(
        Event::ItemSnapshot {
            item,
            partial: Item::AgentMessage {
                markdown: "snapshot".into(),
            },
        },
        None,
        1,
        1,
    ));
    m.apply(&delta);
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: "snapshotha".into()
        }
    );
    m.apply(&envelope(
        Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: Item::AgentMessage {
                markdown: "final".into(),
            },
        },
        Some(2),
        1,
        2,
    ));
    m.apply(&delta);
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: "final".into()
        }
    );
    assert_eq!(
        m.apply(&envelope(
            Event::Lagged {
                resume_after: Seq(2)
            },
            None,
            1,
            2
        )),
        Effect::Resubscribe {
            after: Some((Epoch(1), Seq(2)))
        }
    );
    assert_eq!(
        m.apply(&envelope(
            Event::Reset {
                epoch: Epoch(2),
                head: Seq(1)
            },
            None,
            2,
            2
        )),
        Effect::Resubscribe { after: None }
    );
    assert!(m.items.is_empty());
    assert_eq!(m.cursor, Seq(0));
    // An old A stream cannot deliver into a new A view after A -> B -> A.
    m.apply(&envelope(
        started(item, ItemKind::AgentMessage),
        Some(1),
        2,
        1,
    ));
    m.apply(&delta);
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: String::new()
        }
    );
    let mut current = delta.clone();
    current.epoch = Epoch(2);
    current.at = Seq(0);
    m.apply(&current);
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: String::new()
        }
    );
    current.at = Seq(1);
    m.apply(&current);
    assert_eq!(
        m.items[&item].body,
        Item::AgentMessage {
            markdown: "ha".into()
        }
    );
    m.apply(&envelope(
        Event::Status {
            phase: Phase::Thinking,
        },
        None,
        2,
        1,
    ));
    assert_eq!(m.phase, Phase::Thinking);
}
proptest! {
    #![proptest_config(ProptestConfig { cases: 128, failure_persistence: None, ..Default::default() })]
    #[test]
    fn live_equals_replay(chunks in prop::collection::vec("[^\\p{C}]{0,32}", 1..80), choices in prop::collection::vec(any::<u8>(), 1..80), kind in 0u8..5) {
        let item = ItemId::new();
        let k = match kind { 0 => ItemKind::AgentMessage, 1 => ItemKind::Reasoning, 2 | 3 => ItemKind::ToolCall, _ => ItemKind::Terminal };
        let start = envelope(started(item, k), Some(1), 1, 1);
        let mut full = Transcript::default(); let mut lossy = Transcript::default(); full.apply(&start); lossy.apply(&start);
        let delta = |append: String| match kind { 0 => Delta::Text { append }, 1 => Delta::Reasoning { append }, 2 => Delta::ToolInput { append }, _ => Delta::Output { append, stream: Stream::Stdout } };
        let mut batch = String::new(); let mut expected_partial = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            full.apply(&envelope(Event::ItemDelta { item, delta: delta(chunk.clone()) }, None, 1, 1));
            let choice = choices[i % choices.len()];
            if choice & 1 != 0 { batch.push_str(chunk); expected_partial.push_str(chunk); }
            if choice & 2 != 0 || i + 1 == chunks.len() {
                lossy.apply(&envelope(Event::ItemDelta { item, delta: delta(std::mem::take(&mut batch)) }, None, 1, 1));
            }
        }
        let row = &lossy.items[&item];
        let actual_partial = match &row.body {
            Item::AgentMessage { markdown } => markdown.clone(), Item::Reasoning { summary, .. } => summary.clone(),
            Item::ToolCall { output, .. } if kind == 3 => output.iter().filter_map(|b| if let OutputBlock::Text { text } = b { Some(text.as_str()) } else { None }).collect::<String>(),
            Item::ToolCall { input_raw, .. } => input_raw.clone(), _ => row.terminal_tail.clone(),
        };
        prop_assert_eq!(actual_partial, expected_partial);
        let body = full.items[&item].body.clone();
        let complete = envelope(Event::ItemCompleted { item, status: ItemStatus::Completed, body }, Some(2), 1, 2);
        full.apply(&complete); lossy.apply(&complete);
        prop_assert_eq!(&full, &lossy);
    }
}
proptest! {
    #![proptest_config(ProptestConfig { cases: 128, failure_persistence: None, ..Default::default() })]
    #[test]
    fn snapshot_repairs_dropped_and_coalesced_deltas_before_completion(chunks in prop::collection::vec("[a-z]{1,32}", 3..40), choices in prop::collection::vec(any::<u8>(), 1..40), kind in 0u8..4) {
        let item = ItemId::new();
        let k = match kind { 0 => ItemKind::AgentMessage, 1 => ItemKind::Reasoning, _ => ItemKind::ToolCall };
        let start = envelope(started(item, k), Some(1), 1, 1);
        let mut full = Transcript::default(); let mut lossy = Transcript::default(); full.apply(&start); lossy.apply(&start);
        let delta = |append: String| match kind { 0 => Delta::Text { append }, 1 => Delta::Reasoning { append }, 2 => Delta::ToolInput { append }, _ => Delta::Output { append, stream: Stream::Stdout } };
        let mid = chunks.len()/2;
        let mut batch = String::new();
        for (i, chunk) in chunks[..mid].iter().enumerate() {
            full.apply(&envelope(Event::ItemDelta { item, delta: delta(chunk.clone()) }, None, 1, 1));
            // Always drop the first nonempty chunk; randomly drop/coalesce others.
            if i > 0 && choices[i % choices.len()] & 1 != 0 { batch.push_str(chunk); }
        }
        lossy.apply(&envelope(Event::ItemDelta { item, delta: delta(batch) }, None, 1, 1));
        prop_assert_ne!(&full.items, &lossy.items, "broken control: loss without a snapshot must differ");
        let snapshot = envelope(Event::ItemSnapshot { item, partial: full.items[&item].body.clone() }, None, 1, 1);
        full.apply(&snapshot); lossy.apply(&snapshot);
        prop_assert_eq!(&full, &lossy);
        prop_assert!(lossy.items[&item].completed.is_none());
        for chunk in &chunks[mid..] {
            let e = envelope(Event::ItemDelta { item, delta: delta(chunk.clone()) }, None, 1, 1);
            full.apply(&e); lossy.apply(&e);
            prop_assert_eq!(&full, &lossy);
            prop_assert!(lossy.items[&item].completed.is_none());
        }
    }
}
#[test]
fn stick_and_release() {
    let mut scroll = bboxy450_harness::frontends::gui::chat_scroll::State::default();
    assert_eq!(scroll.layout_offset().min(200.), 200.);
    assert!(scroll.wheel(200., 200., -25.));
    assert!(scroll.wheel(200., 200., -25.));
    assert_eq!(scroll.layout_offset(), 150.);
    scroll.layout_solved();
    assert_eq!(scroll.layout_offset().clamp(0., 300.), 150.);
    assert!(!scroll.wheel(150., 300., f32::NAN));
    scroll.follow_latest();
    assert!(scroll.following());
    assert_eq!(scroll.layout_offset().min(400.), 400.);
}

#[tokio::test]
async fn v0_2_heads_and_terminal_snapshot_repair() {
    let raw: Vec<serde_json::Value> = include_str!("fixtures/gui/v0_2-identity-terminal.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let typed: Vec<Envelope> = raw
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    bboxy450_harness::protocol::laws::check_transcript(&typed).unwrap();
    let call = ItemId("00000000000000000000000071".parse().unwrap());
    let terminal = ItemId("00000000000000000000000072".parse().unwrap());
    let mut model = Transcript::default();
    for (i, value) in raw.iter().enumerate().take(9) {
        // Drop the terminal's first delta: the snapshot must repair it.
        if i != 6 {
            model.apply_wire(value).unwrap();
        }
        if i == 3 {
            assert!(
                matches!(&model.items[&call].body, Item::ToolCall { call_id, name, tool_kind: ToolKind::Execute, .. } if call_id == "exec-7" && name == "bash")
            );
        }
        if i == 5 {
            assert!(
                matches!(&model.items[&terminal].body, Item::Terminal { terminal, command, .. } if terminal == "term-7" && command == "printf hello")
            );
        }
    }
    assert_eq!(model.items[&terminal].terminal_tail, "hello!");
    // Bound and Unicode safety of a live tail, without inventing a blob fetch.
    let mut delta = raw[8].clone();
    delta["event"]["delta"]["append"] = serde_json::json!("🦊".repeat(20_000));
    model.apply_wire(&delta).unwrap();
    let tail = &model.items[&terminal].terminal_tail;
    assert!(tail.starts_with('…'));
    assert!(tail.len() <= 64 * 1024);
    assert!(tail.ends_with('🦊'));
    for value in &raw[9..] {
        model.apply_wire(value).unwrap();
    }
    assert!(model.items[&terminal].terminal_tail.is_empty());
    let replay = read_replay("tests/fixtures/gui/v0_2-identity-terminal.jsonl".into())
        .await
        .unwrap();
    assert_eq!(replay.items, model.items);
}
