---
authorship:
  kind: seat
  process_generation: '1'
  public_key: u0vQJ7ngwMS5yr2jtfd3HrkhSWQXq2tpEWFJBh5W+4A=
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
created: 2026-11-01T08:53:16.408Z
created_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
id: 01M6B508DHRETFHE9NEXHZS2TW
number: 3022
provenance:
  digest: sha256:ac7e84a289523d8246bf30a1f3d8bcb209ce3845080264629c74e1ffabe8e751
  frontmatter_digest: sha256:fb1e528a8d704547b3633aec8b3c769e9947fc792bdd15de3bc34a4ecded1545
  log: .bboxy/log/2026-11-01.jsonl#19175
  sig: EhGVnPSi8GgRhCZp2mM4CbBC1aEZYcdTETieF75LF5/UjEamdwdI7/p9cwOv3A6cuI7YWcqdYOqGrANpO6BgAg==
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
type: note
updated: 2026-11-01T09:15:29.619Z
updated_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
---
# E1c handoff — ACP, MCP and automatic compaction

Task: [[2026-10-31-harness-acp-mcp-compaction]] (#1992). Executor: E1 harness-core.
Worktree: `/Users/feral/mydev/bboxy450-harness-core`.
Branch: `e1c-acp-mcp-compaction`. No merge performed.

The implementation follows the baselined ICDs, including protocol **0.4**
(`Effect::Read`) and the queued-cancellation frontier clarification in `72d6549`
(cherry-picked as signed `237f441`). The pin comparison preceded code changes
in signed `ed99393`. Implementation commits: signed `1e8925e`, `52dd070`, `a757963` and the
item-7 fixture follow-up `ca59d8e`.
The full gate and live smoke ran on `a757963a71ed11d93111d3034ed4f7d47a9c960d`.
The final signed handoff head and PR URL are supplied by `task_report`.
Item 7 is now complete: 200 consecutive seat-process runs and the full gate
pass; the audit and new evidence appear at the end of this report.

## Files, mechanisms and acceptance evidence

1. **ACP pin.** [acp-1.12-diff.md](acp-1.12-diff.md) checks every relied-on
   method, capability, content, permission and framing fact against schema 1.12.0.
   Valid wire mappings are unchanged. Malformed-array handling is stricter;
   permission replies retain their outer `outcome` wrapper. SDK 3.4.0 is pinned
   without unstable features in `Cargo.toml` / `Cargo.lock`.
2. **ACP face.** `src/frontends/acp.rs` implements stdio JSON-RPC, per-session
   subscriptions and pending prompt replies, suffix repair, item mappings,
   permission requests, replay-before-load-response, restricted capabilities,
   malformed-input recovery and owned shutdown. `src/frontends/headless.rs`
   dispatches `acp` and defers credentials into the cancellable Host factory.
   `src/gateway/{host,mod}.rs` supports independent loaded sessions and records
   ACP approval provenance. `tests/acp_face.rs` drives real scripted-provider
   turns, a real stdio process, permissions, queued cancellation and concurrent
   sessions; `tests/fixtures/acp/*.jsonl` covers tool/diff/terminal and loaded
   Plan output. The internal forced-drop test exercises both completion and
   snapshot repair and asserts zero divergences. Logs pass `check_transcript`.
3. **MCP client.** `src/mcp.rs` owns session TaskGroup supervisors and process
   groups, fixed turn snapshots, normalized names/limits, generation fencing,
   timeout/cancellation, five-attempt exhaustion, debounced re-listing, scrubbed
   stderr excerpts and result/blob mapping. `src/tools.rs` and engine policy
   route every call through `decide`; read concurrency never implies policy
   trust. `tests/bin/mcp_fixture.rs` is feature-gated test infrastructure.
   `tests/mcp_client.rs` proves concurrent reads with a two-party barrier,
   approval for untrusted writes, explicit Ask/Deny for trusted reads, crash,
   timeout, cancellation notification, changing snapshots, five process births,
   secret scrubbing, removal of the reserved capability and Close during a
   hung re-list. The final MCP suite has 11 passing tests.
   `src/process.rs` / `src/provider/profile.rs` also bound command resolution's
   stdout and cancellation without spilling credential bytes to disk.
4. **Automatic compaction.** `src/context/auto.rs`, `src/context/mod.rs`,
   `src/engine/{history,mod,turn_loop}.rs` count the exact wire JSON / Usage
   anchor, apply the 80% trigger and 60% ladder target, persist reductions before
   model requests, clip completed tool output and omit completed reasoning /
   images before summarizing. Rebuilt history retains tool pairing and the
   cache prefix. Both reductions use the latest ending below every open turn.
   `src/protocol/{mod,laws}.rs`, `protocol.schema.json` and regenerated protocol,
   compact and seat fixtures carry the new item and enforce its boundaries.
   `tests/context_auto.rs` covers 30 turns, exact thresholds/Unicode/Usage,
   single-request overflow, unknown-window notice, recovery, real SIGKILL,
   failed blob durability and queued cancellation. Its ignored child helper is
   launched explicitly by the passing parent SIGKILL test. No operator input.
5. **Formal.** `tests/formal/models.rs` adds HarnessAutoCompact and
   HarnessMcpServer; `tests/formal_nonvacuity.rs` asserts named properties,
   reachable A→B→A held responses and the queued-cancellation frontier.
   `verification/{corpus,contracts,coverage}.toml` inventories R22–R24 and the
   actual sources, bounds and implementation-only witnesses. Models use the
   existing crates.io bboxy450-tlc/Yarn stack and pinned Java parity gate.
   [model-controls.json](e1c/model-controls.json) records all seven new broken
   controls, their actual counterexamples and the passing corresponding model.
6. **Live smoke.** `scripts/smoke-live.py` and `src/frontends/smoke.rs` add an
   ACP-driven read/edit turn using the same free nemotron profile. Each face
   has a hard cap of three provider requests. Only the existing `!cmd` is copied
   into the profile; credentials and raw transcripts are never committed.
   Final outcome: **PASS**, exit **0**, on both faces. Each used exactly three
   requests and recorded one read, one edit, the changed file, EndTurn and
   `check_transcript: true`. Redacted record:
   [smoke-live-2026-11-01.json](smoke-live-2026-11-01.json).

## Red → green evidence

`CARGO_BUILD_JOBS=2 python3 -I scripts/test-e1c-controls.py` exited **0**.
Each control temporarily mutates the named production mechanism, asserts the
specific behavioral failure, restores the source in `finally`, and runs the
same test green. Source digests and exact commands are in
[process-controls.json](e1c/process-controls.json); output is in the adjacent
`*-red.log` and `*-green.log` files (trailing whitespace normalized only).

| Control | Red exit | Restored exit |
| --- | ---: | ---: |
| Disable MCP call timeout | 101 | 0 |
| Keep a crashed MCP call pending | 101 | 0 |
| Wait for MCP readiness before starting a turn | 101 | 0 |
| Ignore cancellation during tools/list re-list | 101 | 0 |
| Restore the prior unbounded pipe resolver | 101 | 0 |

The queued-cancellation process/API witness separately failed with
`queued cancellation compacted the running turn` (**101**), then passed (**0**)
after the signed ICD clarification and frontier fix. See
[queued-cancel-red.log](e1c/queued-cancel-red.log) and
[queued-cancel-green.log](e1c/queued-cancel-green.log). The green command includes
`context_auto`, `formal_models`, `formal_nonvacuity` and `protocol_laws`.
The law witness also rejects the overtaken cut with the exact
`context-boundary / reduction exceeds completed frontier` detail.

Formal red controls are expected checker failures, not failing gate commands:

| Model control | Counterexample |
| --- | --- |
| BrokenHarnessAutoCompactA | safety: NeverReducesOpenTurn |
| BrokenHarnessAutoCompactB | safety: BoundariesMonotone |
| BrokenHarnessAutoCompactC | liveness: BoundaryEventuallySends |
| BrokenHarnessAutoCompactD | safety: NeverReducesOpenTurn after queued cancellation |
| BrokenHarnessMcpServerA | deadlock with readiness-gated admission |
| BrokenHarnessMcpServerB | safety: NoStaleResult |
| BrokenHarnessMcpServerC | liveness: CallEventuallyEnds |

## Commands and actual exits

| Command | Exit / evidence |
| --- | --- |
| Initial `CARGO_BUILD_JOBS=2 make test` | **2**, seat-process test races; retained in `e1c/make-test-initial.log` and `e1c/test-initial.json` |
| Gate after lifecycle fixes, before KB metadata repair | **2**, all Rust and 52-model native/TLC checks passed; KB rejected the unindexed pin note. `e1c/make-test-kb-failure.log`, `e1c/test-kb-failure.json` |
| Targeted `cargo test --offline --locked --quiet --features acp,test-fixtures --test mcp_client --test seat_process` | **0**, 11 MCP + 3 seat tests; repeated in the final `e1c/make-test.log` |
| Final `CARGO_BUILD_JOBS=2 make test` | **0**; `e1c/make-test.log`, `test-current.json`, `gui-gate.json` |
| `make smoke-live` (one invocation for E1c) | **0**, PASS on both faces; `e1c/smoke-live.log`, `smoke-live-2026-11-01.json` |
| `python3 -I scripts/checks.py headers` | **0**, PASS |
| `python3 -I scripts/checks.py secrets` | **0**, PASS |
| `git verify-commit` for implementation commits | **0**, Boxy / `9B07E849480C68328CA20684844ECC0587DE7AB0` |

The initial gate races were corrected in the test producer/control: publish the
key PID by atomic rename, and send SIGHUP explicitly after the PTY reader's
readiness witness rather than relying on the timing of PTY descriptor release.
Functional shutdown deadlines remain; no performance wall-clock gate was added.

## Authorized GUI hunks and limits

Decision [[d2400-e1c-scope-gui-display-arms]] widens this task only for plain
display arms. E3 owner review is required before the Oracle merges:

- `src/frontends/gui/view.rs::kind_label`: ContextTrim label.
- `src/frontends/gui/view.rs::display_body`: readable ContextTrim boundary.
- `src/frontends/gui/smoke.rs`: ContextTrim expectation and required text;
  compaction expectation derives the fixture's actual boundary. The latter was
  confirmed by the Architect in message `01M6B07DS3AT4CRHF1ZYWKKTC9`.

No other GUI behavior, state or layout changed. The all-items fixture's formerly
invalid early compaction was corrected under the protocol ruling (`46df005`),
with derived goldens regenerated. Plan remains witnessed by load replay, as the
ICD requires; no plan-producing tool was invented.

Native/TLC parity proves the recorded finite graphs and verdicts under their
listed fairness/bounds. It does not prove arbitrary Rust refinement or full TLC
replacement. Unsupported ACP methods and HTTP/SSE MCP transports remain the
ICD's explicit degradation. All 52 formal models (21 contracts, one crate) have identical full reachable
state/transition sets and expected native/TLC verdicts. The GUI smoke rendered
22 visible items, all 15 kinds, three approval buttons and the scrolled viewport.
No known implementation blocker remains. E3 review of the listed GUI hunks is
still required before merge. Prior untracked E1b artifacts are preserved.

The records commit changes no verification inputs. Current-head formal evidence
is refreshed after that commit and retained under `verification/evidence/`;
the final `task_report` supplies its digest and the signed head. Regenerated
receipts/logs are run outputs, distinct from the clean source tree.

## Item 7 — d2019 PID publication follow-up

The seat fix was already present in signed `52dd070`: `echo $$` writes
`key.pid.tmp`, then `mv` publishes `key.pid`. The initial empty-file
failure remains in `e1c/make-test-initial.log`.

Required command:

```sh
for i in $(seq 200); do CARGO_BUILD_JOBS=2 cargo test --test seat_process || exit 1; done
```

Actual exit: **0**. All **200 consecutive runs** passed all three tests:
**600 passed, 0 failed, 0 ignored**. The log contains exactly 200 complete
three-test success summaries and one test-binary path. The seat source matches
the committed fix byte for byte. [seat-200.json](e1c/seat-200.json) records the
source and binary SHA256; [seat-200.log](e1c/seat-200.log) retains the output.

The requested audit found and fixed three other producers in signed
`ca59d8e`, using a temporary file in the same directory followed by rename:

| Location | Finding / mechanism |
| --- | --- |
| `tests/profile_lifecycle.rs:16` | Key-command PID was created before its bytes were written; publish it by rename. |
| `tests/store_recovery.rs:198` | Recovery fixture's group PID had the same exists-before-write race; publish it by rename. |
| `tests/bin/mcp_fixture.rs:29`, consumed by `tests/acp_face.rs:525` | Escaped-child PID marker could become visible before `std::fs::write` completed; write a sibling temporary file and rename it. |
| `tests/cancel_process.rs:11` | Existing helper retries until the PID parses. Direct reads at :71/:76 follow LEADER_EXITED: both shell writes completed before the leader exited, and only then can the member emit the marker. No change needed. |
| `tests/mcp_client.rs:281` / :569 | PID lists are read after Close joins the server or after terminal fifth-failure notification. Each append completes before initialization; no writer remains at the read. No change needed. |
| `tests/seat_process.rs:130` | Already atomic; the 200-run stress provides the requested evidence. |

Machine-readable inventory: [pidfile-audit.json](e1c/pidfile-audit.json).

`CARGO_BUILD_JOBS=2 make test` on `ca59d8e` exited **0**, unattended.
The canonical stack includes the affected ACP, profile, recovery and cancellation
tests, all 52 native/TLC models and the GUI gate. Evidence:
[item7-make-test.log](e1c/item7-make-test.log), `test-current.json`,
`gui-gate.json`, [item7-formal-code.json](e1c/item7-formal-code.json).

This follow-up changes only test fixtures and evidence. The earlier live PASS
remains the observed smoke result on unchanged product code; no additional live
requests were made. Current-head formal receipts are refreshed after the records
commit and named in the task report.
