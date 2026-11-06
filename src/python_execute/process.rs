// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Exact-owned subprocess capture; leaders remain unreaped until their group is fenced.
use super::*;
use std::os::unix::{fs::PermissionsExt, process::ExitStatusExt};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
};

pub(super) struct Captured {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub pid: u32,
    pub cancelled: bool,
    pub timed_out: bool,
}
pub(super) async fn resolve(name: &str) -> Option<PathBuf> {
    resolve_in(name, std::env::current_dir().ok()?).await
}
pub(super) async fn resolve_in(name: &str, cwd: PathBuf) -> Option<PathBuf> {
    if name.is_empty() || name.contains('\0') {
        return None;
    }
    let name = name.to_owned();
    crate::runtime::blocking(move || {
        let candidates = if name.contains('/') {
            vec![PathBuf::from(&name)]
        } else {
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .map(|p| p.join(&name))
                .collect()
        };
        // Do not canonicalize executable symlinks: multicall launchers (ATC,
        // busybox, virtualenvs) select behavior from the invoked basename.
        Ok::<_, std::io::Error>(
            candidates
                .into_iter()
                .map(|p| if p.is_absolute() { p } else { cwd.join(p) })
                .find(|p| {
                    p.metadata()
                        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                }),
        )
    })
    .await
    .ok()
    .flatten()
}
impl Service {
    pub(super) async fn capture(
        &self,
        run: &Arc<Run>,
        mut command: Command,
        cwd: PathBuf,
        input: Option<Vec<u8>>,
        timeout_ms: u64,
        python: bool,
    ) -> Result<Captured, Value> {
        use std::process::Stdio;
        command
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = crate::process::OwnedChild::spawn(command).map_err(|e| {
            fault(
                "command_unavailable",
                &self
                    .0
                    .redactor
                    .scrub(&format!("cannot start owned child: {e}")),
            )
        })?;
        let pid = child.pgid();
        if python {
            {
                let mut r = run.record.lock().await;
                r.pid = Some(pid);
                r.status = "running".into();
            }
            self.persist(run)
                .await
                .map_err(|_| fault("execution_storage", "cannot persist child custody"))?;
        }
        let mut stdin = child
            .stdin()
            .ok_or_else(|| fault("process_io", "missing child stdin"))?;
        let cancel = run.cancel.clone();
        let input_task = bbt_rt::handle().spawn(async move {
            if let Some(mut input) = input {
                if python {
                    input.push(b'\n');
                }
                tokio::select! {biased;_=cancel.cancelled()=>{},_=stdin.write_all(&input)=>{}}
            }
            let _ = stdin.shutdown().await;
        });
        let stdout = child
            .stdout()
            .ok_or_else(|| fault("process_io", "missing child stdout"))?;
        let stderr = child
            .stderr()
            .ok_or_else(|| fault("process_io", "missing child stderr"))?;
        let pipe_cancel = CancellationToken::new();
        let pipe = |reader: Box<dyn AsyncRead + Unpin + Send>, stream: &'static str| {
            let s = self.clone();
            let r = run.clone();
            let stop = pipe_cancel.clone();
            bbt_rt::handle().spawn(async move { s.drain(&r, reader, stream, python, stop).await })
        };
        let out = pipe(Box::new(stdout), "stdout");
        let err = pipe(Box::new(stderr), "stderr");
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut tick = tokio::time::interval(Duration::from_millis(10));
        let mut cancelled = false;
        let mut timed_out = false;
        loop {
            if child
                .exited()
                .map_err(|_| fault("process_custody", "cannot observe owned child"))?
            {
                break;
            }
            tokio::select! {biased;
                _=run.cancel.cancelled()=>{cancelled=true;break;},
                _=self.0.cancel.cancelled()=>{cancelled=true;break;},
                _=tokio::time::sleep_until(deadline)=>{timed_out=true;break;},
                _=tick.tick()=>{},
            }
        }
        if cancelled || timed_out {
            run.cancel.cancel();
        }
        // This exact group's leader is still waitable, including after natural
        // exit. OwnedChild fences remaining descendants BEFORE reaping the PID.
        let status = child
            .wait()
            .await
            .map_err(|_| fault("process_custody", "owned child retirement failed"))?;
        pipe_cancel.cancel();
        let _ = input_task.await;
        let (stdout, stdout_bytes) = out
            .await
            .map_err(|_| fault("process_io", "stdout reader failed"))?;
        let (stderr, stderr_bytes) = err
            .await
            .map_err(|_| fault("process_io", "stderr reader failed"))?;
        Ok(Captured {
            stdout,
            stderr,
            stdout_bytes,
            stderr_bytes,
            code: status.code(),
            signal: status.signal(),
            pid,
            cancelled,
            timed_out,
        })
    }
    async fn drain(
        &self,
        run: &Run,
        mut reader: Box<dyn AsyncRead + Unpin + Send>,
        stream: &str,
        publish: bool,
        cancel: CancellationToken,
    ) -> (Vec<u8>, u64) {
        let mut retained = Vec::new();
        let mut total = 0;
        let mut scrubber = crate::redact::StreamScrubber::new(self.0.redactor.clone());
        let mut buffer = [0u8; 8192];
        loop {
            let next = tokio::select! {biased;
                r=reader.read(&mut buffer)=>r,
                _=cancel.cancelled()=>{
                    // An escaped pipe holder must not keep the host waiting.
                    match tokio::time::timeout(Duration::from_millis(100),reader.read(&mut buffer)).await {Ok(r)=>r,Err(_)=>break}
                }
            };
            let Ok(n) = next else { break };
            if n == 0 {
                break;
            }
            let bytes = scrubber.push(&buffer[..n], false);
            total += bytes.len() as u64;
            if publish {
                self.append(run, stream, bytes).await;
            } else {
                let size = bytes.len().min(OUTPUT.saturating_sub(retained.len()));
                retained.extend_from_slice(&bytes[..size]);
            }
        }
        let last = scrubber.push(&[], true);
        total += last.len() as u64;
        if publish {
            self.append(run, stream, last).await;
        } else {
            let n = last.len().min(OUTPUT.saturating_sub(retained.len()));
            retained.extend_from_slice(&last[..n]);
        }
        (retained, total)
    }
}
