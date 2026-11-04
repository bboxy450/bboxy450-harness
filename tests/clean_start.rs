// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "standalone")]
#![allow(clippy::disallowed_methods)]
#[path = "support/clean_start.rs"]
mod fixture;
#[path = "support/onboarding_startup.rs"]
mod onboarding_startup_fixture;
use std::path::Path;
const EXE: &str = env!("CARGO_BIN_EXE_bboxy450-harness");
fn cli(home: &Path, extra: &[&str], env: Option<&str>) -> std::process::Output {
    let mut command = fixture::command(Path::new(EXE), home);
    command
        .args(["-p", "private local turn", "--json"])
        .args(extra);
    if let Some(value) = env {
        command.env("BBOXY450_HARNESS_PROFILE", value);
    }
    fixture::run(&mut command)
}
fn selected(output: &std::process::Output, name: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert!(
        events
            .iter()
            .any(|e| e.to_string().contains(&format!("\"profile\":\"{name}\""))),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("PRIVATE-CLEAN-START"));
}
#[test]
fn clean_start_cli_private_discovery_precedence_and_preserved_bytes() {
    let home = fixture::home();
    fixture::install(home.path(), true);
    selected(&cli(home.path(), &[], None), "codex");
    let root = home.path().join(".bboxy450-harness");
    let original = fixture::profile(home.path(), "codex");
    let profile = bboxy450_harness::provider::profile::Profile::parse(
        std::str::from_utf8(&original).unwrap(),
    )
    .unwrap();
    assert_eq!(profile.agent.unwrap().command, ["codex-acp"]);
    assert!(!root.join("profiles/codex-acp.md").exists());
    assert!(!root.join("default-profile").exists());
    std::fs::remove_file(root.join("profiles/codex.md")).unwrap();
    selected(&cli(home.path(), &[], None), "codex");
    assert_eq!(fixture::profile(home.path(), "codex"), original);
    assert!(!root.join("default-profile").exists());
    let mut edited = original.clone();
    edited.extend_from_slice(b"User edits remain.\n");
    std::fs::write(root.join("profiles/codex.md"), &edited).unwrap();
    for name in ["saved", "environment", "explicit"] {
        std::fs::write(root.join(format!("profiles/{name}.md")), &original).unwrap();
    }
    std::fs::write(root.join("default-profile"), " saved \n").unwrap();
    selected(&cli(home.path(), &[], None), "saved");
    selected(&cli(home.path(), &[], Some("  ")), "saved");
    selected(&cli(home.path(), &[], Some("environment")), "environment");
    selected(
        &cli(home.path(), &["--profile", "explicit"], Some("environment")),
        "explicit",
    );
    assert_eq!(
        std::fs::read(root.join("default-profile")).unwrap(),
        b" saved \n"
    );
    assert_eq!(fixture::profile(home.path(), "codex"), edited);
    let output = cli(home.path(), &["--profile", "missing"], None);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--profile NAME"));
    std::fs::write(root.join("default-profile"), "missing-saved\n").unwrap();
    let output = cli(home.path(), &[], None);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing-saved"));
    assert_eq!(
        std::fs::read(root.join("default-profile")).unwrap(),
        b"missing-saved\n"
    );
    let argv = std::fs::read_to_string(home.path().join("adapter-argv")).unwrap();
    assert_eq!(argv.lines().count(), 6);
    for line in argv.lines() {
        let args: Vec<String> = serde_json::from_str(line).unwrap();
        assert_eq!(Path::new(&args[0]).file_name().unwrap(), "codex-acp");
        assert_eq!(args.len(), 1);
    }
}
#[test]
fn clean_start_no_agent_and_native_without_wrapper_are_actionable() {
    for native in [false, true] {
        let home = fixture::home();
        if native {
            fixture::install(home.path(), false);
        }
        let output = cli(home.path(), &[], None);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("gui for setup")
                && error.contains("codex-acp")
                && !error.contains("nemotron"),
            "{error}"
        );
        assert!(!home.path().join("adapter-argv").exists());
        assert!(
            !home
                .path()
                .join(".bboxy450-harness/default-profile")
                .exists()
        );
    }
}
#[test]
fn clean_start_native_gui_normal_entrypoint() {
    for installed in [false, true] {
        let home = fixture::home();
        if installed {
            fixture::install(home.path(), true);
        }
        let mut command = fixture::command(Path::new(EXE), home.path());
        command.arg("gui").env("BBOXY450_HARNESS_SMOKE_WINDOW", "1");
        let output = fixture::run(&mut command);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{error}");
        assert!(error.contains("visible=false"), "{error}");
        assert!(
            error.contains(if installed {
                "profiles=1"
            } else {
                "setup=true; profiles=0"
            }),
            "{error}"
        );
        if installed {
            assert!(
                !home.path().join("adapter-argv").exists(),
                "idle startup must not launch an agent"
            );
        }
    }
}
#[test]
fn clean_start_concurrent_create_new_and_repeat() {
    use bboxy450_harness::onboarding::{Environment, discover};
    let home = fixture::home();
    fixture::install(home.path(), true);
    let env = Environment {
        home: home.path().into(),
        profiles: home.path().join(".bboxy450-harness/profiles"),
        path: home.path().join("bin").into_os_string(),
        os: "macos".into(),
    };
    let threads: Vec<_> = (0..6)
        .map(|_| {
            let env = env.clone();
            std::thread::spawn(move || discover(&env))
        })
        .collect();
    for thread in threads {
        assert!(thread.join().unwrap().error.is_none());
    }
    let bytes = fixture::profile(home.path(), "codex");
    discover(&env);
    assert_eq!(fixture::profile(home.path(), "codex"), bytes);
    assert_eq!(std::fs::read_dir(env.profiles).unwrap().count(), 1);
}

#[test]
fn clean_start_seat_normal_entrypoint_uses_discovered_codex() {
    use std::{
        process::Stdio,
        time::{Duration, Instant},
    };
    let home = fixture::home();
    fixture::install(home.path(), true);
    let socket = home.path().join("seat.sock");
    let mut command = fixture::command(Path::new(EXE), home.path());
    let mut child = command
        .args(["seat", "--json", "--socket"])
        .arg(&socket)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !socket.exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "private seat {pid} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(child.stdin.take());
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            panic!("private seat {pid} did not close");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"profile\":\"codex\""));
    assert!(
        !home.path().join("adapter-argv").exists(),
        "idle startup must not launch an agent"
    );
}
