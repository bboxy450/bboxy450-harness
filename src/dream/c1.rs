// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Private non-seat C1 worker. No shared blocking pool, HOME, provider or discovery.
use super::{bootstrap, c1_wire as w, d0, inspect};
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
// Observational fail-fast guard; the amended hard reservation invariant is
// tracked separately and must not be inferred from this allocator sample.
const HEAP_MAX: u64 = 96 * 1024 * 1024;
const UNIT_NS: u64 = 1_000_000;
#[cfg(test)]
const WAIT: Duration = Duration::from_millis(9);
// Fixed private metrics profile c1-native-debt/1; all CPU/wall/debt values are ns.
// ResultDescriptor captures RAW inclusive max before descriptor serialization;
// terminal_metrics captures through joined owner + final control tick, before
// telemetry formatting/process teardown. Parent wait4 usage covers that tail.
const NATIVE_CPU_NS: u64 = 2_000_000;
const NATIVE_WALL_NS: u64 = 25_000_000;
const SPAN_NS: u64 = UNIT_NS + NATIVE_CPU_NS;
#[derive(Clone, Copy)]
enum NativeKind {
    ComputeOwnerCreate = 0,
    PublishNoreplace = 1,
}
#[derive(Default)]
struct NativeStats {
    count: AtomicU64,
    cpu: AtomicU64,
    wall: AtomicU64,
    max_cpu: AtomicU64,
    max_wall: AtomicU64,
    inclusive_max_cpu: AtomicU64,
    inclusive_max_wall: AtomicU64,
    app_max_cpu: AtomicU64,
    effects: AtomicU64,
    failures_with_effect: AtomicU64,
}
struct Accounting {
    origin: Instant,
    clock_failed: AtomicBool,
    resource_failed: AtomicBool,
    rest_until: AtomicU64,
    debt_total: AtomicU64,
    rest_total: AtomicU64,
    inclusive_max: AtomicU64,
    inclusive_wall_max: AtomicU64,
    app_max: AtomicU64,
    total_cpu: AtomicU64,
    captured_max: AtomicU64,
    owner_cpu: AtomicU64,
    control_cpu: AtomicU64,
    control_parts: [AtomicU64; 4],
    native: [NativeStats; 2],
}
impl Accounting {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
            clock_failed: AtomicBool::new(false),
            resource_failed: AtomicBool::new(false),
            rest_until: AtomicU64::new(0),
            debt_total: AtomicU64::new(0),
            rest_total: AtomicU64::new(0),
            inclusive_max: AtomicU64::new(0),
            inclusive_wall_max: AtomicU64::new(0),
            app_max: AtomicU64::new(0),
            total_cpu: AtomicU64::new(0),
            captured_max: AtomicU64::new(0),
            owner_cpu: AtomicU64::new(0),
            control_cpu: AtomicU64::new(0),
            control_parts: std::array::from_fn(|_| AtomicU64::new(0)),
            native: std::array::from_fn(|_| NativeStats::default()),
        }
    }
    fn now(&self) -> u64 {
        self.origin.elapsed().as_nanos().min(u64::MAX as u128) as u64
    }
    fn pay(&self) {
        let at = self.now();
        loop {
            let now = self.now();
            let until = self.rest_until.load(Ordering::Acquire);
            if now >= until {
                break;
            }
            thread::sleep(Duration::from_nanos(until - now));
        }
        self.rest_total.fetch_add(self.now() - at, Ordering::AcqRel);
    }
    fn charge(&self, cpu: u64, app: u64) {
        // Debt starts at the end, never at admission or before blocking IO.
        let debt = if cpu == u64::MAX {
            9 * SPAN_NS
        } else {
            cpu.max(UNIT_NS).saturating_mul(9)
        };
        self.rest_until
            .fetch_max(self.now().saturating_add(debt), Ordering::AcqRel);
        self.debt_total.fetch_add(debt, Ordering::AcqRel);
        if cpu == u64::MAX {
            // Unknown cost is a sticky failed measurement, never a zero sample
            // or wrapping total. Only bounded mandatory settlement remains.
            self.clock_failed.store(true, Ordering::Release);
        } else {
            self.total_cpu.fetch_add(cpu, Ordering::AcqRel);
        }
        self.inclusive_max.fetch_max(cpu, Ordering::AcqRel);
        self.app_max.fetch_max(app, Ordering::AcqRel);
    }
}
fn interval(a: u64, b: u64) -> u64 {
    if a == u64::MAX || b == u64::MAX || b < a {
        u64::MAX
    } else {
        b - a
    }
}
/// The returned owner/effect NEVER lives behind the verdict Result. Even a bad
/// ending clock returns the real handle/status to the caller before refusal.
struct NativeOutcome<T> {
    value: T,
    verdict: Result<(), w::Code>,
    cpu: u64,
}
fn errno_pointer() -> *mut libc::c_int {
    #[cfg(target_os = "macos")]
    {
        unsafe { libc::__error() }
    }
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::__errno_location() }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        compile_error!("C1 native timing needs a thread-local errno adapter");
    }
}
fn native_call<T>(
    a: &Accounting,
    kind: NativeKind,
    f: impl FnOnce() -> T,
    effect: impl FnOnce(&T) -> bool,
) -> Result<NativeOutcome<T>, w::Code> {
    native_call_clock(a, kind, f, effect, cpu_ns, Instant::now)
}
fn native_call_clock<T>(
    a: &Accounting,
    kind: NativeKind,
    f: impl FnOnce() -> T,
    effect: impl FnOnce(&T) -> bool,
    mut cpu: impl FnMut() -> u64,
    mut wall: impl FnMut() -> Instant,
) -> Result<NativeOutcome<T>, w::Code> {
    a.pay();
    if a.clock_failed.load(Ordering::Acquire) || a.resource_failed.load(Ordering::Acquire) {
        return Err(w::Code::ResourceLimit);
    }
    let limit = match kind {
        NativeKind::ComputeOwnerCreate => 1,
        NativeKind::PublishNoreplace => 4097,
    };
    if a.native[kind as usize].count.load(Ordering::Acquire) >= limit {
        return Err(w::Code::ResourceLimit);
    }
    let span_wall_start = Instant::now();
    let span_start = cpu();
    let call_start = cpu();
    let wall_start = wall();
    if span_start == u64::MAX || call_start == u64::MAX {
        a.clock_failed.store(true, Ordering::Release);
        return Err(w::Code::ResourceLimit);
    }
    let value = f();
    // Mandatory scalar return capture precedes every clock/cancel verdict.
    // Probe failure must not replace the syscall's original unknown-effect errno.
    let returned_errno = unsafe { *errno_pointer() };
    let wall_end = wall();
    let call_end = cpu();
    unsafe {
        *errno_pointer() = returned_errno;
    }
    let has_effect = effect(&value);
    let span_end = cpu();
    let span_wall = span_wall_start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    let native_cpu = interval(call_start, call_end);
    let native_wall = match wall_end.checked_duration_since(wall_start) {
        Some(d) => d.as_nanos().min(u64::MAX as u128) as u64,
        None => {
            a.clock_failed.store(true, Ordering::Release);
            u64::MAX
        }
    };
    let inclusive = interval(span_start, span_end);
    // Separate measured application subintervals; inclusive CPU stays RAW.
    let app = interval(span_start, call_start).saturating_add(interval(call_end, span_end));
    let stats = &a.native[kind as usize];
    stats.count.fetch_add(1, Ordering::AcqRel);
    if native_cpu != u64::MAX {
        stats.cpu.fetch_add(native_cpu, Ordering::AcqRel);
    }
    if native_wall != u64::MAX {
        stats.wall.fetch_add(native_wall, Ordering::AcqRel);
    }
    stats.max_cpu.fetch_max(native_cpu, Ordering::AcqRel);
    stats.max_wall.fetch_max(native_wall, Ordering::AcqRel);
    stats
        .inclusive_max_cpu
        .fetch_max(inclusive, Ordering::AcqRel);
    stats
        .inclusive_max_wall
        .fetch_max(span_wall, Ordering::AcqRel);
    stats.app_max_cpu.fetch_max(app, Ordering::AcqRel);
    a.inclusive_wall_max.fetch_max(span_wall, Ordering::AcqRel);
    if has_effect {
        stats.effects.fetch_add(1, Ordering::AcqRel);
    }
    a.charge(inclusive, app);
    let verdict = if stats.count.load(Ordering::Acquire) > limit
        || native_cpu > NATIVE_CPU_NS
        || native_wall > NATIVE_WALL_NS
        || app > UNIT_NS
        || inclusive > SPAN_NS
    {
        if has_effect {
            stats.failures_with_effect.fetch_add(1, Ordering::AcqRel);
        }
        a.resource_failed.store(true, Ordering::Release);
        Err(w::Code::ResourceLimit)
    } else {
        Ok(())
    };
    Ok(NativeOutcome {
        value,
        verdict,
        cpu: inclusive,
    })
}
struct ControlTick<'a> {
    at: u64,
    part_at: u64,
    accounting: &'a Accounting,
    max: &'a AtomicU64,
}
impl ControlTick<'_> {
    fn part(&mut self, index: usize) {
        let end = cpu_ns();
        self.accounting.control_parts[index]
            .fetch_max(interval(self.part_at, end), Ordering::AcqRel);
        self.part_at = end;
    }
}
impl Drop for ControlTick<'_> {
    fn drop(&mut self) {
        let used = interval(self.at, cpu_ns());
        self.max.fetch_max(used, Ordering::AcqRel);
        self.accounting
            .inclusive_max
            .fetch_max(used, Ordering::AcqRel);
        if used == u64::MAX {
            self.accounting.clock_failed.store(true, Ordering::Release);
        } else {
            self.accounting
                .control_cpu
                .fetch_add(used, Ordering::AcqRel);
        }
    }
}
fn owner_result<T>(a: &Accounting, at: u64, value: T, clock: impl FnOnce() -> u64) -> T {
    let elapsed = interval(at, clock());
    a.owner_cpu.store(elapsed, Ordering::Release);
    if elapsed == u64::MAX {
        a.clock_failed.store(true, Ordering::Release);
    }
    value
}
fn process_usage(
    a: &Accounting,
    control: &AtomicU64,
    probe: impl FnOnce() -> Option<(u64, u64)>,
) -> (u64, u64) {
    match probe() {
        Some((user, system)) if user != u64::MAX && system != u64::MAX => (user, system),
        _ => {
            a.clock_failed.store(true, Ordering::Release);
            // The final telemetry sample follows Done; the outer early entry
            // checks this sticky verdict and exits nonzero, so it cannot certify
            // Completed even if the earlier frame was already delivered.
            control.fetch_max(u64::MAX, Ordering::AcqRel);
            (u64::MAX, u64::MAX)
        }
    }
}
struct TerminalMetrics {
    accounting: Arc<Accounting>,
    control: Arc<AtomicU64>,
}
impl Drop for TerminalMetrics {
    fn drop(&mut self) {
        let a = &self.accounting;
        a.pay();
        let load = |v: &AtomicU64| v.load(Ordering::Acquire);
        for (tag, s) in ["compute_owner_create", "publish_noreplace"]
            .into_iter()
            .zip(&a.native)
        {
            eprintln!(
                "dream-c1 native tag={tag} count={} cpu_ns={} wall_ns={} max_cpu_ns={} max_wall_ns={} inclusive_max_cpu_ns={} inclusive_max_wall_ns={} app_max_cpu_ns={} effects={} failures_with_effect={}",
                load(&s.count),
                load(&s.cpu),
                load(&s.wall),
                load(&s.max_cpu),
                load(&s.max_wall),
                load(&s.inclusive_max_cpu),
                load(&s.inclusive_max_wall),
                load(&s.app_max_cpu),
                load(&s.effects),
                load(&s.failures_with_effect)
            );
        }
        let (user, system) = process_usage(a, &self.control, || {
            let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
            if unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) } != 0 {
                return None;
            }
            let r = unsafe { r.assume_init() };
            let ns = |t: libc::timeval| {
                let sec = u64::try_from(t.tv_sec).ok()?;
                let usec = u64::try_from(t.tv_usec).ok().filter(|v| *v < 1_000_000)?;
                sec.checked_mul(1_000_000_000)?.checked_add(usec * 1000)
            };
            Some((ns(r.ru_utime)?, ns(r.ru_stime)?))
        });
        eprintln!(
            "dream-c1 cost_known={}",
            !a.clock_failed.load(Ordering::Acquire)
        );
        let overall = load(&a.inclusive_max).max(load(&self.control));
        eprintln!(
            "dream-c1 control_parts input_max_cpu_ns={} protocol_max_cpu_ns={} settlement_max_cpu_ns={} output_max_cpu_ns={} total_cpu_ns={}",
            load(&a.control_parts[0]),
            load(&a.control_parts[1]),
            load(&a.control_parts[2]),
            load(&a.control_parts[3]),
            load(&a.control_cpu)
        );
        eprintln!(
            "dream-c1 terminal_metrics profile=c1-native-debt/1 cutoff=joined_final_control_pre_telemetry captured_max_cpu_ns={} inclusive_max_cpu_ns={overall} app_max_cpu_ns={} control_max_cpu_ns={} inclusive_max_wall_ns={} span_cpu_ns={} owner_thread_cpu_ns={} process_user_cpu_ns={user} process_system_cpu_ns={system} wall_ns={} debt_total_ns={} rest_observed_ns={} remaining_debt_ns={}",
            load(&a.captured_max),
            load(&a.app_max),
            load(&self.control),
            load(&a.inclusive_wall_max),
            load(&a.total_cpu),
            load(&a.owner_cpu),
            a.now(),
            load(&a.debt_total),
            load(&a.rest_total),
            load(&a.rest_until).saturating_sub(a.now())
        );
    }
}
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

/// Strict executable-only dispatch before the shared Tokio runtime exists.
/// Darwin exposes borrowed argv/env pointers, so recognition allocates no argv Vec.
pub fn early_worker_entry() -> Option<i32> {
    // Samples precede borrowed recognition; failed samples only refuse the
    // recognized managed worker, leaving ordinary launch selection unchanged.
    let prelude_sample = bootstrap::prelude_clock();
    let entry_sample = bootstrap::monotonic_ns();
    let Some((argc, argv)) = bootstrap::arguments() else {
        return if unsafe { bootstrap::environment(bootstrap::ENV.as_bytes()) }
            .ok()
            .flatten()
            .is_some()
        {
            Some(2)
        } else {
            None
        };
    };
    if argc < 3 {
        return None;
    }
    let arg = |i, max| unsafe { bootstrap::borrowed(*argv.add(i), max) };
    if arg(1, 5).ok() != Some(b"dream".as_slice())
        || arg(2, 10).ok() != Some(b"inspect-c1".as_slice())
    {
        return None;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (prelude_sample, entry_sample);
        return Some(2);
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStringExt;
        let prelude_at = match prelude_sample {
            Ok(n) => n,
            Err(_) => return Some(1),
        };
        let entry_ns = match entry_sample {
            Ok(n) => n,
            Err(_) => return Some(1),
        };
        if argc != 5 || arg(3, 9).ok() != Some(b"--job-dir".as_slice()) {
            return Some(2);
        }
        let raw = match arg(4, 4096) {
            Ok(raw) if raw.first() == Some(&b'/') => raw,
            _ => return Some(2),
        };
        let basename = raw.rsplit(|c| *c == b'/').next().unwrap();
        let t = match unsafe { bootstrap::environment(bootstrap::ENV.as_bytes()) }
            .ok()
            .flatten()
            .and_then(|b| bootstrap::token(b).ok())
        {
            Some(t) => t,
            None => return Some(2),
        };
        let mut gate = match unsafe {
            bootstrap::Gate::wait(
                t,
                bootstrap::Purpose::InspectC1,
                basename,
                entry_ns,
                prelude_at,
            )
        } {
            Ok(gate) => gate,
            Err(_) => return Some(2),
        };
        if gate.admit(0).is_err() {
            return Some(2);
        }
        let argv_at = cpu_ns();
        let mut owned = Vec::new();
        if owned.try_reserve_exact(raw.len()).is_err() {
            return Some(1);
        }
        owned.extend_from_slice(raw);
        let path = PathBuf::from(std::ffi::OsString::from_vec(owned));
        let maximum = Arc::new(AtomicU64::new(0));
        let argv_cpu = interval(argv_at, cpu_ns());
        maximum.fetch_max(argv_cpu, Ordering::AcqRel);
        if argv_cpu > UNIT_NS {
            return Some(1);
        }
        thread::sleep(Duration::from_nanos(9 * argv_cpu.max(UNIT_NS)));
        if gate.check().is_err() {
            return Some(2);
        }
        let code = run(path, maximum.clone(), gate);
        let code = if maximum.load(Ordering::Acquire) > UNIT_NS {
            1
        } else {
            code
        };
        eprintln!(
            "dream-c1 reactor_max_cpu_ns={} startup_runtime=none",
            maximum.load(Ordering::Acquire)
        );
        Some(code)
    }
}
/// A late async entry cannot satisfy the pre-runtime gate. No ungated fallback.
pub async fn cli(_args: impl Iterator<Item = String>) -> i32 {
    2
}
#[cfg(test)]
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
fn cpu_ns() -> u64 {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, t.as_mut_ptr()) } != 0 {
        return u64::MAX;
    }
    let t = unsafe { t.assume_init() };
    t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
}
#[cfg(target_os = "macos")]
fn heaps() -> (u64, u64) {
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
    (stats.size as u64, stats.peak.max(stats.size) as u64)
}
#[cfg(not(target_os = "macos"))]
fn heaps() -> (u64, u64) {
    // No unmeasured heap certification on a platform without this allocator probe.
    (u64::MAX, u64::MAX)
}
fn heap() -> u64 {
    heaps().1
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
#[derive(Clone, Copy)]
struct PublicationResult {
    status: i32,
    errno: i32,
    operation: u32,
    dev: u64,
    ino: u64,
    nonce: [u8; 26],
}
#[derive(Default)]
struct PublicationCustody {
    operation: AtomicU64,
    dev: AtomicU64,
    ino: AtomicU64,
    state: AtomicU8,
    errno: AtomicU64,
}
impl PublicationCustody {
    fn retain(&self, r: &PublicationResult) {
        self.operation.store(r.operation as u64, Ordering::Release);
        self.dev.store(r.dev, Ordering::Release);
        self.ino.store(r.ino, Ordering::Release);
        self.errno.store(r.errno as u64, Ordering::Release);
        self.state.store(
            if r.status == 0 {
                1
            } else if r.errno == libc::EINTR || r.errno == libc::EIO {
                2
            } else {
                0
            },
            Ordering::Release,
        );
        // nonce is owned in the typed result before clocks and in Start for the
        // whole run; the deterministic temp/path mapping is unchanged.
        debug_assert!(r.nonce.iter().all(u8::is_ascii));
    }
}
struct Control {
    cancel: AtomicU8,
    foreground: AtomicBool,
    deadline: Instant,
    reactor: Arc<AtomicU64>,
    memory: Arc<storage::Ledger>,
    accounting: Arc<Accounting>,
    publication: PublicationCustody,
    cleanup_uncertain: AtomicBool,
    cleanup_cpu: AtomicU64,
    cleanup_max: AtomicU64,
    cleanup_overruns: AtomicU64,
}
impl Control {
    fn error(&self) -> Option<w::Code> {
        if self.accounting.clock_failed.load(Ordering::Acquire)
            || self.accounting.resource_failed.load(Ordering::Acquire)
            || self.reactor.load(Ordering::Acquire) > UNIT_NS
            || self.cleanup_max.load(Ordering::Acquire) > UNIT_NS
        {
            return Some(w::Code::ResourceLimit);
        }
        if self.cleanup_uncertain.load(Ordering::Acquire) {
            return Some(w::Code::StorageUnavailable);
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
    fn native<T>(
        &mut self,
        kind: NativeKind,
        f: impl FnOnce() -> T,
        effect: impl FnOnce(&T) -> bool,
    ) -> Result<NativeOutcome<T>, w::Code> {
        self.control.accounting.pay();
        loop {
            if let Some(e) = self.control.error() {
                return Err(e);
            }
            if !self.control.foreground.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        if heap() > HEAP_MAX {
            return Err(w::Code::ResourceLimit);
        }
        let out = native_call(&self.control.accounting, kind, f, effect)?;
        self.max_unit = self.max_unit.max(out.cpu);
        self.counters.cpu = self.counters.cpu.saturating_add(out.cpu);
        if out.verdict.is_err() {
            self.overruns += 1;
            self.control.cancel.store(4, Ordering::Release);
        }
        // No cancellation/timing decision may discard out.value.
        Ok(out)
    }
    fn track<T: disposal::Drain>(
        &mut self,
        bytes: u64,
        f: impl FnOnce() -> Result<T, w::Code>,
    ) -> Result<storage::Tracked<T>, w::Code> {
        self.track_clock(bytes, f, cpu_ns)
    }
    fn track_clock<T: disposal::Drain>(
        &mut self,
        bytes: u64,
        f: impl FnOnce() -> Result<T, w::Code>,
        clock: impl FnMut() -> u64,
    ) -> Result<storage::Tracked<T>, w::Code> {
        let credit = self.control.memory.reserve(bytes)?;
        let value = self.unit_clock(f, clock)?;
        Ok(storage::Tracked { value, credit })
    }
    #[track_caller]
    fn unit<T: disposal::Drain>(
        &mut self,
        f: impl FnOnce() -> Result<T, w::Code>,
    ) -> Result<T, w::Code> {
        self.unit_clock(f, cpu_ns)
    }
    #[track_caller]
    fn unit_clock<T: disposal::Drain>(
        &mut self,
        f: impl FnOnce() -> Result<T, w::Code>,
        mut clock: impl FnMut() -> u64,
    ) -> Result<T, w::Code> {
        self.control.accounting.pay();
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
        if live > HEAP_MAX {
            return Err(w::Code::ResourceLimit);
        }
        let wall_at = Instant::now();
        let start = clock();
        if start == u64::MAX {
            self.control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
            return Err(w::Code::ResourceLimit);
        }
        // The slot outlives every admission verdict, including a failed clock.
        // Its stepped destructor finishes before track's credit can be released.
        let mut result = Paced::new(None, &self.control, disposal::Drain::drain_step);
        *result = Some(f());
        let end = clock();
        let elapsed = interval(start, end);
        self.control.accounting.inclusive_wall_max.fetch_max(
            wall_at.elapsed().as_nanos().min(u64::MAX as u128) as u64,
            Ordering::AcqRel,
        );
        self.control.accounting.charge(elapsed, elapsed);
        if elapsed == u64::MAX {
            return Err(w::Code::ResourceLimit);
        }
        self.max_unit = self.max_unit.max(elapsed);
        self.counters.cpu = self.counters.cpu.saturating_add(elapsed);
        self.heap_peak = self.heap_peak.max(heap());
        if elapsed > UNIT_NS || self.heap_peak > HEAP_MAX {
            self.control
                .accounting
                .resource_failed
                .store(true, Ordering::Release);
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
        result.take_value().unwrap()
    }
}
/// Settlement is mandatory after cancellation. These small cleanup units retain
/// the same CPU/duty rule, independently of foreground/expensive admission flags.
fn cleanup_unit(control: &Control, f: impl FnOnce()) {
    cleanup_unit_clock(control, f, cpu_ns);
}
fn cleanup_unit_clock(control: &Control, f: impl FnOnce(), mut clock: impl FnMut() -> u64) {
    control.accounting.pay();
    let wall_at = Instant::now();
    let at = clock();
    f();
    let end = clock();
    // Missing/reversed cleanup clocks retain unknown cost and fail settlement;
    // mandatory bounded disposal still finishes before credit release.
    let elapsed = interval(at, end);
    control.accounting.inclusive_wall_max.fetch_max(
        wall_at.elapsed().as_nanos().min(u64::MAX as u128) as u64,
        Ordering::AcqRel,
    );
    control.accounting.charge(elapsed, elapsed);
    if elapsed != u64::MAX {
        control.cleanup_cpu.fetch_add(elapsed, Ordering::AcqRel);
    }
    control.cleanup_max.fetch_max(elapsed, Ordering::AcqRel);
    if elapsed > UNIT_NS {
        control
            .accounting
            .resource_failed
            .store(true, Ordering::Release);
        control.cleanup_overruns.fetch_add(1, Ordering::AcqRel);
    }
}
struct Paced<T> {
    value: Option<T>,
    step: fn(&mut T) -> bool,
    control: Arc<Control>,
}
impl<T> Paced<T> {
    fn new(value: T, control: &Arc<Control>, step: fn(&mut T) -> bool) -> Self {
        Self {
            value: Some(value),
            step,
            control: control.clone(),
        }
    }
    fn take_value(&mut self) -> T {
        self.value.take().unwrap()
    }
}
impl<T> std::ops::Deref for Paced<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}
impl<T> std::ops::DerefMut for Paced<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}
impl<T> Drop for Paced<T> {
    fn drop(&mut self) {
        while self.value.is_some() {
            cleanup_unit(&self.control, || {
                let start = cpu_ns();
                // The finite entry bound also applies if the CPU clock fails.
                for _ in 0..1024 {
                    if !(self.step)(self.value.as_mut().unwrap()) {
                        drop(self.value.take());
                        break;
                    }
                    if cpu_ns().saturating_sub(start) >= 400_000 {
                        break;
                    }
                }
            });
        }
    }
}
impl Budget {
    fn cleanup(&mut self, f: impl FnOnce() -> Result<(), w::Code>) -> Result<(), w::Code> {
        let mut result = Ok(());
        cleanup_unit(&self.control, || {
            result = f();
        });
        if self.control.cleanup_max.load(Ordering::Acquire) > UNIT_NS {
            return Err(w::Code::ResourceLimit);
        }
        result
    }
}
/// Incremental ownership teardown for admitted, depth-limited IR. One step
/// removes at most one collection entry/scalar payload; recursion follows only
/// the preflight-bounded (32-deep) JSON path. No serialization, clone or heap
/// allocation is needed to dispose of a rejected typed result.
mod disposal {
    use super::*;
    use crate::protocol::*;
    pub(super) trait Drain {
        /// true: a bounded piece was removed; false: only fixed leaf fields remain.
        fn drain_step(&mut self) -> bool;
    }
    macro_rules! leaves {
        ($($t:ty),* $(,)?) => {$(impl Drain for $t {
            fn drain_step(&mut self) -> bool { false }
        })*};
    }
    leaves!(
        (),
        bool,
        u8,
        usize,
        u64,
        w::Hex,
        Dir,
        File,
        (File, Metadata),
        (CString, CString),
        storage::Shape,
        String,
        PathBuf,
        Location,
        Citation
    );
    impl<T: Drain> Drain for Option<T> {
        fn drain_step(&mut self) -> bool {
            let Some(v) = self.as_mut() else { return false };
            if !v.drain_step() {
                self.take();
            }
            true
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
    impl<T: Drain> Drain for storage::Tracked<T> {
        fn drain_step(&mut self) -> bool {
            self.value.drain_step()
        }
    }
    impl<T: Drain> Drain for Box<T> {
        fn drain_step(&mut self) -> bool {
            (**self).drain_step()
        }
    }
    impl<T: Drain> Drain for Vec<T> {
        fn drain_step(&mut self) -> bool {
            let Some(v) = self.last_mut() else {
                return false;
            };
            if !v.drain_step() {
                self.pop();
            }
            true
        }
    }
    impl<T: Drain> Drain for BTreeMap<String, T> {
        fn drain_step(&mut self) -> bool {
            let Some(mut entry) = self.first_entry() else {
                return false;
            };
            if !entry.get_mut().drain_step() {
                entry.remove_entry();
            }
            true
        }
    }
    impl Drain for serde_json::Value {
        fn drain_step(&mut self) -> bool {
            use serde_json::Value;
            match self {
                Value::Null => false,
                Value::Array(v) => v.drain_step(),
                Value::Object(v) => {
                    let Some((key, child)) = v.iter_mut().next() else {
                        return false;
                    };
                    if child.drain_step() {
                        return true;
                    }
                    // serde_json::Map has no pop-entry API. Borrow a bounded
                    // stack copy of the key, then remove the now-empty leaf.
                    // This frame is entered only for removal, not recursively.
                    #[inline(never)]
                    fn remove<const N: usize>(v: &mut serde_json::Map<String, Value>) {
                        let mut key_bytes = [0u8; N];
                        let key = v.keys().next().unwrap();
                        let len = key.len();
                        key_bytes[..len].copy_from_slice(key.as_bytes());
                        let key = std::str::from_utf8(&key_bytes[..len]).unwrap();
                        v.remove_entry(key);
                    }
                    debug_assert!(key.len() <= w::MAX_FRAME);
                    if key.len() <= 512 {
                        remove::<512>(v);
                    } else {
                        remove::<{ w::MAX_FRAME }>(v);
                    }
                    true
                }
                _ => {
                    *self = Value::Null;
                    true
                }
            }
        }
    }
    macro_rules! fields {
        ($t:ty; $($field:ident),+ $(,)?) => { impl Drain for $t {
            fn drain_step(&mut self) -> bool { false $(|| self.$field.drain_step())+ }
        }};
    }
    // Simple tagged enums have at most fixed leaf ownership except Unknown.
    macro_rules! unknown {
        ($($t:ty),+ $(,)?) => {$(impl Drain for $t {
            fn drain_step(&mut self) -> bool {
                match self { Self::Unknown(v) => v.drain_step(), _ => false }
            }
        })+};
    }
    unknown!(
        ItemStatus,
        TurnOrigin,
        ItemKind,
        ToolKind,
        ItemHead,
        Stream,
        ChangeKind,
        PlanStatus,
        Priority,
        Effect,
        ApprovalDecisionKind,
        ApprovalDecision,
        Resolver,
        Level,
        SystemRole,
        Rejection,
        ErrorClass,
        Phase,
        DreamPhase,
        ScopeLevel,
        MemoryOp,
        EntryState,
        Arm,
        Verdict,
        SinkKind,
        RecallBasis,
        ReceiptKind,
        ReceiptOutcome,
        EvidenceRef,
        Trigger,
        UsageScope,
        BackendKind,
        NumberSource
    );
    // ItemHead::ToolCall embeds ToolKind, beyond the simple Unknown case.
    // Handle that field via the containing Event below.
    fields!(Envelope; ext, event);
    fields!(SessionMeta; dream);
    fields!(DreamRun; phase, trigger, scope);
    fields!(Scope; level, repos);
    fields!(Usage; scope, backend, cost, source, unreported);
    fields!(Backend; kind);
    fields!(Cost; source);
    fields!(PlanEntry; status, priority);
    fields!(ApprovalSubject; effect, paths);
    fields!(TrialEpisode; origin, runs);
    fields!(TrialRun; arm, usage);
    fields!(SinkRef; kind);
    impl Drain for StopReason {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) => v.drain_step(),
                Self::Error { class, .. } => class.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for Event {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) => v.drain_step(),
                Self::SessionStarted { meta } => meta.drain_step(),
                Self::TurnStarted { origin, .. } => origin.drain_step(),
                Self::ContextChanged { config, .. } => config.drain_step(),
                Self::ItemStarted { kind, head, .. } => {
                    kind.drain_step()
                        || match head {
                            Some(ItemHead::ToolCall { tool_kind, .. }) => tool_kind.drain_step(),
                            _ => head.drain_step(),
                        }
                }
                Self::ItemCompleted { status, body, .. } => {
                    status.drain_step() || body.drain_step()
                }
                Self::TurnEnded { reason, usage, .. } => reason.drain_step() || usage.drain_step(),
                Self::OpRejected { reason, .. } => reason.drain_step(),
                Self::ItemDelta { delta, .. } => delta.drain_step(),
                Self::ItemSnapshot { partial, .. } => partial.drain_step(),
                Self::Status { phase } => phase.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for Delta {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) => v.drain_step(),
                Self::Plan { entries } => entries.drain_step(),
                Self::Output { stream, .. } => stream.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for Item {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) => v.drain_step(),
                Self::UserMessage { blocks } => blocks.drain_step(),
                Self::Reasoning {
                    provider_opaque, ..
                } => provider_opaque.drain_step(),
                Self::ToolCall {
                    tool_kind,
                    output,
                    locations,
                    raw_output,
                    ..
                } => {
                    tool_kind.drain_step()
                        || output.drain_step()
                        || locations.drain_step()
                        || raw_output.drain_step()
                }
                Self::FileChange { change, .. } => change.drain_step(),
                Self::Plan { entries } => entries.drain_step(),
                Self::Approval {
                    subject,
                    options,
                    decision,
                    by,
                } => {
                    subject.drain_step()
                        || options.drain_step()
                        || decision.drain_step()
                        || by.drain_step()
                }
                Self::Usage { usage } => usage.drain_step(),
                Self::Notice { level, .. } => level.drain_step(),
                Self::SystemMessage { role, .. } => role.drain_step(),
                Self::Native { raw, .. } => raw.drain_step(),
                Self::MemoryWrite {
                    op,
                    evidence,
                    draft,
                    ..
                } => op.drain_step() || evidence.drain_step() || draft.drain_step(),
                Self::MemoryState { state, refs, .. } => state.drain_step() || refs.drain_step(),
                Self::Trial {
                    episodes,
                    controls,
                    verdict,
                    ..
                } => episodes.drain_step() || controls.drain_step() || verdict.drain_step(),
                Self::Adoption { sink, .. } => sink.drain_step(),
                Self::Recall { sink, basis, .. } => sink.drain_step() || basis.drain_step(),
                Self::Receipt {
                    kind,
                    outcome,
                    sink,
                    ..
                } => kind.drain_step() || outcome.drain_step() || sink.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for InputBlock {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) => v.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for OutputBlock {
        fn drain_step(&mut self) -> bool {
            match self {
                Self::Unknown(v) | Self::Json { value: v } => v.drain_step(),
                Self::Diff { change, .. } => change.drain_step(),
                Self::Citations { entries } => entries.drain_step(),
                _ => false,
            }
        }
    }
    impl Drain for w::Manifest {
        fn drain_step(&mut self) -> bool {
            if let Some(source) = self.sources.last_mut() {
                if source.blobs.pop().is_none() {
                    self.sources.pop();
                }
                return true;
            }
            false
        }
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
                Ok(())
            })?;
            let (a, dest) = b.unit(|| {
                Ok((
                    CString::new(temp.as_str()).unwrap(),
                    CString::new(name).unwrap(),
                ))
            })?;
            let operation = if name == "result.json" {
                4096
            } else {
                name.strip_prefix("page-")
                    .and_then(|n| n.strip_suffix(".json"))
                    .and_then(|n| n.parse::<u32>().ok())
                    .filter(|n| *n < 4096)
                    .ok_or(w::Code::StorageUnavailable)?
            };
            let original_nonce: [u8; 26] = nonce
                .0
                .as_bytes()
                .try_into()
                .map_err(|_| w::Code::ProtocolError)?;
            let control = b.control.clone();
            let effect = b.native(
                NativeKind::PublishNoreplace,
                || {
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
                    status
                },
                |status| {
                    let errno = if *status == 0 {
                        0
                    } else {
                        io::Error::last_os_error()
                            .raw_os_error()
                            .unwrap_or(libc::EIO)
                    };
                    let result = PublicationResult {
                        status: *status,
                        errno,
                        operation,
                        dev: before.dev(),
                        ino: before.ino(),
                        nonce: original_nonce,
                    };
                    control.publication.retain(&result);
                    *status == 0
                },
            )?;
            // The exclusive rename effect is captured with original nonce and
            // staged inode (before), even if a later clock/cancel verdict fails.
            effect.verdict?;
            if effect.value != 0 {
                return Err(w::Code::StorageUnavailable);
            }
            if let Some(e) = b.control.error() {
                return Err(e);
            }
            b.unit(|| self.0.sync_all().map_err(|_| w::Code::StorageUnavailable))
        })();
        // Only this exclusive-created inode can be unlinked, even on cancellation.
        let control = b.control.clone();
        let cleanup = b.cleanup(|| {
            // Unknown publication effect retains both names/inode for parent
            // settlement; do not delete evidence or repeat the operation.
            if control.publication.state.load(Ordering::Acquire) == 2 {
                return Ok(());
            }
            let a = CString::new(temp.as_str()).unwrap();
            let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
            let status = unsafe {
                libc::fstatat(
                    self.0.as_raw_fd(),
                    a.as_ptr(),
                    st.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if status != 0 {
                if io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                    return Ok(());
                }
                control.cleanup_uncertain.store(true, Ordering::Release);
                return Err(w::Code::StorageUnavailable);
            }
            let st = unsafe { st.assume_init() };
            if st.st_dev as u64 != before.dev()
                || st.st_ino != before.ino()
                || unsafe { libc::unlinkat(self.0.as_raw_fd(), a.as_ptr(), 0) } != 0
            {
                control.cleanup_uncertain.store(true, Ordering::Release);
                return Err(w::Code::StorageUnavailable);
            }
            Ok(())
        });
        match result {
            Err(e) => Err(e),
            Ok(()) => cleanup,
        }
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
fn read_bounded(
    dir: &Dir,
    name: &str,
    max: u64,
    b: &mut Budget,
) -> Result<storage::Tracked<Vec<u8>>, w::Code> {
    let (mut file, before) = b.unit(|| dir.file(name, max))?;
    let mut bytes = b.track(before.len(), || {
        let mut v = Vec::new();
        v.try_reserve_exact(before.len() as usize)
            .map_err(|_| w::Code::ResourceLimit)?;
        Ok(v)
    })?;
    let mut chunk = [0u8; CHUNK];
    loop {
        let n = b.unit(|| {
            file.read(&mut chunk)
                .map_err(|_| w::Code::SourceUnavailable)
        })?;
        if n == 0 {
            break;
        }
        if bytes.len() + n > before.len() as usize {
            return Err(w::Code::SourceChanged);
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
fn selected(
    input: &Dir,
    source: &w::Source,
    b: &mut Budget,
) -> Result<storage::Tracked<d0::ClosedLog>, w::Code> {
    let name = format!("{}.jsonl", source.session.0);
    let (mut file, before) = b.unit(|| input.file(&name, INPUT_MAX))?;
    if before.len() != source.len.0 {
        return Err(w::Code::SourceChanged);
    }
    let mut hasher = Sha256::new();
    let _line_hash_credit = b.control.memory.reserve(512)?;
    let mut line_hasher = Sha256::new();
    let mut line_digest = w::hash([]);
    let mut chunk = [0u8; CHUNK];
    let line = b.track(w::MAX_FRAME as u64, || {
        let mut v = Vec::new();
        v.try_reserve_exact(w::MAX_FRAME)
            .map_err(|_| w::Code::ResourceLimit)?;
        Ok(v)
    })?;
    let mut line = Paced::new(line, &b.control, |v| {
        v.clear();
        false
    });
    // Persistent upper-credit transfers cover retained declared typed entries and
    // payloads; opaque container nodes remain measured helper overhead.
    let mut state_credit = b.control.memory.reserve(0)?;
    let mut law = Paced::new(
        inspect::Transcript::default(),
        &b.control,
        inspect::Transcript::drain_step,
    );
    let mut log = Paced::new(None::<d0::ClosedLog>, &b.control, |v| {
        if let Some(log) = v.as_mut()
            && log.drain_step()
        {
            return true;
        }
        v.take();
        false
    });
    let mut seen = Paced::new(BTreeMap::new(), &b.control, |v| v.pop_first().is_some());
    let mut seq = 0;
    let mut length = 0;
    let mut last = None;
    let redactor = crate::redact::Set::default();
    let memory = b.control.memory.clone();
    macro_rules! observe_record {
        ($event:expr,$typed:expr) => {{
            let event = $event;
            let typed = $typed;
            inspect::validate_event(event, source, seq)?;
            let refs = inspect::blobs(&event.event)?;
            let mut payload = match &event.event {
                Event::SessionStarted { .. }
                | Event::ItemStarted { head: Some(_), .. }
                | Event::TurnEnded { usage: Some(_), .. } => typed,
                Event::ItemCompleted {
                    body: crate::protocol::Item::ToolCall { input_raw, .. },
                    ..
                } => typed
                    .checked_add(
                        match storage::preflight(input_raw.as_bytes(), storage::WORK) {
                            Ok(s) => s
                                .typed(input_raw.len())?
                                .checked_add(s.canonical()?)
                                .ok_or(w::Code::ResourceLimit)?,
                            Err(w::Code::MalformedIr) => input_raw.len() as u64,
                            Err(e) => return Err(e),
                        },
                    )
                    .ok_or(w::Code::ResourceLimit)?,
                _ => 0,
            };
            if !refs.is_empty() {
                payload = payload.max(typed);
            }
            // At most one new entry in each of twelve accumulator collections
            // (turn sets/maps, item sets/maps, calls/denials/tools/patterns);
            // two requested slots cover each Vec capacity's doubling bound.
            // Doubled Vec
            // requested capacities and fixed owner slots are declared here.
            let delta = 24u64
                .checked_mul(storage::slot())
                .and_then(|n| n.checked_add(payload))
                .ok_or(w::Code::ResourceLimit)?;
            let retained = state_credit
                .bytes
                .checked_add(delta)
                .ok_or(w::Code::ResourceLimit)?;
            let next_credit = memory.reserve(
                state_credit
                    .bytes
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(delta))
                    .ok_or(w::Code::ResourceLimit)?,
            )?;
            let old = std::mem::replace(&mut state_credit, next_credit);
            drop(old);
            law.observe(event).map_err(|_| w::Code::MalformedIr)?;
            if let Event::SessionStarted { meta } = &event.event {
                *log = Some(d0::ClosedLog::new(meta.clone()));
            }
            for r in refs {
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
                line_digest.clone(),
            ));
            let is_tool = matches!(
                &event.event,
                Event::ItemCompleted {
                    body: crate::protocol::Item::ToolCall { .. },
                    ..
                }
            );
            if !is_tool {
                log.as_mut()
                    .ok_or(w::Code::MalformedIr)?
                    .observe_pinned(event, &redactor, false, &line_digest.0)
                    .map_err(|_| w::Code::ResourceLimit)?;
            }
            state_credit.shrink(retained);
            Ok::<bool, w::Code>(is_tool)
        }};
    }
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
        while pos < n {
            let mut deferred = Paced::new(None, &b.control, disposal::Drain::drain_step);
            let mut complete = false;
            let mut is_tool = false;
            let mut shape = None;
            let count = b.counters.envelopes;
            b.unit(|| {
                let end = chunk[pos..n]
                    .iter()
                    .position(|c| *c == b'\n')
                    .map(|i| pos + i + 1)
                    .unwrap_or(n);
                if line.len() + end - pos > w::MAX_FRAME {
                    return Err(w::Code::ResourceLimit);
                }
                let part = &chunk[pos..end];
                let hashed = part.len() - usize::from(part.last() == Some(&b'\n'));
                line_hasher.update(&part[..hashed]);
                line.extend_from_slice(part);
                pos = end;
                if line.last() == Some(&b'\n') {
                    line_digest = w::Hex(format!("{:x}", line_hasher.finalize_reset()));
                    seq += 1;
                    if count >= 65536 {
                        return Err(w::Code::ResourceLimit);
                    }
                    if line.len() <= 8192 {
                        std::str::from_utf8(&line).map_err(|_| w::Code::MalformedIr)?;
                        let checked = storage::preflight(&line, storage::WORK)?;
                        shape = Some(checked);
                        // Tiny, shallow records share one measured unit. This
                        // avoids four timer-coalesced waits for tens of us CPU.
                        if checked.values + checked.keys <= 128 {
                            let typed = checked.typed(line.len())?;
                            let credit = memory.reserve(typed)?;
                            let event = Envelope::from_line(
                                std::str::from_utf8(&line).map_err(|_| w::Code::MalformedIr)?,
                            )
                            .map_err(|_| w::Code::MalformedIr)?;
                            is_tool = observe_record!(&event, typed)?;
                            if is_tool {
                                *deferred = Some(storage::Tracked {
                                    value: event,
                                    credit,
                                });
                            }
                            complete = true;
                            line.clear();
                        }
                    }
                }
                Ok(())
            })?;
            if !complete && line.last() == Some(&b'\n') {
                let shape = match shape {
                    Some(shape) => shape,
                    None => b.unit(|| {
                        std::str::from_utf8(&line).map_err(|_| w::Code::MalformedIr)?;
                        storage::preflight(&line, storage::WORK)
                    })?,
                };
                let typed = shape.typed(line.len())?;
                let event = b.track(typed, || {
                    Envelope::from_line(
                        std::str::from_utf8(&line).map_err(|_| w::Code::MalformedIr)?,
                    )
                    .map_err(|_| w::Code::MalformedIr)
                })?;
                *deferred = Some(event);
                is_tool = b.unit(|| {
                    let tool = observe_record!(&deferred.as_ref().unwrap().value, typed)?;
                    line.clear();
                    Ok(tool)
                })?;
                complete = true;
            }
            if complete {
                b.counters.envelopes += 1;
            }
            if is_tool {
                if !b.normalization_ready {
                    b.unit(|| {
                        super::fingerprint::warm_normalize_engine();
                        Ok(())
                    })?;
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
                b.unit(|| {
                    let event = deferred.as_ref().unwrap();
                    log.as_mut()
                        .ok_or(w::Code::MalformedIr)?
                        .observe_pinned(event, &redactor, false, &line_digest.0)
                        .map_err(|_| w::Code::ResourceLimit)?;
                    Ok(())
                })?;
            }
            if deferred.is_some() {
                drop(deferred);
            } else {
                deferred.take_value();
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
    drop((law, seen));
    drop(line);
    Ok(storage::Tracked {
        value: log.take().ok_or(w::Code::MalformedIr)?,
        credit: state_credit,
    })
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
        control: control.clone(),
        progress,
        counters: Counters {
            phase: w::Phase::Validate,
            bytes: 0,
            envelopes: 0,
            cpu: 0,
        },
        max_unit: control.accounting.inclusive_max.load(Ordering::Acquire),
        heap_peak: 0,
        overruns: 0,
        paused: false,
        normalization_ready: false,
        normalization_units: [0; 4],
    };
    let startup_heap = heap();
    let mut source_index = None;
    let mut result = (|| {
        b.unit(|| {
            if unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, 10) } != 0 {
                Err(w::Code::ResourceLimit)
            } else {
                Ok(())
            }
        })?;
        #[cfg(target_os = "macos")]
        {
            unsafe extern "C" {
                fn pthread_set_qos_class_self_np(class: u32, relative: i32) -> i32;
            }
            b.unit(|| {
                if unsafe { pthread_set_qos_class_self_np(0x11, 0) } != 0 {
                    Err(w::Code::ResourceLimit)
                } else {
                    Ok(())
                }
            })?;
        }
        let dir = b.unit(|| Dir::open(&path))?;
        let input = b.unit(|| dir.dir("input"))?;
        let output = b.unit(|| dir.dir("output"))?;
        let bytes = read_bounded(&input, "manifest.json", w::MAX_FRAME as u64, &mut b)?;
        let manifest_shape = b.unit(|| {
            storage::preflight(&bytes, storage::WORK).map_err(|e| {
                if e == w::Code::MalformedIr {
                    w::Code::ProtocolError
                } else {
                    e
                }
            })
        })?;
        let manifest = b.track(manifest_shape.typed(bytes.len())?, || {
            if w::hash(bytes.as_slice()) != start.manifest_sha256 {
                return Err(w::Code::SourceMismatch);
            }
            w::decode::<w::Manifest>(&bytes)
        })?;
        drop(bytes);
        let manifest = Paced::new(manifest, &b.control, disposal::Drain::drain_step);
        b.unit(|| validate_manifest(&manifest, &start))?;
        let _source_slots = b.control.memory.reserve(
            (manifest.sources.len()
                * (std::mem::size_of::<d0::ClosedLog>() + std::mem::size_of::<storage::Credit>()))
                as u64,
        )?;
        let mut state_credits = Vec::new();
        state_credits
            .try_reserve_exact(manifest.sources.len())
            .map_err(|_| w::Code::ResourceLimit)?;
        let mut logs = Paced::new(Vec::<d0::ClosedLog>::new(), &b.control, |v| {
            if let Some(log) = v.last_mut()
                && log.drain_step()
            {
                return true;
            }
            v.pop().is_some()
        });
        logs.try_reserve_exact(manifest.sources.len())
            .map_err(|_| w::Code::ResourceLimit)?;
        for (i, s) in manifest.sources.iter().enumerate() {
            source_index = Some(i as u32);
            let selected = selected(&input, s, &mut b)?;
            logs.push(selected.value);
            state_credits.push(selected.credit);
        }
        source_index = None;
        b.announce(w::Phase::Replay);
        let result_bound = d0::inspection_storage_bound_budgeted(&logs, |f| {
            b.unit(|| f().map_err(|_| w::Code::ResourceLimit))
                .map_err(|_| io::Error::other("bounded inspection stopped"))
        })
        .map_err(|_| w::Code::ResourceLimit)?;
        let _result_credit = b.control.memory.reserve(result_bound)?;
        let mut unit_error = None;
        let rows = d0::inspect_closed_budgeted(logs.take_value(), |cleanup, f| {
            let result = if cleanup {
                b.cleanup(|| f().map_err(|_| w::Code::ResourceLimit))
            } else {
                b.unit(|| f().map_err(|_| w::Code::ResourceLimit))
            };
            result.map_err(|e| {
                unit_error = Some(e);
                io::Error::other("bounded inspection stopped")
            })
        })
        .map_err(|_| unit_error.unwrap_or(w::Code::ResourceLimit))?;
        drop(state_credits);
        let mut rows = Paced::new(rows, &b.control, |v| v.pop().is_some());
        let episodes = rows
            .iter()
            .filter(|r| matches!(r, w::Row::Episode(_)))
            .count() as u32;
        let clusters = rows.len() as u32 - episodes;
        b.announce(w::Phase::Publish);
        // Page-index entries and owned digest strings remain charged through result
        // publication. Fixed admission also covers the old/new Vec growth overlap.
        let _index_credit = b
            .control
            .memory
            .reserve(8192 * (std::mem::size_of::<w::PageRef>() as u64 + 64))?;
        let mut pages = Vec::new();
        pages
            .try_reserve_exact(4096)
            .map_err(|_| w::Code::ResourceLimit)?;
        let _page_credit = b
            .control
            .memory
            .reserve(4 * 129 * (8192 + 32 * std::mem::size_of::<serde_json::Value>() as u64))?;
        let mut page = Paced::new(
            w::Page {
                schema: w::PageSchema::V1,
                run_id: start.run_id.clone(),
                manifest_sha256: start.manifest_sha256.clone(),
                index: 0,
                rows: vec![],
                next: Some(1),
            },
            &b.control,
            |v| v.rows.pop().is_some(),
        );
        let mut total = 0usize;
        let mut rows = Paced::new(rows.take_value().into_iter(), &b.control, |v| {
            v.next().is_some()
        });
        for row in rows.by_ref() {
            let mut row = Some(row);
            let size = b.unit(|| {
                if w::canonical_limited(row.as_ref().unwrap(), 8192)?.len() > 8192 {
                    return Err(w::Code::ResourceLimit);
                }
                page.rows
                    .try_reserve(1)
                    .map_err(|_| w::Code::ResourceLimit)?;
                page.rows.push(row.take().unwrap());
                w::canonical_limited(&*page, 25 * 1024).map(|v| v.len())
            })?;
            if size > 16 * 1024 || page.rows.len() > 128 {
                let last = page.rows.pop().unwrap();
                publish_page(&output, &page, &start, &mut pages, &mut total, &mut b)?;
                page.index += 1;
                page.next = Some(page.index + 1);
                let mut next = Vec::new();
                next.try_reserve_exact(1)
                    .map_err(|_| w::Code::ResourceLimit)?;
                next.push(last);
                b.cleanup(|| {
                    page.rows = next;
                    Ok(())
                })?;
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
            // PRE-DESCRIPTOR cutoff: later serialization/publication is terminal-only.
            max_poll_cpu_ns: w::Number(
                b.control
                    .accounting
                    .inclusive_max
                    .load(Ordering::Acquire)
                    .max(b.control.reactor.load(Ordering::Acquire)),
            ),
            knowledge: w::Knowledge::NotRequested,
            trust: w::Trust::NotEvaluated,
            pages,
        };
        b.control
            .accounting
            .captured_max
            .store(result.max_poll_cpu_ns.0, Ordering::Release);
        let descriptor_bound = 4096u64 + (result.pages.len() as u64) * 256;
        let _descriptor_credit = b.control.memory.reserve(
            descriptor_bound
                .checked_mul(3)
                .ok_or(w::Code::ResourceLimit)?,
        )?;
        let bytes = b.unit(|| {
            w::canonical_limited(&result, (descriptor_bound as usize).min(w::MAX_FRAME))
        })?;
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
    if b.control.cleanup_max.load(Ordering::Acquire) > UNIT_NS {
        result = Err(w::Code::ResourceLimit);
    }
    b.max_unit = b
        .max_unit
        .max(b.control.cleanup_max.load(Ordering::Acquire));
    b.overruns += b.control.cleanup_overruns.load(Ordering::Acquire);
    b.counters.cpu = b
        .counters
        .cpu
        .saturating_add(b.control.cleanup_cpu.load(Ordering::Acquire));
    b.announce(w::Phase::Settling);
    let effect = &b.control.publication;
    eprintln!(
        "dream-c1 publication_custody run_id={} nonce={} operation={} dev={} ino={} effect={} errno={} cleanup_uncertain={} path_mapping=page_index_or_4096_result temp_mapping=dot_name_original_Start_nonce_tmp",
        start.run_id.0,
        start.nonce.0,
        effect.operation.load(Ordering::Acquire),
        effect.dev.load(Ordering::Acquire),
        effect.ino.load(Ordering::Acquire),
        effect.state.load(Ordering::Acquire),
        effect.errno.load(Ordering::Acquire),
        b.control.cleanup_uncertain.load(Ordering::Acquire)
    );
    // One bounded machine-readable metrics line, with no input or IO error text.
    let (heap_live, final_peak) = heaps();
    b.heap_peak = b.heap_peak.max(final_peak);
    let (live, peak, cumulative) = b.control.memory.snapshot();
    eprintln!(
        "dream-c1 metrics heap_live_bytes={heap_live} heap_peak_bytes={} startup_heap_peak_bytes={startup_heap} compute_stack_reserved_bytes=2097152 rss_peak_bytes={} compute_cpu_ns={} max_unit_cpu_ns={} overruns={} envelopes={} bytes={} storage_declared_live_bytes={live} storage_reserved_peak_bytes={peak} storage_cumulative_credits_bytes={cumulative} control_reserved_bytes={} helper_reserved_bytes={} credit_ceiling_bytes={} native_cumulative_bytes=unmeasured normalization_unit_0_cpu_ns={} normalization_unit_1_cpu_ns={} normalization_unit_2_cpu_ns={} normalization_unit_3_cpu_ns={} cleanup_cpu_ns={} max_cleanup_unit_cpu_ns={}",
        b.heap_peak,
        rss(),
        b.counters.cpu,
        b.max_unit,
        b.overruns,
        b.counters.envelopes,
        b.counters.bytes,
        storage::CONTROL,
        storage::HELPERS,
        storage::CONTROL + storage::HELPERS + storage::WORK,
        b.normalization_units[0],
        b.normalization_units[1],
        b.normalization_units[2],
        b.normalization_units[3],
        b.control.cleanup_cpu.load(Ordering::Acquire),
        b.control.cleanup_max.load(Ordering::Acquire)
    );
    b.control.accounting.pay();
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
    let bytes = b.unit(|| w::canonical_limited(page, 16 * 1024))?;
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
    pages.try_reserve(1).map_err(|_| w::Code::ResourceLimit)?;
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
const OUT_FRAME: usize = 1024;
struct Frame {
    bytes: [u8; OUT_FRAME],
    len: usize,
}
impl std::ops::Deref for Frame {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
impl Write for Frame {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > OUT_FRAME - self.len {
            return Err(io::Error::other("bounded frame"));
        }
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn framed(out: &w::WorkerOut) -> Frame {
    let mut frame = Frame {
        bytes: [0; OUT_FRAME],
        len: 0,
    };
    // Three closed variants: all strings are constant or validated ASCII ULID/Hex;
    // at most twelve <=20-digit scalar fields. Their source-derived maximum is
    // below 1 KiB. Direct serde writing avoids canonical Value/Vec allocations.
    serde_json::to_writer(&mut frame, out).expect("bounded WorkerOut shape");
    frame.write_all(b"\n").expect("bounded WorkerOut LF");
    frame
}
/// Preflight the closed, flat WorkerIn shape before serde's tagged Content tree.
/// Valid strings need <=64 ASCII scalars (<=384 bytes when fully escaped), keys
/// <=17 ASCII scalars. No array/object value can reach buffered decoding.
fn flat_control(bytes: &[u8]) -> Result<(), w::Code> {
    fn ws(b: &[u8], i: &mut usize) {
        while b.get(*i).is_some_and(u8::is_ascii_whitespace) {
            *i += 1
        }
    }
    fn string(b: &[u8], i: &mut usize, max: usize) -> Result<(), w::Code> {
        if b.get(*i) != Some(&b'"') {
            return Err(w::Code::ProtocolError);
        }
        *i += 1;
        let start = *i;
        while let Some(c) = b.get(*i) {
            if *i - start > max {
                return Err(w::Code::ResourceLimit);
            }
            match c {
                b'"' => {
                    *i += 1;
                    return Ok(());
                }
                b'\\' => *i += 2,
                _ => *i += 1,
            }
        }
        Err(w::Code::ProtocolError)
    }
    let mut i = 0;
    ws(bytes, &mut i);
    if bytes.get(i) != Some(&b'{') {
        return Err(w::Code::ProtocolError);
    }
    i += 1;
    let mut fields = 0;
    loop {
        ws(bytes, &mut i);
        if bytes.get(i) == Some(&b'}') {
            i += 1;
            break;
        }
        fields += 1;
        if fields > 10 {
            return Err(w::Code::ProtocolError);
        }
        string(bytes, &mut i, 128)?;
        ws(bytes, &mut i);
        if bytes.get(i) != Some(&b':') {
            return Err(w::Code::ProtocolError);
        }
        i += 1;
        ws(bytes, &mut i);
        if bytes.get(i) == Some(&b'"') {
            string(bytes, &mut i, 384)?
        } else {
            let start = i;
            while bytes
                .get(i)
                .is_some_and(|c| !matches!(c, b',' | b'}') && !c.is_ascii_whitespace())
            {
                if matches!(bytes[i], b'[' | b'{' | b'"') {
                    return Err(w::Code::ProtocolError);
                }
                i += 1;
                if i - start > 32 {
                    return Err(w::Code::ResourceLimit);
                }
            }
            if i == start {
                return Err(w::Code::ProtocolError);
            }
        }
        ws(bytes, &mut i);
        match bytes.get(i) {
            Some(b',') => i += 1,
            Some(b'}') => {
                i += 1;
                break;
            }
            _ => return Err(w::Code::ProtocolError),
        }
    }
    ws(bytes, &mut i);
    if i != bytes.len() {
        return Err(w::Code::ProtocolError);
    }
    Ok(())
}
fn settled_result(
    control: &Control,
    result: Result<Completed, w::Error>,
) -> Result<Completed, w::Error> {
    if let Some(code) = control.error() {
        Err(w::Error { code, source: None })
    } else {
        result
    }
}
type Active = (
    w::Start,
    Arc<Control>,
    Arc<crossbeam_queue::ArrayQueue<Counters>>,
);
struct ComputeWork {
    path: PathBuf,
    start: w::Start,
    control: Arc<Control>,
    progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
}
struct ParkedOwner {
    accounting: Arc<Accounting>,
    sender: Option<std::sync::mpsc::SyncSender<ComputeWork>>,
    handle: Option<thread::JoinHandle<Result<Completed, w::Error>>>,
}
impl Drop for ParkedOwner {
    fn drop(&mut self) {
        // EOF/malformed/missing Start disconnects the parked owner, then joins
        // the actual returned handle, including timing-failed creation.
        self.accounting.pay();
        drop(self.sender.take());
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
fn bootstrap_failure(
    error: bootstrap::Error,
    usage: bootstrap::Usage,
    accounting: &Accounting,
    reactor: &AtomicU64,
) -> i32 {
    bootstrap::merge_usage(usage, reactor, &accounting.control_cpu);
    if error == bootstrap::Error::Clock {
        accounting.clock_failed.store(true, Ordering::Release);
        reactor.fetch_max(u64::MAX, Ordering::AcqRel);
    } else if error == bootstrap::Error::Limit {
        accounting.resource_failed.store(true, Ordering::Release);
    }
    2
}
fn run(path: PathBuf, reactor: Arc<AtomicU64>, mut gate: bootstrap::Gate) -> i32 {
    if gate.admit(1).is_err() {
        return 2;
    }
    let bootstrap_at = cpu_ns();
    let accounting = Arc::new(Accounting::new());
    let _terminal = TerminalMetrics {
        accounting: accounting.clone(),
        control: reactor.clone(),
    };
    let (sender, receiver) = std::sync::mpsc::sync_channel::<ComputeWork>(1);
    let owner_accounting = accounting.clone();
    let builder = thread::Builder::new()
        .name("dream-c1-compute".into())
        .stack_size(2 * 1024 * 1024);
    // Only Builder::spawn itself is tagged. Channel/closure/builder setup above
    // is ordinary bootstrap application work, with no filesystem/provider access.
    let entry = move || {
        let at = cpu_ns();
        let result = match receiver.recv() {
            Ok(w) => compute(w.path, w.start, w.control, w.progress),
            Err(_) => Err(w::Error {
                code: w::Code::Cancelled,
                source: None,
            }),
        };
        owner_result(&owner_accounting, at, result, cpu_ns)
    };
    let setup_cpu = interval(bootstrap_at, cpu_ns());
    accounting.charge(setup_cpu, setup_cpu);
    if setup_cpu > UNIT_NS {
        return 1;
    }
    accounting.pay();
    if let Err(e) = gate.admit(2) {
        return bootstrap_failure(e, gate.usage, &accounting, &reactor);
    }
    let created = match native_call(
        &accounting,
        NativeKind::ComputeOwnerCreate,
        || builder.spawn(entry),
        |r| r.is_ok(),
    ) {
        Ok(out) => out,
        Err(_) => return 1,
    };
    let mut owner = ParkedOwner {
        accounting: accounting.clone(),
        sender: Some(sender),
        handle: created.value.ok(),
    };
    if created.verdict.is_err() || owner.handle.is_none() {
        return 1;
    }
    // Bootstrap native debt is fully paid BEFORE Hello or first work.
    accounting.pay();
    if let Err(e) = gate.admit(3) {
        return bootstrap_failure(e, gate.usage, &accounting, &reactor);
    }
    let setup_at = cpu_ns();
    if nonblocking(0).is_err() || nonblocking(1).is_err() {
        return 2;
    }
    let mut pending = VecDeque::new();
    if pending.try_reserve_exact(33).is_err() {
        return 1;
    }
    pending.push_back(framed(&w::WorkerOut::Hello(w::Hello {
        protocol: w::Protocol::V1,
        algorithm: w::Algorithm::D0C1,
        schema_sha256: w::schema_hash(),
        ir_major: 1,
        ir_minor_max: 3,
    })));
    let mut offset = 0;
    let mut input = Vec::new();
    if input.try_reserve_exact(w::MAX_FRAME).is_err() {
        return 1;
    }
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
    let setup_cpu = interval(setup_at, cpu_ns());
    accounting.charge(setup_cpu, setup_cpu);
    if setup_cpu > UNIT_NS {
        return 1;
    }
    accounting.pay();
    let (permit, bootstrap_usage) = match gate.retire() {
        Ok(p) => p,
        Err(e) => return bootstrap_failure(e, gate.usage, &accounting, &reactor),
    };
    drop(gate);
    bootstrap::merge_usage(bootstrap_usage, &reactor, &accounting.control_cpu);
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
        let mut tick_guard = ControlTick {
            at: tick,
            part_at: tick,
            accounting: &accounting,
            max: &reactor,
        };
        if !eof && (fds[0].revents & (libc::POLLIN | libc::POLLHUP)) != 0 {
            let n = unsafe { libc::read(0, chunk.as_mut_ptr().cast(), chunk.len()) };
            if n == 0 {
                eof = true;
                if let Some((_, c, _)) = &state {
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
                        let msg =
                            flat_control(&input).and_then(|()| w::decode::<w::WorkerIn>(&input));
                        input.clear();
                        match (msg, &mut state) {
                            (Ok(w::WorkerIn::Start(s)), None)
                                if exit.is_none()
                                    && !permit.probe
                                    && bootstrap::ulid(s.run_id.0.as_bytes()).ok()
                                        == Some(permit.run_id)
                                    && bootstrap::ulid(s.nonce.0.as_bytes()).ok()
                                        == Some(permit.nonce)
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
                                    memory: storage::Ledger::new(),
                                    accounting: accounting.clone(),
                                    publication: PublicationCustody::default(),
                                    cleanup_uncertain: AtomicBool::new(false),
                                    cleanup_cpu: AtomicU64::new(0),
                                    cleanup_max: AtomicU64::new(0),
                                    cleanup_overruns: AtomicU64::new(0),
                                });
                                let tx = Arc::new(crossbeam_queue::ArrayQueue::new(32));
                                let rx = tx.clone();
                                let cc = c.clone();
                                let ss = s.clone();
                                let p = path.clone();
                                let work = ComputeWork {
                                    path: p,
                                    start: ss,
                                    control: cc,
                                    progress: tx,
                                };
                                if owner.sender.take().unwrap().try_send(work).is_ok() {
                                    state = Some((s, c, rx));
                                } else {
                                    protocol_failed = true;
                                }
                            }
                            (Ok(w::WorkerIn::Activity(a)), Some((s, c, lane)))
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
                            (Ok(w::WorkerIn::Cancel(a)), Some((s, c, _)))
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
        tick_guard.part(0);
        if protocol_failed {
            eof = true;
            if let Some((_, c, _)) = &state {
                c.cancel.store(3, Ordering::Release);
            } else if exit.is_some() {
                exit = Some(2);
            } else {
                return 2;
            }
        }
        tick_guard.part(1);
        if let Some((s, c, rx)) = &state {
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
            if owner.handle.as_ref().unwrap().is_finished() {
                let (s, c, rx) = state.take().unwrap();
                let h = owner.handle.take().unwrap();
                drop(rx);
                let result = h.join().unwrap_or(Err(w::Error {
                    code: w::Code::WorkerLost,
                    source: None,
                }));
                let result = settled_result(&c, result);
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
        tick_guard.part(2);
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
                if let Some((_, c, _)) = &state {
                    c.cancel.store(1, Ordering::Release);
                    pending.clear();
                    offset = 0;
                } else {
                    return 2;
                }
            }
        }
        tick_guard.part(3);
        drop(tick_guard);
        let elapsed = interval(tick, cpu_ns());
        accounting
            .inclusive_max
            .fetch_max(elapsed, Ordering::AcqRel);
        max_control = max_control.max(elapsed);
        reactor.fetch_max(elapsed, Ordering::AcqRel);
        if elapsed > UNIT_NS {
            if let Some((_, c, _)) = &state {
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
                memory: storage::Ledger::new(),
                accounting: Arc::new(Accounting::new()),
                publication: PublicationCustody::default(),
                cleanup_uncertain: AtomicBool::new(false),
                cleanup_cpu: AtomicU64::new(0),
                cleanup_max: AtomicU64::new(0),
                cleanup_overruns: AtomicU64::new(0),
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
            paused: false,
            normalization_ready: false,
            normalization_units: [0; 4],
        }
    }
    #[test]
    fn native_verdict_retains_real_owner_on_cpu_wall_clock_cancel_and_repeated_admission() {
        for failure in 0..4 {
            let accounting = Arc::new(Accounting::new());
            let (tx, rx) = std::sync::mpsc::channel();
            let ended = Arc::new(AtomicBool::new(false));
            let end_flag = ended.clone();
            let builder = thread::Builder::new().stack_size(2 * 1024 * 1024);
            let base = Instant::now();
            let mut c = [
                0,
                10,
                if failure == 0 {
                    2_000_011
                } else if failure == 2 {
                    u64::MAX
                } else {
                    500_010
                },
                if failure == 2 {
                    u64::MAX
                } else if failure == 0 {
                    2_000_021
                } else {
                    500_020
                },
            ]
            .into_iter();
            let mut wall = [
                base,
                base + Duration::from_millis(if failure == 1 { 26 } else { 1 }),
            ]
            .into_iter();
            let out = native_call_clock(
                &accounting,
                NativeKind::ComputeOwnerCreate,
                || {
                    builder.spawn(move || {
                        rx.recv().unwrap();
                        end_flag.store(true, Ordering::Release);
                    })
                },
                |r| r.is_ok(),
                || c.next().unwrap(),
                || wall.next().unwrap(),
            )
            .unwrap();
            let handle = out.value.unwrap();
            if failure < 3 {
                assert_eq!(out.verdict, Err(w::Code::ResourceLimit));
            } else {
                assert_eq!(out.verdict, Ok(()));
            }
            assert!(
                !handle.is_finished(),
                "failed verdict lost the actual parked owner"
            );
            // Independent control makes progress while actual owner is withheld.
            let control = AtomicBool::new(false);
            control.store(true, Ordering::Release);
            assert!(control.load(Ordering::Acquire));
            assert!(
                native_call(
                    &accounting,
                    NativeKind::ComputeOwnerCreate,
                    || panic!("repeated creation admitted"),
                    |_: &()| false
                )
                .is_err()
            );
            tx.send(()).unwrap();
            handle.join().unwrap();
            assert!(ended.load(Ordering::Acquire));
            accounting.pay();
            assert!(accounting.now() >= accounting.rest_until.load(Ordering::Acquire));
            assert_eq!(accounting.native[0].count.load(Ordering::Acquire), 1);
            assert_eq!(accounting.native[0].effects.load(Ordering::Acquire), 1);
            eprintln!(
                "native-owner-failure mode={failure} actual_join=true clock_fixture=true effect=present"
            );
        }
    }
    #[test]
    fn publication_effect_survives_timing_failure_and_missing_entry_clock_never_executes() {
        for cpu_failure in [true, false] {
            let home = tempfile::tempdir().unwrap();
            let staged = home.path().join("stage");
            let published = home.path().join("page-000000.json");
            std::fs::write(&staged, b"retained immutable effect").unwrap();
            let before = std::fs::metadata(&staged).unwrap();
            let a = Accounting::new();
            let base = Instant::now();
            let mut c = [
                0,
                1,
                if cpu_failure { NATIVE_CPU_NS + 2 } else { 2 },
                if cpu_failure { NATIVE_CPU_NS + 3 } else { 3 },
            ]
            .into_iter();
            let mut wall = [
                base,
                base + Duration::from_millis(if cpu_failure { 1 } else { 26 }),
            ]
            .into_iter();
            let out = native_call_clock(
                &a,
                NativeKind::PublishNoreplace,
                || std::fs::hard_link(&staged, &published),
                |r| r.is_ok(),
                || c.next().unwrap(),
                || wall.next().unwrap(),
            )
            .unwrap();
            assert!(out.value.is_ok());
            assert_eq!(out.verdict, Err(w::Code::ResourceLimit));
            let effect = std::fs::metadata(&published).unwrap();
            assert_eq!((effect.dev(), effect.ino()), (before.dev(), before.ino()));
            assert_eq!(
                std::fs::read(&published).unwrap(),
                b"retained immutable effect"
            );
            assert!(
                native_call(
                    &a,
                    NativeKind::PublishNoreplace,
                    || panic!("failed effect repeated"),
                    |_: &()| false
                )
                .is_err()
            );
            assert_eq!(a.native[1].failures_with_effect.load(Ordering::Acquire), 1);
        }
        let a = Accounting::new();
        assert!(
            native_call_clock(
                &a,
                NativeKind::ComputeOwnerCreate,
                || panic!("missing clock admitted effect"),
                |_: &()| false,
                || u64::MAX,
                Instant::now
            )
            .is_err()
        );
        assert_eq!(a.native[0].count.load(Ordering::Acquire), 0);
    }
    #[test]
    fn original_unknown_publication_errno_survives_postcall_clock_failure() {
        let home = tempfile::tempdir().unwrap();
        let stage = home.path().join(".page-000000.original_nonce.tmp");
        let dest = home.path().join("page-000000.json");
        std::fs::write(&stage, b"unknown-effect-evidence").unwrap();
        let metadata = std::fs::metadata(&stage).unwrap();
        let custody = PublicationCustody::default();
        let a = Accounting::new();
        let mut reads = 0;
        let out = native_call_clock(
            &a,
            NativeKind::PublishNoreplace,
            || {
                // Actual staged inode plus an injected indeterminate return. This
                // is an adversarial effect fixture, never a native timing benchmark.
                std::fs::hard_link(&stage, &dest).unwrap();
                unsafe {
                    *errno_pointer() = libc::EIO;
                }
                -1
            },
            |status| {
                let errno = unsafe { *errno_pointer() };
                custody.retain(&PublicationResult {
                    status: *status,
                    errno,
                    operation: 0,
                    dev: metadata.dev(),
                    ino: metadata.ino(),
                    nonce: *b"01ARZ3NDEKTSV4RRFFQ69G5FAV",
                });
                false
            },
            || {
                reads += 1;
                if reads >= 3 {
                    unsafe {
                        *errno_pointer() = libc::EINVAL;
                    }
                    u64::MAX
                } else {
                    reads
                }
            },
            Instant::now,
        )
        .unwrap();
        assert_eq!(out.value, -1);
        assert_eq!(out.verdict, Err(w::Code::ResourceLimit));
        assert_eq!(custody.errno.load(Ordering::Acquire), libc::EIO as u64);
        assert_eq!(custody.state.load(Ordering::Acquire), 2);
        assert_eq!(std::fs::metadata(&dest).unwrap().ino(), metadata.ino());
        assert!(
            stage.exists() && dest.exists(),
            "unknown effect evidence was removed"
        );
    }
    #[test]
    fn reversed_cleanup_clock_retains_disposal_effect_and_sticky_unknown_cost() {
        let control = budget(false).control;
        let credit = control.memory.reserve(1024).unwrap();
        let dropped = Arc::new(AtomicBool::new(false));
        let mut value = Paced::new(
            DropProbe {
                remaining: 8,
                control: control.clone(),
                dropped: dropped.clone(),
            },
            &control,
            disposal::Drain::drain_step,
        );
        let home = tempfile::tempdir().unwrap();
        let stage = home.path().join("stage");
        let output = home.path().join("page-000000.json");
        std::fs::write(&stage, b"original-publication-effect").unwrap();
        std::fs::hard_link(&stage, &output).unwrap();
        let inode = std::fs::metadata(&output).unwrap().ino();
        let mut clocks = [100, 99].into_iter();
        cleanup_unit_clock(
            &control,
            || {
                assert!(disposal::Drain::drain_step(&mut *value));
            },
            || clocks.next().unwrap(),
        );
        assert_eq!(control.error(), Some(w::Code::ResourceLimit));
        assert!(control.accounting.clock_failed.load(Ordering::Acquire));
        assert_eq!(
            control.cleanup_max.load(Ordering::Acquire),
            u64::MAX,
            "reversed cost was certified as zero"
        );
        assert_eq!(control.cleanup_overruns.load(Ordering::Acquire), 1);
        assert!(!dropped.load(Ordering::Acquire));
        assert_eq!(control.memory.snapshot().0, 1024);
        let mut b = budget(false);
        b.control = control.clone();
        assert_eq!(
            b.unit(|| panic!("unknown cleanup cost admitted work")),
            Err::<(), _>(w::Code::ResourceLimit)
        );
        drop(value);
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(control.memory.snapshot().0, 1024);
        drop(credit);
        assert_eq!(control.memory.snapshot().0, 0);
        assert_eq!(std::fs::metadata(&output).unwrap().ino(), inode);
        assert!(stage.exists());
        assert!(
            control.accounting.clock_failed.load(Ordering::Acquire),
            "later real cleanup reset unknown cost"
        );
    }
    #[test]
    fn final_owner_clock_and_process_usage_fail_closed_after_actual_join() {
        for end in [u64::MAX, 99] {
            let control = budget(false).control;
            let child_control = control.clone();
            let handle = thread::Builder::new()
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    // Only the final owner sample is injected; preceding accounting
                    // and the actual returned owner/Completed value remain valid.
                    owner_result(
                        &child_control.accounting,
                        100,
                        Ok(Completed {
                            hash: w::hash(b"owned result"),
                            pages: 1,
                        }),
                        || end,
                    )
                })
                .unwrap();
            let result = handle.join().unwrap();
            assert!(result.is_ok(), "tail probe lost the real returned result");
            assert!(control.accounting.clock_failed.load(Ordering::Acquire));
            assert_eq!(
                control.accounting.owner_cpu.load(Ordering::Acquire),
                u64::MAX
            );
            assert!(matches!(
                settled_result(&control, result),
                Err(w::Error {
                    code: w::Code::ResourceLimit,
                    ..
                })
            ));
        }
        let control = budget(false).control;
        assert_eq!(
            process_usage(&control.accounting, &control.reactor, || None),
            (u64::MAX, u64::MAX)
        );
        assert!(control.accounting.clock_failed.load(Ordering::Acquire));
        assert_eq!(
            control.reactor.load(Ordering::Acquire),
            u64::MAX,
            "final usage failure could still exit zero"
        );
        assert_eq!(control.error(), Some(w::Code::ResourceLimit));
    }
    #[test]
    fn rest_is_after_span_and_strict_parser_control_cleanup_have_no_native_exemption() {
        let mut b = budget(false);
        b.unit(|| {
            thread::sleep(Duration::from_millis(12));
            Ok(())
        })
        .unwrap();
        let end = Instant::now();
        b.unit(|| Ok(())).unwrap();
        assert!(
            end.elapsed() >= WAIT,
            "IO time incorrectly paid post-span debt"
        );
        let control = budget(false).control;
        cleanup_unit(&control, || {
            let at = cpu_ns();
            while interval(at, cpu_ns()) < 2 * UNIT_NS {
                std::hint::spin_loop();
            }
        });
        assert_eq!(control.error(), Some(w::Code::ResourceLimit));
        let a = Accounting::new();
        let maximum = AtomicU64::new(0);
        {
            let at = cpu_ns();
            let _tick = ControlTick {
                at,
                part_at: at,
                accounting: &a,
                max: &maximum,
            };
            while interval(at, cpu_ns()) < 2 * UNIT_NS {
                std::hint::spin_loop();
            }
        }
        assert!(maximum.load(Ordering::Acquire) > UNIT_NS);
        assert_eq!(a.native[0].count.load(Ordering::Acquire), 0);
        assert_eq!(a.native[1].count.load(Ordering::Acquire), 0);
    }
    /// A leaf destructor is cheap only after every owned piece was stepped.
    struct DropProbe {
        remaining: usize,
        control: Arc<Control>,
        dropped: Arc<AtomicBool>,
    }
    impl disposal::Drain for DropProbe {
        fn drain_step(&mut self) -> bool {
            if self.remaining == 0 {
                return false;
            }
            let at = cpu_ns();
            while cpu_ns().saturating_sub(at) < 120_000 {
                std::hint::spin_loop();
            }
            self.remaining -= 1;
            true
        }
    }
    impl Drop for DropProbe {
        fn drop(&mut self) {
            assert_eq!(self.remaining, 0, "unstepped owned-result destructor");
            assert_eq!(
                self.control.memory.snapshot().0,
                1024,
                "credit released ahead of ownership"
            );
            self.dropped.store(true, Ordering::Release);
        }
    }
    #[test]
    fn rejected_owned_result_drains_before_credit_release_for_cpu_clock_and_cancel() {
        for failure in 0..3 {
            let mut b = budget(false);
            let control = b.control.clone();
            let dropped = Arc::new(AtomicBool::new(false));
            let at = Instant::now();
            let mut reads = 0;
            let result = b.track_clock(
                1024,
                || {
                    if failure == 0 {
                        let at = cpu_ns();
                        while cpu_ns().saturating_sub(at) <= 2 * UNIT_NS {
                            std::hint::spin_loop();
                        }
                    } else if failure == 2 {
                        control.cancel.store(1, Ordering::Release);
                    }
                    Ok(DropProbe {
                        remaining: 8,
                        control: control.clone(),
                        dropped: dropped.clone(),
                    })
                },
                || {
                    reads += 1;
                    if failure == 1 && reads == 2 {
                        u64::MAX
                    } else {
                        cpu_ns()
                    }
                },
            );
            assert!(
                matches!(result, Err(w::Code::Cancelled) if failure == 2)
                    || matches!(result, Err(w::Code::ResourceLimit) if failure != 2)
            );
            assert!(
                dropped.load(Ordering::Acquire),
                "return preceded real destruction"
            );
            assert_eq!(control.memory.snapshot().0, 0);
            assert!(at.elapsed() >= WAIT * 2, "cleanup was not stepped/paced");
            assert!(control.cleanup_cpu.load(Ordering::Acquire) >= 8 * 120_000);
            assert!(control.cleanup_max.load(Ordering::Acquire) <= UNIT_NS);
            assert_eq!(control.cleanup_overruns.load(Ordering::Acquire), 0);
        }
        let mut b = budget(false);
        assert!(matches!(
            b.unit_clock::<()>(
                || panic!("unavailable start clock admitted work"),
                || u64::MAX
            ),
            Err(w::Code::ResourceLimit)
        ));
    }
    #[test]
    fn near_limit_parsed_envelope_failure_and_cancel_have_paced_disposal() {
        use serde_json::json;
        // Real serde envelope, nested arbitrary JSON plus near-frame raw Arc.
        let mut input = json!({
            "v":{"major":1,"minor":3},"session":"01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "epoch":1,"seq":2,"at":2,"ts_ms":0,"cause":null,
            "event":{"type":"session_closed"},
            "ext":{"dense":[ ],"padding":""}
        });
        input["ext"]["dense"] =
            serde_json::Value::Array((0..4096).map(|_| json!({"leaf":["retained"]})).collect());
        let bytes = serde_json::to_vec(&input).unwrap();
        input["ext"]["padding"] = json!("p".repeat(w::MAX_FRAME - bytes.len() - 32));
        let bytes = serde_json::to_vec(&input).unwrap();
        assert!(bytes.len() > w::MAX_FRAME - 64 && bytes.len() < w::MAX_FRAME);
        let typed = storage::preflight(&bytes, storage::WORK)
            .unwrap()
            .typed(bytes.len())
            .unwrap();
        for failure in 0..3 {
            let mut b = budget(false);
            let control = b.control.clone();
            let mut reads = 0;
            let at = Instant::now();
            let result = b.track_clock(
                typed,
                || {
                    let e = Envelope::from_line(std::str::from_utf8(&bytes).unwrap()).unwrap();
                    if failure == 2 {
                        control.cancel.store(1, Ordering::Release);
                    }
                    Ok(e)
                },
                || {
                    reads += 1;
                    // Reject a successfully parsed owner, independently of parsing speed.
                    match (failure, reads) {
                        (1, 2) => u64::MAX,
                        (0, 1) => 0,
                        (0, 2) => UNIT_NS + 1,
                        (2, 1) => 0,
                        (2, 2) => 1,
                        _ => cpu_ns(),
                    }
                },
            );
            let expected = if failure == 2 {
                w::Code::Cancelled
            } else {
                w::Code::ResourceLimit
            };
            assert!(matches!(result, Err(code) if code == expected));
            assert_eq!(control.memory.snapshot().0, 0);
            assert!(
                at.elapsed() >= WAIT * 8,
                "near-limit owner destroyed as one unit"
            );
            assert!(control.cleanup_cpu.load(Ordering::Acquire) > 0);
            assert!(control.cleanup_max.load(Ordering::Acquire) <= UNIT_NS);
            assert_eq!(control.cleanup_overruns.load(Ordering::Acquire), 0);
            eprintln!(
                "dream-c1 owned_envelope_disposal failure={failure} bytes={} cleanup_cpu_ns={} cleanup_max_cpu_ns={} live_credits={}",
                bytes.len(),
                control.cleanup_cpu.load(Ordering::Acquire),
                control.cleanup_max.load(Ordering::Acquire),
                control.memory.snapshot().0
            );
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
        let (entered, ready) = std::sync::mpsc::channel();
        let h = thread::spawn(move || {
            entered.send(()).unwrap();
            b.unit(|| panic!("foreground unit admitted"))
        });
        ready
            .recv_timeout(Duration::from_millis(250))
            .expect("test owner did not enter");
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

#[cfg(test)]
mod control_storage {
    use super::*;
    #[test]
    fn flat_preflight_rejects_nested_allocations_and_accepts_escaped_keys() {
        assert_eq!(
            flat_control(br#"{"type":"start","payload":[[[{"x":"y"}]]]}"#),
            Err(w::Code::ProtocolError)
        );
        let escaped=br#"{"\u0061\u0063\u0074\u0069\u0076\u0069\u0074\u0079\u005f\u0072\u0065\u0076\u0069\u0073\u0069\u006f\u006e":1}"#;
        assert!(flat_control(escaped).is_ok());
        assert_eq!(
            std::mem::size_of::<Frame>(),
            OUT_FRAME + std::mem::size_of::<usize>()
        );
        assert!(33 * std::mem::size_of::<Frame>() + w::MAX_FRAME < 2 * 1024 * 1024);
    }
}

/// Declared owned-storage credits. Opaque serde/BTree/regex/native overhead uses
/// the separate fixed 16 MiB helper allowance and measured full-child ceiling.
/// Credits are not an allocator interception or an all-times native heap proof.
pub(crate) mod storage {
    use super::*;
    pub const CONTROL: u64 = 2 * 1024 * 1024;
    pub const HELPERS: u64 = 16 * 1024 * 1024;
    pub const WORK: u64 = 78 * 1024 * 1024;
    pub struct Ledger {
        live: AtomicU64,
        peak: AtomicU64,
        cumulative: AtomicU64,
    }
    impl Ledger {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                live: AtomicU64::new(0),
                peak: AtomicU64::new(0),
                cumulative: AtomicU64::new(0),
            })
        }
        pub fn reserve(self: &Arc<Self>, bytes: u64) -> Result<Credit, w::Code> {
            let mut old = self.live.load(Ordering::Acquire);
            loop {
                let next = old
                    .checked_add(bytes)
                    .filter(|n| *n <= WORK)
                    .ok_or(w::Code::ResourceLimit)?;
                match self.live.compare_exchange_weak(
                    old,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.peak.fetch_max(next, Ordering::Relaxed);
                        self.cumulative.fetch_add(bytes, Ordering::Relaxed);
                        return Ok(Credit {
                            ledger: self.clone(),
                            bytes,
                        });
                    }
                    Err(now) => old = now,
                }
            }
        }
        pub fn snapshot(&self) -> (u64, u64, u64) {
            (
                self.live.load(Ordering::Acquire),
                self.peak.load(Ordering::Acquire),
                self.cumulative.load(Ordering::Acquire),
            )
        }
    }
    pub struct Credit {
        ledger: Arc<Ledger>,
        pub bytes: u64,
    }
    impl Credit {
        pub fn shrink(&mut self, bytes: u64) {
            assert!(bytes <= self.bytes);
            self.ledger
                .live
                .fetch_sub(self.bytes - bytes, Ordering::AcqRel);
            self.bytes = bytes;
        }
    }
    impl Drop for Credit {
        fn drop(&mut self) {
            self.ledger.live.fetch_sub(self.bytes, Ordering::AcqRel);
        }
    }
    pub struct Tracked<T> {
        pub value: T,
        pub credit: Credit,
    }
    impl<T> std::ops::Deref for Tracked<T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.value
        }
    }
    impl<T> std::ops::DerefMut for Tracked<T> {
        fn deref_mut(&mut self) -> &mut T {
            &mut self.value
        }
    }
    /// Conservative declared IR typed-slot bound. These owners contain the largest
    /// inline protocol variants; strings and Vec payload capacities are additional.
    /// Internal BTree nodes/serde Content are helper overhead, not these slot bytes.
    pub fn slot() -> u64 {
        (std::mem::size_of::<crate::protocol::Envelope>()
            + std::mem::size_of::<crate::protocol::Item>()
            + std::mem::size_of::<crate::protocol::Usage>()
            + std::mem::size_of::<crate::protocol::SessionMeta>()) as u64
    }
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Shape {
        pub values: u64,
        pub keys: u64,
        pub strings: u64,
        pub containers: u64,
    }
    impl Shape {
        /// Two requested typed slots per admitted value/key cover Vec capacity growth;
        /// two raw-string capacities cover decoded owned strings and their growth.
        /// Raw Arc line storage and the Envelope owner are charged separately here.
        pub fn typed(&self, raw: usize) -> Result<u64, w::Code> {
            self.values
                .checked_add(self.keys)
                .and_then(|n| n.checked_mul(slot()))
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| n.checked_add(self.strings.checked_mul(2)?))
                .and_then(|n| n.checked_add(raw as u64 + slot()))
                .ok_or(w::Code::ResourceLimit)
        }
        /// JSON string serialization cannot exceed its original escaped spelling;
        /// numeric serialization uses <=24 bytes, plus container/member delimiters.
        pub fn canonical(&self) -> Result<u64, w::Code> {
            self.values
                .checked_mul(26)
                .and_then(|n| n.checked_add(self.keys.checked_mul(3)?))
                .and_then(|n| n.checked_add(self.strings))
                .and_then(|n| n.checked_add(self.containers.checked_mul(2)?))
                .ok_or(w::Code::ResourceLimit)
        }
    }
    /// Non-materializing preflight: depth32, 4096 members/container, 65536 total
    /// values+keys. Early declared-credit refusal precedes general serde trees.
    pub fn preflight(raw: &[u8], available: u64) -> Result<Shape, w::Code> {
        struct Scan<'a> {
            b: &'a [u8],
            i: usize,
            s: Shape,
            available: u64,
        }
        impl Scan<'_> {
            fn ws(&mut self) {
                while self.b.get(self.i).is_some_and(u8::is_ascii_whitespace) {
                    self.i += 1
                }
            }
            fn count(&mut self, key: bool) -> Result<(), w::Code> {
                if key {
                    self.s.keys += 1
                } else {
                    self.s.values += 1
                }
                if self.s.values + self.s.keys > 65536
                    || self.s.typed(self.b.len())? > self.available
                {
                    return Err(w::Code::ResourceLimit);
                }
                Ok(())
            }
            fn string(&mut self) -> Result<(), w::Code> {
                if self.b.get(self.i) != Some(&b'"') {
                    return Err(w::Code::MalformedIr);
                }
                let at = self.i;
                self.i += 1;
                loop {
                    match self.b.get(self.i) {
                        Some(b'"') => {
                            self.i += 1;
                            self.s.strings += (self.i - at) as u64;
                            return Ok(());
                        }
                        Some(b'\\') => {
                            self.i += 1;
                            match self.b.get(self.i) {
                                Some(b'u') => {
                                    for _ in 0..4 {
                                        self.i += 1;
                                        if !self.b.get(self.i).is_some_and(u8::is_ascii_hexdigit) {
                                            return Err(w::Code::MalformedIr);
                                        }
                                    }
                                    self.i += 1
                                }
                                Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                                    self.i += 1
                                }
                                _ => return Err(w::Code::MalformedIr),
                            }
                        }
                        Some(c) if *c >= 0x20 => self.i += 1,
                        _ => return Err(w::Code::MalformedIr),
                    }
                }
            }
            fn value(&mut self, depth: u8) -> Result<(), w::Code> {
                self.ws();
                self.count(false)?;
                match self.b.get(self.i) {
                    Some(b'"') => self.string(),
                    Some(open @ (b'{' | b'[')) => {
                        if depth >= 32 {
                            return Err(w::Code::ResourceLimit);
                        }
                        let object = *open == b'{';
                        let close = if object { b'}' } else { b']' };
                        self.i += 1;
                        self.s.containers += 1;
                        self.ws();
                        if self.b.get(self.i) == Some(&close) {
                            self.i += 1;
                            return Ok(());
                        }
                        let mut members = 0;
                        loop {
                            members += 1;
                            if members > 4096 {
                                return Err(w::Code::ResourceLimit);
                            }
                            if object {
                                self.ws();
                                self.count(true)?;
                                self.string()?;
                                self.ws();
                                if self.b.get(self.i) != Some(&b':') {
                                    return Err(w::Code::MalformedIr);
                                }
                                self.i += 1
                            }
                            self.value(depth + 1)?;
                            self.ws();
                            match self.b.get(self.i) {
                                Some(c) if *c == close => {
                                    self.i += 1;
                                    break;
                                }
                                Some(b',') => self.i += 1,
                                _ => return Err(w::Code::MalformedIr),
                            }
                        }
                        Ok(())
                    }
                    Some(b't') => self.literal(b"true"),
                    Some(b'f') => self.literal(b"false"),
                    Some(b'n') => self.literal(b"null"),
                    Some(b'-' | b'0'..=b'9') => {
                        let start = self.i;
                        while self.b.get(self.i).is_some_and(|c| {
                            matches!(c, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                        }) {
                            self.i += 1
                        }
                        if self.i == start {
                            return Err(w::Code::MalformedIr);
                        }
                        Ok(())
                    }
                    _ => Err(w::Code::MalformedIr),
                }
            }
            fn literal(&mut self, v: &[u8]) -> Result<(), w::Code> {
                if !self.b[self.i..].starts_with(v) {
                    return Err(w::Code::MalformedIr);
                }
                self.i += v.len();
                Ok(())
            }
        }
        if raw.len() > w::MAX_FRAME {
            return Err(w::Code::ResourceLimit);
        }
        let mut scan = Scan {
            b: raw,
            i: 0,
            s: Shape::default(),
            available,
        };
        scan.value(0)?;
        scan.ws();
        if scan.i != raw.len() {
            return Err(w::Code::MalformedIr);
        }
        if scan.s.typed(raw.len())? > available {
            return Err(w::Code::ResourceLimit);
        }
        Ok(scan.s)
    }
}

#[cfg(test)]
mod storage_tests {
    use super::{storage::*, *};
    #[test]
    fn credits_refuse_growth_and_overlap_preserve_control_and_release_on_transfer() {
        assert_eq!(WORK + CONTROL + HELPERS, HEAP_MAX);
        let ledger = Ledger::new();
        let held = ledger.reserve(WORK - 8).unwrap();
        let old = ledger.reserve(8).unwrap();
        assert!(matches!(ledger.reserve(16), Err(w::Code::ResourceLimit)));
        assert_eq!(ledger.snapshot().0, WORK);
        // The error/control frame is preprovisioned independently of work credits.
        let f = framed(&w::WorkerOut::Hello(w::Hello {
            protocol: w::Protocol::V1,
            algorithm: w::Algorithm::D0C1,
            schema_sha256: w::schema_hash(),
            ir_major: 1,
            ir_minor_max: 3,
        }));
        assert!(f.len() < OUT_FRAME);
        drop(old);
        drop(held);
        let tracked = Tracked {
            value: vec![1u8; 128],
            credit: ledger.reserve(128).unwrap(),
        };
        let child = thread::spawn(move || {
            assert_eq!(tracked.credit.bytes, 128);
            drop(tracked);
        });
        child.join().unwrap();
        assert_eq!(ledger.snapshot().0, 0);
        assert_eq!(ledger.snapshot().1, WORK);
        assert!(ledger.snapshot().2 > WORK);
        assert!(matches!(
            ledger.reserve(u64::MAX),
            Err(w::Code::ResourceLimit)
        ));
    }
    #[test]
    fn causal_missing_reservation_is_detected_from_owned_capacity() {
        let ledger = Ledger::new();
        let mut owned = Vec::<u64>::new();
        let bytes = 128 * std::mem::size_of::<u64>() as u64;
        let credit = ledger.reserve(bytes).unwrap();
        owned.try_reserve_exact(128).unwrap();
        let footprint = (owned.capacity() * std::mem::size_of::<u64>()) as u64;
        assert!(footprint <= ledger.snapshot().0);
        drop(owned);
        drop(credit);
        // Causal control: execute the same owned growth with the reserve omitted.
        let mut bypass = Vec::<u64>::new();
        bypass.try_reserve_exact(128).unwrap();
        assert!((bypass.capacity() * std::mem::size_of::<u64>()) as u64 > ledger.snapshot().0);
    }
    #[test]
    fn structural_preflight_bounds_depth_width_total_and_credit_before_serde() {
        let nested = format!("{}0{}", "[".repeat(32), "]".repeat(32));
        assert!(preflight(nested.as_bytes(), u64::MAX).is_ok());
        let deep = format!("[{nested}]");
        assert_eq!(
            preflight(deep.as_bytes(), u64::MAX).unwrap_err(),
            w::Code::ResourceLimit
        );
        let wide = format!(
            "[{}]",
            std::iter::repeat_n("0", 4097).collect::<Vec<_>>().join(",")
        );
        assert_eq!(
            preflight(wide.as_bytes(), u64::MAX).unwrap_err(),
            w::Code::ResourceLimit
        );
        let inner = format!(
            "[{}]",
            std::iter::repeat_n("0", 4096).collect::<Vec<_>>().join(",")
        );
        let dense = format!("[{}]", vec![inner; 16].join(","));
        assert!(dense.len() < w::MAX_FRAME);
        assert_eq!(
            preflight(dense.as_bytes(), u64::MAX).unwrap_err(),
            w::Code::ResourceLimit
        );
        let escape = br#"{"a":"\u0000\u00e9\\\"","b":[1,true,null]}"#;
        let shape = preflight(escape, WORK).unwrap();
        assert!(shape.typed(escape.len()).unwrap() > escape.len() as u64);
        assert!(
            shape.canonical().unwrap()
                >= w::canonical(&serde_json::from_slice::<serde_json::Value>(escape).unwrap())
                    .unwrap()
                    .len() as u64
        );
        assert_eq!(preflight(escape, 1).unwrap_err(), w::Code::ResourceLimit);
    }
}
