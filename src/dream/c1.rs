// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Private non-seat C1 worker. No shared blocking pool, HOME, provider or discovery.
use super::{c1_wire as w, d0, inspect};
use crate::protocol::{Envelope, Event};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::CString,
    fs::{File, Metadata},
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, RawFd},
        unix::fs::MetadataExt,
    },
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const CHUNK: usize = 64 * 1024;
const INPUT_MAX: u64 = 64 * 1024 * 1024;
const HEAP_MAX: u64 = 128 * 1024 * 1024;
const UNIT_NS: u64 = 1_000_000;
const WAIT: Duration = Duration::from_millis(9);
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum DreamOwner {
    #[default]
    Legacy,
    TermC1,
}
impl DreamOwner {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "term-c1" => Some(Self::TermC1),
            _ => None,
        }
    }
    pub fn legacy(self) -> bool {
        self == Self::Legacy
    }
}

/// Dispatch before cwd/HOME/profile/capability initialization. Join happens off reactor.
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
    let reactor = Arc::new(AtomicU64::new(0));
    let (code, maximum) = observe_reactor(serve(path, reactor.clone()), reactor).await;
    eprintln!("dream-c1 reactor_max_cpu_ns={maximum}");
    if maximum > UNIT_NS { 1 } else { code }
}
async fn observe_reactor<T>(
    future: impl std::future::Future<Output = T>,
    maximum: Arc<AtomicU64>,
) -> (T, u64) {
    let mut future = std::pin::pin!(future);
    let result = std::future::poll_fn(|cx| {
        let start = cpu_ns();
        let poll = future.as_mut().poll(cx);
        let end = cpu_ns();
        let used = if start == u64::MAX || end == u64::MAX {
            u64::MAX
        } else {
            end.saturating_sub(start)
        };
        maximum.fetch_max(used, Ordering::AcqRel);
        poll
    })
    .await;
    (result, maximum.load(Ordering::Acquire))
}
async fn serve(path: PathBuf, reactor: Arc<AtomicU64>) -> i32 {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let Ok(owner) = thread::Builder::new()
        .name("dream-c1-control".into())
        .spawn(move || {
            let _ = tx.send(run(path, reactor));
        })
    else {
        return 1;
    };
    // Compute joins on its control owner. Observe control-thread exit before
    // joining the already-finished handle; no wait/reap syscall on the reactor.
    let code = rx.await.unwrap_or(1);
    while !owner.is_finished() {
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    if owner.join().is_err() { 1 } else { code }
}
fn cpu_ns() -> u64 {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, t.as_mut_ptr()) } != 0 {
        return u64::MAX;
    }
    let t = unsafe { t.assume_init() };
    t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
}
#[cfg(target_os = "macos")]
fn heap() -> u64 {
    #[repr(C)]
    struct Statistics {
        blocks: u32,
        size: usize,
        peak: usize,
        allocated: usize,
    }
    unsafe extern "C" {
        fn malloc_zone_statistics(zone: *mut libc::c_void, stats: *mut Statistics);
    }
    let mut stats = Statistics {
        blocks: 0,
        size: 0,
        peak: 0,
        allocated: 0,
    };
    unsafe { malloc_zone_statistics(std::ptr::null_mut(), &mut stats) };
    stats.peak.max(stats.size) as u64
}
#[cfg(not(target_os = "macos"))]
fn heap() -> u64 {
    // No unmeasured heap certification on a platform without this allocator probe.
    u64::MAX
}
fn rss() -> u64 {
    let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) } != 0 {
        return 0;
    }
    let r = unsafe { r.assume_init() };
    if cfg!(target_os = "macos") {
        r.ru_maxrss as u64
    } else {
        r.ru_maxrss as u64 * 1024
    }
}
struct Control {
    cancel: AtomicU8,
    foreground: AtomicBool,
    deadline: Instant,
    reactor: Arc<AtomicU64>,
}
impl Control {
    fn error(&self) -> Option<w::Code> {
        if self.reactor.load(Ordering::Acquire) > UNIT_NS {
            return Some(w::Code::ResourceLimit);
        }
        match self.cancel.load(Ordering::Acquire) {
            1 => Some(w::Code::Cancelled),
            2 => Some(w::Code::Deadline),
            3 => Some(w::Code::ProtocolError),
            4 => Some(w::Code::ResourceLimit),
            _ if Instant::now() >= self.deadline => Some(w::Code::Deadline),
            _ => None,
        }
    }
}
struct Budget {
    control: Arc<Control>,
    progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    counters: Counters,
    max_unit: u64,
    heap_peak: u64,
    overruns: u64,
    last_end: Option<Instant>,
    paused: bool,
    normalization_ready: bool,
    normalization_units: [u64; 4],
}
#[derive(Clone, Copy)]
struct Counters {
    phase: w::Phase,
    bytes: u64,
    envelopes: u64,
    cpu: u64,
}
impl Budget {
    fn announce(&mut self, phase: w::Phase) {
        self.counters.phase = phase;
        self.progress.force_push(self.counters);
    }
    #[track_caller]
    fn unit<T>(&mut self, f: impl FnOnce() -> Result<T, w::Code>) -> Result<T, w::Code> {
        if let Some(end) = self.last_end {
            let elapsed = end.elapsed();
            if elapsed < WAIT {
                thread::sleep(WAIT - elapsed)
            }
        }
        loop {
            if let Some(e) = self.control.error() {
                return Err(e);
            }
            if !self.control.foreground.load(Ordering::Acquire) {
                break;
            }
            if !self.paused {
                self.paused = true;
                self.announce(w::Phase::Paused)
            }
            thread::sleep(Duration::from_millis(2));
        }
        if self.paused {
            self.paused = false;
            self.announce(w::Phase::Validate)
        }
        let live = heap();
        self.heap_peak = self.heap_peak.max(live);
        // Reserve an upper bound for the next bounded record's parser allocations.
        if live > HEAP_MAX - 16 * 1024 * 1024 {
            return Err(w::Code::ResourceLimit);
        }
        let start = cpu_ns();
        if start == u64::MAX {
            return Err(w::Code::ResourceLimit);
        }
        let result = f();
        let end = cpu_ns();
        if end == u64::MAX {
            return Err(w::Code::ResourceLimit);
        }
        let elapsed = end.saturating_sub(start);
        self.last_end = Some(Instant::now());
        self.max_unit = self.max_unit.max(elapsed);
        self.counters.cpu = self.counters.cpu.saturating_add(elapsed);
        self.heap_peak = self.heap_peak.max(heap());
        if elapsed > UNIT_NS || self.heap_peak > HEAP_MAX {
            self.overruns += 1;
            eprintln!(
                "dream-c1 rejected_unit line={} phase={:?} cpu_ns={elapsed}",
                std::panic::Location::caller().line(),
                self.counters.phase
            );
            return Err(w::Code::ResourceLimit);
        }
        if let Some(e) = self.control.error() {
            return Err(e);
        }
        result
    }
}
struct Dir(File);
impl Dir {
    fn open(path: &Path) -> Result<Self, w::Code> {
        let mut fd = File::open("/").map_err(|_| w::Code::StorageUnavailable)?;
        for c in path.components() {
            match c {
                Component::RootDir => {}
                Component::Normal(n) => {
                    let n = CString::new(n.as_encoded_bytes()).map_err(|_| w::Code::BadRequest)?;
                    let next = unsafe {
                        libc::openat(
                            fd.as_raw_fd(),
                            n.as_ptr(),
                            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                        )
                    };
                    if next < 0 {
                        return Err(w::Code::StorageUnavailable);
                    }
                    fd = unsafe { File::from_raw_fd(next) };
                }
                _ => return Err(w::Code::BadRequest),
            }
        }
        let dir = Self(fd);
        dir.check()?;
        Ok(dir)
    }
    fn check(&self) -> Result<(), w::Code> {
        let m = self.0.metadata().map_err(|_| w::Code::StorageUnavailable)?;
        if m.is_dir() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0 {
            Ok(())
        } else {
            Err(w::Code::StorageUnavailable)
        }
    }
    fn dir(&self, name: &str) -> Result<Self, w::Code> {
        let file = self.openat(name, libc::O_RDONLY | libc::O_DIRECTORY, 0)?;
        let d = Self(file);
        d.check()?;
        Ok(d)
    }
    fn openat(&self, name: &str, flags: i32, mode: u32) -> Result<File, w::Code> {
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return Err(w::Code::StorageUnavailable);
        }
        let n = CString::new(name).map_err(|_| w::Code::StorageUnavailable)?;
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                n.as_ptr(),
                flags | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
                mode,
            )
        };
        if fd < 0 {
            return Err(w::Code::StorageUnavailable);
        }
        Ok(unsafe { File::from_raw_fd(fd) })
    }
    fn file(&self, name: &str, max: u64) -> Result<(File, Metadata), w::Code> {
        let f = self.openat(name, libc::O_RDONLY, 0).map_err(|e| {
            if io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                w::Code::SourceUnavailable
            } else {
                e
            }
        })?;
        let m = f.metadata().map_err(|_| w::Code::SourceUnavailable)?;
        if !m.is_file()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
            || m.nlink() != 1
        {
            return Err(w::Code::StorageUnavailable);
        }
        if m.len() > max {
            return Err(w::Code::ResourceLimit);
        }
        Ok((f, m))
    }
    fn unchanged(&self, name: &str, f: &File, before: &Metadata) -> Result<(), w::Code> {
        let (_, path) = self.file(name, INPUT_MAX)?;
        let after = f.metadata().map_err(|_| w::Code::SourceChanged)?;
        if identity(before) == identity(&after) && identity(before) == identity(&path) {
            Ok(())
        } else {
            Err(w::Code::SourceChanged)
        }
    }
    /// Directory-fd exclusive rename publishes without replacement or alias links.
    fn publish(
        &self,
        name: &str,
        bytes: &[u8],
        nonce: &w::Ulid,
        b: &mut Budget,
    ) -> Result<(), w::Code> {
        let temp = format!(".{name}.{}.tmp", nonce.0);
        let mut file =
            b.unit(|| self.openat(&temp, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, 0o600))?;
        let before = file.metadata().map_err(|_| w::Code::StorageUnavailable)?;
        let result = (|| {
            for chunk in bytes.chunks(CHUNK) {
                b.unit(|| {
                    file.write_all(chunk)
                        .map_err(|_| w::Code::StorageUnavailable)
                })?;
            }
            b.unit(|| file.sync_all().map_err(|_| w::Code::StorageUnavailable))?;
            b.unit(|| {
                let (_, current) = self.file(&temp, bytes.len() as u64)?;
                if current.dev() != before.dev()
                    || current.ino() != before.ino()
                    || current.len() != bytes.len() as u64
                {
                    return Err(w::Code::StorageUnavailable);
                }
                let a = CString::new(temp.as_str()).unwrap();
                let dest = CString::new(name).unwrap();
                #[cfg(target_os = "macos")]
                let status = unsafe {
                    libc::renameatx_np(
                        self.0.as_raw_fd(),
                        a.as_ptr(),
                        self.0.as_raw_fd(),
                        dest.as_ptr(),
                        libc::RENAME_EXCL,
                    )
                };
                #[cfg(target_os = "linux")]
                let status = unsafe {
                    libc::renameat2(
                        self.0.as_raw_fd(),
                        a.as_ptr(),
                        self.0.as_raw_fd(),
                        dest.as_ptr(),
                        libc::RENAME_NOREPLACE,
                    )
                };
                #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                let status = -1;
                if status != 0 {
                    return Err(w::Code::StorageUnavailable);
                }
                Ok(())
            })?;
            b.unit(|| self.0.sync_all().map_err(|_| w::Code::StorageUnavailable))
        })();
        // Only this exclusive-created inode can be unlinked, even on cancellation.
        let a = CString::new(temp.as_str()).unwrap();
        let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe {
            libc::fstatat(
                self.0.as_raw_fd(),
                a.as_ptr(),
                st.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
        {
            let st = unsafe { st.assume_init() };
            if st.st_dev as u64 == before.dev() && st.st_ino == before.ino() {
                unsafe { libc::unlinkat(self.0.as_raw_fd(), a.as_ptr(), 0) };
            }
        }
        result
    }
}
fn identity(m: &Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
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
fn read_bounded(dir: &Dir, name: &str, max: u64, b: &mut Budget) -> Result<Vec<u8>, w::Code> {
    let (mut file, before) = b.unit(|| dir.file(name, max))?;
    let mut bytes = b.unit(|| Ok(Vec::with_capacity(before.len() as usize)))?;
    let mut chunk = [0u8; CHUNK];
    loop {
        let n = b.unit(|| {
            file.read(&mut chunk)
                .map_err(|_| w::Code::SourceUnavailable)
        })?;
        if n == 0 {
            break;
        }
        if bytes.len() + n > max as usize {
            return Err(w::Code::ResourceLimit);
        }
        b.unit(|| {
            bytes.extend_from_slice(&chunk[..n]);
            Ok(())
        })?;
    }
    b.unit(|| dir.unchanged(name, &file, &before))?;
    Ok(bytes)
}
fn validate_manifest(m: &w::Manifest, start: &w::Start) -> Result<(), w::Code> {
    if m.run_id != start.run_id || m.nonce != start.nonce {
        return Err(w::Code::ProtocolError);
    }
    if m.context.generation.0 == 0
        || m.context.grant_epoch.0 == 0
        || m.sources.is_empty()
        || m.sources.len() > 32
    {
        return Err(w::Code::BadRequest);
    }
    let mut total = 0u64;
    let mut refs = 0usize;
    let mut prior = None;
    let mut ids = BTreeSet::new();
    for s in &m.sources {
        let key = (s.source_generation, s.session.clone());
        if s.source_generation.0 == 0
            || s.closed_seq.0 == 0
            || prior.as_ref().is_some_and(|p| p >= &key)
            || !ids.insert(&s.session)
        {
            return Err(w::Code::BadRequest);
        }
        prior = Some(key);
        total = total.checked_add(s.len.0).ok_or(w::Code::ResourceLimit)?;
        let mut hash = None;
        for r in &s.blobs {
            if hash.is_some_and(|p| p >= &r.sha256) {
                return Err(w::Code::BlobMismatch);
            }
            hash = Some(&r.sha256);
            total = total.checked_add(r.len.0).ok_or(w::Code::ResourceLimit)?;
            refs += 1;
        }
    }
    if total > INPUT_MAX || refs > 1024 {
        return Err(w::Code::ResourceLimit);
    }
    Ok(())
}
fn selected(input: &Dir, source: &w::Source, b: &mut Budget) -> Result<d0::ClosedLog, w::Code> {
    let name = format!("{}.jsonl", source.session.0);
    let (mut file, before) = b.unit(|| input.file(&name, INPUT_MAX))?;
    if before.len() != source.len.0 {
        return Err(w::Code::SourceChanged);
    }
    let mut hasher = Sha256::new();
    let mut chunk = [0u8; CHUNK];
    let mut line = b.unit(|| Ok(Vec::with_capacity(w::MAX_FRAME)))?;
    let mut law = inspect::Transcript::default();
    let mut log = None;
    let mut seen = BTreeMap::new();
    let mut seq = 0;
    let mut length = 0;
    let mut last = None;
    let redactor = crate::redact::Set::default();
    loop {
        let n = b.unit(|| {
            let n = file
                .read(&mut chunk)
                .map_err(|_| w::Code::SourceUnavailable)?;
            hasher.update(&chunk[..n]);
            Ok(n)
        })?;
        if n == 0 {
            break;
        }
        length += n as u64;
        if length > source.len.0 {
            return Err(w::Code::SourceChanged);
        }
        b.counters.bytes += n as u64;
        let mut pos = 0;
        let mut envelope_count = b.counters.envelopes;
        while pos < n {
            b.unit(|| {
                let end = chunk[pos..n]
                    .iter()
                    .position(|c| *c == b'\n')
                    .map(|i| pos + i + 1)
                    .unwrap_or(n);
                if line.len() + end - pos > w::MAX_FRAME {
                    return Err(w::Code::ResourceLimit);
                }
                line.extend_from_slice(&chunk[pos..end]);
                pos = end;
                if line.last() != Some(&b'\n') {
                    return Ok(());
                }
                seq += 1;
                envelope_count += 1;
                if envelope_count > 65536 {
                    return Err(w::Code::ResourceLimit);
                }
                let raw = std::str::from_utf8(&line).map_err(|_| w::Code::MalformedIr)?;
                let event = Envelope::from_line(raw).map_err(|_| w::Code::MalformedIr)?;
                inspect::validate_event(&event, source, seq)?;
                law.observe(&event).map_err(|_| w::Code::MalformedIr)?;
                if let Event::SessionStarted { meta } = &event.event {
                    log = Some(d0::ClosedLog::new(meta.clone()));
                }
                for r in inspect::blobs(&event.event) {
                    if r.sha256.len() != 64
                        || w::Hex::try_from(r.sha256.clone()).is_err()
                        || r.mime.len() > 256
                        || r.len > w::MAX_SAFE
                    {
                        return Err(w::Code::BlobMismatch);
                    }
                    if let Some(prior) = seen.insert(r.sha256.clone(), r.clone())
                        && prior != *r
                    {
                        return Err(w::Code::BlobMismatch);
                    }
                    if seen.len() > 1024 {
                        return Err(w::Code::ResourceLimit);
                    }
                }
                last = Some((
                    matches!(event.event, Event::SessionClosed),
                    w::hash(&line[..line.len() - 1]),
                ));
                log.as_mut()
                    .ok_or(w::Code::MalformedIr)?
                    .observe(&event, &redactor, false)
                    .map_err(|_| w::Code::ResourceLimit)?;
                line.clear();
                Ok(())
            })?;
            b.counters.envelopes = envelope_count;
            if log.is_some() && !b.normalization_ready {
                for i in 0..super::fingerprint::NORMALIZE_MASKS {
                    let before = b.counters.cpu;
                    b.unit(|| {
                        super::fingerprint::warm_normalize_mask(i);
                        Ok(())
                    })?;
                    b.normalization_units[i] = b.counters.cpu - before;
                }
                b.normalization_ready = true;
            }
        }
        b.announce(w::Phase::Validate);
    }
    if !line.is_empty() || length != source.len.0 {
        return Err(w::Code::MalformedIr);
    }
    b.unit(|| input.unchanged(&name, &file, &before))?;
    b.unit(|| {
        if w::Hex(format!("{:x}", hasher.finalize())) != source.sha256 {
            return Err(w::Code::SourceChanged);
        }
        let Some((closed, digest)) = last else {
            return Err(w::Code::SourceOpen);
        };
        if !closed {
            return Err(w::Code::SourceOpen);
        }
        if seq != source.closed_seq.0 || digest != source.closed_line_sha256 {
            return Err(w::Code::SourceMismatch);
        }
        law.finish().map_err(|_| w::Code::MalformedIr)?;
        if seen.len() != source.blobs.len() {
            return Err(w::Code::BlobMismatch);
        }
        for r in &source.blobs {
            let Some(found) = seen.get(&r.sha256.0) else {
                return Err(w::Code::BlobMismatch);
            };
            if found.len != r.len.0 || found.mime != r.mime.0 {
                return Err(w::Code::BlobMismatch);
            }
        }
        Ok(())
    })?;
    if !source.blobs.is_empty() {
        let blobs = b.unit(|| input.dir(&source.session.0)?.dir("blobs"))?;
        for r in &source.blobs {
            let (mut f, meta) = b.unit(|| {
                blobs
                    .file(&r.sha256.0, r.len.0)
                    .map_err(|_| w::Code::BlobMismatch)
            })?;
            let mut hash = Sha256::new();
            let mut len = 0;
            loop {
                let n = b.unit(|| {
                    let n = f.read(&mut chunk).map_err(|_| w::Code::BlobMismatch)?;
                    hash.update(&chunk[..n]);
                    Ok(n)
                })?;
                if n == 0 {
                    break;
                }
                len += n as u64;
                if len > r.len.0 {
                    return Err(w::Code::BlobMismatch);
                }
                b.counters.bytes += n as u64;
            }
            b.unit(|| {
                blobs
                    .unchanged(&r.sha256.0, &f, &meta)
                    .map_err(|_| w::Code::BlobMismatch)?;
                if len != r.len.0 || w::Hex(format!("{:x}", hash.finalize())) != r.sha256 {
                    return Err(w::Code::BlobMismatch);
                }
                Ok(())
            })?;
        }
    }
    // Destruction of retained law/parser tables belongs to a measured unit too.
    b.unit(|| {
        drop((law, seen, line));
        Ok(())
    })?;
    log.ok_or(w::Code::MalformedIr)
}
struct Completed {
    hash: w::Hex,
    pages: u32,
}
fn compute(
    path: PathBuf,
    start: w::Start,
    control: Arc<Control>,
    progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
) -> Result<Completed, w::Error> {
    let mut b = Budget {
        control,
        progress,
        counters: Counters {
            phase: w::Phase::Validate,
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
    };
    let mut source_index = None;
    let result = (|| {
        if unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, 10) } != 0 {
            return Err(w::Code::ResourceLimit);
        }
        #[cfg(target_os = "macos")]
        {
            unsafe extern "C" {
                fn pthread_set_qos_class_self_np(class: u32, relative: i32) -> i32;
            }
            if unsafe { pthread_set_qos_class_self_np(0x11, 0) } != 0 {
                return Err(w::Code::ResourceLimit);
            }
        }
        let dir = b.unit(|| Dir::open(&path))?;
        let input = b.unit(|| dir.dir("input"))?;
        let output = b.unit(|| dir.dir("output"))?;
        let bytes = read_bounded(&input, "manifest.json", w::MAX_FRAME as u64, &mut b)?;
        let manifest = b.unit(|| {
            if w::hash(&bytes) != start.manifest_sha256 {
                return Err(w::Code::SourceMismatch);
            }
            let m = w::decode::<w::Manifest>(&bytes)?;
            validate_manifest(&m, &start)?;
            Ok(m)
        })?;
        let mut logs = vec![];
        for (i, s) in manifest.sources.iter().enumerate() {
            source_index = Some(i as u32);
            logs.push(selected(&input, s, &mut b)?);
        }
        source_index = None;
        b.announce(w::Phase::Replay);
        let mut unit_error = None;
        let rows = d0::inspect_closed(logs, |f| {
            b.unit(|| f().map_err(|_| w::Code::ResourceLimit))
                .map_err(|e| {
                    unit_error = Some(e);
                    io::Error::other("bounded inspection stopped")
                })
        })
        .map_err(|_| unit_error.unwrap_or(w::Code::ResourceLimit))?;
        let episodes = rows
            .iter()
            .filter(|r| matches!(r, w::Row::Episode(_)))
            .count() as u32;
        let clusters = rows.len() as u32 - episodes;
        b.announce(w::Phase::Publish);
        let mut pages = vec![];
        let mut page = w::Page {
            schema: w::PageSchema::V1,
            run_id: start.run_id.clone(),
            manifest_sha256: start.manifest_sha256.clone(),
            index: 0,
            rows: vec![],
            next: Some(1),
        };
        let mut total = 0usize;
        for row in rows {
            let mut row = Some(row);
            let size = b.unit(|| {
                if w::canonical(row.as_ref().unwrap())
                    .map_err(|_| w::Code::ResourceLimit)?
                    .len()
                    > 8192
                {
                    return Err(w::Code::ResourceLimit);
                }
                page.rows.push(row.take().unwrap());
                w::canonical(&page)
                    .map(|v| v.len())
                    .map_err(|_| w::Code::ResourceLimit)
            })?;
            if size > 16 * 1024 || page.rows.len() > 128 {
                let last = page.rows.pop().unwrap();
                publish_page(&output, &page, &start, &mut pages, &mut total, &mut b)?;
                page.index += 1;
                page.next = Some(page.index + 1);
                page.rows = vec![last];
            }
        }
        page.next = None;
        publish_page(&output, &page, &start, &mut pages, &mut total, &mut b)?;
        let result = w::ResultDescriptor {
            schema: w::ResultSchema::V1,
            run_id: start.run_id.clone(),
            nonce: start.nonce.clone(),
            manifest_sha256: start.manifest_sha256.clone(),
            algorithm: w::Algorithm::D0C1,
            source_count: manifest.sources.len() as u32,
            episode_count: episodes,
            cluster_count: clusters,
            max_poll_cpu_ns: w::Number(b.max_unit.max(b.control.reactor.load(Ordering::Acquire))),
            knowledge: w::Knowledge::NotRequested,
            trust: w::Trust::NotEvaluated,
            pages,
        };
        let bytes = b.unit(|| w::canonical(&result).map_err(|_| w::Code::ResourceLimit))?;
        if bytes.len() > w::MAX_FRAME {
            return Err(w::Code::ResourceLimit);
        }
        let hash = b.unit(|| Ok(w::hash(&bytes)))?;
        output.publish("result.json", &bytes, &start.nonce, &mut b)?;
        Ok(Completed {
            hash,
            pages: result.pages.len() as u32,
        })
    })();
    b.announce(w::Phase::Settling);
    // Fixed, bounded diagnostics only; stderr never contains input or IO error text.
    eprintln!(
        "dream-c1 metrics heap_peak_bytes={} rss_peak_bytes={} compute_cpu_ns={} max_unit_cpu_ns={} overruns={} envelopes={} bytes={}",
        b.heap_peak,
        rss(),
        b.counters.cpu,
        b.max_unit,
        b.overruns,
        b.counters.envelopes,
        b.counters.bytes
    );
    eprintln!(
        "dream-c1 normalization_unit_cpu_ns={:?}",
        b.normalization_units
    );
    result.map_err(|code| w::Error {
        code,
        source: source_index,
    })
}
fn publish_page(
    output: &Dir,
    page: &w::Page,
    start: &w::Start,
    pages: &mut Vec<w::PageRef>,
    total: &mut usize,
    b: &mut Budget,
) -> Result<(), w::Code> {
    let bytes = b.unit(|| w::canonical(page).map_err(|_| w::Code::ResourceLimit))?;
    *total += bytes.len();
    if bytes.len() > 16 * 1024 || *total > 16 * 1024 * 1024 {
        return Err(w::Code::ResourceLimit);
    }
    let hash = b.unit(|| Ok(w::hash(&bytes)))?;
    output.publish(
        &format!("page-{:06}.json", page.index),
        &bytes,
        &start.nonce,
        b,
    )?;
    pages.push(w::PageRef {
        index: page.index,
        len: w::Number(bytes.len() as u64),
        sha256: hash,
    });
    Ok(())
}
fn nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn framed(out: &w::WorkerOut) -> Vec<u8> {
    let mut v = w::canonical(out).expect("fixed protocol serialization");
    v.push(b'\n');
    v
}
type Active = (
    w::Start,
    Arc<Control>,
    thread::JoinHandle<Result<Completed, w::Error>>,
    Arc<crossbeam_queue::ArrayQueue<Counters>>,
);
fn run(path: PathBuf, reactor: Arc<AtomicU64>) -> i32 {
    if nonblocking(0).is_err() || nonblocking(1).is_err() {
        return 2;
    }
    let mut pending = VecDeque::from([framed(&w::WorkerOut::Hello(w::Hello {
        protocol: w::Protocol::V1,
        algorithm: w::Algorithm::D0C1,
        schema_sha256: w::schema_hash(),
        ir_major: 1,
        ir_minor_max: 3,
    }))]);
    let mut offset = 0;
    let mut input = Vec::with_capacity(w::MAX_FRAME);
    let mut chunk = [0u8; 8192];
    let mut eof = false;
    let mut state: Option<Active> = None;
    let mut seq = 0u64;
    let mut revision = 0;
    let mut exit = None;
    let mut settled_at = None;
    let mut protocol_failed = false;
    let mut max_control = 0u64;
    let mut last_counters = Counters {
        phase: w::Phase::Validate,
        bytes: 0,
        envelopes: 0,
        cpu: 0,
    };
    loop {
        let mut fds = [
            libc::pollfd {
                fd: 0,
                events: if eof { 0 } else { libc::POLLIN },
                revents: 0,
            },
            libc::pollfd {
                fd: 1,
                events: if pending.is_empty() { 0 } else { libc::POLLOUT },
                revents: 0,
            },
        ];
        unsafe { libc::poll(fds.as_mut_ptr(), 2, 2) };
        let tick = cpu_ns();
        if !eof && (fds[0].revents & (libc::POLLIN | libc::POLLHUP)) != 0 {
            let n = unsafe { libc::read(0, chunk.as_mut_ptr().cast(), chunk.len()) };
            if n == 0 {
                eof = true;
                if let Some((_, c, _, _)) = &state {
                    c.cancel
                        .store(if input.is_empty() { 1 } else { 3 }, Ordering::Release);
                    if !input.is_empty() {
                        protocol_failed = true;
                    }
                } else {
                    return if input.is_empty() { 0 } else { 2 };
                }
            } else if n > 0 {
                for byte in &chunk[..n as usize] {
                    if input.len() == w::MAX_FRAME {
                        protocol_failed = true;
                        break;
                    }
                    input.push(*byte);
                    if *byte == b'\n' {
                        let msg = w::decode::<w::WorkerIn>(&input);
                        input.clear();
                        match (msg, &mut state) {
                            (Ok(w::WorkerIn::Start(s)), None)
                                if exit.is_none()
                                    && (1..=30000).contains(&s.timeout_ms)
                                    && path.file_name().and_then(|n| n.to_str())
                                        == Some(s.run_id.0.as_str()) =>
                            {
                                revision = s.activity_revision.0;
                                let c = Arc::new(Control {
                                    cancel: AtomicU8::new(0),
                                    foreground: AtomicBool::new(s.foreground),
                                    deadline: Instant::now()
                                        + Duration::from_millis(s.timeout_ms as u64),
                                    reactor: reactor.clone(),
                                });
                                let tx = Arc::new(crossbeam_queue::ArrayQueue::new(32));
                                let rx = tx.clone();
                                let cc = c.clone();
                                let ss = s.clone();
                                let p = path.clone();
                                match thread::Builder::new()
                                    .name("dream-c1-compute".into())
                                    .spawn(move || compute(p, ss, cc, tx))
                                {
                                    Ok(h) => state = Some((s, c, h, rx)),
                                    Err(_) => {
                                        protocol_failed = true;
                                    }
                                }
                            }
                            (Ok(w::WorkerIn::Activity(a)), Some((s, c, _, lane)))
                                if a.run_id == s.run_id && a.nonce == s.nonce =>
                            {
                                if a.revision.0 > revision {
                                    revision = a.revision.0;
                                    c.foreground.store(a.foreground, Ordering::Release);
                                    if a.foreground {
                                        let mut paused = last_counters;
                                        paused.phase = w::Phase::Paused;
                                        lane.force_push(paused);
                                    }
                                }
                            }
                            (Ok(w::WorkerIn::Cancel(a)), Some((s, c, _, _)))
                                if a.run_id == s.run_id && a.nonce == s.nonce =>
                            {
                                c.cancel.store(1, Ordering::Release);
                            }
                            _ => protocol_failed = true,
                        }
                        if protocol_failed {
                            break;
                        }
                    }
                }
            } else if io::Error::last_os_error().kind() != io::ErrorKind::WouldBlock {
                protocol_failed = true;
            }
        }
        if protocol_failed {
            eof = true;
            if let Some((_, c, _, _)) = &state {
                c.cancel.store(3, Ordering::Release);
            } else if exit.is_some() {
                exit = Some(2);
            } else {
                return 2;
            }
        }
        if let Some((s, c, h, rx)) = &state {
            if Instant::now() >= c.deadline {
                let _ = c
                    .cancel
                    .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire);
            }
            // Drop counters before sequence assignment. One small completion batch per poll.
            if pending.len() < 32 {
                let mut latest = None;
                for _ in 0..32 {
                    if let Some(c) = rx.pop() {
                        latest = Some(c)
                    } else {
                        break;
                    }
                }
                if let Some(mut counter) = latest {
                    if c.foreground.load(Ordering::Acquire) {
                        counter.phase = w::Phase::Paused;
                    }
                    last_counters = counter;
                    seq += 1;
                    pending.push_back(framed(&w::WorkerOut::Progress(w::Progress {
                        protocol: w::Protocol::V1,
                        run_id: s.run_id.clone(),
                        nonce: s.nonce.clone(),
                        seq: w::Number(seq),
                        phase: counter.phase,
                        bytes: w::Number(counter.bytes),
                        envelopes: w::Number(counter.envelopes),
                        cpu_ns: w::Number(counter.cpu),
                    })));
                }
            }
            if h.is_finished() {
                let (s, c, h, rx) = state.take().unwrap();
                drop(rx);
                let result = h.join().unwrap_or(Err(w::Error {
                    code: w::Code::WorkerLost,
                    source: None,
                }));
                let result = if let Some(code) = c.error() {
                    Err(w::Error { code, source: None })
                } else {
                    result
                };
                let (outcome, hash, pages, error, code) = match result {
                    Ok(r) => (w::Outcome::Completed, Some(r.hash), r.pages, None, 0),
                    Err(e) if e.code == w::Code::Cancelled => {
                        (w::Outcome::Cancelled, None, 0, Some(e), 0)
                    }
                    Err(e) => (
                        w::Outcome::Failed,
                        None,
                        0,
                        Some(e),
                        if protocol_failed { 2 } else { 1 },
                    ),
                };
                seq += 1;
                pending.push_back(framed(&w::WorkerOut::Done(w::Done {
                    protocol: w::Protocol::V1,
                    run_id: s.run_id,
                    nonce: s.nonce,
                    seq: w::Number(seq),
                    outcome,
                    result_sha256: hash,
                    pages,
                    error,
                })));
                exit = Some(code);
                settled_at = Some(Instant::now());
            }
        }
        if let Some(front) = pending.front() {
            let n =
                unsafe { libc::write(1, front[offset..].as_ptr().cast(), front.len() - offset) };
            if n > 0 {
                offset += n as usize;
                if offset == front.len() {
                    pending.pop_front();
                    offset = 0;
                }
            } else if n < 0 && io::Error::last_os_error().kind() != io::ErrorKind::WouldBlock {
                if let Some((_, c, _, _)) = &state {
                    c.cancel.store(1, Ordering::Release);
                    pending.clear();
                    offset = 0;
                } else {
                    return 2;
                }
            }
        }
        let elapsed = cpu_ns().saturating_sub(tick);
        max_control = max_control.max(elapsed);
        if elapsed > UNIT_NS {
            if let Some((_, c, _, _)) = &state {
                c.cancel.store(4, Ordering::Release);
                eof = true;
            } else if exit.is_some() {
                return 1;
            } else {
                return 2;
            }
        }
        if let Some(code) = exit {
            if pending.is_empty() {
                eprintln!("dream-c1 control_max_cpu_ns={max_control}");
                return code;
            }
            if settled_at.is_some_and(|at| at.elapsed() > Duration::from_secs(2)) {
                return 2;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn budget(foreground: bool) -> Budget {
        Budget {
            control: Arc::new(Control {
                cancel: AtomicU8::new(0),
                foreground: AtomicBool::new(foreground),
                deadline: Instant::now() + Duration::from_secs(2),
                reactor: Arc::new(AtomicU64::new(0)),
            }),
            progress: Arc::new(crossbeam_queue::ArrayQueue::new(32)),
            counters: Counters {
                phase: w::Phase::Validate,
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
    #[test]
    fn actual_budget_rejects_cpu_burst_control() {
        let mut b = budget(false);
        let result = b.unit(|| {
            let at = cpu_ns();
            while cpu_ns().saturating_sub(at) < 2_000_000 {
                std::hint::spin_loop();
            }
            Ok(())
        });
        assert_eq!(result, Err(w::Code::ResourceLimit));
        assert_eq!(b.overruns, 1);
    }
    #[test]
    fn units_wait_and_saturated_lane_keeps_pause_with_independent_cancel() {
        let mut b = budget(true);
        for _ in 0..64 {
            b.announce(w::Phase::Validate);
        }
        let c = b.control.clone();
        let lane = b.progress.clone();
        let h = thread::spawn(move || b.unit(|| panic!("foreground unit admitted")));
        let until = Instant::now() + Duration::from_millis(100);
        loop {
            if let Some(p) = lane.pop() {
                if p.phase == w::Phase::Paused {
                    break;
                }
            }
            assert!(
                Instant::now() < until,
                "pause dropped by saturated progress"
            );
            thread::yield_now();
        }
        c.cancel.store(1, Ordering::Release);
        assert_eq!(h.join().unwrap(), Err::<(), _>(w::Code::Cancelled));
        let mut b = budget(false);
        b.unit(|| Ok(())).unwrap();
        let at = Instant::now();
        b.unit(|| Ok(())).unwrap();
        assert!(at.elapsed() >= WAIT);
    }
    #[test]
    fn held_unit_owns_only_its_worker_and_cancel_waits_for_real_settlement() {
        let mut stalled = budget(false);
        let c = stalled.control.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let (entered, ready) = std::sync::mpsc::channel();
        let h = thread::spawn(move || {
            stalled.unit(|| {
                entered.send(()).unwrap();
                rx.recv().unwrap();
                Ok(())
            })
        });
        ready.recv_timeout(Duration::from_secs(1)).unwrap();
        let at = Instant::now();
        for _ in 0..200 {
            c.foreground.store(true, Ordering::Release);
            c.foreground.store(false, Ordering::Release);
            assert!(!h.is_finished());
        }
        assert!(at.elapsed() < Duration::from_millis(100));
        let mut other = budget(false);
        other.unit(|| Ok(())).unwrap();
        assert!(at.elapsed() < Duration::from_secs(2));
        c.cancel.store(1, Ordering::Release);
        assert!(!h.is_finished());
        tx.send(()).unwrap();
        assert_eq!(h.join().unwrap(), Err(w::Code::Cancelled));
    }
}

#[cfg(test)]
mod reactor_control {
    use super::*;
    #[tokio::test]
    async fn inline_hash_parse_cpu_control_is_observed_by_real_poll_guard() {
        let maximum = Arc::new(AtomicU64::new(0));
        let (_, used) = observe_reactor(
            async {
                let at = cpu_ns();
                while cpu_ns().saturating_sub(at) < 2_000_000 {
                    let bytes = b"{\"closed\":true}";
                    let _: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                    std::hint::black_box(w::hash(bytes));
                }
            },
            maximum,
        )
        .await;
        assert!(used > UNIT_NS, "inline CPU mutation escaped reactor guard");
    }
}

#[cfg(test)]
mod custody_control {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn replaced_regular_source_is_detected_by_actual_fd_path_identity() {
        let root = tempfile::Builder::new().tempdir_in("/private/tmp").unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = root.path().join("source");
        std::fs::write(&path, b"original").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let dir = Dir::open(root.path()).unwrap();
        let (f, before) = dir.file("source", 1024).unwrap();
        let replacement = root.path().join("replacement");
        std::fs::write(&replacement, b"original").unwrap();
        std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert_eq!(
            dir.unchanged("source", &f, &before),
            Err(w::Code::SourceChanged)
        );
    }
}
