// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    gateway::{
        Gateway, Subscribe,
        unix::{ClientError, UnixClient, UnixServer},
    },
    protocol::*,
};
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
};
#[tokio::test]
async fn transport_replay_and_duplicate_admission_equal_inproc() {
    let s = support::session(vec![support::sse("text")], false).await;
    let path = s.temp.path().join("sock");
    let server = UnixServer::bind(path.clone(), s.gateway.clone())
        .await
        .unwrap();
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let req = Subscribe {
        session: s.gateway.hello().meta.session,
        after: None,
        ephemeral: false,
    };
    let (client, mut sub) = UnixClient::connect(&path, req).await.unwrap();
    let op = support::prompt(&s.gateway, "remote");
    let ack = client.send(op.clone()).await.unwrap();
    assert_eq!(ack.op_id, op.op_id);
    assert!(s.gateway.send(op.clone()).unwrap().duplicate);
    assert!(client.send(op).await.unwrap().duplicate);
    let events = support::end(&mut sub).await;
    bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
    client.close();
    server.close().await.unwrap();
    support::close(s).await;
}
#[tokio::test]
async fn withheld_ack_times_out_without_blocking_envelopes_and_same_id_is_safe() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("mock.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let events = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .collect::<Vec<_>>();
    let meta = if let Event::SessionStarted { meta } = &events[0].event {
        meta.clone()
    } else {
        panic!()
    };
    let hello = bboxy450_harness::gateway::Hello {
        v: PROTOCOL,
        epoch: Epoch(1),
        head: Seq(0),
        meta: meta.clone(),
    };
    let effects = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let n = effects.clone();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let (r, mut w) = socket.into_split();
        let mut r = BufReader::new(r).lines();
        r.next_line().await.unwrap();
        w.write_all((serde_json::to_string(&hello).unwrap() + "\n").as_bytes())
            .await
            .unwrap();
        let first: OpEnvelope =
            serde_json::from_str(&r.next_line().await.unwrap().unwrap()).unwrap();
        n.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        for e in &events {
            w.write_all((serde_json::to_string(e).unwrap() + "\n").as_bytes())
                .await
                .unwrap();
        }
        let second: OpEnvelope =
            serde_json::from_str(&r.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(first.op_id, second.op_id);
        let ack =
            serde_json::json!({"op_id":second.op_id,"ack":{"op_id":second.op_id,"duplicate":true}});
        w.write_all((ack.to_string() + "\n").as_bytes())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
    let (client, mut sub) = UnixClient::connect(
        &path,
        Subscribe {
            session: meta.session,
            after: None,
            ephemeral: false,
        },
    )
    .await
    .unwrap();
    let op = OpEnvelope {
        v: PROTOCOL,
        session: meta.session,
        op_id: OpId::new(),
        op: Op::Compact,
    };
    let c = client.clone();
    let intent = op.clone();
    let start = Instant::now();
    let send = tokio::spawn(async move { c.send(intent).await });
    let transcript = support::end(&mut sub).await;
    assert!(!transcript.is_empty());
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(
        send.await.unwrap(),
        Err(ClientError::Timeout { op_id: op.op_id })
    );
    assert!(start.elapsed() < Duration::from_millis(5100));
    assert!(client.send(op).await.unwrap().duplicate);
    assert_eq!(effects.load(std::sync::atomic::Ordering::SeqCst), 1);
    client.close();
    server.await.unwrap();
}
#[tokio::test]
async fn disconnection_resolves_all_pending_sends() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("sock");
    let listener = UnixListener::bind(&path).unwrap();
    let meta = support::meta(t.path().into());
    let hello = bboxy450_harness::gateway::Hello {
        v: PROTOCOL,
        epoch: Epoch(1),
        head: Seq(0),
        meta: meta.clone(),
    };
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let (r, mut w) = socket.into_split();
        let mut r = BufReader::new(r).lines();
        r.next_line().await.unwrap();
        w.write_all((serde_json::to_string(&hello).unwrap() + "\n").as_bytes())
            .await
            .unwrap();
        r.next_line().await.unwrap();
        r.next_line().await.unwrap();
    });
    let (c, _) = UnixClient::connect(
        &path,
        Subscribe {
            session: meta.session,
            after: None,
            ephemeral: false,
        },
    )
    .await
    .unwrap();
    let op = |id| OpEnvelope {
        v: PROTOCOL,
        session: meta.session,
        op_id: id,
        op: Op::Compact,
    };
    let (a, b) = tokio::join!(c.send(op(OpId::new())), c.send(op(OpId::new())));
    assert_eq!(a, Err(ClientError::Disconnected));
    assert_eq!(b, Err(ClientError::Disconnected));
    server.await.unwrap();
}
