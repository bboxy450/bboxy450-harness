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
  digest: sha256:b1daf791cf071a0121b2903ebcaf863d5869fc7c1de2b467c65ac1d12bd46367
  frontmatter_digest: sha256:dd73a91a9a6d02f75a66763ecfb764f5dd359f605f9283df434d9dcd68d2e329
  log: .bboxy/log/2026-11-01.jsonl#4951
  sig: qdum0Cumh3OTixeLbz2y7M2YZFw/4RPLgd3z2V/6zh9h4D0WfSgJRJEjffx7m1rq/5GOADqqcw4Tn2D2b+XmCg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1b: harness core hardening — cancel stall, pgid reuse, non-vacuous models and tests, streamed exec output'
type: task
updated: 2026-11-01T04:01:51.841Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
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
