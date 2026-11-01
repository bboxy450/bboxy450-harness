---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
id: 01M6AVTZZ1J8WKRH8H8Y00BHWF
links:
- rel: index
  to: '[[interfaces]]'
- rel: carries
  to: '[[protocol]]'
- rel: uses
  to: '[[gateway]]'
- rel: feeds
  to: '[[mcp]]'
number: 1982
provenance:
  digest: sha256:37d9d267e391e5d883f5dccd9517407f0c44b790a2a5916e4e296cbf46c0853b
  frontmatter_digest: sha256:7f5e3ab52c7fd0fe8af4a0b856d1b3ecf317e38751975473fd9eb37f13f18c68
  log: .bboxy/log/2026-11-01.jsonl#12464
  sig: VMrk9gHH+BneFjVRbEgo2Wnbc/UXJnTvv+QN9EcC9qXrveRZ6nirDwu0gsOgIOF5AO+mmgeqEuV4p1qCuWesBQ==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: baselined
title: 'ICD: ACP face — Agent Client Protocol v1 over stdio (v0.1)'
type: interface
updated: 2026-11-01T06:30:09.944Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# ICD: ACP face

| | |
|---|---|
| Side A | the engine through `Host` and `InProc` ([[gateway]] §1b) (E1) |
| Side B | any ACP client (Zed, bboxy450-term, `winnow`) that speaks Agent Client Protocol v1 to `bboxy450-harness acp` over stdio |
| Owner | `src/frontends/acp.rs` (E1c) |
| Pin | crate `agent-client-protocol = "=3.4.0"`, which pins `agent-client-protocol-schema = "=1.12.0"`; `ProtocolVersion::V1`. **No `unstable_*` feature is enabled.** Moving the pin is an ICD change |
| Framing | JSON-RPC 2.0, one message per line on stdout. Stdout carries nothing else; logs go to stderr, scrubbed by the redactor |

The type and method facts below were read from schema 1.9.1 (`agent-client-protocol-schema-1.9.1/src/v1/`) and the ACP repository at b4c72a8. The implementation **must confirm them against 1.12.0** before coding. A difference is reported to the Architect; it is not silently adapted.

## 1. Methods

| ACP method (agent side) | Harness | Notes |
|---|---|---|
| `initialize` | answers `protocolVersion: 1`, whatever the client asks for | capabilities in §1a |
| `authenticate` | not advertised (`authMethods: []`) | `-32601` if called |
| `session/new {cwd, mcpServers}` | `Host::create(StartSession{cwd, profile: --profile, fork: None})` | `sessionId` is the harness `SessionId` ULID string; stdio `mcpServers` join the session ([[mcp]] §1) |
| `session/load {sessionId, cwd, mcpServers}` | `Host::open(sessionId)`, then the §3 replay, then the response | `cwd` must equal `meta.cwd`, else `-32602`. A closed or unknown session → `-32002` resource not found |
| `session/prompt {prompt}` | `Op::UserTurn{blocks, mode: Queue}` | the response is sent at that turn's `TurnEnded` (§2) |
| `session/cancel` (notification) | `Op::Interrupt{turn: None}` | the pending `session/prompt` resolves with `stopReason: cancelled` within 2 s (law [[cancel-is-cancel]]) |
| `session/close` | `Op::Close` | advertised in `sessionCapabilities.close` |
| `session/set_mode`, `session/set_config_option`, `session/list`, `session/resume`, `session/delete`, `logout` | not supported, not advertised | `-32601` |
| any unstable method (`session/fork`, `providers/*`, `nes/*`, `mcp/*`, `document/*`) | not supported | `-32601` |
| any unknown method | | `-32601`. An unknown *notification* is ignored and counted |

The harness **calls no client method except `session/update` and `session/request_permission`**. It never uses client `fs/*`, `terminal/*` or `elicitation/*`. Tools run in the harness under its own policy, so a client without those capabilities loses nothing.

### 1a. Advertised capabilities

`agentCapabilities`:
- `loadSession: true`;
- `promptCapabilities: {image: Caps.images, audio: false, embeddedContext: true}`;
- `mcpCapabilities: {http: false, sse: false}`;
- `sessionCapabilities: {close}`.

Nothing else is advertised.

### 1b. Prompt content → `InputBlock`

| ACP `ContentBlock` | `InputBlock` |
|---|---|
| `text` | `Text` |
| `image` | `Image` (base64 decoded into a blob) |
| `resource_link` with a `file://` URI | `Mention{path}` |
| `resource` (embedded) | `Text`, prefixed by one line `<uri>:` |
| `audio`, or any other | `Text` with `[unsupported <type> omitted]`, plus one `Notice{Warn}` |

### 1c. Concurrency of the face

- **Handlers never hold the dispatcher.** The SDK dispatches requests one at a time (kittens `src/acp.rs:10-17`). A `session/prompt` handler therefore sends the op and parks the reply on the session's own task, so `session/cancel` and other sessions' requests run while a prompt is pending.
- **Several sessions per process.** Each session has its own `InProc` and its own subscription task.
- **A second prompt while one is running** is queued (`TurnMode::Queue`). Each prompt is answered at its own turn's `TurnEnded`.

## 2. `TurnEnded` → `session/prompt` response

| `StopReason` | Response |
|---|---|
| `EndTurn` | `{stopReason: "end_turn"}` |
| `Cancelled`, `Interrupted` | `"cancelled"` |
| `MaxTokens` | `"max_tokens"` |
| `MaxRounds` | `"max_turn_requests"` |
| `Error{class, message}` | JSON-RPC error `-32603`, `data: {class, message}` (already redacted) |

A refusal from the provider is `Error{class: BadRequest}` in v0.3. ACP's `refusal` is not used.

## 3. Item → `session/update`

The face subscribes with `ephemeral: true` for live prompts. For `session/load` it first replays durable history with `ephemeral: false`, then responds, then goes live ([[gateway]] §2 order).

ACP chunks only append, so the face keeps `sent[item]`, the count of characters already sent for each text-bearing item. A delta, an `ItemSnapshot` or an `ItemCompleted` sends `body[sent..]` if the body extends what was sent. **Dropped deltas are repaired by sending the missing suffix.** If a body does not extend what was sent (impossible for append-only text), the face sends nothing more for that item and counts a `divergence`. Tests assert the count stays at 0.

| Item / event | `sessionUpdate` | Mapping |
|---|---|---|
| `UserMessage` | `user_message_chunk` | **replay only** (a live prompt is not echoed); one chunk per block |
| `AgentMessage` | `agent_message_chunk` | `{type: text, text: suffix}` |
| `Reasoning` | `agent_thought_chunk` | `summary` suffix; `raw` and `provider_opaque` are never sent |
| `ToolCall` at `ItemStarted{head}` | `tool_call` | `toolCallId` = harness `ItemId`; `title` = `name`; `kind` = `tool_kind` 1:1 (`Unknown` → `other`); `status: in_progress`; `rawInput` once complete |
| `ToolCall` at `ItemCompleted` | `tool_call_update` | `status`: `completed`, or `failed` for `Failed`/`Cancelled`/`Interrupted`; `content`: the output as text, clipped to 64 KiB; `locations`: `{path, line}` |
| `FileChange` (child of a tool call) | `tool_call_update` on the parent | a `content` text block holding the `unified` diff in a ```` ```diff ```` fence. **Degraded:** ACP `diff` needs full old and new text, which the item does not carry |
| `Terminal` | `tool_call_update` on the parent, or its own `tool_call` with `kind: execute` if it has no parent | `content` text: `$ <command>`, then `live` at snapshot or the output at completion (clipped to 64 KiB), then `exit <n>`. **Degraded:** ACP `terminal` content needs a client-created terminal, which the harness does not use |
| `Plan` | `plan` | full replacement; `{content: text, priority (Medium if None), status}`. `Cancelled` → `completed` with the content prefixed `(cancelled) ` |
| `Approval` | `session/request_permission` (a request, §4) | |
| `TurnDiff`, `Image` (agent side), `Usage`, `Notice`, `Compaction`, `ContextTrim`, `Subagent`, `Unknown` | not sent | `Notice{Error}` text also goes to stderr. The ACP updates for these are unstable features |

## 4. Approvals

- **Request.** `Approval` `ItemStarted` sends `session/request_permission {sessionId, toolCall: <tool_call_update for the parent>, options}`. Each offered `ApprovalDecisionKind` becomes a `PermissionOption`: `Allow` → `allow_once`, `AllowAlways` → `allow_always`, `Deny` → `reject_once`. The `optionId` is the snake_case decision name.
- **Response.**
  - `{outcome: selected, optionId}` → `Op::ResolveApproval{decision}`, with `Resolver::Face("acp")`.
  - `{outcome: cancelled}` → `Deny{reason: "client cancelled"}`.
  - A JSON-RPC error, or a client that disconnects → the profile's `on_unattended` (default deny, [[architect-review-v0]] §7).
- **The engine never waits on the client.** The approval is an ordinary pending approval. An `Interrupt` or `Close` resolves it, and the face then drops the late response (`AlreadyResolved`).

## 5. Lifecycle and failure

- **Start.** The client spawns `bboxy450-harness acp [--profile <name>]`.
- **Stdin EOF or a broken stdout** = the client went away. Every session the face holds gets `Op::Close`, and the process exits within 2 s.
- **A malformed line** → JSON-RPC parse error `-32700`. The connection stays open.
- **An outbound message over 256 KiB** is clipped at its text fields; it is never split.

## 6. Verified by

| Clause | Evidence |
|---|---|
| §1, §1a methods and capabilities | `tests/acp_face.rs`, using a scripted ACP client over a pipe and the scripted provider: the `initialize` capabilities equal §1a byte for byte; every unsupported method in §1 returns `-32601`; an unknown notification is ignored |
| §1c dispatcher | `session/cancel` sent 100 ms after `session/prompt` gives `stopReason: cancelled` in ≤ 2 s; two sessions prompt concurrently and both finish |
| §2 stop reasons | one scripted turn per row; `Error` gives `-32603` with `data.class` |
| §3 mapping and suffix repair | golden transcripts `tests/fixtures/acp/*.jsonl`. Text, reasoning, tool call, file change and terminal come from **real scripted-provider turns**. `Plan`, and any other item kind that no engine path produces yet, comes from the existing all-items protocol fixture, loaded with `session/load` and mapped by the same face code. No plan-producing tool or event is added for the test; a lag-injected run (ephemeral drops forced) produces the same concatenated text as an undropped run; `divergence = 0` |
| §3 load | `session/load` of a recorded session sends every replayed update **before** its response, and the replay equals the golden |
| §4 approvals | selected/cancelled/error/disconnect each give the stated decision; a late response after `Interrupt` is dropped |
| §5 lifecycle | closing stdin mid-turn: the process exits in ≤ 2 s, no child is alive, and `check_transcript` passes on the log |
| one prompt, one response | the prompt ↔ `TurnEnded` pairing is a projection of HarnessTurnLifecycle's one-outcome property; `acp_face` is its witness. No separate model |

## Change log

- **2026-11-01, v0.1 baselined at the Oracle's request** (message 01M6AVJ56YFJSKHDJ9Q6Z17QNH). Alternatives rejected:
  - client fs/terminal delegation: two execution paths, and a policy bypass;
  - unstable ACP features: they track a moving draft;
  - a full ACP `diff`: it needs old and new text in `FileChange`. That would be an additive field later, if a client needs it.
- **2026-11-01, at E1's question** (message 01M6AWED4QVM27ZVE6ZHP7WY5D). §6: `Plan` is witnessed through the all-items fixture via `session/load`, because no engine path produces a plan in v0.3. A plan-producing tool is a separate design decision, not a test fixture.

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: ACP clients (E1c implements the face).
