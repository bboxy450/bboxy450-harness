---
id: "0ERX7PT0Z3S73CS5EFTEHN8VA4"
number: 6
type: interface
title: "ICD: protocol — Op, Event, Envelope, Item lifecycle (v0.4)"
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[harness-zoomed-out]]", rel: implements }
  - { to: "[[gateway]]", rel: carried-by }
  - { to: "[[architect-review-v0]]", rel: amended-by }
created: "2026-10-31T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: protocol — Op, Event, Envelope, Item lifecycle

| | |
|---|---|
| Side A | `engine` + `store` (E1): the only producer of `Envelope`, the only consumer of `Op` |
| Side B | every face: `gui` (E2), `headless`, `acp`, `seat` line mode; tests and fixtures |
| Module | `src/protocol/`. Depends on `serde`, `serde_json`, `ulid` only: no tokio, no toolkit |
| Version | `PROTOCOL: Version { major: 0, minor: 4 }` (0.2 adds `ItemStarted.head` and `Terminal.live`; 0.3 adds `Item::ContextTrim`; 0.4 adds `Effect::Read`) |
| Change rule | Additive changes are minor: a new variant behind `Unknown`, or a new optional field. Anything else is a major change, which needs both owners and a decision record. A face's question about this boundary is a missing line here |

## 1. Identity and ordering

```rust
pub struct SessionId(Ulid);   pub struct TurnId(Ulid);
pub struct ItemId(Ulid);      pub struct OpId(Ulid);     // OpId is minted by the client
pub struct Seq(u64);          // per session, durable events only, starts at 1, gapless; never reused within an epoch
pub struct Epoch(u32);        // +1 each time the store opens the session log for writing; never reused (§1 open)
```

- The store assigns `Seq` at append, and `Seq` is the order of truth. Ephemeral events carry no `Seq`.
- Within one `Epoch`, a published `Seq` is never revoked. The store writes (`write(2)`) before it publishes, so a *process* crash loses nothing that was published. A kernel crash or power loss can lose the unsynced tail; the next `Epoch` then appends from the surviving head, so a lost seq can be **reassigned** to a different event. A durable position is therefore `(Epoch, Seq)`, not `Seq` alone. Which old-epoch positions are still valid is the *trusted head* rule ([[gateway]] §2).
- **Open for writing**, in this order, before the writer is admitted or any event is published:
  1. Read the log and drop a torn last line. `fsync` the log, so the surviving head is durable and is the new epoch's base.
  2. `epoch = max(hwm, max epoch in the log) + 1`, where `hwm` is the decimal `u32` in `sessions/<id>/epoch` (absent = 0).
  3. Write `epoch` to `sessions/<id>/epoch` atomically: temp file, `fsync`, rename, `fsync` the directory.

  An epoch's events can only be published after its number is durable, so losing every record of an epoch can never make its number reusable. An unreadable `epoch` file fails the open (`Refused::Closed`, [[gateway]] §1b); it is never guessed. A fork writes the child's `epoch` file (`1`) inside its atomic create (§4c).
- The engine mints every ULID except `OpId`. The kind of an `ItemId` never changes.

## 2. Ops (face → engine)

```rust
pub struct OpEnvelope { pub v: Version, pub op_id: OpId, pub session: SessionId, pub op: Op }

#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    StartSession    { cwd: PathBuf, profile: Option<String>, fork: Option<(SessionId, Seq)> },
    UserTurn        { blocks: Vec<InputBlock>, mode: TurnMode },
    Interrupt       { turn: Option<TurnId> },            // None = the running turn
    ResolveApproval { approval: ItemId, decision: ApprovalDecision },
    SetProfile      { profile: String },                 // idle only
    Compact,
    Close,                                               // stop the session; detaching is dropping a Subscription, not an op
}
pub enum TurnMode { Queue, Steer }
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputBlock { Text { text: String }, Image { blob: BlobRef }, Mention { path: PathBuf } }
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ApprovalDecision { Allow, AllowAlways, Deny { reason: Option<String> } }
```

For `StartSession`, `OpEnvelope.session` is a new `SessionId` minted by the client, and the envelope goes to the async `Host::create` ([[gateway]] §1b), not to `Gateway::send`. If that id already exists, `create` returns `Refused::SessionExists`, unless the `op_id` is the same, in which case the call is a duplicate `Ack`.

- **Lanes.**
  - The *work lane* carries `StartSession`, `UserTurn`, `SetProfile` and `Compact`. It is a FIFO inbox bounded at 64. When the inbox is full, `send` returns `Busy`.
  - The *control lane* carries `Interrupt`, `ResolveApproval` and `Close`. It is a separate FIFO bounded at 256, and the actor drains it before the work lane on every poll (`#[yields_to]`).
  - 256 is above the largest number of control ops that can legitimately be outstanding: 65 turns that can be interrupted, plus at most 64 pending approvals, plus 1 close, gives 130. A control `Busy` is therefore reachable only by flooding duplicates. The SteerQueue model checks this bound. See law [[never-block]].
- **Control collisions**, decided when the actor consumes the op, in admission order:
  - `Interrupt{turn: None}` targets the turn that is running *at consume time*. Queued turns are not affected.
  - Two interrupts that resolve to the same turn: the first gets the `TurnEnded`, the second gets `OpRejected{Coalesced{into}}`.
  - Two `ResolveApproval` ops for one approval: the first is applied, the second gets `OpRejected{AlreadyResolved}`.
  - `Close` after `Close`: the second gets `OpRejected{Coalesced{into}}`.
  - Any op after `SessionClosed`: `send` returns `Refused::Closed`.
- **Idempotence.** A repeated `op_id` returns the first `Ack` and has no second effect. Memory is bounded: the session keeps its last 1024 op ids.
- **Outcome.** `Ack` means *admitted*, never *executed*. Every acked op has exactly one *outcome*: the first durable event whose `cause` is `Some(op_id)`. Later effects may carry the same cause.

  | Op | Outcome on success | Outcome on refusal (`OpRejected{reason}`) |
  |---|---|---|
  | `StartSession` | `SessionStarted`, seq 1. Profile resolution follows, also caused by this op: `ProfileChanged`, or a `Notice{Error}` item | — (refusals are synchronous from `Host::create`: `SessionExists`, `BadFork`) |
  | `UserTurn{Queue}` | `TurnQueued` | `InvalidInput` |
  | `UserTurn{Steer}` | the steer's `ItemStarted{UserMessage}` in the running turn, or `TurnQueued` if no turn is running | `InvalidInput` |
  | `SetProfile` | `ProfileChanged` | `NotIdle`, `UnknownProfile`, `ProfileInvalid` |
  | `Compact` | `ItemStarted{Compaction}` with `turn: None`. It runs when idle, in FIFO order with turns | `NothingToCompact` |
  | `Interrupt` | `TurnEnded{Cancelled}` of the target | `NoSuchTurn`, `Coalesced` |
  | `ResolveApproval` | the approval's `ItemCompleted` | `NoSuchApproval`, `AlreadyResolved` |
  | `Close` | `TurnEnded{Cancelled}` of the running turn, if any, else `SessionClosed`. After the running turn, `TurnEnded{Cancelled}` follows for each queued turn, and then `SessionClosed` | `Coalesced` |

- **No profile.** A session whose profile failed to resolve still accepts turns. Each such turn ends `Error{class: Config}`.
- **Steer.**
  - A steer is delivered into the running turn at the next round boundary, as a `UserMessage` item.
  - If the turn ends before that, the steer becomes a queued turn.
  - Either way, an acked steer appears exactly once.
- **Refusals.**
  - `SetProfile` while a turn runs → `OpRejected{reason: NotIdle}`.
  - `ResolveApproval` on an already resolved approval → `OpRejected{reason: AlreadyResolved}`. The first resolution wins.
- **Turn queue.**
  - An `Interrupt` addressed to a queued turn gives `TurnEnded{Cancelled}` with no `TurnStarted`.
  - Every `TurnQueued` gets exactly one `TurnEnded`.

## 3. Envelope and events (engine → face)

```rust
pub struct Envelope {
    pub v: Version,
    pub session: SessionId,
    pub epoch: Epoch,
    pub seq: Option<Seq>,   // Some ⇔ durable: persisted, replayable
    pub at: Seq,            // durable head when emitted; orders an ephemeral event against durable ones
    pub ts_ms: u64,
    pub cause: Option<OpId>,
    pub event: Event,
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    // durable (seq = Some)
    SessionStarted   { meta: SessionMeta },              // seq 1 of every session
    SessionRecovered { prior_epoch: Epoch, closed_items: u32, closed_turns: u32 },
    ProfileChanged   { profile: String, model: String },
    TurnQueued       { turn: TurnId },
    TurnStarted      { turn: TurnId },
    ItemStarted      { turn: Option<TurnId>, item: ItemId, kind: ItemKind, parent: Option<ItemId>,
                       #[serde(default)] head: Option<ItemHead> },   // identity known at start; see §4
    ItemCompleted    { item: ItemId, status: ItemStatus, body: Item },
    TurnEnded        { turn: TurnId, reason: StopReason, usage: Option<Usage> },
    OpRejected       { op: OpId, reason: Rejection },
    SessionClosed,                                       // last durable event; a closed log is not recovered as interrupted
    // ephemeral (seq = None)
    ItemDelta        { item: ItemId, delta: Delta },
    ItemSnapshot     { item: ItemId, partial: Item },    // on subscribe and after coalescing, for each open item
    Status           { phase: Phase },
    // subscription control (ephemeral; always the last envelope on its stream)
    Lagged           { resume_after: Seq },
    Reset            { epoch: Epoch, head: Seq },
    #[serde(other)] Unknown,
}

#[serde(tag = "status", rename_all = "snake_case")]
pub enum ItemStatus { Completed, Failed { reason: String }, Cancelled, Interrupted }
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum StopReason {
    EndTurn, Cancelled, MaxTokens, MaxRounds, Interrupted,
    Error { class: ErrorClass, message: String },        // message is redacted
}
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
    Text      { append: String },                        // AgentMessage
    Reasoning { append: String },
    ToolInput { append: String },                        // raw JSON fragment as streamed
    Output    { stream: Stream, append: String },        // Terminal / ToolCall; UTF-8 lossy; ≤16 KiB per delta
    Plan      { entries: Vec<PlanEntry> },               // replaces the plan, does not append
}
```

A turn has exactly one terminal event, `TurnEnded`. An engine error is `StopReason::Error`, not a separate event type ([[architect-review-v0]] §1).

## 4. Items

```rust
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Item {
    UserMessage  { blocks: Vec<InputBlock> },
    AgentMessage { markdown: String },
    Reasoning    { summary: String, raw: Option<String>, provider_opaque: Option<Value> }, // opaque: replayed verbatim, never rendered
    ToolCall     { call_id: String, name: String, tool_kind: ToolKind, input_raw: String,
                   output: Vec<OutputBlock>, locations: Vec<Location>, spill: Option<BlobRef> },
    FileChange   { path: PathBuf, change: ChangeKind, unified: String },
    TurnDiff     { unified: String },
    Terminal     { terminal: String, command: String, exit: Option<i32>, signal: Option<i32>, output: Option<BlobRef>,
                   #[serde(default)] live: Option<String> },   // snapshots only; always None in ItemCompleted
    Plan         { entries: Vec<PlanEntry> },
    Approval     { subject: ApprovalSubject, options: Vec<ApprovalDecisionKind>,
                   decision: Option<ApprovalDecision>, by: Option<Resolver> },  // Resolver = Face(name) | Rule(id) | Unattended
    Image        { mime: String, blob: BlobRef },
    Compaction   { summary: BlobRef, replaced_through: Seq },
    ContextTrim  { clipped_through: Seq },                  // §4d: older items are sent clipped
    Usage        { usage: Usage },
    Notice       { level: Level, text: String },          // retries and warnings; redacted
    Subagent     { session: SessionId },
    #[serde(other)] Unknown,
}
pub struct BlobRef { pub sha256: String /* hex */, pub len: u64, pub mime: String }  // stored at sessions/<id>/blobs/<sha256>
#[serde(tag = "kind", rename_all = "snake_case")]          // same strings as ItemKind
pub enum ItemHead {
    ToolCall { call_id: String, name: String, tool_kind: ToolKind },
    Terminal { terminal: String, command: String },
    #[serde(other)] Unknown,
}
pub enum ToolKind { Read, Edit, Delete, Move, Search, Execute, Think, Fetch, Other }  // maps 1:1 to ACP ToolKind
```

### 4a. Supporting types

Every enum is externally simple: unit variants serialise as snake_case strings, and data variants are internally tagged with the tag named in the comment. Every enum consumers match on ends with `#[serde(other)] Unknown`. Tuple variants carrying primitives are not used, because serde's internal tagging cannot carry them.

```rust
pub struct Version { pub major: u16, pub minor: u16 }          // {"major":0,"minor":1}
pub struct SessionMeta {
    pub session: SessionId, pub cwd: PathBuf,
    pub profile: Option<String>,                               // requested name; resolution reports ProfileChanged
    pub fork: Option<(SessionId, Seq)>,
    pub protocol: Version, pub harness: String,                // crate version, e.g. "0.1.0"
    pub created_ms: u64,
}
pub enum ItemKind {                                            // same strings as Item's "kind" tag
    UserMessage, AgentMessage, Reasoning, ToolCall, FileChange, TurnDiff, Terminal,
    Plan, Approval, Image, Compaction, ContextTrim, Usage, Notice, Subagent, #[serde(other)] Unknown,
}
pub struct Usage {
    pub input_tokens: u64, pub output_tokens: u64,
    pub cached_input_tokens: Option<u64>, pub reasoning_tokens: Option<u64>,
    pub cost_micros_usd: Option<u64>,                          // provider-reported only, never estimated
}
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputBlock {
    Text { text: String },                                     // ≤ 50 KiB inline, the rest in ToolCall.spill
    Json { value: Value },
    Image { blob: BlobRef },
    Diff { path: PathBuf, unified: String },
    Error { text: String },                                    // a model-visible tool error (is_error)
}
pub struct Location { pub path: PathBuf, pub line: Option<u32>, pub end_line: Option<u32> }  // 1-based, inclusive
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ChangeKind { Add, Modify, Delete, Rename { from: PathBuf } }
pub enum Stream { Stdout, Stderr }
pub struct PlanEntry { pub text: String, pub status: PlanStatus, pub priority: Option<Priority> }  // ACP plan entry 1:1
pub enum PlanStatus { Pending, InProgress, Completed, Cancelled }
pub enum Priority { High, Medium, Low }
pub struct ApprovalSubject {
    pub tool_call: ItemId,
    pub effect: Effect,                                        // the decided EffectClass, 1:1 (law 6)
    pub summary: String,                                       // one redacted human line
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,                               // redacted argv for Exec
}
pub enum Effect { Read, Write, Exec, Network, #[serde(other)] Unknown }  // Read: an explicit Ask rule on a read-only effect
pub enum ApprovalDecisionKind { Allow, AllowAlways, Deny }     // the options a face may offer
#[serde(tag = "by", rename_all = "snake_case")]
pub enum Resolver { Face { name: String }, Rule { id: String }, Unattended, Cancelled }
pub enum Level { Info, Warn, Error }
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum Phase {
    Idle, Queued { turns: u32 }, Thinking, Streaming,
    Tool { item: ItemId }, Approval { item: ItemId },
    Retrying { attempt: u32, wait_ms: u64 }, Closed,
}
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Rejection {
    NotIdle, UnknownProfile { name: String }, ProfileInvalid { detail: String },  // detail is redacted
    NoSuchTurn, NoSuchApproval, AlreadyResolved, NothingToCompact,
    InvalidInput { detail: String }, Coalesced { into: OpId },
    #[serde(other)] Unknown,
}
pub enum ErrorClass {                                          // shared with provider ICD §1
    Transient, RateLimited, Auth, BadRequest, Malformed, MidStream, Cancelled, Config, Store, Tool, Internal,
}
```

`ErrorClass::Cancelled` exists for `ProviderError` and is serialised as `"cancelled"`. It never appears inside `StopReason::Error`, because a cancelled turn ends `StopReason::Cancelled`. `check_transcript` rejects `Error{class: cancelled}`.

**Lifecycle per item id.**
1. `ItemStarted`, durable.
2. Zero or more `ItemDelta`, ephemeral.
3. `ItemCompleted`, durable, exactly once, carrying the *full* body.

The completed body is authoritative, so a face that missed deltas loses nothing.

**Head.** `ItemStarted` for a `ToolCall` or a `Terminal` must carry `head`; for other kinds it is `None`. A `ToolCall` item starts when the provider stream has given both `id` and `name` for its index; `ToolInput` deltas that arrive earlier are buffered and emitted after the start. The completed body's `call_id`, `name`, `tool_kind` (or `terminal`, `command`) equal the head.

**Live terminal output.** `Item::apply(Delta::Output)` on a `Terminal` appends `append` to `live` (interleaving stdout and stderr in arrival order) and keeps only the last 64 KiB: it cuts at a char boundary and prefixes `…`. An `ItemSnapshot` of an open `Terminal` therefore carries the same `live` a face would hold had it seen every delta since the cut. `ItemCompleted` carries `live: None`; the full output is in `output`. `parent` nests tool output, approvals and subagents under their tool call.

**Recovery reads only the log.** No runtime projection survives a process restart, so a closure is built the same way whether or not the process restarted. A closed item's body is the kind's empty body (empty strings and vectors, `None` options) with the identity fields taken from `head`.

**Recovery.** A log that ends in `SessionClosed` is loaded read-only and needs no recovery. When the store opens any other log whose tail has items or turns without terminal events, it appends the following in the new epoch:
1. `ItemCompleted{status: Interrupted}` for each open item, innermost first.
2. `TurnEnded{reason: Interrupted}` for each open turn.
3. `SessionRecovered`.

A `ToolCall` closed this way carries the synthetic output `"tool call was not completed"`, so the provider history stays paired.

### 4b. Compaction and context replay

Phase 1 compaction is **deterministic and makes no model call**: the summary is a pure digest of the log, so the same log prefix always gives the same blob bytes and `sha256`. A model-written summary is a later decision; it would need an inference budget, and replay would still read the stored blob, never regenerate it.

- **Window.** Let `C` be the latest `ItemCompleted{Compaction, status: completed}` (`r0 = C.replaced_through`, or 0 if none). When `Compact` runs (idle, FIFO with turns), `r` is the seq of the last `TurnEnded` in the log. If there is no `TurnEnded` with seq > `r0`, the outcome is `OpRejected{NothingToCompact}`. Otherwise the window is the turns whose `TurnEnded` seq is in `(r0, r]`.
- **Digest.** `pub fn compact_summary(prev: Option<&str>, window: &[Envelope]) -> String` in `src/context/compact.rs`, UTF-8 Markdown, mime `text/markdown`:
  ```text
  # Earlier conversation (compacted through seq <r>)
  <prev, without its first line>
  ## Turn <TurnStarted seq>
  User: <Text blocks joined by "\n", clipped to 2000 chars; Image → [image <sha256[..8]>]; Mention → @<path>>
  Tool: <name> <input_raw clipped to 200> → <ItemStatus tag>[ exit <n>]      (one line per ToolCall; Terminal likewise with its command)
  Changed: <path> (<ChangeKind tag>)                                         (one line per FileChange)
  Assistant: <the turn's last AgentMessage, clipped to 4000 chars>
  Ended: <StopReason tag>
  ```
  Only completed item bodies are read. `Reasoning` (including `provider_opaque`), `Usage`, `Notice`, `Plan`, `Image` items, tool and terminal output, and `TurnDiff` are dropped. A line whose source is absent is omitted. *Clipped to n* counts Unicode scalar values and appends `…` when it cuts.
  - **Unstarted turns.** A turn with no `TurnStarted` (queued, then cancelled or closed) is omitted from the digest. The model never saw it.
  - **One line per field.** Every interpolated value is clipped first, then escaped: `\` → `\\`, LF → `\n`, CR → `\r`, TAB → `\t`, and every other C0/C1 control character, U+2028 and U+2029 → `\u{XXXX}` (uppercase hex, at least 4 digits). So a digest line starts with `## Turn ` only when the digest wrote that header.
  - **Parsing `prev`.** Line 1 is the heading. Line 2 is `(<k> earlier turns omitted)` if present. Every later line starting with `## Turn ` begins a section, which runs to the next one. Nothing else in `prev` is interpreted.
- **Cap.** The digest is at most 32 KiB. Over that, whole `## Turn` sections are dropped oldest first (those inherited from `prev` included), and one line `(<k> earlier turns omitted)` follows the heading, where `k` accumulates across compactions.
- **Item.** `ItemStarted{Compaction}` with `turn: None`, no deltas, then `ItemCompleted{Compaction{summary, replaced_through: r}}`. The blob is written and fsynced before `ItemCompleted` is appended.
- **Replay rule (what the model sees).** A request's messages are: the system and tool prefix; then, if `C` exists, one user-role message whose content is the summary blob text; then the wire mapping of every item whose `ItemStarted` seq is > `C.replaced_through`, in log order. Items at or below `replaced_through` never enter a request again. The log is not rewritten: faces still show the full history (law [[log-is-truth]]). `stable_prefix` covers the system prefix plus the summary message, so it changes only at a compaction.
- **No automatic ladder in phase 1.** Clip/drop/summarize on overflow ([[harness-zoomed-out]] `context`) is deferred. An over-long request fails as `ErrorClass::BadRequest`, the turn ends with `StopReason::Error`, and the face offers `Compact`.

### 4d. Automatic compaction

Automatic compaction keeps a request inside the model's window without the user asking. Every step is recorded as a durable item, so a request stays a pure function of the log (§4b replay rule) and the cached prefix changes only at those items.

- **Window `W`.** The profile's `context_tokens` if set, otherwise `Caps::context_tokens` ([[provider]] §1). If neither is known, automatic compaction is off for the session and one `Notice{Warn, "context window unknown; automatic compaction off"}` is appended at the first round. The profile field `auto_compact: bool` (default `true`) turns it off explicitly.
- **Estimate `E`.** Before each round's request, from exactly two inputs: the log, and the turn's tool snapshot ([[mcp]] §2), which is fixed for the turn.
  - **Serialization.** `bytes(x)` is the length of `serde_json::to_vec(x)`, compact with no whitespace, of the value exactly as it goes into the request body. Model, `max_tokens`, `stream`, `tool_choice` and other request metadata are never counted.
  - **With a `Usage` item in the session:** `E = input_tokens + output_tokens` of the latest one, plus `ceil(Σ bytes(m) / 4)` over the wire messages `m` that come from items started after that `Usage` item. The `Usage` already includes the tools as they were then. A tool snapshot that changed since then is not re-counted; with at most 128 MCP tools, the 20 % margin covers it.
  - **Anchor validity.** The latest `Usage` is a valid anchor only if no `ContextTrim` or `Compaction` item completed after it. A reduction changes the request that the `Usage` measured, so after one the anchor is void. Then the no-`Usage` formula below runs over the rebuilt request, until the next round reports a new `Usage`. This is what lets the ladder's re-estimate go down.
  - **With no valid anchor** (no `Usage` yet, or one voided by a reduction): `E = ceil((Σ bytes(m) over all messages + bytes(tools)) / 4)`, where `tools` is the request's `tools` array.
  - The same log and the same snapshot always give the same `E`. Only the trigger decision reads `E`. The request itself is rebuilt from the durable `ContextTrim` and `Compaction` items, so a restart with a different snapshot cannot change a rebuilt request.
- **Trigger.** At a round boundary (before the request, never during a stream), if `E + max_tokens > 0.8 · W`. `max_tokens` is the request's value, or `W / 8` when it is unset.
- **Ladder.** Each step reduces only *completed turns*, whose `TurnEnded` seq is ≤ `r`, the latest `TurnEnded` in the log. The current turn is never reduced. After each step the engine re-estimates and stops when `E + max_tokens ≤ 0.6 · W`.
  1. **Trim** (clip and drop together). Append `ContextTrim{clipped_through: r}`. In requests, every item with `ItemStarted` seq ≤ `clipped_through` (and above the latest `replaced_through`) is sent with: tool and terminal output cut to its first 2 KiB plus `…[clipped <n> bytes]`; `Reasoning` items and `Image` blocks omitted. Skipped when `r` ≤ the latest `clipped_through`.
  2. **Summarize.** A §4b compaction with window `(r0, r]` and the same digest. Skipped when there is no `TurnEnded` above `r0`.
  3. **Nothing left to reduce.** The request is sent as it is. If it overflows, the turn ends with `StopReason::Error{class: BadRequest}` as in §4b. The ladder never runs again in the same round, so it cannot loop.
- **Items.** Both steps are items in the current turn: `ItemStarted{turn: Some(current), kind}` then `ItemCompleted`, before the round's request is built. A trim has no blob; a summary follows §4b (blob fsynced before `ItemCompleted`). A manual `Op::Compact` is unchanged: it runs when idle, with `turn: None`.
- **Cache stability.** `clipped_through` and `replaced_through` only increase, and only at these items. Between them, `messages[..stable_prefix]` is byte-identical from round to round ([[provider]] §2 rule 6). The prefix is the system prompt and tools, plus the summary message, plus the trimmed range. Each trim or summary costs exactly one cache miss.
- **Recovery and fork.** Both items are ordinary durable items, so recovery and fork (§4c) carry them unchanged. A restart rebuilds the same request.

### 4c. Fork

Phase 1 fork **materializes** the parent prefix into the child's own log. A child never reads its parent after creation, so replay, recovery and the gateway stay single-session.

- **Fork point.** `StartSession{fork: Some((parent, at))}` is valid when the parent log exists, `2 ≤ at ≤` its durable head, `at` is below any `SessionClosed`, and the prefix `1..=at` is *quiescent*: every `TurnQueued`, `TurnStarted` and `ItemStarted` in it has its `TurnEnded` or `ItemCompleted` in it too. Otherwise `Host::create` returns `Refused::BadFork`. A `TurnEnded` seq is always a valid point.
- **Child log.** Seq 1 is the child's `SessionStarted` with `meta.fork = Some((parent, at))`. Then, for each parent durable envelope with seq `2..=at`, a copy with **the same seq**, `event`, `ts_ms` and `cause`, and with `session` and `epoch` set to the child's. So child seq *n* is parent seq *n* for *n* ≤ `at`, and a `Compaction.replaced_through` copied from the parent stays valid. Live child events start at `at + 1`, beginning with the child's `ProfileChanged` (or `Notice`).
- **Identity.** Copied `TurnId`, `ItemId` and `call_id` values are kept. An item's identity is `(SessionId, ItemId)`, so the same `ItemId` in parent and child are different items. Copied `cause` op ids are history only: the child's idempotence memory holds only ops the child admitted, starting with the `StartSession` op id.
- **Blobs.** Every `BlobRef` in the copied range is copied (hard link if possible) into `sessions/<child>/blobs/`.
- **Atomicity.** Blobs, then the log to a temp file, fsync, rename. A crash leaves no child or a complete one. Reading the parent's lines `≤ at` is safe while the parent appends, because the log is append-only.
- **Equivalence.** The child's first request equals the request the parent would have built at `at` with the child's profile.

## 5. Wire and size

- JSON, one `Envelope` per line (`\n`), UTF-8. The same bytes go into the store, onto the socket, out of `--json` stdout, and into fixtures.
- An envelope is at most 256 KiB. Anything larger goes to a blob, and the item carries a `BlobRef`.
- Consumers accept `Unknown` variants and unknown fields: they count them and do not fail.
- A major version mismatch is refused at subscribe with `Refused::Version`.
- `schemars` exports `protocol.schema.json`. The golden fixture set is `tests/fixtures/protocol/*.jsonl`. E2 builds the GUI against these fixtures before the engine exists.

## 6. Timing

- Deltas are coalesced per item, per subscriber, at 50 ms (at most 20 Hz). `Status` is replace-latest.
- After an `Interrupt` is acked, the running turn reaches `TurnEnded{Cancelled}` within 2 s, with its process groups killed. See law [[cancel-is-cancel]].
- Rounds per turn default to 50. At the limit the turn ends with `MaxRounds`, so no turn is unbounded.

## 7. Verified by

| Clause | Evidence |
|---|---|
| §3 one terminal per turn; §4 item lifecycle and recovery | model AgentTurnLifecycle + BrokenAgentTurnLifecycle; `check_transcript` (below) run over every e2e transcript |
| §2 lanes, idempotence, steer exactly once | model InputPreemption + BrokenInputPreemption; `tests/gateway_ops.rs` |
| §5 round trip and forward compatibility | `tests/protocol_golden.rs`: every fixture line parses and re-serialises byte-identical; an injected future variant parses as `Unknown` |
| §4b compaction | `tests/context_compact.rs`: golden digests in `tests/fixtures/compact/*.md`; the same log compacted twice gives the same `sha256`; the request after a compaction contains no item with `ItemStarted` seq ≤ `replaced_through`; a second `Compact` with no new turn gives `NothingToCompact`; the 32 KiB cap drops oldest turns first; a user message containing `\n## Turn 1\n` survives two compactions as one escaped line and the section count is unchanged; a queued-then-cancelled turn adds no section. Pure function: implementation-only coverage, recorded in the formal inventory |
| §4d automatic compaction | model **HarnessAutoCompact** + broken controls (below); `tests/context_auto.rs` with the scripted provider and `context_tokens: 4000`: (a) a 30-turn session triggers trim, then summary, and every request stays ≤ 0.8 · W after the ladder; (b) two consecutive requests with no new trim or summary share a byte-identical `messages[..stable_prefix]`; (c) kill and reopen between rounds rebuilds a byte-identical request; (d) a single turn bigger than `W` sends once and ends with `Error{BadRequest}`, with no second ladder pass; (e) an unknown window yields one `Notice` and no `ContextTrim`; `check_transcript`: `clipped_through` and `replaced_through` never decrease, and are never above the latest `TurnEnded` before them |
| §4c fork | `tests/store_fork.rs`: child seq *n* equals parent seq *n* for *n* ≤ `at` (event bytes compared); a non-quiescent `at` gives `BadFork`; the parent's log hash is unchanged; deleting the parent after the fork leaves the child openable; the child's first `ModelRequest` equals the parent's at `at`; a fork taken while the parent appends copies exactly `1..=at`; `check_transcript` passes on the child |
| §4 head and live output | `check_transcript`: every `ToolCall`/`Terminal` `ItemStarted` has a `head` of matching kind, the completed body's identity equals it, and `ItemCompleted` never has `live: Some`. `tests/store_recovery.rs`: kill the process mid tool call; after reopening, the recovered `ToolCall` keeps its `call_id` and `name`, and the next `ModelRequest` pairs the tool result to it. `tests/gateway_replay.rs`: subscribe during a 200 KiB exec; the snapshot's `live` equals the last 64 KiB of the deltas |
| §1 epoch and `Reset` | model SubscriberReplay + BrokenSubscriberReplay; `tests/store_recovery.rs` truncates a log tail and asserts `Reset` |

`src/protocol/laws.rs` exports `pub fn check_transcript(&[Envelope]) -> Result<(), LawViolation>`. It is the executable form of laws 1, 2 and 5. Every e2e test and every GUI fixture test calls it, and fixtures are rejected if they fail it.

## Change log

- **2026-10-31, v0.1 completed at E1's question** (message 01M6ADW6HBB0YCX5Z746AJ7WPG). This adds §4a, the outcome table, the control-lane bound and collisions, and `SessionClosed`. `Close` loses its `mode`, because detaching is dropping a subscription. `Phase` becomes struct variants. Nothing had been implemented against the earlier text, so the version is unchanged.
- **2026-10-31, at E1's second question** (message 01M6AE3W63JVE3QANQ392DN2HH). This adds the tag keys `InputBlock`=`type`, `ItemStatus`=`status`, `StopReason`=`reason` and `Delta`=`type`. It also adds `ErrorClass::Cancelled` to the enum, matching provider ICD §1. Version unchanged.
- **2026-11-01, at E1's third question** (message 01M6AHRY2RX664DTK1S4NJ0QR8). `StartSession` is admitted by `Host::create` ([[gateway]] §1b). Version unchanged.
- **2026-11-01, at E1's fourth question** (message 01M6AJMVVHA6DCBHGFXM8BCFHM). Adds §4b (deterministic digest compaction, the replay rule, no automatic ladder in phase 1) and §4c (fork materializes the parent prefix with identical seqs). Alternatives rejected: a model-written summary (non-deterministic, spends inference outside [[budgets]]); a fork that references the parent (cross-session reads in replay and recovery, and a dangling parent on delete). Adds `Refused::BadFork` ([[gateway]]). Version unchanged.
- **2026-11-01, at E1's fifth question** (message 01M6AJW9H6MSQS31Q2VC9RNSYB). §4b digest: unstarted turns are omitted (rejected alternative: a `TurnQueued` seq header for a turn the model never saw); fields are clipped, then escaped to one line; the `prev` parse is fixed. Version unchanged.
- **2026-11-01, at the Oracle's question** (message 01M6AMASRXSGVHYFESFGWZVCRK). §1 said seqs are never reused, yet after tail loss the next epoch reassigns them. Corrected: positions are `(Epoch, Seq)`; validity is gateway §2's trusted head. Decision [[d1134-reset-trusted-head]]. Version unchanged.
- **2026-11-01, v0.2 at E1's E1b question** (message 01M6AMHT3Y47902K55F608ZY3X). Adds `ItemStarted.head` (durable identity for `ToolCall` and `Terminal`; recovery reads only the log) and `Terminal.live` (a 64 KiB tail that snapshots repair). Additive fields, so this is a minor bump. Alternatives rejected: keeping runtime identities with a synthetic fallback (a closure would then depend on whether the process restarted, which breaks [[log-is-truth]]); a separate durable identity event (one more event per item, for data that is known at start).
- **2026-11-01, at E1's question** (message 01M6AN4153XBMPVHYPEWP3TNM8). Epoch allocated from the log alone is reused when a whole epoch's tail is lost, which defeats the trusted head. Adds the open order: fsync the surviving log, then a durable epoch high-water mark outside the JSONL, then admit the writer. Amends [[d1134-reset-trusted-head]]. Version unchanged (store-internal, not on the wire).
- **2026-11-01, v0.3 at the Oracle's request** (message 01M6AVJ56YFJSKHDJ9Q6Z17QNH). Adds §4d, automatic compaction: an 80 % trigger on a log-only estimate, a trim then summarize ladder over completed turns, and the durable `ContextTrim` item. Alternatives rejected: request-only clipping (the request would depend on state that is not in the log, and every frontier move would miss the cache); a model-written summary (§4b); a reactive retry on overflow (it needs a new `ErrorClass` for a case the 80 % margin already covers). Clip and drop are one step, not two, so the ladder costs one cache miss instead of two.
- **2026-11-01, at E1's question** (message 01M6AWMVTNJ3W0WSZZ21WFVYK5). §4d estimate: `bytes` is compact `serde_json` of the request-body values; messages always count; tools count only before the first `Usage`; metadata never counts. `E` is a function of the log plus the turn's tool snapshot, not of the log alone. Version unchanged.
- **2026-11-01, at E1's question** (message 01M6AWS6G0H9448REFFMCDEDEE). §4d: a `ContextTrim` or `Compaction` after the latest `Usage` voids it as the anchor, and the estimate restarts from the rebuilt request. Version unchanged.
- **2026-11-01, v0.4 at E1's question** (message 01M6AXXPH2FPRY891W0Q5BPXJY). `Effect` gains `Read` (and `Unknown`). `ApprovalSubject.effect` is the decided `EffectClass` mapped 1:1 (`ReadOnly` → `Read`), so an explicit `Ask` rule on a read — builtin or a trusted MCP tool — carries an honest effect. The default policy still allows reads without asking. Alternative rejected: labelling a read approval `Exec`, which would show the user a false effect. Minor bump.

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: E2.
