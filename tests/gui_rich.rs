// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![cfg(feature = "embedded")]
mod support;
use bboxy450_harness::{
    frontends::gui::{bridge::Bridge, view::HarnessView},
    protocol::*,
    runtime,
};
use std::{sync::Arc, time::Duration};
struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
fn replay_file(root: &std::path::Path, text: &str) -> std::path::PathBuf {
    let base: Envelope = serde_json::from_str(
        include_str!("fixtures/protocol/all-items.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let item = ItemId::new();
    let mut started = base.clone();
    started.seq = Some(Seq(2));
    started.at = Seq(2);
    started.event = Event::ItemStarted {
        turn: None,
        item,
        kind: ItemKind::AgentMessage,
        parent: None,
        head: None,
    };
    let mut completed = base.clone();
    completed.seq = Some(Seq(3));
    completed.at = Seq(3);
    completed.event = Event::ItemCompleted {
        item,
        status: ItemStatus::Completed,
        body: Item::AgentMessage {
            markdown: text.into(),
        },
    };
    let path = root.join("replay.jsonl");
    let mut bytes = Vec::new();
    for e in [base, started, completed] {
        serde_json::to_writer(&mut bytes, &e).unwrap();
        bytes.push(b'\n');
    }
    std::fs::write(&path, bytes).unwrap();
    path
}
async fn ready(bridge: &mut Bridge) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if bridge.state.prepared.is_some() {
                break;
            }
            assert!(bridge.state.error.is_none(), "{:?}", bridge.state.error);
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn large_markdown_uses_the_production_replay_preparation_worker() {
    bbt_rt::register_ui_thread();
    let temp = tempfile::tempdir().unwrap();
    let text = format!(
        "{}\n\n```rust\nfn main() {{ println!(\"ready\"); }}\n```\n",
        "content ".repeat(8192)
    );
    let before = runtime::blocking_stats().calls(runtime::BlockingWork::MarkdownPrepare);
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(replay_file(temp.path(), &text));
    ready(&mut bridge).await;
    let prepared = bridge.state.prepared.as_ref().unwrap();
    assert!(!prepared.prepared_on_ui);
    assert!(prepared.parsed_markdown_bytes >= text.len());
    assert!(
        runtime::blocking_stats().calls(runtime::BlockingWork::MarkdownPrepare) > before,
        "real replay must execute the markdown permit hook"
    );
    bridge.detach().unwrap().await.unwrap();
}
#[tokio::test]
async fn the_end_of_a_64k_reply_is_readable_after_scrolling() {
    let temp = tempfile::tempdir().unwrap();
    let text = format!(
        "{}\nEND-OF-LONG-REPLY",
        "readable paragraph\n\n".repeat(4096)
    );
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(replay_file(temp.path(), &text));
    ready(&mut bridge).await;
    let id = bridge.state.transcript.order[0];
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [1120., 860.]).unwrap();
    assert!(frame.max_scroll > 0.);
    assert_eq!(frame.scroll, frame.max_scroll);
    assert!(
        frame
            .item_text(id)
            .is_some_and(|s| s.contains("END-OF-LONG-REPLY")),
        "long reply must remain fully readable"
    );
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn streamed_64k_markdown_has_linear_parse_work_and_reuses_completed_layout() {
    use bboxy450_harness::frontends::gui::{bridge::read_replay, view::Preparer};
    let mut model = read_replay("tests/fixtures/protocol/all-items.jsonl".into())
        .await
        .unwrap();
    let id = model
        .items
        .values()
        .find(|r| r.kind == ItemKind::AgentMessage)
        .unwrap()
        .id;
    let mut row = model.items[&id].clone();
    row.completed = None;
    row.status = None;
    row.body = Item::AgentMessage {
        markdown: String::new(),
    };
    model.items = im::OrdMap::unit(id, row);
    model.order = im::Vector::unit(id);
    let (parsed, received, first, second) = runtime::blocking(move || {
        let text = format!("{}x", "word ".repeat(13107));
        assert_eq!(text.len(), 64 * 1024);
        let mut preparer = Preparer::default();
        let mut result = None;
        for end in (256..=text.len()).step_by(256) {
            model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
                markdown: text[..end].into(),
            };
            result = Some(preparer.prepare(Arc::new(model.clone()), 1120.).unwrap());
        }
        model.items.get_mut(&id).unwrap().completed = Some(Seq(100));
        model.items.get_mut(&id).unwrap().status = Some(ItemStatus::Completed);
        let full = preparer.prepare(Arc::new(model.clone()), 1120.).unwrap();
        assert!(result.is_some());
        let first = full.layout_builds;
        model.phase = Phase::Thinking;
        let second = preparer
            .prepare(Arc::new(model), 1120.)
            .unwrap()
            .layout_builds;
        (
            full.parsed_markdown_bytes,
            full.received_markdown_bytes,
            first,
            second,
        )
    })
    .await;
    assert_eq!(received, 64 * 1024);
    assert!(parsed > 0);
    assert!(
        parsed <= 4 * received,
        "quadratic reparsing: {parsed} parsed bytes for {received} received bytes"
    );
    assert_eq!(first, second, "completed layout must remain retained");
}
#[test]
fn editor_history_and_slash_completion_use_the_upgraded_editor() {
    use bboxy450_harness::frontends::gui::editor::Input;
    use bbt_ui::{
        modules::composer::{Command, Composer, Outcome},
        text_edit::Command as Edit,
    };
    let mut input = Input::new(Composer {
        to: vec![bbt_ui::modules::composer::Address {
            label: "Agent".into(),
            destination: "session".into(),
            ..Default::default()
        }],
        focus: true,
        ..Default::default()
    });
    input.handle(Command::Edit(Edit::Insert("first prompt".into())));
    let Outcome::Submit(sent) = input.handle(Command::Send) else {
        panic!("submit")
    };
    input.stored(&sent);
    input.vertical(true);
    assert_eq!(input.view.text, "first prompt");
    input.vertical(false);
    assert!(input.view.text.is_empty());
    input.handle(Command::Edit(Edit::Insert("line one\nline two".into())));
    input.vertical(true);
    assert_eq!(input.view.text, "line one\nline two");
    input.vertical(true);
    assert_eq!(input.view.text, "first prompt");
    input.vertical(false);
    assert_eq!(input.view.text, "line one\nline two");
    input.handle(Command::Edit(Edit::SelectAll));
    input.handle(Command::Edit(Edit::Insert("/int".into())));
    let mut popover = input.popover().unwrap();
    let edit = popover
        .handle(bbt_ui::modules::approval::Key::Enter)
        .unwrap();
    input.handle(Command::Edit(edit));
    assert_eq!(input.view.text, "/interrupt");
    input.handle(Command::Edit(Edit::SelectAll));
    input.handle(Command::Edit(Edit::Insert("@file".into())));
    assert_eq!(input.completion().unwrap().trigger, '@');
    assert!(input.popover().is_none());
}

#[tokio::test]
async fn the_view_shows_live_tool_role_and_tail_then_the_full_terminal_blob() {
    use bboxy450_harness::frontends::gui::view::Action;
    use bbt_ui::modules::approval::Key;
    let command = "printf LIVE-TAIL; while [ ! -f gui-release ]; do sleep 0.01; done; python3 -c 'print(\"payload \" * 10000 + \"BLOB-END\", end=\"\")'";
    let first=format!("data: {}\n\ndata: [DONE]\n\n",serde_json::json!({"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"rich-live","type":"function","function":{"name":"bash","arguments":serde_json::json!({"command":command}).to_string()}}]},"finish_reason":"tool_calls"}]})).into_bytes();
    let session = support::session(vec![first, support::sse("text")], true).await;
    let mut observer = support::subscribe(&session.gateway, false);
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.attach_with_blobs(
        session.gateway.clone(),
        bboxy450_harness::store::Blobs {
            dir: session
                .temp
                .path()
                .join("sessions")
                .join(session.gateway.session().to_string())
                .join("blobs"),
            redactor: Default::default(),
        },
    );
    ready(&mut bridge).await;
    let mut view = HarnessView::default();
    view.composer
        .handle(bbt_ui::modules::composer::Command::Edit(
            bbt_ui::text_edit::Command::Insert("run it".into()),
        ));
    view.activate(Action::Send, &bridge);
    let observed = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            let terminal = bridge
                .state
                .transcript
                .items
                .values()
                .find(|r| {
                    r.kind == ItemKind::Terminal
                        && r.status.is_none()
                        && r.terminal_tail.contains("LIVE-TAIL")
                })
                .map(|r| r.id);
            if let Some(id) = terminal {
                let frame = view.render(&bridge.state, [1120., 7000.])?;
                let text = frame.item_text(id).unwrap_or_default();
                let tool = bridge
                    .state
                    .transcript
                    .items
                    .values()
                    .find(|r| r.kind == ItemKind::ToolCall)
                    .unwrap()
                    .id;
                if text.contains("LIVE-TAIL")
                    && text.contains("TOOL")
                    && frame.item_text(tool).is_some_and(|s| s.contains("Running"))
                {
                    return Ok::<_, String>(id);
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await;
    std::fs::write(session.temp.path().join("gui-release"), "").unwrap();
    support::end(&mut observer).await;
    let id = observed.expect("live viewport timed out").unwrap();
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if bridge
                .state
                .transcript
                .items
                .get(&id)
                .is_some_and(|r| r.status.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    let text = view
        .render(&bridge.state, [1120., 200000.])
        .unwrap()
        .item_text(id)
        .unwrap();
    assert!(
        text.matches("payload").count() >= 10000,
        "completed terminal blob lost its full output"
    );
    assert!(text.contains("BLOB-END"));
    assert!(!view.approval_key(Key::Enter, &bridge));
    let retired = bridge
        .state
        .transcript
        .items
        .values()
        .find(|r| r.kind == ItemKind::Approval)
        .unwrap()
        .id;
    view.activate(Action::Approval(retired, ApprovalDecision::Allow), &bridge);
    let retired_blocked = view.notice.as_deref() == Some("Approval expired");
    bridge.detach().unwrap().await.unwrap();
    let log = support::close(session).await;
    assert!(retired_blocked);
    assert!(
        !log.iter()
            .any(|e| matches!(e.event, Event::OpRejected { .. })),
        "retired view dispatched a stale operation"
    );
}
#[tokio::test]
async fn closing_an_old_gateway_does_not_close_a_new_actor_with_the_same_label() {
    use bboxy450_harness::{
        engine::{Config, SessionActor},
        gateway::Gateway,
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
    };
    let label = SessionId::new();
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    async fn actor(
        root: &std::path::Path,
        label: SessionId,
    ) -> (
        bboxy450_harness::gateway::InProc,
        tokio::task::JoinHandle<std::io::Result<()>>,
    ) {
        let mut meta = support::meta(root.into());
        meta.session = label;
        let mut provider = ScriptedProvider::new(vec![support::sse("text"), support::sse("text")]);
        provider.delay = Duration::from_millis(10);
        SessionActor::open(
            Config {
                meta,
                profiles: root.join("profiles"),
                sessions: root.join("sessions"),
                provider: Arc::new(provider),
                model: "scripted".into(),
                policy: Policy::allow_all(Unattended::Deny),
                redactor: Default::default(),
                profile_error: None,
                max_rounds: 50,
            },
            OpId::new(),
        )
        .await
        .unwrap()
    }
    let (old, old_task) = actor(a.path(), label).await;
    let (new, new_task) = actor(b.path(), label).await;
    let mut old_events = support::subscribe(&old, false);
    let mut new_events = support::subscribe(&new, false);
    new.send(support::prompt(&new, "keep running")).unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        while !matches!(
            new_events.recv().await.unwrap().event,
            Event::TurnStarted { .. }
        ) {}
    })
    .await
    .unwrap();
    old.send(support::op(&old, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        while !matches!(old_events.recv().await.unwrap().event, Event::SessionClosed) {}
    })
    .await
    .unwrap();
    let done = support::end(&mut new_events).await;
    assert!(!done.iter().any(|e| matches!(e.event, Event::SessionClosed)));
    assert!(done.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    new.send(support::prompt(&new, "still alive")).unwrap();
    support::end(&mut new_events).await;
    new.send(support::op(&new, Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(8), old_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(8), new_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

async fn until(bridge: &mut Bridge, predicate: impl Fn(&Bridge) -> bool) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if predicate(bridge) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn disclosure_controls_change_the_view_without_reparsing_or_core_operations() {
    use bboxy450_harness::frontends::gui::{rich::Toggle, view::Action};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay("tests/fixtures/gui/rich-all-items.jsonl".into());
    ready(&mut bridge).await;
    let reasoning = bridge
        .state
        .transcript
        .items
        .values()
        .find(|r| r.kind == ItemKind::Reasoning)
        .unwrap()
        .id;
    let source = bridge.state.transcript.clone();
    let parsed = bridge
        .state
        .prepared
        .as_ref()
        .unwrap()
        .parsed_markdown_bytes;
    let mut view = HarnessView::default();
    let action = view
        .render(&bridge.state, [1120., 7000.])
        .unwrap()
        .actions
        .iter()
        .find_map(|(_, a)| {
            matches!(a,Action::Toggle(id,Toggle::Reasoning) if *id==reasoning).then(|| a.clone())
        })
        .unwrap();
    view.activate(action, &bridge);
    until(&mut bridge, |b| {
        b.state
            .faces
            .get(&reasoning)
            .is_some_and(|f| f.reasoning_closed)
    })
    .await;
    let text = view
        .render(&bridge.state, [1120., 7000.])
        .unwrap()
        .item_text(reasoning)
        .unwrap();
    assert!(text.contains("Show reasoning"));
    assert!(!text.contains("Compare the current contents"));
    assert_eq!(bridge.state.transcript.items, source.items);
    assert_eq!(
        bridge
            .state
            .prepared
            .as_ref()
            .unwrap()
            .parsed_markdown_bytes,
        parsed
    );
    assert!(view.notice.is_none());
    view.activate(Action::Toggle(reasoning, Toggle::Reasoning), &bridge);
    until(&mut bridge, |b| {
        b.state
            .faces
            .get(&reasoning)
            .is_some_and(|f| !f.reasoning_closed)
    })
    .await;
    assert!(
        view.render(&bridge.state, [1120., 7000.])
            .unwrap()
            .item_text(reasoning)
            .unwrap()
            .contains("Compare the current contents")
    );
    bridge.detach().unwrap().await.unwrap();
}
#[tokio::test]
async fn a_long_code_line_remains_readable_in_the_expandable_source_view() {
    use bboxy450_harness::frontends::gui::{rich::Toggle, view::Action};
    let temp = tempfile::tempdir().unwrap();
    let text = format!("```text\n{} END-OF-LONG-CODE\n```", "word ".repeat(14000));
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(replay_file(temp.path(), &text));
    ready(&mut bridge).await;
    let id = bridge.state.transcript.order[0];
    let mut view = HarnessView::default();
    let action = view
        .render(&bridge.state, [1120., 860.])
        .unwrap()
        .actions
        .iter()
        .find_map(|(_, a)| {
            matches!(a,Action::Toggle(item,Toggle::Source) if *item==id).then(|| a.clone())
        })
        .expect("source disclosure must be visible");
    view.activate(action, &bridge);
    until(&mut bridge, |b| {
        b.state.faces.get(&id).is_some_and(|f| f.source)
    })
    .await;
    assert!(
        view.render(&bridge.state, [1120., 860.])
            .unwrap()
            .item_text(id)
            .is_some_and(|s| s.contains("END-OF-LONG-CODE"))
    );
    bridge.detach().unwrap().await.unwrap();
}
#[tokio::test]
async fn approval_keyboard_and_escape_resolve_the_actual_prompt() {
    use bboxy450_harness::frontends::gui::view::Action;
    use bbt_ui::modules::approval::Key;
    for (key, expected) in [
        (Some(Key::Down), ApprovalDecision::AllowAlways),
        (Some(Key::End), ApprovalDecision::Deny { reason: None }),
        (
            None,
            ApprovalDecision::Deny {
                reason: Some("Cancelled".into()),
            },
        ),
    ] {
        let session =
            support::session(vec![support::sse("edit"), support::sse("text")], false).await;
        let mut events = support::subscribe(&session.gateway, false);
        let mut bridge = Bridge::new(Arc::new(Wake));
        bridge.attach(session.gateway.clone());
        ready(&mut bridge).await;
        let mut view = HarnessView::default();
        view.edit(bbt_ui::text_edit::Command::Insert("edit input".into()));
        view.activate(Action::Send, &bridge);
        until(&mut bridge, |b| {
            b.state.prepared.is_some()
                && b.state
                    .transcript
                    .items
                    .values()
                    .any(|r| r.kind == ItemKind::Approval && r.status.is_none())
        })
        .await;
        assert_eq!(
            view.render(&bridge.state, [1120., 7000.])
                .unwrap()
                .actions
                .iter()
                .filter(|(_, a)| matches!(a, Action::Approval(..)))
                .count(),
            3
        );
        if let Some(key) = key {
            assert!(view.approval_key(key, &bridge));
            assert!(view.approval_key(Key::Enter, &bridge));
        } else {
            view.edit(bbt_ui::text_edit::Command::Insert("/int".into()));
            assert!(view.completion_open());
            assert!(view.cancel_prompt(&bridge));
            assert_eq!(view.composer.view.text, "/int");
        }
        let ended = support::end(&mut events).await;
        assert!(ended.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:Item::Approval{decision:Some(d),..},..} if d==&expected)));
        bridge.detach().unwrap().await.unwrap();
        support::close(session).await;
    }
}

#[test]
fn a_completion_from_a_retired_token_does_not_replace_new_typing() {
    use bbt_ui::{modules::composer::Command, text_edit::Command as Edit};
    let mut view = HarnessView::default();
    view.edit(Edit::Insert("/int".into()));
    let mut old = view.composer.popover().unwrap();
    let stale = old.handle(bbt_ui::modules::approval::Key::Enter).unwrap();
    view.edit(Edit::SelectAll);
    view.edit(Edit::Insert("/co".into()));
    view.composer.handle(Command::Edit(stale));
    assert_eq!(view.composer.view.text, "/co");
}
