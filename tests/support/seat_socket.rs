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

struct Seat {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    chunks: std::sync::mpsc::Receiver<Vec<u8>>,
    reader: std::thread::JoinHandle<()>,
    bytes: Vec<u8>,
}
impl Seat {
    fn start(root: &Path, gui: bool, resume: Option<&str>) -> Self {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = if gui {
            let mut c = CommandBuilder::new(std::env::current_exe().unwrap());
            c.args([
                "--exact",
                "seat_socket::gui_socket_worker",
                "--nocapture",
                "--test-threads=1",
            ]);
            c.env("HARNESS_GUI_SOCKET_WORKER", "1");
            if let Some(session) = resume {
                c.env("HARNESS_GUI_SOCKET_RESUME", session);
            }
            c
        } else {
            let mut c = CommandBuilder::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
            c.args(["seat", "--profile", "scripted", "--socket"]);
            c.arg(root.join("seat.sock"));
            c.arg("--scripted");
            c.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse"));
            if let Some(session) = resume {
                c.args(["--resume", session]);
            }
            c
        };
        // A direct invocation is isolated too, without relying on make's wrapper.
        for key in ["BBOXY_CAPABILITY", "BBOXY_CAPABILITY_FILE", "BBOXY_SOCK"] {
            command.env_remove(key);
        }
        command.env("HOME", root);
        command.env("TERM", "dumb");
        command.env("BBOXY_SEAT_ID", SEAT);
        let child = pty.slave.spawn_command(command).unwrap();
        drop(pty.slave);
        let writer = pty.master.take_writer().unwrap();
        let mut read = pty.master.try_clone_reader().unwrap();
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
            master: pty.master,
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
                unsafe { libc::kill(self.child.process_id().unwrap() as i32, libc::SIGKILL) },
                0
            );
        } else {
            self.input(b"\x04");
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                if !crash {
                    assert!(status.success(), "{}", String::from_utf8_lossy(&self.bytes));
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
) -> (PathBuf, bboxy450_harness::gateway::Hello) {
    bbt_rt::handle().block_on(async {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let path = if gui {
                    std::fs::read_dir(root.join(".bboxy450-harness/run"))
                        .ok()
                        .and_then(|entries| {
                            entries.flatten().map(|e| e.path()).find(|p| {
                                p.file_stem()
                                    .and_then(|s| s.to_str())
                                    .and_then(|s| s.parse().ok())
                                    .map(SessionId)
                                    != previous
                            })
                        })
                } else {
                    Some(root.join("seat.sock"))
                };
                if let Some(path) = path
                    && let Ok((client, sub)) = UnixClient::discover(&path, None, false, false).await
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
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("session socket did not bind")
    })
}
fn lifecycle(gui: bool) {
    let own = tempfile::Builder::new()
        .prefix("pane-")
        .tempdir_in("/tmp")
        .unwrap();
    let mut seat = Seat::start(own.path(), gui, None);
    let (path, first) = wait_hello(own.path(), gui, None);
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
    let (second_path, second) = wait_hello(own.path(), gui, Some(first.meta.session));
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
    let (third_path, third) = wait_hello(own.path(), gui, Some(second.meta.session));
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
        let (_, hello) = wait_hello(own.path(), gui, None);
        if hello.epoch > third.epoch {
            break hello;
        }
        assert!(Instant::now() < deadline);
    };
    assert_eq!(reopened.meta.session, third.meta.session);
    resumed.input(b"\n"); // release canonical EOT on the worker's stdin
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

// A GUI host worker, driven through a disposable PTY without opening a window.
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
    let seat = Seat::start(own.path(), false, None);
    let (path, first) = wait_hello(own.path(), false, None);
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
    let resumed = Seat::start(own.path(), false, Some(&first.meta.session.to_string()));
    let (_, second) = wait_hello(own.path(), false, None);
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
