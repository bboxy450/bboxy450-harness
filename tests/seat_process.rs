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
    command.args(["seat", "--yolo", "--cwd"]);
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
    let child = pty.slave.spawn_command(command).unwrap();
    drop(pty.slave);
    (temp, pty.master, child)
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
            if String::from_utf8_lossy(&all).contains("approval:") {
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
            if String::from_utf8_lossy(&all).contains("EndTurn") {
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
