// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Offline usage accounting. Prefix identity is resolved before scope selection.
use crate::protocol::*;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub rows: Vec<Row>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Row {
    pub backend_kind: BackendKind,
    pub backend: String,
    pub model: Option<String>,
    pub currency: Option<String>,
    pub scope: UsageScope,
    pub source: NumberSource,
    pub cost_source: Option<NumberSource>,
    pub missing: BTreeMap<String, u64>,
    pub numbers: BTreeMap<String, u64>,
    pub records: u64,
}
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    usage: Usage,
    turn: Option<TurnId>,
    scope: UsageScope,
    legacy: bool,
}
type Origin = (SessionId, Seq);
fn files(path: &Path, out: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                files(&path, out)?;
            } else if path.extension().is_some_and(|ext| ext == "jsonl") {
                out.insert(path);
            }
        }
    } else if path.is_file() {
        out.insert(path.to_owned());
    } else {
        return Err(format!("usage input unavailable: {}", path.display()));
    }
    Ok(())
}
pub fn report_paths(paths: &[PathBuf]) -> Result<Report, String> {
    let mut inputs = BTreeSet::new();
    for path in paths {
        files(path, &mut inputs)?;
    }
    let mut logs = vec![];
    for path in inputs {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let log = text
            .split_inclusive('\n')
            .map(Envelope::from_line)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        logs.push(log);
    }
    report_logs(&logs)
}
pub fn report_logs(logs: &[Vec<Envelope>]) -> Result<Report, String> {
    let mut forks = BTreeMap::new();
    for log in logs {
        if let Some(Envelope {
            event: Event::SessionStarted { meta },
            ..
        }) = log.first()
            && let Some(old) = forks.insert(meta.session, meta.fork)
            && old != meta.fork
        {
            return Err("conflicting session lineage".into());
        }
    }
    fn origin(
        mut session: SessionId,
        seq: Seq,
        forks: &BTreeMap<SessionId, Option<(SessionId, Seq)>>,
    ) -> Result<Origin, String> {
        let mut seen = BTreeSet::new();
        while let Some(Some((parent, at))) = forks.get(&session) {
            if seq > *at || seq == Seq(1) {
                break;
            }
            if !seen.insert(session) {
                return Err("cyclic fork lineage".into());
            }
            session = *parent;
        }
        Ok((session, seq))
    }
    let mut entries = BTreeMap::<Origin, Entry>::new();
    for log in logs {
        let mut owners = BTreeMap::new();
        for e in log {
            let Some(seq) = e.seq else {
                continue;
            };
            let (usage, turn, scope) = match &e.event {
                Event::ItemStarted { item, turn, .. } => {
                    owners.insert(*item, *turn);
                    continue;
                }
                Event::ItemCompleted {
                    item,
                    body: Item::Usage { usage },
                    status: ItemStatus::Completed,
                } => (
                    usage,
                    owners.get(item).copied().flatten(),
                    usage.scope.clone().unwrap_or(UsageScope::Call),
                ),
                Event::TurnEnded {
                    turn,
                    usage: Some(usage),
                    ..
                } => (usage, Some(*turn), UsageScope::Turn),
                _ => continue,
            };
            // New forks retain original identities in bytes; old reserializing
            // forks are resolved from the header chain whenever it is available.
            let key = origin(e.session, seq, &forks)?;
            let entry = Entry {
                usage: usage.clone(),
                turn,
                scope,
                legacy: e.v.major == 0 || (e.v.major == 1 && e.v.minor == 0),
            };
            if let Some(previous) = entries.insert(key, entry.clone())
                && previous != entry
            {
                return Err(format!("conflicting usage at {}:{}", key.0, key.1.0));
            }
        }
    }
    let mut detailed_sessions = BTreeSet::new();
    for log in logs {
        if log.iter().any(|e| {
            matches!(&e.event,
            Event::ItemCompleted { body: Item::Usage { usage }, .. }
                if usage.scope != Some(UsageScope::Session))
                || matches!(e.event, Event::TurnEnded { usage: Some(_), .. })
        }) {
            for e in log {
                if let Some(seq) = e.seq {
                    detailed_sessions.insert(origin(e.session, seq, &forks)?.0);
                }
            }
        }
    }
    let calls: BTreeSet<_> = entries
        .values()
        .filter(|e| e.scope == UsageScope::Call)
        .filter_map(|e| e.turn)
        .collect();
    let mut latest = BTreeMap::<SessionId, Seq>::new();
    for ((session, seq), e) in &entries {
        if e.scope == UsageScope::Session {
            latest
                .entry(*session)
                .and_modify(|n| *n = (*n).max(*seq))
                .or_insert(*seq);
        }
    }
    // Group provenance as well as its least certain member: a partially absent
    // row cannot contaminate a fully reported total.
    let mut rows = BTreeMap::<String, Row>::new();
    for ((session, seq), entry) in entries {
        if entry.scope == UsageScope::Turn && entry.turn.is_some_and(|t| calls.contains(&t)) {
            continue;
        }
        if entry.scope == UsageScope::Session
            && (latest.get(&session) != Some(&seq) || detailed_sessions.contains(&session))
        {
            continue;
        }
        let mut usage = entry.usage;
        if entry.legacy {
            for (key, value) in [
                ("input_tokens", usage.input_tokens),
                ("output_tokens", usage.output_tokens),
            ] {
                if value == 0 && !usage.unreported.iter().any(|s| s == key) {
                    usage.unreported.push(key.into());
                }
            }
        }
        let source = if entry.legacy {
            NumberSource::Unknown(serde_json::json!("unknown"))
        } else {
            usage
                .source
                .clone()
                .unwrap_or_else(|| NumberSource::Unknown(serde_json::json!("unknown")))
        };
        let mut numbers = usage.numbers();
        numbers.remove("cost.amount_micros");
        numbers.remove("cost_micros_usd");
        let currency = usage
            .cost
            .as_ref()
            .map(|c| c.currency.clone())
            .or_else(|| usage.cost_micros_usd.map(|_| "USD".into()));
        let cost = usage
            .cost
            .as_ref()
            .map(|c| (c.amount_micros, c.source.clone()))
            .or_else(|| usage.cost_micros_usd.map(|n| (n, source.clone())));
        let cost_source = cost.clone().map(|(_, source)| source);
        if let Some((amount, _)) = cost {
            numbers.insert("cost_micros".into(), amount);
        }
        let (backend_kind, backend) = usage
            .backend
            .as_ref()
            .map(|b| (b.kind.clone(), b.name.clone()))
            .unwrap_or((
                BackendKind::Unknown(serde_json::json!("unknown")),
                "unknown".into(),
            ));
        let key = serde_json::to_string(&(
            &backend_kind,
            &backend,
            &usage.model,
            &currency,
            &entry.scope,
            &source,
            &cost_source,
        ))
        .map_err(|e| e.to_string())?;
        let row = rows.entry(key).or_insert_with(|| Row {
            backend_kind,
            backend,
            model: usage.model.clone(),
            currency,
            scope: entry.scope,
            source,
            cost_source,
            missing: BTreeMap::new(),
            numbers: BTreeMap::new(),
            records: 0,
        });
        row.records = row
            .records
            .checked_add(1)
            .ok_or("usage record count overflow")?;
        for name in usage.unreported {
            let count = row.missing.entry(name).or_default();
            *count = count.checked_add(1).ok_or("missing counter overflow")?;
        }
        for (key, value) in numbers {
            let total = row.numbers.entry(key).or_default();
            *total = total.checked_add(value).ok_or("usage total overflow")?;
        }
    }
    Ok(Report {
        rows: rows.into_values().collect(),
    })
}
pub fn render(report: &Report) -> String {
    let mut text = String::new();
    for row in &report.rows {
        text.push_str(&format!(
            "{:?}/{} · {} · {} · {:?} · {:?}: {} ({} records; missing: {})\n",
            row.backend_kind,
            row.backend,
            row.model.as_deref().unwrap_or("unreported model"),
            row.currency.as_deref().unwrap_or("no currency"),
            row.scope,
            row.source,
            row.numbers
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", "),
            row.records,
            row.missing
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    text
}
