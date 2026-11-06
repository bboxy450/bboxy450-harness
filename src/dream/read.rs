// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Closed, selected, verified-at-revision publication exporter. No live Recall effect.
use super::{c1_wire as c, read_wire as w, resolve::HelperTicket, worker::*};
use crate::protocol::{Envelope, Event, SessionId};
use disposal::Drain;
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

pub(crate) struct ReadJob;
fn internal(code: w::Code) -> c::Code {
    match code {
        w::Code::BadRequest => c::Code::BadRequest,
        w::Code::SourceMismatch => c::Code::SourceMismatch,
        w::Code::SourceChanged => c::Code::SourceChanged,
        w::Code::UnsupportedIr => c::Code::UnsupportedIr,
        w::Code::BlobMismatch => c::Code::BlobMismatch,
        w::Code::ResourceLimit => c::Code::ResourceLimit,
        w::Code::StorageUnavailable => c::Code::StorageUnavailable,
        w::Code::Cancelled => c::Code::Cancelled,
        w::Code::Deadline => c::Code::Deadline,
        w::Code::WorkerLost => c::Code::WorkerLost,
        _ => c::Code::ProtocolError,
    }
}
fn external(code: c::Code) -> w::Code {
    match code {
        c::Code::BadRequest => w::Code::BadRequest,
        c::Code::SourceChanged => w::Code::SourceChanged,
        c::Code::SourceMismatch => w::Code::SourceMismatch,
        c::Code::UnsupportedIr => w::Code::UnsupportedIr,
        c::Code::BlobMismatch => w::Code::BlobMismatch,
        c::Code::ResourceLimit => w::Code::ResourceLimit,
        c::Code::StorageUnavailable => w::Code::StorageUnavailable,
        c::Code::Cancelled => w::Code::Cancelled,
        c::Code::Deadline => w::Code::Deadline,
        c::Code::WorkerLost => w::Code::WorkerLost,
        c::Code::SourceUnavailable => w::Code::VerificationUnproven,
        c::Code::ProtocolError => w::Code::ProtocolError,
        _ => w::Code::VerificationFailed,
    }
}
fn err(code: w::Code) -> w::Error {
    w::Error { code, source: None }
}
impl Job for ReadJob {
    fn purpose() -> super::bootstrap::Purpose {
        super::bootstrap::Purpose::ReadPrepare
    }
    type Error = w::Error;
    fn error(e: c::Error) -> Self::Error {
        w::Error {
            code: external(e.code),
            source: e.source,
        }
    }
    fn code(e: &Self::Error) -> c::Code {
        internal(e.code)
    }
    fn hello() -> Frame {
        framed(&w::WorkerOut::Hello(w::Hello {
            protocol: w::Protocol::V1,
            algorithm: w::Algorithm::ReadPrepare,
            schema_sha256: w::schema_hash(),
            ir_major: 1,
            ir_minor_max: 3,
        }))
    }
    fn decode(bytes: &[u8]) -> Result<c::WorkerIn, c::Code> {
        Ok(
            match w::decode(bytes).map_err(|_| c::Code::ProtocolError)? {
                w::WorkerIn::Start(s) => c::WorkerIn::Start(c::Start {
                    protocol: c::Protocol::V1,
                    run_id: s.run_id,
                    nonce: s.nonce,
                    manifest_sha256: s.input_sha256,
                    timeout_ms: s.timeout_ms,
                    activity_revision: s.activity_revision,
                    foreground: s.foreground,
                }),
                w::WorkerIn::Activity(a) => c::WorkerIn::Activity(c::Activity {
                    protocol: c::Protocol::V1,
                    run_id: a.run_id,
                    nonce: a.nonce,
                    revision: a.revision,
                    foreground: a.foreground,
                }),
                w::WorkerIn::Cancel(a) => c::WorkerIn::Cancel(c::WorkerCancel {
                    protocol: c::Protocol::V1,
                    run_id: a.run_id,
                    nonce: a.nonce,
                }),
            },
        )
    }
    fn progress(p: c::Progress) -> Frame {
        framed(&w::WorkerOut::Progress(w::Progress {
            protocol: w::Protocol::V1,
            run_id: p.run_id,
            nonce: p.nonce,
            seq: p.seq,
            phase: match p.phase {
                c::Phase::Validate => w::Phase::Validate,
                c::Phase::Paused => w::Phase::Paused,
                c::Phase::Settling => w::Phase::Settling,
                _ => w::Phase::Prepare,
            },
            bytes: p.bytes,
            envelopes: p.envelopes,
            cpu_ns: p.cpu_ns,
        }))
    }
    fn done(d: c::Done, error: Option<w::Error>) -> Frame {
        framed(&w::WorkerOut::Done(w::Done {
            protocol: w::Protocol::V1,
            run_id: d.run_id,
            nonce: d.nonce,
            seq: d.seq,
            outcome: d.outcome,
            result_sha256: d.result_sha256,
            pages: d.pages,
            error,
        }))
    }
    fn compute(
        path: PathBuf,
        start: c::Start,
        control: Arc<Control>,
        progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    ) -> Result<Completed, w::Error> {
        compute(path, start, control, progress)
    }
}
/// Managed reads are admitted only by the canonical borrowed earliest-entry
/// gate, before argv allocation or async-runtime initialization. A late CLI call
/// cannot reopen that admission path or start a worker.
pub async fn cli(_args: impl Iterator<Item = String>) -> i32 {
    2
}
impl Drain for std::fs::Metadata {
    fn drain_step(&mut self) -> bool {
        false
    }
}
impl Drain for (String, super::recall::PreparedLesson) {
    fn drain_step(&mut self) -> bool {
        self.1.drain_step()
    }
}
impl Drain for (Dir, String, File, std::fs::Metadata) {
    fn drain_step(&mut self) -> bool {
        false
    }
}
impl<T: Drain> Drain for Result<T, w::Code> {
    fn drain_step(&mut self) -> bool {
        match self {
            Ok(v) => v.drain_step(),
            Err(_) => false,
        }
    }
}
impl Drain for super::inspect::Transcript {
    fn drain_step(&mut self) -> bool {
        super::inspect::Transcript::drain_step(self)
    }
}
impl Drain for c::Source {
    fn drain_step(&mut self) -> bool {
        self.blobs.pop().is_some()
    }
}
impl Drain for w::Input {
    fn drain_step(&mut self) -> bool {
        self.closed_sources.drain_step()
            || self.records.pop().is_some()
            || self.publications.pop().is_some()
            || self.evidence.drain_step()
    }
}
impl Drain for w::EvidenceBundle {
    fn drain_step(&mut self) -> bool {
        self.files.pop().is_some()
            || self.roots.pop().is_some()
            || self.public_keys.pop().is_some()
            || self.artifacts.pop().is_some()
            || self.catalogue.pop().is_some()
    }
}
impl Drain for w::PreparedEntry {
    fn drain_step(&mut self) -> bool {
        self.nodes.pop().is_some() || self.edges.pop().is_some()
    }
}
impl Drain for w::Page {
    fn drain_step(&mut self) -> bool {
        self.entries.drain_step()
    }
}
impl Drain for w::Snapshot {
    fn drain_step(&mut self) -> bool {
        self.pages.pop().is_some()
            || self.entry_ids.pop().is_some()
            || self.coverage.kinds.pop().is_some()
    }
}
struct OwnedLogs(BTreeMap<SessionId, Vec<Envelope>>);
impl Drain for OwnedLogs {
    fn drain_step(&mut self) -> bool {
        let Some(mut e) = self.0.first_entry() else {
            return false;
        };
        if !e.get_mut().drain_step() {
            e.remove_entry();
        }
        true
    }
}
fn budget(control: Arc<Control>, progress: Arc<crossbeam_queue::ArrayQueue<Counters>>) -> Budget {
    Budget::new_read(control, progress)
}
fn low_priority() -> Result<(), c::Code> {
    if unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, 10) } != 0 {
        return Err(c::Code::ResourceLimit);
    }
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(class: u32, relative: i32) -> i32;
            fn pthread_mach_thread_np(thread: libc::pthread_t) -> u32;
            fn thread_policy_set(thread: u32, flavor: i32, policy: *const i32, count: u32) -> i32;
        }
        if unsafe { pthread_set_qos_class_self_np(0x11, 0) } != 0 {
            return Err(c::Code::ResourceLimit);
        }
        // Utility QoS otherwise coalesces9ms waits into23–44ms on the measured
        // host. Select precise timer latency without raising CPU QoS/priority or
        // shortening the shared mandatory9ms wait. DarwinSDK thread_policy.h7,
        // task_policy.h LATENCY_QOS_TIER_0. Native scheduling remains measured.
        let latency_tier = 0x00ff_0001i32;
        if unsafe {
            thread_policy_set(
                pthread_mach_thread_np(libc::pthread_self()),
                7,
                &latency_tier,
                1,
            )
        } != 0
        {
            return Err(c::Code::ResourceLimit);
        }
    }
    Ok(())
}
fn relative_file(
    dir: &Dir,
    path: &str,
    max: u64,
    b: &mut Budget,
) -> Result<(Dir, String, File, std::fs::Metadata), c::Code> {
    b.unit(|| relative_file_raw(dir, path, max))
}

fn relative_file_raw(
    dir: &Dir,
    path: &str,
    max: u64,
) -> Result<(Dir, String, File, std::fs::Metadata), c::Code> {
    let mut parts = path.split('/').peekable();
    let first = parts.next().ok_or(c::Code::BadRequest)?;
    let mut parent = dir.dir(first)?;
    while let Some(name) = parts.next() {
        if parts.peek().is_none() {
            let (file, metadata) = parent.file(name, max)?;
            return Ok((parent, name.into(), file, metadata));
        }
        parent = parent.dir(name)?;
    }
    Err(c::Code::BadRequest)
}
/// Inventory validation is finite enumeration of host-selected paths, never discovery.
fn inventory(dir: &Dir, m: &w::Input, b: &mut Budget) -> Result<(), c::Code> {
    if m.closed_sources.len() > 32
        || m.records.len() > 512
        || m.evidence.catalogue.len() > 512
        || m.evidence.files.len() > 2048
        || m.evidence.roots.len() > 32
        || m.publications.len() > 512
        || m.evidence.public_keys.len() > 32
        || m.evidence.artifacts.len() > 2048
    {
        return Err(c::Code::ResourceLimit);
    }
    if m.context.generation.0 == 0 || m.context.grant_epoch.0 == 0 {
        return Err(c::Code::BadRequest);
    }
    let _inventory_credit = b
        .control
        .memory
        .reserve((m.evidence.files.len() * 256) as u64)?;
    let mut total = 0u64;
    let mut last = "";
    let mut paths = BTreeSet::new();
    for (index, f) in m.evidence.files.iter().enumerate() {
        if f.slot as usize != index
            || f.path.0.as_str() <= last
            || !f.path.0.starts_with("input/")
            || f.path.0 == "input/manifest.json"
        {
            return Err(c::Code::BadRequest);
        }
        last = &f.path.0;
        paths.insert(f.path.0.as_str());
        if !f.path.0.starts_with("input/evidence/")
            && !m.closed_sources.iter().any(|source| {
                f.path.0 == format!("input/{}.jsonl", source.session.0)
                    || source.blobs.iter().any(|blob| {
                        f.path.0 == format!("input/{}/blobs/{}", source.session.0, blob.sha256.0)
                    })
            })
        {
            return Err(c::Code::BadRequest);
        }
        total = total.checked_add(f.len.0).ok_or(c::Code::ResourceLimit)?;
        if total > w::MAX_INPUT {
            return Err(c::Code::ResourceLimit);
        }
        if f.len.0 <= 8192 {
            b.unit(|| {
                let (parent, name, mut file, before) = relative_file_raw(dir, &f.path.0, f.len.0)?;
                if before.len() != f.len.0 {
                    return Err(c::Code::SourceChanged);
                }
                let mut chunk = [0u8; 8192];
                file.read_exact(&mut chunk[..f.len.0 as usize])
                    .map_err(|_| c::Code::SourceChanged)?;
                parent.unchanged(&name, &file, &before)?;
                if w::hash(&chunk[..f.len.0 as usize]) != f.sha256 {
                    return Err(c::Code::SourceChanged);
                }
                Ok(())
            })?;
        } else {
            let (parent, name, mut file, before) =
                b.unit(|| relative_file_raw(dir, &f.path.0, f.len.0))?;
            if before.len() != f.len.0 {
                return Err(c::Code::SourceChanged);
            }
            let mut hasher = Sha256::new();
            let mut length = 0;
            let mut chunk = [0u8; 16384];
            loop {
                let n = b.unit(|| {
                    let n = file
                        .read(&mut chunk)
                        .map_err(|_| c::Code::StorageUnavailable)?;
                    hasher.update(&chunk[..n]);
                    Ok(n)
                })?;
                if n == 0 {
                    break;
                }
                length += n as u64;
                if length > f.len.0 {
                    return Err(c::Code::SourceChanged);
                }
            }
            b.unit(|| {
                parent.unchanged(&name, &file, &before)?;
                if length != f.len.0 || w::Hex(format!("{:x}", hasher.finalize())) != f.sha256 {
                    return Err(c::Code::SourceChanged);
                }
                Ok(())
            })?;
        }
        b.counters.bytes = b.counters.bytes.saturating_add(f.len.0);
    }
    for source in &m.closed_sources {
        if !paths.contains(format!("input/{}.jsonl", source.session.0).as_str()) {
            return Err(c::Code::SourceUnavailable);
        }
        for blob in &source.blobs {
            if !paths
                .contains(format!("input/{}/blobs/{}", source.session.0, blob.sha256.0).as_str())
            {
                return Err(c::Code::SourceUnavailable);
            }
        }
    }
    if m.records.len() + m.evidence.catalogue.len() > 1024 {
        return Err(c::Code::ResourceLimit);
    }
    let mut pins = BTreeSet::new();
    for pin in m.records.iter().chain(m.evidence.catalogue.iter()) {
        pins.insert((&pin.root.0, &pin.path.0, &pin.commit.0, &pin.digest.0));
    }
    if pins.len() > 512 {
        return Err(c::Code::ResourceLimit);
    }
    let mut roots = BTreeSet::new();
    for (index, root) in m.evidence.roots.iter().enumerate() {
        if !roots.insert(root.root.0.as_str())
            || root.dir.0 != format!("input/evidence/roots/{index}")
        {
            return Err(c::Code::BadRequest);
        }
        for path in &paths {
            if path.starts_with(&format!("{}/", root.dir.0)) {
                let suffix = &path[root.dir.0.len() + 1..];
                if suffix == ".git"
                    || suffix.starts_with(".git/hooks/")
                    || suffix == ".git/objects/info/alternates"
                    || suffix == ".git/info/grafts"
                    || suffix.starts_with(".git/worktrees/")
                    || suffix == ".git/commondir"
                {
                    return Err(c::Code::BadRequest);
                }
            }
        }
    }
    for slot in std::iter::once(m.evidence.policy)
        .chain(m.evidence.public_keys.iter().map(|k| k.file))
        .chain(m.evidence.artifacts.iter().map(|a| a.file))
    {
        if slot as usize >= m.evidence.files.len() {
            return Err(c::Code::SourceUnavailable);
        }
    }
    Ok(())
}
impl Drain for storage::Credit {
    fn drain_step(&mut self) -> bool {
        false
    }
}
/// Parser overlap keeps C1's full preflight reservation. After serde returns,
/// requested capacities are bounded by the concrete v1.3 collection element
/// sizes plus raw/string bytes and four outer Envelope slots (growth overlap).
/// Opaque map/allocator overhead remains the unchanged measured helper margin.
fn retained_capacity(shape: storage::Shape, raw: usize) -> Result<u64, c::Code> {
    use crate::protocol::*;
    let element = [
        std::mem::size_of::<TrialRun>(),
        std::mem::size_of::<TrialEpisode>(),
        std::mem::size_of::<InputBlock>(),
        std::mem::size_of::<OutputBlock>(),
        std::mem::size_of::<EvidenceRef>(),
        std::mem::size_of::<Location>(),
        std::mem::size_of::<PlanEntry>(),
        std::mem::size_of::<ApprovalDecisionKind>(),
        std::mem::size_of::<Citation>(),
        std::mem::size_of::<Backend>(),
        std::mem::size_of::<Cost>(),
        std::mem::size_of::<serde_json::Value>(),
    ]
    .into_iter()
    .max()
    .unwrap() as u64;
    (shape.values + shape.keys)
        .checked_mul(element)
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add((raw as u64).checked_mul(4)?))
        .and_then(|n| n.checked_add(storage::slot().checked_mul(4)?))
        .ok_or(c::Code::ResourceLimit)
}
struct BundleLogs {
    logs: OwnedLogs,
    credits: Vec<storage::Credit>,
}
impl Drain for BundleLogs {
    fn drain_step(&mut self) -> bool {
        self.logs.drain_step() || self.credits.pop().is_some()
    }
}
fn selected_logs(input: &Dir, m: &w::Input, b: &mut Budget) -> Result<BundleLogs, c::Code> {
    let control = b.control.clone();
    let mut result = Paced::new(
        BundleLogs {
            logs: OwnedLogs(BTreeMap::new()),
            credits: vec![],
        },
        &b.control,
        Drain::drain_step,
    );
    let mut retired_laws = Paced::new(
        Vec::<super::inspect::Transcript>::new(),
        &b.control,
        Drain::drain_step,
    );
    let _reader_credit = b.control.memory.reserve(8192)?;
    let mut line = b.track(w::MAX_FRAME as u64, || {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(w::MAX_FRAME)
            .map_err(|_| c::Code::ResourceLimit)?;
        Ok(bytes)
    })?;
    let mut prior = None;
    let mut sessions = BTreeSet::new();
    let mut blob_count = 0usize;
    for source in &m.closed_sources {
        let key = (source.source_generation.0, source.session.0.as_str());
        if prior.is_some_and(|p| key <= p)
            || source.source_generation.0 == 0
            || source.closed_seq.0 == 0
            || !sessions.insert(&source.session.0)
        {
            return Err(c::Code::BadRequest);
        }
        prior = Some(key);
        blob_count = blob_count
            .checked_add(source.blobs.len())
            .ok_or(c::Code::ResourceLimit)?;
        if blob_count > 1024 || source.blobs.windows(2).any(|p| p[0].sha256 >= p[1].sha256) {
            return Err(c::Code::ResourceLimit);
        }
        let path = format!("input/{}.jsonl", source.session.0);
        let inventoried = m
            .evidence
            .files
            .iter()
            .find(|f| f.path.0 == path)
            .ok_or(c::Code::SourceUnavailable)?;
        if inventoried.len != source.len || inventoried.sha256 != source.sha256 {
            return Err(c::Code::SourceMismatch);
        }
        let name = format!("{}.jsonl", source.session.0);
        let (file, meta) = b.unit(|| input.file(&name, source.len.0))?;
        let mut reader = BufReader::with_capacity(8192, file);
        let mut law = Paced::new(
            super::inspect::Transcript::default(),
            &b.control,
            super::inspect::Transcript::drain_step,
        );
        let mut log = Paced::new(Vec::<Envelope>::new(), &b.control, Drain::drain_step);
        let mut seen = Paced::new(BTreeMap::new(), &b.control, |map| map.pop_first().is_some());
        // This slot outlives the work-unit verdict. Rejected typed envelopes
        // drain after that unit closes, while result retains their storage
        // credits until the event and any copied transcript state are gone.
        let mut pending = Paced::new(None::<Envelope>, &b.control, Drain::drain_step);
        let mut seq = 0;
        let mut last_hash = w::hash([]);
        let memory = b.control.memory.clone();
        let mut shape = None;
        let mut eof = false;
        let mut count = b.counters.envelopes;
        loop {
            // This is a finite continuation, not a whole-log work unit. Each
            // boundary performs at most64 chunks/preflights/typed records and
            // yields after400us, under the unchanged outer1ms/9ms Budget guard.
            b.unit(|| {
                let started = cpu_ns();
                if control.observe_interval(started, started) == u64::MAX {
                    return Err(c::Code::ResourceLimit);
                }
                for _ in 0..64 {
                    if let Some(parsed_shape) = shape.take() {
                        if count >= 65536 {
                            return Err(c::Code::ResourceLimit);
                        }
                        let parsed_shape: storage::Shape = parsed_shape;
                        result
                            .credits
                            .try_reserve(1)
                            .map_err(|_| c::Code::ResourceLimit)?;
                        let credit = memory.reserve(parsed_shape.typed(line.len())?)?;
                        let event = Envelope::from_line(
                            std::str::from_utf8(&line).map_err(|_| c::Code::MalformedIr)?,
                        )
                        .map_err(|_| c::Code::MalformedIr)?;
                        result.credits.push(credit);
                        *pending = Some(event);
                        let event = pending.as_ref().unwrap();
                        seq += 1;
                        super::inspect::validate_event(event, source, seq)?;
                        law.observe(event).map_err(|_| c::Code::MalformedIr)?;
                        last_hash = w::hash(line.strip_suffix(b"\n").ok_or(c::Code::MalformedIr)?);
                        for blob in super::inspect::blobs(&event.event)? {
                            if let Some(old) = seen.insert(blob.sha256.clone(), blob.clone())
                                && old != *blob
                            {
                                return Err(c::Code::BlobMismatch);
                            }
                        }
                        log.try_reserve(1).map_err(|_| c::Code::ResourceLimit)?;
                        let credit = result.credits.last_mut().unwrap();
                        credit.shrink(
                            credit
                                .bytes
                                .min(retained_capacity(parsed_shape, line.len())?),
                        );
                        log.push(pending.take().unwrap());
                        line.clear();
                        count += 1;
                    } else if line.last() == Some(&b'\n') {
                        shape = Some(storage::preflight(&line, storage::WORK)?);
                    } else {
                        let available = reader.fill_buf().map_err(|_| c::Code::SourceChanged)?;
                        if available.is_empty() {
                            eof = true;
                            break;
                        }
                        let n = available
                            .iter()
                            .position(|c| *c == b'\n')
                            .map_or(available.len(), |i| i + 1);
                        if line.len() + n > w::MAX_FRAME {
                            return Err(c::Code::ResourceLimit);
                        }
                        line.extend_from_slice(&available[..n]);
                        reader.consume(n);
                    }
                    if control.observe_interval(started, cpu_ns()) >= 400_000 {
                        break;
                    }
                }
                Ok(())
            })?;
            b.counters.envelopes = count;
            b.announce(c::Phase::Validate);
            if eof {
                break;
            }
        }
        b.unit(|| {
            input.unchanged(&name, reader.get_ref(), &meta)?;
            if !line.is_empty()
                || seq != source.closed_seq.0
                || last_hash != source.closed_line_sha256
                || !log
                    .last()
                    .is_some_and(|e| matches!(e.event, Event::SessionClosed))
            {
                return Err(c::Code::SourceMismatch);
            }
            law.finish().map_err(|_| c::Code::MalformedIr)?;
            if seen.len() != source.blobs.len() {
                return Err(c::Code::BlobMismatch);
            }
            for blob in &source.blobs {
                let actual = seen.get(&blob.sha256.0).ok_or(c::Code::BlobMismatch)?;
                if actual.len != blob.len.0 || actual.mime != blob.mime.0 {
                    return Err(c::Code::BlobMismatch);
                }
                let file = m
                    .evidence
                    .files
                    .iter()
                    .find(|f| {
                        f.path.0 == format!("input/{}/blobs/{}", source.session.0, blob.sha256.0)
                    })
                    .ok_or(c::Code::SourceUnavailable)?;
                if file.len != blob.len || file.sha256 != blob.sha256 {
                    return Err(c::Code::BlobMismatch);
                }
            }
            Ok(())
        })?;
        let session = source
            .session
            .0
            .parse()
            .map(SessionId)
            .map_err(|_| c::Code::BadRequest)?;
        result.logs.0.insert(session, log.take_value());
        retired_laws.push(law.take_value());
    }
    Ok(result.take_value())
}
fn slot_bytes(
    dir: &Dir,
    m: &w::Input,
    slot: u32,
    b: &mut Budget,
    max: u64,
) -> Result<storage::Tracked<Vec<u8>>, c::Code> {
    let file = m
        .evidence
        .files
        .get(slot as usize)
        .ok_or(c::Code::SourceUnavailable)?;
    if file.len.0 <= 8192 {
        return b.track(file.len.0, || {
            let (parent, name, mut input, before) = relative_file_raw(dir, &file.path.0, max)?;
            if before.len() != file.len.0 {
                return Err(c::Code::SourceChanged);
            }
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(file.len.0 as usize)
                .map_err(|_| c::Code::ResourceLimit)?;
            bytes.resize(file.len.0 as usize, 0);
            input
                .read_exact(&mut bytes)
                .map_err(|_| c::Code::SourceChanged)?;
            parent.unchanged(&name, &input, &before)?;
            if w::hash(&bytes) != file.sha256 {
                return Err(c::Code::SourceChanged);
            }
            Ok(bytes)
        });
    }
    let (parent, name, _, _) = relative_file(dir, &file.path.0, max, b)?;
    let bytes = read_bounded(&parent, &name, max, b)?;
    let mut hasher = Sha256::new();
    for chunk in bytes.chunks(16384) {
        b.unit(|| {
            hasher.update(chunk);
            Ok(())
        })?;
    }
    b.unit(|| {
        if bytes.len() as u64 != file.len.0
            || w::Hex(format!("{:x}", hasher.finalize())) != file.sha256
        {
            return Err(c::Code::SourceChanged);
        }
        Ok(())
    })?;
    Ok(bytes)
}
/// Historical domains are derived from exact host-admitted logical bindings,
/// never from a receipt or the worker's HOME. Native sinks remain unavailable.
fn logical_domains(m: &w::Input) -> Result<super::receipt::Domains, c::Code> {
    let mut sessions = BTreeSet::new();
    let mut homes = BTreeSet::new();
    for source in &m.closed_sources {
        for blob in &source.blobs {
            let physical = format!("input/{}/blobs/{}", source.session.0, blob.sha256.0);
            let file = m
                .evidence
                .files
                .iter()
                .find(|f| f.path.0 == physical)
                .ok_or(c::Code::SourceUnavailable)?;
            let tail = format!("/{}/blobs/{}", source.session.0, blob.sha256.0);
            for binding in m.evidence.artifacts.iter().filter(|a| a.file == file.slot) {
                if let Some(prefix) = binding.logical.0.strip_suffix(&tail) {
                    sessions.insert(PathBuf::from(prefix));
                }
            }
        }
    }
    for artifact in &m.evidence.artifacts {
        if let Some((home, _)) = artifact
            .logical
            .0
            .split_once("/.bboxy450-harness/dream/context/")
        {
            homes.insert(PathBuf::from(home));
        }
    }
    if sessions.len() != 1 || homes.is_empty() || homes.len() > 32 {
        return Err(c::Code::SourceUnavailable);
    }
    let sessions = sessions.pop_first().unwrap();
    let home = homes.pop_first().unwrap();
    if !sessions.is_absolute()
        || !home.is_absolute()
        || sessions.components().any(|c| {
            !matches!(
                c,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )
        })
    {
        return Err(c::Code::BadRequest);
    }
    let mut domains = super::receipt::Domains::configured(sessions.clone(), home);
    for home in homes {
        if !home.is_absolute() {
            return Err(c::Code::BadRequest);
        }
        let extra = super::receipt::Domains::configured(sessions.clone(), home);
        domains.sink_roots.extend(extra.sink_roots);
        domains.caches.extend(extra.caches);
    }
    Ok(domains)
}
struct ProofClock {
    budget: Rc<RefCell<Budget>>,
    started: Option<u64>,
    failure: Rc<RefCell<Option<c::Code>>>,
    epoch: Option<u64>,
    epoch_wall: Option<Instant>,
    accounted: u64,
    steps: u16,
    credits: Rc<RefCell<Vec<storage::Credit>>>,
    helper_clock: Option<u64>,
    helper_wall: Option<Instant>,
    helper_summary: Rc<RefCell<HelperSummary>>,
}
impl ProofClock {
    fn remember(&self, code: c::Code) {
        if self.failure.borrow().is_none() {
            *self.failure.borrow_mut() = Some(code);
        }
    }
    /// Close one actual ordinary batch. Tiny primitive edges only observe the
    /// open batch; the canonical owner receives its raw CPU/debt exactly once.
    fn close_batch(&mut self) -> Result<(), super::entry::Invalid> {
        let Some(at) = self.epoch.take() else {
            self.started = None;
            return Ok(());
        };
        let end = cpu_ns();
        let used = self.budget.borrow().control.observe_interval(at, end);
        let wall = self
            .epoch_wall
            .take()
            .map(|at| at.elapsed().as_nanos().min(u64::MAX as u128) as u64)
            .unwrap_or(u64::MAX);
        self.started = None;
        self.accounted = 0;
        self.steps = 0;
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.cost_known &= used != u64::MAX && wall != u64::MAX;
            summary.proof_app_max_cpu = summary.proof_app_max_cpu.max(used);
        }
        let result = self
            .budget
            .borrow_mut()
            .read_charge(used, used, wall, 1_000_000);
        if let Err(code) = result {
            self.remember(code);
            return Err(super::entry::Invalid("selected ordinary batch refused"));
        }
        Ok(())
    }
}
#[cfg(test)]
impl Drop for ProofClock {
    fn drop(&mut self) {
        if Rc::strong_count(&self.helper_summary) == 1 {
            self.helper_summary.borrow().emit();
            eprintln!(
                "dream-read helper_fixture_failure={:?} control={:?}",
                *self.failure.borrow(),
                self.budget.borrow().control.error()
            );
        }
    }
}
#[derive(Default)]
struct HelperSummary {
    cost_known: bool,
    proof_app_max_cpu: u64,
    creations: u64,
    attempts: u64,
    effects: u64,
    failures_with_effect: u64,
    unknown_effects: u64,
    pre_effect_refusals: u64,
    pre_effect_last_sequence: u16,
    last_errno: i32,
    last_failed_sequence: u16,
    native_cpu: u64,
    native_max_cpu: u64,
    native_max_wall: u64,
    inclusive_max_cpu: u64,
    inclusive_max_wall: u64,
    app_max_cpu: u64,
    creation_debt: u64,
    retirement_count: u64,
    retirement_pending_cpu: u64,
    retirement_cpu: u64,
    retirement_max_cpu: u64,
    retirement_inclusive_max_cpu: u64,
    retirement_unknown: u64,
    retirement_conservative_debt: u64,
    #[cfg(test)]
    retirement_last_end: Option<Instant>,
    settled: u64,
    rolled_cpu: u64,
    reaped_rss: u64,
    reap_ticket: Option<HelperTicket>,
    totals: super::c1::HelperMetrics,
}
impl HelperSummary {
    fn emit(&self) {
        eprintln!(
            "dream-read helper_cost profile=read-helper-native/1 cost_known={} proof_app_max_cpu_ns={}",
            self.cost_known, self.proof_app_max_cpu
        );
        eprintln!(
            "dream-read helper_native profile=read-helper-native/1 tag=process_create count={} reservations={} effects={} failures_with_effect={} unknown_effects={} last_errno={} last_failed_sequence={} cpu_ns={} max_cpu_ns={} max_wall_ns={} inclusive_max_cpu_ns={} inclusive_max_wall_ns={} app_max_cpu_ns={} debt_ns={}",
            self.creations,
            self.attempts,
            self.effects,
            self.failures_with_effect,
            self.unknown_effects,
            self.last_errno,
            self.last_failed_sequence,
            self.native_cpu,
            self.native_max_cpu,
            self.native_max_wall,
            self.inclusive_max_cpu,
            self.inclusive_max_wall,
            self.app_max_cpu,
            self.creation_debt
        );
        eprintln!(
            "dream-read helper_retirement profile=read-helper-native/1 tag=application count={} pending_pipe_cpu_ns={} cpu_ns={} max_cpu_ns={} inclusive_max_cpu_ns={} unknown_clocks={} conservative_additional_debt_ns={}",
            self.retirement_count,
            self.retirement_pending_cpu,
            self.retirement_cpu,
            self.retirement_max_cpu,
            self.retirement_inclusive_max_cpu,
            self.retirement_unknown,
            self.retirement_conservative_debt
        );
        eprintln!(
            "dream-read helper_before_effect profile=read-helper-native/1 refusals={} last_reserved_sequence={} native_attempted=false effect=absent",
            self.pre_effect_refusals, self.pre_effect_last_sequence
        );
        let m = &self.totals;
        eprintln!(
            "dream-read helper_lifetime profile=read-helper-native/1 settled={} grants={} charge_ns={} max_charge_ns={} debt_ns={} controller_cpu_ns={} pipe_cpu_ns={} observed_direct_cpu_ns={} observed_descendant_cpu_ns={} rolled_descendant_cpu_ns={} reaped_rollup_cpu_ns={} initial_kernel_cpu_ns={} descendants={} verified_descendants={} coverage_known={} first_failure_line={} first_failure_sample_stage={}",
            self.settled,
            m.grants,
            m.charged_ns,
            m.max_charge_ns,
            m.debt_ns,
            m.controller_cpu_ns,
            m.pipe_cpu_ns,
            m.observed_direct_cpu_ns,
            m.observed_descendant_cpu_ns,
            m.rolled_descendant_cpu_ns,
            self.rolled_cpu,
            m.initial_kernel_cpu_ns,
            m.descendants,
            m.verified_descendants,
            m.coverage_known,
            m.first_failure_line,
            m.first_failure_sample_stage
        );
        let last_stop = m.last_stop;
        let first_stop = m.first_failed_stop;
        eprintln!(
            "dream-read helper_continuity profile=read-helper-native/1 query_fallbacks={} terminal_observations={} last_guard={} last_native_code={}",
            m.continuity_query_fallbacks, m.continuity_terminal_observations,
            m.last_continuity_guard, m.last_continuity_native_code
        );
        for (kind, receipt) in [("last", last_stop), ("first_failed", first_stop)] {
            if receipt.attempt != 0 {
                eprintln!(
                    "dream-read helper_stop_identity kind={} sequence={} pid={} pgid={} birth={} start_abstime={} audit_version={} attempt={} result={} errno={} observed_ns={} injected={}",
                    kind, receipt.sequence, receipt.pid, receipt.pgid, receipt.birth,
                    receipt.start_abstime, receipt.audit_version, receipt.attempt,
                    receipt.result, receipt.rawerrno, receipt.observed_ns, receipt.injected
                );
            }
        }
        eprintln!(
            "dream-read helper_stop profile=read-helper-native/1 attempts={} failures={} last_sequence={} last_pgid={} last_attempt={} last_result={} last_errno={} first_failure_sequence={} first_failure_pgid={} first_failure_attempt={} first_failure_result={} first_failure_errno={}",
            m.stop_attempts,
            m.stop_failures,
            last_stop.sequence,
            last_stop.pgid,
            last_stop.attempt,
            last_stop.result,
            last_stop.rawerrno,
            first_stop.sequence,
            first_stop.pgid,
            first_stop.attempt,
            first_stop.result,
            first_stop.rawerrno
        );
        eprintln!(
            "dream-read helper_memory profile=read-helper-native/1 simultaneous_group_rss_peak_bytes={} sum_separate_rss_peak_bytes={} reaped_rss_high_water_bytes={} mixed_main_heap_plus_group_rss_peak_bytes={} group_rss_limit_bytes={} mixed_limit_bytes={} native_heap_coverage=sampled_rss_not_allocator",
            m.group_rss_peak,
            m.separate_rss_sum,
            self.reaped_rss,
            m.mixed_heap_rss_peak,
            storage::HELPERS,
            96 * 1024 * 1024
        );
        eprintln!(
            "dream-read helper_window_totals profile=read-helper-native/1 decision=65489 window_ns={} active_cpu_ns={} active_control_ns={} roster_closures={} reopened_slots={} unknown_windows={} clock_failures={} post_units={} post_cpu_ns={} post_max_cpu_ns={} late_kernel_cpu_ns={} late_kernel_observations={}",
            m.window_ns,
            m.active_cpu_ns,
            m.active_control_ns,
            m.roster_closures,
            m.reopened_slots,
            m.unknown_windows,
            m.clock_failures,
            m.post_units,
            m.post_cpu_ns,
            m.post_max_cpu_ns,
            m.late_kernel_cpu_ns,
            m.late_kernel_observations
        );
        for (selection, r) in [
            ("worst_q", m.worst_grant),
            ("first_failed_pending_lower_bound", m.first_failed_pending),
        ] {
            if selection != "worst_q" && r.sequence == 0 {
                continue;
            }
            eprintln!(
                "dream-read helper_window_receipt profile=read-helper-native/1 decision=65489 selection={} sequence={} pid={} pgid={} birth={} reserved={} t0_ns={} end0_ns={} end1_ns={} close_ns={} member0_pid={} member1_pid={} member0_birth={} member1_birth={} member0_start={} member1_start={} member0_version={} member1_version={} controller_baseline_ns={} controller_end_ns={} pipe_baseline_ns={} pipe_end_ns={} w_ns={} a_ns={} c_ns={} q_ns={} closed={}",
                selection,
                r.sequence,
                r.pid,
                r.pgid,
                r.birth,
                r.reserved,
                r.t0_ns,
                r.end_ns[0],
                r.end_ns[1],
                r.close_ns,
                r.member_pid[0],
                r.member_pid[1],
                r.member_birth[0],
                r.member_birth[1],
                r.member_start[0],
                r.member_start[1],
                r.member_version[0],
                r.member_version[1],
                r.controller_baseline_ns,
                r.controller_end_ns,
                r.pipe_baseline_ns,
                r.pipe_end_ns,
                r.w_ns,
                r.a_ns,
                r.c_ns,
                r.q_ns,
                r.closed
            );
        }
        let (sequence, pid, pgid, birth) = self.reap_ticket.map_or((0, 0, 0, 0), |t| {
            (t.sequence, t.pid, t.pgid, t.start_identity)
        });
        eprintln!(
            "dream-read helper_reap_receipt profile=read-helper-native/1 selection=latest_reaped sequence={sequence} pid={pid} pgid={pgid} birth={birth} user_ns={} system_ns={} user_upper_ns={} system_upper_ns={} quantization_ns={} ri_user_ticks={} ri_system_ticks={} timebase_numer={} timebase_denom={} reconciliation_unproven={} unexplained_cpu_ns={}",
            m.reap_user_ns,
            m.reap_system_ns,
            m.reap_user_upper_ns,
            m.reap_system_upper_ns,
            m.reap_quantization_ns,
            m.reap_ri_user_ticks,
            m.reap_ri_system_ticks,
            m.reap_timebase_numer,
            m.reap_timebase_denom,
            m.reconciliation_unproven,
            m.unexplained_cpu_ns
        );
    }
}
/// Unit witnesses wrap this real adapter rather than substituting a test
/// scheduler. The controller and owned native-helper path remain production code.
#[cfg(test)]
pub(crate) fn helper_fixture_clock(control: Arc<Control>) -> Box<dyn super::resolve::ProofPace> {
    helper_tests::fixture_priority();
    let b = Rc::new(RefCell::new(budget(
        control.clone(),
        Arc::new(crossbeam_queue::ArrayQueue::new(32)),
    )));
    let summary = Rc::new(RefCell::new(HelperSummary::default()));
    summary.borrow_mut().cost_known = true;
    summary.borrow_mut().totals.coverage_known = true;
    let credits = Rc::new(RefCell::new(vec![
        control
            .memory
            .reserve(11 * 1024 * 1024)
            .expect("shared helper/cache credit"),
    ]));
    Box::new(ProofClock {
        budget: b,
        started: None,
        failure: Rc::new(RefCell::new(None)),
        epoch: None,
        epoch_wall: None,
        accounted: 0,
        steps: 0,
        credits,
        helper_clock: None,
        helper_wall: None,
        helper_summary: summary,
    })
}
impl super::resolve::ProofPace for ProofClock {
    fn helper_refused(&mut self, reason: &'static str) {
        let code = if reason.starts_with("resource_limit:") {
            c::Code::ResourceLimit
        } else if reason.starts_with("source_changed:") {
            c::Code::SourceChanged
        } else {
            c::Code::SourceUnavailable
        };
        if self.failure.borrow().is_none() {
            *self.failure.borrow_mut() = Some(self.budget.borrow().control.error().unwrap_or(code));
        }
    }
    fn helper_before_effect_refused(
        &mut self,
        sequence: u16,
        credits: u16,
        _cpu_ns: u64,
        wall_ns: u64,
    ) -> Result<(), super::entry::Invalid> {
        let _ = self.close_batch();
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.cost_known = false;
            summary.proof_app_max_cpu = u64::MAX;
            summary.attempts += u64::from(credits);
            summary.pre_effect_refusals += 1;
            summary.pre_effect_last_sequence = sequence;
        }
        let control = {
            let mut budget = self.budget.borrow_mut();
            let _ = budget.read_charge(u64::MAX, u64::MAX, wall_ns, 1_000_000);
            budget.control.clone()
        };
        self.remember(control.error().unwrap_or(c::Code::ResourceLimit));
        control.read_pay();
        Err(super::entry::Invalid(
            "selected helper beginning clock unavailable",
        ))
    }
    fn native_created(
        &mut self,
        span: super::resolve::NativeCreation,
    ) -> Result<(), super::entry::Invalid> {
        // Normally suspend already closed this batch. A failed caller still
        // cannot discard either its pending application cost or native effect.
        let _ = self.close_batch();
        let known = [
            span.cpu_ns,
            span.wall_ns,
            span.inclusive_cpu_ns,
            span.inclusive_wall_ns,
            span.app_cpu_ns,
        ]
        .into_iter()
        .all(|v| v != u64::MAX);
        let debt = span.inclusive_cpu_ns.max(1_000_000).checked_mul(9);
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.cost_known &= known;
            summary.creations += 1;
            summary.attempts += span.credits as u64;
            summary.effects += u64::from(span.identity.is_some());
            summary.native_cpu = summary.native_cpu.saturating_add(span.cpu_ns);
            summary.native_max_cpu = summary.native_max_cpu.max(span.cpu_ns);
            summary.native_max_wall = summary.native_max_wall.max(span.wall_ns);
            summary.inclusive_max_cpu = summary.inclusive_max_cpu.max(span.inclusive_cpu_ns);
            summary.inclusive_max_wall = summary.inclusive_max_wall.max(span.inclusive_wall_ns);
            summary.app_max_cpu = summary.app_max_cpu.max(span.app_cpu_ns);
            summary.creation_debt = summary
                .creation_debt
                .saturating_add(debt.unwrap_or(u64::MAX));
        }
        let (control, charged) = {
            let mut b = self.budget.borrow_mut();
            let charged = b.read_charge(
                span.inclusive_cpu_ns,
                span.app_cpu_ns,
                span.inclusive_wall_ns,
                3_000_000,
            );
            (b.control.clone(), charged)
        };
        let code = if !known || debt.is_none() {
            Some(c::Code::SourceUnavailable)
        } else if span.cpu_ns > 2_000_000
            || span.wall_ns > 25_000_000
            || span.inclusive_cpu_ns > 3_000_000
            || span.app_cpu_ns > 1_000_000
        {
            Some(c::Code::ResourceLimit)
        } else if span.rawerrno != 0 || span.identity.is_none() {
            Some(c::Code::SourceUnavailable)
        } else {
            None
        };
        if code.is_some() {
            let mut summary = self.helper_summary.borrow_mut();
            summary.last_errno = span.rawerrno;
            summary.last_failed_sequence = span.sequence;
            summary.failures_with_effect += u64::from(span.identity.is_some());
            summary.unknown_effects += u64::from(span.identity.is_none() && span.rawerrno == 0);
        }
        // The typed suspended effect remains attached while canonical shared
        // debt is paid, including failed native/cancel/clock verdicts.
        if code == Some(c::Code::ResourceLimit) {
            let _ = control
                .cancel
                .compare_exchange(0, 4, Ordering::AcqRel, Ordering::Acquire);
        }
        control.read_pay();
        if let Some(code) = code
            .or_else(|| *self.failure.borrow())
            .or_else(|| charged.err())
            .or_else(|| control.error())
        {
            *self.failure.borrow_mut() = Some(code);
            return Err(super::entry::Invalid("selected native creation refused"));
        }
        if let Some(identity) = span.identity {
            let mut helper = control
                .helper
                .lock()
                .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
            let h = helper
                .as_mut()
                .filter(|h| {
                    h.pgid == identity.pgid
                        && h.identity.sequence == identity.sequence
                        && h.identity.start_identity == identity.start_identity
                })
                .ok_or(super::entry::Invalid("helper native custody mismatch"))?;
            // The controller owns initial-kernel corroboration and its fresh
            // ordinary debt. Open the single IO handoff before grants become ready.
            h.settle_creation(&control)
                .map_err(|_| super::entry::Invalid("helper creation handoff refused"))?;
            h.begin_pipe()
                .map_err(|_| super::entry::Invalid("helper IO handoff already open"))?;
            self.helper_clock = Some(cpu_ns());
            self.helper_wall = Some(Instant::now());
            if self.helper_clock == Some(u64::MAX) {
                self.helper_summary.borrow_mut().cost_known = false;
                self.remember(c::Code::SourceUnavailable);
                return Err(super::entry::Invalid(
                    "helper IO beginning clock unavailable",
                ));
            }
            h.ready = true;
        }
        Ok(())
    }
    fn boundary(&mut self) -> Result<(), super::entry::Invalid> {
        super::resolve::ProofPace::after(self)?;
        self.close_batch()?;
        super::resolve::ProofPace::before(self)
    }

    fn reserve(&mut self, bytes: u64) -> Result<(), super::entry::Invalid> {
        let credit = self
            .budget
            .borrow()
            .control
            .memory
            .reserve(bytes)
            .map_err(|code| {
                *self.failure.borrow_mut() = Some(code);
                super::entry::Invalid("selected storage reservation")
            })?;
        self.credits.borrow_mut().push(credit);
        Ok(())
    }
    fn helper_usage(
        &mut self,
        ticket: HelperTicket,
        user_ns: u64,
        system_ns: u64,
        rss_bytes: u64,
    ) -> Result<(), super::entry::Invalid> {
        let at = cpu_ns();
        let wall_at = Instant::now();
        let control = self.budget.borrow().control.clone();
        let mut helper = control
            .helper
            .lock()
            .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
        let h = helper
            .as_mut()
            .filter(|h| h.pgid == ticket.pgid && h.identity.ticket() == ticket)
            .ok_or(super::entry::Invalid("helper usage custody"))?;
        let measured = h.reconcile_usage(&control, user_ns, system_ns, rss_bytes);
        {
            let mut summary = self.helper_summary.borrow_mut();
            let cpu = user_ns
                .checked_add(system_ns)
                .filter(|_| user_ns != u64::MAX && system_ns != u64::MAX);
            summary.cost_known &= cpu.is_some();
            summary.rolled_cpu = cpu
                .and_then(|cpu| summary.rolled_cpu.checked_add(cpu))
                .unwrap_or(u64::MAX);
            summary.reaped_rss = summary.reaped_rss.max(rss_bytes);
        }
        let used = control.observe_interval(at, cpu_ns());
        let wall = u64::try_from(wall_at.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let recorded = self
            .budget
            .borrow_mut()
            .read_charge(used, used, wall, 1_000_000);
        // The full observation belongs to this existing proof owner. A controller
        // cannot commit its newly observed late CPU before this endpoint.
        let committed = h.commit_post_observation(&control, used, wall);
        let code = control.error();
        drop(helper);
        control.read_pay();
        if measured.is_err() || committed.is_err() || recorded.is_err() {
            self.remember(
                code.or(recorded.err())
                    .unwrap_or(c::Code::SourceUnavailable),
            );
            return Err(super::entry::Invalid("helper usage accounting refused"));
        }
        Ok(())
    }

    fn helper_attach(
        &mut self,
        identity: super::resolve::HelperIdentity,
    ) -> Result<(), super::entry::Invalid> {
        let b = self.budget.borrow();
        let mut helper = b.control.helper.lock().unwrap_or_else(|poisoned| {
            b.control.cancel.store(4, Ordering::Release);
            poisoned.into_inner()
        });
        if identity.pgid <= 0
            || identity.pid != identity.pgid
            || identity.sequence == 0
            || !(1..=2).contains(&identity.expected_members)
            || helper.is_some()
        {
            return Err(super::entry::Invalid("helper owner limit"));
        }
        *helper = Some(super::c1::HelperGroup::attached(identity));
        let waiting = Instant::now();
        loop {
            let h = helper.as_mut().unwrap();
            match h.sample(&b.control) {
                Ok(true) => break,
                Ok(false)
                    if h.initial_stop_pending
                        && waiting.elapsed() < Duration::from_millis(25)
                        && b.control.error().is_none() =>
                {
                    // START_SUSPENDED publishes BSD SSTOP before its kernel
                    // tail is quiescent. No SIGSTOP substitute and no user CPU
                    // is admitted; release control custody while waiting.
                    drop(helper);
                    std::thread::sleep(Duration::from_micros(100));
                    helper = b.control.helper.lock().unwrap_or_else(|p| p.into_inner());
                }
                _ => {
                    h.ready = false;
                    *self.failure.borrow_mut() =
                        Some(b.control.error().unwrap_or(c::Code::SourceUnavailable));
                    return Err(super::entry::Invalid("helper was not born stopped"));
                }
            }
        }
        Ok(())
    }
    fn helper_retire(&mut self, ticket: HelperTicket) -> Result<(), super::entry::Invalid> {
        let at = cpu_ns();
        let wall_at = Instant::now();
        let control = self.budget.borrow().control.clone();
        let mut helper = control.helper.lock().unwrap_or_else(|poisoned| {
            let _ = control
                .cancel
                .compare_exchange(0, 4, Ordering::AcqRel, Ordering::Acquire);
            poisoned.into_inner()
        });
        let h = helper
            .as_mut()
            .filter(|h| h.identity.ticket() == ticket)
            .ok_or(super::entry::Invalid("helper custody mismatch"))?;
        if let Some(code) = control.error() {
            self.remember(code);
        }
        if h.retiring {
            return Ok(());
        }
        h.ready = false; // revoke future CONT before releasing controller custody
        let previous = self.helper_clock.take();
        let previous_wall = self.helper_wall.take();
        let had_pipe = h.pipe_is_open();
        let orphan = !had_pipe && previous.is_some();
        if orphan {
            h.metrics.coverage_known = false;
        }
        let pending = if had_pipe || orphan {
            previous
                .map(|start| control.observe_interval(start, at))
                .unwrap_or(u64::MAX)
        } else {
            0
        };
        let pending_wall = if had_pipe || orphan {
            previous_wall
                .and_then(|start| wall_at.checked_duration_since(start))
                .and_then(|span| u64::try_from(span.as_nanos()).ok())
                .unwrap_or(u64::MAX)
        } else {
            0
        };
        if had_pipe {
            let _ = h.account_pipe(&control, pending, pending_wall);
        }
        h.begin_pipe()
            .map_err(|_| super::entry::Invalid("retirement owner scope already open"))?;
        drop(helper);
        // This existing owner reports its whole stop-wait prefix before the
        // controller commits Q. Wait only for physical closure, not that commit.
        loop {
            let helper = control.helper.lock().unwrap_or_else(|p| p.into_inner());
            let h = helper
                .as_ref()
                .filter(|h| h.identity.ticket() == ticket)
                .ok_or(super::entry::Invalid("helper retirement custody changed"))?;
            let stop = h.closure_observed()
                || control.error().is_some()
                || self.failure.borrow().is_some();
            drop(helper);
            if stop {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let prefix_end = cpu_ns();
        let prefix_wall = u64::try_from(wall_at.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let prefix = control.observe_interval(at, prefix_end);
        let mut helper = control.helper.lock().unwrap_or_else(|p| p.into_inner());
        let h = helper
            .as_mut()
            .filter(|h| h.identity.ticket() == ticket)
            .ok_or(super::entry::Invalid("helper retirement custody changed"))?;
        // An already stopped failed owner needs no further proof admission.
        // Keep its adjacent scalar prefix and exact retirement effects inside
        // ONE ordinary unit, rather than inventing an empty paced work unit
        // between them. Running/open-window and pending-IO paths stay split.
        let merged_retirement = !had_pipe
            && !orphan
            && h.closure_observed()
            && (control.error().is_some() || self.failure.borrow().is_some());
        if !merged_retirement {
            let _ = h.account_pipe(&control, prefix, prefix_wall);
        }
        drop(helper);
        if had_pipe || orphan {
            let _ = self
                .budget
                .borrow_mut()
                .read_charge(pending, pending, pending_wall, 1_000_000);
        }
        let prefix_recorded = if merged_retirement {
            Ok(())
        } else {
            self.budget
                .borrow_mut()
                .read_charge(prefix, prefix, prefix_wall, 1_000_000)
        };
        if !merged_retirement {
            control.read_pay();
        }
        // On a valid path finish closure/corroboration and pay every additive
        // debt. A failed path retains open windows until actual reap/quarantine.
        if orphan {
            self.remember(control.error().unwrap_or(c::Code::SourceUnavailable));
        }
        loop {
            if control.error().is_some() || self.failure.borrow().is_some() {
                break;
            }
            control.read_pay();
            let helper = control.helper.lock().unwrap_or_else(|p| p.into_inner());
            let h = helper
                .as_ref()
                .filter(|h| h.identity.ticket() == ticket)
                .ok_or(super::entry::Invalid("helper retirement custody changed"))?;
            let pending =
                h.pending_closure() || h.pending_corroboration() || Instant::now() < h.debt_until;
            drop(helper);
            if !pending {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let signal_at = if merged_retirement { at } else { cpu_ns() };
        let signal_wall = if merged_retirement {
            wall_at
        } else {
            Instant::now()
        };
        let mut helper = control.helper.lock().unwrap_or_else(|p| p.into_inner());
        let h = helper
            .as_mut()
            .filter(|h| h.identity.ticket() == ticket)
            .ok_or(super::entry::Invalid("helper retirement custody changed"))?;
        if !merged_retirement {
            h.begin_pipe()
                .map_err(|_| super::entry::Invalid("retirement signal scope already open"))?;
        }
        h.retiring = true; // serialized handoff BEFORE final signals or any reap
        let effects = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| h.retire_members()));
        if !matches!(effects, Ok(Ok(()))) {
            h.metrics.coverage_known = false;
            self.remember(c::Code::SourceUnavailable);
        }
        let signal_end = cpu_ns();
        let signal_ended = Instant::now();
        let signal_cpu = control.observe_interval(signal_at, signal_end);
        let signal_wall_ns = signal_ended
            .checked_duration_since(signal_wall)
            .and_then(|span| u64::try_from(span.as_nanos()).ok())
            .unwrap_or(u64::MAX);
        let attributed = h.account_pipe(&control, signal_cpu, signal_wall_ns);
        drop(helper);
        let recorded =
            self.budget
                .borrow_mut()
                .read_charge(signal_cpu, signal_cpu, signal_wall_ns, 1_000_000);
        self.helper_clock = None;
        self.helper_wall = None; // no IO was admitted after retirement
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.retirement_count = summary.retirement_count.saturating_add(1);
            summary.retirement_pending_cpu = summary.retirement_pending_cpu.saturating_add(pending);
            summary.retirement_cpu = summary
                .retirement_cpu
                .saturating_add(if merged_retirement { 0 } else { prefix })
                .saturating_add(signal_cpu);
            summary.retirement_max_cpu = summary.retirement_max_cpu.max(prefix).max(signal_cpu);
            summary.retirement_inclusive_max_cpu = summary
                .retirement_inclusive_max_cpu
                .max(prefix)
                .max(signal_cpu)
                .max(pending);
            let known = [pending, prefix, signal_cpu, prefix_wall, signal_wall_ns]
                .into_iter()
                .all(|value| value != u64::MAX);
            summary.cost_known &= known;
            summary.retirement_unknown =
                summary.retirement_unknown.saturating_add(u64::from(!known));
            #[cfg(test)]
            {
                summary.retirement_last_end = Some(signal_ended);
            }
        }
        control.read_pay();
        let failure = *self.failure.borrow();
        if let Some(code) = failure
            .or_else(|| control.error())
            .or(recorded.err())
            .or(prefix_recorded.err())
        {
            self.remember(code);
            return Err(super::entry::Invalid("helper retirement refused"));
        }
        if attributed.is_err() {
            self.remember(c::Code::SourceUnavailable);
            return Err(super::entry::Invalid(
                "helper retirement accounting refused",
            ));
        }
        Ok(())
    }

    fn helper_detach(&mut self, ticket: HelperTicket) -> Result<(), super::entry::Invalid> {
        let b = self.budget.borrow();
        let (mut helper, poisoned) = match b.control.helper.lock() {
            Ok(helper) => (helper, false),
            Err(poisoned) => (poisoned.into_inner(), true),
        };
        if !helper
            .as_ref()
            .is_some_and(|h| h.pgid == ticket.pgid && h.identity.ticket() == ticket && h.retiring)
        {
            return Err(super::entry::Invalid("helper settlement mismatch"));
        }
        if poisoned {
            let _ = b
                .control
                .cancel
                .compare_exchange(0, 4, Ordering::AcqRel, Ordering::Acquire);
        }
        loop {
            if helper
                .as_ref()
                .is_some_and(|h| h.retirement_settlement_observed())
            {
                // helper_usage already observed exact physical exit after
                // actual leader wait4/group disappearance. Keep that monotonic
                // fact and its paid debt; do not create another observation.
                break;
            }
            // Unknown physical exit retains this original owner/slot. Status
            // and cancellation remain on the independent control owner; no
            // further work, output certification or PID-based signal occurs.
            drop(helper);
            let mut exited = false;
            cleanup_unit(&b.control, || {
                let mut current = b.control.helper.lock().unwrap_or_else(|p| p.into_inner());
                exited = current.as_mut().is_some_and(|h| {
                    h.pgid == ticket.pgid
                        && h.identity.ticket() == ticket
                        && h.retiring
                        && h.settle_quarantine(&b.control)
                });
            });
            helper = b.control.helper.lock().unwrap_or_else(|p| p.into_inner());
            if !helper.as_ref().is_some_and(|h| {
                h.pgid == ticket.pgid && h.identity.ticket() == ticket && h.retiring
            }) {
                return Err(super::entry::Invalid("helper quarantine custody changed"));
            }
            if exited {
                break;
            }
        }
        let mut h = helper.take().unwrap();
        drop(helper);
        // All expensive owners remain parked through residual/reconciled debt.
        while Instant::now() < h.debt_until {
            std::thread::sleep(
                h.debt_until
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(1)),
            );
        }
        if h.identity.expected_members == 2
            && (h.metrics.descendants != 1
                || h.metrics.verified_descendants != 1
                || h.metrics.rolled_descendant_cpu_ns < h.metrics.observed_descendant_cpu_ns)
        {
            h.metrics.coverage_known = false;
            if self.failure.borrow().is_none() {
                *self.failure.borrow_mut() =
                    Some(b.control.error().unwrap_or(c::Code::SourceUnavailable));
            }
        }
        let mut summary = self.helper_summary.borrow_mut();
        summary.cost_known &= h.metrics.clock_failures == 0
            && h.metrics.charged_ns != u64::MAX
            && h.metrics.controller_cpu_ns != u64::MAX
            && h.metrics.pipe_cpu_ns != u64::MAX
            && h.metrics.unexplained_cpu_ns != u64::MAX;
        summary.settled += 1;
        summary.reap_ticket = Some(ticket);
        let m = &mut summary.totals;
        m.reap_user_ns = h.metrics.reap_user_ns;
        m.reap_system_ns = h.metrics.reap_system_ns;
        m.reap_user_upper_ns = h.metrics.reap_user_upper_ns;
        m.reap_system_upper_ns = h.metrics.reap_system_upper_ns;
        m.reap_quantization_ns = h.metrics.reap_quantization_ns;
        m.reap_ri_user_ticks = h.metrics.reap_ri_user_ticks;
        m.reap_ri_system_ticks = h.metrics.reap_ri_system_ticks;
        m.reap_timebase_numer = h.metrics.reap_timebase_numer;
        m.reap_timebase_denom = h.metrics.reap_timebase_denom;
        m.reconciliation_unproven = m
            .reconciliation_unproven
            .saturating_add(h.metrics.reconciliation_unproven);
        m.unexplained_cpu_ns = m
            .unexplained_cpu_ns
            .saturating_add(h.metrics.unexplained_cpu_ns);
        if h.metrics.worst_grant.q_ns > m.worst_grant.q_ns {
            m.worst_grant = h.metrics.worst_grant;
        }
        if m.first_failed_pending.sequence == 0 {
            m.first_failed_pending = h.metrics.first_failed_pending;
        }
        m.window_ns = m.window_ns.saturating_add(h.metrics.window_ns);
        m.active_cpu_ns = m.active_cpu_ns.saturating_add(h.metrics.active_cpu_ns);
        m.active_control_ns = m
            .active_control_ns
            .saturating_add(h.metrics.active_control_ns);
        m.roster_closures = m.roster_closures.saturating_add(h.metrics.roster_closures);
        m.reopened_slots = m.reopened_slots.saturating_add(h.metrics.reopened_slots);
        m.unknown_windows = m.unknown_windows.saturating_add(h.metrics.unknown_windows);
        m.clock_failures = m.clock_failures.saturating_add(h.metrics.clock_failures);
        m.post_units = m.post_units.saturating_add(h.metrics.post_units);
        m.post_cpu_ns = m.post_cpu_ns.saturating_add(h.metrics.post_cpu_ns);
        m.post_max_cpu_ns = m.post_max_cpu_ns.max(h.metrics.post_max_cpu_ns);
        m.late_kernel_cpu_ns = m
            .late_kernel_cpu_ns
            .saturating_add(h.metrics.late_kernel_cpu_ns);
        m.late_kernel_observations = m
            .late_kernel_observations
            .saturating_add(h.metrics.late_kernel_observations);
        m.grants = m.grants.saturating_add(h.metrics.grants);
        m.charged_ns = m.charged_ns.saturating_add(h.metrics.charged_ns);
        m.max_charge_ns = m.max_charge_ns.max(h.metrics.max_charge_ns);
        m.debt_ns = m.debt_ns.saturating_add(h.metrics.debt_ns);
        m.controller_cpu_ns = m
            .controller_cpu_ns
            .saturating_add(h.metrics.controller_cpu_ns);
        m.pipe_cpu_ns = m.pipe_cpu_ns.saturating_add(h.metrics.pipe_cpu_ns);
        m.observed_direct_cpu_ns = m
            .observed_direct_cpu_ns
            .saturating_add(h.metrics.observed_direct_cpu_ns);
        m.observed_descendant_cpu_ns = m
            .observed_descendant_cpu_ns
            .saturating_add(h.metrics.observed_descendant_cpu_ns);
        m.rolled_descendant_cpu_ns = m
            .rolled_descendant_cpu_ns
            .saturating_add(h.metrics.rolled_descendant_cpu_ns);
        m.group_rss_peak = m.group_rss_peak.max(h.metrics.group_rss_peak);
        m.separate_rss_sum = m.separate_rss_sum.max(h.metrics.separate_rss_sum);
        m.mixed_heap_rss_peak = m.mixed_heap_rss_peak.max(h.metrics.mixed_heap_rss_peak);
        m.descendants = m.descendants.saturating_add(h.metrics.descendants);
        m.verified_descendants = m
            .verified_descendants
            .saturating_add(h.metrics.verified_descendants);
        m.initial_kernel_cpu_ns = m
            .initial_kernel_cpu_ns
            .saturating_add(h.metrics.initial_kernel_cpu_ns);
        m.stop_attempts = m.stop_attempts.saturating_add(h.metrics.stop_attempts);
        m.stop_failures = m.stop_failures.saturating_add(h.metrics.stop_failures);
        m.continuity_query_fallbacks = m
            .continuity_query_fallbacks
            .saturating_add(h.metrics.continuity_query_fallbacks);
        m.continuity_terminal_observations = m
            .continuity_terminal_observations
            .saturating_add(h.metrics.continuity_terminal_observations);
        if h.metrics.last_continuity_guard != 0 {
            m.last_continuity_guard = h.metrics.last_continuity_guard;
            m.last_continuity_native_code = h.metrics.last_continuity_native_code;
        }
        if h.metrics.last_stop.attempt != 0 {
            m.last_stop = h.metrics.last_stop;
        }
        if m.first_failed_stop.attempt == 0 {
            m.first_failed_stop = h.metrics.first_failed_stop;
        }
        if m.first_failure_line == 0 {
            m.first_failure_line = h.metrics.first_failure_line;
            m.first_failure_sample_stage = h.metrics.first_failure_sample_stage;
        }
        m.coverage_known &= h.metrics.coverage_known;
        self.helper_clock = None;
        self.helper_wall = None;
        Ok(())
    }
    fn helper_poll(&mut self, ticket: HelperTicket) -> Result<(), super::entry::Invalid> {
        let control = self.budget.borrow().control.clone();
        let mut helper = control
            .helper
            .lock()
            .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
        let h = helper
            .as_mut()
            .filter(|h| h.identity.ticket() == ticket && !h.retiring)
            .ok_or(super::entry::Invalid("helper custody mismatch"))?;
        // Validate the full ticket before consuming either admitted clock.
        let now = cpu_ns();
        let ended = Instant::now();
        let previous = self.helper_clock.take();
        let wall_at = self.helper_wall.take();
        let open = h.pipe_is_open();
        let used = previous
            .map(|at| control.observe_interval(at, now))
            .unwrap_or(u64::MAX);
        let wall = wall_at
            .and_then(|at| ended.checked_duration_since(at))
            .and_then(|span| u64::try_from(span.as_nanos()).ok())
            .unwrap_or(u64::MAX);
        let attributed = if open {
            h.account_pipe(&control, used, wall)
        } else {
            Ok(())
        };
        drop(helper);
        let orphan = !open && previous.is_some();
        let recorded = if open || orphan {
            self.budget
                .borrow_mut()
                .read_charge(used, used, wall, 1_000_000)
        } else {
            Ok(())
        };
        control.read_pay();
        if orphan || attributed.is_err() || recorded.is_err() {
            self.remember(
                control
                    .error()
                    .or(recorded.err())
                    .unwrap_or(c::Code::SourceUnavailable),
            );
            return Err(super::entry::Invalid("helper IO accounting refused"));
        }
        loop {
            let failure = *self.failure.borrow();
            if let Some(code) = failure.or_else(|| control.error()) {
                self.remember(code);
                return Err(super::entry::Invalid("helper stopped"));
            }
            control.read_pay();
            let mut helper = control
                .helper
                .lock()
                .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
            let h = helper
                .as_mut()
                .filter(|h| h.identity.ticket() == ticket && !h.retiring)
                .ok_or(super::entry::Invalid("helper custody mismatch"))?;
            let blocked = (h.pending_closure() && !h.running)
                || h.pending_corroboration()
                || Instant::now() < h.debt_until
                || control.foreground.load(Ordering::Acquire)
                || !h.ready;
            if !blocked {
                h.begin_pipe()
                    .map_err(|_| super::entry::Invalid("helper IO handoff already open"))?;
                self.helper_clock = Some(cpu_ns());
                self.helper_wall = Some(Instant::now());
                if self.helper_clock == Some(u64::MAX) {
                    h.ready = false;
                    self.helper_summary.borrow_mut().cost_known = false;
                    self.remember(c::Code::SourceUnavailable);
                    return Err(super::entry::Invalid(
                        "helper IO beginning clock unavailable",
                    ));
                }
                return Ok(());
            }
            drop(helper);
            // Only independent scalar admission/control checks run while parked;
            // source/identity/roster proof remains on the measured controller.
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn cleanup(&mut self, step: &mut dyn FnMut() -> bool) {
        let running = self.started.is_some();
        let _ = super::resolve::ProofPace::after(self);
        let _ = self.close_batch();
        let control = self.budget.borrow().control.clone();
        let mut more = true;
        while more {
            cleanup_unit(&control, || {
                let start = cpu_ns();
                control.observe_interval(start, start);
                for _ in 0..1024 {
                    more = step();
                    if !more || control.observe_interval(start, cpu_ns()) >= 400_000 {
                        break;
                    }
                }
            });
        }
        if running {
            let _ = super::resolve::ProofPace::before(self);
        }
        // Cleanup uses its independent clock even after a failed proof clock.
        if control.cleanup_max.load(Ordering::Acquire) > 1_000_000 {
            *self.failure.borrow_mut() = Some(c::Code::ResourceLimit);
        }
    }

    fn before(&mut self) -> Result<(), super::entry::Invalid> {
        if self.failure.borrow().is_some() {
            let _ = self.close_batch();
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        let control = self.budget.borrow().control.clone();
        if control.helper.lock().map_or(true, |h| h.is_some()) {
            let _ = self.close_batch();
            self.remember(c::Code::SourceUnavailable);
            return Err(super::entry::Invalid("main proof overlaps helper owner"));
        }
        let foreground = control.foreground.load(Ordering::Acquire);
        if self.epoch.is_none() || self.accounted >= 400_000 || self.steps >= 64 || foreground {
            self.close_batch()?;
            let admitted = self.budget.borrow_mut().read_begin();
            let (at, wall) = admitted.map_err(|code| {
                self.remember(code);
                super::entry::Invalid("selected worker stopped")
            })?;
            self.epoch = Some(at);
            self.epoch_wall = Some(wall);
            self.accounted = 0;
            self.steps = 0;
        } else if let Some(code) = control.error() {
            let _ = self.close_batch();
            self.remember(code);
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        let start = cpu_ns();
        if control.observe_interval(self.epoch.unwrap_or(u64::MAX), start) == u64::MAX {
            self.helper_summary.borrow_mut().cost_known = false;
            self.remember(c::Code::ResourceLimit);
            let _ = self.close_batch();
            return Err(super::entry::Invalid(
                "selected primitive clock unavailable",
            ));
        }
        self.started = Some(start);
        Ok(())
    }
    fn after(&mut self) -> Result<(), super::entry::Invalid> {
        if self.started.take().is_none() {
            return Ok(());
        }
        let end = cpu_ns();
        let at = self.epoch.unwrap_or(u64::MAX);
        let used = self.budget.borrow().control.observe_interval(at, end);
        self.steps = self.steps.saturating_add(1);
        self.accounted = used;
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.cost_known &= used != u64::MAX;
            summary.proof_app_max_cpu = summary.proof_app_max_cpu.max(used);
        }
        let (code, heap_over) = {
            let mut b = self.budget.borrow_mut();
            b.heap_peak = b.heap_peak.max(heaps().0);
            (b.control.error(), b.heap_peak > 96 * 1024 * 1024)
        };
        if used > 1_000_000 || heap_over {
            self.remember(c::Code::ResourceLimit);
            let _ = self.close_batch();
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        if let Some(code) = code {
            self.remember(code);
            let _ = self.close_batch();
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        if used >= 400_000 || self.steps >= 64 {
            self.close_batch()?;
        }
        Ok(())
    }
    fn suspend(&mut self) -> Result<(), super::entry::Invalid> {
        let observed = self.after();
        let closed = self.close_batch();
        // The canonical shared debt starts at actual closure and is paid before
        // a native admission; no earlier idle or separate fixed WAIT is credit.
        self.budget.borrow().control.read_pay();
        observed.and(closed)
    }
}
/// Selected primitives finish synchronously under their own paced checkpoints.
/// A pending dependency is unsupported; no hidden runtime or helper pool is created.
fn selected_ready<T>(
    future: impl std::future::Future<Output = T>,
) -> Result<T, super::entry::Invalid> {
    let mut future = std::pin::pin!(future);
    let waker = futures_util::task::noop_waker();
    let mut context = std::task::Context::from_waker(&waker);
    match future.as_mut().poll(&mut context) {
        std::task::Poll::Ready(value) => Ok(value),
        std::task::Poll::Pending => Err(super::entry::Invalid(
            "selected asynchronous dependency unavailable",
        )),
    }
}
fn prefixed_hex(digest: &str) -> Result<w::Hex, c::Code> {
    digest
        .strip_prefix("sha256:")
        .ok_or(c::Code::ProtocolError)?
        .to_owned()
        .try_into()
        .map_err(|_| c::Code::ProtocolError)
}
fn compute(
    path: PathBuf,
    start: c::Start,
    control: Arc<Control>,
    progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
) -> Result<Completed, w::Error> {
    let b = Rc::new(RefCell::new(budget(control.clone(), progress)));
    let mut proof_error = None;
    let helper_summary = Rc::new(RefCell::new(HelperSummary::default()));
    helper_summary.borrow_mut().cost_known = true;
    helper_summary.borrow_mut().totals.coverage_known = true;
    let mut stage = "admit";
    let wall = Instant::now();
    let mut source_index = None;
    let result = (|| -> Result<Completed, c::Code> {
        low_priority()?;
        let dir = b.borrow_mut().unit(|| Dir::open(&path))?;
        let scratch = b.borrow_mut().unit(|| {
            use std::os::fd::AsRawFd;
            let parent = dir.inventory_fd()?;
            if unsafe { libc::mkdirat(parent.as_raw_fd(), c"proof-scratch".as_ptr(), 0o700) } != 0 {
                return Err(c::Code::StorageUnavailable);
            }
            let _owned = dir.dir("proof-scratch")?;
            Ok(path.join("proof-scratch"))
        })?;
        let job_before = b.borrow_mut().unit(|| {
            dir.inventory_fd()?
                .metadata()
                .map_err(|_| c::Code::SourceChanged)
        })?;
        let input = b.borrow_mut().unit(|| dir.dir("input"))?;
        let output = b.borrow_mut().unit(|| dir.dir("output"))?;
        let raw = read_bounded(
            &input,
            "manifest.json",
            w::MAX_FRAME as u64,
            &mut b.borrow_mut(),
        )?;
        let shape = b
            .borrow_mut()
            .unit(|| storage::preflight(&raw, storage::WORK))?;
        let raw_len = raw.len();
        let mut manifest = b.borrow_mut().track(shape.typed(raw.len())?, || {
            if w::hash(&*raw) != start.manifest_sha256 {
                return Err(c::Code::SourceMismatch);
            }
            w::decode::<w::Input>(&raw).map_err(internal)
        })?;
        let element = [
            std::mem::size_of::<w::Source>(),
            std::mem::size_of::<w::RecordPin>(),
            std::mem::size_of::<w::BundleFile>(),
            std::mem::size_of::<w::RootBinding>(),
            std::mem::size_of::<w::KeyBinding>(),
            std::mem::size_of::<w::ArtifactBinding>(),
            std::mem::size_of::<c::Blob>(),
        ]
        .into_iter()
        .max()
        .unwrap() as u64;
        let retained = (shape.values + shape.keys)
            .checked_mul(element)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add((raw_len as u64).checked_mul(4)?))
            .and_then(|n| n.checked_add(4 * std::mem::size_of::<w::Input>() as u64))
            .ok_or(c::Code::ResourceLimit)?;
        // Release only the now-destroyed parser overlap; retain concrete DTO
        // capacities, raw/string copies and old/new Vec growth coverage.
        manifest.credit.shrink(manifest.credit.bytes.min(retained));
        drop(raw);
        let manifest = Paced::new(manifest, &control, Drain::drain_step);
        if manifest.nonce != start.nonce {
            return Err(c::Code::SourceMismatch);
        }
        let custody = Paced::new(
            capture_custody(&input, &manifest.evidence.files, &mut b.borrow_mut())?,
            &control,
            Drain::drain_step,
        );
        stage = "inventory";
        inventory(&dir, &manifest, &mut b.borrow_mut())?;
        // Only the explicitly selected signed-commit adapter exists. Native/provider
        // drafts do not become trusted merely because their transport was admitted.
        if manifest.context.provider.0 != "git" {
            proof_error = Some(w::Code::VerificationUnproven);
            return Err(c::Code::ProtocolError);
        }
        let policy = slot_bytes(
            &dir,
            &manifest,
            manifest.evidence.policy,
            &mut b.borrow_mut(),
            2 * 1024 * 1024,
        )?;
        if w::hash(&*policy) != manifest.trust_policy {
            return Err(c::Code::SourceMismatch);
        }
        let policy_file = &manifest.evidence.files[manifest.evidence.policy as usize];
        if !policy_file.path.0.starts_with("input/evidence/")
            || manifest
                .evidence
                .roots
                .iter()
                .any(|r| policy_file.path.0.starts_with(&format!("{}/", r.dir.0)))
        {
            return Err(c::Code::BadRequest);
        }
        let policy_shape = b
            .borrow_mut()
            .unit(|| storage::preflight(&policy, storage::WORK))?;
        let policy_value = b
            .borrow_mut()
            .track(policy_shape.typed(policy.len())?, || {
                serde_json::from_slice::<serde_json::Value>(&policy)
                    .map_err(|_| c::Code::ProtocolError)
            })?;
        let policy_value = Paced::new(policy_value, &control, Drain::drain_step);
        b.borrow_mut().unit(|| {
            super::trust::Policy::parse(&policy)
                .map(|_| ())
                .map_err(|_| c::Code::ProtocolError)
        })?;
        let source_revision = b
            .borrow_mut()
            .unit(|| w::source_revision(&manifest).map_err(|_| c::Code::ProtocolError))?;
        stage = "closed_sources";
        let mut logs = Paced::new(
            selected_logs(&input, &manifest, &mut b.borrow_mut())?,
            &control,
            Drain::drain_step,
        );
        let _output_credit = control.memory.reserve((w::MAX_SNAPSHOT * 4) as u64)?;
        let mut entries = Paced::new(Vec::<w::PreparedEntry>::new(), &control, Drain::drain_step);
        let mut producer = w::hash([]);
        let mut omitted = 0u32;
        if !manifest.records.is_empty() || !manifest.publications.is_empty() {
            if manifest.records.is_empty()
                || manifest.publications.is_empty()
                || manifest.evidence.roots.is_empty()
            {
                proof_error = Some(w::Code::VerificationUnproven);
                return Err(c::Code::ProtocolError);
            }
            for candidate in &manifest.records {
                let root = manifest
                    .evidence
                    .roots
                    .iter()
                    .find(|r| r.root == candidate.root)
                    .ok_or(c::Code::SourceUnavailable)?;
                if !manifest.evidence.catalogue.iter().any(|pin| {
                    pin.root == candidate.root
                        && pin.path == candidate.path
                        && pin.commit == root.main
                        && pin.digest == candidate.digest
                        && pin.id == candidate.id
                }) {
                    proof_error = Some(w::Code::VerificationUnproven);
                    return Err(c::Code::SourceUnavailable);
                }
            }
            let repo = manifest
                .evidence
                .roots
                .iter()
                .find(|r| r.root == manifest.context.root)
                .ok_or(c::Code::SourceUnavailable)?
                .repo
                .0
                .clone();
            if manifest
                .records
                .iter()
                .any(|r| r.root != manifest.context.root)
            {
                return Err(c::Code::BadRequest);
            }
            let domains = logical_domains(&manifest)?;
            let decoded = logs
                .credits
                .iter()
                .try_fold(0u64, |n, c| n.checked_add(c.bytes))
                .ok_or(c::Code::ResourceLimit)?;
            let _proof_credit = control.memory.reserve(
                decoded
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(11 * 1024 * 1024))
                    .ok_or(c::Code::ResourceLimit)?,
            )?;
            let proof_credits = Rc::new(RefCell::new(Vec::new()));
            stage = "bindings";
            let mut binding_credits = Vec::new();
            let mut bindings = Paced::new(
                super::resolve::SelectedBindings::default(),
                &control,
                Drain::drain_step,
            );
            bindings.scratch = Some(scratch.clone());
            binding_credits.push(
                control
                    .memory
                    .reserve((manifest.evidence.files.len() * 256) as u64)?,
            );
            for pin in manifest
                .records
                .iter()
                .chain(manifest.evidence.catalogue.iter())
            {
                let key = (
                    pin.root.0.clone(),
                    pin.commit.0.clone(),
                    super::resolve::repository_path(&pin.path.0)
                        .map_err(|_| c::Code::BadRequest)?,
                );
                if let Some(previous) = bindings.records.insert(key, pin.digest.0.clone())
                    && previous != pin.digest.0
                {
                    return Err(c::Code::SourceMismatch);
                }
            }
            let mut logical = BTreeSet::new();
            for binding in &manifest.evidence.artifacts {
                if !logical.insert(binding.logical.0.clone()) {
                    return Err(c::Code::BadRequest);
                }
                let data = slot_bytes(
                    &dir,
                    &manifest,
                    binding.file,
                    &mut b.borrow_mut(),
                    w::MAX_INPUT,
                )?;
                let storage::Tracked { value, credit } = data;
                bindings
                    .artifacts
                    .insert(binding.logical.0.clone().into(), value);
                binding_credits.push(credit);
            }
            for source in &manifest.closed_sources {
                let session = source
                    .session
                    .0
                    .parse()
                    .map(SessionId)
                    .map_err(|_| c::Code::BadRequest)?;
                for blob in &source.blobs {
                    let expected = domains
                        .sessions
                        .join(&source.session.0)
                        .join("blobs")
                        .join(&blob.sha256.0);
                    let data = bindings
                        .artifacts
                        .get(&expected)
                        .ok_or(c::Code::SourceUnavailable)?;
                    let credit = control.memory.reserve(data.len() as u64)?;
                    let value = b.borrow_mut().unit(|| Ok(data.clone()))?;
                    bindings
                        .blobs
                        .insert((session, blob.sha256.0.clone()), value);
                    binding_credits.push(credit);
                }
            }
            let mut fingerprints = BTreeSet::new();
            for key in &manifest.evidence.public_keys {
                if !fingerprints.insert(key.fingerprint.0.clone()) {
                    return Err(c::Code::BadRequest);
                }
                let signers = policy_value["git_signers"]
                    .as_array()
                    .ok_or(c::Code::ProtocolError)?;
                let found = signers
                    .iter()
                    .filter(|s| s["fingerprint"].as_str() == Some(&key.fingerprint.0))
                    .collect::<Vec<_>>();
                if found.len() != 1 {
                    return Err(c::Code::BadRequest);
                }
                let logical = found[0]["public_key_file"]
                    .as_str()
                    .ok_or(c::Code::ProtocolError)?;
                let f = &manifest.evidence.files[key.file as usize];
                if !f.path.0.starts_with("input/evidence/") || f.len.0 > 2 * 1024 * 1024 {
                    return Err(c::Code::BadRequest);
                }
                let physical = path.join(&f.path.0);
                let key = slot_bytes(
                    &dir,
                    &manifest,
                    key.file,
                    &mut b.borrow_mut(),
                    2 * 1024 * 1024,
                )?;
                let storage::Tracked { value, credit } = key;
                bindings.key_bytes.insert(physical.clone(), value);
                binding_credits.push(credit);
                bindings.keys.insert(logical.into(), physical);
            }
            let mut roots = Vec::new();
            for root in &manifest.evidence.roots {
                validate_git_config(&dir, &manifest, root, &mut b.borrow_mut())?;
                roots.push(super::resolve::Root {
                    name: root.root.0.clone(),
                    repo: root.repo.0.clone(),
                    path: path.join(&root.dir.0),
                    main: root.main.0.clone(),
                    cross_repo: false,
                });
            }
            let bindings = Paced::new(Arc::new(bindings.take_value()), &control, Drain::drain_step);
            let failure = Rc::new(RefCell::new(None));
            stage = "selected_proof";
            let scope = super::resolve::SelectedScope::enter(
                (*bindings).clone(),
                Box::new(ProofClock {
                    budget: b.clone(),
                    started: None,
                    failure: failure.clone(),
                    epoch: None,
                    epoch_wall: None,
                    accounted: 0,
                    steps: 0,
                    credits: proof_credits.clone(),
                    helper_clock: None,
                    helper_wall: None,
                    helper_summary: helper_summary.clone(),
                }),
            )
            .map_err(|_| c::Code::ProtocolError)?;
            let mut qualified=Paced::new((|| ->Result<Vec<(String,super::recall::PreparedLesson)>,super::seal::ProofError> {
                let trust=selected_ready(super::trust::Trust::load_selected(&policy,roots.clone()))??;
                let mut trust=super::resolve::ProofOwned::new(trust);
                let owned=std::mem::take(&mut logs.logs.0);
                let resolver=super::resolve::Resolver::from_selected(domains.sessions.clone(),roots,owned,domains)?.with_trust(trust.take_value());
                let mut resolver=super::resolve::ProofOwned::new(resolver);
                let mut groups=super::resolve::ProofOwned::new(BTreeMap::<String,Vec<crate::protocol::EvidenceRef>>::new());
                for pin in &manifest.publications {
                    let event=selected_ready(resolver.event_async(&pin.clone().into()))??;
                    let Event::ItemCompleted{body:crate::protocol::Item::Receipt{harness:Some(harness),..},..}=&event.event else {
                        return Err(super::seal::ProofError::Fail("publication carrier"));
                    };
                    if !matches!(harness.as_str(),"engine"|"acp") {return Err(super::seal::ProofError::Unproven("harness_incompatible"));}
                    groups.entry(harness.clone()).or_default().push(pin.clone().into());
                }
                let mut result=super::resolve::ProofOwned::new(vec![]);
                while let Some((harness,requested)) = groups.pop_first() {
                    let requested=super::resolve::ProofOwned::new(requested);
                    let lessons=selected_ready(super::recall::prepare_selected(&mut resolver,&requested,&harness,&repo,&crate::redact::Set::default(),manifest.evidence.selected_at_ms.0))??;
                    for lesson in lessons {result.push((harness.clone(),lesson));}
                }
                Ok(result.take_value())
            })(),&control,Drain::drain_step);
            if qualified.is_ok() {
                if let Err(reason) = super::resolve::revalidate_prepared_tools() {
                    if reason.0.starts_with("source_changed:") {
                        return Err(c::Code::SourceChanged);
                    }
                    proof_error = Some(w::Code::VerificationUnproven);
                    return Err(c::Code::SourceUnavailable);
                }
                if super::trust::cleanup_selected_scratch().is_err() {
                    proof_error = Some(w::Code::VerificationUnproven);
                    return Err(c::Code::SourceUnavailable);
                }
            }
            drop(scope);
            if let Some(code) = *failure.borrow() {
                return Err(code);
            }
            let lessons = match qualified.take_value() {
                Ok(v) => v,
                Err(super::seal::ProofError::Fail(_)) => {
                    proof_error = Some(w::Code::VerificationFailed);
                    return Err(c::Code::ProtocolError);
                }
                Err(super::seal::ProofError::Unproven(reason)) => {
                    proof_error = Some(if reason == "harness_incompatible" {
                        w::Code::HarnessIncompatible
                    } else if reason == "producer_unapproved" {
                        w::Code::ProducerUnapproved
                    } else {
                        w::Code::VerificationUnproven
                    });
                    return Err(c::Code::ProtocolError);
                }
            };
            let mut ids = BTreeSet::new();
            let mut lessons = Paced::new(lessons.into_iter(), &control, |v| {
                if let Some((_, lesson)) = v.as_mut_slice().first_mut()
                    && lesson.drain_step()
                {
                    return true;
                }
                v.next().is_some()
            });
            for (harness, lesson) in lessons.by_ref() {
                let mut lesson = Paced::new(lesson, &control, Drain::drain_step);
                if !manifest.evidence.catalogue.iter().any(|p| {
                    p.root.0 == lesson.current_record.root
                        && p.path.0 == lesson.current_record.path
                        && Some(p.id.0.as_str()) == lesson.current_record.id.as_deref()
                        && Some(p.commit.0.as_str()) == lesson.current_record.commit.as_deref()
                        && p.digest.0 == lesson.current_record.digest
                }) {
                    proof_error = Some(w::Code::VerificationUnproven);
                    return Err(c::Code::SourceUnavailable);
                }
                if !manifest
                    .records
                    .iter()
                    .any(|pin| crate::protocol::RecordPin::from(pin.clone()) == lesson.record)
                    || !ids.insert(lesson.entry.clone())
                {
                    proof_error = Some(w::Code::VerificationFailed);
                    return Err(c::Code::ProtocolError);
                }
                let producer_hash = prefixed_hex(&lesson.producer_digest)?;
                if !entries.is_empty() && producer != producer_hash {
                    proof_error = Some(w::Code::VerificationFailed);
                    return Err(c::Code::ProtocolError);
                }
                producer = producer_hash.clone();
                let prepared = b.borrow_mut().unit(|| {
                    let mut nodes = vec![];
                    let mut edges = vec![];
                    for (index, pin) in lesson.evidence.iter().take(15).cloned().enumerate() {
                        let pin = match pin {
                            crate::protocol::EvidenceRef::Event { session, .. }
                                if manifest.closed_sources.iter().any(|source| {
                                    source.session.0 == session.to_string() && source.repo.0 == repo
                                }) =>
                            {
                                Some(w::Pin::Event(
                                    pin.try_into().map_err(|_| c::Code::ProtocolError)?,
                                ))
                            }
                            crate::protocol::EvidenceRef::Record {
                                root,
                                id,
                                path,
                                commit,
                                digest,
                            } if root == manifest.context.root.0 => Some(w::Pin::Record(
                                crate::protocol::RecordPin {
                                    root,
                                    id: Some(id),
                                    path,
                                    commit: Some(commit),
                                    digest,
                                }
                                .try_into()
                                .map_err(|_| c::Code::ProtocolError)?,
                            )),
                            _ => None,
                        };
                        let node = w::Id(format!("evidence-{index}"));
                        nodes.push(w::EvidenceNode {
                            id: node.clone(),
                            kind: w::Id(if pin.is_some() { "source" } else { "redacted" }.into()),
                            pin,
                        });
                        edges.push(w::EvidenceEdge {
                            from: w::Id(lesson.entry.clone()),
                            to: node,
                            relation: w::Id("supported_by".into()),
                        });
                    }
                    let id = w::Id(lesson.entry.clone());
                    nodes.insert(
                        0,
                        w::EvidenceNode {
                            id: id.clone(),
                            kind: w::Id("lesson".into()),
                            pin: Some(w::Pin::Record(
                                lesson
                                    .record
                                    .clone()
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                            )),
                        },
                    );
                    let prepared = w::PreparedEntry {
                        hit: w::Hit {
                            handle: id.clone(),
                            id: id.clone(),
                            kind: w::Kind::Lesson,
                            title: std::mem::take(&mut lesson.identity.title),
                            preview: "Verified at the admitted source revision".into(),
                            repo: w::Id(repo.clone()),
                            harness: w::Id(harness),
                            proof: w::LessonProof {
                                kind: w::ProofKind::VerifiedLesson,
                                entry: id,
                                revival: u32::try_from(lesson.identity.revival)
                                    .map_err(|_| c::Code::ResourceLimit)?,
                                record: lesson
                                    .record
                                    .clone()
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                trial: lesson
                                    .trial
                                    .clone()
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                publication: lesson
                                    .publication
                                    .clone()
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                outer: lesson
                                    .outer
                                    .clone()
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                projection: prefixed_hex(&lesson.projection_digest)?,
                                trust_policy: manifest.trust_policy.clone(),
                                producer: producer_hash,
                                currentness: prefixed_hex(&lesson.currentness_digest)?,
                            },
                        },
                        why: std::mem::take(&mut lesson.identity.why),
                        how_to_apply: std::mem::take(&mut lesson.identity.how_to_apply),
                        context: std::mem::take(&mut lesson.context),
                        citation: std::mem::take(&mut lesson.citation),
                        nodes,
                        edges,
                    };
                    Ok(prepared)
                })?;
                let mut prepared = Paced::new(prepared, &control, Drain::drain_step);
                match b.borrow_mut().unit(|| Ok(w::validate_entry(&prepared)))? {
                    Ok(()) => entries.push(prepared.take_value()),
                    Err(w::Code::ResourceLimit) => {
                        omitted += 1;
                    }
                    Err(e) => return Err(internal(e)),
                }
            }
            drop(bindings);
            drop(binding_credits);
        }
        stage = "publish";
        b.borrow_mut().announce(c::Phase::Publish);
        // Revalidate the exact original finite bundle after proof use. No mutable
        // ref/config/record can silently switch the source revision mid-export.
        inventory(&dir, &manifest, &mut b.borrow_mut())?;
        let mut refs = vec![];
        let mut entry_ids = vec![];
        let mut total = 0usize;
        let mut page = Paced::new(
            w::Page {
                schema: w::PageSchema::V1,
                run_id: start.run_id.clone(),
                nonce: start.nonce.clone(),
                index: 0,
                entries: vec![],
                next: Some(1),
            },
            &control,
            Drain::drain_step,
        );
        let mut entries = Paced::new(entries.take_value().into_iter(), &control, |v| {
            if let Some(entry) = v.as_mut_slice().first_mut()
                && entry.drain_step()
            {
                return true;
            }
            v.next().is_some()
        });
        for entry in entries.by_ref() {
            let entry_id = entry.hit.id.clone();
            page.entries.push(entry);
            let size = b.borrow_mut().unit(|| {
                w::canonical_limited(&*page, 32 * 1024)
                    .map(|v| v.len())
                    .map_err(internal)
            })?;
            if size > w::MAX_PAGE {
                let mut last = Paced::new(page.entries.pop().unwrap(), &control, Drain::drain_step);
                publish_page(&output, &page, &mut refs, &mut total, &mut b.borrow_mut())?;
                page.index += 1;
                drop(Paced::new(
                    std::mem::take(&mut page.entries),
                    &control,
                    Drain::drain_step,
                ));
                page.entries.push(last.take_value());
                page.next = Some(page.index + 1);
            }
            entry_ids.push(entry_id);
        }
        page.next = None;
        publish_page(&output, &page, &mut refs, &mut total, &mut b.borrow_mut())?;
        // PRE-DESCRIPTOR cutoff, identical to the canonical shared owner:
        // serialization/publication, teardown and actual join remain terminal.
        // Read wire bytes do not gain a second metrics or authority field.
        b.borrow().capture_max();
        let descriptor = w::Snapshot {
            schema: w::SnapshotSchema::V1,
            run_id: start.run_id.clone(),
            nonce: start.nonce.clone(),
            context: manifest.context.clone(),
            input_digest: start.manifest_sha256.clone(),
            source_revision,
            producer,
            trust_policy: manifest.trust_policy.clone(),
            coverage: w::Coverage {
                kinds: vec![w::Kind::Lesson],
                indexed: entry_ids.len() as u32,
                omitted,
                complete: omitted == 0,
                currentness: w::Currentness::VerifiedAtRevision,
            },
            entry_ids,
            pages: refs,
        };
        let descriptor = Paced::new(descriptor, &control, Drain::drain_step);
        let bytes = b
            .borrow_mut()
            .unit(|| w::canonical_limited(&*descriptor, w::MAX_FRAME).map_err(internal))?;
        if total + bytes.len() > w::MAX_SNAPSHOT {
            return Err(c::Code::ResourceLimit);
        }
        let hash = b.borrow_mut().unit(|| Ok(w::hash(&bytes)))?;
        validate_custody(&input, &custody, &mut b.borrow_mut())?;
        inventory(&dir, &manifest, &mut b.borrow_mut())?;
        b.borrow_mut().unit(|| {
            let current = Dir::open(&path)?
                .inventory_fd()?
                .metadata()
                .map_err(|_| c::Code::SourceChanged)?;
            let held = dir
                .inventory_fd()?
                .metadata()
                .map_err(|_| c::Code::SourceChanged)?;
            if file_stamp(&current) != file_stamp(&job_before)
                || file_stamp(&held) != file_stamp(&job_before)
            {
                return Err(c::Code::SourceChanged);
            }
            Ok(())
        })?;
        let current_manifest = read_bounded(
            &input,
            "manifest.json",
            w::MAX_FRAME as u64,
            &mut b.borrow_mut(),
        )?;
        if w::hash(&*current_manifest) != start.manifest_sha256 {
            return Err(c::Code::SourceChanged);
        }
        output.publish("result.json", &bytes, &start.nonce, &mut b.borrow_mut())?;
        Ok(Completed {
            hash,
            pages: descriptor.pages.len() as u32,
        })
    })();
    // Fixed diagnostics are ordinary measured settlement work, including on a
    // failed proof. Their later cost belongs to terminal accounting, not the
    // immutable descriptor's earlier capture cutoff.
    cleanup_unit(&control, || {
        helper_summary.borrow().emit();
        let mut b = b.borrow_mut();
        b.announce(c::Phase::Settling);
        let (heap_live, peak) = heaps();
        let (live, reserved, cumulative) = control.memory.snapshot();
        eprintln!(
            "dream-read metrics heap_live_bytes={heap_live} heap_peak_bytes={peak} rss_peak_bytes={} max_unit_cpu_ns={} overruns={} storage_declared_live_bytes={live} storage_reserved_peak_bytes={reserved} storage_cumulative_credits_bytes={cumulative} control_reserved_bytes={} helper_reserved_bytes={} credit_ceiling_bytes={} compute_cpu_ns={} bytes={} envelopes={} native_cumulative_bytes=unmeasured cleanup_cpu_ns={} max_cleanup_unit_cpu_ns={}",
            rss(),
            b.max_unit,
            b.overruns,
            storage::CONTROL,
            storage::HELPERS,
            storage::WORK + storage::HELPERS + storage::CONTROL,
            b.counters.cpu,
            b.counters.bytes,
            b.counters.envelopes,
            control.cleanup_cpu.load(Ordering::Acquire),
            control.cleanup_max.load(Ordering::Acquire)
        );
        if let Err(code) = &result {
            eprintln!(
                "dream-read refused stage={stage} code={code:?} wall_ms={}",
                wall.elapsed().as_millis()
            );
        }
    });
    if control.cleanup_max.load(Ordering::Acquire) > 1_000_000 {
        return Err(err(w::Code::ResourceLimit));
    }
    result.map_err(|code| w::Error {
        code: proof_error.unwrap_or_else(|| external(code)),
        source: source_index.take(),
    })
}
fn file_stamp(m: &std::fs::Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    use std::os::unix::fs::MetadataExt;
    (
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    )
}

#[cfg(test)]
mod helper_tests {
    use super::super::resolve::{NativeCreation, ProofPace};
    use super::*;
    pub(super) fn fixture_priority() {
        // A real worker installs this once on its dedicated owner. Some unit
        // controls construct two owners on one test thread; Darwin refuses a
        // repeated pthread QoS change after the explicit latency policy.
        thread_local! {
            static PRIORITY: std::cell::OnceCell<()> = const { std::cell::OnceCell::new() };
        }
        PRIORITY.with(|once| {
            once.get_or_init(|| low_priority().expect("private helper fixture priority"));
        });
    }
    #[test]
    fn late_async_read_cli_refuses_without_consuming_arguments() {
        let arguments = std::iter::from_fn(|| -> Option<String> {
            panic!("late read CLI must not inspect source/job arguments")
        });
        assert_eq!(selected_ready(cli(arguments)).unwrap(), 2);
    }
    #[test]
    fn suspend_closes_ordinary_batch_and_pays_canonical_debt_before_native_lane() {
        let (mut clock, summary, control) = owner();
        clock.before().unwrap();
        std::hint::black_box(17u64);
        let at = Instant::now();
        clock.suspend().unwrap();
        assert!(at.elapsed() >= Duration::from_millis(9));
        assert!(clock.epoch.is_none() && clock.epoch_wall.is_none());
        assert!(clock.started.is_none());
        let budget = clock.budget.borrow();
        assert!(budget.counters.cpu > 0 && budget.counters.cpu <= 1_000_000);
        assert_eq!(budget.capture_max(), budget.counters.cpu);
        assert_eq!(budget.max_unit, budget.counters.cpu);
        assert!(summary.borrow().cost_known);
        assert!(control.error().is_none());
    }
    fn owner() -> (ProofClock, Rc<RefCell<HelperSummary>>, Arc<Control>) {
        fixture_priority();
        let control = Arc::new(Control::fixture(Instant::now() + Duration::from_secs(2)));
        let summary = Rc::new(RefCell::new(HelperSummary::default()));
        summary.borrow_mut().cost_known = true;
        summary.borrow_mut().totals.coverage_known = true;
        (
            ProofClock {
                budget: Rc::new(RefCell::new(budget(
                    control.clone(),
                    Arc::new(crossbeam_queue::ArrayQueue::new(32)),
                ))),
                started: None,
                failure: Rc::new(RefCell::new(None)),
                epoch: None,
                epoch_wall: None,
                accounted: 0,
                steps: 0,
                credits: Rc::new(RefCell::new(vec![])),
                helper_clock: None,
                helper_wall: None,
                helper_summary: summary.clone(),
            },
            summary,
            control,
        )
    }
    fn span(native: u64, inclusive: u64, app: u64) -> NativeCreation {
        NativeCreation {
            sequence: 1,
            rawerrno: libc::ENOENT,
            identity: None,
            cpu_ns: native,
            wall_ns: 100_000,
            inclusive_cpu_ns: inclusive,
            inclusive_wall_ns: 200_000,
            app_cpu_ns: app,
            credits: 1,
        }
    }
    #[test]
    fn native_application_label_cannot_hide_ordinary_work() {
        let (mut clock, summary, _) = owner();
        assert!(
            clock
                .native_created(span(500_000, 2_000_000, 1_500_000))
                .is_err()
        );
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        assert_eq!(summary.borrow().app_max_cpu, 1_500_000);
        assert_eq!(summary.borrow().effects, 0);
    }
    #[test]
    fn native_debt_starts_after_span_and_cannot_use_saved_idle() {
        let (mut clock, summary, _) = owner();
        std::thread::sleep(Duration::from_millis(25));
        let at = Instant::now();
        assert!(
            clock
                .native_created(span(1_900_000, 2_000_000, 100_000))
                .is_err()
        );
        assert!(at.elapsed() >= Duration::from_millis(18));
        assert_eq!(summary.borrow().creation_debt, 18_000_000);
    }
    #[test]
    fn cancellation_does_not_reset_failed_creation_debt() {
        let (mut clock, summary, control) = owner();
        control.cancel.store(1, Ordering::Release);
        let at = Instant::now();
        assert!(
            clock
                .native_created(span(100_000, 200_000, 100_000))
                .is_err()
        );
        assert!(at.elapsed() >= Duration::from_millis(9));
        assert_eq!(summary.borrow().creation_debt, 9_000_000);
        // This span reports a real no-effect ENOENT; cancellation cannot erase
        // that verdict or its debt, and the independent cancel flag survives.
        assert_eq!(*clock.failure.borrow(), Some(c::Code::SourceUnavailable));
        assert_eq!(control.cancel.load(Ordering::Acquire), 1);
    }
    #[test]
    fn native_clock_loss_retains_unknown_raw_cost_without_zero_certification() {
        let (mut clock, summary, control) = owner();
        assert!(
            clock
                .native_created(span(u64::MAX, u64::MAX, u64::MAX))
                .is_err()
        );
        assert!(!summary.borrow().cost_known);
        assert_eq!(summary.borrow().inclusive_max_cpu, u64::MAX);
        assert_eq!(*clock.failure.borrow(), Some(c::Code::SourceUnavailable));
        assert_eq!(clock.budget.borrow().capture_max(), u64::MAX);
        assert_eq!(control.error(), Some(c::Code::ResourceLimit));
    }
    #[test]
    fn native_cpu_limit_retains_raw_cost_and_debt() {
        let (mut clock, summary, _) = owner();
        let value = span(2_000_001, 2_000_001, 0);
        assert!(clock.native_created(value).is_err());
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        let summary = summary.borrow();
        assert!(summary.cost_known);
        assert_eq!(summary.native_max_cpu, 2_000_001);
        assert_eq!(summary.inclusive_max_cpu, 2_000_001);
        assert_eq!(summary.app_max_cpu, 0);
        assert_eq!(summary.creation_debt, 18_000_009);
    }
    #[test]
    fn native_wall_limit_retains_raw_cost_and_debt() {
        let (mut clock, summary, _) = owner();
        let mut value = span(100_000, 200_000, 100_000);
        value.wall_ns = 25_000_001;
        value.inclusive_wall_ns = 25_100_001;
        assert!(clock.native_created(value).is_err());
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        let summary = summary.borrow();
        assert!(summary.cost_known);
        assert_eq!(summary.native_max_wall, 25_000_001);
        assert_eq!(summary.inclusive_max_wall, 25_100_001);
        assert_eq!(summary.native_max_cpu, 100_000);
        assert_eq!(summary.creation_debt, 9_000_000);
    }
    #[test]
    fn native_inclusive_limit_retains_raw_cost_and_debt() {
        let (mut clock, summary, _) = owner();
        // Independent raw-inclusive guard input: both constituent guard values
        // stay at their limits, so removing only raw3ms changes the verdict.
        let value = span(2_000_000, 3_000_001, 1_000_000);
        assert!(clock.native_created(value).is_err());
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        let summary = summary.borrow();
        assert!(summary.cost_known);
        assert_eq!(summary.native_max_cpu, 2_000_000);
        assert_eq!(summary.app_max_cpu, 1_000_000);
        assert_eq!(summary.inclusive_max_cpu, 3_000_001);
        assert_eq!(summary.creation_debt, 27_000_009);
    }
    #[test]
    fn creation_cap_is_consumed_for_implicit_descendant_without_refund() {
        let (clock, _, _) = owner();
        let scope = super::super::resolve::SelectedScope::enter(
            Arc::new(super::super::resolve::SelectedBindings::default()),
            Box::new(clock),
        )
        .unwrap();
        for expected in (2..=256).step_by(2) {
            assert_eq!(
                super::super::resolve::helper_creation_reserve(2).unwrap(),
                expected
            );
        }
        assert!(super::super::resolve::helper_creation_reserve(1).is_err());
        drop(scope);
    }
    #[cfg(target_os = "macos")]
    struct StoppedChild {
        pid: i32,
        status: Option<std::process::ExitStatus>,
    }
    #[cfg(target_os = "macos")]
    impl StoppedChild {
        fn id(&self) -> u32 {
            self.pid as u32
        }
        fn kill(&mut self) -> std::io::Result<()> {
            if self.status.is_some() {
                return Ok(());
            }
            if unsafe { libc::kill(self.pid, libc::SIGKILL) } == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
        fn observe(&mut self, flags: i32) -> std::io::Result<Option<std::process::ExitStatus>> {
            use std::os::unix::process::ExitStatusExt;
            if self.status.is_some() {
                return Ok(self.status);
            }
            let mut status = 0;
            let got = unsafe { libc::waitpid(self.pid, &mut status, flags) };
            if got == self.pid {
                self.status = Some(std::process::ExitStatus::from_raw(status));
                Ok(self.status)
            } else if got == 0 {
                Ok(None)
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
        fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
            self.observe(libc::WNOHANG)
        }
        fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
            loop {
                match self.observe(0) {
                    Ok(Some(status)) => return Ok(status),
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                    Ok(None) => unreachable!(),
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    struct RetirementChild(StoppedChild);
    #[cfg(target_os = "macos")]
    impl Drop for RetirementChild {
        fn drop(&mut self) {
            // Only the exact private child created below, retained unreaped.
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    #[cfg(target_os = "macos")]
    fn retirement_child(control: &Arc<Control>) -> (RetirementChild, i32) {
        // Independent real kernel birth: retirement tests require a child that
        // has executed no user code. A post-spawn STOP has an uncontrolled gap.
        let mut attr = std::mem::MaybeUninit::<libc::posix_spawnattr_t>::uninit();
        assert_eq!(unsafe { libc::posix_spawnattr_init(attr.as_mut_ptr()) }, 0);
        let mut attr = unsafe { attr.assume_init() };
        assert_eq!(unsafe { libc::posix_spawnattr_setpgroup(&mut attr, 0) }, 0);
        assert_eq!(
            unsafe {
                libc::posix_spawnattr_setflags(
                    &mut attr,
                    (libc::POSIX_SPAWN_START_SUSPENDED | libc::POSIX_SPAWN_SETPGROUP) as i16,
                )
            },
            0
        );
        let argv = [
            c"sleep".as_ptr().cast_mut(),
            c"30".as_ptr().cast_mut(),
            std::ptr::null_mut(),
        ];
        let env = [c"LANG=C".as_ptr().cast_mut(), std::ptr::null_mut()];
        let mut pid = 0;
        let result = unsafe {
            libc::posix_spawn(
                &mut pid,
                c"/bin/sleep".as_ptr(),
                std::ptr::null(),
                &attr,
                argv.as_ptr(),
                env.as_ptr(),
            )
        };
        assert!(
            pid > 0,
            "private fixture birth must yield an owned PID: {result}"
        );
        let child = RetirementChild(StoppedChild { pid, status: None });
        assert_eq!(unsafe { libc::posix_spawnattr_destroy(&mut attr) }, 0);
        assert_eq!(result, 0);
        let until = Instant::now() + Duration::from_secs(1);
        let info = loop {
            let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
            let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
            assert_eq!(
                unsafe {
                    libc::proc_pidinfo(
                        pid,
                        libc::PROC_PIDTBSDINFO,
                        0,
                        info.as_mut_ptr().cast(),
                        size,
                    )
                },
                size
            );
            let info = unsafe { info.assume_init() };
            if info.pbi_status == 4 {
                break info;
            }
            assert!(
                Instant::now() < until,
                "own private child must actually stop"
            );
            std::thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(info.pbi_pgid, pid as u32);
        let identity = super::super::resolve::HelperIdentity {
            sequence: 1,
            pid,
            pgid: pid,
            start_identity: info.pbi_start_tvsec * 1_000_000 + info.pbi_start_tvusec,
            expected_members: 1,
            image: std::fs::canonicalize("/bin/sleep").unwrap(),
            descendant_image: None,
        };
        *control.helper.lock().unwrap() = Some(super::super::c1::HelperGroup::attached(identity));
        // Exercise the real born-stopped observer before the native handoff.
        // START_SUSPENDED may publish SSTOP before its kernel tail quiesces.
        control.foreground.store(true, Ordering::Release);
        let observing = Instant::now();
        loop {
            let mut owner = control.helper.lock().unwrap();
            let helper = owner.as_mut().unwrap();
            match helper.sample(control) {
                Ok(true) => {
                    helper.settle_creation(control).unwrap();
                    break;
                }
                Ok(false)
                    if helper.initial_stop_pending
                        && observing.elapsed() < Duration::from_millis(25) => {}
                result => panic!("private suspended child observation: {result:?}"),
            }
            drop(owner);
            // Drop the mutex guard before the next real kernel observation.
            std::thread::sleep(Duration::from_micros(100));
        }
        (child, pid)
    }
    #[cfg(target_os = "macos")]
    struct ControlPump {
        stop: Arc<std::sync::atomic::AtomicBool>,
        thread: Option<std::thread::JoinHandle<()>>,
    }
    #[cfg(target_os = "macos")]
    impl ControlPump {
        fn start(control: Arc<Control>) -> Self {
            let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let stopping = stop.clone();
            let thread = std::thread::spawn(move || {
                fixture_priority();
                while !stopping.load(Ordering::Acquire) {
                    let marker = control.helper_controller_marker();
                    let at = cpu_ns();
                    control.helper_tick();
                    let running = control
                        .helper
                        .lock()
                        .unwrap()
                        .as_ref()
                        .is_some_and(|helper| helper.running && !helper.retiring);
                    control
                        .helper_account_controller(marker, control.observe_interval(at, cpu_ns()));
                    if !running {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }
            });
            Self {
                stop,
                thread: Some(thread),
            }
        }
    }
    #[cfg(target_os = "macos")]
    impl Drop for ControlPump {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
            self.thread.take().unwrap().join().unwrap();
        }
    }
    #[cfg(target_os = "macos")]
    fn begin_fixture_pipe(clock: &mut ProofClock, control: &Arc<Control>) -> u64 {
        control
            .helper
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .begin_pipe()
            .unwrap();
        let start = cpu_ns();
        clock.helper_clock = Some(start);
        clock.helper_wall = Some(Instant::now());
        start
    }
    #[cfg(target_os = "macos")]
    fn retirement_ticket(control: &Arc<Control>) -> HelperTicket {
        control
            .helper
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .identity
            .ticket()
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn retirement_application_rest_starts_after_actual_span_despite_saved_idle() {
        let (mut clock, summary, control) = owner();
        let (_child, _pid) = retirement_child(&control);
        std::thread::sleep(Duration::from_millis(25));
        let _pump = ControlPump::start(control.clone());
        begin_fixture_pipe(&mut clock, &control);
        clock.helper_retire(retirement_ticket(&control)).unwrap();
        let returned = Instant::now();
        let summary = summary.borrow();
        assert_eq!(summary.retirement_count, 1);
        assert_eq!(summary.retirement_unknown, 0);
        assert!(summary.retirement_inclusive_max_cpu <= 1_000_000);
        let required = summary.retirement_inclusive_max_cpu.max(1_000_000) * 9;
        assert!(
            returned >= summary.retirement_last_end.unwrap() + Duration::from_nanos(required),
            "canonical retirement debt must start after the actual signal span"
        );
        let helper = control.helper.lock().unwrap();
        let helper = helper.as_ref().unwrap();
        assert!(helper.retiring && !helper.ready);
        assert_eq!(
            summary.creations, 0,
            "ordinary retirement never gets a native label"
        );
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn retirement_pipe_tail_enforces_application_limit_and_preserves_first_cancel() {
        for cancelled in [false, true] {
            let (mut clock, summary, control) = owner();
            let (_child, _pid) = retirement_child(&control);
            let _pump = ControlPump::start(control.clone());
            let start = begin_fixture_pipe(&mut clock, &control);
            while control.observe_interval(start, cpu_ns()) < 2_000_000 {
                std::hint::black_box(cpu_ns());
            }
            if cancelled {
                control.cancel.store(1, Ordering::Release);
            }
            assert!(clock.helper_retire(retirement_ticket(&control)).is_err());
            let returned = Instant::now();
            let summary = summary.borrow();
            assert!(summary.retirement_pending_cpu >= 2_000_000);
            assert!(summary.retirement_inclusive_max_cpu >= 2_000_000);
            assert_eq!(summary.creations, 0);
            let b = clock.budget.borrow();
            assert!(b.max_unit >= 2_000_000 && b.overruns != 0);
            assert!(b.counters.cpu >= summary.retirement_inclusive_max_cpu);
            let code = if cancelled {
                c::Code::Cancelled
            } else {
                c::Code::ResourceLimit
            };
            assert_eq!(*clock.failure.borrow(), Some(code));
            assert_eq!(
                control.cancel.load(Ordering::Acquire),
                if cancelled { 1 } else { 4 }
            );
            let helper = control.helper.lock().unwrap();
            let helper = helper.as_ref().unwrap();
            assert!(helper.metrics.pipe_cpu_ns >= 2_000_000 && helper.retiring);
            assert!(returned >= summary.retirement_last_end.unwrap() + Duration::from_millis(9));
        }
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn retirement_unknown_clock_keeps_raw_unknown_and_owned_quarantine() {
        let (mut clock, summary, control) = owner();
        let (_child, _pid) = retirement_child(&control);
        let known_debt = Instant::now() + Duration::from_millis(17);
        control.helper.lock().unwrap().as_mut().unwrap().debt_until = known_debt;
        let _pump = ControlPump::start(control.clone());
        begin_fixture_pipe(&mut clock, &control);
        clock.helper_clock = Some(u64::MAX);
        assert!(clock.helper_retire(retirement_ticket(&control)).is_err());
        let summary = summary.borrow();
        assert!(!summary.cost_known);
        assert_eq!(summary.retirement_pending_cpu, u64::MAX);
        assert_eq!(summary.retirement_inclusive_max_cpu, u64::MAX);
        assert_eq!(summary.retirement_unknown, 1);
        assert_eq!(
            summary.retirement_conservative_debt, 0,
            "no fabricated finite payment for unknown CPU"
        );
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        let helper = control.helper.lock().unwrap();
        let helper = helper.as_ref().unwrap();
        assert!(helper.retiring && !helper.ready && !helper.metrics.coverage_known);
        assert_eq!(helper.metrics.pipe_cpu_ns, u64::MAX);
        assert!(helper.debt_until >= known_debt);
    }

    #[cfg(target_os = "macos")]
    fn reap_private_child(child: &mut RetirementChild, pid: i32) {
        assert_eq!(child.0.id(), pid as u32);
        child.0.kill().unwrap();
        assert!(!child.0.wait().unwrap().success());
        // Child::wait records the actual reap, so its Drop cannot signal a
        // later process that happens to reuse this numeric identity.
        assert!(child.0.try_wait().unwrap().is_some());
        assert_eq!(unsafe { libc::kill(-pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(target_os = "macos")]
    fn assert_private_child_unreaped(child: &RetirementChild, pid: i32) {
        assert_eq!(child.0.id(), pid as u32);
        let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
        assert_eq!(
            unsafe {
                libc::waitid(
                    libc::P_PID,
                    pid as libc::id_t,
                    info.as_mut_ptr(),
                    libc::WNOHANG | libc::WEXITED | libc::WNOWAIT,
                )
            },
            0,
            "original child custody must survive retirement until actual reap"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn same_numeric_group_rejects_stale_ticket_before_every_custody_effect() {
        let (mut clock, summary, control) = owner();
        let (mut child, pid) = retirement_child(&control);
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "fixture exited before stale hooks"
        );
        let ticket = retirement_ticket(&control);
        let stale = [
            HelperTicket {
                sequence: ticket.sequence.checked_add(1).unwrap(),
                ..ticket
            },
            HelperTicket {
                start_identity: ticket.start_identity.checked_add(1).unwrap(),
                ..ticket
            },
            HelperTicket {
                pid: ticket.pid.checked_add(1).unwrap(),
                ..ticket
            },
            HelperTicket {
                pgid: ticket.pgid.checked_add(1).unwrap(),
                ..ticket
            },
        ];
        let checkpoint = begin_fixture_pipe(&mut clock, &control);
        for stale in stale {
            assert!(clock.helper_poll(stale).is_err());
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "stale poll changed child"
            );
            assert!(clock.helper_usage(stale, 7, 10, 19).is_err());
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "stale usage changed child"
            );
            assert!(clock.helper_retire(stale).is_err());
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "stale retirement changed child"
            );
            assert_eq!(clock.helper_clock, Some(checkpoint));
            assert!(clock.failure.borrow().is_none());
            assert_eq!(control.cancel.load(Ordering::Acquire), 0);
            assert_eq!(summary.borrow().retirement_count, 0);
            assert_eq!(summary.borrow().rolled_cpu, 0);
            let helper = control.helper.lock().unwrap();
            let helper = helper.as_ref().unwrap();
            assert_eq!(helper.identity.ticket(), ticket);
            assert!(!helper.retiring && !helper.running && !helper.ready);
            assert_eq!(helper.metrics.pipe_cpu_ns, 0);
            assert_eq!(helper.metrics.charged_ns, 0);
            assert_eq!(helper.metrics.debt_ns, 0);
        }
        assert!(child.0.try_wait().unwrap().is_none());

        // Detach's independent incarnation guard is exercised with retirement
        // already true, so its refusal cannot be attributed to that state gate.
        let _pump = ControlPump::start(control.clone());
        let _ = clock.helper_retire(ticket);
        let debt = control.helper.lock().unwrap().as_ref().unwrap().debt_until;
        for stale in stale {
            assert!(clock.helper_detach(stale).is_err());
            let helper = control.helper.lock().unwrap();
            let helper = helper.as_ref().unwrap();
            assert_eq!(helper.identity.ticket(), ticket);
            assert!(helper.retiring);
            assert_eq!(helper.debt_until, debt);
            assert_eq!(summary.borrow().settled, 0);
        }
        assert_private_child_unreaped(&child, pid);
        reap_private_child(&mut child, pid);
        clock.helper_detach(ticket).unwrap();
        assert!(control.helper.lock().unwrap().is_none());
        assert_eq!(summary.borrow().settled, 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn main_proof_before_refuses_even_stopped_owned_helper_until_real_reap() {
        let (mut clock, summary, control) = owner();
        let (mut child, pid) = retirement_child(&control);
        assert!(child.0.try_wait().unwrap().is_none());
        let error = clock.before().unwrap_err();
        assert_eq!(error.0, "main proof overlaps helper owner");
        assert_eq!(*clock.failure.borrow(), Some(c::Code::SourceUnavailable));
        assert!(clock.started.is_none() && clock.epoch.is_none());
        assert_eq!(clock.steps, 0);
        assert_eq!(clock.budget.borrow().counters.cpu, 0);
        assert_eq!(summary.borrow().settled, 0);
        assert_eq!(control.helper.lock().unwrap().as_ref().unwrap().pgid, pid);

        let _pump = ControlPump::start(control.clone());
        begin_fixture_pipe(&mut clock, &control);
        assert!(clock.helper_retire(retirement_ticket(&control)).is_err());
        assert!(control.helper.lock().unwrap().as_ref().unwrap().retiring);
        assert_private_child_unreaped(&child, pid);
        assert_eq!(summary.borrow().settled, 0);
        reap_private_child(&mut child, pid);
        clock.helper_detach(retirement_ticket(&control)).unwrap();
        assert!(control.helper.lock().unwrap().is_none());
        assert_eq!(summary.borrow().settled, 1);
        assert!(
            clock.before().is_err(),
            "refusal cannot certify new proof work"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn cancelled_stopped_owner_retires_in_one_adjacent_paid_unit_before_real_reap() {
        let (mut clock, summary, control) = owner();
        let (mut child, pid) = retirement_child(&control);
        let ticket = retirement_ticket(&control);
        control.cancel.store(1, Ordering::Release);
        assert!(clock.helper_retire(ticket).is_err());
        let returned = Instant::now();
        {
            let summary = summary.borrow();
            assert_eq!(summary.retirement_count, 1);
            assert_eq!(summary.retirement_pending_cpu, 0);
            assert_eq!(summary.retirement_unknown, 0);
            assert!(summary.retirement_cpu > 0 && summary.retirement_cpu <= 1_000_000);
            assert_eq!(summary.retirement_cpu, summary.retirement_max_cpu);
            assert!(returned >= summary.retirement_last_end.unwrap() + Duration::from_millis(9));
            assert_eq!(summary.settled, 0, "signal delivery cannot certify reap");
        }
        assert_private_child_unreaped(&child, pid);
        reap_private_child(&mut child, pid);
        clock.helper_detach(ticket).unwrap();
        assert_eq!(summary.borrow().settled, 1);
        assert!(control.helper.lock().unwrap().is_none());
        assert_eq!(*clock.failure.borrow(), Some(c::Code::Cancelled));
        assert_eq!(control.cancel.load(Ordering::Acquire), 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn work_credit_exhaustion_preserves_control_cancel_and_owned_child_settlement() {
        let (mut clock, summary, control) = owner();
        let (mut child, pid) = retirement_child(&control);
        // Exhaust declared work custody, without allocating a test-sized heap
        // or borrowing the protected control/opaque-native allocations.
        let work = control.memory.reserve(storage::WORK).unwrap();
        assert!(clock.reserve(1).is_err());
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        assert_eq!(control.memory.snapshot().0, storage::WORK);
        assert!(clock.credits.borrow().is_empty());

        control.cancel.store(1, Ordering::Release);
        assert_eq!(control.error(), Some(c::Code::Cancelled));
        let _pump = ControlPump::start(control.clone());
        begin_fixture_pipe(&mut clock, &control);
        assert!(clock.helper_retire(retirement_ticket(&control)).is_err());
        assert!(control.helper.lock().unwrap().as_ref().unwrap().retiring);
        assert_eq!(control.cancel.load(Ordering::Acquire), 1);
        assert_eq!(control.memory.snapshot().0, storage::WORK);
        assert_eq!(summary.borrow().settled, 0);
        reap_private_child(&mut child, pid);
        clock.helper_detach(retirement_ticket(&control)).unwrap();
        assert!(control.helper.lock().unwrap().is_none());
        assert_eq!(summary.borrow().settled, 1);
        assert_eq!(control.memory.snapshot().0, storage::WORK);
        assert_eq!(*clock.failure.borrow(), Some(c::Code::ResourceLimit));
        assert_eq!(control.cancel.load(Ordering::Acquire), 1);
        drop(work);
        assert_eq!(control.memory.snapshot().0, 0);
    }
}
fn validate_git_config(
    dir: &Dir,
    m: &w::Input,
    root: &w::RootBinding,
    b: &mut Budget,
) -> Result<(), c::Code> {
    let config = format!("{}/.git/config", root.dir.0);
    let slot = m
        .evidence
        .files
        .iter()
        .find(|f| f.path.0 == config)
        .ok_or(c::Code::SourceUnavailable)?
        .slot;
    let raw = slot_bytes(dir, m, slot, b, 16 * 1024)?;
    let text = std::str::from_utf8(&raw).map_err(|_| c::Code::BadRequest)?;
    let mut section = "";
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            if !matches!(line, "[core]" | "[remote \"origin\"]") {
                return Err(c::Code::BadRequest);
            }
            section = line;
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(c::Code::BadRequest)?;
        let (key, value) = (key.trim(), value.trim());
        let valid = match section {
            "[core]" => matches!(
                (key, value),
                ("repositoryformatversion", "0")
                    | ("filemode", "true")
                    | ("filemode", "false")
                    | ("bare", "false")
                    | ("logallrefupdates", "true")
                    | ("ignorecase", "true")
                    | ("precomposeunicode", "true")
            ),
            "[remote \"origin\"]" => {
                key == "url"
                    && crate::vcs::normalize_repo_key(value).as_deref()
                        == Some(root.repo.0.as_str())
            }
            _ => false,
        };
        if !valid {
            return Err(c::Code::BadRequest);
        }
    }
    Ok(())
}
fn publish_page(
    output: &Dir,
    page: &w::Page,
    refs: &mut Vec<w::PageRef>,
    total: &mut usize,
    b: &mut Budget,
) -> Result<(), c::Code> {
    let bytes = b.unit(|| w::canonical_limited(page, w::MAX_PAGE).map_err(internal))?;
    if *total + bytes.len() > w::MAX_SNAPSHOT {
        return Err(c::Code::ResourceLimit);
    }
    let hash = b.unit(|| Ok(w::hash(&bytes)))?;
    output.publish(
        &format!("page-{:06}.json", page.index),
        &bytes,
        &page.nonce,
        b,
    )?;
    refs.push(w::PageRef {
        index: page.index,
        len: w::Number(bytes.len() as u64),
        sha256: hash,
    });
    *total += bytes.len();
    Ok(())
}
