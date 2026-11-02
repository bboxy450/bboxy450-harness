// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

use bboxy450_harness::protocol::{Envelope, Event, laws::check_transcript};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
struct Seat {
    root: PathBuf,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    input: Option<Box<dyn Write + Send>>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    reaped: bool,
}
fn terminal_writer(master: &dyn portable_pty::MasterPty) -> Box<dyn Write + Send> {
    use std::os::fd::FromRawFd;
    // portable-pty's writer Drop submits newline + VEOF. A hangup witness must
    // close descriptors without first sending the graceful Ctrl-D operation.
    let fd = unsafe { libc::fcntl(master.as_raw_fd().unwrap(), libc::F_DUPFD_CLOEXEC, 3) };
    assert!(fd >= 0, "PTY writer duplication failed");
    Box::new(unsafe { std::fs::File::from_raw_fd(fd) })
}
fn wait_for(mut test: impl FnMut() -> bool, budget: Duration, message: &str) {
    let end = Instant::now() + budget;
    while !test() {
        assert!(Instant::now() < end, "{message}");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn group_pid(root: &Path, name: &str) -> Option<i32> {
    std::fs::read_to_string(root.join(name))
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}
fn group_dead(pgid: i32) -> bool {
    (unsafe { libc::kill(-pgid, 0) }) == -1
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
}

fn events(root: &Path) -> Vec<Envelope> {
    std::fs::read_dir(root.join("sessions"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .flat_map(|s| {
            s.lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect::<Vec<_>>()
        })
        .collect()
}
impl Seat {
    fn spawn(root: &Path, resume: Option<&str>, mcp: bool) -> Self {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        command.env_clear();
        command.env("HOME", root);
        command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
        command.args(["seat", "--yolo", "--cwd"]);
        command.arg(root);
        command.arg("--sessions");
        command.arg(root.join("sessions"));
        command.arg("--scripted");
        command.arg(root.join("tool.sse"));
        if mcp {
            command.arg("--mcp-config");
            command.arg(root.join("mcp.json"));
        }
        if let Some(id) = resume {
            command.args(["--resume", id]);
        }
        let child = pty.slave.spawn_command(command).unwrap();
        drop(pty.slave);
        let input = terminal_writer(pty.master.as_ref());
        Self {
            root: root.into(),
            master: Some(pty.master),
            input: Some(input),
            child,
            reaped: false,
        }
    }
    fn write(&mut self, bytes: &[u8]) {
        let input = self.input.as_mut().unwrap();
        input.write_all(bytes).unwrap();
        input.flush().unwrap();
    }
    fn wait_prompt(&self) {
        let fd = self.master.as_ref().unwrap().as_raw_fd().unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        let mut output = Vec::new();
        while !String::from_utf8_lossy(&output).contains("› ") {
            assert!(Instant::now() < end, "seat prompt did not become ready");
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 20) } > 0 && poll.revents & libc::POLLIN != 0 {
                let mut bytes = [0u8; 4096];
                let n = unsafe { libc::read(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
                assert!(n > 0, "PTY ended before prompt");
                output.extend_from_slice(&bytes[..n as usize]);
            }
        }
    }
    fn exit(&mut self, budget: Duration) -> portable_pty::ExitStatus {
        let mut observed = None;
        wait_for(
            || {
                observed = self.child.try_wait().unwrap();
                observed.is_some()
            },
            budget,
            "seat exit hang guard",
        );
        self.reaped = true;
        observed.unwrap()
    }
}
impl Drop for Seat {
    fn drop(&mut self) {
        self.input.take();
        self.master.take();
        // Clear descendants before waiting: Darwin can keep a killed session
        // leader in exit while its orphaned PTY holders remain alive.
        for name in ["group.pid", "mcp-group.pid"] {
            if let Some(pgid) = group_pid(&self.root, name) {
                unsafe {
                    libc::kill(-pgid, libc::SIGKILL);
                }
            }
        }
        if !self.reaped {
            unsafe {
                libc::kill(self.child.process_id().unwrap() as i32, libc::SIGKILL);
            }
            let end = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let root = temp.path();
    // Also ignore TERM, so the normal 500ms cancel grace cannot accidentally
    // satisfy the supervisor's shorter 250ms budget in the broken control.
    let command =
        "trap '' HUP; trap '' TERM; echo $$ > group.pid.tmp; mv group.pid.tmp group.pid; sleep 600";
    let delta = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"hangup-owned-tool","type":"function","function":{"name":"bash","arguments":serde_json::json!({"command":command}).to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    std::fs::write(
        root.join("tool.sse"),
        format!("data: {delta}\n\ndata: {finish}\n\ndata: [DONE]\n\n"),
    )
    .unwrap();
    std::fs::write(root.join("mcp.json"),serde_json::json!({"mcpServers":{"fixture":{"command":env!("CARGO_BIN_EXE_mcp-fixture"),"args":["stubborn_group"],"env":{"MCP_GROUP":root.join("mcp-group.pid")}}}}).to_string()).unwrap();
    temp
}
fn supervisor_case(signal: Option<i32>) {
    let temp = fixture();
    let root = temp.path();
    let mut seat = Seat::spawn(root, None, true);
    seat.write(b"start a tool\n");
    wait_for(
        || group_pid(root, "group.pid").is_some() && group_pid(root, "mcp-group.pid").is_some(),
        Duration::from_secs(10),
        "owned groups did not start",
    );
    let groups = [
        group_pid(root, "group.pid").unwrap(),
        group_pid(root, "mcp-group.pid").unwrap(),
    ];
    assert!(groups.iter().all(|p| !group_dead(*p)));
    if let Some(signal) = signal {
        assert_eq!(
            unsafe { libc::kill(seat.child.process_id().unwrap() as i32, signal) },
            0
        );
    } else {
        // No reader clone remains: this really closes the last master descriptor.
        seat.input.take();
        seat.master.take();
    }
    std::thread::sleep(Duration::from_millis(250));
    if let Some(status) = seat.child.try_wait().unwrap() {
        seat.reaped = true;
        eprintln!("seat supervisor observed exit: {status:?}");
    } else {
        unsafe {
            libc::kill(seat.child.process_id().unwrap() as i32, libc::SIGKILL);
        }
        eprintln!("seat supervisor exit pending while groups retire");
    }
    seat.input.take();
    seat.master.take();
    let limit = Instant::now() + Duration::from_secs(2);
    while groups.iter().any(|p| !group_dead(*p)) && Instant::now() < limit {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        groups.iter().all(|p| group_dead(*p)),
        "owned group survived supervisor escalation: {groups:?}"
    );
    if !seat.reaped {
        let status = seat.exit(Duration::from_secs(5));
        eprintln!("seat supervisor observed exit: {status:?}");
    }
    let log = events(root);
    if matches!(log.last().map(|e| &e.event), Some(Event::SessionClosed)) {
        // ICD embed-seat §3 permits finishing the graceful path before the
        // supervisor's escalation. Keep that valid log, and exercise recovery
        // with an actual SIGKILL of a separate, still-pending owned seat.
        check_transcript(&log).unwrap();
        let abrupt = crash_fixture();
        assert_recovery(abrupt.path());
    } else {
        assert_recovery(root);
    }
}
fn crash_fixture() -> tempfile::TempDir {
    let temp = fixture();
    let root = temp.path();
    // Crash the shared core through the headless face. It has the same durable
    // log without introducing a second PTY/session-leader teardown race.
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .args(["--json", "--yolo", "-p", "start a tool", "--cwd"])
        .arg(root)
        .arg("--sessions")
        .arg(root.join("sessions"))
        .arg("--scripted")
        .arg(root.join("tool.sse"))
        .arg("--mcp-config")
        .arg(root.join("mcp.json"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut seat = Seat {
        root: root.into(),
        master: None,
        input: None,
        child: Box::new(child),
        reaped: false,
    };
    wait_for(
        || group_pid(root, "group.pid").is_some() && group_pid(root, "mcp-group.pid").is_some(),
        Duration::from_secs(10),
        "crash fixture groups did not start",
    );
    let groups = [
        group_pid(root, "group.pid").unwrap(),
        group_pid(root, "mcp-group.pid").unwrap(),
    ];
    assert_eq!(
        unsafe { libc::kill(seat.child.process_id().unwrap() as i32, libc::SIGKILL) },
        0
    );
    // SIGKILL cannot run the harness's handler; the fixture owns cleanup.
    for pgid in groups {
        unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        }
    }
    assert!(!seat.exit(Duration::from_secs(5)).success());
    drop(seat);
    wait_for(
        || groups.iter().all(|p| group_dead(*p)),
        Duration::from_secs(2),
        "crash fixture cleanup left an owned group",
    );
    temp
}
fn assert_recovery(root: &Path) {
    assert!(
        !matches!(
            events(root).last().map(|e| &e.event),
            Some(Event::SessionClosed)
        ),
        "fixture must exercise abrupt recovery"
    );
    let session = events(root)[0].session.to_string();
    let mut resumed = Seat::spawn(root, Some(&session), false);
    // A terminal reads its seat: an undrained PTY fills and blocks the seat's stdout.
    let screen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut reader = resumed.master.as_ref().unwrap().try_clone_reader().unwrap();
    let drained = screen.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut b = [0u8; 4096];
        while let Ok(n) = reader.read(&mut b) {
            if n == 0 {
                break;
            }
            drained.lock().unwrap().extend_from_slice(&b[..n]);
        }
    });
    wait_for(
        || {
            events(root)
                .iter()
                .any(|e| matches!(e.event, Event::SessionRecovered { .. }))
        },
        Duration::from_secs(10),
        "seat --resume did not recover",
    );
    resumed.write(b"\x04");
    let status = resumed.exit(Duration::from_secs(2));
    if !status.success() {
        let output = String::from_utf8_lossy(&screen.lock().unwrap()).into_owned();
        panic!("resumed Ctrl-D failed: {status:?}: {output}");
    }
    let log = events(root);
    check_transcript(&log).unwrap();
    assert!(matches!(log.last().unwrap().event, Event::SessionClosed));
}
#[test]
fn graceful_supervisor_exit_retains_a_real_crash_recovery_witness() {
    let abrupt = crash_fixture();
    assert_recovery(abrupt.path());
}
#[test]
fn hup_kills_owned_groups_before_supervisor_escalation_and_resume_recovers() {
    supervisor_case(Some(libc::SIGHUP));
}
#[test]
fn term_kills_owned_groups_before_supervisor_escalation_and_resume_recovers() {
    supervisor_case(Some(libc::SIGTERM));
}
#[test]
fn pty_master_close_kills_owned_groups_before_escalation_and_resume_recovers() {
    supervisor_case(None);
}
#[test]
fn control_d_at_the_prompt_closes_and_fsyncs_gracefully() {
    let temp = fixture();
    let root = temp.path();
    let mut seat = Seat::spawn(root, None, true);
    wait_for(
        || group_pid(root, "mcp-group.pid").is_some(),
        Duration::from_secs(10),
        "MCP group did not start",
    );
    let pgid = group_pid(root, "mcp-group.pid").unwrap();
    seat.wait_prompt();
    seat.write(b"\x04");
    assert!(seat.exit(Duration::from_secs(2)).success());
    wait_for(
        || group_dead(pgid),
        Duration::from_secs(2),
        "MCP group survived graceful Ctrl-D",
    );
    let log = events(root);
    assert!(matches!(log.last().unwrap().event, Event::SessionClosed));
    check_transcript(&log).unwrap();
}

#[test]
fn writer_drop_is_not_cooked_eof() {
    use std::{io::Read, os::unix::fs::OpenOptionsExt};
    let pty = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut slave = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(pty.master.tty_name().unwrap())
        .unwrap();
    drop(terminal_writer(pty.master.as_ref()));
    let mut byte = [0u8; 1];
    let read = slave.read(&mut byte);
    assert!(
        matches!(read,Err(ref error) if error.kind()==std::io::ErrorKind::WouldBlock),
        "writer drop injected terminal input: {read:?}"
    );
}
