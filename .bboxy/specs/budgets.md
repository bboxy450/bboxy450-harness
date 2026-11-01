---
type: spec
title: "budgets: shared resources the harness spends, allocated with margins"
status: draft
links:
  - { to: "bboxy450-term:[[budgets]]", rel: builds-on }
  - { to: "[[interfaces]]", rel: constrains }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Budgets

The numbers in this file are allocations: they are decisions. Measured numbers confirm an allocation or refute it. A cut that spends another subsystem's share is decided here before it lands.

## 1. bbt-rt IO workers: 2 threads, shared with all of bboxy450-term when embedded

`bbt-rt/src/lib.rs:51`: there are two workers for the whole process.

| Rule | Allocation |
|---|---|
| A task on `bbt_rt::handle()` never blocks and never runs more than 1 ms of CPU between awaits | hard rule |
| File-system walks (`grep`, `glob`, `read` of a directory), diff computation, markdown preparation of more than 64 KiB, and JSONL fsync | run on `spawn_blocking`, behind one harness-wide semaphore of 4 permits |
| Store append | `write(2)` is on the session actor and fsync is on `spawn_blocking` (gateway §6) |

Check: `tests/rt_budget.rs` runs a 10k-file grep fixture while a 1 ms ticker runs on `bbt_rt::handle()`. The ticker's p99 lateness must be ≤ 5 ms.

## 2. Frame time (embedded, as a LocalPage at 120 Hz)

These rows are taken from the upstream rows, not added to them.

| Stage | Upstream row | Harness share |
|---|---|---|
| Drain of harness subscriptions | Drain 1.5 ms | ≤ 0.3 ms: `try_recv` until empty or until the budget is spent, then defer |
| Harness view projection (layout + paint) | Projection 1.0 ms | ≤ 0.6 ms while the page is visible; 0 when hidden. Layout is reused for unchanged completed items, so only the open tail is rebuilt |
| Markdown parse | — | never on the UI thread for a block over 4 KiB. Prepared on IO (AsyncRenderPreparation shape), and the previous layout is shown until it is ready |

When standalone, the window has its own 4 ms CPU path. Check: `tests/gui_budget.rs` replays a 100k-delta fixture through the headless frame driver and asserts the harness drain + projection p99 ≤ 0.9 ms.

## 3. Memory and disk per session

| Resource | Allocation |
|---|---|
| Subscriber buffer | 4096 envelopes × ≤ 256 KiB worst case; the typical envelope is < 1 KiB |
| Work inbox / op-id memory | 64 / 1024 |
| Tool output inline | 50 KiB, then spill to a blob |
| Event log | durable events only, no deltas. Target ≤ 2 MB per 100 turns (kittens: 3.9 MB, 96 % of it deltas) |
| Parallel read-only tools | 4 |

## 4. Inference: the free route

The live smoke spends at most 3 requests per run (kittens incident: free-route quota exhaustion at 15 requests).

Owner: Nyx Copperwing <ARCHITECT>.
