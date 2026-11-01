// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::protocol::*;
/// Append-only projection. Model/tool text cannot inject terminal controls.
pub fn render(envelope: &Envelope) -> Vec<u8> {
    let text = match &envelope.event {
        Event::SessionStarted { meta } => format!(
            "session {} · {}\n› ",
            meta.session,
            meta.profile.as_deref().unwrap_or("unconfigured")
        ),
        Event::ProfileChanged { profile, model } => format!("profile {profile} · {model}\n"),
        Event::ItemStarted {
            kind: ItemKind::ToolCall,
            ..
        } => String::new(),
        Event::ItemCompleted { body, status, .. } => match body {
            Item::AgentMessage { markdown } => format!("{markdown}\n"),
            Item::Reasoning { summary, .. } => format!("thinking: {summary}\n"),
            Item::ToolCall {
                name,
                input_raw,
                output,
                spill,
                ..
            } => format!(
                "▸ {name} {input_raw}\n{} {} ({} lines{})\n",
                if matches!(status, ItemStatus::Completed) {
                    "✓"
                } else {
                    "✗"
                },
                crate::engine::history::output_text(output)
                    .lines()
                    .next()
                    .unwrap_or(""),
                crate::engine::history::output_text(output).lines().count(),
                spill
                    .as_ref()
                    .map(|b| format!(", blob {}", b.sha256))
                    .unwrap_or_default()
            ),
            Item::FileChange { path, unified, .. } => format!("{}\n{unified}\n", path.display()),
            Item::Approval {
                subject,
                options,
                decision,
                ..
            } => format!(
                "approval: {} · {:?} · {:?}\n",
                subject.summary, options, decision
            ),
            Item::Plan { entries } => entries
                .iter()
                .map(|p| format!("[{:?}] {}\n", p.status, p.text))
                .collect(),
            Item::Notice { level, text } => format!("{level:?}: {text}\n"),
            _ => String::new(),
        },
        Event::TurnEnded { reason, .. } => format!("— {reason:?}\n› "),
        Event::OpRejected { reason, .. } => format!("rejected: {reason:?}\n"),
        Event::SessionRecovered {
            closed_items,
            closed_turns,
            ..
        } => format!("recovered {closed_items} items, {closed_turns} turns\n"),
        Event::Lagged { resume_after } => {
            format!("transcript lagged; reconnect after {}\n", resume_after.0)
        }
        Event::Reset { head, .. } => format!("transcript reset to {}\n", head.0),
        _ => String::new(),
    };
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect::<String>()
        .into_bytes()
}
