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
    let (tx, rx) = std::sync::mpsc::channel();
    let reader_thread = std::thread::spawn(move || {
        let mut b = [0; 4096];
        let mut all = vec![];
        while let Ok(n) = reader.read(&mut b) {
            if n == 0 {
                break;
            }
            all.extend_from_slice(&b[..n]);
            if String::from_utf8_lossy(&all).contains("sleep 600") {
                let _ = tx.send(());
                break;
            }
        }
    });
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    reader_thread.join().unwrap();
    let start = Instant::now();
    // Explicit hangup avoids racing PTY foreground-group signal delivery.
    assert_eq!(
        unsafe { libc::kill(child.process_id().unwrap() as i32, libc::SIGHUP) },
        0
    );
    drop(writer);
    drop(master);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    }
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
