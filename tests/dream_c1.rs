// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Canonical transport corpus, independent hashes and private worker witnesses.
use bboxy450_harness::dream::c1_wire as w;
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
    os::unix::fs::PermissionsExt,
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
        Worker::launch(&self.path)
    }
}
struct Worker {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    start: Option<w::Start>,
}
impl Worker {
    fn launch(path: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .args(["dream", "inspect-c1", "--job-dir"])
            .arg(path)
            .env_clear()
            .env("LANG", "C.UTF-8")
            .env("TMPDIR", path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut w = Self {
            child,
            input,
            output,
            start: None,
        };
        match w.read() {
            w::WorkerOut::Hello(h) => {
                assert_eq!(h.schema_sha256, w::schema_hash());
                assert_eq!(h.ir_major, 1);
                assert_eq!(h.ir_minor_max, 3);
            }
            other => panic!("{other:?}"),
        };
        w
    }
    fn read(&mut self) -> w::WorkerOut {
        let mut b = Vec::new();
        self.output.read_until(b'\n', &mut b).unwrap();
        assert!(
            !b.is_empty(),
            "worker unexpectedly exited {:?}",
            self.child.try_wait()
        );
        w::decode(&b).unwrap()
    }
    fn send(&mut self, message: &w::WorkerIn) {
        let mut b = w::canonical(message).unwrap();
        b.push(b'\n');
        self.input.as_mut().unwrap().write_all(&b).unwrap();
    }
    fn start(&mut self, s: &w::Start) {
        self.send(&w::WorkerIn::Start(s.clone()));
        self.start = Some(s.clone());
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
    fn done(&mut self) -> (w::Done, String) {
        let mut seq = 0;
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
        let until = Instant::now() + Duration::from_secs(3);
        let status = loop {
            if let Some(s) = self.child.try_wait().unwrap() {
                break s;
            }
            assert!(Instant::now() < until, "worker did not settle");
            std::thread::sleep(Duration::from_millis(2));
        };
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
        (done, metrics)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
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
    assert!(result.max_poll_cpu_ns.0 <= 1_000_000);
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
    let mut worker = Worker::launch(&path);
    drop(worker.input.take());
    assert!(worker.child.wait().unwrap().success());
    assert!(!path.exists());
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
    assert!(metric(metrics, "max_unit_cpu_ns") <= 1_000_000);
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
