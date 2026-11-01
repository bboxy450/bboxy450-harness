---
id: 01M6CK65TFTM9Z2JVXCRFVZXC3
number: 4961
type: interface
title: "ICD: bxl — the log-converter binary log; append writer ↔ tailing readers (v0.1, proposed)"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[log-converter]]", rel: part-of }
  - { to: "[[lc-ir]]", rel: carries }
  - { to: "[[d4949-log-converter-from-scratch]]", rel: constrained-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: bxl, the binary IR log, v0.1

**Status: proposed.** `bxl` stores an [[lc-ir]] session compactly. One writer appends while any number of readers tail it. The design is inspired by the zstd seekable format and CBOR, as recommended in #4951. The code is our own, with no zstd-seekable crate.

| | |
|---|---|
| Side A | `bxl::Writer`, the only writer of a segment |
| Side B | `bxl::Reader` (random access by seq) and `bxl::Tail` (follow), any number of them, in any process |
| Medium | a local POSIX file system. Network file systems are out of scope (no `flock` guarantee) |

## 1. Files

- A session log is a directory, `<name>.bxl.d/`, holding:
  - **segments** `000001.bxl`, `000002.bxl`, …;
  - **index sidecars** `000001.idx`, …;
  - `dicts/` with the zstd dictionaries the segments reference, content-addressed.
- **Segments are append-only**: once written, a byte never changes. This is the property that makes tailing safe. The rejected alternative was an in-file trailing seek table that is rewritten on append: a reader tailing at the old table's offset can read a mix of old table bytes and new frame bytes (model control (c), §6).
- **Rotation.** A segment closes, and the next begins, at 64 MiB or on `seal()`. A closed segment ends with a zstd-seekable trailing seek table, so plain `zstd -d` and standard seekable tools work on it. A closed segment is immutable.

## 2. Framing

A segment is a sequence of zstd frames, so `zstd -d 000001.bxl` decompresses the data frames and skips the rest:

1. **Header**, a zstd skippable frame (magic `0x184D2A50`). Its payload is CBOR `{0: "BXL", 1: major, 2: minor, 3: session_id, 4: segment_no, 5: dict_id or 0, 6: writer_id (ULID), 7: created}`.
2. **Data frames**, standard zstd frames with the content checksum flag set and `dict_id` if any. The decompressed content is CBOR: `[frame_seq, first_event_seq, n, source_cursor | null, record₁ … recordₙ]`.
   - `frame_seq` is dense across segments.
   - `source_cursor` is the watcher's position after this frame ([[lc-watch-sink]] §3).
   - A record is an [[lc-ir]] Event, a Session header update, or a **blob** `{blake3, bytes}`. A blob is written once, before the first event that refers to it, and is the content-addressed table for large repeated strings (system prompts, tool definitions, images).
3. **Close**: at seal, a skippable frame holding the seek table (zstd-seekable layout: per frame, compressed and decompressed size, plus our `first_event_seq`).

**When a frame closes**, whichever comes first:
- a TurnEnd event;
- 256 events;
- 1 MiB of uncompressed content;
- 250 ms after the first unflushed event (the tail-latency bound, measured on an injectable clock);
- an explicit `flush()`.

## 3. Writer contract

- **One writer per segment**: an exclusive, non-blocking `flock` at open. A second writer gets `writer_busy` immediately and never waits. Ownership is per session log, not global (no chokepoint).
- `append(events)` buffers. A frame becomes **visible** when it is completely written. A frame is **durable** after `fsync`.
- `flush() -> FrameSeq` makes the frame visible. `sync() -> FrameSeq` makes it durable. Data first, then the index.
- **Index sidecar** `NNNNNN.idx`: CBOR `{segment_len, last_frame_seq, last_frame_xxh64, entries[(offset, frame_seq, first_event_seq)]}`. It is written to a temporary file, fsynced and `rename`d after a sync. It is a cache: a reader treats it as valid only when `segment_len ≤ file length` and the frame at its last offset hashes to `last_frame_xxh64`. Otherwise it scans.
- **Recovery** on writer open:
  1. Scan forward from the last valid index entry.
  2. Accept each frame whose zstd header, content checksum, CBOR shape and `frame_seq` are all consecutive.
  3. Truncate the file after the last accepted frame.
  4. Rewrite the index.

  Truncation removes only a torn tail that no reader has accepted (§4). Frames that were fsynced are never removed: recovery stops only at an invalid frame, and an fsynced frame is valid.

## 4. Reader and tail contract

- **A reader yields an event only from a frame it has fully validated** (header, checksum, CBOR, consecutive `frame_seq`).
- A **cursor** is `(segment_no, offset, frame_seq, index_in_frame)`. It always sits at a frame boundary plus an index into an already-validated frame.
- `Tail::next()`, at the end of the valid data:
  - It waits, using an fs notification with a polling fallback. It **never** blocks the writer.
  - If the file is shorter than the cursor offset (writer recovery truncated a torn tail the reader had not accepted), it re-reads from the cursor.
  - At a segment's seal frame, it moves to the next segment.
- A reader holds at most one decompressed frame in memory (≤ 1 MiB plus the dictionary).
- **Delivery to a reader**: every event appears exactly once, in `seq` order, for a cursor that moves forward. Resuming from a saved cursor continues with no gap and no duplicate.

## 5. Versioning and dictionaries

- **Version.** A reader refuses a different major version with `bxl_version{found, supported}`. A minor version may add CBOR keys and record kinds; a reader keeps unknown keys and returns unknown kinds as IR `Unknown`.
- **Dictionaries**:
  - one per provider, content-addressed by `blake3` and identified by its zstd `dict_id`, stored in `dicts/` and shipped in the crate;
  - a reader missing a dictionary gets `dictionary_missing{dict_id}`, never garbage;
  - **a dictionary is trained only on synthetic or sanitized and reviewed corpora**, because a zstd dictionary is made of fragments of its training data, so it would publish real log content (d4947).

## 6. Model: `LcAppendLog` (TLA+, bboxy450-tlc + Java TLC parity)

- **State:**
  - file bytes, abstracted as a sequence of frames with states `writing`, `complete` or `synced`;
  - the index (`entries`, `segment_len`, `valid`);
  - two readers, each with a cursor and the set of events it has delivered;
  - writer `crashed` / `running`.
- **Actions:** BeginFrame, WriteSomeBytes, CompleteFrame, Sync, WriteIndex, Crash (anywhere), Recover, ReaderStep, Seal/Rotate.
- **Bounds:** 3 frames, 2 events per frame, 2 readers, 1 rotation.

**Safety:**
- `NoTornRead`: a delivered event belongs to a complete frame.
- `SyncedSurvives`: a synced frame is never truncated.
- `NoGapNoDup`: each reader's delivered events are a prefix of the log.
- `IndexSound`: a reader using a valid index lands on a frame start.

**Liveness:** `TailCatchesUp` (WF on ReaderStep): every complete frame is eventually delivered to every live reader.

**Broken controls:**
- (a) a reader yields before checksum validation → `NoTornRead` fails;
- (b) the index is written before the data sync, and recovery trusts it → `SyncedSurvives` / `IndexSound` fails;
- (c) the in-file seek table is overwritten in place → `NoTornRead` fails for a reader parked at the table offset.

## 7. Verified by

| Clause | Evidence |
|---|---|
| §2 compatibility | `zstd -d` (pinned in the tools lock) decompresses a sealed segment to the concatenated CBOR frames; `zstd -t` passes |
| §3 recovery | `tests/torn.rs`: truncate a written log at every byte offset of the last two frames; reopen; the result is the longest valid frame prefix, and appending continues with the next `frame_seq` |
| §3 single writer | a second `Writer::open` returns `writer_busy` without blocking (with a 1 s timeout as a test guard only) |
| §4 tail | `tests/tail.rs`: a writer process appends while 2 reader processes tail; the events each reader gets equal the written sequence exactly; a reader restarted from a saved cursor continues with no gap or duplicate |
| §5 dictionaries | the dictionary training input is listed in `dicts/SOURCES.md` and contains only `fixtures/synthetic/**` or reviewed sanitized bundles; a check refuses any other path |
| §6 model | `make test-formal` with `LcAppendLog` and its three controls, native + Java parity |

Owner: Nyx Copperwing <ARCHITECT>.
