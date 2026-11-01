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
async fn close_cancels_and_joins_idle_profile_key_command() {
    let mut s = support::session(vec![support::sse("text")], false).await;
    let profiles = s.temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let pid = s.temp.path().join("key.pid");
    std::fs::write(profiles.join("slow.md"),format!("---\nname: slow\nmodel: unused\nendpoint: http://127.0.0.1:9/v1\napi_key: '!sleep 600 & echo $$ > {}; wait'\n---\n",pid.display())).unwrap();
    let change = support::op(
        &s.gateway,
        Op::SetProfile {
            profile: "slow".into(),
        },
    );
    s.gateway.send(change.clone()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !pid.exists() {
        assert!(Instant::now() < deadline, "profile command not started");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let pgid: i32 = std::fs::read_to_string(pid)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // The ready file is written after the child exists, so this witnesses a
    // two-process group rather than racing the shell's initial child spawn.
    let mut sub = support::subscribe(&s.gateway, false);
    let start = Instant::now();
    s.gateway.send(support::op(&s.gateway, Op::Close)).unwrap();
    let mut seen = vec![];
    let closed = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = sub.recv().await {
            let done = matches!(event.event, Event::SessionClosed);
            seen.push(serde_json::to_value(&event.event).unwrap()["type"].clone());
            if done {
                return;
            }
        }
    })
    .await;
    if closed.is_err() {
        let group_live = unsafe { libc::kill(-pgid, 0) } == 0;
        // This test owns the group and must leave no sleeping child on failure.
        if group_live {
            unsafe {
                libc::kill(-pgid, libc::SIGKILL);
            }
        }
        let _ = tokio::time::timeout(Duration::from_secs(2), &mut s.task).await;
        panic!("Close missed its unchanged 2s deadline; group_live={group_live}; events={seen:?}");
    }
    assert!(start.elapsed() < Duration::from_secs(2));
    let events = support::close(s).await;
    assert_eq!(unsafe { libc::kill(-pgid, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    assert_eq!(events.iter().filter(|e|e.cause==Some(change.op_id)&&matches!(e.event,Event::OpRejected{..})).count(),1);
    assert!(
        !events
            .iter()
            .any(|e| matches!(&e.event,Event::ProfileChanged{model,..} if model=="unused"))
    );
}
#[tokio::test]
async fn profile_resolution_applies_once_and_never_fetches_a_model() {
    let s = support::session(vec![], false).await;
    let profiles = s.temp.path().join("profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(profiles.join("next.md"),"---\nname: next\nmodel: next-model\nendpoint: http://127.0.0.1:9/v1\napi_key: '!printf test-credential'\n---\n").unwrap();
    let mut sub = support::subscribe(&s.gateway, false);
    let op = support::op(
        &s.gateway,
        Op::SetProfile {
            profile: "next".into(),
        },
    );
    s.gateway.send(op.clone()).unwrap();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(2), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if e.cause == Some(op.op_id) && matches!(e.event, Event::ProfileChanged { .. }) {
            break;
        }
    }
    assert!(s.gateway.send(op).unwrap().duplicate);
    support::close(s).await;
}
