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
updated: "2026-10-31T00:00:00Z"
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
| Launch | A → B | argv `bboxy450-harness seat [--resume <session>] --cwd <dir>`, under a PTY from the supervisor |
| Identity | A → B | env `BBOXY_SEAT_ID`, `BBOXY_GENERATION`, `BBOXY_ROLE`, `BBOXY_SOCK`, `BBOXY_CAPABILITY` (`wake-up/src/launch.rs:487-500`). The capability is never logged, persisted or shown (law [[secrets-never-persist]]) |
| MCP | B → A | The harness `mcp` client starts the stdio bridge `bboxy450-term mcp` as a session MCP server, the same as for the Claude seat. Tools enter the registry under the `bboxy__` prefix |
| Startup context | A → B | The system prompt carries NAME, ROLE, host/cwd and "call bboxy's orient MCP first", as in ICD #348's startup-context line. It is part of the stable prefix (provider §2.6) |
| Screen | B → A | Line mode (below) |
| Wake / input | A → B | Phase 1a: `WakeMethod::Pty` guarded input. One submitted line becomes `Op::UserTurn{Queue}`, and a line typed while a turn runs is also queued. Phase 1b (later): a native socket wake |
| State | B → A | Phase 1a: `StateSource::Cadence`, grade Inferred. This is the same honest grade pi/omp get today (`bbt-mux/src/adapters/mod.rs:80-93`) |
| Session id | B → A | Phase 1a: printed on the first screen line as `session <ulid>`. Phase 1b: registered through the hook action as `harness_session_id` |

## 2. Line mode: the definition of "not a TUI"

- Output is append-only UTF-8 lines. The only escapes allowed are SGR (colour and bold) and OSC 8 (links). There is no cursor addressing, no alt-screen, no clearing and no redraw. Native scrollback is the transcript.
- **It is a projection of the protocol, item by item.**
  - `AgentMessage` deltas are written as they arrive.
  - A `ToolCall` is one header line `▸ name args…` and one result line `✓|✗ summary (n lines, spill path)`.
  - `Approval` prints its options and reads one key line.
  - `TurnEnded` prints `— reason` followed by the prompt `› `.
- Input is cooked line mode. `Ctrl-C` sends `Op::Interrupt` while a turn runs and does nothing at the prompt. `Ctrl-D` at an empty prompt sends `Close{Stop}`.
- The renderer is `src/frontends/seat.rs`, about 300 lines, and is a pure function `Envelope → Vec<u8>`. It is golden-tested against the protocol fixtures. It has no event loop of its own.

## 3. Lifecycle

| Event | Behaviour |
|---|---|
| Start | Print `session <ulid> · profile · model`, then the prompt. The bboxy MCP connects asynchronously. If it is unavailable, print a `Notice` and continue (never block) |
| Restart (generation + 1) | Bboxy relaunches with `--resume <session>`. Store recovery runs (protocol §4), the screen replays the last 20 completed items, then the prompt |
| PTY hangup / seat close | Treated as `Close{Stop}`: cancel the turn, kill process groups, fsync, and exit 0 within 2 s |
| SIGTERM | Same as hangup |

## 4. Changes on Side A (core changes: they need the ICD #348 owner)

1. `bbt-mux/src/mcp/seats.rs:22`: add `"bboxy450-harness"` to `APPROVED_PROFILES`.
2. `bbt-mux/src/adapters/mod.rs:80`: add a generic record arm for `bboxy450-harness`, mirroring pi/omp, with `command = ["bboxy450-harness","seat"]`, `WakeMethod::Pty`, `StateSource::Cadence` and `hooks: false`.
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
| §4 Side A | after the amendment: bboxy450-term `make test-harness-native` with a `bboxy450-harness` seat-spawn case. Unproven until then |
| EmbedLifecycle | model EmbedLifecycle (reuses LifecycleOwnership's shape) |

Owner: Nyx Copperwing <ARCHITECT> (Side B); Corvin Duskmantle <ARCHITECT> (Side A, ICD #348).
