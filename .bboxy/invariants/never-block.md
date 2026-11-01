---
id: "01BPEQYJJMDWB644MESM3S4T1J"
number: 10
type: invariant
title: "send never awaits; the engine never waits on a face; control ops are never refused Busy"
status: active
check: cargo test --test gateway_ops --test gateway_lag --test rt_budget && make test-formal MODELS=SteerQueue,SubscriberReplay,DirtyWakeFrame && sh scripts/ui-lint.sh
cost: "one stalled window freezes every turn in the session, and an interrupt that waits behind a full inbox is no interrupt (operator: NEVER BLOCK SWARM)"
links:
  - { to: "[[gateway]]", rel: constrains }
  - { to: "[[budgets]]", rel: constrains }
  - { to: "bboxy450-term:[[ui-never-waits]]", rel: sibling }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 3: never block

**Property.**
1. `Gateway::send` and `Gateway::subscribe` are non-`async` and return in bounded time. A full work lane returns `Busy`.
2. `Interrupt`, `ResolveApproval` and `Close` are admitted whatever the state of the work lane.
3. The engine publishes with `try_push` only. A subscriber that does not read gets `Lagged` and is closed, while the engine and the other subscribers finish the turn.
4. No harness task holds a `bbt-rt` worker for more than 1 ms ([[budgets]] §1).
5. The GUI face does no IO on the UI thread. It runs under bboxy450-term's `clippy.toml` `disallowed_methods`.
6. An approval with no face attached that can answer it never waits: it resolves `by: Unattended` under the profile's `on_unattended` policy (default `Deny`) ([[architect-review-v0]] §7).

**Check.**
- `gateway_ops::busy_not_wait`, `gateway_ops::interrupt_when_full` and `gateway_lag::firehose_dead_reader` must pass.
- `rt_budget` must pass.
- Models SteerQueue, SubscriberReplay and DirtyWakeFrame must pass, and their broken controls must be rejected.
- `ui-lint` runs over `src/frontends/gui` once that directory exists.

Owner: Nyx Copperwing <ARCHITECT>.
