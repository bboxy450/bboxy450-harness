// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{gateway::Gateway, protocol::*};
use std::time::{Duration, Instant};
#[tokio::test]
async fn cancel_kills_group_and_joins_before_ending() {
    let command = "echo $$ > group.pid; sleep 600 & wait";
    let chunk = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"owned-call","type":"function","function":{"name":"bash","arguments":serde_json::json!({"command":command}).to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    let s = support::session(
        vec![format!("data: {chunk}\n\ndata: {finish}\n\ndata: [DONE]\n\n").into_bytes()],
        true,
    )
    .await;
    let mut sub = support::subscribe(&s.gateway, true);
    s.gateway
        .send(support::prompt(&s.gateway, "run a child"))
        .unwrap();
    let mut transcript = vec![];
    loop {
        let e = tokio::time::timeout(Duration::from_secs(3), sub.recv())
            .await
            .unwrap()
            .unwrap();
        let ready = matches!(
            e.event,
            Event::ItemCompleted {
                body: Item::Approval {
                    decision: Some(ApprovalDecision::Allow),
                    ..
                },
                ..
            }
        );
        transcript.push(e);
        if ready {
            break;
        }
    }
    let pidfile = s.temp.path().join("group.pid");
    let deadline = Instant::now() + Duration::from_secs(3);
    while !pidfile.exists() {
        assert!(Instant::now() < deadline, "tool process did not start");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let pgid: i32 = std::fs::read_to_string(pidfile)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let start = Instant::now();
    s.gateway
        .send(support::op(&s.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    transcript.extend(support::end(&mut sub).await);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(matches!(
        transcript.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    ));
    bboxy450_harness::protocol::laws::check_transcript(&transcript).unwrap();
    assert_eq!(unsafe { libc::kill(-pgid, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    support::close(s).await;
}
#[tokio::test]
async fn real_process_group_witness() {
    let t = tempfile::tempdir().unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    let c = token.clone();
    let req = bboxy450_harness::process::Request {
        program: "bash".into(),
        args: vec!["-c".into(), "sleep 600 & sleep 600; wait".into()],
        stdin: None,
        cwd: t.path().into(),
        timeout: Duration::from_secs(900),
        spill_dir: t.path().join("spill"),
        cap: 51200,
        redactor: Default::default(),
    };
    let task = bbt_rt::handle()
        .spawn(async move { bboxy450_harness::process::run(req, c).await.unwrap() });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let start = Instant::now();
    token.cancel();
    let result = task.await.unwrap();
    assert!(result.cancelled);
    assert!(start.elapsed() < Duration::from_secs(2));
    for _ in 0..100 {
        if unsafe { libc::kill(-(result.pgid as i32), 0) } == -1 {
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("process group survived cancellation");
}
#[tokio::test]
async fn escaped_pipe_holder_cannot_stall_a_cancelled_turn() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/setsid.py");
    let command = format!(
        "python3 '{}' escaped.pid & echo $$ > group.pid; wait",
        fixture.display()
    );
    let args = serde_json::json!({"command":command});
    let chunk = serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"escaped-call","type":"function","function":{"name":"bash","arguments":args.to_string()}}]},"finish_reason":null}]});
    let finish = serde_json::json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]});
    let s = support::session(
        vec![format!("data: {chunk}\n\ndata: {finish}\n\ndata: [DONE]\n\n").into_bytes()],
        true,
    )
    .await;
    let mut sub = support::subscribe(&s.gateway, true);
    s.gateway
        .send(support::prompt(&s.gateway, "spawn escaped pipe holder"))
        .unwrap();
    let pidfile = s.temp.path().join("escaped.pid");
    let limit = Instant::now() + Duration::from_secs(4);
    while !pidfile.exists() {
        assert!(Instant::now() < limit, "escaped fixture not ready");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let pid: i32 = std::fs::read_to_string(pidfile)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let start = Instant::now();
    s.gateway
        .send(support::op(&s.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(2), support::end(&mut sub)).await;
    let elapsed = start.elapsed();
    // Test-owned escaped session is deliberately outside product ownership.
    unsafe {
        libc::kill(-pid, libc::SIGKILL);
    }
    let _ = support::close(s).await;
    let transcript = result.expect("escaped pipe kept the cancelled turn alive beyond 2 seconds");
    assert!(elapsed < Duration::from_secs(2));
    assert!(matches!(
        transcript.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    ));
}
