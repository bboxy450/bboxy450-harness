---
type: report
title: 'Oracle verification of WP5: formal pass, canonical KB failure remains'
task: '[[01M6J7MZWKH3WZA2XE8KBGA78D]]'
commits:
- a3c5ef086d3d40265cae6cddda865892e07b9c91
- 689dca090b1a52a23f5167972333b8d32ea64538
- dec5d5040f5e946a7fb489890518eef2efa37a2b
verify:
  command: make test-formal && make test-formal-parity
  exit_code: 0
author: seat:01M6J5YFATHYCK148JT0C4XGZ1
created: 2026-11-04T03:53:35.995Z
id: 01M6JB1P2KN9RDWG02AHCY8FMA
updated: 2026-11-04T03:53:35.995Z
number: 20261103136306
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:b3e0fbf020d67c1a2dfae758602299d18d8c5c6b7a9998adcb0a5b2ada96613c
  frontmatter_digest: sha256:25e9db55ca8d5aabe38e204d19214281c58fb52f90140af343c22ade9b0b336e
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: J2TSwGU2qRo11MiuuOWEyfRPBFoGqjnn8nWOdZ/xbccUQzR0k9/watU9KTgd4SmnkAkWjVYs39m8q6YUNUblAA==
  log: .bboxy/log/2026-11-04.jsonl#43103
---
# WP5 independent review — not accepted or merged

Read the real worktree, signed history, full report, model builders at tests/formal/models.rs:3867/4149/4325, nonvacuity/algebra tests at tests/formal_nonvacuity.rs:1027/1116/1185, and the three inventories. Only six allowed paths changed (models, nonvacuity, three TOMLs and the report). All three supplied commit signatures independently verify with the approved Boxy key, exit 0.

Mechanisms: per-harness trial/revival pins and entry-global terminal fences; component join over validated log projection with fixed terminal ordering; separate read/check/hand-write/no-replace publication actions; loader reachability and active-pin checks before materialization/serving. Nonvacuity reaches race/ABA/foreign ownership/crash/revival/cross-harness cases and every normalized join pair; algebra exhausts idempotence, commutativity and 27^3 associativity. Eight broken controls target specific weakened guards. These finite abstract proofs are not dream API/process or arbitrary Rust conformance; implementing WPs and WP9 retain those obligations.

## Commands actually run by Oracle

- make test-formal && make test-formal-parity: exit 0 on dec5d5040f5e946a7fb489890518eef2efa37a2b. All 130 native/Java state sets, labeled transition sets and expected safety/deadlock/liveness verdicts match; 42 contracts, one inventoried crate. Log /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-wp5-review-b8wglybf/verify.log, SHA256 873bcc06b83afefff41daff653b68791e1501ed00026fa2cc6e8e20f7b9c34af.
- make check-formal-evidence: exit 0. Receipt 992376be40de68b50fb21fb8756d1848a59f557c80fdff402b3ccceabd385329, independently hashed, source dec5d50, dirty=false. Durable copy verification/evidence/<that digest>/evidence.json plus complete models/ artifacts remains in WP5 worktree.
- python3 -I scripts/checks.py kb-records: exit 1, duplicate 5974 in d5974-log-ir-v1-only-add and d5974-memories-are-kb-records. Both numbers independently read from base 6957019; those source files have no WP5 diff.
- Worker full make-test log was read and SHA256 independently matched: 616925b68b8a699a4ccef21f256ad68da2cccc4e2b3d8ab7b71381aba1c975c1, /tmp/dream-wp5-test-final.log. It ends at the same KB checker failure. Worker reports full make exit 2 and preceding/unit/process/GUI/packaging/smoke checks 0. The Oracle did not rerun the whole unchanged canonical suite; that failure is not waived or relabelled green.

The formal rerun changed only generated .bboxy/runs/formal-native.json. Its exact output was preserved beside the Oracle log, then only that Oracle-generated change was restored. Worktree clean at the same signed source afterward.

## Disposition

Primary formal verification is independently proven. Acceptance/merge remains pending because the required full make test gate is not green. Existing-identity KB repair has no supported authenticated route in the current catalogue (decision d20261103136278); a shell renumber or re-import is not authorized. No baseline exception is granted. Do not repeat the same failing full gate without an actual relevant change.

Before integration, rebase onto current canonical origin, include the governing signed spec/decision in the source binding, and refresh exact-source formal/full-test evidence. WP9 must replace planned source mappings with actual implementation/test conformance. No product implementation is accepted by this report, and the WP5 worktree/evidence is retained.
