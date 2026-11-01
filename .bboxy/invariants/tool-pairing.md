---
id: "0JTW8XYW1ZCZ725NH3YJ1AMJP5"
number: 14
type: invariant
title: "every tool call in saved history has exactly one result; cancelled or interrupted calls get a synthetic one"
status: active
check: cargo test --test protocol_laws --test e2e_scripted --test store_recovery && make test-formal MODELS=ToolPairing
cost: "the next request is provider-invalid (an unpaired tool_call_id is a 400) and the session cannot continue"
links:
  - { to: "[[protocol]]", rel: constrains }
  - { to: "[[provider]]", rel: constrains }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 2: pairing

**Property.**
- Every `ToolCall` item that is `ItemStarted` in a turn is `ItemCompleted` before that turn's `TurnEnded`, or is closed by recovery.
- `history::to_wire(log)` produces a `messages[]` in which every assistant `tool_calls[i].id` has exactly one `role:tool` message with that `tool_call_id`, and no orphan tool message exists. This is kittens `pair_remaining`, `src/agent.rs:1504-1516`.

**Check.**
- `check_transcript` checks both clauses on every e2e transcript and every recovered truncation.
- The scripted fixtures cover cancel mid-tool, cancel between two parallel tools, and a tool panic.
- Model ToolPairing must pass. BrokenToolPairing skips the synthetic result on cancel and must be rejected.

Owner: Nyx Copperwing <ARCHITECT>.
