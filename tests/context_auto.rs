// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    context::{self, auto},
    engine::{Config, SessionActor},
    gateway::Gateway,
    policy::Policy,
    protocol::*,
    provider::profile::Unattended,
    provider::{Caps, Finish, ModelEvent, ModelRequest, Provider, ProviderError},
    redact::Set,
};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use tokio_util::sync::CancellationToken;
struct Recording {
    requests: Arc<Mutex<Vec<ModelRequest>>>,
    ids: AtomicU64,
    overflow: bool,
}
impl Provider for Recording {
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: true,
            tool_choice: true,
            context_tokens: 0,
        }
    }
    fn stream(
        &self,
        req: ModelRequest,
        _: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        self.requests.lock().unwrap().push(req.clone());
        let mut events = vec![];
        if self.overflow {
            events.push(ModelEvent::Failed(ProviderError::new(
                ErrorClass::BadRequest,
                "context overflow",
            )));
        } else if req.messages.last().is_some_and(|m| m["role"] == "tool") {
            events.push(ModelEvent::Text("Read the file.".into()));
            events.push(ModelEvent::Finish(Finish::Stop));
        } else {
            let id = format!("read-{}", self.ids.fetch_add(1, Ordering::Relaxed));
            let args = json!({"path":"input.txt"});
            events.push(ModelEvent::Reasoning("private thought".repeat(40)));
            events.push(ModelEvent::ToolCallDelta {
                index: 0,
                id: Some(id.clone()),
                name: Some("read".into()),
                args: args.to_string(),
            });
            events.push(ModelEvent::ToolCallReady {
                index: 0,
                id,
                name: "read".into(),
                args_raw: args.to_string(),
                args: Ok(args),
            });
            events.push(ModelEvent::Finish(Finish::ToolCalls));
        }
        Box::pin(futures_util::stream::iter(events))
    }
}
async fn setup(
    window: Option<u64>,
    overflow: bool,
) -> (support::Session, Arc<Mutex<Vec<ModelRequest>>>, Config) {
    let temp = if let Some(root) = std::env::var_os("HARNESS_CONTEXT_CHILD_ROOT") {
        tempfile::tempdir_in(root).unwrap()
    } else {
        tempfile::tempdir().unwrap()
    };
    std::fs::write(temp.path().join("input.txt"), "x".repeat(4000)).unwrap();
    std::fs::create_dir(temp.path().join("profiles")).unwrap();
    let mut profile =
        json!({"name":"scripted","model":"mock","endpoint":"https://example.invalid/v1"});
    if let Some(window) = window {
        profile["context_tokens"] = json!(window);
    }
    std::fs::write(
        temp.path().join("profiles/scripted.md"),
        format!("---\n{}\n---\n", serde_yaml::to_string(&profile).unwrap()),
    )
    .unwrap();
    let requests = Arc::new(Mutex::new(vec![]));
    let provider = Arc::new(Recording {
        requests: requests.clone(),
        ids: 0.into(),
        overflow,
    });
    let config = Config {
        meta: support::meta(temp.path().into()),
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider,
        model: "mock".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 4,
        profile_error: None,
    };
    let (gateway, task) = SessionActor::open(config.clone(), OpId::new())
        .await
        .unwrap();
    (
        support::Session {
            temp,
            gateway,
            task,
        },
        requests,
        config,
    )
}
#[tokio::test]
async fn thirty_turns_trim_then_summarize_and_share_the_stable_prefix() {
    let (s, requests, _) = setup(Some(4000), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    for _ in 0..30 {
        s.gateway
            .send(support::prompt(&s.gateway, "Read the file again."))
            .unwrap();
        let events = support::end(&mut sub).await;
        assert!(matches!(
            events.last().unwrap().event,
            Event::TurnEnded {
                reason: StopReason::EndTurn,
                ..
            }
        ));
    }
    let log = support::close(s).await;
    assert!(log.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            body: Item::ContextTrim { .. },
            ..
        }
    )));
    assert!(log.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            body: Item::Compaction { .. },
            ..
        }
    )));
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 60);
    for request in requests.iter() {
        // Compaction bounds the conversation; tool definitions are a fixed cost
        // that grows when the session's own Python MCP becomes ready.
        let estimate = auto::estimate(&[], &request.messages, &[]);
        assert!(estimate <= 3200, "conversation estimate {estimate}");
        bboxy450_harness::engine::history::check_wire(&request.messages).unwrap();
    }
    // Each request in this provider starts one Reasoning or AgentMessage item.
    // Its preceding durable history identifies the reduction frontier at that
    // exact request, rather than inferring a frontier from coincidentally equal
    // prefixes (including the trivially stable one-message system prompt).
    let starts = log
        .iter()
        .filter_map(|e| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::Reasoning | ItemKind::AgentMessage,
                    ..
                }
            )
            .then_some(e.seq.unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(starts.len(), requests.len());
    let mut baseline = None;
    let mut compared = 0;
    for (request, start) in requests.iter().zip(starts) {
        let prefix_log = log
            .iter()
            .filter(|e| e.seq.is_some_and(|s| s < start))
            .cloned()
            .collect::<Vec<_>>();
        let frontier = (
            context::clipped_through(&prefix_log),
            context::latest(&prefix_log)
                .map(|(_, s)| s)
                .unwrap_or(Seq(0)),
        );
        if frontier == (Seq(0), Seq(0)) {
            continue;
        }
        assert!(
            request.stable_prefix > 1,
            "reduced cache prefix contains only the system prompt"
        );
        let bytes = serde_json::to_vec(&request.messages[..request.stable_prefix]).unwrap();
        if let Some((old_frontier, old_len, old_bytes)) = &baseline
            && *old_frontier == frontier
        {
            assert_eq!(
                request.stable_prefix, *old_len,
                "cache prefix length changed without a reduction"
            );
            assert_eq!(
                &bytes, old_bytes,
                "cache prefix changed without a reduction"
            );
            compared += 1;
        }
        baseline = Some((frontier, request.stable_prefix, bytes));
    }
    assert!(
        compared > 0,
        "no non-system prefix was compared between reductions"
    );
}
#[tokio::test]
async fn trimming_alone_fits_without_a_following_summary() {
    let (s, requests, _) = setup(Some(10000), false).await;
    std::fs::write(s.temp.path().join("input.txt"), "x".repeat(30000)).unwrap();
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "first large read"))
        .unwrap();
    support::end(&mut sub).await;
    std::fs::write(s.temp.path().join("input.txt"), "small").unwrap();
    s.gateway
        .send(support::prompt(&s.gateway, "read the now-small file"))
        .unwrap();
    let events = support::end(&mut sub).await;
    assert!(
        events.iter().any(|e| matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::ContextTrim { .. },
                ..
            }
        )),
        "trim-only path was never reached"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e.event,
            Event::ItemStarted {
                kind: ItemKind::Compaction,
                ..
            }
        )),
        "summary followed a sufficient trim"
    );
    {
        let requests = requests.lock().unwrap();
        let after = &requests[2];
        assert!(after.stable_prefix > 1);
        assert!(auto::fits(
            auto::estimate(&[], &after.messages, &after.tools),
            1250,
            10000
        ));
    }
    support::close(s).await;
}
#[tokio::test]
async fn oversized_open_turn_sends_once_and_unknown_window_warns_once() {
    let (s, requests, _) = setup(Some(4000), true).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, &"big".repeat(10000)))
        .unwrap();
    let events = support::end(&mut sub).await;
    assert!(matches!(
        events.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Error {
                class: ErrorClass::BadRequest,
                ..
            },
            ..
        }
    ));
    let log = support::close(s).await;
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert!(!log.iter().any(|e| matches!(
        e.event,
        Event::ItemStarted {
            kind: ItemKind::ContextTrim | ItemKind::Compaction,
            ..
        }
    )));
    let (s, _, _) = setup(None, false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    for _ in 0..2 {
        s.gateway.send(support::prompt(&s.gateway, "read")).unwrap();
        support::end(&mut sub).await;
    }
    let log = support::close(s).await;
    assert_eq!(log.iter().filter(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text=="context window unknown; automatic compaction off")).count(),1);
    assert!(!log.iter().any(|e| matches!(
        e.event,
        Event::ItemStarted {
            kind: ItemKind::ContextTrim,
            ..
        }
    )));
}
#[tokio::test]
async fn durable_frontier_reopens_to_the_same_request() {
    let (s, _, config) = setup(Some(4000), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    for _ in 0..8 {
        s.gateway.send(support::prompt(&s.gateway, "read")).unwrap();
        support::end(&mut sub).await;
    }
    let log = sub_log(&config);
    let blobs = bboxy450_harness::store::Blobs {
        dir: config
            .sessions
            .join(config.meta.session.to_string())
            .join("blobs"),
        redactor: config.redactor.clone(),
    };
    let before = context::replay(&log, &blobs).await.unwrap();
    s.task.abort();
    let _ = s.task.await;
    let store =
        bboxy450_harness::store::Store::open(config.sessions, config.meta, None, config.redactor)
            .await
            .unwrap();
    let after = context::replay(&store.events, &store.blobs).await.unwrap();
    assert_eq!(
        serde_json::to_vec(&before).unwrap(),
        serde_json::to_vec(&after).unwrap()
    );
    bboxy450_harness::protocol::laws::check_authored(&store.events).unwrap();
}
fn sub_log(config: &Config) -> Vec<Envelope> {
    std::fs::read_to_string(bboxy450_harness::store::log_path(
        &config.sessions,
        config.meta.session,
    ))
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
#[test]
fn thresholds_and_unicode_clipping_are_exact() {
    assert!(!auto::trigger(700, 100, 1000));
    assert!(auto::trigger(701, 100, 1000));
    assert!(auto::fits(500, 100, 1000));
    assert!(!auto::fits(501, 100, 1000));
    let clipped = context::clip_output(&"λ".repeat(1500));
    assert!(clipped.starts_with(&"λ".repeat(1024)));
    assert!(clipped.ends_with("…[clipped 952 bytes]"));
}
/// Child half of the real SIGKILL/reopen witness; invoked by its parent test.
#[tokio::test]
#[ignore]
async fn context_crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("HARNESS_CONTEXT_CHILD_ROOT").unwrap());
    let (s, _, config) = setup(Some(4000), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    for _ in 0..8 {
        s.gateway.send(support::prompt(&s.gateway, "read")).unwrap();
        support::end(&mut sub).await;
    }
    let mut log = sub_log(&config);
    let blobs = bboxy450_harness::store::Blobs {
        dir: config
            .sessions
            .join(config.meta.session.to_string())
            .join("blobs"),
        redactor: config.redactor.clone(),
    };
    let tools = bboxy450_harness::tools::Registry::new()
        .definitions()
        .iter()
        .map(|d| d.wire.clone())
        .collect::<Vec<_>>();
    let (tail, mut prefix) = context::replay(&log, &blobs).await.unwrap();
    let system = json!({"role":"system","content":format!("You are bboxy450-harness, a coding assistant. Use the provided tools. Working directory: {}\nBuild a world agents can take apart, understand, change and put back together. Subtract before you add; minimize concepts, not lines.",config.meta.cwd.display())});
    let mut messages = vec![system.clone()];
    messages.extend(tail);
    messages.push(json!({"role":"user","content":"after restart"}));
    if auto::trigger(auto::estimate(&log, &messages, &tools), 500, 4000) {
        let op = support::op(&s.gateway, Op::Compact);
        let cause = op.op_id;
        s.gateway.send(op).unwrap();
        loop {
            let event = sub.recv().await.unwrap();
            if event.cause == Some(cause)
                && matches!(
                    event.event,
                    Event::ItemCompleted {
                        body: Item::Compaction { .. },
                        ..
                    }
                )
            {
                break;
            }
        }
        log = sub_log(&config);
        let replay = context::replay(&log, &blobs).await.unwrap();
        prefix = replay.1;
        messages = vec![system];
        messages.extend(replay.0);
        messages.push(json!({"role":"user","content":"after restart"}));
    }
    assert!(log.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            body: Item::ContextTrim { .. },
            ..
        }
    )));
    let record = json!({"meta":config.meta,"sessions":config.sessions,"profiles":config.profiles,"messages":messages,"tools":tools,"stable_prefix":prefix});
    std::fs::write(
        root.join("ready.json"),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    std::future::pending::<()>().await;
}
#[tokio::test]
async fn sigkill_between_rounds_rebuilds_the_identical_model_request() {
    use std::os::unix::process::ExitStatusExt;
    let root = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "context_crash_child"])
        .env("HARNESS_CONTEXT_CHILD_ROOT", root.path())
        .env_remove("BBOXY_CAPABILITY")
        .env_remove("BBOXY_CAPABILITY_FILE")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let path = root.path().join("ready.json");
    let ready = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            if let Ok(bytes) = std::fs::read(&path)
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            {
                break value;
            }
            if let Some(status) = child.try_wait().unwrap() {
                panic!("crash fixture exited early: {status}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await;
    unsafe {
        libc::kill(child.id() as i32, libc::SIGKILL);
    }
    assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGKILL));
    let ready = ready.expect("crash fixture never reached a durable frontier");
    let requests = Arc::new(Mutex::new(vec![]));
    let config = Config {
        meta: serde_json::from_value(ready["meta"].clone()).unwrap(),
        sessions: serde_json::from_value(ready["sessions"].clone()).unwrap(),
        profiles: serde_json::from_value(ready["profiles"].clone()).unwrap(),
        provider: Arc::new(Recording {
            requests: requests.clone(),
            ids: 1000.into(),
            overflow: false,
        }),
        model: "mock".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 4,
        profile_error: None,
    };
    let (gateway, task) = SessionActor::open(config.clone(), OpId::new())
        .await
        .unwrap();
    let hello = gateway.hello();
    let mut sub = gateway
        .subscribe(bboxy450_harness::gateway::Subscribe {
            session: gateway.session(),
            after: Some((hello.epoch, hello.head)),
            ephemeral: false,
        })
        .unwrap();
    gateway
        .send(support::prompt(&gateway, "after restart"))
        .unwrap();
    support::end(&mut sub).await;
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    let requests = requests.lock().unwrap();
    let first = &requests[0];
    assert_eq!(
        serde_json::to_vec(&first.messages).unwrap(),
        serde_json::to_vec(&ready["messages"]).unwrap()
    );
    assert_eq!(serde_json::to_value(&first.tools).unwrap(), ready["tools"]);
    assert_eq!(json!(first.stable_prefix), ready["stable_prefix"]);
    bboxy450_harness::protocol::laws::check_authored(&sub_log(&config)).unwrap();
}
#[test]
fn session_context_and_missing_call_counters_cannot_anchor_a_rebuilt_request() {
    let mut log = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .collect::<Vec<_>>();
    log.retain(|e| {
        !matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::Compaction { .. } | Item::ContextTrim { .. },
                ..
            }
        )
    });
    let messages = vec![json!({"role":"user","content":"a rebuilt request after an ACP handoff"})];
    let mut without_usage = log.clone();
    without_usage.retain(|e| {
        !matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::Usage { .. },
                ..
            }
        )
    });
    let expected = auto::estimate(&without_usage, &messages, &[]);
    assert!(expected > 0);
    for (scope, unreported) in [
        (UsageScope::Session, vec![]),
        (
            UsageScope::Session,
            vec!["input_tokens".into(), "output_tokens".into()],
        ),
        (
            UsageScope::Call,
            vec!["input_tokens".into(), "output_tokens".into()],
        ),
    ] {
        for event in &mut log {
            if let Event::ItemCompleted {
                body: Item::Usage { usage },
                ..
            } = &mut event.event
            {
                *usage = Usage {
                    scope: Some(scope.clone()),
                    context_used: Some(60000),
                    context_window: Some(200000),
                    unreported: unreported.clone(),
                    ..Default::default()
                };
            }
        }
        assert_eq!(auto::estimate(&log, &messages, &[]), expected);
    }
}
#[test]
fn usage_anchors_count_exact_new_messages_and_reductions_void_them() {
    let mut log = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .collect::<Vec<_>>();
    for e in &mut log {
        if let Event::ItemCompleted {
            body: Item::Usage { usage },
            ..
        } = &mut e.event
        {
            usage.input_tokens = 100;
            usage.output_tokens = 20;
        }
    }
    let full = log.clone();
    log.retain(|e| {
        !matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::Compaction { .. } | Item::ContextTrim { .. },
                ..
            }
        )
    });
    let tools = vec![bboxy450_harness::provider::ToolDef {
        name: "tool".into(),
        description: "large definition".repeat(1000),
        parameters: json!({"type":"object"}),
    }];
    let new = [
        json!({"role":"user","content":"Read two files again."}),
        json!({"role":"assistant","content":"I’ll read both files. Continuing."}),
        json!({"role":"user","content":"Read two files to finish."}),
    ];
    let bytes = new
        .iter()
        .map(|m| serde_json::to_vec(m).unwrap().len() as u64)
        .sum::<u64>();
    assert_eq!(auto::estimate(&log, &[], &tools), 120 + bytes.div_ceil(4));
    let messages = vec![json!({"role":"user","content":"rebuilt"})];
    let bytes = serde_json::to_vec(&messages[0]).unwrap().len()
        + serde_json::to_vec(&bboxy450_harness::provider::tools_wire(&tools))
            .unwrap()
            .len();
    assert_eq!(
        auto::estimate(&full, &messages, &tools),
        (bytes as u64).div_ceil(4)
    );
}
#[tokio::test]
async fn failed_summary_blob_faults_before_request_and_recovery_closes_the_item() {
    let (s, requests, config) = setup(Some(4000), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway.send(support::prompt(&s.gateway, "read")).unwrap();
    support::end(&mut sub).await;
    let dir = config
        .sessions
        .join(config.meta.session.to_string())
        .join("blobs");
    std::fs::write(&dir, b"not a directory").unwrap();
    let mut task = s.task;
    let mut fault = false;
    for _ in 0..12 {
        s.gateway.send(support::prompt(&s.gateway, "read")).unwrap();
        loop {
            let event = tokio::time::timeout(std::time::Duration::from_secs(8), sub.recv())
                .await
                .unwrap()
                .unwrap();
            if matches!(
                event.event,
                Event::ItemStarted {
                    kind: ItemKind::Compaction,
                    ..
                }
            ) {
                let count = requests.lock().unwrap().len();
                let result = tokio::time::timeout(std::time::Duration::from_secs(8), &mut task)
                    .await
                    .expect("store fault left the actor running");
                assert!(matches!(result, Ok(Err(_))));
                assert_eq!(requests.lock().unwrap().len(), count);
                fault = true;
                break;
            }
            if matches!(event.event, Event::TurnEnded { .. }) {
                break;
            }
        }
        if fault {
            break;
        }
    }
    assert!(fault, "summary never reached the failing blob store");
    std::fs::remove_file(dir).unwrap();
    let store =
        bboxy450_harness::store::Store::open(config.sessions, config.meta, None, config.redactor)
            .await
            .unwrap();
    assert!(store.events.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            status: ItemStatus::Interrupted,
            body: Item::Compaction { .. },
            ..
        }
    )));
    bboxy450_harness::protocol::laws::check_authored(&store.events).unwrap();
}
struct PausedBoundary {
    inner: Recording,
    hold: Arc<std::sync::atomic::AtomicBool>,
    release: Arc<tokio::sync::Notify>,
}
impl Provider for PausedBoundary {
    fn caps(&self) -> Caps {
        self.inner.caps()
    }
    fn stream(
        &self,
        request: ModelRequest,
        cancel: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        use futures_util::StreamExt;
        let mut stream = self.inner.stream(request, cancel.clone());
        let held = self.hold.swap(false, Ordering::AcqRel);
        let release = self.release.clone();
        Box::pin(
            async_stream::stream! {while let Some(event)=stream.next().await {if held&&matches!(event,ModelEvent::Finish(_)){tokio::select!{biased;_=cancel.cancelled()=>return,_=release.notified()=>{}}}yield event;}},
        )
    }
}
#[tokio::test]
async fn cancelling_a_queued_turn_cannot_move_the_frontier_over_the_running_turn() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("input.txt"), "x".repeat(4000)).unwrap();
    std::fs::create_dir(temp.path().join("profiles")).unwrap();
    std::fs::write(temp.path().join("profiles/scripted.md"),"---\nname: scripted\nmodel: mock\nendpoint: https://example.invalid/v1\ncontext_tokens: 4000\n---\n").unwrap();
    let requests = Arc::new(Mutex::new(vec![]));
    let hold = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let release = Arc::new(tokio::sync::Notify::new());
    let config = Config {
        meta: support::meta(temp.path().into()),
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider: Arc::new(PausedBoundary {
            inner: Recording {
                requests: requests.clone(),
                ids: 0.into(),
                overflow: false,
            },
            hold: hold.clone(),
            release: release.clone(),
        }),
        model: "mock".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 4,
        profile_error: None,
    };
    let (gateway, task) = SessionActor::open(config.clone(), OpId::new())
        .await
        .unwrap();
    let mut sub = support::subscribe(&gateway, false);
    for _ in 0..6 {
        gateway.send(support::prompt(&gateway, "read")).unwrap();
        support::end(&mut sub).await;
    }
    let begin = requests.lock().unwrap().len();
    let current = format!("CURRENT {}", "z".repeat(12000));
    hold.store(true, Ordering::Release);
    gateway.send(support::prompt(&gateway, &current)).unwrap();
    loop {
        let e = sub.recv().await.unwrap();
        if matches!(
            e.event,
            Event::ItemStarted {
                kind: ItemKind::ToolCall,
                ..
            }
        ) {
            break;
        }
    }
    let queued = support::prompt(&gateway, "queued B");
    let cause = queued.op_id;
    gateway.send(queued).unwrap();
    let turn = loop {
        let e = sub.recv().await.unwrap();
        if e.cause == Some(cause)
            && let Event::TurnQueued { turn } = e.event
        {
            break turn;
        }
    };
    gateway
        .send(support::op(&gateway, Op::Interrupt { turn: Some(turn) }))
        .unwrap();
    loop {
        let e = sub.recv().await.unwrap();
        if matches!(e.event,Event::TurnEnded {turn:t,reason:StopReason::Cancelled,..} if t==turn) {
            break;
        }
    }
    release.notify_one();
    support::end(&mut sub).await;
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    let requests = requests.lock().unwrap();
    assert!(
        requests[begin + 1]
            .messages
            .iter()
            .any(|m| m["role"] == "user" && m["content"] == current),
        "queued cancellation compacted the running turn"
    );
    assert!(
        requests[begin + 1]
            .messages
            .iter()
            .any(|m| m["role"] == "tool")
    );
    assert_eq!(requests.len() - begin, 2);
    assert_eq!(
        requests[begin].messages[..requests[begin].stable_prefix],
        requests[begin + 1].messages[..requests[begin + 1].stable_prefix]
    );
    bboxy450_harness::protocol::laws::check_authored(&sub_log(&config)).unwrap();
}
