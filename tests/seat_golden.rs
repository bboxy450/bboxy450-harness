// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::*;
#[test]
fn fixtures_are_append_only_and_escape_safe() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in std::fs::read_dir(root.join("tests/fixtures/protocol")).unwrap() {
        let file = file.unwrap().path();
        let events = std::fs::read_to_string(&file)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_transcript(&events).unwrap();
        let mut projection = bboxy450_harness::frontends::seat::LineProjection::default();
        let bytes = events
            .iter()
            .flat_map(|e| projection.render(e))
            .collect::<Vec<_>>();
        assert!(!bytes.contains(&0x1b));
        let path = root
            .join("tests/fixtures/seat")
            .join(file.file_stem().unwrap())
            .with_extension("txt");
        if std::env::var_os("UPDATE_GOLDENS").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &bytes).unwrap();
        }
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
