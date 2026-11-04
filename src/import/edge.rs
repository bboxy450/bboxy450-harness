// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Consumed leaves are explicit; unfamiliar nested fields remain at their native
//! path, including array positions. Source discriminants accompany a nonempty
//! remainder so a later projector can interpret it.
use super::Native;
use crate::protocol::Vcs;

/// Promote only actual native fields; never inspect today's checkout to fill
/// historical gaps. Unknown native Git leaves remain in the raw projection.
pub fn vcs(native: Native, record: &Value) -> Option<Vcs> {
    let text = |value: Option<&Value>| {
        value
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() < crate::vcs::MAX_METADATA_BYTES)
            .map(str::to_owned)
    };
    let value = match native {
        Native::ClaudeCode if record.get("isSidechain").and_then(Value::as_bool) != Some(true) => {
            Vcs {
                branch: text(record.get("gitBranch")),
                ..Default::default()
            }
        }
        Native::Codex if record.get("type").and_then(Value::as_str) == Some("session_meta") => {
            Vcs {
                repo: record
                    .pointer("/payload/git/repository_url")
                    .and_then(Value::as_str)
                    .and_then(crate::vcs::normalize_repo_key),
                branch: text(record.pointer("/payload/git/branch")),
                commit: text(record.pointer("/payload/git/commit_hash")),
                dirty: None,
            }
        }
        _ => return None,
    };
    (value != Vcs::default()).then_some(value)
}

pub(super) fn sanitized_record(native: Native, record: &Value) -> Value {
    let mut out = record.clone();
    if native == Native::Codex
        && record.get("type").and_then(Value::as_str) == Some("session_meta")
        && let Some(Value::String(url)) = out.pointer_mut("/payload/git/repository_url")
    {
        *url = crate::vcs::public_remote(url);
    }
    out
}
use serde_json::Value;

fn remove(value: &mut Value, keys: &[&str]) {
    if let Some(o) = value.as_object_mut() {
        for key in keys {
            o.remove(*key);
        }
    }
}
fn empty(value: &Value) -> bool {
    matches!(value, Value::Object(o) if o.is_empty())
        || matches!(value, Value::Array(a) if a.iter().all(empty))
}
fn content(value: &mut Value) {
    if let Some(parts) = value.as_array_mut() {
        for part in parts {
            let tag = part.get("type").cloned();
            let call = part.get("tool_use_id").cloned();
            match part.get("type").and_then(Value::as_str) {
                Some("text" | "input_text" | "output_text" | "summary_text") => {
                    remove(part, &["type", "text"])
                }
                Some("thinking") => remove(part, &["type", "thinking"]),
                Some("tool_use") => remove(part, &["type", "id", "name", "input"]),
                Some("tool_result") => {
                    if let Some(c) = part.get_mut("content") {
                        if c.is_string() {
                            *c = Value::Object(Default::default());
                        } else {
                            content(c);
                        }
                    }
                    remove(part, &["type", "tool_use_id", "is_error"]);
                    if part.get("content").is_some_and(empty) {
                        remove(part, &["content"]);
                    }
                }
                _ => {}
            }
            if !empty(part)
                && let Some(object) = part.as_object_mut()
            {
                if let Some(tag) = tag {
                    object.insert("type".into(), tag);
                }
                if let Some(call) = call {
                    object.insert("tool_use_id".into(), call);
                }
            }
        }
    }
}

/// This allowlist contains only values carried by native identity or typed
/// fields. Unknown values of known keys are retained, never coerced away.
pub fn remainder(native: Native, record: &Value) -> Value {
    let mut out = sanitized_record(native, record);
    let source_type = record.get("type").cloned();
    if record
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(super::rfc3339_ms)
        .is_some()
    {
        remove(&mut out, &["timestamp"]);
    }
    match native {
        Native::ClaudeCode => {
            remove(&mut out, &["type"]);
            for key in if super::native_identity(record).is_some() {
                vec!["uuid", "parentUuid"]
            } else {
                vec![]
            } {
                if record
                    .get(key)
                    .is_some_and(|v| v.is_string() || v.is_null())
                {
                    remove(&mut out, &[key]);
                }
            }
            if let Some(message) = out.get_mut("message") {
                if message.get("role") == record.get("type") {
                    remove(message, &["role"]);
                }
                if message.get("content").is_some_and(Value::is_string) {
                    remove(message, &["content"]);
                } else if let Some(c) = message.get_mut("content") {
                    content(c);
                    if empty(c) {
                        remove(message, &["content"]);
                    }
                }
                if message.get("stop_reason").and_then(Value::as_str) == Some("end_turn") {
                    remove(message, &["stop_reason"]);
                }
                if empty(message) {
                    remove(&mut out, &["message"]);
                }
            }
        }
        Native::Codex => {
            remove(&mut out, &["type"]);
            if let Some(payload) = out.get_mut("payload") {
                let kind = payload
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                match kind.as_str() {
                    "message" => {
                        remove(payload, &["type", "role"]);
                        if let Some(c) = payload.get_mut("content") {
                            content(c);
                            if empty(c) {
                                remove(payload, &["content"]);
                            }
                        }
                    }
                    "reasoning" => {
                        remove(payload, &["type"]);
                        if let Some(c) = payload.get_mut("summary") {
                            content(c);
                            if empty(c) {
                                remove(payload, &["summary"]);
                            }
                        }
                    }
                    "function_call" | "custom_tool_call" | "local_shell_call" => remove(
                        payload,
                        &["type", "call_id", "name", "arguments", "input", "action"],
                    ),
                    "function_call_output"
                    | "custom_tool_call_output"
                    | "local_shell_call_output" => {
                        remove(payload, &["type", "call_id"]);
                        if payload
                            .pointer("/output/content")
                            .is_some_and(Value::is_string)
                        {
                            if let Some(output) = payload.get_mut("output") {
                                remove(output, &["content"]);
                                if empty(output) {
                                    remove(payload, &["output"]);
                                }
                            }
                        } else {
                            remove(payload, &["output"]);
                        }
                    }
                    "task_started" | "task_complete" | "turn_aborted" => remove(payload, &["type"]),
                    _ => {}
                }
                if !empty(payload)
                    && let (Some(kind), Some(object)) =
                        (record.pointer("/payload/type"), payload.as_object_mut())
                {
                    object.insert("type".into(), kind.clone());
                }
                if empty(payload) {
                    remove(&mut out, &["payload"]);
                }
            }
        }
    }
    if !empty(&out)
        && let (Some(kind), Some(object)) = (source_type, out.as_object_mut())
    {
        object.insert("type".into(), kind);
    }
    out
}
