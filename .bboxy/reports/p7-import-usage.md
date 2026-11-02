---
id: 01M6F69S3ZQH59VFPV8JWXC977
number: 7007
type: report
title: p7-import-usage — native usage and bounded blob previews
status: complete
created: 2026-11-02T22:32:54.655375+00:00
implementation: cb769fb4fc6fe0575bfdf5c3bfb3cd6f39268861
updated: 2026-11-02T23:40:20.333287+00:00
---

COMPLETE — Oracle review reworks for p7-import-usage. Worktree `/Users/feral/mydev/bboxy450-harness-p7-import-usage`, branch `parity/p7-import-usage`. Implementation `cb769fb4fc6fe0575bfdf5c3bfb3cd6f39268861` is a signed squash of only p7's work, made on its own merge-base 26ebcf178a571acaf5a80d682d0d6fd316df5f2b and then rebased onto origin/main `ecfd1e67b4ad8a128b52893ee2b60cb1327e13a2` after p6 landed. No push or merge.

Both requested reworks are complete:

1. All former raw-artifact commits were removed from the p7 branch history by the squash. The p7 diff under `.bboxy/runs/` contains exactly one small summary: `p7-import-usage.json` (commands, exits, hashes, counts). There is no bulk JSON, log, evidence dump, screenshot, .tla/.edges copy, or bulk-check.py in the task commits. The disposable bulk driver and all raw output live outside Git under `/tmp/p7-review-evidence-jul7x57c`. Regenerated `.bboxy/runs/formal-native.json` is copied there and restored to the upstream baseline before handoff. The bulk checker is a one-off verification driver, not a shipped tool.
2. P6 landed squashed as `4c9f37d4788eed04546238f6caeb8278b1ebc36c`, subject “Report seat readiness and preserve typed ACP session usage”; `39683ae` locked its 42 definitions. The requested “Write ACP session usage” subject/hash was rewritten during p6's squash. Main's actual shared Usage/Backend/Cost definitions and ACP writer were inspected before rebasing. P7 defines no protocol types and changes neither `src/protocol/mod.rs` nor `protocol.schema.json`. P7's only adaptation is constructing p6's `Option<Box<Backend>>`; the accepted import logic is retained.

Usage mechanism: `src/import/mod.rs:162` (turn aggregation), `:184` (Claude), `:238` (Codex), `:603` (partial counter parsing), `:642` (derived sums) counts Claude message/request aliases once and deduplicates Codex cumulative snapshots without collapsing distinct equal-sized calls. Imported call records have scope `call`, `backend{kind: import, name: claude-code|codex, version if reported}`, exact known model/request identity and source `reported`. Raw native cumulative snapshots stay session scope and reported; cumulative-only data never fabricates a call. `TurnEnded.usage` totals calls with scope `turn`, source `derived`. Source is one scalar per record; there is no per-field source map. Missing required counts are zero placeholders named in `unreported`; reported zero remains distinct. Unknown optional counters/costs stay absent. The importer does not fabricate a missing counter or drop a partial record.

Spill mechanism: `src/import/spill.rs:42` persists oversized text before appending its bounded preview, sets typed tool spill references, preserves UTF-8 boundaries, considers JSON escaping and redacts before truncation. `src/import/watch.rs` uses this same writer for live records and close-time completions. Failed blob writes publish no dangling reference.

Changed files: `src/import/{mod,spill,watch}.rs`; `tests/import_native.rs`; four synthetic native usage/spill fixtures (`tests/import_native.rs:115`, `:192`, `:260`, `:336`, `:451`); `tests/formal/models.rs`, `tests/formal_nonvacuity.rs`; `verification/{README.md,contracts.toml,corpus.toml,coverage.toml}`; `ATTRIBUTION.md`; this report and the single small summary. Source/type/schema/provider/GUI/context changes now belong entirely to p6/main, not p7. Rebase conflicts retained the upstream seat/grace models plus p7's usage/spill/scope controls. The merged inventory has 90 models and 32 contracts.

Actual verification after p6 landed and the rebase completed (all tests unattended, private HOME, real CARGO_HOME/RUSTUP_HOME, empty RUSTC_WRAPPER; local TCP/Metal/TLC run outside sandbox):

| Command | Exit |
|---|---:|
| `make test-ir` | 0 |
| `cargo fmt --check` | 0 |
| `cargo clippy --offline --locked --all-features --all-targets -- -D warnings` | 0 |
| `cargo clippy --offline --locked --all-features --lib --bin bboxy450-harness -- -D warnings` | 0 |
| `cargo test --offline --locked --features standalone --test import_native --test protocol_golden --test protocol_laws --test context_auto --test anthropic_provider --test acp_agent` | 0 |
| `cargo test --offline --locked --features standalone --test gui_render --test gui_rich --test gui_bridge --test gui_fixtures --test gui_cli` | 0 |
| `cargo run --offline --locked --features standalone -- gui --replay tests/fixtures/gui/rich-all-items.jsonl --headless --screenshot /tmp/p7-review-evidence-jul7x57c/smoke.png` | 0 |
| `python3 -I scripts/checks.py headers` | 0 |
| `python3 -I scripts/checks.py kb-records` | 0 |
| `git diff --check` | 0 |
| `make test-formal` | 0 |
| `python3 -I /tmp/p7-review-evidence-jul7x57c/formal-check.py` | 0 |
| `python3 -I /tmp/p7-review-evidence-jul7x57c/bulk-check.py` | 0 |

The targeted run passed ACP 13, Anthropic 19, context 10, import 13, protocol golden 3 and protocol-law 3 tests. GUI checks passed 40 tests; actual Metal headless smoke passed 22 visible items, 15 kinds, three approval buttons and scrolled viewport. `make test-ir` preserves all 42 locked definitions and passes nine refused/three allowed controls. The frozen IR corpus and lock are byte-identical to main. No old label assertions were changed. `make test-formal` passed 90 native model variants, 18 nonvacuity witnesses and the shared Yarn projection. Full `make test` remains the Oracle integration gate under COMMON.md; it is not claimed here.

Fresh exact native/Java graph and verdict evidence:

| Model | States | Transitions | Java verdict / exit |
|---|---:|---:|---|
| HarnessNativeUsage | 68 | 207 | pass / 0 |
| HarnessNativeSpill | 7 | 6 | pass / 0 |
| BrokenHarnessNativeUsage | 204 | 509 | safety / 12 |
| BrokenHarnessNativeSpill | 8 | 7 | safety / 12 |
| BrokenHarnessNativeUsageScopes | 68 | 207 | safety / 12 |

Usage bounds: two identified calls over two turn endings and three independent cumulative snapshots 1→3→1. The scope control deliberately adds session counters to call totals and is rejected. Spill bounds: one pending small/large record, blob success/failure, persistence before reference. Fairness is local ending/put/append, never arbitrary native IO. Models do not prove Rust refinement, parsing, numeric/UTF-8/JSON byte arithmetic, kernel durability or pixels; the real import/watch tests cover those implementation paths. Expected safety exit 12 is not a tool error.

Fresh 80-file check, 2026-11-02T23:36:10.736459+00:00 through 2026-11-02T23:37:15.845029+00:00: newest 40 per harness by modification time, using private stable copies. Each import and `check-transcript` exited 0. All backend/scope/source assertions, independent native counter totals, per-turn sums, serialized limits, and typed spill hash/length checks passed. Source logs were read only; raw transcripts are not committed.

| Harness | Import/check pass | Fail | Call input | Call output | Total input + output |
|---|---:|---:|---:|---:|---:|
| Claude Code | 40/40 | 0 | 8,974,650,692 | 13,239,319 | **8,987,890,011** |
| Codex | 40/40 | 0 | 2,393,487,734 | 8,982,619 | **2,402,470,353** |

Only call scopes contribute to these totals; derived turns and cumulative sessions are not added again. Largest serialized envelope: 51,913 bytes, below 262,144. Full per-file counts, cache/reasoning subcounters and blob checks are in `/tmp/p7-review-evidence-jul7x57c/bulk.json`; source/model/tool hashes and graphs are under `/tmp/p7-review-evidence-jul7x57c/formal/`, and native evidence is `/tmp/p7-review-evidence-jul7x57c/native.json`. Git retains only the small summary.

Before/after side.png and score.json are N/A for this importer task; no visual layout was changed. Offscreen smoke is `/tmp/p7-review-evidence-jul7x57c/smoke.png`. No Chrome was launched. Any manual Chrome command must use `--use-mock-keychain --password-store=basic --user-data-dir=$(mktemp -d)`; LAB is `/Users/feral/mydev/bboxy450-term-harness-ref/lab/web`.

One merge helper initially failed while combining inventory text; it was corrected before final verification. The merged TOML, conflict-marker scan, inventory shape and IR gate were validated before the signed source squash was amended. No verification was bypassed. There are no remaining review blockers. Generic text spill expansion/fork-copying, file rotation/truncation semantics and global IR-version/usage-CLI work remain outside this accepted task scope.
