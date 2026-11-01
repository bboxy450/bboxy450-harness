// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#[path = "formal/export.rs"]
mod export;
#[path = "formal/models.rs"]
mod models;
#[test]
fn native_contracts_and_broken_controls() {
    let path = std::env::var("FORMAL_OUT").unwrap_or_else(|_| "target/formal/models".into());
    let root = std::path::Path::new(&path);
    std::fs::create_dir_all(root).unwrap();
    let mut manifest = String::new();
    for (m, expected) in models::corpus() {
        export::export(m, expected, root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}
