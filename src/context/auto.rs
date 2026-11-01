// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Durable automatic-compaction settings and replay boundaries (protocol §4d).
use crate::protocol::*;
#[derive(Clone, Debug)]
pub struct Options {
    pub enabled: bool,
    pub window: Option<u64>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            enabled: true,
            window: None,
        }
    }
}
/// Latest ending below every still-open started turn (§4b's quiescent frontier).
/// Cancelling an unstarted queued turn cannot move the cut across a running turn.
pub fn ended_through(log: &[Envelope]) -> Seq {
    let mut open = std::collections::BTreeMap::new();
    for e in log {
        match e.event {
            Event::TurnStarted { turn } => {
                if let Some(seq) = e.seq {
                    open.insert(turn, seq);
                }
            }
            Event::TurnEnded { turn, .. } => {
                open.remove(&turn);
            }
            _ => {}
        }
    }
    let first = open.values().min().copied();
    log.iter()
        .rev()
        .find_map(|e| match e.event {
            Event::TurnEnded { .. } => e.seq.filter(|s| first.is_none_or(|start| *s < start)),
            _ => None,
        })
        .unwrap_or(Seq(0))
}
pub fn trigger(estimate: u64, max_tokens: u64, window: u64) -> bool {
    u128::from(estimate.saturating_add(max_tokens)) * 5 > u128::from(window) * 4
}
pub fn fits(estimate: u64, max_tokens: u64, window: u64) -> bool {
    u128::from(estimate.saturating_add(max_tokens)) * 5 <= u128::from(window) * 3
}
/// Exact compact JSON accounting; metadata is excluded by protocol §4d.
pub fn estimate(
    log: &[Envelope],
    messages: &[serde_json::Value],
    tools: &[crate::provider::ToolDef],
) -> u64 {
    let mut anchor = None;
    for e in log {
        if let Event::ItemCompleted {
            status: ItemStatus::Completed,
            body,
            ..
        } = &e.event
        {
            match body {
                Item::Usage { usage } => anchor = e.seq.map(|s| (s, usage)),
                Item::ContextTrim { .. } | Item::Compaction { .. } => anchor = None,
                _ => {}
            }
        }
    }
    let bytes =
        |v: &serde_json::Value| serde_json::to_vec(v).expect("JSON value serializes").len() as u64;
    if let Some((seq, usage)) = anchor {
        let added = crate::engine::history::tagged(&super::reduced(log))
            .into_iter()
            .filter(|(s, _)| *s > seq)
            .map(|(_, m)| bytes(&m))
            .sum::<u64>();
        usage
            .input_tokens
            .saturating_add(usage.output_tokens)
            .saturating_add(added.div_ceil(4))
    } else {
        let n =
            messages.iter().map(bytes).sum::<u64>() + bytes(&crate::provider::tools_wire(tools));
        n.div_ceil(4)
    }
}
