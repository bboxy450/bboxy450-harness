// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{gateway::Gateway, protocol::*};
use support::*;
#[tokio::test]
async fn parallel_reads_then_edit_then_answer() {
    let s = session(vec![sse("tools"), sse("edit"), sse("text")], true).await;
    let mut sub = subscribe(&s.gateway, false);
    s.gateway.send(prompt(&s.gateway, "read and edit")).unwrap();
    let transcript = end(&mut sub).await;
    bboxy450_harness::protocol::laws::check_transcript(&transcript).unwrap();
    assert!(matches!(
        transcript.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    ));
    assert_eq!(
        std::fs::read_to_string(s.temp.path().join("input.txt")).unwrap(),
        "after\n"
    );
    let log = close(s).await;
    assert_eq!(
        log.iter()
            .filter(|e| matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::ToolCall { .. },
                    ..
                }
            ))
            .count(),
        3
    );
}
#[tokio::test]
async fn malformed_stream_has_one_ending() {
    let s = session(vec![sse("missing-finish")], false).await;
    let mut sub = subscribe(&s.gateway, false);
    s.gateway.send(prompt(&s.gateway, "hello")).unwrap();
    let t = end(&mut sub).await;
    assert!(matches!(
        t.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Error {
                class: ErrorClass::Malformed,
                ..
            },
            ..
        }
    ));
    close(s).await;
}
#[tokio::test]
async fn steer_at_round_boundary_appears_once_in_running_turn() {
    let mut p = bboxy450_harness::provider::ScriptedProvider::new(vec![sse("tools"), sse("text")]);
    p.delay = std::time::Duration::from_micros(200);
    let s = with_provider(std::sync::Arc::new(p), false).await;
    let mut sub = subscribe(&s.gateway, false);
    s.gateway.send(prompt(&s.gateway, "first")).unwrap();
    let mut transcript = vec![];
    loop {
        let e = sub.recv().await.unwrap();
        let started = matches!(e.event, Event::TurnStarted { .. });
        transcript.push(e);
        if started {
            break;
        }
    }
    let steer = op(
        &s.gateway,
        Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "steered context".into(),
            }],
            mode: TurnMode::Steer,
        },
    );
    s.gateway.send(steer.clone()).unwrap();
    transcript.extend(end(&mut sub).await);
    assert_eq!(
        transcript
            .iter()
            .filter(|e| matches!(e.event, Event::TurnQueued { .. }))
            .count(),
        1
    );
    assert_eq!(
        transcript
            .iter()
            .filter(|e| matches!(
                e.event,
                Event::ItemStarted {
                    kind: ItemKind::UserMessage,
                    ..
                }
            ) && e.cause == Some(steer.op_id))
            .count(),
        1
    );
    bboxy450_harness::protocol::laws::check_transcript(&transcript).unwrap();
    close(s).await;
}
#[tokio::test]
async fn steer_fallback_preserves_admission_order() {
    let mut p = bboxy450_harness::provider::ScriptedProvider::new(vec![
        sse("text"),
        sse("text"),
        sse("text"),
    ]);
    p.delay = std::time::Duration::from_micros(200);
    let s = with_provider(std::sync::Arc::new(p), false).await;
    let mut sub = subscribe(&s.gateway, false);
    s.gateway.send(prompt(&s.gateway, "first")).unwrap();
    let mut events = vec![];
    loop {
        let e = sub.recv().await.unwrap();
        let start = matches!(e.event, Event::TurnStarted { .. });
        events.push(e);
        if start {
            break;
        }
    }
    for (id, text) in [(99u128, "second"), (10u128, "third")] {
        let mut steer = op(
            &s.gateway,
            Op::UserTurn {
                blocks: vec![InputBlock::Text { text: text.into() }],
                mode: TurnMode::Steer,
            },
        );
        steer.op_id = OpId(ulid::Ulid::from(id));
        s.gateway.send(steer).unwrap();
    }
    for _ in 0..3 {
        events.extend(end(&mut sub).await);
    }
    let users = events
        .iter()
        .filter_map(|e| {
            if let Event::ItemCompleted {
                body: Item::UserMessage { blocks },
                ..
            } = &e.event
            {
                if let InputBlock::Text { text } = &blocks[0] {
                    Some(text.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(users, ["first", "second", "third"]);
    bboxy450_harness::protocol::laws::check_transcript(&events).unwrap();
    close(s).await;
}
#[tokio::test]
async fn bad_argument_object_is_a_paired_model_visible_error() {
    let raw=br#"data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"bad-args","function":{"name":"read","arguments":"[]"}}]},"finish_reason":"tool_calls"}]}

data: [DONE]

"#.to_vec();
    let s = session(vec![raw, sse("text")], false).await;
    let mut sub = subscribe(&s.gateway, false);
    s.gateway.send(prompt(&s.gateway, "call tools")).unwrap();
    let t = end(&mut sub).await;
    assert!(t.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:Item::ToolCall{output,..},..} if matches!(&output[0],OutputBlock::Error{..}))));
    assert!(matches!(
        t.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    ));
    close(s).await;
}
