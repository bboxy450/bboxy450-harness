// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Offscreen Metal frame witness. No window, display permission or model.
use super::{
    bridge::{FrameState, read_replay},
    view::HarnessView,
};
use bbt_render::{chrome::ChromeRenderer, device::DeviceContext};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

pub async fn run(replay: PathBuf, screenshot: PathBuf) -> Result<(), String> {
    let model = read_replay(replay).await?;
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
        println!(
            "gui-smoke: PASS · {} visible items, {} kinds, Metal {}×{} · {}",
            frame.visible.len(),
            actual.len(),
            width,
            height,
            screenshot.display()
        );
        Ok(())
    })
    .await
}
