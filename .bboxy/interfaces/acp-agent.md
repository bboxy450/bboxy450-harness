---
id: 01M6F45VPBS6BPP3C24D0MBDA3
number: 5971
type: interface
title: 'ICD: ACP agent backend — the harness as an ACP v1 client of an external agent (v0.1)'
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[protocol]]", rel: carries }
  - { to: "[[gateway]]", rel: uses }
  - { to: "[[acp]]", rel: mirrors }
  - { to: "[[acp-protocol-and-adapters]]", rel: builds-on }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: constrained-by }
created: 2026-11-02T22:00:00Z
updated: 2026-11-02T22:00:00Z
---
# ICD: ACP agent backend

[[acp]] is the harness *serving* ACP to a client. This ICD is the other direction: the harness *driving* an external ACP agent and writing what it does into our log ([[d5972-our-log-format-is-the-center]]). Baselined against `main` 3d76710. Research input: [[acp-protocol-and-adapters]] (corrections 1–10 are binding here).

| | |
|---|---|
| Side A | the session actor `AcpAgentActor` (`src/backend/acp_agent.rs`), behind the same `Gateway`/`InProc` as the engine ([[gateway]]); every face sees only [[protocol]] `Event`s and sends only `Op`s |
| Side B | any ACP v1 agent spawned from a profile with `provider: acp-agent` (Claude Agent, codex-acp, Gemini CLI, goose, opencode, …). Untrusted child process |
| Owner A | the harness Executor holding `src/backend/acp_agent.rs`; interface changes need the Architect |
| Owner B | the agent vendor. We control only the profile line that launches it |
| Wire | JSON-RPC 2.0, one UTF-8 message per line, over the child's stdin/stdout; stderr is a 2 KiB tail (`STDERR_TAIL`), scrubbed by the redactor before it reaches the log |
| Protocol | `protocolVersion: 1` sent; any other echoed version ends the agent (`acp_agent.rs` `Call::Initialize`) |
| Crate | `agent-client-protocol = "=3.4.0"` (features `stdio`), shared with [[acp]]. The backend hand-rolls the JSON and uses the crate's schema only to validate the outgoing `PromptRequest` (`acp_agent.rs:478`). See §8 |

## 1. Launch and environment

- **Profile.** `AgentSpec { command: Vec<String>, env: BTreeMap }` (`src/provider/profile.rs:48`). `command[0]` is exec'd with no shell in the session cwd through `OwnedChild` (own process group). Launchers pin exact versions (`npx pkg@x.y.z`); a floating version is a profile defect, not a runtime choice.
- **Environment.** The child **inherits the harness environment** and the profile's `env` is layered on top (`Command::envs`). A value starting `!` is a command resolved at spawn; its trimmed output is registered with the redactor before use. The profile never holds a secret literal.
- **Auth.** The harness never calls `authenticate` and advertises no `auth.terminal`. An error with code `-32000` or text containing "auth" ends the turn `Error{class: Auth}` with the hint "authenticate the agent in a terminal (API key, console login, Bedrock or Vertex)". **Never** offered, selected or triggered: `claude-ai-login` (Anthropic terms, research correction 10). Claude agents run on `ANTHROPIC_API_KEY`, console login, Bedrock or Vertex only.
- **Model.** The profile `model:` is matched against the `session/new` result's config option with `category == "model"` (or `id == "model"`): exact value, else a value ending `/<model>` (`pick_model`). Match ≠ current → `session/set_config_option {configId: "model"}` before the first prompt; refused → Warn notice, the turn runs on the agent's model. Not offered → Warn notice naming the agent's current model. There is no `session/set_model` (stable has none).
- **Lazy start.** The agent is spawned by the first `UserTurn`, not by `StartSession`. Spawn failure ends that turn `Error{class: Config}`; the next turn tries again.

## 2. Op → wire

| Op | Wire | Notes |
|---|---|---|
| `UserTurn` | queued (cap 64, else `InvalidInput`); when idle: `initialize` → `session/new {cwd, mcpServers: []}` → [`set_config_option`] → `session/prompt` | ACP has no steering: a turn sent while one runs waits its turn (`TurnQueued`) |
| `Interrupt` running | `session/cancel` + every pending permission answered `{"outcome":"cancelled"}` | grace `CANCEL_GRACE = 5 s`, §4 |
| `Interrupt` queued | none | the queued turn emits `TurnStarted` + `TurnEnded{Interrupted}` |
| `ResolveApproval` | `request_permission` reply | §3 |
| `SetProfile` | none | refused `InvalidInput`: an ACP session keeps its agent. Switching is `Host::switch` (fork + handoff, [[d5972-our-log-format-is-the-center]] §3) |
| `Compact` | none | refused `NothingToCompact`; the agent owns its context |
| `Close` | `session/cancel` if running, then SIGKILL of the group | open turn ends `Cancelled`; queued turns end `Cancelled`; then `SessionClosed` |

The harness sends no `session/load` or `session/resume`: a reopened or forked session starts a **fresh** agent session and hands it the conversation as text (§5).

## 3. Agent → client

**Every agent→client request is answered.** An unanswered request wedges the turn.

| Request | Answer |
|---|---|
| `session/request_permission` while a turn runs and no cancel is pending | an `Approval` item (effect from the tool kind: edit/delete/move → Write, execute → Exec, fetch → Network, else Unknown) and `Phase::Approval`; then, in order: profile yolo → `allow_once` (resolver `profile-yolo`); no approver attached → `on_unattended` (allow → `allow_once`, deny → `reject_once`/`reject_always` first present, resolver `Unattended`); else a face's `ResolveApproval` |
| `session/request_permission` with no turn or after cancel | `{"outcome":"cancelled"}` at once |
| anything else (`fs/*`, `terminal/*`, `elicitation/*`, unknown) | JSON-RPC error `-32601` "… is not supported by this client" |

Option choice: Allow → `allow_once`; AllowAlways → `allow_always` (only from a face, never automatically); Deny → `reject_once`, else `reject_always`. No matching option → `cancelled`, recorded as Deny "cancelled". `*_always` is never picked by a rule or by unattended policy.

Client capabilities sent: `fs.readTextFile = fs.writeTextFile = false`, `terminal = false`; `clientInfo {name: "bboxy450-harness", version}`. The agent uses its own tools.

## 4. Turn lifecycle and cancel contract

States per prompt: `queued → starting (initialize/new/set_model) → prompting ⇄ awaiting-permission → cancelling → ended`.

1. **One ending.** Each `TurnStarted` gets exactly one `TurnEnded`; no item is open after it (`end_turn` settles permissions, flushes text, closes tools — unreported tools `Interrupted` or the failure status — and completes the plan).
2. **Cancel.** After `session/cancel` the harness answers every pending permission `cancelled` and keeps applying `session/update`s until the prompt response. `stopReason: cancelled` → `Interrupted` if a face interrupted, else `Cancelled`. If the prompt was never sent, the turn ends `Interrupted` locally.
3. **Grace.** If the agent has not ended the prompt `CANCEL_GRACE` after cancel, its process group is killed, a Warn notice is written and the turn ends `Interrupted`.
4. **Stop reasons.** `end_turn` → EndTurn; `max_tokens` → MaxTokens; `max_turn_requests` → MaxRounds; `refusal` → `Error{BadRequest}`; anything else → `Error{Malformed}`.

## 5. Update → Item

| `session/update` | Item |
|---|---|
| `agent_message_chunk` / `agent_thought_chunk` | `AgentMessage` / reasoning, streamed as deltas, one item per run of chunks |
| `tool_call` / `tool_call_update` | `ToolCall` keyed by `toolCallId`; kind via `tool_kind`; content → `OutputBlock` (`content`, `diff` as unified diff, `terminal` id); `locations` display-only |
| `plan` | `Plan`, replaced whole on each update |
| `available_commands_update`, `current_mode_update`, `config_option_update`, `session_info_update`, `usage_update`, `user_message_chunk` | ignored |
| anything else | one Warn `Notice` per unknown kind (`warned` set) |

**Handoff.** A fresh agent on a session that already has turns gets `handoff()` text (user/assistant messages and one line per tool call, newest kept, 24 KiB budget, `[earlier turns omitted]` marker) prepended to its first prompt, and an Info notice counting the turns. The log, not the agent, is the session's memory.

## 6. Failure behaviour

| Failure | Behaviour |
|---|---|
| line > `MAX_LINE` (8 MiB), non-JSON, or neither request nor response | agent killed; running turn ends `Error{MidStream}` with the stderr tail; idle → Warn notice |
| stdout EOF | same, after up to 500 ms for stderr to drain |
| error response to `initialize` / `session/new` | agent killed; turn ends `Error{Auth}` (auth hint) or `Error{BadRequest}` |
| error response to `session/prompt` | turn ends as above; agent kept |
| protocol version ≠ 1 | agent killed with "speaks protocol N, not 1" |
| the agent never ends a cancelled prompt | §4.3 |
| the harness closes | the group is SIGKILLed; no orphan outlives the session |

Nothing on this boundary blocks a face: ops are admitted by the gateway lanes, the wire reader and writer are separate tasks, and the actor's `select!` serves control ops first.

## 7. Verified by

| Claim | Evidence |
|---|---|
| one ending; cancel answers every pending permission; cancelled turn ends | model `HarnessAcpAgentTurn` + `BrokenHarnessAcpAgentTurn` (`tests/formal/models.rs:2011`), contract in `verification/contracts.toml:194` |
| mapping, approvals, `-32601`, unattended `reject_once`, interrupt keeps late updates, crash restart, malformed line, handoff on switch, model matching | `tests/acp_agent.rs` (scripted agent, 8 tests) |
| a real agent end to end | not in `make test` (needs a key). Night run: one turn on `claude-agent-acp` and `codex-acp` with `check_transcript` OK; **SKIP** without a key, never PASS |

**Gaps (unproven).** The kill-after-grace path (§4.3) has no witness: `tests/acp_agent.rs:263` asserts the agent ended *before* the grace. `usage_update` is dropped, so ACP turns record no usage. No test that `claude-ai-login` is never sent beyond the absence of any `authenticate` call.

## 8. Versioning

- The wire is ACP `protocolVersion 1`. The SDK major is not the wire version.
- `agent-client-protocol` 5.0.0 (2026-11-02, schema 1.15, `v1::schema`/`v2::schema`, dropped responders answer with an error) is the next pin. Moving it is **one change to both** [[acp]] and this ICD: the face and the backend never sit on different SDK majors. Acceptance for the move: `tests/acp_face.rs`, `tests/acp_agent.rs` and the `tests/fixtures/acp/*.jsonl` goldens pass unchanged, and `cargo tree -d` shows one copy of the crate.
- No `unstable_*` feature is enabled. A protocol v2 agent is refused (§6) until a v2 row is written here.
- New update kinds are additive: they become a Notice until mapped. A newly mapped kind is a §5 row change.

## Change log

- v0.1, 2026-11-02, Lumen Foxglove <ARCHITECT>: baselined from 3d76710 and [[acp-protocol-and-adapters]].
