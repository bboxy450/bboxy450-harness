---
id: 01M6F69S3ZQH59VFPV8JWXC977
number: 7007
type: report
title: p7-import-usage — native usage and bounded blob previews
status: complete
created: 2026-11-02T22:32:54.655375+00:00
implementation: f2682f547b408e2d66e65c5fb64aab9d0867969f
updated: 2026-11-02T23:19:20.978727+00:00
---

COMPLETE — p7-import-usage, including the Oracle update to rebase and follow log IR v1 §4a. Worktree `/Users/feral/mydev/bboxy450-harness-p7-import-usage`, branch `parity/p7-import-usage`. Rebased onto origin/main snapshot `26ebcf178a571acaf5a80d682d0d6fd316df5f2b`; current implementation `f2682f547b408e2d66e65c5fb64aab9d0867969f`. Rebase conflicts retained both upstream ACP formal models and p7 models. Every rewritten task commit is signed by Boxy <bboxy450@gmail.com> using `9B07E849480C68328CA20684844ECC0587DE7AB0`. No push or merge.

Read the binding spec from origin/main, now available as `.bboxy/specs/log-ir-v1.md`. `make test-ir` passes: 37 locked definitions preserved, nine forbidden controls refused and three permitted additions accepted. The lock and frozen `tests/fixtures/ir-v1/*` are untouched. Only optional Usage fields/tolerant enum definitions were added to the wire schema; existing fields retain their types and meanings. The branch includes 7da3091. The parity script retains `--use-mock-keychain --password-store=basic`; p7 launched no Chrome process.

Files and acceptance evidence:

- `src/protocol/mod.rs:403` — optional `scope`, `model`, `backend`, `request_id`, `context_used`, `context_window`, `source`, and default-empty `unreported`; typed `UsageScope`, `Backend`, `BackendKind`, `NumberSource`, each enum with Unknown. `protocol.schema.json` is freshly generated; frozen logs and the lock were not edited.
- `src/import/mod.rs:184` — Claude usage counted once by message/request aliases, with UUID fallback. It records exact reported model/request identity and native `version`, with canonical input including reported cache tiers. `src/import/mod.rs:238` — Codex last-call counters are call scope; cumulative totals are separate session scope, deduplicated by cumulative snapshot, with resets preserved. CLI version comes from `session_meta.cli_version`. A cumulative-only record never invents a model call or a turn. Session totals do not claim the current model billed all previous calls.
- `src/import/mod.rs:162`, `:603`, `:642` — every native call/session record has `backend.kind=import`, native backend name, and `source=reported`. Summed turn totals are `scope=turn`, `source=derived`, with no request ID. Unknown optional numbers stay None. Missing required numbers use zero plus their exact names in `unreported`; reported zero has no missing marker. Empty usage produces no record. A reported context window alone produces session scope with named required placeholders. Unrepresentable turn sums stay unknown. No prices, costs or token estimates are fabricated.
- `src/import/spill.rs:42`, `src/import/watch.rs` — both import and live watch persist full oversized text before appending bounded UTF-8 previews. Tool output sets `ToolCall.spill`; other large text carries a visible blob digest. JSON-escaped size is considered; redaction precedes truncation, and failed blob writes publish no dangling reference.
- `src/context/auto.rs:69`, `tests/context_auto.rs:510` — session/turn totals and unreported call counts cannot become context-size anchors. Legacy unscoped anchors retain their behavior. `src/provider/parser.rs` defaults new optional fields. `src/provider/http.rs` and `src/frontends/gui/model.rs` box internal parser/turn-state payloads after the larger Usage triggered Clippy's enum-size gate; wire fields were not boxed or retyped.
- `tests/import_native.rs:115`, `:192`, `:260`, `:336`, `:375`, `:451`, `:466`, `:538`, `:589` — 13 passing import/watch tests cover deduplication, scopes, provenance, native versions, exact model/request IDs, absent versus reported zero, context-only records, overflow, Unicode/control-character spills, blob bytes, live append, failed writes, redaction and continue-on-ACP handoff. Fixtures: `tests/fixtures/native/{claude,codex}-{usage,spill}.jsonl`. Existing frozen/golden logs remain unchanged.
- `tests/formal/models.rs`, `tests/formal_nonvacuity.rs`, `verification/{contracts,corpus,coverage}.toml`, `verification/README.md` — native usage and spill contracts plus the new scope-mixing control are inventoried. The usage model keeps two call identities across two endings, and three independent cumulative snapshots 1→3→1. Snapshots cannot increase call/turn totals. Required-number labels, byte arithmetic and parsing remain implementation witnesses. `ATTRIBUTION.md` and source headers retain the policy.

Actual final gate results (private HOME, real CARGO_HOME/RUSTUP_HOME, empty RUSTC_WRAPPER, unattended):

| Command | Exit |
|---|---:|
| `make test-ir` | 0 |
| `cargo fmt --check` | 0 |
| `cargo clippy --offline --locked --all-features --all-targets -- -D warnings` | 0 |
| `cargo clippy --offline --locked --all-features --lib --bin bboxy450-harness -- -D warnings` | 0 |
| `cargo test --offline --locked --features standalone --test import_native --test protocol_golden --test protocol_laws --test context_auto --test anthropic_provider` | 0 |
| `cargo test --offline --locked --features standalone --test gui_render --test gui_rich --test gui_bridge --test gui_fixtures --test gui_cli` | 0 |
| `cargo run --offline --locked --features standalone -- gui --replay tests/fixtures/gui/rich-all-items.jsonl --headless --screenshot /Users/feral/mydev/bboxy450-harness-p7-import-usage/target/p7-evidence/smoke.png` | 0 |
| `python3 -I scripts/checks.py headers` | 0 |
| `python3 -I scripts/checks.py kb-records` | 0 |
| `git diff --check` | 0 |
| `make test-formal` | 0 |
| `python3 -I .bboxy/runs/p7-import-formal/check.py` | 0 |
| `python3 -I .bboxy/runs/p7-import-gates/bulk-check.py` | 0 |

The combined import/provider/context/protocol command passed 19 provider + 9 context + 13 import + 3 golden + 3 law tests. The GUI command passed 40 tests. Actual offscreen Metal smoke passed with 22 visible items, 15 kinds, three approval buttons and scrolled viewport. `make test-formal` passed all 86 native variants, the nonvacuity witnesses and the shared Yarn reactor projection. Targeted Java TLC compared complete reachable state and labeled-transition sets, with exact parity for all five p7 variants:

| Model | States | Transitions | Java verdict / exit |
|---|---:|---:|---|
| HarnessNativeUsage | 68 | 207 | pass / 0 |
| HarnessNativeSpill | 7 | 6 | pass / 0 |
| BrokenHarnessNativeUsage | 204 | 509 | safety / 12 |
| BrokenHarnessNativeSpill | 8 | 7 | safety / 12 |
| BrokenHarnessNativeUsageScopes | 68 | 207 | safety / 12 |

Expected safety exits are successful broken controls, not tool errors. Fairness is local ending/put/append only; native IO fairness is not assumed. Models do not prove arbitrary Rust refinement, parsing, token arithmetic, actual filesystem durability or pixels. Full `make test` and whole-corpus Java parity remain the Oracle integration gate under COMMON.md; no such run is claimed.

Fresh bulk run 2026-11-02T23:01:44.249237+00:00 through 2026-11-02T23:03:02.844348+00:00: newest 40 files per native harness by modification time, using private stable copies removed on exit. Every import and `check-transcript` command exited 0. Independent native-record token checks, scope/backend/source assertions, per-turn sums, envelope bounds and typed spill SHA256/length checks all passed. Source files were read only. Raw native/imported transcripts were not committed.

| Harness | Import/check pass | Fail | Call input | Call output | Total input + output | Call records | Session snapshots |
|---|---:|---:|---:|---:|---:|---:|---:|
| Claude Code | 40/40 | 0 | 8,960,684,005 | 13,218,626 | **8,973,902,631** | 20,048 | 0 |
| Codex | 40/40 | 0 | 2,461,853,224 | 9,148,046 | **2,471,001,270** | 11,033 | 11,033 |

Only call scopes contribute to these totals. Derived turn totals and cumulative session snapshots are never added again. Known reported cache/reasoning subcounters remain in the bulk JSON. Claude produced four tool spill references/seven unique blobs; Codex produced 29 tool spill references/29 blobs. Largest serialized envelope: 51,913 bytes, below 262,144.

Committed evidence is limited to the small command/result/hash summary `.bboxy/runs/p7-import-usage.json`, as required by the revised COMMON.md. Raw logs, five TLA/config/state/edge sets, bulk output and screenshot are preserved outside Git under `/tmp/p7-import-usage-evidence-n_yn712z`. The summary binds their hashes; all archived source/model/gate artifact hashes were verified after copying. The pre-existing generated `.bboxy/runs/formal-native.json` is restored to the rebase baseline, rather than committing a regenerated run. Before/after side.png and score.json are N/A for this importer task; no visual layout or labels were changed by p7.

Resolved gate failures: the larger metadata exposed two enum-size warnings, fixed by internal boxing. Sandboxed provider fixtures could not bind local TCP listeners; all final gates passed outside the sandbox with private homes and actual Metal access. Earlier sandbox sccache/Metal/TLC restrictions and the nullable-metadata bulk-checker issue were already resolved unattended. No operator click, Keychain prompt, pane authorization or physical probe was used.

Remaining scope limits: global protocol 1.0/1.1 promotion, all-backend authored validation, cost/price records and the offline usage CLI belong to the separate IR task. File rotation/truncation semantics are unchanged. Generic text blob markers are visible; automatic full-text expansion and fork-copying those generic marker blobs are not added. Typed tool spills retain the existing blob/fork path. Sparse native identities permit only metadata-supported deduplication. No assigned check is blocked. Report: [[p7-import-usage]].

COMMON.md was re-read after the Oracle correction. The reference is `LAB=/Users/feral/mydev/bboxy450-term-harness-ref/lab/web`. Any manual headless Chrome invocation must include `--use-mock-keychain --password-store=basic --user-data-dir=$(mktemp -d)`. No Chrome was launched for p7. Implementation and checked model sources are unchanged by the evidence-packaging correction.

Oracle amendment audit (2026-11-02T23:19:20.978727+00:00): re-read log IR §4a from `c4246de2a64a8b25bcd8dfe2f18dd2b77d18ecf8`. The existing implementation already has one scalar `Usage.source: Option<NumberSource>` and `unreported: Vec<String>`, with no per-field source map (`src/protocol/mod.rs:403`). Native call/session records are reported; summed turn records are derived. Required missing counts are named zero placeholders and reported zero is distinct (`src/import/mod.rs:603`, `:642`). P7 emits no unreported Cost record; any authored Cost has its separate source under §4a. No implementation change was necessary. Re-ran `cargo test --offline --locked --features standalone --test import_native partial_usage_names_absent_required_counts_and_preserves_reported_zero` (exit 0, one test covering both native formats), `make test-ir` (exit 0), and explicit schema assertions for scalar source/string-array unreported/no sources map (exit 0). Raw audit logs are under `/tmp/p7-import-usage-evidence-n_yn712z`; their hashes are in the sole small summary. Checked source/model inputs remain unchanged.
