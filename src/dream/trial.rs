// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Trial eligibility and scoring over observations; execution requires enforced isolation.
use super::{entry, projection};
use crate::protocol::*;
use serde::Serialize;
use std::collections::BTreeSet;

pub const THRESHOLDS: &str = "dream-thresholds@2";
pub const HELD_OUT: usize = 4;
pub const CONTROLS: usize = 2;
pub const RUNS_PER_ARM: usize = 2;
pub const TOKEN_CAP: u64 = 2_000_000;
pub const TURN_SECONDS: u64 = 30 * 60;
pub const VERIFY_SECONDS: u64 = 10 * 60;

/// This is an observation predicate, not pin resolution or execution authority.
/// The planner must resolve the metadata/task/run-log bytes before using it.
pub fn original_verify_eligible(
    meta: &SessionMeta,
    verify: Option<&str>,
    exit: Option<i64>,
    elapsed_ms: Option<u64>,
) -> bool {
    let Some(vcs) = &meta.vcs else { return false };
    if vcs.dirty != Some(false)
        || !vcs.commit.as_deref().is_some_and(entry::valid_commit)
        || exit.is_none()
    {
        return false;
    }
    let Some(verify) = verify.filter(|v| !v.trim().is_empty()) else {
        return false;
    };
    if let Some(elapsed) = elapsed_ms {
        return elapsed <= VERIFY_SECONDS * 1000;
    }
    // No shell is interpreted here. Unknown command forms still need a supported
    // verifier before execution. Make/test spellings conservatively require timing.
    let words = verify
        .split_whitespace()
        .map(|w| w.trim_matches(['\'', '"', ';']))
        .collect::<Vec<_>>();
    !(words
        .iter()
        .any(|w| matches!(w.rsplit('/').next(), Some("make" | "gmake")))
        && words.contains(&"test"))
}

#[derive(Clone, Debug, Serialize)]
pub struct Score {
    pub verdict: Verdict,
    pub reason: String,
    /// Known lower bound, including failed/cancelled runs. Unknown is not free.
    pub known_tokens: u64,
    pub total_tokens: Option<u64>,
    /// Absent or mixed/uncomparable monetary costs remain absent, never zero.
    pub cost_micros_usd: Option<u64>,
}

/// Producer-side completeness/accounting precedes the ONE shared qualifier.
/// This never resolves evidence, authorizes a transition, or rewrites usage sources.
pub fn score(item: &Item) -> Score {score_context(item,None)}
pub fn score_retained(item:&Item,logs:&std::collections::BTreeMap<SessionId,Vec<Envelope>>)->Score {
    let mut candidate=item.clone();if let Item::Trial{verdict,..}=&mut candidate {*verdict=Verdict::Pass;}
    let proof=projection::check_trial_usage(&candidate,logs);score_context(&candidate,Some(&proof))
}
fn score_context(item:&Item,proof:Option<&Result<projection::CheckedTrialUsage,projection::UsageProofError>>)->Score {
    let mut score = Score {
        verdict: Verdict::Inconclusive,
        reason: "incomplete trial observations".into(),
        known_tokens: 0,
        total_tokens: None,
        cost_micros_usd: None,
    };
    let Item::Trial {
        episodes,
        controls,
        thresholds,
        ..
    } = item
    else {
        return score;
    };
    let mut complete =
        thresholds == THRESHOLDS && episodes.len() == HELD_OUT && controls.len() == CONTROLS;
    let mut sessions = BTreeSet::new();
    let mut origins = BTreeSet::new();
    let mut unit = None;
    let mut sum = Some(0u64);
    let mut cost = Some(0u64);
    for (control, group) in [(false, episodes), (true, controls)] {
        for episode in group {
            complete &= !matches!(episode.origin, EvidenceRef::Unknown(_))
                && serde_json::to_string(&episode.origin)
                    .ok()
                    .is_some_and(|p| origins.insert(p));
            let mut counts = [0usize; 2];
            for run in &episode.runs {
                match run.arm {
                    Arm::With => counts[0] += 1,
                    Arm::Without => counts[1] += 1,
                    _ => complete = false,
                }
                complete &= sessions.insert(run.session) && run.verify_exit.is_some();
                let Some(usage) = &run.usage else {
                    complete = false;
                    cost = None;
                    continue;
                };
                let numbers = usage.numbers();
                let input = numbers.get("input_tokens").copied();
                let output = numbers.get("output_tokens").copied();
                let reported = input.zip(output).and_then(|(a, b)| a.checked_add(b));
                for n in [input, output].into_iter().flatten() {
                    if let Some(total) = score.known_tokens.checked_add(n) {
                        score.known_tokens = total
                    } else {
                        sum = None;
                        complete = false
                    }
                }
                sum = sum.zip(reported).and_then(|(a, b)| a.checked_add(b));
                let source = match usage.source {
                    Some(NumberSource::Reported) => Some("reported"),
                    Some(NumberSource::Estimated) => Some("estimated"),
                    Some(NumberSource::Derived) => proof.and_then(|p|p.as_ref().ok()).and_then(|p|p.basis(item,run)).and_then(|s|match s {NumberSource::Reported=>Some("reported"),NumberSource::Estimated=>Some("estimated"),_=>None}),
                    _ => None,
                };
                let key = (
                    source,
                    usage.backend.as_ref().map(|v| v.name.clone()),
                    usage.model.clone(),
                );
                complete &= reported.is_some()
                    && key.0.is_some()
                    && key.1.as_ref().is_some_and(|v| !v.is_empty())
                    && key.2.as_ref().is_some_and(|v| !v.is_empty())
                    && matches!(usage.scope, Some(UsageScope::Turn) | Some(UsageScope::Call))
                    && unit.as_ref().is_none_or(|u| u == &key);
                unit = Some(key);
                let amount = usage
                    .cost
                    .as_ref()
                    .filter(|c| c.currency == "USD" && c.source == NumberSource::Reported)
                    .map(|c| c.amount_micros);
                cost = cost.zip(amount).and_then(|(a, b)| a.checked_add(b));
            }
            complete &= counts == [if control { 1 } else { RUNS_PER_ARM }; 2];
        }
    }
    score.total_tokens = if complete { sum } else { None };
    score.cost_micros_usd = cost;
    if !complete || sum.is_none() {
        return score;
    }
    if sum.is_some_and(|v| v > TOKEN_CAP) {
        score.reason = "per-entry token cap".into();
        return score;
    }
    let mut candidate = item.clone();
    if let Item::Trial { verdict, .. } = &mut candidate {
        *verdict = Verdict::Pass;
    }
    match projection::qualifying_trial_context(&candidate,proof) {
        Ok(_) => {
            score.verdict = Verdict::Pass;
            score.reason = "held-out gain, controls and token median pass thresholds@2".into();
        }
        Err(entry::Invalid(
            reason @ ("trial gain threshold"
            | "trial harms control episode"
            | "trial token cost threshold"),
        )) => {
            score.verdict = Verdict::Fail;
            score.reason = reason.into();
        }
        Err(entry::Invalid(reason)) => score.reason = reason.into(),
    }
    score
}

/// Use the shared graph's already resolved edges; never resolve a link by guessing
/// a filesystem root. Only predecessor relations can contaminate this entry.
pub fn closure(
    graph: &super::graph::Graph,
    entry: &str,
) -> Result<BTreeSet<String>, entry::Invalid> {
    let mut active = BTreeSet::new();
    let mut closed = BTreeSet::new();
    let mut stack = vec![(entry.to_owned(), false)];
    while let Some((id, done)) = stack.pop() {
        if done {
            active.remove(&id);
            closed.insert(id);
            continue;
        }
        if closed.contains(&id) {
            continue;
        }
        if !active.insert(id.clone()) {
            return Err(entry::Invalid("cyclic trial evidence closure"));
        }
        if graph.nodes.get(&id).is_none_or(|n| n.kind != "memory") {
            return Err(entry::Invalid("unresolved trial memory closure"));
        }
        stack.push((id.clone(), true));
        for edge in graph.edges.iter().filter(|e| {
            e.from == id && matches!(e.relation.as_str(), "derived_from" | "supersedes")
        }) {
            if edge.external {
                return Err(entry::Invalid("unresolved trial predecessor root/pin"));
            }
            if active.contains(&edge.to) {
                return Err(entry::Invalid("cyclic trial evidence closure"));
            }
            if !closed.contains(&edge.to) {
                stack.push((edge.to.clone(), false));
            }
        }
    }
    Ok(closed)
}

/// Every Recall basis (and a future item naming this entry) is possible exposure.
/// A fork's containing log is exposed even when the retained envelope names its parent.
pub fn exposed_sessions(
    logs: &std::collections::BTreeMap<SessionId, Vec<Envelope>>,
    closure: &BTreeSet<String>,
) -> BTreeSet<SessionId> {
    let mut exposed = BTreeSet::new();
    for (custody, events) in logs {
        for event in events {
            let Event::ItemCompleted { body, .. } = &event.event else {
                continue;
            };
            let contaminated = match body {
                Item::Recall { entry, .. } => closure.contains(entry),
                Item::Unknown(value) => value
                    .get("entry")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| closure.contains(id)),
                _ => false,
            };
            if contaminated {
                exposed.insert(*custody);
                exposed.insert(event.session);
            }
        }
    }
    exposed
}

#[derive(Clone, Debug, Serialize)]
pub struct Selection {
    pub episodes: Vec<super::d0::Episode>,
    pub controls: Vec<super::d0::Episode>,
}
impl Selection {
    pub fn complete(&self) -> bool {
        self.episodes.len() == HELD_OUT && self.controls.len() == CONTROLS
    }
}

/// Deterministic selection from an eligible, resolving snapshot, never a cache
/// permission or a source of eligibility. The executor must resolve pins first.
pub fn select_observed(
    episodes: &[super::d0::Episode],
    fingerprint: &str,
    evidence: &[EvidenceRef],
    exposed: &BTreeSet<SessionId>,
) -> Result<Selection, entry::Invalid> {
    if fingerprint.len() != 64 || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(entry::Invalid("invalid trial cluster"));
    }
    let evidence = evidence
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| entry::Invalid("trial evidence encoding"))?;
    let mut unique = std::collections::BTreeMap::new();
    for episode in episodes {
        if matches!(episode.origin, EvidenceRef::Unknown(_)) {
            continue;
        }
        let key = serde_json::to_string(&episode.origin)
            .map_err(|_| entry::Invalid("trial origin encoding"))?;
        if evidence.contains(&key)
            || exposed.contains(&episode.session)
            || episode.members.keys().any(|s| exposed.contains(s))
        {
            continue;
        }
        if let Some(prior) = unique.insert(key, episode)
            && serde_json::to_value(prior).map_err(|_| entry::Invalid("episode encoding"))?
                != serde_json::to_value(episode).map_err(|_| entry::Invalid("episode encoding"))?
        {
            return Err(entry::Invalid("conflicting episode observations"));
        }
    }
    let mut selected = Selection {
        episodes: Vec::new(),
        controls: Vec::new(),
    };
    for episode in unique.into_values() {
        let (target, limit) = if episode.fingerprints.contains(fingerprint) {
            (&mut selected.episodes, HELD_OUT)
        } else {
            (&mut selected.controls, CONTROLS)
        };
        if target.len() < limit {
            target.push(episode.clone());
        }
    }
    Ok(selected)
}

use std::{collections::BTreeMap,path::{Path,PathBuf},sync::Arc,time::Duration};
use crate::{provider::Provider,redact::Set,runtime::blocking,store::Store};
use tokio_util::sync::CancellationToken;

/// Opaque plan: every executable source/pin is resolved before scratch effects.
/// Native/profile/general-command adapters fail closed before this is constructed.
pub struct Plan {
    entry:String,
    harness:String,
    memory:EvidenceRef,
    record:RecordPin,
    quote:String,
    selected:Selection,
    sources:BTreeMap<String,Source>,
    logs:Arc<BTreeMap<SessionId,Vec<Envelope>>>,
    sessions:PathBuf,
}
#[derive(Clone)]
struct Source {meta:SessionMeta,cut:Seq,blocks:Vec<InputBlock>,model:String,repo:PathBuf,commit:String}
fn key(pin:&EvidenceRef)->Result<String,entry::Invalid> {serde_json::to_string(pin).map_err(|_|entry::Invalid("trial pin encoding"))}
fn metas(logs:&BTreeMap<SessionId,Vec<Envelope>>)->BTreeMap<SessionId,SessionMeta> {
    logs.iter().filter_map(|(id,events)|events.first().and_then(|e|if let Event::SessionStarted{meta}=&e.event {Some((*id,meta.clone()))}else{None})).collect()
}
fn trial_terminal(projection:&projection::Projection,id:&str,harness:&str)->bool {
    projection.entries.get(id).is_none_or(|e|e.terminal() || e.pairs.get(harness).is_some_and(|p|p.retired || p.rejected))
}
fn synthesis_fingerprint(record:&entry::Record,logs:&BTreeMap<SessionId,Vec<Envelope>>)->Result<String,entry::Invalid> {
    let origin=&record.front["origin"];
    let session:SessionId=serde_json::from_value(origin["session"].clone()).map_err(|_|entry::Invalid("trial synthesis origin unavailable"))?;
    let seq=origin["seq"].as_u64().ok_or(entry::Invalid("trial synthesis origin unavailable"))?;
    let log=logs.get(&session).ok_or(entry::Invalid("trial synthesis custody unavailable"))?;
    if !log.first().is_some_and(|e|matches!(&e.event,Event::SessionStarted{meta} if meta.harness=="dream" && meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Synthesis))) {return Err(entry::Invalid("trial synthesis phase unavailable"));}
    if !log.iter().any(|e| e.seq==Some(Seq(seq)) && matches!(&e.event,Event::ItemCompleted{body:Item::MemoryWrite{entry,..},..} if Some(entry.as_str())==record.front["entry"].as_str())) {return Err(entry::Invalid("trial origin MemoryWrite mismatch"));}
    let input=log.iter().rev().filter(|e|e.seq.is_some_and(|s|s.0<seq)).find_map(|e|if let Event::ItemCompleted{body:Item::UserMessage{blocks},..}=&e.event {blocks.first().and_then(|b|if let InputBlock::Text{text}=b {Some(text.as_str())}else{None})}else{None}).ok_or(entry::Invalid("trial source cluster unavailable"))?;
    let fp=input.strip_prefix("<cluster fingerprint=\"").and_then(|s|s.split_once("\">\n").map(|p|p.0)).filter(|s|s.len()==64 && s.bytes().all(|b|b.is_ascii_hexdigit())).ok_or(entry::Invalid("trial source cluster unavailable"))?;
    Ok(fp.into())
}
async fn memory_event(resolver:&super::resolve::Resolver,pin:&EvidenceRef)->Result<Envelope,entry::Invalid> {
    let EvidenceRef::Event{session,..}=pin else{return Err(entry::Invalid("trial memory must pin a durable event"));};
    let log=resolver.logs.get(session).ok_or(entry::Invalid("trial memory custody unavailable"))?;
    projection::retained_bytes(log).map_err(|e|entry::Invalid(e.detail()))?;
    resolver.event_async(pin).await
}
async fn memory_record(resolver:&mut super::resolve::Resolver,body:&Item,set:&Set)->Result<(String,RecordPin,entry::Record),entry::Invalid> {
    let Item::MemoryWrite{entry:id,record,op,draft,..}=body else {return Err(entry::Invalid("not a memory revision"));};
    if !resolver.roots.contains_key(&record.root) {return Err(entry::Invalid("unmounted trial memory root"));}
    super::resolve::repository_path(&record.path)?;
    let value=if *op==MemoryOp::Pending && record.commit.is_none() {
        let draft=draft.as_ref().ok_or(entry::Invalid("trial pending draft unavailable"))?;
        let draft=draft.clone();blocking(move || entry::Record::draft(&draft)).await?
    } else {resolver.record(record).await?.record.clone()};
    let check=value.clone();let expected=id.clone();let digest=record.digest.clone();let set=set.clone();
    blocking(move || {
        if check.validate_identity()?.id()?!=expected || entry::digest(&check.bytes)!=digest {return Err(entry::Invalid("trial memory pin/identity mismatch"));}
        super::deny::value(&check.front,&set).map_err(|_|entry::Invalid("denied trial memory"))?;
        super::deny::text(&check.body).map_err(|_|entry::Invalid("denied trial memory"))?;
        Ok::<_,entry::Invalid>(())
    }).await?;
    Ok((id.clone(),record.clone(),value))
}
/// Eligibility is reconstructed from retained IR, pinned records, the shared graph
/// and current monotone projection. A caller's cached D0 verdict grants nothing.
pub async fn prepare(
    resolver:&mut super::resolve::Resolver,episodes:&[super::d0::Episode],memory:&EvidenceRef,harness:&str,set:&Set,
)->Result<Plan,entry::Invalid> {
    if harness!=env!("CARGO_PKG_VERSION") {return Err(entry::Invalid("native/ACP trial isolation unavailable"));}
    let event=memory_event(resolver,memory).await?;
    let Event::ItemCompleted{body,..}=event.event else {return Err(entry::Invalid("trial memory event unavailable"));};
    let (id,pin,record)=memory_record(resolver,&body,set).await?;
    let logs=resolver.logs.clone();let all=projection::union(logs.values().flatten().cloned())?;
    let facts=projection::facts_with_logs(&all,&logs)?;let projected=projection::Projection::build(&facts)?;
    if trial_terminal(&projected,&id,harness) {return Err(entry::Invalid("terminal or unproposed trial pair"));}
    let fingerprint=synthesis_fingerprint(&record,&logs)?;
    let mut memories=BTreeMap::new();
    memories.insert(id.clone(),super::graph::Memory {root:pin.root.clone(),path:pin.path.clone(),record:record.clone()});
    // Resolve additional observed revisions for the shared graph. Missing/bad
    // referenced nodes remain external and closure() rejects them, never guesses.
    for fact in &facts {if let Item::MemoryWrite{entry:e,..}=&fact.body {
        if e==&id {continue;}
        if let Ok((e,p,r))=memory_record(resolver,&fact.body,set).await {
            if memories.get(&e).is_some_and(|old:&super::graph::Memory|old.record.bytes!=r.bytes) {return Err(entry::Invalid("ambiguous trial predecessor revision"));}
            memories.insert(e,super::graph::Memory {root:p.root,path:p.path,record:r});
        }
    }}
    let metadata=metas(&logs);
    let graph=super::graph::Graph::build(&memories.values().cloned().collect::<Vec<_>>(),&facts,&metadata,&projected,&BTreeSet::new(),false,0)?;
    let related=closure(&graph,&id)?;
    let exposed=exposed_sessions(&logs,&related);
    let mut evidence=Vec::new();let mut trusted=false;
    for related_id in &related {
        let memory=memories.get(related_id).ok_or(entry::Invalid("trial closure record unavailable"))?;
        let identity=memory.record.identity()?;let mut resolved=Vec::new();
        for value in &identity.evidence {
            let p=resolver.evidence_value(&memory.root,value).await?;
            trusted|=resolver.evidence(&p).await?;resolved.push(p.clone());evidence.push(p);
        }
        resolver.scope(&memory.record.scope()?,&resolved).await?;
    }
    if !trusted {return Err(entry::Invalid("trial has no independently trusted anchor"));}
    let mut eligible=Vec::new();let mut sources=BTreeMap::new();
    for episode in episodes {
        let Some(task)=&episode.task else {continue;};
        let EvidenceRef::Record{root,id:task_id,path,commit,digest}=&task.pin else {continue;};
        let resolved=resolver.record(&RecordPin {root:root.clone(),id:Some(task_id.clone()),path:path.clone(),commit:Some(commit.clone()),digest:digest.clone()}).await?;
        let actual=super::join::Task::from_committed(root,path,commit,&resolved.record.bytes).map_err(|_|entry::Invalid("trial task re-resolution failed"))?;
        if key(&actual.pin)?!=key(&task.pin)? {return Err(entry::Invalid("trial task pin changed"));}
        if actual.verify.as_deref()!=Some("sh -c 'test -f fixed'") {continue;}
        let timing=resolved.record.front.pointer("/verify_result/elapsed_ms");
        if timing.is_some_and(|v|!v.is_null() && v.as_u64().is_none()) {return Err(entry::Invalid("malformed original verify elapsed"));}
        let elapsed=timing.and_then(|v|v.as_u64());
        resolver.scope(&record.scope()?,&[actual.pin.clone(),episode.origin.clone()]).await?;
        let mut source=None;
        for (session,turns) in &episode.members {
            let Some(meta)=metadata.get(session) else {continue;};
            if meta.harness!=harness || !original_verify_eligible(meta,actual.verify.as_deref(),actual.verify_exit,elapsed) {continue;}
            if meta.profile.as_deref().is_some_and(|p|p!="scripted") {continue;}
            let Some(log)=logs.get(session) else {continue;};
            if !log.last().is_some_and(|e|matches!(e.event,Event::SessionClosed)) {continue;}
            projection::retained_bytes(log).map_err(|e|entry::Invalid(e.detail()))?;
            let mut projected=log.clone();for e in &mut projected {e.project_into(meta);}
            if crate::protocol::laws::check_transcript(&projected).is_err() {return Err(entry::Invalid("trial source transcript invalid"));}
            for turn in turns {
                let Some(start)=log.iter().find(|e|matches!(e.event,Event::TurnStarted{turn:t,..} if t==*turn)) else {continue;};
                let Some(start_seq)=start.seq else {continue;};
                if start.session!=*session || meta.fork.is_some_and(|(_,cut)|start_seq<=cut) {continue;}
                let Some(cut)=log.iter().filter_map(|e|e.seq.filter(|seq|*seq<start_seq)).max().filter(|s|s.0>=2) else {continue;};
                let model=match &start.event {Event::TurnStarted{model:Some(model),..} if !model.is_empty() && model.len()<=1024 && super::deny::value(&serde_json::json!(model),set).is_ok()=>model.clone(),_=>continue};
                // Preserve original UserMessage blocks exactly; this initial adapter
                // refuses mentions/resources/media before any effectful execution.
                let interval=log.iter().skip_while(|e|e.seq!=Some(start_seq)).skip(1).take_while(|e|!matches!(e.event,Event::TurnEnded{turn:t,..} if t==*turn)).collect::<Vec<_>>();
                let users=interval.iter().filter_map(|e|if let Event::ItemStarted{turn:Some(t),item,kind:ItemKind::UserMessage,..}=e.event {(*turn==t).then_some(item)}else{None}).collect::<BTreeSet<_>>();
                let blocks=interval.iter().find_map(|e|if let Event::ItemCompleted{item,body:Item::UserMessage{blocks},..}=&e.event {users.contains(item).then(||blocks.clone())}else{None});
                let Some(blocks)=blocks.filter(|b|!b.is_empty() && b.iter().all(|b|matches!(b,InputBlock::Text{..}))) else {continue;};
                if blocks.iter().any(|b|matches!(b,InputBlock::Text{text} if super::deny::text(text).is_err() || set.scrub(text)!=*text)) {continue;}
                // WP2 capture/import and Root::open already use the common repo
                // key helper. This is a canonical key, not a remote URL to parse again.
                let Some(repo)=resolver.roots.values().find(|r|meta.vcs.as_ref().and_then(|v|v.repo.as_ref()).is_some_and(|repo|repo==&r.repo)) else {continue;};
                source=Some(Source {meta:meta.clone(),cut,blocks,model,repo:repo.path.clone(),commit:meta.vcs.as_ref().and_then(|v|v.commit.clone()).unwrap()});break;
            }
            if source.is_some(){break;}
        }
        if let Some(source)=source {let mut observed=episode.clone();observed.session=source.meta.session;eligible.push(observed);sources.insert(key(&episode.origin)?,source);}
    }
    let selected=select_observed(&eligible,&fingerprint,&evidence,&exposed)?;
    if !selected.complete() {return Err(entry::Invalid("fewer than four held-out and two control episodes"));}
    let quote=format!("<trial-memory note=\"Quoted historical reference; the user request and repository files take precedence.\">\n{}\n</trial-memory>",super::deny::neutralize(&record.body));
    if quote.len()>8192 {return Err(entry::Invalid("trial memory quote exceeds bound"));}
    Ok(Plan {entry:id,harness:harness.into(),memory:memory.clone(),record:pin,quote,selected,sources,logs,sessions:resolver.sessions.clone()})
}

// The ordinary engine retains exact source IR. Redact its rendered request at
// the provider boundary as well, including inherited context and tool output;
// this does not rewrite historical bytes or grant any new tool/policy authority.
struct TrialProvider {inner:Arc<dyn Provider>,set:Set}
impl Provider for TrialProvider {
    fn id(&self)->&str {self.inner.id()}
    fn caps(&self)->crate::provider::Caps {self.inner.caps()}
    fn stream(&self,mut request:crate::provider::ModelRequest,cancel:CancellationToken)->std::pin::Pin<Box<dyn futures_core::Stream<Item=crate::provider::ModelEvent>+Send>> {
        let value=super::deny::model_input(serde_json::Value::Array(request.messages),&self.set);
        let serde_json::Value::Array(messages)=value else {unreachable!("model_input preserves array shape")};
        request.messages=messages;self.inner.stream(request,cancel)
    }
}
/// Trusted adapter factory, never deserialized from IR/model/profile data. The
/// initial adapter supports the retained scripted engine profile, not native ACP
/// or arbitrary verifier commands. Providers remain trusted ordinary Rust code.
pub type Providers=Arc<dyn Fn(&str)->Result<Arc<dyn Provider>,entry::Invalid>+Send+Sync>;
pub struct Options {
    pub scratch_parent:PathBuf,
    pub retained_parent:PathBuf,
    pub claim_root:PathBuf,
    pub providers:Providers,
    pub redactor:Set,
    /// Caller allocation can tighten, never enlarge the contract's ceilings.
    pub turn_timeout:Duration,
    pub verify_timeout:Duration,
    pub token_cap:u64,
}
#[derive(Serialize)]
pub struct Execution {
    trial:Item,
    memory:EvidenceRef,
    record:RecordPin,
    retained:PathBuf,
    reason:String,
    quarantined:bool,
    /// Retained CALL counters actually observed, including failed/cancelled work.
    /// A lower bound; missing counters are never asserted to cost zero.
    known_tokens:u64,
    #[serde(skip)]
    logs:BTreeMap<SessionId,Vec<Envelope>>,
    #[serde(skip)]
    sessions:PathBuf,
}
impl Execution {
    pub fn trial(&self)->&Item {&self.trial}
    pub fn retained_logs(&self)->&BTreeMap<SessionId,Vec<Envelope>> {&self.logs}
    pub fn sessions(&self)->PathBuf {self.sessions.clone()}
    pub fn known_tokens(&self)->u64 {self.known_tokens}
    pub fn quarantined(&self)->bool {self.quarantined}
    pub fn reason(&self)->&str {&self.reason}
}
/// Dropping the caller cancels its local token without cancelling upstream. The
/// single detached trial owner retains all scratch/claim/process observations.
pub struct Operation {task:tokio::task::JoinHandle<Result<Execution,entry::Invalid>>,cancel:CancellationToken,finished:bool}
impl Operation {
    pub async fn result(&mut self)->Result<Execution,entry::Invalid> {
        if self.finished {return Err(entry::Invalid("trial result already consumed"));}
        let _cancel_on_abandon=self.cancel.clone().drop_guard();
        let result=(&mut self.task).await.map_err(|_|entry::Invalid("trial owner unavailable; resources quarantined"));
        self.finished=true;result?
    }
}
impl Drop for Operation {fn drop(&mut self){self.cancel.cancel();}}
pub fn start(plan:Plan,options:Options,cancel:CancellationToken)->Operation {
    let local=cancel.child_token();let owned=local.clone();
    let task=bbt_rt::handle().spawn(async move {execute(plan,options,owned).await});
    Operation {task,cancel:local,finished:false}
}
struct OwnedDirectory {path:PathBuf,device:u64,inode:u64}
impl OwnedDirectory {
    fn create(parent:&Path,label:&str)->std::io::Result<Self> {
        crate::runtime::assert_off_rt_worker();
        Self::named(parent,&format!("dream-trial-{label}-{}",ulid::Ulid::new()))
    }
    fn named(parent:&Path,name:&str)->std::io::Result<Self> {
        use std::os::unix::fs::{DirBuilderExt,MetadataExt};
        crate::runtime::assert_off_rt_worker();
        let path=parent.join(name);
        std::fs::DirBuilder::new().mode(0o700).create(&path)?;
        let meta=std::fs::symlink_metadata(&path)?;
        Ok(Self {path,device:meta.dev(),inode:meta.ino()})
    }
    fn remove(self)->std::io::Result<()> {
        use std::os::unix::fs::MetadataExt;
        crate::runtime::assert_off_rt_worker();
        let m=std::fs::symlink_metadata(&self.path)?;
        if !m.is_dir() || m.file_type().is_symlink() || m.dev()!=self.device || m.ino()!=self.inode {return Err(std::io::Error::other("trial scratch identity changed"));}
        std::fs::remove_dir_all(self.path)
    }
}
struct Owner {
    scratch:Option<OwnedDirectory>,arm:Option<OwnedDirectory>,claim:Option<super::d0::Claim>,quarantine_required:bool,cancel:CancellationToken,
    observations:Vec<crate::process::Settlement>,quarantine:PathBuf,home:PathBuf,redactor:Set,
}
impl Owner {
    async fn command(&mut self,program:&str,args:Vec<String>,cwd:PathBuf,timeout:Duration,cancel:CancellationToken)->Result<crate::process::Output,entry::Invalid> {
        // Explicit allowlisted per-child environment. No process/global HOME,
        // shell startup, credentials, signing agent or source Git config leaks.
        let home=self.home.clone();let set=self.redactor.clone();
        let mut environment=vec!["-i".into(),format!("HOME={}",home.display()),format!("TMPDIR={}",home.display()),"PATH=/usr/bin:/bin".into(),"LC_ALL=C".into(),"GIT_CONFIG_NOSYSTEM=1".into(),"GIT_CONFIG_SYSTEM=/dev/null".into(),"GIT_CONFIG_GLOBAL=/dev/null".into(),"GIT_TERMINAL_PROMPT=0".into(),"GIT_ASKPASS=/usr/bin/false".into(),"SSH_ASKPASS=/usr/bin/false".into(),"GIT_LFS_SKIP_SMUDGE=1".into(),"GIT_NO_LAZY_FETCH=1".into(),"GIT_TEMPLATE_DIR=/dev/null".into(),program.into()];
        environment.extend(args);
        let mut operation=crate::process::observe_capture(crate::process::Request {program:"/usr/bin/env".into(),args:environment,stdin:None,cwd,timeout,spill_dir:home.join("unused-spill"),cap:64*1024,redactor:set.clone()},cancel);
        let settlement=operation.settlement();self.observations.push(settlement.clone());
        let result=operation.result().await.map_err(|_|entry::Invalid("trial owned command failed/cancelled"));
        if settlement.wait().await!=crate::process::SettlementStatus::Confirmed {return Err(entry::Invalid("trial process settlement unavailable"));}
        result
    }
    async fn finish_arm(&mut self)->Result<(),entry::Invalid> {
        if self.observations.iter().any(|s|s.status()!=crate::process::SettlementStatus::Confirmed) {self.quarantine_required=true;return Err(entry::Invalid("trial arm settlement unavailable"));}
        if let Some(arm)=self.arm.take() && blocking(move ||arm.remove()).await.is_err() {self.quarantine_required=true;return Err(entry::Invalid("trial arm cleanup identity/unavailable"));}
        Ok(())
    }
    async fn finish(mut self)->Result<(),entry::Invalid> {
        for observation in &self.observations {if observation.wait().await!=crate::process::SettlementStatus::Confirmed {return Err(entry::Invalid("trial cleanup remains quarantined"));}}
        if self.quarantine_required {return Err(entry::Invalid("trial cleanup remains quarantined"));}
        self.finish_arm().await?;
        if let Some(scratch)=self.scratch.take() {blocking(move ||scratch.remove()).await.map_err(|_|entry::Invalid("trial scratch cleanup identity/unavailable"))?;}
        drop(self.claim.take());Ok(())
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(claim)=self.claim.take() {
            let scratch=self.scratch.take();let arm=self.arm.take();let path=self.quarantine.clone();let observations=std::mem::take(&mut self.observations);
            // Missing/failed owner acknowledgement is deliberately NOT recovery
            // proof. Keep these resources in this local retirement continuation;
            // a marker makes this one candidate skip-able after process death.
            drop(bbt_rt::handle().spawn(async move {
                let _=blocking(move ||std::fs::write(path,b"{\"status\":\"inconclusive\",\"settlement\":\"unproven\"}\n")).await;
                std::future::pending::<()>().await;
                drop((claim,scratch,arm,observations));
            }));
        }
    }
}
async fn private_dirs(paths:Vec<PathBuf>)->Result<(),entry::Invalid> {
    blocking(move || {use std::os::unix::fs::DirBuilderExt;for p in paths {std::fs::DirBuilder::new().mode(0o700).create(p).map_err(|_|entry::Invalid("trial private directory creation failed"))?;}Ok(())}).await
}
async fn copy_source_custody(plan:&Plan,destination:PathBuf)->Result<(),entry::Invalid> {
    let source_root=plan.sessions.clone();let selected=plan.logs.keys().copied().collect::<BTreeSet<_>>();let logs=plan.logs.clone();
    blocking(move || {
        use std::{io::{Read,Write},os::unix::fs::{OpenOptionsExt,DirBuilderExt}};
        let mut budget=64u64*1024*1024;
        for session in selected {
            let log=logs.get(&session).ok_or(entry::Invalid("trial source custody missing"))?;
            let mut target=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(crate::store::log_path(&destination,session)).map_err(|_|entry::Invalid("trial source log copy failed"))?;
            for e in log {let bytes=e.wire_bytes().map_err(|_|entry::Invalid("trial source bytes unavailable"))?;budget=budget.checked_sub(bytes.len() as u64).ok_or(entry::Invalid("trial custody copy exceeds bound"))?;target.write_all(&bytes).map_err(|_|entry::Invalid("trial source log copy failed"))?;}
            crate::runtime::sync_all(&target).map_err(|_|entry::Invalid("trial source log sync failed"))?;
            let source=source_root.join(session.to_string()).join("blobs");
            let folder=destination.join(session.to_string());std::fs::DirBuilder::new().mode(0o700).create(&folder).map_err(|_|entry::Invalid("trial blob folder failed"))?;let to=folder.join("blobs");std::fs::DirBuilder::new().mode(0o700).create(&to).map_err(|_|entry::Invalid("trial blob folder failed"))?;
            let parent=source.parent().ok_or(entry::Invalid("trial source blob parent unavailable"))?;
            match std::fs::symlink_metadata(parent) {
                Ok(m) if m.is_dir() && !m.file_type().is_symlink()=>{},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,
                _=>return Err(entry::Invalid("unsafe trial source blob parent")),
            }
            match std::fs::symlink_metadata(&source) {
                Ok(m) if m.is_dir() && !m.file_type().is_symlink()=>{},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,
                _=>return Err(entry::Invalid("unsafe trial source blob directory")),
            }
            for item in std::fs::read_dir(source).map_err(|_|entry::Invalid("trial source blob custody unavailable"))? {
                let item=item.map_err(|_|entry::Invalid("trial source blob unavailable"))?;let name=item.file_name();let Some(hash)=name.to_str().filter(|s|s.len()==64 && s.bytes().all(|b|b.is_ascii_hexdigit())) else {return Err(entry::Invalid("trial source blob name invalid"));};
                let input=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC|libc::O_NONBLOCK).open(item.path()).map_err(|_|entry::Invalid("trial source blob refused"))?;
                let m=input.metadata().map_err(|_|entry::Invalid("trial blob metadata failed"))?;if !m.is_file(){return Err(entry::Invalid("trial source blob is not regular"));}budget=budget.checked_sub(m.len()).ok_or(entry::Invalid("trial custody copy exceeds bound"))?;
                let mut bytes=Vec::new();input.take(m.len()+1).read_to_end(&mut bytes).map_err(|_|entry::Invalid("trial blob copy failed"))?;
                if bytes.len() as u64!=m.len() || super::fingerprint::digest(&bytes)!=hash {return Err(entry::Invalid("trial source blob digest changed"));}
                let mut file=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(to.join(hash)).map_err(|_|entry::Invalid("trial blob copy failed"))?;file.write_all(&bytes).map_err(|_|entry::Invalid("trial blob copy failed"))?;crate::runtime::sync_all(&file).map_err(|_|entry::Invalid("trial blob sync failed"))?;
            }
        }
        Ok(())
    }).await
}
async fn execute(plan:Plan,options:Options,cancel:CancellationToken)->Result<Execution,entry::Invalid> {
    let hash=super::fingerprint::digest(format!("{}\0{}",plan.entry,plan.harness));
    let claim_root=options.claim_root.clone();let quarantine=claim_root.join(format!("trial-quarantine-{hash}.json"));let marker=quarantine.clone();
    let claim=blocking(move || {if marker.exists(){return Err(entry::Invalid("trial candidate has unproven cleanup"));}super::d0::Claim::acquire(&claim_root,&hash).map_err(|_|entry::Invalid("trial claim unavailable"))}).await?;
    let Some(claim)=claim else {return Err(entry::Invalid("trial candidate busy; no waiting"));};
    let parent=options.scratch_parent.clone();let scratch=blocking(move ||OwnedDirectory::create(&parent,"scratch")).await.map_err(|_|entry::Invalid("trial scratch creation failed"))?;
    let root=scratch.path.clone();let mut owner=Owner {scratch:Some(scratch),arm:None,claim:Some(claim),quarantine_required:false,cancel:cancel.clone(),observations:vec![],quarantine,home:root.join("active-arm/home"),redactor:options.redactor.clone()};
    let parent=options.retained_parent.clone();let retained=blocking(move ||OwnedDirectory::create(&parent,"retained")).await.map_err(|_|entry::Invalid("trial retained custody creation failed"))?;
    let retained=retained.path; // Deliberately retained, never an ephemeral Drop cleanup.
    let sessions=retained.join("sessions");let home=root.join("active-arm/home");let profiles=root.join("profiles");
    private_dirs(vec![sessions.clone(),profiles.clone()]).await?;
    copy_source_custody(&plan,sessions.clone()).await?;
    let mut outcome=Execution {trial:Item::Trial {entry:plan.entry.clone(),harness:plan.harness.clone(),thresholds:THRESHOLDS.into(),episodes:vec![],controls:vec![],verdict:Verdict::Inconclusive,reason:"trial execution incomplete".into()},memory:plan.memory.clone(),record:plan.record.clone(),retained:retained.clone(),reason:String::new(),quarantined:false,known_tokens:0,sessions:sessions.clone(),logs:plan.logs.as_ref().clone()};
    let environment=ArmEnvironment {options:&options,sessions:&sessions,profiles:&profiles,home:&home,root:&root};
    let mut spent=0u64;let cap=options.token_cap.min(TOKEN_CAP);let mut problem=None;
    'groups: for (control,selected) in [(false,&plan.selected.episodes),(true,&plan.selected.controls)] {
        for episode in selected {
            let source=plan.sources.get(&key(&episode.origin)?).ok_or(entry::Invalid("selected trial source unavailable"))?;
            let mut result=TrialEpisode {origin:episode.origin.clone(),runs:vec![]};
            for arm in [Arm::With,Arm::Without] {for _ in 0..if control {1}else{RUNS_PER_ARM} {
                if cancel.is_cancelled() || spent>=cap {problem=Some("trial cancelled or token allocation exhausted");break;}
                let run=run_arm(&mut owner,&plan,source,arm.clone(),&environment,cap-spent,cancel.child_token()).await;
                match run {
                    Ok((run,log,valid))=>{
                        let (known,complete)=observed_spend(&log,source.cut,run.session);
                        if let Some(total)=spent.checked_add(known) {spent=total;}else{spent=u64::MAX;problem=Some("trial token overflow");}
                        outcome.known_tokens=spent;
                        if !complete || run.usage.is_none(){problem=Some("trial usage unavailable/overflow");}
                        outcome.logs.insert(run.session,log);result.runs.push(run);if !valid {problem=Some("trial isolation/turn/deadline/verification unavailable");}
                    }
                    Err(error)=>{problem=Some(error.0);}
                }
                if problem.is_some(){break;}
            } if problem.is_some(){break;}}
            if let Item::Trial {episodes,controls,..}=&mut outcome.trial {if control {controls.push(result);}else{episodes.push(result);}}
            if problem.is_some(){break 'groups;}
        }
    }
    let scored=score_retained(&outcome.trial,&outcome.logs);
    let reason=problem.unwrap_or(&scored.reason).to_owned();let verdict=if problem.is_some(){Verdict::Inconclusive}else{scored.verdict};
    if let Item::Trial{verdict:v,reason:r,..}=&mut outcome.trial {*v=verdict;*r=reason.clone();}outcome.reason=reason;
    if owner.finish().await.is_err() {outcome.quarantined=true;if let Item::Trial{verdict,reason,..}=&mut outcome.trial {*verdict=Verdict::Inconclusive;*reason="trial cleanup unproven; retained quarantine".into();}}
    Ok(outcome)
}
fn observed_spend(log:&[Envelope],cut:Seq,session:SessionId)->(u64,bool) {
    let mut known=0u64;let mut complete=true;let mut found=false;
    for event in log.iter().filter(|e|e.session==session && e.seq.is_some_and(|s|s>cut)) {
        if let Event::ItemCompleted{body:Item::Usage{usage},..}=&event.event {
            if usage.scope!=Some(UsageScope::Call) {continue;}
            found=true;let numbers=usage.numbers();
            for field in ["input_tokens","output_tokens"] {if let Some(n)=numbers.get(field) {if let Some(sum)=known.checked_add(*n){known=sum;}else{known=u64::MAX;complete=false;}}else{complete=false;}}
        }
    }
    (known,complete && found)
}
#[derive(Clone,Copy)]
struct ArmEnvironment<'a> {options:&'a Options,sessions:&'a Path,profiles:&'a Path,home:&'a Path,root:&'a Path}
async fn run_arm(owner:&mut Owner,plan:&Plan,source:&Source,arm:Arm,environment:&ArmEnvironment<'_>,remaining:u64,cancel:CancellationToken)->Result<(TrialRun,Vec<Envelope>,bool),entry::Invalid> {
    use std::os::unix::fs::PermissionsExt;
    let ArmEnvironment {options,sessions,profiles,home,root}=*environment;
    let deadline=tokio::time::Instant::now()+options.turn_timeout.min(Duration::from_secs(TURN_SECONDS));
    let remaining_time=||deadline.checked_duration_since(tokio::time::Instant::now()).ok_or(entry::Invalid("trial turn deadline exhausted during setup"));
    let id=SessionId::new();let parent=root.to_path_buf();
    let directory=blocking(move ||OwnedDirectory::named(&parent,"active-arm")).await.map_err(|_|entry::Invalid("trial arm directory creation failed"))?;
    let folder=directory.path.clone();owner.arm=Some(directory);private_dirs(vec![home.to_path_buf()]).await?;
    let git=folder.join("repository.git");let worktree=folder.join("worktree");
    let prefix=vec!["--no-pager".into(),"--no-optional-locks".into(),"-c".into(),"core.hooksPath=/dev/null".into(),"-c".into(),"core.fsmonitor=false".into(),"-c".into(),"credential.helper=".into(),"-c".into(),"core.protectHFS=true".into(),"-c".into(),"core.protectNTFS=true".into(),"-c".into(),"protocol.allow=never".into()];
    let mut args=prefix.clone();args.extend(["-c".into(),"protocol.file.allow=always".into(),"clone".into(),"--bare".into(),"--local".into(),"--no-hardlinks".into(),"--no-checkout".into(),"--".into(),source.repo.to_string_lossy().into_owned(),git.to_string_lossy().into_owned()]);
    let result=owner.command("/usr/bin/git",args,folder.clone(),remaining_time()?,cancel.child_token()).await?;
    if result.code!=Some(0) || result.cancelled {return Err(entry::Invalid("private repository copy unavailable"));}
    // This adapter does not materialize submodule repositories. Refuse an
    // incomplete snapshot before inference; neither lazy fetch nor hooks run.
    let mut inventory=prefix.clone();inventory.extend(["--git-dir".into(),git.to_string_lossy().into_owned(),"ls-tree".into(),"-r".into(),"--format=%(objectmode)".into(),source.commit.clone()]);
    let result=owner.command("/usr/bin/git",inventory,folder.clone(),remaining_time()?,cancel.child_token()).await?;
    if result.cancelled || result.code!=Some(0) || result.bytes.split(|b|*b==b'\n').any(|mode|mode==b"160000") {return Err(entry::Invalid("trial complete plain-repository snapshot unavailable"));}
    let mut args=prefix;args.extend(["--git-dir".into(),git.to_string_lossy().into_owned(),"worktree".into(),"add".into(),"--detach".into(),worktree.to_string_lossy().into_owned(),source.commit.clone()]);
    let result=owner.command("/usr/bin/git",args,folder,remaining_time()?,cancel.child_token()).await?;
    if result.code!=Some(0) || result.cancelled {return Err(entry::Invalid("private detached worktree unavailable"));}
    let cwd=worktree.clone();blocking(move ||std::fs::set_permissions(cwd,std::fs::Permissions::from_mode(0o700))).await.map_err(|_|entry::Invalid("trial private worktree failed"))?;
    let provider=(options.providers)(&source.model)?;
    if provider.id()!="scripted" {return Err(entry::Invalid("unpinned trial provider/profile unsupported"));}
    let provider:Arc<dyn Provider>=Arc::new(TrialProvider {inner:provider,set:options.redactor.clone()});
    if let Some(profile)=&source.meta.profile {
        let path=crate::provider::profile::Profile::file_in(profiles,profile).map_err(|_|entry::Invalid("trial profile name unavailable"))?;
        let bytes=format!("---\nname: scripted\nmodel: {}\nendpoint: http://127.0.0.1:1/v1/chat/completions\nauto_compact: false\n---\n",serde_json::to_string(&source.model).map_err(|_|entry::Invalid("trial model encoding"))?);
        let path2=path.clone();blocking(move || {use std::io::Write;use std::os::unix::fs::OpenOptionsExt;match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path2){Ok(mut f)=>f.write_all(bytes.as_bytes()),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>if std::fs::read_to_string(path2)?==bytes {Ok(())}else{Err(std::io::Error::other("trial profile changed"))},Err(e)=>Err(e)}}).await.map_err(|_|entry::Invalid("trial private profile failed"))?;
    }
    let c=worktree.clone();let s=sessions.to_path_buf();let p=profiles.to_path_buf();let h=home.to_path_buf();
    let capability=blocking(move ||crate::tools::TrialCapability::mint(c,s,p,h)).await.map_err(|_|entry::Invalid("trial descriptor capability unavailable"))?;
    let mut meta=source.meta.clone();meta.session=id;meta.cwd=worktree.clone();meta.fork=Some((source.meta.session,source.cut));meta.created_ms=crate::store::now_ms();
    let config=crate::engine::Config {meta,sessions:sessions.to_path_buf(),profiles:profiles.to_path_buf(),provider,model:source.model.clone(),policy:crate::policy::Policy::dream_trial(),redactor:options.redactor.clone(),max_rounds:16,profile_error:None};
    let mut blocks=source.blocks.clone();if arm==Arm::With {blocks.push(InputBlock::Text{text:plan.quote.clone()});}
    let continuation=crate::frontends::headless::trial_continuation(config,capability.clone(),blocks,cancel.child_token(),remaining,deadline).await;
    let (log,mut valid,settled,ended_at)=match continuation {Ok(result)=>result,Err(error)=>{
        eprintln!("trial continuation unavailable: {}",options.redactor.scrub(&error.to_string()));
        capability.finish().await;
        let path=crate::store::log_path(sessions,id);
        let log=blocking(move || std::fs::read_to_string(path).ok().map(|text|text.split_inclusive('\n').map(Envelope::from_line).collect::<Result<Vec<_>,_>>()).transpose()).await.ok().flatten().unwrap_or_default();
        (log,false,false,None)
    }};
    if !settled {owner.quarantine_required=true;valid=false;}
    let starts=log.iter().filter(|e|e.seq.is_some_and(|s|s>source.cut) && e.session==id).filter_map(|e|if let Event::TurnStarted{turn,..}=e.event {Some(turn)}else{None}).collect::<Vec<_>>();
    let fresh=if starts.len()==1 {starts.first().copied()}else{valid=false;None};
    let usage=log.iter().filter(|e|e.seq.is_some_and(|s|s>source.cut) && e.session==id).find_map(|e|if let Event::TurnEnded{turn,usage,..}=&e.event {if Some(*turn)==fresh {usage.clone()}else{None}}else{None});
    let verify_remaining=ended_at.and_then(|at|(at+options.verify_timeout.min(Duration::from_secs(VERIFY_SECONDS))).checked_duration_since(tokio::time::Instant::now()));
    if verify_remaining.is_none() {valid=false;}
    let exit=if valid && !cancel.is_cancelled() && verify_remaining.is_some() {
        match capability.verifier_preflight().await {
            Ok(_)=>{match owner.command("/bin/sh",vec!["-c".into(),"test -f fixed".into()],worktree,verify_remaining.unwrap_or_default(),cancel.child_token()).await {Ok(output) if !output.cancelled=>output.code,_=>{valid=false;None}}}
            Err(_)=>{valid=false;None}
        }
    }else{None};
    if exit.is_none() {valid=false;}
    if settled && owner.finish_arm().await.is_err() {valid=false;}
    Ok((TrialRun {arm,session:id,verify_exit:exit,usage},log,valid))
}

/// Append only to the caller-owned ordinary dream trial Store. This stage has no
/// publication receipt/seal and remains UNPROVEN for complete audit (WP8/9).
pub async fn append(store:&mut Store,resolver:&mut super::resolve::Resolver,result:&Execution)->Result<(),entry::Invalid> {
    let custody=store.blobs.dir.parent().and_then(Path::parent);
    if custody!=Some(result.sessions.as_path()) {return Err(entry::Invalid("trial output must retain ordinary arm custody together"));}
    if store.closed || !store.events.first().is_some_and(|e|matches!(&e.event,Event::SessionStarted{meta} if meta.harness=="dream" && meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Trial))) {return Err(entry::Invalid("trial output requires owned trial dream Store"));}
    let Item::Trial{entry:id,harness,verdict,..}=&result.trial else {return Err(entry::Invalid("invalid trial result"));};
    let event=memory_event(resolver,&result.memory).await?;let Event::ItemCompleted{body,..}=event.event else{return Err(entry::Invalid("trial source memory unavailable"));};
    let (source_id,pin,_)=memory_record(resolver,&body,&Set::default()).await?;
    if source_id!=*id || pin!=result.record {return Err(entry::Invalid("trial source revision changed"));}
    let mut all=resolver.logs.values().flatten().cloned().collect::<Vec<_>>();all.extend(store.events.clone());
    let facts=projection::facts_with_logs(&projection::union(all)?,&resolver.logs)?;let current=projection::Projection::build(&facts)?;
    let terminal=trial_terminal(&current,id,harness);
    let mut bodies=vec![result.trial.clone()];
    // Actual Trial pin is known only after its durable append.
    let body=bodies.remove(0);let item=ItemId::new();store.append(Event::ItemStarted{turn:None,item,kind:body.kind(),parent:None,head:None},None).await.map_err(|_|entry::Invalid("trial append failed"))?;
    let event=store.append(Event::ItemCompleted{item,status:ItemStatus::Completed,body},None).await.map_err(|_|entry::Invalid("trial append failed"))?;
    let bytes=event.wire_bytes().map_err(|_|entry::Invalid("trial pin bytes unavailable"))?;
    let trial_pin=EvidenceRef::Event {session:event.session,seq:event.seq.ok_or(entry::Invalid("trial seq missing"))?,digest:entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes))};
    let mut states=vec![(EntryState::Tested,"trial observations recorded",vec![result.memory.clone(),trial_pin.clone()])];
    let scored=score_retained(&result.trial,&result.logs);
    if !terminal && !result.quarantined && scored.verdict==*verdict {
        match verdict {Verdict::Pass=>states.push((EntryState::Adopted,"qualified same-harness trial",vec![trial_pin])),Verdict::Fail=>states.push((EntryState::Rejected,"completed trial failed numeric gates",vec![trial_pin])),_=>{}}
    }
    for (state,reason,refs) in states {
        let body=Item::MemoryState {entry:id.clone(),state,harness:Some(harness.clone()),reason:reason.into(),refs};let item=ItemId::new();
        store.append(Event::ItemStarted{turn:None,item,kind:body.kind(),parent:None,head:None},None).await.map_err(|_|entry::Invalid("trial state append failed"))?;
        store.append(Event::ItemCompleted{item,status:ItemStatus::Completed,body},None).await.map_err(|_|entry::Invalid("trial state append failed"))?;
    }
    store.sync().await.map_err(|_|entry::Invalid("trial output sync failed"))
}

/// Unsupported/busy/ineligible observations still produce lawful Trial + Tested
/// in the caller's ordinary custody, with no claimed isolation, free usage or pass.
pub async fn defer(store:&mut Store,resolver:&mut super::resolve::Resolver,memory:&EvidenceRef,harness:&str,why:entry::Invalid)->Result<(),entry::Invalid> {
    if harness.is_empty() || harness.len()>128 || harness.chars().any(char::is_control) {return Err(entry::Invalid("invalid trial harness"));}
    let event=memory_event(resolver,memory).await?;let Event::ItemCompleted{body,..}=event.event else{return Err(entry::Invalid("trial memory unavailable"));};
    let (id,record,_)=memory_record(resolver,&body,&Set::default()).await?;
    let sessions=store.blobs.dir.parent().and_then(Path::parent).ok_or(entry::Invalid("trial store custody unavailable"))?.to_path_buf();
    let result=Execution {trial:Item::Trial {entry:id,harness:harness.into(),thresholds:THRESHOLDS.into(),episodes:vec![],controls:vec![],verdict:Verdict::Inconclusive,reason:why.0.into()},memory:memory.clone(),record,retained:sessions.clone(),reason:why.0.into(),quarantined:false,known_tokens:0,logs:resolver.logs.as_ref().clone(),sessions};
    append(store,resolver,&result).await
}

#[cfg(test)]
mod input_tests {
    use super::*;
    struct Capture(Arc<std::sync::Mutex<Vec<crate::provider::ModelRequest>>>);
    impl Provider for Capture {
        fn id(&self)->&str {"scripted"}
        fn caps(&self)->crate::provider::Caps {crate::provider::ScriptedProvider::new(vec![]).caps()}
        fn stream(&self,request:crate::provider::ModelRequest,_cancel:CancellationToken)->std::pin::Pin<Box<dyn futures_core::Stream<Item=crate::provider::ModelEvent>+Send>> {
            self.0.lock().unwrap().push(request);Box::pin(futures_util::stream::empty())
        }
    }
    #[test]
    fn inherited_input_uses_common_deny_without_rewriting_source_or_model_identity() {
        let set=Set::default();let registered="private-fixture-registered-value";set.register(registered);
        let raw=format!("retained {registered}, {}",["sk-",&"Z".repeat(24)].concat());let source=serde_json::json!({"role":"user","content":raw});
        let captured=Arc::new(std::sync::Mutex::new(Vec::new()));let provider=TrialProvider {inner:Arc::new(Capture(captured.clone())),set:set.clone()};
        let stream=provider.stream(crate::provider::ModelRequest {model:"scripted".into(),messages:vec![source.clone()],tools:vec![],tool_choice:crate::provider::ToolChoice::Auto,max_tokens:None,stable_prefix:0},CancellationToken::new());drop(stream);
        let requests=captured.lock().unwrap();assert_eq!(requests.len(),1);assert_eq!(requests[0].model,"scripted");assert!(super::super::deny::value(&serde_json::json!(requests[0].messages),&set).is_ok());assert!(source["content"].as_str().unwrap().contains(registered));
    }
}
