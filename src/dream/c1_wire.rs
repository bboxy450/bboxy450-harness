// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Sole Dream C1 transport authority. Hashes cover canonical bytes without LF.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const MAX_FRAME: usize = 256 * 1024;
pub const MAX_SAFE: u64 = (1u64 << 53) - 1;

macro_rules! text_scalar {
    ($name:ident, $check:expr) => {
        #[derive(
            Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(pub String);
        impl TryFrom<String> for $name {
            type Error = &'static str;
            fn try_from(v: String) -> Result<Self, Self::Error> {
                if ($check)(&v) {
                    Ok(Self(v))
                } else {
                    Err(stringify!($name))
                }
            }
        }
        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}
text_scalar!(Ulid, |s: &str| s.len() == 26
    && s.parse::<ulid::Ulid>().is_ok_and(|id| id.to_string() == s));
text_scalar!(Hex, |s: &str| s.len() == 64
    && s.bytes()
        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
text_scalar!(Id, |s: &str| !s.is_empty()
    && s.len() <= 256
    && !s.contains('\0'));
text_scalar!(Repo, |s: &str| !s.is_empty()
    && s.len() <= 4096
    && !s.contains('\0'));
text_scalar!(Mime, |s: &str| s.len() <= 256 && !s.contains('\0'));
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "u64", into = "u64")]
pub struct Number(pub u64);
impl TryFrom<u64> for Number {
    type Error = &'static str;
    fn try_from(n: u64) -> Result<Self, Self::Error> {
        if n <= MAX_SAFE {
            Ok(Self(n))
        } else {
            Err("unsafe integer")
        }
    }
}
impl From<Number> for u64 {
    fn from(n: Number) -> u64 {
        n.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "u64", into = "u64")]
pub struct Version;
impl TryFrom<u64> for Version {
    type Error = &'static str;
    fn try_from(n: u64) -> Result<Self, Self::Error> {
        if n == 1 { Ok(Self) } else { Err("version") }
    }
}
impl From<Version> for u64 {
    fn from(_: Version) -> u64 {
        1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[serde(rename = "bboxy.dream.c1/1")]
    V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    #[serde(rename = "d0-c1/1")]
    D0C1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManifestSchema {
    #[serde(rename = "bboxy.dream.c1.manifest/1")]
    V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultSchema {
    #[serde(rename = "bboxy.dream.c1.result/1")]
    V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PageSchema {
    #[serde(rename = "bboxy.dream.c1.page/1")]
    V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    Manual,
    SessionClosed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Deferred,
    Preparing,
    Running,
    Paused,
    Cancelling,
    Settling,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    Quarantined,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    BadRequest,
    VersionMismatch,
    NotFound,
    StaleContext,
    OpIdReused,
    Busy,
    RunnerUnavailable,
    SourceUnavailable,
    SourceOpen,
    SourceChanged,
    MalformedIr,
    UnsupportedIr,
    SourceMismatch,
    BlobMismatch,
    ResourceLimit,
    StorageUnavailable,
    WorkerLost,
    ProtocolError,
    Cancelled,
    Deadline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Validate,
    Replay,
    Publish,
    Paused,
    Settling,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Completed,
    Failed,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Knowledge {
    NotRequested,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    NotEvaluated,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub account: Id,
    pub account_epoch: Number,
    pub seat: Ulid,
    pub generation: Number,
    pub grant_epoch: Number,
    pub swarm: Ulid,
    pub swarm_revision: Number,
    pub workspace: Id,
    pub workspace_revision: Number,
    pub root: Id,
    pub root_revision: Number,
    pub provider: Id,
    pub provider_epoch: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub session: Ulid,
    pub source_generation: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Error {
    pub code: Code,
    pub source: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunView {
    pub run_id: Ulid,
    pub revision: Number,
    pub state: State,
    pub settled: bool,
    pub request_sha256: Hex,
    pub manifest_sha256: Option<Hex>,
    pub result_sha256: Option<Hex>,
    pub pages: u32,
    pub error: Option<Error>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Blob {
    pub sha256: Hex,
    pub len: Number,
    pub mime: Mime,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub session: Ulid,
    pub source_generation: Number,
    pub repo: Repo,
    pub len: Number,
    pub sha256: Hex,
    pub closed_seq: Number,
    pub closed_line_sha256: Hex,
    pub blobs: Vec<Blob>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: ManifestSchema,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub context: Context,
    pub trigger: Trigger,
    pub algorithm: Algorithm,
    pub sources: Vec<Source>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub manifest_sha256: Hex,
    pub timeout_ms: u32,
    pub activity_revision: Number,
    pub foreground: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub revision: Number,
    pub foreground: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkerCancel {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub protocol: Protocol,
    pub algorithm: Algorithm,
    pub schema_sha256: Hex,
    pub ir_major: u32,
    pub ir_minor_max: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub seq: Number,
    pub phase: Phase,
    pub bytes: Number,
    pub envelopes: Number,
    pub cpu_ns: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Done {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub seq: Number,
    pub outcome: Outcome,
    pub result_sha256: Option<Hex>,
    pub pages: u32,
    pub error: Option<Error>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageRef {
    pub index: u32,
    pub len: Number,
    pub sha256: Hex,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultDescriptor {
    pub schema: ResultSchema,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub manifest_sha256: Hex,
    pub algorithm: Algorithm,
    pub source_count: u32,
    pub episode_count: u32,
    pub cluster_count: u32,
    pub max_poll_cpu_ns: Number,
    pub knowledge: Knowledge,
    pub trust: Trust,
    pub pages: Vec<PageRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub schema: PageSchema,
    pub run_id: Ulid,
    pub manifest_sha256: Hex,
    pub index: u32,
    pub rows: Vec<Row>,
    pub next: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub session: Ulid,
    pub seq: Number,
    pub digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub session: Ulid,
    pub turn: Ulid,
    pub origin: Origin,
    pub fingerprints: Vec<Hex>,
    pub first_ms: Number,
    pub last_ms: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cluster {
    pub repo: Repo,
    pub fingerprint: Hex,
    pub count: u32,
    pub first_ms: Number,
    pub last_ms: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub v: Version,
    pub op_id: Ulid,
    pub sources: Vec<Selection>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub v: Version,
    pub run_id: Ulid,
    pub page: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cancel {
    pub v: Version,
    pub op_id: Ulid,
    pub run_id: Ulid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerIn {
    Start(Start),
    Activity(Activity),
    Cancel(WorkerCancel),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerOut {
    Hello(Hello),
    Progress(Progress),
    Done(Done),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Row {
    Episode(Episode),
    Cluster(Cluster),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Control {
    Inspect(Inspect),
    Status(Status),
    Cancel(Cancel),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ControlReply {
    Success(Box<Success>),
    Failure(Failure),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Success {
    pub v: Version,
    pub ok: True,
    pub run: RunView,
    pub page: Option<Page>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub v: Version,
    pub ok: False,
    pub error: Error,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "bool", into = "bool")]
pub struct True;
impl TryFrom<bool> for True {
    type Error = &'static str;
    fn try_from(b: bool) -> Result<Self, Self::Error> {
        if b {
            Ok(Self)
        } else {
            Err("reply discriminator")
        }
    }
}
impl From<True> for bool {
    fn from(_: True) -> bool {
        true
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "bool", into = "bool")]
pub struct False;
impl TryFrom<bool> for False {
    type Error = &'static str;
    fn try_from(b: bool) -> Result<Self, Self::Error> {
        if !b {
            Ok(Self)
        } else {
            Err("reply discriminator")
        }
    }
}
impl From<False> for bool {
    fn from(_: False) -> bool {
        false
    }
}
#[derive(JsonSchema)]
#[allow(dead_code)]
struct Authority {
    control: Control,
    reply: ControlReply,
    manifest: Manifest,
    worker_in: WorkerIn,
    worker_out: WorkerOut,
    result: ResultDescriptor,
    page: Page,
}

pub fn canonical<T: Serialize>(value: &T) -> std::result::Result<Vec<u8>, serde_json::Error> {
    // ACP can transitively enable serde_json/preserve_order. Sort recursively
    // explicitly so feature unification never changes canonical bytes.
    fn sorted(v: Value) -> Value {
        match v {
            Value::Object(map) => {
                let mut pairs: Vec<_> = map.into_iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(&b.0));
                Value::Object(pairs.into_iter().map(|(k, v)| (k, sorted(v))).collect())
            }
            Value::Array(values) => Value::Array(values.into_iter().map(sorted).collect()),
            other => other,
        }
    }
    serde_json::to_vec(&sorted(serde_json::to_value(value)?))
}
pub fn hash(bytes: impl AsRef<[u8]>) -> Hex {
    Hex(super::fingerprint::digest(bytes))
}
pub fn schema() -> Value {
    let mut s = serde_json::to_value(schemars::schema_for!(Authority)).expect("schema");
    let defs = s["$defs"].as_object_mut().expect("definitions");
    for (name, min, max) in [("Number", 0, MAX_SAFE), ("Version", 1, 1)] {
        defs.insert(
            name.into(),
            serde_json::json!({"type":"integer","minimum":min,"maximum":max}),
        );
    }
    for (name, pattern, max) in [
        ("Ulid", "^[0-7][0-9A-HJKMNP-TV-Z]{25}$", 26),
        ("Hex", "^[0-9a-f]{64}$", 64),
        ("Id", "^[^\u{0000}]+$", 256),
        ("Repo", "^[^\u{0000}]+$", 4096),
        ("Mime", "^[^\u{0000}]*$", 256),
    ] {
        defs.insert(
            name.into(),
            serde_json::json!({"type":"string","pattern":pattern,"maxLength":max}),
        );
    }
    defs.insert("True".into(), serde_json::json!({"const":true}));
    defs.insert("False".into(), serde_json::json!({"const":false}));
    fn limits(v: &mut Value) {
        match v {
            Value::Object(m) => {
                if m.get("format").and_then(Value::as_str) == Some("uint32") {
                    m.insert("maximum".into(), u32::MAX.into());
                }
                for x in m.values_mut() {
                    limits(x)
                }
            }
            Value::Array(a) => {
                for x in a {
                    limits(x)
                }
            }
            _ => {}
        }
    }
    limits(&mut s);
    s
}
pub fn schema_bytes() -> Vec<u8> {
    canonical(&schema()).expect("schema bytes")
}
pub fn schema_hash() -> Hex {
    hash(schema_bytes())
}
/// Strict bounded framing; typed serde visitors reject duplicate/unknown fields.
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> std::result::Result<T, Code> {
    if bytes.len() > MAX_FRAME || bytes.is_empty() {
        return Err(Code::ResourceLimit);
    }
    serde_json::from_slice(bytes).map_err(|_| Code::ProtocolError)
}
