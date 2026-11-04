// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::{fixture::Fixture, r, support};
use bboxy450_harness::{
    dream::{
        entry::{self, Identity},
        projection::{self, Projection},
        receipt::{
            self, Body, Budget, Candidate, Domains, Generated, Inventory, InventorySession,
            Selection, Source,
        },
        seal::{Owner, Prefix},
        trust::Producer,
    },
    protocol::*,
};
use r::*;
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};
pub struct AuditFixture {
    pub fixture: Fixture,
    pub sessions: PathBuf,
    pub domains: Domains,
    pub entry: String,
    pub logs: BTreeMap<SessionId, Vec<Envelope>>,
    pub publications: Vec<(SessionId, usize, Body)>,
    pub producer: Producer,
    pub record: RecordPin,
}
impl AuditFixture {
    pub fn finish(
        &mut self,
        session: SessionId,
        mut log: Vec<Envelope>,
        owner: Owner,
        sources: Vec<Source>,
    ) {
        let id = item(90000 + u128::from(session.0));
        begin(&mut log, session, id, ItemKind::Receipt);
        let mut body = run_body(session, self.producer.clone());
        body.sources = Some(sources);
        let (_, bytes, carrier) = body
            .freeze(&owner, id, &log, &bboxy450_harness::redact::Set::default())
            .unwrap();
        write_blob(&self.sessions, session, &carrier, &bytes);
        completed(&mut log, session, id, carrier);
        push(&mut log, session, Event::SessionClosed);
        assert!(
            laws::check_transcript(&log).is_ok(),
            "{session}: {:?}",
            laws::check_transcript(&log)
        );
        write_log(&self.sessions, session, &log);
        self.logs.insert(session, log);
    }
    pub fn sources(&self, ids: &[SessionId]) -> Vec<Source> {
        let mut ids = ids.to_vec();
        ids.sort();
        ids.iter()
            .map(|s| {
                let log = &self.logs[s];
                Source {
                    session: *s,
                    through_seq: log.len() as u64,
                    prefix_digest: bboxy450_harness::dream::seal::prefix(log, log.len() as u64)
                        .unwrap(),
                    mode: "read_only".into(),
                }
            })
            .collect()
    }
    pub fn normal(&mut self, session: SessionId, exit: i32) {
        let mut log = start(session, &self.fixture.repo.path, false);
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
        write_log(&self.sessions, session, &log);
        self.logs.insert(session, log);
    }
    pub fn new(stale: bool) -> Self {
        Self::build(stale, false, |body| vec![body])
    }
    pub fn skipped() -> Self {
        Self::build(false, true, |body| vec![body])
    }
    pub fn with_adoptions(transform: fn(Item) -> Vec<Item>) -> Self {
        Self::build(false, false, transform)
    }
    fn build(stale: bool, skipped: bool, transform: fn(Item) -> Vec<Item>) -> Self {
        let mut fixture = Fixture::new();
        let producer = fixture.approve_fixture();
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
        front["scope"] = json!({"level":"repo","repos":["fixture.invalid/dream/repo"]});
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
            root: "fixture.invalid/dream/repo".into(),
            id: Some("01M00000000000000000000003".into()),
            path: ".bboxy/memories/lesson.md".into(),
            commit: Some(commit),
            digest: entry::digest(&record.bytes),
        };
        let mut f = Self {
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
        let mut log = start(build, &f.fixture.repo.path, true);
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
            f.normal(source, 1);
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
                    f.normal(session, code);
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
        let mut log = start(trial, &f.fixture.repo.path, true);
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
        let mut log = start(adopt, &f.fixture.repo.path, true);
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
        let publication = sid(4);
        let mut log = start(publication, &f.fixture.repo.path, true);
        let run_owner = Owner::default();
        let target = f
            .fixture
            .repo
            .home
            .path()
            .join(".bboxy450-harness/dream/context")
            .join(&bboxy450_harness::dream::fingerprint::digest("fixture.invalid/dream/repo")[..16])
            .join(format!("{}.md", item(40)));
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        for (which, outcome) in [
            (0, ReceiptOutcome::Published),
            (1, ReceiptOutcome::Idempotent),
        ] {
            let id = item(40 + which);
            begin(&mut log, publication, id, ItemKind::Receipt);
            let cuts = f
                .logs
                .iter()
                .map(|(session, events)| Prefix {
                    session: *session,
                    through_seq: events.len() as u64,
                    prefix_digest: bboxy450_harness::dream::seal::prefix(
                        events,
                        events.len() as u64,
                    )
                    .unwrap(),
                })
                .collect::<Vec<_>>();
            let all = f.logs.values().flatten().cloned().collect::<Vec<_>>();
            let facts = projection::facts(&all).unwrap();
            let projection = Projection::build(&facts).unwrap();
            let at = u128::from(publication.0) as u64 * 1000 + log.len() as u64;
            let rank = bboxy450_harness::dream::graph::node_strength(
                4.0,
                3005.0 / 86400000.0,
                at as f64 / 86400000.0,
            );
            let mut candidate = Candidate {
                entry: entry.clone(),
                record: pin.clone(),
                rank,
                lines: 1,
                bytes: 0,
            };
            let sink = SinkRef {
                kind: SinkKind::EngineContext,
                path: target.to_str().unwrap().into(),
            };
            let block =
                receipt::render_index(&[(candidate.clone(), record.clone())], &sink.kind).unwrap();
            candidate.bytes =
                (block.len() - receipt::render_index(&[], &sink.kind).unwrap().len()) as u64;
            let hash = bboxy450_harness::dream::fingerprint::digest(&block);
            let temp = target.with_file_name(format!(".dream-tmp-{}", ulid::Ulid::new()));
            let ops = if skipped {
                vec![]
            } else if which == 0 {
                vec![
                    json!({"i":0,"op":"read","path":target,"result":"absent"}),
                    json!({"i":1,"op":"create_temp","path":temp,"result":"ok"}),
                    json!({"i":2,"op":"write_temp","path":temp,"sha256":hash,"len":block.len(),"result":"ok"}),
                    json!({"i":3,"op":"fsync","path":temp,"result":"ok"}),
                    json!({"i":4,"op":"link_noreplace","from":temp,"to":target,"result":"ok"}),
                    json!({"i":5,"op":"fsync_dir","path":target.parent(),"result":"ok"}),
                    json!({"i":6,"op":"unlink_own_temp","path":temp,"result":"ok"}),
                ]
            } else {
                vec![
                    json!({"i":0,"op":"read","path":target,"result":"present","sha256":hash,"len":block.len()}),
                ]
            };
            let owner = Owner::default();
            for effect in receipt::parse_effects(&ops).unwrap() {
                owner
                    .admit()
                    .unwrap()
                    .finish(effect.operation.clone())
                    .unwrap();
                run_owner.admit().unwrap().finish(effect.operation).unwrap();
            }
            // Fixture materialization uses actual no-replace/link and fsync, with an independent byte observation.
            if !skipped && which == 0 {
                use std::io::Write;
                let mut fd = std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&temp)
                    .unwrap();
                fd.write_all(&block).unwrap();
                fd.sync_all().unwrap();
                std::fs::hard_link(&temp, &target).unwrap();
                std::fs::File::open(target.parent().unwrap())
                    .unwrap()
                    .sync_all()
                    .unwrap();
                std::fs::remove_file(&temp).unwrap();
            } else if !skipped {
                assert_eq!(std::fs::read(&target).unwrap(), block);
            }
            let mut body = run_body(publication, f.producer.clone());
            body.kind = ReceiptKind::Publication;
            body.outcome = if skipped {
                ReceiptOutcome::Skipped
            } else {
                outcome
            };
            if skipped {
                body.skip_reason = Some("private publication refusal".into());
            }
            body.harness = Some("engine".into());
            body.sink = Some(sink.clone());
            body.sources = None;
            body.cut = Some(cuts.clone());
            body.inventory = Some(Inventory {
                root: f.sessions.clone(),
                sessions: cuts
                    .iter()
                    .map(|c| InventorySession {
                        session: c.session,
                        through_seq: c.through_seq,
                    })
                    .collect(),
            });
            body.projection_digest = Some(entry::digest(
                entry::canonical_value(serde_json::to_value(projection).unwrap()).unwrap(),
            ));
            body.selection = Some(Selection {
                thresholds: "dream-thresholds@2".into(),
                budget: Budget {
                    lines: 40,
                    bytes: 8192,
                },
                candidates: vec![candidate],
                selected: vec![entry.clone()],
                block_digest: entry::digest(&block),
                generated: vec![Generated {
                    path: target.clone(),
                    sha256: hash,
                    len: block.len() as u64,
                }],
            });
            let (body, bytes, carrier) = body
                .freeze(&owner, id, &log, &bboxy450_harness::redact::Set::default())
                .unwrap();
            let Item::Receipt {
                blob: Some(blob), ..
            } = &carrier
            else {
                unreachable!()
            };
            let sha = blob.sha256.clone();
            write_blob(&f.sessions, publication, &carrier, &bytes);
            let index = log.len();
            completed(&mut log, publication, id, carrier);
            if !skipped {
                let body = Item::Adoption {
                    entry: entry.clone(),
                    harness: "engine".into(),
                    sink,
                    record: Some(pin.clone()),
                    block_digest: entry::digest(&block),
                    receipt: Some(format!("sha256:{sha}")),
                };
                for (n, body) in transform(body).into_iter().enumerate() {
                    let adoption = item(60 + which + (n as u128) * 1000);
                    begin(&mut log, publication, adoption, ItemKind::Adoption);
                    completed(&mut log, publication, adoption, body);
                }
            }
            run_owner
                .admit()
                .unwrap()
                .finish(receipt::Operation::WriteOwnBlob {
                    sha256: sha,
                    result: receipt::ResultCode::Ok,
                })
                .unwrap();
            run_owner
                .admit()
                .unwrap()
                .finish(receipt::Operation::AppendOwnLog {
                    session: publication,
                    seq_from: body.seal.as_ref().unwrap().own_prefix.through_seq,
                    seq_to: log.len() as u64,
                    result: receipt::ResultCode::Ok,
                })
                .unwrap();
            f.publications.push((publication, index, body));
        }
        let source_ids = f.logs.keys().copied().collect::<Vec<_>>();
        let sources = f.sources(&source_ids);
        f.finish(publication, log, run_owner, sources);
        if stale {
            front["status"] = json!("expired");
            front["harnesses"] = json!([]);
            let changed = support::sign_record(front, &body, &f.fixture.repo.key);
            std::fs::write(
                f.fixture.repo.path.join(".bboxy/memories/lesson.md"),
                &changed.bytes,
            )
            .unwrap();
            f.fixture.repo.git(&["add", "."]);
            f.fixture
                .repo
                .git(&["commit", "-qm", "private terminal projection metadata"]);
            let commit = f.fixture.repo.git(&["rev-parse", "HEAD"]);
            f.fixture
                .repo
                .git(&["update-ref", "refs/remotes/origin/main", &commit]);
            let terminal = sid(5);
            let mut log = start(terminal, &f.fixture.repo.path, true);
            begin(&mut log, terminal, item(70), ItemKind::MemoryState);
            completed(
                &mut log,
                terminal,
                item(70),
                Item::MemoryState {
                    entry: entry.clone(),
                    state: EntryState::Expired,
                    harness: None,
                    reason: "terminal outside publication cut".into(),
                    refs: vec![],
                },
            );
            f.finish(terminal, log, Owner::default(), vec![]);
        }
        f
    }
    pub fn add_reported_recall(&mut self) {
        let session = sid(6);
        let mut log = start(session, &self.fixture.repo.path, false);
        let body = &self.publications[0].2;
        begin(&mut log, session, item(80), ItemKind::Recall);
        completed(
            &mut log,
            session,
            item(80),
            Item::Recall {
                entry: self.entry.clone(),
                basis: RecallBasis::Reported,
                record: Some(self.record.clone()),
                sink: body.sink.clone(),
                block_digest: Some(body.selection.as_ref().unwrap().block_digest.clone()),
            },
        );
        push(&mut log, session, Event::SessionClosed);
        write_log(&self.sessions, session, &log);
        self.logs.insert(session, log);
    }
    pub async fn resolver(&self) -> bboxy450_harness::dream::resolve::Resolver {
        let root = bboxy450_harness::dream::resolve::Root::open(
            "fixture.invalid/dream/repo".into(),
            self.fixture.repo.path.clone(),
        )
        .await
        .unwrap();
        bboxy450_harness::dream::resolve::Resolver::open(self.sessions.clone(), vec![root])
            .await
            .unwrap()
            .with_trust(self.fixture.load().await.unwrap())
            .with_receipt_domains(self.domains.clone())
    }
}
