---
type: report
title: Oracle verification of corrected Dream WP2 candidate
task: '[[01M6J8TT7HY4B7YPAP8SEWBE40]]'
commits:
- 8b4be671e2955b59edd53ff55f8e1645ec4f146f
- 1dc87fb9a329ed0a152efcb685d30db9f32a37df
verify:
  command: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
  exit_code: 0
author: seat:01M6J5YFATHYCK148JT0C4XGZ1
created: 2026-11-04T05:36:40.998Z
id: 01M6JGYE7768KMY97SC49NVXNC
updated: 2026-11-04T05:36:40.998Z
number: 20261103136340
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:9dcb8e0c108295eb0687c7771e9046352c49e02fd2d2e574950243c5ad1c1fbc
  frontmatter_digest: sha256:4b8cabe796cbe8d592b6fdb1a67824e4d99cb44fb1e45ceec9b83df38128e816
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: txu2RZepQVaXW0g5/GeylnKcjdo+qEMwSibvuuZ2xNRks3EOQzVb0J41atx7+wOO0PAZ/3ZSaSq/AJcth8pcAg==
  log: .bboxy/log/2026-11-04.jsonl#43524
---
The corrected WP2 candidate is independently checked for downstream development. This is not task acceptance or implementation publication: canonical make test remains red at unchanged duplicate KB number 5974. WP3 is delegated on the signed isolated candidate under [[d20261103136319-dream-checked-candidate-dependencies]].

Source inspected: src/vcs.rs:192 forces --ignore-submodules=none. tests/dream_vcs.rs:205 exercises signed actual parent/child Git repositories with both local and .gitmodules ignore=all, changed tracked child content and changed gitlink; API, engine, ACP and persisted headers must report dirty. Failed/capped/stalled status cases retain known metadata and omit dirty; the three-second query/capture budget and owned-process cleanup remain unchanged. Production correction changes only the status flag and explanatory comment; tests retain credential/Trace2 and authored transcript checks.

The original Oracle actual-harness counterexample was rerun against the corrected binary, with the original signed Git fixture retained at /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-vcs-submodule-nvc9pgy_. oracle-fixed-witness.json records program exit 0, observed_dirty=true, expected_dirty=true, source 1dc87fb9a329ed0a152efcb685d30db9f32a37df and unchanged binary SHA256 9817abac5e2e4701558d825e31312afd9ac227d241e759d4ee20a134a088d642. The earlier first CLI attempt failed at socket startup and was explicitly corrected in the signed task record before the subsequent successful old-source dirty=false witness; it is not counted as successful evidence.

Independent command: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent && cargo test --test ir_edge && make check-formal-evidence — exit 0. Primary suite: 25 tests; ir_edge: 8 tests. Log /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-wp2-fixed-rkav2pmo/verify.log SHA256 301a7e61a84219092158e7785dac0bce64d76cf80f2646f3e7d23fd13564fbb2; result.json retained beside it. Formal receipt source is exact report HEAD 1dc87fb9a329ed0a152efcb685d30db9f32a37df, dirty=false, 130 models/42 contracts, evidence SHA256 08b5ba6ca1fa1d9f761e7dc2fd41bd1428c81eb32e0bdca9e47d715ded1efd69. This review revalidated the receipt; the earlier independent WP5 review ran the complete native/TLC comparison. It does not claim another full parity rerun here.

git verify-commit 8b4be671e2955b59edd53ff55f8e1645ec4f146f and 1dc87fb9a329ed0a152efcb685d30db9f32a37df — both exit 0, approved Boxy key 9B07E849480C68328CA20684844ECC0587DE7AB0. Read the append-only correction report, including its disclosed overlapping feature-build GUI failure and serialized successful rerun. GUI, full canonical make test and all parity were not independently rerun in this correction review. Retained worker logs identify the unchanged duplicate5974 canonical failure; there is no waiver.

WP2 source/index are clean; its five generated run evidence files remain dirty and retained. No cleanup touched those files. WP3 prepared worktree /Users/feral/mydev/bboxy450-harness-dream-wp3 is based on origin/main bcb7247ed534b7e08ac64c7cea740d1ede51e8d6 with verified signed WP1/WP5/corrected-WP2 cherry-picks, clean prepared HEAD 92521aa93ebcb8c20daab84fa92008dbd298ca1f. Its live brief and delegation are signed separately. Old receipts do not transfer across these cherry-picks; final current-source gates remain required.

Known limits: sequential Git observations are not an atomic changing-checkout snapshot; finite process models do not prove arbitrary Git/filesystem behavior. Original companion mirror repair and canonical KB acceptance are pending the authenticated existing-identity host capability documented in [[d20261103136278-task-import-cannot-repair-existing-number-collisions]]. No submodule update in term or swarm shutdown has occurred; both remain after accepted implementation.
