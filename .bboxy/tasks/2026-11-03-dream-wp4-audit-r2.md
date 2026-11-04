---
type: task
title: 'Dream WP4 revision 2: audit'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp4
allowed_paths:
- src/dream/audit.rs
- src/dream/graph.rs
- src/dream/entry.rs
- src/dream/projection.rs
- src/dream/resolve.rs
- src/dream/deny.rs
- src/dream/mod.rs
- tests/dream_audit_broken.rs
- tests/dream_graph.rs
- tests/dream_identity.rs
- tests/fixtures/dream/good/**
- tests/fixtures/dream/broken/**
- .bboxy/reports/
- verification/coverage.toml
verify: cargo test --test dream_audit_broken --test dream_graph --test dream_identity
depends_on:
- '[[2026-11-03-dream-wp3-d0-r2]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TV4EVP7GZG875FC5J4P5
status: open
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T04:07:15.385Z
created: 2026-11-04T03:14:54.789Z
number: 20261103136210
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:56ebea47cf2209a7932e72a95a36df6bb623fe9748d4641505e2378d92bc56a3
  frontmatter_digest: sha256:4a4d3a6c99132e60e661bca4d388003597dec904850b2f6ea9758197c3b4615b
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: j4A0Odjjx+aI8rG7mROOeWGA6OhM1yYxV/Uxi44CB2jjh5cGalmMETUPPs6xDf9wwoydtznR6b+B09qBhfKJDA==
  log: .bboxy/log/2026-11-04.jsonl#43231
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

This is a NEW revision-2 brief against signed dream-log b4894fb6dd4e9731256760683ae2548a03f72bad §§2–12, with materially refined paths, dependencies and/or verification. Read §12 precedence because original companion mirrors remain blocked by duplicate-number host repair. Any older draft attempted during the transport outage stays unassigned; never execute duplicate work or infer assignment from record existence. Original uncertain operation IDs are not reused or replayed.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp4
Allowed paths: src/dream/audit.rs, src/dream/graph.rs, src/dream/entry.rs, src/dream/projection.rs, src/dream/resolve.rs, src/dream/deny.rs, src/dream/mod.rs, tests/dream_audit_broken.rs, tests/dream_graph.rs, tests/dream_identity.rs, tests/fixtures/dream/good/**, tests/fixtures/dream/broken/**, .bboxy/reports/.
Prerequisites: WP3 integrated and independently verified. Oracle delegates when prerequisites exist; do not park a worker on an unavailable prerequisite.
Outcome/acceptance: Audit P1-P9, why/graph and shared identity/pin/projection/deny validators. mod only exports/dispatch. Exact canonical identity vectors and committed pins; per-harness plus global-terminal projection; all broken fixtures reject with named P and good fixture accepts. No clock/arrival-order authority, no hand-edited body overwrite.
Verify: cargo test --test dream_audit_broken --test dream_graph --test dream_identity
Shared files are owned sequentially by the dependencies in dream-log §10. No blanket src/dream/** ownership. Report exact test outputs/exits and meaningful process witnesses; check_transcript means the actual protocol checker/invariants, not an invented shell target. Model evidence is supplied by WP5 and refreshed against final code by WP9.

Oracle source-inventory amendment 2026-11-03: every new Rust module must be listed before this package can produce fresh current-source formal evidence; scripts/formal.py enforces exact source equality. verification/coverage.toml is authorized only for this WP's actually introduced source paths, their planned-to-current promotion where relevant, and a precise note of this WP's real API/process tests and still-unmodeled behavior. Do not list nonexistent files, widen another WP's implementation scope, claim model conformance from inventory membership, omit sources to make the gate pass, or weaken a model/control/check. If a new stateful contract is not covered by the existing refined models and real witnesses, report the exact modeling gap for a separate scoped amendment. WP9 retains final cross-package contract/source/test mapping and fresh evidence responsibility; this per-package inventory maintenance cannot be deferred until WP9. Verify command unchanged; required formal/parity and canonical gates remain in force.
