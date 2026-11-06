// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Adapted from the existing dream/good, dream/trust and dream/sinks scripted
//! builders. Root selection precedes every record/receipt signature. Completed
//! evidence is never renamed or rewritten. This is private fixture authority.
use super::support::sign_record;
use super::{fixture::Fixture, r, sinks, sufficient::AuditFixture, support};
use bboxy450_harness::{
    dream::{
        entry::{self, Identity},
        lower,
        receipt::Domains,
        resolve::{Resolver, Root},
        seal::Owner,
        trust::Producer,
    },
    protocol::*,
    redact::Set,
    store::Store,
};
use r::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, os::unix::fs::PermissionsExt, path::Path};

fn new_fixture(root: &str) -> Fixture {
    assert!(
        root.starts_with("fixture.invalid/dream/")
            && !root.contains("..")
            && root.split('/').count() == 3
    );
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join("repo");
    std::fs::create_dir(&path).unwrap();
    let policy = home.path().join("trust.json");
    let repo = support::Repository {
        home,
        path,
        commit: String::new(),
        bytes: vec![],
        key: support::key_pair(),
    };
    let gpg = tempfile::Builder::new()
        .prefix("dream-root-gpg-")
        .tempdir_in("/private/tmp")
        .unwrap();
    std::fs::set_permissions(gpg.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut f = Fixture {
        repo,
        gpg,
        policy,
        value: Value::Null,
        pin: EvidenceRef::Unknown(json!({})),
    };
    f.repo.git(&["init", "-q"]);
    f.repo.git(&["config", "user.name", "Boxy"]);
    f.repo.git(&["config", "user.email", "bboxy450@gmail.com"]);
    f.repo
        .git(&["remote", "add", "origin", &format!("https://{root}.git")]);
    f.gpg(&[
        "--quick-generate-key",
        "Boxy <bboxy450@gmail.com>",
        "ed25519",
        "sign",
        "0",
    ]);
    let listing = String::from_utf8(f.gpg(&["--with-colons", "--list-keys"])).unwrap();
    let fingerprint = listing
        .lines()
        .find(|line| line.starts_with("fpr:"))
        .unwrap()
        .split(':')
        .nth(9)
        .unwrap()
        .to_owned();
    let key = f.repo.home.path().join("public.asc");
    std::fs::write(&key, f.gpg(&["--armor", "--export", &fingerprint])).unwrap();
    let wrapper = f.repo.home.path().join("sign-gpg");
    std::fs::write(&wrapper,format!("#!/bin/sh\nexec /opt/homebrew/bin/gpg --no-options --homedir '{}' --batch --pinentry-mode loopback \"$@\"\n",f.gpg.path().display())).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    f.repo.git(&["config", "commit.gpgsign", "true"]);
    f.repo.git(&["config", "gpg.format", "openpgp"]);
    f.repo
        .git(&["config", "gpg.program", wrapper.to_str().unwrap()]);
    f.repo.git(&["config", "user.signingkey", &fingerprint]);
    for dir in ["decisions", "runs", "reports", "tasks"] {
        std::fs::create_dir_all(f.repo.path.join(".bboxy").join(dir)).unwrap();
    }
    std::fs::write(f.repo.path.join(".bboxy/dream.toml"), "cross_repo = true\n").unwrap();
    let anchor = support::sign_record(
        json!({"id":"01M00000000000000000000002","number":1,"type":"decision","title":"independently committed"}),
        "private root anchor\n",
        &f.repo.key,
    );
    f.repo.bytes = anchor.bytes.clone();
    std::fs::write(
        f.repo.path.join(".bboxy/decisions/my proof.md"),
        &anchor.bytes,
    )
    .unwrap();
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "private OpenPGP root"]);
    let commit = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo.commit = commit.clone();
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    f.pin = EvidenceRef::Record {
        root: root.into(),
        id: "01M00000000000000000000002".into(),
        path: ".bboxy/decisions/my proof.md".into(),
        commit,
        digest: entry::digest(&anchor.bytes),
    };
    f.value = json!({"schema":"bboxy.dream.trust/2","roots":[{"repo":root,"git_dir":f.repo.path,"branch":"refs/remotes/origin/main"}],"git_signers":[{"fingerprint":fingerprint,"public_key_file":key,"repos":[root],"revoked_after":null}],"kb_signer_kinds":["seat"],"producers":[]});
    f.write();
    f
}
// Adapted from https://github.com/bboxy450/bboxy450-harness/blob/main/tests/fixtures/dream/trust/fixture.rs.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn approve_fixture(f: &mut Fixture, root: &str) -> Producer {
    let source = f.repo.git(&["rev-parse", "HEAD"]);
    let tree = f.repo.git(&["rev-parse", "HEAD^{tree}"]);
    let exe =
        entry::digest(std::fs::read(std::env::current_exe().unwrap()).unwrap())[7..].to_owned();
    let producer = json!({"repo":root,"commit":source,"tree":tree,"version":"test/1","build":"test/private","exe_sha256":exe,"capabilities":["receipt/1","effect-wrapper/1","no-replace/1","constructive-seal/1"]});
    let mut gate = producer.clone();
    gate["schema"] = json!("bboxy.dream.producer-gate/1");
    gate["witnesses"] = json!([
            {"id":"dream_trust::receipt_header_body_roundtrip","exit_code":0},
            {"id":"dream_sinks::receipt_ops_match_independent_observer","exit_code":0},
            {"id":"dream_sinks::hand_write_between_read_and_link_preserved","exit_code":0},
            {"id":"dream_sinks::overwrite_rename_control_fails_audit","exit_code":0},
            {"id":"dream_trust::receipt_seal_constructs_without_self_hash","exit_code":0},
            {"id":"dream_sinks::late_owned_source_write_after_seal_rejected","exit_code":0},
            {"id":"dream_sinks::broken_seal_exception_fails_audit","exit_code":0}]);
    std::fs::create_dir_all(f.repo.path.join(".bboxy/runs")).unwrap();
    std::fs::create_dir_all(f.repo.path.join(".bboxy/reports")).unwrap();
    std::fs::create_dir_all(f.repo.path.join(".bboxy/tasks")).unwrap();
    let bytes = serde_json::to_vec(&gate).unwrap();
    std::fs::write(f.repo.path.join(".bboxy/runs/gate.json"), &bytes).unwrap();
    f.repo.git(&["add", "."]);
    f.repo
        .git(&["commit", "-qm", "private measured artifact gate"]);
    let g = f.repo.git(&["rev-parse", "HEAD"]);
    let report = sign_record(
        json!({"id":"01M00000000000000000000005","number":5,"type":"report","task":"[[01M00000000000000000000004]]","commits":[source,g]}),
        "isolated gate report; no live approval\n",
        &f.repo.key,
    );
    std::fs::write(f.repo.path.join(".bboxy/reports/gate.md"), &report.bytes).unwrap();
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "private full gate report"]);
    let r = f.repo.git(&["rev-parse", "HEAD"]);
    let task = sign_record(
        json!({"id":"01M00000000000000000000004","number":4,"type":"task","title":"fixture approval","status":"accepted","verify_result":{"exit_code":0},"report":{"commit":r,"report":"[[reports/gate]]"}}),
        "accepted private artifact fixture\n",
        &f.repo.key,
    );
    std::fs::write(f.repo.path.join(".bboxy/tasks/approval.md"), &task.bytes).unwrap();
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "private accepted gate task"]);
    let t = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo.git(&["update-ref", "refs/remotes/origin/main", &t]);
    let mut approval = producer.clone();
    approval["gate"] = json!({"commit":g,"path":".bboxy/runs/gate.json","sha256":&entry::digest(&bytes)[7..],"task":{"kind":"record","root":root,"id":"01M00000000000000000000004","path":".bboxy/tasks/approval.md","commit":t,"digest":entry::digest(&task.bytes)}});
    f.value["producers"] = json!([approval]);
    f.write();
    serde_json::from_value(producer).unwrap()
}
fn start(root: &str, session: SessionId, cwd: &Path, dream: bool) -> Vec<Envelope> {
    let meta = json!({"session":session,"cwd":cwd,"harness":if dream{"dream"}else{"scripted"},"profile":"scripted","created_ms":0,"fork":null,"protocol":{"major":1,"minor":3},"vcs":{"repo":root,"dirty":false},"dream":if dream{json!({"phase":"audit","trigger":{"kind":"operator","message":"private fixture"},"scope":{"level":"repo","repos":[root]}})}else{Value::Null}});
    let mut log = vec![];
    push(
        &mut log,
        session,
        Event::SessionStarted {
            meta: serde_json::from_value(meta).unwrap(),
        },
    );
    log
}
fn normal(f: &mut AuditFixture, root: &str, session: SessionId, exit: i32) {
    let mut log = start(root, session, &f.fixture.repo.path, false);
    let id = item(u128::from(session.0) + 10000);
    begin(&mut log, session, id, ItemKind::Terminal);
    completed(
        &mut log,
        session,
        id,
        Item::Terminal {
            terminal: "private-verify".into(),
            command: "test -f fixed".into(),
            exit: Some(exit),
            signal: None,
            output: None,
            live: None,
        },
    );
    push(&mut log, session, Event::SessionClosed);
    write_log(&f.sessions, session, &log);
    f.logs.insert(session, log);
}
// Adapted from https://github.com/bboxy450/bboxy450-harness/blob/main/tests/fixtures/dream/good/audit_fixture.rs.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub fn audit_fixture(root: &str) -> AuditFixture {
    let mut fixture = new_fixture(root);
    let producer = approve_fixture(&mut fixture, root);
    let sessions = fixture.repo.home.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let domains = Domains::configured(sessions.clone(), fixture.repo.home.path().into());
    let anchor = fixture.pin.clone();
    let identity = Identity {
        kind: "lesson".into(),
        title: "Run the private verifier".into(),
        why: "A checked source protects this fixture.".into(),
        how_to_apply: "Run the verifier before publication.".into(),
        evidence: vec![serde_json::to_value(&anchor).unwrap()],
        derived_from: vec![],
        revival: 0,
    };
    let entry = identity.id().unwrap();
    let body = format!(
        "# {}\n\n## Why\n{}\n\n## How to apply\n{}\n",
        identity.title, identity.why, identity.how_to_apply
    );
    let mut front = serde_json::to_value(&identity).unwrap();
    front["id"] = json!("01M00000000000000000000003");
    front["number"] = json!(3);
    front["type"] = json!("memory");
    front["description"] = json!("A verified private memory.");
    front["entry"] = json!(entry);
    front["content_sha256"] = json!(entry::digest(&body));
    front["status"] = json!("adopted");
    front["harnesses"] = json!(["engine"]);
    front["scope"] = json!({"level":"repo","repos":[root]});
    let record = support::sign_record(front.clone(), &body, &fixture.repo.key);
    std::fs::create_dir_all(fixture.repo.path.join(".bboxy/memories")).unwrap();
    std::fs::write(
        fixture.repo.path.join(".bboxy/memories/lesson.md"),
        &record.bytes,
    )
    .unwrap();
    fixture.repo.git(&["add", "."]);
    fixture
        .repo
        .git(&["commit", "-qm", "private adopted memory"]);
    let commit = fixture.repo.git(&["rev-parse", "HEAD"]);
    fixture
        .repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    let pin = RecordPin {
        root: root.into(),
        id: Some("01M00000000000000000000003".into()),
        path: ".bboxy/memories/lesson.md".into(),
        commit: Some(commit),
        digest: entry::digest(&record.bytes),
    };
    let mut f = AuditFixture {
        fixture,
        sessions,
        domains,
        entry: entry.clone(),
        logs: BTreeMap::new(),
        publications: vec![],
        producer,
        record: pin.clone(),
    };
    let build = sid(1);
    let mut log = start(root, build, &f.fixture.repo.path, true);
    for (id, body, kind) in [
        (
            item(10),
            Item::MemoryWrite {
                entry: entry.clone(),
                op: MemoryOp::Created,
                record: pin.clone(),
                evidence: vec![anchor],
                draft: None,
            },
            ItemKind::MemoryWrite,
        ),
        (
            item(11),
            Item::MemoryState {
                entry: entry.clone(),
                state: EntryState::Proposed,
                harness: None,
                reason: "fixture".into(),
                refs: vec![],
            },
            ItemKind::MemoryState,
        ),
    ] {
        begin(&mut log, build, id, kind);
        completed(&mut log, build, id, body);
    }
    f.finish(build, log, Owner::default(), vec![]);
    let mut next = 200;
    let mut origins = vec![];
    let mut run_ids = vec![];
    let mut episodes = vec![];
    let mut controls = vec![];
    for group in 0..6 {
        let source = sid(100 + group);
        normal(&mut f, root, source, 1);
        let line = &f.logs[&source][2];
        let origin = EvidenceRef::Event {
            session: source,
            seq: line.seq.unwrap(),
            digest: entry::digest(line.wire_bytes().unwrap().strip_suffix(b"\n").unwrap()),
        };
        origins.push(source);
        let mut runs = vec![];
        for arm in [Arm::With, Arm::Without] {
            for index in 0..if group < 4 { 2 } else { 1 } {
                let home = tempfile::tempdir().unwrap();
                if arm == Arm::With || group >= 4 || index == 0 {
                    std::fs::write(home.path().join("fixed"), b"ok").unwrap();
                }
                let code = std::process::Command::new("/bin/sh")
                    .args(["-c", "test -f fixed"])
                    .current_dir(home.path())
                    .status()
                    .unwrap()
                    .code()
                    .unwrap();
                let session = sid(next);
                next += 1;
                normal(&mut f, root, session, code);
                run_ids.push(session);
                runs.push(TrialRun {
                    arm: arm.clone(),
                    session,
                    verify_exit: Some(code),
                    usage: Some(Usage {
                        input_tokens: 100,
                        output_tokens: 10,
                        source: Some(NumberSource::Reported),
                        model: Some("scripted".into()),
                        backend: Some(Box::new(Backend {
                            kind: BackendKind::Engine,
                            name: "scripted".into(),
                            version: None,
                        })),
                        ..Default::default()
                    }),
                });
            }
        }
        let episode = TrialEpisode { origin, runs };
        if group < 4 {
            episodes.push(episode)
        } else {
            controls.push(episode)
        }
    }
    let trial = sid(2);
    let mut log = start(root, trial, &f.fixture.repo.path, true);
    begin(&mut log, trial, item(20), ItemKind::Trial);
    completed(
        &mut log,
        trial,
        item(20),
        Item::Trial {
            entry: entry.clone(),
            harness: "engine".into(),
            thresholds: "dream-thresholds@2".into(),
            episodes,
            controls,
            verdict: Verdict::Pass,
            reason: "real private verifier with/without runs".into(),
        },
    );
    let trial_pin = EvidenceRef::Event {
        session: trial,
        seq: Seq(3),
        digest: entry::digest(log[2].wire_bytes().unwrap().strip_suffix(b"\n").unwrap()),
    };
    origins.extend(run_ids);
    let sources = f.sources(&origins);
    f.finish(trial, log, Owner::default(), sources);
    let adopt = sid(3);
    let mut log = start(root, adopt, &f.fixture.repo.path, true);
    for (id, state, refs) in [
        (item(30), EntryState::Tested, vec![]),
        (item(31), EntryState::Adopted, vec![trial_pin]),
    ] {
        begin(&mut log, adopt, id, ItemKind::MemoryState);
        completed(
            &mut log,
            adopt,
            id,
            Item::MemoryState {
                entry: entry.clone(),
                state,
                harness: Some("engine".into()),
                reason: "independent trial pin".into(),
                refs,
            },
        );
    }
    let sources = f.sources(&[trial]);
    f.finish(adopt, log, Owner::default(), sources);

    let home = f.fixture.repo.home.path();
    let canonical = home.canonicalize().unwrap();
    f.domains
        .sink_roots
        .extend(f.domains.sink_roots.clone().iter().filter_map(|p| {
            p.strip_prefix(home)
                .ok()
                .map(|suffix| canonical.join(suffix))
        }));
    f
}
pub async fn resolver(f: &AuditFixture, root: &str) -> Resolver {
    let selected = Root::open(root.into(), f.fixture.repo.path.clone())
        .await
        .unwrap();
    Resolver::open(f.sessions.clone(), vec![selected])
        .await
        .unwrap()
        .with_trust(f.fixture.load().await.unwrap())
        .with_receipt_domains(f.domains.clone())
}
fn context_sink(f: &AuditFixture, root: &str) -> SinkRef {
    SinkRef {
        kind: SinkKind::EngineContext,
        path: f
            .fixture
            .repo
            .home
            .path()
            .canonicalize()
            .unwrap()
            .join(".bboxy450-harness/dream/context")
            .join(&bboxy450_harness::dream::fingerprint::digest(root)[..16])
            .join(format!("{}.md", ulid::Ulid::new()))
            .to_str()
            .unwrap()
            .into(),
    }
}
pub struct Completed {
    pub fixture: AuditFixture,
    pub logs: BTreeMap<SessionId, Vec<Envelope>>,
    pub publication: Envelope,
}
pub async fn completed_publication(root: &str) -> Completed {
    let f = audit_fixture(root);
    let mut selected = resolver(&f, root).await;
    let session = SessionId::new();
    let Event::SessionStarted { mut meta } = start(root, sid(444), &f.fixture.repo.path, true)[0]
        .event
        .clone()
    else {
        unreachable!()
    };
    meta.session = session;
    meta.dream.as_mut().unwrap().phase = DreamPhase::Lower;
    let store = Store::open(f.sessions.clone(), meta, None, Set::default())
        .await
        .unwrap();
    let owner = Owner::default();
    let plan = lower::prepare_at(
        &store,
        &mut selected,
        context_sink(&f, root),
        "engine",
        None,
        &Set::default(),
        4005,
    )
    .await
    .unwrap();
    let (store, published) = lower::publish(store, plan, owner.clone(), Set::default())
        .unwrap()
        .settle()
        .await;
    let item = published.unwrap().accepted.item;
    let (store, _) = sinks::finish_run(store, &f, &owner).await;
    let publication = store
        .events
        .iter()
        .find(|event| matches!(&event.event,Event::ItemCompleted{item:found,..}if *found==item))
        .unwrap()
        .clone();
    let mut logs = (*selected.logs).clone();
    logs.insert(store.session, store.events.clone());
    Completed {
        fixture: f,
        logs,
        publication,
    }
}
