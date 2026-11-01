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

#[tokio::test]
async fn deny_star_never_executes_the_counting_tool() {
    use bboxy450_harness::{
        policy::{Policy, Rule, RuleDecision},
        provider::{ScriptedProvider, profile::Unattended},
    };
    let program =
        "from pathlib import Path;p=Path('counter');p.write_text(str(int(p.read_text())+1))";
    let args = serde_json::json!({"program":"python3","args":["-c",program]});
    for allow in [false, true] {
        let policy = if allow {
            Policy::allow_all(Unattended::Deny)
        } else {
            Policy {
                rules: vec![Rule {
                    id: "deny-all".into(),
                    tool: "*".into(),
                    effect: None,
                    decision: RuleDecision::Deny,
                }],
                on_unattended: Unattended::Deny,
            }
        };
        let provider = std::sync::Arc::new(ScriptedProvider::new(vec![
            support::tool_sse("count", "exec", args.clone()),
            support::sse("text"),
        ]));
        let s = support::with_policy(provider, policy).await;
        std::fs::write(s.temp.path().join("counter"), "0").unwrap();
        let mut sub = support::subscribe(&s.gateway, false);
        s.gateway
            .send(support::prompt(&s.gateway, "count"))
            .unwrap();
        support::end(&mut sub).await;
        assert_eq!(
            std::fs::read_to_string(s.temp.path().join("counter")).unwrap(),
            if allow { "1" } else { "0" }
        );
        let log = support::close(s).await;
        if !allow {
            assert!(log.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:Item::Approval{by:Some(Resolver::Rule{id}),decision:Some(ApprovalDecision::Deny{..}),..},..} if id=="deny")));
        }
    }
}
#[tokio::test]
async fn old_turn_approval_cannot_resolve_the_current_turn() {
    let args = serde_json::json!({"path":"input.txt","old":"before","new":"after"});
    let s = support::session(
        vec![
            support::tool_sse("approval-A", "edit", args.clone()),
            support::tool_sse("approval-B", "edit", args),
            support::sse("text"),
        ],
        false,
    )
    .await;
    let mut sub = support::subscribe(&s.gateway, false);
    sub.set_approver(true);
    s.gateway
        .send(support::prompt(&s.gateway, "turn A"))
        .unwrap();
    let old = loop {
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
    s.gateway
        .send(support::op(&s.gateway, Op::Interrupt { turn: None }))
        .unwrap();
    let ended = support::end(&mut sub).await;
    assert!(matches!(
        ended.last().unwrap().event,
        Event::TurnEnded {
            reason: StopReason::Cancelled,
            ..
        }
    ));
    s.gateway
        .send(support::prompt(&s.gateway, "turn B"))
        .unwrap();
    let current = loop {
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
    assert_ne!(old, current);
    let stale = support::op(
        &s.gateway,
        Op::ResolveApproval {
            approval: old,
            decision: ApprovalDecision::AllowAlways,
        },
    );
    s.gateway.send(stale.clone()).unwrap();
    loop {
        let e = tokio::time::timeout(std::time::Duration::from_secs(2), sub.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(
            !matches!(e.event,Event::ItemCompleted{item,..} if item==current),
            "stale answer resolved current approval"
        );
        if let Event::OpRejected { op, reason } = e.event
            && op == stale.op_id
        {
            assert!(matches!(reason, Rejection::AlreadyResolved));
            break;
        }
    }
    s.gateway
        .send(support::op(
            &s.gateway,
            Op::ResolveApproval {
                approval: current,
                decision: ApprovalDecision::Deny { reason: None },
            },
        ))
        .unwrap();
    support::end(&mut sub).await;
    assert_eq!(
        std::fs::read_to_string(s.temp.path().join("input.txt")).unwrap(),
        "before\n"
    );
    support::close(s).await;
}
