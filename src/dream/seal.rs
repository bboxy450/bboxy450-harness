// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Constructive per-invocation freeze and retained prefix/suffix validation.
//! Model: HarnessDreamSeal; hashes/OS confinement/completeness remain real witnesses.
use super::receipt::{Effect, Operation};
use crate::protocol::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofError {
    Fail(&'static str),
    Unproven(&'static str),
}
pub type Proof<T> = Result<T, ProofError>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub session: SessionId,
    pub through_seq: u64,
    pub prefix_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChildState {
    Joined,
    KilledReaped,
    Active,
    Unknown,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChildCoverage {
    Observed,
    EnforcedReadOnly,
    Unobserved,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Child {
    pub id: ulid::Ulid,
    pub state: ChildState,
    pub coverage: ChildCoverage,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub observer: String,
    pub covers: Vec<String>,
    pub excludes: Vec<String>,
    pub interval: String,
}
impl Default for Coverage {
    fn default() -> Self {
        Self {
            observer: "approved-producer-self".into(),
            covers: vec!["producer_protected_effects".into()],
            excludes: vec!["external_writers".into(), "native_loader_state".into()],
            interval: "start_through_freeze".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Seal {
    pub version: u64,
    pub receipt_item: ItemId,
    pub own_prefix: Prefix,
    pub effect_count: u64,
    pub children: Vec<Child>,
}

pub fn prefix(events: &[Envelope], through: u64) -> Proof<String> {
    if through == 0 || through as usize > events.len() {
        return Err(ProofError::Unproven("truncated_witness"));
    }
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for (index, event) in events.iter().take(through as usize).enumerate() {
        super::resolve::checkpoint()?;
        if event.seq != Some(Seq(index as u64 + 1)) {
            return Err(ProofError::Unproven("truncated_witness"));
        }
        let wire = event
            .wire_bytes()
            .map_err(|_| ProofError::Unproven("truncated_witness"))?;
        if !wire.ends_with(b"\n") {
            return Err(ProofError::Unproven("truncated_witness"));
        }
        for chunk in wire.chunks(16384) {
            super::resolve::checkpoint()?;
            hash.update(chunk);
        }
    }
    Ok(format!("sha256:{:x}", hash.finalize()))
}
impl Seal {
    pub fn validate(
        &self,
        coverage: &Coverage,
        effects: &[Effect],
        events: &[Envelope],
        session: SessionId,
        item: ItemId,
        publication: Option<usize>,
    ) -> Proof<()> {
        if *coverage != Coverage::default() || self.version != 1 {
            return Err(ProofError::Unproven("unknown_seal_coverage"));
        }
        if self.receipt_item != item || self.own_prefix.session != session {
            return Err(ProofError::Fail("seal_item_mismatch"));
        }
        if self.effect_count != effects.len() as u64 {
            return Err(ProofError::Fail("effect_count"));
        }
        if effects.iter().enumerate().any(|(i, e)| e.i != i as u64) {
            return Err(ProofError::Unproven("truncated_witness"));
        }
        if prefix(events, self.own_prefix.through_seq)? != self.own_prefix.prefix_digest {
            return Err(ProofError::Fail("own_prefix_mismatch"));
        }
        let n = self.own_prefix.through_seq as usize;
        if !matches!(&events[n-1].event,Event::ItemStarted{item:i,kind:ItemKind::Receipt,..} if *i==item)
        {
            return Err(ProofError::Fail("seal_item_mismatch"));
        }
        let Some(completed) = events.get(n) else {
            return Err(ProofError::Unproven("truncated_witness"));
        };
        if !matches!(&completed.event,Event::ItemCompleted{item:i,status:ItemStatus::Completed,body:Item::Receipt{..}} if *i==item)
        {
            return Err(ProofError::Fail("forbidden_suffix"));
        }
        let mut prior = None;
        for child in &self.children {
            if prior.is_some_and(|p| p >= child.id) {
                return Err(ProofError::Fail("child_order"));
            }
            prior = Some(child.id);
            if matches!(child.state, ChildState::Active | ChildState::Unknown) {
                return Err(ProofError::Unproven("owned_child_active"));
            }
            if child.coverage == ChildCoverage::Unobserved {
                return Err(ProofError::Unproven("owned_child_unobserved"));
            }
        }
        let tail = &events[n + 1..];
        if let Some(count) = publication {
            if tail.len() < count * 2 {
                return Err(ProofError::Unproven("truncated_witness"));
            }
            for pair in tail[..count * 2].as_chunks::<2>().0 {
                if !matches!((&pair[0].event,&pair[1].event),
                    (Event::ItemStarted{item:a,kind:ItemKind::Adoption,..},Event::ItemCompleted{item:b,status:ItemStatus::Completed,body:Item::Adoption{..}}) if a==b)
                {
                    return Err(ProofError::Fail("forbidden_suffix"));
                }
            }
        } else {
            let tail = if matches!(
                tail.first().map(|e| &e.event),
                Some(Event::TurnEnded { .. })
            ) {
                &tail[1..]
            } else {
                tail
            };
            if tail.len() != 1 || !matches!(tail[0].event, Event::SessionClosed) {
                return Err(ProofError::Fail("forbidden_suffix"));
            }
        }
        Ok(())
    }
}

struct State {
    frozen: bool,
    inflight: u64,
    incomplete: bool,
    effects: Vec<Effect>,
    children: BTreeMap<ulid::Ulid, Child>,
}
/// One owner per invocation; try-lock is a bounded refusal, never a global queue.
/// This token constrains cooperating wrappers. Child observation/confinement and
/// truthful effect completeness require approved executable/process evidence.
#[derive(Clone)]
pub struct Owner(Arc<Mutex<State>>);
pub struct Admission {
    owner: Owner,
    finished: bool,
}
impl Default for Owner {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(State {
            frozen: false,
            inflight: 0,
            incomplete: false,
            effects: vec![],
            children: BTreeMap::new(),
        })))
    }
}
impl Owner {
    pub fn admit(&self) -> Proof<Admission> {
        let mut state = self
            .0
            .try_lock()
            .map_err(|_| ProofError::Unproven("owner_busy"))?;
        if state.frozen {
            return Err(ProofError::Fail("late_protected_effect"));
        }
        state.inflight = state
            .inflight
            .checked_add(1)
            .ok_or(ProofError::Unproven("effect_bound"))?;
        Ok(Admission {
            owner: self.clone(),
            finished: false,
        })
    }
    pub fn child(&self, child: Child) -> Proof<()> {
        let mut s = self
            .0
            .try_lock()
            .map_err(|_| ProofError::Unproven("owner_busy"))?;
        if s.frozen {
            return Err(ProofError::Fail("late_protected_effect"));
        }
        if !s.children.contains_key(&child.id)
            && !matches!(child.state, ChildState::Active | ChildState::Unknown)
        {
            return Err(ProofError::Fail("child_unregistered"));
        }
        if let Some(prior) = s.children.get(&child.id)
            && prior.state != ChildState::Active
        {
            return Err(ProofError::Fail("child_reopened"));
        }
        s.children.insert(child.id, child);
        Ok(())
    }
    /// Read-only snapshot for the shared E interpreter before reserving the seal.
    /// Outstanding admissions make a snapshot unavailable, just as they prevent freeze.
    pub(crate) fn effects(&self) -> Proof<Vec<Effect>> {
        let s = self
            .0
            .try_lock()
            .map_err(|_| ProofError::Unproven("owner_busy"))?;
        if s.frozen || s.inflight != 0 || s.incomplete {
            return Err(ProofError::Unproven("inflight_or_unobserved_effect"));
        }
        Ok(s.effects.clone())
    }
    pub fn freeze(
        &self,
        item: ItemId,
        session: SessionId,
        events: &[Envelope],
    ) -> Proof<(Seal, Vec<Effect>)> {
        let through = events.len() as u64;
        let digest = prefix(events, through)?;
        if !matches!(events.last().map(|e|&e.event),Some(Event::ItemStarted{item:i,kind:ItemKind::Receipt,..}) if *i==item)
        {
            return Err(ProofError::Fail("seal_item_mismatch"));
        }
        let mut s = self
            .0
            .try_lock()
            .map_err(|_| ProofError::Unproven("owner_busy"))?;
        if s.frozen {
            return Err(ProofError::Fail("already_frozen"));
        }
        if s.inflight != 0 || s.incomplete {
            return Err(ProofError::Unproven("inflight_or_unobserved_effect"));
        }
        for c in s.children.values() {
            if matches!(c.state, ChildState::Active | ChildState::Unknown) {
                return Err(ProofError::Unproven("owned_child_active"));
            }
            if c.coverage == ChildCoverage::Unobserved {
                return Err(ProofError::Unproven("owned_child_unobserved"));
            }
        }
        s.frozen = true;
        Ok((
            Seal {
                version: 1,
                receipt_item: item,
                own_prefix: Prefix {
                    session,
                    through_seq: through,
                    prefix_digest: digest,
                },
                effect_count: s.effects.len() as u64,
                children: s.children.values().cloned().collect(),
            },
            s.effects.clone(),
        ))
    }
}
impl Admission {
    pub fn finish(mut self, operation: Operation) -> Proof<()> {
        let mut s = self
            .owner
            .0
            .try_lock()
            .map_err(|_| ProofError::Unproven("owner_busy"))?;
        if s.frozen {
            return Err(ProofError::Fail("late_protected_effect"));
        }
        let i = s.effects.len() as u64;
        s.effects.push(Effect { i, operation });
        s.inflight -= 1;
        self.finished = true;
        Ok(())
    }
}
impl Drop for Admission {
    fn drop(&mut self) {
        if !self.finished {
            // Failure to acquire cannot silently enable a freeze: outstanding count remains nonzero.
            if let Ok(mut s) = self.owner.0.try_lock() {
                s.inflight -= 1;
                s.incomplete = true;
            }
        }
    }
}

/// Final own-custody exception, on the existing permitted blocking worker.
/// The current Store, reserved Receipt prefix and canonical body bind the target;
/// no source/sink path comes from receipt text. Publication callers must additionally
/// record this custody in their still-recording enclosing run. On an uncertain IO
/// result retain/query the already-known BlobRef, never redispatch blindly.
pub fn persist(
    store: &crate::store::Store,
    body: &super::receipt::Body,
    bytes: &[u8],
    blob: &BlobRef,
    redactor: &crate::redact::Set,
) -> Proof<BlobRef> {
    use std::{
        io::{Read, Write},
        os::unix::io::{AsRawFd, FromRawFd},
    };
    crate::runtime::assert_off_rt_worker();
    let seal = body
        .seal
        .as_ref()
        .ok_or(ProofError::Unproven("missing_seal"))?;
    if store.closed
        || store.session != body.run.session
        || store.session != seal.own_prefix.session
        || store.events.len() as u64 != seal.own_prefix.through_seq
        || !matches!(store.events.last().map(|e|&e.event),Some(Event::ItemStarted{item,kind:ItemKind::Receipt,..}) if *item==seal.receipt_item)
        || !matches!(store.events.first().map(|e|&e.event),Some(Event::SessionStarted{meta}) if meta.harness=="dream"&&meta.dream.is_some())
    {
        return Err(ProofError::Fail("writer_touched_source"));
    }
    if prefix(&store.events, seal.own_prefix.through_seq)? != seal.own_prefix.prefix_digest {
        return Err(ProofError::Fail("own_prefix_mismatch"));
    }
    let value = serde_json::to_value(body).map_err(|_| ProofError::Fail("malformed_receipt"))?;
    super::deny::value(&value, redactor).map_err(|_| ProofError::Fail("denied_receipt"))?;
    if body.encode()?.as_slice() != bytes
        || blob.sha256 != super::fingerprint::digest(bytes)
        || blob.len != bytes.len() as u64
        || blob.mime != super::receipt::MIME
    {
        return Err(ProofError::Fail("receipt_blob_binding"));
    }
    let dir = &store.blobs.dir;
    if dir.file_name().is_none_or(|n| n != "blobs")
        || dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|p| p.to_str())
            != Some(store.session.to_string().as_str())
    {
        return Err(ProofError::Fail("writer_touched_source"));
    }
    let root = dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or(ProofError::Fail("writer_touched_source"))?;
    use std::os::unix::fs::OpenOptionsExt;
    let root = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root)
        .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    let open_dir = |parent: &std::fs::File, name: &str| -> Proof<std::fs::File> {
        let name =
            std::ffi::CString::new(name).map_err(|_| ProofError::Fail("writer_touched_source"))?;
        let code = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if code < 0 {
            return Err(ProofError::Unproven("seal_custody_pending"));
        }
        Ok(unsafe { std::fs::File::from_raw_fd(code) })
    };
    let log_name = std::ffi::CString::new(format!("{}.jsonl", store.session)).unwrap();
    let fd = unsafe {
        libc::openat(
            root.as_raw_fd(),
            log_name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(ProofError::Unproven("seal_custody_pending"));
    }
    let log = unsafe { std::fs::File::from_raw_fd(fd) };
    let metadata = log
        .metadata()
        .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    if !metadata.is_file() {
        return Err(ProofError::Fail("own_prefix_mismatch"));
    }
    let mut log = log.take(
        metadata
            .len()
            .checked_add(1)
            .ok_or(ProofError::Unproven("seal_custody_pending"))?,
    );
    use sha2::Digest;
    let mut hash = sha2::Sha256::new();
    let mut buffer = [0; 65536];
    let mut count = 0u64;
    loop {
        let n = log
            .read(&mut buffer)
            .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        count += n as u64;
    }
    if count != metadata.len()
        || format!("sha256:{:x}", hash.finalize()) != seal.own_prefix.prefix_digest
    {
        return Err(ProofError::Fail("own_prefix_mismatch"));
    }
    let own = open_dir(&root, &store.session.to_string())?;
    let name = c"blobs";
    let code = unsafe { libc::mkdirat(own.as_raw_fd(), name.as_ptr(), 0o700) };
    if code < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST) {
        return Err(ProofError::Unproven("seal_custody_pending"));
    }
    if code == 0 {
        own.sync_all()
            .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    }
    let directory = open_dir(&own, "blobs")?;
    let target = std::ffi::CString::new(blob.sha256.as_str())
        .map_err(|_| ProofError::Fail("receipt_blob_binding"))?;
    let read_existing = || -> Proof<()> {
        let code = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                target.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if code < 0 {
            return Err(ProofError::Unproven("seal_custody_pending"));
        }
        let mut file = unsafe { std::fs::File::from_raw_fd(code) };
        if !file
            .metadata()
            .map_err(|_| ProofError::Unproven("seal_custody_pending"))?
            .is_file()
        {
            return Err(ProofError::Unproven("seal_blob_conflict"));
        }
        let mut actual = vec![];
        std::io::Read::by_ref(&mut file)
            .take(256 * 1024 + 1)
            .read_to_end(&mut actual)
            .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
        if actual != bytes {
            return Err(ProofError::Unproven("seal_blob_conflict"));
        }
        Ok(())
    };
    // No streaming/exclusive-open publication: only complete fsynced private bytes.
    let temp = std::ffi::CString::new(format!(".dream-tmp-{}", ulid::Ulid::new())).unwrap();
    let raw = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            temp.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if raw < 0 {
        return Err(ProofError::Unproven("seal_custody_pending"));
    }
    struct Temp<'a> {
        dir: &'a std::fs::File,
        name: std::ffi::CString,
    }
    impl Drop for Temp<'_> {
        fn drop(&mut self) {
            unsafe { libc::unlinkat(self.dir.as_raw_fd(), self.name.as_ptr(), 0) };
        }
    }
    let guard = Temp {
        dir: &directory,
        name: temp,
    };
    let mut file = unsafe { std::fs::File::from_raw_fd(raw) };
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    let linked = unsafe {
        libc::linkat(
            directory.as_raw_fd(),
            guard.name.as_ptr(),
            directory.as_raw_fd(),
            target.as_ptr(),
            0,
        )
    };
    if linked < 0 {
        if std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST) {
            return Err(ProofError::Unproven("seal_custody_pending"));
        }
        read_existing()?;
    }
    directory
        .sync_all()
        .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    drop(guard);
    directory
        .sync_all()
        .map_err(|_| ProofError::Unproven("seal_custody_pending"))?;
    Ok(blob.clone())
}
