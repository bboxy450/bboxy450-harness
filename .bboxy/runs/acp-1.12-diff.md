# ACP 1.9.1 → 1.12.0 pin check

Task #1992. Completed before production-code changes. Branch
`e1c-acp-mcp-compaction`, based on origin/main `47e2e98` (includes `c745ad9`).

Sources: immutable crates.io schema packages 1.9.1 and 1.12.0 extracted under
`target/acp-schema-check/`; cached agent-client-protocol 3.4.0 source; official
[SDK dependency listing](https://docs.rs/crate/agent-client-protocol/3.4.0) and
[schema 1.12.0 v1 sources](https://docs.rs/agent-client-protocol-schema/1.12.0/src/agent_client_protocol_schema/v1/mod.rs.html).

- schema 1.9.1 crate SHA256: `1d595a79e1665d02c91c7dd1bedd118501e75cc0a3d5b1651b5150ec4c5c1e0d`
- schema 1.12.0 crate SHA256: `430e900fdb40a811c1a3e0d85b1134e485d96699f8b90889a15c1da6a0b26c3f`

| ICD reliance | 1.12.0 source | Finding / effect |
| --- | --- | --- |
| SDK and protocol pin | SDK Cargo.toml; schema lib.rs | SDK =3.4.0 depends on schema =1.12.0. V1 remains 1. No unstable feature is needed. SDK defaults are empty; native stdio is an explicit feature. |
| initialize and advertised capabilities | v1/agent.rs InitializeRequest/Response, AgentCapabilities, PromptCapabilities, McpCapabilities, SessionCapabilities | Existing field names, booleans and close={} remain. auth is a defaulted capability field in both versions; omit it in the deliberately restricted advertised object. HTTP/SSE false booleans serialize explicitly. |
| new/load/prompt/cancel/close | v1/agent.rs NewSessionRequest, LoadSessionRequest, PromptRequest, CancelNotification, CloseSessionRequest | Stable v1 methods remain. session IDs remain strings; cwd remains PathBuf. mcpServers stdio remains available without unstable features. |
| unsupported method policy | v1/agent.rs method-name constants and request enums | set_mode, set_config_option, list, resume, delete, logout still exist as protocol methods but remain deliberately unsupported by the ICD. Removed NES/document experimental types do not change returning -32601. New systemPrompt, subagent and session-message draft types are gated and will stay disabled. |
| prompt content | v1/content.rs ContentBlock | text/image/audio/resource_link/resource tags and payload fields remain. Image still uses data and mimeType; resources still carry a URI. Unsupported-content degradation can stay unchanged. |
| stop reasons | v1/agent.rs StopReason | end_turn, cancelled, max_tokens, max_turn_requests, refusal unchanged. Harness errors still use JSON-RPC -32603 and the harness does not emit ACP refusal. |
| chunks and suffix repair | v1/client.rs SessionUpdate, ContentChunk | user_message_chunk, agent_message_chunk and agent_thought_chunk retain the same content shape. No change to append/suffix repair. |
| tool updates, IDs and status | v1/tool_call.rs ToolCall/Update, ToolKind, ToolCallStatus | toolCallId/title/kind/status/rawInput/content/locations remain. All harness kinds have matching outbound values; ACP also has switch_mode, which this face never emits. Existing in_progress/completed/failed mapping is valid. |
| diff/terminal degradation | v1/tool_call.rs ToolCallContent, Diff, Terminal | Diff still needs old/new text and Terminal still references a client-created terminal ID. Emit text content for harness unified diffs and owned terminal output as specified. |
| plan | v1/plan.rs Plan, PlanEntry, PlanEntryPriority/Status | Stable full plan replacement and low/medium/high; pending/in_progress/completed remain. Harness cancelled entries still degrade to completed with the content prefix. |
| permission request and selected/cancelled | v1/client.rs RequestPermissionRequest/Response, PermissionOption/Kind, RequestPermissionOutcome | The request still carries sessionId/toolCall/options. optionId and allow_once/allow_always/reject_once are unchanged. The response has an outer outcome field whose value is the tagged selected/cancelled object (the ICD shorthand describes this inner object). |
| JSON-RPC and lifecycle | SDK jsonrpc/, schema Error/ErrorCode | JSON-RPC 2.0, request/notification distinction and codes -32700/-32601/-32602/-32603/-32002 remain compatible. The face still owns shutdown and the 256 KiB outbound cap. |

Actual differences relevant to validation, without changing the mapping:

- New/load MCP arrays, stdio args/env and additionalDirectories switch from
  tolerant DefaultOnError/VecSkipError to DefaultOnNull. Valid requests map
  identically; malformed elements must produce invalid params rather than being
  silently discarded. Omitted fields follow their actual serde defaults; the
  face must not assume a required array is optional merely because a builder
  supplies an empty one.
- Authentication fallback now refuses a malformed terminal-auth object instead
  of reinterpreting it as agent auth. This face advertises no auth methods.
- Unstable NES/document support is removed, and new gated draft variants appear.
  All are outside the advertised method set and remain unsupported.
- Notice/usage/compaction update variants are present in the v1 schema; skipping
  them remains the ICD's explicit restricted mapping. Their presence does not
  authorize advertising or sending additional client capabilities.
- The SDK has a builder/typed-handler API in 3.4.0 rather than the old kittens
  Agent trait integration. Dispatcher/request ownership must still allow a
  pending prompt alongside cancellation and other sessions.

No valid-wire mapping change is required by the 1.12.0 pin. No Architect message
is needed for this pin comparison. Tests will validate the chosen advertised
object and emitted request/update values against the pinned stable schema.
