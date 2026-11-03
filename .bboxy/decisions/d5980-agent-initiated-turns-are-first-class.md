---
id: 01M6GT6P7BT7P53BWYSXRAYZ2S
number: 5980
type: decision
title: Agent-initiated turns are first-class in the ACP backend; the harness never invents a prompt
status: accepted
links:
  - { to: "[[acp-agent]]", rel: constrains }
  - { to: "[[log-ir-v1]]", rel: extends }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: builds-on }
  - { to: "[[2026-11-03-agent-initiated-turns]]", rel: implemented-by }
created: 2026-11-03T13:50:00Z
updated: 2026-11-03T13:50:00Z
---
**Date.** 2026-11-03. **Recorded by** Lumen Foxglove <ARCHITECT>.

**Gap (Oracle, live use).** Claude executors on `claude-exec` (claude-agent-acp) start a background job (`Bash run_in_background`, `Monitor`), end their turn "waiting for the completion notice", and never visibly wake. Three seats sat idle about six hours until a `UserTurn` was sent by hand. The question asked: should the harness or the adapter re-prompt the agent after an agent-side async completion?

**Evidence.**
- Sessions `01M6FP48BKHNS7` and `01M6FRKYBRQ45D` (claude-exec): `turn_ended end_turn` at 23:04:25 and 23:03:14 on 2026-11-02. Their jobs finished minutes later (`target/chrome/final3.log` holds `### DONE`, mtime 23:06:49; `target/gaps-hij/after-build4.log` "after build exit 0", mtime 23:03:36). Zero envelopes until the by-hand turn at 05:21:57.
- The seats' harness is `~/Applications/bboxy450-term.app/Contents/MacOS/bboxy450-harness`, PID 12929 started 2026-11-02 14:10, before 5826403 (16:07) and f73a034 (16:45). That code's `update_inner` returned `Ok(())` for any update with no turn running ("content without a turn is not ours to show"). Today's `src/backend/acp_agent.rs:1341` records such updates as `Item::Native` only. In both versions no face shows them and no phase changes.
- `src/backend/acp_agent.rs:1090`: a `session/request_permission` with no turn running is answered `cancelled` at once ([[acp-agent]] §3).
- Witness, 2026-11-03, `claude-agent-acp` 0.100.0 over Claude Code 2.1.304, driven over stdio with exactly the harness's `initialize` capabilities (scratch script `acp-wake-witness.py`; prompt: run `sleep 6; echo WITNESS_DONE` in the background, then end the turn). `session/prompt` returned `stopReason: end_turn` at +4.8 s. At +11.2 s, with no prompt open, the adapter sent `session/update` `agent_message_chunk` "The background command finished with exit code 0 …" and a `usage_update` whose `_meta._claude/origin.kind` is `task-notification`. **The agent wakes itself about one second after the job ends.**
- Same witness with the permission request after the turn answered `cancelled`, as the harness does: the agent's `echo WOKE > woke.txt` ends `tool_call_update status=failed`, the agent reports "aborted at the permission step", and the file is not written. **The harness sabotages the wake it cannot see.**
- Adapter source (`dist/acp-agent.js`: `startAutonomousCycle`, `AUTONOMOUS_RESULT_ORIGINS`, `UNREPORTED_FOREGROUND`): a model cycle Claude Code starts on its own is forwarded as ordinary updates; ACP v1 carries no bracket for it. With the AIR `asyncTasks` client capability the adapter also sends `async_task_spawned {taskType: "turn"}` … `async_task_state_update {state: "completed"}` around the cycle and a shell task lifecycle with `outputFilePath` (witness run `air`).

**Decision.**
1. **The agent wakes itself; the harness never invents a prompt.** The harness sends `session/prompt` only for a `UserTurn` op. No scheduled poll, no synthetic "your job finished" turn. A wake already exists in the agent; a harness prompt would double it, a fabricated user line is invented data in the log (E-law "never invented"), and a clock burns tokens on every idle seat.
2. **Work that arrives while no turn runs opens an agent-initiated turn.** `TurnStarted{origin: agent}`, no `UserMessage`, `cause: None`; every face renders it; `Phase` follows it; a permission request inside it is resolved by the same rules as in a user turn ([[acp-agent]] §3: profile yolo, unattended policy, face). Session gauges (`usage_update`, `available_commands_update`, `session_info_update`, …) stay idle items and open nothing.
3. **It ends on the agent's explicit end signal when the agent gives one, else on quiet.** With nothing open (no tool item, no permission) and no update for `AGENT_TURN_QUIET` (30 s; test override `BBOXY450_ACP_AGENT_TURN_QUIET_MS`), the turn ends `TurnEnded{reason: quiet{ms}}`. Never `end_turn`: the agent did not say it ended. A queued `UserTurn` shortens the wait to "nothing open". A later update opens another agent turn.
4. **IR 1.2 additions** ([[log-ir-v1]] §4b): `TurnStarted.origin?: TurnOrigin {user | agent | unknown}` (absent means user; every 1.0/1.1 log has only user turns) and `StopReason::Quiet{ms}`.
5. **The agent-side rule stays until the fix is deployed.** COMMON.md's "never end a turn to wait" remains in force for seats on a harness without this change; once the term bundle carries it, the rule is lifted for Claude seats. Agents that never self-wake keep it; that is the agent's loop, not ours.

**Alternatives rejected.**
- *Scheduled poll turn from the harness or the term.* Invented input on a timer; cannot tell "waiting for a job" from "done and idle"; the agent is already awake when the job ends.
- *Synthetic wake turn when the adapter reports a task completion.* Needs the AIR `asyncTasks` capability, is Claude-only, and duplicates the follow-up the CLI already runs.
- *Hold the user prompt open until background work ends.* The agent ended the prompt; an ACP v1 prompt is one turn, and a held prompt would block cancel and queueing.
- *ACP v2 `state_update` as the bracket.* Correct in principle (`idle`/`running` are explicit) but claude-agent-acp 0.100 does not wire its foreground reporter for stdio clients. Revisit at the SDK 5.x move ([[acp-agent]] §8).
- *Declare AIR `asyncTasks` now for an explicit bracket.* Deferred: the declaration flips the adapter into AIR-client mode (`dist/tool-calls/client-capabilities.js:43`), and the witness showed `rawInput` arriving later in that mode. It is the next step if quiet-close proves inadequate.

**Consequences.** Brief [[2026-11-03-agent-initiated-turns]]. [[acp-agent]] moves to v0.2. [[log-ir-v1]] gains §4b and `PROTOCOL` 1.2 (lock refreshed by `make lock-ir`, fixture `v1.2-agent-turn.jsonl` frozen). The seats need the term bundle's harness rebuilt before the gap closes in live use; until then the COMMON.md rule holds. An agent-initiated turn is an open turn for the status heartbeat of [[app-harness]] #348.
