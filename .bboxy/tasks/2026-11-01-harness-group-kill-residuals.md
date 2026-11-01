---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
artifacts:
- '[[01M6BVZC1FQAZV9PFXRA8FEP6D]]'
- '[[01M6BWZ7K0Y4D03KAJB39AFY16]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T14:01:26.194Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BPMGVA3DAJ4SB1XPD02S5Z
number: 4589
provenance:
  digest: sha256:6441ef8f424a98f611b89b2c7d119b24712ed4ba0af0e40fd8711aa4ef66a173
  frontmatter_digest: sha256:7cbbdef6ad0ef3726bd6c3c2ed10818e1d6981e3685056a05f6e2424d7c3bad4
  log: .bboxy/log/2026-11-01.jsonl#31151
  sig: KLy3VojjFdgq2ziaYcoqpvuyx6ty4z4qb3N8diCCgVVoLHDHMBfk6Buu1oshmnhAP0MzvIkNCrTejNOmg+gQAg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Round 2. The core is good: the Oracle's review passed OwnedChild sole reaping, kill→retire→wait on every path including Drop and cancel, and the tokio orphan queue sits after the fence. Fix these on the same branch (PR #14):

    MUST:
    1. Latency regression, mcp.rs:489-492. finish_child now always sends SIGTERM and sleeps a fixed 1 s, even when the server has already exited. The old code skipped this. Every MCP close and restart now costs ≥1 s, half of the Ctrl-D 2 s exit budget. Fix: if the leader has already exited (observe it with WNOWAIT, process.rs:404), skip straight to the fenced kill/wait. Otherwise SIGTERM, then poll for exit up to 1 s (no fixed sleep), then the fenced KILL. Add a test: closing an already-exited server finishes in well under 1 s, asserted structurally (no TERM sent and no sleep taken), not as a wall-clock percentile.

    SHOULD:
    2. The missing-id witness is weak. The fixture reaps the child itself at mcp.rs:654-658 before finish_child, so members_dead comes from the fixture, not from finish_child. Make finish_child itself responsible for the live members on that path, or rename the test to what it actually proves.
    3. `registered_at_reap` (process.rs:243-248) is near-tautological, because it is computed after retire_for_reap. In the reap-lease model, `registered` (var 9) is read only by its own invariant. Make a signal action depend on `registered` so that the broken variant produces a foreign signal; that is behavioural, not a deleted assignment.
    4. Flake risk at seat_process.rs:209. Clearing CLOEXEC on a live master while other tests spawn can leak the fd into their children. Isolate that test: run it alone, as with the seat_launch split, or use a dedicated process.

    Then run exactly `make test`, refresh the controls and the report, and send task_report.
report:
  commit: 9e8bbe6808bbeb7683bfe662f7ab24947f76cf13
  op_id: 01M6BY0W8413EZQ17HS70QM4XK
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: |-
    Round 2 complete on e1g-group-kill-residuals; PR #14 remains OPEN, ready and MERGEABLE: https://github.com/bboxy450/bboxy450-harness/pull/14 . No merge. Signed head9e8bbe6808bbeb7683bfe662f7ab24947f76cf13; all seven task commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. Rebased onto bb5478b including PR16 default tests/preserve_order.
    MUST1: WNOWAIT skips TERM/grace for exited leaders; live leaders receive TERM and exit polling in ≤5ms requested sleeps up to1s, then the existing fenced KILL/retire/wait. A real exited-leader/live-member fixture requires finish_child to complete in one poll, with zero TERM/sleep entries and no survivor/foreign signal. The old flow is red101; separate TERM and direct uninstrumented-sleep controls each101→0.
    All SHOULD findings fixed: the missing-id test is honestly renamed already-fenced idempotence; registered_at_reap is removed and the actual handler CAS/dispatch is exercised after release; the model's registration now enables a real foreign signal (NoSignalToEmptyGroup, Java Inv2 after SignalEmptyGroup); CLOEXEC exposure is excluded from both concurrent suites and run alone.
    Exactly make test=0 unattended on signed sourceae2d7fd; all135 inputs match the final records head. Sixteen canonical children plus four GUI children exit0. All eleven refreshed controls show named101→0 with matching current source hashes. Full committed report: .bboxy/runs/2026-11-01-group-kill-residuals-report.log; current evidence under group-kill-residuals/round2/.
    Final-head make test-formal-parity=0 and make check-formal-evidence=0:67 variants/25 contracts, full graph/verdict parity, clean receipt613fe57aed56632dc5f6ad92df8d4be33777f489d8422cf804b7bc5b55a4c906. Reap-lease control now46states/92edges, TLC12 at an actual foreign signal; good35/65,TLC0. Tracked tree clean; current-head proof/logs retained. The previously reviewed exclusive-reaper precondition and deterministic PID-reuse-injection limit are explicit. No assigned finding waived or additional live-provider request.
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1g: close residual group-kill windows (ECHILD reuse, MCP finish_child error paths)'
type: task
updated: 2026-11-01T16:15:37.227Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: MUST1 exited-leader fast cleanup; no TERM or sleep, actual live member killed, no foreign signal
    reference: src/process.rs::OwnedChild::{exited,grace_until}; src/mcp.rs:485 + fixtures::finish_exited_leader; tests/mcp_client.rs:914; round2/exited-baseline-red.log101→exited-green.log0; controls/mcp-exited-{term,grace}-{red,green}.log each101→0; one-poll assertion detects direct uninstrumented tokio::sleep
  - criterion: SHOULD2 already-fenced idempotence accurately named; finish_child owns live-member cleanup in a separate real witness
    reference: tests/mcp_client.rs:861 renamed mcp_finish_already_fenced_child_is_idempotent_and_keeps_foreign_group_safe; actual child.idNone, zero TERM/sleep, safe foreign group; new exited-leader/member witness and existing real receiver-error witness; round2/controls/mcp-reap-ownership-{red,green}.log both named assertions101→0
  - criterion: SHOULD3 behavioral signal coupling in implementation and formal model
    reference: src/process.rs:97 shared signal_registered_group CAS/dispatch, post-kernel-wait probe; registered_at_reap removed; tests/formal/models.rs:940 SignalEmptyGroup reads registered; tests/formal_nonvacuity.rs reachable reaped+registered+foreign signal assertion; round2/model-controls/BrokenHarnessProcessReapLease.log Inv2 with v0=2 v1=0 v6=1 v9=1; native NoSignalToEmptyGroup; controls/reap-lease-model101→0; full graph46/92 native/TLC agree
  - criterion: SHOULD4 deliberate CLOEXEC exposure isolated without omitting the witness
    reference: scripts/test.py skips the case in default and feature all-targets suites, then runs cargo test --offline --locked --features acp,test-fixtures --test seat_process non_pty_spawn_does_not_keep_a_sibling_master_alive -- --exact; round2/make-test.log one passing case + test-current.json separate exit0; controls/pty-spawn-fds101→0
  - criterion: Exact unattended full gate and refreshed controls
    reference: .bboxy/runs/group-kill-residuals/round2/{make-test.log,test-current.json,gui-gate.json,controls/controls.json}; make test0 onae2d7fd04cc81265d14ed9615f29e5e175826fa3;16 canonical+4 GUI child exits0,135 input hashes match9e8bbe6,11 current controls101→0; default/feature/unit/process/e2e/GUI smoke and secret/header scans pass
  - criterion: Signed ready PR, committed full report and current-head formal receipt
    reference: PR14 head9e8bbe6808bbeb7683bfe662f7ab24947f76cf13; git verify-commit all7=0; report .bboxy/runs/2026-11-01-group-kill-residuals-report.log sha256:f0ff82ee983fa67996bc35c7c935af2a5bb59e219c992427ad6375f5c4bdcd69; final-head parity0 + strict evidence0,67variants/25contracts, verification/evidence/613fe57aed56632dc5f6ad92df8d4be33777f489d8422cf804b7bc5b55a4c906/evidence.json; no source dirt
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main (adf3e5d or later) and open a PR. Do not merge. These are the two low-severity residuals from the Oracle's review of PR #13.

1. **ECHILD path** (`src/process.rs:356`). When an external reaper has already collected the leader, `reaped()` runs after the PID is released. That leaves a window in which the signal handler can SIGKILL a reused pgid. Fix: on ECHILD, retire the slot so the handler can no longer target it before the release is observable, or document why the window can't occur. Prove it with a unit witness, as `released_registration_does_not_clear_a_reused_slot` does.
2. **MCP `finish_child`** (`src/mcp.rs:488-494`). If `child.id()` is None, or `rx` errors, members are unregistered or dropped without the fenced group kill, and `OwnedGroup::drop` can SIGKILL after the SDK reaped the leader. Fix: route both paths through the same fenced kill-then-retire order, or prove them unreachable with a typed invariant. Add a test that forces each path with the stubborn-group fixture: no member survives and no foreign pgid is signalled.

Broken controls: each fix gets one, and it must go 101 → 0 at its named assertion.

Acceptance:
- `make test` exits 0 unattended, and native/TLC parity holds.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-group-kill-residuals-report.log`.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1g complete on e1g-group-kill-residuals. PR #14 is OPEN, ready and MERGEABLE: https://github.com/bboxy450/bboxy450-harness/pull/14 . No merge. Signed head2c82b5a37ba57a029346b530791c011c022f9cb9; all five task commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. Full committed report: .bboxy/runs/2026-11-01-group-kill-residuals-report.log (Report: [[2026-11-01-group-kill-residuals-report]]).
OwnedChild privately owns all reaping; group KILL/registration retirement precede wait and Drop. SIGCHLD autoreaping is rejected. The documented invariant excludes arbitrary foreign unsafe reapers; ECHILD cannot retroactively fence such code. MCP SDK receives only pipes, and actual missing-id/receiver-error fixtures use stubborn members plus a disposable foreign group with explicit deterministic stale-signal injection.
Exactly make test=0 unattended on signed source54a426a; all135 gate input hashes match the final records head. Fourteen canonical children plus four GUI children exit0. Nine current-source controls each101→0, including both named foreign-group assertions. Twenty complete nine-case PTY suites pass180/180 on3ec647f; the later native unit EOF-readiness precondition is checked by the final gate. Original gate failures and deterministic descriptor/EOF corrections are committed, with no relaxed shutdown deadlines.
Final-head make test-formal-parity=0 and make check-formal-evidence=0:67 variants/25 contracts, full graphs and verdicts agree; clean receipt8b6426123b1fbb5d9adf72fe2ec5b5c64c9833513bdc121a1a6d1a0f6491f556. New reap-lease and lost-child models fail named safety properties under native and TLC (TLC12). Tracked worktree clean; current-head proof/log archives retained. No added live-provider request or real secret value.

```json
{
  "commit": "2c82b5a37ba57a029346b530791c011c022f9cb9",
  "op_id": "01M6BWHFNFTGTGK34TTDRA6551",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1 ECHILD reuse window excluded by exclusive wait ownership; actual registry retirement before release and named unit negative control",
        "reference": "src/process.rs:190/:196/:238/:272; verification/contracts.toml exclusive-reaper precondition; group-kill-residuals/{all-controls.json,reap-lease-red.log,reap-lease-green.log,sigchld-admission-red.log,sigchld-admission-green.log}; both controls101→0"
      },
      {
        "criterion": "2 Both MCP exceptional cleanup paths forced with stubborn groups; no surviving member or foreign signal; each path's named assertion goes101→0",
        "reference": "src/mcp.rs:450/:485/:640; tests/mcp_client.rs:861/:886; src/process.rs::fixtures; group-kill-residuals/mcp-reap-ownership-{red,green}.log + all-controls.json; real idNone and receiverErr, deterministic foreign-reuse injection documented in full report"
      },
      {
        "criterion": "State-based ownership/MCP models and new broken controls agree under native and pinned Java TLC",
        "reference": "tests/formal/models.rs; tests/formal_nonvacuity.rs:408; verification/{contracts,corpus,coverage}.toml; committed group-kill-residuals/{formal-code.json,model-controls/}; Process35/65 good and broken safetyNoRegistrationAfterReap; MCP26/47 good, lost23/39 safetyNoMemberAfterClose;67 variants/25 contracts"
      },
      {
        "criterion": "Exact unattended canonical acceptance and concrete PTY regression witnesses",
        "reference": ".bboxy/runs/group-kill-residuals/{make-test.log,test-current.json,gui-gate.json,all-controls.json,pty-20-final.log,pty-20-final.json,pty-failure-extract.log}; make test0 on54a426a,14+4 children0,135 inputs identical at2c82b5a;9 controls101→0,180 repeated PTY cases; initial failed gates retained"
      },
      {
        "criterion": "Signed pushed ready PR and full committed report; current-head strict proof receipt",
        "reference": "PR14 head2c82b5a37ba57a029346b530791c011c022f9cb9; git verify-commit all5=0; .bboxy/runs/2026-11-01-group-kill-residuals-report.log sha256:5afd555bb9dd7424b8ada3ce18be3da2036aec7465baf9a9e68805605067136e; make test-formal-parity0 + make check-formal-evidence0 on2c82b5a; verification/evidence/8b6426123b1fbb5d9adf72fe2ec5b5c64c9833513bdc121a1a6d1a0f6491f556/evidence.json; tracked source clean"
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

Round 2 complete on e1g-group-kill-residuals; PR #14 remains OPEN, ready and MERGEABLE: https://github.com/bboxy450/bboxy450-harness/pull/14 . No merge. Signed head9e8bbe6808bbeb7683bfe662f7ab24947f76cf13; all seven task commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. Rebased onto bb5478b including PR16 default tests/preserve_order.
MUST1: WNOWAIT skips TERM/grace for exited leaders; live leaders receive TERM and exit polling in ≤5ms requested sleeps up to1s, then the existing fenced KILL/retire/wait. A real exited-leader/live-member fixture requires finish_child to complete in one poll, with zero TERM/sleep entries and no survivor/foreign signal. The old flow is red101; separate TERM and direct uninstrumented-sleep controls each101→0.
All SHOULD findings fixed: the missing-id test is honestly renamed already-fenced idempotence; registered_at_reap is removed and the actual handler CAS/dispatch is exercised after release; the model's registration now enables a real foreign signal (NoSignalToEmptyGroup, Java Inv2 after SignalEmptyGroup); CLOEXEC exposure is excluded from both concurrent suites and run alone.
Exactly make test=0 unattended on signed sourceae2d7fd; all135 inputs match the final records head. Sixteen canonical children plus four GUI children exit0. All eleven refreshed controls show named101→0 with matching current source hashes. Full committed report: .bboxy/runs/2026-11-01-group-kill-residuals-report.log; current evidence under group-kill-residuals/round2/.
Final-head make test-formal-parity=0 and make check-formal-evidence=0:67 variants/25 contracts, full graph/verdict parity, clean receipt613fe57aed56632dc5f6ad92df8d4be33777f489d8422cf804b7bc5b55a4c906. Reap-lease control now46states/92edges, TLC12 at an actual foreign signal; good35/65,TLC0. Tracked tree clean; current-head proof/logs retained. The previously reviewed exclusive-reaper precondition and deterministic PID-reuse-injection limit are explicit. No assigned finding waived or additional live-provider request.

```json
{
  "commit": "9e8bbe6808bbeb7683bfe662f7ab24947f76cf13",
  "op_id": "01M6BY0W8413EZQ17HS70QM4XK",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "MUST1 exited-leader fast cleanup; no TERM or sleep, actual live member killed, no foreign signal",
        "reference": "src/process.rs::OwnedChild::{exited,grace_until}; src/mcp.rs:485 + fixtures::finish_exited_leader; tests/mcp_client.rs:914; round2/exited-baseline-red.log101→exited-green.log0; controls/mcp-exited-{term,grace}-{red,green}.log each101→0; one-poll assertion detects direct uninstrumented tokio::sleep"
      },
      {
        "criterion": "SHOULD2 already-fenced idempotence accurately named; finish_child owns live-member cleanup in a separate real witness",
        "reference": "tests/mcp_client.rs:861 renamed mcp_finish_already_fenced_child_is_idempotent_and_keeps_foreign_group_safe; actual child.idNone, zero TERM/sleep, safe foreign group; new exited-leader/member witness and existing real receiver-error witness; round2/controls/mcp-reap-ownership-{red,green}.log both named assertions101→0"
      },
      {
        "criterion": "SHOULD3 behavioral signal coupling in implementation and formal model",
        "reference": "src/process.rs:97 shared signal_registered_group CAS/dispatch, post-kernel-wait probe; registered_at_reap removed; tests/formal/models.rs:940 SignalEmptyGroup reads registered; tests/formal_nonvacuity.rs reachable reaped+registered+foreign signal assertion; round2/model-controls/BrokenHarnessProcessReapLease.log Inv2 with v0=2 v1=0 v6=1 v9=1; native NoSignalToEmptyGroup; controls/reap-lease-model101→0; full graph46/92 native/TLC agree"
      },
      {
        "criterion": "SHOULD4 deliberate CLOEXEC exposure isolated without omitting the witness",
        "reference": "scripts/test.py skips the case in default and feature all-targets suites, then runs cargo test --offline --locked --features acp,test-fixtures --test seat_process non_pty_spawn_does_not_keep_a_sibling_master_alive -- --exact; round2/make-test.log one passing case + test-current.json separate exit0; controls/pty-spawn-fds101→0"
      },
      {
        "criterion": "Exact unattended full gate and refreshed controls",
        "reference": ".bboxy/runs/group-kill-residuals/round2/{make-test.log,test-current.json,gui-gate.json,controls/controls.json}; make test0 onae2d7fd04cc81265d14ed9615f29e5e175826fa3;16 canonical+4 GUI child exits0,135 input hashes match9e8bbe6,11 current controls101→0; default/feature/unit/process/e2e/GUI smoke and secret/header scans pass"
      },
      {
        "criterion": "Signed ready PR, committed full report and current-head formal receipt",
        "reference": "PR14 head9e8bbe6808bbeb7683bfe662f7ab24947f76cf13; git verify-commit all7=0; report .bboxy/runs/2026-11-01-group-kill-residuals-report.log sha256:f0ff82ee983fa67996bc35c7c935af2a5bb59e219c992427ad6375f5c4bdcd69; final-head parity0 + strict evidence0,67variants/25contracts, verification/evidence/613fe57aed56632dc5f6ad92df8d4be33777f489d8422cf804b7bc5b55a4c906/evidence.json; no source dirt"
      }
    ],
    "exit_code": 0
  }
}
```
