// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{gateway::Gateway, protocol::*};
#[tokio::test]
async fn unattended_denies_and_registry_declares_effects() {
    let s = support::session(vec![support::sse("edit"), support::sse("text")], false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    s.gateway
        .send(support::prompt(&s.gateway, "edit input"))
        .unwrap();
    let t = support::end(&mut sub).await;
    assert!(t.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            body: Item::Approval {
                by: Some(Resolver::Unattended),
                decision: Some(ApprovalDecision::Deny { .. }),
                ..
            },
            ..
        }
    )));
    assert_eq!(
        std::fs::read_to_string(s.temp.path().join("input.txt")).unwrap(),
        "before\n"
    );
    for d in bboxy450_harness::tools::Registry::new().definitions() {
        assert_eq!(
            d.effect.effect().is_none(),
            d.concurrency == bboxy450_harness::tools::Concurrency::ReadOnly
        );
    }
    support::close(s).await;
}
#[tokio::test]
async fn first_resolution_wins_and_stale_is_rejected() {
    let s = support::session(vec![support::sse("edit"), support::sse("text")], false).await;
    let mut sub = support::subscribe(&s.gateway, false);
    sub.set_approver(true);
    s.gateway.send(support::prompt(&s.gateway, "edit")).unwrap();
    let approval = loop {
        let e = sub.recv().await.unwrap();
        if let Event::ItemStarted {
            item,
            kind: ItemKind::Approval,
            ..
        } = e.event
        {
            break item;
        }
    };
    let first = support::op(
        &s.gateway,
        Op::ResolveApproval {
            approval,
            decision: ApprovalDecision::Allow,
        },
    );
    s.gateway.send(first).unwrap();
    let stale = support::op(
        &s.gateway,
        Op::ResolveApproval {
            approval,
            decision: ApprovalDecision::AllowAlways,
        },
    );
    s.gateway.send(stale.clone()).unwrap();
    let events = support::end(&mut sub).await;
    assert!(events.iter().any(|e|matches!(e.event,Event::OpRejected{op,reason:Rejection::AlreadyResolved} if op==stale.op_id)));
    assert_eq!(
        std::fs::read_to_string(s.temp.path().join("input.txt")).unwrap(),
        "after\n"
    );
    support::close(s).await;
}
