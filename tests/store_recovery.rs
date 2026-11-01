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
