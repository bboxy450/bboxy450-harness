// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    engine::Config,
    gateway::{
        Gateway, Host, Refused, Subscribe,
        unix::{UnixClient, UnixServer},
    },
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
};
use std::{sync::Arc, time::Duration};
fn config(temp: &tempfile::TempDir, session: SessionId) -> Config {
    let mut meta = support::meta(temp.path().into());
    meta.session = session;
    Config {
        meta,
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Set::default(),
        max_rounds: 2,
        profile_error: None,
    }
}
fn start(session: SessionId, cwd: &std::path::Path) -> OpEnvelope {
    OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op: Op::StartSession {
            cwd: cwd.into(),
            profile: Some("scripted".into()),
            fork: None,
        },
    }
}
#[tokio::test]
async fn create_is_idempotent_and_bound_gateway_never_creates() {
    let t = tempfile::tempdir().unwrap();
    let session = SessionId::new();
    let host = Host::new(config(&t, session));
    let intent = start(session, t.path());
    let (g, ack) = host.create(intent.clone()).await.unwrap();
    assert!(!ack.duplicate);
    let (_, ack) = host.create(intent.clone()).await.unwrap();
    assert!(ack.duplicate);
    assert_eq!(g.send(intent), Err(Refused::SessionExists));
    let count = std::fs::read_to_string(t.path().join("sessions").join(format!("{session}.jsonl")))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .filter(|e| matches!(e.event, Event::SessionStarted { .. }))
        .count();
    assert_eq!(count, 1);
    g.send(support::op(&g, Op::Close)).unwrap();
    host.join().await.unwrap();
    assert!(matches!(host.open(session).await, Err(Refused::Closed)));
}
#[tokio::test]
async fn open_recovers_before_hello_and_subscribe_never_loads() {
    let t = tempfile::tempdir().unwrap();
    let all = include_str!("fixtures/protocol/interrupt-mid-tool.jsonl")
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .filter(|e| e.seq.is_some())
        .collect::<Vec<_>>();
    let session = all[0].session;
    let root = t.path().join("sessions");
    std::fs::create_dir(&root).unwrap();
    let prefix = all
        .iter()
        .take_while(|e| !matches!(e.event, Event::ItemCompleted { .. }))
        .map(|e| serde_json::to_string(e).unwrap() + "\n")
        .collect::<String>();
    std::fs::write(root.join(format!("{session}.jsonl")), prefix).unwrap();
    let host = Host::new(config(&t, session));
    let socket = t.path().join("session.sock");
    assert!(!socket.exists());
    let g = host.open(session).await.unwrap();
    // Subscription creation is a synchronous result, with no runtime to poll.
    let subscribe: fn(
        &bboxy450_harness::gateway::InProc,
        Subscribe,
    ) -> Result<bboxy450_harness::gateway::Subscription, Refused> = Gateway::subscribe;
    let mut sub = subscribe(
        &g,
        Subscribe {
            session,
            after: None,
            ephemeral: false,
        },
    )
    .unwrap();
    assert_eq!(sub.hello().epoch, Epoch(2));
    let head = sub.hello().head;
    let bytes = std::fs::read_to_string(root.join(format!("{session}.jsonl"))).unwrap();
    let log = bytes
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(head, log.last().unwrap().seq.unwrap());
    laws::check_transcript(&log).unwrap();
    assert!(matches!(
        g.subscribe(Subscribe {
            session: SessionId::new(),
            after: None,
            ephemeral: false
        }),
        Err(Refused::UnknownSession)
    ));
    assert_eq!(host.open(session).await.unwrap().hello().epoch, Epoch(2));
    let server = UnixServer::bind(socket.clone(), g.clone()).await.unwrap();
    let (client, _) = UnixClient::connect(
        &socket,
        Subscribe {
            session,
            after: None,
            ephemeral: false,
        },
    )
    .await
    .unwrap();
    client.close();
    server.close().await.unwrap();
    g.send(support::op(&g, Op::Close)).unwrap();
    host.join().await.unwrap();
    while sub.recv().await.is_some() {}
}
#[tokio::test]
async fn closed_missing_and_corrupt_sessions_are_not_exposed() {
    let t = tempfile::tempdir().unwrap();
    let session = SessionId::new();
    let cfg = config(&t, session);
    let root = cfg.sessions.clone();
    let host = Host::new(cfg.clone());
    assert!(matches!(
        host.open(session).await,
        Err(Refused::UnknownSession)
    ));
    let (g, _) = host.create(start(session, t.path())).await.unwrap();
    g.send(support::op(&g, Op::Close)).unwrap();
    host.join().await.unwrap();
    let reloaded = Host::new(cfg);
    assert!(matches!(reloaded.open(session).await, Err(Refused::Closed)));
    let bad = SessionId::new();
    std::fs::write(root.join(format!("{bad}.jsonl")), "not JSON\n").unwrap();
    assert!(matches!(reloaded.open(bad).await, Err(Refused::Closed)));
    let original = std::fs::read_to_string(root.join(format!("{session}.jsonl"))).unwrap();
    let repaired = original
        .lines()
        .map(|line| serde_json::from_str::<Envelope>(line).unwrap())
        .filter(|e| !matches!(e.event, Event::SessionClosed))
        .map(|mut e| {
            e.session = bad;
            if let Event::SessionStarted { meta } = &mut e.event {
                meta.session = bad;
            }
            serde_json::to_string(&e).unwrap() + "\n"
        })
        .collect::<String>();
    std::fs::write(root.join(format!("{bad}.jsonl")), repaired).unwrap();
    let recovered = reloaded.open(bad).await.unwrap();
    let mut sub = support::subscribe(&recovered, false);
    let head = sub.hello().head;
    let mut notices = 0;
    loop {
        let event = sub.recv().await.unwrap();
        if matches!(&event.event,Event::ItemCompleted{body:Item::Notice{level:Level::Error,text},..} if text.contains("recovery attempt failed"))
        {
            notices += 1;
        }
        if event.seq == Some(head) {
            break;
        }
    }
    assert_eq!(notices, 1);
    recovered.send(support::op(&recovered, Op::Close)).unwrap();
    reloaded.join().await.unwrap();
}

#[tokio::test]
async fn idle_socket_keeps_a_silent_client_and_emits_heartbeat() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let session = SessionId::new();
    let config = config(&temp, session);
    let host = Host::new(config);
    let (gateway, _) = host.create(start(session, temp.path())).await.unwrap();
    let socket = UnixServer::bind(temp.path().join("idle.sock"), gateway.clone())
        .await
        .unwrap();
    let (_client, mut sub) = UnixClient::connect(
        &socket.path,
        Subscribe {
            session,
            after: None,
            ephemeral: true,
        },
    )
    .await
    .unwrap();
    let start = std::time::Instant::now();
    loop {
        let event = tokio::time::timeout(Duration::from_secs(17), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(event.event, Event::Status { phase: Phase::Idle }) {
            assert!(event.seq.is_none());
            break;
        }
    }
    assert!(start.elapsed() >= Duration::from_secs(14));
    gateway
        .send(OpEnvelope {
            v: PROTOCOL,
            session,
            op_id: OpId::new(),
            op: Op::Close,
        })
        .unwrap();
    host.join().await.unwrap();
    socket.close().await.unwrap();
}
