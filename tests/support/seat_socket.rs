// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use base64::{Engine, engine::general_purpose::STANDARD};
use bboxy450_harness::{
    gateway::{Subscribe, unix::UnixClient},
    protocol::*,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const SEAT: &str = "00000000000000000000000042";

fn announcements(bytes: &[u8]) -> Vec<serde_json::Value> {
    let text = String::from_utf8_lossy(bytes);
    let prefix = format!("\x1b]7502;v=1:seat={SEAT}:ev=HarnessSession:d=");
    let mut clean = text.to_string();
    let mut out = vec![];
    while let Some(start) = clean.find(&prefix) {
        let Some(end) = clean[start..].find("\x1b\\").map(|n| start + n + 2) else {
            break;
        };
        assert!(end - start <= 4096);
        let payload = &clean[start + prefix.len()..end - 2];
        let value: serde_json::Value =
            serde_json::from_slice(&STANDARD.decode(payload).unwrap()).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 1);
        assert_eq!(value["raw"].as_object().unwrap().len(), 4);
        assert_eq!(
            value["raw"]["protocol"],
            serde_json::to_value(PROTOCOL).unwrap()
        );
        out.push(value["raw"].clone());
        clean.replace_range(start..end, "");
    }
    // Output is append-only; all complete escapes must be this one hook frame.
    if !clean.contains(&prefix) {
        assert!(
            !clean.contains('\x1b'),
            "unexpected terminal control: {clean:?}"
        );
    }
    out
}

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
            c.args(["seat", "--profile", "scripted", "--scripted"]);
            c.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse"));
            if let Some(session) = resume {
                c.args(["--resume", session]);
            }
            c
        };
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
    fn announcement(&mut self, count: usize) -> serde_json::Value {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let values = announcements(&self.bytes);
            if values.len() >= count {
                assert_eq!(values.len(), count);
                return values[count - 1].clone();
            }
            let chunk = self
                .chunks
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|_| {
                    panic!("no announcement: {}", String::from_utf8_lossy(&self.bytes))
                });
            self.bytes.extend(chunk);
        }
    }
    fn input(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
    }
    fn finish(mut self, crash: bool, count: usize) {
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
        assert_eq!(announcements(&self.bytes).len(), count);
    }
}
fn wait_removed(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while path.exists() {
        assert!(Instant::now() < deadline, "socket not removed");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn socket(value: &serde_json::Value) -> PathBuf {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    let path = PathBuf::from(value["socket"].as_str().unwrap());
    assert!(path.is_absolute());
    assert_eq!(
        path.file_stem().unwrap().to_str().unwrap(),
        value["session"].as_str().unwrap()
    );
    let meta = std::fs::symlink_metadata(&path).unwrap();
    assert!(meta.file_type().is_socket());
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    path
}
fn lifecycle(gui: bool) {
    let own = tempfile::Builder::new()
        .prefix("pane-")
        .tempdir_in("/tmp")
        .unwrap();
    let mut seat = Seat::start(own.path(), gui, None);
    let first = seat.announcement(1);
    let first_path = socket(&first);
    bbt_rt::handle().block_on(async {
        let session = SessionId(first["session"].as_str().unwrap().parse().unwrap());
        let (client, mut sub) = UnixClient::connect(&first_path, Subscribe { session, after: None, ephemeral: false }).await.unwrap();
        let op = OpEnvelope { v: PROTOCOL, session, op_id: OpId::new(), op: Op::UserTurn { blocks: vec![InputBlock::Text { text: "one socket turn".into() }], mode: TurnMode::Queue } };
        assert!(!client.send(op.clone()).await.unwrap().duplicate);
        assert!(client.send(op.clone()).await.unwrap().duplicate);
        let events = tokio::time::timeout(Duration::from_secs(10), async {
            let mut events = vec![];
            while let Some(e) = sub.recv().await {
                let done = matches!(e.event, Event::TurnEnded { .. });
                events.push(e);
                if done { break; }
            }
            events
        }).await.unwrap();
        assert_eq!(events.iter().filter(|e| e.cause == Some(op.op_id) && matches!(e.event, Event::TurnQueued { .. })).count(), 1);
        assert!(events.iter().any(|e| matches!(e.event, Event::TurnEnded { .. })));
    });
    seat.input(b"/profile other\n");
    let second = seat.announcement(2);
    let second_path = socket(&second);
    wait_removed(&first_path);
    seat.input(b"/profile scripted\n");
    let third = seat.announcement(3);
    let third_path = socket(&third);
    wait_removed(&second_path);
    assert_ne!(first["session"], second["session"]);
    assert_ne!(first["session"], third["session"]);
    assert_ne!(second["session"], third["session"]);
    seat.finish(true, 3);
    assert!(
        third_path.exists(),
        "crash fixture must leave a stale socket"
    );
    let mut resumed = Seat::start(own.path(), gui, Some(third["session"].as_str().unwrap()));
    let reopened = resumed.announcement(1);
    assert_eq!(socket(&reopened), third_path);
    assert_eq!(reopened["session"], third["session"]);
    assert!(reopened["epoch"].as_u64().unwrap() > third["epoch"].as_u64().unwrap());
    resumed.finish(false, 1);
    wait_removed(&third_path);
}
#[test]
fn headless_socket_announces_start_switch_aba_resume_and_cleans_up() {
    lifecycle(false);
}
#[test]
fn gui_socket_announces_start_switch_aba_resume_and_cleans_up() {
    lifecycle(true);
}

// A GUI host worker, driven through a disposable PTY without opening a window.
// Re-executing this test also captures its real startup/switch/resume OSC bytes.
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
            meta: SessionMeta { session: SessionId::new(), cwd: root.clone(), profile: Some("scripted".into()), fork: None, protocol: PROTOCOL, harness: env!("CARGO_PKG_VERSION").into(), created_ms: 0 },
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
