// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Prepared-read wire authority. Exact immutable evidence is never reserialized.
pub use super::c1_wire::{Context, Hex, Id, Number, PageRef, Source, Ulid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub const MAX_FRAME: usize = 256 * 1024;
pub const MAX_SAFE: u64 = super::c1_wire::MAX_SAFE;
pub const MAX_INPUT: u64 = 64 * 1024 * 1024;
pub const MAX_SNAPSHOT: usize = 4 * 1024 * 1024;
pub const MAX_PAGE: usize = 16 * 1024;
pub const MAX_ENTRY: usize = 12 * 1024;
fn explicit_nullable<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
macro_rules! bounded_text {
    ($name:ident, $max:expr) => {
        fn $name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
            let text = String::deserialize(d)?;
            if text.len() > $max {
                Err(serde::de::Error::custom("prepared text byte limit"))
            } else {
                Ok(text)
            }
        }
    };
}
bounded_text!(text160, 160);
bounded_text!(text256, 256);
bounded_text!(text2048, 2048);
bounded_text!(text8192, 8192);
macro_rules! bounded_vec {
    ($name:ident,$item:ty,$max:expr) => {
        fn $name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<$item>, D::Error> {
            let values = Vec::<$item>::deserialize(d)?;
            if values.len() > $max {
                Err(serde::de::Error::custom("prepared collection limit"))
            } else {
                Ok(values)
            }
        }
    };
}
bounded_vec!(files2048, BundleFile, 2048);
bounded_vec!(roots32, RootBinding, 32);
bounded_vec!(records512, RecordPin, 512);
bounded_vec!(sources32, Source, 32);
bounded_vec!(nodes16, EvidenceNode, 16);
bounded_vec!(edges32, EvidenceEdge, 32);

macro_rules! scalar {
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
            fn from(v: $name) -> String {
                v.0
            }
        }
    };
}
scalar!(PathText, |s: &str| !s.is_empty()
    && s.len() <= 4096
    && !s.contains('\0'));
scalar!(RelativePath, |s: &str| !s.is_empty()
    && s.len() <= 4096
    && !s.contains(['\0', '\\'])
    && s.split('/').all(|c| !c.is_empty() && c != "." && c != ".."));
scalar!(Commit, |s: &str| super::entry::valid_commit(s));
scalar!(Digest, |s: &str| s
    .strip_prefix("sha256:")
    .is_some_and(|h| Hex::try_from(h.to_owned()).is_ok()));

macro_rules! discriminator {
    ($name:ident, $variant:ident, $value:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        pub enum $name {
            #[serde(rename = $value)]
            $variant,
        }
    };
}
discriminator!(Protocol, V1, "bboxy.dream.read/1");
discriminator!(Algorithm, ReadPrepare, "read-prepare/1");
discriminator!(InputSchema, V1, "bboxy.dream.read-input/1");
discriminator!(SnapshotSchema, V1, "bboxy.dream.read-snapshot/1");
discriminator!(PageSchema, V1, "bboxy.dream.read-page/1");
discriminator!(Kind, Lesson, "lesson");
discriminator!(ProofKind, VerifiedLesson, "verified_lesson");
discriminator!(Currentness, VerifiedAtRevision, "verified_at_revision");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    BadRequest,
    SourceMismatch,
    SourceChanged,
    UnsupportedIr,
    BlobMismatch,
    ProducerUnapproved,
    VerificationUnproven,
    VerificationFailed,
    HarnessIncompatible,
    ResourceLimit,
    StorageUnavailable,
    ProtocolError,
    Cancelled,
    Deadline,
    WorkerLost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Validate,
    Prepare,
    Paused,
    Settling,
}
pub use super::c1_wire::Outcome;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordPin {
    pub root: Id,
    pub id: Id,
    pub path: PathText,
    pub commit: Commit,
    pub digest: Digest,
}
impl TryFrom<crate::protocol::RecordPin> for RecordPin {
    type Error = &'static str;
    fn try_from(p: crate::protocol::RecordPin) -> Result<Self, Self::Error> {
        Ok(Self {
            root: p.root.try_into()?,
            id: p.id.ok_or("record id")?.try_into()?,
            path: p.path.try_into()?,
            commit: p.commit.ok_or("record commit")?.try_into()?,
            digest: p.digest.try_into()?,
        })
    }
}
impl From<RecordPin> for crate::protocol::RecordPin {
    fn from(p: RecordPin) -> Self {
        Self {
            root: p.root.0,
            id: Some(p.id.0),
            path: p.path.0,
            commit: Some(p.commit.0),
            digest: p.digest.0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventPin {
    pub session: Ulid,
    pub seq: Number,
    pub digest: Digest,
}
impl TryFrom<crate::protocol::EvidenceRef> for EventPin {
    type Error = &'static str;
    fn try_from(p: crate::protocol::EvidenceRef) -> Result<Self, Self::Error> {
        match p {
            crate::protocol::EvidenceRef::Event {
                session,
                seq,
                digest,
            } => Ok(Self {
                session: session.0.to_string().try_into()?,
                seq: seq.0.try_into()?,
                digest: digest.try_into()?,
            }),
            _ => Err("event pin"),
        }
    }
}
impl From<EventPin> for crate::protocol::EvidenceRef {
    fn from(p: EventPin) -> Self {
        Self::Event {
            session: crate::protocol::SessionId(p.session.0.parse().expect("validated ULID")),
            seq: crate::protocol::Seq(p.seq.0),
            digest: p.digest.0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pin {
    Event(EventPin),
    Record(RecordPin),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BundleFile {
    pub slot: u32,
    pub path: RelativePath,
    pub len: Number,
    pub sha256: Hex,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RootBinding {
    pub root: Id,
    pub repo: Id,
    pub main: Commit,
    pub dir: RelativePath,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KeyBinding {
    pub fingerprint: Id,
    pub file: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactBinding {
    pub logical: PathText,
    pub file: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBundle {
    #[serde(deserialize_with = "files2048")]
    pub files: Vec<BundleFile>,
    #[serde(deserialize_with = "roots32")]
    pub roots: Vec<RootBinding>,
    pub policy: u32,
    pub public_keys: Vec<KeyBinding>,
    pub artifacts: Vec<ArtifactBinding>,
    #[serde(deserialize_with = "records512")]
    pub catalogue: Vec<RecordPin>,
    pub selected_at_ms: Number,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub schema: InputSchema,
    pub nonce: Ulid,
    pub context: Context,
    #[serde(deserialize_with = "sources32")]
    pub closed_sources: Vec<Source>,
    #[serde(deserialize_with = "records512")]
    pub records: Vec<RecordPin>,
    pub publications: Vec<EventPin>,
    pub trust_policy: Hex,
    pub provider_revision: Id,
    pub evidence: EvidenceBundle,
}
pub type Manifest = Input;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub input_sha256: Hex,
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
pub struct Cancel {
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
pub struct PrepareError {
    pub code: Code,
    #[serde(deserialize_with = "explicit_nullable")]
    pub source: Option<u32>,
}
pub type Error = PrepareError;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Done {
    pub protocol: Protocol,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub seq: Number,
    pub outcome: Outcome,
    #[serde(deserialize_with = "explicit_nullable")]
    pub result_sha256: Option<Hex>,
    pub pages: u32,
    #[serde(deserialize_with = "explicit_nullable")]
    pub error: Option<PrepareError>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerIn {
    Start(Start),
    Activity(Activity),
    Cancel(Cancel),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerOut {
    Hello(Hello),
    Progress(Progress),
    Done(Done),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub kinds: Vec<Kind>,
    pub indexed: u32,
    pub omitted: u32,
    pub complete: bool,
    pub currentness: Currentness,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LessonProof {
    pub kind: ProofKind,
    pub entry: Id,
    pub revival: u32,
    pub record: RecordPin,
    pub trial: EventPin,
    pub publication: EventPin,
    pub outer: EventPin,
    pub projection: Hex,
    pub trust_policy: Hex,
    pub producer: Hex,
    pub currentness: Hex,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hit {
    pub handle: Id,
    pub id: Id,
    pub kind: Kind,
    #[serde(deserialize_with = "text160")]
    pub title: String,
    #[serde(deserialize_with = "text256")]
    pub preview: String,
    pub repo: Id,
    pub harness: Id,
    pub proof: LessonProof,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceNode {
    pub id: Id,
    pub kind: Id,
    #[serde(deserialize_with = "explicit_nullable")]
    pub pin: Option<Pin>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceEdge {
    pub from: Id,
    pub to: Id,
    pub relation: Id,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedEntry {
    pub hit: Hit,
    #[serde(deserialize_with = "text2048")]
    pub why: String,
    #[serde(deserialize_with = "text2048")]
    pub how_to_apply: String,
    #[serde(deserialize_with = "text8192")]
    pub context: String,
    #[serde(deserialize_with = "text256")]
    pub citation: String,
    #[serde(deserialize_with = "nodes16")]
    pub nodes: Vec<EvidenceNode>,
    #[serde(deserialize_with = "edges32")]
    pub edges: Vec<EvidenceEdge>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub schema: PageSchema,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub index: u32,
    pub entries: Vec<PreparedEntry>,
    #[serde(deserialize_with = "explicit_nullable")]
    pub next: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: SnapshotSchema,
    pub run_id: Ulid,
    pub nonce: Ulid,
    pub context: Context,
    pub input_digest: Hex,
    pub source_revision: Hex,
    pub producer: Hex,
    pub trust_policy: Hex,
    pub entry_ids: Vec<Id>,
    pub coverage: Coverage,
    pub pages: Vec<PageRef>,
}
pub type ResultDescriptor = Snapshot;

pub fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    super::c1_wire::canonical(value)
}
pub fn canonical_limited<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, Code> {
    super::c1_wire::canonical_limited(value, limit).map_err(|_| Code::ResourceLimit)
}
pub fn hash(bytes: impl AsRef<[u8]>) -> Hex {
    super::c1_wire::hash(bytes)
}
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Code> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(Code::ResourceLimit);
    }
    serde_json::from_slice(bytes).map_err(|_| Code::ProtocolError)
}
pub fn source_revision(input: &Input) -> Result<Hex, serde_json::Error> {
    #[derive(Serialize)]
    struct Revision<'a> {
        context: &'a Context,
        provider_revision: &'a Id,
        closed_sources: &'a [Source],
        evidence: &'a EvidenceBundle,
        trust_policy: &'a Hex,
    }
    canonical(&Revision {
        context: &input.context,
        provider_revision: &input.provider_revision,
        closed_sources: &input.closed_sources,
        evidence: &input.evidence,
        trust_policy: &input.trust_policy,
    })
    .map(hash)
}
pub fn validate_entry(e: &PreparedEntry) -> Result<(), Code> {
    if e.hit.title.len() > 160
        || e.hit.preview.len() > 256
        || e.why.len() > 2048
        || e.how_to_apply.len() > 2048
        || e.context.len() > 8192
        || e.citation.len() > 256
        || e.nodes.len() > 16
        || e.edges.len() > 32
    {
        return Err(Code::ResourceLimit);
    }
    if e.hit.id != e.hit.proof.entry {
        return Err(Code::VerificationFailed);
    }
    canonical_limited(e, MAX_ENTRY).map(|_| ())
}

#[derive(JsonSchema)]
#[allow(dead_code)]
struct Authority {
    input: Input,
    worker_in: WorkerIn,
    worker_out: WorkerOut,
    snapshot: Snapshot,
    page: Page,
}
pub fn schema() -> Value {
    let mut s = serde_json::to_value(schemars::schema_for!(Authority)).expect("read schema");
    let defs = s["$defs"].as_object_mut().expect("read definitions");
    defs.insert(
        "Number".into(),
        serde_json::json!({"type":"integer","minimum":0,"maximum":MAX_SAFE}),
    );
    for (name, pattern, max) in [
        ("Ulid", "^[0-7][0-9A-HJKMNP-TV-Z]{25}$", 26),
        ("Hex", "^[0-9a-f]{64}$", 64),
        ("Id", "^[^\u{0000}]+$", 256),
        ("Repo", "^[^\u{0000}]+$", 4096),
        ("Mime", "^[^\u{0000}]*$", 256),
        ("Digest", "^sha256:[0-9a-f]{64}$", 71),
        ("Commit", "^([0-9a-fA-F]{40}|[0-9a-fA-F]{64})$", 64),
        ("PathText", "^[^\u{0000}]+$", 4096),
        (
            "RelativePath",
            "^(?!.*(?:^|/)(?:\\.|\\.\\.)(?:/|$))[^/\\\\\u{0000}]+(?:/[^/\\\\\u{0000}]+)*$",
            4096,
        ),
    ] {
        defs.insert(
            name.into(),
            serde_json::json!({"type":"string","pattern":pattern,"maxLength":max}),
        );
    }
    for (name, field, cap) in [
        ("Input", "closed_sources", 32),
        ("Input", "records", 512),
        ("EvidenceBundle", "files", 2048),
        ("EvidenceBundle", "roots", 32),
        ("EvidenceBundle", "catalogue", 512),
        ("PreparedEntry", "nodes", 16),
        ("PreparedEntry", "edges", 32),
    ] {
        defs[name]["properties"][field]["maxItems"] = cap.into();
    }
    for (name, field, cap) in [
        ("Hit", "title", 160),
        ("Hit", "preview", 256),
        ("PreparedEntry", "why", 2048),
        ("PreparedEntry", "how_to_apply", 2048),
        ("PreparedEntry", "context", 8192),
        ("PreparedEntry", "citation", 256),
    ] {
        defs[name]["properties"][field]["maxLength"] = cap.into();
    }
    fn limits(v: &mut Value) {
        match v {
            Value::Object(m) => {
                let nullable = ["source", "result_sha256", "error", "pin", "next"]
                    .into_iter()
                    .filter(|field| {
                        m.get("properties")
                            .and_then(Value::as_object)
                            .is_some_and(|properties| properties.contains_key(*field))
                    })
                    .collect::<Vec<_>>();
                if !nullable.is_empty() {
                    let required = m
                        .entry("required")
                        .or_insert_with(|| Value::Array(vec![]))
                        .as_array_mut()
                        .expect("closed object required");
                    for field in nullable {
                        if !required.iter().any(|v| v.as_str() == Some(field)) {
                            required.push(field.into());
                        }
                    }
                }
                if m.get("format").and_then(Value::as_str) == Some("uint32") {
                    m.insert("maximum".into(), u32::MAX.into());
                }
                for v in m.values_mut() {
                    limits(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    limits(v);
                }
            }
            _ => {}
        }
    }
    limits(&mut s);
    s
}
pub fn schema_bytes() -> Vec<u8> {
    canonical(&schema()).expect("read schema bytes")
}
pub fn schema_hash() -> Hex {
    hash(include_bytes!("../../schema/dream-read-v1.schema.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn fixture_root() -> &'static Path {
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/dream-read"
        ))
    }
    fn dto(name: &str, bytes: &[u8]) -> Result<Vec<u8>, Code> {
        macro_rules! typed {
            ($t:ty) => {
                canonical(&decode::<$t>(bytes)?).map_err(|_| Code::ProtocolError)
            };
        }
        match name {
            "input" => typed!(Input),
            "snapshot" => typed!(Snapshot),
            "page" => typed!(Page),
            "start" | "activity" | "cancel" => typed!(WorkerIn),
            "hello" | "progress" | "done" | "unproven" => typed!(WorkerOut),
            _ => panic!("unknown read fixture {name}"),
        }
    }
    #[test]
    fn schema_and_canonical_corpus() {
        let schema = schema_bytes();
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/schema/dream-read-v1.schema.json"
        ));
        if std::env::var_os("DREAM_READ_GENERATE").is_some() {
            std::fs::write(path, &schema).unwrap();
            return;
        }
        assert_eq!(std::fs::read(path).unwrap(), schema);
        let root = fixture_root();
        let lock: Value =
            serde_json::from_slice(&std::fs::read(root.join("lock.json")).unwrap()).unwrap();
        assert_eq!(hash(&schema).0, lock["schema_sha256"]);
        for (path, digest) in lock["files"].as_object().unwrap() {
            assert_eq!(
                hash(std::fs::read(root.join(path)).unwrap()).0,
                digest.as_str().unwrap()
            );
        }
        let vectors: Vec<Value> =
            serde_json::from_slice(&std::fs::read(root.join("vectors/canonical.json")).unwrap())
                .unwrap();
        for v in vectors {
            let b = canonical(&v["value"]).unwrap();
            assert_eq!(b, v["canonical"].as_str().unwrap().as_bytes());
            assert_eq!(hash(&b).0, v["sha256"]);
            assert_eq!(canonical_limited(&v["value"], b.len()).unwrap(), b);
            assert_eq!(
                canonical_limited(&v["value"], b.len() - 1),
                Err(Code::ResourceLimit)
            );
        }
        for entry in std::fs::read_dir(root.join("valid")).unwrap() {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(
                dto(path.file_stem().unwrap().to_str().unwrap(), &bytes).unwrap(),
                bytes
            );
        }
        let invalid: Vec<Value> =
            serde_json::from_slice(&std::fs::read(root.join("invalid/cases.json")).unwrap())
                .unwrap();
        for c in invalid {
            assert!(
                dto(
                    c["dto"].as_str().unwrap(),
                    c["bytes"].as_str().unwrap().as_bytes()
                )
                .is_err(),
                "{c}"
            );
        }
    }
    #[test]
    fn independent_python_hash_vectors() {
        let result = std::process::Command::new("python3").args(["-I", "-c", r#"
import hashlib,json,pathlib,sys
p=pathlib.Path(sys.argv[1])
vectors=json.loads((p/'vectors/canonical.json').read_bytes())
for v in vectors:
    b=json.dumps(v['value'],ensure_ascii=False,sort_keys=True,separators=(',',':'),allow_nan=False).encode()
    assert b == v['canonical'].encode()
    assert hashlib.sha256(b).hexdigest() == v['sha256']
print('independent prepared-read canonical vectors:',len(vectors))
"#]).arg(fixture_root()).output().unwrap();
        assert!(result.status.success(), "{:?}", result);
    }
    #[test]
    fn selected_revision_and_entry_limits_are_exact() {
        let input: Input = decode(include_bytes!(
            "../../tests/fixtures/dream-read/valid/input.json"
        ))
        .unwrap();
        let revision = source_revision(&input).unwrap();
        let vector: Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/dream-read/source-revision.json"
        ))
        .unwrap();
        assert_eq!(revision.0, vector["sha256"]);
        assert_eq!(canonical(&serde_json::json!({"context":input.context,"provider_revision":input.provider_revision,
            "closed_sources":input.closed_sources,"evidence":input.evidence,"trust_policy":input.trust_policy})).unwrap(),
            vector["canonical"].as_str().unwrap().as_bytes());
        let page: Page = decode(include_bytes!(
            "../../tests/fixtures/dream-read/valid/page.json"
        ))
        .unwrap();
        let mut e = page.entries[0].clone();
        validate_entry(&e).unwrap();
        e.why = "x".repeat(2049);
        assert_eq!(validate_entry(&e), Err(Code::ResourceLimit));
    }
}
