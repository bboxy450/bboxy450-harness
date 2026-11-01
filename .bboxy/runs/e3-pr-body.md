Adds the standalone `gui` command, offline fixture replay, and a pure ItemId transcript projection with additive protocol v0.2 identity heads and terminal snapshot repair. Gateway work and transcript preparation run on bbt-rt; the UI receives immutable state through Latest/UiSender and retains completed layouts. Prepared snapshot identity is retained through frame replacement to avoid A → B → A cache reuse. The native host uses StreamDock, the toolkit composer, approval controls, scoped scrolling, and explicit interrupt/stop actions.

Verification:

- `make test-gui`: exit 0, including the injected-clock 100k-delta gate and offscreen Metal smoke (17 visible items, all 14 kinds).
- `make test-formal`: exit 0; all 33 native/Java reachable-state, transition, and verdict comparisons match, including broken controls.
- All-target/all-feature clippy: exit 0. Core tests excluding the Oracle-owned ticker case: exit 0.
- The final `make test` run hit the unchanged ticker timing case (9.215ms; previous failures 8.38ms and 7.51ms). Oracle disposition: **rt_budget wall-clock, E1b #1117 item 15**, recorded in budget spec 82f2edd and decision 8de0476. No other failure is waived.
- `make bench-budgets`: UNMEASURED at load 19.01 on 10 CPUs, as required by the updated spec.

Plain previews use the published toolkit 0.1.1; rich widget replacement follows the Oracle's phase 2 merge signal. Native window presentation and physical IME are not exercised by the headless tests.

[Report](https://github.com/bboxy450/bboxy450-harness/blob/e3-gui/.bboxy/runs/e3-gui-report.md) · [Headless screenshot](https://github.com/bboxy450/bboxy450-harness/blob/e3-gui/.bboxy/runs/gui-smoke.png)
