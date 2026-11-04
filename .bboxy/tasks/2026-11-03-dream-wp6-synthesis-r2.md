---
type: task
title: 'Dream WP6 revision 2: synthesis'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp6
allowed_paths:
- src/dream/synth.rs
- src/dream/prompt.rs
- src/dream/mod.rs
- tests/dream_pipeline.rs
- tests/support/dream.rs
- tests/fixtures/dream/synth/**
- .bboxy/reports/
- verification/coverage.toml
verify: cargo test --test dream_pipeline synth
depends_on:
- '[[2026-11-03-dream-wp3-d0-r2]]'
- '[[2026-11-03-dream-wp4-audit-r2]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TVSRQ0KRYMQ3ET73F67S
status: open
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T04:07:16.459Z
created: 2026-11-04T03:14:55.866Z
number: 20261103136212
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:691ada5cdaa6c3523eed3ce9641456ab47f28253ef5e0bbafa445778a0cdd6c3
  frontmatter_digest: sha256:99c5422f2f0acb7bd153b6f3e3f4aa32ea40aedf992eaf64eededb49a5d32f66
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: DUmxeF7fA/FG2RmIiIuUUbbanbeLKmiuSE2A6ywuOccBHDpsP9nTYG9kVxqajNPbv5jrr+cEn2wzEP3uKl7+CQ==
  log: .bboxy/log/2026-11-04.jsonl#43239
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

This is a NEW revision-2 brief against signed dream-log b4894fb6dd4e9731256760683ae2548a03f72bad §§2–12, with materially refined paths, dependencies and/or verification. Read §12 precedence because original companion mirrors remain blocked by duplicate-number host repair. Any older draft attempted during the transport outage stays unassigned; never execute duplicate work or infer assignment from record existence. Original uncertain operation IDs are not reused or replayed.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp6
Allowed paths: src/dream/synth.rs, src/dream/prompt.rs, src/dream/mod.rs, tests/dream_pipeline.rs, tests/support/dream.rs, tests/fixtures/dream/synth/**, .bboxy/reports/.
Prerequisites: WP3, WP4 integrated and independently verified. Oracle delegates when prerequisites exist; do not park a worker on an unavailable prerequisite.
Outcome/acceptance: Strict D1 synthesis prompt/parse and Pending/Proposed events; consume WP4 shared validators. mod only dispatch; pipeline only synthesis cases. Scripted provider/private HOME, no tools/model filesystem effects, unknown fields refused, no-op, candidate limits/evidence allowlist, redaction and deny-scan.
Verify: cargo test --test dream_pipeline synth
Shared files are owned sequentially by the dependencies in dream-log §10. No blanket src/dream/** ownership. Report exact test outputs/exits and meaningful process witnesses; check_transcript means the actual protocol checker/invariants, not an invented shell target. Model evidence is supplied by WP5 and refreshed against final code by WP9.

Oracle source-inventory amendment 2026-11-03: every new Rust module must be listed before this package can produce fresh current-source formal evidence; scripts/formal.py enforces exact source equality. verification/coverage.toml is authorized only for this WP's actually introduced source paths, their planned-to-current promotion where relevant, and a precise note of this WP's real API/process tests and still-unmodeled behavior. Do not list nonexistent files, widen another WP's implementation scope, claim model conformance from inventory membership, omit sources to make the gate pass, or weaken a model/control/check. If a new stateful contract is not covered by the existing refined models and real witnesses, report the exact modeling gap for a separate scoped amendment. WP9 retains final cross-package contract/source/test mapping and fresh evidence responsibility; this per-package inventory maintenance cannot be deferred until WP9. Verify command unchanged; required formal/parity and canonical gates remain in force.
