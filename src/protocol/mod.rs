// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use ulid::Ulid;
mod agent_config;
pub use agent_config::*;
pub mod laws;
mod usage;
// Additive dream-log §2 wire contracts; no IO or hashing.
/// `sha256:` plus lowercase hex SHA-256 of canonical memory JSON:
/// {kind,title,why,how_to_apply,evidence,derived_from,revival}. Object keys sort,
/// whitespace is absent, strings are NFC, evidence/derived_from sort by canonical
/// JSON, and revival distinguishes each re-proposal after a terminal state.
/// A wire string alias preserves the spec's String API; the dreamer verifies it.
pub type EntryId = String;
/// `sha256:` plus lowercase hex SHA-256. An event hashes exact stored envelope
/// bytes excluding only the trailing LF (not CR or whitespace); a record hashes
/// file bytes at its pinned commit; a sink hashes only its dream-owned bytes.
pub type Digest = String;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Vcs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dirty: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DreamRun {
    #[serde(default)]
    pub phase: DreamPhase,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Trigger {
    SessionClosed {
        #[serde(default = "empty_session")]
        session: SessionId,
    },
    Quota {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        provider: String,
        #[serde(default)]
        resets_at: u64,
    },
    Operator {
        #[serde(default)]
        message: Option<String>,
    },
    Dream {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
    },
    Schedule {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Scope {
    #[serde(default)]
    pub level: ScopeLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repos: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceRef {
    /// Untrusted even when the line digest matches.
    Event {
        #[serde(default = "empty_session")]
        session: SessionId,
        #[serde(default)]
        seq: Seq,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        digest: Digest,
    },
    /// Trust additionally requires the dream-log §6.2 anchor checks.
    Record {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        root: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        id: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        path: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        commit: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        digest: Digest,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RecordPin {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub digest: Digest,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrialEpisode {
    #[serde(default)]
    pub origin: EvidenceRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<TrialRun>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrialRun {
    #[serde(default)]
    pub arm: Arm,
    #[serde(default = "empty_session")]
    pub session: SessionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_exit: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SinkRef {
    #[serde(default)]
    pub kind: SinkKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
}

// Defaults represent absent reader data, never a claim by a writer. Authored
// validation recursively rejects Unknown, including these missing-field values.
macro_rules! dream_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant,)+
            #[serde(untagged)]
            Unknown(Value),
        }
        impl Default for $name {
            fn default() -> Self { Self::Unknown(Value::Null) }
        }
    };
}
dream_enum!(DreamPhase {
    Replay,
    Synthesis,
    Trial,
    Lower,
    Flush,
    Audit
});
dream_enum!(ScopeLevel { Repo, System });
dream_enum!(MemoryOp {
    Pending,
    Created,
    Revised
});
dream_enum!(EntryState {
    Proposed,
    Tested,
    Adopted,
    Rejected,
    Superseded,
    Expired,
    Retired
});
dream_enum!(Arm { With, Without });
dream_enum!(Verdict {
    Pass,
    Fail,
    Inconclusive
});
dream_enum!(SinkKind {
    ClaudeMemory,
    CodexExtension,
    GrokInbox,
    EngineContext,
    AcpContext,
    Proposal
});
dream_enum!(RecallBasis {
    Reported,
    Inferred,
    Cited
});
dream_enum!(ReceiptKind { Publication, Run });
dream_enum!(ReceiptOutcome {
    Published,
    Idempotent,
    Skipped,
    Completed,
    Failed
});

// Missing source identities are a fixed sentinel, never a freshly minted ID.
fn empty_session() -> SessionId {
    SessionId(Ulid::from(0u128))
}
impl Default for Trigger {
    fn default() -> Self {
        Self::Unknown(Value::Null)
    }
}
impl Default for EvidenceRef {
    fn default() -> Self {
        Self::Unknown(Value::Null)
    }
}
pub const PROTOCOL: Version = Version { major: 1, minor: 3 };
pub const MAX_ENVELOPE_BYTES: usize = 256 * 1024;
macro_rules! identity {($($name:ident),*)=>{$(
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Serialize,Deserialize,JsonSchema)]
#[serde(transparent)]
pub struct $name(#[schemars(with="String")] pub Ulid);
impl $name {pub fn new()->Self{Self(Ulid::new())}}
impl Default for $name {fn default()->Self{Self::new()}}
impl std::fmt::Display for $name {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{self.0.fmt(f)}}
)*}}
identity!(SessionId, TurnId, ItemId, OpId);
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(transparent)]
pub struct Seq(pub u64);
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(transparent)]
pub struct Epoch(pub u32);
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OpEnvelope {
    pub v: Version,
    pub op_id: OpId,
    pub session: SessionId,
    pub op: Op,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    /// Harmless actor-level negotiation, never a model prompt or a permission grant.
    QueryInputCapabilities {
        #[serde(
            default = "missing_identity",
            skip_serializing_if = "is_missing_identity"
        )]
        request: OpId,
    },
    StartSession {
        cwd: PathBuf,
        profile: Option<String>,
        fork: Option<(SessionId, Seq)>,
    },
    UserTurn {
        blocks: Vec<InputBlock>,
        mode: TurnMode,
    },
    Interrupt {
        turn: Option<TurnId>,
    },
    ResolveApproval {
        approval: ItemId,
        decision: ApprovalDecision,
    },
    /// The human's reply to a `Question` item, as an ACP elicitation response.
    AnswerQuestion {
        #[serde(default)]
        question: ItemId,
        #[serde(default)]
        answer: Value,
    },
    SetProfile {
        profile: String,
    },
    /// A selection from a specific negotiated child and complete option revision.
    SetAgentConfig {
        #[serde(default, skip_serializing_if = "is_missing_value")]
        epoch: Epoch,
        #[serde(
            default = "missing_identity",
            skip_serializing_if = "is_missing_identity"
        )]
        instance: OpId,
        #[serde(default, skip_serializing_if = "is_missing_value")]
        revision: u64,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        config_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Compact,
    Close,
    #[serde(untagged)]
    Unknown(Value),
}
// Missing identity is a stable invalid sentinel, never a fresh child identity.
fn missing_identity() -> OpId {
    OpId(Ulid::nil())
}
fn is_missing_identity(value: &OpId) -> bool {
    value.0 == Ulid::nil()
}
fn is_missing_value<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TurnMode {
    Queue,
    Steer,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputBlock {
    Text {
        text: String,
    },
    Image {
        blob: BlobRef,
    },
    Audio {
        #[serde(default, skip_serializing_if = "BlobRef::is_empty")]
        blob: BlobRef,
    },
    Resource {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        uri: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        mime: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blob: Option<BlobRef>,
    },
    ResourceLink {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        uri: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    Mention {
        path: PathBuf,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ApprovalDecision {
    Allow,
    AllowAlways,
    Deny {
        reason: Option<String>,
    },
    #[serde(untagged)]
    Unknown(Value),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Envelope {
    /// Original JSONL bytes, including the newline. Never part of the wire schema.
    #[serde(skip)]
    #[schemars(skip)]
    pub raw: Option<Arc<str>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ext: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<Native>,

    pub v: Version,
    pub session: SessionId,
    pub epoch: Epoch,
    pub seq: Option<Seq>,
    pub at: Seq,
    pub ts_ms: u64,
    pub cause: Option<OpId>,
    pub event: Event,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Ephemeral response bound to the live subscriber's fresh challenge.
    InputCapabilities {
        #[serde(
            default = "missing_identity",
            skip_serializing_if = "is_missing_identity"
        )]
        request: OpId,
        #[serde(default, skip_serializing_if = "is_missing_value")]
        image_custody_version: u32,
    },
    SessionStarted {
        meta: SessionMeta,
    },
    SessionRecovered {
        prior_epoch: Epoch,
        closed_items: u32,
        closed_turns: u32,
    },
    ProfileChanged {
        profile: String,
        model: String,
    },
    TurnQueued {
        turn: TurnId,
    },
    TurnStarted {
        turn: TurnId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        /// Absent in logs before IR 1.2; absence means user.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        origin: Option<TurnOrigin>,
    },
    ContextChanged {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        permission_mode: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        config: Option<BTreeMap<String, Value>>,
    },
    ItemStarted {
        turn: Option<TurnId>,
        item: ItemId,
        kind: ItemKind,
        parent: Option<ItemId>,
        #[serde(default)]
        head: Option<ItemHead>,
    },
    ItemCompleted {
        item: ItemId,
        status: ItemStatus,
        body: Item,
    },
    TurnEnded {
        turn: TurnId,
        reason: StopReason,
        usage: Option<Usage>,
    },
    OpRejected {
        op: OpId,
        reason: Rejection,
    },
    SessionClosed,

    ItemDelta {
        item: ItemId,
        delta: Delta,
    },
    ItemSnapshot {
        item: ItemId,
        partial: Item,
    },
    Status {
        phase: Phase,
    },

    Lagged {
        resume_after: Seq,
    },
    Reset {
        epoch: Epoch,
        head: Seq,
    },
    #[serde(untagged)]
    Unknown(Value),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ItemStatus {
    Completed,
    Failed {
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    Cancelled,
    Interrupted,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    Quiet {
        #[serde(default)]
        ms: u64,
    },
    Cancelled,
    MaxTokens,
    MaxRounds,
    Interrupted,
    Error {
        class: ErrorClass,
        message: String,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnOrigin {
    User,
    Agent,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
    Text {
        append: String,
    },
    Reasoning {
        append: String,
    },
    ToolInput {
        append: String,
    },
    Output {
        stream: Stream,
        append: String,
    },
    Plan {
        entries: Vec<PlanEntry>,
    },
    #[serde(untagged)]
    Unknown(Value),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Item {
    UserMessage {
        blocks: Vec<InputBlock>,
    },
    AgentMessage {
        markdown: String,
    },
    Reasoning {
        summary: String,
        raw: Option<String>,
        provider_opaque: Option<Value>,
    },
    ToolCall {
        call_id: String,
        name: String,
        tool_kind: ToolKind,
        input_raw: String,
        output: Vec<OutputBlock>,
        locations: Vec<Location>,
        spill: Option<BlobRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        server: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        raw_output: Option<Value>,
    },
    FileChange {
        path: PathBuf,
        change: ChangeKind,
        unified: String,
    },
    TurnDiff {
        unified: String,
    },
    Terminal {
        terminal: String,
        command: String,
        exit: Option<i32>,
        signal: Option<i32>,
        output: Option<BlobRef>,
        #[serde(default)]
        live: Option<String>,
    },
    Plan {
        entries: Vec<PlanEntry>,
    },
    Approval {
        subject: ApprovalSubject,
        options: Vec<ApprovalDecisionKind>,
        decision: Option<ApprovalDecision>,
        by: Option<Resolver>,
    },
    /// The agent asks the human (ACP form elicitation). `schema` is the
    /// agent's requested JSON Schema, kept as sent; `answer` is the ACP reply
    /// (`{"action":"accept","content":{..}}`, `decline` or `cancel`).
    Question {
        #[serde(default)]
        message: String,
        #[serde(default)]
        schema: Value,
        #[serde(default)]
        answer: Option<Value>,
    },
    Image {
        mime: String,
        blob: BlobRef,
    },
    Compaction {
        summary: BlobRef,
        replaced_through: Seq,
    },
    ContextTrim {
        clipped_through: Seq,
    },
    Usage {
        usage: Usage,
    },
    Notice {
        level: Level,
        text: String,
    },
    Subagent {
        session: SessionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        call: Option<ItemId>,
    },
    Command {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        args: String,
    },
    Hook {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        event: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        outcome: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },
    SystemMessage {
        #[serde(default, skip_serializing_if = "SystemRole::is_unknown")]
        role: SystemRole,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        text: String,
    },
    Native {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        provider: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        record_type: String,
        #[serde(default, skip_serializing_if = "Value::is_null")]
        raw: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blob: Option<BlobRef>,
    },
    MemoryWrite {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
        #[serde(default)]
        op: MemoryOp,
        #[serde(default)]
        record: RecordPin,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<EvidenceRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        draft: Option<Value>,
    },
    MemoryState {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
        #[serde(default)]
        state: EntryState,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        harness: Option<String>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        reason: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        refs: Vec<EvidenceRef>,
    },
    Trial {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        harness: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        thresholds: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        episodes: Vec<TrialEpisode>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        controls: Vec<TrialEpisode>,
        #[serde(default)]
        verdict: Verdict,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        reason: String,
    },
    Adoption {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        harness: String,
        #[serde(default)]
        sink: SinkRef,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        block_digest: Digest,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        record: Option<RecordPin>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt: Option<Digest>,
    },
    Recall {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        entry: EntryId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sink: Option<SinkRef>,
        #[serde(default)]
        basis: RecallBasis,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        record: Option<RecordPin>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        block_digest: Option<Digest>,
    },
    Receipt {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        schema: String,
        // Item owns the `kind` discriminator; the receipt body's kind remains
        // in its blob. d20261103136378 fixes this additive carrier wire key.
        #[serde(default, rename = "receipt_kind")]
        kind: ReceiptKind,
        #[serde(default)]
        outcome: ReceiptOutcome,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        harness: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sink: Option<SinkRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blob: Option<BlobRef>,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemHead {
    ToolCall {
        call_id: String,
        name: String,
        tool_kind: ToolKind,
    },
    Terminal {
        terminal: String,
        command: String,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlobRef {
    pub sha256: String, /* hex */
    pub len: u64,
    pub mime: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    Move,
    Search,
    Execute,
    Think,
    Fetch,
    Other,
    #[serde(untagged)]
    Unknown(Value),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SessionMeta {
    pub session: SessionId,
    pub cwd: PathBuf,
    pub profile: Option<String>,
    pub fork: Option<(SessionId, Seq)>,
    pub protocol: Version,
    pub harness: String,
    pub created_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcs: Option<Vcs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dream: Option<DreamRun>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    UserMessage,
    AgentMessage,
    Reasoning,
    ToolCall,
    FileChange,
    TurnDiff,
    Terminal,
    Plan,
    Approval,
    Question,
    Image,
    Compaction,
    ContextTrim,
    Usage,
    Notice,
    Subagent,
    Command,
    Hook,
    SystemMessage,
    Native,
    MemoryWrite,
    MemoryState,
    Trial,
    Adoption,
    Recall,
    Receipt,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub cost_micros_usd: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<UsageScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<Box<Backend>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_used: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<Box<Cost>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<NumberSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreported: Vec<String>,
}
/// Log IR v1 §4a: counters from calls, turn totals and cumulative sessions must
/// never be summed together. Missing required counters are named in unreported.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageScope {
    Call,
    Turn,
    Session,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Backend {
    pub kind: BackendKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Engine,
    AcpAgent,
    Import,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NumberSource {
    Reported,
    Derived,
    Estimated,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Cost {
    pub amount_micros: u64,
    pub currency: String,
    pub source: NumberSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_source: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputBlock {
    Text {
        text: String,
    },
    Json {
        value: Value,
    },
    Image {
        blob: BlobRef,
    },
    Diff {
        path: PathBuf,
        unified: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        change: Option<ChangeKind>,
    },
    Audio {
        #[serde(default, skip_serializing_if = "BlobRef::is_empty")]
        blob: BlobRef,
    },
    Resource {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        uri: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        mime: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blob: Option<BlobRef>,
    },
    Terminal {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        terminal: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        exit: Option<i32>,
    },
    Citations {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        entries: Vec<Citation>,
    },
    Error {
        text: String,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Location {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub end_line: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ChangeKind {
    Add,
    Modify,
    Delete,
    Rename {
        from: PathBuf,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    Stdout,
    Stderr,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlanEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub text: String,
    pub status: PlanStatus,
    pub priority: Option<Priority>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalSubject {
    pub tool_call: ItemId,
    pub effect: Effect,
    pub summary: String,
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Read,
    Write,
    Exec,
    Network,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecisionKind {
    Allow,
    AllowAlways,
    Deny,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "by", rename_all = "snake_case")]
pub enum Resolver {
    Face {
        name: String,
    },
    Rule {
        id: String,
    },
    Unattended,
    Cancelled,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Info,
    Warn,
    Error,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Queued {
        turns: u32,
    },
    Thinking,
    Streaming,
    Tool {
        item: ItemId,
    },
    Approval {
        item: ItemId,
    },
    Retrying {
        attempt: u32,
        wait_ms: u64,
    },
    Closed,
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Rejection {
    NotIdle,
    UnknownProfile {
        name: String,
    },
    ProfileInvalid {
        detail: String,
    },
    NoSuchTurn,
    NoSuchApproval,
    AlreadyResolved,
    NothingToCompact,
    InvalidInput {
        detail: String,
    },
    Coalesced {
        into: OpId,
    },
    #[serde(untagged)]
    Unknown(Value),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    Transient,
    RateLimited,
    Auth,
    BadRequest,
    Malformed,
    MidStream,
    Cancelled,
    Config,
    Store,
    Tool,
    Internal,
    #[serde(untagged)]
    Unknown(Value),
}
impl Event {
    pub fn is_durable(&self) -> bool {
        matches!(
            self,
            Self::SessionStarted { .. }
                | Self::SessionRecovered { .. }
                | Self::ProfileChanged { .. }
                | Self::ContextChanged { .. }
                | Self::TurnQueued { .. }
                | Self::TurnStarted { .. }
                | Self::ItemStarted { .. }
                | Self::ItemCompleted { .. }
                | Self::TurnEnded { .. }
                | Self::OpRejected { .. }
                | Self::SessionClosed
        )
    }
}
impl Item {
    pub fn kind(&self) -> ItemKind {
        match self {
            Self::UserMessage { .. } => ItemKind::UserMessage,
            Self::AgentMessage { .. } => ItemKind::AgentMessage,
            Self::Reasoning { .. } => ItemKind::Reasoning,
            Self::ToolCall { .. } => ItemKind::ToolCall,
            Self::FileChange { .. } => ItemKind::FileChange,
            Self::TurnDiff { .. } => ItemKind::TurnDiff,
            Self::Terminal { .. } => ItemKind::Terminal,
            Self::Plan { .. } => ItemKind::Plan,
            Self::Approval { .. } => ItemKind::Approval,
            Self::Question { .. } => ItemKind::Question,
            Self::Image { .. } => ItemKind::Image,
            Self::Compaction { .. } => ItemKind::Compaction,
            Self::ContextTrim { .. } => ItemKind::ContextTrim,
            Self::Usage { .. } => ItemKind::Usage,
            Self::Notice { .. } => ItemKind::Notice,
            Self::Subagent { .. } => ItemKind::Subagent,
            Self::Command { .. } => ItemKind::Command,
            Self::Hook { .. } => ItemKind::Hook,
            Self::SystemMessage { .. } => ItemKind::SystemMessage,
            Self::Native { .. } => ItemKind::Native,
            Self::MemoryWrite { .. } => ItemKind::MemoryWrite,
            Self::MemoryState { .. } => ItemKind::MemoryState,
            Self::Trial { .. } => ItemKind::Trial,
            Self::Adoption { .. } => ItemKind::Adoption,
            Self::Recall { .. } => ItemKind::Recall,
            Self::Receipt { .. } => ItemKind::Receipt,
            Self::Unknown(raw) => ItemKind::Unknown(raw.clone()),
        }
    }
    pub fn apply(&mut self, delta: &Delta) {
        match (self, delta) {
            (Self::AgentMessage { markdown }, Delta::Text { append }) => markdown.push_str(append),
            (Self::Reasoning { summary, .. }, Delta::Reasoning { append }) => {
                summary.push_str(append)
            }
            (Self::ToolCall { input_raw, .. }, Delta::ToolInput { append }) => {
                input_raw.push_str(append)
            }
            (Self::ToolCall { output, .. }, Delta::Output { append, .. }) => {
                output.push(OutputBlock::Text {
                    text: append.clone(),
                })
            }
            (Self::Terminal { live, .. }, Delta::Output { append, .. }) => {
                let text = live.get_or_insert_with(String::new);
                text.push_str(append);
                if text.len() > 64 * 1024 {
                    let mut cut = text.len() - 64 * 1024;
                    while !text.is_char_boundary(cut) {
                        cut += 1;
                    }
                    let tail = text[cut..].to_string();
                    *text = format!("…{tail}");
                }
            }
            (Self::Plan { entries }, Delta::Plan { entries: next }) => entries.clone_from(next),
            _ => {}
        }
    }
    pub fn head(&self) -> Option<ItemHead> {
        match self {
            Self::ToolCall {
                call_id,
                name,
                tool_kind,
                ..
            } => Some(ItemHead::ToolCall {
                call_id: call_id.clone(),
                name: name.clone(),
                tool_kind: tool_kind.clone(),
            }),
            Self::Terminal {
                terminal, command, ..
            } => Some(ItemHead::Terminal {
                terminal: terminal.clone(),
                command: command.clone(),
            }),
            _ => None,
        }
    }
    pub fn interrupted_from_head(kind: ItemKind, id: ItemId, head: Option<&ItemHead>) -> Self {
        let mut body = Self::interrupted(kind, id);
        match (&mut body, head) {
            (
                Self::ToolCall {
                    call_id,
                    name,
                    tool_kind,
                    input_raw,
                    ..
                },
                Some(ItemHead::ToolCall {
                    call_id: c,
                    name: n,
                    tool_kind: k,
                }),
            ) => {
                call_id.clone_from(c);
                name.clone_from(n);
                *tool_kind = k.clone();
                input_raw.clear();
            }
            (
                Self::Terminal {
                    terminal, command, ..
                },
                Some(ItemHead::Terminal {
                    terminal: t,
                    command: c,
                }),
            ) => {
                terminal.clone_from(t);
                command.clone_from(c);
            }
            _ => {}
        }
        body
    }
    pub fn interrupted(kind: ItemKind, id: ItemId) -> Self {
        // Durable ItemStarted stores only kind. Recovery must use a synthetic
        // body when the process died before an authoritative body was stored.
        match kind {
            ItemKind::UserMessage => Self::UserMessage { blocks: vec![] },
            ItemKind::AgentMessage => Self::AgentMessage {
                markdown: String::new(),
            },
            ItemKind::Reasoning => Self::Reasoning {
                summary: String::new(),
                raw: None,
                provider_opaque: None,
            },
            ItemKind::ToolCall => Self::ToolCall {
                title: None,
                server: None,
                raw_output: None,
                call_id: id.to_string(),
                name: "interrupted".into(),
                tool_kind: ToolKind::Other,
                input_raw: "{}".into(),
                output: vec![OutputBlock::Error {
                    text: "tool call was not completed".into(),
                }],
                locations: vec![],
                spill: None,
            },
            ItemKind::FileChange => Self::FileChange {
                path: PathBuf::new(),
                change: ChangeKind::Modify,
                unified: String::new(),
            },
            ItemKind::TurnDiff => Self::TurnDiff {
                unified: String::new(),
            },
            ItemKind::Terminal => Self::Terminal {
                terminal: id.to_string(),
                command: String::new(),
                exit: None,
                signal: None,
                output: None,
                live: None,
            },
            ItemKind::Plan => Self::Plan { entries: vec![] },
            ItemKind::Approval => Self::Approval {
                subject: ApprovalSubject {
                    tool_call: id,
                    effect: Effect::Write,
                    summary: "interrupted".into(),
                    paths: vec![],
                    command: None,
                },
                options: vec![],
                decision: Some(ApprovalDecision::Deny {
                    reason: Some("interrupted".into()),
                }),
                by: Some(Resolver::Cancelled),
            },
            ItemKind::Question => Self::Question {
                message: String::new(),
                schema: Value::Null,
                answer: Some(serde_json::json!({"action": "cancel"})),
            },
            ItemKind::Image => Self::Image {
                mime: "application/octet-stream".into(),
                blob: BlobRef::empty(),
            },
            ItemKind::Compaction => Self::Compaction {
                summary: BlobRef::empty(),
                replaced_through: Seq(0),
            },
            ItemKind::ContextTrim => Self::ContextTrim {
                clipped_through: Seq(0),
            },
            ItemKind::Usage => Self::Usage {
                usage: Usage::default().authored(
                    UsageScope::Call,
                    Backend {
                        kind: BackendKind::Engine,
                        name: "harness-recovery".into(),
                        version: Some(env!("CARGO_PKG_VERSION").into()),
                    },
                    None,
                ),
            },
            ItemKind::Notice => Self::Notice {
                level: Level::Warn,
                text: "interrupted".into(),
            },
            ItemKind::Subagent => Self::Subagent {
                name: None,
                call: None,
                session: SessionId(id.0),
            },
            ItemKind::Command => Self::Command {
                name: String::new(),
                args: String::new(),
            },
            ItemKind::Hook => Self::Hook {
                event: String::new(),
                name: String::new(),
                outcome: "interrupted".into(),
                output: None,
            },
            ItemKind::SystemMessage => Self::SystemMessage {
                role: SystemRole::System,
                text: String::new(),
            },
            ItemKind::Native => Self::Native {
                provider: "unknown".into(),
                record_type: "interrupted".into(),
                raw: Value::Null,
                blob: None,
            },
            // Recovery keeps the kind and marks the completion Interrupted.
            // Empty identities cannot be mistaken for a committed memory entry.
            ItemKind::MemoryWrite => Self::MemoryWrite {
                entry: String::new(),
                op: MemoryOp::Pending,
                record: RecordPin::default(),
                evidence: vec![],
                draft: None,
            },
            ItemKind::MemoryState => Self::MemoryState {
                entry: String::new(),
                state: EntryState::Proposed,
                harness: None,
                reason: "interrupted".into(),
                refs: vec![],
            },
            ItemKind::Trial => Self::Trial {
                entry: String::new(),
                harness: String::new(),
                thresholds: String::new(),
                episodes: vec![],
                controls: vec![],
                verdict: Verdict::Inconclusive,
                reason: "interrupted".into(),
            },
            ItemKind::Adoption => Self::Adoption {
                entry: String::new(),
                harness: String::new(),
                sink: SinkRef {
                    kind: SinkKind::Proposal,
                    path: String::new(),
                },
                block_digest: String::new(),
                record: None,
                receipt: None,
            },
            ItemKind::Recall => Self::Recall {
                entry: String::new(),
                sink: None,
                basis: RecallBasis::Inferred,
                record: None,
                block_digest: None,
            },
            ItemKind::Receipt => Self::Receipt {
                schema: "bboxy.dream.receipt/1".into(),
                kind: ReceiptKind::Run,
                outcome: ReceiptOutcome::Failed,
                harness: None,
                sink: None,
                blob: None,
            },
            ItemKind::Unknown(raw) => Self::Notice {
                level: Level::Warn,
                text: format!("Unsupported item {} was interrupted", unknown_tag(&raw)),
            },
        }
    }
}
impl BlobRef {
    pub fn empty() -> Self {
        Self {
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            len: 0,
            mime: "application/octet-stream".into(),
        }
    }
}
/// Schemars is a build-time schema dependency; protocol contains no runtime IO.
pub fn schema() -> schemars::Schema {
    schemars::schema_for!(ProtocolSchema)
}
/// Stable schema output without enabling insertion-ordered maps in the workspace.
pub fn schema_value() -> serde_json::Value {
    let mut value: serde_json::Value = schema().into();
    value.sort_all_objects();
    value
}
#[derive(JsonSchema)]
#[allow(dead_code)]
struct ProtocolSchema {
    envelope: Envelope,
    op: OpEnvelope,
}

impl PartialEq for Envelope {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
            && self.session == other.session
            && self.epoch == other.epoch
            && self.seq == other.seq
            && self.at == other.at
            && self.ts_ms == other.ts_ms
            && self.cause == other.cause
            && self.event == other.event
            && self.ext == other.ext
            && self.native == other.native
    }
}
impl Envelope {
    /// Read without claiming authorship; forwarding uses the retained line.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        let mut envelope: Self = serde_json::from_str(line)?;
        envelope.raw = Some(Arc::from(line));
        Ok(envelope)
    }
    pub fn wire_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        if let Some(line) = &self.raw {
            return Ok(line.as_bytes().to_vec());
        }
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
    /// A fork's parsed projection uses local identity while its bytes retain origin.
    pub fn project_into(&mut self, meta: &SessionMeta) {
        if meta
            .fork
            .is_some_and(|(_, at)| self.seq.is_some_and(|seq| seq > Seq(1) && seq <= at))
        {
            self.session = meta.session;
            self.epoch = Epoch(1);
        }
    }
}
impl Version {
    pub fn is_readable(self) -> bool {
        self.major <= 1 // 0.x is the v1 baseline.
    }
}

macro_rules! unknown_predicate {
    ($($name:ident),* $(,)?) => {$(impl $name {
        pub fn is_unknown(&self)->bool {matches!(self,Self::Unknown(_))}
        pub fn reader_label(&self)->String {match self {Self::Unknown(raw)=>format!("Unknown({})",unknown_tag(raw)),_=>format!("{self:?}")}}
    })*};
}
unknown_predicate!(
    TurnMode,
    ToolKind,
    ItemKind,
    Stream,
    PlanStatus,
    Priority,
    Effect,
    ApprovalDecisionKind,
    Level,
    ErrorClass,
    UsageScope,
    BackendKind,
    NumberSource,
    ApprovalDecision,
    Rejection,
    SystemRole,
    DreamPhase,
    Trigger,
    ScopeLevel,
    EvidenceRef,
    MemoryOp,
    EntryState,
    Arm,
    Verdict,
    SinkKind,
    RecallBasis,
    ReceiptKind,
    ReceiptOutcome
);
/// A neutral reader label keeps the source's own tag, never its opaque payload.
pub fn unknown_tag(value: &Value) -> &str {
    let tag = value
        .as_str()
        .or_else(|| {
            [
                "type", "kind", "status", "reason", "phase", "decision", "op", "by",
            ]
            .into_iter()
            .find_map(|key| value.get(key).and_then(Value::as_str))
        })
        .unwrap_or("unknown");
    let mut end = tag.len().min(128);
    while !tag.is_char_boundary(end) {
        end -= 1;
    }
    &tag[..end]
}

impl StopReason {
    pub fn reader_label(&self) -> String {
        match self {
            Self::Unknown(raw) => format!("Unknown({})", unknown_tag(raw)),
            Self::Error { class, message } => format!(
                "Error {{ class: {}, message: {:?} }}",
                class.reader_label(),
                message
            ),
            _ => format!("{self:?}"),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Native {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts_ms: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Source {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub harness: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser_revision: Option<u32>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Citation {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemRole {
    System,
    Developer,
    #[serde(untagged)]
    Unknown(Value),
}
impl Default for SystemRole {
    fn default() -> Self {
        Self::Unknown(Value::Null)
    }
}
impl Default for BlobRef {
    fn default() -> Self {
        Self::empty()
    }
}
impl BlobRef {
    fn is_empty(&self) -> bool {
        self.len == 0 && self.sha256 == Self::empty().sha256
    }
}

impl InputBlock {
    pub fn display_text(&self) -> String {
        match self {
            Self::Text { text } => text.clone(),
            Self::Image { blob } => format!("Image: {} · {} bytes", blob.mime, blob.len),
            Self::Audio { blob } => format!("Audio: {} · {} bytes", blob.mime, blob.len),
            Self::Resource {
                uri,
                mime,
                blob: Some(blob),
            } if uri.starts_with("bboxy-intake:") => format!("Image: {mime} · {} bytes", blob.len),
            Self::Resource { uri, mime, .. } => format!("Resource: {uri} · {mime}"),
            Self::ResourceLink { uri, title } => {
                format!("{}: {uri}", title.as_deref().unwrap_or("Resource"))
            }
            Self::Mention { path } => format!("@{}", path.display()),
            Self::Unknown(raw) => format!("Unsupported input: {}", unknown_tag(raw)),
        }
    }
}
impl OutputBlock {
    pub fn display_text(&self) -> String {
        match self {
            Self::Text { text } | Self::Error { text } => text.clone(),
            Self::Json { value } => value.to_string(),
            Self::Diff { unified, .. } => unified.clone(),
            Self::Image { blob } => format!("Image: {} · {} bytes", blob.mime, blob.len),
            Self::Audio { blob } => format!("Audio: {} · {} bytes", blob.mime, blob.len),
            Self::Resource { uri, mime, .. } => format!("Resource: {uri} · {mime}"),
            Self::Terminal { terminal, exit } => format!(
                "Terminal: {terminal}{}",
                exit.map(|n| format!(" · exit {n}")).unwrap_or_default()
            ),
            Self::Citations { entries } => entries
                .iter()
                .map(|c| {
                    format!(
                        "{}: {}{}",
                        c.title.as_deref().unwrap_or("Citation"),
                        c.url,
                        c.snippet
                            .as_ref()
                            .map(|s| format!(" — {s}"))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Unknown(raw) => format!("Unsupported tool output: {}", unknown_tag(raw)),
        }
    }
}
impl Item {
    pub fn extension_text(&self) -> Option<String> {
        Some(match self {
            Self::MemoryWrite { .. }
            | Self::MemoryState { .. }
            | Self::Trial { .. }
            | Self::Adoption { .. }
            | Self::Recall { .. } => serde_json::to_string(self).ok()?,
            Self::Receipt { .. } => serde_json::to_string(self).ok()?,
            Self::Command { name, args } => format!("Command: /{name} {args}"),
            Self::Hook {
                event,
                name,
                outcome,
                output,
            } => format!(
                "Hook: {event} · {name} · {outcome}{}",
                output
                    .as_ref()
                    .map(|s| format!("\n{s}"))
                    .unwrap_or_default()
            ),
            Self::SystemMessage { role, text } => format!("{}: {text}", role.reader_label()),
            Self::Native {
                provider,
                record_type,
                raw,
                blob,
            } => format!(
                "Native: {provider}/{record_type} · {} bytes",
                blob.as_ref()
                    .map(|b| b.len)
                    .unwrap_or_else(|| raw.to_string().len() as u64)
            ),
            _ => return None,
        })
    }
}
