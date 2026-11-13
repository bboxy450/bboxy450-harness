// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        anim::Clock,
        bridge::FrameState,
        model::{Transcript, ViewItem},
        rich::Face,
        view::{HarnessView, Preparer, find},
    },
    protocol::*,
};
use bbt_ui::components::LayoutKind;
use std::sync::Arc;
const SIZE: [f32; 2] = [900., 700.];

fn push(model: &mut Transcript, body: Item) -> ItemId {
    let id = ItemId::new();
    model.order.push_back(id);
    model.items.insert(
        id,
        ViewItem {
            id,
            turn: None,
            parent: None,
            started: Seq(1),
            completed: Some(Seq(2)),
            kind: body.kind(),
            body,
            status: Some(ItemStatus::Completed),
            terminal_tail: String::new(),
            started_ms: 0,
            completed_ms: Some(1),
        },
    );
    id
}
fn history() -> Transcript {
    let mut model = Transcript {
        session: Some(SessionId::new()),
        cursor: Seq(2),
        ..Default::default()
    };
    push(
        &mut model,
        Item::AgentMessage {
            markdown: "Paragraph of retained history.\n\n".repeat(100),
        },
    );
    model
}
fn prepared(model: Transcript, faces: im::OrdMap<ItemId, Face>) -> FrameState {
    let model = Arc::new(model);
    let mut p = Preparer::default();
    FrameState {
        prepared: Some(Arc::new(
            p.prepare_with_faces(model.clone(), SIZE[0], &faces)
                .unwrap(),
        )),
        transcript: model,
        faces: Arc::new(faces),
        ready: true,
        ..Default::default()
    }
}

#[tokio::test]
async fn resume_jumps_to_latest_after_delayed_preparation_and_keeps_manual_scroll() {
    bboxy450_harness::runtime::blocking(|| {
        let state = prepared(history(), Default::default());
        let mut loading = state.clone();
        loading.prepared = None;
        loading.ready = false;
        let mut view = HarnessView::default();
        view.anim = Clock::live(false);
        view.anim.tick(0.);
        view.render(&loading, SIZE).unwrap();
        view.anim.tick(1000.);
        let frame = view.render(&state, SIZE).unwrap();
        assert!(frame.max_scroll > 1000.);
        assert_eq!(
            frame.scroll, frame.max_scroll,
            "a returning view must not glide from the start"
        );
        assert!(view.wheel([880., 300.], -250.));
        let manual = view.render(&state, SIZE).unwrap().scroll;
        for tick in 1..80 {
            view.anim.tick(1000. + tick as f32 * 40.);
            assert_eq!(view.render(&state, SIZE).unwrap().scroll, manual);
        }
        view.follow_latest();
        let frame = view.render(&state, SIZE).unwrap();
        assert_eq!(frame.scroll, frame.max_scroll);
        let mut next = (*state.transcript).clone();
        next.epoch = Epoch(next.epoch.0 + 1);
        view.wheel([880., 300.], -250.);
        let next = prepared(next, Default::default());
        let frame = view.render(&next, SIZE).unwrap();
        assert_eq!(
            frame.scroll, frame.max_scroll,
            "a new epoch starts at the latest message"
        );
    })
    .await;
}

#[tokio::test]
async fn inset_output_scroll_has_one_owner_and_hands_off_only_at_its_edge() {
    bboxy450_harness::runtime::blocking(|| {
        let mut model = history();
        let body = serde_json::from_value(serde_json::json!({
            "kind":"tool_call", "call_id":"read", "name":"read", "tool_kind":"read",
            "input_raw":"{\"path\":\"notes.md\"}",
            "output":[{"type":"text","text":format!("# Output\n{}", "- output line\n".repeat(100))}],
            "locations":[],"spill":null
        })).unwrap();
        let id = push(&mut model, body);
        let state = prepared(model, im::ordmap! { id => Face { output_closed: true, ..Default::default() } });
        let mut view = HarnessView::default();
        let frame = view.render(&state, SIZE).unwrap();
        let outer = frame.scroll;
        let key = format!("mdout/{id}");
        let node = &find(frame.layout.as_ref().unwrap(), &key).unwrap().children[0];
        let at = [node.rect.x + 20., node.rect.y + 30.];
        assert!(view.wheel(at, 200.));
        let frame = view.render(&state, SIZE).unwrap();
        assert_eq!(frame.scroll, outer);
        let node = &find(frame.layout.as_ref().unwrap(), &key).unwrap().children[0];
        assert!(matches!(node.kind, LayoutKind::ScrollView { offset, .. } if offset[1] > 0.));
        assert!(view.wheel(at, -100_000.));
        assert_eq!(view.render(&state, SIZE).unwrap().scroll, outer);
        assert!(view.wheel(at, -180.));
        assert_eq!(view.render(&state, SIZE).unwrap().scroll, outer - 180.);
        assert!(!view.scroll.following());
    }).await;
}

#[tokio::test]
async fn vertical_swipes_never_pan_a_single_long_line() {
    bboxy450_harness::runtime::blocking(|| {
        let mut model = history();
        let body = serde_json::from_value(serde_json::json!({
            "kind":"tool_call", "call_id":"wide", "name":"shell", "tool_kind":"execute",
            "input_raw":"", "output":[{"type":"text","text":"long ".repeat(600)}],
            "locations":[],"spill":null
        }))
        .unwrap();
        let id = push(&mut model, body);
        let state = prepared(model, Default::default());
        let mut view = HarnessView::default();
        let frame = view.render(&state, SIZE).unwrap();
        let key = format!("codeout/{id}/output");
        let node = &find(frame.layout.as_ref().unwrap(), &key)
            .expect("code output")
            .children[0];
        let at = [node.rect.x + 20., node.rect.y + 10.];
        assert!(view.wheel_delta(at, [180., 0.]));
        let frame = view.render(&state, SIZE).unwrap();
        let node = &find(frame.layout.as_ref().unwrap(), &key).unwrap().children[0];
        let x = match node.kind {
            LayoutKind::ScrollView { offset, .. } => offset[0],
            _ => panic!("scroll panel"),
        };
        assert!(x > 0.);
        assert!(view.wheel(at, -150.));
        assert!(!view.scroll.following());
        view.follow_latest();
        let frame = view.render(&state, SIZE).unwrap();
        let node = &find(frame.layout.as_ref().unwrap(), &key).unwrap().children[0];
        assert!(matches!(node.kind, LayoutKind::ScrollView { offset, .. } if offset[0] == x));
    })
    .await;
}
