---
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
    Close           { mode: CloseMode },
}
pub enum TurnMode { Queue, Steer }
pub enum InputBlock { Text { text: String }, Image { blob: BlobRef }, Mention { path: PathBuf } }
pub enum ApprovalDecision { Allow, AllowAlways, Deny { reason: Option<String> } }
pub enum CloseMode { Detach, Stop }
```

- **Lanes.**
  - The *control lane* carries `Interrupt`, `ResolveApproval` and `Close`. Each op type has its own latched slot, and the control lane never returns `Busy`.
  - The *work lane* carries every other op. Its inbox is bounded at 64; when the inbox is full, `send` returns `Busy`. See law [[never-block]].
- **Idempotence.** A repeated `op_id` returns the first `Ack` and has no second effect. Memory is bounded: the session keeps its last 1024 op ids.
- **Outcome.** `Ack` means *admitted*, never *executed*. Every acked op produces exactly one outcome event with `cause: Some(op_id)`:
  - `UserTurn{Queue}` → `TurnQueued`
  - `UserTurn{Steer}` → the steer's `ItemStarted{UserMessage}`
  - a refusal after the ack → `OpRejected`
  - `Interrupt` → `TurnEnded{Cancelled}` of its target, or `OpRejected{NoSuchTurn}` if there is none
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
    // ephemeral (seq = None)
    ItemDelta        { item: ItemId, delta: Delta },
    ItemSnapshot     { item: ItemId, partial: Item },    // on subscribe and after coalescing, for each open item
    Status           { phase: Phase },                   // Idle | Queued(u32) | Thinking | Streaming | Tool | Approval | Retrying{attempt, wait_ms}
    // subscription control (ephemeral; always the last envelope on its stream)
    Lagged           { resume_after: Seq },
    Reset            { epoch: Epoch, head: Seq },
    #[serde(other)] Unknown,
}

pub enum ItemStatus { Completed, Failed { reason: String }, Cancelled, Interrupted }
pub enum StopReason {
    EndTurn, Cancelled, MaxTokens, MaxRounds, Interrupted,
    Error { class: ErrorClass, message: String },        // message is redacted
}
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

**Lifecycle per item id.**
1. `ItemStarted`, durable.
2. Zero or more `ItemDelta`, ephemeral.
3. `ItemCompleted`, durable, exactly once, carrying the *full* body.

The completed body is authoritative, so a face that missed deltas loses nothing. `parent` nests tool output, approvals and subagents under their tool call.

**Recovery.** When the store opens a log whose tail has items or turns without terminal events, it appends the following in the new epoch:
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

Owner: Nyx Copperwing <ARCHITECT>. Side A: E1. Side B: E2.
