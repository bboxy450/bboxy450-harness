// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{gateway::unix::UnixClient, protocol::*};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const SEAT: &str = "00000000000000000000000042";

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
}
struct Seat {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: Child,
    writer: Box<dyn Write + Send>,
    chunks: std::sync::mpsc::Receiver<Vec<u8>>,
    reader: std::thread::JoinHandle<()>,
    bytes: Vec<u8>,
}
impl Seat {
    fn start(root: &Path, gui: bool, resume: Option<&str>) -> Self {
        let (master, child, writer, mut read): (_, _, Box<dyn Write + Send>, Box<dyn Read + Send>) =
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
                    .stdout(Stdio::piped());
                for key in ["BBOXY_CAPABILITY", "BBOXY_CAPABILITY_FILE", "BBOXY_SOCK"] {
                    command.env_remove(key);
                }
                if let Some(session) = resume {
                    command.env("HARNESS_GUI_SOCKET_RESUME", session);
                }
                let mut child = super::spawn_without_sibling_fds(&mut command);
                let writer = Box::new(child.stdin.take().unwrap());
                let read = Box::new(child.stdout.take().unwrap());
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
        let (tx, chunks) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut buffer = [0; 4096];
            while let Ok(n) = read.read(&mut buffer) {
                if n == 0 || tx.send(buffer[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Self {
            master,
            child,
            writer,
            chunks,
            reader,
            bytes: vec![],
        }
    }
    fn input(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
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
                    assert!(status, "{}", String::from_utf8_lossy(&self.bytes));
                }
                break;
            }
            assert!(Instant::now() < deadline, "seat failed to exit");
            std::thread::sleep(Duration::from_millis(5));
        }
        drop(self.writer);
        drop(self.master);
        self.reader.join().unwrap();
        for chunk in self.chunks.try_iter() {
            self.bytes.extend(chunk);
        }
        assert!(
            !String::from_utf8_lossy(&self.bytes).contains("\x1b]7502"),
            "no in-band discovery"
        );
        // CSI may style text, but must never address the cursor or redraw.
        assert!(
            !regex::bytes::Regex::new(r"\x1b\[[0-9;]*[^m0-9;]")
                .unwrap()
                .is_match(&self.bytes)
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
                for chunk in seat.chunks.try_iter() {
                    seat.bytes.extend(chunk);
                }
                if let Some(status) = seat.child.try_wait().unwrap() {
                    panic!(
                        "socket child exited {status:?}: {}",
                        String::from_utf8_lossy(&seat.bytes)
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
                String::from_utf8_lossy(&seat.bytes)
            )
        })
    })
}
fn lifecycle(gui: bool) {
    let own = tempfile::Builder::new()
        .prefix("pane-")
        .tempdir_in("/tmp")
        .unwrap();
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
    // Exercise both socket SetProfile and the original terminal command.
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
    bbt_rt::handle().block_on(async {
        tokio::time::timeout(Duration::from_secs(2), async {
            while let Some(e) = sub.recv().await {
                if matches!(e.event, Event::SessionClosed) {
                    return;
                }
            }
            panic!("old subscriber lost SessionClosed");
        })
        .await
        .unwrap();
    });
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
    let (third_path, third) = wait_hello(own.path(), gui, Some(second.meta.session), &mut seat);
    assert_ne!(
        first.meta.session, third.meta.session,
        "A-B-A is a third birth"
    );
    assert_ne!(second.meta.session, third.meta.session);
    seat.finish(true);
    assert!(third_path.exists(), "crash must leave a stale socket");
    let mut resumed = Seat::start(own.path(), gui, Some(&third.meta.session.to_string()));
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

// A GUI host worker, driven through private pipes without opening a window.
// A private HOME and scripted stdin drive startup/switch/resume without OS input.
#[test]
fn gui_socket_worker() {
    if std::env::var("HARNESS_GUI_SOCKET_WORKER").as_deref() != Ok("1") {
        return;
    }
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
            meta: SessionMeta { source: None, session: SessionId::new(), cwd: root.clone(), profile: Some("scripted".into()), fork: None, protocol: PROTOCOL, harness: env!("CARGO_PKG_VERSION").into(), created_ms: 0 },
            sessions: root.join("sessions"), profiles: root.join("profiles"), provider: Arc::new(ScriptedProvider::new(vec![include_bytes!("../fixtures/sse/text.sse").to_vec()])), model: "scripted".into(), profile_error: None, policy: Policy::new(Unattended::Deny), redactor: Set::default(), max_rounds: 50,
        };
        let mut bridge = Bridge::new(Arc::new(Wake));
        if let Ok(session) = std::env::var("HARNESS_GUI_SOCKET_RESUME") { bridge.resume(config, SessionId(session.parse().unwrap())); }
        else { bridge.create(config); }
        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
        let mut stopping = false;
        loop {
            bridge.drain(Duration::from_micros(300));
            assert!(bridge.state.error.is_none(), "{:?}", bridge.state.error);
            if bridge.state.transcript.closed { break; }
            tokio::select! {
                line = lines.next_line(), if bridge.state.ready && !stopping => {
                    match line.unwrap() {
                        Some(line) if line.starts_with("/profile ") => bridge.switch_profile(line[9..].trim().into()).unwrap(),
                        _ => { bridge.send(Op::Close).unwrap(); stopping = true; }
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
    let own = tempfile::Builder::new()
        .prefix("retry-")
        .tempdir_in("/tmp")
        .unwrap();
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
