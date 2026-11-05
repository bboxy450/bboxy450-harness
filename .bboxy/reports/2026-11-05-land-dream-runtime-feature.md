---
type: report
title: Accepted Dream runtime feature integration and current-source evidence
task: '[[01M6MQS2G3E5PNCKWF4J6DMXJ4]]'
commits:
- c59957a03ebbee9dda19d9db0c5ae675efc5c04b
- 58d6cd4c519e3020cf807a7d5c8548b80573f156
- 3427d9b0d6434b4796dc4f1b030d23bb7f6f283d
- cdb8fffc9c823b7de429a47da34c051211e7de5b
- 01a220cbe5258fccffb2f0de43741dc253f10ab7
- dc4bc5a9a0338c4e57bb96dcd351b659aa80b69c
- 9fd64b85cdcd19436271f6f1901a8b51218be2bd
- 53fa22992ccce0d8afb054616ea0a429087db4c7
- 136b81b50f4700aece03995cfd02d50f915bcb3a
- 32db54422cb7e9c8fde2e9d0900b616d7dc8f322
- 8c12cc5f1a5402bc01076f4f29572307ed664282
- 35953259bbd72cef2ac1b1de749d40ca9e7170e0
- c3638b5f82057f9ef3d3da149ca3d2fd0625768b
- a9fda8508c50932c0ab94ca8dfe1f0005f947ef4
- 611d6e17d4cadc91c7e4443ce2fc8f9fe5d77287
- d028761059d6a4ba45525463a9720dffa283a245
- eb0eb5b478a5dd70342d5b86887e61b170bdf85f
- 911904b4200947ee0db2aa437ed7b6d5074deade
- 7829ab66b037560ac8a90c6da13bb9fd7a2cfac5
- 15decd4395ed453767896e15d14ac1bd5184fd93
verify:
  command: 'cargo test --features acp --test dream_vcs && cargo test --lib tools:: && cargo test --lib process:: && cargo test --lib dream:: && cargo test --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_replay --test dream_pipeline --test cancel_process --test store_fork --test store_recovery --test policy_gate && make test-ir && cargo test --test import_native --test ir_edge --test protocol_golden --test ir_v1_corpus && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app'
  exit_code: 0
author: seat:01M6KDR96J84HR56MCXQ2JSW5Y
created: 2026-11-05T03:18:16.527Z
id: 01M6MVDQCNH513W058B4J2A317
updated: 2026-11-05T03:18:16.527Z
number: 20261103140050
created_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
updated_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
authorship:
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  kind: seat
  public_key: +go7EGHCYyMM/7hLFRzzAOFaKqWCHKAaxEdDnpzj8+I=
  process_generation: '1'
provenance:
  digest: sha256:7756a47e91b28b962b48c5ec2ff7bd3962d7a48c1cc806939a882cd2172dd955
  frontmatter_digest: sha256:0c72288b04102c6f4229f1501a0183d74a190859cfc8c4377f2016395a11f590
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  sig: laldoKWTNrxv4wLVc7laH4vzv5sl+IIaBC+OYpRJ7hd7z4p5R8GQtp6iDXg8JXaSR3EADIyjMMMHvrwJCKHkAQ==
  log: .bboxy/log/2026-11-05.jsonl#57445
---
Integrated the accepted runtime on `feature/dream-runtime-land-20261105`, using only the assigned WP6 writing checkout/cache. Finished source is `15decd4395ed453767896e15d14ac1bd5184fd93`. The exact twelve-part feature command exited0 at `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5`; composed default and ACP regressions, the amended capability fixtures, seven separately executed seat-launch cases and the nonPTY case all exited0. The last change pins only the inherited seat fixture's profile. Its signed task amendment expressly retains the prior exact/composed passes and requires affected cases plus fresh source/report-HEAD evidence. Current-source native/Java verification is0 at the finished source, with192 models,60 contracts and1 crate. This is a feature preparation handoff; no push, main merge, publication, release, live inference, producer approval or full Dream completion occurred.

Project origin and attached credit remain https://github.com/bboxy450/bboxy450-harness and its [attribution policy](https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md).

## Authority, base and lineage

Task [[01M6MQS2G3E5PNCKWF4J6DMXJ4]]. Current task etag `21c72d5e12e41ece6ac8fe30bb851f4cc7a05ac4289acdf21f434798e18b7d7c`, signed Oracle amendments for the exact constructive-seal record and two fixture-profile pins. The verify string remained unchanged. No worker task/spec amendment was made.

Initial checkout was clean in source/index except generated formal-native.json, at signed main/landing `20f1844e60694784a985e81b7cceb2dbdc04c525`. Fetch/recheck returned that same main; this is the actual base. Main's IR PR28, Receipt PR29 and VCS PR30 were retained rather than reimported. All seven prior feature/landing/WP6 refs and seven pre-existing untracked evidence files remain hash-identical. The136-model generated native JSON was copied byte-exact before narrowly restoring only that path to switch branches: retained SHA256 `ca44b518a0f3a208b4597838ef6026b64b97cd3897640c1f6d84efdb22e1e687`. No clean/reset, peer checkout/target cleanup or new writing worktree was used.

The17 authorized source commits were applied in dependency order with signed commits, original-source and cherry-pick lineage, and this report's trailer. Existing report trailers became Original-Report references.

| Order | Original signed implementation | New signed integration |
|---|---|---|
| 1 | `cbe16da81b60c29f760f19e37d0c1430a0e4b7b6` | `c59957a03ebbee9dda19d9db0c5ae675efc5c04b` |
| 2 | `3757dceed3b260134686521eb8f95ee2e7a0ccb8` | `58d6cd4c519e3020cf807a7d5c8548b80573f156` |
| 3 | `90e9c0bbeda6c3b8ebdcbcbd4dfda2b32a356726` | `3427d9b0d6434b4796dc4f1b030d23bb7f6f283d` |
| 4 | `b48d44cd5123a9842dd96b25cdf1a1313c07810b` | `cdb8fffc9c823b7de429a47da34c051211e7de5b` |
| 5 | `0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7` | `01a220cbe5258fccffb2f0de43741dc253f10ab7` |
| 6 | `75486df6ad3082f8c145b2e54ea2171e26b37001` | `dc4bc5a9a0338c4e57bb96dcd351b659aa80b69c` |
| 7 | `d3aab3b08bb0be24518b66b44fed83d9209d61e5` | `9fd64b85cdcd19436271f6f1901a8b51218be2bd` |
| 8 | `fd64540610168af18f0fb1a09c55b6337679b106` | `53fa22992ccce0d8afb054616ea0a429087db4c7` |
| 9 | `c1085e903b375966535bb35bb70eb3f1a87e3543` | `136b81b50f4700aece03995cfd02d50f915bcb3a` |
| 10 | `012ff149c2f1a70d9614c8611366984893376f02` | `32db54422cb7e9c8fde2e9d0900b616d7dc8f322` |
| 11 | `9fb4b1111b6e524a7c6de070156b33856ef87b1e` | `8c12cc5f1a5402bc01076f4f29572307ed664282` |
| 12 | `13bad329445bfc19ece978dc6b43236fbcbca078` | `35953259bbd72cef2ac1b1de749d40ca9e7170e0` |
| 13 | `1f2b6c7de0a5f8709816a69b98920f550f783907` | `c3638b5f82057f9ef3d3da149ca3d2fd0625768b` |
| 14 | `fe1c4e89491d83a7b5c144fc994e0e1fcdefafb8` | `a9fda8508c50932c0ab94ca8dfe1f0005f947ef4` |
| 15 | `24525ba6c586d811a97540278223e967ce3f5ed8` | `611d6e17d4cadc91c7e4443ce2fc8f9fe5d77287` |
| 16 | `137c395b190e77e6a1dbad2cd2bd5aae433d8d64` | `d028761059d6a4ba45525463a9720dffa283a245` |
| 17 | `dfa694ed26f4932024a457307dc270d7206c5c11` | `eb0eb5b478a5dd70342d5b86887e61b170bdf85f` |

Additional signed mechanical commits:
- `911904b4200947ee0db2aa437ed7b6d5074deade`: only `trial: None` in the named GUI LaunchOptions consumer; the named attachment consumer already uses Default.
- `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5`: exact11796-byte constructive-seal record from signed `1c86843ed128e54232a678006bb78effcab3428a`, plus the authorized capability fixture's explicit nemotron profile.
- `15decd4395ed453767896e15d14ac1bd5184fd93`: only explicit nemotron profile in seat-launch command_inner, preserving its real loopback provider.

All20 source commits and all17 originals were verified with Git, approved fingerprint `9B07E849480C68328CA20684844ECC0587DE7AB0`, author/committer Boxy <bboxy450@gmail.com>. `feature-land-dream-runtime-source-signatures.json` SHA256 `6e239ffc0c4d810aea6defccba73d675cbdc4f25b8ae53ac30ae99df34283cff` and `feature-land-dream-runtime-lineage.json` retain the evidence.

Consumed component reports include [[reports/2026-11-03-dream-wp3-d0-r2]], [[reports/2026-11-04-dream-wp4-pure-verifiers-checkpoint]], [[reports/2026-11-04-dream-wp4-audit-seal]], [[reports/2026-11-04-dream-wp4-adoption-binding]], [[reports/2026-11-04-dream-wp4-recall-final]], [[reports/2026-11-04-dream-hashbrown-record-cache]], [[reports/2026-11-04-dream-through-wp6-integration-outcome]], [[reports/2026-11-04-dream-wp7-trial-r3]] and [[reports/2026-11-05-oracle-dream-trial-acceptance]]. Historical reports/receipts certify only their own source, never this integration.

## Conflict dispositions and preservation

Named TOML deltas were applied over current main, preserving all current-main rows and fields. Corpus is the exact named136/175 union:192 unique records, with every original corpus row unchanged. Contracts retain current ScrollStick semantics and add only the accepted D1 source/test/assumption mapping to existing ProviderStream; their properties/controls are unchanged. Main's human labels, fairness groups, Ite-free guarded-action compatibility, ACP steering/configuration, GUI scrolling and all accepted Dream/process/Store/trial families remain present. No new stateful design or checker adjustment was introduced.

Store retains both main's intake module and the accepted no-replace export. ATTRIBUTION retains main's composer/onboarding/shared-license text and exact accepted dependency credit. Cargo adds only ring, icu_normalizer and exact hashbrown0.17.1/default-features=false direct edges; all488 locked package records/versions and all unrelated dependency edges remain unchanged. LICENSE remains SHA256 `07c15c9bc6b7c6ca2b8ef48c57421f4317cacc3ba6b27272dbbbc1c99013daaa`.

The protected-file check compares296 pre-existing out-of-patch source/tool/fixture/license files against main.294 remain byte-identical; the only exceptions are the two expressly amended one-line private-profile pins, checked against exact before/after strings. All16 frozen IR JSONL fixtures, protocol/schema/lock, current tool/checker/pin snapshots and unrelated GUI/ACP/configuration files remain unchanged. All204 historical decision/run files supplied by the17 patches match accepted1c bytes. The seventh normative record is separately exact SHA256 `312e4cddcd6539f8a0feee517bc1dc7fce58b070c9875eb87399155b5595e133`; no identity, number, authorship or signature was manufactured.

Accepted `src/dream/**`, process, tools, fork and Dream test bytes match1c. Surrounding engine/backend/import/headless changes preserve current main and add only accepted hunks. No old task/report tree, IR/VCS/Receipt implementation snapshot, Cargo upgrade, new toolkit snapshot or WP8/9 implementation was imported. Raw dispositions are in `feature-land-dream-runtime-conflicts.json` SHA256 `c4908bd444026904662d316af186fce53754574cec6e37a2872cf390992f47f9`; inventory union SHA256 `8b83c7913c77152f0f66844e4244a69918bdb08e77cc2a997936f7b42ff67a99`; amendment evidence SHA256 `41b7ff54827135cdfb11be19e155d2fb066c83f090a2a26927f6be6c223df24a`.

## Acceptance mechanisms and actual witnesses

| Required outcome | Source mechanism | Executed evidence |
|---|---|---|
| D0 replay, native custody, nonblocking cache and ordinary close hooks | `src/import/mod.rs:1109` caps snapshots16MiB/10000 records and custody128MiB; `src/dream/d0.rs:122` owns per-fingerprint nonblocking merge, `:657`/engine close hooks detach ordinary work, `:715` replays retained sessions. Cooperative log/input boundaries preserve the actual1000000ns guard at `:692`. | dream_replay13 parent tests plus three real child invocations, pipeline12-log replay/pin/cache/source witness, actual inline CPU broken control, missing/conflicting/changed/native/truncated/crash custody controls. Source logs preserve failures; no deadline/cap relaxation. |
| Shared memory identity, pins, graph and per-harness lifecycle | `entry.rs:52,96,284` validate/NFC/canonical identity; `resolve.rs:446,485` resolve exact Event/Record pins; `graph.rs:141,421` build/why; projection retains per-(entry,harness) states and global terminal fences. | dream_identity15 and dream_graph9 cases; committed/root/digest/nofollow/CRLF, dangling/conflicting pins, same-pair and terminal tests. Source bytes remain untouched. |
| Independent trust, audit and constructive receipts | `trust.rs:24,465,713,758` check V/T and independently approved exact producer/build/gate/task identities; `receipt.rs:184,559` interpret effects/canonical body; `seal.rs:247,295` freeze completed owned effects; `audit.rs:127,713` audit retained cuts and suffixes. | dream_audit_broken7, dream_trust8, dream_seal7 parent cases and real delayed-child fixture. Actual read-only CLI/P1–P9 rejection, unowned/late/ABA children, missing authority, skipped Recall, foreign-session Adoption and duplicate piggyback controls. |
| Hashbrown cache | `resolve.rs:313,485`: private HashTable<Arc<ResolvedRecord>>, borrowed complete pin equality/hash and std RandomState; trust transitions clear classification. | Actual Arc identity, collisions/resize/dedup, complete Option identity, failure retry and deterministic observable output tests; no new benchmark claim. |
| Bounded D1 synthesis | `synth.rs:20,93,523` limits output32KiB, strictly parses known output fields and resolved evidence, reuses common identity/deny, and emits Pending/Proposed only; prompt input is quoted low-trust data with no filesystem tools. | Actual dedicated scripted provider/request and same Store writer; private-HOME CLI, deterministic bounds/no-op/NFC, unsafe/partial/tool/failed/cancelled output controls in all20 pipeline cases. No live service call. |
| Owned process admission, result observation and cleanup | `process.rs:711` keeps off-worker admission and exclusive child ownership, cancellation-first/deadline observations; observed capture registers settlement before effects and distinguishes result from confirmed reap/IO completion. Heap-pinned factoring retains default stack behavior. |15 process units,8 cancel_process parent tests plus owned child invocation, actual queue/cancel/deadline/environment controls, leader/group/escaped-pipe reaping, dropped/failed/channel/unobserved cleanup negatives. |
| Store leases and queued-only fork frontier | `store.rs:80,195` retains writer lifetime leases and opaque trial validation; `store/fork.rs` retains queued prefix through validated trial frontier, no public active fork permission. | store_fork5, store_recovery4, lease units, exact raw/ext/unknown/CRLF/blob/fork custody and receipt corpus tests. Recovery advances epoch once; failure/unavailable observations do not become success. |
| Real confined trials and thresholds | `trial.rs:675,1087,1386,1944` prepare/start/execute/append exact held-out plans, serialized arms per local claim, independent entries and lawful pair states; `tools.rs:660,701,910` opaque confinement and pinned launch/store validation precede effects. | Actual20 arms,40 captured requests,4400 retained Derived tokens; with8/8 versus without0/8 and unchanged controls.2200-token failure rejects only its pair. Real verifier, original timing bindings, spend/unknowns/deadline/abandonment, usage join/basis, queued frontier and unsupported-native Inconclusive controls. |
| Confinement and close-effect boundary | `engine/mod.rs:180,421` validates trusted trial capability before VCS/Store/MCP, denies widening and suppresses trial D0/IR-advice close hooks; ordinary hooks remain nonblocking. | Real engine launch/refusal and tools tests, symlink/hardlink/FIFO/path/rebinding/forged-permit/exec controls, trial close-hook regression, exact source/log byte assertions. |
| Current-main UI, ACP and native behavior | Only one new None default in the named GUI consumer; inherited fixture profile pins alter no assertion or production bootstrap. | Actual composed default and ACP all-target runs;2 capability cases in each configuration; seven seat-launch cases with real loopback provider; separately executed nonPTY case; all-feature Clippy and native app smoke. |
| Formal proof and limitations | `tests/formal/models.rs`, corpus/contracts/coverage retain finite bounds, owners, atomic actions, properties, expected broken verdicts, fairness and source/API mapping. |11 formal-model tests,39 nonvacuity tests, reactor projection,192 complete native/Java reachable-state/labeled-transition/verdict comparisons; full artifact preservation below. No count-only parity or arbitrary Rust/kernel/crypto proof claim. |

## Commands, exits and retained logs

All log/JSON basenames below live under `.bboxy/runs/feature-land-dream-runtime-`. The JSON records retain command, actual exit, start/elapsed, source, generated-only dirty paths, private environment and log SHA256. Tests used owned short /tmp homes/temporaries, disposable keys/fixtures/sockets, offline Cargo, two build jobs, debug-free private profiles, no sccache, no RUST_MIN_STACK override and no operator input. Feature builds serialized in this checkout's target.

Exact frontmatter feature command, actual exit0 at7829ab66:
```sh
cargo test --features acp --test dream_vcs && cargo test --lib tools:: && cargo test --lib process:: && cargo test --lib dream:: && cargo test --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_replay --test dream_pipeline --test cancel_process --test store_fork --test store_recovery --test policy_gate && make test-ir && cargo test --test import_native --test ir_edge --test protocol_golden --test ir_v1_corpus && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
```
Every connected subcommand exited0. Required unit/integration targets ran nonzero cases; deliberate ignored fixture entries were invoked by their real parent tests. No new skips/retries/locks, suite serialization or timeout inflation was added. The all-target exclusions below are the existing separately executed seat/nonPTY cases.

| Log tag | Actual command | Exit | Source | Log SHA256 |
|---|---|---:|---|---|
| source-green | `Exact feature command below` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `dd6e2072b94b4e4cf49f348f26f68c080217f531c72060c6f0aad943c003ca0f` |
| source-current | `make test-formal && make test-formal-parity && make check-formal-evidence` | 0 | `15decd4395ed453767896e15d14ac1bd5184fd93` | `6343fc06ef66c8891f07066d1d3011d7f66e8156e37cbe8dd301e2240e81ba06` |
| composed-default-short | `cargo test --offline --locked --all-targets -- --skip non_pty_spawn_does_not_keep_a_sibling_master_alive` | 0 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `d5e42d7838d8d9d81b00c8c7e35f5d168f8524c71b69f2f08eaa41b396500f8d` |
| composed-acp-short | `cargo test --offline --locked --all-targets --features acp,test-fixtures -- --skip seat_launch:: --skip non_pty_spawn_does_not_keep_a_sibling_master_alive` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `a10b2621cab24982f55c3073aab47f53e7e859805bef053d55410fbd6c24cb3d` |
| capability-default | `cargo test --offline --locked --features test-fixtures --test capability_delivery processes::profile_and_acp` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `ff8ba8b8682246637a5614dc0001077dec9eadade79a3ae338e82b0289a6c19a` |
| capability-acp | `cargo test --offline --locked --features acp,test-fixtures --test capability_delivery processes::profile_and_acp` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `98127e2423bb38b906f6883256a7761f7d6698e1d7813d9b1585e7a994680de8` |
| seat-launch-fixed | `cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_launch::` | 0 | `15decd4395ed453767896e15d14ac1bd5184fd93` | `2923a34d770c9cb10ebc526c933fd89903d9390f739b95f059ddb76578d17698` |
| nonpty | `cargo test --offline --locked --features acp,test-fixtures --test seat_process non_pty_spawn_does_not_keep_a_sibling_master_alive -- --exact` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `abd2a658303f6761a4656ceb85a7df427ae9461420f305518d8eb84c4b1564ac` |
| static-final-source | `python3 -I scripts/checks.py headers && python3 -I scripts/checks.py secrets` | 0 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `4f3519361c9706df98f5ea425497dd985445dcaafa8e31c4e1df26adce74e687` |

The composed default pass at911904b precedes only the retained normative record and the two fixture-profile amendments. Its default-feature compilation excludes those test-fixtures helpers. The later composed ACP pass includes the capability correction and excludes seat_launch as prescribed; the final seven corrected seat cases run separately at15decd4. The signed final amendment explicitly retains both composed passes and the exact feature pass, without reopening them for polishing. Current-source and report-HEAD formal evidence bind the finished tree.

Native smoke evidence is `feature-land-dream-runtime-smoke-evidence.json`, SHA256 `69e2a5dfb698e092be1aa911e73d7609b38be5a3bab344538ca877bd8c2102cd`: actual launcher PID5254 exit0, hidden native window/surface/live frame, replay0, strict ad-hoc codesign0 and check-transcript0. `scripts/smoke-app.py:65` actually invokes check-transcript on the newly created native session. No visible desktop presentation or OS input/capture is claimed. Native process record SHA256 `a26f6eccae8ef260c74d51b62aa613a324fa8c418a364f9f164012e4604f59a4`.

## Formal union, byte preservation and source binding

The actual named union is192 models,60 contracts,1 crate;621426 states and2986392 labeled edges, with391 source/tool/spec input hashes. All192 native graphs and verdicts agree with pinned Java TLC, including safety/deadlock/liveness controls. No model/control/property/fairness weakening occurred.

All136 main model records and all952 stable model artifacts are byte-identical to main's retained136-model archive.173 of175 accepted candidate model records are exact. The only inherited candidate differences are HarnessScrollStick/BrokenHarnessScrollStick, for main's intentional panel/resume semantics: nine changed stable artifacts are enumerated, and every changed byte is current main's retained model.1020 accepted historical stable artifacts remain exact. The28 new trial model TLA/metadata/verdict records are identical to the accepted candidate and their full current native/Java graphs were recomputed; historical raw new-trial graph files were not in this checkout's cache, so old raw-file equality is not invented. Volatile Java timing logs/DOT identifiers are excluded from byte comparison, never from full graph comparison.

After the final fixture pin, all192 model records and all1344 stable artifacts also match the earlier green192-model archive exactly. `feature-land-dream-runtime-model-proof.py source-current` exited0 and independently checks named union, artifact bytes, exact permitted fixture diffs, all391 current input bytes, retained refs/evidence and the saved original generated native JSON. Current-source proof JSON SHA256 `fe41e668da18728aa9b7c2521ae166685459c4722d4b637ea255fa33b612ae12`; earlier source proof SHA256 `9835260e4c605f160b60ca66d3163d8417eec7419009aed130f81e324d8efa58`.

Exact-feature source receipt: `522b95088465fbdf6fe44b7ac8bb0dd613632d72d2453705c7dd120caf98ca97` at7829ab66. Finished-source receipt: `c52c2f50e988a73634ca57236cb5b55b928c2eb148cd683f53dd466d580ff17f` at15decd4395ed453767896e15d14ac1bd5184fd93. Both are durable under `verification/evidence/<digest>/`, independently source/tool/model-bound with dirty:false. Historical receipts are not rebound.

This immutable report is committed after those source checks. Fresh report-HEAD results are subsequently retained in `feature-land-dream-runtime-final-head.log/json` and `feature-land-dream-runtime-final-head-model-union.json`; their actual HEAD, receipt, hashes and exit must be read together with the native task_report commit. They are created by rerunning native/formal/parity/evidence after the report commit, never by copying an old receipt or editing its source field. The report does not manufacture a self-referential commit/hash.

## Failures, corrections and limits

All original failures remain retained:
| Log tag | Actual command | Exit | Source | Log SHA256 |
|---|---|---:|---|---|
| feature-gate | `Exact feature command below` | 2 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `19b281c3bfe74f8ab8810126854190356c976f262da8dda2f8291a5a380f56cb` |
| composed-default | `cargo test --offline --locked --all-targets -- --skip non_pty_spawn_does_not_keep_a_sibling_master_alive` | 101 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `5e4086bdfda6976d739188166b0122e8af9cb15a82f2286593d87ebb579550f6` |
| composed-acp | `cargo test --offline --locked --all-targets --features acp,test-fixtures -- --skip seat_launch:: --skip non_pty_spawn_does_not_keep_a_sibling_master_alive` | 101 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `cf96e32c31322ab430a6f49f7571b7e11523b956fce1689c56f35f1b3312c5e2` |
| seat-launch | `cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_launch::` | 101 | `7829ab66b037560ac8a90c6da13bb9fd7a2cfac5` | `80e550535108c1f5f187b8ffcc0a8b3914eac4593d33580d0d84d5f10a11bb28` |
| gui-probe-corrected | `target/dream-runtime-gui-probe --nocapture` | 101 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `e6c8b0a5553fe109557e8ee2d6c98a25a59548218558092a81533abb1c80fcfb` |
| gui-probe | `target/dream-runtime-gui-probe --nocapture` | 127 | `911904b4200947ee0db2aa437ed7b6d5074deade` | `1a8c327fffe7f46fcbc756e19711efa78106dd85e3a47c86cf038be18813b164` |

- The first metadata helper rejected compact TOML `name=` formatting. Its shell incorrectly continued staging, leaving unverified TOML conflict markers in intermediate signed c59957a/58d6cd4.3427d9b reconstructed the inventories from main plus exact accepted semantic deltas. No successful check is claimed for those intermediate commits. Subsequent commits parse TOML and reject markers before committing; the final named inventory/graphs/gates pass.
- The first exact gate stopped2 at missing HarnessDreamSeal normative source, after its runtime/IR/import/Clippy checks passed. The Oracle amended scope; the existing signed record was retained exactly, with no model/checker change; the exact gate then passed0.
- My initial task runner used a long Darwin temporary prefix. A bounded same-API diagnostic reproduced101 with safe error `session socket unavailable`; only the owned runner's TMPDIR moved to short /tmp. The identical diagnostic then passed0 (log SHA256 `f16133883555f8026cf54f26d84ccfead6d863ca83e414b25d42567b8ce513ed`) and the composed default suite passed0. No product path, assertion or timeout changed. The first diagnostic build also failed1 from manually chosen incompatible serde-core artifacts; the attempted absent executable returned127, retained above. A minimal API-only corrected probe compiled and supplied the actual socket evidence.
- Both MCP-origin capability failures and four seat-launch failures came from implicit current-main codex selection while fixtures wrote nemotron.md. Actual private processes exit0 in both modes, but implicit codex omits the fixture reporter and explicit nemotron starts it; observation log SHA256 `b3ca177b3d834af149337eeb40d300121f3353ccb81c185e3553ef62456cc126`. Oracle amendments authorize only two explicit-profile arguments. Both configurations' capability assertions and all seven real-loopback seat assertions then pass unchanged.
- The old KB5974 collision premise was fixed on inherited main, as the task states; this feature neither repaired nor filtered the KB, nor reran an obsolete full make-test hold. This report claims the exact feature gate, listed composed/affected tests and current formal/native evidence, not an unrun full make test or unrelated GUI timer/formatting repair.
- WP8 sinks/Recall production and WP9 triggers/live orchestration remain separate. Existing audit/seal/rendering/graph consumers are integrated; this does not provide live producer authority or a complete receipted run. Missing/unapproved effects remain UNPROVEN; absent local policy disables automatic adoption without prompting. Trial-qualified pair states do not imply an Item::Adoption, publication history, effective native loading or completed semantic audit.
- Native/ACP/general/heavy trial adapters remain honestly Inconclusive before unsupported inference. The confined lightweight verifier remains the accepted exact `sh -c 'test -f fixed'` witness. Capability wrappers are not an OS sandbox for hostile in-process Rust, mounts or same-UID adversaries. Native EOF is not proof of native process completion; quota/IO scheduling, cryptography, syscall durability, arbitrary source behavior and universal timing remain implementation assumptions/witnesses rather than finite-model theorems.
- No peer implementation, policy bootstrap, live model service, credentials or operator interaction was used. The old accepted source reports/evidence remain intact; no historical uncertain operation was redispatched.

At source handoff the index and source are clean; only canonical generated `.bboxy/runs/formal-native.json` is tracked-dirty. Own run/proof files and the seven prior untracked files are retained separately. The final report-only commit and fresh generated evidence preserve this distinction; no broad add/cleanup is performed. Oracle independently verifies and lands the feature.
