// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        entry,
        graph::{self, Graph, Memory},
        projection::{self, Fact, Projection},
    },
    protocol::*,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn session(n: u128) -> SessionId {
    SessionId(ulid::Ulid::from(n))
}
fn pin(n: u64) -> EvidenceRef {
    EvidenceRef::Event {
        session: session(1),
        seq: Seq(n),
        digest: format!(
            "sha256:{}",
            format!("{n:x}")
                .repeat(64)
                .chars()
                .take(64)
                .collect::<String>()
        ),
    }
}
fn fact(n: u64, body: Item) -> Fact {
    Fact::unproved(pin(n), session(1), Seq(n), 1000 - n, body)
}
fn usage() -> Usage {
    Usage {
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
    }
}
fn trial(entry: &str, harness: &str) -> Item {
    let mut next = 100u128;
    let mut episode = |n: u64, control: bool| {
        let mut runs = Vec::new();
        for arm in [Arm::With, Arm::Without] {
            for index in 0..if control { 1 } else { 2 } {
                next += 1;
                runs.push(TrialRun {
                    arm: arm.clone(),
                    session: session(next),
                    verify_exit: Some(if matches!(arm, Arm::With) || control || index == 0 {
                        0
                    } else {
                        1
                    }),
                    usage: Some(usage()),
                });
            }
        }
        TrialEpisode {
            origin: pin(n),
            runs,
        }
    };
    Item::Trial {
        entry: entry.into(),
        harness: harness.into(),
        thresholds: "dream-thresholds@2".into(),
        episodes: (10..14).map(|n| episode(n, false)).collect(),
        controls: (20..22).map(|n| episode(n, true)).collect(),
        verdict: Verdict::Pass,
        reason: "scripted held-out comparison".into(),
    }
}
fn state(
    n: u64,
    entry: &str,
    harness: Option<&str>,
    state: EntryState,
    refs: Vec<EvidenceRef>,
) -> Fact {
    fact(
        n,
        Item::MemoryState {
            entry: entry.into(),
            state,
            harness: harness.map(str::to_owned),
            reason: format!("reason-{n}"),
            refs,
        },
    )
}
fn passed(entry: &str, harness: &str) -> Vec<Fact> {
    vec![
        state(1, entry, None, EntryState::Proposed, vec![]),
        fact(2, trial(entry, harness)),
        state(3, entry, Some(harness), EntryState::Tested, vec![]),
        state(4, entry, Some(harness), EntryState::Adopted, vec![pin(2)]),
    ]
}

#[test]
fn projection_ignores_clocks_order_duplicates_and_cross_harness_rejection() {
    let entry = format!("sha256:{}", "1".repeat(64));
    let mut facts = passed(&entry, "engine");
    facts.push(state(
        5,
        &entry,
        Some("codex"),
        EntryState::Rejected,
        vec![],
    ));
    let expected = Projection::build(&facts).unwrap();
    let e = &expected.entries[&entry];
    assert_eq!(e.harnesses(), vec!["engine"]);
    assert_eq!(e.status(), "adopted");
    assert_eq!(e.pairs["codex"].state(), "rejected");
    for offset in 0..facts.len() {
        let mut reordered = facts.clone();
        reordered.rotate_left(offset);
        reordered.reverse();
        for fact in &mut reordered {
            fact.at = u64::MAX - fact.seq.0;
        }
        reordered.extend(facts.clone());
        assert_eq!(
            Projection::build(&reordered).unwrap().entries,
            expected.entries
        );
    }
    facts.push(state(6, &entry, None, EntryState::Expired, vec![]));
    facts.push(state(
        7,
        &entry,
        Some("engine"),
        EntryState::Adopted,
        vec![pin(2)],
    ));
    let projection = Projection::build(&facts).unwrap();
    assert_eq!(projection.entries[&entry].status(), "expired");
    assert!(projection.entries[&entry].harnesses().is_empty());
    facts.push(state(8, &entry, None, EntryState::Superseded, vec![]));
    facts.reverse();
    assert_eq!(
        Projection::build(&facts).unwrap().entries[&entry].status(),
        "superseded"
    );
}

#[test]
fn pair_terminal_reasons_survive_and_inconclusive_remains_tested() {
    let entry = format!("sha256:{}", "1".repeat(64));
    let mut facts = passed(&entry, "engine");
    facts.push(state(
        5,
        &entry,
        Some("engine"),
        EntryState::Rejected,
        vec![],
    ));
    facts.push(state(
        6,
        &entry,
        Some("engine"),
        EntryState::Retired,
        vec![],
    ));
    facts.push(state(7, &entry, Some("codex"), EntryState::Tested, vec![]));
    let mut inconclusive = trial(&entry, "codex");
    if let Item::Trial { verdict, .. } = &mut inconclusive {
        *verdict = Verdict::Inconclusive;
    }
    facts.push(fact(8, inconclusive));
    let p = Projection::build(&facts).unwrap();
    let e = &p.entries[&entry];
    assert_eq!(e.status(), "tested");
    assert_eq!(e.pairs["engine"].state(), "retired");
    assert_eq!(e.pairs["engine"].reasons.len(), 2);
    assert!(!e.active("engine"));
    assert!(!e.active("codex"));
}

#[test]
fn adoption_cannot_borrow_a_harness_revision_or_unknown_threshold() {
    let entry = format!("sha256:{}", "1".repeat(64));
    for mode in [0, 1, 2, 3] {
        let mut facts = passed(&entry, "engine");
        match mode {
            0 => {
                if let Item::Trial { harness, .. } = &mut facts[1].body {
                    *harness = "codex".into();
                }
            }
            1 => {
                if let Item::Trial { entry, .. } = &mut facts[1].body {
                    *entry = format!("sha256:{}", "2".repeat(64));
                }
            }
            2 => {
                if let Item::Trial { thresholds, .. } = &mut facts[1].body {
                    *thresholds = "dream-thresholds@future".into();
                }
            }
            _ => {
                if let Item::MemoryState { refs, .. } = &mut facts[3].body {
                    *refs = vec![pin(999)];
                }
            }
        }
        assert!(Projection::build(&facts).is_err(), "mode {mode}");
    }
    let mut old = trial(&entry, "engine");
    if let Item::Trial { thresholds, .. } = &mut old {
        *thresholds = "dream-thresholds@1".into();
    }
    assert_eq!(projection::qualifying_trial(&old).unwrap(), 4);
    let mut future = passed(&entry, "engine");
    future[1].seq = Seq(10);
    future[1].pin = pin(10);
    if let Item::MemoryState { refs, .. } = &mut future[3].body {
        *refs = vec![pin(10)];
    }
    assert!(
        Projection::build(&future)
            .unwrap_err()
            .0
            .contains("causally prior")
    );
}

#[test]
fn numeric_pass_cannot_fill_missing_cost_duplicate_held_out_or_harm_controls() {
    let entry = format!("sha256:{}", "1".repeat(64));
    for mode in 0..6 {
        let mut trial = trial(&entry, "engine");
        if let Item::Trial {
            episodes, controls, ..
        } = &mut trial
        {
            match mode {
                0 => episodes[0].runs[0].usage = None,
                1 => episodes[1].origin = episodes[0].origin.clone(),
                2 => controls[0].runs[0].verify_exit = Some(1),
                3 => episodes[0].runs[0].usage.as_mut().unwrap().source = None,
                4 => {
                    for episode in episodes {
                        for run in &mut episode.runs {
                            if matches!(run.arm, Arm::With) {
                                run.usage.as_mut().unwrap().input_tokens = 1000;
                            }
                        }
                    }
                }
                _ => episodes[0].runs[0].verify_exit = None,
            }
        }
        assert!(projection::qualifying_trial(&trial).is_err(), "mode {mode}");
    }
}

#[test]
fn durable_union_rejects_conflicting_raw_identity_and_keeps_unknown_bytes() {
    let v: Value = serde_json::from_str(
        include_str!("fixtures/ir-v1/v1.3-dream-session.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let mut line = serde_json::to_string(&v).unwrap();
    line.push('\n');
    let original = Envelope::from_line(&line).unwrap();
    let mut unknown = v.clone();
    unknown["future_field"] = json!({"unconsumed":"retained"});
    let bytes = format!("{}\n", serde_json::to_string(&unknown).unwrap());
    let changed = Envelope::from_line(&bytes).unwrap();
    assert!(projection::union([original.clone(), changed.clone()]).is_err());
    let kept = projection::union([changed.clone(), changed]).unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].wire_bytes().unwrap(), bytes.as_bytes());
    assert_eq!(
        projection::union([original.clone(), original])
            .unwrap()
            .len(),
        1
    );
}

fn memory(root: &str, path: &str, title: &str, links: Value) -> Memory {
    let v: Value =
        serde_json::from_str(include_str!("fixtures/dream/good/identity-vectors.json")).unwrap();
    let mut identity: entry::Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    identity.title = title.into();
    identity.derived_from.clear();
    let body = format!(
        "# {}\n\n## Why\n{}\n\n## How to apply\n{}\n",
        identity.title, identity.why, identity.how_to_apply
    );
    let mut front = serde_json::to_value(&identity).unwrap();
    front["type"] = json!("memory");
    front["entry"] = json!(identity.id().unwrap());
    front["content_sha256"] = json!(entry::digest(&body));
    front["links"] = links;
    Memory {
        root: root.into(),
        path: path.into(),
        record: entry::Record::parse(
            format!("---\n{}---\n{body}", serde_yaml::to_string(&front).unwrap()).as_bytes(),
        )
        .unwrap(),
    }
}

#[test]
fn cross_root_links_match_whole_authorized_names_and_keep_external_edges() {
    let a = memory(
        "host/owner/repo",
        ".bboxy/memories/a.md",
        "A",
        json!([
        {"to":"[[kb:host/owner/repo/memories/b]]","rel":"refines","weight":2},
        {"to":"[[kb:agents/memories/b]]","rel":"related","weight":1}]),
    );
    let b = memory("host/owner/repo", ".bboxy/memories/b.md", "B", json!([]));
    let other = memory("host/owner", ".bboxy/memories/b.md", "other B", json!([]));
    let a_id = a.record.identity().unwrap().id().unwrap();
    let b_id = b.record.identity().unwrap().id().unwrap();
    let graph = Graph::build(
        &[a, b, other],
        &[],
        &BTreeMap::new(),
        &Projection::default(),
        &BTreeSet::new(),
        false,
        0,
    )
    .unwrap();
    assert!(
        graph
            .edges
            .iter()
            .any(|e| e.from == a_id && e.to == b_id && !e.external && e.weight == 2.0)
    );
    assert!(
        graph
            .edges
            .iter()
            .any(|e| e.from == a_id && e.to == "[[kb:agents/memories/b]]" && e.external)
    );
    let why = graph.why(&a_id).unwrap();
    assert!(why.nodes.contains_key(&b_id));
    assert!(why.nodes.contains_key("[[kb:agents/memories/b]]"));
}

#[test]
fn graph_refuses_invalid_weights_and_strength_is_only_a_read_projection() {
    let bad = memory(
        "repo",
        ".bboxy/memories/a.md",
        "A",
        json!([{"to":"[[missing]]","weight":0}]),
    );
    assert!(
        Graph::build(
            &[bad],
            &[],
            &BTreeMap::new(),
            &Projection::default(),
            &BTreeSet::new(),
            false,
            0
        )
        .is_err()
    );
    assert!((graph::node_strength(4.0, 0.0, 30.0) - 3.0 / std::f64::consts::E).abs() < 1e-12);
    assert_eq!(graph::node_strength(10.0, 0.0, 0.0), 3.0);
    assert!(
        (graph::edge_strength(2.0, [0.0, 30.0], 30.0)
            - (2.0 + 0.25 * (1.0 + 1.0 / std::f64::consts::E)))
            .abs()
            < 1e-12
    );
    assert_eq!(graph::edge_strength(9.9, [0.0, 0.0], 0.0), 10.0);
}

#[test]
fn net_gain_keeps_failed_held_out_observations() {
    let id = entry::digest(b"gain fixture");
    let positive = trial(&id, "engine");
    assert_eq!(projection::trial_gain(&positive).unwrap(), 4);
    let mut failed = positive.clone();
    if let Item::Trial {
        episodes, verdict, ..
    } = &mut failed
    {
        *verdict = Verdict::Fail;
        for episode in episodes {
            for run in &mut episode.runs {
                run.verify_exit = Some(if run.arm == Arm::Without { 0 } else { 1 });
            }
        }
    }
    assert!(projection::qualifying_trial(&failed).is_err());
    assert_eq!(
        projection::trial_gain(&failed).unwrap(),
        -8,
        "failing adoption gates cannot erase negative gain"
    );
    if let Item::Trial { episodes, .. } = &mut failed {
        episodes[0].runs[0].verify_exit = None;
    }
    assert!(projection::trial_gain(&failed).is_err());
}

async fn retained_derived_fixture() -> (tempfile::TempDir, BTreeMap<SessionId, Vec<Envelope>>, Item)
{
    use bboxy450_harness::{redact::Set, store::Store};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("sessions");
    let meta = SessionMeta {
        session: SessionId::new(),
        cwd: temp.path().into(),
        profile: None,
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: 0,
        vcs: None,
        dream: None,
        source: None,
    };
    let mut parent = Store::open(root.clone(), meta.clone(), None, Set::default())
        .await
        .unwrap();
    parent
        .append(
            Event::ProfileChanged {
                profile: "scripted".into(),
                model: "scripted".into(),
            },
            None,
        )
        .await
        .unwrap();
    parent.append(Event::SessionClosed, None).await.unwrap();
    let mut logs = BTreeMap::from([(meta.session, parent.events.clone())]);
    drop(parent);
    let mut trial = trial(&format!("sha256:{}", "a".repeat(64)), "scripted");
    let Item::Trial {
        episodes, controls, ..
    } = &mut trial
    else {
        unreachable!()
    };
    for run in episodes
        .iter_mut()
        .chain(controls)
        .flat_map(|e| &mut e.runs)
    {
        let mut meta = meta.clone();
        meta.session = SessionId::new();
        meta.fork = Some((logs.keys().next().copied().unwrap(), Seq(2)));
        let mut child = Store::open(root.clone(), meta.clone(), None, Set::default())
            .await
            .unwrap();
        let turn = TurnId::new();
        child
            .append(Event::TurnQueued { turn }, None)
            .await
            .unwrap();
        child
            .append(
                Event::TurnStarted {
                    turn,
                    model: Some("scripted".into()),
                    origin: Some(TurnOrigin::User),
                },
                None,
            )
            .await
            .unwrap();
        let mut calls = vec![];
        for _ in 0..2 {
            let mut usage = usage();
            usage.scope = Some(UsageScope::Call);
            calls.push(usage.clone());
            let item = ItemId::new();
            child
                .append(
                    Event::ItemStarted {
                        turn: Some(turn),
                        item,
                        kind: ItemKind::Usage,
                        parent: None,
                        head: None,
                    },
                    None,
                )
                .await
                .unwrap();
            child
                .append(
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body: Item::Usage { usage },
                    },
                    None,
                )
                .await
                .unwrap();
        }
        let usage = Usage::sum_calls(&calls).unwrap();
        child
            .append(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::EndTurn,
                    usage: usage.clone(),
                },
                None,
            )
            .await
            .unwrap();
        child.append(Event::SessionClosed, None).await.unwrap();
        run.session = meta.session;
        run.usage = usage;
        logs.insert(
            meta.session,
            std::fs::read_to_string(root.join(format!("{}.jsonl", meta.session)))
                .unwrap()
                .split_inclusive('\n')
                .map(Envelope::from_line)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        );
    }
    (temp, logs, trial)
}
fn retained_trial_fact(item: Item, logs: &BTreeMap<SessionId, Vec<Envelope>>) -> Fact {
    let envelope = Envelope {
        raw: None,
        ext: Default::default(),
        native: None,
        v: PROTOCOL,
        session: session(90),
        epoch: Epoch(1),
        seq: Some(Seq(2)),
        at: Seq(2),
        ts_ms: 0,
        cause: None,
        event: Event::ItemCompleted {
            item: ItemId::new(),
            status: ItemStatus::Completed,
            body: item,
        },
    };
    projection::facts_with_logs(&[envelope], logs)
        .unwrap()
        .remove(0)
}
#[tokio::test]
async fn derived_qualification_binds_closed_fork_call_leaves_and_current_fact() {
    let (_temp, logs, trial) = retained_derived_fixture().await;
    let checked = retained_trial_fact(trial.clone(), &logs);
    assert!(
        checked.usage_error().is_none(),
        "{:?}",
        checked.usage_error()
    );
    assert_eq!(checked.qualifying_trial().unwrap(), 4);
    assert_eq!(checked.trial_gain().unwrap(), 4);
    assert!(projection::qualifying_trial(&trial).is_err());
    assert!(projection::trial_gain(&trial).is_err());
    assert!(
        !serde_json::to_string(&checked)
            .unwrap()
            .contains("checked_usage")
    );
    let mut changed = checked.clone();
    if let Item::Trial { episodes, .. } = &mut changed.body {
        episodes[0].runs[0].session = session(999);
    }
    assert!(matches!(
        changed.usage_error(),
        Some(projection::UsageProofError::Mismatch(_))
    ));
    assert!(changed.qualifying_trial().is_err());
    for harness_change in [false, true] {
        let mut foreign = checked.clone();
        if let Item::Trial { entry, harness, .. } = &mut foreign.body {
            if harness_change {
                *harness = "foreign-harness".into();
            } else {
                *entry = entry::digest(b"different-entry");
            }
        }
        assert!(matches!(
            foreign.usage_error(),
            Some(projection::UsageProofError::Mismatch(_))
        ));
        assert!(foreign.qualifying_trial().is_err());
    }
    let Item::Trial { episodes, .. } = &trial else {
        unreachable!()
    };
    let target = episodes[0].runs[0].session;
    for mode in 0..13 {
        let mut context = logs.clone();
        let mut item = trial.clone();
        if mode == 0 {
            context.remove(&target);
        } else {
            let log = context.get_mut(&target).unwrap();
            for e in log.iter_mut() {
                e.raw = None;
            }
            match mode {
                1 => {
                    log.pop();
                }
                2 => {
                    if let Event::SessionStarted { meta } = &mut log[0].event {
                        meta.fork = Some((session(999), Seq(2)));
                    }
                }
                3..=8 => {
                    let usages = log
                        .iter_mut()
                        .filter_map(|e| {
                            if let Event::ItemCompleted {
                                body: Item::Usage { usage },
                                ..
                            } = &mut e.event
                            {
                                Some(usage)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let first = usages.into_iter().next().unwrap();
                    match mode {
                        3 => first.unreported.push("input_tokens".into()),
                        4 => first.source = Some(NumberSource::Estimated),
                        5 => first.model = Some("different".into()),
                        6 => first.backend.as_mut().unwrap().name = "different".into(),
                        7 => first.source = Some(NumberSource::Derived),
                        8 => first.input_tokens = u64::MAX,
                        _ => unreachable!(),
                    }
                }
                9 => {
                    if let Item::Trial { episodes, .. } = &mut item {
                        episodes[0].runs[0].usage.as_mut().unwrap().reasoning_tokens = Some(99);
                    }
                }
                10 => {
                    let event = log
                        .iter_mut()
                        .find(|e| {
                            matches!(
                                e.event,
                                Event::ItemCompleted {
                                    body: Item::Usage { .. },
                                    ..
                                }
                            )
                        })
                        .unwrap();
                    event.session = session(999);
                }
                11 => {
                    let original = log
                        .iter()
                        .find(|e| {
                            matches!(
                                e.event,
                                Event::ItemCompleted {
                                    body: Item::Usage { .. },
                                    ..
                                }
                            )
                        })
                        .unwrap()
                        .clone();
                    let end = log
                        .iter()
                        .position(|e| matches!(e.event, Event::TurnEnded { .. }))
                        .unwrap();
                    log.insert(end, original);
                }
                12 => {
                    if let Item::Trial { episodes, .. } = &mut item {
                        episodes[0].runs[0].usage.as_mut().unwrap().source =
                            Some(NumberSource::Reported);
                    }
                }
                _ => unreachable!(),
            }
        }
        let fact = retained_trial_fact(item, &context);
        let error = fact.usage_error();
        assert!(error.is_some(), "mode {mode}");
        assert!(fact.qualifying_trial().is_err(), "mode {mode}");
        assert!(fact.trial_gain().is_err(), "mode {mode}");
        if matches!(mode, 0 | 1 | 2 | 3 | 7 | 8) {
            assert!(
                matches!(error, Some(projection::UsageProofError::Unavailable(_))),
                "mode {mode}: {error:?}"
            );
        } else {
            assert!(
                matches!(error, Some(projection::UsageProofError::Mismatch(_))),
                "mode {mode}: {error:?}"
            );
        }
    }
}
