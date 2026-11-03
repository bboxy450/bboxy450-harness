---
id: 01M6GXR2A7H6K9P4VYBN8EDQSC
number: 25981
type: report
title: 'Task 5981 — agent-initiated ACP turns'
status: complete
updated: 2026-11-03T16:20:58.671050+00:00
created: 2026-11-03T14:21:07.442293+00:00
implementation: b67931e6d718624ede65502a9e7efcdfae5ad847
links:
  - { to: "[[2026-11-03-agent-initiated-turns]]", rel: reports }
  - { to: "[[d5980-agent-initiated-turns-are-first-class]]", rel: implements }
---
ROLE: EXECUTOR. Worktree `/Users/feral/mydev/bboxy450-harness-agent-turns`, branch `feat/agent-initiated-turns`. No push or merge. The implementation is signed by Boxy using `9B07E849480C68328CA20684844ECC0587DE7AB0`; `git verify-commit HEAD` exited 0.

Implementation and scripted verification are complete. The final full `make test`, targeted ACP/IR/face tests, `make test-ir`, `make test-formal`, native/Java reference parity and GUI/package checks exit 0. Live adapter verification is SKIP, described below.

## Implementation and acceptance evidence

- Protocol: `src/protocol/mod.rs:14` — IR 1.2; `:190` — optional origin, absent means user; `:265` — Quiet with unsigned milliseconds; `:282` — tagged User/Agent/retained Unknown origin. `src/protocol/laws.rs:93` admits an agent turn without fabricating TurnQueued, preserving accepted-turn/end accounting; the authorship check rejects unknown origins. `tests/protocol_golden.rs` verifies legacy absent-origin serialization, known and unknown origin round trips, Quiet serialization, and its read default.
- Compatibility: Quiet.ms has `serde(default)` for reading, so E2's prohibition of new required fields remains enforced. Every writer emits its actual measured value, including zero for an immediate queued-prompt ending. `scripts/ir-compat.py` is unchanged. `make lock-ir` alone refreshed the schema lock and added the new fixture hash; all prior frozen corpus files remain byte-identical.
- Idle work and permissions: `src/backend/acp_agent.rs:620` creates a fresh agent TurnId with no cause or UserMessage and marks it Thinking; `:1435` routes the five work update kinds through the normal mapper; `:1188` opens on an idle permission before normal yolo/unattended/face resolution. No synthetic prompt is sent. Session gauges retain their idle path; existing cancel handling remains in place.
- Quiet and queue: `src/backend/acp_agent.rs:52` defines the 30s default and one-time unsigned-ms override; `:386` shortens only an agent turn with no tool/permission open when a user is queued; `:660` guards closure; `:668` measures actual silence; `:680` rearms an expired timer while work is open; `:977` rearms on every wire message. TurnEnded usage remains None. The idle heartbeat was not modified by this implementation.
- Process witnesses: `tests/acp_agent.rs:868` covers yolo Allow/Rule(profile-yolo), complete tool, no UserMessage, all turn-owned items, Quiet >=200ms, no open items at end, and two separate wakes. `:916` opens directly on a permission and selects reject_once with Resolver::Unattended. `:943` holds a permission/tool past the quiet deadline, queues a user, then ends Quiet below 200ms and starts that user in order. `:1051` proves idle gauges open nothing; `:1078` proves session gauges rearm the deadline of an already open agent turn.
- Broken control: `src/backend/acp_agent.rs:279` samples `BBOXY450_ACP_IDLE_CONTROL=hide` once; its documentation sits beside the timing override. `tests/acp_agent.rs:1015` drives the same wake and asserts no agent TurnStarted, permission outcome cancelled, and rejection by the positive lifecycle assertion. The fixture agent's wake/wake-twice paths are in `tests/fixtures/acp-agent/fake_agent.py`.
- Faces: `src/frontends/gui/model.rs:152` retains origins; `src/frontends/gui/face_rows.rs` puts the one-line heading on the first visible item, including folded/grouped cards; `src/frontends/gui/view.rs` covers fallback row kinds. The rich receipt preserves the Quiet title. `src/frontends/seat.rs:1073` prints the heading; `src/frontends/acp.rs:652` exports it as an agent chunk without inventing a prompt response. `tests/ir_v1_corpus.rs:262` checks seat work and state=working while the agent turn is open, then idle after Quiet; `:312` renders the rich fixture without an agent-origin user bubble and checks work visibility. A rule-resolved approval retains the current face's folded presentation. The ACP fixture projection test in `src/frontends/acp.rs:1145` validates notifications against the pinned SDK schema and checks heading/text/tool output without a prompt response.
- Formal: `tests/formal/models.rs:2272` extends HarnessAcpAgentTurn to 16 bounded variables, with agent-origin idle work/permission open, one tool, abstract quiet, every-wire rearm, queued-user shortening and one ending. NoWorkItemOutsideTurn, PermissionCancelledOnlyAfterCancel, QuietEndsOnlyWithoutOpenWork and AgentTurnEventuallyEnds are checked. `:2552` proves the agent/permission/shortened-end/ended states reachable and the idle-cancel control rejected. `verification/contracts.toml` records finite bounds, fairness, abstract timer and real-duration witnesses; `verification/corpus.toml` inventories BrokenHarnessAcpAgentIdleCancel; `verification/coverage.toml` states implementation-only timing/bytes/rendering coverage. Native check: good model 1,253 reachable states; idle-cancel control 1,043 states, expected safety failure. No arbitrary Rust refinement or full TLC replacement claim.

## Verification after chrome rebase

Rebased without conflict onto origin/main `0c403165e815c2f7bf77ccac293784b021f42f96` (0c40316), including the tight transcript, tool groups/icons and standalone chrome. The rebased implementation is `b67931e6d718624ede65502a9e7efcdfae5ad847`. Its source did not need further changes. Report-only metadata is amended after the checks; native/Java evidence is regenerated afterward to bind the final signed HEAD.

Reported acceptance runs use a private HOME/TMPDIR with explicit CARGO_HOME/RUSTUP_HOME and empty RUSTC_WRAPPER (sandboxed sccache returned EPERM). No operator input is required. Raw artifacts are not committed.

- `cargo fmt --check` and both COMMON Clippy gates — exit 0; `/tmp/agent-turns-chrome-fmt.log`, `-clippy.log`, `-clippy-ui.log`.
- Targeted ACP, frozen IR, protocol and GUI suites, including `gui_chrome` — exit 0; `/tmp/agent-turns-chrome-targeted.log`. The exact commands and exits for all scoped checks are `/tmp/agent-turns-5981.BLVgoB/chrome-rebase-checks.json`.
- `make test-ir` — exit 0; 47 definitions, 12 frozen logs, 10 forbidden controls refused and 4 legal controls accepted in both schema forms; `/tmp/agent-turns-chrome-ir.log`.
- `make test-formal` — exit 0; 96 models, one crate inventoried; `/tmp/agent-turns-chrome-formal.log`.
- Full `make test` — **exit 0**, source `4f05e76ce5a59d89ea42d4db8d68096aee70be7b`; all 20 canonical script commands exit 0, private_home=true, operator_input=false. Native/Java parity, GUI gate, Metal replay smoke and unsigned packaging checks pass. Log: `/tmp/agent-turns-chrome-make-test.log`; actual command/exit records: `/tmp/agent-turns-5981.BLVgoB/chrome-artifacts/test-current.json` and `gui-gate.json`; timing/exit: `/tmp/agent-turns-5981.BLVgoB/chrome-full-test-result.json`.
- `make check-formal-evidence` — exit 0 after that full gate; 96 models, 34 contracts, one crate, dirty=false, exact native/reference reachable-state and labeled-transition graph equality. `/tmp/agent-turns-chrome-receipt.log`; current HEAD-bound evidence remains at `target/formal/evidence.json` and in untracked `verification/evidence/<digest>/` beyond target cleanup.
- Direct `BBOXY450_ACP_IDLE_CONTROL=hide` witness — exit 0 for the expected inversion; `/tmp/agent-turns-chrome-broken.log` contains `BBOXY450_ACP_IDLE_CONTROL=hide: hidden work, cancelled permission; positive assertion rejected`. The positive assertion panics as expected and the inversion test catches it.
- `check-transcript tests/fixtures/ir-v1/v1.2-agent-turn.jsonl` and the rebased 900x1100 offscreen fixture capture — exit 0.

The first pre-chrome full attempt found two retained ACP extension shapes without pending records. They are now registered in `tests/fixtures/ir-v1/pending/31716927b7dd41c5e61cfa2fdb822107bea5ba4b1781f346884d69252d948c2e.jsonl` and `490726cf84dc9ee192dd066d99bdca8547fd60bb27aaf381f42325383975fa99.jsonl`. The scanner and frozen log were unchanged; the canonical full-corpus scan now passes. Historical failed output remains `/tmp/agent-turns-make-test-first.log`.

Fixture: `tests/fixtures/ir-v1/v1.2-agent-turn.jsonl`, SHA256 `3b97b9d90c16638f3e04d56dc267d19947bbf72611475b9af4f4c008ee94acd6`. This is the first scripted wake test's durable log with session/op/item/turn IDs, absolute times and local paths redacted. It retains event order, relative times, policy outcomes and the measured Quiet duration. The frozen-corpus reader calls check_authored on it.

Visual capture on chrome base: `/tmp/agent-turns-5981.BLVgoB/chrome-agent-turn.png` (inspected); previous-base capture: `/tmp/agent-turns-5981.BLVgoB/agent-turn.png`. Before/after side.png and score.json are N/A: this task adds a lifecycle witness, and does not modify parity fixtures or the parity baseline. The rich capture displays agent continued, woke, Wake tool with its result, done and Quiet.

Live adapter: **SKIP**. `claude-agent-acp` is installed and profile claude-acp exists, but no ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN or CLAUDE_CODE_OAUTH_TOKEN is present, and no file credential exists. Keychain login was not probed because this witness must run without an operator prompt. Availability record: `/tmp/agent-turns-5981.BLVgoB/chrome-real-adapter.json`. There is no PASS claim for the real sleep/WOKE-file scenario; that behavior remains unproven in this task.

## Scope and integration

The typed origin field also required mechanical constructor migrations in `src/engine/mod.rs`, `src/import/mod.rs`, `tests/seat_golden.rs`, `tests/store_fork.rs`, and `tests/ir_v1_additions.rs`. The last preserves its frozen 1.1 version instead of regenerating old bytes. The new broken control additionally requires `verification/corpus.toml`; omission fails the formal inventory. The two pending JSON records above are also required by the existing full-corpus scanner gate. These mechanical migrations and fixture registrations are the only paths beyond the task's explicit allowed list. Existing attribution headers and root ATTRIBUTION.md remain intact.

Rebased onto origin/main including the tight transcript and parallel-tool-group changes; the latter's group rail/inset/caption is preserved around the new heading. Base at handoff: origin/main `0c403165e815c2f7bf77ccac293784b021f42f96` (0c40316). The separate status-heartbeat branch has not landed at handoff. StateReporter already treats every TurnStarted as open independent of origin, and the fixture witness asserts this for an agent turn. Whoever lands second must rebase and refresh current-source evidence.

ICD §7 / matrix R31 evidence line for the Architect: `tests/acp_agent.rs:868, :916, :943, :1015, :1051, :1078; tests/ir_v1_corpus.rs:262, :312; src/frontends/acp.rs:1145; HarnessAcpAgentTurn + BrokenHarnessAcpAgentIdleCancel — scripted idle work/permission opens origin=agent without user input, policy selects allow_once/reject_once, quiet closes only without open work, a queued user shortens the wait and follows it, all faces show the continuation, seat state remains working while open; real adapter SKIP.`

Worktree disposition: task source and report are signed commits, with no push or merge. Generated tracked run summaries/PNGs are copied to the evidence directory above and restored to their original tracked contents. `verification/evidence/` is an intentionally untracked durable formal artifact store, retained for review and never staged.
