// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#[path = "support/sse_server.rs"]
mod sse_server;
mod support;
use bboxy450_harness::{
    protocol::ErrorClass,
    provider::*,
    redact::{Secret, Set},
};
use futures_util::StreamExt;
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
fn request() -> ModelRequest {
    ModelRequest {
        model: "mock".into(),
        messages: vec![],
        tools: vec![],
        tool_choice: ToolChoice::Auto,
        max_tokens: None,
        stable_prefix: 0,
    }
}
fn client(server: &sse_server::Server) -> OpenAiCompat {
    let set = Set::default();
    let key = Secret::new("test-credential".into(), &set);
    OpenAiCompat::new(
        &server.endpoint,
        key,
        set,
        Options {
            backoff: Duration::from_millis(1),
            read_stall: Duration::from_millis(50),
            max_requests: Some(3),
            retry_deadline: Some(Duration::from_secs(3)),
        },
    )
    .unwrap()
}
#[tokio::test]
async fn strict_assembly_and_fragmentation() {
    for (name, good, ready) in [
        ("text", true, 0),
        ("tools", true, 2),
        ("parallel", true, 2),
        ("utf8-split", true, 1),
        ("duplicate-id", false, 0),
        ("missing-finish", false, 0),
    ] {
        let provider = ScriptedProvider::new(vec![support::sse(name)]);
        let events = provider
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await;
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, ModelEvent::Finish(_) | ModelEvent::Failed(_)))
                .count(),
            1,
            "{name}"
        );
        assert_eq!(
            matches!(events.last(), Some(ModelEvent::Finish(_))),
            good,
            "{name}: {events:?}"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, ModelEvent::ToolCallReady { .. }))
                .count(),
            ready
        );
    }
}
#[tokio::test]
async fn real_http_retry_after_then_success() {
    let mut retry = sse_server::ok(vec![]);
    retry.status = 429;
    retry.retry_after = Some(1);
    let server = sse_server::Server::start(vec![retry, sse_server::ok(support::sse("text"))]).await;
    let start = Instant::now();
    let events = client(&server)
        .stream(request(), CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert_eq!(server.attempts.load(Ordering::SeqCst), 2);
    assert!(matches!(
        events.first(),
        Some(ModelEvent::Retrying {
            class: ErrorClass::RateLimited,
            ..
        })
    ));
    assert!(matches!(
        events.last(),
        Some(ModelEvent::Finish(Finish::Stop))
    ));
}
#[tokio::test]
async fn midstream_disconnect_never_retries() {
    let partial=b"data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"partial\"},\"finish_reason\":null}]}\n\n".to_vec();
    let mut reply = sse_server::ok(partial);
    reply.truncated = true;
    let server = sse_server::Server::start(vec![reply]).await;
    let events = client(&server)
        .stream(request(), CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        events.last(),
        Some(ModelEvent::Failed(ProviderError {
            class: ErrorClass::MidStream,
            ..
        }))
    ));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, ModelEvent::Retrying { .. }))
    );
}
#[tokio::test]
async fn cancel_mid_stream() {
    let mut reply = sse_server::ok(support::sse("cancel"));
    reply.hold = Duration::from_secs(1);
    reply.truncated = true;
    let server = sse_server::Server::start(vec![reply]).await;
    let token = CancellationToken::new();
    let mut stream = client(&server).stream(request(), token.clone());
    let start = Instant::now();
    token.cancel();
    assert!(matches!(
        stream.next().await,
        Some(ModelEvent::Failed(ProviderError {
            class: ErrorClass::Cancelled,
            ..
        }))
    ));
    assert!(start.elapsed() < Duration::from_millis(250));
    assert!(stream.next().await.is_none());
}
#[tokio::test]
async fn stall_is_bounded_and_errors_do_not_echo_bodies() {
    let mut reply = sse_server::ok(support::sse("stall"));
    reply.hold = Duration::from_millis(100);
    reply.truncated = true;
    let server = sse_server::Server::start(vec![reply.clone(), reply.clone(), reply]).await;
    let events = client(&server)
        .stream(request(), CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    assert!(matches!(events.last(), Some(ModelEvent::Failed(_))));
    assert!(server.attempts.load(Ordering::SeqCst) <= 3);
    assert!(parse_retry_after("2").unwrap() >= Duration::from_secs(2));
}
#[tokio::test]
async fn empty_final_delta_after_tool_finish_preserves_ready_calls() {
    let mut bytes = support::sse("edit");
    let done = bytes
        .windows(14)
        .position(|s| s == b"data: [DONE]\n\n")
        .unwrap();
    bytes.splice(done..done,b"data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"\",\"reasoning\":\"\",\"tool_calls\":[]},\"finish_reason\":\"tool_calls\"}],\"usage\":{\"prompt_tokens\":2,\"completion_tokens\":3}}\n\n".iter().copied());
    let p = ScriptedProvider::new(vec![bytes]);
    let events = p
        .stream(request(), CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, ModelEvent::ToolCallReady { .. }))
            .count(),
        1
    );
    assert!(matches!(
        events.last(),
        Some(ModelEvent::Finish(Finish::ToolCalls))
    ));
}
