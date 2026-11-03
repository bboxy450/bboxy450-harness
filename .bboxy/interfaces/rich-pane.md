---
id: 01M6F4WS2V5CTBAN082N508QT0
number: 5976
type: interface
title: 'ICD: rich pane — bboxy450-term hosts the harness rich view natively in a seat pane, over the session socket (proposed v0.1)'
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[embed-seat]]", rel: amends }
  - { to: "[[gateway]]", rel: uses }
  - { to: "[[protocol]]", rel: carries }
  - { to: "[[budgets]]", rel: allocates }
  - { to: "[[log-ir-v1]]", rel: constrained-by }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: builds-on }
created: 2026-11-02T23:30:00Z
updated: 2026-11-03T14:30:00Z
---
# ICD: rich pane

**Status: proposed.** Today a harness seat in term is a PTY running the line-mode face ([[embed-seat]]). The operator wants the rich face inside term. This ICD gives a harness seat's pane a second *face*: the `HarnessView` rendered by term's frame loop from the session's own log stream. Sources: term origin/main `76ad633d`, harness main `60653cd`. Side A rows amend term ICD #348 and need its owner (Corvin Duskmantle <ARCHITECT>) to sign them under d4071.

| | |
|---|---|
| Side A | bboxy450-term: pane model (`bbt-ui-model/src/model/window.rs:75` `PaneBody`), scene (`bboxy450-term/src/scene.rs:1803-1822`), App frame loop (`main.rs:7806` `redraw`), logic input guard (`bbt-mux/src/logic.rs:8669` `guarded_input`), launch kit (`wake-up/src/launch.rs:515`) |
| Side B | bboxy450-harness: `seat` face (`src/frontends/seat.rs`), Unix gateway (`src/gateway/unix.rs`, [[gateway]] §1a/§4), `gui::view::HarnessView` + `gui::bridge::Bridge` under feature `embedded` |
| Crosses | `--socket <path>` in the seat argv (A → B); NDJSON `Subscribe`/`Hello`/`Envelope`/`OpEnvelope`/ack over that socket (both ways, [[gateway]] §4); the `bboxy450-harness` crate linked into term (build time, [[d5977-rich-view-dependency-direction]]) |

## 1. Decisions this ICD makes

1. **A face, not a new seat kind.** The seat stays the `bboxy450-harness seat` process under bbt-supervisor, with a PTY and the line-mode screen (d20 holds). The pane gains `face: Terminal | HarnessRich`. Both faces show the same session; switching between them is a view toggle and never touches the process.
2. **The session process owns the session; term owns only views.** A rich view is a disposable gateway subscriber. Dropping it detaches ([[gateway]] §5) and never stops a turn.
3. **The log is the interface.** The rich face renders [[protocol]] envelopes only ([[d5972-our-log-format-is-the-center]]). Term never parses the PTY text of a harness seat, and never reads or writes the session's files except blobs, read-only (§3).
4. **Messages are ops, not keystrokes.** Delivery to a harness seat (`seat_input_text`, composer submit) is `Op::UserTurn{mode: Queue}` over the socket. It never falls back to PTY bytes.

## 2. The seat socket (A chooses the path, B binds it)

Term chooses the path, so nothing has to be discovered.

- **Argv.** The launch kit adds `--socket <abs path>` to the seat argv (`wake-up/src/launch.rs:515`, beside `--mcp-config`). The path is in term's own runtime directory and is at most 96 bytes (macOS `sun_path` is 104), e.g. `<runtime>/h-<seat ulid>.sock`. Without the flag the seat binds `~/.bboxy450-harness/run/<session>.sock` as `headless` does today (`src/frontends/headless.rs:347`); **`seat` binds nothing today and must.**
- **Bind.** Mode 0600, peer uid checked, bound as soon as `Host::create`/`open` returns and before the first prompt is shown. The per-session socket is bound as well, unchanged.
- **Switch.** `Host::switch` makes a new session. The seat closes the listener, unlinks the path and binds it again on the new session's gateway. Subscribers of the old session receive `SessionClosed`; a reconnect to the same path gets the new session's `Hello`. The old per-session socket goes away with the old session.
- **Resume** (`seat --resume`, new epoch) and **restart** bind the same path again.
- **Exit** unlinks the path.
- **Term** learns the session id from `Hello.meta.session` and persists it as `harness_session_id` (the phase 1b row of [[embed-seat]]) for `--resume`. Nothing on the PTY changes: line mode ([[embed-seat]] §2) is untouched.

| State (per seat, Side A) | Meaning |
|---|---|
| `Unbound` | connect fails (`ENOENT`, `ECONNREFUSED`): the rich face shows "starting…"; delivery refuses `harness-socket-unavailable` |
| `Connected{session, epoch, head}` | `Hello` received ([[gateway]] §2); delivery allowed |
| `Closed` | `SessionClosed` or EOF: reconnect with backoff, at most one attempt per 2 s. A successful reconnect is a new `Connected`, possibly on a new session |

## 3. The rich face (Side A renders, Side B supplies the view)

- **Linking.** Term depends on `bboxy450-harness` with `default-features = false, features = ["embedded"]` (no winit/wgpu). The dependency is git-pinned. The toolkit crates are patched to term's in-tree paths, so exactly one copy of each `bboxy450-*` crate is linked (matrix R15: `cargo tree -d … | grep -c bboxy450-` = 0). A pin moves only with a term commit.
- **Bridge over Unix.** Today `Bridge` attaches only `InProc` (`src/frontends/gui/bridge.rs:251`). Side B adds `Bridge::connect(socket, after)`, which runs `UnixClient` on `bbt_rt`, never on the UI thread. It feeds the same `Mailbox`/`FrameState` that `InProc` does. `HarnessView::render(&FrameState, size)` is unchanged.
- **Equality.** The view must produce the same `Frame` for a log, whether it is fed by InProc or Unix.
- **Blobs.** Read-only from `sessions/<id>/blobs` through `store::Blobs` (same user, same machine). A remote seat (ssh or cloud) has no blob path; images show their placeholder until a blob fetch op exists (a [[log-ir-v1]] addition, not in this version).
- **Direction.** Term depending on the harness crate is a two-repo lockstep on every toolkit bump. It is accepted for phase 1 and has a recorded exit: [[d5977-rich-view-dependency-direction]].
- **Scene.** The pane rect that `scene.rs:1803-1822` fills with the PTY grid is filled with the view's `Frame` instead when `face == HarnessRich`. The face toggle is a per-pane UI preference, persisted with the layout. The default is `HarnessRich` for `bboxy450-harness` seats with a session announced, otherwise `Terminal`.

## 4. Input routing

| Focused face | Keys go to | Submit | Interrupt | Approval |
|---|---|---|---|---|
| `Terminal` | the PTY (unchanged) | the seat's line editor → `UserTurn` | Ctrl-C → `Op::Interrupt` (seat face) | line mode keys |
| `HarnessRich` | `HarnessView` composer (`view.rs:519 edit`) | `Op::UserTurn{Queue}` over the socket | the view's Stop → `Op::Interrupt` | `Op::ResolveApproval` over the socket |

- No key reaches the PTY while the rich face is focused.
- Both faces are approvers. The first `ResolveApproval` wins, and the second gets `AlreadyResolved`.
- `/profile NAME` in the rich composer sends the switch through the seat process: an `Op::SetProfile` the seat face handles with `Host::switch`, then it rebinds the seat socket (§2).

## 5. Delivery and readiness (`seat_input_text` for harness seats)

**Readiness.** Positive readiness for a harness seat means **admission**, not idleness: the logic owner holds a `Connected` connection to the seat socket (§2) whose last frame (an envelope or the 15 s heartbeat) is younger than 20 s. It replaces the PTY composer-ready proof for this kind only. `guarded_input` dispatches on kind before the PTY arbiter (`logic.rs:8669`).

**Status.** Seat status (what the harness is doing: rim, sidebar row, `seat_get`) is not derived from socket liveness or output cadence. It comes from the seat's 7501 reports with their 2 s heartbeat and from the session log ([[embed-seat]] p7; #348 phase 1c; operator 2026-11-03). A pane whose rich face holds a `Connected` socket already receives the same events (`TurnStarted`, `TurnEnded`, `Status{phase}`, approval items) and may feed them as a Reported source named `socket`; nothing here requires it.

**Delivery.**
- The logic owner sends `OpEnvelope{op_id, op: UserTurn{blocks: [Text], mode: Queue}}`.
- `op_id` is derived deterministically from the bboxy message id, so a retry is `Ack{duplicate: true}` with one effect ([[gateway]] §7 test (b)).
- A turn running when the message arrives is not a refusal: the turn queues (engine and ACP backend both queue).

| Outcome | `seat_input_text` result |
|---|---|
| `Ack` | submitted; the receipt carries the turn's `cause` = op_id |
| `Ack{duplicate}` | submitted (same effect) |
| `Refused::Busy` | `not_submitted`, cause `harness-busy`, retryable |
| `Refused::Closed` / `UnknownSession` | `not_submitted`, cause `harness-session-closed` |
| socket unbound, or no `Hello` yet | `not_submitted`, cause `harness-socket-unavailable` |
| ack timeout (5 s, [[gateway]] §1a) | `unknown`; the same op_id may be resent and never produces a second turn |

- Zero PTY bytes are written in every row.
- A delivery names the session of the `Hello` it was admitted under (`OpEnvelope.session`). After a switch (A → B) the same path serves B, so a resend for the old message is refused `UnknownSession` by construction and reported `harness-session-closed`. It is never re-aimed at B, because B's turn would need a new message.
- A → B → A (switch and switch back) creates a *third* session, so old op_ids cannot collide.

## 6. Lifecycle

| Event | Session process (supervisor) | Rich view (term UI) | Logic delivery client |
|---|---|---|---|
| Window close / Cmd-Q (detach) | keeps running | dropped (detach) | keeps running (logic survives GUI detach) |
| Reopen / reattach | — | resubscribes `after = (epoch, last seq)`; `Reset` → refetch ([[gateway]] §2) | — |
| Cmd-Option-Q / `ctl app stop` | SIGHUP → SIGKILL 250 ms ([[embed-seat]] §3); groups killed | dropped | dropped |
| Seat restart (generation + 1) | `seat --resume <session>` → new epoch, same socket path | reconnects; `Reset` handling | reconnects |
| `Lagged` | — | resubscribe from `resume_after` | resubscribe |
| Socket vanishes / harness crash | exited process | "session ended" banner, Terminal face still shows the PTY | `Closed` |

## 7. Budgets (term frame 8.33 ms at 120 Hz; CPU path 4 ms: drain 1.5, projection 1.0, encode 1.0, margin 0.5, term `.bboxy/specs/budgets.md:18-33`)

| Resource | Allocation |
|---|---|
| Socket IO, JSON parse, transcript update | off the UI thread (`bbt_rt` task per view), ≤ 1 ms CPU between awaits ([[budgets]] §1) |
| UI drain of all harness views | **≤ 0.3 ms per frame in total**, shared across every visible harness pane, round-robin, remainder deferred, nothing lost (the HarnessDirtyWakeFrame shape) |
| Projection (layout + paint) of all harness views | **≤ 0.6 ms per frame in total**; completed items' layout reused; hidden or occluded panes 0 |
| Markdown > 4 KiB | prepared off the UI thread ([[budgets]] §2) |
| Memory per rich view | ≤ 32 MiB transcript + layout cache (an allocation; **unmeasured**) |
| Connections | one per visible rich view plus one per seat for logic delivery; each socket subscriber buffer 4096 envelopes, server side |

These are the harness's existing single-page shares, now made **aggregate**. More rich panes means slower per-pane updates, never a slower frame. A change that spends more needs a term budget decision first.

## 8. Failure

- A socket connect failure or a version refusal (major > ours, [[log-ir-v1]] §2) puts the rich face in "unavailable: <cause>" with a one-click switch to `Terminal`. The PTY face always works.
- A malformed frame or one over 256 KiB closes that connection ([[gateway]] §4); the view reconnects with backoff, capped at 1 per 2 s.
- No step of the UI or logic waits on the socket: connect, send and read are async; the frame loop reads only `Latest`.

## 9. Verified by

| Clause | Evidence | Side |
|---|---|---|
| §2 seat binds `--socket` (0600), rebinds on switch, unlinks on exit | `tests/seat_process.rs`: mode and lifetime; after `/profile` the same path answers `Hello` with the new session id and the old session's subscriber saw `SessionClosed`; connect during the rebind window fails then succeeds within 2 s; line-mode CSI scan unchanged | B |
| §3 Unix bridge = InProc bridge | `tests/gui_bridge.rs`: for each `tests/fixtures/ir-v1/*.jsonl`, `layout_text` of a Unix-fed view equals the InProc-fed view | B |
| §3 one toolkit copy | R15 inside term | A |
| §3 visual parity | R27 `parity-gate.py`, plus term's offscreen capture of a rich pane on the same fixture: `row_structure_mismatch` vs the harness native capture ≤ 0.01 | A+B |
| §5 delivery | term `make test-harness-native` rich case: harness seat spawned; announcement observed; `seat_input_text` → `Ack` → `TurnEnded` in the log; resend → `Ack{duplicate}` and one turn; socket removed → `harness-socket-unavailable`; zero PTY bytes in every case (byte counter) | A |
| §5 one turn per message, stale session refused | **formal model `HarnessSocketDelivery`**: `msg ∈ {new, sent, acked}`, `session ∈ {A, B}`, `turns_A, turns_B ∈ 0..2`, `resend`, `switch`; safety `turns_A + turns_B ≤ 1` per message and `NoTurnOnUnannouncedSession`; liveness `sent ~> acked ∨ refused` under WF{ack}. Broken controls: (a) a fresh op_id on resend → **safety**; (b) resend re-aimed at the new session after a switch → **safety** | B (model), A (witness above) |
| §6 detach keeps the session; reattach has no gap | term test: rich view subscribed, `ctl app detach`, turn continues; reattach; durable seq list = `1..=head`, dedup counter 0 | A |
| §7 budget | harness `tests/gui_budget.rs` extended to 4 concurrent views on one injected clock: total drain ≤ 0.3 ms, projection ≤ 0.6 ms, nothing lost; term bench row in `make bench-budgets` (p99, UNMEASURED under load, never PASS) | B, A |

## 10. Changes by side

**Side B (harness):**
1. `seat --socket <path>`: bind, rebind on switch, unlink on exit (§2).
2. `Bridge::connect` over `UnixClient` (§3).
3. Seat face handles `SetProfile` via `Host::switch` and rebinds.
4. The `HarnessSocketDelivery` model.
5. The tests in §9.

**Side A (term, #348 owner):**
1. `PaneBody`/pane face toggle.
2. Scene fill from `HarnessView`.
3. Launch kit passes `--socket`; logic connects and persists `harness_session_id` from `Hello`.
4. Logic delivery client and `guarded_input` kind dispatch with the §5 causes.
5. The `bboxy450-harness` dependency with the toolkit patch.
6. Budget rows.
7. Tests in §9.

**Order:** B1 and B2 first (testable alone), then A3/A4 (delivery works with no UI change), then A1/A2 (the visible pane).

## Change log

- v0.1 second pass, 2026-11-02, at the operator's request ("rethink your choices"). Session discovery by an OSC 7502 frame on the PTY is replaced by a term-chosen `--socket` path. It removes the announcement channel, the line-mode amendment, the `observe.rs` change and two Side A states, and makes the stale-session rule hold by construction (the rebound path serves the new session; an old `OpEnvelope.session` is `UnknownSession`). The rejected in-band frame was more moving parts for the same fact. Added the dependency-direction decision.
- v0.1 proposed, 2026-11-02, Lumen Foxglove <ARCHITECT>, at the Oracle's request (message 01M6F48FNBWWFJVGS7W3K0DYST). Alternatives rejected:
  - **a new non-PTY seat kind.** It would change d20 and the supervisor's ownership for every harness seat, for a view concern.
  - **term parsing the line-mode screen.** That is lossy and violates [[d5972-our-log-format-is-the-center]].
  - **delivering by PTY keystrokes with a new readiness proof.** It keeps the composer race that the guard exists for; an op with an idempotent id has no race.
  - **ACP between term and the seat.** It is not lossless for our items (R12); ACP stays at process edges to *other* harnesses.
