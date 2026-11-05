// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Union validated facts first, then take independent pair maxima and global fences.
use super::entry::{self, Invalid};
use crate::protocol::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize)]
pub struct Fact {
    pub pin: EvidenceRef,
    pub session: SessionId,
    pub seq: Seq,
    pub at: u64,
    pub body: Item,
    #[serde(skip)]
    checked_usage: Option<Result<CheckedTrialUsage, UsageProofError>>,
}

/// Missing custody differs from a known contradiction; neither qualifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UsageProofError { Unavailable(&'static str), Mismatch(&'static str) }
impl UsageProofError {
    pub fn detail(&self)->&'static str {match self {Self::Unavailable(s)|Self::Mismatch(s)=>s}}
}
#[derive(Clone, Debug)]
struct CheckedRunUsage {
    session:SessionId,
    aggregate:Usage,
    basis:NumberSource,
    // Exact retained custody/turn binding is kept, never serialized as authority.
    _turn:TurnId,
    _fork:(SessionId,Seq),
    _digests:Vec<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct CheckedTrialUsage {item:Item,runs:BTreeMap<SessionId,CheckedRunUsage>}
impl CheckedTrialUsage {
    pub(crate) fn basis(&self,item:&Item,run:&TrialRun)->Option<NumberSource> {
        if self.item!=*item {return None;}
        let proof=self.runs.get(&run.session)?;
        (run.usage.as_ref()==Some(&proof.aggregate) && run.session==proof.session).then(||proof.basis.clone())
    }
}
impl Fact {
    /// Synthetic/legacy facts carry NO forged proof. A Derived fact needs actual
    /// closed retained logs through facts_with_logs at its consumer's causal cut.
    pub fn unproved(pin:EvidenceRef,session:SessionId,seq:Seq,at:u64,body:Item)->Self {
        Self {pin,session,seq,at,body,checked_usage:None}
    }
    pub fn usage_error(&self)->Option<UsageProofError> {
        if !has_derived(&self.body) && self.checked_usage.is_none() {return None;}
        match &self.checked_usage {
            Some(Ok(proof)) if proof.item==self.body=>None,
            Some(Ok(_))=>Some(UsageProofError::Mismatch("changed trial after usage proof")),
            Some(Err(e))=>Some(e.clone()),
            None=>Some(UsageProofError::Unavailable("derived trial usage proof unavailable")),
        }
    }
    pub fn qualifying_trial(&self)->Result<i64,Invalid> {qualifying_trial_context(&self.body,self.checked_usage.as_ref())}
    pub fn trial_gain(&self)->Result<i64,Invalid> {
        checked_context(&self.body,self.checked_usage.as_ref())?;
        trial_gain_math(&self.body)
    }
}
fn has_derived(item:&Item)->bool {
    matches!(item,Item::Trial{episodes,controls,..} if episodes.iter().chain(controls).flat_map(|e|&e.runs).any(|r|r.usage.as_ref().is_some_and(|u|u.source==Some(NumberSource::Derived))))
}
fn checked_context<'a>(item:&Item,proof:Option<&'a Result<CheckedTrialUsage,UsageProofError>>)->Result<Option<&'a CheckedTrialUsage>,Invalid> {
    match proof {
        Some(Ok(proof)) if proof.item==*item=>Ok(Some(proof)),
        Some(Ok(_))=>Err(Invalid("changed trial after usage proof")),
        Some(Err(UsageProofError::Mismatch(s)))=>Err(Invalid(s)),
        None if !has_derived(item)=>Ok(None),
        _=>Err(Invalid("derived trial usage proof unavailable")),
    }
}
/// Supply ONLY the selected retained logs/cut. Never fetch later/global custody.
pub fn facts_with_logs(envelopes:&[Envelope],logs:&BTreeMap<SessionId,Vec<Envelope>>)->Result<Vec<Fact>,Invalid> {
    let mut facts=facts(envelopes)?;
    for fact in &mut facts {
        if matches!(fact.body,Item::Trial{..}) {fact.checked_usage=Some(check_trial_usage(&fact.body,logs));}
    }
    Ok(facts)
}
pub(crate) fn check_trial_usage(item:&Item,logs:&BTreeMap<SessionId,Vec<Envelope>>)->Result<CheckedTrialUsage,UsageProofError> {
    use UsageProofError::*;
    let Item::Trial{episodes,controls,..}=item else {return Err(Mismatch("not a trial"));};
    let mut runs=BTreeMap::new();let mut basis=None;
    for run in episodes.iter().chain(controls).flat_map(|e|&e.runs) {
        let retained_derived=logs.get(&run.session).is_some_and(|log| {
            let cut=log.first().and_then(|e|if let Event::SessionStarted{meta}=&e.event {meta.fork.map(|f|f.1)}else{None});
            log.iter().any(|e|e.seq.is_some_and(|s|cut.is_some_and(|c|s>c)) && matches!(&e.event,Event::TurnEnded{usage:Some(u),..} if u.source==Some(NumberSource::Derived)))
        });
        if retained_derived && run.usage.as_ref().is_none_or(|u|u.source!=Some(NumberSource::Derived)) {return Err(Mismatch("trial aggregate source differs from retained derived turn"));}
        if run.usage.as_ref().is_some_and(|u|u.source==Some(NumberSource::Derived)) {
            let proof=check_run_usage(run,logs)?;
            let key=(proof.basis.clone(),proof.aggregate.backend.clone(),proof.aggregate.model.clone());
            if basis.as_ref().is_some_and(|b|b!=&key) {return Err(Mismatch("incomparable retained call basis"));}
            basis=Some(key);
            if runs.insert(run.session,proof).is_some() {return Err(Mismatch("duplicate derived run session"));}
        }
    }
    Ok(CheckedTrialUsage {item:item.clone(),runs})
}
/// The decoded view must still agree with its retained line. This is codec
/// custody, not signer authorization or proof of unlogged external behavior.
pub(crate) fn retained_bytes(log:&[Envelope])->Result<(),UsageProofError> {
    use UsageProofError::*;
    for e in log {
        let bytes=e.wire_bytes().map_err(|_|Unavailable("derived event bytes unavailable"))?;
        let text=std::str::from_utf8(&bytes).map_err(|_|Mismatch("derived event bytes invalid"))?;
        let parsed=Envelope::from_line(text).map_err(|_|Mismatch("derived event bytes invalid"))?;
        if serde_json::to_value(e).ok()!=serde_json::to_value(parsed).ok() {return Err(Mismatch("derived event differs from retained bytes"));}
    }
    Ok(())
}
fn check_run_usage(run:&TrialRun,logs:&BTreeMap<SessionId,Vec<Envelope>>)->Result<CheckedRunUsage,UsageProofError> {
    use UsageProofError::*;
    let log=logs.get(&run.session).ok_or(Unavailable("derived run log unavailable"))?;
    retained_bytes(log)?;
    let Some(Envelope {session,event:Event::SessionStarted{meta},..})=log.first() else {return Err(Unavailable("derived run header unavailable"));};
    if *session!=run.session || meta.session!=run.session {return Err(Mismatch("derived run session mismatch"));}
    if !log.last().is_some_and(|e|matches!(e.event,Event::SessionClosed)) {return Err(Unavailable("derived run is not closed"));}
    let Some(fork)=meta.fork else {return Err(Unavailable("derived continuation fork unavailable"));};
    let parent=logs.get(&fork.0).ok_or(Unavailable("derived parent custody unavailable"))?;
    let prefix=parent.iter().filter(|e|e.seq.is_some_and(|seq|seq.0>1 && seq<=fork.1) && !matches!(e.event,Event::SessionClosed)).collect::<Vec<_>>();
    let inherited=log.iter().filter(|e|e.seq.is_some_and(|seq|seq.0>1 && seq<=fork.1)).collect::<Vec<_>>();
    if fork.1.0<2 || !parent.iter().any(|e|e.seq==Some(fork.1)) {return Err(Unavailable("derived fork cut unavailable"));}
    if prefix.len()!=inherited.len() || prefix.iter().zip(&inherited).any(|(a,b)|a.wire_bytes().ok()!=b.wire_bytes().ok()) {return Err(Mismatch("derived fork prefix mismatch"));}
    let mut projected=log.clone();for e in &mut projected {e.project_into(meta);}
    if crate::protocol::laws::check_transcript(&projected).is_err() {return Err(Mismatch("derived run transcript contradiction"));}
    let continuation=log.iter().filter(|e|e.seq.is_some_and(|s|s>fork.1)).collect::<Vec<_>>();
    if continuation.iter().any(|e|e.session!=run.session) {return Err(Mismatch("foreign derived continuation event"));}
    let starts=continuation.iter().filter_map(|e|if let Event::TurnStarted{turn,..}=e.event {Some((turn,e.seq))} else {None}).collect::<Vec<_>>();
    let [(turn,start)]=starts.as_slice() else {return Err(if starts.is_empty(){Unavailable("derived target turn unavailable")}else{Mismatch("ambiguous derived target turn")});};
    let ends=continuation.iter().filter_map(|e|if let Event::TurnEnded{turn:t,usage,..}=&e.event {(*t==*turn).then_some((*t,e.seq,usage))} else {None}).collect::<Vec<_>>();
    let [(ended,end,Some(aggregate))]=ends.as_slice() else {return Err(Unavailable("derived target end unavailable"));};
    if ended!=turn || end<=start {return Err(Mismatch("derived target turn mismatch"));}
    let aggregate: &Usage=aggregate;
    if run.usage.as_ref()!=Some(aggregate) {return Err(Mismatch("derived TrialRun aggregate mismatch"));}
    let mut pending=BTreeMap::new();let mut completed=BTreeSet::new();let mut leaves=Vec::new();let mut basis=None;
    for e in &continuation {
        if e.seq<=*start || e.seq>=*end {continue;}
        match &e.event {
            Event::ItemStarted{item,turn:t,kind:ItemKind::Usage,..}=>{
                if *t!=Some(*turn) || pending.insert(*item,()).is_some() || completed.contains(item) {return Err(Mismatch("derived call identity mismatch"));}
            }
            Event::ItemCompleted{item,status,body:Item::Usage{usage}}=>{
                if pending.remove(item).is_none() || !completed.insert(*item) {return Err(Mismatch("duplicate or unjoined derived call"));}
                if usage.scope!=Some(UsageScope::Call) {continue;}
                if *status!=ItemStatus::Completed {return Err(Unavailable("incomplete derived call"));}
                let source=match usage.source {Some(NumberSource::Reported)=>NumberSource::Reported,Some(NumberSource::Estimated)=>NumberSource::Estimated,_=>return Err(Unavailable("unknown derived call basis"))};
                if !usage.numbers().contains_key("input_tokens") || !usage.numbers().contains_key("output_tokens") || usage.backend.as_ref().is_none_or(|b|b.name.is_empty() || matches!(b.kind,BackendKind::Unknown(_))) || usage.model.as_ref().is_none_or(|s|s.is_empty()) {return Err(Unavailable("missing derived call counters/identity"));}
                let key=(source,usage.backend.clone(),usage.model.clone());
                if basis.as_ref().is_some_and(|b|b!=&key) {return Err(Mismatch("mixed derived call basis/backend/model"));}
                basis=Some(key);leaves.push(usage.clone());
            }
            _=>{}
        }
    }
    if leaves.is_empty() || !pending.is_empty() {return Err(Unavailable("derived call leaves unavailable"));}
    let sum=Usage::sum_calls(&leaves).map_err(|_|Unavailable("derived call sum overflow"))?.ok_or(Unavailable("derived sum unavailable"))?;
    if sum!=*aggregate {return Err(Mismatch("derived TurnEnded aggregate mismatch"));}
    let digests=log.iter().map(|e|e.wire_bytes().map(|bytes|super::entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes))).map_err(|_|Unavailable("derived event bytes unavailable"))).collect::<Result<Vec<_>,_>>()?;
    Ok(CheckedRunUsage {session:run.session,aggregate:sum,basis:basis.unwrap().0,_turn:*turn,_fork:fork,_digests:digests})
}

/// The union key is durable identity, not an arrival index or wall clock.
pub fn union(envelopes: impl IntoIterator<Item = Envelope>) -> Result<Vec<Envelope>, Invalid> {
    let mut union: BTreeMap<(SessionId, Seq), (String, Envelope)> = BTreeMap::new();
    for envelope in envelopes {
        let Some(seq) = envelope.seq else {
            continue;
        };
        let bytes = envelope
            .wire_bytes()
            .map_err(|_| Invalid("envelope bytes unavailable"))?;
        let digest = entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes));
        let key = (envelope.session, seq);
        if let Some((prior, _)) = union.get(&key) {
            if prior != &digest {
                return Err(Invalid("conflicting durable identity digest"));
            }
        } else {
            union.insert(key, (digest, envelope));
        }
    }
    Ok(union.into_values().map(|(_, envelope)| envelope).collect())
}
pub fn facts(envelopes: &[Envelope]) -> Result<Vec<Fact>, Invalid> {
    envelopes
        .iter()
        .filter_map(|envelope| {
            let Event::ItemCompleted { body, .. } = &envelope.event else {
                return None;
            };
            let Some(seq) = envelope.seq else {
                return Some(Err(Invalid("ephemeral memory fact")));
            };
            Some(
                envelope
                    .wire_bytes()
                    .map_err(|_| Invalid("envelope bytes unavailable"))
                    .map(|bytes| Fact {
                        pin: EvidenceRef::Event {
                            session: envelope.session,
                            seq,
                            digest: entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes)),
                        },
                        session: envelope.session,
                        seq,
                        at: envelope.ts_ms,
                        body: body.clone(),
                        checked_usage: None,
                    }),
            )
        })
        .collect()
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Pair {
    pub tested: bool,
    pub adopted: bool,
    pub rejected: bool,
    pub retired: bool,
    pub reasons: BTreeSet<String>,
    pub trials: BTreeSet<String>,
}
impl Pair {
    pub fn active(&self) -> bool {
        self.adopted && !self.rejected && !self.retired
    }
    pub fn state(&self) -> &'static str {
        if self.retired {
            "retired"
        } else if self.rejected {
            "rejected"
        } else if self.adopted {
            "adopted"
        } else if self.tested {
            "tested"
        } else {
            "proposed"
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Entry {
    pub proposed: bool,
    pub expired: bool,
    pub superseded: bool,
    pub reasons: BTreeSet<String>,
    pub pairs: BTreeMap<String, Pair>,
}
impl Entry {
    pub fn terminal(&self) -> bool {
        self.expired || self.superseded
    }
    pub fn active(&self, harness: &str) -> bool {
        !self.terminal() && self.pairs.get(harness).is_some_and(Pair::active)
    }
    pub fn harnesses(&self) -> Vec<String> {
        self.pairs
            .keys()
            .filter(|h| self.active(h))
            .cloned()
            .collect()
    }
    pub fn status(&self) -> &'static str {
        if self.superseded {
            "superseded"
        } else if self.expired {
            "expired"
        } else if !self.harnesses().is_empty() {
            "adopted"
        } else if self.pairs.values().any(|p| p.state() == "tested") {
            "tested"
        } else if self.pairs.values().any(|p| p.retired) {
            "retired"
        } else if self.pairs.values().any(|p| p.rejected) {
            "rejected"
        } else {
            "proposed"
        }
    }
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Projection {
    pub entries: BTreeMap<String, Entry>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordView {
    Current,
    StaleMetadata {
        status: String,
        harnesses: Vec<String>,
    },
    ChangedContent {
        observed_entry: String,
        body_digest: String,
    },
}
/// A changed body is preserved and omitted, never overwritten as projection repair.
pub fn record_view(record: &entry::Record, projection: &Entry) -> Result<RecordView, Invalid> {
    let identity = record.identity()?;
    let observed_entry = identity.id()?;
    if record
        .front
        .get("entry")
        .and_then(serde_json::Value::as_str)
        != Some(observed_entry.as_str())
        || record.content_changed()
    {
        return Ok(RecordView::ChangedContent {
            observed_entry,
            body_digest: entry::digest(&record.body),
        });
    }
    let harnesses = projection.harnesses();
    if record
        .front
        .get("status")
        .and_then(serde_json::Value::as_str)
        != Some(projection.status())
        || record.front.get("harnesses") != Some(&serde_json::json!(harnesses))
    {
        return Ok(RecordView::StaleMetadata {
            status: projection.status().into(),
            harnesses,
        });
    }
    Ok(RecordView::Current)
}

impl Projection {
    /// Validation uses the complete pin set; delayed observations cannot reopen terminal facts.
    pub fn build(facts: &[Fact]) -> Result<Self, Invalid> {
        let mut projection = Self::default();
        for fact in facts {
            if let Item::MemoryState {
                entry,
                state,
                harness,
                reason,
                refs,
            } = &fact.body
            {
                if !entry::valid_digest(entry) {
                    return Err(Invalid("invalid state entry identity"));
                }
                let global = matches!(
                    state,
                    EntryState::Proposed | EntryState::Superseded | EntryState::Expired
                );
                if global != harness.is_none()
                    || harness.as_ref().is_some_and(|h| h.is_empty())
                    || matches!(state, EntryState::Unknown(_))
                {
                    return Err(Invalid("state harness boundary"));
                }
                let e = projection.entries.entry(entry.clone()).or_default();
                match state {
                    EntryState::Proposed => e.proposed = true,
                    EntryState::Superseded => {
                        e.superseded = true;
                        e.reasons.insert(reason.clone());
                    }
                    EntryState::Expired => {
                        e.expired = true;
                        e.reasons.insert(reason.clone());
                    }
                    _ => {
                        let harness = harness.as_deref().ok_or(Invalid("missing pair harness"))?;
                        let pair = e.pairs.entry(harness.into()).or_default();
                        match state {
                            EntryState::Tested => pair.tested = true,
                            EntryState::Adopted => {
                                let mut pins = BTreeSet::new();
                                for pin in refs {
                                    let trial = facts
                                        .iter()
                                        .find(|f| &f.pin == pin)
                                        .ok_or(Invalid("adoption trial pin does not resolve"))?;
                                    if trial.session == fact.session && trial.seq >= fact.seq {
                                        return Err(Invalid(
                                            "adoption trial pin is not causally prior",
                                        ));
                                    }
                                    let Item::Trial {
                                        entry: trial_entry,
                                        harness: trial_harness,
                                        ..
                                    } = &trial.body
                                    else {
                                        return Err(Invalid("adoption does not pin a trial"));
                                    };
                                    if trial_entry != entry || trial_harness != harness {
                                        return Err(Invalid(
                                            "adoption borrows another entry/harness trial",
                                        ));
                                    }
                                    trial.qualifying_trial()?;
                                    pins.insert(
                                        serde_json::to_string(pin)
                                            .map_err(|_| Invalid("trial pin encoding"))?,
                                    );
                                }
                                if pins.is_empty() {
                                    return Err(Invalid("adopted pair has no passing trial pin"));
                                }
                                pair.adopted = true;
                                pair.trials.extend(pins);
                            }
                            EntryState::Rejected => {
                                pair.rejected = true;
                                pair.reasons.insert(reason.clone());
                            }
                            EntryState::Retired => {
                                pair.retired = true;
                                pair.reasons.insert(reason.clone());
                            }
                            _ => return Err(Invalid("unknown entry state")),
                        }
                    }
                }
            }
        }
        if projection.entries.values().any(|e| !e.proposed) {
            return Err(Invalid("state has no proposed entry revision"));
        }
        Ok(projection)
    }
}

/// Both recorded threshold revisions share numeric gates; scheduling is not reinterpreted.
pub fn qualifying_trial(item: &Item) -> Result<i64, Invalid> {qualifying_trial_context(item,None)}
pub(crate) fn qualifying_trial_context(item:&Item,context:Option<&Result<CheckedTrialUsage,UsageProofError>>)->Result<i64,Invalid> {
    let context=checked_context(item,context)?;
    let Item::Trial {
        thresholds,
        episodes,
        controls,
        verdict,
        ..
    } = item
    else {
        return Err(Invalid("not a trial"));
    };
    if !matches!(
        thresholds.as_str(),
        "dream-thresholds@1" | "dream-thresholds@2"
    ) {
        return Err(Invalid("unknown trial thresholds revision"));
    }
    if *verdict != Verdict::Pass {
        return Err(Invalid("trial did not pass"));
    }
    if episodes.len() != 4 || controls.len() != 2 {
        return Err(Invalid("trial held-out/control count"));
    }
    let mut origins = BTreeSet::new();
    let mut sessions = BTreeSet::new();
    let mut unit = None;
    let mut spend = 0u64;
    let mut passes = [0i64; 2];
    let mut tokens = [Vec::new(), Vec::new()];
    for (control, group) in [(false, episodes), (true, controls)] {
        for episode in group {
            let pin = serde_json::to_string(&episode.origin)
                .map_err(|_| Invalid("trial origin encoding"))?;
            if matches!(episode.origin, EvidenceRef::Unknown(_)) || !origins.insert(pin) {
                return Err(Invalid("duplicate or unknown trial origin"));
            }
            let mut count = [0usize; 2];
            let mut successes = [0i64; 2];
            for run in &episode.runs {
                let arm = match run.arm {
                    Arm::With => 0,
                    Arm::Without => 1,
                    _ => return Err(Invalid("unknown trial arm")),
                };
                if !sessions.insert(run.session) {
                    return Err(Invalid("trial sessions are not independent"));
                }
                count[arm] += 1;
                let exit = run
                    .verify_exit
                    .ok_or(Invalid("unreported trial verification"))?;
                successes[arm] += i64::from(exit == 0);
                let usage = run
                    .usage
                    .as_ref()
                    .ok_or(Invalid("unreported trial usage"))?;
                let source = match usage.source {
                    Some(NumberSource::Reported) => "reported",
                    Some(NumberSource::Estimated) => "estimated",
                    Some(NumberSource::Derived) => {
                        let proof=context.and_then(|c|c.runs.get(&run.session)).ok_or(Invalid("derived trial usage proof unavailable"))?;
                        if proof.session!=run.session || proof.aggregate!=*usage {return Err(Invalid("changed derived run after proof"));}
                        match proof.basis {NumberSource::Reported=>"reported",NumberSource::Estimated=>"estimated",_=>return Err(Invalid("invalid checked basis"))}
                    }
                    _ => return Err(Invalid("unknown trial usage source")),
                };
                let key = (
                    source,
                    usage.backend.as_ref().map(|b| b.name.clone()),
                    usage.model.clone(),
                );
                if key.1.is_none()
                    || key.2.is_none()
                    || usage
                        .unreported
                        .iter()
                        .any(|field| matches!(field.as_str(), "input_tokens" | "output_tokens"))
                    || unit.as_ref().is_some_and(|prior| prior != &key)
                {
                    return Err(Invalid("trial usage is not comparable"));
                }
                unit = Some(key);
                let total = usage
                    .input_tokens
                    .checked_add(usage.output_tokens)
                    .ok_or(Invalid("trial token overflow"))?;
                spend = spend
                    .checked_add(total)
                    .ok_or(Invalid("trial token overflow"))?;
                if !control {
                    tokens[arm].push(total);
                }
            }
            if count != [if control { 1 } else { 2 }; 2] {
                return Err(Invalid("trial runs per arm"));
            }
            if control {
                if successes[0] < successes[1] {
                    return Err(Invalid("trial harms control episode"));
                }
            } else {
                for arm in 0..2 {
                    passes[arm] += successes[arm];
                }
            }
        }
    }
    if spend > 2_000_000 {
        return Err(Invalid("trial spend cap"));
    }
    if passes[0] - passes[1] < 2 {
        return Err(Invalid("trial gain threshold"));
    }
    for values in &mut tokens {
        values.sort_unstable();
    }
    let median = |values: &[u64]| u128::from(values[3]) + u128::from(values[4]);
    if median(&tokens[0]) * 4 > median(&tokens[1]) * 5 {
        return Err(Invalid("trial token cost threshold"));
    }
    Ok(passes[0] - passes[1])
}

/// Net held-out verification gain includes failed trials. Cost/control/adoption gates
/// do not erase negative observations from the optional graph strength calculation.
pub fn trial_gain(item: &Item) -> Result<i64, Invalid> {
    checked_context(item,None)?;trial_gain_math(item)
}
fn trial_gain_math(item:&Item)->Result<i64,Invalid> {
    let Item::Trial {
        episodes,
        thresholds,
        verdict,
        ..
    } = item
    else {
        return Err(Invalid("not a trial"));
    };
    if !matches!(verdict, Verdict::Pass | Verdict::Fail)
        || episodes.len() != 4
        || !matches!(
            thresholds.as_str(),
            "dream-thresholds@1" | "dream-thresholds@2"
        )
    {
        return Err(Invalid("incomplete trial gain"));
    }
    let mut seen = BTreeSet::new();
    let mut origins = BTreeSet::new();
    let mut gain = 0;
    for episode in episodes {
        if matches!(episode.origin, EvidenceRef::Unknown(_))
            || !origins.insert(
                serde_json::to_string(&episode.origin)
                    .map_err(|_| Invalid("trial origin encoding"))?,
            )
        {
            return Err(Invalid("duplicate trial origin"));
        }
        let mut count = [0; 2];
        for run in &episode.runs {
            let arm = match run.arm {
                Arm::With => 0,
                Arm::Without => 1,
                _ => return Err(Invalid("unknown trial arm")),
            };
            if !seen.insert(run.session) {
                return Err(Invalid("duplicate trial run"));
            }
            count[arm] += 1;
            if run
                .verify_exit
                .ok_or(Invalid("unreported trial verification"))?
                == 0
            {
                gain += if arm == 0 { 1 } else { -1 };
            }
        }
        if count != [2, 2] {
            return Err(Invalid("incomplete trial gain"));
        }
    }
    Ok(gain)
}
