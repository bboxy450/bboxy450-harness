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
