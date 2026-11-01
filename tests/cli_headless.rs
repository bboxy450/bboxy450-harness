// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::{Envelope, Event, StopReason, laws::check_transcript};
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
    check_transcript(&events).unwrap();
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
