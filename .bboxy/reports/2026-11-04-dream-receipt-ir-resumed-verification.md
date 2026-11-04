---
type: report
title: 'Receipt IR carrier: source-bound verification after canonical snapshot repair'
task: '[[01M6JTM3T7RB0TQNDARJVY6PAS]]'
commits:
- 4ebde7d542b2414422680d07c0695a7f5ff6a752
- 5a779b500c8f5beeb53a860c2b74133d972a317f
- b1dc978ecbc3b3f055bc3bacfd75576732c80d34
supersedes: '[[reports/2026-11-04-dream-receipt-ir-resumed]]'
verify:
  command: make test-ir && cargo test --test protocol_golden --test ir_v1_corpus && cargo check --all-features && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR96J84HR56MCXQ2JSW5Y
created: 2026-11-04T14:49:29.363Z
id: 01M6KGJN86P32K11AYWM9B3JN4
updated: 2026-11-04T14:49:29.363Z
number: 20261103136468
created_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
updated_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
authorship:
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  kind: seat
  public_key: +go7EGHCYyMM/7hLFRzzAOFaKqWCHKAaxEdDnpzj8+I=
  process_generation: '1'
provenance:
  digest: sha256:2c4137b2fed672a8f805830452ea84baecc2bb791be6f244be7320aedbd15135
  frontmatter_digest: sha256:e9e71fc595965b1a85e53fad5830d3b2593817e06ee1332b60a1c8cba4ffb245
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  sig: MvIcdc35zNcxr+H9LUNHAK15XoY9V5gPj2Rl2NzWwZFspF84qxSvTnQ3Ers/XzNux4SK9HUvHEOEK0j+TKWFAw==
  log: .bboxy/log/2026-11-04.jsonl#44780
---
ROLE: EXECUTOR. Task [[01M6JTM3T7RB0TQNDARJVY6PAS]], current amended record etag 515064ef36531a5c37c2d81cc45e7a21de0db81a31430e46c451c95ebf2ef4f5. Worktree /Users/feral/mydev/bboxy450-harness-dream-receipt-ir, branch dream/receipt-ir, prepared base 7d3e22cbc4442cee8ea8bdef6e23f5225708680c. This is an append-only correction to [[reports/2026-11-04-dream-receipt-ir-resumed]], preserving its complete carrier implementation and test evidence while recording the subsequent failed final check, authorized repair and fresh source verification. No older evidence is relabeled for a new source.

Implementation 4ebde7d542b2414422680d07c0695a7f5ff6a752 and recovery report 5a779b500c8f5beeb53a860c2b74133d972a317f are signed Boxy <bboxy450@gmail.com>. Snapshot repair b1dc978ecbc3b3f055bc3bacfd75576732c80d34 is signed by the same approved key. All original carrier source and frozen corpus remain unchanged after 4ebde7d.

## Carrier acceptance evidence

- src/protocol/mod.rs:212, 696, 709 — typed ReceiptKind/ReceiptOutcome/Receipt, optional Adoption.receipt, defaulted new fields and omitted empty/None fields; ItemKind::Receipt. Rust Receipt.kind serializes receipt_kind, distinct from Item's kind=receipt discriminator. Missing/future enum values remain reader-preserved Unknown. Decision [[d20261103136378-dream-receipt-wire-kind]] corrects the schematic field collision.
- src/protocol/laws.rs:270, 432, 538, 659, 687 — dream-only durable authoring, recursive Unknown rejection including nested sink kind, exact current schema and MIME, valid nonempty referenced blob shape, present optional Adoption receipt SHA256 syntax. Old complete Adoption without receipt remains structurally valid; no policy/body/P2 audit is implemented.
- tests/protocol_golden.rs:693 — exact carrier keys, no duplicate discriminator/no alias, defaults, future string/object/null enum values, all known enum combinations and recovery behavior.
- tests/ir_v1_corpus.rs:735, 811, 854 — old optional Adoption validity, valid/malformed receipt digests, typed fixture/body hash/length/MIME assertions, incomplete/future/nested Unknown preservation and authored rejection, schema/MIME/blob/dream-session controls.
- src/store/fork.rs:21 — ONLY adds Receipt.blob to existing referenced-blob collection. tests/ir_v1_corpus.rs:973 exercises real Store, parent replay, live fork/Unix replay and closed child headless replay, exact retained envelope/blob bytes, future enum/item values, extra fields, Unicode and whitespace; parent source remains unchanged. :1137 makes missing/corrupt actual blobs fail before child-log exposure.
- tests/fixtures/ir-v1/v1.3-receipt.jsonl is the sole added frozen JSONL: publication/run carriers and Adoption receipt pin. Attributed test-source receipt body constants (:799) are materialized through the existing real private blob store. The 77-byte publication body hashes to 08b87664441eccc0ab198042d02e3cde08eee9abd3b26398f7bf5aa5e7f45448; the 69-byte run body hashes to 7d0beae56778dd42f7ece33ccef84ec35b378ef0ef2a5c0105cfcc422bff83e4. These minimal carrier fixtures are not semantic audit positives or producer authorization.
- protocol.schema.json was generated with the existing exporter; make lock-ir additively retained all 66 definitions and expanded 15 to 16 frozen files. Each of the 15 pre-existing JSONL hashes and raw bytes matches prepared-base evidence, including v1.3-dream-session.jsonl. Prior version, tags, fields and numeric values remain unchanged.
- The five named frontend files contain mechanical read-only Receipt/ItemKind labels/matches only. verification/coverage.toml and contracts.toml add actual receipt/fork witnesses and explicit byte/schema limitations to the existing replay/fork contract. No new crate, module, lifecycle algorithm, model or broken-control semantics were introduced.

## Failure retained and canonical snapshot repair

The exact verify on clean report source 5a779b500c8f5beeb53a860c2b74133d972a317f exited 2 after every IR/native/Java parity stage passed. All 135 reachable state/transition graphs and verdicts matched, for 44 contracts and 1 crate. Its actual retained receipt is 36ad7f82edcfcf93ba32a2132697b8431e6eabec0ca6e567734023b32739652b (source 5a779b5, dirty=false), but the last make check-formal-evidence failed: "upstream main SOURCE hash changed; re-copy scripts/formal.py". This is a failed check, not green evidence. Log target/receipt-checks/resumed-exact-verify-exit2.log SHA256 1627e2c1c1e3556818d7c5c68cc211270e3b745d93d23931643df370967532c3; generated output retained at target/receipt-checks/resumed-formal-native.json SHA256 faa8cd9f0c3d728b4977bf819ae8888b62e14b81b5bb5835ca72c63a9961e7dd.

Oracle signed amendment message 23NQDJ7W8Q99PH5988TRZ4EP15 authorized exactly verification/upstream-sources.json and its three existing raw snapshots. b1dc978 copies immutable approved signed bboxy450-term commit fe8454217ba8ae43325f6d69964c1a1de15e8779, preserving all six local attribution lines and upstream origin/policy notices. The JSON names that exact revision. Byte equality against each immutable Git object and live local main was asserted before/after writes. Raw hashes:
- scripts/formal.py: 09ca7496419d59f0a41245cff8297b7fdb79dba2a9b81abf5df9e4a6597aa937.
- scripts/test-formal.py: 7627077e427cecf992e00bba1c0ed55f13843f897502f54545cd2e5dd9a4aa85.
- crates/bboxy450-tlc/examples/verify.rs: f45b999db8be8da4a18afd36ef24558c9654f9d58d642d5c30bb1ff7abbdc278.

The upstream deltas were reviewed: embedded toolkit/harness corpus plumbing, per-JVM extraction ownership, optional evidence reuse/rebinding and six further checker examples. These are retained reference bytes; the locally adapted executable scripts, exporter, model/control corpus and dependencies remain byte-identical to 4ebde7d. No upstream runtime feature is claimed as implemented by copying snapshots. Local scripts/formal.py:183 already provides per-model and graph/property Java extraction directories in this private serialized checking slot; scripts/test-formal.py:78 asserts separation. The local driver continues fresh full graph/transition/verdict exploration; evidence reuse/rebinding was not enabled, source_pins was not weakened and no toolkit override suppressed the check. No required correctness fix was identified in the local path used by these checks. Finite-corpus parity does not establish complete TLC conformance or byte/schema/implementation refinement.

## Verification and remaining limits

Historical exact source 4ebde7d exited 0 with receipt 6b4ed6ee270d5c3c7ffa227003f8f93a50517d564f6a2e9c5543fc152c7cf2e4. The prior report retains full default and ACP unit/integration suites, separate seat launch and nonPTY witnesses, clippy all-targets/all-features with warnings denied, formatting/header/secret/inventory checks, all frozen byte assertions and unattended GUI/packaging/native smoke. The initial GUI smoke exited 2 on ENOSPC; only this task-created target/debug was removed, then the serialized lean make test-gui test-app-packaging smoke-app exited 0 with actual Metal rows, unsigned private packaging/install/signature checks and hidden native startup/frame/reap. No peer target, user input, desktop control, Keychain or credentials were used. See the prior report for exact logs, exit statuses and implementation-only limits.

Fresh verification after the authorized snapshot repair is recorded below. Its exact command uses CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0, RUSTC_WRAPPER= and CARGO_NET_OFFLINE=true. A final fresh exact command will run after committing this correction; task_report must name its actual final-HEAD receipt and result, rather than copying/rebinding an earlier receipt.

Canonical duplicate KB5974 remains unchanged and unwaived; its retained actual failing full-gate evidence was not repeatedly rerun. This task does not implement receipt producer/body semantics, causal cuts, authority, P2 audit or effect interpretation. Old Adoption witnesses without receipts remain structurally readable and semantically unproven. No normal import change, dependency change, other worker tree, merge, push, publication or acceptance is claimed.

Original report-write operation 01M6JXDAPCXCQ2GJ43EGT3VHH0 remains unresolved: unknown/ledger missing/terminal=false/retry_allowed=false. Its original report locator returned not_found; that does not establish the earlier effect failed. Oracle disposition 01M6KFKBCAV8PJ1CMD0GZQFVE4 authorized the distinct resumed report. Neither that report nor this correction redispatches the original operation or overwrites it if it appears.

The post-repair unchanged exact verify on clean b1dc978ecbc3b3f055bc3bacfd75576732c80d34 exited 0, including make check-formal-evidence. Retained evidence: verification/evidence/04e41cb122cd87ce57402643d027b30004340a6f6bdf1a770aa36b4230dd5a6d/evidence.json (same SHA256, source b1dc978, dirty=false), 135 full native/Java graph/transition/verdict matches, 44 contracts and 1 crate. Log target/receipt-checks/resumed-repinned-verify.log SHA256 77861bbd984fd2496017095a7964a69d89d44d2cd417dc26dd8e1d9f642c060d. Separate python3 -I scripts/test-formal.py exited 0 (8 infrastructure/negative-control tests). The generated native report was retained as target/receipt-checks/repinned-formal-native.json SHA256 a4c268762ac2a75e29d0ad9818aea076090e56f406c119684759397e93e83e0f, then only that generated tracked file was restored; git status was clean before admitting this correction. Final signed report commit and final-HEAD receipt will be supplied by task_report after the required fresh final run.
