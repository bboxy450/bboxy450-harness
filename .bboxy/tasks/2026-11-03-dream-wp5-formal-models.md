---
type: task
title: 'Dream WP5: Dream lifecycle, merge and sink models with broken controls'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp5
allowed_paths:
- tests/formal/models.rs
- verification/*.toml
- .bboxy/reports/
verify: make test-formal
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J7MZWKH3WZA2XE8KBGA78D
status: claimed
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T02:54:27.621Z
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
provenance:
  digest: sha256:9f16e85c3daa622e36977bd106e780fcb79ce67f121cf83ff975f3b382d55b7e
  frontmatter_digest: sha256:778dd348cad2e41aa8622a3bfd8093e1e23d6ef688a9c720ce8faf821d02ad9f
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: /Bh00B6lIS8GxqrI/TeWloTp3zGW7wRF9wjm4UJIqdM2Dlj2x/cw4lr0YegAFgku+5PyyEYUQ2kcQo9zghI5Dw==
  log: .bboxy/log/2026-11-04.jsonl#40321
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
