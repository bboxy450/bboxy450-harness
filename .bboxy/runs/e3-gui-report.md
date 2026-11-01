---
title: "E3 GUI face — rework evidence"
task: "[[2026-10-31-harness-gui]]"
status: verification-in-progress
---

PR https://github.com/bboxy450/bboxy450-harness/pull/2, branch `e3-gui`. Phase 1 uses published toolkit 0.1.1. No merge performed. This record supersedes the rejected c5c0f19 handoff; final unfiltered verification and rebase onto E1b remain pending. `make test-formal` exited 0 on signed 861a176: 33 models, 16 pairs plus Yarn, exact native/TLC states and transitions; `e3-rework-formal.log` records the retained receipt. UI-specific lib/bin clippy and normal all-target/all-feature clippy both exited 0.

| Oracle item | Mechanism and evidence |
|---|---|
| 1: stale Stop | `tests/formal/models.rs:708`: admit a Stop for birth A, preserve it through detach/consume return/Host drop and B→A replacement. Broken delivery targets the replacement; `StopSignalsOwnBirth` fails. `:840` requires a reachable third-birth state holding the first-birth Stop. Old regression exit 101 (`e3-rework-red-formal.log`); new regression exit 0 (`e3-rework-green-formal.log`). |
| 2: honest coverage | `verification/contracts.toml`: StreamToView title and sources reverted to DirtyWake only. `coverage.toml` explicitly leaves the applied ledger and snapshot/completion repair to implementation tests. No ledger proof claimed. ScrollStick includes wheel-to-bottom re-follow. |
| 3: real budget path | `tests/gui_budget.rs:123` fills the actual bridge's Latest/UiSender through worker replay and resize publications, then calls `Bridge::drain_with_clock`. At 300µs it leaves work queued; later frames drain it and retain the final revision, both items and all 100000 characters. A 64KiB AgentMessage is prepared off the registered UI thread and completed layouts are retained. Bypassing the budget is caught at 1.75ms vs 300µs (exit 101). Restored test exit 0. |
| 4: readable cards | `view.rs:626` handles every kind as readable text, summarizes args, shows terminal exit/output, literal unified diffs, plan statuses, approval subject, image size, token counts and subagent session. Roles derive from kind. `bridge.rs:441` loads and validates completed terminal blobs in background, before layout; scope changes clear prepared output. No UI IO. `smoke.rs` checks each card's actual solved text, tool names, approval buttons and a virtualized scrolled viewport, plus nonblank/nonoverlapping pixels. Old rendering regression exit 101; new rendering and smoke exit 0. |
| 5: snapshot meaning | `gui_fixtures.rs:263`: 128 random loss/coalescing cases repair midway and remain equal after each subsequent delta, before any completion. It first requires inequality without the snapshot. Ignoring snapshots makes this test exit 101; restored test exit 0. |
| 6: real approval buttons | `gui_bridge.rs:90`: find the rendered Allow/Always/Deny action, call `view.activate`, observe the corresponding completed approval through the gateway. Redirecting dispatch to Interrupt makes the test exit 101; restored test exit 0. |

Red controls and exact commands/exits: `e3-rework-controls.json`, with `e3-rework-red-{budget,snapshot,approval}.log`. `e3-rework-green-gui.log`: all focused GUI tests pass (exit 0), with the live-v0.2 test explicitly filtered until the core merge. `e3-rework-smoke.log`: exit 0, 17 items / 14 kinds / three buttons / scrolled viewport; screenshots [full](gui-smoke.png) and [scrolled](gui-smoke-scrolled.png). Headers and secrets checks exit 0.

The live test `gui_render.rs:236` exercises typed Envelope heads, coalesced live snapshots and real completed blob loading. Before E1b, it exits 101 at the terminal identity assertion (`e3-rework-red-live.log`); this is the expected core-v0.1 red boundary, not a waived final gate. Oracle message 01M6AS0GDBJCVW9YM4TXBNB5RG says to keep working until PR #3 lands, then rebase and retain both sides of formal/inventory changes. Full `make test` must exit 0 after that rebase.

Deliberate limits: finite formal contracts are not a Rust refinement proof; StreamToView proves dirty-wake bookkeeping only. Phase 2 widgets await the Oracle signal. Physical window presentation and IME remain outside this headless witness. The product retains full typed bodies; visible previews cap at 4096 UTF-8 bytes. Earlier wall-clock rt_budget disposition was “rt_budget wall-clock, E1b #1117 item 15”; E1b replaces it before the required final gate.
