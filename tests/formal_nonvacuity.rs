// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#[path = "formal/models.rs"]
mod models;
use bbt_tlc::Expr;
fn model(name: &str) -> bbt_tlc::Model {
    models::corpus()
        .into_iter()
        .find(|(m, _)| m.name == name)
        .expect("required contract model absent")
        .0
}
fn variable(m: &bbt_tlc::Model, name: &str) -> usize {
    m.variables
        .iter()
        .position(|v| v.name == name)
        .unwrap_or_else(|| panic!("missing state variable {name}"))
}
#[test]
fn cancellation_tag_changes_and_removing_the_fence_exposes_a_stale_apply() {
    let m = model("HarnessCancelTree");
    let generation = variable(&m, "generation");
    let identity = variable(&m, "identity");
    let held = variable(&m, "held");
    let tag = variable(&m, "held_generation");
    let bad = variable(&m, "bad");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(r.states.iter().all(|s| s[bad] == 0));
    assert!(
        r.states.iter().any(|s| s[tag] == 1),
        "no completion was ever tagged with a later birth"
    );
    assert!(
        r.states
            .iter()
            .any(|s| s[generation] == 2 && s[identity] == 0 && s[held] == 1 && s[tag] == 0),
        "A-B-A never retained A's completion"
    );
    let mut unfenced = m.clone();
    let apply = unfenced
        .actions
        .iter_mut()
        .find(|a| a.name == "HeldCompletion")
        .unwrap();
    apply.guard = Expr::Var(held).eq(Expr::Int(1));
    assert!(
        !unfenced.check(100_000).unwrap().violations.is_empty(),
        "removing the fence still passed"
    );
}
#[test]
fn acked_steer_reaches_both_exactly_once_routes() {
    let m = model("HarnessSteerQueue");
    let delivered = variable(&m, "delivered");
    let queued = variable(&m, "queued");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(r.states.iter().all(|s| s[delivered] + s[queued] <= 1));
    assert!(r.states.iter().any(|s| s[delivered] == 1 && s[queued] == 0));
    assert!(r.states.iter().any(|s| s[delivered] == 0 && s[queued] == 1));
}
#[test]
fn control_capacity_is_reachable_and_its_broken_control_overflows() {
    let m = model("HarnessControlLaneBound");
    let depth = variable(&m, "control_depth");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(r.states.iter().any(|s| s[depth] == 256));
    let b = model("BrokenHarnessControlLaneBound");
    assert!(!b.check(100_000).unwrap().violations.is_empty());
}

#[test]
fn subscriber_model_can_lose_an_entire_epoch_and_rejects_both_old_rules() {
    let m = model("HarnessSubscriberReplay");
    let head = variable(&m, "head");
    let hwm = variable(&m, "hwm");
    let observed = variable(&m, "log_epoch_max");
    let phase = variable(&m, "phase");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(
        r.states
            .iter()
            .any(|s| s[phase] == 2 && s[head] == 0 && s[hwm] == 2 && s[observed] == 0),
        "whole-epoch loss was unreachable"
    );
    for name in [
        "BrokenHarnessSubscriberReplay",
        "BrokenHarnessSubscriberReplayEpochReuse",
    ] {
        let broken = model(name).check(100_000).unwrap();
        assert!(
            broken.violations.iter().any(|v| v.cycle.is_empty()),
            "broken rule had no safety counterexample"
        );
    }
}
