---
id: "0C8JGJM2593703E6D1254YDPN0"
number: 18
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

**Gates and measurements are separate instruments.** A wall-clock percentile is never a `make test` gate: on a shared machine it measures the load, not the code. Each allocation therefore has two checks:
- a **structural** check in `make test`, which is deterministic and decides pass or fail;
- a **measurement** in `make bench-budgets`, which reports. It prints `UNMEASURED` (not PASS) when the 1-minute load average exceeds `ncpu`. A measured value over its allocation exits non-zero and goes to the Architect as a budget defect.

## 1. bbt-rt IO workers: 2 threads, shared with all of bboxy450-term when embedded

`bbt-rt/src/lib.rs:51`: there are two workers for the whole process.

| Rule | Allocation |
|---|---|
| A task on `bbt_rt::handle()` never blocks and never runs more than 1 ms of CPU between awaits | hard rule |
| File-system walks (`grep`, `glob`, `read` of a directory), diff computation, markdown preparation of more than 64 KiB, and JSONL fsync | run on `spawn_blocking`, behind one harness-wide semaphore of 4 permits |
| Store append | `write(2)` is on the session actor and fsync is on `spawn_blocking` (gateway §6) |

Check (structural, `make test`): `tests/rt_budget.rs`.
- Every entry point in the `spawn_blocking` row (grep, glob, directory read, diff, markdown preparation over 64 KiB, fsync) calls `assert_off_rt_worker()`. It panics if the current thread is a `bbt-rt` worker.
- The test drives each entry point and asserts the hook ran.
- The test runs 16 concurrent 10k-file greps and asserts that the semaphore's observed maximum of held permits is exactly 4, and that no walk ran without a permit.

Measurement (`make bench-budgets`): the same grep fixture while a 1 ms ticker runs on `bbt_rt::handle()`. Ticker p99 lateness ≤ 5 ms. This moved out of `make test` on 2026-11-01: it flaked at load 44–60 on 10 CPUs (8.38 ms, then 7.51 ms) on unchanged code. Oracle ruling E1b item 15, agreed by the Architect.

## 2. Frame time (embedded, as a LocalPage at 120 Hz)

These rows are taken from the upstream rows, not added to them.

| Stage | Upstream row | Harness share |
|---|---|---|
| Drain of harness subscriptions | Drain 1.5 ms | ≤ 0.3 ms: `try_recv` until empty or until the budget is spent, then defer |
| Harness view projection (layout + paint) | Projection 1.0 ms | ≤ 0.6 ms while the page is visible; 0 when hidden. Layout is reused for unchanged completed items, so only the open tail is rebuilt |
| Markdown parse | — | never on the UI thread for a block over 4 KiB. Prepared on IO (AsyncRenderPreparation shape), and the previous layout is shown until it is ready |

When standalone, the window has its own 4 ms CPU path. Check (structural, `make test`): `tests/gui_budget.rs`.
- It replays a 100k-delta fixture through the headless frame driver with an injected clock in which each `try_recv` and each item projection advances a fixed cost.
- It asserts that drain stops when it has spent its 0.3 ms, defers the rest, and loses nothing by the end.
- It asserts that a completed item's layout is never rebuilt, and that no markdown block over 4 KiB is parsed on the UI thread.

Measurement (`make bench-budgets`): the same replay on the real clock, with drain + projection p99 ≤ 0.9 ms. It moved for the same reason as §1, before it flaked.

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

## Change log

- **2026-11-01, at the Oracle's ruling E1b item 15** (message 01M6APCPE97RFQMKJY2A9RPZZC). Wall-clock p99 checks leave `make test`. §1 and §2 get structural gates, and the p99s move to `make bench-budgets`, which prints UNMEASURED when load exceeds ncpu. §2 was extended by the Architect for the same reason, before it could flake. Alternative rejected: a looser threshold, which would still measure the load and would hide a real regression.
