---
type: task
title: 'Dream WP7 revision 2: trial'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp7
allowed_paths:
- src/dream/trial.rs
- src/dream/mod.rs
- src/policy.rs
- src/frontends/headless.rs
- tests/dream_pipeline.rs
- tests/support/dream.rs
- tests/fixtures/dream/trial/**
- .bboxy/reports/
verify: cargo test --test dream_pipeline trial && cargo test --test policy_gate
depends_on:
- '[[2026-11-03-dream-wp2-vcs-r2]]'
- '[[2026-11-03-dream-wp6-synthesis-r2]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TX21SQAR0DGZ13DTRQ8A
status: open
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T03:14:56.915Z
created: 2026-11-04T03:14:56.915Z
number: 20261103136214
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:5b69d1f51905518b74444fa1247767c0c6847ca8dd616ac2b3c88d773bc7d29e
  frontmatter_digest: sha256:94923ece8e7df424793f60848049c4a60ad65a0a8350f54410cae9455725e46b
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: qwge0SfTAj2YN35QF2zKK5V+TH8phSWnwFE6dzJ5TeI872x5ndaoGv3rL3XLVMcFLuF67Djp3tt/fbMVOkTAAQ==
  log: .bboxy/log/2026-11-04.jsonl#42228
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

This is a NEW revision-2 brief against signed dream-log b4894fb6dd4e9731256760683ae2548a03f72bad §§2–12, with materially refined paths, dependencies and/or verification. Read §12 precedence because original companion mirrors remain blocked by duplicate-number host repair. Any older draft attempted during the transport outage stays unassigned; never execute duplicate work or infer assignment from record existence. Original uncertain operation IDs are not reused or replayed.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp7
Allowed paths: src/dream/trial.rs, src/dream/mod.rs, src/policy.rs, src/frontends/headless.rs, tests/dream_pipeline.rs, tests/support/dream.rs, tests/fixtures/dream/trial/**, .bboxy/reports/.
Prerequisites: WP2, WP6 integrated and independently verified. Oracle delegates when prerequisites exist; do not park a worker on an unavailable prerequisite.
Outcome/acceptance: Worktree/fork/continue and per-pair trial verdicts; mod only dispatch, policy only trial profile, headless only continuation wiring, pipeline only trial cases. Use thresholds@2 including scheduling; disable automatic recall in both arms and pin unrelated context; insufficient sample/usage/isolation => Inconclusive. No machine-wide candidate queue; host heavy-test claim skipped nonblocking if unavailable. Deny writes/publication outside isolated scratch and operator identity.
Verify: cargo test --test dream_pipeline trial && cargo test --test policy_gate
Shared files are owned sequentially by the dependencies in dream-log §10. No blanket src/dream/** ownership. Report exact test outputs/exits and meaningful process witnesses; check_transcript means the actual protocol checker/invariants, not an invented shell target. Model evidence is supplied by WP5 and refreshed against final code by WP9.
