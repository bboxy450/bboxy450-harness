---
type: spec
title: 'Spec: the dream log — dreaming written in the bboxy IR, with a provenance graph from every memory back to the events it came from'
status: draft
links:
- to: '[[log-ir-v1]]'
  rel: extends
- to: '[[swarm-dreaming]]'
  rel: builds-on
- to: '[[d5963-dream-adoption-and-cross-repo-scope]]'
  rel: constrained-by
- to: '[[log-is-truth]]'
  rel: constrained-by
- to: '[[secrets-never-persist]]'
  rel: constrained-by
id: 01M6J5WGRKN1SDGC6R0F79N5N7
updated: 2026-11-04T02:23:23.545Z
created: 2026-11-04T02:23:23.545Z
number: 5967
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:c8b2d82a9245ed349894662e8fdb180d39ee305037fcae8581a9297cdba35c93
  frontmatter_digest: sha256:f4efe871a24cc524f4c5a0dfc0f4660abb97b812f927848fde3eee714569432c
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: 7G6EUNkxD7VlZUnmg1YaNyG7VkTP/FBJ8T3rR9jHHMIoKxAxZtxsHsVmRzxUZDoeQ+jcXGcnNLTiHO6Kk1CSDw==
  log: .bboxy/log/2026-11-04.jsonl#40164
---
# The dream log

**Operator, 2026-11-03:** "we should extend or adopt our IR to use it as a dream log as well, yes lets learn from everything automatically, everything is a graph, so provenance is important each dream should have provenance and origin, we should always be able to track how a memory came to be!"

**Rule.** Dreaming writes only bboxy IR. No second store.

- Every dream run, every memory, every test, every adoption and every recall is a durable item in a session log.
- The provenance graph is a projection of the logs: `graph = project(logs)`. Rebuilt from the logs alone, it is identical every time ([[log-is-truth]]).
- Every memory can be followed back, edge by edge, to the raw events it came from. A memory without that chain cannot exist in a sink.

## 1. Why the IR and not a new store

The IR already has the three things a provenance graph needs:

- **Stable node identity:** `(session, seq)` plus the original line bytes, kept by E6, so a digest of a line never changes.
- **Derivation edges:** `SessionMeta.fork` and `reproject`.
- **Producer identity:** `SessionMeta.source`, `TurnStarted.model`, `Usage.backend`.

What is missing is a set of item kinds for memories and their lifecycle. These are all additive (E1), proposed for **IR 1.3**.

## 2. IR 1.3 additions (all optional fields or new variants)

| Addition | Shape | Meaning |
|---|---|---|
| Dream session | `SessionMeta.dream: Option<DreamRun>`; `harness = "dream"` | Marks a session as a dream run |
| `DreamRun` | `{ phase: DreamPhase, trigger: Trigger, scope: Scope, parent: Option<SessionId> }` | `DreamPhase` is `replay \| synthesis \| trial \| lower \| audit \| unknown`. `parent` links a dream started by another dream |
| `Trigger` (origin) | `session_closed{session} \| quota{account, reset_ms} \| operator{message?} \| dream{entry} \| schedule{name} \| unknown` | Why this run exists. **Every dream has an origin** |
| `Scope` | `{ repos: Vec<String>, cross_repo: bool }` | `cross_repo` is true only when every repo listed has opted in ([[d5963-dream-adoption-and-cross-repo-scope]]) |
| `EvidenceRef` | `{ session, seq, digest, trust: Trust }` | Points at one source line. `digest` = sha256 of its original bytes. `Trust` is `trusted \| untrusted \| unknown`, derived from the item kind (§4) |
| `Item::Memory` | `{ entry: EntryId, kind: MemoryKind, text, evidence: Vec<EvidenceRef>, derived_from: Vec<EntryId>, supersedes: Vec<EntryId>, scope: Scope, harness: Option<String>, expires_ms: Option<u64> }` | A candidate memory. **Immutable:** a change is a new entry that `supersedes` the old one. `EntryId` = sha256 of the canonical `(kind, text, evidence, derived_from)`, so equal content means equal id. `MemoryKind` is `lesson \| workflow \| brief_template \| invariant_candidate \| fact \| unknown` |
| `Item::MemoryState` | `{ entry, state: EntryState, reason, refs: Vec<EvidenceRef> }` | A lifecycle step. `EntryState` is `proposed \| tested \| adopted \| rejected \| superseded \| expired \| retired \| unknown`. `refs` points at what justified the step: the trial item, or the recall data that retired it |
| `Item::Trial` | `{ entry, harness, episodes: Vec<TrialEpisode>, thresholds: String, verdict: Verdict }` | The fork-replay A/B. Each `TrialEpisode` is `{ origin: EvidenceRef, with: SessionId, without: SessionId, verify_with: i32, verify_without: i32, usage_with?, usage_without? }`. Both replays are ordinary forked sessions, so `SessionMeta.fork` links them to `origin`. `thresholds` names the spec revision whose numbers were applied |
| `Item::Adoption` | `{ entry, harness, sink: SinkRef, sink_digest }` | A lowering. `SinkRef` is `{ kind: claude_memory \| codex_memory \| engine_context \| proposal, path }`; `sink_digest` is the digest of the generated file |
| `Item::Recall` | `{ entry, sink: Option<SinkRef>, digest }` | Written in a **live** session whenever a dreamt entry enters its context. The edge from memory to use |

The render rule is unchanged: a face that predates these items shows the neutral placeholder (E4). Writers never write `unknown` (E5).

## 3. The graph

**Nodes:**
- source events `(session, seq)`
- sessions: live, dream, and trial replays
- entries
- sink files, by `(path, digest)`

**Edges:** every edge is read from one item; no edge is written anywhere else.

| Edge | From item |
|---|---|
| entry ← evidence events | `Memory.evidence` |
| entry ← parent entries | `Memory.derived_from` (dreams over dreams) |
| entry → older entry | `Memory.supersedes` |
| entry ← dream run ← trigger/origin | the session holding `Memory`, and its `DreamRun` |
| entry ← trial ← replay sessions ← origin event | `Trial`, `SessionMeta.fork` |
| entry → sink file | `Adoption` |
| live session → entry | `Recall` |
| each state change ← its justification | `MemoryState.refs` |

**Queries (CLI, read-only, offline):**
- `bboxy450-harness dream why <entry | sink-path:line>` prints the full chain back to raw events, with their text after redaction.
- `bboxy450-harness dream graph [--json]` emits the projection.
- `bboxy450-harness dream audit` is the gate (§5).

**Reverse map from a file.** Each generated line in a sink file ends with a marker `<!-- dream:<entry-id> -->`, or the sink's comment equivalent. A sink file is a projection of the adopted set for that `(harness, scope)`, regenerated whole and never patched. A hand-written memory never carries a marker, and the generator never touches a line without one.

## 4. Learning from everything, safely

- **Every session close is a trigger.** D0 replay runs on every closed session of every harness: engine, ACP, Claude Code import, Codex import. Dream and trial sessions close too, so dreams learn from dreams. The `Trigger` and `parent` chain records this.
- **Trust follows item kind.** Trusted: operator `UserMessage`, task verdicts (verify exit, accept/reject). Untrusted: `ToolResult`/`OutputBlock` content, `Item::Native`, citations, file contents, and any `Memory` text. Unknown kinds count as untrusted.
  - An entry can be auto-adopted only if its evidence closure contains at least one trusted event.
  - An entry reaches an instruction sink only as a `proposal` sink.
- **No self-reinforcement.** An entry's evidence excludes every event in a session that contains a `Recall` of that entry, or of any entry in its `supersedes`/`derived_from` closure.
- **Scope.** Every evidence event's session `cwd` must lie within `Scope.repos`. `cross_repo` requires each of those repos to have a committed opt-in.

## 5. Invariants (to be modelled, `DreamLifecycle` + `DreamProvenance`)

| # | Property | Check (to build) |
|---|---|---|
| P1 | **Traceable.** Every adopted entry's provenance closure is finite, acyclic, and ends only in non-dream source events that resolve: the session exists, the seq exists, and the digest matches the line bytes | `dream audit` |
| P2 | **Files match the log.** Every marked line in a sink maps to an entry whose latest state is `adopted` for that harness and scope. Every such entry has exactly one marked line. Each file's digest equals the latest `Adoption.sink_digest` | `dream audit` |
| P3 | **Origin.** Every dream session has `DreamRun.trigger ≠ unknown`. Every `Memory` sits in a dream session | `dream audit` |
| P4 | **Append-only.** No dream action writes to a session it read. Entries are immutable; change happens only by `supersedes` | E6/E8 corpus checks, plus the model |
| P5 | **Adopted means passed.** No `adopted` state without a preceding `Trial` with `verdict = pass` for the same harness under the current thresholds | `dream audit` |
| P6 | **Trust barrier.** No adopted entry has an all-untrusted evidence closure. No non-`proposal` sink holds an instruction-sink path | `dream audit` |
| P7 | **Scope.** Every evidence event of every entry is within its scope, and cross-repo pooling is only between opted-in repos | `dream audit` |
| P8 | **Secrets.** Every `Memory.text` and every sink line passes the redactor unchanged ([[secrets-never-persist]]) | `dream audit` |
| L1 | **Progress.** A `proposed` entry eventually reaches a state other than `proposed`, under fairness on the scheduler | TLC liveness |

Each property gets a broken control: a fixture violating exactly that property, which `dream audit` must refuse with a non-zero exit.

Model bounds to explore:
- two dream runs racing on overlapping evidence
- a crash between `Memory` and `MemoryState`
- a source session forked after it was cited
- adopt → supersede → re-adopt (A → B → A)
- a recall feeding the next dream (the self-reinforcement edge)

## 6. Open numbers (set before synthesis runs, recorded here by revision)

Trial sample size, number of replays per episode, minimum gain, cost cap, control-set size, expiry, and the per-sink line budget (Claude `MEMORY.md` 200 lines / 25 KB total, shared with hand-written memories).

## Change log

- 2026-11-03, Vesper Greywind <ARCHITECT>: draft, from the operator's direction above, [[swarm-dreaming]] and [[d5963-dream-adoption-and-cross-repo-scope]].
