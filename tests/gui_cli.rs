// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "standalone")]
use std::process::Command;
#[test]
fn replay_cli_is_offline_and_invalid_inputs_fail_without_a_window() {
    let temp = tempfile::tempdir().unwrap();
    let exe = env!("CARGO_BIN_EXE_bboxy450-harness");
    let output = Command::new(exe).args(["gui", "--help"]).output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--replay"));
    let output = Command::new(exe)
        .env("HOME", temp.path())
        .args(["gui", "--headless"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires --replay"));
    let output = Command::new(exe)
        .env("HOME", temp.path())
        .args(["gui", "--replay", "missing.jsonl", "--headless"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        !temp.path().join(".bboxy450-harness").exists(),
        "offline replay never creates a session or looks up a profile"
    );
}
#[test]
fn escape_and_control_c_route_interrupt() {
    use bboxy450_harness::frontends::gui::window::is_interrupt;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    assert!(is_interrupt(
        &Key::Named(NamedKey::Escape),
        ModifiersState::empty()
    ));
    assert!(is_interrupt(
        &Key::Character("c".into()),
        ModifiersState::CONTROL
    ));
    assert!(is_interrupt(
        &Key::Character("C".into()),
        ModifiersState::CONTROL | ModifiersState::SHIFT
    ));
    assert!(!is_interrupt(
        &Key::Character("c".into()),
        ModifiersState::empty()
    ));
}
