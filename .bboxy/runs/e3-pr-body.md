Adds the standalone and embedded GUI consumer on toolkit 0.1.1: retained Envelope projection, background gateway/preparation workers, StreamDock transcript, composer and approval/control actions, offline replay, and headless Metal smoke.

The Oracle rework now models delayed stale Stop delivery through A→B→A, limits StreamToView coverage to DirtyWake, tests the real bridge drain with an injected 0.3ms clock, repairs dropped deltas with a snapshot before completion, and activates rendered approval buttons against the live gateway. Cards show readable typed content and completed terminal blobs load on workers. The smoke checks per-card content, approval controls and a scrolled viewport.

Validation on the rework: focused GUI regressions and semantic headless smoke exit 0; 33 native/TLC models and broken controls match; both clippy configurations exit 0. Mutation controls for ignored budget, ignored snapshot and redirected approval each exit 101 as intended. `make bench-budgets` reports UNMEASURED under load.

Pending final integration: Oracle's PR #3 merge signal, rebase onto main, then unfiltered `make test` exit 0. The live v0.2 test is currently red at the old core's missing terminal head; no final gate is waived. Phase 2 widget adoption awaits its explicit merge signal.

Report: [.bboxy/runs/e3-gui-report.md](.bboxy/runs/e3-gui-report.md)
Screenshots: [.bboxy/runs/gui-smoke.png](.bboxy/runs/gui-smoke.png), [.bboxy/runs/gui-smoke-scrolled.png](.bboxy/runs/gui-smoke-scrolled.png)
