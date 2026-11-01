// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{protocol::*, redact::Set, runtime::blocking};
use sha2::{Digest, Sha256};
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
                f.sync_all()?;
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
pub struct Store {
    file: File,
    pub session: SessionId,
    pub epoch: Epoch,
    pub events: Vec<Envelope>,
    pub blobs: Blobs,
    pub closed: bool,
    redactor: Set,
}
impl Store {
    pub async fn open(
        root: PathBuf,
        meta: SessionMeta,
        cause: Option<OpId>,
        redactor: Set,
    ) -> io::Result<Self> {
        let session = meta.session;
        let path = root.join(format!("{session}.jsonl"));
        let r = redactor.clone();
        let (file, events) = blocking(move || -> io::Result<_> {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut f = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .mode(0o600)
                .open(&path)?;
            // The log itself is the writer lease; no second durable store.
            if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err(io::Error::other("session already has a writer"));
            }
            use std::io::Read;
            let mut bytes = vec![];
            f.read_to_end(&mut bytes)?;
            let mut events = Vec::new();
            let mut valid = 0usize;
            for line in bytes.split_inclusive(|b| *b == b'\n') {
                if !line.ends_with(b"\n") {
                    break;
                }
                let e: Envelope = serde_json::from_slice(line)
                    .map_err(|_| io::Error::other("invalid log record"))?;
                if e.session != session
                    || e.seq != Some(Seq(events.len() as u64 + 1))
                    || !e.event.is_durable()
                {
                    return Err(io::Error::other("invalid durable log order"));
                }
                events.push(e);
                valid += line.len();
            }
            if valid < bytes.len() {
                f.set_len(valid as u64)?;
                f.flush()?;
            }
            Ok((f, events))
        })
        .await?;
        let closed = events
            .last()
            .is_some_and(|e| matches!(e.event, Event::SessionClosed));
        let epoch = if closed {
            events.last().unwrap().epoch
        } else {
            Epoch(
                events
                    .iter()
                    .map(|e| e.epoch.0)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("epoch exhausted"))?,
            )
        };
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
            redactor,
        };
        if store.events.is_empty() {
            store.append(Event::SessionStarted { meta }, cause).await?;
        } else if !store.closed {
            store.recover().await?;
        }
        Ok(store)
    }
    pub fn head(&self) -> Seq {
        Seq(self.events.len() as u64)
    }
    pub fn ephemeral(&self, event: Event) -> Envelope {
        Envelope {
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
        if self.closed || !event.is_durable() {
            return Err(io::Error::other("log is closed or event is ephemeral"));
        }
        let head = Seq(self
            .head()
            .0
            .checked_add(1)
            .ok_or_else(|| io::Error::other("seq exhausted"))?);
        let e = Envelope {
            v: PROTOCOL,
            session: self.session,
            epoch: self.epoch,
            seq: Some(head),
            at: head,
            ts_ms: now_ms(),
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
        blocking(move || f.sync_all()).await
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
                    item, kind, parent, ..
                } => {
                    open.insert(*item, (*kind, *parent));
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
                .find(|id| !open.values().any(|(_, p)| p.as_ref() == Some(id)))
                .ok_or_else(|| io::Error::other("cyclic item parents"))?;
            let (kind, _) = open.remove(&id).unwrap();
            self.append(
                Event::ItemCompleted {
                    item: id,
                    status: ItemStatus::Interrupted,
                    body: Item::interrupted(kind, id),
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
