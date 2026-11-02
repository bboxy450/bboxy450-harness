---
id: "0BQF1650V8GV5FYZHFMEPRVCYY"
number: 7
type: interface
title: "ICD: provider stream — engine ↔ model endpoint (v0.1)"
status: baselined
amendment: proposed
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

## 7. Anthropic Messages backend

**Amendment: proposed.** This section is submitted for Architect review by the
Oracle before merge. Sections 1–6 retain their baseline; both backends implement
the same `Provider` boundary and share `src/provider/http.rs` retry, budget,
cancellation and read-stall ownership.

### 7.1 Profile, authentication and request

`provider: anthropic` selects `Anthropic` in every profile factory (initial
headless/seat/GUI preparation, ACP session creation and idle profile changes).
An omitted endpoint defaults to `https://api.anthropic.com`; a custom endpoint
accepts either a base URL or a trailing `/v1` (with an optional trailing slash);
the client posts to exactly one `/v1/messages`. The existing HTTPS/local
HTTP endpoint validation and `Profile::key` literal/environment/`!cmd` resolution
apply. `x-api-key` contains the resolved API key; `anthropic-version` is
`2023-06-01`. There is no Authorization header, OAuth flow, subscription-token
exchange, Claude Code client identity, beta header or injected prompt prefix.
Redirects are refused so an API key cannot follow a redirect to another origin.

The request sets `stream: true`. Leading system messages become ordered text
blocks in top-level `system`. Conversation text becomes text content blocks.
Assistant function calls become `tool_use {id,name,input}`; each paired tool
result becomes a `tool_result {tool_use_id,content}` in the immediately following
user message. Adjacent messages of the same role concatenate blocks without
rewriting their contents. The generic history pairing check runs before HTTP;
an orphan, duplicate or missing result is BadRequest. Invalid streamed argument
JSON remains a `ToolCallReady.args` error for the engine; after its synthetic
tool result the replay input object is `{}`.

`Auto`, `None`, `Required` and `Named(name)` map respectively to
`{type:auto}`, `{type:none}`, `{type:any}` and `{type:tool,name}`. Tool schemas
use `input_schema`. Required/named choice without tools is BadRequest.
`max_tokens` is required and positive: request value, then profile `max_tokens`,
then **4096**. v0 advertises text/tool/reasoning capability; image input mapping
is not implemented.

### 7.2 Cache breakpoints and reasoning replay

The last tool definition, last system block and last eligible content block at
or before `messages[..stable_prefix]` receive
`cache_control: {type: ephemeral}`. This uses at most three breakpoints. A marker
inside a merged group of tool results stays on the final stable result, even
when later results or user text join the same user message. Nothing after the
stable boundary is marked. Existing text and signed block strings are copied
unchanged; no trimming, prompt prefix or string reserialization is applied.
Thinking/redacted-thinking blocks cannot carry cache markers: if the boundary
ends there, the preceding eligible block is used.

`thinking_delta` yields Reasoning. On a completed signed or redacted thinking
block, ReasoningOpaque carries `{anthropic_content:[native content blocks]}`.
Subsequent closed blocks update this cumulative snapshot, preserving original
block order, thinking text, concatenated signature bytes and opaque redacted
data. The engine's existing opaque-value replacement retains the latest
snapshot. `history::to_wire` keeps it as `reasoning_details`; Anthropic replays
its native content verbatim, checking its tool calls against the paired generic
history. Snapshots are provider-local opaque values; no decryption is attempted.
The existing secrets-never-persist redactor still governs saved and outbound
content.

### 7.3 Streaming and usage

`message_start` opens the stream and emits Usage when supplied. Indexed
`content_block_start` / `content_block_delta` / `content_block_stop` own text,
thinking, signature, redacted-thinking and tool-input assembly. `text_delta`
yields Text; tool starts and `input_json_delta` yield ToolCallDelta. Complete
calls yield ToolCallReady in index order before Finish. Indices are contiguous,
unique and at most 64. Duplicate tool ids, inactive-block deltas, conflicting
block kinds or reasons, missing signatures and truncated frames are Malformed.
SSE frames, argument strings and individual content strings are capped at
256 KiB. `ping` and unknown envelope metadata types have no ModelEvent.

`message_delta` records the stop reason and updates cumulative Usage;
`message_stop` emits exactly one terminal event. `end_turn|stop_sequence` maps
to Stop, `tool_use` to ToolCalls, `max_tokens` to Length and `refusal` to
ContentFilter. Missing/unknown reasons or a tool-presence mismatch are Malformed;
a truncated tool invocation is never dispatched under a non-tool stop reason.
Events following a terminal are ignored. EOF without message_stop is Malformed.

Anthropic's input_tokens excludes cached input. ModelEvent Usage.input_tokens
is the sum of uncached, cache-read and cache-creation input tokens, so existing
context accounting sees the whole prompt. `cached_input_tokens` retains the
cache-read count, and the additive optional `cache_creation_input_tokens`
retains the creation count. Missing fields in later usage deltas retain the
preceding values; output_tokens is cumulative, not added across events.

### 7.4 Failures and fixtures

HTTP 401/403 → Auth; 400/404/413 and other nonretryable statuses → BadRequest;
429 → RateLimited; 408 and every 5xx (including 529 overload) → Transient.
Retry-After seconds/date feeds the shared capped exponential backoff, with a
longer server wait honoured. An in-stream error before output maps its type:
authentication/permission → Auth, invalid_request/not_found/request_too_large →
BadRequest, rate_limit → RateLimited, overloaded/api/timeout → Transient and
unknown → Malformed. After output it is always MidStream, with no retry.
Connection/read-stall errors use the same output boundary and cancellation
rule as OpenAiCompat. Errors contain fixed safe diagnostics or HTTP status;
neither response bodies nor raw reqwest diagnostics are included.

`tests/fixtures/sse/anthropic/` mirrors every §5 required fixture: text,
text-then-tool, parallel, utf8-split, duplicate-id, missing-finish,
429-then-success, 500-after-output (transport disconnect), stall and cancel.
It adds thinking-signature, redacted-thinking, refusal, error-after-output,
stop-sequence, length, unknown/contradictory-finish, invalid-args and
error-before-output. `tests/anthropic_provider.rs` uses the real client through
`tests/support/sse_server.rs`, compares exact event sequences, and inspects
captured HTTP requests for pairing, cache placement, stable bytes and headers.
The server uses only loopback and throwaway credentials; no real key or operator
input is required. Tests also exercise the actual profile factories and
transcript law checker.

`HarnessProviderStream` models the shared retry/terminal ownership and the
Anthropic finish mapping with paired broken retry/finish/ending controls.
Native/TLC parity checks all reachable states and edges. Byte parsing, token
arithmetic, signature equality, redaction, real timers and actual processes
remain implementation witnesses, not claims of Rust refinement.

Protocol references: [Messages streaming](https://platform.claude.com/docs/en/build-with-claude/streaming),
[prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching).
