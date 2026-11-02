// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

mod support;
use bboxy450_harness::{
    frontends::gui::{bridge::Bridge, model::TurnState},
    protocol::*,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
#[derive(Default)]
struct Wake(AtomicUsize);
impl bbt_rt::Waker for Wake {
    fn wake(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
async fn wait(bridge: &mut Bridge, predicate: impl Fn(&Bridge) -> bool) {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            bridge.drain(Duration::from_micros(300));
            if predicate(bridge) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("GUI wait timed out: {:?}", bridge.state.transcript));
}
#[tokio::test]
async fn standalone_start_resolves_profile_and_missing_profile_keeps_notice() {
    use bboxy450_harness::{
        engine::Config,
        policy::Policy,
        provider::{FailedProvider, ProviderError, profile::Unattended},
        redact::Set,
    };
    for configured in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let profiles = temp.path().join("profiles");
        std::fs::create_dir(&profiles).unwrap();
        if configured {
            std::fs::write(
                profiles.join("own-profile.md"),
                "---\nname: own-profile\nmodel: fixture-model\nendpoint: http://127.0.0.1:1/v1/chat/completions\napi_key: '!printf fixture-token'\n---\n",
            )
            .unwrap();
        }
        let mut meta = support::meta(temp.path().into());
        meta.profile = Some("own-profile".into());
        let mut bridge = Bridge::new(Arc::new(Wake::default()));
        bridge.create(Config {
            meta,
            profiles,
            sessions: temp.path().join("sessions"),
            provider: Arc::new(FailedProvider(ProviderError::new(
                ErrorClass::Config,
                "profile not loaded",
            ))),
            model: "unconfigured".into(),
            profile_error: Some("profile not loaded".into()),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 50,
        });
        wait(&mut bridge, |b| {
            b.state.ready
                && if configured {
                    b.state.transcript.profile
                        == Some(("own-profile".into(), "fixture-model".into()))
                } else {
                    b.state.transcript.items.values().any(|row| {
                        matches!(&row.body, Item::Notice { text, .. } if text == "profile could not be resolved")
                    })
                }
        })
        .await;
        assert!(bridge.state.error.is_none());
        bridge.send(Op::Close).unwrap();
        wait(&mut bridge, |b| b.state.transcript.closed).await;
        tokio::time::timeout(Duration::from_secs(8), bridge.detach().unwrap())
            .await
            .unwrap()
            .unwrap();
    }
}
#[tokio::test]
async fn background_subscription_and_detach_preserves_run() {
    let mut provider =
        bboxy450_harness::provider::ScriptedProvider::new(vec![support::sse("text")]);
    provider.delay = Duration::from_millis(5);
    let s = support::with_provider(Arc::new(provider), true).await;
    let mut observer = support::subscribe(&s.gateway, false);
    let wake = Arc::new(Wake::default());
    let mut bridge = Bridge::new(wake.clone());
    bridge.attach(s.gateway.clone());
    wait(&mut bridge, |b| b.state.ready).await;
    bridge
        .send(Op::UserTurn {
            blocks: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            mode: TurnMode::Queue,
        })
        .unwrap();
    wait(&mut bridge, |b| {
        b.state
            .transcript
            .turns
            .values()
            .any(|s| matches!(s, TurnState::Running))
    })
    .await;
    let detached = bridge.detach().unwrap();
    let events = support::end(&mut observer).await;
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::EndTurn,
            ..
        }
    )));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.event, Event::SessionClosed))
    );
    tokio::time::timeout(Duration::from_secs(8), detached)
        .await
        .unwrap()
        .unwrap();
    assert!(wake.0.load(Ordering::Relaxed) > 0);
    support::close(s).await;
}
#[cfg(feature = "embedded")]
#[tokio::test]
async fn rendered_approval_actions_use_control_lane() {
    for decision in [
        ApprovalDecision::Allow,
        ApprovalDecision::AllowAlways,
        ApprovalDecision::Deny { reason: None },
    ] {
        let s = support::session(vec![support::sse("edit"), support::sse("text")], false).await;
        let mut observer = support::subscribe(&s.gateway, false);
        let mut bridge = Bridge::new(Arc::new(Wake::default()));
        bridge.attach(s.gateway.clone());
        wait(&mut bridge, |b| b.state.ready).await;
        bridge
            .send(Op::UserTurn {
                blocks: vec![InputBlock::Text {
                    text: "edit input".into(),
                }],
                mode: TurnMode::Queue,
            })
            .unwrap();
        wait(&mut bridge, |b| {
            b.state
                .transcript
                .items
                .values()
                .any(|r| r.kind == ItemKind::Approval && r.status.is_none())
        })
        .await;
        let id = bridge
            .state
            .transcript
            .items
            .values()
            .find(|r| r.kind == ItemKind::Approval)
            .unwrap()
            .id;
        let mut view = bboxy450_harness::frontends::gui::view::HarnessView::default();
        let frame = view.render(&bridge.state, [1120., 7000.]).unwrap();
        let action = frame
            .actions
            .iter()
            .find_map(|(_, a)| match a {
                bboxy450_harness::frontends::gui::view::Action::Approval(item, d)
                    if *item == id && d == &decision =>
                {
                    Some(a.clone())
                }
                _ => None,
            })
            .expect("approval decision must be a rendered button");
        view.activate(action, &bridge);
        assert!(view.notice.is_none());
        let events = support::end(&mut observer).await;
        assert!(events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::Approval { decision: Some(d), .. }, .. } if d == &decision)));
        let job = bridge.detach().unwrap();
        job.await.unwrap();
        support::close(s).await;
    }
}
