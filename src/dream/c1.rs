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
// Decision66331 changes only prepared posix_spawn. The existing thread-create
// and publish NativeKind gates below retain their25ms native wall ceiling.
pub(crate) const PROCESS_CREATE_WALL_NS: u64 = 250_000_000;

pub(crate) fn process_create_wall_allowed(wall_ns: u64) -> bool {
    wall_ns <= PROCESS_CREATE_WALL_NS
}

/// Invocation-local sum/count of known process_create wall excess over25ms.
/// Observation precedes admission: a refused251ms call still contributes its
/// full226ms excess. Unknown or unrepresentable totals are sticky failures;
/// the caller retains the original raw wall and must not certify this prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessCreateWallExcess {
    pub(crate) process_create_wall_excess_ns: u64,
    pub(crate) process_create_wall_excess_count: u64,
    pub(crate) known: bool,
}
impl Default for ProcessCreateWallExcess {
    fn default() -> Self {
        Self {
            process_create_wall_excess_ns: 0,
            process_create_wall_excess_count: 0,
            known: true,
        }
    }
}
impl ProcessCreateWallExcess {
    pub(crate) fn record(&mut self, wall_ns: u64) -> Result<(), ()> {
        if !self.known || wall_ns == u64::MAX {
            self.known = false;
            return Err(());
        }
        let excess = wall_ns.saturating_sub(NATIVE_WALL_NS);
        if excess == 0 {
            return Ok(());
        }
        match (
            self.process_create_wall_excess_ns.checked_add(excess),
            self.process_create_wall_excess_count.checked_add(1),
        ) {
            (Some(ns), Some(count)) => {
                self.process_create_wall_excess_ns = ns;
                self.process_create_wall_excess_count = count;
                Ok(())
            }
            _ => {
                self.known = false;
                Err(())
            }
        }
    }
}

#[cfg(test)]
mod process_create_wall_tests {
    use super::*;

    #[test]
    fn prepared_spawn_wall_limit_and_full_excess_are_independent() {
        let mut metrics = ProcessCreateWallExcess::default();
        for (wall, allowed) in [
            (25_000_000, true),
            (34_000_000, true),
            (250_000_000, true),
            (251_000_000, false),
        ] {
            metrics.record(wall).unwrap();
            assert_eq!(process_create_wall_allowed(wall), allowed);
        }
        assert_eq!(metrics.process_create_wall_excess_ns, 460_000_000);
        assert_eq!(metrics.process_create_wall_excess_count, 3);
        assert!(metrics.known);
        assert!(!process_create_wall_allowed(u64::MAX));
    }

    #[test]
    fn existing_thread_and_publish_native_classes_still_refuse_34ms() {
        for kind in [NativeKind::ComputeOwnerCreate, NativeKind::PublishNoreplace] {
            let accounting = Accounting::new();
            let start = Instant::now();
            let mut clocks = [start, start + Duration::from_millis(34)].into_iter();
            let out = native_call_clock(
                &accounting,
                kind,
                || 7,
                |_| false,
                || 0,
                || clocks.next().unwrap(),
            )
            .unwrap();
            assert_eq!(out.value, 7);
            assert!(matches!(out.verdict, Err(w::Code::ResourceLimit)));
            assert_eq!(
                accounting.native[kind as usize]
                    .max_wall
                    .load(Ordering::Acquire),
                34_000_000
            );
        }
    }

    #[test]
    fn excess_unknown_and_overflow_cannot_be_finite_complete_totals() {
        let mut unknown = ProcessCreateWallExcess::default();
        unknown.record(34_000_000).unwrap();
        assert!(unknown.record(u64::MAX).is_err());
        assert!(!unknown.known);
        assert_eq!(unknown.process_create_wall_excess_ns, 9_000_000);
        assert_eq!(unknown.process_create_wall_excess_count, 1);
        assert!(unknown.record(34_000_000).is_err());

        let mut overflow = ProcessCreateWallExcess {
            process_create_wall_excess_ns: u64::MAX - 5,
            process_create_wall_excess_count: 1,
            known: true,
        };
        assert!(overflow.record(25_000_006).is_err());
        assert!(!overflow.known);
        assert_eq!(overflow.process_create_wall_excess_ns, u64::MAX - 5);
        assert_eq!(overflow.process_create_wall_excess_count, 1);
    }
}
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
    helper_job_cpu: AtomicU64,
    #[cfg(test)]
    helper_test_skip_live_sampling: AtomicBool,
    helper_control_work: AtomicU64,
    helper_control_rest_until: AtomicU64,
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
            helper_job_cpu: AtomicU64::new(0),
            #[cfg(test)]
            helper_test_skip_live_sampling: AtomicBool::new(false),
            helper_control_work: AtomicU64::new(0),
            helper_control_rest_until: AtomicU64::new(0),
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
    pub(crate) fn fixture_skip_live_sampling(&self, skip: bool) {
        self.accounting
            .helper_test_skip_live_sampling
            .store(skip, Ordering::Release);
    }
    #[cfg(test)]
    pub(crate) fn fixture_debt_totals(&self) -> (u64, u64) {
        (
            self.accounting.debt_total.load(Ordering::Acquire),
            self.accounting.rest_total.load(Ordering::Acquire),
        )
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
// Adapted from bboxy450-harness; policy https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Decision66280: external CPU has command/job totals, never grant windows.
const HELPER_COMMAND_CPU_NS: u64 = 250_000_000;
const HELPER_JOB_CPU_NS: u64 = 3_000_000_000;
const HELPER_SAMPLE_INTERVAL: Duration = Duration::from_millis(25);
const HELPER_SAMPLE_LIMIT: Duration = Duration::from_millis(50);
const HELPER_PAUSE_LIMIT: Duration = Duration::from_millis(100);

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HelperAuditToken {
    value: [u32; 8],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HelperTimebase {
    numer: u32,
    denom: u32,
}
#[derive(Clone, Copy, Default)]
struct HelperMember {
    pid: i32,
    birth: u64,
    start: u64,
    user: u64,
    system: u64,
    cpu: u64,
    rolled: u64,
    rolled_user: u64,
    terminal_ri_at: Option<Instant>,
    rss: u64,
    live: bool,
    stopped: bool,
    verified: bool,
    paused_user: Option<u64>,
    ri_started: Option<Instant>,
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
fn helper_timebase() -> Result<HelperTimebase, ()> {
    let mut t = HelperTimebase::default();
    if unsafe { helper_timebase_info(&mut t) } == 0 && t.numer != 0 && t.denom != 0 {
        Ok(t)
    } else {
        Err(())
    }
}
#[cfg(not(target_os = "macos"))]
fn helper_timebase() -> Result<HelperTimebase, ()> {
    Err(())
}
// Apple libproc maps syscall failure to a zero count with errno preserved.
// Source: https://github.com/apple-oss-distributions/xnu/blob/main/libsyscall/wrappers/libproc/libproc.c#L40-L89
// A zero is an empty census only after a freshly cleared, still-zero errno.
#[cfg(target_os = "macos")]
fn helper_clear_errno() {
    unsafe {
        *libc::__error() = 0;
    }
}
fn helper_census_known(count: i32, errno: i32) -> bool {
    count >= 0 && errno == 0
}
fn helper_census_empty(count: i32, errno: i32) -> bool {
    count == 0 && helper_census_known(count, errno)
}
fn helper_convert_ticks(ticks: u64, t: HelperTimebase) -> Result<u64, ()> {
    if t.numer == 0 || t.denom == 0 {
        return Err(());
    }
    u64::try_from((u128::from(ticks) * u128::from(t.numer)).div_ceil(u128::from(t.denom)))
        .map_err(|_| ())
}
#[cfg(target_os = "macos")]
fn helper_bsd(pid: i32) -> Result<libc::proc_bsdinfo, ()> {
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
    Ok(unsafe { info.assume_init() })
}
#[cfg(target_os = "macos")]
fn helper_birth(info: &libc::proc_bsdinfo) -> Result<u64, ()> {
    info.pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(info.pbi_start_tvusec))
        .ok_or(())
}
#[cfg(target_os = "macos")]
fn helper_audit_member(pid: i32, parent: i32) -> Result<RetiredMember, ()> {
    let before = helper_bsd(pid)?;
    if before.pbi_pid != pid as u32
        || before.pbi_ppid != parent as u32
        || before.pbi_uid != unsafe { libc::geteuid() }
    {
        return Err(());
    }
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
    let after = helper_bsd(pid)?;
    if result != 0
        || count != 8
        || token.value[5] != pid as u32
        || before.pbi_ppid != after.pbi_ppid
        || helper_birth(&before)? != helper_birth(&after)?
    {
        return Err(());
    }
    Ok(RetiredMember {
        pid,
        birth: helper_birth(&before)?,
        token,
        current: true,
    })
}
#[cfg(target_os = "macos")]
fn helper_token_exited(token: &HelperAuditToken) -> Result<bool, i32> {
    let mut path = [0u8; 4096];
    if unsafe { proc_pidpath_audittoken(token, path.as_mut_ptr().cast(), path.len() as u32) } > 0 {
        return Ok(false);
    }
    match io::Error::last_os_error().raw_os_error() {
        Some(libc::ESRCH) => Ok(true),
        Some(e) => Err(e),
        None => Err(libc::EIO),
    }
}
#[cfg(target_os = "macos")]
fn helper_token_signal(token: &HelperAuditToken, signal: i32) -> Result<(), i32> {
    match unsafe { proc_signal_with_audittoken(token, signal) } {
        0 => Ok(()),
        e if e > 0 => Err(e),
        _ => Err(io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(libc::EIO)),
    }
}

/// Bounded raw initial observation. A zero field is meaningful only when its
/// validity bit is present: BSD1, audit2, task4, RI8, image16, final BSD32.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HelperInitialObservation {
    pub(crate) stage: u8,
    pub(crate) valid: u8,
    pub(crate) readiness: i8,
    pub(crate) bsd_result: i32,
    pub(crate) bsd_errno: i32,
    pub(crate) pid: u32,
    pub(crate) ppid: u32,
    pub(crate) pgid: u32,
    pub(crate) uid: u32,
    pub(crate) birth: u64,
    pub(crate) status: u32,
    pub(crate) task_result: i32,
    pub(crate) task_errno: i32,
    pub(crate) threads: i32,
    pub(crate) running: i32,
    pub(crate) priority: i32,
    pub(crate) ri_result: i32,
    pub(crate) ri_errno: i32,
    pub(crate) user_ticks: u64,
    pub(crate) system_ticks: u64,
    pub(crate) start: u64,
    pub(crate) after_status: u32,
    pub(crate) timebase_numer: u32,
    pub(crate) timebase_denom: u32,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HelperMetrics {
    pub(crate) initial: HelperInitialObservation,
    pub(crate) initial_pending_observations: u64,
    pub(crate) controller_cpu_ns: u64,
    pub(crate) pipe_cpu_ns: u64,
    pub(crate) coverage_known: bool,
    pub(crate) clock_failures: u64,
    pub(crate) first_failure_line: u32,
    pub(crate) first_failure_sample_stage: u8,
    pub(crate) charged_ns: u64,
    pub(crate) max_charge_ns: u64,
    pub(crate) debt_ns: u64,
    pub(crate) group_rss_peak: u64,
    pub(crate) mixed_heap_rss_peak: u64,
    pub(crate) observed_direct_cpu_ns: u64,
    pub(crate) observed_descendant_cpu_ns: u64,
    pub(crate) rolled_descendant_cpu_ns: u64,
    pub(crate) descendants: u64,
    pub(crate) verified_descendants: u64,
    pub(crate) reap_user_ns: u64,
    pub(crate) reap_system_ns: u64,
    pub(crate) reap_quantization_ns: u64,
    pub(crate) reap_rss_bytes: u64,
    pub(crate) reap_rss_known: bool,
    pub(crate) group_kill_attempted: bool,
    pub(crate) group_kill_not_attempted_terminal: bool,
    pub(crate) terminal_observations: u64,
    pub(crate) terminal_last_guard: u8,
    pub(crate) terminal_waitid_result: i32,
    pub(crate) terminal_waitid_errno: i32,
    pub(crate) terminal_waitid_pid: i32,
    pub(crate) terminal_waitid_code: i32,
    pub(crate) terminal_bsd_result: i32,
    pub(crate) terminal_bsd_errno: i32,
    pub(crate) terminal_census_result: i32,
    pub(crate) terminal_census_errno: i32,
    pub(crate) terminal_member_pid: i32,
    pub(crate) pause_stop_attempted: bool,
    pub(crate) pause_stop_errno: i32,
    pub(crate) pause_ack_ns: u64,
    pub(crate) pause_user_baseline_ns: u64,
    pub(crate) pause_terminal_user_ns: u64,
    pub(crate) pause_user_excess_ns: u64,
    pub(crate) pause_user_quantization_ns: u64,
    pub(crate) pause_user_checked: bool,
    pub(crate) pause_user_pass: bool,
    pub(crate) group_kill_errno: i32,
    pub(crate) group_kill_raced_exit: bool,
    pub(crate) qos_checked_mask: u8,
    pub(crate) live_sample_count: u64,
    pub(crate) max_sample_gap_ns: u64,
    pub(crate) ri_previous_start_ns: [u64; 2],
    pub(crate) ri_start_ns: [u64; 2],
    pub(crate) ri_end_ns: [u64; 2],
    pub(crate) ri_gap_ns: [u64; 2],
    pub(crate) ri_previous_valid_mask: u8,
    pub(crate) ri_start_valid_mask: u8,
    pub(crate) ri_end_valid_mask: u8,
    pub(crate) escaped_kill_attempts: u64,
    pub(crate) escaped_kill_failures: u64,
    pub(crate) escaped_kill_errno: i32,
    pub(crate) child_cost_unproven: bool,
    pub(crate) child_rollup_proven: bool,
    pub(crate) child_terminal_user_ticks: u64,
    pub(crate) child_terminal_system_ticks: u64,
    pub(crate) leader_child_user_ticks: u64,
    pub(crate) leader_child_system_ticks: u64,
    pub(crate) child_edge_closed: bool,
    pub(crate) child_edge_result: i32,
    pub(crate) child_edge_errno: i32,
    pub(crate) child_rollup_read_after_exit: bool,
    pub(crate) job_cpu_ns: u64,
    pub(crate) wait4_reads: u64,
    pub(crate) reap_authorized: bool,
    merged: bool,
}
impl HelperMetrics {
    pub(crate) fn merge(&mut self, other: &Self) {
        macro_rules! sum { ($($f:ident),* $(,)?) => { $(self.$f = self.$f.saturating_add(other.$f);)* }; }
        macro_rules! peak { ($($f:ident),* $(,)?) => { $(self.$f = self.$f.max(other.$f);)* }; }
        if other.max_sample_gap_ns >= self.max_sample_gap_ns {
            self.ri_previous_start_ns = other.ri_previous_start_ns;
            self.ri_start_ns = other.ri_start_ns;
            self.ri_end_ns = other.ri_end_ns;
            self.ri_gap_ns = other.ri_gap_ns;
            self.ri_previous_valid_mask = other.ri_previous_valid_mask;
            self.ri_start_valid_mask = other.ri_start_valid_mask;
            self.ri_end_valid_mask = other.ri_end_valid_mask;
        }
        self.child_cost_unproven |= other.child_cost_unproven;
        self.child_rollup_proven |= other.child_rollup_proven;
        if other.child_edge_result != 0
            || other.child_edge_errno != 0
            || other.child_edge_closed
            || other.child_cost_unproven
        {
            self.child_edge_result = other.child_edge_result;
            self.child_edge_errno = other.child_edge_errno;
        }
        if other.child_terminal_user_ticks != 0 || other.child_terminal_system_ticks != 0 {
            self.child_terminal_user_ticks = other.child_terminal_user_ticks;
            self.child_terminal_system_ticks = other.child_terminal_system_ticks;
            self.leader_child_user_ticks = other.leader_child_user_ticks;
            self.leader_child_system_ticks = other.leader_child_system_ticks;
            self.child_edge_closed = other.child_edge_closed;
            self.child_rollup_read_after_exit = other.child_rollup_read_after_exit;
        }
        self.escaped_kill_attempts = self
            .escaped_kill_attempts
            .saturating_add(other.escaped_kill_attempts);
        self.escaped_kill_failures = self
            .escaped_kill_failures
            .saturating_add(other.escaped_kill_failures);
        if other.escaped_kill_errno != 0 {
            self.escaped_kill_errno = other.escaped_kill_errno;
        }
        sum!(
            controller_cpu_ns,
            pipe_cpu_ns,
            clock_failures,
            charged_ns,
            debt_ns,
            descendants,
            verified_descendants,
            reap_user_ns,
            reap_system_ns,
            reap_quantization_ns,
            live_sample_count,
            wait4_reads
        );
        peak!(
            max_charge_ns,
            group_rss_peak,
            mixed_heap_rss_peak,
            observed_direct_cpu_ns,
            observed_descendant_cpu_ns,
            rolled_descendant_cpu_ns,
            job_cpu_ns,
            max_sample_gap_ns,
            reap_rss_bytes
        );
        if self.first_failure_line == 0 {
            self.first_failure_line = other.first_failure_line;
            self.first_failure_sample_stage = other.first_failure_sample_stage;
        }
        if other.group_kill_errno != 0 {
            self.group_kill_errno = other.group_kill_errno;
        }
        self.initial_pending_observations = self
            .initial_pending_observations
            .saturating_add(other.initial_pending_observations);
        if other.initial.stage != 0 {
            self.initial = other.initial;
        }
        self.reap_authorized |= other.reap_authorized;
        self.group_kill_attempted |= other.group_kill_attempted;
        self.group_kill_not_attempted_terminal |= other.group_kill_not_attempted_terminal;
        self.terminal_observations = self
            .terminal_observations
            .saturating_add(other.terminal_observations);
        if other.terminal_observations != 0 {
            self.terminal_last_guard = other.terminal_last_guard;
            self.terminal_waitid_result = other.terminal_waitid_result;
            self.terminal_waitid_errno = other.terminal_waitid_errno;
            self.terminal_waitid_pid = other.terminal_waitid_pid;
            self.terminal_waitid_code = other.terminal_waitid_code;
            self.terminal_bsd_result = other.terminal_bsd_result;
            self.terminal_bsd_errno = other.terminal_bsd_errno;
            self.terminal_census_result = other.terminal_census_result;
            self.terminal_census_errno = other.terminal_census_errno;
            self.terminal_member_pid = other.terminal_member_pid;
        }
        self.pause_stop_attempted |= other.pause_stop_attempted;
        if other.pause_stop_errno != 0 {
            self.pause_stop_errno = other.pause_stop_errno;
        }
        self.pause_ack_ns = self.pause_ack_ns.max(other.pause_ack_ns);
        if other.pause_user_checked
            && (!self.pause_user_checked || other.pause_user_excess_ns > self.pause_user_excess_ns)
        {
            self.pause_user_baseline_ns = other.pause_user_baseline_ns;
            self.pause_terminal_user_ns = other.pause_terminal_user_ns;
            self.pause_user_excess_ns = other.pause_user_excess_ns;
            self.pause_user_quantization_ns = other.pause_user_quantization_ns;
            self.pause_user_checked = true;
            self.pause_user_pass = other.pause_user_pass;
        }
        self.group_kill_raced_exit |= other.group_kill_raced_exit;
        self.qos_checked_mask |= other.qos_checked_mask;
        self.reap_rss_known = if self.merged {
            self.reap_rss_known && other.reap_rss_known
        } else {
            other.reap_rss_known
        };
        self.coverage_known = if self.merged {
            self.coverage_known && other.coverage_known
        } else {
            other.coverage_known
        };
        self.merged = true;
    }
}

/// Original unreaped leader pins the group. There are no external grant clocks,
/// window endpoints, adaptive slices, or terminal conservation allowances.
pub(crate) struct HelperGroup {
    pub(crate) pgid: i32,
    pub(crate) identity: super::resolve::HelperIdentity,
    pub(crate) running: bool,
    pub(crate) retiring: bool,
    pub(crate) ready: bool,
    pub(crate) confirmed_stopped: bool,
    pub(crate) debt_until: Instant,
    pub(crate) metrics: HelperMetrics,
    members: [HelperMember; 2],
    retired: [RetiredMember; 8],
    timebase: HelperTimebase,
    pipe_open: bool,
    payload_admitted: bool,
    creation_settled: bool,
    failed: bool,
    pause_started: Option<Instant>,
    pause_failed: bool,
    last_sample: Instant,
    final_kill_attempted: bool,
    terminal_pending: bool,
    exited: bool,
    pause_ack: Option<Duration>,
    pause_user_baseline: Option<u64>,
    own_wait4: bool,
    reap_authorized: bool,
    group_gone: bool,
    terminal_usage: Option<super::resolve::HelperReapUsage>,
    terminal_pass: bool,
}
impl HelperGroup {
    pub(crate) fn attached(identity: super::resolve::HelperIdentity) -> Self {
        let now = Instant::now();
        Self {
            pgid: identity.pgid,
            identity,
            running: false,
            retiring: false,
            ready: false,
            confirmed_stopped: false,
            debt_until: now,
            metrics: HelperMetrics {
                coverage_known: true,
                ..HelperMetrics::default()
            },
            members: [HelperMember::default(); 2],
            retired: [RetiredMember::default(); 8],
            timebase: helper_timebase().unwrap_or_default(),
            pipe_open: false,
            payload_admitted: false,
            creation_settled: false,
            failed: false,
            pause_started: None,
            pause_failed: false,
            last_sample: now.checked_sub(HELPER_SAMPLE_INTERVAL).unwrap_or(now),
            final_kill_attempted: false,
            terminal_pending: false,
            exited: false,
            pause_ack: None,
            pause_user_baseline: None,
            own_wait4: false,
            reap_authorized: false,
            group_gone: false,
            terminal_usage: None,
            terminal_pass: false,
        }
    }
    #[track_caller]
    fn reject(&mut self, control: &Control, resource: bool) {
        self.failed = true;
        self.ready = false;
        if self.metrics.first_failure_line == 0 {
            self.metrics.first_failure_line = std::panic::Location::caller().line();
        }
        if resource {
            control
                .accounting
                .resource_failed
                .store(true, Ordering::Release);
            let _ = control
                .cancel
                .compare_exchange(0, 4, Ordering::AcqRel, Ordering::Acquire);
        } else {
            self.metrics.coverage_known = false;
            control
                .cancel
                .compare_exchange(0, 5, Ordering::AcqRel, Ordering::Acquire)
                .ok();
        }
    }
    pub(crate) fn exit_observed(&self) -> bool {
        self.exited
    }
    pub(crate) fn payload_input_blocked(&self) -> bool {
        self.retiring
            || self.failed
            || self.terminal_pending
            || self.own_wait4
            || self.exited
            || self.pause_started.is_some()
    }
    pub(crate) fn pipe_is_open(&self) -> bool {
        self.pipe_open
    }
    pub(crate) fn payload_pipe_is_open(&self) -> bool {
        self.pipe_open && self.payload_admitted && !self.payload_input_blocked()
    }
    pub(crate) fn begin_pipe(&mut self) -> Result<(), ()> {
        if self.pipe_open || !self.creation_settled || self.metrics.qos_checked_mask & 1 == 0 {
            return Err(());
        }
        self.pipe_open = true;
        self.payload_admitted = !self.payload_input_blocked();
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn begin_pipe_accounting(&mut self) -> Result<(), ()> {
        if self.pipe_open {
            return Err(());
        }
        self.pipe_open = true;
        self.payload_admitted = false;
        Ok(())
    }
    pub(crate) fn account_pipe(
        &mut self,
        control: &Control,
        cpu: u64,
        wall: u64,
    ) -> Result<(), ()> {
        if !self.pipe_open {
            return Err(());
        }
        self.pipe_open = false;
        self.payload_admitted = false;
        self.metrics.pipe_cpu_ns = self.metrics.pipe_cpu_ns.saturating_add(cpu);
        if cpu == u64::MAX || wall == u64::MAX {
            self.metrics.clock_failures += 1;
            self.metrics.coverage_known = false;
            control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
            self.reject(control, true);
            return Err(());
        }
        if cpu > UNIT_NS {
            self.reject(control, true);
            return Err(());
        }
        Ok(())
    }
    pub(crate) fn account_controller(&mut self, control: &Control, cpu: u64) -> Result<(), ()> {
        self.metrics.controller_cpu_ns = self.metrics.controller_cpu_ns.saturating_add(cpu);
        if cpu == u64::MAX {
            self.metrics.clock_failures += 1;
            self.metrics.coverage_known = false;
            control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
            self.reject(control, true);
            return Err(());
        }
        if cpu > UNIT_NS {
            self.reject(control, true);
            return Err(());
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn retain(&mut self, member: RetiredMember) -> Result<(), ()> {
        if self.retired.iter().any(|m| {
            m.pid == member.pid && m.birth == member.birth && m.token.value == member.token.value
        }) {
            return Ok(());
        }
        let index = self.retired.iter().position(|m| m.pid == 0).ok_or(())?;
        for old in &mut self.retired {
            if old.pid == member.pid {
                old.current = false;
            }
        }
        self.retired[index] = member;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn census(&mut self) -> Result<([i32; 3], usize), ()> {
        let mut pids = [0i32; 3];
        helper_clear_errno();
        let count = unsafe {
            libc::proc_listpgrppids(
                self.pgid,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as i32,
            )
        };
        let errno = io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(libc::EIO);
        self.metrics.terminal_census_result = count;
        self.metrics.terminal_census_errno = errno;
        if !helper_census_known(count, errno)
            || count as usize >= pids.len()
            || count as usize > usize::from(self.identity.expected_members)
        {
            return Err(());
        }
        Ok((pids, count as usize))
    }
    fn retained_pause_matches(&self, prior: HelperMember, user: u64, stopped: bool) -> bool {
        let acknowledged_pause = self.pause_started.is_some() && self.pause_ack.is_some();
        if acknowledged_pause && prior.pid == 0 {
            return false;
        }
        if self.confirmed_stopped || acknowledged_pause {
            prior
                .paused_user
                .is_none_or(|expected| expected == user && stopped)
        } else {
            true
        }
    }
    #[cfg(target_os = "macos")]
    fn observe_member(&mut self, control: &Control, pid: i32) -> Result<HelperMember, ()> {
        use std::os::unix::ffi::OsStrExt;
        let leader = pid == self.identity.pid;
        let initial = leader && !self.creation_settled;
        let before = if initial {
            self.metrics.initial = HelperInitialObservation {
                stage: 1,
                timebase_numer: self.timebase.numer,
                timebase_denom: self.timebase.denom,
                ..HelperInitialObservation::default()
            };
            let mut bsd = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
            let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
            let result = unsafe {
                libc::proc_pidinfo(
                    pid,
                    libc::PROC_PIDTBSDINFO,
                    0,
                    bsd.as_mut_ptr().cast(),
                    size,
                )
            };
            self.metrics.initial.bsd_result = result;
            self.metrics.initial.bsd_errno = if result == size {
                0
            } else {
                io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO)
            };
            if result != size {
                return Err(());
            }
            self.metrics.initial.valid |= 1;
            let info = unsafe { bsd.assume_init() };
            self.metrics.initial.pid = info.pbi_pid;
            self.metrics.initial.ppid = info.pbi_ppid;
            self.metrics.initial.pgid = info.pbi_pgid;
            self.metrics.initial.uid = info.pbi_uid;
            self.metrics.initial.status = info.pbi_status;
            self.metrics.initial.birth = helper_birth(&info)?;
            info
        } else {
            helper_bsd(pid)?
        };
        let birth = helper_birth(&before)?;
        if before.pbi_pid != pid as u32
            || before.pbi_pgid != self.pgid as u32
            || before.pbi_uid != unsafe { libc::geteuid() }
        {
            return Err(());
        }
        if leader
            && (birth != self.identity.start_identity
                || before.pbi_ppid != unsafe { libc::getpid() } as u32)
        {
            return Err(());
        }
        if !leader && before.pbi_ppid != self.identity.pid as u32 {
            return Err(());
        }
        let prior = self.members[usize::from(!leader)];
        if prior.pid != 0 && (prior.pid != pid || prior.birth != birth) {
            return Err(());
        }
        if before.pbi_status == libc::SZOMB {
            return Ok(HelperMember {
                pid,
                birth,
                live: false,
                stopped: false,
                ..prior
            });
        }
        let retained = helper_audit_member(pid, before.pbi_ppid as i32)?;
        self.retain(retained)?; // retain before image/QoS/CPU verdicts
        if initial {
            self.metrics.initial.stage = 2;
            self.metrics.initial.valid |= 2;
        }
        let mut task = std::mem::MaybeUninit::<libc::proc_taskinfo>::uninit();
        let size = std::mem::size_of::<libc::proc_taskinfo>() as i32;
        let result = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTASKINFO,
                0,
                task.as_mut_ptr().cast(),
                size,
            )
        };
        if initial {
            self.metrics.initial.stage = 3;
            self.metrics.initial.task_result = result;
            self.metrics.initial.task_errno = if result == size {
                0
            } else {
                io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO)
            };
        }
        if result != size {
            return Err(());
        }
        let task = unsafe { task.assume_init() };
        if initial {
            self.metrics.initial.valid |= 4;
            self.metrics.initial.threads = task.pti_threadnum;
            self.metrics.initial.running = task.pti_numrunning;
            self.metrics.initial.priority = task.pti_priority;
        }
        if task.pti_priority != 4
            || task.pti_threadnum != 1
            || !(0..=1).contains(&task.pti_numrunning)
        {
            self.reject(control, true);
            return Err(());
        }
        let ri_index = usize::from(!leader);
        let ri_started = Instant::now();
        self.metrics.ri_start_valid_mask &= !(1 << ri_index);
        self.metrics.ri_end_valid_mask &= !(1 << ri_index);
        if let Some(previous) = prior.ri_started {
            self.metrics.ri_previous_start_ns[ri_index] = self.ri_clock_ns(control, previous)?;
            self.metrics.ri_previous_valid_mask |= 1 << ri_index;
        } else {
            self.metrics.ri_previous_valid_mask &= !(1 << ri_index);
        }
        self.metrics.ri_start_ns[ri_index] = self.ri_clock_ns(control, ri_started)?;
        self.metrics.ri_start_valid_mask |= 1 << ri_index;
        if !self.record_member_gap(ri_index, ri_started) {
            self.reject(control, true);
            let _ = self.retire_members();
            return Err(());
        }
        let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v2>::uninit();
        let result = unsafe { libc::proc_pid_rusage(pid, 2, usage.as_mut_ptr().cast()) };
        let ri_ended = Instant::now();
        self.metrics.ri_end_ns[ri_index] = self.ri_clock_ns(control, ri_ended)?;
        self.metrics.ri_end_valid_mask |= 1 << ri_index;
        if !self.record_member_gap(ri_index, ri_ended) {
            self.reject(control, true);
            let _ = self.retire_members();
            return Err(());
        }

        if initial {
            self.metrics.initial.stage = 4;
            self.metrics.initial.ri_result = result;
            self.metrics.initial.ri_errno = if result == 0 {
                0
            } else {
                io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO)
            };
        }
        if result != 0 {
            return Err(());
        }
        let usage = unsafe { usage.assume_init() };
        if initial {
            self.metrics.initial.valid |= 8;
            self.metrics.initial.user_ticks = usage.ri_user_time;
            self.metrics.initial.system_ticks = usage.ri_system_time;
            self.metrics.initial.start = usage.ri_proc_start_abstime;
        }
        if usage.ri_proc_start_abstime == 0
            || (prior.start != 0 && usage.ri_proc_start_abstime != prior.start)
        {
            return Err(());
        }
        let mut path = [0u8; 4096];
        let n = unsafe {
            proc_pidpath_audittoken(&retained.token, path.as_mut_ptr().cast(), path.len() as u32)
        };
        if n <= 0 || n as usize >= path.len() {
            return Err(());
        }
        let length = path.iter().position(|&b| b == 0).ok_or(())?;
        let image = &path[..length];
        let expected = if leader {
            self.identity.image.as_path()
        } else {
            self.identity.descendant_image.as_deref().ok_or(())?
        };
        let verified = image == expected.as_os_str().as_bytes();
        if initial {
            self.metrics.initial.stage = 5;
            if verified {
                self.metrics.initial.valid |= 16;
            }
        }
        // A source-qualified fork may briefly retain the leader image. It is
        // not a verified verifier and may not be treated as one at settlement.
        if !verified
            && (leader || prior.verified || image != self.identity.image.as_os_str().as_bytes())
        {
            return Err(());
        }
        let after = helper_bsd(pid)?;
        if initial {
            self.metrics.initial.stage = 6;
            self.metrics.initial.after_status = after.pbi_status;
        }
        if helper_birth(&after)? != birth
            || after.pbi_ppid != before.pbi_ppid
            || after.pbi_pgid != before.pbi_pgid
        {
            return Err(());
        }
        if initial {
            self.metrics.initial.valid |= 32;
        }
        let cpu = helper_convert_ticks(
            usage
                .ri_user_time
                .checked_add(usage.ri_system_time)
                .ok_or(())?,
            self.timebase,
        )?;
        let rolled = helper_convert_ticks(
            usage
                .ri_child_user_time
                .checked_add(usage.ri_child_system_time)
                .ok_or(())?,
            self.timebase,
        )?;
        if cpu < prior.cpu
            || rolled < prior.rolled
            || !self.retained_pause_matches(
                prior,
                usage.ri_user_time,
                after.pbi_status == libc::SSTOP && task.pti_numrunning == 0,
            )
        {
            return Err(());
        }
        self.metrics.qos_checked_mask |= if leader { 1 } else { 2 };
        Ok(HelperMember {
            pid,
            birth,
            start: usage.ri_proc_start_abstime,
            user: usage.ri_user_time,
            system: usage.ri_system_time,
            cpu,
            rolled,
            rolled_user: usage.ri_child_user_time,
            terminal_ri_at: prior.terminal_ri_at,
            rss: usage.ri_resident_size,
            live: true,
            stopped: after.pbi_status == libc::SSTOP && task.pti_numrunning == 0,
            verified,
            paused_user: prior.paused_user,
            ri_started: Some(ri_started),
        })
    }
    fn ri_clock_ns(&mut self, control: &Control, at: Instant) -> Result<u64, ()> {
        let value = at
            .checked_duration_since(control.accounting.origin)
            .and_then(|d| u64::try_from(d.as_nanos()).ok());
        if let Some(value) = value {
            return Ok(value);
        }
        self.metrics.clock_failures = self.metrics.clock_failures.saturating_add(1);
        self.metrics.coverage_known = false;
        control
            .accounting
            .clock_failed
            .store(true, Ordering::Release);
        Err(())
    }
    fn record_member_gap(&mut self, index: usize, now: Instant) -> bool {
        if !self.creation_settled || self.own_wait4 {
            return true;
        }
        let member = self.members[index];
        if member.pid == 0 || !member.live {
            return true;
        }
        let gap = member
            .ri_started
            .and_then(|before| now.checked_duration_since(before));
        let ns = gap
            .and_then(|d| u64::try_from(d.as_nanos()).ok())
            .unwrap_or(u64::MAX);
        self.metrics.ri_gap_ns[index] = ns;
        self.metrics.max_sample_gap_ns = self.metrics.max_sample_gap_ns.max(ns);
        gap.is_some_and(|value| value <= HELPER_SAMPLE_LIMIT)
    }
    fn record_sample_gap(&mut self, now: Instant) -> bool {
        // This early check is conservative; the decisive same-member check is
        // repeated at actual RI begin and return before its prior stamp changes.
        let mut pass = true;
        for index in 0..2 {
            pass &= self.record_member_gap(index, now);
        }
        pass
    }
    #[cfg(target_os = "macos")]
    fn sample_inner(&mut self, control: &Control) -> Result<bool, ()> {
        if self.own_wait4 {
            return Ok(true);
        }
        let sample_started = Instant::now();
        if !self.record_sample_gap(sample_started) {
            self.reject(control, true);
            let _ = self.retire_members();
            return Err(());
        }
        if self.original_exited() {
            self.running = false;
            self.exited = true;
            return Ok(true);
        }
        let (pids, count) = self.census()?;
        let mut all_stopped = true;
        let mut rss_sum = 0u64;
        for &pid in &pids[..count] {
            let index = usize::from(pid != self.identity.pid);
            let member = self.observe_member(control, pid)?;
            if index == 1 && self.members[1].pid == 0 {
                self.metrics.descendants += 1;
            }
            if index == 1 && member.verified && !self.members[1].verified {
                self.metrics.verified_descendants += 1;
            }
            rss_sum = rss_sum.checked_add(member.rss).ok_or(())?;
            all_stopped &= !member.live || member.stopped;
            self.members[index] = member;
        }
        let exited = super::resolve::helper_exited(self.identity.pid).map_err(|_| ())?;
        if !pids[..count].contains(&self.identity.pid) && !exited {
            return Err(());
        }
        if exited {
            self.running = false;
            self.exited = true;
        }
        self.metrics.live_sample_count += 1;
        self.metrics.observed_direct_cpu_ns = self.members[0].cpu;
        self.metrics.observed_descendant_cpu_ns = self.members[1].cpu;
        self.metrics.rolled_descendant_cpu_ns = self.members[0].rolled;
        self.metrics.group_rss_peak = self.metrics.group_rss_peak.max(rss_sum);
        let mixed = heap().checked_add(rss_sum).ok_or(())?;
        self.metrics.mixed_heap_rss_peak = self.metrics.mixed_heap_rss_peak.max(mixed);
        let cpu = self.members[0]
            .cpu
            .checked_add(self.members[0].rolled.max(self.members[1].cpu))
            .ok_or(())?;
        let job = control
            .accounting
            .helper_job_cpu
            .load(Ordering::Acquire)
            .checked_add(cpu)
            .ok_or(())?;
        self.metrics.job_cpu_ns = job;
        if cpu > HELPER_COMMAND_CPU_NS
            || job > HELPER_JOB_CPU_NS
            || rss_sum > storage::HELPERS
            || mixed > HEAP_MAX
        {
            self.reject(control, true);
            let _ = self.retire_members();
            return Err(());
        }
        if self.known_retained_child() {
            let _ = self.try_child_rollup();
        }
        self.last_sample = sample_started;
        if !all_stopped {
            self.confirmed_stopped = false;
        }
        if all_stopped && !exited {
            self.confirmed_stopped = true;
            for member in &mut self.members {
                if member.live {
                    member.paused_user = Some(member.user);
                }
            }
            self.running = false;
        }
        Ok(exited || all_stopped)
    }
    pub(crate) fn sample(&mut self, control: &Control) -> Result<bool, ()> {
        #[cfg(target_os = "macos")]
        {
            let result = self.sample_inner(control);
            if result.is_err() && !self.failed {
                self.reject(control, false);
            }
            result
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.reject(control, false);
            Err(())
        }
    }
    fn positive_original(&self) -> bool {
        self.pgid > 0
            && self.identity.pid == self.pgid
            && self.identity.pgid == self.pgid
            && self.identity.start_identity != 0
            && !self.own_wait4
    }
    fn final_kill_no_attempt_allowed(original_terminal: bool, census_terminal: bool) -> bool {
        original_terminal && census_terminal
    }
    fn token_cleanup_required(current: bool, birth_matches: bool, outside_group: bool) -> bool {
        current && birth_matches && outside_group
    }
    #[cfg(target_os = "macos")]
    fn group_observed_terminal(&mut self) -> bool {
        let Ok((pids, count)) = self.census() else {
            return false;
        };
        for &pid in &pids[..count] {
            if pid == self.identity.pid {
                continue;
            } // fresh positive own WNOWAIT
            let Some(member) = self
                .retired
                .iter()
                .find(|m| m.pid == pid && m.current)
                .copied()
            else {
                return false;
            };
            if !self.retained_member_exited(member) {
                return false;
            }
        }
        true
    }
    fn signal_group(&mut self, signal: i32) -> Result<(), i32> {
        // Issuing Ready seals effects under the same owner mutex BEFORE the
        // consuming wait4 may run outside it. The later usage callback is not
        // the first point at which reaping can have released the original pin.
        if self.reap_authorized || self.own_wait4 {
            return Err(libc::EALREADY);
        }
        if !self.positive_original() {
            return Err(libc::EINVAL);
        }
        #[cfg(target_os = "macos")]
        {
            let original_terminal = self.original_exited();
            if original_terminal {
                if signal != libc::SIGKILL {
                    return Err(libc::ESRCH);
                }
                let group_terminal = self.group_observed_terminal();
                if Self::final_kill_no_attempt_allowed(original_terminal, group_terminal) {
                    self.final_kill_attempted = true;
                    self.metrics.group_kill_not_attempted_terminal = true;
                    self.terminal_pending = true;
                    return Ok(());
                }
                self.metrics.coverage_known = false;
                self.metrics.child_cost_unproven = true;
                // Positive own WNOWAIT plus the retained unreaped incarnation
                // still pins this group. A live/unknown child must be killed;
                // the leader's later wait4 cannot certify an unrolled child.
            } else if !helper_bsd(self.identity.pid).is_ok_and(|info| {
                helper_birth(&info) == Ok(self.identity.start_identity)
                    && info.pbi_pid == self.identity.pid as u32
                    && info.pbi_ppid == unsafe { libc::getpid() } as u32
                    && info.pbi_pgid == self.pgid as u32
                    && info.pbi_uid == unsafe { libc::geteuid() }
            }) {
                return Err(libc::ESTALE);
            }
            if signal == libc::SIGKILL {
                if self.members[1].pid > 0 && self.members[1].live {
                    self.metrics.child_cost_unproven = true;
                    self.metrics.coverage_known = false;
                }
                self.final_kill_attempted = true;
                self.metrics.group_kill_attempted = true;
            }
            if signal == libc::SIGSTOP {
                self.metrics.pause_stop_attempted = true;
            }
            let result = unsafe { libc::kill(-self.pgid, signal) };
            let errno = if result == 0 {
                0
            } else {
                io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO)
            };
            if signal == libc::SIGKILL {
                self.metrics.group_kill_errno = errno;
            }
            if signal == libc::SIGSTOP {
                self.metrics.pause_stop_errno = errno;
            }
            if result == 0 { Ok(()) } else { Err(errno) }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = signal;
            Err(libc::ENOTSUP)
        }
    }
    fn known_retained_child(&self) -> bool {
        self.members[1].pid > 0
            || self
                .retired
                .iter()
                .any(|r| r.pid > 0 && r.pid != self.identity.pid)
    }
    fn rollup_counters_match(
        child_user: u64,
        child_system: u64,
        rolled_user: u64,
        rolled_system: u64,
    ) -> bool {
        (rolled_user != 0 || rolled_system != 0)
            && rolled_user >= child_user
            && rolled_system >= child_system
    }
    #[cfg(target_os = "macos")]
    fn try_child_rollup(&mut self) -> bool {
        if self.metrics.child_rollup_proven {
            return true;
        }
        if !self.positive_original()
            || self.identity.expected_members != 2
            || self.identity.descendant_image.is_none()
            || self.metrics.descendants != 1
            || self.metrics.verified_descendants != 1
        {
            return false;
        }
        let mut child = self.members[1];
        let leader = self.members[0];
        if !child.verified
            || child.pid <= 0
            || child.start == 0
            || leader.pid != self.identity.pid
            || leader.birth != self.identity.start_identity
            || leader.start == 0
        {
            return false;
        }
        if self.retired.iter().any(|r| {
            r.pid > 0
                && r.pid != self.identity.pid
                && (r.pid != child.pid || r.birth != child.birth)
        }) {
            return false;
        }
        let Some(token) = self
            .retired
            .iter()
            .find(|r| r.pid == child.pid && r.birth == child.birth && r.current)
            .map(|r| r.token)
        else {
            return false;
        };
        if helper_token_exited(&token) != Ok(true) {
            return false;
        }
        // This is mandatory terminal proof, not a normal sampler reset. An
        // unavailable query preserves the original terminal observation.
        let mut terminal = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
        if unsafe { libc::proc_pid_rusage(child.pid, 1, terminal.as_mut_ptr().cast()) } == 0 {
            let terminal = unsafe { terminal.assume_init() };
            if terminal.ri_proc_start_abstime != child.start
                || terminal.ri_proc_exit_abstime == 0
                || terminal.ri_user_time < child.user
                || terminal.ri_system_time < child.system
            {
                return false;
            }
            child.user = terminal.ri_user_time;
            child.system = terminal.ri_system_time;
            child.terminal_ri_at = Some(Instant::now());
            child.live = false;
            self.members[1] = child;
        }
        let Some(exited_at) = child.terminal_ri_at else {
            return false;
        };
        let mut children = [0i32; 2];
        helper_clear_errno();
        let count = unsafe {
            libc::proc_listchildpids(
                self.identity.pid,
                children.as_mut_ptr().cast(),
                std::mem::size_of_val(&children) as i32,
            )
        };
        let errno = io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(libc::EIO);
        self.metrics.child_edge_result = count;
        self.metrics.child_edge_errno = errno;
        if !helper_census_empty(count, errno) {
            return false;
        }
        self.metrics.child_edge_closed = true;
        let read_at = Instant::now();
        let mut rolled = std::mem::MaybeUninit::<libc::rusage_info_v2>::uninit();
        if unsafe { libc::proc_pid_rusage(self.identity.pid, 2, rolled.as_mut_ptr().cast()) } != 0 {
            return false;
        }
        let rolled = unsafe { rolled.assume_init() };
        if rolled.ri_proc_start_abstime != leader.start || read_at < exited_at {
            return false;
        }
        self.metrics.child_rollup_read_after_exit = true;
        self.metrics.child_terminal_user_ticks = child.user;
        self.metrics.child_terminal_system_ticks = child.system;
        self.metrics.leader_child_user_ticks = rolled.ri_child_user_time;
        self.metrics.leader_child_system_ticks = rolled.ri_child_system_time;
        self.metrics.child_rollup_proven = Self::rollup_counters_match(
            child.user,
            child.system,
            rolled.ri_child_user_time,
            rolled.ri_child_system_time,
        );
        self.metrics.child_rollup_proven
    }
    #[cfg(not(target_os = "macos"))]
    fn try_child_rollup(&mut self) -> bool {
        false
    }
    fn record_pause_ack(&mut self, elapsed: Duration) -> Result<(), ()> {
        // Nonduplicated user counters, captured post-positive-ACK. A reaped
        // child rollup replaces its live counter; it is never added twice.
        let direct = helper_convert_ticks(self.members[0].user, self.timebase)?;
        let child = helper_convert_ticks(
            self.members[1].user.max(self.members[0].rolled_user),
            self.timebase,
        )?;
        let user = direct.checked_add(child).ok_or(())?;
        self.pause_user_baseline = Some(user);
        self.metrics.pause_user_baseline_ns = user;
        self.pause_ack = Some(elapsed);
        self.metrics.pause_ack_ns = u64::try_from(elapsed.as_nanos()).map_err(|_| ())?;
        Ok(())
    }
    fn terminal_pause_user(&mut self, user: u64) -> bool {
        let Some(before) = self.pause_user_baseline else {
            return true;
        };
        self.metrics.pause_user_checked = true;
        self.metrics.pause_user_baseline_ns = before;
        self.metrics.pause_terminal_user_ns = user;
        self.metrics.pause_user_excess_ns = user.saturating_sub(before);
        // wait4's user field has microsecond resolution. This is solely its
        // one-microsecond observer rounding, never free CPU or a T allowance.
        self.metrics.pause_user_quantization_ns = 1_000;
        self.metrics.pause_user_pass =
            user != u64::MAX && before.checked_add(1_000).is_some_and(|upper| user <= upper);
        self.metrics.pause_user_pass
    }
    fn initial_kernel_tail_pending(&self) -> bool {
        let i = self.metrics.initial;
        let m = self.members[0];
        !self.creation_settled
            && !self.running
            && !self.retiring
            && !self.failed
            && self.original_retained()
            && i.valid == 63
            && i.pid == self.identity.pid as u32
            && i.pgid == self.pgid as u32
            && i.birth == self.identity.start_identity
            && i.start != 0
            && i.start == m.start
            && i.status == libc::SSTOP
            && i.after_status == libc::SSTOP
            && i.threads == 1
            && i.running == 1
            && i.priority == 4
            && i.user_ticks == 0
            && m.live
            && m.verified
            && !m.stopped
    }
    /// false is a retained, positively identified SSTOP/kernel-tail observation,
    /// not admission. The caller pays this complete unit before re-observing.
    pub(crate) fn settle_creation(&mut self, control: &Control) -> Result<bool, ()> {
        if self.creation_settled {
            return if self.failed { Err(()) } else { Ok(true) };
        }
        self.metrics.initial.readiness = -1;
        if control.error().is_some() {
            return Err(());
        }
        match self.sample(control) {
            Ok(true) => {}
            Ok(false) if self.initial_kernel_tail_pending() => {
                self.metrics.initial.readiness = 0;
                self.metrics.initial_pending_observations =
                    self.metrics.initial_pending_observations.saturating_add(1);
                return Ok(false);
            }
            Ok(false) => {
                self.metrics.initial.readiness = -1;
                self.reject(control, false);
                return Err(());
            }
            Err(()) => {
                self.metrics.initial.readiness = -1;
                return Err(());
            }
        }
        if self.metrics.qos_checked_mask & 1 == 0
            || !self.confirmed_stopped
            || self.members[0].birth != self.identity.start_identity
            || control.error().is_some()
        {
            self.metrics.initial.readiness = -1;
            return Err(());
        }
        self.creation_settled = true;
        if control.foreground.load(Ordering::Acquire) {
            self.pause_started = Some(Instant::now());
            if self.record_pause_ack(Duration::ZERO).is_err() {
                self.metrics.initial.readiness = -1;
                self.reject(control, true);
                return Err(());
            }
        } else {
            if self.signal_group(libc::SIGCONT).is_err() {
                self.metrics.initial.readiness = -1;
                self.reject(control, false);
                return Err(());
            }
            self.confirmed_stopped = false;
            self.running = true;
            for member in &mut self.members {
                member.paused_user = None;
            }
        }
        self.metrics.initial.readiness = 1;
        Ok(true)
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn retire_members(&mut self) -> Result<(), ()> {
        self.retiring = true;
        self.ready = false;
        self.pipe_open = false;
        self.payload_admitted = false;
        if self.own_wait4 || self.reap_authorized {
            return Ok(());
        } // no group OR escaped-token effects after Ready/own reap
        if self.final_kill_attempted {
            return Ok(());
        }
        if !self.positive_original() {
            self.metrics.coverage_known = false;
            return Err(());
        }
        // Retain escaped recipients as effects before any refusal; only exact
        // audit-token signals may address them outside the pinned group.
        let mut children = [0i32; 8];
        helper_clear_errno();
        let count = unsafe {
            libc::proc_listchildpids(
                self.identity.pid,
                children.as_mut_ptr().cast(),
                std::mem::size_of_val(&children) as i32,
            )
        };
        let errno = io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(libc::EIO);
        self.metrics.child_edge_result = count;
        self.metrics.child_edge_errno = errno;
        if !helper_census_known(count, errno) || count as usize >= children.len() {
            self.metrics.coverage_known = false;
            if self.identity.expected_members == 2 {
                self.metrics.child_cost_unproven = true;
            }
        }
        if helper_census_known(count, errno) && (count as usize) < children.len() {
            for &pid in &children[..count as usize] {
                if let Ok(member) = helper_audit_member(pid, self.identity.pid) {
                    let _ = self.retain(member);
                }
            }
        }
        if self.known_retained_child() {
            let _ = self.try_child_rollup();
        }
        let result = self.signal_group(libc::SIGKILL);
        if !self.metrics.group_kill_attempted && !self.metrics.group_kill_not_attempted_terminal {
            self.metrics.coverage_known = false;
            return Err(());
        }
        self.terminal_pending |= matches!(self.metrics.group_kill_errno, libc::EPERM | libc::ESRCH);
        for member in self.retired.iter().filter(|m| m.pid > 0 && m.current) {
            if helper_bsd(member.pid).is_ok_and(|info| {
                Self::token_cleanup_required(
                    member.current,
                    helper_birth(&info) == Ok(member.birth),
                    info.pbi_pgid != self.pgid as u32,
                )
            }) {
                self.metrics.coverage_known = false;
                self.metrics.child_cost_unproven = true;
                self.metrics.escaped_kill_attempts =
                    self.metrics.escaped_kill_attempts.saturating_add(1);
                if let Err(errno) = helper_token_signal(&member.token, libc::SIGKILL) {
                    self.metrics.escaped_kill_failures =
                        self.metrics.escaped_kill_failures.saturating_add(1);
                    self.metrics.escaped_kill_errno = errno;
                }
            }
        }
        if result.is_ok() || self.terminal_pending {
            Ok(())
        } else {
            Err(())
        }
    }
    #[cfg(not(target_os = "macos"))]
    pub(crate) fn retire_members(&mut self) -> Result<(), ()> {
        self.retiring = true;
        Err(())
    }
    fn original_retained(&self) -> bool {
        if !self.positive_original() {
            return false;
        }
        let m = self.members[0];
        let live_pin = m.pid == self.identity.pid
            && m.birth == self.identity.start_identity
            && m.start != 0
            && m.verified
            && self.retired.iter().any(|r| {
                r.pid == m.pid
                    && r.birth == m.birth
                    && r.current
                    && r.token.value[5] == m.pid as u32
            });
        // Failed attachment still owns the exact typed native birth. No CONT
        // was issued by this owner; terminal CPU will come from its own wait4,
        // and missing QoS/admission can never turn this cleanup into trust.
        live_pin || (!self.creation_settled && !self.running && m.pid == 0)
    }
    #[cfg(target_os = "macos")]
    fn original_exited(&mut self) -> bool {
        if self.own_wait4 {
            return true;
        }
        self.metrics.terminal_observations = self.metrics.terminal_observations.saturating_add(1);
        self.metrics.terminal_last_guard = 1;
        if !self.original_retained() {
            return false;
        }
        let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                self.identity.pid as libc::id_t,
                info.as_mut_ptr(),
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        self.metrics.terminal_waitid_result = result;
        self.metrics.terminal_waitid_errno = if result == 0 {
            0
        } else {
            io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EIO)
        };
        self.metrics.terminal_waitid_pid = 0;
        self.metrics.terminal_waitid_code = 0;
        self.metrics.terminal_last_guard = 2;
        if result != 0 {
            return false;
        }
        let info = unsafe { info.assume_init() };
        self.metrics.terminal_waitid_pid = info.si_pid;
        self.metrics.terminal_waitid_code = info.si_code;
        self.metrics.terminal_last_guard = 3;
        if info.si_pid != self.identity.pid
            || !matches!(
                info.si_code,
                libc::CLD_EXITED | libc::CLD_KILLED | libc::CLD_DUMPED
            )
        {
            return false;
        }
        let mut bsd = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
        let result = unsafe {
            libc::proc_pidinfo(
                self.identity.pid,
                libc::PROC_PIDTBSDINFO,
                0,
                bsd.as_mut_ptr().cast(),
                size,
            )
        };
        self.metrics.terminal_bsd_result = result;
        self.metrics.terminal_bsd_errno = if result == size {
            0
        } else {
            io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EIO)
        };
        self.metrics.terminal_last_guard = 4;
        if result == size {
            let bsd = unsafe { bsd.assume_init() };
            if bsd.pbi_pid != self.identity.pid as u32
                || bsd.pbi_ppid != unsafe { libc::getpid() } as u32
                || bsd.pbi_uid != unsafe { libc::geteuid() }
                || bsd.pbi_pgid != self.pgid as u32
                || helper_birth(&bsd) != Ok(self.identity.start_identity)
                || bsd.pbi_status != libc::SZOMB
            {
                return false;
            }
            self.metrics.terminal_last_guard = 5;
        } else {
            // Positive OWN WNOWAIT + original unreaped incarnation pin supplies
            // exit. Query absence supplies no endpoint or new signaling right.
            self.metrics.terminal_last_guard = 6;
        }
        self.exited = true;
        self.running = false;
        self.members[0].live = false;
        true
    }
    #[cfg(not(target_os = "macos"))]
    fn original_exited(&mut self) -> bool {
        false
    }
    #[cfg(target_os = "macos")]
    fn retained_member_exited(&self, member: RetiredMember) -> bool {
        let Some(known) = self
            .members
            .iter()
            .find(|m| m.pid == member.pid && m.birth == member.birth && m.start != 0)
        else {
            return false;
        };
        if helper_token_exited(&member.token) != Ok(true) {
            return false;
        }
        let bsd = helper_bsd(member.pid);
        if self.metrics.child_rollup_proven && known.terminal_ri_at.is_some() {
            return match bsd {
                Ok(info) => {
                    helper_birth(&info) == Ok(member.birth) && info.pbi_status == libc::SZOMB
                }
                Err(()) => true,
            };
        }
        if let Ok(ref info) = bsd {
            if helper_birth(info) != Ok(member.birth) || info.pbi_status != libc::SZOMB {
                return false;
            }
            // Positive same-birth zombie + original token exit, not absence.
            return true;
        }
        let mut usage = std::mem::MaybeUninit::<libc::rusage_info_v1>::uninit();
        if unsafe { libc::proc_pid_rusage(member.pid, 1, usage.as_mut_ptr().cast()) } != 0 {
            return false;
        }
        let usage = unsafe { usage.assume_init() };
        usage.ri_proc_start_abstime == known.start && usage.ri_proc_exit_abstime != 0
    }
    pub(crate) fn observe_post_exit(&mut self, _: &Control) -> Result<bool, ()> {
        Ok(self.original_exited())
    }
    pub(crate) fn terminal_reap_readiness(
        &mut self,
        _: &Control,
        ticket: super::resolve::HelperTicket,
    ) -> super::resolve::HelperReapReadiness {
        if self.own_wait4 || ticket != self.identity.ticket() || !self.original_exited() {
            return super::resolve::HelperReapReadiness::Held;
        }
        #[cfg(target_os = "macos")]
        {
            let Ok((pids, count)) = self.census() else {
                return super::resolve::HelperReapReadiness::Held;
            };
            for &pid in &pids[..count] {
                if pid == self.identity.pid {
                    continue;
                } // SAME positive own WNOWAIT above
                self.metrics.terminal_member_pid = pid;
                let Some(member) = self
                    .retired
                    .iter()
                    .find(|m| m.pid == pid && m.current)
                    .copied()
                else {
                    self.metrics.terminal_last_guard = 10;
                    return super::resolve::HelperReapReadiness::Held;
                };
                if !self.retained_member_exited(member) {
                    self.metrics.terminal_last_guard = 11;
                    return super::resolve::HelperReapReadiness::Held;
                }
            }
            for member in self
                .retired
                .iter()
                .copied()
                .filter(|m| m.pid > 0 && m.current && m.pid != self.identity.pid)
            {
                self.metrics.terminal_member_pid = member.pid;
                if !self.retained_member_exited(member) {
                    self.metrics.terminal_last_guard = 12;
                    return super::resolve::HelperReapReadiness::Held;
                }
            }
        }
        self.reap_authorized = true;
        self.metrics.reap_authorized = true;
        super::resolve::HelperReapReadiness::Ready
    }
    fn same_terminal(
        a: super::resolve::HelperReapUsage,
        b: super::resolve::HelperReapUsage,
    ) -> bool {
        a.pid == b.pid
            && a.user_ns == b.user_ns
            && a.system_ns == b.system_ns
            && a.rss_bytes == b.rss_bytes
            && a.observed_ns == b.observed_ns
            && a.readings == b.readings
    }
    /// Reconcile the immutable original wait4 budget/cost verdict, not physical
    /// completion. A pending group observation cannot erase or reject known
    /// CPU. Detach/release still needs positive group-gone and paid debt.
    pub(crate) fn reconcile_usage(
        &mut self,
        control: &Control,
        usage: super::resolve::HelperReapUsage,
    ) -> Result<(), ()> {
        if let Some(prior) = self.terminal_usage {
            if Self::same_terminal(prior, usage) && usage.group_gone {
                self.group_gone = true;
                if self.terminal_pending {
                    self.terminal_pending = false;
                    self.metrics.group_kill_raced_exit |= self.metrics.group_kill_attempted
                        && matches!(self.metrics.group_kill_errno, libc::EPERM | libc::ESRCH);
                }
            }
            return if Self::same_terminal(prior, usage) && self.terminal_pass {
                Ok(())
            } else {
                Err(())
            };
        }
        if usage.pid != self.identity.pid || usage.readings != 1 {
            self.reject(control, false);
            return Err(());
        }
        self.terminal_usage = Some(usage);
        self.own_wait4 = true;
        self.running = false;
        self.group_gone = usage.group_gone;
        self.metrics.wait4_reads = 1;
        self.metrics.reap_user_ns = usage.user_ns;
        self.metrics.reap_system_ns = usage.system_ns;
        if self.known_retained_child() && !self.metrics.child_rollup_proven {
            self.metrics.child_cost_unproven = true;
            self.metrics.coverage_known = false;
        }
        self.metrics.reap_quantization_ns = 2_000;
        self.metrics.reap_rss_bytes = usage.rss_bytes;
        let rss_known = usage.rss_bytes != u64::MAX;
        self.metrics.reap_rss_known = rss_known;
        if !rss_known {
            self.metrics.coverage_known = false;
        }
        let cpu = usage
            .user_ns
            .checked_add(1_000)
            .and_then(|u| {
                usage
                    .system_ns
                    .checked_add(1_000)
                    .and_then(|s| u.checked_add(s))
            })
            .unwrap_or(u64::MAX);
        self.metrics.charged_ns = cpu;
        self.metrics.max_charge_ns = cpu;
        let old = control
            .accounting
            .helper_job_cpu
            .try_update(Ordering::AcqRel, Ordering::Acquire, |v| {
                Some(v.saturating_add(cpu))
            })
            .unwrap_or(u64::MAX);
        let job = old.saturating_add(cpu);
        self.metrics.job_cpu_ns = job;
        let debt = control.helper_extend_debt(cpu);
        self.metrics.debt_ns = debt;
        self.debt_until = Instant::now()
            .checked_add(Duration::from_nanos(debt))
            .unwrap_or_else(Instant::now);
        if self.group_gone && self.terminal_pending {
            self.metrics.group_kill_raced_exit |= self.metrics.group_kill_attempted
                && matches!(self.metrics.group_kill_errno, libc::EPERM | libc::ESRCH);
            self.terminal_pending = false;
        }
        let clock_known = usage.observed_ns != 0 && usage.observed_ns != u64::MAX;
        if !clock_known {
            self.metrics.clock_failures += 1;
            control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
        }
        let pause_user_pass = self.terminal_pause_user(usage.user_ns);
        self.terminal_pass = clock_known
            && pause_user_pass
            && cpu != u64::MAX
            && cpu <= HELPER_COMMAND_CPU_NS
            && job <= HELPER_JOB_CPU_NS
            && rss_known
            && usage.rss_bytes <= storage::HELPERS
            && self.creation_settled
            && self.metrics.qos_checked_mask & 1 != 0
            && (self.identity.expected_members == 1 || self.metrics.verified_descendants == 1)
            && self.metrics.coverage_known
            && !self.failed;
        if !self.terminal_pass {
            if cpu == u64::MAX {
                self.metrics.clock_failures += 1;
            }
            self.reject(
                control,
                cpu > HELPER_COMMAND_CPU_NS
                    || job > HELPER_JOB_CPU_NS
                    || (rss_known && usage.rss_bytes > storage::HELPERS)
                    || !pause_user_pass,
            );
            return Err(());
        }
        Ok(())
    }
    pub(crate) fn retirement_settlement_observed(&self) -> bool {
        // Unknown external CPU has no finite authoritative bill. Its bounded
        // mandatory-observation fallback debt is not permission to release the
        // original custody. Finite over-cap CPU and unknown RSS stay distinct.
        let known_bill = self.metrics.charged_ns != u64::MAX
            && self.metrics.charged_ns.max(UNIT_NS).checked_mul(9) == Some(self.metrics.debt_ns);
        self.own_wait4
            && self.group_gone
            && !self.terminal_pending
            && !self.metrics.child_cost_unproven
            && known_bill
    }
    pub(crate) fn settle_quarantine(&mut self, _: &Control) -> bool {
        if self.own_wait4 && !self.group_gone {
            #[cfg(target_os = "macos")]
            {
                let mut pids = [0i32; 3];
                helper_clear_errno();
                let count = unsafe {
                    libc::proc_listpgrppids(
                        self.pgid,
                        pids.as_mut_ptr().cast(),
                        std::mem::size_of_val(&pids) as i32,
                    )
                };
                let errno = io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO);
                self.metrics.terminal_census_result = count;
                self.metrics.terminal_census_errno = errno;
                self.group_gone = helper_census_empty(count, errno);
            }
        }
        if self.own_wait4 && self.group_gone && self.terminal_pending {
            self.terminal_pending = false;
            self.metrics.group_kill_raced_exit |= self.metrics.group_kill_attempted
                && matches!(self.metrics.group_kill_errno, libc::EPERM | libc::ESRCH);
        }
        self.retirement_settlement_observed()
    }
    #[cfg(test)]
    pub(crate) fn fixture_early_original_settled(&self) -> bool {
        self.retirement_settlement_observed()
    }
    #[cfg(test)]
    pub(crate) fn fixture_descendant_identity(&self) -> Option<(i32, u64, u64, u32)> {
        let m = self.members[1];
        let token = self
            .retired
            .iter()
            .find(|r| r.pid == m.pid && r.birth == m.birth && r.current)?;
        (m.pid > 0 && m.verified).then_some((m.pid, m.birth, m.start, token.token.value[7]))
    }
}
impl Control {
    fn helper_extend_debt(&self, cost: u64) -> u64 {
        let debt = cost
            .max(UNIT_NS)
            .checked_mul(9)
            .filter(|_| cost != u64::MAX)
            .unwrap_or_else(|| {
                self.accounting.clock_failed.store(true, Ordering::Release);
                9 * SPAN_NS
            });
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
    pub(crate) fn helper_controller_rest(&self) {
        let until = self
            .accounting
            .helper_control_rest_until
            .load(Ordering::Acquire);
        let now = self.accounting.now();
        if until > now {
            thread::sleep(Duration::from_nanos(until - now));
        }
    }
    pub(crate) fn helper_note_control_work(&self) {
        self.accounting
            .helper_control_work
            .fetch_add(1, Ordering::AcqRel);
    }
    pub(crate) fn helper_controller_marker(&self) -> Option<(u16, u64)> {
        let owner = self.helper.lock().ok()?;
        Some((
            owner.as_ref().map_or(0, |h| h.identity.sequence),
            self.accounting.helper_control_work.load(Ordering::Acquire),
        ))
    }
    pub(crate) fn helper_account_controller(&self, before: Option<(u16, u64)>, inclusive_cpu: u64) {
        let mut owner = self.helper.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(h) = owner.as_mut() {
            let _ = h.account_controller(self, inclusive_cpu);
        }
        if before.is_none_or(|(_, epoch)| {
            epoch == self.accounting.helper_control_work.load(Ordering::Acquire)
        }) {
            return;
        }
        self.accounting.charge(inclusive_cpu, inclusive_cpu);
        let debt = inclusive_cpu
            .max(UNIT_NS)
            .checked_mul(9)
            .filter(|_| inclusive_cpu != u64::MAX)
            .unwrap_or(9 * SPAN_NS);
        self.accounting.helper_control_rest_until.store(
            self.accounting.now().saturating_add(debt),
            Ordering::Release,
        );
    }
    pub(crate) fn helper_tick(&self) {
        let mut owner = self.helper.lock().unwrap_or_else(|p| p.into_inner());
        let Some(h) = owner.as_mut() else {
            return;
        };
        if h.reap_authorized || h.retiring || !h.creation_settled {
            return;
        }
        if self.error().is_some() {
            self.helper_note_control_work();
            let _ = h.retire_members();
            return;
        }
        let foreground = self.foreground.load(Ordering::Acquire);
        if foreground && h.pause_started.is_none() {
            self.helper_note_control_work();
            h.pause_started = Some(Instant::now());
            h.pause_ack = None;
            if let Err(errno) = h.signal_group(libc::SIGSTOP) {
                h.pause_failed = true;
                h.terminal_pending = true;
                if !matches!(errno, libc::EPERM | libc::ESRCH) {
                    h.reject(self, false);
                }
            }
        }
        let due = h.last_sample.elapsed() >= HELPER_SAMPLE_INTERVAL
            || (h.pause_started.is_some() && !h.confirmed_stopped);
        #[cfg(test)]
        let due = due
            && !(self
                .accounting
                .helper_test_skip_live_sampling
                .load(Ordering::Acquire)
                && !foreground
                && h.pause_started.is_none());
        if due {
            self.helper_note_control_work();
            if h.sample(self).is_err() {
                let _ = h.retire_members();
                return;
            }
        }
        if let Some(started) = h.pause_started {
            if h.exited {
                return;
            }
            if h.pause_ack.is_none() && started.elapsed() > HELPER_PAUSE_LIMIT {
                h.reject(self, true);
                let _ = h.retire_members();
                return;
            }
            if h.confirmed_stopped
                && h.pause_ack.is_none()
                && h.record_pause_ack(started.elapsed()).is_err()
            {
                h.reject(self, true);
                let _ = h.retire_members();
                return;
            }
            if !foreground
                && h.ready
                && h.confirmed_stopped
                && !h.pause_failed
                && self.error().is_none()
            {
                self.helper_note_control_work();
                // Re-read RI under the stopped identity before the one resume.
                if !due && h.sample(self) != Ok(true) {
                    h.reject(self, false);
                    return;
                }
                if h.signal_group(libc::SIGCONT).is_err() {
                    h.reject(self, false);
                    return;
                }
                h.pause_started = None;
                h.pause_ack = None;
                h.pause_user_baseline = None;
                h.confirmed_stopped = false;
                h.running = true;
                for member in &mut h.members {
                    member.paused_user = None;
                }
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
        let wall = Instant::now();
        let cpu = cpu_ns();
        if cpu == u64::MAX {
            self.control
                .accounting
                .clock_failed
                .store(true, Ordering::Release);
            return Err(w::Code::ResourceLimit);
        }
        self.heap_peak = self.heap_peak.max(heap());
        let used = self.control.observe_interval(cpu, cpu_ns());
        if self.heap_peak > HEAP_MAX || used > UNIT_NS {
            let wall_ns = u64::try_from(wall.elapsed().as_nanos()).unwrap_or(u64::MAX);
            let _ = self.read_charge_observed_heap(used, used, wall_ns, UNIT_NS, self.heap_peak);
            self.control.read_pay();
            return Err(w::Code::ResourceLimit);
        }
        Ok((cpu, wall))
    }
    /// Record one already closed ordinary or purpose-admitted prepared-helper
    /// span. The helper caller independently validates native2msCPU/250ms wall
    /// for prepared process_create under66331; other native wall gates stay25ms.
    /// The inspect-c1 NativeKind array and its admission remain unchanged.
    pub(crate) fn read_charge(
        &mut self,
        cpu: u64,
        app: u64,
        wall_ns: u64,
        raw_limit: u64,
    ) -> Result<(), w::Code> {
        self.read_charge_observed_heap(cpu, app, wall_ns, raw_limit, heap())
    }
    pub(crate) fn read_charge_observed_heap(
        &mut self,
        cpu: u64,
        app: u64,
        wall_ns: u64,
        raw_limit: u64,
        observed_heap: u64,
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
        self.heap_peak = self.heap_peak.max(observed_heap);
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
    let _ = cleanup_unit_clock(control, f, cpu_ns);
}
pub(crate) fn cleanup_unit_observed(control: &Control, f: impl FnOnce()) -> u64 {
    cleanup_unit_clock(control, f, cpu_ns)
}
fn cleanup_unit_clock(control: &Control, f: impl FnOnce(), mut clock: impl FnMut() -> u64) -> u64 {
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
    elapsed
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
        if J::purpose() == bootstrap::Purpose::ReadPrepare
            && let Some((_, control, _)) = &state
        {
            control.helper_controller_rest();
        }
        let read_control_at = (J::purpose() == bootstrap::Purpose::ReadPrepare).then(cpu_ns);
        let helper_marker = if read_control_at.is_some() {
            state
                .as_ref()
                .and_then(|(_, control, _)| control.helper_controller_marker())
        } else {
            None
        };
        let has_helper = read_control_at.is_some()
            && state.as_ref().is_some_and(|(_, control, _)| {
                control
                    .helper
                    .lock()
                    .map_or(true, |helper| helper.is_some())
            });
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
        unsafe { libc::poll(fds.as_mut_ptr(), 2, if has_helper { 1 } else { 2 }) };
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
            if fds.iter().any(|fd| fd.revents != 0) {
                control.helper_note_control_work();
            }
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
mod helper_total_budget_tests {
    use super::*;
    fn group() -> HelperGroup {
        // API inputs only. No process exists at these labels, and none of these
        // arithmetic tests calls a signal/census/native settlement method.
        let mut h = HelperGroup::attached(super::super::resolve::HelperIdentity {
            sequence: 1,
            pid: 101,
            pgid: 101,
            start_identity: 7,
            expected_members: 1,
            image: PathBuf::from("/private/api-fixture"),
            descendant_image: None,
        });
        h.creation_settled = true;
        h.metrics.qos_checked_mask = 1;
        h
    }
    fn usage(user: u64, system: u64, gone: bool) -> super::super::resolve::HelperReapUsage {
        super::super::resolve::HelperReapUsage {
            pid: 101,
            user_ns: user,
            system_ns: system,
            rss_bytes: 1024,
            observed_ns: 42,
            readings: 1,
            group_gone: gone,
        }
    }
    #[test]
    fn helper_total_unknown_rss_is_unproven_while_finite_excess_is_resource_limit() {
        for rss in [u64::MAX, storage::HELPERS + 1] {
            let c = Control::fixture(Instant::now() + Duration::from_secs(30));
            let mut h = group();
            let mut raw = usage(4_000, 1_000, true);
            raw.rss_bytes = rss;
            assert!(h.reconcile_usage(&c, raw).is_err());
            assert_eq!(h.metrics.reap_rss_bytes, rss);
            assert_eq!(h.metrics.reap_rss_known, rss != u64::MAX);
            assert_eq!(h.metrics.charged_ns, 7_000);
            assert_eq!(h.metrics.debt_ns, 9_000_000);
            assert!(h.retirement_settlement_observed());
            if rss == u64::MAX {
                assert!(!h.metrics.coverage_known);
                assert!(!c.accounting.resource_failed.load(Ordering::Acquire));
                assert_eq!(c.error(), Some(w::Code::SourceUnavailable));
            } else {
                assert!(c.accounting.resource_failed.load(Ordering::Acquire));
                assert_eq!(c.error(), Some(w::Code::ResourceLimit));
            }
        }
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        let mut raw = usage(HELPER_COMMAND_CPU_NS, 0, true);
        raw.rss_bytes = u64::MAX;
        assert!(h.reconcile_usage(&c, raw).is_err());
        assert!(c.accounting.resource_failed.load(Ordering::Acquire)); // independent finite CPU breach survives unknown RSS
    }
    #[test]
    fn helper_total_pause_user_terminal_catches_growth_below_cpu_cap() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        h.pause_started = Some(Instant::now());
        h.pause_ack = Some(Duration::ZERO);
        h.pause_user_baseline = Some(5_000);
        assert!(h.reconcile_usage(&c, usage(25_000, 1_000, true)).is_err());
        assert_eq!(h.metrics.charged_ns, 28_000);
        assert!(h.metrics.charged_ns < HELPER_COMMAND_CPU_NS);
        assert_eq!(h.metrics.debt_ns, 9_000_000);
        assert!(h.metrics.pause_user_checked && !h.metrics.pause_user_pass);
        assert_eq!(h.metrics.pause_user_excess_ns, 20_000);
        assert_eq!(h.metrics.pause_user_quantization_ns, 1_000);
        assert!(h.retirement_settlement_observed());
        let mut h = group();
        h.pause_user_baseline = Some(5_000);
        assert!(h.terminal_pause_user(6_000));
        assert!(!h.terminal_pause_user(6_001));
    }
    #[test]
    fn helper_total_initial_kernel_tail_is_pending_only_with_full_zero_user_source() {
        let mut h = group();
        h.creation_settled = false;
        let mut token = HelperAuditToken::default();
        token.value[5] = 101;
        h.members[0] = HelperMember {
            pid: 101,
            birth: 7,
            start: 19,
            verified: true,
            live: true,
            ..HelperMember::default()
        };
        h.retired[0] = RetiredMember {
            pid: 101,
            birth: 7,
            token,
            current: true,
        };
        h.metrics.initial = HelperInitialObservation {
            valid: 63,
            pid: 101,
            pgid: 101,
            birth: 7,
            start: 19,
            status: libc::SSTOP,
            after_status: libc::SSTOP,
            threads: 1,
            running: 1,
            priority: 4,
            user_ticks: 0,
            ..HelperInitialObservation::default()
        };
        assert!(h.initial_kernel_tail_pending());
        h.metrics.initial.user_ticks = 1;
        assert!(!h.initial_kernel_tail_pending());
        h.metrics.initial.user_ticks = 0;
        h.metrics.initial.valid = 62;
        assert!(!h.initial_kernel_tail_pending());
        h.metrics.initial.valid = 63;
        h.metrics.initial.running = 0;
        assert!(!h.initial_kernel_tail_pending());
        h.metrics.initial.running = 1;
        h.running = true;
        assert!(!h.initial_kernel_tail_pending());
    }
    #[test]
    fn helper_total_terminal_pin_is_retained_before_missing_bsd_can_seek_own_exit() {
        let mut h = group();
        assert!(!h.original_retained()); // admitted fixture label has no native pin
        let mut token = HelperAuditToken::default();
        token.value[5] = 101;
        h.members[0] = HelperMember {
            pid: 101,
            birth: 7,
            start: 19,
            verified: true,
            ..HelperMember::default()
        };
        h.retired[0] = RetiredMember {
            pid: 101,
            birth: 7,
            token,
            current: true,
        };
        assert!(h.original_retained());
        h.identity.start_identity = 8;
        assert!(!h.original_retained());
        h.identity.start_identity = 7;
        h.members[0] = HelperMember::default();
        h.creation_settled = false;
        assert!(h.original_retained()); // typed unactivated birth, cleanup only
        h.running = true;
        assert!(!h.original_retained());
    }
    #[test]
    fn helper_total_known_exit_before_kill_is_not_a_fabricated_kernel_signal_race() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        h.terminal_pending = true;
        h.metrics.group_kill_not_attempted_terminal = true;
        h.reconcile_usage(&c, usage(4_000, 0, true)).unwrap();
        assert!(h.retirement_settlement_observed());
        assert!(h.metrics.group_kill_not_attempted_terminal);
        assert!(!h.metrics.group_kill_attempted && !h.metrics.group_kill_raced_exit);
        assert_eq!(h.metrics.group_kill_errno, 0);
    }
    #[test]
    fn helper_total_reap_authorization_seals_effects_before_usage_callback() {
        let mut h = group();
        h.reap_authorized = true;
        assert!(!h.own_wait4); // producer may consume wait4 before its callback
        for signal in [libc::SIGSTOP, libc::SIGCONT, libc::SIGKILL] {
            assert_eq!(h.signal_group(signal), Err(libc::EALREADY));
        }
        assert_eq!(h.retire_members(), Ok(())); // neither group nor escaped token effect
        assert!(!h.metrics.group_kill_attempted && !h.metrics.pause_stop_attempted);
        assert_eq!(h.metrics.escaped_kill_attempts, 0);
        assert!(!h.retirement_settlement_observed());
    }
    #[test]
    fn helper_total_known_wait4_is_paid_before_group_proof_without_double_billing() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        let first = usage(4_000, 1_000, false);
        assert_eq!(h.reconcile_usage(&c, first), Ok(()));
        assert!(h.own_wait4 && !h.group_gone);
        assert!(!h.retirement_settlement_observed());
        assert!(!h.failed && h.metrics.coverage_known);
        assert_eq!(h.metrics.charged_ns, 7_000);
        assert_eq!(h.metrics.debt_ns, 9_000_000);
        let debt = c.fixture_debt_totals();
        let mut observed = first;
        observed.group_gone = true;
        assert_eq!(h.reconcile_usage(&c, observed), Ok(()));
        assert!(h.retirement_settlement_observed());
        assert_eq!(c.fixture_debt_totals(), debt);
        assert_eq!(c.accounting.helper_job_cpu.load(Ordering::Acquire), 7_000);
        assert_eq!(h.metrics.wait4_reads, 1);
        observed.user_ns += 1;
        assert!(h.reconcile_usage(&c, observed).is_err());
        assert_eq!(c.fixture_debt_totals(), debt);
    }
    #[test]
    fn helper_total_false_empty_census_never_proves_child_or_group_absence() {
        assert!(helper_census_empty(0, 0));
        assert!(!helper_census_empty(0, libc::EIO));
        assert!(!helper_census_empty(0, libc::ESRCH));
        assert!(!helper_census_empty(-1, libc::EIO));
        assert!(!helper_census_empty(1, 0));
        assert!(!helper_census_known(1, libc::EIO));
        assert!(helper_census_known(1, 0));
    }
    #[test]
    fn helper_total_already_exited_unrolled_child_still_has_unknown_cost() {
        assert!(HelperGroup::rollup_counters_match(4, 7, 4, 8));
        assert!(!HelperGroup::rollup_counters_match(4, 7, 3, 8));
        assert!(!HelperGroup::rollup_counters_match(4, 7, 4, 6));
        assert!(!HelperGroup::rollup_counters_match(0, 0, 0, 0));
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        h.identity.expected_members = 2;
        h.members[1] = HelperMember {
            pid: 102,
            birth: 9,
            start: 21,
            verified: true,
            live: false,
            user: 4,
            system: 7,
            terminal_ri_at: Some(Instant::now()),
            ..HelperMember::default()
        };
        h.metrics.descendants = 1;
        h.metrics.verified_descendants = 1;
        assert!(h.reconcile_usage(&c, usage(1_000, 0, true)).is_err());
        assert!(h.metrics.child_cost_unproven && !h.metrics.coverage_known);
        assert_eq!(h.metrics.charged_ns, 3_000);
        assert_eq!(h.metrics.debt_ns, 9_000_000);
        assert!(!h.retirement_settlement_observed());
    }
    #[test]
    fn helper_total_leader_exit_does_not_release_live_or_escaped_child_custody() {
        assert!(!HelperGroup::final_kill_no_attempt_allowed(true, false));
        assert!(!HelperGroup::final_kill_no_attempt_allowed(false, true));
        assert!(HelperGroup::final_kill_no_attempt_allowed(true, true));
        assert!(HelperGroup::token_cleanup_required(true, true, true));
        assert!(!HelperGroup::token_cleanup_required(false, true, true));
        assert!(!HelperGroup::token_cleanup_required(true, false, true));
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        h.metrics.child_cost_unproven = true;
        h.metrics.coverage_known = false;
        assert!(h.reconcile_usage(&c, usage(1_000, 0, true)).is_err());
        assert!(h.own_wait4 && h.group_gone);
        assert!(!h.retirement_settlement_observed());
        assert_eq!(h.metrics.charged_ns, 3_000); // known leader cost preserved, never a child's bill
        assert!(h.metrics.child_cost_unproven);
    }
    #[test]
    fn helper_total_per_member_read_gap_cannot_be_hidden_by_early_global_clock() {
        let mut h = group();
        let base = Instant::now();
        h.members[0] = HelperMember {
            pid: 101,
            live: true,
            ri_started: Some(base),
            ..HelperMember::default()
        };
        h.members[1] = HelperMember {
            pid: 102,
            live: true,
            ri_started: Some(base + Duration::from_millis(5)),
            ..HelperMember::default()
        };
        assert!(h.record_sample_gap(base + Duration::from_millis(40)));
        assert!(!h.record_member_gap(1, base + Duration::from_millis(60)));
        assert_eq!(h.metrics.ri_gap_ns[1], 55_000_000);
        assert_eq!(
            h.members[1].ri_started,
            Some(base + Duration::from_millis(5))
        );
        assert!(h.record_member_gap(0, base + Duration::from_millis(40)));
        assert!(!h.record_member_gap(0, base + Duration::from_millis(60))); // return-time bound, previousstamp unmodified
        assert_eq!(h.members[0].ri_started, Some(base));
    }
    #[test]
    fn helper_total_sample_gap_is_observed_and_never_reset_on_a_miss() {
        let mut h = group();
        let now = Instant::now();
        h.members[0] = HelperMember {
            pid: 101,
            live: true,
            ri_started: Some(now - Duration::from_millis(51)),
            ..HelperMember::default()
        };
        let old = h.members[0].ri_started;
        assert!(!h.record_sample_gap(now));
        assert_eq!(h.metrics.max_sample_gap_ns, 51_000_000);
        assert_eq!(h.members[0].ri_started, old);
        h.creation_settled = false;
        assert!(h.record_sample_gap(now)); // initial suspended birth has no active clock
        h.creation_settled = true;
        h.pause_started = old;
        h.pause_ack = Some(Duration::ZERO);
        assert!(!h.record_sample_gap(now)); // paused living members remain sampled
    }
    #[test]
    fn helper_total_pause_ack_remains_binding_after_a_later_running_observation() {
        let mut h = group();
        h.pause_started = Some(Instant::now());
        h.pause_ack = Some(Duration::from_millis(1));
        h.confirmed_stopped = false;
        let prior = HelperMember {
            pid: 101,
            paused_user: Some(7),
            ..HelperMember::default()
        };
        assert!(h.retained_pause_matches(prior, 7, true));
        assert!(!h.retained_pause_matches(prior, 8, true));
        assert!(!h.retained_pause_matches(prior, 7, false));
        assert!(!h.retained_pause_matches(HelperMember::default(), 0, true));
        h.pause_started = None;
        h.pause_ack = None;
        assert!(h.retained_pause_matches(prior, 8, false));
    }
    #[test]
    fn helper_total_nonpositive_or_mismatched_effect_never_signals_a_group() {
        for (pid, pgid, birth) in [(0, 0, 0), (-1, -1, 1), (101, 102, 1), (101, 101, 0)] {
            let mut h = group();
            h.identity.pid = pid;
            h.identity.pgid = pgid;
            h.identity.start_identity = birth;
            h.pgid = pgid;
            for signal in [libc::SIGSTOP, libc::SIGCONT, libc::SIGKILL] {
                assert_eq!(h.signal_group(signal), Err(libc::EINVAL));
            }
            assert!(!h.metrics.group_kill_attempted && !h.metrics.pause_stop_attempted);
        }
    }
    #[test]
    fn helper_total_wait4_once_has_independent_job_caps_and_fresh_debt() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        let raw = usage(10_000, 4_000, true);
        let admitted_at = Instant::now();
        assert_eq!(h.reconcile_usage(&c, raw), Ok(()));
        assert_eq!(h.metrics.charged_ns, 16_000);
        assert_eq!(h.metrics.reap_quantization_ns, 2_000);
        assert_eq!(h.metrics.debt_ns, 9_000_000);
        assert!(h.debt_until >= admitted_at + Duration::from_millis(9));
        let before = c.fixture_debt_totals();
        assert_eq!(h.reconcile_usage(&c, raw), Ok(()));
        assert_eq!(c.fixture_debt_totals(), before);
        assert_eq!(c.accounting.helper_job_cpu.load(Ordering::Acquire), 16_000);
        assert_eq!(h.metrics.wait4_reads, 1);
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        c.accounting
            .helper_job_cpu
            .store(HELPER_JOB_CPU_NS - 1_000, Ordering::Release);
        let mut h = group();
        assert!(h.reconcile_usage(&c, usage(2_000, 0, true)).is_err());
        assert_eq!(h.metrics.charged_ns, 4_000);
        assert_eq!(h.metrics.job_cpu_ns, HELPER_JOB_CPU_NS + 3_000);
        assert!(c.accounting.resource_failed.load(Ordering::Acquire));
    }
    #[test]
    fn helper_total_terminal_excess_and_unknown_are_never_clamped_or_free() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        assert!(
            h.reconcile_usage(&c, usage(HELPER_COMMAND_CPU_NS + 1, 0, true))
                .is_err()
        );
        assert_eq!(h.metrics.charged_ns, HELPER_COMMAND_CPU_NS + 2_001);
        assert_eq!(h.metrics.debt_ns, 9 * (HELPER_COMMAND_CPU_NS + 2_001));
        assert!(h.retirement_settlement_observed()); // refusal and physical settlement differ
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        assert!(h.reconcile_usage(&c, usage(u64::MAX, 0, true)).is_err());
        assert_eq!(h.metrics.charged_ns, u64::MAX);
        assert_eq!(h.metrics.debt_ns, 9 * SPAN_NS);
        assert!(c.accounting.clock_failed.load(Ordering::Acquire));
        assert!(h.own_wait4 && h.group_gone);
        assert!(
            !h.retirement_settlement_observed(),
            "finite mandatory fallback is not an unknown helper CPU bill or release authority"
        );
    }
    #[test]
    fn helper_total_final_eperm_needs_own_reap_and_group_gone() {
        for gone in [false, true] {
            let c = Control::fixture(Instant::now() + Duration::from_secs(30));
            let mut h = group();
            h.terminal_pending = true;
            h.metrics.group_kill_errno = libc::EPERM;
            h.metrics.group_kill_attempted = true;
            assert!(!h.retirement_settlement_observed());
            assert!(h.reconcile_usage(&c, usage(5_000, 0, gone)).is_ok());
            assert_eq!(h.retirement_settlement_observed(), gone);
            assert_eq!(h.metrics.group_kill_raced_exit, gone);
            assert_eq!(h.metrics.group_kill_errno, libc::EPERM);
        }
    }
    #[test]
    fn helper_total_idle_control_does_not_extend_terminal_debt() {
        let c = Control::fixture(Instant::now() + Duration::from_secs(30));
        let mut h = group();
        h.reconcile_usage(&c, usage(10_000, 0, true)).unwrap();
        let until = c.accounting.rest_until.load(Ordering::Acquire);
        for _ in 0..8 {
            let marker = c.helper_controller_marker();
            c.helper_account_controller(marker, 100);
        }
        assert_eq!(c.accounting.rest_until.load(Ordering::Acquire), until);
        assert_eq!(c.fixture_debt_totals().0, 9_000_000);
        let marker = c.helper_controller_marker();
        c.helper_note_control_work();
        c.helper_account_controller(marker, 100);
        assert_eq!(c.fixture_debt_totals().0, 18_000_000);
        assert!(
            c.accounting
                .helper_control_rest_until
                .load(Ordering::Acquire)
                > c.accounting.now()
        );
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
