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
updated: "2026-10-31T00:00:00Z"
---
# ICD: gateway

| | |
|---|---|
| Side A | `engine` and `store` (E1), behind `src/gateway/` |
| Side B | the faces: `gui` (E2), `headless`, `acp`, `seat` |
| Transports | `InProc` (the same process; the GUI and tests) and `Unix` (socket `~/.bboxy450-harness/run/<session>.sock`, mode 0600, peer uid checked). Both have identical semantics |

## 1. Trait

```rust
pub trait Gateway: Send + Sync {
    /// Never awaits and never blocks: try_send into the session inbox.
    fn send(&self, op: OpEnvelope) -> Result<Ack, Refused>;
    /// Never awaits: registers a subscriber and returns at once.
    fn subscribe(&self, req: Subscribe) -> Result<Subscription, Refused>;
}
pub struct Ack { pub op_id: OpId, pub duplicate: bool }
pub enum Refused { Busy, UnknownSession, Version { server: Version }, Closed }
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

## 2. Delivery order for one subscription

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

- **Start.** `StartSession` creates the actor and the log. A subscribe to a session that exists on disk but is not loaded loads it, which runs recovery (protocol §4), and then serves.
- **Detach.** `Close{Detach}` from the last face leaves the actor running until its turn ends. It then stays idle and resumable.
- **Stop.** `Close{Stop}` cancels the running turn, fsyncs, closes subscriptions with no further frames, and drops the actor.
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
| §4 transports equal | the same `gateway_*` suite runs against `InProc` and `Unix` (one `#[rstest]` matrix) |

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: E2 (the GUI uses `InProc`).
