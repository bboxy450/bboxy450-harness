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
            stream: stream.clone(),
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
async fn leader_exit(pid: u32) -> io::Result<()> {
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
// Decision d20261103136600. Per-operation observation has no Child/PID/kill
// authority. A result is distinct from all explicit admission/reap/IO completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementStatus {
    Pending,
    Confirmed,
    Unavailable,
}
#[derive(Clone)]
pub struct Settlement(std::sync::Arc<SettlementState>);
struct SettlementState {
    state: std::sync::Mutex<(usize, bool)>,
    changed: tokio::sync::Notify,
}
impl Settlement {
    fn new() -> Self {
        Self(std::sync::Arc::new(SettlementState {
            state: std::sync::Mutex::new((0, false)),
            changed: tokio::sync::Notify::new(),
        }))
    }
    pub fn status(&self) -> SettlementStatus {
        let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.1 {
            SettlementStatus::Unavailable
        } else if s.0 == 0 {
            SettlementStatus::Confirmed
        } else {
            SettlementStatus::Pending
        }
    }
    pub async fn wait(&self) -> SettlementStatus {
        loop {
            let notified = self.0.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let status = self.status();
            if status != SettlementStatus::Pending {
                return status;
            }
            notified.await;
        }
    }
    fn work(&self) -> SettlementWork {
        self.0.state.lock().unwrap_or_else(|e| e.into_inner()).0 += 1;
        SettlementWork(Some(self.clone()))
    }
}
struct SettlementWork(Option<Settlement>);
impl SettlementWork {
    fn complete(mut self) {
        if let Some(s) = self.0.take() {
            let mut state = s.0.state.lock().unwrap_or_else(|e| e.into_inner());
            state.0 -= 1;
            drop(state);
            s.0.changed.notify_waiters();
        }
    }
}
impl Drop for SettlementWork {
    fn drop(&mut self) {
        if let Some(s) = self.0.take() {
            let mut state = s.0.state.lock().unwrap_or_else(|e| e.into_inner());
            state.0 -= 1;
            state.1 = true;
            drop(state);
            s.0.changed.notify_waiters();
        }
    }
}
/// Lazy capture-only operation: register its observation BEFORE polling result.
/// Drop cancels only its own child token; the existing owner/retirement path lives
/// until explicit completion. Never-polled Drop explicitly refuses unopened work.
pub struct ObservedCapture {
    request: Option<Request>,
    observation: Settlement,
    initial: Option<SettlementWork>,
    cancel: CancellationToken,
}
pub fn observe_capture(req: Request, cancel: CancellationToken) -> ObservedCapture {
    let observation = Settlement::new();
    let initial = Some(observation.work());
    ObservedCapture {
        request: Some(req),
        observation,
        initial,
        cancel: cancel.child_token(),
    }
}
impl ObservedCapture {
    pub fn settlement(&self) -> Settlement {
        self.observation.clone()
    }
    pub async fn result(&mut self) -> io::Result<Output> {
        let req = self
            .request
            .take()
            .ok_or_else(|| io::Error::other("observed result already taken"))?;
        let ticket = self.initial.take().expect("registered owner");
        let observation = self.observation.clone();
        let cancel = self.cancel.clone();
        let _cancel_on_abandon = cancel.clone().drop_guard();
        // This is the same run_inner/admission/OwnedChild, not another runner.
        bbt_rt::handle()
            .spawn(async move {
                let result = run_inner_observed(req, cancel, None, true, Some(observation)).await;
                ticket.complete();
                result
            })
            .await
            .map_err(|_| io::Error::other("observed process owner unavailable"))?
    }
}
impl Drop for ObservedCapture {
    fn drop(&mut self) {
        self.cancel.cancel();
        if self.request.take().is_some() {
            // No owner future was ever created; admission cannot run later.
            if let Some(ticket) = self.initial.take() {
                ticket.complete();
            }
        }
    }
}
// Wrapper retains the EXISTING OwnedChild on observed early error/unwind. It
// never signals an id after OwnedChild's existing retirement fence has released.
struct ActiveChild {
    child: Option<OwnedChild>,
    work: Option<SettlementWork>,
    waited: bool,
}
impl std::ops::Deref for ActiveChild {
    type Target = OwnedChild;
    fn deref(&self) -> &OwnedChild {
        self.child.as_ref().unwrap()
    }
}
impl std::ops::DerefMut for ActiveChild {
    fn deref_mut(&mut self) -> &mut OwnedChild {
        self.child.as_mut().unwrap()
    }
}
impl ActiveChild {
    async fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        let result = self.child.as_mut().unwrap().wait().await;
        if result.is_ok() {
            self.waited = true;
        } else {
            self.work.take();
        }
        result
    }
}
impl Drop for ActiveChild {
    fn drop(&mut self) {
        if let Some(ticket) = self.work.take() {
            if self.waited {
                drop(self.child.take());
                ticket.complete();
            } else if let Some(mut child) = self.child.take() {
                child.send(libc::SIGKILL);
                drop(bbt_rt::handle().spawn(async move {
                    if child.wait().await.is_ok() {
                        drop(child);
                        ticket.complete();
                    }
                }));
            }
        }
    }
}
// A queued closure that has never entered make owns no child/effect. Explicit
// refusal/drop-before-start completes its admission ticket; panic after entry
// leaves an unavailable ticket, never a positive acknowledgement.
struct AdmissionWork<F: FnOnce() -> io::Result<OwnedChild>> {
    make: Option<F>,
    work: Option<SettlementWork>,
}
impl<F: FnOnce() -> io::Result<OwnedChild>> AdmissionWork<F> {
    fn run(mut self) -> io::Result<SpawnResult> {
        let make = self.make.take().unwrap();
        let result = make();
        match result {
            Ok(child) => Ok(SpawnResult(Some(child), self.work.take())),
            Err(e) => {
                if let Some(ticket) = self.work.take() {
                    ticket.complete();
                }
                Err(e)
            }
        }
    }
}
impl<F: FnOnce() -> io::Result<OwnedChild>> Drop for AdmissionWork<F> {
    fn drop(&mut self) {
        if self.make.is_some()
            && let Some(ticket) = self.work.take()
        {
            ticket.complete();
        }
    }
}
// Decision d20261103136528: a queued job/result keeps the existing exclusive
// OwnedChild. API cancellation/drop is not a joined-cleanup or seal witness.
struct SpawnResult(Option<OwnedChild>, Option<SettlementWork>);
impl Drop for SpawnResult {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            child.send(libc::SIGKILL);
            // Both an abandoned sender and an unreceived channel result arrive
            // here. Keep the same owner through its existing fenced wait.
            let work = self.1.take();
            drop(bbt_rt::handle().spawn(async move {
                let result = child.wait().await;
                drop(child);
                if result.is_ok()
                    && let Some(work) = work
                {
                    work.complete();
                }
            }));
        }
    }
}
// Decision d20261103136567: classify caller observations, never infer their
// historical cause from channel closure. The packet keeps its existing owner.
fn observe_spawn_packet(
    result: Result<io::Result<SpawnResult>, tokio::sync::oneshot::error::RecvError>,
    cancelled: bool,
    now: Instant,
    deadline: Instant,
) -> io::Result<SpawnResult> {
    if cancelled {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "cancelled during process admission",
        ));
    }
    if now >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "process admission deadline",
        ));
    }
    let packet = result.map_err(|_| {
        io::Error::new(io::ErrorKind::Interrupted, "process admission unavailable")
    })??;
    Ok(packet)
}
#[cfg(test)]
fn observe_spawn_result(
    result: Result<io::Result<SpawnResult>, tokio::sync::oneshot::error::RecvError>,
    cancelled: bool,
    now: Instant,
    deadline: Instant,
) -> io::Result<OwnedChild> {
    let mut packet = observe_spawn_packet(result, cancelled, now, deadline)?;
    Ok(packet.0.take().expect("exclusive admitted child"))
}
async fn spawn_packet(
    make: impl FnOnce() -> io::Result<OwnedChild> + Send + 'static,
    cancel: CancellationToken,
    deadline: Instant,
    observation: Option<Settlement>,
) -> io::Result<SpawnResult> {
    let local = cancel.child_token();
    let _drop_guard = local.clone().drop_guard();
    let job_cancel = local.clone();
    let (send, receive) = tokio::sync::oneshot::channel();
    let admission = AdmissionWork {
        make: Some(make),
        work: observation.as_ref().map(Settlement::work),
    };
    drop(bbt_rt::handle().spawn(async move {
        let worker_cancel = job_cancel.clone();
        let queued = crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            let result = if worker_cancel.is_cancelled() {
                Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "cancelled before process admission",
                ))
            } else if Instant::now() >= deadline {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "process admission deadline",
                ))
            } else {
                admission.run()
            };
            // Never return OwnedChild as an abandoned blocking-task output.
            // The channel packet owns retirement even if its receiver drops.
            let _ = send.send(result);
        });
        tokio::pin!(queued);
        tokio::select! {biased;
            _=job_cancel.cancelled()=>{},
            _=sleep_until(deadline)=>{},
            _=&mut queued=>{},
        }
    }));
    tokio::select! {biased;
        _=cancel.cancelled()=>Err(io::Error::new(io::ErrorKind::Interrupted,"cancelled during process admission")),
        _=sleep_until(deadline)=>Err(io::Error::new(io::ErrorKind::TimedOut,"process admission deadline")),
        result=receive=>observe_spawn_packet(result, cancel.is_cancelled(), Instant::now(), deadline),
    }
}
#[cfg(test)]
async fn spawn_admission(
    make: impl FnOnce() -> io::Result<OwnedChild> + Send + 'static,
    cancel: CancellationToken,
    deadline: Instant,
) -> io::Result<OwnedChild> {
    let mut packet = spawn_packet(make, cancel, deadline, None).await?;
    Ok(packet.0.take().expect("exclusive admitted child"))
}
async fn run_inner(
    req: Request,
    cancel: CancellationToken,
    sink: Option<OutputSink>,
    capture_only: bool,
) -> io::Result<Output> {
    // The additive observation layer has a large capture future (pipe buffers).
    // Keep that new layer out of ordinary callers' nested inline stack frames.
    Box::pin(run_inner_observed(req, cancel, sink, capture_only, None)).await
}
async fn run_inner_observed(
    req: Request,
    cancel: CancellationToken,
    sink: Option<OutputSink>,
    capture_only: bool,
    observation: Option<Settlement>,
) -> io::Result<Output> {
    let deadline = Instant::now() + req.timeout;
    if cancel.is_cancelled() {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
    }
    let has_input = req.stdin.is_some();
    let mut packet = spawn_packet(
        move || {
            let mut command = Command::new(req.program);
            command
                .args(req.args)
                .current_dir(req.cwd)
                .stdin(if has_input {
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
            OwnedChild::spawn(command)
        },
        cancel.clone(),
        deadline,
        observation,
    )
    .await?;
    let mut child = ActiveChild {
        child: packet.0.take(),
        work: packet.1.take(),
        waited: false,
    };
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
async fn terminate(group: &OwnedChild) {
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

#[cfg(test)]
mod spawn_admission_tests {
    use super::*;
    use std::sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    type Gate = Arc<(Mutex<bool>, Condvar)>;
    struct Release(Gate);
    impl Release {
        fn open(&self) {
            *self.0.0.lock().unwrap() = true;
            self.0.1.notify_all();
        }
    }
    impl Drop for Release {
        fn drop(&mut self) {
            self.open();
        }
    }
    fn gate() -> (Gate, Release) {
        let g = Arc::new((Mutex::new(false), Condvar::new()));
        (g.clone(), Release(g))
    }
    fn wait(g: &Gate) {
        let mut open = g.0.lock().unwrap();
        while !*open {
            open = g.1.wait(open).unwrap();
        }
    }
    struct Settled(Option<tokio::sync::oneshot::Sender<()>>);
    impl Drop for Settled {
        fn drop(&mut self) {
            if let Some(s) = self.0.take() {
                let _ = s.send(());
            }
        }
    }
    fn sleeper() -> io::Result<OwnedChild> {
        let mut command = Command::new("/bin/sleep");
        command
            .arg("600")
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        OwnedChild::spawn(command)
    }
    async fn reaped(pid: u32) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let observed = exited_without_reaping(pid);
                if observed
                    .as_ref()
                    .err()
                    .is_some_and(|e| e.raw_os_error() == Some(libc::ECHILD))
                {
                    assert_eq!(unsafe { libc::kill(-(pid as i32), 0) }, -1);
                    assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("test-owned admitted child was not reaped");
    }
    #[tokio::test]
    async fn closed_elapsed_result_beats_a_not_ready_caller_timer() {
        let (send, receive) = tokio::sync::oneshot::channel::<io::Result<SpawnResult>>();
        drop(send);
        let now = Instant::now();
        // Explicitly pending caller timer models the other runtime's deadline
        // observation preceding readiness here. The real receive helper runs.
        let result = tokio::select! {biased;
            _=std::future::pending::<()>()=>unreachable!("caller timer is not ready"),
            result=receive=>observe_spawn_result(result, false, now, now),
        };
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::TimedOut);
    }
    #[tokio::test]
    async fn result_observations_preserve_priority_cutoff_and_explicit_errors() {
        let now = Instant::now();
        for (cancelled, deadline, expected) in [
            (
                false,
                now + Duration::from_secs(1),
                io::ErrorKind::Interrupted,
            ),
            (
                true,
                now + Duration::from_secs(1),
                io::ErrorKind::Interrupted,
            ),
            (true, now, io::ErrorKind::Interrupted),
            (false, now, io::ErrorKind::TimedOut),
            (
                false,
                now - Duration::from_nanos(1),
                io::ErrorKind::TimedOut,
            ),
        ] {
            let (send, receive) = tokio::sync::oneshot::channel::<io::Result<SpawnResult>>();
            drop(send);
            let error = observe_spawn_result(receive.await, cancelled, now, deadline)
                .err()
                .unwrap();
            assert_eq!(error.kind(), expected);
            if cancelled {
                assert_eq!(error.to_string(), "cancelled during process admission");
            } else if now < deadline {
                assert_eq!(error.to_string(), "process admission unavailable");
            }
        }
        for (cancelled, deadline, expected) in [
            (
                false,
                now + Duration::from_secs(1),
                io::ErrorKind::PermissionDenied,
            ),
            (false, now, io::ErrorKind::TimedOut),
            (true, now, io::ErrorKind::Interrupted),
        ] {
            let error = observe_spawn_result(
                Ok(Err(io::Error::from_raw_os_error(libc::EACCES))),
                cancelled,
                now,
                deadline,
            )
            .err()
            .unwrap();
            assert_eq!(error.kind(), expected);
            if expected == io::ErrorKind::PermissionDenied {
                assert_eq!(error.raw_os_error(), Some(libc::EACCES));
            }
        }
    }
    #[tokio::test]
    async fn result_observation_hands_off_live_and_reaps_late_owned_packets() {
        for (cancelled, late) in [(false, false), (false, true), (true, false), (true, true)] {
            let child = crate::runtime::blocking(sleeper).await.unwrap();
            let pid = child.pgid();
            let (send, receive) = tokio::sync::oneshot::channel();
            assert!(send.send(Ok(SpawnResult(Some(child), None))).is_ok());
            let now = Instant::now();
            let deadline = if late {
                now
            } else {
                now + Duration::from_secs(5)
            };
            let result = observe_spawn_result(receive.await, cancelled, now, deadline);
            if !cancelled && !late {
                let mut child = result.unwrap();
                assert_eq!(child.pgid(), pid);
                child.send(libc::SIGKILL);
                child.wait().await.unwrap();
            } else {
                let kind = match result {
                    Err(error) => error.kind(),
                    Ok(mut unexpected) => {
                        // Retain cleanup even when run against the old mapping.
                        unexpected.send(libc::SIGKILL);
                        unexpected.wait().await.unwrap();
                        panic!("late/cancelled packet was handed off");
                    }
                };
                assert_eq!(
                    kind,
                    if cancelled {
                        io::ErrorKind::Interrupted
                    } else {
                        io::ErrorKind::TimedOut
                    }
                );
            }
            // ECHILD plus ESRCH for this exact private group prove actual reap;
            // the helper returning an error alone makes no joined-cleanup claim.
            reaped(pid).await;
        }
    }
    #[tokio::test]
    async fn queued_cancel_drop_and_deadline_do_not_need_permit_service() {
        let (held, release) = gate();
        let mut workers = Vec::new();
        for _ in 0..4 {
            let held = held.clone();
            let (sent, ready) = tokio::sync::oneshot::channel();
            workers.push(bbt_rt::handle().spawn(async move {
                crate::runtime::blocking(move || {
                    sent.send(()).unwrap();
                    wait(&held);
                })
                .await
            }));
            tokio::time::timeout(Duration::from_secs(3), ready)
                .await
                .unwrap()
                .unwrap();
        }
        for mode in 0..3 {
            let upstream = CancellationToken::new();
            let called = Arc::new(AtomicBool::new(false));
            let seen = called.clone();
            let (done, settled) = tokio::sync::oneshot::channel();
            let settled_guard = Settled(Some(done));
            let deadline = Instant::now()
                + if mode == 2 {
                    Duration::from_millis(30)
                } else {
                    Duration::from_secs(5)
                };
            let mut admission = Box::pin(spawn_admission(
                move || {
                    let _settled = settled_guard;
                    seen.store(true, Ordering::SeqCst);
                    sleeper()
                },
                upstream.clone(),
                deadline,
            ));
            assert!(futures_util::poll!(admission.as_mut()).is_pending());
            if mode == 0 {
                upstream.cancel();
                let error = tokio::time::timeout(Duration::from_secs(2), &mut admission)
                    .await
                    .unwrap()
                    .err()
                    .unwrap();
                assert_eq!(error.kind(), io::ErrorKind::Interrupted);
            } else if mode == 1 {
                drop(admission);
            } else {
                let error = tokio::time::timeout(Duration::from_secs(2), &mut admission)
                    .await
                    .unwrap()
                    .err()
                    .unwrap();
                assert_eq!(error.kind(), io::ErrorKind::TimedOut);
            }
            tokio::time::timeout(Duration::from_secs(2), settled)
                .await
                .unwrap()
                .unwrap();
            assert!(
                !called.load(Ordering::SeqCst),
                "cancelled queued factory ran, mode {mode}"
            );
            if mode != 0 {
                assert!(
                    !upstream.is_cancelled(),
                    "local drop/deadline cancelled upstream"
                );
            }
        }
        // Permits stay held until all cancellation/drop/deadline observations.
        release.open();
        for worker in workers {
            worker.await.unwrap();
        }
    }
    #[tokio::test]
    async fn abandonment_during_admission_and_after_spawn_retains_reaping_owner() {
        for after_spawn in [false, true] {
            let (g, release) = gate();
            let upstream = CancellationToken::new();
            let (entered, ready) = tokio::sync::oneshot::channel();
            let (pid_send, pid_recv) = tokio::sync::oneshot::channel();
            let mut admission = Box::pin(spawn_admission(
                move || {
                    if !after_spawn {
                        let _ = entered.send(());
                        wait(&g);
                        let child = sleeper()?;
                        let _ = pid_send.send(child.pgid());
                        Ok(child)
                    } else {
                        let child = sleeper()?;
                        let _ = pid_send.send(child.pgid());
                        let _ = entered.send(());
                        wait(&g);
                        Ok(child)
                    }
                },
                upstream.clone(),
                Instant::now() + Duration::from_secs(5),
            ));
            assert!(futures_util::poll!(admission.as_mut()).is_pending());
            tokio::time::timeout(Duration::from_secs(2), ready)
                .await
                .unwrap()
                .unwrap();
            drop(admission);
            assert!(!upstream.is_cancelled());
            release.open();
            let pid = tokio::time::timeout(Duration::from_secs(2), pid_recv)
                .await
                .unwrap()
                .unwrap();
            reaped(pid).await;
        }
    }
    #[tokio::test]
    async fn unreceived_owned_result_and_normal_handoff_are_not_owner_loss() {
        let child = sleeper().unwrap();
        let pid = child.pgid();
        let (send, receive) = tokio::sync::oneshot::channel();
        assert!(send.send(SpawnResult(Some(child), None)).is_ok());
        drop(receive);
        reaped(pid).await;
        let upstream = CancellationToken::new();
        let mut child = spawn_admission(
            sleeper,
            upstream.clone(),
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();
        let pid = child.pgid();
        assert!(!upstream.is_cancelled());
        child.send(libc::SIGKILL);
        child.wait().await.unwrap();
        reaped(pid).await;
    }
    #[tokio::test]
    async fn store_writer_release_during_owned_spawn_diagnostic() {
        use crate::{protocol::*, store::Store};
        use std::{
            io::Write,
            os::{fd::AsRawFd, unix::net::UnixStream},
        };
        let own = tempfile::tempdir_in("/tmp").unwrap();
        let root = own.path().join("sessions");
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: own.path().into(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "engine".into(),
            created_ms: 0,
            vcs: None,
            dream: None,
            source: None,
        };
        let store = Store::open(root.clone(), meta.clone(), None, Default::default())
            .await
            .unwrap();
        let (ready_parent, ready_child) = UnixStream::pair().unwrap();
        let (release_parent, release_child) = UnixStream::pair().unwrap();
        struct ReleaseExec(UnixStream);
        impl Drop for ReleaseExec {
            fn drop(&mut self) {
                let _ = self.0.write_all(&[1]);
            }
        }
        let release = ReleaseExec(release_parent);
        ready_parent.set_nonblocking(true).unwrap();
        let mut ready = tokio::net::UnixStream::from_std(ready_parent).unwrap();
        let mut admission = Box::pin(spawn_admission(
            move || {
                let ready_fd = ready_child.as_raw_fd();
                let release_fd = release_child.as_raw_fd();
                let mut command = Command::new("/usr/bin/true");
                command
                    .env_clear()
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                unsafe {
                    command.pre_exec(move || {
                        let pid = libc::getpid().to_ne_bytes();
                        if libc::write(ready_fd, pid.as_ptr().cast(), pid.len())
                            != pid.len() as isize
                        {
                            return Err(io::Error::last_os_error());
                        }
                        let mut byte = 0u8;
                        loop {
                            let n = libc::read(release_fd, (&mut byte as *mut u8).cast(), 1);
                            if n == 1 {
                                break;
                            }
                            if n < 0
                                && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                            {
                                continue;
                            }
                            return Err(io::Error::from_raw_os_error(libc::EIO));
                        }
                        Ok(())
                    });
                }
                let child = OwnedChild::spawn(command);
                drop(ready_child);
                drop(release_child);
                child
            },
            CancellationToken::new(),
            Instant::now() + Duration::from_secs(5),
        ));
        assert!(futures_util::poll!(admission.as_mut()).is_pending());
        let mut pid_bytes = [0u8; 4];
        tokio::time::timeout(Duration::from_secs(2), ready.read_exact(&mut pid_bytes))
            .await
            .unwrap()
            .unwrap();
        let pid = i32::from_ne_bytes(pid_bytes);
        drop(store);
        // Native nonblocking probe avoids needing another shared blocking permit
        // while this test deliberately parks the admitted spawn before exec.
        let probe = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(crate::store::log_path(&root, meta.session))
            .unwrap();
        let locked = unsafe { libc::flock(probe.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        let errno = if locked < 0 {
            io::Error::last_os_error().raw_os_error()
        } else {
            None
        };
        let replacement = if locked == 0 {
            Some(probe)
        } else {
            drop(probe);
            None
        };
        eprintln!(
            "pre-exec inherited descriptor: child={pid}, reopen_ok={}, flock_return={locked}, errno={errno:?}",
            locked == 0
        );
        drop(release);
        let mut child = tokio::time::timeout(Duration::from_secs(3), admission)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(child.pgid(), pid as u32);
        child.wait().await.unwrap();
        if let Some(replacement) = replacement {
            let contender = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(crate::store::log_path(&root, meta.session))
                .unwrap();
            assert_eq!(
                unsafe { libc::flock(contender.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
                -1,
                "old inherited close released the replacement writer"
            );
            assert_eq!(
                io::Error::last_os_error().raw_os_error(),
                Some(libc::EWOULDBLOCK)
            );
            drop(contender);
            drop(replacement);
        }
        let first_ok = locked == 0;
        let after = Store::open(root, meta, None, Default::default()).await;
        assert!(after.is_ok(), "writer remains after actual child exec/reap");
        drop(after);
        assert!(
            first_ok,
            "Store lifetime ended but pre-exec inherited descriptor retained its flock"
        );
    }
}

#[cfg(test)]
mod settlement_tests {
    use super::*;
    use std::sync::{Arc, Condvar, Mutex};
    fn gate() -> Arc<(Mutex<bool>, Condvar)> {
        Arc::new((Mutex::new(false), Condvar::new()))
    }
    fn block(g: &Arc<(Mutex<bool>, Condvar)>) {
        let mut open = g.0.lock().unwrap();
        while !*open {
            open = g.1.wait(open).unwrap();
        }
    }
    struct Release(Arc<(Mutex<bool>, Condvar)>);
    impl Drop for Release {
        fn drop(&mut self) {
            *self.0.0.lock().unwrap() = true;
            self.0.1.notify_all();
        }
    }
    fn request(path: &std::path::Path, command: &str) -> Request {
        Request {
            program: "/usr/bin/env".into(),
            args: vec![
                "-i".into(),
                "PATH=/usr/bin:/bin".into(),
                "/bin/sh".into(),
                "-c".into(),
                command.into(),
            ],
            stdin: None,
            cwd: path.into(),
            timeout: Duration::from_secs(3),
            spill_dir: path.join("spill"),
            cap: 1024,
            redactor: crate::redact::Set::default(),
        }
    }
    async fn confirmed(s: &Settlement) {
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), s.wait())
                .await
                .unwrap(),
            SettlementStatus::Confirmed
        );
    }
    async fn isolated_case(name: &'static str) -> bool {
        if std::env::var("DREAM_WP7_SETTLEMENT_CASE").as_deref() == Ok(name) {
            return false;
        }
        // Native barriers deliberately retain bounded IO permits. Give each
        // fixture its own runtime instead of contending with another fixture's
        // four-permit allocation; all original process assertions still run.
        let exe = std::env::current_exe().unwrap();
        let result = tokio::task::spawn_blocking(move || {
            std::process::Command::new(exe)
                .args([
                    "--exact",
                    &format!("process::settlement_tests::{name}"),
                    "--nocapture",
                ])
                .env("DREAM_WP7_SETTLEMENT_CASE", name)
                .output()
                .unwrap()
        })
        .await
        .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        true
    }
    #[tokio::test]
    async fn observed_never_polled_live_and_lost_ack_are_distinct() {
        if isolated_case("observed_never_polled_live_and_lost_ack_are_distinct").await {
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let upstream = CancellationToken::new();
        let op = observe_capture(request(temp.path(), "touch forbidden"), upstream.clone());
        let s = op.settlement();
        assert_eq!(s.status(), SettlementStatus::Pending);
        drop(op);
        confirmed(&s).await;
        assert!(!temp.path().join("forbidden").exists());
        assert!(!upstream.is_cancelled());
        let mut op = observe_capture(request(temp.path(), "printf joined"), upstream.clone());
        let s = op.settlement();
        let result = op.result().await.unwrap();
        assert_eq!(result.bytes, b"joined");
        assert_eq!(result.code, Some(0));
        confirmed(&s).await;
        let s = Settlement::new();
        let ticket = s.work();
        let observer = s.clone();
        drop(observer);
        assert_eq!(s.status(), SettlementStatus::Pending);
        drop(ticket);
        assert_eq!(s.wait().await, SettlementStatus::Unavailable);
        let s = Settlement::new();
        let ticket = s.work();
        let task = tokio::spawn(async move {
            let _ticket = ticket;
            panic!("private missing acknowledgement control");
        });
        assert!(task.await.is_err());
        assert_eq!(s.wait().await, SettlementStatus::Unavailable);
    }
    #[tokio::test]
    async fn observed_late_spawn_keeps_resources_while_an_independent_operation_finishes() {
        if isolated_case(
            "observed_late_spawn_keeps_resources_while_an_independent_operation_finishes",
        )
        .await
        {
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let g = gate();
        let release = Release(g.clone());
        let s = Settlement::new();
        let root_work = s.work();
        let upstream = CancellationToken::new();
        let (ready, entered) = tokio::sync::oneshot::channel();
        let (pid_send, pid_receive) = tokio::sync::oneshot::channel();
        let deadline = Instant::now() + Duration::from_millis(60);
        let observation = s.clone();
        let a = tokio::spawn(async move {
            let r = spawn_packet(
                move || {
                    ready.send(()).unwrap();
                    block(&g);
                    let mut command = Command::new("/bin/sh");
                    command
                        .args(["-c", "sleep 1; echo escaped > after-cleanup"])
                        .current_dir(root)
                        .env_clear()
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    let child = OwnedChild::spawn(command)?;
                    pid_send.send(child.pgid()).unwrap();
                    Ok(child)
                },
                upstream,
                deadline,
                Some(observation),
            )
            .await;
            root_work.complete();
            r.err().unwrap().kind()
        });
        entered.await.unwrap();
        assert_eq!(a.await.unwrap(), io::ErrorKind::TimedOut);
        assert_eq!(s.status(), SettlementStatus::Pending);
        let mut b = observe_capture(
            request(temp.path(), "printf independent"),
            CancellationToken::new(),
        );
        let bs = b.settlement();
        assert_eq!(b.result().await.unwrap().bytes, b"independent");
        confirmed(&bs).await;
        assert_eq!(
            s.status(),
            SettlementStatus::Pending,
            "B proof cannot confirm A"
        );
        assert!(
            temp.path().exists(),
            "scratch retained while native job pending"
        );
        drop(release);
        let pid = pid_receive.await.unwrap();
        confirmed(&s).await;
        assert_eq!(
            exited_without_reaping(pid).unwrap_err().raw_os_error(),
            Some(libc::ECHILD)
        );
        assert_eq!(unsafe { libc::kill(-(pid as i32), 0) }, -1);
        assert!(!temp.path().join("after-cleanup").exists());
    }
    #[tokio::test]
    async fn observed_early_io_error_retires_the_actual_same_owner() {
        if isolated_case("observed_early_io_error_retires_the_actual_same_owner").await {
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let s = Settlement::new();
        let root = s.work();
        let mut packet = spawn_packet(
            || {
                let mut c = Command::new("/bin/sleep");
                c.arg("600")
                    .env_clear()
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                OwnedChild::spawn(c)
            },
            CancellationToken::new(),
            Instant::now() + Duration::from_secs(3),
            Some(s.clone()),
        )
        .await
        .unwrap();
        let child = ActiveChild {
            child: packet.0.take(),
            work: packet.1.take(),
            waited: false,
        };
        let pid = child.pgid();
        // Deterministic post-admission IO-error injection exercises the exact
        // run_inner error/unwind owner guard; no orphan-queue success is assumed.
        let result: io::Result<()> = async move {
            let _owner = child;
            Err(io::Error::from_raw_os_error(libc::EIO))
        }
        .await;
        assert_eq!(result.unwrap_err().raw_os_error(), Some(libc::EIO));
        root.complete();
        confirmed(&s).await;
        assert_eq!(
            exited_without_reaping(pid).unwrap_err().raw_os_error(),
            Some(libc::ECHILD)
        );
        assert!(!temp.path().join("spill").exists());
    }
    #[tokio::test]
    async fn observed_queued_refusal_does_not_wait_for_four_busy_permits() {
        if isolated_case("observed_queued_refusal_does_not_wait_for_four_busy_permits").await {
            return;
        }
        let g = gate();
        let release = Release(g.clone());
        let mut jobs = vec![];
        for _ in 0..4 {
            let g = g.clone();
            let (send, ready) = tokio::sync::oneshot::channel();
            jobs.push(tokio::spawn(async move {
                crate::runtime::blocking(move || {
                    send.send(()).unwrap();
                    block(&g);
                })
                .await
            }));
            ready.await.unwrap();
        }
        let temp = tempfile::tempdir().unwrap();
        for mode in 0..3 {
            let upstream = CancellationToken::new();
            let mut req = request(temp.path(), "touch forbidden");
            if mode == 1 {
                req.timeout = Duration::from_millis(30);
            }
            let mut op = observe_capture(req, upstream.clone());
            let s = op.settlement();
            let mut result = Box::pin(op.result());
            assert!(futures_util::poll!(result.as_mut()).is_pending());
            if mode == 0 {
                upstream.cancel();
                assert_eq!(result.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
            } else if mode == 1 {
                assert_eq!(result.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
            } else {
                drop(result);
                drop(op);
                assert!(!upstream.is_cancelled());
            }
            confirmed(&s).await;
        }
        assert!(!temp.path().join("forbidden").exists());
        drop(release);
        for job in jobs {
            job.await.unwrap();
        }
    }
}
