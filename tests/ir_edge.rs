// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::import::{Native, edge::remainder};
use serde_json::json;

#[test]
fn recursive_consumption_retains_native_extra_keys_and_array_positions() {
    let claude = json!({"type":"assistant","uuid":"a","parentUuid":"p","timestamp":"2026-11-02T10:00:00Z",
        "message":{"role":"assistant","content":[{"type":"text","text":"known","future":{"nested":7}},
            {"type":"new_block","opaque":[1,2]}],"new_message_key":false},"vendor":true});
    let kept = remainder(Native::ClaudeCode, &claude);
    assert_eq!(
        kept["message"]["content"][0]["future"],
        claude["message"]["content"][0]["future"]
    );
    assert_eq!(
        kept["message"]["content"][1],
        claude["message"]["content"][1]
    );
    assert_eq!(kept["message"]["new_message_key"], false);
    assert_eq!(kept["vendor"], true);
    assert!(kept.get("uuid").is_none());
    assert!(kept.get("timestamp").is_none());
    let codex = json!({"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"known","citations":[{"future_url":"https://example.invalid"}]}],"future_flag":3}});
    let kept = remainder(Native::Codex, &codex);
    assert_eq!(
        kept["payload"]["content"][0]["citations"],
        codex["payload"]["content"][0]["citations"]
    );
    assert_eq!(kept["payload"]["future_flag"], 3);
}
