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

fn markdown_tool(kind: ToolKind, name: &str, path: &str, text: &str) -> Item {
    serde_json::from_value(serde_json::json!({
        "kind": "tool_call", "call_id": name, "name": name, "tool_kind": kind,
        "input_raw": serde_json::json!({"path": path, "command": "printf markdown"}).to_string(),
        "output": [{"type": "text", "text": text}], "locations": [], "spill": null
    }))
    .unwrap()
}

fn markdown_replay(
    root: &std::path::Path,
    items: Vec<(Item, Option<ItemStatus>)>,
) -> (std::path::PathBuf, Vec<ItemId>) {
    let base: Envelope = serde_json::from_str(
        include_str!("fixtures/protocol/all-items.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let mut events = vec![base.clone()];
    let mut ids = vec![];
    for (body, status) in items {
        let item = ItemId::new();
        ids.push(item);
        let mut event = base.clone();
        event.seq = Some(Seq(events.len() as u64 + 1));
        event.at = event.seq.unwrap();
        event.event = Event::ItemStarted {
            turn: None,
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        };
        events.push(event.clone());
        if let Some(status) = status {
            event.seq = Some(Seq(events.len() as u64 + 1));
            event.at = event.seq.unwrap();
            event.event = Event::ItemCompleted { item, status, body };
        } else if let Item::Reasoning { summary, .. } = body {
            event.seq = None;
            event.event = Event::ItemDelta {
                item,
                delta: Delta::Reasoning { append: summary },
            };
        } else {
            panic!("live fixture is reasoning");
        }
        events.push(event);
    }
    let path = root.join("markdown-output.jsonl");
    let mut bytes = Vec::new();
    for event in events {
        serde_json::to_writer(&mut bytes, &event).unwrap();
        bytes.push(b'\n');
    }
    std::fs::write(&path, bytes).unwrap();
    (path, ids)
}

fn markdown_canvas(node: &bbt_ui::components::Layout) -> bool {
    if matches!(&node.kind, bbt_ui::components::LayoutKind::Named(name) if name.ends_with("/canvas"))
    {
        return true;
    }
    node.children.iter().any(markdown_canvas)
}

#[tokio::test]
async fn markdown_output_read_renders_headings_lists_and_scrolls_without_reparsing() {
    use bboxy450_harness::frontends::gui::{rich::Toggle, view::find};
    let temp = tempfile::tempdir().unwrap();
    let mut text = "1: # Worker-rendered title\n2:\n".to_owned();
    for i in 0..60 {
        text.push_str(&format!("{}: - list entry {i}\n", i + 3));
    }
    text.push_str("63: - END-OF-MARKDOWN-READ\n");
    let (path, ids) = markdown_replay(
        temp.path(),
        vec![(
            markdown_tool(ToolKind::Read, "read", "notes.md", &text),
            Some(ItemStatus::Completed),
        )],
    );
    let id = ids[0];
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(path);
    ready(&mut bridge).await;
    bridge.toggle(id, Toggle::Output);
    until(&mut bridge, |b| {
        b.state.faces.get(&id).is_some_and(|f| f.output_closed)
    })
    .await;
    let parsed = bridge
        .state
        .prepared
        .as_ref()
        .unwrap()
        .parsed_markdown_bytes;
    assert!(parsed > 0, "Read must enter the production Markdown worker");
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [900., 1100.]).unwrap();
    let layout = frame.layout.as_ref().unwrap();
    let panel = find(layout, &format!("mdpanel/{id}")).expect("Read has an inset panel");
    assert!(panel.rect.h <= 420.01, "panel height {}", panel.rect.h);
    let scroll = find(layout, &format!("mdout/{id}"))
        .unwrap()
        .children
        .first()
        .unwrap();
    assert!(
        matches!(scroll.kind, bbt_ui::components::LayoutKind::ScrollView { content, .. } if content[1] > scroll.rect.h)
    );
    fn heading_and_list(node: &bbt_ui::components::Layout, heading: &mut bool, list: &mut bool) {
        if let bbt_ui::components::LayoutKind::Text { runs, .. } = &node.kind {
            *heading |= runs
                .iter()
                .any(|r| r.text.contains("title") && r.style.weight >= 600 && !r.style.mono);
        }
        if let bbt_ui::components::LayoutKind::Named(name) = &node.kind {
            *list |= name.contains("/bullet");
        }
        for child in &node.children {
            heading_and_list(child, heading, list);
        }
    }
    let (mut heading, mut list) = (false, false);
    heading_and_list(panel, &mut heading, &mut list);
    assert!(
        heading && list,
        "Read must contain a styled heading ({heading}) and list markers ({list})"
    );
    let visible = frame.item_text(id).unwrap();
    assert!(!visible.contains("1:") && !visible.contains("# Worker-rendered"));
    assert!(!visible.contains("END-OF-MARKDOWN-READ"));
    let at = [scroll.rect.x + 20., scroll.rect.y + 20.];
    assert!(view.wheel(at, 100_000.));
    let frame = view.render(&bridge.state, [900., 1100.]).unwrap();
    assert!(
        frame
            .item_text(id)
            .unwrap()
            .contains("END-OF-MARKDOWN-READ")
    );
    assert_eq!(
        bridge
            .state
            .prepared
            .as_ref()
            .unwrap()
            .parsed_markdown_bytes,
        parsed
    );
    assert!(view.wheel(at, -100_000.));
    assert!(
        view.render(&bridge.state, [900., 1100.])
            .unwrap()
            .item_text(id)
            .unwrap()
            .contains("Worker-rendered title")
    );
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn markdown_output_mermaid_tool_result_is_a_native_figure_and_execute_stays_literal() {
    use bboxy450_harness::frontends::gui::{rich::Toggle, view::find};
    let text = "```mermaid\ngraph LR\nA[Worker] --> B[Figure]\n```";
    let temp = tempfile::tempdir().unwrap();
    let (path, ids) = markdown_replay(
        temp.path(),
        vec![
            (
                markdown_tool(ToolKind::Fetch, "fetch", "", text),
                Some(ItemStatus::Completed),
            ),
            (
                markdown_tool(ToolKind::Execute, "bash", "", text),
                Some(ItemStatus::Completed),
            ),
            (
                markdown_tool(ToolKind::Read, "read", "failed.md", text),
                Some(ItemStatus::Failed {
                    code: None,
                    reason: "read failed".into(),
                }),
            ),
        ],
    );
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(path);
    ready(&mut bridge).await;
    bridge.toggle(ids[0], Toggle::Output);
    until(&mut bridge, |b| {
        b.state.faces.get(&ids[0]).is_some_and(|f| f.output_closed)
    })
    .await;
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [900., 1800.]).unwrap();
    let layout = frame.layout.as_ref().unwrap();
    let figure = find(layout, &format!("mdout/{}", ids[0])).expect("tool Markdown panel");
    assert!(
        markdown_canvas(figure),
        "Mermaid must paint a native diagram canvas"
    );
    let prose = frame.item_text(ids[0]).unwrap();
    assert!(prose.contains("Drawing") && prose.contains("Source"));
    assert!(!prose.contains("graph LR"));
    assert!(find(layout, &format!("mdout/{}", ids[1])).is_none());
    assert!(
        frame.item_text(ids[1]).unwrap().contains("graph LR"),
        "execute output must stay literal mono"
    );
    assert!(
        find(layout, &format!("mdout/{}", ids[2])).is_none(),
        "failed Read stays raw"
    );
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn markdown_output_reasoning_is_dim_prose_and_live_text_is_literal() {
    use bboxy450_harness::frontends::gui::{rich::Toggle, view::find};
    let text = "# Reasoned heading\n\n- first option\n- second option\n\n```mermaid\ngraph LR\nA --> B\n```";
    let temp = tempfile::tempdir().unwrap();
    let (path, ids) = markdown_replay(
        temp.path(),
        vec![
            (
                Item::Reasoning {
                    summary: text.into(),
                    raw: None,
                    provider_opaque: None,
                },
                Some(ItemStatus::Completed),
            ),
            (
                Item::Reasoning {
                    summary: "**still thinking**".into(),
                    raw: None,
                    provider_opaque: None,
                },
                Some(ItemStatus::Completed),
            ),
        ],
    );
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay(path);
    ready(&mut bridge).await;
    let parsed = bridge
        .state
        .prepared
        .as_ref()
        .unwrap()
        .parsed_markdown_bytes;
    bridge.toggle(ids[0], Toggle::Reasoning);
    until(&mut bridge, |b| {
        b.state.faces.get(&ids[0]).is_some_and(|f| f.reasoning_open)
    })
    .await;
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [900., 1800.]).unwrap();
    let body = find(
        frame.layout.as_ref().unwrap(),
        &format!("reasoning/{}/body", ids[0]),
    )
    .unwrap();
    assert!(markdown_canvas(body));
    fn dim_prose(node: &bbt_ui::components::Layout) -> bool {
        if let bbt_ui::components::LayoutKind::Text { runs, .. } = &node.kind
            && runs.iter().any(|r| {
                r.text.contains("Reasoned")
                    && (r.style.size - 12.5).abs() < 0.01
                    && r.style.color == bbt_ui_model::draw::Color::hex("#8A90B0").0
            })
        {
            return true;
        }
        node.children.iter().any(dim_prose)
    }
    assert!(dim_prose(body));
    assert_eq!(
        bridge
            .state
            .prepared
            .as_ref()
            .unwrap()
            .parsed_markdown_bytes,
        parsed,
        "opening reasoning must reuse prepared blocks"
    );
    let mut model = (*bridge.state.transcript).clone();
    let live = model.items.get_mut(&ids[1]).unwrap();
    live.status = None;
    live.completed = None;
    let source = Arc::new(model);
    let prepared = runtime::blocking({
        let source = source.clone();
        move || {
            bboxy450_harness::frontends::gui::view::Preparer::default()
                .prepare(source, 900.)
                .unwrap()
        }
    })
    .await;
    let state = bboxy450_harness::frontends::gui::bridge::FrameState {
        transcript: source,
        prepared: Some(Arc::new(prepared)),
        ..Default::default()
    };
    assert!(
        view.render(&state, [900., 1800.])
            .unwrap()
            .item_text(ids[1])
            .unwrap()
            .contains("**still thinking** ▍")
    );
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn large_markdown_uses_the_production_replay_preparation_worker() {
    bbt_rt::register_ui_thread();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        bboxy450_harness::frontends::gui::view::Preparer::default().prepare(Arc::default(), 1120.)
    }));
    assert!(
        result.is_err(),
        "UI ownership assertion must reject preparation"
    );
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
    let mut popover = input.popover(&[]).unwrap();
    let edit = popover
        .handle(bbt_ui::modules::approval::Key::Enter)
        .unwrap();
    input.handle(Command::Edit(edit));
    assert_eq!(input.view.text, "/interrupt");
    input.handle(Command::Edit(Edit::SelectAll));
    input.handle(Command::Edit(Edit::Insert("@file".into())));
    assert_eq!(input.completion().unwrap().trigger, '@');
    assert!(input.popover(&[]).is_none());
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
                // the terminal folds into its tool card, which reads as running: the
                // verb, and the three seat dots in the status slot (harness-spec.md
                // "Tool card": run = three bouncing seat dots)
                fn dots(n: &bbt_ui::components::Layout) -> usize {
                    usize::from(matches!(
                        n.kind,
                        bbt_ui::components::LayoutKind::Path { .. }
                    )) + n.children.iter().map(dots).sum::<usize>()
                }
                let status = frame.layout.as_ref().and_then(|l| {
                    bboxy450_harness::frontends::gui::view::find(l, &format!("tool/{tool}/status"))
                });
                if text.contains("LIVE-TAIL")
                    && frame.item_text(tool).is_some_and(|s| s.contains("Run"))
                    && status.is_some_and(|n| dots(n) == 3)
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
async fn closing_an_old_gateway_does_not_close_a_reopened_actor_in_the_same_store() {
    use bboxy450_harness::{
        engine::{Config, SessionActor},
        frontends::gui::view::Action,
        gateway::{Gateway, Refused},
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
    };
    let root = tempfile::tempdir().unwrap();
    let meta = support::meta(root.path().into());
    let config = Config {
        meta,
        profiles: root.path().join("profiles"),
        sessions: root.path().join("sessions"),
        provider: Arc::new(ScriptedProvider::new(vec![
            support::sse("text"),
            support::sse("text"),
        ])),
        model: "scripted".into(),
        policy: Policy::allow_all(Unattended::Deny),
        redactor: Default::default(),
        profile_error: None,
        max_rounds: 50,
    };
    let start = OpId::new();
    let (old, old_task) = SessionActor::open(config.clone(), start).await.unwrap();
    let mut attached = Bridge::new(Arc::new(Wake));
    attached.attach(old.clone());
    ready(&mut attached).await;
    let old_birth = old.hello().epoch;
    let mut view = HarnessView::default();
    // the header has no Stop; `/stop` from the composer is fenced to the rendered birth
    view.render(&attached.state, [1120., 860.]).unwrap();
    view.composer
        .handle(bbt_ui::modules::composer::Command::Edit(
            bbt_ui::text_edit::Command::Insert("/stop".into()),
        ));
    attached.detach().unwrap().await.unwrap();
    // Crash/recovery of the SAME session directory, not two disjoint identities.
    old_task.abort();
    assert!(old_task.await.unwrap_err().is_cancelled());
    let (new, new_task) = SessionActor::open(config, start).await.unwrap();
    assert_eq!(old.session(), new.session());
    assert_ne!(old_birth, new.hello().epoch);
    let mut attached = Bridge::new(Arc::new(Wake));
    attached.attach(new.clone());
    ready(&mut attached).await;
    let mut events = support::subscribe(&new, false);
    new.send(support::prompt(&new, "keep running")).unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        while !matches!(
            events.recv().await.unwrap().event,
            Event::TurnStarted { .. }
        ) {}
    })
    .await
    .unwrap();
    assert!(matches!(
        old.send(support::op(&old, Op::Close)),
        Err(Refused::Closed)
    ));
    // A delayed rendered Close admitted by the old attachment must not be
    // delivered to the replacement merely because the public session matches.
    view.activate(Action::Send, &attached);
    let stale_notice = view.notice.clone();
    let done = support::end(&mut events).await;
    assert!(!done.iter().any(|e| matches!(e.event, Event::SessionClosed)));
    assert!(done.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    new.send(support::prompt(&new, "still alive")).unwrap();
    let again = support::end(&mut events).await;
    assert!(again.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert_eq!(stale_notice.as_deref(), Some("Session replaced"));
    attached.detach().unwrap().await.unwrap();
    new.send(support::op(&new, Op::Close)).unwrap();
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
            .is_some_and(|f| f.reasoning_open)
    })
    .await;
    // collapsed by default; the disclosure opens the full reasoning body
    let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
    let text = frame.item_text(reasoning).unwrap();
    assert!(text.contains("Compare the current contents"));
    assert!(
        bboxy450_harness::frontends::gui::view::find(
            frame.layout.as_ref().unwrap(),
            &format!("reasoning/{reasoning}/body")
        )
        .is_some()
    );
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
            .is_some_and(|f| !f.reasoning_open)
    })
    .await;
    let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
    assert!(
        bboxy450_harness::frontends::gui::view::find(
            frame.layout.as_ref().unwrap(),
            &format!("reasoning/{reasoning}/body")
        )
        .is_none()
    );
    // collapsed again: the gist is the row (a thought under 1s carries no "Thought" label,
    // harness-spec.md "Tight transcript")
    assert!(
        frame
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
            #[cfg(feature = "standalone")]
            bboxy450_harness::frontends::gui::window::route_key(
                &mut view,
                &bridge,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                None,
                winit::keyboard::ModifiersState::empty(),
            );
            #[cfg(not(feature = "standalone"))]
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
    let mut old = view.composer.popover(&[]).unwrap();
    let stale = old.handle(bbt_ui::modules::approval::Key::Enter).unwrap();
    view.edit(Edit::SelectAll);
    view.edit(Edit::Insert("/co".into()));
    view.composer.handle(Command::Edit(stale));
    assert_eq!(view.composer.view.text, "/co");
}

async fn streaming_model() -> (bboxy450_harness::frontends::gui::model::Transcript, ItemId) {
    let mut model = bboxy450_harness::frontends::gui::bridge::read_replay(
        "tests/fixtures/protocol/all-items.jsonl".into(),
    )
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
    (model, id)
}
#[tokio::test]
async fn structured_64k_markdown_measures_toolkit_incrementality_without_throttling() {
    let (mut model, id) = streaming_model().await;
    let (parsed, received, appends, deferred, blocks) = runtime::blocking(move || {
        let unit = "A paragraph with **bold**, a [link](https://example.invalid) and words.\n\n- first list entry\n- second list entry\n\n```rust\nfn example() { let value = 7; }\n```\n\n";
        let mut text = unit.repeat(65536 / unit.len() + 1); text.truncate(65536);
        let mut p = bboxy450_harness::frontends::gui::view::Preparer::default(); let mut result = None;
        for end in (1024..=65536).step_by(1024) {
            model.items.get_mut(&id).unwrap().body = Item::AgentMessage { markdown: text[..end].into() };
            result = Some(p.prepare(Arc::new(model.clone()), 1060.).unwrap());
        }
        let r = result.unwrap(); (r.parsed_markdown_bytes, r.received_markdown_bytes, r.markdown_appends, r.deferred_markdown_batches, r.markdown_block_layouts)
    }).await;
    assert_eq!(received, 65536);
    assert_eq!(
        appends, 64,
        "every batch must reach StreamingMarkdown.append"
    );
    assert_eq!(deferred, 0, "credit must not mask non-incremental parsing");
    assert!(
        parsed <= received * 4,
        "toolkit reparsed {parsed} bytes for {received}"
    );
    assert!(
        blocks < 4096,
        "stable blocks must be reused, built {blocks}"
    );
    println!(
        "structured Markdown: {parsed} parsed / {received} received, {appends} appends, {deferred} deferred, {blocks} block layouts"
    );
}
#[tokio::test]
async fn deferred_tail_is_literal_preview_then_formatted_on_completion() {
    let (mut model, id) = streaming_model().await;
    runtime::blocking(move || {
        let mut p = bboxy450_harness::frontends::gui::view::Preparer::default();
        let mut preview = None;
        let mut text = String::new();
        for i in 1..32 {
            text.push_str(&format!("{}**PREVIEW_{i}**", "unfinished ".repeat(40)));
            model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
                markdown: text.clone(),
            };
            let r = p.prepare(Arc::new(model.clone()), 1060.).unwrap();
            if r.deferred_markdown_batches > 0 {
                preview = Some((i, r));
                break;
            }
        }
        let (i, prepared) = preview.expect("unfinished block must exhaust its parse credit");
        let mut state = bboxy450_harness::frontends::gui::bridge::FrameState {
            transcript: prepared.source.clone(),
            ready: true,
            replay: true,
            prepared: Some(Arc::new(prepared)),
            ..Default::default()
        };
        let mut view = HarnessView::default();
        assert!(
            view.render(&state, [1120., 7000.])
                .unwrap()
                .item_text(id)
                .unwrap()
                .contains(&format!("**PREVIEW_{i}**")),
            "deferred suffix must be an honest literal preview"
        );
        model.items.get_mut(&id).unwrap().completed = Some(Seq(999));
        model.items.get_mut(&id).unwrap().status = Some(ItemStatus::Completed);
        state.prepared = Some(Arc::new(p.prepare(Arc::new(model.clone()), 1060.).unwrap()));
        state.transcript = state.prepared.as_ref().unwrap().source.clone();
        let frame = view.render(&state, [1120., 7000.]).unwrap();
        let rendered = frame.item_text(id).unwrap();
        assert!(rendered.contains(&format!("PREVIEW_{i}")));
        assert!(
            !rendered.contains("**"),
            "completion must flush Markdown formatting"
        );
    })
    .await;
}
#[cfg(feature = "standalone")]
#[tokio::test]
async fn escape_key_routing_closes_completion_before_interrupting_the_live_turn() {
    use bboxy450_harness::frontends::gui::{view::Action, window::route_key};
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    let mut provider =
        bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    provider.delay = Duration::from_secs(3600);
    let session = support::with_provider(Arc::new(provider), true).await;
    let mut events = support::subscribe(&session.gateway, false);
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.attach(session.gateway.clone());
    ready(&mut bridge).await;
    let mut view = HarnessView::default();
    view.edit(bbt_ui::text_edit::Command::Insert("/int".into()));
    assert!(view.completion_open());
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Escape),
        None,
        ModifiersState::empty(),
    );
    assert!(!view.completion_open());
    assert_eq!(view.composer.view.text, "/int");
    assert!(view.notice.is_none());
    view.edit(bbt_ui::text_edit::Command::SelectAll);
    view.edit(bbt_ui::text_edit::Command::Insert("work".into()));
    view.activate(Action::Send, &bridge);
    tokio::time::timeout(Duration::from_secs(8), async {
        while !matches!(
            events.recv().await.unwrap().event,
            Event::TurnStarted { .. }
        ) {}
    })
    .await
    .unwrap();
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Escape),
        None,
        ModifiersState::empty(),
    );
    let done = support::end(&mut events).await;
    assert!(done.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    )));
    bridge.detach().unwrap().await.unwrap();
    support::close(session).await;
}

#[tokio::test]
async fn definition_lines_in_containers_repair_previously_formatted_blocks() {
    let (mut model, id) = streaming_model().await;
    runtime::blocking(move || {
        for (prefix, definition) in [
            (
                "[reference][r]\n\nA stable paragraph.\n\nA trailing paragraph.\n\n",
                "> [r]: https://example.invalid\n\n",
            ),
            (
                "[reference][r]\n\nA stable paragraph.\n\nA trailing paragraph.\n\n",
                "- [r]: https://example.invalid\n\n",
            ),
            (
                "[reference][foo bar]\n\nA stable paragraph.\n\nA trailing paragraph.\n\n",
                "[foo\nbar]: https://example.invalid\n\n",
            ),
            (
                "A note[^f].\n\nA stable paragraph.\n\nA trailing paragraph.\n\n",
                "- [^f]: A footnote.\n\n",
            ),
        ] {
            model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
                markdown: prefix.into(),
            };
            let mut stream = bboxy450_harness::frontends::gui::view::Preparer::default();
            stream.prepare(Arc::new(model.clone()), 1120.).unwrap();
            model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
                markdown: format!("{prefix}{definition}"),
            };
            let source = Arc::new(model.clone());
            let live = stream.prepare(source.clone(), 1120.).unwrap();
            let full = bboxy450_harness::frontends::gui::view::Preparer::default()
                .prepare(source.clone(), 1120.)
                .unwrap();
            let render = |prepared| {
                let state = bboxy450_harness::frontends::gui::bridge::FrameState {
                    transcript: source.clone(),
                    ready: true,
                    replay: true,
                    prepared: Some(Arc::new(prepared)),
                    ..Default::default()
                };
                let mut view = HarnessView::default();
                let frame = view.render(&state, [1120., 7000.]).unwrap();
                bboxy450_harness::frontends::gui::view::layout_text(
                    bboxy450_harness::frontends::gui::view::find(
                        frame.layout.as_ref().unwrap(),
                        &format!("body/{id}"),
                    )
                    .unwrap(),
                )
            };
            assert_eq!(
                render(live),
                render(full),
                "definition {definition:?} must invalidate the earlier formatted prefix"
            );
        }
    })
    .await;
}

#[tokio::test]
async fn context_trim_is_a_prepared_rich_callout_with_its_actual_frontier() {
    use bboxy450_harness::frontends::gui::{
        rich::Toggle,
        view::{Action, find, layout_text},
    };
    let events = include_str!("fixtures/gui/rich-all-items.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<Envelope>(line).unwrap())
        .collect::<Vec<_>>();
    bboxy450_harness::protocol::laws::check_transcript(&events).unwrap();
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.replay("tests/fixtures/gui/rich-all-items.jsonl".into());
    ready(&mut bridge).await;
    let row = bridge
        .state
        .transcript
        .items
        .values()
        .find(|row| row.kind == ItemKind::ContextTrim)
        .expect("fixture must exercise the new kind");
    let Item::ContextTrim { clipped_through } = &row.body else {
        panic!("typed trim body missing")
    };
    let id = row.id;
    let frontier = clipped_through.0;
    let mut view = HarnessView::default();
    let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
    let body = find(frame.layout.as_ref().unwrap(), &format!("body/{id}")).unwrap();
    let text = layout_text(body);
    assert!(text.contains(&format!("Context trimmed through event {frontier}")));
    assert!(text.contains("2 KiB") && text.contains("model context"));
    assert!(text.contains("Tool output limited") && !text.contains("terminal output"));
    fn emphasized(node: &bbt_ui::components::Layout) -> bool {
        if let bbt_ui::components::LayoutKind::Text { runs, .. } = &node.kind
            && runs.iter().any(|r| r.style.weight >= 600)
        {
            return true;
        }
        node.children.iter().any(emphasized)
    }
    assert!(
        emphasized(body),
        "the trim callout must have a styled heading/emphasis"
    );
    assert!(
        !frame
            .actions
            .iter()
            .any(|(_, a)| matches!(a,Action::Toggle(item,Toggle::Source) if *item==id)),
        "generated status copy has no source disclosure"
    );
    bridge.detach().unwrap().await.unwrap();
}

#[test]
fn profile_info_carries_kind_model_and_agent_never_a_key() {
    use bboxy450_harness::frontends::gui::bridge::{ProfileInfo, ProfileKind};
    let model = ProfileInfo::parse(
        "opus",
        "---\nname: opus\nprovider: anthropic\nmodel: claude-opus-5-5\napi_key: fixture-secret-value\n---\n",
    );
    assert_eq!(model.kind, ProfileKind::Model);
    assert_eq!(model.model, "claude-opus-5-5");
    assert!(!format!("{model:?}").contains("fixture-secret-value"));
    let agent = ProfileInfo::parse(
        "claude-acp",
        "---\nname: claude-acp\nprovider: acp-agent\nagent:\n  command: [/opt/bin/claude-agent-acp, --flag]\n  env:\n    TOKEN: fixture-env-value\n---\n",
    );
    assert_eq!(agent.kind, ProfileKind::Agent);
    assert_eq!(agent.agent, ["claude-agent-acp", "--flag"]);
    assert!(!format!("{agent:?}").contains("fixture-env-value"));
    assert_eq!(
        ProfileInfo::parse("broken", "no frontmatter").kind,
        ProfileKind::Unreadable
    );
    // the key source kind only, and env presence, never a value
    use bboxy450_harness::frontends::gui::bridge::ProfileAuth;
    let source = |line: &str| {
        ProfileInfo::parse(
            "m",
            &format!("---\nname: m\nmodel: x\nendpoint: https://example.invalid/v1\n{line}\n---\n"),
        )
    };
    let keychain = source("api_key: '!security find-generic-password -s fixture -w'");
    assert_eq!(
        (keychain.auth, keychain.auth_ok),
        (ProfileAuth::Keychain, true)
    );
    let missing = source("api_key_env: BBOXY450_TEST_NEVER_SET_KEY");
    assert_eq!((missing.auth, missing.auth_ok), (ProfileAuth::Env, false));
    let with_env = ProfileInfo::parse(
        "pi",
        "---\nname: pi\nprovider: acp-agent\nagent:\n  command: [pi-acp]\n  env:\n    A: x\n    B: y\n---\n",
    );
    assert_eq!(with_env.agent_env, 2);
}

fn switcher_state() -> bboxy450_harness::frontends::gui::bridge::FrameState {
    use bboxy450_harness::frontends::gui::{bridge::ProfileInfo, model::Transcript};
    let profile = |name: &str, text: &str| ProfileInfo::parse(name, text);
    bboxy450_harness::frontends::gui::bridge::FrameState {
        transcript: Arc::new(Transcript {
            profile: Some(("haiku".into(), "claude-haiku-5-5".into())),
            ..Default::default()
        }),
        ready: true,
        profiles: Arc::new(vec![
            profile(
                "codex-acp",
                "---\nname: codex-acp\nprovider: acp-agent\nagent:\n  command: [codex-acp]\n---\n",
            ),
            profile(
                "haiku",
                "---\nname: haiku\nprovider: anthropic\nmodel: claude-haiku-5-5\n---\n",
            ),
            profile(
                "opus",
                "---\nname: opus\nprovider: anthropic\nmodel: claude-opus-5-5\n---\n",
            ),
        ]),
        ..Default::default()
    }
}

#[test]
fn header_chip_opens_the_profile_switcher_and_a_row_switches_like_slash_profile() {
    use bboxy450_harness::frontends::gui::view::{Action, MenuKey};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state = switcher_state();
    let mut view = HarnessView::default();
    let state = bridge.state.clone();
    let chip = view
        .render(&state, [900., 700.])
        .unwrap()
        .actions
        .iter()
        .find(|(_, a)| *a == Action::Profiles)
        .map(|(r, _)| *r)
        .expect("the header profile chip is clickable");
    assert!(chip.w > 0. && chip.y < 40.);
    view.activate(Action::Profiles, &bridge);
    assert!(view.profile_menu_open());
    let rows: Vec<String> = view
        .render(&state, [900., 700.])
        .unwrap()
        .actions
        .iter()
        .filter_map(|(_, a)| match a {
            Action::SwitchProfile(name) => Some(name.clone()),
            _ => None,
        })
        .collect();
    // ACP agents first, then models (actions are hit-tested front to back)
    let mut shown = rows.clone();
    shown.reverse();
    assert_eq!(shown, ["codex-acp", "haiku", "opus"]);
    // typing filters; Enter on a replay has no live host: same refusal as /profile NAME
    view.profile_menu_key(MenuKey::Text("opu".into()), &bridge);
    view.profile_menu_key(MenuKey::Enter, &bridge);
    assert!(!view.profile_menu_open());
    assert_eq!(
        view.notice.as_deref(),
        Some("Could not switch: no live session to switch")
    );
    // choosing the current profile does not fork the session
    view.activate(Action::SwitchProfile("haiku".into()), &bridge);
    assert_eq!(view.notice.as_deref(), Some("Already on haiku"));
    // a click outside the card closes it
    view.activate(Action::Profiles, &bridge);
    view.activate(Action::Menu(false), &bridge);
    assert!(!view.profile_menu_open());
}

#[test]
fn slash_popover_is_a_menu_card_with_details_and_no_debug_keys() {
    use bboxy450_harness::frontends::gui::{face, menu, view::layout_text};
    let state = switcher_state();
    let mut view = HarnessView::default();
    view.render(&state, [900., 700.]).unwrap();
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    assert!(view.completion_open());
    let frame = view.render(&state, [900., 700.]).unwrap();
    let hits: Vec<_> = frame
        .actions
        .iter()
        .filter(|(_, a)| {
            matches!(
                a,
                bboxy450_harness::frontends::gui::view::Action::Complete(..)
            )
        })
        .collect();
    assert_eq!(hits.len(), 6, "three commands and three profiles");
    // anchored above the composer, below the header
    assert!(
        hits.iter()
            .all(|(r, _)| r.y > 40. && r.bottom() < 700. - 60.)
    );
    let ctx = bboxy450_harness::frontends::gui::view::context();
    let mut popover = view.composer.popover(&["opus".into()]).unwrap();
    popover.handle(bbt_ui::modules::approval::Key::Down);
    let element = menu::slash_menu(
        &popover,
        &state.profiles,
        10,
        &face::ink(ctx),
        &menu::Look::settled(menu::slash_selected_key(&popover)),
    );
    let metrics = bboxy450_harness::frontends::gui::view::Metrics::default();
    let layout = element
        .layout(
            bbt_ui_model::layout::Rect::new(0., 0., 440., 0.),
            ctx,
            &metrics,
        )
        .unwrap();
    let text = layout_text(&layout);
    assert!(text.contains("/interrupt Stop the running turn"), "{text}");
    assert!(text.contains("/profile opus claude-opus-5-5"), "{text}");
    assert!(text.contains("COMMANDS"), "{text}");
    assert!(text.contains("SWITCH PROFILE"), "{text}");
    assert!(text.contains("select ⏎ run esc close 4 matches"), "{text}");
    assert!(!text.contains("completion/"), "{text}");
}

#[cfg(feature = "standalone")]
#[test]
fn named_space_key_preserves_the_composer_draft() {
    let bridge = Bridge::new(Arc::new(Wake));
    let mut view = HarnessView::default();
    dispatch_text_keys(&mut view, &bridge, "a b");
    assert_eq!(view.composer.view.text, "a b");
}

#[cfg(feature = "standalone")]
#[test]
fn named_space_key_preserves_the_profile_search() {
    use bbt_ui_model::draw::Prim;
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state = switcher_state();
    let mut view = HarnessView::default();
    view.render(&bridge.state, [900., 700.]).unwrap();
    view.toggle_profile_menu();
    dispatch_text_keys(&mut view, &bridge, "a b");
    let frame = view.render(&bridge.state, [900., 700.]).unwrap();
    // the switcher is painted over the face (Frame::over)
    let painted: Vec<_> = frame
        .list
        .primitives
        .iter()
        .chain(&frame.over.primitives)
        .filter_map(|p| match p {
            Prim::Text { text, .. } => Some(text.as_ref()),
            _ => None,
        })
        .collect();
    assert!(painted.contains(&"a b"), "profile search: {painted:?}");
    assert!(painted.contains(&"No profile matches “a b”"));
    assert!(view.composer.view.text.is_empty());
    assert!(view.profile_menu_open());
}

#[cfg(feature = "standalone")]
#[test]
fn named_space_key_preserves_slash_profile_input() {
    use bboxy450_harness::frontends::gui::{view::Action, window::route_key};
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state = switcher_state();
    let mut view = HarnessView::default();
    view.render(&bridge.state, [900., 700.]).unwrap();
    dispatch_text_keys(&mut view, &bridge, "/profile");
    assert!(view.completion_open());
    let frame = view.render(&bridge.state, [900., 700.]).unwrap();
    assert!(frame.actions.iter().any(|(_, action)| matches!(
        action, Action::Complete(_, value) if value == "/profile opus"
    )));
    // Whitespace ends the completion token; arguments remain ordinary draft text.
    dispatch_text_keys(&mut view, &bridge, " opus");
    assert_eq!(view.composer.view.text, "/profile opus");
    assert!(!view.completion_open());
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Enter),
        None,
        ModifiersState::empty(),
    );
    assert_eq!(
        view.notice.as_deref(),
        Some("Could not switch: no live session to switch")
    );
}

#[cfg(feature = "standalone")]
fn dispatch_text_keys(view: &mut HarnessView, bridge: &Bridge, text: &str) {
    use bboxy450_harness::frontends::gui::window::route_key;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    for ch in text.chars() {
        let text = ch.to_string();
        let key = if ch == ' ' {
            Key::Named(NamedKey::Space)
        } else {
            Key::Character(text.clone().into())
        };
        route_key(view, bridge, key, Some(&text), ModifiersState::empty());
    }
}

#[cfg(feature = "standalone")]
#[test]
fn named_space_key_keeps_text_and_shortcut_and_composition_guards() {
    use bboxy450_harness::frontends::gui::window::route_key;
    use bbt_ui::text_edit::Command as Edit;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    let bridge = Bridge::new(Arc::new(Wake));
    let mut view = HarnessView::default();
    for menu in [false, true] {
        if menu {
            view.toggle_profile_menu();
        }
        for (text, modifiers) in [
            (Some(" "), ModifiersState::CONTROL),
            (Some(" "), ModifiersState::SUPER),
            (None, ModifiersState::empty()),
            (Some("\t"), ModifiersState::empty()),
        ] {
            route_key(
                &mut view,
                &bridge,
                Key::Named(NamedKey::Space),
                text,
                modifiers,
            );
        }
        assert!(view.composer.view.text.is_empty());
        if menu {
            // An unchanged query still shows the search placeholder.
            let frame = view.render(&bridge.state, [900., 700.]).unwrap();
            assert!(frame.over.primitives.iter().any(|p| matches!(
                p, bbt_ui_model::draw::Prim::Text { text, .. } if text.as_ref() == "Switch profile…"
            )));
            view.toggle_profile_menu();
        }
    }
    view.edit(Edit::Preedit {
        text: "preedit".into(),
        caret: Some(7),
    });
    assert!(view.composer.composing());
    let draft = view.composer.view.text.clone();
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Space),
        Some(" "),
        ModifiersState::empty(),
    );
    assert_eq!(view.composer.view.text, draft);
    assert!(view.composer.composing());
}

#[cfg(feature = "standalone")]
#[test]
fn profile_switcher_keys_filter_move_and_escape_before_the_composer() {
    use bboxy450_harness::frontends::gui::window::route_key;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state = switcher_state();
    let mut view = HarnessView::default();
    view.render(&bridge.state.clone(), [900., 700.]).unwrap();
    view.toggle_profile_menu();
    let none = ModifiersState::empty();
    route_key(
        &mut view,
        &bridge,
        Key::Character("x".into()),
        Some("x"),
        none,
    );
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::ArrowDown),
        None,
        none,
    );
    assert!(
        view.composer.view.text.is_empty(),
        "menu keys never reach the composer"
    );
    route_key(&mut view, &bridge, Key::Named(NamedKey::Escape), None, none);
    assert!(!view.profile_menu_open());
    assert!(
        view.notice.is_none(),
        "escape closed the menu, it did not interrupt"
    );
}

#[cfg(feature = "standalone")]
#[test]
fn enter_on_the_slash_menu_runs_the_command() {
    use bboxy450_harness::frontends::gui::window::route_key;
    use winit::keyboard::{Key, ModifiersState, NamedKey};
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.state = switcher_state();
    let mut view = HarnessView::default();
    view.render(&bridge.state.clone(), [900., 700.]).unwrap();
    // "/profiles" runs: it opens the switcher, as the web menu does
    view.edit(bbt_ui::text_edit::Command::Insert("/profiles".into()));
    assert!(view.completion_open());
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Enter),
        None,
        ModifiersState::empty(),
    );
    assert!(view.profile_menu_open());
    assert!(
        view.composer.view.text.is_empty(),
        "the command was sent, not left as text"
    );
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Escape),
        None,
        ModifiersState::empty(),
    );
    // "/profile opus" runs the switch: on a replay that is the /profile refusal
    view.edit(bbt_ui::text_edit::Command::Insert("/prof".into()));
    for _ in 0..3 {
        route_key(
            &mut view,
            &bridge,
            Key::Named(NamedKey::ArrowDown),
            None,
            ModifiersState::empty(),
        );
    }
    route_key(
        &mut view,
        &bridge,
        Key::Named(NamedKey::Enter),
        None,
        ModifiersState::empty(),
    );
    assert_eq!(
        view.notice.as_deref(),
        Some("Could not switch: no live session to switch")
    );
}

#[tokio::test]
async fn gui_polish_mcp_cards_show_real_raw_results_on_expansion() {
    use bboxy450_harness::frontends::gui::{
        bridge::FrameState,
        rich::{Face, Faces},
        view::{Preparer, layout_text},
    };
    let temp = tempfile::tempdir().unwrap();
    let make = |raw: serde_json::Value, output: Vec<OutputBlock>| Item::ToolCall {
        call_id: ItemId::new().to_string(),
        name: "mcp__fixture__result".into(),
        tool_kind: ToolKind::Other,
        title: Some("MCP fixture".into()),
        server: Some("fixture".into()),
        input_raw: "{}".into(),
        output: if output.is_empty() {
            vec![OutputBlock::Json { value: raw.clone() }]
        } else {
            output
        },
        raw_output: Some(raw),
        locations: vec![],
        spill: None,
    };
    let (path, ids) = markdown_replay(
        temp.path(),
        vec![
            (
                make(
                    serde_json::json!({"content":[{"type":"text","text":"# Real MCP result\n- first\n- second"}], "structuredContent":{"count":2}}),
                    vec![OutputBlock::Text {
                        text: "(no output)".into(),
                    }],
                ),
                Some(ItemStatus::Completed),
            ),
            (
                make(
                    serde_json::json!({"structuredContent":{"phase":"ready","nested":{"count":3}}}),
                    vec![],
                ),
                Some(ItemStatus::Completed),
            ),
            (
                make(
                    serde_json::json!({"content":[{"type":"image","mimeType":"image/png","data":"private-binary"},{"type":"resource_link","uri":"https://example.test/result","name":"result"}]}),
                    vec![],
                ),
                Some(ItemStatus::Completed),
            ),
            (
                make(
                    serde_json::json!({"content":[{"type":"text","text":(0..120).map(|i|format!("row {i}: full available MCP output\n")).collect::<String>() + "END-OF-AVAILABLE-RESULT"}]}),
                    vec![],
                ),
                Some(ItemStatus::Completed),
            ),
        ],
    );
    if let Some(directory) = std::env::var_os("GUI_POLISH_CAPTURE_DIR") {
        use bboxy450_harness::frontends::gui::{options::Options, smoke::capture};
        let screenshot = std::path::PathBuf::from(directory).join("mcp-output.png");
        let options = Options::parse(
            [
                "--replay",
                path.to_str().unwrap(),
                "--capture",
                "900x1200",
                "--expand",
                "all",
                "--screenshot",
                screenshot.to_str().unwrap(),
            ]
            .into_iter()
            .map(String::from),
        )
        .unwrap();
        capture(options).await.unwrap();
    }
    let source = Arc::new(
        bboxy450_harness::frontends::gui::bridge::read_replay(path)
            .await
            .unwrap(),
    );
    runtime::blocking(move || {
        let mut preparer = Preparer::default();
        let mut state = FrameState {
            transcript: source,
            ready: true,
            replay: true,
            ..Default::default()
        };
        state.prepared = Some(Arc::new(
            preparer.prepare(state.transcript.clone(), 900.).unwrap(),
        ));
        let mut view = HarnessView::default();
        let collapsed = layout_text(
            view.render(&state, [900., 2400.])
                .unwrap()
                .layout
                .as_ref()
                .unwrap(),
        );
        assert!(!collapsed.contains("Real MCP result"));
        assert!(collapsed.contains("lines"));
        let long_id = *ids.last().unwrap();
        let faces: Faces = ids
            .into_iter()
            .map(|id| {
                (
                    id,
                    Face {
                        output_closed: true,
                        ..Default::default()
                    },
                )
            })
            .collect();
        state.prepared = Some(Arc::new(
            preparer
                .prepare_with_faces(state.transcript.clone(), 900., &faces)
                .unwrap(),
        ));
        state.faces = Arc::new(faces);
        let expanded = layout_text(
            view.render(&state, [900., 2400.])
                .unwrap()
                .layout
                .as_ref()
                .unwrap(),
        );
        for needle in [
            "Real MCP result",
            "first",
            "count",
            "ready",
            "https://example.test/result",
            "image/png",
        ] {
            assert!(expanded.contains(needle), "missing {needle}: {expanded}");
        }
        assert!(!expanded.contains("(no output)"));
        assert!(!expanded.contains("private-binary"));
        let frame = view.render(&state, [900., 2400.]).unwrap();
        let panel = bboxy450_harness::frontends::gui::view::find(
            frame.layout.as_ref().unwrap(),
            &format!("mdout/{long_id}"),
        )
        .unwrap();
        let scroll = &panel.children[0];
        assert!(scroll.rect.h <= 400.);
        let at = [scroll.rect.x + 10., scroll.rect.y + 10.];
        assert!(view.wheel(at, 100_000.));
        assert!(
            view.render(&state, [900., 2400.])
                .unwrap()
                .item_text(long_id)
                .unwrap()
                .contains("END-OF-AVAILABLE-RESULT"),
            "expansion must retain the result beyond 1600 characters"
        );
    })
    .await;
}

#[tokio::test]
async fn acp_controls_tool_cards_quiet_code_bound_long_commands_and_keep_full_payload_scrollable() {
    use bboxy450_harness::frontends::gui::{
        bridge::{FrameState, read_replay},
        rich::{Face, Faces},
        view::{Preparer, find, layout_text},
    };
    let temp = tempfile::tempdir().unwrap();
    let commands = [
        format!("printf '%s' '{}'", "single-token-".repeat(120)),
        format!(
            "python3 - <<'PY'\n{}\nprint('COMMAND-END')\nPY",
            (0..90)
                .map(|n| format!("print('line-{n}:{}')", "payload".repeat(20)))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    ];
    let output = (0..100)
        .map(|n| format!("output-{n}:{}", "long-token-".repeat(40)))
        .chain(std::iter::once("OUTPUT-END".into()))
        .collect::<Vec<_>>()
        .join("\n");
    let items=commands.iter().enumerate().map(|(index,command)|{
        let body:Item=serde_json::from_value(serde_json::json!({"kind":"tool_call","call_id":format!("bounds-{index}"),"name":"bash","tool_kind":"execute","title":command,"input_raw":serde_json::json!({"command":command}).to_string(),"output":[{"type":"text","text":output}],"locations":[],"spill":null})).unwrap();
        (body,Some(ItemStatus::Failed{reason:"fixture".into(),code:None}))
    }).collect();
    let (path, ids) = markdown_replay(temp.path(), items);
    let source = Arc::new(read_replay(path).await.unwrap());
    runtime::blocking(move || {
        for width in [280., 1120.] {
            let mut preparer = Preparer::default();
            let faces: Faces = ids
                .iter()
                .map(|id| {
                    (
                        *id,
                        Face {
                            arguments_closed: true,
                            ..Default::default()
                        },
                    )
                })
                .collect();
            let mut state = FrameState {
                transcript: source.clone(),
                ready: true,
                replay: true,
                faces: Arc::new(faces.clone()),
                ..Default::default()
            };
            state.prepared = Some(Arc::new(
                preparer
                    .prepare_with_faces(source.clone(), width, &faces)
                    .unwrap(),
            ));
            let mut view = HarnessView::default();
            view.scroll.follow_latest();
            let frame = view.render(&state, [width, 1800.]).unwrap();
            let layout = frame.layout.as_ref().unwrap();
            for id in &ids {
                let card = find(layout, &format!("tool/{id}")).unwrap();
                assert!(
                    card.rect.x >= 0. && card.rect.right() <= width,
                    "card escaped pane {width}: {:?}",
                    card.rect
                );
                let head = find(card, &format!("toolhead/{id}")).unwrap();
                assert!(
                    head.rect.h < 80.,
                    "command created an oversized header: {:?}",
                    head.rect
                );
                let command = find(card, &format!("codeout/{id}/command"))
                    .expect("full command has a bounded code viewport");
                let output = find(card, &format!("codeout/{id}/output"))
                    .expect("full output has a bounded code viewport");
                for panel in [command, output] {
                    assert!(
                        panel.rect.x >= card.rect.x
                            && panel.rect.right() <= card.rect.right() + 0.1
                    );
                    assert!(panel.rect.h <= 400.);
                }
            }
            // The last card is visible. Scroll both axes inside its output without moving the transcript.
            let id = ids[1];
            let key = format!("codeout/{id}/output");
            let panel = find(layout, &key).unwrap().rect;
            let before = frame.scroll;
            assert!(view.wheel_delta([panel.x+5.,panel.y+5.],[0.,100_000.]));
            let frame=view.render(&state,[width,1800.]).unwrap();
            assert_eq!(frame.scroll,before);
            assert!(layout_text(find(frame.layout.as_ref().unwrap(),&key).unwrap()).contains("OUTPUT-END"),"output tail disappeared");
            let action=frame.actions.iter().find_map(|(_,a)|match a {bboxy450_harness::frontends::gui::view::Action::PanelScroll(k,d) if k==&key && d[0]>0.=>Some(a.clone()),_=>None}).unwrap();
            view.activate(action,&Bridge::new(Arc::new(Wake)));
            let frame=view.render(&state,[width,1800.]).unwrap();
            let output=find(frame.layout.as_ref().unwrap(),&key).unwrap();
            assert!(matches!(output.children[0].kind,bbt_ui::components::LayoutKind::ScrollView{offset,..} if offset[0]>0.));
            let command_key=format!("codeout/{id}/command");
            let command=find(frame.layout.as_ref().unwrap(),&command_key).unwrap().rect;
            assert!(view.wheel_delta([command.x+5.,command.y+5.],[0.,100_000.]));
            let frame=view.render(&state,[width,1800.]).unwrap();
            let text=layout_text(find(frame.layout.as_ref().unwrap(),&command_key).unwrap());
            assert!(text.contains("COMMAND-END"),"highlighted Python tail disappeared: {text}");
            let expanded_height=find(frame.layout.as_ref().unwrap(),&command_key).unwrap().rect.h;
            let compact=Faces::default();
            state.faces=Arc::new(compact.clone());
            state.prepared=Some(Arc::new(preparer.prepare_with_faces(source.clone(),width,&compact).unwrap()));
            view.scroll.follow_latest();
            let frame=view.render(&state,[width,1800.]).unwrap();
            let compact_height=find(frame.layout.as_ref().unwrap(),&command_key).unwrap().rect.h;
            assert!(compact_height<=76. && expanded_height>compact_height,"command expansion must change the bounded viewport height");
        }
    })
    .await;
}

#[tokio::test]
async fn acp_controls_tool_cards_text_zoom_keeps_full_horizontal_extent() {
    use bboxy450_harness::frontends::gui::{
        face,
        view::{Metrics, context},
    };
    use bbt_ui::{components as c, modules::seat_row::named};
    use bbt_ui_model::layout::Rect;
    runtime::blocking(|| {
        let metrics = Metrics::default();
        let ctx = context();
        let ink = face::ink(ctx);
        for scale in [0.9, 1., 1.6] {
            let content = face::term_line(
                vec![("UNBROKEN-COMMAND-".repeat(100), ink.fg, 400, false)],
                12.,
                false,
            );
            let ([width, height], _) = content.natural(f32::INFINITY, ctx, &metrics).unwrap();
            let mut panel = named(
                "codeout/fixture/command",
                c::Element::ScrollView(c::ScrollView {
                    viewport: [240., height],
                    offset: [0., 0.],
                    content_width: Some(width),
                    content: Box::new(content),
                }),
            );
            face::zoom_text(&mut panel, scale);
            let layout = panel
                .layout(Rect::new(0., 0., 240., 0.), ctx, &metrics)
                .unwrap();
            let scroll = &layout.children[0];
            let c::LayoutKind::ScrollView { content, .. } = scroll.kind else {
                panic!("literal scroll viewport");
            };
            fn right(n: &c::Layout) -> f32 {
                let own = match &n.kind {
                    c::LayoutKind::Text { runs, .. } => {
                        runs.iter().map(|r| r.rect.right()).fold(n.rect.x, f32::max)
                    }
                    _ => n.rect.x,
                };
                n.children.iter().map(right).fold(own, f32::max)
            }
            assert!(
                content[0] + 0.1 >= right(&scroll.children[0]) - scroll.rect.x,
                "zoom {scale}: scroll extent {content:?} lost the command tail"
            );
            assert_eq!(scroll.rect.w, 240., "zoom must not widen the card viewport");
        }
    })
    .await;
}

// Project-owned ACP title/name and large-MCP producer witnesses. Keep credit:
// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
fn quiet_tool(name: &str, title: &str, kind: ToolKind, output: &str) -> Item {
    serde_json::from_value(serde_json::json!({
        "kind":"tool_call", "call_id":"quiet-code", "name":name, "title":title,
        "tool_kind":kind, "input_raw":"{}", "locations":[], "spill":null,
        "output":[{"type":"text", "text":output}]
    }))
    .unwrap()
}

fn quiet_runs<'a>(
    layout: &'a bbt_ui::components::Layout,
    out: &mut Vec<&'a bbt_ui::components::PositionedRun>,
) {
    if let bbt_ui::components::LayoutKind::Text { runs, .. } = &layout.kind {
        out.extend(runs);
    }
    for child in &layout.children {
        quiet_runs(child, out);
    }
}

#[tokio::test]
async fn quiet_code_acp_name_title_python_and_shell_keep_bounded_accessible_source() {
    use bboxy450_harness::frontends::gui::{
        bridge::{FrameState, read_replay},
        rich::{Face, Faces},
        view::{Preparer, find, layout_text},
        zoom::Zoom,
    };
    let python = format!(
        "from pathlib import Path\n{}\nprint('PYTHON-LAST-雪🦊')\n",
        (0..50)
            .map(|n| format!("print('line-{n}', 42, 'αβ')"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let heredoc = format!("python3 - <<'PY'\n{python}PY");
    let shell = format!(
        "git add ATTRIBUTION.md Makefile\n{}\nprintf 'SHELL-LAST-雪🦊'",
        "git diff --cached --stat\n".repeat(50)
    );
    let ambiguous = "python3 - <<PY\nprint('$HOME')\nPY\nprintf 'MIXED-LAST'".to_owned();
    let output = format!(
        "OUTPUT-FIRST\n{}\nOUTPUT-LAST",
        "real-output-雪🦊\n".repeat(80)
    );
    for (name, title, command, highlighted) in [
        (heredoc.clone(), heredoc.clone(), heredoc.clone(), true),
        ("exec".into(), heredoc.clone(), heredoc.clone(), true),
        (shell.clone(), shell.clone(), shell.clone(), false),
        ("exec".into(), shell.clone(), shell.clone(), false),
        (ambiguous.clone(), ambiguous.clone(), ambiguous, false),
        (
            "python3 -c 'print(42, \"雪🦊\")'".into(),
            "Run Python".into(),
            "python3 -c 'print(42, \"雪🦊\")'".into(),
            true,
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let (path, ids) = markdown_replay(
            temp.path(),
            vec![(
                quiet_tool(&name, &title, ToolKind::Execute, &output),
                Some(ItemStatus::Failed {
                    reason: "fixture".into(),
                    code: None,
                }),
            )],
        );
        // Give the production duration slot real elapsed time, without a clock or provider.
        let events = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .enumerate()
            .map(|(n, line)| {
                let mut event: Envelope = serde_json::from_str(line).unwrap();
                event.ts_ms = n as u64 * 1500;
                serde_json::to_string(&event).unwrap()
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, events + "\n").unwrap();
        let source = Arc::new(read_replay(path).await.unwrap());
        let id = ids[0];
        runtime::blocking(move || {
            for width in [280., 1120.] {
                for percent in [90, 100, 160] {
                    for expanded in [false, true] {
                        let zoom = Zoom::from_percent(percent);
                        let mut preparer = Preparer::default();
                        preparer.set_zoom(zoom);
                        let faces: Faces = [(
                            id,
                            Face {
                                arguments_closed: expanded,
                                source: highlighted && expanded,
                                ..Default::default()
                            },
                        )]
                        .into_iter()
                        .collect();
                        let state = FrameState {
                            transcript: source.clone(),
                            ready: true,
                            replay: true,
                            faces: Arc::new(faces.clone()),
                            prepared: Some(Arc::new(
                                preparer
                                    .prepare_with_faces(source.clone(), width, &faces)
                                    .unwrap(),
                            )),
                            ..Default::default()
                        };
                        let mut view = HarnessView::standalone(zoom);
                        let frame = view.render(&state, [width, 1800.]).unwrap();
                        let card =
                            find(frame.layout.as_ref().unwrap(), &format!("tool/{id}")).unwrap();
                        let head = find(card, &format!("toolhead/{id}")).unwrap();
                        let heading = layout_text(head);
                        assert!(
                            head.rect.h < 75. && heading.chars().count() < 160,
                            "unbounded heading {heading:?}"
                        );
                        assert!(
                            (!highlighted || !heading.contains("python3 -"))
                                && !heading.contains("git diff")
                                && !heading.contains("PYTHON-LAST"),
                            "script repeated in heading: {heading:?}"
                        );
                        assert!(
                            heading.contains("Run") && heading.contains("1.5s"),
                            "label/duration lost: {heading:?}"
                        );
                        let status = find(card, &format!("tool/{id}/status")).unwrap();
                        assert!(
                            status.rect.right() <= card.rect.right() + 0.1,
                            "status escaped at {width}/{percent}"
                        );
                        let key = format!("codeout/{id}/command");
                        let panel = find(card, &key).unwrap();
                        assert!(panel.rect.h <= if expanded { 320.1 } else { 76.1 });
                        assert!(panel.rect.right() <= card.rect.right() + 0.1);
                        let first = layout_text(panel);
                        assert!(
                            first.contains(if highlighted {
                                "print"
                            } else {
                                command.lines().next().unwrap()
                            }),
                            "first line lost: {first}"
                        );
                        let mut runs = vec![];
                        quiet_runs(panel, &mut runs);
                        assert!(runs.iter().all(|r| r.text.len() <= 128));
                        let colors: std::collections::BTreeSet<_> = runs
                            .iter()
                            .map(|r| r.style.color.map(f32::to_bits))
                            .collect();
                        assert!(
                            if highlighted {
                                colors.len() >= 3
                            } else {
                                colors.len() == 1
                            },
                            "wrong language colors: {colors:?}"
                        );
                        if highlighted {
                            let raw = find(card, &format!("codeout/{id}/original"));
                            if expanded {
                                assert!(layout_text(raw.unwrap()).starts_with("python3"));
                            } else {
                                assert!(raw.is_none(), "original wrapper must be opt-in");
                            }
                            // The replay retains the exact wrapper; the pure parser borrows its bytes.
                            let snippet =
                                bboxy450_harness::frontends::gui::rich::python_snippet(&command)
                                    .unwrap();
                            assert_eq!(snippet.original, command);
                        }
                        let at = [panel.rect.x + 2., panel.rect.y + 2.];
                        let before = frame.scroll;
                        if command.lines().count() > 10 {
                            assert!(view.wheel_delta(at, [0., 100_000.]));
                            let frame = view.render(&state, [width, 1800.]).unwrap();
                            assert_eq!(frame.scroll, before, "command wheel escaped to transcript");
                            let tail =
                                layout_text(find(frame.layout.as_ref().unwrap(), &key).unwrap());
                            assert!(
                                tail.contains(if highlighted {
                                    "PYTHON-LAST-雪🦊"
                                } else {
                                    command.lines().last().unwrap()
                                }),
                                "last line lost: {tail}"
                            );
                        }
                    }
                }
            }
        })
        .await;
    }
}

#[tokio::test]
async fn quiet_code_large_mcp_expansion_raster_preserves_unicode_first_and_last_output() {
    use bboxy450_harness::frontends::gui::{
        assets,
        bridge::{FrameState, read_replay},
        rich::{Face, Faces},
        view::{Preparer, find, layout_text},
    };
    let temp = tempfile::tempdir().unwrap();
    // knowledge_fetch-shaped, public synthetic record: a large escaped source line and many fields.
    // This catches the real native raster limit; scissoring alone cannot bound a text canvas.
    let output=serde_json::to_string_pretty(&serde_json::json!({"id":"fixture", "body":format!("MCP-FIRST:{}:MCP-INNER-LAST", "α雪🦊\\\"".repeat(8000)), "resources":(0..100).map(|n|format!("record-{n}")).collect::<Vec<_>>(),"zz_tail":"MCP-LAST-雪🦊"})).unwrap();
    assert!(output.len() > 100_000);
    let (path, ids) = markdown_replay(
        temp.path(),
        vec![(
            quiet_tool(
                "mcp.bboxy.knowledge_fetch",
                "Fetch knowledge",
                ToolKind::Fetch,
                &output,
            ),
            Some(ItemStatus::Completed),
        )],
    );
    let source = Arc::new(read_replay(path).await.unwrap());
    let device = bbt_render::device::DeviceContext::new().await.unwrap();
    runtime::blocking(move || {
        let id = ids[0];
        let mut renderer =
            bbt_render::chrome::ChromeRenderer::with_context(device, assets::icons()).unwrap();
        for width in [280., 640.] {
            let faces: Faces = [(
                id,
                Face {
                    output_closed: true,
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect();
            let mut preparer = Preparer::default();
            let state = FrameState {
                transcript: source.clone(),
                ready: true,
                replay: true,
                faces: Arc::new(faces.clone()),
                prepared: Some(Arc::new(
                    preparer
                        .prepare_with_faces(source.clone(), width, &faces)
                        .unwrap(),
                )),
                ..Default::default()
            };
            let mut view = HarnessView::default();
            let frame = view.render(&state, [width, 900.]).unwrap();
            let layout = frame.layout.as_ref().unwrap();
            let head = find(layout, &format!("toolhead/{id}")).unwrap();
            assert!(
                layout_text(head).contains("knowledge fetch")
                    && layout_text(head).contains("bboxy")
            );
            let key = format!("mdout/{id}");
            let panel = find(layout, &key).unwrap();
            assert!(panel.rect.h <= 400.1 && layout_text(panel).contains("MCP-FIRST"));
            let mut runs = vec![];
            quiet_runs(panel, &mut runs);
            assert!(runs.iter().all(|r| r.text.len() <= 128));
            let pixels = renderer
                .render_to_rgba(&frame.list, width as u32, 900, 1.)
                .expect("real MCP native raster must not allocate an oversized text line");
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| p[0] > 160 && p[1] > 160 && p[2] > 160),
                "native glyph ink absent"
            );
            if let Some(root) = std::env::var_os("QUIET_CODE_CAPTURE_DIR") {
                let root = std::path::PathBuf::from(root);
                std::fs::create_dir_all(&root).unwrap();
                let mut encoder = png::Encoder::new(
                    std::fs::File::create(root.join(format!("mcp-expanded-{width}.png"))).unwrap(),
                    width as u32,
                    900,
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&pixels)
                    .unwrap();
            }
            let at = [panel.rect.x + 5., panel.rect.y + 5.];
            assert!(view.wheel_delta(at, [0., 1_000_000.]));
            let frame = view.render(&state, [width, 900.]).unwrap();
            assert!(
                layout_text(find(frame.layout.as_ref().unwrap(), &key).unwrap())
                    .contains("MCP-LAST-雪🦊")
            );
            renderer
                .render_to_rgba(&frame.list, width as u32, 900, 1.)
                .unwrap();
            // Exact original data remains in the transcript; producer bounds change no IR bytes.
            let Item::ToolCall { output: blocks, .. } = &source.items[&id].body else {
                panic!("MCP fixture")
            };
            assert_eq!(
                blocks[0],
                OutputBlock::Text {
                    text: output.clone()
                }
            );
        }
    })
    .await;
}
