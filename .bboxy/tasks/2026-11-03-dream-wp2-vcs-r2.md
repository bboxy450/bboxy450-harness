---
type: task
title: 'Dream WP2 revision 2: vcs'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp2
allowed_paths:
- src/vcs.rs
- src/lib.rs
- src/engine/mod.rs
- src/backend/acp_agent.rs
- src/import/mod.rs
- src/import/edge.rs
- tests/dream_vcs.rs
- tests/import_native.rs
- tests/cli_headless.rs
- .bboxy/reports/
verify: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
depends_on:
- '[[2026-11-03-dream-wp1-ir-1-3]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TT7HY4B7YPAP8SEWBE40
status: open
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T03:14:53.626Z
created: 2026-11-04T03:14:53.626Z
number: 20261103136208
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:981b0a921853739067316f7b98737ecd4162540654e18c81a8b9359d059cd4a7
  frontmatter_digest: sha256:3a79a40b6e8d0fa92b64b80870d418e3e0b14150eaf9bae7be37bde0717179bb
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: TMHUtg2gx3mqpq+rvxD9W2VbjoI62zmkWB+D+yN6VojqC8i+Yadm1dhXeBxI5ATxG9Uy465DZc7SGZLlkJyrAg==
  log: .bboxy/log/2026-11-04.jsonl#42210
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

This is a NEW revision-2 brief against signed dream-log b4894fb6dd4e9731256760683ae2548a03f72bad §§2–12, with materially refined paths, dependencies and/or verification. Read §12 precedence because original companion mirrors remain blocked by duplicate-number host repair. Any older draft attempted during the transport outage stays unassigned; never execute duplicate work or infer assignment from record existence. Original uncertain operation IDs are not reused or replayed.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp2
Allowed paths: src/vcs.rs, src/lib.rs, src/engine/mod.rs, src/backend/acp_agent.rs, src/import/mod.rs, src/import/edge.rs, tests/dream_vcs.rs, tests/import_native.rs, tests/cli_headless.rs, .bboxy/reports/.
Prerequisites: WP1 integrated and independently verified. Oracle delegates when prerequisites exist; do not park a worker on an unavailable prerequisite.
Outcome/acceptance: VCS capture and shared normalized repo-key helper; importer mapping. src/lib.rs only vcs declaration; engine/backend only metadata capture. Source must be actual git/native data, absent fields remain absent. Witness no-repo/git failure/dirty checkout and differing native availability. Avoid blocking active request/UI on git.
Verify: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
Shared files are owned sequentially by the dependencies in dream-log §10. No blanket src/dream/** ownership. Report exact test outputs/exits and meaningful process witnesses; check_transcript means the actual protocol checker/invariants, not an invented shell target. Model evidence is supplied by WP5 and refreshed against final code by WP9.
