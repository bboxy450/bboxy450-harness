---
type: report
title: Dream WP4 pure verifiers checkpoint; full receipt audit incomplete
task: '[[01M6J8TV4EVP7GZG875FC5J4P5]]'
commits:
- b48d44cd5123a9842dd96b25cdf1a1313c07810b
verify:
  command: cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T14:35:40.038Z
id: 01M6KFSB6JQA1VRP4XCA7FZ26Q
updated: 2026-11-04T14:35:40.038Z
number: 20261103136420
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:fc88352126c37dbbb15d47a317c761c714469297224c15d7589c43b122c2ddac
  frontmatter_digest: sha256:2ad335e9b60d5d8e9c8c3691df3b71babaf0d8595936687e777a94e2bb78e4d8
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: /DQPiE3SKGAjfIlDTSa7qi7+CULNdEJYGUTUgejhS7q/r9qYtyhFpPk4TUri9BQSXuTNMjkMhVZ5JewT9FfHDg==
  log: .bboxy/log/2026-11-04.jsonl#44413
---
# WP4 pure verifiers checkpoint — full audit remains incomplete

This is a committed implementation checkpoint, **not a completion, acceptance, merge or publication report**. The current scoped task is `01M6J8TV4EVP7GZG875FC5J4P5`, fetched after reassignment to seat `01M6KDR90VJTRA1S722FZFECRZ`, etag `879fd8c5c796f451eddcae11168fe7e8cf609e4f18b044852d373ed619d17137`. Source commit `b48d44cd5123a9842dd96b25cdf1a1313c07810b` is signed Boxy <bboxy450@gmail.com>; `git verify-commit` exited 0. Comparison base remains `7d3e22cbc4442cee8ea8bdef6e23f5225708680c`, branch `dream/wp4`, worktree `/Users/feral/mydev/bboxy450-harness-dream-wp4`. No rebase, merge, push, publication, live-policy bootstrap, protocol edit or other worktree edit was performed.

## Boundary and remaining decision

Consumed the current authenticated brief and both complete pages of decision `01M6JSCCZQBYTQ2490GP20SCSV` / d20261103136362, revision 2, digest `7e6856f3f1d8410b41d97714884c9cbe591e4ee166c604c88e20605861c3d82e`. The brief approves its V/T, trust/2 policy, approved artifact checks and pre-seal E for pure implementation, while explicitly rejecting its circular all-mutations sealing claim. That narrower ruling governs this checkpoint.

The exact remaining inputs are (1) the reviewed constructive seal schema, pre-seal interval and allowed post-seal custody operations, retained blob/header bindings, publication sealing observed by the outer run, and owned-tool/child coverage rules/model/control/witness; and (2) the Oracle's independently verified typed Receipt carrier integration commit and amendment. No carrier integration commit was supplied. WP4 cannot edit protocol/schema/corpus or another worker's tree. This checkpoint therefore does not invent R, final receipt encoding/builder, causal cuts or coverage assertions.

`audit.rs:183` returns **P4 UNPROVEN missing_run_receipt for every dream session**. Every Adoption still receives P2 UNPROVEN; current artifact equality never proves publication history. The proposed-only CLI fixture now asserts exit 1 rather than masquerading as the required good audit. The complete adopted/publication good fixture, receipt_header_body_roundtrip, accepted R, P2 cut/projection/selection/adoption checks, P4 retained source/seal checks, and positive sink-path:line provenance are still **unimplemented and unproven**. Work remains claimed; no task_report completion is submitted by this checkpoint.

## Implemented mechanisms and evidence

- **Identity and byte preservation.** `entry.rs:31` defines the seven-field identity; canonical JSON recursively sorts keys, normalizes keys/strings with the existing locked ICU NFC implementation, sorts the specified evidence/predecessor arrays and refuses NFC key collisions. Fixed independently computed UTF-8 vectors include composed/decomposed Latin and Hangul, pin differences, evidence ordering and revival. Exact record/body bytes remain distinct from projection metadata. `dream_identity` checks byte digest changes, stale metadata versus changed content, bounds, nofollow ancestor/final symlinks and CRLF Event digest identity.
- **Deny and scope.** `deny.rs:13` retains all ten baselined deny families plus registered-secret checks, recursive value/key redaction, tag neutralization and instruction-path detection. `resolve.rs` resolves only mounted roots and selected normal retained sessions; paths are confined to the KB, blobs must be regular files and commits actual commit objects on the captured branch. Legacy and typed locator spelling stays in identity. Unknown scope/source fields do not become invented facts. Private Git/process fixtures check committed opt-in versus working-tree changes and no custody from cache/transient import state.
- **Exact KB V.** `trust.rs:24` owns the sole DSSE byte verifier; the compatibility helper in resolve delegates to it. It checks exact UTF-8 LF/frontmatter boundaries, provenance-last canonical serde_yaml/serde_json re-rendering, exact body/frontmatter digests, the pinned 128-byte hexadecimal payload and DSSE PAE, canonical padded base64 lengths, Ed25519 and signer/id/generation bindings. The pinned producer is term `b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8`, index.rs:1044–1073. The retained real producer blob has SHA256 `51e052c1f90d3322351848263d9ce1097a9f31e30860bf00d6bd95577c5f5a36`. Tests reject byte/YAML/crypto/binding controls; unsigned provenance.log is not authority.
- **Independent T and approved artifacts.** `trust.rs:465,660,711` load a strict external trust/2 policy and implement scoped repository binding, captured branch membership, exact pinned bytes, last-touch blob/signature checks, allowlisted OpenPGP fingerprints and ancestry revocation. Policies inside repositories/sessions/sinks, symlinks, unsafe ownership/mode, aliases/duplicates and key fingerprint mismatches fail closed. No product policy is created. Public-key inspection/import/verification use fresh short private keyrings, no operator keyring, no autostart, network, Git tracing, hooks or prompts, with the existing bounded owned process runner. Approved artifacts bind source/tree/version/build/executable/capabilities, signed gate bytes, required zero-exit witnesses and an independently trusted accepted task/report ancestry. Real private OpenPGP/Ed25519/Git fixtures test authority, K2, revocation, resolver integration, accepted-task joins and changed executable/old commit/capability/gate negatives. Synthetic approval fixtures are not live producer approvals.
- **Pre-seal E, without an R claim.** `receipt.rs:96,184` strictly parse and interpret the admitted operation vocabulary over explicit per-path state. Dense indices, temp ownership/digest/fsync, no-replace outcomes, directory sync, idempotence/conflict, source-session ownership and cache boundaries are checked. Tests reject unowned/unsynced/wrong-digest sources, missing directory sync/result, unknown fields/effects, rename/unlink/truncate/chmod, ABA and inconsistent or falsely successful links. The shared API does not certify effect completeness or solve the still-unbaselined seal.
- **Projection, audit and graph.** `projection.rs:198` unions validated per-harness/global-terminal facts without arrival clocks, requires exact same-pair causal trial pins and known numeric thresholds, and prevents terminal reopening. Trial gates check independent arms, cost provenance/comparability, spend, control harm and exact median arithmetic. Failed trial gains are retained for optional graph strength. `audit.rs:126` invokes the real protocol checker on captured fork projections and checks P1/P3/P5–P9/identity aspects with named failures; missing P2/P4 proof is UNPROVEN. `graph.rs` retains finite provenance/external links, deterministic record origins, read-only derived strengths and redacted resolved evidence snippets. Accepted recall strength is joined only through T-validated accepted task records, not D0's unsigned verdict observations. mod.rs changes only exports/dispatch.
- **Real read-only smoke.** `dream_audit_broken` materializes retained P1–P9 templates in private signed repositories, runs the actual audit/graph/why CLI, asserts named failure/exit status, and verifies unchanged source/record/sink hashes and no extra session. A matching existing artifact without history remains UNPROVEN. These are rejection/read-only witnesses, not the missing successful historical publication fixture.

## Collected commands

At clean source commit b48d44c, the exact current primary command was:

```
cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust && make test-formal && make test-formal-parity && make check-formal-evidence
```

Exit **0**: **30 primary tests** (3 audit, 8 graph, 13 identity, 6 trust); native formal/nonvacuity/projection 4+31+1, the 8 script-control tests, and full **135-model / 44-contract / 1-crate native/Java reachable-state, labeled-edge and verdict parity**. Retained formal receipt `8cdd70c39aa327bfffbc529d786d5070ac8a5410162927c971e94f853aa95877` binds b48d44c with dirty=false and exact current source/model/tool hashes. It is a checkpoint receipt, not final combined carrier/seal evidence. Primary log `/tmp/dream-wp4-primary-b48d44c.log`, SHA256 `da707c64e50e48507c8821c16253e2bd86b09b12b6b448a7f41f0e8405570271`.

`make test-ir && cargo test --offline --locked --test protocol_golden --test ir_v1_corpus --test dream_replay --test ir_edge --test protocol_laws --test store_fork --features acp-agent` exited **0**: 64 locked definitions, 10 refused/4 accepted compatibility controls, **47 tests** plus explicitly invoked private child fixtures; the two child entry points remain ignored in the ordinary replay listing. Real replay/engine/ACP/watch smoke and fork/corpus byte custody passed. Log `/tmp/dream-wp4-regressions-b48d44c.log`, SHA256 `ddb62b19044765794c5c84a461e32d12ed3e0431d344b96b8b8b78c537793e87`.

`cargo clippy --offline --locked --all-targets --all-features -- -D warnings` exited **0**. Log `/tmp/dream-wp4-trust-clippy-3.log`, SHA256 `df6e8a691d23fb87805a01791d0c895dc46a8eb9df0efaaac1f84edde0e49420`. fmt, diff whitespace, Rust headers and exact source inventory checks exited 0. Python assertions confirmed all 15 inherited frozen IR fixtures byte-identical to the prepared base and every other locked package/version/checksum/dependency unchanged. Cargo exposes only the two specifically authorized existing direct packages (ICU 2.4.0 with existing compiled_data and ring 0.17.14).

Initial development checks were not passes: trace flattening initially rejected valid operations, flattened producer fields initially admitted an unknown field, and a large composed unoptimized async future caused a stack overflow. Strict explicit wire structs and boxed owned capture futures corrected these; current full suites run on the ordinary test stack without increasing it. A long public keyring path initially made GnuPG unavailable; a fresh short private path and no-autostart verification removed that dependency. Earlier WP4 work also recorded the ENOSPC check failure; no peer target/evidence cleanup was performed. Generic diagnostics never echo secret matches.

## Gates, scope and limits

The unchanged canonical duplicate KB **5974** failure remains **unwaived**. The task explicitly carries the retained actual WP3/Oracle canonical result (make exit 2) and forbids another unchanged red loop. WP4 has not rerun or filtered that gate, repaired numbering, claimed canonical green, or claimed final acceptance. Fresh complete canonical/GUI/packaging/night validation remains required after the final coherent dependency/seal implementation; it is not replaced by these scoped tests.

coverage.toml lists all actually introduced modules; only the three existing DreamLifecycle/Merge/Sink mappings and precise implementation assumptions were updated in contracts.toml. No model, property, broken control or checker was weakened. E is a trace abstraction of the existing finite DreamSink model. Actual producer completeness/owned children, byte/crypto/kernel semantics and universal timing/Rust correctness remain implementation evidence or explicit assumptions; finite parity proves none of them. WP9 retains final combined mapping/evidence responsibility.

Only assigned source/test/fixture/Cargo/mapping paths and this append-only report changed. Mandated formal commands also refreshed their own generated formal-native/current evidence; those generated files are retained, not committed or used to modify any peer tree. Source was clean at the recorded b48d44c gate. The later report commit changes HEAD, so a fresh post-report parity/evidence refresh is still necessary and will be retained separately; no b48d44c receipt is represented as covering another HEAD.
