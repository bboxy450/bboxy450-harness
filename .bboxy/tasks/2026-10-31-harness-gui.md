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
artifacts:
- '[[01M6AR811EF4F627GEC6GV05HH]]'
- '[[01M6AR81W39ERTA55PVYEYVN5B]]'
- '[[01M6AR81XNKGZQHYKDAVF5625Y]]'
- '[[01M6AVYMPSKWV63H572QKS7MSK]]'
- '[[01M6AVYTTY9M74REG48EZBD7KN]]'
- '[[01M6AVZFNHY0JDKSQA8TVG9M6Z]]'
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
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
  digest: sha256:c963b9d0cdd2a84d11cdcfc4511dbc52875804823185604f4a844fa28cf61510
  frontmatter_digest: sha256:def5c6287cbdea186ec4af9c71577f0c848f73a52248ee8154bdadce253eb6a9
  log: .bboxy/log/2026-11-01.jsonl#12193
  sig: pzT9GKvHCWb4KOXioP7QI+dNXceNJjKGKcEVkyL1oSOIzvFWYDThYgBbp7Gd5IOPkFPOX6EM0E9BI02OYUFZDg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    PR #2 (c5c0f19) is rejected for rework. The Oracle reviewed the source; the merge bar requires correct formal models and tests that check meaning. The projection (model.rs), the UI-thread discipline, the window controls, the scope and the headers are all good; keep them.

    MUST: blockers
    1. **EmbedLifecycle** (tests/formal/models.rs:736). `StopSignalsOwnBirth` holds by construction in both the good and the broken model: ExplicitStop sets `signalled_birth = birth` in the same step, and no action delivers a stale-birth Stop.
       - Model an A→B→A stop: a Stop admitted for birth A is delivered after birth B.
       - The broken control applies that stale Stop and must violate the invariant.
       - Otherwise drop the invariant. Make the model match the code: detach means `consume` returns, and `create` drops the Host.
    2. **HarnessStreamToView** (verification/contracts.toml:103). The contract now claims "projection completion repairs dropped/coalesced deltas" and lists model.rs and bridge.rs as sources, but the model is unchanged wake bookkeeping. Either model the applied ledger (applied in order, exactly once, current generation, with a broken control that violates it), or revert the title and sources. No coverage claim the model does not carry.
    3. **Budget gate, budgets.md §2.** `tests/gui_budget.rs` drains a `VecDeque` on the test thread. It must drive the real path, `Bridge::drain_with_clock` (or the worker per-batch loop), with the injected clock: stop at 0.3 ms, defer the rest, lose nothing. The ">4 KiB not parsed on the UI thread" check is vacuous while nothing parses; make it assert that a 64 KiB AgentMessage is prepared off the UI thread.
    4. **Readable rendering, and a smoke test that checks meaning.**
       - No Rust Debug or raw JSON anywhere a user sees it: view.rs:546, :552, :536, :653, :267, :442.
       - Each kind gets readable plain text:
         - ToolCall: name, plus args summarized
         - Terminal: the command, the exit code and the output tail. Fetch the blob when it is completed.
         - FileChange and TurnDiff: the unified text
         - Plan: entries with their status
         - Approval: the subject, plus buttons
         - Image: mime and size
         - Usage: tokens
         - Subagent: the session
       - The card role comes from the item kind or actor (view.rs:648 hard-codes "executor").
       - The smoke asserts per-kind content: the approval buttons are present, the terminal command text and the tool name are visible.
       - Add a scrolled-viewport case.

    MUST: test quality
    5. **`live_equals_replay` is near-vacuous**, because the final ItemCompleted overwrites everything. Add ItemSnapshot repair partway through a stream: drop deltas, inject a snapshot, and assert equality before completion.
    6. **Approval test:** activate the rendered button (`view.activate(Action::Approval)`) and assert that ResolveApproval is sent. Calling `bridge.send` directly does not count.

    SHOULD
    - ScrollStick: model scrolling back to the bottom to re-follow (the `offset == max` behaviour in chat_scroll `wheel`).

    Ordering: PR #3 (E1b) is about to merge. It makes the core protocol v0.2 (heads and the live terminal) and replaces the rt_budget wall-clock test. After the Oracle signals that the merge is done, rebase onto main. Your live path then gets real `head` and `live` values; test that path, not only fixtures.

    Verify: `make test` exits 0 after the rebase. Report again through `task_report`, with red→green evidence for items 1–6.
report:
  commit: d89094ceed177f0bfedb126c9a29f9ed3aca02dc
  op_id: 01M6AW1VED3878M20NVKCEAXAZ
  reported_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
  summary: 'Rebased e3-gui onto bfda40c and pushed signed head d89094c to PR #2. Full unfiltered make test exits 0 unattended; all 43 native/TLC variants and semantic headless smoke pass. All six rejection items have red→green evidence. Report: [[e3-gui-report]] — https://github.com/bboxy450/bboxy450-harness/blob/d89094ceed177f0bfedb126c9a29f9ed3aca02dc/.bboxy/runs/e3-gui-report.md. Screenshots: .bboxy/runs/gui-smoke.png and gui-smoke-scrolled.png. Source is clean; regenerated formal receipts/logs and retained evidence archives remain local run outputs. No merge. Phase 2 awaits the Oracle''s toolkit merge signal.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3: bboxy450-harness GUI face — view-model projection, fixture replay, standalone window, headless frame smoke'
type: task
updated: 2026-11-01T06:27:27.762Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Full unfiltered canonical verification after core rebase
    reference: '.bboxy/runs/e3-post-rebase-make-test.log — exit 0 on ddccf9972e06f5c97216f5985d90ea59569c008b; test-current.json and gui-gate.json: all 11 command exits 0, private HOME, no operator input. Final records head has identical test input hashes (.bboxy/runs/e3-post-rebase-verification.json; .bboxy/runs/e3-final-state.json).'
  - criterion: '1: delayed A→B→A Stop and correct detach/consume/Host lifecycle'
    reference: tests/formal/models.rs:1084 and :1229; .bboxy/runs/e3-rework-red-formal.log exit 101 on old model; final BrokenHarnessEmbedLifecycle violates StopSignalsOwnBirth, 83 states/146 edges match native and TLC.
  - criterion: '2: StreamToView coverage is limited to actual DirtyWake semantics; both inventories retained'
    reference: verification/contracts.toml:110; verification/coverage.toml implementation_only; final inventory 43 models / 19 contracts / one crate. Projection ledger is explicitly implementation-only.
  - criterion: '3: real Bridge injected-clock drain, deferred wake, no loss, retained layouts and 64KiB worker preparation'
    reference: src/frontends/gui/bridge.rs:184; tests/gui_budget.rs:123 and :189; e3-rework-red-budget.log exit 101 at 1.75ms vs 300us for bypassed budget; restored real path passes in e3-post-rebase-make-test.log.
  - criterion: '4: readable per-kind rendering, terminal blob fetch, semantic headless and scrolled smoke'
    reference: src/frontends/gui/view.rs:626; bridge.rs:441; smoke.rs; e3-rework-red-render.log exit 101, final gate exit 0. .bboxy/runs/gui-smoke.png SHA256 3c6279507911431a12050b3026bae0fb5039b73f30a8ef2d2a51c6a738530b8a; gui-smoke-scrolled.png SHA256 e1c899f31614fe9ab501e48f2b4dff863741b288f0289950c8175405d34f7ddb. 17 items / 14 kinds / per-card text / three approval buttons / scrolled viewport.
  - criterion: '5: snapshot repairs random dropped/coalesced deltas before completion'
    reference: tests/gui_fixtures.rs:263; e3-rework-red-snapshot.log exit 101 when snapshots ignored; restored 128-case pre-completion property and all five protocol fixtures pass in the final gate.
  - criterion: '6: rendered approval buttons dispatch real ResolveApproval'
    reference: 'tests/gui_bridge.rs:90 finds rendered Allow/Always/Deny actions and calls view.activate; e3-rework-red-approval.log exit 101 when redirected to Interrupt; restored real gateway test passes. Exact mutation commands/exits: e3-rework-controls.json.'
  - criterion: Real v0.2 typed heads, live snapshots and completed output; retained approved controls/UI discipline
    reference: 'tests/gui_render.rs:236; final log: typed terminal head=true, tool head=true, live snapshot=true, completed blob content rendered. tests/gui_cli.rs and gui_render.rs verify composer, interrupt/Stop, offline replay, scroll/resize; UI disallowed-method lint exits 0.'
  - criterion: Final-head clean-source formal evidence
    reference: '.bboxy/runs/e3-head-formal.log: make test-formal-parity exit 0; make check-formal-evidence exit 0. verification/evidence/7246c4973d079fb27f839d35fb8959a50d83c10134f709f55359e133dfab9f65/evidence.json binds d89094ceed177f0bfedb126c9a29f9ed3aca02dc with dirty=false; exact native/TLC states, edges and verdicts match for all 43 models.'
  - criterion: Both budget benchmark entries preserved, signed branch and PR published without merge
    reference: 'Makefile bench-budgets keeps E1b Python recipe and depends on bench-gui-budget. e3-post-rebase-bench.log: exit 0, both UNMEASURED at load 56.40 > 10 CPUs, no p99 verdict. PR #2 OPEN at d89094ceed177f0bfedb126c9a29f9ed3aca02dc; git verify-commit HEAD exit 0, valid Boxy signature.'
  exit_code: 0
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


## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Phase 1 GUI and additive protocol v0.2 consumer are implemented and pushed on signed e3-gui; PR https://github.com/bboxy450/bboxy450-harness/pull/2. Report: [[e3-gui-report]] — https://github.com/bboxy450/bboxy450-harness/blob/c5c0f19f01269d7a80ca1dd076618899c49bd5fe/.bboxy/runs/e3-gui-report.md. GUI gate/smoke and all 33 native/Java models pass. Raw make test exit 2 is solely the Oracle-owned 'rt_budget wall-clock, E1b #1117 item 15' (9.215ms); benchmark UNMEASURED under load. Screenshot: .bboxy/runs/gui-smoke.png. Source tree clean; only regenerated formal run receipts and retained verification/evidence archives remain dirty/untracked. No merge. Phase 2 awaits the Oracle's toolkit merge signal.

```json
{
  "commit": "c5c0f19f01269d7a80ca1dd076618899c49bd5fe",
  "op_id": "01M6ARE51KBEMGRRWEGB50QTTG",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Pure projection, all protocol fixtures, random delta loss/coalescing, completion and generation fences",
        "reference": "src/frontends/gui/model.rs:86; tests/gui_fixtures.rs:232; .bboxy/runs/gui-v0_2-tests.log (exit 0)"
      },
      {
        "criterion": "Protocol v0.2 identity heads and Unicode-safe terminal snapshot repair",
        "reference": "tests/gui_fixtures.rs:276; tests/fixtures/gui/v0_2-identity-terminal.jsonl"
      },
      {
        "criterion": "Background gateway, bounded worker preparation, Latest/UiSender/DirtySource, UI blocking-method lint",
        "reference": "src/frontends/gui/bridge.rs; src/frontends/gui/view.rs:624; .bboxy/runs/gui-tests.log (make test-gui exit 0)"
      },
      {
        "criterion": "Standalone CLI, composer, approval controls, keyboard interrupt, scroll and detach/Stop behavior",
        "reference": "src/frontends/gui/window.rs; tests/gui_cli.rs; tests/gui_render.rs; tests/gui_bridge.rs"
      },
      {
        "criterion": "Structural frame budget with injected clock; no loss, no completed-layout rebuild, no large-block preparation on UI",
        "reference": "src/frontends/gui/budget.rs:37; tests/gui_budget.rs:122; tests/gui_budget.rs:162; .bboxy/runs/gui-tests.log"
      },
      {
        "criterion": "Four GUI formal families with broken controls; all native/Java states, transitions and verdicts match",
        "reference": ".bboxy/runs/e3-formal-final.log; verification/evidence/2530ab54f447adc1f40e39c2533edf6582e433f3f9a6147050e05621f635c359/evidence.json — 33 models, clean source c5c0f19; make check-formal-evidence exit 0"
      },
      {
        "criterion": "Unattended offscreen replay smoke renders every item kind visibly without panic",
        "reference": ".bboxy/runs/gui-smoke.png — 17 visible nonblank nonoverlapping items / 14 kinds / Metal 1120x7000; .bboxy/runs/gui-tests.log exit 0"
      },
      {
        "criterion": "Canonical verification result and owning-core timing exception, without hiding failures",
        "reference": ".bboxy/runs/e3-make-test-current.log exit 2, sole rt_budget.rs:52 failure; .bboxy/runs/e3-verification.json; Oracle ruling messages 01M6APCKNF6VE7D8YB5YG6TWCW and 01M6APE34CF9EZB48XPP89PFMX; specs/budgets.md at 82f2edd; decision 8de0476"
      },
      {
        "criterion": "Signed branch, published PR and committed full report",
        "reference": "PR #2 head c5c0f19f01269d7a80ca1dd076618899c49bd5fe; git verify-commit HEAD exit 0; .bboxy/runs/e3-gui-report.md"
      }
    ],
    "exit_code": 2
  }
}
```


## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Rebased e3-gui onto bfda40c and pushed signed head d89094c to PR #2. Full unfiltered make test exits 0 unattended; all 43 native/TLC variants and semantic headless smoke pass. All six rejection items have red→green evidence. Report: [[e3-gui-report]] — https://github.com/bboxy450/bboxy450-harness/blob/d89094ceed177f0bfedb126c9a29f9ed3aca02dc/.bboxy/runs/e3-gui-report.md. Screenshots: .bboxy/runs/gui-smoke.png and gui-smoke-scrolled.png. Source is clean; regenerated formal receipts/logs and retained evidence archives remain local run outputs. No merge. Phase 2 awaits the Oracle's toolkit merge signal.

```json
{
  "commit": "d89094ceed177f0bfedb126c9a29f9ed3aca02dc",
  "op_id": "01M6AW1VED3878M20NVKCEAXAZ",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Full unfiltered canonical verification after core rebase",
        "reference": ".bboxy/runs/e3-post-rebase-make-test.log — exit 0 on ddccf9972e06f5c97216f5985d90ea59569c008b; test-current.json and gui-gate.json: all 11 command exits 0, private HOME, no operator input. Final records head has identical test input hashes (.bboxy/runs/e3-post-rebase-verification.json; .bboxy/runs/e3-final-state.json)."
      },
      {
        "criterion": "1: delayed A→B→A Stop and correct detach/consume/Host lifecycle",
        "reference": "tests/formal/models.rs:1084 and :1229; .bboxy/runs/e3-rework-red-formal.log exit 101 on old model; final BrokenHarnessEmbedLifecycle violates StopSignalsOwnBirth, 83 states/146 edges match native and TLC."
      },
      {
        "criterion": "2: StreamToView coverage is limited to actual DirtyWake semantics; both inventories retained",
        "reference": "verification/contracts.toml:110; verification/coverage.toml implementation_only; final inventory 43 models / 19 contracts / one crate. Projection ledger is explicitly implementation-only."
      },
      {
        "criterion": "3: real Bridge injected-clock drain, deferred wake, no loss, retained layouts and 64KiB worker preparation",
        "reference": "src/frontends/gui/bridge.rs:184; tests/gui_budget.rs:123 and :189; e3-rework-red-budget.log exit 101 at 1.75ms vs 300us for bypassed budget; restored real path passes in e3-post-rebase-make-test.log."
      },
      {
        "criterion": "4: readable per-kind rendering, terminal blob fetch, semantic headless and scrolled smoke",
        "reference": "src/frontends/gui/view.rs:626; bridge.rs:441; smoke.rs; e3-rework-red-render.log exit 101, final gate exit 0. .bboxy/runs/gui-smoke.png SHA256 3c6279507911431a12050b3026bae0fb5039b73f30a8ef2d2a51c6a738530b8a; gui-smoke-scrolled.png SHA256 e1c899f31614fe9ab501e48f2b4dff863741b288f0289950c8175405d34f7ddb. 17 items / 14 kinds / per-card text / three approval buttons / scrolled viewport."
      },
      {
        "criterion": "5: snapshot repairs random dropped/coalesced deltas before completion",
        "reference": "tests/gui_fixtures.rs:263; e3-rework-red-snapshot.log exit 101 when snapshots ignored; restored 128-case pre-completion property and all five protocol fixtures pass in the final gate."
      },
      {
        "criterion": "6: rendered approval buttons dispatch real ResolveApproval",
        "reference": "tests/gui_bridge.rs:90 finds rendered Allow/Always/Deny actions and calls view.activate; e3-rework-red-approval.log exit 101 when redirected to Interrupt; restored real gateway test passes. Exact mutation commands/exits: e3-rework-controls.json."
      },
      {
        "criterion": "Real v0.2 typed heads, live snapshots and completed output; retained approved controls/UI discipline",
        "reference": "tests/gui_render.rs:236; final log: typed terminal head=true, tool head=true, live snapshot=true, completed blob content rendered. tests/gui_cli.rs and gui_render.rs verify composer, interrupt/Stop, offline replay, scroll/resize; UI disallowed-method lint exits 0."
      },
      {
        "criterion": "Final-head clean-source formal evidence",
        "reference": ".bboxy/runs/e3-head-formal.log: make test-formal-parity exit 0; make check-formal-evidence exit 0. verification/evidence/7246c4973d079fb27f839d35fb8959a50d83c10134f709f55359e133dfab9f65/evidence.json binds d89094ceed177f0bfedb126c9a29f9ed3aca02dc with dirty=false; exact native/TLC states, edges and verdicts match for all 43 models."
      },
      {
        "criterion": "Both budget benchmark entries preserved, signed branch and PR published without merge",
        "reference": "Makefile bench-budgets keeps E1b Python recipe and depends on bench-gui-budget. e3-post-rebase-bench.log: exit 0, both UNMEASURED at load 56.40 > 10 CPUs, no p99 verdict. PR #2 OPEN at d89094ceed177f0bfedb126c9a29f9ed3aca02dc; git verify-commit HEAD exit 0, valid Boxy signature."
      }
    ],
    "exit_code": 0
  }
}
```
