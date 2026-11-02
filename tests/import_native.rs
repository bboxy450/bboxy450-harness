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
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    let items = completed(&log);
    assert!(
        matches!(&items[0].1, Item::UserMessage { blocks } if matches!(&blocks[..], [InputBlock::Text { text }] if text == "Find the flaky test."))
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
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
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
    bboxy450_harness::protocol::laws::check_transcript(&watched).unwrap();
    bboxy450_harness::protocol::laws::check_transcript(&forked).unwrap();
    assert!(watched.iter().any(|e| matches!(&e.event, Event::OpRejected { reason: Rejection::InvalidInput { detail }, .. } if detail.contains("/profile"))));
    assert!(completed(&forked).iter().any(|(_, b)| matches!(b, Item::AgentMessage { markdown }
        if markdown.starts_with("ECHO:") && markdown.contains("[user] Find the flaky test.")
            && markdown.contains("[tool Bash]") && markdown.contains("[assistant] The build fails before any test runs.") && markdown.contains("what failed?"))),
        "{:?}", completed(&forked));
}
