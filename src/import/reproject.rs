// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! A derived log keeps its immutable source prefix and appends newly available
//! projections. Raw source lines are forwarded, never reserialized.
use super::{Native, Projected, Projector, native_identity};
use crate::{
    protocol::*,
    redact::Set,
    store::{Blobs, Store},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Path, PathBuf},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Load CLI redaction before deriving a log; callers with an explicit Set use run.
pub async fn run_cli(source: PathBuf, out: PathBuf) -> io::Result<PathBuf> {
    let redactor = Set::default();
    crate::capability::register(&redactor).await;
    run(source, out, redactor).await
}

pub async fn run(source: PathBuf, out: PathBuf, redactor: Set) -> io::Result<PathBuf> {
    let file = tokio::fs::File::open(&source).await?;
    let len = file.metadata().await?.len();
    let mut bytes = vec![];
    file.take(len).read_to_end(&mut bytes).await?;
    let records = bbt_rt::handle()
        .spawn_blocking(move || -> io::Result<Vec<Envelope>> {
            bytes
                .split_inclusive(|b| *b == b'\n')
                .filter(|line| line.ends_with(b"\n"))
                .map(|line| {
                    Envelope::from_line(std::str::from_utf8(line).map_err(io::Error::other)?)
                        .map_err(io::Error::other)
                })
                .collect()
        })
        .await
        .map_err(io::Error::other)??;
    let first = records
        .first()
        .ok_or_else(|| io::Error::other("empty source log"))?;
    let Event::SessionStarted { meta: original } = &first.event else {
        return Err(io::Error::other("source has no session header"));
    };
    let prefix: Vec<_> = records
        .iter()
        .take_while(|e| !matches!(e.event, Event::SessionClosed))
        .collect();
    let at = prefix
        .last()
        .and_then(|e| e.seq)
        .ok_or_else(|| io::Error::other("source has no durable prefix"))?;
    let mut meta = original.clone();
    meta.session = SessionId::new();
    meta.protocol = PROTOCOL;
    meta.fork = Some((original.session, at));
    meta.created_ms = crate::store::now_ms();
    meta.harness = env!("CARGO_PKG_VERSION").into();
    if let Some(source) = &mut meta.source {
        source.parser_revision = Some(source.parser_revision.unwrap_or(0).saturating_add(1));
    }
    let mut header = first.clone();
    header.raw = None;
    header.native = None;
    header.ts_ms = meta.created_ms;
    header.session = meta.session;
    header.epoch = Epoch(1);
    header.event = Event::SessionStarted { meta: meta.clone() };
    header.cause = Some(OpId::new());
    let source_root = source.parent().unwrap_or(Path::new("."));
    let source_blobs = Blobs {
        dir: source_root.join(original.session.to_string()).join("blobs"),
        redactor: redactor.clone(),
    };
    let target = out.join(format!("{}.jsonl", meta.session));
    let staging = out.join(format!(".reproject-{}-{}.tmp", meta.session, OpId::new()));
    tokio::fs::create_dir_all(&out).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staging)
        .await?;
    file.write_all(&header.wire_bytes()?).await?;
    for e in prefix.iter().skip(1) {
        file.write_all(&e.wire_bytes()?).await?;
    }
    file.sync_all().await?;
    drop(file);
    // All source blobs belong to the copied prefix, including references hidden
    // inside future opaque values and the older textual spill representation.
    if let Ok(mut entries) = tokio::fs::read_dir(&source_blobs.dir).await {
        let dir = out.join(meta.session.to_string()).join("blobs");
        tokio::fs::create_dir_all(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.len() == 64
                && name.bytes().all(|b| b.is_ascii_hexdigit())
                && entry.file_type().await?.is_file()
            {
                tokio::fs::copy(entry.path(), dir.join(name.as_ref())).await?;
                tokio::fs::File::open(dir.join(name.as_ref()))
                    .await?
                    .sync_all()
                    .await?;
            }
        }
    }
    tokio::fs::hard_link(&staging, &target).await?;
    tokio::fs::remove_file(&staging).await?;
    let mut seen: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in &records {
        if let Event::ItemCompleted { body, .. } = &e.event
            && !matches!(body, Item::Native { .. })
        {
            seen.entry(key(e, None))
                .or_default()
                .insert(serde_json::to_string(body)?);
        }
    }
    let mut projectors: BTreeMap<String, Projector> = BTreeMap::new();
    let mut projected: Vec<(Projected, String, u64)> = vec![];
    let mut owners: BTreeMap<ItemId, String> = BTreeMap::new();
    for e in &records {
        let mut candidates = vec![];
        if let Event::ItemCompleted {
            body:
                Item::Native {
                    provider,
                    raw,
                    blob,
                    ..
                },
            ..
        } = &e.event
        {
            let raw = match blob {
                Some(blob) => serde_json::from_slice(&source_blobs.get(blob).await?)?,
                None => raw.clone(),
            };
            candidates.push((provider.clone(), raw));
        }
        for (provider, value) in &e.ext {
            candidates.push((provider.clone(), inflate(value, &source_blobs).await?));
        }
        for (provider, raw) in candidates {
            let record_key = key(e, Some(&raw));
            if let Some(native) = Native::parse(&provider) {
                let p = projectors
                    .entry(provider)
                    .or_insert_with(|| Projector::new(native));
                for event in p.project(&raw) {
                    if let Event::ItemStarted { item, .. } = event.event {
                        owners.insert(item, record_key.clone());
                    }
                    projected.push((event, record_key.clone(), e.ts_ms));
                }
            } else if provider.starts_with("acp") {
                #[cfg(feature = "acp-agent")]
                if let Some(body) = crate::backend::acp_agent::reproject_update(&provider, &raw) {
                    let item = ItemId::new();
                    let identity = e.native.clone();
                    projected.push((
                        Projected {
                            event: Event::ItemStarted {
                                turn: None,
                                item,
                                kind: body.kind(),
                                parent: None,
                                head: body.head(),
                            },
                            native: identity.clone(),
                            ext: Default::default(),
                        },
                        record_key.clone(),
                        e.ts_ms,
                    ));
                    projected.push((
                        Projected {
                            event: Event::ItemCompleted {
                                item,
                                status: ItemStatus::Completed,
                                body,
                            },
                            native: identity,
                            ext: Default::default(),
                        },
                        record_key,
                        e.ts_ms,
                    ));
                }
            }
        }
    }
    for p in projectors.values_mut() {
        for event in p.finish(StopReason::EndTurn) {
            let owner = match &event {
                Event::ItemCompleted { item, .. } => owners.get(item).cloned().unwrap_or_default(),
                _ => String::new(),
            };
            projected.push((
                Projected {
                    event,
                    native: None,
                    ext: Default::default(),
                },
                owner,
                meta.created_ms,
            ));
        }
    }
    let mut skip = BTreeSet::new();
    for (e, key, _) in &projected {
        if let Event::ItemCompleted { item, body, .. } = &e.event
            && (matches!(body, Item::Native { .. })
                || seen.get(key).is_some_and(|old| {
                    old.contains(&serde_json::to_string(body).unwrap_or_default())
                }))
        {
            skip.insert(*item);
        }
    }
    let active: BTreeSet<TurnId> = projected
        .iter()
        .filter_map(|(e, _, _)| match e.event {
            Event::ItemStarted {
                turn: Some(turn),
                item,
                ..
            } if !skip.contains(&item) => Some(turn),
            _ => None,
        })
        .collect();
    let mut store = Store::open(out.clone(), meta.clone(), None, redactor.clone()).await?;
    for (e, _, ts) in projected {
        let keep = match &e.event {
            Event::ItemStarted { item, .. } | Event::ItemCompleted { item, .. } => {
                !skip.contains(item)
            }
            Event::TurnQueued { turn }
            | Event::TurnStarted { turn, .. }
            | Event::TurnEnded { turn, .. } => active.contains(turn),
            _ => true,
        };
        if keep {
            super::append_projected(&mut store, e.event, None, ts, e.native, e.ext).await?;
        }
    }
    store.append(Event::SessionClosed, None).await?;
    drop(crate::frontends::ir_scan::on_close(
        target.clone(),
        redactor,
        |text| eprintln!("Info: {text}"),
    ));
    Ok(target)
}
fn key(e: &Envelope, raw: Option<&Value>) -> String {
    raw.and_then(native_identity)
        .or_else(|| e.native.clone())
        .map(|n| n.id)
        .unwrap_or_else(|| format!("seq:{}", e.seq.map(|s| s.0).unwrap_or(0)))
}
async fn inflate(value: &Value, blobs: &Blobs) -> io::Result<Value> {
    if let Some(blob) = value.get("$blob") {
        let blob: BlobRef = serde_json::from_value(blob.clone())?;
        return Ok(serde_json::from_slice(&blobs.get(&blob).await?)?);
    }
    Ok(value.clone())
}
