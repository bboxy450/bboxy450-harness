// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Actual ordinary-worker birth/permit/lease witnesses. Included by dream_c1.rs.
//! Private process observations do not certify the consumer's real authority fence.
use super::*;
use std::io::Read;
use std::os::fd::AsRawFd;

const WAITING: &str = "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0";

struct Refused {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: String,
    pid: u32,
}

fn missing_path(job: &Job) -> std::path::PathBuf {
    job._home.path().join(ulid::Ulid::new().to_string())
}

fn waiting(held: &mut HeldWorker) -> u32 {
    assert_eq!(held.marker.trim(), WAITING);
    let child = held
        .child
        .as_mut()
        .expect("actual birth must already be retained");
    assert!(
        child.try_wait().unwrap().is_none(),
        "held child exited before verdict/grant"
    );
    assert!(held.lease.is_some());
    assert!(boot::monotonic_ns().unwrap() < held.readiness_ns);
    child.id()
}

fn grant_once(held: &mut HeldWorker, raw: &[u8]) {
    // One bounded write, never write_all/retry. All corpus lengths fit PIPE_BUF.
    assert!(raw.len() <= 256);
    let written = unsafe {
        libc::write(
            held.lease.as_ref().unwrap().as_raw_fd(),
            raw.as_ptr().cast(),
            raw.len(),
        )
    };
    assert_eq!(written, raw.len() as isize, "single permit transfer failed");
}

fn collected(mut held: HeldWorker) -> Refused {
    // The object retains its exact Child through every assertion and escalation.
    // Keep any writer lease open while the child rejects its bytes/time bound.
    let pid = held.child.as_ref().unwrap().id();
    let stdout_fd = held
        .child
        .as_ref()
        .unwrap()
        .stdout
        .as_ref()
        .unwrap()
        .as_raw_fd();
    let flags = unsafe { libc::fcntl(stdout_fd, libc::F_GETFL) };
    assert!(flags >= 0);
    assert_eq!(
        unsafe { libc::fcntl(stdout_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
        0
    );
    let mut stdout = Vec::new();
    let until = Instant::now() + Duration::from_secs(2);
    let original_readiness = held.readiness_ns;
    let mut readiness_elapsed = false;
    let mut timed_out = false;
    loop {
        // Observe stdout without a blocking BufRead/another reader, throughout
        // the ORIGINAL readiness horizon and subsequent bounded settlement.
        let mut p = libc::pollfd {
            fd: stdout_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let polled = unsafe { libc::poll(&mut p, 1, 0) };
        assert!(polled >= 0);
        if p.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
            let mut bytes = [0u8; 1024];
            let n = unsafe { libc::read(stdout_fd, bytes.as_mut_ptr().cast(), bytes.len()) };
            if n > 0 {
                assert!(
                    stdout.len() + n as usize <= 16 * 1024,
                    "unexpected unbounded stdout"
                );
                stdout.extend_from_slice(&bytes[..n as usize]);
            } else if n < 0 {
                assert_eq!(
                    std::io::Error::last_os_error().kind(),
                    std::io::ErrorKind::WouldBlock
                );
            }
        }
        readiness_elapsed |= boot::monotonic_ns().unwrap() >= original_readiness;
        if held.child.as_mut().unwrap().try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= until {
            timed_out = true;
            // Existing helper: 2s cooperative EOF, exact owned PID SIGTERM,
            // another 2s, exact Child kill only if still alive, actual reap.
            drop(held.lease.take());
            drop(held.child.as_mut().unwrap().stdin.take());
            settle_owned(held.child.as_mut().unwrap());
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let child = held.child.as_mut().unwrap();
    let status = child.wait().unwrap();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stdout.len() <= 16 * 1024 && stderr.len() <= 16 * 1024);
    assert!(
        !timed_out,
        "permit refusal exceeded 2s; pid={pid} original_readiness_elapsed={readiness_elapsed} status={status:?} stdout={stdout:?} stderr={stderr}"
    );
    Refused {
        status,
        stdout,
        stderr,
        pid,
    }
}

fn no_hello(refused: &Refused, context: &str) {
    assert_eq!(
        refused.status.code(),
        Some(2),
        "{context}: pid={} stderr={}",
        refused.pid,
        refused.stderr
    );
    assert!(
        refused.stdout.is_empty(),
        "{context}: ungranted/malformed worker emitted stdout {:?}",
        refused.stdout
    );
    assert!(
        !refused.stderr.contains("state=paid_hello"),
        "{context}: false paid Hello {}",
        refused.stderr
    );
    assert!(
        !refused.stderr.contains("dream-c1 metrics "),
        "{context}: compute/source path ran {}",
        refused.stderr
    );
}

fn no_native_owner(refused: &Refused, context: &str) {
    assert!(
        !refused.stderr.contains("native tag=compute_owner_create"),
        "{context}: prepermit native effect {}",
        refused.stderr
    );
}

fn finish_without_done(worker: &mut Worker, expected_exit: i32) -> String {
    let (status, _) = worker.reap_usage();
    let mut rest = Vec::new();
    worker.output.read_to_end(&mut rest).unwrap();
    let mut metrics = String::new();
    worker
        .child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut metrics)
        .unwrap();
    assert_eq!(
        status.code(),
        Some(expected_exit),
        "{status:?} stdout={rest:?} {metrics}"
    );
    assert!(
        rest.is_empty(),
        "unexpected post-Hello protocol output {rest:?}"
    );
    assert!(metrics.len() <= 16 * 1024);
    assert!(
        metrics.contains("dream-bootstrap state=paid_hello profile=dream-bootstrap/1"),
        "missing actual paid-bootstrap receipt: {metrics}"
    );
    assert!(
        !metrics.contains("dream-c1 metrics "),
        "source/compute ran without an admissible Start: {metrics}"
    );
    let native = metrics
        .lines()
        .find(|l| l.starts_with("dream-c1 native tag=compute_owner_create "))
        .expect("real parked-owner receipt");
    assert_eq!(metric(native, "count"), 1, "{metrics}");
    assert_eq!(metric(native, "effects"), 1, "{metrics}");
    metrics
}

#[test]
#[ignore = "load-sensitive budget/wall window; re-enable via tasks/2026-11-06-qos-builds-never-starve-mcp"]
fn bootstrap_held_birth_has_zero_admission_and_independent_probe_then_real_completion() {
    let job = Job::new(1, 0, false, false);
    // Establish the independent source-free peer BEFORE the held birth. Two
    // complete sequential bootstraps are not assumed to fit one250ms window.
    let absent = missing_path(&job);
    let probe_nonce = w::Ulid(ulid::Ulid::new().to_string());
    let mut probe = Worker::launch(&absent, &probe_nonce, None, true);
    let mut held = HeldWorker::spawn(
        &job.path,
        &job.start.nonce,
        Some(&job.manifest.context),
        false,
    );
    let pid = waiting(&mut held);
    for _ in 0..200 {
        let at = Instant::now();
        assert!(held.child.as_mut().unwrap().try_wait().unwrap().is_none());
        assert!(at.elapsed() <= Duration::from_millis(100));
    }
    // Its real EOF/cancel/exit control still settles while the first parent has
    // not evaluated cost, paid debt or released its permit.
    drop(probe.input.take());
    let probe_metrics = finish_without_done(&mut probe, 0);
    assert!(!absent.exists());
    assert!(held.child.as_mut().unwrap().try_wait().unwrap().is_none());
    assert!(
        boot::monotonic_ns().unwrap() < held.readiness_ns,
        "independent control exceeded original held readiness"
    );
    let mut worker = held.release();
    assert_eq!(
        worker.child.id(),
        pid,
        "release replaced/retried the actual held child"
    );
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {metrics}");
    assert!(job.path.join("output/result.json").is_file());
    eprintln!(
        "bootstrap-held-independent actual_pid={pid} no_retry=true probe_receipt={probe_metrics} completed_receipt={metrics}"
    );
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_missing_environment_refuses_approved_ordinary_worker_before_source_or_owner() {
    let job = Job::new(0, 0, false, false);
    let absent = missing_path(&job);
    let (runner, _) = approved_runner();
    let child = Command::new(runner)
        .args(["dream", "inspect-c1", "--job-dir"])
        .arg(&absent)
        .env_clear()
        .env("LANG", "C.UTF-8")
        .env("TMPDIR", job._home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Retain real ownership before observing any native cost/verdict. This is a
    // malformed-launch negative and never receives a permit or work admission.
    struct Owned(Child);
    impl Drop for Owned {
        fn drop(&mut self) {
            drop(self.0.stdin.take());
            settle_owned(&mut self.0);
        }
    }
    let mut owned = Owned(child);
    let pid = owned.0.id();
    drop(owned.0.stdin.take());
    settle_owned(&mut owned.0);
    let status = owned.0.wait().unwrap();
    let mut stdout = Vec::new();
    owned
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    let mut stderr = String::new();
    owned
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    let refused = Refused {
        status,
        stdout,
        stderr,
        pid,
    };
    no_hello(&refused, "missing bootstrap environment");
    no_native_owner(&refused, "missing bootstrap environment");
    assert!(!absent.exists());
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_malformed_partial_extra_duplicate_and_revoked_permits_refuse_actual_processes() {
    let job = Job::new(0, 0, false, false);
    for case in [
        "expired",
        "birth",
        "binding",
        "run",
        "purpose",
        "reserved",
        "tail",
        "sequence0",
        "sequence2",
        "incarnation0",
        "truncated",
        "partial",
        "extra",
        "duplicate",
        "eof",
        "revoked",
    ] {
        let absent = missing_path(&job);
        let mut held = HeldWorker::spawn(
            &absent,
            &job.start.nonce,
            Some(&job.manifest.context),
            false,
        );
        waiting(&mut held);
        let mut raw = boot::encode(&held.permit).to_vec();
        match case {
            "expired" => raw[112..120].copy_from_slice(
                &boot::monotonic_ns()
                    .unwrap()
                    .saturating_sub(1)
                    .to_le_bytes(),
            ),
            "birth" => raw[16] ^= 1,
            "binding" => raw[64] ^= 1,
            "run" => raw[32] ^= 1,
            "purpose" => raw[8] = boot::Purpose::ReadPrepare as u8,
            "reserved" => raw[10] = 1,
            "tail" => raw[127] = 1,
            "sequence0" => raw[104..112].copy_from_slice(&0u64.to_le_bytes()),
            "sequence2" => raw[104..112].copy_from_slice(&2u64.to_le_bytes()),
            "incarnation0" => raw[96..104].copy_from_slice(&0u64.to_le_bytes()),
            "truncated" => {
                raw.truncate(127);
            }
            "partial" => {
                raw.truncate(64);
            }
            "extra" => raw.push(0),
            "duplicate" => raw.extend_from_within(..),
            "eof" => {}
            "revoked" => {}
            _ => unreachable!(),
        }
        if case != "eof" {
            grant_once(&mut held, &raw);
        }
        if case == "eof" || case == "revoked" {
            drop(held.lease.take());
        }
        let refused = collected(held);
        no_hello(&refused, case);
        if case != "revoked" {
            no_native_owner(&refused, case);
        }
        assert!(!absent.exists(), "{case}: missing source path was created");
        eprintln!(
            "bootstrap-permit-negative case={case} actual_reaped_pid={} stderr={}",
            refused.pid, refused.stderr
        );
    }
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_probe_and_original_nonce_fences_refuse_hostile_start_before_source() {
    let job = Job::new(1, 0, false, false);
    let absent = missing_path(&job);
    let probe_nonce = w::Ulid(ulid::Ulid::new().to_string());
    let mut probe = Worker::launch(&absent, &probe_nonce, None, true);
    let mut hostile = job.start.clone();
    hostile.run_id = w::Ulid(absent.file_name().unwrap().to_str().unwrap().into());
    hostile.nonce = probe_nonce;
    probe.start(&hostile);
    let probe_metrics = finish_without_done(&mut probe, 2);
    assert!(!absent.exists());
    let before = std::fs::read(job.source()).unwrap();
    let mut worker = job.launch();
    let mut wrong = job.start.clone();
    wrong.nonce = w::Ulid(ulid::Ulid::new().to_string());
    worker.start(&wrong);
    let nonce_metrics = finish_without_done(&mut worker, 2);
    assert_eq!(std::fs::read(job.source()).unwrap(), before);
    assert!(!job.path.join("output/result.json").exists());
    eprintln!("bootstrap-hostile-start probe={probe_metrics} original_nonce={nonce_metrics}");
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_parent_unknown_clock_or_stale_binding_closes_actual_birth_without_grant_or_retry() {
    let job = Job::new(0, 0, false, false);
    for cause in ["unknown-native-clock", "stale-aba-binding"] {
        let absent = missing_path(&job);
        let mut held = HeldWorker::spawn(
            &absent,
            &job.start.nonce,
            Some(&job.manifest.context),
            false,
        );
        let pid = waiting(&mut held);
        let original = held.permit.binding;
        if cause == "unknown-native-clock" {
            held.native_cpu = None;
            assert!(
                held.launch_verdict().is_err(),
                "actual cost guard accepted unknown native cost"
            );
        } else {
            // Inject the parent refusal condition only. This does not substitute
            // for the consumer's real public-incarnation A1/B2/A3 publisher gate.
            held.permit.binding[0] ^= 1;
            assert_ne!(held.permit.binding, original);
            assert!(
                held.grant_verdict().is_err(),
                "actual final handoff accepted changed original binding"
            );
        }
        assert_eq!(held.child.as_ref().unwrap().id(), pid);
        assert!(held.child.as_mut().unwrap().try_wait().unwrap().is_none());
        // Deliberately never call release/grant_once: an actual birth exists,
        // but unavailable cost/stale authority cannot authorize any next work.
        drop(held.lease.take());
        let refused = collected(held);
        no_hello(&refused, cause);
        no_native_owner(&refused, cause);
        assert_eq!(refused.pid, pid);
        assert!(!absent.exists());
        eprintln!(
            "bootstrap-parent-refusal cause={cause} grant_count=0 retry_count=0 actual_reaped_pid={pid}"
        );
    }
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_lease_eof_after_qualifying_hello_does_not_revoke_matching_start() {
    let job = Job::new(1, 0, false, false);
    // release intentionally closes the one-shot lease only AFTER verified Hello.
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {metrics}");
    assert!(job.path.join("output/result.json").is_file());
    assert!(metrics.contains("dream-bootstrap state=paid_hello"));
    eprintln!("bootstrap-after-Hello-EOF completed_actual_reap=true {metrics}");
}

#[test]
#[ignore = "real-runner test: runs only via make test-dream-c1 (needs the approved DREAM_C1_RUNNER)"]
fn bootstrap_foreign_closed_writable_and_bad_environment_descriptors_refuse() {
    use std::os::unix::process::CommandExt;
    let job = Job::new(0, 0, false, false);
    let (runner, _) = approved_runner();
    for case in [
        "closed",
        "writer",
        "regular",
        "profile",
        "birth",
        "binding",
        "extra-field",
    ] {
        let absent = missing_path(&job);
        let (read, write) = pipe_files();
        let regular_path = job._home.path().join(format!("foreign-{case}"));
        std::fs::write(&regular_path, b"not an anonymous permit pipe").unwrap();
        let regular = std::fs::File::open(&regular_path).unwrap();
        let inherit = match case {
            "writer" => write.as_raw_fd(),
            "regular" => regular.as_raw_fd(),
            _ => read.as_raw_fd(),
        };
        let mut value = format!("1:{inherit}:01ARZ3NDEKTSV4RRFFQ69G5FAV:{}", "0".repeat(64));
        match case {
            "profile" => value.replace_range(..1, "2"),
            "birth" => value = value.replace("01ARZ3NDEKTSV4RRFFQ69G5FAV", "not-an-id"),
            "binding" => value = value.replace(&"0".repeat(64), &"A".repeat(64)),
            "extra-field" => value.push_str(":extra"),
            _ => {}
        }
        let mut command = Command::new(runner);
        command
            .args(["dream", "inspect-c1", "--job-dir"])
            .arg(&absent)
            .env_clear()
            .env("LANG", "C.UTF-8")
            .env("TMPDIR", job._home.path())
            .env(boot::ENV, value)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if case != "closed" {
            unsafe {
                command.pre_exec(move || {
                    if libc::fcntl(inherit, libc::F_SETFD, 0) == 0 {
                        Ok(())
                    } else {
                        Err(std::io::Error::last_os_error())
                    }
                });
            }
        }
        struct Actual(Child);
        impl Drop for Actual {
            fn drop(&mut self) {
                drop(self.0.stdin.take());
                settle_owned(&mut self.0);
            }
        }
        let mut actual = Actual(command.spawn().unwrap());
        let pid = actual.0.id();
        drop(read);
        drop(write);
        drop(regular);
        drop(actual.0.stdin.take());
        settle_owned(&mut actual.0);
        let status = actual.0.wait().unwrap();
        let mut stdout = Vec::new();
        actual
            .0
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut stdout)
            .unwrap();
        let mut stderr = String::new();
        actual
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        no_hello(
            &Refused {
                status,
                stdout,
                stderr,
                pid,
            },
            case,
        );
        assert!(!absent.exists());
    }
}
