---
type: invariant
title: "every accepted turn ends exactly once; every started item reaches a terminal state, including after cancel and crash recovery"
status: active
check: cargo test --test protocol_laws --test e2e_scripted && make test-formal MODELS=AgentTurnLifecycle
cost: "a face waits forever on a turn that never ends, or renders two endings; kittens shipped this double-emission bug (src/agent.rs:221-224)"
links:
  - { to: "[[protocol]]", rel: constrains }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 1: one ending

**Property.** For every `TurnQueued{turn}` in a session log there is exactly one `TurnEnded{turn}`, after it. For every `ItemStarted{item}` there is exactly one `ItemCompleted{item}`, after it. Both hold for a log that was recovered after a process was killed at any line boundary (protocol §3, §4).

**Check.**
- `check_transcript` (`src/protocol/laws.rs`) runs over every transcript that `tests/e2e_scripted.rs` produces.
- `tests/store_recovery.rs` truncates a fixture log at every line and recovers it.
- Model AgentTurnLifecycle must pass, and BrokenAgentTurnLifecycle, which emits a second `TurnEnded` when cancel races `Finish`, must be rejected.
- Exit 0 means the law held.

Owner: Nyx Copperwing <ARCHITECT>.
