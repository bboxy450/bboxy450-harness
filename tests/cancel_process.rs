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
    let s = support::session(vec![support::sse("bash")], true).await;
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
    tokio::time::sleep(Duration::from_millis(40)).await;
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
    let log = support::close(s).await;
    for e in log {
        if let Event::ItemCompleted {
            body: Item::Terminal { terminal, .. },
            ..
        } = e.event
        {
            let pgid: i32 = terminal.parse().unwrap();
            assert_eq!(unsafe { libc::kill(-pgid, 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
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
async fn a_b_a_is_fenced_by_birth_not_identity() {
    use bboxy450_harness::{
        redact::Set,
        store::Blobs,
        tools::{Context, Registry},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("a"), "late old result").unwrap();
    let generation = Arc::new(AtomicU64::new(1));
    let old = Context {
        cwd: temp.path().into(),
        blobs: Blobs {
            dir: temp.path().join("blobs"),
            redactor: Set::default(),
        },
        redactor: Set::default(),
        generation: 1,
        current_generation: generation.clone(),
        cancel: tokio_util::sync::CancellationToken::new(),
    };
    generation.store(2, Ordering::Release);
    generation.store(3, Ordering::Release);
    let output = Registry::new()
        .run("read", serde_json::json!({"path":"a"}), old, None)
        .await;
    assert!(output.failed);
    assert!(!format!("{:?}", output.output).contains("late old result"));
}
