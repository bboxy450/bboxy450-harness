---
id: "0ERX7PT0Z3S73CS5EFTEHN8VA4"
number: 6
type: interface
title: "ICD: protocol — Op, Event, Envelope, Item lifecycle (v0.1)"
status: baselined
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[harness-zoomed-out]]", rel: implements }
  - { to: "[[gateway]]", rel: carried-by }
  - { to: "[[architect-review-v0]]", rel: amended-by }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# ICD: protocol — Op, Event, Envelope, Item lifecycle

| | |
|---|---|
| Side A | `engine` + `store` (E1): the only producer of `Envelope`, the only consumer of `Op` |
| Side B | every face: `gui` (E2), `headless`, `acp`, `seat` line mode; tests and fixtures |
| Module | `src/protocol/`. Depends on `serde`, `serde_json`, `ulid` only: no tokio, no toolkit |
| Version | `PROTOCOL: Version { major: 0, minor: 1 }` |
| Change rule | Additive changes are minor: a new variant behind `Unknown`, or a new optional field. Anything else is a major change, which needs both owners and a decision record. A face's question about this boundary is a missing line here |

## 1. Identity and ordering

```rust
pub struct SessionId(Ulid);   pub struct TurnId(Ulid);
pub struct ItemId(Ulid);      pub struct OpId(Ulid);     // OpId is minted by the client
pub struct Seq(u64);          // per session, durable events only, starts at 1, gapless, never reused
pub struct Epoch(u32);        // +1 each time the store opens the session log for writing
```

- The store assigns `Seq` at append, and `Seq` is the order of truth. Ephemeral events carry no `Seq`.
- Within one `Epoch`, a published `Seq` is never revoked. The store writes (`write(2)`) before it publishes, so a *process* crash loses nothing that was published. A kernel crash or power loss can lose the unsynced tail; the next `Epoch` then starts at the surviving head. A subscriber holding `seq > head` receives `Reset` (§3).
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

For `StartSession`, `OpEnvelope.session` is a new `SessionId` minted by the client. If that id already exists, `send` returns `Refused::SessionExists`, unless the `op_id` is the same, in which case the call is a duplicate `Ack`.

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
  | `StartSession` | `SessionStarted`, seq 1. Profile resolution follows, also caused by this op: `ProfileChanged`, or a `Notice{Error}` item | — (refusals are synchronous: `SessionExists`) |
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
    ItemStarted      { turn: Option<TurnId>, item: ItemId, kind: ItemKind, parent: Option<ItemId> },
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
    Terminal     { terminal: String, command: String, exit: Option<i32>, signal: Option<i32>, output: Option<BlobRef> },
    Plan         { entries: Vec<PlanEntry> },
    Approval     { subject: ApprovalSubject, options: Vec<ApprovalDecisionKind>,
                   decision: Option<ApprovalDecision>, by: Option<Resolver> },  // Resolver = Face(name) | Rule(id) | Unattended
    Image        { mime: String, blob: BlobRef },
    Compaction   { summary: BlobRef, replaced_through: Seq },
    Usage        { usage: Usage },
    Notice       { level: Level, text: String },          // retries and warnings; redacted
    Subagent     { session: SessionId },
    #[serde(other)] Unknown,
}
pub struct BlobRef { pub sha256: String /* hex */, pub len: u64, pub mime: String }  // stored at sessions/<id>/blobs/<sha256>
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
    Plan, Approval, Image, Compaction, Usage, Notice, Subagent, #[serde(other)] Unknown,
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
    pub effect: Effect,                                        // Write | Exec | Network (law 6)
    pub summary: String,                                       // one redacted human line
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,                               // redacted argv for Exec
}
pub enum Effect { Write, Exec, Network }
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

The completed body is authoritative, so a face that missed deltas loses nothing. `parent` nests tool output, approvals and subagents under their tool call.

**Recovery.** A log that ends in `SessionClosed` is loaded read-only and needs no recovery. When the store opens any other log whose tail has items or turns without terminal events, it appends the following in the new epoch:
1. `ItemCompleted{status: Interrupted}` for each open item, innermost first.
2. `TurnEnded{reason: Interrupted}` for each open turn.
3. `SessionRecovered`.

A `ToolCall` closed this way carries the synthetic output `"tool call was not completed"`, so the provider history stays paired.

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
| §1 epoch and `Reset` | model SubscriberReplay + BrokenSubscriberReplay; `tests/store_recovery.rs` truncates a log tail and asserts `Reset` |

`src/protocol/laws.rs` exports `pub fn check_transcript(&[Envelope]) -> Result<(), LawViolation>`. It is the executable form of laws 1, 2 and 5. Every e2e test and every GUI fixture test calls it, and fixtures are rejected if they fail it.

## Change log

- **2026-10-31, v0.1 completed at E1's question** (message 01M6ADW6HBB0YCX5Z746AJ7WPG). This adds §4a, the outcome table, the control-lane bound and collisions, and `SessionClosed`. `Close` loses its `mode`, because detaching is dropping a subscription. `Phase` becomes struct variants. Nothing had been implemented against the earlier text, so the version is unchanged.
- **2026-10-31, at E1's second question** (message 01M6AE3W63JVE3QANQ392DN2HH). This adds the tag keys `InputBlock`=`type`, `ItemStatus`=`status`, `StopReason`=`reason` and `Delta`=`type`. It also adds `ErrorClass::Cancelled` to the enum, matching provider ICD §1. Version unchanged.

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: E2.
