---
title: "E3 GUI face — phase 1 implementation and verification"
task: "[[2026-10-31-harness-gui]]"
status: verification-in-progress
---

Implementation is complete on the published toolkit 0.1.1 and rebased onto main b9b5d8f. The full gate passed at implementation commit 5e08a820152288050c783f384f50820c8bd7c78b. The additional Oracle v0.2 ruling is now implemented and its targeted tests pass; the final full gate is being refreshed. This record is not an acceptance claim.

- `src/frontends/gui/model.rs`: pure ItemId projection, authoritative completion replacement, durable seq deduplication, epoch/at fences, lag resume and Reset rebuild. Additive protocol v0.2 identity heads are available before deltas, and Terminal.live snapshots repair the bounded Unicode tail.
- `src/frontends/gui/bridge.rs`: background Host factory and subscription on bbt-rt, bounded preparation workers, UiSender/Latest/DirtySource delivery, causal intent tracking through detach, and resize preparation.
- `src/frontends/gui/view.rs`, `chat_scroll.rs`, `assets.rs`: StreamDock rows, retained composer/header, immutable completed layouts and fixed-height offscreen elision, scoped tail following, approval controls and toolkit SVG assets.
- `src/frontends/gui/window.rs`, `options.rs`, `src/main.rs`: standalone native host, GUI CLI, replay with no model, keyboard interrupt and explicit Stop; window close detaches and retains admitted work.
- `tests/gui_*`, `tests/formal/models.rs`, `verification/`, `Makefile`: fixture/property/API/frame witnesses, three additional positive/broken model pairs (retaining HarnessStreamToView), isolated GUI gate and offscreen Metal smoke in make test.

Verified so far:

- `make test`: exit 0 at 5e08a82 before the v0.2 addition; private-HOME core/unit/integration/process tests, fmt/clippy, headers/secrets, 33 exact native/Java reachable-state/transition/verdict comparisons including all broken controls, and the GUI gate/smoke. Log: `.bboxy/runs/e3-make-test.log`.
- `cargo test --offline --locked --test gui_fixtures --test gui_bridge`: exit 0 after v0.2; 6 fixture/property cases and 2 live gateway cases. `.bboxy/runs/gui-v0_2-tests.log`; added fixture `tests/fixtures/gui/v0_2-identity-terminal.jsonl`.

- `cargo test --offline --locked --test gui_fixtures --test gui_bridge`: exit 0 before final source changes; every protocol fixture passes check_transcript and projects; 128 random loss/coalescing cases repair at completion.
- `make test-gui`: exit 0; exact toolkit blocking-method lint catalogue, feature tests and Metal smoke in a private HOME, without operator input. Actual log: `.bboxy/runs/gui-tests.log`.
- Headless smoke: 17 visible, nonblank, nonoverlapping items, all 14 kinds, Metal 1120×7000. Screenshot: `.bboxy/runs/gui-smoke.png`.
- Frame driver: 100000 deltas / 200 frames; CPU p99 0.413ms against 0.9ms. Wall-time p99 is retained separately; it includes scheduling delays during concurrent builds and is not claimed to meet 0.9ms.
- `cargo fmt --check`, header check and secret scan: exit 0 before final verification.

Earlier failures and fixes: a non-flat container material, missing SVG icon registration, and rebuilding transcript/composer layouts on every frame. Initial wall-time failures are retained in `.bboxy/runs/gui-wall-failure.log`; the final gate measures the specified CPU allocation and retains wall time explicitly.

Remaining: refresh make test after the v0.2 addition, then signed branch push and PR. Phase 2 rich toolkit widgets await the Oracle's explicit merge signal. Native window presentation and physical IME have not been exercised; the framebuffer, toolkit layout and gateway effects are tested headlessly. Plain previews are capped at 4096 UTF-8 bytes, with the full typed body retained in the transcript/log.

The current main core still exposes protocol v0.1 while E1b implements v0.2. The GUI keeps additive wire fields during replay and reads those same fields from in-process typed envelopes once the new core is linked. It does not fork or modify protocol/core files.
