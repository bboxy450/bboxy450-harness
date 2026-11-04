---
type: report
title: 'Hashbrown private provenance cache: owned-pin indexing, proof and release measurement'
task: '[[01M6KTKM6HJC8G9YAB95F3NVN3]]'
commits:
- fd64540610168af18f0fb1a09c55b6337679b106
verify:
  command: 'cargo test --lib dream::resolve:: && cargo test --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence'
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T18:14:20.766Z
id: 01M6KW9RJEFE8V08BF79NWG3QF
updated: 2026-11-04T18:14:20.766Z
number: 20261103136526
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:de9357fae83f22c41b8e26453b5ad71687a0dbb0d2307a8ba4bd3a514b0f152c
  frontmatter_digest: sha256:089bddb63e2da7b770d1b100ecd405a1f56c5a3cd478bd4a3c83d58eabb13db1
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: NbOLqNiknz0fMSiX4QSy8tKu0a/T71pubB3MYBkRv0vFbsiz+1j5fOj+m62XvlonZjFrZpDZl0db2x6Qxui/Bw==
  log: .bboxy/log/2026-11-04.jsonl#45499
---
# Hashbrown private record-cache adoption

Implemented the bounded adoption in signed source **fd64540610168af18f0fb1a09c55b6337679b106**, branch perf/dream-hashbrown. Exact assigned verification exited **0**: **3 nonzero cache units plus45 identity/graph/audit/trust/seal tests**,138 native/Java models and45 contracts. The one explicitly run release cache-kernel benchmark observed a **2.116× median old/new ratio** (724.08ns→342.16ns per hit) and removes **978,260 bytes of duplicate serialized-key payload** for4096 entries. Timing was variable and covers only this kernel; the byte count is String payload length, not process memory. This is a useful bounded change; canonical KB5974 remains unwaived and no merge/publication occurred.

## Assignment, source and dependency scope

Task [[01M6KTKM6HJC8G9YAB95F3NVN3]], fetched etaged75b6e12204c28599968a547c39eb2467c910d7d0fbef2c90cb3bab702f3b36, assigned this Executor the existing writing tree /Users/feral/mydev/bboxy450-harness-dream-wp4, now perf/dream-hashbrown, exact base **71b4adab4dd80018de6338ccbfa751ac78649848**. The old dream/wp4 ref still resolves to that exact base. Its reports and retained base evidence were not edited. New source fd64540610168af18f0fb1a09c55b6337679b106 is GPG-signed Boxy <bboxy450@gmail.com>, actual git verify-commit0, with the required Report trailer.

Source changes are exactly Cargo.toml/Cargo.lock, ATTRIBUTION.md, src/dream/resolve.rs, tests/dream_identity.rs and narrowly scoped cache notes/mapping in verification/coverage.toml/contracts.toml. Performance/check evidence is under the authorized .bboxy/runs/hashbrown-* log/json/txt paths. No other worktree/target was created or edited, no rebase/reset or collection sweep was performed, and no graph/IR/projection/receipt container, resolver permission/file/path/digest/signature logic, API/wire shape, lifecycle, retry, lock or persistence contract changed. No recruitment, live model spend, live credentials/policy/bootstrap, shared source edit, main merge/push, publication or KB identity repair.

Cargo directly exposes **hashbrown = "=0.17.1", default-features=false**, requesting **no features**. Cargo.lock changes only the root package's dependency edge to the already-locked hashbrown0.17.1; every package version/checksum/unrelated pin is unchanged. Default-mode before/after inverse feature trees and hashbrown-feature-comparison.json both show no hashbrown features, with no newly enabled default-hasher/foldhash/allocator feature. Existing unrelated transitive feature requests retain their own terms.

Checked the local locked table.rs/manifest and [official HashTable docs, rendered as0.17.1](https://docs.rs/hashbrown/latest/hashbrown/struct.HashTable.html): explicit hash/equality and a consistent insertion/rehash hash callback are the intended API. [Upstream](https://github.com/rust-lang/hashbrown) records std's existing SwissTable adoption and MIT OR Apache-2.0 licensing. The benefit here is indexing an already-owned pin without a second owned key or per-hit JSON construction. ATTRIBUTION.md accurately says dependency/API use, with no copied implementation; project source/license/origin headers remain attached. Local pinned source: /Users/feral/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.17.1/src/table.rs. The version-qualified docs URL was inaccessible through web on the first attempt; the official latest page succeeded and explicitly identified0.17.1. No dependency download/upgrade was needed.

## Mechanism and evidence per criterion

| Requirement | Mechanism and executable evidence |
|---|---|
| Private HashTable over existing owned pin | resolve.rs:281 stores only HashTable<Arc<ResolvedRecord>> plus its hasher. The existing immutable record owns the sole cache pin; there is no separate String/composite key or second stored pin |
| Borrowed hit, complete identity, consistent seed | record_hash:285 hashes borrowed root/path/id/commit/digest as one tuple, preserving String/Option boundaries. get/find and entry/rehash use the same function and the same stored builder; complete RecordPin equality is always checked |
| Untrusted-pin hasher and safe APIs | production RecordCache's concrete default parameter is std::collections::hash_map::RandomState. HashTable find/entry/or_insert/get/clear are safe; no raw/unsafe hashbrown API, new hasher crate or mutable key exposure. The private test generic supplies a constant collision hasher only in cfg(test) |
| Unique insertion and immutable Arc behavior | entry.or_insert retains the existing Arc on a duplicate complete pin. A contradictory replacement record cannot add an entry or overwrite classification; ordinary misses return the newly stored Arc. Existing with_trust clears the entire records cache before replacing policy |
| Complete fields and None/Some identity | private record_cache_matches_all_fields_and_exact_option_identity varies each of five fields, None/Some-empty/Some-one/Some-two for id/commit, all under the same forced hash. Every distinct pin returns its own correct Arc |
| Collisions, resizing and clearing | exercise_growth inserts512 records, asserts more than one actual capacity growth, queries all512 in a permutation, checks exact Arc/value, rejects a contradictory duplicate without growing len, clears all hits, then accepts a fresh classification. It runs with constant collisions and independent production RandomState seeds |
| Canonical observable data across seeds | private canonical snapshots of full pin/record bytes/flags agree under two RandomStates and the collision hasher. Real independently constructed Resolvers queried in opposite orders yield identical sorted pin/byte output; no hash values or hash iteration order are claimed deterministic |
| Actual Resolver errors/permissions remain intact | dream_identity.rs:437 checks exact repeated Arc reuse, committed/live/id-none/path-alias distinctions, all five rejected fields plus Some-empty id/commit, unchanged bytes/signatures, and a missing live file that succeeds after creation with the same pin (no negative cache) |
| Trust invalidation | dream_identity.rs:523 uses actual private signed Git/KB policy: untrusted→authorized→revoked transitions return new Arcs with unchanged bytes/signature and fresh trusted flags. Under authorized policy, id=None remains untrusted and cannot reuse the identified record's Arc |
| No stateful/formal expansion | existing immutable one-owner get/insert/clear/persistence semantics remain. coverage/contracts record actual API/unit tests and explicit hashbrown/allocator/Hasher safety/hash-quality/performance abstractions. No model/property/control/bound was altered |
| Canonical substrate/unattended behavior | all current audit/graph/identity/trust/seal suites, real cross-session Adoption API/CLI rejection, IR/golden/fork/replay/Unix/headless compatibility and native smoke pass. All16 frozen JSONL fixtures match base71b4ada byte-for-byte |

Hot-path source inspection: Resolver::record retains its existing pin validation, then passes &RecordPin to RecordCache::get and clones only the returned Arc. record_hash borrows all fields; HashTable::find and derived pin equality build no owned key. serde_json::to_string(pin) has disappeared from the production records hit/insertion path. JSON serialization remains only in the explicit old-path benchmark and tests/other unrelated canonical operations. HashTable misses may allocate table capacity; the claim is specifically **zero extra JSON/key allocation or duplicate key storage on cached hits**, not zero total process allocation.

The synthetic private units test the index; actual Resolver tests separately exercise unchanged file/crypto/trust behavior. Arc snapshots already held by callers remain immutable; cache invalidation prevents their reuse as a newly resolved classification, matching the previous contract.

## Reproducible performance measurement

Executed **once**, as requested:
```
cargo test --release --lib hashbrown_record_cache_bench -- --ignored --nocapture
```
Exit0 at fd64540610168af18f0fb1a09c55b6337679b106; debug_assertions=false, Rust1.99.0 b940084d7, aarch64-apple-darwin, AppleM4/macOS27.0.1, CARGO_BUILD_JOBS=2 and the existing release profile/target.

The ignored resolve.rs:974 benchmark prepares the identical4096 synthetic pins/Arc records in the prior serde-JSON+BTreeMap path and the actual new helper. Each path warms16384 hits, then runs262144 hits per sample for7 samples (1,835,008 measured hits/path), alternating which path runs first. A fixed coprime permutation visits each key equally. Both paths black_box the borrowed inputs and cloned Arc results, including equivalent Arc clone/drop work. IO, file validation, signatures and cold insertion are outside the timed kernel. No benchmark dependency, unsafe allocator instrumentation or CI wall-clock threshold.

| Kernel | Sample durations, milliseconds | Median ns/hit |
|---|---|---:|
| Old JSON+BTreeMap |124.504,167.070,219.100,261.324,189.814,210.722,177.265|724.084 |
| Actual HashTable helper |39.685,208.889,56.685,89.695,111.624,101.034,75.863|342.161 |

Observed median ratio old/new **2.116209417**. Samples vary substantially and one paired new sample is slower; other shared-machine load was not controlled. This supports a kernel benefit alongside the concrete allocation/storage simplification; it is not an end-to-end Resolver/audit speed claim.

Old duplicate serialized-key payload sum: **978260 bytes** over4096 keys, using actual String::len. New duplicate key payload: **0**, established by the stored type/source (only Arc<ResolvedRecord>). Old timed serialization emits62608640 key-payload bytes per sample by the known uniform workload. These lengths exclude String capacity, tree nodes, table buckets/tags, allocator overhead and all other process memory; no RSS or total-allocation conclusion is drawn.

Durable raw data: .bboxy/runs/hashbrown-benchmark.log SHA2565c4ae1e24fae32b3a606d4f25b08a77af88bc83abbfc52e7a8b54e5ea730ca22; parsed workload/timing/source/toolchain/platform data hashbrown-benchmark.json SHA2563241af36ed140fd3b7c8b27fb81793a56b051b779e4525edaa41bdea25ef7c7b. The benchmark was not rerun to select a better result. Disposition: retain this narrow adoption, preserve all ordered output containers, and make no broader collection or end-to-end performance claim.

## Exact checks and retained source evidence

Assigned command:
```
cargo test --lib dream::resolve:: && cargo test --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
```
**Exit0 at fd64540610168af18f0fb1a09c55b6337679b106.** The cache filter runs3 actual tests plus1 ignored benchmark; the second invocation runs45 tests (identity15, graph8, audit7, trust8, seal7 plus invoked private child). Native formal/nonvacuity/reactor tests and all **138 full native/Java reachable-state/labeled-transition/verdict sets** pass for45contracts/1crate. Source-head receipt **5016fa4f443cf086b2addf85de211f084bfeeafea15ebff81f185648879fb98c**, dirty=false, retained under verification/evidence/<digest>/evidence.json and target/formal/evidence.json; check-formal-evidence0. Primary log hashbrown-primary.log SHA2564cc6bac1938394b547b8db4769091653b071aaed7bc8697b3a37e45ad2d56009.

Actual status rows are hashbrown-verify.json SHA25605a6719d5528ee8260488266bb78e5153ad59532465636d17b0f814dfc83e8ac:
| Command | Exit and evidence |
|---|---|
| Explicit release benchmark above |0; benchmark.log/json |
| Exact assigned command above |0; primary.log |
| make test-ir |0; hashbrown-ir.log |
| cargo test --offline --locked --test protocol_golden --test ir_v1_corpus --test store_fork --test ir_edge --test dream_replay --features acp-agent |0;50main regressions plus explicitly invoked private children, hashbrown-substrate.log |
| make smoke-app |0; hashbrown-smoke.log |
| python3 -I scripts/checks.py headers |0; hashbrown-headers.log |
| python3 -I scripts/checks.py secrets |0; hashbrown-secrets.log |

Additionally cargo fmt --check/git diff --check0, offline initial lib check0, targeted new Resolver tests2passed0, cache3passed0, and all-feature clippy0 on the exact final source bytes before commit: cargo clippy --offline --locked --all-targets --all-features -- -D warnings, hashbrown-clippy-fixed.log SHA25639e23f572ea6ebd390316213b73ae0e6ad05c85d7d60826b6ee3eef95905095d. Before/after feature tree commands0; hashbrown-feature-comparison.json SHA256e7a340183ae5fbd14cd1cb80d06ead86c46efed3ce841f14d6f7eaecc091652c.

Fresh native smoke log SHA256a7efa5c8003104690ed5fc0ab6c3023199d3d58501057bc009e5a8a64e8ed0dd; target/app-smoke/evidence.json SHA256e02939f12d7743eff19c2d40e0224035d372296399425902891f897f8ca2ae28 names source fd645406…, private_home=true/operator_input=false and replay/native launcher/codesign/actual CLI check-transcript exits0. It builds the release/ad-hoc app in the existing target, renders its own replay, creates a hidden NSWindow/Metal surface/private live frame and exits/reaps its exact process. No desktop input/capture, visible-presentation claim, Apple signing identity, Keychain prompt or operator credential. check_transcript means the real protocol laws and bundled CLI checker, not an invented shell target.

The full report/evidence commit changes HEAD without changing tested source/dependency bytes. Final-HEAD native/Java parity and source evidence validation are refreshed after that commit; task_report links the resulting actual receipt. The fd645406 receipt is not represented as certifying an unknown later HEAD.

## Failures, assumptions and disposition limits

Two development failures are retained: initial cache-unit compilation101 due omitted generic type annotations in tests (hashbrown-cache-units.log), fixed by explicit production cache types; initial clippy101 flagged the constant release-profile assertion in the ignored benchmark (hashbrown-precommit-clippy.log), replaced by reporting debug_assertions in its data. Both corrected checks pass; neither was counted as a successful run. The only explicit benchmark run is the release0 above.

This memoization replacement introduces no new stateful contract. Existing finite models abstract the index and do not prove hashbrown/allocator/Hasher memory safety, hash quality, allocation counts, arbitrary Rust/OS behavior or benchmark timings. Unit/API/process tests provide the concrete observations above. No property/control/model/checker was weakened, omitted or bypassed.

Canonical **make test remains not green** because unchanged KB5974 is unwaived. Per the signed task, no unchanged full canonical failure loop was rerun. Its retained actual prior make2/checker1 log remains /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3; the conflicting records were untouched. No fresh entire default+ACP/full GUI/packaging canonical pass is claimed. Original WP4 remains reported, not main-accepted. Source is clean; required tracked formal-native.json remains modified and retained, as does the untouched hashbrown-base-formal-native.json. All generated performance/check files belong to this task, stay in authorized run paths, and contain no secrets/raw operator transcripts. The Executor reports this verified isolated implementation for independent Oracle review.
