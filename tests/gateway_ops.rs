// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    gateway::{Ack, Gateway, Refused},
    protocol::*,
};
use std::time::{Duration, Instant};
async fn running_session() -> (
    support::Session,
    bboxy450_harness::gateway::Subscription,
    TurnId,
) {
    let mut p = bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    p.delay = Duration::from_millis(3);
    let s = support::with_provider(std::sync::Arc::new(p), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "first"))
        .unwrap();
    let turn = loop {
        let e = sub.recv().await.unwrap();
        if let Event::TurnStarted { turn } = e.event {
            break turn;
        }
    };
    (s, sub, turn)
}
fn fill(g: &bboxy450_harness::gateway::InProc) {
    for _ in 0..10000 {
        if g.send(support::prompt(g, "queued")) == Err(Refused::Busy) {
            return;
        }
    }
    panic!("work inbox never became full");
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
    let (s, mut sub, turn) = running_session().await;
    fill(&s.gateway);
    let op = support::prompt(&s.gateway, "refused work");
    // The trait returns the result directly; it cannot yield a reservation future.
    let send: fn(&bboxy450_harness::gateway::InProc, OpEnvelope) -> Result<Ack, Refused> =
        Gateway::send;
    assert_eq!(send(&s.gateway, op), Err(Refused::Busy));
    let close = support::op(&s.gateway, Op::Close);
    s.gateway.send(close.clone()).unwrap();
    let events = support::end(&mut sub).await;
    assert!(events.iter().any(
        |e| matches!(e.event,Event::TurnEnded{turn:t,reason:StopReason::Cancelled,..} if t==turn)
            && e.cause == Some(close.op_id)
    ));
    support::close(s).await;
}
#[tokio::test]
async fn interrupt_when_full() {
    let (s, mut sub, turn) = running_session().await;
    fill(&s.gateway);
    let interrupt = support::op(&s.gateway, Op::Interrupt { turn: Some(turn) });
    let start = Instant::now();
    assert!(!s.gateway.send(interrupt.clone()).unwrap().duplicate);
    assert!(s.gateway.send(interrupt.clone()).unwrap().duplicate);
    let events = support::end(&mut sub).await;
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(events.iter().any(
        |e| matches!(e.event,Event::TurnEnded{turn:t,reason:StopReason::Cancelled,..} if t==turn)
            && e.cause == Some(interrupt.op_id)
    ));
    let log = support::close(s).await;
    assert_eq!(
        log.iter()
            .filter(|e| matches!(e.event,Event::TurnEnded{turn:t,..} if t==turn))
            .count(),
        1
    );
}
