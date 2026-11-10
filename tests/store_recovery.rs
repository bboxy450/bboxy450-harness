// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    protocol::{laws::check_transcript, *},
    redact::Set,
    store::Store,
};
#[tokio::test]
async fn every_truncation_recovers_innermost_and_pairs() {
    let text = include_str!("fixtures/protocol/all-items.jsonl");
    let lines = text
        .lines()
        .filter(|l| serde_json::from_str::<Envelope>(l).unwrap().seq.is_some())
        .collect::<Vec<_>>();
    let first: Envelope = serde_json::from_str(lines[0]).unwrap();
    let Event::SessionStarted { meta } = first.event else {
        panic!()
    };
    for n in 0..=lines.len() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join(format!("{}.jsonl", meta.session)),
            lines[..n].join("\n") + if n > 0 { "\n" } else { "" },
        )
        .unwrap();
        let store = Store::open(temp.path().into(), meta.clone(), None, Set::default())
            .await
            .unwrap();
        check_transcript(&store.events).unwrap_or_else(|e| panic!("cut {n}: {e}"));
        bboxy450_harness::engine::history::check_wire(&bboxy450_harness::engine::history::to_wire(
            &store.events,
        ))
        .unwrap();
        if n > 0 {
            assert_eq!(store.epoch, Epoch(2));
        }
        assert!(store.events.iter().filter(|e|matches!(e.event,Event::ItemCompleted{body:Item::ToolCall{..},status:ItemStatus::Interrupted,..})).all(|e|matches!(&e.event,Event::ItemCompleted{body:Item::ToolCall{output,..},..} if matches!(&output[0],OutputBlock::Error{text} if text=="tool call was not completed"))));
    }
}
#[tokio::test]
async fn incomplete_line_single_writer_and_read_only_close() {
    let temp = tempfile::tempdir().unwrap();
    let meta = support::meta(temp.path().into());
    let mut s = Store::open(temp.path().into(), meta.clone(), None, Set::default())
        .await
        .unwrap();
    assert!(
        Store::open(temp.path().into(), meta.clone(), None, Set::default())
            .await
            .is_err()
    );
    s.append(Event::SessionClosed, None).await.unwrap();
    drop(s);
    let s = Store::open(temp.path().into(), meta, None, Set::default())
        .await
        .unwrap();
    assert!(s.closed);
    assert_eq!(s.epoch, Epoch(1));
}

#[tokio::test]
async fn trusted_head_cursors_survive_reassigned_sequences() {
    use bboxy450_harness::{
        engine::{Config, SessionActor},
        gateway::{Gateway, Subscribe, TryRecv},
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
    };
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let root = t.path().join("sessions");
    let meta = support::meta(t.path().into());
    let id = meta.session;
    let mut store = Store::open(root.clone(), meta.clone(), None, Default::default())
        .await
        .unwrap();
    while store.head().0 < 10 {
        store
            .append(
                Event::ProfileChanged {
                    profile: "scripted".into(),
                    model: format!("old-{}", store.head().0),
                },
                None,
            )
            .await
            .unwrap();
    }
    drop(store);
    let path = root.join(format!("{id}.jsonl"));
    let truncate = |count: usize| {
        let bytes = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            bytes
                .lines()
                .take(count)
                .map(|l| l.to_string() + "\n")
                .collect::<String>(),
        )
        .unwrap();
    };
    truncate(8);
    let cfg = Config {
        meta,
        sessions: root.clone(),
        profiles: t.path().join("profiles"),
        provider: std::sync::Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: Default::default(),
        max_rounds: 1,
        profile_error: None,
    };
    let (gateway, task) = SessionActor::open(cfg.clone(), OpId::new()).await.unwrap();
    assert_eq!(gateway.hello().epoch, Epoch(2));
    assert_eq!(gateway.hello().head, Seq(9));
    support::runtime_ready(&gateway).await;
    let mut sub = support::subscribe(&gateway, false);
    for _ in 0..2 {
        gateway
            .send(support::op(
                &gateway,
                Op::UserTurn {
                    blocks: vec![],
                    mode: TurnMode::Queue,
                },
            ))
            .unwrap();
    }
    // Both empty turns are refused; the head after them is the replay target.
    let (mut rejected, mut head) = (0, 0);
    while rejected < 2 {
        let event = sub.recv().await.unwrap();
        head = event.seq.map_or(head, |s| s.0);
        rejected += usize::from(matches!(event.event, Event::OpRejected { .. }));
    }
    let request = |epoch, seq| Subscribe {
        session: id,
        after: Some((Epoch(epoch), Seq(seq))),
        ephemeral: false,
    };
    let mut stale = gateway.subscribe(request(1, 10)).unwrap();
    assert!(matches!(
        stale.recv().await.unwrap().event,
        Event::Reset {
            epoch: Epoch(2),
            head: Seq(8)
        }
    ));
    assert!(stale.recv().await.is_none());
    let mut safe = gateway.subscribe(request(1, 8)).unwrap();
    for seq in 9..=head {
        assert_eq!(safe.recv().await.unwrap().seq, Some(Seq(seq)));
    }
    assert!(matches!(safe.try_recv(), TryRecv::Empty));
    let mut current = gateway.subscribe(request(2, head)).unwrap();
    assert!(matches!(current.try_recv(), TryRecv::Empty));
    let mut future = gateway.subscribe(request(4, 1)).unwrap();
    assert!(matches!(
        future.recv().await.unwrap().event,
        Event::Reset { head: Seq(0), .. }
    ));
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    truncate(6);
    let (gateway, task) = SessionActor::open(cfg, OpId::new()).await.unwrap();
    assert_eq!(
        gateway.hello().epoch,
        Epoch(3),
        "a completely lost epoch must never be reused"
    );
    for (epoch, seq) in [(1, 8), (2, 9)] {
        let mut stale = gateway.subscribe(request(epoch, seq)).unwrap();
        assert!(matches!(
            stale.recv().await.unwrap().event,
            Event::Reset {
                epoch: Epoch(3),
                head: Seq(6)
            }
        ));
    }
    gateway.send(support::op(&gateway, Op::Close)).unwrap();
    task.await.unwrap().unwrap();
    std::fs::write(root.join(id.to_string()).join("epoch"), "not-an-epoch").unwrap();
    // A closed log is read-only; use the interrupted prefix to exercise allocator failure.
    truncate(6);
    let mut old = support::meta(t.path().into());
    old.session = id;
    assert!(
        Store::open(root, old, None, Default::default())
            .await
            .is_err()
    );
}
#[test]
fn killed_harness_recovers_the_original_tool_identity() {
    use std::time::{Duration, Instant};
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let root = t.path().join("sessions");
    let args = serde_json::json!({"command":"echo $$ > group.pid.tmp; mv group.pid.tmp group.pid; sleep 30 & wait"});
    let chunk = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"recovery-original-id","type":"function","function":{"name":"bash","arguments":args.to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    let script = t.path().join("tool.sse");
    std::fs::write(
        &script,
        format!("data: {chunk}\n\ndata: {finish}\n\ndata: [DONE]\n\n"),
    )
    .unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args(["-p", "run", "--json", "--yolo", "--cwd"])
        .arg(t.path())
        .arg("--sessions")
        .arg(&root)
        .arg("--scripted")
        .arg(script)
        .env("HOME", t.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pidfile = t.path().join("group.pid");
    let limit = Instant::now() + Duration::from_secs(4);
    while !pidfile.exists() {
        if Instant::now() > limit {
            let _ = child.kill();
            let _ = child.wait();
            panic!("tool process not ready");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let pgid: i32 = std::fs::read_to_string(pidfile)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    unsafe {
        libc::kill(-pgid, libc::SIGKILL);
    }
    let path = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    let first: Envelope = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    let meta = match first.event {
        Event::SessionStarted { meta } => meta,
        _ => unreachable!(),
    };
    let store = bbt_rt::handle()
        .block_on(Store::open(root, meta, None, Set::default()))
        .unwrap();
    let (id, name) = store
        .events
        .iter()
        .find_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::ToolCall { call_id, name, .. },
                ..
            } => Some((call_id, name)),
            _ => None,
        })
        .unwrap();
    assert_eq!(id, "recovery-original-id");
    assert_eq!(name, "bash");
    let wire = bboxy450_harness::engine::history::to_wire(&store.events);
    bboxy450_harness::engine::history::check_wire(&wire).unwrap();
    assert!(
        wire.iter()
            .any(|m| m["role"] == "tool" && m["tool_call_id"] == "recovery-original-id")
    );
    check_transcript(&store.events).unwrap();
}
