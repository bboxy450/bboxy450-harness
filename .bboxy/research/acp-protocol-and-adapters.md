---
authorship:
  kind: seat
  process_generation: '1'
  public_key: atGZZsv4YXvtG3Kx8WGrNL3NVI51UpOUXvRyj/KiL28=
  signer: seat:01M6ERRV5C70R0BRT8DXVRY9VF
created: 2026-11-02T20:08:05.159Z
created_by: seat:01M6ERRV5C70R0BRT8DXVRY9VF
id: 01M6EY0K5HJF7S4501BEZ4WEA9
links:
- rel: informs
  to: '[[2026-11-02-harness-acp-agent-backend]]'
- rel: context
  to: '[[acp]]'
number: 5704
provenance:
  digest: sha256:91b8923044898e2ea3cbf1f072be5addaf0d326e2f114257af23156a2c674ef7
  frontmatter_digest: sha256:9f3d1e655bc6cb69d5d7bee1f7fa1f600a0819bc09773231c20912c8333f4c98
  log: .bboxy/log/2026-11-02.jsonl#37994
  sig: WQOa7bbKwsZuLzE14DN7ANYk/kAgQlloNinAR/XpT5N7xgJliTfURdCkItSPousobbNwAdbHBtLEBrEGHV6aAg==
  signer: seat:01M6ERRV5C70R0BRT8DXVRY9VF
status: reported
title: 'ACP (Agent Client Protocol): protocol, Rust SDK and agent adapters, as of 2026-11-02'
type: research
updated: 2026-11-02T20:08:05.159Z
updated_by: seat:01M6ERRV5C70R0BRT8DXVRY9VF
---
# ACP: protocol, SDK and adapters (2026-11-02)

This is research input for task #5686 (ACP agent backend). It is the Agent Client Protocol by Zed (agentclientprotocol.com), not IBM/BeeAI's "Agent Communication Protocol". The sources are current web pages, fetched 2026-11-02. Items marked UNVERIFIED could not be confirmed. Treat this as a pointer set and re-read the spec pages before coding.

## Corrections to #5686 (read first)

The Executor applies these over the task brief:

1. **Crate pin.**
   - Our pin is `agent-client-protocol =3.4.0`. The latest is **5.0.0, released 2026-11-02**, and it is breaking: schema 1.15, `Builder::new`, `v1::schema`/`v2::schema`, rmcp behind `rmcp_3`, batch removed, and dropped responders now answer with an error.
   - Phase 1 stays on 3.4.0 so the face and backend share one version. The 5.x upgrade is its own later task, with the v5 migration guide. https://github.com/agentclientprotocol/rust-sdk/releases
2. **Spawning.**
   - In 3.4 the subprocess helper (`AcpAgent`, `AcpAgentConfig`) sits behind a separate `process` feature; our pin has only `stdio`.
   - Phase 1 spawns through `crate::process::OwnedChild` and hands its pipes to the `Stdio` transport, so no new feature is needed.
   - The API shape is `Client.builder().name(..).connect_with(transport, async |cx| ..)`. `ClientSideConnection` is gone. https://docs.rs/agent-client-protocol/3.4.0/agent_client_protocol/
3. **Sessions.**
   - Add `session/resume` (restore without replay) next to `session/load` (replays history as updates before responding). Gate strictly on `sessionCapabilities`.
   - `session/close` is idempotent: treat `-32002` as success. https://agentclientprotocol.com/protocol/session-setup
4. **Model and mode.**
   - There is no `session/set_model` in stable. A model is a `SessionConfigOption` with category `model`, set through `session/set_config_option`.
   - Map the profile `model:` to that option when advertised; otherwise surface a notice. https://agentclientprotocol.com/protocol/schema
5. **Updates.**
   - There are 14 `session/update` variants. Beyond the brief's list they include `config_option_update`, `session_info_update`, `usage_update` (→ `Usage`), `user_message_chunk` (load replay), and the cap-gated `notice` / `compaction_update` / `compaction_summary_chunk`.
   - Phase 1 maps text, thought, tool_call/update, plan and usage. Everything else becomes a `Notice`.
   - Stop reasons also include `max_turn_requests` (→ Length). https://agentclientprotocol.com/protocol/prompt-turn
6. **Cancellation contract.**
   - After `session/cancel`, the client **must** answer every pending `request_permission` with `{"outcome":"cancelled"}` and keep accepting updates.
   - The agent ends the prompt with `stopReason: cancelled`, not an error. This belongs in the formal model.
7. **Unsupported agent→client requests.** Reply `-32601`; an unanswered request wedges the turn. In phase 1 that covers fs/*, terminal/* and elicitation/*.
8. **Permission options.** `allow_once | allow_always | reject_once | reject_always`. `on_unattended: deny` → `reject_once`. Never pick `*_always` automatically.
9. **Auth.**
   - `authMethods` entries are of type `agent` (call `authenticate{methodId}`) or `terminal`. For `terminal`, the client re-spawns the agent with the method's args/env (no shell), exit 0 means success, then it re-initializes. It requires the client cap `auth.terminal`.
   - Phase 1 advertises no `auth.terminal`. On `auth_required`, it shows a notice telling the user to authenticate the agent in a terminal. https://agentclientprotocol.com/protocol/authentication
10. **Compliance (binding).**
    - Anthropic's Agent SDK docs say: "Unless previously approved, Anthropic does not allow third party developers to offer claude.ai login or rate limits for their products, including agents built on the Claude Agent SDK. Use the API key authentication methods described in the Quickstart instead." https://code.claude.com/docs/en/agent-sdk/overview
    - The `claude-agent-acp` adapter advertises a `claude-ai-login` ("Claude Subscription") terminal method anyway.
    - **Our harness never offers, selects or triggers `claude-ai-login`.** For Claude via ACP, the only supported paths are `console-login`, `ANTHROPIC_API_KEY`, Bedrock or Vertex.
    - The same page bans "Claude Code" branding for third-party agents: use "Claude Agent".
    - This replaces #5686's phase-2 note about folding #5205 in via claude-code-acp "with whatever Claude login it finds".

## Protocol core

- **Transport.** JSON-RPC 2.0 over stdio, one UTF-8 JSON message per line with no embedded newlines. stdout carries only ACP; stderr is logs. The client spawns the agent and shuts it down by closing stdin, then terminating it. HTTP transport is still an RFD. https://agentclientprotocol.com/protocol/transports
- **Version.** `protocolVersion` is an integer, currently `1`. The agent echoes ours or returns its own latest; on a mismatch we close and say so. A v2 draft exists. https://agentclientprotocol.com/protocol/initialization
- **Capabilities.** Omitted means unsupported.
  - Client: `fs.readTextFile`/`writeTextFile`, `terminal`, `auth.terminal`, `elicitation`, `session.configOptions|compaction|notices`.
  - Agent: `loadSession`, `promptCapabilities.{image,audio,embeddedContext}` (`text` and `resource_link` are baseline), `mcpCapabilities.{http,sse}`, `auth.logout`, `sessionCapabilities.{resume,list,close,delete,additionalDirectories}`.
  - `clientInfo` / `agentInfo` are `{name,title?,version}`; send ours.
- **Agent methods.** initialize, authenticate, logout, session/new|load|resume|prompt|set_mode|set_config_option|list|close|delete, and the notification session/cancel.
- **Client methods.** session/request_permission, session/update (notification), fs/read_text_file|write_text_file, terminal/create|output|wait_for_exit|kill|release, elicitation/create|complete. Protocol-level: `$/cancel_request`. https://agentclientprotocol.com/protocol/schema
- **Unstable (draft; do not depend on).** session/fork|set_title|status|archive, providers/*, mcp/message, auth/status, `systemPrompt`. https://agentclientprotocol.com/protocol/draft/schema
- **Content blocks.** text, image, audio, resource_link, resource.
- **Tool calls.**
  - kind: read, edit, delete, move, search, execute, think, fetch, switch_mode, other
  - status: pending, in_progress, completed, failed
  - content: `content`, `diff{path,oldText?,newText}`, `terminal{terminalId}`
  - `locations[{path,line?}]` are display-only

  https://agentclientprotocol.com/protocol/tool-calls
- **Terminals (phase 2).** `create` returns an id immediately; `output` returns `{output,truncated,exitStatus?}`. `kill` keeps the id, while `release` kills and frees it. The agent must release. https://agentclientprotocol.com/protocol/terminals
- **Extensibility.** `_meta` on any type and `_`-prefixed custom methods. Keys are camelCase; discriminators are snake_case. https://agentclientprotocol.com/protocol/overview

## Agents and adapters (from the official registry)

The registry is `https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json`, with top-level `{version, agents[], extensions[]}`. Each entry carries `id, name, version, description, repository, license, icon, distribution`. `distribution` is `npx{package,args,env}`, `uvx{…}` or `binary{<os-arch>:{archive,cmd,args,env,sha256}}`. Entries are validated against `agent.schema.json`, and 44 agents are listed. https://agentclientprotocol.com/get-started/registry

| Agent | Launch | Auth | Notes |
|---|---|---|---|
| Claude Agent (`agentclientprotocol/claude-agent-acp`, formerly zed-industries/claude-code-acp) | `npx @agentclientprotocol/claude-agent-acp@0.100.0` | terminal methods `claude-ai-login` (**never offer**) / `console-login`; `ANTHROPIC_API_KEY`; gateway | Apache-2.0; load/resume/list/close/delete/fork |
| Codex (`agentclientprotocol/codex-acp`) | `npx @agentclientprotocol/codex-acp@2.5.0` (`CODEX_PATH`) | ChatGPT login, `CODEX_API_KEY`/`OPENAI_API_KEY` | Apache-2.0. The OpenAI-login policy was not researched (UNVERIFIED) |
| Gemini CLI | `npx @google/gemini-cli@0.66.0 --acp` | Gemini login (UNVERIFIED detail) | `--experimental-acp` is legacy and fails on new builds |
| goose | `goose acp` | its configured provider | 1.57.0 |
| OpenCode | `opencode acp` | its provider config | 2.2.5 |
| Auggie | `npx @augmentcode/auggie@0.36.0 --acp` | Augment account | |
| Qwen Code | `npx @qwen-code/qwen-code@0.26.5 --acp --experimental-skills` | not checked | |
| Kimi CLI | `kimi acp` | not checked | |
| Cursor | `cursor-agent acp` | Cursor account | Reported to never send `request_permission` and to mark rejected tools `completed` |
| GitHub Copilot | `npx @github/copilot@1.0.105 --acp` | GitHub login | |
| fast-agent | `uvx fast-agent-acp==0.10.1 -x` | provider keys | Python |
| Junie | `junie --acp=true` | JetBrains account | |
| Mistral Vibe | `vibe-acp` | Mistral account | Apache-2.0 |

Also listed: Amp, Cline, Devin, Factory Droid, Kilo, Poolside, Qoder, Grok Build, Google Antigravity. **Clients:** Zed, JetBrains, Emacs agent-shell, Neovim (CodeCompanion, avante, agentic.nvim), Obsidian, marimo, Jupyter, VS Code extensions, Qt Creator, Sublime. https://agentclientprotocol.com/get-started/clients

## Implementation gotchas

- **stdout pollution.** Stray prints corrupt the framing. Treat a parse failure as an adapter bug (Notice + turn Error) and bound line sizes.
- **Capability variance.** Session load/resume/neither differs per agent; never call an unadvertised method.
- **MCP passthrough.** stdio servers are mandatory for agents to support; http/sse are cap-gated. Do not forward stdio servers that came from workspace files or agent output without the user's approval. Never echo `env`/`headers` values (phase 2).
- **Pinned launchers.** The registry pins exact versions (`npx pkg@x.y.z`); keep the version in the profile and record it in the ICD for reproducibility. Registry install is a phase 2 discovery feature with sha256-checked binaries. No auto-update in tests.
- **Licensing.** Adapters and the Rust SDK are Apache-2.0. Claude Agent SDK use falls under Anthropic Commercial Terms.
- **Windows.** Neither the spec nor the registry covers Windows-specific stdio behaviour (UNVERIFIED); this harness is macOS-first.

## Feeds
[[2026-11-02-harness-acp-agent-backend]] (#5686): phase 1 implements under corrections 1–10. Phase 2: config options, terminal auth (non-Claude agents only), fs/terminal caps, registry discovery, and the 5.x SDK migration.
