// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![allow(dead_code)]
use bboxy450_harness::{
    engine::{Config, SessionActor},
    gateway::{Gateway, InProc, Subscribe, Subscription},
    policy::Policy,
    protocol::*,
    provider::{Provider, ScriptedProvider, profile::Unattended},
    redact::Set,
};
use std::{path::PathBuf, sync::Arc, time::Duration};
pub struct Session {
    pub temp: tempfile::TempDir,
    pub gateway: InProc,
    pub task: tokio::task::JoinHandle<std::io::Result<()>>,
}
pub fn meta(cwd: PathBuf) -> SessionMeta {
    SessionMeta {
        vcs: None,
        dream: None,
        source: None,
        session: SessionId::new(),
        cwd,
        profile: Some("scripted".into()),
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: 0,
    }
}
pub async fn session(rounds: Vec<Vec<u8>>, allow: bool) -> Session {
    with_provider(Arc::new(ScriptedProvider::new(rounds)), allow).await
}
pub async fn with_provider(provider: Arc<dyn Provider>, allow: bool) -> Session {
    with_policy(
        provider,
        if allow {
            Policy::allow_all(Unattended::Deny)
        } else {
            Policy::new(Unattended::Deny)
        },
    )
    .await
}
pub async fn with_policy(provider: Arc<dyn Provider>, policy: Policy) -> Session {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("input.txt"), "before\n").unwrap();
    std::fs::write(temp.path().join("other.txt"), "other\n").unwrap();
    let config = Config {
        meta: meta(temp.path().into()),
        profiles: temp.path().join("profiles"),
        sessions: temp.path().join("sessions"),
        provider,
        model: "scripted".into(),
        policy,
        redactor: Set::default(),
        profile_error: None,
        max_rounds: 50,
    };
    let (gateway, task) = SessionActor::open(config, OpId::new()).await.unwrap();
    Session {
        temp,
        gateway,
        task,
    }
}
pub fn subscribe(g: &InProc, ephemeral: bool) -> Subscription {
    g.subscribe(Subscribe {
        session: g.hello().meta.session,
        after: None,
        ephemeral,
    })
    .unwrap()
}
pub fn op(g: &InProc, op: Op) -> OpEnvelope {
    OpEnvelope {
        v: PROTOCOL,
        session: g.hello().meta.session,
        op_id: OpId::new(),
        op,
    }
}
pub fn prompt(g: &InProc, text: &str) -> OpEnvelope {
    op(
        g,
        Op::UserTurn {
            blocks: vec![InputBlock::Text { text: text.into() }],
            mode: TurnMode::Queue,
        },
    )
}
pub async fn end(sub: &mut Subscription) -> Vec<Envelope> {
    let mut out = vec![];
    loop {
        let e = tokio::time::timeout(Duration::from_secs(8), sub.recv())
            .await
            .expect("turn timed out")
            .expect("stream closed");
        let done = matches!(e.event, Event::TurnEnded { .. });
        out.push(e);
        if done {
            break;
        }
    }
    out
}
pub async fn close(s: Session) -> Vec<Envelope> {
    let path = s
        .temp
        .path()
        .join("sessions")
        .join(format!("{}.jsonl", s.gateway.hello().meta.session));
    let _ = s.gateway.send(op(&s.gateway, Op::Close));
    tokio::time::timeout(Duration::from_secs(2), s.task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let log = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect::<Vec<_>>();
    bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
    bboxy450_harness::engine::history::check_wire(&bboxy450_harness::engine::history::to_wire(
        &log,
    ))
    .unwrap();
    log
}
pub fn sse(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/sse/{name}.sse")),
    )
    .unwrap()
}

pub fn tool_sse(id: &str, name: &str, args: serde_json::Value) -> Vec<u8> {
    let chunk = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":id,"type":"function","function":{"name":name,"arguments":args.to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    format!("data: {chunk}\n\ndata: {finish}\n\ndata: [DONE]\n\n").into_bytes()
}
