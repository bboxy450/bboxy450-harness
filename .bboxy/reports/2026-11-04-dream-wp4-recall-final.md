---
type: report
title: 'Dream WP4 final: audit, constructive seal and skipped-recall correction'
task: '[[01M6J8TV4EVP7GZG875FC5J4P5]]'
supersedes: '[[01M6KPQ72MNZQ3BJP4AN64MZNR]]'
commits:
- b48d44cd5123a9842dd96b25cdf1a1313c07810b
- 0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7
- 98ad494b79dd53b022158ca4eeb0e724a717e367
- 75486df6ad3082f8c145b2e54ea2171e26b37001
verify:
  command: cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T16:50:56.224Z
id: 01M6KQH16NND639THWAJAE4NQ2
updated: 2026-11-04T16:50:56.224Z
number: 20261103136493
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:3027d24552932d9916cc91ef15cba2b7cabce6be4f4f7b0198ec65b171100f2d
  frontmatter_digest: sha256:8f67c57c51a74ffda8e422b79814be024f930b7fd0b734b5076bd0620c40c383
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: EVubkMevsC1ix16YT28X44R1q+QbEjtVOKvjkFzo5Xwx9TJ8pmnEeQjvHCeiJAyh3UGpbgryIP4arDUQNviAAA==
  log: .bboxy/log/2026-11-04.jsonl#45210
---
# Dream WP4 final report

Source `75486df6ad3082f8c145b2e54ea2171e26b37001` implements the scoped WP4 assignment and its exact current verify command exited **0**: **41 primary tests, 138 native/Java graph/verdict comparisons, 45 contracts**. Fresh unattended native/app smoke also exited0. This is an isolated candidate handoff; canonical KB5974 still prevents acceptance/merge.

This append-only full report supersedes [[01M6KPQ72MNZQ3BJP4AN64MZNR]], including that report's premature implementation-complete claim. Final review found a missed successful-publication condition after its source/report gates passed. The earlier source0fea176 and report98ad494 really passed their stated40 tests and138-model checks, but those tests did not expose valid skipped publications incorrectly authorizing reported Recall and generated-line provenance. The correction below has an actual red/green audit witness. The earlier source-head receipt924d9d8b… and report-head parity/evidence at98ad494 remain historical, not final-source evidence.

## Task, commits, scope and authority

Current task is `01M6J8TV4EVP7GZG875FC5J4P5`, verify revision3, fetched etag `72739272d6e4452174bca2d40e4738fbbc51996e32b145de05b80199385a708e`. Worktree /Users/feral/mydev/bboxy450-harness-dream-wp4; branch dream/wp4; authenticated author seat01M6KDR90VJTRA1S722FZFECRZ. Governing text is dream-log/refinement/native retained-IR custody, reviewed V/T/E/R1–R3 and operative [[d20261103136471-dream-constructive-receipt-seal]] (record01M6KGTF6AYWC6NN1H3CY9FKMM, digest312e4cddcd6539f8a0feee517bc1dc7fce58b070c9875eb87399155b5595e133).

Original prepared base remains7d3e22cbc4442cee8ea8bdef6e23f5225708680c. Original pure sourceb48d44c and reports29afe26/388cd39 remain unchanged. The Oracle's checked carrier/tooling/decision integration base10c722781bcdaaaaac2fa0266f0787fdf93034f2 is read-only dependency source. Own constructive source0fea176 changes19 scoped source/test/fixture/mapping files against10c72278. Own correction75486df changes only audit.rs, receipt.rs, dream_audit_broken.rs, the good audit fixture helper and coverage.toml against report98ad494. No protocol/schema/frozen-corpus/frontend/tooling dependency or other worktree was edited. No rebase/reset, recruitment, merge, push, publication, policy bootstrap or real producer approval occurred. Source headers retain origin/license/absolute attribution policy; no new dependency/version/feature change was made in these phases (the prior authorized already-locked direct ICU2.4.0/ring0.17.14 exposure remains as reported).

Commits0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7,98ad494b79dd53b022158ca4eeb0e724a717e367 and75486df6ad3082f8c145b2e54ea2171e26b37001 are signed Boxy <bboxy450@gmail.com>; actual git verify-commit exits0. The full-report commit follows tested source; after committing it, final-HEAD native/Java parity and exact evidence validation are refreshed and linked in task_report. No receipt for another HEAD is transferred.

## Files and mechanisms

- **entry.rs / deny.rs / projection.rs / resolve.rs:** one NFC seven-field identity implementation; exact committed RecordPin/event byte resolution and captured authorized root/main membership; strict scope and optional-field bounds; shared deny/registered-value scanning; union identity/digest conflict rejection; per-harness maxima/global terminal fences; numeric qualifying trials; separate exact content and derived metadata. No clock/arrival authority or hand-body overwrite.
- **trust.rs:** exact pinned KB producer DSSE byte verification V, separately authorized external trust/2 T, scoped OpenPGP branch/last-touch/ancestry checks, private bounded verification processes and noncircular artifact approval. Source S and gate G precede signed full report R and appear in its commits. current_exe is measured once off the runtime thread, independent of recalled cwd. No policy/artifact/capability match means untrusted/producer_unapproved. Real producer bytes SHA25651e052c1f90d3322351848263d9ce1097a9f31e30860bf00d6bd95577c5f5a36 are byte evidence, never key authorization.
- **receipt.rs / seal.rs:** canonical bounded body/schema/builder; strict typed header/blob/body binding; exact retained LF-inclusive prefix and allowed suffix; dense E operation interpretation; causal cut/inventory/projection/rank/budget-fitting subset and generated bytes; committed pins and post-receipt Adoption bindings; foreign source-prefix closure; outer publication sealing observation; actual configured protected domains. A per-invocation try-lock admission owner refuses inflight/dropped work, active/unobserved children and late admission without a global queue. Actual Store-derived own-custody uses owned temp/write/fsync/no-replace link/directory fsync, verifies the retained own prefix and preserves conflicts. Unknown/missing proof is unproven; known effects/binding/suffix violations fail. No self-hash or final own-blob write is put inside its own frozen body.
- **audit.rs / graph.rs / mod.rs:** read-only P1–P9, audit/graph/why dispatch and finite provenance closure; shared renderer-bound sink:line nodes; old artifact presence is not proof. Current hand edits are preserved and labelled drift; terminal exposure outside a valid cut is labelled stale_exposure. The held Tokio stdout writer is flushed before explicit process exit, fixing actual empty-JSON transport loss. mod.rs only exports/dispatches.
- **Correction75486df:** a successfully validated **Skipped** publication remains a skipped diagnostic, including any known temp leak, and is excluded from the successful-publication set. It cannot authorize reported engine/ACP Recall or a materialized sink-line graph node; nonexistent skipped targets are not mislabelled as drift. Published/Idempotent fixtures still authorize matching pinned reported Recall. Serve-time currency/product insertion remains WP8's explicitly assigned W boundary.
- **Tests/fixtures and verification mappings:** good retained signed adopted/publication/idempotent/run/terminal/skip/Recall cases; named P1–P9 broken fixtures; canonical identity vectors; actual private Git/OpenPGP/KB policy chains, retained normal sessions,20 scripted verification subprocesses, real no-replace/fsync publication, exact source hash preservation, real CLI queries and delayed-child/ABA observer. coverage/contracts inventory actual APIs/process tests and honest pending product assumptions. Three finite seal models/controls and nonvacuity are added without weakening any existing contract or checker.

## Acceptance matrix

| Criterion | Source mechanism and executable evidence |
|---|---|
| P1 traceable finite closure, exact pins | audit::run, Resolver::record/evidence and projection::union; dream_identity fixed bytes/CRLF/root/main/nofollow cases, dream_graph conflicting raw identity/closure cases, actual P1 broken CLI exits1 |
| P2 publication, selection and pins | receipt::accept_with_publications/check_outer/cut_facts/check_publication; exact typed/blob/ops/cut/projection/rank/budget/generated/Adoption checks and all negative controls in dream_seal; sufficient retained publication/idempotence audit+why CLI0; new valid-skip/reported-Recall red/green control |
| P3 origin | audit::run requires known DreamRun trigger and MemoryWrite in dream session; named P3 fixture/API/CLI rejection |
| P4 append-only/EntryId | canonical identity and exact source prefix closure; actual Store own-custody conflict preservation; real delayed child WRITE/RESTORE observed through pipe, equal final bytes still produce actual P4 FAIL writer_touched_source; active/unobserved child remains unproven |
| P5 adopted means same-pair pass | Projection::build and qualifying_trial: resolving same-entry/harness trial, known thresholds, held-out/control/run/usage/spend gates; dream_graph cross-harness/revival/terminal/numeric negative cases and P5 broken fixture |
| P6 trusted anchor/instruction boundary | trust::verify_bytes plus independently scoped T/artifact/root authorization and audit closure; actual private signer/K2/revocation/root/policy/gate/report/executable controls; instruction sinks permit Proposal only; P6 broken fixture |
| P7 scope | exact committed root/repo/cross-repo opt-in; candidate sink scope derives from actual mounts/configuration, missing native scope unproven; dream_identity and named P7 fixture |
| P8 secrets | shared deny/registered-set checks over draft/record/rendered block and authored receipt, neutralized low-trust text and scrubbed CLI diagnostics; every pattern family/object key/registered value tested, P8 broken fixture never echoes the fake secret |
| P9 content/projection | exact content digest distinct from derived status/harness metadata; changed body preserved/rejected, never repaired by overwrite; stale projection separately reported; identity/body and P9 broken fixtures; sufficient terminal metadata fixture0 |
| Why/graph and additive substrate | exact cross-root names/links, optional strength read projection and actual receipt/pinned sink-line chains; real private audit/why CLI0 and unchanged source hashes; all15 old frozen JSONL bytes unchanged; real golden/corpus/fork/Unix/headless/unknown/blob regressions |
| Unattended bounded ownership | no new pool/global queue/operator step; try-lock owner plus real joined/reaped private child, bounded Git/public-key verification, own homes/sockets/processes, fresh ad-hoc app replay/hidden native startup/check-transcript0 |

The initial sufficient good fixture and implementation details are retained in [[reports/2026-11-04-dream-wp4-audit-seal]]. Its fixtures use isolated signed synthetic artifact policy to exercise consumer V/T/R and actual private process observations; its gate rows do **not** certify seven live producer witnesses or approve any live executable. Product effect wrappers, independent sink observer and live artifact gate remain WP8/9. No successful audit is inferred from current artifact presence, a self-key, equal before/after bytes or joined child rows alone.

## Formal and process evidence

Before introducing Owner, the new finite model/nonvacuity command exited0; /tmp/dream-wp4-seal-model-first.log SHA256f4add0ae054b191bf049bd7ed000ceee275acaa3d8e2f8ad973113e1acc24a8c. State: one producer; absent/active/joined/reaped child; unobserved/observed/enforced coverage; recording/frozen/sealed/closed phase; one inflight effect; spawned/late-protected/custody bits. Atomic admit/complete/spawn/observe/join/reap/freeze/seal/close actions enforce NoProtectedEffectsAfterFreeze, FrozenOwnsNoMutableChild and FreezeHasNoInflightEffect. No arbitrary-IO fairness/liveness claim. Nonvacuity explores useful closed/observed-child states, refuses unobserved freeze and reaches both actual controls.

Full native/Java parity at final source has HarnessDreamSeal pass35states/61edges; BrokenHarnessDreamSealChild expected safety59/100 and TLC12; BrokenHarnessDreamSealLateWrite expected safety50/101 and TLC12. All138 model reachable-state sets, labeled edges and verdicts agree across native and pinned Java TLC. inventory is45contracts/1crate; nothing omitted to pass. Finite parity is not proof of hashes/crypto/syscalls/arbitrary Rust/OS behavior. Truthful approved producer completeness and actual confinement/durability remain explicit W assumptions with API/process witnesses; missing observations remain unproven. Native loader/consolidation/forgetting is excluded.

The real private delayed child is explicitly invoked by its parent despite its ordinary ignored listing. It has an exact recorded PID, closed environment, bounded readiness/exit and join/reap. The safe active child prevents freeze; earlier unobserved work also prevents proof. The deliberately broken source-write exception performs real ABA and independently observed WRITE/RESTORE, producing P4 FAIL despite equal final source bytes. Pure R4 controls additionally exercise forbidden detached suffix, header/item/count/prefix/coverage/child/cut/outer/source and generated-selection bindings.

## Latest verification and red/green correction

Exact primary:
```
cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
```
**Exit0 at75486df**, /tmp/dream-wp4-recall-final-primary.log SHA256942a1cb7c6c62a3bf63cbbc4ed36aa299bda71a7e66316e7fc54ffba83cf6cc8. **41 primary tests**: audit5,graph8,identity13,seal7 plus invoked private child,trust8; native formal tests4/nonvacuity32/reactor1 and full138-model/45-contract parity. Collected commands/statuses /tmp/dream-wp4-recall-final-verify.json SHA2562551da30dd37e974ece27066ea5a5f34217f521cf76be6d404aeb43d3d04ebce.

Current-source evidence receipt **0b87ddddeb964d48ce1b8489483dd0e63c7bcfceca869bff34f4eb6d2a794f5e**, source75486df6ad3082f8c145b2e54ea2171e26b37001,dirty=false,138models/45contracts/1crate, retained under verification/evidence/<digest>/evidence.json and target/formal/evidence.json. Exact make check-formal-evidence exited0. Final-report HEAD gets a fresh parity/evidence receipt after this report commit, supplied by task_report; no old receipt certifies it.

The new actual API test `cargo test --offline --locked --test dream_audit_broken reported_recall_requires_successful_publication_not_a_valid_skip`:
- **Red101** before correction: a valid skipped retained fixture audited0; adding reported Recall incorrectly still audited0 rather than expected1. /tmp/dream-wp4-recall-skip-red.log SHA2562d1fd7159891b21cd5c6957b70bfb16878ea4018011737f9caf26e6c3d09c53b.
- **Green0** after correction: real Published/Idempotent Recall accepted0; valid skip alone accepted0 with skipped label/no generated-line node; reported Recall from skip rejected P2 recall_publication_binding, exit1. /tmp/dream-wp4-recall-skip-green.log SHA25645e3389857155d3f0252fd6e98e8fac25a6563071f0cdc2025988ee8c9f7ba0f. The final complete primary reruns the control.

`cargo clippy --offline --locked --all-targets --all-features -- -D warnings` exited0 on the final source bytes before commit; /tmp/dream-wp4-recall-final-clippy.log SHA2569349b0cf53587473fafee6187ec2e6d80387cbb0d124b2025f69d89beea0b6ca. git diff --check and source equality to the signed source commit exited0.

Fresh **make smoke-app exit0** at75486df, /tmp/dream-wp4-recall-final-smoke.log SHA2567317ba7f9defa4792038160b1273cea6e5a3bf1338aa1a6a0014b6c66a432fad. Actual target/app-smoke/evidence.json SHA2563bab96ffb9fbdd50673cd11fb13cffafc59c40560d1cc0fd160db050ebcf3e12 names75486df, private_home=true/operator_input=false and replay/native launcher/codesign/check-transcript exits0. Actual release/ad-hoc bundle, PNG replay, hidden NSWindow/Metal surface/private live frame and exact owned process exit/reap were exercised. No desktop input/capture, visible-presentation claim, Apple signing identity, Keychain prompt or credentials.

Additional complete substrate checks at the unchanged constructive source0fea176 remain explicitly historical: fmt0, all-feature clippy0, make test-ir0,33library units0,50main replay/IR/edge/golden/fork regressions plus3 invoked private children0, headers0,secrets0,3protocol_laws0. Logs/statuses/hashes are in the superseded full report and /tmp/dream-wp4-seal-additional.json (SHA256212f6ab89d333740531dca78375759cd19f8b7fc6799e58af23780dec54ae5e3). These are not represented as new runs after75486df. All15 prior frozen fixtures remain byte-identical to7d3e22cb (/tmp/dream-wp4-seal-old-corpus.json SHA256c2c657284a5b22bce8f0a43acc88dd5c22791bfa7db6a7ae1786f82fd65cbafb); this correction changes no substrate source. Actual check_transcript means protocol::laws::check_transcript/check_authored in API/replay/corpus and the fresh bundled CLI checker, never an invented shell target.

## Baseline, failures and remaining limits

The prior checkpoint's missing Terminal head/missing seal/outer observation/async stdout and compile/lint failures were corrected and disclosed in its reports. Its changed-upstream-pin formal exit2 was resolved only by the Oracle's checked dependency integration; no WP4 pin/checker bypass occurred. The earlier40-test completion claim is superseded because of the independently constructed skip counterexample above; red101 is retained, not counted as a pass.

Canonical **make test is not green**. Per the signed task's no-unchanged-red-loop ruling, WP4 did not rerun/filter/waive it. Retained actual /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3 records make2 after kb-records1: duplicate KB5974 in d5974-log-ir-v1-only-add.md and d5974-memories-are-kb-records.md. Both files remain byte-identical, hashes28f3442ce7dbfa406518135828bf5f063f1b06b573f3fa2b7987ef4c832ba450 and d2eb97c855304806f9da7abfaa6ff6b2d003a1320aea582982fbe963c378a4d4. Original mirror conflicts and this canonical blocker remain unwaived.

No fresh full all-target/default+ACP/GUI/packaging canonical gate is claimed after the final correction. Fresh exact assigned verification, all-feature compile/lint and native/app smoke are green; combined-main full gates, real trial/sink producers/serve-time insertion, independent observer/live executable gates and native loader state remain the assigned downstream/Oracle responsibilities. No operator step, live approval or merge is introduced. Source is clean; only the required tracked generated formal-native.json remains modified and retained, with ignored source-bound evidence. The Executor reports a verified isolated implementation; the Oracle independently verifies and accepts, and the unwaived canonical/full gate still prevents final swarm acceptance.
