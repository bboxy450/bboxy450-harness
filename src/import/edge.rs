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
        }
    }
}

/// This allowlist contains only values carried by native identity or typed
/// fields. Unknown values of known keys are retained, never coerced away.
pub fn remainder(native: Native, record: &Value) -> Value {
    let mut out = record.clone();
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
            for key in ["uuid", "parentUuid"] {
                if record
                    .get(key)
                    .is_some_and(|v| v.is_string() || v.is_null())
                {
                    remove(&mut out, &[key]);
                }
            }
            if let Some(message) = out.get_mut("message") {
                remove(message, &["role"]);
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
                    | "local_shell_call_output" => remove(payload, &["type", "call_id", "output"]),
                    "task_started" | "task_complete" | "turn_aborted" => remove(payload, &["type"]),
                    _ => {}
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
