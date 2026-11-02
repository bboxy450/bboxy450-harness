// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Native records share the engine's blob-before-completion rule, for import and watch.
use crate::{protocol::*, store::Store};
use serde_json::Value;
use std::io;

const INLINE_BYTES: usize = 50 * 1024;
const PREVIEW_BYTES: usize = 8 * 1024;

fn oversized(text: &str) -> bool {
    // JSON escaping matters: a short run of control characters can exceed the wire cap.
    text.len() > INLINE_BYTES
        || serde_json::to_vec(text).is_ok_and(|bytes| bytes.len() > INLINE_BYTES)
}
fn preview(text: &str, blob: &BlobRef) -> String {
    let mut end = text.len().min(PREVIEW_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[full content: blob {} ({} bytes)]",
        &text[..end],
        blob.sha256,
        blob.len
    )
}
fn strings(value: &mut Value) -> Vec<&mut String> {
    match value {
        Value::String(s) => vec![s],
        Value::Array(values) => values.iter_mut().flat_map(strings).collect(),
        Value::Object(values) => values.values_mut().flat_map(strings).collect(),
        _ => vec![],
    }
}

pub(super) async fn append(store: &mut Store, mut event: Event, at: u64) -> io::Result<Envelope> {
    // Redact before cutting: a secret crossing the preview boundary must not leak a prefix.
    event = store
        .blobs
        .redactor
        .sanitize(&event)
        .map_err(io::Error::other)?;
    if let Event::ItemCompleted {
        body: Item::ToolCall { output, spill, .. },
        ..
    } = &mut event
        && let [OutputBlock::Text { text } | OutputBlock::Error { text }] = output.as_mut_slice()
        && oversized(text)
    {
        let blob = store
            .blobs
            .put(text.as_bytes().to_vec(), "text/plain")
            .await?;
        *text = preview(text, &blob);
        *spill = Some(blob);
    }
    // User/assistant/reasoning text and tool input have no typed spill slot in v0.4.
    // Preserve exact text in a blob and leave a visible digest, never silently clip it.
    let mut value = serde_json::to_value(&event)?;
    for text in strings(&mut value) {
        if oversized(text) {
            let blob = store
                .blobs
                .put(text.as_bytes().to_vec(), "text/plain")
                .await?;
            *text = preview(text, &blob);
        }
    }
    store
        .append_at(serde_json::from_value(value)?, None, at)
        .await
}
