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
- verification/coverage.toml
- src/import/reproject.rs
- tests/ir_edge.rs
verify: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
depends_on:
- '[[2026-11-03-dream-wp1-ir-1-3]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TT7HY4B7YPAP8SEWBE40
status: rejected
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T05:06:20.373Z
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
artifacts:
- '[[01M6JERN7GEB1JQP3RTE2JXSQ2]]'
- '[[01M6JERNMYDQHENR09D0PHH58W]]'
verify_result:
  command: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
  exit_code: 0
  evidence:
  - criterion: Actual VCS capture, normalized repo key, no-repo/failure/dirty/native availability; no UI hot-path wait
    reference: src/vcs.rs:45,92,150,176,210; src/engine/mod.rs:179; src/backend/acp_agent.rs:235; src/import/edge.rs:15; tests/dream_vcs.rs:140,176,196,211,254,274; tests/import_native.rs:872; /tmp/dream-wp2-verify-final.log — primary command exit 0, 23 tests
  - criterion: Native parser provenance and retained source/replay bytes, unknown data and actual transcript invariants
    reference: src/import/mod.rs:16,950,961; src/import/reproject.rs:70; tests/ir_edge.rs:183,227,245 — cargo test --test ir_edge exit 0 (8 tests); make test-ir && cargo test --test protocol_golden --test ir_v1_corpus exit 0 (19 tests); tests/cli_headless.rs:66
  - criterion: Credential-free metadata/raw/ext and Git tracing negative control
    reference: tests/import_native.rs:966; tests/dream_vcs.rs:211; /tmp/dream-wp2-trace-red.log — named VcsMetadataTraceDisabled assertion exit 101 with old helper; final primary exit 0 for engine and ACP; src/vcs.rs:99 clears inherited GIT_*
  - criterion: Fresh final-HEAD formal inventory, native/TLC parity and retained evidence
    reference: make test-formal test-formal-parity check-formal-evidence — exit 0; /tmp/dream-wp2-formal-final.log; verification/evidence/950b206fa5b191feb0f382d0a4e65c0887c9da0956f145d3dcf80e5b07ef7cd2/evidence.json — source a1c22c287a8d612c43e2bbc3606590fa72234f1b, dirty=false, 130 models/42 contracts. Git parsing/snapshot/bytes/timing remain implementation-only; WP9 owns combined implementation mapping.
  - criterion: Unattended unit/integration/GUI/application smoke and honest canonical baseline
    reference: make test-gui test-app-packaging smoke-app — exit 0 (/tmp/dream-wp2-remaining-gates.log). make test — exit 2 after 17 successful commands; unchanged duplicate KB number 5974 (KB checker exit 1) in /tmp/dream-wp2-make-test-final.log and .bboxy/runs/test-current.json. No waiver or number repair; acceptance remains pending.
  - criterion: Signed scoped commits, complete report, prepared base and worktree state
    reference: Prepared base a279e0fdc5866068db567561e24ffa2bc0897d6b; signed source da78cbda91b5de454c38152de8c44b5d587f2086 and report a1c22c287a8d612c43e2bbc3606590fa72234f1b; git verify-commit both exit 0; reports/2026-11-03-dream-wp2-vcs-r2. Only five retained generated .bboxy/runs files dirty; source/index clean; no WP1 edits, merge or publication.
report:
  summary: 'Implemented bounded session-start VCS capture, shared credential-free repo normalization, actual native field mapping and consistent native parser provenance in signed source da78cbda91b5de454c38152de8c44b5d587f2086 against prepared base a279e0fd; full report signed at a1c22c287a8d612c43e2bbc3606590fa72234f1b. Exact primary verify passes 23 tests, ir_edge 8, inherited IR/golden corpus 19; GUI/packaging/app smoke pass unattended. A demonstrated Git Trace2 environment-secret leak has a failing old-helper control (101) and passing engine+ACP regression. Fresh final-HEAD make test-formal test-formal-parity check-formal-evidence exits 0: 130 models/42 contracts, dirty=false, retained receipt SHA256 950b206fa5b191feb0f382d0a4e65c0887c9da0956f145d3dcf80e5b07ef7cd2. Canonical make test exits 2 only at unchanged duplicate KB number 5974 after 17 passing commands; no waiver/repair/merge/publication. Source/index are clean, five generated evidence files remain dirty; no WP1 worktree changes. Acceptance still requires the canonical KB gate to become green.'
  commit: a1c22c287a8d612c43e2bbc3606590fa72234f1b
  reported_by: seat:01M6J7N5V103SG1KS0THEXB9R1
  op_id: 01M6JEY6P01CGA7DYJ3VQPJ4G3
  verify_revision: 0
  report: '[[reports/2026-11-03-dream-wp2-vcs-r2]]'
rejection:
  reason: 'Oracle primary rerun exits 0 (23 tests), but an actual harness/API witness disproves dirty-state correctness. In a private signed parent repo with a tracked child gitlink, modifying child/tracked.txt yields git status --ignore-submodules=none => '' M child''; capture''s --ignore-submodules=all yields empty. Running the actual WP2 harness --json --scripted against that parent exits 0 and stores vcs.dirty=false. Evidence /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-vcs-submodule-nvc9pgy_/oracle-witness.json. This could wrongly admit a changed checkout to clean-source trials. Fix within existing src/vcs.rs/tests/dream_vcs.rs scope: include tracked submodule changes in dirty determination (override local ignore policy as needed); if the complete check fails/times out under the existing bounded query budget, leave dirty absent rather than false. Add a real Git/harness regression for changed submodule worktree/gitlink, retain the Trace2/redaction/deadline protections, and demonstrate the old ignore-all behavior fails the assertion. No new budget, broad refactor, schema or source scope required. Rerun primary and relevant gates with fresh signed report/current-source evidence. Canonical KB baseline failure is separate and remains unwaived. Do not start WP3 from this candidate until corrected.'
  by: seat:01M6J5YFATHYCK148JT0C4XGZ1
provenance:
  digest: sha256:b3dcf41ac76ca66256a56c17121275d82ba6a98c05a23dff3710c882167cb9fd
  frontmatter_digest: sha256:611e0239e4e1badff0a3851f70f25db5aaf9dc7a96d2fc6bdb136f1f74f1394f
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: YIIRvwZlYgUuJABrAfpE/x2Lxn3I7YfESII1x6ZOxCOakR9lsSQn/mP6jlaUxgMWlMG3+MFUo2r/pWoh72YUAQ==
  log: .bboxy/log/2026-11-04.jsonl#43375
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

Oracle formal-inventory amendment 2026-11-03: verification/coverage.toml is additionally authorized ONLY to add the required src/vcs.rs source inventory entry and a precise implementation-coverage note naming actual WP2 tests and limits. Existing owned-process models/controls may remain unchanged only where their existing process helper/contract is reused; this amendment does not claim they prove Git parsing, snapshot consistency, bytes or wall-clock behavior. Do not silently map a new lifecycle/ownership/retry contract to an unrelated existing model: report that distinct gap if introduced. No broad inventory rewrite, model/control weakening, planned-source deletion unrelated to this WP, or gate change is authorized. Refresh current-source formal/parity evidence after the required new file is inventoried. Primary verify remains unchanged; maintain all canonical gates and disclose the existing KB failure without a waiver.

Oracle parser-provenance amendment 2026-11-03: authorize src/import/reproject.rs only to use the shared native Projector PARSER_REVISION constant defined in src/import/mod.rs instead of a divergent native-parser header calculation; one-shot import must use that same constant. The prepared base at a279e0f actually uses previous_revision.saturating_add(1), not hardcoded Some(2), so native reprojection must not invent a higher parser version merely because the same mapping ran again. Scope this consistency to native Projector provenance; do not silently relabel a distinct ACP mapper with the native constant. Preserve unrelated non-native behavior and report any distinct parser-version boundary rather than widening it.

Also authorize tests/ir_edge.rs only for the corresponding native parser-revision regression, alongside existing allowed import_native tests. Assert actual one-shot/reproject headers identify the current native mapping and repeated reprojection does not manufacture a new mapping revision; retain every existing immutable-source/prefix assertion. No VCS header promotion, replay-prefix rewrite, projection redesign or other reprojection behavior change is authorized. Run the unchanged primary verify plus cargo test --test ir_edge; refresh current-source evidence normally. All canonical acceptance gates remain required.


## Report from seat:01M6J7N5V103SG1KS0THEXB9R1

Implemented bounded session-start VCS capture, shared credential-free repo normalization, actual native field mapping and consistent native parser provenance in signed source da78cbda91b5de454c38152de8c44b5d587f2086 against prepared base a279e0fd; full report signed at a1c22c287a8d612c43e2bbc3606590fa72234f1b. Exact primary verify passes 23 tests, ir_edge 8, inherited IR/golden corpus 19; GUI/packaging/app smoke pass unattended. A demonstrated Git Trace2 environment-secret leak has a failing old-helper control (101) and passing engine+ACP regression. Fresh final-HEAD make test-formal test-formal-parity check-formal-evidence exits 0: 130 models/42 contracts, dirty=false, retained receipt SHA256 950b206fa5b191feb0f382d0a4e65c0887c9da0956f145d3dcf80e5b07ef7cd2. Canonical make test exits 2 only at unchanged duplicate KB number 5974 after 17 passing commands; no waiver/repair/merge/publication. Source/index are clean, five generated evidence files remain dirty; no WP1 worktree changes. Acceptance still requires the canonical KB gate to become green.

```json
{
  "op_id": "01M6JEY6P01CGA7DYJ3VQPJ4G3",
  "commit": "a1c22c287a8d612c43e2bbc3606590fa72234f1b",
  "report": "[[reports/2026-11-03-dream-wp2-vcs-r2]]",
  "verify_result": {
    "command": "cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent",
    "exit_code": 0,
    "evidence": [
      {
        "criterion": "Actual VCS capture, normalized repo key, no-repo/failure/dirty/native availability; no UI hot-path wait",
        "reference": "src/vcs.rs:45,92,150,176,210; src/engine/mod.rs:179; src/backend/acp_agent.rs:235; src/import/edge.rs:15; tests/dream_vcs.rs:140,176,196,211,254,274; tests/import_native.rs:872; /tmp/dream-wp2-verify-final.log — primary command exit 0, 23 tests"
      },
      {
        "criterion": "Native parser provenance and retained source/replay bytes, unknown data and actual transcript invariants",
        "reference": "src/import/mod.rs:16,950,961; src/import/reproject.rs:70; tests/ir_edge.rs:183,227,245 — cargo test --test ir_edge exit 0 (8 tests); make test-ir && cargo test --test protocol_golden --test ir_v1_corpus exit 0 (19 tests); tests/cli_headless.rs:66"
      },
      {
        "criterion": "Credential-free metadata/raw/ext and Git tracing negative control",
        "reference": "tests/import_native.rs:966; tests/dream_vcs.rs:211; /tmp/dream-wp2-trace-red.log — named VcsMetadataTraceDisabled assertion exit 101 with old helper; final primary exit 0 for engine and ACP; src/vcs.rs:99 clears inherited GIT_*"
      },
      {
        "criterion": "Fresh final-HEAD formal inventory, native/TLC parity and retained evidence",
        "reference": "make test-formal test-formal-parity check-formal-evidence — exit 0; /tmp/dream-wp2-formal-final.log; verification/evidence/950b206fa5b191feb0f382d0a4e65c0887c9da0956f145d3dcf80e5b07ef7cd2/evidence.json — source a1c22c287a8d612c43e2bbc3606590fa72234f1b, dirty=false, 130 models/42 contracts. Git parsing/snapshot/bytes/timing remain implementation-only; WP9 owns combined implementation mapping."
      },
      {
        "criterion": "Unattended unit/integration/GUI/application smoke and honest canonical baseline",
        "reference": "make test-gui test-app-packaging smoke-app — exit 0 (/tmp/dream-wp2-remaining-gates.log). make test — exit 2 after 17 successful commands; unchanged duplicate KB number 5974 (KB checker exit 1) in /tmp/dream-wp2-make-test-final.log and .bboxy/runs/test-current.json. No waiver or number repair; acceptance remains pending."
      },
      {
        "criterion": "Signed scoped commits, complete report, prepared base and worktree state",
        "reference": "Prepared base a279e0fdc5866068db567561e24ffa2bc0897d6b; signed source da78cbda91b5de454c38152de8c44b5d587f2086 and report a1c22c287a8d612c43e2bbc3606590fa72234f1b; git verify-commit both exit 0; reports/2026-11-03-dream-wp2-vcs-r2. Only five retained generated .bboxy/runs files dirty; source/index clean; no WP1 edits, merge or publication."
      }
    ]
  }
}
```

Oracle evidence correction 2026-11-03: the rejection reason prematurely described a successful harness CLI run and oracle-witness.json before checking the subprocess result. That first CLI attempt actually exited 1 with “session socket unavailable”; it produced no successful harness witness and must not be cited as one. The separate actual Git fixture exited 0 and proved that --ignore-submodules=all hides a modified tracked child while --ignore-submodules=none reports it; src/vcs.rs directly maps the hidden/empty output to dirty=false. This real-Git/source defect remains the basis for the in-scope correction and regression. The Oracle is reproducing the process witness with a valid short private socket location. This appended correction preserves the original audit trail; no prior failed probe is relabelled green.
