---
type: decision
title: Derived trial usage binds retained continuation call facts without relabelling
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[d20261103136594-dream-trial-engine-capability]]'
  rel: decision
  weight: 3
id: 01M6M6Y8A1M9W36GP9N41J8BVH
updated: 2026-11-04T21:20:17.879Z
created: 2026-11-04T21:20:17.879Z
number: 20261103136597
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:27825c9c98a6c6e8e94715631d5dee43429d371ac33a196064a45a30da5714f9
  frontmatter_digest: sha256:9df562fda975243db92253036d309c8f7f9c649b86e8d69ffd5bcd4edaae12f4
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: lrhLfg6yUlgb24rvxYpOVu+ybBgmtNHBwnxi1d3EtVzddPWSch0iiTqsIsQ5XXDma4cUMxuz3wfqV6CO7pimAw==
  log: .bboxy/log/2026-11-04.jsonl#46103
---
Oracle ruling for WP7 usage blocker01M6M62EJGE8A9JG3YJ2J5GATV. Owners: Nyx Copperwing and Executor01M6KDR90VJTRA1S722FZFECRZ. Preserve Derived accounting; do not relabel it or loosen qualification to a source-tag assertion.

Source facts read: engine/turn_loop.rs:277–292 uses Usage::sum_calls for TurnEnded. protocol/usage.rs:84–155 marks its sum scopeTurn/sourceDerived, preserves missing required counters as unreported and aggregates optional counters/cost. projection.rs:348–352 accepts only Reported/Estimated in TrialRun; qualifying_trial is context-free and Projection::build calls it for adoption. Audit currently checks only closed run-session presence. TrialRun has session/arm/verify_exit/usage, no turn locator. SessionMeta.fork is (parent_session,cut_seq). A sum over an entire forked log would double-count inherited calls and cannot be used.

A. RETAINED ACCOUNTING / TARGET
New ordinary-engine TrialRun.usage retains the exact Derived TurnEnded aggregate. A single reported call is still aggregated by sum_calls; no fake source conversion even for one call. The producer keeps the actual closed ordinary arm session and fork envelope/blob custody.

Resolve a Derived run against ONLY its own retained session, within the selected audit/receipt causal cut. Validate session/header/fork identity/order/closed custody using existing transcript/pin laws. Determine the unique newly started continuation turn AFTER the declared fork cut; do not infer it from timestamps, global seq order, newest unrelated turn, cached totals or a task label. The target TurnStarted/TurnEnded must unambiguously bind the same session and turn. No unique target/closed prefix/fork proof => unavailable/Inconclusive, not a guessed pass. No new TrialRun wire field or ext convention is authorized.

Collect actual completed UsageScope::Call items belonging to that target turn after its start and before its end, joining ItemStarted/ItemCompleted by local session AND item/turn identity. Exclude all inherited prefix calls, other sessions/turns, Turn/Session aggregate rows and later work. Reject duplicate/contradictory known call facts. Preserve exact retained envelope digests/prefix/fork/turn binding in the in-memory proof. Missing/unreported/cancelled-without-complete-counters data is not zero. Retained-IR proof does not prove unlogged external calls; producer completeness remains the existing approved-source/receipt assumption, never inferred from a number tag.

Recompute with the ONE protocol Usage::sum_calls over those leaves; require a nonempty valid leaf set and exact equality to BOTH the target TurnEnded.usage and TrialRun.usage, including scopeDerived/counters/unreported/backend/model/optional cost fields. Do not duplicate aggregation arithmetic, ignore a mismatched optional field or use a prior turn's sum. A known mismatch is P5 FAIL; missing/truncated/unavailable proof is P5 UNPROVEN and producer Inconclusive. No successful producer/adoption/strength qualification until the required proof is available.

B. EFFECTIVE COMPARABILITY
Each contributing call must be scopeCall with complete required input/output counters, known sourceReported OR sourceEstimated, and known matching backend/model identity. Check consistency explicitly; sum_calls choosing a first backend does not prove all leaves match. Call-level Derived/Unknown/missing source or heterogeneous Reported/Estimated basis is not a homogeneous basis. Across all compared trial arms/runs, retain the same underlying Reported or consistently Estimated basis and comparable backend/model; two Derived tags alone never establish this. Do not change the serialized Derived aggregate to that transient effective basis. Valid explicit reported zero is distinct from unreported counters. Missing/overflow/incomparable input remains unavailable/Inconclusive; do not substitute free work.

Token spend/median/gain/control/N/R/thresholds@2 arithmetic remains byte-preserved in meaning, including existing input+output token accounting and checked overflow. Optional monetary data remains optional/labelled; do not turn missing money into zero or change currency semantics. This extension supplies provenance/basis, not new thresholds or independent price arithmetic.

C. ONE SHARED CHECK / NO BYPASS
Introduce one opaque in-memory checked run-usage proof/context minted by the common projection/retained-envelope validation path, never by deserialized flags or a caller's asserted basis. It binds run session, target turn/fork/cut, exact aggregate and underlying basis. Bind its use to the current run/session/usage so cloning it then mutating a Fact/TrialRun cannot authorize different data. It is not serialized into IR, API Fact JSON or KB as new authority.

Thread that context through the common qualifier/projection and audit/producer calls. A context-free qualifying_trial must continue to refuse unproved Derived; do not just accept sourceDerived in the existing match. Existing non-Derived Reported/Estimated behavior and fixtures stay compatible; this does not retroactively make those legacy labels cryptographic measurements. A new producer's ordinary aggregate must use the Derived proof path, not choose the legacy path to evade it.

An internal proof carried with facts (skip serialization) or a narrowly extended checked API is acceptable. If a private proof field changes Fact construction, provide a safe no-proof constructor for old synthetic Reported tests rather than a public forged-proof flag. Common audit/graph/receipt/producer consumers cannot qualify Derived adoption, comparable cost or positive usage-dependent strength without this same check. Preserve the documented negative/failed-trial observation behavior; do not erase evidence merely because it failed a positive gate.

Use only the envelopes actually supplied by that consumer's retained context/cut. Missing child arm logs must remain unavailable; never read later global/current logs to retroactively approve a past receipt, widen a cut silently, copy a cache verdict or manufacture an event pin. Receipt availability classification may adapt only for this proof; all R1–R4/E, seal, per-Adoption binding and trust rules remain intact. Audit must distinguish missing proof from known contradictions while keeping both nonzero, not report an unavailable Derived run as positive.

D. TESTS / MODELS
Use actual private ordinary engine with a tool round so multiple reported CALL items produce a retained Derived TurnEnded and TrialRun total. Both producer and read-only API/CLI audit/qualifier must consume the same proof. Positive fixtures need complete source/cut context; standalone stage receipts remain UNPROVEN under prior law.

Controls: no call rows; wrong/missing turn; inherited prefix or another-session calls; duplicate item; omitted/missing required counters; Reported/Estimated mixture; differing backend/model; sum overflow; aggregate/TrialRun field tamper; unknown/Derived leaf source; open/truncated/missing session; copied proof with changed session/aggregate; child logs outside selected cut. Known mismatch =>FAIL, absent proof=>UNPROVEN; neither can create Pass/Adopted or a positive qualification. Preserve all old good/broken fixtures and current numeric edge tests.

Add a small finite usage-provenance/basis join model to WP7's new families if the new proof is not represented, with explicit old-prefix/target/foreign session or turn and homogeneous/missing/mixed basis states, plus reachable blind-Derived/wrong-join controls and real counterparts. Retain existing147 model semantics and source/metadata-byte proof except explicitly new families. Finite sums/identity predicates do not prove real bytes, call completeness or kernel custody. No new concurrency, store, global registry or protocol field.

E. NARROW PATHS
Add src/dream/projection.rs for this shared proof/qualification; src/dream/audit.rs only matching P5 proof consumption/classification; tests/dream_graph.rs only proof regressions and any no-proof Fact constructor adaptation; tests/dream_audit_broken.rs only actual Derived positive/negative API/CLI controls. If needed, src/dream/graph.rs, src/dream/receipt.rs and src/dream/prompt.rs may ONLY mechanically thread the common proof/checked qualifier/gain/availability call, preserving their other source logic. Existing trial/pipeline/support/fixtures/formal paths cover production and witnesses. No protocol/usage.rs/turn_loop/IR/schema, thresholds, cache/resolve, R1–R4/E/trust/seal, generic engine accounting or old source-log edits. Exact verification must add the audit/graph test suites to current tools/trial/policy/formal chain. Fresh signed report/task_report required; no canonical KB5974 waiver/main merge.