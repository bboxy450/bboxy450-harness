---
id: 01M6GT6P7B8WQP6AKSD3X90QJZ
number: 5981
type: task
title: 'ACP backend: an update or permission request with no turn running opens an agent-initiated turn (origin agent), resolved by policy, ended by quiet; IR 1.2 origin + Quiet'
status: open
role: executor
links:
  - { to: "[[d5980-agent-initiated-turns-are-first-class]]", rel: implements }
  - { to: "[[acp-agent]]", rel: implements }
  - { to: "[[log-ir-v1]]", rel: implements }
allowed_paths:
- src/backend/acp_agent.rs
- src/protocol/
- src/frontends/
- tests/acp_agent.rs
- tests/fixtures/acp-agent/fake_agent.py
- tests/fixtures/ir-v1/v1.2-agent-turn.jsonl
- tests/ir_v1_corpus.rs
- tests/formal/models.rs
- tests/protocol_golden.rs
- protocol.schema.json
- verification/protocol-v1.lock.json
- verification/contracts.toml
- verification/coverage.toml
- .bboxy/reports/
verify: make test
created: 2026-11-03T13:55:00Z
updated: 2026-11-03T13:55:00Z
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness, own worktree off origin/main. Binding: decision d5980, ICD `.bboxy/interfaces/acp-agent.md` v0.2 (§3, §4.5, §5), spec `.bboxy/specs/log-ir-v1.md` §4b. The lock is refreshed only by `make lock-ir`; existing frozen corpus files are never edited.

**Why.** Claude Code behind claude-agent-acp wakes itself about one second after a background job ends and streams the follow-up as `session/update`s with no prompt open (witness in d5980). Today `src/backend/acp_agent.rs:1341` records those updates as `Native` only, and `:1090` answers any `session/request_permission` with no turn running `cancelled`, so the woken agent's tools fail and nobody sees it. Three executor seats sat idle six hours this way.

## Steps

1. **Protocol** (`src/protocol/mod.rs`): add `TurnOrigin` (`#[serde(tag = "kind", rename_all = "snake_case")]`: `User`, `Agent`, `#[serde(untagged)] Unknown(Value)`), `TurnStarted.origin: Option<TurnOrigin>` (`default`, `skip_serializing_if` none; absent means user), and `StopReason::Quiet { ms: u64 }`. `PROTOCOL` → `{1, 2}`. Regenerate `protocol.schema.json` (`tests/protocol_golden.rs`), then `make lock-ir` and commit the lock with the change. Nothing existing changes name, type, tag or meaning.
2. **Open on work** (`acp_agent.rs` `update_inner`, today `:1341`): when `self.running.is_none()` and the kind is `agent_message_chunk`, `agent_thought_chunk`, `tool_call`, `tool_call_update` or `plan`, call a new `start_agent_turn()` (fresh `TurnId`; `TurnStarted{origin: Some(Agent), model}`; `Running` with `prompt_sent: true`, `cause: None`; `Phase::Thinking`) and then map the update exactly as inside a turn. Every other kind while idle keeps today's path (`usage_update` → idle `Usage`; the rest → idle `Native`). User turns set `origin: Some(User)` in `start()`.
3. **Open on a permission request** (`request()`, today `:1090`): `session/request_permission` with no turn running → `start_agent_turn()` first, then the normal path (Approval item, yolo rule, unattended policy, face). After `session/cancel` the answer stays `cancelled` (unchanged).
4. **Quiet end.** `AGENT_TURN_QUIET = 30 s`, overridden by a valid unsigned millisecond `BBOXY450_ACP_AGENT_TURN_QUIET_MS` sampled once at actor load (same shape as `cancel_grace()`). An agent-initiated `Running` carries `quiet_at: Option<Instant>`, re-armed on every wire message of that turn; a `sleep_until(quiet_at)` arm in the `select!` next to `cancel_at`. On fire: a tool item or permission still open → re-arm; otherwise `end_turn(StopReason::Quiet{ms}, ItemStatus::Completed)` with `ms` the actual silence. While the op queue is non-empty, end as soon as nothing is open (`ms` = actual silence). `TurnEnded.usage` is `None` for agent turns (never invented; the `usage_update` items are already recorded). The idle heartbeat (`:373`) is untouched.
5. **Broken control.** `BBOXY450_ACP_IDLE_CONTROL=hide` (read once, test-only, documented beside the override) restores today's behaviour: idle work → `Native`, idle permission → `cancelled`, no turn opened. The tests in step 7 must fail under it, and one test asserts exactly that.
6. **Fake agent** (`tests/fixtures/acp-agent/fake_agent.py`): scenario `wake`: answer the prompt with `end_turn`; after 50 ms send `agent_message_chunk` "woke", a `tool_call` whose `session/request_permission` it waits on, `tool_call_update {status: completed}`, `agent_message_chunk` "done"; then stay silent. Scenario `wake-twice`: `wake`, then after 300 ms one more chunk (a second agent turn).
7. **Tests** (`tests/acp_agent.rs`, `BBOXY450_ACP_AGENT_TURN_QUIET_MS=200`):
   - `idle_work_opens_an_agent_turn_resolved_by_policy_and_ended_by_quiet`: yolo profile → `TurnStarted{origin: Agent}` with no `UserMessage`; the Approval resolves `allow_once` (`Resolver::Rule{profile-yolo}`), the tool completes, `TurnEnded{Quiet{ms ≥ 200}}`, no item open after it, every item of the turn carries its `TurnId`; `wake-twice` yields two agent turns.
   - unattended deny → `reject_once`, never `cancelled`.
   - a `UserTurn` sent during the agent turn → `TurnQueued`; the agent turn ends `Quiet` as soon as its tool completes (well under 200 ms); the user turn then starts with `origin: User`, in order.
   - the broken control: under `BBOXY450_ACP_IDLE_CONTROL=hide` the first scenario yields no `TurnStarted` and a `cancelled` permission; the test asserts this inversion.
   - `tests/ir_v1_corpus.rs`: freeze the redacted log of the first test as `tests/fixtures/ir-v1/v1.2-agent-turn.jsonl` (`make lock-ir` records its hash); `check_authored` passes on it.
8. **Faces.** Every face that renders a turn must render one without a `UserMessage`: the rich view (`src/frontends/gui/view.rs`), the seat line mode and the ACP face. An agent turn shows a one-line head ("agent continued" or the origin label) where the user bubble would be. The existing fixture-render test renders `v1.2-agent-turn.jsonl` without panic and with its items visible. Parity fixtures and the parity baseline are untouched.
9. **Status.** An agent-initiated turn is an open turn for the seat status report: whatever the status-harness work (#348 amendment, Oracle 2026-11-03) emits while a user turn is open, it emits for an agent turn too. Coordinate through the Oracle if that branch is still open.
10. **Formal.** `HarnessAcpAgentTurn` (`tests/formal/models.rs`): add the agent-origin open (an update with no prompt in flight opens a turn), the quiet end (enabled only when nothing is open) and the queued-prompt shortening. Invariants: one ending per turn; no work item outside a turn; a permission is never answered `cancelled` while a turn is open and no cancel is pending. Broken control `BrokenHarnessAcpAgentIdleCancel` (idle permission answered cancelled) must fail. Update the `verification/contracts.toml` entry (quiet is an abstract timer; real durations are the implementation witness) and `coverage.toml`.

**Verify:** `make test`. Also, outside `make test`, one run against the real adapter with profile `claude-acp` and the witness prompt from d5980 (run `sleep 6; echo WITNESS_DONE` in the background, end the turn; later run `echo WOKE > woke.txt`): the session log shows `TurnStarted{origin: agent}` → the tool call allowed → `TurnEnded{quiet}` and `woke.txt` exists. If no Claude login is present, report SKIP, never PASS.

**Report** in `.bboxy/reports/`: file:line per step, `make test` exit, the broken-control output, the fixture's hash, the real-adapter run's log excerpt (or SKIP), and the ICD §7 / matrix R31 evidence line for the Architect to copy.
