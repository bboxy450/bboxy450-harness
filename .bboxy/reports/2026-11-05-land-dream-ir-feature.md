---
type: report
title: 'Dream IR 1.3 first feature landing: additive current-main integration and finite compatibility evidence'
task: '[[01M6MHTMXRVY2AN6EKWM8T6B5Z]]'
commits:
- ed3a911ca0f2b55e6be0be9e719d73ef3b4a6479
- 54a80508ec22bab5016009869a7c3e56438fb1a2
- 8bf16d5c4fd4785764bbfd04d4ba1c34b57559f3
- 4786fd837c1ebc873d7c8089c4b529c24b4167be
verify:
  command: make test-ir && cargo test --test protocol_golden --test ir_v1_corpus --test store_fork && cargo clippy --all-targets --all-features -- -D warnings && cargo test --features acp-agent --test acp_agent steering_ && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
  exit_code: 0
author: seat:01M6KDR96J84HR56MCXQ2JSW5Y
created: 2026-11-05T01:10:11.194Z
source: 4786fd837c1ebc873d7c8089c4b529c24b4167be
verify_revision: 1
id: 01M6MM363ZTA784ZCAQMD88E4H
updated: 2026-11-05T01:10:11.194Z
number: 20261103136625
created_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
updated_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
authorship:
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  kind: seat
  public_key: +go7EGHCYyMM/7hLFRzzAOFaKqWCHKAaxEdDnpzj8+I=
  process_generation: '1'
provenance:
  digest: sha256:2ca4f75b785e829fa498bcc5f0c5521850cc361b971d73e07766c1feb9819ab1
  frontmatter_digest: sha256:86cb394e8b2b75dd9ef041313130d6c7042cbfd945e56100fe56ee9ae5a53768
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  sig: KhBrYoBkyL5ZMujGhlhsG5FUyoPXFe1+p/66t6t22jRrb7hjOgUvVs96G9v5nbWPBClPmbXTtIWzPqIwsy2yDg==
  log: .bboxy/log/2026-11-05.jsonl#46551
---
Prepared the first bounded Dream IR 1.3 feature on current main. Exact task verification revision 1 exited 0 at source `4786fd837c1ebc873d7c8089c4b529c24b4167be`. All 135 native/Java reachable-state, action-labeled-transition and verdict comparisons match (45 contracts, one crate; dirty=false). This is a source/evidence handoff for Oracle's PR28; no merge, push, publication, Receipt carrier or Dream runtime was performed by this Executor.

The assigned writing checkout/cache is `/Users/feral/mydev/bboxy450-harness-dream-wp6`, branch `feature/dream-ir13-land-20261105`. Its previous `dream/wp6` branch remains exactly `c98d185bc2182f3a37ceee936031081f8c7cb2bd`. Before switching, tracked source and index were clean except the known generated formal-native output. Its exact bytes were copied to `.bboxy/runs/feature-land-ir-wp6-retained-native.json`, SHA256 `f50a38f1dd4dc8177b4c1bb722d03f524065554b7bc9acdf6fe0e9fe984aec11`, equality asserted, and only the original generated path restored. This Executor removed no other reports, evidence or caches. The signed prior branch/report and exact generated output remain retained. See feature-land-ir-preservation.json and feature-land-ir-base.json in .bboxy/runs.

Fetched actual base: `0ab55023228a7f7f49305c9c08f5bdbcfb39c10f`, the signed next main commit after observed `baf77427df00ce99d4a9a392ebc8bef7e1b5478c`. Its existing signature verifies (historical key B9ABDC578390946B71B6C20C3526057098BAC1AD); no old identity was adopted. Its 14-file delta supplies current steering, transcript pane, GUI and model work. The branch preserves that work. Original WP1 source `0c53f325b1f88a1efc720401b89c49767eaa12e0` verifies with approved Boxy key `9B07E849480C68328CA20684844ECC0587DE7AB0` and was applied with cherry-pick -x and signing, never a whole old-file checkout.

| New signed commit | Mechanism |
|---|---|
| ed3a911ca0f2b55e6be0be9e719d73ef3b4a6479 | Original additive WP1 patch with its cherry-pick origin trailer. All automatic merges succeeded without textual conflicts. Protocol/schema/lock union retains main's newer fields and variants. |
| 54a80508ec22bab5016009869a7c3e56438fb1a2 | Only missing vcs=None/dream=None in current GUI motion SessionMeta literal. |
| 8bf16d5c4fd4785764bbfd04d4ba1c34b57559f3 | Explicit Oracle amendment: four unavailable model conditionals lowered without changing their finite relation; exhaustive equivalence witness; identical-value GUI test initializer; four raw upstream pin files refreshed after actual source-pin-only failure. |
| 4786fd837c1ebc873d7c8089c4b529c24b4167be | Explicit second amendment: only HarnessAcpSteering checks normalized to mandatory pair; dedicated targets retained as additional_checks. |

Every new commit's author/committer is Boxy <bboxy450@gmail.com>, with the approved key above and Report trailer. Oracle stated it had already pushed source54a8050/opened PR28; later commits were added locally without rewriting those commits or pushing.

Acceptance mechanisms and evidence:

- `src/protocol/mod.rs:22,40,682,792`: passive EntryId/Digest aliases, DreamRun/Vcs/Scope/pin/evidence/trial/sink supporting types, five Item/ItemKind variants and optional SessionMeta fields. PROTOCOL is 1.3. No Receipt item or runtime implementation is included. Existing types/defaults/tags remain. Absent optional SessionMeta additions are omitted; supporting fields have reader defaults that do not mint identity or imply authority.
- `src/protocol/laws.rs:363,413,502`: recursive Unknown refusal for authored values and structural presence/default checks. These do not resolve commits, qualify trials, compute canonical EntryIds, authorize a sink or audit semantics. `src/protocol/mod.rs:1374,1665` preserves known interrupted kinds and supplies passive JSON text.
- `tests/protocol_golden.rs:535,611,640` and `tests/ir_v1_corpus.rs:396,443,599,628,652,704`: known/future/missing values, nested unknown payloads, incomplete-reader versus authored-writer controls, trial usage positions and exact digest-line custody.
- `tests/ir_v1_corpus.rs:58,510`: actual Store open/closed replay/fork/Unix/headless and check-transcript witnesses retain stored bytes and future fields. `tests/store_fork.rs` retains its four actual API witnesses. These are byte witnesses, not a mathematical proof of serialization.
- `src/frontends/gui/face_rows.rs:729`, `src/frontends/gui/view.rs:2839`, `src/frontends/seat.rs:2005` and ACP arms only display the passive items. `tests/gui_bridge.rs:149` derives fixture coverage from the frozen lock; all 15 fixtures pass actual Unix/InProc view comparison. GUI interactions, attachment/onboarding/agent configuration code and existing tests are preserved.
- All 117 existing files under tests/fixtures, including all 12 old frozen IR JSONL files and GUI/golden assets, are byte-identical to actual base. Only v1.3-dream-session.jsonl, v1.3-recall.jsonl and v1.3-vcs.jsonl were added; they carry embedded origin/policy text. `feature-land-ir-all-fixtures-preserved.json` records every old hash. Schema/golden generation equality and additive schema/lock controls pass. Main LICENSE, ATTRIBUTION.md, dependencies, agent_config.rs and all unrelated model families remain unchanged.
- `scripts/ir-compat.py` reads actual PROTOCOL for lock version and checks current lock plus deliberately broken removals/renames/type/required/unknown/corpus controls. No compatibility bypass or stale whole schema replacement was used.

Oracle-authorized model compatibility is limited to `tests/formal/models.rs:4346,4395,4455`. Variables, bounds, initial values, invariants, responses, deadlock flags and all registrations/verdicts are retained. Split branches keep the same human action label; generated TLA actions continue to use indices.

| Family | Original index -> lowered indices | Fairness |
|---|---|---|
| AttachmentIntake, good/broken | Complete 3 -> 3,4, partitioned by current versus stale birth/draft | WF([3]) -> WF([3,4]), one disjunctive group |
| AttachmentCapability, good/broken and Discovery control | LiveResponse 2 -> 2,3; SendImage 3 -> 4,5; SendText 4 -> 6; DiscoveryCompletionIgnored 5 -> 7; extra Discovery control 6 -> 8 | None before or after |
| OnboardingInstall, good/broken | Click 0 -> 0,1; Cancel 1 -> 2; Reap 2 -> 3; DelayedComplete 3 -> 4 | WF(Reap [2]) -> WF([3]) |

`tests/formal_models.rs:163` exhaustively checks all 2,688 declared states, including unreachable combinations and both branches in good/broken/Discovery modes, against the original Rust integer conditional math. It asserts 6,256 successor effects with original human labels, disjoint branches, exact variable metadata and fairness groups. No new general model framework, dependency patch or model family was added. Other family bytes before the affected three-function region and all existing formal tests are byte-identical; feature-land-ir-compatibility-preservation.json records that boundary. All 135 canonical model graphs/verdicts subsequently match pinned Java TLC. Finite equivalence/parity does not prove arbitrary Rust refinement or complete TLC replacement.

`verification/contracts.toml:17` now has checks=[make test-formal,make test], preserving the original make test-acp-steering and make test-acp-steering-formal names in additional_checks. No steering guard/property/fairness/bound/registration changed. Actual revision-1 API command runs three steering cases (running Codex/Claude without cancellation; fallback only for explicitly unconsumed input; unknown result retained without replay), all exit 0. The dedicated release-profile targets themselves were not run or claimed.

Raw upstream source-pin refresh is from signed immutable term main `2b1f9c3883363555b308cdf7f38fc916797f0e53` (approved Boxy signature verified). Each snapshot's six-line local attribution prefix is preserved; remaining bytes equal the exact Git object and live main. Only the three existing snapshots plus upstream-sources.json changed.

| Raw source | Old SHA256 | New SHA256 |
|---|---|---|
| scripts/formal.py | 7ac68f32996c5a3ec051dce6886a2419431a264f763924d36796d3d2b401100d | 09ca7496419d59f0a41245cff8297b7fdb79dba2a9b81abf5df9e4a6597aa937 |
| scripts/test-formal.py | 808ceaff7994903925defd8b08ac45eaeecec7d46d7d6ee7d711901d6571a7b3 | 7627077e427cecf992e00bba1c0ed55f13843f897502f54545cd2e5dd9a4aa85 |
| crates/bboxy450-tlc/examples/verify.rs | 88b6400d6a883b25f2eca777cb936ba568f9e9ef702f51aef4f7b83e022c8fb8 | f45b999db8be8da4a18afd36ef24558c9654f9d58d642d5c30bb1ff7abbdc278 |

Review is retained in feature-land-ir-upstream-review.json/.diff. Upstream changes include workspace composition, additional checker constructs, private JVM extraction ownership and evidence reuse/rebinding. They were reviewed as raw reference changes; the harness's adapted executable checker/exporter and dependencies remain byte-identical to main. No upstream runtime feature, evidence reuse/rebinding or new Expr support is claimed.

Actual verification at source4786fd8:

```sh
make test-ir && cargo test --test protocol_golden --test ir_v1_corpus --test store_fork && cargo clippy --all-targets --all-features -- -D warnings && cargo test --features acp-agent --test acp_agent steering_ && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
```

Exit 0; `.bboxy/runs/feature-land-ir-revision1-source-primary.log` SHA256 `0c89269cbf33d7a01713af7f1d5f5cc6870a5d542cddeca2620177e034b20732`. IR corpus 11, golden 8, fork 4, steering API 3, formal model tests 10, nonvacuity 26, reactor projection 1 and formal infrastructure 8 all pass. Clippy all targets/all features passes. All native/Java checks ran freshly with no reuse. Source formal receipt is `eea0220b71085526e96e6e0ba6b18803e06210c516fff48cd6b3952a919fae46`, retained at verification/evidence/<receipt>/evidence.json (source4786fd8, dirty=false, 135 models/45 contracts/one crate). feature-land-ir-source-receipt.json records exact source and smoke evidence.

Actual make smoke-app passes header/license checks, release bundle/plist, strict ad-hoc codesign, Metal replay PNG (21 visible items/14 kinds/readable payloads/three approval buttons), hidden native window/surface/live frame, child reap and the real check-transcript subprocess. Smoke JSON records transcript_exit_code=0, replay_exit_code=0 and launcher_exit_code=0. No operator desktop input, Keychain, physical probe or credentials were used. Visible presentation was not tested. Builds/features were serialized in this assigned target, lean debug profiles/offline/two Cargo jobs, with private HOME/TMPDIR outside a Git root.

Supplemental retained checks: all-feature Unix/InProc GUI bridge 11 pass; GUI motion 10 and usage report 6 pass; named unchanged-value header fixture passes. The earlier --lib protocol:: invocation selected zero tests and is compilation evidence only.

Failures and limits remain retained with their original exits, never relabeled green:

| Retained log (.bboxy/runs/feature-land-ir- prefix) | Actual result/disposition |
|---|---|
| baseline-clippy.log; initial-2.log | Baseline main/feature clippy101; header fixture field_reassign_with_default. Approved initializer-only fix passes current clippy. |
| baseline-formal.log; composed-0.log | Baseline main/feature make2: four unavailable Ite/.ite uses with published TLC0.1.1. Approved guarded lowering and equivalence proof fix it without Cargo changes. |
| conditional-lowering.log | Development witness compiler101 (missing closure parameter type); explicit &str added before signed compatibility commit; lowering-fixed.log exits0. |
| compat-formal.log | make2 solely from upstream raw pin drift, after model tests passed; exact reviewed four-file refresh above. |
| source-primary.log | Exact old verify make2: unchanged HarnessAcpSteering malformed mandatory checks declaration. Explicit amendment fixed only metadata and added actual steering API verify. |
| composed-2.log; baseline-gui-timer.log | Extra GUI chrome test fails101 on both feature and signed base: zoom_pill_shows_the_percent_then_fades expects a positive timed wake, receives Some(0.0). Its test/behavior is unchanged; Oracle explicitly records it as inherited and excludes further GUI repair from scope. Six other chrome cases pass. |
| initial/composed source54a8050 logs | Retain original green IR/corpus/fork and smoke/check-transcript results for that exact source; they do not certify later source by themselves. |

The historical KB5974 red premise is already changed on chosen main: signed main afe92dae97b2a5f86209863fd1545d7208171b9b changed the IR record number to 93377961361017; the memory record stays 5974. Normal current-source `python3 -I scripts/checks.py kb-records` exits0 (2,925 records), log SHA256 `2bbf6f2d3810934621decbf3b1fc7524de474ccade544a266295edbcf9dcf1e9`. This feature performed no renumber/repair/filter/waiver. Historical red evidence remains historical. No full make test loop was run; this report claims the exact bounded feature gates above, not a green full suite or the inherited GUI timer.

Before report creation, external build-output cleanup was observed: target/feature-land-ir, target/formal and target/wp6-checks disappeared, and available disk increased from 8.3 GiB to 86 GiB. No deletion was issued by this Executor; the actor/cause is unproven. All source verification logs had already been copied byte-exact into .bboxy/runs/feature-land-ir-*, and the source formal archive remains retained. Prior WP6 target-only logs are no longer available; its signed branch/report and durable formal archive remain. feature-land-ir-target-observation.json records the actual retained state. Final verification will use only the assigned target and will not recreate other worktrees' caches.

At this report's source snapshot, tracked implementation/index are clean and only generated formal-native.json is modified; task-owned evidence files are retained separately. This report and its evidence are to be committed with the Report trailer. Committing the report necessarily advances HEAD: the unchanged revision-1 exact command will then be rerun at that final HEAD, generating a fresh native/Java archive and `.bboxy/runs/feature-land-ir-final-head.json` plus final-head.log. Those actual final-HEAD hashes/receipt and signed report commit are supplied in task_report; the source4786fd8 receipt above is not rebound or copied as final-HEAD proof. No required final-HEAD pass is asserted in advance.
