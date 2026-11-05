// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Offscreen Metal frame witness. No window, display permission or model.
use super::{
    bridge::{FrameState, read_replay_with_redactor},
    options::Options,
    rich::{Faces, Toggle},
    view::{Action, Frame, HarnessView, Preparer, find, layout_text},
};
use bbt_render::{
    chrome::{ChromeRenderer, materials},
    device::DeviceContext,
};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

/// The frame as the window composes it: the flat face, the alive rims through the material
/// pass (frozen at time 0: the end state), then the menus and the zoom pill over both.
fn compose(
    renderer: &mut ChromeRenderer,
    frame: &super::view::Frame,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    materials::render_composed_to_rgba(
        renderer,
        materials::Scene {
            backdrop: materials::Backdrop::DrawList(&frame.list),
            items: &frame.rims,
            width,
            height,
            scale: 1.,
            time_ms: 0.,
            theme: frame.theme,
        },
        &frame.over,
    )
    .map_err(|e| e.to_string())
}

/// Render-only frame of a replay at a fixed size, for web/native parity. No
/// fixture assertions; the transcript shows from its top.
/// Chrome is the live standalone tree; `--no-chrome` omits its title bar.
/// `--open-menu` renders a floating menu open; the profile switcher lists
/// `$HOME/.bboxy450-harness/profiles` (switcher-safe fields only).
pub async fn capture(options: Options) -> Result<(), String> {
    let replay = options.replay.clone().ok_or("capture requires --replay")?;
    let model = read_replay_with_redactor(replay, crate::redact::Set::default()).await?;
    let profiles = match std::env::var_os("HOME") {
        Some(home) if options.open_menu.is_some() => {
            super::bridge::profile_names(&PathBuf::from(home).join(".bboxy450-harness/profiles"))
                .await
        }
        _ => vec![],
    };
    let context = DeviceContext::new().await.map_err(|e| e.to_string())?;
    crate::runtime::blocking(move || {
        let [width, height] = options.capture.ok_or("capture requires --capture WxH")?;
        let mut state = FrameState {
            transcript: Arc::new(model),
            ready: true,
            replay: true,
            profiles: Arc::new(profiles),
            ..Default::default()
        };
        let mut view = HarnessView::default();
        let frame = capture_frame(&mut view, &mut state, &options)?;
        let mut renderer = ChromeRenderer::with_context(context, super::assets::icons())
            .map_err(|e| e.to_string())?;
        let rgba = compose(&mut renderer, frame, width, height)?;
        write_png(&options.screenshot, width, height, &rgba)
    })
    .await
}

/// Start and solve a production capture without the GPU, so tests can inspect its nodes.
/// Call on the blocking preparation pool, just like `capture`.
pub fn capture_frame<'a>(
    view: &'a mut HarnessView,
    state: &mut FrameState,
    options: &Options,
) -> Result<&'a Frame, String> {
    use crate::protocol::ItemKind;
    let size = options.capture.ok_or("capture requires --capture WxH")?;
    let theme = view.theme();
    *view = HarnessView::standalone(options.zoom.unwrap_or_default());
    view.set_theme(theme);
    view.chrome = !options.no_chrome;
    let [width, height] = size.map(|v| v as f32);
    let mut preparer = Preparer::default();
    preparer.set_zoom(view.zoom());
    preparer.set_theme(view.theme());
    let mut faces = Faces::default();
    loop {
        let prepared = preparer.prepare_with_faces(state.transcript.clone(), width, &faces)?;
        let mut changed = false;
        if options.expand_all {
            for row in state.transcript.items.values() {
                let mut face = faces.get(&row.id).cloned().unwrap_or_default();
                let has = |key: &str| prepared.layouts().any(|layout| find(layout, key).is_some());
                let toggle = match row.kind {
                    ItemKind::Reasoning if !face.reasoning_open => Some(Toggle::Reasoning),
                    ItemKind::AgentMessage
                        if !face.output_closed && has(&format!("loop/{}", row.id)) =>
                    {
                        Some(Toggle::Output)
                    }
                    ItemKind::ToolCall
                        if has(&format!("tool/{}", row.id))
                            && !has(&format!("tool/{}/body", row.id)) =>
                    {
                        Some(Toggle::Output)
                    }
                    _ => None,
                };
                if let Some(toggle) = toggle {
                    // Exactly the state mutation used by Bridge::toggle on a click.
                    // Solved bodies preserve kind/status defaults, including auto-open tools.
                    face.apply(toggle);
                    faces.insert(row.id, face);
                    changed = true;
                }
            }
        }
        if !changed {
            state.faces = Arc::new(faces);
            state.prepared = Some(Arc::new(prepared));
            break;
        }
        // Opening a repeated-reply group exposes its cards on the next preparation.
        // Each iteration only opens disclosures in this immutable replay.
    }
    // Render once so the view holds this frame's profiles, then open the menu.
    view.render(state, [width, height])?;
    match options.open_menu {
        Some(super::options::Menu::Profiles) => view.toggle_profile_menu(),
        Some(super::options::Menu::Slash) => {
            view.edit(bbt_ui::text_edit::Command::Insert("/".into()))
        }
        None => {}
    }
    view.render(state, [width, height])
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
        let rgba = compose(&mut renderer, frame, width, height)?;
        use crate::protocol::ItemKind;
        // Native records and usage never draw in the transcript. Usage figures feed
        // the header meter, which a 0/0 total (this fixture) leaves out.
        let drawn = |v: &&super::model::ViewItem| !matches!(v.kind, ItemKind::Native | ItemKind::Usage);
        let expected: BTreeSet<_> = state
            .transcript
            .items
            .values()
            .filter(drawn)
            .map(|v| format!("{:?}", v.kind))
            .collect();
        let actual: BTreeSet<_> = frame
            .visible
            .iter()
            .map(|(_, k, _)| format!("{k:?}"))
            .collect();
        let drawn_count = state.transcript.items.values().filter(drawn).count();
        if actual != expected || frame.visible.len() != drawn_count {
            return Err(format!(
                "not every drawn item is visible: expected {expected:?}, got {actual:?}"
            ));
        }
        if super::view::find(frame.layout.as_ref().ok_or("missing solved layout")?, "meter").is_some() {
            return Err("a 0/0 usage total drew the header meter".into());
        }
        for row in state.transcript.items.values() {
            let trim_expected = match &row.body {
                crate::protocol::Item::ContextTrim { clipped_through } =>
                    Some(format!("Context trimmed through event {}", clipped_through.0)),
                _ => None,
            };
            let expected = match row.kind {
                ItemKind::UserMessage => "Read two files", ItemKind::AgentMessage => "I’ll read both files.",
                ItemKind::Reasoning => "Compare the current contents.", ItemKind::ToolCall => "",
                ItemKind::FileChange => "+ hello world", ItemKind::TurnDiff => "+++ b/hello.txt",
                // the head names the command; the body is the output alone (the call's text)
                ItemKind::Terminal => "hello", ItemKind::Plan => "In progress: Update greeting",
                ItemKind::Approval => "Edit hello.txt", ItemKind::Image => "Image: image/png",
                ItemKind::ContextTrim => trim_expected.as_deref().expect("context trim fixture body"),
                ItemKind::Usage | ItemKind::Native => {
                    // nothing in the feed: never "0 in · 0 out" (the operator's screenshot)
                    if frame.item_text(row.id).is_some_and(|t| !t.trim().is_empty()) { return Err(format!("hidden item {} drew a row: {:?}", row.id, frame.item_text(row.id))); }
                    continue;
                }
                ItemKind::Compaction => "Context compacted",
                ItemKind::Notice => "fixture complete", ItemKind::Subagent => "Session: 00000000000000000000000009",
                ItemKind::Command | ItemKind::Hook | ItemKind::SystemMessage => "",
                ItemKind::MemoryWrite | ItemKind::MemoryState | ItemKind::Trial | ItemKind::Adoption | ItemKind::Recall | ItemKind::Receipt => "",
                ItemKind::Unknown(_) => return Err("unexpected unknown fixture item".into()),
            };
            // a tool card names its action: the verb the rich face derives from the tool
            if let crate::protocol::Item::ToolCall { name, tool_kind, input_raw, output, .. } = &row.body
                && !frame.item_text(row.id).is_some_and(|text| text.contains(&super::face::describe(name, tool_kind.clone(), input_raw, output, "").verb)) { return Err(format!("tool card {} lacks its action", row.id)); }
            if !frame.item_text(row.id).is_some_and(|text| text.contains(expected)) { return Err(format!("card {} lacks {expected}", row.id)); }
        }
        let text = layout_text(frame.layout.as_ref().ok_or("missing solved layout")?);
        let required = [
            "Read two files and update hello.txt", "I’ll read both files.",
            "Compare the current contents.", "Read hello.txt", "printf ok",
            "--- a/hello.txt", "+ hello world", "In progress: Update greeting", "Edit hello.txt",
            "Image: image/png", "0 bytes",
            "Session: 00000000000000000000000009", "Context trimmed through event ", "fixture complete",
        ];
        for expected in required { if !text.contains(expected) { return Err(format!("missing readable content: {expected}; actual text: {text}")); } }
        if text.contains("\"kind\":") || text.contains("Some(") { return Err("serialized payload reached a card".into()); }
        if text.contains("0 in · 0 out") { return Err("a usage item drew its figures in the feed".into()); }
        // harness-spec.md "Tight transcript": success says nothing, no `auto` badge, no rounds
        for noise in ["exit 0", "auto", "round"] { if text.contains(noise) { return Err(format!("tight transcript drew {noise:?}: {text}")); } }
        let approval_count = frame.actions.iter().filter(|(_, a)| matches!(a, Action::Approval(_, _))).count();
        if approval_count != 3 || !["Allow", "Always", "Deny"].iter().all(|s| text.contains(s)) { return Err("approval buttons missing".into()); }
        let layout=frame.layout.as_ref().ok_or("smoke layout missing")?;
        for (index, id) in state.transcript.order.iter().enumerate() {
            // native records and usage have no row at all
            if matches!(state.transcript.items[id].kind, ItemKind::Native | ItemKind::Usage) { continue; }
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
            // Usage draws no row (harness-spec.md "Refinement pass 2"); checked above
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
        let scrolled = compose(&mut renderer, frame, 1120, 860)?;
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

#[cfg(test)]
mod gui_polish_capture {
    use super::*;
    #[tokio::test]
    async fn gui_polish_loader_and_pending_native_captures() {
        use crate::{
            frontends::gui::{bridge::PendingPrompt, model::Transcript},
            protocol::*,
        };
        let directory = std::env::var_os("GUI_POLISH_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("bboxy-gui-polish-captures"));
        tokio::fs::create_dir_all(&directory).await.unwrap();
        let device = DeviceContext::new().await.unwrap();
        crate::runtime::blocking(move || {
            let mut renderer =
                ChromeRenderer::with_context(device, super::super::assets::icons()).unwrap();
            let mut view = HarnessView::default();
            let mut state = FrameState::default();
            let (width, height) = (470, 680);
            let frame = view.render(&state, [width as f32, height as f32]).unwrap();
            write_png(
                &directory.join("loading.png"),
                width,
                height,
                &compose(&mut renderer, frame, width, height).unwrap(),
            )
            .unwrap();
            let mut event: Envelope = serde_json::from_str(
                include_str!("../../../tests/fixtures/protocol/all-items.jsonl")
                    .lines()
                    .next()
                    .unwrap(),
            )
            .unwrap();
            let mut model = Transcript::default();
            model.hello(event.session, event.epoch);
            state.ready = true;
            state.transcript = Arc::new(model);
            let op = OpId::new();
            state.pending_prompts = Arc::new(
                vec![(
                    op,
                    PendingPrompt::new(
                        op,
                        event.session,
                        event.epoch,
                        "Queued while the current turn runs".into(),
                    ),
                )]
                .into(),
            );
            let frame = view.render(&state, [width as f32, height as f32]).unwrap();
            write_png(
                &directory.join("pending.png"),
                width,
                height,
                &compose(&mut renderer, frame, width, height).unwrap(),
            )
            .unwrap();
            let item = ItemId::new();
            event.seq = Some(Seq(2));
            event.at = Seq(2);
            event.event = Event::ItemStarted {
                turn: None,
                item,
                kind: ItemKind::UserMessage,
                parent: None,
                head: None,
            };
            let mut model = (*state.transcript).clone();
            model.apply(&event);
            // A durable start has a cause, but no durable user body yet. It must
            // not paint a second, empty blue bubble beside the pending draft.
            state.transcript = Arc::new(model.clone());
            state.prepared = Some(Arc::new(
                Preparer::default()
                    .prepare(state.transcript.clone(), width as f32)
                    .unwrap(),
            ));
            let frame = view.render(&state, [width as f32, height as f32]).unwrap();
            assert!(
                !frame
                    .visible
                    .iter()
                    .any(|(_, kind, rect)| *kind == ItemKind::UserMessage && rect.h > 0.)
            );
            assert_eq!(
                layout_text(frame.layout.as_ref().unwrap())
                    .matches("YOU")
                    .count(),
                1
            );
            event.seq = Some(Seq(3));
            event.at = Seq(3);
            event.event = Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::UserMessage {
                    blocks: vec![InputBlock::Text {
                        text: "Queued while the current turn runs".into(),
                    }],
                },
            };
            model.apply(&event);
            state.transcript = Arc::new(model);
            state.pending_prompts = Default::default();
            state.prepared = Some(Arc::new(
                Preparer::default()
                    .prepare(state.transcript.clone(), width as f32)
                    .unwrap(),
            ));
            let frame = view.render(&state, [width as f32, height as f32]).unwrap();
            write_png(
                &directory.join("durable.png"),
                width,
                height,
                &compose(&mut renderer, frame, width, height).unwrap(),
            )
            .unwrap();
        })
        .await;
    }
}
