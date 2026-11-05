// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        receipt::{self, Body, Domains},
        seal::{Child, ChildCoverage, ChildState, Owner, ProofError},
    },
    protocol::*,
};
use serde_json::json;
#[allow(dead_code)]
#[path = "fixtures/dream/trust/fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "fixtures/dream/good/receipt_support.rs"]
mod r;
#[allow(dead_code)]
#[path = "fixtures/dream/good/support.rs"]
mod support;
use r::*;

#[tokio::test]
async fn receipt_seal_constructs_without_self_hash_and_exact_suffix_roundtrips() {
    let mut f = fixture::Fixture::new();
    let producer = f.approve_fixture();
    let session = sid(1000);
    let id = item(1001);
    let root = f.repo.home.path().join("sessions");
    std::fs::create_dir(&root).unwrap();
    let mut log = start(session, &f.repo.path, true);
    begin(&mut log, session, id, ItemKind::Receipt);
    let owner = Owner::default();
    let (body, bytes, carrier) = run_body(session, producer)
        .freeze(&owner, id, &log, &bboxy450_harness::redact::Set::default())
        .unwrap();
    assert!(
        !body
            .ops
            .iter()
            .any(|o| o["sha256"] == json!(bboxy450_harness::dream::fingerprint::digest(&bytes))),
        "no self-hash operation"
    );
    assert_eq!(Body::parse(&bytes).unwrap().encode().unwrap(), bytes);
    assert_eq!(
        owner.admit().err(),
        Some(ProofError::Fail("late_protected_effect"))
    );
    write_blob(&root, session, &carrier, &bytes);
    completed(&mut log, session, id, carrier);
    push(&mut log, session, Event::SessionClosed);
    let domains = Domains::configured(root, f.repo.home.path().into());
    let trust = f.load().await.unwrap();
    let accepted = receipt::accept(&log[2], &log, &domains, Some(&trust))
        .await
        .unwrap();
    receipt::check_sources(&accepted, &[], &std::collections::BTreeMap::new()).unwrap();
    for (field, value, expected) in [
        (
            "version",
            json!(2),
            ProofError::Unproven("unknown_seal_coverage"),
        ),
        ("effect_count", json!(1), ProofError::Fail("effect_count")),
        (
            "receipt_item",
            json!(item(55)),
            ProofError::Fail("seal_item_mismatch"),
        ),
    ] {
        let mut bad = body.clone();
        let mut seal = serde_json::to_value(&bad.seal).unwrap();
        seal[field] = value;
        bad.seal = Some(serde_json::from_value(seal).unwrap());
        let encoded = bad.encode().unwrap();
        let result = Body::parse(&encoded).and_then(|b| {
            b.seal.as_ref().unwrap().validate(
                b.coverage.as_ref().unwrap(),
                &[],
                &log,
                session,
                id,
                None,
            )
        });
        assert_eq!(result.unwrap_err(), expected);
    }
    let mut bad = log.clone();
    push(
        &mut bad,
        session,
        Event::ProfileChanged {
            profile: "prohibited detached hook".into(),
            model: "scripted".into(),
        },
    );
    assert_eq!(
        body.seal
            .as_ref()
            .unwrap()
            .validate(
                body.coverage.as_ref().unwrap(),
                &[],
                &bad,
                session,
                id,
                None
            )
            .unwrap_err(),
        ProofError::Fail("forbidden_suffix")
    );
    let mut bad = body.clone();
    bad.seal.as_mut().unwrap().own_prefix.prefix_digest = "sha256:00".into();
    assert_eq!(
        bad.seal
            .as_ref()
            .unwrap()
            .validate(bad.coverage.as_ref().unwrap(), &[], &log, session, id, None)
            .unwrap_err(),
        ProofError::Fail("own_prefix_mismatch")
    );
}

#[test]
fn freeze_requires_observed_quiescence_and_completed_admissions() {
    let session = sid(99);
    let id = item(9);
    let mut log = start(session, std::path::Path::new("/tmp"), true);
    begin(&mut log, session, id, ItemKind::Receipt);
    let owner = Owner::default();
    let pending = owner.admit().unwrap();
    assert_eq!(
        owner.freeze(id, session, &log).err(),
        Some(ProofError::Unproven("inflight_or_unobserved_effect"))
    );
    drop(pending);
    assert!(
        owner.freeze(id, session, &log).is_err(),
        "dropped work cannot silently become observed"
    );
    let owner = Owner::default();
    let child = ulid::Ulid::new();
    owner
        .child(Child {
            id: child,
            state: ChildState::Active,
            coverage: ChildCoverage::Observed,
        })
        .unwrap();
    assert_eq!(
        owner.freeze(id, session, &log).err(),
        Some(ProofError::Unproven("owned_child_active"))
    );
    owner
        .child(Child {
            id: child,
            state: ChildState::Joined,
            coverage: ChildCoverage::Unobserved,
        })
        .unwrap();
    assert_eq!(
        owner.freeze(id, session, &log).err(),
        Some(ProofError::Unproven("owned_child_unobserved"))
    );
}

#[test]
#[ignore = "private child process fixture, explicitly invoked by the process witness"]
fn owned_child_fixture() {
    use std::io::{BufRead, Write};
    let Some(path) = std::env::var_os("DREAM_SEAL_CHILD_SOURCE") else {
        return;
    };
    println!("READY");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input).unwrap();
    if input.trim() == "allow" {
        let original = std::fs::read(&path).unwrap();
        std::fs::write(&path, b"changed").unwrap();
        println!("WRITE");
        std::io::stdout().flush().unwrap();
        std::fs::write(&path, original).unwrap();
        println!("RESTORE");
    }
}
#[tokio::test]
async fn real_delayed_child_is_unproven_and_broken_exception_exposes_aba() {
    use std::{
        io::{BufRead, Write},
        process::{Command, Stdio},
        time::Duration,
    };
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let source_session = sid(2);
    let mut source_log = start(source_session, temp.path(), false);
    push(&mut source_log, source_session, Event::SessionClosed);
    write_log(&sessions, source_session, &source_log);
    let source = sessions.join(format!("{source_session}.jsonl"));
    let original = std::fs::read(&source).unwrap();
    for broken in [false, true] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "owned_child_fixture", "--ignored", "--nocapture"])
            .env_clear()
            .env("DREAM_SEAL_CHILD_SOURCE", &source)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let _ = tx.send(line.unwrap());
            }
        });
        loop {
            if rx.recv_timeout(Duration::from_secs(5)).unwrap() == "READY" {
                break;
            }
        }
        let owner = Owner::default();
        let id = ulid::Ulid::new();
        owner
            .child(Child {
                id,
                state: ChildState::Active,
                coverage: ChildCoverage::Observed,
            })
            .unwrap();
        let session = sid(if broken { 4 } else { 3 });
        let item = item(10 + u128::from(session.0));
        let mut log = start(session, temp.path(), true);
        begin(&mut log, session, item, ItemKind::Receipt);
        assert_eq!(
            owner.freeze(item, session, &log).err(),
            Some(ProofError::Unproven("owned_child_active"))
        );
        writeln!(
            child.stdin.take().unwrap(),
            "{}",
            if broken { "allow" } else { "deny" }
        )
        .unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(s) = child.try_wait().unwrap() {
                break s;
            }
            if std::time::Instant::now() > until {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("owned child {pid} deadline");
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        assert!(status.success());
        reader.join().unwrap();
        let observed = rx.try_iter().collect::<Vec<_>>();
        assert_eq!(std::fs::read(&source).unwrap(), original);
        owner
            .child(Child {
                id,
                state: ChildState::Joined,
                coverage: ChildCoverage::Observed,
            })
            .unwrap();
        let (seal, _) = owner.freeze(item, session, &log).unwrap();
        assert!(owner.admit().is_err());
        let domains = Domains::configured(sessions.clone(), temp.path().into());
        let ops = if broken {
            assert!(
                observed.iter().any(|l| l == "WRITE") && observed.iter().any(|l| l == "RESTORE")
            );
            vec![json!({"i":0,"op":"cache_write","path":source,"result":"ok"})]
        } else {
            assert!(!observed.iter().any(|l| l == "WRITE"));
            vec![]
        };
        assert_eq!(
            domains.source_write(session, &ops),
            broken,
            "actual observed ABA cannot be excused by equal bytes or an own-custody exception"
        );
        // Independent outer observation records the broken publication exception's
        // actual source write before the final run freeze. It is not reconstructed
        // from equal source bytes or from a disappeared child's joined row.
        let producer = bboxy450_harness::dream::trust::Producer {
            repo: "fixture.invalid/dream/repo".into(),
            commit: "0".repeat(40),
            tree: "1".repeat(40),
            version: "unapproved-test".into(),
            build: "private".into(),
            exe_sha256: "2".repeat(64),
            capabilities: vec![],
        };
        let mut body = run_body(session, producer);
        body.seal = Some(seal);
        body.coverage = Some(bboxy450_harness::dream::seal::Coverage::default());
        body.ops = ops;
        body.seal.as_mut().unwrap().effect_count = body.ops.len() as u64;
        let bytes = body.encode().unwrap();
        let blob = BlobRef {
            sha256: bboxy450_harness::dream::fingerprint::digest(&bytes),
            len: bytes.len() as u64,
            mime: receipt::MIME.into(),
        };
        let carrier = Item::Receipt {
            schema: receipt::SCHEMA.into(),
            kind: ReceiptKind::Run,
            outcome: ReceiptOutcome::Completed,
            harness: None,
            sink: None,
            blob: Some(blob),
        };
        write_blob(&sessions, session, &carrier, &bytes);
        completed(&mut log, session, item, carrier);
        push(&mut log, session, Event::SessionClosed);
        write_log(&sessions, session, &log);
        let mut resolver =
            bboxy450_harness::dream::resolve::Resolver::open(sessions.clone(), vec![])
                .await
                .unwrap()
                .with_receipt_domains(domains);
        let audit = bboxy450_harness::dream::audit::run(
            &mut resolver,
            bboxy450_harness::redact::Set::default(),
            false,
            0,
        )
        .await
        .unwrap();
        assert_eq!(
            audit.findings.iter().any(|f| f.check == "P4"
                && f.kind == "violation"
                && f.detail == "writer_touched_source"),
            broken
        );
        assert_eq!(
            audit.exit_code(),
            1,
            "unapproved producer must never be silently certified"
        );
    }
}

#[allow(dead_code)]
#[path = "fixtures/dream/good/audit_fixture.rs"]
mod sufficient;
#[tokio::test]
async fn retained_seal_cut_selection_and_outer_coverage_controls_reject() {
    use bboxy450_harness::dream::seal::ProofError::{Fail, Unproven};
    use std::collections::BTreeSet;
    let fixture = sufficient::AuditFixture::new(false);
    let mut resolver = fixture.resolver().await;
    let trust = fixture.fixture.load().await.unwrap();
    let (session, index, body) = &fixture.publications[0];
    let log = &fixture.logs[session];
    let accepted = receipt::accept(&log[*index], log, &fixture.domains, Some(&trust))
        .await
        .unwrap();
    let mut bad = body.clone();
    bad.seal.as_mut().unwrap().children = vec![Child {
        id: ulid::Ulid::new(),
        state: ChildState::Joined,
        coverage: ChildCoverage::Unobserved,
    }];
    assert_eq!(
        bad.seal
            .as_ref()
            .unwrap()
            .validate(
                bad.coverage.as_ref().unwrap(),
                &receipt::parse_effects(&bad.ops).unwrap(),
                log,
                *session,
                accepted.item,
                Some(1)
            )
            .unwrap_err(),
        Unproven("owned_child_unobserved")
    );
    let mut bad = body.clone();
    let child = Child {
        id: ulid::Ulid::new(),
        state: ChildState::Joined,
        coverage: ChildCoverage::Observed,
    };
    bad.seal.as_mut().unwrap().children = vec![child.clone(), child];
    assert_eq!(
        bad.seal
            .as_ref()
            .unwrap()
            .validate(
                bad.coverage.as_ref().unwrap(),
                &receipt::parse_effects(&bad.ops).unwrap(),
                log,
                *session,
                accepted.item,
                Some(1)
            )
            .unwrap_err(),
        Fail("child_order")
    );
    let mut bad = accepted.clone();
    bad.body.inventory.as_mut().unwrap().sessions.pop();
    assert_eq!(
        receipt::check_publication(&bad, &mut resolver, &fixture.domains, &BTreeSet::new())
            .await
            .unwrap_err(),
        Fail("ignored_log")
    );
    let mut bad = accepted.clone();
    bad.body.cut.as_mut().unwrap()[0].prefix_digest = "sha256:wrong".into();
    assert_eq!(
        receipt::check_publication(&bad, &mut resolver, &fixture.domains, &BTreeSet::new())
            .await
            .unwrap_err(),
        Unproven("stale_cut")
    );
    for (case, expected) in [
        (0, "projection_mismatch"),
        (1, "candidate_rank_pin"),
        (2, "candidate_rank_pin"),
        (3, "budget_subset"),
        (4, "block_digest"),
        (5, "generated_binding"),
    ] {
        let mut bad = accepted.clone();
        match case {
            0 => bad.body.projection_digest = Some("sha256:wrong".into()),
            1 => bad.body.selection.as_mut().unwrap().candidates[0].rank += 1.0,
            2 => {
                bad.body.selection.as_mut().unwrap().candidates[0]
                    .record
                    .digest = "sha256:wrong".into()
            }
            3 => bad.body.selection.as_mut().unwrap().selected.clear(),
            4 => bad.body.selection.as_mut().unwrap().block_digest = "sha256:wrong".into(),
            _ => bad.body.selection.as_mut().unwrap().generated[0].sha256 = "0".repeat(64),
        }
        assert_eq!(
            receipt::check_publication(&bad, &mut resolver, &fixture.domains, &BTreeSet::new())
                .await
                .unwrap_err(),
            Fail(expected)
        );
    }
    let mut missing = serde_json::to_value(body).unwrap();
    missing.as_object_mut().unwrap().remove("seal");
    let bytes = bboxy450_harness::dream::entry::canonical_value(missing).unwrap();
    assert_eq!(Body::parse(&bytes).unwrap_err(), Unproven("missing_seal"));
    let mut bad = accepted.clone();
    bad.body.producer.exe_sha256 = "f".repeat(64);
    assert!(
        trust
            .approve(&bad.body.producer, &["constructive-seal/1"])
            .await
            .is_err()
    );
    let blob = fixture
        .sessions
        .join(session.to_string())
        .join("blobs")
        .join(&accepted.blob.sha256);
    let bytes = std::fs::read(&blob).unwrap();
    std::fs::remove_file(&blob).unwrap();
    assert_eq!(
        receipt::accept(&log[*index], log, &fixture.domains, Some(&trust))
            .await
            .unwrap_err(),
        Unproven("lost_witness")
    );
    std::fs::write(&blob, &bytes[..bytes.len() / 2]).unwrap();
    assert_eq!(
        receipt::accept(&log[*index], log, &fixture.domains, Some(&trust))
            .await
            .unwrap_err(),
        Unproven("truncated_witness")
    );
    std::fs::write(&blob, bytes).unwrap();
    let run_event = log
        .iter()
        .find(|e| {
            matches!(
                &e.event,
                Event::ItemCompleted {
                    body: Item::Receipt {
                        kind: ReceiptKind::Run,
                        ..
                    },
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(
        receipt::accept(run_event, log, &fixture.domains, Some(&trust))
            .await
            .unwrap_err(),
        Unproven("publication_effects_unobserved")
    );
    let pubs = fixture
        .publications
        .iter()
        .map(|(_, i, _)| *i)
        .collect::<Vec<_>>();
    let mut receipts = vec![];
    for i in pubs {
        receipts.push(
            receipt::accept(&log[i], log, &fixture.domains, Some(&trust))
                .await
                .unwrap(),
        );
    }
    let run = receipt::accept_with_publications(
        run_event,
        log,
        &fixture.domains,
        Some(&trust),
        &receipts,
    )
    .await
    .unwrap();
    receipt::check_outer(&accepted, &run).unwrap();
    let mut bad = run.clone();
    bad.body.ops.retain(|o| o["op"] != "write_own_blob");
    assert_eq!(
        receipt::check_outer(&accepted, &bad).unwrap_err(),
        Unproven("publication_seal_unobserved")
    );
    let mut bad = run.clone();
    bad.body.sources.as_mut().unwrap()[0].prefix_digest = "sha256:wrong".into();
    assert_eq!(
        receipt::check_sources(&bad, &[], &fixture.logs).unwrap_err(),
        Unproven("source_mutated")
    );
    let own = bboxy450_harness::dream::entry::digest(b"own blob disguise");
    let ops = vec![
        json!({"i":0,"op":"write_own_blob","sha256":own,"path":fixture.sessions.join("source.jsonl"),"result":"ok"}),
    ];
    assert!(fixture.domains.source_write(*session, &ops));
}

#[tokio::test]
async fn own_receipt_custody_is_atomic_noreplace_and_preserves_a_conflict() {
    use bboxy450_harness::{
        dream::{entry, seal},
        redact::Set,
        store::Store,
    };
    let temp = tempfile::tempdir().unwrap();
    let session = sid(900);
    let item = item(901);
    let header = start(session, temp.path(), true);
    let Event::SessionStarted { meta } = &header[0].event else {
        unreachable!()
    };
    let mut store = Store::open(temp.path().into(), meta.clone(), None, Set::default())
        .await
        .unwrap();
    store
        .append(
            Event::ItemStarted {
                item,
                kind: ItemKind::Receipt,
                turn: None,
                parent: None,
                head: None,
            },
            None,
        )
        .await
        .unwrap();
    let producer = bboxy450_harness::dream::trust::Producer {
        repo: "fixture.invalid/dream/repo".into(),
        commit: "0".repeat(40),
        tree: "1".repeat(40),
        version: "unapproved-fixture".into(),
        build: "private".into(),
        exe_sha256: "2".repeat(64),
        capabilities: vec![],
    };
    let (body, bytes, carrier) = run_body(session, producer)
        .freeze(&Owner::default(), item, &store.events, &Set::default())
        .unwrap();
    let Item::Receipt {
        blob: Some(blob), ..
    } = &carrier
    else {
        unreachable!()
    };
    let blob = blob.clone();
    let (store, body, bytes, blob) = bboxy450_harness::runtime::blocking(move || {
        let result = seal::persist(&store, &body, &bytes, &blob, &Set::default()).unwrap();
        assert_eq!(result, blob);
        assert_eq!(
            entry::digest(std::fs::read(store.blobs.dir.join(&blob.sha256)).unwrap()),
            format!("sha256:{}", blob.sha256)
        );
        seal::persist(&store, &body, &bytes, &blob, &Set::default()).unwrap();
        std::fs::write(store.blobs.dir.join(&blob.sha256), b"foreign bytes").unwrap();
        assert_eq!(
            seal::persist(&store, &body, &bytes, &blob, &Set::default()).unwrap_err(),
            ProofError::Unproven("seal_blob_conflict")
        );
        assert_eq!(
            std::fs::read(store.blobs.dir.join(&blob.sha256)).unwrap(),
            b"foreign bytes"
        );
        assert!(!std::fs::read_dir(&store.blobs.dir).unwrap().any(|f| {
            f.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".dream-tmp-")
        }));
        (store, body, bytes, blob)
    })
    .await;
    assert_eq!(body.run.session, store.session);
    assert_eq!(bytes.len() as u64, blob.len);
}

#[test]
fn registered_data_cannot_be_sealed_or_silently_redacted_into_different_pins() {
    let session = sid(980);
    let id = item(981);
    let mut log = start(session, std::path::Path::new("/tmp"), true);
    begin(&mut log, session, id, ItemKind::Receipt);
    let producer = bboxy450_harness::dream::trust::Producer {
        repo: "fixture.invalid/dream/repo".into(),
        commit: "0".repeat(40),
        tree: "1".repeat(40),
        version: "registered-private-value".into(),
        build: "private".into(),
        exe_sha256: "2".repeat(64),
        capabilities: vec![],
    };
    let set = bboxy450_harness::redact::Set::default();
    set.register("registered-private-value");
    assert_eq!(
        run_body(session, producer)
            .freeze(&Owner::default(), id, &log, &set)
            .unwrap_err(),
        ProofError::Fail("denied_receipt")
    );
}

#[test]
fn native_resource_shapes_bind_full_entries_and_fixed_adapter_bytes() {
    use bboxy450_harness::dream::{
        entry::{self, Identity},
        receipt::{CODEX_ADAPTER, Candidate, render_resources},
    };
    let identity: Identity = serde_json::from_value(
        serde_json::from_str::<serde_json::Value>(include_str!(
            "fixtures/dream/good/identity-vectors.json"
        ))
        .unwrap()["identity"]
            .clone(),
    )
    .unwrap();
    let id = identity.id().unwrap();
    let mut front = serde_json::to_value(&identity).unwrap();
    front["type"] = json!("memory");
    front["id"] = json!("01M00000000000000000000003");
    front["entry"] = json!(id);
    front["description"] = json!("Private format vector.");
    front["scope"] = json!({"level":"repo","repos":["fixture.invalid/dream/repo"]});
    front["content_sha256"] = json!(entry::digest("body\n"));
    let record = support::sign_record(front, "body\n", &support::key_pair());
    let candidate = Candidate {
        entry: id.clone(),
        record: RecordPin {
            root: "fixture.invalid/dream/repo".into(),
            id: Some("01M00000000000000000000003".into()),
            path: ".bboxy/memories/vector.md".into(),
            commit: Some("0".repeat(40)),
            digest: entry::digest(&record.bytes),
        },
        rank: 1.0,
        lines: 1,
        bytes: 0,
    };
    for (kind, path) in [
        (
            SinkKind::ClaudeMemory,
            "/private/home/.claude/projects/fixture/memory/MEMORY.md",
        ),
        (
            SinkKind::CodexExtension,
            "/private/home/.codex/memories/extensions/bboxy",
        ),
        (
            SinkKind::GrokInbox,
            "/private/home/.grok/memory-v2/global/observations/_inbox",
        ),
        (
            SinkKind::Proposal,
            "/private/home/.bboxy450-harness/dream/proposals",
        ),
    ] {
        let sink = SinkRef {
            kind: kind.clone(),
            path: path.into(),
        };
        let (block, files) =
            render_resources(&[(candidate.clone(), record.clone())], &sink, sid(99), 0).unwrap();
        assert!(!block.is_empty());
        assert!(
            files
                .values()
                .any(|v| String::from_utf8_lossy(v).contains(&id))
        );
        for bytes in files.values() {
            bboxy450_harness::dream::deny::text(std::str::from_utf8(bytes).unwrap()).unwrap();
        }
        if kind == SinkKind::CodexExtension {
            assert_eq!(
                files
                    .get(&std::path::Path::new(path).join("instructions.md"))
                    .unwrap(),
                CODEX_ADAPTER.as_bytes()
            );
        }
    }
}
