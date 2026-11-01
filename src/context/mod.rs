// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub mod auto;
pub mod compact;
use crate::{engine::history, protocol::*, provider::WireMessage, store::Blobs};
pub fn latest(log: &[Envelope]) -> Option<(&BlobRef, Seq)> {
    log.iter().rev().find_map(|e| match &e.event {
        Event::ItemCompleted {
            status: ItemStatus::Completed,
            body:
                Item::Compaction {
                    summary,
                    replaced_through,
                },
            ..
        } => Some((summary, *replaced_through)),
        _ => None,
    })
}
pub fn clipped_through(log: &[Envelope]) -> Seq {
    log.iter()
        .filter_map(|e| match e.event {
            Event::ItemCompleted {
                status: ItemStatus::Completed,
                body: Item::ContextTrim { clipped_through },
                ..
            } => Some(clipped_through),
            _ => None,
        })
        .max()
        .unwrap_or(Seq(0))
}
pub fn clip_output(text: &str) -> String {
    if text.len() <= 2048 {
        return text.into();
    }
    let mut end = 2048;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[clipped {} bytes]", &text[..end], text.len() - end)
}
/// Apply only durable reductions to completed bodies, preserving call/result IDs.
pub fn reduced(log: &[Envelope]) -> Vec<Envelope> {
    let replaced = latest(log).map(|(_, s)| s).unwrap_or(Seq(0));
    let clipped = clipped_through(log);
    let starts: std::collections::BTreeMap<_, _> = log
        .iter()
        .filter_map(|e| match e.event {
            Event::ItemStarted { item, .. } => e.seq.map(|s| (item, s)),
            _ => None,
        })
        .collect();
    log.iter()
        .filter(|e| e.seq.is_some_and(|s| s > replaced))
        .filter_map(|e| {
            let item = match e.event {
                Event::ItemStarted { item, .. } | Event::ItemCompleted { item, .. } => Some(item),
                _ => None,
            };
            if let Some(item) = item
                && starts.get(&item).is_some_and(|s| *s <= replaced)
            {
                return None;
            }
            let mut e = e.clone();
            if let Event::ItemCompleted { item, body, .. } = &mut e.event
                && starts.get(item).is_some_and(|s| *s <= clipped)
            {
                match body {
                    Item::ToolCall { output, .. } => {
                        output.retain(|b| !matches!(b, OutputBlock::Image { .. }));
                        *output = vec![OutputBlock::Text {
                            text: clip_output(&history::output_text(output)),
                        }];
                    }
                    Item::Reasoning { .. } | Item::Image { .. } => return None,
                    Item::UserMessage { blocks } => {
                        blocks.retain(|b| !matches!(b, InputBlock::Image { .. }))
                    }
                    _ => {}
                }
            }
            Some(e)
        })
        .collect()
}
pub async fn replay(log: &[Envelope], blobs: &Blobs) -> std::io::Result<(Vec<WireMessage>, usize)> {
    let mut messages = vec![];
    if let Some((blob, _)) = latest(log) {
        let summary = String::from_utf8(blobs.get(blob).await?).map_err(std::io::Error::other)?;
        messages.push(serde_json::json!({"role":"user","content":summary}));
    }
    let tail = reduced(log);
    let clipped = clipped_through(log);
    let stable = tail
        .iter()
        .filter(|e| e.seq.is_some_and(|s| s <= clipped))
        .cloned()
        .collect::<Vec<_>>();
    let prefix = 1 + messages.len() + history::to_wire(&stable).len();
    messages.extend(history::to_wire(&tail));
    Ok((messages, prefix))
}
