---
type: report
title: 'Dream WP8: confirmed flush, immutable publication and quoted Recall'
task: '[[01M6MQKNA5TD4HMBKHN3JTZ09H]]'
commits:
- 09bf9e9702e24a1572117f6fabddc344abce8514
- 0978382e0efd5fe161dd320bc656f78472a9a9c9
- 228f64b9c192c258e97b40ca5f4d3ae392ab7b1a
- 8711e49cb30f8b43bc00e029691fd09d48d3b607
verify:
  command: 'cargo test --lib dream:: && cargo test --features acp-agent --test dream_sinks --test dream_recall --test dream_pipeline && cargo test --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app'
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-05T03:51:27.050Z
id: 01M6MXAFA6T8CBY80T3N61M8FX
updated: 2026-11-05T03:51:27.050Z
number: 20261103141124
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:daf3eadc4ccc93b3462fab50e59aa764c635df163098da9362b2a7127fabd221
  frontmatter_digest: sha256:bc5fde2da3bf90facbf9f78df87d2e6ec7c19c1a16a0f50a01ced5370958b941
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: lhXP90KI3zR43pVbn/qj8TSRDEYi6O8ijnbzsNa3sTXscnBZ8ddz1IPKLplrmYly2jPTsiQx8RO7FZmAB/K1AA==
  log: .bboxy/log/2026-11-05.jsonl#61581
---
Implemented confirmed flush, immutable publication and explicit quoted Recall on the accepted WP7 candidate. The exact revision0 verification below exited 0 at source 228f64b9c192c258e97b40ca5f4d3ae392ab7b1a. This is an Executor candidate handoff, not a main merge or whole dream-runtime acceptance. The operator subsequently requested only necessary checks; no extra all-target sweep, benchmark or unchanged canonical-red loop was run.

## Custody and scope

Writing tree /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp8-publication-recall; comparison base 1c86843ed128e54232a678006bb78effcab3428a. Task revision0 was fetched through MCP, etag 18ef47e416bad52f9a99d2d18d98ddbf6e33389374f8c35b3ea67d092d570882. Governing dream-log and all cited trust/receipt/seal/trial decisions were consumed through MCP. No other checkout, main source, task/spec, schema, protocol, Cargo, cache, trust bootstrap, publisher or credentials were edited.

New commits:
- 09bf9e9702e24a1572117f6fabddc344abce8514 — preserve the incoming136-model observation and approved snapshot pin.
- 0978382e0efd5fe161dd320bc656f78472a9a9c9 — declare/check WP8 finite interfaces before stateful runtime implementation.
- 228f64b9c192c258e97b40ca5f4d3ae392ab7b1a — implementation, actual API/process tests and refined caller-abandonment control.
- 8711e49cb30f8b43bc00e029691fd09d48d3b607 — actual logs, command manifest, model/fixture preservation and independent-observer evidence.

All use Boxy <bboxy450@gmail.com>, approved key 9B07E849480C68328CA20684844ECC0587DE7AB0 and this Report trailer. The host-produced report is copied byte-exact and committed separately; the final task_report names that signed report HEAD and fresh report-HEAD evidence.

The four permitted snapshot paths were compared with approved main365a7753c207be3671d791efcbec55971648c001: only verification/upstream-sources.json required a delta; three raw snapshots were already identical. The approved term pin is2b1f9c3883363555b308cdf7f38fc916797f0e53. Details/hashes: .bboxy/runs/dream-wp8-upstream-refresh.json. No executable checker or model was weakened.

## Mechanism and acceptance evidence

1. src/dream/flush.rs:24,56,109,255 — caller-authenticated Route and affine Ticket. State changes precede transport await; lost replies query original write/commit IDs, never replay an unknown effect. A matching confirmed commit must resolve actual mounted origin/main bytes, signature, complete draft fields, identity and scope before a Created event is durable. Current projection supplies status/harness metadata. No route is a labelled no-op; failures stay ineligible. tests/dream_sinks.rs uses actual private Unix JSON-RPC, signed KB records and Git commits: lost write and commit, original-ID queries, exactly one write/commit, duplicate append, commit failure, wrong root and no route. Caller-retained ticket restart persistence and a live route/trigger remain WP9; there is no credential discovery or shell KB writer.

2. src/dream/lower.rs:44,65,535,658,682 — opaque plans require measured approved producer, committed pins, current per-harness projection, shared graph closure/trust/scope/deny and thresholds@2. receipt.rs factors the existing candidate/rank/budget/render/scope builders for consumer and producer without changing old validation rules. Publication uses descriptor traversal with no-follow, owned exclusive temp, one exact write, file sync, atomic link-to-absent, parent sync and own-temp removal. Existing bytes are actually read/compared; SHA collision ambiguity refuses publication. Symlinks, hardlinks, non-files, differing resources and unknown ownership are preserved. Existing shared Claude MEMORY.md is skipped; new topics precede a new index. Codex fixed adapter/resources use the common bytes/full identity. tests/dream_sinks.rs exercises actual publication/idempotence, shared-index skip, altered full-ID resource, different harness, handwritten files, file/parent symlink and hardlink controls.

3. lower.rs:301,322,355; seal.rs Owner::effects — one owned async publication retains Store, blocking IO and admission/child ownership after caller Drop. Cancellation before link cleans its own temp and emits a skipped publication, no Adoption. Actual blocking-worker barrier/settlement process tests establish no freeze while inflight and no unowned continuation; the enabled owner-loss model control fails. Existing Body::freeze/seal::persist and ordinary Store/blob custody produce the publication, exact contiguous same-session Adoption suffix, durable sync and outer-run observations. check_adoption/check_publication/check_outer are exercised on real retained bytes, including a final run fixture. Unreceipted/unapproved/unconsumed stage output is not complete audit success. Outer production run sealing remains caller/WP9.

4. tests/fixtures/dream/sinks/observer.c is compiled separately and interposed in private child processes. It records actual libc open/write/fsync-or-F_FULLFSYNC/link/unlink/mkdir results and byte digests, independently of declared receipt effects. The verifier checks exclusive creation, single-write digest/length, actual write→sync→link→directory-sync ordering and rejects foreign writes/rename. A deliberately bad overwrite/ABA control restores the original final bytes and still fails. Wrong actual temp bytes and omitted directory sync also fail despite declared success. A real hand writer between read and link is preserved and the product skips. Retained dream-wp8-observer-*.json files contain outcomes, raw-call counts and trace hashes; no raw file bytes or transcripts. This observes this producer's actual used libc boundary; it does not prove arbitrary malicious Rust/syscalls, hostile same-UID directory/mount relocation, kernel correctness or vendor consumption.

5. src/dream/recall.rs:28,130; engine/mod.rs:147,1013; backend/acp_agent.rs:234,815 — an opaque Context is prepared off the request path through the existing full audit/current projection, exact committed pins/common render and immutable artifact verification. At the actual Engine PrepareRound/ACP first-prompt seam, durable exact Recall append+sync succeeds before separate user/reference context is inserted. Failed append/redaction, drift, terminal snapshot or wrong scope yields no bytes. System/developer policy and tool authority are unchanged. tests/dream_recall.rs captures an actual Provider request and a real private stdio ACP prompt after handoff/before user input; the receiver reads the already-durable log. Tests cover forged/altered payload, genuine append failure and terminal refusal. The lib dream::recall unit mints actual TrialCapability and runs both Engine arms: an enabled quote control proves scope would otherwise accept, while both actual trial requests/logs suppress automatic Recall.

6. recall.rs:213; import/mod.rs:207; import/edge.rs:231 — explicit validated retained publication timelines only. Actual Codex assistant text with the written oai-mem-citation resource syntax produces Cited pins; malformed suffixes/default uncertainty produce Inferred. Claude uses actual native cwd against the mounted root and only Inferred. Wrong/missing identity/timeline produces none. Real native record mapping tests preserve exact entry/record/digest pins; no guessed native JSON citation field or inferred strength is introduced. Existing exposure exclusion logic is retained.

## Verification

Exact command:
```sh
cargo test --lib dream:: && cargo test --features acp-agent --test dream_sinks --test dream_recall --test dream_pipeline && cargo test --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
```
Exit0; .bboxy/runs/dream-wp8-primary.log SHA2569a8ad2654e629d8afccc9979afee1def4f85bd337a881ceebfca8cd79764dd1d. Metadata .bboxy/runs/dream-wp8-primary.json records488.610s, source-clean, offline/two jobs/private HOME+TMPDIR/no operator input and328 source/test/script hashes. All328 still matched after execution.

79 main tests passed, with two inherited ignored cases (the cache benchmark and retained trust probe); private child witnesses were actually invoked. The exact command also executes45 formal Rust/nonvacuity/reactor tests through each required Make target. Full190 native/Java reachable-state, labeled-edge and verdict comparisons passed,57 contracts/one crate/370 current pinned inputs. Source receipt56ac1c7d80f8e30e51c02e25c3f383abc0c56ba9c3dd02d8d6122bc274fb91e7 has dirty=false. All-feature Clippy exited0.

make smoke-app exited0: fresh release bundle/ad-hoc signature, empty private HOME, hidden native NSWindow/Metal surface/live frame, actual exit/reap, replay/scroll and strict codesign. scripts/smoke-app.py:65 invokes the real binary check-transcript; that subprocess passed. Actual artifacts are retained under target/app-smoke, including evidence.json/native-process.json/launcher.log. No Apple identity, Keychain, desktop input or live service was used.

.bboxy/runs/dream-wp8-commands.json records all36 developmental command strings, actual exit statuses and raw-log hashes. Earlier compile/shape/private-fixture failures, initial missing model controls and observed missing Recall at the discarded initial Engine-context seam remain red historical evidence; each was repaired before the exact final gate. The initial noncanonical-number refusal is handled before filesystem effects without rounding rank math or relaxing the existing consumer. Unrepresentable canonical ranks remain explicitly UNPROVEN.

## Preserved evidence and limits

.bboxy/runs/dream-wp8-preserved.json and dream-wp8-model-union.json prove all16 frozen IR files unchanged, all175 legacy model/corpus/nonvacuity source bodies unchanged, every old full model record and retained TLA/cfg/metadata/state/edge artifact equal, and all370 formal inputs current. New families are Flush, Publication and Recall:3 positive models plus12 reachable broken controls; no old lifecycle/control/registration was changed. Finite quotients do not prove Rust/OS/crypto, universal cleanup fairness or arbitrary timing.

Preserved refs: dream/wp7-r3=1c86843ed128e54232a678006bb78effcab3428a; feature/dream-receipt-land-20261105=237fe8b849463017555c9d47b0064bac844d272e; landing/dream-receipt-main-20261105=365a7753c207be3671d791efcbec55971648c001. Incoming136-model bytes are preserved at dream-wp8-base-main-native136.json, SHA256abb20d8312623d7ee0f5f5fc4e8ef89bd85f5432bd2f719eef3b37cdd5507f5d. Old reports and post-report proofs were not edited or cleaned.

Grok archive/full-job-id loader authority and proposal consumption are unavailable here: those adapter components remain explicitly unsupported/UNPROVEN, with no effect or Adoption. General native/ACP trial adapters remain the accepted WP7 Inconclusive limit. Live authenticated route/trigger, durable caller-ticket restart retention, producer-policy installation, automatic current-snapshot refresh and final production run sealing are WP9. No live policy is bootstrapped. Snapshot validation establishes its observed causal cut, not instantaneous knowledge of unobserved future terminal events or vendor consolidation/forgetting.

Current main's KB collisions were fixed by afe92dae; the old candidate's historical KB failure was not repeated or represented as a current-main failure. No main merge/push/release occurred. Product source/index is clean; generated formal-native.json is retained separately from source dirt. Source whitespace check exited0. Raw historical compiler/process evidence retains original whitespace, so a blanket evidence-log diff-check reports warnings; those immutable logs were not rewritten to cosmetically pass.
