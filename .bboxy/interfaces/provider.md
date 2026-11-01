---
type: interface
title: "ICD: provider stream — engine ↔ model endpoint (v0.1)"
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[kittens-spec]]", rel: evidence }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# ICD: provider stream

| | |
|---|---|
| Side A | `engine` TurnRunner (E1) |
| Side B | `provider`, an OpenAI-compatible chat/completions SSE client (E1), and the model endpoint (external; OpenRouter first) |
| Lineage | kittens `src/provider.rs:1-17`, `:541-566`, `:748-793`; `src/retry.rs:26-83` ([[kittens-spec]] §3) |

## 1. Trait

```rust
pub trait Provider: Send + Sync {
    fn caps(&self) -> Caps;   // images, reasoning, tool_choice, context_tokens
    fn stream(&self, req: ModelRequest, cancel: CancellationToken)
        -> Pin<Box<dyn Stream<Item = ModelEvent> + Send>>;
}
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<WireMessage>,      // provider-valid: every tool call paired (law 2)
    pub tools: Vec<ToolDef>,
    pub tool_choice: ToolChoice,
    pub max_tokens: Option<u32>,
    pub stable_prefix: usize,            // messages[..stable_prefix] are byte-identical across the session's rounds
}
pub enum ModelEvent {
    Retrying { attempt: u32, wait_ms: u64, class: ErrorClass },  // only before Output
    Text(String),
    Reasoning(String),
    ReasoningOpaque(Value),              // reasoning_details, kept verbatim for replay
    ToolCallDelta { index: u8, id: Option<String>, name: Option<String>, args: String },
    ToolCallReady { index: u8, id: String, name: String, args_raw: String, args: Result<Value, String> },
    Usage(Usage),
    Finish(Finish),                      // terminal: Stop | ToolCalls | Length | ContentFilter
    Failed(ProviderError),               // terminal
}
pub struct ProviderError { pub class: ErrorClass, pub message: String /* redacted, never the raw body */ }
pub enum ErrorClass { Transient, RateLimited, Auth, BadRequest, Malformed, MidStream, Cancelled, Store, Tool }
```

## 2. Stream rules

1. **One ending.** Exactly one terminal event, `Finish` or `Failed`, and nothing after it.
2. **Output boundary.** Output is the first `Text`, `Reasoning`, `ReasoningOpaque` or `ToolCallDelta`. `Retrying` may occur only before output. After output, any error is `Failed{MidStream}`, and the engine ends the turn with `StopReason::Error`. It never silently replays a half-shown answer.
3. **Tool-call assembly.**
   - Fragments merge by `index`, which must be at most 64.
   - A duplicate id, or an id or name that changes, is `Failed{Malformed}`.
   - `ToolCallReady` is emitted for each index before `Finish(ToolCalls)`.
   - Arguments that do not parse as a JSON object give `args: Err(..)`. The engine turns that into a model-visible tool error, not a turn failure.
4. **Finish reason.** A finish reason that is missing, unknown, or contradicts the content (`stop` while tool calls are present) is `Failed{Malformed}`.
5. **Cancel.** When the token fires, the HTTP body is dropped and the stream yields `Failed{Cancelled}` within 250 ms.
6. **Prefix stability.** The engine assembles `messages[..stable_prefix]` once per session and sends it byte-identical every round. A provider may add cache markers; it never rewrites the prefix.

## 3. Retry and timing

- **Transient:** HTTP 408, 500, 502, 503 and 504, and connection errors. **RateLimited:** HTTP 429.
- **Backoff:** 5 s · 2^n plus jitter, capped at 600 s. A longer `Retry-After` is honoured.
- **Retries run until cancel or `retry_deadline`.** `retry_deadline` is a profile field. The default is none, which is kittens behaviour. The live smoke sets 90 s.
- **Timeouts:** connect 60 s; read stall 600 s; no deadline on the whole attempt.
- **Diagnostics:** every attempt and every wait is reported as `Retrying` and becomes a durable `Notice` item.
- **Never retried:** Auth, BadRequest, Malformed.

## 4. Profiles and keys

- **Location.** Profiles live in `~/.bboxy450-harness/profiles/<name>.md` as YAML frontmatter, with the kittens keys: `name provider model endpoint api_key|api_key_env retry_deadline`.
- **Endpoint.** It must be `https` (`http` only for localhost), with no userinfo and no query. A custom endpoint never inherits another profile's key.
- **`!cmd` keys.** An `api_key` that starts with `!` is a command. It runs once per session through `bbt_rt::spawn_command` (`sh -c`) in its own process group, with a 10 s timeout and an 8 KiB output cap, and it is cancellable.
  - The value is held as `Zeroizing<String>` and registered with the redactor (law [[secrets-never-persist]]).
  - On a timeout or a non-zero exit, the class is `Auth`, with the message `key command failed (exit N)` and no output.
- **Request quirks.** OpenRouter requests omit `parallel_tool_calls:false` and `provider.require_parameters` (kittens incident issue-184: they cause a 404 on free Nemotron routes).

## 5. Test doubles

- **`ScriptedProvider`** replays `tests/fixtures/sse/*.sse`. These are raw SSE bytes, in which a line `#split` forces a chunk boundary, so TCP fragmentation is covered.
- **Required fixtures:**
  - text only
  - text then tool call
  - two parallel tool calls
  - a tool call split mid-UTF-8
  - a duplicate tool id
  - a missing finish reason
  - 429 then success
  - 500 after output
  - a stall (the read-stall timer is shortened by test config)
  - cancel mid-stream
- **Wiremock-style HTTP server.** `tests/support/sse_server.rs` serves the same fixtures over real HTTP, so `OpenAiCompat` itself is tested and not only the parser.

## 6. Verified by

| Clause | Evidence |
|---|---|
| §2.1–2.4 | `tests/provider_stream.rs` over every fixture, asserting the exact `ModelEvent` sequence |
| §2.5 cancel ≤ 250 ms | `tests/provider_stream.rs::cancel_mid_stream` with a timer assertion |
| §3 retry before output only | `429_then_ok` passes; `500_after_output` gives `MidStream` with no retry |
| §4 redaction | invariant [[secrets-never-persist]] |
| live endpoint | `make smoke-live`. It is not part of `make test` ([[architect-review-v0]] §8) |

Owner: Nyx Copperwing <ARCHITECT>. Both sides: E1.
