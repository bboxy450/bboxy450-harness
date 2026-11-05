// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{protocol::*, redact::Set, runtime::blocking};
use sha2::{Digest, Sha256};
mod fork;
pub(crate) use fork::publish as publish_no_replace;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{self, Write},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct Blobs {
    pub dir: PathBuf,
    pub redactor: Set,
}
impl Blobs {
    pub async fn put(&self, bytes: Vec<u8>, mime: &str) -> io::Result<BlobRef> {
        let bytes = self.redactor.bytes(&bytes);
        let mime = mime.to_string();
        let dir = self.dir.clone();
        blocking(move || {
            std::fs::create_dir_all(&dir)?;
            let hash = format!("{:x}", Sha256::digest(&bytes));
            let path = dir.join(&hash);
            if !path.exists() {
                let mut f = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(path)?;
                f.write_all(&bytes)?;
                crate::runtime::sync_all(&f)?;
            }
            Ok(BlobRef {
                sha256: hash,
                len: bytes.len() as u64,
                mime,
            })
        })
        .await
    }
    pub async fn get(&self, blob: &BlobRef) -> io::Result<Vec<u8>> {
        if blob.sha256.len() != 64 || !blob.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::other("invalid blob digest"));
        }
        let path = self.dir.join(&blob.sha256);
        let b = blob.clone();
        blocking(move || {
            let bytes = std::fs::read(path)?;
            if bytes.len() as u64 != b.len || format!("{:x}", Sha256::digest(&bytes)) != b.sha256 {
                return Err(io::Error::other("corrupt blob"));
            }
            Ok(bytes)
        })
        .await
    }
}
fn read_epoch(dir: &Path) -> io::Result<u32> {
    match std::fs::read_to_string(dir.join("epoch")) {
        Ok(text) => text
            .trim()
            .parse::<u32>()
            .map_err(|_| io::Error::other("invalid epoch high-water mark")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e),
    }
}
pub(super) fn write_epoch(dir: &Path, epoch: u32) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let temp = dir.join(format!(".epoch-{}.tmp", OpId::new()));
    let result = (|| {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temp)?;
        writeln!(f, "{epoch}")?;
        crate::runtime::sync_all(&f)?;
        std::fs::rename(&temp, dir.join("epoch"))?;
        crate::runtime::sync_all(&File::open(dir)?)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
// bboxy450-harness; policy https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// d20261103136541: a single non-Clone guard owns the still-valid descriptor
// across the opening job/result and Store. Bare fsync/inherited clones do not.
struct WriterLease {
    file: File,
    released: bool,
    #[cfg(test)]
    unlock_override: Option<fn(&File) -> io::Result<()>>,
    #[cfg(test)]
    release_observed: Option<std::sync::mpsc::SyncSender<bool>>,
}
impl WriterLease {
    fn acquire(file: File) -> io::Result<Self> {
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            file,
            released: false,
            #[cfg(test)]
            unlock_override: None,
            #[cfg(test)]
            release_observed: None,
        })
    }
    fn release_with(&mut self, unlock: impl FnOnce(&File) -> io::Result<()>) -> io::Result<()> {
        if !self.released {
            unlock(&self.file)?;
            self.released = true;
        }
        Ok(())
    }
    fn release(&mut self) -> io::Result<()> {
        #[cfg(test)]
        if let Some(unlock) = self.unlock_override {
            return self.release_with(unlock);
        }
        self.release_with(|file| {
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) } == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        })
    }
}
impl std::ops::Deref for WriterLease {
    type Target = File;
    fn deref(&self) -> &File {
        &self.file
    }
}
impl std::ops::DerefMut for WriterLease {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}
impl Drop for WriterLease {
    fn drop(&mut self) {
        // Drop cannot certify an exceptional failure. The File stays valid
        // through this one best-effort attempt, then closes normally.
        let released = self.release();
        #[cfg(test)]
        if let Some(observer) = self.release_observed.take() {
            let _ = observer.send(released.is_ok());
        }
        #[cfg(not(test))]
        let _ = released;
    }
}
pub struct Store {
    file: WriterLease,
    pub session: SessionId,
    pub epoch: Epoch,
    pub events: Vec<Envelope>,
    pub blobs: Blobs,
    pub closed: bool,
    pub(crate) fresh: bool,
    redactor: Set,
}
impl Store {
    pub async fn open(
        root: PathBuf,
        meta: SessionMeta,
        cause: Option<OpId>,
        redactor: Set,
    ) -> io::Result<Self> {
        Self::open_impl(
            root,
            meta,
            cause,
            redactor,
            false,
            None,
            fork::Frontier::Quiescent,
        )
        .await
    }
    /// Internal queued-frontier extension, gated by the trusted pinned launch
    /// capability rather than an IR/profile/public StartSession boolean.
    pub(crate) async fn open_trial(
        root: PathBuf,
        meta: SessionMeta,
        cause: Option<OpId>,
        redactor: Set,
        capability: &crate::tools::TrialCapability,
    ) -> io::Result<Self> {
        let cap = capability.clone();
        let cwd = meta.cwd.clone();
        let sessions = root.clone();
        blocking(move || cap.validate_store(&cwd, &sessions)).await?;
        Self::open_impl(
            root,
            meta,
            cause,
            redactor,
            false,
            None,
            fork::Frontier::TrialQueued,
        )
        .await
    }
    /// Opt-in private native-import staging only. Existing opens keep their clocks.
    pub(crate) async fn open_import_staging(
        root: PathBuf,
        meta: SessionMeta,
        redactor: Set,
        initial_at: u64,
        ext: BTreeMap<String, serde_json::Value>,
    ) -> io::Result<Self> {
        Self::open_impl(
            root,
            meta,
            None,
            redactor,
            false,
            Some((initial_at, ext)),
            fork::Frontier::Quiescent,
        )
        .await
    }
    async fn open_impl(
        root: PathBuf,
        meta: SessionMeta,
        cause: Option<OpId>,
        redactor: Set,
        _restore_writer_durability: bool,
        initial: Option<(u64, BTreeMap<String, serde_json::Value>)>,
        frontier: fork::Frontier,
    ) -> io::Result<Self> {
        let session = meta.session;
        let path = root.join(format!("{session}.jsonl"));
        let r = redactor.clone();
        let fork_root = root.clone();
        let fork_meta = redactor.sanitize(&meta).map_err(io::Error::other)?;
        let (file, events, created_fork, queued_fork, epoch) =
            blocking(move || -> io::Result<_> {
                std::fs::create_dir_all(path.parent().unwrap())?;
                let created_fork = !path.exists() && fork_meta.fork.is_some();
                let queued_fork = if created_fork {
                    match &frontier {
                        fork::Frontier::Quiescent => {
                            fork::materialize(&fork_root, &path, &fork_meta, cause)?;
                            false
                        }
                        fork::Frontier::TrialQueued => fork::materialize_frontier(
                            &fork_root,
                            &path,
                            &fork_meta,
                            cause,
                            frontier.clone(),
                        )?,
                        #[cfg(test)]
                        fork::Frontier::TrialQueuedTest(_) => fork::materialize_frontier(
                            &fork_root,
                            &path,
                            &fork_meta,
                            cause,
                            frontier.clone(),
                        )?,
                    }
                } else {
                    false
                };
                let f = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .mode(0o600)
                    .open(&path)?;
                // The log itself remains the lease; guard it before any read/error.
                let mut f = WriterLease::acquire(f)
                    .map_err(|_| io::Error::other("session already has a writer"))?;
                use std::io::Read;
                let mut bytes = vec![];
                f.read_to_end(&mut bytes)?;
                let mut events = Vec::new();
                let mut valid = 0usize;
                for line in bytes.split_inclusive(|b| *b == b'\n') {
                    if !line.ends_with(b"\n") {
                        break;
                    }
                    let mut e =
                        Envelope::from_line(std::str::from_utf8(line).map_err(io::Error::other)?)
                            .map_err(|_| io::Error::other("invalid log record"))?;
                    if let Some(Envelope {
                        event: Event::SessionStarted { meta },
                        ..
                    }) = events.first()
                    {
                        e.project_into(meta);
                    }
                    if e.session != session || e.seq != Some(Seq(events.len() as u64 + 1)) {
                        return Err(io::Error::other("invalid durable log order"));
                    }
                    #[cfg(test)]
                    if _restore_writer_durability && !e.event.is_durable() {
                        return Err(io::Error::other(
                            "broken reader rejected unknown durable event",
                        ));
                    }
                    events.push(e);
                    valid += line.len();
                }
                if valid < bytes.len() {
                    f.set_len(valid as u64)?;
                    f.flush()?;
                }
                crate::runtime::sync_all(&f)?;
                let closed = events
                    .last()
                    .is_some_and(|e| matches!(e.event, Event::SessionClosed));
                let observed = events.iter().map(|e| e.epoch.0).max().unwrap_or(0);
                let epoch = if closed {
                    observed
                } else if created_fork {
                    if queued_fork { 2 } else { 1 }
                } else {
                    read_epoch(&fork_root.join(session.to_string()))?
                        .max(observed)
                        .checked_add(1)
                        .ok_or_else(|| io::Error::other("epoch exhausted"))?
                };
                if !closed && (!created_fork || queued_fork) {
                    write_epoch(&fork_root.join(session.to_string()), epoch)?;
                }
                #[cfg(test)]
                if let fork::Frontier::TrialQueuedTest(boundary) = &frontier {
                    f.release_observed = Some(boundary.retired.clone());
                    let _ = boundary.reached.send(());
                    let mut released = boundary.release.0.lock().unwrap();
                    while !*released {
                        released = boundary.release.1.wait(released).unwrap();
                    }
                    drop(released);
                    if boundary.fail {
                        return Err(io::Error::other(
                            "injected trial opening error after epoch reservation",
                        ));
                    }
                }
                Ok((f, events, created_fork, queued_fork, Epoch(epoch)))
            })
            .await?;
        let closed = events
            .last()
            .is_some_and(|e| matches!(e.event, Event::SessionClosed));
        let fresh = events.is_empty() || created_fork;
        let blobs = Blobs {
            dir: root.join(session.to_string()).join("blobs"),
            redactor: r,
        };
        let mut store = Self {
            file,
            session,
            epoch,
            events,
            blobs,
            closed,
            fresh,
            redactor,
        };
        if store.events.is_empty() {
            if let Some((at, ext)) = initial {
                store
                    .append_projected_at(Event::SessionStarted { meta }, cause, at, None, ext)
                    .await?;
            } else {
                store.append(Event::SessionStarted { meta }, cause).await?;
            }
        } else if !store.closed && (!created_fork || queued_fork) {
            store.recover().await?;
        }
        Ok(store)
    }
    /// Read-only replay never takes the writer lease, recovers, or changes epochs.
    pub async fn replay_closed(root: PathBuf, session: SessionId) -> io::Result<Vec<Envelope>> {
        blocking(move || {
            let bytes = std::fs::read(log_path(&root, session))?;
            if !bytes.ends_with(b"\n") {
                return Err(io::Error::other("incomplete closed log"));
            }
            let mut events = bytes
                .split_inclusive(|b| *b == b'\n')
                .map(|line| {
                    Envelope::from_line(std::str::from_utf8(line).map_err(io::Error::other)?)
                        .map_err(io::Error::other)
                })
                .collect::<Result<Vec<Envelope>, _>>()
                .map_err(|_| io::Error::other("invalid log record"))?;
            let meta = match events.first() {
                Some(Envelope {
                    event: Event::SessionStarted { meta },
                    ..
                }) => meta.clone(),
                _ => return Err(io::Error::other("missing session header")),
            };
            for e in &mut events {
                e.project_into(&meta);
            }
            if events
                .iter()
                .any(|e| e.session != session || e.seq.is_none())
                || !events
                    .last()
                    .is_some_and(|e| matches!(e.event, Event::SessionClosed))
            {
                return Err(io::Error::other("log is not closed"));
            }
            crate::protocol::laws::check_transcript(&events).map_err(io::Error::other)?;
            Ok(events)
        })
        .await
    }
    pub fn head(&self) -> Seq {
        Seq(self.events.len() as u64)
    }
    pub fn ephemeral(&self, event: Event) -> Envelope {
        Envelope {
            raw: None,
            ext: Default::default(),
            native: None,
            v: PROTOCOL,
            session: self.session,
            epoch: self.epoch,
            seq: None,
            at: self.head(),
            ts_ms: now_ms(),
            cause: None,
            event,
        }
    }
    pub async fn append(&mut self, event: Event, cause: Option<OpId>) -> io::Result<Envelope> {
        self.append_at(event, cause, now_ms()).await
    }
    /// Append with the time the event happened (an imported native record keeps its own clock).
    pub async fn append_at(
        &mut self,
        event: Event,
        cause: Option<OpId>,
        ts_ms: u64,
    ) -> io::Result<Envelope> {
        self.append_projected_at(event, cause, ts_ms, None, Default::default())
            .await
    }
    /// Persist a projector's provenance and unconsumed fields under the normal
    /// redaction, envelope-size and durability rules.
    pub async fn append_projected_at(
        &mut self,
        event: Event,
        cause: Option<OpId>,
        ts_ms: u64,
        native: Option<Native>,
        ext: std::collections::BTreeMap<String, serde_json::Value>,
    ) -> io::Result<Envelope> {
        if self.closed || !event.is_durable() {
            return Err(io::Error::other("log is closed or event is ephemeral"));
        }
        let head = Seq(self
            .head()
            .0
            .checked_add(1)
            .ok_or_else(|| io::Error::other("seq exhausted"))?);
        let e = Envelope {
            raw: None,
            ext,
            native,
            v: PROTOCOL,
            session: self.session,
            epoch: self.epoch,
            seq: Some(head),
            at: head,
            ts_ms,
            cause,
            event,
        };
        let e = self.redactor.sanitize(&e).map_err(io::Error::other)?;
        let mut bytes = serde_json::to_vec(&e)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(io::Error::other(
                "envelope exceeds 256 KiB; spill content to a blob",
            ));
        }
        self.file.write_all(&bytes)?;
        self.closed = matches!(e.event, Event::SessionClosed);
        self.events.push(e.clone());
        if matches!(
            e.event,
            Event::TurnEnded { .. }
                | Event::SessionClosed
                | Event::ItemStarted {
                    kind: ItemKind::Approval,
                    ..
                }
                | Event::ItemCompleted {
                    body: Item::Approval { .. },
                    ..
                }
        ) {
            self.sync().await?;
        }
        Ok(e)
    }
    pub async fn sync(&self) -> io::Result<()> {
        let f = self.file.try_clone()?;
        blocking(move || crate::runtime::sync_all(&f)).await
    }
    async fn recover(&mut self) -> io::Result<()> {
        let prior_epoch = self.events.last().unwrap().epoch;
        let mut open = BTreeMap::new();
        let mut turns = BTreeSet::new();
        for e in &self.events {
            match &e.event {
                Event::TurnQueued { turn } => {
                    turns.insert(*turn);
                }
                Event::TurnEnded { turn, .. } => {
                    turns.remove(turn);
                }
                Event::ItemStarted {
                    item,
                    kind,
                    parent,
                    head,
                    ..
                } => {
                    open.insert(*item, (kind.clone(), *parent, head.clone()));
                }
                Event::ItemCompleted { item, .. } => {
                    open.remove(item);
                }
                _ => {}
            }
        }
        let closed_items = open.len() as u32;
        let closed_turns = turns.len() as u32;
        while !open.is_empty() {
            let id = *open
                .keys()
                .find(|id| !open.values().any(|(_, p, _)| p.as_ref() == Some(id)))
                .ok_or_else(|| io::Error::other("cyclic item parents"))?;
            let (kind, _, head) = open.remove(&id).unwrap();
            self.append(
                Event::ItemCompleted {
                    item: id,
                    status: ItemStatus::Interrupted,
                    body: Item::interrupted_from_head(kind, id, head.as_ref()),
                },
                None,
            )
            .await?;
        }
        for turn in turns {
            self.append(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::Interrupted,
                    usage: None,
                },
                None,
            )
            .await?;
        }
        self.append(
            Event::SessionRecovered {
                prior_epoch,
                closed_items,
                closed_turns,
            },
            None,
        )
        .await?;
        self.sync().await
    }
}
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn log_path(root: &Path, session: SessionId) -> PathBuf {
    root.join(format!("{session}.jsonl"))
}
impl Blobs {
    pub async fn ingest(&self, path: PathBuf, mime: &str) -> io::Result<BlobRef> {
        let dir = self.dir.clone();
        let mime = mime.to_owned();
        blocking(move || {
            use std::io::Read;
            let mut f = File::open(&path)?;
            let mut hash = Sha256::new();
            let mut len = 0;
            let mut buf = [0; 16 * 1024];
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hash.update(&buf[..n]);
                len += n as u64;
            }
            let sha256 = format!("{:x}", hash.finalize());
            std::fs::create_dir_all(&dir)?;
            let destination = dir.join(&sha256);
            if !destination.exists() {
                std::fs::rename(&path, &destination)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            Ok(BlobRef { sha256, len, mime })
        })
        .await
    }
}

#[cfg(test)]
mod durability_controls {
    use super::*;
    #[tokio::test]
    async fn writer_durability_on_read_refuses_a_forward_compatible_log() {
        let temp = tempfile::tempdir().unwrap();
        let meta = SessionMeta {
            vcs: None,
            dream: None,
            source: None,
            session: SessionId::new(),
            cwd: temp.path().into(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "test".into(),
            created_ms: 0,
        };
        let root = temp.path().join("sessions");
        let store = Store::open(root.clone(), meta.clone(), None, Default::default())
            .await
            .unwrap();
        drop(store);
        let future = format!(
            "{{\"v\":{{\"major\":1,\"minor\":99}},\"session\":\"{}\",\"epoch\":1,\"seq\":2,\"at\":2,\"ts_ms\":0,\"cause\":null,\"event\":{{\"type\":\"future_durable\",\"keep\":true}}}}\n",
            meta.session
        );
        OpenOptions::new()
            .append(true)
            .open(log_path(&root, meta.session))
            .unwrap()
            .write_all(future.as_bytes())
            .unwrap();
        let broken = Store::open_impl(
            root.clone(),
            meta.clone(),
            None,
            Default::default(),
            true,
            None,
            fork::Frontier::Quiescent,
        )
        .await;
        assert!(
            broken.is_err(),
            "broken old reader unexpectedly accepted future data"
        );
        let mut good = Store::open(root.clone(), meta, None, Default::default())
            .await
            .unwrap();
        assert_eq!(good.events[1].wire_bytes().unwrap(), future.as_bytes());
        good.append(Event::SessionClosed, None).await.unwrap();
        let session = good.session;
        drop(good);
        let replay = Store::replay_closed(root, session).await.unwrap();
        assert_eq!(replay[1].wire_bytes().unwrap(), future.as_bytes());
        println!(
            "broken writer-side durability reader: open rejected; seq reader opened and replayed"
        );
    }
}

#[cfg(test)]
mod lease_tests {
    use super::*;
    use std::sync::{Arc, Condvar, Mutex};
    fn open(path: &Path) -> File {
        OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .mode(0o600)
            .open(path)
            .unwrap()
    }
    fn lease(path: &Path) -> WriterLease {
        WriterLease::acquire(open(path)).unwrap()
    }
    fn busy(path: &Path) {
        let error = WriterLease::acquire(open(path))
            .err()
            .expect("another writer entered a live lease");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    }
    #[test]
    fn checked_unlock_failure_and_stale_released_guard_preserve_exact_generation() {
        let own = tempfile::tempdir().unwrap();
        let path = own.path().join("lease");
        let mut old = lease(&path);
        let duplicate = old.try_clone().unwrap();
        let fd = old.file.as_raw_fd();
        let error = old
            .release_with(|file| {
                assert_eq!(file.as_raw_fd(), fd);
                Err(io::Error::from_raw_os_error(libc::EIO))
            })
            .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::EIO));
        assert!(!old.released);
        busy(&path);
        old.release().unwrap();
        assert!(old.released);
        let replacement = lease(&path); // old bare duplicate is deliberately STILL open
        old.release_with(|_| panic!("already-released guard made a stale syscall"))
            .unwrap();
        drop(old);
        busy(&path);
        drop(duplicate);
        busy(&path);
        drop(replacement);
        drop(lease(&path));
    }
    #[test]
    fn exceptional_drop_unlock_failure_is_nonpanicking_and_unavailable() {
        fn fail(_: &File) -> io::Result<()> {
            Err(io::Error::from_raw_os_error(libc::EIO))
        }
        let own = tempfile::tempdir().unwrap();
        let path = own.path().join("lease");
        let mut guard = lease(&path);
        let duplicate = guard.try_clone().unwrap();
        guard.unlock_override = Some(fail);
        assert_eq!(guard.release().unwrap_err().raw_os_error(), Some(libc::EIO));
        assert!(!guard.released);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(guard))).unwrap();
        busy(&path); // failure is not a successful release claim
        drop(duplicate);
        drop(lease(&path));
    }
    #[tokio::test]
    async fn bare_fsync_clone_and_session_closed_keep_the_live_store_exclusive() {
        let own = tempfile::tempdir().unwrap();
        let root = own.path().join("sessions");
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: own.path().into(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "engine".into(),
            created_ms: 0,
            vcs: None,
            dream: None,
            source: None,
        };
        let mut store = Store::open(root.clone(), meta.clone(), None, Set::default())
            .await
            .unwrap();
        let duplicate = store.file.try_clone().unwrap();
        duplicate.sync_all().unwrap();
        drop(duplicate);
        store.sync().await.unwrap();
        assert!(
            Store::open(root.clone(), meta.clone(), None, Set::default())
                .await
                .is_err()
        );
        store.append(Event::SessionClosed, None).await.unwrap();
        assert!(
            Store::open(root.clone(), meta.clone(), None, Set::default())
                .await
                .is_err()
        );
        let duplicate = store.file.try_clone().unwrap();
        let session = meta.session;
        let before = std::fs::read(log_path(&root, session)).unwrap();
        drop(store);
        let replacement = Store::open(root.clone(), meta.clone(), None, Set::default())
            .await
            .unwrap();
        drop(duplicate);
        assert!(
            Store::open(root.clone(), meta.clone(), None, Set::default())
                .await
                .is_err()
        );
        drop(replacement);
        let again = Store::open(root.clone(), meta, None, Set::default())
            .await
            .unwrap();
        drop(again);
        assert_eq!(std::fs::read(log_path(&root, session)).unwrap(), before);
    }
    #[tokio::test]
    async fn post_acquisition_error_and_abandoned_job_output_destroy_the_guard() {
        let own = tempfile::tempdir().unwrap();
        let path = own.path().join("constructor-error");
        let cloned_path = path.clone();
        let (send, receive) = tokio::sync::oneshot::channel();
        let result: io::Result<()> = blocking(move || {
            let owner = lease(&cloned_path);
            let _ = send.send(owner.try_clone().unwrap());
            Err(io::Error::from_raw_os_error(libc::EINVAL))
        })
        .await;
        assert_eq!(result.unwrap_err().raw_os_error(), Some(libc::EINVAL));
        let duplicate = receive.await.unwrap();
        drop(lease(&path));
        drop(duplicate);
        type Gate = Arc<(Mutex<bool>, Condvar)>;
        struct Release(Gate);
        impl Drop for Release {
            fn drop(&mut self) {
                *self.0.0.lock().unwrap() = true;
                self.0.1.notify_all();
            }
        }
        struct AfterGuard(Option<tokio::sync::oneshot::Sender<()>>);
        impl Drop for AfterGuard {
            fn drop(&mut self) {
                if let Some(send) = self.0.take() {
                    let _ = send.send(());
                }
            }
        }
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let release = Release(gate.clone());
        let path = own.path().join("abandoned-job");
        let cloned_path = path.clone();
        let (send, receive) = tokio::sync::oneshot::channel();
        let (done, dropped) = tokio::sync::oneshot::channel();
        let mut opening = Box::pin(blocking(move || {
            let mut owner = lease(&cloned_path);
            let _ = send.send(owner.try_clone().unwrap());
            let mut open = gate.0.lock().unwrap();
            while !*open {
                open = gate.1.wait(open).unwrap();
            }
            drop(open);
            owner
                .write_all(b"job retained its lease through recovery mutation")
                .unwrap();
            (owner, AfterGuard(Some(done))) // fields drop in order, owner BEFORE observation
        }));
        assert!(futures_util::poll!(opening.as_mut()).is_pending());
        let mut receive = receive;
        let mut completed = false;
        let acquisition=tokio::time::timeout(std::time::Duration::from_secs(3),async {
            tokio::select! {biased;
                ack=&mut receive=>ack.map_err(|_|"acquisition acknowledgement closed"),
                returned=&mut opening=>{completed=true;drop(returned);Err("opening completed before acquisition acknowledgement")},
            }
        }).await;
        let duplicate = match acquisition {
            Ok(Ok(duplicate)) => duplicate,
            failure => {
                // Release this exact fixture gate, then join/drop the SAME
                // opening output before a failing assertion. No detached job.
                drop(release);
                if !completed {
                    drop(opening.await);
                }
                let _ = dropped.await;
                panic!("lease acquisition failed after fixture cleanup: {failure:?}");
            }
        };
        drop(opening);
        busy(&path); // caller abandonment does not end a mutating job's ownership
        drop(release);
        tokio::time::timeout(std::time::Duration::from_secs(3), dropped)
            .await
            .unwrap()
            .unwrap();
        let replacement = lease(&path);
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"job retained its lease through recovery mutation"
        );
        drop(duplicate);
        busy(&path);
        drop(replacement);
    }
    #[tokio::test]
    async fn actual_store_parse_error_releases_its_acquired_lease() {
        let own = tempfile::tempdir().unwrap();
        let root = own.path().join("sessions");
        std::fs::create_dir(&root).unwrap();
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: own.path().into(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "engine".into(),
            created_ms: 0,
            vcs: None,
            dream: None,
            source: None,
        };
        let path = log_path(&root, meta.session);
        std::fs::write(&path, b"synthetic invalid JSON\n").unwrap();
        assert!(Store::open(root, meta, None, Set::default()).await.is_err());
        drop(lease(&path));
        assert_eq!(std::fs::read(path).unwrap(), b"synthetic invalid JSON\n");
    }
}

#[cfg(test)]
mod trial_frontier_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    async fn fixture() -> (
        tempfile::TempDir,
        crate::tools::TrialCapability,
        SessionMeta,
        Vec<TurnId>,
        Seq,
        BlobRef,
    ) {
        let temp = tempfile::tempdir().unwrap();
        for name in ["repo", "sessions", "profiles", "home"] {
            std::fs::create_dir(temp.path().join(name)).unwrap();
            std::fs::set_permissions(
                temp.path().join(name),
                std::fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let root = temp.path().to_path_buf();
        let capability = blocking(move || {
            crate::tools::TrialCapability::mint(
                root.join("repo"),
                root.join("sessions"),
                root.join("profiles"),
                root.join("home"),
            )
        })
        .await
        .unwrap();
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: temp.path().join("repo"),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: 0,
            vcs: None,
            dream: None,
            source: None,
        };
        let mut parent = Store::open(
            temp.path().join("sessions"),
            meta.clone(),
            None,
            Set::default(),
        )
        .await
        .unwrap();
        parent
            .append(
                Event::ProfileChanged {
                    profile: "scripted".into(),
                    model: "scripted".into(),
                },
                None,
            )
            .await
            .unwrap();
        let blob = parent
            .blobs
            .put(b"private inherited blob".to_vec(), "image/png")
            .await
            .unwrap();
        let item = ItemId::new();
        parent
            .append(
                Event::ItemStarted {
                    turn: None,
                    item,
                    kind: ItemKind::Image,
                    parent: None,
                    head: None,
                },
                None,
            )
            .await
            .unwrap();
        parent
            .append(
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::Image {
                        mime: "image/png".into(),
                        blob: blob.clone(),
                    },
                },
                None,
            )
            .await
            .unwrap();
        drop(parent);
        let mut bytes =
            std::fs::read(log_path(&temp.path().join("sessions"), meta.session)).unwrap();
        // Retained future-writer fixture, deliberately bypassing the current
        // authored writer. Unknown event/ext fields and CRLF must pass through.
        let future = serde_json::json!({"v":PROTOCOL,"session":meta.session,"epoch":1,"seq":5,"at":5,"ts_ms":0,"provider_future":{"kept":[1,2]},"event":{"type":"future_marker","opaque":{"kept":true}}});
        bytes.extend(format!("  {future}\r\n").as_bytes());
        let turns = vec![TurnId::new(), TurnId::new()];
        for (i, turn) in turns.iter().enumerate() {
            let seq = Seq(6 + i as u64);
            let event = Envelope {
                raw: None,
                ext: Default::default(),
                native: None,
                v: PROTOCOL,
                session: meta.session,
                epoch: Epoch(1),
                seq: Some(seq),
                at: seq,
                ts_ms: 0,
                cause: None,
                event: Event::TurnQueued { turn: *turn },
            };
            bytes.extend(event.wire_bytes().unwrap());
        }
        std::fs::write(log_path(&temp.path().join("sessions"), meta.session), bytes).unwrap();
        (temp, capability, meta, turns, Seq(7), blob)
    }
    fn child(meta: &SessionMeta, cut: Seq) -> SessionMeta {
        let mut child = meta.clone();
        child.session = SessionId::new();
        child.fork = Some((meta.session, cut));
        child
    }
    #[tokio::test]
    async fn queued_trial_exact_future_prefix_blobs_epoch_and_parent_independence() {
        let (temp, capability, parent, turns, cut, blob) = fixture().await;
        let root = temp.path().join("sessions");
        let source = std::fs::read(log_path(&root, parent.session)).unwrap();
        let meta = child(&parent, cut);
        let unpolled = Box::pin(Store::open_trial(
            root.clone(),
            meta.clone(),
            None,
            Set::default(),
            &capability,
        ));
        drop(unpolled);
        assert!(!log_path(&root, meta.session).exists());
        let store = Store::open_trial(
            root.clone(),
            meta.clone(),
            None,
            Set::default(),
            &capability,
        )
        .await
        .unwrap();
        assert_eq!(store.epoch, Epoch(2));
        assert_eq!(store.head(), Seq(10));
        assert!(matches!(
            store.events.last().unwrap().event,
            Event::SessionRecovered {
                prior_epoch: Epoch(1),
                closed_items: 0,
                closed_turns: 2
            }
        ));
        for turn in &turns {
            assert_eq!(store.events.iter().filter(|e|matches!(e.event,Event::TurnEnded{turn:t,reason:StopReason::Interrupted,usage:None} if t==*turn)).count(),1);
        }
        let copied = std::fs::read(log_path(&root, meta.session)).unwrap();
        let original = source
            .split_inclusive(|b| *b == b'\n')
            .skip(1)
            .collect::<Vec<_>>();
        let inherited = copied
            .split_inclusive(|b| *b == b'\n')
            .skip(1)
            .take(6)
            .collect::<Vec<_>>();
        assert_eq!(original, inherited);
        assert_eq!(
            source,
            std::fs::read(log_path(&root, parent.session)).unwrap()
        );
        let target = store.blobs.dir.join(&blob.sha256);
        std::fs::remove_file(
            root.join(parent.session.to_string())
                .join("blobs")
                .join(&blob.sha256),
        )
        .unwrap();
        std::fs::remove_file(log_path(&root, parent.session)).unwrap();
        assert_eq!(std::fs::read(target).unwrap(), b"private inherited blob");
        drop(store);
        let reopened = Store::open(root.clone(), meta.clone(), None, Set::default())
            .await
            .unwrap();
        assert_eq!(reopened.epoch, Epoch(3));
        assert!(matches!(
            reopened.events.last().unwrap().event,
            Event::SessionRecovered {
                closed_items: 0,
                closed_turns: 0,
                ..
            }
        ));
        for turn in turns {
            assert_eq!(
                reopened
                    .events
                    .iter()
                    .filter(|e| matches!(e.event,Event::TurnEnded{turn:t,..} if t==turn))
                    .count(),
                1
            );
        }
    }
    #[tokio::test]
    async fn queued_trial_crash_after_prefix_and_partial_suffix_recovers_once() {
        for partial in [false, true] {
            let (temp, _capability, parent, turns, cut, _blob) = fixture().await;
            let root = temp.path().join("sessions");
            let meta = child(&parent, cut);
            let path = log_path(&root, meta.session);
            let m = meta.clone();
            let r = root.clone();
            blocking(move || {
                fork::materialize_frontier(&r, &path, &m, None, fork::Frontier::TrialQueued)
            })
            .await
            .unwrap();
            if partial {
                let r = root.clone();
                let m = meta.clone();
                let turn = turns[0];
                blocking(move || {
                    use std::io::Write;
                    write_epoch(&r.join(m.session.to_string()), 2).unwrap();
                    let seq = Seq(cut.0 + 1);
                    let event = Envelope {
                        raw: None,
                        ext: Default::default(),
                        native: None,
                        v: PROTOCOL,
                        session: m.session,
                        epoch: Epoch(2),
                        seq: Some(seq),
                        at: seq,
                        ts_ms: 0,
                        cause: None,
                        event: Event::TurnEnded {
                            turn,
                            reason: StopReason::Interrupted,
                            usage: None,
                        },
                    };
                    let mut file = OpenOptions::new()
                        .append(true)
                        .open(log_path(&r, m.session))
                        .unwrap();
                    file.write_all(&event.wire_bytes().unwrap()).unwrap();
                    crate::runtime::sync_all(&file).unwrap();
                })
                .await;
            }
            let store = Store::open(root, meta, None, Set::default()).await.unwrap();
            assert_eq!(store.epoch, Epoch(if partial { 3 } else { 2 }));
            assert!(
                matches!(store.events.last().unwrap().event,Event::SessionRecovered{closed_items:0,closed_turns,..} if closed_turns==if partial{1}else{2})
            );
            for turn in turns {
                assert_eq!(
                    store
                        .events
                        .iter()
                        .filter(|e| matches!(e.event,Event::TurnEnded{turn:t,..} if t==turn))
                        .count(),
                    1
                );
            }
        }
    }
    #[tokio::test]
    async fn queued_trial_cancelled_or_error_open_keeps_lease_until_real_drop_and_recovers() {
        // This test deliberately holds a native blocking permit at its epoch
        // boundary. Give that barrier its own process, as the observed-spawn
        // controls do; never starve unrelated lease tests' unchanged deadlines.
        if std::env::var_os("DREAM_WP7_FRONTIER_CHILD").is_none() {
            let output=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","store::trial_frontier_tests::queued_trial_cancelled_or_error_open_keeps_lease_until_real_drop_and_recovers","--nocapture"])
                .env("DREAM_WP7_FRONTIER_CHILD","1").output().unwrap();
            assert!(
                output.status.success(),
                "private queued-frontier control failed: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        for fail in [false, true] {
            let (temp, capability, parent, turns, cut, _) = fixture().await;
            let root = temp.path().join("sessions");
            let meta = child(&parent, cut);
            let cap = capability.clone();
            let cwd = meta.cwd.clone();
            let sessions = root.clone();
            blocking(move || cap.validate_store(&cwd, &sessions))
                .await
                .unwrap();
            let (retired, receiver) = std::sync::mpsc::sync_channel(1);
            let (reached, arrival) = std::sync::mpsc::sync_channel(1);
            let boundary = std::sync::Arc::new(TrialOpenBoundary {
                reached,
                release: (std::sync::Mutex::new(false), std::sync::Condvar::new()),
                retired,
                fail,
            });
            let release = TrialBoundaryRelease(boundary.clone());
            let b = boundary.clone();
            let r = root.clone();
            let m = meta.clone();
            let mut task = Some(bbt_rt::handle().spawn(async move {
                Store::open_impl(
                    r,
                    m,
                    None,
                    Set::default(),
                    false,
                    None,
                    fork::Frontier::TrialQueuedTest(b),
                )
                .await
            }));
            blocking(move || arrival.recv_timeout(std::time::Duration::from_secs(3)))
                .await
                .unwrap();
            assert_eq!(read_epoch(&root.join(meta.session.to_string())).unwrap(), 2);
            assert!(
                Store::open(root.clone(), meta.clone(), None, Set::default())
                    .await
                    .is_err(),
                "opening job still owns the lease"
            );
            if !fail {
                let task = task.take().unwrap();
                task.abort();
                assert!(matches!(task.await,Err(e) if e.is_cancelled()));
            }
            drop(release);
            if fail {
                assert!(task.take().unwrap().await.unwrap().is_err());
            }
            assert!(
                blocking(move || receiver.recv_timeout(std::time::Duration::from_secs(3)))
                    .await
                    .unwrap(),
                "actual WriterLease::release acknowledged"
            );
            let store = Store::open(root, meta, None, Set::default()).await.unwrap();
            assert_eq!(store.epoch, Epoch(3));
            for turn in turns {
                assert_eq!(
                    store
                        .events
                        .iter()
                        .filter(|e| matches!(e.event,Event::TurnEnded{turn:t,..} if t==turn))
                        .count(),
                    1
                );
            }
            assert!(matches!(
                store.events.last().unwrap().event,
                Event::SessionRecovered {
                    closed_turns: 2,
                    ..
                }
            ));
        }
    }
    #[tokio::test]
    async fn queued_trial_refuses_active_open_malformed_and_missing_blob_before_child() {
        for mode in 0..6 {
            let (temp, capability, parent, turns, cut, blob) = fixture().await;
            let root = temp.path().join("sessions");
            let path = log_path(&root, parent.session);
            let mut bytes = std::fs::read(&path).unwrap();
            let meta = child(&parent, cut);
            let mut target_cut = cut;
            if mode == 5 {
                std::fs::remove_file(
                    root.join(parent.session.to_string())
                        .join("blobs")
                        .join(&blob.sha256),
                )
                .unwrap();
            } else {
                let event = match mode {
                    0 => Event::TurnStarted {
                        turn: turns[0],
                        model: None,
                        origin: None,
                    },
                    1 => Event::ItemStarted {
                        turn: None,
                        item: ItemId::new(),
                        kind: ItemKind::Notice,
                        parent: None,
                        head: None,
                    },
                    2 => Event::TurnQueued { turn: turns[0] },
                    3 => Event::SessionClosed,
                    4 => Event::ProfileChanged {
                        profile: "scripted".into(),
                        model: "scripted".into(),
                    },
                    _ => unreachable!(),
                };
                let seq = Seq(cut.0 + if mode == 4 { 2 } else { 1 });
                let event = Envelope {
                    raw: None,
                    ext: Default::default(),
                    native: None,
                    v: PROTOCOL,
                    session: parent.session,
                    epoch: Epoch(1),
                    seq: Some(seq),
                    at: seq,
                    ts_ms: 0,
                    cause: None,
                    event,
                };
                bytes.extend(event.wire_bytes().unwrap());
                std::fs::write(path, bytes).unwrap();
                target_cut = Seq(cut.0 + 1);
            }
            let mut meta = meta;
            meta.fork = Some((parent.session, target_cut));
            assert!(
                Store::open_trial(
                    root.clone(),
                    meta.clone(),
                    None,
                    Set::default(),
                    &capability
                )
                .await
                .is_err(),
                "mode {mode}"
            );
            assert!(!log_path(&root, meta.session).exists(), "mode {mode}");
        }
    }
}

// Per-invocation deterministic test boundary, never a global hook or product API.
#[cfg(test)]
struct TrialOpenBoundary {
    reached: std::sync::mpsc::SyncSender<()>,
    release: (std::sync::Mutex<bool>, std::sync::Condvar),
    retired: std::sync::mpsc::SyncSender<bool>,
    fail: bool,
}

#[cfg(test)]
struct TrialBoundaryRelease(std::sync::Arc<TrialOpenBoundary>);
#[cfg(test)]
impl Drop for TrialBoundaryRelease {
    fn drop(&mut self) {
        *self.0.release.0.lock().unwrap() = true;
        self.0.release.1.notify_all();
    }
}
