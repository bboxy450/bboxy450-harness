// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Exercise the single recipe owner's offline byte/custody regressions.
//! This gate does not certify a real pair or provide runtime admission.
#[test]
fn private_builder_source_and_publication_controls() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("target");
    std::fs::create_dir_all(&target).unwrap();
    let result = std::process::Command::new("/usr/bin/env")
        .args(["python3", "-I"])
        .arg(root.join("scripts/dream-proof-tools.py"))
        .arg("unit")
        .arg("--target")
        .arg(&target)
        .env_clear()
        .env("PATH", "/opt/homebrew/bin:/usr/bin:/bin")
        .env("HOME", &target)
        .env("TMPDIR", &target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "exit={:?}\n{}\n{}",
        result.status.code(),
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
