---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
id: 01M6AVX3PM1GNYFG7D52X7KMYS
links:
- rel: index
  to: '[[interfaces]]'
- rel: carries
  to: '[[protocol]]'
- rel: fed-by
  to: '[[acp]]'
- rel: constrains
  to: '[[policy-gates-effects]]'
- rel: constrains
  to: '[[secrets-never-persist]]'
- rel: constrains
  to: '[[never-block]]'
- rel: depends-on
  to: '[[seat-capability-delivery]]'
number: 1988
provenance:
  digest: sha256:b149e6505461ef6937f0cff7d5ed5fd5ed0f8bf45627da7be4cb683fbaffd2e8
  frontmatter_digest: sha256:22870445b37a56d21696243ad0e8a607895760cf7404c4d5ed54e43966688781
  log: .bboxy/log/2026-11-01.jsonl#28373
  sig: /fA9r99Uk4DpFBkBOCAb2mONgxG+NeIZt620dcMNnzOkdoeiLA4Q+Pc7sSS0HF+0okZ2njD5rIO2o3dKp2vyCA==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: baselined
title: 'ICD: MCP client — session-lived servers, registry, policy, degradation (v0.1)'
type: interface
updated: 2026-11-01T13:46:41.298Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# ICD: MCP client

| | |
|---|---|
| Side A | the engine: `src/mcp.rs` (E1c), the tool `Registry` (`src/tools.rs:41`), `Policy::decide` (`src/policy.rs:76`), the redactor (`src/redact.rs`) |
| Side B | any MCP server the user configures; it is untrusted code in a child process |
| Pin | `rmcp = "=3.5.1"`, client side, stdio transport only (`TokioChildProcess`, `rmcp/src/transport/child_process.rs:36`). Handshake version `LATEST_WITH_INITIALIZE` = `2025-11-25` (`rmcp/src/model.rs:201`); the stateless `2026-07-28` is not used yet. HTTP and SSE servers are not supported in v0.1. Moving the pin is an ICD change |

## 1. Configuration and lifetime

- **Sources.**
  - The profile's `mcpServers: {<name>: {command, args, env, timeout_s, trust_hints}}` (kittens keys, plus the two harness keys).
  - ACP `session/new` stdio `mcpServers` ([[acp]] §1).
  - `seat --mcp-config <path>`: a JSON file `{"mcpServers": {...}}` with the same keys, written by the bboxy450-term launch kit ([[embed-seat]] §1). These are *launcher servers*.
  - Same name in more than one source: launcher wins over ACP, and ACP wins over the profile, with one `Notice{Info}`.
  - At most 16 servers per session; the rest are ignored with a `Notice{Warn}`.
- **Lifetime = the session.** Servers are spawned when `Host::create` or `Host::open` returns, on the session's `TaskGroup` (law [[cancel-is-cancel]]). Each runs in its own process group.
  - `Op::Close` stops every server: the process group gets SIGTERM, then SIGKILL after 1 s.
  - The face exits, or the process exits, with the same effect.
  - Never per turn (kittens `src/agent.rs:268` is the rejected design).
- **States** per server: `starting → ready`, `ready → failed → starting` (restart), `failed → dead`.
  - Restart backoff is 1 s · 2ⁿ, capped at 30 s.
  - **Binding count: 5 failed attempts in one session make the server `dead`.** That is the initial attempt plus at most 4 restarts. A failure is a failed start or an exit after `ready`. It stays `dead` until the session is reopened.
  - Each restart increments the server's `generation`.
- **Startup is never awaited.** `initialize` plus `tools/list` has a 10 s deadline, and that deadline only marks the server `failed`. No session start, round or turn waits for a server.

## 2. Tools in the registry

- **Snapshot per turn.** The tools offered to the model are fixed at `TurnStarted`: the builtins first, then each `ready` server's tools, sorted by server name and then tool name.
  - A server that becomes ready, fails or sends `notifications/tools/list_changed` mid-turn changes the *next* turn's snapshot. The re-list is debounced by 1 s.
  - The tool list therefore stays inside `stable_prefix` for the whole turn ([[provider]] §2 rule 6).
- **Name.**
  - Format: `mcp__<server>__<tool>`, with every character outside `[A-Za-z0-9_-]` replaced by `_`, cut to 64 characters.
  - A name collision keeps the first in sort order and drops the rest, with one `Notice{Warn}` each.
  - At most 128 MCP tools per snapshot; the rest are dropped with one `Notice{Warn}`.
- **Classes** (`ToolAnnotations`, `rmcp/src/model/tool.rs:51-92`):

  | Hint | `Concurrency` | `EffectClass` for policy | `ToolKind` |
  |---|---|---|---|
  | `readOnlyHint: true` | `ReadOnly`: may run beside other read-only tools, within the 4-way limit | `ReadOnly` **only if** the server has `trust_hints: true`, else `Exec` | `Read`, or `Other` when the server is untrusted |
  | anything else | `Exclusive` | `Exec`, or `Network` if `openWorldHint: true` | `Other` |

  Hints are claims by untrusted code. They may lower *concurrency*, where a lie only races the liar. They never lower *policy* without the user's explicit `trust_hints`.
- **Every call goes through `Policy::decide(Action{tool: <full name>, effect, generation})`**, exactly like the builtins. `Ask` creates an `Approval` item; `Deny` fails the item with the rule's reason. There is no MCP-specific bypass (law [[policy-gates-effects]]).

## 3. Calls

- **Item.** A `ToolCall` item with `head{call_id, name, tool_kind}` (protocol §4).
- **Result mapping.**
  - Text content → `OutputBlock::Text`.
  - Image content → a blob.
  - `structuredContent` → pretty JSON text.
  - `isError: true` → `ItemStatus::Failed{reason: <first text line>}`.
  - Output over 50 KiB is spilled to a blob ([[budgets]]).
- **Timeout.** Per server `timeout_s`, default 60, sent through `PeerRequestOptions.timeout` (`rmcp/src/service.rs:772`). On expiry:
  - the request is cancelled (`notifications/cancelled`);
  - the item ends `Failed{reason: "mcp <server>: timed out after <n> s"}`;
  - one `Notice{Warn}` is appended.

  The turn continues, and the model sees the failure as the tool result.
- **Interrupt.** It cancels in-flight calls the same way, so the 2 s rule holds. A late response after cancellation is dropped.
- **Generations.** A response from an earlier server generation is dropped. A call that was in flight when its server failed ends `Failed{reason: "mcp <server>: server exited"}`.
- **Calls to a server that is not `ready`** (possible within a turn's fixed snapshot) fail at once with `Failed{reason: "mcp <server>: <state>"}`. They never wait for a restart.

## 4. Degradation and labels

- **Every state change is one `Notice`,** with text `mcp <server>: <state>[: <reason>]`. Examples: `mcp gh: ready (12 tools)`, `mcp gh: failed: exit 1`, `mcp gh: dead after 5 failures`.
- **Levels:** `Info` for ready, `Warn` for failed and restarting, `Error` for dead.
- **One per change.** A server flapping within the backoff produces one notice per state change, never one per call.
- **Stderr.** A server's stderr is kept in a 64 KiB ring per server and is never written to the log. The last 2 KiB is attached, scrubbed, to the `failed` notice.

## 5. Secrets

- **Env values.** Each `env` value is a literal, `$VAR`, or a kittens `!cmd` (10 s timeout, 8 KiB cap, [[provider]] §4).
- **No `BBOXY_*` delivery to profile or ACP servers.** A profile or ACP server never receives a `BBOXY_*` value. Launcher servers (§1 sources) are the only exception: they inherit `BBOXY_*` and may reference it. `BBOXY_CAPABILITY` is a seat credential, and a profile or ACP server is untrusted code. Each path is closed as follows (`src/mcp.rs` at eb384ac):
  - **Inheritance.** Every `BBOXY_*` variable is removed before spawn (`scrub_bboxy_env`, `:501-511`).
  - **A configured key named `BBOXY_*`.** The key is not set and the server spawns without it (`:571`).
  - **A `$BBOXY_*` or `${BBOXY_*}` reference in any `env` value.** **The whole server is refused**, with `ErrorClass::Config` `"reserved environment reference"` and one `Notice` (`validate_environment`, `:512-526`). The turn continues. Refusing is safer than spawning with a silently altered environment (Oracle ruling, message 01M6BNKJW0082VPSJ4YZF6HYAX).
  - **A `!cmd`.** The command runs with `BBOXY_*` removed from its environment. So `!printenv BBOXY_SOCK` fails with `"environment command failed"`, and the server is refused. If a resolved value contains the capability, the server is refused with `"reserved capability in environment"` (`:581`). The value compared is the one read from `BBOXY_CAPABILITY_FILE` ([[seat-capability-delivery]] §2 Side B). That reading is implemented in PR #18; until it merges, main still reads `BBOXY_CAPABILITY`, which Side A no longer sets, and the refusal is inactive.
- **Same-uid exception (what this does not claim).** On macOS, any process running under the same uid can read another process's exec-time environment with `ps -Eww -p <pid>`. This still works after the process has unset the variable; it was verified on 2026-11-01. So the rule above guarantees non-delivery, not secrecy. A profile server, or any tool any seat runs, can still find the capability in the environment of the harness or of its parent seat. Until [[seat-capability-delivery]] takes the capability out of the environment, this ICD does not claim that the capability is hidden from same-uid code. That proposal also does not defend against an active same-uid attacker who reads files.
- **Registration.** Every resolved value is registered with the redactor `Set` (`src/redact.rs:36`) **before** the child is spawned.
- **Scrubbing.** Tool arguments are scrubbed before they are written; results, `Notice` text and stderr excerpts are scrubbed before they enter any item (law [[secrets-never-persist]]).
- **Profiles.** The profile file stores the reference (`$VAR`, `!cmd`), never the resolved value.

## 6. Verified by

| Clause | Evidence |
|---|---|
| §1 lifetime | `tests/mcp_client.rs` against a fixture server binary (`tests/bin/mcp_fixture.rs`, rmcp server side, with modes `normal`, `slow`, `crash`, `hang_init`, `list_changed`, `liar`, `echo_env`). `Op::Close` leaves no fixture process alive (checked by process group); `crash` gives exactly 5 spawns (the initial one plus 4 restarts), then `dead` and no sixth spawn |
| §1 never awaited | with `hang_init`, a scripted turn starts and completes; the server is absent from the snapshot; a `failed` notice appears after 10 s without delaying any envelope |
| §2 snapshot, names, classes | `list_changed` mid-turn changes only the next turn's tools; `messages[..stable_prefix]` stays byte-identical within a turn; `liar` (readOnlyHint on a writer) gets `ReadOnly` concurrency but reaches `Ask` under the default policy, and with `trust_hints: true` it is allowed; names over 64 characters and collisions follow §2 |
| §3 calls | `slow` gives `Failed` with the timeout reason within `timeout_s` + 1 s, and the turn continues; an `Interrupt` during `slow` gives `TurnEnded{Cancelled}` in ≤ 2 s; a response from the previous generation after a crash is dropped |
| §4 labels | the notice count equals the number of state changes in each mode |
| §5 secrets | `echo_env` returns its secret env value; the log, the notices and the stderr excerpt contain no copy (`grep -c` = 0 over the session directory) |
| §5 no `BBOXY_*` delivery | existing tests in `tests/support/seat_launch.rs` (PR #8, #11). Inheritance: `profile_server_cannot_inherit_bboxy_environment`, `acp_server_cannot_inherit_bboxy_environment`. The launcher exception, and a configured `BBOXY_TEST_CONFIGURED` key that reaches only the launcher server: `launcher_flags_deliver_context_tools_and_only_launcher_environment`. `$`/`${}` references refused with a typed configuration refusal, for profile and ACP servers: `profile_and_acp_refuse_bboxy_environment_indirection`. The `!cmd` resolver scrub: `command_resolver_cannot_copy_seat_environment_into_mcp_aliases` |
| §5 same-uid exception | not verified here: it is a stated non-claim. The removal check is [[seat-capability-delivery]] §4 |
| model | **HarnessMcpServer**, below |

**HarnessMcpServer**, adapted from PluginConnection (generations A→B→A, as in HarnessCancelTree).
- State: `state ∈ {starting, ready, failed, dead}`, `gen 0..2`, `failures 0..2`, `call ∈ {none, inflight(g)}`, `round_open`.
- Safety:
  - `NoStaleResult`: a result is accepted only if `g = gen`.
  - `RoundNeverWaits`: `StartRound` is enabled in every server state.
  - `DeadIsFinal`: once `dead`, a server never returns to `starting` within the session.
- Liveness: `inflight ~> none` under WF{timeout, respond}.
- Broken controls:
  - (a) `StartRound` requires `ready`: **deadlock** under `hang_init`;
  - (b) accepting a result from any generation: **safety**;
  - (c) no timeout action: **liveness**.
- The model's bound is `failures 0..2` (dead at 2). The `failures = limit → dead` transition is the same; the 5 in §1 is a parameter, and the implementation test checks it.

## Change log

- **2026-11-01, v0.1 baselined at the Oracle's request** (message 01M6AVJ56YFJSKHDJ9Q6Z17QNH). Alternatives rejected:
  - servers per turn (kittens): spawn cost on every turn, and a tool list that changes the cache prefix each turn;
  - trusting `readOnlyHint` for policy: an untrusted server could auto-approve its own writes;
  - HTTP and SSE transports in v0.1: they need auth and OAuth, which is a separate secret path;
  - waiting for servers at session start: one hung server would block every turn.
- **2026-11-01, at E1's question** (message 01M6AWED4QVM27ZVE6ZHP7WY5D). §1 and §6 disagreed on the failure count. Binding: 5 failed attempts (the initial one plus 4 restarts), then `dead`. §6 now asserts 5 spawns and no sixth. Version unchanged.
- **2026-11-01, at the Oracle's embed-seat request** (message 01M6BGD5FMZ55AJ4V6SS4Q11KK). §1 adds launcher servers from `seat --mcp-config`. §5 states environment inheritance, which the ICD had not specified: servers inherit the process environment, but `BBOXY_*` is removed except for launcher servers. `src/mcp.rs` at cdde43d inherits everything, so a profile server could read the seat capability; that is a defect against this line. Alternative rejected: clearing the whole environment, which breaks `PATH`/`HOME` for ordinary servers. Version unchanged.
- **2026-11-01, at the Oracle's security finding** (message 01M6BN8XAGAW3C6AGHYZY3NWRK, from the PR #11 review). §5 changes in two ways.
  - Non-delivery now covers all three paths: inheritance, config keys, and resolver references. The resolver path was open: `$BBOXY_CAPABILITY` in a profile `env` would have been resolved.
  - The same-uid exception is stated, so the ICD claims nothing it cannot enforce. I re-verified it on this machine: `ps -Eww` shows a child's exec-time environment even after the child unsets the variable.

  The removal is proposed in [[seat-capability-delivery]]. Version unchanged.

- **2026-11-01, at the Oracle's ruling** (message 01M6BNKJW0082VPSJ4YZF6HYAX). The previous entry said a dropped `$BBOXY_*` reference produces a `Notice` and the server still spawns. That was wrong: the accepted code at eb384ac (PR #11, task #4344) refuses the whole server with `ErrorClass::Config`, and the code is right. §5 now describes the four paths exactly as eb384ac implements them, with line references. The `!cmd` path is closed by running the resolver with `BBOXY_*` scrubbed, not by matching the output. §6 cites the existing `seat_launch.rs` tests instead of proposing new ones. Version unchanged. (Edited directly while the knowledge transport was unavailable; the record's signature is renewed through `knowledge_write`.)

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: configured MCP servers (E1c implements the client).
