---
id: 01M6F45VPBS6BPP3C24D0MBDA3
number: 5971
type: interface
title: 'ICD: ACP agent backend — the harness as an ACP v1 client of an external agent (v0.2)'
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[protocol]]", rel: carries }
  - { to: "[[gateway]]", rel: uses }
  - { to: "[[acp]]", rel: mirrors }
  - { to: "[[acp-protocol-and-adapters]]", rel: builds-on }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: constrained-by }
created: 2026-11-02T22:00:00Z
updated: "2026-11-09T20:50:08.108435+00:00"
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
- **Model.** The profile `model:` is matched against the `session/new` result's config option with `category == "model"` (or `id == "model"`): exact value, else a value ending `/<model>`, else an offered alias equal to a whole `-`, `/` or `.` separated token of the requested model (`claude-haiku-5-5` matches `haiku`; `pick_model`). Match ≠ current → `session/set_config_option {configId: "model"}` before the first prompt; refused → Warn notice, the turn runs on the agent's model. Not offered → Warn notice naming the agent's current model. There is no `session/set_model` (stable has none).
- **Lazy start.** The agent is spawned by the first `UserTurn`, not by `StartSession`. Spawn failure ends that turn `Error{class: Config}`; the next turn tries again.

- **Full-access default (operator, 2026-11-09).** All ACP profiles, including
  previously saved profiles without a policy field, default to `policy: yolo`.
  Explicit `policy: ask` remains an opt-in; launch `--yolo` overrides it.
  Codex ACP starts with `INITIAL_AGENT_MODE=agent-full-access`; Claude receives
  `allowDangerouslySkipPermissions: true` in its session options and confirms
  `bypassPermissions` before prompting. Gemini and OMP receive `--yolo`; OMP uses
  its native `omp --mode acp` server, while Pi keeps `pi-acp` and its unrestricted
  RPC tools. Grok uses `--always-approve` with `GROK_SANDBOX=off`, OpenCode uses
  `OPENCODE_PERMISSION={"*":"allow"}`, and Goose uses `GOOSE_MODE=auto`.
  Only an actually advertised full-access permission mode/config option is
  selected, with its acknowledgement preceding the prompt. Pi thinking modes
  and model identifiers are not permission modes. Refusal ends the turn visibly.

## 2. Op → wire

| Op | Wire | Notes |
|---|---|---|
| `UserTurn` | queued (cap 64, else `InvalidInput`); when idle: `initialize` → `session/new {cwd, mcpServers: []}` → [`set_config_option`] → `session/prompt` | ACP has no steering: a turn sent while one runs waits its turn (`TurnQueued`) |
| `Interrupt` running | `session/cancel` + every pending permission answered `{"outcome":"cancelled"}` | grace defaults to `CANCEL_GRACE = 5 s` (test override `BBOXY450_ACP_CANCEL_GRACE_MS`), §4 |
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
| `session/request_permission` with no turn running | opens an agent-initiated turn (§4.5), then the row above |
| `session/request_permission` after cancel | `{"outcome":"cancelled"}` at once |
| anything else (`fs/*`, `terminal/*`, `elicitation/*`, unknown) | JSON-RPC error `-32601` "… is not supported by this client" |

Option choice: Allow → `allow_once`; under yolo only, fall back to `allow_always`
when no `allow_once` is offered. AllowAlways → `allow_always`; Deny →
`reject_once`, else `reject_always`. No matching option → `cancelled`, recorded
as Deny "cancelled". Explicit ask/unattended behavior is unchanged. Cancellation
always answers `cancelled`, even under yolo.

Client capabilities sent: `fs.readTextFile = fs.writeTextFile = false`, `terminal = false`; `clientInfo {name: "bboxy450-harness", version}`. The agent uses its own tools.

## 4. Turn lifecycle and cancel contract

States per prompt: `queued → starting (initialize/new/set_model) → prompting ⇄ awaiting-permission → cancelling → ended`.

1. **One ending.** Each `TurnStarted` gets exactly one `TurnEnded`; no item is open after it (`end_turn` settles permissions, flushes text, closes tools — unreported tools `Interrupted` or the failure status — and completes the plan).
2. **Cancel.** After `session/cancel` the harness answers every pending permission `cancelled` and keeps applying `session/update`s until the prompt response. `stopReason: cancelled` → `Interrupted` if a face interrupted, else `Cancelled`. If the prompt was never sent, the turn ends `Interrupted` locally.
3. **Grace.** If the agent has not ended the prompt within the cancel grace after cancel (default `CANCEL_GRACE`, overridden by a valid unsigned millisecond value in `BBOXY450_ACP_CANCEL_GRACE_MS`, sampled once at actor load; invalid values retain the default), its process group is killed, a Warn notice is written and the turn ends `Interrupted`.
4. **Stop reasons.** `end_turn` → EndTurn; `max_tokens` → MaxTokens; `max_turn_requests` → MaxRounds; `refusal` → `Error{BadRequest}`; anything else → `Error{Malformed}`.
5. **Agent-initiated turns** ([[d5980-agent-initiated-turns-are-first-class]]). An agent may start work with no prompt open (Claude Code's follow-up after a background job ends; a scheduler fire). The first work update (`agent_message_chunk`, `agent_thought_chunk`, `tool_call`, `tool_call_update`, `plan`) or permission request that arrives while no turn runs opens a turn: `TurnStarted{origin: agent}`, no `UserMessage`, `cause: None`, `Phase::Thinking`; from then on §3 and §5 apply unchanged. The harness never sends a prompt it was not given: no poll, no synthetic wake. The turn ends when the agent gives an explicit end signal (ACP v1 has none today, §8) or after `AGENT_TURN_QUIET` (30 s; test override `BBOXY450_ACP_AGENT_TURN_QUIET_MS`, sampled once at actor load) with no tool item and no permission open: `TurnEnded{reason: quiet{ms}}`, never `end_turn`, `usage: None`. A queued `UserTurn` shortens the wait to "nothing open". Session gauges (`usage_update` and the §5 ignored kinds) open nothing. User turns carry `origin: user`; an absent origin means user (logs before IR 1.2).

## 5. Update → Item

| `session/update` | Item |
|---|---|
| `agent_message_chunk` / `agent_thought_chunk` | `AgentMessage` / reasoning, streamed as deltas, one item per run of chunks |
| `tool_call` / `tool_call_update` | `ToolCall` keyed by `toolCallId`; kind via `tool_kind`; content → `OutputBlock` (`content`, `diff` as unified diff, `terminal` id); `locations` display-only |
| `plan` | `Plan`, replaced whole on each update |
| `usage_update` | `Item::Usage` with `scope=session`, `source=reported`, `context_used=used`, `context_window=size`, backend agent name/version from initialize (command fallback), and model from the agent model config when known. Cost is reported, currency-preserving, rounded to nearest micro-unit with `price_source=provider`; USD also writes legacy `cost_micros_usd`. Unknown call input/output are 0 and named in `unreported`. Empty/malformed unreported values are omitted, and an update with no valid number creates no item. Accepted even between turns (idle item), per [[log-ir-v1]] §4a |
| `available_commands_update`, `current_mode_update`, `config_option_update`, `session_info_update`, `user_message_chunk` | ignored |
| any work row above while no turn runs | opens an agent-initiated turn (§4.5), then maps as above |
| anything else | `Item::Native{provider: "acp:<agent>", record_type: <kind>, raw}` in or out of a turn ([[log-ir-v1]] §7); since f73a034 (earlier: one Warn `Notice` per unknown kind) |

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
| mapping, approvals, `-32601`, unattended `reject_once`, interrupt keeps late updates, crash restart, malformed line, handoff on switch, model matching | `tests/acp_agent.rs` (scripted agent, including usage retention and kill-after-grace group witness) |
| a real agent end to end | not in `make test` (needs a key). Night run: one turn on `claude-agent-acp` and `codex-acp` with `check_transcript` OK; **SKIP** without a key, never PASS |
| agent-initiated turn (§4.5): idle work or an idle permission request opens it, policy resolves it (never `cancelled`), quiet ends it once, a user turn queues behind it | `tests/acp_agent.rs` and model `HarnessAcpAgentTurn` with broken control `BrokenHarnessAcpAgentIdleCancel`, brief [[2026-11-03-agent-initiated-turns]]; the real-adapter run in that brief's report. **unproven** until it lands. Witness of the gap itself: d5980 (adapter 0.100.0, Claude Code 2.1.304) |

**Remaining gap (unproven).** No test that `claude-ai-login` is never sent beyond the absence of any `authenticate` call.

## 8. Versioning

- The wire is ACP `protocolVersion 1`. The SDK major is not the wire version.
- `agent-client-protocol` 5.0.0 (2026-11-02, schema 1.15, `v1::schema`/`v2::schema`, dropped responders answer with an error) is the next pin. Moving it is **one change to both** [[acp]] and this ICD: the face and the backend never sit on different SDK majors. Acceptance for the move: `tests/acp_face.rs`, `tests/acp_agent.rs` and the `tests/fixtures/acp/*.jsonl` goldens pass unchanged, and `cargo tree -d` shows one copy of the crate.
- No `unstable_*` feature is enabled. A protocol v2 agent is refused (§6) until a v2 row is written here.
- New update kinds are additive: they become `Item::Native` until mapped. A newly mapped kind is a §5 row change.
- Agent-initiated work is bracketed by quiet on v1 (§4.5). An explicit bracket exists in two places and is adopted only by a §4.5 row change: ACP v2 `state_update` (`idle`/`running`/`requires_action`), and claude-agent-acp's AIR `asyncTasks` client extension (`async_task_spawned {taskType: "turn"}` … `async_task_state_update {state}`), which also flips the adapter into AIR-client mode and changes when `rawInput` arrives.

## Change log

- v0.2, 2026-11-03, Lumen Foxglove <ARCHITECT>: agent-initiated turns (§4.5; §3 idle permission opens a turn instead of `cancelled`; §5 idle work row; §7 row; §8 bracket note), from [[d5980-agent-initiated-turns-are-first-class]]. §5 "anything else" corrected to `Item::Native` (f73a034).

- p6-seat-state, 2026-11-02, Executor under the Oracle brief: alias matching, additive session usage mapping and a test-only grace override close the assigned gaps. `tests/acp_agent.rs` witnesses an ignored cancel, Interrupted ending, Warn notice and `kill(-pgid, 0) == ESRCH` for an agent with a live child. Usage field meanings checked against the [ACP v1 prompt-turn source](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/protocol/v1/prompt-turn.mdx). On the Oracle update after rebasing onto c4246de, usage uses log IR v1 §4a: new optional Usage fields and definitions; existing names, types and meanings stay unchanged. The earlier Info Notice representation is superseded.

- v0.1, 2026-11-02, Lumen Foxglove <ARCHITECT>: baselined from 3d76710 and [[acp-protocol-and-adapters]].
