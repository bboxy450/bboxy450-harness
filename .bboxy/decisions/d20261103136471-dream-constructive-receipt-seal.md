---
type: decision
title: Constructive receipt sealing and noncircular producer approval
status: active
owners:
- Nyx Copperwing — producer integration and acceptance
- Dream WP4 Executor — shared receipt consumer
links:
- to: '[[d20261103136362-dream-audit-trust-root-and-receipts]]'
  rel: supersedes
  weight: 3
- to: '[[d20261103136378-dream-receipt-wire-kind]]'
  rel: implements
  weight: 3
id: 01M6KGTF6AYWC6NN1H3CY9FKMM
updated: 2026-11-04T14:53:45.547Z
created: 2026-11-04T14:53:45.547Z
number: 20261103136471
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:d71ac94ea397f3d071c26275bafb957977fccb0045d0d04e02e2ffceef9e7a8e
  frontmatter_digest: sha256:b77959ca77cafd6137e5677396ac02598e043a45266498a2c2e9983cb5e8322e
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: eHMDExSfpnnC6H7Y8OVbDqQc8qatGR0Dd9Kj0Vi6Iqa3YfRXZsnsB7WosqpHCCYdBULo+pB7Z4nTYeu6FAddDA==
  log: .bboxy/log/2026-11-04.jsonl#44822
---
Oracle ruling, 2026-11-04. This resolves the remaining self-recording and gate/build circularities identified in the rejected audit-law task01M6JRSZQTWPHMG085V047H62F. The Architect's reviewed revision2 supplies V/T, producer artifact identity, deterministic E, causal cut, selection and R1–R3. This decision supersedes only its coverage/sealing and gate-report ancestry lines. It grants no implementation paths; signed task amendments do. Original Architect task is not retrospectively accepted. Current Claude weekly quota is exhausted; the operator's standing order requires the Oracle to rule and work to continue.

## Decision and proof boundary

A receipt proves bounded observations by an approved truthful producer. It cannot encode its own final hash or prove arbitrary filesystem history. P2 still requires all producer effects on every sink/generated target and P4 still requires all producer effects on cited source/session custody to be accounted for, including owned tools/children. External writers/native loader state remain the existing explicit exclusions. Merely equal before/after bytes cannot excuse producer ABA or an unobserved owned child.

Replace coverage with exactly observer:"approved-producer-self", covers:["producer_protected_effects"], excludes:["external_writers","native_loader_state"], interval:"start_through_freeze". Protected effects comprise all reads/writes needed by E/P2/P4 on selected sink/generated paths, the selected sessions root (including cited logs/blobs), own receipt/log custody, and the configured D0 cache. Trial worktree changes outside these domains are not claimed as source/sink mutations, but an owned child's ability to touch a protected domain must be observed/constrained; naming its private cwd is not confinement. Protected roots and allowed cache locations are resolved by the consumer from its actual input/configuration and cut, never granted by receipt text. No runtime proof of a missing observer is invented.

## Constructive sequence and exact seal

Keep typed Item::Receipt, receipt_kind, Adoption.receipt and receipt body schema bboxy.dream.receipt/1. Add a required body object seal with exactly: version:1; receipt_item:<containing ItemId>; own_prefix:{session,through_seq,prefix_digest}; effect_count:<u64>; children:[{id:<unique ULID>,state:"joined"|"killed_reaped"|"active"|"unknown",coverage:"observed"|"enforced_read_only"|"unobserved"}]. Children are sorted by id and unique. Empty children means the approved producer actually started no children, not that it omitted them. Counts and IDs are declarations checked structurally; their completeness/quiescence/enforcement is a W obligation of the approved executable and independent process witness, never inferred from these fields alone. In particular joined alone does not prove the child's earlier effects were observed. Every non-read-only child requires complete relevant effect coverage during the interval. Missing coverage or any active/unknown child makes the receipt UNPROVEN owned_child_unobserved or owned_child_active.

1. Reserve/start the receipt item in own dream session S. Complete the effect work; record each completed result in dense ops. A publication's effect interval covers its entire publication invocation, including hand-write conflict handling. The outer run recorder also observes those effects and publication-receipt custody operations.
2. Revoke further effect admission for this interval, join all its owned work or terminate and reap its exact owned process groups, finish in-flight calls, and freeze ops. The shared per-run owner token must make late protected mutation impossible through the approved wrapper. A child outside observable/enforced ownership prevents proof even if its process disappeared. This is per-run/per-publication ownership, never a machine-wide lock or approval slot.
3. own_prefix is the exact retained own-log prefix through that receipt's ItemStarted, including its LF and excluding its ItemCompleted. No other appends may interleave in this receipt's sealing suffix. prefix_digest has the same sha256:hex convention as other receipt prefix digests. effect_count equals ops length. Freeze the canonical body, compute its digest, then seal it. The body contains no digest of itself, its own blob write or the suffix that refers to it.
4. Seal into S's existing blob store with owned-temp/write/fsync/no-replace/directory-fsync custody; retain the typed ItemCompleted referencing the actual BlobRef. Never overwrite an existing blob or log prefix. A publication's own sealing writes are recorded by the outer run, whose body is not frozen yet. There is no second store or new IR envelope field.
5. After a publication freeze, only its own receipt-blob custody and contiguous typed Receipt completion plus matching Adoption ItemStarted/Completed pairs are permitted until that publication invocation returns. Adoptions remain after the Receipt and match §13.5 P2(d); skipped publication emits none. The outer run observes this entire suffix and may then start another publication.
6. After final run freeze, only that receipt's own blob custody, its exact typed Receipt completion, at most one required TurnEnded, and SessionEnded are permitted. No new semantic effect, source write, publication, child spawn, cache write or detached close hook is hidden in this exception. The dream producer must not trigger its own detached D0 hook; ordinary non-dream session hooks retain their nonblocking behavior. An inherited helper unable to honor this boundary yields UNPROVEN, not a wider exception. No child/future with protected write authority may outlive the final freeze.

The final run's narrowly defined own sealing writes are expressly outside its frozen ops. Their safe implementation is W, tested against the approved executable; scripts separately check actual retained blob/header/prefix/suffix bindings. Their present existence is not claimed to prove past fsync/completeness. Publication sealing is additionally covered by its enclosing run. This terminates the proof construction without requiring a hash fixed point or infinitely nested receipts.

## R4 and failure disposition

R1–R3 remain mandatory. R4 requires the exact known coverage and seal version, matching containing session/item, exact frozen own prefix bytes, effect_count, complete dense effects, unique known child rows, and the permitted typed suffix above. Unknown/missing seal/coverage, missing or truncated custody, absent outer run for a publication, missing approved seal capability or unobserved child coverage is UNPROVEN exit1 with a named reason. A known prefix/header/item mismatch, effect_count discrepancy, forbidden suffix or recorded late protected mutation is FAIL exit1. A proven producer source mutation is always P4 FAIL writer_touched_source, including ABA and a write disguised as own custody. Do not certify source nonmutation solely from prefix equality or child rows. P2 publication history, selection at its cut, terminal fencing and current Recall currency retain their original full checks. Old carrier-only fixture bodies remain reader-compatible but fail semantic R4; every old frozen corpus byte stays unchanged.

Both receipt kinds now require capability constructive-seal/1 in addition to their existing sets. Required gate witness ids for it: dream_trust::receipt_seal_constructs_without_self_hash; dream_sinks::late_owned_source_write_after_seal_rejected; dream_sinks::broken_seal_exception_fails_audit. WP4 supplies consumer/builder and real private-process owned-child witnesses now; WP8 supplies the sink observer/executable gate witnesses before live producer approval. Until then producer_unapproved/UNPROVEN is the honest live result. A delayed child attempting a protected source write must be rejected, or explicitly make proof unproven; the deliberately broken exception must expose the actual forbidden write and audit FAIL. Arbitrary children cannot be called external writers to evade this control.

## Formal and API proof

Add finite HarnessDreamSeal plus BrokenHarnessDreamSealLateWrite and BrokenHarnessDreamSealChild. State includes one producer, zero/one owned child, phase recording/frozen/sealed/closed, one in-flight effect, source-mutated flag, and custody-only versus protected operation. Freeze requires no in-flight effect and owned child joined/reaped with observed/enforced coverage. Closed/sealed never admits a protected write; good model has no mutable child after freeze. Controls deliberately allow a late protected write or freeze with a live child; each must reach its named safety violation. Enumerate the finite bounds and no fairness/liveness claim for arbitrary IO; run full native/Java reachable states, labeled transitions and verdict parity. The model proves bounded ownership/order only; canonical JSON, hashes, signatures, syscall durability and actual child capabilities need implementation/process witnesses. WP4 adds these model/control/API mappings under a scoped amendment; WP8/WP9 later map the real producers. No nonexistent producer path may be listed as current evidence.

## Noncircular artifact approval

Replace §13.2b's impossible requirement that the accepted gate task's report.commit predate the producer source commit. Sequence is: signed source S; build executable B from S; measure B; run required unattended witnesses against that exact B; commit signed gate log G binding S/tree/version/build/B and witness results; commit the full gate report R and accept/commit the gate task; then Oracle installs reusable policy entry linking B/S/G and that accepted task. S must be an ancestor of R, not the reverse. The report's commits must explicitly include S and G; all existing T/G/blob/root checks apply. Report/gate/task recording may follow S without rebuilding B; those records do not claim to alter the tested executable. An actual implementation/build-input change requires a new artifact digest and gate. Tests may bootstrap only isolated fixture policy and keys; none needs live producer approval. No product run approves itself and no per-job approval/queue is introduced.

Producer source/build identity is independent of a recalled session's repository/cwd. The gate's measured B and approved artifact map bind it to S; runtime measures current_exe at start and matches that artifact, rather than running git in an arbitrary session cwd. No matching policy/artifact means UNPROVEN producer_unapproved and auto-adoption disabled. Local hostile editors of policy/executable/logs remain the already explicit excluded threat model, not a reason to misclassify ordinary owned-child effects.
