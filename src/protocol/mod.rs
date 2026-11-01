// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use ulid::Ulid;
pub mod laws;
pub const PROTOCOL: Version = Version { major: 0, minor: 2 };
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
    SetProfile {
        profile: String,
    },
    Compact,
    Close,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TurnMode {
    Queue,
    Steer,
    #[serde(other)]
    Unknown,
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
    Mention {
        path: PathBuf,
    },
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ApprovalDecision {
    Allow,
    AllowAlways,
    Deny {
        reason: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Envelope {
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
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ItemStatus {
    Completed,
    Failed {
        reason: String,
    },
    Cancelled,
    Interrupted,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    Cancelled,
    MaxTokens,
    MaxRounds,
    Interrupted,
    Error {
        class: ErrorClass,
        message: String,
    },
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
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
    Image {
        mime: String,
        blob: BlobRef,
    },
    Compaction {
        summary: BlobRef,
        replaced_through: Seq,
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
    },
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlobRef {
    pub sha256: String, /* hex */
    pub len: u64,
    pub mime: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    #[serde(other)]
    Unknown,
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
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    Image,
    Compaction,
    Usage,
    Notice,
    Subagent,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub cost_micros_usd: Option<u64>,
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
    },
    Error {
        text: String,
    },
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    Stdout,
    Stderr,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlanEntry {
    pub text: String,
    pub status: PlanStatus,
    pub priority: Option<Priority>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalSubject {
    pub tool_call: ItemId,
    pub effect: Effect,
    pub summary: String,
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Write,
    Exec,
    Network,
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecisionKind {
    Allow,
    AllowAlways,
    Deny,
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Info,
    Warn,
    Error,
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
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
    #[serde(other)]
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    #[serde(other)]
    Unknown,
}
impl Event {
    pub fn is_durable(&self) -> bool {
        matches!(
            self,
            Self::SessionStarted { .. }
                | Self::SessionRecovered { .. }
                | Self::ProfileChanged { .. }
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
            Self::Image { .. } => ItemKind::Image,
            Self::Compaction { .. } => ItemKind::Compaction,
            Self::Usage { .. } => ItemKind::Usage,
            Self::Notice { .. } => ItemKind::Notice,
            Self::Subagent { .. } => ItemKind::Subagent,
            Self::Unknown => ItemKind::Unknown,
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
                tool_kind: *tool_kind,
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
                *tool_kind = *k;
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
            ItemKind::Image => Self::Image {
                mime: "application/octet-stream".into(),
                blob: BlobRef::empty(),
            },
            ItemKind::Compaction => Self::Compaction {
                summary: BlobRef::empty(),
                replaced_through: Seq(0),
            },
            ItemKind::Usage => Self::Usage {
                usage: Usage::default(),
            },
            ItemKind::Notice => Self::Notice {
                level: Level::Warn,
                text: "interrupted".into(),
            },
            ItemKind::Subagent => Self::Subagent {
                session: SessionId(id.0),
            },
            ItemKind::Unknown => Self::Unknown,
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
#[derive(JsonSchema)]
#[allow(dead_code)]
struct ProtocolSchema {
    envelope: Envelope,
    op: OpEnvelope,
}
