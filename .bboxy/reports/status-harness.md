---
id: 01M6GV82PX2T0C00JMQD3FAZX4
number: 20261103135813
type: report
title: 'status-harness — open-turn state heartbeats and live phase details'
status: completed
created: 2026-11-03
role: Executor
task: status-harness
---
# status-harness — Oracle amendment

Worktree: `/Users/feral/mydev/bboxy450-harness-heartbeat`; branch `fix/state-heartbeat`.
The amendment supersedes the original 466e8bd report and its pre-rebase evidence.
`git fetch origin` and a signed `git rebase origin/main` completed without conflicts.
Final rebase basis: 0c403165e815c2f7bf77ccac293784b021f42f96, Standalone window chrome.
Only verification/README.md conflicted; the merged inventories give 98 models /
34 contracts. Both model families and their documentation are preserved. No push
or merge.

## Binding records read

- `d3ab423`: decision d5980, ACP ICD v0.2 and task #5981. Agent-origin work opens
  a TurnStarted with no UserMessage; Quiet ends it. The backend/protocol
  implementation belongs to the parallel #5981 Executor.
- `297415f`, `.bboxy/interfaces/embed-seat.md` p7: every open turn, including an
  agent-origin turn, owns a two-second heartbeat; Status changes publish at once;
  phase text is Thinking, Writing, the transcript tool head, or approval subject.
- `8a94218`: records the #348 phase 1c landing on term e8140d42.
- `.bboxy/interfaces/rich-pane.md` §5: socket readiness is admission; seat status
  is Reported through 7501/log/socket observations, independent of output cadence.

## Implementation

- `src/frontends/seat.rs:114` StateReporter: every TurnStarted enters the same open
  set through a `..` pattern. It does not inspect origin or require a UserMessage.
  Every non-error ending, including the additive Quiet reason, removes that turn.
  No new protocol/backend implementation is included here.
- `src/frontends/seat.rs:57` — the existing two-second deadline remains independent of phase updates and
  probes. Closed/idle turns have no heartbeat; delayed polls skip missed ticks.
- `src/frontends/seat.rs:250`, `:319` — explicit Thinking, Streaming (displayed as Writing), and Tool{item} statuses
  choose the live phase and emit immediately, without waiting for a timer.
  Late snapshots and newly started child items cannot replace an explicit
  Status phase. Native item-based inference applies before explicit selection
  and resumes after Retrying. Selected tool snapshots repair that head and publish changed detail.
- `src/frontends/seat.rs:335`, `:368` — seat_tool, tool_target and short_path are shared with the actual seat
  transcript. The reporter keeps its real verbs, including Write and Search,
  cwd-relative targets and the search location. Approval msg is its subject,
  with no added "Waiting for approval" label.
- The original ACP launcher prefix remains in msg; title stays profile · model.
  Redaction, control stripping, UTF-8-safe 192/2048 byte title/detail caps and
  the 256-byte error-message cap remain. The TTY-only event-loop timer and
  JSON/non-TTY exclusion in headless.rs are unchanged by this amendment.
- `tests/seat_golden.rs:497`, `:676`: immediate Status reports, stale tool snapshots, two
  heartbeat intervals through Writing/blocked, origin absent/user/agent wire
  shapes without UserMessage, and Quiet-to-idle behavior. Existing fixture
  constructors deserialize old wire shapes so the optional #5981 field will
  not require new Rust constructor fields in this test module.
- `tests/seat_process.rs`: the silent-tool witness still crosses five seconds,
  sees three periodic reports, then returns idle on its own child's SIGINT.
- `tests/formal/models.rs:2196` — HarnessSeatReport now has 13 finite variables: seat phase/report 0..3,
  profile/title 0..2 (A-B-A), probe/input flags, report age 0..2, heartbeat bit,
  origin 0..2 (none/user/agent) and work/reported phases 0..2
  (Thinking/Writing/Tool), an explicit-selection flag and one child flag.
  Origin starts share the timer. Work-phase changes
  are atomic with reporting and never advance the clock. Weak fairness covers
  the clock group and probe reply. The merged corpus is 98 variants / 34 contracts.
  No arbitrary Rust refinement or real-time scheduling proof is claimed.
- BrokenHarnessSeatReportPhase changes phase without reporting it; the
  implementation control status-stale-phase suppresses the Status selection.
  BrokenHarnessSeatReportChild and status-child-phase let a child replace an
  explicit phase and must fail.
  The existing no-heartbeat formal and real-PTY controls remain.
- ATTRIBUTION.md credits the project-owned addition. Formal inventories map the
  amended model, controls, assumptions and implementation witnesses. Generated
  formal archives stay repository-local and ignored; raw logs stay outside Git.

## Post-chrome foreground verification

Latest Oracle instruction: status-harness is accepted in principle (original
466e8bd make test 0); after Standalone window chrome lands on origin/main,
rebase and rerun seat and GUI tests in the foreground, then report the new hash.
Chrome landed as 0c40316 and is an ancestor of this branch. The signed rebase
completed; only the README inventory line conflicted.

All six commands in the private-HOME foreground runner exited 0:

- seat golden + process tests with acp,test-fixtures, excluding the separately
  run launch burst and descriptor-exposure fixture: 30 passed.
- seat_launch process cases: 7 passed.
- isolated descriptor-exposure process fixture: 1 passed.
- make test-gui: 99 passed, GUI clippy/dependency check green,
  offscreen Metal smoke PASS (21 visible items, 14 kinds, 3 approval buttons).
- dedicated gui_chrome tests: 7 passed (rim state, title-bar
  hit regions, zoom grid, layout reflow, zoom pill and anchored row).
- make test-formal-native: 22 Rust test functions passed;
  all 98 native models and registered controls exported, 34 contracts,
  one crate inventoried. Final signed-head Java parity is linked in the external
  handoff after the report commit.

The tested rebased commit before this metadata-only report amendment was
f964ebe911ed91ecfd32569fa86fb19d1244be47. StateReporter source is byte-identical to the previously verified
amendment; its three earlier mutation controls each failed with 101 and recovered
with 0. The rebase changes only the README conflict resolution and integrates
accepted Chrome code; no reporter implementation was changed for this rebase.

Current logs: /tmp/status-harness-chrome.xwkXdC/; verification.json records every command and actual exit.
Raw logs, screenshots and generated formal receipts remain outside Git. The one
compact .bboxy/runs/status-harness.json summary records current source/log hashes.
Tracked runtime artifacts were restored after preserving them in that log directory.

A full make test was not rerun after the amendment/rebase. Its original pre-amend
run exited 0; that is historical evidence, not a claim of a current full-suite
run. The latest requested post-chrome seat/GUI gates above are green. The earlier
machine-wide preflight was stopped before make; no waiting job remains.

## Integration boundary and ICD question

Origin/Quiet are still specified records on this rebase basis; their typed
protocol fields and actual autonomous ACP wake lifecycle are #5981's work.
The regression exercises their additive wire shapes against the reporter;
it does not claim that the current backend opens autonomous turns or retains
origin as typed metadata. The `..` start pattern and wildcard non-error ending
remain valid when those additions land. Term-side status precedence/readiness
and live seat_list integration are not changed or claimed as tested here.

The initial brief requires an ACP-agent prefix in msg; p7 enumerates phase
values without explicitly mentioning that prefix. This implementation retains
that original requirement and makes the phase suffix match p7. The remaining
ICD question is whether p7 should explicitly permit that prefix.

Tool detail uses what the live event actually supplies, through the same
formatter as the transcript. It never invents unavailable ACP rawInput/path
metadata. A real logged-in external ACP agent was not used for this frontend
amendment; that integration witness belongs to #5981.

## Handoff

The work and this report carry a Boxy signature and Report: [[status-harness]].
The final external handoff records the final hash, clean worktree, signed-head
formal receipt and command results. The requested post-chrome Executor handoff
is complete; the Oracle reviews and lands it. No push, merge, installation,
publication or deployment is performed.
