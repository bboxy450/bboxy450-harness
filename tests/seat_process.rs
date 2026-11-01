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
    std::thread::sleep(Duration::from_millis(50));
    let start = Instant::now();
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
    drop(writer);
    drop(master);
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(5));
    }
}
