---
type: report
title: Process admission preserves cancellation and the elapsed deadline
task: '[[01M6M15ADG2P2F6PD5S071HPVT]]'
supersedes: '[[reports/2026-11-04-dream-through-wp6-integration]]'
commits:
- 9fb4b1111b6e524a7c6de070156b33856ef87b1e
verify:
  command: 'cargo test --lib dream::resolve:: && cargo test --lib process:: && cargo test --lib store:: && cargo test --features acp-agent --test cancel_process --test store_recovery --test store_fork --test ir_v1_corpus --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_pipeline && make test-formal && make test-formal-parity && make check-formal-evidence'
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T20:33:32.765Z
id: 01M6M48MEPXRTAB2V88F92ENXE
updated: 2026-11-04T20:33:32.765Z
number: 20261103136575
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:b102ab24dc59d030240c1cf8f648029f18091689dbe3ea42987550e6eb13b6ca
  frontmatter_digest: sha256:e5d35631c659de0edc6cd249011152eb1071f99365c39cea15be7063a46672af
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: lBRgBTYKSlCa6o6n+iEAG3dlROeX3rPZ5jYTSv79Cv2D3i8zBmid7VGVCOSZmT2fQYX14q1vBLMySoKIIXrYCw==
  log: .bboxy/log/2026-11-04.jsonl#45945
---
The combined WP1–6/hashbrown candidate now preserves the caller's observed cancellation and absolute deadline when receiving a process result. Signed source **9fb4b1111b6e524a7c6de070156b33856ef87b1e** fixes the deterministic closed-channel classification defect. The exact primary, composed default and ACP all-target gates, isolated canonical seat cases, all-feature clippy, IR/golden and native app/check-transcript smoke all exit0. Canonical make test remains held by unchanged KB5974 and is not claimed green.

This is the fresh report for [[01M6M15ADG2P2F6PD5S071HPVT]], superseding [[reports/2026-11-04-dream-through-wp6-integration]] without changing that report or any old evidence. The signed task amendment (fetched etag47dd20a04902d90e8f7ac1886700885f089d82c9017cf4202d6efd01a0115d30) authorizes this exact exception. I read the complete Oracle ruling [[d20261103136567-dream-admission-result-observation]], id01M6M2PFYMG5HQV0KG2AV6FZZF, digest3a908ea0060f3440909203837bb75ae92dc3aeadc71f699079c9ab18479a392b, before editing. All work stays in /Users/feral/mydev/bboxy450-harness-dream-wp4 on dream/integration-through-wp6. No other worktree, WP6 backport, new task/cohort, dependency upgrade, main merge/push, live inference, policy bootstrap, publication or KB identity repair occurred.

The retained integration base is9c3251e01348a6ab96804a7473fe17f8d1234e4a. It already contains these signed cherry-picks, whose original/new signatures and bytes were checked in the preceding report/proof:

| Original | Integrated |
|---|---|
|893551168ce40b7c33f7236c29ecac45dabb0aca|c1085e903b375966535bb35bb70eb3f1a87e3543|
|913db696f3ae1acd061d229c0e55acdc12a80b66|620f37b5d847f90682beb5a1b196efb86f4d8ac9|
|0c2706cad61dfdc775de265541d9a22ded906676|012ff149c2f1a70d9614c8611366984893376f02|
|c98d185bc2182f3a37ceee936031081f8c7cb2bd|21bd689c070d122932381a5f49d04954adc39bda|

perf/dream-hashbrown stays at9ac3c707a57a84db107cb26d21920956a42ebf03 and dream/wp4 at71b4adab4dd80018de6338ccbfa751ac78649848. The original guard/native138-model copy, both WP6 reports, original four-pick dispositions, WP4 full audit/Adoption/seal corrections and hashbrown attribution/cache/pins remain retained. The previously untracked145-model report-HEAD proof is also retained separately; dream-integration-outcome-base-native.json preserves its native JSON before147-model refresh.

src/process.rs:470 adds one private receive helper. The actual receive arm samples cancellation, then now against the SAME absolute deadline: cancellation returns Interrupted; otherwise now>=deadline returns TimedOut; otherwise the explicit child/io error or unavailable Interrupted is preserved. Existing biased outer cancellation-first and timer arms remain. Early refusal drops the still-owned nested SpawnResult, invoking its existing kill/fenced-wait/reap path. No new owner, scheduler, queue, retry, timer, lock, sleep or budget is introduced. Observations are local reads at this boundary, not a reconstructed atomic historical timestamp.

The repair/proof boundary is precise:

| Criterion | Concrete mechanism/evidence |
|---|---|
|Deterministic original defect|src/process.rs:942 selects a closed oneshot receive with an explicitly pending caller timer and observed now==deadline. The actual production receive helper executes. Extracting the old helper mapping gives cargo101 (Interrupted vs TimedOut), then the corrected helper gives0; no outer ready timer supplies the verdict |
|Priority/equality/error behavior|process.rs:955 covers closed-before-cutoff unavailable, cancelled-before-cutoff, cancellation+elapsed priority, exact equality, strict elapsed and explicit EACCES forwarding/override. Before cutoff the original raw OS error remains EACCES |
|Live and late ownership|process.rs:1012 creates four actual private sleep children using the existing blocking facility. Live success returns the same PID/owner; elapsed/cancelled packets refuse and retire through existing SpawnResult. Existing reaped helper asserts ECHILD and exact group ESRCH after fenced wait; unexpected success is explicitly killed/waited before test failure. API refusal alone is not treated as cleanup joined |
|Public API/current assertions|tests/cancel_process.rs:413 adds zero-budget refusal/cancellation-priority/no-execution/upstream-token checks. Every old assertion and all old bytes in that file remain an exact prefix, including the original30ms queue deadline assertion |
|Narrow source change|Only process.rs, cancel_process.rs, models.rs, formal_nonvacuity.rs and three inventory/contract/corpus TOMLs change from9c3251e. The proof removes only the new helper/tests and reverses the receive call to reproduce the entire original process.rs exactly: worker admission, packet Drop, signals, environment filtering, ownership and other tests are unchanged |
|No collateral change|dream-integration-outcome-proof.py verifies3795 retained tracked files, old reports/evidence, full audit/Adoption/cache/Store/schema/dependency/fixture bytes, all57 old non-catalogue model functions and old145 registrations/47 contracts. Audit stdout/flush and all containing-session/full-pin Adoption controls remain intact |
|Model preservation|The proof compares all145 prior models' canonical states, labeled edges, metadata, TLA, normal cfg and graph cfg byte-for-byte with the retained41ffbe70… archive. Only the new good/broken pair and its mapping are added; the generic exporter is unchanged |

The proof exits0; dream-integration-outcome-proof.json SHA2567234ac9000f559e3f153e8547c570e481ac9291c23cbbb47abfb62e4d2f67d16 contains the file/model hashes and assertions. Cargo pins/features, ATTRIBUTION and all16 frozen root IR JSONL fixtures stay unchanged. No release cache benchmark was rerun.

Test-first evidence is durable: dream-integration-outcome-red.log SHA256565bd9bad8116faf772ca97969c93ccfec3e907ecc2a0b35a779dc345aaef82f exits101 on the one exact test. outcome-old-helper.rs is the actual old-behavior extraction snapshot SHA256f08a652b4b1d98b4afb5f67b60eb917fc0ac4408329750a704d30bd4b5191dee; outcome-red.json records command/disposition. The focused corrected process suite exits0 with11 tests, outcome-green.log SHA256dfa2b82292cda70584acf6f856a405460df9aebdc14509edf82d9d1b1da66e3c. The original composed ACP101 log remains byte-identical SHA256172a32bbc3cf358e9a5cb2542d908294bcbd0b1739a1fe43dcfb7ebd3c77fa78. Earlier isolated8 passes and145-model receipts remain historical observations, not repair certification.

The new finite family is HarnessProcessAdmissionResult and BrokenHarnessProcessAdmissionResult. Six variables are observed cancellation, elapsed deadline, caller timer readiness, worker deadline observation, received kind (pending/closed/success/error) and returned class (pending/Interrupted/TimedOut/success/error). Inputs freeze at return. The ordered safety properties are ObservedCancellationHasPriority, ObservedElapsedResultIsTimedOut and BeforeCutoffPreservesOutcome. No fairness or cleanup progress assumption is added.

Before correcting production behavior, the new model/nonvacuity test exits0 (outcome-model-before.log SHA256e2980f9cb1ebeda2f6be11b1160f89f1e25d354ebe329a63821dbf1f5683d1cf); its source hash still matches current models.rs. formal_nonvacuity.rs:1628 reaches the exact uncancelled closed/elapsed/timer-not-ready case: good [0,1,0,1,1,2], old-mapping control [0,1,0,1,1,1]. It also reaches cancellation priority for all kinds and before-cutoff handoff/error/unavailable. outcome-exact-traces.json checks actual labeled edges ObserveElapsedDeadline → WorkerDeadlineClosesChannel → ReturnTimedOut1 / OldMapping1. The old-mapping control violates the same timeout property, rather than an invented unrelated control.

All147 native/Java reachable-state sets, labeled-transition sets and safety/deadlock/liveness verdicts match:145 unchanged plus2 new models,48 contracts/1 crate. The pair has50 states/76 edges each; good TLC0 and broken expected safety TLC12. Aggregate546630 states/2612456 edges is an observation, not the basis for conformance. Fresh source receipt **02f765b083ac9e2779d740866d0c589537a9825fb5d3629a35488e7493460e68** binds9fb4b111,dirty=false and357 actual input hashes; complete models/receipt are in verification/evidence/<digest>/ and copied outcome-candidate-evidence.json/summary.json. Predecessor145-model receipts cannot certify this exception.

Exact assigned primary:
```
cargo test --lib dream::resolve:: && cargo test --lib process:: && cargo test --lib store:: && cargo test --features acp-agent --test cancel_process --test store_recovery --test store_fork --test ir_v1_corpus --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_pipeline && make test-formal && make test-formal-parity && make check-formal-evidence
```
Exit0; filters run3 cache,11 process and7 Store tests; the ten integration suites run84 main tests plus explicitly invoked private child witnesses. Full current command/exit/raw-log/hash rows are in dream-integration-outcome-verification.json:

| Command | Exit | Raw log |
|---|---:|---|
|cargo test --lib dream::resolve:: && cargo test --lib process:: && cargo test --lib store:: && cargo test --features acp-agent --test cancel_process --test store_recovery --test store_fork --test ir_v1_corpus --test dream_identity --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal --test dream_pipeline && make test-formal && make test-formal-parity && make check-formal-evidence|0|dream-integration-outcome-primary.log|
|cargo clippy --offline --locked --all-targets --all-features -- -D warnings|0|dream-integration-outcome-clippy.log|
|make test-ir && cargo test --offline --locked --test protocol_golden --test ir_v1_corpus|0|dream-integration-outcome-ir.log|
|cargo test --offline --locked --all-targets -- --skip non_pty_spawn_does_not_keep_a_sibling_master_alive|0|dream-integration-outcome-default.log|
|cargo test --offline --locked --all-targets --features acp,test-fixtures -- --skip seat_launch:: --skip non_pty_spawn_does_not_keep_a_sibling_master_alive|0|dream-integration-outcome-acp.log|
|cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_launch::|0|dream-integration-outcome-seat-launch.log|
|cargo test --offline --locked --features acp,test-fixtures --test seat_process non_pty_spawn_does_not_keep_a_sibling_master_alive -- --exact|0|dream-integration-outcome-nonpty.log|
|make smoke-app|0|dream-integration-outcome-smoke.log|
|cargo fmt --check && python3 -I scripts/checks.py headers && python3 -I scripts/checks.py secrets|0|dream-integration-outcome-source-checks.log|

The source primary log hash is5d456cb0f033eef4afe012b2be9b6afc052fb124441e1fc21432b7369dc7b002; corrected full ACP hashc5f469f276c480e16d641cd042b1b94e14de00e7ab79c9696cc441af9844ce5b includes the previously failing unchanged queue test passing and the remaining ACP targets actually run. This is a single full rerun after the semantic correction, not repeated unchanged attempts until green.

Builds are serialized in this checkout's existing target, cached/offline, two jobs, debug0/no-incremental, private HOME/TMPDIR and removed seat/capability environment. Pure test-first helper/model checks do not access an operator account. No test needs operator input. Native smoke rebuilds the corrected standalone source, ad-hoc signs, renders its own replay, creates a hidden NSWindow/Metal surface/private frame and exits/reaps its exact launcher. Actual bundled check-transcript, replay, launcher and strict codesign all exit0. outcome-native-smoke.json SHA25643f786d690394b87d2634fabad3e3c1a64fc59e74dc83a696f88fd675f9fe415 names9fb4b111,private_home=true/operator_input=false; visible presentation is not claimed. No desktop input/capture, live credential, Apple signing identity or Keychain prompt is used.

This report/evidence commit preserves the tested source/dependency/model bytes. After committing it, the COMPLETE exact primary command is rerun at report HEAD, including fresh147-model native/Java/reference evidence; task_report supplies that actual final SHA, log/receipt and source/input equality. It does not transfer the9fb receipt to an unknown later HEAD.

The new quotient proves only its finite observation rule. It does not prove real clock precision, atomic cancellation/clock history, arbitrary Tokio/channel/syscall scheduling, Rust/kernel refinement, cryptographic authority, complete producer observation or hashbrown/allocator safety/hash quality. Actual child cleanup is separately witnessed, not inferred from refusal or finite model membership. Existing ownership and Store models/bounds/properties remain unchanged. WP7–9 trial/flush/lower/recall/run integration and live producer approval remain unimplemented by this task.

Canonical make test remains held by the same duplicate KB5974, without a repeated unchanged red loop or waiver. Actual retained make2/checker1 log /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3 and both record hashes remain in dream-integration-canonical-baseline.json. Source whitespace checks pass. Full-patch raw-log EOF findings are recorded honestly at final handoff; old/new raw bytes are not trimmed. Source/index cleanliness is reported separately from retained generated formal-native.json and named post-report proofs. All commits use Boxy <bboxy450@gmail.com>, approved key9B07E849480C68328CA20684844ECC0587DE7AB0 and Report trailers. This is the corrected isolated candidate handoff for independent Oracle verification, not main acceptance or publication.
