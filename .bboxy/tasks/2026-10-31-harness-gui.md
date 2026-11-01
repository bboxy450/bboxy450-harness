---
allowed_paths:
- src/frontends/gui/
- src/frontends/mod.rs
- src/main.rs
- src/lib.rs
- tests/gui_*
- tests/formal/
- tests/fixtures/gui/
- verification/
- Cargo.toml
- Cargo.lock
- Makefile
- .bboxy/runs/
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T04:02:23.938Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6AMBMZY3B9SDXBN6ZEKA8D1
number: 1121
provenance:
  digest: sha256:017dd9defe699069f6a94095439ac3298675daced655cbc4a09f5d0c39ecd987
  frontmatter_digest: sha256:968111ac7be1f0133b078dc547b01d142f97bb00e28f684294b04d813c578c9e
  log: .bboxy/log/2026-11-01.jsonl#4962
  sig: jQGzn5FMzmFNR6jfNof7ZZNipLniKyZYh3C21Y3l+TqTWpDjy8eg1TLX+Qp/YXtsn0Z5jmAhIQDHjMulsszXAA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3: bboxy450-harness GUI face — view-model projection, fixture replay, standalone window, headless frame smoke'
type: task
updated: 2026-11-01T04:02:23.938Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
# E3: the GUI face

ROLE: EXECUTOR (Codex).

**Worktree:** `/Users/feral/mydev/bboxy450-harness-gui`, branch `e3-gui`, based on `origin/main` `71a4d95`. Push the branch and open a PR. Do not merge.

E1b is hardening core files in parallel. Stay inside `allowed_paths`, and rebase onto `main` before you report.

## Read first
- `.bboxy/specs/harness-zoomed-out.md`, especially the Items and Formal-first sections
- `.bboxy/specs/toolkit-lowering.md`, the GUI and Async sections
- `.bboxy/interfaces/protocol.md`, especially §3–§5. The GUI is purely a consumer of Envelopes.
- `.bboxy/interfaces/gateway.md`
- `.bboxy/interfaces/interfaces.md`, the rendering rows: StreamToView, DirtyWakeFrame, ScrollStick and EmbedLifecycle
- `.bboxy/specs/budgets.md`, for the frame budget and preparation workers
- `.bboxy/research/bboxy450-term-toolkit.md` §2–3, with pointers to `bbt-ui`, `bbt-rt`, StreamDock, `chat_scroll` and the bboxybook window pattern

## Outcome

### Phase 1 (now, on the published toolkit 0.1.1)
1. **`src/frontends/gui/model.rs`.** A pure view-model projection: Envelope stream → transcript state keyed by ItemId.
   - It handles ItemStarted, ItemDelta, ItemSnapshot, ItemCompleted (where the full body wins), TurnQueued/Started/Ended, Status, Lagged (re-subscribe after `resume_after`) and Reset (rebuild).
   - It has no UI dependencies and is fully unit-testable.
   - Run every fixture in `tests/fixtures/protocol/*.jsonl` through it.
   - Property test: a projection fed with deltas dropped or coalesced, at random, equals the projection fed the full stream once completion arrives. Deltas are applied in order, exactly once, to the current item generation.
2. **Gateway subscription on `bbt_rt::handle()`.** Results reach the UI through `UiSender`/`Latest`, with a `DirtySource` per frame. The UI thread never awaits and never runs `std::fs` or process work. Follow `bbt-rt`'s clippy `disallowed_methods`.
3. **Standalone window (feature `standalone`)** following the `bboxybook/src/window.rs` pattern.
   - Transcript built on StreamDock and `chat_scroll` (stick-to-bottom), plus a composer that sends `Op::UserTurn`.
   - Esc or Ctrl-C sends `Interrupt` on the control lane.
   - Approval items get Allow / Always / Deny buttons that send `ResolveApproval`.
   - Render every item kind with the existing widgets; plain text is acceptable where a rich widget is still missing.
   - `bboxy450-harness gui [--profile P] [--cwd D]`
   - `bboxy450-harness gui --replay <fixture.jsonl>` runs offline, with no model.
4. **Formal models (Harness\*), each with a broken control,** using the existing `tests/formal` harness and inventory:
   - HarnessStreamToView: reuse or adapt DirtyWake
   - HarnessDirtyWakeFrame
   - HarnessScrollStick
   - HarnessEmbedLifecycle: detach preserves the run; only an explicit Stop ends it
5. **Headless frame smoke.** `make test-gui-smoke`, which `make test` runs.
   - It renders `--replay all-items.jsonl` offscreen (bboxybook-style headless).
   - It asserts that every item kind produced a visible element and that nothing panicked.
   - It runs with no display interaction and no operator involvement.

### Phase 2 (after the Oracle tells you E2's toolkit widgets have merged)
Replace the placeholders with:
- the streaming markdown and syntax-highlighting transcript
- the diff view, for FileChange and TurnDiff
- tool-call cards
- the approval atom
- the editor upgrades

Pin them through the git-rev `[patch]` described in architect-review-v0 §6.

## Verify
`make test`, unattended, exit 0. It includes fmt, clippy, the new GUI tests, the formal models and `test-gui-smoke`.

## Report
Send one `task_report` with:
- files and mechanism
- commands and exit codes
- evidence for each outcome
- a screenshot path under `.bboxy/runs/`, from the headless smoke
- the branch and its signed head
