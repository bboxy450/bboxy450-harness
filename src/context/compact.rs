// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Deterministic digest specified by protocol ICD §4b; no inference or log rewrite.
use crate::protocol::*;
use std::collections::{BTreeMap, VecDeque};
fn clip(text: &str, n: usize) -> String {
    let mut chars = text.chars();
    let mut out = chars.by_ref().take(n).collect::<String>();
    if chars.next().is_some() {
        out.push('…');
    }
    out
}
fn field(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\\' => "\\\\".into(),
            '\n' => "\\n".into(),
            '\r' => "\\r".into(),
            '\t' => "\\t".into(),
            c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {
                format!("\\u{{{:04X}}}", c as u32)
            }
            c => c.to_string(),
        })
        .collect()
}
fn tag<T: serde::Serialize>(value: &T, key: &str) -> String {
    let value = serde_json::to_value(value).expect("protocol serializes");
    value
        .as_str()
        .or_else(|| value[key].as_str())
        .unwrap_or("unknown")
        .into()
}
struct Turn {
    seq: Seq,
    items: Vec<ItemId>,
    reason: Option<String>,
}
pub fn compact_summary(prev: Option<&str>, window: &[Envelope]) -> String {
    let mut turns = BTreeMap::<TurnId, Turn>::new();
    let mut bodies = BTreeMap::new();
    let mut through = Seq(0);
    for e in window {
        match &e.event {
            Event::TurnStarted { turn } => {
                turns
                    .entry(*turn)
                    .or_insert(Turn {
                        seq: e.seq.unwrap_or(e.at),
                        items: vec![],
                        reason: None,
                    })
                    .seq = e.seq.unwrap_or(e.at);
            }
            Event::ItemStarted {
                turn: Some(turn),
                item,
                ..
            } => {
                if let Some(t) = turns.get_mut(turn) {
                    t.items.push(*item);
                }
            }
            Event::ItemCompleted { item, status, body } => {
                bodies.insert(*item, (status, body));
            }
            Event::TurnEnded { turn, reason, .. } => {
                through = e.seq.unwrap_or(e.at);
                if let Some(t) = turns.get_mut(turn) {
                    t.reason = Some(tag(reason, "reason"));
                }
            }
            _ => {}
        }
    }
    let mut omitted = 0usize;
    let mut sections = VecDeque::<String>::new();
    if let Some(prev) = prev {
        for (index, line) in prev.lines().skip(1).enumerate() {
            if let Some(n) = line
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(" earlier turns omitted)"))
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|_| index == 0)
            {
                omitted = n;
            } else if line.starts_with("## Turn ") {
                sections.push_back(format!("{line}\n"));
            } else if let Some(last) = sections.back_mut() {
                last.push_str(line);
                last.push('\n');
            }
        }
    }
    let mut ordered = turns
        .values()
        .filter(|t| t.reason.is_some())
        .collect::<Vec<_>>();
    ordered.sort_by_key(|t| t.seq);
    for turn in ordered {
        let mut section = format!("## Turn {}\n", turn.seq.0);
        let mut assistant = None;
        for id in &turn.items {
            let Some((status, body)) = bodies.get(id) else {
                continue;
            };
            match body {
                Item::UserMessage { blocks } => {
                    let text = blocks
                        .iter()
                        .filter_map(|b| match b {
                            InputBlock::Text { text } => Some(text.clone()),
                            InputBlock::Image { blob } => Some(format!(
                                "[image {}]",
                                blob.sha256.chars().take(8).collect::<String>()
                            )),
                            InputBlock::Mention { path } => Some(format!("@{}", path.display())),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    section.push_str(&format!("User: {}\n", field(&clip(&text, 2000))));
                }
                Item::ToolCall {
                    name, input_raw, ..
                } => section.push_str(&format!(
                    "Tool: {} {} → {}\n",
                    field(name),
                    field(&clip(input_raw, 200)),
                    tag(status, "status")
                )),
                Item::Terminal { command, exit, .. } => {
                    let exit = exit.map(|n| format!(" exit {n}")).unwrap_or_default();
                    section.push_str(&format!(
                        "Tool: {} → {}{exit}\n",
                        field(&clip(command, 200)),
                        tag(status, "status")
                    ));
                }
                Item::FileChange { path, change, .. } => section.push_str(&format!(
                    "Changed: {} ({})\n",
                    field(&path.to_string_lossy()),
                    tag(change, "op")
                )),
                Item::AgentMessage { markdown } => assistant = Some(markdown),
                _ => {}
            }
        }
        if let Some(text) = assistant {
            section.push_str(&format!("Assistant: {}\n", field(&clip(text, 4000))));
        }
        section.push_str(&format!("Ended: {}\n", turn.reason.as_deref().unwrap()));
        sections.push_back(section);
    }
    loop {
        let mut out = format!(
            "# Earlier conversation (compacted through seq {})\n",
            through.0
        );
        if omitted > 0 {
            out.push_str(&format!("({omitted} earlier turns omitted)\n"));
        }
        for section in &sections {
            out.push_str(section);
        }
        if out.len() <= 32 * 1024 || sections.is_empty() {
            return out;
        }
        sections.pop_front();
        omitted += 1;
    }
}
