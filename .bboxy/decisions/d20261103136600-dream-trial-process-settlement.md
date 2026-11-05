---
type: decision
title: Trial process results and observable settlement retain scratch and claim ownership
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[d20261103136594-dream-trial-engine-capability]]'
  rel: decision
  weight: 3
- to: '[[d20261103136528-dream-process-spawn-off-runtime-worker]]'
  rel: decision
  weight: 3
id: 01M6M7NN9M2ZJBQZ6MJEXAHE4K
updated: 2026-11-04T21:33:05.027Z
created: 2026-11-04T21:33:05.027Z
number: 20261103136600
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:19723c4da2dfae76dd7b252d7d83c721886f3ca95279cd55392df86634b47c94
  frontmatter_digest: sha256:c1e1e63692c13c3718aa9b8ab079ed44a1d140303fd7c862c4fcb18cc8c4ba03
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: sY2yoTMKDjtXs5uIP+NG+ygr1II8Tgb3UVbmBamK+ZFeRhN9SiWU5YUAx27G1Umt4dfGcYQ3mOlf1ys9TOagDg==
  log: .bboxy/log/2026-11-04.jsonl#46157
---
Oracle ruling for WP7 cleanup blocker01M6M74X89171EWVA6A793NWP8. Owners: Nyx Copperwing and Executor01M6KDR90VJTRA1S722FZFECRZ. The earlier laws deliberately state API return is not joined cleanup; now authorize the missing observability needed to retain scratch/claim ownership correctly.

Evidence read: process.rs SpawnResult::drop kills then schedules detached child.wait; spawn_admission's local cancellation/deadline can return before its queued/native spawn finishes or that retirement wait completes. run_inner also has IO/error exits after child admission. Public run/run_stream return no settlement handle. Thus Err, deadline, dropped receiver, closed output pipe, timer expiry and task output alone cannot certify no future mutation or claim release. No existing public joined API was found.

A. ADDITIVE OBSERVED OPERATION
Authorize one small opaque observed-process/settlement API in src/process.rs, reusing the SAME existing admission helper, OwnedChild, group reservation, signals and fenced wait/reap. Prefer a bounded capture-only variant for WP7 scratch Git/setup and the exact lightweight verifier: it needs no streaming sink or spill-file writer, avoiding a generic observation framework. Existing run/run_stream/capture_stdout signatures and ordinary behavior stay compatible; do not rewrite Request/Output or sweep callers.

The operation exposes its ordinary result separately from a read-only settlement observation/handle. Creation/registration occurs before an effect can be admitted, so the caller can retain the observation even if result waiting is cancelled or dropped. The observer has no Child, live PID kill authority, second executor, shared scheduler, pool, global registry or permission-grant meaning. It may use existing runtime tasks/one-shot or per-operation state. Do not use Drop/refcount/channel closure as a positive acknowledgement.

Keep caller result responsiveness and d20261103136567 cancellation/deadline classification unchanged. Use a local child cancellation token/drop guard, never cancel upstream. Caller/result abandonment initiates existing cancellation/retirement without abandoning the real owner. An already admitted spawn still retains that owner even if it completes after result Err. Do not abort a cleanup task merely because its caller stopped waiting.

B. POSITIVE SETTLEMENT CONDITION
A successful settlement acknowledgement requires ALL owned work for this operation to be quiescent:
- admission is closed, no queued/admitted spawn can later create another child or mutate scratch;
- either no child was created and the admission job/result is explicitly finished/refused/dropped-before-start, OR every created owned child went through the existing fenced termination/wait and actual successful reap;
- any abandoned result packet/retirement continuation has explicitly finished the same ownership path;
- the observed operation's own pipe/input/drain/capture work is finished; no detached writer/flush/spill job can remain.
If an observed variant shares an existing error path that would just Drop a child into Tokio's orphan queue, factor only enough private cleanup to keep that owner until its explicit wait/reap acknowledgement. Do not mark settled at scheduling of a wait, registry removal, result delivery, Drop, an expired grace timer or an assumed orphan-queue completion. Optional private factoring is permitted; a second independent process owner/runner is not.

A queued cancellation can establish no-work settlement after explicit removal/refusal of the queued work without requiring service of the four busy permits. If a native spawn/cleanup is already running, settlement may remain pending AFTER the result deadline until that work actually finishes. No universal OS cleanup time/fairness promise is made. A missing/failed/panicked/dropped acknowledgement, wait error or ownership violation is Unproven/unavailable, never automatically confirmed. Never signal a raw/reused/foreign PID to force a proof. Stage result and settlement errors remain distinct.

No new per-job approval/capacity request, retries or global barrier. Independent entries/operations continue. An observer is not a signed receipt, approved producer or proof of malicious/escaped/unowned external processes; ordinary ownership assumptions remain explicit. In particular the supported adapter cannot use arbitrary repo hooks/filters/remote helpers that spawn uncontrolled writers: use private controlled Git configuration/environment, no source hooks/network/credential helpers, and return Inconclusive when such isolation is unavailable.

C. TRIAL RESOURCE LIFETIME
WP7 registers every effectful setup/Git/verifier operation with its existing ONE trial owner BEFORE awaiting the result. The owner retains scratch roots and the candidate claim until all registered settlement acknowledgements are confirmed. Caller cancellation/drop must transfer/retain BOTH observations and those resource leases in its existing local retirement continuation; no async borrowed Store writer is duplicated or reopened.

Do not let a normal RAII claim/File drop unlock or scratch destructor delete while an observed process can still act. If settlement is unavailable, record Inconclusive/Unproven and retain owned pending/quarantined state safely instead of falsely cleaning or freeing the claim. Do not introduce a new global recovery protocol here; any exact unsupported crash/recovery case stays unproven. Process/runtime death does not itself become a current-source cleanup receipt. A local pending entry is labelled/skip-able and must not block unrelated candidates or message delivery.

Scratch teardown acts only on paths this trial created and still owns, after confirmation; guard identity on reuse/cleanup. Keep retained closed arm IR/blobs/proof custody available after ephemeral worktree/HOME cleanup, so usage/event pins still resolve. Do not delete retained session evidence with the scratch tree. This seam does not authorize a second durable store or changing Store/fork publication.

D. MODELS BEFORE STATEFUL CODE / WITNESSES
Keep all pre-WP7 147 model semantics and existing ownership/lease/outcome contracts unchanged. Add a bounded settlement/resource-retention family to WP7's new models: queued/admitted/native-spawn/result/retiring/reaped/no-child, result-returned, caller-dropped, observer-confirmed/unavailable, claim/scratch ownership and zero/one late writer. Separate result and settlement; model queued refusal without permit fairness, spawn completion/retirement under stated fairness, delayed observation and receiver loss. Success cannot be reached merely by missing acknowledgement. Reachable controls: release on Err, confirm on channel/observer Drop, and release/cleanup before a delayed spawn/wait. Tie the same properties to actual API/process tests; no bounded graph proves OS timing or arbitrary Rust correctness.

Private deterministic witnesses must cover: never-polled/no-child; all four permits held with queued cancel/deadline; drop before/during admission and during an unreceived result; an actually late admitted child behind a native barrier; successful live operation; an IO/error exit after admission; observer receiver loss/panic/unavailable classification; stale/reused identity no signal; and TWO independent operations so one pending cleanup is not a machine-wide barrier. Verify ordinary API result may return while settlement/resources are still pending, then release the private barrier, wait for exact owned PID/group reap, and only THEN prove scratch/claim can be released. Actual external private sentinel must never be written after confirmed cleanup. Cleanup fixtures release their own barriers and join exact children even on failure, with no operator input or broad process kill.

The adapter must use the joined observation, not pid polling or an extra executor wrapping an unobserved public run. Tests also prove caller/upstream cancellation isolation and that opaque proof from operationA cannot release operationB. Default/unobserved process tests, the exact deadline race regression and Store lease probe remain green.

E. NARROW SCOPE / CHECKS
Add src/process.rs ONLY the additive observed/capture settlement API, optional observation plumbing/private owner cleanup factoring and its private tests. Add tests/cancel_process.rs ONLY public settlement/cancellation/retention witnesses, keeping every old assertion. Existing trial/headless/engine/tools paths cover its consumption; formal corpus/contracts/coverage/nonvacuity paths cover the new family. No process policy/executable/env classification rewrite, unrelated IO redesign, general executor, dependency, Store/IR/schema/receipt change. If a required external seam remains, provide exact lines before touching it.

Exact verify becomes: cargo test --lib tools:: && cargo test --lib process:: && cargo test --lib dream:: && cargo test --test dream_pipeline trial && cargo test --test policy_gate --test cancel_process --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence.
Refresh full source/model/tool native/Java evidence on the final report HEAD, plus ordinary/default/ACP/all-feature clippy/IR and unattended native/check-transcript smoke as already required. Fresh signed report/task_report required for this changed verify. Existing thresholds/timeouts/spend remain unchanged. No canonical KB5974 waiver, live producer approval, main merge or publication.