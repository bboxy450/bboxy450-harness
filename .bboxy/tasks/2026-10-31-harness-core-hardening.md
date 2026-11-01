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
- '[[01M6ASAE4HAYQ8C7MWNYMVBWGD]]'
- '[[01M6AT4J1JB3FSQY4MRPZEY0AV]]'
- '[[01M6ATWSYJ84KZ8AC2FPAC8K6Y]]'
- '[[01M6ATXEE09EQ9A2VRX6FHS8EG]]'
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
  digest: sha256:2aa77ce488d142dcb424eca1f41dc0fa27ff1ab893d5c1c1ab0a984555bb0d31
  frontmatter_digest: sha256:aa62b0747864600635590e830b88b7bbae9b466e3d91a15277b10c0f9142c6e7
  log: .bboxy/log/2026-11-01.jsonl#11256
  sig: YMakSNaV5TVH2oUn2/RfmcbLyiknJVcoP/W+eUNu/VQ/L8I12OjR3caRRY7qwjVQb5jmCnFgSS6vcOOTDYITDA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Nearly there. The rework itself is verified:
    - **Diff review:** the WNOWAIT leader-exit handling and the group SIGKILL before reap are correct.
    - **smoke-live:** PASS on the main+PR#3 merge, with no key-shaped strings in the logs.

    One blocker remains. `make test` on main+PR#3 exited 2 at `tests/gateway_ops.rs:61` `busy_not_wait`: `left: Ok(Ack)`, `right: Err(Busy)`.
    - **Reproduced:** it fails 1 time in 20 when run on its own.
    - **Cause:** a race inside the test. The product behaviour is correct. `fill()` returns as soon as it sees `Busy`, but the actor keeps draining the work lane, so a slot frees before the next `send`. A flaky gate is not acceptable.

    **Fix: make the test deterministic.**
    - Either assert on the exact `send` that returned `Busy`: have `fill()` return that result and check it.
    - Or hold the actor's work-lane consumer with a test-only latch while you fill the lane, then release it.
    - Apply the same treatment to `interrupt_when_full`, so that its Interrupt still proves the control lane is never Busy while the work lane is full.

    **Evidence required:** 200 consecutive runs pass, using `for i in $(seq 200); do cargo test -q --test gateway_ops || exit 1; done`, plus `make test` exiting 0. Push to the same branch and report. Keep the change limited to this test file.
report:
  commit: abc7e953940972724676300a30e0b90adf0ba878
  op_id: 01M6ATYBHAFMTNB4PMYKACCDZB
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: 'Final gateway-test race follow-up for rejection 6BYA552SKP3255G2XKEZW141TG. Only tests/gateway_ops.rs changed (20 insertions, 13 deletions); signed head abc7e953940972724676300a30e0b90adf0ba878 is pushed on e1b-hardening. PR #3 is OPEN/MERGEABLE/CLEAN; no merge. fill_and_control returns the exact Busy work result and immediately issued control result, so neither test asserts against a later racy work probe. The initial turn uses a cancellation-aware fixture delay. Required exact loop `for i in $(seq 200); do cargo test -q --test gateway_ops || exit 1; done`: exit 0, 200 consecutive runs, all three tests each time. make test: exit 0, unattended (74 Rust tests, 13 Python gate checks, 37 native/TLC variants). Stress file checksum and every canonical input hash match the final commit. Final-head parity and strict receipt: exit 0, clean source; receipt 3ea786f0557b81a1736e755dd4c5df9a9d80e5879acbed1307f5cf6db108afbf. Full report is in the signed commit message to honor the one-file scope: https://github.com/bboxy450/bboxy450-harness/commit/abc7e953940972724676300a30e0b90adf0ba878 . Generated evidence remains local/untracked under .bboxy/runs/gateway-determinism/. Tracked tree clean; no new live requests.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1b: harness core hardening — cancel stall, pgid reuse, non-vacuous models and tests, streamed exec output'
type: task
updated: 2026-11-01T06:07:48.218Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Fix busy_not_wait using the specific send that returned Busy, without a later work probe
    reference: tests/gateway_ops.rs::fill_and_control and ::busy_not_wait; signed commit abc7e953940972724676300a30e0b90adf0ba878
  - criterion: Apply the same observed-full/control-admission treatment to interrupt_when_full; preserve duplicate Ack, Cancelled cause and single ending
    reference: tests/gateway_ops.rs::interrupt_when_full; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/200-runs.log
  - criterion: 200 consecutive runs using the exact requested shell loop
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/200-runs.json (exit_code 0, consecutive_passes 200); 200-runs.log; test_file_sha256 0ac333bb286fb98840f9c14ec3f070fb52d7440c9d1879f304ab993b62bb27b7
  - criterion: make test exits 0 unattended
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/make-test.json; make-test.log; test-current.json — all eight canonical layers exit 0
  - criterion: Exactly one changed file; signed/pushed same branch and current-head formal proof
    reference: /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/final-head-verification.json; final-head-formal.json; verification/evidence/3ea786f0557b81a1736e755dd4c5df9a9d80e5879acbed1307f5cf6db108afbf/evidence.json — source abc7e953940972724676300a30e0b90adf0ba878, dirty false, 37 exact graph/verdict pairs
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


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

Focused rework for rejection 434WGCZCFRNMZZCH092EG8VGA7 is pushed to the same e1b-hardening branch, signed head 0bab268354f0c1a04adf17e5abe592758660ca58. PR #3 is OPEN/MERGEABLE/CLEAN; no merge. Leader exit now uses WNOWAIT and preserves ownership until group SIGKILL and final reap. Real member-survival witness: 101 on unchanged 1d8fa15 product, then 0 after the fix. Both ownership controls fail the exact required invariant under native and TLC (exit 12). All requested refinements (4,6,7,10,14) included. make test exited 0 on 1400c1e: 74 Rust tests, 13 Python gate checks, 37 model variants; all test input hashes match final head. Final-head native/Java parity and strict receipt exited 0; receipt 90cb409779d89e2e397015d3237e6179e34874d7a448fdb9789483d3a43aa26c. Full committed report: https://github.com/bboxy450/bboxy450-harness/blob/0bab268354f0c1a04adf17e5abe592758660ca58/.bboxy/runs/2026-10-31-harness-core-hardening-rework-report.log . No extra live requests for this focused rework; earlier live PASS is historical. Tracked tree clean, local run/receipt archives retained.

```json
{
  "commit": "0bab268354f0c1a04adf17e5abe592758660ca58",
  "op_id": "01M6AT5E65D4RXCQ9V41TKE0AN",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Blocker item 2: cancellation kills a surviving member after leader exit, without signalling after final reap",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/leader-exit-red.json (101 on 1d8fa15); targeted-green.json (0); src/process.rs:119; tests/cancel_process.rs:38; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-rework-report.log"
      },
      {
        "criterion": "Process control (a) fails NoMemberAfterCancel; control (b) fails NoSignalToEmptyGroup, on both native and TLC",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/model-controls.json; models-red.json (101); targeted-green.json (0); tests/formal_nonvacuity.rs; final receipt 90cb409779d89e2e397015d3237e6179e34874d7a448fdb9789483d3a43aa26c"
      },
      {
        "criterion": "Item 4: bounded type slots, producers/consumers, conservation and real broken overflow to 257",
        "reference": "tests/formal/models.rs:300; tests/formal_nonvacuity.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/model-controls.json (8580/8708 complete states; broken slot/capacity violations; TLC 12)"
      },
      {
        "criterion": "Item 6: assert each mutation's exact law and detail",
        "reference": "tests/protocol_laws.rs:15; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/targeted-green.log; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-rework-report.log"
      },
      {
        "criterion": "Item 7: poll ESRCH and complete PID publication",
        "reference": "tests/cancel_process.rs:11 and :23; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/targeted-green.log; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/2026-10-31-harness-core-hardening-rework-report.log"
      },
      {
        "criterion": "Item 10: remove <1 ms gates; verify direct synchronous admission and held try_reserve capacity without an actor/runtime",
        "reference": "src/gateway/mod.rs admission_tests::held_capacity_returns_busy_without_an_actor_or_runtime; tests/gateway_ops.rs; tests/gateway_host.rs; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/targeted-green.log"
      },
      {
        "criterion": "Item 14: require EpochNeverReused as the violated invariant for control (e)",
        "reference": "tests/formal_nonvacuity.rs::subscriber_model_can_lose_an_entire_epoch_and_rejects_both_old_rules; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/targeted-green.log"
      },
      {
        "criterion": "Full unattended stack and final signed head, matching input hashes, pinned parity and strict receipt",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/rework/make-test.json; make-test.log; final-head-formal.json; final-head-verification.json; /Users/feral/mydev/bboxy450-harness-core/verification/evidence/90cb409779d89e2e397015d3237e6179e34874d7a448fdb9789483d3a43aa26c/evidence.json"
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

Final gateway-test race follow-up for rejection 6BYA552SKP3255G2XKEZW141TG. Only tests/gateway_ops.rs changed (20 insertions, 13 deletions); signed head abc7e953940972724676300a30e0b90adf0ba878 is pushed on e1b-hardening. PR #3 is OPEN/MERGEABLE/CLEAN; no merge. fill_and_control returns the exact Busy work result and immediately issued control result, so neither test asserts against a later racy work probe. The initial turn uses a cancellation-aware fixture delay. Required exact loop `for i in $(seq 200); do cargo test -q --test gateway_ops || exit 1; done`: exit 0, 200 consecutive runs, all three tests each time. make test: exit 0, unattended (74 Rust tests, 13 Python gate checks, 37 native/TLC variants). Stress file checksum and every canonical input hash match the final commit. Final-head parity and strict receipt: exit 0, clean source; receipt 3ea786f0557b81a1736e755dd4c5df9a9d80e5879acbed1307f5cf6db108afbf. Full report is in the signed commit message to honor the one-file scope: https://github.com/bboxy450/bboxy450-harness/commit/abc7e953940972724676300a30e0b90adf0ba878 . Generated evidence remains local/untracked under .bboxy/runs/gateway-determinism/. Tracked tree clean; no new live requests.

```json
{
  "commit": "abc7e953940972724676300a30e0b90adf0ba878",
  "op_id": "01M6ATYBHAFMTNB4PMYKACCDZB",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Fix busy_not_wait using the specific send that returned Busy, without a later work probe",
        "reference": "tests/gateway_ops.rs::fill_and_control and ::busy_not_wait; signed commit abc7e953940972724676300a30e0b90adf0ba878"
      },
      {
        "criterion": "Apply the same observed-full/control-admission treatment to interrupt_when_full; preserve duplicate Ack, Cancelled cause and single ending",
        "reference": "tests/gateway_ops.rs::interrupt_when_full; /Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/200-runs.log"
      },
      {
        "criterion": "200 consecutive runs using the exact requested shell loop",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/200-runs.json (exit_code 0, consecutive_passes 200); 200-runs.log; test_file_sha256 0ac333bb286fb98840f9c14ec3f070fb52d7440c9d1879f304ab993b62bb27b7"
      },
      {
        "criterion": "make test exits 0 unattended",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/make-test.json; make-test.log; test-current.json — all eight canonical layers exit 0"
      },
      {
        "criterion": "Exactly one changed file; signed/pushed same branch and current-head formal proof",
        "reference": "/Users/feral/mydev/bboxy450-harness-core/.bboxy/runs/gateway-determinism/final-head-verification.json; final-head-formal.json; verification/evidence/3ea786f0557b81a1736e755dd4c5df9a9d80e5879acbed1307f5cf6db108afbf/evidence.json — source abc7e953940972724676300a30e0b90adf0ba878, dirty false, 37 exact graph/verdict pairs"
      }
    ],
    "exit_code": 0
  }
}
```
