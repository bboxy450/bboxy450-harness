// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Offscreen Metal frame witness. No window, display permission or model.
use super::{
    bridge::{FrameState, read_replay},
    view::{Action, HarnessView, layout_text},
};
use bbt_render::{chrome::ChromeRenderer, device::DeviceContext};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

pub async fn run(replay: PathBuf, screenshot: PathBuf) -> Result<(), String> {
    let mut model = read_replay(replay).await?;
    // The completed replay proves all kinds; an open copy also proves the
    // actionable approval state in the same offscreen frame.
    for id in model.order.clone() {
        let row = model.items.get_mut(&id).unwrap();
        if let crate::protocol::Item::Approval {
            decision, options, ..
        } = &mut row.body
        {
            *decision = None;
            *options = vec![
                crate::protocol::ApprovalDecisionKind::Allow,
                crate::protocol::ApprovalDecisionKind::AllowAlways,
                crate::protocol::ApprovalDecisionKind::Deny,
            ];
            row.status = None;
            row.completed = None;
        }
    }
    let context = DeviceContext::new().await.map_err(|e| e.to_string())?;
    crate::runtime::blocking(move || {
        let mut state = FrameState {
            transcript: Arc::new(model),
            ready: true,
            replay: true,
            ..Default::default()
        };
        state.prepared = Some(Arc::new(
            super::view::Preparer::default().prepare(state.transcript.clone(), 1120.)?,
        ));
        let mut view = HarnessView::default();
        let width = 1120;
        let height = 7000;
        let frame = view.render(&state, [width as f32, height as f32])?;
        let mut renderer = ChromeRenderer::with_context(context, super::assets::icons())
            .map_err(|e| e.to_string())?;
        let rgba = renderer
            .render_to_rgba(&frame.list, width, height, 1.)
            .map_err(|e| e.to_string())?;
        let expected: BTreeSet<_> = state
            .transcript
            .items
            .values()
            .map(|v| format!("{:?}", v.kind))
            .collect();
        let actual: BTreeSet<_> = frame
            .visible
            .iter()
            .map(|(_, k, _)| format!("{k:?}"))
            .collect();
        if actual != expected || frame.visible.len() != state.transcript.items.len() {
            return Err(format!(
                "not every item is visible: expected {expected:?}, got {actual:?}"
            ));
        }
        use crate::protocol::ItemKind;
        for row in state.transcript.items.values() {
            let compaction_expected=if let crate::protocol::Item::Compaction {replaced_through,..}=&row.body {Some(format!("Compacted through event {}",replaced_through.0))}else{None};
            let expected = match row.kind {
                ItemKind::UserMessage => "Read two files", ItemKind::AgentMessage => "I’ll read both files.",
                ItemKind::Reasoning => "Compare the current contents.", ItemKind::ToolCall => "Arguments: path: hello.txt",
                ItemKind::FileChange => "+hello world", ItemKind::TurnDiff => "+++ b/hello.txt",
                ItemKind::Terminal => "printf ok", ItemKind::Plan => "In progress: Update greeting",
                ItemKind::Approval => "Edit hello.txt", ItemKind::Image => "Image: image/png",
                ItemKind::ContextTrim => "Context trimmed through event ",
                ItemKind::Usage => "Input: 0 tokens", ItemKind::Compaction => compaction_expected.as_deref().expect("compaction fixture body"),
                ItemKind::Notice => "fixture complete", ItemKind::Subagent => "Session: 00000000000000000000000009",
                ItemKind::Unknown => return Err("unexpected unknown fixture item".into()),
            };
            if let crate::protocol::Item::ToolCall { name, .. } = &row.body
                && !frame.item_text(row.id).is_some_and(|text| text.contains(name)) { return Err(format!("tool card {} lacks its name", row.id)); }
            if !frame.item_text(row.id).is_some_and(|text| text.contains(expected)) { return Err(format!("card {} lacks {expected}", row.id)); }
        }
        let text = layout_text(frame.layout.as_ref().ok_or("missing solved layout")?);
        let required = [
            "Read two files and update hello.txt", "I’ll read both files.",
            "Compare the current contents.", "Arguments: path: hello.txt", "printf ok", "Exit code: 0",
            "--- a/hello.txt", "+hello world", "In progress: Update greeting", "Edit hello.txt",
            "Image: image/png", "0 bytes", "Input: 0 tokens", "Output: 0 tokens",
            "Session: 00000000000000000000000009", "Context trimmed through event ", "fixture complete",
        ];
        for expected in required { if !text.contains(expected) { return Err(format!("missing readable content: {expected}; actual text: {text}")); } }
        if text.contains("\"kind\":") || text.contains("Some(") { return Err("serialized payload reached a card".into()); }
        let approval_count = frame.actions.iter().filter(|(_, a)| matches!(a, Action::Approval(_, _))).count();
        if approval_count != 3 || !["Allow", "Always", "Deny"].iter().all(|s| text.contains(s)) { return Err("approval buttons missing".into()); }
        let mut bottom = 0.;
        for (id, kind, r) in &frame.visible {
            if r.y < bottom - 0.5 {
                return Err(format!("overlapping item {id}"));
            }
            bottom = r.bottom();
            let mut colors = BTreeSet::new();
            for y in r.y.ceil().max(0.) as u32..r.bottom().floor().min(height as f32) as u32 {
                for x in r.x.ceil().max(0.) as u32..r.right().floor().min(width as f32) as u32 {
                    let i = ((y * width + x) * 4) as usize;
                    colors.insert([rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]);
                    if colors.len() > 8 {
                        break;
                    }
                }
                if colors.len() > 8 {
                    break;
                }
            }
            if colors.len() <= 8 {
                return Err(format!("blank item region {kind:?} ({id})"));
            }
        }
        let kinds: BTreeSet<_> = [
            "UserMessage",
            "AgentMessage",
            "Reasoning",
            "ToolCall",
            "FileChange",
            "TurnDiff",
            "Terminal",
            "Plan",
            "Approval",
            "Image",
            "Compaction",
            "Usage",
            "Notice",
            "Subagent",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        if !kinds.is_subset(&actual) {
            return Err(format!(
                "all-items fixture is missing kinds: {:?}",
                kinds.difference(&actual).collect::<Vec<_>>()
            ));
        }
        if let Some(parent) = screenshot.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let file = std::fs::File::create(&screenshot).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&rgba))
            .map_err(|e| e.to_string())?;
        let item_count = frame.visible.len();
        let kinds_count = actual.len();
        let frame = view.render(&state, [1120., 860.])?;
        let max = frame.max_scroll;
        if max <= 0. { return Err("smoke needs a scrollable viewport".into()); }
        view.scroll.wheel(max, max, -max / 2.);
        view.invalidate();
        let frame = view.render(&state, [1120., 860.])?;
        if frame.scroll <= 0. || frame.scroll >= max || frame.visible.is_empty() || frame.visible.len() >= item_count { return Err("scrolled viewport did not virtualize visible rows".into()); }
        if layout_text(frame.layout.as_ref().ok_or("scrolled layout missing")?).is_empty() { return Err("scrolled viewport has no readable text".into()); }
        let scrolled = renderer.render_to_rgba(&frame.list, 1120, 860, 1.).map_err(|e| e.to_string())?;
        let scrolled_path = screenshot.with_file_name("gui-smoke-scrolled.png");
        let file = std::fs::File::create(scrolled_path).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(file, 1120, 860); encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().and_then(|mut w| w.write_image_data(&scrolled)).map_err(|e| e.to_string())?;
        println!(
            "gui-smoke: PASS · {} visible items, {} kinds, readable payloads, 3 approval buttons, scrolled viewport, Metal {}×{} · {}",
            item_count,
            kinds_count,
            width,
            height,
            screenshot.display()
        );
        Ok(())
    })
    .await
}
