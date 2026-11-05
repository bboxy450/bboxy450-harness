// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Exact byte custody and explicit mounted-root boundaries; no filesystem root discovery.
use super::entry::{self, Invalid, Record};
use crate::protocol::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, hash_map::RandomState},
    hash::BuildHasher,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub struct Root {
    pub name: String,
    pub repo: String,
    pub path: PathBuf,
    pub main: String,
    pub cross_repo: bool,
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path != ".bboxy"
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && !path.contains(['\0', '\\'])
}
/// MCP RecordPins are KB-relative; D0's git-object pins are repository-relative.
/// Normalize only for lookup/permission checks, never rewrite their signed/identity spelling.
pub fn repository_path(path: &str) -> Result<String, Invalid> {
    if !safe_path(path) {
        return Err(Invalid("record path escapes mounted KB"));
    }
    Ok(if path.starts_with(".bboxy/") {
        path.into()
    } else {
        format!(".bboxy/{path}")
    })
}

// Adapted from bboxy450-harness WP2/WP3 owned Git capture isolation in src/vcs.rs
// and src/dream/join.rs. Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
async fn git(
    path: &Path,
    args: &[&str],
    deadline: tokio::time::Instant,
) -> Result<(i32, Vec<u8>), Invalid> {
    let timeout = deadline
        .checked_duration_since(tokio::time::Instant::now())
        .ok_or(Invalid("audit git deadline"))?;
    let mut command = Vec::new();
    for name in std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with("GIT_"))
    {
        command.extend(["-u".into(), name]);
    }
    command.extend(
        [
            "GIT_CONFIG_NOSYSTEM=1",
            "GIT_CONFIG_GLOBAL=/dev/null",
            "GIT_TERMINAL_PROMPT=0",
            "GIT_ASKPASS=/usr/bin/false",
            "SSH_ASKPASS=/usr/bin/false",
            "git",
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]
        .map(str::to_owned),
    );
    command.extend(args.iter().map(|a| (*a).to_owned()));
    // The existing KB provider's MAX_DOCUMENT is 2MiB. Captured bytes never become diagnostics.
    let cap = 2 * 1024 * 1024;
    let output = Box::pin(crate::process::capture_stdout(
        crate::process::Request {
            program: "/usr/bin/env".into(),
            args: command,
            stdin: None,
            cwd: path.into(),
            timeout,
            spill_dir: Default::default(),
            cap,
            redactor: Default::default(),
        },
        tokio_util::sync::CancellationToken::new(),
    ))
    .await
    .map_err(|_| Invalid("audit git unavailable"))?;
    if output.cancelled || output.bytes.len() >= cap {
        return Err(Invalid("audit git deadline/output bound"));
    }
    Ok((
        output.code.ok_or(Invalid("audit git did not exit"))?,
        output.bytes,
    ))
}
fn deadline() -> tokio::time::Instant {
    tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET
}
impl Root {
    /// Only call for an explicitly authorized mounted root. A name is never interpreted as a path.
    pub async fn open(name: String, path: PathBuf) -> Result<Self, Invalid> {
        if name.is_empty() || name.contains(['\0', '\n']) {
            return Err(Invalid("invalid mounted root name"));
        }
        let repo = crate::vcs::repo_key(&path)
            .await
            .ok_or(Invalid("mounted root repository unavailable"))?;
        let until = deadline();
        let (code, bytes) = git(&path, &["rev-parse", "--show-toplevel"], until).await?;
        if code != 0 {
            return Err(Invalid("mounted root repository unavailable"));
        }
        let path = PathBuf::from(
            std::str::from_utf8(&bytes)
                .map_err(|_| Invalid("invalid git root"))?
                .trim_end_matches(['\r', '\n']),
        );
        let (code, bytes) = git(&path, &["rev-parse", "--verify", "origin/main"], until).await?;
        let main = std::str::from_utf8(&bytes)
            .map_err(|_| Invalid("invalid main pin"))?
            .trim()
            .to_owned();
        if code != 0 || !entry::valid_commit(&main) {
            return Err(Invalid("mounted origin/main unavailable"));
        }
        let object = format!("{main}:.bboxy/dream.toml");
        let (code, bytes) = git(&path, &["show", &object], until).await?;
        let cross_repo = code == 0 && std::str::from_utf8(&bytes).is_ok_and(cross_repo_opt_in);
        Ok(Self {
            name,
            repo,
            path,
            main,
            cross_repo,
        })
    }
    pub async fn paths(&self, prefix: &str) -> Result<Vec<String>, Invalid> {
        self.paths_at(prefix, &self.main).await
    }
    pub async fn paths_at(&self, prefix: &str, commit: &str) -> Result<Vec<String>, Invalid> {
        let normalized = repository_path(prefix)?;
        let prefix = normalized.as_str();
        if !entry::valid_commit(commit) {
            return Err(Invalid("invalid catalogue commit"));
        }
        let until = deadline();
        let (code, kind) = git(&self.path, &["cat-file", "-t", commit], until).await?;
        if code != 0 || kind != b"commit\n" {
            return Err(Invalid("record pin is not a commit object"));
        }
        let (member, _) = git(
            &self.path,
            &["merge-base", "--is-ancestor", commit, &self.main],
            until,
        )
        .await?;
        if member != 0 {
            return Err(Invalid("catalogue commit is outside mounted origin/main"));
        }
        let (code, bytes) = git(
            &self.path,
            &["ls-tree", "-r", "-z", "--name-only", commit, "--", prefix],
            until,
        )
        .await?;
        if code != 0 {
            return Err(Invalid("committed record catalogue unavailable"));
        }
        crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            bytes
                .split(|b| *b == 0)
                .filter(|b| !b.is_empty())
                .map(|b| {
                    std::str::from_utf8(b)
                        .map(str::to_owned)
                        .map_err(|_| Invalid("non-UTF8 record path"))
                })
                .collect()
        })
        .await
    }
    pub async fn read(&self, path: &str, commit: &str) -> Result<Vec<u8>, Invalid> {
        if !entry::valid_commit(commit) {
            return Err(Invalid("invalid committed record pin"));
        }
        let normalized = repository_path(path)?;
        let path = normalized.as_str();
        let until = deadline();
        let (code, kind) = git(&self.path, &["cat-file", "-t", commit], until).await?;
        if code != 0 || kind != b"commit\n" {
            return Err(Invalid("record pin is not a commit object"));
        }
        let (member, _) = git(
            &self.path,
            &["merge-base", "--is-ancestor", commit, &self.main],
            until,
        )
        .await?;
        if member != 0 {
            return Err(Invalid("record commit is not on mounted origin/main"));
        }
        let (code, mode) = git(&self.path, &["ls-tree", "-z", commit, "--", path], until).await?;
        let regular = mode.split(|b| *b == 0).any(|line| {
            line.iter().position(|b| *b == b'\t').is_some_and(|at| {
                let (head, name) = (&line[..at], &line[at + 1..]);
                name == path.as_bytes()
                    && (head.starts_with(b"100644 blob ") || head.starts_with(b"100755 blob "))
            })
        });
        if code != 0 || !regular {
            return Err(Invalid("record is not a committed regular file"));
        }
        let object = format!("{commit}:{path}");
        let (code, bytes) = git(&self.path, &["show", &object], until).await?;
        if code != 0 {
            return Err(Invalid("committed record unavailable"));
        }
        Ok(bytes)
    }
}
fn cross_repo_opt_in(text: &str) -> bool {
    let mut seen = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            break;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "cross_repo"
        {
            if seen.is_some() {
                return false;
            }
            seen = Some(value.trim() == "true");
        }
    }
    seen == Some(true)
}

/// Match the actual KB writer's DSSE payload, retaining YAML key order.
/// Adapted from bboxy450-term b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8,
/// crates/bbt-mux/src/index.rs:1044-1073 and provenance/author.rs.
/// Upstream policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
/// This checks bytes only. A self-signature never establishes signer authority.
pub fn signature_bytes(record: &Record) -> bool {
    let Some(id) = record.front.get("id").and_then(Value::as_str) else {
        return false;
    };
    super::trust::verify_bytes(&record.bytes, id, &["seat".into()])
        .is_ok_and(|v| v.front == record.front && v.body == record.body && v.yaml == record.yaml)
}

#[derive(Clone)]
pub struct ResolvedRecord {
    pub pin: RecordPin,
    pub record: Record,
    pub trusted: bool,
    pub signature_valid: bool,
}
// Safe API use of hashbrown (MIT OR Apache-2.0), https://github.com/rust-lang/hashbrown.
// Store only the Arc: the immutable record already owns the complete pin.
// No cache iteration escapes into canonical output. Production uses RandomState;
// the private generic only permits collision/rehash tests without weakening it.
#[derive(Default)]
struct RecordCache<S = RandomState> {
    table: hashbrown::HashTable<Arc<ResolvedRecord>>,
    hash_builder: S,
}
fn record_hash<S: BuildHasher>(builder: &S, pin: &RecordPin) -> u64 {
    // Hash's tuple/string/Option encodings preserve field boundaries and None/Some.
    builder.hash_one((&pin.root, &pin.path, &pin.id, &pin.commit, &pin.digest))
}
impl<S: BuildHasher> RecordCache<S> {
    fn get(&self, pin: &RecordPin) -> Option<&Arc<ResolvedRecord>> {
        self.table
            .find(record_hash(&self.hash_builder, pin), |record| {
                record.pin == *pin
            })
    }
    fn insert(&mut self, record: Arc<ResolvedRecord>) -> Arc<ResolvedRecord> {
        let builder = &self.hash_builder;
        self.table
            .entry(
                record_hash(builder, &record.pin),
                |cached| cached.pin == record.pin,
                |cached| record_hash(builder, &cached.pin),
            )
            .or_insert(record)
            .get()
            .clone()
    }
    fn clear(&mut self) {
        self.table.clear();
    }
}

pub struct Resolver {
    pub roots: BTreeMap<String, Root>,
    pub sessions: PathBuf,
    pub logs: Arc<BTreeMap<SessionId, Vec<Envelope>>>,
    records: RecordCache,
    trust: Option<Arc<super::trust::Trust>>,
    receipt_domains: Option<super::receipt::Domains>,
    catalogues: BTreeMap<(String, String), Vec<String>>,
}
impl Resolver {
    pub async fn open(sessions: PathBuf, roots: Vec<Root>) -> Result<Self, Invalid> {
        let input = sessions.clone();
        let logs = crate::runtime::blocking(move || {
            use std::io::Read;
            use std::os::unix::fs::OpenOptionsExt;
            crate::runtime::assert_off_rt_worker();
            let mut logs = BTreeMap::new();
            for file in
                std::fs::read_dir(input).map_err(|_| Invalid("sessions root unavailable"))?
            {
                let file = file.map_err(|_| Invalid("session directory unavailable"))?;
                let path = file.path();
                if path.extension().is_none_or(|ext| ext != "jsonl") {
                    continue;
                }
                let Some(session) = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.parse::<ulid::Ulid>().ok())
                    .map(SessionId)
                else {
                    return Err(Invalid("sessions root contains non-custody JSONL input"));
                };
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
                    .open(&path)
                    .map_err(|_| Invalid("unsafe session input"))?;
                if !file
                    .metadata()
                    .map_err(|_| Invalid("session metadata unavailable"))?
                    .is_file()
                {
                    return Err(Invalid("unsafe session input"));
                }
                let mut text = String::new();
                file.read_to_string(&mut text)
                    .map_err(|_| Invalid("session bytes unavailable"))?;
                let events = text
                    .split_inclusive('\n')
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| {
                        Envelope::from_line(line)
                            .map_err(|_| Invalid("malformed retained envelope"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                logs.insert(session, events);
            }
            Ok::<_, Invalid>(logs)
        })
        .await?;
        let mut mounted = BTreeMap::new();
        for root in roots {
            if mounted.insert(root.name.clone(), root).is_some() {
                return Err(Invalid("duplicate mounted root name"));
            }
        }
        Ok(Self {
            roots: mounted,
            sessions,
            logs: Arc::new(logs),
            records: RecordCache::default(),
            trust: None,
            receipt_domains: None,
            catalogues: BTreeMap::new(),
        })
    }
    /// Supply only independently loaded local policy; records never select policy contents.
    pub fn with_receipt_domains(mut self, domains: super::receipt::Domains) -> Self {
        self.receipt_domains = Some(domains);
        self
    }
    pub fn receipt_domains(&self) -> super::receipt::Domains {
        self.receipt_domains.clone().unwrap_or_else(|| {
            super::receipt::Domains::configured(
                self.sessions.clone(),
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default(),
            )
        })
    }
    pub fn trust(&self) -> Option<Arc<super::trust::Trust>> {
        self.trust.clone()
    }
    pub fn with_trust(mut self, trust: super::trust::Trust) -> Self {
        self.records.clear();
        self.trust = Some(Arc::new(trust));
        self
    }
    /// Accepted verdicts used for graph strength must pass T, not just D0's locator join.
    pub async fn accepted_tasks(&self) -> Result<Vec<super::join::Task>, Invalid> {
        let Some(trust) = &self.trust else {
            return Ok(vec![]);
        };
        let mut accepted = Vec::new();
        for root in self.roots.values() {
            if root.name != root.repo {
                continue;
            }
            for path in root
                .paths(".bboxy/tasks")
                .await?
                .into_iter()
                .filter(|p| p.ends_with(".md"))
            {
                let bytes = root.read(&path, &root.main).await?;
                let repo = root.repo.clone();
                let commit = root.main.clone();
                let task = crate::runtime::blocking(move || {
                    super::join::Task::from_committed(&repo, &path, &commit, &bytes)
                })
                .await;
                if let Ok(task) = task
                    && task.verdict == super::join::Verdict::Accepted
                    && trust.anchor(&task.pin).await.is_ok()
                {
                    accepted.push(task);
                }
            }
        }
        Ok(accepted)
    }
    pub fn event(&self, pin: &EvidenceRef) -> Result<Envelope, Invalid> {
        Self::event_from(&self.logs, pin)
    }
    fn event_from(
        logs: &BTreeMap<SessionId, Vec<Envelope>>,
        pin: &EvidenceRef,
    ) -> Result<Envelope, Invalid> {
        crate::runtime::assert_off_rt_worker();
        let EvidenceRef::Event {
            session,
            seq,
            digest,
        } = pin
        else {
            return Err(Invalid("not an event pin"));
        };
        let event = logs
            .get(session)
            .and_then(|events| {
                events
                    .iter()
                    .find(|e| e.session == *session && e.seq == Some(*seq))
            })
            .ok_or(Invalid("event identity does not resolve"))?;
        let bytes = event
            .wire_bytes()
            .map_err(|_| Invalid("event bytes unavailable"))?;
        if !entry::valid_digest(digest)
            || entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes)) != *digest
        {
            return Err(Invalid("event line digest mismatch"));
        }
        Ok(event.clone())
    }
    pub async fn event_async(&self, pin: &EvidenceRef) -> Result<Envelope, Invalid> {
        let logs = self.logs.clone();
        let pin = pin.clone();
        crate::runtime::blocking(move || Self::event_from(&logs, &pin)).await
    }
    pub async fn record(&mut self, pin: &RecordPin) -> Result<Arc<ResolvedRecord>, Invalid> {
        if !entry::valid_digest(&pin.digest) || !safe_path(&pin.path) {
            return Err(Invalid("invalid memory record pin"));
        }
        if let Some(record) = self.records.get(pin) {
            return Ok(record.clone());
        }
        let root = self
            .roots
            .get(&pin.root)
            .ok_or(Invalid("record root is not mounted"))?;
        let bytes = if let Some(commit) = &pin.commit {
            root.read(&pin.path, commit).await?
        } else {
            let root_path = root.path.clone();
            let path = repository_path(&pin.path)?;
            crate::runtime::blocking(move || {
                use std::{
                    io::Read,
                    os::{
                        fd::{AsRawFd, FromRawFd},
                        unix::fs::OpenOptionsExt,
                    },
                };
                crate::runtime::assert_off_rt_worker();
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
                    .open(root_path)
                    .map_err(|_| Invalid("live record unavailable"))?;
                let components = path.split('/').collect::<Vec<_>>();
                for (index, component) in components.iter().enumerate() {
                    let name = std::ffi::CString::new(*component)
                        .map_err(|_| Invalid("invalid live record path"))?;
                    let flags = libc::O_RDONLY
                        | libc::O_NOFOLLOW
                        | libc::O_NONBLOCK
                        | libc::O_CLOEXEC
                        | if index + 1 < components.len() {
                            libc::O_DIRECTORY
                        } else {
                            0
                        };
                    let descriptor =
                        unsafe { libc::openat(file.as_raw_fd(), name.as_ptr(), flags) };
                    if descriptor < 0 {
                        return Err(Invalid("unsafe live record path"));
                    }
                    file = unsafe { std::fs::File::from_raw_fd(descriptor) };
                }
                if !file
                    .metadata()
                    .map_err(|_| Invalid("live record unavailable"))?
                    .is_file()
                {
                    return Err(Invalid("unsafe live record"));
                }
                let mut bytes = Vec::new();
                file.by_ref()
                    .take(2 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| Invalid("live record unavailable"))?;
                if bytes.len() > 2 * 1024 * 1024 {
                    return Err(Invalid("live record byte bound"));
                }
                Ok(bytes)
            })
            .await?
        };
        let expected = pin.digest.clone();
        let (record, signature_valid) = crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            if entry::digest(&bytes) != expected {
                return Err(Invalid("record byte digest mismatch"));
            }
            let record = Record::parse(&bytes)?;
            let signature_valid = signature_bytes(&record);
            Ok::<_, Invalid>((record, signature_valid))
        })
        .await?;
        if pin
            .id
            .as_deref()
            .is_some_and(|id| record.front.get("id").and_then(Value::as_str) != Some(id))
        {
            return Err(Invalid("record id mismatch"));
        }
        let trusted =
            if let (Some(trust), Some(id), Some(commit)) = (&self.trust, &pin.id, &pin.commit) {
                trust
                    .anchor(&EvidenceRef::Record {
                        root: pin.root.clone(),
                        id: id.clone(),
                        path: pin.path.clone(),
                        commit: commit.clone(),
                        digest: pin.digest.clone(),
                    })
                    .await
                    .is_ok()
            } else {
                false
            };
        let result = Arc::new(ResolvedRecord {
            pin: pin.clone(),
            record,
            trusted,
            signature_valid,
        });
        Ok(self.records.insert(result))
    }
    pub async fn evidence(&mut self, pin: &EvidenceRef) -> Result<bool, Invalid> {
        match pin {
            EvidenceRef::Event { .. } => {
                self.event_async(pin).await?;
                Ok(false)
            }
            EvidenceRef::Record {
                root,
                id,
                path,
                commit,
                digest,
            } => {
                if id.is_empty() || !entry::valid_commit(commit) {
                    return Err(Invalid("incomplete record evidence pin"));
                }
                Ok(self
                    .record(&RecordPin {
                        root: root.clone(),
                        id: Some(id.clone()),
                        path: path.clone(),
                        commit: Some(commit.clone()),
                        digest: digest.clone(),
                    })
                    .await?
                    .trusted)
            }
            _ => Err(Invalid("unknown evidence pin")),
        }
    }
    /// Locator spelling stays in EntryId; resolution returns observed pins without rewriting it.
    pub async fn evidence_value(
        &mut self,
        local_root: &str,
        value: &Value,
    ) -> Result<EvidenceRef, Invalid> {
        if value.get("kind").is_some() {
            let pin: EvidenceRef = serde_json::from_value(value.clone())
                .map_err(|_| Invalid("invalid evidence shape"))?;
            self.evidence(&pin).await?;
            return Ok(pin);
        }
        let reference = value
            .get("ref")
            .and_then(Value::as_str)
            .ok_or(Invalid("missing evidence locator"))?;
        let reference = reference
            .strip_prefix("[[")
            .and_then(|r| r.strip_suffix("]]"))
            .unwrap_or(reference);
        let digest = value
            .get("digest")
            .and_then(Value::as_str)
            .filter(|d| entry::valid_digest(d))
            .ok_or(Invalid("missing exact evidence digest"))?;
        if let Some(event) = reference.strip_prefix("ir:") {
            let (session, seq) = event
                .split_once('#')
                .ok_or(Invalid("invalid event locator"))?;
            let session = SessionId(
                session
                    .parse()
                    .map_err(|_| Invalid("invalid event session"))?,
            );
            let seq = Seq(seq.parse().map_err(|_| Invalid("invalid event sequence"))?);
            let pin = EvidenceRef::Event {
                session,
                seq,
                digest: digest.into(),
            };
            self.event_async(&pin).await?;
            return Ok(pin);
        }
        let (root, locator) = if let Some(cross) = reference.strip_prefix("kb:") {
            let name = self
                .roots
                .keys()
                .filter(|name| cross.starts_with(&format!("{name}/")))
                .max_by_key(|name| name.len())
                .ok_or(Invalid("cross-root evidence is not mounted"))?
                .clone();
            let locator = cross
                .strip_prefix(&format!("{name}/"))
                .ok_or(Invalid("invalid root prefix"))?
                .to_owned();
            (name, locator)
        } else {
            (
                value
                    .get("root")
                    .and_then(Value::as_str)
                    .unwrap_or(local_root)
                    .to_owned(),
                reference.to_owned(),
            )
        };
        if value
            .get("root")
            .and_then(Value::as_str)
            .is_some_and(|r| r != root)
        {
            return Err(Invalid("conflicting evidence root"));
        }
        let commit = value
            .get("commit")
            .and_then(Value::as_str)
            .filter(|c| entry::valid_commit(c))
            .ok_or(Invalid("record evidence requires an exact commit"))?
            .to_owned();
        let key = (root.clone(), commit.clone());
        if !self.catalogues.contains_key(&key) {
            let paths = self
                .roots
                .get(&root)
                .ok_or(Invalid("evidence root is not mounted"))?
                .paths_at(".bboxy/", &commit)
                .await?;
            self.catalogues.insert(key.clone(), paths);
        }
        let paths = self.catalogues[&key]
            .iter()
            .filter(|path| {
                let relative = path.trim_start_matches(".bboxy/").trim_end_matches(".md");
                locator == **path
                    || locator == relative
                    || relative.rsplit('/').next() == Some(locator.as_str())
            })
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err(Invalid("ambiguous or missing committed evidence locator"));
        }
        let path = paths[0].to_owned();
        if value
            .get("path")
            .and_then(Value::as_str)
            .is_some_and(|p| !repository_path(p).is_ok_and(|p| p == path))
        {
            return Err(Invalid("conflicting evidence path"));
        }
        let result = self
            .record(&RecordPin {
                root: root.clone(),
                id: value.get("id").and_then(Value::as_str).map(str::to_owned),
                path: path.clone(),
                commit: Some(commit.clone()),
                digest: digest.into(),
            })
            .await?;
        let id = result
            .record
            .front
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(Invalid("evidence record has no id"))?;
        Ok(EvidenceRef::Record {
            root,
            id: id.into(),
            path,
            commit,
            digest: digest.into(),
        })
    }
    pub async fn scope(&mut self, scope: &Scope, evidence: &[EvidenceRef]) -> Result<(), Invalid> {
        let repos: BTreeSet<_> = scope.repos.iter().collect();
        if repos.is_empty()
            || repos.len() != scope.repos.len()
            || matches!(scope.level, ScopeLevel::Unknown(_))
        {
            return Err(Invalid("invalid memory scope"));
        }
        for pin in evidence {
            let repo = match pin {
                EvidenceRef::Event { session, .. } => {
                    self.event_async(pin).await?;
                    let meta = self
                        .logs
                        .get(session)
                        .and_then(|events| {
                            events.iter().find_map(|e| {
                                if let Event::SessionStarted { meta } = &e.event {
                                    Some(meta)
                                } else {
                                    None
                                }
                            })
                        })
                        .ok_or(Invalid("evidence session header unavailable"))?;
                    match meta.vcs.as_ref().and_then(|v| v.repo.clone()) {
                        Some(repo) => repo,
                        None => crate::vcs::repo_key(&meta.cwd)
                            .await
                            .ok_or(Invalid("evidence session repository unavailable"))?,
                    }
                }
                EvidenceRef::Record { root, .. } => {
                    self.evidence(pin).await?;
                    self.roots
                        .get(root)
                        .ok_or(Invalid("evidence root unavailable"))?
                        .repo
                        .clone()
                }
                _ => return Err(Invalid("unknown scope evidence")),
            };
            if !repos.contains(&repo) {
                return Err(Invalid("evidence repository is outside memory scope"));
            }
        }
        if matches!(scope.level, ScopeLevel::System) || repos.len() > 1 {
            for repo in repos {
                if !self
                    .roots
                    .values()
                    .any(|root| &root.repo == repo && root.cross_repo)
                {
                    return Err(Invalid("cross-repository scope lacks committed opt-in"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::hash::{BuildHasherDefault, Hasher};

    fn pin(n: usize) -> RecordPin {
        RecordPin {
            root: format!("fixture.invalid/dream/repository-{}", n % 16),
            path: format!(".bboxy/memories/lesson-{n}.md"),
            id: Some(format!("record-{n}")),
            commit: Some(format!("{n:040x}")),
            digest: format!("sha256:{n:064x}"),
        }
    }
    fn resolved(pin: RecordPin, marker: usize, trusted: bool) -> Arc<ResolvedRecord> {
        Arc::new(ResolvedRecord {
            pin,
            record: Record {
                front: json!({"marker":marker}),
                yaml: serde_yaml::Value::Null,
                body: format!("synthetic record {marker}"),
                bytes: format!("synthetic bytes {marker}").into_bytes(),
            },
            signature_valid: trusted,
            trusted,
        })
    }

    #[test]
    fn record_cache_matches_all_fields_and_exact_option_identity() {
        let mut base = pin(0);
        base.id = None;
        base.commit = None;
        let mut pins = vec![base.clone()];
        for field in 0..5 {
            let mut different = base.clone();
            match field {
                0 => different.root.push_str("-other"),
                1 => different.path.push_str(".other"),
                2 => different.id = Some(String::new()),
                3 => different.commit = Some(String::new()),
                _ => different.digest.push('1'),
            }
            pins.push(different);
        }
        for value in ["one", "two"] {
            let mut id = base.clone();
            id.id = Some(value.into());
            pins.push(id);
            let mut commit = base.clone();
            commit.commit = Some(value.into());
            pins.push(commit);
        }
        let mut cache = RecordCache {
            table: hashbrown::HashTable::new(),
            hash_builder: BuildHasherDefault::<Collide>::default(),
        };
        let records = pins
            .into_iter()
            .enumerate()
            .map(|(i, p)| resolved(p, i, false))
            .collect::<Vec<_>>();
        for record in &records {
            cache.insert(record.clone());
        }
        assert_eq!(cache.table.len(), records.len());
        for record in records {
            let hit = cache.get(&record.pin).unwrap();
            assert!(Arc::ptr_eq(hit, &record));
            assert_eq!(hit.record.bytes, record.record.bytes);
        }
        assert!(cache.get(&pin(99)).is_none());
    }

    // Deliberate collision control, used only in this private test module.
    #[derive(Default)]
    struct Collide;
    impl Hasher for Collide {
        fn finish(&self) -> u64 {
            7
        }
        fn write(&mut self, _: &[u8]) {}
    }
    fn exercise_growth<S: BuildHasher>(hash_builder: S) -> Vec<String> {
        let mut cache = RecordCache {
            table: hashbrown::HashTable::new(),
            hash_builder,
        };
        let records = (0..512)
            .map(|i| resolved(pin(i), i, true))
            .collect::<Vec<_>>();
        let mut growths = 0;
        let mut capacity = cache.table.capacity();
        for record in &records {
            assert!(Arc::ptr_eq(&cache.insert(record.clone()), record));
            if cache.table.capacity() != capacity {
                growths += 1;
                capacity = cache.table.capacity();
            }
        }
        assert!(growths > 1, "exercise actual resize/rehash callbacks");
        let mut canonical = vec![];
        for i in (0..512).map(|n| (n * 37) % 512) {
            let record = cache.get(&records[i].pin).unwrap();
            assert!(Arc::ptr_eq(record, &records[i]));
            canonical.push(
                serde_json::to_string(&(
                    &record.pin,
                    &record.record.front,
                    &record.record.bytes,
                    record.trusted,
                    record.signature_valid,
                ))
                .unwrap(),
            );
        }
        let contradictory = resolved(records[0].pin.clone(), 9999, false);
        assert!(Arc::ptr_eq(
            &cache.insert(contradictory.clone()),
            &records[0]
        ));
        assert_eq!(cache.table.len(), records.len());
        assert!(cache.get(&records[0].pin).unwrap().trusted);
        cache.clear();
        assert_eq!(cache.table.len(), 0);
        assert!(records.iter().all(|r| cache.get(&r.pin).is_none()));
        assert!(Arc::ptr_eq(
            &cache.insert(contradictory.clone()),
            &contradictory
        ));
        assert!(!cache.get(&contradictory.pin).unwrap().trusted);
        canonical.sort();
        canonical
    }
    #[test]
    fn record_cache_collisions_resize_duplicates_and_clear() {
        let colliding = exercise_growth(BuildHasherDefault::<Collide>::default());
        let seeded = exercise_growth(RandomState::new());
        assert_eq!(
            colliding, seeded,
            "collisions never choose the wrong complete pin"
        );
    }
    #[test]
    fn record_cache_random_seeds_preserve_canonical_observable_records() {
        assert_eq!(
            exercise_growth(RandomState::new()),
            exercise_growth(RandomState::new())
        );
    }

    /// Bounded lookup kernel only: no filesystem, signatures, resolver validation or IO.
    #[test]
    #[ignore = "explicit release-mode measurement, never a CI timing threshold"]
    fn hashbrown_record_cache_bench() {
        use std::{hint::black_box, time::Instant};
        const RECORDS: usize = 4096;
        const LOOKUPS: usize = 262144;
        const SAMPLES: usize = 7;
        let pins = (0..RECORDS).map(pin).collect::<Vec<_>>();
        let mut old = BTreeMap::new();
        let mut new: RecordCache = RecordCache::default();
        for (i, pin) in pins.iter().enumerate() {
            let record = resolved(pin.clone(), i, false);
            old.insert(serde_json::to_string(pin).unwrap(), record.clone());
            new.insert(record);
        }
        let payload = old.keys().map(String::len).sum::<usize>();
        let old_sample = |count: usize| {
            let start = Instant::now();
            for n in 0..count {
                let key = serde_json::to_string(black_box(&pins[(n * 17) % RECORDS])).unwrap();
                black_box(old.get(&key).unwrap().clone());
            }
            start.elapsed().as_nanos()
        };
        let new_sample = |count: usize| {
            let start = Instant::now();
            for n in 0..count {
                black_box(
                    new.get(black_box(&pins[(n * 17) % RECORDS]))
                        .unwrap()
                        .clone(),
                );
            }
            start.elapsed().as_nanos()
        };
        old_sample(16384);
        new_sample(16384);
        let mut old_ns = vec![];
        let mut new_ns = vec![];
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                old_ns.push(old_sample(LOOKUPS));
                new_ns.push(new_sample(LOOKUPS));
            } else {
                new_ns.push(new_sample(LOOKUPS));
                old_ns.push(old_sample(LOOKUPS));
            }
        }
        let median = |values: &[u128]| {
            let mut v = values.to_vec();
            v.sort();
            v[v.len() / 2]
        };
        println!(
            "hashbrown_record_cache_bench {}",
            json!({
                "records":RECORDS,"debug_assertions":cfg!(debug_assertions),"lookups_per_sample_per_path":LOOKUPS,"samples":SAMPLES,
                "warmup_lookups_per_path":16384,"sample_order":"alternating old/new",
                "old_serde_btree_ns":old_ns,"new_hash_table_ns":new_ns,
                "old_median_ns_per_lookup":median(&old_ns) as f64 / LOOKUPS as f64,
                "new_median_ns_per_lookup":median(&new_ns) as f64 / LOOKUPS as f64,
                "old_over_new_median_ratio":median(&old_ns) as f64 / median(&new_ns) as f64,
                "old_duplicate_serialized_key_payload_bytes":payload,
                "new_duplicate_key_payload_bytes":0,
                "old_serialized_key_bytes_per_sample":payload*(LOOKUPS/RECORDS),
                "limits":"cache kernel; payload lengths exclude node/bucket/capacity/allocator overhead; not end-to-end or process memory"
            })
        );
    }
}
