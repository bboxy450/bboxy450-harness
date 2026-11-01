// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{gateway::Gateway, protocol::*};
use std::time::Duration;
#[tokio::test]
async fn exec_streams_a_repairable_tail_before_completion() {
    let program = "import os,time;os.write(1,b'x'*204800);open('emitted','w').write('ready');\nwhile not os.path.exists('release'): time.sleep(.01)";
    let args = serde_json::json!({"program":"python3","args":["-c",program]});
    let chunk = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"stream-exec","type":"function","function":{"name":"exec","arguments":args.to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    let s = support::session(
        vec![
            format!("data: {chunk}\n\ndata: {finish}\n\ndata: [DONE]\n\n").into_bytes(),
            support::sse("text"),
        ],
        true,
    )
    .await;
    let mut sub = support::subscribe(&s.gateway, true);
    s.gateway
        .send(support::prompt(&s.gateway, "stream exec"))
        .unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(e) = sub.recv().await {
            if let Event::ItemSnapshot {
                partial: Item::Terminal { .. },
                ..
            } = &e.event
            {
                let value = serde_json::to_value(&e).unwrap();
                let live = value["event"]["partial"]["live"].as_str().unwrap_or("");
                if live.ends_with(&"x".repeat(65536)) {
                    return e;
                }
            }
        }
        panic!("stream ended before live output");
    })
    .await;
    if let Ok(snapshot) = &observed {
        let hello = s.gateway.hello();
        let mut late = s
            .gateway
            .subscribe(bboxy450_harness::gateway::Subscribe {
                session: s.gateway.session(),
                after: Some((hello.epoch, hello.head)),
                ephemeral: true,
            })
            .unwrap();
        let repair = tokio::time::timeout(Duration::from_millis(500), async {
            loop {
                let e = late.recv().await.unwrap();
                if matches!(
                    e.event,
                    Event::ItemSnapshot {
                        partial: Item::Terminal { .. },
                        ..
                    }
                ) {
                    break e;
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(
            serde_json::to_value(repair).unwrap()["event"]["partial"]["live"],
            serde_json::to_value(snapshot).unwrap()["event"]["partial"]["live"]
        );
    }
    std::fs::write(s.temp.path().join("release"), "").unwrap();
    let end = support::end(&mut sub).await;
    let events = support::close(s).await;
    let snapshot = observed.expect("no live terminal tail was delivered while the exec was held");
    let value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(
        value["event"]["partial"]["live"],
        format!("…{}", "x".repeat(65536))
    );
    assert!(end.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:Item::Terminal{output:Some(blob),..},..} if blob.len==204800)));
}

#[tokio::test]
async fn output_chunks_never_exceed_sixteen_kib_after_lossy_decoding() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let req = bboxy450_harness::process::Request {
        program: "python3".into(),
        args: vec![
            "-c".into(),
            "import os;os.write(1,bytes([255])*16384)".into(),
        ],
        stdin: None,
        cwd: temp.path().into(),
        timeout: Duration::from_secs(5),
        spill_dir: temp.path().join("spill"),
        cap: 51200,
        redactor: Default::default(),
    };
    let run = bboxy450_harness::process::run_stream(
        req,
        tokio_util::sync::CancellationToken::new(),
        Some(tx),
    );
    let drain = async {
        let mut chunks = vec![];
        while let Some(chunk) = rx.recv().await {
            assert!(chunk.text.len() <= 16 * 1024);
            assert!(!chunk.text.is_empty());
            chunks.push(chunk.text);
        }
        chunks
    };
    let (out, chunks) = tokio::join!(run, drain);
    assert_eq!(out.unwrap().code, Some(0));
    assert!(chunks.len() >= 3);
    assert_eq!(chunks.concat(), "�".repeat(16384));
}
