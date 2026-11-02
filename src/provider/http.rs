// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy)]
pub(super) enum Format {
    OpenAi,
    Anthropic,
}

#[derive(Clone)]
pub(super) struct Http {
    client: reqwest::Client,
    endpoint: String,
    key: Arc<Secret>,
    requests: Arc<AtomicU32>,
    format: Format,
}
impl Http {
    pub fn new(endpoint: String, key: Secret, format: Format) -> Result<Self, ProviderError> {
        profile::validate_endpoint(&endpoint)?;
        let client = reqwest::Client::builder().connect_timeout(Duration::from_secs(60));
        let client = match format {
            // reqwest's bearer handling is retained for legacy profiles; a
            // custom x-api-key must never follow an Anthropic redirect.
            Format::OpenAi => client,
            Format::Anthropic => client.redirect(reqwest::redirect::Policy::none()),
        };
        Ok(Self {
            client: client
                .build()
                .map_err(|_| ProviderError::new(ErrorClass::Internal, "HTTP client unavailable"))?,
            endpoint,
            key: Arc::new(key),
            requests: Arc::default(),
            format,
        })
    }
    pub fn request_count(&self) -> u32 {
        self.requests.load(Ordering::Relaxed)
    }
    pub fn stream(
        &self,
        body: Value,
        options: Options,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        let this = self.clone();
        Box::pin(async_stream::stream! {
            let started = tokio::time::Instant::now();
            let mut attempt = 0u32;
            loop {
                if cancel.is_cancelled() {
                    yield cancelled(); return;
                }
                if let Some(max) = options.max_requests {
                    if this.requests.try_update(Ordering::AcqRel, Ordering::Acquire,
                        |n| (n < max).then_some(n + 1)).is_err() {
                        yield ModelEvent::Failed(ProviderError::new(ErrorClass::Transient, "request budget exhausted")); return;
                    }
                } else {
                    this.requests.fetch_add(1, Ordering::Relaxed);
                }
                attempt = attempt.saturating_add(1);
                let request = this.client.post(&this.endpoint).json(&body);
                let request = match this.format {
                    Format::OpenAi => request.bearer_auth(this.key.expose()),
                    Format::Anthropic => request.header("x-api-key", this.key.expose())
                        .header("anthropic-version", "2023-06-01"),
                };
                let result = tokio::select! { biased;
                    _ = cancel.cancelled() => { yield cancelled(); return; },
                    response = request.send() => response,
                };
                let mut retry_after = None;
                let failure = match result {
                    Ok(mut response) if response.status().is_success() => {
                        let mut parser = Parser::new(this.format);
                        let mut output = false;
                        loop {
                            let chunk = tokio::select! { biased;
                                _ = cancel.cancelled() => { yield cancelled(); return; },
                                result = tokio::time::timeout(options.read_stall, response.chunk()) => result,
                            };
                            let events = match chunk {
                                Ok(Ok(Some(bytes))) => parser.feed(&bytes),
                                Ok(Ok(None)) => parser.end(),
                                _ if output => {
                                    yield ModelEvent::Failed(ProviderError::new(ErrorClass::MidStream, "provider stream interrupted")); return;
                                },
                                _ => break ProviderError::new(ErrorClass::Transient, "provider stream interrupted"),
                            };
                            let mut retry = None;
                            for event in events {
                                if let ModelEvent::Failed(ref error) = event
                                    && !output && retryable(&error.class) {
                                    retry = Some(error.clone()); break;
                                }
                                output |= matches!(event, ModelEvent::Text(_) | ModelEvent::Reasoning(_)
                                    | ModelEvent::ReasoningOpaque(_) | ModelEvent::ToolCallDelta { .. });
                                let terminal = matches!(event, ModelEvent::Finish(_) | ModelEvent::Failed(_));
                                yield event;
                                if terminal { return; }
                            }
                            if let Some(error) = retry { break error; }
                        }
                    },
                    Ok(response) => {
                        let status = response.status().as_u16();
                        retry_after = response.headers().get("retry-after")
                            .and_then(|v| v.to_str().ok()).and_then(parse_retry_after);
                        // The body and reqwest errors can reflect credentials. Never format either.
                        ProviderError::new(http_class(status, this.format), format!("provider request refused (HTTP {status})"))
                    },
                    Err(_) => ProviderError::new(ErrorClass::Transient, "provider connection failed"),
                };
                if !retryable(&failure.class) {
                    yield ModelEvent::Failed(failure); return;
                }
                let base = options.backoff.saturating_mul(2u32.saturating_pow((attempt - 1).min(16)));
                let jitter = Duration::from_millis((ulid::Ulid::new().0 % 250) as u64);
                let wait = retry_after.unwrap_or_default().max((base + jitter).min(Duration::from_secs(600)));
                if options.max_requests.is_some_and(|n| attempt >= n)
                    || options.retry_deadline.is_some_and(|d| started.elapsed().saturating_add(wait) > d) {
                    yield ModelEvent::Failed(ProviderError::new(failure.class, "retry budget exhausted")); return;
                }
                yield ModelEvent::Retrying { attempt, wait_ms: wait.as_millis().min(u64::MAX as u128) as u64, class: failure.class };
                tokio::select! { biased;
                    _ = cancel.cancelled() => { yield cancelled(); return; },
                    _ = tokio::time::sleep(wait) => {},
                }
            }
        })
    }
}
fn cancelled() -> ModelEvent {
    ModelEvent::Failed(ProviderError::new(ErrorClass::Cancelled, "cancelled"))
}
fn retryable(class: &ErrorClass) -> bool {
    matches!(class, ErrorClass::Transient | ErrorClass::RateLimited)
}
fn http_class(status: u16, format: Format) -> ErrorClass {
    match status {
        401 | 403 => ErrorClass::Auth,
        429 => ErrorClass::RateLimited,
        408 | 500 | 502 | 503 | 504 => ErrorClass::Transient,
        500..=599 if matches!(format, Format::Anthropic) => ErrorClass::Transient,
        _ => ErrorClass::BadRequest,
    }
}
enum Parser {
    OpenAi(super::parser::SseParser),
    Anthropic(Box<super::anthropic_parser::Parser>),
}
impl Parser {
    fn new(format: Format) -> Self {
        match format {
            Format::OpenAi => Self::OpenAi(Default::default()),
            Format::Anthropic => Self::Anthropic(Default::default()),
        }
    }
    fn feed(&mut self, bytes: &[u8]) -> Vec<ModelEvent> {
        match self {
            Self::OpenAi(parser) => parser.feed(bytes),
            Self::Anthropic(parser) => parser.feed(bytes),
        }
    }
    fn end(&mut self) -> Vec<ModelEvent> {
        match self {
            Self::OpenAi(parser) => parser.end(),
            Self::Anthropic(parser) => parser.end(),
        }
    }
}
