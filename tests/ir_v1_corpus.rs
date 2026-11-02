// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    engine::{Config, SessionActor},
    gateway::{Gateway, Subscribe, unix::UnixServer},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    store::{Store, log_path},
};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

#[test]
fn frozen_corpus_is_readable() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ir-v1");
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "jsonl") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let events: Vec<_> = text
            .split_inclusive('\n')
            .map(|line| {
                let e = Envelope::from_line(line).unwrap();
                assert!(!matches!(e.event, Event::Unknown(_)), "{}", path.display());
                e
            })
            .collect();
        laws::check_transcript(&events).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        count += 1;
    }
    assert!(count >= 10);
}

#[tokio::test]
async fn future_bytes_survive_open_unix_fork_and_headless() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("sessions");
    let meta = support::meta(temp.path().into());
    let parent_id = meta.session;
    let parent = Store::open(root.clone(), meta.clone(), None, Default::default())
        .await
        .unwrap();
    let path = log_path(&root, parent_id);
    drop(parent);
    let future = format!(
        "{{ \"v\":{{\"major\":1,\"minor\":99}},\"session\":\"{parent_id}\",\"epoch\":1,\"seq\":2,\"at\":2,\"ts_ms\":123,\"cause\":null,\"future_field\":{{\"keep\":42}},\"event\":{{\"type\":\"future_event\",\"new_field\":true}} }}\n"
    );
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(future.as_bytes())
        .unwrap();
    let parent = Store::open(root.clone(), meta.clone(), None, Default::default())
        .await
        .unwrap();
    assert_eq!(parent.events[1].wire_bytes().unwrap(), future.as_bytes());
    let at = parent.head();
    drop(parent);
    let config = Config {
        meta: meta.clone(),
        sessions: root.clone(),
        profiles: temp.path().join("profiles"),
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Default::default(),
        max_rounds: 1,
        profile_error: None,
    };
    let (gateway, task) = SessionActor::open(config, OpId::new()).await.unwrap();
    let socket = temp.path().join("replay.sock");
    let server = UnixServer::bind(socket.clone(), gateway.clone())
        .await
        .unwrap();
    let (read, mut write) = UnixStream::connect(&socket).await.unwrap().into_split();
    write.write_all((serde_json::json!({"v":PROTOCOL,"subscribe":Subscribe{session:parent_id,after:None,ephemeral:false}}).to_string()+"\n").as_bytes()).await.unwrap();
    let mut reader = BufReader::new(read);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap(); // hello
    let head: serde_json::Value = serde_json::from_str(&line).unwrap();
    let count = head["head"].as_u64().unwrap();
    let mut seqs = vec![];
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        let event = Envelope::from_line(&line).unwrap();
        seqs.push(event.seq.unwrap().0);
        if event.seq == Some(Seq(2)) {
            assert_eq!(line, future);
        }
    }
    assert_eq!(seqs, (1..=count).collect::<Vec<_>>());
    server.close().await.unwrap();
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    let mut child_meta = support::meta(temp.path().into());
    child_meta.fork = Some((parent_id, at));
    let child = Store::open(root.clone(), child_meta.clone(), None, Default::default())
        .await
        .unwrap();
    assert_eq!(child.events[1].wire_bytes().unwrap(), future.as_bytes());
    let bytes = std::fs::read_to_string(log_path(&root, child.session)).unwrap();
    assert_eq!(bytes.split_inclusive('\n').nth(1).unwrap(), future);
    drop(child);
    let mut child = Store::open(root.clone(), child_meta, None, Default::default())
        .await
        .unwrap();
    child.append(Event::SessionClosed, None).await.unwrap();
    let id = child.session;
    drop(child);
    let closed = Store::replay_closed(root.clone(), id).await.unwrap();
    assert_eq!(closed[1].wire_bytes().unwrap(), future.as_bytes());
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .env("HOME", temp.path())
        .env_remove("BBOXY_CAPABILITY")
        .env_remove("BBOXY_CAPABILITY_FILE")
        .args([
            "--json",
            "--resume",
            &id.to_string(),
            "--sessions",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8(result.stdout)
            .unwrap()
            .split_inclusive('\n')
            .nth(1)
            .unwrap(),
        future
    );
}

#[test]
fn writer_rejects_nested_unknown_without_rejecting_opaque_text() {
    let events: Vec<Envelope> = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    laws::check_authored(&events).unwrap();
    let mut bad = events.clone();
    let e = bad
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::UserMessage { .. },
                    ..
                }
            )
        })
        .unwrap();
    if let Event::ItemCompleted {
        body: Item::UserMessage { blocks },
        ..
    } = &mut e.event
    {
        blocks.push(InputBlock::Unknown(
            serde_json::json!({"type":"future_input","data":42}),
        ));
    }
    laws::check_transcript(&bad).unwrap();
    assert_eq!(laws::check_authored(&bad).unwrap_err().law, "authorship");
    assert!(
        Version {
            major: 0,
            minor: 99
        }
        .is_readable()
    );
    assert!(
        Version {
            major: 1,
            minor: 99
        }
        .is_readable()
    );
    assert!(!Version { major: 2, minor: 0 }.is_readable());
}
