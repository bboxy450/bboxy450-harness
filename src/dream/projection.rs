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
                                    qualifying_trial(&trial.body)?;
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
pub fn qualifying_trial(item: &Item) -> Result<i64, Invalid> {
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
