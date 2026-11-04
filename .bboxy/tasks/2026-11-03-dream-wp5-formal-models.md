---
type: task
title: 'Dream WP5: Dream lifecycle, merge and sink models with broken controls'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp5
allowed_paths:
- tests/formal/models.rs
- tests/formal/export.rs
- tests/formal_nonvacuity.rs
- verification/corpus.toml
- verification/coverage.toml
- verification/contracts.toml
- .bboxy/reports/
verify: make test-formal && make test-formal-parity
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J7MZWKH3WZA2XE8KBGA78D
status: claimed
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T03:13:28.534Z
created: 2026-11-04T02:54:14.180Z
number: 5989
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
assignee: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
verify_revision: 1
provenance:
  digest: sha256:f284e576332bfda9d8790ccfe1c20bbe06b29e568dce8fda0dbd681228e2ce8e
  frontmatter_digest: sha256:916acaa82752c0870382009414a094afd062218d6bd85d891ff8729ef11d3b3f
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: uihJQmqwMqE5hnTedbGfP9GFpIOm1WUHbsBr9B9RGiZFgYOMI1SLJhqdt+dKjpqKOg15t0CeNUdlrFID1MYlCQ==
  log: .bboxy/log/2026-11-04.jsonl#42167
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp5
Allowed paths: tests/formal/models.rs, verification/*.toml, .bboxy/reports/.
Required outcome and acceptance: Implement dream-log §8 three formal models and Broken controls, inventories, finite bounds, safety/liveness, fairness, source mapping. Independent of implementation. Lumen is reviewing sink TOCTOU and lifecycle semantics; obtain refined contracts before finalizing affected models. Do useful lifecycle/merge work immediately. Confirm exact reachable-state/transition and verdict parity against pinned Java TLC as canonical stack supports; report unmodeled implementation behavior explicitly.
Verify: make test-formal

Oracle refinement 2026-11-03: signed dream-log at b4894fb6dd4e9731256760683ae2548a03f72bad §§8,12 is the current baseline, superseding earlier sink and per-harness projection assumptions. Model state[e,h], global terminal fencing, same-pair pinned Trial(Pass), new revival identity, set-union/idempotent/commutative/associative projection independent of arrival/wall clock. Model HandEdit between Check and Publish; good path is atomic no-replace, conflict => explicit skip, never overwrite-rename. Reach the decisive interleavings and assert named safety/nonvacuity properties in tests/formal_nonvacuity.rs. tests/formal/export.rs is allowed only if the current export registry needs mechanical additions. Use only verification/{corpus,coverage,contracts}.toml. Include full reachable state/transition graph and safety/deadlock/liveness verdict parity against pinned Java TLC; unsupported semantics, incomplete exploration, timeout or disagreement fails. Scheduling is dream-thresholds@2, not @1; score/spend/timeout rows unchanged. Final source conformance/hashes will be refreshed after implementation/WP9; clearly separate model proof from API/process witnesses owned by implementing WPs. New verify replaces the old command: make test-formal && make test-formal-parity. A fresh task_report under that exact command is required before acceptance.
