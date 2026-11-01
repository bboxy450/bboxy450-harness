---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T04:45:38.990Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
decided: 2026-10-31
id: 01M6APTV7TZMHXYY4NHP8S3XP1
links:
- rel: amends
  to: '[[2026-10-31-harness-core-hardening]]'
- rel: assigns
  to: '[[d1134-reset-trusted-head]]'
- rel: implements
  to: '[[budgets]]'
number: 1372
provenance:
  digest: sha256:fae7fc086f15cbc92846190d1db0619b0f94ace2938606eea246e292aa3adb79
  frontmatter_digest: sha256:46be9a6d6498f63e9ef0a99ccb018c5d3d73779fe15bc47eae99c0f3d461a6dd
  log: .bboxy/log/2026-11-01.jsonl#7282
  sig: uCJqyZihSSKlg6N9R7Jc8PSA4tIwahor7xX54HLN92wUq2grArWjY5RZQuNlEtaCsvpYx9ANwZqNlMg32gGHBw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: decided
title: 'Task #1117 (E1b) scope: item 14 Reset trusted head (d1134) and item 15 structural rt budget gate'
type: decision
updated: 2026-11-01T04:45:38.990Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
# E1b scope amendment: items 14 and 15

Task #1117 is claimed, and the task record is protected, so this record amends it instead. It supersedes the brief's line "Out of scope: the Reset rule".

**Item 14. The Reset trusted head ([[d1134-reset-trusted-head]]).** The Oracle assigns d1134 to E1b, so this work is now in scope:
- the trusted-head rule in the gateway;
- a tail-loss action in HarnessSubscriberReplay;
- broken control (d), which must FAIL safety under both the native checker and TLC;
- four cursor cases in `tests/store_recovery.rs`.

**Item 15. A structural runtime-budget gate, per `specs/budgets.md` §1 at `82f2edd`.** E3 saw `tests/rt_budget.rs:52` fail with a wall-clock p99 of 8.38 ms and then 7.51 ms, on unchanged code, at load 44–60 on 10 CPUs. The new gate in `make test` has three checks:
- `assert_off_rt_worker()` hooks in every spawn_blocking entry point;
- 16 concurrent greps hold at most 4 permits;
- no walk runs without a permit.

Add a `make bench-budgets` target. When load exceeds ncpu it reports UNMEASURED, never PASS. If a measurement exceeds its allocation, it exits non-zero.

Both items require red→green evidence in the report.
