// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

use bboxy450_harness::{
    dream::{
        d0::{self, Cluster, Merge},
        fingerprint as fp, join,
    },
    protocol::*,
    redact::Set,
};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    time::Duration,
};

fn push(events: &mut Vec<Envelope>, session: SessionId, event: Event) {
    let seq = Seq(events.len() as u64 + 1);
    events.push(Envelope {
        v: PROTOCOL,
        session,
        epoch: Epoch(1),
        seq: Some(seq),
        at: seq,
        ts_ms: seq.0 * 100,
        cause: None,
        native: None,
        raw: None,
        ext: Default::default(),
        event,
    });
}
fn meta(cwd: &Path) -> SessionMeta {
    SessionMeta {
        session: SessionId::new(),
        cwd: cwd.into(),
        vcs: Some(Vcs {
            repo: Some("github.com/fixture/dream".into()),
            ..Default::default()
        }),
        dream: None,
        source: None,
        profile: Some("scripted".into()),
        fork: None,
        protocol: PROTOCOL,
        harness: "fixture".into(),
        created_ms: 0,
    }
}
fn write_log(path: &Path, cwd: &Path, task: Option<&str>, turns: usize) -> Vec<u8> {
    let meta = meta(cwd);
    let session = meta.session;
    let mut events = vec![];
    push(&mut events, session, Event::SessionStarted { meta });
    for n in 0..turns {
        let turn = TurnId::new();
        push(&mut events, session, Event::TurnQueued { turn });
        push(
            &mut events,
            session,
            Event::TurnStarted {
                turn,
                model: Some("fixture".into()),
                origin: None,
            },
        );
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
            push(
                &mut events,
                session,
                Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::ToolCall,
                    parent: None,
                    head: None,
                },
            );
            push(
                &mut events,
                session,
                Event::ItemCompleted {
                    item,
                    status,
                    body: Item::ToolCall {
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
                    },
                },
            );
        }
        let approval = ItemId::new();
        push(
            &mut events,
            session,
            Event::ItemStarted {
                turn: Some(turn),
                item: approval,
                kind: ItemKind::Approval,
                parent: None,
                head: None,
            },
        );
        push(
            &mut events,
            session,
            Event::ItemCompleted {
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
            },
        );
        if let Some(task) = task {
            let item = ItemId::new();
            push(
                &mut events,
                session,
                Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::ToolCall,
                    parent: None,
                    head: None,
                },
            );
            push(
                &mut events,
                session,
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::ToolCall {
                        call_id: item.to_string(),
                        name: "mcp__bboxy__task_report".into(),
                        tool_kind: ToolKind::Other,
                        input_raw: serde_json::json!({"task_path":task}).to_string(),
                        output: vec![OutputBlock::Text {
                            text: "reported".into(),
                        }],
                        locations: vec![],
                        spill: None,
                        title: None,
                        server: None,
                        raw_output: None,
                    },
                },
            );
        }
        push(
            &mut events,
            session,
            Event::TurnEnded {
                turn,
                reason: StopReason::Quiet { ms: 10 },
                usage: Some(Usage {
                    cost_micros_usd: Some(if n == 19 { 10_000 } else { n as u64 + 1 }),
                    source: Some(NumberSource::Reported),
                    scope: Some(UsageScope::Turn),
                    model: Some("fixture".into()),
                    backend: Some(Box::new(Backend {
                        kind: BackendKind::Engine,
                        name: "engine".into(),
                        version: None,
                    })),
                    ..Default::default()
                }),
            },
        );
    }
    push(&mut events, session, Event::SessionClosed);
    events[0]
        .ext
        .insert("future".into(), serde_json::json!({"keep":true}));
    let heads = events
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted { item, body, .. } => Some((*item, body.head())),
            _ => None,
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for e in &mut events {
        if let Event::ItemStarted { item, head, .. } = &mut e.event {
            *head = heads.get(item).cloned().flatten();
        }
    }
    laws::check_authored(&events).unwrap();
    let bytes = events
        .iter()
        .flat_map(|e| {
            let mut v = serde_json::to_vec(e).unwrap();
            v.push(b'\n');
            v
        })
        .collect::<Vec<_>>();
    std::fs::write(path, &bytes).unwrap();
    bytes
}
fn cache(root: &Path, fingerprint: &str) -> Cluster {
    serde_json::from_slice(
        &std::fs::read(
            root.join("d0")
                .join(fp::digest("github.com/fixture/dream"))
                .join(format!("{fingerprint}.json")),
        )
        .unwrap(),
    )
    .unwrap()
}
fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
struct Repo {
    home: tempfile::TempDir,
    path: PathBuf,
    head: String,
}
impl Repo {
    fn new() -> Self {
        let home = tempfile::tempdir_in("/tmp").unwrap();
        let path = home.path().join("repo");
        std::fs::create_dir(&path).unwrap();
        git(&path, &["init", "-b", "main"]);
        let key = home.path().join("signing");
        assert!(
            Command::new("/usr/bin/ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-f"])
                .arg(&key)
                .status()
                .unwrap()
                .success()
        );
        for (name, value) in [
            ("user.name", "Boxy"),
            ("user.email", "bboxy450@gmail.com"),
            ("gpg.format", "ssh"),
            ("commit.gpgsign", "true"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git(&path, &["config", name, value]);
        }
        git(&path, &["config", "user.signingkey", key.to_str().unwrap()]);
        std::fs::create_dir_all(path.join(".bboxy/tasks/nested")).unwrap();
        std::fs::write(path.join(".bboxy/tasks/nested/task with spaces.md"),"---\nid: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nnumber: 42\ntype: task\nstatus: claimed\nverify: cargo test\nverify_result: {exit_code: 7}\nrejection: {reason: 'BROKEN /tmp/path 123'}\n---\nfixture\n").unwrap();
        std::fs::write(path.join(".bboxy/tasks/other.md"),"---\nid: 01ARZ3NDEKTSV4RRFFQ69G5FAW\nnumber: 43\ntype: task\nstatus: done\nverify_result: {exit_code: 0}\n---\nother\n").unwrap();
        git(&path, &["add", ".bboxy"]);
        git(&path, &["commit", "-m", "private signed task fixture"]);
        let head = git(&path, &["rev-parse", "HEAD"]);
        git(&path, &["update-ref", "refs/remotes/origin/main", &head]);
        assert!(
            git(&path, &["cat-file", "-p", "HEAD"])
                .contains("gpgsig -----BEGIN SSH SIGNATURE-----")
        );
        Self { home, path, head }
    }
}
fn cli(home: &Path, args: &[&str], extra: impl FnOnce(&mut Command)) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    c.args(args)
        .env("HOME", home)
        .env("TMPDIR", home)
        .env_remove("BBOXY_CAPABILITY")
        .env_remove("BBOXY_CAPABILITY_FILE")
        .env_remove("BBOXY_SEAT_ID")
        .env_remove("BBOXY_SOCK");
    extra(&mut c);
    c.output().unwrap()
}
#[tokio::test]
async fn replay_fingerprints_turns_exact_bytes_cost_and_union_are_deterministic() {
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let path = t.path().join("input.jsonl");
    let original = write_log(&path, t.path(), None, 20);
    let root = t.path().join("dream");
    let a = d0::replay(vec![path.clone()], root.clone(), false, Set::default())
        .await
        .unwrap();
    assert_eq!(a.episodes.len(), 20);
    assert_eq!(a.updated, 5);
    assert!(a.max_poll_cpu_ns <= 1_000_000);
    let quiet = fp::digest("quiet-stop");
    let cost = fp::digest("cost-outlier:engine/fixture");
    assert_eq!(cache(&root, &quiet).count, 20);
    assert_eq!(cache(&root, &cost).count, 1);
    let before = std::fs::read(
        root.join("d0")
            .join(fp::digest("github.com/fixture/dream"))
            .join(format!("{quiet}.json")),
    )
    .unwrap();
    let b = d0::replay(
        vec![path.clone(), path.clone()],
        root.clone(),
        true,
        Set::default(),
    )
    .await
    .unwrap();
    assert_eq!(
        a.episodes
            .iter()
            .map(|e| (&e.origin, &e.fingerprints))
            .collect::<Vec<_>>(),
        b.episodes
            .iter()
            .map(|e| (&e.origin, &e.fingerprints))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        before,
        std::fs::read(
            root.join("d0")
                .join(fp::digest("github.com/fixture/dream"))
                .join(format!("{quiet}.json"))
        )
        .unwrap()
    );
    let EvidenceRef::Event { seq, digest, .. } = &a.episodes[0].origin else {
        panic!()
    };
    let line = original
        .split(|b| *b == b'\n')
        .nth(seq.0 as usize - 1)
        .unwrap();
    assert_eq!(*digest, format!("sha256:{}", fp::digest(line)));
    assert_eq!(original, std::fs::read(&path).unwrap());
    let open = t.path().join("open.jsonl");
    let cut = original.rsplitn(3, |b| *b == b'\n').nth(1).unwrap().len();
    std::fs::write(&open, &original[..original.len() - cut - 1]).unwrap();
    let skipped = d0::replay(vec![open], root, false, Set::default())
        .await
        .unwrap();
    assert_eq!(skipped.skipped, 1);
}
#[tokio::test]
async fn committed_task_paths_are_enumerated_and_pins_are_exact() {
    let repo = Repo::new();
    let taskpath = ".bboxy/tasks/nested/task with spaces.md";
    let tasks = join::committed_tasks(&repo.path, "github.com/fixture/dream")
        .await
        .unwrap();
    assert_eq!(tasks.len(), 2);
    let nested = repo.path.join("sub/directory");
    std::fs::create_dir_all(&nested).unwrap();
    assert_eq!(
        join::committed_tasks(&nested, "github.com/fixture/dream")
            .await
            .unwrap()
            .len(),
        2
    );
    let task = tasks.iter().find(|t| t.number == Some(42)).unwrap();
    assert_eq!(task.verdict, join::Verdict::Rejected);
    let EvidenceRef::Record {
        path,
        commit,
        digest,
        ..
    } = &task.pin
    else {
        panic!()
    };
    assert_eq!(path, taskpath);
    assert_eq!(commit, &repo.head);
    assert_eq!(
        digest,
        &format!(
            "sha256:{}",
            fp::digest(std::fs::read(repo.path.join(taskpath)).unwrap())
        )
    );
    let input = repo.home.path().join("session.jsonl");
    let original = write_log(&input, &repo.path, Some("#42"), 2);
    let result = d0::replay(
        vec![input.clone()],
        repo.home.path().join("dream"),
        false,
        Set::default(),
    )
    .await
    .unwrap();
    assert_eq!(result.episodes.len(), 1);
    let e = &result.episodes[0];
    assert_eq!(e.origin, task.pin);
    assert_eq!(e.turns.len(), 2);
    assert!(
        e.fingerprints
            .contains(&fp::digest("rejected:broken <path> <n>"))
    );
    assert!(
        e.fingerprints
            .contains(&fp::digest("verify-fail:cargo test"))
    );
    assert_eq!(original, std::fs::read(input).unwrap());
}
#[test]
fn cli_pins_a_moving_origin_before_any_task_read_and_never_writes_sessions() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::new();
    let taskpath = repo.path.join(".bboxy/tasks/nested/task with spaces.md");
    let old = std::fs::read(&taskpath).unwrap();
    std::fs::write(&taskpath,"---\nid: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nnumber: 42\ntype: task\nstatus: done\nverify_result: {exit_code: 0}\n---\nnew\n").unwrap();
    git(
        &repo.path,
        &["commit", "-am", "private signed changed task"],
    );
    let next = git(&repo.path, &["rev-parse", "HEAD"]);
    let input = repo.home.path().join("session.jsonl");
    let source = write_log(&input, &repo.path, Some("01ARZ3NDEKTSV4RRFFQ69G5FAV"), 1);
    let bin = repo.home.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let wrapper = bin.join("git");
    std::fs::write(&wrapper,"#!/bin/sh\ncase \"$*\" in\n  *ls-tree*) /usr/bin/git \"$@\" || exit $?; /usr/bin/git update-ref refs/remotes/origin/main \"$NEXT_TASK_COMMIT\"; exit $?;;\nesac\nexec /usr/bin/git \"$@\"\n").unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    let out = cli(
        repo.home.path(),
        &["dream", "replay", "--json", input.to_str().unwrap()],
        |c| {
            c.env("PATH", &bin).env("NEXT_TASK_COMMIT", &next);
        },
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["episodes"][0]["origin"]["commit"], repo.head);
    assert_eq!(value["episodes"][0]["task"]["verdict"], "rejected");
    assert_eq!(
        value["episodes"][0]["origin"]["digest"],
        format!("sha256:{}", fp::digest(old))
    );
    assert_eq!(git(&repo.path, &["rev-parse", "origin/main"]), next);
    assert_eq!(source, std::fs::read(input).unwrap());
    assert!(!repo.home.path().join(".bboxy450-harness/sessions").exists());
    assert_eq!(
        cli(repo.home.path(), &["dream", "unknown"], |_| {})
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        cli(
            repo.home.path(),
            &["dream", "replay", "missing.jsonl"],
            |_| {}
        )
        .status
        .code(),
        Some(2)
    );
}
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
#[ignore = "private child fixture, spawned only by the process witness"]
fn process_claim_child() {
    if let Some(input) = std::env::var_os("DREAM_NATIVE_INPUT") {
        let sessions = PathBuf::from(std::env::var_os("DREAM_NATIVE_SESSIONS").unwrap());
        let stage = bbt_rt::handle().block_on(async {
            let source = bboxy450_harness::import::open_native_source(PathBuf::from(input))
                .await
                .unwrap();
            let snapshot = bboxy450_harness::import::snapshot_native(source, Set::default())
                .await
                .unwrap();
            bboxy450_harness::import::stage_native(snapshot, sessions.clone(), Set::default())
                .await
                .unwrap()
                .unwrap()
        });
        let session = stage.session;
        if std::env::var_os("DREAM_NATIVE_PUBLISH").is_some() {
            bbt_rt::handle().block_on(stage.publish()).unwrap();
            std::fs::write(
                std::env::var_os("DREAM_CLAIM_READY").unwrap(),
                session.to_string(),
            )
            .unwrap();
            std::thread::sleep(Duration::from_secs(600));
        } else {
            std::fs::write(
                std::env::var_os("DREAM_CLAIM_READY").unwrap(),
                session.to_string(),
            )
            .unwrap();
            std::thread::sleep(Duration::from_secs(600));
            drop(stage);
        }
        return;
    }
    let root = PathBuf::from(std::env::var_os("DREAM_CLAIM_ROOT").unwrap());
    let fingerprint = std::env::var("DREAM_CLAIM_FP").unwrap();
    let claim = bbt_rt::handle()
        .block_on(bboxy450_harness::runtime::blocking(move || {
            d0::Claim::acquire(&root, &fingerprint)
        }))
        .unwrap()
        .unwrap();
    std::fs::write(std::env::var_os("DREAM_CLAIM_READY").unwrap(), b"ready").unwrap();
    std::thread::sleep(Duration::from_secs(600));
    drop(claim);
}
fn cluster(fingerprint: &str, pin: u128) -> Cluster {
    Cluster {
        fingerprint: fingerprint.into(),
        count: 999,
        episodes: vec![EvidenceRef::Event {
            session: SessionId(ulid::Ulid::from(pin)),
            seq: Seq(1),
            digest: format!("sha256:{}", fp::digest(pin.to_le_bytes())),
        }],
        first: pin as u64,
        last: pin as u64,
    }
}
#[tokio::test]
async fn busy_claim_skips_other_shards_progress_and_process_crash_releases() {
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let root = t.path().join("dream");
    let a = fp::digest("quiet-stop");
    let b = fp::digest("other");
    let ready = t.path().join("ready");
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "process_claim_child", "--ignored"])
            .env("DREAM_CLAIM_ROOT", &root)
            .env("DREAM_CLAIM_FP", &a)
            .env("DREAM_CLAIM_READY", &ready)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let busy = tokio::time::timeout(
        Duration::from_secs(2),
        d0::merge_cluster(
            root.clone(),
            "github.com/fixture/dream".into(),
            cluster(&a, 1),
            false,
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(busy, Merge::Busy);
    assert_eq!(
        d0::merge_cluster(
            root.clone(),
            "github.com/fixture/dream".into(),
            cluster(&b, 2),
            false
        )
        .await
        .unwrap(),
        Merge::Updated
    );
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    for (pin, rebuild) in [(1, false), (2, false), (1, true)] {
        assert_eq!(
            d0::merge_cluster(
                root.clone(),
                "github.com/fixture/dream".into(),
                cluster(&a, pin),
                rebuild
            )
            .await
            .unwrap(),
            Merge::Updated
        );
    }
    let merged = cache(&root, &a);
    assert_eq!(merged.count, 2);
    assert_eq!(merged.episodes.len(), 2);
    assert_eq!((merged.first, merged.last), (1, 2));
}
#[tokio::test]
async fn close_returns_and_shared_runtime_progresses_while_d0_read_is_delayed() {
    use std::sync::{Arc, Condvar, Mutex};
    type Gate = Arc<(Mutex<bool>, Condvar)>;
    struct Release(Gate);
    impl Drop for Release {
        fn drop(&mut self) {
            *self.0.0.lock().unwrap() = true;
            self.0.1.notify_all();
        }
    }
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let input = t.path().join("source.jsonl");
    let source = write_log(&input, t.path(), None, 1);
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release = Release(gate.clone());
    let (sent, ready) = std::sync::mpsc::channel();
    let mut workers = vec![];
    for _ in 0..4 {
        let gate = gate.clone();
        let sent = sent.clone();
        workers.push(
            bbt_rt::handle().spawn(bboxy450_harness::runtime::blocking(move || {
                bboxy450_harness::runtime::assert_off_rt_worker();
                sent.send(()).unwrap();
                let mut opened = gate.0.lock().unwrap();
                while !*opened {
                    opened = gate.1.wait(opened).unwrap();
                }
            })),
        );
    }
    tokio::task::spawn_blocking(move || {
        for _ in 0..4 {
            ready.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    })
    .await
    .unwrap();
    let root = t.path().join("dream");
    let handle = d0::on_close(input.clone(), root.clone(), Set::default());
    tokio::time::timeout(
        Duration::from_secs(2),
        bbt_rt::handle().spawn(async {
            tokio::task::yield_now().await;
            true
        }),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        !handle.is_finished(),
        "D0 must still be delayed by the four occupied existing permits"
    );
    drop(release);
    for worker in workers {
        worker.await.unwrap();
    }
    let result = handle.await.unwrap().unwrap();
    assert_eq!(result.updated, 4);
    assert!(result.max_poll_cpu_ns <= 1_000_000);
    assert_eq!(cache(&root, &fp::digest("quiet-stop")).count, 1);
    assert_eq!(source, std::fs::read(input).unwrap());
}

fn native_input(home: &Path, cwd: &Path, kind: &str, large: bool) -> PathBuf {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/dream/replay/native-{kind}.jsonl"));
    let mut records = std::fs::read_to_string(fixture)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .collect::<Vec<_>>();
    for r in &mut records {
        if r.get("cwd").is_some() {
            r["cwd"] = serde_json::json!(cwd);
        }
        if r.pointer("/payload/cwd").is_some() {
            r["payload"]["cwd"] = serde_json::json!(cwd);
        }
    }
    if large {
        records.push(serde_json::json!({"type":"future-oversized","sessionId":"native-claude","future":{"retain":format!("{}fixture-native-secret{}","🙂".repeat(20_000),"z".repeat(70_000))}}));
    }
    let path = home.join(format!("native-{kind}.jsonl"));
    std::fs::write(
        &path,
        records
            .iter()
            .map(|r| r.to_string() + "\n")
            .collect::<String>(),
    )
    .unwrap();
    path
}
fn public_logs(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return vec![];
    }
    let mut result = std::fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "jsonl")
                && p.file_stem()
                    .unwrap()
                    .to_str()
                    .is_some_and(|s| s.parse::<ulid::Ulid>().is_ok())
        })
        .collect::<Vec<_>>();
    result.sort();
    result
}
fn blobs(root: &Path, session: SessionId) -> std::collections::BTreeMap<String, Vec<u8>> {
    let dir = root.join(session.to_string()).join("blobs");
    if !dir.exists() {
        return Default::default();
    }
    std::fs::read_dir(dir)
        .unwrap()
        .map(|p| {
            let p = p.unwrap().path();
            (
                p.file_name().unwrap().to_str().unwrap().to_owned(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect()
}
#[tokio::test]
async fn native_roots_replay_rebuild_and_exact_blob_pins_are_identical() {
    use bboxy450_harness::import;
    for kind in ["claude", "codex"] {
        let repo = Repo::new();
        let input = native_input(repo.home.path(), &repo.path, kind, true);
        let original = std::fs::read(&input).unwrap();
        let set = Set::default();
        set.register("fixture-native-secret");
        let a = repo.home.path().join("sessions-a");
        let b = repo.home.path().join("sessions-b");
        let pa = import::materialize_native(input.clone(), a.clone(), set.clone())
            .await
            .unwrap()
            .unwrap();
        let pb = import::materialize_native(input.clone(), b.clone(), set.clone())
            .await
            .unwrap()
            .unwrap();
        let ba = std::fs::read(&pa).unwrap();
        assert_eq!(ba, std::fs::read(&pb).unwrap());
        let events = std::str::from_utf8(&ba)
            .unwrap()
            .lines()
            .map(|l| Envelope::from_line(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_authored(&events).unwrap();
        let session = events[0].session;
        assert_eq!(events[0].ts_ms, 0);
        assert_eq!(events[1].ts_ms, 0);
        assert!(
            events
                .iter()
                .all(|e| e.cause.is_none() && e.epoch == Epoch(1))
        );
        let Event::SessionStarted { meta } = &events[0].event else {
            panic!()
        };
        assert!(meta.dream.is_none());
        assert_eq!(
            meta.source.as_ref().unwrap().native_id.as_deref(),
            Some(if kind == "claude" {
                "native-claude"
            } else {
                "native-codex"
            })
        );
        assert_eq!(
            events[0].ext["native_materialization"]["native_process_completion"],
            "unproven"
        );
        let ab = blobs(&a, session);
        assert!(!ab.is_empty());
        assert_eq!(ab, blobs(&b, session));
        assert!(ab.values().any(|bytes| {
            serde_json::from_slice::<serde_json::Value>(bytes)
                .ok()
                .and_then(|v| {
                    v.pointer("/future/retain")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .is_some_and(|s| s.len() > 100_000 && s.contains("<redacted>"))
        }));
        for (name, bytes) in &ab {
            assert_eq!(name, &fp::digest(bytes));
            assert!(!String::from_utf8_lossy(bytes).contains("fixture-native-secret"));
        }
        for forbidden in [
            "fixture-native-secret",
            "fixture-password",
            "private-query",
            "private-fragment",
        ] {
            assert!(!String::from_utf8_lossy(&ba).contains(forbidden));
        }
        assert!(
            events.iter().any(|e| e
                .native
                .as_ref()
                .is_some_and(|n| n.id == "u1" || n.id == "future1"))
                || kind == "codex"
        );
        let root = repo.home.path().join("dream");
        let r1 = d0::replay_with_sessions(
            vec![input.clone()],
            root.clone(),
            a.clone(),
            false,
            set.clone(),
        )
        .await
        .unwrap();
        let r2 = d0::replay_with_sessions(
            vec![input.clone(), pa.clone()],
            root.clone(),
            a.clone(),
            true,
            set.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            r1.episodes.iter().map(|e| &e.origin).collect::<Vec<_>>(),
            r2.episodes.iter().map(|e| &e.origin).collect::<Vec<_>>()
        );
        assert!(!r1.episodes.is_empty());
        for episode in &r2.episodes {
            let EvidenceRef::Event {
                session,
                seq,
                digest,
            } = &episode.origin
            else {
                panic!()
            };
            let bytes = std::fs::read(a.join(format!("{session}.jsonl"))).unwrap();
            let line = bytes
                .split(|b| *b == b'\n')
                .nth(seq.0 as usize - 1)
                .unwrap();
            assert_eq!(digest, &format!("sha256:{}", fp::digest(line)));
        }
        assert_eq!(public_logs(&a), vec![pa]);
        assert_eq!(original, std::fs::read(&input).unwrap());
    }
}
#[tokio::test]
async fn native_malformed_changed_and_bounded_snapshots_never_publish() {
    use bboxy450_harness::import;
    let repo = Repo::new();
    let input = native_input(repo.home.path(), &repo.path, "claude", false);
    let sessions = repo.home.path().join("sessions");
    let opened = import::open_native_source(input.clone()).await.unwrap();
    std::fs::write(&input, b"changed\n").unwrap();
    assert!(
        import::snapshot_native(opened, Set::default())
            .await
            .is_err()
    );
    let input = native_input(repo.home.path(), &repo.path, "claude", false);
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&input)
        .unwrap()
        .write_all(b"{\"incomplete\":")
        .unwrap();
    assert!(
        import::materialize_native(input.clone(), sessions.clone(), Set::default())
            .await
            .is_err()
    );
    assert!(public_logs(&sessions).is_empty());
    std::fs::File::create(&input)
        .unwrap()
        .set_len(import::MAX_NATIVE_SNAPSHOT_BYTES + 1)
        .unwrap();
    assert!(import::open_native_source(input).await.is_err());
}
#[tokio::test]
async fn native_conflicting_log_and_blob_are_refused_without_repair() {
    use bboxy450_harness::import;
    for conflict_blob in [false, true] {
        let repo = Repo::new();
        let input = native_input(repo.home.path(), &repo.path, "codex", true);
        let sessions = repo.home.path().join("sessions");
        let path = import::materialize_native(input.clone(), sessions.clone(), Set::default())
            .await
            .unwrap()
            .unwrap();
        let target = if conflict_blob {
            let e: Envelope = serde_json::from_slice(
                std::fs::read(&path)
                    .unwrap()
                    .split(|b| *b == b'\n')
                    .next()
                    .unwrap(),
            )
            .unwrap();
            let map = blobs(&sessions, e.session);
            sessions
                .join(e.session.to_string())
                .join("blobs")
                .join(map.keys().next().unwrap())
        } else {
            path.clone()
        };
        std::fs::write(&target, b"foreign-edit").unwrap();
        assert!(
            import::materialize_native(input, sessions, Set::default())
                .await
                .is_err()
        );
        assert_eq!(std::fs::read(target).unwrap(), b"foreign-edit");
    }
}
#[tokio::test]
async fn native_crash_before_and_after_publication_recovers_only_complete_custody() {
    use bboxy450_harness::import;
    for publish in [false, true] {
        let repo = Repo::new();
        let input = native_input(repo.home.path(), &repo.path, "claude", false);
        let sessions = repo.home.path().join("sessions");
        let ready = repo.home.path().join("ready");
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "process_claim_child", "--ignored"])
            .env("DREAM_NATIVE_INPUT", &input)
            .env("DREAM_NATIVE_SESSIONS", &sessions)
            .env("DREAM_CLAIM_READY", &ready)
            .stdout(std::process::Stdio::null());
        if publish {
            command.env("DREAM_NATIVE_PUBLISH", "1");
        }
        let mut child = OwnedChild(command.spawn().unwrap());
        tokio::time::timeout(Duration::from_secs(10), async {
            while std::fs::read_to_string(&ready)
                .ok()
                .is_none_or(|s| s.parse::<ulid::Ulid>().is_err())
            {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(public_logs(&sessions).len(), usize::from(publish));
        let root = repo.home.path().join("dream");
        if !publish {
            let r = d0::replay_with_sessions(
                vec![sessions.clone()],
                root.clone(),
                sessions.clone(),
                false,
                Set::default(),
            )
            .await
            .unwrap();
            assert!(r.episodes.is_empty());
        }
        let id = std::fs::read_to_string(ready).unwrap();
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        let path = import::materialize_native(input, sessions.clone(), Set::default())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(path.file_stem().unwrap().to_str().unwrap(), id);
        assert_eq!(public_logs(&sessions).len(), 1);
        let result = d0::replay_with_sessions(vec![path], root, sessions, false, Set::default())
            .await
            .unwrap();
        assert_eq!(result.episodes.len(), 1);
    }
}
#[test]
fn native_two_processes_race_without_duplicate_imports_or_cache_counts() {
    let repo = Repo::new();
    let input = native_input(repo.home.path(), &repo.path, "codex", false);
    let sessions = repo.home.path().join("sessions");
    let spawn = || {
        let mut c = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        c.args(["dream", "replay", "--json", "--sessions"])
            .arg(&sessions)
            .arg(&input)
            .env("HOME", repo.home.path())
            .env_remove("BBOXY_CAPABILITY")
            .env_remove("BBOXY_CAPABILITY_FILE")
            .env_remove("BBOXY_SEAT_ID")
            .env_remove("BBOXY_SOCK")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        c.spawn().unwrap()
    };
    let a = spawn();
    let b = spawn();
    for child in [a, b] {
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(public_logs(&sessions).len(), 1);
    let args = [
        "dream",
        "replay",
        "--json",
        "--rebuild",
        "--sessions",
        sessions.to_str().unwrap(),
        input.to_str().unwrap(),
    ];
    let one = cli(repo.home.path(), &args, |_| {});
    let two = cli(repo.home.path(), &args, |_| {});
    assert!(one.status.success() && two.status.success());
    let one: serde_json::Value = serde_json::from_slice(&one.stdout).unwrap();
    let two: serde_json::Value = serde_json::from_slice(&two.stdout).unwrap();
    assert_eq!(one["episodes"], two["episodes"]);
    let root = repo.home.path().join(".bboxy450-harness/dream/d0");
    let mut counts = Vec::new();
    for dir in std::fs::read_dir(root).unwrap() {
        for path in std::fs::read_dir(dir.unwrap().path()).unwrap() {
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path.unwrap().path()).unwrap()).unwrap();
            counts.push(value["count"].as_u64().unwrap());
        }
    }
    assert!(!counts.is_empty());
    assert!(counts.iter().all(|n| *n == 1));
}
#[test]
fn native_id_domains_have_fixed_vectors() {
    assert_eq!(
        format!(
            "{:032x}",
            u128::from(bboxy450_harness::import::materialized_id(
                &[0; 32], "session", 0
            ))
        ),
        "431e49d2ad5347ab796adf4e9e69f05d"
    );
    assert_eq!(
        format!(
            "{:032x}",
            u128::from(bboxy450_harness::import::materialized_id(
                &[0; 32], "turn", 0
            ))
        ),
        "0c856e6828882406940ff3cc3be0a6fa"
    );
    assert_eq!(
        format!(
            "{:032x}",
            u128::from(bboxy450_harness::import::materialized_id(
                &[0; 32], "item", 0
            ))
        ),
        "82f0a104c9a86f7d2db8841c0e8ec933"
    );
    assert_eq!(
        format!(
            "{:032x}",
            u128::from(bboxy450_harness::import::materialized_id(
                &[0; 32], "call", 0
            ))
        ),
        "b0010a8abbe64187cb5617fbbc6765b1"
    );
    assert_eq!(
        format!(
            "{:032x}",
            u128::from(bboxy450_harness::import::materialized_id(&[0; 32], "op", 0))
        ),
        "5c146a72e5b91ad1385155300ba14607"
    );
}

#[test]
#[ignore = "private close-hook process fixture"]
fn process_close_child() {
    use bboxy450_harness::{
        engine::Config,
        gateway::{Gateway, Host},
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
    };
    use std::sync::Arc;
    bbt_rt::handle().block_on(async {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        let kind = std::env::var("DREAM_CLOSE_KIND").unwrap();
        let cwd = PathBuf::from(std::env::var_os("DREAM_CLOSE_CWD").unwrap());
        let input = PathBuf::from(std::env::var_os("DREAM_CLOSE_INPUT").unwrap());
        let sessions = home.join("sessions");
        let meta = if kind == "watch" {
            meta(&cwd)
        } else {
            let first: Envelope = serde_json::from_slice(
                std::fs::read(&input)
                    .unwrap()
                    .split(|b| *b == b'\n')
                    .next()
                    .unwrap(),
            )
            .unwrap();
            let Event::SessionStarted { meta } = first.event else {
                panic!()
            };
            meta
        };
        let config = Config {
            meta: meta.clone(),
            sessions: sessions.clone(),
            profiles: home.join("profiles"),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        };
        let term_c1 = std::env::var("DREAM_TERM_C1").as_deref() == Ok("1");
        let foreground_config = config.clone();
        let host = Host::new(config).with_launch(bboxy450_harness::engine::LaunchOptions {
            dream_owner: if term_c1 {
                bboxy450_harness::dream::c1::DreamOwner::TermC1
            } else {
                bboxy450_harness::dream::c1::DreamOwner::Legacy
            },
            ..Default::default()
        });
        let gateway = if kind == "watch" {
            host.watch(
                Some(bboxy450_harness::import::Native::ClaudeCode),
                input.clone(),
                cwd,
            )
            .await
            .unwrap()
        } else {
            host.open(meta.session).await.unwrap()
        };
        if kind == "watch" {
            let mut sub = gateway
                .subscribe(bboxy450_harness::gateway::Subscribe {
                    session: gateway.session(),
                    after: None,
                    ephemeral: false,
                })
                .unwrap();
            tokio::time::timeout(Duration::from_secs(8), async {
                loop {
                    if let Some(e) = sub.recv().await
                        && matches!(e.event, Event::TurnEnded { .. })
                    {
                        break;
                    }
                }
            })
            .await
            .unwrap();
        }
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let foreground = if term_c1 {
            Some(bbt_rt::handle().spawn(async move {
                let mut config = foreground_config;
                config.meta.session = SessionId::new();
                config.meta.profile = None;
                config.meta.fork = None;
                let session = config.meta.session;
                let cwd = config.meta.cwd.clone();
                let host = Host::new(config).with_launch(bboxy450_harness::engine::LaunchOptions {
                    dream_owner: bboxy450_harness::dream::c1::DreamOwner::TermC1,
                    ..Default::default()
                });
                let (g, _) = host
                    .create(OpEnvelope {
                        v: PROTOCOL,
                        session,
                        op_id: OpId::new(),
                        op: Op::StartSession {
                            cwd,
                            profile: None,
                            fork: None,
                        },
                    })
                    .await
                    .unwrap();
                let mut sub = g
                    .subscribe(bboxy450_harness::gateway::Subscribe {
                        session,
                        after: Some((g.hello().epoch, g.hello().head)),
                        ephemeral: false,
                    })
                    .unwrap();
                let mut ready = Some(ready_tx);
                let mut times = Vec::new();
                for _ in 0..200 {
                    let at = std::time::Instant::now();
                    g.send(OpEnvelope {
                        v: PROTOCOL,
                        session,
                        op_id: OpId::new(),
                        op: Op::UserTurn {
                            blocks: vec![InputBlock::Text {
                                text: "private foreground message".into(),
                            }],
                            mode: TurnMode::Queue,
                        },
                    })
                    .unwrap();
                    if let Some(tx) = ready.take() {
                        let _ = tx.send(());
                    }
                    tokio::time::timeout(Duration::from_millis(250), async {
                        loop {
                            if let Some(e) = sub.recv().await
                                && matches!(e.event, Event::TurnEnded { .. })
                            {
                                break;
                            }
                        }
                    })
                    .await
                    .expect("foreground turn stalled behind close");
                    times.push(at.elapsed().as_micros());
                }
                times.sort_unstable();
                assert!(times[198] <= 100_000, "foreground p99 {:?}", times[198]);
                assert!(times[199] <= 250_000);
                g.send(OpEnvelope {
                    v: PROTOCOL,
                    session,
                    op_id: OpId::new(),
                    op: Op::Close,
                })
                .unwrap();
                tokio::time::timeout(Duration::from_secs(2), host.join())
                    .await
                    .unwrap()
                    .unwrap();
                eprintln!(
                    "managed-close foreground rounds=200 p99_us={} max_us={}",
                    times[198], times[199]
                );
            }))
        } else {
            None
        };
        if term_c1 {
            ready_rx.await.unwrap();
        }
        let id = gateway.session();
        gateway
            .send(OpEnvelope {
                v: PROTOCOL,
                session: id,
                op_id: OpId::new(),
                op: Op::Close,
            })
            .unwrap();
        tokio::time::timeout(Duration::from_secs(8), host.join())
            .await
            .unwrap()
            .unwrap();
        let log = sessions.join(format!("{id}.jsonl"));
        let before = std::fs::read(&log).unwrap();
        let fingerprint = if kind == "watch" {
            fp::tool_fail("Read", "Missing /tmp/native-path 42")
        } else {
            fp::digest("quiet-stop")
        };
        let path = home
            .join(".bboxy450-harness/dream/d0")
            .join(fp::digest("github.com/fixture/dream"))
            .join(format!("{fingerprint}.json"));
        if term_c1 {
            let events = std::str::from_utf8(&before)
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
                .collect::<Vec<_>>();
            assert!(matches!(events.last().unwrap().event, Event::SessionClosed));
            tokio::time::sleep(Duration::from_millis(150)).await;
            assert!(!path.exists(), "term-c1 still started legacy D0");
            foreground.unwrap().await.unwrap();
            assert_eq!(
                bboxy450_harness::dream::legacy_close_invocations(),
                (0, 0),
                "managed close invoked a detached legacy owner"
            );
            assert!(
                !home.join(".bboxy450-harness/ir-unknowns").exists(),
                "managed close started IR advice"
            );
            assert_eq!(before, std::fs::read(log).unwrap());
            return;
        }
        tokio::time::timeout(Duration::from_secs(8), async {
            while !path.exists() {
                tokio::task::yield_now().await;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("D0AutomaticCloseHook: closed source never reached derived cache");
        assert_eq!(bboxy450_harness::dream::legacy_close_invocations(), (1, 1));
        assert_eq!(before, std::fs::read(log).unwrap());
    });
}
#[test]
fn engine_acp_and_watch_close_hooks_drive_the_real_detached_pipeline() {
    for term_c1 in [false, true] {
        for kind in ["engine", "acp", "watch"] {
            let repo = Repo::new();
            let input = if kind == "watch" {
                native_input(repo.home.path(), &repo.path, "claude", false)
            } else {
                let path = repo.home.path().join("prepared.jsonl");
                let bytes = write_log(&path, &repo.path, None, 1);
                let mut events = std::str::from_utf8(&bytes)
                    .unwrap()
                    .lines()
                    .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
                    .collect::<Vec<_>>();
                events.pop();
                let Event::SessionStarted { meta } = &mut events[0].event else {
                    panic!()
                };
                meta.profile = Some(if kind == "acp" { "fake" } else { "scripted" }.into());
                let id = meta.session;
                let sessions = repo.home.path().join("sessions");
                std::fs::create_dir(&sessions).unwrap();
                let target = sessions.join(format!("{id}.jsonl"));
                if term_c1 {
                    let blob=serde_json::to_vec(&serde_json::json!({"type":"private_future_kind","payload":"x".repeat(4*1024*1024)})).unwrap();
                    let hash = fp::digest(&blob);
                    let blobs = sessions.join(id.to_string()).join("blobs");
                    std::fs::create_dir_all(&blobs).unwrap();
                    std::fs::write(blobs.join(&hash), &blob).unwrap();
                    let item = ItemId::new();
                    let template = events.last().unwrap().clone();
                    for event in [
                        Event::ItemStarted {
                            turn: None,
                            item,
                            kind: ItemKind::Native,
                            parent: None,
                            head: None,
                        },
                        Event::ItemCompleted {
                            item,
                            status: ItemStatus::Completed,
                            body: Item::Native {
                                provider: "fixture".into(),
                                record_type: "large-json".into(),
                                raw: serde_json::Value::Null,
                                blob: Some(BlobRef {
                                    sha256: hash.clone(),
                                    len: blob.len() as u64,
                                    mime: "application/json".into(),
                                }),
                            },
                        },
                    ] {
                        let seq = Seq(events.len() as u64 + 1);
                        events.push(Envelope {
                            event,
                            seq: Some(seq),
                            at: seq,
                            ts_ms: seq.0,
                            ..template.clone()
                        });
                    }
                    eprintln!("managed-close large JSON blob bytes={}", blob.len());
                }
                std::fs::write(
                    &target,
                    events
                        .iter()
                        .map(|e| serde_json::to_string(e).unwrap() + "\n")
                        .collect::<String>(),
                )
                .unwrap();
                target
            };
            let profiles = repo.home.path().join("profiles");
            std::fs::create_dir(&profiles).unwrap();
            let fake = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/acp-agent/fake_agent.py");
            std::fs::write(profiles.join("fake.md"),format!("---\nname: fake\nprovider: acp-agent\nagent:\n  command: [/usr/bin/python3, -I, {}, text]\non_unattended: deny\n---\n",fake.display())).unwrap();
            let mut child = OwnedChild(
                Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "process_close_child", "--ignored", "--nocapture"])
                    .env("HOME", repo.home.path())
                    .env("DREAM_CLOSE_KIND", kind)
                    .env("DREAM_TERM_C1", if term_c1 { "1" } else { "0" })
                    .env("DREAM_CLOSE_CWD", &repo.path)
                    .env("DREAM_CLOSE_INPUT", input)
                    .env_remove("BBOXY_CAPABILITY")
                    .env_remove("BBOXY_CAPABILITY_FILE")
                    .env_remove("BBOXY_SEAT_ID")
                    .env_remove("BBOXY_SOCK")
                    .spawn()
                    .unwrap(),
            );
            let status = child.0.wait().unwrap();
            assert!(
                status.success(),
                "{kind} automatic D0 close hook failed, term-c1={term_c1}"
            );
        }
    }
}

#[tokio::test]
async fn native_canonical_input_redaction_and_content_versions_keep_old_custody() {
    use bboxy450_harness::import;
    let repo = Repo::new();
    let input = native_input(repo.home.path(), &repo.path, "claude", true);
    let sessions = repo.home.path().join("sessions");
    let one = import::materialize_native(input.clone(), sessions.clone(), Set::default())
        .await
        .unwrap()
        .unwrap();
    let old = std::fs::read(&one).unwrap();
    let redactor = Set::default();
    redactor.register("fixture-native-secret");
    let two = import::materialize_native(input.clone(), sessions.clone(), redactor.clone())
        .await
        .unwrap()
        .unwrap();
    assert_ne!(one, two);
    let two_bytes = std::fs::read(&two).unwrap();
    // Another filesystem path/mtime and reversed object-key order are not identity.
    let copy = repo.home.path().join("copy.jsonl");
    let text = std::fs::read_to_string(&input).unwrap();
    let reversed = text
        .lines()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            let object = value.as_object().unwrap();
            format!(
                "{{{}}}\n",
                object
                    .iter()
                    .rev()
                    .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), v))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<String>();
    std::fs::write(&copy, reversed).unwrap();
    assert_eq!(
        import::materialize_native(copy, sessions.clone(), redactor.clone())
            .await
            .unwrap()
            .unwrap(),
        two
    );
    std::fs::write(
        &input,
        text.replace("Read the missing file.", "Read changed content."),
    )
    .unwrap();
    let three = import::materialize_native(input, sessions.clone(), redactor)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(two, three);
    assert_eq!(public_logs(&sessions).len(), 3);
    assert_eq!(old, std::fs::read(one).unwrap());
    assert_eq!(two_bytes, std::fs::read(two).unwrap());
}

#[tokio::test]
async fn selected_inspection_reuses_exact_d0_semantics_without_cache_or_task_join() {
    let t = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = t.path().join("input.jsonl");
    let bytes = write_log(&path, t.path(), None, 20);
    let events = std::str::from_utf8(&bytes)
        .unwrap()
        .split_inclusive('\n')
        .map(|l| Envelope::from_line(l).unwrap())
        .collect::<Vec<_>>();
    let Event::SessionStarted { meta } = &events[0].event else {
        panic!()
    };
    let mut log = d0::ClosedLog::new(meta.clone());
    let mut streaming = bboxy450_harness::dream::inspect::Transcript::default();
    for e in &events {
        streaming.observe(e).unwrap();
        log.observe(e, &Set::default(), false).unwrap();
    }
    streaming.finish().unwrap();
    laws::check_transcript(&events).unwrap();
    let rows = d0::inspect_closed(vec![log], |f| f()).unwrap();
    assert_eq!(
        std::fs::read_dir(t.path()).unwrap().count(),
        1,
        "selected inspection wrote side effects"
    );
    let replay = d0::replay(
        vec![path.clone()],
        t.path().join("cache"),
        false,
        Set::default(),
    )
    .await
    .unwrap();
    let selected = rows
        .iter()
        .filter_map(|r| {
            if let bboxy450_harness::dream::c1_wire::Row::Episode(e) = r {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), replay.episodes.len());
    for e in &replay.episodes {
        let EvidenceRef::Event { seq, digest, .. } = &e.origin else {
            panic!()
        };
        let row = selected.iter().find(|r| r.origin.seq.0 == seq.0).unwrap();
        assert_eq!(&row.origin.digest, digest);
        assert_eq!(
            row.fingerprints
                .iter()
                .map(|h| h.0.clone())
                .collect::<std::collections::BTreeSet<_>>(),
            e.fingerprints
        );
    }
    assert_eq!(bytes, std::fs::read(path).unwrap());
}
