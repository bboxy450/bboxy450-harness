// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "test-fixtures")]
mod support;
use bboxy450_harness::{
    engine::Config,
    gateway::{Gateway, Host, InProc, Subscription},
    mcp::ServerConfig,
    policy::Policy,
    protocol::*,
    provider::profile::Unattended,
    provider::{Caps, Finish, ModelEvent, ModelRequest, Provider},
    redact::Set,
};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use tokio_util::sync::CancellationToken;
struct Program {
    calls: Mutex<Vec<Vec<String>>>,
    requests: Mutex<Vec<ModelRequest>>,
    ids: AtomicU64,
}
impl Provider for Program {
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: false,
            tool_choice: true,
            context_tokens: 128000,
        }
    }
    fn stream(
        &self,
        request: ModelRequest,
        _: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        let start = request.messages.last().is_some_and(|m| m["role"] == "user");
        self.requests.lock().unwrap().push(request);
        let mut events = vec![];
        if start {
            let mut calls = self.calls.lock().unwrap();
            let names = if calls.is_empty() {
                vec![]
            } else {
                calls.remove(0)
            };
            for (index, name) in names.into_iter().enumerate() {
                let id = format!("call-{}", self.ids.fetch_add(1, Ordering::Relaxed));
                events.push(ModelEvent::ToolCallDelta {
                    index: index as u8,
                    id: Some(id.clone()),
                    name: Some(name.clone()),
                    args: "{}".into(),
                });
                events.push(ModelEvent::ToolCallReady {
                    index: index as u8,
                    id,
                    name,
                    args_raw: "{}".into(),
                    args: Ok(json!({})),
                });
            }
        }
        if events.is_empty() {
            events.push(ModelEvent::Text("done".into()));
            events.push(ModelEvent::Finish(Finish::Stop));
        } else {
            events.push(ModelEvent::Finish(Finish::ToolCalls));
        }
        Box::pin(futures_util::stream::iter(events))
    }
}
struct Harness {
    temp: tempfile::TempDir,
    host: Host,
    gateway: InProc,
    sub: Subscription,
    provider: Arc<Program>,
}
async fn start(
    mode: &str,
    trust: bool,
    program: Vec<Vec<&str>>,
    env: std::collections::BTreeMap<String, String>,
) -> Harness {
    start_policy(mode, trust, program, env, Policy::new(Unattended::Deny)).await
}
async fn start_policy(
    mode: &str,
    trust: bool,
    program: Vec<Vec<&str>>,
    env: std::collections::BTreeMap<String, String>,
    policy: Policy,
) -> Harness {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("profiles")).unwrap();
    let server = ServerConfig {
        command: env!("CARGO_BIN_EXE_mcp-fixture").into(),
        args: vec![mode.into()],
        env,
        timeout_s: if mode == "parallel" { 5 } else { 1 },
        trust_hints: trust,
    };
    let profile = json!({"name":"scripted","model":"mock","endpoint":"https://example.invalid/v1","mcpServers":{"fixture":server}});
    std::fs::write(
        temp.path().join("profiles/scripted.md"),
        format!("---\n{}\n---\n", serde_yaml::to_string(&profile).unwrap()),
    )
    .unwrap();
    let provider = Arc::new(Program {
        calls: Mutex::new(
            program
                .into_iter()
                .map(|p| p.into_iter().map(str::to_string).collect())
                .collect(),
        ),
        requests: Mutex::default(),
        ids: 0.into(),
    });
    let meta = support::meta(temp.path().into());
    let config = Config {
        meta: meta.clone(),
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider: provider.clone(),
        model: "mock".into(),
        policy,
        redactor: Set::default(),
        profile_error: None,
        max_rounds: 4,
    };
    let host = Host::new(config);
    let gateway = host
        .create(OpEnvelope {
            v: PROTOCOL,
            session: meta.session,
            op_id: OpId::new(),
            op: Op::StartSession {
                cwd: meta.cwd,
                profile: meta.profile,
                fork: None,
            },
        })
        .await
        .unwrap()
        .0;
    let mut sub = support::subscribe(&gateway, true);
    sub.set_approver(true);
    Harness {
        temp,
        host,
        gateway,
        sub,
        provider,
    }
}
async fn ready(h: &mut Harness) {
    loop {
        let event = tokio::time::timeout(std::time::Duration::from_secs(8), h.sub.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(event.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text=="mcp fixture: ready")
        {
            return;
        }
    }
}
async fn turn(h: &mut Harness, approve: bool) -> Vec<Envelope> {
    h.gateway.send(support::prompt(&h.gateway, "next")).unwrap();
    let mut events = vec![];
    loop {
        let event = tokio::time::timeout(std::time::Duration::from_secs(8), h.sub.recv())
            .await
            .expect("MCP must not block turn")
            .unwrap();
        if let Event::ItemSnapshot {
            item,
            partial: Item::Approval { .. },
            ..
        } = &event.event
        {
            let _ = h.gateway.send(support::op(
                &h.gateway,
                Op::ResolveApproval {
                    approval: *item,
                    decision: if approve {
                        ApprovalDecision::Allow
                    } else {
                        ApprovalDecision::Deny { reason: None }
                    },
                },
            ));
        }
        let end = matches!(event.event, Event::TurnEnded { .. });
        events.push(event);
        if end {
            return events;
        }
    }
}
async fn close(h: Harness) -> Vec<Envelope> {
    h.gateway.send(support::op(&h.gateway, Op::Close)).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(15), h.host.join())
        .await
        .expect("MCP close() hang guard")
        .unwrap();
    transcript(&h)
}
fn transcript(h: &Harness) -> Vec<Envelope> {
    let text = std::fs::read_to_string(bboxy450_harness::store::log_path(
        &h.temp.path().join("sessions"),
        h.gateway.session(),
    ))
    .unwrap();
    let log = text
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect::<Vec<_>>();
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    log
}
async fn group_dead(pgid: i32) -> bool {
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            if unsafe { libc::kill(-pgid, 0) } == -1 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .is_ok()
}
#[tokio::test]
async fn normalized_names_keep_the_first_collision_and_cap_at_64() {
    let long = format!("mcp__fixture__{}", "z".repeat(90))
        .chars()
        .take(64)
        .collect::<String>();
    let mut h = start(
        "names",
        true,
        vec![vec!["mcp__fixture__a_b", &long]],
        Default::default(),
    )
    .await;
    ready(&mut h).await;
    let events = turn(&mut h, false).await;
    let names = {
        let requests = h.provider.requests.lock().unwrap();
        requests[0]
            .tools
            .iter()
            .filter(|t| t.name.starts_with("mcp__"))
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names, vec!["mcp__fixture__a_b".to_owned(), long.clone()]);
    assert_eq!(long.len(), 64);
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::ToolCall {output,..},..} if output.iter().any(|b|matches!(b,OutputBlock::Text {text} if text=="a.b")))));
    let log = close(h).await;
    assert_eq!(log.iter().filter(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text.contains("tool name collision"))).count(),1);
}
#[tokio::test]
async fn interrupt_notifies_a_slow_call_and_ends_cancelled() {
    let temp = tempfile::tempdir().unwrap();
    let called = temp.path().join("called");
    let cancelled = temp.path().join("cancelled");
    let mut h = start(
        "normal",
        true,
        vec![vec!["mcp__fixture__slow"]],
        [
            ("MCP_CALLS".into(), called.to_string_lossy().into()),
            ("MCP_CANCELS".into(), cancelled.to_string_lossy().into()),
        ]
        .into(),
    )
    .await;
    ready(&mut h).await;
    h.gateway
        .send(support::prompt(&h.gateway, "slow call"))
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        while !called.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    h.gateway
        .send(support::op(&h.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    let events = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        let mut events = vec![];
        loop {
            let e = h.sub.recv().await.unwrap();
            let end = matches!(e.event, Event::TurnEnded { .. });
            events.push(e);
            if end {
                break events;
            }
        }
    })
    .await
    .expect("MCP interrupt did not settle");
    assert!(matches!(
        events.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    ));
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        while !cancelled.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("no SDK cancellation notification");
    close(h).await;
}
#[tokio::test]
async fn crash_stderr_and_spilled_results_never_persist_the_secret() {
    let secret = "MCP_CANARY_FOR_REDACTION_908475";
    let mut h = start(
        "echo_env",
        true,
        vec![vec!["mcp__fixture__echo"], vec!["mcp__fixture__crash"]],
        [
            ("MCP_SECRET".into(), secret.into()),
            ("MCP_LARGE".into(), "large-output".into()),
        ]
        .into(),
    )
    .await;
    ready(&mut h).await;
    let first = turn(&mut h, false).await;
    assert!(
        first.iter().any(|e| matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::ToolCall { spill: Some(_), .. },
                ..
            }
        )),
        "secret-bearing output was never spilled"
    );
    let mut events = turn(&mut h, false).await;
    let notice=tokio::time::timeout(std::time::Duration::from_secs(8),async {
        while !events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text.starts_with("mcp fixture: failed"))) {events.push(h.sub.recv().await.unwrap());}
    }).await;
    assert!(
        notice.is_ok(),
        "crash notice missing; notices={:?}; calls={:?}",
        events
            .iter()
            .filter_map(|e| match &e.event {
                Event::ItemCompleted {
                    body: Item::Notice { text, .. },
                    ..
                } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>(),
        events
            .iter()
            .filter_map(|e| match &e.event {
                Event::ItemCompleted {
                    body: Item::ToolCall { name, .. },
                    status,
                    ..
                } => Some((name, status)),
                _ => None,
            })
            .collect::<Vec<_>>()
    );
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text.starts_with("mcp fixture: failed")&&text.contains("<redacted>"))),"scrubbed stderr excerpt missing");
    let root = h.temp.path().join("sessions");
    h.gateway.send(support::op(&h.gateway, Op::Close)).unwrap();
    h.host.join().await.unwrap();
    let log = transcript(&h);
    assert!(!serde_json::to_string(&log).unwrap().contains(secret));
    let mut pending = vec![root];
    let mut files = vec![];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    assert!(
        files
            .iter()
            .any(|p| p.to_string_lossy().contains("/blobs/")),
        "blob scan was empty"
    );
    for path in files {
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.windows(secret.len()).any(|b| b == secret.as_bytes()),
            "secret persisted in {}",
            path.display()
        );
    }
}
#[tokio::test]
async fn close_kills_a_server_and_member_that_ignore_eof_and_term() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("group");
    let mut h = start(
        "stubborn_group",
        true,
        vec![],
        [("MCP_GROUP".into(), path.to_string_lossy().into())].into(),
    )
    .await;
    ready(&mut h).await;
    let ids = std::fs::read_to_string(path)
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    let pgid = ids[0];
    assert_eq!(
        unsafe { libc::getpgid(ids[1]) },
        pgid,
        "member must share the server group"
    );
    let (joined, dead) = {
        let closing = h.host.close_all();
        tokio::pin!(closing);
        let joined = tokio::time::timeout(std::time::Duration::from_secs(15), &mut closing).await;
        let dead = joined.is_ok() && group_dead(pgid).await;
        if !dead {
            unsafe {
                libc::kill(-pgid, libc::SIGKILL);
            }
            if joined.is_err() {
                closing.await;
            }
        }
        (joined.is_ok(), dead)
    };
    assert!(
        joined,
        "MCP server group survived Close: Close join hang guard"
    );
    assert!(dead, "MCP server group survived Close: orphaned member");
    bboxy450_harness::protocol::laws::check_transcript(&transcript(&h)).unwrap();
}
#[tokio::test]
async fn starting_hung_server_never_blocks_a_turn() {
    let mut h = start("hang_init", false, vec![vec![]], Default::default()).await;
    let events = turn(&mut h, false).await;
    assert!(matches!(
        events.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    ));
    let log = close(h).await;
    assert!(!log.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text=="mcp fixture: ready")));
}
#[tokio::test]
async fn close_cancels_a_hung_tools_relist_and_reaps_the_server() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("relist");
    let pids = temp.path().join("pids");
    let mut h = start(
        "hang_relist",
        true,
        vec![vec!["mcp__fixture__change"]],
        [
            ("MCP_RELIST".into(), marker.to_string_lossy().into()),
            ("MCP_SPAWNS".into(), pids.to_string_lossy().into()),
        ]
        .into(),
    )
    .await;
    ready(&mut h).await;
    turn(&mut h, false).await;
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        while !marker.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("fixture never received the re-list request");
    let stopped = {
        let closing = h.host.close_all();
        tokio::pin!(closing);
        let stopped = tokio::time::timeout(std::time::Duration::from_secs(15), &mut closing).await;
        if stopped.is_err() {
            closing.await;
        }
        stopped
    };
    assert!(
        stopped.is_ok(),
        "Close waited for the hung tools/list request"
    );
    let log = transcript(&h);
    assert!(matches!(log.last().unwrap().event, Event::SessionClosed));
    for pgid in std::fs::read_to_string(pids)
        .unwrap()
        .lines()
        .map(|s| s.parse::<i32>().unwrap())
    {
        assert!(group_dead(pgid).await, "fixture group survived Close");
    }
}
#[tokio::test]
async fn timeout_is_labelled_cancelled_and_turn_continues() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cancelled");
    let mut h = start(
        "normal",
        true,
        vec![vec!["mcp__fixture__slow"]],
        [("MCP_CANCELS".into(), path.to_string_lossy().into())].into(),
    )
    .await;
    ready(&mut h).await;
    let events = turn(&mut h, true).await;
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::ToolCall {output,..},status:ItemStatus::Failed {..},..} if serde_json::to_string(output).unwrap().contains("mcp fixture: timed out after 1 s"))));
    assert!(matches!(
        events.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    ));
    let log = close(h).await;
    assert_eq!(std::fs::read_to_string(path).unwrap(), "cancelled");
    assert_eq!(log.iter().filter(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text=="mcp fixture: timed out after 1 s")).count(),1);
}
#[tokio::test]
async fn crash_mid_call_is_failed_and_turn_continues() {
    let mut h = start(
        "normal",
        true,
        vec![vec!["mcp__fixture__crash"]],
        Default::default(),
    )
    .await;
    ready(&mut h).await;
    let events = turn(&mut h, true).await;
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {status:ItemStatus::Failed {..},body:Item::ToolCall {output,..},..} if serde_json::to_string(output).unwrap().contains("server exited"))));
    assert!(matches!(
        events.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    ));
    close(h).await;
}
#[tokio::test]
async fn readonly_concurrency_is_separate_from_untrusted_policy() {
    let mut h = start(
        "parallel",
        false,
        vec![
            vec!["mcp__fixture__read", "mcp__fixture__read"],
            vec!["mcp__fixture__write"],
        ],
        Default::default(),
    )
    .await;
    ready(&mut h).await;
    let first = turn(&mut h, true).await;
    let starts = first
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::ToolCall,
                    ..
                }
            )
            .then_some(i)
        })
        .collect::<Vec<_>>();
    let end = first
        .iter()
        .position(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::ToolCall { .. },
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(starts.len(), 2);
    assert_eq!(
        first
            .iter()
            .filter(|e| matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::ToolCall { .. },
                    status: ItemStatus::Completed,
                    ..
                }
            ))
            .count(),
        2,
        "both calls must cross the server-side two-party barrier"
    );
    assert!(
        starts[1] < end,
        "both calls must start before either completes"
    );
    assert_eq!(
        first
            .iter()
            .filter(|e| matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::Approval,
                    ..
                }
            ))
            .count(),
        2
    );
    let second = turn(&mut h, false).await;
    assert!(second.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            status: ItemStatus::Failed { .. },
            body: Item::ToolCall { .. },
            ..
        }
    )));
    close(h).await;
}
#[tokio::test]
async fn trusted_read_has_no_question_and_secrets_are_scrubbed() {
    let secret = "fixture-private-value-928473";
    let mut h = start(
        "echo_env",
        true,
        vec![vec!["mcp__fixture__echo"]],
        [("MCP_SECRET".into(), secret.into())].into(),
    )
    .await;
    ready(&mut h).await;
    let events = turn(&mut h, false).await;
    assert!(!events.iter().any(|e| matches!(
        e.event,
        Event::ItemStarted {
            kind: ItemKind::Approval,
            ..
        }
    )));
    let log = close(h).await;
    assert!(!serde_json::to_string(&log).unwrap().contains(secret));
    assert!(serde_json::to_string(&log).unwrap().contains("<redacted>"));
}
#[tokio::test]
async fn changed_list_waits_for_next_turn_snapshot() {
    let mut h = start(
        "list_changed",
        true,
        vec![vec!["mcp__fixture__change"]],
        Default::default(),
    )
    .await;
    ready(&mut h).await;
    turn(&mut h, false).await;
    {
        let requests = h.provider.requests.lock().unwrap();
        assert!(
            requests[0]
                .tools
                .iter()
                .any(|t| t.name == "mcp__fixture__read")
        );
        assert_eq!(
            serde_json::to_vec(&requests[0].tools).unwrap(),
            serde_json::to_vec(&requests[1].tools).unwrap(),
            "a turn's snapshot changed mid-turn"
        );
    }
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            turn(&mut h, false).await;
            let changed = h
                .provider
                .requests
                .lock()
                .unwrap()
                .last()
                .unwrap()
                .tools
                .iter()
                .any(|t| t.name == "mcp__fixture__later");
            if changed {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("list change was never visible to a later turn");
    close(h).await;
}

#[tokio::test]
async fn every_trusted_read_obeys_deny_and_explicit_ask() {
    use bboxy450_harness::policy::{Rule, RuleDecision};
    for ask in [false, true] {
        let policy = Policy {
            rules: vec![Rule {
                id: "explicit-read".into(),
                tool: "*".into(),
                effect: None,
                decision: if ask {
                    RuleDecision::Ask
                } else {
                    RuleDecision::Deny
                },
            }],
            on_unattended: Unattended::Deny,
        };
        let mut h = start_policy(
            "normal",
            true,
            vec![vec!["mcp__fixture__read"]],
            Default::default(),
            policy,
        )
        .await;
        ready(&mut h).await;
        let events = turn(&mut h, ask).await;
        if ask {
            assert!(events.iter().any(|e| matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::Approval {
                        subject: ApprovalSubject {
                            effect: Effect::Read,
                            ..
                        },
                        ..
                    },
                    ..
                }
            )));
        }
        assert!(
            events.iter().any(|e| matches!(
                e.event,
                Event::ItemCompleted {
                    status: ItemStatus::Failed { .. },
                    body: Item::ToolCall { .. },
                    ..
                }
            )) != ask
        );
        close(h).await;
    }
}
#[tokio::test]
async fn five_failed_attempts_are_terminal_and_close_reaps_each_group() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("spawns");
    let mut h = start(
        "init_crash",
        false,
        vec![],
        [("MCP_SPAWNS".into(), path.to_string_lossy().into())].into(),
    )
    .await;
    tokio::time::timeout(std::time::Duration::from_secs(40),async {loop {let event=h.sub.recv().await.unwrap();if matches!(&event.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text=="mcp fixture: dead"){break;}}}).await.expect("five-attempt lifecycle did not finish");
    let pids = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(pids.len(), 5);
    close(h).await;
    for pid in pids {
        assert_eq!(unsafe { libc::kill(-pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}
#[tokio::test]
async fn a_lying_write_hint_needs_explicit_trust_to_bypass_the_question() {
    for trust in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("writes");
        let mut h = start(
            "liar",
            trust,
            vec![vec!["mcp__fixture__write"]],
            [("MCP_WRITE_COUNT".into(), path.to_string_lossy().into())].into(),
        )
        .await;
        ready(&mut h).await;
        let events = turn(&mut h, false).await;
        assert_eq!(path.exists(), trust);
        assert_eq!(
            events.iter().any(|e| matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::Approval,
                    ..
                }
            )),
            !trust
        );
        close(h).await;
    }
}
#[tokio::test]
async fn reserved_capability_is_removed_after_configured_environment_overrides() {
    let mut h = start(
        "check_env",
        true,
        vec![vec!["mcp__fixture__echo"]],
        [("BBOXY_CAPABILITY".into(), "throwaway-fixture-value".into())].into(),
    )
    .await;
    ready(&mut h).await;
    let events = turn(&mut h, false).await;
    let output = serde_json::to_string(&events).unwrap();
    assert!(output.contains("capability absent"));
    assert!(!output.contains("capability present"));
    close(h).await;
}

#[tokio::test]
async fn mcp_finish_already_fenced_child_is_idempotent_and_keeps_foreign_group_safe() {
    let temp = tempfile::tempdir().unwrap();
    let result = bboxy450_harness::mcp::fixtures::finish_with_fenced_child_or_failed_handoff(
        std::path::Path::new(env!("CARGO_BIN_EXE_mcp-fixture")),
        temp.path(),
        true,
    )
    .await;
    assert!(
        result.missing_id,
        "fixture did not force child.id() == None"
    );
    assert!(!result.failed_handoff);
    assert_eq!(
        result.late_signals, 0,
        "already-fenced cleanup signalled a foreign pgid after reap"
    );
    assert!(result.foreign_alive);
    assert_eq!(result.term_signals, 0, "already-fenced cleanup sent TERM");
    assert_eq!(
        result.grace_sleeps, 0,
        "already-fenced cleanup slept through grace"
    );
    assert!(
        result.members_dead,
        "fenced fixture did not retire its member"
    );
}
#[tokio::test]
async fn mcp_finish_failed_handoff_keeps_members_and_foreign_groups_safe() {
    let temp = tempfile::tempdir().unwrap();
    let result = bboxy450_harness::mcp::fixtures::finish_with_fenced_child_or_failed_handoff(
        std::path::Path::new(env!("CARGO_BIN_EXE_mcp-fixture")),
        temp.path(),
        false,
    )
    .await;
    assert!(
        result.failed_handoff,
        "fixture did not force a receiver error"
    );
    assert_eq!(
        result.late_signals, 0,
        "failed-handoff cleanup signalled a foreign pgid after reap"
    );
    assert!(result.foreign_alive);
    assert!(
        result.members_dead,
        "failed-handoff cleanup left a stubborn member alive"
    );
}

#[tokio::test]
async fn mcp_finish_exited_leader_skips_term_and_grace_but_kills_members() {
    let temp = tempfile::tempdir().unwrap();
    let result = bboxy450_harness::mcp::fixtures::finish_exited_leader(
        std::path::Path::new(env!("CARGO_BIN_EXE_mcp-fixture")),
        temp.path(),
    )
    .await;
    assert!(
        result.member_present_before_finish,
        "finish_child had no member to clean up"
    );
    assert!(!result.missing_id && !result.failed_handoff);
    assert_eq!(
        result.term_signals, 0,
        "already-exited MCP cleanup sent TERM"
    );
    assert_eq!(
        result.grace_sleeps, 0,
        "already-exited MCP cleanup slept through grace"
    );
    assert!(
        result.members_dead,
        "already-exited MCP cleanup left a member alive"
    );
    assert_eq!(
        result.late_signals, 0,
        "already-exited MCP cleanup signalled a foreign pgid"
    );
    assert!(result.foreign_alive);
}
