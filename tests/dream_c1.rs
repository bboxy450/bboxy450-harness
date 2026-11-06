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
fn finite_parent_interval(start: u64, end: u64) -> Option<u64> {
    (start != u64::MAX && end != u64::MAX)
        .then(|| end.checked_sub(start))
        .flatten()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
struct NativeBirthObservation {
    native_cpu_ns: Option<u64>,
    native_wall_ns: u128,
    cpu_over_budget: Option<bool>,
    wall_over_budget: bool,
    unknown_cpu: bool,
}
impl NativeBirthObservation {
    fn new(cpu: Option<u64>, wall: Duration) -> Self {
        let cpu = cpu.filter(|n| *n != u64::MAX);
        Self {
            native_cpu_ns: cpu,
            native_wall_ns: wall.as_nanos(),
            cpu_over_budget: cpu.map(|n| n > 2_000_000),
            wall_over_budget: wall > Duration::from_millis(25),
            unknown_cpu: cpu.is_none(),
        }
    }
    fn verdict(self) -> Result<u64, &'static str> {
        let cpu = self.native_cpu_ns.ok_or("unknown native birth cost")?;
        if self.cpu_over_budget == Some(true) || self.wall_over_budget {
            return Err("native birth allocation");
        }
        Ok(cpu)
    }
}
struct NativeBirthDiagnostic {
    pid: u32,
    run: w::Ulid,
    artifact: w::Hex,
    cpu_start_ns: Option<u64>,
    cpu_end_ns: Option<u64>,
    original: NativeBirthObservation,
    applied_guard: std::cell::Cell<Option<NativeBirthObservation>>,
    permit_attempted: bool,
    diagnostic_only: bool,
}
#[derive(Clone, Copy, Debug)]
enum NativeSettlement {
    ExactPidWait4,
    HeldDrop,
    WorkerDrop,
    Injected,
}
impl NativeSettlement {
    fn label(self) -> &'static str {
        match self {
            Self::ExactPidWait4 => "exact_pid_wait4",
            Self::HeldDrop => "held_drop_after_settle_owned",
            Self::WorkerDrop => "worker_drop_after_settle_owned",
            Self::Injected => "injected",
        }
    }
}
struct JsonOption<T>(Option<T>);
impl<T: std::fmt::Display> std::fmt::Display for JsonOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => f.write_str("null"),
        }
    }
}
impl std::fmt::Display for NativeBirthObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"native_cpu_ns\":{},\"native_wall_ns\":{},\"cpu_over_budget\":{},\"wall_over_budget\":{},\"unknown_cpu\":{}}}",
            JsonOption(self.native_cpu_ns),
            self.native_wall_ns,
            JsonOption(self.cpu_over_budget),
            self.wall_over_budget,
            self.unknown_cpu
        )
    }
}
#[derive(Clone, Copy)]
struct NativeDiagnosticIdentity {
    pid: u32,
    run: [u8; 26],
    artifact: [u8; 64],
}
impl NativeDiagnosticIdentity {
    fn new(record: &NativeBirthDiagnostic) -> Option<Self> {
        // Birth preparation already validated these identities. Recheck exact
        // bounded alphabets so direct JSON requires neither escaping nor heap.
        boot::ulid(record.run.0.as_bytes()).ok()?;
        if !record.artifact.0.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        Some(Self {
            pid: record.pid,
            run: record.run.0.as_bytes().try_into().ok()?,
            artifact: record.artifact.0.as_bytes().try_into().ok()?,
        })
    }
    fn run(&self) -> &str {
        std::str::from_utf8(&self.run).unwrap()
    }
    fn artifact(&self) -> &str {
        std::str::from_utf8(&self.artifact).unwrap()
    }
}
impl std::fmt::Debug for NativeDiagnosticIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeDiagnosticIdentity")
            .field("pid", &self.pid)
            .field("run", &self.run())
            .field("artifact", &self.artifact())
            .finish()
    }
}
#[derive(Clone, Copy, Debug)]
struct NativeBirthRow {
    identity: NativeDiagnosticIdentity,
    cpu_start_ns: Option<u64>,
    cpu_end_ns: Option<u64>,
    original: NativeBirthObservation,
    applied_guard: Option<NativeBirthObservation>,
    permit_attempted: bool,
    settled: bool,
    via: NativeSettlement,
    exit_code: Option<i32>,
    exit_signal: Option<i32>,
    diagnostic_only: bool,
}
impl std::fmt::Display for NativeBirthRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "{{\"event\":\"dream-c1 parent_native_birth\",\"pid\":{},",
                "\"run_id\":\"{}\",\"artifact_sha256\":\"{}\",",
                "\"cpu_start_ns\":{},\"cpu_end_ns\":{},\"original\":{},\"original_verdict\":"
            ),
            self.identity.pid,
            self.identity.run(),
            self.identity.artifact(),
            JsonOption(self.cpu_start_ns),
            JsonOption(self.cpu_end_ns),
            self.original
        )?;
        match self.original.verdict().err() {
            Some(reason) => write!(f, "\"{reason}\"")?,
            None => f.write_str("null")?,
        }
        write!(
            f,
            concat!(
                ",\"applied_guard\":{},\"native_guard_admitted\":{},",
                "\"permit_attempted\":{},\"actual_settled\":{},\"settled_via\":\"{}\",",
                "\"exit_code\":{},\"exit_signal\":{},\"diagnostic_only\":{},\"runtime_conformance\":false}}"
            ),
            JsonOption(self.applied_guard),
            JsonOption(self.applied_guard.map(|s| s.verdict().is_ok())),
            self.permit_attempted,
            self.settled,
            self.via.label(),
            JsonOption(self.exit_code),
            JsonOption(self.exit_signal),
            self.diagnostic_only
        )
    }
}
const NATIVE_DIAGNOSTIC_CAPACITY: usize = 2048;
struct NativeDiagnosticLine<const N: usize> {
    bytes: [u8; N],
    len: usize,
}
impl<const N: usize> NativeDiagnosticLine<N> {
    fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }
    fn format(&mut self, value: impl std::fmt::Display) -> bool {
        use std::fmt::Write;
        self.len = 0;
        writeln!(self, "{value}").is_ok()
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
impl<const N: usize> std::fmt::Write for NativeDiagnosticLine<N> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(std::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(std::fmt::Error)?;
        target.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NativeDiagnosticCost {
    start_ns: Option<u64>,
    end_ns: Option<u64>,
    cpu_ns: Option<u64>,
    owed_ns: Option<u64>,
    over_budget: Option<bool>,
    repayment_overflow: bool,
}
impl NativeDiagnosticCost {
    fn new(start: u64, end: u64) -> Self {
        let cpu_ns = finite_parent_interval(start, end);
        let owed_ns = cpu_ns.and_then(|n| n.max(1_000_000).checked_mul(9));
        Self {
            start_ns: (start != u64::MAX).then_some(start),
            end_ns: (end != u64::MAX).then_some(end),
            cpu_ns,
            owed_ns,
            over_budget: cpu_ns.map(|n| n > 1_000_000),
            repayment_overflow: cpu_ns.is_some() && owed_ns.is_none(),
        }
    }
    fn accepted(self) -> bool {
        self.over_budget == Some(false) && self.owed_ns.is_some() && !self.repayment_overflow
    }
}
impl std::fmt::Display for NativeDiagnosticCost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"start_ns\":{},\"end_ns\":{},\"cpu_ns\":{},\"owed_ns\":{},\"over_budget\":{},\"repayment_overflow\":{}}}",
            JsonOption(self.start_ns),
            JsonOption(self.end_ns),
            JsonOption(self.cpu_ns),
            JsonOption(self.owed_ns),
            JsonOption(self.over_budget),
            self.repayment_overflow
        )
    }
}
fn native_diagnostic_step<T>(
    clock: &mut impl FnMut() -> u64,
    pay: &mut impl FnMut(NativeDiagnosticCost, Instant),
    work: impl FnOnce() -> T,
) -> (T, NativeDiagnosticCost) {
    let start = clock();
    let result = work();
    let cost = NativeDiagnosticCost::new(start, clock());
    pay(cost, Instant::now());
    (result, cost)
}
#[derive(Clone, Copy, Debug)]
struct NativeDiagnosticWrite {
    attempted: bool,
    requested: usize,
    returned: Option<isize>,
    errno: Option<i32>,
}
impl NativeDiagnosticWrite {
    fn perform(
        bytes: Option<&[u8]>,
        output: &mut impl FnMut(&[u8]) -> (isize, Option<i32>),
    ) -> Self {
        match bytes {
            Some(bytes) => {
                let (returned, errno) = output(bytes);
                Self {
                    attempted: true,
                    requested: bytes.len(),
                    returned: Some(returned),
                    errno,
                }
            }
            None => Self {
                attempted: false,
                requested: 0,
                returned: None,
                errno: None,
            },
        }
    }
    fn complete(self) -> bool {
        self.attempted && self.returned == Some(self.requested as isize) && self.errno.is_none()
    }
}
impl std::fmt::Display for NativeDiagnosticWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"attempted\":{},\"requested\":{},\"returned\":{},\"errno\":{}}}",
            self.attempted,
            self.requested,
            JsonOption(self.returned),
            JsonOption(self.errno)
        )
    }
}
#[derive(Debug)]
struct NativeDiagnosticMain {
    pid: u32,
    row: Option<NativeBirthRow>,
    format_ok: bool,
    output: NativeDiagnosticWrite,
    costs: [NativeDiagnosticCost; 3], // format, one write, owned disposal
}
impl std::fmt::Display for NativeDiagnosticMain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A failed main write can leave a partial JSON fragment. Fence it in
        // this same bounded receipt attempt so the numeric receipt has a line.
        if self.output.attempted && !self.output.complete() {
            f.write_str("\n")?;
        }
        write!(
            f,
            concat!(
                "{{\"event\":\"dream-c1 parent_native_bookkeeping\",\"cutoff\":\"after_birth_write_and_disposal\",",
                "\"receipt_tail\":\"separately_measured_checked_paid; failure_outcome_retains_tail_samples\",",
                "\"pid\":{},\"birth\":{},\"format_ok\":{},\"output\":{},",
                "\"format\":{},\"write\":{},\"dispose\":{}}}"
            ),
            self.pid,
            JsonOption(self.row),
            self.format_ok,
            self.output,
            self.costs[0],
            self.costs[1],
            self.costs[2]
        )
    }
}
#[derive(Debug)]
struct NativeDiagnosticOutcome {
    main: NativeDiagnosticMain,
    receipt_format_ok: bool,
    receipt_output: NativeDiagnosticWrite,
    receipt_costs: [NativeDiagnosticCost; 2], // format, one write
}
impl NativeDiagnosticOutcome {
    fn accepted(&self) -> bool {
        self.main.row.is_some_and(|r| r.settled)
            && self.main.format_ok
            && self.main.output.complete()
            && self.receipt_format_ok
            && self.receipt_output.complete()
            && self
                .main
                .costs
                .iter()
                .chain(&self.receipt_costs)
                .all(|c| c.accepted())
    }
}
impl NativeBirthDiagnostic {
    fn row(
        &self,
        status: Option<std::process::ExitStatus>,
        via: NativeSettlement,
    ) -> Option<NativeBirthRow> {
        use std::os::unix::process::ExitStatusExt;
        Some(NativeBirthRow {
            identity: NativeDiagnosticIdentity::new(self)?,
            cpu_start_ns: self.cpu_start_ns,
            cpu_end_ns: self.cpu_end_ns,
            original: self.original,
            applied_guard: self.applied_guard.get(),
            permit_attempted: self.permit_attempted,
            settled: status.is_some(),
            via,
            exit_code: status.and_then(|s| s.code()),
            exit_signal: status.and_then(|s| s.signal()),
            diagnostic_only: self.diagnostic_only,
        })
    }
    fn emit_with<const N: usize>(
        self,
        status: Option<std::process::ExitStatus>,
        via: NativeSettlement,
        mut clock: impl FnMut() -> u64,
        mut output: impl FnMut(&[u8]) -> (isize, Option<i32>),
        mut pay: impl FnMut(NativeDiagnosticCost, Instant),
    ) -> NativeDiagnosticOutcome {
        // Settlement precedes this entire function. No heap serializer or
        // output occurs inside the original native birth span.
        let pid = self.pid;
        let ((mut line, row, format_ok), format_cost) =
            native_diagnostic_step(&mut clock, &mut pay, || {
                let mut line = NativeDiagnosticLine::<N>::new();
                let row = self.row(status, via);
                let ok = row.is_some_and(|r| line.format(r));
                (line, row, ok)
            });
        let (output_record, write_cost) = native_diagnostic_step(&mut clock, &mut pay, || {
            NativeDiagnosticWrite::perform(format_ok.then(|| line.bytes()), &mut output)
        });
        let (_, dispose_cost) = native_diagnostic_step(&mut clock, &mut pay, || drop(self));
        let main = NativeDiagnosticMain {
            pid,
            row,
            format_ok,
            output: output_record,
            costs: [format_cost, write_cost, dispose_cost],
        };
        // Finite terminal receipt: it reports completed main units, not its own
        // future ending sample. Both receipt units still get measured/checked/
        // paid; any failure returns their exact numeric/unknown samples to the
        // enclosing failing test. No recursive receipt-for-receipt or assertion
        // that the external test-harness error-reporting tail was measured.
        let (receipt_format_ok, receipt_format_cost) =
            native_diagnostic_step(&mut clock, &mut pay, || line.format(&main));
        let (receipt_output, receipt_write_cost) =
            native_diagnostic_step(&mut clock, &mut pay, || {
                NativeDiagnosticWrite::perform(receipt_format_ok.then(|| line.bytes()), &mut output)
            });
        NativeDiagnosticOutcome {
            main,
            receipt_format_ok,
            receipt_output,
            receipt_costs: [receipt_format_cost, receipt_write_cost],
        }
    }
    fn emit(
        self,
        status: Option<std::process::ExitStatus>,
        via: NativeSettlement,
    ) -> NativeDiagnosticOutcome {
        self.emit_with::<NATIVE_DIAGNOSTIC_CAPACITY>(
            status,
            via,
            parent_cpu,
            |bytes| {
                let written = unsafe { libc::write(2, bytes.as_ptr().cast(), bytes.len()) };
                let errno = (written < 0)
                    .then(|| std::io::Error::last_os_error().raw_os_error())
                    .flatten();
                (written, errno)
            },
            |cost, end| {
                if cost.owed_ns.is_some() {
                    // checked debt arithmetic above; no MAX/unknown becomes zero.
                    parent_rest(cost.cpu_ns.unwrap(), end);
                }
            },
        )
    }
}
fn retain_native_diagnostic(
    diagnostic: &mut Option<NativeBirthDiagnostic>,
    status: Option<std::process::ExitStatus>,
    via: NativeSettlement,
) {
    if let Some(record) = diagnostic.take() {
        let outcome = record.emit(status, via);
        if !outcome.accepted() {
            // Only external, already-failing test-harness error reporting is
            // beyond the explicit receipt cutoff. Keep numeric tail failures
            // even during an earlier refusal's unwind; never double-panic.
            if std::thread::panicking() {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "\nnative diagnostic failure external_error_tail_unmeasured {outcome:?}"
                );
            } else {
                panic!("native diagnostic failure external_error_tail_unmeasured {outcome:?}");
            }
        }
    }
}
// Adapted from this project's bootstrap allocator witness. Keep the origin:
// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
struct NativeDiagnosticAllocator;
thread_local! {
    static NATIVE_DIAGNOSTIC_ALLOCATIONS: std::cell::Cell<Option<(usize, usize)>> = const { std::cell::Cell::new(None) };
}
unsafe impl std::alloc::GlobalAlloc for NativeDiagnosticAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let _ = NATIVE_DIAGNOSTIC_ALLOCATIONS.try_with(|c| c.set(c.get().map(|(a, d)| (a + 1, d))));
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: std::alloc::Layout) {
        let _ = NATIVE_DIAGNOSTIC_ALLOCATIONS.try_with(|c| c.set(c.get().map(|(a, d)| (a, d + 1))));
        unsafe { std::alloc::System.dealloc(p, layout) }
    }
    unsafe fn realloc(&self, p: *mut u8, layout: std::alloc::Layout, n: usize) -> *mut u8 {
        let _ = NATIVE_DIAGNOSTIC_ALLOCATIONS.try_with(|c| c.set(c.get().map(|(a, d)| (a + 1, d))));
        unsafe { std::alloc::System.realloc(p, layout, n) }
    }
}
#[global_allocator]
static NATIVE_DIAGNOSTIC_ALLOCATOR: NativeDiagnosticAllocator = NativeDiagnosticAllocator;
fn injected_native_diagnostic() -> NativeBirthDiagnostic {
    NativeBirthDiagnostic {
        pid: 123,
        run: w::Ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV".into()),
        artifact: w::hash(b"injected; never executed"),
        cpu_start_ns: Some(100),
        cpu_end_ns: Some(200),
        original: NativeBirthObservation::new(Some(100), Duration::from_nanos(200)),
        applied_guard: std::cell::Cell::new(None),
        permit_attempted: false,
        diagnostic_only: true,
    }
}
#[test]
fn native_birth_diagnostic_formatter_is_bounded_and_allocation_free() {
    use std::os::unix::process::ExitStatusExt;
    let record = injected_native_diagnostic();
    let mut row = record
        .row(
            Some(std::process::ExitStatus::from_raw(2 << 8)),
            NativeSettlement::WorkerDrop,
        )
        .unwrap();
    // Independent field upper bounds, even where this combination cannot be an
    // actual observation: u32<=10digits, u64<=20, u128<=39, signed status<=11.
    row.identity.pid = u32::MAX;
    row.cpu_start_ns = Some(u64::MAX - 1);
    row.cpu_end_ns = Some(u64::MAX - 1);
    row.original.native_cpu_ns = Some(u64::MAX - 1);
    row.original.native_wall_ns = u128::MAX;
    row.original.cpu_over_budget = Some(true);
    row.original.wall_over_budget = false;
    row.original.unknown_cpu = false;
    row.applied_guard = Some(row.original);
    row.exit_code = Some(i32::MIN);
    row.exit_signal = Some(i32::MIN);
    row.settled = false;
    row.diagnostic_only = false;
    let cost = NativeDiagnosticCost {
        start_ns: Some(u64::MAX - 1),
        end_ns: Some(u64::MAX - 1),
        cpu_ns: Some(u64::MAX - 1),
        owed_ns: Some(u64::MAX - 1),
        over_budget: Some(false),
        repayment_overflow: false,
    };
    let main = NativeDiagnosticMain {
        pid: u32::MAX,
        row: Some(row),
        format_ok: false,
        output: NativeDiagnosticWrite {
            attempted: false,
            requested: usize::MAX,
            returned: Some(isize::MIN),
            errno: Some(i32::MIN),
        },
        costs: [cost; 3],
    };
    let mut line = NativeDiagnosticLine::<NATIVE_DIAGNOSTIC_CAPACITY>::new();
    NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.set(Some((0, 0))));
    let row_ok = line.format(row);
    let row_bytes = line.len;
    let receipt_ok = line.format(&main);
    let receipt_bytes = line.len;
    let counts = NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.replace(None).unwrap());
    assert!(row_ok && receipt_ok);
    assert_eq!(counts, (0, 0));
    assert_eq!(line.bytes().last(), Some(&b'\n'));
    assert!(row_bytes < NATIVE_DIAGNOSTIC_CAPACITY && receipt_bytes < NATIVE_DIAGNOSTIC_CAPACITY);
    eprintln!(
        "native-diagnostic max_birth_bytes={row_bytes} max_receipt_bytes={receipt_bytes} fixed_capacity={NATIVE_DIAGNOSTIC_CAPACITY} allocations=0"
    );
    let mut exact = NativeDiagnosticLine::<4>::new();
    let mut short = NativeDiagnosticLine::<3>::new();
    NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.set(Some((0, 0))));
    assert!(exact.format(123));
    assert!(!short.format(123));
    let counts = NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.replace(None).unwrap());
    assert_eq!(exact.bytes(), b"123\n");
    assert_eq!(counts, (0, 0), "overflow errors must not allocate either");
    NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.set(Some((0, 0))));
    let allocating_control = std::hint::black_box(serde_json::json!({"pid": row.identity.pid}));
    let counts = NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.replace(None).unwrap());
    assert!(
        counts.0 > 0,
        "the former Value-tree path must actually allocate"
    );
    drop(allocating_control);
    let mut invalid = injected_native_diagnostic();
    invalid.run.0 = "\"".repeat(26);
    assert!(invalid.row(None, NativeSettlement::Injected).is_none());
    invalid = injected_native_diagnostic();
    invalid.artifact.0.replace_range(..1, "\"");
    assert!(invalid.row(None, NativeSettlement::Injected).is_none());
}
#[test]
fn native_birth_diagnostic_cost_boundaries_unknown_and_overflow() {
    for (start, end, cpu, over, overflow, accepted) in [
        (100, 100, Some(0), Some(false), false, true),
        (100, 1_000_100, Some(1_000_000), Some(false), false, true),
        (100, 1_000_101, Some(1_000_001), Some(true), false, false),
        (u64::MAX, 100, None, None, false, false),
        (100, u64::MAX, None, None, false, false),
        (100, 99, None, None, false, false),
        (0, u64::MAX - 1, Some(u64::MAX - 1), Some(true), true, false),
    ] {
        let cost = NativeDiagnosticCost::new(start, end);
        assert_eq!(cost.cpu_ns, cpu);
        assert_eq!(cost.over_budget, over);
        assert_eq!(cost.repayment_overflow, overflow);
        assert_eq!(cost.accepted(), accepted);
        assert_eq!(
            cost.owed_ns,
            cpu.and_then(|n| n.max(1_000_000).checked_mul(9))
        );
        let mut line = NativeDiagnosticLine::<512>::new();
        assert!(line.format(cost));
        let json: serde_json::Value = serde_json::from_slice(line.bytes()).unwrap();
        if let Some(cpu) = cpu {
            assert_eq!(json["cpu_ns"].as_u64(), Some(cpu));
        } else {
            assert!(json["cpu_ns"].is_null());
        }
    }
    let unknown = NativeDiagnosticCost::new(u64::MAX, 1);
    assert!(!unknown.accepted() && unknown.cpu_ns.unwrap_or(0) <= 1_000_000);
    let overflow = NativeDiagnosticCost::new(0, u64::MAX - 1);
    assert!(!overflow.accepted() && overflow.owed_ns.is_none());
    assert_ne!(
        9_u64.wrapping_mul(u64::MAX - 1),
        0,
        "wrapping is not valid debt"
    );
}
#[test]
fn native_birth_diagnostic_steps_retain_numeric_effects_and_dispose_before_end() {
    use std::os::unix::process::ExitStatusExt;
    // Pure controller injection: no Child, command, pipe, file or sleep. Every
    // main/tail phase can fail independently without losing its numeric samples.
    for failed_phase in 0..=5 {
        for failure in 0..3 {
            let record = injected_native_diagnostic();
            let mut samples: usize = 0;
            let mut writes = 0;
            let mut paid = [None; 5];
            let mut pay_count = 0;
            let mut receipt = [0; NATIVE_DIAGNOSTIC_CAPACITY];
            let mut receipt_len = 0;
            NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.set(Some((0, 0))));
            let outcome = record.emit_with::<NATIVE_DIAGNOSTIC_CAPACITY>(
                Some(std::process::ExitStatus::from_raw(0)),
                NativeSettlement::Injected,
                || {
                    samples += 1;
                    if samples == 6 {
                        assert!(
                            NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.get().unwrap().1) >= 2,
                            "owned identity strings must be disposed BEFORE ending disposal sample"
                        );
                    }
                    let phase = samples.div_ceil(2);
                    if phase == failed_phase {
                        match failure {
                            0 => return if samples % 2 == 1 { 100 } else { 1_000_101 },
                            1 => return u64::MAX,
                            _ => return if samples % 2 == 1 { 0 } else { u64::MAX - 1 },
                        }
                    }
                    100 + samples as u64
                },
                |bytes| {
                    writes += 1;
                    if writes == 2 {
                        receipt[..bytes.len()].copy_from_slice(bytes);
                        receipt_len = bytes.len();
                    }
                    (bytes.len() as isize, None)
                },
                |cost, _| {
                    paid[pay_count] = cost.owed_ns;
                    pay_count += 1;
                },
            );
            let counts = NATIVE_DIAGNOSTIC_ALLOCATIONS.with(|c| c.replace(None).unwrap());
            assert_eq!(counts.0, 0, "real typed/stack path must not allocate");
            assert_eq!((samples, writes, pay_count), (10, 2, 5));
            assert_eq!(outcome.accepted(), failed_phase == 0);
            let all = [
                outcome.main.costs[0],
                outcome.main.costs[1],
                outcome.main.costs[2],
                outcome.receipt_costs[0],
                outcome.receipt_costs[1],
            ];
            for (index, cost) in all.iter().enumerate() {
                assert_eq!(cost.owed_ns, paid[index]);
                if index + 1 == failed_phase {
                    assert!(!cost.accepted());
                    match failure {
                        0 => assert_eq!(cost.cpu_ns, Some(1_000_001)),
                        1 => assert_eq!(cost.cpu_ns, None),
                        _ => assert!(cost.repayment_overflow),
                    }
                } else {
                    assert!(cost.accepted());
                }
            }
            let json: serde_json::Value = serde_json::from_slice(&receipt[..receipt_len]).unwrap();
            assert_eq!(json["cutoff"], "after_birth_write_and_disposal");
            assert_eq!(json["birth"]["actual_settled"], true);
            assert_eq!(json["birth"]["pid"], 123);
            if (1..=3).contains(&failed_phase) {
                let label = ["format", "write", "dispose"][failed_phase - 1];
                assert_eq!(json[label]["cpu_ns"].as_u64(), all[failed_phase - 1].cpu_ns);
            }
        }
    }
    for (failed_write, returned, errno) in [
        (1, -1, Some(libc::EAGAIN)),
        (1, 1, None),
        (2, -1, Some(libc::EIO)),
        (2, 1, None),
    ] {
        let mut writes = 0;
        let mut emitted = [0; NATIVE_DIAGNOSTIC_CAPACITY * 2];
        let mut emitted_len = 0;
        let outcome = injected_native_diagnostic().emit_with::<NATIVE_DIAGNOSTIC_CAPACITY>(
            Some(std::process::ExitStatus::from_raw(0)),
            NativeSettlement::Injected,
            || 100,
            |bytes| {
                writes += 1;
                let effect = if writes == failed_write {
                    (returned, errno)
                } else {
                    (bytes.len() as isize, None)
                };
                if effect.0 > 0 {
                    let n = usize::try_from(effect.0).unwrap();
                    emitted[emitted_len..emitted_len + n].copy_from_slice(&bytes[..n]);
                    emitted_len += n;
                }
                effect
            },
            |_, _| {},
        );
        assert!(!outcome.accepted());
        assert_eq!(writes, 2, "one attempt per record, no short/error retry");
        let effect = if failed_write == 1 {
            outcome.main.output
        } else {
            outcome.receipt_output
        };
        assert_eq!(effect.returned, Some(returned));
        assert_eq!(effect.errno, errno);
        if failed_write == 1 {
            let final_line = emitted[..emitted_len]
                .rsplit(|b| *b == b'\n')
                .find(|line| !line.is_empty())
                .unwrap();
            let receipt: serde_json::Value = serde_json::from_slice(final_line).unwrap();
            assert_eq!(receipt["event"], "dream-c1 parent_native_bookkeeping");
            assert_eq!(receipt["output"]["returned"], returned);
            assert_eq!(receipt["birth"]["pid"], 123);
        }
    }
    let mut writes = 0;
    let overflow = injected_native_diagnostic().emit_with::<1>(
        Some(std::process::ExitStatus::from_raw(0)),
        NativeSettlement::Injected,
        || 100,
        |_| {
            writes += 1;
            (0, None)
        },
        |_, _| {},
    );
    assert!(!overflow.accepted());
    assert_eq!(writes, 0, "never submit a truncated row");
    assert!(!overflow.main.format_ok && !overflow.receipt_format_ok);
    let unsettled = injected_native_diagnostic().emit_with::<NATIVE_DIAGNOSTIC_CAPACITY>(
        None,
        NativeSettlement::Injected,
        || 100,
        |bytes| (bytes.len() as isize, None),
        |_, _| {},
    );
    assert!(
        !unsettled.accepted(),
        "a synthetic status cannot invent settlement"
    );
}
#[test]
fn native_birth_diagnostic_and_guard_injected_boundaries() {
    use std::os::unix::process::ExitStatusExt;
    let ns = Duration::from_nanos;
    for (cpu, wall, cpu_over, wall_over, unknown, accepted) in [
        (Some(0), 0, Some(false), false, false, true),
        (Some(2_000_000), 25_000_000, Some(false), false, false, true),
        (Some(2_000_001), 25_000_000, Some(true), false, false, false),
        (Some(2_000_000), 25_000_001, Some(false), true, false, false),
        (Some(2_000_001), 25_000_001, Some(true), true, false, false),
        (None, 25_000_000, None, false, true, false),
        (None, 25_000_001, None, true, true, false),
        (Some(u64::MAX), 0, None, false, true, false),
    ] {
        let sample = NativeBirthObservation::new(cpu, ns(wall));
        assert_eq!(sample.cpu_over_budget, cpu_over);
        assert_eq!(sample.wall_over_budget, wall_over);
        assert_eq!(sample.unknown_cpu, unknown);
        assert_eq!(sample.verdict().is_ok(), accepted);
        let record = NativeBirthDiagnostic {
            pid: 123,
            run: w::Ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV".into()),
            artifact: w::hash(b"injected; never executed"),
            cpu_start_ns: Some(100),
            cpu_end_ns: cpu.and_then(|n| n.checked_add(100)),
            original: sample,
            applied_guard: std::cell::Cell::new(Some(sample)),
            permit_attempted: false,
            diagnostic_only: true,
        };
        let typed = record
            .row(
                Some(std::process::ExitStatus::from_raw(2 << 8)),
                NativeSettlement::Injected,
            )
            .unwrap();
        let mut line = NativeDiagnosticLine::<NATIVE_DIAGNOSTIC_CAPACITY>::new();
        assert!(line.format(typed));
        let row: serde_json::Value = serde_json::from_slice(line.bytes()).unwrap();
        assert_eq!(row["pid"], 123);
        assert_eq!(row["run_id"], record.run.0);
        assert_eq!(row["artifact_sha256"], record.artifact.0);
        assert_eq!(row["original"]["native_wall_ns"], wall);
        assert_eq!(row["actual_settled"], true);
        assert_eq!(row["exit_code"], 2);
        assert_eq!(row["diagnostic_only"], true);
        assert_eq!(row["runtime_conformance"], false);
        if unknown {
            assert!(row["original"]["native_cpu_ns"].is_null());
        } else {
            assert_eq!(row["original"]["native_cpu_ns"].as_u64(), cpu);
        }
    }
    for (start, end) in [(u64::MAX, 100), (100, u64::MAX), (100, 99)] {
        let cpu = finite_parent_interval(start, end);
        assert_eq!(cpu, None);
        assert!(NativeBirthObservation::new(cpu, ns(0)).verdict().is_err());
    }
    assert_eq!(finite_parent_interval(100, 100), Some(0));
    assert_eq!(finite_parent_interval(100, 101), Some(1));
    // Causal broken guards really admit the independently excessive/unknown
    // sample that the shared guard rejects. No worker/process is needed.
    let cpu_bad = NativeBirthObservation::new(Some(2_000_001), ns(0));
    let omit_cpu = |s: NativeBirthObservation| s.native_cpu_ns.is_some() && !s.wall_over_budget;
    assert!(cpu_bad.verdict().is_err() && omit_cpu(cpu_bad));
    let wall_bad = NativeBirthObservation::new(Some(0), ns(25_000_001));
    let omit_wall = |s: NativeBirthObservation| s.cpu_over_budget == Some(false);
    assert!(wall_bad.verdict().is_err() && omit_wall(wall_bad));
    let unknown_bad = NativeBirthObservation::new(None, ns(0));
    let unknown_as_zero = |s: NativeBirthObservation| {
        s.native_cpu_ns.unwrap_or(0) <= 2_000_000 && !s.wall_over_budget
    };
    assert!(unknown_bad.verdict().is_err() && unknown_as_zero(unknown_bad));
}
fn birth_interval(start: u64, end: u64) -> Option<u64> {
    if start == u64::MAX || end == u64::MAX {
        None
    } else {
        end.checked_sub(start)
    }
}
fn birth_costs(start: u64, call_start: u64, call_end: u64, end: u64) -> [Option<u64>; 3] {
    [
        birth_interval(call_start, call_end),
        birth_interval(start, call_start)
            .and_then(|before| birth_interval(call_end, end)?.checked_add(before)),
        birth_interval(start, end),
    ]
}
fn birth_verdict(
    native: Option<u64>,
    wall: Duration,
    application: Option<u64>,
    inclusive: Option<u64>,
) -> Result<u64, &'static str> {
    let native = native.ok_or("unknown native birth cost")?;
    let application = application.ok_or("unknown birth application cost")?;
    let inclusive = inclusive.ok_or("unknown inclusive birth cost")?;
    if native > 2_000_000 {
        return Err("native birth CPU allocation");
    }
    if wall > Duration::from_millis(25) {
        return Err("native birth wall allocation");
    }
    if application > 1_000_000 {
        return Err("birth application CPU allocation");
    }
    if inclusive > 3_000_000 {
        return Err("inclusive birth CPU allocation");
    }
    if native.checked_add(application) != Some(inclusive) {
        return Err("inconsistent birth CPU intervals");
    }
    Ok(inclusive)
}
fn birth_debt(inclusive: Option<u64>) -> Duration {
    // Unknown cost remains a refusal, with finite mandatory settlement debt.
    // Duration multiplication also retains a known overrun without u64-ns wrap.
    Duration::from_nanos(inclusive.unwrap_or(3_000_000).max(1_000_000)) * 9
}
fn birth_readiness(sample: Option<u64>, lag: Option<Duration>) -> Option<u64> {
    // The monotonic sample follows the native ending clock. Subtract the full
    // wall bracket from actual spawn return through that sample's end, so the
    // ending-clock probes cannot grant extra time beyond the original250ms.
    sample?
        .checked_sub(u64::try_from(lag?.as_nanos()).ok()?)?
        .checked_add(boot::WINDOW_NS)
}
#[derive(Debug, PartialEq, Eq)]
enum HelloReadinessFailure {
    Clock,
    ReversedClock,
    Expired(u64),
    Native(i32),
    Interrupted,
    Events(i16),
}
fn hello_readiness(
    deadline: u64,
    mut clock: impl FnMut() -> Result<u64, ()>,
    mut poll: impl FnMut(i32) -> Result<i16, i32>,
) -> Result<(), HelloReadinessFailure> {
    let mut previous = 0;
    // An interrupted observation is not a readiness timeout. Retry only the
    // poll, at most four times, against the SAME original absolute deadline.
    // There is no second child, permit, grant or new readiness window.
    for _ in 0..=4 {
        let now = clock().map_err(|_| HelloReadinessFailure::Clock)?;
        if now < previous {
            return Err(HelloReadinessFailure::ReversedClock);
        }
        if now >= deadline {
            return Err(HelloReadinessFailure::Expired(now));
        }
        let result = poll((deadline - now).div_ceil(1_000_000).min(i32::MAX as u64) as i32);
        let after = clock().map_err(|_| HelloReadinessFailure::Clock)?;
        if after < now {
            return Err(HelloReadinessFailure::ReversedClock);
        }
        if after >= deadline {
            return Err(HelloReadinessFailure::Expired(after));
        }
        previous = after;
        match result {
            Ok(events) if events & libc::POLLIN != 0 => return Ok(()),
            Ok(0) => return Err(HelloReadinessFailure::Expired(after)),
            Ok(events) => return Err(HelloReadinessFailure::Events(events)),
            Err(libc::EINTR) => (),
            Err(errno) => return Err(HelloReadinessFailure::Native(errno)),
        }
    }
    Err(HelloReadinessFailure::Interrupted)
}
#[test]
fn interrupted_hello_observation_keeps_original_deadline_and_failures() {
    let mut clock = [0, 1_000_000, 2_000_000, 3_000_000].into_iter();
    let mut timeouts = Vec::new();
    let mut results = [Err(libc::EINTR), Ok(libc::POLLIN)].into_iter();
    assert_eq!(
        hello_readiness(
            4_000_000,
            || Ok(clock.next().unwrap()),
            |timeout| {
                timeouts.push(timeout);
                results.next().unwrap()
            },
        ),
        Ok(())
    );
    assert_eq!(timeouts, [4, 2]);
    for events in [Ok(libc::POLLIN), Err(libc::EINTR)] {
        let mut clock = [0, 4_000_000].into_iter();
        assert_eq!(
            hello_readiness(4_000_000, || Ok(clock.next().unwrap()), |_| events),
            Err(HelloReadinessFailure::Expired(4_000_000))
        );
    }
    assert_eq!(
        hello_readiness(4_000_000, || Ok(1), |_| Err(libc::EINTR)),
        Err(HelloReadinessFailure::Interrupted)
    );
    assert_eq!(
        hello_readiness(4_000_000, || Ok(1), |_| Err(libc::EBADF)),
        Err(HelloReadinessFailure::Native(libc::EBADF))
    );
    assert_eq!(
        hello_readiness(4_000_000, || Ok(1), |_| Ok(libc::POLLHUP)),
        Err(HelloReadinessFailure::Events(libc::POLLHUP))
    );
    assert_eq!(
        hello_readiness(4_000_000, || Err(()), |_| panic!("unknown clock polled")),
        Err(HelloReadinessFailure::Clock)
    );
    let mut clock = [2, 1].into_iter();
    assert_eq!(
        hello_readiness(
            4_000_000,
            || Ok(clock.next().unwrap()),
            |_| Ok(libc::POLLIN)
        ),
        Err(HelloReadinessFailure::ReversedClock)
    );
}
#[test]
fn unavailable_or_reversed_parent_samples_never_become_zero_cost() {
    for (start, end) in [(u64::MAX, u64::MAX), (100, u64::MAX), (100, 99)] {
        assert!(std::panic::catch_unwind(|| parent_interval(start, end)).is_err());
    }
    assert_eq!(parent_interval(100, 101), 1);
}
#[test]
fn fixture_birth_separates_native_application_and_full_inclusive_cost() {
    // Real fixture verdict path: ordinary readiness work can legitimately take
    // total CPU above2ms while exact native CPU remains within its own bound.
    let [native, application, inclusive] = birth_costs(0, 100_000, 2_000_000, 2_100_000);
    assert_eq!(
        [native, application, inclusive],
        [Some(1_900_000), Some(200_000), Some(2_100_000)]
    );
    assert_eq!(
        birth_verdict(native, Duration::from_millis(25), application, inclusive),
        Ok(2_100_000)
    );
    assert_eq!(birth_debt(inclusive), Duration::from_nanos(18_900_000));
    assert_eq!(
        birth_verdict(
            Some(2_000_000),
            Duration::from_millis(25),
            Some(1_000_000),
            Some(3_000_000)
        ),
        Ok(3_000_000)
    );
    for (native, wall, application, inclusive, expected) in [
        (
            Some(2_000_001),
            Duration::ZERO,
            Some(0),
            Some(2_000_001),
            "native birth CPU allocation",
        ),
        (
            Some(1),
            Duration::from_nanos(25_000_001),
            Some(0),
            Some(1),
            "native birth wall allocation",
        ),
        (
            Some(1),
            Duration::ZERO,
            Some(1_000_001),
            Some(1_000_002),
            "birth application CPU allocation",
        ),
        (
            Some(2_000_000),
            Duration::ZERO,
            Some(1_000_000),
            Some(3_000_001),
            "inclusive birth CPU allocation",
        ),
        (
            Some(1),
            Duration::ZERO,
            Some(1),
            Some(3),
            "inconsistent birth CPU intervals",
        ),
        (
            None,
            Duration::ZERO,
            Some(1),
            Some(2),
            "unknown native birth cost",
        ),
        (
            Some(1),
            Duration::ZERO,
            None,
            Some(2),
            "unknown birth application cost",
        ),
        (
            Some(1),
            Duration::ZERO,
            Some(1),
            None,
            "unknown inclusive birth cost",
        ),
    ] {
        assert_eq!(
            birth_verdict(native, wall, application, inclusive),
            Err(expected)
        );
    }
    assert_eq!(birth_costs(0, 100, 99, 200)[0], None);
    assert_eq!(birth_costs(0, 100, u64::MAX, 200)[0], None);
    assert_eq!(birth_costs(0, 100, 150, u64::MAX)[1..], [None, None]);
    assert_eq!(birth_debt(None), Duration::from_millis(27));
    assert_eq!(birth_debt(Some(4_000_000)), Duration::from_millis(36));
    assert_eq!(birth_debt(Some(0)), Duration::from_millis(9));
}
#[test]
fn fixture_birth_readiness_does_not_add_ending_probe_time_to_original_window() {
    assert_eq!(
        birth_readiness(Some(1_000_000), Some(Duration::from_nanos(30_000))),
        Some(970_000 + boot::WINDOW_NS)
    );
    assert_eq!(birth_readiness(None, Some(Duration::ZERO)), None);
    assert_eq!(
        birth_readiness(Some(1), Some(Duration::from_nanos(2))),
        None
    );
    assert_eq!(birth_readiness(Some(u64::MAX), Some(Duration::ZERO)), None);
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
fn entry_verdict(trace: &str, observed_ns: u64) -> Result<[u64; 8], &'static str> {
    let mut lines = trace.split_inclusive('\n');
    let line = lines.next().ok_or("missing entry")?;
    if line.len() > 256 || !line.ends_with('\n') {
        return Err("incomplete or oversized entry");
    }
    let mut fields = line.trim_end_matches('\n').split(' ');
    if fields.next() != Some("dream-bootstrap") || fields.next() != Some("entry") {
        return Err("entry must precede waiting and permit");
    }
    let mut values = [0; 8];
    for (i, key) in ["s=", "g=", "e=", "m=", "u=", "k=", "r=", "p=", "ok="]
        .iter()
        .enumerate()
    {
        let text = fields
            .next()
            .and_then(|f| f.strip_prefix(key))
            .ok_or("entry key order")?;
        if text.is_empty()
            || text.len() > 20
            || (text.len() > 1 && text.starts_with('0'))
            || !text.bytes().all(|b| b.is_ascii_digit())
        {
            return Err("entry numeric field");
        }
        let value = text.parse::<u64>().map_err(|_| "entry u64 overflow")?;
        if i < 8 {
            values[i] = value;
        } else if value > 1 || (value == 0 && [0, 4, 5, 6, 7].iter().any(|i| values[*i] != 0)) {
            return Err("entry rusage status");
        }
    }
    if fields.next().is_some() || values[2] > values[3] || values[3] > observed_ns {
        return Err("entry fields or monotonic clock chain");
    }
    if !lines
        .next()
        .is_some_and(|l| l.starts_with("dream-bootstrap state=waiting ") && l.ends_with('\n'))
        || lines.next().is_some()
    {
        return Err("exactly one entry then waiting required");
    }
    // e may precede the parent's spawn return. No unsigned subtraction or raw
    // s/g/u/k/r/p conversion/classification is meaningful here.
    Ok(values)
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
    birth_diagnostic: Option<NativeBirthDiagnostic>,
    application_cpu: Option<u64>,
    inclusive_cpu: Option<u64>,
    inclusive_wall: Duration,
    birth_debt_paid: bool,
    runner_hash: [u8; 64],
    marker: String,
    entry_trace: String,
    entry_observed_ns: u64,
    expected_schema: w::Hex,
    prepared: Option<Command>,
    frozen_binding: [u8; 32],
    job_deadline: Instant,
}
impl HeldWorker {
    fn spawn(path: &Path, nonce: &w::Ulid, context: Option<&w::Context>, probe: bool) -> Self {
        let (runner, worker_hash) = approved_runner();
        Self::spawn_on(path, nonce, context, probe, runner, worker_hash, "")
    }
    #[allow(clippy::too_many_arguments)]
    fn spawn_on(
        path: &Path,
        nonce: &w::Ulid,
        context: Option<&w::Context>,
        probe: bool,
        runner: &Path,
        worker_hash: &w::Hex,
        bypass: &str,
    ) -> Self {
        use std::os::unix::process::CommandExt;
        let job_deadline = Instant::now() + Duration::from_secs(30);
        let prepare_at = parent_cpu();
        let mut runner_hash = [0u8; 64];
        runner_hash.copy_from_slice(worker_hash.0.as_bytes());
        let expected_schema = w::schema_hash();
        let diagnostic_only =
            std::env::var_os("DREAM_C1_DIAGNOSTIC_ONLY").is_some_and(|v| v == "1");
        if let Some(expected) = std::env::var_os("DREAM_C1_EXPECTED_WORKER_SHA256") {
            assert_eq!(
                expected,
                worker_hash.0.as_str(),
                "diagnostic artifact changed before birth"
            );
        }
        let artifact = worker_hash.clone();
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
        if !bypass.is_empty() {
            command.env("BBOXY_DREAM_BOOTSTRAP_TEST_BYPASS", bypass);
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
        let span_at = parent_cpu();
        assert_ne!(
            span_at,
            u64::MAX,
            "missing span clock must not create child"
        );
        let native_at = parent_cpu();
        assert_ne!(
            native_at,
            u64::MAX,
            "missing entry clock must not create child"
        );
        let wall_at = Instant::now();
        // Retain actual Child before evaluating the ending native clock/verdict.
        let spawned = command.spawn();
        let spawn_return = Instant::now();
        let end_cpu = parent_cpu();
        let native_wall = spawn_return.duration_since(wall_at);
        let native_cpu = finite_parent_interval(native_at, end_cpu);
        // Readiness and its full wall bracket are ordinary work after native
        // return; the original250ms window never gains ending-clock time.
        let return_sample = boot::monotonic_ns().ok();
        let sample_end = Instant::now();
        let readiness = birth_readiness(
            return_sample,
            sample_end.checked_duration_since(spawn_return),
        );
        // A failed spawn also retains ending samples before formatting. No
        // returned Child means effect/settlement are unknown, never absence.
        let child = spawned.unwrap_or_else(|error| {
            panic!(
                "native spawn returned no Child run={} artifact={} samples={:?} effect=unknown settlement=unknown error={error:?}",
                run.0,
                artifact.0,
                NativeBirthObservation::new(native_cpu, native_wall)
            )
        });
        let pid = child.id();
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
            birth_end: spawn_return,
            launched,
            readiness_ns,
            native_cpu,
            native_wall,
            birth_diagnostic: Some(NativeBirthDiagnostic {
                pid,
                run,
                artifact,
                cpu_start_ns: (native_at != u64::MAX).then_some(native_at),
                cpu_end_ns: (end_cpu != u64::MAX).then_some(end_cpu),
                original: NativeBirthObservation::new(native_cpu, native_wall),
                applied_guard: std::cell::Cell::new(None),
                permit_attempted: false,
                diagnostic_only,
            }),
            application_cpu: None,
            inclusive_cpu: None,
            inclusive_wall: Duration::ZERO,
            birth_debt_paid: false,
            runner_hash,
            marker: String::new(),
            entry_trace: String::new(),
            entry_observed_ns: 0,
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
        if !bypass.is_empty() {
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
        // The ordinary return half includes retained-record assembly and the
        // original inherited-read FD close, before any marker wait or verdict.
        let span_end = parent_cpu();
        let birth_end = Instant::now();
        let [native_cpu, application_cpu, inclusive_cpu] =
            birth_costs(span_at, native_at, end_cpu, span_end);
        result.native_cpu = native_cpu;
        result.application_cpu = application_cpu;
        result.inclusive_cpu = inclusive_cpu;
        result.inclusive_wall = birth_end.duration_since(launched);
        result.birth_end = birth_end;
        // The actual child/write lease is now retained even if readiness clock
        // capture fails. No timing verdict can lose the returned process.
        result.readiness_ns = match readiness {
            Some(deadline) => deadline,
            None => {
                result.birth_refusal("spawn-return readiness unavailable");
                panic!("spawn-return readiness unavailable");
            }
        };
        result.permit.deadline_ns = result.readiness_ns;
        // Retained-record assembly and the inherited-FD close are included
        // in application_cpu/inclusive_cpu above. The single original birth
        // debt is paid by pay_birth_debt; do not overlap a second retention debt.
        if bypass.is_empty() {
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
        let mut bytes = [0u8; 512];
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
            if bytes[..n].iter().filter(|b| **b == b'\n').count() >= 2 {
                break;
            }
        }
        self.entry_observed_ns = boot::monotonic_ns().unwrap();
        self.entry_trace = String::from_utf8(bytes[..n].to_vec()).unwrap();
        self.entry_trace.lines().last().unwrap_or("").to_owned() + "\n"
    }
    fn waiting_verdict(&self) -> Result<(), &'static str> {
        entry_verdict(&self.entry_trace, self.entry_observed_ns)?;
        if self.marker.trim()
            == "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0"
        {
            Ok(())
        } else {
            Err("controlled bootstrap span preceded permit")
        }
    }
    fn launch_verdict(&self) -> Result<u64, &'static str> {
        let observed = NativeBirthObservation::new(self.native_cpu, self.native_wall);
        if let Some(record) = &self.birth_diagnostic {
            record.applied_guard.set(Some(observed));
        }
        birth_verdict(
            self.native_cpu,
            self.native_wall,
            self.application_cpu,
            self.inclusive_cpu,
        )
    }
    fn pay_birth_debt(&mut self) {
        if std::mem::replace(&mut self.birth_debt_paid, true) {
            return;
        }
        if let Some(left) = birth_debt(self.inclusive_cpu).checked_sub(self.birth_end.elapsed()) {
            std::thread::sleep(left);
        }
    }
    fn birth_refusal(&self, reason: &'static str) {
        // One bounded record: fixed labels, numeric fields, a static reason and
        // exactly64 public hash bytes. No argv, permit, context or source data.
        eprintln!(
            "dream-c1 parent_birth_refusal reason={reason:?} pid={:?} native_cpu_ns={:?} native_wall_ns={} application_cpu_ns={:?} inclusive_cpu_ns={:?} inclusive_wall_ns={} runner_sha256={} grant_count=0 retry_count=0",
            self.child.as_ref().map(Child::id),
            self.native_cpu,
            self.native_wall.as_nanos(),
            self.application_cpu,
            self.inclusive_cpu,
            self.inclusive_wall.as_nanos(),
            std::str::from_utf8(&self.runner_hash).unwrap()
        );
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
        if let Err(reason) = self.launch_verdict() {
            self.birth_refusal(reason);
            panic!("native birth verdict refused grant: {reason}");
        }
        self.pay_birth_debt();
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
            birth_diagnostic: self.birth_diagnostic.take(),
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
        if let Some(record) = &mut worker.birth_diagnostic {
            record.permit_attempted = true;
        }
        let wrote = unsafe {
            libc::write(
                worker.bootstrap_lease.as_ref().unwrap().as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len(),
            )
        };
        assert_eq!(wrote, 128);
        assert!(parent_interval(grant_at, parent_cpu()) <= 1_000_000);
        let mut p = libc::pollfd {
            fd: worker.output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let mut last_poll = None;
        let hello_observation = hello_readiness(
            self.readiness_ns,
            || boot::monotonic_ns().map_err(|_| ()),
            |timeout| {
                p.revents = 0;
                let result = unsafe { libc::poll(&mut p, 1, timeout) };
                let errno = if result < 0 {
                    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
                } else {
                    0
                };
                last_poll = Some((result, p.revents, errno));
                if result < 0 {
                    Err(errno)
                } else {
                    Ok(p.revents)
                }
            },
        );
        assert!(
            hello_observation.is_ok(),
            "Hello readiness refused observation={hello_observation:?} raw_poll_result_events_errno={last_poll:?} pid={} original_deadline_ns={} observed_ns={:?} child_before_lease_revoke={:?}",
            worker.child.id(),
            self.readiness_ns,
            boot::monotonic_ns(),
            worker.child.try_wait(),
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
        }
        // Revoke the lease before paying failure debt; keep the exact Child
        // pinned through that rest and the existing escalation/reap path.
        self.pay_birth_debt();
        if let Some(child) = self.child.as_mut() {
            settle_owned(child);
            let status = child.try_wait().ok().flatten();
            retain_native_diagnostic(
                &mut self.birth_diagnostic,
                status,
                NativeSettlement::HeldDrop,
            );
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
    birth_diagnostic: Option<NativeBirthDiagnostic>,
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
                let status = std::process::ExitStatus::from_raw(status);
                retain_native_diagnostic(
                    &mut self.birth_diagnostic,
                    Some(status),
                    NativeSettlement::ExactPidWait4,
                );
                return (status, unsafe { usage.assume_init() });
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
            let status = self.child.try_wait().ok().flatten();
            retain_native_diagnostic(
                &mut self.birth_diagnostic,
                status,
                NativeSettlement::WorkerDrop,
            );
        }
    }
}
fn complete(job: &Job) -> (w::ResultDescriptor, Vec<w::Row>, String) {
    let mut worker = job.launch();
    worker.start(&job.start);
    complete_worker(job, &mut worker)
}
fn complete_worker(job: &Job, worker: &mut Worker) -> (w::ResultDescriptor, Vec<w::Row>, String) {
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
        let mut worker = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job.launch()))
            .unwrap_or_else(|failure| {
                eprintln!(
                    "dream-c1 source_control_launch_refused mode={mode} expected={expected:?}"
                );
                std::panic::resume_unwind(failure)
            });
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
            if let w::WorkerOut::Progress(p) = worker.read()
                && p.phase == w::Phase::Paused
            {
                break;
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
        if let w::WorkerOut::Progress(p) = worker.read()
            && p.phase == w::Phase::Paused
        {
            break;
        }
    }
    assert!(at.elapsed() < Duration::from_millis(100));
    let second = Job::new(0, 0, false, false);
    let at = Instant::now();
    let mut independent = second.launch();
    independent.start(&second.start);
    let phase = match independent.read() {
        w::WorkerOut::Progress(progress) => progress.phase,
        other => panic!("independent worker did not enter work: {other:?}"),
    };
    assert_ne!(phase, w::Phase::Paused);
    let progress_elapsed = at.elapsed();
    assert!(
        progress_elapsed < Duration::from_secs(2),
        "paused root held another worker"
    );
    // Admission/work progress is the independent-owner boundary. Completion
    // retains its original product deadline and all CPU, paid debt, publication
    // and custody checks; deliberate paid rest is not a blocked owner.
    let (_, _, metrics) = complete_worker(&second, &mut independent);
    eprintln!(
        "paused-independent progress_wall_ns={} phase={phase:?} {metrics}",
        progress_elapsed.as_nanos()
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
        if let w::WorkerOut::Progress(p) = killed.read()
            && p.phase == w::Phase::Validate
        {
            break;
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
    // A separately owned paused worker supplies fresh control roundtrips
    // throughout the real rejection, paced cleanup and reap below. Deliberate
    // paid rest belongs to the unchanged product deadline, not the 2s bound
    // on independent control progress.
    let mut control_job = Job::new(0, 0, false, false);
    control_job.start.foreground = true;
    let mut independent = control_job.launch();
    independent.start(&control_job.start);
    assert!(matches!(independent.read(), w::WorkerOut::Progress(p) if p.phase == w::Phase::Paused));
    let mut worker = job.launch();
    let at = Instant::now();
    worker.start(&job.start);
    use std::sync::atomic::{AtomicBool, Ordering};
    let finished = AtomicBool::new(false);
    let (done, metrics, roundtrips, maximum) = std::thread::scope(|scope| {
        let controls = scope.spawn(|| {
            let mut revision = 2;
            let mut roundtrips = 0;
            let mut maximum = Duration::ZERO;
            while !finished.load(Ordering::Acquire) {
                let elapsed = independent
                    .timed_paused_activity(revision, Duration::from_secs(2))
                    .expect("independent control stalled during paced rejection/cleanup");
                maximum = maximum.max(elapsed);
                roundtrips += 1;
                revision += 1;
                std::thread::sleep(Duration::from_millis(10));
            }
            independent.cancel();
            let (done, metrics) = independent.done();
            assert_eq!(done.outcome, w::Outcome::Cancelled, "{metrics}");
            (roundtrips, maximum)
        });
        // Also retire/join the independently owned controller if a native
        // assertion fails; no detached test thread or child survives unwinding.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker.done()));
        finished.store(true, Ordering::Release);
        let (roundtrips, maximum) = controls.join().unwrap();
        let (done, metrics) = result.unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        (done, metrics, roundtrips, maximum)
    });
    assert!(roundtrips > 0);
    assert!(maximum < Duration::from_secs(2));
    assert!(
        Instant::now() < worker.job_deadline,
        "original 30s Done/reap deadline exceeded: {metrics}"
    );
    eprintln!(
        "near-limit-cleanup independent_roundtrips={roundtrips} max_control_ns={}",
        maximum.as_nanos()
    );
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert!(matches!(
        done.error.as_ref().unwrap().code,
        w::Code::UnsupportedIr | w::Code::ResourceLimit
    ));
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
        if let w::WorkerOut::Progress(p) = worker.read()
            && p.envelopes.0 >= 100
        {
            break;
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
    let last = job
        .bytes
        .split_inclusive(|b| *b == b'\n')
        .next_back()
        .unwrap();
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
    let mut held = HeldWorker::spawn_on(&path, &nonce, None, true, &binary, &digest, "1");
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
    // This negative was collected through actual reap rather than positive
    // readiness. Validate its real entry evidence first, so a missing entry
    // cannot accidentally stand in for rejection of the actual allocation.
    held.entry_trace = held.marker.clone();
    held.entry_observed_ns = boot::monotonic_ns().unwrap();
    entry_verdict(&held.entry_trace, held.entry_observed_ns).unwrap();
    held.marker = held.entry_trace.lines().last().unwrap().to_owned() + "\n";
    assert_eq!(
        held.waiting_verdict(),
        Err("controlled bootstrap span preceded permit"),
        "the checker must reject the allocation itself"
    );
    let pid = held.child.as_ref().unwrap().id();
    assert!(!path.exists());
    eprintln!(
        "bootstrap-earliest-causal-control actual_reaped_pid={pid} rejected_prepermit_allocation=true"
    );
}

// This private test-process calls the real Gate, never a term worker readiness
// probe. The existing Make fixture-name filter executes its causal mutants too.
#[test]
fn bootstrap_earliest_gate_bypass_control_entry_process() {
    use std::io::Read;
    if let Ok(config) = std::env::var("DREAM_ENTRY_PRIVATE_CHILD") {
        let mut parts = config.split(':');
        let fd: i32 = parts.next().unwrap().parse().unwrap();
        let burn: u64 = parts.next().unwrap().parse().unwrap();
        let e = boot::monotonic_ns().unwrap();
        let prelude = boot::prelude_clock().unwrap();
        // Deliberately broken prelude CPU, in this test executable only. A full
        // observe must refuse; skipping it causally admits the same real work.
        while boot::prelude_clock().unwrap().checked_sub(prelude).unwrap() < burn {
            std::hint::spin_loop();
        }
        let t = boot::Token {
            fd,
            birth: [0; 16],
            binding: [0; 32],
        };
        let result = unsafe {
            boot::Gate::wait(
                t,
                boot::Purpose::InspectC1,
                b"00000000000000000000000000",
                e,
                prelude,
            )
        };
        match result {
            Ok(gate) => println!(
                "entry-result admitted cpu={} prepermit={} max={} reads={}",
                gate.usage.cpu_ns,
                gate.usage.prepermit_cpu_ns,
                gate.usage.max_cpu_ns,
                gate.usage.reads
            ),
            Err(error) => println!("entry-result refused {error:?}"),
        }
        return;
    }
    let mut cases = vec![
        ("ordinary", "", 0u64),
        ("eof", "", 0),
        ("pipe", "", 0),
        ("overbudget", "", 2_000_000),
    ];
    if cfg!(feature = "test-fixtures") {
        cases.extend([
            ("late", "entry_after_permit", 0),
            ("duplicate", "entry_duplicate", 0),
            ("noobserve", "entry_noobserve", 2_000_000),
        ]);
    }
    let binary = std::env::current_exe().unwrap();
    eprintln!(
        "entry-private-process profile=dream-bootstrap/1 test_executable=true artifact={} sha256={} features_test_fixtures={} source_bootstrap_sha256={} source_test_sha256={}",
        binary.display(),
        w::hash(std::fs::read(&binary).unwrap()).0,
        cfg!(feature = "test-fixtures"),
        w::hash(include_bytes!("../src/dream/bootstrap.rs")).0,
        w::hash(include_bytes!("dream_c1.rs")).0,
    );
    for (case, fixture, burn) in cases {
        use std::os::unix::process::CommandExt;
        let (read, write) = pipe_files();
        let fd = read.as_raw_fd();
        let mut command = Command::new(&binary);
        command
            .args([
                "--exact",
                "bootstrap_earliest_gate_bypass_control_entry_process",
                "--nocapture",
                "--test-threads=1",
            ])
            .env_clear()
            .env("DREAM_ENTRY_PRIVATE_CHILD", format!("{fd}:{burn}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if case == "pipe" {
            command.stderr(Stdio::null());
        }
        if !fixture.is_empty() {
            command.env("BBOXY_DREAM_BOOTSTRAP_TEST_BYPASS", fixture);
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
        struct OwnedEntryChild(Child);
        impl Drop for OwnedEntryChild {
            fn drop(&mut self) {
                settle_owned(&mut self.0);
            }
        }
        let mut owned = OwnedEntryChild(command.spawn().unwrap());
        drop(read);
        if case == "pipe" {
            settle_owned(&mut owned.0);
            assert!(owned.0.wait().unwrap().success());
            let mut output = String::new();
            owned
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut output)
                .unwrap();
            assert!(output.contains("entry-result refused Pipe"), "{output}");
            eprintln!(
                "entry-private-process case=pipe actual_reap_pid={} result={output:?}",
                owned.0.id()
            );
            drop(write);
            continue;
        }
        let stderr = owned.0.stderr.as_ref().unwrap().as_raw_fd();
        let until = Instant::now() + Duration::from_secs(2);
        let mut trace = Vec::new();
        while !trace
            .windows(b"owner_admissions=0\n".len())
            .any(|w| w == b"owner_admissions=0\n")
        {
            assert!(
                Instant::now() < until,
                "{case}: missing actual waiting {trace:?}"
            );
            let mut poll = libc::pollfd {
                fd: stderr,
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 10) } > 0 {
                let mut bytes = [0u8; 512];
                let n = unsafe { libc::read(stderr, bytes.as_mut_ptr().cast(), bytes.len()) };
                assert!(n > 0, "{case}: entry/waiting pipe closed {trace:?}");
                trace.extend_from_slice(&bytes[..n as usize]);
                assert!(trace.len() <= 1024);
            }
        }
        let observed = boot::monotonic_ns().unwrap();
        let before_permit = String::from_utf8(trace.clone()).unwrap();
        if case == "late" || case == "duplicate" {
            assert!(
                entry_verdict(&before_permit, observed).is_err(),
                "causal entry mutant accepted before permit"
            );
        } else {
            let values = entry_verdict(&before_permit, observed).unwrap();
            assert!(values[2] <= values[3] && values[3] <= observed);
        }
        if case != "eof" && case != "overbudget" {
            let permit = boot::Permit {
                purpose: boot::Purpose::InspectC1,
                probe: true,
                birth: [0; 16],
                run_id: [0; 16],
                nonce: [0; 16],
                binding: [0; 32],
                incarnation: 1,
                sequence: 1,
                deadline_ns: boot::monotonic_ns().unwrap() + boot::WINDOW_NS,
            };
            let bytes = boot::encode(&permit);
            assert_eq!(
                unsafe { libc::write(write.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) },
                128
            );
            // Retain the real lease until the Gate has returned; no EOF-induced
            // refusal can masquerade as the noobserve control's admission.
            while owned.0.try_wait().unwrap().is_none() && Instant::now() < until {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        drop(write);
        settle_owned(&mut owned.0);
        assert!(owned.0.wait().unwrap().success());
        owned
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_end(&mut trace)
            .unwrap();
        let trace = String::from_utf8(trace).unwrap();
        let mut output = String::new();
        owned
            .0
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();
        if case == "eof" {
            assert!(output.contains("entry-result refused Revoked"), "{output}");
        } else if case == "overbudget" {
            assert!(output.contains("entry-result refused Limit"), "{output}");
        } else {
            assert!(
                output.contains("entry-result admitted"),
                "{case}: {output} {trace}"
            );
            if case == "ordinary" {
                let receipt = output
                    .lines()
                    .find(|l| l.contains("entry-result admitted"))
                    .unwrap();
                assert!(metric(receipt, "prepermit") > 0);
                assert!(metric(receipt, "max") <= 1_000_000);
                assert!(metric(receipt, "cpu") >= metric(receipt, "prepermit"));
            }
        }
        if case == "late" {
            let waiting_at = trace.find("state=waiting").unwrap();
            let entry_at = trace.find("dream-bootstrap entry ").unwrap();
            assert!(
                entry_at > waiting_at,
                "late control must really emit after the read"
            );
            assert!(entry_verdict(&trace, boot::monotonic_ns().unwrap()).is_err());
        }
        if case == "duplicate" {
            assert_eq!(trace.matches("dream-bootstrap entry ").count(), 2);
        }
        if case == "noobserve" {
            // Same >1ms actual prelude refused above is incorrectly admitted by
            // this mutant. This assertion records its intended rejection.
            let receipt = output
                .lines()
                .find(|l| l.contains("entry-result admitted"))
                .unwrap();
            assert!(burn > 1_000_000 && metric(receipt, "max") <= 1_000_000);
            assert!(
                metric(receipt, "prepermit") < burn,
                "mutant must actually omit the burnt CPU"
            );
        }
        eprintln!(
            "entry-private-process case={case} actual_reap_pid={} before_permit_bytes={} trace={trace:?} result={output:?}",
            owned.0.id(),
            before_permit.len()
        );
    }
}
