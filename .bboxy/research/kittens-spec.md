---
id: "0YMED29KM9PAPZ4BS5E5GMW8FE"
number: 16
type: research
title: "Kittens — design spec of prior work (research input for bboxy450-harness)"
created: 2026-10-31
---
# Kittens — design spec of prior work (research input for bboxy450-harness)

Source: private repo `Gonzih/kittens` (https://github.com/Gonzih/kittens), cloned 2026-10-31 at
`2d73edc` (0.3.0 + 2 record commits). 554 commits, 2026-10-02 → 2026-10-15. Paths below are
relative to that checkout. Evidence format: `file:line — mechanism`. Credentials were never read;
profile values are described by key name only.

Not kittens worktrees: `~/mydev/winnow-kittens-acp` and `~/mydev/winnow-kittens-issue` are
branches of `Gonzih/winnow` (consumer) — commits `cd1af3b`/`b5f7e4e` "bounded Kittens ACP
free-model probe", `30f13f6` "verified Nemotron ACP and MCP test results". They drive kittens over
ACP; no newer kittens source there.

Stated intent: `.kb/docs/spec.md:20` — "pi's simple agent loop, a Grok-inspired terminal interface,
and ACP. One repository, one crate, one binary". `.kb/docs/spec.md:26-32` — "embeddable,
controllable, scriptable swarm member … TUI is a thin adapter over those same events … yolo by
default … No permission popups, approval policy engine or sandbox."

## 1. Crate / module layout

- One crate, one binary, edition 2024, `Cargo.toml:1-5`. `src/main.rs:8-17` — `#[tokio::main]`,
  `cli::run()` owns exit code; stderr for diagnostics, stdout for machine output.
- `src/lib.rs:1-16` — library-first: entry points `agent::run_turn` (one prompt → one terminal)
  and `agent::serve` (persistent actor). 55 modules.
- Size (src incl. inline tests): 58.3k lines; integration tests 34.5k lines (`tests/*.rs`, 984
  `#[test]`/`#[tokio::test]` total). Clusters (approx): core loop/provider/tools/config 18.8k ·
  TUI 12.3k · adoption of external harnesses (Codex/Claude PTY panes, cron) 8.0k · swarm
  bus/graph/control/host/registry/socket/listen/project 8.7k · KB/plan/workgraph/semantic 3.8k ·
  extras (computer-use, Cloudflare search, JEV decisions, images, clipboard) 3.7k.
- Notable deps `Cargo.toml:13-56`: tokio full, reqwest+rustls+eventsource-stream (SSE),
  ratatui 0.29 + crossterm 0.28 (event-stream), `agent-client-protocol =2.2.0`, `rmcp 3.5`
  (child-process MCP client), rusqlite bundled, `fastembed` (local embeddings, ~heavy),
  portable-pty + vendored `vt100` fork (`vendor/vt100/KITTENS-PROVENANCE.md`), jsonschema.

## 2. Agent loop

Core is one sequential driver behind an in-memory bus; every mode (TUI, `-p`, `--json`, ACP,
`--listen`, socket) is an adapter over the same `Event` stream.

- Types: `src/types.rs:92-110` — `AgentInput` = `Profile|Prompt|ImagePrompt|Activate|Cancel{turn_id}`
  (serde tag `type`). `src/types.rs:209-245` — `Event` = `ProfileChanged|ThinkingDelta|TextDelta|
  ToolStart|ToolEnd|ToolChanges|TurnEnd|Ready|Error`. `src/types.rs:191-197` — `StopReason` =
  `EndTurn|Cancelled|MaxTokens|MaxTurnRequests` (maps 1:1 to ACP, `src/acp.rs:681-689`).
- Invariant "exactly one terminal (`TurnEnd` xor `Error`) per accepted prompt":
  `src/agent.rs:37-41` doc; enforced in `pinned_turn` `src/agent.rs:1823-1880` and `finalize`
  `src/agent.rs:132-203` (hook → session save → SQLite checkpoint → emit `TurnEnd`; any failure →
  one `Error`, no `TurnEnd`). Fix history shows double-emission bugs (`src/agent.rs:221-224`
  comment "Emitting here as well produced two").
- Turn lifecycle `drive` `src/agent.rs:206-313`: validate bus tool → compile output schema →
  build provider → image capability probe → connect MCP servers **per turn** (`:268`, "MCP
  servers live for the turn and are closed on every exit path") → `drive_inner` → MCP shutdown.
- Round loop `drive_inner` `src/agent.rs:316-833`:
  - push user msg `:330`; `max_turns` 0 = unlimited (`:343`, CLI default 0, `src/cli.rs` arg doc
    "Unlimited by default").
  - tool defs rebuilt per round `:360-384` (native + computer + web_search + McpSelect + selected
    MCP + bus/agent/adoption tools).
  - system prompt **rebuilt each round, never persisted** `:414-417`; assembled from
    instructions file + hard-coded guidance string `:418` + plan JSON `:440-445` + KB spec context
    `:446-468` + harness identity/registry metadata `:469-486` + activation context + runtime
    addresses `:519-535`. Each is a separate leading `system` message inserted at index
    `leading`.
  - streaming via a borrowed `FnMut` delta sink that awaits each bus emit (`:552-584`, "no lock,
    no channel, no spawn"). Text deltas accumulate so a cancelled turn reports what user saw.
  - stop dispatch `:643-668`: EndTurn → validate required tools / JSON schema; non-ToolCalls →
    return; ToolCalls → parse `pending (id,name,args)` `:711-733` (args kept as wire string in
    history, parsed to `Value` for dispatch).
  - after tools: `pair_remaining` `:1504-1516` appends `"tool call was not run"` for every
    unanswered id so history is always provider-valid, including on cancel/failure (`:754`).
- Parallel tools: `run_calls` `src/agent.rs:1113-1201` — contiguous runs of parallel-eligible
  calls (max 4, `:1149`) go through `futures_util::future::join_all`; anything else is serial.
  Eligibility `parallel_eligible` `:1071-1086`: `read`/`tree`, `web_search`, MCP tools with
  `readOnlyHint` (`src/mcp.rs:246`); plan/status/computer/McpSelect and all writes serial; any
  `before_tool`/`after_tool` hook forces serial (`:1144`, `src/hooks.rs:111`). Outputs retained
  in model call order before fallible delivery (`finish_parallel_calls` `:1204-1269`).
- Cancellation: `tokio_util::CancellationToken` threaded everywhere; "detected from the token,
  never by matching error text" `src/agent.rs:203-205`. `serve` gives each turn `cancel.child_token()`
  `:1667`; `Cancel{turn_id}` matches running envelope id or removes a queued prompt and reports it
  `Cancelled` (`accept` `:1784-1795`). On abort the turn is signalled then awaited, never dropped
  (`:1700-1707`). Tool processes run in their own process group and are group-killed
  (`src/tools.rs:1241-1273`).
- Serve loop `serve_inner` `src/agent.rs:1606-1737`: subscribe inbox → `Ready` → `VecDeque` queue,
  one foreground turn at a time; `tokio::select!` unbiased while running so a busy inbox can't
  starve the turn (`:1678-1680`); profile switch refused mid-turn (`:1770-1777`).
- Retries `src/retry.rs` + `src/provider.rs:335-410`: transient = 408/429/500/502/503/504
  (`retry.rs:26-28`); generation retries **forever until cancel** (`Retries::generation`
  persistent, `retry.rs:68-83`: 5s·2^n + jitter, cap 600s, honors longer Retry-After); search keeps
  4 retries / 120s budget (`retry.rs:24`). Retries only before any model output
  (`provider.rs:459` `!received_output`). Timeouts: connect 60s, read-stall 600s, no whole-attempt
  deadline (`provider.rs:34-35`).
- Context compaction: **none.** No summarisation/truncation of history anywhere (grep `compact|
  summariz|context_length` hits only output formatting). History grows unbounded; per-tool output
  capped at 50 KiB with spill-to-tempfile (`src/tools.rs:24,1197-1210`). `finish_reason:"length"`
  → `MaxTokens` (`provider.rs:544`).
- Output contracts (`--output-schema`, `--require-tool`): `src/agent.rs:386-411` narrows offered
  tools to the one required; `provider.rs:240-265` sends `tool_choice` named function /
  `response_format: json_schema strict` + `provider.require_parameters:true`; failures categorised
  via `contract_failure` `:835-855`.

## 3. Provider abstraction

- `src/capabilities.rs:53-95` — trait `Generation { supports_image_input, complete_with_options,
  complete }` with `GenerationDelta::{Answer,Thinking}` and `Stop::{EndTurn,ToolCalls,MaxTokens}`.
  Parallel traits `DecisionSelection` (`:111`) and `Search` (`:143`). Only one real `Generation`
  impl: `src/provider.rs:634` (OpenAI-compatible chat/completions SSE, OpenRouter-flavoured).
- Request body `src/provider.rs:238-241` — `stream:true`, `reasoning:{exclude:false}`,
  `stream_options.include_usage`, `n:1`. No prompt caching controls.
- Stream parse rules `src/provider.rs:1-17`: deltas streamed; tool-call fragments merged by index
  (`merge_fragment` `:748`, index ≤ 64 `:62`); duplicate ids rejected (`:793`); truncated or
  contradictory calls never executable; conflicting/unknown/missing `finish_reason` = error
  (`:541-566`); `reasoning_details` preserved verbatim for replay (`:687-720`); provider error text
  never echoed (credential reflection).
- Image capability from OpenRouter model catalog (`provider.rs:2781`).
- Diagnostics: every failed attempt + retry wait logged as `service_error`/`service_retry` on the
  causal bus journal (`src/diagnostics.rs`; observed in runtime logs, §11).
- Config resolution `src/config.rs:21-137`: profile (`--profile`/`KITTENS_PROFILE`/
  `~/.kittens/default.md` symlink, `:1936-1970`) else legacy env `OPENROUTER_API_KEY` →
  `~/.pi/agent/auth.json` (`:166-250`); model `--model` → `KITTENS_MODEL` → pi settings only if
  pi's defaultProvider is openrouter (`:433-488`).
- Profiles `src/config.rs:1737-1808`: `~/.kittens/profiles/NAME.md`, YAML frontmatter between
  `---` lines; keys `name description strengths tags provider model endpoint api_key|api_key_env`,
  optional blocks `decision{provider api_key_env …}`, `search{api_key_env account_id_env gateway_id
  byok_alias}`, `computer`, `output`. YAML errors never echoed (`:1768`). Endpoint must be http(s)
  with no userinfo/query (`:1784-1792`). Custom endpoints never inherit the OpenRouter key
  (`:1873-1875`).
- Key resolution: `api_key` value beginning `!` is a shell command (pi convention) run once via
  `sh -c` with 10s timeout, 8 KiB cap, own process group, cancellable (`src/config.rs:253-430`,
  applied `:1877-1881`). `Config` Debug redacts and Serialize omits `api_key` (`src/types.rs:58`,
  tests `:295-320`).
- Live profile switch: `AgentInput::Profile` staged-then-swapped atomically while idle
  (`src/agent.rs:1640-1660`).

## 4. Tool set and edit strategy

Schemas `src/tools.rs:37-150` (OpenAI function format, all `additionalProperties:false`):

| tool | args | mechanism |
|---|---|---|
| `read` | path, offset≥1, limit≥1 | UTF-8 file lines (default 2000, `:28`) or long dir listing incl. hidden (`:433`) |
| `write` | path, content | full overwrite, mkdir -p (`:545`) |
| `edit` | path, oldText, newText | exactly-one-match replace, else fail unchanged (`:616-713`) |
| `apply_patch` | edits[1..64]{path,oldText,newText} | all staged & checked before any write; no rollback on commit I/O error (`:715-920`) |
| `bash` | command, timeout (default 120s `:30`) | process group, 50 KiB cap, spill to temp file |
| `exec` | program, args[≤256], stdin, timeout | literal argv, no shell (`:135`) |
| `tree` | path, depth≤32, dirs_only… | sorted, no symlink follow (`:922`) |
| `plan` / `status` / `kb` | — | runtime-context tools (per-agent plan, grounding, Markdown KB) |
| `web_search` | query | Cloudflare AI-gateway search, optional (`src/web_search.rs`) |
| `McpSelect` | query | JEV/decision model picks one MCP tool from catalog ≤254 (`src/agent.rs:862-1048`); selected defs added next round |
| `agent`, adoption tools, `cron` | op-tagged | spawn/send/inspect child agents & PTY panes (`src/agent_tool.rs:70-80`) |

- No grep/glob tool: search is via `bash`.
- Every failure is a `ToolOutput{is_error:true}` the model sees, never a turn failure
  (`src/tools.rs:1-7`). `tool_recovery::finalize` appends corrective hints from actual offered
  definitions, ≤8 KiB, never executes suggestions (`src/tool_recovery.rs:13-45`).
- `ToolChanges` events carry bounded before/after file previews for UI diffs
  (`src/tool_preview.rs`, `src/agent.rs:1307-1318`).
- Hooks `src/hooks.rs:1-10`: `<cwd>/.kittens/config.json` `{"hooks":{"before_tool","after_tool",
  "turn_end"}}` argv lists; MCP in same file under `mcpServers` (`src/mcp.rs` header).

## 5. Session persistence and resume

- `src/session.rs:1-14` — session = raw OpenAI wire `messages[]`; file `{"version":1,"messages":[…]}`;
  atomic temp-sibling + rename, 0600 (`:133-168`); roles validated on load (`:21,170`).
- Exclusive writer lease: `flock` on sibling `.NAME.json.lock` (`:26-72`), canonicalised path.
- Default path `~/.kittens/sessions/<harness-uuid>.json` (`src/host.rs:228-234`).
- Duplicate store: full session JSON also written into SQLite `agents.wire` on every checkpoint
  (`src/registry.rs:446-456`), at `running` and `idle` transitions (`src/agent.rs:1665`, `:173`).
- Registry `~/.kittens/registry.sqlite3` tables: harnesses, agents, records, documents (+FTS5),
  assemblies, embeddings, plans, current_tasks (`src/registry.rs:165-175`). Each query opens a new
  connection on the blocking pool (`src/registry.rs:792-798`).
- Causal journal `~/.kittens/logs/<uuid>.jsonl` — every bus envelope `{sequence,timestamp,uuid,
  envelope{id,from,route,in_reply_to,correlation_id,payload}}` (`src/registry.rs:674+`).
- Resume `src/resume.rs` (`prepare(target)` → session + lease + cwd); `kittens resume`, TUI
  `/resume [UUID|PATH|--all|--last]`; adopted Codex conversations resumed by thread id
  (`src/adoption_resume.rs`, commit `334a25f`).

## 6. TUI

- `src/tui.rs:1` "Thin Ratatui input and activity view over the shared agent service." Subscribes
  to `agent/root` topic, `graph/status`, and the root inbox (`src/tui.rs:445-453`).
- Inline, not alt-screen: `src/tui/inline.rs:1` — "immutable rows flow once; only the live band is
  diffed"; history goes to native terminal scrollback (commit `7ceb592`). Fullscreen only for mux
  surface (`src/tui/mux/surface.rs`).
- One giant `tokio::select!` event loop `src/tui.rs:587-620` over: shutdown signal, mux frames
  (`watch`), mux completions (`mpsc`), redraw tick, dashboard/project/task `watch` channels,
  appearance (light/dark detection `src/tui/theme.rs`), bus events, stdin. Dirty flag + frame
  deadline for redraw throttling.
- Input: single stdin owner separating terminal appearance reports from keys
  (`src/tui/input.rs:1`); Emacs keys, `/` fuzzy picker, `:` Vim-style command mode with completion
  (`src/tui/mux/completion.rs`), Ctrl+V clipboard images, Ctrl+C cancel/exit (`src/tui.rs:1212`).
- Dashboard F2 (`src/tui/dashboard.rs`, read-only projections), current-task worker polling
  registry every 500 ms (`src/tui/current_task.rs:1`).
- Mux: PTY panes of other harnesses rendered via vendored vt100 (`src/tui/mux.rs`,
  `src/adoption_terminal.rs`).
- Code shape: `src/tui.rs` 5.4k lines with dense multi-statement one-liners (e.g. `:580-640`).

## 7. ACP integration

- `src/acp.rs:1-18` — ACP v1 over stdio via `agent-client-protocol` SDK; **no second loop**: each
  `session/new` starts a `Host` (`src/acp.rs:181`, member `acp-<shortid>`) and `session/prompt`
  publishes `AgentInput::Prompt` on its bus, then translates events (`run_prompt` `:600-679`).
- SDK dispatches serially, so init/prompt callbacks enqueue work to an app-owned `JoinSet` worker
  and return immediately, letting `cancel` arrive mid-turn (`:10-17`, `own_workers` `:403`).
- Event → `session/update`: thinking → `AgentThoughtChunk`, text → `AgentMessageChunk`,
  tool_start → `ToolCall{InProgress, raw_input}`, tool_end → `ToolCallUpdate` (`:691-740`); kind
  from tool name (`:741`).
- Capabilities advertised: `embedded_context` only (`:112-118`). Client MCP servers from
  `session/new` accepted (stdio only, `stdio_servers` `:505`). Not implemented: `session/load`,
  `request_permission`, client fs/terminal delegation, modes (grep empty).
- Open issue #53 "unexplained ACP broken-output cleanup test timeout".

## 8. System prompt, instructions, skills

- `src/instructions.rs:12-18` — instruction candidates `AGENTS.override.md`, `AGENTS.md`,
  `CLAUDE.md`, `.claude/CLAUDE.md`; scopes `.kittens .codex .agents .claude`; ancestor walk + global
  fallbacks (`~/.kittens`, `$CODEX_HOME`, `~/.codex`) (`:101-160`).
- Skills: recursive `SKILL.md` discovery with YAML `name/description`; roots per ancestor scope +
  `~/.kittens/skills`, `$CODEX_HOME/skills`, `~/.agents/skills`, `~/.claude/skills`
  (`:143-157`); only metadata listed, bodies read lazily with `read` (`:168`).
- Hard-coded guidance strings: `src/agent.rs:418`, `src/instructions.rs:10` (`ADOPTION_GUIDANCE`).
- Shipped skill `skills/kittens/SKILL.md` (CLI discovery/list/logs/search/send).

## 9. Subagents and swarm

- Bus `src/bus.rs:1-14`: one Tokio task owns id allocation + subscription table; commands +
  oneshot acks; exact routes `Member|Topic`, no wildcards (`:27-36`); per-subscriber
  `mpsc::unbounded_channel` (`:366`). In-memory only.
- Graph `src/graph.rs` — named async node tasks, state published to `graph/status`.
- Controller `src/control.rs:64-120` — versioned endpoint `feral/control`: ops
  `Mux|Start|Emit|Profiles|List|Stop`; `NodeSpec` = `Agent{cwd,specialization,profile}|Source{cmd→
  route}|Command|Router…`. "an emitted message is queued work, not evidence that its recipient
  completed it" (`:1-5`).
- Model-facing `agent` tool: `spawn{name,instructions,cwd,profile,prompt}|send|inspect`
  (`src/agent_tool.rs:70-80`). Children are full `serve` actors on the same bus.
- Host `src/host.rs:21-40` owns config (`Arc<RwLock<Config>>`), bus, graph, controller, adoption
  manager, journal, socket, session lease.
- Cross-process: per-harness UUID + Unix socket `/tmp/kittens-sockets/UUID.sock`
  (`skills/kittens/SKILL.md`), `kittens send UUID -p`, `--listen` JSONL control (`src/listen.rs`).
- Adoption (`src/adoption*.rs`, ~8k lines): runs Codex (app-server protocol) and Claude Code in
  owned PTY panes with forced full permissions (commit `9c47da8`), messaging, cron.
- Routing/JEV (`src/routing.rs`, `src/jev.rs`): external finite-choice classifier picks a
  destination/tool.
- Project/workgraph (`src/project.rs`, `src/knowledge.rs`, `src/kb.rs`): `.kb` Markdown tasks with
  named invariant checks gating dependencies; "A model terminal is a claim. Only the existing bash
  runner's observed result verifies a task." (`src/project.rs` header).

## 10. Async runtime and concurrency

- Single multi-thread Tokio runtime shared by every mode (`src/main.rs:8`, README "shared async
  runtime").
- Patterns: actor-ish single-writer tasks (bus router, registry journal worker, current-task
  worker); `CancellationToken` trees; `watch` channels for UI projections; `JoinSet` for ACP
  workers; `spawn_blocking` for SQLite; `join_all` for parallel tools.
- Unbounded channels on the bus — no backpressure; thinking deltas dominate traffic (§11).
- Shutdown: SIGINT/SIGTERM via `src/shutdown.rs:2-9`; Host shutdown joins Graph before Bus
  (`src/control.rs:3`).

## 11. Local runtime layout (`~/.kittens`, structure only)

- `default.md` → symlink to `profiles/minimax-flash.md`.
- `profiles/*.md` (5: ling-flash, minimax-flash, nemotron, nemotron-super-vl,
  tool-recovery-executor): frontmatter keys as §3; `api_key` holds a `!`-prefixed shell command
  that fetches the key (values not read).
- `sessions/<uuid>.json` + `.<uuid>.json.lock` (23 sessions); shape `{version:int,
  messages:[{role,content,…}]}`.
- `logs/<uuid>.jsonl` (41 files, one up to 3.9 MB): in that file 13,117 envelopes of which 10,489
  `thinking_delta`, 2,167 `text_delta`, 107 tool_start/end, 32 service_error, 31 service_retry —
  per-token journaling ≈ 96% of log volume.
- `registry.sqlite3` (tables §5), `skills/` empty, `backups/` (pre-migration copies).

## 12. Tests and verification

- 984 test fns; integration suites per subsystem (`tests/agent.rs`, `tests/acp.rs` 3.6k,
  `tests/tui.rs` 3.1k, `tests/cli.rs` 3.9k…). Mock providers: `wiremock` and Python fixture
  servers (`tests/fixtures/openrouter_server.py`, `mcp_server.py`, `codex_mock.py`). Bats smoke
  `tests/smoke.bats`. Python dogfood harness `scripts/dogfood_run.py` against free models.
- CI `.github/workflows/ci.yml`: ubuntu+macos, fmt, clippy `-D warnings`, test, bats. Hosted CI
  blocked by billing (open issue #52) → verification was local runs recorded in `.kb`.
- Formal verification: **none** (no TLA+/Yarn/loom/proptest; grep empty). Invariants are prose in
  `.kb/specs/*.md` (40+ specs) plus example tests; `kb workgraph` gates on named shell checks.

## 13. History and issues — themes

- Cadence: 377 of 554 commits in the first two days (10-02/10-03), 106 on 10-07, then tapering.
  Subjects: 142 `docs:`, 83 `fix:`, 43 `Record …` (evidence-only commits after each PR). No
  reverts.
- Fix themes (`git log` subjects): cancellation dominates (16 fix subjects mention cancel: startup
  cancel, MCP startup budget, drain futures, bash deadline through pipe drain, hook before cancel),
  then provider/tool/hook pairing.
- Incidents (`.kb/incidents/`, 17): `generation-retry-exhaustion.md` (4-retry cap killed turns in
  real outages → persistent retry), `issue-184-tool-routing.md` (`parallel_tool_calls:false` +
  `require_parameters` made OpenRouter 404 on free Nemotron), `issue-181` (free model malformed
  JSON/skipped required MCP), `102-free-provider-quota` / `slash-dogfood-quota` (free-route
  15-request quota exhausted during orientation), `session-lease-fork-release`,
  `session-mux-quit-order`, `iterm-mouse-input` (#151 mouse escapes leaked into composer),
  `slash-build-storage` (#106 disk exhaustion from per-task build dirs).
- Open: #210 macOS adopted-pane shutdown EPERM; #53 ACP cleanup timeout; #52 CI billing.

## 14. What worked / what hurt / what to keep / what to drop

What worked (evidence):
- One loop, many adapters: ACP/TUI/JSONL/socket all observe the same `Event` stream
  (`src/acp.rs:1-6`, `src/tui.rs:445-453`); ACP needed only ~750 lines.
- Terminal-exactly-once + history pairing (`src/agent.rs:1504`, `finalize`) — history never
  becomes provider-invalid after cancel/failure.
- Token-based cancellation with child tokens and process-group kill; signal-then-await instead of
  drop (`src/agent.rs:1700-1707`).
- Strict stream assembly (finish_reason contradiction checks, args kept as wire string,
  reasoning_details replayed) — free/odd OpenRouter models stayed usable.
- Persistent pre-output retry with logged attempts (incident `generation-retry-exhaustion`).
- Exact-unique `edit` + staged `apply_patch`; tool errors as model-visible output with recovery
  hints; 50 KiB cap + spill file.
- Parallel read-only batches with order-preserving results; hooks force serial.
- Inline TUI using native scrollback; secret hygiene (redacted Debug, no echo of provider/YAML
  errors, 0600 atomic writes, flock lease).

What hurt:
- Scope explosion: in 13 days the crate grew to 58k src lines — adoption of Codex/Claude PTYs
  (8k), KB/workgraph/provenance/semantic embeddings (3.8k + fastembed), JEV decision routing,
  computer-use, Cloudflare search. The "Pi simplicity" goal was lost.
- System prompt rebuilt every round from filesystem + KB + registry queries (`src/agent.rs:414-535`)
  — per-round I/O, non-cacheable prefix, prompt varies mid-turn.
- MCP servers spawned and reaped every turn (`src/agent.rs:268`); `McpSelect` adds an extra
  model hop plus an external decision provider.
- No context compaction; unlimited turns by default → long sessions eventually overflow.
- Double persistence (JSON file + SQLite `wire` column, full clone each checkpoint) and a new
  SQLite connection per query.
- Journal records every thinking/text delta (≈96% of a 3.9 MB log).
- Unbounded bus channels; no backpressure.
- Process overhead: 142 docs + 43 "Record" commits vs 83 fixes; `.kb` evidence prose with
  exit codes and hashes; hosted CI never ran (#52).
- Free-model dogfooding drove features around provider quirks (`require_parameters`, quotas)
  rather than core quality.
- `src/tui.rs` 5.4k-line monolith with one-line multi-statement code.
- Tests are examples only; concurrency/cancel bugs kept recurring (16 cancel fixes) with no model.

Keep:
- `Event`/`AgentInput` enums, exactly-one-terminal invariant, `pair_remaining`.
- `Generation` trait + strict SSE assembler + persistent pre-output retry policy.
- CancellationToken tree; process-group isolation; signal-then-await.
- Tool set: read/write/edit(unique)/apply_patch(staged)/bash/exec/tree; is_error-as-output; cap+spill;
  recovery hints.
- Parallel read-only batching (≤4, order preserved).
- Profiles as Markdown+YAML frontmatter with `!command` key resolution and env-var alternative;
  secret-hygiene rules.
- Instruction/skill discovery (AGENTS/CLAUDE files, lazy SKILL.md catalog).
- ACP-as-adapter over the same bus; inline TUI with native scrollback.
- Atomic 0600 session JSON + flock lease; causal JSONL with correlation ids.

Drop / redesign:
- Adoption of other harnesses, KB/workgraph/provenance/embeddings, JEV routing, computer-use,
  decision adapters — out of core (separate crates or not at all).
- Per-round system-prompt rebuild → build once per turn (or session), stable cacheable prefix.
- Per-turn MCP lifecycle and McpSelect hop → session-lived MCP clients, direct tool exposure.
- Second persistence store; per-delta journaling (log turn/tool boundaries + final text).
- Unbounded channels; giant TUI select loop in one file.
- Add what was missing: context compaction, default turn budget, ACP `session/load` +
  permissions, and TLA+ models for turn/cancel/queue/pairing (the 16 cancel fixes are the model's
  first counterexamples).
