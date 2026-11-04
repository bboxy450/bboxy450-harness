---
type: research
title: 'Swarm dreaming: offline consolidation over the bboxy log IR, across every harness'
status: draft
links:
- to: '[[log-ir-v1]]'
  rel: builds-on
- to: '[[log-is-truth]]'
  rel: constrained-by
- to: '[[secrets-never-persist]]'
  rel: constrained-by
- to: '[[never-block]]'
  rel: constrained-by
- to: '[[lc-ir]]'
  rel: builds-on
id: 01M6J5FZJZ9GG17Q3W59JZ32A5
updated: 2026-11-04T02:19:05.931Z
created: 2026-11-04T02:19:05.931Z
number: 5961
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:a1497ccbc4e5d09f0a4850119d1459a717cb36d437c8abf3fbd9947e9a01b16b
  frontmatter_digest: sha256:a7bb19c727ea7554bc0526e197b2eae8a4862a2e7b04a65bc4e4acc016c4e93c
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: OcA0n+gPmbMNKke9arLwuAHVzMSegRXdZuHnZjW76864j3MYOGcEjyHeZFmhdHNMGnCXPxLyWRTUK8H3Ub7ZCw==
  log: .bboxy/log/2026-11-04.jsonl#40132
---
# Swarm dreaming over the IR

**Question (operator, 2026-11-03).** The log IR is the store format and the render core of the harness. Can it also be the input for *dreaming*: offline processing of past sessions that makes future sessions better? Can that work for the whole swarm and for every harness (engine, ACP agents, Claude Code import, Codex import)?

**Answer.** Yes. Our IR already has most of what published dreaming systems had to build for themselves. Four properties carry it:

1. **One harness-agnostic event log.** Every source is projected into the same IR: `SessionMeta.source`, `Envelope.native`, and `Usage.backend`/`model`/`source` (`log-ir-v1` §3, §4a).
2. **Immutable input.** Old logs are never edited (E8, [[log-is-truth]]).
3. **Derived logs.** `SessionMeta.fork: Option<(SessionId, Seq)>` (`src/protocol/mod.rs:473`) and `reproject` (§7) already produce new logs from old ones.
4. **A working offline miner.** `ir-scan` already turns logs into reviewed changes (§8).

Dreaming generalises §8 from *schema* knowledge to *behavioural* knowledge.

## 1. State of the art (verified 2026-11-03; UNVERIFIED items marked)

| System | Mechanism | Lesson for us |
|---|---|---|
| Anthropic Managed Agents **Dreams** (preview, 2026-05-06, https://platform.claude.com/docs/en/managed-agents/dreams) | Async job: memory store + session ids → new consolidated store. Inputs are never modified; multi-agent; changes auto-apply or wait for review. No published metrics | Dreams read immutable inputs and write a new snapshot. We already have this shape |
| Sleep-time compute (arXiv 2504.13171) | Reason over context before queries arrive. ~5× less test-time compute at equal accuracy; gain grows with how predictable the queries are | Dream about what the swarm will be asked again (open tasks, repos in flight), not about everything |
| Codex CLI Memories (https://developers.openai.com/codex/memories.md) | Extract from idle threads, then consolidate with its own model into local files | Codex is already dreaming privately, in its own format. Our IR can be the shared layer above it |
| Claude Code auto memory | Per-project `memory/` + `MEMORY.md` index (first 200 lines / 25 KB loaded). A "/dream" or Auto Dream feature is UNVERIFIED | A target to lower into, with a hard size budget |
| ExpeL 2308.10144, AWM 2409.07429 (+24.6% Mind2Web, +51.1% WebArena) | Contrast success with failure trajectories → rules and reusable workflows | Mine contrasts, not summaries |
| Voyager 2305.16291, DGM 2505.22954, SICA 2504.15228 | Keep a skill or variant only after it passes a scripted benchmark | Adoption is decided by a script |
| Zep/Graphiti 2501.13956 | Facts carry validity intervals and are invalidated, not deleted | Supersede, never delete |
| MINJA 2503.03704 (98% injection, 77% attack success), AgentPoison 2407.12784, MemoryGraft 2512.16962 | Memory poisoning through ordinary inputs, tool output and repo docs | Taint follows item kind. Tainted evidence never becomes an instruction |
| Complementary learning systems; generative replay; DreamerV3 | Fast episodic memory is replayed into slow structure, and policies are trained "in imagination" | Our fork-at-`seq` replay *is* imagination with ground truth |

The field has no controlled benchmarks for vendor consolidation features (Anthropic, Codex, Letta). Evaluation is the open problem, and it is where we have an edge (§3.4).

## 2. What we have that they don't

- **Labels for free.** Swarm tasks carry a `verify` command, a recorded exit status, and `task_accept`/`task_reject` with a reason. Joining task → seat → session gives trajectories labelled by script, not graded by the agent itself. That is the supervision ExpeL and AWM had to approximate.
- **Every harness in one schema.** A Codex failure and a Claude Code failure on the same repo are comparable rows. Vendor dreaming is single-harness by construction.
- **Counterfactual replay.** A fork at `(session, seq)` reconstructs the exact context just before a decision. A candidate memory can be tested by re-running from that point, with and without the memory, and scoring with the task's own `verify`.
- **Cost on the same rows.** §4a usage makes a dream's spend and its saving measurable in the same units.
- **Local input volume (2026-11-03):** 138 harness sessions (62 MB), 512 Claude Code JSONL (923 MB), 160 Codex JSONL (6.2 GB), 175 `ir-unknowns` fingerprints. That is enough to mine now.

## 3. Shape (proposal, not baselined)

### 3.1 A dream is an IR session

A dream run is an ordinary session log, as follows:

- `SessionMeta.harness = "dream"`.
- Its inputs are evidence pointers, `EvidenceRef { session, seq, digest }`, never copies.
- Its outputs are items in the same log.

Consequences:

- Faces render dreams with no new UI.
- §4a usage meters them.
- Dreams can themselves be dreamt over and audited.
- [[log-is-truth]] holds unchanged: dreaming reads logs and writes new ones.

### 3.2 Two phases, cheap first

| Phase | Analogy | Work | Cost |
|---|---|---|---|
| **D0 replay** | NREM | Deterministic, no model. Segment turns into episodes, join them to task verdicts, fingerprint tool-failure→retry→success chains, `Quiet` stops, permission denials and cost outliers. Same fingerprint-merge store as `ir-scan` | CPU only. Runs at session close, off the hot path, bounded by log size (`budgets` §1) |
| **D1 synthesis** | REM | Model-driven, over D0 clusters only. Contrast verdict-labelled episodes and emit candidate **lessons**, **workflows**, **brief templates**, **invariant candidates**. Every claim cites `EvidenceRef`s. A candidate with no resolvable citation is discarded by code | Paid only from idle quota (§3.5) |

`ir-scan` is D0 for schema knowledge. It already proves the pipeline end to end: catalogue → proposal → task record → Executor → gate.

### 3.3 Candidate lifecycle (model it in TLA+)

`observed → proposed → tested → adopted → superseded | expired | rejected`

Invariants to state and model, each with a broken control:

- **I1 Evidence resolves.** No `adopted` entry has an `EvidenceRef` whose `(session, seq, digest)` fails to resolve against the store.
- **I2 Taint barrier.** An entry whose evidence comes only from untrusted item kinds never reaches an instruction sink. Untrusted kinds: tool results, `Item::Native`, citations, file contents. Trusted: operator messages, task verdicts.
- **I3 Logs are read-only.** No dream action writes to a session it reads.
- **I4 Supersede, don't delete.** An entry leaves the active set only by superseded/expired, with a pointer to what replaced it.
- **I5 Secrets.** Every D1 input and output passes `src/redact.rs`, and [[secrets-never-persist]] holds.
- **Liveness.** A `proposed` entry eventually reaches a terminal or `adopted` state, under fairness on the scheduler. It never sits unconsumed (ZOOM OUT: unconsumed output is cost).

Explore: two dreams racing on overlapping evidence, a dream crash mid-write, a source session forked after a dream cited it, and adoption → supersede → re-adoption (A → B → A).

### 3.4 Adoption is a script

- **Tested** means a fork-replay A/B: sample N held-out episodes from the candidate's cluster, fork each at the decision `seq`, run with and without the entry, and score by the task's `verify` exit code plus §4a cost.
- **Adopt** only when the gain is positive and the variance is bounded. Otherwise reject, and record the numbers.
- Replays are expensive. Sample, and cap per-candidate spend.
- Proposed IR addition (1.3, additive): **`Item::Recall { entry, digest }`**, written whenever a harness injects a dreamt entry into context. Without it we cannot attribute outcomes to memories later. This is the single most important new IR kind for this work.

### 3.5 No chokepoint, no blocking

- No nightly global scan as the only path. D0 is per session at close (like §8 (a)), and merges are keyed by fingerprint, so they commute.
- D1 is sharded by cluster. Any seat can take a shard by claiming it visibly.
- **Budget from expiring quota.** `usage_read` returns reset times per account window. Tokens that would expire unused before a reset fund D1. When the swarm is busy, nobody dreams. Allocate this as an explicit line in `budgets`.
- Dream output never blocks a live session. Recall is best-effort, labelled, and degrades to nothing.

### 3.6 Lowering to each harness (one dream, many sinks)

| Sink | Trust | What lands |
|---|---|---|
| Claude Code `memory/` + `MEMORY.md` | low (context) | adopted lessons, within the 200-line / 25 KB index budget |
| Codex `[memories]` files | low (context) | the same entries, in Codex's format |
| Engine/ACP context at session start | low | `Item::Recall`-logged injection |
| `~/.agents/skills/*`, `AGENTS.md`, `.bboxy` invariants and decisions | **high (instructions)** | **proposals only**, as task or decision drafts the Architect or Oracle disposes. I2 forbids auto-landing here |
| `.bboxy/tasks/` | — | D0 findings that are defects (flaky test, missing ICD line), as `ir-scan --propose` does |

### 3.7 What swarm dreaming finds that single-agent dreaming cannot

- The same failure fingerprint across N seats and harnesses → one root-cause task instead of N retries.
- Brief → reject → re-brief loops → brief template fixes, and ICD lines the Executors kept asking about.
- Spend outliers by `backend`/`model` per task class → routing advice (which harness for which task class).
- Gates refused twice with the same reason → the premise is wrong (the ZOOM OUT rule, measured from logs rather than recalled).

## 4. Risks

- **Poisoning.** Tool output and web content are inside our logs. I2 plus redaction is the defence. Never auto-adopt into instruction sinks.
- **Hallucinated lessons.** Mitigated by mandatory citations (I1) and the A/B gate (§3.4).
- **Self-reinforcement.** A dream cites sessions that were shaped by earlier dreams. `Item::Recall` makes this visible. Exclude recall-influenced episodes from the evidence for the same entry.
- **Cost.** Fork-replay is the expensive step. Use only quota that would otherwise expire, sample, and cap per candidate.
- **Privacy.** Codex's 6.2 GB contains other projects. Scope evidence per repository, and share across repositories only through redacted, explicit sinks.

## 5. Recommended sequence

1. **Spec + ICD** (Architect): `dream` session meta, `EvidenceRef`, `Item::Recall` as IR 1.3 additions; the `DreamLifecycle` TLA+ model with I1–I5 and broken controls.
2. **D0** (Executor): deterministic episode and verdict join, plus failure fingerprints. No model, so it can land immediately.
3. **Recall logging** in the engine/ACP context path. Measure before you optimise.
4. **D1 + fork-replay A/B** on one task class with a clean `verify` signal, e.g. Executor briefs in this repo.
5. Lowering to Claude/Codex memory sinks only after step 4 shows a scripted gain.

## Decisions needed

- **Auto-adoption into low-trust context sinks** (Claude/Codex memory) after a passing A/B, versus proposals only everywhere. Recommendation: auto-adopt into low-trust sinks, proposals only for instruction sinks.
- **Cross-repository evidence.** Recommendation: off by default.
