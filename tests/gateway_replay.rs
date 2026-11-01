// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    gateway::{Gateway, Subscribe, TryRecv},
    protocol::*,
};
#[tokio::test]
async fn subscribe_at_every_seq_and_reset() {
    let s = support::session(vec![support::sse("text")], false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "hello"))
        .unwrap();
    let events = support::end(&mut sub).await;
    let head = events.last().unwrap().seq.unwrap().0;
    let epoch = s.gateway.hello().epoch;
    for after in 0..=head {
        let mut sub = s
            .gateway
            .subscribe(Subscribe {
                session: s.gateway.hello().meta.session,
                after: Some((epoch, Seq(after))),
                ephemeral: false,
            })
            .unwrap();
        let mut seq = (1..=after).collect::<Vec<_>>();
        while let TryRecv::Envelope(e) = sub.try_recv() {
            if let Some(s) = e.seq {
                seq.push(s.0);
            }
        }
        assert_eq!(seq, (1..=head).collect::<Vec<_>>());
    }
    let mut sub = s
        .gateway
        .subscribe(Subscribe {
            session: s.gateway.hello().meta.session,
            after: Some((Epoch(0), Seq(head + 10))),
            ephemeral: true,
        })
        .unwrap();
    assert!(matches!(
        sub.recv().await.unwrap().event,
        Event::Reset { .. }
    ));
    assert!(sub.recv().await.is_none());
    support::close(s).await;
}
