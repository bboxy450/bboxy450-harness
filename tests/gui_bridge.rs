// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

mod support;
use bboxy450_harness::{
    frontends::gui::{bridge::Bridge, model::TurnState},
    protocol::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
#[derive(Default)]
struct Wake(AtomicUsize);
impl bbt_rt::Waker for Wake {
    fn wake(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
async fn wait(bridge: &mut Bridge, predicate: impl Fn(&Bridge) -> bool) {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if predicate(bridge) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("GUI wait timed out: {:?}", bridge.state.transcript));
}
#[tokio::test]
async fn standalone_start_resolves_profile_and_missing_profile_keeps_notice() {
    use bboxy450_harness::{
        engine::Config,
        policy::Policy,
        provider::{FailedProvider, ProviderError, profile::Unattended},
        redact::Set,
    };
    for configured in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let profiles = temp.path().join("profiles");
        std::fs::create_dir(&profiles).unwrap();
        if configured {
            std::fs::write(
                profiles.join("own-profile.md"),
                "---\nname: own-profile\nmodel: fixture-model\nendpoint: http://127.0.0.1:1/v1/chat/completions\napi_key: '!printf fixture-token'\n---\n",
            )
            .unwrap();
        }
        let mut meta = support::meta(temp.path().into());
        meta.profile = Some("own-profile".into());
        let mut bridge = Bridge::new(Arc::new(Wake::default()));
        bridge.create(Config {
            meta,
            profiles,
            sessions: temp.path().join("sessions"),
            provider: Arc::new(FailedProvider(ProviderError::new(
                ErrorClass::Config,
                "profile not loaded",
            ))),
            model: "unconfigured".into(),
            profile_error: Some("profile not loaded".into()),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 50,
        });
        wait(&mut bridge, |b| {
            b.state.ready
                && if configured {
                    b.state.transcript.profile
                        == Some(("own-profile".into(), "fixture-model".into()))
                } else {
                    b.state.transcript.items.values().any(|row| {
                        matches!(&row.body, Item::Notice { text, .. } if text == "profile could not be resolved")
                    })
                }
        })
        .await;
        assert!(bridge.state.error.is_none());
        bridge.send(Op::Close).unwrap();
        wait(&mut bridge, |b| b.state.transcript.closed).await;
        tokio::time::timeout(Duration::from_secs(8), bridge.detach().unwrap())
            .await
            .unwrap()
            .unwrap();
    }
}
#[tokio::test]
async fn background_subscription_and_detach_preserves_run() {
    let mut provider =
        bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    provider.delay = Duration::from_millis(5);
    let s = support::with_provider(Arc::new(provider), true).await;
    let mut observer = support::subscribe(&s.gateway, false);
    let wake = Arc::new(Wake::default());
    let mut bridge = Bridge::new(wake.clone());
    bridge.attach(s.gateway.clone());
    wait(&mut bridge, |b| b.state.ready).await;
    bridge
        .send(Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            mode: TurnMode::Queue,
        })
        .unwrap();
    wait(&mut bridge, |b| {
        b.state
            .transcript
            .turns
            .values()
            .any(|s| matches!(s, TurnState::Running))
    })
    .await;
    let detached = bridge.detach().unwrap();
    let events = support::end(&mut observer).await;
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.event, Event::SessionClosed))
    );
    tokio::time::timeout(Duration::from_secs(8), detached)
        .await
        .unwrap()
        .unwrap();
    assert!(wake.0.load(Ordering::Relaxed) > 0);
    support::close(s).await;
}

#[cfg(feature = "test-fixtures")]
#[tokio::test]
async fn unix_and_inproc_views_equal_every_ir_v1_fixture() {
    use bboxy450_harness::gateway::{InProc, unix::UnixServer};
    let mut fixtures = std::fs::read_dir("tests/fixtures/ir-v1")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|s| s == "jsonl"))
        .collect::<Vec<_>>();
    fixtures.sort();
    assert_eq!(
        fixtures.len(),
        12,
        "every locked IR v1 fixture must be covered"
    );
    for path in fixtures {
        let events = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_transcript(&events).unwrap();
        let Event::SessionStarted { meta } = &events[0].event else {
            panic!("fixture has no session");
        };
        let (gateway, publish) = InProc::fixture(meta.clone(), events[0].epoch);
        let own = tempfile::tempdir_in("/tmp").unwrap();
        let socket = own.path().join(format!("{}.sock", meta.session));
        let server = UnixServer::bind(socket.clone(), gateway.clone())
            .await
            .unwrap();
        let mut local = Bridge::new(Arc::new(Wake::default()));
        let mut unix = Bridge::new(Arc::new(Wake::default()));
        local.attach(gateway);
        unix.connect(socket, None);
        wait(&mut local, |b| b.state.ready).await;
        wait(&mut unix, |b| b.state.ready).await;
        for event in &events {
            publish(event.clone());
        }
        let expected = bboxy450_harness::frontends::gui::bridge::read_replay(path.clone())
            .await
            .unwrap();
        for bridge in [&mut local, &mut unix] {
            wait(bridge, |b| {
                *b.state.transcript == expected
                    && (expected.closed || b.state.ready && b.state.error.is_none())
            })
            .await;
            assert!(
                bridge.state.error.is_none(),
                "{}: {:?}",
                path.display(),
                bridge.state.error
            );
            assert_eq!(bridge.state.transcript.duplicate_durable, 0);
        }
        #[cfg(feature = "embedded")]
        {
            use bboxy450_harness::frontends::gui::view::{HarnessView, layout_text};
            let mut local_view = HarnessView::default();
            let mut unix_view = HarnessView::default();
            let a = local_view.render(&local.state, [1120., 16000.]).unwrap();
            let b = unix_view.render(&unix.state, [1120., 16000.]).unwrap();
            assert_eq!(
                layout_text(a.layout.as_ref().unwrap()),
                layout_text(b.layout.as_ref().unwrap()),
                "{}",
                path.display()
            );
        }
        local.detach().unwrap().await.unwrap();
        unix.detach().unwrap().await.unwrap();
        server.close().await.unwrap();
    }
}

#[tokio::test]
async fn unix_bridge_dispatches_off_thread_and_detach_keeps_its_turn() {
    use bboxy450_harness::{frontends::gui::bridge::Submission, gateway::unix::UnixServer};
    let mut provider =
        bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    provider.delay = Duration::from_millis(5);
    let s = support::with_provider(Arc::new(provider), true).await;
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let socket = own.path().join(format!("{}.sock", s.gateway.session()));
    let server = UnixServer::bind(socket.clone(), s.gateway.clone())
        .await
        .unwrap();
    let mut observer = support::subscribe(&s.gateway, false);
    let mut bridge = Bridge::new(Arc::new(Wake::default()));
    bbt_rt::register_ui_thread();
    bridge.connect(socket.clone(), None);
    wait(&mut bridge, |b| b.state.ready).await;
    let sent = bridge
        .send(Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "socket turn".into(),
            }],
            mode: TurnMode::Queue,
        })
        .unwrap();
    let Submission::Pending { op_id } = sent else {
        panic!("local scheduling must never claim a server Ack");
    };
    wait(&mut bridge, |b| {
        b.state
            .transcript
            .turns
            .values()
            .any(|t| matches!(t, TurnState::Running))
    })
    .await;
    let detached = bridge.detach().unwrap();
    let events = support::end(&mut observer).await;
    assert_eq!(
        events
            .iter()
            .filter(|e| e.cause == Some(op_id) && matches!(e.event, Event::TurnQueued { .. }))
            .count(),
        1
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.event, Event::SessionClosed))
    );
    tokio::time::timeout(Duration::from_secs(8), detached)
        .await
        .unwrap()
        .unwrap();
    assert!(
        socket.exists(),
        "view detach must not unlink the session socket"
    );
    support::close(s).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while socket.exists() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    server.close().await.unwrap();
}

#[tokio::test]
async fn unix_bridge_unavailable_connect_is_cancelled_without_ui_io() {
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let mut bridge = Bridge::new(Arc::new(Wake::default()));
    bridge.connect(own.path().join(format!("{}.sock", SessionId::new())), None);
    wait(&mut bridge, |b| b.state.error.is_some()).await;
    assert!(!bridge.state.ready);
    assert!(matches!(
        bridge.send(Op::Interrupt { turn: None }),
        Err(bboxy450_harness::gateway::Refused::Closed)
    ));
    tokio::time::timeout(Duration::from_secs(1), bridge.detach().unwrap())
        .await
        .unwrap()
        .unwrap();
}
#[cfg(feature = "embedded")]
#[tokio::test]
async fn rendered_approval_actions_use_control_lane() {
    for remote in [false, true] {
        for decision in [
            ApprovalDecision::Allow,
            ApprovalDecision::AllowAlways,
            ApprovalDecision::Deny { reason: None },
        ] {
            let s = support::session(vec![support::sse("edit"), support::sse("text")], false).await;
            let mut observer = support::subscribe(&s.gateway, false);
            let mut bridge = Bridge::new(Arc::new(Wake::default()));
            let own = tempfile::tempdir_in("/tmp").unwrap();
            let server = if remote {
                let socket = own.path().join(format!("{}.sock", s.gateway.session()));
                let server = bboxy450_harness::gateway::unix::UnixServer::bind(
                    socket.clone(),
                    s.gateway.clone(),
                )
                .await
                .unwrap();
                bridge.connect(socket, None);
                Some(server)
            } else {
                bridge.attach(s.gateway.clone());
                None
            };
            wait(&mut bridge, |b| b.state.ready).await;
            bridge
                .send(Op::UserTurn {
                    blocks: vec![InputBlock::Text {
                        text: "edit input".into(),
                    }],
                    mode: TurnMode::Queue,
                })
                .unwrap();
            wait(&mut bridge, |b| {
                b.state
                    .transcript
                    .items
                    .values()
                    .any(|r| r.kind == ItemKind::Approval && r.status.is_none())
            })
            .await;
            let id = bridge
                .state
                .transcript
                .items
                .values()
                .find(|r| r.kind == ItemKind::Approval)
                .unwrap()
                .id;
            let mut view = bboxy450_harness::frontends::gui::view::HarnessView::default();
            let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
            let action = frame
                .actions
                .iter()
                .find_map(|(_, a)| match a {
                    bboxy450_harness::frontends::gui::view::Action::Approval(item, d)
                        if *item == id && d == &decision =>
                    {
                        Some(a.clone())
                    }
                    _ => None,
                })
                .expect("approval decision must be a rendered button");
            view.activate(action, &bridge);
            assert!(view.notice.is_none());
            let events = support::end(&mut observer).await;
            assert!(events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::Approval { decision: Some(d), .. }, .. } if d == &decision)));
            let job = bridge.detach().unwrap();
            job.await.unwrap();
            if let Some(server) = server {
                server.close().await.unwrap();
            }
            support::close(s).await;
        }
    }
}

#[tokio::test]
async fn arbitrary_seat_address_reconnects_across_aba_without_retargeting_intent() {
    use bboxy450_harness::gateway::{Refused, unix::UnixServer};
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let socket = own.path().join("host-chosen.sock");
    let mut bridge = Bridge::new(Arc::new(Wake::default()));
    bridge.connect(socket.clone(), None);
    let mut sessions = vec![];
    let mut first_op = None;
    for birth in 0..3 {
        let s = support::session(vec![support::sse("text")], true).await;
        let session = s.gateway.session();
        sessions.push(session);
        let server = UnixServer::bind(socket.clone(), s.gateway.clone())
            .await
            .unwrap();
        wait(&mut bridge, |b| {
            b.state.ready && b.state.transcript.session == Some(session)
        })
        .await;
        assert_eq!(bridge.state.transcript.duplicate_durable, 0);
        if birth == 0 {
            first_op = Some(OpEnvelope {
                v: PROTOCOL,
                session,
                op_id: bboxy450_harness::gateway::unix::UnixClient::message_op_id("one-message"),
                op: Op::UserTurn {
                    blocks: vec![InputBlock::Text { text: "one".into() }],
                    mode: TurnMode::Queue,
                },
            });
        } else {
            assert!(matches!(
                bridge.send_envelope(first_op.clone().unwrap()),
                Err(Refused::UnknownSession)
            ));
        }
        support::close(s).await;
        server.retire().await.unwrap();
        assert!(
            bboxy450_harness::gateway::unix::UnixClient::discover(&socket, None, false, false)
                .await
                .is_err(),
            "the controlled rebind window is unbound"
        );
        wait(&mut bridge, |b| !b.state.ready).await;
    }
    assert_ne!(sessions[0], sessions[2]);
    bridge.detach().unwrap().await.unwrap();
}

#[tokio::test]
async fn message_ids_are_deterministic_and_namespaced() {
    use bboxy450_harness::gateway::unix::UnixClient;
    assert_eq!(
        UnixClient::message_op_id("message-a"),
        UnixClient::message_op_id("message-a")
    );
    assert_ne!(
        UnixClient::message_op_id("message-a"),
        UnixClient::message_op_id("message-b")
    );
    assert_ne!(
        UnixClient::message_op_id(""),
        UnixClient::message_op_id("message-a")
    );
}

#[tokio::test]
async fn seat_alias_refuses_files_symlinks_and_live_owners() {
    use bboxy450_harness::{frontends::seat::bind_alias, gateway::unix::UnixClient};
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let s = support::session(vec![support::sse("text")], true).await;
    let file = own.path().join("file");
    std::fs::write(&file, b"keep").unwrap();
    let link = own.path().join("link");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    for path in [&file, &link] {
        assert!(bind_alias(path, s.gateway.clone()).await.is_err());
    }
    assert_eq!(std::fs::read(&file).unwrap(), b"keep");
    assert!(
        bind_alias(std::path::Path::new("relative.sock"), s.gateway.clone())
            .await
            .is_err()
    );
    let socket = own.path().join("seat.sock");
    let server = bind_alias(&socket, s.gateway.clone()).await.unwrap();
    assert!(bind_alias(&socket, s.gateway.clone()).await.is_err());
    let (client, sub) = UnixClient::discover(&socket, None, false, false)
        .await
        .unwrap();
    assert_eq!(sub.hello().meta.session, s.gateway.session());
    client.close();
    support::close(s).await;
    server.retire().await.unwrap();
}

#[tokio::test]
async fn stale_unknown_dispatch_does_not_hold_detach_on_the_replacement_session() {
    use bboxy450_harness::gateway::unix::UnixServer;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    let first = support::session(vec![support::sse("text")], true).await;
    let own = tempfile::tempdir_in("/tmp").unwrap();
    let socket = own.path().join("seat.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let hello = first.gateway.hello();
    let (sent, received) = tokio::sync::oneshot::channel();
    let (release, released) = tokio::sync::oneshot::channel();
    let peer = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (read, mut write) = stream.into_split();
        let mut read = tokio::io::BufReader::new(read);
        let mut line = String::new();
        read.read_line(&mut line).await.unwrap();
        let mut bytes = serde_json::to_vec(&hello).unwrap();
        bytes.push(b'\n');
        write.write_all(&bytes).await.unwrap();
        line.clear();
        read.read_line(&mut line).await.unwrap();
        let op: OpEnvelope = serde_json::from_str(&line).unwrap();
        assert!(matches!(op.op, Op::UserTurn { .. }));
        // Close the session before the deliberately withheld admission reply.
        // The consumer leaves this connection's receive loop while the reply
        // task is still pending, making stale-outcome ordering deterministic.
        let closed = Envelope {
            v: PROTOCOL,
            session: hello.meta.session,
            epoch: hello.epoch,
            seq: Some(Seq(1)),
            at: Seq(1),
            ts_ms: 0,
            cause: None,
            event: Event::SessionClosed,
            raw: None,
            ext: Default::default(),
            native: None,
        };
        write
            .write_all(&closed.wire_bytes().unwrap())
            .await
            .unwrap();
        sent.send(()).unwrap();
        released.await.unwrap();
    });
    let mut bridge = Bridge::new(Arc::new(Wake::default()));
    bridge.connect(socket.clone(), None);
    wait(&mut bridge, |b| b.state.ready).await;
    bridge
        .send(Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "old intent".into(),
            }],
            mode: TurnMode::Queue,
        })
        .unwrap();
    received.await.unwrap();
    wait(&mut bridge, |b| b.state.transcript.closed).await;
    release.send(()).unwrap();
    peer.await.unwrap();
    std::fs::remove_file(&socket).unwrap();
    let next = support::session(vec![support::sse("text")], true).await;
    let session = next.gateway.session();
    let server = UnixServer::bind(socket, next.gateway.clone())
        .await
        .unwrap();
    wait(&mut bridge, |b| {
        b.state.ready && b.state.transcript.session == Some(session)
    })
    .await;
    assert!(
        bridge.state.error.is_none(),
        "stale response polluted the new face"
    );
    tokio::time::timeout(Duration::from_secs(1), bridge.detach().unwrap())
        .await
        .unwrap()
        .unwrap();
    support::close(next).await;
    server.retire().await.unwrap();
    support::close(first).await;
}
