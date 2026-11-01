// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    gateway::{Gateway, Subscribe},
    protocol::*,
    provider::{Caps, Finish, ModelEvent, ModelRequest, Provider},
};
use std::{sync::Arc, time::Duration};
struct Noise;
impl Provider for Noise {
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: false,
            tool_choice: false,
            context_tokens: 128000,
        }
    }
    fn stream(
        &self,
        _: ModelRequest,
        c: tokio_util::sync::CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        Box::pin(
            async_stream::stream! {for i in 0..2300{if c.is_cancelled(){return;}yield ModelEvent::Retrying{attempt:i,wait_ms:0,class:ErrorClass::Transient};}for _ in 0..100000{if c.is_cancelled(){return;}yield ModelEvent::Text("x".into());}yield ModelEvent::Finish(Finish::Stop);},
        )
    }
}
#[tokio::test]
async fn firehose_dead_reader() {
    let s = support::with_provider(Arc::new(Noise), false).await;
    let mut slow = support::subscribe(&s.gateway, true);
    let mut fast = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "firehose"))
        .unwrap();
    let mut events = vec![];
    loop {
        let e = tokio::time::timeout(Duration::from_secs(30), fast.recv())
            .await
            .unwrap()
            .unwrap();
        let end = matches!(e.event, Event::TurnEnded { .. });
        events.push(e);
        if end {
            break;
        }
    }
    bboxy450_harness::protocol::laws::check_transcript(&events).unwrap();
    let (epoch, mut after) = (slow.hello().epoch, Seq(0));
    let mut got_lag = false;
    while let Some(e) = slow.recv().await {
        if let Event::Lagged { resume_after } = e.event {
            after = resume_after;
            got_lag = true;
        }
    }
    assert!(got_lag);
    let mut replay = s
        .gateway
        .subscribe(Subscribe {
            session: s.gateway.hello().meta.session,
            after: Some((epoch, after)),
            ephemeral: false,
        })
        .unwrap();
    let mut expected = after.0 + 1;
    while expected <= s.gateway.hello().head.0 {
        let e = replay.recv().await.unwrap();
        if let Some(seq) = e.seq {
            assert_eq!(seq.0, expected);
            expected += 1;
        }
    }
    support::close(s).await;
}
