// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Native harness logs (Claude Code, Codex) into our log format: one-shot import, live
//! watch through the Gateway, and continuing a watched session on an ACP agent.
mod support;
use bboxy450_harness::{
    engine::Config,
    gateway::{Gateway, Host, Subscribe},
    import::{Native, import_file, rfc3339_ms},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
};
use std::{path::Path, sync::Arc, time::Duration};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name)
}
fn read(path: &Path) -> Vec<Envelope> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
fn completed(log: &[Envelope]) -> Vec<(ItemStatus, Item)> {
    log.iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted { status, body, .. } => Some((status.clone(), body.clone())),
            _ => None,
        })
        .collect()
}

fn numbers(usage: Usage) -> Usage {
    Usage {
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cached_input_tokens: usage.cached_input_tokens,
        cache_creation_input_tokens: usage.cache_creation_input_tokens,
        reasoning_tokens: usage.reasoning_tokens,
        cost_micros_usd: usage.cost_micros_usd,
        ..Usage::default()
    }
}
fn assert_import_usage(usage: &Usage, scope: UsageScope) {
    assert_eq!(usage.scope, Some(scope.clone()));
    assert_eq!(
        usage.source,
        Some(if scope == UsageScope::Turn {
            NumberSource::Derived
        } else {
            NumberSource::Reported
        })
    );
    let backend = usage.backend.as_ref().expect("import backend required");
    assert_eq!(backend.kind, BackendKind::Import);
    assert!(matches!(backend.name.as_str(), "claude-code" | "codex"));
}
fn usages(log: &[Envelope]) -> Vec<Usage> {
    completed(log)
        .into_iter()
        .filter_map(|(_, body)| match body {
            Item::Usage { usage } => {
                let scope = usage.scope.clone().unwrap();
                assert_import_usage(&usage, scope.clone());
                (scope == UsageScope::Call).then(|| numbers(usage))
            }
            _ => None,
        })
        .collect()
}

fn turn_usages(log: &[Envelope]) -> Vec<Usage> {
    log.iter()
        .filter_map(|e| match &e.event {
            Event::TurnEnded {
                usage: Some(usage), ..
            } => {
                assert_import_usage(usage, UsageScope::Turn);
                Some(numbers(usage.clone()))
            }
            _ => None,
        })
        .collect()
}

fn usage_models(log: &[Envelope]) -> Vec<String> {
    let mut model = String::new();
    let mut models = vec![];
    for e in log {
        match &e.event {
            Event::ProfileChanged { model: next, .. } => model = next.clone(),
            Event::ItemCompleted {
                body: Item::Usage { usage },
                ..
            } if usage.scope == Some(UsageScope::Call) => {
                assert_eq!(usage.model.as_deref(), Some(model.as_str()));
                models.push(model.clone());
            }
            _ => {}
        }
    }
    models
}

#[tokio::test]
async fn claude_usage_counts_message_blocks_once_and_keeps_cache_tiers_and_model() {
    let t = tempfile::tempdir().unwrap();
    let session = import_file(None, &fixture("claude-usage.jsonl"), t.path().into())
        .await
        .unwrap();
    let log = read(&t.path().join(format!("{session}.jsonl")));
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    assert_eq!(
        usages(&log),
        vec![
            Usage {
                input_tokens: 32,
                output_tokens: 3,
                cached_input_tokens: Some(10),
                cache_creation_input_tokens: Some(20),
                reasoning_tokens: Some(1),
                ..Usage::default()
            },
            Usage {
                input_tokens: 15,
                output_tokens: 7,
                cached_input_tokens: Some(5),
                cache_creation_input_tokens: Some(6),
                reasoning_tokens: Some(2),
                ..Usage::default()
            },
            Usage {
                input_tokens: 1,
                output_tokens: 2,
                ..Usage::default()
            },
        ]
    );
    assert_eq!(
        turn_usages(&log),
        vec![
            Usage {
                input_tokens: 47,
                output_tokens: 10,
                cached_input_tokens: Some(15),
                cache_creation_input_tokens: Some(26),
                reasoning_tokens: Some(3),
                ..Usage::default()
            },
            Usage {
                input_tokens: 1,
                output_tokens: 2,
                ..Usage::default()
            },
        ]
    );
    assert_eq!(
        usage_models(&log),
        ["claude-one", "claude-one", "claude-two"]
    );
    let reported: Vec<_> = completed(&log)
        .into_iter()
        .filter_map(|(_, b)| match b {
            Item::Usage { usage } => Some(usage),
            _ => None,
        })
        .collect();
    assert_eq!(
        reported
            .iter()
            .map(|u| u.request_id.as_deref())
            .collect::<Vec<_>>(),
        [Some("request-a"), Some("request-b"), Some("request-c")]
    );
    assert!(
        reported
            .iter()
            .all(|u| u.backend.as_ref().unwrap().version.as_deref() == Some("claude-fixture-2"))
    );
}

#[tokio::test]
async fn codex_usage_deduplicates_cumulative_snapshots_not_equal_calls_and_handles_resets() {
    let t = tempfile::tempdir().unwrap();
    let session = import_file(None, &fixture("codex-usage.jsonl"), t.path().into())
        .await
        .unwrap();
    let log = read(&t.path().join(format!("{session}.jsonl")));
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    let calls = usages(&log);
    assert_eq!(calls.len(), 3);
    assert_eq!(
        calls[0], calls[1],
        "equal calls with different cumulative counters both count"
    );
    assert_eq!(
        calls[2],
        Usage {
            input_tokens: 3,
            output_tokens: 1,
            cached_input_tokens: Some(0),
            cache_creation_input_tokens: Some(0),
            reasoning_tokens: Some(0),
            ..Usage::default()
        }
    );
    assert_eq!(
        turn_usages(&log),
        vec![
            Usage {
                input_tokens: 20,
                output_tokens: 8,
                cached_input_tokens: Some(6),
                cache_creation_input_tokens: Some(4),
                reasoning_tokens: Some(2),
                ..Usage::default()
            },
            Usage {
                input_tokens: 3,
                output_tokens: 1,
                cached_input_tokens: Some(0),
                cache_creation_input_tokens: Some(0),
                reasoning_tokens: Some(0),
                ..Usage::default()
            },
        ]
    );
    assert_eq!(usage_models(&log), ["codex-one", "codex-one", "codex-two"]);
    let snapshots: Vec<_> = completed(&log)
        .into_iter()
        .filter_map(|(_, b)| match b {
            Item::Usage { usage } if usage.scope == Some(UsageScope::Session) => {
                assert_import_usage(&usage, UsageScope::Session);
                assert!(
                    usage.model.is_none(),
                    "session totals can span billed models"
                );
                assert_eq!(
                    usage.backend.as_ref().unwrap().version.as_deref(),
                    Some("codex-fixture-3")
                );
                Some((usage.input_tokens, usage.output_tokens))
            }
            _ => None,
        })
        .collect();
    assert_eq!(snapshots, [(10, 4), (20, 8), (25, 10), (3, 1)]);
}

#[test]
fn partial_usage_names_absent_required_counts_and_preserves_reported_zero() {
    use bboxy450_harness::import::Projector;
    use serde_json::json;
    for native in [Native::ClaudeCode, Native::Codex] {
        let mut p = Projector::new(native);
        p.push(&match native {
            Native::ClaudeCode => {
                json!({"type":"user","version":"fixture-version","message":{"content":"hello"}})
            }
            Native::Codex => {
                json!({"type":"session_meta","payload":{"cli_version":"fixture-version"}})
            }
        });
        let mut seen = vec![];
        for (id, usage) in [
            ("partial", json!({"output_tokens":3})),
            ("zero", json!({"input_tokens":0,"output_tokens":0})),
        ] {
            let events = p.push(&match native {
                Native::ClaudeCode => json!({"type":"assistant","requestId":id,"message":{"id":id,"usage":usage,"content":[]}}),
                Native::Codex => json!({"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":usage}}}),
            });
            for e in events {
                if let Event::ItemCompleted {
                    body: Item::Usage { usage },
                    ..
                } = e
                {
                    seen.push(usage);
                }
            }
        }
        assert_eq!(seen.len(), 2);
        for u in &seen {
            assert_import_usage(u, UsageScope::Call);
            assert_eq!(
                u.backend.as_ref().unwrap().version.as_deref(),
                Some("fixture-version")
            );
            assert_eq!(u.backend.as_ref().unwrap().name, native.label());
            assert!(u.model.is_none());
            assert!(
                u.cached_input_tokens.is_none()
                    && u.reasoning_tokens.is_none()
                    && u.cost_micros_usd.is_none()
            );
        }
        assert_eq!(seen[0].unreported, ["input_tokens"]);
        assert_eq!((seen[0].input_tokens, seen[0].output_tokens), (0, 3));
        assert!(seen[1].unreported.is_empty());
        let ended = p.finish(StopReason::EndTurn);
        let Event::TurnEnded {
            usage: Some(total), ..
        } = &ended[0]
        else {
            panic!("missing derived turn total")
        };
        assert_import_usage(total, UsageScope::Turn);
        assert_eq!(total.unreported, ["input_tokens"]);
        assert_eq!((total.input_tokens, total.output_tokens), (0, 3));
    }
    assert_eq!(
        serde_json::from_value::<UsageScope>(json!("future-scope")).unwrap(),
        UsageScope::Unknown(json!("future-scope"))
    );
    assert_eq!(
        serde_json::from_value::<NumberSource>(json!("future-source")).unwrap(),
        NumberSource::Unknown(json!("future-source"))
    );
    assert_eq!(
        serde_json::from_value::<BackendKind>(json!("future-backend")).unwrap(),
        BackendKind::Unknown(json!("future-backend"))
    );
}

#[test]
fn unknown_usage_is_absent_and_cumulative_only_usage_stays_session_scope() {
    use bboxy450_harness::import::Projector;
    use serde_json::json;
    let mut p = Projector::new(Native::Codex);
    assert!(
        p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":null}}))
            .iter()
            .all(|e| !matches!(
                e,
                Event::ItemCompleted {
                    body: Item::Usage { .. },
                    ..
                }
            ))
    );
    assert!(p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{}}}})).iter().all(|e|!matches!(e,Event::ItemCompleted {body:Item::Usage {..},..})));
    let events = p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":7,"output_tokens":2}}}}));
    assert_eq!(events.iter().filter(|e| matches!(e, Event::ItemCompleted { body: Item::Usage { usage }, .. } if usage.input_tokens == 7 && usage.output_tokens == 2 && usage.scope == Some(UsageScope::Session) && usage.source == Some(NumberSource::Reported))).count(), 1);
    assert!(
        p.finish(StopReason::Interrupted).is_empty(),
        "a session snapshot does not invent a turn"
    );
    let events = p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":{"model_context_window":200000}}}));
    let usage = events
        .iter()
        .find_map(|e| match e {
            Event::ItemCompleted {
                body: Item::Usage { usage },
                ..
            } => Some(usage),
            _ => None,
        })
        .unwrap();
    assert_import_usage(usage, UsageScope::Session);
    assert_eq!(usage.context_window, Some(200000));
    assert_eq!(usage.unreported, ["input_tokens", "output_tokens"]);
    assert!(p.finish(StopReason::EndTurn).is_empty());
    let mut p = Projector::new(Native::ClaudeCode);
    p.push(&json!({"type":"user","message":{"content":"hello"}}));
    assert!(matches!(
        &p.finish(StopReason::EndTurn)[..],
        [Event::TurnEnded { usage: None, .. }]
    ));
}

#[test]
fn usage_overflow_remains_unknown_and_resets_for_the_next_turn() {
    use bboxy450_harness::import::Projector;
    use serde_json::json;
    let mut p = Projector::new(Native::Codex);
    for input in [u64::MAX, 1, 2] {
        p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":input,"output_tokens":1}}}}));
    }
    assert!(matches!(
        &p.finish(StopReason::EndTurn)[..],
        [Event::TurnEnded { usage: None, .. }]
    ));
    p.push(&json!({"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":3,"output_tokens":1}}}}));
    assert!(
        matches!(&p.finish(StopReason::EndTurn)[..], [Event::TurnEnded { usage: Some(usage), .. }] if usage.input_tokens == 3)
    );
}

fn spill_fixture(name: &str, text: &str) -> String {
    fn replace(v: &mut serde_json::Value, text: &str) {
        match v {
            serde_json::Value::String(s) if s == "__LARGE_TEXT__" => *s = text.to_owned(),
            serde_json::Value::Array(a) => a.iter_mut().for_each(|v| replace(v, text)),
            serde_json::Value::Object(o) => o.values_mut().for_each(|v| replace(v, text)),
            _ => {}
        }
    }
    std::fs::read_to_string(fixture(name))
        .unwrap()
        .lines()
        .map(|line| {
            let mut v = serde_json::from_str(line).unwrap();
            replace(&mut v, text);
            serde_json::to_string(&v).unwrap() + "\n"
        })
        .collect()
}

async fn assert_spilled(sessions: &Path, session: SessionId, text: &str) {
    let path = sessions.join(format!("{session}.jsonl"));
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.lines().all(|l| l.len() < MAX_ENVELOPE_BYTES));
    let log = read(&path);
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    let items = completed(&log);
    let (_, tool) = items
        .iter()
        .find(|(_, b)| matches!(b, Item::ToolCall { spill: Some(_), .. }))
        .unwrap();
    let Item::ToolCall {
        spill: Some(blob),
        output,
        input_raw,
        ..
    } = tool
    else {
        panic!("missing tool spill")
    };
    let blobs = bboxy450_harness::store::Blobs {
        dir: sessions.join(session.to_string()).join("blobs"),
        redactor: Set::default(),
    };
    assert_eq!(blobs.get(blob).await.unwrap(), text.as_bytes());
    assert!(input_raw.contains("[full content: blob "));
    assert!(
        matches!(&output[..], [OutputBlock::Text { text } | OutputBlock::Error { text }] if text.len() < 9 * 1024 && text.contains(&blob.sha256))
    );
    assert!(items.iter().any(
        |(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown.contains(&blob.sha256))
    ));
    assert!(items.iter().any(
        |(_, b)| matches!(b, Item::Reasoning { summary, .. } if summary.contains(&blob.sha256))
    ));
    assert!(items.iter().any(|(_, b)| matches!(b, Item::UserMessage { blocks } if matches!(&blocks[..], [InputBlock::Text { text }] if text.contains(&blob.sha256)))));
}

#[tokio::test]
async fn both_import_formats_spill_large_outputs_inputs_and_text_with_unicode_safe_previews() {
    // The second input exceeds 256 KiB only after JSON escaping, despite 44 KiB of text.
    for text in ["🦊界\0\"\\".repeat(40_000), "\0".repeat(44_000)] {
        for name in ["claude-spill.jsonl", "codex-spill.jsonl"] {
            let t = tempfile::tempdir().unwrap();
            let native = t.path().join("native.jsonl");
            std::fs::write(&native, spill_fixture(name, &text)).unwrap();
            let sessions = t.path().join("sessions");
            let session = import_file(None, &native, sessions.clone()).await.unwrap();
            assert_spilled(&sessions, session, &text).await;
        }
    }
}

#[tokio::test]
async fn live_watch_uses_the_same_usage_and_spill_path_for_both_formats() {
    use std::io::Write;
    let text = "🦊\0".repeat(65_536);
    for (native, usage, spill) in [
        (
            Native::ClaudeCode,
            "claude-usage.jsonl",
            "claude-spill.jsonl",
        ),
        (Native::Codex, "codex-usage.jsonl", "codex-spill.jsonl"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let sessions = t.path().join("sessions");
        let host = Host::new(Config {
            meta: support::meta(t.path().into()),
            sessions: sessions.clone(),
            profiles: t.path().join("profiles"),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        });
        let path = t.path().join("watch.jsonl");
        std::fs::write(&path, std::fs::read(fixture(usage)).unwrap()).unwrap();
        let watch = host
            .watch(Some(native), path.clone(), t.path().into())
            .await
            .unwrap();
        let mut sub = watch
            .subscribe(Subscribe {
                session: watch.session(),
                after: None,
                ephemeral: false,
            })
            .unwrap();
        let mut ended = 0;
        while ended < 2 {
            let e = tokio::time::timeout(Duration::from_secs(10), sub.recv())
                .await
                .unwrap()
                .unwrap();
            ended += usize::from(matches!(e.event, Event::TurnEnded { .. }));
        }
        let before = read(&sessions.join(format!("{}.jsonl", watch.session())));
        assert_eq!(usages(&before).len(), 3);
        // Appended bytes, rather than a preloaded file, exercise the live tail path.
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        write!(f, "{}", spill_fixture(spill, &text)).unwrap();
        loop {
            let e = tokio::time::timeout(Duration::from_secs(10), sub.recv())
                .await
                .unwrap()
                .unwrap();
            if matches!(e.event, Event::TurnEnded { .. }) {
                break;
            }
        }
        watch.send(support::op(&watch, Op::Close)).unwrap();
        tokio::time::timeout(Duration::from_secs(10), host.join())
            .await
            .unwrap()
            .unwrap();
        assert_spilled(&sessions, watch.session(), &text).await;
    }
}

#[tokio::test]
async fn watched_spill_failure_publishes_no_dangling_completion() {
    use std::io::Write;
    let t = tempfile::tempdir().unwrap();
    let sessions = t.path().join("sessions");
    let host = Host::new(Config {
        meta: support::meta(t.path().into()),
        sessions: sessions.clone(),
        profiles: t.path().join("profiles"),
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 2,
        profile_error: None,
    });
    let path = t.path().join("watch.jsonl");
    std::fs::write(&path, "").unwrap();
    let watch = host
        .watch(Some(Native::Codex), path.clone(), t.path().into())
        .await
        .unwrap();
    // An exact fixture-owned path makes every blob write fail without permissions tricks.
    std::fs::write(
        sessions.join(watch.session().to_string()).join("blobs"),
        "not a directory",
    )
    .unwrap();
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    write!(
        f,
        "{}",
        spill_fixture("codex-spill.jsonl", &"x".repeat(MAX_ENVELOPE_BYTES))
    )
    .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(10), host.join())
            .await
            .unwrap()
            .is_err()
    );
    let log = read(&sessions.join(format!("{}.jsonl", watch.session())));
    assert!(
        completed(&log)
            .iter()
            .all(|(_, body)| matches!(body, Item::Native { blob: None, .. })),
        "failed persistence must not publish a reference; metadata Native items may precede it"
    );
}

#[tokio::test]
async fn preview_boundaries_are_cut_only_after_redaction() {
    let t = tempfile::tempdir().unwrap();
    let sessions = t.path().join("sessions");
    let secret = "fixture-sensitive-boundary";
    let redactor = Set::default();
    redactor.register(secret);
    let mut meta = support::meta(t.path().into());
    meta.session = SessionId::new();
    let host = Host::new(Config {
        meta,
        sessions: sessions.clone(),
        profiles: t.path().join("profiles"),
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: redactor.clone(),
        max_rounds: 2,
        profile_error: None,
    });
    let text = format!(
        "{}{}{}",
        "x".repeat(8192 - 10),
        secret,
        "y".repeat(MAX_ENVELOPE_BYTES)
    );
    let path = t.path().join("watch.jsonl");
    std::fs::write(&path, spill_fixture("codex-spill.jsonl", &text)).unwrap();
    let watch = host
        .watch(Some(Native::Codex), path, t.path().into())
        .await
        .unwrap();
    let mut sub = watch
        .subscribe(Subscribe {
            session: watch.session(),
            after: None,
            ephemeral: false,
        })
        .unwrap();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(10), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(e.event, Event::TurnEnded { .. }) {
            break;
        }
    }
    watch.send(support::op(&watch, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(10), host.join())
        .await
        .unwrap()
        .unwrap();
    let raw = std::fs::read_to_string(sessions.join(format!("{}.jsonl", watch.session()))).unwrap();
    assert!(
        !raw.contains("fixture-se"),
        "even a cut prefix of the registered secret must be absent"
    );
    assert_spilled(&sessions, watch.session(), &redactor.scrub(&text)).await;
}

#[test]
fn rfc3339_timestamps_become_epoch_milliseconds() {
    assert_eq!(rfc3339_ms("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(
        rfc3339_ms("2026-11-02T10:00:00.000Z"),
        Some(1_793_613_600_000)
    );
    assert_eq!(
        rfc3339_ms("2026-11-02T02:00:00.5-08:00"),
        Some(1_793_613_600_500)
    );
    assert_eq!(rfc3339_ms("not a time"), None);
}

#[tokio::test]
async fn a_claude_code_session_projects_into_a_valid_log_with_its_own_clock() {
    let t = tempfile::tempdir().unwrap();
    let session = import_file(None, &fixture("claude.jsonl"), t.path().into())
        .await
        .unwrap();
    let log = read(&t.path().join(format!("{session}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
    let items = completed(&log);
    assert!(
        items.iter().any(|(_,body)|matches!(body, Item::UserMessage { blocks } if matches!(&blocks[..], [InputBlock::Text { text }] if text == "Find the flaky test.")))
    );
    assert!(items.iter().any(|(_, b)| matches!(b, Item::Reasoning { summary, .. } if summary == "Look at the test list first.")));
    assert!(items.iter().any(|(s, b)| matches!(s, ItemStatus::Failed { .. })
        && matches!(b, Item::ToolCall { name, tool_kind: ToolKind::Execute, input_raw, output, .. }
            if name == "Bash" && input_raw.contains("cargo test --list") && matches!(&output[..], [OutputBlock::Error { text }] if text == "error: could not compile"))));
    assert!(items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "The build fails before any test runs.")));
    // side chains and harness bookkeeping are not part of the conversation
    assert!(!items.iter().any(
        |(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown.contains("subagent"))
    ));
    assert!(!items.iter().any(|(_, b)| matches!(b, Item::UserMessage { blocks } if format!("{blocks:?}").contains("/clear"))));
    assert!(log.iter().any(|e| matches!(&e.event, Event::SessionStarted { meta } if meta.cwd == Path::new("/work/demo"))));
    assert!(log.iter().any(
        |e| matches!(&e.event, Event::ProfileChanged { model, .. } if model == "claude-opus-5-5")
    ));
    let ended = log
        .iter()
        .find(|e| matches!(e.event, Event::TurnEnded { .. }))
        .unwrap();
    assert_eq!(
        ended.ts_ms,
        rfc3339_ms("2026-11-02T10:00:07.000Z").unwrap(),
        "native time, not import time"
    );
}

#[tokio::test]
async fn a_codex_rollout_projects_turns_reasoning_and_tool_calls() {
    let t = tempfile::tempdir().unwrap();
    let session = import_file(
        Some(Native::Codex),
        &fixture("codex.jsonl"),
        t.path().into(),
    )
    .await
    .unwrap();
    let log = read(&t.path().join(format!("{session}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
    let items = completed(&log);
    let users: Vec<_> = items
        .iter()
        .filter(|(_, b)| matches!(b, Item::UserMessage { .. }))
        .collect();
    assert_eq!(users.len(), 1, "environment context is not a prompt");
    assert!(
        items
            .iter()
            .any(|(_, b)| matches!(b, Item::Reasoning { summary, .. } if summary == "Use ls."))
    );
    assert!(items.iter().any(|(s, b)| matches!(s, ItemStatus::Completed)
        && matches!(b, Item::ToolCall { name, tool_kind: ToolKind::Execute, output, .. } if name == "exec_command" && matches!(&output[..], [OutputBlock::Text { text }] if text == "note.txt"))));
    assert!(items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "There is one file: note.txt.")));
    assert_eq!(
        log.iter()
            .filter(|e| matches!(
                e.event,
                Event::TurnEnded {
                    reason: StopReason::EndTurn,
                    ..
                }
            ))
            .count(),
        1
    );
}

#[tokio::test]
async fn a_watched_native_log_streams_live_and_continues_on_an_acp_agent_with_the_handoff() {
    let t = tempfile::tempdir().unwrap();
    let profiles = t.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/acp-agent/fake_agent.py");
    std::fs::write(
        profiles.join("echo.md"),
        format!("---\nname: echo\nprovider: acp-agent\nagent:\n  command: [python3, -I, {}, echo]\n---\n", script.display()),
    )
    .unwrap();
    let mut meta = support::meta(t.path().into());
    meta.session = SessionId::new();
    let host = Host::new(Config {
        meta,
        sessions: t.path().join("sessions"),
        profiles,
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 2,
        profile_error: None,
    });
    // the native session grows while we watch it
    let native = t.path().join("live.jsonl");
    let lines: Vec<String> = std::fs::read_to_string(fixture("claude.jsonl"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    std::fs::write(&native, lines[..4].join("\n") + "\n").unwrap();
    let watch = host
        .watch(Some(Native::ClaudeCode), native.clone(), t.path().into())
        .await
        .unwrap();
    let mut sub = watch
        .subscribe(Subscribe {
            session: watch.session(),
            after: None,
            ephemeral: false,
        })
        .unwrap();
    let mut seen = vec![];
    let wait_for =
        |seen: &[Envelope], pred: &dyn Fn(&Event) -> bool| seen.iter().any(|e| pred(&e.event));
    while !wait_for(&seen, &|e| {
        matches!(
            e,
            Event::ItemCompleted {
                body: Item::Reasoning { .. },
                ..
            }
        )
    }) {
        seen.push(
            tokio::time::timeout(Duration::from_secs(5), sub.recv())
                .await
                .unwrap()
                .unwrap(),
        );
    }
    // append the rest of the native session: it arrives live
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&native)
            .unwrap();
        writeln!(f, "{}", lines[4..].join("\n")).unwrap();
    }
    while !wait_for(&seen, &|e| matches!(e, Event::TurnEnded { .. })) {
        seen.push(
            tokio::time::timeout(Duration::from_secs(5), sub.recv())
                .await
                .unwrap()
                .unwrap(),
        );
    }
    // a turn on a watched session is refused with the way forward
    watch.send(support::prompt(&watch, "hi")).unwrap();
    // continue it on an ACP agent: fork at head, the agent gets the conversation
    let next = host.switch(&watch, "echo".into()).await.unwrap();
    let mut sub = next
        .subscribe(Subscribe {
            session: next.session(),
            after: Some((next.hello().epoch, next.hello().head)),
            ephemeral: false,
        })
        .unwrap();
    next.send(support::prompt(&next, "what failed?")).unwrap();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(10), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(e.event, Event::TurnEnded { .. }) {
            break;
        }
    }
    let child = next.session();
    next.send(support::op(&next, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(10), host.join())
        .await
        .unwrap()
        .unwrap();
    let watched = read(
        &t.path()
            .join("sessions")
            .join(format!("{}.jsonl", watch.session())),
    );
    let forked = read(&t.path().join("sessions").join(format!("{child}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&watched).unwrap();
    bboxy450_harness::protocol::laws::check_authored(&forked).unwrap();
    assert!(watched.iter().any(|e| matches!(&e.event, Event::OpRejected { reason: Rejection::InvalidInput { detail }, .. } if detail.contains("/profile"))));
    assert!(completed(&forked).iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown }
        if markdown.starts_with("ECHO:") && markdown.contains("[user] Find the flaky test.")
            && markdown.contains("[tool Bash]") && markdown.contains("[assistant] The build fails before any test runs.") && markdown.contains("what failed?"))),
        "{:?}", completed(&forked));
}
