---
type: decision
title: Trial-only exact queued frontier forks recover before fresh continuation
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[protocol]]'
  rel: extends
  weight: 3
- to: '[[d20261103136597-dream-trial-derived-usage-proof]]'
  rel: decision
  weight: 3
id: 01M6MA2TRBKW5RTYSFVVD1FMDD
updated: 2026-11-04T22:15:13.624Z
created: 2026-11-04T22:15:13.624Z
number: 20261103136603
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:24cf29933ffe77a817fdd95889398c7cc681508e7c722c037e6b1d16ec576c00
  frontmatter_digest: sha256:b681f224bd4d15bc0d2e5ce406279075162f1802b754799c622cfa974b5bb540
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: CiznzvxIQ5G8/sYUhVVhj/wkRcy42lTCGR988ei1nvI5QUIFkfzrCYpKcenRjcdVixp2vgYNicc7IPljfvt7AQ==
  log: .bboxy/log/2026-11-04.jsonl#46183
---
Oracle ruling on WP7 blocker01M6M9YCGNKFNC2XRCTPRJ8RS5. Owners of this internal trial/Store boundary: Nyx Copperwing and Executor01M6KDR90VJTRA1S722FZFECRZ. This adds a trusted internal operation, not a change to the public StartSession fork contract.

Evidence: src/store/fork.rs materialize calls the complete-transcript checker on the parent prefix; src/protocol/laws.rs check_transcript rejects queued!=ended and any open item. interfaces/protocol §4c requires a quiescent public fork. Store::recover already appends Interrupted endings for unfinished queues, then SessionRecovered, but open_impl deliberately skips recovery for a newly materialized fork. The actual real-arm diagnostic dream-wp7-real-positive-diagnostic.log shows invalid fork point or parent and a failing positive assertion. Author source remains uncommitted; this is boundary authorization, not acceptance.

CHOICE
Keep dream-log §4.3's exact last durable seq BEFORE the selected TurnStarted. Do not silently move the cut earlier, omit a queued envelope, rewrite retained bytes, pretend the old fork API succeeded or relax check_transcript. Add an explicit internal opt-in path only for the already validated opaque trial LaunchOptions capability. Ordinary Store::open, Host/public StartSession, default/Yolo and ordinary quiescent forks retain their current contracts and refusal tests. A profile name, meta.harness, serialized flag or caller-provided boolean alone cannot enable this path.

A. ACCEPTED FRONTIER
The opt-in may materialize a valid quiescent prefix as before, or a prefix whose ONLY unfinished work is one or more TurnQueued identities with NO TurnStarted for any of those unfinished identities. Every previously started turn has ended and every item has completed. No open/running turn, open item, closed session, gap/duplicate, orphan/duplicate ending, malformed envelope, unreadable version, invalid durable head or missing/corrupt referenced blob is allowed. All existing parent/cut bounds and prefix identity checks remain.

Use the existing complete checker as the authority for the rest of the prefix. A small private frontier classifier plus a validation-only clone completed with canonical Interrupted TurnEnded events is permitted: run unmodified check_transcript on that completed validation view. Such synthetic view bytes are NEVER copied into the inherited range, used as source evidence or published as parent history. Explicitly require the unfinished queue identities to have never started; completing an active turn in the validation view is not permission to fork it. Do not add a second transcript-law implementation or depend only on matching an error string.

The selected source episode/TurnStarted and cut remain pinned in the trial's existing selection. Pending unrelated queued admissions are inherited history only, all retired by recovery; none may be replayed as commands or counted as the new arm. Source logs/parents are read-only throughout.

B. EXACT CUSTODY AND ORDINARY RECOVERY
Materialize seq2..=cut with the existing raw-line pass-through, unknown-field/variant tolerance, referenced blob verification, fsync and no-replace publication. Header seq1 alone is the authored child header with its original parent/cut link. Preserve every inherited line byte, seq, timestamp, cause and blob; parsed local session/epoch projection remains the existing Envelope::project_into rule. Parent append/delete independence remains.

For the new queued-only child, before returning an operational Store or exposing a gateway/provider request, reserve/persist the normal next child epoch and run the existing Store::recover on that ONE child writer. A fresh fork's inherited projection is epoch1; its recovery suffix starts at cut+1 in durable epoch2. Do not reuse/reset epochs or mutate the parent epoch. Reuse current recovery ordering and actual Interrupted TurnEnded/SessionRecovered events, with correct closed_items=0 and exact pending-queue closed_turns. No invented Cancelled/EndTurn success, usage, model output or executed inherited UserTurn. Quiescent fork open behavior/head/epoch remains unchanged.

Opening/recovering errors do not return a usable actor. A crash after complete prefix publication but before/during the suffix leaves an ordinary unclosed child that ordinary Store recovery can finish; it must not execute an inherited queue. Preserve the writer-lease guard through errors/cancellation/opening work, durable epoch reservation and normal recovery, with no second writer, reopen-by-hand, bypass lease or new persistence protocol. Later recovery may append a new SessionRecovered but cannot end the same queue twice. Keep prefix atomicity distinct from suffix completion; do not claim a published prefix is already a complete operational transcript.

C. FRESH CONTINUATION / USAGE
Only after the recovery suffix succeeds may the ordinary engine append profile/notice and admit a NEW continuation UserTurn with a fresh TurnId. Send the exact selected original UserMessage blocks as the new input, with the existing bounded quoted memory addition only in the with arm. The inherited interrupted queues have no new provider/tool call and contribute no trial success or usage. The first TurnEnded after cut can now be an inherited-queue recovery ending: never pick it as the arm. Bind the arm to its unique new post-fork TurnStarted and matching TurnEnded, and its own call usage proof, using d20261103136597. Both producer and shared validation use that identity, not ordinal guessing, timestamps or a first-ending heuristic. No aggregate source relabelling or schema change.

The validated trial capability selects the narrow Store operation from existing engine wiring. Avoid adding public Config/SessionMeta fields, changing wire StartSession or sweeping callers. The internal method may be pub(crate); integration tests may exercise it through the existing capability/engine path while unit tests cover its classifier. A new public unauthenticated allow-incomplete flag is outside scope.

D. BEFORE-CODE MODEL AND REAL WITNESSES
Add a small bounded queued-frontier/recovery/continuation family and reachable broken controls to the existing WP7 formal paths. State: valid/malformed/queued-only/started/item-open frontier; prefix published and byte-preserved; lease/epoch reservation; zero/one/two pending queues; crash/reopen; interrupted suffix count; actor exposed; fresh continuation and usage binding. Separate classification, prefix publish, reserve epoch, recover each queue, recovery complete, actor exposure and fresh run. Safety: only the accepted frontier; exact inherited bytes; no actor before recovery; each accepted queue ends once; no inherited queue execution; fresh arm cannot use inherited ending/usage; parent unchanged. Progress only under explicit local persistence/recovery fairness, not repeated crashes. Broken controls must reach rejection for accepting started/open work, exposing before recovery, rewriting/skipping the queue, and treating inherited recovery ending as fresh success. Keep all pre-WP7147 model state/edge/metadata semantics and existing Store/lease/fork model semantics intact; new contract owns this extension.

Real tests: preserve existing ordinary invalid_fork_points refusal and quiescent epoch/head/request/blob assertions; actual queued-only child through trial engine succeeds with exact raw prefix including future-data lines and blobs; parent hash unchanged and child survives parent deletion; one and multiple queues interrupted exactly once before a fresh provider request; running-turn/open-item/bad-seq/duplicate/closed/missing-blob controls refuse before a usable child; unpolled/error/cancel/reopen at prefix/epoch/partial-recovery boundaries retains lease/epoch and never executes queue; new provider turn/call usage pin matches fresh identity, not recovery suffix. Fixed-cut parent appends do not widen the copied prefix. Use deterministic private fixture/barrier controls, no operator or retry-until-green. The actual enforced twenty-arm trial must now run; the old diagnostic remains retained as red evidence.

E. SCOPE AND VERIFY
Add src/store/fork.rs only the private opt-in classifier/materialization validation; src/store.rs only the internal opt-in open and ordinary recovery/epoch wiring plus private tests. Existing allowed engine/headless/trial/projection paths may wire and bind this operation under the validated capability. Add tests/store_fork.rs only queued-frontier/recovery/API/byte-custody witnesses, preserving existing public assertions. No protocol/laws.rs or Request/Config/SessionMeta/IR/schema/Store lease changes, ordinary fork weakening, generic incomplete fork, second store or dependency edit.

Exact verify becomes:
cargo test --lib tools:: && cargo test --lib process:: && cargo test --lib dream:: && cargo test --test dream_pipeline trial && cargo test --test policy_gate --test cancel_process --test store_fork --test store_recovery --test protocol_laws --test ir_v1_corpus --test dream_graph --test dream_audit_broken --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence

Fresh signed report/task_report with source/report-HEAD evidence is required; retain full default/ACP/clippy/IR/unattended native/check-transcript checks, exact20 real-arm positive and relevant negative controls. Canonical KB5974, no live policy/bootstrap/model spend, no main merge/push/publication and final term/shutdown order remain unchanged.