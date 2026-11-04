---
type: report
title: 'Dream WP4: retained audit, trust and constructive receipt seal'
task: '[[01M6J8TV4EVP7GZG875FC5J4P5]]'
supersedes: '[[01M6KG25JS2DZE6ECB640S2DDB]]'
commits:
- b48d44cd5123a9842dd96b25cdf1a1313c07810b
- 29afe26a320d40649938a02e10e6b1e68f13a39b
- 388cd3986060cbbc5f50f0e8ff1fc29a6ce939d0
- 0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7
verify:
  command: cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T16:36:50.242Z
id: 01M6KPQ72MNZQ3BJP4AN64MZNR
updated: 2026-11-04T16:36:50.242Z
number: 20261103136490
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:781427f284d61e36a85c03b38f7498bf68a6b801fe50f08a55ad87b50333f0ca
  frontmatter_digest: sha256:666ae5021b929ebb90eeeb18d6bcda5755a37822a1da27d6f0ae2197ffb3be05
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: iqawN0i+O6zdTnyVwQG7gYvLOkrTLZFWzJkufneiJ8mxc0yNdu1trGWbSrPhuHMZT64AxPyP3c+Bo84uEz3nDA==
  log: .bboxy/log/2026-11-04.jsonl#45173
---
# Dream WP4 audit and constructive seal handoff

The assigned implementation is complete as an isolated candidate. Source commit `0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7` is signed Boxy <bboxy450@gmail.com> and its exact current primary command exited **0**. This supersedes the incomplete implementation/tooling disposition in [[01M6KG25JS2DZE6ECB640S2DDB]] and [[01M6KFSB6JQA1VRP4XCA7FZ26Q]]; their historical failures and earlier evidence remain accurate. Canonical acceptance is still blocked by the unchanged duplicate KB5974 gate. No merge, push, publication, live trust-policy installation or real producer approval is claimed.

## Governing task and source custody

Task `01M6J8TV4EVP7GZG875FC5J4P5` was fetched again before handoff: etag `72739272d6e4452174bca2d40e4738fbbc51996e32b145de05b80199385a708e`, verify revision3, claimed, worktree /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp4. Governing text is dream-log and its committed refinement/native retained-IR custody, reviewed V/T/E/R1–R3, and operative [[d20261103136471-dream-constructive-receipt-seal]] (01M6KGTF6AYWC6NN1H3CY9FKMM; digest312e4cddcd6539f8a0feee517bc1dc7fce58b070c9875eb87399155b5595e133).

The original comparison base remains `7d3e22cbc4442cee8ea8bdef6e23f5225708680c`. The earlier pure checkpoint is b48d44c, with append-only reports29afe26/388cd39. The Oracle's checked integration base is `10c722781bcdaaaaac2fa0266f0787fdf93034f2`: carrier/tooling/report picks84f7b445/04a6aeab/2f4edd76/8252aab2 and operative decision10c72278 were inherited read-only. This phase's diff is **10c72278..0fea176**, 19 scoped source/test/fixture/mapping files. No dependency protocol/schema/frozen-corpus/frontend/tooling file was edited. There was no rebase/reset or write to WP1/WP2/WP3/WP5/carrier trees. Every source file retains the project/license/absolute attribution-policy header. The earlier checkpoint exposed only the authorized already-locked ICU2.4.0/ring0.17.14 direct dependencies; this phase adds no dependency/version/feature change.

`git verify-commit` exited0 for original base7d3e22cb, pure sourceb48d44c, integrated base10c72278 and source0fea176 (actual rows /tmp/dream-wp4-seal-signatures.json). Source was clean for the primary gate; the required formal commands refresh .bboxy/runs/formal-native.json and ignored retained evidence, which are preserved outside source commits. The full report commit changes HEAD without changing tested implementation bytes; final-HEAD parity/evidence is refreshed after that commit and linked by task_report. The source-head receipt below is not represented as certifying an unknown later HEAD.

## Implementation and acceptance evidence

- **P1/P3/P4 identity, origin and retained byte custody.** entry.rs:25,112 and resolve.rs:275 share the NFC seven-field EntryId, strict exact committed RecordPin/event resolution, authorized captured-root/main membership, nofollow custody and unchanged raw line identity. Unknown dream triggers and MemoryWrite outside dream sessions reject. audit.rs:127 checks actual protocol laws, finite/acyclic resolving closure, materialized/draft identity and source evidence. Identity tests include fixed composed/decomposed Latin/Hangul vectors, ordering, NFC collisions, CRLF digests, conflicting raw identities, absent commits, scope and ancestor/final symlinks. No clock or arrival order chooses lifecycle authority.
- **P5/P6/P7/P9 projection and authority.** projection.rs builds union sets, independent (entry,harness) maxima and global terminal fences; adopted pairs require exact resolving same-entry/same-harness qualifying Trial(Pass), known thresholds and reported comparable usage/spend/control/run counts. Numeric rank/time is a deterministic publication/strength calculation, never lifecycle authority. Failed held-out observations remain in net gain. Exact record/body bytes stay separate from stale status/harness metadata; changed content is preserved/rejected as changed content, never overwritten as repair. audit.rs rejects borrowed trials, revival/terminal reopening, untrusted anchors, incorrect scope and non-Proposal instruction sinks. tests/dream_graph.rs and the named P1–P9 broken fixture loop exercise these distinctions.
- **V/T and noncircular artifact approval.** trust.rs:24 is the sole exact KB DSSE producer-byte verifier: LF/frontmatter/provenance exclusions, serde YAML/JSON byte reproduction, PAE, digests, padded base64, Ed25519 and signer/id/generation bindings. Pinned real producer bytes SHA25651e052c1f90d3322351848263d9ce1097a9f31e30860bf00d6bd95577c5f5a36 remain a byte vector, not signer authorization. External trust/2 policy separately binds actual root/branch, allowlisted OpenPGP signer, last-touch blob/signature and ancestry revocation. Approved artifacts bind source/tree/version/build/current executable/capabilities plus signed gate and independently trusted accepted task/full report. Source S and gate G must precede report R, whose commits explicitly include S/G. current_exe is measured once on the existing blocking worker, independent of session cwd. Private OpenPGP/Git/KB-key fixtures cover accepted S→build→G→R→task→policy plus reversed ancestry, changed executable/gate, missing gate, policy permission/root/K2/rotation controls. Synthetic fixture gate rows exercise the verifier schema/chain; they do not certify that seven live-producer witnesses ran or approve any live binary.
- **P2/R1–R4 and deterministic publication.** receipt.rs:532,845,1091,1128,1160,1267 owns canonical bounded body encoding/parsing, exact typed header/body/blob/session/item binding, dense E effects, read-only source-prefix closure, publication outer-run observation, retained cut/inventory/projection digest, deterministic rank/tie/budget-fitting subset, committed candidate pins, shared generated bytes and exact post-receipt Adoption pins/block/receipt digests. Missing/ambiguous/unapproved proof is named UNPROVEN; known mismatching effects/bindings/suffixes are FAIL. Old minimal carrier fixtures remain wire-readable and semantically insufficient. Shared rendering includes Engine/ACP quoted context, Claude index/topic, fixed Codex adapter/full-entry resources and Grok/proposal shapes, with explicit actual-scope requirements. Missing Grok loader scope is unproven. Current materialization is only a read-only drift/unknown diagnostic and never historical proof. Retired retained exposure is labelled stale_exposure. An actual later hand edit remains byte-identical after a successful audit and receives drift.
- **Constructive seal and P4 protected effects.** seal.rs:83,192,328 implements exact LF-inclusive prefix through Receipt ItemStarted, strict seal/coverage/count/child/header/suffix validation, and one nonblocking per-invocation admission owner. Outstanding/dropped admissions and active/unobserved children prevent freeze; late admission/reopened children fail. No global claim/queue or new executor pool is added. Canonical freeze requires the registered deny set and refuses normalization that silently changes pins. The final own-custody helper derives Store/blobs from the actual Store, verifies the actual retained prefix, uses owned0600 temp/write/fsync/no-replace link/directory fsync, preserves conflicts, cleans only its own temp and exposes uncertain outcomes as unproven for query rather than blind retry. A receipt never records its own final digest/write; publication sealing is observed by the still-recording outer run. Final suffix excludes new semantic effects, source/cache writes, publications or detached work. Source domains derive from selected sessions/configuration, never receipt text; source mutations/ABA and own-custody disguises fail P4 writer_touched_source.
- **P8 and provenance queries.** One deny/registered-data boundary scans drafts/records/rendered blocks and receipt construction. Diagnostics never echo matches; CLI rendering scrubs registered/deny data. Tests cover every baselined family, object keys, registered values and normalization/pin preservation. graph.rs links cross-root exact names, record/evidence/trial/state/adoption/receipt and actual rendered sink-line provenance without making presence into approval. audit/graph/why are read-only; good API and actual CLI path:line queries return0, all broken fixtures return1 with their named P, syntax/unknown queries retain2. audit.rs:990 retains one Tokio stdout writer and awaits flush before main's explicit process exit, fixing the observed empty-JSON transport race without changing verdicts.

The sufficient fixture in tests/fixtures/dream/good/audit_fixture.rs uses real private signed Git records and isolated policy, retained source/build/trial/adoption sessions, six held-out/control origins and20 actual private shell verification runs, committed record pins, no-replace/fsynced publication and an idempotent repeat. Outer run receipts cover publication effects/sealing; foreign source pins resolve against normal retained sessions. A later terminal event lies outside the publication cut and yields stale_exposure. tests/dream_audit_broken.rs:281 checks audit0, graph/why line binding, real isolated CLI0, subsequent hand-edit drift0 and unchanged source-log hashes. These are sufficient retained fixtures and real private process/API observations, not claims that WP7/WP8 product producers already exist.

## Formal contract and actual process controls

The new finite model was written/checked **before** implementing the new ownership primitive. `cargo test --offline --locked --test formal_nonvacuity dream_seal_freeze_and_both_broken_controls_are_nonvacuous` exited0; /tmp/dream-wp4-seal-model-first.log SHA256f4add0ae054b191bf049bd7ed000ceee275acaa3d8e2f8ad973113e1acc24a8c.

tests/formal/models.rs:4989 names phase recording/frozen/sealed/closed, absent/active/joined/reaped child, unobserved/observed/enforced coverage, zero/one inflight effect, spawned/late-protected/custody bits; atomic admit/complete/spawn/observe/join/reap/freeze/seal/close actions; invariants NoProtectedEffectsAfterFreeze, FrozenOwnsNoMutableChild and FreezeHasNoInflightEffect. No arbitrary-IO fairness/liveness claim is made. tests/formal_nonvacuity.rs:1387 proves useful closed/observed-child states, rejects joined-unobserved freeze, and reaches both named broken controls. Full native/Java state/labeled-transition/verdict comparisons are:
- HarnessDreamSeal: **pass**,35states/61edges.
- BrokenHarnessDreamSealChild: expected **safety**,59states/100edges, TLC12.
- BrokenHarnessDreamSealLateWrite: expected **safety**,50states/101edges, TLC12.

tests/dream_seal.rs:191 spawns its exact private delayed child with closed environment and bounded readiness/exit, observes WRITE/RESTORE through a real pipe, and joins/reaps it. Safe active-child freeze is UNPROVEN; an unobserved earlier child is also unproven. The broken own-exception branch actually performs source ABA, restores equal final bytes, retains independent observations and causes actual audit P4 FAIL writer_touched_source. Equal bytes or a joined row never substitute for observation. Other controls cover wrong source domains, own-custody disguise, prohibited detached suffix, missing/unknown seal/coverage, duplicate/unknown children, prefix/header/item/count mismatch, cut/inventory/selection/rank/budget/block/generated/adoption/source/outer observation bindings and blob loss/corruption. The normally ignored private child case is explicitly invoked by its parent witness.

verification/contracts.toml and coverage.toml map actual source/tests,138 models and45 contracts, retaining planned product gaps honestly. Finite quotients do not prove cryptography, byte hashes, syscalls, real confinement/durability, truthful producer completeness or arbitrary OS/Rust behavior. WP8 owns real effect wrappers/independent sink observer/executable gates; WP9 owns final combined mapping and coherent-main verification. Owned model tools/children cannot be relabelled external writers. No matching approved executable/capability or scope/observation evidence means producer_unapproved/UNPROVEN; no live bootstrap or approval queue is introduced. Native loader reachability/consolidation/forgetting remains explicitly excluded.

## Commands, collected exit codes and retained evidence

The exact primary command is:
```
cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
```
**Exit0 at source0fea176**, /tmp/dream-wp4-seal-primary.log SHA2561272bea9b2dde9d61c85cd318145f868e047814800e7444f176cdc2ecda902b5; command/status record /tmp/dream-wp4-seal-primary.json. It ran **40 primary tests** (audit4, graph8, identity13, seal7 plus invoked private child, trust8), native formal tests4/nonvacuity32/reactor1, and all138 native/Java full graph/verdict comparisons. Final source receipt **924d9d8b3b1719865ddda1d1237f0cc1933dde447000c5643b34bea4afce016d**, source0fea176,dirty=false,138models/45contracts/1crate, retained under verification/evidence/<digest>/evidence.json and target/formal/evidence.json; `make check-formal-evidence` exited0. No former checkpoint receipt was reused for this combined source.

Additional actual commands (all exit0, logs indexed /tmp/dream-wp4-seal-additional.json, SHA256212f6ab89d333740531dca78375759cd19f8b7fc6799e58af23780dec54ae5e3):
| Command | Result | Log |
|---|---|---|
| cargo fmt --check |0| /tmp/dream-wp4-seal-additional-0.log |
| cargo clippy --offline --locked --all-targets --all-features -- -D warnings |0| /tmp/dream-wp4-seal-additional-1.log |
| make test-ir |0; locked schema/compatibility and broken controls| /tmp/dream-wp4-seal-additional-2.log |
| cargo test --offline --locked --lib |0;33unit tests| /tmp/dream-wp4-seal-additional-3.log |
| cargo test --offline --locked --test protocol_golden --test ir_v1_corpus --test store_fork --test ir_edge --test dream_replay --features acp-agent |0;50main regressions plus3 explicitly invoked private child checks| /tmp/dream-wp4-seal-additional-4.log |
| python3 -I scripts/checks.py headers |0| /tmp/dream-wp4-seal-additional-5.log |
| python3 -I scripts/checks.py secrets |0| /tmp/dream-wp4-seal-additional-6.log |
| cargo test --offline --locked --test protocol_laws |0;3tests| /tmp/dream-wp4-seal-smoke-0.log |
| make smoke-app |0;fresh release, ad-hoc bundle, replay/render, strict codesign, hidden native Metal window/surface/frame, exit/reap and real CLI check-transcript| /tmp/dream-wp4-seal-smoke-1.log |

Smoke log SHA256bad588d94de1db5a8689847e04f2e390af526db7f317d647caa3648741863cd1; actual target/app-smoke/evidence.json SHA256a169af008da9c4b1b6eb6c8a6b6d10d6ec5bef3b18ae61a502cfc37547473147 names source0fea176 and every replay/native/codesign/transcript exit0, private_home=true/operator_input=false. Hidden-window evidence does not claim visible presentation. No desktop input/capture, Apple signing identity, Keychain prompt or operator credentials were used. No fresh complete GUI/packaging gate is claimed by this WP; inherited checked carrier evidence remains separately attributed to the Oracle amendment.

All15 prior frozen JSONL fixtures compare byte-for-byte with7d3e22cb; /tmp/dream-wp4-seal-old-corpus.json SHA256c2c657284a5b22bce8f0a43acc88dd5c22791bfa7db6a7ae1786f82fd65cbafb. Corpus16, golden/fork/Unix/headless blob and unknown roundtrip checks run through the actual inherited carrier tests. `check_transcript` means protocol::laws::check_transcript/check_authored used by audit/replay/corpus and the actual bundled CLI check-transcript in smoke; no invented shell target.

## Failures, baseline and remaining acceptance limits

Development failures were retained rather than counted as passes. The first sufficient fixture failed101 on the actual missing Terminal durable head, then was corrected. Seal controls initially exposed missing-seal/outer-observation handling, corrected before green; actual empty CLI stdout exposed the async process-exit race, now flushed and tested. Intermediate compile/lint iterations failed101 and were corrected. The first new drift test failed101 because absolute nofollow traversal treated macOS's /var alias as unavailable; diagnostics now open the actual configured sink root and then nofollow relative components, with restored40-test suite0 (/tmp/dream-wp4-seal-five-suites-2.log SHA25696ff6faac3fe426268e7bcf32632d7f171580ee0731120895d431a9fa8af6584). Prior checkpoint post-report formal exit2 due moved upstream pins remains recorded in its append-only report; the Oracle integrated the separately checked carrier/tooling repair, and current exact verification0 uses it without local bypass.

Canonical `make test` is **not green and was not rerun in an unchanged red loop**. The retained actual WP3 canonical log /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3 records make2 after the real kb-records checker1: duplicate number5974 in d5974-log-ir-v1-only-add.md and d5974-memories-are-kb-records.md. Their current SHA256 values still match28f3442ce7dbfa406518135828bf5f063f1b06b573f3fa2b7987ef4c832ba450 and d2eb97c855304806f9da7abfaa6ff6b2d003a1320aea582982fbe963c378a4d4. No numbering repair, gate filtering/waiver, third identical run or main acceptance is claimed. Original conflicting mirrors remain blocked under dream-log precedence.

WP4 delivers shared validators/builders and sufficient retained/pure/private-process evidence. Real trial/sink producer wrappers, independent observer/live artifact approval and final coherent-main canonical/full GUI/packaging integration remain with their explicitly assigned WPs/Oracle. The unwaived KB/full gate and those combined integration obligations prevent final swarm acceptance; this report is an Executor handoff for independent review.
