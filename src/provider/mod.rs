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
pub mod claude_code;
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
    /// External agents own their tools and loop; they never enter TurnRunner.
    fn claude_code(&self) -> Option<claude_code::Options> {
        None
    }
    fn caps(&self) -> Caps;
    fn stream(
        &self,
        req: ModelRequest,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>>;
}
#[derive(Clone)]
pub struct OpenAiCompat {
    client: reqwest::Client,
    endpoint: String,
    key: Arc<Secret>,
    pub redactor: Set,
    pub options: Options,
    requests: Arc<std::sync::atomic::AtomicU32>,
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
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn new(
        endpoint: &str,
        key: Secret,
        redactor: Set,
        options: Options,
    ) -> Result<Self, ProviderError> {
        profile::validate_endpoint(endpoint)?;
        Ok(Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(60))
                .build()
                .map_err(|_| ProviderError::new(ErrorClass::Internal, "HTTP client unavailable"))?,
            endpoint: format!("{}/chat/completions", endpoint.trim_end_matches('/')),
            key: Arc::new(key),
            redactor,
            options,
            requests: Arc::default(),
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
        let this = self.clone();
        Box::pin(async_stream::stream! {
             let choice=match req.tool_choice{ToolChoice::Auto=>json!("auto"),ToolChoice::None=>json!("none"),ToolChoice::Required=>json!("required"),ToolChoice::Named(name)=>json!({"type":"function","function":{"name":name}})};
             let mut body=json!({"model":req.model,"messages":this.redactor.value(json!(req.messages)),"stream":true,"stream_options":{"include_usage":true},"reasoning":{"exclude":false}});
             if !req.tools.is_empty(){body["tools"]=tools_wire(&req.tools);body["tool_choice"]=choice;}
             if let Some(max)=req.max_tokens{body["max_tokens"]=json!(max);}
             let started=tokio::time::Instant::now();let mut attempt=0u32;
             loop {
              if this.options.max_requests.is_some_and(|max|this.requests.try_update(std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire,|n|(n<max).then_some(n+1)).is_err()) {yield ModelEvent::Failed(ProviderError::new(ErrorClass::Transient,"request budget exhausted"));return;}
        if this.options.max_requests.is_none(){this.requests.fetch_add(1,std::sync::atomic::Ordering::Relaxed);}
        attempt+=1;
              let result=tokio::select!{biased;_=cancel.cancelled()=>{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},r=this.client.post(&this.endpoint).bearer_auth(this.key.expose()).json(&body).send()=>r};
              let mut response=match result {
               Ok(response) if response.status().is_success()=>response,
               result=>{
                let (class,retry_after,status)=match result {
                 Ok(r)=>{let n=r.status().as_u16();let c=match n{429=>ErrorClass::RateLimited,408|500|502|503|504=>ErrorClass::Transient,401|403=>ErrorClass::Auth,_=>ErrorClass::BadRequest};let after=r.headers().get("retry-after").and_then(|v|v.to_str().ok()).and_then(parse_retry_after);(c,after,n)},
                 Err(_)=>(ErrorClass::Transient,None,0),
                };
                if !matches!(class,ErrorClass::Transient|ErrorClass::RateLimited){yield ModelEvent::Failed(ProviderError::new(class,format!("provider request refused (HTTP {})",status)));return;}
                let base=this.options.backoff.saturating_mul(2u32.saturating_pow((attempt-1).min(16))).min(Duration::from_secs(600));
                let jitter=Duration::from_millis((ulid::Ulid::new().0%250) as u64);let wait=retry_after.unwrap_or_default().max((base+jitter).min(Duration::from_secs(600)));
                if this.options.max_requests.is_some_and(|n|attempt>=n)||this.options.retry_deadline.is_some_and(|d|started.elapsed()+wait>d){yield ModelEvent::Failed(ProviderError::new(class,"retry budget exhausted"));return;}
                yield ModelEvent::Retrying{attempt,wait_ms:wait.as_millis() as u64,class};
                tokio::select!{biased;_=cancel.cancelled()=>{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},_=tokio::time::sleep(wait)=>{}}
                continue;
               }
              };
              let mut parser=parser::SseParser::default();let mut output=false;let mut terminal=false;
              loop {
               let chunk=tokio::select! {biased;
                _=cancel.cancelled()=>{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},
                r=tokio::time::timeout(this.options.read_stall,response.chunk())=>r,
               };
               match chunk {
                Ok(Ok(Some(bytes)))=>{for event in parser.feed(&bytes){output|=matches!(event,ModelEvent::Text(_)|ModelEvent::Reasoning(_)|ModelEvent::ReasoningOpaque(_)|ModelEvent::ToolCallDelta{..});terminal|=matches!(event,ModelEvent::Failed(_)|ModelEvent::Finish(_));yield event;}if terminal{return;}},
                Ok(Ok(None))=>{for e in parser.end(){yield e;}return;},
                _=>{if output {yield ModelEvent::Failed(ProviderError::new(ErrorClass::MidStream,"provider stream interrupted"));return;}break;}
               }
              }
              if this.options.max_requests.is_some_and(|n|attempt>=n){yield ModelEvent::Failed(ProviderError::new(ErrorClass::Transient,"retry budget exhausted"));return;}
              let wait=this.options.backoff.saturating_mul(2u32.saturating_pow((attempt-1).min(16))).min(Duration::from_secs(600));
              if this.options.retry_deadline.is_some_and(|d|started.elapsed()+wait>d){yield ModelEvent::Failed(ProviderError::new(ErrorClass::Transient,"retry budget exhausted"));return;}
              yield ModelEvent::Retrying{attempt,wait_ms:wait.as_millis() as u64,class:ErrorClass::Transient};
              tokio::select!{_=cancel.cancelled()=>{yield ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},_=tokio::time::sleep(wait)=>{}}
             }
            })
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
