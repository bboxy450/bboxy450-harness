---
title: "E3 GUI face — phase 1 implementation and verification"
task: "[[2026-10-31-harness-gui]]"
status: verification-in-progress
---

Implementation is complete on the published toolkit 0.1.1. Full repository verification follows rebasing onto current main; this record is not an acceptance claim.

- `src/frontends/gui/model.rs`: pure ItemId projection, authoritative completion replacement, durable seq deduplication, epoch/at fences, lag resume and Reset rebuild.
- `src/frontends/gui/bridge.rs`: background Host factory and subscription on bbt-rt, bounded preparation workers, UiSender/Latest/DirtySource delivery, causal intent tracking through detach, and resize preparation.
- `src/frontends/gui/view.rs`, `chat_scroll.rs`, `assets.rs`: StreamDock rows, retained composer/header, immutable completed layouts and fixed-height offscreen elision, scoped tail following, approval controls and toolkit SVG assets.
- `src/frontends/gui/window.rs`, `options.rs`, `src/main.rs`: standalone native host, GUI CLI, replay with no model, keyboard interrupt and explicit Stop; window close detaches and retains admitted work.
- `tests/gui_*`, `tests/formal/models.rs`, `verification/`, `Makefile`: fixture/property/API/frame witnesses, three additional positive/broken model pairs (retaining HarnessStreamToView), isolated GUI gate and offscreen Metal smoke in make test.

Verified so far:

- `cargo test --offline --locked --test gui_fixtures --test gui_bridge`: exit 0 before final source changes; every protocol fixture passes check_transcript and projects; 128 random loss/coalescing cases repair at completion.
- `make test-gui`: exit 0; exact toolkit blocking-method lint catalogue, feature tests and Metal smoke in a private HOME, without operator input. Actual log: `.bboxy/runs/gui-tests.log`.
- Headless smoke: 17 visible, nonblank, nonoverlapping items, all 14 kinds, Metal 1120×7000. Screenshot: `.bboxy/runs/gui-smoke.png`.
- Frame driver: 100000 deltas / 200 frames; CPU p99 0.413ms against 0.9ms. Wall-time p99 is retained separately; it includes scheduling delays during concurrent builds and is not claimed to meet 0.9ms.
- `cargo fmt --check`, header check and secret scan: exit 0 before final verification.

Earlier failures and fixes: a non-flat container material, missing SVG icon registration, and rebuilding transcript/composer layouts on every frame. Initial wall-time failures are retained in `.bboxy/runs/gui-wall-failure.log`; the final gate measures the specified CPU allocation and retains wall time explicitly.

Remaining: rebase, full make test including pinned native/Java parity, signed branch push and PR. Phase 2 rich toolkit widgets await the Oracle's explicit merge signal. Native window presentation and physical IME have not been exercised; the framebuffer, toolkit layout and gateway effects are tested headlessly. Plain previews are capped at 4096 UTF-8 bytes, with the full typed body retained in the transcript/log.
