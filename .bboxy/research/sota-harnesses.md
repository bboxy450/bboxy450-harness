---
id: "0WW297TPTWCSS0JMPEWWV8HS5G"
number: 17
type: research
title: "State of the art: open-source coding-agent harnesses (2026-10-31)"
created: 2026-10-31
---
# State of the art: open-source coding-agent harnesses (2026-10-31)

Purpose: inform `bboxy450-harness` (Rust, native GUI, rich content, async-first).
Method: shallow clones read with grep/sed (no repo code executed), plus current public docs
for Claude Code and MCP. Pointers are `repo/path:line — mechanism`. Lines verified by grep at
the SHAs below; spot-checked by the Oracle (Aider `return` before fuzzy match, opencode
`SubscriberCapacity`, grok two-pass, goose `select_all`, Codex channel bounds and RwLock gating,
grok `acp_channels`, omp `hashline` default).

Origin: research by Claude (Oracle) for bboxy450-harness; cite as "Adapted from
bboxy450-harness/.bboxy/research/sota-harnesses.md" (see root ATTRIBUTION.md once it exists).

## 0. Sources and abbreviations

Clone root: `/private/tmp/claude-501/-Users-feral-mydev-bboxy450-harness/01a194c1-b460-45ac-87fe-4f3b6d2c93ea/scratchpad/sota/`

| Tag | Repo | SHA | Date | Lang |
|---|---|---|---|---|
| CX | openai/codex (`codex-rs/`) | 67f0994 | 2026-10-31 | Rust |
| GB | xai-org/grok-build (canonical xAI "Grok Build"; 27.6k stars) | 2bdd1d6 | 2026-09-29 | Rust |
| GC | superagent-ai/grok-cli (community) | fb97af8 | 2026-05-15 | TS/Bun |
| PI | badlogic/pi-mono (`packages/`) | 2c8d1fe | 2026-11-01 | TS |
| OMP | can1357/oh-my-pi | 1c291b8 | 2026-11-01 | TS + 80k LoC Rust |
| OC | sst/opencode (`packages/`) | 9a611d3 | 2026-10-31 | TS/Effect |
| GS | block/goose (`crates/`) | b23d21c | 2026-10-31 | Rust |
| AD | Aider-AI/aider (`aider/`) | 5dc9490 | 2026-05-22 | Python |
| ACP | zed-industries/agent-client-protocol | b4c72a8 | 2026-10-31 | Rust schema |
| CC | Claude Code docs, code.claude.com/docs/en/* | — | 2026-10-31 | docs |
| MCP | modelcontextprotocol.io spec 2026-07-28 | — | — | docs |

Path shorthands: `GBc=GB/crates/codegen`, `GBm=GB/crates/common`, `GBsh=GBc/xai-grok-shell/src`,
`GBturn=GBsh/session/acp_session_impl`.

Headline drift since the "common knowledge" picture:
- grok-cli canonical is now **xai-org/grok-build**, a large Rust ACP-native harness (not the TS community CLI).
- Codex TUI and `exec` now run **on the app-server** (in-process client), not directly on core. No ACP. No `codex mcp-server` at HEAD.
- opencode is an **Effect rewrite**: event-sourced session, durable/ephemeral events, sqlite, SSE with replay-by-sequence.
- goose dropped `goosed`; desktop spawns `goose serve` = **ACP over HTTP/WS**.
- pi-mono now ships MCP, project trust, codemode; still no permissions, no subagents.
- ACP has session list/resume/close/delete stable; fork/subagents/plan-ops unstable; v2 draft drops client fs/terminal.
- MCP 2026-07-28 is stateless (initialize is legacy), roots deprecated, tasks extension, MCP Apps (`ui://`).

---

## 1. OpenAI Codex (codex-rs)

```
 tui ──┐  exec --json ──┐  IDE ── JSON-RPC (stdio|ws|uds) ──┐
       └── app-server-client (InProcess | Remote) ─────────┤
                                                           v
            app-server  (v2 thread/turn/item; bespoke_event_handling)
                          │ Op ↓   ↑ EventMsg (protocol crate)
   core: ThreadManager → CodexThread → Session: submission_loop / run_turn
    ├─ client.rs → codex-api (SSE | WS Responses) → codex-client (HTTP, retry)
    ├─ tools: router/registry/parallel/orchestrator → sandboxing (seatbelt|bwrap+seccomp|win)
    │        execpolicy (starlark) · network-proxy · guardian · apply-patch · unified_exec
    ├─ codex-mcp → rmcp-client · hooks · skills · core-plugins · code-mode (V8)
    └─ rollout JSONL · thread-store · state (sqlx sqlite)
```

- **Loop.** `CX/core/src/session/mod.rs:670-671` — submission channel `async_channel::bounded`, event channel `unbounded`. `CX/core/src/session/handlers.rs:459` — `submission_loop` matches `Op`. `CX/core/src/tasks/mod.rs:181,273` — `SessionTask` + `spawn_task`. `CX/core/src/session/turn.rs:177,490` — `run_turn(.., cancellation_token)` outer loop; `:636` `needs_follow_up = model_needs_follow_up || has_pending_input`. Tool calls are dispatched *while the stream is still running*: `CX/core/src/stream_events_utils.rs:316` pushes an already-started `tool_future` into a `FuturesOrdered`, drained at `turn.rs:2676`.
- **Protocol.** `CX/protocol/src/protocol.rs:696` `Op` (Interrupt, TurnInput, ExecApproval, PatchApproval, ResolveElicitation, Compact, Review, Shutdown, RunUserShellCommand…). `:1518` `EventMsg` (TurnStarted/Complete, TokenCount, AgentMessageContentDelta, ExecCommandBegin/OutputDelta/End, ExecApprovalRequest, ApplyPatchApprovalRequest, PatchApplyBegin/Updated/End, TurnDiff, PlanUpdate/PlanDelta, TurnAborted, ItemStarted/Completed, Hook*, Collab*). `CX/protocol/src/items.rs:50` `TurnItem` is the new item model; old per-kind events derived in `legacy_events.rs:73`.
- **app-server v2.** `CX/app-server-protocol/src/protocol/common.rs:502` client requests `thread/start|resume|fork`, `turn/start|steer|interrupt`; `:1851` server→client requests (`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `mcpServer/elicitation/request`); `:2006` notifications (`item/agentMessage/delta`, `item/commandExecution/outputDelta`, `turn/diff/updated`, `turn/plan/updated`). Mapping core→wire: `CX/app-server/src/bespoke_event_handling.rs:148`. `CX/tui/src/lib.rs:32-39` — TUI uses `InProcessAppServerClient` / `RemoteAppServerClient`.
- **Streaming.** `CX/core/src/client.rs:2499` WS preferred (`previous_response_id` + incremental input, `:2271-2290`), else SSE `:1905`. `CX/codex-api/src/sse/responses.rs:386-547` maps Responses events; idle timeout `:598`.
- **Tools.** `CX/core/src/tools/parallel.rs:54,286` — `Arc<RwLock<()>>`: parallel-safe tools take read, others take write (run alone). `:370` race vs `cancellation_token.cancelled()`. `CX/core/src/tasks/mod.rs:72,573` — interrupt = abort_all_tasks with 100ms grace, then `TurnAborted`. `CX/core/src/exec.rs:63,70` default 10s exec timeout, exit 124. `CX/core/src/unified_exec/mod.rs:66-75` PTY sessions: yield 250ms–30s, 1MiB cap, ≤64 procs. `CX/core/src/tools/orchestrator.rs:125,664` approval→sandbox→"retry unsandboxed?" ladder.
- **Edit.** Lark grammar `CX/core/assets/tools/apply_patch.lark` as a *freeform* tool (`CX/core/src/tools/handlers/apply_patch_spec.rs:5,24`). Parser `CX/apply-patch/src/parser.rs:145` + `streaming_parser.rs`; fuzzy context seek `seek_sequence.rs:45,73`. Safety `CX/core/src/safety.rs:67` AutoApprove|AskUser|Reject.
- **Context.** `CX/protocol/src/openai_models.rs:563` auto-compact at min(cfg, 90% window). Local `CX/core/src/compact.rs:117,154,701`; remote opaque compaction item `compact_remote_v2.rs:112,421`; async mid-turn `session/async_compaction.rs`. Truncation applied to live history only, rollout keeps full payload (`context_manager/history.rs:531,542`). Typed injected context: `CX/context-fragments/src/fragment.rs:56`.
- **Persistence.** `CX/rollout/src/recorder.rs:86` JSONL; `CX/history/src/lib.rs:216` `RolloutItem`, `:394` `InitialHistory{New,Cleared,Resumed,Forked}`. `CX/core/src/thread_manager.rs:1456,1702` resume/fork. `CX/thread-store/src/store.rs:129` trait `ThreadStore`. `CX/state` sqlite, 62 migrations.
- **Permissions.** `CX/protocol/src/protocol.rs:1146` `AskForApproval{UnlessTrusted,OnRequest,Granular,Never}`, `:1232` `SandboxPolicy{DangerFullAccess,ReadOnly,ExternalSandbox,WorkspaceWrite}`, `:4428` `ReviewDecision` (incl. ApprovedExecpolicyAmendment, NetworkPolicyAmendment, TimedOut). Seatbelt `CX/sandboxing/src/seatbelt.rs:62`; Linux bwrap+seccomp `CX/linux-sandbox/src/linux_run_main.rs:58`; Starlark rules `CX/execpolicy/src/parser.rs:349,410`; network proxy modes `CX/network-proxy/src/config.rs:395`; guardian auto-reviewer `CX/core/src/guardian/mod.rs:1`.
- **Providers.** `CX/model-provider-info/src/lib.rs:116` `WireApi` = Responses only (chat rejected `:109`). `CX/model-provider/src/provider.rs:123` trait.
- **Extensions.** Hooks `CX/protocol/src/protocol.rs:1786-1800` (14 events incl. PreModelRequest/PostModelRequest, Interrupt), handler types Command|McpTool|Prompt|Agent (`protocol/src/hooks.rs:150`). Skills `CX/skills/src/parser.rs:22`. Plugins `CX/core-plugins/src/agent_plugin_manifest.rs:73`. Code-mode JS in V8 `CX/core/src/tools/code_mode/execute_handler.rs:244`.
- **Subagents.** v2 tools spawn_agent/send_message/followup_task/wait_agent/list_agents/interrupt_agent (`CX/core/src/tools/handlers/multi_agents_v2/spawn.rs:37`); mailbox `agent/control/mailbox.rs:1`; roles `.toml` `CX/agent-roles/src/agent_role_config.rs:10`.
- **Headless.** `CX/exec/src/cli.rs:65` `--json` → JSONL `ThreadEvent` (`exec_events.rs:11`).
- **Rich typed events.** PlanUpdate (`CX/protocol/src/plan_tool.rs:24`), TurnDiff (`CX/core/src/turn_diff_tracker.rs:49,114`), ImageView/ImageGeneration, WebSearchBegin/End, reasoning deltas, McpToolCall items, PatchApplyUpdated.

## 2. xAI Grok Build (xai-org/grok-build)

```
 grok binary (GBc/xai-grok-pager-bin) composition root
  ├─ TUI xai-grok-pager (ratatui; fullscreen|minimal) ── in-proc ACP channel pair | UDS → Leader
  ├─ headless -p ; `agent stdio|headless|serve|leader`
  ▼
 xai-grok-shell: MvpAgent (impl acp::Agent) → SessionActor per session
  ├─ chat-state actor · hunk-tracker actor · sampler actor (L1 SSE → L2 SamplingEvent → L3 retry)
  ├─ xai-grok-tools registry (namespaces GrokBuild|Hashline|Codex|OpenCode|MCP) / xai-tool-runtime
  ├─ xai-grok-mcp (rmcp, OAuth, SDK MCP over reverse ACP) · hooks · plugins · skills
  └─ permission-rules + sandbox (nono: Landlock/Seatbelt) · workspace (checkpoints)
```

- **Protocol = ACP everywhere**, even in-process: `GBc/xai-acp-lib/src/channel.rs:10-28` tokio mpsc pair; `gateway.rs:16-26` bridges to JSON-RPC, `_meta` → tracing spans. `GBc/xai-grok-pager/src/acp/spawn.rs:324,363-374` agent on single-thread `LocalSet` in its own thread. Vendor extension rail `_x.ai/session/update` with ~84-variant `XaiSessionUpdate` (`GBsh/extensions/notification.rs:517`).
- **Leader process**: one per machine owns agents; TUI/IDE/headless attach over UDS, request IDs namespaced (`GBsh/leader/mod.rs:1-33`).
- **Loop.** `GBturn/turn.rs:2615,2747` — loop: LoopStarted → stationarity check (identical tool-call runs nudge then hard-stop, `:3960`) → drain interjections at safe point (`interjection.rs:303`) → prefire compaction (`:2853`) → sample (`sampler_turn.rs:1716`) → `execute_tool_calls` (`:3820`). Session actor `GBsh/session/acp_session.rs:741`.
- **Streaming.** `GBc/xai-grok-sampler/src/lib.rs:6-10` three layers; `events.rs:39` `SamplingEvent{StreamStarted,FirstToken,ChannelToken{Text|Reasoning|Narration},ToolCallDelta,Completed,Retrying,Failed,DoomLoopSignals,ImagesStripped}`. Client-tunable chunk merge buffer via `InitializeRequest.meta.bufferingSettings` (`GBsh/agent/update_chunk_merge.rs:7-14`). Tools stream partial output (`GBm/xai-tool-runtime/src/streaming.rs:1`).
- **Tools.** `GBm/xai-tool-runtime/src/tool.rs:37,88-102` `trait Tool` returns `ToolStream`, dynamic descriptions. All approved calls on `FuturesUnordered` (`GBturn/tool_calls.rs:728,1008`) with **per-path file locks** (`:623,640`); later calls in a batch get synthetic "cancelled" results after a rejection (`:502-530`); waits race user interjection (`:902`). Bash 120s default / 300s model max / 24h background (`GBc/xai-grok-tools/src/implementations/grok_build/bash/mod.rs:427,898`), single knob `block_until_ms` (`:463`).
- **Edit.** Per-model *tool namespaces* mimicking other harnesses (`GBc/xai-grok-tools/src/types/tool.rs:29`): search/replace, Codex apply_patch, OpenCode edit, **Hashline** with 3 anchor schemes (ContentOnly, ChunkFingerprint, CheckpointChain) and stale-anchor recovery (`grok_build_hashline/scheme.rs:1-17`). Hunks attributed agent vs external (`GBc/xai-hunk-tracker/src/lib.rs:1`).
- **Context.** **Prefire two-pass compaction**: pass 1 summarizes ~95% in background before limit, pass 2 folds the ~5% tail (`GBsh/session/two_pass.rs:1-13`, split const `:13`). Compacted transcript written as `compaction/segment_*.md` + `INDEX.md`, model can read it back (`GBc/xai-compaction-transcript/src/lib.rs:15`). Oversized prompts offloaded to files (`GBturn/prompt_offload.rs:1`). BM25 deferred MCP tool search (`GBsh/session/tool_index.rs:1`). Transport-agnostic compaction core (`GBm/xai-grok-compaction/src/lib.rs:1-20`).
- **Persistence.** Single FIFO persistence actor with an explicit **loss contract** (fsync barriers, atomic rename, dir fsync) `GBsh/session/persistence.rs:1-13`. Per-session JSONL (`chat_history`, `updates`, `rewind_points`, `events`), torn lines skipped (`storage/jsonl/mod.rs:633`). Replay window 64 in-flight (`GBsh/agent/mvp_agent/replay.rs:15`). Fork `GBsh/session/fork.rs:1`; per-prompt file rewind (`GBc/xai-grok-workspace/src/session/file_state.rs:1`); worktree pool.
- **Permissions.** Allow/Deny/Ask rules (`GBc/xai-grok-permission-rules/src/types.rs:127`), bash split by tree-sitter (`bash_permission_script.rs:1`), reads `.claude/settings.json` (`claude_settings.rs:1`). Auto mode = LLM transcript classifier with fast paths (`GBc/xai-grok-workspace/src/permission/auto_mode/mod.rs:1`). nono sandbox once at startup + per-subprocess seccomp net block; profiles Workspace|Devbox|ReadOnly|Strict|Off|Custom (`GBc/xai-grok-sandbox/src/profiles.rs:70`).
- **Providers.** `ApiBackend{ChatCompletions,Responses,Messages}` (`GBc/xai-grok-sampling-types/src/types.rs:1104`); structured output native or forced-tool fallback (`GBturn/turn.rs:2738`).
- **Extensions.** 16 hook events with Claude/Cursor aliases (`GBc/xai-grok-hooks/src/event.rs:78-175`); plugins accept `.claude-plugin/plugin.json` (`GBc/xai-grok-agent/src/plugins/manifest.rs:1-6`); skills multi-source; SDK MCP over reverse ACP `x.ai/mcp/sdk_call` (`GBc/xai-grok-mcp/src/acp_transport.rs:1`).
- **Subagents.** `task` tool → `SubagentBackend` + coordinator actor, `MaxSubagentDepth` (`GBc/xai-grok-tools/src/implementations/grok_build/task/mod.rs:1-16`); resolution override>role>persona>parent (`GBc/xai-grok-subagent-resolution/src/lib.rs:1`); parent↔child mid-turn messaging (`GBturn/parent_interject.rs`).
- **TUI.** Elm-ish `Action → dispatch → Effect` on `JoinSet` → `TaskResult` (`GBc/xai-grok-pager/src/app/actions.rs:55`, `effects/mod.rs:2`). Streaming markdown re-renders only the tail after the last stable checkpoint (`GBc/xai-grok-markdown/src/streaming.rs:1-12`); LaTeX→Unicode; pure-Rust Mermaid→PNG (dagre+resvg, crash isolated) `GBc/xai-grok-mermaid/src/lib.rs:1-14`; Kitty images (`GBc/xai-grok-pager-render/src/terminal/image.rs:1`); per-cell table selection; mouse hit-testing.
- **Behavior guards.** Goal stop detector (`GBsh/session/goal_stop_detector.rs:1`), laziness classifier (`GBturn/laziness.rs`), server doom-loop signals (`GBc/xai-grok-sampler/src/doom_loop.rs:1`).

**grok-cli (community, GC).** Bun + Vercel AI SDK v6 `streamText({stopWhen: stepCountIs})` is the loop (`GC/src/agent/agent.ts:1324,1824`); zod tools incl. ~10 computer_* (`GC/src/grok/tools.ts:51,387`); exact old_string edit + diff preview (`GC/src/tools/file.ts:90`); `task` subagents (`tools.ts:366`); SQLite sessions (`GC/src/storage/migrations.ts:39`); MCP via `@ai-sdk/mcp`. Thin; nothing to steal beyond what GB does better.

## 3. pi (badlogic/pi-mono)

```
 pi CLI (coding-agent): interactive TUI | print/json | rpc (JSONL stdio) | SDK
   │ AgentSession: sessions, compaction, extensions, skills, MCP, trust
   ▼
 pi-agent-core: Agent{steer/followUp queues} → agentLoop → AgentEvent stream
   ▼
 pi-ai: model registry → provider streams → AssistantMessageEventStream (~40 providers)
 side: pi-tui (diff render) · pi-mcp · codemode (QuickJS wasm) · durable
```

- **Loop + events.** `PI/agent/src/agent-loop.ts:47,111`; events agent_start/turn_start/message_start/update/end/tool_execution_start/update/end/turn_end/agent_end (`PI/agent/src/types.ts:516-532`). **Steering vs follow-up queues**: steer polled mid-turn (`agent-loop.ts:185,214,304`), follow-up after inner loop (`:311`); `PI/agent/src/agent.ts:308,313`. Hooks `convertToLlm`, `transformContext`, `beforeToolCall` (block/terminate), `afterToolCall` (`types.ts:222-341`).
- **Provider.** One message union (`PI/ai/src/types.ts:659`), one stream event set text_delta/thinking_delta/toolcall_start/toolcall_delta/done/error (`:821-837`). **Cross-provider handoff** transform: drops/converts thinking, keeps signed, normalizes tool-call IDs (`PI/ai/src/api/transform-messages.ts:91-157`). AbortSignal in options (`types.ts:161`).
- **Tools.** Default exactly read/bash/edit/write (`PI/coding-agent/src/core/tools/index.ts:195`). Edit = `edits[]{oldText,newText}` unique, non-overlapping, repairs malformed args, fuzzy fallback after normalization (`tools/edit.ts:23-131`, `edit-diff.ts:34`). Parallel by default; any `executionMode:"sequential"` serializes the batch (`agent-loop.ts:527`); per-file mutation queue (`tools/file-mutation-queue.ts:32`).
- **Compaction.** reserve 16384, keepRecent 20000 (`core/compaction/compaction.ts:128`); branch summaries for abandoned branches (`branch-summarization.ts`); one overflow-recovery retry (`core/agent-session.ts:419`).
- **Sessions.** Append-only **JSONL tree** (id/parentId), `branch`, `branchWithSummary`, `forkFrom` (`core/session-manager.ts:41,1589-1915`).
- **Permissions.** None by design; docs say run in a container. Only *project trust* gates loading `.pi/` resources (`core/project-trust.ts:47`). Approval gate is an example extension.
- **Extensions.** `on(event)` incl. `context`, `tool_call`, `tool_result`, `provider_stream_event`, `session_compact` (`core/extensions/types.ts:1621-1688`); `registerTool/Command/Shortcut/Flag/MessageRenderer` (`:1695-1739`). npm "pi-package" distribution. MCP now built in with exposure modes codemode|deferred|direct|hidden (`core/mcp-servers.ts:52`).
- **Subagents.** Not built in (example extension only).
- **UI.** pi-tui line-diff rendering (`PI/tui/src/tui-main-screen.ts:129,214`); RPC commands prompt/steer/follow_up/abort/compact/fork (`modes/rpc/rpc-types.ts:22-62`); SDK `createAgentSession` (`core/sdk.ts:191`).

## 4. oh-my-pi (omp)

```
 omp CLI: interactive | print | rpc(+wire) | ACP | SDK          collab-web (browser guest)
   │ AgentSession: sessions(file|sql|redis), TTSR, advisor, plan, task, MCP, LSP, DAP
   ▼                                                  ▲ pi-wire frames (AES-GCM relay)
 agent-core fork: loop + live steering + speculative exec + append-only ctx + shake
   ▼
 pi-ai fork: 60+ providers, text tool-call "dialects", auth broker
   ▼
 pi-natives (Rust N-API): pi-edit hashline, grep/glob, pty, CoW iso, snapcompact raster
```

- **Over pi:** hashline edit default (`OMP/packages/coding-agent/src/edit/settings.ts:40`; Rust `OMP/crates/pi-edit/src/modes/hashline/format.rs:97`; `[PATH#TAG]` 4-hex snapshot tag, stale tag rejects). Tool concurrency `shared|exclusive`, bash decides per call (`OMP/packages/agent/src/agent-loop.ts:3856`, `tools/bash.ts:645`). **Live steering** into an in-flight response (`live-steering.ts:1-20`). **Speculative tool execution** (`speculative-execution.ts:234`). **Append-only, cache-stable request prefix** (`append-only-context.ts:2-9`). **TTSR** regex rules abort the stream mid-token, inject rule, retry (`session/ttsr-coordinator.ts:75`). Advisor model reviews turns. Text tool-call **dialects** for models without native tool calling (`packages/ai/src/dialect/`).
- **Compaction ladder:** LLM summary, provider-native, `shake` (mechanical placeholder replacement, no LLM, `compaction/shake.ts:2-9`), pruning, **snapcompact** (history rasterized to PNG pixel-font frames for vision models, Rust `crates/pi-natives/src/snapcompact.rs`).
- **Sessions.** Pluggable storage File|Sql|Redis (`session-storage.ts:731`, `sql-session-storage.ts:254`, `redis-session-storage.ts:109`); imports Claude/Codex sessions.
- **Permissions.** always-ask|always-ask-cached|write|yolo, **default yolo** (`tools/approval.ts:44,110`); CoW isolation for subagents (`OMP/crates/pi-iso`: apfs/btrfs/overlayfs/projfs).
- **Subagents.** `task` tool with schema-validated structured yields (`task/structured-subagent.ts`), concurrency-limited fan-out (`task/parallel.ts:26`).
- **UI.** ACP mode (`modes/acp/`), pi-wire collab protocol `GuestFrame`/`HostFrame` (`OMP/packages/wire/src/index.ts:328,349`), browser guest client.
- **~/.omp (structure only; agent.db not opened):** `agent/{agent.db,history.db,models.db,skill-descriptions.db}` sqlite-WAL; `config.yml` keys `modelRoles{default,smol,slow,vision,plan,commit,tiny,memory,task,advisor,judge}`, `tools.approvalMode`; `mcp.json`; `extensions/*.ts`; `blobs/` sha256-named images; `sessions/<cwd-slug>/<ISO>_<uuidv7>.jsonl` + same-named artifact dir (`N.bash.log`, `0.shake.log`). Line 1 is a padded, in-place-rewritten title record; then `{"type":"session","version":3,…}`; then id/parentId entries (message, custom, model_change, thinking_level_change, compaction). Idea: **model roles** as a first-class config axis.

## 5. opencode (sst/opencode, Effect rewrite)

```
 TUI  Desktop(Electron)  Web/session-ui  ACP(cli/src/acp)  SDK
   \______ HTTP + SSE /api/event (protocol HttpApi → openapi) ______/
            server (Effect HttpRouter)   Bus: durable → sqlite log | ephemeral deltas
     SessionRunner → step → @opencode/ai stream | AI-SDK doStream adapter
       tool fibers (forkScoped) · Snapshot (shadow git) · Projector → drizzle sqlite
```

- **Event-sourced session.** `OC/schema/src/session-event.ts:334-648` (`session.step.started/ended`, `text.started/delta/ended`, `reasoning.*`, `tool.input.started/delta/ended`, `tool.called/progress/success/failed`, `retry.scheduled`, `compaction.*`, `revert.staged/committed`); `OC/core/src/session/projector.ts` folds to tables. `OC/schema/src/event.ts:42,55` each event is **durable (versioned, persisted) or ephemeral**.
- **Client/server.** Contract defined once (`OC/protocol/src/groups/*.ts`) → OpenAPI → SDK. `OC/protocol/src/groups/event.ts:61` SSE; `OC/server/src/event-feed.ts:8` **per-subscriber 4096 buffer, overflow → `SubscriberOverflowError`** (slow consumer is dropped, producer never blocks); 15s heartbeat (`handlers/event.ts:19-27`). `OC/protocol/src/groups/session.ts:652` **replay durable log `?after=<seq>&follow=`**. `OC/core/src/bus.ts:22,35` sequences. Shared local service discovery + version check (`OC/client/src/service.ts:25-33`); one SSE shared by many subscribers (`shared-events.ts:17`).
- **Streaming.** Text deltas batched every 100ms before publish (`OC/core/src/session/runner/publish-llm-event.ts:98,186`).
- **Tools.** Each tool call its own scoped fiber; abort = `Fiber.interruptAll`, closing events published under `uninterruptibleMask` (`OC/core/src/session/runner/step.ts:96,130-149`). Output cap 2000 lines/50KB, overflow spilled to file kept 7 days (`OC/core/src/tool-output.ts:13`). Edit: exact → normalize typographic quotes/dashes → ignore trailing ws; uniqueness unless replaceAll; preserve EOL; formatter after edit (`OC/core/src/tool/plugin/edit.ts:49-118,370`). LSP diagnostics after edit still TODO (`edit.ts:258`).
- **Compaction.** 15% reserve ≥16k; background pre-summary at 85%; reduction ladder: clip tool outputs to 2k → remove outputs → user-only → omit (`OC/core/src/session/compaction.ts:128-157`).
- **Persistence.** drizzle sqlite (`session, message, part, inbox, blob, event log`) `OC/core/src/session/sql.ts:28`; **shadow-git snapshot per step** (`OC/core/src/snapshot.ts:92`, `runner/step.ts:76`); staged revert (`session/revert.ts:22`); fork route (`session.ts:258`).
- **Permissions.** allow|deny|ask with wildcard last-match, default ask; replies once|always|reject; `Unattended` ruleset for no-human runs (`OC/core/src/permission.ts:52,86`).
- **Extensions.** hook domains aisdk/session/permission/shell/tool; only `tool execute.before` may reject (`OC/core/src/plugin/hooks.ts:13-23`). Agents primary|subagent|all (`OC/schema/src/agent.ts:30`).
- **ACP.** Agent side via `@agentclientprotocol/sdk` (`OC/cli/src/acp/agent.ts:11`); child sessions projected into parent (`acp/child.ts:39`); compaction surfaced as ACP `compaction_update` (`acp/compaction.ts:79`).

## 6. goose (block/goose)

```
 Desktop Electron ─spawn `goose serve`→ ACP over HTTP/WS (axum) ← other ACP clients (Zed: `goose acp`)
 goose CLI ─in-process→ AgentServices.reply()
   StateMachine [ops: ToolApproval, Skill, Recipe, ToolExecution, Subagent, Steer, Compaction, Retry…] + Inference
   Provider trait · ExtensionManager (MCP stdio/http/builtin/platform) · ToolInspectionManager · sqlite
```

- **Loop as composable ops.** `GS/goose-agent/src/machine.rs:45,162,241` list of `Step::Operation` then `Step::Inference`; order in `GS/goose/src/agents/state_machine_builder.rs:73-146`. `AgentEvent` (`GS/goose-agent/src/events.rs:9`) Message|Usage|McpNotification|HistoryReplaced. Content blocks incl. `ToolConfirmationRequest`, `ActionRequired`, `Thinking`, `RedactedThinking`, `SystemNotification` (`GS/goose-provider-types/src/conversation/message.rs:318`).
- **Tools.** `futures::stream::select_all` over tool streams (`GS/goose/src/agents/state_machine/ops_toolcalling.rs:1055`); per-tool `tokio::select!` across notifications/action-required/result (`agents/tool_execution.rs:135`); `CancellationToken` (`ops_toolcalling.rs:403,833`); per-extension timeout (`extension_manager/mod.rs:123`); responses >200k chars spilled to file (`large_response_handler.rs:5`). Exact before/after edit (`platform_extensions/developer/edit.rs:129`).
- **Context.** threshold 0.8, limit 225k; max 2 error-triggered compactions; **tool-pair compaction** summarizes old call/result pairs in batches of 10 (`GS/goose/src/context_mgmt/mod.rs:25-61`, `ops_tool_pair_compaction.rs:21`).
- **Permissions.** Modes Auto|Approve|SmartApprove|Chat (`GS/goose-provider-types/src/goose_mode.rs:22`); **inspector pipeline**: Permission, LLM read-only judge, prompt-injection Security, Egress, Adversary, Repetition (`GS/goose/src/tool_inspection.rs:57,75`).
- **Fork** = copy+truncate (`GS/goose/src/acp/server/fork_session.rs:34`). `AcpProvider` drives an external ACP agent *as a model* (`GS/goose/src/acp/provider.rs:274`). Recipes = parameterized, schema-typed agent configs (`GS/goose/src/recipe/mod.rs:43`). MCP Apps `ui://` HTML resources proxied (`GS/goose/src/goose_apps/resource.rs:78`, `acp/mcp_app_proxy.rs`).

## 7. Aider

```
 user → Coder.run_one → send_message → LLM(system+examples+summary+repo map+files+history)
        ^ reflected_message (≤3)                         | parse edits per format
        └── lint/test/apply error ←── apply_edits → auto_commit (git)
 RepoMap: tree-sitter def/ref → MultiDiGraph → personalized PageRank → binary-search fit to budget
```

- **Repo map.** `AD/repomap.py:279,319` tree-sitter tags; `:365,476-505` ref→def edges weighted (×10 mentioned ident, ×50 chat file, ×0.1 private/overdefined, sqrt(refs)); `:519` personalized PageRank; `:676-703` binary search tag count to ±15% of `max_map_tokens`; `:124` budget ×8 when no files in chat; mtime-keyed disk cache `:217`.
- **Edit formats.** whole (`AD/coders/wholefile_coder.py:13`), SEARCH/REPLACE `diff` (`editblock_coder.py:18`), `udiff` (`udiff_coder.py:49`), `patch` (`patch_coder.py:217`), editor-* ; **architect mode**: strong model plans, fresh editor Coder applies (`architect_coder.py:17-48`). Format is per-model config.
- **Apply failure → reflection, not silent fuzz.** Exact/whitespace/`...` elision tolerant (`editblock_coder.py:157,190`); edit-distance fuzzy is dead code behind a bare `return` (`:183`, verified). On miss: "Did you mean" lines + count applied, fed back as reflection (`:92-125`, `base_coder.py:2315`). udiff uses a strategy ladder incl. diff-match-patch and relative indentation (`search_replace.py:528-565`).
- **Reflection loop** max 3 (`base_coder.py:101,924-944`); auto-lint/test feed errors back (`:1599-1622`).
- **Git.** Dirty-commit user changes before AI edits (`base_coder.py:2411`); LLM commit message via weak model (`repo.py:326`); author "(aider)" attribution (`repo.py:165`).
- **History summarization** in a background thread, swapped in only if history unchanged (optimistic CAS) (`base_coder.py:1002-1034`; `history.py:33`).

## 8. Agent Client Protocol (Zed)

```
 Client(editor) ─spawn→ Agent   [JSON-RPC 2.0, NDJSON over stdio; stderr = logs]
  initialize{protocolVersion, clientCapabilities} → {agentCapabilities, authMethods}
  session/new{cwd, mcpServers[]} → {sessionId, modes?, configOptions?}
  session/prompt{ContentBlock[]} ───────────────→ turn
     ← session/update (chunks, tool_call(+_update), plan, usage, …)
     ← session/request_permission | fs/*_text_file | terminal/*   (agent→client requests)
  session/cancel (notif) → prompt resolves {stopReason}
```

- Transport `ACP/docs/protocol/v1/transports.mdx:6-24`; HTTP/WS still RFD. Version `ACP/agent-client-protocol-schema/src/version.rs:19-39` (LATEST=V1).
- Methods `ACP/schema/v1/meta.json`: agent `initialize, authenticate, session/new|load|prompt|cancel|list|resume|close|delete|set_mode|set_config_option, logout`; client `session/request_permission, session/update, fs/read_text_file, fs/write_text_file, terminal/create|output|release|wait_for_exit|kill, elicitation/create|complete`; `$/cancel_request`.
- `session/update` variants (`src/v1/client.rs:105`): user/agent message chunk, agent_thought_chunk, tool_call, tool_call_update, plan, available_commands_update, current_mode_update, config_option_update, session_info_update, usage_update; gated: notice, compaction_update, compaction_summary_chunk; unstable: plan_update/plan_removed, subagent_update, session_message(_chunk) (`:124-165`).
- Content: `ContentBlock` Text|Image|Audio|ResourceLink|Resource (`src/v1/content.rs:32`). `ToolCall{toolCallId,title,kind,status,content[],locations[],rawInput,rawOutput}` (`tool_call.rs:31-68`); `ToolKind` read|edit|delete|move|search|execute|think|fetch|switch_mode|other (`:473`); status pending|in_progress|completed|failed (`:514`); content `content|diff{path,oldText?,newText}|terminal{terminalId}` (`:546,676`). StopReason end_turn|max_tokens|max_turn_requests|refusal|cancelled (`agent.rs:4404`). Permission options allow_once|allow_always|reject_once|reject_always (`client.rs:2376`).
- Model selection = config option category Model/ThoughtLevel (`agent.rs:3404`), no set_model.
- MCP passing: `mcpServers` Http|Sse|Stdio, unstable MCP-over-ACP (`agent.rs:3744,3767`).
- **v2 draft** (`docs/announcements/acp-v2-draft.mdx:21-50`): updates any time (not only in a turn); messages/tool calls **patched by ID** (omit = keep, null = clear, chunks append); structured file changes add/delete/modify/move/copy/binary + optional git patch; `tool_call/cancel`, `terminal/cancel`; client fs/terminal removed.
- Adopters seen: grok-build (internal protocol), goose (primary server), opencode, omp. Not Codex.

## 9. Claude Code (public docs) and MCP

- **Hooks** (code.claude.com/docs/en/hooks.md): ~30 events — Session{Start,End}, Setup, UserPromptSubmit, UserPromptExpansion, Stop, StopFailure, PreToolUse, PermissionRequest, PermissionDenied, PostToolUse, PostToolUseFailure, PostToolBatch, Subagent{Start,Stop}, Task{Created,Completed}, TeammateIdle, Pre/PostCompact, Pre/PostModelSwitch, Notification, MessageDisplay, Elicitation(Result), ConfigChange, InstructionsLoaded, CwdChanged, DirectoryAdded, FileChanged, Worktree{Create,Remove}. JSON on stdin; exit 2 = block (stderr reason); PreToolUse output `permissionDecision allow|deny|ask|defer`, `updatedInput`, `additionalContext`. Hook types command|http|mcp_tool|prompt|agent. Hook allow does not override deny rules.
- **Subagents** (sub-agents.md): markdown + frontmatter (`tools`, `disallowedTools`, `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `hooks`, `background`, `isolation: worktree`, `effort`); own context; parent gets final report only; default concurrency 20, nesting depth 3; resume via SendMessage.
- **Compaction** (context-window.md): auto as window fills, `/compact [focus]`, `/rewind` summarize-from-here; survives: system prompt, root CLAUDE.md (re-read from disk), plan, ≤5 recently touched files, invoked skill bodies (5k each/25k total); Pre/PostCompact hooks; SessionStart matcher `compact` to re-inject.
- **Permissions** (permissions.md, sandboxing.md): modes default|acceptEdits|plan|auto(classifier)|dontAsk|bypassPermissions; rules `Tool(specifier)`, deny→ask→allow; SDK order hooks→deny→ask→mode→allow→`canUseTool`; OS sandbox Seatbelt/bubblewrap, `allowedDomains`, built-in file tools and hooks run outside the sandbox.
- **Agent SDK** (agent-sdk/*): message stream `system/init`, `assistant`, `user` (with `parent_tool_use_id`), `stream_event` partials, `result` (cost, session id), task progress/notification; `resume`, `forkSession`; `claude -p --output-format stream-json`; in-process MCP tools via `createSdkMcpServer`.
- **Skills** (skills.md): progressive disclosure — name+description always loaded, body on invoke, supporting files on demand; `context: fork`; `paths:` scoped rules.
- **MCP 2026-07-28** (modelcontextprotocol.io/specification/2026-07-28): stateless JSON-RPC, per-request capabilities in `_meta`; stdio + Streamable HTTP; tools with `outputSchema`/`structuredContent`, annotations readOnly/destructive/idempotent/openWorld (untrusted hints); content text|image|audio|resource_link|resource; `isError` vs JSON-RPC error; **input_required (MRTR)** multi-round tool calls; elicitation form|url; **tasks extension** (working|input_required|completed|failed|cancelled, poll/subscribe); progress tokens; `notifications/cancelled`; **MCP Apps** `ui://` HTML in sandboxed iframe with postMessage JSON-RPC; roots deprecated.

---

## 10. Cross-harness comparison

| Axis | CX | GB | PI | OMP | OC | GS | AD |
|---|---|---|---|---|---|---|---|
| Core↔UI | Op/EventMsg + app-server JSON-RPC | ACP (in-proc channels / UDS leader) | AgentEvent + RPC JSONL | + ACP + wire | HTTP+SSE, event-sourced | ACP HTTP/WS | in-proc |
| Headless | exec --json | -p, agent serve | print/json/rpc | same | server | goose run / ACP | --message |
| Tool parallel | RwLock read/write | FuturesUnordered + path locks | parallel unless sequential + file queue | shared/exclusive | fiber per call | select_all | n/a |
| Cancel | CancellationToken, 100ms grace | select! + synthetic cancelled | AbortSignal | AbortSignal | Fiber.interruptAll | CancellationToken | ctrl-c |
| Edit | apply_patch (Lark freeform) | namespaces: S/R, apply_patch, hashline | exact multi-edit + fuzzy | hashline (Rust) | exact+normalize ladder | exact | S/R, udiff, architect |
| Compaction | local / remote opaque / async | prefire 2-pass, segments on disk | summary + branch summary | summary/shake/prune/snapcompact | reduction ladder + bg pre-summary | threshold + tool-pair | bg summary CAS |
| Persistence | JSONL rollout + sqlite state | JSONL + persistence actor loss contract | JSONL tree | JSONL tree / sql / redis | sqlite event log + shadow git | sqlite | git |
| Fork | thread/fork | fork.rs | branch/forkFrom | fork/branch | fork route | copy+truncate | — |
| Sandbox | seatbelt/bwrap/seccomp/win + starlark + net proxy | nono Landlock/Seatbelt + seccomp | none | CoW iso, yolo default | rules only | inspectors | none |
| Subagents | v2 agent tools + mailbox | task + coordinator | none | task + typed yields | subagent tool | subagent op | architect |
| ACP | no | yes (core) | no | yes | yes | yes (core) | no |

---

## 11. Synthesis

### 11.1 Canonical moving parts

| # | Component | Responsibility |
|---|---|---|
| C1 | **Protocol** (types crate) | Op (inbound), Event (outbound), Item model, ids, versioning. No deps. |
| C2 | **Engine / Session actor** | Owns one session's state; serializes Ops; runs turns; emits Events. |
| C3 | **Turn runner** | sample → dispatch tools → fold results → follow-up decision → stop. |
| C4 | **Provider layer** | Model registry, per-wire adapters (Responses, Messages, ChatCompletions, Gemini), normalized stream events, retry/backoff, cross-provider transcript transform. |
| C5 | **Context manager** | Prompt assembly (system, fragments, memory files, skills index), token accounting, truncation, compaction strategies, cache-stable prefix. |
| C6 | **Tool runtime** | Registry, schemas, concurrency class, timeouts, output caps/spill, streaming partials, cancellation. |
| C7 | **Exec subsystem** | Process/PTY spawn, background jobs, output capture, kill tree. |
| C8 | **Edit subsystem** | Patch grammar(s), matcher ladder, per-path locks, diff tracker (turn diff, hunk attribution), formatter/LSP feedback. |
| C9 | **Policy** | Approval rules, modes, classifiers/inspectors, sandbox profiles, network policy; returns Allow/Ask/Deny + amendments. |
| C10 | **Sandbox backends** | Seatbelt / Landlock+seccomp / bwrap / CoW worktree. |
| C11 | **Store** | Append-only event log (durable), projections (sessions list, search), blobs, checkpoints/snapshots, fork/resume. |
| C12 | **Extension host** | Hooks, MCP client (+ server), skills, plugins, slash commands, custom renderers. |
| C13 | **Agent manager** | Subagent spawn/resume, mailbox, depth/concurrency limits, isolation, structured yields. |
| C14 | **Gateway / adapters** | Fan-out of events to frontends: in-proc GUI, TUI, ACP stdio, headless JSONL, remote (WS/UDS). Replay by sequence. |
| C15 | **Frontends** | GUI (native), TUI, ACP editor, headless CLI, SDK. Pure consumers of C1. |

Pairwise interfaces (only the non-trivial ones):

| From → To | Interface | What crosses | Ref model |
|---|---|---|---|
| C15 ↔ C14 | `Op` in, `Envelope{seq, session, Event}` out; subscribe(after_seq, filter) | typed ops; durable + ephemeral events | OC event-feed + log?after=; CX app-server |
| C14 ↔ C2 | `SessionHandle{tx_op: bounded mpsc, events: broadcast}` | Op, Event | CX `SessionIo` (bounded sub / unbounded ev) |
| C2 → C3 | `run_turn(ctx, CancellationToken) -> TurnOutcome` | turn input, settings snapshot | CX `run_turn`, GB `process_conversation_turn_inner` |
| C3 ↔ C4 | `stream(Request, Cancel) -> Stream<ModelEvent>` | normalized deltas: text, reasoning, tool_call_delta, usage, done, error | PI `AssistantMessageEventStream`, GB `SamplingEvent` |
| C3 ↔ C6 | `dispatch(ToolCall, ToolCtx) -> Stream<ToolUpdate>` + `concurrency()` | args, partial output, final result (content blocks + structured) | GB `ToolStream`, CX RwLock gate |
| C6 → C9 | `decide(Action) -> Allow | Ask(req) | Deny(reason)` | normalized action (exec argv, path write, net host, mcp tool) | CX orchestrator, OC permission.evaluate |
| C9 ↔ C15 | Ask = Event `ApprovalRequested{id, subject, options}`; Op `ApprovalResolved` | | ACP request_permission, CX ExecApproval |
| C6 → C7/C8/C10 | spawn(spec, sandbox_profile) / apply(patch) | | CX sandboxing manager |
| C3 → C5 | `assemble(history) -> Prompt`; `should_compact(usage)`; `compact(strategy)` | | CX compact.rs, GB two_pass |
| C2 → C11 | `append(Event) -> seq` (single writer per session), `load(id) -> Vec<Event>`, `fork(id, at_seq)` | | GB persistence actor, OC event log |
| C12 ↔ C3/C6 | hook points pre/post tool, prompt submit, compact, stop; MCP tools appear as C6 tools | | CC hooks, CX hook_runtime |
| C13 ↔ C2 | child = another Session with parent link; parent sees child events tagged `parent_id` | | CC `parent_tool_use_id`, OC acp/child.ts |

### 11.2 Async architecture for Rust

Shape: **headless engine of actors, event-sourced, typed protocol; every frontend is an adapter.**

```
 GUI ─┐  TUI ─┐  ACP stdio ─┐  headless JSONL ─┐  remote WS/UDS ─┐
      └───────┴─────────────┴──────────────────┴────────────────┘
                     Gateway: subscribe(after_seq), send(Op) → Ack
                       │ bounded mpsc (ops)   ▲ broadcast/watch (events)
          Engine ── SessionActor[n] ── TurnTask (JoinSet, CancellationToken tree)
                       │                        ├─ ProviderStream
                       │                        └─ ToolTask[k] (concurrency class, timeout)
                   EventLog writer (one per session, fsync policy) → sqlite/JSONL + blobs
```

1. **Protocol crate first** (C1). `enum Op`, `enum Event`, `struct Item` with stable ids; `#[serde(tag="type")]`; schema generated (schemars) for ACP/TS consumers. Mirror the **ACP v2 patch-by-id** model: `ItemStarted{id, kind}`, `ItemDelta{id, patch}`, `ItemCompleted{id, final}` — deltas append, finals replace. GUIs then render one card per item id. (CX TurnItem, ACP v2.)
2. **Durable vs ephemeral events** (OC `schema/src/event.ts:42`). Durable: items completed, approvals, settings, compaction, fork points → event log, monotonic `seq`. Ephemeral: text/reasoning/tool-output deltas → never persisted, coalesced. Any client reconnects with `subscribe(after_seq)`: replay durable, then live. Late GUI windows and crashed frontends recover for free.
3. **Session = actor.** One `tokio::task` owning state; inbox `mpsc::channel(N)` bounded; `send` from UI uses `try_send` → on Full returns `Busy` immediately (never awaits in a UI thread). Ops are cheap (enqueue only); work happens in spawned turn tasks. This is the CX submission_loop pattern minus its unbounded event queue.
4. **Fan-out without backpressure on the engine.** Engine publishes to a per-session `broadcast` (ephemeral) and the log (durable). Each subscriber has its own bounded buffer (OC uses 4096); on lag: drop ephemeral, mark `Lagged{from_seq}`, client re-fetches from the log. **The engine never waits on a frontend.** (OC `SubscriberOverflowError`; tokio `broadcast::RecvError::Lagged`.) Coalesce deltas per item at ~30–100ms per subscriber (OC 100ms, GB client-tunable `bufferingSettings`).
5. **Cancellation tree.** `tokio_util::sync::CancellationToken` per session → child per turn → child per tool/subagent. Interrupt = cancel turn token, grace period (CX 100ms), then abort JoinSet; always emit terminal events for every open item (`cancelled`) under a non-cancellable section (OC `uninterruptibleMask`, GB synthetic "cancelled due to earlier…" results). Kill process trees, not pids.
6. **Structured concurrency.** Each turn owns a `JoinSet`; tools never outlive their turn unless explicitly promoted to background jobs (owned by a session-level `JobManager` with its own token and visible state). No detached `tokio::spawn` without an owner.
7. **Tool concurrency classes.** `ReadOnly` (parallel), `Exclusive` (alone; RwLock write, CX), plus **per-path locks** for writes inside parallel batches (GB `tool_calls.rs:623`, PI file-mutation-queue). Start tools as soon as their call is complete in the stream (CX `stream_events_utils.rs:316`), not after the message ends.
8. **Timeouts everywhere, labelled.** Provider idle timeout (CX `sse/responses.rs:598`), per-tool timeout with model-visible max (GB 300s / CX 10s default), approval timeout → `TimedOut` decision (CX `ReviewDecision`), MCP per-server timeout (GS). A timeout produces an event, never a hang.
9. **Never-blocking send/receive.** User input during a turn = `Op::Steer` queued and drained at safe points (PI steer queue, GB interjections, CX `turn/steer`); follow-ups queued for after the turn. Approval waits are `select!` with the cancel token and timeout. Identity/provider unavailable → degrade with a labelled event, keep session responsive.
10. **Single-writer persistence actor** with explicit loss contract (GB `persistence.rs`): append, fsync at turn/approval boundaries, atomic rename for snapshots, torn-line tolerant loader. Tree-shaped log (id/parentId, PI/OMP) so fork/rewind/branch are pointer moves.
11. **Frontend adapters are thin, all in-repo, all tested headless.** GUI and TUI link the engine in-process via the same `Gateway` trait used by the remote transport (CX `InProcessAppServerClient`/`RemoteAppServerClient`; GB in-proc ACP channels). ACP adapter maps Item events → `session/update`, approvals → `request_permission`. Headless = JSONL of the same envelopes (CX exec --json).
12. **GUI thread rule.** The GUI never awaits the engine; it drains a channel each frame (`try_recv` loop), keeps a view-model projection keyed by item id, and requests repaint on event arrival. Expensive rendering (markdown parse, syntax highlight, diff layout, mermaid) runs on a worker pool with incremental "stable prefix" caching (GB `xai-grok-markdown/src/streaming.rs`).
13. **Formal model targets** (operator mandate): session actor op/event ordering, cancel tree (no orphan item left non-terminal), approval lifecycle (Ask → Resolved|TimedOut|Cancelled exactly once), subscriber lag/replay (no durable gap after reconnect), fork A→B→A.

### 11.3 Rich content a GUI-first harness should render natively

| Content | Typed event exists in | Recommended item kind |
|---|---|---|
| Streaming markdown (incl. code fences, LaTeX, links) | all (deltas); GB stable-prefix renderer | `AgentMessage{blocks}` |
| Reasoning / thinking (collapsible, summary vs raw) | CX ReasoningContentDelta, ACP agent_thought_chunk, PI thinking_delta, GS Thinking/RedactedThinking | `Reasoning{summary, raw?}` |
| Tool-call card (title, kind, status, args, locations, output) | ACP ToolCall/ToolKind/status, CX ItemStarted/Completed, OC tool.* events | `ToolCall{kind, status, input, output_blocks, locations}` |
| Diff (per edit + aggregate turn diff, hunk attribution) | ACP `diff{path,oldText,newText}`, ACP v2 structured file changes, CX TurnDiff/PatchApply*, GB hunk-tracker | `FileChange{ops[], unified?}` + `TurnDiff` |
| Terminal output (ANSI, live, PTY, background) | ACP `terminal{terminalId}`, CX ExecCommandOutputDelta, GB background tasks | `Terminal{id, stream, exit}` |
| Plan / todo with statuses | ACP plan (+unstable plan_update), CX PlanUpdate/PlanDelta | `Plan{entries[{text,status,priority}]}` |
| Approval prompt (options, subject, risk) | ACP request_permission options, CX ExecApprovalRequest/ApplyPatchApprovalRequest, GS ToolConfirmationRequest/ActionRequired | `Approval{subject, options, reason}` |
| Images (user, tool, generated, view_image) | ACP Image block, MCP image, CX ImageView/ImageGeneration, GS Image | `Image{mime, blob_ref}` |
| Tables | markdown only today (GB per-cell selection in TUI) | parse GFM tables to a real grid |
| Mermaid / diagrams | GB pure-Rust mermaid→PNG | render inline, cache by hash |
| Citations / web search results / resource links | CX WebSearchBegin/End, MCP resource_link, ACP ResourceLink | `Citation{uri,title,snippet}` |
| Structured tool output (JSON schema) | MCP structuredContent/outputSchema | collapsible tree / table |
| Subagent activity (nested transcript) | ACP unstable subagent_update, CX Collab*, CC parent_tool_use_id, OC child projection | nested session card |
| Compaction marker (+ summary viewable) | ACP compaction_update/summary_chunk, CX ContextCompacted, OC compaction.* | `Compaction{summary_ref}` |
| Usage / cost / context gauge | ACP usage_update, CX TokenCount | status bar |
| Elicitation forms | MCP elicitation form/url, ACP elicitation/create | native form |
| HTML app panels | MCP Apps `ui://` (GS proxies) | sandboxed webview, opt-in |
| Errors/notices/retries | OC retry.scheduled, GB Retrying, ACP notice | inline banner |

Best-typed today: **ACP** (tool kinds/status/diff/terminal/plan) and **Codex TurnItem** (richest per-kind items). Model internal events as Codex-style items, export losslessly to ACP.

### 11.4 Ideas to steal (ranked)

1. **Typed item lifecycle with patch-by-id** (started/delta/completed) — CX `protocol/src/items.rs:50`; ACP v2 draft. One model for GUI cards, ACP, JSONL, persistence.
2. **Durable vs ephemeral events + replay `after=seq`** — OC `schema/src/event.ts:42`, `groups/session.ts:652`, `server/src/event-feed.ts:8`. Frontends crash and reconnect without engine cooperation.
3. **Engine headless, every UI is a client of the same protocol, in-proc and remote via one trait** — CX `app-server-client/src/lib.rs:5-19`; GB in-proc ACP channels `xai-acp-lib/src/channel.rs:10`.
4. **Native ACP adapter from day one** — GB, GS, OC, OMP all speak it; free Zed/JetBrains integration.
5. **Dispatch tools mid-stream + RwLock concurrency classes + per-path write locks** — CX `stream_events_utils.rs:316`, `tools/parallel.rs:286`; GB `tool_calls.rs:623`.
6. **Steer vs follow-up queues; interjections only at safe points** — PI `agent/src/agent.ts:308-313`; GB `interjection.rs:303`; CX `turn/steer`.
7. **Prefire two-pass compaction + compacted segments on disk the model can reread** — GB `two_pass.rs`, `xai-compaction-transcript/src/lib.rs:15`.
8. **Cheap compaction rungs before LLM summary** (clip tool output → drop → tool-pair summary) — OC `compaction.ts:145`; GS tool-pair; OMP shake.
9. **Truncate live context, keep full payload in log; spill big outputs to files** — CX `context_manager/history.rs:531`; OC `tool-output.ts:13`; GS `large_response_handler.rs`.
10. **Edit strategy per model, multiple formats behind one diff tracker** — GB tool namespaces `types/tool.rs:29`; OMP `edit/settings.ts`; AD per-model edit_format. Include hashline (stale-anchor rejection) and Codex apply_patch grammar.
11. **On apply miss, reflect with "did you mean" context instead of silent fuzzy** — AD `editblock_coder.py:92-125` (fuzzy deliberately disabled `:183`).
12. **Shadow-git snapshot per step → revert/rewind** — OC `snapshot.ts:92`; GB rewind points.
13. **JSONL tree session (id/parentId) for branch/fork/rewind** — PI `session-manager.ts:41`.
14. **Single persistence actor with written loss contract** — GB `persistence.rs:1-13`.
15. **Policy as data: Starlark/glob rules + amendments from approvals ("always allow this prefix")** — CX `execpolicy`, `ReviewDecision::ApprovedExecpolicyAmendment`; OC `Unattended` ruleset.
16. **Inspector pipeline (permission, injection, egress, repetition)** — GS `tool_inspection.rs:57`; GB stationarity/doom-loop.
17. **Cross-provider transcript transform** — PI `transform-messages.ts:91`.
18. **Repo map via tree-sitter + personalized PageRank under a token budget** — AD `repomap.py:365-703`.
19. **Hooks compatible with Claude Code names and exit-2 semantics; plugins accept `.claude-plugin`** — GB `xai-grok-hooks/src/event.rs:78`, `plugins/manifest.rs`. Ecosystem reuse for free.
20. **Model roles** (default/smol/plan/commit/vision/advisor) — OMP `~/.omp/agent/config.yml`; AD weak model for commit messages.
21. **Streaming markdown with stable-prefix re-render; pure-Rust mermaid** — GB `xai-grok-markdown/src/streaming.rs`, `xai-grok-mermaid`.
22. **Deferred tool loading via search (BM25) for big MCP catalogs** — GB `tool_index.rs`; PI MCP exposure modes.

### 11.5 Anti-patterns to avoid

1. **Unbounded event queue from engine to UI** — CX `session/mod.rs:671` `async_channel::unbounded()`; a stalled consumer grows memory. Use bounded per-subscriber + lag/replay.
2. **Agent loop delegated to an SDK step loop** — GC `streamText({stopWhen})` (`agent.ts:1324`); you lose cancellation granularity, steering, mid-stream dispatch.
3. **No permission model / yolo default** — PI (by design), OMP `approval.ts:110` default yolo. Ship ask-by-default with fast "always allow" amendments and a sandbox.
4. **Built-in tools outside the sandbox** — CC docs: file tools, MCP, hooks run outside the bash sandbox. Put every side effect behind one policy gate.
5. **Silent fuzzy edits** — risk of wrong-place edits; AD disabled its own. Fuzzy only with unique match + reported normalization.
6. **Per-kind legacy events alongside items** — CX keeps `legacy_events.rs` shims; pick the item model once.
7. **Single giant loop function** — GB `turn.rs` loop spans thousands of lines; OMP `agent-loop.ts` 4119 lines. Keep the turn runner a small state machine (GS composable ops is the better shape).
8. **Provider lock-in at the wire level** — CX removed chat completions (`model-provider-info/src/lib.rs:109`). Keep a provider trait with multiple wires.
9. **TUI/UI owning session state** — avoid; GB/CX moved state into the engine and made the TUI a client.
10. **Process-global sandbox applied once at startup** — GB nono at startup; prevents per-tool profiles. Prefer per-spawn sandbox (CX SandboxManager::transform).
11. **Dual stores without a single source of truth** — CX rollout JSONL + sqlite state + thread-store. Make the event log canonical; everything else is a projection.
12. **LSP/diagnostic feedback missing after edits** — OC TODO (`edit.ts:258`); GS none. Feed formatter + diagnostics back as tool output.
13. **Classifier-only safety** (LLM judge as the gate) — GS SmartApprove, GB/CC auto mode. Fine as an advisor; hard gates must be rules + sandbox.

### 11.6 Recommended first cut for bboxy450-harness (smallest thing that proves the architecture)

1. `harness-protocol`: Op, Event envelope `{seq, session_id, item_id?, durable: bool, payload}`, Item kinds from 11.3; schemars export.
2. `harness-engine`: SessionActor (bounded inbox), TurnRunner (CancellationToken tree, JoinSet), one provider (Anthropic Messages) + one OpenAI Responses, tools read/write/edit/bash with concurrency classes and path locks.
3. `harness-store`: append-only JSONL tree + sqlite projection; replay `after_seq`.
4. `harness-gateway`: in-proc + UDS; subscribe/replay/lag.
5. Frontends: headless JSONL first (tests), then GUI, then ACP adapter, TUI last.
6. TLA+ models: actor op ordering, cancel tree, approval lifecycle, subscriber replay — each with a broken control.
