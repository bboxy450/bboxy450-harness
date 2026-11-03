// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const CAP: usize = 256 * 1024;
struct Block {
    value: Value,
    args: String,
    closed: bool,
}
#[derive(Default)]
pub(super) struct Parser {
    buffer: Vec<u8>,
    started: bool,
    terminal: bool,
    output: bool,
    blocks: BTreeMap<u8, Block>,
    ids: BTreeSet<String>,
    finish: Option<Finish>,
    usage: Usage,
    uncached: u64,
    opaque: Option<Value>,
}
impl Parser {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<ModelEvent> {
        if self.terminal {
            return vec![];
        }
        self.buffer.extend_from_slice(bytes);
        let mut events = vec![];
        loop {
            let boundary = self
                .buffer
                .windows(2)
                .position(|b| b == b"\n\n")
                .map(|p| (p, 2))
                .into_iter()
                .chain(
                    self.buffer
                        .windows(4)
                        .position(|b| b == b"\r\n\r\n")
                        .map(|p| (p, 4)),
                )
                .min_by_key(|(p, _)| *p);
            let Some((end, width)) = boundary else {
                break;
            };
            if end > CAP {
                events.extend(self.fail("SSE frame exceeds 256 KiB"));
                break;
            }
            let frame = self.buffer.drain(..end + width).collect::<Vec<_>>();
            let result = self.frame(&frame);
            match result {
                Ok(out) => {
                    self.output |= out.iter().any(|event| {
                        matches!(
                            event,
                            ModelEvent::Text(_)
                                | ModelEvent::Reasoning(_)
                                | ModelEvent::ReasoningOpaque(_)
                                | ModelEvent::ToolCallDelta { .. }
                        )
                    });
                    events.extend(out);
                }
                Err(message) => events.extend(self.fail(message)),
            }
            if self.terminal {
                break;
            }
        }
        if !self.terminal && self.buffer.len() > CAP {
            events.extend(self.fail("SSE frame exceeds 256 KiB"));
        }
        events
    }
    pub fn end(&mut self) -> Vec<ModelEvent> {
        if self.terminal {
            vec![]
        } else {
            self.fail("missing message_stop")
        }
    }
    fn fail(&mut self, message: &str) -> Vec<ModelEvent> {
        self.terminal = true;
        vec![ModelEvent::Failed(ProviderError::new(
            ErrorClass::Malformed,
            message,
        ))]
    }
    fn frame(&mut self, frame: &[u8]) -> Result<Vec<ModelEvent>, &'static str> {
        let text = std::str::from_utf8(frame).map_err(|_| "invalid SSE UTF-8")?;
        let data = text
            .lines()
            .filter_map(|line| {
                line.strip_prefix("data:")
                    .map(|s| s.strip_prefix(' ').unwrap_or(s))
            })
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() {
            return Ok(vec![]);
        }
        let v: Value = serde_json::from_str(&data).map_err(|_| "invalid SSE JSON")?;
        let kind = v["type"].as_str().ok_or("missing SSE type")?;
        if text
            .lines()
            .filter_map(|line| line.strip_prefix("event:").map(str::trim))
            .any(|name| name != kind)
        {
            return Err("SSE event type mismatch");
        }
        if kind == "ping" {
            return Ok(vec![]);
        }
        if kind == "error" {
            self.terminal = true;
            let class = if self.output {
                ErrorClass::MidStream
            } else {
                match v["error"]["type"].as_str() {
                    Some("overloaded_error" | "api_error" | "timeout_error") => {
                        ErrorClass::Transient
                    }
                    Some("rate_limit_error") => ErrorClass::RateLimited,
                    Some("authentication_error" | "permission_error") => ErrorClass::Auth,
                    Some("invalid_request_error" | "not_found_error" | "request_too_large") => {
                        ErrorClass::BadRequest
                    }
                    _ => ErrorClass::Malformed,
                }
            };
            return Ok(vec![ModelEvent::Failed(ProviderError::new(
                class,
                "provider stream error",
            ))]);
        }
        if kind == "message_start" {
            if self.started {
                return Err("duplicate message_start");
            }
            if !v["message"]["content"]
                .as_array()
                .is_some_and(Vec::is_empty)
                || !v["message"]["stop_reason"].is_null()
            {
                return Err("invalid message_start");
            }
            self.started = true;
            return Ok(self.usage(&v["message"]["usage"]));
        }
        if !self.started {
            return Err("event before message_start");
        }
        match kind {
            "content_block_start" => self.start(&v),
            "content_block_delta" => self.delta(&v),
            "content_block_stop" => {
                if self.finish.is_some() {
                    return Err("content after stop reason");
                }
                let index = index(&v)?;
                let block = self
                    .blocks
                    .get_mut(&index)
                    .filter(|b| !b.closed)
                    .ok_or("stop for inactive block")?;
                if block.value["type"] == "thinking"
                    && block.value["signature"].as_str().is_none_or(str::is_empty)
                {
                    return Err("thinking block missing signature");
                }
                if block.value["type"] == "tool_use" {
                    if block.args.is_empty() {
                        block.args = block.value["input"].to_string();
                    }
                    block.value["input"] = arguments(&block.args).unwrap_or_else(|_| json!({}));
                }
                block.closed = true;
                Ok(self.opaque())
            }
            "message_delta" => {
                if self.blocks.values().any(|b| !b.closed) {
                    return Err("message_delta before block stop");
                }
                if let Some(reason) = v["delta"].get("stop_reason").filter(|v| !v.is_null()) {
                    let finish = match reason.as_str() {
                        Some("end_turn" | "stop_sequence") => Finish::Stop,
                        Some("tool_use") => Finish::ToolCalls,
                        Some("max_tokens") => Finish::Length,
                        Some("refusal") => Finish::ContentFilter,
                        _ => return Err("unknown stop reason"),
                    };
                    if self.finish.as_ref().is_some_and(|prior| prior != &finish) {
                        return Err("conflicting stop reasons");
                    }
                    self.finish = Some(finish);
                }
                Ok(self.usage(&v["usage"]))
            }
            "message_stop" => {
                if self.blocks.values().any(|b| !b.closed) {
                    return Err("message_stop before block stop");
                }
                let finish = self.finish.take().ok_or("missing stop reason")?;
                if matches!(finish, Finish::Stop) && !self.ids.is_empty()
                    || matches!(finish, Finish::ToolCalls) && self.ids.is_empty()
                {
                    return Err("stop reason contradicts tool calls");
                }
                let mut events = self.opaque();
                if matches!(finish, Finish::ToolCalls) {
                    for (&index, block) in &self.blocks {
                        if block.value["type"] == "tool_use" {
                            events.push(ModelEvent::ToolCallReady {
                                index,
                                id: block.value["id"].as_str().unwrap().into(),
                                name: block.value["name"].as_str().unwrap().into(),
                                args_raw: block.args.clone(),
                                args: arguments(&block.args),
                            });
                        }
                    }
                }
                self.terminal = true;
                events.push(ModelEvent::Finish(finish));
                Ok(events)
            }
            // SSE may acquire new metadata event types. Unknown content or deltas
            // fail below; unknown envelope metadata is safe to ignore.
            _ => Ok(vec![]),
        }
    }
    fn start(&mut self, v: &Value) -> Result<Vec<ModelEvent>, &'static str> {
        if self.finish.is_some() {
            return Err("content after stop reason");
        }
        let index = index(v)?;
        if index as usize != self.blocks.len() {
            return Err("duplicate or out-of-order block index");
        }
        let value = v["content_block"].clone();
        let mut events = vec![];
        match value["type"].as_str() {
            Some("text") => {
                let text = value["text"].as_str().ok_or("invalid text block")?;
                if !text.is_empty() {
                    events.push(ModelEvent::Text(text.into()));
                }
            }
            Some("thinking") => {
                let thinking = value["thinking"].as_str().ok_or("invalid thinking block")?;
                if !thinking.is_empty() {
                    events.push(ModelEvent::Reasoning(thinking.into()));
                }
            }
            Some("redacted_thinking") if value["data"].is_string() => {}
            Some("tool_use") => {
                let id = value["id"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or("invalid tool id")?;
                let name = value["name"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or("invalid tool name")?;
                if !value["input"].is_object() {
                    return Err("invalid initial tool input");
                }
                if !self.ids.insert(id.into()) {
                    return Err("duplicate tool id");
                }
                events.push(ModelEvent::ToolCallDelta {
                    index,
                    id: Some(id.into()),
                    name: Some(name.into()),
                    args: String::new(),
                });
            }
            _ => return Err("unsupported content block"),
        }
        self.blocks.insert(
            index,
            Block {
                value,
                args: String::new(),
                closed: false,
            },
        );
        Ok(events)
    }
    fn delta(&mut self, v: &Value) -> Result<Vec<ModelEvent>, &'static str> {
        if self.finish.is_some() {
            return Err("content after stop reason");
        }
        let index = index(v)?;
        let block = self
            .blocks
            .get_mut(&index)
            .filter(|b| !b.closed)
            .ok_or("delta for inactive block")?;
        let delta = &v["delta"];
        match (block.value["type"].as_str(), delta["type"].as_str()) {
            (Some("text"), Some("text_delta")) => {
                let text = append(&mut block.value, "text", delta, "text")?;
                Ok(if text.is_empty() {
                    vec![]
                } else {
                    vec![ModelEvent::Text(text)]
                })
            }
            (Some("thinking"), Some("thinking_delta")) => {
                let text = append(&mut block.value, "thinking", delta, "thinking")?;
                Ok(if text.is_empty() {
                    vec![]
                } else {
                    vec![ModelEvent::Reasoning(text)]
                })
            }
            (Some("thinking"), Some("signature_delta")) => {
                append(&mut block.value, "signature", delta, "signature")?;
                Ok(vec![])
            }
            (Some("tool_use"), Some("input_json_delta")) => {
                let args = delta["partial_json"]
                    .as_str()
                    .ok_or("invalid input JSON delta")?;
                if !block.value["input"].as_object().unwrap().is_empty() {
                    return Err("input delta after nonempty input");
                }
                if block.args.len() + args.len() > CAP {
                    return Err("tool arguments exceed cap");
                }
                block.args.push_str(args);
                Ok(vec![ModelEvent::ToolCallDelta {
                    index,
                    id: None,
                    name: None,
                    args: args.into(),
                }])
            }
            _ => Err("delta contradicts content block"),
        }
    }
    fn opaque(&mut self) -> Vec<ModelEvent> {
        if !self.blocks.values().any(|b| {
            b.closed
                && matches!(
                    b.value["type"].as_str(),
                    Some("thinking" | "redacted_thinking")
                )
        }) {
            return vec![];
        }
        let snapshot = json!({"anthropic_content":self.blocks.values().filter(|b| b.closed).map(|b| &b.value).collect::<Vec<_>>()});
        if self.opaque.as_ref() == Some(&snapshot) {
            return vec![];
        }
        self.opaque = Some(snapshot.clone());
        vec![ModelEvent::ReasoningOpaque(snapshot)]
    }
    fn usage(&mut self, value: &Value) -> Vec<ModelEvent> {
        if !value.is_object() {
            return vec![];
        }
        if self.usage.source.is_none() {
            self.usage.reported("input_tokens", false);
            self.usage.reported("output_tokens", false);
        }
        if let Some(n) = value["input_tokens"].as_u64() {
            self.uncached = n;
            self.usage.reported("input_tokens", true);
        }
        if let Some(n) = value["output_tokens"].as_u64() {
            self.usage.output_tokens = n;
            self.usage.reported("output_tokens", true);
        }
        if let Some(n) = value["cache_read_input_tokens"].as_u64() {
            self.usage.cached_input_tokens = Some(n);
        }
        if let Some(n) = value["cache_creation_input_tokens"].as_u64() {
            self.usage.cache_creation_input_tokens = Some(n);
        }
        if !self.usage.unreported.iter().any(|s| s == "input_tokens") {
            let Some(total) = self
                .uncached
                .checked_add(self.usage.cached_input_tokens.unwrap_or(0))
                .and_then(|n| n.checked_add(self.usage.cache_creation_input_tokens.unwrap_or(0)))
            else {
                return self.fail("usage input counter overflow");
            };
            self.usage.input_tokens = total;
        }
        vec![ModelEvent::Usage(self.usage.clone())]
    }
}
fn index(v: &Value) -> Result<u8, &'static str> {
    v["index"]
        .as_u64()
        .filter(|n| *n <= 64)
        .map(|n| n as u8)
        .ok_or("invalid block index")
}
fn append(
    block: &mut Value,
    field: &str,
    delta: &Value,
    key: &str,
) -> Result<String, &'static str> {
    let text = delta[key].as_str().ok_or("invalid content delta")?;
    let prior = block[field].as_str().unwrap_or("");
    if prior.len() + text.len() > CAP {
        return Err("content block exceeds cap");
    }
    block[field] = json!(prior.to_owned() + text);
    Ok(text.into())
}
fn arguments(raw: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .filter(Value::is_object)
        .ok_or_else(|| "arguments must be a JSON object".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_tool_input_and_signed_reasoning_survive_every_single_byte_split() {
        for raw in [
            include_bytes!("../../tests/fixtures/sse/anthropic/utf8-split.sse").as_slice(),
            include_bytes!("../../tests/fixtures/sse/anthropic/thinking-signature.sse").as_slice(),
        ] {
            let mut bytes = raw.to_vec();
            while let Some(at) = bytes.windows(7).position(|b| b == b"#split\n") {
                bytes.drain(at..at + 7);
            }
            let expected = Parser::default().feed(&bytes);
            assert!(matches!(expected.last(), Some(ModelEvent::Finish(_))));
            let mut parser = Parser::default();
            let mut actual = Vec::new();
            for byte in &bytes {
                actual.extend(parser.feed(&[*byte]));
            }
            actual.extend(parser.end());
            assert_eq!(actual, expected);
            for split in 0..=bytes.len() {
                let mut parser = Parser::default();
                let mut actual = parser.feed(&bytes[..split]);
                actual.extend(parser.feed(&bytes[split..]));
                actual.extend(parser.end());
                assert_eq!(actual, expected, "split at {split}");
            }
        }
    }
}
