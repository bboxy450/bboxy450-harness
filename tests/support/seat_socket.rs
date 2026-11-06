// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{gateway::unix::UnixClient, protocol::*};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
const SEAT: &str = "00000000000000000000000042";
const OUTPUT_LIMIT: usize = 64 * 1024;

struct FailureHome(tempfile::TempDir);
impl std::ops::Deref for FailureHome {
    type Target = tempfile::TempDir;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for FailureHome {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.disable_cleanup(true);
            eprintln!("seat-fixture preserved_home={}", self.0.path().display());
        }
    }
}
#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    discarded: u64,
    eof: bool,
    error: Option<i32>,
}
fn capture(
    mut read: Box<dyn Read + Send>,
    output: Arc<Mutex<Output>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buffer = [0; 4096];
        loop {
            match read.read(&mut buffer) {
                Ok(0) => {
                    output.lock().unwrap_or_else(|e| e.into_inner()).eof = true;
                    break;
                }
                Ok(n) => {
                    let mut output = output.lock().unwrap_or_else(|e| e.into_inner());
                    let retained = n.min(OUTPUT_LIMIT - output.bytes.len());
                    output.bytes.extend_from_slice(&buffer[..retained]);
                    output.discarded = output.discarded.saturating_add((n - retained) as u64);
                    // Keep draining after the evidence cap; diagnostics cannot
                    // backpressure the child's actual command/close path.
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => (),
                Err(error) => {
                    output.lock().unwrap_or_else(|e| e.into_inner()).error = error.raw_os_error();
                    break;
                }
            }
        }
    })
}

enum Child {
    Pty(Box<dyn portable_pty::Child + Send + Sync>),
    Gui(std::process::Child),
}
impl Child {
    fn pid(&self) -> u32 {
        match self {
            Self::Pty(c) => c.process_id().unwrap(),
            Self::Gui(c) => c.id(),
        }
    }
    fn try_wait(&mut self) -> std::io::Result<Option<bool>> {
        match self {
            Self::Pty(c) => c.try_wait().map(|s| s.map(|s| s.success())),
            Self::Gui(c) => c.try_wait().map(|s| s.map(|s| s.success())),
        }
    }
    fn status_text(&mut self) -> std::io::Result<Option<String>> {
        match self {
            Self::Pty(c) => c.try_wait().map(|s| s.map(|s| format!("{s:?}"))),
            Self::Gui(c) => c.try_wait().map(|s| s.map(|s| format!("{s:?}"))),
        }
    }
}
struct Seat {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: Child,
    writer: Option<Box<dyn Write + Send>>,
    readers: Vec<std::thread::JoinHandle<()>>,
    stdout: Arc<Mutex<Output>>,
    stderr: Arc<Mutex<Output>>,
    root: PathBuf,
    stage: &'static str,
    observations: Vec<serde_json::Value>,
    started: Instant,
    finished: bool,
}
impl Seat {
    fn start(root: &Path, gui: bool, resume: Option<&str>) -> Self {
        let mut error_read: Option<Box<dyn Read + Send>> = None;
        let (master, child, writer, read): (_, _, Box<dyn Write + Send>, Box<dyn Read + Send>) =
            if gui {
                // The GUI host needs a scripted control stream, not a terminal.
                // Pipes avoid sharing terminal lifecycle or descriptor timing with
                // the concurrently running PTY contract witnesses.
                use std::process::{Command, Stdio};
                let mut command = Command::new(std::env::current_exe().unwrap());
                command
                    .args([
                        "--exact",
                        "seat_socket::gui_socket_worker",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("HARNESS_GUI_SOCKET_WORKER", "1")
                    .env("HOME", root)
                    .env("TERM", "dumb")
                    .env("BBOXY_SEAT_ID", SEAT)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                for key in ["BBOXY_CAPABILITY", "BBOXY_CAPABILITY_FILE", "BBOXY_SOCK"] {
                    command.env_remove(key);
                }
                if let Some(session) = resume {
                    command.env("HARNESS_GUI_SOCKET_RESUME", session);
                }
                let mut child = super::spawn_without_sibling_fds(&mut command);
                let writer = Box::new(child.stdin.take().unwrap());
                let read = Box::new(child.stdout.take().unwrap());
                error_read = Some(Box::new(child.stderr.take().unwrap()));
                (None, Child::Gui(child), writer, read)
            } else {
                let pty = native_pty_system()
                    .openpty(PtySize {
                        rows: 24,
                        cols: 100,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .unwrap();
                let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
                command.args(["seat", "--profile", "scripted", "--socket"]);
                command.arg(root.join("seat.sock"));
                command.arg("--scripted");
                command
                    .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse"));
                if let Some(session) = resume {
                    command.args(["--resume", session]);
                }
                for key in ["BBOXY_CAPABILITY", "BBOXY_CAPABILITY_FILE", "BBOXY_SOCK"] {
                    command.env_remove(key);
                }
                command.env("HOME", root);
                command.env("TERM", "dumb");
                command.env("BBOXY_SEAT_ID", SEAT);
                let child = pty.slave.spawn_command(command).unwrap();
                drop(pty.slave);
                let writer = pty.master.take_writer().unwrap();
                let read = pty.master.try_clone_reader().unwrap();
                (Some(pty.master), Child::Pty(child), writer, read)
            };
        let stdout = Arc::new(Mutex::new(Output::default()));
        let stderr = Arc::new(Mutex::new(Output::default()));
        let mut readers = vec![capture(read, stdout.clone())];
        if let Some(read) = error_read {
            readers.push(capture(read, stderr.clone()));
        }
        Self {
            master,
            child,
            writer: Some(writer),
            readers,
            stdout,
            stderr,
            root: root.to_owned(),
            stage: "spawned",
            observations: vec![],
            started: Instant::now(),
            finished: false,
        }
    }
    fn output(&self) -> Vec<u8> {
        self.stdout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .bytes
            .clone()
    }
    fn observe(&mut self, kind: &'static str, event: Option<&Envelope>) {
        if self.observations.len() == 16 {
            self.observations.remove(0);
        }
        self.observations.push(serde_json::json!({
            "kind":kind,"elapsed_ns":self.started.elapsed().as_nanos(),
            "session":event.map(|e| e.session),"epoch":event.map(|e| e.epoch),
            "seq":event.and_then(|e| e.seq),"at":event.map(|e| e.at),
        }));
    }
    fn input(&mut self, bytes: &[u8]) {
        self.writer.as_mut().unwrap().write_all(bytes).unwrap();
        self.writer.as_mut().unwrap().flush().unwrap();
    }
    fn finish(mut self, crash: bool) {
        if crash {
            assert_eq!(
                unsafe { libc::kill(self.child.pid() as i32, libc::SIGKILL) },
                0
            );
        } else {
            if matches!(self.child, Child::Gui(_)) {
                self.input(b"/quit\n");
            } else {
                self.input(b"\x04");
            }
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                if !crash {
                    assert!(status, "{}", String::from_utf8_lossy(&self.output()));
                }
                break;
            }
            assert!(Instant::now() < deadline, "seat failed to exit");
            std::thread::sleep(Duration::from_millis(5));
        }
        drop(self.writer.take());
        drop(self.master.take());
        for reader in self.readers.drain(..) {
            reader.join().unwrap();
        }
        let discarded = self
            .stdout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .discarded;
        assert_eq!(
            discarded, 0,
            "normal fixture requires a complete bounded stdout observation"
        );
        let bytes = self.output();
        assert!(
            !String::from_utf8_lossy(&bytes).contains("\x1b]7502"),
            "no in-band discovery"
        );
        // CSI may style text, but must never address the cursor or redraw.
        assert!(
            !regex::bytes::Regex::new(r"\x1b\[[0-9;]*[^m0-9;]")
                .unwrap()
                .is_match(&bytes)
        );
        self.finished = true;
    }
}
fn durable_tail(root: &Path) -> serde_json::Value {
    let mut files = vec![];
    if let Ok(entries) = std::fs::read_dir(root.join(".bboxy450-harness/sessions")) {
        for path in entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
            .take(4)
        {
            let read = (|| -> std::io::Result<Vec<serde_json::Value>> {
                let meta = std::fs::symlink_metadata(&path)?;
                if !meta.is_file() || meta.file_type().is_symlink() {
                    return Err(std::io::Error::other("not a regular owned fixture log"));
                }
                let mut file = std::fs::File::open(&path)?;
                let offset = meta.len().saturating_sub(OUTPUT_LIMIT as u64);
                file.seek(SeekFrom::Start(offset))?;
                let mut bytes = vec![];
                file.take(OUTPUT_LIMIT as u64).read_to_end(&mut bytes)?;
                let mut rows = std::collections::VecDeque::new();
                for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
                    if line.is_empty() || (offset != 0 && index == 0) {
                        continue;
                    }
                    let row = match serde_json::from_slice::<serde_json::Value>(line) {
                        Ok(event) => serde_json::json!({"kind":event["event"]["type"],
                            "session":event["session"],"epoch":event["epoch"],
                            "seq":event["seq"],"at":event["at"],"cause":event["cause"]}),
                        Err(_) => serde_json::json!({"invalid_or_partial_json":true}),
                    };
                    if rows.len() == 16 {
                        rows.pop_front();
                    }
                    rows.push_back(row);
                }
                Ok(rows.into())
            })();
            files.push(
                serde_json::json!({"file":path.file_name().and_then(|s|s.to_str()),
                "tail":read.as_ref().ok(),"error":read.err().map(|e|e.to_string())}),
            );
        }
    }
    files.into()
}
fn private_evidence(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)
}
impl Drop for Seat {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let pid = self.child.pid();
        let before = self.child.status_text();
        let before_text = format!("{before:?}");
        let mut settled = matches!(before, Ok(Some(_)));
        let mut known = before.is_ok();
        let durable_before = durable_tail(&self.root);
        let stdout_before = self
            .stdout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .bytes
            .len();
        let stderr_before = self
            .stderr
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .bytes
            .len();
        drop(self.writer.take());
        drop(self.master.take());
        let mut signals = vec![];
        // Only this unreaped Child pins this exact PID. EOF/TERM/KILL are
        // failure cleanup, never evidence that the original Close was delivered.
        for (signal, wait) in [
            (libc::SIGTERM, Duration::from_millis(250)),
            (libc::SIGKILL, Duration::from_secs(2)),
        ] {
            if settled || !known {
                break;
            }
            let result = unsafe { libc::kill(pid as i32, signal) };
            let errno = if result != 0 {
                std::io::Error::last_os_error().raw_os_error()
            } else {
                None
            };
            signals.push((signal, result, errno));
            let until = Instant::now() + wait;
            loop {
                match self.child.status_text() {
                    Ok(Some(_)) => {
                        settled = true;
                        break;
                    }
                    Ok(None) if Instant::now() < until => {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Ok(None) => break,
                    Err(_) => {
                        known = false;
                        break;
                    }
                }
            }
        }
        let after = format!("{:?}", self.child.status_text());
        let reader_count = self.readers.len();
        let reader_deadline = Instant::now() + Duration::from_millis(250);
        while self.readers.iter().any(|reader| !reader.is_finished())
            && Instant::now() < reader_deadline
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut readers_joined = true;
        let mut readers_detached = 0;
        for reader in self.readers.drain(..) {
            if reader.is_finished() {
                readers_joined &= reader.join().is_ok();
            } else {
                // Another owned writer may retain a pipe after leader reap.
                // A diagnostic reader is disposable; never turn its missing
                // EOF into an unbounded join or an asserted terminal fact.
                readers_joined = false;
                readers_detached += 1;
            }
        }
        let stdout = self.stdout.lock().unwrap_or_else(|e| e.into_inner());
        let stderr = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        let record = serde_json::json!({"stage":self.stage,"pid":pid,
            "elapsed_ns":self.started.elapsed().as_nanos(),"status_before_settlement":before_text,
            "status_after_settlement":after,"settled":settled,"signals":signals,
            "reader_wait_limit_ns":250000000,"reader_count":reader_count,
            "readers_joined":readers_joined,"readers_detached":readers_detached,"observations":self.observations,
            "stdout_before_settlement_bytes":stdout_before,"stderr_before_settlement_bytes":stderr_before,
            "stdout_bytes":stdout.bytes.len(),"stdout_eof":stdout.eof,"stdout_error":stdout.error,"stdout_discarded":stdout.discarded,
            "stderr_bytes":stderr.bytes.len(),"stderr_eof":stderr.eof,"stderr_error":stderr.error,"stderr_discarded":stderr.discarded,
            "durable_before_settlement":durable_before,"durable_after_settlement":durable_tail(&self.root)});
        let prefix = format!("seat-failure-{pid}");
        let wrote = (|| -> std::io::Result<()> {
            private_evidence(
                &self.root.join(format!("{prefix}-stdout.log")),
                &stdout.bytes,
            )?;
            private_evidence(
                &self.root.join(format!("{prefix}-stderr.log")),
                &stderr.bytes,
            )?;
            private_evidence(
                &self.root.join(format!("{prefix}.json")),
                &serde_json::to_vec(&record)?,
            )
        })();
        eprintln!(
            "seat-fixture failure_evidence={} record={record} write_result={wrote:?}",
            self.root.join(format!("{prefix}.json")).display()
        );
    }
}
fn wait_removed(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while path.exists() {
        assert!(Instant::now() < deadline, "socket not removed");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn wait_hello(
    root: &Path,
    gui: bool,
    previous: Option<SessionId>,
    seat: &mut Seat,
) -> (PathBuf, bboxy450_harness::gateway::Hello) {
    bbt_rt::handle().block_on(async {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                if let Some(status) = seat.child.try_wait().unwrap() {
                    panic!(
                        "socket child exited {status:?}: {}",
                        String::from_utf8_lossy(&seat.output())
                    );
                }
                let mut paths = if gui {
                    std::fs::read_dir(root.join(".bboxy450-harness/run"))
                        .into_iter()
                        .flatten()
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| {
                            p.extension().is_some_and(|ext| ext == "sock")
                                && p.file_stem()
                                    .and_then(|s| s.to_str())
                                    .and_then(|s| s.parse().ok())
                                    .map(SessionId)
                                    .is_some_and(|id| Some(id) != previous)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![root.join("seat.sock")]
                };
                paths.sort();
                // A crash can leave several retired addresses. Probe every
                // candidate; an unavailable first entry cannot hide the live one.
                for path in paths {
                    if let Ok((client, sub)) = UnixClient::discover(&path, None, false, false).await
                    {
                        let hello = sub.hello().clone();
                        client.close();
                        if Some(hello.meta.session) != previous {
                            use std::os::unix::fs::PermissionsExt;
                            assert_eq!(
                                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                                0o600
                            );
                            return (path, hello);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            panic!(
                "session socket did not bind: gui={gui}, previous={previous:?}, output={}",
                String::from_utf8_lossy(&seat.output())
            )
        })
    })
}
fn lifecycle(gui: bool) {
    let own = FailureHome(
        tempfile::Builder::new()
            .prefix("pane-")
            .tempdir_in("/tmp")
            .unwrap(),
    );
    if gui {
        let run = own.path().join(".bboxy450-harness/run");
        std::fs::create_dir_all(&run).unwrap();
        // Deterministic regression: the lowest candidate is a real stale socket,
        // left by a retired/crashed generation before the current address binds.
        let stale = run.join("00000000000000000000000000.sock");
        drop(std::os::unix::net::UnixListener::bind(&stale).unwrap());
        assert!(stale.exists());
    }
    let mut seat = Seat::start(own.path(), gui, None);
    seat.stage = "initial_hello";
    let (path, first) = wait_hello(own.path(), gui, None, &mut seat);
    let first_path = own
        .path()
        .join(format!(".bboxy450-harness/run/{}.sock", first.meta.session));
    assert!(first_path.exists(), "per-session socket also binds");
    let (client, mut sub, op) = bbt_rt::handle().block_on(async {
        let (client, mut sub) = UnixClient::discover(&path, None, false, false)
            .await
            .unwrap();
        let op = OpEnvelope {
            v: PROTOCOL,
            session: first.meta.session,
            op_id: UnixClient::message_op_id("private-test-message"),
            op: Op::UserTurn {
                blocks: vec![InputBlock::Text {
                    text: "one socket turn".into(),
                }],
                mode: TurnMode::Queue,
            },
        };
        assert!(!client.send(op.clone()).await.unwrap().duplicate);
        assert!(client.send(op.clone()).await.unwrap().duplicate);
        let events = tokio::time::timeout(Duration::from_secs(10), async {
            let mut events = vec![];
            while let Some(e) = sub.recv().await {
                let done = matches!(e.event, Event::TurnEnded { .. });
                events.push(e);
                if done {
                    break;
                }
            }
            events
        })
        .await
        .unwrap();
        assert_eq!(
                events
                    .iter()
                    .filter(|e| e.cause == Some(op.op_id)
                        && matches!(e.event, Event::TurnQueued { .. }))
                    .count(),
                1
            );
        assert!(
            events
                .iter()
                .any(|e| matches!(e.event, Event::TurnEnded { .. }))
        );
        (client, sub, op)
    });
    seat.observe("turn_completed", None);
    // Exercise both socket SetProfile and the original terminal command.
    seat.stage = "profile_switch_command";
    if gui {
        seat.input(b"/profile other\n");
    } else {
        bbt_rt::handle()
            .block_on(client.send(OpEnvelope {
                v: PROTOCOL,
                session: first.meta.session,
                op_id: OpId::new(),
                op: Op::SetProfile {
                    profile: "other".into(),
                },
            }))
            .unwrap();
    }
    seat.observe("switch_command_written_or_socket_acked", None);
    seat.stage = "old_session_closed_wait";
    bbt_rt::handle().block_on(async {
        tokio::time::timeout(Duration::from_secs(2), async {
            while let Some(e) = sub.recv().await {
                seat.observe(
                    if matches!(e.event, Event::SessionClosed) {
                        "subscriber_session_closed"
                    } else {
                        "subscriber_event"
                    },
                    Some(&e),
                );
                if matches!(e.event, Event::SessionClosed) {
                    return;
                }
            }
            panic!("old subscriber lost SessionClosed");
        })
        .await
        .unwrap();
    });
    seat.stage = "second_hello";
    let switched = Instant::now();
    let (second_path, second) = wait_hello(own.path(), gui, Some(first.meta.session), &mut seat);
    assert!(switched.elapsed() < Duration::from_secs(2));
    wait_removed(&first_path);
    if !gui {
        assert_eq!(second_path, path);
    }
    bbt_rt::handle().block_on(async {
        let (next, _) = UnixClient::discover(&second_path, None, false, false)
            .await
            .unwrap();
        assert_eq!(
            next.send(op.clone()).await,
            Err(bboxy450_harness::gateway::unix::ClientError::Refused(
                bboxy450_harness::gateway::Refused::UnknownSession
            ))
        );
        next.close();
    });
    seat.input(b"/profile scripted\n");
    seat.stage = "third_hello";
    let (third_path, third) = wait_hello(own.path(), gui, Some(second.meta.session), &mut seat);
    assert_ne!(
        first.meta.session, third.meta.session,
        "A-B-A is a third birth"
    );
    assert_ne!(second.meta.session, third.meta.session);
    seat.finish(true);
    assert!(third_path.exists(), "crash must leave a stale socket");
    let mut resumed = Seat::start(own.path(), gui, Some(&third.meta.session.to_string()));
    resumed.stage = "resumed_hello";
    // The stale address initially refuses connections; wait for the recovered epoch.
    let deadline = Instant::now() + Duration::from_secs(15);
    let reopened = loop {
        let (_, hello) = wait_hello(own.path(), gui, None, &mut resumed);
        if hello.epoch > third.epoch {
            break hello;
        }
        assert!(Instant::now() < deadline);
    };
    assert_eq!(reopened.meta.session, third.meta.session);
    resumed.finish(false);
    wait_removed(&third_path);
    let log = own.path().join(format!(
        ".bboxy450-harness/sessions/{}.jsonl",
        third.meta.session
    ));
    let events: Vec<Envelope> = std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    laws::check_transcript(&events).unwrap();
}
#[test]
fn headless_socket_start_switch_aba_resume_and_cleanup() {
    lifecycle(false);
}
#[test]
fn gui_socket_start_switch_aba_resume_and_cleanup() {
    lifecycle(true);
}

#[test]
fn gui_abandoned_fixture_retains_bounded_evidence_and_actually_reaps_its_child() {
    use std::os::unix::fs::PermissionsExt;
    let mut own = tempfile::Builder::new()
        .prefix("pane-evidence-control-")
        .tempdir_in("/tmp")
        .unwrap();
    own.disable_cleanup(true);
    let mut seat = Seat::start(own.path(), true, None);
    let _ = wait_hello(own.path(), true, None, &mut seat);
    let pid = seat.child.pid();
    seat.stage = "source_known_abandoned_fixture_control";
    drop(seat); // Actual failure-cleanup API, with the original Child still owned.
    let record_path = own.path().join(format!("seat-failure-{pid}.json"));
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
    assert_eq!(record["settled"], true);
    assert_eq!(record["readers_joined"], true);
    assert_eq!(record["stdout_eof"], true);
    assert_eq!(record["stderr_eof"], true);
    assert!(
        record["status_after_settlement"]
            .as_str()
            .unwrap()
            .starts_with("Ok(Some(")
    );
    for suffix in ["stdout.log", "stderr.log"] {
        let meta =
            std::fs::metadata(own.path().join(format!("seat-failure-{pid}-{suffix}"))).unwrap();
        assert!(meta.len() <= OUTPUT_LIMIT as u64);
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }
    let stderr = std::fs::read(own.path().join(format!("seat-failure-{pid}-stderr.log"))).unwrap();
    assert!(String::from_utf8_lossy(&stderr).contains("stage=worker_start"));
    eprintln!(
        "seat-fixture source_known_abandon_control pid={pid} actual_reap=true evidence={}",
        record_path.display()
    );
    // A separate actual Read/EOF control proves cap exhaustion drains instead
    // of blocking the owner or growing the retained diagnostic payload.
    let captured = Arc::new(Mutex::new(Output::default()));
    capture(
        Box::new(std::io::Cursor::new(vec![b'x'; OUTPUT_LIMIT + 113])),
        captured.clone(),
    )
    .join()
    .unwrap();
    let captured = captured.lock().unwrap();
    assert_eq!(captured.bytes.len(), OUTPUT_LIMIT);
    assert_eq!(captured.discarded, 113);
    assert!(captured.eof && captured.error.is_none());
}

#[test]
fn gui_failure_evidence_does_not_join_a_reader_whose_owned_peer_is_still_open() {
    let mut own = tempfile::Builder::new()
        .prefix("pane-held-reader-control-")
        .tempdir_in("/tmp")
        .unwrap();
    own.disable_cleanup(true);
    let mut seat = Seat::start(own.path(), true, None);
    let _ = wait_hello(own.path(), true, None, &mut seat);
    let pid = seat.child.pid();
    let (read, held) = std::os::unix::net::UnixStream::pair().unwrap();
    let captured = Arc::new(Mutex::new(Output::default()));
    seat.readers.push(capture(Box::new(read), captured.clone()));
    let (release, released) = std::sync::mpsc::channel();
    let peer = std::thread::spawn(move || {
        // A broken unbounded join must fail rather than hang the unattended
        // control. The correct path returns while this exact peer is open.
        let released_by_test = released.recv_timeout(Duration::from_secs(10)).is_ok();
        drop(held);
        released_by_test
    });
    seat.stage = "source_known_open_reader_control";
    drop(seat);
    let eof_at_return = captured.lock().unwrap().eof;
    let release_sent = release.send(()).is_ok();
    let released_by_test = peer.join().unwrap();
    let record_path = own.path().join(format!("seat-failure-{pid}.json"));
    let record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
    assert!(release_sent && released_by_test && !eof_at_return);
    assert_eq!(record["settled"], true);
    assert_eq!(record["reader_count"], 3);
    assert_eq!(record["readers_joined"], false);
    assert_eq!(record["readers_detached"], 1);
    let until = Instant::now() + Duration::from_secs(2);
    while !captured.lock().unwrap().eof && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(captured.lock().unwrap().eof);
    eprintln!(
        "seat-fixture source_known_open_reader_control pid={pid} actual_reap=true eof_at_return={eof_at_return} peer_released_after_return={released_by_test} evidence={}",
        record_path.display()
    );
}

// A GUI host worker, driven through private pipes without opening a window.
// A private HOME and scripted stdin drive startup/switch/resume without OS input.
#[test]
fn gui_socket_worker() {
    if std::env::var("HARNESS_GUI_SOCKET_WORKER").as_deref() != Ok("1") {
        return;
    }
    eprintln!("seat-fixture stage=worker_start pid={}", unsafe {
        libc::getpid()
    });
    use bboxy450_harness::{
        engine::Config,
        frontends::gui::bridge::Bridge,
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
        redact::Set,
    };
    use std::sync::Arc;
    struct Wake;
    impl bbt_rt::Waker for Wake {
        fn wake(&self) {}
    }
    bbt_rt::handle().block_on(async {
        use tokio::io::AsyncBufReadExt;
        let root = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".bboxy450-harness");
        let config = Config {
            meta: SessionMeta { vcs: None, dream: None, source: None, session: SessionId::new(), cwd: root.clone(), profile: Some("scripted".into()), fork: None, protocol: PROTOCOL, harness: env!("CARGO_PKG_VERSION").into(), created_ms: 0 },
            sessions: root.join("sessions"), profiles: root.join("profiles"), provider: Arc::new(ScriptedProvider::new(vec![include_bytes!("../fixtures/sse/text.sse").to_vec()])), model: "scripted".into(), profile_error: None, policy: Policy::new(Unattended::Deny), redactor: Set::default(), max_rounds: 50,
        };
        let mut bridge = Bridge::new(Arc::new(Wake));
        if let Ok(session) = std::env::var("HARNESS_GUI_SOCKET_RESUME") { bridge.resume(config, SessionId(session.parse().unwrap())); }
        else { bridge.create(config); }
        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
        let mut stopping = false;
        let mut last_ready = None;
        loop {
            bridge.drain(Duration::from_micros(300));
            assert!(bridge.state.error.is_none(), "{:?}", bridge.state.error);
            if bridge.state.ready && bridge.state.transcript.session != last_ready {
                last_ready = bridge.state.transcript.session;
                eprintln!("seat-fixture stage=ui_ready session={last_ready:?} cursor={:?}", bridge.state.transcript.cursor);
            }
            if bridge.state.transcript.closed { eprintln!("seat-fixture stage=worker_closed session={:?} cursor={:?}", bridge.state.transcript.session, bridge.state.transcript.cursor); break; }
            tokio::select! {
                line = lines.next_line(), if bridge.state.ready && !stopping => {
                    match line.unwrap() {
                        Some(line) if line.starts_with("/profile ") => {
                            eprintln!("seat-fixture stage=profile_command_read session={:?} cursor={:?}", bridge.state.transcript.session, bridge.state.transcript.cursor);
                            let sent = bridge.switch_profile(line[9..].trim().into());
                            eprintln!("seat-fixture stage=profile_command_enqueued result={sent:?}");
                            sent.unwrap();
                        },
                        _ => { eprintln!("seat-fixture stage=close_input session={:?}", bridge.state.transcript.session); bridge.send(Op::Close).unwrap(); stopping = true; }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(1)) => {}
            }
        }
        bridge.detach().unwrap().await.unwrap();
    });
}

#[test]
fn restart_retains_durable_message_id_and_has_one_effect() {
    let own = FailureHome(
        tempfile::Builder::new()
            .prefix("retry-")
            .tempdir_in("/tmp")
            .unwrap(),
    );
    let mut seat = Seat::start(own.path(), false, None);
    let (path, first) = wait_hello(own.path(), false, None, &mut seat);
    let op = OpEnvelope {
        v: PROTOCOL,
        session: first.meta.session,
        op_id: UnixClient::message_op_id("retry-after-restart"),
        op: Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "exactly one".into(),
            }],
            mode: TurnMode::Queue,
        },
    };
    bbt_rt::handle().block_on(async {
        let (client, mut sub) = UnixClient::discover(&path, None, false, false)
            .await
            .unwrap();
        client.send(op.clone()).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(e) = sub.recv().await {
                if matches!(e.event, Event::TurnEnded { .. }) {
                    break;
                }
            }
        })
        .await
        .unwrap();
    });
    seat.finish(true);
    let mut resumed = Seat::start(own.path(), false, Some(&first.meta.session.to_string()));
    let (_, second) = wait_hello(own.path(), false, None, &mut resumed);
    assert!(second.epoch > first.epoch);
    bbt_rt::handle().block_on(async {
        let (client, _) = UnixClient::discover(&path, None, false, false)
            .await
            .unwrap();
        assert!(client.send(op.clone()).await.unwrap().duplicate);
        client.close();
    });
    resumed.finish(false);
    let text = std::fs::read_to_string(own.path().join(format!(
        ".bboxy450-harness/sessions/{}.jsonl",
        first.meta.session
    )))
    .unwrap();
    let events: Vec<Envelope> = text
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|e| e.cause == Some(op.op_id) && matches!(e.event, Event::TurnQueued { .. }))
            .count(),
        1
    );
    laws::check_transcript(&events).unwrap();
}
