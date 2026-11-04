// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Read-only audit over a captured union. Missing historical authority is UNPROVEN.
use super::{
    deny,
    entry::{self, Identity, Invalid, Record},
    graph::{Graph, Memory},
    projection::{self, Projection, RecordView},
    resolve::{ResolvedRecord, Resolver, Root},
};
use crate::{protocol::*, redact::Set};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub check: String,
    pub kind: String,
    pub detail: String,
}
#[derive(Default, Serialize)]
pub struct Audit {
    pub findings: Vec<Finding>,
    pub projection: Projection,
    pub graph: Graph,
    pub record_views: BTreeMap<String, RecordView>,
}
impl Audit {
    fn reject(&mut self, check: &str, detail: &str) {
        self.findings.push(Finding {
            check: check.into(),
            kind: "violation".into(),
            detail: detail.into(),
        });
    }
    fn unproven(&mut self, check: &str, detail: &str) {
        self.findings.push(Finding {
            check: check.into(),
            kind: "unproven".into(),
            detail: detail.into(),
        });
    }
    pub fn exit_code(&self) -> i32 {
        if self.findings.is_empty() { 0 } else { 1 }
    }
}
struct Info {
    record: Arc<ResolvedRecord>,
    identity: Identity,
    evidence: Vec<EvidenceRef>,
}
fn pin_key(pin: &EvidenceRef) -> Result<String, Invalid> {
    let mut value = serde_json::to_value(pin).map_err(|_| Invalid("pin encoding"))?;
    value.sort_all_objects();
    serde_json::to_string(&value).map_err(|_| Invalid("pin encoding"))
}
async fn inspect(
    record: Arc<ResolvedRecord>,
    resolver: &mut Resolver,
    audit: &mut Audit,
    set: &Set,
) -> Option<(String, Info)> {
    let source = record.clone();
    let redact = set.clone();
    let parsed = crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let safe = deny::value(&source.record.front, &redact).is_ok()
            && redact.scrub(&source.record.body) == source.record.body
            && deny::text(&source.record.body).is_ok();
        (
            source
                .record
                .validate_identity()
                .and_then(|identity| Ok((identity.id()?, identity))),
            safe,
        )
    })
    .await;
    if !parsed.1 {
        audit.reject("P8", "record contains denied or registered secret data");
    }
    let (id, identity) = match parsed.0 {
        Ok(pair) => pair,
        Err(error) => {
            audit.reject("P9", error.0);
            return None;
        }
    };
    let mut evidence = Vec::new();
    for value in &identity.evidence {
        match resolver.evidence_value(&record.pin.root, value).await {
            Ok(pin) => evidence.push(pin),
            Err(error) => audit.unproven("P1", error.0),
        }
    }
    let source = record.clone();
    match crate::runtime::blocking(move || source.record.scope()).await {
        Ok(scope) => {
            if let Err(error) = resolver.scope(&scope, &evidence).await {
                audit.reject("P7", error.0);
            }
        }
        Err(error) => audit.reject("P7", error.0),
    }
    Some((
        id,
        Info {
            record,
            identity,
            evidence,
        },
    ))
}

/// Resolve only the supplied normal sessions and explicitly mounted KB roots.
/// No audit path takes a writer lease, repairs bytes, or interprets a cache as evidence.
pub async fn run(
    resolver: &mut Resolver,
    set: Set,
    with_strength: bool,
    now_ms: u64,
) -> Result<Audit, Invalid> {
    let logs = resolver.logs.clone();
    let (envelopes, metas, transcript_bad) = crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let mut envelopes = Vec::new();
        let mut metas = BTreeMap::new();
        let mut bad = false;
        for (session, events) in logs.iter() {
            let Some(meta) = events.iter().find_map(|e| {
                if let Event::SessionStarted { meta } = &e.event {
                    Some(meta.clone())
                } else {
                    None
                }
            }) else {
                bad = true;
                continue;
            };
            let mut projected = events.clone();
            for event in &mut projected {
                event.project_into(&meta);
            }
            bad |= meta.session != *session || laws::check_transcript(&projected).is_err();
            metas.insert(*session, meta);
            envelopes.extend(events.clone());
        }
        (projection::union(envelopes), metas, bad)
    })
    .await;
    let mut audit = Audit::default();
    if transcript_bad {
        audit.reject("P1", "retained transcript checker failed");
    }
    let envelopes = match envelopes {
        Ok(events) => events,
        Err(error) => {
            audit.reject("P1", error.0);
            return Ok(audit);
        }
    };
    let facts = Arc::new(crate::runtime::blocking(move || projection::facts(&envelopes)).await?);
    for meta in metas.values() {
        if (meta.harness == "dream" || meta.dream.is_some())
            && meta
                .dream
                .as_ref()
                .is_none_or(|run| matches!(run.trigger, Trigger::Unknown(_)))
        {
            audit.reject("P3", "dream session has no known trigger");
        }
        if meta.harness == "dream" || meta.dream.is_some() {
            // No approved final sealing/carrier dependency has been integrated yet.
            audit.unproven("P4", "missing_run_receipt");
        }
    }
    let mut infos: BTreeMap<String, Info> = BTreeMap::new();
    let mut current = BTreeSet::new();
    let mut body_revisions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let roots = resolver.roots.values().cloned().collect::<Vec<_>>();
    for root in roots {
        for path in root
            .paths(".bboxy/memories")
            .await?
            .into_iter()
            .filter(|p| p.ends_with(".md"))
        {
            let bytes = root.read(&path, &root.main).await?;
            let digest = crate::runtime::blocking(move || entry::digest(bytes)).await;
            let pin = RecordPin {
                root: root.name.clone(),
                path: path.clone(),
                id: None,
                commit: Some(root.main.clone()),
                digest,
            };
            match resolver.record(&pin).await {
                Ok(record) => {
                    if let Some((id, info)) = inspect(record, resolver, &mut audit, &set).await {
                        let source = info.record.clone();
                        let digest =
                            crate::runtime::blocking(move || entry::digest(&source.record.body))
                                .await;
                        body_revisions.entry(id.clone()).or_default().insert(digest);
                        current.insert(id.clone());
                        infos.insert(id, info);
                    }
                }
                Err(error) => audit.unproven("P1", error.0),
            }
        }
    }
    let mut adopted = BTreeSet::new();
    for fact in facts.iter() {
        let dream = metas
            .get(&fact.session)
            .is_some_and(|m| m.harness == "dream" && m.dream.is_some());
        match &fact.body {
            Item::MemoryWrite {
                entry: declared,
                op,
                record,
                evidence,
                draft,
            } => {
                if !dream {
                    audit.reject("P3", "MemoryWrite is outside a dream session");
                }
                if evidence.iter().any(
                    |pin| matches!(pin,EvidenceRef::Event{session,..} if *session==fact.session),
                ) {
                    audit.reject("P4", "dream writer cites its own source session");
                }
                if evidence
                    .iter()
                    .any(|pin| matches!(pin, EvidenceRef::Event { .. }))
                {
                    audit.unproven(
                        "P4",
                        "no independently verified writer-effect witness for cited sessions",
                    );
                }
                if let Some(draft) = draft {
                    let draft = draft.clone();
                    let redact = set.clone();
                    let declared = declared.clone();
                    let (identity, safe) = crate::runtime::blocking(move || {
                        let safe = deny::value(&draft, &redact).is_ok();
                        let identity = Record::draft(&draft)
                            .and_then(|r| r.validate_identity())
                            .and_then(|i| i.id());
                        (identity.map(|id| id == declared), safe)
                    })
                    .await;
                    if !safe {
                        audit.reject("P8", "draft contains denied or registered secret data");
                    }
                    if identity != Ok(true) {
                        audit.reject("P4", "draft does not recompute to its entry identity");
                    }
                }
                if matches!(op, MemoryOp::Pending) {
                    if draft.is_none() {
                        audit.reject("P4", "pending write has no full draft");
                    }
                    continue;
                }
                if !matches!(op, MemoryOp::Created | MemoryOp::Revised) || record.commit.is_none() {
                    audit.unproven("P4", "materialization has no confirmed commit");
                    continue;
                }
                match resolver.record(record).await {
                    Ok(record) => {
                        if let Some((id, info)) = inspect(record, resolver, &mut audit, &set).await
                        {
                            if &id != declared {
                                audit.reject(
                                    "P4",
                                    "materialized record does not recompute to its entry identity",
                                );
                            }
                            let source = info.record.clone();
                            let digest = crate::runtime::blocking(move || {
                                entry::digest(&source.record.body)
                            })
                            .await;
                            body_revisions.entry(id.clone()).or_default().insert(digest);
                            infos.entry(id).or_insert(info);
                        }
                    }
                    Err(error) => audit.unproven("P1", error.0),
                }
            }
            Item::MemoryState {
                entry,
                state: EntryState::Adopted,
                ..
            } => {
                adopted.insert(entry.clone());
            }
            Item::Adoption { sink, .. } => {
                if deny::instruction_sink(&sink.path) && !matches!(sink.kind, SinkKind::Proposal) {
                    audit.reject(
                        "P6",
                        "automatic materialization targets an instruction sink",
                    );
                }
                audit.unproven(
                    "P2",
                    "authenticated publication-at-time and writer-effect evidence is unavailable",
                );
                audit.unproven("P8", "publication block has no verified resolving custody");
            }
            _ => {}
        }
    }
    for revisions in body_revisions.values() {
        if revisions.len() > 1 {
            audit.reject(
                "P9",
                "changed body revision is preserved, never restored as metadata repair",
            );
        }
    }
    let source = facts.clone();
    match crate::runtime::blocking(move || Projection::build(&source)).await {
        Ok(projection) => audit.projection = projection,
        Err(error) => audit.reject("P5", error.0),
    }
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut anchors = BTreeSet::new();
    let mut todo = adopted.iter().cloned().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(id) = todo.pop() {
        if !visited.insert(id.clone()) {
            continue;
        }
        let Some(info) = infos.get(&id) else {
            audit.unproven("P1", "entry closure has no resolving record");
            continue;
        };
        let evidence = info.evidence.clone();
        let mut children = info
            .identity
            .derived_from
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for pin in evidence {
            match resolver.evidence(&pin).await {
                Ok(true) => {
                    anchors.insert(id.clone());
                }
                Ok(false) => {}
                Err(error) => audit.unproven("P1", error.0),
            }
            if let EvidenceRef::Record {
                root,
                id: record_id,
                path,
                commit,
                digest,
            } = &pin
                && let Ok(record) = resolver
                    .record(&RecordPin {
                        root: root.clone(),
                        id: Some(record_id.clone()),
                        path: path.clone(),
                        commit: Some(commit.clone()),
                        digest: digest.clone(),
                    })
                    .await
                && record.record.front.get("type").and_then(Value::as_str) == Some("memory")
                && let Some((child, info)) = inspect(record, resolver, &mut audit, &set).await
            {
                children.insert(child.clone());
                infos.entry(child).or_insert(info);
            }
        }
        todo.extend(children.iter().cloned());
        adjacency.insert(id, children);
    }
    for entry in &adopted {
        let mut stack = vec![(entry.clone(), false)];
        let mut active = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut trusted = false;
        let mut closure = BTreeSet::new();
        while let Some((id, exit)) = stack.pop() {
            if exit {
                active.remove(&id);
                continue;
            }
            if active.contains(&id) {
                audit.reject("P1", "entry evidence/derived_from closure is cyclic");
                continue;
            }
            if !seen.insert(id.clone()) {
                continue;
            }
            trusted |= anchors.contains(&id);
            closure.insert(id.clone());
            active.insert(id.clone());
            stack.push((id.clone(), true));
            if let Some(children) = adjacency.get(&id) {
                stack.extend(children.iter().map(|child| (child.clone(), false)));
            }
        }
        if !trusted {
            audit.reject(
                "P6",
                "adopted entry has no independently authorized trusted anchor",
            );
        }
        if body_revisions.get(entry).is_some_and(|r| r.len() > 1) {
            audit.reject(
                "P5",
                "passing trial cannot transfer across changed body revisions",
            );
        }
        let exposed = facts
            .iter()
            .filter_map(|f| {
                if let Item::Recall { entry, .. } = &f.body {
                    closure.contains(entry).then_some(f.session)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        let evidence = closure
            .iter()
            .filter_map(|id| infos.get(id))
            .flat_map(|info| info.evidence.iter())
            .map(pin_key)
            .collect::<Result<BTreeSet<_>, _>>()?;
        for fact in facts.iter() {
            if let Item::Trial {
                entry: trial_entry,
                episodes,
                controls,
                ..
            } = &fact.body
                && trial_entry == entry
            {
                for episode in episodes.iter().chain(controls) {
                    if evidence.contains(&pin_key(&episode.origin)?) {
                        audit.reject(
                            "P5",
                            "trial episode is not held out from the evidence closure",
                        );
                    }
                    let source = match episode.origin {
                        EvidenceRef::Event { session, .. } => Some(session),
                        _ => None,
                    };
                    if source.is_some_and(|s| exposed.contains(&s)) {
                        audit.reject("P5", "recall exposure contaminates a trial session");
                    }
                    if let Err(error) = resolver.evidence(&episode.origin).await {
                        audit.unproven("P1", error.0);
                    }
                    for run in &episode.runs {
                        if resolver.logs.get(&run.session).is_none_or(|events| {
                            !events
                                .last()
                                .is_some_and(|e| matches!(e.event, Event::SessionClosed))
                        }) {
                            audit.unproven("P5", "trial run session custody is unavailable");
                        }
                    }
                }
            }
        }
    }
    let mut memories = Vec::new();
    for (id, info) in &infos {
        if current.contains(id) {
            let source = info.record.clone();
            let projected = audit
                .projection
                .entries
                .get(id)
                .cloned()
                .unwrap_or_default();
            match crate::runtime::blocking(move || {
                projection::record_view(&source.record, &projected)
            })
            .await
            {
                Ok(view) => {
                    if view != RecordView::Current {
                        audit.reject("P9","record content/projection differs; preserve bytes and omit from recall");
                    }
                    audit.record_views.insert(id.clone(), view);
                }
                Err(error) => audit.reject("P9", error.0),
            }
        }
        memories.push(info.record.clone());
    }
    let projected = audit.projection.clone();
    let tasks = resolver.accepted_tasks().await?;
    let retained = resolver.logs.clone();
    let accepted = crate::runtime::blocking(move || {
        retained
            .iter()
            .filter_map(|(session, events)| {
                let closed = events
                    .iter()
                    .any(|e| matches!(e.event, Event::SessionClosed));
                let matched = events.iter().any(|e| match &e.event {
                    Event::ItemCompleted {
                        status: ItemStatus::Completed,
                        body:
                            Item::ToolCall {
                                name, input_raw, ..
                            },
                        ..
                    } => serde_json::from_str::<Value>(input_raw)
                        .is_ok_and(|input| tasks.iter().any(|t| t.matches(name, &input))),
                    _ => false,
                });
                (closed && matched).then_some(*session)
            })
            .collect::<BTreeSet<_>>()
    })
    .await;
    audit.graph = crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let memories = memories
            .into_iter()
            .map(|record| Memory {
                root: record.pin.root.clone(),
                path: record.pin.path.clone(),
                record: record.record.clone(),
            })
            .collect::<Vec<_>>();
        Graph::build(
            &memories,
            &facts,
            &metas,
            &projected,
            &accepted,
            with_strength,
            now_ms,
        )
    })
    .await?;
    // Resolve evidence for why output from the same retained/pinned custody; snippets
    // are redacted presentation, never a replacement for their exact byte pins.
    let pins = audit
        .graph
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            node.data
                .get("pin")
                .and_then(|v| serde_json::from_value::<EvidenceRef>(v.clone()).ok())
                .map(|p| (id.clone(), p))
        })
        .collect::<Vec<_>>();
    for (id, pin) in pins {
        let value=match &pin {
            EvidenceRef::Event{..}=>resolver.event_async(&pin).await.ok().and_then(|e|serde_json::to_value(e).ok()),
            EvidenceRef::Record{root,id,path,commit,digest}=>resolver.record(&RecordPin{root:root.clone(),id:Some(id.clone()),path:path.clone(),commit:Some(commit.clone()),digest:digest.clone()}).await.ok().map(|r|
                json!({"frontmatter":r.record.front,"body":r.record.body,"trusted":r.trusted})),
            _=>None,
        };
        if let Some(value) = value {
            let redact = set.clone();
            let snippet = crate::runtime::blocking(move || {
                let value = deny::model_input(value, &redact);
                let text = serde_json::to_string(&value).unwrap_or_default();
                let excerpt = text.chars().take(4096).collect::<String>();
                json!({"text":excerpt,"redacted":true,"truncated":text.chars().count()>4096})
            })
            .await;
            if let Some(node) = audit.graph.nodes.get_mut(&id) {
                node.data["resolved"] = snippet;
            }
        }
    }
    audit.findings.sort();
    audit.findings.dedup();
    Ok(audit)
}

pub async fn cli(subcommand: &str, args: impl Iterator<Item = String>) -> Result<i32, String> {
    let mut args = args;
    let mut sessions = None;
    let mut trust_policy = None;
    let mut roots = Vec::new();
    let mut json_output = false;
    let mut strength = false;
    let mut query = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trust-policy" => {
                let Some(path) = args.next() else {
                    return Ok(2);
                };
                trust_policy = Some(PathBuf::from(path));
            }
            "--sessions" => {
                let Some(path) = args.next() else {
                    return Ok(2);
                };
                sessions = Some(PathBuf::from(path));
            }
            "--root" => {
                let Some(root) = args.next() else {
                    return Ok(2);
                };
                let Some((name, path)) = root.split_once('=') else {
                    return Ok(2);
                };
                roots.push((name.to_owned(), PathBuf::from(path)));
            }
            "--json" => json_output = true,
            "--strength" if subcommand == "graph" => strength = true,
            _ if subcommand == "why" && !arg.starts_with('-') && query.is_none() => {
                query = Some(arg)
            }
            _ => return Ok(2),
        }
    }
    if subcommand == "why" && query.is_none() {
        return Ok(2);
    }
    let set = Set::default();
    crate::capability::register(&set).await;
    let root = super::directory().map_err(|_| "dream directory unavailable")?;
    let sessions = sessions.unwrap_or_else(|| root.with_file_name("sessions"));
    if roots.is_empty() {
        let cwd = std::env::current_dir().map_err(|_| "dream cwd unavailable")?;
        let Some(repo) = crate::vcs::repo_key(&cwd).await else {
            return Ok(2);
        };
        roots.push((repo, cwd));
    }
    let mut mounted = Vec::new();
    for (name, path) in roots {
        match Root::open(name, path).await {
            Ok(root) => mounted.push(root),
            Err(_) => return Ok(2),
        }
    }
    let mut resolver = match Resolver::open(sessions, mounted).await {
        Ok(resolver) => resolver,
        Err(_) => return Ok(2),
    };
    let policy_path = trust_policy.or_else(|| {
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".bboxy450-harness/dream/trust.json"))
    });
    if let Some(path) = policy_path {
        let mut forbidden = resolver
            .roots
            .values()
            .map(|r| r.path.clone())
            .collect::<Vec<_>>();
        forbidden.push(resolver.sessions.clone());
        let logs = resolver.logs.clone();
        forbidden.extend(
            crate::runtime::blocking(move || {
                logs.values()
                    .flatten()
                    .filter_map(|e| match &e.event {
                        Event::ItemCompleted {
                            body: Item::Adoption { sink, .. },
                            ..
                        } => Some(PathBuf::from(&sink.path)),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .await,
        );
        if let Ok(trust) = super::trust::Trust::load(path, forbidden).await {
            resolver = resolver.with_trust(trust);
        }
    }
    let audit = match run(&mut resolver, set.clone(), strength, crate::store::now_ms()).await {
        Ok(audit) => audit,
        Err(_) => return Ok(2),
    };
    let subcommand = subcommand.to_owned();
    let (code, bytes) =
        crate::runtime::blocking(move || render(audit, &subcommand, query, json_output, &set))
            .await
            .map_err(|_| "dream query encoding unavailable")?;
    use tokio::io::AsyncWriteExt;
    tokio::io::stdout()
        .write_all(&bytes)
        .await
        .map_err(|_| "dream query output unavailable")?;
    Ok(code)
}

fn render(
    mut audit: Audit,
    subcommand: &str,
    query: Option<String>,
    json_output: bool,
    set: &Set,
) -> Result<(i32, Vec<u8>), Invalid> {
    crate::runtime::assert_off_rt_worker();
    let mut code = audit.exit_code();
    let value = match subcommand {
        "audit" => serde_json::to_value(&audit),
        "graph" => serde_json::to_value(&audit.graph),
        "why" => {
            let query = query.ok_or(Invalid("missing provenance query"))?;
            let matches = audit
                .graph
                .nodes
                .values()
                .filter(|node| {
                    node.id == query
                        || node
                            .data
                            .get("path")
                            .and_then(Value::as_str)
                            .is_some_and(|path| {
                                path.trim_end_matches(".md").rsplit('/').next()
                                    == Some(query.as_str())
                            })
                })
                .map(|node| node.id.clone())
                .collect::<Vec<_>>();
            if matches.len() == 1 {
                serde_json::to_value(audit.graph.why(&matches[0])?)
            } else {
                let known_sink = query.rsplit_once(':').is_some_and(|(path, line)| {
                    line.parse::<usize>().is_ok_and(|n| n > 0)
                        && audit.graph.nodes.values().any(|node| {
                            node.kind == "sink"
                                && node.data.pointer("/sink/path").and_then(Value::as_str)
                                    == Some(path)
                        })
                });
                if known_sink {
                    code = 1;
                    audit.unproven(
                        "P2",
                        "why materialization has no verified artifact/line binding",
                    );
                    serde_json::to_value(&audit)
                } else {
                    code = 2;
                    Ok(json!({"error":"unknown or ambiguous provenance query"}))
                }
            }
        }
        _ => return Err(Invalid("unknown dream query")),
    }
    .map_err(|_| Invalid("query encoding"))?;
    let value = deny::model_input(value, set);
    let bytes = if json_output || subcommand != "audit" {
        let mut bytes = serde_json::to_vec(&value).map_err(|_| Invalid("query encoding"))?;
        bytes.push(b'\n');
        bytes
    } else {
        let mut text = String::new();
        for finding in audit.findings {
            text.push_str(&format!(
                "{} {}: {}\n",
                finding.check,
                finding.kind.to_ascii_uppercase(),
                finding.detail
            ));
        }
        if text.is_empty() {
            text = "dream audit: no violations in the captured input\n".into();
        }
        deny::model_input(Value::String(text), set)
            .as_str()
            .ok_or(Invalid("query encoding"))?
            .as_bytes()
            .to_vec()
    };
    Ok((code, bytes))
}
