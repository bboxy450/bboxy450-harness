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

pub(super) async fn append(store: &mut Store, event: Event, at: u64) -> io::Result<Envelope> {
    append_projected(store, event, None, at, None, Default::default()).await
}
pub(crate) async fn append_projected(
    store: &mut Store,
    mut event: Event,
    cause: Option<OpId>,
    at: u64,
    native: Option<Native>,
    mut ext: std::collections::BTreeMap<String, Value>,
) -> io::Result<Envelope> {
    // Redact before cutting: a secret crossing the preview boundary must not leak a prefix.
    event = store
        .blobs
        .redactor
        .sanitize(&event)
        .map_err(io::Error::other)?;
    if let Event::ItemCompleted {
        body: Item::Native { raw, blob, .. },
        ..
    } = &mut event
    {
        let bytes = serde_json::to_vec(raw)?;
        if bytes.len() > 64 * 1024 {
            let reference = store.blobs.put(bytes.clone(), "application/json").await?;
            *raw = json_preview(&bytes);
            *blob = Some(reference);
        }
    }
    for value in ext.values_mut() {
        *value = store.blobs.redactor.value(value.clone());
        let bytes = serde_json::to_vec(value)?;
        if bytes.len() > 64 * 1024 {
            let blob = store.blobs.put(bytes.clone(), "application/json").await?;
            *value = serde_json::json!({"$blob":blob,"$preview":json_preview(&bytes)});
        }
    }
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
    let texts = if matches!(
        &event,
        Event::ItemCompleted {
            body: Item::Native { .. },
            ..
        }
    ) {
        vec![]
    } else {
        strings(&mut value)
    };
    for text in texts {
        if oversized(text) {
            let blob = store
                .blobs
                .put(text.as_bytes().to_vec(), "text/plain")
                .await?;
            *text = preview(text, &blob);
        }
    }
    store
        .append_projected_at(serde_json::from_value(value)?, cause, at, native, ext)
        .await
}
fn json_preview(bytes: &[u8]) -> Value {
    let mut end = bytes.len().min(4096);
    while end > 0 && std::str::from_utf8(&bytes[..end]).is_err() {
        end -= 1;
    }
    Value::String(String::from_utf8_lossy(&bytes[..end]).into_owned())
}
