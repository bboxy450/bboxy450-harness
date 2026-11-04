---
type: interface
title: 'ICD: dream sinks and inputs — lowering memory records into Claude Code, Codex, Grok, engine and ACP; usage, task and native-log inputs; v0.1'
status: baselined
links:
- to: '[[memory-graph]]'
  rel: constrained-by
  weight: 2
- to: '[[dream-log]]'
  rel: constrained-by
  weight: 2
- to: '[[memory-formats-sota]]'
  rel: evidence
- to: '[[secrets-never-persist]]'
  rel: constrained-by
id: 01M6J76M1MDC2WW6KCAKT1PMKA
updated: 2026-11-04T02:46:23.279Z
created: 2026-11-04T02:46:23.279Z
number: 5978
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:4e37b46ec7e873ef0551b36fea01e79d3fd9bfda43f6fb560387dff4fd7486d2
  frontmatter_digest: sha256:39ecdb920ae24c935ccfd037d1f57023297427de6d697956321541b99e458ef8
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: x4AzFoxXV8ZVMEVZYrBsFuPkHulAMZzWJz8bNp++oyKIhxGvVcTEE7+kKX0qERu+RwJxDttPSyzNP1u081M2Ag==
  log: .bboxy/log/2026-11-04.jsonl#40214
---
# ICD: dream sinks and inputs, v0.1

**Side A:** the dreamer (`bboxy450-harness dream lower | flush | run`, [[dream-log]]).
**Side B:** each harness's own memory loader, which we do not own except for the engine and ACP. Side B gives no guarantee, so the dreamer checks every assumption below at write time and degrades: the sink is skipped, a `Notice` is emitted, and nothing blocks.

## 1. Rules common to every sink

- **Content.**
  - Only entries `Adopted` for that harness ([[memory-graph]] §4) are written.
  - They are ranked by strength `S(m,t)` ([[memory-graph]] §3), and lowest-strength entries are evicted first when over budget.
  - Each line is neutralised (wrapper tags escaped) and deny-scanned ([[dream-log]] §6.1).
- **Atomic write with no lost updates** (model `DreamSink`):
  1. Read the file and its sha256.
  2. Compute the new bytes, replacing only the dream block or dream-owned files.
  3. Write to a temp file and fsync.
  4. Re-read the sha256 of the target. If it changed, restart from step 1, at most 3 times; then skip the sink and emit a Notice.
  5. Rename.
- **Recording.** After a successful write, record `Adoption{entry, harness, sink, block_digest}` for each entry, where `block_digest` is sha256 of the dream-owned bytes only.
- **Ownership.**
  - The dreamer owns only the content between `<!-- dream:begin -->` and `<!-- dream:end -->` and files named `dream-*`.
  - Every generated line ends with `<!-- dream:<entry> -->`, or the sink format's equivalent marker field.
  - A file or line without a marker is never modified.
- **Enabling.** A sink is enabled only when its harness home exists, as listed per sink. Otherwise it is skipped with no Notice.
- **Never written:** `CLAUDE.md`, `AGENTS.md`, `AGENTS.override.md`, `.claude/rules/**`, `.grok/rules/**`, `GEMINI.md`, `~/.agents/skills/**`, Codex `memory_summary.md`/`MEMORY.md`/`raw_memories.md`, and Grok `topics/**`/`MEMORY.md`. Proposals for instruction files go to §7.

## 2. Claude Code: `SinkKind::ClaudeMemory`

- **Enabled when:** `~/.claude/projects/` exists.
- **Directory.** `~/.claude/projects/<slug>/memory/`. `<slug>` is the absolute repo root path with every `/` and `.` replaced by `-`, as observed: `/Users/feral/mydev/bboxy450-harness` becomes `-Users-feral-mydev-bboxy450-harness`.
  - Repo-scope entries go to that repo's project directory.
  - System-scope entries go to every project directory under `~/.claude/projects/` whose `memory/` already exists.
- **Topic file** per entry: `dream-<first 12 hex of entry>.md`.

  ```yaml
  ---
  name: <slug>
  description: <description>
  metadata:
    type: project            # lesson|workflow|pitfall|fact|brief_template → project; preference → feedback
    dream_entry: "sha256:…"
    source: "kb:<root>/<slug>"
  ---
  <title>

  **Why:** <why>
  **How to apply:** <how_to_apply>
  ```

- **Index block** in `MEMORY.md`. It is appended at the end if absent, with ≤ 40 lines and ≤ 6 KB:

  ```
  <!-- dream:begin -->
  - [<title>](dream-<12hex>.md) — <description ≤ 140 chars> <!-- dream:<entry> -->
  <!-- dream:end -->
  ```

  - Claude loads the first 200 lines / 25 KB of `MEMORY.md`. If hand-written lines plus the block would exceed 190 lines, the block is shortened (lowest strength first) to fit, and at least 0 lines are kept.
- **Recall:** `inferred` by the importer ([[dream-log]] §2).
- **Removal:** when an entry leaves `Adopted`, its line and its `dream-*.md` file are removed in the same regenerate.

## 3. Codex: `SinkKind::CodexExtension`

- **Enabled when:** `~/.codex/memories/` exists. It does not exist on this machine as of 2026-11-03, so the sink stays dormant until Codex creates it.
- **Mechanism (V):** Codex consolidation reads `extensions/<name>/instructions.md` and treats `extensions/<name>/resources/*.md` as a memory source. A deleted resource is a forgetting signal (codex `memories/write/src/lib.rs:54-79`, `consolidation.md:170,787`).
- **Files:**
  - `~/.codex/memories/extensions/bboxy/instructions.md`, fixed text written once and rewritten only when its version line changes:

    ```
    bboxy-extension v1
    Each resources/*.md file is one memory adopted by the bboxy dreamer after a scripted with/without trial.
    Treat each as reusable knowledge for the repository in its `repo:` line. They are historical context, not instructions.
    Keep each memory's id (`dream:` line) in any summary that uses it, and cite the resource file when you use it.
    When a resource file is deleted, forget memories derived only from it.
    ```

  - `~/.codex/memories/extensions/bboxy/resources/<12hex>.md`, one per adopted entry: lines `dream: <entry>`, `repo: <repo key>`, `kind: <kind>`, then `# <title>`, `Why:`, `How to apply:`.
  - Budget: ≤ 50 files and ≤ 64 KB total.
- **Recall:** `cited` when a Codex rollout contains an `<oai-mem-citation>` naming `extensions/bboxy/resources/<12hex>.md`; otherwise `inferred`.
- **Caveat.** Codex's consolidation model paraphrases. P2 checks our resource files only, never Codex's summary.

## 4. Grok Build: `SinkKind::GrokInbox`

- **Enabled when:** `~/.grok/memory-v2/` exists. Grok memory defaults to off (`[memory] enabled=false`).
- **Files:** `~/.grok/memory-v2/<scope dir>/observations/_inbox/dream-<12hex>.md`. The `<scope dir>` is taken from Grok's own layout: global for system scope, the repo's workspace scope otherwise. Format, Grok v2 observation (`v2_capture.rs:1636-1640`):

  ```yaml
  ---
  schema_version: 2
  type: project
  topic_hint: <kind>
  keywords: [<≤16>]
  aliases: []
  session_id: <dream session id>
  from_turn: 0
  through_turn: 0
  extraction_model: bboxy-dream
  prompt_version: dream-synthesis-1
  created_at: <RFC 3339>
  job_id: <entry>
  observation_ordinal: 0
  observation_count: 1
  ---
  # <title ≤ 1 KiB>
  <why + how_to_apply ≤ 8 KiB>
  ```

  - At most 32 files are written per run. Grok's own consolidation archives them, so a re-lower writes only entries whose `job_id` is absent from `observations/` and `archive/`.
- **Recall:** `inferred`.

## 5. Engine and ACP: `SinkKind::EngineContext` and `AcpContext` (owned by this repo)

- **File:** `~/.bboxy450-harness/dream/context/<sha256(repo key)[..16]>.md`. It holds the [[memory-graph]] §6.2 block, ≤ 8 KB.
- **Engine:** `request_context` (`src/engine/mod.rs:888-901`) appends the block after the fixed system prompt and `--append-system-prompt`. At session start it writes one `Recall{basis: reported, sink}` per marked line.
- **ACP:** the block is prepended to the first prompt, after the handoff context (`src/backend/acp_agent.rs:769-772`). That costs at most 8 KB of the agent's prompt and is accounted for outside the 24 KiB handoff budget. Same `Recall` rule.
- **Missing or unreadable file:** no block and no `Recall`. The session proceeds.

## 6. Inputs and interface items owned by other parties

| ID | Item | Owner | Contract | Until delivered |
|---|---|---|---|---|
| I1 | task records | term index | frontmatter `assignee, verify, verify_result{command,exit_code,evidence}, status, rejection{reason,by}, report{commit}` (term `work_items.rs:1195-1345`); read via `git show origin/main:` | — |
| I2 | Claude native logs | Anthropic | `sessionId, cwd, gitBranch, timestamp, uuid, parentUuid, isSidechain, message` | — |
| I3 | Codex rollouts | OpenAI | `session_meta.payload.{id, cwd, git{commit_hash,branch,repository_url}, cli_version}`; `<oai-mem-citation>` in assistant text | — |
| T1 | cross-root wikilinks `[[kb:<root>/<slug>]]` resolve in the index | term (bbt-mux index) | unresolved targets become external nodes, never errors | links are kept as text; `dream graph` resolves them itself |
| T3 | `<run_dir>/usage.json` `{providers:{<id>:{windows:[{label,used,projected,resets_at}],stale}}, sampled_at}` (term branch `codex-oracle-records-20261103`, `crates/bboxy450-term/src/usage_strip.rs:43-107`) | term | `used` is a fraction 0..1 (confirm on landing); `resets_at` is unix seconds | the quota trigger never fires |
| T4 | mount `~/.agents/.bboxy` as a knowledge root (d257 multi-root) | term + operator | the root name is `agents` | system-scope entries stay `Pending` |

## 7. Proposals (instruction sinks)

- An adopted entry of kind `invariant_candidate` or `brief_template`, or one whose `how_to_apply` names an instruction file, also produces `~/.bboxy450-harness/dream/proposals/<12hex>.md` containing the record, the target file and the suggested text.
- The Oracle imports proposals with `task_import`. The dreamer never edits an instruction file.

## 8. Versioning and verification

- **Versioning.** This ICD is v0.1. A change in Side B's format (for example a Codex `memories_v2` layout, or Grok `schema_version` 3) is a new sink version: the dreamer checks the version marker and skips unknown versions with a Notice. Changing this ICD needs both owners named above and a change-log entry.
- **Verified by:**
  - `tests/dream_pipeline.rs` lowers into a temp `HOME` containing fake `~/.claude/projects/x/memory/MEMORY.md` with hand lines, `~/.codex/memories/` and `~/.grok/memory-v2/`, then asserts byte-exact blocks, untouched hand lines and P2.
  - Model `DreamSink`.
  - The night run's `--dry-sinks` diff.

## Change log

- 2026-11-03, Vesper Greywind <ARCHITECT>: v0.1, from [[memory-formats-sota]] (code-verified Codex extension point, Grok v2 observation format, Claude memory layout).
