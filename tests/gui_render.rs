// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "embedded")]
#[path = "support/quiet_replay.rs"]
mod quiet_replay;
mod support;
use bboxy450_harness::{
    frontends::gui::{
        bridge::{Bridge, FrameState, read_replay},
        model::Transcript,
        view::{Action, HarnessView, Preparer},
    },
    protocol::*,
};
use std::{sync::Arc, time::Duration};
struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
async fn prepare(
    preparer: &mut Preparer,
    source: Arc<Transcript>,
    width: f32,
) -> bboxy450_harness::frontends::gui::view::PreparedRows {
    let mut owned = std::mem::take(preparer);
    let (next, result) = bboxy450_harness::runtime::blocking(move || {
        let result = owned.prepare(source, width);
        (owned, result)
    })
    .await;
    *preparer = next;
    result.unwrap()
}

#[tokio::test]
async fn full_window_capture_renders_its_shared_title_bar_assets_to_png() {
    use bboxy450_harness::frontends::gui::{options::Options, smoke::capture};
    let temp = tempfile::tempdir().unwrap();
    let mut images = vec![];
    for (name, extra) in [
        ("window", vec![]),
        ("face", vec!["--no-chrome"]),
        ("zoom", vec!["--zoom", "1.2"]),
    ] {
        let path = temp.path().join(format!("{name}.png"));
        let options = Options::parse(
            [
                "--replay",
                "tests/fixtures/parity/acp-claude-acp.jsonl",
                "--capture",
                "900x1100",
                "--screenshot",
                path.to_str().unwrap(),
            ]
            .into_iter()
            .chain(extra)
            .map(String::from),
        )
        .unwrap();
        capture(options).await.unwrap();
        let decoder = png::Decoder::new(std::io::Cursor::new(std::fs::read(path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (900, 1100));
        let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut rgba).unwrap();
        images.push(rgba);
    }
    assert_ne!(
        images[0], images[1],
        "title bar must change the captured pixels"
    );
    assert_ne!(images[0], images[2], "zoom must change the captured pixels");
}

#[tokio::test]
async fn capture_defaults_to_live_chrome_and_zoom_reprepares_text_at_120_percent() {
    use bboxy450_harness::frontends::gui::{
        options::Options,
        smoke::capture_frame,
        view::{find, layout_text},
    };
    let fixture = "tests/fixtures/parity/acp-claude-acp.jsonl";
    let transcript = Arc::new(read_replay(fixture.into()).await.unwrap());
    bboxy450_harness::runtime::blocking(move || {
        let mut state = FrameState {
            transcript,
            ready: true,
            replay: true,
            ..Default::default()
        };
        let options = |extra: &[&str]| {
            Options::parse(
                ["--replay", fixture, "--capture", "900x32000"]
                    .into_iter()
                    .chain(extra.iter().copied())
                    .map(String::from),
            )
            .unwrap()
        };
        let mut view = HarnessView::default();
        let full = capture_frame(&mut view, &mut state, &options(&[])).unwrap();
        let full = full.layout.as_ref().unwrap().clone();
        for key in [
            "bar",
            "bar/logo",
            "bar/win/i",
            "bar/win/i1",
            "bar/win/i2",
            "bar/tbtn1",
        ] {
            assert!(find(&full, key).is_some(), "default capture lacks {key}");
        }
        assert!(layout_text(find(&full, "bar").unwrap()).contains("claude-acp · acpdemo"));
        let face = capture_frame(&mut view, &mut state, &options(&["--no-chrome"])).unwrap();
        let face = face.layout.as_ref().unwrap();
        assert!(find(face, "bar").is_none());
        assert_eq!(find(face, "toolbar").unwrap().rect.y, 0.);
        let zoomed = capture_frame(&mut view, &mut state, &options(&["--zoom", "1.2"])).unwrap();
        let zoomed = zoomed.layout.as_ref().unwrap();
        assert_eq!(state.prepared.as_ref().unwrap().zoom, 120);
        fn text(n: &bbt_ui::components::Layout, out: &mut Vec<(String, Vec<f32>, f32)>) {
            if let bbt_ui::components::LayoutKind::Text { runs, .. } = &n.kind {
                out.push((
                    layout_text(n),
                    runs.iter().map(|r| r.style.size).collect(),
                    n.rect.h,
                ));
            }
            for child in &n.children {
                text(child, out);
            }
        }
        let reply = state
            .transcript
            .items
            .values()
            .find(|r| r.kind == ItemKind::AgentMessage)
            .unwrap();
        for key in [
            "toolbar".to_string(),
            "composer".to_string(),
            format!("body/{}", reply.id),
        ] {
            let (mut before, mut after) = (vec![], vec![]);
            text(find(&full, &key).unwrap(), &mut before);
            text(find(zoomed, &key).unwrap(), &mut after);
            assert!(!before.is_empty(), "{key} has no measurable text");
            assert_eq!(before.len(), after.len(), "{key}");
            for ((s, a, h), (t, b, k)) in before.iter().zip(&after) {
                assert_eq!(s, t, "{key}");
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    assert!((b / a - 1.2).abs() < 0.001, "{key}: font {a} -> {b}");
                }
                assert!((k - h * 1.2).abs() <= 1., "{key}: text height {h} -> {k}");
            }
        }
        let (mut bar, mut zoomed_bar) = (vec![], vec![]);
        text(find(&full, "bar").unwrap(), &mut bar);
        text(find(zoomed, "bar").unwrap(), &mut zoomed_bar);
        assert_eq!(
            bar, zoomed_bar,
            "the live title bar stays outside text zoom"
        );
    })
    .await;
}

#[tokio::test]
async fn unstyled_items_are_quiet_notice_lines() {
    use bboxy450_harness::frontends::gui::{
        smoke::capture_frame,
        view::{find, layout_text},
    };
    let events = quiet_replay::notices();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("notices.jsonl");
    let bytes = events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap() + "\n")
        .collect::<String>();
    std::fs::write(&path, bytes).unwrap();
    let transcript = Arc::new(read_replay(path).await.unwrap());
    bboxy450_harness::runtime::blocking(move || {
        let mut state = FrameState {
            transcript,
            ready: true,
            replay: true,
            ..Default::default()
        };
        let mut view = HarnessView::default();
        let frame = capture_frame(
            &mut view,
            &mut state,
            &bboxy450_harness::frontends::gui::options::Options::parse(
                [
                    "--replay",
                    "unused.jsonl",
                    "--capture",
                    "900x1100",
                    "--no-chrome",
                ]
                .map(String::from),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(frame.visible.len(), 5);
        for ((id, _, _), expected) in frame.visible.iter().zip([
            "Ran /help",
            "Hook fixture-hook",
            "Session instructions",
            "Context compacted",
            "Unsupported item: future_renderer",
        ]) {
            assert_eq!(frame.item_text(*id).unwrap().trim(), expected);
            let layout = frame.layout.as_ref().unwrap();
            let body = find(layout, &format!("body/{id}")).unwrap();
            fn has_hairline(node: &bbt_ui::components::Layout) -> bool {
                (matches!(&node.kind, bbt_ui::components::LayoutKind::Rects { .. })
                    && node.rect.h == 1.)
                    || node.children.iter().any(has_hairline)
            }
            if expected == "Context compacted" {
                assert!(has_hairline(body), "CompactionHairline");
            } else {
                assert!(body.rect.h <= 18., "QuietNoticeHasNoCardChrome: {expected}");
            }
        }
        let text = layout_text(frame.layout.as_ref().unwrap());
        assert!(
            !text.contains("STAYS_IN_LOG")
                && !text.contains("Completed")
                && !text.contains("SYSTEM")
        );
    })
    .await;
}

#[tokio::test]
async fn native_chunks_do_not_change_replayed_rows() {
    use bboxy450_harness::frontends::gui::{view::layout_text, zoom::Zoom};
    for count in [1, 16] {
        let with_native = quiet_replay::replay(count);
        let without_native = quiet_replay::without_native(&with_native);
        for (native_only, percent) in [
            (false, 90),
            (false, 100),
            (false, 160),
            (true, 90),
            (true, 100),
            (true, 160),
        ] {
            let mut frames = vec![];
            for mut events in [without_native.clone(), with_native.clone()] {
                if native_only {
                    let keep: std::collections::HashSet<_> = events
                        .iter()
                        .filter_map(|e| match e.event {
                            Event::ItemStarted {
                                item,
                                kind: ItemKind::Native,
                                ..
                            } => Some(item),
                            _ => None,
                        })
                        .collect();
                    events.retain(|e| match e.event {
                        Event::ItemStarted { item, .. }
                        | Event::ItemDelta { item, .. }
                        | Event::ItemSnapshot { item, .. }
                        | Event::ItemCompleted { item, .. } => keep.contains(&item),
                        _ => true,
                    });
                    quiet_replay::resequence(&mut events);
                    laws::check_transcript(&events).unwrap();
                }
                let temp = tempfile::tempdir().unwrap();
                let path = temp.path().join("native-chunks.jsonl");
                let bytes = events
                    .iter()
                    .map(|e| serde_json::to_string(e).unwrap() + "\n")
                    .collect::<String>();
                std::fs::write(&path, bytes).unwrap();
                let transcript = Arc::new(read_replay(path).await.unwrap());
                let frame = bboxy450_harness::runtime::blocking(move || {
                    let mut state = FrameState {
                        transcript,
                        ready: true,
                        replay: true,
                        ..Default::default()
                    };
                    let mut view = HarnessView::default();
                    view.chrome = true;
                    let zoom = Zoom::from_percent(percent);
                    view.restore_zoom(zoom, &Bridge::new(Arc::new(Wake)));
                    let mut preparer = Preparer::default();
                    preparer.set_zoom(zoom);
                    state.prepared = Some(Arc::new(
                        preparer.prepare(state.transcript.clone(), 900.).unwrap(),
                    ));
                    let frame = view.render(&state, [900., 1100.]).unwrap();
                    assert!(
                        bboxy450_harness::frontends::gui::view::find(
                            frame.layout.as_ref().unwrap(),
                            "bar"
                        )
                        .is_some(),
                        "NativeReplayChromeMounted"
                    );
                    assert!(
                        frame
                            .visible
                            .iter()
                            .all(|(_, kind, _)| *kind != ItemKind::Native),
                        "NativeNeverDraws"
                    );
                    if !native_only {
                        let first_tool = state
                            .transcript
                            .order
                            .iter()
                            .find(|id| state.transcript.items[id].kind == ItemKind::ToolCall)
                            .unwrap();
                        assert_eq!(
                            caption(frame.layout.as_ref().unwrap(), *first_tool).as_deref(),
                            Some("2 CALLS · ONE ROUND"),
                            "NativeReplayGroupUnchanged"
                        );
                    }
                    (
                        frame.visible.clone(),
                        (frame.scroll, frame.max_scroll),
                        layout_text(frame.layout.as_ref().unwrap()),
                        format!(
                            "{:?}{:?}{:?}",
                            frame.list.primitives, frame.rims, frame.over.primitives
                        ),
                    )
                })
                .await;
                frames.push(frame);
            }
            let [before, after] = frames.as_slice() else {
                unreachable!()
            };
            assert_eq!(
                before.0, after.0,
                "NativeReplayRowsUnchanged: N={count}, native_only={native_only}, zoom={percent}"
            );
            assert_eq!(before.1, after.1, "NativeReplayScrollUnchanged");
            assert_eq!(before.2, after.2, "NativeReplayTextUnchanged");
            assert_eq!(before.3, after.3, "NativeReplayPaintUnchanged");
        }
    }
}

#[tokio::test]
async fn capture_expand_all_contains_every_tool_and_reasoning_body_including_folded_groups() {
    use bboxy450_harness::frontends::gui::{
        options::Options,
        rich::{Face, Toggle},
        smoke::capture_frame,
        view::find,
    };
    for fixture in [
        "tests/fixtures/gui/rich-all-items.jsonl",
        "tests/fixtures/parity/acp-opencode.jsonl",
    ] {
        let transcript = Arc::new(read_replay(fixture.into()).await.unwrap());
        bboxy450_harness::runtime::blocking(move || {
            let mut state = FrameState {
                transcript,
                ready: true,
                replay: true,
                ..Default::default()
            };
            // Tall enough that viewport clipping cannot remove any fixture row.
            let mut options =
                Options::parse(["--replay", fixture, "--capture", "900x32000"].map(String::from))
                    .unwrap();
            let mut default_view = HarnessView::default();
            let default = capture_frame(&mut default_view, &mut state, &options).unwrap();
            assert!(
                state.faces.is_empty(),
                "default capture creates no face overrides"
            );
            let mut expanded_view = HarnessView::default();
            options.expand_all = true;
            let expanded = capture_frame(&mut expanded_view, &mut state, &options).unwrap();
            assert_eq!(expanded.max_scroll, 0., "fixture must fit in this capture");
            let before = default.layout.as_ref().unwrap();
            let after = expanded.layout.as_ref().unwrap();
            let mut tools = 0;
            let mut thoughts = 0;
            let mut default_closed_tools = 0;
            let mut default_closed_thoughts = 0;
            for row in state.transcript.items.values() {
                let (key, toggle) = match row.kind {
                    ItemKind::ToolCall => {
                        tools += 1;
                        (format!("tool/{}/body", row.id), Toggle::Output)
                    }
                    ItemKind::Reasoning => {
                        thoughts += 1;
                        (format!("reasoning/{}/body", row.id), Toggle::Reasoning)
                    }
                    _ => continue,
                };
                assert!(
                    find(after, &key).is_some(),
                    "{fixture}: missing expanded {key}"
                );
                if row.kind == ItemKind::ToolCall {
                    let motion_body = find(after, &format!("toolbody/{}", row.id))
                        .expect("expanded tool retains its fold-motion hook");
                    assert!(find(motion_body, &key).is_some());
                }
                if find(before, &key).is_none() {
                    if row.kind == ItemKind::Reasoning {
                        default_closed_thoughts += 1;
                    } else {
                        default_closed_tools += 1;
                        if find(before, &format!("tool/{}", row.id)).is_none() {
                            // Its repeated-reply group was folded, so it had no default card.
                            continue;
                        }
                    }
                    let mut clicked = Face::default();
                    clicked.apply(toggle);
                    assert_eq!(state.faces.get(&row.id), Some(&clicked));
                } else if row.kind == ItemKind::ToolCall {
                    assert!(
                        !state.faces.get(&row.id).is_some_and(|f| f.output_closed),
                        "an automatically open tool must stay open"
                    );
                }
            }
            assert!(tools > 0 && thoughts > 0);
            assert!(default_closed_tools > 0 && default_closed_thoughts > 0);
            if fixture.ends_with("acp-opencode.jsonl") {
                assert!(
                    state.transcript.items.values().any(|row| {
                        row.kind == ItemKind::AgentMessage
                            && find(after, &format!("loop/{}", row.id)).is_some()
                            && state.faces.get(&row.id).is_some_and(|f| f.output_closed)
                    }),
                    "the repeated-reply group itself must be open"
                );
            }
            // The opt-in capture must not affect the next ordinary capture.
            let mut restored_view = HarnessView::default();
            options.expand_all = false;
            let restored = capture_frame(&mut restored_view, &mut state, &options).unwrap();
            assert!(state.faces.is_empty());
            assert_eq!(
                bboxy450_harness::frontends::gui::view::layout_text(before),
                bboxy450_harness::frontends::gui::view::layout_text(
                    restored.layout.as_ref().unwrap()
                )
            );
        })
        .await;
    }
}

#[tokio::test]
async fn completion_surface_occludes_acp_error_notice_and_red_turn_receipt() {
    use bboxy450_harness::frontends::gui::model::{TurnState, ViewItem};
    let item = ItemId::new();
    let turn = TurnId::new();
    let mut transcript = Transcript {
        profile: Some(("fake-acp".into(), "acp:fixture-agent".into())),
        ..Default::default()
    };
    transcript.order.push_back(item);
    transcript.items.insert(
        item,
        ViewItem {
            id: item,
            turn: Some(turn),
            parent: None,
            started: Seq(1),
            completed: Some(Seq(2)),
            kind: ItemKind::Notice,
            body: Item::Notice {
                level: Level::Error,
                text: "ACP agent failed to start.\nNo such file or directory (os error 2).".into(),
            },
            status: Some(ItemStatus::Completed),
            terminal_tail: String::new(),
            started_ms: 0,
            completed_ms: Some(1),
        },
    );
    transcript.turns.insert(
        turn,
        TurnState::Ended {
            reason: StopReason::Error {
                class: ErrorClass::Config,
                message: "agent exited".into(),
            },
            usage: None,
        },
    );
    let mut populated = FrameState {
        transcript: Arc::new(transcript),
        ready: true,
        replay: true,
        ..Default::default()
    };
    populated.prepared = Some(Arc::new(
        prepare(&mut Preparer::default(), populated.transcript.clone(), 900.).await,
    ));
    let mut empty = (*populated.transcript).clone();
    empty.order.clear();
    empty.items.clear();
    let mut empty = FrameState {
        transcript: Arc::new(empty),
        prepared: None,
        ..populated.clone()
    };
    empty.prepared = Some(Arc::new(
        prepare(&mut Preparer::default(), empty.transcript.clone(), 900.).await,
    ));
    let context = bbt_render::device::DeviceContext::new().await.unwrap();
    bboxy450_harness::runtime::blocking(move || {
        let mut view = HarnessView::default();
        view.render(&populated, [900., 300.]).unwrap();
        view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
        assert!(view.completion_open());
        let mut renderer = bbt_render::chrome::ChromeRenderer::with_context(context,
            bboxy450_harness::frontends::gui::assets::icons()).unwrap();
        let frame = view.render(&populated, [900., 300.]).unwrap();
        // the menu is painted over the face (Frame::over); the window draws the face, then
        // the rims, then the overlay onto one target, which one flat list reproduces here
        let whole = |frame: &bboxy450_harness::frontends::gui::view::Frame| {
            let mut list = frame.list.clone();
            list.primitives.extend(frame.over.primitives.iter().cloned());
            list
        };
        let popup = frame.over.primitives.iter().find_map(|p| match p {
            // The "/" menu card (harness-spec.md flat fallback): an opaque
            // surface_2 fill, inside its 1px seat ring at left 12.
            bbt_ui_model::draw::Prim::Rect { rect, fill, .. }
                if rect.x == 13. && rect.w > 300. && fill.0 == bbt_ui_model::draw::Color::hex("#1B1F3F").0 => Some(*rect),
            _ => None,
        });
        let without_surface = |list: &bbt_ui_model::draw::DrawList| {
            let mut list = list.clone();
            list.primitives.retain(|p| !matches!(p,
                bbt_ui_model::draw::Prim::Rect { rect, .. } if Some(*rect) == popup));
            list
        };
        let full = renderer.render_to_rgba(&whole(frame), 900, 300, 1.).unwrap();
        let broken_full = renderer.render_to_rgba(&without_surface(&whole(frame)), 900, 300, 1.).unwrap();
        let frame = view.render(&empty, [900., 300.]).unwrap();
        let blank = renderer.render_to_rgba(&whole(frame), 900, 300, 1.).unwrap();
        let broken_blank = renderer.render_to_rgba(&without_surface(&whole(frame)), 900, 300, 1.).unwrap();
        assert!(view.cancel_prompt(&Bridge::new(Arc::new(Wake))));
        let frame = view.render(&populated, [900., 300.]).unwrap();
        let transcript = renderer.render_to_rgba(&whole(frame), 900, 300, 1.).unwrap();
        if let Some(out) = std::env::var_os("HARNESS_RENDER_EVIDENCE") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            for (name, bytes) in [("completion-populated.png", &full), ("completion-empty.png", &blank), ("completion-broken-populated.png", &broken_full), ("completion-broken-empty.png", &broken_blank), ("transcript.png", &transcript)] {
                let mut encoder = png::Encoder::new(std::fs::File::create(out.join(name)).unwrap(), 900, 300);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.write_header().unwrap().write_image_data(bytes).unwrap();
            }
        }
        let popup = popup.expect("completion overlay must have an opaque surface");
        let mut covered_ink = 0;
        let mut leaked_ink = 0;
        let mut red_ink = 0;
        // The rounded outer edge deliberately exposes the backdrop; every
        // interior pixel must be independent of transcript notices/receipts.
        for y in (popup.y + 12.) as usize..(popup.bottom() - 12.) as usize {
            for x in (popup.x + 12.) as usize..(popup.right() - 12.) as usize {
                let p = (y * 900 + x) * 4;
                assert_eq!(&full[p..p + 4], &blank[p..p + 4], "transcript overdraw at {x},{y}");
                covered_ink += usize::from(full[p..p + 4] != transcript[p..p + 4]);
                leaked_ink += usize::from(broken_full[p..p + 4] != broken_blank[p..p + 4]);
                red_ink += usize::from(transcript[p] > transcript[p + 1].saturating_add(40));
            }
        }
        assert!(covered_ink > 1000, "witness must actually cover transcript ink");
        assert!(leaked_ink > 100, "transparent-popover control must leak transcript ink");
        assert!(red_ink > 30, "ACP error and receipt must paint red beneath the popover");
        println!("completion occlusion: {popup:?}; {covered_ink} changed pixels; {red_ink} underlying red pixels; zero transcript-dependent interior pixels; transparent control leaks {leaked_ink} pixels");
    }).await;
}
#[tokio::test]
async fn actual_layout_sticks_releases_and_renders_approval_actions() {
    let mut model = read_replay("tests/fixtures/protocol/all-items.jsonl".into())
        .await
        .unwrap();
    let id = model
        .items
        .values()
        .find(|r| r.kind == ItemKind::Approval)
        .unwrap()
        .id;
    let row = model.items.get_mut(&id).unwrap();
    row.status = None;
    row.completed = None;
    if let Item::Approval {
        decision, options, ..
    } = &mut row.body
    {
        *decision = None;
        *options = vec![
            ApprovalDecisionKind::Allow,
            ApprovalDecisionKind::AllowAlways,
            ApprovalDecisionKind::Deny,
        ];
    }
    let mut state = FrameState {
        transcript: Arc::new(model),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut preparer = Preparer::default();
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, state.transcript.clone(), 1120.).await,
    ));
    let mut view = HarnessView::default();
    let frame = view.render(&state, [1120., 860.]).unwrap();
    assert!(frame.max_scroll > 0.);
    assert_eq!(frame.scroll, frame.max_scroll);
    let old_max = frame.max_scroll;
    assert!(view.scroll.wheel(old_max, old_max, -100.));
    view.invalidate();
    let frame = view.render(&state, [1120., 860.]).unwrap();
    assert_eq!(frame.scroll, old_max - 100.);
    let mut grown = (*state.transcript).clone();
    let mut row = grown.items[grown.order.back().unwrap()].clone();
    row.id = ItemId::new();
    row.started = Seq(grown.cursor.0 + 1);
    row.completed = Some(Seq(grown.cursor.0 + 2));
    row.kind = ItemKind::Notice;
    row.body = Item::Notice {
        level: Level::Info,
        text: "more content\n".repeat(100),
    };
    grown.order.push_back(row.id);
    grown.items.insert(row.id, row);
    let mut grown = FrameState {
        transcript: Arc::new(grown),
        ..state.clone()
    };
    grown.prepared = Some(Arc::new(
        prepare(&mut preparer, grown.transcript.clone(), 1120.).await,
    ));
    assert_eq!(
        grown.prepared.as_ref().unwrap().layout_builds,
        state.prepared.as_ref().unwrap().layout_builds + 3,
        "only the open approval, the appended item, and the row handing it the turn receipt rebuild"
    );
    let frame = view.render(&grown, [1120., 860.]).unwrap();
    assert_eq!(frame.scroll, old_max - 100.);
    view.scroll.follow_latest();
    view.invalidate();
    let frame = view.render(&grown, [1120., 860.]).unwrap();
    assert_eq!(frame.scroll, frame.max_scroll);
    let frame = view.render(&state, [1120., 7000.]).unwrap();
    assert_eq!(
        frame
            .actions
            .iter()
            .filter(|(_, action)| matches!(action, Action::Approval(_, _)))
            .count(),
        3
    );
    // every item but the usage item, which draws no row (harness-spec.md "Refinement pass 2")
    assert_eq!(
        frame.visible.len(),
        state
            .transcript
            .items
            .values()
            .filter(|r| !matches!(r.kind, ItemKind::Usage | ItemKind::Native))
            .count()
    );
}
#[tokio::test]
async fn composer_send_and_control_buttons_reach_real_gateway() {
    let mut provider =
        bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    provider.delay = Duration::from_millis(20);
    let session = support::with_provider(Arc::new(provider), true).await;
    let mut observer = support::subscribe(&session.gateway, false);
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.attach(session.gateway.clone());
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if bridge.state.ready {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    let mut view = HarnessView::default();
    view.composer
        .handle(bbt_ui::modules::composer::Command::Edit(
            bbt_ui::text_edit::Command::Insert("from composer".into()),
        ));
    view.activate(Action::Send, &bridge);
    assert!(view.composer.view.text.is_empty());
    loop {
        let e = observer.recv().await.unwrap();
        if matches!(e.event, Event::TurnStarted { .. }) {
            break;
        }
    }
    view.activate(Action::Interrupt, &bridge);
    let events = support::end(&mut observer).await;
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    )));
    view.activate(Action::Stop, &bridge);
    loop {
        let e = observer.recv().await.unwrap();
        if matches!(e.event, Event::SessionClosed) {
            break;
        }
    }
    session.task.await.unwrap().unwrap();
    let job = bridge.detach().unwrap();
    job.await.unwrap();
    let mut offline = Bridge::new(Arc::new(Wake));
    offline.state.transcript = Arc::new(Transcript::default());
    view.composer
        .handle(bbt_ui::modules::composer::Command::Edit(
            bbt_ui::text_edit::Command::Insert("retain this draft".into()),
        ));
    view.activate(Action::Send, &offline);
    assert_eq!(view.composer.view.text, "retain this draft");
    assert!(view.notice.is_some());
}

#[tokio::test]
async fn replay_resize_publishes_current_width_and_invalidates_frame_cache() {
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay("tests/fixtures/protocol/all-items.jsonl".into());
    async fn ready(bridge: &mut Bridge, width: f32) {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                bridge.drain(Duration::from_micros(300));
                if bridge
                    .state
                    .prepared
                    .as_ref()
                    .is_some_and(|p| p.width == width)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
    }
    ready(&mut bridge, 1120.).await;
    let mut view = HarnessView::default();
    view.render(&bridge.state, [1120., 860.]).unwrap();
    bridge.set_width(800.);
    view.render(&bridge.state, [800., 860.]).unwrap();
    ready(&mut bridge, 800.).await;
    let frame = view.render(&bridge.state, [800., 860.]).unwrap();
    assert!(!frame.visible.is_empty());
    assert!(frame.visible.iter().all(|(_, _, r)| r.right() <= 800.));
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn cards_show_readable_content_instead_of_serialized_payloads() {
    let model = read_replay("tests/fixtures/protocol/all-items.jsonl".into())
        .await
        .unwrap();
    let mut state = FrameState {
        transcript: Arc::new(model),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut preparer = Preparer::default();
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, state.transcript.clone(), 1120.).await,
    ));
    let mut view = HarnessView::default();
    let frame = view.render(&state, [1120., 7000.]).unwrap();
    let text = bboxy450_harness::frontends::gui::view::layout_text(frame.layout.as_ref().unwrap());
    assert!(
        !text.contains("\"kind\":"),
        "raw JSON is not a readable card: {text}"
    );
    // harness-spec.md "Tight transcript": a successful command says nothing about its exit
    assert!(
        !text.contains("exit 0"),
        "a zero exit must not be drawn: {text}"
    );
    assert!(!text.contains("Some("));
}

fn hand_item(
    id: ItemId,
    kind: ItemKind,
    body: Item,
    terminal_tail: &str,
) -> bboxy450_harness::frontends::gui::model::ViewItem {
    bboxy450_harness::frontends::gui::model::ViewItem {
        id,
        turn: None,
        parent: None,
        started: Seq(1),
        completed: Some(Seq(2)),
        kind,
        body,
        status: Some(ItemStatus::Completed),
        terminal_tail: terminal_tail.into(),
        started_ms: 0,
        completed_ms: Some(1),
    }
}
async fn hand_state(rows: Vec<bboxy450_harness::frontends::gui::model::ViewItem>) -> FrameState {
    let mut t = Transcript::default();
    for row in rows {
        t.order.push_back(row.id);
        t.items.insert(row.id, row);
    }
    let mut state = FrameState {
        transcript: Arc::new(t),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut preparer = Preparer::default();
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, state.transcript.clone(), 900.).await,
    ));
    state
}

/// harness-spec.md "Refinement pass 2": a usage item draws nothing in the feed and adds no
/// space; the header meter sums the items and shows once input tokens arrive, never for a
/// 0/0 total (the operator's screenshot showed "0 in · 0 out" rows).
#[tokio::test]
async fn usage_items_feed_the_header_meter_and_draw_no_row() {
    use bboxy450_harness::frontends::gui::view::{find, layout_text};
    let mut heights = vec![];
    for (tokens_in, tokens_out, meter) in [(0, 0, false), (1500, 300, true)] {
        let reply = ItemId::new();
        let usage = ItemId::new();
        let state = hand_state(vec![
            hand_item(
                reply,
                ItemKind::AgentMessage,
                Item::AgentMessage {
                    markdown: "A reply.".into(),
                },
                "",
            ),
            hand_item(
                usage,
                ItemKind::Usage,
                Item::Usage {
                    usage: Usage {
                        input_tokens: tokens_in,
                        output_tokens: tokens_out,
                        ..Default::default()
                    },
                },
                "",
            ),
        ])
        .await;
        let mut view = HarnessView::default();
        let frame = view.render(&state, [900., 700.]).unwrap();
        let layout = frame.layout.as_ref().unwrap();
        assert!(
            frame.item_text(usage).unwrap_or_default().trim().is_empty(),
            "the usage item drew a row"
        );
        assert!(
            !frame.visible.iter().any(|(id, ..)| *id == usage),
            "a usage item is never a visible row"
        );
        let feed = layout_text(find(layout, "transcript").unwrap());
        assert!(
            !feed.contains(" in · ") && !feed.contains("token"),
            "token figures reached the feed: {feed}"
        );
        // A hidden item needs no display slot; any retained slot must remain empty.
        if let Some(slot) = find(layout, &format!("cached/{usage}")) {
            assert_eq!(
                slot.rect.h, 0.,
                "the usage item added space: {:?}",
                slot.rect
            );
            assert!(layout_text(slot).is_empty(), "the usage slot drew text");
        }
        // the reply carries the feed's top and bottom padding either way
        heights.push(find(layout, &format!("item/{reply}")).unwrap().rect.h);
        assert_eq!(
            find(layout, "meter").is_some(),
            meter,
            "the meter shows only once input tokens arrive"
        );
        if meter {
            let bar = layout_text(find(layout, "toolbar").unwrap());
            assert!(bar.contains("1.8k"), "the meter shows in + out: {bar}");
        }
    }
    assert!((heights[0] - heights[1]).abs() < 0.5);
}

/// harness-spec.md "Tight transcript": the body never repeats the command and a zero exit
/// is not drawn; a non-zero exit is the header chip; the output sits on 7 px top and 10 px
/// bottom padding with nothing under the last line.
#[tokio::test]
async fn terminal_body_has_no_command_or_exit_line_and_keeps_its_padding() {
    use bboxy450_harness::frontends::gui::view::{find, layout_text};
    for (exit, chip) in [(Some(0), None), (Some(3), Some("exit 3"))] {
        let id = ItemId::new();
        let state = hand_state(vec![hand_item(
            id,
            ItemKind::Terminal,
            Item::Terminal {
                terminal: "t1".into(),
                command: "printf ok".into(),
                exit,
                signal: None,
                output: None,
                live: None,
            },
            "ok\nagain",
        )])
        .await;
        let mut view = HarnessView::default();
        let frame = view.render(&state, [900., 700.]).unwrap();
        let layout = frame.layout.as_ref().unwrap();
        let term = find(layout, &format!("term/{id}")).expect("the terminal block");
        let body = layout_text(term);
        assert!(body.contains("ok") && body.contains("again"), "{body}");
        assert!(
            !body.contains("printf") && !body.contains("exit"),
            "the body repeated the command or the exit: {body}"
        );
        let head = layout_text(find(layout, &format!("toolhead/{id}")).unwrap());
        assert!(
            head.contains("printf ok"),
            "the head names the command: {head}"
        );
        match chip {
            Some(chip) => assert!(head.contains(chip), "a non-zero exit is the chip: {head}"),
            None => assert!(!head.contains("exit"), "a zero exit is not drawn: {head}"),
        }
        let out = find(layout, &format!("term/{id}/output")).expect("the output block");
        let lines = &out.children.first().expect("the padded box").children;
        let (first, last) = (
            lines.first().expect("output lines").rect,
            lines.last().expect("output lines").rect,
        );
        assert!(
            (first.y - out.rect.y - 7.).abs() < 0.5,
            "7 px above the first line: {} under {}",
            first.y,
            out.rect.y
        );
        assert!(
            (out.rect.bottom() - last.bottom() - 10.).abs() < 0.5,
            "10 px under the last line: {} above {}",
            last.bottom(),
            out.rect.bottom()
        );
    }
}

/// harness-spec.md "Refinement pass 2", message text rhythm: headings 15/700 with 8 above and
/// 6 below, paragraphs 0 above and 6 below, lists 2 above and 8 below on a 20 px indent with
/// 2 px between items. Adjacent margins collapse to the larger one, as on the web.
#[tokio::test]
async fn message_text_rhythm_matches_the_web_reference() {
    use bboxy450_harness::frontends::gui::view::find;
    use bbt_ui::components::{Layout, LayoutKind};
    fn text_node<'a>(n: &'a Layout, want: &str) -> Option<&'a Layout> {
        if let LayoutKind::Text { runs, .. } = &n.kind
            && runs
                .iter()
                .map(|r| r.text.as_str())
                .collect::<String>()
                .trim()
                == want
        {
            return Some(n);
        }
        n.children.iter().find_map(|c| text_node(c, want))
    }
    let id = ItemId::new();
    let state = hand_state(vec![hand_item(
        id,
        ItemKind::AgentMessage,
        Item::AgentMessage {
            markdown: "## Title\n\nOne.\n\n- alpha\n- beta\n\nTwo.".into(),
        },
        "",
    )])
    .await;
    let mut view = HarnessView::default();
    let frame = view.render(&state, [900., 700.]).unwrap();
    let layout = frame.layout.as_ref().unwrap();
    let block = |i: usize| {
        find(layout, &format!("mdblock/{i}"))
            .expect("markdown block")
            .rect
    };
    let (heading, one, list, two) = (block(0), block(1), block(2), block(3));
    let gap = |a: bbt_ui_model::layout::Rect, b: bbt_ui_model::layout::Rect| b.y - a.bottom();
    assert!(
        (gap(heading, one) - 6.).abs() < 0.01,
        "heading 6 below: {}",
        gap(heading, one)
    );
    assert!(
        (gap(one, list) - 6.).abs() < 0.01,
        "paragraph 6 below, list 2 above: {}",
        gap(one, list)
    );
    assert!(
        (gap(list, two) - 8.).abs() < 0.01,
        "list 8 below: {}",
        gap(list, two)
    );
    let title = text_node(layout, "Title").expect("heading text");
    let LayoutKind::Text { runs, .. } = &title.kind else {
        unreachable!()
    };
    assert!(
        runs.iter()
            .all(|r| (r.style.size - 15.).abs() < 0.01 && r.style.weight >= 700),
        "heading 15/700"
    );
    let (alpha, beta, one_text) = (
        text_node(layout, "alpha").expect("first item"),
        text_node(layout, "beta").expect("second item"),
        text_node(layout, "One.").expect("paragraph"),
    );
    assert!(
        (alpha.rect.x - one_text.rect.x - 20.).abs() < 0.01,
        "list indent 20: {} vs {}",
        alpha.rect.x,
        one_text.rect.x
    );
    assert!(
        (beta.rect.y - alpha.rect.bottom() - 2.).abs() < 0.01,
        "2 px between items: {}",
        beta.rect.y - alpha.rect.bottom()
    );
}

#[tokio::test]
async fn live_terminal_heads_snapshots_and_completed_blob_reach_the_view() {
    use bboxy450_harness::gateway::Gateway;
    let command = "printf stream-head; sleep 0.3; printf stream-tail";
    let first = format!("data: {}\n\ndata: [DONE]\n\n", serde_json::json!({"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"gui-terminal","type":"function","function":{"name":"bash","arguments":serde_json::json!({"command":command}).to_string()}}]},"finish_reason":"tool_calls"}]})).into_bytes();
    let session = support::session(vec![first, support::sse("text")], true).await;
    let blobs = bboxy450_harness::store::Blobs {
        dir: session
            .temp
            .path()
            .join("sessions")
            .join(session.gateway.hello().meta.session.to_string())
            .join("blobs"),
        redactor: Default::default(),
    };
    let mut observer = support::subscribe(&session.gateway, true);
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.attach_with_blobs(session.gateway.clone(), blobs);
    session
        .gateway
        .send(support::prompt(&session.gateway, "run command"))
        .unwrap();
    let mut projection = Transcript::default();
    let mut saw_head = false;
    let mut saw_live = false;
    let mut saw_tool_head = false;
    let mut terminal_id = None;
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(e) = observer.recv().await {
            projection.apply(&e);
            match e.event {
                Event::ItemStarted { item, kind: ItemKind::Terminal, head, .. } => {
                    terminal_id = Some(item);
                    if let Some(ItemHead::Terminal { terminal, command: sent_command }) = head {
                        saw_head = !terminal.is_empty() && sent_command.contains(command)
                            && matches!(&projection.items[&item].body, Item::Terminal { terminal: t, command: c, .. } if t == &terminal && c == &sent_command);
                    }
                }
                Event::ItemStarted { item, kind: ItemKind::ToolCall, head: Some(ItemHead::ToolCall { call_id, name, .. }), .. } => {
                    saw_tool_head = call_id == "gui-terminal" && name == "bash"
                        && matches!(&projection.items[&item].body, Item::ToolCall { call_id: c, name: n, .. } if c == &call_id && n == &name);
                }
                Event::ItemSnapshot { item, partial: Item::Terminal { .. } } => {
                    let live = serde_json::to_value(&e.event).unwrap();
                    let expected = live.pointer("/partial/live").and_then(serde_json::Value::as_str).unwrap_or("");
                    if !expected.is_empty() { assert_eq!(projection.items[&item].terminal_tail, expected); saw_live = true; }
                }
                Event::TurnEnded { .. } => break,
                _ => {}
            }
        }
    }).await.unwrap();
    let terminal_id = terminal_id.expect("real bash created a terminal");
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if bridge.state.prepared.is_some()
                && bridge
                    .state
                    .transcript
                    .items
                    .get(&terminal_id)
                    .is_some_and(|r| r.completed.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
    let text = frame.item_text(terminal_id).unwrap();
    // harness-spec.md "Tight transcript": the body never repeats the command the head names
    assert!(
        !text.contains(command),
        "the terminal body repeated the command: {text}"
    );
    // the exit status is the tool card header's chip, never a body line (harness-spec.md
    // "Refinement pass 2"): the terminal folds into its tool call, whose header shows it
    assert!(
        !text.contains("exit 0"),
        "the terminal body appended an exit line: {text}"
    );
    let head = bridge.state.transcript.items[&terminal_id]
        .parent
        .map_or(format!("toolhead/{terminal_id}"), |p| {
            format!("toolhead/{p}")
        });
    let head = bboxy450_harness::frontends::gui::view::find(frame.layout.as_ref().unwrap(), &head)
        .expect("the tool card header");
    // harness-spec.md "Tight transcript": a zero exit is not drawn anywhere; the check mark
    // already says it
    assert!(
        !bboxy450_harness::frontends::gui::view::layout_text(head).contains("exit"),
        "the header drew a zero exit"
    );
    assert!(
        text.contains("stream-headstream-tail"),
        "completed blob must supply output, not just the command: {text}"
    );
    bridge.detach().unwrap().await.unwrap();
    support::close(session).await;
    println!(
        "gui-live: completed blob content rendered; typed terminal head={saw_head}, tool head={saw_tool_head}, live snapshot={saw_live}"
    );
    assert!(
        saw_head,
        "live Envelope head must name the command before completion"
    );
    assert!(
        saw_tool_head,
        "live gateway must project durable tool identity before deltas"
    );
    assert!(
        saw_live,
        "live gateway must deliver coalesced v0.2 output snapshots"
    );
}

#[tokio::test]
async fn structural_layout_smoke_checks_restored_geometry() {
    assert_eq!(
        bboxy450_harness::frontends::gui::view::CONTENTLESS_SHAPE_PATTERNS,
        ["*/rail", "composer/text/caret", "composer/address/*/dot"],
        "ContentlessShapePatternsStayExplicit"
    );
    let mut model = read_replay("tests/fixtures/gui/rich-all-items.jsonl".into())
        .await
        .unwrap();
    for id in model.order.clone() {
        let row = model.items.get_mut(&id).unwrap();
        if let Item::Approval {
            decision, options, ..
        } = &mut row.body
        {
            *decision = None;
            *options = vec![
                ApprovalDecisionKind::Allow,
                ApprovalDecisionKind::AllowAlways,
                ApprovalDecisionKind::Deny,
            ];
            row.status = None;
            row.completed = None;
        }
    }
    let mut state = FrameState {
        transcript: Arc::new(model),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut preparer = Preparer::default();
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, state.transcript.clone(), 1120.).await,
    ));
    let mut view = HarnessView::default();
    let frame = view.render(&state, [1120., 7000.]).unwrap();
    let checked = bboxy450_harness::frontends::gui::layout_checks::check(
        state.prepared.as_ref().unwrap(),
        frame.layout.as_ref().unwrap(),
    )
    .unwrap();
    println!("prepared structural geometry: {checked:?}");
    // every drawn item is addressable; folded ones (approval, terminal, diff) live inside a
    // tool card, and a usage item draws no row at all (harness-spec.md "Refinement pass 2")
    let layout = frame.layout.as_ref().unwrap();
    assert!(state.transcript.order.iter().all(|id| {
        matches!(
            state.transcript.items[id].kind,
            ItemKind::Usage | ItemKind::Native
        ) || bboxy450_harness::frontends::gui::view::find(layout, &format!("item/{id}")).is_some()
    }));
    assert!(checked.cards > 0 && checked.cards <= state.transcript.items.len());
    assert_eq!(checked.disclosures, checked.accordion_headers);
    // header: no controls (interrupt/stop live in the composer); the phase and the
    // profile chip sit centred in the toolbar row
    let find = |k: &str| bboxy450_harness::frontends::gui::view::find(layout, k);
    let toolbar = find("toolbar").unwrap();
    for key in ["jump", "interrupt", "stop"] {
        let inside = find(key).is_some_and(|n| {
            n.rect.y >= toolbar.rect.y && n.rect.y + n.rect.h <= toolbar.rect.y + toolbar.rect.h
        });
        assert!(!inside, "HeaderHasNoControls: {key}");
    }
    // the chip only exists once a profile resolved
    let chip = state.transcript.profile.is_some();
    for key in ["phase", "profile"]
        .into_iter()
        .filter(|k| chip || *k == "phase")
    {
        let node = find(key).unwrap();
        assert!(
            ((toolbar.rect.y + toolbar.rect.h / 2.) - (node.rect.y + node.rect.h / 2.)).abs() < 0.5,
            "ToolbarLabelCentered: {key} {:?} in {:?}",
            node.rect,
            toolbar.rect
        );
    }
    // rich face: one disclosure per tool card and per reasoning row
    let disclosures = state
        .transcript
        .items
        .values()
        .filter(|r| matches!(r.kind, ItemKind::ToolCall | ItemKind::Reasoning))
        .count();
    assert!(
        checked.markdown_blocks >= 4
            && checked.accordion_headers == disclosures
            && checked.inline_rows >= 2,
        "{checked:?}"
    );
}

/// The composer is the web's one box (harness-spec.md Composer table): ring only while focused,
/// a phase hint, the 26 px round send that becomes a stop square while a turn runs with an
/// empty draft, and the toolkit hit contract (send, body, caret) intact.
#[test]
fn composer_box_follows_phase_and_keeps_hit_contract() {
    use bboxy450_harness::frontends::gui::view::find;
    use bbt_ui::modules::composer::{Hit, Regions};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state.ready = true;
    let mut view = HarnessView::default();
    let mut render = |phase: Phase, focused: bool| {
        bridge.state.transcript = Arc::new(Transcript {
            phase,
            ..Default::default()
        });
        view.focused = focused;
        view.invalidate();
        let layout = view
            .render(&bridge.state, [900., 700.])
            .unwrap()
            .layout
            .clone()
            .unwrap();
        (layout, view.send_stops(&bridge.state))
    };
    let (idle, stops) = render(Phase::Idle, false);
    assert!(!stops, "idle send never interrupts");
    assert!(
        find(&idle, "composer/hint").is_none(),
        "idle hides the hint"
    );
    assert!(
        Regions::new(&idle).unwrap().caret.is_none(),
        "no caret without window focus"
    );
    let (live, stops) = render(Phase::Streaming, true);
    assert!(
        stops,
        "running with an empty draft, send is the stop square"
    );
    fn rim(l: &bbt_ui::components::Layout) -> Option<bbt_ui::GlowActivity> {
        fn style(n: &bbt_ui::components::Layout) -> Option<&bbt_ui::ResolvedStyle> {
            match &n.kind {
                bbt_ui::components::LayoutKind::Box { style, .. } => Some(style),
                _ => n.children.iter().find_map(style),
            }
        }
        style(find(l, "composer")?)?.rim.map(|r| r.activity)
    }
    assert_eq!(
        rim(&live),
        Some(bbt_ui::GlowActivity::Active),
        "the alive rim runs with the turn (no sweep line)"
    );
    assert!(find(&live, "composer/sweep").is_none());
    assert_eq!(rim(&idle), Some(bbt_ui::GlowActivity::Idle));
    let hint = find(&live, "composer/hint").expect("a running turn shows esc interrupt");
    let send = find(&live, "composer/send").unwrap().rect;
    let panel = find(&live, "composer").unwrap().rect;
    assert!(hint.rect.right() <= send.x, "hint sits left of send");
    assert_eq!((send.w, send.h), (26., 26.));
    assert!(panel.bottom() <= 700. - 11. && panel.w <= 860.);
    let regions = Regions::new(&live).unwrap();
    let centre = |r: bbt_ui_model::layout::Rect| [r.x + r.w / 2., r.y + r.h / 2.];
    assert_eq!(regions.hit(centre(send)), Some(Hit::Send));
    assert_eq!(regions.hit(centre(regions.body)), Some(Hit::Body));
    assert!(
        regions.caret.is_some(),
        "the focused draft paints its caret"
    );
    let (ask, _) = render(
        Phase::Approval {
            item: ItemId::new(),
        },
        false,
    );
    assert_eq!(
        rim(&ask),
        Some(bbt_ui::GlowActivity::Waiting),
        "ask is the amber rim, not the run look"
    );
    assert!(
        find(&ask, "composer/hint").is_some(),
        "ask shows the 1/2/3 hint"
    );
}

#[tokio::test]
async fn v1_additions_have_quiet_notices_and_no_native_rows() {
    let model = read_replay("tests/fixtures/ir-v1/v1.1-all-items.jsonl".into())
        .await
        .unwrap();
    let mut state = FrameState {
        transcript: Arc::new(model),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut preparer = Preparer::default();
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, state.transcript.clone(), 1120.).await,
    ));
    let mut view = HarnessView::default();
    let frame = view.render(&state, [1120., 14000.]).unwrap();
    let text = bboxy450_harness::frontends::gui::view::layout_text(frame.layout.as_ref().unwrap());
    for label in [
        "Ran /help",
        "Hook fixture-hook",
        "Developer instructions",
        "Audio:",
        "Read reference",
        "fixture-mcp",
    ] {
        assert!(text.contains(label), "missing {label}: {text}");
    }
    assert!(!text.contains("opaque fixture"));
    assert!(!text.contains("unmapped") && !text.contains("NATIVE_GAP"));
    for row in state.transcript.items.values() {
        if row.kind == ItemKind::Native {
            assert!(!frame.visible.iter().any(|(id, _, _)| *id == row.id));
            assert!(frame.item_text(row.id).is_none());
        }
    }
}

fn hand_run(id: ItemId, lines: usize) -> bboxy450_harness::frontends::gui::model::ViewItem {
    let out = (1..=lines)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    hand_item(
        id,
        ItemKind::ToolCall,
        Item::ToolCall {
            call_id: format!("call-{id}"),
            name: "bash".into(),
            tool_kind: ToolKind::Execute,
            input_raw: serde_json::json!({"command": "make test"}).to_string(),
            output: vec![OutputBlock::Text { text: out }],
            locations: vec![],
            spill: None,
            title: None,
            server: None,
            raw_output: None,
        },
        "",
    )
}

/// harness-spec.md "Tight transcript": the head carries no `auto` badge and no duration under
/// 1s and pads 6/10; a finished, successful command folds unless its output is 6 lines or
/// fewer, while a failed one stays open; the feed gap is 8.
#[tokio::test]
async fn tight_tool_heads_fold_rule_and_feed_gap() {
    use bboxy450_harness::frontends::gui::view::{find, layout_text};
    let (short, long, failed, slow) = (ItemId::new(), ItemId::new(), ItemId::new(), ItemId::new());
    let mut rows = vec![
        hand_run(short, 3),
        hand_run(long, 9),
        hand_run(failed, 9),
        hand_run(slow, 2),
    ];
    rows[2].status = Some(ItemStatus::Failed {
        reason: "boom".into(),
        code: None,
    });
    for r in &mut rows {
        r.completed_ms = Some(400);
        // one run per turn: consecutive tool cards in one round share a rail and a 6 px
        // member gap (harness.js mount, gaps-hij); this test measures the plain feed gap
        r.turn = Some(TurnId::new());
    }
    rows[3].completed_ms = Some(2500);
    let state = hand_state(rows).await;
    let mut view = HarnessView::default();
    let frame = view.render(&state, [900., 1600.]).unwrap();
    let layout = frame.layout.as_ref().unwrap();
    let text = |key: &str| layout_text(find(layout, key).unwrap_or_else(|| panic!("{key}")));
    for id in [short, long, failed] {
        let h = text(&format!("toolhead/{id}"));
        assert!(h.contains("make test"), "the head names the command: {h}");
        assert!(
            !h.contains("auto") && !h.contains("ms"),
            "no badge, no sub-second duration: {h}"
        );
    }
    assert!(
        text(&format!("toolhead/{slow}")).contains("2.5s"),
        "the duration shows from 1s"
    );
    // head padding 6: the kind tile sits 6 px under the head's top edge
    let head = find(layout, &format!("toolhead/{short}")).unwrap();
    let tile = head.children[0].children[0].rect;
    assert!(
        (tile.y - head.rect.y - 6.).abs() < 0.01,
        "head top padding: {}",
        tile.y - head.rect.y
    );
    // the fold rule
    assert!(
        find(layout, &format!("term/{short}")).is_some_and(|n| n.rect.h > 0.),
        "3 lines of output: open"
    );
    assert!(
        find(layout, &format!("term/{long}")).is_none(),
        "9 lines of output: folded"
    );
    assert!(
        find(layout, &format!("term/{failed}")).is_some_and(|n| n.rect.h > 0.),
        "failed: open"
    );
    // feed gap 8 between rows (4 + 4 around the row bodies)
    let a = find(layout, &format!("body/{short}")).unwrap().rect;
    let b = find(layout, &format!("body/{long}")).unwrap().rect;
    assert!(
        (b.y - a.bottom() - 8.).abs() < 0.01,
        "feed gap 8: {}",
        b.y - a.bottom()
    );
}

/// harness-spec.md "Tight transcript": the receipt is a hairline rule and one line: mark,
/// title, the duration from 1s (always when the turn did not end ok), tools and edits, never
/// rounds, and no card. A failure keeps its red rule, its reason and the actions.
#[tokio::test]
async fn tight_receipt_is_a_rule_and_one_line() {
    use bboxy450_harness::frontends::gui::{
        model::TurnState,
        view::{find, layout_text},
    };
    let cases: Vec<(StopReason, u64, Vec<&str>, Vec<&str>)> = vec![
        (
            StopReason::EndTurn,
            400,
            vec!["Done", "1 tool"],
            vec!["round", "ms", "Retry"],
        ),
        (
            StopReason::EndTurn,
            2500,
            vec!["Done", "2.5s", "1 tool"],
            vec!["round"],
        ),
        (
            StopReason::Error {
                class: ErrorClass::Transient,
                message: "socket closed".into(),
            },
            400,
            vec![
                "Connection dropped",
                "400ms",
                "socket closed",
                "Retry",
                "Switch profile",
            ],
            vec!["round"],
        ),
    ];
    for (reason, took, wanted, unwanted) in cases {
        let turn = TurnId::new();
        let (think, tool, reply) = (ItemId::new(), ItemId::new(), ItemId::new());
        let mut rows = vec![
            hand_item(
                think,
                ItemKind::Reasoning,
                Item::Reasoning {
                    summary: "Plan it.".into(),
                    raw: None,
                    provider_opaque: None,
                },
                "",
            ),
            hand_run(tool, 2),
            hand_item(
                reply,
                ItemKind::AgentMessage,
                Item::AgentMessage {
                    markdown: "Done it.".into(),
                },
                "",
            ),
        ];
        for r in &mut rows {
            r.turn = Some(turn);
            r.completed_ms = Some(took);
        }
        let mut t = Transcript::default();
        for row in rows {
            t.order.push_back(row.id);
            t.items.insert(row.id, row);
        }
        let ok = matches!(reason, StopReason::EndTurn);
        t.turns.insert(
            turn,
            TurnState::Ended {
                reason,
                usage: None,
            },
        );
        let mut state = FrameState {
            transcript: Arc::new(t),
            ready: true,
            replay: true,
            ..Default::default()
        };
        let mut preparer = Preparer::default();
        state.prepared = Some(Arc::new(
            prepare(&mut preparer, state.transcript.clone(), 900.).await,
        ));
        let mut view = HarnessView::default();
        let frame = view.render(&state, [900., 900.]).unwrap();
        let layout = frame.layout.as_ref().unwrap();
        let receipt = find(layout, &format!("turn/{}", turn.0)).expect("receipt");
        let text = layout_text(receipt);
        for w in &wanted {
            assert!(text.contains(w), "receipt lacks {w:?}: {text}");
        }
        for u in &unwanted {
            assert!(!text.contains(u), "receipt drew {u:?}: {text}");
        }
        // a hairline rule above, then the line: no card fill
        let column = &receipt.children[0];
        let rule = &column.children[0];
        assert!(
            (rule.rect.h - 1.).abs() < 0.01 && rule.rect.w > 100.,
            "a 1 px rule above: {:?}",
            rule.rect
        );
        let bbt_ui::components::LayoutKind::Box { style, .. } = &column.children[1].kind else {
            panic!("the receipt line box");
        };
        assert_eq!(style.fill[3], 0., "no card fill behind the receipt");
        let _ = ok;
    }
}

/// harness-spec.md "Tight transcript": a thought under 1s has no "Thought" label, only its
/// 12 px gist; from 1s it reads "Thought for N". Its chevron is hidden at rest.
#[tokio::test]
async fn short_thoughts_show_the_gist_alone() {
    use bboxy450_harness::frontends::gui::view::{find, layout_text};
    for (took, label) in [(400, None), (1500, Some("Thought for 1.5s"))] {
        let id = ItemId::new();
        let mut row = hand_item(
            id,
            ItemKind::Reasoning,
            Item::Reasoning {
                summary: "Compare the files. Then decide.".into(),
                raw: None,
                provider_opaque: None,
            },
            "",
        );
        row.completed_ms = Some(took);
        let state = hand_state(vec![row]).await;
        let mut view = HarnessView::default();
        let frame = view.render(&state, [900., 700.]).unwrap();
        let layout = frame.layout.as_ref().unwrap();
        let toggle = find(layout, &format!("reasoning/{id}/toggle")).expect("thought row");
        let text = layout_text(toggle);
        assert!(text.contains("Compare the files."), "the gist: {text}");
        match label {
            Some(l) => assert!(text.contains(l), "{text}"),
            None => assert!(
                !text.contains("Thought"),
                "a short thought has no label: {text}"
            ),
        }
        fn gist_size(n: &bbt_ui::components::Layout) -> Option<f32> {
            if let bbt_ui::components::LayoutKind::Text { runs, .. } = &n.kind
                && let Some(r) = runs.iter().find(|r| r.text.contains("Compare"))
            {
                return Some(r.style.size);
            }
            n.children.iter().find_map(gist_size)
        }
        assert_eq!(gist_size(toggle), Some(12.), "the gist is 12 px");
        assert!(
            find(layout, &format!("reasoning/{id}/toggle/chev"))
                .is_none_or(|n| n.children.is_empty()),
            "the chevron is hidden at rest"
        );
    }
}

// ---------- gaps-hij: parallel tool calls, the search head and the kind icons ----------
fn tool_item(name: &str, kind: &str, input: serde_json::Value, out: &str) -> Item {
    serde_json::from_value(serde_json::json!({
        "kind": "tool_call", "call_id": name, "name": name, "tool_kind": kind,
        "input_raw": input.to_string(),
        "output": [{"type": "text", "text": out}], "locations": [], "spill": null
    }))
    .unwrap()
}
fn face_row(
    turn: TurnId,
    seq: u64,
    body: Item,
) -> bboxy450_harness::frontends::gui::model::ViewItem {
    bboxy450_harness::frontends::gui::model::ViewItem {
        id: ItemId::new(),
        turn: Some(turn),
        parent: None,
        started: Seq(seq),
        completed: Some(Seq(seq + 1)),
        kind: body.kind(),
        body,
        status: Some(ItemStatus::Completed),
        terminal_tail: String::new(),
        started_ms: seq,
        completed_ms: Some(seq + 1),
    }
}
/// Lays the rows out as the face does (cwd /repo, 900 wide) and returns the frame layout.
async fn face_layout(
    rows: Vec<bboxy450_harness::frontends::gui::model::ViewItem>,
) -> bbt_ui::components::Layout {
    let mut transcript = Transcript {
        meta: Some(support::meta("/repo".into())),
        ..Default::default()
    };
    for row in rows {
        transcript.order.push_back(row.id);
        transcript.items.insert(row.id, row);
    }
    let mut state = FrameState {
        transcript: Arc::new(transcript),
        ready: true,
        replay: true,
        ..Default::default()
    };
    state.prepared = Some(Arc::new(
        prepare(&mut Preparer::default(), state.transcript.clone(), 900.).await,
    ));
    let mut view = HarnessView::default();
    view.render(&state, [900., 4000.])
        .unwrap()
        .layout
        .clone()
        .unwrap()
}
fn caption(layout: &bbt_ui::components::Layout, id: ItemId) -> Option<String> {
    bboxy450_harness::frontends::gui::view::find(layout, &format!("group/{id}/caption"))
        .map(bboxy450_harness::frontends::gui::view::layout_text)
}
fn card_x(layout: &bbt_ui::components::Layout, id: ItemId) -> f32 {
    bboxy450_harness::frontends::gui::view::find(layout, &format!("tool/{id}"))
        .unwrap()
        .rect
        .x
}

/// harness.js mount(): consecutive tool cards with no other block between them share a
/// rail and a "N CALLS · ONE ROUND" caption. One call is no group; a reasoning or message
/// block (a new round) and a turn boundary end the run; a usage item never mounts, so it
/// neither joins nor breaks one.
#[tokio::test]
async fn parallel_tool_calls_share_a_rail_and_caption() {
    let turn = TurnId::new();
    let read = |n: u64| {
        face_row(
            turn,
            n,
            tool_item(
                "read",
                "read",
                serde_json::json!({"path": "/repo/src/lib.rs"}),
                "1: fn main() {}",
            ),
        )
    };
    let thought = |n: u64| {
        face_row(
            turn,
            n,
            Item::Reasoning {
                summary: "Next round.".into(),
                raw: None,
                provider_opaque: None,
            },
        )
    };
    // one call: no caption, the card at the feed's column
    let one = vec![thought(1), read(3)];
    let (t, a) = (one[0].id, one[1].id);
    let layout = face_layout(one).await;
    assert_eq!(caption(&layout, a), None);
    assert_eq!(caption(&layout, t), None);
    let column = card_x(&layout, a);

    // two calls in one round: the caption on the first only, both cards inset 14px
    let two = vec![read(1), read(3)];
    let (a, b) = (two[0].id, two[1].id);
    let layout = face_layout(two).await;
    assert_eq!(caption(&layout, a).as_deref(), Some("2 CALLS · ONE ROUND"));
    assert_eq!(caption(&layout, b), None);
    assert!(
        (card_x(&layout, a) - column - 14.).abs() < 0.5,
        "{} vs {column}",
        card_x(&layout, a)
    );
    assert!((card_x(&layout, b) - column - 14.).abs() < 0.5);

    // three calls
    let three = vec![read(1), read(3), read(5)];
    let ids: Vec<ItemId> = three.iter().map(|r| r.id).collect();
    let layout = face_layout(three).await;
    assert_eq!(
        caption(&layout, ids[0]).as_deref(),
        Some("3 CALLS · ONE ROUND")
    );
    assert!(ids[1..].iter().all(|id| caption(&layout, *id).is_none()));

    // across rounds: a thought between two calls ends the run; two rounds of two make two groups
    let rounds = vec![
        read(1),
        thought(3),
        read(5),
        read(7),
        thought(9),
        read(11),
        read(13),
    ];
    let ids: Vec<ItemId> = rounds.iter().map(|r| r.id).collect();
    let layout = face_layout(rounds).await;
    assert_eq!(caption(&layout, ids[0]), None);
    assert!((card_x(&layout, ids[0]) - column).abs() < 0.5);
    assert_eq!(
        caption(&layout, ids[2]).as_deref(),
        Some("2 CALLS · ONE ROUND")
    );
    assert_eq!(caption(&layout, ids[3]), None);
    assert_eq!(
        caption(&layout, ids[5]).as_deref(),
        Some("2 CALLS · ONE ROUND")
    );
    assert_eq!(caption(&layout, ids[6]), None);

    // across turns: the receipt block sits between them on the web
    let mut other = read(3);
    other.turn = Some(TurnId::new());
    let turns = vec![read(1), other];
    let ids: Vec<ItemId> = turns.iter().map(|r| r.id).collect();
    let layout = face_layout(turns).await;
    assert!(ids.iter().all(|id| caption(&layout, *id).is_none()));

    // a usage item between two calls: the web never mounts it, so the two still share a rail
    let usage: Item = serde_json::from_value(
        serde_json::json!({"kind": "usage", "usage": {"input_tokens": 10, "output_tokens": 2}}),
    )
    .unwrap();
    let with_usage = vec![read(1), face_row(turn, 3, usage), read(5)];
    let ids: Vec<ItemId> = with_usage.iter().map(|r| r.id).collect();
    let layout = face_layout(with_usage).await;
    assert_eq!(
        caption(&layout, ids[0]).as_deref(),
        Some("2 CALLS · ONE ROUND")
    );
}

/// A Search card's head reads `Search <pattern> in <path>` (harness.js `where`), and every
/// match in a hit is a mark pill (`.hx-hits mark`).
#[tokio::test]
async fn search_head_says_where_and_hits_mark_each_match() {
    use bboxy450_harness::frontends::gui::view::{find, layout_text};
    let turn = TurnId::new();
    let grep = face_row(
        turn,
        1,
        tool_item(
            "grep",
            "search",
            serde_json::json!({"pattern": "evict", "path": "/repo/src"}),
            "/repo/src/lib.rs:20: self.evict();\n/repo/src/lib.rs:47: was evicted, then evict again\n/repo/README.md",
        ),
    );
    let id = grep.id;
    let layout = face_layout(vec![grep]).await;
    let head = layout_text(find(&layout, &format!("toolhead/{id}")).unwrap());
    let (verb, pattern, place) = (
        head.find("Search").unwrap(),
        head.find("evict").unwrap(),
        head.find("in src").unwrap(),
    );
    assert!(verb < pattern && pattern < place, "{head}");
    assert!(head.contains("3 hits"), "{head}");
    let mark =
        |row: usize, k: usize| find(&layout, &format!("hits/{id}/{row}/mark/{k}")).map(layout_text);
    assert_eq!(mark(0, 0).as_deref(), Some("evict"));
    assert_eq!(mark(0, 1), None);
    // the second hit matches twice: "evict" inside "evicted" and the bare word
    assert_eq!(mark(1, 0).as_deref(), Some("evict"));
    assert_eq!(mark(1, 1).as_deref(), Some("evict"));
    assert_eq!(mark(1, 2), None);
    // a line without `:N:` is a file alone: no text, no marks
    assert_eq!(mark(2, 0), None);
    let text = layout_text(find(&layout, &format!("item/{id}")).unwrap());
    assert!(
        text.contains("src/lib.rs:20") && text.contains("README.md"),
        "{text}"
    );
}

#[test]
fn hit_lines_split_as_the_web_regex_does() {
    use bboxy450_harness::frontends::face_core::{hit_line, hit_spans};
    assert_eq!(
        hit_line("src/a.rs:12: let x = 1;"),
        ("src/a.rs", Some("12"), " let x = 1;")
    );
    assert_eq!(hit_line("C:/x/y.rs:3:z"), ("C:/x/y.rs", Some("3"), "z"));
    assert_eq!(hit_line("just/a/file.rs"), ("just/a/file.rs", None, ""));
    assert_eq!(hit_line("a:b:9:c"), ("a:b", Some("9"), "c"));
    let re = regex::Regex::new("fo+").unwrap();
    let spans = |t: &str| hit_spans(t, Some(&re));
    let own = |v: Vec<(&str, bool)>| -> Vec<(String, bool)> {
        v.into_iter().map(|(s, b)| (s.to_owned(), b)).collect()
    };
    assert_eq!(
        spans("let x = foo(bar)"),
        own(vec![("let x = ", false), ("foo", true), ("(bar)", false)])
    );
    assert_eq!(
        spans("fo foo"),
        own(vec![("fo", true), (" ", false), ("foo", true)])
    );
    assert_eq!(spans("none here"), own(vec![("none here", false)]));
    assert_eq!(hit_spans("plain", None), own(vec![("plain", false)]));
    assert_eq!(spans(""), vec![]);
}

/// harness.js ICON and `.hx-ki` colours, one glyph per tool kind: fetch is the globe, other
/// the sparkle, and no kind falls through to the sparkle by accident.
#[test]
fn tool_kind_icons_and_colours_match_the_web_table() {
    use bboxy450_harness::frontends::{face_core::kind_hex, gui::face::icon_commands};
    use bbt_ui_model::draw::PathCommand as P;
    let kinds = [
        ("read", "#58D7FF"),
        ("search", "#B07BFF"),
        ("execute", "#2EE6A8"),
        ("edit", "#FFC53D"),
        ("delete", "#FF6B6B"),
        ("move", "#F2D16B"),
        ("fetch", "#4FD1FF"),
        ("other", "#FF4FA3"),
    ];
    let glyphs: Vec<Vec<P>> = kinds
        .iter()
        .map(|(k, hex)| {
            assert_eq!(kind_hex(k), *hex, "{k}");
            icon_commands(k).unwrap_or_else(|| panic!("{k} has no icon of its own"))
        })
        .collect();
    for (i, a) in glyphs.iter().enumerate() {
        for (j, b) in glyphs.iter().enumerate() {
            assert!(
                i == j || a != b,
                "{} and {} share a glyph",
                kinds[i].0,
                kinds[j].0
            );
        }
    }
    // the globe: a circle (cubic arcs), the equator and two meridians
    let fetch = &glyphs[6];
    assert!(fetch.iter().any(|c| matches!(c, P::Cubic(_))));
    assert!(
        fetch
            .windows(2)
            .any(|w| w == [P::Move([2., 8.]), P::Line([14., 8.])])
    );
    assert_eq!(
        fetch
            .iter()
            .filter(|c| matches!(c, P::Move([8., 2.])))
            .count(),
        2
    );
    // the sparkle is `other`, and only an unknown name borrows it
    assert_eq!(glyphs[7][0], P::Move([8., 2.]));
    assert_eq!(glyphs[7].len(), 9);
    assert_eq!(icon_commands("no-such-icon"), None);
    // the delete bin and the move arrow carry the web's path points
    assert_eq!(glyphs[4][0], P::Move([3., 4.5]));
    assert!(glyphs[5].contains(&P::Line([13.5, 8.])));
}

#[test]
fn gui_polish_loader_is_centered_and_pending_bubbles_are_scoped() {
    use bboxy450_harness::frontends::gui::{
        bridge::PendingPrompt,
        view::{find, layout_text},
    };
    let mut view = HarnessView::default();
    for chrome in [false, true] {
        view.chrome = chrome;
        for size in [[280., 430.], [1120., 800.], [470., 680.]] {
            for preparing in [false, true] {
                let mut state = FrameState::default();
                state.ready = preparing;
                if preparing {
                    let item = ItemId::new();
                    let mut t = Transcript::default();
                    let mut e: Envelope = serde_json::from_str(
                        include_str!("fixtures/protocol/all-items.jsonl")
                            .lines()
                            .next()
                            .unwrap(),
                    )
                    .unwrap();
                    e.event = Event::ItemStarted {
                        turn: None,
                        item,
                        kind: ItemKind::AgentMessage,
                        parent: None,
                        head: None,
                    };
                    t.apply(&e);
                    state.transcript = Arc::new(t);
                }
                let frame = view.render(&state, size).unwrap();
                let layout = frame.layout.as_ref().unwrap();
                let loader = find(layout, "session-loader").unwrap().rect;
                let mark = find(layout, "session-loader/mark")
                    .expect("welcome portrait above loading label");
                let label = find(layout, "session-loader/label").unwrap();
                assert!(mark.rect.bottom() < label.rect.y);
                assert!(
                    !layout_text(find(layout, "session-loader").unwrap())
                        .contains("What are we building")
                );
                assert!(find(layout, "empty/chip/0").is_none());

                let viewport = find(layout, "transcript").unwrap().children[0].rect;
                assert!(
                    ((loader.x + loader.w / 2.) - (viewport.x + viewport.w / 2.)).abs() < 1.,
                    "horizontal: {loader:?} {viewport:?}"
                );
                assert!(
                    ((loader.y + loader.h / 2.) - (viewport.y + viewport.h / 2.)).abs() < 1.,
                    "vertical: {loader:?} {viewport:?}"
                );
            }
        }
    }
    let session = SessionId::new();
    let mut model = Transcript::default();
    model.hello(session, Epoch(7));
    let mut state = FrameState {
        transcript: Arc::new(model),
        ready: true,
        ..Default::default()
    };
    let op = OpId::new();
    state.pending_prompts = Arc::new(
        [(
            op,
            PendingPrompt::new(op, session, Epoch(7), "identical text".into()),
        )]
        .into_iter()
        .collect(),
    );
    let layout = view
        .render(&state, [470., 680.])
        .unwrap()
        .layout
        .as_ref()
        .unwrap();
    assert_eq!(layout_text(layout).matches("identical text").count(), 1);
    assert!(find(layout, &format!("pending/{op}")).is_some());
    let mut switched = (*state.transcript).clone();
    switched.hello(SessionId::new(), Epoch(7));
    state.transcript = Arc::new(switched);
    assert!(
        !layout_text(
            view.render(&state, [470., 680.])
                .unwrap()
                .layout
                .as_ref()
                .unwrap()
        )
        .contains("identical text")
    );
}

#[tokio::test]
async fn profiles_pending_history_remains_visible_through_preparation_gaps() {
    use bboxy450_harness::frontends::gui::{bridge::PendingPrompt, view::layout_text};
    let mut event: Envelope = serde_json::from_str(
        include_str!("fixtures/protocol/all-items.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let mut model = Transcript::default();
    model.apply(&event);
    for body in [
        Item::UserMessage {
            blocks: vec![InputBlock::Text {
                text: "PRIOR-USER-HISTORY".into(),
            }],
        },
        Item::AgentMessage {
            markdown: "PRIOR-ASSISTANT-HISTORY remains visible while I work.".into(),
        },
    ] {
        let item = ItemId::new();
        event.seq = Some(Seq(model.cursor.0 + 1));
        event.at = event.seq.unwrap();
        event.event = Event::ItemStarted {
            turn: None,
            item,
            kind: body.kind(),
            parent: None,
            head: None,
        };
        model.apply(&event);
        event.seq = Some(Seq(model.cursor.0 + 1));
        event.at = event.seq.unwrap();
        event.event = Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body,
        };
        model.apply(&event);
    }
    model.phase = Phase::Streaming;
    let source = Arc::new(model);
    let mut preparer = Preparer::default();
    let prepared = Arc::new(prepare(&mut preparer, source.clone(), 900.).await);
    let device = bbt_render::device::DeviceContext::new().await.unwrap();
    bboxy450_harness::runtime::blocking(move || {
        let mut state = FrameState {
            transcript: source,
            prepared: Some(prepared),
            ready: true,
            ..Default::default()
        };
        let mut view = HarnessView::default();
        let mut renderer = bbt_render::chrome::ChromeRenderer::with_context(
            device,
            bboxy450_harness::frontends::gui::assets::icons(),
        )
        .unwrap();
        let raster = |renderer: &mut bbt_render::chrome::ChromeRenderer,
                      frame: &bboxy450_harness::frontends::gui::view::Frame| {
            bbt_render::chrome::materials::render_composed_to_rgba(
                renderer,
                bbt_render::chrome::materials::Scene {
                    backdrop: bbt_render::chrome::materials::Backdrop::DrawList(&frame.list),
                    items: &frame.rims,
                    width: 900,
                    height: 1100,
                    scale: 1.,
                    time_ms: 0.,
                    theme: bbt_ui_model::Theme::Dark,
                },
                &frame.over,
            )
            .unwrap()
        };

        let check = |view: &mut HarnessView, state: &FrameState, pending: usize| {
            let text = layout_text(
                view.render(state, [900., 1100.])
                    .unwrap()
                    .layout
                    .as_ref()
                    .unwrap(),
            );
            assert!(
                text.contains("PRIOR-USER-HISTORY") && text.contains("PRIOR-ASSISTANT-HISTORY"),
                "prior history disappeared: {text}"
            );
            assert_eq!(text.matches("REPEATED-QUEUED-PROMPT").count(), pending);
        };
        check(&mut view, &state, 0);
        let prior_regions = view.frame.visible.clone();
        assert_eq!(prior_regions.len(), 2);
        let baseline = raster(&mut renderer, &view.frame);

        let prompts: im::OrdMap<_, _> = (0..2)
            .map(|_| {
                let op = OpId::new();
                (
                    op,
                    PendingPrompt::new(
                        op,
                        state.transcript.session.unwrap(),
                        state.transcript.epoch,
                        "REPEATED-QUEUED-PROMPT".into(),
                    ),
                )
            })
            .collect();
        state.pending_prompts = Arc::new(prompts);
        view.edit(bbt_ui::text_edit::Command::Insert(
            "draft typed while active".into(),
        ));
        check(&mut view, &state, 2);
        // Metadata/phase publication wraps the same retained log in a new Arc.
        state.transcript = Arc::new((*state.transcript).clone());
        check(&mut view, &state, 2);
        state.prepared = None; // the next worker preparation is deliberately delayed
        check(&mut view, &state, 2);
        let directory = std::env::var_os("PROFILES_PENDING_CAPTURE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("bboxy-profiles-pending-captures"));
        std::fs::create_dir_all(&directory).unwrap();
        let frame = view.render(&state, [900., 1100.]).unwrap();
        let rgba = raster(&mut renderer, frame);
        for (_, _, region) in &prior_regions {
            let pixels = |image: &[u8]| {
                let mut out = Vec::new();
                for y in
                    region.y.floor().max(0.) as usize..region.bottom().ceil().min(1100.) as usize
                {
                    for x in
                        region.x.floor().max(0.) as usize..region.right().ceil().min(900.) as usize
                    {
                        let i = (y * 900 + x) * 4;
                        out.push([image[i], image[i + 1], image[i + 2], image[i + 3]]);
                    }
                }
                out
            };
            let before = pixels(&baseline);
            let after = pixels(&rgba);
            assert!(
                before
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    > 8,
                "baseline history has no rasterized glyph ink"
            );
            assert!(
                before == after,
                "prior history raster changed or disappeared in region {region:?}"
            );
        }
        for (id, _, _) in &prior_regions {
            assert!(
                frame
                    .visible
                    .iter()
                    .any(|(visible, _, r)| visible == id && r.h > 0.),
                "prior history was clipped to zero height"
            );
        }
        let file =
            std::fs::File::create(directory.join("prior-history-and-gray-tail.png")).unwrap();
        let mut png = png::Encoder::new(file, 900, 1100);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.write_header().unwrap().write_image_data(&rgba).unwrap();
        // Refused/unknown status updates cannot discard the retained history.
        let queued = state.pending_prompts.clone();
        let mut refused = (*state.pending_prompts).clone();
        for op in refused.keys().copied().collect::<Vec<_>>() {
            refused.get_mut(&op).unwrap().status = "Rejected · message was not sent".into();
        }
        state.pending_prompts = Arc::new(refused);
        check(&mut view, &state, 2);
        state.pending_prompts = queued;
        let mut model = (*state.transcript).clone();
        for _ in 0..2 {
            let item = ItemId::new();
            event.seq = Some(Seq(model.cursor.0 + 1));
            event.at = event.seq.unwrap();
            event.event = Event::ItemStarted {
                turn: None,
                item,
                kind: ItemKind::UserMessage,
                parent: None,
                head: None,
            };
            model.apply(&event);
            event.seq = Some(Seq(model.cursor.0 + 1));
            event.at = event.seq.unwrap();
            event.event = Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::UserMessage {
                    blocks: vec![InputBlock::Text {
                        text: "REPEATED-QUEUED-PROMPT".into(),
                    }],
                },
            };
            model.apply(&event);
        }
        state.transcript = Arc::new(model);
        check(&mut view, &state, 2); // durable log arrived, preparation still delayed: gray only
        state.prepared = Some(Arc::new(
            preparer.prepare(state.transcript.clone(), 900.).unwrap(),
        ));
        state.pending_prompts = Arc::new(Default::default());
        check(&mut view, &state, 2); // exactly two durable blue rows, with all history
        let mut switched = Transcript::default();
        switched.hello(SessionId::new(), Epoch(state.transcript.epoch.0 + 1));
        state.transcript = Arc::new(switched);
        state.prepared = None;
        assert!(
            !layout_text(
                view.render(&state, [900., 1100.])
                    .unwrap()
                    .layout
                    .as_ref()
                    .unwrap()
            )
            .contains("PRIOR-USER-HISTORY")
        );
    })
    .await;
}
