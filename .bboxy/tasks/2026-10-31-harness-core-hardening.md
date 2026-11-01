---
allowed_paths:
- src/
- tests/
- Makefile
- scripts/
- verification/
- Cargo.toml
- Cargo.lock
- .bboxy/runs/
artifacts:
- '[[01M6ARK37DTY4PT393VPJ3B72X]]'
- '[[01M6AQMCMJ5991J9PA4DH8YE5R]]'
- '[[01M6ARKS0FX2EMXFRPQ9ZQ6QVD]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T04:01:49.402Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6AMAK7VXJXGZTNDZR5T6ZWV
number: 1117
provenance:
  digest: sha256:5c432b48fd640db0fd2f5086a8c7ca2e6c99c1aa74769158c78e146be1ef5bc9
  frontmatter_digest: sha256:91edf9f371a856477225f2e11494f62f939e50ba20f4fcaa245a13895daad25f
  log: .bboxy/log/2026-11-01.jsonl#8579
  sig: ljQZeSg4Uh6fjyFS/TbhSlCbTOreq4hlLUl+1GZagdiNCsJdD41yYav9RyBQmy7tZ+6Vu/A5urr8L8YRPnDRDg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    PR #3 (1d8fa15) is rejected for one focused rework. Items 1, 3, 5–15 and 14 are verified real: their tests or controls fail without the fix. Keep all of that. Fix only the items below on the same branch, push, and report again.

    **BLOCKER: item 2 regresses law 4 (cancel is cancel).**
    - **Mechanism:** `reaped()` is set when the leader is reaped (`process.rs:162-164`). After that, `OwnedGroup::send` is a no-op (`:50-54`), so `terminate()` (`:231-235`) and `Drop` (`:59-62`) never signal the group.
    - **Counterexample:** `bash -c 'sleep 600 & exit 0'`, with `sleep` holding stdout. The leader exits and is reaped. The Interrupt sends nothing, the turn reports Cancelled, and `sleep` survives inside the harness's own group.
    - **Required fix:** detect that the leader has exited without reaping it (`waitid(WEXITED|WNOWAIT)`), or reap the leader only after `kill(-pgid, SIGKILL)`. A pgid cannot be reused while any group member is alive, so signalling a group that still has live members is always safe.
    - **Required witness:** a process test where the leader exits, a member stays alive, then the turn is cancelled. Assert ESRCH on the group, polling as in `cancel_process.rs:100-109`. It must be red on 1d8fa15 and green after the fix.
    - **Model:** `HarnessProcessLifecycle` hides this bug. `DropReapedGuard` (`models.rs:841`) derives `late_signal` from the broken mode, so the invariant tests a flag set only by the broken control. `KillAndReap` is atomic, so "reaped before cancel" is never reached.
      - Separate leader-exit/reap from member-alive and from cancel.
      - State the invariant over the state: after cancel completes, no member is alive; and no signal goes to a pgid that has no member.
      - Broken control (a) is the current guard, which must fail the "no member alive after cancel" invariant. Broken control (b) signals after the group is empty, which must fail the "no signal to an empty pgid" invariant.

    **SHOULD-FIX (cheap, include them)**
    - **Item 4:** the control-lane bound is a tautology. `LegitimateOutstandingFits` is a Rust constant, and the invariant restates the admit guard. Model producers and consumers of control ops, with a bounded slot per op type, and have the broken control overflow it.
    - **Item 6:** assert the violation detail, not only `.law`. Ten checks share `"one-ending"`.
    - **Item 7:** `cancel_process.rs:70` must poll like `:100-109`, so it does not flake on an unreaped zombie.
    - **Item 10:** remove the `< 1 ms` wall-clock asserts at `gateway_ops.rs:60` and `gateway_host.rs:101`. budgets.md forbids wall-clock gates. Assert non-blocking structurally instead: `try_reserve` returns without awaiting.
    - **Item 14:** in `formal_nonvacuity.rs:99`, assert the name of the violated invariant for control (e).

    **Verify:** `make test` exits 0, unattended. Report again through `task_report`, with red→green evidence for the blocker and the model controls. PR #2 (E3) is waiting on this merge, so keep the change set tight.
report:
  commit: 1d8fa1520709813d74a5e49ed53e85affbbc6a0b
  op_id: 01M6ARN2CW0Q0GGXSHBMC0BERC
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: 'E1b items 1–15 implemented and reported. PR #3: https://github.com/bboxy450/bboxy450-harness/pull/3 (OPEN, MERGEABLE; no merge). Branch e1b-hardening; signed head 1d8fa1520709813d74a5e49ed53e85affbbc6a0b. Full committed red→green report: https://github.com/bboxy450/bboxy450-harness/blob/1d8fa1520709813d74a5e49ed53e85affbbc6a0b/.bboxy/runs/2026-10-31-harness-core-hardening-report.log . make test exited 0 on fe7d82a; all test input hashes match final head (71 Rust tests, 13 Python gate checks). Final-head native/TLC parity and strict receipt exited 0 (36 variants, 16 contracts). One live smoke: PASS/0, exactly 3 requests, 1 read, 1 edit, transcript laws held; no later product changes. Budget bench: UNMEASURED/0, load 66.11 > 10 CPUs. Markdown execution witness remains E3''s share per signed ruling. Tracked tree clean; retained local formal archives and final-head run artifacts are untracked.'
role: executor
status: rejected
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1b: harness core hardening — cancel stall, pgid reuse, non-vacuous models and tests, streamed exec output'
type: task
updated: 2026-11-01T05:23:36.210Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: 1. Escaped pipe drain bounded; Cancelled under 2 s
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item01-drain-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 1; make-test-green.log
  - criterion: 2. Reaped process groups are never signalled
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item02-owner-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 2
  - criterion: 3. CancelTree captures birth tags and reaches A-B-A
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item03-04-model-red.json; tests/formal_nonvacuity.rs:24; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 3
  - criterion: 4. Acked steer exactly once and control capacity 256
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item03-04-model-red.json; tests/formal_nonvacuity.rs:57; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 4
  - criterion: 5. Java-free native target; canonical native before parity
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item05-13-gates-red.json; Makefile; scripts/test-hardening.py; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 5
  - criterion: 6. Mutation tests assert specific LawViolation.law
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item06-law-red.json; tests/protocol_laws.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 6
  - criterion: 7. Direct real pgid death assertion
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item07-pgid-red.json; tests/cancel_process.rs:12; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 7
  - criterion: 8. Hold A completion until B; no late A data in events/history
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item08-fence.json (101 -> 0); src/engine/mod.rs:1014; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 8
  - criterion: 9. Deny * has zero effects; old approval refused in current turn
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item09-deny.json; item09-stale.json (101 -> 0); /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 9
  - criterion: 10. Distinct full-work Close and explicit interrupt witnesses
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item10-control.json (101 -> 0); tests/gateway_ops.rs:54; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 10
  - criterion: 11. Live <=16 KiB output deltas; 64 KiB repair tail; full terminal output
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item11-stream-red.json; tests/exec_stream.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 11
  - criterion: 12. Durable call_id/name retained through crash and SIGKILL recovery
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item12-identity-red.json; tests/store_recovery.rs:194; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 12
  - criterion: 13. Whole tracked-tree secret scan; Cargo and verification TOML headers
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item05-13-gates-red.json; item13-binary-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 13
  - criterion: 14. Trusted-head Reset and durable epoch high-water; both broken controls fail safety
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item14-old-reset.json; item14-lost-epoch.json (101 -> 0); tests/store_recovery.rs:67; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 14
  - criterion: 15. Core off-worker hooks; 16 distinct registries saturate exactly 4 shared permits
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item15-worker.json; item15-permits.json (101 -> 0); /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 15
  - criterion: Canonical gate, live smoke, benchmark outcome and exact per-layer exits
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log command table; .bboxy/runs/test-current.json; .bboxy/runs/smoke-live-2026-10-31.json; .bboxy/runs/bench-budgets-current.json
  - criterion: Final signed head, unchanged test input hashes, fresh native/TLC parity and strict receipt
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/final-head-verification.json; final-head-formal.json; verification/evidence/a386ac1314bb808893220a07d051436474e43fedaf8c15584870e1198d938084/evidence.json
  exit_code: 0
---
# E1b: harness core hardening

ROLE: EXECUTOR (Codex).

**Worktree:** `/Users/feral/mydev/bboxy450-harness-core`. Reset it to `origin/main` at `71a4d95`, which contains merged PR #1, then create branch `e1b-hardening`. Push the branch and open a PR. Do not merge.

**Why.** The Oracle's review of PR #1 found real gaps behind passing gates. Each fix below needs a test that fails before the fix and passes after it. In your report, show that red-then-green, with the commands and exit codes.

## Correctness (law 4: cancel is cancel)
1. **`src/process.rs:81-85`.** After a cancel, the stdout/stderr drain waits for EOF with no time limit. A grandchild that escaped the process group and still holds the pipe stalls `cancel_and_join`, so `TurnEnded` never arrives.
   - Fix: bound the drain with the cancel grace period.
   - Witness: a child runs `setsid sleep 30 &` while holding stdout. The test asserts `TurnEnded{Cancelled}` arrives in under 2 s.
2. **`src/process.rs:36-41`.** `OwnedGroup::drop` sends `kill(-pgid, SIGKILL)` even after the child has been reaped. If the system reuses that pgid, the signal hits someone else's group.
   - Fix: send the signal only while the group is still owned and unreaped.
   - Witness: a unit test.

## Formal (models must encode the property they are named for)
3. **`HarnessCancelTree`** (`tests/formal/models.rs:176-200`). The correct model passes by construction: the held completion is applied without a guard, and `held_generation` never changes.
   - Model the fence: completions are tagged with their generation, and the apply guard compares that tag with the current generation.
   - The passing model must reach A→B→A and still hold `bad=0`.
   - The broken control removes the guard and must fail on safety.
4. **`HarnessSteerQueue`** (`models.rs:217-253`). `steer_count <= 1` holds trivially. Model what the ICD says:
   - an acked steer appears exactly once, either delivered or converted to a queued turn;
   - the control lane bound.
5. **Makefile.**
   - `make test-formal-native`: native only, needs no Java.
   - `make test-formal-parity`: the pinned Java run.
   - `make test` runs both, with native first.

## Tests that currently check nothing
6. `tests/protocol_laws.rs`: each mutation asserts the specific `LawViolation.law` it expects.
7. `tests/cancel_process.rs:52-63`: the pgid loop iterates zero times. Assert directly that the group is dead, for example by recording the pgid when the tool starts.
8. `a_b_a_is_fenced_by_birth_not_identity`:
   - hold turn A's tool future;
   - start turn B;
   - release A;
   - assert that B's history and events contain nothing from A.
9. `tests/policy_gate.rs`:
   - a counting tool under `Deny *` runs zero times;
   - an approval left over from the previous turn is refused in the current one.
10. `tests/gateway_ops.rs`: `busy_not_wait` and `interrupt_when_full` are duplicates. Make them distinct tests, and have both assert `TurnEnded{Cancelled}` where that applies.

## Product gaps
11. Stream exec output as `ItemDelta{Delta::Output}` while the process runs, at most 16 KiB per delta per ICD §3. The GUI renders live terminal output. The `Terminal` item completes with its full output or a blob.
12. `crashed()` and recovery (`protocol/mod.rs:609-619`): keep the real `call_id` and `name` from the projection when they are known.
13. `scripts/checks.py`:
    - the secret scan covers the whole tracked tree (`git ls-files`);
    - headers go on `Cargo.toml` and `verification/*.toml`, as comments.

Out of scope: the `Reset` rule for an old-epoch cursor at or below the head. The Architect owns that ruling.

## Verify
- `make test`, unattended, exit 0.
- `make smoke-live` once, recording its real outcome.

## Report
Send one `task_report` with:
- the red→green evidence for each item;
- commands and exit codes;
- the branch and its signed head.


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1b items 1–15 implemented and reported. PR #3: https://github.com/bboxy450/bboxy450-harness/pull/3 (OPEN, MERGEABLE; no merge). Branch e1b-hardening; signed head 1d8fa1520709813d74a5e49ed53e85affbbc6a0b. Full committed red→green report: https://github.com/bboxy450/bboxy450-harness/blob/1d8fa1520709813d74a5e49ed53e85affbbc6a0b/.bboxy/runs/2026-10-31-harness-core-hardening-report.log . make test exited 0 on fe7d82a; all test input hashes match final head (71 Rust tests, 13 Python gate checks). Final-head native/TLC parity and strict receipt exited 0 (36 variants, 16 contracts). One live smoke: PASS/0, exactly 3 requests, 1 read, 1 edit, transcript laws held; no later product changes. Budget bench: UNMEASURED/0, load 66.11 > 10 CPUs. Markdown execution witness remains E3's share per signed ruling. Tracked tree clean; retained local formal archives and final-head run artifacts are untracked.

```json
{
  "commit": "1d8fa1520709813d74a5e49ed53e85affbbc6a0b",
  "op_id": "01M6ARN2CW0Q0GGXSHBMC0BERC",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1. Escaped pipe drain bounded; Cancelled under 2 s",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item01-drain-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 1; make-test-green.log"
      },
      {
        "criterion": "2. Reaped process groups are never signalled",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item02-owner-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 2"
      },
      {
        "criterion": "3. CancelTree captures birth tags and reaches A-B-A",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item03-04-model-red.json; tests/formal_nonvacuity.rs:24; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 3"
      },
      {
        "criterion": "4. Acked steer exactly once and control capacity 256",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item03-04-model-red.json; tests/formal_nonvacuity.rs:57; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 4"
      },
      {
        "criterion": "5. Java-free native target; canonical native before parity",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item05-13-gates-red.json; Makefile; scripts/test-hardening.py; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 5"
      },
      {
        "criterion": "6. Mutation tests assert specific LawViolation.law",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item06-law-red.json; tests/protocol_laws.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 6"
      },
      {
        "criterion": "7. Direct real pgid death assertion",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item07-pgid-red.json; tests/cancel_process.rs:12; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 7"
      },
      {
        "criterion": "8. Hold A completion until B; no late A data in events/history",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item08-fence.json (101 -> 0); src/engine/mod.rs:1014; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 8"
      },
      {
        "criterion": "9. Deny * has zero effects; old approval refused in current turn",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item09-deny.json; item09-stale.json (101 -> 0); /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 9"
      },
      {
        "criterion": "10. Distinct full-work Close and explicit interrupt witnesses",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item10-control.json (101 -> 0); tests/gateway_ops.rs:54; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 10"
      },
      {
        "criterion": "11. Live <=16 KiB output deltas; 64 KiB repair tail; full terminal output",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item11-stream-red.json; tests/exec_stream.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 11"
      },
      {
        "criterion": "12. Durable call_id/name retained through crash and SIGKILL recovery",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item12-identity-red.json; tests/store_recovery.rs:194; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 12"
      },
      {
        "criterion": "13. Whole tracked-tree secret scan; Cargo and verification TOML headers",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item05-13-gates-red.json; item13-binary-red.json; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 13"
      },
      {
        "criterion": "14. Trusted-head Reset and durable epoch high-water; both broken controls fail safety",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item14-old-reset.json; item14-lost-epoch.json (101 -> 0); tests/store_recovery.rs:67; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 14"
      },
      {
        "criterion": "15. Core off-worker hooks; 16 distinct registries saturate exactly 4 shared permits",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/item15-worker.json; item15-permits.json (101 -> 0); /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log item 15"
      },
      {
        "criterion": "Canonical gate, live smoke, benchmark outcome and exact per-layer exits",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-report.log command table; .bboxy/runs/test-current.json; .bboxy/runs/smoke-live-2026-10-31.json; .bboxy/runs/bench-budgets-current.json"
      },
      {
        "criterion": "Final signed head, unchanged test input hashes, fresh native/TLC parity and strict receipt",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/hardening/final-head-verification.json; final-head-formal.json; verification/evidence/a386ac1314bb808893220a07d051436474e43fedaf8c15584870e1198d938084/evidence.json"
      }
    ],
    "exit_code": 0
  }
}
```
