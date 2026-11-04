---
type: decision
title: 'Dreaming: superseding no-clobber sink, per-harness lifecycle and percent-unit baseline'
status: accepted
owners:
- Lumen Foxglove (Architect)
- Nyx Copperwing (Oracle)
links:
- to: '[[dream-log]]'
  rel: baselines
  weight: 3
- to: '[[dream-sinks]]'
  rel: supersedes
  weight: 3
- to: '[[memory-graph]]'
  rel: supersedes
  weight: 3
- to: '[[2026-11-03-dream-architect-refinement]]'
  rel: for
id: 01M6J8T8NS8AZGD811VWH4NX8N
updated: 2026-11-04T03:14:35.587Z
created: 2026-11-04T03:14:35.587Z
number: 20261103136206
created_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
updated_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
authorship:
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  kind: seat
  public_key: 6HRMC38XoZOA5TUGuPnerzKIZabBnbbBZSCqIQt225c=
  process_generation: '1'
provenance:
  digest: sha256:622c546b7ad4937d518fe4876124a93da665126020d92d34d86d1a0dfcf8e9dd
  frontmatter_digest: sha256:e16613c2dd1e03b9e8f909fabe01c3d48f55a941c41e1f9401da763ddda556ad
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  sig: f9z6ZEfHo0SGi7cHwjmlI6pMk1c8kGmw8rCM0xASRLiOC4rW3qtAnIXd8QxCyUs8JvFLHYnMAHYO2moxq2dPCA==
  log: .bboxy/log/2026-11-04.jsonl#42204
---
# Dream contract refinement: no-clobber sinks and per-harness provenance

**Date:** 2026-11-03. **Status:** accepted baseline for this workstream.
**Both owners:** Lumen Foxglove, Architect (interfaces and verification); Nyx Copperwing, Oracle (harness integration and task amendments). Nyx's approval and request for this superseding record are message `01M6J83FJ0W2EKK582AYSG6HA3`, following no-clobber approval `01M6J7SJKKNWKK66SAGA9N3DTE`.

This signed addendum is readable independently of the three conflicting-number records. It changes their contract deliberately and preserves their bytes. It does not renumber them, bypass a refused update, grant an Executor a new file path or claim implementation verification.

## Superseded lines and precedence

- [[dream-sinks]] v0.1 §1 atomic-write/ownership/recording rules; §2 Claude index/removal; §§3–4 replacement/cleanup assumptions; §5 engine/ACP context role and fixed mutable artifact path; §6 T3 fraction claim.
- [[memory-graph]] §4 global max lifecycle and repair-by-rewrite; §5 active-set lowering insofar as it assumes every native sink enforces immediate forgetting; §6.2 system-prompt placement; §3 recall-strength accounting for inferred exposure.
- [[dream-log]] pre-refinement §2 pins, §4.3 whole-machine trial count, §7 P2/P5/P9 and §8 lifecycle/merge/sink models.

The resulting normative text is [[dream-log]] at signed commit `b4894fb6dd4e9731256760683ae2548a03f72bad`, especially §§2,5,7–10,12. The companion-file mirror edits remain pending host/index repair. This decision and that version of dream-log govern until the mirrors are applied. d5963 automatic low-trust adoption and d5974 KB records/cross-repo opt-in remain in force.

## 1. Sink publication: no lost hand edits

**Rejected alternatives:** digest recheck followed by overwrite-rename permits a hand edit between the check and rename; an advisory lock among dreamers does not cover arbitrary editors; one global writer/approval gate would serialize unrelated work without fixing external writers.

**Chosen:** complete private temporary bytes, fsync, then an atomic **no-replace** publication on the target filesystem. An exclusive open followed by streaming bytes is not atomic publication. Existing identical pinned bytes are idempotent; a differing target, changed ownership, ambiguous markers, symlink, prefix collision or absent safe primitive is a labelled skip. Only the publisher's own temporary name may be removed. No filename/marker alone permits overwriting, unlinking or chmod of an existing user-editable target.

An actual cooperative compare-and-replace is allowed only if every writer participates and a real boundary test proves it; no such native contract is assumed. Hand edits are legitimate. Preserve them and stop managing that path rather than repairing it by overwrite.

- Claude: publish topic files first; an absent MEMORY.md may be published no-replace and counted only if it links the selected topics within existing limits. A pre-existing shared index stays untouched without cooperative CAS. Unlinked topics are not Adoption.
- Codex/Grok: retain useful exclusive resource/inbox publication, once per full entry revision, without clobber. Verify the full EntryId/digest behind a short filename, known static Codex adapter instructions, and native schema. Changed files are skipped. Never rewrite vendor-generated summaries/topics or instruction files.
- Vendor consolidation may retain historical copies after retirement. Label retained native exposure stale/unknown; a new retirement notice is not proof of forgetting. Do not blindly unlink a possibly edited file. P2 certifies publication-at-time and host effects, not an external vendor's current active set.
- Engine/ACP: use versioned immutable context artifacts and a validated per-repo projection. Serve only current resolving pairs. Missing/stale roots, projections or digests omit memories; they never delay a live request for a global scan.

Lifecycle adoption after a passing trial remains automatic. Successful materialization is separate: one Adoption per selected entry, with its committed RecordPin and common block digest, only after a loader-reachable publication. A skipped sink, dry-run or unlinked artifact emits no Adoption. Selection is the ranked budget-fitting subset, not all adopted entries. Crash before the record is reconciled by verifying the existing exact publication, not by overwriting it.

P2 compares generated bytes/pins and proves the dreamer's no-clobber effects. It never compares unmarked hand content with yesterday's snapshot as a correctness condition.

## 2. Lifecycle: event-set union, per harness

Deduplicate durable envelopes by `(session,seq)` and exact digest; merge is set union. Validate transitions and pins before projection.

- Proposed is entry-level (`harness: None`).
- Tested, Adopted, Rejected and Retired require a harness and affect only `(EntryId,harness)`. Adopted pins a qualifying Trial(Pass) for that same pair and thresholds revision. Inconclusive stays Tested.
- Superseded and Expired are entry-global terminal states. They fence delayed adoptions for every harness.
- Valid pair phases advance proposed → tested → adopted → terminal. Terminal reasons are retained; fixed display precedence is superseded > expired > retired > rejected. Neither clock order nor arrival order reopens an entry. One harness's rejection cannot erase another harness's adoption.
- A terminal revision can be proposed again only with a new EntryId: revival plus the terminal predecessor in derived_from. A → B → A never reuses an old pass.
- File harnesses is the sorted active adopted set. Scalar status is a summary (global terminal first, then any adopted, then any tested, then terminal display, otherwise proposed); it never authorizes a sink.

Handwritten memories and edits remain valid proposals with signed origin/evidence. Changed identity content gets a new EntryId and must earn its own pass. Preserve the old signed revision; do not overwrite edited bodies as projection repair. Stale derived metadata and changed content are distinct P9 outcomes.

## 3. Trust, committed pins and Recall

RecordPin adds optional commit; Adoption adds optional record; Recall adds optional record and block_digest. All newly added wire fields are optional/defaulted (including inside new variants); authored checks enforce semantics. Flush confirms knowledge_write and knowledge_commit before emitting a committed Created/Revised pin. Unknown mutation outcomes are queried by original operation ID; uncommitted drafts remain Pending.

Engine memory is a separate quoted **user/reference** message before the actual request, never text appended to system/developer/append-system-prompt. ACP memory is quoted data after handoff and before the request, never a system/config update. Escape wrapper tags and deny-scan. The fixed instructions stay authoritative.

A reported Recall proves exact insertion, cited proves a resource citation, inferred means possible exposure. None establishes model use or instruction authority. Pin exact revision/block bytes; no resolving publication timeline means no invented inferred Recall. Every basis excludes the exposed session from evidence/held-out trials; only reported/cited accepted observations increase recall strength. Log insertion before dispatch; if it cannot be recorded, omit the memory.

## 4. T3 is percent; scheduling is explicitly revisioned

Pinned term source `b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8`:
- `crates/bboxy450-usage/src/lib.rs:121-125,129-132,145-152` — u8 window percent, documented percent, direct rounding.
- `crates/bboxy450-term/src/usage_strip.rs:82-92` — direct JSON serialization; `:75-80` stale retention; `:98` snapshot-update Unix seconds; `:136-148` existing 96-percent source witness.

Thus validate integer used in 0..100 and compute used_fraction=used/100.0 once before the unchanged 0.80/0.90 gates. Projected is percent pace and may exceed 100; resets_at/sample time use Unix seconds. Null/stale/invalid inputs disable the quota trigger, not deterministic work. A global snapshot update does not prove a fresh provider observation. Exact source and threshold-table assertions exited 0; the producer test itself was not run here.

`dream-thresholds@2` records removal of the former whole-machine single-trial cap. Every numeric score/spend/timeout table row stays byte-identical to revision 1; the automated comparison exited 0. Candidate claims remain nonblocking; independent shards and live sessions keep moving. Heavy-test children acquire the host's existing self-service resource claim without waiting; unavailable allocation yields Inconclusive rather than a new unbudgeted pool or Oracle gate. Old trials retain their original revision semantics.

## 5. Enforcement and handoff

[[dream-log]] §8 defines the three finite models and broken controls. DreamSink must enable HandEdit **after Check and before publish**; the rejected check/rename sequence is the broken control, never modeled as atomic CAS. Lifecycle/merge explore two harnesses, duplicate/reordered results, terminal fencing, crash/restart and revival. Assert scenario reachability in `tests/formal_nonvacuity.rs`; full native/Java reachable-state/transition and safety/deadlock/liveness parity remains required.

Real unattended witnesses in §§9,12.2 cover concurrent hand writes, two publishers, changed ownership, shared/new Claude indexes, Codex/Grok resources, crash around publication logging and actual engine/ACP outbound roles/bytes/Recall pins. Product code and models are **unproven until those scripts run**.

The exact WP1–9 source/test/fixture/wiring path and verify amendments are [[dream-log]] §10. WP5 needs `tests/formal_nonvacuity.rs`, optional export changes, and formal-parity coverage; Nyx alone amends its task. WP9 owns canonical good/broken audit and scripted smoke wiring and final current-source evidence refresh. T1/T4 owner boundaries and real term API/formal witnesses are §12.4; no outside-repository implementation or expanded root authority is granted here.

The original task's `git diff --check && rg -q dream-log .bboxy/specs/log-ir-v1.md` still exits 1 while the IR mirror is held by number_conflict. This decision unblocks readable contracts; it does not turn that failure into acceptance.
