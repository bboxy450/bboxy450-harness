// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! KB integrity is V; repository authorization is independently evaluated as T.
//! Adapted from bboxy450-term b2f4b0c index.rs:1044-1073 and dream §13.1–13.2b.
//! Upstream policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use super::{
    entry::{self, Invalid, Record},
    resolve::{Root, repository_path},
};
use crate::protocol::EvidenceRef;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// This has no authority bit. Passing V never makes the record trusted.
pub fn verify_bytes(bytes: &[u8], id: &str, kinds: &[String]) -> Result<Record, Invalid> {
    super::resolve::checkpoint()?;
    let text = std::str::from_utf8(bytes).map_err(|_| Invalid("MALFORMED"))?;
    if !text.starts_with("---\n") || text.contains("\r\n") {
        return Err(Invalid("MALFORMED"));
    }
    let mut record =
        super::resolve::ProofOwned::new(Record::parse(bytes).map_err(|_| Invalid("MALFORMED"))?);
    super::resolve::checkpoint()?;
    let mut front = super::resolve::ProofOwned::new(
        serde_yaml::from_str::<Value>(
            text.strip_prefix("---\n")
                .unwrap()
                .split_once("\n---\n")
                .map(|(front, _)| front)
                .ok_or(Invalid("MALFORMED"))?,
        )
        .map_err(|_| Invalid("MALFORMED"))?,
    );
    super::resolve::checkpoint()?;
    let object = front.as_object_mut().ok_or(Invalid("MALFORMED"))?;
    if object.keys().next_back().map(String::as_str) != Some("provenance") {
        return Err(Invalid("NONCANONICAL"));
    }
    let proof =
        super::resolve::ProofOwned::new(object.remove("provenance").ok_or(Invalid("MALFORMED"))?);
    let proof_object = proof.as_object().ok_or(Invalid("MALFORMED"))?;
    for field in ["digest", "frontmatter_digest", "signer", "sig"] {
        if !proof_object.get(field).is_some_and(Value::is_string) {
            return Err(Invalid("MALFORMED"));
        }
    }
    let unsigned = serde_yaml::to_string(&*front).map_err(|_| Invalid("MALFORMED"))?;
    super::resolve::checkpoint()?;
    front
        .as_object_mut()
        .unwrap()
        .insert("provenance".into(), proof.clone());
    let rendered = format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(&*front).map_err(|_| Invalid("MALFORMED"))?,
        record.body
    );
    super::resolve::checkpoint()?;
    if rendered.as_bytes() != bytes {
        return Err(Invalid("NONCANONICAL"));
    }
    let f = entry::digest(unsigned.as_bytes());
    let d = entry::digest(record.body.as_bytes());
    if proof["frontmatter_digest"].as_str() != Some(&f) || proof["digest"].as_str() != Some(&d) {
        return Err(Invalid("DIGEST_MISMATCH"));
    }
    let author = &front["authorship"];
    let signer = proof["signer"].as_str().unwrap();
    let kind = author["kind"].as_str().ok_or(Invalid("SIGNER_BINDING"))?;
    if author["signer"].as_str() != Some(signer)
        || front["updated_by"].as_str() != Some(signer)
        || !kinds.iter().any(|k| k == kind)
        || front["id"].as_str() != Some(id)
        || id.parse::<ulid::Ulid>().is_err()
    {
        return Err(Invalid("SIGNER_BINDING"));
    }
    if kind == "seat" {
        let seat = signer
            .strip_prefix("seat:")
            .ok_or(Invalid("SIGNER_BINDING"))?;
        if seat.parse::<ulid::Ulid>().is_err() {
            return Err(Invalid("SIGNER_BINDING"));
        }
        let generation = &author["process_generation"];
        if !generation.is_null()
            && !generation
                .as_str()
                .is_some_and(|g| g.parse::<u64>().is_ok_and(|n| n >= 1 && n.to_string() == g))
        {
            return Err(Invalid("SIGNER_BINDING"));
        }
    }
    let decode = |value: &Value, len| -> Result<Vec<u8>, Invalid> {
        let text = value.as_str().ok_or(Invalid("MALFORMED"))?;
        let decoded = STANDARD.decode(text).map_err(|_| Invalid("MALFORMED"))?;
        if decoded.len() != len || STANDARD.encode(&decoded) != text {
            return Err(Invalid("MALFORMED"));
        }
        Ok(decoded)
    };
    let key = decode(&author["public_key"], 32)?;
    let sig = decode(&proof["sig"], 64)?;
    let pae = format!("DSSEv1 13 bboxy/item/v1 128 {}{}", &f[7..], &d[7..]);
    super::resolve::checkpoint()?;
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
        .verify(pae.as_bytes(), &sig)
        .map_err(|_| Invalid("BAD_SIGNATURE"))?;
    super::resolve::checkpoint()?;
    Ok(record.take_value())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema: String,
    pub roots: Vec<PolicyRoot>,
    pub git_signers: Vec<Signer>,
    pub kb_signer_kinds: Vec<String>,
    pub producers: Vec<ProducerApproval>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRoot {
    pub repo: String,
    pub git_dir: PathBuf,
    pub branch: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signer {
    pub fingerprint: String,
    pub public_key_file: PathBuf,
    pub repos: Vec<String>,
    pub revoked_after: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub repo: String,
    pub commit: String,
    pub tree: String,
    pub version: String,
    pub build: String,
    pub exe_sha256: String,
    pub capabilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "ApprovalWire")]
pub struct ProducerApproval {
    #[serde(flatten)]
    pub producer: Producer,
    pub gate: Gate,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    pub commit: String,
    pub path: String,
    pub sha256: String,
    #[serde(deserialize_with = "policy_record")]
    pub task: EvidenceRef,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub id: String,
    pub exit_code: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "GateWire")]
pub struct ProducerGate {
    pub schema: String,
    #[serde(flatten)]
    pub producer: Producer,
    pub witnesses: Vec<Witness>,
}
fn policy_record<'de, D: serde::Deserializer<'de>>(d: D) -> Result<EvidenceRef, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Pin {
        kind: String,
        root: String,
        id: String,
        path: String,
        commit: String,
        digest: String,
    }
    let p = Pin::deserialize(d)?;
    if p.kind != "record"
        || !normalized(&p.root)
        || p.id.parse::<ulid::Ulid>().is_err()
        || !p.path.starts_with(".bboxy/")
        || repository_path(&p.path).is_err()
        || !entry::valid_commit(&p.commit)
        || !entry::valid_digest(&p.digest)
    {
        return Err(serde::de::Error::custom("invalid policy record pin"));
    }
    Ok(EvidenceRef::Record {
        root: p.root,
        id: p.id,
        path: p.path,
        commit: p.commit,
        digest: p.digest,
    })
}
// Explicit wire structs keep flattened policy/producer fields strict and reject duplicates.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalWire {
    repo: String,
    commit: String,
    tree: String,
    version: String,
    build: String,
    exe_sha256: String,
    capabilities: Vec<String>,
    gate: Gate,
}
impl From<ApprovalWire> for ProducerApproval {
    fn from(w: ApprovalWire) -> Self {
        Self {
            producer: Producer {
                repo: w.repo,
                commit: w.commit,
                tree: w.tree,
                version: w.version,
                build: w.build,
                exe_sha256: w.exe_sha256,
                capabilities: w.capabilities,
            },
            gate: w.gate,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GateWire {
    schema: String,
    repo: String,
    commit: String,
    tree: String,
    version: String,
    build: String,
    exe_sha256: String,
    capabilities: Vec<String>,
    witnesses: Vec<Witness>,
}
impl From<GateWire> for ProducerGate {
    fn from(w: GateWire) -> Self {
        Self {
            producer: Producer {
                repo: w.repo,
                commit: w.commit,
                tree: w.tree,
                version: w.version,
                build: w.build,
                exe_sha256: w.exe_sha256,
                capabilities: w.capabilities,
            },
            schema: w.schema,
            witnesses: w.witnesses,
        }
    }
}
fn raw_sha(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn normalized(repo: &str) -> bool {
    if let Some(path) = repo.strip_prefix("path:") {
        return Path::new(path).is_absolute()
            && !Path::new(path).components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            });
    }
    crate::vcs::normalize_repo_key(&format!("https://{repo}")).as_deref() == Some(repo)
}
impl Policy {
    pub fn parse(bytes: &[u8]) -> Result<Self, Invalid> {
        let policy: Self =
            serde_json::from_slice(bytes).map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
        if policy.schema != "bboxy.dream.trust/2" || policy.kb_signer_kinds.is_empty() {
            return Err(Invalid("TRUST_UNAVAILABLE"));
        }
        let mut repos = std::collections::BTreeSet::new();
        for root in &policy.roots {
            if !normalized(&root.repo)
                || !repos.insert(root.repo.clone())
                || !root.git_dir.is_absolute()
                || !root.branch.starts_with("refs/")
                || root.branch.chars().any(char::is_whitespace)
                || root
                    .branch
                    .contains(['\0', '~', '^', ':', '?', '*', '[', '\\'])
            {
                return Err(Invalid("TRUST_UNAVAILABLE"));
            }
        }
        for signer in &policy.git_signers {
            if signer.fingerprint.len() != 40
                || !signer
                    .fingerprint
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'A'..=b'F').contains(&c))
                || !signer.public_key_file.is_absolute()
                || signer.repos.iter().any(|r| !repos.contains(r))
                || signer
                    .revoked_after
                    .as_deref()
                    .is_some_and(|c| !entry::valid_commit(c))
            {
                return Err(Invalid("TRUST_UNAVAILABLE"));
            }
        }
        for approval in &policy.producers {
            let p = &approval.producer;
            if !repos.contains(&p.repo)
                || !entry::valid_commit(&p.commit)
                || !entry::valid_commit(&p.tree)
                || !entry::valid_commit(&approval.gate.commit)
                || !raw_sha(&p.exe_sha256)
                || !raw_sha(&approval.gate.sha256)
                || repository_path(&approval.gate.path).is_err()
                || !approval.gate.path.starts_with(".bboxy/")
            {
                return Err(Invalid("TRUST_UNAVAILABLE"));
            }
        }
        Ok(policy)
    }
}

/// Selected public keys are admitted immutable bytes, never pathname IO.
/// Only the two existing exact public-key operations accept a key binding.
pub(crate) fn selected_key_command(
    path: &Path,
    args: &[String],
    home: Option<&Path>,
) -> Result<(i32, Vec<u8>), Invalid> {
    let result = selected_key_command_owned(path, args, home);
    if let Err(error) = &result {
        super::resolve::helper_refusal(error.0);
    }
    result
}
fn selected_key_command_owned(
    path: &Path,
    args: &[String],
    home: Option<&Path>,
) -> Result<(i32, Vec<u8>), Invalid> {
    let home = home.ok_or(Invalid("selected private public key custody unavailable"))?;
    let action = args
        .iter()
        .position(|arg| matches!(arg.as_str(), "--show-keys" | "--import"))
        .ok_or(Invalid("selected public key operation unavailable"))?;
    if action + 2 != args.len()
        || args[..action].iter().any(|arg| {
            !matches!(
                arg.as_str(),
                "--no-options" | "--no-autostart" | "--batch" | "--no-tty" | "--with-colons"
            )
        })
    {
        return Err(Invalid("selected public key argument unavailable"));
    }
    if ["--no-options", "--no-autostart", "--batch", "--no-tty"]
        .iter()
        .any(|required| !args[..action].iter().any(|arg| arg == required))
    {
        return Err(Invalid("selected public key isolation unavailable"));
    }
    let bindings =
        super::resolve::bindings().ok_or(Invalid("selected public key bindings unavailable"))?;
    let physical = Path::new(&args[action + 1]);
    if !bindings.keys.values().any(|bound| bound == physical) {
        return Err(Invalid("selected public key binding unavailable"));
    }
    let bytes = bindings
        .key_bytes
        .get(physical)
        .ok_or(Invalid("selected public key bytes unavailable"))?;
    if bytes.is_empty() {
        return Err(Invalid("selected public key byte bound"));
    }
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(Invalid("resource_limit: selected public key byte bound"));
    }
    let mut pipe_args = args.to_vec();
    pipe_args[action + 1] = "-".into();
    super::resolve::selected_command_input(path, "gpg", &pipe_args, Some(home), Some(bytes))
}

/// Every subprocess uses the existing owned process runner and a shared deadline.
/// No operator keyring, Git tracing/configuration, network, hooks or prompts.
async fn command(
    path: &Path,
    args: Vec<String>,
    home: Option<&Path>,
    until: tokio::time::Instant,
) -> Result<(i32, Vec<u8>), Invalid> {
    if super::resolve::selected() {
        let (program, args) = args.split_first().ok_or(Invalid("TRUST_UNAVAILABLE"))?;
        if program == "gpg" {
            return selected_key_command(path, args, home);
        }
        return super::resolve::selected_command(path, program, args, home);
    }
    let mut isolated = Vec::new();
    for name in std::env::vars_os()
        .filter_map(|(k, _)| k.into_string().ok())
        .filter(|k| k.starts_with("GIT_") || k.starts_with("GNUPG") || k == "GPG_AGENT_INFO")
    {
        isolated.extend(["-u".into(), name]);
    }
    isolated.extend(
        [
            "GIT_CONFIG_NOSYSTEM=1",
            "GIT_CONFIG_GLOBAL=/dev/null",
            "GIT_TERMINAL_PROMPT=0",
            "GIT_ASKPASS=/usr/bin/false",
            "SSH_ASKPASS=/usr/bin/false",
        ]
        .map(str::to_owned),
    );
    if let Some(home) = home {
        isolated.push(format!("GNUPGHOME={}", home.display()));
    }
    isolated.extend(args);
    let cap = 2 * 1024 * 1024;
    let out = Box::pin(crate::process::capture_stdout(
        crate::process::Request {
            program: "/usr/bin/env".into(),
            args: isolated,
            stdin: None,
            cwd: path.into(),
            timeout: until
                .checked_duration_since(tokio::time::Instant::now())
                .ok_or(Invalid("TRUST_UNAVAILABLE"))?,
            spill_dir: Default::default(),
            cap,
            redactor: Default::default(),
        },
        tokio_util::sync::CancellationToken::new(),
    ))
    .await
    .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
    if out.cancelled || out.bytes.len() >= cap {
        return Err(Invalid("TRUST_UNAVAILABLE"));
    }
    Ok((out.code.ok_or(Invalid("TRUST_UNAVAILABLE"))?, out.bytes))
}
async fn git(
    root: &Root,
    args: &[&str],
    home: Option<&Path>,
    until: tokio::time::Instant,
) -> Result<Vec<u8>, Invalid> {
    let mut cmd = [
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
        "-c",
        "gpg.format=openpgp",
        "-c",
        "gpg.program=gpg",
        "-c",
        "gpg.openpgp.program=gpg",
    ]
    .map(str::to_owned)
    .to_vec();
    cmd.extend(args.iter().map(|s| (*s).to_owned()));
    let (code, bytes) = command(&root.path, cmd, home, until).await?;
    if code != 0 {
        return Err(Invalid("untrusted_git"));
    }
    Ok(bytes)
}
fn line(bytes: &[u8]) -> Result<&str, Invalid> {
    std::str::from_utf8(bytes)
        .map(str::trim)
        .map_err(|_| Invalid("untrusted_git"))
}

struct ScratchDirectory {
    stream: usize,
    parent: i32,
    name: Option<std::ffi::CString>,
}
impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if self.stream != 0 {
            unsafe {
                libc::closedir(self.stream as *mut libc::DIR);
            }
        }
    }
}
/// Only this constructor's private public-key scratch tree is removable. Walking
/// and unlinking are descriptor-relative and never follow a helper-created link.
struct PublicHome {
    path: PathBuf,
    directories: Vec<ScratchDirectory>,
    pending: Option<std::ffi::CString>,
    started: bool,
    finished: bool,
    failed: bool,
    keep_root: bool,
}
impl std::ops::Deref for PublicHome {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.path
    }
}
impl PublicHome {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            directories: vec![],
            pending: None,
            started: false,
            finished: false,
            failed: false,
            keep_root: false,
        }
    }
    fn finish(&mut self) -> Result<(), Invalid> {
        use super::c1::disposal::Drain;
        super::resolve::cleanup_steps(&mut || self.drain_step());
        if self.failed {
            Err(Invalid("selected public scratch cleanup failed"))
        } else {
            Ok(())
        }
    }
}
impl super::c1::disposal::Drain for PublicHome {
    fn drain_step(&mut self) -> bool {
        if self.finished {
            return false;
        }
        if !self.started {
            self.started = true;
            let Ok(path) = std::ffi::CString::new(self.path.as_os_str().as_encoded_bytes()) else {
                self.failed = true;
                self.finished = true;
                return false;
            };
            let raw = unsafe {
                libc::open(
                    path.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC,
                )
            };
            if raw < 0 {
                self.failed |=
                    std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound;
                self.finished = true;
                return false;
            }
            let stream = unsafe { libc::fdopendir(raw) };
            if stream.is_null() {
                unsafe {
                    libc::close(raw);
                }
                self.failed = true;
                self.finished = true;
                return false;
            }
            self.directories.push(ScratchDirectory {
                stream: stream as usize,
                parent: -1,
                name: None,
            });
            return true;
        }
        if let Some(name) = self.pending.take() {
            let parent =
                unsafe { libc::dirfd(self.directories.last().unwrap().stream as *mut libc::DIR) };
            let mut info = std::mem::MaybeUninit::<libc::stat>::uninit();
            if unsafe {
                libc::fstatat(
                    parent,
                    name.as_ptr(),
                    info.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                self.failed |=
                    std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound;
                return true;
            }
            let info = unsafe { info.assume_init() };
            if info.st_mode & libc::S_IFMT == libc::S_IFDIR {
                // Public GPG scratch has shallow fixed shape. Unexpected trees
                // are retained as a cleanup failure, never traversed unboundedly.
                if self.directories.len() >= 16 {
                    self.failed = true;
                    return true;
                }
                let raw = unsafe {
                    libc::openat(
                        parent,
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC,
                    )
                };
                if raw < 0 {
                    self.failed = true;
                    return true;
                }
                let stream = unsafe { libc::fdopendir(raw) };
                if stream.is_null() {
                    unsafe {
                        libc::close(raw);
                    }
                    self.failed = true;
                    return true;
                }
                self.directories.push(ScratchDirectory {
                    stream: stream as usize,
                    parent,
                    name: Some(name),
                });
            } else if unsafe { libc::unlinkat(parent, name.as_ptr(), 0) } != 0 {
                self.failed |=
                    std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound;
            }
            return true;
        }
        let directory = self.directories.last().unwrap();
        #[cfg(target_os = "macos")]
        unsafe {
            *libc::__error() = 0;
        }
        #[cfg(not(target_os = "macos"))]
        unsafe {
            *libc::__errno_location() = 0;
        }
        let entry = unsafe { libc::readdir(directory.stream as *mut libc::DIR) };
        if !entry.is_null() {
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
            if name.to_bytes() != b"." && name.to_bytes() != b".." {
                self.pending = Some(name.to_owned());
            }
            return true;
        }
        self.failed |= std::io::Error::last_os_error().raw_os_error().unwrap_or(0) != 0;
        let mut directory = self.directories.pop().unwrap();
        unsafe {
            libc::closedir(directory.stream as *mut libc::DIR);
        }
        directory.stream = 0;
        if let Some(name) = &directory.name {
            if unsafe { libc::unlinkat(directory.parent, name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
                self.failed |=
                    std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound;
            }
        } else {
            if !self.keep_root && std::fs::remove_dir(&self.path).is_err() {
                self.failed = true;
            }
            self.finished = true;
        }
        true
    }
}
impl Drop for PublicHome {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish();
        }
    }
}
async fn private_home() -> Result<PublicHome, Invalid> {
    super::resolve::proof_blocking(|| {
        use std::os::unix::fs::DirBuilderExt;
        super::resolve::assert_proof_owner();
        let base = if super::resolve::selected() {
            super::resolve::bindings()
                .and_then(|bindings| bindings.scratch.clone())
                .ok_or(Invalid("helper private scratch unavailable"))?
        } else {
            PathBuf::from("/tmp")
        };
        let path = base.join(format!("dream-public-{}", ulid::Ulid::new()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
        let home = PublicHome::new(path);
        std::fs::write(
            home.path.join("gpg.conf"),
            "no-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\n",
        )
        .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
        Ok(home)
    })
    .await?
}
async fn remove_home(mut home: PublicHome) -> Result<(), Invalid> {
    if super::resolve::selected() {
        return home.finish();
    }
    crate::runtime::blocking(move || home.finish()).await
}
pub(crate) fn cleanup_selected_scratch() -> Result<(), Invalid> {
    if !super::resolve::selected() {
        return Ok(());
    }
    if super::resolve::helper_active() {
        return Err(Invalid("helper still owns private scratch"));
    }
    let Some(path) = super::resolve::bindings().and_then(|bindings| bindings.scratch.clone())
    else {
        return Ok(());
    };
    use std::os::unix::fs::MetadataExt;
    let meta =
        std::fs::symlink_metadata(&path).map_err(|_| Invalid("helper scratch unavailable"))?;
    if !meta.is_dir() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
        return Err(Invalid("helper scratch custody unavailable"));
    }
    let mut scratch = PublicHome::new(path);
    scratch.keep_root = true;
    scratch.finish()
}
pub struct Trust {
    policy: Policy,
    /// Only independently bound repository keys are present. No aliases.
    roots: BTreeMap<String, Root>,
    selected_cache_tag: String,
}
impl Trust {
    /// Parse the exact admitted public policy; bind its logical roots and keys
    /// solely to independently mounted finite snapshots. Never open a policy
    /// supplied pathname or infer origin/main or the operator's HOME/keyring.
    pub(crate) async fn load_selected(bytes: &[u8], supplied: Vec<Root>) -> Result<Self, Invalid> {
        if !super::resolve::selected() {
            return Err(Invalid("selected trust owner unavailable"));
        }
        super::resolve::checkpoint()?;
        let mut policy = Policy::parse(bytes)?;
        super::resolve::admit_policy_branches(policy.roots.iter().map(|root| root.branch.clone()))?;
        let bindings =
            super::resolve::bindings().ok_or(Invalid("selected trust bindings unavailable"))?;
        let mut roots = BTreeMap::new();
        for config in &policy.roots {
            super::resolve::checkpoint()?;
            let found = supplied
                .iter()
                .filter(|root| root.repo == config.repo)
                .collect::<Vec<_>>();
            if found.len() != 1 {
                return Err(Invalid("selected trust root unavailable"));
            }
            let root = found[0].clone();
            let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
            let bare = git(&root, &["rev-parse", "--is-bare-repository"], None, until).await?;
            let actual = git(&root, &["rev-parse", "--show-toplevel"], None, until).await?;
            let head = git(
                &root,
                &["rev-parse", "--verify", &config.branch],
                None,
                until,
            )
            .await?;
            let origin = git(
                &root,
                &["config", "--get", "remote.origin.url"],
                None,
                until,
            )
            .await?;
            let main = git(
                &root,
                &["rev-parse", "--verify", "refs/remotes/origin/main"],
                None,
                until,
            )
            .await?;
            if bare != b"false\n"
                || Path::new(line(&actual)?) != root.path
                || line(&head)? != root.main
                || line(&main)? != root.main
                || crate::vcs::normalize_repo_key(line(&origin)?).as_deref()
                    != Some(config.repo.as_str())
                || roots.insert(config.repo.clone(), root).is_some()
            {
                return Err(Invalid("selected trust root mismatch"));
            }
        }
        for signer in &mut policy.git_signers {
            super::resolve::checkpoint()?;
            signer.public_key_file = bindings
                .keys
                .get(&signer.public_key_file)
                .cloned()
                .ok_or(Invalid("selected public key unavailable"))?;
        }
        let result = Self {
            policy,
            roots,
            selected_cache_tag: ulid::Ulid::new().to_string(),
        };
        let private = private_home().await?;
        let checked = async {
            for signer in &result.policy.git_signers {
                super::resolve::checkpoint()?;
                let root = result
                    .roots
                    .values()
                    .next()
                    .ok_or(Invalid("selected trust root unavailable"))?;
                let (code, bytes) = command(
                    &root.path,
                    vec![
                        "gpg".into(),
                        "--no-options".into(),
                        "--no-autostart".into(),
                        "--batch".into(),
                        "--no-tty".into(),
                        "--with-colons".into(),
                        "--show-keys".into(),
                        signer.public_key_file.to_string_lossy().into_owned(),
                    ],
                    Some(&private),
                    tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET,
                )
                .await?;
                let mut primary = false;
                let mut fingerprints = vec![];
                for row in line(&bytes)?.lines() {
                    super::resolve::checkpoint()?;
                    let fields = row.split(':').collect::<Vec<_>>();
                    if fields[0] == "pub" {
                        primary = true;
                    } else if fields[0] == "sub" {
                        primary = false;
                    } else if fields[0] == "fpr" && primary {
                        fingerprints.push(fields.get(9).copied().unwrap_or(""));
                        primary = false;
                    }
                    if fields[0] == "sec" || fields[0] == "ssb" {
                        return Err(Invalid("selected public key required"));
                    }
                }
                if code != 0 || fingerprints != [signer.fingerprint.as_str()] {
                    return Err(Invalid("TRUST_UNAVAILABLE"));
                }
            }
            Ok(())
        }
        .await;
        remove_home(private).await?;
        checked?;
        Ok(result)
    }
    pub async fn load(path: PathBuf, forbidden: Vec<PathBuf>) -> Result<Self, Invalid> {
        let (policy, file) = super::resolve::proof_blocking(move || {
            use std::{
                io::Read,
                os::unix::fs::{MetadataExt, OpenOptionsExt},
            };
            super::resolve::assert_proof_owner();
            let mut fd = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
                .open(&path)
                .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
            let meta = fd.metadata().map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
            if !meta.is_file()
                || meta.uid() != unsafe { libc::geteuid() }
                || meta.mode() & 0o022 != 0
            {
                return Err(Invalid("TRUST_UNAVAILABLE"));
            }
            let file = std::fs::canonicalize(path).map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
            for excluded in forbidden {
                if std::fs::canonicalize(excluded).is_ok_and(|p| file.starts_with(p)) {
                    return Err(Invalid("TRUST_UNAVAILABLE"));
                }
            }
            let mut bytes = Vec::new();
            fd.by_ref()
                .take(2 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
            if bytes.len() > 2 * 1024 * 1024 {
                return Err(Invalid("TRUST_UNAVAILABLE"));
            }
            Ok((Policy::parse(&bytes)?, file))
        })
        .await??;
        // The policy may not be smuggled through an unrelated repository either.
        let location = Root {
            name: String::new(),
            repo: String::new(),
            path: file.parent().unwrap().into(),
            main: String::new(),
            cross_repo: false,
        };
        if git(
            &location,
            &["rev-parse", "--absolute-git-dir"],
            None,
            tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET,
        )
        .await
        .is_ok()
        {
            return Err(Invalid("TRUST_UNAVAILABLE"));
        }
        let mut roots = BTreeMap::new();
        let mut paths = std::collections::BTreeSet::new();
        for config in &policy.roots {
            if let Ok(mut root) = Root::open(config.repo.clone(), config.git_dir.clone()).await {
                if file.starts_with(&root.path) || !paths.insert(root.path.clone()) {
                    return Err(Invalid("TRUST_UNAVAILABLE"));
                }
                if root.repo != config.repo {
                    continue;
                }
                let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
                if let Ok(bytes) = git(
                    &root,
                    &["rev-parse", "--verify", &config.branch],
                    None,
                    until,
                )
                .await
                    && let Ok(commit) = line(&bytes)
                    && entry::valid_commit(commit)
                {
                    root.main = commit.into();
                    roots.insert(config.repo.clone(), root);
                }
            }
        }
        let result = Self {
            policy,
            roots,
            selected_cache_tag: ulid::Ulid::new().to_string(),
        };
        // Fingerprint inspection never consults or creates the operator's keyring.
        let private = private_home().await?;
        let checked = async {
            // Fingerprints are a policy-wide requirement, independent of individual roots.
            for signer in &result.policy.git_signers {
                let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
                let (code, bytes) = command(
                    file.parent().unwrap(),
                    vec![
                        "gpg".into(),
                        "--no-options".into(),
                        "--no-autostart".into(),
                        "--batch".into(),
                        "--no-tty".into(),
                        "--with-colons".into(),
                        "--show-keys".into(),
                        signer.public_key_file.to_string_lossy().into_owned(),
                    ],
                    Some(&private),
                    until,
                )
                .await?;
                let mut primary = false;
                let mut fingerprints = vec![];
                for row in line(&bytes)?.lines() {
                    let fields = row.split(':').collect::<Vec<_>>();
                    if fields[0] == "pub" {
                        primary = true;
                    } else if fields[0] == "sub" {
                        primary = false;
                    } else if fields[0] == "fpr" && primary {
                        fingerprints.push(fields.get(9).copied().unwrap_or(""));
                        primary = false;
                    }
                }
                if code != 0 || fingerprints != [signer.fingerprint.as_str()] {
                    return Err(Invalid("TRUST_UNAVAILABLE"));
                }
            }
            Ok::<_, Invalid>(())
        }
        .await;
        remove_home(private).await?;
        checked?;
        Ok(result)
    }
    async fn signed(&self, root: &Root, commit: &str) -> Result<(), Invalid> {
        let cache_key = format!("sig:{}:{}:{commit}", self.selected_cache_tag, root.repo);
        if super::resolve::selected() {
            super::resolve::checkpoint()?;
            if super::resolve::trust_cache_has(&cache_key) {
                return Ok(());
            }
        }
        let keys = self
            .policy
            .git_signers
            .iter()
            .filter(|s| s.repos.contains(&root.repo))
            .cloned()
            .collect::<Vec<_>>();
        let private = private_home().await?;
        let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
        let result = async {
            for key in &keys {
                super::resolve::checkpoint()?;
                let (code, _) = command(
                    &root.path,
                    vec![
                        "gpg".into(),
                        "--no-options".into(),
                        "--no-autostart".into(),
                        "--batch".into(),
                        "--no-tty".into(),
                        "--import".into(),
                        key.public_key_file.to_string_lossy().into_owned(),
                    ],
                    Some(&private),
                    until,
                )
                .await?;
                if code != 0 {
                    return Err(Invalid("TRUST_UNAVAILABLE"));
                }
            }
            let bytes = git(
                root,
                &["log", "-1", "--format=%G?%x00%GF%x00%GP", commit],
                Some(&private),
                until,
            )
            .await?;
            let fields = line(&bytes)?.split('\0').collect::<Vec<_>>();
            if fields.len() != 3 || !matches!(fields[0], "G" | "U") {
                return Err(Invalid("unknown_signer"));
            }
            for signer in &keys {
                super::resolve::checkpoint()?;
                if fields[1] != signer.fingerprint && fields[2] != signer.fingerprint {
                    continue;
                }
                if let Some(cut) = &signer.revoked_after
                    && (!entry::valid_commit(cut)
                        || git(
                            root,
                            &["merge-base", "--is-ancestor", commit, cut],
                            None,
                            until,
                        )
                        .await
                        .is_err())
                {
                    continue;
                }
                return Ok(());
            }
            Err(Invalid("unknown_or_revoked_signer"))
        }
        .await;
        remove_home(private).await?;
        if result.is_ok() && super::resolve::selected() {
            super::resolve::trust_cache_put(cache_key);
        }
        result
    }
    pub async fn anchor(&self, pin: &EvidenceRef) -> Result<Record, Invalid> {
        let EvidenceRef::Record { root, .. } = pin else {
            return Err(Invalid("not_record"));
        };
        let root = if super::resolve::selected() {
            let mut matches = self.roots.values().filter(|mounted| mounted.name == *root);
            let found = matches.next().ok_or(Invalid("unknown_root"))?;
            if matches.next().is_some() {
                return Err(Invalid("ambiguous selected root"));
            }
            found
        } else {
            self.roots.get(root).ok_or(Invalid("unknown_root"))?
        };
        self.anchor_at(pin, root).await
    }
    async fn anchor_at(&self, pin: &EvidenceRef, root: &Root) -> Result<Record, Invalid> {
        let EvidenceRef::Record {
            id,
            path,
            commit,
            digest,
            ..
        } = pin
        else {
            return Err(Invalid("not_record"));
        };
        let path = repository_path(path)?;
        let bytes = root.read(&path, commit).await?;
        super::resolve::reserve_record(&bytes)?;
        let expected = digest.clone();
        let id = id.clone();
        let kinds = self.policy.kb_signer_kinds.clone();
        let record = super::resolve::proof_blocking(move || {
            if entry::digest(&bytes) != expected {
                return Err(Invalid("pin_digest"));
            }
            verify_bytes(&bytes, &id, &kinds)
        })
        .await??;
        if record.front["type"] != "decision"
            && !(record.front["type"] == "task"
                && (record.front.get("verify_result").is_some()
                    || record.front.get("rejection").is_some()))
        {
            return Err(Invalid("not_anchor"));
        }
        let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
        let last = git(root, &["rev-list", "-1", commit, "--", &path], None, until).await?;
        let last = line(&last)?;
        if !entry::valid_commit(last) {
            return Err(Invalid("last_touch_unavailable"));
        }
        let expected = git(
            root,
            &["rev-parse", &format!("{commit}:{path}")],
            None,
            until,
        )
        .await?;
        let actual = git(root, &["rev-parse", &format!("{last}:{path}")], None, until).await?;
        if actual != expected {
            return Err(Invalid("last_touch_blob"));
        }
        self.signed(root, last).await?;
        Ok(record)
    }
    /// Measure the running artifact independently of any recalled/session cwd.
    /// Producers call this during startup; process-wide identity is measured once.
    pub async fn current_producer(&self, required: &[&str]) -> Result<Producer, Invalid> {
        static EXE: std::sync::OnceLock<Result<String, Invalid>> = std::sync::OnceLock::new();
        let digest = super::resolve::proof_blocking(|| {
            EXE.get_or_init(|| {
                use sha2::Digest;
                use std::io::Read;
                let path = std::env::current_exe().map_err(|_| Invalid("producer_unapproved"))?;
                let mut file =
                    std::fs::File::open(path).map_err(|_| Invalid("producer_unapproved"))?;
                if !file
                    .metadata()
                    .map_err(|_| Invalid("producer_unapproved"))?
                    .is_file()
                {
                    return Err(Invalid("producer_unapproved"));
                }
                let mut hash = sha2::Sha256::new();
                let mut bytes = [0; 65536];
                loop {
                    let n = file
                        .read(&mut bytes)
                        .map_err(|_| Invalid("producer_unapproved"))?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&bytes[..n]);
                }
                Ok(format!("{:x}", hash.finalize()))
            })
            .clone()
        })
        .await??;
        let candidates = self
            .policy
            .producers
            .iter()
            .filter(|p| p.producer.exe_sha256 == digest)
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(Invalid("producer_unapproved"));
        }
        let producer = candidates[0].producer.clone();
        self.approve(&producer, required).await?;
        Ok(producer)
    }
    pub async fn approve(&self, producer: &Producer, required: &[&str]) -> Result<(), Invalid> {
        let entries = self
            .policy
            .producers
            .iter()
            .filter(|p| p.producer == *producer)
            .collect::<Vec<_>>();
        if entries.len() != 1
            || required
                .iter()
                .any(|c| !producer.capabilities.iter().any(|p| p == c))
        {
            return Err(Invalid("producer_unapproved"));
        }
        let cache_key = if super::resolve::selected() {
            super::resolve::checkpoint()?;
            let encoded = serde_json::to_string(&(producer, required))
                .map_err(|_| Invalid("producer_unapproved"))?;
            let key = format!("producer:{}:{encoded}", self.selected_cache_tag);
            if super::resolve::trust_cache_has(&key) {
                return Ok(());
            }
            Some(key)
        } else {
            None
        };
        let root = self
            .roots
            .get(&producer.repo)
            .ok_or(Invalid("producer_unapproved"))?;
        let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
        if git(root, &["cat-file", "-t", &producer.commit], None, until).await? != b"commit\n" {
            return Err(Invalid("producer_unapproved"));
        }
        git(
            root,
            &["merge-base", "--is-ancestor", &producer.commit, &root.main],
            None,
            until,
        )
        .await?;
        self.signed(root, &producer.commit).await?;
        let tree = git(
            root,
            &["rev-parse", &format!("{}^{{tree}}", producer.commit)],
            None,
            until,
        )
        .await?;
        if line(&tree)? != producer.tree {
            return Err(Invalid("producer_unapproved"));
        }
        let gate = &entries[0].gate;
        let bytes = root.read(&gate.path, &gate.commit).await?;
        let expected = gate.sha256.clone();
        let bytes = super::resolve::proof_blocking(move || {
            if super::fingerprint::digest(&bytes) != expected {
                return Err(Invalid("producer_unapproved"));
            }
            Ok(bytes)
        })
        .await??;
        let last = git(
            root,
            &["rev-list", "-1", &gate.commit, "--", &gate.path],
            None,
            until,
        )
        .await?;
        let last = line(&last)?;
        let expected = git(
            root,
            &["rev-parse", &format!("{}:{}", gate.commit, gate.path)],
            None,
            until,
        )
        .await?;
        let actual = git(
            root,
            &["rev-parse", &format!("{last}:{}", gate.path)],
            None,
            until,
        )
        .await?;
        if expected != actual {
            return Err(Invalid("producer_unapproved"));
        }
        self.signed(root, last).await?;
        let record = self.anchor_at(&gate.task, root).await?;
        if !matches!(record.front["status"].as_str(), Some("accepted" | "done"))
            || record.front["verify_result"]["exit_code"].as_i64() != Some(0)
        {
            return Err(Invalid("producer_unapproved"));
        }
        let report = record.front["report"]["commit"]
            .as_str()
            .ok_or(Invalid("producer_unapproved"))?;
        if !entry::valid_commit(report) {
            return Err(Invalid("producer_unapproved"));
        }
        git(
            root,
            &["merge-base", "--is-ancestor", &producer.commit, report],
            None,
            tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET,
        )
        .await
        .map_err(|_| Invalid("report_source_ancestry"))?;
        let EvidenceRef::Record {
            root: task_root,
            id: task_id,
            ..
        } = &gate.task
        else {
            return Err(Invalid("producer_unapproved"));
        };
        if task_root != &producer.repo {
            return Err(Invalid("producer_unapproved"));
        }
        git(
            root,
            &["merge-base", "--is-ancestor", &gate.commit, report],
            None,
            tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET,
        )
        .await?;
        let locator = record.front["report"]["report"]
            .as_str()
            .ok_or(Invalid("producer_unapproved"))?;
        let locator = locator
            .trim_start_matches("[[")
            .trim_end_matches("]]")
            .trim_end_matches(".md");
        let paths = root.paths_at(".bboxy/reports", report).await?;
        let paths = paths
            .into_iter()
            .filter(|p| p.trim_start_matches(".bboxy/").trim_end_matches(".md") == locator)
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err(Invalid("producer_unapproved"));
        }
        let path = &paths[0];
        let report_bytes = root.read(path, report).await?;
        super::resolve::reserve_record(&report_bytes)?;
        let kinds = self.policy.kb_signer_kinds.clone();
        let source = producer.commit.clone();
        let gate_commit = gate.commit.clone();
        let task_id = task_id.clone();
        let task_path = if let EvidenceRef::Record { path, .. } = &gate.task {
            path.clone()
        } else {
            String::new()
        };
        let task_number = record
            .front
            .get("number")
            .and_then(|n| n.as_u64().or_else(|| n.as_str()?.parse().ok()));
        super::resolve::proof_blocking(move || {
            let parsed = Record::parse(&report_bytes)?;
            let id = parsed.front["id"]
                .as_str()
                .ok_or(Invalid("producer_unapproved"))?;
            let parsed = verify_bytes(&report_bytes, id, &kinds)?;
            if parsed.front["type"] != "report"
                || !parsed.front["commits"].as_array().is_some_and(|c| {
                    c.contains(&Value::String(source)) && c.contains(&Value::String(gate_commit))
                })
                || !parsed.front["task"].as_str().is_some_and(|s| {
                    let s = s
                        .trim_start_matches("[[")
                        .trim_end_matches("]]")
                        .trim_end_matches(".md");
                    s == task_id
                        || s == task_path.trim_end_matches(".md")
                        || s == task_path
                            .trim_start_matches(".bboxy/")
                            .trim_end_matches(".md")
                        || s == task_path
                            .trim_start_matches(".bboxy/tasks/")
                            .trim_end_matches(".md")
                        || task_number.is_some_and(|n| s == format!("#{n}"))
                })
            {
                return Err(Invalid("producer_unapproved"));
            }
            Ok(())
        })
        .await??;
        let until = tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET;
        let touching = git(root, &["rev-list", "-1", report, "--", path], None, until).await?;
        let touching = line(&touching)?;
        if git(
            root,
            &["rev-parse", &format!("{touching}:{path}")],
            None,
            until,
        )
        .await?
            != git(
                root,
                &["rev-parse", &format!("{report}:{path}")],
                None,
                until,
            )
            .await?
        {
            return Err(Invalid("producer_unapproved"));
        }
        self.signed(root, touching).await?;
        let expected = producer.clone();
        let checked = super::resolve::proof_blocking(move || {
            let gate: ProducerGate =
                serde_json::from_slice(&bytes).map_err(|_| Invalid("producer_unapproved"))?;
            if gate.schema != "bboxy.dream.producer-gate/1" || gate.producer != expected {
                return Err(Invalid("producer_unapproved"));
            }
            for capability in &expected.capabilities {
                super::resolve::checkpoint()?;
                let ids: &[&str] = match capability.as_str() {
                    "receipt/1" => &["dream_trust::receipt_header_body_roundtrip"],
                    "effect-wrapper/1" => &["dream_sinks::receipt_ops_match_independent_observer"],
                    "constructive-seal/1" => &[
                        "dream_trust::receipt_seal_constructs_without_self_hash",
                        "dream_sinks::late_owned_source_write_after_seal_rejected",
                        "dream_sinks::broken_seal_exception_fails_audit",
                    ],
                    "no-replace/1" => &[
                        "dream_sinks::hand_write_between_read_and_link_preserved",
                        "dream_sinks::overwrite_rename_control_fails_audit",
                    ],
                    _ => return Err(Invalid("producer_unapproved")),
                };
                for id in ids {
                    super::resolve::checkpoint()?;
                    let witnesses = gate
                        .witnesses
                        .iter()
                        .filter(|w| w.id == *id)
                        .collect::<Vec<_>>();
                    if witnesses.len() != 1 || witnesses[0].exit_code != 0 {
                        return Err(Invalid("producer_unapproved"));
                    }
                }
            }
            Ok(())
        })
        .await?;
        if checked.is_ok()
            && let Some(key) = cache_key
        {
            super::resolve::trust_cache_put(key);
        }
        checked
    }
}

impl super::c1::disposal::Drain for Trust {
    fn drain_step(&mut self) -> bool {
        if self.roots.pop_first().is_some()
            || self.policy.roots.pop().is_some()
            || self.policy.kb_signer_kinds.pop().is_some()
        {
            return true;
        }
        if let Some(signer) = self.policy.git_signers.last_mut() {
            if signer.repos.pop().is_none() {
                self.policy.git_signers.pop();
            }
            return true;
        }
        if let Some(approval) = self.policy.producers.last_mut() {
            if approval.producer.capabilities.pop().is_none() {
                self.policy.producers.pop();
            }
            return true;
        }
        false
    }
}

impl super::c1::disposal::Drain for Policy {
    fn drain_step(&mut self) -> bool {
        if self.roots.pop().is_some() || self.kb_signer_kinds.pop().is_some() {
            return true;
        }
        if let Some(signer) = self.git_signers.last_mut() {
            if signer.repos.pop().is_none() {
                self.git_signers.pop();
            }
            return true;
        }
        if let Some(approval) = self.producers.last_mut() {
            if approval.producer.capabilities.pop().is_none() {
                self.producers.pop();
            }
            return true;
        }
        false
    }
}
