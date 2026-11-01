// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        assets,
        bridge::{FrameState, read_replay},
        view::{HarnessView, Preparer},
    },
    import::claude::{Options, convert_with},
    protocol::*,
};
use futures_util::StreamExt;
use std::{collections::BTreeSet, sync::Arc};
#[tokio::test]
async fn converted_synthetic_replay_renders_every_mapped_kind_offscreen() {
    let root = tempfile::tempdir().unwrap();
    let bytes = include_bytes!("fixtures/import/claude-synthetic.jsonl").to_vec();
    let mut stream = convert_with(
        std::io::Cursor::new(bytes),
        Options {
            supplied_redactor_only: true,
            blob_root: Some(root.path().into()),
            ..Default::default()
        },
    );
    let mut events = vec![];
    while let Some(e) = stream.next().await {
        events.push(e.unwrap());
    }
    laws::check_transcript(&events).unwrap();
    let replay = root.path().join("synthetic-converted.jsonl");
    let jsonl = events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap() + "\n")
        .collect::<String>();
    tokio::fs::write(&replay, jsonl).await.unwrap();
    let model = read_replay(replay).await.unwrap();
    let context = bbt_render::device::DeviceContext::new().await.unwrap();
    bboxy450_harness::runtime::blocking(move || {
        let mut state = FrameState {
            transcript: Arc::new(model),
            ready: true,
            replay: true,
            ..Default::default()
        };
        state.prepared = Some(Arc::new(
            Preparer::default()
                .prepare(state.transcript.clone(), 1120.)
                .unwrap(),
        ));
        let mut view = HarnessView::default();
        let frame = view.render(&state, [1120., 7000.]).unwrap();
        let visible = frame
            .visible
            .iter()
            .map(|(_, k, _)| format!("{k:?}"))
            .collect::<BTreeSet<_>>();
        for (kind, meaning) in [
            (ItemKind::UserMessage, "synthetic"),
            (ItemKind::AgentMessage, "greeting"),
            (ItemKind::Reasoning, "synthetic"),
            (ItemKind::ToolCall, "Bash"),
            (ItemKind::Terminal, "printf"),
            (ItemKind::FileChange, "hello world"),
            (ItemKind::TurnDiff, "hello.txt"),
            (ItemKind::Plan, "Update greeting"),
            (ItemKind::Image, "image/png"),
            (ItemKind::Subagent, "Session:"),
            (ItemKind::Compaction, "Compacted through"),
            (ItemKind::Usage, "12 tokens"),
            (ItemKind::Notice, "Claude"),
        ] {
            assert!(
                visible.contains(&format!("{kind:?}")),
                "ImportedKindVisible: {kind:?}"
            );
            assert!(
                frame.visible.iter().any(|(id, k, _)| *k == kind
                    && frame.item_text(*id).is_some_and(|s| s.contains(meaning))),
                "ImportedMeaningVisible: {kind:?}"
            );
        }
        bboxy450_harness::frontends::gui::layout_checks::check(
            state.prepared.as_ref().unwrap(),
            frame.layout.as_ref().unwrap(),
        )
        .unwrap();
        let mut renderer =
            bbt_render::chrome::ChromeRenderer::with_context(context, assets::icons()).unwrap();
        let rgba = renderer
            .render_to_rgba(&frame.list, 1120, 7000, 1.)
            .unwrap();
        assert!(
            rgba.as_chunks::<4>()
                .0
                .iter()
                .map(|v| [v[0], v[1], v[2]])
                .collect::<BTreeSet<_>>()
                .len()
                > 20
        );
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".bboxy/runs/import-claude-smoke.png");
        let mut encoder = png::Encoder::new(std::fs::File::create(path).unwrap(), 1120, 7000);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&rgba)
            .unwrap();
        println!(
            "import-gui-smoke: synthetic only, {} items, {} kinds",
            frame.visible.len(),
            visible.len()
        );
    })
    .await;
}
