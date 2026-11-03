// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub mod face_core;
pub mod headless;
#[path = "../ir_scan.rs"]
pub mod ir_scan;
pub mod seat;
pub mod usage;

pub mod smoke;

pub mod gui;

pub fn native_label(body: &crate::protocol::Item) -> Option<String> {
    if let crate::protocol::Item::Native {
        provider,
        record_type,
        raw,
        blob,
    } = body
    {
        let provider = provider.split_whitespace().collect::<Vec<_>>().join(" ");
        let kind = record_type.split_whitespace().collect::<Vec<_>>().join(" ");
        Some(format!(
            "{provider} · {kind} · {} bytes",
            blob.as_ref()
                .map(|b| b.len)
                .unwrap_or_else(|| raw.to_string().len() as u64)
        ))
    } else {
        None
    }
}

#[cfg(feature = "acp")]
pub mod acp;
