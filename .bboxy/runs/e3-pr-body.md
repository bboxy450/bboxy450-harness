Adds the standalone/embedded GUI consumer on toolkit 0.1.1: pure retained Envelope projection, background subscription and preparation, StreamDock transcript, composer/approval/control actions, offline replay and headless Metal rendering.

Rebased onto bfda40c (protocol v0.2 core). The rework models stale Stop delivery through A→B→A, limits StreamToView coverage to DirtyWake, tests the real bridge with an injected 0.3ms clock, checks snapshot repair before completion and activates rendered approval buttons. Cards show readable content; completed terminal blobs load on workers. The smoke checks per-card meaning, controls and scrolling.

Validation: full unfiltered `make test` exits 0 unattended, including 43 native/TLC variants across 19 contracts, unit/integration/process tests, both clippy checks, GUI tests and actual offscreen replay smoke. Live typed terminal/tool heads, live snapshots and completed output are tested against the real gateway. All commits are signed. Both benchmark entries report UNMEASURED under load; no performance verdict claimed.

Report: https://github.com/bboxy450/bboxy450-harness/blob/e3-gui/.bboxy/runs/e3-gui-report.md
Screenshots: `.bboxy/runs/gui-smoke.png` and `.bboxy/runs/gui-smoke-scrolled.png`.

Phase 2 widget adoption awaits its explicit merge signal. Do not merge as Executor.
