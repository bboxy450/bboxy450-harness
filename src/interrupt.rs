// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

use crate::protocol::{Envelope, ErrorClass, Event, InputBlock, Interrupt, StopReason, TurnId};
use std::time::Duration;
use tokio::time::Instant;

struct Cause {
    id: &'static str,
    provider: &'static [&'static str],
    rpc: &'static [i64],
    text: &'static [&'static str],
    class: ErrorClass,
}

macro_rules! cause {
    ($id:literal, $class:ident, [$($code:literal),*], [$($rpc:literal),*], [$($text:literal),*]) => {
        Cause { id: $id, class: ErrorClass::$class, provider: &[$($code),*], rpc: &[$($rpc),*], text: &[$($text),*] }
    };
}

// Match explicit source/provider codes first across the whole table, then JSON-RPC
// codes, then (only for unstructured failures) diagnostic text.
const CAUSES: &[Cause] = &[
    cause!(
        "provider.capacity",
        Transient,
        ["model_capacity_exceeded", "capacity_exceeded"],
        [],
        ["selected model is at capacity"]
    ),
    cause!(
        "provider.rate_limit",
        RateLimited,
        ["rate_limit_exceeded", "rate_limit_error"],
        [429],
        ["rate limit", "too many requests", "usage limit"]
    ),
    cause!(
        "provider.overloaded",
        Transient,
        ["overloaded_error", "overloaded"],
        [529],
        ["overloaded"]
    ),
    cause!(
        "provider.5xx",
        Transient,
        ["server_error", "service_unavailable"],
        [500, 502, 503, 504],
        ["service unavailable", "bad gateway"]
    ),
    cause!(
        "provider.auth",
        Auth,
        ["authentication_error", "invalid_api_key"],
        [-32000, 401, 403],
        ["authenticate", "authentication", "unauthorized"]
    ),
    cause!(
        "agent.exited",
        MidStream,
        [],
        [],
        [
            "agent exited",
            "agent prompt transport closed",
            "acp steering transport closed"
        ]
    ),
    cause!(
        "agent.protocol",
        Malformed,
        [],
        [-32700, -32600],
        [
            "agent sent ",
            "unknown stop reason",
            "returned no sessionid",
            "speaks protocol"
        ]
    ),
    cause!("agent.config", Config, [], [], ["agent could not start"]),
    cause!("agent.cancelled", Cancelled, [], [], []),
    cause!("harness.exited", Internal, [], [], []),
    cause!(
        "harness.panic",
        Internal,
        [],
        [],
        ["panicked at", "runner failed: panic", "task panicked"]
    ),
    cause!(
        "hook.failed",
        Internal,
        ["hook_failed"],
        [],
        ["hook failed", "hook error"]
    ),
    cause!(
        "mcp.unavailable",
        Tool,
        ["mcp_unavailable"],
        [],
        ["mcp transport", "mcp unavailable"]
    ),
    cause!(
        "tool.timeout",
        Tool,
        ["tool_timeout"],
        [],
        [
            "tool timed out",
            "command timed out",
            "startup deadline exceeded",
            "agent sent nothing for"
        ]
    ),
    cause!("context.max_tokens", BadRequest, [], [], []),
    cause!("turn.max_rounds", BadRequest, [], [], []),
    cause!(
        "refusal",
        BadRequest,
        [],
        [],
        ["the agent refused this prompt", "content filtered"]
    ),
];

pub(crate) fn classify(
    signal: Option<&str>,
    provider: Option<&str>,
    rpc: Option<i64>,
    message: &str,
    fallback: ErrorClass,
) -> (&'static str, ErrorClass) {
    let cause = CAUSES
        .iter()
        .find(|cause| {
            signal == Some(cause.id) || provider.is_some_and(|code| cause.provider.contains(&code))
        })
        .or_else(|| {
            CAUSES
                .iter()
                .find(|cause| rpc.is_some_and(|code| cause.rpc.contains(&code)))
        })
        .or_else(|| {
            let lower = message.to_ascii_lowercase();
            CAUSES
                .iter()
                .find(|cause| cause.text.iter().any(|text| lower.contains(text)))
        });
    cause.map_or(("unknown", fallback), |cause| {
        (cause.id, cause.class.clone())
    })
}

const BACKOFF: [u64; 6] = [1, 3, 6, 10, 15, 30];

#[derive(Default)]
pub(crate) struct Recovery {
    attempt: u64,
    pending: Option<(TurnId, Instant)>,
}
impl Recovery {
    pub(crate) fn replay(events: &[Envelope]) -> Self {
        let mut recovery = Self::default();
        // A fork copies its parent's prefix for context. Retry ownership stays with the parent.
        let inherited_through = events
            .first()
            .and_then(|envelope| match &envelope.event {
                Event::SessionStarted { meta } => meta.fork.map(|(_, seq)| seq.0),
                _ => None,
            })
            .unwrap_or_default();
        for envelope in events.iter().filter(|e| e.at.0 > inherited_through) {
            match &envelope.event {
                Event::TurnEnded {
                    turn,
                    reason:
                        StopReason::Error {
                            interrupt:
                                Some(Interrupt {
                                    attempt,
                                    deadline_ms,
                                    ..
                                }),
                            ..
                        },
                    ..
                } => {
                    recovery.attempt = *attempt;
                    recovery.pending = Some((
                        *turn,
                        Instant::now()
                            + Duration::from_millis(
                                deadline_ms
                                    .saturating_sub(crate::store::now_ms())
                                    .min(30 * 60 * 1000),
                            ),
                    ));
                }
                Event::TurnStarted { .. } | Event::SessionClosed => recovery.pending = None,
                Event::InterruptDismissed { turn } => {
                    recovery.take(Some(*turn));
                }
                Event::TurnEnded {
                    reason: StopReason::EndTurn | StopReason::Quiet { .. },
                    ..
                } => {
                    recovery.completed();
                }
                _ => {}
            }
        }
        recovery
    }

    pub(crate) fn failed(
        &mut self,
        turn: TurnId,
        cause: &str,
        class: ErrorClass,
        message: String,
    ) -> StopReason {
        self.attempt = self.attempt.saturating_add(1);
        let bucket = (self.attempt - 1).min((BACKOFF.len() - 1) as u64) as usize;
        let delay = BACKOFF[bucket] * retry_minute_ms();
        self.pending = Some((turn, Instant::now() + Duration::from_millis(delay)));
        StopReason::Error {
            class,
            message,
            interrupt: Some(Interrupt {
                cause: cause.into(),
                attempt: self.attempt,
                deadline_ms: crate::store::now_ms().saturating_add(delay),
            }),
        }
    }

    pub(crate) fn classify_end(&mut self, turn: TurnId, reason: StopReason) -> StopReason {
        let (signal, class, message) = match reason {
            StopReason::Error {
                class,
                message,
                interrupt: None,
            } => (None, class, message),
            StopReason::MaxTokens => (
                Some("context.max_tokens"),
                ErrorClass::BadRequest,
                "Hit the output limit".into(),
            ),
            StopReason::MaxRounds => (
                Some("turn.max_rounds"),
                ErrorClass::BadRequest,
                "Stopped at the round limit".into(),
            ),
            StopReason::Cancelled => (
                Some("agent.cancelled"),
                ErrorClass::Cancelled,
                "The agent cancelled the prompt".into(),
            ),
            StopReason::EndTurn | StopReason::Quiet { .. } => {
                self.completed();
                return reason;
            }
            _ => return reason,
        };
        let (cause, class) = classify(signal, None, None, &message, class);
        self.failed(turn, cause, class, message)
    }

    fn completed(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.pending.map(|(_, instant)| instant)
    }

    pub(crate) fn take(&mut self, target: Option<TurnId>) -> Option<TurnId> {
        if self
            .pending
            .is_some_and(|(turn, _)| target.is_none_or(|id| id == turn))
        {
            self.pending.take().map(|(turn, _)| turn)
        } else {
            None
        }
    }
}

fn retry_minute_ms() -> u64 {
    // The private process fixtures run the production timer on a scaled clock.
    #[cfg(debug_assertions)]
    if let Some(ms) = std::env::var("BBOXY450_INTERRUPT_MINUTE_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|ms| (1..60_000).contains(ms))
    {
        return ms;
    }
    60_000
}

pub(crate) fn continue_prompt() -> Vec<InputBlock> {
    vec![InputBlock::Text {
        text: "Continue".into(),
    }]
}

pub(crate) fn seconds_remaining(deadline_ms: u64, now_ms: u64) -> u64 {
    deadline_ms.saturating_sub(now_ms).div_ceil(1000)
}
