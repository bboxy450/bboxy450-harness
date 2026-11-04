// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![allow(dead_code, clippy::disallowed_methods)] // Private process fixtures, never the UI thread.
use std::{
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
pub fn home() -> tempfile::TempDir {
    let root = tempfile::Builder::new()
        .prefix("bb-onboard-")
        .tempdir_in("/tmp")
        .unwrap();
    let bin = root.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let manager = bin.join("npm");
    std::fs::write(&manager,"#!/bin/sh\n[ \"$*\" = 'install --global @agentclientprotocol/codex-acp@2.5.0' ] || exit 9\nprintf '%s\\n' \"$*\" > \"$HOME/installer-argv\"\nprintf '#!/bin/sh\\nexit 0\\n' > \"$HOME/bin/codex-acp\"\n/bin/chmod 700 \"$HOME/bin/codex-acp\"\necho PRIVATE-INSTALL-SUCCEEDED\n").unwrap();
    std::fs::set_permissions(manager, std::fs::Permissions::from_mode(0o700)).unwrap();
    root
}
pub fn command(program: &Path, home: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .env_clear()
        .env("HOME", home)
        .env("PATH", home.join("bin"))
        .env("RUST_TEST_THREADS", "1")
        .env("BBOXY_REDUCED_MOTION", "1")
        .current_dir(home);
    for name in ["ONBOARDING_EVIDENCE_DIR", "ONBOARDING_CAPTURE_DIR"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}
pub fn run(command: &mut Command) -> Output {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id(); // Only this owned, recorded child may be killed on deadline.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut timed_out = false;
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        !timed_out,
        "private child {pid} timed out: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
pub fn evidence(name: &str, value: serde_json::Value) {
    if let Some(root) = std::env::var_os("ONBOARDING_EVIDENCE_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(name), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
}
