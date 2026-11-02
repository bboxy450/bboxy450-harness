// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Owned process groups; cancellation joins leader and pipe readers before return.
use std::os::unix::process::ExitStatusExt;
use std::{io, path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    time::{Instant, sleep_until},
};
use tokio_util::sync::CancellationToken;
#[derive(Debug)]
pub struct Output {
    pub bytes: Vec<u8>,
    pub spill: Option<PathBuf>,
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub pgid: u32,
    pub cancelled: bool,
}
pub struct Request {
    pub program: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub spill_dir: PathBuf,
    pub cap: usize,
    pub redactor: crate::redact::Set,
}
// Signal-path registry: no lock, allocation, runtime poll or await. A negative
// pgid temporarily reserves a slot for the signal handler; retirement cannot
// release the leader PID until that signal has finished. -1 reserves a spawn.
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
static CLOSING: AtomicBool = AtomicBool::new(false);
static GROUPS: [AtomicI32; 256] = [const { AtomicI32::new(0) }; 256];
struct GroupReservation {
    slot: &'static AtomicI32,
    pgid: i32,
}
fn reserve_group() -> io::Result<GroupReservation> {
    if CLOSING.load(Ordering::SeqCst) {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "process shutting down",
        ));
    }
    for slot in &GROUPS {
        if slot
            .compare_exchange(0, -1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return Ok(GroupReservation { slot, pgid: 0 });
        }
    }
    // Fail before spawn; an unregistered process is never admitted.
    Err(io::Error::other("owned process-group registry full"))
}
impl GroupReservation {
    fn bind(&mut self, pgid: u32) {
        self.pgid = pgid as i32;
        self.slot.store(self.pgid, Ordering::SeqCst);
        // Paired SC publication closes the spawn-versus-signal race: either the
        // handler sees the group, or bind observes closing and kills it itself.
        if CLOSING.load(Ordering::SeqCst) {
            unsafe {
                libc::kill(-self.pgid, libc::SIGKILL);
            }
        }
    }
    fn release(&mut self) {
        if self.pgid == -1 {
            return;
        }
        let expected = if self.pgid == 0 { -1 } else { self.pgid };
        loop {
            match self
                .slot
                .compare_exchange(expected, 0, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) | Err(0) => break,
                Err(_) => std::hint::spin_loop(), // handler's one kill syscall
            }
        }
        self.pgid = -1;
    }
}
impl Drop for GroupReservation {
    fn drop(&mut self) {
        self.release();
    }
}
fn signal_registered_group(slot: &AtomicI32, pgid: i32, send: impl FnOnce(i32)) {
    if pgid > 1
        && slot
            .compare_exchange(pgid, -pgid, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    {
        send(pgid);
        slot.store(pgid, Ordering::SeqCst);
    }
}
pub(crate) fn kill_owned_groups() {
    CLOSING.store(true, Ordering::SeqCst);
    for slot in &GROUPS {
        signal_registered_group(slot, slot.load(Ordering::SeqCst), |pgid| unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        });
    }
}
extern "C" fn shutdown_signal(_signal: i32) {
    #[cfg(target_vendor = "apple")]
    let errno = unsafe { libc::__error() };
    #[cfg(target_os = "linux")]
    let errno = unsafe { libc::__errno_location() };
    #[cfg(any(target_vendor = "apple", target_os = "linux"))]
    let saved = unsafe { *errno };
    kill_owned_groups();
    #[cfg(any(target_vendor = "apple", target_os = "linux"))]
    unsafe {
        *errno = saved;
    }
}
pub(crate) fn install_shutdown_handlers() -> io::Result<()> {
    // Install before Tokio's signal registration. signal-hook-registry chains
    // this previous handler before notifying the async signal listener.
    for signal in [libc::SIGHUP, libc::SIGTERM] {
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        action.sa_sigaction = shutdown_signal as *const () as usize;
        action.sa_flags = libc::SA_RESTART;
        unsafe {
            libc::sigemptyset(&mut action.sa_mask);
            libc::sigaddset(&mut action.sa_mask, libc::SIGHUP);
            libc::sigaddset(&mut action.sa_mask, libc::SIGTERM);
            if libc::sigaction(signal, &action, std::ptr::null_mut()) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
struct OwnedGroup {
    pgid: u32,
    unreaped: bool,
    registration: GroupReservation,
    signal: Box<dyn Fn(u32, i32) + Send + Sync>,
}
impl OwnedGroup {
    fn new(pgid: u32, registration: GroupReservation) -> Self {
        let mut group = Self {
            pgid,
            unreaped: true,
            registration,
            signal: Box::new(|pgid, signal| unsafe {
                libc::kill(-(pgid as i32), signal);
            }),
        };
        group.registration.bind(pgid);
        group
    }
    fn send(&self, signal: i32) {
        if self.unreaped {
            (self.signal)(self.pgid, signal);
        }
    }
    // Remove the signal-path reservation BEFORE the wait that can release/reuse
    // the PID. Callers have already killed the group, or observed external reap.
    fn reaped(&mut self) {
        self.registration.release();
        self.unreaped = false;
    }
}
impl Drop for OwnedGroup {
    fn drop(&mut self) {
        self.send(libc::SIGKILL);
    }
}
/// The only owner of a waitable leader and its group lease. Neither the SDK
/// nor pipe readers receive Child/a wait future. All mutable child access stays
/// in this module; both wait and Drop retire the group before Tokio may reap.
///
/// ECHILD cannot precede retirement in the supported process: SIGCHLD automatic
/// reaping is refused at spawn, WNOWAIT never collects, and Tokio's orphan queue
/// gets this child only when it is dropped (after this Drop fence). Installing
/// a reaping SIGCHLD disposition or calling waitpid on another owner's child
/// from foreign/unsafe embedding code violates this ownership contract.
/// See Tokio 1.54.1 process/unix/{reap,orphan}.rs: only the owned child is polled.
pub(crate) struct OwnedChild {
    group: OwnedGroup,
    child: tokio::process::Child,
    #[cfg(feature = "test-fixtures")]
    audit: Option<std::sync::Arc<fixtures::Audit>>,
}
fn check_sigchld(action: &libc::sigaction) -> io::Result<()> {
    if action.sa_sigaction == libc::SIG_IGN || action.sa_flags & libc::SA_NOCLDWAIT != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SIGCHLD must retain child status",
        ));
    }
    Ok(())
}
impl OwnedChild {
    pub(crate) fn spawn(mut command: Command) -> io::Result<Self> {
        let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
        if unsafe { libc::sigaction(libc::SIGCHLD, std::ptr::null(), action.as_mut_ptr()) } == -1 {
            return Err(io::Error::last_os_error());
        }
        let action = unsafe { action.assume_init() };
        check_sigchld(&action)?;
        let reservation = reserve_group()?;
        let child = command.process_group(0).kill_on_drop(true).spawn()?;
        let pgid = child.id().expect("new child is waitable");
        Ok(Self {
            group: OwnedGroup::new(pgid, reservation),
            child,
            #[cfg(feature = "test-fixtures")]
            audit: None,
        })
    }
    pub(crate) fn pgid(&self) -> u32 {
        self.group.pgid
    }
    pub(crate) fn stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }
    pub(crate) fn stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr.take()
    }
    pub(crate) fn stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        self.child.stdin.take()
    }
    pub(crate) fn send(&self, signal: i32) {
        #[cfg(feature = "test-fixtures")]
        if signal == libc::SIGTERM
            && let Some(audit) = &self.audit
        {
            audit.term_signals.fetch_add(1, Ordering::SeqCst);
        }
        self.group.send(signal);
    }
    pub(crate) fn exited(&self) -> io::Result<bool> {
        if !self.group.unreaped {
            return Ok(true); // An earlier owned wait already killed and retired this group.
        }
        exited_without_reaping(self.pgid())
    }
    pub(crate) async fn grace_until(&mut self, deadline: Instant) {
        #[cfg(feature = "test-fixtures")]
        if let Some(audit) = &self.audit {
            audit.grace_sleeps.fetch_add(1, Ordering::SeqCst);
        }
        sleep_until(deadline).await;
    }
    fn retire_for_reap(&mut self) {
        self.group.send(libc::SIGKILL);
        self.group.reaped();
    }
    pub(crate) async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        self.retire_for_reap();
        let result = self.child.wait().await;
        #[cfg(feature = "test-fixtures")]
        if let Some(audit) = &self.audit {
            audit.released.store(true, Ordering::SeqCst);
            // Exercise the handler's real CAS/signal action after PID release.
            // A reused slot for another pid must not be targeted by this probe.
            signal_registered_group(self.group.registration.slot, self.pgid() as i32, |pgid| {
                (self.group.signal)(pgid as u32, libc::SIGKILL);
            });
        }
        result
    }
    fn disown_after_contract_violation(&mut self) {
        // Defense in depth for foreign code breaking the documented invariant;
        // observing ECHILD cannot retroactively fence an arbitrary external reap.
        self.group.reaped();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.retire_for_reap();
    }
}
#[cfg(test)]
mod ownership_tests {
    use super::*;
    #[tokio::test]
    async fn echild_after_owned_reap_never_leaves_a_signal_target() {
        use std::sync::{Arc, atomic::AtomicUsize};
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "exit 0"])
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut owner = OwnedChild::spawn(command).unwrap();
        let pgid = owner.pgid();
        let released = Arc::new(AtomicBool::new(false));
        let late = Arc::new(AtomicUsize::new(0));
        let flag = released.clone();
        let seen = late.clone();
        owner.group.signal = Box::new(move |_, _| {
            if flag.load(Ordering::SeqCst) {
                seen.fetch_add(1, Ordering::SeqCst);
            }
        });
        owner.wait().await.unwrap();
        released.store(true, Ordering::SeqCst);
        let slot = owner.group.registration.slot;
        let visible = owner.group.unreaped
            && matches!(slot.load(Ordering::SeqCst),pid if pid==pgid as i32 || pid==-(pgid as i32));
        let error = exited_without_reaping(pgid).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::ECHILD));
        signal_registered_group(slot, pgid as i32, |pid| {
            (owner.group.signal)(pid as u32, libc::SIGKILL);
        });
        drop(owner);
        assert!(
            !visible,
            "ECHILD observed a PID released while its group was registered"
        );
        assert_eq!(
            late.load(Ordering::SeqCst),
            0,
            "retired leader generated a foreign-group signal"
        );
    }
    #[test]
    fn automatic_sigchld_reaping_is_refused_before_spawn() {
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        action.sa_sigaction = libc::SIG_IGN;
        assert!(
            check_sigchld(&action).is_err(),
            "automatic SIGCHLD reaping was admitted"
        );
        action.sa_sigaction = libc::SIG_DFL;
        action.sa_flags = libc::SA_NOCLDWAIT;
        assert!(check_sigchld(&action).is_err(), "SA_NOCLDWAIT was admitted");
        action.sa_flags = 0;
        assert!(check_sigchld(&action).is_ok());
    }
    #[test]
    fn released_registration_does_not_clear_a_reused_slot() {
        static SLOT: AtomicI32 = AtomicI32::new(123);
        let mut reservation = GroupReservation {
            slot: &SLOT,
            pgid: 123,
        };
        reservation.release();
        assert_eq!(SLOT.load(Ordering::SeqCst), 0);
        SLOT.store(456, Ordering::SeqCst);
        drop(reservation);
        assert_eq!(
            SLOT.load(Ordering::SeqCst),
            456,
            "retired registration touched its replacement"
        );
    }
    #[test]
    fn reaped_group_drop_never_signals_a_reused_identity() {
        let signals = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let seen = signals.clone();
        let mut group = OwnedGroup::new(123, reserve_group().unwrap());
        group.signal = Box::new(move |pid, signal| seen.lock().unwrap().push((pid, signal)));
        group.reaped();
        drop(group);
        assert!(
            signals.lock().unwrap().is_empty(),
            "a reaped group was signalled"
        );
    }
}
#[derive(Debug)]
pub struct Chunk {
    pub stream: crate::protocol::Stream,
    pub text: String,
}
pub type OutputSink = tokio::sync::mpsc::Sender<Chunk>;
async fn emit(
    sink: &Option<OutputSink>,
    cancel: &CancellationToken,
    stream: crate::protocol::Stream,
    bytes: Vec<u8>,
) {
    let Some(sink) = sink else {
        return;
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut rest = text.as_ref();
    while !rest.is_empty() {
        let mut n = rest.len().min(16 * 1024);
        while !rest.is_char_boundary(n) {
            n -= 1;
        }
        let (chunk, tail) = rest.split_at(n);
        let message = Chunk {
            stream,
            text: chunk.into(),
        };
        tokio::select! {biased;_=cancel.cancelled()=>return,r=sink.send(message)=>if r.is_err(){return;}}
        rest = tail;
    }
}
pub async fn run(req: Request, cancel: CancellationToken) -> io::Result<Output> {
    run_stream(req, cancel, None).await
}

// Keep the exited leader waitable: its PID reserves our pgid until the group
// has been killed. Polling Child::wait here would reap it before pipe holders
// exit, losing both the ownership fence and the ability to cancel those members.
fn exited_without_reaping(pid: u32) -> io::Result<bool> {
    loop {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                pid as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
            )
        };
        if result == 0 {
            return Ok(info.si_signo == libc::SIGCHLD);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
pub(crate) async fn leader_exit(pid: u32) -> io::Result<()> {
    while !exited_without_reaping(pid)? {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    Ok(())
}
pub async fn run_stream(
    req: Request,
    cancel: CancellationToken,
    sink: Option<OutputSink>,
) -> io::Result<Output> {
    run_inner(req, cancel, sink, false).await
}
/// Bounded, stdout-only command resolution: never spill credential bytes to disk.
pub(crate) async fn capture_stdout(req: Request, cancel: CancellationToken) -> io::Result<Output> {
    run_inner(req, cancel.child_token(), None, true).await
}
async fn run_inner(
    req: Request,
    cancel: CancellationToken,
    sink: Option<OutputSink>,
    capture_only: bool,
) -> io::Result<Output> {
    if cancel.is_cancelled() {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
    }
    let mut command = Command::new(req.program);
    command
        .args(req.args)
        .current_dir(req.cwd)
        .stdin(if req.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    // Tools, terminals and !cmd resolvers have no seat-identity exception.
    // Launcher MCP children use their separate, origin-checked spawn path.
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"BBOXY_") {
            command.env_remove(name);
        }
    }
    let mut child = OwnedChild::spawn(command)?;
    let pgid = child.pgid();
    let mut stdout = child.stdout().unwrap();
    let mut stderr = child.stderr().unwrap();
    let mut input = child.stdin();
    let input_bytes = req.stdin.unwrap_or_default().into_bytes();
    let mut input_offset = 0;
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    let mut spill = None;
    let mut file = None;
    let mut scrubber = crate::redact::StreamScrubber::new(req.redactor.clone());
    let mut live_scrubber = crate::redact::OrderedScrubber::new(req.redactor.clone());
    let mut a = [0u8; 16 * 1024];
    let mut b = [0u8; 16 * 1024];
    let mut out_open = true;
    let mut err_open = true;
    let mut status = None;
    let mut leader_done = false;
    let exited = leader_exit(pgid);
    tokio::pin!(exited);
    let mut cancelled = false;
    let mut drain_deadline = None;
    let deadline = Instant::now() + req.timeout;
    loop {
        if (leader_done || status.is_some()) && !out_open && !err_open {
            break;
        }
        let chunk = tokio::select! {biased;
            _=cancel.cancelled(),if !cancelled=> {cancelled=true;terminate(&child).await;if status.is_none(){let waited=child.wait().await;status=Some(waited?);}drain_deadline=Some(Instant::now()+Duration::from_millis(500));None},
            _=sleep_until(deadline),if !cancelled=>{cancelled=true;terminate(&child).await;if status.is_none(){let waited=child.wait().await;status=Some(waited?);}drain_deadline=Some(Instant::now()+Duration::from_millis(500));None},
            result=&mut exited,if !leader_done && status.is_none()=>{
                if let Err(error)=result {
                    // Private OwnedChild wait/Drop and the SIGCHLD admission
                    // invariant exclude ECHILD before retirement. This fallback
                    // disowns immediately if foreign unsafe code violates it.
                    if error.raw_os_error()==Some(libc::ECHILD) {child.disown_after_contract_violation();}
                    return Err(error);
                }
                leader_done=true;
                None
            },
            _=async{sleep_until(drain_deadline.unwrap()).await},if drain_deadline.is_some()=>{out_open=false;err_open=false;input=None;None},
            result=async {input.as_mut().unwrap().write(&input_bytes[input_offset..]).await},if input.is_some()=>{
                match result {Ok(n)=>input_offset+=n,Err(_)=>{input=None;}}
                if input_offset==input_bytes.len(){input=None;}None
            },
            n=stdout.read(&mut a),if out_open=>{let n=n?;out_open=n>0;Some((crate::protocol::Stream::Stdout,a[..n].to_vec()))},
            n=stderr.read(&mut b),if err_open=>{let n=n?;err_open=n>0;Some((crate::protocol::Stream::Stderr,b[..n].to_vec()))},
        };
        if let Some((stream, raw)) = chunk {
            if capture_only {
                if matches!(stream, crate::protocol::Stream::Stdout) {
                    let keep = (req.cap - bytes.len()).min(raw.len());
                    bytes.extend_from_slice(&raw[..keep]);
                    if keep < raw.len() {
                        cancel.cancel();
                    }
                }
                continue;
            }
            for (stream, live) in live_scrubber.push(stream, &raw, false) {
                emit(&sink, &cancel, stream, live).await;
            }
            let chunk = scrubber.push(&raw, false);
            if bytes.len() + chunk.len() > req.cap && file.is_none() {
                tokio::fs::create_dir_all(&req.spill_dir).await?;
                let path = req.spill_dir.join(format!("{}.out", ulid::Ulid::new()));
                let mut f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                    .await?;
                f.write_all(&bytes).await?;
                file = Some(f);
                spill = Some(path);
            }
            if let Some(f) = file.as_mut() {
                f.write_all(&chunk).await?;
            }
            let keep = (req.cap - bytes.len()).min(chunk.len());
            bytes.extend_from_slice(&chunk[..keep]);
        }
    }
    if status.is_none() {
        // Even when the pipes closed normally, retire any background members
        // before releasing the leader's PID. Drop then cannot hit a reused pgid.
        let waited = child.wait().await;
        status = Some(waited?);
    }
    if !capture_only {
        for (stream, live) in live_scrubber.push(crate::protocol::Stream::Stdout, &[], true) {
            emit(&sink, &cancel, stream, live).await;
        }
        let tail = scrubber.push(&[], true);
        if bytes.len() + tail.len() > req.cap && file.is_none() {
            tokio::fs::create_dir_all(&req.spill_dir).await?;
            let path = req.spill_dir.join(format!("{}.out", ulid::Ulid::new()));
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .await?;
            f.write_all(&bytes).await?;
            file = Some(f);
            spill = Some(path);
        }
        if let Some(f) = file.as_mut() {
            f.write_all(&tail).await?;
            f.flush().await?;
        }
        let keep = (req.cap - bytes.len()).min(tail.len());
        bytes.extend_from_slice(&tail[..keep]);
    }
    let status = status.expect("child waited");
    Ok(Output {
        bytes: std::mem::take(&mut *bytes),
        spill,
        code: status.code(),
        signal: status.signal(),
        pgid,
        cancelled,
    })
}
pub(crate) async fn terminate(group: &OwnedChild) {
    group.send(libc::SIGTERM);
    tokio::time::sleep(Duration::from_millis(500)).await;
    group.send(libc::SIGKILL);
}

// Fault fixtures share the real owner/wait boundary. They run only in the
// explicit test-fixtures build and use their own disposable foreign group.
#[cfg(feature = "test-fixtures")]
pub mod fixtures {
    use super::*;
    use std::{
        path::Path,
        sync::{Arc, atomic::AtomicUsize},
    };
    pub(super) struct Audit {
        pub(super) released: AtomicBool,
        pub(super) term_signals: AtomicUsize,
        pub(super) grace_sleeps: AtomicUsize,
        late: AtomicUsize,
    }
    pub struct Observation {
        pub members_dead: bool,
        pub foreign_alive: bool,
        pub late_signals: usize,
        pub term_signals: usize,
        pub grace_sleeps: usize,
        pub member_present_before_finish: bool,
        pub missing_id: bool,
        pub failed_handoff: bool,
    }
    pub(crate) struct Fixture {
        pub(crate) child: Option<OwnedChild>,
        pub(crate) pipes: Option<(tokio::process::ChildStdout, tokio::process::ChildStdin)>,
        group: i32,
        member: i32,
        member_present_before_finish: bool,
        foreign: Option<std::process::Child>,
        audit: Arc<Audit>,
        retired: bool,
    }
    fn gone(pgid: i32) -> bool {
        (unsafe { libc::kill(-pgid, 0) }) == -1
            && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    }
    impl Fixture {
        pub(crate) async fn start(program: &Path, root: &Path) -> Self {
            use std::os::unix::process::CommandExt;
            let file = root.join("members");
            let mut command = Command::new(program);
            command
                .arg("stubborn_group")
                .env_clear()
                .env("MCP_GROUP", &file)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            let mut child = OwnedChild::spawn(command).unwrap();
            let group = child.pgid() as i32;
            let pipes = Some((child.stdout().unwrap(), child.stdin().unwrap()));
            tokio::time::timeout(Duration::from_secs(10), async {
                while !file.exists() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("stubborn group did not publish");
            let member = std::fs::read_to_string(&file)
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            let foreign = std::process::Command::new("/bin/sleep")
                .arg("600")
                .env_clear()
                .process_group(0)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let foreign_pid = foreign.id() as i32;
            let audit = Arc::new(Audit {
                released: AtomicBool::new(false),
                term_signals: AtomicUsize::new(0),
                grace_sleeps: AtomicUsize::new(0),
                late: AtomicUsize::new(0),
            });
            let seen = audit.clone();
            let original = std::mem::replace(&mut child.group.signal, Box::new(|_, _| {}));
            child.group.signal = Box::new(move |pgid, signal| {
                if seen.released.load(Ordering::SeqCst) {
                    // Deterministic unit-style PID reuse injection: once the
                    // original leader is released, a stale target is foreign.
                    // This foreign group is owned solely by this test fixture.
                    seen.late.fetch_add(1, Ordering::SeqCst);
                    unsafe {
                        libc::kill(-foreign_pid, signal);
                    }
                } else {
                    original(pgid, signal);
                }
            });
            child.audit = Some(audit.clone());
            Self {
                child: Some(child),
                pipes,
                group,
                member,
                member_present_before_finish: false,
                foreign: Some(foreign),
                audit,
                retired: false,
            }
        }
        pub(crate) async fn exit_leader(&mut self) {
            assert_eq!(unsafe { libc::kill(self.group, libc::SIGKILL) }, 0);
            tokio::time::timeout(Duration::from_secs(5), async {
                while !self.child.as_ref().unwrap().exited().unwrap() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("leader did not become waitable");
        }
        pub(crate) fn record_member_before_finish(&mut self) {
            self.member_present_before_finish = unsafe { libc::kill(self.member, 0) } == 0;
        }
        pub(crate) async fn observe(
            mut self,
            missing_id: bool,
            failed_handoff: bool,
        ) -> Observation {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !gone(self.group) && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            self.retired = gone(self.group);
            let foreign_alive = self.foreign.as_mut().unwrap().try_wait().unwrap().is_none();
            if !foreign_alive {
                self.foreign.take();
            }
            Observation {
                members_dead: self.retired,
                foreign_alive,
                late_signals: self.audit.late.load(Ordering::SeqCst),
                term_signals: self.audit.term_signals.load(Ordering::SeqCst),
                grace_sleeps: self.audit.grace_sleeps.load(Ordering::SeqCst),
                member_present_before_finish: self.member_present_before_finish,
                missing_id,
                failed_handoff,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if !self.retired {
                unsafe {
                    libc::kill(-self.group, libc::SIGKILL);
                }
            }
            if let Some(mut child) = self.foreign.take() {
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.wait();
            }
        }
    }
    pub(crate) fn missing_id(child: &OwnedChild) -> bool {
        child.child.id().is_none()
    }
}
