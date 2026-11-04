// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Read-only provenance and strength. Times rank observations, never lifecycle facts.
use super::{
    entry::{self, Invalid, Record},
    projection::{Fact, Projection},
};
use crate::protocol::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Memory {
    pub root: String,
    pub path: String,
    pub record: Record,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f64>,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f64,
    pub external: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f64>,
}
#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
}

pub fn evidence_node(pin: &EvidenceRef) -> Result<String, Invalid> {
    match pin {
        EvidenceRef::Event {
            session,
            seq,
            digest,
        } => Ok(format!("ir:{session}#{}@{digest}", seq.0)),
        EvidenceRef::Record {
            root,
            path,
            commit,
            digest,
            ..
        } => Ok(format!("kb:{root}/{path}@{commit}#{digest}")),
        _ => Err(Invalid("unknown graph evidence")),
    }
}
/// Days are explicit. Neither function is used to authorize a state or sink.
pub fn node_strength(gain: f64, last: f64, now: f64) -> f64 {
    (1.0 + 0.5 * gain.min(4.0)) * (-(now - last) / 30.0).exp()
}
pub fn edge_strength(weight: f64, coactivations: impl IntoIterator<Item = f64>, now: f64) -> f64 {
    (weight
        + 0.25
            * coactivations
                .into_iter()
                .map(|time| (-(now - time) / 30.0).exp())
                .sum::<f64>())
    .min(10.0)
}
fn slug(path: &str) -> &str {
    path.trim_start_matches(".bboxy/")
        .strip_suffix(".md")
        .unwrap_or(path.trim_start_matches(".bboxy/"))
}
fn target(
    target: &str,
    root: &str,
    catalogue: &BTreeMap<(String, String), String>,
    roots: &BTreeSet<String>,
) -> Option<String> {
    let target = target
        .strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
        .unwrap_or(target);
    let (root, target) = if let Some(cross) = target.strip_prefix("kb:") {
        let root = roots
            .iter()
            .filter(|r| cross.starts_with(&format!("{r}/")))
            .max_by_key(|r| r.len())?;
        (root.as_str(), cross.strip_prefix(&format!("{root}/"))?)
    } else {
        (root, target)
    };
    catalogue.get(&(root.into(), target.into())).cloned()
}
impl Graph {
    fn node(&mut self, id: String, kind: &str, data: Value) {
        self.nodes.entry(id.clone()).or_insert(Node {
            id,
            kind: kind.into(),
            data,
            strength: None,
        });
    }
    fn edge(
        &mut self,
        from: &str,
        to: String,
        relation: &str,
        weight: f64,
        external: bool,
    ) -> Result<(), Invalid> {
        if !weight.is_finite() || weight <= 0.0 || weight > 10.0 {
            return Err(Invalid("invalid graph link weight"));
        }
        if external {
            self.node(
                to.clone(),
                "external",
                json!({"locator":to,"resolving":false}),
            );
        }
        self.edges.push(Edge {
            from: from.into(),
            to,
            relation: relation.into(),
            weight,
            external,
            strength: None,
        });
        Ok(())
    }
    /// Inputs are authorized, resolved snapshots. An external wikilink never triggers IO.
    pub fn build(
        memories: &[Memory],
        facts: &[Fact],
        metas: &BTreeMap<SessionId, SessionMeta>,
        projection: &Projection,
        accepted: &BTreeSet<SessionId>,
        with_strength: bool,
        now_ms: u64,
    ) -> Result<Self, Invalid> {
        let mut graph = Self::default();
        let mut memories = memories.iter().collect::<Vec<_>>();
        memories.sort_by_cached_key(|m| (&m.root, &m.path, entry::digest(&m.record.bytes)));
        let mut catalogue = BTreeMap::new();
        let mut roots = BTreeSet::new();
        for memory in &memories {
            let id = memory
                .record
                .front
                .get("entry")
                .and_then(Value::as_str)
                .ok_or(Invalid("memory graph missing entry"))?
                .to_owned();
            roots.insert(memory.root.clone());
            for name in [
                slug(&memory.path),
                memory
                    .path
                    .trim_start_matches(".bboxy/")
                    .trim_end_matches(".md")
                    .rsplit('/')
                    .next()
                    .unwrap_or(""),
            ] {
                let key = (memory.root.clone(), name.into());
                if catalogue
                    .insert(key, id.clone())
                    .is_some_and(|prior| prior != id)
                {
                    return Err(Invalid("ambiguous memory graph locator"));
                }
            }
            graph.node(
                id.clone(),
                "memory",
                json!({"entry":id,"root":memory.root,"path":memory.path,
                "title":memory.record.front.get("title"),"projection":projection.entries.get(&id)}),
            );
            let origin = json!({"root":memory.root,"path":memory.path});
            let records = graph
                .nodes
                .get_mut(&id)
                .unwrap()
                .data
                .as_object_mut()
                .unwrap()
                .entry("records")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap();
            if !records.contains(&origin) {
                records.push(origin);
            }
        }
        static LINKS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let body_links = LINKS.get_or_init(|| {
            regex::Regex::new(r"\[\[([^\]\n]+)\]\](?:\{([a-z_]+):([0-9.]+)\})?").unwrap()
        });
        for memory in &memories {
            let identity = memory.record.identity()?;
            let id = identity.id()?;
            for predecessor in &identity.derived_from {
                graph.edge(
                    &id,
                    predecessor.clone(),
                    "derived_from",
                    2.0,
                    !graph.nodes.contains_key(predecessor),
                )?;
            }
            if let Some(links) = memory.record.front.get("links").and_then(Value::as_array) {
                for link in links {
                    let to = link
                        .get("to")
                        .and_then(Value::as_str)
                        .ok_or(Invalid("invalid memory link"))?;
                    let relation = link.get("rel").and_then(Value::as_str).unwrap_or("related");
                    let default = match relation {
                        "evidence" | "derived_from" | "generalizes" | "refines" => 2.0,
                        "supersedes" | "contradicts" => 3.0,
                        _ => 1.0,
                    };
                    let weight = link
                        .get("weight")
                        .map_or(Some(default), Value::as_f64)
                        .ok_or(Invalid("invalid memory link weight"))?;
                    let resolved = target(to, &memory.root, &catalogue, &roots);
                    graph.edge(
                        &id,
                        resolved.clone().unwrap_or_else(|| to.into()),
                        relation,
                        weight,
                        resolved.is_none(),
                    )?;
                }
            }
            for capture in body_links.captures_iter(&memory.record.body) {
                let to = &capture[1];
                let relation = capture.get(2).map_or("related", |m| m.as_str());
                let weight = capture
                    .get(3)
                    .map_or(Ok(1.0), |m| m.as_str().parse())
                    .map_err(|_| Invalid("invalid body link weight"))?;
                let resolved = target(to, &memory.root, &catalogue, &roots);
                graph.edge(
                    &id,
                    resolved.clone().unwrap_or_else(|| format!("[[{to}]]")),
                    relation,
                    weight,
                    resolved.is_none(),
                )?;
            }
        }
        let mut gains: BTreeMap<String, f64> = BTreeMap::new();
        let mut last: BTreeMap<String, u64> = BTreeMap::new();
        let mut recalls: BTreeMap<SessionId, BTreeMap<String, u64>> = BTreeMap::new();
        for (session, meta) in metas {
            let node = format!("session:{session}");
            graph.node(
                node.clone(),
                "session",
                json!({"session":session,"dream":meta.dream,"source":meta.source}),
            );
            if let Some(dream) = &meta.dream {
                let trigger = serde_json::to_value(&dream.trigger)
                    .map_err(|_| Invalid("trigger encoding"))?;
                let id = format!(
                    "trigger:{}",
                    entry::digest(
                        serde_json::to_vec(&trigger).map_err(|_| Invalid("trigger encoding"))?
                    )
                );
                graph.node(id.clone(), "trigger", trigger);
                graph.edge(&node, id, "triggered_by", 1.0, false)?;
            }
        }
        for fact in facts {
            let pin = evidence_node(&fact.pin)?;
            let session = format!("session:{}", fact.session);
            match &fact.body {
                Item::MemoryWrite {
                    entry,
                    record,
                    evidence,
                    ..
                } => {
                    graph.node(
                        pin.clone(),
                        "memory_write",
                        json!({"pin":fact.pin,"record":record}),
                    );
                    graph.edge(entry, pin.clone(), "written_by", 1.0, false)?;
                    for evidence in evidence {
                        let node = evidence_node(evidence)?;
                        graph.node(node.clone(), "evidence", json!({"pin":evidence}));
                        graph.edge(entry, node, "evidence", 2.0, false)?;
                    }
                }
                Item::Trial {
                    entry,
                    episodes,
                    controls,
                    ..
                } => {
                    graph.node(
                        pin.clone(),
                        "trial",
                        json!({"pin":fact.pin,"trial":fact.body}),
                    );
                    graph.edge(entry, pin.clone(), "tested_by", 2.0, false)?;
                    for episode in episodes.iter().chain(controls) {
                        let node = evidence_node(&episode.origin)?;
                        graph.node(node.clone(), "evidence", json!({"pin":episode.origin}));
                        graph.edge(&pin, node, "held_out", 2.0, false)?;
                    }
                    if let Ok(gain) = fact.trial_gain() {
                        *gains.entry(entry.clone()).or_default() += gain as f64;
                    }
                }
                Item::MemoryState {
                    entry,
                    state,
                    harness,
                    ..
                } => {
                    graph.node(
                        pin.clone(),
                        "state",
                        json!({"pin":fact.pin,"state":state,"harness":harness}),
                    );
                    graph.edge(entry, pin.clone(), "state", 1.0, false)?;
                    if matches!(state, EntryState::Adopted) {
                        last.entry(entry.clone())
                            .and_modify(|t| *t = (*t).max(fact.at))
                            .or_insert(fact.at);
                    }
                }
                Item::Adoption {
                    entry,
                    sink,
                    record,
                    block_digest,
                    ..
                } => {
                    graph.node(pin.clone(),"publication",json!({"pin":fact.pin,"sink":sink,"record":record,"block_digest":block_digest}));
                    graph.edge(entry, pin.clone(), "materialized_by", 1.0, false)?;
                    let sink_id = format!("sink:{}", sink.path);
                    graph.node(sink_id.clone(), "sink", json!({"sink":sink}));
                    graph.edge(&sink_id, entry.clone(), "materializes", 1.0, false)?;
                }
                Item::Recall { entry, basis, .. } => {
                    graph.node(
                        pin.clone(),
                        "recall",
                        json!({"pin":fact.pin,"basis":basis,"model_use":false}),
                    );
                    graph.edge(entry, pin.clone(), "recalled_by", 1.0, false)?;
                    if accepted.contains(&fact.session)
                        && matches!(basis, RecallBasis::Reported | RecallBasis::Cited)
                    {
                        recalls
                            .entry(fact.session)
                            .or_default()
                            .entry(entry.clone())
                            .and_modify(|t| *t = (*t).max(fact.at))
                            .or_insert(fact.at);
                        last.entry(entry.clone())
                            .and_modify(|t| *t = (*t).max(fact.at))
                            .or_insert(fact.at);
                    }
                }
                Item::Receipt { .. } => {
                    graph.node(
                        pin.clone(),
                        "receipt",
                        json!({"pin":fact.pin,"receipt":fact.body,"presence_is_not_approval":true}),
                    );
                }
                _ => continue,
            }
            graph.edge(&pin, session, "generated_in", 1.0, false)?;
        }
        graph.edges.sort_by(|a, b| {
            (&a.from, &a.to, &a.relation, a.weight.to_bits()).cmp(&(
                &b.from,
                &b.to,
                &b.relation,
                b.weight.to_bits(),
            ))
        });
        graph.edges.dedup();
        if with_strength {
            let days = |ms: u64| ms as f64 / 86_400_000.0;
            let now = days(now_ms);
            for (id, node) in &mut graph.nodes {
                if node.kind == "memory" {
                    node.strength = last
                        .get(id)
                        .map(|at| node_strength(*gains.get(id).unwrap_or(&0.0), days(*at), now));
                }
            }
            for edge in &mut graph.edges {
                let times = recalls
                    .values()
                    .filter_map(|r| Some(days((*r.get(&edge.from)?).max(*r.get(&edge.to)?))));
                edge.strength = Some(edge_strength(edge.weight, times, now));
            }
        }
        Ok(graph)
    }
    /// The chain is a finite graph walk, including explicitly unresolved external nodes.
    pub fn why(&self, start: &str) -> Result<Self, Invalid> {
        if !self.nodes.contains_key(start) {
            return Err(Invalid("unknown provenance query"));
        }
        let mut result = Self::default();
        let mut todo = vec![start.to_owned()];
        let mut seen = BTreeSet::new();
        while let Some(id) = todo.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(node) = self.nodes.get(&id) {
                result.nodes.insert(id.clone(), node.clone());
            }
            for edge in self.edges.iter().filter(|e| e.from == id) {
                todo.push(edge.to.clone());
                result.edges.push(edge.clone());
            }
        }
        result
            .edges
            .sort_by(|a, b| (&a.from, &a.to, &a.relation).cmp(&(&b.from, &b.to, &b.relation)));
        result.edges.dedup();
        Ok(result)
    }
}
