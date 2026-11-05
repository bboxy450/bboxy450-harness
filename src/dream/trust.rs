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
    let text = std::str::from_utf8(bytes).map_err(|_| Invalid("MALFORMED"))?;
    if !text.starts_with("---\n") || text.contains("\r\n") {
        return Err(Invalid("MALFORMED"));
    }
    let record = Record::parse(bytes).map_err(|_| Invalid("MALFORMED"))?;
    let mut front: Value = serde_yaml::from_str(
        text.strip_prefix("---\n")
            .unwrap()
            .split_once("\n---\n")
            .map(|(front, _)| front)
            .ok_or(Invalid("MALFORMED"))?,
    )
    .map_err(|_| Invalid("MALFORMED"))?;
    let object = front.as_object_mut().ok_or(Invalid("MALFORMED"))?;
    if object.keys().next_back().map(String::as_str) != Some("provenance") {
        return Err(Invalid("NONCANONICAL"));
    }
    let proof = object.remove("provenance").ok_or(Invalid("MALFORMED"))?;
    let proof_object = proof.as_object().ok_or(Invalid("MALFORMED"))?;
    for field in ["digest", "frontmatter_digest", "signer", "sig"] {
        if !proof_object.get(field).is_some_and(Value::is_string) {
            return Err(Invalid("MALFORMED"));
        }
    }
    let unsigned = serde_yaml::to_string(&front).map_err(|_| Invalid("MALFORMED"))?;
    front
        .as_object_mut()
        .unwrap()
        .insert("provenance".into(), proof.clone());
    let rendered = format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(&front).map_err(|_| Invalid("MALFORMED"))?,
        record.body
    );
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
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
        .verify(pae.as_bytes(), &sig)
        .map_err(|_| Invalid("BAD_SIGNATURE"))?;
    Ok(record)
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

/// Every subprocess uses the existing owned process runner and a shared deadline.
/// No operator keyring, Git tracing/configuration, network, hooks or prompts.
async fn command(
    path: &Path,
    args: Vec<String>,
    home: Option<&Path>,
    until: tokio::time::Instant,
) -> Result<(i32, Vec<u8>), Invalid> {
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

async fn private_home() -> Result<PathBuf, Invalid> {
    crate::runtime::blocking(|| {
        use std::os::unix::fs::DirBuilderExt;
        crate::runtime::assert_off_rt_worker();
        // Short private path keeps GnuPG Unix sockets below macOS sockaddr_un bounds.
        let path = PathBuf::from("/tmp").join(format!("dream-public-{}", ulid::Ulid::new()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|_| Invalid("TRUST_UNAVAILABLE"))?;
        if std::fs::write(
            path.join("gpg.conf"),
            "no-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\n",
        )
        .is_err()
        {
            let _ = std::fs::remove_dir_all(&path);
            return Err(Invalid("TRUST_UNAVAILABLE"));
        }
        Ok(path)
    })
    .await
}
async fn remove_home(path: PathBuf) {
    crate::runtime::blocking(move || {
        let _ = std::fs::remove_dir_all(path);
    })
    .await;
}
pub struct Trust {
    policy: Policy,
    /// Only independently bound repository keys are present. No aliases.
    roots: BTreeMap<String, Root>,
}
impl Trust {
    pub async fn load(path: PathBuf, forbidden: Vec<PathBuf>) -> Result<Self, Invalid> {
        let (policy, file) = crate::runtime::blocking(move || {
            use std::{
                io::Read,
                os::unix::fs::{MetadataExt, OpenOptionsExt},
            };
            crate::runtime::assert_off_rt_worker();
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
        .await?;
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
        let result = Self { policy, roots };
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
        remove_home(private).await;
        checked?;
        Ok(result)
    }
    async fn signed(&self, root: &Root, commit: &str) -> Result<(), Invalid> {
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
        remove_home(private).await;
        result
    }
    pub async fn anchor(&self, pin: &EvidenceRef) -> Result<Record, Invalid> {
        let EvidenceRef::Record {
            root,
            id,
            path,
            commit,
            digest,
        } = pin
        else {
            return Err(Invalid("not_record"));
        };
        let root = self.roots.get(root).ok_or(Invalid("unknown_root"))?;
        let path = repository_path(path)?;
        let bytes = root.read(&path, commit).await?;
        let expected = digest.clone();
        let id = id.clone();
        let kinds = self.policy.kb_signer_kinds.clone();
        let record = crate::runtime::blocking(move || {
            if entry::digest(&bytes) != expected {
                return Err(Invalid("pin_digest"));
            }
            verify_bytes(&bytes, &id, &kinds)
        })
        .await?;
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
        let bytes = crate::runtime::blocking(move || {
            if super::fingerprint::digest(&bytes) != expected {
                return Err(Invalid("producer_unapproved"));
            }
            Ok(bytes)
        })
        .await?;
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
        let record = self.anchor(&gate.task).await?;
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
            &["merge-base", "--is-ancestor", report, &producer.commit],
            None,
            tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET,
        )
        .await?;
        let expected = producer.clone();
        crate::runtime::blocking(move || {
            let gate: ProducerGate =
                serde_json::from_slice(&bytes).map_err(|_| Invalid("producer_unapproved"))?;
            if gate.schema != "bboxy.dream.producer-gate/1" || gate.producer != expected {
                return Err(Invalid("producer_unapproved"));
            }
            for capability in &expected.capabilities {
                let ids: &[&str] = match capability.as_str() {
                    "receipt/1" => &["dream_trust::receipt_header_body_roundtrip"],
                    "effect-wrapper/1" => &["dream_sinks::receipt_ops_match_independent_observer"],
                    "no-replace/1" => &[
                        "dream_sinks::hand_write_between_read_and_link_preserved",
                        "dream_sinks::overwrite_rename_control_fails_audit",
                    ],
                    _ => return Err(Invalid("producer_unapproved")),
                };
                for id in ids {
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
        .await
    }
}
