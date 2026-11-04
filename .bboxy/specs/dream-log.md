---
type: spec
title: 'Spec: the dream log — dreaming written in the bboxy IR (1.3), pipeline, numbers, provenance graph, invariants, models, tests and work packages'
status: baselined
links:
- to: '[[log-ir-v1]]'
  rel: extends
  weight: 3
- to: '[[memory-graph]]'
  rel: see
  weight: 2
- to: '[[dream-sinks]]'
  rel: see
  weight: 2
- to: '[[swarm-dreaming]]'
  rel: builds-on
- to: '[[d5963-dream-adoption-and-cross-repo-scope]]'
  rel: constrained-by
- to: '[[d5974-memories-are-kb-records]]'
  rel: constrained-by
- to: '[[log-is-truth]]'
  rel: constrained-by
- to: '[[secrets-never-persist]]'
  rel: constrained-by
- to: '[[never-block]]'
  rel: constrained-by
- to: '[[budgets]]'
  rel: constrained-by
id: 01M6J5WGRKN1SDGC6R0F79N5N7
updated: 2026-11-04T02:45:41.101Z
number: 5967
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:84a2c7a7e8c1cf8ad0702504578c4f168f02982df61e9ac2b1cd105de5b757ab
  frontmatter_digest: sha256:7d757effa32b437b502d05046740ee4dc6a01d27a5410a4c04fe19db739a7e1b
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: bhlOqEF1+mYsYXmpFhhOB7rnfprwFfNq3dqljDLKiqI4g6AYs7CjClJCZ/F3OI75IndS0A1iCduUXe3Qs1RIAw==
  log: .bboxy/log/2026-11-04.jsonl#40209
---
# The dream log

**Operator, 2026-11-03:** "we should extend or adopt our IR to use it as a dream log as well, yes lets learn from everything automatically, everything is a graph, so provenance is important each dream should have provenance and origin, we should always be able to track how a memory came to be!"

**Rule.** The IR holds how each memory came to be. The KB holds what each memory says ([[memory-graph]], d5974).

- Every dream run, every memory write, trial, adoption and recall is a durable IR item.
- The provenance graph is `project(logs + memory records)`, and rebuilding it is deterministic.
- No second store. D0's cache is derived data and can be rebuilt (§4.1).

**Code home.** `bboxy450-harness`, under a new module `src/dream/`, plus the protocol types. The IR stays in this repo until [[log-ir-v1]] §6 moves it.

## 1. Facts this spec rests on (verified at harness b711e93, term main, 2026-11-03)

| Fact | Evidence |
|---|---|
| Fork copies the parent's first `at` lines byte-identically and links blobs | `src/store/fork.rs:107` |
| A session has one writer, enforced by `flock` | `src/store.rs:136-139` |
| Headless run: `-p PROMPT --json --profile P --sessions DIR`, closes on the first `TurnEnded` | `src/frontends/headless.rs:262-288, 575` |
| A session-close hook already runs offline work off the hot path | `src/ir_scan.rs:594-666`, engine `src/engine/mod.rs:350-371`, ACP `src/backend/acp_agent.rs:357`, watch `src/import/watch.rs:82` |
| The engine system prompt is fixed and loads no memory | `src/engine/mod.rs:888-901` |
| ACP prior context is flattened into the first prompt, budget 24 KiB | `src/backend/acp_agent.rs:2038, 769-772` |
| No line digest of envelope bytes exists; the raw line is retained | `src/protocol/mod.rs:1089` `wire_bytes` |
| `SessionMeta` has no VCS fields; the importer drops Claude `gitBranch` and Codex `git{commit_hash,branch,repository_url}` into `ext` | `src/protocol/mod.rs:469`, `src/import/edge.rs:66` |
| Tool output lives in `ToolCall.output`; there is no `ToolResult` item | `src/protocol/mod.rs:313` |
| The redactor replaces only registered literal secrets | `src/redact.rs:84-128` |
| Formal models are Rust builders, listed in three inventories | `tests/formal/models.rs`, `verification/{corpus,coverage,contracts}.toml`, `scripts/formal.py:327-330` |
| The IR gate | `make test-ir`, `make lock-ir`, `tests/fixtures/ir-v1/v<maj>.<min>-*.jsonl`. `PROTOCOL = {1,2}` (`src/protocol/mod.rs:14`); note `ir-compat.py:331` hardcodes `{1,0}` in the lock |
| Task records carry `assignee, verify, verify_result{command,exit_code,evidence}, status, rejection{reason,by}, report{commit,…}` | term `crates/bbt-mux/src/mcp/work_items.rs:1195-1345` |
| A seat keeps only its latest `harness_session_id`, with no history | term `crates/bbt-proto/src/registry.rs:101` |
| Claude records carry `gitBranch` and no commit; Codex `session_meta.git` has `commit_hash` | native logs on this machine |

## 2. IR 1.3 additions

All additions are additive (E1). Every new union and enum gets `Unknown(Value)` (E4), and writers never write it (E5). Rust shapes:

```rust
// SessionMeta
pub vcs: Option<Vcs>,          // #[serde(default, skip_serializing_if = "Option::is_none")]
pub dream: Option<DreamRun>,   // same attributes
pub struct Vcs { pub repo: Option<String>, pub branch: Option<String>, pub commit: Option<String>, pub dirty: Option<bool> }
pub struct DreamRun { pub phase: DreamPhase, pub trigger: Trigger, pub scope: Scope,
    pub parent: Option<SessionId>, pub prompt_version: Option<String>, pub thresholds: Option<String> }
pub enum DreamPhase { Replay, Synthesis, Trial, Lower, Flush, Audit, Unknown(Value) }
pub enum Trigger { SessionClosed { session: SessionId }, Quota { provider: String, resets_at: u64 },
    Operator { message: Option<String> }, Dream { entry: String }, Schedule { name: String }, Unknown(Value) }
pub struct Scope { pub level: ScopeLevel, pub repos: Vec<String> }   // ScopeLevel { Repo, System, Unknown(Value) }

pub enum EvidenceRef {
    Event { session: SessionId, seq: Seq, digest: String },                     // untrusted
    Record { root: String, id: String, path: String, commit: String, digest: String }, // trusted iff §6.2
    Unknown(Value) }
pub struct RecordPin { pub root: String, pub id: Option<String>, pub path: String, pub digest: String }

// Item variants (durable)
Item::MemoryWrite { entry: String, op: MemoryOp, record: RecordPin, evidence: Vec<EvidenceRef>,
    draft: Option<Value> }               // op: Pending|Created|Revised|Unknown; draft = full record (frontmatter+body) while Pending
Item::MemoryState { entry: String, state: EntryState, harness: Option<String>, reason: String, refs: Vec<EvidenceRef> }
Item::Trial { entry: String, harness: String, thresholds: String, episodes: Vec<TrialEpisode>,
    controls: Vec<TrialEpisode>, verdict: Verdict, reason: String }
pub struct TrialEpisode { pub origin: EvidenceRef, pub runs: Vec<TrialRun> }
pub struct TrialRun { pub arm: Arm /* With|Without|Unknown */, pub session: SessionId, pub verify_exit: Option<i32>, pub usage: Option<Usage> }
Item::Adoption { entry: String, harness: String, sink: SinkRef, block_digest: String }
pub struct SinkRef { pub kind: SinkKind, pub path: String }
// SinkKind { ClaudeMemory, CodexExtension, GrokInbox, EngineContext, AcpContext, Proposal, Unknown(Value) }
Item::Recall { entry: String, sink: Option<SinkRef>, basis: RecallBasis }   // Reported|Inferred|Cited|Unknown
```

- **`EntryState`:** `Proposed | Tested | Adopted | Rejected | Superseded | Expired | Retired | Unknown`.
- **`Verdict`:** `Pass | Fail | Inconclusive | Unknown`.
- **`ItemKind`** gains the 5 matching kinds.

**Line digest.** `digest = "sha256:" + hex(sha256(line))`, where `line` is the exact stored bytes of the envelope without the trailing `\n`, as retained by `wire_bytes`. A record digest is `sha256` of the file bytes at `commit`.

**EntryId.** `"sha256:" + hex(sha256(canonical))`.
- `canonical` is the JSON of `{kind, title, why, how_to_apply, evidence, derived_from, revival}`.
- Object keys are sorted, there is no whitespace, `evidence` and `derived_from` arrays are sorted by their canonical JSON, and strings are NFC-normalised.
- `revival` is 0, or n for the n-th re-proposal after a terminal state.

**Fixtures.** `tests/fixtures/ir-v1/v1.3-dream-session.jsonl`, `v1.3-recall.jsonl` and `v1.3-vcs.jsonl`. `make lock-ir` is committed with the change. Fixing the hardcoded `{1,0}` at `ir-compat.py:331` is part of WP1.

**Who writes what:**

| Item or field | Writer |
|---|---|
| `SessionMeta.vcs` | engine and ACP backends at session start (`git rev-parse HEAD`, `--abbrev-ref HEAD`, `status --porcelain` empty ⇒ `dirty: false`; remote per [[memory-graph]] §1.1). Importer: Codex `session_meta.git` → `repo/branch/commit`; Claude `gitBranch` → `branch` only. Never invented |
| `DreamRun`, `MemoryWrite`, `MemoryState`, `Trial`, `Adoption` | the dreamer only, in dream sessions (`harness = "dream"`) |
| `Recall` | the engine and ACP backends when they inject (`reported`). The importer for Claude Code (`inferred`: the adoption timeline of the project's sink at `created_ms`). The importer for Codex (`cited`: `<oai-mem-citation>` entries naming `extensions/bboxy/resources/<entry>.md`; otherwise `inferred`). Grok: `inferred` |

## 3. CLI (read-only queries plus pipeline commands)

`bboxy450-harness dream <sub>` is added to the `run_cli` match at `src/frontends/headless.rs:93-224`. Every subcommand:
- takes `--sessions DIR` (default `~/.bboxy450-harness/sessions`) and `--json`;
- exits 0 on success, 1 when a check is violated, 2 on bad usage or input errors.

| Sub | Does |
|---|---|
| `replay [--rebuild] PATH…` | D0 over closed sessions or native files. It updates the cache and writes no session |
| `synthesize [--cluster FP] [--profile P] [--trigger …]` | D1. It writes a dream session with `MemoryWrite{Pending}` items |
| `trial ENTRY [--harness H]` | Trial runs (§4.3). It writes `Trial` and `MemoryState` |
| `flush` | Inside a bboxy seat only (`BBOXY_SOCK` and `BBOXY_CAPABILITY_FILE` present): writes pending drafts through MCP `knowledge_write`, records `MemoryWrite{Created|Revised}` with the returned id and digest, then `knowledge_commit`. Outside a seat it is a no-op that exits 0 with a notice |
| `lower [--harness H]` | Regenerates the sink blocks ([[dream-sinks]]) and records `Adoption` |
| `run` | `replay` → `synthesize` (if quota allows) → `trial` (eligible) → `flush` → `lower`. Each step degrades on its own |
| `why <entry \| path:line \| slug>` | Prints the provenance chain: sink line → entry → trial → dream session → trigger → evidence lines (redacted text) |
| `graph [--strength]` | `{nodes, edges}` JSON with strength ([[memory-graph]] §3) |
| `audit` | The gate (§7) |

## 4. Pipeline

### 4.1 D0 replay (deterministic, no model)

- **Trigger:** `ir_scan::session_closed` gains a sibling `dream::d0::session_closed` with the same detached pattern (`src/ir_scan.rs:594-666`). Also `dream replay` over native files (Claude, Codex) via `import`.
- **Episode:** one task, or one turn when no task joins.
  - **Task join, from the IR alone:** a session belongs to task T if it contains a `ToolCall` whose name ends in `task_report`, `task_accept` or `task_reject`, and whose input JSON contains T's `id`, `#number` or path.
  - Verdict:
    - `accepted`: status `accepted`/`done` with `verify_result.exit_code == 0`
    - `rejected`: a `rejection` is present
    - `failed`: `verify_result.exit_code ≠ 0`
    - `none`: otherwise
  - Task records are read from `git show <repo>/origin/main:.bboxy/tasks/*.md`, so their `EvidenceRef::Record.commit` is real.
- **Fingerprints** (deterministic strings, hashed sha256):
  - `tool-fail:<tool name>:<first line of output, normalised>`. Normalising lower-cases the text and replaces digits, hex runs of 7 or more, absolute paths and quoted strings with placeholders.
  - `retry:<tool>:<normalised args>`, for 2 or more identical calls in a turn followed by success.
  - `quiet-stop`; `approval-denied:<tool>`; `cost-outlier:<backend>/<model>`, for a turn above the p95 of reported usage for that pair.
  - `rejected:<normalised rejection reason>`; `verify-fail:<verify command>`.
- **Cache:** `~/.bboxy450-harness/dream/d0/<repo-key-hash>/<fingerprint>.json` holding `{fingerprint, count, episodes: [EvidenceRef], first, last}`. It merges by union, so merges commute. Nothing cites the cache.
- **Budget:** ≤ 1 ms CPU between awaits, never on the UI thread ([[budgets]] §1).

### 4.2 D1 synthesis

- **Eligible cluster:** ≥ 3 episodes, at least 1 of them with a `Record` verdict anchor, in a scope whose repos are allowed (§6.3).
- **Model run:** an ordinary engine session with `SessionMeta.dream{phase: Synthesis}`, the profile given by `--profile` (default: the default profile), and prompt `dream-synthesis-1` ([[memory-graph]] §6.1).
- **Episode rendering:** assembled from the IR with `ToolCall.output` truncated to 4 KiB per block, passed through `redact::Set` and the deny scan (§6.1).
- **Output:** the model's JSON is parsed strictly. Each valid candidate becomes a `MemoryWrite{op: Pending, draft}` plus `MemoryState{Proposed}`.

### 4.3 Trial

- **Eligible episode:**
  - it has a `Vcs.commit` and `dirty == false`;
  - its task has a `verify` and a recorded exit;
  - the original verify ran in ≤ 10 min, from the `.bboxy/runs` log `elapsed` when present; otherwise the episode is eligible only if its verify command is not `make test`;
  - it is not among the entry's evidence (held out);
  - it is in a different session from any `Recall` of the entry or its `derived_from`/`supersedes` closure.
- **Each run:**
  1. `git worktree add --detach <tmp> <commit>` in a scratch copy, removed afterwards.
  2. Fork the episode's session at the last durable seq before the episode turn's `TurnStarted` (`SessionMeta.fork`).
  3. Continue headless in `<tmp>` with the episode's harness profile, re-sending the original `UserMessage` blocks:
     - `with` arm: the [[memory-graph]] §6.2 block containing only this entry;
     - `without` arm: no block.
  4. After `TurnEnded`, or after 30 min, run the task's `verify` in `<tmp>` and record the exit code.
  - Imported native episodes replay through the ACP agent profile for that harness, so both arms use the same lossy handoff.
- **Isolation:** trial sessions run with a policy profile `dream-trial` that denies `git push`, `gh` writes, `cargo publish`, network installs and writes outside `<tmp>` (enforced by `src/policy.rs`, [[policy-gates-effects]]). At most one trial runs per machine at a time ([[budgets]]; the CPU memory rule: one full test at a time).
- **Verdict:** under thresholds `dream-thresholds@1` (§5). The result is written as `Trial`, then `MemoryState{Tested}`, then `Adopted` or `Rejected` per harness.

### 4.4 Lower, recall, retire

- **Lower:** [[dream-sinks]]. Each block write records `Adoption{block_digest}`.
- **Recall:** per §2.
- **Retire:** checked in `dream run`.
  - **Expired:** `expires` has passed and a re-trial is not `Pass`. A re-trial that passes extends `expires` by 30 days.
  - **Retired:** live evidence, as defined in §5.

### 4.5 Triggers, claims, no chokepoint

- **Triggers:**
  - D0 runs on every session close.
  - `dream run` is started by the `Quota` trigger, an `Operator` request, or a `Schedule` (the night run).
  - Any seat or cron may start it. None is required.
- **Claims:** a shard is a cluster fingerprint. Before work on a cluster or entry, the dreamer takes `flock(LOCK_EX|LOCK_NB)` on `~/.bboxy450-harness/dream/claims/<shard>`. The file content is `{pid, seat?, started}`. If the lock is busy, the shard is skipped, not waited for. The lease is released when the process exits.
- **Quota input:** `<term run dir>/usage.json` ([[dream-sinks]] §6, T3).
  - D1 and trials start only when the chosen provider window has `used < 0.80`, `resets_at − now ≤ 3 h`, and the file age is ≤ 600 s.
  - The check is repeated between jobs, and work stops once `used ≥ 0.90`.
  - When the file is missing or stale, the quota trigger never fires and the other triggers still work.

## 5. Numbers: `dream-thresholds@1`

| Name | Value |
|---|---|
| D1 cluster minimum | 3 episodes, at least 1 trusted anchor |
| D1 candidates per cluster per run | ≤ 3 |
| D1 input per run | ≤ 60k tokens. Spend ≤ 200k tokens per run (reported usage, else estimated) |
| Trial held-out episodes N | 4 (fewer available ⇒ `Inconclusive`, and the entry stays `Tested`) |
| Runs per arm per episode R | 2 (16 runs) |
| Control episodes | 2, from other clusters, R = 1 |
| Pass | `passes(with) − passes(without) ≥ 2` over the 8 runs per arm. Controls: `passes(with) ≥ passes(without)`. Cost: `median(tokens with) ≤ 1.25 × median(tokens without)` |
| Per-entry trial spend cap | 2M tokens. Exceeding it ⇒ `Inconclusive` |
| Per-run timeout | 30 min to `TurnEnded`, then 10 min for verify |
| Expiry | 30 days after adoption |
| Live retirement | ≥ 10 recall sessions with verdicts in the cluster, and `accept_rate(recall) − accept_rate(no recall) ≤ −0.20` |
| Entry text | title ≤ 120, why ≤ 1500, how_to_apply ≤ 1500, body ≤ 8 KiB |

Changing any number creates a new revision `dream-thresholds@n` in this table's change log. `Trial.thresholds` names the revision that was applied.

## 6. Safety

### 6.1 Deny scan

Every model input and every record or sink write must pass two checks:
1. `redact::Set::value`, which removes registered secrets.
2. A deny regex list in `src/dream/deny.rs`: `sk-[A-Za-z0-9_-]{20,}`, `sk-ant-`, `gh[pousr]_[A-Za-z0-9]{20,}`, `github_pat_`, `AKIA[0-9A-Z]{16}`, `-----BEGIN [A-Z ]*PRIVATE KEY-----`, `xox[baprs]-`, `eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.`, `(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*\S{8,}`, and `https?://[^/\s]*:[^@\s]*@`.

On model input, a match is replaced with `<redacted>`. In a candidate or sink line, a match **rejects** the candidate. This mirrors Codex's `secrets/src/sanitizer.rs:4-22`. Grok does not redact memory.

### 6.2 Trust

- A trusted anchor is an `EvidenceRef::Record` to a task record with `verify_result` or `rejection`, or to a `decision` record, committed on `origin/main` with a valid KB `provenance.sig`.
- Every `Event` is untrusted.
- **Auto-adoption into context sinks** requires at least one trusted anchor in the entry's evidence closure.
- **Instruction sinks** only ever receive proposals.

### 6.3 Scope

- Every `Event`'s session repo (§2 `Vcs.repo`, else [[memory-graph]] §1.1) must be in `Scope.repos`.
- A `Scope` of level `system`, or one with more than one repo, requires each listed repo's `.bboxy/dream.toml` to have `cross_repo = true` at its `origin/main`.

### 6.4 Self-reinforcement

The evidence set excludes events from any session that recalled the entry or its closure (§4.3).

## 7. `dream audit`, the gate

Exit 0 means every check below holds.
- Each check has a broken fixture under `tests/fixtures/dream/broken/<P>.jsonl` (plus files) that must exit 1 naming that P.
- The good fixture `tests/fixtures/dream/good/` must exit 0.

| # | Check |
|---|---|
| P1 | **Traceable.** Every `Adopted` entry's closure (evidence, `derived_from`) is acyclic and finite, and every `Event` resolves: the session exists, the seq exists, and the line digest matches. Every `Record` resolves (`git show commit:path`, sha256 matches) |
| P2 | **Files match the log.** For every sink, the marked lines (`dream:<entry>`) are exactly the entries `Adopted` for that harness and scope. The block digest equals the last `Adoption.block_digest`. Unmarked lines are untouched since the previous `Adoption` |
| P3 | **Origin.** Every dream session has `DreamRun.trigger` that is not `Unknown`. Every `MemoryWrite` is in a dream session |
| P4 | **Append-only.** No dream session's writer touched a session it cites. `EntryId` recomputed from the draft or record equals `entry` |
| P5 | **Adopted means passed.** Every `Adopted{harness}` is preceded by `Trial{harness, verdict: Pass}` for that entry under a known thresholds revision |
| P6 | **Trust barrier.** Every adopted entry has a trusted anchor. No `Adoption.sink.kind` other than `Proposal` points at an instruction-sink path ([[memory-graph]] §5) |
| P7 | **Scope.** §6.3 holds for every entry |
| P8 | **Secrets.** Every `MemoryWrite.draft`, record body and sink block passes §6.1 with zero matches |
| P9 | **Record projection.** Each KB memory record's `status`, `harnesses`, `entry` and `content_sha256` equal the log projection ([[memory-graph]] §4) |

`dream audit` joins `make test`, over fixtures, and the night run, over real data.

## 8. Formal models

Rust builders in `tests/formal/models.rs`. Each is registered in `verification/corpus.toml`, `coverage.toml` (crate `bboxy450-harness`, sources `src/dream/**`) and `contracts.toml`, with a `Broken*` control.

| Model | State | Actions | Safety | Liveness | Bounds | Broken control |
|---|---|---|---|---|---|---|
| `DreamLifecycle` | `state[e] ∈ EntryState`, `trial[e,h]`, `sink[h] ⊆ E`, `crashed` | Propose, TrialPass, TrialFail, TrialInconclusive, Adopt, Lower, Supersede, Expire, Retire, Crash, Restart, Revive | AdoptedHasPass (P5); SinkSubsetAdopted after Lower; StateMonotone under max order ([[memory-graph]] §4); RevivalNewId | ProposedLeaves: WF(TrialPass ∨ TrialFail ∨ TrialInconclusive) ⇒ ◇(state ≠ Proposed) | 2 entries, 2 harnesses, 1 crash, 1 revival | `BrokenDreamLifecycle`: Adopt enabled without TrialPass |
| `DreamMerge` | two dreamer logs `L1, L2`, projection `P` | Write1, Write2 (any state for the same entry), Merge | Commutative: `P(L1∪L2)` is independent of interleaving; terminal ties are deterministic | Converges once writes stop | 1 entry, 3 writes each | `BrokenDreamMerge`: last writer wins by arrival order |
| `DreamSink` | file lines `{hand, marked}`, adopted set, concurrent hand writer | HandAppend, HandEdit, Regenerate (read → replace block → rename, retry ≤ 3 when the digest changed) | HandLinesPreserved; BlockEqualsAdopted after a successful Regenerate | Regenerate eventually succeeds when hand writes stop | 2 hand writes, 2 adopts | `BrokenDreamSink`: whole-file rewrite from a stale read |

## 9. Tests (all unattended, `HOME` and sessions in temp dirs, no network)

| Level | Test | Covers |
|---|---|---|
| unit | `dream::entry_id` canonical vectors; `deny` patterns hit and miss; fingerprint normalisation; repo key; trust classification; strength formula vectors | §2, §4.1, §6 |
| integration | `tests/dream_pipeline.rs`: fixture sessions → `replay` → `synthesize --scripted <SSE>` (the existing scripted provider) → `trial` against a toy git repo whose `verify` is `sh -c 'test -f fixed'`, with a scripted agent that creates or skips `fixed` depending on whether the memory block is present → `lower` into a temp `HOME` → `audit` exits 0 | §3-§5, P1-P9 |
| integration | `tests/dream_audit_broken.rs`: every broken fixture exits 1 with its P | §7 |
| integration | `tests/ir_v1_corpus.rs` gains the 1.3 fixtures (open, replay, fork with byte identity) | E6, E8 |
| formal | the three models and their broken controls in `make test-formal` | §8 |
| smoke | `bboxy450-harness dream run --sessions <tmp corpus>` with a scripted profile, outside a seat: exits 0, `flush` is a no-op notice, and the sink block is written | §4.5 |
| e2e (night run, not in `make test`) | `dream run` over real local sessions, inside a seat, with `--dry-sinks` (writes blocks to a temp copy); report `.bboxy/runs/dream-<date>.json` | validation |

## 10. Work packages (for the Oracle; each is one Executor task)

| WP | Scope | Allowed paths | Depends | Verify |
|---|---|---|---|---|
| 1 IR 1.3 | §2 types, schema, lock, fixtures; fix `ir-compat.py:331` | `src/protocol/**`, `protocol.schema.json`, `verification/protocol-v1.lock.json`, `scripts/ir-compat.py`, `tests/fixtures/ir-v1/**`, `tests/protocol_golden.rs`, `tests/ir_v1_corpus.rs` | — | `make test-ir && cargo test --test protocol_golden --test ir_v1_corpus` |
| 2 VCS | `SessionMeta.vcs` at start (engine, ACP); importer mapping | `src/engine/**`, `src/backend/acp_agent.rs`, `src/import/**`, `tests/import*.rs` | 1 | `cargo test --test cli_headless --test import_native` (or the existing importer test name) |
| 3 D0 | `src/dream/{mod,d0,fingerprint,join,repo}.rs`, the close hook, `dream replay` | `src/dream/**`, `src/ir_scan.rs` (hook only), `src/frontends/headless.rs` (dispatch), `tests/dream_*.rs` | 1 | `cargo test dream::` |
| 4 Audit | `dream audit/why/graph`; good and broken fixtures | `src/dream/{audit,graph}.rs`, `tests/fixtures/dream/**`, `tests/dream_audit_broken.rs` | 1 | `cargo test --test dream_audit_broken` |
| 5 Models | the three models and controls; the three inventories | `tests/formal/models.rs`, `verification/*.toml` | — | `make test-formal` |
| 6 Synthesis | prompt, strict parse, `MemoryWrite{Pending}`, deny scan | `src/dream/{synth,deny,prompt}.rs` | 3, 4 | `cargo test --test dream_pipeline synth` |
| 7 Trial | worktree, fork-continue, `dream-trial` policy, verdict | `src/dream/trial.rs`, `src/policy.rs` (profile only), `src/frontends/headless.rs` | 2, 6 | `cargo test --test dream_pipeline trial` |
| 8 Flush + sinks + recall | MCP writes in a seat; [[dream-sinks]] lowering; engine and ACP injection and `Recall` | `src/dream/{flush,lower}.rs`, `src/engine/mod.rs` (`request_context`), `src/backend/acp_agent.rs` (first-prompt block) | 4, 7 | `cargo test --test dream_pipeline && make test` |
| 9 Triggers | quota reader, claims, `dream run`, night-run script | `src/dream/{run,claims,quota}.rs`, `scripts/dream-night.sh` | 8 | `cargo test dream::run && make test` |

Every package also keeps `make test`, `check_transcript` and `make test-formal` green, signs every commit, and ships its report under `.bboxy/reports/`.

## 11. Interface items outside this repo ([[dream-sinks]] §6)

- **T1:** the term index resolves `[[kb:<root>/<slug>]]` across mounted roots.
- **T2:** none. Flush uses the existing seat MCP `knowledge_write`/`knowledge_commit`.
- **T3:** `usage.json` lands on term main.
- **T4:** the system root `~/.agents/.bboxy` is mounted as a knowledge root.

Until T4, system-scope candidates stay `Pending` in the log. That costs nothing, because nothing blocks on them.

## Change log

- 2026-11-03, Vesper Greywind <ARCHITECT>: rewritten to implementation-ready from code evidence (§1). Memory content moved to KB records (d5974, [[memory-graph]]). Added thresholds@1, the deny scan, audit P1-P9, three models, the tests, and WP1-9.
- 2026-11-03, Vesper Greywind <ARCHITECT>: first draft.
