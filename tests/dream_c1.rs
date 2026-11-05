// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Canonical transport corpus, independent hashes and private worker witnesses.
use bboxy450_harness::dream::{bootstrap as boot, c1_wire as w};
use std::path::Path;
fn root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/dream-c1"
    ))
}
fn dto(name: &str, bytes: &[u8]) -> Result<Vec<u8>, w::Code> {
    macro_rules! read {
        ($t:ty) => {
            w::canonical(&w::decode::<$t>(bytes)?).map_err(|_| w::Code::ProtocolError)
        };
    }
    match name {
        "manifest" => read!(w::Manifest),
        "page" => read!(w::Page),
        "result" => read!(w::ResultDescriptor),
        "start" | "activity" | "worker-cancel" => read!(w::WorkerIn),
        "hello" | "progress" | "done" => read!(w::WorkerOut),
        "inspect" | "status" | "cancel" => read!(w::Control),
        "success" | "failure" => read!(w::ControlReply),
        _ => panic!("unknown fixture {name}"),
    }
}
#[test]
fn wire_schema_and_vectors() {
    let schema = w::schema_bytes();
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schema/dream-c1-v1.schema.json"
    ));
    if std::env::var_os("DREAM_C1_GENERATE").is_some() {
        std::fs::write(path, &schema).unwrap();
    }
    assert_eq!(
        w::hash(std::fs::read(path).unwrap()),
        w::hash(&schema),
        "regenerate schema from c1_wire only"
    );
    let lock: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root().join("lock.json")).unwrap()).unwrap();
    assert_eq!(w::hash(&schema).0, lock["schema_sha256"]);
    for (path, hash) in lock["files"].as_object().unwrap() {
        assert_eq!(
            w::hash(std::fs::read(root().join(path)).unwrap()).0,
            hash.as_str().unwrap()
        );
    }
    let vectors: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(root().join("vectors/canonical.json")).unwrap())
            .unwrap();
    for v in vectors {
        let bytes = w::canonical(&v["value"]).unwrap();
        assert_eq!(bytes, v["canonical"].as_str().unwrap().as_bytes());
        assert_eq!(
            w::canonical_limited(&v["value"], bytes.len()).unwrap(),
            bytes
        );
        assert_eq!(
            w::canonical_limited(&v["value"], bytes.len() - 1),
            Err(w::Code::ResourceLimit)
        );
        assert_eq!(w::hash(bytes).0, v["sha256"]);
    }
    for entry in std::fs::read_dir(root().join("valid")).unwrap() {
        let path = entry.unwrap().path();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            dto(path.file_stem().unwrap().to_str().unwrap(), &bytes).unwrap(),
            bytes
        );
    }
    for entry in std::fs::read_dir(root().join("invalid")).unwrap() {
        let path = entry.unwrap().path();
        assert!(
            w::decode::<w::WorkerIn>(&std::fs::read(&path).unwrap()).is_err(),
            "{}",
            path.display()
        );
    }
}
#[test]
fn independent_python_canonical_hashes() {
    let result=std::process::Command::new("python3").args(["-I","-c",r#"
import hashlib,json,pathlib,sys
p=pathlib.Path(sys.argv[1]); vectors=json.loads((p/'vectors/canonical.json').read_bytes())
for v in vectors:
    b=json.dumps(v['value'],ensure_ascii=False,sort_keys=True,separators=(',',':'),allow_nan=False).encode()
    assert b == v['canonical'].encode()
    assert hashlib.sha256(b).hexdigest() == v['sha256']
print('independent canonical vectors:',len(vectors))
"#]).arg(root()).output().unwrap();
    assert!(result.status.success(), "{:?}", result);
}

use bboxy450_harness::protocol::*;
use std::{
    io::{BufRead, BufReader, Write},
    os::{fd::AsRawFd, unix::fs::PermissionsExt},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};
fn private_dir(path: &Path) {
    std::fs::create_dir(path).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
fn private_file(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
struct Job {
    _home: tempfile::TempDir,
    path: std::path::PathBuf,
    start: w::Start,
    manifest: w::Manifest,
    bytes: Vec<u8>,
}
impl Job {
    fn new(turns: usize, padding: usize, blob: bool, crlf: bool) -> Self {
        let home = tempfile::Builder::new()
            .prefix("dream-c1-")
            .tempdir_in("/private/tmp")
            .unwrap();
        let run = w::Ulid(ulid::Ulid::new().to_string());
        let nonce = w::Ulid(ulid::Ulid::new().to_string());
        let path = home.path().join(&run.0);
        private_dir(&path);
        private_dir(&path.join("input"));
        private_dir(&path.join("output"));
        let session = SessionId::new();
        let mut events = vec![];
        let meta = SessionMeta {
            session,
            cwd: "/fixture".into(),
            vcs: Some(Vcs {
                repo: Some("github.com/fixture/dream".into()),
                ..Default::default()
            }),
            dream: None,
            source: None,
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "fixture".into(),
            created_ms: 0,
        };
        let mut push = |event| {
            let seq = Seq(events.len() as u64 + 1);
            events.push(Envelope {
                v: PROTOCOL,
                session,
                epoch: Epoch(1),
                seq: Some(seq),
                at: seq,
                ts_ms: seq.0,
                cause: None,
                native: None,
                raw: None,
                ext: Default::default(),
                event,
            });
        };
        push(Event::SessionStarted { meta });
        let blob_bytes = b"private typed blob";
        let br = BlobRef {
            sha256: w::hash(blob_bytes).0,
            len: blob_bytes.len() as u64,
            mime: "image/png".into(),
        };
        for _ in 0..turns {
            let turn = TurnId::new();
            push(Event::TurnQueued { turn });
            push(Event::TurnStarted {
                turn,
                model: None,
                origin: None,
            });
            if padding > 0 {
                let item = ItemId::new();
                push(Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::AgentMessage,
                    parent: None,
                    head: None,
                });
                push(Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::AgentMessage {
                        markdown: "x".repeat(padding),
                    },
                });
            }
            if blob {
                let item = ItemId::new();
                push(Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::Image,
                    parent: None,
                    head: None,
                });
                push(Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::Image {
                        mime: "image/png".into(),
                        blob: br.clone(),
                    },
                });
            }
            push(Event::TurnEnded {
                turn,
                reason: StopReason::Quiet { ms: 1 },
                usage: None,
            });
        }
        push(Event::SessionClosed);
        let mut bytes = vec![];
        let mut last = vec![];
        for e in &events {
            let mut line = serde_json::to_vec(e).unwrap();
            if crlf {
                line.push(b'\r')
            }
            last = line.clone();
            bytes.extend(line);
            bytes.push(b'\n');
        }
        let mut manifest: w::Manifest =
            serde_json::from_slice(&std::fs::read(root().join("valid/manifest.json")).unwrap())
                .unwrap();
        manifest.run_id = run.clone();
        manifest.nonce = nonce.clone();
        let s = &mut manifest.sources[0];
        s.session = w::Ulid(session.to_string());
        s.len = w::Number(bytes.len() as u64);
        s.sha256 = w::hash(&bytes);
        s.closed_seq = w::Number(events.len() as u64);
        s.closed_line_sha256 = w::hash(last);
        if blob {
            s.blobs = vec![w::Blob {
                sha256: w::Hex(br.sha256.clone()),
                len: w::Number(br.len),
                mime: w::Mime(br.mime),
            }];
            private_dir(&path.join("input").join(&s.session.0));
            private_dir(&path.join("input").join(&s.session.0).join("blobs"));
            private_file(
                &path
                    .join("input")
                    .join(&s.session.0)
                    .join("blobs")
                    .join(&br.sha256),
                blob_bytes,
            );
        }
        private_file(
            &path.join("input").join(format!("{}.jsonl", s.session.0)),
            &bytes,
        );
        let mut job = Self {
            _home: home,
            path,
            start: w::Start {
                protocol: w::Protocol::V1,
                run_id: run,
                nonce,
                manifest_sha256: w::hash([]),
                timeout_ms: 30000,
                activity_revision: w::Number(1),
                foreground: false,
            },
            manifest,
            bytes,
        };
        job.commit_manifest();
        job
    }
    fn source(&self) -> std::path::PathBuf {
        self.path
            .join("input")
            .join(format!("{}.jsonl", self.manifest.sources[0].session.0))
    }
    fn commit_manifest(&mut self) {
        let bytes = w::canonical(&self.manifest).unwrap();
        self.start.manifest_sha256 = w::hash(&bytes);
        private_file(&self.path.join("input/manifest.json"), &bytes);
    }
    fn launch(&self) -> Worker {
        Worker::launch(
            &self.path,
            &self.start.nonce,
            Some(&self.manifest.context),
            false,
        )
    }
}
fn parent_cpu() -> u64 {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, t.as_mut_ptr()) } != 0 {
        return u64::MAX;
    }
    let t = unsafe { t.assume_init() };
    (t.tv_sec as u64)
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(t.tv_nsec as u64))
        .unwrap_or(u64::MAX)
}
fn parent_rest(cpu: u64, end: Instant) {
    let owed = Duration::from_nanos(9 * cpu.max(1_000_000));
    if let Some(left) = owed.checked_sub(end.elapsed()) {
        std::thread::sleep(left);
    }
}
fn parent_interval(start: u64, end: u64) -> u64 {
    assert_ne!(start, u64::MAX, "unavailable parent entry clock");
    assert_ne!(end, u64::MAX, "unavailable parent ending clock");
    end.checked_sub(start).expect("reversed parent clock")
}
#[test]
fn unavailable_or_reversed_parent_samples_never_become_zero_cost() {
    for (start, end) in [(u64::MAX, u64::MAX), (100, u64::MAX), (100, 99)] {
        assert!(std::panic::catch_unwind(|| parent_interval(start, end)).is_err());
    }
    assert_eq!(parent_interval(100, 101), 1);
}
fn approved_runner() -> &'static (std::path::PathBuf, w::Hex) {
    use sha2::{Digest, Sha256};
    use std::os::unix::fs::MetadataExt;
    type Stamp = (u64, u64, u64, i64, i64, i64, i64);
    static STAMP: std::sync::OnceLock<Stamp> = std::sync::OnceLock::new();
    let stamp = |p: &Path| {
        let m = std::fs::symlink_metadata(p).unwrap();
        assert!(m.is_file() && !m.file_type().is_symlink());
        (
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    static APPROVED: std::sync::OnceLock<(std::path::PathBuf, w::Hex)> = std::sync::OnceLock::new();
    let approved = APPROVED.get_or_init(|| {
        let path = std::path::PathBuf::from(
            std::env::var_os("DREAM_C1_RUNNER")
                .expect("gate requires explicit preapproved ordinary worker"),
        );
        let folder = path.parent().unwrap();
        assert_eq!(
            std::fs::read(folder.join("bootstrap-support.txt")).unwrap(),
            b"dream-bootstrap/1\n"
        );
        let receipts = std::fs::read_to_string(folder.join("hashes.txt")).unwrap();
        let expected = receipts
            .lines()
            .find(|l| {
                l.split_whitespace()
                    .last()
                    .is_some_and(|p| p.ends_with("/worker"))
            })
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        // Provisioning verification precedes every birth; streaming bounded
        // hashing/rest happens once, never an executable capability probe.
        let before = stamp(&path);
        let mut file = std::fs::File::open(&path).unwrap();
        let mut digest = Sha256::new();
        let mut buf = [0; 16384];
        use std::io::Read;
        loop {
            let at = parent_cpu();
            let n = file.read(&mut buf).unwrap();
            digest.update(&buf[..n]);
            let used = parent_interval(at, parent_cpu());
            let end = Instant::now();
            assert!(used <= 1_000_000);
            parent_rest(used, end);
            if n == 0 {
                break;
            }
        }
        let hash = format!("{:x}", digest.finalize());
        assert_eq!(hash, expected, "unapproved candidate must not execute");
        assert_eq!(
            stamp(&path),
            before,
            "artifact changed during pre-birth approval"
        );
        STAMP.set(before).unwrap();
        (path, w::Hex(hash))
    });
    assert_eq!(
        stamp(&approved.0),
        *STAMP.get().unwrap(),
        "approved artifact identity changed before birth"
    );
    approved
}
fn pipe_files() -> (std::fs::File, std::fs::File) {
    use std::os::fd::FromRawFd;
    let mut p = [-1; 2];
    assert_eq!(unsafe { libc::pipe(p.as_mut_ptr()) }, 0);
    for fd in p {
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) },
            0
        );
        let f = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFL, f | libc::O_NONBLOCK) },
            0
        );
    }
    unsafe {
        (
            std::fs::File::from_raw_fd(p[0]),
            std::fs::File::from_raw_fd(p[1]),
        )
    }
}
struct HeldWorker {
    child: Option<Child>,
    lease: Option<std::fs::File>,
    permit: boot::Permit,
    birth_end: Instant,
    launched: Instant,
    readiness_ns: u64,
    native_cpu: Option<u64>,
    native_wall: Duration,
    marker: String,
    expected_schema: w::Hex,
    prepared: Option<Command>,
    frozen_binding: [u8; 32],
    job_deadline: Instant,
}
impl HeldWorker {
    fn spawn(path: &Path, nonce: &w::Ulid, context: Option<&w::Context>, probe: bool) -> Self {
        let (runner, worker_hash) = approved_runner();
        Self::spawn_on(path, nonce, context, probe, runner, worker_hash, false)
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_on(
        path: &Path,
        nonce: &w::Ulid,
        context: Option<&w::Context>,
        probe: bool,
        runner: &Path,
        worker_hash: &w::Hex,
        bypass: bool,
    ) -> Self {
        use std::os::unix::process::CommandExt;
        let job_deadline = Instant::now() + Duration::from_secs(30);
        let prepare_at = parent_cpu();
        let expected_schema = w::schema_hash();
        let (read, write) = pipe_files();
        let fd = read.as_raw_fd();
        let birth = ulid::Ulid::new();
        let run = w::Ulid(path.file_name().unwrap().to_str().unwrap().into());
        // Validate fixed identities during preparation, before a Child exists.
        let run_bytes = boot::ulid(run.0.as_bytes()).unwrap();
        let nonce_bytes = boot::ulid(nonce.0.as_bytes()).unwrap();
        let op = w::Ulid(ulid::Ulid::new().to_string());
        static INCARNATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let incarnation = INCARNATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let binding = boot::Binding::new(
            boot::Purpose::InspectC1,
            probe,
            &op,
            &run,
            nonce,
            incarnation,
            context,
            worker_hash,
        )
        .unwrap();
        let hash = w::hash(binding.bytes().unwrap());
        let raw_binding: [u8; 32] = (0..32)
            .map(|i| u8::from_str_radix(&hash.0[i * 2..i * 2 + 2], 16).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut command = Command::new(runner);
        command
            .args(["dream", "inspect-c1", "--job-dir"])
            .arg(path)
            .env_clear()
            .env("LANG", "C.UTF-8")
            .env("TMPDIR", path)
            .env(boot::ENV, format!("1:{fd}:{birth}:{}", hash.0))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if bypass {
            command.env("BBOXY_DREAM_BOOTSTRAP_TEST_BYPASS", "1");
        }
        unsafe {
            command.pre_exec(move || {
                if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let prepare_used = parent_interval(prepare_at, parent_cpu());
        let prepare_end = Instant::now();
        assert!(prepare_used <= 1_000_000);
        parent_rest(prepare_used, prepare_end);
        let launched = Instant::now();
        let native_at = parent_cpu();
        assert_ne!(
            native_at,
            u64::MAX,
            "missing entry clock must not create child"
        );
        let wall_at = Instant::now();
        // Retain actual Child before evaluating the ending native clock/verdict.
        let child = command.spawn().unwrap();
        let native_wall = wall_at.elapsed();
        let spawn_return_ns = boot::monotonic_ns();
        let end_cpu = parent_cpu();
        let birth_end = Instant::now();
        let readiness_ns = 0;
        let inherited_read_number = read.as_raw_fd();
        let mut result = Self {
            child: Some(child),
            lease: Some(write),
            permit: boot::Permit {
                purpose: boot::Purpose::InspectC1,
                probe,
                birth: birth.to_bytes(),
                run_id: run_bytes,
                nonce: nonce_bytes,
                binding: raw_binding,
                incarnation,
                sequence: 1,
                deadline_ns: readiness_ns,
            },
            birth_end,
            launched,
            readiness_ns,
            native_cpu: if end_cpu == u64::MAX {
                None
            } else {
                end_cpu.checked_sub(native_at)
            },
            native_wall,
            marker: String::new(),
            expected_schema,
            prepared: Some(command),
            frozen_binding: raw_binding,
            job_deadline,
        };
        assert_ne!(
            inherited_read_number,
            result
                .child
                .as_ref()
                .unwrap()
                .stderr
                .as_ref()
                .unwrap()
                .as_raw_fd()
        );
        if bypass {
            eprintln!(
                "bootstrap-fd-debug pid={} gate_read_fd={inherited_read_number} gate_write_fd={} stderr_read_fd={}",
                result.child.as_ref().unwrap().id(),
                result.lease.as_ref().unwrap().as_raw_fd(),
                result
                    .child
                    .as_ref()
                    .unwrap()
                    .stderr
                    .as_ref()
                    .unwrap()
                    .as_raw_fd()
            );
        }
        drop(read);
        // The actual child/write lease is now retained even if readiness clock
        // capture fails. No timing verdict can lose the returned process.
        result.readiness_ns = spawn_return_ns
            .unwrap()
            .checked_add(boot::WINDOW_NS)
            .unwrap();
        result.permit.deadline_ns = result.readiness_ns;
        if !bypass {
            result.marker = result.read_waiting();
        }
        result
    }
    fn read_waiting(&mut self) -> String {
        let stderr = self
            .child
            .as_ref()
            .unwrap()
            .stderr
            .as_ref()
            .unwrap()
            .as_raw_fd();
        let mut bytes = [0u8; 256];
        let mut n = 0;
        while n < bytes.len() {
            let left = self
                .readiness_ns
                .checked_sub(boot::monotonic_ns().unwrap())
                .expect("original readiness expired");
            let mut fd = libc::pollfd {
                fd: stderr,
                events: libc::POLLIN,
                revents: 0,
            };
            let polled = unsafe { libc::poll(&mut fd, 1, left.div_ceil(1_000_000) as i32) };
            assert!(
                polled > 0,
                "missing source-bound wait marker pid={} state={:?} native_cpu={:?} native_wall_ns={} lifetime_ms={} stderr_fd={stderr} partial={:?}",
                self.child.as_ref().unwrap().id(),
                self.child.as_mut().unwrap().try_wait(),
                self.native_cpu,
                self.native_wall.as_nanos(),
                self.launched.elapsed().as_millis(),
                &bytes[..n]
            );
            let count =
                unsafe { libc::read(stderr, bytes[n..].as_mut_ptr().cast(), bytes.len() - n) };
            assert!(count > 0, "bootstrap refused before wait marker");
            n += count as usize;
            if bytes[..n].ends_with(b"\n") {
                break;
            }
        }
        String::from_utf8(bytes[..n].to_vec()).unwrap()
    }
    fn waiting_verdict(&self) -> Result<(), &'static str> {
        if self.marker.trim()
            == "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0"
        {
            Ok(())
        } else {
            Err("controlled bootstrap span preceded permit")
        }
    }
    fn launch_verdict(&self) -> Result<u64, &'static str> {
        let cpu = self.native_cpu.ok_or("unknown native birth cost")?;
        if cpu > 2_000_000 || self.native_wall > Duration::from_millis(25) {
            return Err("native birth allocation");
        }
        Ok(cpu)
    }
    fn grant_verdict(&self) -> Result<(), &'static str> {
        if self.permit.binding != self.frozen_binding {
            return Err("original launch binding changed");
        }
        if boot::monotonic_ns().map_err(|_| "unknown readiness")? >= self.readiness_ns {
            return Err("original readiness expired");
        }
        Ok(())
    }
    fn release(mut self) -> Worker {
        self.waiting_verdict()
            .expect("earliest controlled admission violation");
        assert!(self.child.as_mut().unwrap().try_wait().unwrap().is_none());
        let cpu = self
            .launch_verdict()
            .expect("native birth verdict refused grant");
        parent_rest(cpu, self.birth_end);
        let setup_at = parent_cpu();
        drop(self.prepared.take());
        let mut child = self.child.take().unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut worker = Worker {
            child,
            input,
            output,
            start: None,
            sequence: 0,
            reaped: false,
            launched: self.launched,
            bootstrap_lease: self.lease.take(),
            job_deadline: self.job_deadline,
        };
        let setup_cpu = parent_interval(setup_at, parent_cpu());
        let end = Instant::now();
        assert!(setup_cpu <= 1_000_000);
        parent_rest(setup_cpu, end);
        // Exactly one after-debt control guard/encode/transfer/write. Worker
        // already owns the lease+actual Child, including any partial-write or
        // later ending-clock refusal; Drop closes it before real settlement.
        let grant_at = parent_cpu();
        self.grant_verdict().expect("handoff refused");
        let bytes = boot::encode(&self.permit);
        let wrote = unsafe {
            libc::write(
                worker.bootstrap_lease.as_ref().unwrap().as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len(),
            )
        };
        assert_eq!(wrote, 128);
        assert!(parent_interval(grant_at, parent_cpu()) <= 1_000_000);
        let remaining = self
            .readiness_ns
            .checked_sub(boot::monotonic_ns().unwrap())
            .expect("readiness not reset by grant");
        let mut p = libc::pollfd {
            fd: worker.output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut p, 1, remaining.div_ceil(1_000_000) as i32) } > 0,
            "Hello readiness exceeded"
        );
        match worker.read() {
            w::WorkerOut::Hello(h) => {
                assert_eq!(h.schema_sha256, self.expected_schema);
                assert_eq!(h.ir_major, 1);
                assert_eq!(h.ir_minor_max, 3);
            }
            other => panic!("{other:?}"),
        }
        assert!(
            boot::monotonic_ns().unwrap() < self.readiness_ns,
            "qualifying Hello exceeded original readiness"
        );
        drop(worker.bootstrap_lease.take());
        worker
    }
}
fn settle_owned(child: &mut Child) {
    let until = Instant::now() + Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(2));
    }
    if child.try_wait().unwrap().is_some() {
        return;
    }
    assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
    let until = Instant::now() + Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(2));
    }
    if child.try_wait().unwrap().is_none() {
        child.kill().unwrap();
        child.wait().unwrap();
    }
}
impl Drop for HeldWorker {
    fn drop(&mut self) {
        drop(self.lease.take());
        if let Some(child) = self.child.as_mut() {
            drop(child.stdin.take());
            settle_owned(child);
        }
    }
}
struct Worker {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    start: Option<w::Start>,
    sequence: u64,
    reaped: bool,
    launched: Instant,
    bootstrap_lease: Option<std::fs::File>,
    job_deadline: Instant,
}
impl Worker {
    fn launch(path: &Path, nonce: &w::Ulid, context: Option<&w::Context>, probe: bool) -> Self {
        HeldWorker::spawn(path, nonce, context, probe).release()
    }
    fn read(&mut self) -> w::WorkerOut {
        let mut b = Vec::new();
        self.output.read_until(b'\n', &mut b).unwrap();
        assert!(
            !b.is_empty(),
            "worker unexpectedly exited {:?}",
            self.child.try_wait()
        );
        let out = w::decode(&b).unwrap();
        let seq = match &out {
            w::WorkerOut::Progress(p) => Some(p.seq.0),
            w::WorkerOut::Done(d) => Some(d.seq.0),
            _ => None,
        };
        if let Some(seq) = seq {
            assert_eq!(seq, self.sequence + 1);
            self.sequence = seq;
        }
        out
    }
    fn send(&mut self, message: &w::WorkerIn) {
        let mut b = w::canonical(message).unwrap();
        b.push(b'\n');
        self.input.as_mut().unwrap().write_all(&b).unwrap();
    }
    fn start(&mut self, s: &w::Start) {
        let at = parent_cpu();
        let mut start = s.clone();
        if (1..=30000).contains(&start.timeout_ms) {
            let remaining = self
                .job_deadline
                .checked_duration_since(Instant::now())
                .expect("original job deadline expired before Start")
                .as_millis();
            assert!(remaining > 0);
            start.timeout_ms = start.timeout_ms.min(remaining as u32);
        }
        self.send(&w::WorkerIn::Start(start.clone()));
        self.start = Some(start);
        assert!(
            parent_interval(at, parent_cpu()) <= 1_000_000,
            "whole final Start control exceeds1ms"
        );
    }
    fn cancel(&mut self) {
        let s = self.start.clone().unwrap();
        self.send(&w::WorkerIn::Cancel(w::WorkerCancel {
            protocol: w::Protocol::V1,
            run_id: s.run_id,
            nonce: s.nonce,
        }));
    }
    fn activity(&mut self, revision: u64, foreground: bool) {
        let s = self.start.clone().unwrap();
        self.send(&w::WorkerIn::Activity(w::Activity {
            protocol: w::Protocol::V1,
            run_id: s.run_id,
            nonce: s.nonce,
            revision: w::Number(revision),
            foreground,
        }));
    }
    fn timed_paused_activity(&mut self, revision: u64, timeout: Duration) -> Option<Duration> {
        // Call only after initial compute Paused is consumed, with exactly one
        // request outstanding. An ignored stale revision must time out, proving
        // the next old queued frame cannot stand in for this request.
        assert!(self.output.buffer().is_empty());
        let at = Instant::now();
        self.activity(revision, true);
        self.wait_paused_response(at, timeout)
    }
    fn wait_paused_response(&mut self, at: Instant, timeout: Duration) -> Option<Duration> {
        loop {
            let left = timeout.checked_sub(at.elapsed())?;
            let mut fd = libc::pollfd {
                fd: self.output.get_ref().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe {
                libc::poll(
                    &mut fd,
                    1,
                    left.as_millis().clamp(1, i32::MAX as u128) as i32,
                )
            };
            if ready < 0 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::EINTR)
                );
                continue;
            }
            if ready == 0 {
                return None;
            }
            match self.read() {
                w::WorkerOut::Progress(p) if p.phase == w::Phase::Paused => {
                    assert_eq!(p.bytes.0, 0);
                    return Some(at.elapsed());
                }
                other => panic!("unexpected paused Activity response: {other:?}"),
            }
        }
    }
    fn reap_usage(&mut self) -> (std::process::ExitStatus, libc::rusage) {
        use std::os::unix::process::ExitStatusExt;
        let until = Instant::now() + Duration::from_secs(3);
        loop {
            let mut status = 0;
            let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
            let pid = unsafe {
                libc::wait4(
                    self.child.id() as libc::pid_t,
                    &mut status,
                    libc::WNOHANG,
                    usage.as_mut_ptr(),
                )
            };
            if pid > 0 {
                assert_eq!(pid as u32, self.child.id());
                self.reaped = true;
                return (std::process::ExitStatus::from_raw(status), unsafe {
                    usage.assume_init()
                });
            }
            if pid < 0 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::EINTR)
                );
            }
            assert!(Instant::now() < until, "worker did not settle");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn done(&mut self) -> (w::Done, String) {
        let mut seq = self.sequence;
        let done = loop {
            match self.read() {
                w::WorkerOut::Progress(p) => {
                    assert_eq!(p.seq.0, seq + 1);
                    seq = p.seq.0
                }
                w::WorkerOut::Done(d) => {
                    assert_eq!(d.seq.0, seq + 1);
                    break d;
                }
                _ => panic!("extra hello"),
            }
        };
        let (status, usage) = self.reap_usage();
        let expected = if done.outcome == w::Outcome::Failed {
            1
        } else {
            0
        };
        assert_eq!(status.code(), Some(expected), "{done:?}");
        let mut extra = String::new();
        self.output.read_line(&mut extra).unwrap();
        assert!(extra.is_empty(), "extra terminal frame");
        let mut metrics = String::new();
        use std::io::Read;
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut metrics)
            .unwrap();
        assert!(metrics.len() <= 16 * 1024);
        assert!(metrics.contains("dream-bootstrap state=paid_hello profile=dream-bootstrap/1"));
        let ns = |t: libc::timeval| t.tv_sec as u64 * 1_000_000_000 + t.tv_usec as u64 * 1000;
        let user = ns(usage.ru_utime);
        let system = ns(usage.ru_stime);
        if let Some(terminal) = metrics
            .lines()
            .find(|l| l.starts_with("dream-c1 terminal_metrics "))
        {
            assert!(user >= metric(terminal, "process_user_cpu_ns"));
            assert!(system >= metric(terminal, "process_system_cpu_ns"));
        }
        // Separate post-reap complete-process usage, never added to overlapping
        // child caller/owner observations. Includes final telemetry and exit.
        metrics.push_str(&format!("dream-c1 parent_reap reap_user_cpu_ns={user} reap_system_cpu_ns={system} parent_elapsed_wall_ns={} cutoff=actual_wait4_exit\n", self.launched.elapsed().as_nanos()));
        (done, metrics)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if !self.reaped {
            drop(self.bootstrap_lease.take());
            drop(self.input.take());
            settle_owned(&mut self.child);
        }
    }
}
fn complete(job: &Job) -> (w::ResultDescriptor, Vec<w::Row>, String) {
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {metrics}");
    let bytes = std::fs::read(job.path.join("output/result.json")).unwrap();
    assert_eq!(Some(w::hash(&bytes)), done.result_sha256);
    let result: w::ResultDescriptor = w::decode(&bytes).unwrap();
    assert_eq!(result.manifest_sha256, job.start.manifest_sha256);
    assert_eq!(result.knowledge, w::Knowledge::NotRequested);
    assert_eq!(result.trust, w::Trust::NotEvaluated);
    assert!(result.max_poll_cpu_ns.0 <= 3_000_000);
    assert_native_gate(&metrics);
    assert_eq!(
        result.max_poll_cpu_ns.0,
        metric(&metrics, "captured_max_cpu_ns")
    );
    let mut rows = vec![];
    for (i, p) in result.pages.iter().enumerate() {
        let bytes = std::fs::read(job.path.join(format!("output/page-{i:06}.json"))).unwrap();
        assert_eq!(p.sha256, w::hash(&bytes));
        assert_eq!(p.len.0, bytes.len() as u64);
        assert!(bytes.len() <= 16 * 1024);
        let page: w::Page = w::decode(&bytes).unwrap();
        assert!(page.rows.len() <= 128);
        assert_eq!(
            page.next,
            if i + 1 == result.pages.len() {
                None
            } else {
                Some(i as u32 + 1)
            }
        );
        rows.extend(page.rows);
    }
    assert_eq!(std::fs::read(job.source()).unwrap(), job.bytes);
    (result, rows, metrics)
}
#[test]
fn hello_precedes_all_source_access_and_private_bootstrap() {
    let root = tempfile::Builder::new().tempdir_in("/private/tmp").unwrap();
    let path = root.path().join(ulid::Ulid::new().to_string());
    let nonce = w::Ulid(ulid::Ulid::new().to_string());
    let mut worker = Worker::launch(&path, &nonce, None, true);
    drop(worker.input.take());
    assert!(worker.child.wait().unwrap().success());
    let mut metrics = String::new();
    std::io::Read::read_to_string(&mut worker.child.stderr.take().unwrap(), &mut metrics).unwrap();
    assert_native_gate(&metrics);
    let native = metrics
        .lines()
        .find(|l| l.starts_with("dream-c1 native tag=compute_owner_create "))
        .unwrap();
    assert_eq!(metric(native, "count"), 1);
    assert_eq!(metric(native, "effects"), 1);
    assert!(!path.exists());
    eprintln!("cleared-environment Hello-before-source EOF actual_parked_owner_joined {metrics}");
}
#[test]
fn empty_closed_and_raw_crlf_origins_are_exact() {
    let empty = Job::new(0, 0, false, false);
    let (r, rows, _) = complete(&empty);
    assert_eq!((r.episode_count, r.cluster_count, r.pages.len()), (0, 0, 1));
    assert!(rows.is_empty());
    let job = Job::new(2, 0, true, true);
    let (result, rows, metrics) = complete(&job);
    assert_eq!((result.episode_count, result.cluster_count), (2, 1));
    let episodes = rows
        .iter()
        .filter_map(|r| {
            if let w::Row::Episode(e) = r {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for e in episodes {
        let line = job
            .bytes
            .split_inclusive(|b| *b == b'\n')
            .nth(e.origin.seq.0 as usize - 1)
            .unwrap();
        assert_eq!(
            e.origin.digest,
            format!("sha256:{}", w::hash(&line[..line.len() - 1]).0)
        );
        assert_eq!(e.fingerprints, vec![w::hash("quiet-stop")]);
    }
    eprintln!("{metrics}");
}
#[test]
fn source_blob_sequence_version_and_custody_controls() {
    for mode in 0..10 {
        let mut job = Job::new(1, 0, true, false);
        let expected = match mode {
            0 => {
                job.manifest.sources[0].closed_line_sha256 = w::hash("wrong");
                w::Code::SourceMismatch
            }
            1 => {
                job.manifest.sources[0].blobs.clear();
                w::Code::BlobMismatch
            }
            2 => {
                let blob = &job.manifest.sources[0].blobs[0];
                private_file(
                    &job.path
                        .join("input")
                        .join(&job.manifest.sources[0].session.0)
                        .join("blobs")
                        .join(&blob.sha256.0),
                    b"wrong",
                );
                w::Code::BlobMismatch
            }
            3 => {
                std::fs::remove_file(job.source()).unwrap();
                std::os::unix::fs::symlink("/dev/null", job.source()).unwrap();
                w::Code::StorageUnavailable
            }
            4 => {
                let linked = job.path.join("linked");
                std::fs::hard_link(job.source(), linked).unwrap();
                w::Code::StorageUnavailable
            }
            5 => {
                let mut text = String::from_utf8(job.bytes.clone()).unwrap();
                text = text.replace("\"major\":1", "\"major\":2");
                job.bytes = text.into_bytes();
                private_file(&job.source(), &job.bytes);
                job.manifest.sources[0].len = w::Number(job.bytes.len() as u64);
                job.manifest.sources[0].sha256 = w::hash(&job.bytes);
                w::Code::UnsupportedIr
            }
            6 => {
                let keep = job.bytes.iter().rposition(|b| *b == b'\n').unwrap();
                job.bytes.truncate(keep);
                private_file(&job.source(), &job.bytes);
                job.manifest.sources[0].len = w::Number(job.bytes.len() as u64);
                job.manifest.sources[0].sha256 = w::hash(&job.bytes);
                w::Code::MalformedIr
            }
            7 => {
                job.manifest.sources[0].repo = w::Repo("github.com/foreign/repo".into());
                w::Code::SourceMismatch
            }
            8 => {
                job.manifest.sources[0].len = w::Number(64 * 1024 * 1024 + 1);
                w::Code::ResourceLimit
            }
            _ => {
                let cut = job.bytes[..job.bytes.len() - 1]
                    .iter()
                    .rposition(|b| *b == b'\n')
                    .unwrap()
                    + 1;
                job.bytes.truncate(cut);
                private_file(&job.source(), &job.bytes);
                job.manifest.sources[0].len = w::Number(job.bytes.len() as u64);
                job.manifest.sources[0].sha256 = w::hash(&job.bytes);
                w::Code::SourceOpen
            }
        };
        job.commit_manifest();
        let mut worker = job.launch();
        worker.start(&job.start);
        let (done, metrics) = worker.done();
        assert_eq!(done.outcome, w::Outcome::Failed, "mode {mode} {metrics}");
        assert_eq!(done.error.unwrap().code, expected, "mode {mode} {metrics}");
        assert!(!job.path.join("output/result.json").exists());
    }
}
#[test]
fn foreground_pause_stale_activity_cancel_eof_and_deadline_settle() {
    for mode in 0..4 {
        let mut job = Job::new(1, 0, false, false);
        job.start.foreground = true;
        if mode == 3 {
            job.start.timeout_ms = 80;
        }
        let mut worker = job.launch();
        worker.start(&job.start);
        let begin = Instant::now();
        loop {
            if let w::WorkerOut::Progress(p) = worker.read() {
                if p.phase == w::Phase::Paused {
                    break;
                }
            }
        }
        assert!(begin.elapsed() < Duration::from_millis(100));
        assert!(
            std::fs::read_dir(job.path.join("output"))
                .unwrap()
                .next()
                .is_none()
        );
        // Old quiescent revision must never resume a foreground run.
        worker.activity(1, false);
        std::thread::sleep(Duration::from_millis(15));
        assert!(
            std::fs::read_dir(job.path.join("output"))
                .unwrap()
                .next()
                .is_none()
        );
        match mode {
            0 => worker.activity(2, false),
            1 => worker.cancel(),
            2 => drop(worker.input.take()),
            _ => {}
        }
        // paused progress already consumed; done helper expects next seq=1, so read terminal here.
        let mut done = None;
        while done.is_none() {
            if let w::WorkerOut::Done(d) = worker.read() {
                done = Some(d);
            }
        }
        let done = done.unwrap();
        let status = worker.child.wait().unwrap();
        let mut metrics = String::new();
        std::io::Read::read_to_string(&mut worker.child.stderr.take().unwrap(), &mut metrics)
            .unwrap();
        assert_eq!(
            done.outcome,
            if mode == 0 {
                w::Outcome::Completed
            } else if mode == 3 {
                w::Outcome::Failed
            } else {
                w::Outcome::Cancelled
            },
            "mode={mode} {done:?} {metrics}"
        );
        assert_eq!(status.code(), Some(if mode == 3 { 1 } else { 0 }));
    }
}
#[test]
fn immutable_results_are_never_clobbered() {
    let job = Job::new(1, 0, false, false);
    complete(&job);
    let before = std::fs::read(job.path.join("output/result.json")).unwrap();
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, _) = worker.done();
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert_eq!(done.error.unwrap().code, w::Code::StorageUnavailable);
    assert_eq!(
        before,
        std::fs::read(job.path.join("output/result.json")).unwrap()
    );
}
#[test]
fn large_input_records_heap_rss_and_resource_rejection() {
    let job = Job::new(40, 60 * 1024, false, false);
    assert!(job.bytes.len() > 2 * 1024 * 1024);
    let (result, rows, metrics) = complete(&job);
    assert_eq!(result.episode_count, 40);
    assert_eq!(rows.len(), 41);
    assert!(metrics.contains("heap_peak_bytes="));
    assert!(metrics.contains("rss_peak_bytes="));
    assert!(metrics.contains("overruns=0"));
    assert_memory_gate(&metrics);
    eprintln!("large-input {metrics}");
    let mut over = Job::new(1, 0, false, false);
    over.manifest.sources[0].len = w::Number(64 * 1024 * 1024 + 1);
    over.commit_manifest();
    let mut worker = over.launch();
    worker.start(&over.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.error.unwrap().code, w::Code::ResourceLimit);
    eprintln!("bound-exceeding {metrics}");
}

#[test]
fn all_dtos_reject_unknown_duplicate_and_required_missing_fields() {
    for entry in std::fs::read_dir(root().join("valid")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v.as_object_mut()
            .unwrap()
            .insert("unexpected".into(), true.into());
        assert!(
            dto(name, &w::canonical(&v).unwrap()).is_err(),
            "unknown {name}"
        );
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let (key, value) = v.as_object().unwrap().iter().next().unwrap();
        let mut duplicate = bytes[..bytes.len() - 1].to_vec();
        duplicate.extend(
            format!(
                ",{}:{}}}",
                serde_json::to_string(key).unwrap(),
                serde_json::to_string(value).unwrap()
            )
            .bytes(),
        );
        assert!(dto(name, &duplicate).is_err(), "duplicate {name}.{key}");
        let required = match name {
            "manifest" | "page" | "result" => "schema",
            "start" | "activity" | "worker-cancel" | "hello" | "progress" | "done" => "protocol",
            _ => "v",
        };
        let mut v = v;
        v.as_object_mut().unwrap().remove(required);
        assert!(
            dto(name, &w::canonical(&v).unwrap()).is_err(),
            "missing {name}.{required}"
        );
    }
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root().join("valid/manifest.json")).unwrap())
            .unwrap();
    manifest["context"]["account"] = "é".repeat(128).into();
    assert!(w::decode::<w::Manifest>(&w::canonical(&manifest).unwrap()).is_ok());
    manifest["context"]["account"] = "é".repeat(129).into();
    assert!(w::decode::<w::Manifest>(&w::canonical(&manifest).unwrap()).is_err());
}
#[test]
fn multiple_pages_preserve_exact_counts_and_order() {
    let job = Job::new(80, 0, false, false);
    let (result, rows, _) = complete(&job);
    assert!(result.pages.len() > 1);
    assert_eq!(result.episode_count, 80);
    assert_eq!(result.cluster_count, 1);
    assert_eq!(rows.len(), 81);
    let mut last = 0;
    for row in rows {
        match row {
            w::Row::Episode(e) => {
                assert!(e.origin.seq.0 > last);
                last = e.origin.seq.0
            }
            w::Row::Cluster(c) => assert_eq!(c.count, 80),
        }
    }
}
#[test]
fn partial_frame_eof_wrong_nonce_and_oversize_settle_as_protocol_failure() {
    for mode in 0..3 {
        let mut job = Job::new(1, 0, false, false);
        job.start.foreground = true;
        let mut worker = job.launch();
        worker.start(&job.start);
        match mode {
            0 => {
                worker
                    .input
                    .as_mut()
                    .unwrap()
                    .write_all(b"{\"protocol\":")
                    .unwrap();
                drop(worker.input.take());
            }
            1 => {
                let mut s = job.start.clone();
                s.nonce = w::Ulid(ulid::Ulid::new().to_string());
                worker.send(&w::WorkerIn::Cancel(w::WorkerCancel {
                    protocol: w::Protocol::V1,
                    run_id: s.run_id,
                    nonce: s.nonce,
                }));
            }
            _ => {
                let b = vec![b'x'; w::MAX_FRAME + 1];
                worker.input.as_mut().unwrap().write_all(&b).unwrap();
            }
        }
        let done = loop {
            if let w::WorkerOut::Done(d) = worker.read() {
                break d;
            }
        };
        assert_eq!(done.outcome, w::Outcome::Failed);
        assert_eq!(done.error.unwrap().code, w::Code::ProtocolError);
        assert_eq!(worker.child.wait().unwrap().code(), Some(2));
        assert!(!job.path.join("output/result.json").exists());
    }
}
#[test]
fn source_growth_is_source_changed() {
    let job = Job::new(1, 0, false, false);
    let mut grown = job.bytes.clone();
    grown.push(b' ');
    private_file(&job.source(), &grown);
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, _) = worker.done();
    assert_eq!(done.error.unwrap().code, w::Code::SourceChanged);
}

#[test]
fn real_worker_reuses_failure_retry_and_denial_fingerprints() {
    let mut job = Job::new(1, 0, false, false);
    let mut events: Vec<Envelope> = std::str::from_utf8(&job.bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let (session, turn) = events
        .iter()
        .find_map(|e| {
            if let Event::TurnStarted { turn, .. } = e.event {
                Some((e.session, turn))
            } else {
                None
            }
        })
        .unwrap();
    let mut inserted = vec![];
    let failed = ItemId::new();
    for (item, status, output) in [
        (
            failed,
            ItemStatus::Failed {
                reason: "missing".into(),
                code: None,
            },
            "MISSING /tmp/one 12\nsecond",
        ),
        (ItemId::new(), ItemStatus::Completed, "ok"),
    ] {
        let body = Item::ToolCall {
            call_id: item.to_string(),
            name: "Read".into(),
            tool_kind: ToolKind::Read,
            input_raw: r#"{"path":"/tmp/input"}"#.into(),
            output: vec![OutputBlock::Text {
                text: output.into(),
            }],
            locations: vec![],
            spill: None,
            title: None,
            server: None,
            raw_output: None,
        };
        inserted.push(Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: ItemKind::ToolCall,
            parent: None,
            head: body.head(),
        });
        inserted.push(Event::ItemCompleted { item, status, body });
    }
    let approval = ItemId::new();
    inserted.push(Event::ItemStarted {
        turn: Some(turn),
        item: approval,
        kind: ItemKind::Approval,
        parent: None,
        head: None,
    });
    inserted.push(Event::ItemCompleted {
        item: approval,
        status: ItemStatus::Completed,
        body: Item::Approval {
            subject: ApprovalSubject {
                tool_call: failed,
                effect: Effect::Read,
                summary: "Read".into(),
                paths: vec![],
                command: None,
            },
            options: vec![ApprovalDecisionKind::Deny],
            decision: Some(ApprovalDecision::Deny { reason: None }),
            by: Some(Resolver::Unattended),
        },
    });
    let template = events[2].clone();
    events.splice(
        3..3,
        inserted.into_iter().map(|event| Envelope {
            event,
            ..template.clone()
        }),
    );
    for (i, e) in events.iter_mut().enumerate() {
        e.seq = Some(Seq(i as u64 + 1));
        e.at = e.seq.unwrap();
        e.ts_ms = i as u64 + 1;
        e.session = session;
    }
    laws::check_transcript(&events).unwrap();
    job.bytes = events
        .iter()
        .flat_map(|e| e.wire_bytes().unwrap())
        .collect();
    let s = &mut job.manifest.sources[0];
    s.len = w::Number(job.bytes.len() as u64);
    s.sha256 = w::hash(&job.bytes);
    s.closed_seq = w::Number(events.len() as u64);
    let last = job.bytes[..job.bytes.len() - 1]
        .split(|b| *b == b'\n')
        .next_back()
        .unwrap();
    s.closed_line_sha256 = w::hash(last);
    private_file(&job.source(), &job.bytes);
    job.commit_manifest();
    let (result, rows, metrics) = complete(&job);
    assert_eq!((result.episode_count, result.cluster_count), (1, 4));
    let w::Row::Episode(e) = &rows[0] else {
        panic!()
    };
    let actual: std::collections::BTreeSet<_> =
        e.fingerprints.iter().map(|h| h.0.clone()).collect();
    let expected = [
        w::hash("quiet-stop").0,
        w::hash("approval-denied:Read").0,
        bboxy450_harness::dream::fingerprint::tool_fail("Read", "MISSING /tmp/one 12\nsecond"),
        bboxy450_harness::dream::fingerprint::retry("Read", r#"{"path":"/tmp/input"}"#),
    ]
    .into_iter()
    .collect();
    assert_eq!(actual, expected);
    eprintln!("real-fingerprints {metrics}");
}

#[tokio::test]
async fn actual_store_fork_is_explicitly_unsupported_and_parent_rows_never_pass() {
    use bboxy450_harness::store::{Store, log_path};
    let mut job = Job::new(0, 0, false, false);
    let first = Envelope::from_line(
        std::str::from_utf8(job.bytes.split_inclusive(|b| *b == b'\n').next().unwrap()).unwrap(),
    )
    .unwrap();
    let Event::SessionStarted { meta } = first.event else {
        panic!()
    };
    let sessions = job._home.path().join("store");
    let mut parent = Store::open(
        sessions.clone(),
        meta.clone(),
        Some(OpId::new()),
        Default::default(),
    )
    .await
    .unwrap();
    let turn = TurnId::new();
    for e in [
        Event::TurnQueued { turn },
        Event::TurnStarted {
            turn,
            model: None,
            origin: None,
        },
        Event::TurnEnded {
            turn,
            reason: StopReason::EndTurn,
            usage: None,
        },
    ] {
        parent.append(e, None).await.unwrap();
    }
    let mut child_meta = meta;
    child_meta.session = SessionId::new();
    child_meta.fork = Some((parent.session, parent.head()));
    let mut child = Store::open(
        sessions.clone(),
        child_meta.clone(),
        Some(OpId::new()),
        Default::default(),
    )
    .await
    .unwrap();
    child.append(Event::SessionClosed, None).await.unwrap();
    let original = std::fs::read(log_path(&sessions, child_meta.session)).unwrap();
    let events: Vec<Envelope> = std::str::from_utf8(&original)
        .unwrap()
        .split_inclusive('\n')
        .map(|l| Envelope::from_line(l).unwrap())
        .collect();
    laws::check_transcript(&events).unwrap();
    assert!(events.iter().skip(1).any(|e| e.session == parent.session));
    for mode in 0..3 {
        let mut copied = events.clone();
        if mode == 1 {
            let last = copied.last_mut().unwrap();
            last.session = parent.session;
            assert!(laws::check_transcript(&copied).is_err());
        }
        if mode == 2 {
            let Event::SessionStarted { meta } = &mut copied[0].event else {
                panic!()
            };
            meta.fork = None;
        }
        let bytes = if mode == 0 {
            original.clone()
        } else {
            copied
                .iter_mut()
                .flat_map(|e| {
                    e.raw = None;
                    e.wire_bytes().unwrap()
                })
                .collect()
        };
        job.bytes = bytes.clone();
        let s = &mut job.manifest.sources[0];
        s.session = w::Ulid(child_meta.session.to_string());
        s.len = w::Number(bytes.len() as u64);
        s.sha256 = w::hash(&bytes);
        s.closed_seq = w::Number(copied.len() as u64);
        s.closed_line_sha256 = w::hash(
            bytes[..bytes.len() - 1]
                .split(|b| *b == b'\n')
                .next_back()
                .unwrap(),
        );
        private_file(&job.source(), &bytes);
        job.commit_manifest();
        if mode == 0
            && let Some(path) = std::env::var_os("DREAM_C1_EXPORT_FORK_BUNDLE")
        {
            let path = std::path::PathBuf::from(path);
            assert!(path.is_absolute());
            private_dir(&path);
            private_dir(&path.join("sessions"));
            private_file(
                &path
                    .join("sessions")
                    .join(format!("{}.jsonl", child_meta.session)),
                &original,
            );
            private_file(
                &path.join("manifest.json"),
                &w::canonical(&job.manifest).unwrap(),
            );
            eprintln!(
                "real-Store fork_export={} session={} repo={} ir_len={} ir_sha256={} closed_seq={} closed_line_sha256={}",
                path.display(),
                child_meta.session,
                job.manifest.sources[0].repo.0,
                original.len(),
                job.manifest.sources[0].sha256.0,
                job.manifest.sources[0].closed_seq.0,
                job.manifest.sources[0].closed_line_sha256.0
            );
        }
        let mut worker = job.launch();
        worker.start(&job.start);
        let (done, metrics) = worker.done();
        assert_eq!(
            done.error.unwrap().code,
            if mode == 2 {
                w::Code::MalformedIr
            } else {
                w::Code::UnsupportedIr
            },
            "fork mode={mode} {metrics}"
        );
        assert!(!job.path.join("output/result.json").exists());
        assert_eq!(bytes, std::fs::read(job.source()).unwrap());
    }
    assert_eq!(
        original,
        std::fs::read(log_path(&sessions, child_meta.session)).unwrap()
    );
}

#[test]
fn paused_owner_does_not_delay_another_process_and_owned_kill_never_completes() {
    let mut first = Job::new(40, 60 * 1024, false, false);
    first.start.foreground = true;
    let mut worker = first.launch();
    worker.start(&first.start);
    let at = Instant::now();
    loop {
        if let w::WorkerOut::Progress(p) = worker.read() {
            if p.phase == w::Phase::Paused {
                break;
            }
        }
    }
    assert!(at.elapsed() < Duration::from_millis(100));
    let second = Job::new(0, 0, false, false);
    let at = Instant::now();
    complete(&second);
    assert!(
        at.elapsed() < Duration::from_secs(2),
        "paused root held another worker"
    );
    assert!(
        std::fs::read_dir(first.path.join("output"))
            .unwrap()
            .next()
            .is_none()
    );
    let at = Instant::now();
    worker.cancel();
    let done = loop {
        if let w::WorkerOut::Done(d) = worker.read() {
            break d;
        }
    };
    assert_eq!(done.outcome, w::Outcome::Cancelled);
    assert!(worker.child.wait().unwrap().success());
    assert!(at.elapsed() < Duration::from_millis(250));
    let mut killed = first.launch();
    first.start.foreground = false;
    killed.start(&first.start);
    loop {
        if let w::WorkerOut::Progress(p) = killed.read() {
            if p.phase == w::Phase::Validate {
                break;
            }
        }
    }
    killed.child.kill().unwrap();
    let status = killed.child.wait().unwrap();
    assert!(!status.success());
    assert!(!first.path.join("output/result.json").exists());
    assert_eq!(first.bytes, std::fs::read(first.source()).unwrap());
}

#[test]
fn actual_over_limit_ir_corpus_is_rejected_before_reading() {
    use sha2::{Digest, Sha256};
    let mut job = Job::new(0, 0, false, false);
    let first = Envelope::from_line(
        std::str::from_utf8(job.bytes.split_inclusive(|b| *b == b'\n').next().unwrap()).unwrap(),
    )
    .unwrap();
    let session = first.session;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(job.source())
        .unwrap();
    let mut hash = Sha256::new();
    let mut seq = 0u64;
    let mut len = 0u64;
    let mut final_hash = w::hash([]);
    let mut push = |event| {
        seq += 1;
        let e = Envelope {
            event,
            seq: Some(Seq(seq)),
            at: Seq(seq),
            ts_ms: seq,
            raw: None,
            ..first.clone()
        };
        let line = e.wire_bytes().unwrap();
        assert!(line.len() <= w::MAX_FRAME);
        hash.update(&line);
        file.write_all(&line).unwrap();
        len += line.len() as u64;
        final_hash = w::hash(&line[..line.len() - 1]);
    };
    push(first.event.clone());
    let turn = TurnId::new();
    push(Event::TurnQueued { turn });
    push(Event::TurnStarted {
        turn,
        model: None,
        origin: None,
    });
    let payload = "x".repeat(128 * 1024);
    for _ in 0..512 {
        let item = ItemId::new();
        push(Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: ItemKind::AgentMessage,
            parent: None,
            head: None,
        });
        push(Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: Item::AgentMessage {
                markdown: payload.clone(),
            },
        });
    }
    push(Event::TurnEnded {
        turn,
        reason: StopReason::EndTurn,
        usage: None,
    });
    push(Event::SessionClosed);
    drop(push);
    file.sync_all().unwrap();
    drop(file);
    assert!(len > 64 * 1024 * 1024);
    assert_eq!(std::fs::metadata(job.source()).unwrap().len(), len);
    let s = &mut job.manifest.sources[0];
    s.session = w::Ulid(session.to_string());
    s.len = w::Number(len);
    s.sha256 = w::Hex(format!("{:x}", hash.finalize()));
    s.closed_seq = w::Number(seq);
    s.closed_line_sha256 = final_hash;
    job.commit_manifest();
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.error.unwrap().code, w::Code::ResourceLimit);
    assert!(metrics.contains("envelopes=0 bytes=0"));
    assert!(
        std::fs::read_dir(job.path.join("output"))
            .unwrap()
            .next()
            .is_none()
    );
    assert_eq!(std::fs::metadata(job.source()).unwrap().len(), len);
    eprintln!("actual-over-limit source_bytes={len} {metrics}");
}

fn metric(metrics: &str, key: &str) -> u64 {
    metrics
        .split_whitespace()
        .find_map(|field| field.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("missing {key}: {metrics}"))
        .parse()
        .unwrap()
}
fn assert_finite_usage(metrics: &str, key: &str) {
    assert_ne!(
        metric(metrics, key),
        u64::MAX,
        "unavailable {key} accepted as finite"
    );
}
#[test]
fn unavailable_tail_usage_is_rejected_by_the_actual_metrics_gate() {
    for key in [
        "process_user_cpu_ns",
        "process_system_cpu_ns",
        "owner_thread_cpu_ns",
    ] {
        let invalid = format!("{key}={}", u64::MAX);
        assert!(std::panic::catch_unwind(|| assert_finite_usage(&invalid, key)).is_err());
        assert_finite_usage(&format!("{key}=1"), key);
    }
}
fn assert_native_gate(metrics: &str) {
    let terminal = metrics
        .lines()
        .find(|l| l.starts_with("dream-c1 terminal_metrics "))
        .unwrap();
    assert!(terminal.contains("profile=c1-native-debt/1"), "{metrics}");
    assert!(metrics.contains("cost_known=true"), "{metrics}");
    assert!(metric(terminal, "app_max_cpu_ns") <= 1_000_000, "{metrics}");
    assert!(
        metric(terminal, "control_max_cpu_ns") <= 1_000_000,
        "{metrics}"
    );
    assert!(
        metric(terminal, "inclusive_max_cpu_ns") <= 3_000_000,
        "{metrics}"
    );
    assert!(metric(terminal, "captured_max_cpu_ns") <= metric(terminal, "inclusive_max_cpu_ns"));
    assert_eq!(metric(terminal, "remaining_debt_ns"), 0);
    for key in [
        "process_user_cpu_ns",
        "process_system_cpu_ns",
        "owner_thread_cpu_ns",
    ] {
        assert_finite_usage(terminal, key);
    }
    assert!(metric(terminal, "process_user_cpu_ns") > 0);
    assert!(metric(terminal, "owner_thread_cpu_ns") > 0);
    for tag in ["compute_owner_create", "publish_noreplace"] {
        let line = metrics
            .lines()
            .find(|l| l.starts_with(&format!("dream-c1 native tag={tag} ")))
            .unwrap();
        assert!(metric(line, "max_cpu_ns") <= 2_000_000, "{line}");
        assert!(metric(line, "max_wall_ns") <= 25_000_000, "{line}");
        assert!(metric(line, "inclusive_max_cpu_ns") <= 3_000_000, "{line}");
        assert!(metric(line, "app_max_cpu_ns") <= 1_000_000, "{line}");
        assert!(
            metric(line, "max_wall_ns") <= metric(line, "inclusive_max_wall_ns"),
            "{line}"
        );
        assert_eq!(metric(line, "failures_with_effect"), 0);
        assert!(
            metric(line, "count")
                <= if tag == "compute_owner_create" {
                    1
                } else {
                    4097
                }
        );
    }
}
fn assert_memory_gate(metrics: &str) {
    assert_eq!(
        metrics
            .lines()
            .filter(|l| l.starts_with("dream-c1 metrics "))
            .count(),
        1
    );
    assert!(metric(metrics, "heap_live_bytes") <= 96 * 1024 * 1024);
    assert!(metric(metrics, "heap_peak_bytes") <= 96 * 1024 * 1024);
    assert!(metric(metrics, "storage_reserved_peak_bytes") <= 78 * 1024 * 1024);
    assert_eq!(metric(metrics, "storage_declared_live_bytes"), 0);
    assert_eq!(metric(metrics, "control_reserved_bytes"), 2 * 1024 * 1024);
    assert_eq!(metric(metrics, "helper_reserved_bytes"), 16 * 1024 * 1024);
    assert_eq!(metric(metrics, "credit_ceiling_bytes"), 96 * 1024 * 1024);
    assert!(metrics.contains("native_cumulative_bytes=unmeasured"));
    assert_eq!(metric(metrics, "overruns"), 0);
    assert!(metric(metrics, "max_unit_cpu_ns") <= 3_000_000);
    assert_native_gate(metrics);
}
#[test]
fn child_declared_storage_overflow_precedes_serde_and_controls_still_settle() {
    let mut job = Job::new(1, 0, false, false);
    let mut lines = job
        .bytes
        .split(|b| *b == b'\n')
        .filter(|v| !v.is_empty())
        .map(|v| serde_json::from_slice::<serde_json::Value>(v).unwrap())
        .collect::<Vec<_>>();
    // Each container and the entire document stay below their independent caps.
    // The much larger declared typed capacity must be refused before materialization.
    lines[0]["ext"]["dense"] = serde_json::json!(vec![vec![0u8; 4096]; 10]);
    job.bytes.clear();
    for line in &lines {
        job.bytes.extend(serde_json::to_vec(line).unwrap());
        job.bytes.push(b'\n');
    }
    job.manifest.sources[0].len = w::Number(job.bytes.len() as u64);
    job.manifest.sources[0].sha256 = w::hash(&job.bytes);
    private_file(&job.source(), &job.bytes);
    job.commit_manifest();
    let mut worker = job.launch();
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(
        done.error.unwrap().code,
        w::Code::ResourceLimit,
        "{metrics}"
    );
    assert_memory_gate(&metrics);
    assert!(!job.path.join("output/result.json").exists());
    assert_eq!(std::fs::read(job.source()).unwrap(), job.bytes);
    eprintln!("declared-child-overflow {metrics}");
}
#[test]
fn four_children_keep_independent_controls_and_individual_memory_shares() {
    let jobs = (0..4)
        .map(|_| {
            let mut j = Job::new(20, 128 * 1024, false, false);
            j.start.foreground = true;
            j
        })
        .collect::<Vec<_>>();
    let mut workers = jobs.iter().map(Job::launch).collect::<Vec<_>>();
    for (worker, job) in workers.iter_mut().zip(&jobs) {
        worker.start(&job.start);
    }
    for worker in &mut workers {
        loop {
            if matches!(
                worker.read(),
                w::WorkerOut::Progress(w::Progress {
                    phase: w::Phase::Paused,
                    ..
                })
            ) {
                break;
            }
        }
    }
    // Resume one large input; the three paused owners retain immediate Cancel.
    workers[0].activity(2, false);
    for worker in &mut workers[1..] {
        worker.cancel();
    }
    for worker in &mut workers[1..] {
        let at = Instant::now();
        let terminal = loop {
            if let w::WorkerOut::Done(d) = worker.read() {
                break d;
            }
        };
        assert_eq!(terminal.outcome, w::Outcome::Cancelled);
        assert!(worker.child.wait().unwrap().success());
        assert!(at.elapsed() < Duration::from_millis(500));
        let mut metrics = String::new();
        std::io::Read::read_to_string(&mut worker.child.stderr.take().unwrap(), &mut metrics)
            .unwrap();
        assert_memory_gate(&metrics);
    }
    let terminal = loop {
        if let w::WorkerOut::Done(d) = workers[0].read() {
            break d;
        }
    };
    let status = workers[0].child.wait().unwrap();
    let mut metrics = String::new();
    std::io::Read::read_to_string(&mut workers[0].child.stderr.take().unwrap(), &mut metrics)
        .unwrap();
    assert_eq!(
        terminal.outcome,
        w::Outcome::Completed,
        "{terminal:?} {metrics}"
    );
    assert!(status.success(), "{metrics}");
    assert_memory_gate(&metrics);
    eprintln!("four-child-large-input {metrics}");
}

#[test]
fn near_limit_rejected_envelope_drains_before_process_done_and_join() {
    // A real private worker parses an owned, near-frame header before the law
    // rejects its unsupported version. This pairs the exact injected-clock and
    // post-parse Cancel API witnesses with the actual pipe/Done/join boundary.
    let mut job = Job::new(0, 0, false, false);
    let lines = job
        .bytes
        .split_inclusive(|c| *c == b'\n')
        .collect::<Vec<_>>();
    let mut header: serde_json::Value = serde_json::from_slice(lines[0]).unwrap();
    header["v"]["major"] = serde_json::json!(2);
    header["ext"] = serde_json::json!({"dense": (0..4096).map(|_| serde_json::json!({"leaf":["retained"]})).collect::<Vec<_>>(), "padding":""});
    let initial = serde_json::to_vec(&header).unwrap();
    header["ext"]["padding"] = serde_json::json!("p".repeat(w::MAX_FRAME - initial.len() - 32));
    let mut bytes = serde_json::to_vec(&header).unwrap();
    assert_eq!(bytes.len(), w::MAX_FRAME - 32);
    bytes.push(b'\n');
    for line in &lines[1..] {
        bytes.extend_from_slice(line);
    }
    job.bytes = bytes;
    private_file(&job.source(), &job.bytes);
    job.manifest.sources[0].len = w::Number(job.bytes.len() as u64);
    job.manifest.sources[0].sha256 = w::hash(&job.bytes);
    job.commit_manifest();
    let mut worker = job.launch();
    let at = Instant::now();
    worker.start(&job.start);
    let (done, metrics) = worker.done();
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert!(matches!(
        done.error.as_ref().unwrap().code,
        w::Code::UnsupportedIr | w::Code::ResourceLimit
    ));
    assert!(at.elapsed() < Duration::from_secs(2));
    // The intentionally dense parse can itself exceed one CPU unit. That is a
    // causal negative, not a relaxed success threshold: it must fail ResourceLimit
    // and retain the measured overrun while cleanup still respects its bound.
    if done.error.as_ref().unwrap().code == w::Code::ResourceLimit {
        assert!(metric(&metrics, "overruns") > 0, "{metrics}");
        assert!(metric(&metrics, "max_unit_cpu_ns") > 1_000_000, "{metrics}");
        assert_eq!(metric(&metrics, "storage_declared_live_bytes"), 0);
        assert!(metric(&metrics, "heap_peak_bytes") <= 96 * 1024 * 1024);
    } else {
        assert_memory_gate(&metrics);
    }
    assert!(
        metric(&metrics, "max_cleanup_unit_cpu_ns") <= 1_000_000,
        "{metrics}"
    );
    assert!(metric(&metrics, "cleanup_cpu_ns") > 0, "{metrics}");
    assert!(metric(&metrics, "storage_reserved_peak_bytes") > 1024 * 1024);
    assert!(metric(&metrics, "compute_cpu_ns") > 0);
    assert!(
        std::fs::read_dir(job.path.join("output"))
            .unwrap()
            .next()
            .is_none()
    );
    eprintln!(
        "near-limit-rejected-owner elapsed_ms={} {metrics}",
        at.elapsed().as_millis()
    );
}

#[test]
fn cancel_after_retained_state_growth_reaps_with_paced_teardown() {
    let job = Job::new(80, 16 * 1024, false, false);
    let mut worker = job.launch();
    worker.start(&job.start);
    loop {
        if let w::WorkerOut::Progress(p) = worker.read() {
            if p.envelopes.0 >= 100 {
                break;
            }
        }
    }
    let begin = Instant::now();
    worker.cancel();
    let done = loop {
        if let w::WorkerOut::Done(d) = worker.read() {
            break d;
        }
    };
    assert_eq!(done.outcome, w::Outcome::Cancelled);
    assert!(worker.child.wait().unwrap().success());
    assert!(begin.elapsed() < Duration::from_millis(500));
    let mut metrics = String::new();
    std::io::Read::read_to_string(&mut worker.child.stderr.take().unwrap(), &mut metrics).unwrap();
    assert_memory_gate(&metrics);
    assert!(metric(&metrics, "storage_reserved_peak_bytes") > 1024 * 1024);
    eprintln!("retained-cancel {metrics}");
}

#[tokio::test]
async fn real_store_multi_mib_ir_and_typed_blob_bundle() {
    use bboxy450_harness::store::{Store, log_path};
    let mut job = Job::new(0, 0, false, false);
    let Event::SessionStarted { meta } = Envelope::from_line(
        std::str::from_utf8(job.bytes.split_inclusive(|b| *b == b'\n').next().unwrap()).unwrap(),
    )
    .unwrap()
    .event
    else {
        panic!()
    };
    let sessions = job._home.path().join("real-store");
    let mut store = Store::open(
        sessions.clone(),
        meta.clone(),
        Some(OpId::new()),
        Default::default(),
    )
    .await
    .unwrap();
    let turn = TurnId::new();
    store
        .append(Event::TurnQueued { turn }, None)
        .await
        .unwrap();
    store
        .append(
            Event::TurnStarted {
                turn,
                model: None,
                origin: None,
            },
            None,
        )
        .await
        .unwrap();
    for _ in 0..32 {
        let item = ItemId::new();
        store
            .append(
                Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::AgentMessage,
                    parent: None,
                    head: None,
                },
                None,
            )
            .await
            .unwrap();
        store
            .append(
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::AgentMessage {
                        markdown: "x".repeat(96 * 1024),
                    },
                },
                None,
            )
            .await
            .unwrap();
    }
    let blob = serde_json::to_vec(
        &serde_json::json!({"type":"private_future_kind","payload":"x".repeat(4*1024*1024)}),
    )
    .unwrap();
    let reference = store
        .blobs
        .put(blob.clone(), "application/json")
        .await
        .unwrap();
    let item = ItemId::new();
    store
        .append(
            Event::ItemStarted {
                turn: Some(turn),
                item,
                kind: ItemKind::Native,
                parent: None,
                head: None,
            },
            None,
        )
        .await
        .unwrap();
    store
        .append(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::Native {
                    provider: "private-fixture".into(),
                    record_type: "typed-json-blob".into(),
                    raw: serde_json::Value::Null,
                    blob: Some(reference.clone()),
                },
            },
            None,
        )
        .await
        .unwrap();
    store
        .append(
            Event::TurnEnded {
                turn,
                reason: StopReason::Quiet { ms: 1 },
                usage: None,
            },
            None,
        )
        .await
        .unwrap();
    let closed = store.append(Event::SessionClosed, None).await.unwrap();
    drop(store);
    job.bytes = std::fs::read(log_path(&sessions, meta.session)).unwrap();
    assert!(job.bytes.len() > 3 * 1024 * 1024);
    let s = &mut job.manifest.sources[0];
    s.len = w::Number(job.bytes.len() as u64);
    s.sha256 = w::hash(&job.bytes);
    s.closed_seq = w::Number(closed.seq.unwrap().0);
    let last = job.bytes.split_inclusive(|b| *b == b'\n').last().unwrap();
    s.closed_line_sha256 = w::hash(&last[..last.len() - 1]);
    s.blobs = vec![w::Blob {
        sha256: w::Hex(reference.sha256.clone()),
        len: w::Number(reference.len),
        mime: w::Mime(reference.mime.clone()),
    }];
    let session_dir = job.path.join("input").join(meta.session.to_string());
    private_dir(&session_dir);
    private_dir(&session_dir.join("blobs"));
    private_file(&session_dir.join("blobs").join(&reference.sha256), &blob);
    private_file(&job.source(), &job.bytes);
    job.commit_manifest();
    laws::check_transcript(
        &std::str::from_utf8(&job.bytes)
            .unwrap()
            .split_inclusive('\n')
            .map(|l| Envelope::from_line(l).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let (result, _, metrics) = complete(&job);
    assert_eq!(result.episode_count, 1);
    assert_memory_gate(&metrics);
    if let Some(path) = std::env::var_os("DREAM_C1_EXPORT_BUNDLE") {
        let path = std::path::PathBuf::from(path);
        assert!(path.is_absolute());
        // Caller supplies an absent private export path; never overwrite or repair.
        private_dir(&path);
        private_dir(&path.join("sessions"));
        private_file(
            &path
                .join("sessions")
                .join(format!("{}.jsonl", meta.session)),
            &job.bytes,
        );
        let dir = path.join("sessions").join(meta.session.to_string());
        private_dir(&dir);
        private_dir(&dir.join("blobs"));
        private_file(&dir.join("blobs").join(&reference.sha256), &blob);
        private_file(
            &path.join("manifest.json"),
            &w::canonical(&job.manifest).unwrap(),
        );
        eprintln!(
            "real-Store exported_bundle={} session={} repo={} ir_len={} ir_sha256={} blob_len={} blob_sha256={}",
            path.display(),
            meta.session,
            job.manifest.sources[0].repo.0,
            job.bytes.len(),
            job.manifest.sources[0].sha256.0,
            reference.len,
            reference.sha256
        );
    }
    eprintln!("real-Store-multi-MiB {metrics}");
}

#[test]
fn four_ordinary_release_workers_complete_multi_mib_with_independent_control_and_reap() {
    let jobs: Vec<_> = (0..4)
        .map(|i| {
            let mut job = Job::new(40, 60 * 1024, false, false);
            job.manifest.context.root = w::Id(format!("fixture-root-{}", i / 2));
            job.commit_manifest();
            job
        })
        .collect();
    assert!(jobs.iter().all(|j| j.bytes.len() > 2 * 1024 * 1024));
    let mut workers: Vec<_> = jobs.iter().map(Job::launch).collect();
    for (worker, job) in workers.iter_mut().zip(&jobs) {
        let mut start = job.start.clone();
        start.foreground = true;
        worker.start(&start);
    }
    for worker in &mut workers {
        match worker.read() {
            w::WorkerOut::Progress(p) if p.phase == w::Phase::Paused => assert_eq!(p.bytes.0, 0),
            other => panic!("missing initial compute Paused: {other:?}"),
        }
    }
    // Causal negative: initial revision0 is ignored. The actual response timer
    // must fail on silence, rather than consume a queued initial Paused frame.
    assert!(
        workers[0]
            .timed_paused_activity(0, Duration::from_millis(25))
            .is_none()
    );
    assert!(workers[0].child.try_wait().unwrap().is_none());
    // Delay a genuinely NEW Activity by withholding only its final LF. No
    // request is outstanding after the stale control; before LF the same real
    // response timer must refuse silence. Then drain its actual response.
    let start = workers[0].start.clone().unwrap();
    let delayed = w::WorkerIn::Activity(w::Activity {
        protocol: w::Protocol::V1,
        run_id: start.run_id,
        nonce: start.nonce,
        revision: w::Number(2),
        foreground: true,
    });
    workers[0]
        .input
        .as_mut()
        .unwrap()
        .write_all(&w::canonical(&delayed).unwrap())
        .unwrap();
    assert!(
        workers[0]
            .wait_paused_response(Instant::now(), Duration::from_millis(25))
            .is_none()
    );
    workers[0].input.as_mut().unwrap().write_all(b"\n").unwrap();
    assert!(
        workers[0]
            .wait_paused_response(Instant::now(), Duration::from_millis(250))
            .is_some()
    );
    let mut control_times = Vec::new();
    // Start already carried revision1; revision2 was consumed by the delayed
    // causal control. Every timed revision below is strictly newer for all4.
    for revision in 3..=52 {
        for worker in &mut workers {
            control_times.push(
                worker
                    .timed_paused_activity(revision, Duration::from_millis(250))
                    .expect("new Activity had no causally subsequent response"),
            );
        }
    }
    control_times.sort();
    assert_eq!(control_times.len(), 200);
    assert!(control_times[198] <= Duration::from_millis(100));
    assert!(control_times[199] <= Duration::from_millis(250));
    for worker in &mut workers {
        worker.activity(53, false);
    }
    eprintln!(
        "four-ordinary-release actual_control_roundtrips=200 initial_paused_drained=true ignored_revision_control=timeout delayed_fresh_activity_control=timeout_then_response p99_us={} max_us={}",
        control_times[198].as_micros(),
        control_times[199].as_micros()
    );
    for (i, worker) in workers.iter_mut().enumerate() {
        let (done, metrics) = worker.done();
        assert_eq!(
            done.outcome,
            w::Outcome::Completed,
            "job={i} {done:?} {metrics}"
        );
        assert_memory_gate(&metrics);
        assert_eq!(done.pages, 1);
        eprintln!("four-ordinary-release job={i} reaped=true {metrics}");
    }
}

#[path = "fixtures/dream-bootstrap/process.rs"]
mod bootstrap_process;

#[cfg(feature = "test-fixtures")]
#[test]
fn bootstrap_earliest_gate_bypass_control_rejects_actual_prepermit_allocation() {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let binary = std::path::PathBuf::from(env!("CARGO_BIN_EXE_bboxy450-harness"));
    let mut source = std::fs::File::open(&binary).unwrap();
    let mut sha = Sha256::new();
    let mut buf = [0; 16384];
    loop {
        let at = parent_cpu();
        let n = source.read(&mut buf).unwrap();
        sha.update(&buf[..n]);
        let cpu = parent_interval(at, parent_cpu());
        let end = Instant::now();
        assert!(
            cpu <= 1_000_000,
            "binary approval chunk len={n} cpu_ns={cpu}"
        );
        parent_rest(cpu, end);
        if n == 0 {
            break;
        }
    }
    let digest = w::Hex(format!("{:x}", sha.finalize()));
    // Cargo supplies this exact fixture binary from the current source/features;
    // declare its hash/profile before birth. It is never a managed capability
    // candidate or a substitute for the ordinary-release positive artifact.
    eprintln!(
        "bootstrap-causal-artifact profile=dream-bootstrap/1 features=test-fixtures kind=earliest-bypass sha256={} binary={}",
        digest.0,
        binary.display()
    );
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join(ulid::Ulid::new().to_string());
    let nonce = w::Ulid(ulid::Ulid::new().to_string());
    let mut held = HeldWorker::spawn_on(&path, &nonce, None, true, &binary, &digest, true);
    // This is a deliberately invalid artifact, never a positive readiness
    // candidate. Revoke immediately, then retain/reap its actual effect. A cold
    // loader may not reach entry within250ms; that never grants extra readiness.
    // The negative still requires the actual source-bound allocation marker,
    // rather than treating silence/timeout as proof of a causal bypass.
    drop(held.lease.take());
    drop(held.child.as_mut().unwrap().stdin.take());
    settle_owned(held.child.as_mut().unwrap());
    let status = held.child.as_mut().unwrap().try_wait().unwrap().unwrap();
    assert_eq!(status.code(), Some(2));
    if let Ok(cpu) = held.launch_verdict() {
        parent_rest(cpu, held.birth_end);
    }
    let stderr = held.child.as_mut().unwrap().stderr.take().unwrap();
    stderr.take(1024).read_to_string(&mut held.marker).unwrap();
    eprintln!(
        "bootstrap-bypass causal_witness_cutoff=actual_reap readiness_certified=false grant_count=0 marker={}",
        held.marker.trim()
    );
    assert!(
        held.marker.contains("argv_admissions=1"),
        "mutant did not execute its controlled allocation"
    );
    assert!(
        held.waiting_verdict().is_err(),
        "actual earliest-gate checker accepted the causal bypass"
    );
    let pid = held.child.as_ref().unwrap().id();
    assert!(!path.exists());
    eprintln!(
        "bootstrap-earliest-causal-control actual_reaped_pid={pid} rejected_prepermit_allocation=true"
    );
}
