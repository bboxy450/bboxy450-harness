// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "embedded")]
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
        let popup = frame.list.primitives.iter().find_map(|p| match p {
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
        let full = renderer.render_to_rgba(&frame.list, 900, 300, 1.).unwrap();
        let broken_full = renderer.render_to_rgba(&without_surface(&frame.list), 900, 300, 1.).unwrap();
        let frame = view.render(&empty, [900., 300.]).unwrap();
        let blank = renderer.render_to_rgba(&frame.list, 900, 300, 1.).unwrap();
        let broken_blank = renderer.render_to_rgba(&without_surface(&frame.list), 900, 300, 1.).unwrap();
        assert!(view.cancel_prompt(&Bridge::new(Arc::new(Wake))));
        let frame = view.render(&populated, [900., 300.]).unwrap();
        let transcript = renderer.render_to_rgba(&frame.list, 900, 300, 1.).unwrap();
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
    assert_eq!(frame.visible.len(), state.transcript.items.len());
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
    assert!(
        text.contains("exit 0"),
        "terminal exit must be explicit readable text"
    );
    assert!(!text.contains("Some("));
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
    assert!(text.contains(command));
    assert!(text.contains("exit 0"));
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
    // every item is addressable; folded ones (approval, terminal, diff) live inside a tool card
    let layout = frame.layout.as_ref().unwrap();
    assert!(state.transcript.order.iter().all(|id| {
        bboxy450_harness::frontends::gui::view::find(layout, &format!("item/{id}")).is_some()
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
    assert!(
        find(&live, "composer/sweep").is_some(),
        "running sweep line"
    );
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
    assert!(
        find(&ask, "composer/sweep").is_none(),
        "ask is not the run look"
    );
    assert!(
        find(&ask, "composer/hint").is_some(),
        "ask shows the 1/2/3 hint"
    );
}

#[tokio::test]
async fn v1_additions_have_readable_rich_cards() {
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
        "Command:",
        "Hook:",
        "Developer:",
        "Native:",
        "Audio:",
        "Read reference",
        "fixture-mcp",
    ] {
        assert!(text.contains(label), "missing {label}: {text}");
    }
    assert!(!text.contains("opaque fixture"));
}
