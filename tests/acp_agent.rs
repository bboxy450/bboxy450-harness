// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! The ACP agent backend through the real Host, against a scripted ACP agent
//! (tests/fixtures/acp-agent/fake_agent.py). Every log passes check_authored.
#![cfg(feature = "acp-agent")]
mod support;
use bboxy450_harness::{
    engine::Config,
    gateway::{Gateway, Host, InProc, Subscribe, Subscription},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
};
use std::{sync::Arc, time::Duration};

struct Fake {
    temp: tempfile::TempDir,
    host: Host,
    gateway: InProc,
    session: SessionId,
}
async fn fake(scenario: &str, unattended: Unattended) -> Fake {
    fake_command(scenario, unattended, None).await
}
async fn fake_command(
    scenario: &str,
    unattended: Unattended,
    command: Option<Vec<String>>,
) -> Fake {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/acp-agent/fake_agent.py");
    let command = command.unwrap_or_else(|| {
        vec![
            "python3".into(),
            "-I".into(),
            script.display().to_string(),
            scenario.into(),
        ]
    });
    let command = serde_json::to_string(&command).unwrap();
    std::fs::write(
        profiles.join("fake.md"),
        format!(
            "---\nname: fake\nprovider: acp-agent\nagent:\n  command: {command}\non_unattended: {}\n---\n",
            if matches!(unattended, Unattended::Allow) { "allow" } else { "deny" }
        ),
    )
    .unwrap();
    std::fs::write(
        profiles.join("echo.md"),
        format!("---\nname: echo\nprovider: acp-agent\nagent:\n  command: [python3, -I, {}, echo]\n---\n", script.display()),
    )
    .unwrap();
    // The wake witnesses exercise profile yolo independently of unattended deny.
    if matches!(scenario, "wake" | "wake-twice") {
        let path = profiles.join("fake.md");
        let profile = std::fs::read_to_string(&path)
            .unwrap()
            .replace("provider: acp-agent", "provider: acp-agent\npolicy: yolo");
        std::fs::write(path, profile).unwrap();
    }
    let session = SessionId::new();
    let mut meta = support::meta(temp.path().into());
    meta.session = session;
    let host = Host::new(Config {
        meta,
        sessions: temp.path().join("sessions"),
        profiles,
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 2,
        profile_error: None,
    });
    let (gateway, _) = host
        .create(OpEnvelope {
            v: PROTOCOL,
            session,
            op_id: OpId::new(),
            op: Op::StartSession {
                cwd: temp.path().into(),
                profile: Some("fake".into()),
                fork: None,
            },
        })
        .await
        .unwrap();
    Fake {
        temp,
        host,
        gateway,
        session,
    }
}
fn subscribe(g: &InProc, approver: bool) -> Subscription {
    let mut sub = g
        .subscribe(Subscribe {
            session: g.hello().meta.session,
            after: None,
            ephemeral: true,
        })
        .unwrap();
    sub.set_approver(approver);
    sub
}
async fn next(sub: &mut Subscription) -> Envelope {
    tokio::time::timeout(Duration::from_secs(10), sub.recv())
        .await
        .expect("event timed out")
        .expect("stream closed")
}
async fn until_end(sub: &mut Subscription) -> Vec<Envelope> {
    let mut out = vec![];
    loop {
        let e = next(sub).await;
        let done = matches!(e.event, Event::TurnEnded { .. });
        out.push(e);
        if done {
            return out;
        }
    }
}
/// Close the session; return its durable log, checked against the transcript laws.
async fn close(f: Fake) -> Vec<Envelope> {
    f.gateway.send(support::op(&f.gateway, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(10), f.host.join())
        .await
        .unwrap()
        .unwrap();
    let path = f
        .temp
        .path()
        .join("sessions")
        .join(format!("{}.jsonl", f.session));
    let log: Vec<Envelope> = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
    log
}
fn completed(log: &[Envelope]) -> Vec<(ItemStatus, Item)> {
    log.iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted { status, body, .. } => Some((status.clone(), body.clone())),
            _ => None,
        })
        .collect()
}
fn ended(log: &[Envelope]) -> Vec<StopReason> {
    log.iter()
        .filter_map(|e| match &e.event {
            Event::TurnEnded { reason, .. } => Some(reason.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn bare_agent_uses_login_shell_path_and_cached_fallback() {
    const CHILD: &str = "HARNESS_PATH_WITNESS";
    if let Ok(case) = std::env::var(CHILD) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/acp-agent/fake_agent.py");
            let program = if case == "absolute" {
                std::env::var("WITNESS_AGENT").unwrap()
            } else {
                "fixture-agent".into()
            };
            let command = vec![
                program,
                "-I".into(),
                script.display().to_string(),
                "text".into(),
            ];
            let a = fake_command("text", Unattended::Deny, Some(command.clone())).await;
            let b = fake_command("text", Unattended::Deny, Some(command)).await;
            let mut sa = subscribe(&a.gateway, false);
            let mut sb = subscribe(&b.gateway, false);
            a.gateway.send(support::prompt(&a.gateway, "hi")).unwrap();
            b.gateway.send(support::prompt(&b.gateway, "hi")).unwrap();
            tokio::join!(until_end(&mut sa), until_end(&mut sb));
            for log in [close(a).await, close(b).await] {
                assert_eq!(ended(&log), vec![StopReason::EndTurn], "{log:?}");
                assert!(completed(&log).iter().any(|(_, body)| matches!(body,
                    Item::AgentMessage { markdown } if markdown == "Hello from the fake agent.")));
            }
        });
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    let python = std::process::Command::new("python3")
        .args(["-I", "-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap();
    assert!(python.status.success());
    let python = String::from_utf8(python.stdout).unwrap();
    for case in ["login", "missing", "failed", "empty", "timeout", "absolute"] {
        let home = tempfile::tempdir_in("/tmp").unwrap();
        let bin = home.path().join(if case == "login" || case == "absolute" {
            "login bin"
        } else if case == "empty" {
            ".cargo/bin"
        } else {
            ".local/bin"
        });
        std::fs::create_dir_all(&bin).unwrap();
        let agent = bin.join("fixture-agent");
        let interpreter = if case == "absolute" {
            python.trim()
        } else {
            "path-python"
        };
        std::fs::write(&agent, format!("#!/bin/sh\nprintf '%s\\n' \"$PATH\" >> \"$HOME/child-path\"\nexec {interpreter} \"$@\"\n")).unwrap();
        std::fs::set_permissions(&agent, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink(python.trim(), bin.join("path-python")).unwrap();
        let shell = home.path().join("login-shell");
        let path = format!("{}:/usr/bin:/bin", bin.display());
        if case != "missing" {
            let body = match case {
                "failed" => "exit 7\n",
                "empty" => "exit 0\n",
                "timeout" => "echo $$ > \"$HOME/shell-pid\"\nexec /bin/sleep 30\n",
                _ => "printf 'startup banner\\n%s' \"$WITNESS_LOGIN_PATH\"\n",
            };
            std::fs::write(&shell, format!("#!/bin/sh\n[ \"$1\" = -ilc ] && [ \"$2\" = 'printf %s \"$PATH\"' ] || exit 19\nprintf 'probe\\n' >> \"$HOME/probes\"\n{body}")).unwrap();
            std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        // Two sessions race the once-only probe in a fresh process. No test
        // changes the parent process's environment or consults its real shell.
        let started = std::time::Instant::now();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "bare_agent_uses_login_shell_path_and_cached_fallback",
                "--nocapture",
            ])
            .env_clear()
            .env(CHILD, case)
            .env("HOME", home.path())
            .env("PATH", "/usr/bin:/bin")
            .env("SHELL", &shell)
            .env("WITNESS_LOGIN_PATH", &path)
            .env("WITNESS_AGENT", &agent)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{case}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "{case}: shell deadline did not bound startup"
        );
        let probes = std::fs::read_to_string(home.path().join("probes")).unwrap_or_default();
        assert_eq!(
            probes.lines().count(),
            usize::from(!["missing", "absolute"].contains(&case)),
            "{case}: probe was not cached"
        );
        let child_path = std::fs::read_to_string(home.path().join("child-path")).unwrap();
        assert_eq!(child_path.lines().count(), 2);
        for child_path in child_path.lines() {
            if case == "absolute" {
                assert_eq!(child_path, "/usr/bin:/bin");
            } else if case == "login" {
                assert_eq!(child_path, path);
            } else {
                for dir in [
                    "/opt/homebrew/bin".into(),
                    "/usr/local/bin".into(),
                    home.path().join(".local/bin"),
                    home.path().join(".cargo/bin"),
                ] {
                    assert!(
                        std::env::split_paths(child_path).any(|p| p == dir),
                        "{case}: missing {}",
                        dir.display()
                    );
                }
            }
        }
        if case == "timeout" {
            let pid: i32 = std::fs::read_to_string(home.path().join("shell-pid"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "timed-out shell survived"
            );
        }
    }
}

#[tokio::test]
async fn a_text_turn_maps_thought_message_and_plan_and_ends_once() {
    let f = fake("text", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "hi")).unwrap();
    let live = until_end(&mut sub).await;
    // a turn this short completes before the 50 ms delta flush; live streaming is witnessed by the cancel test
    let _ = live;
    let log = close(f).await;
    let items = completed(&log);
    assert!(items.iter().any(
        |(_, b)| matches!(b, Item::Reasoning { summary, .. } if summary == "Thinking about it.")
    ));
    assert!(items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "Hello from the fake agent.")));
    assert!(items.iter().any(|(_, b)| matches!(b, Item::Plan { entries } if entries.len() == 1 && entries[0].text == "say hello")));
    // metadata updates are kept by the agent, not shown as notices
    assert!(!items.iter().any(|(_, b)| matches!(b, Item::Notice { .. })));
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);
    assert!(log.iter().any(|e| matches!(&e.event, Event::ProfileChanged { model, .. } if model.starts_with("acp:python3"))));
}

#[tokio::test]
async fn a_face_approves_a_permission_and_unsupported_requests_get_32601() {
    let f = fake("tool", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, true);
    f.gateway
        .send(support::prompt(&f.gateway, "run ls"))
        .unwrap();
    // answer the approval as a face would
    loop {
        let e = next(&mut sub).await;
        if let Event::ItemSnapshot {
            item,
            partial: Item::Approval { options, .. },
        } = &e.event
        {
            assert_eq!(
                options,
                &vec![
                    ApprovalDecisionKind::Allow,
                    ApprovalDecisionKind::AllowAlways,
                    ApprovalDecisionKind::Deny
                ]
            );
            f.gateway
                .send(support::op(
                    &f.gateway,
                    Op::ResolveApproval {
                        approval: *item,
                        decision: ApprovalDecision::Allow,
                    },
                ))
                .unwrap();
            break;
        }
    }
    until_end(&mut sub).await;
    let log = close(f).await;
    let items = completed(&log);
    assert!(items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown.starts_with("fs answered -32601."))), "{items:?}");
    assert!(items.iter().any(|(s, b)| matches!(s, ItemStatus::Completed) && matches!(b, Item::ToolCall { name, tool_kind: ToolKind::Execute, input_raw, output, .. } if name == "Run ls" && input_raw.contains("\"ls\"") && matches!(&output[..], [OutputBlock::Text { text }] if text == "note.txt"))));
    assert!(items.iter().any(|(_, b)| matches!(
        b,
        Item::Approval {
            decision: Some(ApprovalDecision::Allow),
            by: Some(Resolver::Face { .. }),
            ..
        }
    )));
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);
}

#[tokio::test]
async fn unattended_deny_rejects_once_and_never_picks_always() {
    let f = fake("tool", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "run ls"))
        .unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    let items = completed(&log);
    assert!(items.iter().any(|(_, b)| matches!(
        b,
        Item::Approval {
            decision: Some(ApprovalDecision::Deny { .. }),
            by: Some(Resolver::Unattended),
            ..
        }
    )));
    // the agent saw the reject option, not "always"
    assert!(items.iter().any(|(s, b)| matches!(s, ItemStatus::Failed { .. }) && matches!(b, Item::ToolCall { output, .. } if matches!(&output[..], [OutputBlock::Text { text }] if text.contains("\"no\"")))));
}

#[tokio::test]
async fn interrupt_sends_cancel_keeps_late_updates_and_ends_interrupted() {
    let f = fake("cancel", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "work")).unwrap();
    loop {
        if matches!(
            next(&mut sub).await.event,
            Event::ItemSnapshot {
                partial: Item::AgentMessage { .. },
                ..
            }
        ) {
            break;
        }
    }
    let started = std::time::Instant::now();
    f.gateway
        .send(support::op(&f.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    until_end(&mut sub).await;
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "the agent ended the prompt itself, before the kill grace"
    );
    let log = close(f).await;
    assert_eq!(ended(&log), vec![StopReason::Interrupted]);
    assert!(completed(&log).iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "Working (late update after cancel)")));
}

#[tokio::test]
async fn an_agent_crash_ends_the_turn_with_its_stderr_and_the_next_turn_restarts_it() {
    let f = fake("crash", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "go")).unwrap();
    until_end(&mut sub).await;
    // the session stays usable: a new turn spawns a new agent
    f.gateway
        .send(support::prompt(&f.gateway, "again"))
        .unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    let reasons = ended(&log);
    assert_eq!(reasons.len(), 2);
    assert!(reasons.iter().all(|r| matches!(r, StopReason::Error { class: ErrorClass::MidStream, message } if message.contains("fake agent: boom"))), "{reasons:?}");
    // the partial message is kept, closed with the turn
    assert!(
        completed(&log).iter().any(
            |(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "About to fail")
        )
    );
}

#[tokio::test]
async fn an_unknown_update_is_native_and_a_malformed_line_ends_the_turn() {
    let f = fake("unknown", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "?")).unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    assert!(completed(&log).iter().any(|(_, b)| matches!(b, Item::Native {record_type,raw,..} if record_type=="mystery_update" && raw["sessionUpdate"]=="mystery_update")));
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);

    let f = fake("malformed", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "?")).unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    assert!(
        matches!(&ended(&log)[..], [StopReason::Error { message, .. }] if message.contains("malformed JSON-RPC"))
    );
}

#[tokio::test]
async fn switching_profiles_forks_the_session_and_hands_the_conversation_to_the_new_agent() {
    let f = fake("text", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "remember the word heliotrope"))
        .unwrap();
    until_end(&mut sub).await;
    drop(sub);
    let next = f.host.switch(&f.gateway, "echo".into()).await.unwrap();
    assert_ne!(next.session(), f.session);
    let mut sub = subscribe(&next, false);
    // the subscription replays the forked history first, including its TurnEnded
    until_end(&mut sub).await;
    next.send(support::prompt(&next, "what was the word?"))
        .unwrap();
    until_end(&mut sub).await;
    let child = next.session();
    next.send(support::op(&next, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(10), f.host.join())
        .await
        .unwrap()
        .unwrap();
    let read = |s: SessionId| -> Vec<Envelope> {
        std::fs::read_to_string(f.temp.path().join("sessions").join(format!("{s}.jsonl")))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    };
    let (old, new) = (read(f.session), read(child));
    bboxy450_harness::protocol::laws::check_authored(&old).unwrap();
    bboxy450_harness::protocol::laws::check_authored(&new).unwrap();
    assert!(
        matches!(old.last().map(|e| &e.event), Some(Event::SessionClosed)),
        "the old session closes"
    );
    let items = completed(&new);
    // the forked log keeps the whole conversation, then the new agent's turn
    assert!(items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown == "Hello from the fake agent.")));
    assert!(
        items.iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown } if markdown.starts_with("ECHO:") && markdown.contains("[user] remember the word heliotrope") && markdown.contains("[assistant] Hello from the fake agent.") && markdown.contains("what was the word?"))),
        "the fresh agent got the handoff: {items:?} {:?}", ended(&new)
    );
    assert!(
        new.iter().any(
            |e| matches!(&e.event, Event::ProfileChanged { profile, .. } if profile == "echo")
        )
    );
}

#[test]
fn a_profile_model_matches_the_agents_model_option_exactly_or_by_provider_suffix() {
    use bboxy450_harness::backend::acp_agent::{ModelChoice, pick_model};
    let offered = serde_json::json!({ "configOptions": [{ "id": "model", "category": "model", "currentValue": "openrouter/x:free",
        "options": [{ "value": "openrouter/x:free" }, { "value": "anthropic/claude-haiku-5-5" }] }] });
    assert!(
        matches!(pick_model(&offered, "claude-haiku-5-5"), ModelChoice::Set(v) if v == "anthropic/claude-haiku-5-5")
    );
    assert!(
        matches!(pick_model(&offered, "anthropic/claude-haiku-5-5"), ModelChoice::Set(v) if v == "anthropic/claude-haiku-5-5")
    );
    assert!(matches!(
        pick_model(&offered, "openrouter/x:free"),
        ModelChoice::Current
    ));
    assert!(
        matches!(pick_model(&offered, "gpt-9"), ModelChoice::NotOffered(Some(c)) if c == "openrouter/x:free")
    );
    assert!(matches!(
        pick_model(&serde_json::json!({}), "m"),
        ModelChoice::NotOffered(None)
    ));
}

#[test]
fn model_aliases_match_whole_tokens_after_exact_and_provider_suffix() {
    use bboxy450_harness::backend::acp_agent::{ModelChoice, pick_model};
    let offered = serde_json::json!({"configOptions": [{"id": "model", "currentValue": "sonnet",
        "options": [{"value": "haiku"}, {"value": "sonnet"}, {"value": "opus"}]}]});
    for model in ["claude-haiku-5-5", "anthropic/haiku/5", "claude.haiku.5"] {
        assert!(matches!(pick_model(&offered, model), ModelChoice::Set(v) if v == "haiku"));
    }
    assert!(matches!(
        pick_model(&offered, "claude-sonnet-5"),
        ModelChoice::Current
    ));
    for model in ["claude-haikus-5", "claudeHaiku", "notopus", ""] {
        assert!(matches!(
            pick_model(&offered, model),
            ModelChoice::NotOffered(_)
        ));
    }
    let offered = serde_json::json!({"configOptions": [{"category": "model", "currentValue": "opus",
        "options": [{"value": "haiku"}, {"value": "vendor/claude-haiku-5-5"}, {"value": "claude-haiku-5-5"}]}]});
    assert!(
        matches!(pick_model(&offered, "claude-haiku-5-5"), ModelChoice::Set(v) if v == "claude-haiku-5-5")
    );
    let offered = serde_json::json!({"configOptions": [{"category": "model", "currentValue": "opus",
        "options": [{"value": "haiku"}, {"value": "vendor/claude-haiku-5-5"}]}]});
    assert!(
        matches!(pick_model(&offered, "claude-haiku-5-5"), ModelChoice::Set(v) if v == "vendor/claude-haiku-5-5")
    );
}

#[tokio::test]
async fn usage_updates_are_session_scope_ir_with_reported_numbers_and_missing_tokens() {
    let f = fake("usage", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "usage"))
        .unwrap();
    until_end(&mut sub).await;
    let idle_item = loop {
        if let Event::ItemCompleted {
            item,
            body: Item::Usage { usage },
            ..
        } = next(&mut sub).await.event
            && usage.context_used == Some(55000)
        {
            break item;
        }
    };
    let log = close(f).await;
    let values: Vec<Usage> = completed(&log)
        .into_iter()
        .filter_map(|(_, body)| {
            if let Item::Usage { usage } = body {
                Some(usage)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(values.len(), 3);
    for usage in &values {
        assert_eq!(usage.scope, Some(UsageScope::Session));
        assert_eq!(usage.source, Some(NumberSource::Reported));
        assert_eq!(
            usage.backend,
            Some(Box::new(Backend {
                kind: BackendKind::AcpAgent,
                name: "fixture-agent".into(),
                version: Some("1.2.3".into())
            }))
        );
        assert_eq!(usage.model.as_deref(), Some("reported-model"));
        assert_eq!(usage.context_window, Some(200000));
        assert_eq!((usage.input_tokens, usage.output_tokens), (0, 0));
        assert_eq!(usage.unreported, ["input_tokens", "output_tokens"]);
        assert_eq!(usage.cached_input_tokens, None);
        assert_eq!(usage.reasoning_tokens, None);
        assert_eq!(usage.cost.as_ref().unwrap().source, NumberSource::Reported);
        assert_eq!(
            usage.cost.as_ref().unwrap().price_source.as_deref(),
            Some("provider")
        );
        assert_eq!(
            serde_json::from_value::<Usage>(serde_json::to_value(usage).unwrap()).unwrap(),
            *usage
        );
    }
    assert_eq!(
        values.iter().map(|u| u.context_used).collect::<Vec<_>>(),
        [Some(53000), Some(54000), Some(55000)]
    );
    assert_eq!(
        values[0].cost,
        Some(Box::new(Cost {
            amount_micros: 45000,
            currency: "USD".into(),
            source: NumberSource::Reported,
            price_source: Some("provider".into())
        }))
    );
    assert_eq!(values[0].cost_micros_usd, Some(45000));
    assert_eq!(values[1].cost.as_ref().unwrap().currency, "EUR");
    assert_eq!(values[1].cost.as_ref().unwrap().amount_micros, 100000);
    assert_eq!(values[1].cost_micros_usd, None);
    assert_eq!(values[2].cost_micros_usd, Some(2));
    assert!(log.iter().any(|e| matches!(e.event, Event::ItemStarted { item, turn: None, kind: ItemKind::Usage, .. } if item == idle_item)), "post-prompt accounting is an idle session item");
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);
}

#[test]
fn empty_or_invalid_usage_never_fabricates_counters_or_a_bill() {
    use bboxy450_harness::backend::acp_agent::session_usage;
    let backend = Backend {
        kind: BackendKind::AcpAgent,
        name: "test".into(),
        version: None,
    };
    for value in [
        serde_json::json!({}),
        serde_json::json!({"used":null,"size":-1}),
        serde_json::json!({"cost":{"amount":-1,"currency":"USD"}}),
        serde_json::json!({"cost":{"amount":1e100,"currency":"USD"}}),
        serde_json::json!({"cost":{"amount":1,"currency":"unknown"}}),
    ] {
        assert!(
            session_usage(&value, backend.clone(), None).is_none(),
            "{value}"
        );
    }
    let usage = session_usage(&serde_json::json!({"used":0}), backend.clone(), None).unwrap();
    assert_eq!(usage.context_used, Some(0));
    assert_eq!(usage.context_window, None);
    assert_eq!(usage.cost, None);
    assert_eq!(usage.model, None);
    let usage = session_usage(
        &serde_json::json!({"cost":{"amount":0.1,"currency":"EUR"}}),
        backend,
        None,
    )
    .unwrap();
    assert_eq!(usage.context_used, None);
    assert_eq!(usage.context_window, None);
    assert_eq!(usage.cost.unwrap().amount_micros, 100000);
    assert_eq!(usage.cost_micros_usd, None);
    assert_eq!(usage.unreported, ["input_tokens", "output_tokens"]);
}

#[tokio::test]
async fn cancel_grace_kills_an_unresponsive_agents_whole_process_group() {
    // Isolate the test override in a subprocess: no mutation of the parallel
    // test runner's environment and no dependence on an operator's profile.
    if std::env::var_os("BBOXY450_ACP_GRACE_WITNESS").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "cancel_grace_kills_an_unresponsive_agents_whole_process_group",
                "--nocapture",
            ])
            .env("BBOXY450_ACP_GRACE_WITNESS", "1")
            .env("BBOXY450_ACP_CANCEL_GRACE_MS", "50")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let f = fake("ignore-cancel", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "work")).unwrap();
    loop {
        if matches!(
            next(&mut sub).await.event,
            Event::ItemSnapshot {
                partial: Item::AgentMessage { .. },
                ..
            }
        ) {
            break;
        }
    }
    let pids: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(f.temp.path().join("agent-group.json")).unwrap(),
    )
    .unwrap();
    let group = pids["group"].as_i64().unwrap() as i32;
    assert_eq!(pids["leader"].as_i64().unwrap(), i64::from(group));
    assert_eq!(
        unsafe { libc::kill(-group, 0) },
        0,
        "the group must exist before cancel"
    );
    let started = std::time::Instant::now();
    f.gateway
        .send(support::op(&f.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    until_end(&mut sub).await;
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "test grace override was not used"
    );
    assert!(
        f.temp.path().join("cancel-seen").exists(),
        "the agent received session/cancel and ignored it"
    );
    let log = close(f).await;
    assert_eq!(ended(&log), vec![StopReason::Interrupted]);
    assert!(completed(&log).iter().any(
        |(_, body)| matches!(body, Item::Notice { level: Level::Warn, text }
        if text.contains("50ms of cancel") && text.contains("process group was killed"))
    ));
    assert!(
        completed(&log)
            .iter()
            .any(|(status, body)| matches!(status, ItemStatus::Interrupted)
                && matches!(body, Item::ToolCall { name, .. } if name == "Still working"))
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        if unsafe { libc::kill(-group, 0) } == -1 {
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "owned ACP group {group} survived kill-after-grace: {pids}"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

// Environment overrides belong to a private test process, never to parallel
// actors in this runner. These witnesses drive the real gateway and child wire.
fn isolated_wake(name: &str, hide: bool) -> bool {
    if std::env::var("BBOXY450_ACP_WAKE_WITNESS").as_deref() == Ok(name) {
        return false;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--nocapture"])
        .env("BBOXY450_ACP_WAKE_WITNESS", name)
        .env("BBOXY450_ACP_AGENT_TURN_QUIET_MS", "200");
    if hide {
        command.env("BBOXY450_ACP_IDLE_CONTROL", "hide");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
fn agent_turns(log: &[Envelope]) -> Vec<TurnId> {
    log.iter()
        .filter_map(|e| match e.event {
            Event::TurnStarted {
                turn,
                origin: Some(TurnOrigin::Agent),
                ..
            } => {
                assert_eq!(e.cause, None);
                Some(turn)
            }
            _ => None,
        })
        .collect()
}
fn assert_agent_turn(log: &[Envelope]) {
    let turns = agent_turns(log);
    assert!(!turns.is_empty(), "idle work must open an agent turn");
    let mut active = None;
    let mut open = std::collections::BTreeSet::new();
    for e in log {
        match &e.event {
            Event::TurnStarted { turn, .. } => active = Some(*turn),
            Event::ItemStarted { item, turn, .. } => {
                if active.is_some_and(|t| turns.contains(&t)) {
                    assert_eq!(*turn, active, "every agent-turn item has its owner");
                    open.insert(*item);
                }
            }
            Event::ItemCompleted { item, body, .. } => {
                open.remove(item);
                if active.is_some_and(|t| turns.contains(&t)) {
                    assert!(!matches!(body, Item::UserMessage { .. }));
                }
            }
            Event::TurnEnded {
                turn,
                reason,
                usage,
            } => {
                if turns.contains(turn) {
                    assert!(matches!(reason, StopReason::Quiet { .. }));
                    assert_eq!(*usage, None);
                    assert!(open.is_empty());
                    assert_eq!(e.cause, None);
                }
                active = None;
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn idle_work_opens_an_agent_turn_resolved_by_policy_and_ended_by_quiet() {
    let name = "idle_work_opens_an_agent_turn_resolved_by_policy_and_ended_by_quiet";
    if isolated_wake(name, false) {
        return;
    }
    for scenario in ["wake", "wake-twice"] {
        let f = fake(scenario, Unattended::Deny).await;
        let mut sub = subscribe(&f.gateway, false);
        f.gateway
            .send(support::prompt(&f.gateway, "start background work"))
            .unwrap();
        until_end(&mut sub).await; // prompted user turn
        until_end(&mut sub).await; // first unsolicited turn
        if scenario == "wake-twice" {
            until_end(&mut sub).await;
        }
        let log = close(f).await;
        assert_agent_turn(&log);
        assert_eq!(
            agent_turns(&log).len(),
            if scenario == "wake-twice" { 2 } else { 1 }
        );
        assert!(
            ended(&log)
                .iter()
                .skip(1)
                .all(|r| matches!(r, StopReason::Quiet { ms } if *ms >= 200))
        );
        assert!(completed(&log).iter().any(|(_, body)| matches!(body,
            Item::Approval { decision: Some(ApprovalDecision::Allow), by: Some(Resolver::Rule { id }), .. } if id == "profile-yolo")));
        assert!(completed(&log).iter().any(|(s, body)| *s == ItemStatus::Completed && matches!(body,
            Item::ToolCall { output, .. } if serde_json::to_string(output).unwrap().contains("allow_once"))));
        assert!(completed(&log).iter().any(
            |(_, body)| matches!(body, Item::AgentMessage { markdown } if markdown == "done")
        ));
        if scenario == "wake"
            && let Some(path) = std::env::var_os("BBOXY450_AGENT_TURN_FIXTURE_OUT")
        {
            let text = log
                .iter()
                .map(|e| serde_json::to_string(e).unwrap() + "\n")
                .collect::<String>();
            std::fs::write(path, text).unwrap();
        }
    }
}

#[tokio::test]
async fn an_idle_permission_opens_a_turn_and_unattended_deny_rejects_once() {
    let name = "an_idle_permission_opens_a_turn_and_unattended_deny_rejects_once";
    if isolated_wake(name, false) {
        return;
    }
    let f = fake("wake-permission", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "start"))
        .unwrap();
    until_end(&mut sub).await;
    until_end(&mut sub).await;
    let outcome: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(f.temp.path().join("wake-outcome.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        outcome,
        serde_json::json!({"outcome":"selected","optionId":"reject_once"})
    );
    let log = close(f).await;
    assert_agent_turn(&log);
    assert!(completed(&log).iter().any(|(_, body)| matches!(body,
        Item::Approval { decision: Some(ApprovalDecision::Deny { reason }), by: Some(Resolver::Unattended), .. } if reason.as_deref().is_some_and(|r| r.contains("on_unattended=deny")))));
}

#[tokio::test]
async fn a_queued_user_turn_shortens_quiet_only_after_the_agent_tool_and_permission_complete() {
    let name =
        "a_queued_user_turn_shortens_quiet_only_after_the_agent_tool_and_permission_complete";
    if isolated_wake(name, false) {
        return;
    }
    let f = fake("wake-permission", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, true);
    f.gateway
        .send(support::prompt(&f.gateway, "start"))
        .unwrap();
    until_end(&mut sub).await;
    let mut agent = None;
    let approval = loop {
        match next(&mut sub).await.event {
            Event::TurnStarted {
                turn,
                origin: Some(TurnOrigin::Agent),
                ..
            } => agent = Some(turn),
            Event::ItemSnapshot {
                item,
                partial: Item::Approval { .. },
            } => break item,
            _ => {}
        }
    };
    let agent = agent.expect("the idle permission opens a turn");
    f.gateway
        .send(support::prompt(&f.gateway, "next user request"))
        .unwrap();
    let queued = loop {
        match next(&mut sub).await.event {
            Event::TurnQueued { turn } => break turn,
            Event::TurnEnded { turn, .. } if turn == agent => panic!("ended with permission open"),
            _ => {}
        }
    };
    // More than one quiet interval with both a tool and a permission open.
    tokio::time::sleep(Duration::from_millis(250)).await;
    let at = std::time::Instant::now();
    f.gateway
        .send(support::op(
            &f.gateway,
            Op::ResolveApproval {
                approval,
                decision: ApprovalDecision::Allow,
            },
        ))
        .unwrap();
    let events = until_end(&mut sub).await;
    assert!(
        at.elapsed() < Duration::from_millis(200),
        "queued user waited for the quiet interval"
    );
    assert!(events.iter().any(|e| matches!(e.event, Event::TurnEnded { turn, reason: StopReason::Quiet { ms }, .. } if turn == agent && ms < 200)));
    let events = until_end(&mut sub).await;
    assert!(events.iter().any(|e| matches!(e.event, Event::TurnStarted { turn, origin: Some(TurnOrigin::User), .. } if turn == queued)));
    let log = close(f).await;
    assert_agent_turn(&log);
    let end = log
        .iter()
        .position(|e| matches!(e.event, Event::TurnEnded { turn, .. } if turn == agent))
        .unwrap();
    let start = log
        .iter()
        .position(|e| matches!(e.event, Event::TurnStarted { turn, .. } if turn == queued))
        .unwrap();
    assert!(end < start);
}

#[tokio::test]
async fn the_idle_hide_control_reproduces_hidden_work_and_cancelled_permission() {
    let name = "the_idle_hide_control_reproduces_hidden_work_and_cancelled_permission";
    if isolated_wake(name, true) {
        return;
    }
    let f = fake("wake", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "start"))
        .unwrap();
    until_end(&mut sub).await;
    loop {
        if matches!(next(&mut sub).await.event, Event::ItemCompleted { body: Item::Native { record_type, raw, .. }, .. }
            if record_type == "agent_message_chunk" && raw.pointer("/content/text").and_then(serde_json::Value::as_str) == Some("done"))
        {
            break;
        }
    }
    let outcome: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(f.temp.path().join("wake-outcome.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(outcome, serde_json::json!({"outcome":"cancelled"}));
    let log = close(f).await;
    assert!(agent_turns(&log).is_empty());
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);
    assert!(
        std::panic::catch_unwind(|| assert_agent_turn(&log)).is_err(),
        "the positive lifecycle assertion must reject the broken control"
    );
    println!(
        "BBOXY450_ACP_IDLE_CONTROL=hide: hidden work, cancelled permission; positive assertion rejected"
    );
}

#[tokio::test]
async fn idle_session_gauges_never_open_an_agent_turn() {
    let f = fake("idle-gauges", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "start"))
        .unwrap();
    until_end(&mut sub).await;
    loop {
        if matches!(next(&mut sub).await.event, Event::ItemCompleted { body: Item::Native { record_type, .. }, .. } if record_type == "session_info_update")
        {
            break;
        }
    }
    let log = close(f).await;
    assert!(agent_turns(&log).is_empty());
    assert!(log.iter().any(|e| matches!(
        e.event,
        Event::ItemStarted {
            kind: ItemKind::Usage,
            turn: None,
            ..
        }
    )));
    assert_eq!(ended(&log), vec![StopReason::EndTurn]);
}

#[tokio::test]
async fn session_gauges_rearm_the_quiet_deadline_of_an_open_agent_turn() {
    let name = "session_gauges_rearm_the_quiet_deadline_of_an_open_agent_turn";
    if isolated_wake(name, false) {
        return;
    }
    let f = fake("wake-rearm", Unattended::Allow).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::prompt(&f.gateway, "start"))
        .unwrap();
    until_end(&mut sub).await;
    let events = until_end(&mut sub).await;
    let log = close(f).await;
    assert_agent_turn(&log);
    assert_eq!(
        agent_turns(&log).len(),
        1,
        "gauges must prevent an early quiet ending"
    );
    assert_eq!(
        completed(&events)
            .iter()
            .filter(|(_, body)| matches!(body, Item::Usage { .. }))
            .count(),
        3
    );
    let first = events
        .iter()
        .find(|e| matches!(e.event, Event::TurnStarted { .. }))
        .unwrap()
        .ts_ms;
    let end = events.last().unwrap();
    assert!(end.ts_ms.saturating_sub(first) >= 650);
    assert!(
        matches!(end.event, Event::TurnEnded { reason: StopReason::Quiet { ms }, .. } if ms >= 200)
    );
}

#[tokio::test]
async fn gui_polish_real_mcp_result_survives_acp_completion() {
    let f = fake("gui-polish-mcp", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway
        .send(support::op(
            &f.gateway,
            Op::UserTurn {
                blocks: vec![InputBlock::Text {
                    text: "fixture only".into(),
                }],
                mode: TurnMode::Queue,
            },
        ))
        .unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    let tools: Vec<_> = completed(&log)
        .into_iter()
        .filter_map(|(_, item)| {
            if let Item::ToolCall {
                output, raw_output, ..
            } = item
            {
                Some((output, raw_output))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(tools.len(), 3);
    for (output, raw) in tools {
        assert!(
            matches!(&output[..], [OutputBlock::Json { value }] if Some(value) == raw.as_ref()),
            "raw result must be paired as real structured output: {output:?}"
        );
    }
}
