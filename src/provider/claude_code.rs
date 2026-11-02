// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Claude Code is an external agent, never a completion provider for harness tools.
use super::*;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub const VERSION: &str = "2.1.304";
pub const MAX_LINE: usize = 1024 * 1024;
pub const AUTH_FIX: &str =
    "Claude Code authentication unavailable; run claude /login or claude setup-token";
#[derive(Clone)]
pub struct Options {
    pub model: String,
    pub permission_mode: Option<String>,
}
pub struct AgentProvider(pub Options);
impl Provider for AgentProvider {
    fn claude_code(&self) -> Option<Options> {
        Some(self.0.clone())
    }
    fn caps(&self) -> Caps {
        Caps {
            images: false,
            reasoning: true,
            tool_choice: false,
            context_tokens: 0,
        }
    }
    fn stream(
        &self,
        _: ModelRequest,
        _: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>> {
        Box::pin(futures_util::stream::once(async {
            ModelEvent::Failed(ProviderError::new(
                ErrorClass::Internal,
                "Claude Code must run as an external agent",
            ))
        }))
    }
}
impl Options {
    pub fn argv(&self, resume: Option<&str>) -> Vec<String> {
        let mut args = vec![
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        if !self.model.is_empty() {
            args.extend(["--model".into(), self.model.clone()]);
        }
        if let Some(mode) = &self.permission_mode {
            args.extend(["--permission-mode".into(), mode.clone()]);
        }
        if let Some(id) = resume {
            args.extend(["--resume".into(), id.into()]);
        }
        args
    }
}
/// Cancellation-safe framing: partial bytes remain here when a read future is dropped.
/// No read_until/read_line allocation can grow beyond the cap.
#[derive(Default)]
pub struct JsonLines {
    bytes: Vec<u8>,
}
impl JsonLines {
    pub async fn next(
        &mut self,
        reader: &mut (impl AsyncBufRead + Unpin),
    ) -> Result<Option<Value>, ProviderError> {
        loop {
            let bytes = reader
                .fill_buf()
                .await
                .map_err(|_| malformed("Claude Code stdout read failed"))?;
            if bytes.is_empty() {
                return if self.bytes.is_empty() {
                    Ok(None)
                } else {
                    Err(malformed("truncated Claude Code JSON line"))
                };
            }
            let n = bytes
                .iter()
                .position(|b| *b == b'\n')
                .map_or(bytes.len(), |i| i + 1);
            if self.bytes.len() + n > MAX_LINE {
                return Err(malformed("Claude Code JSON line exceeds 1 MiB"));
            }
            let complete = bytes[n - 1] == b'\n';
            self.bytes.extend_from_slice(&bytes[..n]);
            reader.consume(n);
            if complete {
                let bytes = std::mem::take(&mut self.bytes);
                let parsed = if bytes.len() > 4096 {
                    crate::runtime::blocking(move || serde_json::from_slice::<Value>(&bytes)).await
                } else {
                    serde_json::from_slice::<Value>(&bytes)
                }
                .map_err(|_| malformed("malformed Claude Code JSON line"));
                let v = parsed?;
                if !v.is_object() {
                    return Err(malformed("Claude Code event is not an object"));
                }
                return Ok(Some(v));
            }
        }
    }
}
pub fn malformed(text: &str) -> ProviderError {
    ProviderError::new(ErrorClass::Malformed, text)
}
pub fn auth_error(text: &str) -> bool {
    let s = text.to_ascii_lowercase();
    [
        "not logged in",
        "authentication_failed",
        "authentication_error",
        "invalid api key",
        "oauth token",
        "please run /login",
        "login required",
        "failed to authenticate",
    ]
    .iter()
    .any(|x| s.contains(x))
}
/// Do not resolve or inspect credential values, including inherited OAuth env.
/// Scrub known credential syntax and already-registered harness secrets on display.
pub fn scrub(set: &Set, text: &str) -> String {
    static TOKENS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static FIELDS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let text = set.scrub(text);
    let text = TOKENS
        .get_or_init(|| regex::Regex::new(r"sk-[a-zA-Z0-9_-]{8,}").unwrap())
        .replace_all(&text, "<redacted>");
    FIELDS.get_or_init(|| regex::Regex::new(r#"(?i)(?:bearer\s+|(?:authorization|access_token|refresh_token|claude_code_oauth_token|api_key)[\"'\s]*[:=][\"'\s]*)[^\s\"',}]+"#).unwrap()).replace_all(&text, "<redacted>").into_owned()
}
pub fn usage(event: &Value) -> Usage {
    let u = &event["usage"];
    Usage {
        input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
        cached_input_tokens: u["cache_read_input_tokens"].as_u64(),
        reasoning_tokens: None,
        cost_micros_usd: event["total_cost_usd"]
            .as_f64()
            .filter(|n| n.is_finite() && *n >= 0.0)
            .map(|n| (n * 1_000_000.0).round() as u64),
    }
}
