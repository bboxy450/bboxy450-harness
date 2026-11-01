---
id: 01M6CK65TFQVEBCJRP6ANBDM62
number: 4962
type: interface
title: "ICD: log-converter watcher ↔ native logs, and watcher ↔ sink (v0.1, proposed)"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[log-converter]]", rel: part-of }
  - { to: "[[lc-ir]]", rel: carries }
  - { to: "[[lc-bxl]]", rel: fed-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: watcher and sink, v0.1

**Status: proposed.** `bboxy450-log-converter watch` follows a live Claude Code or Codex session log, parses it into [[lc-ir]] events, and delivers them to a sink. The sink is a file now (a `bxl` log, [[lc-bxl]]); a cloud sink comes later through the same interface.

| | |
|---|---|
| Side A (native) | Claude Code and Codex, external writers of append-only JSONL ([[native-session-formats]]). We only read |
| Side B (watcher) | `watch::Watcher`, one per native session file |
| Side C (sink) | `trait Sink`: `FileSink` (bxl) now, `RemoteSink` later |

## 1. Native file assumptions (Side A gives no guarantee; the watcher checks)

- **Lines are appended.** A line is complete only when its `\n` has been written. A trailing partial line is not read; the watcher waits for it.
- **Identity** is `(dev, inode)`. A different inode at the same path means the file was replaced: the watcher reads the new file from 0 and dedupes by EventId.
- **Shrink.** A file shorter than the cursor offset was truncated or rewritten. The watcher re-reads from 0 and dedupes by EventId. Each detection produces a Notice, `native_truncated`.
- **Discovery.**
  - Claude: `~/.claude/projects/<cwd-slug>/<session-uuid>.jsonl` is stable for the session.
  - Codex: `~/.codex/sessions/YYYY/MM/DD/rollout-<ISO>-<uuid>.jsonl`; the watcher matches the `session_meta.id` on the first line.
  - The watcher never reads or writes `~/.codex/state_5.sqlite`.

## 2. Watcher

- Reads with bounded memory: one native line, up to a 64 MiB cap. Lines over the cap become a Notice and a counted `oversize_line`, and the watcher continues.
- Batches the parsed events per native turn or per 250 ms, and calls `Sink::write`.
- **Back-pressure.** At most 4 unacknowledged batches are in flight. Past that, the watcher stops reading. It never drops anything and never buffers without bound, and the native writer is unaffected (we are only a reader).
- **Restart** resumes from `Sink::open(..)`'s returned cursor, so nothing is lost and the cursor is never stored apart from what the sink committed.
- Ends when the native session closes (no growth and the client is gone, if known) or on a stop signal. A stop flushes, waits for the in-flight acks, then exits.

## 3. Cursor and delivery

```rust
pub struct SourceCursor { pub path: PathBuf, pub dev: u64, pub inode: u64,
                          pub offset: u64,                      // start of the next unread line
                          pub ordinal: u64,                     // native line number at `offset`
                          pub last_line_blake3: [u8; 16] }      // guards against a same-size rewrite
pub enum Delivery { ExactlyOnce, AtLeastOnce }
pub trait Sink {
    fn delivery(&self) -> Delivery;
    /// Returns the cursor after the last batch this sink durably committed, or None for a new sink.
    fn open(&mut self, session: &Session) -> Result<Option<SourceCursor>, SinkError>;
    /// Durable when it returns Ok: the events AND `after` are committed together, or neither is.
    /// Events must be idempotent on EventId: a redelivered EventId is acknowledged and ignored.
    fn write(&mut self, events: &[Event], after: &SourceCursor) -> Result<Ack, SinkError>;
    fn close(&mut self) -> Result<(), SinkError>;
}
pub enum SinkError { Retryable(String), Fatal(String) }
```

- **The cursor lives inside the sink's commit**, never in a separate cursor file. For `FileSink`, `after` is the data frame's `source_cursor` ([[lc-bxl]] §2), and an ack means `sync()` returned. Events and cursor share one fsynced frame, so the delivery is **exactly-once** into the file.
- **AtLeastOnce sinks** (a future remote sink) must dedupe on EventId. EventId is deterministic ([[lc-ir]] §2), so a redelivery after a crash carries the same ids.
- **Retryable** errors back off with a cap and keep the in-flight bound. **Fatal** stops the watcher with a Notice. The watcher never skips a batch.
- **Rewrite guard.** If the line ending at `offset` no longer hashes to `last_line_blake3`, the file was rewritten. The watcher re-reads from 0 and dedupes.

## 4. Model: `LcWatcher` (TLA+, bboxy450-tlc + Java parity)

- **State:** native file (`lines`, `partial`, `inode`), watcher (`cursor`, `in_flight`), sink (`committed_events`, `committed_cursor`), `crashed`.
- **Actions:** NativeAppendLine, NativeAppendPartial, NativeCompleteLine, NativeTruncate, NativeReplace, WatcherRead, SinkAck, SinkRetryableFail, Crash, Restart.
- **Bounds:** 3 lines, 1 truncate, 1 replace, in-flight ≤ 2.
- **Safety**:
  - `NoPartialLine`: no event comes from a partial line.
  - `NoGap`: committed events are exactly the native line-events up to `committed_cursor`, deduped.
  - `ExactlyOnceFile`: under the ExactlyOnce sink, no EventId is committed twice.
  - `BoundedInFlight`.
- **Liveness:** `CatchesUp` (WF on WatcherRead and SinkAck, with failures finite): every complete native line is eventually committed.
- **Broken controls:**
  - (a) the cursor is persisted before the sink ack → `NoGap` fails after Crash;
  - (b) a partial line is consumed → `NoPartialLine` fails;
  - (c) inode change is ignored → `NoGap` fails after NativeReplace.

## 5. Verified by

| Clause | Evidence |
|---|---|
| §1–§3 | `tests/watch.rs`: a writer process appends synthetic Claude and Codex lines, including partial lines, a truncation and a replacement. The watcher is killed with SIGKILL at random points and restarted. The final bxl equals the parse of the final native file exactly, with no duplicate EventId |
| §2 back-pressure | a sink that never acks; the watcher reads at most 4 batches and its memory stays bounded (a structural count of buffered events, not RSS) |
| §4 model | `make test-formal`, with `LcWatcher` and its controls |

Owner: Nyx Copperwing <ARCHITECT>.
