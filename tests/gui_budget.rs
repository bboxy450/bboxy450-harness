// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        bridge::Bridge,
        budget::{Clock, Work},
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
    fixture_with_deltas(100_000)
}
fn fixture_with_deltas(count: usize) -> (std::collections::VecDeque<Envelope>, ItemId) {
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
    for _ in 0..count {
        queue.push_back(delta.clone());
    }
    let mut complete = base;
    complete.seq = Some(Seq(5));
    complete.at = Seq(5);
    complete.event = Event::ItemCompleted {
        item,
        status: ItemStatus::Completed,
        body: Item::AgentMessage {
            markdown: "x".repeat(count),
        },
    };
    queue.push_back(complete);
    (queue, item)
}
#[tokio::test]
async fn injected_clock_stops_at_300us_defers_and_loses_no_envelope() {
    let (queue, item) = fixture();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("replay.jsonl");
    let mut data = Vec::new();
    for event in &queue {
        serde_json::to_writer(&mut data, event).unwrap();
        data.push(b'\n');
    }
    std::fs::write(&path, data).unwrap();
    let mut bridge = Bridge::new(Arc::new(Wake::default()));
    bridge.replay(path);
    async fn publication(bridge: &Bridge, after: u64) {
        tokio::time::timeout(Duration::from_secs(30), async {
            while bridge.published_revision() <= after {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
    }
    publication(&bridge, 0).await;
    // Exercise real worker publications, fill the UI notification queue and
    // leave all draining to Bridge's production path.
    for i in 0..15 {
        let revision = bridge.published_revision();
        bridge.set_width(if i % 2 == 0 { 1000. } else { 1120. });
        publication(&bridge, revision).await;
    }
    let expected_revision = bridge.published_revision();
    let clock = InjectedClock::default();
    assert!(bridge.drain_with_clock(Duration::from_micros(300), &clock));
    assert_eq!(clock.now(), Duration::from_micros(300));
    assert!(
        bridge.pending_notices(),
        "the production drain must defer queued work"
    );
    assert!(
        !bridge.dirty.drain().is_empty(),
        "deferred bridge work must wake the next frame"
    );
    let mut frames = 1;
    while bridge.pending_notices() {
        let before = clock.now();
        bridge.drain_with_clock(Duration::from_micros(300), &clock);
        assert!(clock.now() - before <= Duration::from_micros(300));
        frames += 1;
        assert!(frames <= 8);
    }
    assert_eq!(bridge.state.revision, expected_revision);
    let model = &bridge.state.transcript;
    assert_eq!(model.duplicate_durable, 0);
    assert_eq!(model.cursor, Seq(5));
    assert_eq!(model.items.len(), 2);
    assert_eq!(
        model.items[&item].body,
        Item::AgentMessage {
            markdown: "x".repeat(100_000)
        }
    );
    bridge.detach().unwrap().await.unwrap();
    println!(
        "gui-budget: real Bridge, 100000 deltas, {frames} injected-clock frames, 300us limit, deferred notifications and zero projection loss"
    );
}
#[tokio::test]
async fn completed_layouts_are_retained() {
    let (queue, _) = fixture();
    let mut model = Transcript::default();
    for e in queue {
        model.apply(&e);
    }
    let source = Arc::new(model);
    let (first, second) = bboxy450_harness::runtime::blocking(move || {
        let mut preparer = Preparer::default();
        let first = preparer.prepare(source.clone(), 1120.).unwrap();
        let mut next = (*source).clone();
        next.phase = Phase::Thinking;
        let second = preparer.prepare(Arc::new(next), 1120.).unwrap();
        (first.layout_builds, second.layout_builds)
    })
    .await;
    assert_eq!(first, 2);
    assert_eq!(second, first, "unchanged completed items never rebuild");
}

/// One shared clock for all four production bridges and native projections.
/// Drain replacements cost 100us; an atomic prepared render costs 200us.
/// These deterministic charges test scheduling, not measured machine timings.
#[derive(Default)]
struct AggregateClock(std::cell::Cell<Duration>);
impl Clock for AggregateClock {
    fn now(&self) -> Duration {
        self.0.get()
    }
    fn charge(&self, work: Work) {
        let cost = match work {
            Work::TryRecv => Duration::ZERO,
            Work::Projection => Duration::from_micros(100),
            Work::Render => Duration::from_micros(200),
        };
        self.0.set(self.0.get() + cost);
    }
}
struct Pane {
    bridge: Bridge,
    view: bboxy450_harness::frontends::gui::view::HarnessView,
    item: ItemId,
    target: u64,
    rendered: u64,
    frames: usize,
}
#[tokio::test]
async fn four_views_share_one_clock_drain_and_projection_without_losing_the_tail() {
    use bboxy450_harness::frontends::gui::{
        budget::round_robin,
        view::{HarnessView, layout_text},
    };
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let mut panes = vec![];
    async fn publication(bridge: &Bridge, after: u64) {
        tokio::time::timeout(Duration::from_secs(30), async {
            while bridge.published_revision() <= after {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
    }
    for n in 0..4 {
        let (queue, item) = fixture_with_deltas(128);
        let path = own.path().join(format!("view-{n}.jsonl"));
        let mut data = Vec::new();
        for event in queue {
            serde_json::to_writer(&mut data, &event).unwrap();
            data.push(b'\n');
        }
        std::fs::write(&path, data).unwrap();
        let mut bridge = Bridge::new(Arc::new(Wake::default()));
        bridge.replay(path);
        publication(&bridge, 0).await;
        for i in 0..15 {
            let revision = bridge.published_revision();
            bridge.set_width(if i % 2 == 0 { 1000. } else { 1120. });
            publication(&bridge, revision).await;
        }
        let target = bridge.published_revision();
        panes.push(Pane {
            bridge,
            view: HarnessView::default(),
            item,
            target,
            rendered: 0,
            frames: 0,
        });
    }
    let clock = AggregateClock::default();
    // Establish four admitted immutable states; retain their notification tails.
    // Setup is outside frame accounting, like a frame following a worker burst.
    for pane in &mut panes {
        pane.bridge
            .drain_with_clock(Duration::from_micros(100), &clock);
        assert_eq!(pane.bridge.state.revision, pane.target);
        assert!(pane.bridge.pending_notices());
    }
    let mut drain_cursor = 0;
    let mut projection_cursor = 0;
    let mut frames = 0;
    loop {
        frames += 1;
        let drain = round_robin(
            &mut panes,
            &mut drain_cursor,
            &clock,
            Duration::from_micros(300),
            |pane, remaining| {
                pane.bridge.drain_with_clock(remaining, &clock);
            },
        );
        assert!(
            drain.spent <= Duration::from_micros(300),
            "aggregate drain exceeded 300us"
        );
        let projection = round_robin(
            &mut panes,
            &mut projection_cursor,
            &clock,
            Duration::from_micros(600),
            |pane, _remaining| {
                if pane.rendered != pane.bridge.state.revision {
                    let frame = pane
                        .view
                        .render(&pane.bridge.state, [1120., 7000.])
                        .unwrap();
                    assert!(
                        layout_text(frame.layout.as_ref().unwrap()).contains("retained history")
                    );
                    pane.rendered = pane.bridge.state.revision;
                    pane.frames += 1;
                    clock.charge(Work::Render);
                }
            },
        );
        assert!(
            projection.spent <= Duration::from_micros(600),
            "aggregate projection exceeded 600us"
        );
        if frames == 1 {
            assert_eq!(drain.spent, Duration::from_micros(300));
            assert_eq!(projection.spent, Duration::from_micros(600));
            assert_eq!(
                projection.received, 3,
                "host must rearm the retained fourth view"
            );
            assert_eq!(
                panes.iter().filter(|p| p.rendered == p.target).count(),
                3,
                "fourth render must remain admitted for the next frame"
            );
            assert!(panes.iter().any(|p| p.bridge.pending_notices()));
        }
        if panes
            .iter()
            .all(|p| !p.bridge.pending_notices() && p.rendered == p.target)
        {
            break;
        }
        assert!(frames < 40, "round-robin work did not make progress");
    }
    for mut pane in panes {
        assert_eq!(pane.bridge.state.revision, pane.target);
        assert_eq!(pane.frames, 1);
        assert_eq!(pane.bridge.state.transcript.cursor, Seq(5));
        assert_eq!(pane.bridge.state.transcript.duplicate_durable, 0);
        assert_eq!(pane.bridge.state.transcript.items.len(), 2);
        assert_eq!(
            pane.bridge.state.transcript.items[&pane.item].body,
            Item::AgentMessage {
                markdown: "x".repeat(128)
            }
        );
        pane.bridge.detach().unwrap().await.unwrap();
    }
    println!(
        "gui aggregate budget: four real bridges/native views; {frames} frames; shared injected clock, drain <=300us, projection <=600us; deferred fourth projection and notification tails retained; CPU margins UNMEASURED"
    );
}
