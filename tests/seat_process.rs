// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};
#[cfg(feature = "test-fixtures")]
#[path = "support/seat_launch.rs"]
mod seat_launch;
#[cfg(feature = "test-fixtures")]
#[path = "support/seat_shutdown.rs"]
mod seat_shutdown;
#[path = "support/seat_socket.rs"]
mod seat_socket;
// Match portable-pty's pre-exec descriptor fence for the one non-PTY child.
// openpty publishes descriptors before setting CLOEXEC; concurrent fixtures
// must never hand a sibling's master to the key-resolution process.
fn spawn_without_sibling_fds(command: &mut std::process::Command) -> std::process::Child {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(|| {
            portable_pty::unix::close_random_fds();
            Ok(())
        });
    }
    command.spawn().unwrap()
}
fn launch(
    script: &str,
) -> (
    tempfile::TempDir,
    Box<dyn portable_pty::MasterPty + Send>,
    Box<dyn portable_pty::Child + Send + Sync>,
) {
    launch_options(script, true, false)
}
fn launch_options(
    script: &str,
    yolo: bool,
    json: bool,
) -> (
    tempfile::TempDir,
    Box<dyn portable_pty::MasterPty + Send>,
    Box<dyn portable_pty::Child + Send + Sync>,
) {
    let temp = tempfile::Builder::new()
        .prefix("harness-")
        .tempdir_in("/tmp")
        .unwrap();
    let pty = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    command.args(["seat", "--profile", "seat-test", "--cwd"]);
    command.arg(temp.path());
    command.arg("--sessions");
    command.arg(temp.path().join("sessions"));
    command.arg("--scripted");
    command.arg(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/sse/{script}.sse")),
    );
    command.env("HOME", temp.path());
    command.env("BBOXY_SOCK", temp.path().join("not-there"));
    if yolo {
        command.arg("--yolo");
    }
    if json {
        command.arg("--json");
    }
    let child = pty.slave.spawn_command(command).unwrap();
    drop(pty.slave);
    (temp, pty.master, child)
}

struct ReportingSeat {
    temp: tempfile::TempDir,
    _master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    receive: std::sync::mpsc::Receiver<Vec<u8>>,
    output: Vec<u8>,
}
impl ReportingSeat {
    fn spawn(script: &str, yolo: bool, json: bool) -> Self {
        let (temp, master, child) = launch_options(script, yolo, json);
        let writer = master.take_writer().unwrap();
        let mut reader = master.try_clone_reader().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = [0; 4096];
            while let Ok(n) = reader.read(&mut bytes) {
                if n == 0 || send.send(bytes[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Self {
            temp,
            _master: master,
            child,
            writer,
            receive,
            output: vec![],
        }
    }
    fn write(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
    }
    fn until(&mut self, predicate: impl Fn(&[u8]) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !predicate(&self.output) {
            let chunk = self
                .receive
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|e| {
                    panic!(
                        "seat output {e}: {:?}",
                        String::from_utf8_lossy(&self.output)
                    )
                });
            self.output.extend_from_slice(&chunk);
        }
    }
    fn close(&mut self) -> Vec<bboxy450_harness::protocol::Envelope> {
        self.write(b"\x04");
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "seat did not close after Ctrl-D");
            std::thread::sleep(Duration::from_millis(5));
        }
        let path = std::fs::read_dir(self.temp.path().join("sessions"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .unwrap();
        let log: Vec<_> = std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
        log
    }
}
impl Drop for ReportingSeat {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|w| w == needle)
}
fn report_count(bytes: &[u8]) -> usize {
    bytes
        .windows(b"\x1b]7501;state=".len())
        .filter(|w| *w == b"\x1b]7501;state=")
        .count()
}

#[test]
fn tty_seat_answers_a_probe_without_enter_and_filters_it_out_of_the_turn() {
    use bboxy450_harness::protocol::{Event, InputBlock, Item};
    let mut seat = ReportingSeat::spawn("text", true, false);
    seat.until(|out| contains(out, b"state=idle:title="));
    let reports = report_count(&seat.output);
    seat.write(b"hel\x1b]7501;");
    seat.write(b"?\x07");
    seat.until(|out| report_count(out) > reports);
    assert!(
        !contains(&seat.output, b"state=working"),
        "the probe must not submit a turn"
    );
    seat.write(b"lo\n");
    seat.until(|out| {
        contains(out, b"Done")
            && out.split(|b| *b == 0x1b).next_back().is_some_and(|last| {
                last.starts_with(b"]7501;state=idle:title=") && last.ends_with(b"\x07")
            })
    });
    assert!(contains(&seat.output, b"state=working:title="));
    let last_report = seat.output.split(|b| *b == 0x1b).next_back().unwrap();
    assert!(last_report.starts_with(b"]7501;state=idle:title="));
    seat.write(b"fail next\n");
    seat.until(|out| contains(out, b"state=error:title="));
    let reports = report_count(&seat.output);
    seat.write(b"\x1b]7501;?\x07");
    seat.until(|out| report_count(out) > reports);
    assert!(
        seat.output
            .split(|b| *b == 0x1b)
            .next_back()
            .unwrap()
            .starts_with(b"]7501;state=error:")
    );
    let log = seat.close();
    let users: Vec<_> = log
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::UserMessage { blocks },
                ..
            } => Some(blocks),
            _ => None,
        })
        .collect();
    assert!(
        matches!(&users[..], [a, b] if matches!(&a[..], [InputBlock::Text { text }] if text == "hello")
        && matches!(&b[..], [InputBlock::Text { text }] if text == "fail next"))
    );
}

#[test]
fn tty_seat_reports_a_pending_permission_and_resumes_after_unattended_denial() {
    let mut seat = ReportingSeat::spawn("bash", false, false);
    seat.until(|out| contains(out, b"state=idle:title="));
    seat.write(b"run a tool\n");
    seat.until(|out| contains(out, b":kind=permission\x07") && contains(out, b"state=error"));
    let text = String::from_utf8_lossy(&seat.output);
    let blocked = text.find("state=blocked").unwrap();
    assert!(
        text[blocked..].contains("state=working"),
        "approval completion restores working"
    );
    seat.close();
}

#[test]
fn silent_tool_keeps_reporting_working_until_interrupted() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let mut seat = ReportingSeat::spawn("bash", true, false);
    seat.until(|out| contains(out, b"state=idle:title="));
    let started = Instant::now();
    seat.write(b"run a quiet tool\n");
    let detail = STANDARD.encode("Run sleep 600 & sleep 600; wait");
    let needle = format!(":msg={detail}\x07");
    seat.until(|out| contains(out, needle.as_bytes()));
    // The initial phase report plus three periodic reports crosses term's old
    // five-second reported-state shield while the actual process stays silent.
    seat.until(|out| {
        out.windows(needle.len())
            .filter(|w| *w == needle.as_bytes())
            .count()
            >= 4
    });
    assert!(started.elapsed() >= Duration::from_secs(5));
    assert!(started.elapsed() < Duration::from_secs(9));
    let idle = seat
        .output
        .windows(b"state=idle".len())
        .filter(|w| *w == b"state=idle")
        .count();
    assert_eq!(
        unsafe { libc::kill(seat.child.process_id().unwrap() as i32, libc::SIGINT) },
        0
    );
    seat.until(|out| {
        out.windows(b"state=idle".len())
            .filter(|w| *w == b"state=idle")
            .count()
            > idle
    });
    let log = seat.close();
    assert!(log.iter().any(|e| matches!(
        &e.event,
        bboxy450_harness::protocol::Event::TurnEnded {
            reason: bboxy450_harness::protocol::StopReason::Cancelled,
            ..
        }
    )));
}

#[test]
fn json_seat_never_emits_state_reports_even_on_a_tty() {
    let mut seat = ReportingSeat::spawn("text", true, true);
    seat.until(|out| contains(out, b"session_started"));
    seat.write(b"hi\n");
    seat.until(|out| contains(out, b"turn_ended"));
    assert_eq!(report_count(&seat.output), 0);
    seat.close();
}

#[test]
fn piped_seat_never_emits_state_reports() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let mut child = spawn_without_sibling_fds(
        std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .args(["seat", "--scripted"])
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/sse/text.sse"),
            )
            .env("HOME", temp.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped()),
    );
    child.stdin.take().unwrap().write_all(b"hello\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(report_count(&output.stdout), 0);
}
#[test]
fn hangup_stops_without_an_operator() {
    let (_temp, master, mut child) = launch("bash");
    let mut writer = master.take_writer().unwrap();
    writer.write_all(b"start a tool\n").unwrap();
    writer.flush().unwrap();
    let mut reader = master.try_clone_reader().unwrap();
    let terminal_fd = master.as_raw_fd().unwrap();
    let reader_done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let reader_stop = reader_done.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader_thread = std::thread::spawn(move || {
        let mut b = [0; 4096];
        let mut all = vec![];
        let mut announced = false;
        let reader_deadline = Instant::now() + Duration::from_secs(7);
        while !reader_stop.load(std::sync::atomic::Ordering::Acquire)
            && Instant::now() < reader_deadline
        {
            let mut poll = libc::pollfd {
                fd: terminal_fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut poll, 1, 5) };
            if ready <= 0 {
                if ready < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR)
                {
                    break;
                }
                continue;
            }
            let Ok(n) = reader.read(&mut b) else {
                break;
            };
            if n == 0 {
                break;
            }
            let retained = n.min(65536_usize.saturating_sub(all.len()));
            all.extend_from_slice(&b[..retained]);
            if !announced && String::from_utf8_lossy(&all).contains("sleep 600") {
                let _ = tx.send(all.clone());
                announced = true;
            }
        }
        all
    });
    let observed = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let start = Instant::now();
    // Explicit hangup avoids racing PTY foreground-group signal delivery.
    let original_pid = child.process_id().unwrap();
    assert_eq!(unsafe { libc::kill(original_pid as i32, libc::SIGHUP) }, 0);
    // Retain the terminal until this explicit SIGHUP has settled. Closing the
    // master earlier adds a second physical hangup during the same teardown.
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    };
    reader_done.store(true, std::sync::atomic::Ordering::Release);
    // The retained master keeps this poll fd valid until the bounded reader
    // observes its stop flag; no descendant-held slave can block this join.
    let output = reader_thread.join().unwrap();
    drop(writer);
    drop(master);
    eprintln!(
        "hangup original_pid={original_pid} actual_exit={status:?} captured_terminal_bytes={} original_reaped_before_pty_close=true",
        output.len()
    );
    assert!(
        status.success(),
        "hangup original_pid={original_pid} status={status:?} pre_hangup_output={:?} terminal_output={:?}",
        String::from_utf8_lossy(&observed),
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn no_bboxy_sock_still_finishes_a_turn() {
    let (_temp, master, mut child) = launch("text");
    let mut writer = master.take_writer().unwrap();
    writer.write_all(b"hello\n").unwrap();
    writer.flush().unwrap();
    let mut reader = master.try_clone_reader().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let read = std::thread::spawn(move || {
        let mut b = [0; 4096];
        let mut all = vec![];
        while let Ok(n) = reader.read(&mut b) {
            if n == 0 {
                break;
            }
            all.extend_from_slice(&b[..n]);
            if String::from_utf8_lossy(&all).contains("✓ Done") {
                let _ = tx.send(all);
                return;
            }
        }
    });
    let output = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(String::from_utf8_lossy(&output).contains("Done."));
    read.join().unwrap();
    // This witnesses an unavailable socket, not master-descriptor lifetime.
    // Cooked EOF closes this seat even while sibling process fixtures run.
    writer.write_all(b"\x04").unwrap();
    writer.flush().unwrap();
    drop(writer);
    drop(master);
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn terminate_during_key_resolution_joins_the_key_process() {
    let temp = tempfile::Builder::new()
        .prefix("harness-key-")
        .tempdir_in("/tmp")
        .unwrap();
    let profiles = temp.path().join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let pid = temp.path().join("key.pid");
    std::fs::write(profiles.join("slow.md"),format!("---\nname: slow\nmodel: unused\nendpoint: http://127.0.0.1:9/v1\napi_key: '!echo $$ > {}.tmp; mv {}.tmp {}; sleep 600'\n---\n",pid.display(),pid.display(),pid.display())).unwrap();
    let mut child = spawn_without_sibling_fds(
        std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .args(["seat", "--profile", "slow", "--cwd"])
            .arg(temp.path())
            .env("HOME", temp.path())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped()),
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while !pid.exists() {
        assert!(Instant::now() < deadline, "key command not started");
        std::thread::sleep(Duration::from_millis(5));
    }
    let pgid: i32 = std::fs::read_to_string(pid)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let start = Instant::now();
    assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(unsafe { libc::kill(-pgid, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[test]
// scripts/test.py runs this case alone: it deliberately exposes a master fd.
fn non_pty_spawn_does_not_keep_a_sibling_master_alive() {
    use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
    let pty = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let slave = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(pty.master.tty_name().unwrap())
        .unwrap();
    let fd = pty.master.as_raw_fd().unwrap();
    // Deliberately expose the exact openpty -> CLOEXEC window, without timing.
    assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }, 0);
    let mut child = spawn_without_sibling_fds(
        std::process::Command::new("/bin/sh")
            .args(["-c", "printf R; exec sleep 600"])
            .env_clear()
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()),
    );
    child
        .stdout
        .as_mut()
        .unwrap()
        .read_exact(&mut [0u8; 1])
        .unwrap();
    drop(pty.master);
    let mut poll = libc::pollfd {
        fd: slave.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    unsafe {
        // Hang guard for kernel close and another concurrent pre-exec fence.
        libc::poll(&mut poll, 1, 2000);
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert_ne!(
        poll.revents & libc::POLLHUP,
        0,
        "non-PTY child retained a sibling master"
    );
}

#[test]
fn term_c1_owner_is_typed_and_accepted_before_seat_bootstrap() {
    let home = tempfile::tempdir_in("/private/tmp").unwrap();
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args(["seat", "--dream-owner", "term-c1", "--replay"])
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/ir-v1/minimal.jsonl"),
        )
        .env_clear()
        .env("HOME", home.path())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    // The fixture path need not exist: argument validation must reach replay IO.
    assert!(!String::from_utf8_lossy(&status.stderr).contains("dream-owner"));
    let invalid = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args(["seat", "--dream-owner", "other"])
        .env_clear()
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid dream owner"));
}

#[cfg(not(feature = "test-fixtures"))]
#[test]
fn dream_c1_retained_fixture_is_not_a_default_build_option() {
    use std::os::unix::process::CommandExt;
    let home = tempfile::tempdir_in("/private/tmp").unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    command
        .args([
            "seat",
            "--dream-c1-fixture-next-session",
            "0000000000000000000000002B",
        ])
        .env_clear()
        .env("HOME", home.path())
        .stdin(std::process::Stdio::null());
    unsafe {
        command.pre_exec(|| {
            portable_pty::unix::close_random_fds();
            Ok(())
        });
    }
    let output = command.output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option"));
    assert!(!home.path().join(".bboxy450-harness").exists());
}

#[cfg(feature = "test-fixtures")]
mod dream_c1_retained_fixture {
    use bboxy450_harness::{
        gateway::{Subscription, unix::UnixClient},
        protocol::*,
    };
    use std::{
        ffi::OsString,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        process::Stdio,
        time::Duration,
    };
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    const NEXT: &str = "0000000000000000000000002B";
    struct Paths {
        home: tempfile::TempDir,
        cwd: PathBuf,
        sessions: PathBuf,
        socket: PathBuf,
        script: PathBuf,
        managed: Option<Managed>,
        yolo: bool,
    }
    struct Managed {
        home: PathBuf,
        profile: PathBuf,
        mcp: PathBuf,
        capability: PathBuf,
        socket: PathBuf,
        receipt: PathBuf,
        _listener: std::os::unix::net::UnixListener,
    }
    impl Paths {
        fn new() -> Self {
            let home = tempfile::Builder::new()
                .prefix("c1-host-")
                .permissions(std::fs::Permissions::from_mode(0o700))
                .tempdir_in("/private/tmp")
                .unwrap();
            let cwd = home.path().join("cwd");
            let sessions = home.path().join("sessions");
            for dir in [&cwd, &sessions] {
                std::fs::create_dir(dir).unwrap();
                std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            let script = home.path().join("text.sse");
            std::fs::write(&script, include_bytes!("fixtures/sse/text.sse")).unwrap();
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o600)).unwrap();
            let socket = home.path().join("seat.sock");
            Self {
                home,
                cwd,
                sessions,
                socket,
                script,
                managed: None,
                yolo: false,
            }
        }
        fn home(&self) -> &Path {
            self.managed
                .as_ref()
                .map_or(self.home.path(), |m| m.home.as_path())
        }
        fn managed() -> Self {
            let mut paths = Self::new();
            let root = paths.home.path();
            let home = root.join("home");
            let repository = root.join("repository");
            let runtime = root.join("runtime");
            let overlay = runtime.join("overlay");
            for dir in [
                home.clone(),
                repository.clone(),
                runtime.clone(),
                overlay.clone(),
                home.join(".bboxy450-harness"),
                home.join(".bboxy450-harness/profiles"),
                home.join(".bboxy450-harness/sessions"),
                home.join(".bboxy450-harness/run"),
                overlay.join("zsh"),
                overlay.join("codex"),
                overlay.join("claude"),
            ] {
                std::fs::create_dir(&dir).unwrap();
                std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            let private_file = |path: &Path, bytes: &[u8]| {
                std::fs::write(path, bytes).unwrap();
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            };
            let profile = home.join(".bboxy450-harness/profiles/managed-c1.md");
            private_file(&profile, b"---\nname: managed-c1\nmodel: managed-scripted\nendpoint: http://127.0.0.1:9/v1\nauto_compact: false\n---\n");
            let capability = overlay.join("capability");
            private_file(&capability, format!("{}\n", "ab".repeat(32)).as_bytes());
            let socket = runtime.join("term.sock");
            let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
            std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
            let receipt = overlay.join("managed-receipts.jsonl");
            let server = overlay.join("mcp.py");
            private_file(&server, br#"# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness - https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
import json, os, sys
# Record public launch metadata only. Never open or print capability bytes.
record = {'pid': os.getpid(), 'seat': os.environ.get('BBOXY_SEAT_ID'),
          'generation': os.environ.get('BBOXY_GENERATION'), 'role': os.environ.get('BBOXY_ROLE'),
          'socket': os.environ.get('BBOXY_SOCK'),
          'capability_sibling': os.path.dirname(os.environ.get('BBOXY_CAPABILITY_FILE','')) == os.path.dirname(__file__),
          'legacy_capability': 'BBOXY_CAPABILITY' in os.environ}
fd = os.open(os.environ['C1_RECEIPT'], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
os.write(fd, (json.dumps(record, sort_keys=True) + '\n').encode())
os.close(fd)
for line in sys.stdin:
    request = json.loads(line)
    if 'id' not in request: continue
    method = request.get('method')
    if method == 'initialize':
        result = {'protocolVersion':'2025-11-25','capabilities':{'tools':{}},'serverInfo':{'name':'private-managed-fixture','version':'1'}}
    elif method == 'tools/list': result = {'tools':[]}
    else: result = {}
    print(json.dumps({'jsonrpc':'2.0','id':request['id'],'result':result}), flush=True)
"#);
            let mcp = overlay.join("mcp.json");
            let configuration = serde_json::json!({"mcpServers":{"managed":{
                "command":"/usr/bin/python3","args":["-I","-S",server],"timeout_s":2,
                "env":{"C1_RECEIPT":receipt,"BBOXY_SEAT_ID":"$BBOXY_SEAT_ID", "BBOXY_GENERATION":"$BBOXY_GENERATION", "BBOXY_ROLE":"$BBOXY_ROLE"}
            }}});
            private_file(&mcp, &serde_json::to_vec(&configuration).unwrap());
            for (name, target, content) in [
                ("write-a.sse", "yolo-a.txt", "private explicit --yolo A\n"),
                ("write-b.sse", "yolo-b.txt", "private explicit --yolo B\n"),
            ] {
                let call = serde_json::json!({"choices":[{"index":0,"delta":{"tool_calls":[{
                    "index":0,"id":name,"type":"function","function":{"name":"write", "arguments":serde_json::to_string(&serde_json::json!({"path":target,"content":content})).unwrap()}
                }]},"finish_reason":null}]});
                let finish = serde_json::json!({"choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]});
                private_file(
                    &overlay.join(name),
                    format!("data: {call}\n\ndata: {finish}\n\ndata: [DONE]\n\n").as_bytes(),
                );
            }
            private_file(
                &home.join(".bboxy450-harness/sessions/retained.keep"),
                b"retained private owner\n",
            );
            paths.cwd = repository;
            paths.sessions = home.join(".bboxy450-harness/sessions");
            paths.socket = runtime.join("seat.sock");
            paths.managed = Some(Managed {
                home,
                profile,
                mcp,
                capability,
                socket,
                receipt,
                _listener: listener,
            });
            paths.yolo = true;
            paths
        }
        fn args(&self) -> Vec<OsString> {
            let mut args: Vec<OsString> = ["seat", "--json", "--dream-owner", "term-c1"]
                .into_iter()
                .map(Into::into)
                .collect();
            for (flag, path) in [
                ("--cwd", &self.cwd),
                ("--sessions", &self.sessions),
                ("--socket", &self.socket),
                ("--scripted", &self.script),
                ("--scripted", &self.script),
                ("--scripted", &self.script),
            ] {
                args.push(flag.into());
                args.push(path.as_os_str().to_owned());
            }
            args.extend(["--dream-c1-fixture-next-session".into(), NEXT.into()]);
            if let Some(m) = &self.managed {
                remove(&mut args, "--scripted", true);
                for script in [
                    m.mcp.parent().unwrap().join("write-a.sse"),
                    self.script.clone(),
                    m.mcp.parent().unwrap().join("write-b.sse"),
                    self.script.clone(),
                    self.script.clone(),
                ] {
                    args.extend(["--scripted".into(), script.into_os_string()]);
                }
                args.extend(["--profile".into(), "managed-c1".into(), "--mcp-config".into(), m.mcp.as_os_str().to_owned(),
                    "--append-system-prompt".into(), "NAME: private managed fixture\nROLE: EXECUTOR\nPreserve this context through both sessions.".into()]);
            }
            if self.yolo {
                args.push("--yolo".into());
            }
            args
        }
        fn command(&self, args: Vec<OsString>) -> tokio::process::Command {
            let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
            command
                .args(args)
                .env_clear()
                .env("HOME", self.home())
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .env("LANG", "C")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            #[cfg(target_os = "macos")]
            command.env(
                "__CF_USER_TEXT_ENCODING",
                format!("0x{:X}:0x0:0x0", unsafe { libc::geteuid() }),
            );
            if let Some(m) = &self.managed {
                command
                    .env("BBOXY_SEAT_ID", "00000000000000000000000042")
                    .env("BBOXY_GENERATION", "7")
                    .env("BBOXY_ROLE", "executor")
                    .env("BBOXY_SOCK", &m.socket)
                    .env("BBOXY_CAPABILITY_FILE", &m.capability)
                    .env("TERM_PROGRAM", "bboxy450-term")
                    .env("TERM_PROGRAM_VERSION", "private-fixture")
                    .env("ZDOTDIR", m.mcp.parent().unwrap().join("zsh"))
                    .env("CODEX_HOME", m.mcp.parent().unwrap().join("codex"))
                    .env("CLAUDE_CONFIG_DIR", m.mcp.parent().unwrap().join("claude"));
            }
            unsafe {
                command.pre_exec(|| {
                    portable_pty::unix::close_random_fds();
                    Ok(())
                });
            }
            command
        }
        fn logs(&self) -> Vec<Vec<Envelope>> {
            let mut paths = std::fs::read_dir(&self.sessions)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|s| s == "jsonl"))
                .collect::<Vec<_>>();
            paths.sort();
            paths
                .into_iter()
                .map(|path| {
                    let events = std::fs::read_to_string(path)
                        .unwrap()
                        .lines()
                        .map(|line| serde_json::from_str::<Envelope>(line).unwrap())
                        .collect::<Vec<_>>();
                    laws::check_transcript(&events).unwrap();
                    events
                })
                .collect()
        }
    }
    fn remove(args: &mut Vec<OsString>, key: &str, value: bool) {
        while let Some(i) = args.iter().position(|s| s == key) {
            args.drain(i..i + if value { 2 } else { 1 });
        }
    }
    fn op(session: SessionId, op: Op) -> OpEnvelope {
        OpEnvelope {
            v: PROTOCOL,
            session,
            op_id: OpId::new(),
            op,
        }
    }
    async fn discover(path: &Path, session: Option<SessionId>) -> (UnixClient, Subscription) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Ok((client, sub)) = UnixClient::discover(path, None, false, false).await {
                    if session.is_none_or(|s| sub.hello().meta.session == s) {
                        break (client, sub);
                    }
                    client.close();
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("private retained Host did not publish its actual socket")
    }
    async fn turn(
        client: &UnixClient,
        sub: &mut Subscription,
        text: &str,
        op_id: OpId,
    ) -> Result<(), &'static str> {
        let mut request = op(
            sub.hello().meta.session,
            Op::UserTurn {
                blocks: vec![InputBlock::Text { text: text.into() }],
                mode: TurnMode::Queue,
            },
        );
        request.op_id = op_id;
        let ack = client
            .send(request)
            .await
            .map_err(|_| "turn_admission_failed")?;
        if ack.op_id != op_id || ack.duplicate {
            return Err("turn_ack_identity");
        }
        until_turn(sub, text, Some(op_id)).await
    }
    async fn until_turn(
        sub: &mut Subscription,
        text: &str,
        op_id: Option<OpId>,
    ) -> Result<(), &'static str> {
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut user = false;
            let mut queued = None;
            let mut started = None;
            loop {
                let event = sub.recv().await.ok_or("actual_session_subscription_ended")?;
                match event.event {
                    Event::TurnQueued { turn } if op_id.is_none_or(|id| event.cause == Some(id)) => {
                        if event.seq.is_none() { return Err("queued_not_durable"); }
                        queued = Some(turn);
                    }
                    Event::TurnStarted { turn, .. } if queued == Some(turn) => {
                        if event.seq.is_none() { return Err("started_not_durable"); }
                        started = Some(turn);
                    }
                    Event::ItemCompleted { body: Item::UserMessage { blocks }, .. } => {
                        user |= matches!(&blocks[..], [InputBlock::Text { text: found }] if found == text);
                    }
                    Event::TurnEnded { turn, reason, .. } => {
                        if !user { return Err("actual_user_input_missing"); }
                        if queued != Some(turn) || started != Some(turn) { return Err("actual_turn_lifecycle_missing"); }
                        if event.seq.is_none() { return Err("ending_not_durable"); }
                        if !matches!(reason, StopReason::EndTurn) { return Err("turn_ended_unsuccessfully"); }
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }).await.map_err(|_| "scripted_turn_timeout")?
    }
    async fn require_turn(
        result: Result<(), &'static str>,
        phase: &str,
        session: SessionId,
        paths: &mut Paths,
        child: &mut tokio::process::Child,
        stderr: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    ) {
        if let Err(reason) = result {
            // Retain only this owned private fixture. Read a small log suffix's
            // structural fields, never capability bytes or transcript payloads.
            for _ in 0..20 {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            let mut tails = vec![];
            for path in std::fs::read_dir(&paths.sessions)
                .unwrap()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
                .take(2)
            {
                let mut raw = vec![];
                std::io::Read::read_to_end(
                    &mut std::io::Read::take(std::fs::File::open(&path).unwrap(), 64 * 1024),
                    &mut raw,
                )
                .unwrap();
                let mut rows = raw
                    .split(|b| *b == b'\n')
                    .filter(|b| !b.is_empty())
                    .map(
                        |line| match serde_json::from_slice::<serde_json::Value>(line) {
                            Ok(e) => serde_json::json!({"seq":e["seq"],"kind":e["event"]["type"]}),
                            Err(_) => serde_json::json!({"invalid_json":true}),
                        },
                    )
                    .collect::<Vec<_>>();
                if rows.len() > 8 {
                    rows.drain(..rows.len() - 8);
                }
                tails.push(
                    serde_json::json!({"file":path.file_name().unwrap().to_str(),"tail":rows}),
                );
            }
            let bytes = stderr.lock().unwrap().clone();
            let summary = serde_json::json!({"phase":phase,"session":session,"reason":reason,
                "pid":child.id(),"status":format!("{:?}",child.try_wait().unwrap()),"durable":tails});
            let root = paths.home.path().to_owned();
            for (name, bytes) in [
                (
                    "failure-summary.json",
                    serde_json::to_vec(&summary).unwrap(),
                ),
                ("failure-stderr.log", bytes.clone()),
            ] {
                let path = root.join(name);
                std::fs::write(&path, bytes).unwrap();
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            paths.home.disable_cleanup(true);
            eprintln!(
                "private retained-Host failure {summary}; bounded child stderr={}; private evidence={}",
                String::from_utf8_lossy(&bytes),
                root.display()
            );
            panic!("private retained-Host turn failed in {phase}: {reason}");
        }
    }
    async fn close(client: &UnixClient, sub: &mut Subscription) -> Result<(), &'static str> {
        client
            .send(op(sub.hello().meta.session, Op::Close))
            .await
            .map_err(|_| "close_admission_failed")?;
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let e = sub.recv().await.ok_or("durable_close_subscription_ended")?;
                if matches!(e.event, Event::SessionClosed) {
                    if e.seq.is_none() {
                        return Err("close_not_durable");
                    }
                    return Ok(());
                }
            }
        })
        .await
        .map_err(|_| "durable_close_timeout")??;
        client.close();
        Ok(())
    }
    async fn stdout_until(
        receive: &mut tokio::sync::mpsc::Receiver<Envelope>,
        events: &mut Vec<Envelope>,
        predicate: impl Fn(&Envelope) -> bool,
    ) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let e = receive.recv().await.expect("private stdout reader ended");
                let found = predicate(&e);
                events.push(e);
                if found {
                    break;
                }
            }
        })
        .await
        .unwrap();
    }
    async fn peer_pid(path: &Path) -> i32 {
        let socket = tokio::net::UnixStream::connect(path).await.unwrap();
        socket
            .peer_cred()
            .unwrap()
            .pid()
            .expect("local Unix peer PID")
    }

    #[test]
    fn dream_c1_retained_host_creates_one_nonfork_b_after_a_and_keeps_real_input() {
        bbt_rt::handle().block_on(joined(Paths::new()));
    }

    #[test]
    fn dream_c1_managed_retained_host_preserves_private_launch_inputs() {
        bbt_rt::handle().block_on(joined(Paths::managed()));
    }

    #[test]
    fn dream_c1_managed_retained_host_without_yolo_keeps_unattended_denial() {
        let mut paths = Paths::managed();
        paths.yolo = false;
        bbt_rt::handle().block_on(joined(paths));
    }

    async fn joined(mut paths: Paths) {
        let saved_profile = paths
            .managed
            .as_ref()
            .map(|m| std::fs::read(&m.profile).unwrap());
        let saved_mcp = paths
            .managed
            .as_ref()
            .map(|m| std::fs::read(&m.mcp).unwrap());
        let expected_profile = paths.managed.as_ref().map(|_| "managed-c1".to_owned());
        let mut child = paths.command(paths.args()).spawn().unwrap();
        let pid = child.id().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut stderr = child.stderr.take().unwrap();
        let (send, mut receive) = tokio::sync::mpsc::channel(256);
        let reader = bbt_rt::handle().spawn(async move {
            while let Some(line) = stdout.next_line().await.unwrap() {
                send.send(serde_json::from_str::<Envelope>(&line).unwrap())
                    .await
                    .unwrap();
            }
        });
        let stderr_ring = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed_stderr = stderr_ring.clone();
        let diagnostics = bbt_rt::handle().spawn(async move {
            let mut chunk = [0u8; 1024];
            loop {
                let n = stderr.read(&mut chunk).await.unwrap();
                if n == 0 {
                    break;
                }
                let mut bytes = observed_stderr.lock().unwrap();
                bytes.extend_from_slice(&chunk[..n]);
                let trim = bytes.len().saturating_sub(16 * 1024);
                bytes.drain(..trim);
            }
        });
        let mut events = vec![];
        stdout_until(&mut receive, &mut events, |e| {
            matches!(e.event, Event::SessionStarted { .. })
        })
        .await;
        let a = events.last().unwrap().session;
        let (client_a, mut sub_a) = discover(&paths.socket, Some(a)).await;
        assert_eq!(sub_a.hello().meta.fork, None);
        assert_eq!(sub_a.hello().meta.profile, expected_profile);
        assert_eq!(peer_pid(&paths.socket).await, pid as i32);
        if paths.managed.is_some() {
            stdout_until(&mut receive, &mut events, |e| e.session == a && matches!(&e.event, Event::ItemCompleted { body: Item::Notice { text, .. }, .. } if text == "mcp managed: ready")).await;
        }
        let a_op = OpId("00000000000000000000000007".parse().unwrap());
        let b_op = OpId("00000000000000000000000107".parse().unwrap());
        let result = turn(&client_a, &mut sub_a, "A before closure", a_op).await;
        require_turn(result, "A", a, &mut paths, &mut child, &stderr_ring).await;
        let result = close(&client_a, &mut sub_a).await;
        require_turn(
            result,
            "A durable close",
            a,
            &mut paths,
            &mut child,
            &stderr_ring,
        )
        .await;
        stdout_until(&mut receive, &mut events, |e| {
            e.session == a && matches!(e.event, Event::SessionClosed)
        })
        .await;
        let b = SessionId(NEXT.parse().unwrap());
        let (client_b, mut sub_b) = discover(&paths.socket, Some(b)).await;
        assert_ne!(a, b);
        assert_eq!(sub_b.hello().meta.session, b);
        assert_eq!(sub_b.hello().meta.fork, None);
        assert_eq!(sub_b.hello().meta.profile, expected_profile);
        assert_eq!(peer_pid(&paths.socket).await, pid as i32);
        assert_eq!(child.id(), Some(pid));
        assert!(child.try_wait().unwrap().is_none());
        assert!(
            !paths
                .home()
                .join(format!(".bboxy450-harness/run/{a}.sock"))
                .exists()
        );
        if paths.managed.is_some() {
            stdout_until(&mut receive, &mut events, |e| e.session == b && matches!(&e.event, Event::ItemCompleted { body: Item::Notice { text, .. }, .. } if text == "mcp managed: ready")).await;
        }
        let result = turn(&client_b, &mut sub_b, "B through actual Unix socket", b_op).await;
        require_turn(
            result,
            "B Unix socket",
            b,
            &mut paths,
            &mut child,
            &stderr_ring,
        )
        .await;
        stdin
            .write_all(b"B through retained stdin\n")
            .await
            .unwrap();
        stdin.flush().await.unwrap();
        let result = until_turn(&mut sub_b, "B through retained stdin", None).await;
        require_turn(
            result,
            "B retained stdin",
            b,
            &mut paths,
            &mut child,
            &stderr_ring,
        )
        .await;
        let result = close(&client_b, &mut sub_b).await;
        require_turn(
            result,
            "B durable close",
            b,
            &mut paths,
            &mut child,
            &stderr_ring,
        )
        .await;
        stdout_until(&mut receive, &mut events, |e| {
            e.session == b && matches!(e.event, Event::SessionClosed)
        })
        .await;
        drop(stdin);
        let status = tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), reader)
            .await
            .unwrap()
            .unwrap();
        while let Some(event) = receive.recv().await {
            events.push(event);
        }
        diagnostics.await.unwrap();
        let stderr = stderr_ring.lock().unwrap().clone();
        assert!(status.success(), "{}", String::from_utf8_lossy(&stderr));
        let logs = paths.logs();
        assert_eq!(logs.len(), 2);
        // These exact identifiers contain the admitted generation digit. A
        // corrupted/redacted cause must fail durable decoding or this chain.
        for (session, id) in [(a, a_op), (b, b_op)] {
            assert!(id.to_string().ends_with("07"));
            let log = logs.iter().find(|log| log[0].session == session).unwrap();
            let queued = log
                .iter()
                .filter(|e| e.cause == Some(id))
                .filter_map(|e| match e.event {
                    Event::TurnQueued { turn } => Some(turn),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(queued.len(), 1);
            let turn = queued[0];
            assert_eq!(log.iter().filter(|e| matches!(e.event, Event::TurnStarted { turn: found, .. } if found == turn)).count(), 1);
            assert_eq!(log.iter().filter(|e| matches!(e.event, Event::TurnEnded { turn: found, reason: StopReason::EndTurn, .. } if found == turn)).count(), 1);
        }
        for log in &logs {
            assert!(matches!(&log[0].event, Event::SessionStarted { meta } if meta.fork.is_none()));
            assert_eq!(
                log.iter()
                    .filter(|e| matches!(e.event, Event::SessionClosed))
                    .count(),
                1
            );
            assert_eq!(
                log.iter()
                    .filter(|e| matches!(e.event, Event::TurnStarted { .. }))
                    .count(),
                if log[0].session == a { 1 } else { 2 }
            );
        }
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.event, Event::SessionStarted { .. }))
                .count(),
            2
        );
        assert!(!paths.socket.exists());
        assert!(
            !paths
                .home()
                .join(format!(".bboxy450-harness/run/{b}.sock"))
                .exists()
        );
        if paths.managed.is_none() {
            assert!(!paths.home().join(".bboxy450-harness/profiles").exists());
        }
        assert!(!paths.home().join(".bboxy450-harness/dream").exists());
        if let Some(m) = &paths.managed {
            for (session, target, content) in [
                (a, "yolo-a.txt", "private explicit --yolo A\n"),
                (b, "yolo-b.txt", "private explicit --yolo B\n"),
            ] {
                let log = logs.iter().find(|log| log[0].session == session).unwrap();
                let writes = log
                    .iter()
                    .filter_map(|e| match &e.event {
                        Event::ItemCompleted {
                            status,
                            body: Item::ToolCall { name, .. },
                            ..
                        } if name == "write" => Some(status),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(writes.len(), 1);
                let denied = log.iter().any(|e| {
                    matches!(
                        &e.event,
                        Event::ItemCompleted {
                            body: Item::Approval {
                                decision: Some(ApprovalDecision::Deny { .. }),
                                ..
                            },
                            ..
                        }
                    )
                });
                if paths.yolo {
                    assert!(matches!(writes[0], ItemStatus::Completed));
                    assert!(!denied);
                    assert_eq!(
                        std::fs::read(paths.cwd.join(target)).unwrap(),
                        content.as_bytes()
                    );
                } else {
                    assert!(!matches!(writes[0], ItemStatus::Completed));
                    assert!(denied);
                    assert!(!paths.cwd.join(target).exists());
                }
            }
            assert_eq!(std::fs::read(&m.profile).unwrap(), saved_profile.unwrap());
            assert_eq!(std::fs::read(&m.mcp).unwrap(), saved_mcp.unwrap());
            let metadata = bboxy450_harness::provider::profile::Profile::load(&m.profile)
                .await
                .unwrap();
            assert_eq!(metadata.name, "managed-c1");
            assert!(!metadata.auto_compact);
            assert!(logs.iter().all(|log| matches!(&log[0].event, Event::SessionStarted { meta } if meta.profile.as_deref() == Some("managed-c1"))));
            assert!(
                events
                    .iter()
                    .filter_map(|e| match &e.event {
                        Event::TurnStarted { model, .. } => Some(model),
                        _ => None,
                    })
                    .all(|m| m.as_deref() == Some("managed-scripted"))
            );
            let receipt = std::fs::read_to_string(&m.receipt)
                .unwrap()
                .lines()
                .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(receipt.len(), 2, "one real launcher MCP per Host birth");
            for record in receipt {
                assert_eq!(record["seat"], "00000000000000000000000042");
                assert_eq!(record["generation"], "7");
                assert_eq!(record["role"], "executor");
                assert_eq!(record["socket"].as_str(), m.socket.to_str());
                assert_eq!(record["capability_sibling"], true);
                assert_eq!(record["legacy_capability"], false);
                assert_eq!(
                    unsafe { libc::kill(record["pid"].as_i64().unwrap() as i32, 0) },
                    -1
                );
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
            }
            assert_eq!(
                std::fs::read(paths.sessions.join("retained.keep")).unwrap(),
                b"retained private owner\n"
            );
            eprintln!(
                "managed_fixture profile=true auto_compact=false mcp_births=2 seat=true generation=7 role=true exact_op_suffix07=2 durable_lifecycle=true same_pid=true yolo_explicit={} safe_write_policy_witnesses=2",
                paths.yolo
            );
        }
    }

    #[test]
    fn dream_c1_retained_fixture_physical_eof_never_creates_b() {
        bbt_rt::handle().block_on(async {
            let paths = Paths::new();
            let output = tokio::time::timeout(
                Duration::from_secs(5),
                paths.command(paths.args()).stdin(Stdio::null()).output(),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let logs = paths.logs();
            assert_eq!(logs.len(), 1);
            assert_ne!(logs[0][0].session.to_string(), NEXT);
            assert!(matches!(
                logs[0].last().unwrap().event,
                Event::SessionClosed
            ));
        });
    }

    #[test]
    fn dream_c1_retained_fixture_rejects_widening_before_bootstrap() {
        bbt_rt::handle().block_on(async {
            for mode in 0..21 {
                let paths = Paths::new();
                let mut args = paths.args();
                args.push("--yolo".into());
                match mode {
                    0 => remove(&mut args, "seat", false),
                    1 => remove(&mut args, "--json", false),
                    2 => remove(&mut args, "--scripted", true),
                    3 => remove(&mut args, "--dream-owner", true),
                    4 => args.extend(["--profile".into(), "forbidden".into()]),
                    5 => args.extend(["--mcp-config".into(), "forbidden".into()]),
                    6 => args.extend(["--resume".into(), NEXT.into()]),
                    7 => args.extend(["--replay".into(), "forbidden".into()]),
                    8 => args.push("acp".into()),
                    9 => remove(&mut args, "--cwd", true),
                    10 => remove(&mut args, "--sessions", true),
                    11 => remove(&mut args, "--socket", true),
                    12 => {
                        let i = args.iter().position(|s| s == "--cwd").unwrap();
                        args[i + 1] = "relative".into();
                    }
                    13 => std::fs::set_permissions(
                        paths.home(),
                        std::fs::Permissions::from_mode(0o755),
                    )
                    .unwrap(),
                    14 => {}
                    15 => args.extend(["--dream-c1-fixture-next-session".into(), NEXT.into()]),
                    16 => {
                        let n = args.len();
                        args[n - 1] = "invalid".into();
                    }
                    17 => {
                        let i = args.iter().position(|s| s == "--socket").unwrap();
                        args[i + 1] = "/private/tmp/foreign-c1.sock".into();
                    }
                    18 => args.push("--export-schema".into()),
                    19 => {
                        std::fs::set_permissions(
                            &paths.script,
                            std::fs::Permissions::from_mode(0o644),
                        )
                        .unwrap();
                    }
                    _ => {}
                }
                let mut command = paths.command(args);
                if mode == 14 {
                    command.env("OPENAI_API_KEY", "throwaway-fixture-refusal");
                }
                if mode == 20 {
                    command.env("C1_UNAPPROVED_ENV", "unsupported-fixture-input");
                }
                let output = tokio::time::timeout(
                    Duration::from_secs(5),
                    command.stdin(Stdio::null()).output(),
                )
                .await
                .unwrap()
                .unwrap();
                assert!(!output.status.success(), "accepted widening mode {mode}");
                assert!(
                    paths.logs().is_empty(),
                    "created session for refused mode {mode}"
                );
                assert!(!paths.home().join(".bboxy450-harness").exists());
                assert!(
                    !String::from_utf8_lossy(&output.stderr).contains("throwaway-fixture-refusal")
                );
            }
        });
    }
    #[test]
    fn dream_c1_managed_fixture_rejects_credentials_mixed_roots_and_unsafe_metadata() {
        bbt_rt::handle().block_on(async {
            for mode in 0..22 {
                let paths = Paths::managed();
                let managed = paths.managed.as_ref().unwrap();
                let mut args = paths.args();
                let other = tempfile::Builder::new()
                    .prefix("c1-other-")
                    .permissions(std::fs::Permissions::from_mode(0o700))
                    .tempdir_in("/private/tmp")
                    .unwrap();
                let sentinel = paths.home.path().join("credential-must-not-run");
                let profile_bytes = std::fs::read(&managed.profile).unwrap();
                match mode {
                    0..=6 => {}
                    7 => {
                        let text = String::from_utf8(profile_bytes.clone()).unwrap().replace(
                            "---\nname",
                            &format!("---\napi_key: '!touch {}'\nname", sentinel.display()),
                        );
                        std::fs::write(&managed.profile, text).unwrap();
                    }
                    8 => {
                        let text = String::from_utf8(profile_bytes.clone())
                            .unwrap()
                            .replace("auto_compact: false", "auto_compact: true");
                        std::fs::write(&managed.profile, text).unwrap();
                    }
                    9 => {
                        let text = String::from_utf8(profile_bytes.clone()).unwrap().replace(
                            "---\nname",
                            "---\nprovider: acp-agent\nagent: {command: ['/bin/false']}\nname",
                        );
                        std::fs::write(&managed.profile, text).unwrap();
                    }
                    10 => std::fs::write(&managed.mcp, b"not JSON").unwrap(),
                    11 => {
                        let mut config: serde_json::Value =
                            serde_json::from_slice(&std::fs::read(&managed.mcp).unwrap()).unwrap();
                        config["mcpServers"]["managed"]["env"]["C1_CREDENTIAL"] =
                            serde_json::json!(format!("!touch {}", sentinel.display()));
                        std::fs::write(&managed.mcp, serde_json::to_vec(&config).unwrap()).unwrap();
                    }
                    12 => {
                        let i = args.iter().position(|s| s == "--cwd").unwrap();
                        args[i + 1] = other.path().as_os_str().to_owned();
                    }
                    13 => {
                        let backup = managed.profile.with_extension("original");
                        std::fs::rename(&managed.profile, &backup).unwrap();
                        std::os::unix::fs::symlink(&backup, &managed.profile).unwrap();
                    }
                    14 => std::fs::set_permissions(
                        &managed.profile,
                        std::fs::Permissions::from_mode(0o644),
                    )
                    .unwrap(),
                    15 => std::fs::set_permissions(
                        &managed.capability,
                        std::fs::Permissions::from_mode(0o644),
                    )
                    .unwrap(),
                    16 => std::fs::set_permissions(
                        paths.home(),
                        std::fs::Permissions::from_mode(0o755),
                    )
                    .unwrap(),
                    17 => {
                        let backup = paths.script.with_extension("original");
                        std::fs::rename(&paths.script, &backup).unwrap();
                        std::os::unix::fs::symlink(&backup, &paths.script).unwrap();
                    }
                    18 => std::fs::hard_link(
                        &managed.profile,
                        managed.profile.with_extension("linked"),
                    )
                    .unwrap(),
                    19 => std::fs::write(&managed.profile, vec![b' '; 256 * 1024 + 1]).unwrap(),
                    20 => remove(&mut args, "--mcp-config", true),
                    _ => {}
                }
                let mut command = paths.command(args);
                match mode {
                    0 => {
                        command.env("BBOXY_UNAPPROVED", "unsupported");
                    }
                    1 => {
                        command.env("BBOXY_CAPABILITY", "never-a-raw-capability");
                    }
                    2 => {
                        command.env("BBOXY_GENERATION", "0");
                    }
                    3 => {
                        command.env("BBOXY_SEAT_ID", "not-a-canonical-seat");
                    }
                    4 => {
                        command.env("BBOXY_ROLE", "x".repeat(257));
                    }
                    5 => {
                        command.env("OPENAI_API_KEY", "never-a-provider-key");
                    }
                    6 => {
                        command.env("BBOXY_CAPABILITY_FILE", other.path().join("capability"));
                    }
                    21 => {
                        command.env("HOME", "/private/tmp");
                    }
                    _ => {}
                }
                let output = tokio::time::timeout(
                    Duration::from_secs(5),
                    command.stdin(Stdio::null()).output(),
                )
                .await
                .unwrap()
                .unwrap();
                assert!(!output.status.success(), "accepted managed negative {mode}");
                assert!(
                    String::from_utf8_lossy(&output.stderr)
                        .contains("private Dream C1 fixture refused ["),
                    "unclassified managed negative {mode}"
                );
                assert!(paths.logs().is_empty());
                assert!(
                    !managed.receipt.exists(),
                    "MCP started for rejected private inputs {mode}"
                );
                assert!(!sentinel.exists(), "credential command executed for {mode}");
            }
        });
    }

    #[test]
    fn dream_c1_managed_fixture_never_clobbers_a_reserved_log_or_socket_name() {
        bbt_rt::handle().block_on(async {
            for reserved_log in [false, true] {
                let paths = Paths::managed();
                let target = if reserved_log {
                    paths.sessions.join(format!("{NEXT}.jsonl"))
                } else {
                    paths.socket.clone()
                };
                std::fs::write(&target, b"original private owner\n").unwrap();
                std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
                let output = tokio::time::timeout(
                    Duration::from_secs(5),
                    paths.command(paths.args()).stdin(Stdio::null()).output(),
                )
                .await
                .unwrap()
                .unwrap();
                assert!(!output.status.success());
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains(if reserved_log {
                        "[next_log_exists]"
                    } else {
                        "[socket_exists]"
                    })
                );
                assert_eq!(std::fs::read(&target).unwrap(), b"original private owner\n");
                assert!(!paths.managed.as_ref().unwrap().receipt.exists());
            }
        });
    }
}
