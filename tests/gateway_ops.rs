// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    gateway::{Gateway, Refused},
    protocol::*,
};
use std::time::{Duration, Instant};
async fn full_lane_case() {
    let mut p = bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    p.delay = Duration::from_millis(2);
    let s = support::with_provider(std::sync::Arc::new(p), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    let first = support::prompt(&s.gateway, "first");
    s.gateway.send(first.clone()).unwrap();
    assert!(s.gateway.send(first).unwrap().duplicate);
    let mut busy = false;
    for _ in 0..10000 {
        let e = s.gateway.send(support::prompt(&s.gateway, "queued"));
        if e == Err(Refused::Busy) {
            let start = Instant::now();
            assert_eq!(
                s.gateway.send(support::prompt(&s.gateway, "busy")),
                Err(Refused::Busy)
            );
            assert!(start.elapsed() < Duration::from_millis(1));
            busy = true;
            break;
        }
    }
    assert!(busy);
    let interrupt = support::op(&s.gateway, Op::Interrupt { turn: None });
    assert!(s.gateway.send(interrupt).is_ok());
    let _ = support::end(&mut sub).await;
    support::close(s).await;
}
#[tokio::test]
async fn version_and_unknown_session_refused() {
    let s = support::session(vec![support::sse("text")], false).await;
    let mut op = support::prompt(&s.gateway, "x");
    op.v.major = 99;
    assert!(matches!(s.gateway.send(op), Err(Refused::Version { .. })));
    let mut op = support::prompt(&s.gateway, "x");
    op.session = SessionId::new();
    assert_eq!(s.gateway.send(op), Err(Refused::UnknownSession));
    support::close(s).await;
}

#[tokio::test]
async fn busy_not_wait() {
    full_lane_case().await;
}
#[tokio::test]
async fn interrupt_when_full() {
    full_lane_case().await;
}
