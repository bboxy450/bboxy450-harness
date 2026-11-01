---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- Makefile
- .bboxy/runs/
artifacts:
- '[[01M6BPA5EGP0SD3RMAQX5D9V1M]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T13:11:19.178Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BKRR5BAYBDJHJHS43SZ9QY
number: 4340
provenance:
  digest: sha256:7742a1dee1d16c613bb8d5b13a3d2ab0e9e9366eb125796cb56b7bce8a1cb766
  frontmatter_digest: sha256:66790771c86495ca95270934d7cf9f912f420ed9f0e546fb298b6383e3ffcb10
  log: .bboxy/log/2026-11-01.jsonl#28873
  sig: 4XUFHQMbD3lM+jkYUPAFN4PoMs45esU9r7rXk62Z7gNA27PBTrRkokI7r0QthA2RFpSllcy1qquFVY3v5uHrAA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: 1fbe6413c1a9492372ef484c3d2a2bd3e5c98c77
  op_id: 01M6BPDJ8EB9F1E9KCBQQVGKDH
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: |-
    E1e #4340 complete on e1e-seat-hangup-kill. PR #13 ready: https://github.com/bboxy450/bboxy450-harness/pull/13 . Signed head1fbe6413c1a9492372ef484c3d2a2bd3e5c98c77; both commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
    Full committed report: .bboxy/runs/2026-11-01-seat-hangup-kill-report.log; Report: [[2026-11-01-seat-hangup-kill-report]]. Raw signal handlers dispatch SIGKILL to registered tool/terminal/MCP groups before notifying async cleanup. Spawn publication and pre-reap retirement are fenced. Healthy Ctrl-D stays graceful. Resume replaces only the stale owned socket after the exclusive writer lease.
    Exactly make test=0 unattended on f622dfe, rebased onto main including security PR11/ec7b776. All135 recorded inputs match final records head. SIGHUP, SIGTERM and actual PTY-master-close each use the HUP+TERM-ignoring tool per your 01M6BNHC discriminator correction, plus a real MCP group/member; all groups ESRCH within2s, killed exit recorded (not asserted), resume gives SessionRecovered and check_transcript. Ctrl-D exits0 within2s and log ends SessionClosed. pty-cases.log records all three Killed:9 observations and four passing cases.
    All four controls refreshed on the integrated source: seat-hangup, seat-hangup-model, leader-only and group-kill each101→0 at their named assertion. Every control source hash matches. HarnessProcessLifecycle now reaches supervisor death and checks NoOwnedMemberAfterSupervisorKill over actual member/seat state; broken control native/TLC safety, TLC12, graph45/82; correct35/65 and TLC0.
    Final-head make test-formal-parity=0 and make check-formal-evidence=0:65 model variants/25 contracts, clean receipt2d158e6de6a00352aece22a8cdcfb107d40d9cb69d1bbb8177fc9ae8279e5b91. Full gate log untruncated;13 canonical layers plus GUI/smoke all0. Finite-model/kernel timing limits are explicit; no assigned gap. Tracked tree clean, local current-head proof archives retained. No GUI source/wire changes, live-provider request or credential value.
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1e: synchronous SIGKILL of owned process groups on SIGHUP/SIGTERM/PTY EOF'
type: task
updated: 2026-11-01T14:07:03.702Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Synchronous owned-group SIGKILL before any async signal/EOF cleanup; ownership/spawn/reap fences
    reference: src/process.rs::reserve_group/kill_owned_groups/shutdown_signal/install_shutdown_handlers/OwnedGroup::reaped; src/mcp.rs::supervise/finish_child; headless.rs EOF poll/termios path; ownership/slot-reuse unit tests; complete make-test.log
  - criterion: Real PTY supervisor escalation, all groups dead, observed exit, durable resume
    reference: 'tests/seat_process.rs + tests/support/seat_shutdown.rs::supervisor_case; pty-cases.log: HUP/TERM/master-close all pass, three observed Killed:9 exits, tool ignores HUP+TERM, fixture MCP member, ESRCH2s, SessionRecovered/check_transcript; stale socket product fix after Host::open'
  - criterion: Ctrl-D remains graceful, exit0 within2s, SessionClosed/fsync and no remaining group
    reference: seat_shutdown::control_d_at_the_prompt_closes_and_fsyncs_gracefully; all three resumed Ctrl-D closures; pty-cases.log fourpass; headless final seat exit status
  - criterion: New controls101→0 and existing MCP cleanup controls retained
    reference: .bboxy/runs/seat-hangup-kill/controls.json and eight red/green logs; exact script command in report; seat-hangup orphan assertion, seat-hangup-model named invariant, leader-only orphaned-member assertion, group-kill Close join guard; all source hashes match
  - criterion: State-based model, broken supervisor-before-group-kill variant, full native/TLC parity
    reference: HarnessProcessLifecycle and BrokenHarnessProcessHangup; tests/formal_nonvacuity.rs::synchronous_hangup_kill_precedes_the_supervisors_unjoinable_exit; verification/contracts,corpus,coverage.toml; formal-code.json correct35states/65edges/TLC0, broken45/82/TLC12;65 variants agree
  - criterion: Exact unattended canonical gate, signed pushed head and committed report
    reference: '.bboxy/runs/seat-hangup-kill/make-test.log, test-current.json, gui-gate.json: make test0;13 canonical child commands0, GUI/smoke0,135 input hashes unchanged at1fbe641; signed f622dfe +1fbe641; .bboxy/runs/2026-11-01-seat-hangup-kill-report.log; final-head receipt2d158e6de6a00352aece22a8cdcfb107d40d9cb69d1bbb8177fc9ae8279e5b91'
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main, after PR #8 has merged, and open a PR. Do not merge. Binding: `.bboxy/interfaces/embed-seat.md` §3/§6/§7 at main 42e7a20 or later. This is the delta I sent while #4069 was in flight; PR #8 does not implement it.

Requirement (§3):
- On SIGHUP, SIGTERM or PTY EOF, the seat SIGKILLs every process group it owns: tools, terminals and MCP servers.
- It does this synchronously, within 100 ms, before any await. Only after that may the graceful path run.
- Reason: bbt-supervisor escalates SIGHUP to SIGKILL after 250 ms. Tool groups are outside the seat's own process group, so a tool that ignores HUP would be orphaned.
- Ctrl-D and Op::Close stay graceful: SessionClosed, fsync, exit 0 within 2 s.

Tests (§6, tests/seat_process.rs, real PTY):
1. A tool runs `trap '' HUP; sleep 600`. Send SIGHUP, then SIGKILL the harness after 250 ms. Every owned group must reach ESRCH within 2 s. Record the harness exit; do not assert it. A later `seat --resume <ulid>` must recover, and check_transcript must pass.
2. Ctrl-D: exit 0 within 2 s, and the log ends with SessionClosed.
3. A broken control: remove the synchronous kill. Case 1 must go 101 → 0 at the orphaned-group assertion.

If the signal-to-kill ordering is a stateful contract (HarnessEmbedLifecycle or HarnessProcessLifecycle), update that model. Add a broken variant in which the supervisor's KILL arrives before the group kill.

Acceptance:
- `make test` exits 0 unattended, and native/TLC parity holds.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-seat-hangup-kill-report.log`.
- Send one task_report with verify_result.command exactly `make test`.

E4 (bboxy450-term) depends on this.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1e #4340 complete on e1e-seat-hangup-kill. PR #13 ready: https://github.com/bboxy450/bboxy450-harness/pull/13 . Signed head1fbe6413c1a9492372ef484c3d2a2bd3e5c98c77; both commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
Full committed report: .bboxy/runs/2026-11-01-seat-hangup-kill-report.log; Report: [[2026-11-01-seat-hangup-kill-report]]. Raw signal handlers dispatch SIGKILL to registered tool/terminal/MCP groups before notifying async cleanup. Spawn publication and pre-reap retirement are fenced. Healthy Ctrl-D stays graceful. Resume replaces only the stale owned socket after the exclusive writer lease.
Exactly make test=0 unattended on f622dfe, rebased onto main including security PR11/ec7b776. All135 recorded inputs match final records head. SIGHUP, SIGTERM and actual PTY-master-close each use the HUP+TERM-ignoring tool per your 01M6BNHC discriminator correction, plus a real MCP group/member; all groups ESRCH within2s, killed exit recorded (not asserted), resume gives SessionRecovered and check_transcript. Ctrl-D exits0 within2s and log ends SessionClosed. pty-cases.log records all three Killed:9 observations and four passing cases.
All four controls refreshed on the integrated source: seat-hangup, seat-hangup-model, leader-only and group-kill each101→0 at their named assertion. Every control source hash matches. HarnessProcessLifecycle now reaches supervisor death and checks NoOwnedMemberAfterSupervisorKill over actual member/seat state; broken control native/TLC safety, TLC12, graph45/82; correct35/65 and TLC0.
Final-head make test-formal-parity=0 and make check-formal-evidence=0:65 model variants/25 contracts, clean receipt2d158e6de6a00352aece22a8cdcfb107d40d9cb69d1bbb8177fc9ae8279e5b91. Full gate log untruncated;13 canonical layers plus GUI/smoke all0. Finite-model/kernel timing limits are explicit; no assigned gap. Tracked tree clean, local current-head proof archives retained. No GUI source/wire changes, live-provider request or credential value.

```json
{
  "commit": "1fbe6413c1a9492372ef484c3d2a2bd3e5c98c77",
  "op_id": "01M6BPDJ8EB9F1E9KCBQQVGKDH",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Synchronous owned-group SIGKILL before any async signal/EOF cleanup; ownership/spawn/reap fences",
        "reference": "src/process.rs::reserve_group/kill_owned_groups/shutdown_signal/install_shutdown_handlers/OwnedGroup::reaped; src/mcp.rs::supervise/finish_child; headless.rs EOF poll/termios path; ownership/slot-reuse unit tests; complete make-test.log"
      },
      {
        "criterion": "Real PTY supervisor escalation, all groups dead, observed exit, durable resume",
        "reference": "tests/seat_process.rs + tests/support/seat_shutdown.rs::supervisor_case; pty-cases.log: HUP/TERM/master-close all pass, three observed Killed:9 exits, tool ignores HUP+TERM, fixture MCP member, ESRCH2s, SessionRecovered/check_transcript; stale socket product fix after Host::open"
      },
      {
        "criterion": "Ctrl-D remains graceful, exit0 within2s, SessionClosed/fsync and no remaining group",
        "reference": "seat_shutdown::control_d_at_the_prompt_closes_and_fsyncs_gracefully; all three resumed Ctrl-D closures; pty-cases.log fourpass; headless final seat exit status"
      },
      {
        "criterion": "New controls101→0 and existing MCP cleanup controls retained",
        "reference": ".bboxy/runs/seat-hangup-kill/controls.json and eight red/green logs; exact script command in report; seat-hangup orphan assertion, seat-hangup-model named invariant, leader-only orphaned-member assertion, group-kill Close join guard; all source hashes match"
      },
      {
        "criterion": "State-based model, broken supervisor-before-group-kill variant, full native/TLC parity",
        "reference": "HarnessProcessLifecycle and BrokenHarnessProcessHangup; tests/formal_nonvacuity.rs::synchronous_hangup_kill_precedes_the_supervisors_unjoinable_exit; verification/contracts,corpus,coverage.toml; formal-code.json correct35states/65edges/TLC0, broken45/82/TLC12;65 variants agree"
      },
      {
        "criterion": "Exact unattended canonical gate, signed pushed head and committed report",
        "reference": ".bboxy/runs/seat-hangup-kill/make-test.log, test-current.json, gui-gate.json: make test0;13 canonical child commands0, GUI/smoke0,135 input hashes unchanged at1fbe641; signed f622dfe +1fbe641; .bboxy/runs/2026-11-01-seat-hangup-kill-report.log; final-head receipt2d158e6de6a00352aece22a8cdcfb107d40d9cb69d1bbb8177fc9ae8279e5b91"
      }
    ],
    "exit_code": 0
  }
}
```
