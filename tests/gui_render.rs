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
        preparer.prepare(state.transcript.clone(), 1120.).unwrap(),
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
        preparer.prepare(grown.transcript.clone(), 1120.).unwrap(),
    ));
    assert_eq!(
        grown.prepared.as_ref().unwrap().layout_builds,
        state.prepared.as_ref().unwrap().layout_builds + 2,
        "only the open approval and appended item rebuild"
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
    state.prepared = Some(Arc::new(
        Preparer::default()
            .prepare(state.transcript.clone(), 1120.)
            .unwrap(),
    ));
    let mut view = HarnessView::default();
    let frame = view.render(&state, [1120., 7000.]).unwrap();
    let text = bboxy450_harness::frontends::gui::view::layout_text(frame.layout.as_ref().unwrap());
    assert!(
        !text.contains("\"kind\":"),
        "raw JSON is not a readable card: {text}"
    );
    assert!(
        text.contains("Exit code: 0"),
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
    let mut terminal_id = None;
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(e) = observer.recv().await {
            projection.apply(&e);
            match e.event {
                Event::ItemStarted { item, kind: ItemKind::Terminal, .. } => {
                    terminal_id = Some(item);
                    saw_head = matches!(&projection.items[&item].body, Item::Terminal { command: c, .. } if c == command);
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
    assert!(text.contains("Exit code: 0"));
    assert!(
        text.contains("stream-headstream-tail"),
        "completed blob must supply output, not just the command: {text}"
    );
    bridge.detach().unwrap().await.unwrap();
    support::close(session).await;
    println!(
        "gui-live: completed blob content rendered; typed head={saw_head}, live snapshot={saw_live}"
    );
    assert!(
        saw_head,
        "live Envelope head must name the command before completion"
    );
    assert!(
        saw_live,
        "live gateway must deliver coalesced v0.2 output snapshots"
    );
}
