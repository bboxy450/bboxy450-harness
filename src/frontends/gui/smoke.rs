// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Offscreen Metal frame witness. No window, display permission or model.
use super::{
    bridge::{FrameState, read_replay_with_redactor},
    view::{Action, HarnessView, layout_text},
};
use bbt_render::{chrome::ChromeRenderer, device::DeviceContext};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

/// Render-only frame of a replay at a fixed size, for web/native parity. No
/// fixture assertions; the transcript shows from its top.
/// `menu` renders a floating menu open; the profile switcher lists
/// `$HOME/.bboxy450-harness/profiles` (switcher-safe fields only).
pub async fn capture(
    replay: PathBuf,
    screenshot: PathBuf,
    size: [u32; 2],
    menu: Option<super::options::Menu>,
) -> Result<(), String> {
    let model = read_replay_with_redactor(replay, crate::redact::Set::default()).await?;
    let profiles = match std::env::var_os("HOME") {
        Some(home) if menu.is_some() => {
            super::bridge::profile_names(&PathBuf::from(home).join(".bboxy450-harness/profiles"))
                .await
        }
        _ => vec![],
    };
    let context = DeviceContext::new().await.map_err(|e| e.to_string())?;
    crate::runtime::blocking(move || {
        let [width, height] = size;
        let mut state = FrameState {
            transcript: Arc::new(model),
            ready: true,
            replay: true,
            profiles: Arc::new(profiles),
            ..Default::default()
        };
        state.prepared = Some(Arc::new(
            super::view::Preparer::default().prepare(state.transcript.clone(), width as f32)?,
        ));
        let mut view = HarnessView::default();
        // Render once so the view holds this frame's profiles, then open.
        view.render(&state, [width as f32, height as f32])?;
        match menu {
            Some(super::options::Menu::Profiles) => view.toggle_profile_menu(),
            Some(super::options::Menu::Slash) => {
                view.edit(bbt_ui::text_edit::Command::Insert("/".into()))
            }
            None => {}
        }
        let frame = view.render(&state, [width as f32, height as f32])?;
        let mut renderer = ChromeRenderer::with_context(context, super::assets::icons())
            .map_err(|e| e.to_string())?;
        let rgba = renderer
            .render_to_rgba(&frame.list, width, height, 1.)
            .map_err(|e| e.to_string())?;
        write_png(&screenshot, width, height, &rgba)
    })
    .await
}

pub async fn run(replay: PathBuf, screenshot: PathBuf) -> Result<(), String> {
    let redactor = crate::redact::Set::default();
    crate::capability::register(&redactor).await;
    let notice = redactor.capability_notice();
    if let Some(text) = &notice {
        println!("gui-smoke Notice: {text}");
    }
    let mut model = read_replay_with_redactor(replay, redactor).await?;
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
            notice,
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
        let checked = super::layout_checks::check(state.prepared.as_ref().ok_or("missing prepared rows")?, frame.layout.as_ref().ok_or("missing solved layout")?)?;
        println!("gui-smoke structural: {checked:?}");
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
            let compaction_expected = match &row.body {
                crate::protocol::Item::Compaction { replaced_through, .. } =>
                    Some(format!("Compacted through event {}", replaced_through.0)),
                _ => None,
            };
            let trim_expected = match &row.body {
                crate::protocol::Item::ContextTrim { clipped_through } =>
                    Some(format!("Context trimmed through event {}", clipped_through.0)),
                _ => None,
            };
            let expected = match row.kind {
                ItemKind::UserMessage => "Read two files", ItemKind::AgentMessage => "I’ll read both files.",
                ItemKind::Reasoning => "Compare the current contents.", ItemKind::ToolCall => "",
                ItemKind::FileChange => "+ hello world", ItemKind::TurnDiff => "+++ b/hello.txt",
                ItemKind::Terminal => "printf ok", ItemKind::Plan => "In progress: Update greeting",
                ItemKind::Approval => "Edit hello.txt", ItemKind::Image => "Image: image/png",
                ItemKind::ContextTrim => trim_expected.as_deref().expect("context trim fixture body"),
                ItemKind::Usage => "0 in · 0 out", ItemKind::Compaction => compaction_expected.as_deref().expect("compaction fixture body"),
                ItemKind::Notice => "fixture complete", ItemKind::Subagent => "Session: 00000000000000000000000009",
                ItemKind::Unknown => return Err("unexpected unknown fixture item".into()),
            };
            // a tool card names its action: the verb the rich face derives from the tool
            if let crate::protocol::Item::ToolCall { name, tool_kind, input_raw, output, .. } = &row.body
                && !frame.item_text(row.id).is_some_and(|text| text.contains(&super::face::describe(name, *tool_kind, input_raw, output, "").verb)) { return Err(format!("tool card {} lacks its action", row.id)); }
            if !frame.item_text(row.id).is_some_and(|text| text.contains(expected)) { return Err(format!("card {} lacks {expected}", row.id)); }
        }
        let text = layout_text(frame.layout.as_ref().ok_or("missing solved layout")?);
        let required = [
            "Read two files and update hello.txt", "I’ll read both files.",
            "Compare the current contents.", "Read hello.txt", "printf ok", "exit 0",
            "--- a/hello.txt", "+ hello world", "In progress: Update greeting", "Edit hello.txt",
            "Image: image/png", "0 bytes", "0 in · 0 out",
            "Session: 00000000000000000000000009", "Context trimmed through event ", "fixture complete",
        ];
        for expected in required { if !text.contains(expected) { return Err(format!("missing readable content: {expected}; actual text: {text}")); } }
        if text.contains("\"kind\":") || text.contains("Some(") { return Err("serialized payload reached a card".into()); }
        let approval_count = frame.actions.iter().filter(|(_, a)| matches!(a, Action::Approval(_, _))).count();
        if approval_count != 3 || !["Allow", "Always", "Deny"].iter().all(|s| text.contains(s)) { return Err("approval buttons missing".into()); }
        let layout=frame.layout.as_ref().ok_or("smoke layout missing")?;
        for (index, id) in state.transcript.order.iter().enumerate() {
            // folded into its tool card: no row of its own, but addressable inside the card
            let Some(card)=super::view::find(layout,&format!("e{index}")).map(|n|n.rect) else {
                super::view::find(layout,&format!("item/{id}")).ok_or("folded item is not addressable")?;
                continue;
            };
            let body=super::view::find(layout,&format!("body/{id}")).ok_or("missing inset item body")?.rect;
            if body.x < card.x + 4. || body.right() > card.right() - 4. || body.y < card.y || body.bottom() > card.bottom()+0.5 {
                return Err(format!("body {id} is outside its inset card: body={body:?}, card={card:?}"));
            }
        }

        fn colours(node: &bbt_ui::components::Layout)->BTreeSet<[u32;4]> {
            let mut out=BTreeSet::new();if let bbt_ui::components::LayoutKind::Text{runs,..}=&node.kind {out.extend(runs.iter().map(|r|r.style.color.map(f32::to_bits)));}
            if let bbt_ui::components::LayoutKind::Path {fill:Some(fill),..}=&node.kind {out.insert(fill.map(f32::to_bits));}
            for child in &node.children {out.extend(colours(child));}out
        }
        fn highlighted(node:&bbt_ui::components::Layout)->bool {
            if let bbt_ui::components::LayoutKind::Text {..}=&node.kind
                && layout_text(node).contains("highlight_witness") && colours(node).len()>1 {return true;}
            node.children.iter().any(highlighted)
        }
        if !highlighted(layout) {return Err("code block has no syntax-highlighted colour spans".into());}
        let change=state.transcript.items.values().find(|r|r.kind==crate::protocol::ItemKind::FileChange).ok_or("missing diff")?;
        let removed=super::view::find(layout,&format!("diff/{}/line/0",change.id)).ok_or("diff remove line missing")?;
        let added=super::view::find(layout,&format!("diff/{}/line/1",change.id)).ok_or("diff add line missing")?;
        if !layout_text(removed).contains("− hello")||!layout_text(added).contains("+ hello world")||colours(removed)==colours(added) {return Err("diff add/remove meaning or colours missing".into());}
        for row in state.transcript.items.values().filter(|r|r.kind==crate::protocol::ItemKind::ToolCall) {
            let node=super::view::find(layout,&format!("tool/{}/status",row.id)).ok_or("tool status disc missing")?;
            let ok=bbt_ui::Paint::solid(bbt_ui::Color::Rgba(super::face::ink(super::view::context()).ok)).resolve(super::view::context())?;
            if fill(node)!=Some(ok.map(f32::to_bits)) {return Err("tool status disc lacks the completed colour".into());}
        }
        fn fill(node:&bbt_ui::components::Layout)->Option<[u32;4]> {
            if let bbt_ui::components::LayoutKind::Box{style,..}=&node.kind && style.fill[3]>0. {return Some(style.fill.map(f32::to_bits));}
            node.children.iter().find_map(fill)
        }
        let approval=state.transcript.items.values().find(|r|r.kind==crate::protocol::ItemKind::Approval).ok_or("missing approval")?;
        let allow=super::view::find(layout,&format!("approval/{}/Allow",approval.id)).and_then(fill).ok_or("allow surface missing")?;
        let deny=super::view::find(layout,&format!("approval/{}/Deny",approval.id)).and_then(fill).ok_or("deny surface missing")?;
        let always=super::view::find(layout,&format!("approval/{}/Always",approval.id)).and_then(fill).ok_or("always surface missing")?;
        if allow==deny || allow==always || always==deny {return Err("approval choices need three distinct surfaces".into());}
        let body=super::view::find(layout,&format!("body/{}",approval.id)).ok_or("approval body missing")?.rect;
        for name in ["Allow","Always","Deny"] {
            let button=super::view::find(layout,&format!("approval/{}/{name}",approval.id)).ok_or("approval button missing")?;
            if button.rect.w >= body.w * 0.3 {return Err(format!("approval {name} is not compact: {} of {}",button.rect.w,body.w));}
            if layout_text(button).trim()!=name {return Err(format!("approval {name} has unexpected key/badge content"));}
        }
        for row in state.transcript.items.values().filter(|r|r.kind==ItemKind::ToolCall) {
            fn inset_text(node:&bbt_ui::components::Layout, area:bbt_ui_model::layout::Rect)->bool {
                if let bbt_ui::components::LayoutKind::Text {runs,..}=&node.kind
                    && runs.iter().any(|r| !r.text.trim().is_empty() && (r.rect.x < area.x+7.5 || r.rect.y < area.y+5.5 || r.rect.right()>area.right()-7.5 || r.rect.bottom()>area.bottom()-5.5)) {return false;}
                node.children.iter().all(|child|inset_text(child,area))
            }
            let card=super::view::find(layout,&format!("tool/{}",row.id)).ok_or("tool card missing")?;
            if !inset_text(card,card.rect) {return Err("tool text touches the card border".into());}
            super::view::find(layout,&format!("toolhead/{}",row.id)).ok_or("tool disclosure header missing")?;
        }
        let trim=state.transcript.items.values().find(|r|r.kind==ItemKind::ContextTrim).ok_or("fixture lacks ContextTrim")?;
        let trim_body=super::view::find(layout,&format!("body/{}",trim.id)).ok_or("trim callout missing")?;
        if !layout_text(trim_body).contains("2 KiB") || !layout_text(trim_body).contains("model context") {return Err("trim callout lacks its effect".into());}
        fn emphasized(node:&bbt_ui::components::Layout)->bool {
            if let bbt_ui::components::LayoutKind::Text {runs,..}=&node.kind && runs.iter().any(|r| r.style.weight>=600) {return true;}
            node.children.iter().any(emphasized)
        }
        if !emphasized(trim_body) {return Err("trim callout has no rich heading/emphasis".into());}
        let mut bottom = 0.;
        let mut rows: Vec<bbt_ui_model::layout::Rect> = vec![];
        for (id, kind, r) in &frame.visible {
            // a folded item paints inside an earlier tool card's row, not after it
            let nested = rows.iter().any(|row| r.x >= row.x - 0.5 && r.right() <= row.right() + 0.5 && r.y >= row.y - 0.5 && r.bottom() <= row.bottom() + 0.5);
            if !nested {
                rows.push(*r);
            }
            if !nested && r.y < bottom - 0.5 {
                return Err(format!("overlapping item {id}"));
            }
            if !nested {
                bottom = r.bottom();
            }
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
            "ContextTrim",
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

fn write_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(rgba))
        .map_err(|e| e.to_string())
}
