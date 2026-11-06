// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Canonical private dream-bootstrap/1 authority. Fixed control state, no source IO,
//! hash/JSON decoder, runtime, allocation or new thread before the admitted permit.
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    cell::Cell,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
};
pub const PROFILE: &str = "dream-bootstrap/1";
pub const ENV: &str = "BBOXY_DREAM_BOOTSTRAP";
pub const LENGTH: usize = 128;
pub const WINDOW_NS: u64 = 250_000_000;
const CONTROL_NS: u64 = 1_000_000;
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Format,
    Identity,
    Clock,
    Expired,
    Pipe,
    Revoked,
    Limit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Purpose {
    InspectC1 = 1,
    ReadPrepare = 2,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub fd: RawFd,
    pub birth: [u8; 16],
    pub binding: [u8; 32],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permit {
    pub purpose: Purpose,
    pub probe: bool,
    pub birth: [u8; 16],
    pub run_id: [u8; 16],
    pub nonce: [u8; 16],
    pub binding: [u8; 32],
    pub incarnation: u64,
    pub sequence: u64,
    pub deadline_ns: u64,
}
/// Canonical parent-side binding input. Hashing is ordinary parent work before
/// birth/debt, never called by the child's source-free gate.
#[derive(serde::Serialize)]
pub struct Binding<'a> {
    profile: &'static str,
    purpose: &'static str,
    probe: bool,
    op_id: &'a super::c1_wire::Ulid,
    run_id: &'a super::c1_wire::Ulid,
    nonce: &'a super::c1_wire::Ulid,
    incarnation: u64,
    context: Option<&'a super::c1_wire::Context>,
    worker_sha256: &'a super::c1_wire::Hex,
}
impl<'a> Binding<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        purpose: Purpose,
        probe: bool,
        op_id: &'a super::c1_wire::Ulid,
        run_id: &'a super::c1_wire::Ulid,
        nonce: &'a super::c1_wire::Ulid,
        incarnation: u64,
        context: Option<&'a super::c1_wire::Context>,
        worker_sha256: &'a super::c1_wire::Hex,
    ) -> Result<Self, Error> {
        if incarnation == 0 || probe != context.is_none() {
            return Err(Error::Identity);
        }
        Ok(Self {
            profile: PROFILE,
            purpose: match purpose {
                Purpose::InspectC1 => "inspect-c1",
                Purpose::ReadPrepare => "read-prepare",
            },
            probe,
            op_id,
            run_id,
            nonce,
            incarnation,
            context,
            worker_sha256,
        })
    }
    pub fn bytes(&self) -> Result<Vec<u8>, Error> {
        super::c1_wire::canonical_limited(self, 8192).map_err(|_| Error::Limit)
    }
}
pub fn ulid(bytes: &[u8]) -> Result<[u8; 16], Error> {
    if bytes.len() != 26 {
        return Err(Error::Format);
    }
    let mut n = 0u128;
    for c in bytes {
        let v = ALPHABET.iter().position(|a| a == c).ok_or(Error::Format)? as u128;
        n = n
            .checked_mul(32)
            .and_then(|n| n.checked_add(v))
            .ok_or(Error::Format)?;
    }
    Ok(n.to_be_bytes())
}
fn hex(bytes: &[u8]) -> Result<[u8; 32], Error> {
    if bytes.len() != 64 {
        return Err(Error::Format);
    }
    let mut out = [0; 32];
    let digit = |b| match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(Error::Format),
    };
    for (o, pair) in out.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *o = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(out)
}
pub fn token(bytes: &[u8]) -> Result<Token, Error> {
    if bytes.len() > 128 || !bytes.is_ascii() {
        return Err(Error::Format);
    }
    let mut fields = bytes.split(|c| *c == b':');
    if fields.next() != Some(b"1".as_slice()) {
        return Err(Error::Format);
    }
    let fd = fields.next().ok_or(Error::Format)?;
    if fd.is_empty() {
        return Err(Error::Format);
    }
    let mut n = 0u32;
    for c in fd {
        if !c.is_ascii_digit() {
            return Err(Error::Format);
        }
        n = n
            .checked_mul(10)
            .and_then(|n| n.checked_add((c - b'0') as u32))
            .ok_or(Error::Format)?;
        if n > 65535 {
            return Err(Error::Format);
        }
    }
    if !(3..=65535).contains(&n) {
        return Err(Error::Format);
    }
    let birth = ulid(fields.next().ok_or(Error::Format)?)?;
    let binding = hex(fields.next().ok_or(Error::Format)?)?;
    if fields.next().is_some() {
        return Err(Error::Format);
    }
    Ok(Token {
        fd: n as RawFd,
        birth,
        binding,
    })
}
pub fn decode(bytes: &[u8]) -> Result<Permit, Error> {
    if bytes.len() != LENGTH
        || &bytes[..8] != b"BBDBOOT1"
        || bytes[9] > 1
        || bytes[10..16].iter().chain(&bytes[120..]).any(|b| *b != 0)
    {
        return Err(Error::Format);
    }
    let purpose = match bytes[8] {
        1 => Purpose::InspectC1,
        2 => Purpose::ReadPrepare,
        _ => return Err(Error::Format),
    };
    let number = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let p = Permit {
        purpose,
        probe: bytes[9] == 1,
        birth: bytes[16..32].try_into().unwrap(),
        run_id: bytes[32..48].try_into().unwrap(),
        nonce: bytes[48..64].try_into().unwrap(),
        binding: bytes[64..96].try_into().unwrap(),
        incarnation: number(96),
        sequence: number(104),
        deadline_ns: number(112),
    };
    if p.incarnation == 0 || p.sequence != 1 || p.deadline_ns == 0 {
        return Err(Error::Format);
    }
    Ok(p)
}
pub fn encode(p: &Permit) -> [u8; LENGTH] {
    let mut out = [0; LENGTH];
    out[..8].copy_from_slice(b"BBDBOOT1");
    out[8] = p.purpose as u8;
    out[9] = p.probe as u8;
    out[16..32].copy_from_slice(&p.birth);
    out[32..48].copy_from_slice(&p.run_id);
    out[48..64].copy_from_slice(&p.nonce);
    out[64..96].copy_from_slice(&p.binding);
    out[96..104].copy_from_slice(&p.incarnation.to_le_bytes());
    out[104..112].copy_from_slice(&p.sequence.to_le_bytes());
    out[112..120].copy_from_slice(&p.deadline_ns.to_le_bytes());
    out
}
pub fn bound(p: &Permit, t: &Token, purpose: Purpose, basename: &[u8]) -> Result<(), Error> {
    if p.purpose != purpose
        || p.birth != t.birth
        || p.binding != t.binding
        || p.run_id != ulid(basename)?
    {
        return Err(Error::Identity);
    }
    Ok(())
}
pub fn monotonic_ns() -> Result<u64, Error> {
    clock(libc::CLOCK_MONOTONIC)
}
fn clock(which: libc::clockid_t) -> Result<u64, Error> {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(which, t.as_mut_ptr()) } != 0 {
        return Err(Error::Clock);
    }
    let t = unsafe { t.assume_init() };
    let sec = u64::try_from(t.tv_sec).map_err(|_| Error::Clock)?;
    let ns = u64::try_from(t.tv_nsec).map_err(|_| Error::Clock)?;
    if ns >= 1_000_000_000 {
        return Err(Error::Clock);
    }
    sec.checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(ns))
        .ok_or(Error::Clock)
}
fn cpu() -> Result<u64, Error> {
    clock(libc::CLOCK_THREAD_CPUTIME_ID)
}
#[derive(Clone, Copy, Default)]
pub struct Usage {
    pub cpu_ns: u64,
    pub max_cpu_ns: u64,
    pub prepermit_cpu_ns: u64,
    pub reads: u8,
    pub admissions: [u8; 4],
}
impl Usage {
    fn observe(&mut self, at: u64) -> Result<(), Error> {
        self.observe_end(at, cpu()?)
    }
    fn observe_end(&mut self, at: u64, end: u64) -> Result<(), Error> {
        let used = end.checked_sub(at).ok_or(Error::Clock)?;
        self.cpu_ns = self.cpu_ns.checked_add(used).ok_or(Error::Clock)?;
        self.max_cpu_ns = self.max_cpu_ns.max(used);
        if used > CONTROL_NS {
            Err(Error::Limit)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy)]
struct Identity {
    dev: u64,
    ino: u64,
    uid: u32,
}
fn identity(fd: RawFd) -> Result<Identity, Error> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
    if flags < 0
        || flags & libc::O_ACCMODE != libc::O_RDONLY
        || unsafe { libc::fstat(fd, st.as_mut_ptr()) } != 0
    {
        return Err(Error::Pipe);
    }
    let st = unsafe { st.assume_init() };
    // Darwin anonymous pipes have nlink0 and a stable kernel pipe inode. A named
    // FIFO/regular file is not a scheduling lease. Other platforms refuse entry.
    if st.st_mode & libc::S_IFMT != libc::S_IFIFO
        || st.st_nlink != 0
        || st.st_uid != unsafe { libc::geteuid() }
    {
        return Err(Error::Pipe);
    }
    Ok(Identity {
        dev: st.st_dev as u64,
        ino: st.st_ino,
        uid: st.st_uid,
    })
}
fn ready(fd: RawFd, timeout: i32) -> Result<libc::c_short, Error> {
    let mut p = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    if unsafe { libc::poll(&mut p, 1, timeout) } < 0 {
        return Err(
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                Error::Limit
            } else {
                Error::Pipe
            },
        );
    }
    Ok(p.revents)
}
// Fixed, nonallocating diagnostic records; no input bytes or raw OS errors.
struct Line {
    bytes: [u8; 256],
    len: usize,
}
impl Line {
    fn new(prefix: &[u8]) -> Self {
        let mut s = Self {
            bytes: [0; 256],
            len: 0,
        };
        s.text(prefix);
        s
    }
    fn text(&mut self, b: &[u8]) {
        self.bytes[self.len..self.len + b.len()].copy_from_slice(b);
        self.len += b.len();
    }
    fn number(&mut self, mut v: u64) {
        let mut digits = [0; 20];
        let mut n = 0;
        loop {
            digits[n] = b'0' + (v % 10) as u8;
            n += 1;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        for i in (0..n).rev() {
            self.text(&digits[i..i + 1]);
        }
    }
    fn send(&mut self) -> Result<(), Error> {
        // Only the managed private diagnostic pipe is changed; never mutate an
        // inherited operator terminal's file-description flags.
        let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
        let flags = unsafe { libc::fcntl(2, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fstat(2, st.as_mut_ptr()) } != 0 {
            return Err(Error::Pipe);
        }
        let st = unsafe { st.assume_init() };
        if flags & libc::O_ACCMODE != libc::O_WRONLY
            || st.st_mode & libc::S_IFMT != libc::S_IFIFO
            || st.st_nlink != 0
            || unsafe { libc::fcntl(2, libc::F_SETFL, flags | libc::O_NONBLOCK) } != 0
        {
            return Err(Error::Pipe);
        }
        self.text(b"\n");
        if unsafe { libc::write(2, self.bytes.as_ptr().cast(), self.len) } != self.len as isize {
            Err(Error::Pipe)
        } else {
            Ok(())
        }
    }
}
/// Ordinary entry evidence. Only e/m are CLOCK_MONOTONIC nanoseconds. All other
/// counters are verbatim UNVERIFIED auxiliary data, never admission authority.
#[derive(Default)]
struct Entry {
    values: [u64; 8], // s g e m u k r p
    ok: bool,
}
impl Entry {
    // Keep the 296-byte V4 scratch out of the formatter/write and permit frames.
    #[inline(never)]
    fn sample(&mut self, e: u64) -> Result<(), Error> {
        self.values[2] = e;
        self.values[3] = monotonic_ns()?;
        if self.values[3] < e {
            return Err(Error::Clock);
        }
        #[cfg(target_os = "macos")]
        {
            let mut raw = std::mem::MaybeUninit::<libc::rusage_info_v4>::uninit();
            let status = unsafe {
                libc::proc_pid_rusage(
                    libc::getpid(),
                    libc::RUSAGE_INFO_V4,
                    raw.as_mut_ptr().cast(),
                )
            };
            if status == 0 {
                let raw = unsafe { raw.assume_init_ref() };
                self.values[0] = raw.ri_proc_start_abstime;
                self.values[4] = raw.ri_user_time;
                self.values[5] = raw.ri_system_time;
                self.values[6] = raw.ri_runnable_time;
                self.values[7] = raw.ri_pageins;
            }
            self.rusage_status(status);
            #[allow(deprecated)]
            // Exact SDK primitive required by bootstrap ICD; no new dependency.
            {
                self.values[1] = unsafe { libc::mach_absolute_time() };
            }
        }
        #[cfg(not(target_os = "macos"))]
        self.rusage_status(-1);
        Ok(())
    }
    fn rusage_status(&mut self, status: i32) {
        self.ok = status == 0;
        if !self.ok {
            for i in [0, 4, 5, 6, 7] {
                self.values[i] = 0;
            }
        }
    }
    fn format(&self, line: &mut Line) {
        line.text(b"dream-bootstrap entry");
        for (key, value) in [
            b" s=", b" g=", b" e=", b" m=", b" u=", b" k=", b" r=", b" p=",
        ]
        .iter()
        .zip(&self.values)
        {
            line.text(*key);
            line.number(*value);
        }
        line.text(b" ok=");
        line.number(self.ok as u64);
    }
}
// No owned Line return/copy: one caller-owned buffer reused for both records.
#[inline(never)]
fn entry_line(e: u64, line: &mut Line) -> Result<(), Error> {
    let mut entry = Entry::default();
    entry.sample(e)?;
    entry.format(line);
    line.send()
}
/// One owned read descriptor and finite lease. No shared pool/lock or copied path.
pub struct Gate {
    fd: Option<OwnedFd>,
    identity: Identity,
    pub permit: Permit,
    deadline: u64,
    last_clock: Cell<u64>,
    pub usage: Usage,
}
const _: () = assert!(
    std::mem::size_of::<Gate>()
        + LENGTH
        + std::mem::size_of::<Token>()
        + std::mem::size_of::<Line>()
        + std::mem::size_of::<libc::stat>()
        + std::mem::size_of::<Permit>()
        <= 1024
);
// Source storage phases, including a conservative Gate return slot throughout.
// Entry sampling: Gate192 + Token52 + Identity24 + fd4 + Usage32 + deadline8
// + Entry72 + Line264 + V4 scratch296 + timespec16 + scalar/argument scratch64
// =1024 on aarch64 Darwin. V4's non-inlined sample returns BEFORE formatting/
// fstat and BEFORE the128-byte permit buffer or decoded Permit exist. Formatting
// reuses that same Line by reference (no by-value send copy); number scratch20
// and fstat144 are smaller than the retired V4 scratch. Permit/retire phase is
// the old892-byte aggregate +20 decimal +16 clock +64 scalar scratch =992.
// This is owned layout, not a claim about OS/libSystem or compiler spill frames;
// exact ordinary Mach-O prologues are separately retained with the build receipt.
#[cfg(target_os = "macos")]
const _: () = {
    assert!(std::mem::size_of::<libc::rusage_info_v4>() == 296);
    assert!(
        std::mem::size_of::<Gate>()
            + std::mem::size_of::<Token>()
            + std::mem::size_of::<Identity>()
            + std::mem::size_of::<OwnedFd>()
            + std::mem::size_of::<Usage>()
            + 8
            + std::mem::size_of::<Entry>()
            + std::mem::size_of::<Line>()
            + std::mem::size_of::<libc::rusage_info_v4>()
            + std::mem::size_of::<libc::timespec>()
            + 64
            <= 1024
    );
    assert!(
        std::mem::size_of::<Gate>()
            + LENGTH
            + std::mem::size_of::<Token>()
            + std::mem::size_of::<Line>()
            + std::mem::size_of::<libc::stat>()
            + std::mem::size_of::<Permit>()
            + 20
            + 16
            + 64
            <= 1024
    );
};
impl Gate {
    /// Called with borrowed, bounded argv basename before any Vec/PathBuf/Arc.
    /// # Safety
    /// Token.fd must transfer the unique inherited descriptor ownership from
    /// this launch. No other Rust owner may close or reuse that descriptor.
    pub unsafe fn wait(
        t: Token,
        purpose: Purpose,
        basename: &[u8],
        entry_ns: u64,
        prelude_at: u64,
    ) -> Result<Self, Error> {
        if !(3..=65535).contains(&t.fd) {
            return Err(Error::Pipe);
        }
        let fd = unsafe { OwnedFd::from_raw_fd(t.fd) };
        let id = identity(fd.as_raw_fd())?;
        let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
        if unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } != 0 {
            return Err(Error::Pipe);
        }
        let deadline = entry_ns.checked_add(WINDOW_NS).ok_or(Error::Clock)?;
        let mut usage = Usage::default();
        #[cfg(feature = "test-fixtures")]
        let fixture = unsafe { environment(b"BBOXY_DREAM_BOOTSTRAP_TEST_BYPASS")? };
        #[cfg(feature = "test-fixtures")]
        if unsafe { environment(b"BBOXY_DREAM_BOOTSTRAP_TEST_BYPASS")? } == Some(b"1".as_slice()) {
            // Causal fixture: actual controlled allocation before permit. The
            // same source-bound marker checker MUST reject this artifact.
            let mut v = Vec::<u8>::new();
            v.try_reserve_exact(16).map_err(|_| Error::Limit)?;
            v.push(1);
            std::hint::black_box(&v);
            usage.admissions[0] = 1;
        }
        let mut wait = Line::new(b"");
        #[cfg(feature = "test-fixtures")]
        let late_entry = fixture == Some(b"entry_after_permit".as_slice());
        #[cfg(not(feature = "test-fixtures"))]
        let late_entry = false;
        if !late_entry {
            entry_line(entry_ns, &mut wait)?;
        }
        #[cfg(feature = "test-fixtures")]
        if fixture == Some(b"entry_duplicate".as_slice()) {
            wait.len = 0;
            entry_line(entry_ns, &mut wait)?;
        }
        wait.len = 0;
        wait.text(b"dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=");
        wait.number(usage.admissions[0] as u64);
        wait.text(b" channel_admissions=0 owner_admissions=0");
        wait.send()?;
        #[cfg(feature = "test-fixtures")]
        let skip_observe = fixture == Some(b"entry_noobserve".as_slice());
        #[cfg(not(feature = "test-fixtures"))]
        let skip_observe = false;
        if !skip_observe {
            usage.observe(prelude_at)?;
        }
        usage.prepermit_cpu_ns = usage.cpu_ns;
        let mut bytes = [0; LENGTH];
        let mut filled = 0;
        let mut interrupted = 0;
        let mut last_clock = entry_ns;
        while filled < LENGTH {
            let at = cpu()?;
            let now = monotonic_ns()?;
            if now < last_clock {
                return Err(Error::Clock);
            }
            last_clock = now;
            if now >= deadline {
                return Err(Error::Expired);
            }
            let timeout = (deadline - now).div_ceil(1_000_000).min(i32::MAX as u64) as i32;
            let events = match ready(fd.as_raw_fd(), timeout) {
                Ok(e) => e,
                Err(Error::Limit) if interrupted < 4 => {
                    interrupted += 1;
                    usage.observe(at)?;
                    continue;
                }
                Err(e) => return Err(e),
            };
            if events & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                return Err(Error::Revoked);
            }
            if events & libc::POLLIN == 0 {
                return Err(Error::Expired);
            }
            if usage.reads >= 4 {
                return Err(Error::Limit);
            }
            usage.reads += 1;
            let n = unsafe {
                libc::read(
                    fd.as_raw_fd(),
                    bytes[filled..].as_mut_ptr().cast(),
                    LENGTH - filled,
                )
            };
            if n < 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                && interrupted < 4
            {
                interrupted += 1;
                usage.observe(at)?;
                continue;
            }
            if n <= 0 {
                return Err(Error::Revoked);
            }
            filled += n as usize;
            if late_entry {
                wait.len = 0;
                entry_line(entry_ns, &mut wait)?;
            }
            usage.observe(at)?;
            usage.prepermit_cpu_ns = usage.cpu_ns;
        }
        let at = cpu()?;
        let permit = decode(&bytes)?;
        bound(&permit, &t, purpose, basename)?;
        let mut gate = Self {
            fd: Some(fd),
            identity: id,
            permit,
            deadline: deadline.min(permit.deadline_ns),
            last_clock: Cell::new(last_clock),
            usage,
        };
        gate.check_inner()?;
        gate.usage.observe(at)?;
        Ok(gate)
    }
    fn check_inner(&self) -> Result<(), Error> {
        let now = monotonic_ns()?;
        if now < self.last_clock.get() {
            return Err(Error::Clock);
        }
        self.last_clock.set(now);
        if now >= self.deadline {
            return Err(Error::Expired);
        }
        let fd = self.fd.as_ref().ok_or(Error::Revoked)?.as_raw_fd();
        let id = identity(fd)?;
        if (id.dev, id.ino, id.uid) != (self.identity.dev, self.identity.ino, self.identity.uid) {
            return Err(Error::Identity);
        }
        if ready(fd, 0)? != 0 {
            return Err(Error::Revoked);
        }
        Ok(())
    }
    pub fn check(&mut self) -> Result<(), Error> {
        let at = cpu()?;
        self.check_inner()?;
        self.usage.observe(at)
    }
    pub fn admit(&mut self, span: usize) -> Result<(), Error> {
        self.check()?;
        let value = self.usage.admissions.get_mut(span).ok_or(Error::Format)?;
        *value = value.checked_add(1).ok_or(Error::Limit)?;
        if *value != 1 {
            return Err(Error::Limit);
        }
        Ok(())
    }
    pub fn retire(&mut self) -> Result<(Permit, Usage), Error> {
        self.check()?;
        let at = cpu()?;
        let mut line = Line::new(
            b"dream-bootstrap state=paid_hello profile=dream-bootstrap/1 argv_admissions=",
        );
        line.number(self.usage.admissions[0] as u64);
        line.text(b" channel_admissions=");
        line.number(self.usage.admissions[1] as u64);
        line.text(b" owner_admissions=");
        line.number(self.usage.admissions[2] as u64);
        line.text(b" frame_admissions=");
        line.number(self.usage.admissions[3] as u64);
        line.text(b" permit_reads=");
        line.number(self.usage.reads as u64);
        line.text(b" control_cpu_ns=");
        line.number(self.usage.cpu_ns);
        line.text(b" control_max_cpu_ns=");
        line.number(self.usage.max_cpu_ns);
        line.send()?;
        drop(self.fd.take());
        self.usage.observe(at)?;
        Ok((self.permit, self.usage))
    }
}
/// Only borrowed bounded environment access. No std::env String allocation.
/// # Safety
/// Entry environment pointers must remain immutable for this early dispatch.
pub unsafe fn environment(key: &[u8]) -> Result<Option<&'static [u8]>, Error> {
    #[cfg(target_os = "macos")]
    let env = unsafe { *libc::_NSGetEnviron() };
    #[cfg(not(target_os = "macos"))]
    let env = {
        unsafe extern "C" {
            static environ: *mut *mut libc::c_char;
        }
        unsafe { environ }
    };
    unsafe { environment_in(env.cast::<*const libc::c_char>(), key) }
}
unsafe fn environment_in<'a>(
    env: *const *const libc::c_char,
    key: &[u8],
) -> Result<Option<&'a [u8]>, Error> {
    let mut found = None;
    for i in 0..64 {
        let p = unsafe { *env.add(i) };
        if p.is_null() {
            return Ok(found);
        }
        let mut matches = true;
        for (j, b) in key.iter().enumerate() {
            if unsafe { *p.add(j) as u8 } != *b {
                matches = false;
                break;
            }
        }
        if !matches || unsafe { *p.add(key.len()) as u8 } != b'=' {
            continue;
        }
        if found.is_some() {
            return Err(Error::Format);
        }
        let start = unsafe { p.add(key.len() + 1) };
        let mut len = 0;
        while len <= 128 && unsafe { *start.add(len) } != 0 {
            len += 1;
        }
        if len > 128 {
            return Err(Error::Format);
        }
        found = Some(unsafe { std::slice::from_raw_parts(start.cast::<u8>(), len) });
    }
    Err(Error::Limit)
}
pub fn prelude_clock() -> Result<u64, Error> {
    cpu()
}
pub fn merge_usage(usage: Usage, control_max: &AtomicU64, control_cpu: &AtomicU64) {
    control_max.fetch_max(usage.max_cpu_ns, Ordering::AcqRel);
    control_cpu.fetch_add(usage.cpu_ns, Ordering::AcqRel);
}

/// # Safety
/// `p` must point to a live NUL-terminated argv/environment string.
pub unsafe fn borrowed(p: *const libc::c_char, max: usize) -> Result<&'static [u8], Error> {
    if p.is_null() {
        return Err(Error::Format);
    }
    let mut len = 0;
    while len <= max && unsafe { *p.add(len) } != 0 {
        len += 1;
    }
    if len > max {
        Err(Error::Format)
    } else {
        Ok(unsafe { std::slice::from_raw_parts(p.cast(), len) })
    }
}
#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod unsupported_linux {
    use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
    static ARGC: AtomicI32 = AtomicI32::new(0);
    static ARGV: AtomicPtr<*const libc::c_char> = AtomicPtr::new(std::ptr::null_mut());
    // GNU ELF init_array supplies argc/argv/envp, as used by Rust's own Unix
    // argument capture. Retain borrowed pointers only; unsupported C1 refuses.
    unsafe extern "C" fn capture(
        argc: libc::c_int,
        argv: *const *const libc::c_char,
        _env: *const *const libc::c_char,
    ) {
        ARGC.store(argc, Ordering::Relaxed);
        ARGV.store(argv.cast_mut(), Ordering::Relaxed);
    }
    #[used]
    #[unsafe(link_section = ".init_array")]
    static CAPTURE: unsafe extern "C" fn(
        libc::c_int,
        *const *const libc::c_char,
        *const *const libc::c_char,
    ) = capture;
    pub fn arguments() -> Option<(usize, *const *const libc::c_char)> {
        let n = ARGC.load(Ordering::Relaxed);
        let p = ARGV.load(Ordering::Relaxed);
        if n <= 0 || p.is_null() {
            None
        } else {
            Some((n as usize, p.cast_const()))
        }
    }
}
pub fn arguments() -> Option<(usize, *const *const libc::c_char)> {
    #[cfg(target_os = "macos")]
    {
        Some((
            unsafe { *libc::_NSGetArgc() } as usize,
            unsafe { *libc::_NSGetArgv() }.cast(),
        ))
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        unsupported_linux::arguments()
    }
    #[cfg(not(any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dream::c1_wire as w;
    struct CountAllocator;
    thread_local! {
        static COUNT: Cell<Option<usize>> = const { Cell::new(None) };
    }
    unsafe impl std::alloc::GlobalAlloc for CountAllocator {
        unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
            COUNT.with(|c| c.set(c.get().map(|n| n + 1)));
            unsafe { std::alloc::System.alloc(layout) }
        }
        unsafe fn dealloc(&self, p: *mut u8, layout: std::alloc::Layout) {
            unsafe { std::alloc::System.dealloc(p, layout) }
        }
        unsafe fn realloc(&self, p: *mut u8, layout: std::alloc::Layout, n: usize) -> *mut u8 {
            COUNT.with(|c| c.set(c.get().map(|n| n + 1)));
            unsafe { std::alloc::System.realloc(p, layout, n) }
        }
    }
    #[global_allocator]
    static ALLOCATOR: CountAllocator = CountAllocator;
    #[test]
    fn entry_exact_maximum_bytes_failure_fields_and_actual_formatter_allocations() {
        let mut entry = Entry {
            values: [u64::MAX; 8],
            ok: true,
        };
        let mut line = Line::new(b"");
        COUNT.with(|c| c.set(Some(0)));
        entry.format(&mut line);
        line.text(b"\n");
        let allocations = COUNT.with(|c| c.replace(None).unwrap());
        assert_eq!(allocations, 0);
        assert_eq!(line.len, 211);
        assert_eq!(&line.bytes[..line.len], b"dream-bootstrap entry s=18446744073709551615 g=18446744073709551615 e=18446744073709551615 m=18446744073709551615 u=18446744073709551615 k=18446744073709551615 r=18446744073709551615 p=18446744073709551615 ok=1\n");
        for status in [-1, 1, i32::MIN] {
            entry.values = [u64::MAX; 8];
            entry.rusage_status(status);
            line.len = 0;
            entry.format(&mut line);
            line.text(b"\n");
            assert_eq!(&line.bytes[..line.len], b"dream-bootstrap entry s=0 g=18446744073709551615 e=18446744073709551615 m=18446744073709551615 u=0 k=0 r=0 p=0 ok=0\n");
        }
        entry.values = [0; 8];
        entry.rusage_status(0);
        assert!(entry.ok, "zero counters are still successful raw samples");
        COUNT.with(|c| c.set(Some(0)));
        let allocation_control = std::hint::black_box(Vec::<u8>::with_capacity(16));
        let bad_count = COUNT.with(|c| c.replace(None).unwrap());
        assert!(
            bad_count > 0,
            "allocator witness must detect actual allocation"
        );
        drop(allocation_control);
    }
    #[test]
    fn entry_usage_keeps_finite_clock_budget_and_single_charge() {
        let mut usage = Usage::default();
        assert_eq!(usage.observe_end(100, 300), Ok(()));
        assert_eq!(usage.cpu_ns, 200);
        assert_eq!(usage.observe_end(300, 299), Err(Error::Clock));
        assert_eq!(usage.cpu_ns, 200);
        assert_eq!(usage.observe_end(0, CONTROL_NS + 1), Err(Error::Limit));
        assert_eq!(usage.cpu_ns, CONTROL_NS + 201);
        usage.cpu_ns = u64::MAX;
        assert_eq!(usage.observe_end(0, 1), Err(Error::Clock));
        eprintln!(
            "entry-layout Gate={} Token={} Permit={} Line={} Usage={} Entry={} stat={} timespec={}",
            std::mem::size_of::<Gate>(),
            std::mem::size_of::<Token>(),
            std::mem::size_of::<Permit>(),
            std::mem::size_of::<Line>(),
            std::mem::size_of::<Usage>(),
            std::mem::size_of::<Entry>(),
            std::mem::size_of::<libc::stat>(),
            std::mem::size_of::<libc::timespec>()
        );
    }
    #[test]
    fn independent_canonical_bootstrap_corpus_and_binding_vectors() {
        let root = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/dream-bootstrap"
        ));
        let inventory: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("vectors.json")).unwrap()).unwrap();
        for case in inventory["cases"].as_array().unwrap() {
            let bytes = std::fs::read(root.join(case["file"].as_str().unwrap())).unwrap();
            let decoded = decode(&bytes);
            let env = token(case["environment"].as_str().unwrap().as_bytes());
            assert_eq!(
                decoded.is_ok(),
                case["expect"]["binary_format"] == "valid",
                "{}",
                case["name"]
            );
            assert_eq!(
                env.is_ok(),
                case["expect"]["environment_format"] == "valid",
                "{}",
                case["name"]
            );
            assert_eq!(w::hash(&bytes).0, case["record_sha256"]);
            let inputs = &case["authority"]["inputs"];
            let purpose = if inputs["purpose"] == "inspect-c1" {
                Purpose::InspectC1
            } else {
                Purpose::ReadPrepare
            };
            let context = if inputs["context"].is_null() {
                None
            } else {
                Some(w::decode::<w::Context>(&w::canonical(&inputs["context"]).unwrap()).unwrap())
            };
            let op = w::Ulid(inputs["op_id"].as_str().unwrap().into());
            let run = w::Ulid(inputs["run_id"].as_str().unwrap().into());
            let nonce = w::Ulid(inputs["nonce"].as_str().unwrap().into());
            let hash = w::Hex(inputs["worker_sha256"].as_str().unwrap().into());
            let binding = Binding::new(
                purpose,
                inputs["probe"].as_bool().unwrap(),
                &op,
                &run,
                &nonce,
                inputs["incarnation"].as_u64().unwrap(),
                context.as_ref(),
                &hash,
            )
            .unwrap();
            assert_eq!(
                binding.bytes().unwrap(),
                case["authority"]["binding_canonical_json"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
            );
            assert_eq!(
                w::hash(binding.bytes().unwrap()).0,
                case["authority"]["binding_sha256"]
            );
            if let (Ok(p), Ok(t)) = (decoded, env) {
                assert_eq!(encode(&p).as_slice(), bytes);
                let matches = bound(&p, &t, purpose, run.0.as_bytes()).is_ok()
                    && t.fd == case["authority"]["fd"].as_i64().unwrap() as i32
                    && p.nonce == ulid(nonce.0.as_bytes()).unwrap()
                    && p.probe == inputs["probe"].as_bool().unwrap()
                    && p.incarnation == inputs["incarnation"].as_u64().unwrap()
                    && p.birth
                        == ulid(case["authority"]["birth"].as_str().unwrap().as_bytes()).unwrap();
                assert_eq!(
                    matches,
                    case["expect"]["identity"] == "match",
                    "{}",
                    case["name"]
                );
            }
        }
        let id = w::Ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV".into());
        let hash = w::hash(b"approved worker");
        assert!(Binding::new(Purpose::InspectC1, false, &id, &id, &id, 1, None, &hash).is_err());
        let probe = Binding::new(Purpose::InspectC1, true, &id, &id, &id, 1, None, &hash).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&probe.bytes().unwrap()).unwrap();
        assert!(json.as_object().unwrap().contains_key("context"));
        assert!(json["context"].is_null());
    }
    #[test]
    fn duplicate_environment_and_non_ascii_token_fail_without_allocating_gate_state() {
        let a = std::ffi::CString::new("BBOXY_DREAM_BOOTSTRAP=1:7:01ARZ3NDEKTSV4RRFFQ69G5FAV:0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let b = a.clone();
        let entries = [a.as_ptr(), b.as_ptr(), std::ptr::null()];
        assert_eq!(
            unsafe { environment_in(entries.as_ptr(), ENV.as_bytes()) },
            Err(Error::Format)
        );
        assert!(token(b"1:7:01ARZ3NDEKTSV4RRFFQ69G5FAV:nonascii-\xff").is_err());
    }
}
