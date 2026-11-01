// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        budget::{self, Clock, Work},
        model::Transcript,
        view::Preparer,
    },
    protocol::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
#[derive(Default)]
struct Wake(AtomicUsize);
impl bbt_rt::Waker for Wake {
    fn wake(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
#[test]
fn budgeted_remainder_keeps_its_wake() {
    let wake = Arc::new(Wake::default());
    let dirty = bbt_rt::DirtySet::new(1, wake.clone());
    let (sender, receiver) = bbt_rt::ui_channel(4, dirty.paint_source(0));
    for _ in 0..4 {
        sender
            .try_send(bbt_rt::UiMsg::Changed { source: 0 })
            .unwrap();
    }
    assert_eq!(dirty.drain(), vec![0]);
    receiver.try_recv().unwrap();
    receiver.complete_drain();
    assert_eq!(dirty.drain(), vec![0]);
    assert_eq!(wake.0.load(Ordering::Relaxed), 2);
    for _ in 0..3 {
        receiver.try_recv().unwrap();
    }
    receiver.complete_drain();
    assert!(receiver.is_empty());
    dirty.drain();
    assert!(dirty.drain().is_empty());
}

#[derive(Default)]
struct InjectedClock(std::cell::Cell<Duration>);
impl Clock for InjectedClock {
    fn now(&self) -> Duration {
        self.0.get()
    }
    fn charge(&self, _work: Work) {
        self.0.set(self.0.get() + Duration::from_micros(50));
    }
}
fn fixture() -> (std::collections::VecDeque<Envelope>, ItemId) {
    let base: Envelope = serde_json::from_str(
        include_str!("fixtures/protocol/all-items.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let completed = ItemId::new();
    let item = ItemId::new();
    let mut queue = std::collections::VecDeque::from([base.clone()]);
    let mut push = |event: serde_json::Value, seq: Option<u64>, at: u64| {
        let mut e = base.clone();
        e.event = serde_json::from_value(event).unwrap();
        e.seq = seq.map(Seq);
        e.at = Seq(at);
        queue.push_back(e);
    };
    push(
        serde_json::json!({"type":"item_started","item":completed,"turn":null,"parent":null,"kind":"user_message","head":null}),
        Some(2),
        2,
    );
    push(
        serde_json::json!({"type":"item_completed","item":completed,"status":{"status":"completed"},"body":{"kind":"user_message","blocks":[{"type":"text","text":"retained history"}]}}),
        Some(3),
        3,
    );
    push(
        serde_json::json!({"type":"item_started","item":item,"turn":null,"parent":null,"kind":"agent_message","head":null}),
        Some(4),
        4,
    );
    let mut delta = base.clone();
    delta.seq = None;
    delta.at = Seq(4);
    delta.event = Event::ItemDelta {
        item,
        delta: Delta::Text { append: "x".into() },
    };
    for _ in 0..100_000 {
        queue.push_back(delta.clone());
    }
    let mut complete = base;
    complete.seq = Some(Seq(5));
    complete.at = Seq(5);
    complete.event = Event::ItemCompleted {
        item,
        status: ItemStatus::Completed,
        body: Item::AgentMessage {
            markdown: "x".repeat(100_000),
        },
    };
    queue.push_back(complete);
    (queue, item)
}
#[test]
fn injected_clock_stops_at_300us_defers_and_loses_no_envelope() {
    let (mut queue, item) = fixture();
    let expected = queue.len();
    let clock = InjectedClock::default();
    let mut model = Transcript::default();
    let mut applied = 0;
    let mut frames = 0;
    while !queue.is_empty() {
        let before = queue.len();
        let drained = budget::drain(
            &clock,
            Duration::from_micros(300),
            || queue.pop_front(),
            |e| {
                model.apply(&e);
                applied += 1;
            },
        );
        assert!(drained.spent <= Duration::from_micros(300));
        assert_eq!(before - queue.len(), drained.received);
        if !queue.is_empty() {
            assert_eq!(drained.received, 3);
            assert_eq!(drained.spent, Duration::from_micros(300));
        }
        frames += 1;
    }
    assert_eq!(applied, expected);
    assert_eq!(model.duplicate_durable, 0);
    assert_eq!(model.cursor, Seq(5));
    assert_eq!(
        model.items[&item].body,
        Item::AgentMessage {
            markdown: "x".repeat(100_000)
        }
    );
    println!(
        "gui-budget: 100000 deltas, {frames} deterministic frames, 300us limit, {applied} envelopes applied exactly once, zero loss"
    );
}
#[tokio::test]
async fn completed_layouts_are_retained_and_large_blocks_prepare_off_ui() {
    bbt_rt::register_ui_thread();
    let ui_thread = std::thread::current().id();
    let (queue, _) = fixture();
    let mut model = Transcript::default();
    for e in queue {
        model.apply(&e);
    }
    let source = Arc::new(model);
    let (large, on_ui, first, second, thread) = bboxy450_harness::runtime::blocking(move || {
        let mut preparer = Preparer::default();
        let first = preparer.prepare(source.clone(), 1120.).unwrap();
        let mut next = (*source).clone();
        next.phase = Phase::Thinking;
        let second = preparer.prepare(Arc::new(next), 1120.).unwrap();
        (
            first.large_markdown_blocks,
            first.prepared_on_ui,
            first.layout_builds,
            second.layout_builds,
            std::thread::current().id(),
        )
    })
    .await;
    assert_eq!(first, 2);
    assert_eq!(second, first, "unchanged completed items never rebuild");
    assert_eq!(large, 1);
    assert!(!on_ui);
    assert_ne!(thread, ui_thread);
}
