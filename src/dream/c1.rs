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
fn observed_interval(a: &Accounting, start: u64, end: u64) -> u64 {
    let used = interval(start, end);
    if used == u64::MAX {
        a.clock_failed.store(true, Ordering::Release);
    }
    used
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
    let native_cpu = observed_interval(a, call_start, call_end);
    let native_wall = match wall_end.checked_duration_since(wall_start) {
        Some(d) => d.as_nanos().min(u64::MAX as u128) as u64,
        None => {
            a.clock_failed.store(true, Ordering::Release);
            u64::MAX
        }
    };
    let inclusive = observed_interval(a, span_start, span_end);
    // Separate measured application subintervals; inclusive CPU stays RAW.
    let app = observed_interval(a, span_start, call_start)
        .saturating_add(observed_interval(a, call_end, span_end));
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
        self.part_sample(index, cpu_ns());
    }
    fn part_sample(&mut self, index: usize, end: u64) {
        self.accounting.control_parts[index].fetch_max(
            observed_interval(self.accounting, self.part_at, end),
            Ordering::AcqRel,
        );
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
fn read_early_refusal(purpose: bootstrap::Purpose, reason: &[u8]) -> i32 {
    if purpose == bootstrap::Purpose::ReadPrepare {
        // Fixed source-free failure evidence, without allocating argv, locks,
        // a formatter, or a replacement owner after the failed gate.
        let mut line = [0u8; 80];
        let prefix = b"dream-read early_refusal=";
        line[..prefix.len()].copy_from_slice(prefix);
        if reason.len() < line.len() - prefix.len() {
            line[prefix.len()..prefix.len() + reason.len()].copy_from_slice(reason);
            let end = prefix.len() + reason.len();
            line[end] = b'\n';
            let _actual_write = unsafe { libc::write(2, line.as_ptr().cast(), end + 1) };
        }
    }
    2
}
/// Darwin exposes borrowed argv/env pointers, so recognition allocates no argv Vec.
#[inline(never)]
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
    if arg(1, 5).ok() != Some(b"dream".as_slice()) {
        return None;
    }
    let purpose = match arg(2, 12).ok() {
        Some(b"inspect-c1") => bootstrap::Purpose::InspectC1,
        Some(b"prepare-read") => bootstrap::Purpose::ReadPrepare,
        _ => return None,
    };
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (prelude_sample, entry_sample, purpose);
        return Some(2);
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStringExt;
        let prelude_at = match prelude_sample {
            Ok(n) => n,
            Err(_) => {
                read_early_refusal(purpose, b"clock");
                return Some(1);
            }
        };
        let entry_ns = match entry_sample {
            Ok(n) => n,
            Err(_) => {
                read_early_refusal(purpose, b"clock");
                return Some(1);
            }
        };
        if argc != 5 || arg(3, 9).ok() != Some(b"--job-dir".as_slice()) {
            return Some(read_early_refusal(purpose, b"arguments"));
        }
        let raw = match arg(4, 4096) {
            Ok(raw) if raw.first() == Some(&b'/') => raw,
            _ => return Some(read_early_refusal(purpose, b"path")),
        };
        let basename = raw.rsplit(|c| *c == b'/').next().unwrap();
        let t = match unsafe { bootstrap::environment(bootstrap::ENV.as_bytes()) }
            .ok()
            .flatten()
            .and_then(|b| bootstrap::token(b).ok())
        {
            Some(t) => t,
            None => return Some(read_early_refusal(purpose, b"token")),
        };
        let mut gate =
            match unsafe { bootstrap::Gate::wait(t, purpose, basename, entry_ns, prelude_at) } {
                Ok(gate) => gate,
                Err(error) => {
                    let reason = match error {
                        bootstrap::Error::Format => b"format".as_slice(),
                        bootstrap::Error::Identity => b"identity".as_slice(),
                        bootstrap::Error::Clock => b"clock".as_slice(),
                        bootstrap::Error::Expired => b"expired".as_slice(),
                        bootstrap::Error::Pipe => b"pipe".as_slice(),
                        bootstrap::Error::Revoked => b"revoked".as_slice(),
                        bootstrap::Error::Limit => b"limit".as_slice(),
                    };
                    return Some(read_early_refusal(purpose, reason));
                }
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
        let code = match purpose {
            bootstrap::Purpose::InspectC1 => run::<C1Job>(path, maximum.clone(), gate),
            bootstrap::Purpose::ReadPrepare => {
                run::<super::read::ReadJob>(path, maximum.clone(), gate)
            }
        };
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
            interval(start, end)
        };
        maximum.fetch_max(used, Ordering::AcqRel);
        poll
    })
    .await;
    (result, maximum.load(Ordering::Acquire))
}
pub(crate) fn cpu_ns() -> u64 {
    bootstrap::prelude_clock().unwrap_or(u64::MAX)
}
#[cfg(target_os = "macos")]
pub(crate) fn heaps() -> (u64, u64) {
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
pub(crate) fn heaps() -> (u64, u64) {
    // No unmeasured heap certification on a platform without this allocator probe.
    (u64::MAX, u64::MAX)
}
fn heap() -> u64 {
    heaps().1
}
pub(crate) fn rss() -> u64 {
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
pub(crate) struct Control {
    pub(crate) cancel: AtomicU8,
    pub(crate) foreground: AtomicBool,
    pub(crate) deadline: Instant,
    pub(crate) reactor: Arc<AtomicU64>,
    pub(crate) memory: Arc<storage::Ledger>,
    accounting: Arc<Accounting>,
    publication: PublicationCustody,
    cleanup_uncertain: AtomicBool,
    pub(crate) helper: std::sync::Mutex<Option<HelperGroup>>,
    pub(crate) cleanup_cpu: AtomicU64,
    pub(crate) cleanup_max: AtomicU64,
    pub(crate) cleanup_overruns: AtomicU64,
}
impl Control {
    /// Q2 uses the same admitted owner debt ledger; it never installs a second
    /// ordinary-work scheduler or a native kind in the inspect-c1 profile.
    pub(crate) fn read_pay(&self) {
        self.accounting.pay();
    }
    pub(crate) fn observe_interval(&self, start: u64, end: u64) -> u64 {
        observed_interval(&self.accounting, start, end)
    }
    #[cfg(test)]
    pub(crate) fn fixture(deadline: Instant) -> Self {
        Self {
            cancel: AtomicU8::new(0),
            foreground: AtomicBool::new(false),
            deadline,
            reactor: Arc::new(AtomicU64::new(0)),
            memory: storage::Ledger::new(),
            accounting: Arc::new(Accounting::new()),
            publication: PublicationCustody::default(),
            cleanup_uncertain: AtomicBool::new(false),
            cleanup_cpu: AtomicU64::new(0),
            cleanup_max: AtomicU64::new(0),
            cleanup_overruns: AtomicU64::new(0),
            helper: std::sync::Mutex::new(None),
        }
    }
    pub(crate) fn error(&self) -> Option<w::Code> {
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
            5 => Some(w::Code::SourceUnavailable),
            _ if Instant::now() >= self.deadline => Some(w::Code::Deadline),
            _ => None,
        }
    }
}
/// One explicitly owned proof-helper group; never used by inspect-c1.
/// Its leader stays unreaped until signaling custody is retired, preventing ABA.
pub(crate) struct HelperGroup {
    pub(crate) pgid: i32,
    pub(crate) running: bool,
    pub(crate) retiring: bool,
    pub(crate) changed: Instant,
    pub(crate) ready: bool,
    creation_settled: bool,
    admission_failed: bool,
    #[cfg(test)]
    admission_samples: u64,
    #[cfg(test)]
    fail_stop_verdict: bool,
    pub(crate) initial_stop_pending: bool,
    pub(crate) confirmed_stopped: bool,
    pub(crate) identity: super::resolve::HelperIdentity,
    stop_requested: bool,
    pub(crate) debt_until: Instant,
    pub(crate) lane_cpu: u64,
    members: [HelperMember; 2],
    retired_members: [RetiredMember; 8],
    stopped_roster: [i32; 8],
    grant: HelperGrantLedger,
    corroboration_pending: bool,
    post_commit_pending: bool,
    post_cpu: u64,
    post_late_cpu: u64,
    accounted_helper_cpu: u64,
    rolled_user_ticks: u64,
    rolled_system_ticks: u64,
    grant_commit_pending: bool,
    pipe_open: bool,
    pipe_active: bool,
    corroboration_revision: u64,
    observed_post_revision: u64,
    post_proof_pending: bool,
    post_late_known: bool,
    settled_cpu: Option<u64>,
    usage_reconciled: bool,
    accounting_settled: bool,
    last_clock_ns: Option<u64>,
    pub(crate) metrics: HelperMetrics,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HelperAuditToken {
    value: [u32; 8],
}
#[repr(C)]
struct HelperTimebase {
    numer: u32,
    denom: u32,
}
#[derive(Clone, Copy, Default)]
struct RetiredMember {
    pid: i32,
    birth: u64,
    token: HelperAuditToken,
    current: bool,
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    static mach_task_self_: libc::mach_port_t;
    #[link_name = "mach_timebase_info"]
    fn helper_timebase_info(info: *mut HelperTimebase) -> i32;
    fn task_name_for_pid(task: libc::mach_port_t, pid: i32, name: *mut libc::mach_port_t) -> i32;
    fn proc_signal_with_audittoken(token: *const HelperAuditToken, signal: i32) -> i32;
    fn proc_pidpath_audittoken(
        token: *const HelperAuditToken,
        buffer: *mut libc::c_void,
        len: u32,
    ) -> i32;
    fn mach_port_deallocate(task: libc::mach_port_t, name: libc::mach_port_t) -> i32;
}
#[cfg(target_os = "macos")]
fn helper_token_exited(token: &HelperAuditToken) -> Result<bool, i32> {
    let mut path = [0u8; 4096];
    let result =
        unsafe { proc_pidpath_audittoken(token, path.as_mut_ptr().cast(), path.len() as u32) };
    if result > 0 {
        Ok(false)
    } else {
        match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => Ok(true),
            Some(code) => Err(code),
            None => Err(libc::EIO),
        }
    }
}
#[cfg(target_os = "macos")]
fn helper_ticks_ns(ticks: u64) -> Result<u64, ()> {
    let mut timebase = HelperTimebase { numer: 0, denom: 0 };
    if unsafe { helper_timebase_info(&mut timebase) } != 0
        || timebase.numer == 0
        || timebase.denom == 0
    {
        return Err(());
    }
    let ns = (u128::from(ticks) * u128::from(timebase.numer) + u128::from(timebase.denom) - 1)
        / u128::from(timebase.denom);
    u64::try_from(ns).map_err(|_| ())
}
#[cfg(target_os = "macos")]
fn helper_token_signal(token: &HelperAuditToken, signal: i32) -> Result<(), i32> {
    let result = unsafe { proc_signal_with_audittoken(token, signal) };
    match result {
        0 => Ok(()),
        code if code > 0 => Err(code),
        _ => Err(std::io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(libc::EIO)),
    }
}
#[cfg(target_os = "macos")]
fn helper_audit_member(pid: i32, parent: i32) -> Result<RetiredMember, ()> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
    if unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size,
        )
    } != size
    {
        return Err(());
    }
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid as u32
        || info.pbi_ppid != parent as u32
        || info.pbi_uid != unsafe { libc::geteuid() }
    {
        return Err(());
    }
    let birth = info
        .pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|s| s.checked_add(info.pbi_start_tvusec))
        .ok_or(())?;
    let mut task = 0;
    if unsafe { task_name_for_pid(mach_task_self_, pid, &mut task) } != 0 {
        return Err(());
    }
    let mut token = HelperAuditToken::default();
    let mut count = 8;
    let result = unsafe {
        libc::task_info(
            task,
            15,
            (&mut token as *mut HelperAuditToken).cast(),
            &mut count,
        )
    };
    unsafe {
        mach_port_deallocate(mach_task_self_, task);
    }
    if result != 0 || count != 8 || token.value[5] != pid as u32 {
        return Err(());
    }
    let mut after = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
    if unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            after.as_mut_ptr().cast(),
            size,
        )
    } != size
    {
        return Err(());
    }
    let after = unsafe { after.assume_init() };
    if after.pbi_pid != info.pbi_pid
        || after.pbi_ppid != info.pbi_ppid
        || after.pbi_start_tvsec != info.pbi_start_tvsec
        || after.pbi_start_tvusec != info.pbi_start_tvusec
    {
        return Err(());
    }
    Ok(RetiredMember {
        pid,
        birth,
        token,
        current: true,
    })
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum HelperImagePhase {
    #[default]
    Unobserved,
    ParentClone,
    Descendant,
    Exited,
}
#[derive(Clone, Copy, Default)]
struct HelperSlotWindow {
    pid: i32,
    birth: u64,
    start: u64,
    version: u32,
    ack_ns: Option<u64>,
    through_closure: bool,
}
#[derive(Clone, Copy, Default)]
struct HelperGrantLedger {
    opened: bool,
    t0_ns: u64,
    close_ns: Option<u64>,
    reserved: u8,
    slots: [HelperSlotWindow; 2],
    actual_baseline: u64,
    rolled_baseline: u64,
    charged: u64,
    charged_once: bool,
    controller_baseline: u64,
    pipe_baseline: u64,
    reported_w: u64,
    reported_a: u64,
    reported_c: u64,
    closed: bool,
}
/// One bounded receipt: every raw field belongs to the same worst-Q grant.
#[derive(Clone, Copy, Default)]
pub(crate) struct HelperGrantReceipt {
    pub(crate) sequence: u16,
    pub(crate) pid: i32,
    pub(crate) pgid: i32,
    pub(crate) birth: u64,
    pub(crate) reserved: u8,
    pub(crate) t0_ns: u64,
    pub(crate) end_ns: [u64; 2],
    pub(crate) close_ns: u64,
    pub(crate) member_pid: [i32; 2],
    pub(crate) member_birth: [u64; 2],
    pub(crate) member_start: [u64; 2],
    pub(crate) member_version: [u32; 2],
    pub(crate) w_ns: u64,
    pub(crate) a_ns: u64,
    pub(crate) c_ns: u64,
    pub(crate) q_ns: u64,
    pub(crate) closed: bool,
    pub(crate) controller_baseline_ns: u64,
    pub(crate) controller_end_ns: u64,
    pub(crate) pipe_baseline_ns: u64,
    pub(crate) pipe_end_ns: u64,
}
#[derive(Clone, Copy, Default)]
struct HelperMember {
    pid: i32,
    birth: u64,
    cpu: u64,
    rss_peak: u64,
    start_abstime: u64,
    image_phase: HelperImagePhase,
    preexec_observations: u64,
    audit_version: u32,
    exec_stop_version: u32,
    audit_token: HelperAuditToken,
    user_ticks: u64,
    system_ticks: u64,
    exit_abstime: u64,
}
/// Fixed-size observations, distinct from allocator coverage. Process RSS and
/// Rust heap are deliberately named separately; neither is whole-tree heap.
#[derive(Clone, Copy, Default)]
pub(crate) struct HelperStopReceipt {
    pub(crate) sequence: u16,
    pub(crate) pgid: i32,
    pub(crate) attempt: u64,
    pub(crate) result: i32,
    pub(crate) rawerrno: i32,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct HelperMetrics {
    pub(crate) grants: u64,
    pub(crate) charged_ns: u64,
    pub(crate) max_charge_ns: u64,
    pub(crate) debt_ns: u64,
    pub(crate) controller_cpu_ns: u64,
    pub(crate) pipe_cpu_ns: u64,
    pub(crate) observed_direct_cpu_ns: u64,
    pub(crate) observed_descendant_cpu_ns: u64,
    pub(crate) rolled_descendant_cpu_ns: u64,
    pub(crate) group_rss_peak: u64,
    pub(crate) separate_rss_sum: u64,
    pub(crate) mixed_heap_rss_peak: u64,
    pub(crate) descendants: u64,
    pub(crate) verified_descendants: u64,
    pub(crate) initial_kernel_cpu_ns: u64,
    pub(crate) stop_attempts: u64,
    pub(crate) stop_failures: u64,
    pub(crate) last_stop: HelperStopReceipt,
    pub(crate) first_failed_stop: HelperStopReceipt,
    pub(crate) first_failure_line: u32,
    pub(crate) first_failure_sample_stage: u8,
    sample_stage: u8,
    pub(crate) coverage_known: bool,
    pub(crate) worst_grant: HelperGrantReceipt,
    pub(crate) first_failed_pending: HelperGrantReceipt,
    pub(crate) window_ns: u64,
    pub(crate) active_cpu_ns: u64,
    pub(crate) active_control_ns: u64,
    pub(crate) roster_closures: u64,
    pub(crate) reopened_slots: u64,
    pub(crate) unknown_windows: u64,
    pub(crate) clock_failures: u64,
    pub(crate) post_units: u64,
    pub(crate) post_cpu_ns: u64,
    pub(crate) post_max_cpu_ns: u64,
    pub(crate) late_kernel_cpu_ns: u64,
    pub(crate) late_kernel_observations: u64,
    pub(crate) reap_user_ns: u64,
    pub(crate) reap_system_ns: u64,
    pub(crate) reap_user_upper_ns: u64,
    pub(crate) reap_system_upper_ns: u64,
    pub(crate) reap_quantization_ns: u64,
    pub(crate) reconciliation_unproven: u64,
    pub(crate) unexplained_cpu_ns: u64,
    pub(crate) reap_ri_user_ticks: u64,
    pub(crate) reap_ri_system_ticks: u64,
    pub(crate) reap_timebase_numer: u32,
    pub(crate) reap_timebase_denom: u32,
}
impl HelperGroup {
    fn charge_helper_debt(&mut self, control: &Control, cost: u64) {
        // Preserve unknown raw cost as MAX, but use the canonical finite
        // mandatory-settlement debt. MAX nanoseconds is not a known duration.
        let debt = control.helper_extend_debt(cost);
        self.metrics.debt_ns = self.metrics.debt_ns.saturating_add(debt);
        if cost == u64::MAX || cost.checked_mul(9).is_none() {
            self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
            self.metrics.coverage_known = false;
        }
        if debt != 0 {
            let base = self.debt_until.max(Instant::now());
            self.debt_until = base
                .checked_add(Duration::from_nanos(debt))
                .unwrap_or_else(|| {
                    control
                        .accounting
                        .clock_failed
                        .store(true, Ordering::Release);
                    self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
                    self.metrics.coverage_known = false;
                    base // Preserve every previously known outstanding deadline.
                });
        }
    }
    pub(crate) fn needs_active_poll(&self) -> bool {
        self.running && !self.retiring && !self.admission_failed
    }
    fn signal_group_stop(&mut self) -> HelperStopReceipt {
        let result = unsafe { libc::kill(-self.pgid, libc::SIGSTOP) };
        // Retain the exact native outcome before any completion observation or
        // verdict. Bounded scalar receipts also survive later retirement.
        let rawerrno = if result == 0 {
            0
        } else {
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
        };
        self.metrics.stop_attempts = self.metrics.stop_attempts.saturating_add(1);
        let receipt = HelperStopReceipt {
            sequence: self.identity.sequence,
            pgid: self.pgid,
            attempt: self.metrics.stop_attempts,
            result,
            rawerrno,
        };
        self.metrics.last_stop = receipt;
        if result != 0 {
            self.metrics.stop_failures = self.metrics.stop_failures.saturating_add(1);
            if self.metrics.first_failed_stop.attempt == 0 {
                self.metrics.first_failed_stop = receipt;
            }
        }
        receipt
    }
    fn request_group_stop(&mut self) -> Result<(), ()> {
        if self.stop_requested {
            return Err(()); // A consumed operation is never silently repeated.
        }
        self.stop_requested = true; // retain the attempted effect BEFORE its call/verdict
        let receipt = self.signal_group_stop();
        #[cfg(test)]
        if std::mem::take(&mut self.fail_stop_verdict) {
            // Fault only the verdict; the receipt still contains the actual
            // kernel result/errno. This cannot manufacture a failed signal.
            return Err(());
        }
        // Capture the actual result before observing any completion. ESRCH is
        // a permissible delivery race only when our still-unreaped leader has
        // a positive WNOWAIT exit. It never closes a slot or proves a roster;
        // the normal exact exit/usage/continuity path still has to do that.
        if receipt.result == 0
            || (receipt.rawerrno == libc::ESRCH
                && super::resolve::helper_exited(self.pgid) == Ok(true))
        {
            Ok(())
        } else {
            Err(())
        }
    }
    #[cfg(target_os = "macos")]
    fn stop_observation_due(&self) -> bool {
        // This cheap hint never acknowledges a slot, closes a roster, changes
        // coverage, or admits CONT. Avoid repeating the full proof while a
        // positively owned member is still completing the kernel STOP tail.
        // Unexpected/unknown identities go to the full custody-preserving
        // validator immediately. All hint CPU and elapsed time remain in Q.
        for member in self.members.iter().filter(|member| member.pid > 0) {
            if member.pid == self.pgid && super::resolve::helper_exited(member.pid) == Ok(true) {
                continue;
            }
            let mut info = std::mem::MaybeUninit::<libc::proc_taskallinfo>::uninit();
            let size = std::mem::size_of::<libc::proc_taskallinfo>() as i32;
            if unsafe {
                libc::proc_pidinfo(
                    member.pid,
                    libc::PROC_PIDTASKALLINFO,
                    0,
                    info.as_mut_ptr().cast(),
                    size,
                )
            } != size
            {
                return true;
            }
            let info = unsafe { info.assume_init() };
            let birth = info
                .pbsd
                .pbi_start_tvsec
                .checked_mul(1_000_000)
                .and_then(|value| value.checked_add(info.pbsd.pbi_start_tvusec));
            if birth != Some(member.birth)
                || info.pbsd.pbi_pgid != self.pgid as u32
                || info.ptinfo.pti_threadnum != 1
            {
                return true;
            }
            if info.pbsd.pbi_status != 4 || info.ptinfo.pti_numrunning != 0 {
                return false;
            }
        }
        true
    }
    #[cfg(not(target_os = "macos"))]
    fn stop_observation_due(&self) -> bool {
        true
    }
    #[cfg(test)]
    pub(crate) fn fixture_descendant_identity(&self) -> Option<(i32, u64, u64, u32)> {
        let member = self.members[1];
        (member.image_phase == HelperImagePhase::Descendant
            && member.pid > 0
            && member.birth > 0
            && member.start_abstime > 0
            && member.audit_version > 0)
            .then_some((
                member.pid,
                member.birth,
                member.start_abstime,
                member.audit_version,
            ))
    }
    pub(crate) fn settle_creation(&mut self, control: &Control) -> Result<(), ()> {
        if self.creation_settled
            || self.ready
            || self.retiring
            || control.error().is_some()
            || control.accounting.now() < control.accounting.rest_until.load(Ordering::Acquire)
            || Instant::now() < self.debt_until
        {
            return Err(());
        }
        self.creation_settled = true;
        Ok(())
    }
    fn ledger_clock(&mut self) -> Result<u64, ()> {
        match bootstrap::monotonic_ns() {
            Ok(now) if self.last_clock_ns.is_none_or(|previous| now >= previous) => {
                self.last_clock_ns = Some(now);
                Ok(now)
            }
            Ok(_) => {
                self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
                self.metrics.coverage_known = false;
                Err(())
            }
            Err(_) => {
                self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
                self.metrics.coverage_known = false;
                Err(())
            }
        }
    }
    fn actual_cpu(&self) -> Result<u64, ()> {
        if let Some(settled) = self.settled_cpu {
            return Ok(settled);
        }
        self.members[0]
            .cpu
            .checked_add(
                self.members[1]
                    .cpu
                    .max(self.metrics.rolled_descendant_cpu_ns),
            )
            .ok_or(())
    }
    pub(crate) fn pending_closure(&self) -> bool {
        !self.accounting_settled
            && (self.running
                || self.grant_commit_pending
                || (self.grant.opened && !self.grant.closed))
    }
    pub(crate) fn pending_corroboration(&self) -> bool {
        self.corroboration_pending || self.post_commit_pending
    }
    pub(crate) fn closure_observed(&self) -> bool {
        !self.running && self.confirmed_stopped && (!self.grant.opened || self.grant.closed)
    }
    pub(crate) fn retirement_settlement_observed(&self) -> bool {
        // This is set only after positive current-incarnation exit checks in
        // usage reconciliation or quarantine. Exited tasks cannot resume;
        // unlike a stopped seal, this fact needs no duplicate sampling unit.
        self.retiring && self.accounting_settled
    }
    pub(crate) fn begin_pipe(&mut self) -> Result<(), ()> {
        if self.pipe_open {
            return Err(());
        }
        self.pipe_open = true;
        self.pipe_active = self.pending_closure();
        Ok(())
    }
    pub(crate) fn pipe_is_open(&self) -> bool {
        self.pipe_open
    }
    pub(crate) fn settle_quarantine(&mut self, control: &Control) -> bool {
        let at = cpu_ns();
        let gone = self.retired_members_exited();
        let cpu = control.observe_interval(at, cpu_ns());
        if self.pending_closure() {
            self.lane_cpu = self.lane_cpu.saturating_add(cpu);
            if gone {
                self.finish_grant(control, 0);
            }
        }
        if gone {
            self.accounting_settled = true;
            if !self.post_proof_pending {
                self.corroboration_pending = false;
            }
        }
        gone
    }
    pub(crate) fn account_pipe(
        &mut self,
        control: &Control,
        cpu_ns: u64,
        wall_ns: u64,
    ) -> Result<(), ()> {
        if !self.pipe_open {
            return Err(());
        }
        self.pipe_open = false;
        self.metrics.pipe_cpu_ns = self.metrics.pipe_cpu_ns.saturating_add(cpu_ns);
        if cpu_ns == u64::MAX || wall_ns == u64::MAX {
            self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
            if self.pipe_active || self.pending_closure() {
                self.lane_cpu = u64::MAX;
            } else {
                self.post_cpu = u64::MAX;
            }
            self.reject(control, 5);
            return Err(());
        }
        if self.pipe_active || self.pending_closure() {
            self.lane_cpu = self.lane_cpu.checked_add(cpu_ns).ok_or(())?;
        } else {
            // The read owner records/pays this actual ordinary unit through
            // read_charge once. Only the attribution is recorded here.
            self.metrics.post_units = self.metrics.post_units.saturating_add(1);
            self.metrics.post_cpu_ns = self.metrics.post_cpu_ns.saturating_add(cpu_ns);
            self.metrics.post_max_cpu_ns = self.metrics.post_max_cpu_ns.max(cpu_ns);
            // Pipe-only work does not invalidate an established stopped seal.
            // Its actual ordinary CPU/debt is paid by the read owner;
            // initial/after-grant/late-CPU corroboration remains mandatory.
        }
        self.pipe_active = false;
        if cpu_ns > UNIT_NS {
            self.reject(control, 4);
            return Err(());
        }
        Ok(())
    }
    pub(crate) fn account_controller(&mut self, control: &Control, cpu: u64) -> Result<(), ()> {
        self.metrics.controller_cpu_ns = self.metrics.controller_cpu_ns.saturating_add(cpu);
        if cpu == u64::MAX {
            self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
            if self.pending_closure() {
                self.lane_cpu = u64::MAX;
            } else if self.post_commit_pending {
                self.post_cpu = u64::MAX;
            }
            self.reject(control, 5);
            return Err(());
        }
        if self.pending_closure() {
            self.lane_cpu = self.lane_cpu.checked_add(cpu).ok_or(())?;
        } else if self.post_commit_pending {
            self.post_cpu = self.post_cpu.checked_add(cpu).ok_or(())?;
        }
        if cpu > UNIT_NS {
            self.reject(control, 4);
            return Err(());
        }
        Ok(())
    }
    fn open_grant(&mut self) -> Result<(), ()> {
        let baseline = self.actual_cpu()?;
        let rolled = self
            .rolled_user_ticks
            .checked_add(self.rolled_system_ticks)
            .ok_or(())?;
        let mut ledger = HelperGrantLedger {
            opened: true,
            reserved: self.identity.expected_members,
            actual_baseline: baseline,
            rolled_baseline: rolled,
            controller_baseline: self.metrics.controller_cpu_ns,
            pipe_baseline: self.metrics.pipe_cpu_ns,
            ..HelperGrantLedger::default()
        };
        for index in 0..usize::from(ledger.reserved) {
            let member = self.members[index];
            let unknown = member.pid == 0 || member.image_phase == HelperImagePhase::Exited;
            ledger.slots[index] = HelperSlotWindow {
                pid: member.pid,
                birth: member.birth,
                start: member.start_abstime,
                version: member.audit_version,
                ack_ns: None,
                through_closure: unknown,
            };
            self.metrics.unknown_windows = self
                .metrics
                .unknown_windows
                .saturating_add(u64::from(unknown));
        }
        // This final checked clock precedes the actual CONT, including its
        // complete delivery latency in every reserved slot's open interval.
        ledger.t0_ns = self.ledger_clock()?;
        self.grant = ledger;
        self.lane_cpu = 0;
        if self.pipe_open {
            self.pipe_active = true;
        }
        Ok(())
    }
    fn note_incarnation(&mut self, index: usize, member: HelperMember) {
        if !self.grant.opened {
            return;
        }
        let slot = &mut self.grant.slots[index];
        let changed = slot.pid != 0
            && (slot.pid != member.pid
                || slot.birth != member.birth
                || slot.start != member.start_abstime
                || slot.version != member.audit_version);
        if changed {
            slot.ack_ns = None;
            slot.through_closure = true;
            self.grant.closed = false;
            self.grant.close_ns = None;
            self.metrics.reopened_slots = self.metrics.reopened_slots.saturating_add(1);
        }
        slot.pid = member.pid;
        slot.birth = member.birth;
        slot.start = member.start_abstime;
        slot.version = member.audit_version;
    }
    fn invalidate_slot(&mut self, index: usize) {
        self.metrics.coverage_known = false;
        self.confirmed_stopped = false;
        if self.grant.opened {
            let reopened = self.grant.slots[index].ack_ns.is_some() || self.grant.closed;
            self.grant.slots[index].ack_ns = None;
            self.grant.slots[index].through_closure = true;
            self.grant.closed = false;
            self.grant.close_ns = None;
            self.metrics.reopened_slots = self
                .metrics
                .reopened_slots
                .saturating_add(u64::from(reopened));
        }
    }
    fn acknowledge_slot(&mut self, index: usize) -> Result<(), ()> {
        if !self.grant.opened {
            return Ok(());
        }
        let member = self.members[index];
        self.note_incarnation(index, member);
        let end = self.ledger_clock()?;
        if end < self.grant.t0_ns {
            self.metrics.coverage_known = false;
            return Err(());
        }
        self.grant.slots[index].ack_ns.get_or_insert(end);
        Ok(())
    }
    fn close_roster(&mut self) -> Result<(), ()> {
        if !self.grant.opened {
            return Ok(());
        }
        let close = self.ledger_clock()?;
        if close < self.grant.t0_ns {
            self.metrics.coverage_known = false;
            return Err(());
        }
        for index in 0..usize::from(self.grant.reserved) {
            if self.grant.slots[index].ack_ns.is_none() {
                let empty = index == 1
                    && (self.members[1].pid == 0
                        || self.members[1].image_phase == HelperImagePhase::Exited)
                    && self.rolled_user_ticks.checked_add(self.rolled_system_ticks)
                        == Some(self.grant.rolled_baseline);
                if !empty {
                    self.metrics.coverage_known = false;
                    return Err(());
                }
                self.grant.slots[index].ack_ns = Some(close);
                self.grant.slots[index].through_closure = true;
            }
        }
        self.grant.close_ns = Some(close);
        self.grant.closed = true;
        self.metrics.roster_closures = self.metrics.roster_closures.saturating_add(1);
        Ok(())
    }
    fn pending_charge_lower_bound(&mut self) -> Result<u64, ()> {
        if !self.grant.opened {
            return Ok(0);
        }
        let now = self.ledger_clock()?;
        let mut window = 0u64;
        for slot in &self.grant.slots[..usize::from(self.grant.reserved)] {
            let end = if self.grant.closed && slot.through_closure {
                self.grant.close_ns.ok_or(())?
            } else if slot.through_closure {
                now
            } else {
                slot.ack_ns.unwrap_or(now)
            };
            window = window
                .checked_add(end.checked_sub(self.grant.t0_ns).ok_or(())?)
                .ok_or(())?;
        }
        let actual = self
            .actual_cpu()?
            .checked_sub(self.grant.actual_baseline)
            .ok_or(())?;
        let charge = window
            .max(actual)
            .checked_add(self.grant_control_cpu()?)
            .ok_or(())?;
        if charge > UNIT_NS && self.metrics.first_failed_pending.sequence == 0 {
            let mut receipt = HelperGrantReceipt {
                sequence: self.identity.sequence,
                pid: self.identity.pid,
                pgid: self.pgid,
                birth: self.identity.start_identity,
                reserved: self.grant.reserved,
                t0_ns: self.grant.t0_ns,
                close_ns: self.grant.close_ns.unwrap_or(now),
                w_ns: window,
                a_ns: actual,
                c_ns: self.grant_control_cpu()?,
                q_ns: charge,
                closed: self.grant.closed,
                controller_baseline_ns: self.grant.controller_baseline,
                controller_end_ns: self.metrics.controller_cpu_ns,
                pipe_baseline_ns: self.grant.pipe_baseline,
                pipe_end_ns: self.metrics.pipe_cpu_ns,
                ..HelperGrantReceipt::default()
            };
            for index in 0..usize::from(self.grant.reserved) {
                let slot = self.grant.slots[index];
                receipt.end_ns[index] = if self.grant.closed && slot.through_closure {
                    self.grant.close_ns.ok_or(())?
                } else if slot.through_closure {
                    now
                } else {
                    slot.ack_ns.unwrap_or(now)
                };
                receipt.member_pid[index] = slot.pid;
                receipt.member_birth[index] = slot.birth;
                receipt.member_start[index] = slot.start;
                receipt.member_version[index] = slot.version;
            }
            self.metrics.first_failed_pending = receipt;
        }
        Ok(charge)
    }
    fn grant_control_cpu(&self) -> Result<u64, ()> {
        if self.metrics.controller_cpu_ns == u64::MAX || self.metrics.pipe_cpu_ns == u64::MAX {
            return Err(());
        }
        self.metrics
            .controller_cpu_ns
            .checked_sub(self.grant.controller_baseline)
            .and_then(|cpu| {
                self.metrics
                    .pipe_cpu_ns
                    .checked_sub(self.grant.pipe_baseline)
                    .and_then(|pipe| cpu.checked_add(pipe))
            })
            .ok_or(())
    }
    fn actual_user_ticks(&self) -> Result<u64, ()> {
        self.members[0]
            .user_ticks
            .checked_add(self.members[1].user_ticks.max(self.rolled_user_ticks))
            .ok_or(())
    }
    fn actual_system_ticks(&self) -> Result<u64, ()> {
        self.members[0]
            .system_ticks
            .checked_add(self.members[1].system_ticks.max(self.rolled_system_ticks))
            .ok_or(())
    }
    /// Raw timeval components remain distinct. The explicit interval covers
    /// only separately truncated microsecond representation (own+one child),
    /// not the later exit/calcru cutoff. Every new system upper residual is
    /// charged in full; the exact-host kernel/tool basis remains a prerequisite.
    #[cfg(target_os = "macos")]
    pub(crate) fn reconcile_usage(
        &mut self,
        control: &Control,
        user_ns: u64,
        system_ns: u64,
        rss: u64,
    ) -> Result<(), ()> {
        if self.usage_reconciled {
            return Err(());
        }
        self.usage_reconciled = true;
        self.metrics.reap_user_ns = user_ns;
        self.metrics.reap_system_ns = system_ns;
        self.metrics.reap_quantization_ns = u64::from(self.identity.expected_members)
            .checked_mul(1000)
            .ok_or(())?;
        let width = self.metrics.reap_quantization_ns.checked_sub(1).ok_or(())?;
        let user_upper = user_ns.checked_add(width).ok_or(())?;
        let system_upper = system_ns.checked_add(width).ok_or(())?;
        self.metrics.reap_user_upper_ns = user_upper;
        self.metrics.reap_system_upper_ns = system_upper;
        let user_ticks = self.actual_user_ticks()?;
        let system_ticks = self.actual_system_ticks()?;
        self.metrics.reap_ri_user_ticks = user_ticks;
        self.metrics.reap_ri_system_ticks = system_ticks;
        let mut timebase = HelperTimebase { numer: 0, denom: 0 };
        if unsafe { helper_timebase_info(&mut timebase) } != 0
            || timebase.numer == 0
            || timebase.denom == 0
        {
            return Err(());
        }
        self.metrics.reap_timebase_numer = timebase.numer;
        self.metrics.reap_timebase_denom = timebase.denom;
        let user_before = helper_ticks_ns(user_ticks)?;
        let system_before = helper_ticks_ns(system_ticks)?;
        // A one-nanosecond enclosing RI representation interval is explicit;
        // raw RI ticks/timebase and timeval values are never epsilon-clamped.
        let user_lower = user_before.saturating_sub(1);
        let system_lower = system_before.saturating_sub(1);
        let user_stable = user_ns <= user_before && user_upper >= user_lower;
        let system_forward = system_upper >= system_lower;
        self.post_proof_pending = true;
        self.post_commit_pending = true;
        self.post_cpu = 0;
        self.post_late_known = user_stable && system_forward;
        self.post_late_cpu = if system_forward {
            let kernel = system_upper.checked_sub(system_lower).ok_or(())?;
            if user_stable {
                kernel
            } else {
                user_upper
                    .checked_sub(user_lower)
                    .and_then(|user| kernel.checked_add(user))
                    .unwrap_or(u64::MAX)
            }
        } else {
            u64::MAX
        };
        self.settled_cpu = Some(user_upper.checked_add(system_upper).ok_or(())?);
        if self.pending_closure() {
            self.finish_grant(control, 0);
            // Without a causal closed roster this is terminal accounting only;
            // the full uncertainty window remains in its failed receipt.
            self.post_late_cpu = 0;
            self.post_late_known = true;
        }
        self.accounting_settled = self.retired_members_exited();
        self.corroboration_pending = false;
        if rss > storage::HELPERS {
            self.reject(control, 4);
            return Err(());
        }
        if !self.post_late_known {
            self.metrics.reconciliation_unproven =
                self.metrics.reconciliation_unproven.saturating_add(1);
            self.metrics.coverage_known = false;
            self.reject(control, 5);
            return Err(());
        }
        self.control_verdict(control)
    }
    pub(crate) fn commit_post_observation(
        &mut self,
        control: &Control,
        cpu_ns: u64,
        wall_ns: u64,
    ) -> Result<(), ()> {
        if !self.post_proof_pending {
            return Err(());
        }
        self.post_cpu = cpu_ns;
        if wall_ns == u64::MAX || cpu_ns == u64::MAX {
            self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
            self.post_late_cpu = u64::MAX;
            self.post_late_known = false;
        }
        self.post_proof_pending = false;
        self.commit_stopped_unit(control, Some(cpu_ns));
        self.control_verdict(control)
    }
    fn control_verdict(&self, control: &Control) -> Result<(), ()> {
        if self.metrics.coverage_known && control.error().is_none() {
            Ok(())
        } else {
            Err(())
        }
    }
    #[cfg(target_os = "macos")]
    fn corroborate_stopped(&mut self, control: &Control) -> Result<(), ()> {
        if !self.confirmed_stopped || self.pending_closure() || self.pipe_open {
            return Err(());
        }
        let user = self.actual_user_ticks()?;
        self.observed_post_revision = self.corroboration_revision;
        self.post_cpu = 0;
        self.post_late_cpu = 0;
        self.post_late_known = true;
        self.post_commit_pending = true;
        for index in 0..2 {
            if !self.member_continuity(index)? {
                return Err(());
            }
        }
        self.recheck_stopped_roster()?;
        if self.pending_closure() || self.actual_user_ticks()? != user {
            return Err(());
        }
        // This allocator/RSS metadata does not establish the stopped roster.
        // Observe and charge it in this existing post-closure ordinary unit,
        // before further grants; retain the conservative group peak as a bound.
        let mixed = heaps()
            .0
            .checked_add(self.metrics.group_rss_peak)
            .ok_or(())?;
        self.metrics.mixed_heap_rss_peak = self.metrics.mixed_heap_rss_peak.max(mixed);
        if mixed > HEAP_MAX {
            self.reject(control, 4);
            return Err(());
        }
        let actual = self.actual_cpu()?;
        self.post_late_cpu = actual.checked_sub(self.accounted_helper_cpu).ok_or(())?;
        self.accounted_helper_cpu = actual;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn recheck_stopped_roster(&mut self) -> Result<(), ()> {
        self.recheck_stopped_roster_expected(None)
            .and_then(|same| if same { Ok(()) } else { Err(()) })
    }
    #[cfg(target_os = "macos")]
    fn recheck_stopped_roster_expected(&mut self, expected: Option<&[i32]>) -> Result<bool, ()> {
        let result = self.recheck_stopped_roster_inner(expected);
        if result.is_err() {
            for index in 0..usize::from(self.identity.expected_members) {
                self.invalidate_slot(index);
            }
        }
        result
    }
    #[cfg(target_os = "macos")]
    fn recheck_stopped_roster_inner(&mut self, expected: Option<&[i32]>) -> Result<bool, ()> {
        let mut pids = [0i32; 3];
        let count = unsafe {
            libc::proc_listpgrppids(
                self.pgid,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as i32,
            )
        };
        if count == 0 {
            return self.sample_exited_roster();
        }
        if count < 0 || count as usize > pids.len() {
            return Err(());
        }
        let count = count as usize;
        if count > usize::from(self.identity.expected_members) {
            self.capture_members()?;
            return Err(());
        }
        if let Some(expected) = expected
            && (count != expected.len() || pids[..count].iter().any(|pid| !expected.contains(pid)))
        {
            // This is the final post-continuity census. Retain and stop newly
            // discovered recipients before deciding that this grant is pending.
            self.capture_members()?;
            for member in self.retired_members.iter().filter(|member| {
                member.pid > 0 && member.current && !expected.contains(&member.pid)
            }) {
                if matches!(helper_token_signal(&member.token,libc::SIGSTOP),Err(code) if code!=libc::ESRCH)
                {
                    return Err(());
                }
            }
            return Ok(false);
        }
        if pids[..count]
            .iter()
            .any(|pid| !self.members.iter().any(|member| member.pid == *pid))
        {
            self.capture_members()?;
            return Err(());
        }
        for member in self.members.into_iter().filter(|member| member.pid > 0) {
            if !pids[..count].contains(&member.pid) {
                if member.image_phase != HelperImagePhase::Exited || member.exit_abstime == 0 {
                    return Err(());
                }
                continue;
            }
            if member.image_phase == HelperImagePhase::Exited {
                let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
                if unsafe { libc::proc_pid_rusage(member.pid, 1, usage.as_mut_ptr().cast()) } != 0 {
                    return Err(());
                }
                let usage = unsafe { usage.assume_init() };
                if usage.ri_proc_start_abstime != member.start_abstime
                    || usage.ri_proc_exit_abstime == 0
                {
                    return Err(());
                }
                continue;
            }
            let mut children = [0i32; 3];
            let found = unsafe {
                libc::proc_listchildpids(
                    member.pid,
                    children.as_mut_ptr().cast(),
                    std::mem::size_of_val(&children) as i32,
                )
            };
            if found < 0
                || found as usize >= children.len()
                || children[..found as usize]
                    .iter()
                    .any(|pid| !pids[..count].contains(pid))
            {
                self.capture_members()?;
                return Err(());
            }
        }
        Ok(true)
    }
    #[cfg(not(target_os = "macos"))]
    fn corroborate_stopped(&mut self, _: &Control) -> Result<(), ()> {
        Err(())
    }
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn reconcile_usage(
        &mut self,
        _: &Control,
        _: u64,
        _: u64,
        _: u64,
    ) -> Result<(), ()> {
        Err(())
    }
    fn commit_stopped_unit(&mut self, control: &Control, charged_cpu: Option<u64>) {
        let cost = self.post_cpu.saturating_add(self.post_late_cpu);
        self.metrics.post_units = self.metrics.post_units.saturating_add(1);
        self.metrics.post_cpu_ns = self.metrics.post_cpu_ns.saturating_add(self.post_cpu);
        self.metrics.post_max_cpu_ns = self.metrics.post_max_cpu_ns.max(cost);
        if self.post_late_known {
            self.metrics.late_kernel_cpu_ns = self
                .metrics
                .late_kernel_cpu_ns
                .saturating_add(self.post_late_cpu);
            self.metrics.late_kernel_observations = self
                .metrics
                .late_kernel_observations
                .saturating_add(u64::from(self.post_late_cpu != 0));
        } else {
            self.metrics.unexplained_cpu_ns = self
                .metrics
                .unexplained_cpu_ns
                .saturating_add(self.post_late_cpu);
        }
        // A proof-owned usage observation has already recorded its actual C
        // through read_charge. Charge the inclusive C+L floor once, extending
        // only its uncovered delta; controller-owned new units have no prior C.
        let debt_cost = match charged_cpu {
            Some(cpu) if cost != u64::MAX && cpu != u64::MAX => cost
                .max(UNIT_NS)
                .checked_sub(cpu.max(UNIT_NS))
                .unwrap_or(u64::MAX),
            Some(_) => u64::MAX,
            None => cost.max(UNIT_NS),
        };
        self.charge_helper_debt(control, debt_cost);
        self.post_commit_pending = false;
        self.corroboration_pending =
            !self.retiring && self.corroboration_revision != self.observed_post_revision;
        self.post_cpu = 0;
        self.post_late_cpu = 0;
        if cost > UNIT_NS {
            self.reject(control, 4);
        }
    }
    /// A later required observation closes continuity, rather than treating one
    /// TASKALLINFO struct as an atomic stop/exec/roster transaction.
    #[cfg(target_os = "macos")]
    fn member_continuity(&mut self, index: usize) -> Result<bool, ()> {
        let result = self.member_continuity_inner(index);
        if result.is_err() {
            self.invalidate_slot(index);
        }
        result
    }
    #[cfg(target_os = "macos")]
    fn member_continuity_inner(&mut self, index: usize) -> Result<bool, ()> {
        use std::os::unix::ffi::OsStrExt;
        let prior = self.members[index];
        if prior.pid == 0 {
            return Ok(true);
        }
        if prior.image_phase == HelperImagePhase::Exited
            && prior.exit_abstime != 0
            && helper_token_exited(&prior.audit_token) == Ok(true)
        {
            self.acknowledge_slot(index)?;
            return Ok(true);
        }
        let mut raw = std::mem::MaybeUninit::<libc::proc_taskallinfo>::uninit();
        let size = std::mem::size_of::<libc::proc_taskallinfo>() as i32;
        if unsafe {
            libc::proc_pidinfo(
                prior.pid,
                libc::PROC_PIDTASKALLINFO,
                0,
                raw.as_mut_ptr().cast(),
                size,
            )
        } != size
        {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err(());
            }
            self.exited_member_usage(prior.pid)?;
            self.members[index].image_phase = HelperImagePhase::Exited;
            self.acknowledge_slot(index)?;
            return Ok(true);
        }
        let raw = unsafe { raw.assume_init() };
        let info = raw.pbsd;
        let task = raw.ptinfo;
        let birth = info
            .pbi_start_tvsec
            .checked_mul(1_000_000)
            .and_then(|n| n.checked_add(info.pbi_start_tvusec))
            .ok_or(())?;
        let parent = if index == 0 {
            unsafe { libc::getpid() }
        } else {
            self.pgid
        };
        if info.pbi_pid != prior.pid as u32
            || info.pbi_pgid != self.pgid as u32
            || info.pbi_ppid != parent as u32
            || info.pbi_uid != unsafe { libc::geteuid() }
            || info.pbi_nice < 10
            || birth != prior.birth
        {
            return Err(());
        }
        let mut image = [0u8; 4096];
        let result = unsafe {
            proc_pidpath_audittoken(
                &prior.audit_token,
                image.as_mut_ptr().cast(),
                image.len() as u32,
            )
        };
        if result <= 0 {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err(());
            }
            let current = helper_audit_member(prior.pid, parent)?;
            self.retain_incarnation(current)?;
            let mut member = prior;
            member.audit_token = current.token;
            member.audit_version = current.token.value[7];
            if member.audit_version == prior.audit_version {
                return Err(());
            }
            helper_token_signal(&member.audit_token, libc::SIGSTOP).map_err(|_| ())?;
            member.exec_stop_version = member.audit_version;
            self.note_incarnation(index, member);
            self.members[index] = member;
            self.confirmed_stopped = false;
            let n = unsafe {
                proc_pidpath_audittoken(
                    &member.audit_token,
                    image.as_mut_ptr().cast(),
                    image.len() as u32,
                )
            };
            if n <= 0 {
                return Err(());
            }
            let end = image.iter().position(|n| *n == 0).ok_or(())?;
            let allowed = index == 1
                && current.birth == prior.birth
                && member.audit_version > prior.audit_version
                && prior.image_phase == HelperImagePhase::ParentClone
                && self
                    .identity
                    .descendant_image
                    .as_ref()
                    .is_some_and(|expected| expected.as_os_str().as_bytes() == &image[..end]);
            if !allowed {
                return Err(());
            }
            self.members[index].image_phase = HelperImagePhase::Descendant;
            self.metrics.verified_descendants =
                self.metrics.verified_descendants.checked_add(1).ok_or(())?;
            return Ok(false);
        }
        let end = image.iter().position(|n| *n == 0).ok_or(())?;
        let expected = if index == 0 || prior.image_phase == HelperImagePhase::ParentClone {
            &self.identity.image
        } else {
            self.identity.descendant_image.as_ref().ok_or(())?
        };
        if expected.as_os_str().as_bytes() != &image[..end]
            || task.pti_threadnum != 1
            || task.pti_numrunning != 0
            || info.pbi_status != 4
        {
            return Err(());
        }
        let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
        if unsafe { libc::proc_pid_rusage(prior.pid, 1, usage.as_mut_ptr().cast()) } != 0 {
            return Err(());
        }
        let usage = unsafe { usage.assume_init() };
        if usage.ri_proc_start_abstime != prior.start_abstime
            || usage.ri_proc_exit_abstime != 0
            || usage.ri_system_time < prior.system_ticks
        {
            self.invalidate_slot(index);
            return Err(());
        }
        let user_changed = usage.ri_user_time != prior.user_ticks;
        let cpu = helper_ticks_ns(
            usage
                .ri_user_time
                .checked_add(usage.ri_system_time)
                .ok_or(())?,
        )?;
        if cpu < prior.cpu {
            return Err(());
        }
        self.members[index].cpu = cpu;
        self.members[index].user_ticks = usage.ri_user_time;
        self.members[index].system_ticks = usage.ri_system_time;
        self.members[index].rss_peak = prior.rss_peak.max(usage.ri_resident_size);
        if index == 0 {
            if usage.ri_child_user_time < self.rolled_user_ticks
                || usage.ri_child_system_time < self.rolled_system_ticks
            {
                return Err(());
            }
            self.rolled_user_ticks = usage.ri_child_user_time;
            self.rolled_system_ticks = usage.ri_child_system_time;
            self.metrics.observed_direct_cpu_ns = cpu;
            self.metrics.rolled_descendant_cpu_ns = helper_ticks_ns(
                usage
                    .ri_child_user_time
                    .checked_add(usage.ri_child_system_time)
                    .ok_or(())?,
            )?;
        } else {
            self.metrics.observed_descendant_cpu_ns = cpu;
        }
        if user_changed {
            self.invalidate_slot(index);
            return Err(());
        }
        self.acknowledge_slot(index)?;
        Ok(true)
    }
    #[cfg(target_os = "macos")]
    fn exited_member_usage(&mut self, pid: i32) -> Result<(), ()> {
        let index = self
            .members
            .iter()
            .position(|m| m.pid == pid && m.start_abstime != 0)
            .ok_or(())?;
        let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
        if unsafe { libc::proc_pid_rusage(pid, 1, usage.as_mut_ptr().cast()) } != 0 {
            let known = self.members[index];
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                && known.exit_abstime != 0
                && helper_token_exited(&known.audit_token) == Ok(true)
                && self.retired_members.iter().any(|member| {
                    member.current
                        && member.pid == known.pid
                        && member.birth == known.birth
                        && member.token.value[7] == known.audit_version
                })
            {
                // A previously captured exact exit/start record survives its
                // parent's actual reap; absence alone never supplies this proof.
                return Ok(());
            }
            return Err(());
        }
        let usage = unsafe { usage.assume_init() };
        let invalid_user_after_ack = self.grant.opened
            && self.grant.slots[index].ack_ns.is_some()
            && usage.ri_user_time != self.members[index].user_ticks;
        if usage.ri_proc_start_abstime != self.members[index].start_abstime
            || usage.ri_proc_exit_abstime == 0
        {
            return Err(());
        }
        let cpu = helper_ticks_ns(
            usage
                .ri_user_time
                .checked_add(usage.ri_system_time)
                .ok_or(())?,
        )?;
        if cpu < self.members[index].cpu {
            return Err(());
        }
        self.members[index].cpu = cpu;
        self.members[index].user_ticks = usage.ri_user_time;
        self.members[index].system_ticks = usage.ri_system_time;
        self.members[index].exit_abstime = usage.ri_proc_exit_abstime;
        if index == 0 {
            let rolled = helper_ticks_ns(
                usage
                    .ri_child_user_time
                    .checked_add(usage.ri_child_system_time)
                    .ok_or(())?,
            )?;
            if rolled < self.metrics.rolled_descendant_cpu_ns {
                return Err(());
            }
            self.metrics.observed_direct_cpu_ns = cpu;
            self.metrics.rolled_descendant_cpu_ns = rolled;
            self.rolled_user_ticks = usage.ri_child_user_time;
            self.rolled_system_ticks = usage.ri_child_system_time;
        } else {
            self.metrics.observed_descendant_cpu_ns = cpu;
        }
        if invalid_user_after_ack {
            self.invalidate_slot(index);
            return Err(());
        }
        Ok(())
    }
    pub(crate) fn attached(identity: super::resolve::HelperIdentity) -> Self {
        let now = Instant::now();
        Self {
            pgid: identity.pgid,
            running: false,
            retiring: false,
            changed: now,
            ready: false,
            creation_settled: false,
            admission_failed: false,
            #[cfg(test)]
            admission_samples: 0,
            #[cfg(test)]
            fail_stop_verdict: false,
            initial_stop_pending: false,
            confirmed_stopped: false,
            identity,
            stop_requested: false,
            debt_until: now,
            lane_cpu: 0,
            members: [HelperMember::default(); 2],
            retired_members: [RetiredMember::default(); 8],
            stopped_roster: [0; 8],
            grant: HelperGrantLedger::default(),
            corroboration_pending: true,
            post_commit_pending: false,
            post_cpu: 0,
            post_late_cpu: 0,
            accounted_helper_cpu: 0,
            rolled_user_ticks: 0,
            rolled_system_ticks: 0,
            grant_commit_pending: false,
            pipe_open: false,
            pipe_active: false,
            corroboration_revision: 0,
            observed_post_revision: 0,
            post_proof_pending: false,
            post_late_known: true,
            settled_cpu: None,
            usage_reconciled: false,
            accounting_settled: false,
            last_clock_ns: None,
            metrics: HelperMetrics {
                coverage_known: true,
                ..HelperMetrics::default()
            },
        }
    }
    /// Capture descendant effects before any refusal or leader reap, including
    /// an unexpected escaped child. Signals bind Darwin's PID version token;
    /// an old token cannot affect a later process that reuses the numeric PID.
    #[cfg(target_os = "macos")]
    fn retain_incarnation(&mut self, member: RetiredMember) -> Result<(), ()> {
        if let Some(prior) = self.retired_members.iter().find(|prior| {
            prior.pid == member.pid
                && prior.birth == member.birth
                && prior.token.value[7] == member.token.value[7]
        }) {
            return if prior.current && prior.token.value == member.token.value {
                Ok(())
            } else {
                Err(())
            };
        }
        let free = self
            .retired_members
            .iter()
            .position(|prior| prior.pid == 0)
            .ok_or(())?;
        // Store the new physical recipient before any phase/version verdict;
        // prior incarnations remain retained for read-only exit queries.
        for prior in self
            .retired_members
            .iter_mut()
            .filter(|prior| prior.pid == member.pid)
        {
            prior.current = false;
        }
        self.retired_members[free] = member;
        if let Some(index) = self.members.iter().position(|known| {
            known.pid == member.pid
                && known.birth == member.birth
                && known.audit_version != 0
                && known.audit_version != member.token.value[7]
        }) {
            let mut current = self.members[index];
            current.audit_token = member.token;
            current.audit_version = member.token.value[7];
            self.note_incarnation(index, current);
            if self.retiring {
                self.metrics.coverage_known = false;
            }
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn capture_members(&mut self) -> Result<(), ()> {
        let mut parents = [0i32; 9];
        parents[0] = self.pgid;
        let mut cursor = 0;
        let mut end = 1;
        let mut valid = true;
        while cursor < end {
            let parent = parents[cursor];
            cursor += 1;
            let mut children = [0i32; 3];
            let bytes = unsafe {
                libc::proc_listchildpids(
                    parent,
                    children.as_mut_ptr().cast(),
                    std::mem::size_of_val(&children) as i32,
                )
            };
            if bytes < 0 || bytes as usize > children.len() {
                return Err(());
            }
            let count = bytes as usize;
            if count >= children.len() {
                valid = false;
            }
            for &pid in &children[..count] {
                if pid <= 0 {
                    return Err(());
                }
                if !self.stopped_roster.contains(&pid) {
                    // This exact child came from an owned parent's census.
                    // The unreaped original leader still pins the group: stop
                    // its newly discovered roster before costly token queries.
                    let free = self
                        .stopped_roster
                        .iter()
                        .position(|known| *known == 0)
                        .ok_or(())?;
                    self.stopped_roster[free] = pid;
                    self.stop_requested = true;
                    self.confirmed_stopped = false;
                    // This newly discovered recipient is a distinct roster
                    // effect. Consume its roster entry/STOP state first; keep
                    // its existing strict success-only signal policy.
                    if self.signal_group_stop().result != 0 {
                        return Err(());
                    }
                }
                let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
                let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
                if unsafe {
                    libc::proc_pidinfo(
                        pid,
                        libc::PROC_PIDTBSDINFO,
                        0,
                        info.as_mut_ptr().cast(),
                        size,
                    )
                } != size
                {
                    if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                        && self
                            .retired_members
                            .iter()
                            .any(|m| m.pid == pid && helper_token_exited(&m.token) == Ok(true))
                    {
                        continue;
                    }
                    return Err(());
                }
                let info = unsafe { info.assume_init() };
                if info.pbi_ppid != parent as u32 || info.pbi_uid != unsafe { libc::geteuid() } {
                    return Err(());
                }
                let birth = info
                    .pbi_start_tvsec
                    .checked_mul(1_000_000)
                    .and_then(|s| s.checked_add(info.pbi_start_tvusec))
                    .ok_or(())?;
                if self.retiring
                    || !self
                        .retired_members
                        .iter()
                        .any(|m| m.pid == pid && m.birth == birth)
                {
                    let member = helper_audit_member(pid, parent)?;
                    self.retain_incarnation(member)?;
                }
                if end < parents.len() {
                    parents[end] = pid;
                    end += 1;
                } else {
                    valid = false;
                }
            }
        }
        let logical_children = self
            .retired_members
            .iter()
            .enumerate()
            .filter(|(index, member)| {
                member.pid > 0
                    && member.pid != self.pgid
                    && !self.retired_members[..*index]
                        .iter()
                        .any(|prior| prior.pid == member.pid && prior.birth == member.birth)
            })
            .count();
        if logical_children > usize::from(self.identity.expected_members - 1) {
            self.metrics.coverage_known = false;
            valid = false;
        }
        if valid { Ok(()) } else { Err(()) }
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn retire_members(&mut self) -> Result<(), ()> {
        let mut valid = self.capture_members().is_ok();
        if super::resolve::helper_exited(self.pgid) == Ok(false) {
            match helper_audit_member(self.pgid, unsafe { libc::getpid() }) {
                Ok(member) => valid &= self.retain_incarnation(member).is_ok(),
                Err(()) => valid = false,
            }
        }
        for member in self
            .retired_members
            .iter()
            .filter(|m| m.pid > 0 && m.current)
        {
            if matches!(helper_token_signal(&member.token,libc::SIGSTOP),Err(code) if code!=libc::ESRCH)
            {
                valid = false;
            }
            if matches!(helper_token_signal(&member.token,libc::SIGKILL),Err(code) if code!=libc::ESRCH)
            {
                valid = false;
            }
        }
        if valid { Ok(()) } else { Err(()) }
    }
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn retire_members(&mut self) -> Result<(), ()> {
        Err(())
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn retired_members_exited(&self) -> bool {
        self.retired_members
            .iter()
            .filter(|member| member.pid > 0 && member.current)
            .all(|member| {
                if helper_token_exited(&member.token) != Ok(true) {
                    return false;
                }
                // An old task token's ESRCH is not logical process exit: exec keeps
                // PID/birth while replacing the token. A live same-birth recipient
                // therefore keeps original custody, even after its parent reaped.
                let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
                let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
                let observed = unsafe {
                    libc::proc_pidinfo(
                        member.pid,
                        libc::PROC_PIDTBSDINFO,
                        0,
                        info.as_mut_ptr().cast(),
                        size,
                    )
                };
                let same_birth_zombie = if observed == size {
                    let info = unsafe { info.assume_init() };
                    let birth = info
                        .pbi_start_tvsec
                        .checked_mul(1_000_000)
                        .and_then(|n| n.checked_add(info.pbi_start_tvusec));
                    let Some(birth) = birth else {
                        return false;
                    };
                    if birth != member.birth {
                        return true;
                    }
                    if info.pbi_status != libc::SZOMB {
                        return false;
                    }
                    true
                } else {
                    false
                };
                if !same_birth_zombie
                    && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
                {
                    return false;
                }
                let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
                if unsafe { libc::proc_pid_rusage(member.pid, 1, usage.as_mut_ptr().cast()) } == 0 {
                    let usage = unsafe { usage.assume_init() };
                    return self.members.iter().any(|known| {
                        known.pid == member.pid
                            && known.birth == member.birth
                            && known.start_abstime != 0
                            && known.start_abstime == usage.ri_proc_start_abstime
                            && usage.ri_proc_exit_abstime != 0
                    });
                }
                !same_birth_zombie
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
            })
    }
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn retired_members_exited(&self) -> bool {
        false
    }
    #[track_caller]
    fn reject(&mut self, control: &Control, code: u8) {
        if self.metrics.first_failure_line == 0 {
            self.metrics.first_failure_line = std::panic::Location::caller().line();
            self.metrics.first_failure_sample_stage = self.metrics.sample_stage;
        }
        self.ready = false;
        self.admission_failed = true;
        self.metrics.coverage_known &= code != 5;
        if self.running && !self.stop_requested && !self.retiring {
            // The original unreaped leader still pins this recorded group.
            // Revoke it once, without treating signal delivery as a stop ACK.
            if self.request_group_stop().is_err() {
                self.metrics.coverage_known = false;
            }
        }
        // Never replace an already observed cancellation/protocol outcome.
        let _ = control
            .cancel
            .compare_exchange(0, code, Ordering::AcqRel, Ordering::Acquire);
    }
    /// Charge only endpoints belonging to this grant's completed causal roster.
    /// Failed closure retains every uncertainty interval through settlement.
    pub(crate) fn finish_grant(&mut self, control: &Control, controller_cpu: u64) {
        self.metrics.controller_cpu_ns = self
            .metrics
            .controller_cpu_ns
            .saturating_add(controller_cpu);
        self.lane_cpu = self.lane_cpu.saturating_add(controller_cpu);
        if !self.grant.opened {
            return;
        }
        let now = self.ledger_clock().unwrap_or(u64::MAX);
        let mut receipt = HelperGrantReceipt {
            sequence: self.identity.sequence,
            pid: self.identity.pid,
            pgid: self.pgid,
            birth: self.identity.start_identity,
            reserved: self.grant.reserved,
            t0_ns: self.grant.t0_ns,
            close_ns: self.grant.close_ns.unwrap_or(now),
            c_ns: self.grant_control_cpu().unwrap_or(u64::MAX),
            closed: self.grant.closed,
            controller_baseline_ns: self.grant.controller_baseline,
            controller_end_ns: self.metrics.controller_cpu_ns,
            pipe_baseline_ns: self.grant.pipe_baseline,
            pipe_end_ns: self.metrics.pipe_cpu_ns,
            ..HelperGrantReceipt::default()
        };
        let mut window = Some(0u64);
        for index in 0..usize::from(self.grant.reserved) {
            let slot = self.grant.slots[index];
            let end = if !self.grant.closed {
                now
            } else if slot.through_closure {
                self.grant.close_ns.unwrap_or(u64::MAX)
            } else {
                slot.ack_ns.unwrap_or(u64::MAX)
            };
            receipt.end_ns[index] = end;
            receipt.member_pid[index] = slot.pid;
            receipt.member_birth[index] = slot.birth;
            receipt.member_start[index] = slot.start;
            receipt.member_version[index] = slot.version;
            window = window.and_then(|total| {
                if end == u64::MAX {
                    None
                } else {
                    end.checked_sub(self.grant.t0_ns)
                        .and_then(|span| total.checked_add(span))
                }
            });
        }
        receipt.w_ns = window.unwrap_or(u64::MAX);
        receipt.a_ns = self
            .actual_cpu()
            .ok()
            .and_then(|actual| actual.checked_sub(self.grant.actual_baseline))
            .unwrap_or(u64::MAX);
        receipt.q_ns = receipt.w_ns.max(receipt.a_ns).saturating_add(receipt.c_ns);
        let first = !self.grant.charged_once;
        let additional = receipt
            .q_ns
            .max(self.grant.charged)
            .saturating_sub(self.grant.charged);
        self.grant.charged = self.grant.charged.max(receipt.q_ns);
        self.grant.charged_once = true;
        self.metrics.grants = self.metrics.grants.saturating_add(u64::from(first));
        self.metrics.charged_ns = self.metrics.charged_ns.saturating_add(additional);
        if receipt.q_ns >= self.metrics.max_charge_ns {
            self.metrics.max_charge_ns = receipt.q_ns;
            self.metrics.worst_grant = receipt;
        }
        self.metrics.window_ns = self.metrics.window_ns.saturating_add(
            receipt
                .w_ns
                .max(self.grant.reported_w)
                .saturating_sub(self.grant.reported_w),
        );
        self.metrics.active_cpu_ns = self.metrics.active_cpu_ns.saturating_add(
            receipt
                .a_ns
                .max(self.grant.reported_a)
                .saturating_sub(self.grant.reported_a),
        );
        self.metrics.active_control_ns = self.metrics.active_control_ns.saturating_add(
            receipt
                .c_ns
                .max(self.grant.reported_c)
                .saturating_sub(self.grant.reported_c),
        );
        self.grant.reported_w = self.grant.reported_w.max(receipt.w_ns);
        self.grant.reported_a = self.grant.reported_a.max(receipt.a_ns);
        self.grant.reported_c = self.grant.reported_c.max(receipt.c_ns);
        let debt_cost = if first {
            receipt.q_ns.max(UNIT_NS)
        } else {
            additional
        };
        self.charge_helper_debt(control, debt_cost);
        self.running = false;
        self.grant_commit_pending = false;
        self.stop_requested = false;
        self.lane_cpu = 0;
        self.corroboration_pending = self.grant.closed;
        if self.grant.closed {
            self.accounted_helper_cpu = self.actual_cpu().unwrap_or(u64::MAX);
        }
        if receipt.q_ns > UNIT_NS {
            self.reject(control, 4);
        }
        if !self.grant.closed || receipt.a_ns > receipt.w_ns || receipt.q_ns == u64::MAX {
            self.metrics.coverage_known = false;
            self.metrics.unknown_windows = self.metrics.unknown_windows.saturating_add(1);
            self.reject(control, 5);
        }
    }
    #[cfg(target_os = "macos")]
    fn sample_exited_roster(&mut self) -> Result<bool, ()> {
        if super::resolve::helper_exited(self.pgid) != Ok(true) {
            return Err(());
        }
        for index in 0..usize::from(self.identity.expected_members) {
            let member = self.members[index];
            if member.pid <= 0
                || member.start_abstime == 0
                || member.audit_version == 0
                || (index == 0
                    && (member.pid != self.pgid || member.birth != self.identity.start_identity))
            {
                return Err(()); // The possible unknown second slot stays open.
            }
            self.exited_member_usage(member.pid)?;
            if self.members[index].exit_abstime == 0
                || helper_token_exited(&member.audit_token) != Ok(true)
            {
                return Err(());
            }
            self.members[index].image_phase = HelperImagePhase::Exited;
        }
        if !self.retired_members_exited() || super::resolve::helper_exited(self.pgid) != Ok(true) {
            return Err(());
        }
        let mut pids = [0i32; 3];
        let count = unsafe {
            libc::proc_listpgrppids(
                self.pgid,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as i32,
            )
        };
        if count < 0 || count as usize > pids.len() {
            return Err(());
        }
        if pids[..count as usize].iter().any(|pid| {
            !self
                .members
                .iter()
                .any(|member| member.pid == *pid && member.exit_abstime != 0)
        }) {
            self.capture_members()?;
            return Err(());
        }
        if !self.closed_usage_covered() {
            self.metrics.coverage_known = false;
            return Err(());
        }
        for index in 0..usize::from(self.identity.expected_members) {
            self.acknowledge_slot(index)?;
        }
        self.close_roster()?;
        self.confirmed_stopped = true;
        Ok(true)
    }
    fn closed_usage_covered(&self) -> bool {
        (self.metrics.rolled_descendant_cpu_ns <= self.members[1].cpu
            || (self.members[1].exit_abstime != 0
                && self.rolled_user_ticks == self.members[1].user_ticks
                && self.rolled_system_ticks >= self.members[1].system_ticks))
            && (self.identity.expected_members != 1
                || (self.rolled_user_ticks == 0 && self.rolled_system_ticks == 0))
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn sample(&mut self, control: &Control) -> Result<bool, ()> {
        #[cfg(test)]
        {
            self.admission_samples += 1;
        }
        self.metrics.sample_stage = 1;
        let result = self.sample_inner(control);
        if result.is_ok() {
            self.metrics.sample_stage = 0;
        }
        result
    }
    #[cfg(target_os = "macos")]
    fn sample_inner(&mut self, control: &Control) -> Result<bool, ()> {
        use std::os::unix::ffi::OsStrExt;
        self.confirmed_stopped = false;
        self.capture_members()?;
        self.initial_stop_pending = false;
        self.metrics.sample_stage = 2;
        let mut pids = [0i32; 3];
        let count = unsafe {
            libc::proc_listpgrppids(
                self.pgid,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as i32,
            )
        };
        if count == 0 {
            // Some Darwin census paths omit an unreaped zombie. A retained
            // WNOWAIT leader plus exact positive exit records can close it;
            // an empty census itself never does, especially for an unknown slot.
            return self.sample_exited_roster();
        }
        if count < 0 || count as usize > pids.len() {
            return Err(());
        }
        let count = count as usize;
        if count > self.identity.expected_members as usize || count > 2 {
            return Err(());
        }
        let mut all_stopped = true;
        let mut fresh_exec_ack = false;
        let mut rss_sum = 0u64;
        for &pid in &pids[..count] {
            self.metrics.sample_stage = 3;
            if pid <= 0 {
                return Err(());
            }
            if pid == self.pgid && super::resolve::helper_exited(pid).map_err(|_| ())? {
                // Darwin BSD/task path queries refuse a zombie, although the
                // unreaped typed leader still pins its identity and group.
                if self.members[0].pid != pid
                    || self.members[0].birth != self.identity.start_identity
                {
                    return Err(());
                }
                self.exited_member_usage(pid)?;
                self.members[0].image_phase = HelperImagePhase::Exited;
                self.acknowledge_slot(0)?;
                continue;
            }
            // One kernel snapshot supplies both identity/status and task-thread
            // quiescence. Preserve every predicate while avoiding a second
            // per-member BSD/task round trip in the same bounded sample.
            self.metrics.sample_stage = 4;
            let mut raw = std::mem::MaybeUninit::<libc::proc_taskallinfo>::uninit();
            let size = std::mem::size_of::<libc::proc_taskallinfo>();
            if unsafe {
                libc::proc_pidinfo(
                    pid,
                    libc::PROC_PIDTASKALLINFO,
                    0,
                    raw.as_mut_ptr().cast(),
                    size as i32,
                )
            } != size as i32
            {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                    && self
                        .retired_members
                        .iter()
                        .any(|m| m.pid == pid && helper_token_exited(&m.token) == Ok(true))
                {
                    self.exited_member_usage(pid)?;
                    if let Some(member) = self.members.iter_mut().find(|member| member.pid == pid) {
                        member.image_phase = HelperImagePhase::Exited;
                    }
                    let index = self
                        .members
                        .iter()
                        .position(|member| member.pid == pid)
                        .ok_or(())?;
                    self.acknowledge_slot(index)?;
                    continue;
                }
                return Err(());
            }
            let raw = unsafe { raw.assume_init() };
            self.metrics.sample_stage = 5;
            let info = raw.pbsd;
            let task = raw.ptinfo;
            let birth = info
                .pbi_start_tvsec
                .checked_mul(1_000_000)
                .and_then(|s| s.checked_add(info.pbi_start_tvusec))
                .ok_or(())?;
            if info.pbi_pid != pid as u32
                || info.pbi_pgid != self.pgid as u32
                || info.pbi_uid != unsafe { libc::geteuid() }
                || info.pbi_nice < 10
            {
                return Err(());
            }
            let index = if pid == self.pgid {
                if birth != self.identity.start_identity {
                    return Err(());
                }
                0
            } else {
                if self.identity.expected_members != 2 || info.pbi_ppid != self.pgid as u32 {
                    return Err(());
                }
                1
            };
            let prior = self.members[index];
            if prior.pid != 0 && (prior.pid != pid || prior.birth != birth) {
                return Err(());
            }
            if prior.image_phase == HelperImagePhase::Exited {
                return Err(());
            }
            let image_phase;
            let mut preexec_observations = prior.preexec_observations;
            let mut first_verified_descendant = false;
            let mut member_fresh_exec_ack = false;
            let mut audit_token = prior.audit_token;
            let mut audit_version = prior.audit_version;
            let mut exec_stop_version = prior.exec_stop_version;
            self.metrics.sample_stage = 6;
            if info.pbi_status != 5 || prior.pid == 0 {
                let mut incarnation = if prior.audit_version != 0 {
                    RetiredMember {
                        pid,
                        birth,
                        token: prior.audit_token,
                        current: true,
                    }
                } else if let Some(member) = self
                    .retired_members
                    .iter()
                    .find(|member| member.pid == pid && member.birth == birth && member.current)
                {
                    *member
                } else {
                    helper_audit_member(
                        pid,
                        if index == 0 {
                            unsafe { libc::getpid() }
                        } else {
                            self.pgid
                        },
                    )?
                };
                self.retain_incarnation(incarnation)?;
                let previous_version = incarnation.token.value[7];
                let mut incarnation_changed = false;
                let mut image = [0u8; 4096];
                let mut len = unsafe {
                    proc_pidpath_audittoken(
                        &incarnation.token,
                        image.as_mut_ptr().cast(),
                        image.len() as u32,
                    )
                };
                if len <= 0 {
                    if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                        return Err(());
                    }
                    // The BSD snapshot still described a live owned member,
                    // but its prior task incarnation is gone. Retain the new
                    // recipient before any image/version/timing refusal.
                    let mut numeric_image = [0u8; 4096];
                    if unsafe {
                        libc::proc_pidpath(
                            pid,
                            numeric_image.as_mut_ptr().cast(),
                            numeric_image.len() as u32,
                        )
                    } <= 0
                    {
                        return Err(());
                    }
                    let current = helper_audit_member(
                        pid,
                        if index == 0 {
                            unsafe { libc::getpid() }
                        } else {
                            self.pgid
                        },
                    )?;
                    self.retain_incarnation(current)?;
                    if current.birth != birth || current.token.value[7] == previous_version {
                        return Err(());
                    }
                    incarnation = current;
                    incarnation_changed = true;
                    if exec_stop_version != incarnation.token.value[7] {
                        helper_token_signal(&incarnation.token, libc::SIGSTOP).map_err(|_| ())?;
                        exec_stop_version = incarnation.token.value[7];
                    }
                    // A successful STOP on the old Git task says nothing about
                    // the new GPG task. Require a later fresh quiescence sample.
                    fresh_exec_ack = true;
                    member_fresh_exec_ack = true;
                    self.confirmed_stopped = false;
                    len = unsafe {
                        proc_pidpath_audittoken(
                            &incarnation.token,
                            image.as_mut_ptr().cast(),
                            image.len() as u32,
                        )
                    };
                    if len <= 0 || image != numeric_image {
                        return Err(());
                    }
                }
                audit_token = incarnation.token;
                audit_version = incarnation.token.value[7];
                let end = image.iter().position(|b| *b == 0).ok_or(())?;
                let observed = &image[..end];
                if index == 0 {
                    if observed != self.identity.image.as_os_str().as_bytes() || incarnation_changed
                    {
                        return Err(());
                    }
                    image_phase = HelperImagePhase::Descendant;
                } else {
                    let expected = self.identity.descendant_image.as_ref().ok_or(())?;
                    if observed == expected.as_os_str().as_bytes() {
                        if (incarnation_changed
                            && (audit_version <= previous_version
                                || !matches!(
                                    prior.image_phase,
                                    HelperImagePhase::Unobserved | HelperImagePhase::ParentClone
                                )))
                            || (prior.image_phase == HelperImagePhase::ParentClone
                                && !incarnation_changed)
                        {
                            return Err(());
                        }
                        if exec_stop_version != audit_version {
                            // The first roster STOP may have reached the clone
                            // before this exact task was captured. Stop this
                            // known verifier incarnation once, even when the
                            // earlier clone image was too brief to observe.
                            helper_token_signal(&audit_token, libc::SIGSTOP).map_err(|_| ())?;
                            exec_stop_version = audit_version;
                            fresh_exec_ack = true;
                            member_fresh_exec_ack = true;
                            self.confirmed_stopped = false;
                        }
                        first_verified_descendant =
                            prior.image_phase != HelperImagePhase::Descendant;
                        image_phase = HelperImagePhase::Descendant;
                    } else if !incarnation_changed
                        && observed == self.identity.image.as_os_str().as_bytes()
                        && matches!(
                            prior.image_phase,
                            HelperImagePhase::Unobserved | HelperImagePhase::ParentClone
                        )
                    {
                        // A signed Git verifier's fork initially has the same
                        // already provisioned image. This finite phase can only
                        // advance to the exact provisioned verifier, then exit.
                        image_phase = HelperImagePhase::ParentClone;
                        preexec_observations = preexec_observations.checked_add(1).ok_or(())?;
                    } else {
                        return Err(());
                    }
                }
                if index == 1 && prior.pid == 0 {
                    self.metrics.descendants = self.metrics.descendants.checked_add(1).ok_or(())?;
                }
            } else {
                image_phase = HelperImagePhase::Exited;
            }
            self.metrics.sample_stage = 7;
            let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
            if unsafe { libc::proc_pid_rusage(pid, 1, usage.as_mut_ptr().cast()) } != 0 {
                return Err(());
            }
            let usage = unsafe { usage.assume_init() };
            if usage.ri_proc_start_abstime == 0
                || (prior.start_abstime != 0 && usage.ri_proc_start_abstime != prior.start_abstime)
            {
                return Err(());
            }
            if index == 0 && !self.ready && self.metrics.grants == 0 {
                if usage.ri_user_time != 0 {
                    return Err(());
                }
                self.metrics.initial_kernel_cpu_ns = self
                    .metrics
                    .initial_kernel_cpu_ns
                    .max(helper_ticks_ns(usage.ri_system_time)?);
            }
            let cpu = usage
                .ri_user_time
                .checked_add(usage.ri_system_time)
                .ok_or(())?;
            let cpu = helper_ticks_ns(cpu)?;
            if cpu < prior.cpu {
                return Err(());
            }
            let invalid_user_after_ack = self.grant.opened
                && self.grant.slots[index].ack_ns.is_some()
                && audit_version == prior.audit_version
                && usage.ri_user_time != prior.user_ticks;
            rss_sum = rss_sum.checked_add(usage.ri_resident_size).ok_or(())?;
            self.members[index] = HelperMember {
                pid,
                birth,
                cpu,
                rss_peak: prior.rss_peak.max(usage.ri_resident_size),
                start_abstime: usage.ri_proc_start_abstime,
                image_phase,
                preexec_observations,
                audit_version,
                exec_stop_version,
                audit_token,
                user_ticks: usage.ri_user_time,
                system_ticks: usage.ri_system_time,
                exit_abstime: usage.ri_proc_exit_abstime,
            };
            self.note_incarnation(index, self.members[index]);
            if first_verified_descendant {
                self.metrics.verified_descendants =
                    self.metrics.verified_descendants.checked_add(1).ok_or(())?;
            }
            if index == 0 {
                self.metrics.observed_direct_cpu_ns = cpu;
                let rolled = usage
                    .ri_child_user_time
                    .checked_add(usage.ri_child_system_time)
                    .ok_or(())?;
                let rolled = helper_ticks_ns(rolled)?;
                if rolled < self.metrics.rolled_descendant_cpu_ns {
                    return Err(());
                }
                self.metrics.rolled_descendant_cpu_ns = rolled;
                self.rolled_user_ticks = usage.ri_child_user_time;
                self.rolled_system_ticks = usage.ri_child_system_time;
            } else {
                self.metrics.observed_descendant_cpu_ns = cpu;
            }
            if invalid_user_after_ack {
                self.invalidate_slot(index);
                return Err(());
            }
            if image_phase == HelperImagePhase::Exited {
                if usage.ri_proc_exit_abstime == 0 || helper_token_exited(&audit_token) != Ok(true)
                {
                    return Err(());
                }
                self.acknowledge_slot(index)?;
            }
            // Process status alone can precede task-thread quiescence. The
            // n*wall grant requires one runnable thread per helper process.
            if info.pbi_status != 5 {
                if task.pti_threadnum != 1 || !(0..=1).contains(&task.pti_numrunning) {
                    return Err(());
                }
                if index == 0 {
                    self.initial_stop_pending = info.pbi_status == 4 && task.pti_numrunning == 1;
                }
                all_stopped &= info.pbi_status == 4 && task.pti_numrunning == 0;
                if info.pbi_status == 4 && task.pti_numrunning == 0 && !member_fresh_exec_ack {
                    self.acknowledge_slot(index)?;
                }
            }
        }
        all_stopped &= !fresh_exec_ack;
        if !pids[..count].contains(&self.pgid) {
            return Err(());
        }
        self.metrics.sample_stage = 8;
        self.metrics.group_rss_peak = self.metrics.group_rss_peak.max(rss_sum);
        let separate = self.members[0]
            .rss_peak
            .checked_add(self.members[1].rss_peak)
            .ok_or(())?;
        self.metrics.separate_rss_sum = self.metrics.separate_rss_sum.max(separate);
        if rss_sum > storage::HELPERS {
            self.reject(control, 4);
            return Err(());
        }
        if all_stopped {
            self.metrics.sample_stage = 9;
            // Tentative endpoints become usable only after this SAME grant
            // re-establishes current-task stop/exit continuity and a final
            // complete roster. Every call here remains active controller CPU.
            for index in 0..2 {
                if !self.member_continuity(index)? {
                    return Ok(false);
                }
            }
            if !self.closed_usage_covered() {
                self.metrics.coverage_known = false;
                return Err(());
            }
            // One final census after continuity replaces the redundant earlier
            // group-only query. It still checks the original roster plus every
            // parent/child edge and captures any new effect before judgment.
            self.metrics.sample_stage = 10;
            if !self.recheck_stopped_roster_expected(Some(&pids[..count]))? {
                return Ok(false);
            }
            self.metrics.sample_stage = 11;
            self.close_roster()?;
            self.confirmed_stopped = true;
        }
        Ok(all_stopped)
    }
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn sample(&mut self, _: &Control) -> Result<bool, ()> {
        Err(())
    }
}
impl Control {
    /// The helper cohort's conservative debt joins the canonical ledger. Its
    /// measured owner/pipe CPU is already recorded by read_charge and control
    /// ticks, so extending this debt must not add that CPU a second time.
    fn helper_extend_debt(&self, cost: u64) -> u64 {
        if cost == 0 {
            return 0;
        }
        let debt = if cost == u64::MAX {
            self.accounting.clock_failed.store(true, Ordering::Release);
            9 * SPAN_NS
        } else {
            match cost.checked_mul(9) {
                Some(debt) => debt,
                None => {
                    self.accounting.clock_failed.store(true, Ordering::Release);
                    9 * SPAN_NS
                }
            }
        };
        let now = self.accounting.now();
        if self
            .accounting
            .rest_until
            .try_update(Ordering::AcqRel, Ordering::Acquire, |prior| {
                prior.max(now).checked_add(debt)
            })
            .is_err()
        {
            self.accounting
                .resource_failed
                .store(true, Ordering::Release);
        }
        self.accounting.debt_total.fetch_add(debt, Ordering::AcqRel);
        debt
    }
    pub(crate) fn helper_controller_marker(&self) -> Option<(u16, u64)> {
        self.helper
            .lock()
            .ok()?
            .as_ref()
            .map(|h| (h.identity.sequence, h.metrics.controller_cpu_ns))
    }
    pub(crate) fn helper_account_controller(&self, before: Option<(u16, u64)>, inclusive_cpu: u64) {
        let at = cpu_ns();
        let mut owner = self.helper.lock().unwrap_or_else(|p| p.into_inner());
        let Some(h) = owner.as_mut() else {
            return;
        };
        let already = before
            .filter(|(sequence, _)| *sequence == h.identity.sequence)
            .and_then(|(_, cpu)| h.metrics.controller_cpu_ns.checked_sub(cpu))
            .unwrap_or(0);
        let end = cpu_ns();
        let tail = self.observe_interval(at, end);
        let extra = inclusive_cpu
            .checked_sub(already)
            .and_then(|cpu| cpu.checked_add(tail))
            .filter(|_| at != u64::MAX && end != u64::MAX)
            .unwrap_or(u64::MAX);
        let _ = h.account_controller(self, extra);
        // The complete final control tick, including input/output and required
        // clock checks, joins C before charging this grant. An admitted IO span
        // cannot disappear merely because its owner has not reported it yet.
        if h.grant_commit_pending && !(h.pipe_open && h.pipe_active) {
            h.finish_grant(self, 0);
        }
        if h.post_commit_pending && !h.pending_closure() && !h.post_proof_pending {
            h.commit_stopped_unit(self, None);
        }
    }
    pub(crate) fn helper_tick(&self) {
        let mut helper = self.helper.lock().unwrap_or_else(|poisoned| {
            self.cancel.store(4, Ordering::Release);
            poisoned.into_inner()
        });
        let Some(h) = helper.as_mut() else {
            return;
        };
        if h.retiring {
            return;
        }
        let cpu_start = cpu_ns();
        let foreground = self.foreground.load(Ordering::Acquire);
        let debts_paid = Instant::now() >= h.debt_until
            && self.accounting.now() >= self.accounting.rest_until.load(Ordering::Acquire);
        if h.admission_failed {
            // Preserve possible execution/open windows through real retirement
            // and reap. Failed admission is not permission to repeat expensive
            // proof under this mutex or to invent a physically stopped seal.
        } else if h.grant_commit_pending {
            // Physically stopped; the final owner-CPU handoff is still open.
        } else if h.running {
            let mut sampled_exit = false;
            match super::resolve::helper_exited(h.pgid) {
                Ok(true) => {
                    // An unreaped exit is a reason to attempt the existing
                    // complete exit/usage/roster proof, not to signal a zombie.
                    // Leader exit alone never closes a possible child slot.
                    sampled_exit = true;
                    match h.sample(self) {
                        Ok(true) => {
                            h.running = false;
                            h.grant_commit_pending = true;
                        }
                        Ok(false) => {}
                        Err(()) if !h.admission_failed => h.reject(self, 5),
                        Err(()) => {}
                    }
                }
                Ok(false) => {}
                Err(_) => h.reject(self, 5),
            }
            if h.running
                && !h.admission_failed
                && !h.stop_requested
                && (!h.ready
                    || foreground
                    || self.error().is_some()
                    // Leave room for actual stop/identity/roster closure and
                    // its full controller charge under the unchanged1ms Q.
                    || h.changed.elapsed() >= Duration::from_micros(100))
                && h.request_group_stop().is_err()
            {
                h.reject(self, 5);
            }
            if !sampled_exit && h.stop_requested && !h.admission_failed && h.stop_observation_due()
            {
                match h.sample(self) {
                    Ok(true) => {
                        h.running = false;
                        h.grant_commit_pending = true;
                    }
                    Ok(false) => {}
                    Err(()) if !h.admission_failed => h.reject(self, 5),
                    Err(()) => {}
                }
            }
        } else if h.creation_settled
            && h.corroboration_pending
            && !h.post_commit_pending
            && !h.pipe_open
            && debts_paid
        {
            if h.corroborate_stopped(self).is_err() {
                h.corroboration_pending = false;
                h.confirmed_stopped = false;
                if !h.admission_failed {
                    h.reject(self, 5);
                }
            }
        } else if h.ready
            && !h.pending_closure()
            && !h.pending_corroboration()
            && h.members[0].image_phase != HelperImagePhase::Exited
            && !foreground
            && self.error().is_none()
            && debts_paid
        {
            if !h.confirmed_stopped || h.open_grant().is_err() {
                h.reject(self, 5);
            } else {
                h.confirmed_stopped = false;
                h.changed = Instant::now();
                if unsafe { libc::kill(-h.pgid, libc::SIGCONT) } == 0 {
                    h.running = true;
                } else {
                    h.reject(self, 5);
                }
            }
        }
        let cpu = self.observe_interval(cpu_start, cpu_ns());
        let _ = h.account_controller(self, cpu);
        if !h.admission_failed && h.pending_closure() {
            match h.pending_charge_lower_bound() {
                Ok(cost) if cost > UNIT_NS => h.reject(self, 4),
                Ok(_) => {}
                Err(()) => h.reject(self, 5),
            }
        }
    }
}

pub(crate) struct Budget {
    pub(crate) control: Arc<Control>,
    pub(crate) progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    pub(crate) counters: Counters,
    pub(crate) max_unit: u64,
    pub(crate) heap_peak: u64,
    pub(crate) overruns: u64,
    pub(crate) paused: bool,
    pub(crate) normalization_ready: bool,
    pub(crate) normalization_units: [u64; 4],
}
#[derive(Clone, Copy)]
pub(crate) struct Counters {
    pub(crate) phase: w::Phase,
    pub(crate) bytes: u64,
    pub(crate) envelopes: u64,
    pub(crate) cpu: u64,
}
impl Budget {
    pub(crate) fn new_read(
        control: Arc<Control>,
        progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    ) -> Self {
        Self {
            max_unit: control.accounting.inclusive_max.load(Ordering::Acquire),
            control,
            progress,
            counters: Counters {
                phase: w::Phase::Validate,
                bytes: 0,
                envelopes: 0,
                cpu: 0,
            },
            heap_peak: 0,
            overruns: 0,
            paused: false,
            normalization_ready: false,
            normalization_units: [0; 4],
        }
    }
    /// Admit a closed read batch through the canonical debt/foreground/heap
    /// rules, without manufacturing an empty unit or unpaid preceding debt.
    pub(crate) fn read_begin(&mut self) -> Result<(u64, Instant), w::Code> {
        self.control.accounting.pay();
        loop {
            if let Some(error) = self.control.error() {
                return Err(error);
            }
            if !self.control.foreground.load(Ordering::Acquire) {
                break;
            }
            if !self.paused {
                self.paused = true;
                self.announce(w::Phase::Paused);
            }
            thread::sleep(Duration::from_millis(2));
        }
        if self.paused {
            self.paused = false;
            self.announce(w::Phase::Validate);
        }
        self.heap_peak = self.heap_peak.max(heap());
        if self.heap_peak > HEAP_MAX {
            return Err(w::Code::ResourceLimit);
        }
        let wall = Instant::now();
        let cpu = cpu_ns();
        if cpu == u64::MAX {
            self.control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
            return Err(w::Code::ResourceLimit);
        }
        Ok((cpu, wall))
    }
    /// Record one already closed ordinary or purpose-admitted prepared-helper
    /// span. The helper caller independently validates exact native2ms/25ms;
    /// the inspect-c1 NativeKind array and its admission remain unchanged.
    pub(crate) fn read_charge(
        &mut self,
        cpu: u64,
        app: u64,
        wall_ns: u64,
        raw_limit: u64,
    ) -> Result<(), w::Code> {
        let known = cpu != u64::MAX && app != u64::MAX && wall_ns != u64::MAX;
        let raw = if known { cpu } else { u64::MAX };
        let app = if known { app } else { u64::MAX };
        self.control
            .accounting
            .inclusive_wall_max
            .fetch_max(wall_ns, Ordering::AcqRel);
        self.control.accounting.charge(raw, app);
        self.max_unit = self.max_unit.max(raw);
        if known {
            self.counters.cpu = self.counters.cpu.checked_add(cpu).unwrap_or_else(|| {
                self.control
                    .accounting
                    .clock_failed
                    .store(true, Ordering::Release);
                u64::MAX
            });
        }
        self.heap_peak = self.heap_peak.max(heap());
        if !matches!(raw_limit, UNIT_NS | SPAN_NS)
            || raw > raw_limit
            || app > UNIT_NS
            || self.heap_peak > HEAP_MAX
        {
            self.control
                .accounting
                .resource_failed
                .store(true, Ordering::Release);
            self.overruns = self.overruns.saturating_add(1);
            return Err(w::Code::ResourceLimit);
        }
        self.control.error().map_or(Ok(()), Err)
    }
    pub(crate) fn capture_max(&self) -> u64 {
        let maximum = self
            .control
            .accounting
            .inclusive_max
            .load(Ordering::Acquire)
            .max(self.control.reactor.load(Ordering::Acquire));
        self.control
            .accounting
            .captured_max
            .store(maximum, Ordering::Release);
        maximum
    }
    pub(crate) fn announce(&mut self, phase: w::Phase) {
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
    pub(crate) fn track<T: disposal::Drain>(
        &mut self,
        bytes: u64,
        f: impl FnOnce() -> Result<T, w::Code>,
    ) -> Result<storage::Tracked<T>, w::Code> {
        self.track_clock(bytes, f, cpu_ns)
    }
    pub(crate) fn track_clock<T: disposal::Drain>(
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
    pub(crate) fn unit<T: disposal::Drain>(
        &mut self,
        f: impl FnOnce() -> Result<T, w::Code>,
    ) -> Result<T, w::Code> {
        self.unit_clock(f, cpu_ns)
    }
    #[track_caller]
    pub(crate) fn unit_clock<T: disposal::Drain>(
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
pub(crate) fn cleanup_unit(control: &Control, f: impl FnOnce()) {
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
pub(crate) struct Paced<T> {
    value: Option<T>,
    step: fn(&mut T) -> bool,
    control: Arc<Control>,
}
impl<T> Paced<T> {
    pub(crate) fn new(value: T, control: &Arc<Control>, step: fn(&mut T) -> bool) -> Self {
        Self {
            value: Some(value),
            step,
            control: control.clone(),
        }
    }
    pub(crate) fn take_value(&mut self) -> T {
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
                if start == u64::MAX {
                    self.control
                        .accounting
                        .clock_failed
                        .store(true, Ordering::Release);
                }
                // The finite entry bound also applies if the CPU clock fails.
                for _ in 0..1024 {
                    if !(self.step)(self.value.as_mut().unwrap()) {
                        drop(self.value.take());
                        break;
                    }
                    if observed_interval(&self.control.accounting, start, cpu_ns()) >= 400_000 {
                        break;
                    }
                }
            });
        }
    }
}
impl Budget {
    pub(crate) fn cleanup(
        &mut self,
        f: impl FnOnce() -> Result<(), w::Code>,
    ) -> Result<(), w::Code> {
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
pub(crate) mod disposal {
    use super::*;
    use crate::protocol::*;
    pub(crate) trait Drain {
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

pub(crate) struct Dir(File);
impl Dir {
    pub(crate) fn open(path: &Path) -> Result<Self, w::Code> {
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
    pub(crate) fn dir(&self, name: &str) -> Result<Self, w::Code> {
        let file = self.openat(name, libc::O_RDONLY | libc::O_DIRECTORY, 0)?;
        let d = Self(file);
        d.check()?;
        Ok(d)
    }
    /// Fresh directory description for bounded custody enumeration. Opening the
    /// already-held directory itself avoids shared readdir offsets and follows
    /// no pathname supplied by evidence or by a discovered directory entry.
    pub(crate) fn inventory_fd(&self) -> Result<File, w::Code> {
        self.check()?;
        let fd = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY
                    | libc::O_DIRECTORY
                    | libc::O_NOFOLLOW
                    | libc::O_CLOEXEC
                    | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            return Err(w::Code::StorageUnavailable);
        }
        Ok(unsafe { File::from_raw_fd(fd) })
    }
    pub(crate) fn openat(&self, name: &str, flags: i32, mode: u32) -> Result<File, w::Code> {
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
    pub(crate) fn file(&self, name: &str, max: u64) -> Result<(File, Metadata), w::Code> {
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
    pub(crate) fn unchanged(&self, name: &str, f: &File, before: &Metadata) -> Result<(), w::Code> {
        let (_, path) = self.file(name, INPUT_MAX)?;
        let after = f.metadata().map_err(|_| w::Code::SourceChanged)?;
        if identity(before) == identity(&after) && identity(before) == identity(&path) {
            Ok(())
        } else {
            Err(w::Code::SourceChanged)
        }
    }
    /// Directory-fd exclusive rename publishes without replacement or alias links.
    pub(crate) fn publish(
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
pub(crate) fn read_bounded(
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
pub(crate) struct Completed {
    pub(crate) hash: w::Hex,
    pub(crate) pages: u32,
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
pub(crate) struct Frame {
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
pub(crate) fn framed(out: &impl serde::Serialize) -> Frame {
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
#[cfg(test)]
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
/// Internal adapter only: the owner loop, queues, credits and actual join are shared.
pub(crate) trait Job: Send + 'static {
    type Error: Send + 'static;
    fn purpose() -> bootstrap::Purpose {
        bootstrap::Purpose::InspectC1
    }
    fn error(error: w::Error) -> Self::Error;
    fn code(error: &Self::Error) -> w::Code;
    fn hello() -> Frame;
    fn decode(bytes: &[u8]) -> Result<w::WorkerIn, w::Code>;
    fn progress(progress: w::Progress) -> Frame;
    fn done(done: w::Done, error: Option<Self::Error>) -> Frame;
    fn compute(
        path: PathBuf,
        start: w::Start,
        control: Arc<Control>,
        progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    ) -> Result<Completed, Self::Error>;
}
struct C1Job;
impl Job for C1Job {
    type Error = w::Error;
    fn error(error: w::Error) -> Self::Error {
        error
    }
    fn code(error: &Self::Error) -> w::Code {
        error.code
    }
    fn hello() -> Frame {
        framed(&w::WorkerOut::Hello(w::Hello {
            protocol: w::Protocol::V1,
            algorithm: w::Algorithm::D0C1,
            schema_sha256: w::schema_hash(),
            ir_major: 1,
            ir_minor_max: 3,
        }))
    }
    fn decode(bytes: &[u8]) -> Result<w::WorkerIn, w::Code> {
        w::decode(bytes)
    }
    fn progress(progress: w::Progress) -> Frame {
        framed(&w::WorkerOut::Progress(progress))
    }
    fn done(mut done: w::Done, error: Option<Self::Error>) -> Frame {
        done.error = error;
        framed(&w::WorkerOut::Done(done))
    }
    fn compute(
        path: PathBuf,
        start: w::Start,
        control: Arc<Control>,
        progress: Arc<crossbeam_queue::ArrayQueue<Counters>>,
    ) -> Result<Completed, Self::Error> {
        compute(path, start, control, progress)
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
struct ParkedOwner<J: Job> {
    accounting: Arc<Accounting>,
    reactor: Arc<AtomicU64>,
    sender: Option<std::sync::mpsc::SyncSender<ComputeWork>>,
    handle: Option<thread::JoinHandle<Result<Completed, J::Error>>>,
}
impl<J: Job> ParkedOwner<J> {
    fn settle_clock(&mut self, mut clock: impl FnMut() -> u64) {
        if self.sender.is_none() && self.handle.is_none() {
            return;
        }
        // EOF/malformed/missing Start disconnects the parked owner, then joins
        // the actual returned handle, including timing-failed creation. This is
        // an ordinary settlement unit, with no native exemption. Its ending
        // sample follows the actual join and returned-result disposal.
        self.accounting.pay();
        let wall_at = Instant::now();
        let at = clock();
        drop(self.sender.take());
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let used = observed_interval(&self.accounting, at, clock());
        self.accounting.inclusive_wall_max.fetch_max(
            wall_at.elapsed().as_nanos().min(u64::MAX as u128) as u64,
            Ordering::AcqRel,
        );
        self.accounting.charge(used, used);
        self.reactor.fetch_max(used, Ordering::AcqRel);
        if used > UNIT_NS {
            self.accounting
                .resource_failed
                .store(true, Ordering::Release);
        }
        self.accounting.pay();
    }
}
impl<J: Job> Drop for ParkedOwner<J> {
    fn drop(&mut self) {
        self.settle_clock(cpu_ns);
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
pub(crate) fn run<J: Job>(
    path: PathBuf,
    reactor: Arc<AtomicU64>,
    mut gate: bootstrap::Gate,
) -> i32 {
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
            Ok(w) => J::compute(w.path, w.start, w.control, w.progress),
            Err(_) => Err(J::error(w::Error {
                code: w::Code::Cancelled,
                source: None,
            })),
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
    let mut owner = ParkedOwner::<J> {
        accounting: accounting.clone(),
        reactor: reactor.clone(),
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
    pending.push_back(J::hello());
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
    if permit.purpose != J::purpose() {
        return 2;
    }
    drop(gate);
    bootstrap::merge_usage(bootstrap_usage, &reactor, &accounting.control_cpu);
    loop {
        let read_control_at = (J::purpose() == bootstrap::Purpose::ReadPrepare).then(cpu_ns);
        let helper_marker = if read_control_at.is_some() {
            state
                .as_ref()
                .and_then(|(_, control, _)| control.helper_controller_marker())
        } else {
            None
        };
        let (has_helper, helper_running) = if read_control_at.is_some() {
            state.as_ref().map_or((false, false), |(_, control, _)| {
                control.helper.lock().map_or((true, false), |helper| {
                    (
                        helper.is_some(),
                        helper.as_ref().is_some_and(HelperGroup::needs_active_poll),
                    )
                })
            })
        } else {
            (false, false)
        };
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
        if has_helper && !helper_running {
            thread::sleep(Duration::from_millis(1));
        }
        unsafe { libc::poll(fds.as_mut_ptr(), 2, if has_helper { 0 } else { 2 }) };
        let tick = read_control_at.unwrap_or_else(cpu_ns);
        let mut tick_guard = ControlTick {
            at: tick,
            part_at: tick,
            accounting: &accounting,
            max: &reactor,
        };
        if read_control_at.is_some()
            && let Some((_, control, _)) = &state
        {
            control.helper_tick();
        }
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
                        let msg = flat_control(&input).and_then(|()| J::decode(&input));
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
                                    helper: std::sync::Mutex::new(None),
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
                    pending.push_back(J::progress(w::Progress {
                        protocol: w::Protocol::V1,
                        run_id: s.run_id.clone(),
                        nonce: s.nonce.clone(),
                        seq: w::Number(seq),
                        phase: counter.phase,
                        bytes: w::Number(counter.bytes),
                        envelopes: w::Number(counter.envelopes),
                        cpu_ns: w::Number(counter.cpu),
                    }));
                }
            }
            if owner.handle.as_ref().unwrap().is_finished() {
                let (s, c, rx) = state.take().unwrap();
                let h = owner.handle.take().unwrap();
                drop(rx);
                let result = h.join().unwrap_or_else(|_| {
                    Err(J::error(w::Error {
                        code: w::Code::WorkerLost,
                        source: None,
                    }))
                });
                let result = if let Some(code) = c.error() {
                    Err(J::error(w::Error { code, source: None }))
                } else {
                    result
                };
                let (outcome, hash, pages, error, code) = match result {
                    Ok(r) => (w::Outcome::Completed, Some(r.hash), r.pages, None, 0),
                    Err(e) if J::code(&e) == w::Code::Cancelled => {
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
                let wire_error = error.as_ref().map(|error| w::Error {
                    code: J::code(error),
                    source: None,
                });
                pending.push_back(J::done(
                    w::Done {
                        protocol: w::Protocol::V1,
                        run_id: s.run_id,
                        nonce: s.nonce,
                        seq: w::Number(seq),
                        outcome,
                        result_sha256: hash,
                        pages,
                        error: wire_error,
                    },
                    error,
                ));
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
        if read_control_at.is_some()
            && let Some((_, control, _)) = &state
        {
            control.helper_account_controller(helper_marker, interval(tick, cpu_ns()));
        }
        tick_guard.part(3);
        drop(tick_guard);
        let elapsed = observed_interval(&accounting, tick, cpu_ns());
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
                helper: std::sync::Mutex::new(None),
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
    fn fallback_parked_join_is_measured_and_paid_after_actual_exit_even_on_bad_clocks() {
        for end in [200, 1_000_101, u64::MAX, 99] {
            let accounting = Arc::new(Accounting::new());
            let reactor = Arc::new(AtomicU64::new(0));
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            let exited = Arc::new(AtomicBool::new(false));
            let exited_thread = exited.clone();
            let handle = thread::Builder::new()
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    assert!(receiver.recv().is_err());
                    exited_thread.store(true, Ordering::Release);
                    Err(w::Error {
                        code: w::Code::Cancelled,
                        source: None,
                    })
                })
                .unwrap();
            let mut owner = ParkedOwner::<C1Job> {
                accounting: accounting.clone(),
                reactor: reactor.clone(),
                sender: Some(sender),
                handle: Some(handle),
            };
            let mut clocks = [100, end].into_iter();
            owner.settle_clock(|| clocks.next().unwrap());
            assert!(exited.load(Ordering::Acquire));
            assert!(owner.sender.is_none() && owner.handle.is_none());
            let used = interval(100, end);
            assert_eq!(accounting.app_max.load(Ordering::Acquire), used);
            assert_eq!(reactor.load(Ordering::Acquire), used);
            assert_eq!(
                accounting.resource_failed.load(Ordering::Acquire),
                used > UNIT_NS
            );
            assert_eq!(
                accounting.clock_failed.load(Ordering::Acquire),
                used == u64::MAX
            );
            assert!(accounting.now() >= accounting.rest_until.load(Ordering::Acquire));
            assert!(accounting.rest_total.load(Ordering::Acquire) >= 9 * UNIT_NS);
            let charged = accounting.debt_total.load(Ordering::Acquire);
            drop(owner);
            assert_eq!(
                accounting.debt_total.load(Ordering::Acquire),
                charged,
                "already joined owner was disposed/charged twice"
            );
        }
    }
    #[test]
    fn intermediate_clock_failure_is_sticky_with_finite_outer_samples_and_real_effects() {
        for samples in [
            [0, 10, u64::MAX, 100],
            [0, 10, 9, 100],
            [20, 10, 50, 100],
            [0, 10, 50, 49],
        ] {
            let a = Accounting::new();
            let (tx, rx) = std::sync::mpsc::channel();
            let mut samples = samples.into_iter();
            let out = native_call_clock(
                &a,
                NativeKind::ComputeOwnerCreate,
                || {
                    thread::Builder::new()
                        .stack_size(2 * 1024 * 1024)
                        .spawn(move || rx.recv().unwrap())
                },
                |result| result.is_ok(),
                || samples.next().unwrap(),
                Instant::now,
            )
            .unwrap();
            assert_eq!(out.verdict, Err(w::Code::ResourceLimit));
            assert!(a.clock_failed.load(Ordering::Acquire));
            assert_ne!(
                a.inclusive_max.load(Ordering::Acquire),
                u64::MAX,
                "this witness requires finite outer inclusive samples"
            );
            let actual_owner = out.value.unwrap();
            tx.send(()).unwrap();
            actual_owner.join().unwrap();
            assert_eq!(a.native[0].effects.load(Ordering::Acquire), 1);
            assert_eq!(a.native[0].failures_with_effect.load(Ordering::Acquire), 1);
            a.pay();
        }
        for end in [u64::MAX, 99] {
            let control = budget(false).control;
            let at = cpu_ns();
            let mut tick = ControlTick {
                at,
                part_at: 100,
                accounting: &control.accounting,
                max: &control.reactor,
            };
            tick.part_sample(0, end);
            drop(tick); // actual outer samples remain finite
            assert!(control.accounting.clock_failed.load(Ordering::Acquire));
            assert_eq!(
                control.accounting.control_parts[0].load(Ordering::Acquire),
                u64::MAX
            );
            assert_eq!(control.error(), Some(w::Code::ResourceLimit));
        }
        let control = budget(false).control;
        let mut value = vec![1];
        let mut outer = [100, 200].into_iter();
        cleanup_unit_clock(
            &control,
            || {
                assert_eq!(value.pop(), Some(1)); // real disposal effect retained
                observed_interval(&control.accounting, 100, u64::MAX);
            },
            || outer.next().unwrap(),
        );
        assert!(value.is_empty());
        assert_eq!(control.cleanup_max.load(Ordering::Acquire), 100);
        assert!(control.accounting.clock_failed.load(Ordering::Acquire));
        assert_eq!(control.error(), Some(w::Code::ResourceLimit));
        let a = Accounting::new();
        assert_eq!(observed_interval(&a, 100, u64::MAX), u64::MAX);
        assert!(a.clock_failed.load(Ordering::Acquire)); // final post-drop sample
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
            if let Some(p) = lane.pop()
                && p.phase == w::Phase::Paused
            {
                break;
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

#[cfg(all(test, target_os = "macos"))]
mod helper_admission_tests {
    use super::*;

    fn accounting_group() -> HelperGroup {
        // No effect is created by these arithmetic controls. Keeping running
        // false ensures their refusal cannot signal a numeric fixture identity.
        HelperGroup::attached(super::super::resolve::HelperIdentity {
            sequence: 1,
            pid: 0,
            pgid: 0,
            start_identity: 0,
            expected_members: 1,
            image: PathBuf::from("/private/accounting-fixture"),
            descendant_image: None,
        })
    }

    #[test]
    fn zero_additional_helper_cost_preserves_paid_and_outstanding_debt_exactly() {
        let control = Control::fixture(Instant::now() + Duration::from_secs(2));
        control.helper_extend_debt(1_000_000);
        let until = control.accounting.rest_until.load(Ordering::Acquire);
        let charged = control.accounting.debt_total.load(Ordering::Acquire);
        control.helper_extend_debt(0);
        assert_eq!(control.accounting.rest_until.load(Ordering::Acquire), until);
        assert_eq!(
            control.accounting.debt_total.load(Ordering::Acquire),
            charged
        );
        control.read_pay();
        let until = control.accounting.rest_until.load(Ordering::Acquire);
        control.helper_extend_debt(0);
        assert_eq!(control.accounting.rest_until.load(Ordering::Acquire), until);
        assert_eq!(
            control.accounting.debt_total.load(Ordering::Acquire),
            charged
        );
    }

    #[test]
    fn unknown_helper_cost_retains_raw_failure_and_known_debt_with_finite_settlement() {
        for grant in [false, true] {
            let control = Control::fixture(Instant::now() + Duration::from_secs(2));
            control.helper_extend_debt(UNIT_NS);
            let mut helper = accounting_group();
            let known = Instant::now() + Duration::from_millis(17);
            helper.debt_until = known;
            if grant {
                helper.grant.opened = true;
                helper.grant.reserved = 1;
                helper.grant.t0_ns = bootstrap::monotonic_ns().unwrap();
                helper.metrics.controller_cpu_ns = u64::MAX;
                helper.finish_grant(&control, 0);
                assert_eq!(helper.metrics.max_charge_ns, u64::MAX);
                assert_eq!(helper.metrics.worst_grant.q_ns, u64::MAX);
            } else {
                helper.post_commit_pending = true;
                helper.post_cpu = u64::MAX;
                helper.commit_stopped_unit(&control, None);
                assert_eq!(helper.metrics.post_max_cpu_ns, u64::MAX);
                assert_eq!(helper.metrics.post_cpu_ns, u64::MAX);
            }
            let fallback = 9 * SPAN_NS;
            assert_eq!(helper.metrics.debt_ns, fallback);
            assert_eq!(
                control.accounting.debt_total.load(Ordering::Acquire),
                9 * UNIT_NS + fallback
            );
            assert!(helper.debt_until >= known + Duration::from_nanos(fallback));
            assert!(helper.debt_until < Instant::now() + Duration::from_secs(1));
            assert!(helper.admission_failed && !helper.ready && !helper.metrics.coverage_known);
            assert!(control.accounting.clock_failed.load(Ordering::Acquire));
            assert_eq!(control.error(), Some(w::Code::ResourceLimit));
            control.read_pay();
        }
    }

    #[test]
    fn usage_c_and_l_share_one_fresh_floor_without_hiding_inclusive_overrun() {
        for (cpu, late) in [(100_000, 200_000), (600_000, 600_000), (1_200_000, 600_000)] {
            let control = Arc::new(Control::fixture(Instant::now() + Duration::from_secs(2)));
            let mut budget = Budget::new_read(
                control.clone(),
                Arc::new(crossbeam_queue::ArrayQueue::new(1)),
            );
            let mut helper = accounting_group();
            helper.post_proof_pending = true;
            helper.post_commit_pending = true;
            helper.post_late_known = true;
            helper.post_late_cpu = late;
            let recorded = budget.read_charge(cpu, cpu, 1, UNIT_NS);
            assert_eq!(recorded.is_ok(), cpu <= UNIT_NS);
            let combined = cpu + late;
            let committed = helper.commit_post_observation(&control, cpu, 1);
            assert_eq!(committed.is_ok(), combined <= UNIT_NS);
            let expected = combined.max(UNIT_NS) * 9;
            assert_eq!(
                control.accounting.debt_total.load(Ordering::Acquire),
                expected
            );
            assert_eq!(helper.metrics.debt_ns, expected - cpu.max(UNIT_NS) * 9);
            assert_eq!(helper.metrics.post_cpu_ns, cpu);
            assert_eq!(helper.metrics.post_max_cpu_ns, combined);
            assert_eq!(helper.metrics.late_kernel_cpu_ns, late);
            assert_eq!(budget.counters.cpu, cpu, "actual owner C is recorded once");
            if combined > UNIT_NS {
                assert_eq!(control.error(), Some(w::Code::ResourceLimit));
                assert!(helper.admission_failed && !helper.ready);
            }
            control.read_pay();
        }
    }

    struct OwnedChild {
        pid: i32,
        control: Arc<Control>,
        reaped: bool,
    }
    impl OwnedChild {
        fn reap(&mut self) {
            if self.reaped {
                return;
            }
            // The retained direct child pins this exact group until final kill.
            if let Some(helper) = self
                .control
                .helper
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .as_mut()
            {
                helper.retiring = true;
            }
            unsafe { libc::kill(-self.pid, libc::SIGKILL) };
            let mut status = 0;
            loop {
                let result = unsafe { libc::waitpid(self.pid, &mut status, 0) };
                if result == self.pid {
                    self.reaped = true;
                    break;
                }
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::EINTR)
                );
            }
        }
    }
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            self.reap();
        }
    }

    #[test]
    fn failed_group_stop_verdict_does_not_repeat_or_close_real_owned_child() {
        let control = Arc::new(Control::fixture(Instant::now() + Duration::from_secs(2)));
        let mut proof = crate::dream::read::helper_fixture_clock(control.clone());
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
        assert!(pid > 0, "retain the positive owned native effect");
        let mut child = OwnedChild {
            pid,
            control: control.clone(),
            reaped: false,
        };
        assert_eq!(unsafe { libc::posix_spawnattr_destroy(&mut attr) }, 0);
        assert_eq!(result, 0);
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
        let identity = super::super::resolve::HelperIdentity {
            sequence: 1,
            pid,
            pgid: pid,
            start_identity: info.pbi_start_tvsec * 1_000_000 + info.pbi_start_tvusec,
            expected_members: 1,
            image: std::fs::canonicalize("/bin/sleep").unwrap(),
            descendant_image: None,
        };
        let ticket = identity.ticket();
        proof.helper_attach(identity).unwrap();
        let samples = {
            let mut owner = control.helper.lock().unwrap();
            let helper = owner.as_mut().unwrap();
            assert!(helper.confirmed_stopped);
            helper.open_grant().unwrap();
            // The real child remains kernel-stopped. A possible interval still
            // cannot be closed merely by an attempted STOP or a refusal.
            helper.running = true;
            helper.confirmed_stopped = false;
            helper.fail_stop_verdict = true;
            assert!(helper.request_group_stop().is_err());
            assert!(
                !helper.fail_stop_verdict,
                "the causal verdict fault executed"
            );
            assert_eq!(helper.metrics.last_stop.result, 0);
            assert_eq!(helper.metrics.last_stop.rawerrno, 0);
            helper.reject(&control, 5); // the old ordering issued a second STOP here
            assert!(helper.request_group_stop().is_err());
            assert_eq!(helper.metrics.stop_attempts, 1);
            assert_eq!(
                helper.metrics.stop_failures, 0,
                "native outcome was not fabricated"
            );
            assert_eq!(helper.metrics.last_stop.pgid, pid);
            assert_eq!(helper.metrics.last_stop.sequence, ticket.sequence);
            assert!(helper.stop_requested && helper.admission_failed);
            assert!(helper.running && helper.pending_closure());
            assert!(!helper.confirmed_stopped && !helper.grant.closed);
            assert_eq!(helper.grant.slots[0].ack_ns, None);
            helper.admission_samples
        };
        control.helper_tick();
        {
            let owner = control.helper.lock().unwrap();
            let helper = owner.as_ref().unwrap();
            assert_eq!(helper.metrics.stop_attempts, 1);
            assert_eq!(helper.admission_samples, samples);
            assert!(!helper.confirmed_stopped && !helper.grant.closed);
            assert_eq!(super::super::resolve::helper_exited(pid), Ok(false));
        }
        assert!(proof.helper_retire(ticket).is_err());
        child.reap();
        assert_eq!(unsafe { libc::kill(-pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        proof.helper_detach(ticket).unwrap();
        assert!(control.helper.lock().unwrap().is_none());
    }

    #[test]
    fn failed_admission_stops_repeated_sampling_and_retains_real_open_custody() {
        let control = Arc::new(Control::fixture(Instant::now() + Duration::from_secs(2)));
        let mut proof = crate::dream::read::helper_fixture_clock(control.clone());
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
        assert!(pid > 0, "private process must retain its positive effect");
        let mut child = OwnedChild {
            pid,
            control: control.clone(),
            reaped: false,
        };
        assert_eq!(unsafe { libc::posix_spawnattr_destroy(&mut attr) }, 0);
        assert_eq!(result, 0);
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
        let identity = super::super::resolve::HelperIdentity {
            sequence: 1,
            pid,
            pgid: pid,
            start_identity: info.pbi_start_tvsec * 1_000_000 + info.pbi_start_tvusec,
            expected_members: 1,
            image: std::fs::canonicalize("/bin/sleep").unwrap(),
            descendant_image: None,
        };
        let ticket = identity.ticket();
        *control.helper.lock().unwrap() = Some(HelperGroup::attached(identity));
        let observing = Instant::now();
        loop {
            let mut owner = control.helper.lock().unwrap();
            let helper = owner.as_mut().unwrap();
            match helper.sample(&control) {
                Ok(true) => {
                    helper.settle_creation(&control).unwrap();
                    break;
                }
                Ok(false)
                    if helper.initial_stop_pending
                        && observing.elapsed() < Duration::from_millis(25) => {}
                result => panic!("actual suspended birth: {result:?}"),
            }
            drop(owner);
            thread::sleep(Duration::from_micros(100));
        }
        let marker = control.helper_controller_marker();
        let at = cpu_ns();
        control.helper_tick();
        control.helper_account_controller(marker, control.observe_interval(at, cpu_ns()));
        control.read_pay();
        loop {
            let owner = control.helper.lock().unwrap();
            let helper = owner.as_ref().unwrap();
            assert!(!helper.pending_corroboration() && helper.confirmed_stopped);
            if Instant::now() >= helper.debt_until {
                break;
            }
            drop(owner);
            thread::sleep(Duration::from_millis(1));
        }
        assert!(control.error().is_none());
        assert!(
            !control
                .helper
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .retired_members_exited(),
            "a retained live incarnation is not an exit acknowledgement"
        );
        let (t0, samples) = {
            let mut owner = control.helper.lock().unwrap();
            let helper = owner.as_mut().unwrap();
            helper.open_grant().unwrap();
            helper.ready = true;
            helper.changed = Instant::now();
            assert_eq!(unsafe { libc::kill(-pid, libc::SIGCONT) }, 0);
            helper.running = true;
            helper.confirmed_stopped = false;
            // Causal negative: the real process cannot match this exact image.
            helper.identity.image = PathBuf::from("/private/unavailable-helper-image");
            (helper.grant.t0_ns, helper.admission_samples)
        };
        // Drive a real stopped kernel endpoint before the intended image
        // fault. The admission hint itself must not install any stop seal.
        assert_eq!(unsafe { libc::kill(-pid, libc::SIGSTOP) }, 0);
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            let owner = control.helper.lock().unwrap();
            let helper = owner.as_ref().unwrap();
            assert_eq!(helper.grant.t0_ns, t0);
            assert!(!helper.confirmed_stopped && !helper.grant.closed);
            if helper.stop_observation_due() {
                break;
            }
            assert!(Instant::now() < until, "owned STOP tail must settle");
            drop(owner);
            thread::sleep(Duration::from_micros(100));
        }
        control.foreground.store(true, Ordering::Release);
        control.helper_tick();
        let rejected_samples = {
            let owner = control.helper.lock().unwrap();
            let helper = owner.as_ref().unwrap();
            assert!(helper.admission_failed && !helper.ready && !helper.needs_active_poll());
            assert!(helper.stop_requested && !helper.confirmed_stopped);
            assert!(helper.running && helper.pending_closure() && !helper.grant.closed);
            assert_eq!(helper.grant.t0_ns, t0);
            assert_eq!(helper.admission_samples, samples + 1);
            helper.admission_samples
        };
        for _ in 0..8 {
            let marker = control.helper_controller_marker();
            let at = cpu_ns();
            control.helper_tick();
            control.helper_account_controller(marker, control.observe_interval(at, cpu_ns()));
        }
        {
            let owner = control.helper.lock().unwrap();
            let helper = owner.as_ref().unwrap();
            assert_eq!(helper.admission_samples, rejected_samples);
            assert_eq!(helper.identity.ticket(), ticket);
            assert_eq!(helper.grant.t0_ns, t0);
            assert!(helper.running && helper.pending_closure() && !helper.grant.closed);
            assert!(helper.stop_requested && !helper.confirmed_stopped);
            assert!(helper.metrics.controller_cpu_ns > 0);
        }
        assert!(proof.helper_retire(ticket).is_err());
        let exiting = Instant::now();
        while super::super::resolve::helper_exited(pid) != Ok(true) {
            assert!(
                exiting.elapsed() < Duration::from_secs(1),
                "private original leader must actually exit"
            );
            thread::sleep(Duration::from_micros(100));
        }
        {
            let mut owner = control.helper.lock().unwrap();
            let helper = owner.as_mut().unwrap();
            assert!(
                helper.retired_members_exited(),
                "exact unreaped exit has positive start/RI/token proof"
            );
            let start = helper.members[0].start_abstime;
            helper.members[0].start_abstime = start.checked_add(1).unwrap();
            assert!(
                !helper.retired_members_exited(),
                "same numeric PID without exact retained start does not close custody"
            );
            helper.members[0].start_abstime = start;
            helper.identity.expected_members = 2;
            assert!(
                helper.sample_exited_roster().is_err(),
                "an unobserved possible descendant stays unproven"
            );
            assert!(!helper.grant.closed);
            helper.identity.expected_members = 1;
            assert_eq!(helper.sample_exited_roster(), Ok(true));
            assert!(helper.grant.closed && helper.members[0].exit_abstime != 0);
        }
        child.reap();
        assert_eq!(unsafe { libc::kill(-pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        proof.helper_detach(ticket).unwrap();
        assert!(control.helper.lock().unwrap().is_none());
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
