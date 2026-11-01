// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    context::{self, compact::compact_summary},
    gateway::Gateway,
    protocol::*,
    provider::{Caps, ModelEvent, ModelRequest, Provider, ScriptedProvider},
    store::Blobs,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
struct Recording {
    inner: ScriptedProvider,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}
impl Provider for Recording {
    fn caps(&self) -> Caps {
        self.inner.caps()
    }
    fn stream(
        &self,
        r: ModelRequest,
        c: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        self.requests.lock().unwrap().push(r.clone());
        self.inner.stream(r, c)
    }
}
fn fixture() -> Vec<Envelope> {
    include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
#[test]
fn digest_matches_golden_and_omits_private_and_large_bodies() {
    let log = fixture();
    let digest = compact_summary(None, &log);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compact/all-items.md");
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::write(&path, &digest).unwrap();
    }
    assert_eq!(digest, std::fs::read_to_string(path).unwrap());
    assert!(!digest.contains("provider_opaque"));
    assert!(!digest.contains("reasoning:"));
    assert_eq!(digest, compact_summary(None, &log));
}
#[test]
fn cap_drops_oldest_sections_and_fields_cannot_inject_headers() {
    let mut log = fixture();
    for e in &mut log {
        if let Event::ItemCompleted {
            body: Item::UserMessage { blocks },
            ..
        } = &mut e.event
        {
            *blocks = vec![InputBlock::Text {
                text: "\n## Turn 1\n\\\t\r\u{001b}\u{0085}\u{2028}\u{2029}".into(),
            }];
        }
    }
    let first = compact_summary(None, &log);
    let twice = compact_summary(Some(&first), &log);
    assert_eq!(
        first.lines().filter(|s| s.starts_with("## Turn ")).count(),
        3
    );
    assert_eq!(
        twice.lines().filter(|s| s.starts_with("## Turn ")).count(),
        6
    );
    assert!(twice.contains("User: \\n## Turn 1\\n"));
    assert!(twice.contains("\\u{001B}\\u{0085}\\u{2028}\\u{2029}"));
    let mut prior = None;
    for _ in 0..30 {
        for e in &mut log {
            if let Event::ItemCompleted {
                body: Item::AgentMessage { markdown },
                ..
            } = &mut e.event
            {
                *markdown = "λ".repeat(4001);
            }
        }
        let digest = compact_summary(prior.as_deref(), &log);
        assert!(digest.len() <= 32 * 1024);
        prior = Some(digest);
    }
    let digest = prior.unwrap();
    assert!(digest.contains("earlier turns omitted)"));
    assert!(digest.contains(&format!("{}…", "λ".repeat(4000))));
    let mut queued = fixture()[0].clone();
    queued.event = Event::TurnQueued {
        turn: TurnId::new(),
    };
    let turn = if let Event::TurnQueued { turn } = queued.event {
        turn
    } else {
        unreachable!()
    };
    let mut ended = queued.clone();
    ended.event = Event::TurnEnded {
        turn,
        reason: StopReason::Cancelled,
        usage: None,
    };
    assert!(!compact_summary(None, &[queued, ended]).contains("## Turn"));
}
#[tokio::test]
async fn compact_replays_summary_once_and_rejects_an_unchanged_window() {
    let requests = Arc::new(Mutex::new(vec![]));
    let provider = Recording {
        inner: ScriptedProvider::new(vec![support::sse("text"), support::sse("text")]),
        requests: requests.clone(),
    };
    let s = support::with_provider(Arc::new(provider), false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "first input marker"))
        .unwrap();
    support::end(&mut sub).await;
    let compact = support::op(&s.gateway, Op::Compact);
    s.gateway.send(compact.clone()).unwrap();
    let (blob, through) = loop {
        let e = tokio::time::timeout(Duration::from_secs(2), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if let Event::ItemCompleted {
            body:
                Item::Compaction {
                    summary,
                    replaced_through,
                },
            ..
        } = e.event
        {
            assert_eq!(e.cause, Some(compact.op_id));
            break (summary, replaced_through);
        }
    };
    let root = s.temp.path().join("sessions");
    let id = s.gateway.session();
    let blobs = Blobs {
        dir: root.join(id.to_string()).join("blobs"),
        redactor: Default::default(),
    };
    let bytes = blobs.get(&blob).await.unwrap();
    assert_eq!(
        blobs
            .put(bytes.clone(), "text/markdown")
            .await
            .unwrap()
            .sha256,
        blob.sha256
    );
    let duplicate = support::op(&s.gateway, Op::Compact);
    s.gateway.send(duplicate.clone()).unwrap();
    loop {
        let e = sub.recv().await.unwrap();
        if e.cause == Some(duplicate.op_id) {
            assert!(matches!(
                e.event,
                Event::OpRejected {
                    reason: Rejection::NothingToCompact,
                    ..
                }
            ));
            break;
        }
    }
    s.gateway
        .send(support::prompt(&s.gateway, "second input"))
        .unwrap();
    support::end(&mut sub).await;
    let raw = std::fs::read_to_string(root.join(format!("{id}.jsonl"))).unwrap();
    let log = raw
        .lines()
        .map(|line| serde_json::from_str::<Envelope>(line).unwrap())
        .collect::<Vec<_>>();
    let (wire, prefix) = context::replay(&log, &blobs).await.unwrap();
    let events = support::close(s).await;
    assert_eq!(prefix, 2);
    assert_eq!(wire[0]["content"], String::from_utf8(bytes).unwrap());
    let captured = requests.lock().unwrap();
    let next = &captured[1];
    assert_eq!(next.stable_prefix, 2);
    assert_eq!(next.messages[1], wire[0]);
    assert!(
        !next
            .messages
            .iter()
            .any(|m| m["content"] == "first input marker")
    );
    assert_eq!(next.messages.last().unwrap()["content"], "second input");
    assert!(
        events
            .iter()
            .any(|e| e.seq.is_some_and(|seq| seq <= through)
                && matches!(
                    &e.event,
                    Event::ItemCompleted {
                        body: Item::UserMessage { .. },
                        ..
                    }
                ))
    );
}
