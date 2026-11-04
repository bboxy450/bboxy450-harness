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
status: claimed
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T03:59:42.908Z
created: 2026-11-04T03:14:53.626Z
number: 20261103136208
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
assignee: seat:01M6J7N5V103SG1KS0THEXB9R1
provenance:
  digest: sha256:8c089b9db53eb766a73d3e80c8d41ba6f2071305c2fbf649a84dc54aacfa71d8
  frontmatter_digest: sha256:5ee0c63eae520a6a5127da38cbde2c649fb47e38ce97e25954611a4a2c5e1292
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: nPJ4ndrcRWGQOTAGzNpjqNQh7f3EqOb5TKt2+fNNdbftDFM1F+hHr6noyJmS/pLQzt4Hnmu3nBjijayAJQ2KBQ==
  log: .bboxy/log/2026-11-04.jsonl#43195
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

Oracle dependency-preparation amendment 2026-11-03: WP1 remains reported/unaccepted because the unchanged canonical KB-number gate fails; this must not block independent implementation preparation. The Oracle independently reviewed WP1 source/signatures, reran its exact primary verify (19 tests, exit 0), the complete 15-fixture Unix/InProc bridge test (0), confirmed 12 old corpus byte hashes, and verified the source-bound formal receipt. WP5's exact formal command was independently rerun (130 models, exit 0). No final acceptance or merge is implied.

Your NEW assigned writing worktree is /Users/feral/mydev/bboxy450-harness-dream-wp2, branch dream/wp2, prepared base a279e0fdc5866068db567561e24ffa2bc0897d6b. It starts at canonical origin/main 9996506 and contains only signed dependency candidate cherry-picks: WP1 0c53f325/fd0cd6f0 -> f443e74/83ef838; WP5 a3c5ef0/689dca0/dec5d50 -> 03ba424/53f0037/a279e0f. Governing current dream-log/decision records are therefore present. All inherited dependency source is read-only in WP2; your allowed edit paths and verify command are unchanged. Compare your changes against this prepared base in your report, not against main. The old launch cwd may still be WP1: EVERY source edit, git/build/test command must explicitly use the assigned WP2 workdir. Do not edit the WP1 worktree or alter its five retained generated evidence files.

This explicitly supersedes the earlier requirement that prerequisites first be accepted/integrated into main for starting this bounded candidate work. Acceptance/publication/merge still requires all written gates including canonical make test to be green; no number repair, filtering of that gate or baseline waiver is authorized. Do not publish or merge this branch. Reuse no old receipt for the combined base: create fresh current-source evidence for your completed source, and report the actual canonical gate result and outstanding baseline blocker honestly. A same-premise failing-gate loop is not useful; no operator prompt or speculative cleanup. Later packages may start only under their own signed Oracle task amendment against independently checked candidate prerequisites.

VCS acceptance details: capture actual repository/branch/commit/dirty data without inventing missing native fields, without operator auth/network or exposing credential-bearing remote URLs. Keep subprocess/metadata capture bounded and off the live UI hot path; report a missing interface/budget rather than adding an unbounded wait. Cover no-repo/git failure/dirty and available native mappings with private git fixtures. Source/model/process limits remain explicit; WP9 refreshes final implementation mappings.
