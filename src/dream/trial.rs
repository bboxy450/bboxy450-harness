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
pub fn score(item: &Item) -> Score {
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
    match projection::qualifying_trial(&candidate) {
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
