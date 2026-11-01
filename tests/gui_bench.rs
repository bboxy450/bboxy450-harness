// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        bridge::{Bridge, FrameState},
        model::Transcript,
        view::{HarnessView, Preparer},
    },
    protocol::*,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
#[test]
#[ignore = "wall-clock measurement belongs to make bench-budgets, never make test"]
fn hundred_thousand_deltas_frame_p99() {
    let mut load = [0f64; 3];
    // SAFETY: getloadavg writes at most this three-element output array.
    let observed = unsafe { libc::getloadavg(load.as_mut_ptr(), 3) };
    let cpus = std::thread::available_parallelism().unwrap().get();
    if observed < 1 || load[0] > cpus as f64 {
        println!(
            "UNMEASURED gui-budget: load {:.2}, ncpu {cpus}; no p99 verdict",
            load[0]
        );
        return;
    }
    let mut e: Envelope = serde_json::from_str(
        include_str!("fixtures/protocol/all-items.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let mut model = Transcript::default();
    model.apply(&e);
    let item = ItemId::new();
    e.seq = Some(Seq(2));
    e.at = Seq(2);
    e.event = serde_json::from_value(serde_json::json!({"type":"item_started","turn":null,"item":item,"kind":"agent_message","parent":null,"head":null})).unwrap();
    model.apply(&e);
    e.seq = None;
    e.event = Event::ItemDelta {
        item,
        delta: Delta::Text { append: "x".into() },
    };
    let mut view = HarnessView::default();
    let mut bridge = Bridge::new(Arc::new(Wake));
    let mut samples = vec![];
    let mut preparer = Preparer::default();
    let mut max_stages = [0f64; 5];
    let mut wall_samples = vec![];
    for i in 0..100_000 {
        model.apply(&e);
        if i % 500 == 499 {
            // Projection is performed by the preparation worker/consumer. The
            // measured UI interval starts with an immutable complete snapshot.
            let mut state = FrameState {
                transcript: Arc::new(model.clone()),
                ready: true,
                replay: true,
                ..Default::default()
            };
            state.prepared = Some(Arc::new(
                preparer.prepare(state.transcript.clone(), 1120.).unwrap(),
            ));
            let start = Instant::now();
            let cpu = thread_cpu();
            bridge.drain(Duration::from_micros(300));
            let frame = view.render(&state, [1120., 860.]).unwrap();
            for (i, t) in frame.timings.iter().enumerate() {
                max_stages[i] = max_stages[i].max(t.as_secs_f64() * 1000.);
            }
            samples.push((thread_cpu() - cpu).as_secs_f64() * 1000.);
            wall_samples.push(start.elapsed().as_secs_f64() * 1000.);
        }
    }
    wall_samples.sort_by(f64::total_cmp);
    println!(
        "gui-budget wall p99 including scheduling: {:.3}ms",
        wall_samples[wall_samples.len() * 99 / 100]
    );
    samples.sort_by(f64::total_cmp);
    let cpu_p99 = samples[samples.len() * 99 / 100];
    let p99 = wall_samples[wall_samples.len() * 99 / 100];
    println!(
        "gui-budget: {} frames / 100000 deltas, p99 drain+projection wall {p99:.3}ms; CPU {cpu_p99:.3}ms (limit 0.9ms)",
        samples.len()
    );
    println!("gui-budget stage maxima: {max_stages:?}ms");
    assert!(p99 <= 0.9, "p99={p99:.3}ms");
}

// The budget allocates CPU, not time descheduled by concurrent builds. Retain
// wall-clock p99 alongside it so scheduler delay is visible in the gate log.
fn thread_cpu() -> Duration {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime initializes this correctly aligned output on success.
    assert_eq!(
        unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, value.as_mut_ptr()) },
        0
    );
    let value = unsafe { value.assume_init() };
    Duration::new(value.tv_sec as u64, value.tv_nsec as u32)
}
