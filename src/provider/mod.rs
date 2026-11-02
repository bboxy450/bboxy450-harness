// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    protocol::{ErrorClass, Usage},
    redact::{Secret, Set},
};
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio_util::sync::CancellationToken;
mod anthropic;
mod anthropic_parser;
mod http;
pub use anthropic::Anthropic;
pub mod parser;
pub mod profile;
#[derive(Clone, Debug)]
pub struct Caps {
    pub images: bool,
    pub reasoning: bool,
    pub tool_choice: bool,
    pub context_tokens: u64,
}
pub type WireMessage = Value;
pub fn tools_wire(tools: &[ToolDef]) -> Value {
    json!(
        tools
            .iter()
            .map(|t| json!({"type":"function","function":t}))
            .collect::<Vec<_>>()
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
#[derive(Clone, Debug)]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Named(String),
}
#[derive(Clone, Debug)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<WireMessage>,
    pub tools: Vec<ToolDef>,
    pub tool_choice: ToolChoice,
    pub max_tokens: Option<u32>,
    pub stable_prefix: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ModelEvent {
    Retrying {
        attempt: u32,
        wait_ms: u64,
        class: ErrorClass,
    },
    Text(String),
    Reasoning(String),
    ReasoningOpaque(Value),
    ToolCallDelta {
        index: u8,
        id: Option<String>,
        name: Option<String>,
        args: String,
    },
    ToolCallReady {
        index: u8,
        id: String,
        name: String,
        args_raw: String,
        args: Result<Value, String>,
    },
    Usage(Usage),
    Finish(Finish),
    Failed(ProviderError),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Finish {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderError {
    pub class: ErrorClass,
    pub message: String,
}
impl ProviderError {
    pub fn new(class: ErrorClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}
impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.class, self.message)
    }
}
impl std::error::Error for ProviderError {}
pub trait Provider: Send + Sync {
    fn caps(&self) -> Caps;
    fn stream(
        &self,
        req: ModelRequest,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>>;
}
#[derive(Clone)]
pub struct OpenAiCompat {
    http: http::Http,
    pub redactor: Set,
    pub options: Options,
}
#[derive(Clone, Debug)]
pub struct Options {
    pub retry_deadline: Option<Duration>,
    pub max_requests: Option<u32>,
    pub backoff: Duration,
    pub read_stall: Duration,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            retry_deadline: None,
            max_requests: None,
            backoff: Duration::from_secs(5),
            read_stall: Duration::from_secs(600),
        }
    }
}
impl OpenAiCompat {
    pub fn request_count(&self) -> u32 {
        self.http.request_count()
    }
    pub fn new(
        endpoint: &str,
        key: Secret,
        redactor: Set,
        options: Options,
    ) -> Result<Self, ProviderError> {
        profile::validate_endpoint(endpoint)?;
        Ok(Self {
            http: http::Http::new(
                format!("{}/chat/completions", endpoint.trim_end_matches('/')),
                key,
                http::Format::OpenAi,
            )?,
            redactor,
            options,
        })
    }
}
impl Provider for OpenAiCompat {
    fn caps(&self) -> Caps {
        Caps {
            images: true,
            reasoning: true,
            tool_choice: true,
            context_tokens: 128_000,
        }
    }
    fn stream(
        &self,
        req: ModelRequest,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        let choice = match req.tool_choice {
            ToolChoice::Auto => json!("auto"),
            ToolChoice::None => json!("none"),
            ToolChoice::Required => json!("required"),
            ToolChoice::Named(name) => json!({"type":"function","function":{"name":name}}),
        };
        let mut body = json!({"model":req.model,"messages":self.redactor.value(json!(req.messages)),"stream":true,"stream_options":{"include_usage":true},"reasoning":{"exclude":false}});
        if !req.tools.is_empty() {
            body["tools"] = tools_wire(&req.tools);
            body["tool_choice"] = choice;
        }
        if let Some(max) = req.max_tokens {
            body["max_tokens"] = json!(max);
        }
        self.http.stream(body, self.options.clone(), cancel)
    }
}
pub fn parse_retry_after(value: &str) -> Option<Duration> {
    value
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
        .or_else(|| {
            httpdate::parse_http_date(value)
                .ok()?
                .duration_since(SystemTime::now())
                .ok()
        })
}
#[derive(Clone)]
pub struct ScriptedProvider {
    pub rounds: Arc<Vec<Vec<u8>>>,
    pub calls: Arc<std::sync::atomic::AtomicUsize>,
    pub delay: Duration,
}
impl ScriptedProvider {
    pub fn new(rounds: Vec<Vec<u8>>) -> Self {
        Self {
            rounds: Arc::new(rounds),
            calls: Arc::default(),
            delay: Duration::ZERO,
        }
    }
}
impl Provider for ScriptedProvider {
    fn caps(&self) -> Caps {
        Caps {
            images: true,
            reasoning: true,
            tool_choice: true,
            context_tokens: 128_000,
        }
    }
    fn stream(
        &self,
        _req: ModelRequest,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        let this = self.clone();
        Box::pin(async_stream::stream! {
         let index=this.calls.fetch_add(1,std::sync::atomic::Ordering::SeqCst);let Some(bytes)=this.rounds.get(index) else{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Malformed,"script exhausted"));return;};
         let mut parser=parser::SseParser::default();
         for chunk in bytes.split(|b|*b==0){let mut clean=chunk.to_vec();while let Some(i)=clean.windows(7).position(|w|w==b"#split\n"){clean.drain(i..i+7);}for byte in &clean{
          if !this.delay.is_zero(){tokio::select!{biased;_=cancel.cancelled()=>{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},_=tokio::time::sleep(this.delay)=>{}}}
          if cancel.is_cancelled(){yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;}
          for event in parser.feed(&[*byte]){let terminal=matches!(event,ModelEvent::Finish(_)|ModelEvent::Failed(_));yield event;if terminal{return;}}
         }}for e in parser.end(){yield e;}
        })
    }
}

#[derive(Clone)]
pub struct FailedProvider(pub ProviderError);
impl Provider for FailedProvider {
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: false,
            tool_choice: false,
            context_tokens: 0,
        }
    }
    fn stream(
        &self,
        _: ModelRequest,
        _: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        let error = self.0.clone();
        Box::pin(futures_util::stream::once(async move {
            ModelEvent::Failed(error)
        }))
    }
}
