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
    protocol::{ErrorClass, Usage},
    provider::*,
    redact::{Secret, Set},
};
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/sse/anthropic/{name}.sse")),
    )
    .unwrap()
}
fn request() -> ModelRequest {
    ModelRequest {
        model: "mock".into(),
        messages: vec![json!({"role":"user","content":"Hello"})],
        tools: vec![],
        tool_choice: ToolChoice::Auto,
        max_tokens: None,
        stable_prefix: 0,
    }
}
fn client(server: &sse_server::Server) -> Anthropic {
    let set = Set::default();
    Anthropic::new(
        server.endpoint.trim_end_matches("/v1"),
        Secret::new("fixture-credential".into(), &set),
        set,
        Options {
            backoff: Duration::from_millis(1),
            read_stall: Duration::from_millis(100),
            max_requests: Some(3),
            retry_deadline: Some(Duration::from_secs(4)),
        },
    )
    .unwrap()
}
fn failed(class: ErrorClass, message: &str) -> ModelEvent {
    ModelEvent::Failed(ProviderError::new(class, message))
}
fn call(index: u8, id: &str, name: &str) -> ModelEvent {
    ModelEvent::ToolCallDelta {
        index,
        id: Some(id.into()),
        name: Some(name.into()),
        args: String::new(),
    }
}
fn args(index: u8, raw: &str) -> ModelEvent {
    ModelEvent::ToolCallDelta {
        index,
        id: None,
        name: None,
        args: raw.into(),
    }
}
fn ready(index: u8, id: &str, name: &str, raw: &str) -> ModelEvent {
    ModelEvent::ToolCallReady {
        index,
        id: id.into(),
        name: name.into(),
        args_raw: raw.into(),
        args: serde_json::from_str::<Value>(raw)
            .ok()
            .filter(Value::is_object)
            .ok_or_else(|| "arguments must be a JSON object".into()),
    }
}
fn tool_events(index: u8, id: &str, name: &str, raw: &str) -> Vec<ModelEvent> {
    vec![call(index, id, name), args(index, raw)]
}
async fn events(name: &str) -> Vec<ModelEvent> {
    let server = sse_server::Server::start(vec![sse_server::ok(fixture(name))]).await;
    let out = client(&server)
        .stream(request(), CancellationToken::new())
        .collect()
        .await;
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1, "{name}");
    out
}
#[tokio::test]
async fn fixtures_have_exact_events_through_real_http() {
    let read = r#"{"path":"input.txt"}"#;
    let glob = r#"{"pattern":"*.rs"}"#;
    let utf8 = r#"{"path":"café.txt"}"#;
    let mut tool = vec![ModelEvent::Text("Checking".into())];
    tool.extend(tool_events(1, "call_a", "read", read));
    tool.extend([
        ready(1, "call_a", "read", read),
        ModelEvent::Finish(Finish::ToolCalls),
    ]);
    let mut parallel = tool_events(0, "call_a", "read", read);
    parallel.extend(tool_events(1, "call_b", "glob", glob));
    parallel.extend([
        ready(0, "call_a", "read", read),
        ready(1, "call_b", "glob", glob),
        ModelEvent::Finish(Finish::ToolCalls),
    ]);
    let mut utf8_events = tool_events(0, "call_a", "read", utf8);
    utf8_events.extend([
        ready(0, "call_a", "read", utf8),
        ModelEvent::Finish(Finish::ToolCalls),
    ]);
    let mut duplicate = tool_events(0, "call_a", "read", read);
    duplicate.push(failed(ErrorClass::Malformed, "duplicate tool id"));
    let mut contradiction = tool_events(0, "call_a", "read", read);
    contradiction.push(failed(
        ErrorClass::Malformed,
        "stop reason contradicts tool calls",
    ));
    let mut invalid = tool_events(0, "call_a", "read", "[1]");
    invalid.extend([
        ready(0, "call_a", "read", "[1]"),
        ModelEvent::Finish(Finish::ToolCalls),
    ]);
    for (name, expected) in [
        (
            "text",
            vec![
                ModelEvent::Usage(Usage {
                    input_tokens: 18,
                    output_tokens: 0,
                    cached_input_tokens: Some(5),
                    cache_creation_input_tokens: Some(3),
                    ..Default::default()
                }),
                ModelEvent::Text("Hello".into()),
                ModelEvent::Usage(Usage {
                    input_tokens: 18,
                    output_tokens: 2,
                    cached_input_tokens: Some(5),
                    cache_creation_input_tokens: Some(3),
                    ..Default::default()
                }),
                ModelEvent::Finish(Finish::Stop),
            ],
        ),
        ("text-then-tool", tool),
        ("parallel", parallel),
        ("utf8-split", utf8_events),
        ("duplicate-id", duplicate),
        (
            "missing-finish",
            vec![
                ModelEvent::Text("Hello".into()),
                failed(ErrorClass::Malformed, "missing stop reason"),
            ],
        ),
        (
            "refusal",
            vec![
                ModelEvent::Text("I cannot help.".into()),
                ModelEvent::Finish(Finish::ContentFilter),
            ],
        ),
        (
            "stop-sequence",
            vec![
                ModelEvent::Text("Hello".into()),
                ModelEvent::Finish(Finish::Stop),
            ],
        ),
        (
            "length",
            vec![
                ModelEvent::Text("Hello".into()),
                ModelEvent::Finish(Finish::Length),
            ],
        ),
        (
            "unknown-finish",
            vec![
                ModelEvent::Text("Hello".into()),
                failed(ErrorClass::Malformed, "unknown stop reason"),
            ],
        ),
        ("contradictory-finish", contradiction),
        ("invalid-args", invalid),
        (
            "error-after-output",
            vec![
                ModelEvent::Text("partial".into()),
                failed(ErrorClass::MidStream, "provider stream error"),
            ],
        ),
    ] {
        assert_eq!(events(name).await, expected, "{name}");
    }
}
#[tokio::test]
async fn thinking_signatures_and_redacted_blocks_are_opaque_snapshots() {
    let thinking =
        json!({"type":"thinking","thinking":"Let me think.","signature":"signed-bytes=="});
    let text = json!({"type":"text","text":"Answer"});
    assert_eq!(
        events("thinking-signature").await,
        vec![
            ModelEvent::Reasoning("Let me think.".into()),
            ModelEvent::ReasoningOpaque(json!({"anthropic_content":[thinking]})),
            ModelEvent::Text("Answer".into()),
            ModelEvent::ReasoningOpaque(json!({"anthropic_content":[thinking,text]})),
            ModelEvent::Finish(Finish::Stop),
        ]
    );
    let redacted = json!({"type":"redacted_thinking","data":"opaque-redacted-bytes=="});
    assert_eq!(
        events("redacted-thinking").await,
        vec![
            ModelEvent::ReasoningOpaque(json!({"anthropic_content":[redacted]})),
            ModelEvent::Text("Answer".into()),
            ModelEvent::ReasoningOpaque(json!({"anthropic_content":[redacted,text]})),
            ModelEvent::Finish(Finish::Stop),
        ]
    );
}
#[tokio::test]
async fn retry_after_then_success_is_exact_and_counts_both_requests() {
    let mut retry = sse_server::ok(b"body must never appear".to_vec());
    retry.status = 429;
    retry.retry_after = Some(1);
    let server =
        sse_server::Server::start(vec![retry, sse_server::ok(fixture("429-then-success"))]).await;
    let provider = client(&server);
    let start = Instant::now();
    assert_eq!(
        provider
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![
            ModelEvent::Retrying {
                attempt: 1,
                wait_ms: 1000,
                class: ErrorClass::RateLimited
            },
            ModelEvent::Text("Hello".into()),
            ModelEvent::Finish(Finish::Stop)
        ]
    );
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert_eq!(provider.request_count(), 2);
}
#[tokio::test]
async fn disconnect_stall_cancel_and_error_never_replay_output() {
    let mut reply = sse_server::ok(fixture("500-after-output"));
    reply.truncated = true;
    let server = sse_server::Server::start(vec![reply]).await;
    assert_eq!(
        client(&server)
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![
            ModelEvent::Text("partial".into()),
            failed(ErrorClass::MidStream, "provider stream interrupted")
        ]
    );
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1);
    let mut reply = sse_server::ok(fixture("stall"));
    reply.truncated = true;
    reply.hold = Duration::from_secs(2);
    let server = sse_server::Server::start(vec![reply]).await;
    let mut provider = client(&server);
    provider.options.max_requests = Some(1);
    assert_eq!(
        provider
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![failed(ErrorClass::Transient, "retry budget exhausted")]
    );
    let mut reply = sse_server::ok(fixture("cancel"));
    reply.truncated = true;
    reply.hold = Duration::from_secs(2);
    let server = sse_server::Server::start(vec![reply]).await;
    let token = CancellationToken::new();
    let mut stream = client(&server).stream(request(), token.clone());
    assert_eq!(
        stream.next().await,
        Some(ModelEvent::Text("partial".into()))
    );
    let start = Instant::now();
    token.cancel();
    assert_eq!(
        stream.next().await,
        Some(failed(ErrorClass::Cancelled, "cancelled"))
    );
    assert!(start.elapsed() < Duration::from_millis(250));
    assert!(stream.next().await.is_none());
}
#[tokio::test]
async fn http_and_prestream_errors_are_classified_without_reflecting_bodies() {
    for (status, class) in [
        (401, ErrorClass::Auth),
        (403, ErrorClass::Auth),
        (400, ErrorClass::BadRequest),
        (404, ErrorClass::BadRequest),
        (413, ErrorClass::BadRequest),
        (429, ErrorClass::RateLimited),
        (408, ErrorClass::Transient),
        (500, ErrorClass::Transient),
        (501, ErrorClass::Transient),
        (503, ErrorClass::Transient),
        (529, ErrorClass::Transient),
    ] {
        let mut reply = sse_server::ok(b"fixture-credential body must never appear".to_vec());
        reply.status = status;
        let server = sse_server::Server::start(vec![reply]).await;
        let mut provider = client(&server);
        provider.options.max_requests = Some(1);
        let message = if matches!(class, ErrorClass::Transient | ErrorClass::RateLimited) {
            "retry budget exhausted".to_string()
        } else {
            format!("provider request refused (HTTP {status})")
        };
        assert_eq!(
            provider
                .stream(request(), CancellationToken::new())
                .collect::<Vec<_>>()
                .await,
            vec![failed(class, &message)]
        );
    }
    let server = sse_server::Server::start(vec![
        sse_server::ok(fixture("error-before-output")),
        sse_server::ok(fixture("429-then-success")),
    ])
    .await;
    let mut stream = client(&server).stream(request(), CancellationToken::new());
    assert!(matches!(
        stream.next().await,
        Some(ModelEvent::Retrying {
            attempt: 1,
            class: ErrorClass::Transient,
            ..
        })
    ));
    assert_eq!(
        stream.collect::<Vec<_>>().await,
        vec![
            ModelEvent::Text("Hello".into()),
            ModelEvent::Finish(Finish::Stop)
        ]
    );
}
fn captured(server: &sse_server::Server, index: usize) -> (String, Value) {
    let requests = server.requests.lock().unwrap();
    let text = String::from_utf8(requests[index].clone()).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    (head.to_lowercase(), serde_json::from_str(body).unwrap())
}
fn tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "read".into(),
            description: "Read a file".into(),
            parameters: json!({"type":"object"}),
        },
        ToolDef {
            name: "glob".into(),
            description: "List files".into(),
            parameters: json!({"type":"object"}),
        },
    ]
}
#[tokio::test]
async fn request_shape_preserves_pairing_and_prefix_bytes_and_uses_only_api_key() {
    let server =
        sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success")); 2]).await;
    let mut req = request();
    req.tools = tools();
    req.tool_choice = ToolChoice::Named("read".into());
    req.max_tokens = Some(512);
    req.stable_prefix = 4;
    req.messages = vec![
        json!({"role":"system","content":"Exact system\nline"}),
        json!({"role":"user","content":"Stable é"}),
        json!({"role":"assistant","content":"checking","tool_calls":[
            {"id":"a","function":{"name":"read","arguments":"{ \"path\": \"a\" }"}},
            {"id":"b","function":{"name":"glob","arguments":"{\"pattern\":\"*\"}"}}]}),
        json!({"role":"tool","tool_call_id":"a","content":"A"}),
        json!({"role":"tool","tool_call_id":"b","content":"B"}),
        json!({"role":"user","content":"Suffix one"}),
    ];
    let original = req.messages.clone();
    let mut provider = client(&server);
    provider.max_tokens = Some(256);
    provider
        .stream(req.clone(), CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    req.messages[5]["content"] = json!("Suffix two");
    provider
        .stream(req, CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    let (headers, body) = captured(&server, 0);
    let (_, next) = captured(&server, 1);
    assert!(headers.starts_with("post /v1/messages http/1.1"));
    assert!(headers.contains("x-api-key: fixture-credential"));
    assert!(headers.contains("anthropic-version: 2023-06-01"));
    for absent in ["authorization:", "anthropic-beta:", "client-id:"] {
        assert!(!headers.contains(absent));
    }
    assert_eq!(body["max_tokens"], 512);
    assert_eq!(body["stream"], true);
    assert_eq!(body["tool_choice"], json!({"type":"tool","name":"read"}));
    assert_eq!(
        body["system"],
        json!([{"type":"text","text":"Exact system\nline","cache_control":{"type":"ephemeral"}}])
    );
    assert_eq!(
        body["messages"][1]["content"],
        json!([{"type":"text","text":"checking"},{"type":"tool_use","id":"a","name":"read","input":{"path":"a"}},{"type":"tool_use","id":"b","name":"glob","input":{"pattern":"*"}}])
    );
    assert_eq!(
        body["messages"][2],
        json!({"role":"user","content":[
        {"type":"tool_result","tool_use_id":"a","content":"A","cache_control":{"type":"ephemeral"}},
        {"type":"tool_result","tool_use_id":"b","content":"B"},{"type":"text","text":"Suffix one"}]})
    );
    assert!(body["tools"][0].get("cache_control").is_none());
    assert_eq!(
        body["tools"][1]["cache_control"],
        json!({"type":"ephemeral"})
    );
    assert_eq!(body["tools"][0]["input_schema"], json!({"type":"object"}));
    assert_eq!(body["messages"][0], next["messages"][0]);
    assert_eq!(body["messages"][1], next["messages"][1]);
    assert_eq!(
        serde_json::to_vec(&body["messages"][2]["content"][0]).unwrap(),
        serde_json::to_vec(&next["messages"][2]["content"][0]).unwrap()
    );
    assert_eq!(original[0]["content"], "Exact system\nline");
}
#[tokio::test]
async fn tool_choices_profile_defaults_and_invalid_pairing() {
    let server =
        sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success")); 4]).await;
    for (index, choice) in [
        ToolChoice::Auto,
        ToolChoice::None,
        ToolChoice::Required,
        ToolChoice::Named("read".into()),
    ]
    .into_iter()
    .enumerate()
    {
        let mut req = request();
        req.tools = tools();
        req.tool_choice = choice;
        let mut provider = client(&server);
        if index == 1 {
            provider.max_tokens = Some(123);
        }
        provider
            .stream(req, CancellationToken::new())
            .collect::<Vec<_>>()
            .await;
        let (_, body) = captured(&server, index);
        assert_eq!(
            body["tool_choice"]["type"],
            ["auto", "none", "any", "tool"][index]
        );
        assert_eq!(body["max_tokens"], if index == 1 { 123 } else { 4096 });
    }
    let mut req = request();
    req.messages
        .push(json!({"role":"tool","tool_call_id":"orphan","content":"x"}));
    assert_eq!(
        client(&server)
            .stream(req, CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![failed(ErrorClass::BadRequest, "unpaired tool history")]
    );
    assert_eq!(server.attempts.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn initial_profile_literal_env_and_command_sources_replay_signed_tools_and_scrub_secrets() {
    use bboxy450_harness::protocol::{Envelope, Event, StopReason, laws::check_transcript};
    for source in ["literal", "env", "command"] {
        let server = sse_server::Server::start(vec![
            sse_server::ok(fixture("thinking-tool")),
            sse_server::ok(fixture("429-then-success")),
        ])
        .await;
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let profiles = temp.path().join(".bboxy450-harness/profiles");
        std::fs::create_dir_all(&profiles).unwrap();
        let sentinel = "fixture-credential";
        let key = match source {
            "literal" => format!("api_key: {sentinel}"),
            "env" => "api_key_env: FIXTURE_ANTHROPIC_KEY".into(),
            _ => format!(
                "api_key: '!printf x >> {}; printf {sentinel}'",
                temp.path().join("key-commands").display()
            ),
        };
        std::fs::write(profiles.join("test.md"),format!("---\nname: test\nprovider: anthropic\nmodel: mock\nendpoint: {}\nmax_tokens: 77\n{key}\n---\n",server.endpoint.trim_end_matches("/v1"))).unwrap();
        std::fs::write(
            temp.path().join("input.txt"),
            format!("File contains {sentinel}"),
        )
        .unwrap();
        let output = tokio::time::timeout(
            Duration::from_secs(8),
            tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
                .args([
                    "--json",
                    "--yolo",
                    "--profile",
                    "test",
                    "-p",
                    "Read input.txt",
                ])
                .arg("--cwd")
                .arg(temp.path())
                .arg("--sessions")
                .arg(temp.path().join("sessions"))
                .env("HOME", temp.path())
                .env("FIXTURE_ANTHROPIC_KEY", sentinel)
                .env_remove("BBOXY_CAPABILITY")
                .env_remove("BBOXY_CAPABILITY_FILE")
                .kill_on_drop(true)
                .output(),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(
            output.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains(sentinel));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(sentinel));
        let events = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        check_transcript(&events).unwrap();
        assert!(
            events.iter().any(|e| matches!(
                e.event,
                Event::TurnEnded {
                    reason: StopReason::EndTurn,
                    ..
                }
            )),
            "{events:?}"
        );
        assert_eq!(server.attempts.load(Ordering::SeqCst), 2);
        let (_, first) = captured(&server, 0);
        let (_, second) = captured(&server, 1);
        assert_eq!(first["max_tokens"], 77);
        assert_eq!(second["max_tokens"], 77);
        let messages = second["messages"].as_array().unwrap();
        let assistant = messages.iter().find(|m| m["role"] == "assistant").unwrap();
        assert_eq!(
            assistant["content"],
            json!([
                {"type":"thinking","thinking":"Let me think.","signature":"signed-bytes=="},
                {"type":"tool_use","id":"call_a","name":"read","input":{"path":"input.txt"}}
            ])
        );
        assert!(
            messages
                .iter()
                .any(|m| m["content"].as_array().is_some_and(|blocks| blocks
                    .iter()
                    .any(|b| b["type"] == "tool_result" && b["tool_use_id"] == "call_a")))
        );
        assert!(!serde_json::to_string(&second).unwrap().contains(sentinel));
        for path in std::fs::read_dir(temp.path().join("sessions")).unwrap() {
            let path = path.unwrap().path();
            if path.is_file() {
                assert!(!String::from_utf8_lossy(&std::fs::read(path).unwrap()).contains(sentinel));
            }
        }
        if source == "command" {
            assert_eq!(
                std::fs::read_to_string(temp.path().join("key-commands")).unwrap(),
                "x"
            );
        }
    }
}
#[tokio::test]
async fn idle_profile_switch_dispatches_anthropic() {
    use bboxy450_harness::{
        gateway::Gateway,
        protocol::{Event, Op},
    };
    let server = sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success"))]).await;
    let session = support::session(vec![], false).await;
    let profiles = session.temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(profiles.join("native.md"),format!("---\nname: native\nprovider: anthropic\nmodel: mock\nendpoint: {}\napi_key: '!printf fixture-credential'\n---\n",server.endpoint.trim_end_matches("/v1"))).unwrap();
    let mut sub = support::subscribe(&session.gateway, false);
    session
        .gateway
        .send(support::op(
            &session.gateway,
            Op::SetProfile {
                profile: "native".into(),
            },
        ))
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(e) = sub.recv().await {
            if matches!(e.event, Event::ProfileChanged { .. }) {
                return;
            }
        }
        panic!("profile stream closed");
    })
    .await
    .unwrap();
    session
        .gateway
        .send(support::prompt(&session.gateway, "Hello"))
        .unwrap();
    let ending = support::end(&mut sub).await;
    assert!(ending.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: bboxy450_harness::protocol::StopReason::EndTurn,
            ..
        }
    )));
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1);
    support::close(session).await;
}
#[test]
fn anthropic_profile_has_default_endpoint_and_positive_token_limit() {
    let profile=profile::Profile::parse("---\nname: native\nprovider: anthropic\nmodel: mock\nmax_tokens: 99\napi_key: fixture-credential\n---\n").unwrap();
    assert_eq!(profile.endpoint, "https://api.anthropic.com");
    assert_eq!(profile.max_tokens, Some(99));
    assert!(profile::Profile::parse("---\nname: missing\nmodel: mock\n---\n").is_err());
}

#[tokio::test]
async fn backoff_cancels_promptly_and_connection_refusal_is_transient() {
    let mut reply = sse_server::ok(vec![]);
    reply.status = 503;
    reply.retry_after = Some(60);
    let server = sse_server::Server::start(vec![reply]).await;
    let mut provider = client(&server);
    provider.options.retry_deadline = None;
    let cancel = CancellationToken::new();
    let mut stream = provider.stream(request(), cancel.clone());
    assert_eq!(
        stream.next().await,
        Some(ModelEvent::Retrying {
            attempt: 1,
            wait_ms: 60_000,
            class: ErrorClass::Transient
        })
    );
    let start = Instant::now();
    cancel.cancel();
    assert_eq!(
        stream.next().await,
        Some(failed(ErrorClass::Cancelled, "cancelled"))
    );
    assert!(start.elapsed() < Duration::from_millis(250));
    assert!(stream.next().await.is_none());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let set = Set::default();
    let provider = Anthropic::new(
        &endpoint,
        Secret::new("fixture-credential".into(), &set),
        set,
        Options {
            max_requests: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        provider
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![failed(ErrorClass::Transient, "retry budget exhausted")]
    );
}
#[tokio::test]
async fn malformed_frames_stop_once_without_reflecting_wire_values() {
    let original = String::from_utf8(fixture("429-then-success")).unwrap();
    let boundary = original.find("event: message_delta").unwrap();
    for (body, message) in [
        (original[..boundary].to_owned(), "missing message_stop"),
        (
            original.replace("\"index\":0,\"delta\"", "\"index\":65,\"delta\""),
            "invalid block index",
        ),
        (
            original.replace("\"type\":\"text_delta\"", "\"type\":\"thinking_delta\""),
            "delta contradicts content block",
        ),
        (
            original.replace("\"type\":\"message_stop\"", "\"type\":\"ping\""),
            "SSE event type mismatch",
        ),
    ] {
        let server = sse_server::Server::start(vec![sse_server::ok(body.into_bytes())]).await;
        let events = client(&server)
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await;
        assert_eq!(events.last(), Some(&failed(ErrorClass::Malformed, message)));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, ModelEvent::Failed(_) | ModelEvent::Finish(_)))
                .count(),
            1
        );
    }
    let mut body = fixture("429-then-success");
    body.extend_from_slice(&fixture("error-after-output"));
    let server = sse_server::Server::start(vec![sse_server::ok(body)]).await;
    assert_eq!(
        client(&server)
            .stream(request(), CancellationToken::new())
            .collect::<Vec<_>>()
            .await,
        vec![
            ModelEvent::Text("Hello".into()),
            ModelEvent::Finish(Finish::Stop)
        ]
    );
}

#[tokio::test]
async fn endpoint_accepts_base_or_v1_without_duplicate_version_path() {
    let server =
        sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success")); 4]).await;
    let base = server.endpoint.trim_end_matches("/v1");
    for (index, endpoint) in [
        base.to_string(),
        format!("{base}/"),
        format!("{base}/v1"),
        format!("{base}/v1/"),
    ]
    .into_iter()
    .enumerate()
    {
        let set = Set::default();
        let provider = Anthropic::new(
            &endpoint,
            Secret::new("fixture-credential".into(), &set),
            set,
            Default::default(),
        )
        .unwrap();
        assert_eq!(
            provider
                .stream(request(), CancellationToken::new())
                .collect::<Vec<_>>()
                .await,
            vec![
                ModelEvent::Text("Hello".into()),
                ModelEvent::Finish(Finish::Stop)
            ]
        );
        assert!(
            captured(&server, index)
                .0
                .starts_with("post /v1/messages http/1.1")
        );
    }
}

#[tokio::test]
async fn host_profile_factory_selects_native_backend() {
    use bboxy450_harness::{
        engine::Config,
        gateway::{Gateway, Host},
        policy::Policy,
        protocol::*,
    };
    use std::sync::Arc;
    let server = sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success"))]).await;
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let profiles = temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(profiles.join("native.md"),format!("---\nname: native\nprovider: anthropic\nmodel: mock\nendpoint: {}\napi_key: fixture-credential\n---\n",server.endpoint)).unwrap();
    let meta = support::meta(temp.path().into());
    let session = meta.session;
    let config = Config {
        meta,
        profiles,
        sessions: temp.path().join("sessions"),
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "unused".into(),
        policy: Policy::new(profile::Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 2,
        profile_error: None,
    };
    let host = Host::new(config);
    let (gateway, _) = host
        .create(OpEnvelope {
            v: PROTOCOL,
            session,
            op_id: OpId::new(),
            op: Op::StartSession {
                cwd: temp.path().into(),
                profile: Some("native".into()),
                fork: None,
            },
        })
        .await
        .unwrap();
    let mut sub = support::subscribe(&gateway, false);
    gateway.send(support::prompt(&gateway, "Hello")).unwrap();
    let events = support::end(&mut sub).await;
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert!(
        captured(&server, 0)
            .0
            .starts_with("post /v1/messages http/1.1")
    );
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    host.join().await.unwrap();
    let log = std::fs::read_to_string(
        temp.path()
            .join("sessions")
            .join(format!("{session}.jsonl")),
    )
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
    .collect::<Vec<_>>();
    laws::check_transcript(&log).unwrap();
}
#[tokio::test]
async fn existing_profiles_keep_openai_request_bytes() {
    let server = sse_server::Server::start(vec![sse_server::ok(support::sse("text"))]).await;
    let profile=profile::Profile::parse(&format!("---\nname: compatible\nprovider: openrouter\nmodel: mock\nendpoint: {}\napi_key: fixture-credential\n---\n",server.endpoint)).unwrap();
    let set = Set::default();
    let key = profile.key(&set, CancellationToken::new()).await.unwrap();
    let provider = profile
        .build_provider(key, set, Default::default())
        .unwrap();
    let mut req = request();
    req.tools = tools();
    req.tool_choice = ToolChoice::Required;
    req.stable_prefix = 1;
    let expected = json!({"model":"mock","messages":req.messages,"tools":tools_wire(&req.tools),"tool_choice":"required","stream":true,"stream_options":{"include_usage":true},"reasoning":{"exclude":false}});
    let out = provider
        .stream(req, CancellationToken::new())
        .collect::<Vec<_>>()
        .await;
    assert!(matches!(out.last(), Some(ModelEvent::Finish(Finish::Stop))));
    let (headers, body) = captured(&server, 0);
    assert!(headers.starts_with("post /v1/chat/completions http/1.1"));
    assert!(headers.contains("authorization: bearer fixture-credential"));
    assert!(!headers.contains("x-api-key:"));
    assert!(!headers.contains("anthropic-version:"));
    assert_eq!(body, expected);
    let captured = server.requests.lock().unwrap();
    let bytes = &captured[0];
    let end = bytes.windows(4).position(|b| b == b"\r\n\r\n").unwrap() + 4;
    assert_eq!(&bytes[end..], serde_json::to_vec(&expected).unwrap());
}
#[test]
fn old_usage_replays_and_cache_creation_is_additive() {
    let old = json!({"input_tokens":2,"output_tokens":3,"cached_input_tokens":null,"reasoning_tokens":null,"cost_micros_usd":null});
    let usage: Usage = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(usage.cache_creation_input_tokens, None);
    assert_eq!(serde_json::to_value(&usage).unwrap(), old);
    let usage = Usage {
        cache_creation_input_tokens: Some(7),
        ..usage
    };
    let encoded = serde_json::to_value(&usage).unwrap();
    assert_eq!(encoded["cache_creation_input_tokens"], 7);
    assert_eq!(serde_json::from_value::<Usage>(encoded).unwrap(), usage);
}

#[tokio::test]
async fn signed_and_redacted_native_blocks_replay_verbatim_beside_cache_marker() {
    for name in ["thinking-signature", "redacted-thinking"] {
        let opaque = events(name)
            .await
            .into_iter()
            .filter_map(|e| {
                if let ModelEvent::ReasoningOpaque(v) = e {
                    Some(v)
                } else {
                    None
                }
            })
            .next_back()
            .unwrap();
        let server =
            sse_server::Server::start(vec![sse_server::ok(fixture("429-then-success"))]).await;
        let mut req = request();
        req.stable_prefix = 2;
        req.messages.extend([
            json!({"role":"assistant","content":"Answer","reasoning_details":opaque}),
            json!({"role":"user","content":"Next question"}),
        ]);
        assert_eq!(
            client(&server)
                .stream(req, CancellationToken::new())
                .collect::<Vec<_>>()
                .await,
            vec![
                ModelEvent::Text("Hello".into()),
                ModelEvent::Finish(Finish::Stop)
            ]
        );
        let (_, body) = captured(&server, 0);
        let content = &body["messages"][1]["content"];
        assert_eq!(content[0], opaque["anthropic_content"][0]);
        assert!(content[0].get("cache_control").is_none());
        assert_eq!(
            content[1],
            json!({"type":"text","text":"Answer","cache_control":{"type":"ephemeral"}})
        );
    }
}
