// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::{Envelope, Event, StopReason, laws::check_authored};
#[test]
fn binary_emits_protocol_and_edits_unattended() {
    let t = tempfile::Builder::new()
        .prefix("harness-")
        .tempdir_in("/tmp")
        .unwrap();
    std::fs::write(t.path().join("input.txt"), "before\n").unwrap();
    std::fs::write(t.path().join("other.txt"), "other\n").unwrap();
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args([
            "-p",
            "Read both files and edit input.txt",
            "--json",
            "--yolo",
            "--cwd",
        ])
        .arg(t.path())
        .arg("--sessions")
        .arg(t.path().join("sessions"))
        .arg("--scripted")
        .arg(fixtures.join("tools.sse"))
        .arg("--scripted")
        .arg(fixtures.join("edit.sse"))
        .arg("--scripted")
        .arg(fixtures.join("text.sse"))
        .env("HOME", t.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .collect::<Vec<_>>();
    check_authored(&events).unwrap();
    assert!(
        matches!(&events[0].event, Event::SessionStarted { meta } if meta.vcs.is_none()),
        "a non-repository must not acquire invented VCS data"
    );
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert_eq!(
        std::fs::read_to_string(t.path().join("input.txt")).unwrap(),
        "after\n"
    );
}

#[test]
fn closed_json_replay_does_not_resolve_credentials_or_change_the_log() {
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let mut events = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Envelope>(line).unwrap())
        .filter(|e| e.event.is_durable())
        .collect::<Vec<_>>();
    let mut closed = events.last().unwrap().clone();
    closed.seq = Some(bboxy450_harness::protocol::Seq(closed.seq.unwrap().0 + 1));
    closed.at = closed.seq.unwrap();
    closed.cause = None;
    closed.event = Event::SessionClosed;
    events.push(closed);
    let session = events[0].session;
    let sessions = t.path().join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    let log = sessions.join(format!("{session}.jsonl"));
    let original = events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap() + "\n")
        .collect::<String>();
    std::fs::write(&log, &original).unwrap();
    let profiles = t.path().join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let touched = t.path().join("credential-was-read");
    std::fs::write(profiles.join("nemotron.md"),format!("---\nname: nemotron\nmodel: unused\nendpoint: http://127.0.0.1:9/v1\napi_key: '!touch {}; printf secret'\n---\n",touched.display())).unwrap();
    let bin = t.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let git_marker = t.path().join("git-was-read");
    let git = bin.join("git");
    std::fs::write(
        &git,
        "#!/bin/sh\nprintf touched > \"$VCS_GIT_MARKER\"\nexit 72\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .env("PATH", &bin)
        .env("VCS_GIT_MARKER", &git_marker)
        .args(["--json", "--resume", &session.to_string(), "--sessions"])
        .arg(sessions)
        .env("HOME", t.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, original.as_bytes());
    assert_eq!(std::fs::read(log).unwrap(), original.as_bytes());
    assert!(!touched.exists());
    assert!(!git_marker.exists());
    assert!(!t.path().join(".bboxy450-harness/run").exists());
}
