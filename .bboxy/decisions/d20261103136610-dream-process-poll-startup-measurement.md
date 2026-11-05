---
type: decision
title: Process poll CPU witness separates cold runtime startup from ready work
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[d20261103136528-dream-process-spawn-off-runtime-worker]]'
  rel: clarifies
  weight: 3
- to: '[[budgets]]'
  rel: preserves
  weight: 3
id: 01M6MFNMT2XDB3JYKFF5V2Z23Y
updated: 2026-11-04T23:52:52.969Z
created: 2026-11-04T23:52:52.969Z
number: 20261103136610
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:d29eb3f2438150b03a63733b9ded8ef8c4e6475c76eaded590f44e58804d9e58
  frontmatter_digest: sha256:96a171ccef12d263cd9dbc4ba0d543cf551ba4fff84276d313c7f12587ce9afd
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: 2lt5RA1eg2xaFaU3tUbTv8HKmhkNL2JaHBbNETnHSBZNKpCZ1lbI0LUoB7WuyP0YLY+epYSahrwUuQ84K8KxCQ==
  log: .bboxy/log/2026-11-04.jsonl#46279
---
Oracle ruling for WP7 blocker01M6MFHQNZKNW0KQ5DMW6SNJJH. Owners: Nyx Copperwing and Executor01M6KDR90VJTRA1S722FZFECRZ. This narrows a measurement precondition, not the1ms budget or product runtime behavior.

EVIDENCE
Signed source fe1c4e89491d83a7b5c144fc994e0e1fcdefafb8 has the unchanged inherited synth_d0_process_spawn_poll_bound_witness. Its full default log records first_poll_cpu_ns=1216875, actual child exit/reap before assertion and cargo101; log SHA2563022e21a6d803b6d773c9f34040ed18bf61dc6e4c3631681955f752408ed8482. The source-bound default.json records source_dirty=false. Do not overwrite either file.
The ACTUAL pinned registry dependency bboxy450-rt0.1.1 src/lib.rs:41–65 uses OnceLock<Runtime>::get_or_init in handle(), building the two-worker Tokio runtime. Do not substitute term main's changed source for this dependency. process.rs spawn_packet uses bbt_rt::handle().spawn; the test enters through #[tokio::test] with no explicit bbt runtime initialization before its measured poll. Headless main.rs:13 evaluates bbt_rt::handle() before block_on(cli()). These are source facts. Attribution of the historical1216875ns sample to initialization, allocation or any exact instruction remains UNPROVEN.

CHOICE / NARROW CORRECTION
Authorize tests/dream_pipeline.rs ONLY for reproducible cold/ready-runtime instrumentation of this named inherited witness and a startup-matched precondition. Existing tests/support/dream.rs may supply only its private subprocess/result-custody helpers. No src/main.rs, runtime.rs, D0 guard, pinned bbt_rt crate or ordinary process/runtime semantics are added to scope.

Before constructing/polling the ready-runtime witness's request, call bbt_rt::handle() once, as headless main does before CLI work. Measure/report that initialization separately. Do not pre-poll the tested future, pre-spawn a warmup child, fill the blocking pool, retry/sample until fast, lock/serialize tests, sleep for a quieter host, subtract measured startup from a poll, or silently add initialization inside the measured interval then discard it. Keep the exact original Request workload (3000 arguments/63000 bytes, real /usr/bin/true, private cwd, caps, request deadline/test bound), poll_cpu measurement with CLOCK_THREAD_CPUTIME_ID, owned exit/reap before assertion, and the unchanged first<=1000000ns assertion. Record every observed poll, not just a chosen minimum. A ready-runtime failure remains a failure; this authorization supplies no product-code repair or universal timing claim.

REPRODUCIBLE PHASES / EVIDENCE
Use a fresh private subprocess for each of three clearly named modes, with no unsafe reset of OnceLock or reliance on concurrent test order:
1. cold initialization only: measure the first actual bbt_rt::handle() call's thread CPU separately;
2. cold API observation: direct first actual process::run poll without explicit bbt initialization; retain all polls and real process result/reap;
3. ready-runtime product witness: one explicit initialization outside the measured future, then the same actual process::run/poll measurement and unchanged1ms assertion.
Run each once per verification command and preserve ALL observations/exits, including the cold API's raw1ms comparison. The cold observation may honestly exceed the ready-runtime budget because it includes distinct startup work; do not relabel that sample or claim cold API meets1ms. Historical cold causation cannot be proven by a later paired sample. The correction is justified by matching the documented headless startup boundary, independent of whether a later cold sample happens to be fast.

Subprocesses use only existing private helper identities/homes/paths and exact recorded children. Release private barriers, terminate if required and join/reap before a failing parent assertion; no arbitrary sleep/retry, detached child, broad kill or operator input. A real ready-runtime process run must still execute; fixture output alone or a synthetic sleeping future is not proof. Keep the existing D0 actual inline-CPU broken control green: a deliberate>=2ms CPU block during a guarded poll must still be rejected. No claimed GUI, arbitrary library embedding, all-host or cold-start1ms guarantee follows from these finite observations. Cold library API startup is measured separately; actual D0 in production retains its own full unchanged guard.

No new concurrency/lifecycle contract is introduced, so no gratuitous new model. All current175 native/Java families and pre-WP7147 model semantics remain intact; current source-bound evidence still must be regenerated after the test/report commits. Preserve child/packet/admission/settlement/cancellation/deadline/Store ownership code and existing assertions byte-identically.

CHECKS / HANDOFF
Add focused command cargo test --test dream_pipeline synth_d0_process_spawn_poll_bound_witness -- --exact --nocapture to the task verify, ensuring the test invokes the fresh modes itself (no ignored/skipped helper-only substitute). Then the unchanged primary gates and complete default/ACP matrices, separately executed canonical seat/nonPTY cases, all-feature clippy, IR and unattended native/check-transcript smoke. A ready failure stops acceptance and yields the exact new mechanism/required path; never a third unchanged-red loop. Retain actual old default101 and new cold/ready logs/hashes in separate dream-wp7-* files. A fresh signed append-only full report/task_report is required for verify revision5, with source/test/measurement limits and source/report-HEAD formal evidence. KB5974, no main merge/push/publication/live policy/bootstrap/model spend remain unwaived.