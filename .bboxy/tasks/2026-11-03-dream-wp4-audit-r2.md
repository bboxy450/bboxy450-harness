---
type: task
title: 'Dream WP4 revision 2: audit'
role: executor
worktree: /Users/feral/mydev/bboxy450-harness-dream-wp4
allowed_paths:
- src/dream/audit.rs
- src/dream/graph.rs
- src/dream/entry.rs
- src/dream/projection.rs
- src/dream/resolve.rs
- src/dream/deny.rs
- src/dream/mod.rs
- tests/dream_audit_broken.rs
- tests/dream_graph.rs
- tests/dream_identity.rs
- tests/fixtures/dream/good/**
- tests/fixtures/dream/broken/**
- .bboxy/reports/
- verification/coverage.toml
- verification/contracts.toml
- Cargo.toml
- Cargo.lock
verify: cargo test --test dream_audit_broken --test dream_graph --test dream_identity && make test-formal && make test-formal-parity && make check-formal-evidence
depends_on:
- '[[2026-11-03-dream-wp3-d0-r2]]'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6J8TV4EVP7GZG875FC5J4P5
status: claimed
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T07:54:51.477Z
created: 2026-11-04T03:14:54.789Z
number: 20261103136210
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
verify_revision: 1
assignee: seat:01M6J7N5V103SG1KS0THEXB9R1
provenance:
  digest: sha256:6c4f1703d7ee933259b153d3b76c5c95b55ada6a43af353cc6de642a168c4406
  frontmatter_digest: sha256:72c7e992929825e575d171c2d15e2f67be478d1c43175853b8ac065e32c903cb
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: 2v5mEhvJQwzyLvNuHZeJ3v6Juj0s3qqK+4IrU0GD9R6vvTztSR3RFArfJxmdjfpcdcU1fy5JusxcTAH0unxjCw==
  log: .bboxy/log/2026-11-04.jsonl#43725
---
ROLE: EXECUTOR
Read repository instructions and [[dream-log]], [[memory-graph]], [[dream-sinks]] (all on origin/main). Parent [[2026-11-03-swarm-dreaming-succession]].
Implement only this WP in the assigned isolated worktree; no recruitment, publication or merge. Use knowledge tools for task/report records. Oracle owns task/spec changes; report precise missing boundary lines rather than expanding scope.
Preserve additive IR, unknown round-tripping, log byte identity, redaction, unattended execution and attribution headers. Read ATTRIBUTION.md; keep origin/policy attached to generated source.
Run the listed verify command plus proportionate unit/integration/smoke coverage. Maintain make test, check_transcript and make test-formal; report baseline failures honestly with logs and exit codes. Changed stateful contracts require formal models, broken controls and process witnesses; coordinate with WP5.
Every commit is signed Boxy <bboxy450@gmail.com>. Full report .bboxy/reports/YYYY-MM-DD-dream-wpN-*.md with task, commits, verify{command,exit_code}, author seat, created and evidence for each criterion; commit with Report trailer then task_report. No secrets/raw transcripts. Do not write shared checkout source.

This is a NEW revision-2 brief against signed dream-log b4894fb6dd4e9731256760683ae2548a03f72bad §§2–12, with materially refined paths, dependencies and/or verification. Read §12 precedence because original companion mirrors remain blocked by duplicate-number host repair. Any older draft attempted during the transport outage stays unassigned; never execute duplicate work or infer assignment from record existence. Original uncertain operation IDs are not reused or replayed.

Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp4
Allowed paths: src/dream/audit.rs, src/dream/graph.rs, src/dream/entry.rs, src/dream/projection.rs, src/dream/resolve.rs, src/dream/deny.rs, src/dream/mod.rs, tests/dream_audit_broken.rs, tests/dream_graph.rs, tests/dream_identity.rs, tests/fixtures/dream/good/**, tests/fixtures/dream/broken/**, .bboxy/reports/.
Prerequisite start amendment 2026-11-04: implementation may begin on the Oracle-prepared, independently checked WP1/WP2/WP3/WP5 candidate below under [[d20261103136319-dream-checked-candidate-dependencies]]. This replaces integrated prerequisite only for isolated development; no dependency acceptance, canonical gate waiver, implementation merge or publication is authorized. Final acceptance still requires green canonical KB/full gates on a coherent main base and fresh source-bound evidence.
Outcome/acceptance: Audit P1-P9, why/graph and shared identity/pin/projection/deny validators. mod only exports/dispatch. Exact canonical identity vectors and committed pins; per-harness plus global-terminal projection; all broken fixtures reject with named P and good fixture accepts. No clock/arrival-order authority, no hand-edited body overwrite.
Verify: cargo test --test dream_audit_broken --test dream_graph --test dream_identity
Shared files are owned sequentially by the dependencies in dream-log §10. No blanket src/dream/** ownership. Report exact test outputs/exits and meaningful process witnesses; check_transcript means the actual protocol checker/invariants, not an invented shell target. Model evidence is supplied by WP5 and refreshed against final code by WP9.

Oracle source-inventory amendment 2026-11-03: every new Rust module must be listed before this package can produce fresh current-source formal evidence; scripts/formal.py enforces exact source equality. verification/coverage.toml is authorized only for this WP's actually introduced source paths, their planned-to-current promotion where relevant, and a precise note of this WP's real API/process tests and still-unmodeled behavior. Do not list nonexistent files, widen another WP's implementation scope, claim model conformance from inventory membership, omit sources to make the gate pass, or weaken a model/control/check. If a new stateful contract is not covered by the existing refined models and real witnesses, report the exact modeling gap for a separate scoped amendment. WP9 retains final cross-package contract/source/test mapping and fresh evidence responsibility; this per-package inventory maintenance cannot be deferred until WP9. Verify command unchanged; required formal/parity and canonical gates remain in force.

Oracle prepared-base amendment 2026-11-04: assigned worktree /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp4, clean comparison HEAD 7d3e22cbc4442cee8ea8bdef6e23f5225708680c. It preserves exact checked WP3 source/report 90e9c0bbeda6c3b8ebdcbcbd4dfda2b32a356726 / 7c4f1108cfd56da0bd28cb0761ec965e2fe064df and their WP1/WP2/WP5 dependency chain, plus verified signed current normative document cherry-picks through origin/main 155044619e5c563df69665be3a12337af2745315. No src/tests/Cargo/verification differences were introduced by preparation. All commands and edits MUST use WP4 workdir even though the reused seat launched in WP1. Do not edit WP1/WP2/WP3/WP5 trees or retained evidence.

Consume dream-log with [[d20261103136206-dream-contract-refinement]] and [[d20261103136344-dream-native-retained-ir-custody]]; these are in the prepared tree. Blocked original companion fetches remain number_conflict, with exact governing precedence in dream-log §12; do not manufacture repaired mirrors or a new identity. Pinned native Event evidence resolves against selected normal sessions custody, never transient Projector/cache data. D0's committed task records are locator/verdict observations; their presence/digest is not signature verification. This package must enforce §6.2 provenance.sig, origin/main membership, root/scope and record pins at the trust boundary. No matching trusted source means no trusted anchor or auto-adoption. Future trials/publications remain WP7/WP8's product code; audits need good/broken retained fixtures now, not claims that those implementations exist.

Oracle WP3 review independently ran its complete amended command: 5 dream units, 13 replay tests plus invoked private child fixtures, full 135 native/Java reachable-state/labeled-edge/verdict parity and exact final-source receipt validation, all exit 0. Log /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-wp3-review-408pvvrc/verify.log SHA256 18515788e6c7a28158e9a0e07015ab1111a9db386dbad9e5444ffa723e89653f. Receipt afc8bf0e1242ab7e73d29e54f45a4f96e31bd244061799233b0992581b4e5634 binds report HEAD7c4f110, dirty=false. Another 42 ordinary import/edge/golden/corpus regressions passed independently. Both WP3 signatures and diff-check exited 0. Existing canonical result2/KB5974 is verified from retained actual log, not waived or rerun under unchanged conditions.

Minimal extra mapping path verification/contracts.toml is authorized only to replace planned/pre-implementation references with the actual WP4 validator/projection API/process-test mappings and accurate remaining assumptions for the existing DreamLifecycle/DreamMerge/DreamSink contracts. Do not weaken a property/control, modify unrelated models, claim byte/signature proof from a quotient, list nonexistent source/test paths, or defer this package's honest mapping to WP9. Any new stateful/modeling gap gets its exact source/contract boundary reported; existing finite models plus broken fixtures and real pure/API/process witnesses must substantiate the relevant behavior.

The primary verify is now exactly: cargo test --test dream_audit_broken --test dream_graph --test dream_identity && make test-formal && make test-formal-parity && make check-formal-evidence
This supersedes the earlier three-suite-only verify string, requires a fresh final report/evidence, and refreshes current-source native/Java parity after actual new modules. Canonical make test and unattended integration/smoke remain mandatory for final acceptance; preserve and report the unchanged KB collision without filtering it or repeating an unchanged red gate. Only this package's source/tests/mapping plus its own append-only full report may change. WP9 retains final combined validation; this is implementation work, not a shortcut to accepting the swarm.

Oracle boundary ruling 2026-11-04, peer01M6JRAZNCV7TN8K42RFEK32FP: Cargo.toml and Cargo.lock are now authorized ONLY to expose the already-locked direct dependencies icu_normalizer = "=2.4.0" for NFC and ring = "=0.17.14" for existing Ed25519 verification. Preserve all unrelated versions/checksums, dependency pins/features and license/attribution; no upgrade, network install, unrelated package or new cryptographic scheme. Use the locked local upstream API/source/docs and offline checks. Existing base64/sha2/serde_yaml APIs remain available. If exposing those versions requires any additional package/version change beyond the existing locked graph, report that exact change rather than silently expanding this authorization.

A cryptographically valid self-signature checked against record-supplied authorship.public_key is NOT signer authorization. Until an independently authenticated root/signer binding is established, label the record untrusted; do not promote mere key presence/origin-ref text into a trusted anchor. The exact serialized YAML/body DSSE producer must be pinned, including its metadata exclusions and PAE encoding; do not substitute a guessed canonical JSON signing scheme. Our checked source b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8 has producer crates/bbt-mux/src/index.rs:1044–1073 (the cited913–931 belongs to another revision).

A bounded Architect task [[01M6JRSZQTWPHMG085V047H62F]] is delegated for the exact authority/bootstrap and retained P2/P4 witness schemas, consumer/producer paths and script-checkable proof. This is an independent law-writing task while your scoped pure work continues, not a permission wait. Do not invent a self-authorizing key map, historical receipt or writer causal cut. No P2/P4 success from current artifact presence, wall-clock ordering, equality after a possible ABA write, or unchecked receipt booleans. Required missing/ambiguous/unverifiable evidence is P2/P4 UNPROVEN with exit1; this is an explicit limit, not success or a silent property waiver. Keep parsing/NFC/deny/identity, pure projection and crypto byte-validation work moving within scope; consume the next signed task amendment before depending on the final new witness interface. No critical dependency on opaque Envelope.ext is authorized by this provisional ruling. Exact verify/prepared base remain unchanged; the final report must use the current amended brief.
