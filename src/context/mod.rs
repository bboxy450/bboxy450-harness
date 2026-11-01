// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub mod compact;
use crate::{engine::history, protocol::*, provider::WireMessage, store::Blobs};
pub fn latest(log: &[Envelope]) -> Option<(&BlobRef, Seq)> {
    log.iter().rev().find_map(|e| match &e.event {
        Event::ItemCompleted {
            status: ItemStatus::Completed,
            body:
                Item::Compaction {
                    summary,
                    replaced_through,
                },
            ..
        } => Some((summary, *replaced_through)),
        _ => None,
    })
}
pub async fn replay(log: &[Envelope], blobs: &Blobs) -> std::io::Result<(Vec<WireMessage>, usize)> {
    if let Some((blob, through)) = latest(log) {
        let summary = String::from_utf8(blobs.get(blob).await?).map_err(std::io::Error::other)?;
        let tail = log
            .iter()
            .filter(|e| e.seq.is_some_and(|s| s > through))
            .cloned()
            .collect::<Vec<_>>();
        let mut messages = vec![serde_json::json!({"role":"user","content":summary})];
        messages.extend(history::to_wire(&tail));
        Ok((messages, 2))
    } else {
        Ok((history::to_wire(log), 1))
    }
}
