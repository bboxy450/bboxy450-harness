// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    context,
    engine::{Config, SessionActor},
    gateway::{Gateway, Host, Refused},
    policy::Policy,
    protocol::*,
    provider::{Caps, ModelEvent, ModelRequest, Provider, ScriptedProvider, profile::Unattended},
    store::{Store, log_path},
};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
struct Recording {
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}
impl Provider for Recording {
    fn caps(&self) -> Caps {
        ScriptedProvider::new(vec![]).caps()
    }
    fn stream(
        &self,
        r: ModelRequest,
        c: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        self.requests.lock().unwrap().push(r.clone());
        ScriptedProvider::new(vec![support::sse("text")]).stream(r, c)
    }
}
fn config(
    temp: &tempfile::TempDir,
    meta: SessionMeta,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
) -> Config {
    Config {
        meta,
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider: Arc::new(Recording { requests }),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Default::default(),
        max_rounds: 3,
        profile_error: None,
    }
}
async fn parent(temp: &tempfile::TempDir) -> Store {
    let mut store = Store::open(
        temp.path().join("sessions"),
        support::meta(temp.path().into()),
        Some(OpId::new()),
        Default::default(),
    )
    .await
    .unwrap();
    let blob = store
        .blobs
        .put(b"independent image bytes".to_vec(), "image/png")
        .await
        .unwrap();
    let turn = TurnId::new();
    let user = ItemId::new();
    let answer = ItemId::new();
    for event in [
        Event::TurnQueued { turn },
        Event::TurnStarted {
            turn,
            model: None,
            origin: None,
        },
        Event::ItemStarted {
            turn: Some(turn),
            item: user,
            kind: ItemKind::UserMessage,
            parent: None,
            head: None,
        },
        Event::ItemCompleted {
            item: user,
            status: ItemStatus::Completed,
            body: Item::UserMessage {
                blocks: vec![
                    InputBlock::Text {
                        text: "parent message".into(),
                    },
                    InputBlock::Image { blob },
                ],
            },
        },
        Event::ItemStarted {
            turn: Some(turn),
            item: answer,
            kind: ItemKind::AgentMessage,
            parent: None,
            head: None,
        },
        Event::ItemCompleted {
            item: answer,
            status: ItemStatus::Completed,
            body: Item::AgentMessage {
                markdown: "parent answer".into(),
            },
        },
        Event::TurnEnded {
            turn,
            reason: StopReason::EndTurn,
            usage: None,
        },
    ] {
        store.append(event, None).await.unwrap();
    }
    store
}
#[tokio::test]
async fn fork_is_self_contained_and_builds_the_same_first_request() {
    let temp = tempfile::tempdir().unwrap();
    let parent = parent(&temp).await;
    let root = temp.path().join("sessions");
    let parent_id = parent.session;
    let at = parent.head();
    let original = std::fs::read(log_path(&root, parent_id)).unwrap();
    let mut meta = support::meta(temp.path().into());
    meta.fork = Some((parent_id, at));
    let child_id = meta.session;
    let child = Store::open(
        root.clone(),
        meta.clone(),
        Some(OpId::new()),
        Default::default(),
    )
    .await
    .unwrap();
    assert_eq!(child.head(), at);
    assert_eq!(child.epoch, Epoch(1));
    for (a, b) in child
        .events
        .iter()
        .skip(1)
        .zip(parent.events.iter().skip(1))
    {
        assert_eq!(a.seq, b.seq);
        assert_eq!(a.ts_ms, b.ts_ms);
        assert_eq!(a.cause, b.cause);
        assert_eq!(a.event, b.event);
        assert_eq!(a.session, child_id);
        assert_eq!(a.epoch, Epoch(1));
    }
    laws::check_transcript(&child.events).unwrap();
    assert_eq!(std::fs::read(log_path(&root, parent_id)).unwrap(), original);
    let (parent_wire, _) = context::replay(&parent.events, &parent.blobs)
        .await
        .unwrap();
    let (child_wire, _) = context::replay(&child.events, &child.blobs).await.unwrap();
    assert_eq!(parent_wire, child_wire);
    let image = parent
        .events
        .iter()
        .find_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::UserMessage { blocks },
                ..
            } => blocks.iter().find_map(|b| {
                if let InputBlock::Image { blob } = b {
                    Some(blob.clone())
                } else {
                    None
                }
            }),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        child.blobs.get(&image).await.unwrap(),
        b"independent image bytes"
    );
    let parent_meta = match &parent.events[0].event {
        Event::SessionStarted { meta } => meta.clone(),
        _ => unreachable!(),
    };
    drop(child);
    drop(parent);
    let parent_requests = Arc::new(Mutex::new(vec![]));
    let (g, task) = SessionActor::open(
        config(&temp, parent_meta, parent_requests.clone()),
        OpId::new(),
    )
    .await
    .unwrap();
    let hello = g.hello();
    let mut sub = g
        .subscribe(bboxy450_harness::gateway::Subscribe {
            session: g.session(),
            after: Some((hello.epoch, hello.head)),
            ephemeral: false,
        })
        .unwrap();
    g.send(support::prompt(&g, "continue")).unwrap();
    support::end(&mut sub).await;
    g.send(support::op(&g, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    std::fs::remove_file(log_path(&root, parent_id)).unwrap();
    std::fs::remove_dir_all(root.join(parent_id.to_string())).unwrap();
    let child_requests = Arc::new(Mutex::new(vec![]));
    let (g, task) = SessionActor::open(config(&temp, meta, child_requests.clone()), OpId::new())
        .await
        .unwrap();
    let hello = g.hello();
    let mut sub = g
        .subscribe(bboxy450_harness::gateway::Subscribe {
            session: g.session(),
            after: Some((hello.epoch, hello.head)),
            ephemeral: false,
        })
        .unwrap();
    g.send(support::prompt(&g, "continue")).unwrap();
    support::end(&mut sub).await;
    g.send(support::op(&g, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    let p = parent_requests.lock().unwrap();
    let c = child_requests.lock().unwrap();
    assert_eq!(p[0].messages, c[0].messages);
    assert_eq!(p[0].model, c[0].model);
    assert_eq!(p[0].stable_prefix, c[0].stable_prefix);
    // Runtime MCP servers (bboxy_python) attach asynchronously; the first
    // request does not wait for them, so compare the session's own tools.
    let own = |tools: &[_]| -> Vec<serde_json::Value> {
        serde_json::to_value(tools)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|tool| !tool["name"].as_str().unwrap().starts_with("mcp__"))
            .cloned()
            .collect()
    };
    assert_eq!(own(&p[0].tools), own(&c[0].tools));
    let events = std::fs::read_to_string(log_path(&root, child_id))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .collect::<Vec<_>>();
    laws::check_transcript(&events).unwrap();
}
#[tokio::test]
async fn fork_while_parent_appends_copies_exactly_the_fixed_prefix() {
    let temp = tempfile::tempdir().unwrap();
    let mut parent = parent(&temp).await;
    let at = parent.head();
    let prefix = parent.events.clone();
    let mut meta = support::meta(temp.path().into());
    meta.fork = Some((parent.session, at));
    let child = meta.session;
    let host = Host::new(config(&temp, meta.clone(), Arc::new(Mutex::new(vec![]))));
    let start = OpEnvelope {
        v: PROTOCOL,
        session: child,
        op_id: OpId::new(),
        op: Op::StartSession {
            cwd: meta.cwd,
            profile: meta.profile,
            fork: meta.fork,
        },
    };
    let append = async {
        for _ in 0..50 {
            let item = ItemId::new();
            parent
                .append(
                    Event::ItemStarted {
                        turn: None,
                        item,
                        kind: ItemKind::Notice,
                        parent: None,
                        head: None,
                    },
                    None,
                )
                .await
                .unwrap();
            parent
                .append(
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body: Item::Notice {
                            level: Level::Info,
                            text: "later parent record".into(),
                        },
                    },
                    None,
                )
                .await
                .unwrap();
            tokio::task::yield_now().await;
        }
    };
    let (created, _) = tokio::join!(host.create(start), append);
    let (gateway, _) = created.unwrap();
    assert_eq!(gateway.hello().head, Seq(at.0 + 1));
    // Read the child's log once it is final: a live session may be mid-item
    // (an MCP startup notice) at any instant.
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    host.join().await.unwrap();
    let log = std::fs::read_to_string(log_path(&temp.path().join("sessions"), child)).unwrap();
    let events = log
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .collect::<Vec<_>>();
    for i in 1..at.0 as usize {
        assert_eq!(events[i].event, prefix[i].event);
        assert_eq!(events[i].seq, prefix[i].seq);
    }
    assert!(matches!(
        events[at.0 as usize].event,
        Event::ProfileChanged { .. }
    ));
    laws::check_transcript(&events).unwrap();
}
#[tokio::test]
async fn invalid_fork_points_are_refused_without_a_child_log() {
    let temp = tempfile::tempdir().unwrap();
    let mut parent = parent(&temp).await;
    let head = parent.head();
    parent.append(Event::SessionClosed, None).await.unwrap();
    for (id, at) in [
        (parent.session, Seq(0)),
        (parent.session, Seq(1)),
        (parent.session, Seq(2)),
        (parent.session, Seq(head.0 + 1)),
        (parent.session, Seq(head.0 + 2)),
        (SessionId::new(), head),
    ] {
        let mut meta = support::meta(temp.path().into());
        meta.fork = Some((id, at));
        let child = meta.session;
        let host = Host::new(config(&temp, meta.clone(), Arc::new(Mutex::new(vec![]))));
        let result = host
            .create(OpEnvelope {
                v: PROTOCOL,
                session: child,
                op_id: OpId::new(),
                op: Op::StartSession {
                    cwd: meta.cwd,
                    profile: meta.profile,
                    fork: meta.fork,
                },
            })
            .await;
        assert!(matches!(result, Err(Refused::BadFork)));
        assert!(!log_path(&temp.path().join("sessions"), child).exists());
    }
}

#[tokio::test]
async fn corrupt_parent_blob_never_publishes_a_child_log() {
    let temp = tempfile::tempdir().unwrap();
    let parent = parent(&temp).await;
    let root = temp.path().join("sessions");
    let blob = std::fs::read_dir(&parent.blobs.dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(blob, b"corrupt").unwrap();
    let mut meta = support::meta(temp.path().into());
    meta.fork = Some((parent.session, parent.head()));
    let child = meta.session;
    let error = Store::open(root.clone(), meta, Some(OpId::new()), Default::default())
        .await
        .err()
        .unwrap();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(!log_path(&root, child).exists());
}

#[tokio::test]
async fn queued_trial_frontier_is_not_a_public_fork_permission() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("sessions");
    let mut source = Store::open(
        root.clone(),
        support::meta(temp.path().into()),
        None,
        Default::default(),
    )
    .await
    .unwrap();
    source
        .append(
            Event::ProfileChanged {
                profile: "scripted".into(),
                model: "scripted".into(),
            },
            None,
        )
        .await
        .unwrap();
    source
        .append(
            Event::TurnQueued {
                turn: TurnId::new(),
            },
            None,
        )
        .await
        .unwrap();
    let cut = source.head();
    let bytes = std::fs::read(log_path(&root, source.session)).unwrap();
    let Event::SessionStarted { mut meta } = source.events[0].event.clone() else {
        panic!()
    };
    meta.session = SessionId::new();
    meta.fork = Some((source.session, cut));
    assert!(
        Store::open(root.clone(), meta.clone(), None, Default::default())
            .await
            .is_err()
    );
    assert!(!log_path(&root, meta.session).exists());
    assert_eq!(
        bytes,
        std::fs::read(log_path(&root, source.session)).unwrap()
    );
}
