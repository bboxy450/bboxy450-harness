// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    engine::{Config, SessionActor},
    gateway::{Gateway, Subscribe},
    policy::Policy,
    protocol::*,
    provider::{
        ScriptedProvider,
        profile::{Profile, Unattended},
    },
    redact::{Set, StreamScrubber},
};
use std::sync::Arc;
#[tokio::test]
async fn key_command_model_tools_and_errors_never_persist_secrets() {
    let t = tempfile::tempdir().unwrap();
    let sentinel = format!("{}-SENTINEL-{}", "sk", ulid::Ulid::new());
    let profile=Profile::parse(&format!("---\nname: test\nmodel: mock\nendpoint: http://localhost/v1\napi_key: '!printf %s {sentinel}'\n---\n")).unwrap();
    let set = Set::default();
    let secret = profile
        .key(&set, tokio_util::sync::CancellationToken::new())
        .await
        .unwrap();
    assert!(!format!("{profile:?}{secret:?}{set:?}").contains(&sentinel));
    let mut scrubber = StreamScrubber::new(set.clone());
    let midpoint = sentinel.len() / 2;
    let mut bytes = scrubber.push(&sentinel.as_bytes()[..midpoint], false);
    bytes.extend(scrubber.push(&sentinel.as_bytes()[midpoint..], true));
    assert!(!String::from_utf8_lossy(&bytes).contains(&sentinel));
    std::fs::write(t.path().join("input.txt"), &sentinel).unwrap();
    let text = serde_json::json!({"choices":[{"index":0,"delta":{"content":sentinel},"finish_reason":"stop"}]});
    let rounds = vec![
        support::sse("tools"),
        format!("data: {text}\n\ndata: [DONE]\n\n").into_bytes(),
    ];
    std::fs::write(t.path().join("other.txt"), "other").unwrap();
    let meta = support::meta(t.path().into());
    let config = Config {
        meta: meta.clone(),
        profiles: t.path().join("profiles"),
        sessions: t.path().join("sessions"),
        provider: Arc::new(ScriptedProvider::new(rounds)),
        model: "mock".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: set.clone(),
        profile_error: None,
        max_rounds: 50,
    };
    let (g, task) = SessionActor::open(config, OpId::new()).await.unwrap();
    let mut sub = g
        .subscribe(Subscribe {
            session: meta.session,
            after: None,
            ephemeral: true,
        })
        .unwrap();
    g.send(support::prompt(&g, "read files")).unwrap();
    let events = support::end(&mut sub).await;
    assert!(!serde_json::to_string(&events).unwrap().contains(&sentinel));
    g.send(support::op(&g, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    for e in std::fs::read_dir(t.path().join("sessions")).unwrap() {
        let p = e.unwrap().path();
        if p.is_file() {
            assert!(!String::from_utf8_lossy(&std::fs::read(p).unwrap()).contains(&sentinel));
        }
    }
    let blob = bboxy450_harness::store::Blobs {
        dir: t.path().join("blobs"),
        redactor: set,
    };
    let r = blob
        .put(
            format!("provider error body {sentinel}").into_bytes(),
            "text/plain",
        )
        .await
        .unwrap();
    assert!(!String::from_utf8_lossy(&blob.get(&r).await.unwrap()).contains(&sentinel));
}
#[tokio::test]
async fn keychain_sources_are_not_launched() {
    let p=Profile::parse("---\nname: keychain\nmodel: mock\nendpoint: https://example.org/v1\napi_key: '!security find-generic-password -s deliberately-missing'\n---\n").unwrap();
    let start = std::time::Instant::now();
    assert!(
        p.key(&Set::default(), tokio_util::sync::CancellationToken::new())
            .await
            .is_err()
    );
    assert!(start.elapsed() < std::time::Duration::from_millis(100));
}
#[tokio::test]
async fn split_model_key_is_scrubbed_from_snapshots_and_seat_output() {
    use bboxy450_harness::provider::{Caps, Finish, ModelEvent, ModelRequest, Provider};
    use futures_core::Stream;
    use std::{pin::Pin, time::Duration};
    struct Split(String);
    impl Provider for Split {
        fn caps(&self) -> Caps {
            Caps {
                images: false,
                reasoning: false,
                tool_choice: false,
                context_tokens: 10000,
            }
        }
        fn stream(
            &self,
            _: ModelRequest,
            _: tokio_util::sync::CancellationToken,
        ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
            let key = self.0.clone();
            Box::pin(
                async_stream::stream! {let n=key.len()/2;yield ModelEvent::Text(key[..n].into());tokio::time::sleep(Duration::from_millis(100)).await;yield ModelEvent::Text(key[n..].into());tokio::time::sleep(Duration::from_millis(100)).await;yield ModelEvent::Finish(Finish::Stop);},
            )
        }
    }
    let t = tempfile::tempdir().unwrap();
    let sentinel = format!("{}-SPLIT-{}", "sk", ulid::Ulid::new());
    let set = Set::default();
    set.register(&sentinel);
    let meta = support::meta(t.path().into());
    let config = Config {
        meta: meta.clone(),
        profiles: t.path().join("profiles"),
        sessions: t.path().join("sessions"),
        provider: Arc::new(Split(sentinel.clone())),
        model: "split".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: set,
        max_rounds: 2,
        profile_error: None,
    };
    let (g, task) = SessionActor::open(config, OpId::new()).await.unwrap();
    let mut sub = g
        .subscribe(Subscribe {
            session: meta.session,
            after: None,
            ephemeral: true,
        })
        .unwrap();
    g.send(support::prompt(&g, "stream")).unwrap();
    let events = support::end(&mut sub).await;
    assert!(
        events
            .iter()
            .any(|e| matches!(e.event, Event::ItemSnapshot { .. }))
    );
    assert!(!serde_json::to_string(&events).unwrap().contains(&sentinel));
    let mut projection = bboxy450_harness::frontends::seat::LineProjection::default();
    let output = events
        .iter()
        .flat_map(|e| projection.render(e))
        .collect::<Vec<_>>();
    assert!(!String::from_utf8_lossy(&output).contains(&sentinel));
    g.send(support::op(&g, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
}
