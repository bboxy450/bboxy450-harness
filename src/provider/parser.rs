// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
struct Call {
    id: Option<String>,
    name: Option<String>,
    args: String,
}
#[derive(Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    calls: BTreeMap<u8, Call>,
    ids: BTreeSet<String>,
    finish: Option<Finish>,
    terminal: bool,
    output: bool,
}
impl SseParser {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<ModelEvent> {
        if self.terminal {
            return vec![];
        }
        self.buffer.extend_from_slice(bytes);
        if self.buffer.len() > 256 * 1024 {
            return self.fail("SSE frame exceeds 256 KiB");
        }
        let mut out = vec![];
        loop {
            let boundary = self
                .buffer
                .windows(2)
                .position(|b| b == b"\n\n")
                .map(|p| (p, 2))
                .or_else(|| {
                    self.buffer
                        .windows(4)
                        .position(|b| b == b"\r\n\r\n")
                        .map(|p| (p, 4))
                });
            let Some((p, n)) = boundary else {
                break;
            };
            let bytes = self.buffer.drain(..p + n).collect::<Vec<_>>();
            let text = match std::str::from_utf8(&bytes) {
                Ok(t) => t,
                Err(_) => {
                    out.extend(self.fail("invalid SSE UTF-8"));
                    break;
                }
            };
            let data = text
                .lines()
                .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
                .collect::<Vec<_>>()
                .join("\n");
            if data.is_empty() {
                continue;
            }
            if data == "[DONE]" {
                out.extend(self.end());
                break;
            }
            let v: Value = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(_) => {
                    out.extend(self.fail("invalid SSE JSON"));
                    break;
                }
            };
            if v.get("error").is_some() {
                self.terminal = true;
                out.push(ModelEvent::Failed(ProviderError::new(
                    if self.output {
                        ErrorClass::MidStream
                    } else {
                        ErrorClass::Malformed
                    },
                    "provider stream error",
                )));
                break;
            }
            match self.chunk(v) {
                Ok(events) => {
                    self.output |= events.iter().any(|e| {
                        matches!(
                            e,
                            ModelEvent::Text(_)
                                | ModelEvent::Reasoning(_)
                                | ModelEvent::ReasoningOpaque(_)
                                | ModelEvent::ToolCallDelta { .. }
                        )
                    });
                    out.extend(events)
                }
                Err(message) => {
                    out.extend(self.fail(message));
                    break;
                }
            }
        }
        out
    }
    fn chunk(&mut self, v: Value) -> Result<Vec<ModelEvent>, &'static str> {
        if v.get("error").is_some() {
            return Err("provider stream error");
        }
        let mut out = vec![];
        if let Some(u) = v.get("usage").filter(|v| v.is_object()) {
            out.push(ModelEvent::Usage(Usage {
                input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0),
                output_tokens: u["completion_tokens"].as_u64().unwrap_or(0),
                cached_input_tokens: u["prompt_tokens_details"]["cached_tokens"].as_u64(),
                cache_creation_input_tokens: None,
                reasoning_tokens: u["completion_tokens_details"]["reasoning_tokens"].as_u64(),
                cost_micros_usd: u["cost"].as_f64().map(|n| (n * 1_000_000.) as u64),
                ..Default::default()
            }));
        }
        let Some(choices) = v.get("choices").and_then(Value::as_array) else {
            return Err("missing choices");
        };
        if choices.len() > 1 {
            return Err("multiple choices are unsupported");
        }
        for choice in choices {
            if choice["index"].as_u64().is_some_and(|n| n != 0) {
                return Err("unexpected choice index");
            }
            let delta = &choice["delta"];
            let output_fields = ["content", "reasoning", "reasoning_content"]
                .iter()
                .any(|field| delta[*field].as_str().is_some_and(|s| !s.is_empty()))
                || delta["tool_calls"]
                    .as_array()
                    .is_some_and(|v| !v.is_empty())
                || delta
                    .get("reasoning_details")
                    .is_some_and(|v| !v.is_null() && !v.as_array().is_some_and(Vec::is_empty));
            // OpenRouter emits an empty final delta beside usage after its
            // finish-reason chunk. It adds no output and must not retire a
            // completely assembled tool call as malformed.
            if self.finish.is_some() && output_fields {
                return Err("output after finish reason");
            }
            if let Some(s) = delta["content"].as_str().filter(|s| !s.is_empty()) {
                out.push(ModelEvent::Text(s.into()));
            }
            if let Some(s) = delta["reasoning"]
                .as_str()
                .or_else(|| delta["reasoning_content"].as_str())
                .filter(|s| !s.is_empty())
            {
                out.push(ModelEvent::Reasoning(s.into()));
            }
            if let Some(value) = delta.get("reasoning_details").filter(|v| !v.is_null()) {
                out.push(ModelEvent::ReasoningOpaque(value.clone()));
            }
            if let Some(calls) = delta.get("tool_calls") {
                let calls = calls.as_array().ok_or("tool_calls is not an array")?;
                for c in calls {
                    let index = c["index"]
                        .as_u64()
                        .filter(|n| *n <= 64)
                        .ok_or("invalid tool index")? as u8;
                    let id = c
                        .get("id")
                        .filter(|v| !v.is_null())
                        .map(|v| v.as_str().ok_or("invalid tool id"))
                        .transpose()?
                        .map(str::to_string);
                    let name = c["function"]
                        .get("name")
                        .filter(|v| !v.is_null())
                        .map(|v| v.as_str().ok_or("invalid tool name"))
                        .transpose()?
                        .map(str::to_string);
                    let args = c["function"]
                        .get("arguments")
                        .filter(|v| !v.is_null())
                        .map(|v| v.as_str().ok_or("invalid tool arguments"))
                        .transpose()?
                        .unwrap_or("")
                        .to_string();
                    let call = self.calls.entry(index).or_default();
                    if let Some(id) = &id {
                        if call.id.as_ref().is_some_and(|prior| prior != id) {
                            return Err("tool id changed");
                        }
                        if call.id.is_none() && !self.ids.insert(id.clone()) {
                            return Err("duplicate tool id");
                        }
                        call.id = Some(id.clone());
                    }
                    if let Some(name) = &name {
                        if call.name.as_ref().is_some_and(|prior| prior != name) {
                            return Err("tool name changed");
                        }
                        call.name = Some(name.clone());
                    }
                    if call.args.len() + args.len() > 256 * 1024 {
                        return Err("tool arguments exceed cap");
                    }
                    call.args.push_str(&args);
                    out.push(ModelEvent::ToolCallDelta {
                        index,
                        id,
                        name,
                        args,
                    });
                }
            }
            if let Some(reason) = choice.get("finish_reason").filter(|v| !v.is_null()) {
                let finish = match reason.as_str() {
                    Some("stop") => Finish::Stop,
                    Some("tool_calls") => Finish::ToolCalls,
                    Some("length") => Finish::Length,
                    Some("content_filter") => Finish::ContentFilter,
                    _ => return Err("unknown finish reason"),
                };
                if self.finish.as_ref().is_some_and(|prior| prior != &finish) {
                    return Err("conflicting finish reasons");
                }
                // A repeated identical final reason is metadata, not a second
                // terminal event. Finish is emitted once, after [DONE]/EOF.
                self.finish = Some(finish);
            }
        }
        Ok(out)
    }
    pub fn end(&mut self) -> Vec<ModelEvent> {
        if self.terminal {
            return vec![];
        }
        if !self.buffer.is_empty() && self.buffer.iter().any(|b| !b.is_ascii_whitespace()) {
            return self.fail("truncated SSE frame");
        }
        let Some(finish) = self.finish.take() else {
            return self.fail("missing finish reason");
        };
        if matches!(finish, Finish::Stop) && !self.calls.is_empty() {
            return self.fail("stop contradicts tool calls");
        }
        let mut out = vec![];
        if matches!(finish, Finish::ToolCalls) {
            if self.calls.is_empty() {
                return self.fail("tool finish without calls");
            }
            for (&index, call) in &self.calls {
                let (Some(id), Some(name)) = (&call.id, &call.name) else {
                    return self.fail("incomplete tool identity");
                };
                if id.is_empty() || name.is_empty() {
                    return self.fail("empty tool identity");
                }
                let args = serde_json::from_str::<Value>(&call.args)
                    .ok()
                    .filter(Value::is_object)
                    .ok_or_else(|| "arguments must be a JSON object".into());
                out.push(ModelEvent::ToolCallReady {
                    index,
                    id: id.clone(),
                    name: name.clone(),
                    args_raw: call.args.clone(),
                    args,
                });
            }
        }
        self.terminal = true;
        out.push(ModelEvent::Finish(finish));
        out
    }
    fn fail(&mut self, message: &str) -> Vec<ModelEvent> {
        self.terminal = true;
        vec![ModelEvent::Failed(ProviderError::new(
            ErrorClass::Malformed,
            message,
        ))]
    }
}
