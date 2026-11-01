---
id: "0EEZTE6013TV57EBER08DQZDZ4"
number: 8
type: invariant
title: "cancelling a turn kills its process groups within 2 s and no late result is applied to a newer generation (A → B → A)"
status: active
check: cargo test --test cancel_process --test e2e_scripted && make test-formal MODELS=CancelTree
cost: "orphan processes keep writing the tree after the user said stop, and a late tool result from turn A lands in turn B's history; kittens logged 16 cancel fixes without a model"
links:
  - { to: "[[protocol]]", rel: constrains }
  - { to: "bboxy450-term:[[kill-only-own-pids]]", rel: sibling }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 4: cancel is cancel

**Property.**
- Each turn owns a child `CancellationToken` and a task group. No tool, subagent or provider task is detached; the group is the upstream `bbt-rt` scoped task group.
- On cancel, every tool process group gets SIGTERM and, after 500 ms, SIGKILL. The task group is joined, never dropped. `TurnEnded{Cancelled}` is emitted after the join, within 2 s of the ack.
- A completion is applied only if its `(turn, generation)` equals the current one. Otherwise it is retired and counted. Only process groups the harness spawned are signalled.

**Check.**
- `tests/cancel_process.rs` runs a real `bash` tool `sleep 600 & sleep 600; wait` and cancels it. It asserts `TurnEnded` within 2 s and `kill(-pgid,0)==ESRCH` for every spawned group.
- An A → B → A test resolves a held tool future of turn A after turn B starts and asserts that turn B's history does not contain it.
- Model CancelTree must pass. BrokenCancelTree applies a completion without the generation fence and must be rejected.

Owner: Nyx Copperwing <ARCHITECT>.
