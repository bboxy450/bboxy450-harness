---
id: 01M6F5K006EZR2P6T3RY8YPVVS
number: 202611021402
type: report
title: 'p6-seat-state — seat reports and ACP gap witnesses'
status: completed
created: 2026-11-02
role: Executor
task: p6-seat-state
---
# p6-seat-state

Completed the Oracle's pre-landing rework. The task's commits are squashed onto their own merge-base, then rebased after p4 landed as `98db33a8a54edb0eca3255da88ddfaf2bf8018c5` ("Last transcript row keeps the web feed's 18 px bottom padding"). The final parent is `26ebcf178a571acaf5a80d682d0d6fd316df5f2b`, which includes p4 and the subsequent parity-baseline updates. One signed task commit remains above that parent.

## Implementation retained

- `src/frontends/seat.rs:14`, `:107` and `src/frontends/headless.rs` — TTY-only OSC 7501 state/title reports; immediate split-probe filtering without Enter; normal line input; approval, error and profile-switch projection. Exact bytes and real unattended PTY behavior remain covered by seat_golden/seat_process.
- `src/backend/acp_agent.rs:1562` — model matching uses exact value, provider suffix, then offered whole-token aliases.
- `src/protocol/mod.rs:403`, `src/backend/acp_agent.rs:1503`, `protocol.schema.json` — canonical log IR §4a Usage types and session usage_update writer. There is one Usage.source, a separate Cost.source, and unreported placeholder zeros. Context, backend identity, currency and legacy micro-USD semantics are preserved; empty/invalid accounting is not invented. Idle session accounting is retained.
- `src/context/auto.rs` excludes session gauges/missing call counters from request-estimate anchors. Provider/parser and GUI model indirection preserve wire compatibility and enum-size gates. Tests cover typed accounting, old logs and handoff estimates.
- The private cancel-grace override and fake-agent group witness still prove ignored cancel, exactly one Interrupted ending, the warning and no surviving owned group.
- Rebase conflict resolution retained both upstream AgentPath and p6 SeatReport/grace formal families. The inventory and README now agree on **85 variants / 30 contracts**; no contract was dropped.

## Artifact cleanup and history

Only `.bboxy/runs/p6-seat-state.json` is added under runs. It is a small command/exit/hash summary. `.bboxy/runs/p6-seat-state/` is absent from every task commit, including the earlier provisional logs and two large evidence JSONs. Raw logs, full formal graphs/models/traces/JSON, archives and receipts are outside Git at `/tmp/p6-seat-state-evidence-c3qvt4ch`; new rework output is under its `review-rework/` directory. Raw files were preserved, not discarded.

`.bboxy/runs/formal-native.json` matches the new parent byte for byte. The gate generates it at runtime but never reads its committed version as an acceptance input; the runner copies generated evidence outside Git and restores the tracked parent copy. No regenerated native receipt is committed.

## Post-p4 checks

All use private test homes, real CARGO_HOME/RUSTUP_HOME and no operator interaction. Exact command text, exit codes and log hashes are in the compact summary; raw output remains outside Git.

| Check | Result |
|---|---|
| `make test-ir` | exit 0; 37 locked definitions, frozen hashes and all mutation controls pass |
| all-features ACP/seat/headless/protocol/Anthropic/context tests | exit 0; **72 passed**, one child fixture ignored by direct execution and invoked by its parent crash witness |
| `cargo fmt --check` | exit 0 |
| all-features/all-targets clippy with `-D warnings` | exit 0 |
| GUI clippy under its CLIPPY_CONF_DIR | exit 0 |
| standalone gui_render/gui_rich/gui_bridge/gui_fixtures/gui_cli | exit 0; **40 passed** |
| native headless GUI smoke | exit 0; 22 visible items, 15 kinds, 3 approval buttons; `/tmp/p6-seat-state-after-p4.png` |
| `make test-formal-parity` | exit 0; all **85** complete state/labeled-edge sets and registered safety/deadlock/liveness verdicts match native and pinned Java TLC |
| headers, secret scan, KB records and malformed-record control | each exit 0 |
| artifact-path audit and `git diff --check` | exit 0 |

The post-rebase formal receipt before the final metadata amend has source `17de2a4d7d327d387eb5a8bad9e25e427203da40`, dirty=false, SHA256 `353c2805a48cc677486a7d1132f55dcf77fd71bdcdd5c779c32a38341c5776a3`. Its input hashes match the final source. Current signed-HEAD evidence is refreshed after the metadata amend and linked in the external handoff; it stays outside Git. These are finite bounded contracts, not a claim of arbitrary Rust correctness or complete TLC replacement.

## Handoff

The commit is signed as Boxy <bboxy450@gmail.com>, key `9B07E849480C68328CA20684844ECC0587DE7AB0`, with `Report: [[2026-11-02-p6-seat-state]]`. Source/schema/behavior were retained from the Oracle-accepted code; this rework removes artifacts, resolves additive formal inventory conflicts and refreshes integration evidence. No push, merge, release or publication was performed. The full make test remains with the Oracle per COMMON. Chrome was not needed; any manual launch retains the mandatory mock-keychain/basic-password-store/fresh-profile flags. Visual side.png/score.json are N/A for this task.
