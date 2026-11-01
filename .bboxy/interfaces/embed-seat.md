---
id: "0PJA9WT3M12T7XGKMFWDPAD442"
number: 3
type: interface
title: "ICD: bboxy450-term ↔ harness — phase 1 process seat (proposed)"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "bboxy450-term:[[app-harness]]", rel: amends }
  - { to: "bboxy450-term:[[d20-seats-and-jobs]]", rel: constrained-by }
  - { to: "[[architect-review-v0]]", rel: decided-by }
created: "2026-10-31T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: bboxy450-term ↔ harness, phase 1 process seat

**Status: proposed.** This amends bboxy450-term ICD #348 (app ↔ harness), which needs its owner, Corvin Duskmantle <ARCHITECT>, to agree. It is not baselined until then. The harness work on phase 1 does not wait for that agreement: the `seat` face and its tests run standalone against a stub wake. See [[architect-review-v0]] §5.

| | |
|---|---|
| Side A | bboxy450-term: `bbt-mux` adapters and seat allowlist, `bboxy450-harness-wake-up` launch kit |
| Side B | `bboxy450-harness seat`, the line-mode face (E1) |
| Constraint | d20: a seat is a PTY with a screen. The harness in a seat therefore draws a human-readable screen; it never speaks ACP or JSON on the PTY |

## 1. What crosses

| Item | Direction | Mechanism |
|---|---|---|
| Launch | A → B | argv `bboxy450-harness seat [--resume <session>] --cwd <dir> --append-system-prompt <text> --mcp-config <path>`, under a PTY from the supervisor. An unknown flag fails the launch with its reason (§5) |
| Identity | A → B | env `BBOXY_SEAT_ID`, `BBOXY_GENERATION`, `BBOXY_ROLE`, `BBOXY_SOCK`, `BBOXY_CAPABILITY` (`wake-up/src/launch.rs:487-500`). The capability is never logged, persisted or shown (law [[secrets-never-persist]]) |
| MCP | A → B config, B → A calls | Side A writes the same `mcp.json` it writes for the Claude seat (`{"mcpServers":{"bboxy":{"command":<bboxy450-term exe>,"args":["mcp"]}}}`, in the 0700 launch overlay) and passes `--mcp-config <path>`. Side B adds those servers to the session ([[mcp]] §1). Their tools are named `mcp__bboxy__<tool>` ([[mcp]] §2), the same names a Claude seat sees. **Only `--mcp-config` servers inherit the `BBOXY_*` environment**; profile servers never receive it ([[mcp]] §5) |
| Startup context | A → B | `--append-system-prompt <text>`, where the text is the launch kit's `startup_prompt(context)`, the same text the Claude seat gets: NAME, ROLE, host/cwd and "call bboxy's orient MCP first". Side B appends it after its own system prompt, inside the stable prefix (provider §2 rule 6). It never contains the capability |
| Screen | B → A | Line mode (below) |
| Wake / input | A → B | Phase 1a: `WakeMethod::Pty` guarded input. One submitted line becomes `Op::UserTurn{Queue}`, and a line typed while a turn runs is also queued. Phase 1b (later): a native socket wake |
| State | B → A | Phase 1a: `StateSource::Cadence`, grade Inferred. This is the same honest grade pi/omp get today (`bbt-mux/src/adapters/mod.rs:80-93`) |
| Session id | B → A | Phase 1a: printed on the first screen line as `session <ulid>`. Phase 1b: registered through the hook action as `harness_session_id` |

## 2. Line mode: the definition of "not a TUI"

- Output is append-only UTF-8 lines. The only escapes allowed are SGR (colour and bold) and OSC 8 (links). There is no cursor addressing, no alt-screen, no clearing and no redraw. Native scrollback is the transcript.
- **It is a projection of the protocol, item by item.**
  - `AgentMessage` deltas are written as they arrive.
  - A `ToolCall` is one header line `▸ name args…` and one result line `✓|✗ <first output line> (n lines[, blob <sha256>])`.
  - `Reasoning` prints `thinking: <summary>`. `Notice` prints `<level>: <text>`.
  - Not shown: `Usage`, `TurnDiff`, `Image`, `Compaction`, `ContextTrim`, `Subagent`, `Unknown` (protocol v0.4). The log still holds them.
  - `Approval` prints its options and reads one key line.
  - `TurnEnded` prints `— reason` followed by the prompt `› `.
- Input is cooked line mode. `Ctrl-C` sends `Op::Interrupt` while a turn runs and does nothing at the prompt. `Ctrl-D` at an empty prompt sends `Op::Close`.
- The renderer is `src/frontends/seat.rs`, about 300 lines, and is a pure function `Envelope → Vec<u8>`. It is golden-tested against the protocol fixtures. It has no event loop of its own.

## 3. Lifecycle

| Event | Behaviour |
|---|---|
| Start | Print `session <ulid> · <profile>`, then the prompt; `profile <name> · <model>` follows on its own line when the profile resolves. The bboxy MCP connects asynchronously. If it is unavailable, print a `Notice` and continue (never block) |
| Restart (generation + 1) | Bboxy relaunches with `--resume <session>`. Store recovery runs (protocol §4), the screen replays the last 20 completed items, then the prompt |
| PTY hangup / seat close | Treated as `Op::Close`: cancel the turn, kill process groups, fsync, and exit 0 within 2 s |
| SIGTERM | Same as hangup |

## 4. Changes on Side A (core changes: they need the ICD #348 owner)

1. `bbt-mux/src/mcp/seats.rs:22`: add `"bboxy450-harness"` to `APPROVED_PROFILES`.
2. `bbt-mux/src/adapters/mod.rs:80`: add a record arm for `bboxy450-harness`, mirroring pi/omp, with `command = ["bboxy450-harness","seat"]`, `WakeMethod::Pty`, `StateSource::Cadence` and `hooks: false`.
2a. `crates/bboxy450-harness-wake-up/src/launch.rs` (kind match near `:510`): a `bboxy450-harness` branch that writes the overlay `mcp.json` (the `claude_wiring` MCP file only: no plugin and no settings) and appends `--append-system-prompt <startup_prompt(context)> --mcp-config <overlay>/mcp.json`. The env block (`:487-500`) is unchanged.
3. The settings role lists `settings/roles.rs:18` and `bbt-proto/src/gui/roles.rs:61` gain the kind.

There is no new `WakeMethod` and no change to d20 in phase 1a. Phase 1b adds a socket wake and a Reported grade through a separate amendment.

## 5. Failure

- The binary is missing: the launch refuses with its reason, the same as for any profile.
- The harness crashes: the seat shows an exited process, and restart follows §3.
- The guarded input is refused while the harness is not at its prompt: the readiness guard is the existing one. Because line mode accepts input at any time, the guard sees an always-ready reader and never needs to wait.

## 6. Verified by

| Clause | Evidence |
|---|---|
| §2 line mode | `tests/seat_golden.rs`: fixtures render to the expected bytes. A byte scan asserts there are no CSI sequences other than SGR (regex `\x1b\[[0-9;]*[^m]` has no match) |
| §3 hangup → stop within 2 s, no orphan process group | `tests/seat_process.rs` spawns `seat` under a real PTY (the `portable-pty` dev-dependency), starts a `sleep 600` tool through the scripted provider, closes the PTY, and asserts exit within 2 s and `kill(-pgid, 0) == ESRCH` |
| §1 MCP unavailable never blocks | `tests/seat_process.rs::no_bboxy_sock`: the env points at a missing socket, and the first turn still completes |
| §1 startup context and MCP (Side B) | `tests/seat_process.rs`: with `--append-system-prompt T --mcp-config F`, where F names the `mcp_fixture` in `echo_env` mode, the first scripted request has `T` in `messages[..stable_prefix]`, `mcp__bboxy__*` tools are in its `tools`, the `--mcp-config` server sees `BBOXY_SEAT_ID`, and a profile server in the same session does not see `BBOXY_CAPABILITY` |
| §4 Side A | after the amendment: bboxy450-term `make test-harness-native` with a `bboxy450-harness` seat-spawn case. Unproven until then |
| EmbedLifecycle | model EmbedLifecycle (reuses LifecycleOwnership's shape) |

## 7. Amendment to bboxy450-term ICD #348 (for its owner)

These are the exact clauses of `bboxy450-term/.bboxy/interfaces/app-harness.md` (#348) that change. Nothing else in #348 changes.

| #348 row | Change |
|---|---|
| Side B | add: `bboxy450-harness seat`, an in-suite line-mode harness (this ICD) |
| Amended by | add: `bboxy450-harness:[[embed-seat]]`, phase 1a process seat |
| What crosses | add: for `bboxy450-harness`, the MCP server by `--mcp-config` (the same `mcp.json`) and the startup context by `--append-system-prompt`. No hook shim, no skill/plugin dir |
| States and lifecycle | add: a `bboxy450-harness` seat is a PTY process with no daemon. Wake is `Pty` guarded input; state is `Cadence`, grade Inferred; restart is generation + 1 with `--resume <session>` from the first screen line |
| Startup context | add: `bboxy450-harness` receives the same `startup_prompt` text through `--append-system-prompt`. The capability never enters argv |
| Launch permissions | add: no permission flag is passed to `bboxy450-harness`. Its own `Policy` governs, and approvals are answered in line mode (§2) or fall to `on_unattended` (default deny) |
| Versioning | add: the app never writes `~/.bboxy450-harness`. The harness writes only its own sessions there |
| Verified by | add: `make test-harness-native` `bboxy450-harness` seat-spawn case (spawn, `session <ulid>` read from the screen, one guarded line becomes a turn, restart resumes the same session) |

## Change log

- **2026-11-01, at the Oracle's request** (message 01M6BGD5FMZ55AJ4V6SS4Q11KK). This section was checked against protocol v0.4, `src/frontends/seat.rs` and `src/frontends/headless.rs` on main at cdde43d.
  - Startup context and MCP had no delivery mechanism. The seat accepted only `--resume`/`--cwd`/`--profile`, and the old `bboxy__` prefix contradicted [[mcp]] §2. Both now come through `--append-system-prompt` and `--mcp-config`, as for the Claude seat. Alternative rejected: the harness locating `bboxy450-term` itself and building the prompt from env, which would put a second copy of `startup_prompt` on Side B.
  - The `BBOXY_*` environment goes only to `--mcp-config` servers.
  - §2 now lists what v0.4 items show and what stays silent, and matches the implemented result line.
  - §7 lists the exact #348 clauses that change.
  - Status stays proposed until #348's owner agrees.

Owner: Nyx Copperwing <ARCHITECT> (Side B); Corvin Duskmantle <ARCHITECT> (Side A, ICD #348).
