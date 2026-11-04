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
updated: 2026-11-04T03:09:12.729Z
number: 5967
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
authorship:
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  kind: seat
  public_key: 6HRMC38XoZOA5TUGuPnerzKIZabBnbbBZSCqIQt225c=
  process_generation: '1'
provenance:
  digest: sha256:cb756e4e38aa9306645da0d081e3ea290e2578506a382acdcc2a64138ad63218
  frontmatter_digest: sha256:18d60537b67f957df3380c322e4df483d77d489c764598b87615927674db4fae
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  sig: nC/X2ba8F1N/7w7kLIwmEuonQ4URrut9IRwDepCkiUAvfWj4G+dQu8m9TIt5cPiW2kMb1hQPevBd8aFPvo/aCA==
  log: .bboxy/log/2026-11-04.jsonl#42007
---
# The dream log

**Operator, 2026-11-03:** "we should extend or adopt our IR to use it as a dream log as well, yes lets learn from everything automatically, everything is a graph, so provenance is important each dream should have provenance and origin, we should always be able to track how a memory came to be!"

**Rule.** The IR holds how each memory came to be. The KB holds what each memory says ([[memory-graph]], d5974).

- Every dream run, every memory write, trial, adoption and recall is a durable IR item.
- The provenance graph is `project(logs + memory records)`, and rebuilding it is deterministic.
- No second store. D0's cache is derived data and can be rebuilt (§4.1).

**Refinement precedence (task #5984).** The companion records currently have duplicate KB numbers and their host fetch/update is refused. The corrections in §12 and the refined sections below govern this workstream while their exact mirror edits await host/index repair. This does not claim those companion files were updated. No new identity, number rewrite or shell write bypasses the refusal.

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

All additions are additive (E1). Every new union and enum gets `Unknown(Value)` (E4), and writers never write it (E5). These Rust shapes are schematic: every newly added field, including fields in new variants/structs, is optional or has serde default/omission behavior as E2 requires. Authored validation, rather than a required wire field, enforces the pins and state/harness rules below. Rust shapes:

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
pub struct RecordPin { pub root: String, pub id: Option<String>, pub path: String, pub digest: String, pub commit: Option<String> }

// Item variants (durable)
Item::MemoryWrite { entry: String, op: MemoryOp, record: RecordPin, evidence: Vec<EvidenceRef>,
    draft: Option<Value> }               // op: Pending|Created|Revised|Unknown; draft = full record (frontmatter+body) while Pending
Item::MemoryState { entry: String, state: EntryState, harness: Option<String>, reason: String, refs: Vec<EvidenceRef> }
Item::Trial { entry: String, harness: String, thresholds: String, episodes: Vec<TrialEpisode>,
    controls: Vec<TrialEpisode>, verdict: Verdict, reason: String }
pub struct TrialEpisode { pub origin: EvidenceRef, pub runs: Vec<TrialRun> }
pub struct TrialRun { pub arm: Arm /* With|Without|Unknown */, pub session: SessionId, pub verify_exit: Option<i32>, pub usage: Option<Usage> }
Item::Adoption { entry: String, harness: String, sink: SinkRef, block_digest: String, record: Option<RecordPin> }
pub struct SinkRef { pub kind: SinkKind, pub path: String }
// SinkKind { ClaudeMemory, CodexExtension, GrokInbox, EngineContext, AcpContext, Proposal, Unknown(Value) }
Item::Recall { entry: String, sink: Option<SinkRef>, basis: RecallBasis, record: Option<RecordPin>, block_digest: Option<String> }   // Reported|Inferred|Cited|Unknown
```

- **`EntryState`:** `Proposed | Tested | Adopted | Rejected | Superseded | Expired | Retired | Unknown`.
- **`Verdict`:** `Pass | Fail | Inconclusive | Unknown`.
- **`ItemKind`** gains the 5 matching kinds.

**Line digest.** `digest = "sha256:" + hex(sha256(line))`, where `line` is the exact stored bytes of the envelope without the trailing `\n`, as retained by `wire_bytes`. A record digest is `sha256` of the file bytes at `commit`.

**EntryId.** `"sha256:" + hex(sha256(canonical))`.
- `canonical` is the JSON of `{kind, title, why, how_to_apply, evidence, derived_from, revival}`.
- Object keys are sorted, there is no whitespace, `evidence` and `derived_from` arrays are sorted by their canonical JSON, and strings are NFC-normalised.
- `revival` is 0, or n for the n-th re-proposal after a terminal state.

**Authored semantics.** A flushed RecordPin includes its confirmed commit; an absent commit may be resolved only by a matching live digest and never invented. Pending or unknown-commit writes are not eligible materializations. `Adoption.record` pins the exact memory revision materialized. A reported/cited `Recall` carries that pin and the actual block digest; inferred exposure carries them only when a resolving publication timeline establishes them. Missing pins never become invented ones. `reported` proves insertion, `cited` proves a citation, and `inferred` means possible exposure; none proves model use or instruction authority. No matching timeline means no inferred Recall. All three bases conservatively exclude that session from the entry's evidence/trials; only reported/cited observations can contribute positive recall strength.

`MemoryState` uses the per-harness projection in §12.1. `refs` on an adoption-state transition pins its qualifying `Trial(Pass)`; the trial's EntryId, harness and thresholds revision must match. Revivals include their terminal predecessor in `derived_from`, preventing reuse of an old pass.

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
| `flush` | Inside a bboxy seat only (`BBOXY_SOCK` and `BBOXY_CAPABILITY_FILE` present): writes pending drafts through MCP `knowledge_write`, then `knowledge_commit`; only after confirmed outcomes records `MemoryWrite{Created|Revised}` with the returned id, exact digest and committed RecordPin. Unknown write/commit outcomes are queried by their original operation ID, never blindly repeated; uncommitted drafts stay Pending. Outside a seat it is a no-op that exits 0 with a notice |
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
  - Task paths are enumerated with `git -C <repo> ls-tree -r origin/main -- .bboxy/tasks`, then read with `git -C <repo> show <pinned-commit>:<path>`; a shell wildcard is not a git object path. Their `EvidenceRef::Record.commit` pins the resolved commit.
- **Fingerprints** (deterministic strings, hashed sha256):
  - `tool-fail:<tool name>:<first line of output, normalised>`. Normalising lower-cases the text and replaces digits, hex runs of 7 or more, absolute paths and quoted strings with placeholders.
  - `retry:<tool>:<normalised args>`, for 2 or more identical calls in a turn followed by success.
  - `quiet-stop`; `approval-denied:<tool>`; `cost-outlier:<backend>/<model>`, for a turn above the p95 of reported usage for that pair.
  - `rejected:<normalised rejection reason>`; `verify-fail:<verify command>`.
- **Cache:** `~/.bboxy450-harness/dream/d0/<repo-key-hash>/<fingerprint>.json` holding `{fingerprint, count, episodes: [EvidenceRef], first, last}`. It merges by union of episode pins under a nonblocking fingerprint claim; count is the deduplicated set cardinality, not the sum of competing counts. A busy claim skips this cache update for later replay. Nothing cites the cache.
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
  - Imported native episodes replay through the ACP agent profile for that harness, so both arms use the same lossy handoff. Disable automatic native/bboxy recall in both trial arms and give both the same pinned unrelated context; if that isolation cannot be established, the episode is Inconclusive. Tests use private HOME and disposable scripted identities, never the operator's live memory or seat.
- **Isolation:** trial sessions run with a policy profile `dream-trial` that denies `git push`, `gh` writes, `cargo publish`, network installs and writes outside `<tmp>` (enforced by `src/policy.rs`, [[policy-gates-effects]]). No machine-wide trial lock or central queue serializes candidates. Under `dream-thresholds@2`, arms are serial within their candidate claim; independent candidate preparation, D0, recall and other clusters keep running. Heavy verification uses the existing self-service test-resource claim of the host, acquired without waiting and recorded visibly. If that facility or its allocation is unavailable, skip/defer that heavy arm with an Inconclusive reason; do not invent a capacity allowance, wait for an Oracle, or block unrelated work. A lightweight scripted verifier needs no full-test claim. This allocates no additional share of [[budgets]].
- **Usage validity:** missing/unreported usage does not become a zero-cost arm. If the cost comparison cannot be computed from like-for-like reported (or consistently labelled estimated) values, record Inconclusive.
- **Verdict:** under thresholds `dream-thresholds@2` (§5). The result is written as `Trial`, then `MemoryState{Tested}`, then `Adopted` or `Rejected` per harness.

### 4.4 Lower, recall, retire

- **Lower:** [[dream-sinks]]. Each block write records `Adoption{block_digest}`.
- **Recall:** per §2 and §12.2. Engine context is a separate quoted user/reference message, never appended to the system/developer prompt. ACP carries quoted data before the actual request, never a system/config update.
- **Retire:** checked in `dream run`.
  - **Expired:** `expires` has passed and a re-trial is not `Pass`. A passing re-trial before a terminal Expired event extends `expires` by 30 days. Once terminal, renewal is a new EntryId; a late pass cannot revive it.
  - **Retired:** live evidence, as defined in §5.

### 4.5 Triggers, claims, no chokepoint

- **Triggers:**
  - D0 runs on every session close.
  - `dream run` is started by the `Quota` trigger, an `Operator` request, or a `Schedule` (the night run).
  - Any seat or cron may start it. None is required.
- **Claims:** a shard is a cluster fingerprint. Before work on a cluster or entry, the dreamer takes `flock(LOCK_EX|LOCK_NB)` on `~/.bboxy450-harness/dream/claims/<shard>`. The file content is `{pid, seat?, started}`. If the lock is busy, the shard is skipped, not waited for. The lease is released when the process exits.
- **Quota input:** `<term run dir>/usage.json` ([[dream-sinks]] §6, T3).
  - Producer `used` is integer percent, not a fraction: validate `0..100` and compute `used_fraction = used / 100.0` exactly once (§12.3). Quota-started D1/trials require the chosen provider's worst-first window to have `used_fraction < 0.80`, a future reset with `resets_at − now ≤ 3 h`, `stale == false`, and snapshot age ≤ 600 s. Missing/null/invalid fields disable that automatic trigger.
  - Repeat the check between jobs; start no more inference once `used_fraction ≥ 0.90`. An already running job obeys its existing spend/timeout cap.
  - Missing/stale quota disables automatic inference, not the command: replay, audit, flush and lower still run. A schedule/operator trigger is not a bypass of an unavailable inference budget. No operator prompt is introduced.

## 5. Numbers: `dream-thresholds@2`

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

Changing any number creates a new revision `dream-thresholds@n` in this table's change log. `Trial.thresholds` names the revision actually applied; audits never reinterpret an old trial under a new revision.

**Threshold revisions.**
- `dream-thresholds@1`: the original table and machine-wide single-trial rule, pinned by this record's pre-refinement digest `1926cd39e18e75dc5308d1cab0569dc486cd9cec08b01118b9b2701522ff87e6`.
- `dream-thresholds@2`: all numeric values in the table above are unchanged; §4.3 replaces the single-trial machine cap with nonblocking per-candidate scheduling and the host's existing heavy-test allocation. This explicit revision prevents silently changing that former numeric cap. Percent-to-fraction conversion corrects the consumer's unit, not the 0.80/0.90 gates.

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
| P2 | **Publications match their pins.** Each successful publication's generated bytes/digest and selected budget-fitting subset match its Adoption/RecordPin and the then-adopted `(entry,harness)` pairs. Engine/ACP serve only currently eligible pinned revisions. Missing/unsafe native delivery is labelled skipped, never an Adoption. Hand edits outside generated bytes are legitimate and are not compared with a previous snapshot; concurrent hand-write witnesses must prove the dreamer did not replace or delete them. Changed generated ownership is quarantined/skipped, not restored by overwrite (§12.2) |
| P3 | **Origin.** Every dream session has `DreamRun.trigger` that is not `Unknown`. Every `MemoryWrite` is in a dream session |
| P4 | **Append-only.** No dream session's writer touched a session it cites. `EntryId` recomputed from the draft or record equals `entry` |
| P5 | **Adopted means passed.** Every active `(entry,harness)` pins a qualifying same-pair `Trial(Pass)` under its known thresholds revision; a pass for another harness, record revision or revival never authorizes it. Delayed completion cannot reopen a terminal revision |
| P6 | **Trust barrier.** Every adopted entry has a trusted anchor. No `Adoption.sink.kind` other than `Proposal` points at an instruction-sink path ([[memory-graph]] §5) |
| P7 | **Scope.** §6.3 holds for every entry |
| P8 | **Secrets.** Every `MemoryWrite.draft`, record body and sink block passes §6.1 with zero matches |
| P9 | **Record projection.** The pinned record's `entry`/body digest match its content revision; `status` and `harnesses` follow §12.1. Stale projection metadata and legitimate changed content are distinguished; the latter is preserved as a new proposal and omitted from recall, never overwritten as a repair |

`dream audit` joins `make test`, over fixtures, and the night run, over real data.

## 8. Formal models

Rust builders in `tests/formal/models.rs`. Each is registered in `verification/corpus.toml`, `coverage.toml` (crate `bboxy450-harness`, sources `src/dream/**`) and `contracts.toml`, with a `Broken*` control.

| Model | State | Actions | Safety | Liveness | Bounds | Broken control |
|---|---|---|---|---|---|---|
| `DreamLifecycle` | `state[e,h]`, global terminal[e], pinned trial[e,h], sink[h], crashed | Propose, TrialPass/Fail/Inconclusive, Adopt, Lower, Supersede, Expire, Retire, Crash, Restart, Revive | SamePairPass (P5); NoTerminalReopen; ServedSubsetCurrentlyAdopted; RevivalNewId | ProposedLeaves under WF of eligible trial completion or bounded disposition; no unconditional progress assumption for missing quota/root | 2 entries, 2 harnesses, 1 crash, 1 revival | `BrokenDreamLifecycle`: adopt without same-pair pass; additional cross-harness and stale-revival controls |
| `DreamMerge` | two event sets L1/L2, per-pair projection P | Write1, Write2, Duplicate, Reorder, Merge | union is idempotent/commutative/associative; projection independent of arrival and wall clock; global terminal fences stale adoption | convergence once writes stop and each log is eventually observed | 1 entry, 2 harnesses, 3 writes each | `BrokenDreamMerge`: arrival-order overwrite; cross-harness rejection must also fail |
| `DreamSink` | hand bytes, generated target, published pins, active set, writer stage | Read, Check, HandAppend/HandEdit (including after Check), PublishNoReplace, ConflictSkip, Load, Crash, Restart | NoHandClobber; NoFalseAdoption; ServedPinsValid; generated subset bounded by sink selection | publish or explicit skip; successful retry after quiescence only under fair retry and available target | 2 hand writes, 2 adopts | `BrokenDreamSink`: digest recheck then overwrite-rename; keep HandEdit enabled between the check and rename |

Model clocks, locks and compare operations separately; do not model the rejected read/check/rename sequence as one atomic CAS. Each good model must reach the decisive scenario; a green but unreachable race is vacuous. `tests/formal_nonvacuity.rs` asserts these witnesses and each named broken property. Native-vs-pinned-Java full state/transition/verdict parity remains mandatory; native safety alone does not establish liveness or implementation conformance.

## 9. Tests (all unattended, `HOME` and sessions in temp dirs, no network)

| Level | Test | Covers |
|---|---|---|
| unit | `dream::entry_id` canonical vectors; `deny` patterns hit and miss; fingerprint normalisation; repo key; trust classification; strength formula vectors | §2, §4.1, §6 |
| integration | `tests/dream_pipeline.rs`: fixture sessions → `replay` → `synthesize --scripted <SSE>` (the existing scripted provider) → `trial` against a toy git repo whose `verify` is `sh -c 'test -f fixed'`, with a scripted agent that creates or skips `fixed` depending on whether the memory block is present → `lower` into a temp `HOME` → `audit` exits 0; also drive hand edits at the check/publish boundary, existing-index skip, Codex/Grok no-clobber publication, separate harness verdicts, and actual engine/ACP prompt roles and Recall pins | §3-§5, P1-P9 |
| integration | `tests/dream_audit_broken.rs`: every broken fixture exits 1 with its P | §7 |
| integration | `tests/ir_v1_corpus.rs` gains the 1.3 fixtures (open, replay, fork with byte identity) | E6, E8 |
| formal | the three models and their broken controls in `make test-formal` | §8 |
| smoke | `bboxy450-harness dream run --sessions <tmp corpus>` with a scripted profile, outside a seat: exits 0, `flush` is a no-op notice; a precommitted good-fixture entry lowers successfully while a new Pending draft does not masquerade as a committed adoption | §4.5 |
| e2e (night run, not in `make test`) | `dream run` over real local sessions, inside a seat, with `--dry-sinks` (writes blocks to a temp copy); report `.bboxy/runs/dream-<date>.json` | validation |

## 10. Work packages (for the Oracle; each is one Executor task)

| WP | Scope | Allowed paths | Depends | Verify |
|---|---|---|---|---|
| 1 IR 1.3 | §2; schema/lock/corpus and authored validation | `src/protocol/**`, `protocol.schema.json`, `verification/protocol-v1.lock.json`, `scripts/ir-compat.py`, `tests/fixtures/ir-v1/**`, `tests/{protocol_golden,ir_v1_corpus}.rs`; only mechanical constructors/read-only exhaustive consumers in the Oracle's exact compiler-driven amended list | — | `make test-ir && cargo test --test protocol_golden --test ir_v1_corpus` |
| 2 VCS | VCS capture and shared repo-key helper; importer mapping | `src/vcs.rs`, `src/lib.rs` (vcs declaration only), `src/engine/mod.rs` (meta capture), `src/backend/acp_agent.rs` (meta capture), `src/import/{mod,edge}.rs`, `tests/{dream_vcs,import_native,cli_headless}.rs` | 1 | `cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent` |
| 3 D0 + scaffold | module/CLI dispatch, close hooks, deterministic replay/cache | `src/dream/{mod,d0,fingerprint,join}.rs`, `src/lib.rs` (dream declaration), `src/frontends/headless.rs` (dream dispatch), `src/engine/mod.rs`, `src/backend/acp_agent.rs`, `src/import/watch.rs` (close hooks only), `tests/dream_replay.rs`, `tests/fixtures/dream/replay/**` | 1, 2 | `cargo test dream:: && cargo test --test dream_replay --features acp-agent` |
| 4 Audit + shared validators | audit/why/graph, entry identity, pins/projection, deny scan | `src/dream/{audit,graph,entry,projection,resolve,deny}.rs`, `src/dream/mod.rs` (exports/dispatch), `tests/{dream_audit_broken,dream_graph,dream_identity}.rs`, `tests/fixtures/dream/{good,broken}/**` | 3 scaffold | `cargo test --test dream_audit_broken --test dream_graph --test dream_identity` |
| 5 Models | §8 models, controls, nonvacuity and source inventories | `tests/formal/{models,export}.rs` (export only if needed), `tests/formal_nonvacuity.rs`, `verification/{corpus,coverage,contracts}.toml` | —; final source conformance refreshed after implementing WPs | `make test-formal && make test-formal-parity` |
| 6 Synthesis | D1 prompt/parse and Pending/Proposed events; reuse WP4 validators | `src/dream/{synth,prompt}.rs`, `src/dream/mod.rs` (dispatch), `tests/dream_pipeline.rs` (synth cases), `tests/support/dream.rs`, `tests/fixtures/dream/synth/**` | 3, 4 | `cargo test --test dream_pipeline synth` |
| 7 Trial | worktree/fork/continue, per-pair verdicts, trial policy | `src/dream/trial.rs`, `src/dream/mod.rs` (dispatch), `src/policy.rs` (trial profile), `src/frontends/headless.rs` (trial continuation wiring only), `tests/dream_pipeline.rs` (trial cases), `tests/support/dream.rs`, `tests/fixtures/dream/trial/**` | 2, 6 | `cargo test --test dream_pipeline trial && cargo test --test policy_gate` |
| 8 Flush + sinks + Recall | MCP write/commit outcomes; immutable lower; per-context injection | `src/dream/{flush,lower,recall}.rs`, `src/dream/mod.rs` (dispatch), `src/engine/mod.rs` (request context/start Recall), `src/backend/acp_agent.rs` (first prompt/start Recall), `src/import/{mod,edge}.rs` (cited/inferred Recall), `tests/{dream_pipeline,dream_sinks,dream_recall}.rs`, `tests/support/dream.rs`, `tests/fixtures/dream/{sinks,recall}/**` | 4, 7 | `cargo test --features acp-agent --test dream_pipeline --test dream_sinks --test dream_recall && make test` |
| 9 Triggers + integration | run/claims/quota, private night-run, canonical audit/smoke wiring | `src/dream/{run,claims,quota}.rs`, `src/dream/mod.rs` (dispatch), `scripts/dream-night.sh`, `scripts/test.py`, `Makefile` (dream audit/smoke targets), `tests/{dream_run,dream_smoke}.rs`, `tests/support/dream.rs`, `tests/fixtures/dream/{quota,run}/**`, `verification/{corpus,coverage,contracts}.toml` (current-source mapping only) | 8, 5 | `cargo test --test dream_run --test dream_smoke && make test && make test-formal-parity` |

The table proposes scope amendments; only the delegating Oracle's task amendment authorizes a changed Executor path. Every WP also permits its own `.bboxy/reports/` record. File ownership is sequential along dependencies: WP2 then WP3 then WP7/WP8 for frontend hooks, WP3 then WP4/WP6/WP7/WP8/WP9 for `dream/mod.rs`, and WP6 → WP7 → WP8 → WP9 for shared fixture helpers. No package owns the blanket `src/dream/**`. WP4 owns deny/projection/identity once, so later WPs consume one validator rather than duplicating it.

WP9 owns canonical `make test` wiring of good/broken audit fixtures and scripted smoke, with private HOME, no operator credentials/seat, no prompts/network and assertions that those commands actually ran. Existing cargo all-targets tests are reused, not mirrored. WP5 owns model definitions/nonvacuity; WP9 refreshes inventories against the final implementation hashes, then verifies parity and retained current-source evidence after rebase. `check_transcript` is not a checked-in shell target on this tree; each report must name the actual checker/invariant used, not invent a green command.


Every package also keeps `make test`, `check_transcript` and `make test-formal` green, signs every commit, and ships its report under `.bboxy/reports/`.

## 11. Interface items outside this repo ([[dream-sinks]] §6)

- **T1:** the term index resolves `[[kb:<root>/<slug>]]` across mounted roots.
- **T2:** none. Flush uses the existing seat MCP `knowledge_write`/`knowledge_commit`.
- **T3:** `usage.json` lands on term main.
- **T4:** the system root `~/.agents/.bboxy` is mounted as a knowledge root.

Until T4, system-scope candidates stay `Pending` in the log. That costs nothing, because nothing blocks on them.


## 12. Baseline refinements and companion-file handoffs (task #5984)

### 12.1 Lifecycle and legitimate memory edits (mirror to memory-graph §4)


The authoritative input is the union of durable dream envelopes, deduplicated by `(session, seq)`, with their exact byte digests checked. Merge is set union, never arrival-order overwrite. Validate each transition and its pinned trial before projecting it.

- `Proposed` has `harness: None` and establishes an entry revision.
- `Tested`, `Adopted`, `Rejected` and `Retired` require `harness: Some(h)`; they affect only `(entry, h)`. `Adopted` requires a resolving `Trial(Pass)` for that same entry and harness under a known thresholds revision. `Inconclusive` leaves that pair `Tested`; it is not a rejection.
- `Superseded` and `Expired` have `harness: None` and retire that entry revision for every harness. They dominate all per-harness states, including a delayed adoption.
- Within a pair, valid states advance `proposed < tested < adopted < terminal`. `Rejected` and `Retired` are terminal. Preserve all terminal reasons; for display only, use the fixed order `superseded > expired > retired > rejected`. Wall clocks and log arrival order never reopen a terminal entry.
- A failed re-trial cannot erase another harness's pass. Two contradictory results for one pair converge to the terminal state; a new trial after that needs a new revision.

The file's `harnesses` is the sorted set of currently adopted pairs. Its scalar `status` is a summary: global terminal first; otherwise `adopted` if that set is nonempty, then `tested` if any pair is tested, then the fixed per-harness terminal display order if any pair is terminal, otherwise `proposed`. This summary never authorizes a sink; the pair and its pins do.

Projection metadata is derived, not permission to rewrite a hand-edited body. A stale metadata projection is reported by P9 and can be refreshed through the KB writer's etag contract; a changed body is preserved as a new proposal, not silently restored. A record unavailable at its pinned revision is omitted from recall. P9 distinguishes these cases.

**Re-proposing after a terminal state** creates a new entry. Its EntryId includes `revival: n` and the terminal predecessor in `derived_from` ([[dream-log]] §2). Concurrent identical proposals deduplicate; A → B → A cannot reuse the original adoption or a stale completion.


Hand-written memories retain signed authorship and evidence. Editing identity fields creates a new proposal/EntryId and preserves the old signed revision; no previous pass transfers to changed content. Audit/lower must not overwrite an edit as a repair. A changed byte digest makes that materialization ineligible; stale derived metadata is reported separately. Every origin still passes the same trust, deny-scan, scope and trial gates.

### 12.2 Sink publication and low trust (mirror to dream-sinks v0.2)

**Owners:** Side A is this repo's dream lower/recall implementation; Nyx Copperwing owns its integration. Side B is the native loader vendor, except engine/ACP, which this repo owns. Baseline is host behavior and explicit degradation, not a new guarantee by a vendor. Nyx approved no-clobber degradation in message `01M6J7SJKKNWKK66SAGA9N3DTE`.

- A digest recheck followed by overwrite-rename is not CAS: a hand writer can commit between them. An advisory dreamer-only lock does not protect an editor that does not take it. Delete that algorithm and its unconditional no-lost-update claim.
- Generate into a private, unique temporary path, fsync complete bytes, then publish with an atomic **no-replace** primitive on that filesystem (for example link-to-absent-target, then remove only the publisher's temporary name). Fsync the parent on success. Exclusive creation followed by streaming bytes is not atomic publication. Existing same pinned bytes are an idempotent no-op; differing bytes, symlinks, ambiguous markers, prefix collisions or unavailable safe primitives skip that target with a Notice. Never overwrite/unlink/chmod an existing user-editable target merely because its name starts `dream-` or it has a marker.
- Changed ownership is a legitimate hand edit: preserve it, stop managing that path, label drift, and use a fresh immutable publication name when the loader can consume it. A shared-file update is allowed only with a real, verified cooperative compare-and-replace contract covering **every** writer. No such native contract is established here.
- Only after actual loader-reachable publication append one `Adoption` per selected entry with its committed RecordPin and common block digest (the grouped records identify the selected set). A temporary artifact, dry-run diff or unlinked topic is not adoption. Crash before this record leaves an unclaimed artifact; retry may verify identical bytes, then record the existing publication once. An old artifact never authorizes a currently terminal entry.
- Preserve all hand bytes outside generated content. P2 tests the dreamer's effects and pinned generated bytes; it must not demand that hand content remain equal across runs. The selected set is the ranked **budget-fitting subset**, not every adopted entry.
- **Claude:** keep the existing topic/index format and limits, but publish topics before an index. An absent MEMORY.md can be published no-replace and then counted only if it links all selected topics within the existing line/byte limits. A pre-existing shared MEMORY.md stays untouched without cooperative CAS; report that sink skipped, with no Adoption for unlinked topics. Markers alone do not authorize a rewrite or removal.
- **Codex/Grok:** retain useful lowering through their exclusive resource/inbox names. Publish each entry revision once, no-replace; compare the full EntryId/digest when a 12-hex name exists, never trust the prefix alone. The Codex adapter's fixed instructions text is source-owned, not synthesized; create it only at an absent path or validate the exact known version. Changed instructions/resources or unknown native schema skip that target. Grok de-duplicates against inbox/archive by full job_id. No automatic writes to vendor-generated summaries/topics or instruction files.
- Retiring a native entry does not prove that a vendor forgot a consolidated copy. Never blindly unlink a possibly hand-edited resource. Label such retained native exposure stale/unknown; a separate no-replace retirement notice may communicate it, but cannot establish forgetting. Native inferred Recall is possible exposure only. P2 checks publication-at-time and the host's effects; it does not falsely certify the vendor's current active set.
- **Engine/ACP:** consume versioned immutable context artifacts under `dream/context/<repo-key-hash>/<publication>.md`, within the existing block budget. Select only current resolving pairs from the per-repo projection; a missing/stale projection/root/digest omits memories and never holds the live request for a global scan. The fixed system/developer prompt stays unchanged. Engine uses a separate quoted user/reference message before the actual request; ACP uses quoted data after handoff and before that request, not a config/system update. Escape wrapper tags and deny-scan all rendered fields. Record durable `Recall{reported,record,block_digest}` only for the exact block included; if recording fails, omit the memory.
- Every basis remains low-trust. `reported` proves insertion only; `cited` proves a matching resource citation only; `inferred` is uncertainty. No resolvable publication timeline means no inferred Recall. All bases exclude exposed sessions from evidence/trials; only reported/cited accepted sessions add recall strength.

**Real witnesses:** hand edit/append both before and after the former digest recheck; race two no-replace publishers; prefix collision; edited generated file; shared Claude index skip; new Claude index publication; Codex/Grok resource publication; crash before/after publication record; actual engine/ACP outbound prompt roles/bytes and one Recall per inserted entry. All use private HOME and scripted providers. A stale-read overwrite-rename implementation must fail the same hand-edit witness. Dry-sinks tests rendering only and emits no Adoption.

### 12.3 T3 producer evidence and consumer units (mirror to dream-sinks §6)

Read-only source pin: term `origin/main` resolved to `b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8`; the earlier `codex-oracle-records-20261103` checkout at `3a691536108464e3fac78456f5ebc47b6077a1a8` has the same relevant producer mechanism and was clean for these files.

- `crates/bboxy450-usage/src/lib.rs:121-125` — WindowReading.used is u8, projected is optional u16, reset is optional signed seconds.
- `lib.rs:129-132` — Reading explicitly documents percent used/pace.
- `lib.rs:145-152` — window percent is rounded directly; reset is cast to integer seconds.
- `crates/bboxy450-term/src/usage_strip.rs:82-92` — used/projected/windows serialize directly; no divide-by-100 occurs.
- `usage_strip.rs:75-80` — refresh failure retains old numbers and sets stale=true.
- `usage_strip.rs:98` — sampled_at is Unix seconds for a snapshot update, not a per-provider observation timestamp.
- `usage_strip.rs:136-148` — producer witness writes 96 and asserts JSON used==96.

**T3 consumer contract:** provider/window used is integer percent 0..100; projected is percent pace and may exceed 100; resets_at and sampled_at are Unix seconds, nullable resets disable that quota window. Validate before division and compute used_fraction=used/100.0 once. Require stale=false and valid snapshot age; a fresh machine event does not prove a fresh provider measurement. No usable window, absent file, unknown shape, future sample timestamp or invalid value disables the quota trigger without blocking D0 or conversations. The 0.80/0.90 fractional gates are unchanged. Boundary fixtures must include percent values 79/80/89/90, stale retained numbers, null reset, missing provider, malformed input and future/old sample times.

**Verification commands actually read the pinned git objects**, not an agent's usage report: `git -C /Users/feral/mydev/bboxy450-term show b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8:crates/bboxy450-usage/src/lib.rs` and the same command for `crates/bboxy450-term/src/usage_strip.rs`. Both exited 0. This proves source units; it is not an unattended consumer integration run or a claim that the live usage file exists.

### 12.4 T1 and T4 owner handoffs (no outside-repository writes)

**Routing owner:** Nyx hands these to the term knowledge/workspace Oracle. Read-only seat_get identified current term **Codex Oracle**, seat `01M6HF0S1BAJKHWT2N426W7328`, and **Vesper Greywind**, seat `01M6GVBAY1HW6P67GGDZM9XEBY`, both open in term swarm `01M6CQARWH5A79TE3H4NNWJ4XG`. The former delegates the term recovery/write-route task. Metadata establishes a route, not a new grant or an assignment. Cross-swarm send was refused for Nyx; its operator/inbox handoff must be consumed before claiming an assignee. These items have component owners, but no assigned implementer is asserted.

| Item | Producer/owner and exact boundary | Requested delivery and witness | Harness behavior until delivery |
|---|---|---|---|
| T1 | term `bbt-mux` Index + bundled knowledge provider; pinned term source `index.rs:152-180` parses literal weighted wikilinks and `knowledge/bboxy.rs:424-455` exports the root-local graph | Resolve `kb:<root-name>/<slug>` using the **authorized mounted-root map**, preserving relation/weight and target root identity. Repo keys contain slashes: match a complete authorized root-name prefix, never split at the first slash or interpret as an arbitrary path. Unmounted/denied/missing targets remain labelled external nodes; resolving never grants content access. Test duplicate slugs across repo/agents roots, slash-containing repo keys, unmounted/denied links and unchanged weights. Add source-mapped formal control that wrongly resolves into another root, plus a real provider/API witness | keep literal external edges; dream graph resolves only permitted already-mounted roots, with no filesystem shortcut |
| T4 | term `bbt-mux::knowledge::{workspaces,routing}` and authoritative workspace registry/settings; `workspaces.rs:46-50` captures one root/revision, `:440-486` binds and validates the selected root | Mount repo first and `~/.agents/.bboxy` under name agents without changing captured task/write authority. Separate explicit root-scoped write admission is needed before flush can target agents; a mount alone is not permission and today's knowledge_write has no root parameter. Missing/slow agents cannot stall repo reads, writes or messages. Test temporary agents/repo roots, unavailable/slow agents, explicit write denial, restart and A→B→A revision fencing. Extend existing workspace lifecycle models with a stale/authority-widening broken control and real routed tool witness | system drafts stay Pending; no shell writes or silent fallback into the repo root |

Suggested term verification after a scoped term brief: `cargo test -p bbt-mux --test storage_bus_index --test knowledge --test knowledge_provider --test workspace_roots`, the affected routed-MCP process witness and canonical formal/parity targets. These are owner handoff requirements, **not commands run by this task**. Scope and policy checks remain per root; neither item introduces one global mount queue or resolver lock.

### 12.5 log-ir-v1 §4c mirror edit awaiting index repair

Origin was read exactly from harness `origin/main` at `6957019f8e1d9c03364cab7f0b1efae0386c5a8c`, blob `25b85ebbb44a823dbfe250c068fff85ed47476c6`, id `01M6F4R15R4FFCRBCT162Q7CDQ` / number 5973. Original ID import through knowledge_write was refused; Nyx's checkout reconciliation supplied the original file, whose host lookup is now refused by number_conflict. No replacement identity was created.

After the host repairs that conflict, insert this subsection before §5 and add the dated change-log line below. This recorded text is not evidence that the target file references dream-log.

### 4c. v1.3 dreaming and memory provenance

The field/type contract is [[dream-log]] §2. It adds optional `SessionMeta.vcs` and `SessionMeta.dream`, plus durable `MemoryWrite`, `MemoryState`, `Trial`, `Adoption` and `Recall` items and their matching ItemKinds. Every new union preserves Unknown(raw); every added wire field is optional/defaulted, including fields inside new variants. Authored checks require valid pins and forbid Unknown.

`EvidenceRef` pins exact envelope-line bytes or committed KB-record bytes; `RecordPin` pins the memory revision. `Adoption` is successful sink materialization, separately from per-harness lifecycle adoption; `Recall` records reported insertion, cited reference, or inferred exposure, never model use or instruction authority. The low-trust, per-harness, revival and no-clobber semantics are in [[dream-log]] §§2, 7, 12 and [[memory-graph]]. Threshold numbers are interpreted under the Trial's recorded revision.

New frozen corpus files are `v1.3-dream-session.jsonl`, `v1.3-recall.jsonl` and `v1.3-vcs.jsonl`. Run `make lock-ir`, `make test-ir` and `cargo test --test protocol_golden --test ir_v1_corpus`; old corpus bytes are unchanged. Stateful contracts also require the models, broken controls and real witnesses in [[dream-log]] §§8–9.


Change-log line: 2026-11-03, Lumen Foxglove <ARCHITECT>, task #5984: §4c references the IR 1.3 dreaming/memory provenance contract in [[dream-log]] §2; defaults, revision pins and low-trust Recall semantics remain additive.

## Change log

- 2026-11-03, Lumen Foxglove <ARCHITECT>, task #5984: no-clobber publication/P2, per-harness projection, explicit low-trust Recall, percent-unit source evidence, term owner handoffs, exact WP/test/wiring ownership, model nonvacuity and witnesses. thresholds@2 records only the scheduling change; all numeric score/spend/timeout gates are retained. Companion mirror writes are held by proven number_conflict, not claimed complete.

- 2026-11-03, Vesper Greywind <ARCHITECT>: rewritten to implementation-ready from code evidence (§1). Memory content moved to KB records (d5974, [[memory-graph]]). Added thresholds@1, the deny scan, audit P1-P9, three models, the tests, and WP1-9.
- 2026-11-03, Vesper Greywind <ARCHITECT>: first draft.
