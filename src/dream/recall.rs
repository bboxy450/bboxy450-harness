// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Explicit immutable Recall, prepared off the request path through common audit.
//! This is reference/user data, never policy. Live automatic discovery is WP9.
use super::{
    audit, entry, receipt,
    resolve::Resolver,
    seal::{Proof, ProofError as E},
};
use crate::{protocol::*, redact::Set, store::Store};

/// Affine, private contents: callers cannot forge a quoted payload or proof flag.
/// Preparation observes a caller-selected CURRENT snapshot. There is no implicit
/// cache refresh or historical wall-clock authority. Reprepare before a new start.
pub struct Context {
    harness: String,
    sink: SinkRef,
    block: String,
    digest: String,
    entries: Vec<(String, RecordPin)>,
    repos: Vec<String>,
}
impl Context {
    pub async fn prepare(
        resolver: &mut Resolver,
        session: SessionId,
        item: ItemId,
        set: &Set,
    ) -> Proof<Self> {
        let audit = audit::run(resolver, set.clone(), false, crate::store::now_ms())
            .await
            .map_err(|_| E::Unproven("recall_audit"))?;
        if audit.exit_code() != 0 {
            return Err(E::Unproven("recall_audit"));
        }
        let log = resolver
            .logs
            .get(&session)
            .ok_or(E::Unproven("recall_custody"))?
            .clone();
        let domains = resolver.receipt_domains();
        let trust = resolver.trust().ok_or(E::Unproven("producer_unapproved"))?;
        let event = log
            .iter()
            .find(|e| matches!(&e.event,Event::ItemCompleted{item:i,..} if *i==item))
            .ok_or(E::Unproven("recall_publication"))?;
        let publication = receipt::accept(event, &log, &domains, Some(&trust)).await?;
        let body = &publication.body;
        let sink = body.sink.clone().ok_or(E::Unproven("recall_sink"))?;
        if !matches!(sink.kind, SinkKind::EngineContext | SinkKind::AcpContext)
            || !matches!(
                body.outcome,
                ReceiptOutcome::Published | ReceiptOutcome::Idempotent
            )
        {
            return Err(E::Unproven("recall_loader_unsupported"));
        }
        let selection = body
            .selection
            .as_ref()
            .ok_or(E::Unproven("recall_selection"))?;
        let mut rows = vec![];
        let mut entries = vec![];
        let mut repos = std::collections::BTreeSet::new();
        for id in &selection.selected {
            let state = audit
                .projection
                .entries
                .get(id)
                .ok_or(E::Unproven("recall_projection"))?;
            if !state.active(body.harness.as_deref().unwrap_or(""))
                || !matches!(
                    audit.record_views.get(id),
                    Some(super::projection::RecordView::Current)
                )
            {
                return Err(E::Unproven("recall_terminal_or_stale"));
            }
            let c = selection
                .candidates
                .iter()
                .find(|c| &c.entry == id)
                .ok_or(E::Fail("recall_pin"))?;
            let r = resolver
                .record(&c.record)
                .await
                .map_err(|_| E::Unproven("recall_pin"))?;
            repos.extend(r.record.scope().map_err(|e| E::Fail(e.0))?.repos);
            rows.push((c.clone(), r.record.clone()));
            entries.push((id.clone(), c.record.clone()));
        }
        if entries.is_empty() {
            return Err(E::Unproven("recall_empty"));
        }
        let (block, files) = receipt::render_resources(&rows, &sink, session, event.ts_ms)?;
        if entry::digest(&block) != selection.block_digest
            || files.len() != selection.generated.len()
            || selection.generated.iter().any(|g| {
                files.get(&g.path).is_none_or(|b| {
                    g.len != b.len() as u64 || g.sha256 != super::fingerprint::digest(b)
                })
            })
        {
            return Err(E::Fail("recall_render"));
        }
        let p = publication.clone();
        let d = domains.clone();
        if !crate::runtime::blocking(move || receipt::current_labels(&p, &d))
            .await
            .is_empty()
        {
            return Err(E::Unproven("recall_artifact_drift"));
        }
        Ok(Self {
            harness: body.harness.clone().unwrap(),
            sink,
            block: String::from_utf8(block).map_err(|_| E::Fail("recall_encoding"))?,
            digest: selection.block_digest.clone(),
            entries,
            repos: repos.into_iter().collect(),
        })
    }
    /// A failed append/sync returns no bytes. Trial suppression happens before any
    /// event. The returned exact block is suitable ONLY for a separate user/reference
    /// message; the Engine/ACP seams never concatenate it to system instructions.
    pub async fn append(
        self,
        store: &mut Store,
        trial: bool,
        set: &Set,
    ) -> std::io::Result<Option<String>> {
        super::deny::value(
            &serde_json::json!({"block":self.block,"digest":self.digest,"entries":self.entries}),
            set,
        )
        .map_err(|_| std::io::Error::other("recall denied"))?;
        if trial {
            return Ok(None);
        }
        let Some(meta) = store.events.first().and_then(|e| {
            if let Event::SessionStarted { meta } = &e.event {
                Some(meta)
            } else {
                None
            }
        }) else {
            return Ok(None);
        };
        if store.closed
            || meta.harness != self.harness
            || meta
                .vcs
                .as_ref()
                .and_then(|v| v.repo.as_ref())
                .is_none_or(|r| !self.repos.contains(r))
        {
            return Ok(None);
        }
        for (entry, record) in self.entries {
            let item = ItemId::new();
            store
                .append(
                    Event::ItemStarted {
                        item,
                        kind: ItemKind::Recall,
                        turn: None,
                        parent: None,
                        head: None,
                    },
                    None,
                )
                .await?;
            store
                .append(
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body: Item::Recall {
                            entry,
                            sink: Some(self.sink.clone()),
                            basis: RecallBasis::Reported,
                            record: Some(record),
                            block_digest: Some(self.digest.clone()),
                        },
                    },
                    None,
                )
                .await?;
        }
        store.sync().await?;
        Ok(Some(self.block))
    }
    pub(crate) fn kind(&self) -> &SinkKind {
        &self.sink.kind
    }
}

/// A native timeline is explicit caller-supplied validated publication custody.
/// A citation is evidence of exposure only; it never grants authority/strength.
#[derive(Clone)]
pub struct Exposure {
    pub(crate) sink: SinkRef,
    pub(crate) entries: Vec<(String, RecordPin)>,
    pub(crate) digest: String,
    pub(crate) repo: String,
    pub(crate) cwd: std::path::PathBuf,
}
impl Exposure {
    pub async fn prepare(
        resolver: &mut Resolver,
        session: SessionId,
        item: ItemId,
        set: &Set,
    ) -> Proof<Self> {
        let audit = audit::run(resolver, set.clone(), false, crate::store::now_ms())
            .await
            .map_err(|_| E::Unproven("exposure_audit"))?;
        if audit.exit_code() != 0 {
            return Err(E::Unproven("exposure_audit"));
        }
        let log = resolver
            .logs
            .get(&session)
            .ok_or(E::Unproven("exposure_custody"))?;
        let event = log
            .iter()
            .find(|e| matches!(&e.event,Event::ItemCompleted{item:i,..} if *i==item))
            .ok_or(E::Unproven("exposure_custody"))?;
        let domains = resolver.receipt_domains();
        let trust = resolver.trust().ok_or(E::Unproven("exposure_trust"))?;
        let p = receipt::accept(event, log, &domains, Some(&trust)).await?;
        let sink = p.body.sink.ok_or(E::Unproven("exposure_sink"))?;
        if !matches!(sink.kind, SinkKind::ClaudeMemory | SinkKind::CodexExtension)
            || !matches!(
                p.body.outcome,
                ReceiptOutcome::Published | ReceiptOutcome::Idempotent
            )
        {
            return Err(E::Unproven("exposure_loader_unsupported"));
        }
        let selection = p.body.selection.ok_or(E::Unproven("exposure_selection"))?;
        let mut entries = vec![];
        let mut repos = std::collections::BTreeSet::new();
        for id in selection.selected {
            let c = selection
                .candidates
                .iter()
                .find(|c| c.entry == id)
                .ok_or(E::Fail("exposure_pin"))?;
            let r = resolver
                .record(&c.record)
                .await
                .map_err(|_| E::Unproven("exposure_pin"))?;
            repos.extend(r.record.scope().map_err(|e| E::Fail(e.0))?.repos);
            entries.push((id, c.record.clone()));
        }
        if repos.len() != 1 {
            return Err(E::Unproven("exposure_scope"));
        }
        let repo = repos.into_iter().next().unwrap();
        let cwd = resolver
            .roots
            .values()
            .find(|r| r.repo == repo)
            .ok_or(E::Unproven("exposure_scope"))?
            .path
            .clone();
        Ok(Self {
            sink,
            entries,
            digest: selection.block_digest,
            repo,
            cwd,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        engine::{Config, LaunchOptions, SessionActor},
        gateway::{Gateway, Subscribe},
        policy::Policy,
        provider::{self, profile::Unattended},
    };
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<Vec<serde_json::Value>>>>);
    impl provider::Provider for Capture {
        fn caps(&self) -> provider::Caps {
            provider::Caps {
                images: false,
                reasoning: false,
                tool_choice: true,
                context_tokens: 128000,
            }
        }
        fn stream(
            &self,
            req: provider::ModelRequest,
            _: tokio_util::sync::CancellationToken,
        ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = provider::ModelEvent> + Send>>
        {
            self.0.lock().unwrap().push(req.messages);
            Box::pin(futures_util::stream::iter([
                provider::ModelEvent::Text("private done".into()),
                provider::ModelEvent::Finish(provider::Finish::Stop),
            ]))
        }
    }
    fn context() -> Context {
        Context {
            harness: "engine".into(),
            sink: SinkRef {
                kind: SinkKind::EngineContext,
                path: "/private-fixture/context.md".into(),
            },
            block: "<bboxy-memories>automatic recall must be absent</bboxy-memories>".into(),
            digest: format!("sha256:{}", "a".repeat(64)),
            entries: vec![(
                format!("sha256:{}", "b".repeat(64)),
                RecordPin {
                    root: "fixture.invalid/repo".into(),
                    path: "memories/private.md".into(),
                    id: None,
                    commit: Some("c".repeat(40)),
                    digest: format!("sha256:{}", "d".repeat(64)),
                },
            )],
            repos: vec!["fixture.invalid/repo".into()],
        }
    }
    fn meta(cwd: PathBuf) -> SessionMeta {
        SessionMeta {
            session: SessionId::new(),
            cwd,
            harness: "engine".into(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            created_ms: 0,
            vcs: Some(Vcs {
                repo: Some("fixture.invalid/repo".into()),
                ..Default::default()
            }),
            source: None,
            dream: None,
        }
    }
    #[tokio::test]
    async fn actual_trial_capability_suppresses_automatic_recall_in_both_engine_arms() {
        use std::os::unix::fs::PermissionsExt;
        for arm in [Arm::With, Arm::Without] {
            let home = tempfile::tempdir_in("/tmp").unwrap();
            let root = home.path().canonicalize().unwrap();
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
            let cwd = root.join("cwd");
            let sessions = root.join("sessions");
            let profiles = root.join("profiles");
            for p in [&cwd, &sessions, &profiles] {
                std::fs::create_dir(p).unwrap();
                std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            // Reach the deliberately enabled control with identical quote, repo,
            // harness and pin: absence below cannot be a vacuous scope rejection.
            let mut control =
                Store::open(sessions.clone(), meta(cwd.clone()), None, Set::default())
                    .await
                    .unwrap();
            assert!(
                context()
                    .append(&mut control, false, &Set::default())
                    .await
                    .unwrap()
                    .is_some()
            );
            control.append(Event::SessionClosed, None).await.unwrap();
            drop(control);
            let (c, s, p, h) = (
                cwd.clone(),
                sessions.clone(),
                profiles.clone(),
                root.clone(),
            );
            let capability =
                crate::runtime::blocking(move || crate::tools::TrialCapability::mint(c, s, p, h))
                    .await
                    .unwrap();
            let calls = Arc::new(Mutex::new(vec![]));
            let m = meta(cwd);
            let session = m.session;
            let config = Config {
                meta: m,
                sessions: sessions.clone(),
                profiles,
                provider: Arc::new(Capture(calls.clone())),
                model: "scripted".into(),
                policy: Policy::new(Unattended::Deny),
                redactor: Set::default(),
                max_rounds: 2,
                profile_error: None,
            };
            let (g, task) = SessionActor::open_with_recall(
                config,
                OpId::new(),
                LaunchOptions {
                    trial: Some(capability),
                    ..Default::default()
                },
                context(),
            )
            .await
            .unwrap();
            let mut sub = g
                .subscribe(Subscribe {
                    session,
                    after: None,
                    ephemeral: false,
                })
                .unwrap();
            g.send(OpEnvelope {
                v: PROTOCOL,
                session,
                op_id: OpId::new(),
                op: Op::UserTurn {
                    mode: TurnMode::Queue,
                    blocks: vec![InputBlock::Text {
                        text: format!("private {arm:?} original request"),
                    }],
                },
            })
            .unwrap();
            loop {
                let e = tokio::time::timeout(std::time::Duration::from_secs(10), sub.recv())
                    .await
                    .unwrap()
                    .unwrap();
                if matches!(e.event, Event::TurnEnded { .. }) {
                    break;
                }
            }
            g.send(OpEnvelope {
                v: PROTOCOL,
                session,
                op_id: OpId::new(),
                op: Op::Close,
            })
            .unwrap();
            task.await.unwrap().unwrap();
            let messages = calls.lock().unwrap();
            assert_eq!(messages.len(), 1);
            assert!(
                messages[0]
                    .iter()
                    .all(|m| !m.to_string().contains("automatic recall must be absent"))
            );
            let log = std::fs::read_to_string(crate::store::log_path(&sessions, session)).unwrap();
            assert!(!log.contains("\"kind\":\"recall\""));
        }
    }
}

/// Safe read-only data after the common independent verifier has accepted the
/// exact finite publication/current-head closure. This never appends Recall.
#[derive(Clone)]
pub(crate) struct PreparedLesson {
    pub entry: String,
    pub record: RecordPin,
    pub current_record: RecordPin,
    pub identity: entry::Identity,
    pub context: String,
    pub citation: String,
    pub trial: EvidenceRef,
    pub publication: EvidenceRef,
    pub outer: EvidenceRef,
    pub projection_digest: String,
    pub producer_digest: String,
    pub currentness_digest: String,
    pub evidence: Vec<EvidenceRef>,
}

/// Qualify only explicit publication pins against already completed selected
/// custody. Missing proof remains unproven; malformed proof remains a failure.
/// Call on the installed SelectedScope's dedicated low-priority owner.
pub(crate) async fn prepare_selected(
    resolver: &mut Resolver,
    requested: &[EvidenceRef],
    harness: &str,
    repo: &str,
    set: &Set,
    selected_at_ms: u64,
) -> Proof<Vec<PreparedLesson>> {
    use super::{projection, resolve};
    use std::collections::BTreeMap;
    if !resolve::selected() {
        return Err(E::Unproven("selected proof owner unavailable"));
    }
    resolve::checkpoint()?;
    if requested.is_empty() {
        return Ok(vec![]);
    }
    let audit =
        resolve::ProofOwned::new(audit::run(resolver, set.clone(), false, selected_at_ms).await?);
    if let Some(finding) = audit.findings.first() {
        // Do not convert an invalid seal/signature into an empty successful read.
        return Err(if finding.kind == "violation" {
            E::Fail("selected audit violation")
        } else {
            E::Unproven("selected audit proof unavailable")
        });
    }
    let domains = resolver.receipt_domains();
    let trust = resolver.trust().ok_or(E::Unproven("producer_unapproved"))?;
    let logs = resolver.logs.clone();
    let mut publications = resolve::ProofOwned::new(Vec::new());
    let mut runs = resolve::ProofOwned::new(BTreeMap::new());
    for log in logs.values() {
        for event in log {
            resolve::checkpoint()?;
            if matches!(
                event.event,
                Event::ItemCompleted {
                    body: Item::Receipt { .. },
                    ..
                }
            ) {
                let accepted = receipt::accept_with_publications(
                    event,
                    log,
                    &domains,
                    Some(&trust),
                    &publications,
                )
                .await?;
                if accepted.body.kind == ReceiptKind::Run {
                    runs.insert(event.session, accepted);
                } else {
                    publications.push(accepted);
                }
            }
        }
    }
    let projection_digest = entry::digest(entry::canonical_value(
        serde_json::to_value(&audit.projection)
            .map_err(|_| E::Fail("selected projection encoding"))?,
    )?);
    let mut lessons = resolve::ProofOwned::new(BTreeMap::new());
    for pin in requested {
        resolve::checkpoint()?;
        let event = resolver.event_async(pin).await?;
        let Event::ItemCompleted {
            item,
            body:
                Item::Receipt {
                    kind: ReceiptKind::Publication,
                    ..
                },
            ..
        } = event.event
        else {
            return Err(E::Fail("selected publication pin"));
        };
        let publication = publications
            .iter()
            .find(|p| p.body.run.session == event.session && p.item == item)
            .ok_or(E::Unproven("selected publication unavailable"))?;
        if publication.body.harness.as_deref() != Some(harness) {
            return Err(E::Unproven("harness_incompatible"));
        }
        let sink = publication
            .body
            .sink
            .as_ref()
            .ok_or(E::Unproven("selected sink unavailable"))?;
        if !matches!(sink.kind, SinkKind::EngineContext | SinkKind::AcpContext)
            || !matches!(
                publication.body.outcome,
                ReceiptOutcome::Published | ReceiptOutcome::Idempotent
            )
        {
            return Err(E::Unproven("selected publication unsupported"));
        }
        let run = runs
            .get(&event.session)
            .ok_or(E::Unproven("missing_outer_run"))?;
        receipt::check_outer(publication, run)?;
        let log = logs
            .get(&event.session)
            .ok_or(E::Unproven("selected publication custody"))?;
        let outer_event = log
            .iter()
            .find(|e| e.seq == Some(run.seq))
            .ok_or(E::Unproven("selected outer custody"))?;
        let outer_bytes = outer_event
            .wire_bytes()
            .map_err(|_| E::Unproven("selected outer bytes"))?;
        let outer = EvidenceRef::Event {
            session: event.session,
            seq: run.seq,
            digest: entry::digest(outer_bytes.strip_suffix(b"\n").unwrap_or(&outer_bytes)),
        };
        let selection = publication
            .body
            .selection
            .as_ref()
            .ok_or(E::Unproven("missing_selection"))?;
        for id in &selection.selected {
            resolve::checkpoint()?;
            let state = audit
                .projection
                .entries
                .get(id)
                .ok_or(E::Unproven("selected projection"))?;
            if !state.active(harness)
                || audit.record_views.get(id) != Some(&projection::RecordView::Current)
            {
                return Err(E::Unproven("selected terminal or stale record"));
            }
            let candidate = selection
                .candidates
                .iter()
                .find(|c| &c.entry == id)
                .ok_or(E::Fail("selected candidate binding"))?;
            let record = resolver.record(&candidate.record).await?;
            let identity = record.record.validate_identity()?;
            if identity.id()? != *id || !record.signature_valid {
                return Err(E::Fail("selected record identity or signature"));
            }
            let scope = record.record.scope()?;
            if scope.level != ScopeLevel::Repo || scope.repos != [repo.to_owned()] {
                return Err(E::Unproven("selected repository incompatible"));
            }
            let root = resolver
                .roots
                .get(&candidate.record.root)
                .ok_or(E::Unproven("selected root unavailable"))?;
            let mut current_record = candidate.record.clone();
            current_record.commit = Some(root.main.clone());
            let current = resolver.record(&current_record).await?;
            if current.record.bytes != record.record.bytes
                || !current.signature_valid
                || projection::record_view(&current.record, state)?
                    != projection::RecordView::Current
            {
                return Err(E::Unproven("selected current-head record unavailable"));
            }
            let pair = state
                .pairs
                .get(harness)
                .ok_or(E::Unproven("selected pair unavailable"))?;
            let trial: EvidenceRef = pair
                .trials
                .iter()
                .next()
                .ok_or(E::Unproven("selected trial unavailable"))
                .and_then(|s| {
                    serde_json::from_str(s).map_err(|_| E::Fail("selected trial encoding"))
                })?;
            resolver.event_async(&trial).await?;
            let mut evidence = vec![];
            for value in &identity.evidence {
                resolve::checkpoint()?;
                evidence.push(
                    resolver
                        .evidence_value(&candidate.record.root, value)
                        .await?,
                );
            }
            // Graph::why remains explanatory. Only typed pins leave this owner;
            // arbitrary node payloads and resolved transcripts are never exported.
            let why = resolve::ProofOwned::new(audit.graph.why(id)?);
            for node in why.nodes.values() {
                resolve::checkpoint()?;
                if let Some(value) = node.data.get("pin") {
                    let pin: EvidenceRef = serde_json::from_value(value.clone())
                        .map_err(|_| E::Fail("selected graph pin"))?;
                    if !evidence.contains(&pin) {
                        evidence.push(pin);
                    }
                }
            }
            let kind = match harness {
                "engine" => crate::protocol::SinkKind::EngineContext,
                "acp" => crate::protocol::SinkKind::AcpContext,
                _ => return Err(E::Unproven("harness_incompatible")),
            };
            let context = String::from_utf8(receipt::render_selected_index(
                candidate,
                &current.record,
                &kind,
            )?)
            .map_err(|_| E::Fail("selected context encoding"))?;
            super::deny::text(&context).map_err(|_| E::Fail("selected denied context"))?;
            if set.scrub(&context) != context {
                return Err(E::Fail("selected denied context"));
            }
            let producer_digest = entry::digest(entry::canonical_value(
                serde_json::to_value(&publication.body.producer)
                    .map_err(|_| E::Fail("selected producer encoding"))?,
            )?);
            let currentness_digest = entry::digest(entry::canonical_value(
                serde_json::json!({"record":current_record,"projection":projection_digest,"selected_at_ms":selected_at_ms}),
            )?);
            let citation = format!("dream:{id}");
            let lesson = PreparedLesson {
                entry: id.clone(),
                record: candidate.record.clone(),
                current_record,
                identity,
                context,
                citation,
                trial,
                publication: pin.clone(),
                outer: outer.clone(),
                projection_digest: projection_digest.clone(),
                producer_digest,
                currentness_digest,
                evidence,
            };
            if lessons.insert(id.clone(), lesson).is_some() {
                return Err(E::Fail("selected duplicate lesson publication"));
            }
        }
    }
    Ok(lessons.take_value().into_values().collect())
}

impl super::c1::disposal::Drain for PreparedLesson {
    fn drain_step(&mut self) -> bool {
        self.identity.drain_step() || self.evidence.drain_step()
    }
}
