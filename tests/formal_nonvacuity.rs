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
fn socket_delivery_controls_duplicate_effect_and_retarget_across_session_births() {
    let good = model("HarnessSocketDelivery");
    let current = variable(&good, "session");
    let bound = variable(&good, "bound");
    let target = variable(&good, "target");
    let msg = variable(&good, "msg");
    let a = variable(&good, "turns_A");
    let b = variable(&good, "turns_B");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[current] == 2 && s[bound] == 0 && s[target] == 0 && s[msg] == 1),
        "a pending A message must survive B and the third A birth"
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[current] == 2 && s[bound] == 0 && s[msg] == 3),
        "an old socket must actually refuse the pending message"
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[a] == 1 && s[variable(&good, "resent")] == 1 && s[msg] == 2),
        "duplicate ack must remain reachable"
    );
    let fresh = model("BrokenHarnessSocketDeliveryFreshId")
        .check(100_000)
        .unwrap();
    assert!(
        fresh
            .violations
            .iter()
            .any(|v| v.property == "OneTurnPerMessage")
    );
    assert!(
        fresh.states.iter().any(|s| s[a] == 2),
        "fresh resend ID must create two actual turns"
    );
    let retarget = model("BrokenHarnessSocketDeliveryRetarget")
        .check(100_000)
        .unwrap();
    assert!(
        retarget
            .violations
            .iter()
            .any(|v| v.property.starts_with("NoTurnOnUnannouncedSession"))
    );
    assert!(
        retarget
            .states
            .iter()
            .any(|s| s[a] == 1 && s[b] == 1 && s[bound] == 0),
        "retarget must deliver one message to two real sessions"
    );
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
    let age = variable(&m, "report_age");
    let heartbeat = variable(&m, "heartbeat");
    let origin = variable(&m, "origin");
    let work_phase = variable(&m, "work_phase");
    let reported_phase = variable(&m, "reported_phase");
    let explicit_phase = variable(&m, "explicit_phase");
    let child = variable(&m, "child");
    assert!(
        report
            .states
            .iter()
            .any(|s| s[phase] == 2 && s[profile] == 2 && s[probe] == 1)
    );
    for state in [1, 2] {
        assert!(
            report
                .states
                .iter()
                .any(|s| s[phase] == state && s[age] == 1)
        );
        assert!(
            report
                .states
                .iter()
                .enumerate()
                .any(|(i, s)| s[phase] == state
                    && s[age] == 1
                    && report.edges[i].iter().any(|edge| {
                        let next = &report.states[edge.to];
                        next[phase] == state && next[age] == 0 && next[heartbeat] != s[heartbeat]
                    })),
            "no heartbeat in active state {state}"
        );
    }
    for source in [1, 2] {
        assert!(
            report
                .states
                .iter()
                .enumerate()
                .any(|(i, s)| s[origin] == source
                    && s[age] == 1
                    && report.edges[i].iter().any(|edge| {
                        let next = &report.states[edge.to];
                        next[origin] == source && next[age] == 0 && next[heartbeat] != s[heartbeat]
                    })),
            "origin {source} never refreshes"
        );
        for phase_value in 0..3 {
            assert!(
                report
                    .states
                    .iter()
                    .enumerate()
                    .any(|(i, s)| s[origin] == source
                        && s[phase] == 1
                        && s[work_phase] != phase_value
                        && report.edges[i].iter().any(|edge| {
                            let next = &report.states[edge.to];
                            next[work_phase] == phase_value
                                && next[reported_phase] == phase_value
                                && next[age] == s[age]
                                && next[heartbeat] == s[heartbeat]
                        })),
                "origin {source} phase {phase_value} waits for a heartbeat"
            );
        }
    }
    assert!(
        report
            .states
            .iter()
            .enumerate()
            .any(|(i, s)| s[explicit_phase] == 1
                && s[work_phase] == 1
                && s[child] == 0
                && report.edges[i].iter().any(|edge| {
                    let next = &report.states[edge.to];
                    next[child] == 1 && next[work_phase] == 1 && next[reported_phase] == 1
                })),
        "child arrival never preserves explicit Writing"
    );
    for (i, s) in report
        .states
        .iter()
        .enumerate()
        .filter(|(_, s)| s[phase] == 0 || s[phase] == 3)
    {
        assert!(
            report.edges[i]
                .iter()
                .all(|e| report.states[e.to][heartbeat] == s[heartbeat]),
            "heartbeat while idle/error"
        );
    }
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
        ("BrokenHarnessSeatReportHeartbeat", "WorkingReportFresh"),
        ("BrokenHarnessSeatReportPhase", "PhaseReportedImmediately"),
        ("BrokenHarnessSeatReportChild", "PhaseReportedImmediately"),
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
fn shutdown_watch_covers_a_late_fork_and_output_blocked_physical_eof() {
    let m = model("HarnessSeatShutdown");
    let member = variable(&m, "member");
    let fork = variable(&m, "fork");
    let phase = variable(&m, "phase");
    let closing = variable(&m, "closing");
    let source = variable(&m, "source");
    let blocked = variable(&m, "output_blocked");
    let checked = m.check(100_000).unwrap();
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[fork] == 2 && s[closing] == 1 && s[member] == 0)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[source] == 2 && s[blocked] == 1 && s[phase] == 5)
    );
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[source] == 3 && s[blocked] == 1 && s[phase] == 5)
    );
    assert!(checked.states.iter().any(|s| s[member] == 3));
    for (name, property) in [
        (
            "BrokenHarnessSeatShutdownSingleKill",
            "NoExecutableMembersAfterSupervisorKill",
        ),
        (
            "BrokenHarnessSeatShutdownBlockedOutput",
            "ShutdownObservationIndependentOfOutput",
        ),
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

#[test]
fn socket_lifecycle_controls_reach_restart_aba_cancel_duplicate_and_detach() {
    let good = model("HarnessSocketLifecycle");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    for (name, value) in [
        ("birth", 2),
        ("epoch", 1),
        ("job", 2),
        ("detached", 1),
        ("applied", 1),
    ] {
        assert!(
            checked
                .states
                .iter()
                .any(|s| s[variable(&good, name)] == value),
            "{name} is vacuous"
        );
    }
    for (suffix, property) in [
        ("Stale", "NoStaleCompletion"),
        ("Duplicate", "NoDuplicateCompletion"),
        ("Cancelled", "NoCancelledCompletion"),
        ("Detach", "DetachPreservesSession"),
    ] {
        let broken = model(&format!("BrokenHarnessSocketLifecycle{suffix}"));
        assert!(
            broken
                .check(100_000)
                .unwrap()
                .violations
                .iter()
                .any(|v| v.property == property),
            "{suffix} control did not fail {property}"
        );
    }
}

#[test]
fn stale_socket_intent_control_keeps_a_real_pending_owner_and_breaks_retirement() {
    let m = model("BrokenHarnessSocketLifecycleStalePending");
    let checked = m.check(100_000).unwrap();
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[variable(&m, "pending")] == 1
                && s[variable(&m, "birth")] == 2
                && s[variable(&m, "job_birth")] == 0
                && s[variable(&m, "detached")] == 1)
    );
    assert!(
        checked
            .violations
            .iter()
            .any(|v| v.property == "DetachedOldIntentEventuallyRetired")
    );
}

#[test]
fn four_view_budget_defers_a_real_tail_and_the_missing_guard_exceeds_the_share() {
    let good = model("HarnessFourViewBudget");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[variable(&good, "drain_spent")] == 3 && s[variable(&good, "todo3")] == 1)
    );
    assert!(checked.states.iter().any(
        |s| s[variable(&good, "projection_spent")] == 6 && s[variable(&good, "painted3")] == 0
    ));
    assert!(checked.states.iter().any(|s| {
        (0..4).all(|i| {
            s[variable(&good, &format!("painted{i}"))] == 1
                && s[variable(&good, &format!("todo{i}"))] == 0
        })
    }));
    let bad = model("BrokenHarnessFourViewBudgetTail")
        .check(100_000)
        .unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property == "AggregateProjectionBudget")
    );
}

#[test]
fn idle_status_reaches_durable_only_consumers_without_claiming_progress() {
    let good = model("HarnessSocketHeartbeat");
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty());
    assert!(checked.states.iter().any(|s| {
        s[variable(&good, "clock")] == 21
            && s[variable(&good, "ephemeral")] == 0
            && s[variable(&good, "delivered")] == 1
            && s[variable(&good, "transport_age")] == 6
            && s[variable(&good, "semantic_age")] == 21
    }));
    assert!(
        checked
            .states
            .iter()
            .any(|s| s[variable(&good, "item_delivered")] == 1)
    );
    let bad = model("BrokenHarnessSocketHeartbeat")
        .check(100_000)
        .unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property == "UnconditionalTransportStatus")
    );
    assert!(bad.states.iter().any(|s| s[variable(&good, "clock")] == 21 && s[variable(&good, "transport_age")] == 21));
}

#[test]
fn dream_sink_preserves_check_publish_races_and_labels_foreign_edits() {
    let good = model("HarnessDreamSink").check(100_000).unwrap();
    assert!(
        good.violations.is_empty(),
        "{:?}",
        good.violations
            .iter()
            .map(|v| &v.property)
            .collect::<Vec<_>>()
    );
    assert!(
        good.states.iter().any(|s| s[4] == 2 && s[0] > s[6]),
        "hand edit after the digest check must be reachable"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[4] == 3 && s[0] > s[6] && s[1] == s[2]),
        "immutable publication preserves the hand edit in the race window"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[4] == 4 && s[9] == 2 && s[14] == 1 && s[15] == 0),
        "no-replace collision must refuse rather than claim publication"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[15] == 1 && s[9] == 2 && s[12] == 0 && s[13] == 1),
        "legitimate edit after publication must change the ownership label"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 2 && s[1] == 0 && s[5] == 3 && s[4] == 3),
        "two invalidated reads, including ABA, leave the third attempt available"
    );
    for snapshot in 0..=3 {
        assert!(good.states.iter().any(|s| s[15] == 1 && s[11] == snapshot));
    }
    assert!(
        good.states
            .iter()
            .any(|s| s[15] == 1 && s[17] == 0 && s[19] == 1),
        "crash after publication before Adoption must be reachable"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[15] == 1 && s[17] == 1 && s[19] == 2),
        "restart can record a reachable existing publication"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[22] == 1 && s[4] == 4 && s[17] == 0 && s[15] == 0),
        "a pre-existing shared index must skip with no false Adoption"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[17] == 1 && s[11] > 0 && s[16] == 0 && s[20] == 0),
        "terminal entries must be omitted even when their old artifact remains"
    );
    for (mode, property) in [(2, "NoFalseAdoption"), (3, "ServedPinsValid")] {
        let bad = model(if mode == 2 {
            "BrokenHarnessDreamSinkAdoption"
        } else {
            "BrokenHarnessDreamSinkLoad"
        })
        .check(100_000)
        .unwrap();
        assert!(bad.violations.iter().any(|v| v.property == property));
    }
    let bad = model("BrokenHarnessDreamSink").check(100_000).unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property == "HandLinesPreserved")
    );
    assert!(
        bad.states
            .iter()
            .any(|s| s[4] == 3 && s[0] > s[6] && s[1] != s[2])
    );
}

#[test]
fn dream_lifecycle_reaches_independent_harnesses_crash_and_revival() {
    let graph = model("HarnessDreamLifecycle").check(100_000).unwrap();
    assert!(graph.violations.is_empty());
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[5] == 2 && s[12] == 1 && s[13] == 3),
        "one harness adopts while another remains inconclusive"
    );
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[2] == 1 && s[5] == 2 && s[10] == 1)
    );
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[0] == 1 && s[5] == 2 && s[10] == 2)
    );
    assert!(
        graph.states.iter().any(|s| s[0] == 1 && s[19] == 1),
        "the old adoption must survive global terminal and revival"
    );
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[4] == 1 && s[5] == 2 && s[10] == 0 && s[11] == 0),
        "global terminal overlays an adopted pair and lower removes its sink"
    );
    for (mode, property) in [
        (1, "AdoptedHasSamePairPassA"),
        (2, "GlobalTerminalFencesDelayedAdoption"),
        (3, "AdoptedHasSamePairPassA"),
    ] {
        let bad = model(match mode {
            1 => "BrokenHarnessDreamLifecycle",
            2 => "BrokenHarnessDreamLifecycleFence",
            _ => "BrokenHarnessDreamLifecyclePair",
        })
        .check(100_000)
        .unwrap();
        assert!(
            bad.violations.iter().any(|v| v.property == property),
            "{mode}: {property}"
        );
        if mode == 2 {
            assert!(
                bad.states
                    .iter()
                    .any(|s| s[0] == 1 && s[5] == 2 && s[8] == 1 && s[22] == 0),
                "old passing completion must actually reach a new revival in the control"
            );
        }
        if mode == 3 {
            assert!(
                bad.states
                    .iter()
                    .any(|s| s[7] == 1 && s[14] == 2 && s[8] == 2 && s[5] == 2),
                "wrong harness trial pin must be rejected even when both harnesses passed"
            );
        }
    }
}

#[test]
fn dream_merge_explores_both_orders_and_independent_harnesses() {
    let graph = model("HarnessDreamMerge").check(100_000).unwrap();
    assert!(graph.violations.is_empty());
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[8] == 2 && s[9] == 3 && s[12] == 1),
        "B rejection cannot erase A adoption"
    );
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[2] == 2 && s[7] == 1 && s[8] == 5 && s[9] == 5 && s[12] == 1),
        "an expiry in the other log fences an adoption"
    );
    assert!(
        graph
            .states
            .iter()
            .any(|s| s[4] == 1 && s[7] == 2 && s[8] == 6 && s[12] == 1),
        "global terminal display uses fixed superseded-over-expired order"
    );
    // Extract the actual model's join table, without reimplementing max.
    // A global terminal makes the pair display details irrelevant; this
    // quotient has 25 independent pair states plus expiry and supersession.
    let normalize = |a, b, g| if g == 0 { (a, b, g) } else { (0, 0, g) };
    let mut joins = std::collections::BTreeMap::new();
    for state in graph.states.iter().filter(|s| s[12] == 1) {
        let key = (
            normalize(state[2], state[3], state[4]),
            normalize(state[5], state[6], state[7]),
        );
        let value = normalize(state[8], state[9], state[10]);
        if let Some(previous) = joins.insert(key, value) {
            assert_eq!(previous, value);
        }
    }
    let mut domain = Vec::new();
    for a in 0..=4 {
        for b in 0..=4 {
            domain.push((a, b, 0));
        }
    }
    domain.extend([(0, 0, 1), (0, 0, 2)]);
    assert_eq!(
        joins.len(),
        domain.len() * domain.len(),
        "every quotient input pair must be reachable"
    );
    for &a in &domain {
        assert_eq!(joins[&(a, a)], a, "idempotence");
        for &b in &domain {
            assert_eq!(joins[&(a, b)], joins[&(b, a)], "commutativity");
            for &c in &domain {
                assert_eq!(
                    joins[&(joins[&(a, b)], c)],
                    joins[&(a, joins[&(b, c)])],
                    "associativity"
                );
            }
        }
    }
    for mode in [1, 2] {
        let bad = model(if mode == 1 {
            "BrokenHarnessDreamMerge"
        } else {
            "BrokenHarnessDreamMergeHarness"
        })
        .check(100_000)
        .unwrap();
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == "CommutativeIdempotentProjection")
        );
    }
}

#[test]
fn dream_d0_claim_union_crash_and_detached_caller_are_reachable() {
    let m = model("HarnessDreamD0");
    let g = m.check(100_000).unwrap();
    assert!(
        g.violations.is_empty(),
        "{:?}",
        g.violations.iter().map(|v| &v.property).collect::<Vec<_>>()
    );
    assert!(
        g.states
            .iter()
            .any(|s| s[0] == 3 && s[2] == 2 && s[18] == 1),
        "duplicate union of two distinct pins never reached"
    );
    assert!(
        g.states
            .iter()
            .any(|s| s[6] == 1 && s[9] == 7 && s[16] == 1),
        "busy B must skip and return while A still owns/stalls"
    );
    assert!(
        g.states.iter().enumerate().any(|(i, s)| s[6] == 1
            && s[9] == 1
            && s[10] == 1
            && g.edges[i].iter().any(|e| {
                let n = &g.states[e.to];
                n[6] == 1 && n[7] == 2 && n[9] == 2
            })),
        "other shard cannot acquire independently"
    );
    for phase in [3, 5] {
        assert!(
            g.states.iter().enumerate().any(|(i, s)| s[8] == phase
                && s[17] == 0
                && g.edges[i].iter().any(|e| {
                    let n = &g.states[e.to];
                    n[8] == 8 && n[6] == 0 && n[17] == 1 && n[0] == s[0] && n[4] == s[4]
                })),
            "crash before/after publication must release without changing committed bytes"
        );
    }
    let lost = model("BrokenHarnessDreamD0").check(100_000).unwrap();
    assert!(
        lost.violations
            .iter()
            .any(|v| v.property == "CommittedPinsPreserved0")
    );
    let blocked = model("BrokenHarnessDreamD0Blocking")
        .check(100_000)
        .unwrap();
    assert!(
        blocked
            .violations
            .iter()
            .any(|v| v.property == "DetachedCaller1" && !v.cycle.is_empty())
    );
    assert!(
        blocked.states.iter().enumerate().any(|(i, s)| s[6] == 1
            && s[9] == 1
            && s[16] == 0
            && blocked.edges[i].iter().any(|e| {
                let n = &blocked.states[e.to];
                n[6] == 1 && n[9] == 9 && n[16] == 0
            })),
        "blocking control must enter real busy waiting from a reachable owned shard"
    );
}

#[test]
fn dream_native_custody_never_pins_staging_and_reuses_complete_survivors() {
    let m = model("HarnessDreamNativeCustody");
    let g = m.check(100_000).unwrap();
    assert!(g.violations.is_empty());
    assert!(
        g.states
            .iter()
            .any(|s| s[5] == 1 && s[6] == 1 && s[11] == 1),
        "two materializers must reuse one complete publication"
    );
    assert!(
        g.states
            .iter()
            .any(|s| s[1] == 7 && s[7] == 1 && s[4] == 0 && s[5] == 0),
        "crash before publication has no pins"
    );
    assert!(
        g.states
            .iter()
            .any(|s| s[7] == 1 && s[4] == 1 && (s[5] == 1 || s[6] == 1)),
        "complete publication survives owner crash/replay"
    );
    assert!(
        g.states.iter().enumerate().any(|(i, s)| s[1] == 3
            && s[4] == 0
            && g.edges[i]
                .iter()
                .any(|e| g.states[e.to][4] == 2 && g.states[e.to][1] == 3)),
        "foreign target birth must remain enabled after check"
    );
    let bad = model("BrokenHarnessDreamNativeCustody")
        .check(100_000)
        .unwrap();
    assert!(
        bad.violations
            .iter()
            .any(|v| v.property.starts_with("NoPrematureNativePin"))
    );
    assert!(
        bad.states
            .iter()
            .any(|s| s[5] == 1 && s[4] == 0 && s[3] == 0),
        "control must really pin private, unpublished bytes"
    );
    println!(
        "native custody: {} states, {} labeled edges; premature-pin control rejected",
        g.states.len(),
        g.edges.iter().map(Vec::len).sum::<usize>()
    );
}
