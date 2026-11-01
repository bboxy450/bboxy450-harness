---
id: "0JSWP3Y5BHPEN0Z1ENP01J2TEB"
number: 4
type: interface
title: "ICD: gateway — send, subscribe, lag, replay (v0.1)"
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[protocol]]", rel: carries }
created: "2026-10-31T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: gateway

| | |
|---|---|
| Side A | `engine` and `store` (E1), behind `src/gateway/` |
| Side B | the faces: `gui` (E2), `headless`, `acp`, `seat` |
| Transports | `InProc` (the same process; the GUI and tests) and `Unix` (socket `~/.bboxy450-harness/run/<session>.sock`, mode 0600, peer uid checked). Admission semantics are identical; the client call shapes differ (§1a) |

## 1. Trait

```rust
pub trait Gateway: Send + Sync {
    /// Never awaits and never blocks: try_send into the session inbox.
    fn send(&self, op: OpEnvelope) -> Result<Ack, Refused>;
    /// Never awaits: registers a subscriber and returns at once.
    fn subscribe(&self, req: Subscribe) -> Result<Subscription, Refused>;
}
pub struct Ack { pub op_id: OpId, pub duplicate: bool }
pub enum Refused { Busy, UnknownSession, SessionExists, Version { server: Version }, Closed }
pub struct Subscribe {
    pub session: SessionId,
    pub after: Option<(Epoch, Seq)>,   // None = from seq 1
    pub ephemeral: bool,               // false for headless --json replay, true for live faces
}
pub struct Subscription { /* bounded receiver */ }
impl Subscription {
    pub fn try_recv(&mut self) -> TryRecv;                 // GUI: drained under the frame budget
    pub async fn recv(&mut self) -> Option<Envelope>;      // headless, acp, tests
    pub fn set_waker(&mut self, w: Arc<dyn bbt_rt::Waker>); // marks a DirtySource; no allocation per event
}
```

## 1a. Server admission and transport clients

`Gateway` is the **server-side admission** interface. `InProc` implements it directly, and the Unix server calls it for each frame it reads. Law 3 binds this side: admission never awaits, and the engine never waits on any client.

A **remote client** cannot return the server's answer without waiting for its frame. It is therefore a separate, `async` type:

```rust
pub struct UnixClient { /* one connection: a writer and a reader task that demultiplexes frames */ }
impl UnixClient {
    pub async fn connect(path: &Path, sub: Subscribe) -> Result<(UnixClient, Subscription), ClientError>;
    /// Awaits only this connection's correlated {"ack"} | {"refused"} frame, never the engine.
    pub async fn send(&self, op: OpEnvelope) -> Result<Ack, ClientError>;
}
pub enum ClientError { Refused(Refused), Timeout { op_id: OpId }, Disconnected, Protocol(String) }
```

Rules:
- **Who uses which.** The GUI uses `InProc` only, and no `UnixClient` call ever runs on the UI thread. `headless`, `acp`, `seat` and remote tools use `UnixClient` from async tasks.
- **Replies do not block the stream.** The reader task routes ack and refused frames by `op_id` to waiting `send` calls, and routes envelopes to the `Subscription`. A pending `send` never delays envelope delivery, and the reverse also holds.
- **Deadline.** `send` waits at most 5 s for its frame, then returns `Timeout{op_id}`. The outcome is then **unknown**: the caller may resend the *same* `op_id`, which idempotence makes safe (protocol §2), and must never mint a new id for the same intent.
- **Disconnect.** If the connection drops, every pending `send` returns `Disconnected`, with the same unknown-outcome rule. The `Subscription` ends, and the client reconnects with `after = (epoch, last_seq)`.

## 1b. Loading: an async factory, and a Gateway over loaded sessions only

Opening a log runs recovery (protocol §4), which reads and fsyncs the disk, so it cannot happen inside a non-awaiting call. Loading is therefore a separate `async` step, and a `Gateway` only ever serves a session that is already loaded. Phase 1 has one session per process; there is no multi-session daemon.

```rust
pub struct Host { /* store root, profiles, runtime handles */ }
impl Host {
    /// Mints the log and the actor for a StartSession envelope, then returns the bound gateway.
    pub async fn create(&self, op: OpEnvelope) -> Result<(InProc, Ack), Refused>;
    /// Loads an existing log, runs recovery (Epoch + 1), then returns the bound gateway.
    pub async fn open(&self, session: SessionId) -> Result<InProc, Refused>;
}
impl Gateway for InProc { /* bound to exactly one loaded session */ }
impl InProc { pub fn session(&self) -> SessionId; }
impl Subscription { pub fn hello(&self) -> &Hello; } // always present: the session is loaded
```

Rules:
- **No pending Hello.** `subscribe` succeeds only on a loaded session, so `hello()` is available at once. `subscribe` or `send` naming another session returns `UnknownSession`; it never loads.
- **`StartSession`.** `create` is its only path. Its errors are `SessionExists` (unless the same `op_id`, which returns the duplicate `Ack` and the existing `InProc`) and `Version`. `StartSession` sent through `Gateway::send` returns `SessionExists`. The outcome rule is unchanged: `SessionStarted` with `cause = op_id`, seq 1.
- **`open`.** Its errors are `UnknownSession` (no log) and `Closed` (the log ends in `SessionClosed`; read-only replay of a closed log is the `headless --json` path through the store, not a gateway). A store failure during recovery is `Closed` too, with the error as a `Notice` in the next open; nothing half-loaded is ever served.
- **Unix.** The server calls `Host::create` or `Host::open` first and binds `run/<session>.sock` only after it returns, so a client never connects to an unloaded session. There is no create socket: the process that hosts the session (seat, headless) calls `Host::create` locally, then binds. `UnixClient::connect` always targets a bound session socket with a `Subscribe`, and sending `StartSession` over it returns `SessionExists`.
- **Who awaits.** The GUI awaits `create`/`open` from a background task and shows a loading state; the frame loop never awaits it. Law 1 is unchanged: once loaded, admission never awaits.

## 2. Delivery order

1. **`Hello`.** The stream's first frame is `Hello{ v, epoch, head: Seq, meta }`. On the Unix transport it is a frame of its own; in-process it is the `Subscription::hello()` accessor.
2. **`Reset`, if needed.** If `after.epoch != epoch` and `after.seq > head`, the subscription delivers `Reset{epoch, head}` and closes. The face drops everything above `head` and resubscribes.
3. **Replay.** Every durable envelope with `after.seq < seq ≤ head`, in order.
4. **Snapshots.** If `ephemeral`, one `ItemSnapshot` per open item.
5. **Live.** Durable and ephemeral envelopes in emission order. Durable `seq` values are strictly increasing with no gaps.

Replay and live are joined without a gap or a duplicate. The subscriber is registered with the fan-out *before* the store read, and any live durable envelope with `seq ≤` the last replayed seq is discarded. Faces still deduplicate by `seq` as defence; the dedup counter must stay at 0 in tests.

## 3. Lag

- Each subscriber has a bounded buffer of 4096 envelopes. The engine publishes with `try_push` only.
- Ephemeral deltas are coalesced per item before the buffer, at 50 ms. When the buffer is above ¾ full, the subscriber drops new ephemeral envelopes and counts the drops; the next `ItemSnapshot` repairs the view.
- If a *durable* envelope cannot be pushed, the subscriber receives `Lagged{resume_after: last_delivered_seq}` and is closed. The face resubscribes with `after = (epoch, resume_after)`. The engine never waits, retries or slows down for a subscriber.
- A dropped `Subscription` deregisters itself, and the fan-out never sends to it again.

## 4. Socket framing

- Newline-delimited JSON: `OpEnvelope` from client to server, `Envelope` and `Hello` from server to client.
- One connection carries one subscription plus any number of ops. A send reply is a frame `{"ack":…}` or `{"refused":…}` correlated by `op_id`.
- A heartbeat `Status` is sent every 15 s while idle. A client that sends nothing for 60 s is kept; only an unread buffer closes it, via `Lagged`.
- A frame over 256 KiB is a protocol error, and that connection is closed. The session continues.

## 5. Lifecycle

- **Start.** `Host::create` creates the log and the actor; `Host::open` loads one and runs recovery (protocol §4). Both are `async` and finish before any `Gateway` call (§1b). Subscribe never loads.
- **Detach.** Dropping a `Subscription` is detaching; it is not an op. When the last face detaches, the actor keeps running until its turn ends. It then stays idle and resumable.
- **Stop.** `Op::Close` cancels the running turn and the queued turns, appends `SessionClosed`, fsyncs, closes subscriptions after delivering `SessionClosed`, and drops the actor.
- **Restart** of the process is the same as load: `Epoch + 1`.

## 6. Failure

| Situation | Behaviour |
|---|---|
| Work inbox full | `Refused::Busy`. The face shows it and may retry; nothing is queued in the gateway |
| Store write fails (disk full) | The turn ends with `StopReason::Error{class: Store}`, and that `TurnEnded` is itself not appended. The face sees the error as ephemeral `Status`, then `Reset` on its next subscribe. Recovery closes the turn |
| Face crashes | Its subscription drops. The engine is unaffected |
| Engine panics in a session actor | The actor is restarted by load (recovery) and subscribers get `Reset`. Other sessions are unaffected |

## 7. Verified by

| Clause | Evidence |
|---|---|
| §1 send never awaits | invariant [[never-block]]; `send` is a non-`async fn` (a compile-time fact); `tests/gateway_ops.rs::busy_not_wait` fills the inbox and asserts `Busy` in under 1 ms |
| §2 no gap, no duplicate | model SubscriberReplay + Broken (the broken control registers *after* the store read); `tests/gateway_replay.rs` subscribes at every seq of a fixture session and asserts the concatenated durable seq list equals `1..=head` |
| §3 lag | `tests/gateway_lag.rs`: a never-reading subscriber under a 100k-delta firehose gets `Lagged` while a second subscriber and the engine finish the turn |
| §4 transports equal | the same `gateway_*` suite runs against `InProc` and `Unix` (one `#[rstest]` matrix); for `Unix` the `UnixClient` result is compared to the `InProc` `Ack`/`Refused` |
| §1b loading | `tests/gateway_host.rs`: (a) `subscribe` on a fresh `open` returns at once with `hello().head` equal to the recovered head; (b) `send(StartSession)` on a bound `InProc` returns `SessionExists`; (c) a `create` replayed with the same `op_id` returns `Ack{duplicate: true}` and one `SessionStarted`; (d) `open` on a closed log returns `Closed`; (e) a Unix client connecting before `open` returns sees no socket, then connects once it is bound |
| §1a client rules | `tests/gateway_unix.rs`: (a) a server that withholds the ack frame gives `Timeout` in ≤ 5.1 s while envelopes keep arriving on the same connection; (b) resending the same `op_id` after a timeout gives `Ack{duplicate: true}` and one effect; (c) killing the server gives `Disconnected` to every pending send |

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: E2 (the GUI uses `InProc`).

## Change log

- **2026-11-01, at E1's question** (message 01M6AHRY2RX664DTK1S4NJ0QR8). A non-awaiting `subscribe` with an immediate `hello()` cannot load and recover a log from disk. Adds §1b: an async `Host::create`/`Host::open` factory, a `Gateway` over loaded sessions only, no pending `Hello`, and `StartSession` through `create` only. Restores the missing §2 heading. Alternative rejected: a pending-`Hello` subscription, which adds a state to every face for a case phase 1 does not have (one session per process). Version unchanged; nothing in §5's implicit load had been built.
- **2026-11-01, at E1's follow-up** (message 01M6AJ86HR8KJP4V8K9D441HV1). Removes the undefined "create socket" from the §1b Unix rule: creation is local to the hosting process, and sockets exist only for loaded sessions.
