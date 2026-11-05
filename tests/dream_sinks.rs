// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![allow(dead_code)]
use good as support;
use trial_support::good;
#[path = "fixtures/dream/trust/fixture.rs"]
mod fixture;
#[path = "fixtures/dream/good/receipt_support.rs"]
mod r;
#[path = "fixtures/dream/good/audit_fixture.rs"]
mod sufficient;
#[path = "support/dream.rs"]
mod trial_support;
use bboxy450_harness::{
    dream::{entry, flush},
    protocol::*,
    redact::Set,
    store::Store,
};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

struct Route {
    socket: PathBuf,
    root: Option<String>,
}
impl flush::Route for Route {
    fn root(&self) -> Option<&str> {
        self.root.as_deref()
    }
    fn call<'a>(&'a self, method: &'static str, args: Value) -> flush::Reply<'a> {
        Box::pin(async move {
            let mut socket = tokio::net::UnixStream::connect(&self.socket)
                .await
                .map_err(|_| entry::Invalid("private transport unavailable"))?;
            let mut bytes =
                serde_json::to_vec(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":args}))
                    .unwrap();
            bytes.push(b'\n');
            socket
                .write_all(&bytes)
                .await
                .map_err(|_| entry::Invalid("private transport write"))?;
            let mut line = String::new();
            BufReader::new(socket)
                .read_line(&mut line)
                .await
                .map_err(|_| entry::Invalid("private transport read"))?;
            let v: Value = serde_json::from_str(&line)
                .map_err(|_| entry::Invalid("unknown transport effect"))?;
            Ok(v["result"].clone())
        })
    }
}
fn pending(f: &trial_support::Fixture) -> Item {
    let evidence = f.episodes[0].task.as_ref().unwrap().pin.clone();
    let identity = entry::Identity {
        kind: "lesson".into(),
        title: "Private confirmed flush".into(),
        why: "Retain actual evidence".into(),
        how_to_apply: "Use pinned revisions".into(),
        evidence: vec![serde_json::to_value(&evidence).unwrap()],
        derived_from: vec![],
        revival: 0,
    };
    let id = identity.id().unwrap();
    let body = "Private bounded memory.\n";
    let draft = json!({"frontmatter":{"type":"memory","entry":id,"kind":identity.kind,"title":identity.title,"why":identity.why,"how_to_apply":identity.how_to_apply,"evidence":identity.evidence,"derived_from":[],"revival":0,"scope":f.scope(),"status":"proposed","harnesses":[],"content_sha256":entry::digest(body)},"body":body});
    let r = entry::Record::draft(&draft).unwrap();
    Item::MemoryWrite {
        entry: id,
        op: MemoryOp::Pending,
        record: RecordPin {
            root: trial_support::REPO.into(),
            id: None,
            path: "memories/flush.md".into(),
            digest: entry::digest(&r.bytes),
            commit: None,
        },
        evidence: vec![evidence],
        draft: Some(draft),
    }
}
async fn server(
    f: Arc<trial_support::Fixture>,
    mode: u8,
) -> (Route, tokio::task::JoinHandle<Vec<String>>) {
    let path = f.repo.home.path().join("flush.sock");
    let listener = tokio::net::UnixListener::bind(&path).unwrap();
    let job = tokio::spawn(async move {
        let mut results = std::collections::BTreeMap::<String, Value>::new();
        let mut calls = vec![];
        let count = if mode == 0 {
            4
        } else if mode == 1 {
            2
        } else {
            1
        };
        for _ in 0..count {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let req: Value = serde_json::from_str(&line).unwrap();
            let method = req["method"].as_str().unwrap();
            calls.push(method.to_owned());
            let p = req["params"].clone();
            let result = if method == "operation_status" {
                results
                    .get(p["original_op_id"].as_str().unwrap())
                    .unwrap()
                    .clone()
            } else if method == "knowledge_write" {
                let local = f.clone();
                let params = p.clone();
                let result=bboxy450_harness::runtime::blocking(move||{
                    let mut front=params["frontmatter"].clone();front["id"]=json!("01M00000000000000000000444");front["number"]=json!(444);
                    let record=good::sign_record(front,params["body"].as_str().unwrap(),&local.repo.key);
                    let path=local.repo.path.join(".bboxy/memories/flush.md");std::fs::create_dir_all(path.parent().unwrap()).unwrap();std::fs::write(path,&record.bytes).unwrap();
                    json!({"status":"completed","record":{"root":if mode==2 {"wrong.invalid/root"}else{trial_support::REPO},"id":"01M00000000000000000000444","path":"memories/flush.md","digest":entry::digest(&record.bytes),"commit":null}})
                }).await;
                results.insert(p["op_id"].as_str().unwrap().into(), result.clone());
                result
            } else {
                assert_eq!(method, "knowledge_commit");
                let result = if mode == 1 {
                    json!({"status":"failed"})
                } else {
                    let local = f.clone();
                    let mut pin = p["record"].clone();
                    let commit = bboxy450_harness::runtime::blocking(move || {
                        local.repo.git(&["add", ".bboxy/memories/flush.md"]);
                        local
                            .repo
                            .git(&["commit", "-qm", "private confirmed memory"]);
                        {
                            let head = local.repo.git(&["rev-parse", "HEAD"]);
                            local
                                .repo
                                .git(&["update-ref", "refs/remotes/origin/main", &head]);
                            head
                        }
                    })
                    .await;
                    pin["commit"] = json!(commit);
                    json!({"status":"completed","record":pin})
                };
                results.insert(p["op_id"].as_str().unwrap().into(), result.clone());
                result
            };
            // Real lost transport reply AFTER the write or commit occurred.
            if mode == 0 && method != "operation_status" {
                continue;
            }
            let mut bytes =
                serde_json::to_vec(&json!({"jsonrpc":"2.0","id":1,"result":result})).unwrap();
            bytes.push(b'\n');
            reader.get_mut().write_all(&bytes).await.unwrap();
        }
        calls
    });
    (
        Route {
            socket: path,
            root: Some(trial_support::REPO.into()),
        },
        job,
    )
}
async fn flush_store(f: &trial_support::Fixture) -> Store {
    let mut meta = SessionMeta {
        session: SessionId::new(),
        cwd: f.repo.path.clone(),
        profile: None,
        fork: None,
        protocol: PROTOCOL,
        harness: "dream".into(),
        created_ms: 0,
        vcs: None,
        source: None,
        dream: None,
    };
    meta.harness = "dream".into();
    meta.dream = Some(DreamRun {
        phase: DreamPhase::Flush,
        trigger: Trigger::Operator { message: None },
        scope: f.scope(),
        parent: None,
        prompt_version: None,
        thresholds: Some("dream-thresholds@2".into()),
    });
    Store::open(f.sessions.clone(), meta, None, Set::default())
        .await
        .unwrap()
}
#[tokio::test]
async fn flush_real_mcp_lost_write_and_commit_query_original_before_created() {
    let f = Arc::new(trial_support::Fixture::new().await);
    let mut resolver = f.resolver(false).await;
    let mut ticket = flush::Ticket::prepare(&pending(&f), &Set::default()).unwrap();
    let ops = (
        ticket.original_operations().0.to_owned(),
        ticket.original_operations().1.to_owned(),
    );
    let (route, job) = server(f.clone(), 0).await;
    assert!(matches!(
        ticket
            .advance(Some(&route), &mut resolver, &Set::default())
            .await,
        flush::Outcome::Pending(_)
    ));
    let mut store = flush_store(&f).await;
    assert!(ticket.append(&mut store).await.is_err());
    assert!(matches!(
        ticket
            .advance(Some(&route), &mut resolver, &Set::default())
            .await,
        flush::Outcome::Pending(_)
    ));
    let flush::Outcome::Committed(pin) = ticket
        .advance(Some(&route), &mut resolver, &Set::default())
        .await
    else {
        panic!("confirmed pin absent")
    };
    assert!(pin.commit.is_some());
    assert_eq!(
        ticket.original_operations(),
        (ops.0.as_str(), ops.1.as_str())
    );
    assert_eq!(ticket.append(&mut store).await.unwrap(), pin);
    ticket.append(&mut store).await.unwrap();
    assert_eq!(
        store
            .events
            .iter()
            .filter(|e| matches!(
                &e.event,
                Event::ItemCompleted {
                    body: Item::MemoryWrite {
                        op: MemoryOp::Created,
                        ..
                    },
                    ..
                }
            ))
            .count(),
        1
    );
    store.append(Event::SessionClosed, None).await.unwrap();
    laws::check_authored(&store.events).unwrap();
    assert_eq!(
        job.await.unwrap(),
        [
            "knowledge_write",
            "operation_status",
            "knowledge_commit",
            "operation_status"
        ]
    );
}
#[tokio::test]
async fn flush_real_commit_failure_wrong_root_and_no_route_stay_ineligible() {
    for mode in [1, 2] {
        let f = Arc::new(trial_support::Fixture::new().await);
        let mut resolver = f.resolver(false).await;
        let mut ticket = flush::Ticket::prepare(&pending(&f), &Set::default()).unwrap();
        assert_eq!(
            ticket.advance(None, &mut resolver, &Set::default()).await,
            flush::Outcome::NoRoute
        );
        let (route, job) = server(f.clone(), mode).await;
        assert!(matches!(
            ticket
                .advance(Some(&route), &mut resolver, &Set::default())
                .await,
            flush::Outcome::Failed(_)
        ));
        let mut store = flush_store(&f).await;
        assert!(ticket.append(&mut store).await.is_err());
        let calls = job.await.unwrap();
        assert_eq!(
            calls
                .iter()
                .filter(|m| m.as_str() == "knowledge_write")
                .count(),
            1
        );
    }
}

#[path = "fixtures/dream/sinks/support.rs"]
mod sinks;
use sinks::*;
#[tokio::test]
async fn lower_actual_publication_custody_idempotence_and_outer_run() {
    use bboxy450_harness::dream::{lower, receipt, seal};
    let f = lower_fixture();
    let mut resolver = f.resolver().await;
    let mut store = lower_store(&f).await;
    let sink = context_sink(&f);
    let outer = seal::Owner::default();
    let mut publications = vec![];
    for expected in [ReceiptOutcome::Published, ReceiptOutcome::Idempotent] {
        let plan = lower::prepare_at(
            &store,
            &mut resolver,
            sink.clone(),
            "engine",
            None,
            &Set::default(),
            4005,
        )
        .await
        .unwrap();
        let (s, result) = lower::publish(store, plan, outer.clone(), Set::default())
            .unwrap()
            .settle()
            .await;
        store = s;
        let p = result.unwrap();
        assert_eq!(p.accepted.body.outcome, expected);
        assert!(
            std::fs::read(&sink.path)
                .unwrap()
                .starts_with(b"<bboxy-memories")
        );
        let event = store
            .events
            .iter()
            .find(|e| e.seq == Some(p.accepted.seq))
            .unwrap();
        let t = resolver.trust().unwrap();
        let accepted = receipt::accept(event, &store.events, &f.domains, Some(&t))
            .await
            .unwrap();
        let mut logs = (*resolver.logs).clone();
        logs.insert(store.session, store.events.clone());
        resolver.logs = Arc::new(logs);
        receipt::check_publication(&accepted, &mut resolver, &f.domains, &Default::default())
            .await
            .unwrap();
        publications.push(accepted);
    }
    let (store, run) = finish_run(store, &f, &outer).await;
    let t = resolver.trust().unwrap();
    let event = store
        .events
        .iter()
        .find(|e| e.seq == Some(run.seq))
        .unwrap();
    let accepted = receipt::accept_with_publications(
        event,
        &store.events,
        &f.domains,
        Some(&t),
        &publications,
    )
    .await
    .unwrap();
    for p in &publications {
        receipt::check_outer(p, &accepted).unwrap();
    }
    bboxy450_harness::protocol::laws::check_authored(&store.events).unwrap();
}
#[tokio::test]
async fn lower_handwritten_shared_index_symlink_and_unapproved_are_not_adoptions() {
    use bboxy450_harness::dream::{lower, seal};
    let f = lower_fixture();
    let mut resolver = f.resolver().await;
    let sink = context_sink(&f);
    std::fs::create_dir_all(std::path::Path::new(&sink.path).parent().unwrap()).unwrap();
    std::fs::write(&sink.path, b"handwritten\n").unwrap();
    let store = lower_store(&f).await;
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        sink.clone(),
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (store, result) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(
        result.unwrap().accepted.body.outcome,
        ReceiptOutcome::Skipped
    );
    assert_eq!(std::fs::read(&sink.path).unwrap(), b"handwritten\n");
    assert!(!store.events.iter().any(|e| matches!(
        &e.event,
        Event::ItemCompleted {
            body: Item::Adoption { .. },
            ..
        }
    )));
    let store = lower_store(&f).await;
    let untrusted = bboxy450_harness::dream::resolve::Resolver::open(f.sessions.clone(), vec![])
        .await
        .unwrap();
    let mut untrusted = untrusted.with_receipt_domains(f.domains.clone());
    assert!(
        lower::prepare_at(
            &store,
            &mut untrusted,
            sink.clone(),
            "engine",
            None,
            &Set::default(),
            4005
        )
        .await
        .is_err()
    );
    let store = lower_store(&f).await;
    let target = context_sink(&f);
    std::os::unix::fs::symlink(&sink.path, &target.path).unwrap();
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        target.clone(),
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (_, result) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(
        result.unwrap().accepted.body.outcome,
        ReceiptOutcome::Skipped
    );
    assert_eq!(std::fs::read(&sink.path).unwrap(), b"handwritten\n");
}

#[test]
fn independent_filesystem_observer_process_child() {
    if std::env::var_os("DREAM_WP8_OBSERVER_CHILD").is_none() {
        return;
    }
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        use bboxy450_harness::dream::{lower, seal};
        let f = lower_fixture();
        let mut resolver = f.resolver().await;
        let store = lower_store(&f).await;
        let sink = context_sink(&f);
        let plan = lower::prepare_at(
            &store,
            &mut resolver,
            sink.clone(),
            "engine",
            None,
            &Set::default(),
            4005,
        )
        .await
        .unwrap();
        let marker = std::env::var_os("DREAM_WP8_OBSERVER_MARKER").unwrap();
        std::fs::write(
            &marker,
            f.fixture
                .repo
                .home
                .path()
                .canonicalize()
                .unwrap()
                .as_os_str()
                .as_encoded_bytes(),
        )
        .unwrap();
        let (_, result) = lower::publish(store, plan, seal::Owner::default(), Set::default())
            .unwrap()
            .settle()
            .await;
        let p = result.unwrap();
        let fault = std::env::var("DREAM_WP8_OBSERVER_FAULT").unwrap_or_default();
        if fault == "aba" {
            let original = std::fs::read(&sink.path).unwrap();
            std::fs::write(&sink.path, b"foreign intermediate bytes\n").unwrap();
            std::fs::write(&sink.path, &original).unwrap();
            assert_eq!(std::fs::read(&sink.path).unwrap(), original);
        }
        if fault == "hand_race" {
            assert_eq!(p.accepted.body.outcome, ReceiptOutcome::Skipped);
            assert_eq!(std::fs::read(&sink.path).unwrap(), b"handwritten\n");
        }
        std::fs::remove_file(marker).unwrap();
        let mut report = serde_json::to_value(&p.accepted.body).unwrap();
        report["observer_home"] = json!(f.fixture.repo.home.path().canonicalize().unwrap());
        report["observer_session"] = json!(p.accepted.body.run.session);
        std::fs::write(
            std::env::var_os("DREAM_WP8_OBSERVER_BODY").unwrap(),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
    });
}
fn observed_effects_match(trace: &str, body: &Value) -> bool {
    let ops = body["ops"].as_array().unwrap();
    let rows = trace
        .lines()
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let session = body["observer_session"].as_str().unwrap();
    if rows.is_empty()
        || rows
            .iter()
            .any(|r| r.len() != 6 || r[0] == "rename" || r[0] == "omitted_fsync")
    {
        return false;
    }
    for row in &rows {
        if row[0] == "write"
            || row[0] == "open"
                && row[5]
                    .parse::<i32>()
                    .is_ok_and(|f| f & (libc::O_WRONLY | libc::O_RDWR | libc::O_TRUNC) != 0)
        {
            let path = std::path::Path::new(row[2]);
            let temp = path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(".dream-tmp-"));
            let own = path
                .to_string_lossy()
                .contains(&format!("/{session}/blobs/"))
                || path
                    .file_name()
                    .is_some_and(|n| n == format!("{session}.jsonl").as_str());
            if !temp && !own {
                return false;
            }
        }
    }
    for op in ops
        .iter()
        .filter(|op| op["op"] == "link_noreplace" && op["result"] == "ok")
    {
        let from = op["from"].as_str().unwrap();
        let to = op["to"].as_str().unwrap();
        let parent = std::path::Path::new(to).parent().unwrap().to_str().unwrap();
        let Some(link) = rows
            .iter()
            .position(|r| r[0] == "link" && r[1] == "0" && r[2] == from && r[3] == to)
        else {
            return false;
        };
        let Some(write) = rows[..link]
            .iter()
            .position(|r| r[0] == "write" && r[2] == from)
        else {
            return false;
        };
        if !rows[write + 1..link]
            .iter()
            .any(|r| r[0] == "fsync" && r[1] == "0" && r[2] == from)
            || !rows[link + 1..]
                .iter()
                .any(|r| r[0] == "fsync" && r[1] == "0" && r[2] == parent)
        {
            return false;
        }
        if !rows[..write].iter().any(|r| {
            r[0] == "open"
                && r[1].parse::<i32>().is_ok_and(|fd| fd >= 0)
                && r[2] == from
                && r[5].parse::<i32>().is_ok_and(|flags| {
                    flags & (libc::O_CREAT | libc::O_EXCL) == (libc::O_CREAT | libc::O_EXCL)
                        && flags & libc::O_TRUNC == 0
                })
        }) {
            return false;
        }
    }
    for op in ops {
        let path = op["path"].as_str().unwrap_or("");
        match op["op"].as_str().unwrap() {
            "write_temp" if op["result"] == "ok" => {
                let found = rows
                    .iter()
                    .filter(|r| r[0] == "write" && r[2] == path)
                    .collect::<Vec<_>>();
                if found.len() != 1
                    || found[0][4] != op["sha256"].as_str().unwrap()
                    || found[0][5].parse::<u64>().ok() != op["len"].as_u64()
                {
                    return false;
                }
            }
            "fsync" | "fsync_dir"
                if op["result"] == "ok"
                    && !rows
                        .iter()
                        .any(|r| r[0] == "fsync" && r[1] == "0" && r[2] == path) =>
            {
                return false;
            }
            "link_noreplace"
                if op["result"] == "ok"
                    && !rows.iter().any(|r| {
                        r[0] == "link"
                            && r[1] == "0"
                            && Some(r[2]) == op["from"].as_str()
                            && Some(r[3]) == op["to"].as_str()
                    }) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}
#[test]
fn lower_independent_syscall_digest_observer_rejects_clobber_aba_unsynced_and_wrong_bytes() {
    let private = tempfile::tempdir_in("/tmp").unwrap();
    let lib = private.path().join("observer.dylib");
    let compile = std::process::Command::new("/usr/bin/clang")
        .args(["-dynamiclib", "-Wno-deprecated-declarations"])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/dream/sinks/observer.c"
        ))
        .arg("-o")
        .arg(&lib)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    for fault in ["good", "aba", "wrong_temp", "omit_dir_sync", "hand_race"] {
        let trace = private.path().join(format!("{fault}.trace"));
        let body = private.path().join(format!("{fault}.json"));
        let marker = private.path().join(format!("{fault}.enable"));
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "independent_filesystem_observer_process_child",
                "--nocapture",
            ])
            .env("DYLD_INSERT_LIBRARIES", &lib)
            .env("DREAM_WP8_OBSERVER_CHILD", "1")
            .env("DREAM_WP8_OBSERVER_MARKER", marker)
            .env("DREAM_WP8_OBSERVER_TRACE", &trace)
            .env("DREAM_WP8_OBSERVER_BODY", &body)
            .env("DREAM_WP8_OBSERVER_FAULT", fault)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{fault}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let raw = std::fs::read_to_string(trace).unwrap();
        let body: Value = serde_json::from_slice(&std::fs::read(body).unwrap()).unwrap();
        let observation = json!({"fault":fault,"accepted_by_independent_observer":observed_effects_match(&raw,&body),"producer_declared_outcome":body["outcome"],"raw_call_count":raw.lines().count(),"raw_trace_sha256":bboxy450_harness::dream::fingerprint::digest(&raw),"raw_trace_retention":"private temporary directory; bytes and secret/transcript content never recorded"});
        let run = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".bboxy/runs")
            .join(format!(
                "dream-wp8-observer-{fault}-{}.json",
                ulid::Ulid::new()
            ));
        std::fs::write(run, serde_json::to_vec_pretty(&observation).unwrap()).unwrap();
        assert_eq!(
            observed_effects_match(&raw, &body),
            matches!(fault, "good" | "hand_race"),
            "{fault}: {raw}"
        );
    }
}

#[tokio::test]
async fn lower_claude_index_codex_resources_full_identity_and_unsupported_grok() {
    use bboxy450_harness::dream::{lower, seal};
    let f = lower_fixture();
    let mut resolver = f.resolver().await;
    let home = f.fixture.repo.home.path().canonicalize().unwrap();
    let project = f
        .fixture
        .repo
        .path
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(['/', '.'], "-");
    let claude = SinkRef {
        kind: SinkKind::ClaudeMemory,
        path: home
            .join(".claude/projects")
            .join(project)
            .join("memory/MEMORY.md")
            .to_str()
            .unwrap()
            .into(),
    };
    let codex = SinkRef {
        kind: SinkKind::CodexExtension,
        path: home
            .join(".codex/memories/extensions/bboxy/instructions.md")
            .to_str()
            .unwrap()
            .into(),
    };
    for sink in [&claude, &codex] {
        let store = lower_store(&f).await;
        let plan = lower::prepare_at(
            &store,
            &mut resolver,
            sink.clone(),
            "engine",
            None,
            &Set::default(),
            4005,
        )
        .await
        .unwrap();
        let (store, p) = lower::publish(store, plan, seal::Owner::default(), Set::default())
            .unwrap()
            .settle()
            .await;
        let p = p.unwrap();
        assert_eq!(p.accepted.body.outcome, ReceiptOutcome::Published);
        assert_eq!(
            store
                .events
                .iter()
                .filter(|e| matches!(
                    &e.event,
                    Event::ItemCompleted {
                        body: Item::Adoption { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        let source = std::fs::read_to_string(&sink.path).unwrap();
        if sink.kind == SinkKind::ClaudeMemory {
            assert!(source.contains(&format!("dream:{}", f.entry)));
            let topic = std::path::Path::new(&sink.path)
                .parent()
                .unwrap()
                .join(format!("dream-{}.md", &f.entry[7..19]));
            assert!(
                std::fs::read_to_string(topic)
                    .unwrap()
                    .contains("metadata:")
            );
        } else {
            assert_eq!(source, bboxy450_harness::dream::receipt::CODEX_ADAPTER);
            let resource = std::path::Path::new(&sink.path)
                .parent()
                .unwrap()
                .join("resources")
                .join(format!("{}.md", &f.entry[7..19]));
            assert!(
                std::fs::read_to_string(&resource)
                    .unwrap()
                    .contains(&f.entry)
            );
            std::fs::write(resource, "same 12-hex filename, different full EntryId\n").unwrap();
        }
    }
    // Existing shared Claude index is skipped even when it happens to equal our bytes.
    let original = std::fs::read(&claude.path).unwrap();
    let store = lower_store(&f).await;
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        claude.clone(),
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (store, p) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(p.unwrap().accepted.body.outcome, ReceiptOutcome::Skipped);
    assert_eq!(std::fs::read(&claude.path).unwrap(), original);
    assert!(!store.events.iter().any(|e| matches!(
        &e.event,
        Event::ItemCompleted {
            body: Item::Adoption { .. },
            ..
        }
    )));
    let store = lower_store(&f).await;
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        codex.clone(),
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (_, p) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(p.unwrap().accepted.body.outcome, ReceiptOutcome::Skipped);
    let store = lower_store(&f).await;
    assert!(
        lower::prepare_at(
            &store,
            &mut resolver,
            codex,
            "untried-other-harness",
            None,
            &Set::default(),
            4005
        )
        .await
        .is_err()
    );
    let grok = SinkRef {
        kind: SinkKind::GrokInbox,
        path: home
            .join(".grok/memory-v2/observations/_inbox")
            .to_str()
            .unwrap()
            .into(),
    };
    assert!(matches!(
        lower::prepare_at(
            &store,
            &mut resolver,
            grok,
            "engine",
            None,
            &Set::default(),
            4005
        )
        .await,
        Err(bboxy450_harness::dream::seal::ProofError::Unproven(
            "native_archive_or_proposal_loader_unavailable"
        ))
    ));
}

#[test]
fn publication_abandoned_caller_process_child() {
    if std::env::var_os("DREAM_WP8_DROP_CHILD").is_none() {
        return;
    }
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        use bboxy450_harness::{
            dream::{lower, seal},
            runtime,
        };
        let f = lower_fixture();
        let mut resolver = f.resolver().await;
        let store = lower_store(&f).await;
        let session = store.session;
        let sink = context_sink(&f);
        let plan = lower::prepare_at(
            &store,
            &mut resolver,
            sink.clone(),
            "engine",
            None,
            &Set::default(),
            4005,
        )
        .await
        .unwrap();
        let (ready, wait) = tokio::sync::oneshot::channel();
        let ready = std::sync::Mutex::new(Some(ready));
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let b = barrier.clone();
        let observer = runtime::observe_blocking(move |_| {
            if let Some(tx) = ready.lock().unwrap().take() {
                let _ = tx.send(());
                b.wait();
            }
        });
        let outer = seal::Owner::default();
        let job = lower::publish(store, plan, outer.clone(), Set::default()).unwrap();
        let mut settled = job.settlement();
        wait.await.unwrap();
        drop(job);
        let mut prefix = r::start(r::sid(666), &f.fixture.repo.path, true);
        r::begin(&mut prefix, r::sid(666), r::item(667), ItemKind::Receipt);
        assert!(
            outer.freeze(r::item(667), r::sid(666), &prefix).is_err(),
            "owner must retain inflight/child proof after caller Drop"
        );
        // Release an explicitly held own blocking worker; no sleep/retry/timeout-as-success.
        barrier.wait();
        settled.changed().await.unwrap();
        assert!(*settled.borrow());
        drop(observer);
        assert!(
            !std::path::Path::new(&sink.path).exists(),
            "cancel before link must leave no publication"
        );
        let log = std::fs::read_to_string(bboxy450_harness::store::log_path(&f.sessions, session))
            .unwrap();
        assert!(log.contains("\"receipt_kind\":\"publication\""));
        assert!(!log.contains("\"kind\":\"adoption\""));
        assert!(outer.freeze(r::item(667), r::sid(666), &prefix).is_ok());
    });
}
#[test]
fn lower_dropped_caller_keeps_owned_work_until_real_settlement() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "publication_abandoned_caller_process_child",
            "--nocapture",
        ])
        .env("DREAM_WP8_DROP_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn lower_hardlinked_target_and_symlink_parent_preserve_foreign_names() {
    use bboxy450_harness::dream::{lower, seal};
    let f = lower_fixture();
    let mut resolver = f.resolver().await;
    let home = f.fixture.repo.home.path().canonicalize().unwrap();
    let foreign = home.join("handwritten");
    std::fs::write(&foreign, b"foreign unchanged\n").unwrap();
    let sink = context_sink(&f);
    std::fs::create_dir_all(std::path::Path::new(&sink.path).parent().unwrap()).unwrap();
    std::fs::hard_link(&foreign, &sink.path).unwrap();
    let store = lower_store(&f).await;
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        sink,
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (_, p) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(p.unwrap().accepted.body.outcome, ReceiptOutcome::Skipped);
    assert_eq!(std::fs::read(&foreign).unwrap(), b"foreign unchanged\n");
    let directory = home.join("foreign-directory");
    std::fs::create_dir(&directory).unwrap();
    std::os::unix::fs::symlink(&directory, home.join(".codex")).unwrap();
    let sink = SinkRef {
        kind: SinkKind::CodexExtension,
        path: home
            .join(".codex/memories/extensions/bboxy/instructions.md")
            .to_str()
            .unwrap()
            .into(),
    };
    let store = lower_store(&f).await;
    let plan = lower::prepare_at(
        &store,
        &mut resolver,
        sink,
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (_, p) = lower::publish(store, plan, seal::Owner::default(), Set::default())
        .unwrap()
        .settle()
        .await;
    assert_eq!(p.unwrap().accepted.body.outcome, ReceiptOutcome::Skipped);
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
}
