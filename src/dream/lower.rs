// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Immutable publication using the shared selection, effect and constructive-seal law.
//! No policy bootstrap or run seal: an enclosing approved run must observe this stage.
use super::{
    entry, graph, projection,
    receipt::{self, Body, Domains, Operation, ResultCode as R},
    resolve::Resolver,
    seal::{self, Owner, Proof, ProofError as E},
};
use crate::{protocol::*, redact::Set, store::Store};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::MetadataExt,
    },
    path::{Component, Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

/// Not deserializable: constructed from current committed pins and actual approval.
pub struct Plan {
    body: Body,
    files: BTreeMap<PathBuf, Vec<u8>>,
    domains: Domains,
    head: String,
    at: u64,
}
/// Stage evidence. Until an enclosing run receipt passes R4 this is UNPROVEN
/// for complete audit/Recall; this type does not confer historical authority.
pub struct Publication {
    pub accepted: receipt::Accepted,
    pub events: Vec<Envelope>,
}

pub async fn prepare(
    store: &Store,
    resolver: &mut Resolver,
    sink: SinkRef,
    harness: &str,
    budget: Option<receipt::Budget>,
    set: &Set,
) -> Proof<Plan> {
    prepare_at(
        store,
        resolver,
        sink,
        harness,
        budget,
        set,
        crate::store::now_ms(),
    )
    .await
}
/// Explicit observation time for deterministic replay/builders. Time ranks only;
/// current pin/projection/approval checks still determine eligibility.
pub async fn prepare_at(
    store: &Store,
    resolver: &mut Resolver,
    sink: SinkRef,
    harness: &str,
    budget: Option<receipt::Budget>,
    set: &Set,
    at: u64,
) -> Proof<Plan> {
    if store.closed
        || harness.is_empty()
        || !matches!(store.events.first().map(|e|&e.event), Some(Event::SessionStarted{meta}) if meta.harness=="dream" && meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Lower))
    {
        return Err(E::Fail("publication_session"));
    }
    if matches!(sink.kind, SinkKind::GrokInbox | SinkKind::Proposal) {
        // No written archive locator/loader or cooperative proposal consumer is
        // available in this package. Rendering is shared; materialization cannot
        // masquerade as an effective native adoption.
        return Err(E::Unproven("native_archive_or_proposal_loader_unavailable"));
    }
    let trust = resolver.trust().ok_or(E::Unproven("producer_unapproved"))?;
    let producer = trust
        .current_producer(&[
            "receipt/1",
            "effect-wrapper/1",
            "no-replace/1",
            "constructive-seal/1",
        ])
        .await
        .map_err(|_| E::Unproven("producer_unapproved"))?;
    let domains = resolver.receipt_domains();
    let retained = resolver.logs.clone();
    let session = store.session;
    let own = store.events.clone();
    let (logs, facts, projected, cut, projection_digest) = crate::runtime::blocking(move || {
        let mut logs = (*retained).clone();
        logs.insert(session, own);
        let all = projection::union(logs.values().flatten().cloned()).map_err(|e| E::Fail(e.0))?;
        let facts = projection::facts_with_logs(&all, &logs).map_err(|e| E::Fail(e.0))?;
        let projected = projection::Projection::build(&facts).map_err(|e| E::Fail(e.0))?;
        let cut = logs
            .iter()
            .map(|(s, l)| {
                Ok(seal::Prefix {
                    session: *s,
                    through_seq: l.len() as u64,
                    prefix_digest: seal::prefix(l, l.len() as u64)?,
                })
            })
            .collect::<Proof<Vec<_>>>()?;
        let digest = entry::digest(
            entry::canonical_value(
                serde_json::to_value(&projected).map_err(|_| E::Fail("projection_encoding"))?,
            )
            .map_err(|e| E::Fail(e.0))?,
        );
        Ok::<_, E>((logs, facts, projected, cut, digest))
    })
    .await?;
    let mut body = Body {
        schema: receipt::SCHEMA.into(),
        kind: ReceiptKind::Publication,
        outcome: ReceiptOutcome::Skipped,
        harness: Some(harness.into()),
        sink: Some(sink),
        run: receipt::Run {
            session: store.session,
        },
        producer,
        coverage: None,
        ops: vec![],
        seal: None,
        inventory: Some(receipt::Inventory {
            root: domains.sessions.clone(),
            sessions: cut
                .iter()
                .map(|c| receipt::InventorySession {
                    session: c.session,
                    through_seq: c.through_seq,
                })
                .collect(),
        }),
        cut: Some(cut),
        projection_digest: Some(projection_digest),
        selection: None,
        sources: None,
        skip_reason: None,
    };
    let current = resolver;
    // No unverified receipt session may boost rank. Existing accepted sessions are
    // supplied by the common audit/trust boundary, not guessed from artifact presence.
    let tasks = current
        .accepted_tasks()
        .await
        .map_err(|_| E::Unproven("accepted_session_verdicts"))?;
    let accepted=logs.iter().filter_map(|(s,l)|{
        let closed=l.iter().any(|e|matches!(e.event,Event::SessionClosed));
        let matched=l.iter().any(|e|matches!(&e.event,Event::ItemCompleted{status:ItemStatus::Completed,body:Item::ToolCall{name,input_raw,..},..} if serde_json::from_str(input_raw).is_ok_and(|v|tasks.iter().any(|t|t.matches(name,&v)))));
        (closed&&matched).then_some(*s)
    }).collect::<BTreeSet<_>>();
    let candidates =
        receipt::publication_candidates(&body, &facts, &projected, current, &accepted, at).await?;
    let mut memories = Vec::new();
    for (c, r) in &candidates {
        let resolved = current
            .record(&c.record)
            .await
            .map_err(|_| E::Unproven("candidate_pin"))?;
        if !resolved.signature_valid {
            return Err(E::Unproven("candidate_signature"));
        }
        if !matches!(
            projection::record_view(r, &projected.entries[&c.entry]),
            Ok(projection::RecordView::Current)
        ) {
            return Err(E::Unproven("candidate_metadata"));
        }
        super::deny::value(&r.front, set).map_err(|_| E::Fail("denied_block"))?;
        memories.push(graph::Memory {
            root: c.record.root.clone(),
            path: c.record.path.clone(),
            record: r.clone(),
        });
    }
    // Resolve predecessor records through the same pinned graph as trials/audit.
    for f in &facts {
        if let Item::MemoryWrite {
            record,
            op: MemoryOp::Created | MemoryOp::Revised,
            ..
        } = &f.body
            && let Ok(r) = current.record(record).await
            && !memories
                .iter()
                .any(|m| m.root == record.root && m.path == record.path)
        {
            memories.push(graph::Memory {
                root: record.root.clone(),
                path: record.path.clone(),
                record: r.record.clone(),
            });
        }
    }
    let metadata = logs
        .iter()
        .filter_map(|(s, l)| {
            l.iter().find_map(|e| {
                if let Event::SessionStarted { meta } = &e.event {
                    Some((*s, meta.clone()))
                } else {
                    None
                }
            })
        })
        .collect();
    let graph = graph::Graph::build(
        &memories, &facts, &metadata, &projected, &accepted, false, at,
    )
    .map_err(|e| E::Fail(e.0))?;
    for (c, _) in &candidates {
        let related = super::trial::closure(&graph, &c.entry)
            .map_err(|_| E::Unproven("candidate_closure"))?;
        let mut anchored = false;
        for id in related {
            let m = memories
                .iter()
                .find(|m| m.record.identity().and_then(|i| i.id()).ok().as_ref() == Some(&id))
                .ok_or(E::Unproven("candidate_closure"))?;
            let mut evidence = vec![];
            for value in m.record.identity().map_err(|e| E::Fail(e.0))?.evidence {
                let p = current
                    .evidence_value(&m.root, &value)
                    .await
                    .map_err(|_| E::Unproven("candidate_evidence"))?;
                anchored |= current
                    .evidence(&p)
                    .await
                    .map_err(|_| E::Unproven("candidate_evidence"))?;
                evidence.push(p);
            }
            current
                .scope(&m.record.scope().map_err(|e| E::Fail(e.0))?, &evidence)
                .await
                .map_err(|_| E::Unproven("candidate_scope"))?;
        }
        if !anchored {
            return Err(E::Unproven("candidate_anchor"));
        }
    }
    let budget = budget.unwrap_or(receipt::publication_budget(
        &body.sink.as_ref().unwrap().kind,
    )?);
    let (selection, files) = receipt::publication_selection(&body, candidates.clone(), budget, at)?;
    let selected = candidates
        .into_iter()
        .filter(|(c, _)| selection.selected.contains(&c.entry))
        .collect::<Vec<_>>();
    receipt::publication_scope(&body, current, &domains, &selected, Some(&store.events))?;
    if files.keys().any(|p| {
        !domains.sink_allowed(p) || super::deny::instruction_sink(p.to_str().unwrap_or(""))
    }) {
        return Err(E::Fail("generated_domain"));
    }
    if selection.selected.is_empty() {
        return Err(E::Unproven("no_eligible_memory"));
    }
    // Receipt bytes must survive the EXISTING exact canonical-number consumer.
    // Refuse an unrepresentable rank before any filesystem effect; never round,
    // change rank math, retry the clock, or weaken a consumer comparison.
    let value =
        serde_json::to_value(&selection).map_err(|_| E::Unproven("selection_number_encoding"))?;
    let wire = serde_json::to_vec(&value).map_err(|_| E::Unproven("selection_number_encoding"))?;
    let decoded: receipt::Selection =
        serde_json::from_slice(&wire).map_err(|_| E::Unproven("selection_number_encoding"))?;
    if decoded
        .candidates
        .iter()
        .zip(&selection.candidates)
        .any(|(a, b)| a.rank.to_bits() != b.rank.to_bits())
        || serde_json::to_value(decoded).ok() != Some(value)
    {
        return Err(E::Unproven("selection_number_encoding"));
    }
    body.selection = Some(selection);
    Ok(Plan {
        body,
        files,
        domains,
        head: seal::prefix(&store.events, store.events.len() as u64)?,
        at,
    })
}

/// Caller abandonment requests cancellation but never aborts the owner. The owned
/// Store, blocking job and temp descriptors live until settlement, including on Drop.
pub struct PublicationJob {
    cancel: CancellationToken,
    settled: tokio::sync::watch::Receiver<bool>,
    task: tokio::task::JoinHandle<(Store, Proof<Publication>)>,
}
impl PublicationJob {
    /// Observation of owner settlement only, never proof of publication/audit success.
    pub fn settlement(&self) -> tokio::sync::watch::Receiver<bool> {
        self.settled.clone()
    }
    pub async fn settle(mut self) -> (Store, Proof<Publication>) {
        (&mut self.task)
            .await
            .expect("owned publication task panicked")
    }
}
impl Drop for PublicationJob {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
pub fn publish(store: Store, plan: Plan, outer: Owner, set: Set) -> Proof<PublicationJob> {
    if store.closed
        || store.session != plan.body.run.session
        || seal::prefix(&store.events, store.events.len() as u64)? != plan.head
    {
        return Err(E::Unproven("stale_publication_plan"));
    }
    let child = ulid::Ulid::new();
    outer.child(seal::Child {
        id: child,
        state: seal::ChildState::Active,
        coverage: seal::ChildCoverage::Observed,
    })?;
    let cancel = CancellationToken::new();
    let token = cancel.clone();
    let (done, settled) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(async move {
        let (store, result) = publish_owned(store, plan, &outer, &token, &set).await;
        let joined = outer.child(seal::Child {
            id: child,
            state: seal::ChildState::Joined,
            coverage: seal::ChildCoverage::Observed,
        });
        let result = result.and_then(|r| joined.map(|_| r));
        let _ = done.send(true);
        (store, result)
    });
    Ok(PublicationJob {
        cancel,
        settled,
        task,
    })
}
async fn publish_owned(
    mut store: Store,
    mut plan: Plan,
    outer: &Owner,
    cancel: &CancellationToken,
    set: &Set,
) -> (Store, Proof<Publication>) {
    let owner = Owner::default();
    let files = plan.files.clone();
    let d = plan.domains.clone();
    let p = owner.clone();
    let o = outer.clone();
    let c = cancel.clone();
    let shared = plan
        .body
        .sink
        .as_ref()
        .is_some_and(|s| s.kind == SinkKind::ClaudeMemory)
        .then(|| PathBuf::from(&plan.body.sink.as_ref().unwrap().path));
    let result =
        crate::runtime::blocking(move || materialize(&files, &d, &p, &o, &c, shared.as_deref()))
            .await;
    let prepared = async {
        let outcome = result?;
        plan.body.outcome = match outcome {
            receipt::Outcome::Published => ReceiptOutcome::Published,
            receipt::Outcome::Idempotent => ReceiptOutcome::Idempotent,
            receipt::Outcome::Skipped => ReceiptOutcome::Skipped,
        };
        if plan.body.outcome == ReceiptOutcome::Skipped {
            plan.body.skip_reason =
                Some("cancelled, conflicting or unsupported immutable target".into());
        }
        let item = ItemId::new();
        let from = store.events.len() as u64 + 1;
        store
            .append_at(
                Event::ItemStarted {
                    turn: None,
                    item,
                    kind: ItemKind::Receipt,
                    parent: None,
                    head: None,
                },
                None,
                plan.at,
            )
            .await
            .map_err(|_| E::Unproven("publication_append"))?;
        store
            .sync()
            .await
            .map_err(|_| E::Unproven("publication_append"))?;
        let (body, bytes, carrier) = plan.body.freeze(&owner, item, &store.events, set)?;
        let Item::Receipt {
            blob: Some(blob), ..
        } = &carrier
        else {
            unreachable!()
        };
        let blob = blob.clone();
        Ok((body, bytes, carrier, blob, item, from))
    }
    .await;
    let (body, bytes, carrier, blob, item, from) = match prepared {
        Ok(v) => v,
        Err(e) => return (store, Err(e)),
    };
    let admission = match outer.admit() {
        Ok(v) => v,
        Err(e) => return (store, Err(e)),
    };
    let set2 = set.clone();
    let b = body.clone();
    let expected = blob.clone();
    let (returned, persisted) = crate::runtime::blocking(move || {
        let result = seal::persist(&store, &b, &bytes, &expected, &set2);
        (store, result)
    })
    .await;
    store = returned;
    if let Err(e) = admission
        .finish(Operation::WriteOwnBlob {
            sha256: blob.sha256.clone(),
            result: if persisted.is_ok() { R::Ok } else { R::Error },
        })
        .and(persisted.map(|_| ()))
    {
        return (store, Err(e));
    }
    let result = async {
        let append = outer.admit()?;
        let event = store
            .append(
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: carrier,
                },
                None,
            )
            .await
            .map_err(|_| E::Unproven("publication_append"))?;
        let accepted = receipt::Accepted {
            body: body.clone(),
            blob: blob.clone(),
            item,
            seq: event.seq.unwrap(),
            temp_leaks: vec![],
        };
        if body.outcome != ReceiptOutcome::Skipped {
            let selection = body.selection.as_ref().unwrap();
            for entry in &selection.selected {
                let candidate = selection
                    .candidates
                    .iter()
                    .find(|c| &c.entry == entry)
                    .unwrap();
                let id = ItemId::new();
                store
                    .append(
                        Event::ItemStarted {
                            turn: None,
                            item: id,
                            kind: ItemKind::Adoption,
                            parent: None,
                            head: None,
                        },
                        None,
                    )
                    .await
                    .map_err(|_| E::Unproven("publication_append"))?;
                let event = store
                    .append(
                        Event::ItemCompleted {
                            item: id,
                            status: ItemStatus::Completed,
                            body: Item::Adoption {
                                entry: entry.clone(),
                                harness: body.harness.clone().unwrap(),
                                sink: body.sink.clone().unwrap(),
                                record: Some(candidate.record.clone()),
                                block_digest: selection.block_digest.clone(),
                                receipt: Some(format!("sha256:{}", blob.sha256)),
                            },
                        },
                        None,
                    )
                    .await
                    .map_err(|_| E::Unproven("publication_append"))?;
                receipt::check_adoption(&accepted, &event)?;
            }
        }
        store
            .sync()
            .await
            .map_err(|_| E::Unproven("publication_append"))?;
        append.finish(Operation::AppendOwnLog {
            session: store.session,
            seq_from: from,
            seq_to: store.events.len() as u64,
            result: R::Ok,
        })?;
        Ok(Publication {
            accepted,
            events: store.events.clone(),
        })
    }
    .await;
    (store, result)
}

fn name(path: &Path) -> Proof<CString> {
    CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| E::Fail("generated_domain"))
}
fn io() -> E {
    E::Unproven("publication_io")
}
/// Descriptor traversal never follows a symlink. Missing directories are created
/// with mkdirat, then opened without replacement; existing metadata is never changed.
fn parent(path: &Path, owner: &Owner, outer: &Owner) -> Proof<File> {
    let root = name(Path::new("/"))?;
    let fd = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io());
    }
    let mut dir = unsafe { File::from_raw_fd(fd) };
    let mut walked = PathBuf::from("/");
    for part in path
        .parent()
        .ok_or(E::Fail("generated_domain"))?
        .components()
    {
        let Component::Normal(part) = part else {
            if part == Component::RootDir {
                continue;
            }
            return Err(E::Fail("generated_domain"));
        };
        walked.push(part);
        let n = name(Path::new(part))?;
        let mut fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                n.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
            let a = owner.admit()?;
            let b = outer.admit()?;
            let mut metadata: libc::stat = unsafe { std::mem::zeroed() };
            let status = unsafe {
                libc::fstatat(
                    dir.as_raw_fd(),
                    n.as_ptr(),
                    &mut metadata,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if status == 0 || std::io::Error::last_os_error().kind() != std::io::ErrorKind::NotFound
            {
                return Err(E::Unproven("directory_changed"));
            }
            let absent = Operation::Read {
                path: walked.clone(),
                result: R::Absent,
                sha256: None,
                len: None,
            };
            a.finish(absent.clone())?;
            b.finish(absent)?;
            let a = owner.admit()?;
            let b = outer.admit()?;
            let rc = unsafe { libc::mkdirat(dir.as_raw_fd(), n.as_ptr(), 0o700) };
            let op = Operation::Mkdir {
                path: walked.clone(),
                result: if rc == 0 {
                    R::Ok
                } else if std::io::Error::last_os_error().kind()
                    == std::io::ErrorKind::AlreadyExists
                {
                    R::Eexist
                } else {
                    R::Error
                },
            };
            a.finish(op.clone())?;
            b.finish(op)?;
            fd = unsafe {
                libc::openat(
                    dir.as_raw_fd(),
                    n.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
        }
        if fd < 0 {
            return Err(io());
        }
        dir = unsafe { File::from_raw_fd(fd) };
    }
    Ok(dir)
}
fn read(dir: &File, path: &Path, limit: usize) -> (R, Option<Vec<u8>>) {
    let Ok(n) = name(Path::new(path.file_name().unwrap())) else {
        return (R::Error, None);
    };
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            n.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return (
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
                R::Absent
            } else {
                R::Symlink
            },
            None,
        );
    }
    let file = unsafe { File::from_raw_fd(fd) };
    if !file
        .metadata()
        .is_ok_and(|m| m.is_file() && m.nlink() == 1 && m.len() <= limit as u64)
    {
        return (R::Error, None);
    }
    let mut bytes = vec![];
    if file.take(limit as u64 + 1).read_to_end(&mut bytes).is_err() || bytes.len() > limit {
        return (R::Error, None);
    };
    (R::Present, Some(bytes))
}
fn read_effect(dir: &File, path: &Path, bytes: &[u8], owner: &Owner, outer: &Owner) -> Proof<R> {
    let a = owner.admit()?;
    let b = outer.admit()?;
    let (r, seen) = read(dir, path, bytes.len());
    let exact = seen.as_deref() == Some(bytes);
    let op = Operation::Read {
        path: path.into(),
        result: r.clone(),
        sha256: seen.as_ref().map(super::fingerprint::digest),
        len: seen.as_ref().map(|b| b.len() as u64),
    };
    a.finish(op.clone())?;
    b.finish(op)?;
    if r == R::Present
        && !exact
        && seen.as_ref().is_some_and(|observed| {
            observed.len() == bytes.len()
                && super::fingerprint::digest(observed) == super::fingerprint::digest(bytes)
        })
    {
        return Err(E::Fail("materialization_digest_collision"));
    }
    Ok(r)
}
fn materialize(
    files: &BTreeMap<PathBuf, Vec<u8>>,
    domains: &Domains,
    owner: &Owner,
    outer: &Owner,
    cancel: &CancellationToken,
    shared: Option<&Path>,
) -> Proof<receipt::Outcome> {
    crate::runtime::assert_off_rt_worker();
    if let Some(index) = shared {
        let dir = parent(index, owner, outer)?;
        if read_effect(&dir, index, &files[index], owner, outer)? != R::Absent {
            return Ok(receipt::Outcome::Skipped);
        }
    }
    let mut paths = files.keys().collect::<Vec<_>>();
    paths.sort_by_key(|p| (shared == Some(p.as_path()), (*p).clone()));
    for path in paths {
        if cancel.is_cancelled() {
            break;
        }
        if !domains.sink_allowed(path) {
            return Err(E::Fail("generated_domain"));
        }
        let bytes = &files[path];
        let dir = match parent(path, owner, outer) {
            Ok(d) => d,
            Err(_) => break,
        };
        let initial = read_effect(&dir, path, bytes, owner, outer)?;
        if initial != R::Absent {
            continue;
        }
        let temp = path.with_file_name(format!(".dream-tmp-{}", ulid::Ulid::new()));
        let n = name(Path::new(temp.file_name().unwrap()))?;
        let target = name(Path::new(path.file_name().unwrap()))?;
        let a = owner.admit()?;
        let b = outer.admit()?;
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                n.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        let op = Operation::CreateTemp {
            path: temp.clone(),
            result: if fd >= 0 { R::Ok } else { R::Error },
        };
        a.finish(op.clone())?;
        b.finish(op)?;
        if fd < 0 {
            continue;
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        let a = owner.admit()?;
        let b = outer.admit()?;
        let result = file.write(bytes);
        let full = matches!(result,Ok(n) if n==bytes.len());
        let op = Operation::WriteTemp {
            path: temp.clone(),
            sha256: super::fingerprint::digest(bytes),
            len: bytes.len() as u64,
            result: if full { R::Ok } else { R::Error },
        };
        a.finish(op.clone())?;
        b.finish(op)?;
        let mut synced = false;
        if full && !cancel.is_cancelled() {
            let a = owner.admit()?;
            let b = outer.admit()?;
            crate::runtime::blocking_hook(crate::runtime::BlockingWork::Fsync);
            synced = file.sync_all().is_ok();
            let op = Operation::Fsync {
                path: temp.clone(),
                result: if synced { R::Ok } else { R::Error },
            };
            a.finish(op.clone())?;
            b.finish(op)?;
        }
        if synced && !cancel.is_cancelled() {
            let a = owner.admit()?;
            let b = outer.admit()?;
            let rc = unsafe {
                libc::linkat(
                    dir.as_raw_fd(),
                    n.as_ptr(),
                    dir.as_raw_fd(),
                    target.as_ptr(),
                    0,
                )
            };
            let r = if rc == 0 {
                R::Ok
            } else if std::io::Error::last_os_error().kind() == std::io::ErrorKind::AlreadyExists {
                R::Eexist
            } else {
                R::Error
            };
            let op = Operation::LinkNoreplace {
                from: temp.clone(),
                to: path.clone(),
                result: r.clone(),
            };
            a.finish(op.clone())?;
            b.finish(op)?;
            if r == R::Ok {
                let a = owner.admit()?;
                let b = outer.admit()?;
                let ok = dir.sync_all().is_ok();
                let op = Operation::FsyncDir {
                    path: path.parent().unwrap().into(),
                    result: if ok { R::Ok } else { R::Error },
                };
                a.finish(op.clone())?;
                b.finish(op)?;
            } else if r == R::Eexist {
                read_effect(&dir, path, bytes, owner, outer)?;
            }
        }
        drop(file);
        let a = owner.admit()?;
        let b = outer.admit()?;
        let ok = unsafe { libc::unlinkat(dir.as_raw_fd(), n.as_ptr(), 0) } == 0;
        let op = Operation::UnlinkOwnTemp {
            path: temp,
            result: if ok { R::Ok } else { R::Error },
        };
        a.finish(op.clone())?;
        b.finish(op)?;
    }
    // Snapshot effects without freezing: the Receipt prefix must be appended first.
    let effects = owner.effects()?;
    let generated = files
        .iter()
        .map(|(path, b)| receipt::Generated {
            path: path.clone(),
            sha256: super::fingerprint::digest(b),
            len: b.len() as u64,
        })
        .collect::<Vec<_>>();
    receipt::interpret(
        &effects,
        &generated,
        SessionId::new(),
        &domains.caches,
        None,
    )
    .map(|i| i.outcome)
    .map_err(|e| E::Fail(e.0))
}
