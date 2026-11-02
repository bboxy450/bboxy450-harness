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
updated: 2026-11-02T23:30:00Z
---
# ICD: rich pane

**Status: proposed.** Today a harness seat in term is a PTY running the line-mode face ([[embed-seat]]). The operator wants the rich face inside term. This ICD gives a harness seat's pane a second *face*: the `HarnessView` rendered by term's frame loop from the session's own log stream. Sources: term origin/main `76ad633d`, harness main `60653cd`. Side A rows amend term ICD #348 and need its owner (Corvin Duskmantle <ARCHITECT>) to sign them under d4071.

| | |
|---|---|
| Side A | bboxy450-term: pane model (`bbt-ui-model/src/model/window.rs:75` `PaneBody`), scene (`bboxy450-term/src/scene.rs:1803-1822`), App frame loop (`main.rs:7806` `redraw`), logic input guard (`bbt-mux/src/logic.rs:8669` `guarded_input`), hook observation (`wake-up/src/observe.rs:380`) |
| Side B | bboxy450-harness: `seat` face (`src/frontends/seat.rs`), Unix gateway (`src/gateway/unix.rs`, [[gateway]] §1a/§4), `gui::view::HarnessView` + `gui::bridge::Bridge` under feature `embedded` |
| Crosses | OSC 7502 session announcements (B → A, on the PTY); NDJSON `Subscribe`/`Hello`/`Envelope`/`OpEnvelope`/ack over `~/.bboxy450-harness/run/<session>.sock` (both ways); the `bboxy450-harness` crate linked into term (build time) |

## 1. Decisions this ICD makes

1. **A face, not a new seat kind.** The seat stays the `bboxy450-harness seat` process under bbt-supervisor, with a PTY and the line-mode screen (d20 holds). The pane gains `face: Terminal | HarnessRich`. Both faces show the same session; switching between them is a view toggle and never touches the process.
2. **The session process owns the session; term owns only views.** A rich view is a disposable gateway subscriber. Dropping it detaches ([[gateway]] §5) and never stops a turn.
3. **The log is the interface.** The rich face renders [[protocol]] envelopes only ([[d5972-our-log-format-is-the-center]]). Term never parses the PTY text of a harness seat, and never reads or writes the session's files except blobs, read-only (§3).
4. **Messages are ops, not keystrokes.** Delivery to a harness seat (`seat_input_text`, composer submit) is `Op::UserTurn{mode: Queue}` over the socket. It never falls back to PTY bytes.

## 2. Session discovery (B → A)

The seat face binds `run/<session>.sock` (mode 0600, peer uid checked) as soon as `Host::create`/`open` returns. **Today only `headless` binds (`src/frontends/headless.rs:347`); `seat` must too.** The socket is removed on `SessionClosed` and on exit.

It then writes one hook frame to its PTY, in the existing #348 hook format (`wake-up/src/hook.rs:165`, ≤ 4096 bytes, attributed to the PTY it arrives on, so it carries no capability):

```
ESC ] 7502 ; v=1:seat=<BBOXY_SEAT_ID>:ev=HarnessSession:d=<base64 {"raw":{"session":"<ulid>","socket":"<abs path>","epoch":<n>,"protocol":{"major":M,"minor":m}}}> ESC \
```

- **Emitted** at start, after every `Host::switch` (a switch makes a *new* session: new id, new socket), and after a restart (`--resume`, new epoch).
- **Term** records `{session, socket, epoch}` as the seat's current harness session. It persists `harness_session_id`, which is the phase 1b row of [[embed-seat]], and drops any view or delivery bound to an older session.
- **Line mode** ([[embed-seat]] §2) is amended: exactly this OSC 7502 frame is allowed in addition to SGR and OSC 8.

| State (per seat, Side A) | Meaning |
|---|---|
| `NoSession` | no announcement yet; the rich face shows "starting…" and delivery refuses `harness-session-unknown` |
| `Announced{session, socket, epoch}` | connect allowed |
| `Connected{…, head}` | `Hello` received ([[gateway]] §2); delivery allowed |
| `Closed` | `SessionClosed` seen, or the socket is gone; the next announcement moves the seat back to `Announced` |

## 3. The rich face (Side A renders, Side B supplies the view)

- **Linking.** Term depends on `bboxy450-harness` with `default-features = false, features = ["embedded"]` (no winit/wgpu). The dependency is git-pinned. The toolkit crates are patched to term's in-tree paths, so exactly one copy of each `bboxy450-*` crate is linked (matrix R15: `cargo tree -d … | grep -c bboxy450-` = 0). A pin moves only with a term commit.
- **Bridge over Unix.** Today `Bridge` attaches only `InProc` (`src/frontends/gui/bridge.rs:251`). Side B adds `Bridge::connect(socket, after)`, which runs `UnixClient` on `bbt_rt`, never on the UI thread. It feeds the same `Mailbox`/`FrameState` that `InProc` does. `HarnessView::render(&FrameState, size)` is unchanged.
- **Equality.** The view must produce the same `Frame` for a log, whether it is fed by InProc or Unix.
- **Blobs.** Read-only from `sessions/<id>/blobs` through `store::Blobs` (same user, same machine). A remote seat (ssh or cloud) has no blob path; images show their placeholder until a blob fetch op exists (a [[log-ir-v1]] addition, not in this version).
- **Scene.** The pane rect that `scene.rs:1803-1822` fills with the PTY grid is filled with the view's `Frame` instead when `face == HarnessRich`. The face toggle is a per-pane UI preference, persisted with the layout. The default is `HarnessRich` for `bboxy450-harness` seats with a session announced, otherwise `Terminal`.

## 4. Input routing

| Focused face | Keys go to | Submit | Interrupt | Approval |
|---|---|---|---|---|
| `Terminal` | the PTY (unchanged) | the seat's line editor → `UserTurn` | Ctrl-C → `Op::Interrupt` (seat face) | line mode keys |
| `HarnessRich` | `HarnessView` composer (`view.rs:519 edit`) | `Op::UserTurn{Queue}` over the socket | the view's Stop → `Op::Interrupt` | `Op::ResolveApproval` over the socket |

- No key reaches the PTY while the rich face is focused.
- Both faces are approvers. The first `ResolveApproval` wins, and the second gets `AlreadyResolved`.
- `/profile NAME` in the rich composer sends the switch through the seat process: an `Op::SetProfile` the seat face handles with `Host::switch`. A new `HarnessSession` announcement follows.

## 5. Delivery and readiness (`seat_input_text` for harness seats)

**Readiness.** Positive readiness for a harness seat means **admission**, not idleness: the logic owner holds a `Connected` subscription whose last frame (an envelope or the 15 s heartbeat) is younger than 20 s. It replaces the PTY composer-ready proof for this kind only. `guarded_input` dispatches on kind before the PTY arbiter (`logic.rs:8669`).

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
| no `Connected` state | `not_submitted`, cause `harness-session-unknown` or `harness-socket-unavailable` |
| ack timeout (5 s, [[gateway]] §1a) | `unknown`; the same op_id may be resent and never produces a second turn |

- Zero PTY bytes are written in every row.
- A delivery is bound to the session it was admitted for. After a switch (A → B), a resend for the old message goes to A's socket and is refused `Closed`. It is never re-aimed at B, because B's turn would need a new message.
- A → B → A (switch and switch back) creates a *third* session, so old op_ids cannot collide.

## 6. Lifecycle

| Event | Session process (supervisor) | Rich view (term UI) | Logic delivery client |
|---|---|---|---|
| Window close / Cmd-Q (detach) | keeps running | dropped (detach) | keeps running (logic survives GUI detach) |
| Reopen / reattach | — | resubscribes `after = (epoch, last seq)`; `Reset` → refetch ([[gateway]] §2) | — |
| Cmd-Option-Q / `ctl app stop` | SIGHUP → SIGKILL 250 ms ([[embed-seat]] §3); groups killed | dropped | dropped |
| Seat restart (generation + 1) | `seat --resume <session>` → new epoch → announcement | resubscribes on the announcement | reconnects |
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
| §2 seat binds the socket (0600, removed on close) and announces on start, switch and resume | `tests/seat_process.rs`: socket mode and lifetime; PTY byte capture contains exactly one valid `ev=HarnessSession` frame per (start, switch, resume) with the current session id; line-mode CSI scan still clean | B |
| §3 Unix bridge = InProc bridge | `tests/gui_bridge.rs`: for each `tests/fixtures/ir-v1/*.jsonl`, `layout_text` of a Unix-fed view equals the InProc-fed view | B |
| §3 one toolkit copy | R15 inside term | A |
| §3 visual parity | R27 `parity-gate.py`, plus term's offscreen capture of a rich pane on the same fixture: `row_structure_mismatch` vs the harness native capture ≤ 0.01 | A+B |
| §5 delivery | term `make test-harness-native` rich case: harness seat spawned; announcement observed; `seat_input_text` → `Ack` → `TurnEnded` in the log; resend → `Ack{duplicate}` and one turn; socket removed → `harness-socket-unavailable`; zero PTY bytes in every case (byte counter) | A |
| §5 one turn per message, stale session refused | **formal model `HarnessSocketDelivery`**: `msg ∈ {new, sent, acked}`, `session ∈ {A, B}`, `turns_A, turns_B ∈ 0..2`, `resend`, `switch`; safety `turns_A + turns_B ≤ 1` per message and `NoTurnOnUnannouncedSession`; liveness `sent ~> acked ∨ refused` under WF{ack}. Broken controls: (a) a fresh op_id on resend → **safety**; (b) resend re-aimed at the new session after a switch → **safety** | B (model), A (witness above) |
| §6 detach keeps the session; reattach has no gap | term test: rich view subscribed, `ctl app detach`, turn continues; reattach; durable seq list = `1..=head`, dedup counter 0 | A |
| §7 budget | harness `tests/gui_budget.rs` extended to 4 concurrent views on one injected clock: total drain ≤ 0.3 ms, projection ≤ 0.6 ms, nothing lost; term bench row in `make bench-budgets` (p99, UNMEASURED under load, never PASS) | B, A |

## 10. Changes by side

**Side B (harness):**
1. `seat` binds the socket and emits announcements (§2).
2. `Bridge::connect` over `UnixClient` (§3).
3. Seat face handles `SetProfile` via `Host::switch` and re-announces.
4. The `HarnessSocketDelivery` model.
5. The tests in §9.

**Side A (term, #348 owner):**
1. `PaneBody`/pane face toggle.
2. Scene fill from `HarnessView`.
3. Observe `ev=HarnessSession` (`observe.rs:380` hook parse) and persist `harness_session_id`.
4. Logic delivery client and `guarded_input` kind dispatch with the §5 causes.
5. The `bboxy450-harness` dependency with the toolkit patch.
6. Budget rows.
7. Tests in §9.

**Order:** B1 and B2 first (testable alone), then A3/A4 (delivery works with no UI change), then A1/A2 (the visible pane).

## Change log

- v0.1 proposed, 2026-11-02, Lumen Foxglove <ARCHITECT>, at the Oracle's request (message 01M6F48FNBWWFJVGS7W3K0DYST). Alternatives rejected:
  - **a new non-PTY seat kind.** It would change d20 and the supervisor's ownership for every harness seat, for a view concern.
  - **term parsing the line-mode screen.** That is lossy and violates [[d5972-our-log-format-is-the-center]].
  - **delivering by PTY keystrokes with a new readiness proof.** It keeps the composer race that the guard exists for; an op with an idempotent id has no race.
  - **ACP between term and the seat.** It is not lossless for our items (R12); ACP stays at process edges to *other* harnesses.
