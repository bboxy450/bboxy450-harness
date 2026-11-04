---
type: report
title: 'Receipt IR carrier: distinct resumed evidence handoff after transport recovery'
task: '[[01M6JTM3T7RB0TQNDARJVY6PAS]]'
commits:
- 4ebde7d542b2414422680d07c0695a7f5ff6a752
verify:
  command: make test-ir && cargo test --test protocol_golden --test ir_v1_corpus && cargo check --all-features && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR96J84HR56MCXQ2JSW5Y
created: 2026-11-04T14:33:51.155Z
id: 01M6KFP12Y0BSM04BMSV49DS7H
updated: 2026-11-04T14:33:51.155Z
number: 20261103136410
created_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
updated_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
authorship:
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  kind: seat
  public_key: +go7EGHCYyMM/7hLFRzzAOFaKqWCHKAaxEdDnpzj8+I=
  process_generation: '1'
provenance:
  digest: sha256:9fb123c1e67ccb11b1dd2230ef56d8f6d5823f2078191ba50af06a982fc499a0
  frontmatter_digest: sha256:df7b812c0b7b177992bf8a1ad102a10755ec413972412627b471cc1c7163d906
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  sig: ri7SdlOTSPseq8SVFmG1FCq0raMky/hS/RQ7dHSw+pEpde3uWAWDv+gJM/dEf7phNM7bWnGtQOJpbF6+musyDg==
  log: .bboxy/log/2026-11-04.jsonl#44327
---
This is a DISTINCT crash-recovery evidence report authorized by Oracle disposition `01M6KFKBCAV8PJ1CMD0GZQFVE4`. It is not a retry of an earlier mutation. Original report-write operation `01M6JXDAPCXCQ2GJ43EGT3VHH0` remains unresolved: status unknown, ledger missing, terminal=false, retry_allowed=false. The original locator `reports/2026-11-04-dream-receipt-ir` returned not_found. Neither observation proves that the earlier effect failed; no original operation was redispatched and no earlier record is overwritten or declared superseded.

ROLE: EXECUTOR. Task [[01M6JTM3T7RB0TQNDARJVY6PAS]], worktree `/Users/feral/mydev/bboxy450-harness-dream-receipt-ir`, branch `dream/receipt-ir`, prepared base `7d3e22cbc4442cee8ea8bdef6e23f5225708680c`.

Signed implementation commit `4ebde7d542b2414422680d07c0695a7f5ff6a752` adds the passive Receipt carrier to the private IR 1.3 candidate. No production receipt body/audit/producer, trust policy, normal importer, dependency, other worker tree, merge or publication was changed.

## Implementation and acceptance evidence

- `src/protocol/mod.rs:212,213,696,709`: ReceiptKind/ReceiptOutcome with Unknown(Value), Item::Receipt and ItemKind::Receipt, defaulted/omitted new fields, optional Adoption.receipt, interrupted recovery, kind mapping and read-only extension text. The Oracle's amended wire contract [[d20261103136378-dream-receipt-wire-kind]] is implemented: outer kind remains receipt; Rust Receipt.kind serializes only as receipt_kind, without a kind alias. The independent blob body's kind is unchanged.
- `src/protocol/laws.rs:270,432,659,687`: dream-only Receipt writer validation, recursive Unknown rejection, current schema and structural BlobRef/MIME checks. A present Adoption.receipt is checked for SHA256 syntax; its absence remains valid for historical complete Adoption records. No P2 semantic authority is inferred.
- `src/store/fork.rs:21`: Receipt.blob joins only the existing reference collection; ownership, no-replace publication and all other fork lifecycle mechanisms remain unchanged.
- `src/frontends/{acp,seat}.rs` and `src/frontends/gui/{view,smoke,face_rows}.rs`: mechanical passive exhaustive arms and labels only.
- `tests/protocol_golden.rs:693`: known enum combinations, exactly one discriminator, receipt_kind/no alias, missing fields/defaults, future enum scalar/object/null, optional schema fields, old wire behavior and recovery kind identity.
- `tests/ir_v1_corpus.rs:735,811,854`: old Adoption without receipts remains authorable; valid/malformed optional digests, new/incomplete/future carrier, nested Unknown, current schema, MIME and BlobRef checks, non-dream writer rejection and interrupted recovery.
- `tests/ir_v1_corpus.rs:799,973,1137`: attributed canonical fixture byte constants materialize actual private session blobs. Closed Store replay, live-fork Unix replay and closed-child headless replay preserve envelope/blob bytes, whitespace, Unicode, unknown ItemKind/ReceiptKind/Outcome/SinkKind and extra nested fields. Parent bytes remain exact. Missing/corrupt referenced blobs refuse fork before a visible child log. A typed actor startup suffix is allowed; every retained prefix byte is still asserted.
- `tests/fixtures/ir-v1/v1.3-receipt.jsonl`: the only new frozen JSONL, ten durable envelopes including publication/run receipts, Adoption.receipt and an attribution Notice. Real blob hashes: 08b87664441eccc0ab198042d02e3cde08eee9abd3b26398f7bf5aa5e7f45448 (77 bytes), 7d0beae56778dd42f7ece33ccef84ec35b378ef0ef2a5c0105cfcc422bff83e4 (69 bytes), MIME application/vnd.bboxy.dream-receipt+json.
- Schema generated with existing --export-schema, then make lock-ir: 66 definitions/16 files. Assertions against the prepared base lock prove all 15 prior frozen JSONLs and hashes are unchanged; version remains 1.3. No old fixture or tag/default/value changed.
- `verification/{coverage,contracts}.toml`: only existing fork/replay implementation-witness mappings and explicit remaining byte/schema/semantic limits changed. No crate/module/model/control semantics changed. The fork quotient is not claimed to prove bytes or receipt truth.

## Actual verification

Exact task command: `make test-ir && cargo test --test protocol_golden --test ir_v1_corpus && cargo check --all-features && make test-formal && make test-formal-parity && make check-formal-evidence` — exit 0 on 4ebde7d. It ran 16 corpus and 9 golden tests, all-features compilation, native formal controls, exact pinned-Java parity and current-source evidence validation. Retained archive: `verification/evidence/6b4ed6ee270d5c3c7ffa227003f8f93a50517d564f6a2e9c5543fc152c7cf2e4/evidence.json`; source 4ebde7d, dirty=false, 135 models/44 contracts/one crate. Complete reachable states, labeled transitions and registered verdicts match; incomplete/unsupported checks were not accepted. `make check-formal-evidence` revalidated it on recovery.

All-target default and ACP/test-fixtures suites exited 0. Canonical separate launcher cases and exact non-PTY CLOEXEC witness also exited 0; private HOME/TMPDIR, no live seat/capability inheritance and no operator input. Results retained at `target/receipt-checks/results.json`; output `/tmp/dream-receipt-coverage.log`, SHA256 706c1800fba354ceeb4c58555ae572a8a9959c8b0428949388a07506370b1faa.

`cargo clippy --offline --locked --all-targets --all-features -- -D warnings`, fmt, headers, secret scan, inventory shape and git diff checks exited 0. All 25 scoped protocol/corpus tests and six focused Receipt cases passed. Logs: `/tmp/dream-receipt-clippy.log`, `/tmp/dream-receipt-protocol.log`, `/tmp/dream-receipt-focused.log`.

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 RUSTC_WRAPPER= make test-gui test-app-packaging smoke-app` exited 0, serialized within this worktree's target. Actual Metal frame: 21 visible items/14 kinds; private ad-hoc installation/signatures, hidden native window/surface/live frame and process exit/reap. No Apple identity, Keychain or desktop input. Log `/tmp/dream-receipt-smoke-lean.log`, SHA256 b9bc41e30df3aa951188019480b0c401b0d5fdd34db43b6c6a1d781d1a550608.

Initial GUI attempt exited 2 due ENOSPC (os error 28, 121 MiB free), not a product pass. Only this task-created target/debug directory was removed, retaining formal archives/results; free space recovered to 8.14 GiB. The lean build then passed. No peer cache/process, other worktree or retained formal evidence was cleaned. The failed attempt remains in the coverage log/results.

Two development transport-test assumptions failed: a closed gateway admitted no Unix subscription (EOF), and a live actor appended a typed startup suffix (head 10 versus 9). Final tests use a live fork gateway, preserve/compare every original prefix byte, allow only typed appended suffixes, and compare complete closed-child headless bytes. No product assertion was waived.

## Limits and retained state

The unchanged duplicate KB5974 remains in d5974-log-ir-v1-only-add.md and d5974-memories-are-kb-records.md; git-object assertions confirm both match prepared base 7d3e22c. Retained canonical evidence is KB checker exit 1 / make test exit 2. This assignment explicitly forbids rerunning that unchanged red premise; it was not repeated, repaired or waived. No overall canonical-green or acceptance claim is made.

Fixture blobs are carrier fixtures, not approved-producer/audit-positive bodies. Receipt body semantics, causal cuts, producer/effect completeness, signer trust and P2/P4 audit remain WP4. The passive carrier and immutable structural predicates introduce no new ownership/retry/arbitration algorithm; existing model/control semantics are unchanged. Real API/process tests cover the new finite byte cases.

Implementation was completed under former authenticated seat 01M6J7N5WZ9XEJXWRF45SA6FQ9; restored seat 01M6KDR96J84HR56MCXQ2JSW5Y is separately assigned the same task. Signed source, 15 unchanged old fixtures, clean worktree and the retained current-source receipt were rechecked after restoration. The original report-write operation 01M6JXDAPCXCQ2GJ43EGT3VHH0 remains unknown (ledger missing, terminal=false, retry_allowed=false); it must not be redispatched. Any distinct resumed report is a new authorized handoff, not a claim that the original write failed or succeeded.

The report commit will be signed with its Report trailer, then the exact primary and current-HEAD receipt will be refreshed under lean debug-symbol settings before task_report. Generated run receipts/screenshots are retained under target/receipt-checks/retained-runs and restored in Git. No merge/publication is authorized.