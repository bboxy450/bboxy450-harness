// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;

pub const DEFAULT_ENDPOINT: &str = "https://api.anthropic.com";
/// Used only when neither the request nor the profile supplies max_tokens.
pub const DEFAULT_MAX_TOKENS: u32 = 4096;

#[derive(Clone)]
pub struct Anthropic {
    http: http::Http,
    pub redactor: Set,
    pub options: Options,
    pub max_tokens: Option<u32>,
}
impl Anthropic {
    pub fn new(
        endpoint: &str,
        key: Secret,
        redactor: Set,
        options: Options,
    ) -> Result<Self, ProviderError> {
        let endpoint = if endpoint.is_empty() {
            DEFAULT_ENDPOINT
        } else {
            endpoint
        };
        profile::validate_endpoint(endpoint)?;
        let endpoint = endpoint.trim_end_matches('/');
        let endpoint = endpoint.strip_suffix("/v1").unwrap_or(endpoint);
        Ok(Self {
            http: http::Http::new(
                format!("{endpoint}/v1/messages"),
                key,
                http::Format::Anthropic,
            )?,
            redactor,
            options,
            max_tokens: None,
        })
    }
    pub fn request_count(&self) -> u32 {
        self.http.request_count()
    }
    fn body(&self, req: ModelRequest) -> Result<Value, ProviderError> {
        if req.stable_prefix > req.messages.len() {
            return Err(bad("stable prefix exceeds messages"));
        }
        crate::engine::history::check_wire(&req.messages)
            .map_err(|_| bad("unpaired tool history"))?;
        let max = req
            .max_tokens
            .or(self.max_tokens)
            .unwrap_or(DEFAULT_MAX_TOKENS);
        if max == 0 {
            return Err(bad("max_tokens must be positive"));
        }
        let mut system = Vec::new();
        let mut messages: Vec<Value> = Vec::new();
        let mut stable_end = None;
        for (index, message) in req.messages.into_iter().enumerate() {
            let mut message = self.redactor.value(message);
            super::filter_opaque(&mut message, "anthropic");
            let role = message["role"]
                .as_str()
                .ok_or_else(|| bad("missing message role"))?;
            if role == "system" {
                if !messages.is_empty() {
                    return Err(bad("system must precede conversation"));
                }
                system.extend(text_blocks(&message["content"])?);
                continue;
            }
            let (role, blocks) = match role {
                "user" => ("user", text_blocks(&message["content"])?),
                "assistant" => ("assistant", assistant_blocks(&message)?),
                "tool" => {
                    let id = message["tool_call_id"]
                        .as_str()
                        .ok_or_else(|| bad("missing tool result id"))?;
                    (
                        "user",
                        vec![
                            json!({"type":"tool_result","tool_use_id":id,"content":message["content"]}),
                        ],
                    )
                }
                _ => return Err(bad("unsupported message role")),
            };
            // Consecutive tool results form one user message immediately after tool_use.
            // Appending blocks preserves the original stable blocks and their bytes.
            if messages.last().is_some_and(|m| m["role"] == role) {
                messages.last_mut().unwrap()["content"]
                    .as_array_mut()
                    .unwrap()
                    .extend(blocks);
            } else {
                messages.push(json!({"role":role,"content":blocks}));
            }
            if index < req.stable_prefix {
                let mi = messages.len() - 1;
                let blocks = messages[mi]["content"].as_array().unwrap();
                if let Some(bi) = blocks.iter().rposition(cacheable) {
                    stable_end = Some((mi, bi));
                }
            }
        }
        if let Some((mi, bi)) = stable_end {
            mark(&mut messages[mi]["content"][bi]);
        }
        if let Some(last) = system.last_mut() {
            mark(last);
        }
        let mut body =
            json!({"model":req.model,"messages":messages,"stream":true,"max_tokens":max});
        if !system.is_empty() {
            body["system"] = json!(system);
        }
        if !req.tools.is_empty() {
            let mut tools = req.tools.into_iter().map(|tool| {
                json!({"name":tool.name,"description":tool.description,"input_schema":tool.parameters})
            }).collect::<Vec<_>>();
            mark(tools.last_mut().unwrap());
            body["tools"] = self.redactor.value(json!(tools));
            body["tool_choice"] = match req.tool_choice {
                ToolChoice::Auto => json!({"type":"auto"}),
                ToolChoice::None => json!({"type":"none"}),
                ToolChoice::Required => json!({"type":"any"}),
                ToolChoice::Named(name) => json!({"type":"tool","name":name}),
            };
        } else if matches!(req.tool_choice, ToolChoice::Required | ToolChoice::Named(_)) {
            return Err(bad("tool choice requires tools"));
        }
        Ok(body)
    }
}
impl Provider for Anthropic {
    fn id(&self) -> &str {
        "anthropic"
    }
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: true,
            tool_choice: true,
            context_tokens: 200_000,
        }
    }
    fn stream(
        &self,
        req: ModelRequest,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        match self.body(req) {
            Ok(body) => self.http.stream(body, self.options.clone(), cancel),
            Err(error) => Box::pin(futures_util::stream::once(async move {
                ModelEvent::Failed(error)
            })),
        }
    }
}
fn bad(message: &str) -> ProviderError {
    ProviderError::new(ErrorClass::BadRequest, message)
}
fn cacheable(block: &Value) -> bool {
    matches!(
        block["type"].as_str(),
        Some("text" | "tool_use" | "tool_result")
    )
}
fn mark(block: &mut Value) {
    block["cache_control"] = json!({"type":"ephemeral"});
}
fn text_blocks(content: &Value) -> Result<Vec<Value>, ProviderError> {
    match content {
        Value::Null => Ok(vec![]),
        Value::String(text) => Ok(if text.is_empty() {
            vec![]
        } else {
            vec![json!({"type":"text","text":text})]
        }),
        Value::Array(blocks)
            if blocks
                .iter()
                .all(|b| b["type"] == "text" && b["text"].is_string()) =>
        {
            Ok(blocks.clone())
        }
        _ => Err(bad("unsupported message content")),
    }
}
fn assistant_blocks(message: &Value) -> Result<Vec<Value>, ProviderError> {
    let mut blocks = text_blocks(&message["content"])?;
    if let Some(calls) = message["tool_calls"].as_array() {
        for call in calls {
            let id = call["id"].as_str().ok_or_else(|| bad("missing tool id"))?;
            let name = call["function"]["name"]
                .as_str()
                .ok_or_else(|| bad("missing tool name"))?;
            // An invalid streamed argument object has already received a synthetic tool
            // error from the engine. Preserve the pair on the next request.
            let input = call["function"]["arguments"]
                .as_str()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .filter(Value::is_object)
                .unwrap_or_else(|| json!({}));
            blocks.push(json!({"type":"tool_use","id":id,"name":name,"input":input}));
        }
    }
    if let Some(native) = message["reasoning_details"]["anthropic_content"].as_array() {
        // The opaque snapshot includes block order, thinking text, signatures and
        // redacted_thinking data. Replay those blocks without reconstruction.
        let native_calls = native
            .iter()
            .filter(|b| b["type"] == "tool_use")
            .collect::<Vec<_>>();
        let calls = blocks
            .iter()
            .filter(|b| b["type"] == "tool_use")
            .collect::<Vec<_>>();
        if native_calls != calls
            || native.iter().any(|b| {
                !matches!(
                    b["type"].as_str(),
                    Some("text" | "thinking" | "redacted_thinking" | "tool_use")
                )
            })
        {
            return Err(bad("opaque reasoning contradicts tool history"));
        }
        return Ok(native.clone());
    }
    Ok(blocks)
}
