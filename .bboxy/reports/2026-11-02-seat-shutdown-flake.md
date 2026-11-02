---
id: "01M6FACPP0PSSDS69A2YZPBRC8"
number: 202611021403
type: report
title: "Seat shutdown flake: late forks and PTY output teardown"
created: "2026-11-02"
owner: "Codex Executor, Oracle Vesper Greywind's flake brief"
---

Worktree: `/Users/feral/mydev/bboxy450-harness-flake`; branch `fix/seat-shutdown-flake`;
base `df2ffe27161f22df9d521b68b7099369416ea836` after the p6/p7 rebase. Initial reproduction used `061c065de8cd57165fab6246991d6dca5eca0ba2`. Direct Oracle brief: reproduce the shutdown pair under parallel
GUI/ACP load, determine the cause, fix it, verify, sign, and stop without pushing or merging.

## Finding and mechanism

The original assertion already polled `kill(-pgid, 0)` for ESRCH for two seconds.
The failures included live sleeping processes. Reaping the seat at the failed
assertion and checking another ten times did not retire those groups. Extending
the assertion's deadline would hide an orphan, not repair an early sample.

Two product paths were exposed:

- A tool fixture publishes its leader PID immediately before `sleep 600` is forked.
  Under parallel process load a successful one-shot group SIGKILL can miss that
  in-flight fork. Temporary signal-safe tracing recorded successful SIGKILLs for
  groups 72409 and 72427 after SIGTERM; `ps` then showed a live orphan `sleep`
  72436 in PGID 72427 while its shell had exited. The trace and readiness order
  identify a fork/group-signal race, rather than an ignored SIGKILL or a zombie.
- PTY teardown can reach terminal I/O without delivering the expected signal.
  For seat 74092 the trace contained group registrations 74129/74144 and no
  shutdown signal or group-kill call; both groups remained live after escalation.
  The face awaited stdout, ignored flush errors, and treated write errors as an
  ordinary Close. Ordinary tool cancellation waits 500ms, longer than the
  supervisor's 250ms escalation. Physical EOF observation also depended on the
  face getting back to its input branch and a subsequent terminal poll.

`src/frontends/headless.rs:23` — `stop_cli` sends the existing synchronous group
kill and latches cancellation at the I/O observation. Byte-read errors use it
at :472; command/event, probe/echo and OSC report output failures
also use this hook. `src/frontends/headless.rs:35` — an outer 5ms
watch observes PTY hangup independently of output progress, then repeats group
kills while closing. Registration still fences which groups can be signalled;
the repeat reaches members published after the first signal.

`tests/support/seat_shutdown.rs:271` — the crash/recovery fixture now uses the
existing non-PTY descriptor fence. `portable-pty` sets CLOEXEC after openpty;
this spawn must not inherit a sibling's briefly exposed master and suppress its
hangup. The pre-existing fence witness remains in `tests/seat_process.rs`.

`tests/support/seat_shutdown.rs:208` — two new real-PTY cases produce a reply
larger than Tokio's 2MiB output buffer and the terminal queue, observe its prefix,
then stop draining. A non-controlling PTY exercises EOF without a kernel HUP;
the other case sends explicit SIGHUP. Both retain actual group death and durable
resume/check_transcript assertions. The first 40KB candidate was insufficient:
its reply fit inside Tokio's buffer, and its broken control passed; it was replaced.

The 100ms synchronous-kill contract, 250ms supervisor escalation, two-second
ESRCH deadline, and existing Ctrl-D/recovery assertions are retained. No timeout
was raised. The initial implementation left `src/process.rs` unchanged (SHA256
`f949667909428f78aa3ceb26c3dbe5f768ab65b52d32675ceef8717dc02d033f`); temporary diagnostic instrumentation was removed.
`ATTRIBUTION.md` clarifies that the existing header also accompanies extracted
test fixtures, executable models, regression witnesses and broken controls.

## States and finite model

`tests/formal/models.rs:891` — `HarnessSeatShutdown` names leader/member states
Alive → Signalled → Exited/waitable → Reaped, a bounded pending fork, shutdown
source (signal/physical EOF/I/O error), blocked output, closing and escalation
phase. A pending fork may publish an Alive member after the initial signal;
the next watch signals it before bounded kernel exit and escalation. Exit and
reaping are distinct: kill(0)/ESRCH remains a real process witness.

`tests/formal_nonvacuity.rs:619` — requires reachable late publication while
closing, blocked physical EOF, blocked I/O failure, escalation and reaping.
`BrokenHarnessSeatShutdownSingleKill` violates
`NoExecutableMembersAfterSupervisorKill`; `BrokenHarnessSeatShutdownBlockedOutput`
violates `ShutdownObservationIndependentOfOutput`. Corpus, crate inventory and
source mapping are updated in `verification/{corpus,coverage,contracts}.toml`.

The bounded schedule assumes one pending fork settles before a second watch
opportunity and signalled processes exit before escalation. The new model
coarsens kernel exits to the final settling phase; independent exit/ownership
interleavings remain in `HarnessProcessLifecycle`. Weak fairness supplies
progress, not a real-time guarantee. Arbitrary fork trees, OS scheduling and
PID/reparent/reap effects remain implementation coverage. The canonical stack
exports repository-local TLA+ and compares native/reference graphs and verdicts;
Java is still the reference, with no product-runtime Java dependency.

## Reproduction and rates

Binaries were built with `cargo test --offline --locked --features
standalone,acp,test-fixtures --test seat_process --test gui_render --test acp_agent
--no-run`. Each stress run uses a private HOME; the GUI and ACP binaries loop
concurrently, with their exit codes recorded. No busy-loop surrogate is used.

`python3 target/flake/stress.py before 30` — unmodified pair only, 30/30 loops
passed, 60/60 tests; 35 GUI/ACP load runs passed. This did not reproduce.

`python3 target/flake/stress.py before-suite 30 suite` — unmodified seat binary
also runs its sibling fixtures, excluding only the canonical `seat_launch::`
and deliberately inheritable-master case. Six of 30 suite iterations failed:
HUP twice, PTY close twice, TERM twice. The requested pair therefore failed
4/30 iterations (4/60 cases); all 145 GUI/ACP load runs passed.

`python3 target/flake/stress.py after-io 30 suite` — final implementation, same
concurrent fixture context plus the two stronger regressions: all shutdown cases
passed in 30/30 iterations (240/240, including 60/60 requested pair cases), and
all 163 GUI/ACP load runs passed. One unrelated `hangup_stops_without_an_operator`
fixture failed before its first input at `tests/seat_process.rs:70` with EIO:
the entire seat batch was 329 passed / 1 failed, not an entirely green batch.
The unmodified diagnostic batch also recorded startup failures in this fixture
(:88 Disconnected) and `no_bboxy_sock_still_finishes_a_turn`; they are retained
as separate evidence and were not relaxed or skipped.

`python3 target/flake/stress.py after-pair 30` — final pair only, 30/30 green
loops, 60/60 cases; all 33 GUI/ACP load runs passed. A watch-only intermediate
candidate passed a lighter batch but failed two PTY cases under the larger
regression; direct I/O teardown handling and the crash-spawn fence are included
in the final implementation above.

Raw logs, process samples, diagnostic sources, compiler output and the stress
runner remain in `target/flake/`, outside Git. Stress summaries record the
precommit base HEAD and actual fixture hashes; they do not claim the after
runs used pristine base source. `.bboxy/runs/flake.json` is the one small committed
summary, with current source hashes and command exits.

## Initial validation before the p6/p7 rebase

`target/flake/final-verify.json` — all commands exited 0:

- `cargo fmt --check`.
- `cargo test --offline --locked --features standalone,acp,test-fixtures --test seat_process seat_shutdown:: -- --nocapture` — 8 passed, 0 failed.
- `cargo test --offline --locked --test formal_nonvacuity shutdown_watch_covers_a_late_fork_and_output_blocked_physical_eof -- --exact` — 1 passed.
- `python3 -I scripts/test-e1c-rework-controls.py --case seat-shutdown-watch --case seat-hangup --case seat-hangup-model --out target/flake/final-controls` — all three deliberate mutations exit 101 at their intended assertion; restored sources exit 0. The new control removes the common shutdown observation hook, including the blocked-output watch/error path.
- `make test-formal` — native 84 models, one crate inventoried; broken controls rejected.
- `cargo clippy --offline --locked --all-features --all-targets -- -D warnings`.

Header, inventory-shape and diff-whitespace checks also exited 0.

The original signed commit `3e5f65b82b35d7839dcbe92b0254d559c67341e5` subsequently passed one full `make test`
(exit 0, 392.1s), including native/Java TLC parity and the GUI/packaging targets.
Its actual result is `target/flake/make-test.json`. Source/model/tool freshness
and the retained archive also verified with exit 0. These are historical
results; they do not substitute for the fresh post-rebase gate below.

## Final rebase validation on df2ffe2

Oracle continuation: rebase, preserve p6 reporting and p7, repeat focused/stress/full
checks, and leave the separate startup flake alone. The final tested base is
`df2ffe27161f22df9d521b68b7099369416ea836` (p6, p7 and completed log IR 1.1). The pre-report code commit is
`a48ced3b7a5f0d9fa0bb0ac59b3424a18185a06b`; final source hashes are `target/flake/tip/source-inputs.json`.

P6's raw input/filtering, noncanonical setup, probes, echo, event-driven reports,
report eligibility and profile-switch hydration remain. Main's `wire_bytes()` and
`check_authored` logic remain. Terminal read errors latch shutdown immediately;
command/event/probe/echo/report output errors take the same teardown hook. The
outer event-loop label leaves the complete face on an I/O failure. The branch
does not modify p7/import/protocol code or the startup fixtures relative to this
base. Main independently added the same crash-fixture fd fence: the rebase keeps
its expanded command-builder form. The report number avoids p6's record number.

`target/flake/tip/focused.json` — formatting and all-features/all-targets Clippy
exit 0; shutdown tests 8/8; p6 `tty_seat_` and `state_reports` tests 4/4.
`FLAKE_BINARIES=target/flake/tip/binaries.json python3 target/flake/stress.py tip-stress 30 suite`
— **30/30 green**, **450/450 seat cases**, including the requested pair 60/60,
all shutdown witnesses 240/240 and all p6 cases 120/120. Concurrent GUI/ACP:
192 runs, zero failures. `target/flake/tip/controls.log` — removing the hook
fails the intended assertion (101); restored final source passes (0).

`hangup_stops_without_an_operator` / `no_bboxy_sock_still_finishes_a_turn` remain
the explicitly deferred startup follow-up. They were not changed or skipped.
An earlier rebased stress batch recorded one no-socket Disconnected failure;
the latest 30-run batch is green, which does not prove that flake eliminated.

Intermediate signed snapshot a366f0f passed a complete full gate and 93-model
native/Java parity (450.2s) before main advanced. Read-only integration review
found only equivalent fd-fence forms in the crash fixture, so this final rebase
keeps main's form and refreshes all checks on df2ffe2. The older Clippy failure
and stale shared-sccache TMPDIR failure are retained in `target/flake/rebase2/`
and `target/flake/rebase3/cache-failure/`. Clippy's equivalent let-chain is clean.
Verification now sets `RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER=` only in the private
run environment; no shared daemon or global compiler configuration is touched.

One full `make test` runs on the final signed commit. Its actual result is
`target/flake/tip/make-test.json`; the final handoff is
`/Users/feral/mydev/bboxy450-harness-parity/reports/flake.md`. This committed report
makes no advance claim about that postcommit outcome. The run refreshes all
93 configured models and source/model/tool hashes; complete formal evidence
is retained outside build cleanup and bulk outputs stay outside Git.

No push, merge, publication, installation, credential change, operator desktop
interaction or peer worktree mutation is part of this assignment.
