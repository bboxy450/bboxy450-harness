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
    assert!(String::from_utf8_lossy(&output.stdout).contains("--expand all"));
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
fn capture_expansion_is_opt_in_and_requires_all_and_capture() {
    use bboxy450_harness::frontends::gui::options::{Menu, Options};
    let parse = |args: &[&str]| Options::parse(args.iter().map(|s| s.to_string()));
    let default = parse(&["--replay", "fixture.jsonl", "--capture", "900x1100"]).unwrap();
    assert!(!default.expand_all);
    for args in [
        vec![
            "--replay",
            "fixture.jsonl",
            "--capture",
            "900x1100",
            "--expand",
            "all",
        ],
        vec![
            "--expand",
            "all",
            "--replay",
            "fixture.jsonl",
            "--capture",
            "900x1100",
        ],
    ] {
        let options = parse(&args).unwrap();
        assert!(options.expand_all && options.headless);
        assert_eq!(options.capture, Some([900, 1100]));
    }
    let menu = parse(&[
        "--replay",
        "fixture.jsonl",
        "--capture",
        "900x1100",
        "--expand",
        "all",
        "--open-menu",
        "profiles",
    ])
    .unwrap();
    assert!(menu.expand_all);
    assert_eq!(menu.open_menu, Some(Menu::Profiles));
    for (args, expected) in [
        (vec!["--expand"], "--expand takes all"),
        (vec!["--expand", "tools"], "--expand takes all"),
        (
            vec!["--replay", "fixture.jsonl", "--headless", "--expand", "all"],
            "--expand requires --capture",
        ),
    ] {
        assert_eq!(parse(&args).unwrap_err(), expected);
        let output = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .arg("gui")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
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
