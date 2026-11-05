// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Closed, selected, verified-at-revision publication exporter. No live Recall effect.
use super::{c1_wire as c, read_wire as w, worker::*};
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
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
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
    fn prestart() -> bool {
        true
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
/// Non-Darwin fallback; the normal Darwin route dispatches before runtime startup.
pub async fn cli(mut args: impl Iterator<Item = String>) -> i32 {
    if args.next().as_deref() != Some("--job-dir") {
        return 2;
    }
    let Some(path) = args.next().map(PathBuf::from) else {
        return 2;
    };
    if args.next().is_some() || !path.is_absolute() {
        return 2;
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let Ok(owner) = std::thread::Builder::new()
        .name("dream-read-control".into())
        .spawn(move || {
            let _ = tx.send(run::<ReadJob>(path, Arc::new(AtomicU64::new(0))));
        })
    else {
        return 1;
    };
    let code = rx.await.unwrap_or(1);
    while !owner.is_finished() {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    if owner.join().is_err() { 1 } else { code }
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
    Budget {
        control,
        progress,
        counters: Counters {
            phase: c::Phase::Validate,
            bytes: 0,
            envelopes: 0,
            cpu: 0,
        },
        max_unit: 0,
        heap_peak: 0,
        overruns: 0,
        last_end: None,
        paused: false,
        normalization_ready: false,
        normalization_units: [0; 4],
    }
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
        let mut seen = BTreeMap::new();
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
                for _ in 0..64 {
                    if let Some(parsed_shape) = shape.take() {
                        if count >= 65536 {
                            return Err(c::Code::ResourceLimit);
                        }
                        let parsed_shape: storage::Shape = parsed_shape;
                        let mut credit = memory.reserve(parsed_shape.typed(line.len())?)?;
                        let event = Envelope::from_line(
                            std::str::from_utf8(&line).map_err(|_| c::Code::MalformedIr)?,
                        )
                        .map_err(|_| c::Code::MalformedIr)?;
                        seq += 1;
                        super::inspect::validate_event(&event, source, seq)?;
                        law.observe(&event).map_err(|_| c::Code::MalformedIr)?;
                        last_hash = w::hash(line.strip_suffix(b"\n").ok_or(c::Code::MalformedIr)?);
                        for blob in super::inspect::blobs(&event.event)? {
                            if let Some(old) = seen.insert(blob.sha256.clone(), blob.clone())
                                && old != *blob
                            {
                                return Err(c::Code::BlobMismatch);
                            }
                        }
                        log.try_reserve(1).map_err(|_| c::Code::ResourceLimit)?;
                        log.push(event);
                        credit.shrink(
                            credit
                                .bytes
                                .min(retained_capacity(parsed_shape, line.len())?),
                        );
                        result.credits.push(credit);
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
                    if cpu_ns().saturating_sub(started) >= 400_000 {
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
    accounted: u64,
    steps: u16,
    credits: Rc<RefCell<Vec<storage::Credit>>>,
    helper_clock: Option<u64>,
    helper_summary: Rc<RefCell<HelperSummary>>,
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
    last_errno: i32,
    last_failed_sequence: u16,
    native_cpu: u64,
    native_max_cpu: u64,
    native_max_wall: u64,
    inclusive_max_cpu: u64,
    inclusive_max_wall: u64,
    app_max_cpu: u64,
    creation_debt: u64,
    settled: u64,
    rolled_cpu: u64,
    reaped_rss: u64,
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
        let m = &self.totals;
        eprintln!(
            "dream-read helper_lifetime profile=read-helper-native/1 settled={} grants={} charge_ns={} max_charge_ns={} debt_ns={} controller_cpu_ns={} pipe_cpu_ns={} observed_direct_cpu_ns={} observed_descendant_cpu_ns={} rolled_descendant_cpu_ns={} reaped_rollup_cpu_ns={} descendants={} coverage_known={}",
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
            m.descendants,
            m.coverage_known
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
    }
}
/// Unit witnesses wrap this real adapter rather than substituting a test
/// scheduler. The controller and owned native-helper path remain production code.
#[cfg(test)]
pub(crate) fn helper_fixture_clock(control: Arc<Control>) -> Box<dyn super::resolve::ProofPace> {
    low_priority().expect("private helper fixture priority");
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
        accounted: 0,
        steps: 0,
        credits,
        helper_clock: None,
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
    fn native_created(
        &mut self,
        span: super::resolve::NativeCreation,
    ) -> Result<(), super::entry::Invalid> {
        self.started = None;
        self.epoch = None;
        self.accounted = 0;
        self.steps = 0;
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
        let control = {
            let mut b = self.budget.borrow_mut();
            b.max_unit = b.max_unit.max(span.inclusive_cpu_ns);
            if known {
                b.counters.cpu = b
                    .counters
                    .cpu
                    .checked_add(span.inclusive_cpu_ns)
                    .ok_or(super::entry::Invalid("selected CPU counter overflow"))?;
            }
            b.heap_peak = b.heap_peak.max(heaps().0);
            b.control.clone()
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
        // The suspended effect remains attached throughout payment, including
        // cancellation and failed native verdicts. No executable grant is ready.
        if let Some(debt) = debt.filter(|_| known) {
            let until = Instant::now().checked_add(Duration::from_nanos(debt));
            if let Some(until) = until {
                while Instant::now() < until {
                    std::thread::sleep(
                        until
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(1)),
                    );
                }
            }
        }
        if let Some(code) = code.or_else(|| control.error()) {
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
            h.ready = true;
            h.debt_until = Instant::now();
        }
        self.helper_clock = Some(cpu_ns());
        Ok(())
    }
    fn boundary(&mut self) -> Result<(), super::entry::Invalid> {
        super::resolve::ProofPace::after(self)?;
        self.budget.borrow_mut().last_end = Some(Instant::now());
        self.epoch = None;
        self.accounted = 0;
        self.steps = 0;
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
        pgid: i32,
        cpu_ns: u64,
        rss_bytes: u64,
    ) -> Result<(), super::entry::Invalid> {
        let control = self.budget.borrow().control.clone();
        let mut helper = control
            .helper
            .lock()
            .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
        let h = helper
            .as_mut()
            .filter(|h| h.pgid == pgid)
            .ok_or(super::entry::Invalid("helper usage custody"))?;
        let mut summary = self.helper_summary.borrow_mut();
        summary.rolled_cpu = summary
            .rolled_cpu
            .checked_add(cpu_ns)
            .ok_or(super::entry::Invalid("helper usage overflow"))?;
        summary.reaped_rss = summary.reaped_rss.max(rss_bytes);
        // Conservative grants already cover sampled/reaped CPU; reconciliation
        // can only add debt, never refund earlier charges or stopped intervals.
        if cpu_ns > h.metrics.charged_ns {
            let extra = (cpu_ns - h.metrics.charged_ns)
                .checked_mul(9)
                .ok_or(super::entry::Invalid("helper debt overflow"))?;
            h.metrics.debt_ns = h
                .metrics
                .debt_ns
                .checked_add(extra)
                .ok_or(super::entry::Invalid("helper debt overflow"))?;
            h.debt_until = h
                .debt_until
                .max(Instant::now())
                .checked_add(Duration::from_nanos(extra))
                .ok_or(super::entry::Invalid("helper debt overflow"))?;
        }
        if rss_bytes > storage::HELPERS {
            *self.failure.borrow_mut() = Some(c::Code::ResourceLimit);
            return Err(super::entry::Invalid("selected helper memory bound"));
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
        let h = helper.as_mut().unwrap();
        if h.sample(&b.control) != Ok(true) {
            h.ready = false;
            *self.failure.borrow_mut() = Some(c::Code::SourceUnavailable);
            return Err(super::entry::Invalid("helper was not born stopped"));
        }
        Ok(())
    }
    fn helper_retire(&mut self, pgid: i32) -> Result<(), super::entry::Invalid> {
        let control = self.budget.borrow().control.clone();
        let mut helper = control.helper.lock().unwrap_or_else(|poisoned| {
            control.cancel.store(4, Ordering::Release);
            poisoned.into_inner()
        });
        let h = helper
            .as_mut()
            .filter(|h| h.pgid == pgid)
            .ok_or(super::entry::Invalid("helper custody mismatch"))?;
        h.ready = false;
        // Retirement precedes the owner's last group kill and wait4. If a
        // refusal arrived during a runnable grant, retain its entire observed
        // wall/CPU excess rather than calling the interval stopped or free.
        if h.running {
            let at = cpu_ns();
            unsafe {
                libc::kill(-pgid, libc::SIGSTOP);
            }
            if h.sample(&control) == Ok(true) {
                let end = cpu_ns();
                let used = end
                    .checked_sub(at)
                    .filter(|_| at != u64::MAX && end != u64::MAX)
                    .unwrap_or(u64::MAX);
                h.finish_grant(&control, used);
            } else {
                // No fabricated stop acknowledgement: retain the full unknown
                // grant and refuse trust while the pinned owner kills/reaps it.
                let wall = u64::try_from(h.changed.elapsed().as_nanos()).unwrap_or(u64::MAX);
                let charge = wall
                    .checked_mul(h.identity.expected_members as u64)
                    .and_then(|n| n.checked_add(h.lane_cpu))
                    .unwrap_or(u64::MAX);
                h.metrics.charged_ns = h.metrics.charged_ns.saturating_add(charge);
                h.metrics.max_charge_ns = h.metrics.max_charge_ns.max(charge);
                h.metrics.coverage_known = false;
                if self.failure.borrow().is_none() {
                    *self.failure.borrow_mut() =
                        Some(control.error().unwrap_or(c::Code::SourceUnavailable));
                }
            }
        } else if h.sample(&control) != Ok(true) {
            h.metrics.coverage_known = false;
            if self.failure.borrow().is_none() {
                *self.failure.borrow_mut() =
                    Some(control.error().unwrap_or(c::Code::SourceUnavailable));
            }
        }
        h.retiring = true;
        Ok(())
    }
    fn helper_detach(&mut self, pgid: i32) -> Result<(), super::entry::Invalid> {
        let b = self.budget.borrow();
        let mut helper = b.control.helper.lock().unwrap_or_else(|poisoned| {
            b.control.cancel.store(4, Ordering::Release);
            poisoned.into_inner()
        });
        if !helper
            .as_ref()
            .is_some_and(|h| h.pgid == pgid && h.retiring)
        {
            return Err(super::entry::Invalid("helper settlement mismatch"));
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
                || h.metrics.rolled_descendant_cpu_ns < h.metrics.observed_descendant_cpu_ns)
        {
            h.metrics.coverage_known = false;
            if self.failure.borrow().is_none() {
                *self.failure.borrow_mut() =
                    Some(b.control.error().unwrap_or(c::Code::SourceUnavailable));
            }
        }
        let mut summary = self.helper_summary.borrow_mut();
        summary.settled += 1;
        let m = &mut summary.totals;
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
        m.coverage_known &= h.metrics.coverage_known;
        self.helper_clock = None;
        Ok(())
    }
    fn helper_poll(&mut self, pgid: i32) -> Result<(), super::entry::Invalid> {
        let b = self.budget.borrow();
        let mut helper = b
            .control
            .helper
            .lock()
            .map_err(|_| super::entry::Invalid("helper custody unavailable"))?;
        let h = helper
            .as_mut()
            .filter(|h| h.pgid == pgid)
            .ok_or(super::entry::Invalid("helper custody mismatch"))?;
        let now = cpu_ns();
        let used = self
            .helper_clock
            .replace(now)
            .and_then(|at| now.checked_sub(at))
            .filter(|_| now != u64::MAX)
            .ok_or(super::entry::Invalid("helper pipe clock unknown"))?;
        h.metrics.pipe_cpu_ns = h
            .metrics
            .pipe_cpu_ns
            .checked_add(used)
            .ok_or(super::entry::Invalid("helper pipe CPU overflow"))?;
        h.lane_cpu = h
            .lane_cpu
            .checked_add(used)
            .ok_or(super::entry::Invalid("helper group CPU overflow"))?;
        if !h.running {
            let debt = used
                .checked_mul(9)
                .ok_or(super::entry::Invalid("helper pipe debt overflow"))?;
            h.metrics.debt_ns = h
                .metrics
                .debt_ns
                .checked_add(debt)
                .ok_or(super::entry::Invalid("helper pipe debt overflow"))?;
            h.debt_until = h
                .debt_until
                .checked_add(Duration::from_nanos(debt))
                .ok_or(super::entry::Invalid("helper pipe debt overflow"))?;
        }
        if used > 1_000_000 || h.lane_cpu > 1_000_000 {
            *self.failure.borrow_mut() = Some(c::Code::ResourceLimit);
            b.control.cancel.store(4, Ordering::Release);
            return Err(super::entry::Invalid("helper pipe CPU bound"));
        }
        if let Some(code) = b.control.error() {
            if self.failure.borrow().is_none() {
                *self.failure.borrow_mut() = Some(code);
            }
            return Err(super::entry::Invalid("helper stopped"));
        }
        drop(helper);
        drop(b);
        // The proof owner performs only bounded pipe IO between these parks;
        // the existing independent owner grants/stops the helper meanwhile.
        std::thread::sleep(Duration::from_millis(1));
        Ok(())
    }

    fn cleanup(&mut self, step: &mut dyn FnMut() -> bool) {
        let running = self.started.is_some();
        let _ = super::resolve::ProofPace::after(self);
        let control = self.budget.borrow().control.clone();
        let mut more = true;
        while more {
            cleanup_unit(&control, || {
                let start = cpu_ns();
                for _ in 0..1024 {
                    more = step();
                    if !more || cpu_ns().saturating_sub(start) >= 400_000 {
                        break;
                    }
                }
            });
        }
        if running {
            self.started = Some(cpu_ns());
            self.epoch = self.started;
            self.accounted = 0;
            self.steps = 0;
        }
        // Cleanup uses its independent clock even after a failed proof clock.
        if control.cleanup_max.load(Ordering::Acquire) > 1_000_000 {
            *self.failure.borrow_mut() = Some(c::Code::ResourceLimit);
        }
    }

    fn before(&mut self) -> Result<(), super::entry::Invalid> {
        if self.failure.borrow().is_some() {
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        if self
            .budget
            .borrow()
            .control
            .helper
            .lock()
            .map_or(true, |h| h.is_some())
        {
            *self.failure.borrow_mut() = Some(c::Code::SourceUnavailable);
            return Err(super::entry::Invalid("main proof overlaps helper owner"));
        }
        let foreground = self
            .budget
            .borrow()
            .control
            .foreground
            .load(Ordering::Acquire);
        if self.epoch.is_none() || self.accounted >= 400_000 || self.steps >= 64 || foreground {
            let result = self.budget.borrow_mut().unit(|| Ok(()));
            if let Err(code) = result {
                *self.failure.borrow_mut() = Some(code);
                return Err(super::entry::Invalid("selected worker stopped"));
            }
            self.epoch = Some(cpu_ns());
            self.accounted = 0;
            self.steps = 0;
        } else if let Some(code) = self.budget.borrow().control.error() {
            *self.failure.borrow_mut() = Some(code);
            return Err(super::entry::Invalid("selected worker stopped"));
        }
        self.started = Some(cpu_ns());
        Ok(())
    }
    fn after(&mut self) -> Result<(), super::entry::Invalid> {
        if self.started.take().is_none() {
            return Ok(());
        }
        let end = cpu_ns();
        let start = self.epoch.unwrap_or(u64::MAX);
        let used = if start == u64::MAX || end == u64::MAX {
            u64::MAX
        } else {
            end.checked_sub(start).unwrap_or(u64::MAX)
        };
        self.steps = self.steps.saturating_add(1);
        let mut b = self.budget.borrow_mut();
        b.max_unit = b.max_unit.max(used);
        if used != u64::MAX {
            b.counters.cpu = b
                .counters
                .cpu
                .checked_add(used.saturating_sub(self.accounted))
                .ok_or(super::entry::Invalid("selected CPU counter overflow"))?;
        }
        {
            let mut summary = self.helper_summary.borrow_mut();
            summary.cost_known &= used != u64::MAX;
            summary.proof_app_max_cpu = summary.proof_app_max_cpu.max(used);
        }
        b.heap_peak = b.heap_peak.max(heaps().0);
        self.accounted = used;
        if used >= 400_000 || self.steps >= 64 {
            b.last_end = Some(Instant::now());
        }
        let code = if used > 1_000_000 || b.heap_peak > 96 * 1024 * 1024 {
            b.overruns += 1;
            Some(c::Code::ResourceLimit)
        } else {
            b.control.error()
        };
        if let Some(code) = code {
            *self.failure.borrow_mut() = Some(code);
            Err(super::entry::Invalid("selected worker stopped"))
        } else {
            Ok(())
        }
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
                    accounted: 0,
                    steps: 0,
                    credits: proof_credits.clone(),
                    helper_clock: None,
                    helper_summary: helper_summary.clone(),
                }),
            )
            .map_err(|_| c::Code::ProtocolError)?;
            let mut qualified=Paced::new((|| ->Result<Vec<(String,super::recall::PreparedLesson)>,super::seal::ProofError> {
                let trust=selected_ready(super::trust::Trust::load_selected(&policy,roots.clone()))??;
                let mut trust=Paced::new(trust,&control,Drain::drain_step);
                let owned=std::mem::replace(&mut logs.logs.0,BTreeMap::new());
                let resolver=super::resolve::Resolver::from_selected(domains.sessions.clone(),roots,owned,domains)?.with_trust(trust.take_value());
                let mut resolver=Paced::new(resolver,&control,|r|r.drain_step());
                let mut groups=BTreeMap::<String,Vec<crate::protocol::EvidenceRef>>::new();
                for pin in &manifest.publications {
                    let event=selected_ready(resolver.event_async(&pin.clone().into()))??;
                    let Event::ItemCompleted{body:crate::protocol::Item::Receipt{harness:Some(harness),..},..}=&event.event else {
                        return Err(super::seal::ProofError::Fail("publication carrier"));
                    };
                    if !matches!(harness.as_str(),"engine"|"acp") {return Err(super::seal::ProofError::Unproven("harness_incompatible"));}
                    groups.entry(harness.clone()).or_default().push(pin.clone().into());
                }
                let mut result=vec![];
                for (harness,requested) in groups {
                    let lessons=selected_ready(super::recall::prepare_selected(&mut resolver,&requested,&harness,&repo,&crate::redact::Set::default(),manifest.evidence.selected_at_ms.0))??;
                    for lesson in lessons {result.push((harness.clone(),lesson));}
                }
                Ok(result)
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
                    for (index, pin) in lesson.evidence.into_iter().take(15).enumerate() {
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
                            title: lesson.identity.title,
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
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                trial: lesson
                                    .trial
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                publication: lesson
                                    .publication
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                outer: lesson
                                    .outer
                                    .try_into()
                                    .map_err(|_| c::Code::ProtocolError)?,
                                projection: prefixed_hex(&lesson.projection_digest)?,
                                trust_policy: manifest.trust_policy.clone(),
                                producer: producer_hash,
                                currentness: prefixed_hex(&lesson.currentness_digest)?,
                            },
                        },
                        why: lesson.identity.why,
                        how_to_apply: lesson.identity.how_to_apply,
                        context: lesson.context,
                        citation: lesson.citation,
                        nodes,
                        edges,
                    };
                    Ok(prepared)
                })?;
                match b.borrow_mut().unit(|| Ok(w::validate_entry(&prepared)))? {
                    Ok(()) => entries.push(prepared),
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
                let last = page.entries.pop().unwrap();
                publish_page(&output, &page, &mut refs, &mut total, &mut b.borrow_mut())?;
                page.index += 1;
                page.entries = vec![last];
                page.next = Some(page.index + 1);
            }
            entry_ids.push(entry_id);
        }
        page.next = None;
        publish_page(&output, &page, &mut refs, &mut total, &mut b.borrow_mut())?;
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
    if control.cleanup_max.load(Ordering::Acquire) > 1_000_000 {
        return Err(err(w::Code::ResourceLimit));
    }
    result.map_err(|code| {
        eprintln!(
            "dream-read refused stage={stage} code={code:?} wall_ms={}",
            wall.elapsed().as_millis()
        );
        w::Error {
            code: proof_error.unwrap_or_else(|| external(code)),
            source: source_index.take(),
        }
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
    fn owner() -> (ProofClock, Rc<RefCell<HelperSummary>>, Arc<Control>) {
        low_priority().unwrap();
        let control = Arc::new(Control {
            cancel: std::sync::atomic::AtomicU8::new(0),
            foreground: std::sync::atomic::AtomicBool::new(false),
            deadline: Instant::now() + Duration::from_secs(2),
            reactor: Arc::new(AtomicU64::new(0)),
            memory: storage::Ledger::new(),
            cleanup_cpu: AtomicU64::new(0),
            cleanup_max: AtomicU64::new(0),
            cleanup_overruns: AtomicU64::new(0),
            helper: std::sync::Mutex::new(None),
        });
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
                accounted: 0,
                steps: 0,
                credits: Rc::new(RefCell::new(vec![])),
                helper_clock: None,
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
    fn cancellation_does_not_reset_creation_debt() {
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
        assert_eq!(*clock.failure.borrow(), Some(c::Code::Cancelled));
    }
    #[test]
    fn native_clock_loss_retains_unknown_raw_cost_without_zero_certification() {
        let (mut clock, summary, _) = owner();
        assert!(
            clock
                .native_created(span(u64::MAX, u64::MAX, u64::MAX))
                .is_err()
        );
        assert!(!summary.borrow().cost_known);
        assert_eq!(summary.borrow().inclusive_max_cpu, u64::MAX);
        assert_eq!(*clock.failure.borrow(), Some(c::Code::SourceUnavailable));
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
