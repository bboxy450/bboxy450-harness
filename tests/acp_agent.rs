// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! The ACP agent backend through the real Host, against a scripted ACP agent
//! (tests/fixtures/acp-agent/fake_agent.py). Every log passes check_transcript.
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
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/acp-agent/fake_agent.py");
    std::fs::write(
        profiles.join("fake.md"),
        format!(
            "---\nname: fake\nprovider: acp-agent\nagent:\n  command: [python3, -I, {}, {scenario}]\non_unattended: {}\n---\n",
            script.display(),
            if matches!(unattended, Unattended::Allow) { "allow" } else { "deny" }
        ),
    )
    .unwrap();
    std::fs::write(
        profiles.join("echo.md"),
        format!("---\nname: echo\nprovider: acp-agent\nagent:\n  command: [python3, -I, {}, echo]\n---\n", script.display()),
    )
    .unwrap();
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
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
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
async fn an_unknown_update_is_a_notice_and_a_malformed_line_ends_the_turn() {
    let f = fake("unknown", Unattended::Deny).await;
    let mut sub = subscribe(&f.gateway, false);
    f.gateway.send(support::prompt(&f.gateway, "?")).unwrap();
    until_end(&mut sub).await;
    let log = close(f).await;
    assert!(completed(&log).iter().any(|(_, b)| matches!(b, Item::Notice { level: Level::Warn, text } if text.contains("mystery_update"))));
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
    bboxy450_harness::protocol::laws::check_transcript(&old).unwrap();
    bboxy450_harness::protocol::laws::check_transcript(&new).unwrap();
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
