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
fn native_import_controls_expose_duplicate_accounting_and_references_before_blobs() {
    let m = model("HarnessNativeUsage");
    let turn = variable(&m, "turn");
    let seen = variable(&m, "seen_a");
    let finished = variable(&m, "finished_usage");
    let checked = m.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    let step = variable(&m, "session_step");
    let tokens = variable(&m, "session_tokens");
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[step] == 3 && s[tokens] == 1),
        "cumulative reset A-B-A is reachable"
    );
    assert!(
        model("BrokenHarnessNativeUsageScopes")
            .check(100_000)
            .unwrap()
            .violations
            .iter()
            .any(|v| v.property == "OnlyReportedTokens")
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[turn] == 1 && s[seen] == 1 && s[finished] == 1)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[turn] == 2 && s[finished] == 3)
    );
    let m = model("HarnessNativeSpill");
    let phase = variable(&m, "phase");
    let blob = variable(&m, "blob");
    let reference = variable(&m, "reference");
    let checked = m.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[phase] == 3 && s[blob] == 1 && s[reference] == 1)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[phase] == 4 && s[reference] == 0)
    );
    for (name, property) in [
        ("BrokenHarnessNativeUsage", "OneUsagePerCall"),
        ("BrokenHarnessNativeSpill", "ReferenceOnlyAfterBlob"),
    ] {
        assert!(
            model(name)
                .check(100_000)
                .unwrap()
                .violations
                .iter()
                .any(|v| v.property == property),
            "{name}"
        );
    }
}
#[test]
fn agent_path_controls_witness_duplicate_probe_missing_deadline_and_child_env() {
    let good = model("HarnessAgentPath");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    let cache = variable(&good, "cache");
    let a = variable(&good, "session_a");
    let b = variable(&good, "session_b");
    for path in [2, 3] {
        assert!(
            checked
                .states
                .iter()
                .any(|s| s[cache] == path && s[a] == 2 && s[b] == 2)
        );
    }
    for (name, property) in [
        (
            "BrokenHarnessAgentPathDuplicateProbe",
            "LoginShellProbedOnce",
        ),
        (
            "BrokenHarnessAgentPathNoDeadline",
            "Session2StartupProgress",
        ),
        ("BrokenHarnessAgentPathChildEnv", "Child2UsesResolvedPath"),
    ] {
        let checked = model(name).check(100_000).unwrap();
        assert!(
            checked.violations.iter().any(|v| v.property == property),
            "{name}: {:?}",
            checked.violations
        );
    }
}
#[test]
fn provider_controls_reach_retry_after_output_wrong_finish_and_double_ending() {
    let good = model("HarnessProviderStream");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    let phase = variable(&good, "phase");
    let calls = variable(&good, "calls");
    let output = variable(&good, "output");
    let outcome = variable(&good, "outcome");
    let reported = variable(&good, "usage_reported_total");
    let flushed = variable(&good, "usage_flushed_total");
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[reported] == 4 && s[flushed] == 4)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[calls] == 2 && s[outcome] == 2)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[calls] == 2 && s[outcome] == 3)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[calls] == 2 && s[outcome] == 4)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[phase] == 2 && s[output] == 0)
    );
    for (name, property) in [
        ("BrokenHarnessProviderRetry", "RetryOnlyBeforeOutput"),
        ("BrokenHarnessProviderFinish", "StopAgreesWithCalls"),
        ("BrokenHarnessProviderEnding", "OneTerminal"),
        ("BrokenHarnessProviderEnding", "OneUsageRecordPerResponse"),
        ("BrokenHarnessProviderEnding", "NoUsageLostAcrossRetries"),
    ] {
        let bad = model(name).check(100_000).unwrap();
        assert!(
            bad.violations.iter().any(|v| v.property == property),
            "{name}"
        );
    }
}
#[test]
fn capability_bootstrap_covers_valid_and_degraded_faces_and_legacy_read_leaks() {
    let good = model("HarnessMcpServer");
    let registered = variable(&good, "capability_registered");
    let available = variable(&good, "file_available");
    let prepared = variable(&good, "face_prepared");
    let projected = variable(&good, "output_projected");
    let notice = variable(&good, "notice_emitted");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[registered] == 1 && s[projected] == 1)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[available] == 2 && s[prepared] == 2 && s[notice] == 1)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[available] == 0 && s[prepared] == 3 && s[notice] == 0 && s[projected] == 1)
    );
    let bad = model("BrokenHarnessMcpServerD").check(100_000).unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property == "NoUnredactedCapability")
    );
    let noisy = model("BrokenHarnessMcpServerE").check(100_000).unwrap();
    assert!(
        noisy
            .violations
            .iter()
            .any(|v| v.property == "QuietWhenUnset")
    );
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
fn control_producers_conserve_slots_and_consumers_return_them() {
    let m = model("HarnessControlLaneBound");
    let depth = variable(&m, "control_depth");
    let interrupt = variable(&m, "interrupt_pending");
    let approval = variable(&m, "approval_pending");
    let close = variable(&m, "close_pending");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(
        r.states
            .iter()
            .any(|s| s[depth] == 130 && s[interrupt] == 65 && s[approval] == 64 && s[close] == 1)
    );
    assert_eq!(r.states.iter().map(|s| s[depth]).max(), Some(130));
    let full = r.states.iter().position(|s| s[depth] == 130).unwrap();
    assert!(
        r.edges[full]
            .iter()
            .any(|edge| r.states[edge.to][depth] == 129),
        "full lane cannot be consumed"
    );
    for name in ["ConsumeInterrupt", "ConsumeApproval", "ConsumeClose"] {
        assert!(
            m.actions.iter().any(|a| a.name == name),
            "missing consumer {name}"
        );
    }
    let b = model("BrokenHarnessControlLaneBound");
    let broken = b.check(100_000).unwrap();
    assert!(broken.states.iter().any(|s| s[depth] == 257));
    for property in [
        "InterruptSlotsConserved",
        "LegitimateOutstandingFits",
        "ControlLaneBounded",
    ] {
        assert!(
            broken.violations.iter().any(|v| v.property == property),
            "missing {property}"
        );
    }
}

#[test]
fn leader_exit_is_not_group_exit_and_both_ownership_controls_are_real() {
    let m = model("HarnessProcessLifecycle");
    let leader = variable(&m, "leader");
    let member = variable(&m, "member_alive");
    let cancel = variable(&m, "cancel");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty());
    assert!(
        r.states
            .iter()
            .any(|s| s[leader] == 1 && s[member] == 1 && s[cancel] == 0)
    );
    assert!(r.states.iter().any(|s| s[cancel] == 2));
    assert!(r.states.iter().all(|s| s[cancel] != 2 || s[member] == 0));
    for (name, property) in [
        ("BrokenHarnessProcessReapedMembers", "NoMemberAfterCancel"),
        ("BrokenHarnessProcessOwnership", "NoSignalToEmptyGroup"),
    ] {
        let broken = model(name).check(100_000).unwrap();
        assert!(
            broken
                .violations
                .iter()
                .any(|v| v.property == property && v.cycle.is_empty()),
            "{name}: missing {property}"
        );
    }
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
    for (name, property) in [
        ("BrokenHarnessSubscriberReplay", "FaceViewIsLogPrefix"),
        (
            "BrokenHarnessSubscriberReplayEpochReuse",
            "EpochNeverReused",
        ),
    ] {
        let broken = model(name).check(100_000).unwrap();
        assert!(
            broken
                .violations
                .iter()
                .any(|v| v.property == property && v.cycle.is_empty()),
            "broken rule had no safety counterexample"
        );
    }
}
#[test]
fn automatic_compaction_controls_violate_the_named_contracts() {
    let correct = model("HarnessAutoCompact");
    let report = correct.check(100_000).unwrap();
    assert!(report.violations.is_empty());
    let clipped = variable(&correct, "clipped");
    let replaced = variable(&correct, "replaced");
    let latest = variable(&correct, "latest_ending");
    let frontier = variable(&correct, "ended_through");
    assert!(
        report
            .states
            .iter()
            .any(|s| s[clipped] == 1 && s[replaced] == 1)
    );
    assert!(
        report
            .states
            .iter()
            .any(|s| s[latest] == 3 && s[frontier] == 1)
    );
    assert!(
        report
            .states
            .iter()
            .all(|s| s[clipped] <= s[frontier] && s[replaced] <= s[frontier])
    );
    let phase = variable(&correct, "phase");
    let steps = variable(&correct, "steps");
    let item = variable(&correct, "item");
    assert!(
        report.states.iter().any(|s| s[frontier] == 2),
        "completed frontier never advances"
    );
    assert!(
        report
            .states
            .iter()
            .any(|s| s[phase] == 2 && s[steps] == 1 && s[replaced] == 0),
        "trim-only request is unreachable"
    );
    assert!(
        report.states.iter().any(|s| s[item] == 2 && s[steps] == 0),
        "summary without a new trim is unreachable"
    );
    for (name, property) in [
        ("BrokenHarnessAutoCompactA", "NeverReducesOpenTurn"),
        ("BrokenHarnessAutoCompactB", "BoundariesMonotone"),
        ("BrokenHarnessAutoCompactC", "BoundaryEventuallySends"),
        ("BrokenHarnessAutoCompactD", "NeverReducesOpenTurn"),
    ] {
        let report = model(name).check(100_000).unwrap();
        assert!(
            report.violations.iter().any(|v| v.property == property),
            "{name}: {:#?}",
            report.violations
        );
    }
}
#[test]
fn acp_fifo_and_paused_writer_are_reachable_and_the_controls_are_real() {
    let m = model("HarnessAcpFlow");
    let r = m.check(100_000).unwrap();
    assert!(r.violations.is_empty(), "{:#?}", r.violations);
    let queue = variable(&m, "writer_queue");
    let head = variable(&m, "log_head");
    let turn = variable(&m, "turn");
    assert!(
        r.states.iter().enumerate().any(|(i, s)| s[queue] == 2
            && s[turn] == 1
            && s[head] < 2
            && r.edges[i]
                .iter()
                .any(|e| r.states[e.to][head] == s[head] + 1)),
        "engine cannot emit while the writer queue is full"
    );
    for (name, property) in [
        ("BrokenHarnessAcpFlowA", "CancelHitsItsPrompt"),
        ("BrokenHarnessAcpFlowB", "CloseOnlyOnEof"),
        ("BrokenHarnessAcpFlowC", "EngineNeverWaits"),
    ] {
        let report = model(name).check(100_000).unwrap();
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.property == property && v.cycle.is_empty()),
            "{name} did not fail {property}"
        );
    }
}
#[test]
fn seat_probes_and_stubborn_acp_agents_have_nonvacuous_controls() {
    let m = model("HarnessSeatReport");
    let report = m.check(100_000).unwrap();
    assert!(report.violations.is_empty());
    let phase = variable(&m, "phase");
    let profile = variable(&m, "profile");
    let probe = variable(&m, "probe");
    assert!(
        report
            .states
            .iter()
            .any(|s| s[phase] == 2 && s[profile] == 2 && s[probe] == 1)
    );
    let m = model("HarnessAcpAgentTurn");
    let report = m.check(100_000).unwrap();
    assert!(report.violations.is_empty());
    let phase = variable(&m, "phase");
    let stubborn = variable(&m, "stubborn");
    let alive = variable(&m, "agent_alive");
    assert!(
        report
            .states
            .iter()
            .any(|s| s[phase] == 4 && s[stubborn] == 1 && s[alive] == 0)
    );
    for (name, property) in [
        ("BrokenHarnessSeatReportState", "ReportMatchesState"),
        ("BrokenHarnessSeatReportProbe", "ProbeNeverBecomesInput"),
        (
            "BrokenHarnessAcpAgentGrace",
            "StubbornAgentKilledBeforeEnding",
        ),
    ] {
        let report = model(name).check(100_000).unwrap();
        assert!(
            report.violations.iter().any(|v| v.property == property),
            "{name}: {:#?}",
            report.violations
        );
    }
}
#[test]
fn acp_engine_emit_stays_enabled_with_a_full_writer() {
    for (name, blocked) in [("HarnessAcpFlow", false), ("BrokenHarnessAcpFlowC", true)] {
        let m = model(name);
        let r = m.check(100_000).unwrap();
        if !blocked {
            assert!(
                r.violations.is_empty(),
                "EngineNeverWaits: {:#?}",
                r.violations
            );
        } else {
            assert!(
                r.violations
                    .iter()
                    .any(|v| v.property == "EngineNeverWaits" && v.cycle.is_empty()),
                "missing EngineNeverWaits safety failure"
            );
        }
        let queue = variable(&m, "writer_queue");
        let head = variable(&m, "log_head");
        let turn = variable(&m, "turn");
        let conn = variable(&m, "conn");
        let mut checked = 0;
        for (i, s) in r
            .states
            .iter()
            .enumerate()
            .filter(|(_, s)| s[queue] == 2 && s[turn] == 1 && s[head] < 2 && s[conn] == 1)
        {
            let can_emit = r.edges[i]
                .iter()
                .any(|e| r.states[e.to][head] == s[head] + 1);
            assert_eq!(
                can_emit, !blocked,
                "EngineNeverWaits: actual EngineEmit transition disagrees"
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "full writer never coupled to a pending engine event"
        );
    }
}
#[test]
fn mcp_close_joins_members_even_when_term_is_ignored() {
    let m = model("HarnessMcpGroupClose");
    let r = m.check(100_000).unwrap();
    assert!(
        r.violations.is_empty(),
        "NoMemberAfterClose: {:#?}",
        r.violations
    );
    let phase = variable(&m, "close");
    let leader = variable(&m, "leader_alive");
    let members = variable(&m, "members_alive");
    assert!(
        r.states
            .iter()
            .any(|s| s[phase] == 1 && s[leader] == 1 && s[members] == 2),
        "ignored SIGTERM not reached"
    );
    assert!(
        r.states.iter().any(|s| s[phase] == 3),
        "Close never completes"
    );
    let broken = model("BrokenHarnessMcpGroupClose");
    let r = broken.check(100_000).unwrap();
    assert!(
        r.violations
            .iter()
            .any(|v| v.property == "NoMemberAfterClose" && v.cycle.is_empty()),
        "leader-only control did not fail NoMemberAfterClose"
    );
    assert!(
        r.states
            .iter()
            .any(|s| s[phase] == 3 && s[leader] == 0 && s[members] > 0),
        "leader-only Close hid the orphan"
    );
}
#[test]
fn mcp_controls_reach_aba_deadlock_and_silent_inflight() {
    let correct = model("HarnessMcpServer");
    let report = correct.check(100_000).unwrap();
    assert!(report.violations.is_empty());
    let generation = variable(&correct, "gen");
    let held = variable(&correct, "held");
    let tag = variable(&correct, "held_generation");
    assert!(
        report
            .states
            .iter()
            .any(|s| s[generation] == 2 && s[held] == 1 && s[tag] == 0),
        "no A-B-A held completion"
    );
    for (name, property) in [
        ("BrokenHarnessMcpServerA", "Deadlock"),
        ("BrokenHarnessMcpServerB", "NoStaleResult"),
        ("BrokenHarnessMcpServerC", "CallEventuallyEnds"),
    ] {
        let report = model(name).check(100_000).unwrap();
        assert!(
            report.violations.iter().any(|v| v.property == property),
            "{name}: {:#?}",
            report.violations
        );
    }
}

#[test]
fn synchronous_hangup_kill_precedes_the_supervisors_unjoinable_exit() {
    let m = model("HarnessProcessLifecycle");
    let alive = variable(&m, "seat_alive");
    let member = variable(&m, "member_alive");
    let hup = variable(&m, "hangup");
    let good = m.check(100_000).unwrap();
    assert!(
        good.violations.is_empty(),
        "NoOwnedMemberAfterSupervisorKill"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[hup] == 1 && s[alive] == 1 && s[member] == 0)
    );
    assert!(good.states.iter().any(|s| s[alive] == 0));
    assert!(good.states.iter().all(|s| s[alive] == 1 || s[member] == 0));
    let broken = model("BrokenHarnessProcessHangup").check(100_000).unwrap();
    assert!(
        broken
            .states
            .iter()
            .any(|s| s[hup] == 1 && s[alive] == 0 && s[member] == 1)
    );
    assert!(
        broken
            .violations
            .iter()
            .any(|v| v.property == "NoOwnedMemberAfterSupervisorKill" && v.cycle.is_empty())
    );
}

#[test]
fn reap_identity_is_retired_and_both_mcp_notifications_complete() {
    let owner = model("HarnessProcessLifecycle");
    let leader = variable(&owner, "leader");
    let registered = variable(&owner, "registered");
    let good = owner.check(100_000).unwrap();
    assert!(good.violations.is_empty(), "NoSignalToEmptyGroup");
    assert!(good.states.iter().any(|s| s[leader] == 2));
    assert!(
        good.states
            .iter()
            .all(|s| s[leader] != 2 || s[registered] == 0)
    );
    let bad_owner = model("BrokenHarnessProcessReapLease");
    let foreign_signal = variable(&bad_owner, "signal_to_empty");
    let broken = bad_owner.check(100_000).unwrap();
    assert!(
        broken
            .states
            .iter()
            .any(|s| s[leader] == 2 && s[registered] == 1 && s[foreign_signal] == 1),
        "stale registration never enabled a foreign signal"
    );
    assert!(
        broken
            .violations
            .iter()
            .any(|v| v.property == "NoSignalToEmptyGroup" && v.cycle.is_empty())
    );
    let mcp = model("HarnessMcpGroupClose");
    let notice = variable(&mcp, "notification");
    let done = variable(&mcp, "close");
    let reaped = variable(&mcp, "leader_reaped");
    let good = mcp.check(100_000).unwrap();
    assert!(good.violations.is_empty());
    for state in [1, 2] {
        assert!(
            good.states
                .iter()
                .any(|s| s[notice] == state && s[done] == 3)
        );
    }
    assert!(
        good.states.iter().any(|s| s[reaped] == 1 && s[done] == 0),
        "missing-id path is unreachable"
    );
    let bad = model("BrokenHarnessMcpLostChild").check(100_000).unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property == "NoMemberAfterClose" && v.cycle.is_empty())
    );
}
