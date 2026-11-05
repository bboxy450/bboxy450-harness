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
struct Payload {
    raw: Vec<u8>,
    redacted: Vec<u8>,
    reference: BlobRef,
}
struct Prepared {
    event: Event,
    ext: std::collections::BTreeMap<String, Value>,
    payloads: Vec<Payload>,
}
fn prepare(
    mut event: Event,
    mut ext: std::collections::BTreeMap<String, Value>,
    set: crate::redact::Set,
) -> io::Result<Prepared> {
    use sha2::{Digest, Sha256};
    crate::runtime::assert_off_rt_worker();
    let mut payloads = Vec::new();
    let mut put = |raw: Vec<u8>, mime: &str| {
        let redacted = set.bytes(&raw);
        let reference = BlobRef {
            sha256: format!("{:x}", Sha256::digest(&redacted)),
            len: redacted.len() as u64,
            mime: mime.into(),
        };
        payloads.push(Payload {
            raw,
            redacted,
            reference: reference.clone(),
        });
        reference
    };
    // Same redaction/cut/preview order as the shared ordinary import path.
    event = set.sanitize(&event).map_err(io::Error::other)?;
    if let Event::ItemCompleted {
        body: Item::Native { raw, blob, .. },
        ..
    } = &mut event
    {
        let bytes = serde_json::to_vec(raw)?;
        if bytes.len() > 64 * 1024 {
            let reference = put(bytes.clone(), "application/json");
            *raw = json_preview(&bytes);
            *blob = Some(reference);
        }
    }
    for value in ext.values_mut() {
        *value = set.value(value.clone());
        let bytes = serde_json::to_vec(value)?;
        if bytes.len() > 64 * 1024 {
            let blob = put(bytes.clone(), "application/json");
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
        let blob = put(text.as_bytes().to_vec(), "text/plain");
        *text = preview(text, &blob);
        *spill = Some(blob);
    }
    // A result made of many small fields can exceed the cap in aggregate.
    // Persist both raw and rendered representations before publishing references.
    if let Event::ItemCompleted {
        body: Item::ToolCall { raw_output, output, spill, .. }, ..
    } = &mut event {
        if let Some(raw) = raw_output {
            spill_json(raw, &mut put)?;
        }
        let mut value = serde_json::to_value(&*output)?;
        if let Some(blob) = spill_json(&mut value, &mut put)? {
            *output = vec![OutputBlock::Json { value }];
            *spill = Some(blob);
        }
    }
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
            let blob = put(text.as_bytes().to_vec(), "text/plain");
            *text = preview(text, &blob);
        }
    }
    Ok(Prepared {
        event: serde_json::from_value(value)?,
        ext,
        payloads,
    })
}
pub(crate) async fn append_projected(
    store: &mut Store,
    event: Event,
    cause: Option<OpId>,
    at: u64,
    native: Option<Native>,
    ext: std::collections::BTreeMap<String, Value>,
) -> io::Result<Envelope> {
    append_mode(store, event, cause, at, native, ext, false).await
}
/// Opt-in native custody writes already-scrubbed planned payloads privately and
/// reuses Blobs::ingest; heavy redaction/serialization never polls on an IO worker.
pub(crate) async fn append_projected_staging(
    store: &mut Store,
    event: Event,
    at: u64,
    native: Option<Native>,
    ext: std::collections::BTreeMap<String, Value>,
) -> io::Result<Envelope> {
    append_mode(store, event, None, at, native, ext, true).await
}
async fn append_mode(
    store: &mut Store,
    event: Event,
    cause: Option<OpId>,
    at: u64,
    native: Option<Native>,
    ext: std::collections::BTreeMap<String, Value>,
    staging: bool,
) -> io::Result<Envelope> {
    let set = store.blobs.redactor.clone();
    let prepared = crate::runtime::blocking(move || prepare(event, ext, set)).await?;
    for payload in prepared.payloads {
        let actual = if staging {
            let dir = store.blobs.dir.clone();
            let mime = payload.reference.mime.clone();
            let path = crate::runtime::blocking(move || {
                use std::{io::Write, os::unix::fs::OpenOptionsExt};
                crate::runtime::assert_off_rt_worker();
                std::fs::create_dir_all(&dir)?;
                let path = dir.join(format!(".native-spill-{}", ulid::Ulid::new()));
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)?;
                file.write_all(&payload.redacted)?;
                crate::runtime::sync_all(&file)?;
                Ok::<_, io::Error>(path)
            })
            .await?;
            store.blobs.ingest(path, &mime).await?
        } else {
            store
                .blobs
                .put(payload.raw, &payload.reference.mime)
                .await?
        };
        if actual != payload.reference {
            return Err(io::Error::other(
                "redaction changed during blob preparation",
            ));
        }
    }
    store
        .append_projected_at(prepared.event, cause, at, native, prepared.ext)
        .await
}
fn spill_json(
    value: &mut Value,
    put: &mut impl FnMut(Vec<u8>, &str) -> BlobRef,
) -> io::Result<Option<BlobRef>> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() <= 64 * 1024 {
        return Ok(None);
    }
    let blob = put(bytes.clone(), "application/json");
    *value = serde_json::json!({"$blob":blob,"$preview":json_preview(&bytes)});
    Ok(Some(blob))
}
fn json_preview(bytes: &[u8]) -> Value {
    let mut end = bytes.len().min(4096);
    while end > 0 && std::str::from_utf8(&bytes[..end]).is_err() {
        end -= 1;
    }
    Value::String(String::from_utf8_lossy(&bytes[..end]).into_owned())
}
