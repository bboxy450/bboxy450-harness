---
type: report
title: Checked WP1–6/hashbrown candidate integration; composed ACP deadline failure retained
task: '[[01M6M15ADG2P2F6PD5S071HPVT]]'
commits:
- c1085e903b375966535bb35bb70eb3f1a87e3543
- 620f37b5d847f90682beb5a1b196efb86f4d8ac9
- 012ff149c2f1a70d9614c8611366984893376f02
- 21bd689c070d122932381a5f49d04954adc39bda
verify:
  command: 'cargo test --lib dream::resolve:: && cargo test --lib process:: && cargo test --lib store:: && cargo test --features acp-agent --test cancel_process --test store_recovery --test store_fork --test ir_v1_corpus --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_pipeline && make test-formal && make test-formal-parity && make check-formal-evidence'
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T20:05:07.269Z
id: 01M6M2MK8AWX9G30T7CK2JJ1TF
updated: 2026-11-04T20:05:07.269Z
number: 20261103136565
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:a6a10994d8b07af8f10af1f0d7a08934bbb4cb1794c6ab3f684f89c23dc5958e
  frontmatter_digest: sha256:495624fc6103ee372a948747b2a4a8dcc18675689fa93012cfcf26e35880fe39
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: 8qQUWexfkdFKRCziHKVbOmZ3EaxwVnLQ7JvbdY/uHyqwdTO3I8AKGVolrGRMofoPWry5nezibwWnVvP96FbKAw==
  log: .bboxy/log/2026-11-04.jsonl#45836
---
The local candidate is mechanically integrated and directly tested at **21bd689c070d122932381a5f49d04954adc39bda**, on dream/integration-through-wp6. The exact assigned primary verification passes. **The required composed ACP all-target gate fails101**, exposing an inherited process-admission deadline classification race. This candidate is not ready for acceptance. No semantic repair, main merge/push, publication, live inference, trust bootstrap, other-worktree edit or KB identity repair was performed.

This report belongs to [[01M6M15ADG2P2F6PD5S071HPVT]], fetched claimed with etag61c3c30b48f3778dd22263c3bf4e83e61bda871207af489a5ad7c5db9aabfffe. Writing checkout is /Users/feral/mydev/bboxy450-harness-dream-wp4. Governing records were read through MCP: [[dream-log]], [[d20261103136206-dream-contract-refinement]], [[d20261103136362-dream-audit-trust-root-and-receipts]], [[d20261103136471-dream-constructive-receipt-seal]], [[d20261103136486-dream-synthesis-provider-store-seam]], [[d20261103136528-dream-process-spawn-off-runtime-worker]] and [[d20261103136541-dream-store-writer-lease-lifetime]]. Original conflicting companion mirrors were not repaired or substituted.

Before switching branches I asserted exact perf/dream-hashbrown HEAD9ac3c707a57a84db107cb26d21920956a42ebf03, clean index and no product/source changes. Only tracked formal-native.json dirt and the three named hashbrown post-report proof files were present. dream-integration-start.json records those guards. The exact138-model native bytes were copied to dream-integration-base-formal-native.json, SHA256d2b4d9b31de910313fbaefeb1c9e00efd1875605eb31b8fad27d5dd43b5786f8. Old branch refs, generated proof files and reports remain retained. No reset, rebase, new worktree, fetch or dependency upgrade was used.

All four original signatures and all four resulting signatures verify with approved key9B07E849480C68328CA20684844ECC0587DE7AB0; author and committer are Boxy <bboxy450@gmail.com>. Each cherry-pick is signed and retains its full original SHA and original Report trailer. No commit was empty/content-already-present.

| Original checked commit | Local signed cherry-pick |
|---|---|
|893551168ce40b7c33f7236c29ecac45dabb0aca|c1085e903b375966535bb35bb70eb3f1a87e3543|
|913db696f3ae1acd061d229c0e55acdc12a80b66|620f37b5d847f90682beb5a1b196efb86f4d8ac9|
|0c2706cad61dfdc775de265541d9a22ded906676|012ff149c2f1a70d9614c8611366984893376f02|
|c98d185bc2182f3a37ceee936031081f8c7cb2bd|21bd689c070d122932381a5f49d04954adc39bda|

Mechanical conflict dispositions and resulting byte evidence:

| Boundary | Disposition |
|---|---|
|src/dream/audit.rs|Retained the complete current9ac implementation byte-for-byte. Incoming WP6 stdout transport was already equivalent; current held writer/write_all/await flush and newer comment remain. Containing-session/full-field/full-pin/local-suffix Adoption checks, skip/Recall rules and all current sufficient/negative fixtures remain exact |
|src/dream/mod.rs|Union is exact checked WP6 mod plus pub mod seal; current-to-candidate diff is only five added export/dispatch lines. Audit/graph/receipt/trust/seal coexist with prompt/synth |
|prompt/synth/process/store/cancel_process/pipeline/support/synth fixture|Every incoming payload equals c98d185b bytes exactly; no cancellation/drop-packet/owner/lease implementation was simplified |
|tests/formal/models.rs|Only391 added lines relative to9ac. All57 extracted non-catalogue top-level function bodies from both predecessors remain exact; catalogue mechanically unions families |
|tests/formal_nonvacuity.rs|Exact current file followed by exact incoming appended process/lease tests;198 additions, no deletions |
|verification/corpus.toml|Exact unique registration union with identical common metadata;35 added lines, no deletions |
|verification/contracts.toml|Existing models/controls/property/checks fields remain exact for every existing contract. Only Lifecycle/ProviderStream API/test/assumption mappings incorporate actual D1, retaining full current WP4/cache notes; exact two incoming process/lease contracts appended |
|verification/coverage.toml|Actual prompt/synth source inventory and D1/process/lease evidence union with current WP4/receipt/seal/Adoption/cache notes. Planned trial/lower/recall remain absent and pending. No source is hidden to satisfy inventory |

The reproducible read-only instrument .bboxy/runs/dream-integration-proof.py exits0. Its proof.json records all8 signature checks,10 exact incoming files,3751 retained base files,57 exact function bodies,145 unique complete model registrations,47 contracts and1 crate. SHA256 of proof.json is62ce56e3ec9c43ec1c6de9399a736c009e28e73e67fae112e90d314d52385bb6. All16 frozen root IR JSONL fixtures and32 JSONL files across that fixture subtree retain their bytes. Cargo.toml/lock, hashbrown pin/features/cache/ATTRIBUTION, protocol/schema/lock, shared validators/receipts/seal/trust and existing tests are unchanged. Both incoming reports are retained verbatim: the original report remains historical red evidence; its resumed report is not used as a green composed-gate claim.

One initial proof-instrument run exited1 on CLAUDE.md because it followed a symlink instead of comparing Git's stored symlink target text. The instrument now uses readlink for symlinks; corrected runs exit0. No product change followed that instrument failure. The short initial explanation is retained in dream-integration-proof-initial.txt.

All following observations were made directly against the combined candidate, offline with cached dependencies, two build jobs, private HOME/TMPDIR, lean debug0/no-incremental profiles, removed seat/capability environment and no operator input. Feature builds ran serially in this checkout's existing target. Full command strings, exit codes, durations and raw-log hashes are in dream-integration-verification.json.

| Command | Actual result / log |
|---|---|
|Exact assigned primary command shown below|0; dream-integration-primary.log SHA25690fa9296cd9ac535ff01ea22c53b2126042b0060f61b7c764a17155d7dc8fe0a|
|cargo clippy --offline --locked --all-targets --all-features -- -D warnings|0; dream-integration-clippy.log|
|make test-ir && cargo test --offline --locked --test protocol_golden --test ir_v1_corpus|0; dream-integration-ir.log|
|cargo test --offline --locked --all-targets -- --skip non_pty_spawn_does_not_keep_a_sibling_master_alive|0; dream-integration-default.log|
|cargo test --offline --locked --all-targets --features acp,test-fixtures -- --skip seat_launch:: --skip non_pty_spawn_does_not_keep_a_sibling_master_alive|**101**; dream-integration-acp.log SHA256172a32bbc3cf358e9a5cb2542d908294bcbd0b1739a1fe43dcfb7ebd3c77fa78|
|cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_launch::|0; dream-integration-seat-launch.log|
|cargo test --offline --locked --features acp,test-fixtures --test seat_process non_pty_spawn_does_not_keep_a_sibling_master_alive -- --exact|0; dream-integration-nonpty.log|
|make smoke-app|0; dream-integration-smoke.log SHA25616c5ce98f76aa9adf495a87b9a77957df654f863b0748e509b42b1386dee47ee|
|cargo fmt --check && python3 -I scripts/checks.py headers && python3 -I scripts/checks.py secrets|0; dream-integration-source-checks.log|

Exact primary command:
```
cargo test --lib dream::resolve:: && cargo test --lib process:: && cargo test --lib store:: && cargo test --features acp-agent --test cancel_process --test store_recovery --test store_fork --test ir_v1_corpus --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_pipeline && make test-formal && make test-formal-parity && make check-formal-evidence
```
The three unit filters run actual3,8,7 passing tests respectively (the release cache benchmark stays ignored). The ten primary integration suites run83 passing main tests, plus explicitly invoked private child witnesses; foreign/altered/duplicate Adoption API/CLI controls and legitimate same-session positives pass. No release benchmark was rerun or performance number selected.

Full native/Java reachable-state, labeled-edge and safety/deadlock/liveness verdict comparison passes for the exact145-model union:135 common legacy,3 WP4 Seal,3 ProcessSpawnAdmission and4 StoreWriterLease. Every model has graph_equal=true; the observed aggregate is546530 states and2612304 edges. This is a full per-model comparison, not inference from counts. Candidate receipt bd6b56702ebd3b28ad07d5d910b7d8a3c17c0235a5862fa8613fb1648ea44bfe binds HEAD21bd689c, dirty=false,357 actual input hashes,47 contracts and1 crate. Complete evidence is retained in verification/evidence/<digest>/ including models and in dream-integration-candidate-evidence.json; candidate-summary.json provides receipt and suite rows. No predecessor receipt certifies this source. Final report-HEAD formal/parity/evidence is refreshed after committing this report, and its actual receipt is handed off through task_report.

Native smoke rebuilt the combined standalone app, used ad-hoc signing, rendered its own replay and created a hidden native window/Metal surface/private live frame, then exited/reaped its exact launcher. Actual bundled CLI check-transcript exits0, as do replay, launcher and strict codesign verification. Copied smoke evidence dream-integration-native-smoke.json SHA2569c72c5aabf85a5f893533d2e4011f99b460a1eea592ee437fab4a3127d9e3f17 names HEAD21bd689c/private_home=true/operator_input=false. Visible presentation was not checked. No desktop input/capture, Apple identity, Keychain prompt or live credential was involved.

The new failure is separate from KB5974. At tests/cancel_process.rs:297, process_admission_public_queue_cancellation_and_deadline_are_observable expects TimedOut for a30ms queued admission while all four blocking permits remain held. The composed ACP run observes Interrupted instead. The full gate stops at cancel_process, so later ACP targets are not represented as run/passing. Source and test bytes still match checked c98d185b. At src/process.rs:499–509 the off-worker job's deadline branch drops queued work/sender, while caller receiver closure maps to Interrupted. A deadline/channel-close delivery race is the current source-based hypothesis, not an instrumented proof of the exact scheduling history.

The blocker was sent once to the delegating Oracle (message01M6M2E3AWEDA355T5G9HKMC8E) with exact lines/log/hash and requested decision: a scoped deadline/channel-close classification repair with deterministic regression/model disposition, or a checked corrected WP6 dependency. Frozen-source authority does not authorize inventing that repair here. No deadline/assertion/property/checker was relaxed. Eight bounded invocations of the unchanged actual ACP cancel_process binary's exact queue test all exited0; their binary SHA256641feec302134786645258e569d1d389fd86b0106060f3fc15299c0dca8282da, commands and raw logs are retained in dream-integration-admission-diagnostic.json. This diagnostic does not replace or cure the composed101. The existing finite quotient does not establish concrete cross-runtime timer/channel delivery or exact io::ErrorKind classification; its green result does not refute this API failure.

Canonical make test remains unaccepted under the unchanged KB5974 collision; it was not rerun on an unchanged red premise, filtered, repaired or waived. Retained /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3 records actual make2/checker1; both conflicting record hashes are recorded in dream-integration-canonical-baseline.json. Thus neither the canonical stack nor the composed ACP requirement is green.

Candidate source-only and candidate full-commit whitespace checks against9ac both exit0 before adding this report's raw evidence; dream-integration-whitespace.json records exact commands. Raw old evidence logs remain immutable. Any whitespace findings from newly committed raw Cargo logs are reported separately without trimming their bytes. Source/index state and retained generated formal-native.json dirt are distinguished at final handoff. Old hashbrown proof files remain outside the new commit. Finite models also do not prove arbitrary Rust/kernel behavior, crypto authorization/complete producer observation, hashbrown/allocator/Hasher safety or hash quality. No new live policy, trial/publication/flush/lower/recall/run implementation or WP7–9 completion is claimed. This is a signed local red candidate handoff for the Oracle's precise repair disposition, not acceptance.
