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
struct OwnedGroup {
    pgid: u32,
    unreaped: bool,
    signal: Box<dyn Fn(u32, i32) + Send + Sync>,
}
impl OwnedGroup {
    fn new(pgid: u32) -> Self {
        Self {
            pgid,
            unreaped: true,
            signal: Box::new(|pgid, signal| unsafe {
                libc::kill(-(pgid as i32), signal);
            }),
        }
    }
    fn send(&self, signal: i32) {
        if self.unreaped {
            (self.signal)(self.pgid, signal);
        }
    }
    fn reaped(&mut self) {
        self.unreaped = false;
    }
}
impl Drop for OwnedGroup {
    fn drop(&mut self) {
        self.send(libc::SIGKILL);
    }
}
#[cfg(test)]
mod ownership_tests {
    use super::*;
    #[test]
    fn reaped_group_drop_never_signals_a_reused_identity() {
        let signals = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let seen = signals.clone();
        let mut group = OwnedGroup::new(123);
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
        .env_remove("BBOXY_CAPABILITY")
        .stdin(if req.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    if capture_only {
        // A !cmd resolver is untrusted regardless of the eventual key/server
        // consumer. Preserve PATH/HOME but never hand seat identity to its shell.
        for (name, _) in std::env::vars_os() {
            if name.as_encoded_bytes().starts_with(b"BBOXY_") {
                command.env_remove(name);
            }
        }
    }
    let mut child = command.spawn()?;
    let pgid = child.id().expect("spawned child PID");
    let mut group = OwnedGroup::new(pgid);
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut input = child.stdin.take();
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
            _=cancel.cancelled(),if !cancelled=> {cancelled=true;terminate(&group).await;if status.is_none(){let waited=child.wait().await;group.reaped();status=Some(waited?);}drain_deadline=Some(Instant::now()+Duration::from_millis(500));None},
            _=sleep_until(deadline),if !cancelled=>{cancelled=true;terminate(&group).await;if status.is_none(){let waited=child.wait().await;group.reaped();status=Some(waited?);}drain_deadline=Some(Instant::now()+Duration::from_millis(500));None},
            result=&mut exited,if !leader_done && status.is_none()=>{
                if let Err(error)=result {
                    // An external reaper has already released the reservation.
                    if error.raw_os_error()==Some(libc::ECHILD) { group.reaped(); }
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
        group.send(libc::SIGKILL);
        let waited = child.wait().await;
        group.reaped();
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
async fn terminate(group: &OwnedGroup) {
    group.send(libc::SIGTERM);
    tokio::time::sleep(Duration::from_millis(500)).await;
    group.send(libc::SIGKILL);
}
