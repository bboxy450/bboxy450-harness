// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Finite Q2 producer contracts. Evidence facts abstract existing verifiers;
//! bytes, signatures, bounded process IO and allocator behavior remain witnesses.
use super::*;

/// Decision66050 refines one already-consumed STOP in the existing controller.
/// The immutable profile selects independent observations, never an errno cause.
/// Owner: original unreaped leader/group. Bounds: one grant, at most two members,
/// exact/replaced token classes, failed-call endpoint1 and positive endpoint2. Q classes
/// 1/2 mean within/over the unchanged1ms bound; debt1 means the full fresh9x rest.
/// Native observations are not fair. Only enabled local classification/payment
/// is weakly fair; explicit deadline preserves ownership in quarantine.
/// The identity fact abstracts current live-token/image/RI checks and sticky
/// observed exec refusal. RI v1 has no terminal pidversion: final-image exit
/// continuity also depends on the existing reviewed no-further-exec premise.
/// An unobserved transient exec is not independently excluded by this model.
pub fn failed_stop_terminal(control: u8) -> Model {
    let suffix = ["", "ErrnoAck", "RetryStop", "Resume", "LeaderOnly", "Roster",
        "Identity", "FailedEndpoint", "Controller", "Pipe", "Late", "Quantum",
        "Clock", "Debt", "Reap", "UnknownExit", "WorkWhilePending", "PayloadWhilePending"][control as usize];
    let mut m = base(&format!("DreamReadFailedStop{suffix}"), control != 0, &[
        ("phase", 6, 0), ("profile", 13, 0), ("rawErrnoEpermEsrch", 2, 0),
        ("consumedStopAttempts", 2, 0), ("leaderWnowaitOriginal", 1, 0),
        ("everyMemberPositiveExit", 1, 0), ("causallyClosedRoster", 1, 0),
        ("exactRetainedIncarnation", 1, 0), ("windowEndFailedOrPositive", 2, 0),
        ("controllerIncluded", 1, 0), ("pipeIncluded", 1, 0),
        ("lateAndRetiredIncluded", 1, 0), ("qWithinOrOver1ms", 2, 0),
        ("clockKnownOrdered", 1, 0), ("freshRestDebt", 1, 0),
        ("actualExactWait4AndGroupSettlement", 1, 0),
        ("stoppedAck", 1, 0), ("continueCount", 1, 0), ("proofOrPayloadWork", 1, 0),
        ("terminalClassified", 1, 0), ("ownershipReleased", 1, 0),
    ]);
    const P: usize = 0; const CASE: usize = 1; const ERR: usize = 2;
    const ATT: usize = 3; const LEAD: usize = 4; const MEMBERS: usize = 5;
    const ROSTER: usize = 6; const ID: usize = 7; const END: usize = 8;
    const CTL: usize = 9; const PIPE: usize = 10; const LATE: usize = 11;
    const Q: usize = 12; const CLOCK: usize = 13; const DEBT: usize = 14;
    const REAP: usize = 15; const ACK: usize = 16; const CONT: usize = 17;
    const WORK: usize = 18; const TERMINAL: usize = 19; const RELEASE: usize = 20;
    for profile in 0..=13 {
        for errno in 1..=2 {
            act(&mut m, &format!("ConsumeOriginalStopProfile{profile}Errno{errno}"), eq(P, 0), &[
                (P, Int(1)), (CASE, Int(profile)), (ERR, Int(errno)),
                (ATT, Int(1)), (DEBT, Int(1)),
                (ACK, Int(i64::from(control == 1))),
            ]);
        }
    }
    // A failed signal alone changes none of the independent physical facts.
    // Profiles cover runnable leader, live descendant, unknown slot, replaced
    // token/exec, failed endpoint, omissions, overrun, clock and unavailable reap.
    for profile in 0..=13 {
        act(&mut m, &format!("ObserveIndependentOriginalTerminalFacts{profile}"),
            all([eq(P, 1), eq(CASE, profile)]), &[
                (P, Int(2)),
                (LEAD, Int(i64::from(profile != 1 && profile != 13))),
                (MEMBERS, Int(i64::from(profile != 2))),
                (ROSTER, Int(i64::from(profile != 3))),
                (ID, Int(i64::from(profile != 4))),
                (END, Int(if profile == 5 { 1 } else { 2 })),
                (CTL, Int(i64::from(profile != 6))),
                (PIPE, Int(i64::from(profile != 7))),
                (LATE, Int(i64::from(profile != 8))),
                (Q, Int(if profile == 9 { 2 } else { 1 })),
                (CLOCK, Int(i64::from(profile != 10))),
            ]);
    }
    let predicates = [(LEAD, 1, 15), (MEMBERS, 1, 4), (ROSTER, 1, 5),
        (ID, 1, 6), (END, 2, 7), (CTL, 1, 8), (PIPE, 1, 9), (LATE, 1, 10),
        (Q, 1, 11), (CLOCK, 1, 12)];
    let exact = all(predicates.iter().map(|&(v, val, _)| eq(v, val)));
    let admitted = all(predicates.iter().map(|&(v, val, mutant)|
        if control == mutant { Bool(true) } else { eq(v, val) }));
    let classify = m.actions.len();
    act(&mut m, "ClassifyIndependentTerminalWithoutSignalAck",
        all([eq(P, 2), admitted]), &[(P, Int(3)), (TERMINAL, Int(1))]);
    let pay = m.actions.len();
    act(&mut m, "PayFullFreshDebtAfterPositiveEndpoint",
        all([eq(P, 3), eq(DEBT, 1), eq(CASE, 11).negate()]), &[(DEBT, Int(0))]);
    for actual in 0..=1 {
        act(&mut m, &format!("ObserveActualOriginalWait4AndGroupSettlement{actual}"),
            all([eq(P, 3), if actual == 1 { eq(CASE, 12).negate() }
                else { eq(CASE, 12) }]), &[(P, Int(4)), (REAP, Int(actual))]);
    }
    act(&mut m, "CompleteOnlyAfterActualSettlementAndPaidDebt", all([
        eq(P, 4), if control == 13 { Bool(true) } else { eq(DEBT, 0) },
        if control == 14 { Bool(true) } else { eq(REAP, 1) },
    ]), &[(P, Int(5)), (RELEASE, Int(1))]);
    act(&mut m, "PayRemainingOriginalDebtAfterReap", all([
        eq(P, 4), eq(DEBT, 1), eq(CASE, 11).negate(),
    ]), &[(DEBT, Int(0))]);
    for (mutant, name, slot) in [(2, "BrokenRepeatConsumedStop", ATT),
        (3, "BrokenContinueWhileTerminalPending", CONT),
        (16, "BrokenProofWorkWhileTerminalPending", WORK),
        (17, "BrokenPayloadWriteWhileTerminalPending", WORK)] {
        if control == mutant {
            act(&mut m, name, eq(P, 1), &[(slot, Int(if slot == ATT { 2 } else { 1 }))]);
        }
    }
    act(&mut m, "DeadlineRetainsOriginalOwnedQuarantine",
        all([Int(0).lt(Var(P)), Var(P).lt(Int(5))]), &[(P, Int(6))]);
    safe(&mut m, "ConsumedFailureNeverBecomesStopAck", eq(ACK, 0));
    safe(&mut m, "PendingTerminalNeverRepeatsStopOrContinuesOrWorks",
        all([eq(ATT, 2).negate(), eq(CONT, 0), eq(WORK, 0)]));
    safe(&mut m, "TerminalNeedsEveryIndependentOriginalFact",
        eq(TERMINAL, 0).or(exact.clone()));
    safe(&mut m, "ReleaseRequiresActualSettlementAndFullPaidDebt",
        eq(RELEASE, 0).or(all([eq(TERMINAL, 1), eq(REAP, 1), eq(DEBT, 0)])));
    safe(&mut m, "QuarantineKeepsOriginalEffectAndRawSignalReceipt",
        eq(P, 0).or(all([eq(ATT, 0).negate(), eq(ERR, 0).negate(),
            eq(P, 6).negate().or(eq(RELEASE, 0))])));
    progress(&mut m, "KnownTerminalCanBeClassifiedWithoutAnotherSignal",
        all([eq(P, 2), exact]), eq(TERMINAL, 1).or(eq(P, 6)), vec![classify]);
    // No fairness of successful rest clock, WNOWAIT, RI/audit, wait4 or kernel.
    m.weak_fair.push(vec![pay]);
    m
}

#[cfg(test)]
mod failed_stop_terminal_tests {
    use super::*;

    #[test]
    fn independent_terminal_facts_and_all_broken_controls_export_complete_graphs() {
        let output = std::env::var_os("DREAM_FAILED_STOP_FORMAL_OUT")
            .map(std::path::PathBuf::from);
        if let Some(root) = &output {
            std::fs::create_dir_all(root).unwrap();
        }
        let mut manifest = String::new();
        for control in 0..=17 {
            let model = failed_stop_terminal(control);
            let checked = model.check(100_000).unwrap();
            assert!(checked.violations.iter().all(|v| v.property != "TypeOK"));
            if control == 0 {
                assert!(checked.violations.is_empty());
                assert!(checked.states.iter().any(|s| {
                    s[0] == 1 && s[2] == 1 && s[3] == 1 && s[4] == 0
                        && s[16] == 0 && s[17] == 0 && s[18] == 0 && s[20] == 0
                }), "consumed EPERM is pending without physical terminal facts");
                assert!(checked.states.iter().any(|s| {
                    s[0] == 1 && s[2] == 2 && s[3] == 1 && s[4] == 0
                        && s[16] == 0 && s[17] == 0 && s[18] == 0 && s[20] == 0
                }), "consumed ESRCH is pending without physical terminal facts");
                assert!(checked.states.iter().any(|s| {
                    s[0] == 5 && s[2] != 0 && s[3] == 1 && s[14] == 0
                        && s[15] == 1 && s[19] == 1 && s[20] == 1
                }), "positive independent proof can pay and actually settle without another signal");
                assert!(checked.states.iter().any(|s| {
                    s[0] == 6 && s[2] != 0 && s[3] == 1 && s[20] == 0
                }), "deadline retains raw failure and original owned quarantine");
            } else {
                assert!(!checked.violations.is_empty(), "control {control} must fail causally");
            }
            println!(
                "failed_stop_terminal control={control} model={} states={} labelled_edges={} named_verdicts={:?}",
                model.name,
                checked.states.len(),
                checked.edges.iter().map(Vec::len).sum::<usize>(),
                checked.violations.iter().map(|v| &v.property).collect::<Vec<_>>()
            );
            if let Some(root) = &output {
                super::helper_observer_lowering_export::export(
                    model,
                    if control == 0 { "pass" } else { "safety" },
                    root,
                    &mut manifest,
                ).unwrap();
            }
        }
        if let Some(root) = output {
            std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
        }
    }
}

/// Decision66074 refines the existing helper's refused owned quarantine. The
/// SAME controller holds the original FDs and unreaped leader; this is not a
/// second scheduler, custody recipient or runtime approval path. One captured
/// group reserves one/two members; profiles are one known member, two known
/// members, and an observed second member whose identity remains unknown.
/// Work deadline expiry never releases custody. Native clock/terminal/wait4/
/// group observations are not fair; only local refusal/payment/release actions
/// are weakly fair. External destruction of the controller is not prevented by
/// this actor model: parent/session loss is recorded honestly while it lives.
pub fn quarantine_owner(control: u8) -> Model {
    let suffix = ["", "ReleaseFd", "Exit"][control as usize];
    let mut m = base(
        &format!("DreamReadQuarantineOwner{suffix}"),
        control != 0,
        &[
            ("phase", 5, 0),
            ("memberProfile", 2, 0),
            ("sameWorkOwner", 1, 1),
            ("originalFdsHeld", 1, 1),
            ("controllerAlive", 1, 1),
            ("originalUnreapedPinHeld", 1, 1),
            ("consumedMemberReservations", 2, 1),
            ("knownOriginalMembers", 2, 1),
            ("unknownAdmittedMember", 1, 0),
            ("clockKnownOrdered", 1, 1),
            ("freshObservationDebt", 1, 1),
            ("allOriginalTerminalFacts", 1, 0),
            ("ownActualWait4", 1, 0),
            ("originalGroupGone", 1, 0),
            ("refusalWireAttempt", 2, 0),
            ("parentOrSessionGone", 1, 0),
            ("ownerLostRecorded", 1, 0),
            ("qualifiedSuccess", 1, 0),
            ("replacementRun", 1, 0),
            ("originalWorkDeadlineExpired", 1, 1),
        ],
    );
    const P: usize = 0;
    const OWNER: usize = 2;
    const FD: usize = 3;
    const ALIVE: usize = 4;
    const PIN: usize = 5;
    const RESERVED: usize = 6;
    const KNOWN: usize = 7;
    const UNKNOWN: usize = 8;
    const CLOCK: usize = 9;
    const DEBT: usize = 10;
    const TERMINAL: usize = 11;
    const REAP: usize = 12;
    const GROUP: usize = 13;
    const REFUSAL: usize = 14;
    const PARENT: usize = 15;
    const LOST: usize = 16;
    const QUALIFIED: usize = 17;
    const NEW_RUN: usize = 18;
    for profile in 0..=2 {
        act(
            &mut m,
            &format!("CaptureExistingOriginalGroupProfile{profile}"),
            eq(P, 0),
            &[
                (P, Int(1)),
                (1, Int(profile)),
                (RESERVED, Int(if profile == 0 { 1 } else { 2 })),
                (KNOWN, Int(if profile == 1 { 2 } else { 1 })),
                (UNKNOWN, Int(i64::from(profile == 2))),
            ],
        );
    }
    let mut publish = Vec::new();
    for result in 1..=2 {
        publish.push(m.actions.len());
        act(
            &mut m,
            &format!("AttemptExistingRefusalWireOnceResult{result}"),
            eq(P, 1),
            &[(P, Int(2)), (REFUSAL, Int(result)), (DEBT, Int(1))],
        );
    }
    let holding = all([Int(1).lt(Var(P)), Var(P).lt(Int(5))]);
    for known in 0..=1 {
        act(
            &mut m,
            &format!("ObserveHoldClockKnownOrdered{known}"),
            holding.clone(),
            &[(CLOCK, Int(known)), (DEBT, Int(1))],
        );
    }
    let pay = m.actions.len();
    act(
        &mut m,
        "PayFreshHoldObservationDebt",
        all([holding.clone(), eq(CLOCK, 1), eq(DEBT, 1)]),
        &[(DEBT, Int(0))],
    );
    let members = all([
        eq(UNKNOWN, 0),
        Var(KNOWN).eq(Var(RESERVED)),
        Int(0).lt(Var(KNOWN)),
    ]);
    act(
        &mut m,
        "ObserveEveryOriginalTerminalWithNonvacuousClosedRoster",
        all([
            eq(P, 2),
            eq(CLOCK, 1),
            eq(DEBT, 0),
            members.clone(),
            eq(PIN, 1),
            eq(ALIVE, 1),
            eq(FD, 1),
        ]),
        &[(P, Int(3)), (TERMINAL, Int(1)), (DEBT, Int(1))],
    );
    act(
        &mut m,
        "SameControllerActuallyReapsOriginalLeader",
        all([
            eq(P, 3),
            eq(CLOCK, 1),
            eq(DEBT, 0),
            eq(TERMINAL, 1),
            members.clone(),
            eq(PIN, 1),
            eq(ALIVE, 1),
        ]),
        &[(P, Int(4)), (REAP, Int(1)), (PIN, Int(0)), (DEBT, Int(1))],
    );
    act(
        &mut m,
        "ObserveOriginalGroupGoneWithoutPostReapSignal",
        all([
            eq(P, 4),
            eq(CLOCK, 1),
            eq(DEBT, 0),
            eq(REAP, 1),
            eq(TERMINAL, 1),
            members.clone(),
        ]),
        &[(GROUP, Int(1)), (DEBT, Int(1))],
    );
    act(
        &mut m,
        "ParentOrSessionLossRecordsOwnerLostWithoutRelease",
        all([holding.clone(), eq(PARENT, 0)]),
        &[(PARENT, Int(1)), (LOST, Int(1)), (DEBT, Int(1))],
    );
    act(
        &mut m,
        "ExpiredWorkDeadlineCannotReleaseHeldCustody",
        holding.clone(),
        &[],
    );
    let disposed = all([
        eq(TERMINAL, 1),
        eq(REAP, 1),
        eq(GROUP, 1),
        eq(DEBT, 0),
        members.clone(),
    ]);
    let release = m.actions.len();
    act(
        &mut m,
        "ReleaseOnlyPhysicallySettledRefusalAfterPaidDebt",
        all([eq(P, 4), eq(CLOCK, 1), disposed.clone()]),
        &[(P, Int(5)), (OWNER, Int(0)), (FD, Int(0)), (ALIVE, Int(0))],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenReleaseOriginalFdWhileQuarantined",
            eq(P, 2),
            &[(FD, Int(0))],
        );
    }
    if control == 2 {
        // A stale durable claim may still say owner/FD held after its process
        // exits. That claim cannot substitute for this live-controller fact.
        act(
            &mut m,
            "BrokenControllerExitAtExpiredWorkDeadline",
            eq(P, 2),
            &[(ALIVE, Int(0))],
        );
    }
    safe(
        &mut m,
        "OwnedQuarantineKeepsLiveControllerAndFdsUntilActualSettlement",
        disposed
            .clone()
            .or(all([eq(OWNER, 1), eq(FD, 1), eq(ALIVE, 1)])),
    );
    safe(
        &mut m,
        "OriginalUnreapedPinSurvivesUntilOwnActualWait4",
        eq(REAP, 1).or(eq(PIN, 1)),
    );
    safe(
        &mut m,
        "SettledRefusalHasOwnReapAllMembersGroupGoneAndPaidDebt",
        eq(P, 5).negate().or(disposed),
    );
    safe(
        &mut m,
        "UnknownAdmittedMemberCannotAcquireVacuousSettlement",
        eq(UNKNOWN, 0).or(all([
            eq(TERMINAL, 0),
            eq(REAP, 0),
            eq(GROUP, 0),
            eq(P, 5).negate(),
        ])),
    );
    safe(
        &mut m,
        "ParentLossIsHonestAndNeverQualifiesOrCreatesReplacement",
        all([
            eq(QUALIFIED, 0),
            eq(NEW_RUN, 0),
            eq(PARENT, 0).or(eq(LOST, 1)),
        ]),
    );
    safe(
        &mut m,
        "RefusalWireAttemptPrecedesHoldAndSettlement",
        Var(P).lt(Int(2)).or(eq(REFUSAL, 0).negate()),
    );
    progress(
        &mut m,
        "RefusalAttemptDoesNotWaitForPhysicalSettlement",
        eq(P, 1),
        eq(REFUSAL, 0).negate(),
        publish,
    );
    m.weak_fair.push(vec![pay]);
    m.weak_fair.push(vec![release]);
    m
}

#[cfg(test)]
mod quarantine_owner_tests {
    use super::*;

    #[test]
    fn live_original_custody_and_broken_release_exit_export_complete_graphs() {
        let output = std::env::var_os("DREAM_QUARANTINE_FORMAL_OUT").map(std::path::PathBuf::from);
        if let Some(root) = &output {
            std::fs::create_dir_all(root).unwrap();
        }
        let mut manifest = String::new();
        for control in 0..=2 {
            let model = quarantine_owner(control);
            let checked = model.check(100_000).unwrap();
            assert!(checked.violations.iter().all(|v| v.property != "TypeOK"));
            if control == 0 {
                assert!(checked.violations.is_empty());
                assert!(
                    checked.states.iter().any(|s| s[0] == 2
                        && s[14] == 1
                        && s[2] == 1
                        && s[3] == 1
                        && s[4] == 1
                        && s[5] == 1
                        && s[12] == 0),
                    "prompt refusal leaves the same live owner/FDs/unreaped original"
                );
                assert!(
                    checked.states.iter().any(|s| s[0] == 2
                        && s[8] == 1
                        && s[9] == 0
                        && s[2] == 1
                        && s[3] == 1
                        && s[4] == 1
                        && s[5] == 1),
                    "unknown admitted member or clock never releases custody"
                );
                assert!(
                    checked.states.iter().any(|s| s[0] == 4
                        && s[12] == 1
                        && s[13] == 0
                        && s[2] == 1
                        && s[3] == 1
                        && s[4] == 1
                        && s[5] == 0),
                    "actual leader reap alone does not release the owned group"
                );
                assert!(
                    checked.states.iter().any(|s| s[0] == 2
                        && s[15] == 1
                        && s[16] == 1
                        && s[2] == 1
                        && s[3] == 1
                        && s[4] == 1
                        && s[17] == 0
                        && s[18] == 0),
                    "parent/session loss remains owner-lost refusal with original custody"
                );
                assert!(
                    checked.states.iter().any(|s| s[0] == 5
                        && s[10] == 0
                        && s[11] == 1
                        && s[12] == 1
                        && s[13] == 1
                        && s[17] == 0
                        && s[18] == 0),
                    "own reap plus full group/member proof releases only a settled refusal"
                );
            } else {
                assert_eq!(
                    checked
                        .violations
                        .iter()
                        .map(|v| v.property.as_str())
                        .collect::<Vec<_>>(),
                    vec!["OwnedQuarantineKeepsLiveControllerAndFdsUntilActualSettlement"]
                );
            }
            println!(
                "quarantine_owner control={control} model={} states={} labelled_edges={} named_verdicts={:?}",
                model.name,
                checked.states.len(),
                checked.edges.iter().map(Vec::len).sum::<usize>(),
                checked
                    .violations
                    .iter()
                    .map(|v| &v.property)
                    .collect::<Vec<_>>()
            );
            if let Some(root) = &output {
                super::helper_observer_lowering_export::export(
                    model,
                    if control == 0 { "pass" } else { "safety" },
                    root,
                    &mut manifest,
                )
                .unwrap();
            }
        }
        if let Some(root) = output {
            std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
        }
    }
}

/// Initial creation admission only: three raw TASK states (quiescent, exact
/// SSTOP/one-thread/one-running kernel tail, invalid), two independent queries
/// and at most one fresh full-query generation. The eight lifecycle profiles
/// are initial, settled, ready, running, grant-open, user-growth, expired and
/// unknown final native cost. Runtime STOP sampling is modeled separately by
/// MemberWindow; these profiles test misuse of INITIAL pending authority.
/// OS success/deadline/clock observations are not fair. A pending second query
/// must return to a fresh first query, retaining this original effect and cost.
pub fn initial_quiescence(control: u8) -> Model {
    let suffix = ["", "AcceptPending", "RuntimePending", "SkipFresh"][control as usize];
    let mut m = base(
        &format!("DreamReadInitialQuiescence{suffix}"),
        control != 0,
        &[
            ("phase", 5, 0),
            ("lifecycleProfile", 7, 0),
            ("profileCaptured", 1, 0),
            ("firstRaw", 3, 0),
            ("secondRaw", 3, 0),
            ("firstGeneration", 1, 0),
            ("secondGeneration", 1, 0),
            ("queryGeneration", 1, 0),
            ("pendingSeen", 1, 0),
            ("originalOwner", 1, 1),
            ("userGrowthBeforeAdmission", 1, 0),
            ("originalDeadlineExpired", 1, 0),
            ("acknowledged", 1, 0),
            ("continued", 1, 0),
            ("fullCreationDebt", 1, 1),
            ("nativeCostKnownWithin", 1, 1),
            ("originalUnreapedPin", 1, 1),
        ],
    );
    const P: usize = 0;
    const CTX: usize = 1;
    const SELECTED: usize = 2;
    const FIRST: usize = 3;
    const SECOND: usize = 4;
    const FG: usize = 5;
    const SG: usize = 6;
    const GEN: usize = 7;
    const PENDING: usize = 8;
    const OWNER: usize = 9;
    const USER: usize = 10;
    const DEADLINE: usize = 11;
    const ACK: usize = 12;
    const CONT: usize = 13;
    const DEBT: usize = 14;
    const KNOWN: usize = 15;
    const PIN: usize = 16;
    let initial = eq(CTX, 0).or(eq(CTX, 7));
    let eligible = all([
        initial.clone(),
        eq(USER, 0),
        eq(DEADLINE, 0),
        eq(OWNER, 1),
        eq(PIN, 1),
    ]);
    for profile in 0..=7 {
        act(
            &mut m,
            &format!("CaptureOriginalCreationContext{profile}"),
            all([eq(P, 0), eq(SELECTED, 0)]),
            &[
                (CTX, Int(profile)),
                (SELECTED, Int(1)),
                (USER, Int(i64::from(profile == 5))),
                (DEADLINE, Int(i64::from(profile == 6))),
                (KNOWN, Int(i64::from(profile != 7))),
            ],
        );
    }
    let first = all([eq(P, 0), eq(SELECTED, 1), eq(DEADLINE, 0)]);
    act(
        &mut m,
        "ObserveFirstRawQuiescent",
        first.clone(),
        &[(P, Int(1)), (FIRST, Int(1)), (FG, Var(GEN))],
    );
    act(
        &mut m,
        "ObserveFirstExactKernelTailAsPending",
        all([first.clone(), eligible.clone()]),
        &[
            (P, Int(2)),
            (FIRST, Int(2)),
            (FG, Var(GEN)),
            (PENDING, Int(1)),
        ],
    );
    act(
        &mut m,
        "RefuseFirstKernelTailWithoutInitialAuthority",
        all([first.clone(), eligible.clone().negate()]),
        &[(P, Int(5)), (FIRST, Int(2)), (FG, Var(GEN))],
    );
    act(
        &mut m,
        "RefuseInvalidFirstRawTuple",
        first.clone(),
        &[(P, Int(5)), (FIRST, Int(3)), (FG, Var(GEN))],
    );
    let second = all([eq(P, 1), eq(DEADLINE, 0)]);
    act(
        &mut m,
        "ObserveIndependentSecondQuiescentAndAcknowledge",
        all([second.clone(), eligible.clone()]),
        &[(P, Int(3)), (SECOND, Int(1)), (SG, Var(GEN)), (ACK, Int(1))],
    );
    act(
        &mut m,
        "RefuseSecondQuiescenceWithoutCreationAuthority",
        all([second.clone(), eligible.clone().negate()]),
        &[(P, Int(5)), (SECOND, Int(1)), (SG, Var(GEN))],
    );
    act(
        &mut m,
        "ObserveSecondExactKernelTailWithoutAck",
        all([second.clone(), eligible.clone()]),
        &[
            (P, Int(2)),
            (SECOND, Int(2)),
            (SG, Var(GEN)),
            (PENDING, Int(1)),
        ],
    );
    act(
        &mut m,
        "RefuseRuntimeOrUserGrowthSecondKernelTail",
        all([second.clone(), eligible.clone().negate()]),
        &[(P, Int(5)), (SECOND, Int(2)), (SG, Var(GEN))],
    );
    act(
        &mut m,
        "RefuseInvalidSecondRawTuple",
        second.clone(),
        &[(P, Int(5)), (SECOND, Int(3)), (SG, Var(GEN))],
    );
    act(
        &mut m,
        "PendingRequiresFreshFullFirstAndSecondQueries",
        all([eq(P, 2), eq(GEN, 0), eligible.clone()]),
        &[
            (P, Int(0)),
            (GEN, Int(1)),
            (FIRST, Int(0)),
            (SECOND, Int(0)),
        ],
    );
    act(
        &mut m,
        "OriginalDeadlineRefusesWithoutExtensionOrReplacement",
        all([eq(SELECTED, 1), Var(P).lt(Int(4))]),
        &[(P, Int(5)), (DEADLINE, Int(1))],
    );
    act(
        &mut m,
        "ExpiredContextKeepsOriginalOwnedRefusal",
        all([eq(P, 0), eq(SELECTED, 1), eq(DEADLINE, 1)]),
        &[(P, Int(5))],
    );
    act(
        &mut m,
        "UnknownNativeCreationCostCannotGrantExecution",
        all([eq(P, 3), eq(KNOWN, 0)]),
        &[(P, Int(5))],
    );
    let pay = m.actions.len();
    act(
        &mut m,
        "PayFullFreshCreationDebtAfterObservedValidPair",
        all([eq(P, 3), eq(KNOWN, 1), eq(DEBT, 1), eq(DEADLINE, 0)]),
        &[(DEBT, Int(0))],
    );
    let cont = m.actions.len();
    act(
        &mut m,
        "ContinueOnlyAfterFullAdmissionAndPaidKnownCost",
        all([
            eq(P, 3),
            eq(ACK, 1),
            eq(KNOWN, 1),
            eq(DEBT, 0),
            eligible.clone(),
        ]),
        &[(P, Int(4)), (CONT, Int(1))],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenFirstKernelTailIsAcknowledged",
            all([first, eligible.clone()]),
            &[(P, Int(3)), (FIRST, Int(2)), (FG, Var(GEN)), (ACK, Int(1))],
        );
        act(
            &mut m,
            "BrokenSecondKernelTailIsAcknowledged",
            all([second.clone(), eligible.clone()]),
            &[(P, Int(3)), (SECOND, Int(2)), (SG, Var(GEN)), (ACK, Int(1))],
        );
    }
    if control == 2 {
        act(
            &mut m,
            "BrokenRuntimeKernelTailBorrowsInitialPendingAuthority",
            all([second, initial.clone().negate(), eq(USER, 0)]),
            &[
                (P, Int(2)),
                (SECOND, Int(2)),
                (SG, Var(GEN)),
                (PENDING, Int(1)),
            ],
        );
    }
    if control == 3 {
        act(
            &mut m,
            "BrokenOnlyFreshSecondQueryReusesCachedFirst",
            all([eq(P, 2), eq(FIRST, 1), eq(GEN, 0), eligible.clone()]),
            &[
                (P, Int(3)),
                (GEN, Int(1)),
                (SECOND, Int(1)),
                (SG, Int(1)),
                (ACK, Int(1)),
            ],
        );
    }
    safe(
        &mut m,
        "InitialAckRequiresFreshFullQuiescence",
        eq(ACK, 0).or(all([
            eq(FIRST, 1),
            eq(SECOND, 1),
            Var(FG).eq(Var(GEN)),
            Var(SG).eq(Var(GEN)),
            eq(PENDING, 0).or(eq(GEN, 1)),
            initial.clone(),
            eq(USER, 0),
        ])),
    );
    safe(
        &mut m,
        "KernelTailPendingRequiresInitialZeroUserOwner",
        eq(P, 2).negate().or(eligible),
    );
    safe(
        &mut m,
        "ExecutionRequiresKnownPaidAdmission",
        eq(CONT, 0).or(all([
            eq(ACK, 1),
            eq(DEBT, 0),
            eq(KNOWN, 1),
            eq(DEADLINE, 0),
        ])),
    );
    safe(
        &mut m,
        "OriginalEffectAndUnreapedPinNeverReleasedByPendingOrRefusal",
        all([eq(OWNER, 1), eq(PIN, 1)]),
    );
    m.weak_fair.push(vec![pay]);
    m.weak_fair.push(vec![cont]);
    m
}

#[cfg(test)]
mod initial_quiescence_tests {
    use super::*;
    #[test]
    fn pending_creation_requires_fresh_full_quiescence_and_retains_original_owner() {
        let output =
            std::env::var_os("DREAM_INITIAL_QUIESCENCE_FORMAL_OUT").map(std::path::PathBuf::from);
        if let Some(root) = &output {
            std::fs::create_dir_all(root).unwrap();
        }
        let mut manifest = String::new();
        for control in 0..=3 {
            let model = initial_quiescence(control);
            let graph = model.check(100_000).unwrap();
            assert!(graph.violations.iter().all(|v| v.property != "TypeOK"));
            if control == 0 {
                assert!(graph.violations.is_empty());
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 2 && s[3] == 2 && s[12] == 0)
                );
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 2 && s[3] == 1 && s[4] == 2 && s[12] == 0)
                );
                assert!(graph.states.iter().any(|s| s[0] == 3
                    && s[8] == 1
                    && s[7] == 1
                    && s[5] == 1
                    && s[6] == 1));
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 5 && s[1] == 1 && s[4] == 2 && s[12] == 0)
                );
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 5 && s[11] == 1 && s[9] == 1 && s[16] == 1)
                );
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 5 && s[1] == 7 && s[13] == 0)
                );
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[0] == 4 && s[14] == 0 && s[15] == 1)
                );
            } else {
                let property = if control == 2 {
                    "KernelTailPendingRequiresInitialZeroUserOwner"
                } else {
                    "InitialAckRequiresFreshFullQuiescence"
                };
                assert_eq!(
                    graph
                        .violations
                        .iter()
                        .map(|v| v.property.as_str())
                        .collect::<Vec<_>>(),
                    vec![property]
                );
            }
            println!(
                "initial_quiescence control={control} model={} states={} labelled_edges={} named_verdicts={:?}",
                model.name,
                graph.states.len(),
                graph.edges.iter().map(Vec::len).sum::<usize>(),
                graph
                    .violations
                    .iter()
                    .map(|v| &v.property)
                    .collect::<Vec<_>>()
            );
            if let Some(root) = &output {
                super::helper_observer_lowering_export::export(
                    model,
                    if control == 0 { "pass" } else { "safety" },
                    root,
                    &mut manifest,
                )
                .unwrap();
            }
        }
        if let Some(root) = output {
            std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
        }
    }
}

/// One pre-final-receipt audit cutoff, independent from the existing wire schema.
/// Three bounded repeated-control counts and three persistence/tail outcomes are
/// observed independently. Final receipt IO/durability and process exit remain
/// implementation witnesses; this projection never releases custody or qualifies.
pub fn private_verdicts(control: u8) -> Model {
    let suffix = ["", "FirstErrorOnly", "Persistence", "EarlyTerminal"][usize::from(control)];
    const PH: usize = 0;
    const EXPECT_CONTROL: usize = 1;
    const EXPECT_PERSIST: usize = 2;
    const EXPECT_TAIL: usize = 3;
    const CONTROL: usize = 4;
    const PERSIST: usize = 5;
    const TAIL: usize = 6;
    const CONTROL_COUNT: usize = 7;
    const PERSIST_ERROR: usize = 8;
    const TAIL_ERROR: usize = 9;
    const TAIL_CLOSED: usize = 10;
    const COST: usize = 11;
    const RAW_UNKNOWN: usize = 12;
    const DEBT: usize = 13;
    const PAID: usize = 14;
    const AUDIT: usize = 15;
    let mut m = base(&format!("DreamReadPrivateVerdicts{suffix}"), control != 0, &[
        ("phase", 5, 0), ("actualControlFailureCount", 2, 0),
        ("actualPersistenceOutcome", 3, 0), ("actualTerminalOutcome", 3, 0),
        ("controlVerdict", 2, 0), ("persistenceVerdict", 3, 0),
        ("terminalCostVerdict", 3, 0), ("retainedControlFailureCount", 2, 0),
        ("retainedPersistenceFailure", 1, 0), ("retainedTerminalFailure", 1, 0),
        ("preFinalReceiptTailClosed", 1, 0), ("retainedTailCostBound", 3, 0),
        ("retainedRawUnknown", 1, 0), ("freshTailDebt", 27, 0),
        ("tailDebtActuallyPaid", 1, 0), ("finalAuditAttempted", 1, 0),
    ]);
    for c in 0..=2 {
        for p in 1..=3 {
            for t in 1..=3 {
                act(&mut m, &format!("SelectIndependentOutcomes{c}_{p}_{t}"), eq(PH, 0),
                    &[(PH, Int(1)), (EXPECT_CONTROL, Int(c)),
                        (EXPECT_PERSIST, Int(p)), (EXPECT_TAIL, Int(t))]);
            }
        }
        act(&mut m, &format!("RetainRepeatedControlFailures{c}"),
            all([eq(PH, 1), eq(EXPECT_CONTROL, c)]),
            &[(PH, Int(2)), (CONTROL, Int(if c == 0 { 1 } else { 2 })),
                (CONTROL_COUNT, Int(if control == 1 { c.min(1) } else { c }))]);
        for p in 1..=3 {
            let persistence_error = i64::from(p != 1);
            act(&mut m, &format!("ObserveActualPublicationOutcome{c}_{p}"),
                all([eq(PH, 2), eq(EXPECT_CONTROL, c), eq(EXPECT_PERSIST, p)]),
                &[(PH, Int(3)), (PERSIST, Int(if control == 2 { 1 } else { p })),
                    (PERSIST_ERROR, Int(if control == 1 && c != 0 { 0 } else { persistence_error }))]);
            for t in 1..=3 {
                // Outcome1 is a within-unit bound,2 a genuine overrun,3 raw
                // unknown with the conservative3/27 obligation and no paid claim.
                let cost = t;
                let unknown = i64::from(t == 3);
                let terminal_error = i64::from(t != 1);
                act(&mut m, &format!("CloseAllExitTailWork{c}_{p}_{t}"),
                    all([eq(PH, 3), eq(EXPECT_CONTROL, c),
                        eq(EXPECT_PERSIST, p), eq(EXPECT_TAIL, t)]),
                    &[(PH, Int(4)), (TAIL, Int(t)), (TAIL_CLOSED, Int(1)),
                        (COST, Int(cost)), (RAW_UNKNOWN, Int(unknown)),
                        (DEBT, Int(9 * cost)), (PAID, Int(0)),
                        (TAIL_ERROR, Int(if control == 1 && (c != 0 || p != 1) {
                            0
                        } else { terminal_error }))]);
            }
        }
    }
    let pay = m.actions.len();
    act(&mut m, "ActuallyPayFullKnownClosedTailDebt",
        all([eq(PH, 4), eq(RAW_UNKNOWN, 0), eq(PAID, 0)]),
        &[(DEBT, Int(0)), (PAID, Int(1))]);
    let audit = m.actions.len();
    act(&mut m, "AttemptFinalRawAuditAfterClosedTailCutoff",
        all([eq(PH, 4), eq(PAID, 1).or(eq(RAW_UNKNOWN, 1))]),
        &[(PH, Int(5)), (AUDIT, Int(1))]);
    if control == 3 {
        act(&mut m, "BrokenFinalRawAuditBeforeExitTail", eq(PH, 3),
            &[(PH, Int(5)), (AUDIT, Int(1))]);
    }
    safe(&mut m, "EveryIndependentFailureRemainsRecorded", all([
        eq(CONTROL, 0).or(Var(CONTROL_COUNT).eq(Var(EXPECT_CONTROL))),
        eq(PERSIST, 0).or(eq(EXPECT_PERSIST, 1).and(eq(PERSIST_ERROR, 0))
            .or(eq(EXPECT_PERSIST, 1).negate().and(eq(PERSIST_ERROR, 1)))),
        eq(TAIL_CLOSED, 0).or(eq(EXPECT_TAIL, 1).and(eq(TAIL_ERROR, 0))
            .or(eq(EXPECT_TAIL, 1).negate().and(eq(TAIL_ERROR, 1)))),
    ]));
    safe(&mut m, "PersistenceVerdictMatchesActualAttempt",
        eq(PERSIST, 0).or(Var(PERSIST).eq(Var(EXPECT_PERSIST))));
    safe(&mut m, "TerminalReceiptFollowsAllTailCost",
        eq(AUDIT, 0).or(all([eq(TAIL_CLOSED, 1), Int(0).lt(Var(TAIL))])));
    for t in 1..=3 {
        safe(&mut m, &format!("ClosedTailRetainsFullCostAndFreshDebt{t}"),
            all([eq(TAIL_CLOSED, 1), eq(EXPECT_TAIL, t)]).negate().or(all([
                eq(COST, t), eq(RAW_UNKNOWN, i64::from(t == 3)),
                eq(PAID, 0).and(eq(DEBT, 9 * t)).or(
                    Bool(t != 3).and(eq(PAID, 1)).and(eq(DEBT, 0))),
            ])));
    }
    m.weak_fair.push(vec![pay]);
    m.weak_fair.push(vec![audit]);
    m
}

#[cfg(test)]
mod private_verdict_tests {
    use super::*;
    #[test]
    fn independent_failure_axes_and_final_tail_cutoff_export_all_controls() {
        let output = std::env::var_os("DREAM_PRIVATE_VERDICT_FORMAL_OUT")
            .map(std::path::PathBuf::from);
        let mut manifest = String::new();
        for control in 0..=3 {
            let model = private_verdicts(control);
            let graph = model.check(100_000).unwrap();
            let properties: std::collections::BTreeSet<_> = graph.violations.iter()
                .map(|v| v.property.as_str()).collect();
            let expected: std::collections::BTreeSet<_> = match control {
                1 => ["EveryIndependentFailureRemainsRecorded"].into_iter().collect(),
                2 => ["PersistenceVerdictMatchesActualAttempt"].into_iter().collect(),
                3 => ["TerminalReceiptFollowsAllTailCost"].into_iter().collect(),
                _ => std::collections::BTreeSet::new(),
            };
            assert_eq!(properties, expected);
            assert!(graph.violations.iter().all(|v| v.cycle.is_empty()));
            if control == 0 {
                for c in 0..=2 {
                    for p in 1..=3 {
                        for t in 1..=3 {
                            assert!(graph.states.iter().any(|s|
                                s[1] == c && s[2] == p && s[3] == t && s[15] == 1),
                                "each independent repeated-control/persistence/tail class reaches its final audit attempt");
                        }
                    }
                }
                assert!(graph.states.iter().any(|s| s[15] == 1 && s[1] == 2
                    && s[7] == 2 && s[5] == 2 && s[8] == 1 && s[6] == 2 && s[9] == 1),
                    "an earlier control error cannot hide failed persistence or an over-budget exit tail");
                assert!(graph.states.iter().any(|s| s[15] == 1 && s[12] == 1
                    && s[13] == 27 && s[14] == 0),
                    "unknown tail cost remains raw unknown and unpaid at the explicit audit cutoff");
            }
            println!("private_verdicts control={control} model={} states={} labelled_edges={} named_verdicts={properties:?}",
                model.name, graph.states.len(), graph.edges.iter().map(Vec::len).sum::<usize>());
            if let Some(root) = &output {
                let models = root.join("models");
                std::fs::create_dir_all(&models).unwrap();
                super::helper_observer_lowering_export::export(model,
                    if control == 0 { "pass" } else { "safety" }, &models, &mut manifest).unwrap();
            }
        }
        if let Some(root) = output {
            std::fs::write(root.join("models/manifest.tsv"), manifest).unwrap();
        }
    }
}

/// Shared-codec/currentness projection, not a native lifetime theorem. Owner is
/// the existing private controller; two platform values and three incarnations
/// expose observed A-B-A. The nine profiles abstract independently checked
/// codec, before/after Platform6, incarnation, offline IO, approval and debt facts.
/// Only finite internal classification is weakly fair; no OS success is assumed.
pub fn closure_codec(control: u8) -> Model {
    let suffix = [
        "",
        "Shape",
        "Before",
        "After",
        "Aba",
        "Incarnation",
        "OfflineIo",
        "Approval",
        "Debt",
    ][control as usize];
    let mut m = base(
        &format!("DreamReadClosureCodec{suffix}"),
        control != 0,
        &[
            ("phase", 5, 0),
            ("profile", 8, 0),
            ("platform", 1, 0),
            ("observedChange", 1, 0),
            ("incarnation", 2, 0),
            ("codecBound", 1, 0),
            ("beforeBound", 1, 0),
            ("debtPaid", 1, 0),
            ("offlinePayloadReads", 1, 0),
            ("runtimeApproved", 1, 0),
        ],
    );
    for profile in 0..=8 {
        act(
            &mut m,
            &format!("ParseBoundProfile{profile}"),
            eq(0, 0),
            &[
                (0, Int(1)),
                (1, Int(profile)),
                (5, Int(i64::from(profile != 1))),
                (2, Int(i64::from(profile == 2))),
                (7, Int(i64::from(profile != 8))),
            ],
        );
    }
    let shape = if control == 1 { Bool(true) } else { eq(5, 1) };
    let before = if control == 2 { Bool(true) } else { eq(2, 0) };
    for valid in 0..=1 {
        act(
            &mut m,
            &format!("BeforeBirthFence{valid}"),
            all([
                eq(0, 1),
                shape.clone(),
                before.clone(),
                if valid == 1 {
                    all([eq(2, 0), eq(5, 1)])
                } else {
                    all([eq(2, 0), eq(5, 1)]).negate()
                },
            ]),
            &[
                (0, Int(2)),
                (6, Int(valid)),
                (8, Int(i64::from(control == 6))),
            ],
        );
    }
    // Separate observations preserve a mismatch even if the next sample is A.
    for changed in 0..=1 {
        act(
            &mut m,
            &format!("SettleThenObservePlatform{changed}"),
            all([
                eq(0, 2),
                if changed == 1 {
                    eq(1, 3).or(eq(1, 4))
                } else {
                    eq(1, 3).or(eq(1, 4)).negate()
                },
            ]),
            &[
                (0, Int(3)),
                (2, Int(changed)),
                (3, Int(changed)),
                (4, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "ObservedBReturnsToAWithStickyRevocation",
        all([eq(0, 3), eq(1, 4)]),
        &[
            (0, Int(4)),
            (2, Int(0)),
            (3, Int(i64::from(control != 4))),
            (4, Int(2)),
        ],
    );
    for stale in 0..=1 {
        act(
            &mut m,
            &format!("OtherSettledObservation{stale}"),
            all([
                eq(0, 3),
                eq(1, 4).negate(),
                if stale == 1 {
                    eq(1, 5)
                } else {
                    eq(1, 5).negate()
                },
            ]),
            &[(0, Int(4)), (4, Int(stale))],
        );
    }
    let current = all([
        if control == 3 { Bool(true) } else { eq(2, 0) },
        if control == 3 || control == 4 {
            Bool(true)
        } else {
            eq(3, 0)
        },
        if control == 5 || control == 4 {
            Bool(true)
        } else {
            eq(4, 0)
        },
        if control == 8 { Bool(true) } else { eq(7, 1) },
    ]);
    for valid in 0..=1 {
        let exact = all([eq(2, 0), eq(3, 0), eq(4, 0), eq(7, 1)]);
        act(
            &mut m,
            &format!("ConditionalComponentEvidenceAfterSettledFence{valid}"),
            all([
                eq(0, 4),
                current.clone(),
                if valid == 1 {
                    exact.clone()
                } else {
                    exact.negate()
                },
            ]),
            &[
                (0, Int(5)),
                (6, Int(valid)),
                (9, Int(i64::from(control == 7))),
            ],
        );
    }
    for phase in 1..=4 {
        let rejected = match phase {
            1 => all([shape.clone(), before.clone()]).negate(),
            4 => current.clone().negate(),
            _ => Bool(false),
        };
        act(
            &mut m,
            &format!("RefusePhase{phase}"),
            all([eq(0, phase), rejected]),
            &[(0, Int(5))],
        );
    }
    safe(
        &mut m,
        "BirthRequiresBoundCanonicalCodecAndBeforePlatform",
        eq(0, 2).negate().or(eq(6, 1)),
    );
    for a in m
        .actions
        .iter_mut()
        .filter(|a| a.name.starts_with("RefusePhase"))
    {
        a.updates.push((6, Int(1)));
    }
    safe(
        &mut m,
        "EvidenceRequiresSettledPlatformIncarnationAndPaidDebt",
        eq(0, 5).negate().or(eq(6, 1)),
    );
    safe(&mut m, "OfflineProvenancePayloadsStayOffline", eq(8, 0));
    safe(&mut m, "PrivateEvidenceNeverRuntimeApproval", eq(9, 0));
    let fair = (0..m.actions.len()).collect();
    progress(
        &mut m,
        "InternalBoundClassificationTerminates",
        eq(0, 0).negate(),
        eq(0, 5),
        fair,
    );
    m
}

/// Seven independently observed facts at the authenticated selected revision.
/// A bad fact can be absent, stale or invalid; the concrete verifier distinguishes
/// Unproven from Fail. Neither variant is an eligible lesson in this abstraction.
pub fn eligibility(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadEligibilityFakeCurrent",
        2 => "DreamReadEligibilityOuterSeal",
        3 => "DreamReadEligibilityClosure",
        _ => "DreamReadEligibility",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("disposition", 2, 0),
            ("exactCurrentRecord", 2, 0),
            ("qualifyingTrial", 2, 0),
            ("acceptedPublication", 2, 0),
            ("acceptedOuterSeal", 2, 0),
            ("approvedProducerPolicy", 2, 0),
            ("completeSelectedClosure", 2, 0),
            ("activeCompatibleRevival", 2, 0),
        ],
    );
    let mut fair = vec![];
    for fact in 1..=7 {
        for value in 1..=2 {
            fair.push(m.actions.len());
            act(
                &mut m,
                &format!("ObserveFact{fact}Value{value}"),
                all([eq(0, 0), eq(fact, 0)]),
                &[(fact, Int(value))],
            );
        }
    }
    let observed = all((1..=7).map(|fact| eq(fact, 0).negate()));
    let qualifies = all((1..=7).map(|fact| {
        if matches!((control, fact), (1, 1) | (2, 4) | (3, 6)) {
            Bool(true)
        } else {
            eq(fact, 1)
        }
    }));
    fair.push(m.actions.len());
    act(
        &mut m,
        "CertifyOnlyFullyQualifiedSelectedEntry",
        all([eq(0, 0), observed.clone(), qualifies]),
        &[(0, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "RefuseUnavailableOrInvalidProof",
        all([
            eq(0, 0),
            observed,
            all((1..=7).map(|fact| eq(fact, 1))).negate(),
        ]),
        &[(0, Int(2))],
    );
    safe(
        &mut m,
        "CertifiedHasExactCurrentHead",
        eq(0, 1).negate().or(eq(1, 1)),
    );
    safe(
        &mut m,
        "CertifiedHasAcceptedOuterSeal",
        eq(0, 1).negate().or(eq(4, 1)),
    );
    safe(
        &mut m,
        "CertifiedHasCompleteSelectedClosure",
        eq(0, 1).negate().or(eq(6, 1)),
    );
    safe(
        &mut m,
        "CertifiedHasTrialPublicationPolicyAndActivePair",
        eq(0, 1)
            .negate()
            .or(all([eq(2, 1), eq(3, 1), eq(5, 1), eq(7, 1)])),
    );
    progress(
        &mut m,
        "FiniteSelectionEventuallyHasDisposition",
        Bool(true),
        eq(0, 0).negate(),
        fair,
    );
    m
}

/// One compute owner, one finite helper, four work credits and two protected
/// control credits. The growth overlap and disposal retain their reservations.
/// Fairness assumes local cancel/deadline and actual finite settlement actions;
/// it does not assume that an arbitrary syscall eventually returns.
pub fn ownership(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadOwnershipEarlyExit",
        2 => "DreamReadOwnershipReservation",
        3 => "DreamReadOwnershipStaleGroup",
        4 => "DreamReadOwnershipForeground",
        _ => "DreamReadOwnership",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("phase", 6, 0),
            ("owned", 4, 0),
            ("reserved", 4, 0),
            ("helper", 3, 0),
            ("protectedControl", 2, 2),
            ("foreground", 1, 0),
            ("unit", 1, 0),
            ("helperRunnable", 1, 0),
            ("leaderReaped", 1, 1),
            ("groupBirth", 2, 0),
            ("capturedGroupBirth", 2, 0),
            ("staleSignal", 1, 0),
            ("retirementObserved", 1, 0),
            ("lateSignalRefused", 1, 0),
        ],
    );
    let mut fair = vec![];
    fair.push(m.actions.len());
    act(
        &mut m,
        "ReserveBeforeInitialAllocation",
        eq(0, 0),
        &[(0, Int(1)), (2, if control == 2 { Int(0) } else { Int(2) })],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "AllocateAdmittedOwner",
        eq(0, 1),
        &[(0, Int(2)), (1, Int(2))],
    );
    act(
        &mut m,
        "AttachOnePinnedUnreapedHelperGroup",
        all([eq(0, 2), eq(3, 0), eq(9, 0)]),
        &[(3, Int(1)), (7, Int(0)), (8, Int(0)), (10, Var(9))],
    );
    act(
        &mut m,
        "ObserveForegroundPauseAndStopOwnedGroup",
        all([eq(0, 2), eq(5, 0)]),
        &[(5, Int(1)), (7, Int(0))],
    );
    act(
        &mut m,
        "ForegroundResume",
        all([eq(0, 2), eq(5, 1)]),
        &[(5, Int(0))],
    );
    act(
        &mut m,
        "ResumeOnlyCurrentActiveBackgroundGroup",
        all([
            eq(0, 2),
            eq(3, 1),
            eq(7, 0),
            eq(8, 0),
            Var(9).eq(Var(10)),
            if control == 4 { Bool(true) } else { eq(5, 0) },
        ]),
        &[(7, Int(1))],
    );
    act(
        &mut m,
        "DutyStopOwnedGroup",
        all([eq(3, 1), eq(7, 1), eq(8, 0)]),
        &[(7, Int(0))],
    );
    act(
        &mut m,
        "ReserveOldNewOverlap",
        all([eq(0, 2), eq(1, 2), eq(2, 2), eq(5, 0), eq(6, 0)]),
        &[(2, Int(4))],
    );
    act(
        &mut m,
        "AllocateOnlyAfterOverlapReservation",
        all([eq(0, 2), eq(1, 2), eq(2, 4), eq(5, 0), eq(6, 0)]),
        &[(1, Int(4))],
    );
    act(
        &mut m,
        "InstallGrowthAndReleaseOldAllocation",
        all([eq(0, 2), eq(1, 4), eq(2, 4), eq(5, 0)]),
        &[(1, Int(2)), (2, Int(2)), (6, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "IndependentCancelOrDeadline",
        eq(0, 1).or(eq(0, 2)),
        &[(0, Int(3)), (7, Int(0))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "CompleteLastBoundedUnit",
        all([eq(0, 2), eq(5, 0), eq(6, 1)]),
        &[(0, Int(3)), (7, Int(0))],
    );
    for owned in 1..=4 {
        fair.push(m.actions.len());
        act(
            &mut m,
            &format!("PacedDisposeOwnedPiece{owned}"),
            all([eq(0, 3), eq(1, owned)]),
            &[(1, Int(owned - 1))],
        );
    }
    fair.push(m.actions.len());
    act(
        &mut m,
        "ReleaseCreditAfterActualDisposal",
        all([eq(0, 3), eq(1, 0)]),
        &[(2, Int(0))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "ObserveActualComputeExit",
        all([
            eq(0, 3),
            eq(1, 0),
            eq(2, 0),
            eq(3, 0).or(all([eq(3, 3), eq(8, 1), eq(12, 1)])),
        ]),
        &[(0, Int(4))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "RetireControllerAndFinalSignalBeforeLeaderReap",
        all([eq(0, 3), eq(3, 1), eq(8, 0), Var(9).eq(Var(10))]),
        &[(3, Int(2)), (7, Int(0)), (12, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "ActuallyReapRetiredLeaderAndJoinHelperGroup",
        all([eq(0, 3), eq(3, 2), eq(8, 0), eq(12, 1)]),
        &[(3, Int(3)), (8, Int(1))],
    );
    for birth in 0..2 {
        act(
            &mut m,
            &format!("ExternalPgidReuseAfterReap{birth}"),
            all([eq(3, 3), eq(8, 1), eq(9, birth)]),
            &[(9, Int(birth + 1))],
        );
    }
    act(
        &mut m,
        "DelayedRetiredOrAbaControllerSignal",
        eq(3, 2).or(eq(3, 3)),
        if control == 3 {
            &[(11, Int(1))]
        } else {
            &[(13, Int(1))]
        },
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "JoinExitedOwners",
        all([eq(0, 4), eq(3, 0).or(all([eq(3, 3), eq(8, 1), eq(12, 1)]))]),
        &[(0, Int(5))],
    );
    fair.push(m.actions.len());
    act(&mut m, "DoneOnlyAfterActualJoin", eq(0, 5), &[(0, Int(6))]);
    if control == 1 {
        act(
            &mut m,
            "TreatResultOrChannelLossAsActualExit",
            eq(0, 2).or(eq(0, 3)),
            &[(0, Int(6))],
        );
        act(
            &mut m,
            "ReapLeaderBeforeRetiringController",
            all([eq(0, 3), eq(3, 1), eq(8, 0)]),
            &[(3, Int(3)), (7, Int(0)), (8, Int(1))],
        );
    }
    safe(
        &mut m,
        "EveryOwnedCapacityHasPriorReservation",
        Var(2).lt(Var(1)).negate(),
    );
    safe(&mut m, "ControlReserveCannotBeBorrowed", eq(4, 2));
    safe(
        &mut m,
        "ForegroundOwnedGroupCannotExecute",
        eq(5, 0).or(eq(7, 0)),
    );
    safe(&mut m, "RetiredOrReapedGroupCannotBeSignaled", eq(11, 0));
    safe(
        &mut m,
        "LeaderReapRequiresPriorControllerRetirement",
        eq(3, 3).negate().or(all([eq(8, 1), eq(12, 1), eq(7, 0)])),
    );
    safe(
        &mut m,
        "ComputeExitHasNoOwnedWork",
        Var(0).lt(Int(4)).or(all([
            eq(1, 0),
            eq(2, 0),
            eq(3, 0).or(all([eq(3, 3), eq(8, 1), eq(12, 1)])),
        ])),
    );
    safe(
        &mut m,
        "DoneRequiresAllActualOwnersJoined",
        Var(0).lt(Int(5)).or(all([
            eq(1, 0),
            eq(2, 0),
            eq(3, 0).or(all([eq(3, 3), eq(8, 1), eq(12, 1)])),
        ])),
    );
    progress(
        &mut m,
        "AdmittedWorkerEventuallySettles",
        Bool(true),
        eq(0, 6),
        fair,
    );
    m
}

/// Two immutable ordered pages, descriptor last, actual child exit, one source
/// revision with A -> B -> A changes. Revision identity is monotonic even when
/// labels repeat. This is the admitted bundle cut, not Q3's later live fence.
pub fn snapshot(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadSnapshotDescriptorEarly",
        2 => "DreamReadSnapshotEpochReuse",
        _ => "DreamReadSnapshot",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("sourceRevision", 2, 0),
            ("capturedRevision", 2, 0),
            ("phase", 2, 0),
            ("pages", 2, 0),
            ("descriptor", 1, 0),
            ("actualExit", 1, 0),
            ("disposition", 2, 0),
        ],
    );
    let mut fair = vec![];
    fair.push(m.actions.len());
    act(
        &mut m,
        "ValidateExactManifestAndBundle",
        eq(2, 0),
        &[(1, Var(0)), (2, Int(1))],
    );
    for revision in 0..2 {
        act(
            &mut m,
            &format!("ChangeSourceRevision{revision}"),
            all([eq(0, revision), eq(4, 0), eq(6, 0)]),
            &[(0, Int(revision + 1))],
        );
    }
    let same = Var(0).eq(Var(1));
    let admitted = if control == 2 {
        same.clone().or(all([eq(0, 2), eq(1, 0)]))
    } else {
        same.clone()
    };
    for page in 0..2 {
        fair.push(m.actions.len());
        act(
            &mut m,
            &format!("PublishImmutablePage{page}"),
            all([eq(2, 1), eq(3, page), eq(4, 0), admitted.clone()]),
            &[(3, Int(page + 1))],
        );
    }
    fair.push(m.actions.len());
    act(
        &mut m,
        "PublishDescriptorAfterAllPagesAndRevalidation",
        all([
            eq(2, 1),
            eq(4, 0),
            if control == 1 { Bool(true) } else { eq(3, 2) },
            admitted,
        ]),
        &[(4, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "ActualWorkerExit",
        all([eq(4, 1), eq(5, 0)]),
        &[(5, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "InstallOnlySettledImmutableDescriptor",
        all([eq(4, 1), eq(5, 1), eq(6, 0)]),
        &[(6, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "DetectChangedFiniteBundle",
        all([eq(2, 1), eq(4, 0), same.negate()]),
        &[(2, Int(2))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "RefuseChangedSourceWithoutCertification",
        all([eq(2, 2), eq(6, 0)]),
        &[(6, Int(2))],
    );
    safe(
        &mut m,
        "DescriptorFollowsEveryImmutablePage",
        eq(4, 0).or(eq(3, 2)),
    );
    safe(
        &mut m,
        "DescriptorBindsExactMonotonicSourceRevision",
        eq(4, 0).or(Var(0).eq(Var(1))),
    );
    safe(
        &mut m,
        "InstalledSnapshotRequiresActualExit",
        eq(6, 1)
            .negate()
            .or(all([eq(3, 2), eq(4, 1), eq(5, 1), Var(0).eq(Var(1))])),
    );
    progress(
        &mut m,
        "FinitePreparationEventuallyHasDisposition",
        Bool(true),
        eq(6, 0).negate(),
        fair,
    );
    m
}

/// Decision d93377961365126: one original helper effect and its retained
/// creation ticket. Attempt classes 0/1/2 mean empty/near-bound/exhausted at the
/// production 256 boundary; signature creation consumes two attempts up front.
/// Debt is a normalized positive rest obligation, never elapsed wall time. Real
/// 250us/1ms, native 2ms/25ms/raw3ms and 9*max(1ms,C) witnesses remain concrete.
pub fn helper(control: u8) -> Model {
    helper_observer(control, false)
}

fn helper_observer(control: u8, full_observer: bool) -> Model {
    let name = match control {
        1 => "DreamReadHelperAttempt",
        2 => "DreamReadHelperMember",
        3 => "DreamReadHelperExecutable",
        4 => "DreamReadHelperAba",
        5 => "DreamReadHelperInitialStop",
        6 => "DreamReadHelperStopAck",
        7 => "DreamReadHelperDescendantCharge",
        8 => "DreamReadHelperControllerCharge",
        9 => "DreamReadHelperMainOverlap",
        10 => "DreamReadHelperDebtReset",
        11 => "DreamReadHelperApplicationLabel",
        12 => "DreamReadHelperNativeCpu",
        13 => "DreamReadHelperNativeWall",
        14 => "DreamReadHelperNativeInclusive",
        15 => "DreamReadHelperRunnableCharge",
        16 => "DreamReadHelperEffectError",
        17 => "DreamReadHelperCoverage",
        18 => "DreamReadHelperEarlyTrust",
        19 => "DreamReadHelperClockRelease",
        20 => "DreamReadHelperPanicRelease",
        21 => "DreamReadHelperRevokeRelease",
        22 => "DreamReadHelperStopRetry",
        _ => "DreamReadHelper",
    };
    const ST: usize = 0;
    const INC: usize = 1;
    const CAP: usize = 2;
    const ATT: usize = 3;
    const TICKET: usize = 4;
    const MEMBERS: usize = 5;
    const EXTRA: usize = 6;
    const OWNER: usize = 7;
    const DEBT: usize = 8;
    const OWED: usize = 9;
    const FG: usize = 10;
    const DEADLINE: usize = 11;
    const COVER: usize = 12;
    const TRUST: usize = 13;
    const CONSUMED: usize = 14;
    const EFFECTS: usize = 15;
    const PLAN: usize = 16;
    const STOP: usize = 17;
    const REQUEST: usize = 18;
    const RETIRED: usize = 19;
    const REAPED: usize = 20;
    const SPAN: usize = 21;
    const CASE: usize = 22;
    const LABEL: usize = 23;
    const GRANTS: usize = 24;
    const FAULT: usize = 25;
    const RELEASED: usize = 26;
    const RESERVED: usize = 27;
    const NATIVE: usize = 28;
    const CHARGED: usize = 29;
    const CONTROLLER: usize = 30;
    const DESCENDANT: usize = 31;
    const SPAN_MEMBERS: usize = 32;
    const KNOWN: usize = 33;
    const READY: usize = 34;
    const OUTPUT: usize = 35;
    const APP_USED: usize = 36;
    const CONFIG: usize = 37;
    const REFUSED: usize = 38;
    const INVALID: usize = 39;
    let mut m = base(
        name,
        control != 0,
        &[
            ("state", 7, 0),
            ("incarnationA1B2A3", 2, 0),
            ("effectIncarnation", 2, 0),
            ("creationAttempts", 2, 0),
            ("creationTicket", 2, 0),
            ("members", 2, 0),
            ("unadmittedMember", 1, 0),
            ("workOwnerMainHelperNone", 2, 0),
            ("debt", 2, 0),
            ("observedUnrestedCharge", 2, 0),
            ("foreground", 1, 0),
            ("deadline", 1, 0),
            ("coverage", 1, 0),
            ("outputTrusted", 1, 0),
            ("originalEffectConsumed", 1, 0),
            ("actualOriginalEffects", 2, 0),
            ("approvedMemberLimit", 2, 1),
            ("allStoppedOrExited", 1, 1),
            (
                "stopOperationUnusedConsumedRawOkEsrchVerdictRefusedErrorRetried",
                6,
                0,
            ),
            ("controllerRetired", 1, 0),
            ("allActuallyReaped", 1, 0),
            ("spanNoneNativeHelperApplication", 3, 0),
            ("inputCase", 9, 0),
            ("labelNoneApplicationNative", 2, 0),
            ("hasGranted", 1, 0),
            ("faultNoneClockPanicRevokeEffectResourceMember", 6, 0),
            ("ownershipReleased", 1, 0),
            ("lastCreationReserved", 1, 0),
            ("nativeCreationCertified", 1, 0),
            ("spanCharged", 1, 0),
            ("controllerCharged", 1, 0),
            ("descendantCharged", 1, 0),
            ("membersAtSpan", 2, 0),
            ("accountingKnown", 1, 1),
            ("actualReapReady", 1, 0),
            ("outputObserved", 1, 0),
            ("applicationSpanUsed", 1, 0),
            ("configured", 1, 0),
            ("staleReplyRefused", 1, 0),
            ("invalidated", 1, 0),
        ],
    );
    let exact = Var(INC).eq(Var(CAP));
    let admitted = if control == 4 {
        exact.clone().or(all([eq(INC, 2), eq(CAP, 0)]))
    } else {
        exact.clone()
    };
    let native_ok = all([
        eq(CASE, 3).negate(),
        eq(CASE, 4).negate(),
        eq(CASE, 5).negate(),
    ]);
    let native_guard = native_ok.clone().or(match control {
        12 => eq(CASE, 3),
        13 => eq(CASE, 4),
        14 => eq(CASE, 5),
        _ => Bool(false),
    });
    let quiet = eq(MEMBERS, 0).or(eq(STOP, 1));
    let idle = all([eq(DEBT, 0), eq(OWED, 0), eq(SPAN, 0)]);
    // One input class at a time keeps the complete graph finite and tractable.
    // Classes cover wrong executable/suspend capability, native CPU/wall/raw,
    // ordinary application overrun, runnable-window overrun and attempt edges.
    for case in 0..=9 {
        act(
            &mut m,
            &format!("ConfigureCase{case}"),
            all([eq(CONFIG, 0), eq(ST, 0)]),
            &[
                (CASE, Int(case)),
                (CONFIG, Int(1)),
                (
                    ATT,
                    Int(if case == 8 {
                        2
                    } else if case == 9 {
                        1
                    } else {
                        0
                    }),
                ),
            ],
        );
    }
    for need in 1..=2 {
        for before in 0..=2 {
            let valid = before + need <= 2;
            act(
                &mut m,
                &format!("ReserveAttempt{before}Need{need}"),
                all([
                    eq(CONFIG, 1),
                    eq(ST, 0),
                    eq(TICKET, 0),
                    eq(CONSUMED, 0),
                    eq(FAULT, 0),
                    eq(ATT, before),
                    eq(FG, 0),
                    eq(DEADLINE, 0),
                    idle.clone(),
                    if valid || control == 1 {
                        Bool(true)
                    } else {
                        Bool(false)
                    },
                ]),
                &[
                    (ATT, Int((before + need).min(2))),
                    (TICKET, Int(need)),
                    (PLAN, Int(need)),
                    (CAP, Var(INC)),
                    (RESERVED, Int(i64::from(valid))),
                ],
            );
        }
    }
    for cost in 1..=2 {
        for initially_stopped in 0..=1 {
            act(
                &mut m,
                &format!("CreateSuspendedCost{cost}Stopped{initially_stopped}"),
                all([
                    eq(ST, 0),
                    eq(CONFIG, 1),
                    eq(TICKET, 0).negate(),
                    eq(CONSUMED, 0),
                    eq(MEMBERS, 0),
                    eq(DEADLINE, 0),
                    eq(FAULT, 0),
                    eq(FG, 0),
                    admitted.clone(),
                    idle.clone(),
                    if control == 3 {
                        Bool(true)
                    } else {
                        eq(CASE, 1).negate()
                    },
                    if control == 5 {
                        Bool(true)
                    } else {
                        eq(CASE, 2).negate()
                    },
                    if initially_stopped == 0 {
                        eq(CASE, 2)
                    } else {
                        eq(CASE, 2).negate()
                    },
                    if cost == 1 {
                        native_ok.clone()
                    } else {
                        native_ok.clone().negate()
                    },
                    Var(EFFECTS).lt(Int(2)),
                ]),
                &[
                    (ST, Int(1)),
                    (MEMBERS, Int(1)),
                    (OWNER, Int(0)),
                    (STOP, Int(initially_stopped)),
                    (CONSUMED, Int(1)),
                    (EFFECTS, Var(EFFECTS).plus(Int(1))),
                    (TICKET, Int(0)),
                    (SPAN, Int(1)),
                    (LABEL, Int(2)),
                    (OWED, Int(cost)),
                    (CHARGED, Int(0)),
                    (COVER, Int(0)),
                    (NATIVE, Int(0)),
                ],
            );
        }
    }
    act(
        &mut m,
        "AttachExactSuspendedIdentityBeforeGrant",
        all([
            eq(ST, 1),
            eq(STOP, 1),
            admitted.clone(),
            eq(FAULT, 0),
            eq(DEADLINE, 0),
        ]),
        &[(ST, Int(2))],
    );
    for cost in 1..=2 {
        act(
            &mut m,
            &format!("ChargeNativeInclusiveScope{cost}"),
            all([
                eq(ST, 2),
                eq(SPAN, 1),
                eq(CHARGED, 0),
                eq(OWED, cost),
                eq(KNOWN, 1),
                native_guard.clone(),
                eq(STOP, 1),
            ]),
            &[
                (DEBT, Int(cost)),
                (CHARGED, Int(1)),
                (OWNER, Int(2)),
                (COVER, Int(1)),
                (NATIVE, Int(1)),
                (CONTROLLER, Int(1)),
                (DESCENDANT, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "GrantOneBoundedGroupQuantum",
        all([
            eq(ST, 2).or(eq(ST, 4)),
            eq(STOP, 1),
            eq(OWNER, 2),
            idle.clone(),
            eq(FG, 0),
            eq(DEADLINE, 0),
            eq(FAULT, 0),
            eq(KNOWN, 1),
            eq(NATIVE, 1),
            admitted.clone(),
        ]),
        &[
            (ST, Int(3)),
            (STOP, Int(0)),
            (REQUEST, Int(0)),
            (OWNER, Int(1)),
            (GRANTS, Int(1)),
            (SPAN, Int(2)),
            (LABEL, Int(1)),
            (KNOWN, Int(0)),
            (CHARGED, Int(0)),
            (CONTROLLER, Int(0)),
            (DESCENDANT, Int(0)),
        ],
    );
    act(
        &mut m,
        "ApprovedImplicitGpgAppears",
        all([eq(ST, 3), eq(STOP, 0), eq(PLAN, 2), eq(MEMBERS, 1)]),
        &[(MEMBERS, Int(2))],
    );
    for count in 1..=2 {
        let mut updates = vec![(MEMBERS, Int(2)), (EXTRA, Int(1))];
        if control != 2 {
            updates.extend([
                (ST, Int(7)),
                (FAULT, Int(6)),
                (KNOWN, Int(0)),
                (COVER, Int(0)),
            ]);
        }
        act(
            &mut m,
            &format!("ObserveMemberBeyondAdmittedLimit{count}"),
            all([eq(ST, 3), eq(MEMBERS, count), eq(PLAN, count), eq(EXTRA, 0)]),
            &updates,
        );
    }
    act(
        &mut m,
        "ConsumeStopOperationBeforeNativeEffect",
        all([eq(ST, 3), eq(REQUEST, 0)]),
        &[(REQUEST, Int(1))],
    );
    // One selected logical STOP operation, not the total signals in a job.
    // New roster/audit-version recipients have distinct retained operations;
    // HelperExec covers their current-token stop custody. No send result alone
    // establishes physical STOP, endpoint closure or coverage.
    act(
        &mut m,
        "ObserveConsumedStopNativeRawSuccess",
        all([eq(ST, 3), eq(REQUEST, 1), eq(RETIRED, 0)]),
        &[(REQUEST, Int(2))],
    );
    act(
        &mut m,
        "ObserveConsumedStopEsrchWithPositiveRetainedLeaderExit",
        all([
            eq(ST, 3),
            eq(REQUEST, 1),
            eq(RETIRED, 0),
            eq(CASE, 0),
            eq(PLAN, 1),
        ]),
        &[(REQUEST, Int(3))],
    );
    act(
        &mut m,
        "RefuseStopVerdictAfterRetainedActualKernelSuccess",
        all([
            eq(ST, 3),
            eq(REQUEST, 2),
            eq(RETIRED, 0),
            eq(CASE, 0),
            eq(PLAN, 1),
        ]),
        &[
            (REQUEST, Int(4)),
            (ST, Int(7)),
            (FAULT, Int(6)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    act(
        &mut m,
        "ObserveConsumedStopNativeFailureOrUnknownOutcome",
        all([
            eq(ST, 3),
            eq(REQUEST, 1),
            eq(RETIRED, 0),
            eq(CASE, 0),
            eq(PLAN, 1),
        ]),
        &[
            (REQUEST, Int(5)),
            (ST, Int(7)),
            (FAULT, Int(6)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    if control == 22 {
        act(
            &mut m,
            "BrokenRejectRetriesAlreadyConsumedStopAfterNativeVerdictRefusal",
            all([
                eq(ST, 7),
                eq(REQUEST, 4).or(eq(REQUEST, 5)),
                eq(RETIRED, 0),
                eq(RELEASED, 0),
                eq(EFFECTS, 1),
            ]),
            &[(REQUEST, Int(6))],
        );
    }
    act(
        &mut m,
        "ObserveAllMembersStoppedOrExited",
        all([eq(ST, 3), eq(REQUEST, 2).or(eq(REQUEST, 3)), eq(STOP, 0)]),
        &[(STOP, Int(1))],
    );
    for count in 1..=2 {
        act(
            &mut m,
            &format!("ConfirmStoppedMembers{count}"),
            all([
                eq(ST, 3),
                eq(REQUEST, 2).or(eq(REQUEST, 3)),
                eq(MEMBERS, count),
                if control == 6 {
                    Bool(true)
                } else {
                    eq(STOP, 1)
                },
            ]),
            &[
                (ST, Int(4)),
                (OWNER, Int(2)),
                (OWED, Int(count)),
                (SPAN_MEMBERS, Int(count)),
                (KNOWN, Int(1)),
                (SPAN, Int(2)),
                (LABEL, Int(1)),
                (CHARGED, Int(0)),
            ],
        );
    }
    for count in 1..=2 {
        let omit_descendant = control == 7 && count == 2;
        let omit_controller = control == 8;
        act(
            &mut m,
            &format!("ChargeFullRunnableWallMembersAndController{count}"),
            all([
                eq(ST, 4),
                eq(STOP, 1),
                eq(SPAN, 2),
                eq(CHARGED, 0),
                eq(KNOWN, 1),
                eq(OWED, count),
                if control == 15 {
                    Bool(true)
                } else {
                    eq(CASE, 7).negate()
                },
            ]),
            &[
                (
                    DEBT,
                    Int(count - i64::from(omit_descendant || omit_controller)),
                ),
                (CHARGED, Int(1)),
                (COVER, Int(1)),
                (CONTROLLER, Int(i64::from(!omit_controller))),
                (DESCENDANT, Int(i64::from(!omit_descendant))),
            ],
        );
    }
    for cost in 1..=2 {
        act(
            &mut m,
            &format!("OrdinaryApplicationSpan{cost}"),
            all([
                eq(CONFIG, 1),
                eq(APP_USED, 0),
                eq(FAULT, 0),
                eq(DEADLINE, 0),
                eq(FG, 0),
                if control == 9 {
                    idle.clone().or(all([eq(ST, 3), eq(DEBT, 0), eq(OWED, 0)]))
                } else {
                    idle.clone()
                },
                if control == 9 {
                    eq(ST, 2).or(eq(ST, 3)).or(eq(ST, 4))
                } else {
                    all([
                        eq(ST, 2).or(eq(ST, 4)),
                        quiet.clone(),
                        eq(OWNER, 1).negate(),
                    ])
                },
                if cost == 2 {
                    eq(CASE, 6)
                } else {
                    eq(CASE, 6).negate()
                },
            ]),
            &[
                (OWNER, Int(0)),
                (OWED, Int(cost)),
                (SPAN, Int(3)),
                (LABEL, Int(1)),
                (CHARGED, Int(0)),
                (APP_USED, Int(1)),
            ],
        );
        act(
            &mut m,
            &format!("ChargeOrdinaryApplication{cost}"),
            all([
                eq(SPAN, 3),
                eq(CHARGED, 0),
                eq(OWED, cost),
                eq(KNOWN, 1),
                if control == 11 {
                    Bool(true)
                } else {
                    eq(CASE, 6).negate()
                },
            ]),
            &[
                (DEBT, Int(cost)),
                (CHARGED, Int(1)),
                (OWNER, Int(2)),
                (LABEL, Int(if control == 11 && cost == 2 { 2 } else { 1 })),
                (CONTROLLER, Int(1)),
                (DESCENDANT, Int(1)),
            ],
        );
    }
    let mut settle = vec![];
    for debt in 1..=2 {
        settle.push(m.actions.len());
        act(
            &mut m,
            &format!("RestAfterConfirmedWork{debt}"),
            all([
                eq(DEBT, debt),
                eq(OWED, debt),
                eq(CHARGED, 1),
                eq(OWNER, 2),
                eq(KNOWN, 1),
                quiet.clone(),
            ]),
            &[(DEBT, Int(debt - 1)), (OWED, Int(debt - 1))],
        );
    }
    act(
        &mut m,
        "ReconcilePaidSpanWithoutResettingAnotherDebt",
        all([
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(CHARGED, 1),
            eq(OWNER, 2),
            quiet.clone(),
        ]),
        &[
            (SPAN, Int(0)),
            (LABEL, Int(0)),
            (CHARGED, Int(0)),
            (CONTROLLER, Int(0)),
            (DESCENDANT, Int(0)),
            (SPAN_MEMBERS, Int(0)),
            (APP_USED, Int(0)),
        ],
    );
    for foreground in 0..=1 {
        act(
            &mut m,
            &format!("ObserveForeground{foreground}WithoutDebtReset"),
            all([
                eq(RELEASED, 0),
                eq(FG, foreground).negate(),
                eq(ST, 2)
                    .or(eq(ST, 3))
                    .or(eq(ST, 4))
                    .or(eq(ST, 5).and(eq(DEBT, 0).negate())),
            ]),
            &[(FG, Int(foreground))],
        );
    }
    act(
        &mut m,
        "IoCompletionKeepsDebt",
        all([
            eq(RELEASED, 0),
            eq(DEBT, 0).negate(),
            eq(ST, 2).or(eq(ST, 4)),
        ]),
        &[],
    );
    if control == 10 {
        act(
            &mut m,
            "BrokenForegroundOrIoDebtReset",
            all([eq(CHARGED, 1), eq(DEBT, 0).negate()]),
            &[(DEBT, Int(0)), (FG, Int(1))],
        );
    }
    act(
        &mut m,
        "ObserveOriginalOutputBeforeSettlement",
        all([
            eq(ST, 3).or(eq(ST, 4)),
            Var(GRANTS).lt(Int(1)).negate(),
            eq(FAULT, 0),
        ]),
        &[(OUTPUT, Int(1))],
    );
    act(
        &mut m,
        "ObserveDeadline",
        all([eq(RELEASED, 0), eq(DEADLINE, 0)]),
        &[(DEADLINE, Int(1))],
    );
    for fault in 1..=3 {
        let mut updates = vec![
            (ST, Int(7)),
            (FAULT, Int(fault)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
            (KNOWN, Int(if fault == 3 { 1 } else { 0 })),
        ];
        if fault != 3 {
            // Unknown accounting is terminal for content. Quotient only dead
            // span annotations; retain every outstanding debt/ownership fact.
            updates.extend([
                (SPAN, Int(0)),
                (LABEL, Int(0)),
                (CHARGED, Int(0)),
                (CONTROLLER, Int(0)),
                (DESCENDANT, Int(0)),
                (SPAN_MEMBERS, Int(0)),
                (NATIVE, Int(0)),
                (OUTPUT, Int(0)),
                (GRANTS, Int(0)),
                (APP_USED, Int(0)),
            ]);
        }
        act(
            &mut m,
            &format!("ObserveClockPanicRevoke{fault}"),
            all([
                Var(ST).lt(Int(6)),
                eq(RELEASED, 0),
                eq(FAULT, 0),
                eq(CONFIG, 1),
                eq(EFFECTS, 1),
            ]),
            &updates,
        );
    }
    act(
        &mut m,
        "NativeEffectThenReportedError",
        all([
            eq(ST, 2),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(FAULT, 0),
            eq(CONSUMED, 1),
            eq(GRANTS, 0),
            eq(APP_USED, 0),
        ]),
        &[
            (ST, Int(7)),
            (FAULT, Int(4)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
            (CONSUMED, Int(if control == 16 { 0 } else { 1 })),
        ],
    );
    let overrun = eq(SPAN, 1)
        .and(native_ok.clone().negate())
        .or(eq(SPAN, 2).and(eq(CASE, 7)))
        .or(eq(SPAN, 3).and(eq(CASE, 6)));
    settle.push(m.actions.len());
    act(
        &mut m,
        "RefuseMeasuredOverrunWithoutCertifyingOutput",
        all([overrun, eq(CHARGED, 0), eq(FAULT, 0), eq(RELEASED, 0)]),
        &[
            (ST, Int(7)),
            (FAULT, Int(5)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    for cost in 1..=2 {
        act(
            &mut m,
            &format!("ChargeKnownRejectedSpan{cost}"),
            all([
                eq(ST, 7),
                eq(FAULT, 5),
                eq(KNOWN, 1),
                eq(CHARGED, 0),
                eq(OWED, cost),
                quiet.clone(),
            ]),
            &[
                (DEBT, Int(cost)),
                (CHARGED, Int(1)),
                (OWNER, Int(2)),
                (COVER, Int(0)),
                (CONTROLLER, Int(1)),
                (DESCENDANT, Int(1)),
            ],
        );
    }
    settle.push(m.actions.len());
    act(
        &mut m,
        "DeadlineRetainsOwnedQuarantine",
        all([eq(DEADLINE, 1), Var(ST).lt(Int(5)), eq(RELEASED, 0)]),
        &[(ST, Int(7)), (COVER, Int(0)), (TRUST, Int(0))],
    );
    // Invalidate/restart preserve the original effect and the old pinned owner.
    // A3 repeats the label A, never its incarnation or original authority.
    act(
        &mut m,
        "InvalidateWithoutReleasingOldOwner",
        all([
            eq(INVALID, 0),
            Var(INC).lt(Int(2)),
            eq(RELEASED, 0),
            eq(CONFIG, 1),
            eq(FAULT, 0).or(all([eq(FAULT, 3), eq(ST, 7), eq(INC, 1), eq(KNOWN, 1)])),
        ]),
        &[
            (INVALID, Int(1)),
            (ST, Int(7)),
            (FAULT, Int(3)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    act(
        &mut m,
        "RestartA1B2A3",
        all([eq(INVALID, 1), Var(INC).lt(Int(2))]),
        &[(INC, Var(INC).plus(Int(1))), (INVALID, Int(0))],
    );
    act(
        &mut m,
        "RefuseOldIncarnationReply",
        all([exact.clone().negate(), eq(ST, 7)]),
        &[(REFUSED, Int(1))],
    );
    if control == 4 {
        act(
            &mut m,
            "ReplayOldAReplyIntoA3",
            all([
                eq(ST, 7),
                eq(INC, 2),
                eq(CAP, 0),
                eq(MEMBERS, 0).negate(),
                eq(STOP, 1),
                eq(NATIVE, 1),
                eq(DEBT, 0),
                eq(OWED, 0),
            ]),
            &[
                (ST, Int(2)),
                (FAULT, Int(0)),
                (COVER, Int(1)),
                (OWNER, Int(2)),
            ],
        );
    }
    settle.push(m.actions.len());
    act(
        &mut m,
        "RetireControllerBeforePinnedLeaderFinalSignal",
        all([
            eq(ST, 1).or(eq(ST, 2)).or(eq(ST, 4)).or(eq(ST, 7)),
            eq(RETIRED, 0),
        ]),
        &[(ST, Int(5)), (RETIRED, Int(1))],
    );
    // OS completion is observed, not a fairness assumption. A never-returning
    // observation has an explicit owned quarantine disposition below.
    act(
        &mut m,
        "ObserveActualAllMemberReapReady",
        all([eq(ST, 5), eq(RETIRED, 1), eq(READY, 0)]),
        &[(READY, Int(1))],
    );
    settle.push(m.actions.len());
    act(
        &mut m,
        "ActuallyReapAllMembersAfterControllerRetirement",
        all([eq(ST, 5), eq(RETIRED, 1), eq(READY, 1)]),
        &[
            (ST, Int(6)),
            (MEMBERS, Int(0)),
            (REAPED, Int(1)),
            (STOP, Int(1)),
            (OWNER, Int(2)),
        ],
    );
    settle.push(m.actions.len());
    act(
        &mut m,
        "UnavailableSettlementKeepsOwnedQuarantine",
        all([eq(ST, 5), eq(READY, 0), eq(DEADLINE, 1)]),
        &[(ST, Int(7)), (COVER, Int(0)), (TRUST, Int(0))],
    );
    act(
        &mut m,
        "RetryOnlyUnconsumedOriginalEffect",
        all([
            eq(ST, 6),
            eq(FAULT, 4),
            eq(CONSUMED, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(RELEASED, 0),
        ]),
        &[
            (ST, Int(0)),
            (FAULT, Int(0)),
            (RETIRED, Int(0)),
            (REAPED, Int(0)),
            (READY, Int(0)),
            (NATIVE, Int(0)),
            (COVER, Int(0)),
            (SPAN, Int(0)),
            (LABEL, Int(0)),
        ],
    );
    act(
        &mut m,
        "CoverageReceiptUnavailable",
        all([eq(ST, 6), eq(COVER, 1), eq(RELEASED, 0)]),
        &[(COVER, Int(0)), (TRUST, Int(0))],
    );
    let fully_reaped = all([eq(ST, 6), eq(MEMBERS, 0), eq(RETIRED, 1), eq(REAPED, 1)]);
    act(
        &mut m,
        "TrustOnlyCoveredSettledOriginalOutput",
        all([
            if control == 18 {
                Bool(true)
            } else {
                fully_reaped.clone()
            },
            if control == 17 {
                Bool(true)
            } else {
                eq(COVER, 1)
            },
            if control == 2 {
                Bool(true)
            } else {
                eq(EXTRA, 0)
            },
            eq(NATIVE, 1),
            eq(KNOWN, 1),
            eq(FAULT, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(OWNER, 2),
            eq(OUTPUT, 1),
            eq(FG, 0),
            eq(DEADLINE, 0),
            admitted.clone(),
            eq(CONSUMED, 1),
            eq(TRUST, 0),
        ]),
        &[(TRUST, Int(1))],
    );
    settle.push(m.actions.len());
    act(
        &mut m,
        "ReleaseOnlyActuallySettledKnownOwnership",
        all([
            fully_reaped.clone(),
            eq(KNOWN, 1),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(OWNER, 2),
            eq(RELEASED, 0),
        ]),
        &[(RELEASED, Int(1))],
    );
    for (mode, fault) in [(19, 1), (20, 2), (21, 3)] {
        if control == mode {
            act(
                &mut m,
                &format!("BrokenReleaseOnFault{fault}"),
                all([eq(ST, 7), eq(FAULT, fault), eq(RELEASED, 0)]),
                &[(RELEASED, Int(1)), (OWNER, Int(2))],
            );
        }
    }
    safe(
        &mut m,
        "NativeEffectRequiresReservedAttempt",
        eq(EFFECTS, 0).or(eq(RESERVED, 1)),
    );
    safe(
        &mut m,
        "NativeEffectRequiresExactExecutableCapability",
        eq(EFFECTS, 0).or(eq(CASE, 1).negate()),
    );
    safe(
        &mut m,
        "NoUserCodeBeforeInitialSuspension",
        eq(EFFECTS, 0).or(eq(CASE, 2).negate()),
    );
    safe(
        &mut m,
        "AttachedWorkKeepsExactIncarnation",
        all([
            eq(ST, 1).negate(),
            eq(ST, 2).negate(),
            eq(ST, 3).negate(),
            eq(ST, 4).negate(),
        ])
        .or(exact.clone()),
    );
    safe(
        &mut m,
        "StoppedStateRequiresAllMemberConfirmation",
        eq(ST, 4).negate().or(eq(STOP, 1)),
    );
    safe(
        &mut m,
        "MainProofCannotOverlapRunnableGroup",
        eq(MEMBERS, 0).or(eq(STOP, 1)).or(eq(OWNER, 0).negate()),
    );
    safe(
        &mut m,
        "DebtCoversWholeObservedUnrestedSpan",
        eq(CHARGED, 0).or(Var(DEBT).lt(Var(OWED)).negate()),
    );
    safe(
        &mut m,
        "HelperChargeIncludesDescendantsAndController",
        eq(CHARGED, 0).or(eq(SPAN, 2).negate()).or(all([
            eq(CONTROLLER, 1),
            Var(SPAN_MEMBERS).lt(Int(2)).or(eq(DESCENDANT, 1)),
        ])),
    );
    safe(
        &mut m,
        "OrdinaryApplicationCannotUseNativeLabel",
        eq(LABEL, 2).negate().or(eq(SPAN, 1)),
    );
    safe(
        &mut m,
        "CertifiedNativeCreationMeetsCpuWallInclusiveBounds",
        eq(NATIVE, 0).or(native_ok),
    );
    safe(
        &mut m,
        "CoveredRunnableChargeMeetsWholeGroupWindow",
        eq(SPAN, 2)
            .negate()
            .or(eq(CHARGED, 0))
            .or(eq(COVER, 0))
            .or(eq(CASE, 7).negate()),
    );
    safe(
        &mut m,
        "OriginalNativeEffectNeverReplayed",
        Var(EFFECTS).lt(Int(2)),
    );
    safe(
        &mut m,
        "TrustedOutputHasCompleteCoverage",
        eq(TRUST, 0).or(eq(COVER, 1)),
    );
    safe(
        &mut m,
        "TrustedOutputHasActualReapAndRetirement",
        eq(TRUST, 0).or(fully_reaped.clone()),
    );
    safe(
        &mut m,
        "TrustedOutputHasOnlyAdmittedMembers",
        eq(TRUST, 0).or(eq(EXTRA, 0)),
    );
    safe(
        &mut m,
        "TrustedOutputKeepsOriginalIncarnation",
        eq(TRUST, 0).or(exact),
    );
    safe(
        &mut m,
        "FaultCannotReleaseUnsettledOwnership",
        eq(RELEASED, 0).or(all([
            fully_reaped,
            eq(MEMBERS, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(KNOWN, 1),
        ])),
    );
    safe(
        &mut m,
        "StopAttemptIsNotRetriedAfterAnyNativeOutcome",
        eq(REQUEST, 6).negate(),
    );
    progress(
        &mut m,
        "ControlHasOwnedSettlementOrQuarantineDisposition",
        eq(DEADLINE, 1),
        Var(ST).lt(Int(5)).negate(),
        settle,
    );
    if control != 0 {
        // Keep the original full observer for exact finite equivalence. Each
        // control can violate only these safety boundaries; all invariants are
        // still declared and checked. Boolean value guards prevent TLC from
        // multiplying next-state branches for state-only disjunctions.
        let relevant: &[&str] = match control {
            1 => &["NativeEffectRequiresReservedAttempt"],
            2 => &["TrustedOutputHasOnlyAdmittedMembers"],
            3 => &["NativeEffectRequiresExactExecutableCapability"],
            4 => &["AttachedWorkKeepsExactIncarnation"],
            5 => &[
                "NoUserCodeBeforeInitialSuspension",
                "MainProofCannotOverlapRunnableGroup",
            ],
            6 => &["StoppedStateRequiresAllMemberConfirmation"],
            7 | 8 => &[
                "DebtCoversWholeObservedUnrestedSpan",
                "HelperChargeIncludesDescendantsAndController",
            ],
            9 => &["MainProofCannotOverlapRunnableGroup"],
            10 => &["DebtCoversWholeObservedUnrestedSpan"],
            11 => &["OrdinaryApplicationCannotUseNativeLabel"],
            12..=14 => &["CertifiedNativeCreationMeetsCpuWallInclusiveBounds"],
            15 => &["CoveredRunnableChargeMeetsWholeGroupWindow"],
            16 => &["OriginalNativeEffectNeverReplayed"],
            17 => &["TrustedOutputHasCompleteCoverage"],
            18 => &["TrustedOutputHasActualReapAndRetirement"],
            19..=21 => &["FaultCannotReleaseUnsettledOwnership"],
            22 => &["StopAttemptIsNotRetriedAfterAnyNativeOutcome"],
            _ => unreachable!(),
        };
        assert!(relevant
            .iter()
            .all(|name| m.invariants.iter().any(|i| i.name == *name)));
        let valid_prefix = all(m
            .invariants
            .iter()
            .filter(|i| full_observer || relevant.contains(&i.name.as_str()))
            .map(|i| i.condition.clone()));
        for action in &mut m.actions {
            let guard = action.guard.clone().and(valid_prefix.clone());
            action.guard = if full_observer {
                guard
            } else {
                guard.eq(Bool(true))
            };
        }
        let failed = valid_prefix.negate();
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            if full_observer {
                failed
            } else {
                failed.eq(Bool(true))
            },
            &[],
        );
    }
    m
}

/// One logical descendant can exec from the admitted Git fork image into GPG
/// within an already charged group quantum. Darwin changes its audit task
/// version while PID/birth/PPID/PGID remain fixed; old receipts are read-only.
/// This separate refinement does not multiply the whole helper lifecycle graph.
pub fn helper_exec(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadHelperExecStaleStop",
        2 => "DreamReadHelperExecStaleSignal",
        3 => "DreamReadHelperExecSignalBeforeCapture",
        4 => "DreamReadHelperExecReverseExec",
        5 => "DreamReadHelperExecNoGpg",
        6 => "DreamReadHelperExecEarlyTrust",
        7 => "DreamReadHelperExecEarlyRelease",
        _ => "DreamReadHelperExec",
    };
    const PH: usize = 0;
    const CUR: usize = 1;
    const CAP: usize = 2;
    const OLD: usize = 3;
    const SEAL: usize = 4;
    const STOP: usize = 5;
    const RUN: usize = 6;
    const REQ: usize = 7;
    const DEBT: usize = 8;
    const GPG: usize = 9;
    const COVER: usize = 10;
    const TRUST: usize = 11;
    const RETIRED: usize = 12;
    const EXIT: usize = 13;
    const REAP: usize = 14;
    const RELEASE: usize = 15;
    const QUAR: usize = 16;
    const INVALID: usize = 17;
    const SIG: usize = 18;
    const SIGCUR: usize = 19;
    const SIGCAP: usize = 20;
    const GCUR: usize = 21;
    const GSEAL: usize = 22;
    const GACK: usize = 23;
    const RCUR: usize = 24;
    const RSEAL: usize = 25;
    const RACK: usize = 26;
    const GCAP: usize = 27;
    let mut m = base(
        name,
        control != 0,
        &[
            ("phaseAbsentCloneGpgExited", 3, 0),
            ("currentTaskVersion", 2, 0),
            ("capturedCurrentToken", 2, 0),
            ("retainedOldReadOnlyToken", 1, 0),
            ("stopReceiptVersion", 2, 0),
            ("actualCurrentStopped", 1, 0),
            ("groupQuantumRunning", 1, 0),
            ("stopRequestVersion", 2, 0),
            ("unrestedQuantum", 1, 0),
            ("actualGpgObserved", 1, 0),
            ("currentImageCoverage", 1, 0),
            ("execEvidenceTrusted", 1, 0),
            ("controllerRetired", 1, 0),
            ("actualExitObserved", 1, 0),
            ("actualReapObserved", 1, 0),
            ("ownershipReleased", 1, 0),
            ("ownedQuarantine", 1, 0),
            ("unapprovedImageObserved", 1, 0),
            ("lastSignalToken", 2, 0),
            ("currentAtSignal", 2, 0),
            ("capturedAtSignal", 2, 0),
            ("versionAtGrantAdmission", 2, 0),
            ("receiptAtGrantAdmission", 2, 0),
            ("stoppedAtGrantAdmission", 1, 0),
            ("versionAtRest", 2, 0),
            ("receiptAtRest", 2, 0),
            ("stoppedAtRest", 1, 0),
            ("capturedAtGrantAdmission", 2, 0),
        ],
    );
    let current_captured = Var(CAP).eq(Var(CUR));
    let current_stopped = all([
        eq(STOP, 1),
        Var(SEAL).eq(Var(CUR)),
        current_captured.clone(),
    ]);
    let active = all([eq(QUAR, 0), eq(RELEASE, 0), eq(RETIRED, 0)]);
    act(
        &mut m,
        "AdmitOneLogicalCloneWithCurrentStoppedToken",
        eq(PH, 0),
        &[
            (PH, Int(1)),
            (CUR, Int(1)),
            (CAP, Int(1)),
            (SEAL, Int(1)),
            (STOP, Int(1)),
            (COVER, Int(1)),
        ],
    );
    act(
        &mut m,
        "GrantGroupQuantumFromCurrentStopReceipt",
        all([
            eq(PH, 1).or(eq(PH, 2)),
            active.clone(),
            eq(RUN, 0),
            eq(DEBT, 0),
            current_stopped.clone(),
        ]),
        &[
            (RUN, Int(1)),
            (STOP, Int(0)),
            (REQ, Int(0)),
            (TRUST, Int(0)),
            (GCUR, Var(CUR)),
            (GSEAL, Var(SEAL)),
            (GACK, Var(STOP)),
            (GCAP, Var(CAP)),
        ],
    );
    // This transition requires no prior GPG stop: the already admitted clone
    // quantum owns execution through this one-way native exec boundary.
    act(
        &mut m,
        "KernelExecCloneToExactGpgWithinAdmittedQuantum",
        all([eq(PH, 1), eq(CUR, 1), eq(RUN, 1), active.clone()]),
        &[
            (PH, Int(2)),
            (CUR, Int(2)),
            (OLD, Int(1)),
            (GPG, Int(1)),
            (STOP, Int(0)),
            (COVER, Int(0)),
            (REQ, Int(0)),
        ],
    );
    act(
        &mut m,
        "CaptureNewCurrentTaskTokenBeforeAnyControl",
        all([
            eq(PH, 2),
            eq(CUR, 2),
            eq(CAP, 1),
            eq(QUAR, 0),
            eq(RELEASE, 0),
        ]),
        &[(CAP, Int(2)), (COVER, Int(1))],
    );
    act(
        &mut m,
        "RequestOneFreshStopForKnownCurrentVersion",
        all([
            eq(PH, 1).or(eq(PH, 2)),
            eq(RUN, 1),
            eq(REQ, 0),
            eq(QUAR, 0),
            current_captured.clone(),
        ]),
        &[
            (REQ, Var(CUR)),
            (SIG, Var(CAP)),
            (SIGCUR, Var(CUR)),
            (SIGCAP, Var(CAP)),
        ],
    );
    act(
        &mut m,
        "ObserveCurrentVersionActuallyStopped",
        all([
            eq(PH, 1).or(eq(PH, 2)),
            eq(RUN, 1),
            Var(REQ).eq(Var(CUR)),
            current_captured.clone(),
            eq(QUAR, 0),
        ]),
        &[
            (STOP, Int(1)),
            (SEAL, Var(CUR)),
            (RUN, Int(0)),
            (DEBT, Int(1)),
        ],
    );
    let rest = m.actions.len();
    act(
        &mut m,
        "RestOnlyAfterCurrentTaskStopOrActualExit",
        all([
            eq(DEBT, 1),
            eq(RUN, 0),
            current_stopped.clone().or(eq(EXIT, 1)),
        ]),
        &[
            (DEBT, Int(0)),
            (RCUR, Var(CUR)),
            (RSEAL, Var(SEAL)),
            (RACK, Int(1)),
        ],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenRestOrRegrantUsesReadOnlyOldStopSeal",
            all([eq(PH, 2), eq(CUR, 2), eq(SEAL, 1), eq(RUN, 1), eq(QUAR, 0)]),
            &[(RCUR, Var(CUR)), (RSEAL, Var(SEAL)), (RACK, Var(STOP))],
        );
    }
    if control == 2 {
        act(
            &mut m,
            "BrokenOldTokenSignalsNewExecutableTask",
            all([eq(PH, 2), eq(CAP, 2), eq(OLD, 1), eq(QUAR, 0)]),
            &[(SIG, Int(1)), (SIGCUR, Var(CUR)), (SIGCAP, Var(CAP))],
        );
    }
    if control == 3 {
        act(
            &mut m,
            "BrokenFreshSignalBeforeNewTokenCapture",
            all([eq(PH, 2), eq(CUR, 2), eq(CAP, 1), eq(QUAR, 0)]),
            &[(SIG, Var(CUR)), (SIGCUR, Var(CUR)), (SIGCAP, Var(CAP))],
        );
    }
    for fault in ["ReverseGpgToClone", "OtherDescendantImage", "LeaderExec"] {
        act(
            &mut m,
            &format!("ObserveAndRefuse{fault}"),
            all([eq(PH, 1).or(eq(PH, 2)), eq(QUAR, 0), eq(RELEASE, 0)]),
            &[
                (QUAR, Int(1)),
                (INVALID, Int(1)),
                (COVER, Int(0)),
                (TRUST, Int(0)),
            ],
        );
    }
    if control == 4 {
        act(
            &mut m,
            "BrokenAdmitReverseExecAsClone",
            all([eq(PH, 2), eq(CUR, 2), eq(QUAR, 0)]),
            &[(PH, Int(1))],
        );
    }
    let retire = m.actions.len();
    act(
        &mut m,
        "RetireControllerBeforeRelease",
        all([eq(PH, 0).negate(), eq(RETIRED, 0), eq(RELEASE, 0)]),
        &[(RETIRED, Int(1))],
    );
    // These are observations of native completion, without an OS fairness
    // assumption. Missing completion remains an owned unavailable disposition.
    act(
        &mut m,
        "ObserveRealLogicalChildExit",
        all([eq(PH, 1).or(eq(PH, 2)), eq(RELEASE, 0)]),
        &[
            (PH, Int(3)),
            (EXIT, Int(1)),
            (RUN, Int(0)),
            (STOP, Int(1)),
            (DEBT, Int(1)),
            (SEAL, Var(CUR)),
        ],
    );
    act(
        &mut m,
        "ObserveRealReapAfterControllerRetirement",
        all([eq(EXIT, 1), eq(RETIRED, 1), eq(REAP, 0)]),
        &[(REAP, Int(1))],
    );
    let quarantine = m.actions.len();
    act(
        &mut m,
        "UnavailableCompletionRetainsOwnedQuarantine",
        all([eq(RETIRED, 1), eq(REAP, 0), eq(RELEASE, 0), eq(QUAR, 0)]),
        &[(QUAR, Int(1)), (TRUST, Int(0)), (COVER, Int(0))],
    );
    let settled_task = current_stopped.clone().or(eq(EXIT, 1));
    act(
        &mut m,
        "TrustOnlyObservedGpgAndCurrentTaskEvidence",
        all([
            if control == 5 { Bool(true) } else { eq(GPG, 1) },
            if control == 6 {
                Bool(true)
            } else {
                settled_task.clone()
            },
            eq(COVER, 1),
            eq(QUAR, 0),
            eq(INVALID, 0),
            eq(DEBT, 0),
            eq(TRUST, 0),
        ]),
        &[(TRUST, Int(1))],
    );
    act(
        &mut m,
        "ReleaseOnlyRetiredActuallyReapedLogicalChild",
        all([
            eq(RELEASE, 0),
            eq(RETIRED, 1),
            eq(DEBT, 0),
            if control == 7 {
                Bool(true)
            } else {
                all([eq(EXIT, 1), eq(REAP, 1)])
            },
        ]),
        &[(RELEASE, Int(1))],
    );
    safe(
        &mut m,
        "OnlyOneWayCloneToGpgExecIsAdmitted",
        eq(PH, 1).negate().or(eq(CUR, 1)),
    );
    safe(
        &mut m,
        "RetainedOldTokenNeverControlsCurrentTask",
        eq(SIG, 0).or(Var(SIG).eq(Var(SIGCUR))),
    );
    safe(
        &mut m,
        "CurrentTaskTokenCapturedBeforeControl",
        eq(SIG, 0).or(Var(SIGCAP).eq(Var(SIGCUR))),
    );
    safe(
        &mut m,
        "GroupGrantUsesCurrentSealAtAdmission",
        eq(GCUR, 0).or(all([
            Var(GCUR).eq(Var(GSEAL)),
            Var(GCUR).eq(Var(GCAP)),
            eq(GACK, 1),
        ])),
    );
    safe(
        &mut m,
        "RestNeverUsesOldTaskStopReceipt",
        eq(RCUR, 0).or(all([Var(RCUR).eq(Var(RSEAL)), eq(RACK, 1)])),
    );
    safe(
        &mut m,
        "TrustedExecEvidenceRequiresActualGpg",
        eq(TRUST, 0).or(eq(GPG, 1)),
    );
    safe(
        &mut m,
        "TrustedExecEvidenceRequiresCurrentStopOrRealExit",
        eq(TRUST, 0).or(settled_task),
    );
    safe(
        &mut m,
        "TrustedExecEvidenceRequiresCurrentImageCoverage",
        eq(TRUST, 0).or(all([eq(COVER, 1), eq(INVALID, 0), eq(QUAR, 0)])),
    );
    safe(
        &mut m,
        "ReleaseRequiresActualExitReapAndRetirement",
        eq(RELEASE, 0).or(all([eq(EXIT, 1), eq(REAP, 1), eq(RETIRED, 1)])),
    );
    progress(
        &mut m,
        "RetiredTaskHasReapedOrOwnedUnavailableDisposition",
        eq(RETIRED, 1),
        eq(REAP, 1).or(eq(QUAR, 1)),
        vec![retire, rest, quarantine],
    );
    m
}

/// Decision d93377961365489 legacy window geometries, extended by66236 below:
/// unequal post-call endpoints and a reserved slot whose birth stays unknown.
/// Legacy costs0..3 use an immutable profile's250us/500us tick. Accepted ordinary
/// charges owe9ms; the1.5ms overrun profile conservatively retains18ms. Three
/// accepted observed charges fit additive debt0..27ms. No OS timing is proved.
pub fn member_window(control: u8) -> Model {
    member_window_observer(control, false)
}

fn member_window_observer(control: u8, full_observer: bool) -> Model {
    member_window_projection(control, full_observer, true, true)
}

fn member_window_projection(
    control: u8,
    full_observer: bool,
    measured_law: bool,
    forged_terminal_control: bool,
) -> Model {
    let suffix = [
        "",
        "UnequalStop",
        "BackdatedBirth",
        "StaleExec",
        "Transient",
        "BlockedStop",
        "RequiredController",
        "ActiveOverWindow",
        "LateKernel",
        "OmitPipe",
        "DoublePipe",
        "OmitRollup",
        "DoubleRollup",
        "RetiredCoverage",
        "DebtReset",
        "ContinueOverlap",
        "MainOverlap",
        "ClockRelease",
        "StopRelease",
        "RevokeRelease",
        "CancelRelease",
        "PanicRelease",
        "Aba",
        "OverLimit",
        "LateKernelReplay",
        "FailedAdmissionSample",
        "InitialCreation",
        "InitialDebt",
        "AccountingTailDebt",
        "MeasuredOpenSlot",
        "PreAckCounters",
        "MeasuredOmitRollup",
        "MeasuredRawTicks",
        "MeasuredNoWallGate",
        "MeasuredMultiThread",
        "MixedChildRollup",
        "PreExitRollup",
        "DoubleCountRollup",
        "InitialCorroborationUnpaidStage",
        "InitialTerminalObservationSkipped",
        "TeardownOverCeiling",
        "UnchargedTeardownResidual",
        "Wait4DerivedTeardownCeiling",
        "UnprovenChildTeardownCount",
        "EarlyBornUnpaidPhysicalReap",
        "FixedOversizeRunWindow",
        "MinLiveStopLatency",
        "ExitOrFailedStopGrowth",
        "WindowBasisNoDecrease",
        "PendingSecondGrantCommit",
    ];
    let name = format!("DreamReadMemberWindow{}", suffix[control as usize]);
    const PH: usize = 0;
    const CASE: usize = 1;
    const CONFIG: usize = 2;
    const ATT: usize = 3;
    const EFFECT: usize = 4;
    const INC: usize = 5;
    const CAP: usize = 6;
    const S0: usize = 7;
    const S1: usize = 8;
    const CLOCK: usize = 9;
    const START0: usize = 10;
    const START1: usize = 11;
    const END0: usize = 12;
    const END1: usize = 13;
    const ACK0: usize = 14;
    const ACK1: usize = 15;
    const CUR: usize = 16;
    const TOKEN: usize = 17;
    const ROSTER: usize = 18;
    const REQUIRED: usize = 19;
    const SEAL: usize = 20;
    const W: usize = 21;
    const A: usize = 22;
    const C: usize = 23;
    const Q: usize = 24;
    const KNOWN: usize = 25;
    const COVER: usize = 26;
    const DEBT: usize = 27;
    const OWED: usize = 28;
    const POST: usize = 29;
    const LATE: usize = 30;
    const EXPLAIN: usize = 31;
    const REVALID: usize = 32;
    const WORK: usize = 33;
    const TRUST: usize = 34;
    const FG: usize = 35;
    const DEADLINE: usize = 36;
    const FAULT: usize = 37;
    const RETIRED: usize = 38;
    const EXIT: usize = 39;
    const REAP: usize = 40;
    const OWN: usize = 41;
    const UNCERTAIN: usize = 42;
    const PIPE: usize = 43;
    const ROLL: usize = 44;
    const LATESEEN: usize = 45;
    const POSTDONE: usize = 46;
    const PROCESSED: usize = 47;
    const TOTAL: usize = 48;
    const GRANTS: usize = 49;
    const SUPPORT: usize = 50;
    const QUAR: usize = 51;
    const CREATION: usize = 52;
    const OBSERVE: usize = 53;
    let mut m = base(
        &name,
        control != 0,
        &[
            (
                "phaseNoneReservedBornAttachedRunningTentativeClosedChargedRetiringReaped",
                9,
                0,
            ),
            ("immutableGeometryProfile", 10, 0),
            ("configured", 1, 0),
            ("consumedAttempts", 2, 0),
            ("originalEffect", 1, 0),
            ("epochA1B2A3", 2, 0),
            ("originalEffectEpoch", 2, 0),
            ("slot0AbsentPossibleRunningAckStoppedAckExitedUnknown", 5, 0),
            ("slot1AbsentPossibleRunningAckStoppedAckExitedUnknown", 5, 0),
            ("checkedGrantRelativeClock", 3, 0),
            ("slot0WindowStart", 3, 0),
            ("slot1WindowStart", 3, 0),
            ("slot0TentativeEndpoint", 3, 0),
            ("slot1TentativeEndpoint", 3, 0),
            ("slot0CurrentStopExitProof", 1, 0),
            ("slot1CurrentStopExitOrClosedAbsenceProof", 1, 0),
            ("slot1CurrentExecTask", 2, 1),
            ("capturedExecTask", 2, 1),
            ("sameGrantRosterClosed", 1, 0),
            ("allRequiredControllerPredicatesComplete", 1, 0),
            ("stableStoppedRosterSeal", 1, 0),
            ("windowUnionCost", 3, 0),
            ("observedActiveCpu", 3, 0),
            ("activeControllerAndPipeCpu", 3, 0),
            ("inclusiveQuantumCost", 3, 0),
            ("allCostKnown", 1, 1),
            ("coverage", 1, 0),
            ("unpaidDebtMs", 27, 0),
            ("observedOutstandingDebtMs", 27, 0),
            ("stoppedOrdinaryCpu", 3, 0),
            ("lateKernelChargeCount", 2, 0),
            ("sameIncarnationStableUserKernelExplained", 1, 1),
            ("sealRevalidatedAfterLateCpu", 1, 1),
            ("workOwnerNoneHelperMain", 2, 0),
            ("outputTrusted", 1, 0),
            ("foreground", 1, 0),
            ("deadline", 1, 0),
            ("faultNoneClockStopRevokeCancelPanicCoverage", 6, 0),
            ("controllerRetired", 1, 0),
            ("actualExit", 1, 0),
            ("actualReap", 1, 0),
            ("originalEffectOwned", 1, 0),
            ("unaccountedTransient", 1, 0),
            ("pipeAttributionCount", 2, 0),
            ("rollupAttributionCount", 2, 0),
            ("lateCpuObserved", 1, 0),
            ("postClosureCorroborationDone", 1, 0),
            ("lateObservationReconciled", 1, 0),
            ("cumulativeGrantAndPostDebtMs", 27, 0),
            ("grantCount", 2, 0),
            ("provisionedWholeCommandSupport", 1, 1),
            ("ownedUnavailableQuarantine", 1, 0),
            ("canonicalCreationOwnershipSettled", 1, 0),
            ("ordinaryAdmissionNoneValidFailedUnsettledUnpaid", 4, 0),
        ],
    );
    let exact = Var(INC).eq(Var(CAP));
    let quiet = all([
        eq(SEAL, 1),
        eq(ROSTER, 1),
        eq(REQUIRED, 1),
        eq(ACK0, 1),
        eq(ACK1, 1),
        Var(CUR).eq(Var(TOKEN)),
        eq(UNCERTAIN, 0),
        eq(SUPPORT, 1),
    ]);
    let available = all([
        eq(KNOWN, 1),
        eq(SUPPORT, 1),
        eq(FAULT, 0),
        eq(FG, 0),
        eq(DEADLINE, 0),
        eq(RETIRED, 0),
        exact.clone(),
    ]);
    for case in 0..=10 {
        act(
            &mut m,
            &format!("ConfigureRepresentativeGeometry{case}"),
            all([eq(CONFIG, 0), eq(PH, 0)]),
            &[
                (CASE, Int(case)),
                (CONFIG, Int(1)),
                (SUPPORT, Int(i64::from(case != 9))),
                (UNCERTAIN, Int(i64::from(case == 4))),
                (EXPLAIN, Int(i64::from(case != 8))),
            ],
        );
    }
    act(
        &mut m,
        "ReserveBothPossibleSlotsBeforeAnyEffect",
        all([eq(CONFIG, 1), eq(PH, 0), eq(ATT, 0), eq(SUPPORT, 1)]),
        &[(ATT, Int(2)), (PH, Int(1)), (CAP, Var(INC))],
    );
    act(
        &mut m,
        "CreateStoppedRetainsOriginalEffect",
        all([eq(PH, 1), eq(ATT, 2), available.clone()]),
        &[
            (PH, Int(2)),
            (EFFECT, Int(1)),
            (OWN, Int(1)),
            (S0, Int(3)),
            (S1, Int(1)),
        ],
    );
    act(
        &mut m,
        "AttachCurrentStoppedBirth",
        eq(PH, 2),
        &[(PH, Int(3))],
    );
    // The separate Helper graph supplies native charge/rest/reconciliation.
    // This boundary imports its successful ownership handoff; it does not
    // infer settlement from suspended birth or from an idle controller tick.
    act(
        &mut m,
        "ConfirmCanonicalCreationOwnershipAndBothDebtsSettled",
        all([
            eq(PH, 3),
            eq(CREATION, 0),
            eq(OBSERVE, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            available.clone(),
        ]),
        &[(CREATION, Int(1))],
    );
    act(
        &mut m,
        "ObserveOrdinaryControllerTailBeforeInitialCorroboration",
        all([
            eq(PH, 3),
            eq(CREATION, 1),
            eq(OBSERVE, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            available.clone(),
        ]),
        &[(DEBT, Int(9)), (OWED, Int(9))],
    );
    for settled in 0..=1 {
        for paid in 0..=1 {
            act(
                &mut m,
                &format!("InitialCorroborationCreationSettled{settled}DebtsPaid{paid}"),
                all([
                    eq(PH, 3),
                    eq(OBSERVE, 0),
                    eq(CREATION, settled),
                    available.clone(),
                    if settled == 1 || control == 26 {
                        Bool(true)
                    } else {
                        Bool(false)
                    },
                    if paid == 1 {
                        all([eq(DEBT, 0), eq(OWED, 0)])
                    } else {
                        all([eq(DEBT, 9), eq(OWED, 9)])
                    },
                    if paid == 1 || control == 27 {
                        Bool(true)
                    } else {
                        Bool(false)
                    },
                ]),
                &[
                    (
                        OBSERVE,
                        Int(if settled == 0 {
                            3
                        } else if paid == 0 {
                            4
                        } else {
                            1
                        }),
                    ),
                    (DEBT, Var(DEBT).plus(Int(9))),
                    (OWED, Var(OWED).plus(Int(9))),
                ],
            );
        }
    }
    let open = [
        (PH, Int(4)),
        (CLOCK, Int(1)),
        (START0, Int(1)),
        (START1, Int(1)),
        (END0, Int(0)),
        (END1, Int(0)),
        (ACK0, Int(0)),
        (ACK1, Int(0)),
        (ROSTER, Int(0)),
        (REQUIRED, Int(0)),
        (SEAL, Int(0)),
        (COVER, Int(0)),
        (WORK, Int(1)),
        (S0, Int(2)),
        (S1, Int(1)),
    ];
    let mut first = open.to_vec();
    first.push((GRANTS, Int(1)));
    act(
        &mut m,
        "CheckedT0OpensEveryReservedSlotBeforeCont",
        all([
            eq(PH, 3),
            available.clone(),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(CREATION, 1),
            eq(OBSERVE, 1),
        ]),
        &first,
    );
    for failed in 0..=1 {
        // An admission event abstracts entering ordinary expensive validation,
        // not an external OS transition or mandatory retirement token custody.
        act(
            &mut m,
            &format!("OrdinaryExpensiveObservationAdmissionFailed{failed}"),
            all([
                eq(EFFECT, 1),
                eq(OWN, 1),
                eq(RETIRED, 0),
                eq(CREATION, 1),
                eq(OBSERVE, 1),
                eq(DEBT, 0),
                eq(OWED, 0),
                if failed == 0 {
                    eq(FAULT, 0)
                } else {
                    eq(FAULT, 0).negate()
                },
                if failed == 0 || control == 25 {
                    Bool(true)
                } else {
                    Bool(false)
                },
            ]),
            &[(OBSERVE, Int(if failed == 0 { 1 } else { 2 }))],
        );
    }
    act(
        &mut m,
        "OpenUnknownImplicitSlotWithoutInventingBirthTime",
        all([eq(PH, 4), eq(S1, 1)]),
        &[(S1, Int(5))],
    );
    act(
        &mut m,
        "DiscoverReservedLogicalChildKeepsWindowFromT0",
        all([eq(PH, 4), eq(S1, 5), eq(CASE, 2).negate(), eq(FAULT, 0)]),
        &[(S1, Int(2))],
    );
    act(
        &mut m,
        "ForkExecChangesCurrentTaskButNotLogicalWindow",
        all([eq(PH, 4), eq(CASE, 3), eq(S1, 2), eq(CUR, 1)]),
        &[(CUR, Int(2)), (ACK1, Int(0)), (END1, Int(0))],
    );
    act(
        &mut m,
        "CaptureNewExecTaskBeforeItsStopPredicate",
        all([eq(PH, 4), eq(CUR, 2), eq(TOKEN, 1)]),
        &[(TOKEN, Int(2))],
    );
    // The preceding identity capture can also retain a late real effect for
    // cleanup after refusal. It grants no ordinary admission, ACK, seal or
    // coverage; external fork/exec likewise remains possible until real stop.
    act(
        &mut m,
        "StopRequestAloneClosesNoWindow",
        all([eq(PH, 4), eq(FAULT, 0)]),
        &[],
    );
    for case in 0..=10 {
        let late_endpoint = matches!(case, 0 | 2 | 3 | 4);
        let end1 = if late_endpoint { 3 } else { 2 };
        let w = 1 + (end1 - 1);
        let a = if case == 5 {
            3
        } else if late_endpoint || case == 6 {
            2
        } else {
            1
        };
        let c = if matches!(case, 1 | 6 | 7 | 8 | 9) {
            1
        } else {
            0
        };
        assert!(w <= 3 && a <= 3 && w.max(a) + c <= 3);
        let bad_ack = control == 5;
        let bad_exec = control == 3 && case == 3;
        act(
            &mut m,
            &format!("AckMember0AfterCurrentStopPredicatesCase{case}"),
            all([eq(PH, 4), eq(CASE, case), eq(END0, 0), eq(FAULT, 0)]),
            &[
                (CLOCK, Int(2)),
                (END0, Int(2)),
                (ACK0, Int(i64::from(!bad_ack))),
                (S0, Int(3)),
            ],
        );
        act(
            &mut m,
            &format!("AckKnownMember1AfterCheckedHostClockCase{case}"),
            all([
                eq(PH, 4),
                eq(CASE, case),
                eq(END0, 2),
                eq(END1, 0),
                eq(S1, 2),
                eq(FAULT, 0),
                if case == 3 { eq(CUR, 2) } else { Bool(true) },
                if bad_exec {
                    Bool(true)
                } else {
                    Var(CUR).eq(Var(TOKEN))
                },
            ]),
            &[
                (CLOCK, Int(end1)),
                (END1, Int(end1)),
                (ACK1, Int(1)),
                (S1, Int(3)),
                (PH, Int(5)),
            ],
        );
        if case == 2 {
            act(
                &mut m,
                "UnknownSlotAwaitsCompleteSameGrantExclusion",
                all([eq(PH, 4), eq(CASE, 2), eq(END0, 2), eq(S1, 5)]),
                &[(PH, Int(5))],
            );
        }
        let close_guard = all([
            eq(PH, 5),
            eq(CASE, case),
            eq(FAULT, 0),
            exact.clone(),
            if control == 4 {
                Bool(true)
            } else {
                eq(UNCERTAIN, 0)
            },
            eq(SUPPORT, 1),
            if bad_ack { Bool(true) } else { eq(ACK0, 1) },
            if case == 2 { eq(END1, 0) } else { eq(ACK1, 1) },
            if bad_exec {
                Bool(true)
            } else {
                Var(CUR).eq(Var(TOKEN))
            },
        ]);
        let mut closed = vec![
            (PH, Int(6)),
            (CLOCK, Int(3)),
            (ROSTER, Int(1)),
            (REQUIRED, Int(i64::from(control != 6))),
            (SEAL, Int(1)),
            (WORK, Int(0)),
        ];
        if case == 2 {
            closed.extend([(END1, Int(3)), (ACK1, Int(1)), (S1, Int(0))]);
        }
        act(
            &mut m,
            &format!("CompleteRequiredPredicatesThenCloseSameGrantRoster{case}"),
            close_guard,
            &closed,
        );
        if control == 2 && case == 2 {
            act(
                &mut m,
                "BrokenBackdateUnknownSlotToItsLaterDiscovery",
                all([eq(PH, 5), eq(CASE, 2)]),
                &[(START1, Int(2))],
            );
        }
        let actual_w = if control == 1 && late_endpoint {
            w - 1
        } else {
            w
        };
        let actual_a = if control == 11 {
            a - 1
        } else if control == 12 && !late_endpoint && a < 3 {
            a + 1
        } else {
            a
        };
        let actual_c = if control == 9 && c > 0 {
            c - 1
        } else if control == 10 && c > 0 {
            c + 1
        } else {
            c
        };
        let actual_q = actual_w.max(actual_a).saturating_add(actual_c).min(3);
        let within = case != 6;
        let cover = (a <= w || control == 7) && (within || control == 23);
        let debt = if case == 6 { 18 } else { 9 };
        act(
            &mut m,
            &format!("ChargeExactWindowActiveAndAllControllerCase{case}"),
            all([
                eq(PH, 6),
                eq(CASE, case),
                eq(SEAL, 1),
                eq(TOTAL, 18).or(Var(TOTAL).lt(Int(18))),
                eq(FAULT, 0),
                exact.clone(),
            ]),
            &[
                (PH, Int(7)),
                (W, Int(actual_w)),
                (A, Int(actual_a)),
                (C, Int(actual_c)),
                (Q, Int(actual_q)),
                (
                    PIPE,
                    Int(if c == 0 || control == 9 {
                        0
                    } else if control == 10 {
                        2
                    } else {
                        1
                    }),
                ),
                (
                    ROLL,
                    Int(if control == 11 {
                        0
                    } else if control == 12 {
                        2
                    } else {
                        1
                    }),
                ),
                (COVER, Int(i64::from(cover))),
                (FAULT, Int(if cover { 0 } else { 6 })),
                (DEBT, Var(DEBT).plus(Int(debt))),
                (OWED, Var(OWED).plus(Int(debt))),
                (TOTAL, Var(TOTAL).plus(Int(debt))),
                (POSTDONE, Int(i64::from(!matches!(case, 7 | 8)))),
            ],
        );
        act(
            &mut m,
            &format!("FailedClosureStillRecordsQuantumAndDebtCase{case}"),
            all([
                eq(PH, 6),
                eq(CASE, case),
                eq(SEAL, 1),
                eq(TOTAL, 18).or(Var(TOTAL).lt(Int(18))),
                eq(FAULT, 0).negate(),
            ]),
            &[
                (PH, Int(7)),
                (W, Int(w)),
                (A, Int(a)),
                (C, Int(c)),
                (Q, Int(w.max(a) + c)),
                (COVER, Int(0)),
                (PIPE, Int(i64::from(c > 0))),
                (ROLL, Int(1)),
                (DEBT, Var(DEBT).plus(Int(debt))),
                (OWED, Var(OWED).plus(Int(debt))),
                (TOTAL, Var(TOTAL).plus(Int(debt))),
                (POSTDONE, Int(i64::from(!matches!(case, 7 | 8)))),
            ],
        );
        safe(
            &mut m,
            &format!("WindowUnionRetainsEveryReservedIntervalCase{case}"),
            eq(CASE, case).negate().or(eq(PH, 7).negate()).or(eq(W, w)),
        );
        safe(
            &mut m,
            &format!("ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase{case}"),
            eq(CASE, case).negate().or(eq(PH, 7).negate()).or(eq(A, a)),
        );
        safe(
            &mut m,
            &format!("AllActiveControllerAndPipeCpuCountExactlyOnceCase{case}"),
            eq(CASE, case).negate().or(eq(PH, 7).negate()).or(eq(C, c)),
        );
        safe(
            &mut m,
            &format!("QuantumIsMaxWindowOrActivePlusControllerCase{case}"),
            eq(CASE, case)
                .negate()
                .or(eq(PH, 7).negate())
                .or(eq(Q, w.max(a) + c)),
        );
    }
    act(
        &mut m,
        "UnsupportedCapabilityRefusesBeforeAnyNativeEffect",
        all([eq(CONFIG, 1), eq(SUPPORT, 0), eq(EFFECT, 0)]),
        &[
            (FAULT, Int(6)),
            (KNOWN, Int(0)),
            (COVER, Int(0)),
            (QUAR, Int(1)),
        ],
    );
    act(
        &mut m,
        "RefuseMissingWholeCommandOrTransientCoverage",
        all([
            eq(EFFECT, 1),
            eq(FAULT, 0),
            eq(UNCERTAIN, 1).or(eq(SUPPORT, 0)),
        ]),
        &[(FAULT, Int(6)), (KNOWN, Int(0)), (COVER, Int(0))],
    );
    act(
        &mut m,
        "ObserveNewLateCpuInvalidatesOldStoppedSeal",
        all([
            eq(PH, 7),
            eq(CASE, 7).or(eq(CASE, 8)),
            eq(LATESEEN, 0),
            eq(FAULT, 0),
        ]),
        &[(LATESEEN, Int(1)), (SEAL, Int(0)), (REVALID, Int(0))],
    );
    let late_count = if control == 8 {
        0
    } else if control == 24 {
        2
    } else {
        1
    };
    let late_debt = if control == 8 {
        0
    } else if control == 24 {
        18
    } else {
        9
    };
    for explained in 0..=1 {
        act(
            &mut m,
            &format!("ReconcileLateKernelOnceWithFreshAdditiveDebtExplained{explained}"),
            all([
                eq(PH, 7),
                eq(LATESEEN, 1),
                eq(PROCESSED, 0),
                Var(TOTAL).lt(Int(19)),
                eq(EXPLAIN, explained),
                eq(DEBT, 0),
                eq(OWED, 0),
            ]),
            &[
                (LATE, Int(late_count)),
                (PROCESSED, Int(1)),
                (POST, Int(1)),
                (COVER, if explained == 0 { Int(0) } else { Var(COVER) }),
                (FAULT, if explained == 0 { Int(6) } else { Var(FAULT) }),
                (DEBT, Var(DEBT).plus(Int(late_debt))),
                (OWED, Var(OWED).plus(Int(9))),
                (TOTAL, Var(TOTAL).plus(Int(late_debt))),
            ],
        );
    }
    act(
        &mut m,
        "UnexplainedLateUserOrIncarnationGrowthRefuses",
        all([eq(PROCESSED, 1), eq(EXPLAIN, 0), eq(FAULT, 0)]),
        &[(FAULT, Int(6)), (COVER, Int(0))],
    );
    act(
        &mut m,
        "RevalidateCurrentStoppedRosterAfterExplainedLateKernel",
        all([
            eq(PH, 7),
            eq(PROCESSED, 1),
            eq(EXPLAIN, 1),
            eq(LATE, 1),
            eq(FAULT, 0),
            eq(ROSTER, 1),
            eq(REQUIRED, 1),
            eq(ACK0, 1),
            eq(ACK1, 1),
            exact.clone(),
            Var(CUR).eq(Var(TOKEN)),
        ]),
        &[(SEAL, Int(1)), (REVALID, Int(1))],
    );
    act(
        &mut m,
        "GenuinelyPostSealCorroborationAndFinalizationAreSeparateOrdinaryUnit",
        all([
            eq(PH, 7),
            eq(POSTDONE, 0),
            eq(PROCESSED, 1),
            eq(REVALID, 1),
            eq(DEBT, 0),
            eq(FAULT, 0),
            quiet.clone(),
            Var(TOTAL).lt(Int(19)),
        ]),
        &[
            (POST, Int(1)),
            (POSTDONE, Int(1)),
            (DEBT, Int(if control == 28 { 0 } else { 9 })),
            (OWED, Int(9)),
            (TOTAL, Var(TOTAL).plus(Int(9))),
        ],
    );
    let rest = m.actions.len();
    for debt in 1..=27 {
        act(
            &mut m,
            &format!("ObserveRestAfterWorkEndWithoutIdleCredit{debt}"),
            all([
                eq(DEBT, debt),
                eq(OWED, debt),
                eq(WORK, 0),
                eq(PH, 6).negate(),
            ]),
            &[(DEBT, Int(debt - 1)), (OWED, Int(debt - 1))],
        );
    }
    if control == 14 {
        act(
            &mut m,
            "BrokenSleepOrIoDebtDeadlineReset",
            all([eq(DEBT, 0).negate(), eq(WORK, 0)]),
            &[(DEBT, Int(0))],
        );
    }
    let can_work = all([
        quiet.clone(),
        eq(DEBT, 0),
        eq(OWED, 0),
        eq(POSTDONE, 1),
        eq(REVALID, 1),
        eq(COVER, 1),
        available.clone(),
    ]);
    let mut next = open.to_vec();
    next.push((GRANTS, Int(2)));
    act(
        &mut m,
        "ContinueOnlyAfterClosureCorroborationAndAllDebt",
        all([
            eq(PH, 7),
            eq(GRANTS, 1),
            eq(CASE, 7).negate(),
            eq(CASE, 8).negate(),
            can_work.clone(),
        ]),
        &next,
    );
    if control == 15 {
        act(
            &mut m,
            "BrokenContinueWhileClosureOrDebtPending",
            all([eq(PH, 7), eq(DEBT, 0).negate(), eq(FAULT, 0)]),
            &[(WORK, Int(1))],
        );
    }
    if control == 16 {
        act(
            &mut m,
            "BrokenMainProofOverlapsOpenHelperWindow",
            eq(PH, 4),
            &[(WORK, Int(2))],
        );
    }
    for fault in 1..=5 {
        let broken = control == fault + 16;
        act(
            &mut m,
            &format!("ObserveClockStopRevokeCancelPanic{fault}"),
            all([eq(EFFECT, 1), eq(EXIT, 0), eq(FAULT, 0), eq(OWN, 1)]),
            &[
                (FAULT, Int(i64::from(fault))),
                (KNOWN, Int(i64::from(fault != 1))),
                (COVER, Int(0)),
                (OWN, Int(i64::from(!broken))),
                (TRUST, Int(0)),
            ],
        );
    }
    for foreground in 0..=1 {
        act(
            &mut m,
            &format!("ObserveForegroundFence{foreground}WithoutDebtReset"),
            all([
                eq(EFFECT, 1),
                eq(OWN, 1),
                eq(EXIT, 0),
                eq(FG, foreground).negate(),
                eq(CASE, 1),
            ]),
            &[(FG, Int(foreground)), (TRUST, Int(0)), (WORK, Int(0))],
        );
    }
    act(
        &mut m,
        "ObserveDeadlineRetainsDebtAndOriginalEffect",
        all([
            eq(EFFECT, 1),
            eq(OWN, 1),
            eq(EXIT, 0),
            eq(DEADLINE, 0),
            eq(CASE, 10),
        ]),
        &[
            (DEADLINE, Int(1)),
            (FAULT, Int(4)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    for current in 0..2 {
        act(
            &mut m,
            &format!("RestartOriginalEffectA1B2A3From{current}"),
            all([eq(EFFECT, 1), eq(INC, current), eq(OWN, 1), eq(EXIT, 0)]),
            &[
                (INC, Int(current + 1)),
                (FAULT, Int(3)),
                (COVER, Int(0)),
                (TRUST, Int(0)),
            ],
        );
    }
    if control == 22 {
        act(
            &mut m,
            "BrokenA3ReusesA1WindowAndCurrentnessSeal",
            all([eq(INC, 2), eq(CAP, 0), eq(EFFECT, 1), eq(EXIT, 0)]),
            &[(COVER, Int(1)), (SEAL, Int(1)), (FAULT, Int(0))],
        );
    }
    let retire = m.actions.len();
    act(
        &mut m,
        "RetireRetainsOriginalWindowsAndSignalingCustody",
        all([eq(EFFECT, 1), eq(RETIRED, 0), eq(OWN, 1)]),
        &[(RETIRED, Int(1)), (PH, Int(8)), (WORK, Int(0))],
    );
    if control == 13 {
        act(
            &mut m,
            "BrokenRetiredTokenOrAbsentCensusCertifiesCoverage",
            all([eq(RETIRED, 1), eq(EXIT, 0), eq(ROSTER, 0)]),
            &[(COVER, Int(1)), (SEAL, Int(1))],
        );
    }
    act(
        &mut m,
        "ObserveExactOriginalExitAndRetainedUsage",
        all([eq(RETIRED, 1), eq(EXIT, 0), eq(OWN, 1)]),
        &[(EXIT, Int(1)), (S0, Int(4)), (S1, Int(4))],
    );
    act(
        &mut m,
        "ActuallyReapOriginalGroupAfterRetirement",
        all([eq(RETIRED, 1), eq(EXIT, 1), eq(REAP, 0), eq(OWN, 1)]),
        &[(REAP, Int(1)), (PH, Int(9))],
    );
    act(
        &mut m,
        "ReleaseOnlyAfterRealReapAndEveryKnownDebt",
        all([
            eq(RETIRED, 1),
            eq(REAP, 1),
            eq(OWN, 1),
            eq(DEBT, 0),
            eq(OWED, 0),
        ]),
        &[(OWN, Int(0))],
    );
    act(
        &mut m,
        "ResumeMainOnlyAfterFullSettlementAndPaidCoverage",
        all([
            eq(OWN, 0),
            eq(REAP, 1),
            eq(COVER, 1),
            eq(SEAL, 1),
            eq(POSTDONE, 1),
            eq(REVALID, 1),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(KNOWN, 1),
            eq(FAULT, 0),
            exact.clone(),
            eq(FG, 0),
            eq(DEADLINE, 0),
        ]),
        &[(WORK, Int(2))],
    );
    act(
        &mut m,
        "TrustOnlyClosedQualifiedActuallySettledAccounting",
        all([
            eq(WORK, 2),
            eq(COVER, 1),
            eq(REAP, 1),
            eq(OWN, 0),
            eq(DEBT, 0),
            eq(KNOWN, 1),
            eq(FAULT, 0),
            eq(POSTDONE, 1),
            eq(REVALID, 1),
            exact.clone(),
        ]),
        &[(TRUST, Int(1))],
    );
    let quarantine = m.actions.len();
    act(
        &mut m,
        "LocalUnavailableDispositionRetainsOriginalEffect",
        all([
            eq(FAULT, 0).negate().or(eq(RETIRED, 1)),
            eq(OWN, 1),
            eq(REAP, 0),
        ]),
        &[(COVER, Int(0)), (TRUST, Int(0)), (QUAR, Int(1))],
    );
    safe(
        &mut m,
        "NativeEffectRequiresSupportedCapabilityAtAdmission",
        eq(EFFECT, 0).or(eq(SUPPORT, 1)),
    );
    safe(
        &mut m,
        "StickyFailedAdmissionForbidsOrdinaryExpensiveObservation",
        eq(OBSERVE, 2).negate(),
    );
    safe(
        &mut m,
        "InitialCorroborationRequiresSettledCreationOwnership",
        eq(OBSERVE, 3).negate(),
    );
    safe(
        &mut m,
        "InitialCorroborationRequiresBothPriorDebtsPaid",
        eq(OBSERVE, 4).negate(),
    );
    safe(
        &mut m,
        "BothPossibleWindowsOpenAtOriginalCheckedT0",
        eq(GRANTS, 0).or(all([eq(START0, 1), eq(START1, 1)])),
    );
    safe(
        &mut m,
        "TentativeEndpointsNeedPositiveCurrentTaskAcknowledgement",
        all([
            eq(END0, 0).or(eq(ACK0, 1)),
            eq(END1, 0).or(all([eq(ACK1, 1), Var(CUR).eq(Var(TOKEN))])),
        ]),
    );
    safe(
        &mut m,
        "UsableSealFollowsEveryRequiredClosedRosterPredicate",
        eq(SEAL, 0).or(all([
            eq(ROSTER, 1),
            eq(REQUIRED, 1),
            eq(UNCERTAIN, 0),
            eq(SUPPORT, 1),
            eq(ACK0, 1),
            eq(ACK1, 1),
            Var(CUR).eq(Var(TOKEN)),
        ])),
    );
    safe(
        &mut m,
        "UnexplainedActiveCpuOverWindowCannotCertify",
        eq(COVER, 0).or(Var(W).lt(Var(A)).negate()),
    );
    safe(
        &mut m,
        "KnownOverOneMsQuantumCannotCertify",
        eq(COVER, 0).or(eq(CASE, 6).negate()),
    );
    safe(
        &mut m,
        "LateKernelAttributionIsExplainedAndChargedOnce",
        eq(PROCESSED, 0).or(all([eq(LATE, 1), eq(EXPLAIN, 1).or(eq(COVER, 0))])),
    );
    safe(
        &mut m,
        "DebtNeverDropsObservedOutstandingWork",
        Var(DEBT).lt(Var(OWED)).negate(),
    );
    safe(
        &mut m,
        "NoNewContinuationWhileClosedDebtIsPending",
        eq(PH, 7).negate().or(eq(WORK, 1).negate()),
    );
    safe(
        &mut m,
        "MainProofNeverOverlapsOriginalLiveHelper",
        eq(WORK, 2)
            .negate()
            .or(all([eq(OWN, 0), eq(REAP, 1), eq(DEBT, 0), eq(OWED, 0)])),
    );
    safe(
        &mut m,
        "FailureCannotReleaseUnreapedOriginalEffect",
        eq(EFFECT, 0)
            .or(eq(OWN, 1))
            .or(all([eq(REAP, 1), eq(EXIT, 1), eq(RETIRED, 1)])),
    );
    safe(
        &mut m,
        "CoverageKeepsOriginalEpochAndCompleteRoster",
        eq(COVER, 0).or(all([
            exact,
            eq(ROSTER, 1),
            eq(REQUIRED, 1),
            eq(UNCERTAIN, 0),
            eq(SUPPORT, 1),
        ])),
    );
    safe(
        &mut m,
        "UnknownCostNeverCertifiesFiniteAccounting",
        eq(KNOWN, 1).or(eq(COVER, 0)),
    );
    safe(
        &mut m,
        "TrustRequiresCompleteCurrentPaidActualSettlement",
        eq(TRUST, 0).or(all([
            eq(COVER, 1),
            eq(KNOWN, 1),
            eq(FAULT, 0),
            eq(REAP, 1),
            eq(OWN, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            eq(POSTDONE, 1),
            eq(REVALID, 1),
            eq(SEAL, 1),
        ])),
    );
    // OS exit is not fair; local stop/refusal and retirement remain available.
    progress(
        &mut m,
        "FaultHasRetirementOrOwnedUnavailableDisposition",
        eq(FAULT, 0).negate(),
        eq(RETIRED, 1).or(eq(REAP, 1)).or(eq(QUAR, 1)),
        vec![retire, quarantine, rest],
    );
    if measured_law {
        extend_member_window_measured_law(&mut m, control, forged_terminal_control);
        extend_member_window_adaptive_grants(&mut m, control);
        extend_adaptive_latency_provenance(&mut m, control);
    }
    if control != 0 {
        // TLC otherwise expands the state-only ORs as alternative next-state
        // branches, producing duplicate states/edges for every true disjunct.
        // Select every potentially failed boundary and force one Boolean value.
        // Every invariant and reachable state/labelled edge remains unchanged;
        // the test compares this with the original branching observer exactly.
        let relevant: &[&str] = match control {
            1 => &[
                "QuantumIsMaxWindowOrActivePlusControllerCase0",
                "QuantumIsMaxWindowOrActivePlusControllerCase2",
                "QuantumIsMaxWindowOrActivePlusControllerCase3",
                "WindowUnionRetainsEveryReservedIntervalCase0",
                "WindowUnionRetainsEveryReservedIntervalCase2",
                "WindowUnionRetainsEveryReservedIntervalCase3",
            ],
            2 => &["BothPossibleWindowsOpenAtOriginalCheckedT0"],
            3 => &["TentativeEndpointsNeedPositiveCurrentTaskAcknowledgement"],
            4 => &["UsableSealFollowsEveryRequiredClosedRosterPredicate"],
            5 => &["TentativeEndpointsNeedPositiveCurrentTaskAcknowledgement"],
            6 => &["UsableSealFollowsEveryRequiredClosedRosterPredicate"],
            7 => &["UnexplainedActiveCpuOverWindowCannotCertify"],
            8 => &[
                "DebtNeverDropsObservedOutstandingWork",
                "LateKernelAttributionIsExplainedAndChargedOnce",
            ],
            9 => &[
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase1",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase6",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase7",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase8",
                "QuantumIsMaxWindowOrActivePlusControllerCase1",
                "QuantumIsMaxWindowOrActivePlusControllerCase6",
                "QuantumIsMaxWindowOrActivePlusControllerCase7",
                "QuantumIsMaxWindowOrActivePlusControllerCase8",
            ],
            10 => &[
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase1",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase6",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase7",
                "AllActiveControllerAndPipeCpuCountExactlyOnceCase8",
            ],
            11 => &[
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase0",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase1",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase10",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase2",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase3",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase5",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase6",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase7",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase8",
                "QuantumIsMaxWindowOrActivePlusControllerCase5",
            ],
            12 => &[
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase1",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase10",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase6",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase7",
                "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase8",
            ],
            13 => &[
                "CoverageKeepsOriginalEpochAndCompleteRoster",
                "KnownOverOneMsQuantumCannotCertify",
                "UnknownCostNeverCertifiesFiniteAccounting",
                "UsableSealFollowsEveryRequiredClosedRosterPredicate",
            ],
            14 => &["DebtNeverDropsObservedOutstandingWork"],
            15 => &["NoNewContinuationWhileClosedDebtIsPending"],
            16 => &["MainProofNeverOverlapsOriginalLiveHelper"],
            17 => &["FailureCannotReleaseUnreapedOriginalEffect"],
            18 => &["FailureCannotReleaseUnreapedOriginalEffect"],
            19 => &["FailureCannotReleaseUnreapedOriginalEffect"],
            20 => &["FailureCannotReleaseUnreapedOriginalEffect"],
            21 => &["FailureCannotReleaseUnreapedOriginalEffect"],
            22 => &[
                "CoverageKeepsOriginalEpochAndCompleteRoster",
                "KnownOverOneMsQuantumCannotCertify",
                "LateKernelAttributionIsExplainedAndChargedOnce",
                "UnexplainedActiveCpuOverWindowCannotCertify",
                "UnknownCostNeverCertifiesFiniteAccounting",
                "UsableSealFollowsEveryRequiredClosedRosterPredicate",
            ],
            23 => &["KnownOverOneMsQuantumCannotCertify"],
            24 => &["LateKernelAttributionIsExplainedAndChargedOnce"],
            25 => &["StickyFailedAdmissionForbidsOrdinaryExpensiveObservation"],
            26 => &["InitialCorroborationRequiresSettledCreationOwnership"],
            27 => &["InitialCorroborationRequiresBothPriorDebtsPaid"],
            28 => &["DebtNeverDropsObservedOutstandingWork"],
            29 | 30 => &["MeasuredNeedsClosure"],
            31 | 32 => &[
                "ChargedCoversBurned",
                "ConservationFailureRetainsRawAndRefusesTrust",
            ],
            33 => &["WallBounded"],
            34 => &["MeasuredNeedsClosure", "PerSlotMeasuredCpuFitsWindow"],
            35 => &["MeasuredNeedsClosure", "RollupRequiresUniqueAdmittedChild"],
            36 => &["MeasuredNeedsClosure", "RollupNeedsCausalExitAndLeaderAck"],
            37 => &["MeasuredNeedsClosure", "RollupCountedOnce"],
            38 => &["InitialCorroborationStageRequiresFreshPaidRest"],
            39 => &["InitialEligibleTerminalRiAttemptPrecedesOwnWait4"],
            40 => &[
                "ChargedCoversBurned",
                "ConservationFailureRetainsRawAndRefusesTrust",
            ],
            41 => &["TerminalResidualIsFullyChargedOutsideConservationCredit"],
            42 => &["TeardownCeilingIsIndependentFixedPerActualReap"],
            43 => &["TeardownChildCountNeedsQualifiedRetainedReapProof"],
            44 => &["BornStoppedCreationFaultReapNeedsPaidOriginalPhysicalLatch"],
            45 => &["IssuedAdaptiveGrantUsesMeasuredCostAndStopLatencyCap"],
            46 => &["AdaptiveLatencyRetainsMaximumValidLiveAck"],
            47 => &[
                "AdaptiveLatencyRetainsMaximumValidLiveAck",
                "AdaptiveGrowthNeedsMeasuredLiveSuccessfulStopAck",
            ],
            48 => &["WindowBasisAboveNewTargetHalvesPriorRun"],
            49 => &[
                "PendingDirectExitCannotAdmitExpensiveWorkOrTrust",
                "SecondDirectGrantCommitNeedsOwnWait4AndPriorCredit",
            ],
            _ => unreachable!(),
        };
        assert!(
            relevant
                .iter()
                .all(|name| m.invariants.iter().any(|i| i.name == *name))
        );
        let valid = all(m
            .invariants
            .iter()
            .filter(|i| full_observer || relevant.contains(&i.name.as_str()))
            .map(|i| i.condition.clone()));
        for action in &mut m.actions {
            let guard = action.guard.clone().and(valid.clone());
            action.guard = if full_observer {
                guard
            } else {
                guard.eq(Bool(true))
            };
        }
        let failed = valid.negate();
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            if full_observer {
                failed
            } else {
                failed.eq(Bool(true))
            },
            &[],
        );
    }
    m
}

/// Decision66236 adds correlated numeric observations to the SAME two-slot
/// owner, grant, ACK, roster, debt, retirement and actual-reap actions above.
/// Profile0 retains every65489 geometry/control with counters unavailable and
/// window basis. The appended profiles use explicit ns constants, not a larger
/// product of independently chosen CPU/clock/counter values. Original variable
/// bounds, two grants, A1/B2/A3, one late observation and one post unit remain.
/// Startup125us and member late250us are finite accounting projections, not OS
/// timing guarantees. Debt rounds UP to whole abstract ms, never below9*Q.
/// Counter-read provenance and one-thread/source closure remain premises with
/// real API/process witnesses; the125/3 profile does not prove Mach arithmetic.
fn extend_member_window_measured_law(m: &mut Model, control: u8, forged_terminal_control: bool) {
    const LAW: usize = 54;
    const BASIS: usize = 55;
    const READ0: usize = 56;
    const READ1: usize = 57;
    const BURNED: usize = 58;
    const CHARGED: usize = 59;
    const WALL: usize = 60;
    const A0: usize = 61;
    const A1: usize = 62;
    const W0: usize = 63;
    const W1: usize = 64;
    const MONOTONIC: usize = 65;
    const CONVERTED: usize = 66;
    const TERMINAL: usize = 67;
    const CHECKED: usize = 68;
    const QNS: usize = 69;
    const REPORTED: usize = 70;
    const SEEN: usize = 71;
    const EXCESS: usize = 72;
    const BASIS_EPOCH: usize = 73;
    const CREATED_DEBT: usize = 74;
    const TERMINAL_DEBT: usize = 75;
    const ORIGIN0: usize = 76;
    const ORIGIN1: usize = 77;
    const PENDING: usize = 78;
    const WAIT4: usize = 79;
    const COMMITTED: usize = 80;
    const ROLL_UNIQUE: usize = 81;
    const ROLL_MIXED: usize = 82;
    const ROLL_EXIT: usize = 83;
    const ROLL_ACK: usize = 84;
    const ROLL_COUNT: usize = 85;
    const ROLL_CUMULATIVE: usize = 86;
    const ROLL_BASELINE: usize = 87;
    const PRIOR_LIVE: usize = 88;
    const ROLL_DUPLICATE: usize = 89;
    const RAW_Q: usize = 90;
    const RAW_BASIS: usize = 91;
    const ROLL_PENDING: usize = 92;
    const ROLL_VALIDATED: usize = 93;
    const ROLL_REVOKED: usize = 94;
    const FALLBACK_Q: usize = 95;
    const ROLL_DELTA: usize = 96;
    const FALLBACK_DEBT: usize = 97;
    const ROLL_ZERO_CHILDREN: usize = 98;
    const INITIAL_STAGE: usize = 99;
    const INITIAL_CPU: usize = 100;
    const INITIAL_PRIOR_DEBT: usize = 101;
    const INITIAL_READY: usize = 102;
    const STARTUP_KNOWN: usize = 103;
    const STARTUP_CLOSED: usize = 104;
    const STARTUP_PAID: usize = 105;
    const STARTUP_TIMEBASE: usize = 106;
    const STARTUP_CHARGED: usize = 107;
    const INITIAL_ATTEMPT: usize = 108;
    const INITIAL_READ_KNOWN: usize = 109;
    const INITIAL_RAW_SYSTEM: usize = 110;
    const INITIAL_L: usize = 111;
    const INITIAL_REST_PAID: usize = 112;
    const INITIAL_LAST_CPU: usize = 113;
    const INITIAL_COST_BOUND: usize = 114;
    const INITIAL_ATTEMPT_DEBT: usize = 115;
    const INITIAL_UNKNOWN: usize = 116;
    const INITIAL_UNKNOWN_DEBT: usize = 117;
    const ACTUAL_REAPS: usize = 118;
    const TEARDOWN_LIMIT: usize = 119;
    const TEARDOWN_ALLOWANCE: usize = 120;
    const RESIDUAL: usize = 121;
    const RESIDUAL_CHARGED: usize = 122;
    const RESIDUAL_TOTAL: usize = 123;
    const RESIDUAL_OBSERVED: usize = 124;
    const RESIDUAL_PAID: usize = 125;
    const RESIDUAL_KNOWN: usize = 126;
    const INDEPENDENT_AT_BILL: usize = 127;
    const QUALIFIED_CHILD_REAP: usize = 128;
    const BORN_LATCH: usize = 129;
    const BORN_SOURCE: usize = 130;
    const BORN_COST_SEEN: usize = 131;
    const BORN_COST_RETAINED: usize = 132;
    const BORN_BIRTH_CLOCKS: usize = 133;
    const BORN_COST_PAID: usize = 134;
    const BORN_NATIVE_COST: usize = 135;
    const BORN_NATIVE_DEBT: usize = 136;
    const BORN_EXPECTED_SLOTS: usize = 137;
    const BORN_CHILD_ACTIVATED: usize = 138;
    const BORN_FRESH_SOURCE: usize = 139;
    const BORN_PHYSICAL: usize = 140;
    const BORN_ATTEMPT_RETAINED: usize = 141;
    const BORN_ATTEMPT_END_CLOCKS: usize = 142;
    const BORN_ACCOUNTING_UNKNOWN: usize = 143;
    const BORN_RAW_NATIVE_KNOWN: usize = 144;
    assert_eq!(m.variables.len(), LAW);
    for (name, max) in [
        ("measuredLawProfile", 46),
        ("basisWindowMeasured", 1),
        ("slot0CountersReadNonePreAckPostAckRetainedWait4", 3),
        (
            "slot1CountersReadNonePreAckPostAckRetainedWait4LeaderRollup",
            4,
        ),
        ("cpuBurned", 8_000_000),
        ("cpuCharged", 8_000_000),
        ("wall", 30_000_000),
        ("slot0ActualCpuNs", 2_000_000),
        ("slot1ActualCpuNs", 2_000_000),
        ("slot0WindowNs", 25_000_000),
        ("slot1WindowNs", 25_000_000),
        ("counterMonotonicMask", 3),
        ("counterConversionMask", 3),
        ("terminalConservationPendingPassedRefused", 2),
        ("conservationChecked", 1),
        ("basisQuantumNs", 3_000_000),
        ("reportedMemberCpuNs", 3_000_000),
        ("currentGrantCpuObserved", 1),
        ("conservationExcessNs", 8_000_000),
        ("basisCapturedEpoch", 2),
        ("createdGrantDebtMs", 27),
        ("terminalReconciliationDebtMs", 21),
        ("slot0CounterEndpointNoneStoppedExitedClosedAbsence", 3),
        ("slot1CounterEndpointNoneStoppedExitedClosedAbsence", 3),
        ("directExitAccountingPending", 1),
        ("actualWait4CpuNs", 8_000_000),
        ("committedGrantCount", 2),
        ("rollupUniqueAdmittedChild", 1),
        ("rollupMixedUnadmittedChild", 1),
        ("rollupChildExitProvenBeforeRead", 1),
        ("rollupLeaderAckBeforeRead", 1),
        ("qualifiedChildRollupAttributionCount", 2),
        // Profile43 retains actual first-grant live300us, then cumulative
        // child350us with that SAME300us baseline and a50us nonduplicate delta.
        ("rollupCumulativeChildNs", 350_000),
        ("rollupBaselineChildNs", 300_000),
        ("priorLiveChildCreditNs", 300_000),
        ("rollupDuplicateLiveChildNs", 125_000),
        ("rawGrantQuantumNs", 3_000_000),
        ("rawGrantBasisWindowMeasured", 1),
        ("rollupConservationPending", 1),
        ("rollupConservationValidated", 1),
        ("rollupBasisRevoked", 1),
        ("retainedFallbackQuantumNs", 3_000_000),
        ("rollupCurrentDeltaNs", 250_000),
        ("retainedFallbackDebtMs", 27),
        ("rollupLeaderChildCountZeroBeforeRead", 1),
        ("initialStoppedCorroborationStage", 5),
        ("initialStoppedCorroborationInclusiveCpuNs", 1_164_833),
        ("initialStoppedCorroborationPriorDebtMs", 9),
        ("initialStoppedCorroborationPaidSeal", 1),
        ("initialStartupRawBaselineKnown", 1),
        ("initialStartupOriginalRosterClosed", 1),
        ("initialStartupRetainedPaidEvidence", 1),
        ("initialStartupRetainedTimebaseKnownUnchanged", 1),
        ("initialStartupCreditCommitted", 1),
        ("initialTerminalRiAttemptedBeforeWait4", 1),
        ("initialTerminalRiKnown", 1),
        ("initialTerminalIndependentRawSystemDeltaNs", 75_000),
        ("initialTerminalCreditedKernelNs", 75_000),
        ("initialTerminalObservationCostPaidBeforeWait4", 1),
        ("initialStoppedCorroborationLastInclusiveCpuNs", 1_000_000),
        ("initialTerminalRetainedCostBoundNs", 3_000_000),
        ("initialTerminalRetainedAttemptDebtMs", 27),
        ("initialTerminalConservationUnknown", 1),
        ("initialTerminalUnknownVerdictDebtMs", 27),
        ("actualProvenReapedProcessCount", 2),
        ("fixedExitTeardownLimitPerProcessNs", 3_000_000),
        ("separateExitTeardownConservationAllowanceNs", 6_000_000),
        ("actualTerminalResidualCpuNs", 8_000_000),
        ("terminalResidualChargedAsCostNs", 8_000_000),
        ("terminalResidualInclusiveCostBoundNs", 8_250_000),
        ("terminalResidualCostObserved", 1),
        ("terminalResidualFullCostPaid", 1),
        ("terminalResidualCostKnown", 1),
        ("independentMemberCreditsBeforeResidualBillNs", 8_000_000),
        ("qualifiedUniqueGitReapedChildCountedOnce", 1),
        ("bornStoppedOriginalPhysicalLatch", 1),
        ("bornStoppedExactSourceAtAttach", 1),
        ("bornStoppedNativeCostObserved", 1),
        ("bornStoppedNativeCostRetainedFunded", 1),
        ("bornStoppedReceiptClosingClocksKnown", 1),
        ("bornStoppedNativeCostPaidReceipt", 1),
        ("bornStoppedNativeInclusiveCostBoundNs", 3_000_000),
        ("bornStoppedNativeCreatedDebtMs", 27),
        ("bornStoppedRetainedExpectedMembers", 2),
        ("bornStoppedImplicitReservedSlotActivated", 1),
        ("bornStoppedFreshOriginalTerminalSourceKnown", 1),
        ("bornStoppedFreshPhysicalClosureValidated", 1),
        ("bornStoppedTerminalAttemptRetained", 1),
        ("bornStoppedTerminalAttemptClosingClocksKnown", 1),
        ("bornStoppedStartupAccountingUnproven", 1),
        ("bornStoppedOriginalNativeAdmissionCostKnown", 1),
    ] {
        m.variables.push(Variable::new(name, 0, max, 0));
    }
    {
        // Native vectors address fields by index; TLA export and test lookup
        // address them by name. A duplicate could disagree silently between
        // those consumers even though native full/reduced graphs are equal.
        let names = m
            .variables
            .iter()
            .map(|v| v.name.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            names.len(),
            m.variables.len(),
            "MemberWindow TLA variable names must be unique"
        );
    }
    let new_law = eq(LAW, 0).negate();
    let initial_only = Int(19).lt(Var(LAW)).and(Var(LAW).lt(Int(38)));
    let initial_eligible = all([
        initial_only.clone(),
        eq(49, 0),
        eq(INITIAL_READY, 1),
        eq(STARTUP_CHARGED, 1),
        eq(STARTUP_KNOWN, 1),
        eq(STARTUP_CLOSED, 1),
        eq(STARTUP_PAID, 1),
        eq(STARTUP_TIMEBASE, 1),
        Var(5).eq(Var(6)),
    ]);
    let early_born = Int(28).lt(Var(LAW)).and(Var(LAW).lt(Int(38)));
    // Physical custody is independent of numerical startup/L credit. Expected
    // member2 is an UNACTIVATED reservation under the reviewed no-CONT source
    // law, not proof of an actual child or a count-zero absence inference.
    let born_original = all([
        early_born.clone(),
        eq(BORN_LATCH, 1),
        eq(BORN_SOURCE, 1),
        eq(BORN_EXPECTED_SLOTS, 1).or(eq(BORN_EXPECTED_SLOTS, 2)),
        eq(BORN_CHILD_ACTIVATED, 0),
        eq(49, 0),
        eq(BORN_COST_SEEN, 1),
        eq(BORN_COST_RETAINED, 1),
        eq(BORN_BIRTH_CLOCKS, 1),
        eq(BORN_COST_PAID, 1),
        eq(BORN_ACCOUNTING_UNKNOWN, 1),
        Var(5).eq(Var(6)),
        Var(16).eq(Var(17)),
    ]);
    let born_reap_ready = all([
        born_original.clone(),
        eq(BORN_FRESH_SOURCE, 1),
        eq(BORN_PHYSICAL, 1),
        eq(INITIAL_ATTEMPT, 1),
        eq(BORN_ATTEMPT_RETAINED, 1),
        eq(BORN_ATTEMPT_END_CLOCKS, 1),
        eq(INITIAL_REST_PAID, 1),
        eq(INITIAL_COST_BOUND, 3_000_000),
        eq(INITIAL_ATTEMPT_DEBT, 27),
    ]);
    let unknown_conservation_route = all([
        eq(LAW, 21),
        initial_eligible.clone(),
        eq(INITIAL_READ_KNOWN, 0),
    ])
    .or(born_reap_ready.clone());
    let mut direct_exit = None;
    let mut implicit_exit = None;
    let mut unexpected_wait4 = None;
    let mut excessive_wait4 = None;
    let mut excessive_rollup_wait4 = None;
    let mut rollup_continue = None;
    let mut initial_exit = None;
    let mut within_grant_unknown = None;
    let mut early_birth = None;
    let mut early_attach = None;
    // All physical lifecycle actions are shared. Only the numeric charge
    // projection has separate alternatives; neither path can mint a new owner.
    for action in &mut m.actions {
        if action.name == "ConfirmCanonicalCreationOwnershipAndBothDebtsSettled" {
            action.guard = action.guard.clone().and(early_born.clone().negate());
        }
        if action.name == "AttachCurrentStoppedBirth" {
            let mut attached = action.clone();
            attached.name = "RetainExactBornStoppedPhysicalLatchBeforeAttachCallbackReturns".into();
            attached.guard = attached.guard.and(early_born.clone());
            attached.updates.push((BORN_LATCH, Var(BORN_SOURCE)));
            early_attach = Some(attached);
            action.guard = action.guard.clone().and(early_born.clone().negate());
        }
        if action.name == "OpenUnknownImplicitSlotWithoutInventingBirthTime" {
            // Profile3's unknown/unaccounted reserved child is observed DURING
            // the original admitted grant, after honest stopped startup. It
            // must not poison creation/stopped-stage admission beforehand.
            // Keep the same original pre-CONT slot start and no child ACK.
            let mut unknown = action.clone();
            unknown.name = "OpenWithinGrantUnknownImplicitSlotRetainsOriginalWindow".into();
            unknown.guard = unknown.guard.and(eq(LAW, 3));
            unknown.updates.push((42, Int(1)));
            within_grant_unknown = Some(unknown);
            action.guard = action.guard.clone().and(eq(LAW, 3).negate());
        }
        if action
            .name
            .starts_with("InitialCorroborationCreationSettled")
        {
            action.guard = action.guard.clone().and(eq(LAW, 0));
        }
        if action.name == "ObserveOrdinaryControllerTailBeforeInitialCorroboration" {
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 0).or(eq(INITIAL_STAGE, 0)));
        }
        // Correlated profiles select one observation trajectory. LAW0 alone
        // retains the complete independent legacy fault/foreground product;
        // duplicating it for every numeric constant proves no additional law.
        // Profile12 separately retains A1/B2/A3 while direct exit is pending.
        if action.name.starts_with("ObserveClockStopRevokeCancelPanic")
            || action.name.starts_with("ObserveForegroundFence")
            || action
                .name
                .starts_with("OrdinaryExpensiveObservationAdmissionFailed")
            || action.name == "ObserveDeadlineRetainsDebtAndOriginalEffect"
            || action.name == "RefuseMissingWholeCommandOrTransientCoverage"
        {
            action.guard = action.guard.clone().and(eq(LAW, 0));
        }
        if action.name.starts_with("RestartOriginalEffectA1B2A3From") {
            action.guard = action.guard.clone().and(eq(LAW, 0).or(all([
                eq(LAW, 12),
                eq(PENDING, 1),
                eq(0, 6),
                eq(40, 0),
            ])));
        }
        if action.name == "UnknownSlotAwaitsCompleteSameGrantExclusion" {
            action.guard = action.guard.clone().and(eq(LAW, 3).negate());
        }
        if action.name == "RetireRetainsOriginalWindowsAndSignalingCustody" {
            action.guard = action.guard.clone().and(
                eq(LAW, 0).or(all([
                    eq(27, 0),
                    eq(28, 0),
                    all([initial_eligible.clone(), eq(49, 0), eq(0, 3)])
                        .or(all([born_original.clone(), eq(0, 3)]))
                        .or(all([eq(LAW, 12).or(eq(LAW, 44)), eq(PENDING, 1)]))
                        .or(all([
                            Int(0).lt(Var(QNS)),
                            eq(46, 1),
                            eq(LAW, 1).or(eq(LAW, 15)).negate().or(eq(49, 2)),
                        ])),
                ])),
            );
        }
        if action.name == "ObserveExactOriginalExitAndRetainedUsage"
            || action.name == "ActuallyReapOriginalGroupAfterRetirement"
        {
            // This chooses when the controller observes an external outcome;
            // it does not claim the kernel waits for payment or is fair.
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 0).or(all([eq(27, 0), eq(28, 0)])));
        }
        if action.name == "LocalUnavailableDispositionRetainsOriginalEffect" {
            // A refusal can occur with fresh debt still pending. Record the
            // owned unavailable disposition immediately; it pays no debt,
            // releases no effect and grants no stopped/measurement authority.
            // Requiring retirement first disabled every fair local response
            // while its selected observation waited for that same debt.
            action.guard = action.guard.clone().and(eq(LAW, 0).or(eq(37, 0).negate()));
        }
        if action
            .name
            .starts_with("ChargeExactWindowActiveAndAllControllerCase")
            || action
                .name
                .starts_with("FailedClosureStillRecordsQuantumAndDebtCase")
        {
            action.guard = action.guard.clone().and(eq(LAW, 0));
        }
        if action.name == "ObserveExactOriginalExitAndRetainedUsage" {
            let mut observed = action.clone();
            observed.name = "ObserveUngrantOriginalExitWithIndependentRawKernelTruth".into();
            observed.guard = observed.guard.and(initial_only.clone()).and(
                early_born
                    .clone()
                    .negate()
                    .or(all([born_original.clone(), eq(BORN_FRESH_SOURCE, 1)])),
            );
            observed.updates.push((BORN_PHYSICAL, Int(1)));
            observed.updates.retain(|(slot, _)| *slot != 8);
            observed.updates.extend([
                (8, Int(0)),
                (BURNED, Var(BURNED).plus(Var(INITIAL_RAW_SYSTEM))),
            ]);
            initial_exit = Some(observed);
            action.guard = action.guard.clone().and(initial_only.clone().negate());
        }
        if action.name == "CreateStoppedRetainsOriginalEffect" {
            action
                .updates
                .extend([(BURNED, Int(125_000)), (CHARGED, Int(125_000))]);
            let mut born = action.clone();
            born.name = "CreateOriginalSuspendedEffectWithoutClaimingStartupCredit".into();
            born.guard = born.guard.and(early_born.clone());
            for (slot, value) in &mut born.updates {
                if *slot == CHARGED {
                    *value = Int(0);
                }
            }
            early_birth = Some(born);
            action.guard = action.guard.clone().and(early_born.clone().negate());
        }
        if action.name == "CheckedT0OpensEveryReservedSlotBeforeCont"
            || action.name == "ContinueOnlyAfterClosureCorroborationAndAllDebt"
        {
            action.updates.extend([
                (READ0, Int(0)),
                (READ1, Int(0)),
                (BASIS, Int(0)),
                (SEEN, Int(0)),
                (QNS, Int(0)),
                (REPORTED, Int(0)),
                (ORIGIN0, Int(0)),
                (ORIGIN1, Int(0)),
            ]);
            action.guard = action.guard.clone().and(eq(PENDING, 0));
            if action.name == "CheckedT0OpensEveryReservedSlotBeforeCont" {
                action.guard = action
                    .guard
                    .clone()
                    .and(initial_only.clone().negate())
                    .and(eq(LAW, 0).or(eq(INITIAL_READY, 1)));
            }
        }
        if action.name == "ContinueOnlyAfterClosureCorroborationAndAllDebt" {
            let mut second = action.clone();
            second.name = "ContinueQualifiedRollupSecondGrantRetainsPriorLiveCredit".into();
            second.guard = second.guard.and(eq(LAW, 15));
            second.updates.extend([
                (A0, Int(250_000)),
                (A1, Int(125_000)),
                (W0, Int(375_000)),
                (W1, Int(375_000)),
                (WALL, Int(750_000)),
                (ROLL_UNIQUE, Int(1)),
                (ROLL_MIXED, Int(0)),
                (ROLL_CUMULATIVE, Int(250_000)),
                (ROLL_BASELINE, Var(PRIOR_LIVE)),
                (ROLL_COUNT, Int(0)),
                (ROLL_EXIT, Int(0)),
                (ROLL_ACK, Int(0)),
                (ROLL_ZERO_CHILDREN, Int(0)),
                (ROLL_DELTA, Int(0)),
                (RAW_Q, Int(0)),
                (RAW_BASIS, Int(0)),
            ]);
            rollup_continue = Some(second);
            // Profile1 supplies the two-grant accumulation witness. Each
            // remaining correlated profile focuses one grant plus its selected
            // terminal/late/post observation, retaining the global two bound.
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 0).or(eq(LAW, 1)).or(Int(37).lt(Var(LAW))));
        }
        if action
            .name
            .starts_with("AckMember0AfterCurrentStopPredicatesCase")
        {
            action.guard = action.guard.clone().and(eq(LAW, 0).or(eq(SEEN, 1)));
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 4).negate().or(eq(READ0, 1)));
            if action.name.ends_with("Case2") {
                let mut exited = action.clone();
                exited.name = "ObserveDirectOriginalExitRetainsIdentityBeforeOwnWait4".into();
                let direct_profile = eq(LAW, 12).or(all([eq(LAW, 44), eq(49, 2)]));
                exited.guard = exited.guard.and(direct_profile.clone());
                for (slot, value) in &mut exited.updates {
                    if *slot == 7 {
                        *value = Int(4);
                    }
                }
                exited.updates.push((ORIGIN0, Int(2)));
                direct_exit = Some(exited);
                action.guard = action.guard.clone().and(direct_profile.negate());
            }
        }
        if action.name == "AckKnownMember1AfterCheckedHostClockCase1" {
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 17).negate().or(eq(READ1, 4)));
            let exited_profile = eq(LAW, 13)
                .or(all([eq(LAW, 15), eq(49, 2)]))
                .or(all([Int(15).lt(Var(LAW)), Var(LAW).lt(Int(20))]))
                .or(all([eq(LAW, 43), eq(49, 2)]));
            let mut exited = action.clone();
            exited.name = "ObserveImplicitChildExitWithoutInventingOwnWait4".into();
            exited.guard = exited.guard.and(exited_profile.clone());
            for (slot, value) in &mut exited.updates {
                if *slot == 8 {
                    *value = Int(4);
                }
            }
            exited.updates.push((ORIGIN1, Int(2)));
            implicit_exit = Some(exited);
            action.guard = action.guard.clone().and(exited_profile.negate());
        }
        if action.name == "ActuallyReapOriginalGroupAfterRetirement" {
            let attempted = initial_eligible
                .clone()
                .negate()
                .or(all([eq(INITIAL_ATTEMPT, 1), eq(INITIAL_REST_PAID, 1)]))
                .or(Bool(control == 39));
            action.guard = action
                .guard
                .clone()
                .and(attempted)
                .and(early_born.clone().negate().or(born_reap_ready.clone()));
            // N starts with actual original own wait4, never expected slots,
            // census or residual. The separately qualified child alternative
            // below may add exactly one original Git-reaped GPG once.
            action.updates.extend([
                (ACTUAL_REAPS, Int(1)),
                (TEARDOWN_ALLOWANCE, Var(TEARDOWN_LIMIT)),
            ]);
            let mut rollup_over = action.clone();
            rollup_over.name =
                "ActualWait4IndependentlyRevealsOverTwoCeilingsRollupTeardownCpu".into();
            rollup_over.guard = rollup_over.guard.and(eq(LAW, 19));
            rollup_over.updates.extend([
                (WAIT4, Var(BURNED).plus(Int(400_000))),
                (BURNED, Var(BURNED).plus(Int(400_000))),
            ]);
            excessive_rollup_wait4 = Some(rollup_over);
            let mut over = action.clone();
            over.name = "ActualWait4IndependentlyRevealsOverCeilingTeardownCpu".into();
            over.guard = over.guard.and(eq(LAW, 28));
            over.updates.extend([
                (WAIT4, Var(BURNED).plus(Int(250_000))),
                (BURNED, Var(BURNED).plus(Int(250_000))),
            ]);
            excessive_wait4 = Some(over);
            let mut independent = action.clone();
            independent.name = "ActualWait4IndependentlyRevealsUnchargedCpu".into();
            independent.guard = independent.guard.and(eq(LAW, 14).or(eq(LAW, 22)));
            independent.updates.extend([
                (WAIT4, Var(BURNED).plus(Int(125_000))),
                (BURNED, Var(BURNED).plus(Int(125_000))),
            ]);
            unexpected_wait4 = Some(independent);
            action.guard = action.guard.clone().and(
                eq(LAW, 14)
                    .or(eq(LAW, 19))
                    .or(eq(LAW, 22))
                    .or(eq(LAW, 28))
                    .negate(),
            );
            action.updates.push((WAIT4, Var(BURNED)));
        }
        if action
            .name
            .starts_with("OrdinaryExpensiveObservationAdmissionFailed")
        {
            action.guard = action.guard.clone().and(eq(PENDING, 0));
        }
        if action.name == "ReleaseOnlyAfterRealReapAndEveryKnownDebt" {
            action.guard = action.guard.clone().and(eq(LAW, 0).or(eq(CHECKED, 1)));
        }
        if action.name == "ResumeMainOnlyAfterFullSettlementAndPaidCoverage"
            || action.name == "TrustOnlyClosedQualifiedActuallySettledAccounting"
        {
            action.guard = action
                .guard
                .clone()
                .and(eq(LAW, 0).or(all([eq(CHECKED, 1), eq(TERMINAL, 1)])));
        }
        // Only actual member late-kernel CPU enters member conservation.
        // The separate post-corroboration/controller unit still pays its own
        // existing debt, but creates NO member CPU credit or raw member burn.
        if action
            .name
            .starts_with("ReconcileLateKernelOnceWithFreshAdditiveDebtExplained")
        {
            action.updates.extend([
                (BURNED, Var(BURNED).plus(Int(250_000))),
                (CHARGED, Var(CHARGED).plus(Int(250_000))),
            ]);
        }
    }
    m.actions.extend([
        direct_exit.expect("shared direct exit action"),
        implicit_exit.expect("shared implicit exit action"),
        unexpected_wait4.expect("shared actual wait4 action"),
        excessive_wait4.expect("shared over-ceiling actual wait4 action"),
        excessive_rollup_wait4.expect("shared over-two-ceilings rollup actual wait4 action"),
        rollup_continue.expect("shared second-grant continuation"),
        initial_exit.expect("shared initial original exit"),
        within_grant_unknown.expect("shared running-grant unknown-slot observation"),
        early_birth.expect("shared retained suspended original birth"),
        early_attach.expect("shared original stopped attachment"),
    ]);
    // Clarification2/11:45: at most ONE unique admitted child is counted
    // without a fabricated controller-owned child wait4. Original retired
    // identity/closed child-count-zero, leader ACK before qualified rollup,
    // exclusive nonduplicated child delta and actual original LEADER wait4
    // prove Git already reaped that child. No expected slot/unknown birth,
    // census alone, repeated grant/read or CPU residual can add N.
    let qualified_child_reap = all([
        eq(READ1, 4),
        eq(ORIGIN1, 2),
        eq(8, 4),
        eq(18, 1),
        eq(19, 1),
        eq(ROLL_UNIQUE, 1),
        eq(ROLL_MIXED, 0),
        eq(ROLL_EXIT, 1),
        eq(ROLL_ACK, 1),
        eq(ROLL_ZERO_CHILDREN, 1),
        eq(ROLL_COUNT, 1),
        eq(ROLL_DUPLICATE, 0),
        Var(ROLL_DELTA).eq(Var(ROLL_CUMULATIVE).minus(Var(ROLL_BASELINE))),
        Var(ROLL_BASELINE).eq(Var(PRIOR_LIVE)),
        Var(REPORTED).eq(Var(A0).plus(Var(ROLL_DELTA))),
        Var(5).eq(Var(6)),
    ]);
    let mut reaped_children = vec![];
    for action in &mut m.actions {
        if action.name == "ActuallyReapOriginalGroupAfterRetirement"
            || action.name.starts_with("ActualWait4IndependentlyReveals")
        {
            let mut child = action.clone();
            child.name.push_str("WithExactlyOneQualifiedGitReapedChild");
            child.guard = child.guard.and(qualified_child_reap.clone());
            child
                .updates
                .retain(|(slot, _)| ![ACTUAL_REAPS, TEARDOWN_ALLOWANCE].contains(slot));
            child.updates.extend([
                (ACTUAL_REAPS, Int(2)),
                (QUALIFIED_CHILD_REAP, Int(1)),
                (
                    TEARDOWN_ALLOWANCE,
                    Var(TEARDOWN_LIMIT).plus(Var(TEARDOWN_LIMIT)),
                ),
            ]);
            reaped_children.push(child);
            action.guard = action
                .guard
                .clone()
                .and(qualified_child_reap.clone().negate());
        }
    }
    m.actions.extend(reaped_children);
    // Representative five-slice refinement of already-proved STOPPED work.
    // The C implementation has at most11 checkpoints plus one separate kernel
    // accounting unit (12 total; its single-member control uses8+1). This is
    // NOT that maximum: these five correlated costs retain the observed total
    // 1_164_833ns while each nominal slice is <=500us and the hard gate stays1ms.
    // No open/running grant is split. Every new slice invalidates the seal and
    // records its OWN fresh9ms debt; first CONT waits for the final paid seal.
    act(
        m,
        "ObserveInitialOriginalStoppedSingleSlotAndClosedChildAbsence",
        all([
            initial_only.clone(),
            eq(0, 3),
            eq(52, 1),
            eq(7, 3),
            eq(8, 1),
            eq(STARTUP_CLOSED, 1),
            eq(49, 0),
            eq(27, 0),
            eq(28, 0),
            eq(37, 0),
            Var(5).eq(Var(6)),
        ]),
        &[
            (8, Int(0)),
            (14, Int(1)),
            (15, Int(1)),
            (18, Int(1)),
            (19, Int(1)),
        ],
    );
    let slices = [250_000, 250_000, 250_000, 250_000, 164_833];
    for (stage, cpu) in slices.into_iter().enumerate() {
        let stage = stage as i64;
        assert!(cpu <= 500_000, "stopped slice nominal must remain <=0.5ms");
        act(
            m,
            &format!("InitialStoppedCorroborationMeasuredSlice{stage}"),
            all([
                new_law.clone(),
                eq(0, 3),
                eq(52, 1),
                eq(53, 0),
                initial_only.clone().negate().or(all([
                    eq(8, 0),
                    eq(18, 1),
                    eq(STARTUP_KNOWN, 1),
                    eq(STARTUP_PAID, 1),
                    eq(STARTUP_TIMEBASE, 1),
                ])),
                eq(INITIAL_STAGE, stage),
                eq(7, 3),
                eq(49, 0),
                eq(37, 0),
                eq(38, 0),
                eq(42, 0),
                Var(5).eq(Var(6)),
                all([eq(27, 0), eq(28, 0)]).or(all([
                    Bool(control == 38 && stage == 1),
                    eq(27, 9),
                    eq(28, 9),
                ])),
            ]),
            &[
                (INITIAL_STAGE, Int(stage + 1)),
                (INITIAL_CPU, Var(INITIAL_CPU).plus(Int(cpu))),
                (INITIAL_LAST_CPU, Int(cpu)),
                (INITIAL_PRIOR_DEBT, Var(27)),
                (INITIAL_READY, Int(0)),
                (STARTUP_CHARGED, Int(1)),
                (27, Var(27).plus(Int(9))),
                (28, Var(28).plus(Int(9))),
            ],
        );
    }
    act(
        m,
        "SealInitialStoppedCorroborationOnlyAfterEverySlicePaid",
        all([
            new_law.clone(),
            eq(0, 3),
            eq(52, 1),
            eq(53, 0),
            eq(INITIAL_STAGE, 5),
            eq(27, 0),
            eq(28, 0),
            eq(7, 3),
            eq(49, 0),
            eq(37, 0),
            eq(38, 0),
            Var(5).eq(Var(6)),
        ]),
        &[(INITIAL_READY, Int(1)), (53, Int(1))],
    );
    for profile in 20..=28 {
        // One original stopped helper, no child, no grant and no CONT. Missing
        // baseline/roster/payment/timebase are retained independent premises;
        // they are never repaired from terminal wait4 or controller CPU.
        act(
            m,
            &format!("ConfigureUngrantStartupTerminalProfile{profile}"),
            all([eq(2, 0), eq(0, 0)]),
            &[
                (LAW, Int(profile)),
                (TEARDOWN_LIMIT, Int(150_000)),
                (1, Int(2)),
                (2, Int(1)),
                (STARTUP_KNOWN, Int(i64::from(profile != 23))),
                (STARTUP_CLOSED, Int(i64::from(profile != 24))),
                (STARTUP_PAID, Int(i64::from(profile != 25))),
                (STARTUP_TIMEBASE, Int(i64::from(profile != 26))),
                (
                    INITIAL_RAW_SYSTEM,
                    Int(if profile == 27 { 0 } else { 75_000 }),
                ),
            ],
        );
    }
    // The SAME created/attached owner is retained across an Attach callback
    // panic BEFORE creation settlement/first corroboration. Its fixed public
    // source forbids an implicit child before CONT; expected2 is kept consumed.
    for profile in 29..=37 {
        act(
            m,
            &format!("ConfigureEarlyBornStoppedCreationFaultProfile{profile}"),
            all([eq(2, 0), eq(0, 0)]),
            &[
                (LAW, Int(profile)),
                (TEARDOWN_LIMIT, Int(150_000)),
                (1, Int(2)),
                (2, Int(1)),
                (BORN_SOURCE, Int(i64::from(profile != 31))),
                (BORN_EXPECTED_SLOTS, Int(if profile == 30 { 1 } else { 2 })),
                (BORN_FRESH_SOURCE, Int(i64::from(profile != 33))),
                (BORN_ATTEMPT_END_CLOCKS, Int(i64::from(profile != 34))),
                (BORN_ACCOUNTING_UNKNOWN, Int(1)),
                (INITIAL_RAW_SYSTEM, Int(0)),
            ],
        );
    }
    act(
        m,
        "AttachCallbackFaultRetainsOriginalSuspendedEffectBeforeCreationSettlement",
        all([
            early_born.clone(),
            eq(0, 3),
            eq(52, 0),
            eq(37, 0),
            eq(BORN_COST_SEEN, 0),
        ]),
        &[(37, Int(5)), (26, Int(0)), (34, Int(0))],
    );
    for profile in 29..=37 {
        let native_known = !matches!(profile, 32 | 36 | 37);
        let receipt_clocks_known = profile != 36;
        let funded = !matches!(profile, 35 | 37);
        let bound = if native_known { 2_000_000 } else { 3_000_000 };
        let debt = if profile == 35 {
            0
        } else if profile == 37 {
            9
        } else if native_known {
            18
        } else {
            27
        };
        act(
            m,
            &format!("RetainActualNativeBirthCostDespiteAttachFaultProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(0, 3),
                eq(52, 0),
                eq(37, 5),
                eq(BORN_COST_SEEN, 0),
                eq(27, 0),
                eq(28, 0),
            ]),
            &[
                (BORN_COST_SEEN, Int(1)),
                (BORN_NATIVE_COST, Int(bound)),
                (BORN_COST_RETAINED, Int(i64::from(funded))),
                (BORN_BIRTH_CLOCKS, Int(i64::from(receipt_clocks_known))),
                (BORN_RAW_NATIVE_KNOWN, Int(i64::from(native_known))),
                (25, Int(i64::from(native_known))),
                (BORN_NATIVE_DEBT, Int(debt)),
                (27, Int(debt)),
                (28, Int(debt)),
            ],
        );
    }
    act(
        m,
        "RetainPaidOriginalNativeBirthCostWithoutCreatingStartupCredit",
        all([
            early_born.clone(),
            eq(0, 3),
            eq(52, 0),
            eq(BORN_LATCH, 1),
            eq(BORN_SOURCE, 1),
            eq(BORN_COST_SEEN, 1),
            eq(BORN_COST_RETAINED, 1),
            eq(BORN_BIRTH_CLOCKS, 1),
            eq(BORN_COST_PAID, 0),
            eq(27, 0),
            eq(28, 0),
            eq(49, 0),
            eq(BORN_CHILD_ACTIVATED, 0),
            Var(5).eq(Var(6)),
            Var(16).eq(Var(17)),
        ]),
        &[(BORN_COST_PAID, Int(1))],
    );
    for profile in [29, 30, 32, 34] {
        // One actual RI attempt is retained even when available; absent numeric
        // startup settlement still makes C+L UNKNOWN, not a recovered zero/L.
        act(
            m,
            &format!("AttemptEarlyBornTerminalRiWithUnknownStartupAccountingProfile{profile}"),
            all([
                eq(LAW, profile),
                born_original.clone(),
                eq(BORN_PHYSICAL, 1),
                eq(38, 1),
                eq(39, 1),
                eq(40, 0),
                eq(41, 1),
                eq(INITIAL_ATTEMPT, 0),
                eq(27, 0),
                eq(28, 0),
            ]),
            &[
                (INITIAL_ATTEMPT, Int(1)),
                (INITIAL_READ_KNOWN, Int(i64::from(profile != 30))),
                (BORN_ATTEMPT_RETAINED, Int(1)),
                (25, Int(0)),
                (INITIAL_L, Int(0)),
                (INITIAL_COST_BOUND, Int(3_000_000)),
                (INITIAL_ATTEMPT_DEBT, Int(27)),
                (27, Int(27)),
                (28, Int(27)),
            ],
        );
    }
    if control == 44 {
        act(
            m,
            "BrokenEarlyBornReapSkipsCurrentUnknownAttemptPayment",
            all([
                eq(LAW, 29),
                born_original.clone(),
                eq(BORN_PHYSICAL, 1),
                eq(38, 1),
                eq(39, 1),
                eq(40, 0),
                eq(41, 1),
                eq(INITIAL_ATTEMPT, 1),
                eq(BORN_ATTEMPT_RETAINED, 1),
                eq(BORN_ATTEMPT_END_CLOCKS, 1),
                eq(INITIAL_REST_PAID, 0),
                eq(27, 27),
                eq(28, 27),
            ]),
            &[
                (40, Int(1)),
                (0, Int(9)),
                (WAIT4, Var(BURNED)),
                (ACTUAL_REAPS, Int(1)),
                (TEARDOWN_ALLOWANCE, Var(TEARDOWN_LIMIT)),
            ],
        );
    }
    safe(
        m,
        "UnknownNativeAdmissionNeverBecomesKnownFromPhysicalReceipt",
        early_born
            .clone()
            .negate()
            .or(eq(BORN_COST_SEEN, 0))
            .or(eq(BORN_RAW_NATIVE_KNOWN, 1))
            .or(all([eq(25, 0), eq(BORN_NATIVE_COST, 3_000_000), eq(34, 0)])),
    );
    safe(
        m,
        "RetainedBornNativeCostFundsItsWholeInclusiveUnknownOrKnownLane",
        early_born
            .clone()
            .negate()
            .or(eq(BORN_COST_RETAINED, 0))
            .or(all([
                eq(BORN_RAW_NATIVE_KNOWN, 1),
                eq(BORN_NATIVE_COST, 2_000_000),
                eq(BORN_NATIVE_DEBT, 18),
            ]))
            .or(all([
                eq(BORN_RAW_NATIVE_KNOWN, 0),
                eq(BORN_NATIVE_COST, 3_000_000),
                eq(BORN_NATIVE_DEBT, 27),
            ])),
    );
    safe(
        m,
        "BornStoppedCreationFaultReapNeedsPaidOriginalPhysicalLatch",
        early_born
            .clone()
            .negate()
            .or(eq(40, 0))
            .or(born_reap_ready.clone()),
    );
    safe(
        m,
        "BornStoppedPhysicalOnlyNeverCreatesNumericStartupOrTrust",
        early_born.clone().negate().or(all([
            eq(52, 0),
            eq(INITIAL_READY, 0),
            eq(STARTUP_KNOWN, 0),
            eq(STARTUP_CHARGED, 0),
            eq(CHARGED, 0),
            eq(INITIAL_L, 0),
            eq(TERMINAL, 0),
            eq(26, 0),
            eq(34, 0),
            eq(49, 0),
            eq(BORN_CHILD_ACTIVATED, 0),
            eq(COMMITTED, 0),
        ])),
    );
    safe(
        m,
        "EarlyBornReapCountsOnlyActualOriginalNotUnactivatedReservation",
        early_born.clone().negate().or(eq(40, 0)).or(all([
            eq(ACTUAL_REAPS, 1),
            eq(QUALIFIED_CHILD_REAP, 0),
            eq(TEARDOWN_ALLOWANCE, 150_000),
        ])),
    );
    safe(
        m,
        "UnfundedOrUnprovenBornSourceRetainsOriginalCustody",
        early_born
            .clone()
            .negate()
            .or(eq(BORN_COST_SEEN, 0))
            .or(all([
                eq(BORN_LATCH, 1),
                eq(BORN_COST_RETAINED, 1),
                eq(BORN_BIRTH_CLOCKS, 1),
            ]))
            .or(all([eq(41, 1), eq(40, 0), eq(34, 0)])),
    );
    for profile in 23..=26 {
        act(
            m,
            &format!("RefuseUnprovenInitialStartupRetainsOwnedQuarantineProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(0, 3),
                eq(52, 1),
                eq(37, 0),
                eq(27, 0),
                eq(28, 0),
            ]),
            &[(37, Int(6)), (51, Int(1)), (26, Int(0)), (34, Int(0))],
        );
    }
    for profile in [20, 21, 22, 27, 28] {
        let known = profile != 21;
        let late = if profile == 27 { 0 } else { 75_000 };
        let attempt_debt = if known { 9 } else { 27 };
        let attempt_bound = if known { 250_000 + late } else { 3_000_000 };
        let mut attempt_updates = vec![
            (INITIAL_ATTEMPT, Int(1)),
            (INITIAL_READ_KNOWN, Int(i64::from(known))),
            (INITIAL_L, Int(if known { late } else { 0 })),
            (
                CHARGED,
                Var(CHARGED).plus(Int(if known { late } else { 0 })),
            ),
            (INITIAL_COST_BOUND, Int(attempt_bound)),
            (INITIAL_ATTEMPT_DEBT, Int(attempt_debt)),
            (27, Int(attempt_debt)),
            (28, Int(attempt_debt)),
        ];
        if !known {
            attempt_updates.push((37, Int(6)));
        }
        act(
            m,
            &format!("AttemptIndependentInitialTerminalRiBeforeOwnWait4Profile{profile}"),
            all([
                eq(LAW, profile),
                initial_eligible.clone(),
                eq(38, 1),
                eq(39, 1),
                eq(40, 0),
                eq(41, 1),
                eq(INITIAL_ATTEMPT, 0),
                eq(27, 0),
                eq(28, 0),
            ]),
            &attempt_updates,
        );
    }
    act(
        m,
        "RetainInitialTerminalObservationPaidReceiptBeforeOwnWait4",
        all([
            initial_eligible.clone().or(all([
                born_original.clone(),
                eq(BORN_PHYSICAL, 1),
                eq(BORN_ATTEMPT_RETAINED, 1),
                eq(BORN_ATTEMPT_END_CLOCKS, 1),
            ])),
            eq(38, 1),
            eq(39, 1),
            eq(40, 0),
            eq(INITIAL_ATTEMPT, 1),
            eq(INITIAL_REST_PAID, 0),
            eq(27, 0),
            eq(28, 0),
        ]),
        &[(INITIAL_REST_PAID, Int(1))],
    );
    if control == 39 {
        act(
            m,
            "BrokenInitialUnavailableAttemptReapsBeforeFreshUnknownDebtPaid",
            all([
                eq(LAW, 21),
                initial_eligible.clone(),
                eq(38, 1),
                eq(39, 1),
                eq(40, 0),
                eq(41, 1),
                eq(INITIAL_ATTEMPT, 1),
                eq(INITIAL_READ_KNOWN, 0),
                eq(INITIAL_REST_PAID, 0),
                eq(INITIAL_COST_BOUND, 3_000_000),
                eq(INITIAL_ATTEMPT_DEBT, 27),
                eq(27, 27),
                eq(28, 27),
            ]),
            &[
                (40, Int(1)),
                (0, Int(9)),
                (WAIT4, Var(BURNED)),
                (ACTUAL_REAPS, Int(1)),
                (TEARDOWN_ALLOWANCE, Var(TEARDOWN_LIMIT)),
            ],
        );
    }
    safe(
        m,
        "InitialUnavailableRiRetainsConservativeUnknownCost",
        eq(LAW, 21).negate().or(eq(INITIAL_ATTEMPT, 0)).or(all([
            eq(INITIAL_READ_KNOWN, 0),
            eq(INITIAL_L, 0),
            eq(INITIAL_COST_BOUND, 3_000_000),
            eq(INITIAL_ATTEMPT_DEBT, 27),
            eq(34, 0),
            eq(26, 0),
            eq(37, 6),
        ])),
    );
    safe(
        m,
        "InitialCorroborationStageRequiresFreshPaidRest",
        eq(LAW, 0).or(eq(INITIAL_PRIOR_DEBT, 0)),
    );
    safe(
        m,
        "InitialStoppedCorroborationUnitsRemainOrdinary",
        eq(LAW, 0).or(Var(INITIAL_LAST_CPU).lt(Int(1_000_001))),
    );
    safe(
        m,
        "InitialStoppedSlicesFinishBeforeFirstCont",
        eq(LAW, 0).or(eq(49, 0)).or(all([
            eq(INITIAL_STAGE, 5),
            eq(INITIAL_READY, 1),
            eq(INITIAL_CPU, 1_164_833),
        ])),
    );
    safe(
        m,
        "InitialEligibleTerminalRiAttemptPrecedesOwnWait4",
        initial_eligible
            .clone()
            .negate()
            .or(eq(40, 0))
            .or(all([eq(INITIAL_ATTEMPT, 1), eq(INITIAL_REST_PAID, 1)])),
    );
    safe(
        m,
        "InitialKernelCreditRequiresIndependentPaidOriginalBaseline",
        eq(INITIAL_L, 0).or(all([
            initial_eligible.clone(),
            eq(INITIAL_ATTEMPT, 1),
            eq(INITIAL_READ_KNOWN, 1),
            Var(INITIAL_L).eq(Var(INITIAL_RAW_SYSTEM)),
            eq(49, 0),
        ])),
    );
    safe(
        m,
        "UngrantStartupObservationNeverGrantsHelperOrOutputAuthority",
        initial_only.negate().or(all([
            eq(49, 0),
            eq(COMMITTED, 0),
            eq(QNS, 0),
            eq(BASIS, 0),
            eq(26, 0),
            eq(34, 0),
            eq(33, 0),
        ])),
    );
    // Legacy formula statements concern its unchanged normalized geometry.
    // New raw-ns arithmetic is checked below; all identity, ownership, debt,
    // failure, progress and actual-settlement properties remain shared.
    for invariant in &mut m.invariants {
        if invariant
            .name
            .starts_with("WindowUnionRetainsEveryReservedIntervalCase")
            || invariant
                .name
                .starts_with("ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase")
            || invariant
                .name
                .starts_with("AllActiveControllerAndPipeCpuCountExactlyOnceCase")
            || invariant
                .name
                .starts_with("QuantumIsMaxWindowOrActivePlusControllerCase")
        {
            invariant.condition = new_law.clone().or(invariant.condition.clone());
        }
    }
    // profile: A0,A1,W0,W1,reported pre-ACK, case. Every value is immutable
    // input geometry. The second-thread profile passes aggregate A<=sum(W)
    // while failing A0<=W0, so an aggregate check cannot discharge coverage.
    let profiles = [
        (1, 250_000, 0, 10_000_000, 10_000_000, 250_000, 1),
        (2, 500_000, 250_000, 12_500_000, 12_500_000, 750_000, 1),
        (3, 250_000, 0, 250_000, 250_000, 250_000, 2),
        (4, 375_000, 125_000, 375_000, 375_000, 250_000, 1),
        (5, 250_000, 2_000_000, 500_000, 2_500_000, 2_250_000, 1),
        (6, 500_000, 0, 500_000, 250_000, 500_000, 1),
        (7, 250_000, 0, 15_000_000, 15_000_000, 250_000, 1),
        (8, 500_000, 0, 100_000, 650_000, 500_000, 1),
        (9, 250_000, 0, 250_000, 250_000, 250_000, 1),
        (10, 250_000, 0, 250_000, 250_000, 250_000, 1),
        (11, 250_000, 0, 375_000, 375_000, 250_000, 7),
        (12, 250_000, 0, 250_000, 500_000, 250_000, 2),
        (13, 250_000, 125_000, 375_000, 375_000, 375_000, 1),
        (14, 250_000, 0, 375_000, 375_000, 250_000, 1),
        (15, 125_000, 125_000, 250_000, 250_000, 250_000, 1),
        (16, 250_000, 250_000, 375_000, 375_000, 500_000, 1),
        (17, 250_000, 125_000, 375_000, 375_000, 375_000, 1),
        (18, 250_000, 125_000, 375_000, 375_000, 375_000, 1),
        (19, 250_000, 125_000, 375_000, 375_000, 375_000, 1),
    ];
    for (profile, a0, a1, w0, w1, pre_ack, case) in profiles {
        let wall = w0 + w1;
        let actual = a0 + a1;
        act(
            m,
            &format!("ConfigureMeasuredLawCorrelatedProfile{profile}"),
            all([eq(2, 0), eq(0, 0)]),
            &[
                (LAW, Int(profile)),
                (TEARDOWN_LIMIT, Int(150_000)),
                (1, Int(case)),
                (2, Int(1)),
                (42, Int(0)),
                (A0, Int(a0)),
                (A1, Int(a1)),
                (W0, Int(w0)),
                (W1, Int(w1)),
                (WALL, Int(wall)),
                (MONOTONIC, Int(if profile == 10 { 0 } else { 3 })),
                (CONVERTED, Int(if profile == 9 { 0 } else { 3 })),
            ],
        );
        act(
            m,
            &format!("ObserveActualMemberCpuBeforePositiveAckProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(0, 4),
                eq(SEEN, 0),
                eq(14, 0),
                eq(8, if matches!(profile, 3 | 12) { 5 } else { 2 }),
                if profile == 15 { eq(49, 1) } else { Bool(true) },
            ]),
            &[(BURNED, Var(BURNED).plus(Int(actual))), (SEEN, Int(1))],
        );
        if profile >= 16 {
            continue;
        }
        if profile == 4 {
            act(
                m,
                "ReadCountersBeforePositiveAckRetainsItsProvenance",
                all([
                    eq(LAW, profile),
                    eq(0, 4),
                    eq(SEEN, 1),
                    eq(14, 0),
                    eq(READ0, 0),
                ]),
                &[(READ0, Int(1)), (READ1, Int(1)), (REPORTED, Int(pre_ack))],
            );
        } else if profile == 13 {
            act(
                m,
                "ReadStoppedLeaderNeverLabelsExitedImplicitChildPostAck",
                all([
                    eq(LAW, profile),
                    eq(0, 6),
                    eq(14, 1),
                    eq(15, 1),
                    eq(7, 3),
                    eq(8, 4),
                    eq(18, 1),
                    eq(READ0, 0),
                ]),
                &[
                    (READ0, Int(2)),
                    (ORIGIN0, Int(1)),
                    (READ1, Int(0)),
                    (ORIGIN1, Int(2)),
                    (REPORTED, Int(actual)),
                ],
            );
        } else if !matches!(profile, 3 | 9 | 12) {
            act(
                m,
                &format!("ReadBothCountersOnlyAfterPositiveAckAndRosterProfile{profile}"),
                all([
                    eq(LAW, profile),
                    eq(0, 6),
                    eq(14, 1),
                    eq(15, 1),
                    eq(18, 1),
                    eq(19, 1),
                    eq(7, 3),
                    eq(8, 3),
                    eq(READ0, 0),
                    if profile == 15 { eq(49, 1) } else { Bool(true) },
                ]),
                &[
                    (READ0, Int(2)),
                    (READ1, Int(2)),
                    (REPORTED, Int(actual)),
                    (ORIGIN0, Int(1)),
                    (ORIGIN1, Int(1)),
                ],
            );
        }
        if profile == 12 {
            // Direct-exit RI/identity proof closes physical custody, never the
            // missing actual wait4 counter source. Reap needs no grant charge.
            act(
                m,
                "DeferDirectExitGrantAccountingUntilOriginalOwnWait4",
                all([
                    eq(LAW, profile),
                    eq(0, 6),
                    eq(7, 4),
                    eq(8, 0),
                    eq(14, 1),
                    eq(15, 1),
                    eq(18, 1),
                    eq(20, 1),
                    eq(READ0, 0),
                    eq(PENDING, 0),
                    eq(37, 0),
                ]),
                &[(PENDING, Int(1)), (ORIGIN0, Int(2)), (ORIGIN1, Int(3))],
            );
            continue;
        }
        let omit_rollup = control == 31 && profile == 5;
        let raw_ticks = control == 32 && profile == 6;
        let reported = if omit_rollup {
            a0
        } else if raw_ticks {
            12_000
        } else if profile == 4 {
            pre_ack
        } else {
            actual
        };
        let eligible = !matches!(profile, 3 | 4 | 8 | 9 | 10 | 13);
        let forged = (control == 29 && profile == 3)
            || (control == 30 && profile == 4)
            || (control == 34 && profile == 8);
        let measured = eligible || forged;
        let member = if measured {
            reported
        } else {
            wall.max(reported)
        };
        let q = member + 250_000;
        let debt = (9 * q.max(1_000_000) + 999_999) / 1_000_000;
        assert!(q <= 3_000_000 && debt <= 27);
        let wall_ok = wall <= 25_000_000 || (control == 33 && profile == 7);
        let coverage = (eligible || forged || profile == 13) && q <= 1_000_000 && wall_ok;
        let mut guard = vec![
            eq(LAW, profile),
            eq(37, 0),
            eq(SEEN, 1),
            Var(5).eq(Var(6)),
            Var(48).lt(Int(28 - debt)),
            if profile == 15 { eq(49, 1) } else { Bool(true) },
        ];
        if profile == 3 {
            // The SAME reserved slot remains unknown and unclosed. Its true
            // pre-CONT window is charged; no positive ACK or seal is invented.
            guard.extend([eq(0, 4), eq(14, 1), eq(15, 0), eq(8, 5)]);
        } else {
            guard.extend([
                eq(0, 6),
                eq(20, 1),
                eq(
                    READ0,
                    if profile == 4 {
                        1
                    } else if profile == 9 {
                        0
                    } else {
                        2
                    },
                ),
            ]);
        }
        act(
            m,
            &format!("ChargeBasisDependentQuantumWithRetainedRawProfile{profile}"),
            all(guard),
            &[
                (0, Int(7)),
                (33, Int(0)),
                (BASIS, Int(i64::from(measured))),
                (BASIS_EPOCH, Var(5)),
                (REPORTED, Int(reported)),
                (QNS, Int(q)),
                (CHARGED, Var(CHARGED).plus(Int(member))),
                (CREATED_DEBT, Int(debt)),
                (COMMITTED, Var(COMMITTED).plus(Int(1))),
                (RAW_Q, Int(q)),
                (RAW_BASIS, Int(i64::from(measured))),
                (PRIOR_LIVE, Int(if profile == 15 { 125_000 } else { 0 })),
                (21, Int(2)),
                (22, Int(1)),
                (23, Int(1)),
                (24, Int(3)),
                (26, Int(i64::from(coverage))),
                (37, Int(if coverage { 0 } else { 6 })),
                (27, Var(27).plus(Int(debt))),
                (28, Var(28).plus(Int(debt))),
                (48, Var(48).plus(Int(debt))),
                (43, Int(1)),
                (44, Int(1)),
                (46, Int(i64::from(case != 7))),
            ],
        );
    }
    // Clarified66236: a qualified unique child can be represented by the
    // stopped leader's POST-exit rollup. This is a distinct source from the
    // controller's own wait4, and remains provisional until that final wait4.
    act(
        m,
        "ObserveQualifiedRollupSecondGrantActualCpu",
        all([
            eq(LAW, 15),
            eq(49, 2),
            eq(0, 4),
            eq(SEEN, 0),
            eq(14, 0),
            eq(8, 2),
        ]),
        &[(BURNED, Var(BURNED).plus(Int(375_000))), (SEEN, Int(1))],
    );
    act(
        m,
        "ReadLeaderRollupBeforeProvenChildExitRetainsBadOrdering",
        all([
            eq(LAW, 17),
            eq(0, 4),
            eq(SEEN, 1),
            eq(14, 1),
            eq(7, 3),
            eq(8, 2),
            eq(READ1, 0),
        ]),
        &[
            (READ0, Int(2)),
            (ORIGIN0, Int(1)),
            (READ1, Int(4)),
            (ROLL_UNIQUE, Int(1)),
            (ROLL_MIXED, Int(0)),
            (ROLL_EXIT, Int(0)),
            (ROLL_ACK, Int(1)),
            (ROLL_COUNT, Int(1)),
            (ROLL_ZERO_CHILDREN, Int(0)),
            (ROLL_CUMULATIVE, Int(0)),
            (ROLL_BASELINE, Int(0)),
            (ROLL_DELTA, Int(0)),
            (ROLL_DUPLICATE, Int(0)),
            (REPORTED, Int(250_000)),
        ],
    );
    for profile in [15, 16, 18, 19] {
        let cumulative = if matches!(profile, 15 | 16) {
            250_000
        } else {
            125_000
        };
        let baseline = if profile == 15 { 125_000 } else { 0 };
        let duplicate = if profile == 18 { 125_000 } else { 0 };
        let delta = cumulative - baseline;
        act(
            m,
            &format!("ReadClosedChildRollupAfterLeaderAckProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(49, if profile == 15 { 2 } else { 1 }),
                eq(0, 6),
                eq(14, 1),
                eq(15, 1),
                eq(18, 1),
                eq(19, 1),
                eq(7, 3),
                eq(8, 4),
                eq(READ0, 0),
                eq(READ1, 0),
            ]),
            &[
                (READ0, Int(2)),
                (ORIGIN0, Int(1)),
                (READ1, Int(4)),
                (ORIGIN1, Int(2)),
                (ROLL_UNIQUE, Int(i64::from(profile != 16))),
                (ROLL_MIXED, Int(i64::from(profile == 16))),
                (ROLL_EXIT, Int(1)),
                (ROLL_ACK, Int(1)),
                (ROLL_ZERO_CHILDREN, Int(1)),
                (ROLL_COUNT, Int(if profile == 18 { 2 } else { 1 })),
                (ROLL_CUMULATIVE, Int(cumulative)),
                (ROLL_BASELINE, Int(baseline)),
                (ROLL_DELTA, Int(delta)),
                (ROLL_DUPLICATE, Int(duplicate)),
                (REPORTED, Int(250_000 + delta + duplicate)),
            ],
        );
    }
    for profile in 15..=19 {
        let reported = match profile {
            16 | 18 => 500_000,
            17 => 250_000,
            _ => 375_000,
        };
        let qualified = matches!(profile, 15 | 19);
        let forged = (control == 35 && profile == 16)
            || (control == 36 && profile == 17)
            || (control == 37 && profile == 18);
        let measured = qualified || forged;
        let member = if measured { reported } else { 750_000 };
        let q = member + 250_000;
        act(
            m,
            &format!("ChargeDistinctQualifiedLeaderRollupSourceProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(49, if profile == 15 { 2 } else { 1 }),
                eq(0, 6),
                eq(18, 1),
                eq(19, 1),
                eq(20, 1),
                eq(14, 1),
                eq(15, 1),
                eq(READ0, 2),
                eq(READ1, 4),
                eq(37, 0),
                Var(5).eq(Var(6)),
                eq(REPORTED, reported),
                eq(27, 0),
                eq(28, 0),
                Var(48).lt(Int(19)),
            ]),
            &[
                (0, Int(7)),
                (33, Int(0)),
                (BASIS, Int(i64::from(measured))),
                (BASIS_EPOCH, Var(5)),
                (QNS, Int(q)),
                (CREATED_DEBT, Int(9)),
                (RAW_Q, Int(q)),
                (RAW_BASIS, Int(i64::from(measured))),
                (CHARGED, Var(CHARGED).plus(Int(member))),
                (COMMITTED, Var(COMMITTED).plus(Int(1))),
                (ROLL_PENDING, Int(i64::from(measured))),
                (ROLL_VALIDATED, Int(0)),
                (26, Int(i64::from(measured))),
                (37, Int(if measured { 0 } else { 6 })),
                (27, Int(9)),
                (28, Int(9)),
                (48, Var(48).plus(Int(9))),
                (21, Int(2)),
                (22, Int(1)),
                (23, Int(1)),
                (24, Int(3)),
                (43, Int(1)),
                (44, Int(1)),
                (46, Int(1)),
            ],
        );
    }
    // Reaping stays the original physical action. Certification is a later
    // local outcome; a detected shortfall is a valid refused execution whose
    // raw totals/excess survive. Neither RI exit nor a failed STOP is wait4.
    let bind_direct = m.actions.len();
    act(
        m,
        "BindDirectOwnWait4CounterBeforeConservationClassification",
        all([
            eq(LAW, 12),
            eq(PENDING, 1),
            eq(40, 1),
            eq(READ0, 0),
            eq(ORIGIN0, 2),
            eq(ORIGIN1, 3),
            eq(18, 1),
            eq(14, 1),
            eq(15, 1),
        ]),
        &[(READ0, Int(3)), (REPORTED, Var(WAIT4).minus(Int(125_000)))],
    );
    let commit_direct = m.actions.len();
    act(
        m,
        "CommitDeferredDirectGrantOnlyFromOriginalOwnWait4",
        all([
            eq(LAW, 12),
            eq(PENDING, 1),
            eq(40, 1),
            eq(READ0, 3),
            eq(ORIGIN0, 2),
            eq(ORIGIN1, 3),
            eq(READ1, 0),
            eq(18, 1),
            eq(19, 1),
            eq(20, 1),
            eq(14, 1),
            eq(15, 1),
            eq(42, 0),
            Var(5).eq(Var(6)),
            eq(37, 0),
            eq(27, 0),
            eq(28, 0),
            eq(QNS, 0),
            eq(REPORTED, 250_000),
        ]),
        &[
            (PENDING, Int(0)),
            (BASIS, Int(1)),
            (BASIS_EPOCH, Var(5)),
            (QNS, Int(500_000)),
            (RAW_Q, Int(500_000)),
            (RAW_BASIS, Int(1)),
            (CREATED_DEBT, Int(9)),
            (COMMITTED, Var(COMMITTED).plus(Int(1))),
            (CHARGED, Var(CHARGED).plus(Var(REPORTED))),
            (27, Int(9)),
            (28, Int(9)),
            (48, Var(48).plus(Int(9))),
            (26, Int(1)),
            (46, Int(1)),
        ],
    );
    let refuse_direct = m.actions.len();
    act(
        m,
        "FailedDeferredDirectGrantRetainsWindowChargeForSettlement",
        all([
            eq(LAW, 12),
            eq(PENDING, 1),
            eq(40, 1),
            eq(37, 0).negate(),
            eq(27, 0),
            eq(28, 0),
            eq(QNS, 0),
        ]),
        &[
            (PENDING, Int(0)),
            (BASIS, Int(0)),
            (QNS, Int(1_000_000)),
            (RAW_Q, Int(1_000_000)),
            (RAW_BASIS, Int(0)),
            (COMMITTED, Var(COMMITTED).plus(Int(1))),
            (CREATED_DEBT, Int(9)),
            (CHARGED, Var(CHARGED).plus(Int(750_000))),
            (27, Int(9)),
            (28, Int(9)),
            (48, Var(48).plus(Int(9))),
            (26, Int(0)),
            (34, Int(0)),
        ],
    );
    if control == 30 {
        act(
            m,
            "BrokenDirectExitRiCounterPretendsToBeStoppedPostAck",
            all([eq(LAW, 12), eq(PENDING, 1), eq(40, 0), eq(READ0, 0)]),
            &[(BASIS, Int(1)), (BASIS_EPOCH, Var(5)), (READ0, Int(2))],
        );
    }
    let rhs = Var(CHARGED).plus(Var(TEARDOWN_ALLOWANCE));
    let conserved = rhs.clone().lt(Var(WAIT4)).negate();
    let residual_ready = all([
        new_law.clone(),
        eq(40, 1),
        eq(PENDING, 0),
        eq(RESIDUAL_OBSERVED, 0),
        eq(27, 0),
        eq(28, 0),
        eq(LAW, 21).negate(),
        early_born.clone().negate(),
    ]);
    let bill_start = m.actions.len();
    // Once actual reap is already retained, the remaining response is ONLY
    // local bill/payment/classification bookkeeping. The new terminal bill
    // creates fresh debt after the response request. Keep its exact existing
    // rest transitions weakly fair in this post-reap phase as well, rather
    // than silently allowing stutter with a continuously enabled payment.
    // These are copies of the original paid-work-end transition, with the
    // SAME debt decrement and additional phase guards. No new state/update,
    // saved-idle credit, clock success, stop/reap or OS fairness is assumed.
    // LAW0 never enables them, preserving its entire old labelled graph and
    // fairness semantics; existing global rest actions remain unchanged.
    let terminal_payments = m
        .actions
        .iter()
        .filter(|action| {
            action
                .name
                .starts_with("ObserveRestAfterWorkEndWithoutIdleCredit")
        })
        .map(|action| {
            let mut payment = action.clone();
            payment.name = action.name.replacen(
                "ObserveRestAfterWorkEndWithoutIdleCredit",
                "PayAlreadyReapedTerminalResidualDebtWithoutIdleCredit",
                1,
            );
            payment.guard = payment.guard.and(all([
                new_law.clone(),
                eq(40, 1),
                eq(CHECKED, 0),
                eq(RESIDUAL_OBSERVED, 1),
            ]));
            payment
        })
        .collect::<Vec<_>>();
    assert_eq!(
        terminal_payments.len(),
        27,
        "all existing bounded debt payments must remain represented"
    );
    m.actions.extend(terminal_payments);
    act(
        m,
        "RetainAndChargeKnownZeroResidualOrdinaryWait4Cost",
        all([residual_ready.clone(), Var(CHARGED).lt(Var(WAIT4)).negate()]),
        &[
            (RESIDUAL, Int(0)),
            (RESIDUAL_CHARGED, Int(0)),
            (RESIDUAL_TOTAL, Int(250_000)),
            (RESIDUAL_OBSERVED, Int(1)),
            (RESIDUAL_KNOWN, Int(1)),
            (INDEPENDENT_AT_BILL, Var(CHARGED)),
            (TERMINAL_DEBT, Int(9)),
            (27, Int(9)),
            (28, Int(9)),
        ],
    );
    for debt in [9, 21] {
        let residual = Var(WAIT4).minus(Var(CHARGED));
        let mut charged = vec![
            (RESIDUAL, residual.clone()),
            (RESIDUAL_CHARGED, residual.clone()),
            (RESIDUAL_TOTAL, residual.clone().plus(Int(250_000))),
            (RESIDUAL_OBSERVED, Int(1)),
            (RESIDUAL_KNOWN, Int(1)),
            (INDEPENDENT_AT_BILL, Var(CHARGED)),
            (TERMINAL_DEBT, Int(debt)),
            (27, Int(debt)),
            (28, Int(debt)),
        ];
        if debt == 21 {
            charged.extend([(37, Int(6)), (26, Int(0)), (34, Int(0))]);
        }
        act(
            m,
            &format!("RetainAndChargeFullRawWait4ResidualWithFreshDebt{debt}"),
            all([
                residual_ready.clone(),
                Var(CHARGED).lt(Var(WAIT4)),
                if debt == 9 {
                    residual.clone().lt(Int(750_001))
                } else {
                    residual.clone().lt(Int(750_001)).negate()
                },
                if control == 41 {
                    eq(LAW, 14).negate()
                } else {
                    Bool(true)
                },
            ]),
            &charged,
        );
    }
    if control == 41 {
        act(
            m,
            "BrokenTeardownAllowanceSkipsRealResidualCost",
            all([
                residual_ready.clone(),
                eq(LAW, 14),
                Var(CHARGED).lt(Var(WAIT4)),
            ]),
            &[
                (RESIDUAL, Var(WAIT4).minus(Var(CHARGED))),
                (RESIDUAL_CHARGED, Int(0)),
                (RESIDUAL_TOTAL, Int(250_000)),
                (RESIDUAL_OBSERVED, Int(1)),
                (RESIDUAL_KNOWN, Int(1)),
                (INDEPENDENT_AT_BILL, Var(CHARGED)),
                (TERMINAL_DEBT, Int(9)),
                (27, Int(9)),
                (28, Int(9)),
            ],
        );
    }
    act(
        m,
        "RetainUnknownTerminalWait4CostInConservativeThreeMsLane",
        all([
            unknown_conservation_route.clone(),
            eq(40, 1),
            eq(PENDING, 0),
            eq(INITIAL_ATTEMPT, 1),
            eq(INITIAL_READ_KNOWN, 0).or(early_born.clone()),
            eq(INITIAL_REST_PAID, 1),
            eq(RESIDUAL_OBSERVED, 0),
            eq(27, 0),
            eq(28, 0),
        ]),
        &[
            (RESIDUAL, Int(0)),
            (RESIDUAL_CHARGED, Int(0)),
            (RESIDUAL_KNOWN, Int(0)),
            (RESIDUAL_TOTAL, Int(3_000_000)),
            (RESIDUAL_OBSERVED, Int(1)),
            (INDEPENDENT_AT_BILL, Var(CHARGED)),
            (INITIAL_UNKNOWN_DEBT, Int(27)),
            (27, Int(27)),
            (28, Int(27)),
        ],
    );
    act(
        m,
        "RetainFullTerminalResidualPaymentBeforeConservation",
        all([
            new_law.clone(),
            eq(40, 1),
            eq(PENDING, 0),
            eq(RESIDUAL_OBSERVED, 1),
            eq(RESIDUAL_PAID, 0),
            eq(27, 0),
            eq(28, 0),
        ]),
        &[(RESIDUAL_PAID, Int(1))],
    );
    let bill_paid = all([eq(RESIDUAL_OBSERVED, 1), eq(RESIDUAL_PAID, 1)]);
    act(
        m,
        "CertifyConservationOnlyAfterActualRetainedWait4",
        all([
            new_law.clone(),
            eq(40, 1),
            eq(CHECKED, 0),
            eq(PENDING, 0),
            eq(27, 0),
            eq(28, 0),
            conserved.clone(),
            early_born.clone().negate(),
            bill_paid.clone(),
            initial_eligible
                .clone()
                .negate()
                .or(all([eq(INITIAL_READ_KNOWN, 1), eq(INITIAL_REST_PAID, 1)])),
        ]),
        &[
            (CHECKED, Int(1)),
            (TERMINAL, Int(1)),
            (ROLL_VALIDATED, Var(ROLL_PENDING)),
            (ROLL_PENDING, Int(0)),
        ],
    );
    for profile in 1..=28 {
        let mut refused = vec![
            (CHECKED, Int(1)),
            (TERMINAL, Int(2)),
            (EXCESS, Var(WAIT4).minus(rhs.clone())),
            (37, Int(6)),
            (26, Int(0)),
            (34, Int(0)),
            (ROLL_PENDING, Int(0)),
            (ROLL_VALIDATED, Int(0)),
        ];
        if matches!(profile, 15 | 19) {
            // Effective accounting is withdrawn; immutable raw measured Q,
            // member credit and the actual shortfall are NEVER rewritten to
            // make conservation pass. The fallback bill is retained separately.
            refused.extend([
                (BASIS, Int(0)),
                (QNS, Int(1_000_000)),
                (ROLL_REVOKED, Int(1)),
                (FALLBACK_Q, Int(1_000_000)),
                (FALLBACK_DEBT, Int(9)),
                (27, Int(9)),
                (28, Int(9)),
            ]);
        }
        act(
            m,
            &format!("RefuseConservationExcessRetainsActualRawUsageProfile{profile}"),
            all([
                eq(LAW, profile),
                eq(40, 1),
                eq(CHECKED, 0),
                eq(PENDING, 0),
                eq(27, 0),
                eq(28, 0),
                conserved.clone().negate(),
                bill_paid.clone(),
                Bool(profile != 21),
            ]),
            &refused,
        );
    }
    act(
        m,
        "RefuseUnknownInitialConservationAfterPaidAttemptAndOwnWait4",
        all([
            unknown_conservation_route.clone(),
            eq(INITIAL_ATTEMPT, 1),
            eq(INITIAL_READ_KNOWN, 0).or(early_born.clone()),
            eq(INITIAL_REST_PAID, 1),
            eq(INITIAL_COST_BOUND, 3_000_000),
            eq(INITIAL_ATTEMPT_DEBT, 27),
            eq(40, 1),
            eq(CHECKED, 0),
            eq(27, 0),
            eq(28, 0),
            bill_paid.clone(),
        ]),
        &[
            (CHECKED, Int(1)),
            (TERMINAL, Int(0)),
            (INITIAL_UNKNOWN, Int(1)),
            (INITIAL_UNKNOWN_DEBT, Int(27)),
            (EXCESS, Int(0)),
            (37, Int(6)),
            (26, Int(0)),
            (34, Int(0)),
        ],
    );
    let refuse_end = m.actions.len();
    if forged_terminal_control && matches!(control, 31 | 32) {
        // COMPOSED fault: omit/misconvert a contribution AND independently
        // bypass terminal certification. Pure omission retains the hard gate
        // and must refuse, as the separate nonvacuity checks below require.
        act(
            m,
            "BrokenOmittedCpuContributionForgesTerminalAccounting",
            all([
                eq(LAW, if control == 31 { 5 } else { 6 }),
                eq(40, 1),
                eq(CHECKED, 0),
                eq(BASIS, 1),
                Int(0).lt(Var(QNS)),
                eq(27, 0),
                eq(28, 0),
                eq(REPORTED, if control == 31 { 250_000 } else { 12_000 }),
                conserved.clone().negate(),
                bill_paid.clone(),
            ]),
            &[(CHECKED, Int(1)), (TERMINAL, Int(1))],
        );
    }
    if control == 40 {
        act(
            m,
            "BrokenAcceptsActualPostSampleBurnAboveFixedTeardownCeiling",
            all([
                eq(LAW, 28),
                eq(40, 1),
                eq(CHECKED, 0),
                bill_paid.clone(),
                eq(27, 0),
                eq(28, 0),
                conserved.clone().negate(),
            ]),
            &[(CHECKED, Int(1)), (TERMINAL, Int(1))],
        );
    }
    if control == 42 {
        act(
            m,
            "BrokenDerivesTeardownCeilingFromTheSameWait4Residual",
            all([
                eq(LAW, 28),
                eq(40, 1),
                eq(CHECKED, 0),
                bill_paid.clone(),
                eq(27, 0),
                eq(28, 0),
                eq(TEARDOWN_LIMIT, 150_000),
                Var(TEARDOWN_ALLOWANCE).lt(Var(RESIDUAL)),
            ]),
            &[
                (TEARDOWN_LIMIT, Var(RESIDUAL)),
                (TEARDOWN_ALLOWANCE, Var(RESIDUAL)),
            ],
        );
    }
    if control == 43 {
        act(
            m,
            "BrokenUnprovenExpectedChildAddsAnotherTeardownAllowance",
            all([
                eq(LAW, 28),
                eq(40, 1),
                eq(CHECKED, 0),
                bill_paid.clone(),
                eq(27, 0),
                eq(28, 0),
                eq(ACTUAL_REAPS, 1),
                eq(QUALIFIED_CHILD_REAP, 0),
            ]),
            &[
                (ACTUAL_REAPS, Int(2)),
                (
                    TEARDOWN_ALLOWANCE,
                    Var(TEARDOWN_LIMIT).plus(Var(TEARDOWN_LIMIT)),
                ),
            ],
        );
    }
    safe(
        m,
        "TeardownCeilingIsIndependentFixedPerActualReap",
        eq(LAW, 0).or(all([
            eq(TEARDOWN_LIMIT, 150_000),
            all([eq(ACTUAL_REAPS, 0), eq(TEARDOWN_ALLOWANCE, 0)])
                .or(all([
                    eq(ACTUAL_REAPS, 1),
                    Var(TEARDOWN_ALLOWANCE).eq(Var(TEARDOWN_LIMIT)),
                ]))
                .or(all([
                    eq(ACTUAL_REAPS, 2),
                    Var(TEARDOWN_ALLOWANCE).eq(Var(TEARDOWN_LIMIT).plus(Var(TEARDOWN_LIMIT))),
                ])),
        ])),
    );
    safe(
        m,
        "TeardownCountRequiresOriginalActualOwnWait4",
        eq(LAW, 0).or(all([
            eq(ACTUAL_REAPS, 0).or(eq(40, 1)),
            eq(40, 0).or(eq(ACTUAL_REAPS, 1).or(eq(ACTUAL_REAPS, 2))),
        ])),
    );
    safe(
        m,
        "TeardownChildCountNeedsQualifiedRetainedReapProof",
        eq(LAW, 0).or(all([
            eq(ACTUAL_REAPS, 2).negate().or(all([
                eq(40, 1),
                eq(QUALIFIED_CHILD_REAP, 1),
                qualified_child_reap.clone(),
            ])),
            eq(QUALIFIED_CHILD_REAP, 0).or(all([eq(40, 1), eq(ACTUAL_REAPS, 2)])),
        ])),
    );
    let raw_residual = all([
        Var(CHARGED).lt(Var(WAIT4)),
        Var(RESIDUAL).eq(Var(WAIT4).minus(Var(CHARGED))),
    ])
    .or(all([Var(CHARGED).lt(Var(WAIT4)).negate(), eq(RESIDUAL, 0)]));
    let known_full_bill = all([
        eq(RESIDUAL_KNOWN, 1),
        raw_residual,
        Var(RESIDUAL_CHARGED).eq(Var(RESIDUAL)),
        Var(RESIDUAL_TOTAL).eq(Var(RESIDUAL_CHARGED).plus(Int(250_000))),
        all([eq(TERMINAL_DEBT, 9), Var(RESIDUAL_TOTAL).lt(Int(1_000_001))]).or(all([
            eq(TERMINAL_DEBT, 21),
            Var(RESIDUAL_TOTAL).lt(Int(2_333_334)),
        ])),
    ]);
    let unknown_full_bill = all([
        unknown_conservation_route.clone(),
        eq(INITIAL_READ_KNOWN, 0).or(early_born.clone()),
        eq(RESIDUAL_KNOWN, 0),
        eq(RESIDUAL, 0),
        eq(RESIDUAL_CHARGED, 0),
        eq(RESIDUAL_TOTAL, 3_000_000),
        eq(INITIAL_UNKNOWN_DEBT, 27),
    ]);
    safe(
        m,
        "TerminalResidualIsFullyChargedOutsideConservationCredit",
        eq(RESIDUAL_OBSERVED, 0).or(known_full_bill.or(unknown_full_bill)),
    );
    safe(
        m,
        "TerminalResidualCostNeverRewritesIndependentMemberCredits",
        eq(RESIDUAL_OBSERVED, 0).or(Var(CHARGED).eq(Var(INDEPENDENT_AT_BILL))),
    );
    safe(
        m,
        "TerminalResidualFullPaymentPrecedesConservation",
        eq(LAW, 0).or(eq(CHECKED, 0)).or(bill_paid),
    );
    safe(
        m,
        "KnownTerminalResidualOverOrdinaryLimitCannotCertifyCoverage",
        eq(RESIDUAL_KNOWN, 0)
            .or(Var(RESIDUAL_TOTAL).lt(Int(1_000_001)))
            .or(all([eq(37, 6), eq(26, 0), eq(34, 0)])),
    );
    let unique_rollup = all([eq(ROLL_UNIQUE, 1), eq(ROLL_MIXED, 0)]);
    let causal_rollup = all([
        eq(ROLL_EXIT, 1),
        eq(ROLL_ACK, 1),
        eq(ROLL_ZERO_CHILDREN, 1),
        eq(ORIGIN1, 2),
    ]);
    let counted_rollup = all([
        eq(ROLL_COUNT, 1),
        eq(ROLL_DUPLICATE, 0),
        Var(ROLL_DELTA).eq(Var(ROLL_CUMULATIVE).minus(Var(ROLL_BASELINE))),
        Var(ROLL_BASELINE).eq(Var(PRIOR_LIVE)),
        Var(REPORTED).eq(Var(A0).plus(Var(ROLL_DELTA))),
    ]);
    let rollup_proof = all([
        unique_rollup.clone(),
        causal_rollup.clone(),
        counted_rollup.clone(),
    ]);
    let read = |slot, origin| {
        all([eq(slot, 2), eq(origin, 1)])
            .or(all([eq(slot, 3), eq(origin, 2), eq(40, 1)]))
            .or(all([eq(slot, 0), eq(origin, 3)]))
            .or(all([
                Bool(slot == READ1),
                eq(slot, 4),
                eq(origin, 2),
                rollup_proof.clone(),
            ]))
    };
    let slot_fits = all([Var(W0).lt(Var(A0)).negate(), Var(W1).lt(Var(A1)).negate()]);
    safe(
        m,
        "MeasuredNeedsClosure",
        eq(BASIS, 0).or(all([
            eq(18, 1),
            eq(19, 1),
            eq(14, 1),
            eq(15, 1),
            eq(42, 0),
            eq(50, 1),
            Var(BASIS_EPOCH).eq(Var(6)),
            read(READ0, ORIGIN0),
            read(READ1, ORIGIN1),
            eq(MONOTONIC, 3),
            eq(CONVERTED, 3),
            slot_fits.clone(),
        ])),
    );
    safe(
        m,
        "PerSlotMeasuredCpuFitsWindow",
        eq(BASIS, 0).or(slot_fits),
    );
    let certified_rollup = all([eq(BASIS, 1), eq(READ1, 4)]);
    safe(
        m,
        "RollupRequiresUniqueAdmittedChild",
        certified_rollup.clone().negate().or(unique_rollup),
    );
    safe(
        m,
        "RollupNeedsCausalExitAndLeaderAck",
        certified_rollup.clone().negate().or(causal_rollup),
    );
    safe(
        m,
        "RollupCountedOnce",
        certified_rollup.negate().or(counted_rollup),
    );
    safe(
        m,
        "RollupTrustWaitsForIndependentLeaderWait4",
        eq(READ1, 4).negate().or(eq(34, 0)).or(all([
            eq(40, 1),
            eq(CHECKED, 1),
            eq(TERMINAL, 1),
            eq(ROLL_PENDING, 0),
            eq(ROLL_VALIDATED, 1),
            eq(ROLL_REVOKED, 0),
        ])),
    );
    safe(
        m,
        "RevokedRollupRetainsRawMeasuredReceiptAndFallbackBill",
        eq(ROLL_REVOKED, 0).or(all([
            eq(BASIS, 0),
            eq(RAW_BASIS, 1),
            eq(READ1, 4),
            eq(CHECKED, 1),
            eq(TERMINAL, 2),
            eq(37, 6),
            eq(26, 0),
            eq(34, 0),
            eq(ROLL_PENDING, 0),
            eq(ROLL_VALIDATED, 0),
            Var(RAW_Q).eq(Var(REPORTED).plus(Int(250_000))),
            Var(FALLBACK_Q).eq(Var(WALL).plus(Int(250_000))),
            Var(QNS).eq(Var(FALLBACK_Q)),
            eq(FALLBACK_DEBT, 9),
            Int(0).lt(Var(EXCESS)),
            rhs.clone().lt(Var(WAIT4)),
        ])),
    );
    safe(
        m,
        "InitialTerminalConservationNeedsKnownIndependentObservation",
        initial_eligible
            .clone()
            .negate()
            .or(eq(TERMINAL, 1).negate())
            .or(all([
                eq(INITIAL_ATTEMPT, 1),
                eq(INITIAL_READ_KNOWN, 1),
                eq(INITIAL_REST_PAID, 1),
            ])),
    );
    safe(
        m,
        "ChargedCoversBurned",
        eq(TERMINAL, 1).negate().or(all([
            eq(40, 1),
            eq(CHECKED, 1),
            conserved.clone(),
            rhs.clone().lt(Var(BURNED)).negate(),
        ])),
    );
    safe(
        m,
        "ExitedDirectMeasuredSourceRequiresOriginalOwnWait4",
        eq(LAW, 12)
            .negate()
            .or(eq(BASIS, 0))
            .or(all([eq(40, 1), eq(READ0, 3), eq(ORIGIN0, 2)])),
    );
    safe(
        m,
        "PendingDirectExitCannotAdmitExpensiveWorkOrTrust",
        eq(PENDING, 0).or(all([
            eq(BASIS, 0),
            eq(26, 0),
            eq(QNS, 0),
            eq(CHECKED, 0),
            // Only the new two-grant exited-only profile has a prior real
            // commitment. Preserve it; the CURRENT grant stays uncommitted.
            eq(LAW, 44)
                .and(Var(COMMITTED).plus(Int(1)).eq(Var(49)))
                .or(eq(LAW, 44).negate().and(eq(COMMITTED, 0))),
            eq(33, 0),
            eq(34, 0),
        ])),
    );
    safe(
        m,
        "CounterProvenanceDistinguishesStoppedFromExited",
        all([
            eq(READ0, 2).negate().or(eq(ORIGIN0, 1)),
            eq(READ1, 2).negate().or(eq(ORIGIN1, 1)),
            eq(READ0, 3).negate().or(all([eq(ORIGIN0, 2), eq(40, 1)])),
            eq(READ1, 3).negate().or(all([eq(ORIGIN1, 2), eq(40, 1)])),
        ]),
    );
    safe(
        m,
        "ActualWait4RawUsageRemainsIndependentOfChargedLedger",
        eq(LAW, 0).or(eq(40, 0)).or(Var(WAIT4).eq(Var(BURNED))),
    );
    safe(
        m,
        "WallBounded",
        eq(LAW, 0).or(eq(26, 0)).or(Var(WALL).lt(Int(25_000_001))),
    );
    safe(
        m,
        "CertifiedBasisQuantumIsOrdinary",
        eq(LAW, 0).or(eq(26, 0)).or(Var(QNS).lt(Int(1_000_001))),
    );
    safe(
        m,
        "BasisQuantumIncludesAllActiveControllerCpu",
        eq(LAW, 0).or(eq(QNS, 0)).or(all([
            eq(BASIS, 1),
            Var(QNS).eq(Var(REPORTED).plus(Int(250_000))),
        ])
        .or(all([
            eq(BASIS, 0),
            all([
                Var(WALL).lt(Var(REPORTED)).negate(),
                Var(QNS).eq(Var(WALL).plus(Int(250_000))),
            ])
            .or(all([
                Var(WALL).lt(Var(REPORTED)),
                Var(QNS).eq(Var(REPORTED).plus(Int(250_000))),
            ])),
        ]))),
    );
    let full_debt = (9..=27)
        .map(|debt| {
            all([
                eq(CREATED_DEBT, debt),
                Var(QNS).lt(Int(debt * 1_000_000 / 9 + 1)),
            ])
        })
        .fold(Bool(false), Expr::or);
    safe(
        m,
        "CreatedDebtCoversUnclampedBasisQuantum",
        eq(LAW, 0).or(eq(QNS, 0)).or(full_debt),
    );
    let refused_initial_unknown = all([
        unknown_conservation_route,
        eq(INITIAL_UNKNOWN, 1),
        eq(INITIAL_ATTEMPT, 1),
        eq(INITIAL_READ_KNOWN, 0).or(early_born.clone()),
        eq(INITIAL_REST_PAID, 1),
        eq(INITIAL_L, 0),
        eq(INITIAL_COST_BOUND, 3_000_000),
        eq(INITIAL_ATTEMPT_DEBT, 27),
        eq(INITIAL_UNKNOWN_DEBT, 27),
        eq(40, 1),
        eq(CHECKED, 1),
        eq(TERMINAL, 0),
        eq(EXCESS, 0),
        eq(37, 6),
        eq(26, 0),
        eq(34, 0),
    ]);
    safe(
        m,
        "UnknownInitialConservationNeverFabricatesKnownExcessOrPass",
        eq(INITIAL_UNKNOWN, 0).or(refused_initial_unknown.clone()),
    );
    safe(
        m,
        "ConservationFailureRetainsRawAndRefusesTrust",
        all([
            eq(CHECKED, 0)
                .or(conserved)
                .or(refused_initial_unknown)
                .or(all([
                    eq(TERMINAL, 2),
                    eq(26, 0),
                    eq(34, 0),
                    eq(37, 6),
                    Var(EXCESS).eq(Var(WAIT4).minus(rhs.clone())),
                ])),
            eq(TERMINAL, 2).negate().or(all([
                eq(40, 1),
                eq(CHECKED, 1),
                Int(0).lt(Var(EXCESS)),
                eq(26, 0),
                eq(34, 0),
                all([eq(TERMINAL_DEBT, 9), Var(EXCESS).lt(Int(1_000_001))])
                    .or(all([eq(TERMINAL_DEBT, 21), Var(EXCESS).lt(Int(2_333_334))])),
            ])),
        ]),
    );
    safe(
        m,
        "MeasuredTrustRequiresTerminalConservation",
        eq(LAW, 0)
            .or(eq(34, 0))
            .or(all([eq(CHECKED, 1), eq(TERMINAL, 1)])),
    );
    progress(
        m,
        "ActualReapGetsLocalConservationOutcome",
        all([
            new_law,
            eq(40, 1),
            eq(CHECKED, 0),
            eq(PENDING, 0),
            eq(27, 0),
            eq(28, 0),
        ]),
        eq(CHECKED, 1),
        (bill_start..refuse_end).collect(),
    );
    progress(
        m,
        "RetainedOwnWait4CanResolveDeferredGrantWithoutAnotherCont",
        all([eq(LAW, 12), eq(PENDING, 1), eq(40, 1), eq(27, 0), eq(28, 0)]),
        eq(PENDING, 0),
        vec![bind_direct, commit_direct, refuse_direct],
    );
}

/// The13:00 scheduler amendment changes allocation, NEVER the accounting law.
/// Four correlated trajectories refine the SAME original owner, two-grant,
/// ACK/roster/seal, full-C charge/debt and actual-reap actions. C/STOP reserves
/// are maxima of retained matching observations; an unavailable clock grants
/// no CONT. No scheduler estimate is a physical CPU/ACK-latency guarantee.
/// Prefix ATT retains the old two-slot capacity reservation. Profile38 proves
/// the second capacity slot absent under the immutable expected-one source;
/// profiles39/40 divide by BOTH expected slots, before either is observed live.
fn extend_member_window_adaptive_grants(m: &mut Model, control: u8) {
    const LAW: usize = 54;
    const BASIS: usize = 55;
    const READ0: usize = 56;
    const READ1: usize = 57;
    const BURNED: usize = 58;
    const CHARGED: usize = 59;
    const WALL: usize = 60;
    const A0: usize = 61;
    const A1: usize = 62;
    const W0: usize = 63;
    const W1: usize = 64;
    const MONOTONIC: usize = 65;
    const CONVERTED: usize = 66;
    const QNS: usize = 69;
    const REPORTED: usize = 70;
    const SEEN: usize = 71;
    const BASIS_EPOCH: usize = 73;
    const CREATED_DEBT: usize = 74;
    const ORIGIN0: usize = 76;
    const ORIGIN1: usize = 77;
    const COMMITTED: usize = 80;
    const RAW_Q: usize = 90;
    const RAW_BASIS: usize = 91;
    const INITIAL_READY: usize = 102;
    const TEARDOWN_LIMIT: usize = 119;
    const RUN: usize = 145;
    const CAP: usize = 146;
    const TARGET: usize = 147;
    const C_RESERVE: usize = 148;
    const LATENCY_RESERVE: usize = 149;
    const SLOTS: usize = 150;
    const USED_RUN: usize = 151;
    const USED_CAP: usize = 152;
    const USED_TARGET: usize = 153;
    const CURRENT_C: usize = 154;
    const CURRENT_LATENCY: usize = 155;
    const CURRENT_A: usize = 156;
    const FEEDBACKS: usize = 157;
    const CLOCK_KNOWN: usize = 158;
    const ELAPSED_STAGE: usize = 159;
    const ELAPSED: usize = 160;
    const OPTIONAL_COUNT: usize = 161;
    const OPTIONAL_AT: usize = 162;
    const OPTIONAL_FRESH: usize = 163;
    const NEXT_AFTER_FEEDBACK: usize = 164;
    const FEEDBACK_KIND: usize = 165;
    assert_eq!(m.variables.len(), RUN);
    for (name, max) in [
        ("adaptiveRunBudgetNs", 1_000_000),
        ("adaptiveRunCapNs", 450_000),
        ("adaptiveAggregateTargetANs", 550_000),
        ("adaptiveMaxKnownControllerReserveNs", 500_000),
        ("adaptiveMaxMatchingStopAckLatencyNs", 500_000),
        ("adaptiveExpectedReservedSlots", 2),
        ("issuedAdaptiveRunBudgetNs", 1_000_000),
        ("issuedAdaptiveRunCapNs", 450_000),
        ("issuedAdaptiveAggregateTargetANs", 550_000),
        ("adaptiveCurrentCompleteControllerNs", 500_000),
        ("adaptiveCurrentMatchingStopAckLatencyNs", 500_000),
        ("adaptiveCurrentAggregateActiveNs", 2_500_000),
        ("adaptiveAppliedClosedGrantReceipts", 2),
        ("adaptiveElapsedClockKnown", 1),
        ("adaptiveFreshElapsedNonePreBudgetStopDue", 2),
        ("adaptiveFreshElapsedNs", 1_000_001),
        ("adaptiveOptionalWorkCountInGrant", 1),
        ("adaptiveOptionalWorkPriorElapsedNs", 1_000_000),
        ("adaptiveOptionalWorkHadFreshClock", 1),
        ("adaptiveRetainedNextRunAfterFeedbackNs", 450_000),
        ("adaptiveFeedbackNoneHalveGrowHold", 3),
    ] {
        m.variables.push(Variable::new(name, 0, max, 0));
    }
    assert_eq!(
        m.variables
            .iter()
            .map(|v| &v.name)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        m.variables.len(),
        "adaptive fields keep all exported variable names unique"
    );
    let adaptive = Int(37).lt(Var(LAW));
    let limits = |controller: i64, latency: i64, slots: i64| {
        let target = (1_000_000 - controller - 200_000).clamp(50_000, 550_000);
        let cap = ((target / slots).min(450_000) - latency).clamp(25_000, 450_000);
        (target, cap)
    };
    let next_budget = |run: i64, aggregate: i64, target: i64, cap: i64| {
        if aggregate > target {
            ((run / 2).max(25_000).min(cap), 1)
        } else if aggregate <= target * 3 / 4 {
            ((run + 10_000).min(cap), 2)
        } else {
            (run.min(cap), 3)
        }
    };
    for profile in 38..=41 {
        let slots = if profile == 38 { 1 } else { 2 };
        let (target, cap) = limits(250_000, 100_000, slots);
        let run = if control == 45 && profile == 39 {
            1_000_000
        } else {
            (350_000 / slots).min(cap)
        };
        act(
            m,
            &format!("ConfigureRetainedAdaptiveSchedulerProfile{profile}"),
            all([eq(2, 0), eq(0, 0)]),
            &[
                (LAW, Int(profile)),
                (1, Int(if slots == 1 { 2 } else { 1 })),
                (2, Int(1)),
                (TEARDOWN_LIMIT, Int(150_000)),
                (MONOTONIC, Int(3)),
                (CONVERTED, Int(3)),
                (RUN, Int(run)),
                (CAP, Int(cap)),
                (TARGET, Int(target)),
                (C_RESERVE, Int(250_000)),
                (LATENCY_RESERVE, Int(100_000)),
                (SLOTS, Int(slots)),
                (CLOCK_KNOWN, Int(i64::from(profile != 41))),
            ],
        );
    }
    let mut scheduled_cont = vec![];
    for action in &mut m.actions {
        if action.name == "CheckedT0OpensEveryReservedSlotBeforeCont"
            || action.name == "ContinueOnlyAfterClosureCorroborationAndAllDebt"
        {
            for slots in 1..=2 {
                let mut scheduled = action.clone();
                scheduled
                    .name
                    .push_str(&format!("WithAdaptiveExpectedSlots{slots}"));
                scheduled.guard = scheduled.guard.and(all([
                    adaptive.clone(),
                    eq(SLOTS, slots),
                    eq(CLOCK_KNOWN, 1),
                    // The preceding closed grant's local feedback cannot be
                    // skipped by a new CONT; all existing paid debt guards stay.
                    Var(FEEDBACKS).eq(Var(49)),
                ]));
                scheduled.updates.extend([
                    (USED_RUN, Var(RUN)),
                    (USED_CAP, Var(CAP)),
                    (USED_TARGET, Var(TARGET)),
                    (ELAPSED_STAGE, Int(0)),
                    (ELAPSED, Int(0)),
                    (OPTIONAL_COUNT, Int(0)),
                    (OPTIONAL_AT, Int(0)),
                    (OPTIONAL_FRESH, Int(0)),
                ]);
                if slots == 1 {
                    // Expected-one is an immutable source fact, not census
                    // evidence that a possible second birth never happened.
                    for (slot, value) in &mut scheduled.updates {
                        if *slot == 8 {
                            *value = Int(0);
                        }
                    }
                }
                scheduled_cont.push(scheduled);
            }
            action.guard = action.guard.clone().and(adaptive.clone().negate());
        }
    }
    m.actions.extend(scheduled_cont);
    act(
        m,
        "ExpectedOneSourceKeepsUnusedCapacitySlotClosedAbsent",
        all([
            eq(LAW, 38).or(eq(LAW, 44)).or(eq(LAW, 45)).or(eq(LAW, 46)),
            eq(0, 4),
            eq(10, 1),
            eq(11, 1),
            eq(12, 2),
            eq(8, 0),
            eq(15, 0),
            eq(37, 0),
            eq(SEEN, 1),
        ]),
        &[(0, Int(5))],
    );
    act(
        m,
        "UnknownFreshElapsedClockRefusesBeforeAdaptiveCont",
        all([
            eq(LAW, 41),
            eq(0, 3),
            eq(INITIAL_READY, 1),
            eq(CLOCK_KNOWN, 0),
            eq(49, 0),
            eq(37, 0),
            eq(27, 0),
            eq(28, 0),
        ]),
        &[(25, Int(0)), (37, Int(1)), (26, Int(0)), (34, Int(0))],
    );
    // One bounded optional controller operation is selected in profiles38/39.
    // It must use a fresh current clock; its work invalidates that timestamp.
    // The next read observes STOP due. Profile40 starts already due and does
    // NO optional work: scheduler allocation cannot promise kernel latency.
    act(
        m,
        "ReadFreshElapsedBeforeOptionalGrantWork",
        all([
            adaptive.clone(),
            eq(0, 4),
            eq(SEEN, 0),
            eq(CLOCK_KNOWN, 1),
            eq(ELAPSED_STAGE, 0),
            eq(OPTIONAL_COUNT, 0),
            eq(LAW, 40).negate(),
        ]),
        &[(ELAPSED_STAGE, Int(1)), (ELAPSED, Int(0))],
    );
    act(
        m,
        "OptionalGrantWorkRequiresFreshCurrentPreBudgetElapsed",
        all([
            adaptive.clone(),
            eq(0, 4),
            eq(SEEN, 0),
            eq(CLOCK_KNOWN, 1),
            eq(ELAPSED_STAGE, 1),
            eq(OPTIONAL_COUNT, 0),
            Var(ELAPSED).lt(Var(USED_RUN)),
        ]),
        &[
            (OPTIONAL_COUNT, Int(1)),
            (OPTIONAL_AT, Var(ELAPSED)),
            (OPTIONAL_FRESH, Int(1)),
            (ELAPSED_STAGE, Int(0)),
        ],
    );
    act(
        m,
        "RereadCurrentElapsedAfterOptionalWorkBeforeStop",
        all([
            adaptive.clone(),
            eq(0, 4),
            eq(SEEN, 0),
            eq(CLOCK_KNOWN, 1),
            eq(ELAPSED_STAGE, 0),
            eq(OPTIONAL_COUNT, 1),
        ]),
        &[(ELAPSED_STAGE, Int(2)), (ELAPSED, Var(USED_RUN))],
    );
    act(
        m,
        "FreshElapsedAlreadyDueSkipsOptionalWorkAndStops",
        all([
            eq(LAW, 40),
            eq(0, 4),
            eq(SEEN, 0),
            eq(CLOCK_KNOWN, 1),
            eq(ELAPSED_STAGE, 0),
            eq(OPTIONAL_COUNT, 0),
        ]),
        &[
            (ELAPSED_STAGE, Int(2)),
            (ELAPSED, Var(USED_RUN).plus(Int(1))),
        ],
    );
    let feedback_start = m.actions.len();
    for profile in 38..=40 {
        let slots: i64 = if profile == 38 { 1 } else { 2 };
        let (_, cap) = limits(250_000, 100_000, slots);
        let mut controller_reserve = 250_000;
        let mut latency_reserve = 100_000;
        let mut run = if control == 45 && profile == 39 {
            1_000_000
        } else {
            (350_000 / slots).min(cap)
        };
        let grants = if profile == 40 || (control == 45 && profile == 39) {
            1
        } else {
            2
        };
        for grant in 1..=grants {
            let (a0, a1, w0, w1, controller, latency) = match (profile, grant) {
                (38, 1) => (600_000, 0, 600_000, 0, 250_000, 250_000),
                (38, 2) => (250_000, 0, 275_000, 0, 350_000, 100_000),
                (39, 1) if control == 45 => {
                    (1_100_000, 1_100_000, 1_100_000, 1_100_000, 250_000, 100_000)
                }
                (39, 1) => (300_000, 300_000, 300_000, 300_000, 250_000, 125_000),
                (39, 2) => (125_000, 125_000, 212_500, 212_500, 200_000, 125_000),
                // Retained builder refusal: Q=932917+245583=1178500.
                // These raw observations are NOT changed into acceptance.
                (40, 1) => (466_458, 466_459, 557_000, 557_000, 245_583, 382_000),
                _ => unreachable!(),
            };
            let aggregate = a0 + a1;
            let wall = w0 + w1;
            assert!(aggregate <= wall);
            assert!(w0 <= run + latency && w1 <= run + latency);
            let q = aggregate + controller;
            let debt = (9 * q.max(1_000_000) + 999_999) / 1_000_000;
            assert!(debt <= 27 && q <= 3_000_000);
            act(
                m,
                &format!(
                    "ObserveActualAdaptiveGrantCpuAndMatchingStopAckProfile{profile}Grant{grant}"
                ),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(0, 4),
                    eq(SEEN, 0),
                    eq(14, 0),
                    eq(ELAPSED_STAGE, 2),
                    eq(USED_RUN, run),
                ]),
                &[
                    (BURNED, Var(BURNED).plus(Int(aggregate))),
                    (SEEN, Int(1)),
                    (A0, Int(a0)),
                    (A1, Int(a1)),
                    (W0, Int(w0)),
                    (W1, Int(w1)),
                    (WALL, Int(wall)),
                    (CURRENT_C, Int(controller)),
                    (CURRENT_LATENCY, Int(latency)),
                    (CURRENT_A, Int(aggregate)),
                ],
            );
            act(
                m,
                &format!(
                    "ReadAdaptiveCountersAfterOriginalPositiveAckProfile{profile}Grant{grant}"
                ),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(0, 6),
                    eq(14, 1),
                    eq(15, 1),
                    eq(18, 1),
                    eq(19, 1),
                    eq(20, 1),
                    eq(7, 3),
                    eq(8, if slots == 1 { 0 } else { 3 }),
                    eq(READ0, 0),
                ]),
                &[
                    (READ0, Int(2)),
                    (ORIGIN0, Int(1)),
                    (READ1, Int(if slots == 1 { 0 } else { 2 })),
                    (ORIGIN1, Int(if slots == 1 { 3 } else { 1 })),
                    (REPORTED, Int(aggregate)),
                ],
            );
            act(
                m,
                &format!(
                    "ChargeFullAdaptiveRawQuantumWithoutChangingHardGateProfile{profile}Grant{grant}"
                ),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(0, 6),
                    eq(37, 0),
                    eq(SEEN, 1),
                    eq(READ0, 2),
                    eq(REPORTED, aggregate),
                    eq(18, 1),
                    eq(19, 1),
                    eq(20, 1),
                    eq(27, 0),
                    eq(28, 0),
                    Var(5).eq(Var(6)),
                    Var(48).lt(Int(28 - debt)),
                ]),
                &[
                    (0, Int(7)),
                    (33, Int(0)),
                    (BASIS, Int(1)),
                    (BASIS_EPOCH, Var(5)),
                    (QNS, Int(q)),
                    (RAW_Q, Int(q)),
                    (RAW_BASIS, Int(1)),
                    (CHARGED, Var(CHARGED).plus(Int(aggregate))),
                    (COMMITTED, Var(COMMITTED).plus(Int(1))),
                    (CREATED_DEBT, Int(debt)),
                    (26, Int(i64::from(q <= 1_000_000 && wall <= 25_000_000))),
                    (
                        37,
                        Int(if q <= 1_000_000 && wall <= 25_000_000 {
                            0
                        } else {
                            6
                        }),
                    ),
                    (27, Int(debt)),
                    (28, Int(debt)),
                    (48, Var(48).plus(Int(debt))),
                    (21, Int(2)),
                    (22, Int(1)),
                    (23, Int(1)),
                    (24, Int(3)),
                    (43, Int(1)),
                    (44, Int(1)),
                    (46, Int(1)),
                ],
            );
            controller_reserve = controller_reserve.max(controller);
            latency_reserve = latency_reserve.max(latency);
            let (next_target, next_cap) = limits(controller_reserve, latency_reserve, slots);
            let (next_run, kind) = next_budget(run, aggregate, next_target, next_cap);
            act(
                m,
                &format!(
                    "ApplyRetainedAggregateFeedbackWithoutRepeatingNativeEffectProfile{profile}Grant{grant}"
                ),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(COMMITTED, grant),
                    eq(FEEDBACKS, grant - 1),
                    eq(CLOCK_KNOWN, 1),
                    eq(CURRENT_A, aggregate),
                    eq(CURRENT_C, controller),
                    eq(CURRENT_LATENCY, latency),
                    eq(USED_RUN, run),
                ]),
                &[
                    (C_RESERVE, Int(controller_reserve)),
                    (LATENCY_RESERVE, Int(latency_reserve)),
                    (TARGET, Int(next_target)),
                    (CAP, Int(next_cap)),
                    (RUN, Int(next_run)),
                    (NEXT_AFTER_FEEDBACK, Int(next_run)),
                    (FEEDBACK_KIND, Int(kind)),
                    (FEEDBACKS, Int(grant)),
                ],
            );
            run = next_run;
            assert!(run >= 25_000 && run <= next_cap && next_target >= 50_000);
        }
    }
    let feedback_end = m.actions.len();
    // Old profiles retain their literal250us C projection. New scheduler
    // trajectories retain their OWN measured full C without omitting any tail.
    for invariant in &mut m.invariants {
        if invariant.name == "BasisQuantumIncludesAllActiveControllerCpu" {
            invariant.condition = adaptive
                .clone()
                .negate()
                .and(invariant.condition.clone())
                .or(all([
                    adaptive.clone(),
                    eq(QNS, 0).or(all([
                        eq(BASIS, 1),
                        Var(QNS).eq(Var(REPORTED).plus(Var(CURRENT_C))),
                    ])),
                ]));
        }
    }
    safe(
        m,
        "IssuedAdaptiveGrantUsesMeasuredCostAndStopLatencyCap",
        adaptive.clone().negate().or(eq(COMMITTED, 0)).or(all([
            Int(24_999).lt(Var(USED_RUN)),
            Var(USED_CAP).lt(Var(USED_RUN)).negate(),
            Var(USED_CAP).lt(Int(450_001)),
        ])),
    );
    safe(
        m,
        "AdaptiveFeedbackKeepsCurrentPlanWithinBoundedCap",
        adaptive.clone().negate().or(eq(FEEDBACKS, 0)).or(all([
            Var(RUN).eq(Var(NEXT_AFTER_FEEDBACK)),
            Int(24_999).lt(Var(RUN)),
            Var(CAP).lt(Var(RUN)).negate(),
            Var(CAP).lt(Int(450_001)),
            Int(49_999).lt(Var(TARGET)),
            Var(TARGET).lt(Int(550_001)),
            Int(249_999).lt(Var(C_RESERVE)),
            Int(99_999).lt(Var(LATENCY_RESERVE)),
        ])),
    );
    safe(
        m,
        "AdaptiveOptionalWorkRequiresFreshPreBudgetClockReceipt",
        adaptive.clone().negate().or(eq(OPTIONAL_COUNT, 0)).or(all([
            eq(OPTIONAL_FRESH, 1),
            eq(CLOCK_KNOWN, 1),
            Var(OPTIONAL_AT).lt(Var(USED_RUN)),
        ])),
    );
    safe(
        m,
        "UnknownElapsedClockCannotContinueOrTrustAdaptiveOwner",
        adaptive.clone().negate().or(eq(CLOCK_KNOWN, 1)).or(all([
            eq(49, 0),
            eq(COMMITTED, 0),
            eq(FEEDBACKS, 0),
            eq(34, 0),
            eq(26, 0),
        ])),
    );
    safe(
        m,
        "ExpectedOneSourceNeverActivatesUnusedCapacitySlot",
        eq(LAW, 38).negate().or(eq(49, 0)).or(all([
            eq(SLOTS, 1),
            eq(8, 0).or(eq(8, 4)),
            eq(A1, 0),
            eq(W1, 0),
        ])),
    );
    safe(
        m,
        "AdaptiveOverQuantumRefusalRetainsRawDebtAndNoTrust",
        adaptive
            .clone()
            .negate()
            .or(Var(QNS).lt(Int(1_000_001)))
            .or(all([
                eq(37, 6),
                eq(26, 0),
                eq(34, 0),
                Int(0).lt(Var(CREATED_DEBT)),
            ])),
    );
    let feedback_actions = (feedback_start..feedback_end)
        .filter(|index| {
            m.actions[*index]
                .name
                .starts_with("ApplyRetainedAggregateFeedback")
        })
        .collect();
    progress(
        m,
        "ClosedAdaptiveReceiptGetsLocalBoundedFeedback",
        all([
            adaptive,
            Var(FEEDBACKS).lt(Var(COMMITTED)),
            eq(CLOCK_KNOWN, 1),
        ]),
        Var(FEEDBACKS).eq(Var(COMMITTED)),
        feedback_actions,
    );
}

/// Predictive coherence: per-slot retained ACK provenance refines the existing
/// adaptive state. MAX uses only live same-grant successful-STOP ACKs; exit
/// counters remain valid accounting sources without becoming latency proofs.
/// Five correlated profiles preserve the original <=2 grants and100000 cap.
fn extend_adaptive_latency_provenance(m: &mut Model, control: u8) {
    const LAW: usize = 54;
    const BASIS: usize = 55;
    const READ0: usize = 56;
    const READ1: usize = 57;
    const BURNED: usize = 58;
    const CHARGED: usize = 59;
    const WALL: usize = 60;
    const A0: usize = 61;
    const A1: usize = 62;
    const W0: usize = 63;
    const W1: usize = 64;
    const MONOTONIC: usize = 65;
    const CONVERTED: usize = 66;
    const QNS: usize = 69;
    const REPORTED: usize = 70;
    const SEEN: usize = 71;
    const BASIS_EPOCH: usize = 73;
    const CREATED_DEBT: usize = 74;
    const ORIGIN0: usize = 76;
    const ORIGIN1: usize = 77;
    const PENDING: usize = 78;
    const WAIT4: usize = 79;
    const COMMITTED: usize = 80;
    const RAW_Q: usize = 90;
    const RAW_BASIS: usize = 91;
    const TEARDOWN_LIMIT: usize = 119;
    const RUN: usize = 145;
    const CAP: usize = 146;
    const TARGET: usize = 147;
    const C_RESERVE: usize = 148;
    const LATENCY_RESERVE: usize = 149;
    const SLOTS: usize = 150;
    const USED_RUN: usize = 151;
    const CURRENT_C: usize = 154;
    const CURRENT_LATENCY: usize = 155;
    const CURRENT_A: usize = 156;
    const FEEDBACKS: usize = 157;
    const CLOCK_KNOWN: usize = 158;
    const ELAPSED_STAGE: usize = 159;
    const NEXT_AFTER_FEEDBACK: usize = 164;
    const FEEDBACK_KIND: usize = 165;
    const LIVE_MASK: usize = 166;
    const DELAY0: usize = 167;
    const DELAY1: usize = 168;
    const STOP_MATCH: usize = 169;
    const LATENCY_KNOWN: usize = 170;
    assert_eq!(m.variables.len(), LIVE_MASK);
    for (name, max) in [
        ("adaptiveRetainedValidLiveStopAckMask", 3),
        ("adaptiveRetainedSlot0StopAckDelayNs", 500_000),
        ("adaptiveRetainedSlot1StopAckOrExitDelayNs", 500_000),
        ("adaptiveRetainedSuccessfulSameGrantStop", 1),
        ("adaptiveCurrentLiveLatencyKnown", 1),
    ] {
        m.variables.push(Variable::new(name, 0, max, 0));
    }
    assert_eq!(
        m.variables
            .iter()
            .map(|v| &v.name)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        m.variables.len()
    );
    let limits = |c: i64, latency: i64, slots: i64| {
        let target = (1_000_000 - c - 200_000).clamp(50_000, 550_000);
        (
            target,
            ((target / slots).min(450_000) - latency).clamp(25_000, 450_000),
        )
    };
    for profile in 42..=46 {
        let slots = if matches!(profile, 44..=46) { 1 } else { 2 };
        let (target, cap) = limits(250_000, 100_000, slots);
        act(
            m,
            &format!("ConfigureAdaptiveLiveLatencyProfile{profile}"),
            all([eq(2, 0), eq(0, 0)]),
            &[
                (LAW, Int(profile)),
                (1, Int(if slots == 1 { 2 } else { 1 })),
                (2, Int(1)),
                (TEARDOWN_LIMIT, Int(150_000)),
                (MONOTONIC, Int(3)),
                (CONVERTED, Int(3)),
                (RUN, Int((350_000 / slots).min(cap))),
                (CAP, Int(cap)),
                (TARGET, Int(target)),
                (C_RESERVE, Int(250_000)),
                (LATENCY_RESERVE, Int(100_000)),
                (SLOTS, Int(slots)),
                (CLOCK_KNOWN, Int(1)),
            ],
        );
    }
    let mut feedback_actions = vec![];
    let mut pending_actions = vec![];
    for profile in 42..=46 {
        let slots = if matches!(profile, 44..=46) { 1 } else { 2 };
        let mut c_reserve = 250_000;
        let mut latency_reserve = 100_000;
        let (_, cap) = limits(c_reserve, latency_reserve, slots);
        let mut run = (350_000 / slots).min(cap);
        let grants = if matches!(profile, 43 | 44) { 2 } else { 1 };
        for grant in 1..=grants {
            // A/C/W are independent retained accounting observations. Delay1
            // may be an EXIT receipt and is then excluded despite being larger.
            let (a0, a1, w0, w1, c, delay0, delay1, mask, matched, measured) =
                match (profile, grant) {
                    (42, 1) => (
                        250_000, 275_000, 275_000, 475_000, 300_000, 100_000, 300_000, 3, 1, true,
                    ),
                    (43, 1) => (
                        300_000, 300_000, 300_000, 300_000, 250_000, 125_000, 125_000, 3, 1, true,
                    ),
                    (43, 2) => (
                        250_000, 50_000, 300_000, 400_000, 250_000, 100_000, 300_000, 1, 1, true,
                    ),
                    (44, 1) => (600_000, 0, 600_000, 0, 250_000, 250_000, 0, 1, 1, true),
                    (44, 2) => (100_000, 0, 275_000, 0, 250_000, 100_000, 0, 0, 1, true),
                    (45, 1) => (525_000, 0, 625_000, 0, 300_000, 100_000, 0, 1, 1, false),
                    // Positive stopped accounting can follow an independently
                    // observed quiescent source even if STOP itself was not a
                    // matching successful call. It earns no scheduler latency.
                    (46, 1) => (250_000, 0, 450_000, 0, 250_000, 100_000, 0, 0, 0, false),
                    _ => unreachable!(),
                };
            let aggregate = a0 + a1;
            let wall = w0 + w1;
            let member = if measured {
                aggregate
            } else {
                wall.max(aggregate)
            };
            let q = member + c;
            let debt = (9 * q.max(1_000_000) + 999_999) / 1_000_000;
            assert!(q <= 1_000_000 && a0 <= w0 && a1 <= w1 && debt == 9);
            let live_latency = match mask {
                1 => delay0,
                2 => delay1,
                3 => delay0.max(delay1),
                _ => 0,
            };
            let mut latency_known = mask != 0 && matched == 1;
            let mut selected_latency = live_latency;
            if control == 46 && profile == 42 {
                selected_latency = delay0.min(delay1); // one faulty aggregation
            }
            if control == 47 && matches!(profile, 44 | 46) && grant == grants {
                // One faulty classification accepts the real retained exit or
                // failed-call endpoint as live latency. Eligible exit growth follows;
                // no raw ACK, actual own wait4, CPU or bill is forged.
                latency_known = true;
                selected_latency = delay0;
            }
            act(
                m,
                &format!("ObserveRetainedAdaptiveAckProvenanceProfile{profile}Grant{grant}"),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(0, 4),
                    eq(SEEN, 0),
                    eq(14, 0),
                    eq(ELAPSED_STAGE, 2),
                    eq(USED_RUN, run),
                ]),
                &[
                    (BURNED, Var(BURNED).plus(Int(aggregate))),
                    (SEEN, Int(1)),
                    (A0, Int(a0)),
                    (A1, Int(a1)),
                    (W0, Int(w0)),
                    (W1, Int(w1)),
                    (WALL, Int(wall)),
                    (CURRENT_C, Int(c)),
                    (CURRENT_A, Int(aggregate)),
                    (CURRENT_LATENCY, Int(selected_latency)),
                    (LIVE_MASK, Int(mask)),
                    (DELAY0, Int(delay0)),
                    (DELAY1, Int(delay1)),
                    (STOP_MATCH, Int(matched)),
                    (LATENCY_KNOWN, Int(i64::from(latency_known))),
                ],
            );
            let direct = profile == 44 && grant == 2;
            let rollup = profile == 43 && grant == 2;
            if direct {
                act(
                    m,
                    "DeferExitedOnlyAdaptiveAccountingWithoutLiveStopAck",
                    all([
                        eq(LAW, 44),
                        eq(49, 2),
                        eq(0, 6),
                        eq(7, 4),
                        eq(8, 0),
                        eq(14, 1),
                        eq(15, 1),
                        eq(18, 1),
                        eq(19, 1),
                        eq(20, 1),
                        eq(READ0, 0),
                        eq(REPORTED, 0),
                        eq(PENDING, 0),
                    ]),
                    &[(PENDING, Int(1)), (ORIGIN0, Int(2)), (ORIGIN1, Int(3))],
                );
                let bind = m.actions.len();
                act(
                    m,
                    "BindExitedOnlyAdaptiveActualOwnWait4WithoutLiveLatency",
                    all([
                        eq(LAW, 44),
                        eq(49, 2),
                        eq(PENDING, 1),
                        eq(40, 1),
                        eq(READ0, 0),
                        eq(ORIGIN0, 2),
                        eq(ORIGIN1, 3),
                    ]),
                    &[(READ0, Int(3)), (REPORTED, Var(WAIT4).minus(Var(CHARGED)))],
                );
                pending_actions.push(bind);
            } else {
                let mut updates = vec![
                    (READ0, Int(if measured { 2 } else { 0 })),
                    (ORIGIN0, Int(1)),
                    (
                        READ1,
                        Int(if slots == 1 {
                            0
                        } else if rollup {
                            4
                        } else {
                            2
                        }),
                    ),
                    (
                        ORIGIN1,
                        Int(if slots == 1 {
                            3
                        } else if rollup {
                            2
                        } else {
                            1
                        }),
                    ),
                    (REPORTED, Int(aggregate)),
                ];
                if rollup {
                    updates.extend([
                        (81, Int(1)),
                        (82, Int(0)),
                        (83, Int(1)),
                        (84, Int(1)),
                        (85, Int(1)),
                        (86, Int(350_000)),
                        (87, Var(88)),
                        (89, Int(0)),
                        (96, Int(350_000).minus(Var(88))),
                        (98, Int(1)),
                    ]);
                }
                act(
                    m,
                    &format!("ReadAdaptiveLiveOrQualifiedExitSourceProfile{profile}Grant{grant}"),
                    all([
                        eq(LAW, profile),
                        eq(49, grant),
                        eq(0, 6),
                        eq(7, 3),
                        eq(
                            8,
                            if slots == 1 {
                                0
                            } else if rollup {
                                4
                            } else {
                                3
                            },
                        ),
                        eq(14, 1),
                        eq(15, 1),
                        eq(18, 1),
                        eq(19, 1),
                        eq(20, 1),
                        eq(READ0, 0),
                        eq(REPORTED, 0),
                    ]),
                    &updates,
                );
            }
            let mut charged = vec![
                (0, Int(if direct { 9 } else { 7 })),
                (33, Int(0)),
                (BASIS, Int(i64::from(measured))),
                (BASIS_EPOCH, Var(5)),
                (QNS, Int(q)),
                (RAW_Q, Int(q)),
                (RAW_BASIS, Int(i64::from(measured))),
                (CHARGED, Var(CHARGED).plus(Int(member))),
                (COMMITTED, Var(COMMITTED).plus(Int(1))),
                (CREATED_DEBT, Int(debt)),
                (26, Int(i64::from(profile != 46))),
                (27, Int(debt)),
                (28, Int(debt)),
                (48, Var(48).plus(Int(debt))),
                (21, Int(2)),
                (22, Int(1)),
                (23, Int(1)),
                (24, Int(3)),
                (43, Int(1)),
                (44, Int(1)),
                (46, Int(1)),
            ];
            if profile == 43 && grant == 1 {
                // Retain the actual first grant's live child credit BEFORE
                // the later source can bind its cumulative rollup baseline.
                charged.push((88, Int(a1)));
            }
            if profile == 46 {
                charged.push((37, Int(6)));
            }
            if direct {
                charged.push((PENDING, Int(0)));
            }
            if rollup {
                charged.push((92, Int(1)));
            }
            let charge_index = m.actions.len();
            act(
                m,
                &format!("ChargeAdaptiveSourceRefinementProfile{profile}Grant{grant}"),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(COMMITTED, grant - 1),
                    eq(0, if direct { 9 } else { 6 }),
                    eq(
                        READ0,
                        if direct {
                            3
                        } else if measured {
                            2
                        } else {
                            0
                        },
                    ),
                    eq(REPORTED, aggregate),
                    eq(37, 0),
                    eq(SEEN, 1),
                    eq(14, 1),
                    eq(15, 1),
                    eq(18, 1),
                    eq(19, 1),
                    eq(20, 1),
                    eq(27, 0),
                    eq(28, 0),
                    if direct {
                        all([eq(PENDING, 1), eq(40, 1)])
                    } else {
                        Bool(true)
                    },
                    Var(5).eq(Var(6)),
                ]),
                &charged,
            );
            if direct {
                pending_actions.push(charge_index);
            }
            c_reserve = c_reserve.max(c);
            if latency_known {
                latency_reserve = latency_reserve.max(selected_latency);
            }
            let (target, cap) = limits(c_reserve, latency_reserve, slots);
            let (next, kind) = if aggregate > target && !(control == 48 && profile == 45) {
                ((run / 2).max(25_000).min(cap), 1)
            } else if measured && latency_known && aggregate <= target * 3 / 4 {
                ((run + 10_000).min(cap), 2)
            } else {
                (run.min(cap), 3)
            };
            let feedback = m.actions.len();
            act(
                m,
                &format!("ApplyLiveProvenanceAdaptiveFeedbackProfile{profile}Grant{grant}"),
                all([
                    eq(LAW, profile),
                    eq(49, grant),
                    eq(COMMITTED, grant),
                    eq(FEEDBACKS, grant - 1),
                    eq(CLOCK_KNOWN, 1),
                    eq(CURRENT_A, aggregate),
                    eq(CURRENT_C, c),
                    eq(CURRENT_LATENCY, selected_latency),
                    eq(USED_RUN, run),
                ]),
                &[
                    (C_RESERVE, Int(c_reserve)),
                    (LATENCY_RESERVE, Int(latency_reserve)),
                    (TARGET, Int(target)),
                    (CAP, Int(cap)),
                    (RUN, Int(next)),
                    (NEXT_AFTER_FEEDBACK, Int(next)),
                    (FEEDBACK_KIND, Int(kind)),
                    (FEEDBACKS, Int(grant)),
                ],
            );
            feedback_actions.push(feedback);
            run = next;
        }
    }
    if control == 49 {
        // One omitted CURRENT-grant commit barrier. Prior first-grant credit
        // is retained; no wait4, CPU charge, proof or trust is fabricated.
        act(
            m,
            "BrokenCommitsPendingSecondGrantBeforeOwnWait4",
            all([
                eq(LAW, 44),
                eq(49, 2),
                eq(PENDING, 1),
                eq(COMMITTED, 1),
                eq(40, 0),
                eq(READ0, 0),
                eq(ORIGIN0, 2),
                eq(BASIS, 0),
                eq(QNS, 0),
                eq(CHARGED, 725_000),
                eq(33, 0),
                eq(34, 0),
            ]),
            &[(COMMITTED, Int(2))],
        );
    }
    safe(
        m,
        "SecondDirectGrantCommitNeedsOwnWait4AndPriorCredit",
        eq(LAW, 44).negate().or(eq(COMMITTED, 2).negate()).or(all([
            eq(49, 2),
            eq(PENDING, 0),
            eq(40, 1),
            eq(READ0, 3),
            eq(ORIGIN0, 2),
            eq(CHARGED, 825_000),
            eq(BURNED, 825_000),
        ])),
    );
    // WINDOW is still the SAME exact max(W,A)+C law; the existing measured
    // scheduler profiles use A+C. No bound is weakened by this refinement.
    for invariant in &mut m.invariants {
        if invariant.name == "BasisQuantumIncludesAllActiveControllerCpu" {
            let window = all([
                eq(LAW, 45).or(eq(LAW, 46)),
                eq(BASIS, 0),
                Var(WALL).lt(Var(REPORTED)).negate(),
                Var(QNS).eq(Var(WALL).plus(Var(CURRENT_C))),
            ]);
            invariant.condition = invariant.condition.clone().or(window);
        }
    }
    let new_profile = Int(41).lt(Var(LAW));
    let applied = all([
        new_profile.clone(),
        Int(0).lt(Var(FEEDBACKS)),
        Var(FEEDBACKS).eq(Var(49)),
    ]);
    let one = all([
        eq(LIVE_MASK, 1),
        eq(STOP_MATCH, 1),
        eq(LATENCY_KNOWN, 1),
        eq(ORIGIN0, 1),
        eq(14, 1),
        Var(CURRENT_LATENCY).eq(Var(DELAY0)),
    ]);
    let two = all([
        eq(LIVE_MASK, 3),
        eq(STOP_MATCH, 1),
        eq(LATENCY_KNOWN, 1),
        eq(ORIGIN0, 1),
        eq(ORIGIN1, 1),
        eq(READ0, 2),
        eq(READ1, 2),
        Var(CURRENT_LATENCY).lt(Var(DELAY0)).negate(),
        Var(CURRENT_LATENCY).lt(Var(DELAY1)).negate(),
        Var(CURRENT_LATENCY)
            .eq(Var(DELAY0))
            .or(Var(CURRENT_LATENCY).eq(Var(DELAY1))),
    ]);
    safe(
        m,
        "AdaptiveLatencyRetainsMaximumValidLiveAck",
        applied
            .clone()
            .negate()
            .or(all([eq(LIVE_MASK, 0), eq(LATENCY_KNOWN, 0)])
                .or(one)
                .or(two)),
    );
    safe(
        m,
        "AdaptiveGrowthNeedsMeasuredLiveSuccessfulStopAck",
        applied
            .clone()
            .negate()
            .or(eq(FEEDBACK_KIND, 2).negate())
            .or(all([
                eq(BASIS, 1),
                eq(LATENCY_KNOWN, 1),
                eq(STOP_MATCH, 1),
                Int(0).lt(Var(LIVE_MASK)),
                eq(CLOCK_KNOWN, 1),
            ])),
    );
    safe(
        m,
        "WindowBasisAboveNewTargetHalvesPriorRun",
        eq(LAW, 45).negate().or(applied.negate()).or(all([
            eq(BASIS, 0),
            eq(CURRENT_A, 525_000),
            eq(CURRENT_C, 300_000),
            eq(TARGET, 500_000),
            eq(USED_RUN, 350_000),
            eq(RUN, 175_000),
            eq(FEEDBACK_KIND, 1),
        ])),
    );
    progress(
        m,
        "ClosedSourceRefinedGrantGetsLocalAdaptiveFeedback",
        all([
            new_profile,
            Var(FEEDBACKS).lt(Var(COMMITTED)),
            eq(CLOCK_KNOWN, 1),
        ]),
        Var(FEEDBACKS).eq(Var(COMMITTED)),
        feedback_actions,
    );
    progress(
        m,
        "ExitedOnlyAccountingWaitsForOwnWait4WithoutInventingLiveAck",
        all([eq(LAW, 44), eq(PENDING, 1), eq(40, 1), eq(27, 0), eq(28, 0)]),
        eq(PENDING, 0),
        pending_actions,
    );
}

/// Decision d93377961365654: a reviewed finite program/build premise is distinct
/// from runtime observations. The approved graph assumes that premise; the
/// adversarial graph permits unaccounted work after its explicit withdrawal.
/// Assumption-drop controls deliberately forge the approval/premise relation,
/// retain identical censuses, and must reach actual unaccounted-work trust.
/// Decision65875 adds disjoint normal-unavailable/private-conditional modes.
/// The private projection follows one representative original helper ticket;
/// complete sequential command execution remains a concrete process witness.
/// Private measurement never sets runtime approval, lesson trust or a lifetime
/// theorem, even with exact test-host integrity and bound review assumptions.
pub fn qualification(control: u8) -> Model {
    qualification_observer(control, false)
}

fn qualification_observer(control: u8, full_observer: bool) -> Model {
    let suffix = [
        "",
        "Adversarial",
        "NoBasis",
        "SourceBuild",
        "CommandConfig",
        "LoadedClosure",
        "OsMatch",
        "Stale",
        "Principal",
        "FeatureOnly",
        "DroppedSourcePremise",
        "HiddenProcess",
        "HiddenThread",
        "GroupEscape",
        "Aba",
        "EarlyRelease",
        "ApprovalAsProof",
        "MissingReceipt",
        "EarlyTrust",
        "PrivateIntegrity",
        "PrivateSevenSources",
        "PrivateIdentity",
        "PrivatePins",
        "PrivateTestPremise",
        "PrivateDuplicate",
        "PrivateRestart",
        "PrivateRuntimeApproval",
        "PrivateLifetime",
        "PrivateToRuntime",
        "PrivateNormalBirth",
        "PrivateSelfReference",
        "PrivateSettlement",
        "PrivateDebt",
        "PrivateAba",
        "InnerCoverage",
        "InnerStale",
        "InnerArch",
        "InnerBudget",
        "InnerCandidate",
        "InnerProjection",
        "InnerSyntax",
        "InnerRevalidation",
    ];
    let name = format!("DreamReadQualification{}", suffix[control as usize]);
    let mut m = base(
        &name,
        control >= 2,
        &[
            (
                "phaseAbsentPreparedAdmittedBornRunningClosedChargedRetiredReaped",
                8,
                0,
            ),
            ("requestPrincipal", 1, 0),
            ("qualificationPrincipal", 1, 0),
            ("currentEpochA1B2A3", 2, 0),
            ("qualificationEpoch", 2, 0),
            ("basisApproved", 1, 0),
            ("sourceBuildMatches", 1, 0),
            ("commandConfigMatches", 1, 0),
            ("loadedClosureMatches", 1, 0),
            ("osMatches", 1, 0),
            ("trustedFiniteSourcePremiseHolds", 1, 0),
            ("featureStringsPresent", 1, 0),
            ("qualificationCase", 10, 0),
            ("prepared", 1, 0),
            ("consumedAttempts", 2, 0),
            ("originalNativeEffect", 1, 0),
            ("originalEffectOwned", 1, 0),
            ("reservedLogicalSlots", 2, 0),
            ("closedRoster", 1, 0),
            ("windowCoverage", 1, 0),
            ("unpaidDebtMs", 27, 0),
            ("runtimeCoverage", 1, 0),
            ("actualExit", 1, 0),
            ("actualReap", 1, 0),
            ("controllerRetired", 1, 0),
            ("outputTrusted", 1, 0),
            ("refused", 1, 0),
            ("qualificationRevoked", 1, 0),
            ("unaccountedNoneProcessThreadEscape", 3, 0),
            ("sampledLogicalCount", 2, 0),
            ("birthHadCompleteCurrentQualification", 1, 0),
            ("existingFullPublicationProof", 1, 0),
            ("receiptPresent", 1, 0),
            ("ordinaryCost", 3, 0),
            ("modeRuntimePremiseNormalProbePrivateProbe", 2, 0),
            ("privatePretestCase", 9, 0),
            ("hostExpectedInputIntegrity", 1, 0),
            ("exactSevenAuthenticatedSourcePins", 1, 0),
            (
                "allActualRootArtifactBuildConfigClosureOsProbePinsMatch",
                1,
                0,
            ),
            ("boundSourceReviewTestPremise", 1, 0),
            ("pretestOnlyNoOwnOrFutureResultHash", 1, 0),
            ("currentRunNonce", 1, 0),
            ("capturedOriginalRunNonce", 1, 0),
            ("originalPrivateOperationClaimed", 1, 0),
            ("privateResultRuntimeApproved", 1, 0),
            ("privateResultWholeLifetimeTheorem", 1, 0),
            (
                "privateOutcomeNoneCompletedExpectedRejectRefusedQuarantined",
                4,
                0,
            ),
            ("representativePrivateOriginalEffectDispatches", 2, 0),
            ("privateObservedProofNotReachedValidInvalid", 2, 0),
            ("privateUnpaidOrdinaryAdmission", 1, 0),
            ("innerNoneReservedParsedCoveredBoundRejected", 5, 0),
            ("selectedCandidate", 1, 0),
            ("parsedCandidate", 1, 0),
            ("parsedInnerEpochA1B2A3", 2, 0),
            ("innerReviewAndClosureCoverage", 1, 0),
            ("innerArchCanonicalRawAliasUnsupported", 2, 0),
            ("innerProjectionAndCurrentnessMatch", 1, 0),
            ("logicalMetadataUnknownReservedOver128KiB", 2, 0),
            ("innerCanonicalTypedShape", 1, 0),
            ("immutableInnerProfile", 6, 0),
            ("innerPostSettlementRevalidated", 1, 0),
        ],
    );
    const PH: usize = 0;
    const RP: usize = 1;
    const QP: usize = 2;
    const INC: usize = 3;
    const QINC: usize = 4;
    const BASIS: usize = 5;
    const SOURCE: usize = 6;
    const CONFIG: usize = 7;
    const LOADED: usize = 8;
    const OS: usize = 9;
    const PREMISE: usize = 10;
    const FEATURES: usize = 11;
    const CASE: usize = 12;
    const PREP: usize = 13;
    const ATT: usize = 14;
    const EFFECT: usize = 15;
    const OWN: usize = 16;
    const SLOTS: usize = 17;
    const ROSTER: usize = 18;
    const WINDOW: usize = 19;
    const DEBT: usize = 20;
    const COVER: usize = 21;
    const EXIT: usize = 22;
    const REAP: usize = 23;
    const RETIRED: usize = 24;
    const TRUST: usize = 25;
    const REFUSED: usize = 26;
    const REVOKED: usize = 27;
    const HIDDEN: usize = 28;
    const CENSUS: usize = 29;
    const BIRTH: usize = 30;
    const PROOF: usize = 31;
    const PRESENT: usize = 32;
    const COST: usize = 33;
    const MODE: usize = 34;
    const INPUT: usize = 35;
    const HASH: usize = 36;
    const SEVEN: usize = 37;
    const PINS: usize = 38;
    const REVIEW: usize = 39;
    const PRETEST: usize = 40;
    const NONCE: usize = 41;
    const BOUND_NONCE: usize = 42;
    const CLAIM: usize = 43;
    const APPROVED: usize = 44;
    const THEOREM: usize = 45;
    const OUTCOME: usize = 46;
    const EFFECTS: usize = 47;
    const PRIVATE_PROOF: usize = 48;
    const UNPAID_ADMISSION: usize = 49;
    const INNER: usize = 50;
    const CANDIDATE: usize = 51;
    const PARSED_CANDIDATE: usize = 52;
    const INNER_EPOCH: usize = 53;
    const INNER_COVERAGE: usize = 54;
    const ARCH: usize = 55;
    const INNER_CURRENT: usize = 56;
    const METADATA: usize = 57;
    const SYNTAX: usize = 58;
    const INNER_CASE: usize = 59;
    const REVALIDATED: usize = 60;
    let epoch = Var(INC).eq(Var(QINC));
    let principal = Var(RP).eq(Var(QP));
    let complete = all([
        eq(BASIS, 1),
        eq(SOURCE, 1),
        eq(CONFIG, 1),
        eq(LOADED, 1),
        eq(OS, 1),
        eq(PRESENT, 1),
        eq(REVOKED, 0),
        epoch.clone(),
        principal.clone(),
    ]);
    let mut guards = vec![eq(REVOKED, 0)];
    for (field, mode) in [
        (BASIS, 2),
        (SOURCE, 3),
        (CONFIG, 4),
        (LOADED, 5),
        (OS, 6),
        (PRESENT, 17),
    ] {
        if control != mode {
            guards.push(eq(field, 1));
        }
    }
    if control != 7 {
        guards.push(if control == 14 {
            epoch.clone().or(all([eq(INC, 2), eq(QINC, 0)]))
        } else {
            epoch.clone()
        });
    }
    if control != 8 {
        guards.push(principal.clone());
    }
    let admitted = if control == 9 {
        all([eq(FEATURES, 1), eq(REVOKED, 0)])
    } else {
        all(guards)
    };
    act(
        &mut m,
        "ChooseOtherRequestPrincipalBeforePreparation",
        all([eq(PREP, 0), eq(RP, 0)]),
        &[(RP, Int(1))],
    );
    for principal in 0..=1 {
        for case in 0..=10 {
            let unsupported = matches!(case, 1 | 8 | 9);
            let dropped = (10..=13).contains(&control) && case == 9;
            act(
                &mut m,
                &format!("PrepareQualificationPrincipal{principal}Case{case}"),
                all([eq(PREP, 0), eq(PH, 0), eq(RP, principal)]),
                &[
                    (PH, Int(1)),
                    (PREP, Int(1)),
                    (CASE, Int(case)),
                    (QP, Int(if case == 7 { 1 - principal } else { principal })),
                    (QINC, Int(if case == 6 { 1 } else { 0 })),
                    (BASIS, Int(i64::from(!unsupported || dropped))),
                    (SOURCE, Int(i64::from(!matches!(case, 2 | 8)))),
                    (CONFIG, Int(i64::from(!matches!(case, 3 | 8)))),
                    (LOADED, Int(i64::from(!matches!(case, 4 | 8)))),
                    (OS, Int(i64::from(!matches!(case, 5 | 8)))),
                    (PRESENT, Int(i64::from(case != 10))),
                    (PREMISE, Int(i64::from(case != 9))),
                    (FEATURES, Int(1)),
                ],
            );
        }
    }
    act(
        &mut m,
        "AdmitExactCurrentPreparedQualificationBeforeReservation",
        all([
            eq(PH, 1),
            eq(REFUSED, 0),
            eq(EFFECT, 0),
            eq(ATT, 0),
            eq(DEBT, 0),
            admitted.clone(),
        ]),
        &[(PH, Int(2)), (ATT, Int(2)), (SLOTS, Int(2))],
    );
    for valid in 0..=1 {
        act(
            &mut m,
            &format!("CaptureActualStoppedBirthAfterFreshQualificationValid{valid}"),
            all([
                eq(PH, 2),
                eq(EFFECT, 0),
                eq(ATT, 2),
                eq(REFUSED, 0),
                admitted.clone(),
                if valid == 1 {
                    complete.clone()
                } else {
                    complete.clone().negate()
                },
            ]),
            &[
                (PH, Int(3)),
                (EFFECT, Int(1)),
                (OWN, Int(1)),
                (BIRTH, Int(valid)),
            ],
        );
    }
    act(
        &mut m,
        "ForkExecWithinOriginalTwoSlotAndConsumedReservation",
        all([eq(PH, 3), eq(REFUSED, 0), complete.clone()]),
        &[(PH, Int(4)), (WINDOW, Int(1))],
    );
    // This is an explicit program-trace extension, not a runtime detector. It
    // is excluded ONLY by the declared source premise in the approved world.
    if control == 1 || (10..=13).contains(&control) {
        for kind in 1..=3 {
            let selected = control == 1 || control == 10 || control == kind + 10;
            act(
                &mut m,
                &format!("UnobservedTransientProcessThreadEscape{kind}"),
                all([
                    eq(EFFECT, 1),
                    eq(EXIT, 0),
                    eq(RETIRED, 0),
                    eq(PREMISE, 0),
                    eq(HIDDEN, 0),
                    if selected { Bool(true) } else { Bool(false) },
                ]),
                &[(HIDDEN, Int(i64::from(kind)))],
            );
        }
    }
    for valid in 0..=1 {
        act(
            &mut m,
            &format!("ObserveSameCensusWithoutAssumingHiddenEventDetectionValid{valid}"),
            all([
                eq(EFFECT, 1),
                eq(EXIT, 0),
                eq(RETIRED, 0),
                if valid == 1 {
                    complete.clone().and(eq(REFUSED, 0))
                } else {
                    complete.clone().and(eq(REFUSED, 0)).negate()
                },
            ]),
            &[(CENSUS, Int(2)), (COVER, Int(valid))],
        );
    }
    act(
        &mut m,
        "CloseRosterRequiresCurrentQualificationAndRuntimeChecks",
        all([
            eq(PH, 4),
            eq(CENSUS, 2),
            eq(COVER, 1),
            eq(REFUSED, 0),
            complete.clone(),
        ]),
        &[(PH, Int(5)), (ROSTER, Int(1))],
    );
    for cost in [1, 3] {
        for healthy in 0..=1 {
            let current = complete.clone().and(eq(COVER, 1)).and(eq(REFUSED, 0));
            act(
                &mut m,
                &format!("ChargeOrdinaryObservedCost{cost}Current{healthy}"),
                all([
                    eq(PH, 5),
                    eq(COST, 0),
                    eq(DEBT, 0),
                    if healthy == 1 {
                        current
                    } else {
                        current.negate()
                    },
                ]),
                &[
                    (PH, Int(6)),
                    (COST, Int(cost)),
                    (DEBT, Int(cost * 9)),
                    (COVER, Int(i64::from(cost == 1 && healthy == 1))),
                    (REFUSED, Int(i64::from(cost != 1 || healthy != 1))),
                ],
            );
        }
    }
    let rest = m.actions.len();
    for debt in 1..=27 {
        act(
            &mut m,
            &format!("PayObservedDebtWithoutPremiseOrEpochReset{debt}"),
            eq(DEBT, debt),
            &[(DEBT, Int(debt - 1))],
        );
    }
    act(
        &mut m,
        "ObserveExistingIndependentPublicationProof",
        all([eq(PH, 6), eq(COVER, 1), eq(DEBT, 0), complete.clone()]),
        &[(PROOF, Int(1))],
    );
    act(
        &mut m,
        "RefuseMissingStaleMismatchedQualificationWithoutFallback",
        all([eq(PREP, 1), eq(REFUSED, 0), complete.clone().negate()]),
        &[(REFUSED, Int(1)), (COVER, Int(0)), (TRUST, Int(0))],
    );
    // Source/build boundedness is no longer assumed after this explicit host
    // withdrawal. Later hidden events do not cause these flags themselves.
    act(
        &mut m,
        "HostWithdrawsTrustedProgramPremiseAndApproval",
        all([eq(PREP, 1), eq(BASIS, 1)]),
        &[
            (BASIS, Int(0)),
            (PREMISE, Int(0)),
            (REVOKED, Int(1)),
            (REFUSED, Int(1)),
            (COVER, Int(0)),
            (TRUST, Int(0)),
        ],
    );
    for field in [SOURCE, CONFIG, LOADED, OS] {
        act(
            &mut m,
            &format!("InvalidateExactSourceConfigLibraryOsField{field}"),
            all([eq(PREP, 1), eq(BASIS, 1), eq(field, 1)]),
            &[
                (field, Int(0)),
                (BASIS, Int(0)),
                (PREMISE, Int(0)),
                (REVOKED, Int(1)),
                (REFUSED, Int(1)),
                (COVER, Int(0)),
                (TRUST, Int(0)),
            ],
        );
    }
    for epoch in 0..2 {
        act(
            &mut m,
            &format!("InvalidateRestartA1B2A3From{epoch}"),
            all([eq(PREP, 1), eq(INC, epoch)]),
            &[
                (INC, Int(epoch + 1)),
                (BASIS, Int(0)),
                (REVOKED, Int(1)),
                (REFUSED, Int(1)),
                (COVER, Int(0)),
                (TRUST, Int(0)),
            ],
        );
    }
    if control == 14 {
        act(
            &mut m,
            "BrokenRestoreA1ApprovalUnderA3Label",
            all([eq(INC, 2), eq(QINC, 0), eq(PREP, 1), eq(EFFECT, 0)]),
            &[(BASIS, Int(1)), (REVOKED, Int(0)), (REFUSED, Int(0))],
        );
    }
    let retire = m.actions.len();
    act(
        &mut m,
        "RetireOriginalOwnedGroupWithoutRefundingAttempts",
        all([eq(EFFECT, 1), eq(OWN, 1), eq(RETIRED, 0)]),
        &[(RETIRED, Int(1)), (PH, Int(7))],
    );
    act(
        &mut m,
        "ObserveActualOriginalGroupExit",
        all([eq(EFFECT, 1), eq(RETIRED, 1), eq(EXIT, 0), eq(OWN, 1)]),
        &[(EXIT, Int(1))],
    );
    act(
        &mut m,
        "ObserveActualReapBeforeOwnershipRelease",
        all([eq(EXIT, 1), eq(RETIRED, 1), eq(REAP, 0), eq(OWN, 1)]),
        &[(REAP, Int(1)), (PH, Int(8))],
    );
    act(
        &mut m,
        "ReleaseOnlyAfterActualReapAndPaidDebt",
        all([eq(OWN, 1), eq(RETIRED, 1), eq(REAP, 1), eq(DEBT, 0)]),
        &[(OWN, Int(0))],
    );
    if control == 15 {
        act(
            &mut m,
            "BrokenQualificationErrorDropsOriginalLiveEffect",
            all([eq(EFFECT, 1), eq(OWN, 1), eq(EXIT, 0), eq(REFUSED, 1)]),
            &[(OWN, Int(0))],
        );
    }
    act(
        &mut m,
        "TrustNeedsIndependentProofCurrentBasisCoverageAndActualSettlement",
        all([
            complete.clone(),
            eq(COVER, 1),
            eq(ROSTER, 1),
            eq(WINDOW, 1),
            eq(DEBT, 0),
            eq(REFUSED, 0),
            eq(BIRTH, 1),
            eq(EFFECT, 1),
            if control == 16 {
                Bool(true)
            } else {
                eq(PROOF, 1)
            },
            if control == 18 {
                Bool(true)
            } else {
                all([eq(REAP, 1), eq(OWN, 0), eq(RETIRED, 1)])
            },
        ]),
        &[(TRUST, Int(1))],
    );
    let unavailable = m.actions.len();
    act(
        &mut m,
        "RetainOwnedUnavailableDispositionWithoutOsFairness",
        all([eq(REFUSED, 1), eq(OWN, 1), eq(RETIRED, 0)]),
        &[(RETIRED, Int(1)), (PH, Int(7))],
    );
    safe(
        &mut m,
        "NativeBirthNeedsCompleteCurrentPreparedQualification",
        eq(EFFECT, 0).or(eq(BIRTH, 1)),
    );
    safe(
        &mut m,
        "OriginalEffectKeepsTwoConsumedReservations",
        eq(EFFECT, 0).or(all([eq(ATT, 2), eq(SLOTS, 2)])),
    );
    safe(
        &mut m,
        "QualificationCannotMintIndependentPublicationProof",
        eq(TRUST, 0).or(eq(PROOF, 1)),
    );
    safe(
        &mut m,
        "TrustedOutputHasNoUnaccountedProgramWork",
        eq(TRUST, 0).or(eq(HIDDEN, 0)),
    );
    safe(
        &mut m,
        "TrustKeepsCurrentQualificationAndRuntimeCoverage",
        eq(TRUST, 0).or(all([
            complete,
            eq(COVER, 1),
            eq(ROSTER, 1),
            eq(WINDOW, 1),
            eq(REFUSED, 0),
        ])),
    );
    safe(
        &mut m,
        "TrustRequiresActualSettlementAndPaidDebt",
        eq(TRUST, 0).or(all([eq(REAP, 1), eq(OWN, 0), eq(RETIRED, 1), eq(DEBT, 0)])),
    );
    safe(
        &mut m,
        "QualificationFailureNeverReleasesUnreapedOriginalEffect",
        eq(EFFECT, 0).or(eq(OWN, 1)).or(all([
            eq(EXIT, 1),
            eq(REAP, 1),
            eq(RETIRED, 1),
            eq(DEBT, 0),
        ])),
    );
    progress(
        &mut m,
        "RejectedLiveEffectHasRetirementDisposition",
        eq(REFUSED, 1).and(eq(EFFECT, 1)),
        eq(RETIRED, 1).or(eq(REAP, 1)),
        vec![retire, unavailable, rest],
    );
    // Preserve65654's hypothetical approved runtime world separately from
    // the normal unavailable probe and the65875 private conditional test. No
    // private input/result can take a transition in the runtime authority graph.
    for action in &mut m.actions {
        action.guard = action.guard.clone().and(eq(MODE, 0));
    }
    for invariant in &mut m.invariants {
        invariant.condition = eq(MODE, 0).negate().or(invariant.condition.clone());
    }
    for response in &mut m.responses {
        response.request = eq(MODE, 0).and(response.request.clone());
    }
    act(
        &mut m,
        "ChooseNormalUnavailableProbe",
        all([eq(MODE, 0), eq(PH, 0), eq(PREP, 0)]),
        &[(MODE, Int(1)), (PREP, Int(1)), (PH, Int(1))],
    );
    act(
        &mut m,
        "NormalProbeRefusesWithoutBirthOrRuntimeApproval",
        all([eq(MODE, 1), eq(OUTCOME, 0)]),
        &[(REFUSED, Int(1)), (OUTCOME, Int(3))],
    );
    if control == 29 {
        act(
            &mut m,
            "BrokenNormalProbeFeatureAssertionsCreateNativeEffect",
            all([eq(MODE, 1), eq(EFFECT, 0)]),
            &[(EFFECT, Int(1)), (EFFECTS, Int(1)), (OWN, Int(1))],
        );
    }
    let original = all([
        Var(RP).eq(Var(QP)),
        Var(NONCE).eq(Var(BOUND_NONCE)),
        Var(INC).eq(Var(QINC)),
    ]);
    let outer_current = all([
        eq(HASH, 1),
        eq(SEVEN, 1),
        eq(PINS, 1),
        eq(REVIEW, 1),
        eq(PRETEST, 1),
        eq(REVOKED, 0),
        original.clone(),
    ]);
    let admitted_outer = all([
        if control == 19 {
            Bool(true)
        } else {
            eq(HASH, 1)
        },
        if control == 20 {
            Bool(true)
        } else {
            eq(SEVEN, 1)
        },
        if control == 22 {
            Bool(true)
        } else {
            eq(PINS, 1)
        },
        if control == 23 {
            Bool(true)
        } else {
            eq(REVIEW, 1)
        },
        if control == 30 {
            Bool(true)
        } else {
            eq(PRETEST, 1)
        },
        eq(REVOKED, 0),
        Var(RP).eq(Var(QP)),
        if control == 21 {
            Bool(true)
        } else {
            Var(NONCE).eq(Var(BOUND_NONCE))
        },
        if control == 33 {
            Var(INC).eq(Var(QINC)).or(all([eq(INC, 2), eq(QINC, 0)]))
        } else {
            Var(INC).eq(Var(QINC))
        },
    ]);
    let same_inner = all([
        Var(CANDIDATE).eq(Var(PARSED_CANDIDATE)),
        Var(INC).eq(Var(INNER_EPOCH)),
    ]);
    let inner_complete = all([
        eq(INNER, 4),
        eq(INNER_COVERAGE, 1),
        eq(ARCH, 0),
        eq(INNER_CURRENT, 1),
        eq(METADATA, 1),
        eq(SYNTAX, 1),
        same_inner.clone(),
    ]);
    let current = outer_current.clone().and(inner_complete.clone());
    let admitted_private = admitted_outer.clone().and(inner_complete.clone());
    for input in 0..=9 {
        act(
            &mut m,
            &format!("PreparePrivateImmutablePretestTuple{input}"),
            all([eq(MODE, 0), eq(PH, 0), eq(PREP, 0)]),
            &[
                (MODE, Int(2)),
                (PH, Int(1)),
                (PREP, Int(1)),
                (INPUT, Int(input)),
                (QP, Var(RP)),
                (HASH, Int(i64::from(input != 2))),
                (SEVEN, Int(i64::from(input != 3))),
                (PINS, Int(i64::from(!matches!(input, 5 | 8 | 9)))),
                (REVIEW, Int(i64::from(input != 6))),
                (PRETEST, Int(i64::from(input != 7))),
                (NONCE, Int(i64::from(input == 4))),
                (BOUND_NONCE, Int(0)),
                (QINC, Var(INC)),
                // This is a bound test premise, not Root-approved finite lifetime
                // authority. Matching test hashes never sets BASIS or APPROVED.
                (BASIS, Int(0)),
                (PREMISE, Int(0)),
                (FEATURES, Int(1)),
            ],
        );
    }
    // ICD98a3023e: one shared basis, six identical pins, five independently
    // hashed projections. Profile facts abstract typed/custody/hash checks;
    // review coverage is distinct from source reachability judgement.
    for candidate in 0..=1 {
        for profile in 0..=6 {
            act(
                &mut m,
                &format!("ReservePrivateInnerCandidate{candidate}Profile{profile}"),
                all([
                    eq(MODE, 2),
                    eq(PH, 1),
                    eq(INNER, 0),
                    eq(REFUSED, 0),
                    admitted_outer.clone(),
                ]),
                &[
                    (INNER, Int(1)),
                    (CANDIDATE, Int(candidate)),
                    (PARSED_CANDIDATE, Int(candidate)),
                    (INNER_EPOCH, Var(INC)),
                    (INNER_CASE, Int(profile)),
                    (SYNTAX, Int(i64::from(profile != 1))),
                    (
                        ARCH,
                        Int(if profile == 3 {
                            1
                        } else if profile == 4 {
                            2
                        } else {
                            0
                        }),
                    ),
                    (INNER_CURRENT, Int(i64::from(profile != 5))),
                    (METADATA, Int(if profile == 6 { 2 } else { 1 })),
                ],
            );
        }
    }
    act(
        &mut m,
        "ParseReservedCanonicalTypedInnerBasisExactlyOnce",
        all([
            eq(MODE, 2),
            eq(INNER, 1),
            eq(REFUSED, 0),
            if control == 40 {
                Bool(true)
            } else {
                eq(SYNTAX, 1)
            },
            if control == 37 {
                Bool(true)
            } else {
                eq(METADATA, 1)
            },
        ]),
        &[(INNER, Int(2))],
    );
    act(
        &mut m,
        "CheckEightFindingSevenSourceSixteenCommandFivePhaseClosureCoverage",
        all([
            eq(MODE, 2),
            eq(INNER, 2),
            eq(SYNTAX, 1),
            eq(INNER_CASE, 2).negate(),
            eq(REFUSED, 0),
        ]),
        &[(INNER_COVERAGE, Int(1)), (INNER, Int(3))],
    );
    act(
        &mut m,
        "BindExactCoveredCanonicalCurrentInnerTupleBeforeClaim",
        all([
            eq(MODE, 2),
            eq(REFUSED, 0),
            if control == 34 {
                eq(INNER, 2).or(eq(INNER, 3))
            } else {
                eq(INNER, 3)
            },
            if control == 34 {
                Bool(true)
            } else {
                eq(INNER_COVERAGE, 1)
            },
            if control == 36 {
                Bool(true)
            } else {
                eq(ARCH, 0)
            },
            if control == 39 {
                Bool(true)
            } else {
                eq(INNER_CURRENT, 1)
            },
            if control == 35 {
                Bool(true)
            } else {
                same_inner.clone()
            },
            eq(METADATA, 1),
            eq(SYNTAX, 1),
        ]),
        &[(INNER, Int(4)), (REVALIDATED, Int(0))],
    );
    for epoch in 0..2 {
        act(
            &mut m,
            &format!("ChangeInnerEpochBetweenParseAndBindFrom{epoch}"),
            all([
                eq(MODE, 2),
                eq(CLAIM, 0),
                eq(INNER, 2).or(eq(INNER, 3)),
                eq(INC, epoch),
                eq(REFUSED, 0),
            ]),
            &[(INC, Int(epoch + 1))],
        );
    }
    act(
        &mut m,
        "ChangeSelectedCandidateInvalidatesBoundTupleBeforeClaim",
        all([
            eq(MODE, 2),
            eq(CLAIM, 0),
            eq(INNER, 4),
            eq(CANDIDATE, 0),
            eq(REFUSED, 0),
        ]),
        &[
            (CANDIDATE, Int(1)),
            (INNER, Int(5)),
            (INNER_CURRENT, Int(0)),
            (REVALIDATED, Int(0)),
            (REFUSED, Int(1)),
        ],
    );
    if control == 38 {
        act(
            &mut m,
            "BrokenCandidateSwitchKeepsOtherCandidatesInnerBinding",
            all([
                eq(MODE, 2),
                eq(CLAIM, 0),
                eq(INNER, 4),
                eq(CANDIDATE, 0),
                eq(REFUSED, 0),
            ]),
            &[(CANDIDATE, Int(1))],
        );
    }
    let inner_reject = m.actions.len();
    let incompatible = eq(SYNTAX, 0)
        .or(eq(METADATA, 1).negate())
        .or(eq(ARCH, 0).negate())
        .or(eq(INNER_CURRENT, 0))
        .or(eq(INNER_CASE, 2))
        .or(same_inner.clone().negate());
    act(
        &mut m,
        "RejectIncompleteOrStaleInnerWithoutNativeBirthOrSilentRebind",
        all([
            eq(MODE, 2),
            eq(CLAIM, 0),
            eq(INNER, 0).negate(),
            eq(REFUSED, 0),
            incompatible.clone(),
        ]),
        &[
            (INNER, Int(5)),
            (REFUSED, Int(1)),
            (OUTCOME, Int(3)),
            (REVALIDATED, Int(0)),
        ],
    );
    act(
        &mut m,
        "ClaimOriginalPrivateRunNonceInputAndIncarnationBeforeBirth",
        all([
            eq(MODE, 2),
            eq(PH, 1),
            eq(CLAIM, 0),
            eq(REFUSED, 0),
            admitted_private.clone(),
        ]),
        &[
            (PH, Int(2)),
            (CLAIM, Int(1)),
            (ATT, Int(2)),
            (SLOTS, Int(2)),
        ],
    );
    for valid in 0..=1 {
        act(
            &mut m,
            &format!("CreatePrivateStoppedOriginalWithActualBindingValid{valid}"),
            all([
                eq(MODE, 2),
                eq(PH, 2),
                eq(CLAIM, 1),
                eq(EFFECTS, 0),
                eq(REFUSED, 0),
                eq(ATT, 2),
                admitted_private.clone(),
                if valid == 1 {
                    current.clone()
                } else {
                    current.clone().negate()
                },
            ]),
            &[
                (PH, Int(3)),
                (EFFECT, Int(1)),
                (EFFECTS, Int(1)),
                (OWN, Int(1)),
                (BIRTH, Int(valid)),
                (DEBT, Int(9)),
                (COST, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "DuplicateExactOriginalPrivateOperationRetainsEffectAndDebt",
        all([eq(MODE, 2), eq(CLAIM, 1), current.clone()]),
        &[],
    );
    if control == 24 {
        act(
            &mut m,
            "BrokenDuplicatePrivateOperationCreatesSecondOriginalEffect",
            all([
                eq(MODE, 2),
                eq(CLAIM, 1),
                eq(EFFECTS, 1),
                eq(OWN, 1),
                current.clone(),
            ]),
            &[(EFFECTS, Int(2))],
        );
    }
    act(
        &mut m,
        "PrivateForkExecUsesExistingCanonicalPaidTwoSlotGrant",
        all([
            eq(MODE, 2),
            eq(PH, 3),
            eq(EFFECTS, 1),
            eq(OWN, 1),
            eq(DEBT, 0),
            eq(REFUSED, 0),
            current.clone(),
        ]),
        &[(PH, Int(4)), (WINDOW, Int(1))],
    );
    act(
        &mut m,
        "ObserveActualPrivateCohortClosureAndInclusiveCharge",
        all([eq(MODE, 2), eq(PH, 4), eq(REFUSED, 0), current.clone()]),
        &[
            (PH, Int(5)),
            (ROSTER, Int(1)),
            (COVER, Int(1)),
            (DEBT, Int(9)),
        ],
    );
    act(
        &mut m,
        "PrivateUnknownCoverageRetainsOriginalWindowsAndRejects",
        all([eq(MODE, 2), eq(PH, 4), eq(REFUSED, 0)]),
        &[
            (PH, Int(5)),
            (COVER, Int(0)),
            (REFUSED, Int(1)),
            (DEBT, Int(9)),
        ],
    );
    let private_rest = m.actions.len();
    for debt in 1..=27 {
        act(
            &mut m,
            &format!("PrivatePayFreshObservedDebtAfterWork{debt}"),
            all([eq(MODE, 2), eq(DEBT, debt), eq(PH, 4).negate()]),
            &[(DEBT, Int(debt - 1))],
        );
    }
    for proof in 1..=2 {
        act(
            &mut m,
            &format!("ObserveRealPrivatePublicProofOutcome{proof}"),
            all([
                eq(MODE, 2),
                eq(PH, 5),
                eq(PRIVATE_PROOF, 0),
                eq(COVER, 1),
                eq(DEBT, 0),
                eq(REFUSED, 0),
                current.clone(),
                if proof == 1 {
                    eq(INPUT, 0)
                } else {
                    eq(INPUT, 1)
                },
            ]),
            &[(PRIVATE_PROOF, Int(proof)), (DEBT, Int(9))],
        );
    }
    // The same original retained effect survives invalidation, foreign nonce,
    // restart and A-B-A. No source mutation mints a replacement or erases debt.
    act(
        &mut m,
        "InvalidateChangedOrSameVersionPrivateArtifactPins",
        all([eq(MODE, 2), eq(CLAIM, 1), eq(OUTCOME, 0), eq(PINS, 1)]),
        &[
            (PINS, Int(0)),
            (REFUSED, Int(1)),
            (COVER, Int(0)),
            (INNER, Int(5)),
            (INNER_CURRENT, Int(0)),
            (REVALIDATED, Int(0)),
        ],
    );
    act(
        &mut m,
        "ForeignPrivateNonceCannotClaimExistingEffect",
        all([eq(MODE, 2), eq(CLAIM, 1), eq(OUTCOME, 0), eq(NONCE, 0)]),
        &[
            (NONCE, Int(1)),
            (REFUSED, Int(1)),
            (COVER, Int(0)),
            (INNER, Int(5)),
            (INNER_CURRENT, Int(0)),
            (REVALIDATED, Int(0)),
        ],
    );
    for epoch in 0..2 {
        act(
            &mut m,
            &format!("RestartPrivateOriginalIncarnation{epoch}"),
            all([eq(MODE, 2), eq(CLAIM, 1), eq(OUTCOME, 0), eq(INC, epoch)]),
            &[
                (INC, Int(epoch + 1)),
                (REFUSED, Int(1)),
                (COVER, Int(0)),
                (INNER, Int(5)),
                (INNER_CURRENT, Int(0)),
                (REVALIDATED, Int(0)),
            ],
        );
    }
    if control == 25 {
        act(
            &mut m,
            "BrokenPrivateRestartDropsOwnedUnreapedOriginal",
            all([
                eq(MODE, 2),
                eq(EFFECTS, 1),
                eq(OWN, 1),
                eq(REAP, 0),
                eq(REFUSED, 1),
            ]),
            &[(OWN, Int(0)), (DEBT, Int(0))],
        );
    }
    if control == 33 {
        act(
            &mut m,
            "BrokenPrivateA3ReusesOriginalA1InputClaim",
            all([
                eq(MODE, 2),
                eq(PH, 2),
                eq(CLAIM, 1),
                eq(INC, 2),
                eq(QINC, 0),
                eq(EFFECTS, 0),
            ]),
            &[
                (REFUSED, Int(0)),
                (PINS, Int(1)),
                (INNER, Int(4)),
                (INNER_EPOCH, Var(INC)),
                (INNER_CURRENT, Int(1)),
            ],
        );
    }
    let private_retire = m.actions.len();
    act(
        &mut m,
        "RetirePrivateControllerWithOriginalCustodyBeforeRealReap",
        all([eq(MODE, 2), eq(EFFECTS, 1), eq(OWN, 1), eq(RETIRED, 0)]),
        &[(RETIRED, Int(1)), (PH, Int(6))],
    );
    act(
        &mut m,
        "ObserveActualPrivateOriginalGroupExit",
        all([eq(MODE, 2), eq(RETIRED, 1), eq(EXIT, 0), eq(OWN, 1)]),
        &[(EXIT, Int(1))],
    );
    act(
        &mut m,
        "ActuallyReapPrivateOriginalAndRetiredMembers",
        all([
            eq(MODE, 2),
            eq(RETIRED, 1),
            eq(EXIT, 1),
            eq(REAP, 0),
            eq(OWN, 1),
        ]),
        &[(REAP, Int(1))],
    );
    act(
        &mut m,
        "ReleasePrivateOriginalOnlyAfterActualReapAndPaidDebt",
        all([
            eq(MODE, 2),
            eq(RETIRED, 1),
            eq(REAP, 1),
            eq(OWN, 1),
            eq(DEBT, 0),
        ]),
        &[(OWN, Int(0)), (PH, Int(7))],
    );
    let settled = all([
        eq(RETIRED, 1),
        eq(EXIT, 1),
        eq(REAP, 1),
        eq(OWN, 0),
        eq(DEBT, 0),
    ]);
    act(
        &mut m,
        "RevalidateExactBoundInnerAfterActualPrivateSettlement",
        all([
            eq(MODE, 2),
            eq(PH, 7),
            eq(REFUSED, 0),
            settled.clone(),
            current.clone(),
        ]),
        &[(REVALIDATED, Int(1))],
    );
    for outcome in 1..=2 {
        act(
            &mut m,
            &format!("PublishConditionalPrivateMeasuredOutcome{outcome}NeverRuntimeApproval"),
            all([
                eq(MODE, 2),
                eq(OUTCOME, 0),
                eq(COVER, 1),
                eq(ROSTER, 1),
                eq(PRIVATE_PROOF, outcome),
                eq(REFUSED, 0),
                current.clone(),
                if control == 31 || control == 41 {
                    Bool(true)
                } else {
                    eq(REVALIDATED, 1)
                },
                if control == 31 {
                    Bool(true)
                } else {
                    settled.clone()
                },
            ]),
            &[(OUTCOME, Int(outcome))],
        );
    }
    act(
        &mut m,
        "RejectInvalidPrivatePretestBeforeAnyNativeBirth",
        all([
            eq(MODE, 2),
            eq(EFFECTS, 0),
            eq(OUTCOME, 0),
            outer_current.clone().negate(),
        ]),
        &[(REFUSED, Int(1)), (OUTCOME, Int(3))],
    );
    let private_quarantine = m.actions.len();
    act(
        &mut m,
        "RetainOwnedPrivateUnavailableQuarantine",
        all([eq(MODE, 2), eq(REFUSED, 1), eq(OWN, 1), eq(OUTCOME, 0)]),
        &[(OUTCOME, Int(4))],
    );
    act(
        &mut m,
        "SettledPrivateFailureStillCannotBecomeAvailable",
        all([eq(MODE, 2), eq(REFUSED, 1), eq(OWN, 0), eq(OUTCOME, 0)]),
        &[(OUTCOME, Int(3))],
    );
    act(
        &mut m,
        "RuntimeConsumptionRefusesPrivateResultKindWithoutBirth",
        all([eq(MODE, 2), eq(OUTCOME, 1).or(eq(OUTCOME, 2))]),
        &[],
    );
    if control == 26 {
        act(
            &mut m,
            "BrokenPrivateMeasurementMintsRuntimeApproval",
            all([eq(MODE, 2), eq(OUTCOME, 1)]),
            &[(APPROVED, Int(1))],
        );
    }
    if control == 27 {
        act(
            &mut m,
            "BrokenPrivateCensusBecomesWholeLifetimeTheorem",
            all([eq(MODE, 2), eq(COVER, 1)]),
            &[(THEOREM, Int(1))],
        );
    }
    if control == 28 {
        act(
            &mut m,
            "BrokenRuntimeAcceptsPrivateConditionalResultAsLesson",
            all([eq(MODE, 2), eq(OUTCOME, 1)]),
            &[(TRUST, Int(1))],
        );
    }
    if control == 32 {
        act(
            &mut m,
            "BrokenPrivateMainOrContUsesUnpaidEarlierDebt",
            all([eq(MODE, 2), eq(PH, 5), eq(DEBT, 0).negate()]),
            &[(UNPAID_ADMISSION, Int(1))],
        );
    }
    safe(
        &mut m,
        "NormalProbeRemainsUnavailableBeforeNativeBirth",
        eq(MODE, 1).negate().or(eq(EFFECT, 0)),
    );
    safe(
        &mut m,
        "PrivateNativeBirthRequiresExactCurrentBoundPretest",
        eq(MODE, 2).negate().or(eq(EFFECTS, 0)).or(eq(BIRTH, 1)),
    );
    safe(
        &mut m,
        "PrivateOriginalOperationNeverCreatesDuplicateEffect",
        eq(MODE, 2).negate().or(Var(EFFECTS).lt(Int(2))),
    );
    safe(
        &mut m,
        "PrivateOriginalEffectKeepsConsumedReservations",
        eq(MODE, 2)
            .negate()
            .or(eq(EFFECTS, 0))
            .or(all([eq(ATT, 2), eq(SLOTS, 2)])),
    );
    safe(
        &mut m,
        "PrivateOriginalEffectKeepsCustodyThroughRestart",
        eq(MODE, 2)
            .negate()
            .or(eq(EFFECTS, 0))
            .or(eq(OWN, 1))
            .or(settled.clone()),
    );
    safe(
        &mut m,
        "PrivateConditionalResultNeverGrantsRuntimeAuthority",
        eq(MODE, 2)
            .negate()
            .or(all([eq(APPROVED, 0), eq(TRUST, 0), eq(BASIS, 0)])),
    );
    safe(
        &mut m,
        "PrivateObservationNeverProvesWholeCommandLifetime",
        eq(MODE, 2).negate().or(eq(THEOREM, 0)),
    );
    safe(
        &mut m,
        "PrivateSuccessfulOutcomeRequiresActualPaidProofSettlement",
        eq(MODE, 2)
            .negate()
            .or(eq(OUTCOME, 1).or(eq(OUTCOME, 2)).negate())
            .or(all([
                settled,
                eq(COVER, 1),
                eq(ROSTER, 1),
                current,
                eq(REFUSED, 0),
                eq(OUTCOME, 1)
                    .and(eq(PRIVATE_PROOF, 1))
                    .or(eq(OUTCOME, 2).and(eq(PRIVATE_PROOF, 2))),
            ])),
    );
    safe(
        &mut m,
        "PrivateOrdinaryOrContinueAdmissionNeverOverlapsUnpaidDebt",
        eq(MODE, 2).negate().or(eq(UNPAID_ADMISSION, 0)),
    );
    safe(
        &mut m,
        "BoundPrivateInnerRequiresCompleteReviewAndClosureCoverage",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 4).negate())
            .or(eq(INNER_COVERAGE, 1)),
    );
    safe(
        &mut m,
        "BoundPrivateInnerKeepsParsedCandidateAndExactEpoch",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 4).negate())
            .or(same_inner),
    );
    safe(
        &mut m,
        "BoundPrivateInnerUsesCanonicalAarch64Identity",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 4).negate())
            .or(eq(ARCH, 0)),
    );
    safe(
        &mut m,
        "ParsedPrivateInnerHasReservedCombined128KiBMetadata",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 2).or(eq(INNER, 3)).or(eq(INNER, 4)).negate())
            .or(eq(METADATA, 1)),
    );
    safe(
        &mut m,
        "BoundPrivateInnerHasExactProjectionPinsAndCurrentness",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 4).negate())
            .or(eq(INNER_CURRENT, 1)),
    );
    safe(
        &mut m,
        "ParsedPrivateInnerRequiresCanonicalTypedShape",
        eq(MODE, 2)
            .negate()
            .or(eq(INNER, 2).or(eq(INNER, 3)).or(eq(INNER, 4)).negate())
            .or(eq(SYNTAX, 1)),
    );
    safe(
        &mut m,
        "PrivateSuccessfulOutcomeHasPostSettlementInnerCurrentnessFence",
        eq(MODE, 2)
            .negate()
            .or(eq(OUTCOME, 1).or(eq(OUTCOME, 2)).negate())
            .or(eq(REVALIDATED, 1)),
    );
    progress(
        &mut m,
        "KnownIncompleteParsedInnerEventuallyRejects",
        all([
            eq(MODE, 2),
            eq(CLAIM, 0),
            eq(INNER, 2).or(eq(INNER, 3)),
            incompatible,
        ]),
        eq(REFUSED, 1),
        vec![inner_reject],
    );
    progress(
        &mut m,
        "PrivateFailedOwnedEffectHasRetirementOrQuarantineDisposition",
        all([eq(MODE, 2), eq(REFUSED, 1), eq(OWN, 1)]),
        eq(RETIRED, 1).or(eq(OUTCOME, 4)),
        vec![private_retire, private_quarantine, private_rest],
    );
    if control >= 2 {
        // These are every safety predicate violated in the original control's
        // complete reachable graph. Keep every declared safety/liveness clause;
        // the equivalence test also retains every original labelled transition
        // and named violation with its complete prefix/cycle trace.
        let relevant: &[&str] = match control {
            2..=9 | 14 | 17 => &["NativeBirthNeedsCompleteCurrentPreparedQualification"],
            10..=13 => &["TrustedOutputHasNoUnaccountedProgramWork"],
            15 => &["QualificationFailureNeverReleasesUnreapedOriginalEffect"],
            16 => &["QualificationCannotMintIndependentPublicationProof"],
            18 => &["TrustRequiresActualSettlementAndPaidDebt"],
            19..=23 | 30 | 33 => &["PrivateNativeBirthRequiresExactCurrentBoundPretest"],
            24 => &["PrivateOriginalOperationNeverCreatesDuplicateEffect"],
            25 => &["PrivateOriginalEffectKeepsCustodyThroughRestart"],
            26 | 28 => &["PrivateConditionalResultNeverGrantsRuntimeAuthority"],
            27 => &["PrivateObservationNeverProvesWholeCommandLifetime"],
            29 => &["NormalProbeRemainsUnavailableBeforeNativeBirth"],
            31 => &[
                "PrivateSuccessfulOutcomeRequiresActualPaidProofSettlement",
                "PrivateSuccessfulOutcomeHasPostSettlementInnerCurrentnessFence",
            ],
            32 => &["PrivateOrdinaryOrContinueAdmissionNeverOverlapsUnpaidDebt"],
            34 => &["BoundPrivateInnerRequiresCompleteReviewAndClosureCoverage"],
            35 | 38 => &["BoundPrivateInnerKeepsParsedCandidateAndExactEpoch"],
            36 => &["BoundPrivateInnerUsesCanonicalAarch64Identity"],
            37 => &["ParsedPrivateInnerHasReservedCombined128KiBMetadata"],
            39 => &["BoundPrivateInnerHasExactProjectionPinsAndCurrentness"],
            40 => &["ParsedPrivateInnerRequiresCanonicalTypedShape"],
            41 => &["PrivateSuccessfulOutcomeHasPostSettlementInnerCurrentnessFence"],
            _ => unreachable!(),
        };
        assert!(
            relevant
                .iter()
                .all(|name| m.invariants.iter().any(|i| i.name == *name))
        );
        let valid = all(m
            .invariants
            .iter()
            .filter(|i| full_observer || relevant.contains(&i.name.as_str()))
            .map(|i| i.condition.clone()));
        for action in &mut m.actions {
            let guard = action.guard.clone().and(valid.clone());
            // Evaluate the selected state-only predicates as one Boolean value
            // instead of expanding their ORs as duplicate Next alternatives.
            action.guard = if full_observer {
                guard
            } else {
                guard.eq(Bool(true))
            };
        }
        let failed = valid.negate();
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            if full_observer {
                failed
            } else {
                failed.eq(Bool(true))
            },
            &[],
        );
    }
    m
}

/// B observes one real Git-created signature inode at GS1 and GS2. It never
/// proves which inode GPG opened. The reviewed no-other-writer/source premise,
/// exact native identity and bounded measured IO remain independent witnesses.
pub fn git_sig_temp(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadGitSigTempFixturePath",
        2 => "DreamReadGitSigTempNlink",
        3 => "DreamReadGitSigTempGs2",
        4 => "DreamReadGitSigTempExclusivity",
        5 => "DreamReadGitSigTempWrongCase",
        6 => "DreamReadGitSigTempCleanupIdentity",
        7 => "DreamReadGitSigTempAba",
        8 => "DreamReadGitSigTempForeignUid",
        9 => "DreamReadGitSigTempScratchReplacement",
        _ => "DreamReadGitSigTemp",
    };
    const ST: usize = 0;
    const TMP: usize = 1;
    const ARGV: usize = 2;
    const GS: usize = 3;
    const LEADER: usize = 4;
    const CASE: usize = 5;
    const PROFILE: usize = 6;
    const ORIGINAL: usize = 7;
    const NAMED: usize = 8;
    const HELD: usize = 9;
    const CONTENT: usize = 10;
    const NLINK: usize = 11;
    const ENTRIES: usize = 12;
    const OWN: usize = 13;
    const EPOCH: usize = 14;
    const BOUND_CASE: usize = 15;
    const ABSENCE: usize = 16;
    const FOREIGN_UNLINK: usize = 17;
    const PREMISE: usize = 18;
    const CREATED: usize = 19;
    const GS1_NLINK: usize = 20;
    const GS1_ENTRIES: usize = 21;
    const GS1_UID: usize = 22;
    const ACK: usize = 23;
    const RESULT: usize = 24;
    const HELD_NLINK: usize = 25;
    const BOUND_INODE: usize = 26;
    const SCRATCH_CURRENT: usize = 27;
    let mut m = base(
        name,
        control != 0,
        &[
            (
                "phaseChooseConfiguredScratchChildBoundReapedKilledRefusedDoneQuarantine",
                9,
                0,
            ),
            ("tmpAbsentGit1Git2PlantedSwapped", 4, 0),
            ("argvNoneTemp1Temp2FixtureOther", 4, 0),
            ("gsNoneBound1Bound2Settled", 3, 0),
            ("leaderNoneRunningReapedKilledAndActuallyReaped", 3, 0),
            ("selectedCaseValidCorrupt", 1, 0),
            ("inputProfile", 14, 0),
            ("originalGitInode", 2, 0),
            ("currentNamedInode", 2, 0),
            ("retainedFdInode", 2, 0),
            ("heldBytesValidCorrupt", 2, 0),
            ("observedNlink", 2, 0),
            ("scratchEntries", 2, 0),
            ("signatureFdOwned", 1, 0),
            ("sameNameEpochA1B2A3", 2, 0),
            ("boundBytesValidCorrupt", 2, 0),
            ("postReapAbsenceAndHeldIdentityRehashObserved", 1, 0),
            ("foreignInodeUnlinked", 1, 0),
            ("reviewedSourceNoOtherWriterPremise", 1, 1),
            ("nativeEffectCreated", 1, 0),
            ("capturedGs1Nlink", 2, 0),
            ("capturedGs1Entries", 2, 0),
            ("capturedGs1UidMatches", 1, 0),
            ("verifierIncarnationAcknowledged", 1, 0),
            (
                "resultPendingBCheckpointOkSignatureUnexpectedCoverageUnknownReviewUnboundQuarantineCancelled",
                6,
                0,
            ),
            ("heldInodeNlink", 2, 0),
            ("boundOriginalInode", 2, 0),
            ("retainedScratchStillNamesOriginalDirectory", 1, 1),
        ],
    );
    // Profiles are immutable facts, not guesses discovered by directory scans:
    // clean, fixture argv, hardlink, extra entry, wrong case bytes, preplanted,
    // symlink, foreign uid, early deletion, unavailable/racing observation,
    // other argv, wrong mode/dev/size, invalid scratch/umask, absent source basis,
    // and legacy fixture-token Commands (refused before any birth).
    for case in 0..=1 {
        for profile in 0..=14 {
            act(
                &mut m,
                &format!("SelectCase{case}Profile{profile}"),
                eq(ST, 0),
                &[
                    (ST, Int(1)),
                    (CASE, Int(case)),
                    (PROFILE, Int(profile)),
                    (TMP, Int(if profile == 5 { 3 } else { 0 })),
                    (PREMISE, Int(if profile == 13 { 0 } else { 1 })),
                ],
            );
        }
    }
    let prebirth_ok = all([
        eq(TMP, 0),
        eq(PROFILE, 12).negate(),
        eq(PROFILE, 14).negate(),
        eq(PREMISE, 1),
    ]);
    act(
        &mut m,
        "RetainFreshPrivateEmptyScratchBeforeAnyBirth",
        all([eq(ST, 1), prebirth_ok.clone()]),
        &[(ST, Int(2))],
    );
    act(
        &mut m,
        "RefusePlantedUnsafeOrUnboundScratchBeforeBirth",
        all([eq(ST, 1), prebirth_ok.negate()]),
        &[(ST, Int(7)), (RESULT, Int(4))],
    );
    for inode in 1..=2 {
        for case in 0..=1 {
            for profile in 0..=14 {
                act(
                    &mut m,
                    &format!("GitCreatesItsOwnTempInode{inode}Case{case}Profile{profile}"),
                    all([eq(ST, 2), eq(CASE, case), eq(PROFILE, profile)]),
                    &[
                        (ST, Int(3)),
                        (TMP, Int(if profile == 8 { 0 } else { inode })),
                        (
                            ARGV,
                            Int(if profile == 1 {
                                3
                            } else if profile == 10 {
                                4
                            } else {
                                inode
                            }),
                        ),
                        (LEADER, Int(1)),
                        (ORIGINAL, Int(inode)),
                        (NAMED, Int(if profile == 8 { 0 } else { inode })),
                        (CONTENT, Int(if profile == 4 { 2 - case } else { case + 1 })),
                        (NLINK, Int(if profile == 2 { 2 } else { 1 })),
                        (
                            ENTRIES,
                            Int(if profile == 3 {
                                2
                            } else if profile == 8 {
                                0
                            } else {
                                1
                            }),
                        ),
                        (CREATED, Int(1)),
                    ],
                );
            }
        }
    }
    let expected_hash = all([eq(CASE, 0), eq(CONTENT, 1)]).or(all([eq(CASE, 1), eq(CONTENT, 2)]));
    let exact_argv = Var(ARGV).eq(Var(ORIGINAL));
    let argv_guard = if control == 1 {
        exact_argv.clone().or(eq(ARGV, 3))
    } else {
        exact_argv.clone()
    };
    let nlink_guard = if control == 2 {
        Bool(true)
    } else {
        eq(NLINK, 1)
    };
    let entries_guard = if control == 4 {
        Bool(true)
    } else {
        eq(ENTRIES, 1)
    };
    let hash_guard = if control == 5 {
        expected_hash.clone().negate()
    } else {
        expected_hash.clone()
    };
    let uid_guard = if control == 8 {
        Bool(true)
    } else {
        eq(PROFILE, 7).negate()
    };
    let gs1_ok = all([
        argv_guard,
        nlink_guard,
        entries_guard,
        hash_guard,
        uid_guard,
        Var(NAMED).eq(Var(ORIGINAL)),
        eq(NAMED, 0).negate(),
        eq(PROFILE, 6).negate(),
        eq(PROFILE, 9).negate(),
        eq(PROFILE, 11).negate(),
    ]);
    for uid in 0..=1 {
        act(
            &mut m,
            &format!("Gs1BracketCurrentUnreapedIncarnationAndBindExactTempBeforeAckUid{uid}"),
            all([
                eq(ST, 3),
                eq(LEADER, 1),
                gs1_ok.clone(),
                if uid == 0 {
                    eq(PROFILE, 7)
                } else {
                    eq(PROFILE, 7).negate()
                },
            ]),
            &[
                (ST, Int(4)),
                (GS, Var(ORIGINAL)),
                (HELD, Var(ORIGINAL)),
                (OWN, Int(1)),
                (BOUND_CASE, Var(CONTENT)),
                (BOUND_INODE, Var(ORIGINAL)),
                (GS1_NLINK, Var(NLINK)),
                (HELD_NLINK, Var(NLINK)),
                (GS1_ENTRIES, Var(ENTRIES)),
                (GS1_UID, Int(uid)),
                (ACK, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "Gs1RefusesUnexpectedFileOrArgvWithoutAck",
        all([eq(ST, 3), gs1_ok.clone().negate(), eq(PROFILE, 9).negate()]),
        &[(ST, Int(7)), (RESULT, Int(2))],
    );
    act(
        &mut m,
        "Gs1UnavailableOrRacingObservationRetainsActualEffect",
        all([eq(ST, 3), eq(PROFILE, 9)]),
        &[(ST, Int(7)), (RESULT, Int(3))],
    );
    for inode in 1..=2 {
        act(
            &mut m,
            &format!("SameNameSwapAwayFromHeldInode{inode}"),
            all([eq(ST, 4), eq(HELD, inode), eq(EPOCH, 0)]),
            &[
                (TMP, Int(4)),
                (NAMED, Int(3 - inode)),
                (HELD_NLINK, Int(0)),
                (EPOCH, Int(1)),
            ],
        );
        act(
            &mut m,
            &format!("AbaRestoresSameLexicalNameButNotHeldInode{inode}"),
            all([eq(ST, 4), eq(HELD, inode), eq(EPOCH, 1)]),
            &[(EPOCH, Int(2))],
        );
        act(
            &mut m,
            &format!("NormalGitDeletesItsOwnInode{inode}AndLeaderIsActuallyReaped"),
            all([eq(ST, 4), eq(NAMED, inode), eq(HELD, inode)]),
            &[
                (ST, Int(5)),
                (LEADER, Int(2)),
                (TMP, Int(0)),
                (NAMED, Int(0)),
                (HELD_NLINK, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "LeaderActuallyReapedWithUnexpectedNamedLeftover",
        eq(ST, 4),
        &[(ST, Int(5)), (LEADER, Int(2))],
    );
    act(
        &mut m,
        "HeldBytesChangeAfterGs1",
        all([eq(ST, 4), expected_hash.clone()]),
        &[(CONTENT, Int(3).minus(Var(CONTENT)))],
    );
    act(
        &mut m,
        "ReplaceNamedScratchWhileRetainedOldDirectoryStillLooksValid",
        all([eq(ST, 4).or(eq(ST, 5)), eq(SCRATCH_CURRENT, 1)]),
        &[(SCRATCH_CURRENT, Int(0))],
    );
    let scratch_guard = if control == 9 {
        Bool(true)
    } else {
        eq(SCRATCH_CURRENT, 1)
    };
    let gs2_ok = all([
        scratch_guard,
        eq(NAMED, 0),
        eq(HELD_NLINK, 0),
        Var(HELD).eq(Var(BOUND_INODE)),
        Var(BOUND_INODE).eq(Var(ORIGINAL)),
        Var(CONTENT).eq(Var(BOUND_CASE)),
        expected_hash.clone(),
    ]);
    act(
        &mut m,
        "Gs2PostReapAbsenceHeldIdentityZeroLinksAndRehashThenClose",
        all([eq(ST, 5), gs2_ok.clone()]),
        &[
            (ST, Int(8)),
            (GS, Int(3)),
            (ABSENCE, Int(1)),
            (OWN, Int(0)),
            (HELD, Int(0)),
            (RESULT, Int(1)),
        ],
    );
    act(
        &mut m,
        "Gs2UnexpectedPreservesRetainedInodeInOwnedQuarantine",
        all([eq(ST, 5), gs2_ok.negate()]),
        &[(ST, Int(9)), (RESULT, Int(2))],
    );
    act(
        &mut m,
        "Gs2UnavailableOrRacingObservationRetainsOriginalHeldCustody",
        eq(ST, 5),
        &[(ST, Int(9)), (RESULT, Int(3))],
    );
    if control == 3 {
        act(
            &mut m,
            "BrokenAcceptWithoutGs2Observation",
            eq(ST, 5),
            &[
                (ST, Int(8)),
                (GS, Int(3)),
                (OWN, Int(0)),
                (HELD, Int(0)),
                (RESULT, Int(1)),
            ],
        );
    }
    if control == 7 {
        act(
            &mut m,
            "BrokenAbaNameReplayAcceptsStaleGs1Binding",
            all([eq(ST, 5), eq(EPOCH, 2)]),
            &[
                (ST, Int(8)),
                (GS, Int(3)),
                (OWN, Int(0)),
                (HELD, Int(0)),
                (RESULT, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "ExactOriginalKillAndActualReapRetainsGsAndFileCustody",
        eq(ST, 3).or(eq(ST, 4)).or(all([eq(ST, 7), eq(CREATED, 1)])),
        &[(ST, Int(6)), (LEADER, Int(3))],
    );
    let cleanup = m.actions.len();
    act(
        &mut m,
        "KilledLeftoverUnlinkOnlyOnHeldOriginalDevInoMatch",
        all([
            eq(ST, 6),
            eq(GS, 1).or(eq(GS, 2)),
            Var(NAMED).eq(Var(BOUND_INODE)),
            eq(SCRATCH_CURRENT, 1),
        ]),
        &[
            (ST, Int(8)),
            (TMP, Int(0)),
            (NAMED, Int(0)),
            (HELD_NLINK, Int(0)),
            (GS, Int(3)),
            (OWN, Int(0)),
            (HELD, Int(0)),
            (RESULT, Int(6)),
        ],
    );
    let quarantine = m.actions.len();
    act(
        &mut m,
        "KilledUnknownOrSwappedNameStaysOwnedQuarantineWithoutUnlink",
        all([
            eq(ST, 6),
            eq(GS, 0)
                .or(Var(NAMED).eq(Var(BOUND_INODE)).negate())
                .or(eq(SCRATCH_CURRENT, 0)),
        ]),
        &[(ST, Int(9)), (RESULT, Int(5))],
    );
    if control == 6 {
        act(
            &mut m,
            "BrokenKillCleanupUnlinksSameNameForeignInode",
            all([
                eq(ST, 6),
                eq(GS, 1).or(eq(GS, 2)),
                Var(NAMED).eq(Var(BOUND_INODE)).negate(),
                eq(NAMED, 0).negate(),
            ]),
            &[
                (ST, Int(8)),
                (TMP, Int(0)),
                (NAMED, Int(0)),
                (FOREIGN_UNLINK, Int(1)),
                (GS, Int(3)),
                (OWN, Int(0)),
                (HELD, Int(0)),
                (RESULT, Int(6)),
            ],
        );
    }
    let bound = eq(GS, 0).negate();
    safe(
        &mut m,
        "SettlementRequiresCurrentNamedScratchBeforeReleaseOrUnlink",
        eq(RESULT, 1)
            .or(eq(RESULT, 6))
            .negate()
            .or(eq(SCRATCH_CURRENT, 1)),
    );
    safe(
        &mut m,
        "Gs1BindsActualGeneratedGitArgvNeverFixturePath",
        bound.clone().negate().or(exact_argv),
    );
    safe(
        &mut m,
        "Gs1RequiresExactlyOneLink",
        bound.clone().negate().or(eq(GS1_NLINK, 1)),
    );
    safe(
        &mut m,
        "Gs1RequiresExclusiveScratchEntry",
        bound.clone().negate().or(eq(GS1_ENTRIES, 1)),
    );
    safe(
        &mut m,
        "Gs1BindsSelectedCaseRefNotOppositeCase",
        bound
            .clone()
            .negate()
            .or(all([eq(CASE, 0), eq(BOUND_CASE, 1)]).or(all([eq(CASE, 1), eq(BOUND_CASE, 2)]))),
    );
    safe(
        &mut m,
        "Gs1RequiresSameUid",
        bound.clone().negate().or(eq(GS1_UID, 1)),
    );
    safe(
        &mut m,
        "Gs1RetainsOriginalGitInodeIdentity",
        bound.negate().or(Var(BOUND_INODE).eq(Var(ORIGINAL))),
    );
    safe(
        &mut m,
        "BCheckpointOkRequiresActualGs2AbsenceAtReap",
        eq(RESULT, 1).negate().or(all([
            eq(GS, 3),
            eq(LEADER, 2),
            eq(ABSENCE, 1),
            eq(NAMED, 0),
            eq(TMP, 0),
            eq(OWN, 0),
            expected_hash,
        ])),
    );
    safe(
        &mut m,
        "KillCleanupNeverUnlinksForeignInodeAfterSameNameAba",
        eq(FOREIGN_UNLINK, 0),
    );
    safe(
        &mut m,
        "UnsafeOrUnboundScratchNeverCreatesNativeEffect",
        eq(RESULT, 4).negate().or(eq(CREATED, 0)),
    );
    safe(
        &mut m,
        "NativeBirthRequiresExplicitReviewedSourcePremise",
        eq(CREATED, 0).or(eq(PREMISE, 1)),
    );
    safe(
        &mut m,
        "VerifierAckRequiresGs1BoundCurrentUnreapedObservation",
        eq(ACK, 0).or(all([eq(GS, 0).negate(), eq(PROFILE, 9).negate()])),
    );
    progress(
        &mut m,
        "KnownKilledOriginalEventuallyCleanedOrOwnedQuarantine",
        eq(ST, 6),
        eq(ST, 8).or(eq(ST, 9)),
        vec![cleanup, quarantine],
    );
    if control != 0 {
        let valid = all(m.invariants.iter().map(|i| i.condition.clone()));
        for a in &mut m.actions {
            a.guard = a.guard.clone().and(valid.clone());
        }
        act(
            &mut m,
            "ControlObserverStopsAtFirstFailedBoundary",
            valid.negate(),
            &[],
        );
    }
    m
}

#[cfg(test)]
mod helper_observer_lowering_export {
    // Both standalone formal test crates reuse this one canonical exporter.
    include!("export.rs");
}

#[cfg(test)]
mod helper_observer_lowering_equivalence {
    use super::*;

    #[test]
    fn original_and_lowered_keep_all_states_labelled_edges_named_verdicts_and_traces() {
        let output = std::env::var_os("DREAM_HELPER_OBSERVER_OUT").map(std::path::PathBuf::from);
        let mut full_manifest = String::new();
        let mut lowered_manifest = String::new();
        if let Some(root) = &output {
            for variant in ["original", "lowered"] {
                std::fs::create_dir_all(root.join(variant)).unwrap();
            }
        }
        for control in 0..=22 {
            let full_model = helper_observer(control, true);
            let lowered_model = helper(control);
            assert_eq!(full_model.name, lowered_model.name);
            assert_eq!(format!("{:?}", full_model.variables), format!("{:?}", lowered_model.variables));
            assert_eq!(format!("{:?}", full_model.invariants), format!("{:?}", lowered_model.invariants));
            assert_eq!(format!("{:?}", full_model.responses), format!("{:?}", lowered_model.responses));
            assert_eq!(full_model.weak_fair, lowered_model.weak_fair);
            assert_eq!(full_model.check_deadlock, lowered_model.check_deadlock);
            assert_eq!(full_model.actions.len(), lowered_model.actions.len());
            for (full, lowered) in full_model.actions.iter().zip(&lowered_model.actions) {
                assert_eq!(full.name, lowered.name);
                assert_eq!(format!("{:?}", full.updates), format!("{:?}", lowered.updates));
            }
            let full = full_model.check(100_000).unwrap();
            let lowered = lowered_model.check(100_000).unwrap();
            assert_eq!(full.states, lowered.states, "control{control} complete states");
            assert_eq!(full.edges.len(), lowered.edges.len());
            for (full, lowered) in full.edges.iter().zip(&lowered.edges) {
                assert_eq!(
                    full.iter().map(|edge| (edge.action, edge.to)).collect::<Vec<_>>(),
                    lowered.iter().map(|edge| (edge.action, edge.to)).collect::<Vec<_>>(),
                    "control{control} complete labelled edges"
                );
            }
            assert_eq!(
                full.violations.iter().map(|v| (&v.property, &v.prefix, &v.cycle)).collect::<Vec<_>>(),
                lowered.violations.iter().map(|v| (&v.property, &v.prefix, &v.cycle)).collect::<Vec<_>>(),
                "control{control} named verdicts and complete prefix/cycle traces"
            );
            println!(
                "helper_observer_equivalence control={control} model={} states={} edges={} named_verdicts={:?}",
                full_model.name,
                full.states.len(),
                full.edges.iter().map(Vec::len).sum::<usize>(),
                full.violations.iter().map(|v| &v.property).collect::<Vec<_>>()
            );
            drop(full);
            drop(lowered);
            if let Some(root) = &output {
                // Use the canonical exporter unchanged. Its complete native
                // graphs preserve the original observer as independent evidence;
                // the lowered directory is the narrow pinned-TLC input.
                let expected = if control == 0 { "pass" } else { "safety" };
                helper_observer_lowering_export::export(
                    full_model,
                    expected,
                    &root.join("original"),
                    &mut full_manifest,
                ).unwrap();
                helper_observer_lowering_export::export(
                    lowered_model,
                    expected,
                    &root.join("lowered"),
                    &mut lowered_manifest,
                ).unwrap();
            }
        }
        if let Some(root) = output {
            std::fs::write(root.join("original/manifest.tsv"), full_manifest).unwrap();
            std::fs::write(root.join("lowered/manifest.tsv"), lowered_manifest).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member_window_failure_rows(
        model: &Model,
        states: &[Vec<i64>],
        property: &str,
        prefix: &[usize],
        cycle: &[usize],
        outgoing: impl Fn(usize) -> Vec<(usize, String)>,
    ) {
        let rows = prefix
            .iter()
            .rev()
            .take(2)
            .chain(cycle.iter().take(2))
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for row in rows {
            let values = model
                .variables
                .iter()
                .zip(&states[row])
                .map(|(v, n)| format!("{}={n}", v.name))
                .collect::<Vec<_>>();
            let enabled = outgoing(row).into_iter().take(12).collect::<Vec<_>>();
            let incoming = prefix.windows(2).find(|pair| pair[1] == row).map(|pair| {
                outgoing(pair[0])
                    .into_iter()
                    .filter(|(to, _)| *to == row)
                    .take(4)
                    .collect::<Vec<_>>()
            });
            eprintln!(
                "{} {property} state{row} incoming={incoming:?} enabled={enabled:?} values={values:?}",
                model.name
            );
        }
    }

    #[test]
    fn closure_codec_currentness_has_all_causal_controls() {
        let good = closure_codec(0).check(100_000).unwrap();
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(good.states.iter().any(|s| s[0] == 5 && s[1] == 0));
        assert!(good.states.iter().any(|s| s[0] == 4
            && s[1] == 4
            && s[2] == 0
            && s[3] == 1
            && s[4] == 2));
        for control in 1..=8 {
            let broken = closure_codec(control).check(100_000).unwrap();
            assert!(
                !broken.violations.is_empty(),
                "missing causal control {control}"
            );
            println!(
                "control={control} states={} edges={} verdict=safety",
                broken.states.len(),
                broken.edges.iter().map(Vec::len).sum::<usize>()
            );
        }
        println!("good states={}", good.states.len());
    }

    #[test]
    fn dream_read_git_signature_temp_binds_real_inode_and_both_selected_cases() {
        let good = git_sig_temp(0).check(100_000).unwrap();
        println!("HarnessDreamReadGitSigTemp: {} states", good.states.len());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        for case in 0..=1 {
            for inode in 1..=2 {
                assert!(
                    good.states
                        .iter()
                        .any(|s| s[5] == case && s[7] == inode && s[24] == 1 && s[16] == 1),
                    "selected case{case} real inode{inode} reaches only the B custody checkpoint, not a cryptographic/runtime approval"
                );
            }
        }
        for profile in [1, 2, 3, 4, 6, 7, 8, 10, 11] {
            assert!(
                good.states
                    .iter()
                    .any(|s| s[6] == profile && s[24] == 2 && s[23] == 0 && s[19] == 1),
                "unexpected argv/file profile{profile} retains actual effect without acknowledgement"
            );
            assert!(good.states.iter().all(|s| s[6] != profile || s[23] == 0));
        }
        for profile in [5, 12, 13, 14] {
            assert!(good.states.iter().any(|s| s[6] == profile && s[24] == 4));
            assert!(
                good.states.iter().all(|s| s[6] != profile || s[19] == 0),
                "planted/unsafe/unbound profile{profile} cannot create a helper"
            );
        }
        assert!(
            good.states.iter().any(|s| s[6] == 9
                && s[24] == 3
                && s[4] == 1
                && s[19] == 1
                && s[23] == 0),
            "unavailable current-task bracket keeps the actual original effect without certification"
        );
        assert!(
            good.states.iter().any(|s| s[14] == 2
                && s[4] == 3
                && s[24] == 5
                && s[13] == 1
                && s[8] != s[26]),
            "same-name A1-B2-A3 kill preserves the foreign inode and original held fd in quarantine"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[14] == 2 && s[4] == 2 && s[24] == 2 && s[13] == 1),
            "normal reap with same-name substitution cannot reuse GS1 as GS2"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[24] == 6 && s[8] == 0 && s[13] == 0 && s[4] == 3),
            "kill cleanup can remove and release only its matching original inode after actual reap"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[4] == 2 && s[24] == 3 && s[13] == 1 && s[3] != 0),
            "unavailable GS2 keeps original held-file custody instead of partial certification"
        );
        for (control, invariant) in [
            (1, "Gs1BindsActualGeneratedGitArgvNeverFixturePath"),
            (2, "Gs1RequiresExactlyOneLink"),
            (3, "BCheckpointOkRequiresActualGs2AbsenceAtReap"),
            (4, "Gs1RequiresExclusiveScratchEntry"),
            (5, "Gs1BindsSelectedCaseRefNotOppositeCase"),
            (6, "KillCleanupNeverUnlinksForeignInodeAfterSameNameAba"),
            (7, "BCheckpointOkRequiresActualGs2AbsenceAtReap"),
            (8, "Gs1RequiresSameUid"),
            (
                9,
                "SettlementRequiresCurrentNamedScratchBeforeReleaseOrUnlink",
            ),
        ] {
            let model = git_sig_temp(control);
            let name = model.name.clone();
            let broken = model.check(100_000).unwrap();
            println!("{name}: {} states", broken.states.len());
            assert!(
                broken.violations.iter().any(|v| v.property == invariant),
                "control{control}: {:?}",
                broken.violations
            );
        }
    }

    #[test]
    fn dream_read_qualification_separates_reviewed_program_premise_from_runtime_census() {
        let good = qualification(0).check(100_000).unwrap();
        println!(
            "HarnessDreamReadQualification: {} states",
            good.states.len()
        );
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states.iter().any(|s| s[25] == 1),
            "hypothetical fully approved world can trust after independent proof and actual settlement"
        );
        for case in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            assert!(
                good.states
                    .iter()
                    .any(|s| s[12] == case && s[26] == 1 && s[15] == 0),
                "case {case} has an explicit no-birth refusal"
            );
            assert!(
                good.states.iter().all(|s| s[12] != case || s[15] == 0),
                "case {case} never creates an effect"
            );
        }
        assert!(
            good.states
                .iter()
                .any(|s| s[3] == 2 && s[4] == 0 && s[26] == 1),
            "A1 approval remains stale at A3"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[34] == 1 && s[46] == 3 && s[47] == 0),
            "normal probe remains unavailable with no native birth"
        );
        assert!(
            good.states.iter().any(|s| s[34] == 2
                && s[46] == 1
                && s[48] == 1
                && s[23] == 1
                && s[16] == 0),
            "conditional private positive has real valid proof and actual original reap"
        );
        assert!(
            good.states.iter().any(|s| s[34] == 2
                && s[46] == 2
                && s[48] == 2
                && s[23] == 1
                && s[16] == 0),
            "real corrupt proof is an expected private rejection, never a valid proof"
        );
        assert!(
            good.states
                .iter()
                .all(|s| s[34] != 2 || (s[44] == 0 && s[45] == 0 && s[25] == 0 && s[5] == 0)),
            "every private outcome remains unapproved, conditional and untrusted"
        );
        for case in 2..=9 {
            assert!(
                good.states
                    .iter()
                    .any(|s| s[34] == 2 && s[35] == case && s[46] == 3 && s[47] == 0),
                "invalid private tuple {case} explicitly refuses before birth"
            );
            assert!(
                good.states
                    .iter()
                    .all(|s| s[34] != 2 || s[35] != case || s[47] == 0),
                "invalid private tuple {case} cannot create a helper"
            );
        }
        assert!(
            good.states.iter().any(|s| s[34] == 2
                && s[3] == 2
                && s[4] == 0
                && s[47] == 1
                && s[16] == 1
                && s[20] == 9),
            "private A3 retains original A1 effect, owner and unpaid debt"
        );
        for candidate in 0..=1 {
            assert!(
                good.states.iter().any(|s| s[34] == 2
                    && s[51] == candidate
                    && s[50] == 4
                    && s[46] == 1
                    && s[60] == 1),
                "candidate {candidate} reaches conditional completion only with bound/revalidated inner tuple"
            );
        }
        for profile in 1..=6 {
            assert!(
                good.states
                    .iter()
                    .any(|s| s[34] == 2 && s[59] == profile && s[46] == 3 && s[47] == 0),
                "inner profile {profile} explicitly refuses before any native birth"
            );
            assert!(
                good.states
                    .iter()
                    .all(|s| s[34] != 2 || s[59] != profile || s[47] == 0),
                "bad inner profile {profile} never dispatches a helper"
            );
        }
        assert!(
            good.states.iter().any(|s| s[34] == 2
                && s[3] == 2
                && s[53] == 0
                && s[50] == 5
                && s[47] == 0),
            "A1 parsed inner cannot silently bind at A3"
        );
        assert!(
            good.states.iter().any(|s| s[34] == 2
                && s[50] == 5
                && s[47] == 1
                && s[16] == 1
                && s[20] == 9
                && s[60] == 0),
            "post-bind change retains original owned effect/debt and withdraws currentness"
        );
        let adversarial = qualification(1).check(100_000).unwrap();
        println!(
            "HarnessDreamReadQualificationAdversarial: {} states",
            adversarial.states.len()
        );
        assert!(
            adversarial.violations.is_empty(),
            "{:?}",
            adversarial.violations
        );
        for kind in 1..=3 {
            assert!(
                adversarial
                    .states
                    .iter()
                    .any(|s| s[28] == kind && s[29] == 2 && s[26] == 1 && s[21] == 0),
                "hidden case {kind} leaves the census unchanged; explicit withdrawn approval already refused"
            );
        }
        let expected = [
            (2, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (3, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (4, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (5, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (6, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (7, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (8, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (9, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (10, "TrustedOutputHasNoUnaccountedProgramWork"),
            (11, "TrustedOutputHasNoUnaccountedProgramWork"),
            (12, "TrustedOutputHasNoUnaccountedProgramWork"),
            (13, "TrustedOutputHasNoUnaccountedProgramWork"),
            (14, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (
                15,
                "QualificationFailureNeverReleasesUnreapedOriginalEffect",
            ),
            (16, "QualificationCannotMintIndependentPublicationProof"),
            (17, "NativeBirthNeedsCompleteCurrentPreparedQualification"),
            (18, "TrustRequiresActualSettlementAndPaidDebt"),
            (19, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (20, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (21, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (22, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (23, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (24, "PrivateOriginalOperationNeverCreatesDuplicateEffect"),
            (25, "PrivateOriginalEffectKeepsCustodyThroughRestart"),
            (26, "PrivateConditionalResultNeverGrantsRuntimeAuthority"),
            (27, "PrivateObservationNeverProvesWholeCommandLifetime"),
            (28, "PrivateConditionalResultNeverGrantsRuntimeAuthority"),
            (29, "NormalProbeRemainsUnavailableBeforeNativeBirth"),
            (30, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (
                31,
                "PrivateSuccessfulOutcomeRequiresActualPaidProofSettlement",
            ),
            (
                32,
                "PrivateOrdinaryOrContinueAdmissionNeverOverlapsUnpaidDebt",
            ),
            (33, "PrivateNativeBirthRequiresExactCurrentBoundPretest"),
            (
                34,
                "BoundPrivateInnerRequiresCompleteReviewAndClosureCoverage",
            ),
            (35, "BoundPrivateInnerKeepsParsedCandidateAndExactEpoch"),
            (36, "BoundPrivateInnerUsesCanonicalAarch64Identity"),
            (37, "ParsedPrivateInnerHasReservedCombined128KiBMetadata"),
            (38, "BoundPrivateInnerKeepsParsedCandidateAndExactEpoch"),
            (39, "BoundPrivateInnerHasExactProjectionPinsAndCurrentness"),
            (40, "ParsedPrivateInnerRequiresCanonicalTypedShape"),
            (
                41,
                "PrivateSuccessfulOutcomeHasPostSettlementInnerCurrentnessFence",
            ),
        ];
        for (control, invariant) in expected {
            let model = qualification(control);
            let result = model.check(100_000).unwrap();
            println!(
                "{}: {} states, {:?}",
                model.name,
                result.states.len(),
                result.violations
            );
            assert!(
                result
                    .violations
                    .iter()
                    .any(|v| format!("{v:?}").contains(invariant)),
                "control {control}: {:?}",
                result.violations
            );
            if (10..=13).contains(&control) {
                assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[25] == 1 && s[28] > 0 && s[29] == 2),
                    "control {control} reaches hidden-work trust with an unchanged observed census"
                );
            }
            if control == 22 {
                for case in [5, 8, 9] {
                    assert!(
                        result
                            .states
                            .iter()
                            .any(|s| s[34] == 2 && s[35] == case && s[47] == 1 && s[30] == 0),
                        "omitted current-pin guard exposes substituted/same-version artifact, wrong-root and stale-probe class {case}"
                    );
                }
            }
        }
    }

    #[test]
    fn member_window_observer_guard_keeps_every_original_state_edge_and_named_verdict() {
        for control in 1..=49 {
            let original = member_window_observer(control, true)
                .check(100_000)
                .unwrap();
            let reduced = member_window(control).check(100_000).unwrap();
            // The checker uses deterministic BFS and unchanged action order.
            // Equality of every state and every labelled adjacency entry is
            // stronger than matching counts or aggregate digests.
            assert_eq!(original.states, reduced.states, "control{control} states");
            assert_eq!(original.edges.len(), reduced.edges.len());
            for (a, b) in original.edges.iter().zip(&reduced.edges) {
                assert_eq!(
                    a.iter().map(|e| (e.action, e.to)).collect::<Vec<_>>(),
                    b.iter().map(|e| (e.action, e.to)).collect::<Vec<_>>(),
                    "control{control} labelled edges"
                );
            }
            assert_eq!(
                original
                    .violations
                    .iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle))
                    .collect::<Vec<_>>(),
                reduced
                    .violations
                    .iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle))
                    .collect::<Vec<_>>(),
                "control{control} named verdicts"
            );
        }
    }

    #[test]
    fn measured_extension_keeps_every_legacy_state_labelled_edge_and_named_trace() {
        use std::collections::BTreeSet;
        for control in 0..=28 {
            let legacy = member_window_projection(control, false, false, false);
            let extended = member_window(control);
            let before = legacy.check(100_000).unwrap();
            let after = extended.check(100_000).unwrap();
            let width = legacy.variables.len();
            assert_eq!(width, 54);
            let old_states = before.states.iter().cloned().collect::<BTreeSet<_>>();
            let new_states = after
                .states
                .iter()
                .filter(|s| s[width] == 0)
                .map(|s| s[..width].to_vec())
                .collect::<BTreeSet<_>>();
            assert_eq!(old_states, new_states, "control{control} legacy states");
            let mut old_edges = BTreeSet::new();
            for (from, edges) in before.edges.iter().enumerate() {
                for edge in edges {
                    old_edges.insert((
                        before.states[from].clone(),
                        legacy.actions[edge.action].name.clone(),
                        before.states[edge.to].clone(),
                    ));
                }
            }
            let mut new_edges = BTreeSet::new();
            for (from, edges) in after.edges.iter().enumerate() {
                if after.states[from][width] != 0 {
                    continue;
                }
                for edge in edges {
                    if after.states[edge.to][width] == 0 {
                        new_edges.insert((
                            after.states[from][..width].to_vec(),
                            extended.actions[edge.action].name.clone(),
                            after.states[edge.to][..width].to_vec(),
                        ));
                    }
                }
            }
            assert_eq!(
                old_edges, new_edges,
                "control{control} legacy labelled edges"
            );
            // The pinned checker returns ONE witness per response property.
            // Filtering its whole-model sample by LAW0 is not a legacy verdict
            // check: another immutable profile may supply the first fair SCC
            // and shadow a genuine legacy violation (control14 exposed this).
            // Keep the full canonical state/labelled-edge comparisons above,
            // then check the induced LAW0 branch with ALL properties/fairness.
            // Only edges leaving this immutable branch are disabled; no action
            // indices, bounds, original guards/updates or properties are removed.
            let mut legacy_branch_model = extended.clone();
            for action in &mut legacy_branch_model.actions {
                let next_law = action
                    .updates
                    .iter()
                    .find_map(|(slot, expression)| (*slot == width).then(|| expression.clone()))
                    .unwrap_or(Var(width));
                action.guard = action
                    .guard
                    .clone()
                    .and(eq(width, 0))
                    .and(next_law.eq(Int(0)));
            }
            let legacy_branch = legacy_branch_model.check(100_000).unwrap();
            assert!(legacy_branch.states.iter().all(|state| state[width] == 0));
            let full_legacy_states = after
                .states
                .iter()
                .filter(|s| s[width] == 0)
                .cloned()
                .collect::<BTreeSet<_>>();
            assert_eq!(
                full_legacy_states,
                legacy_branch
                    .states
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                "control{control} induced branch keeps every full canonical legacy state"
            );
            assert_eq!(
                legacy_branch.states.len(),
                old_states.len(),
                "control{control} original-state projection must be one-to-one"
            );
            let fairness_names = |model: &Model, count: usize| {
                model
                    .weak_fair
                    .iter()
                    .take(count)
                    .map(|group| {
                        group
                            .iter()
                            .map(|index| model.actions[*index].name.clone())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                fairness_names(&legacy, legacy.weak_fair.len()),
                fairness_names(&legacy_branch_model, legacy.weak_fair.len()),
                "control{control} every original fair action group stays identical"
            );
            for group in legacy_branch_model
                .weak_fair
                .iter()
                .skip(legacy.weak_fair.len())
            {
                assert!(
                    legacy_branch
                        .edges
                        .iter()
                        .enumerate()
                        .all(|(from, edges)| edges
                            .iter()
                            .all(|edge| edge.to == from || !group.contains(&edge.action))),
                    "control{control} new-law fairness cannot enable a legacy nonstuttering step"
                );
            }
            assert!(
                legacy_branch_model.responses.len() >= legacy.responses.len(),
                "every original response property remains present"
            );
            for (original, retained) in legacy.responses.iter().zip(&legacy_branch_model.responses) {
                assert_eq!(
                    (
                        &original.name,
                        format!("{:?}", original.request),
                        format!("{:?}", original.response)
                    ),
                    (
                        &retained.name,
                        format!("{:?}", retained.request),
                        format!("{:?}", retained.response)
                    ),
                    "control{control} original response predicate stays identical"
                );
            }
            if control == 14 {
                assert!(
                    after
                        .violations
                        .iter()
                        .any(|v| v.property == "FaultHasRetirementOrOwnedUnavailableDisposition"),
                    "whole canonical graph must retain its genuine legacy response violation"
                );
                for violation in after
                    .violations
                    .iter()
                    .filter(|v| v.property == "FaultHasRetirementOrOwnedUnavailableDisposition")
                {
                    let profiles = violation
                        .prefix
                        .iter()
                        .chain(&violation.cycle)
                        .map(|i| after.states[*i][width])
                        .collect::<BTreeSet<_>>();
                    println!(
                        "control14 canonical sampled response witness profiles={profiles:?}; legacy branch checked independently"
                    );
                }
            }
            let old_traces = before
                .violations
                .iter()
                .map(|v| {
                    (
                        v.property.clone(),
                        v.prefix
                            .iter()
                            .map(|i| before.states[*i].clone())
                            .collect::<Vec<_>>(),
                        v.cycle
                            .iter()
                            .map(|i| before.states[*i].clone())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeSet<_>>();
            let new_traces = legacy_branch
                .violations
                .iter()
                .map(|v| {
                    (
                        v.property.clone(),
                        v.prefix
                            .iter()
                            .map(|i| legacy_branch.states[*i][..width].to_vec())
                            .collect::<Vec<_>>(),
                        v.cycle
                            .iter()
                            .map(|i| legacy_branch.states[*i][..width].to_vec())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(
                old_traces, new_traces,
                "control{control} legacy named traces"
            );
        }
    }

    #[test]
    fn measured_member_windows_cover_both_bases_raw_refusals_and_named_controls() {
        let model = member_window(0);
        let variable = |name: &str| model.variables.iter().position(|v| v.name == name).unwrap();
        let law = variable("measuredLawProfile");
        let basis = variable("basisWindowMeasured");
        let read0 = variable("slot0CountersReadNonePreAckPostAckRetainedWait4");
        let read1 = variable("slot1CountersReadNonePreAckPostAckRetainedWait4LeaderRollup");
        let burned = variable("cpuBurned");
        let charged = variable("cpuCharged");
        let wall = variable("wall");
        let q = variable("basisQuantumNs");
        let terminal = variable("terminalConservationPendingPassedRefused");
        let checked = variable("conservationChecked");
        let excess = variable("conservationExcessNs");
        let origin0 = variable("slot0CounterEndpointNoneStoppedExitedClosedAbsence");
        let origin1 = variable("slot1CounterEndpointNoneStoppedExitedClosedAbsence");
        let pending = variable("directExitAccountingPending");
        let wait4 = variable("actualWait4CpuNs");
        let committed = variable("committedGrantCount");
        let unique = variable("rollupUniqueAdmittedChild");
        let mixed = variable("rollupMixedUnadmittedChild");
        let exit_before_read = variable("rollupChildExitProvenBeforeRead");
        let zero_children = variable("rollupLeaderChildCountZeroBeforeRead");
        let ack_before_read = variable("rollupLeaderAckBeforeRead");
        let rollup_count = variable("qualifiedChildRollupAttributionCount");
        let cumulative = variable("rollupCumulativeChildNs");
        let baseline = variable("rollupBaselineChildNs");
        let prior_live = variable("priorLiveChildCreditNs");
        let duplicate = variable("rollupDuplicateLiveChildNs");
        let delta = variable("rollupCurrentDeltaNs");
        let reported = variable("reportedMemberCpuNs");
        let raw_q = variable("rawGrantQuantumNs");
        let raw_basis = variable("rawGrantBasisWindowMeasured");
        let provisional = variable("rollupConservationPending");
        let validated = variable("rollupConservationValidated");
        let revoked = variable("rollupBasisRevoked");
        let fallback_q = variable("retainedFallbackQuantumNs");
        let fallback_debt = variable("retainedFallbackDebtMs");
        let stage = variable("initialStoppedCorroborationStage");
        let staged_cpu = variable("initialStoppedCorroborationInclusiveCpuNs");
        let prior_debt = variable("initialStoppedCorroborationPriorDebtMs");
        let initial_ready = variable("initialStoppedCorroborationPaidSeal");
        let initial_attempt = variable("initialTerminalRiAttemptedBeforeWait4");
        let initial_known = variable("initialTerminalRiKnown");
        let initial_l = variable("initialTerminalCreditedKernelNs");
        let initial_paid = variable("initialTerminalObservationCostPaidBeforeWait4");
        let initial_bound = variable("initialTerminalRetainedCostBoundNs");
        let initial_attempt_debt = variable("initialTerminalRetainedAttemptDebtMs");
        let initial_unknown = variable("initialTerminalConservationUnknown");
        let initial_unknown_debt = variable("initialTerminalUnknownVerdictDebtMs");
        let n_reaped = variable("actualProvenReapedProcessCount");
        let teardown_limit = variable("fixedExitTeardownLimitPerProcessNs");
        let teardown_allowance = variable("separateExitTeardownConservationAllowanceNs");
        let residual = variable("actualTerminalResidualCpuNs");
        let residual_charged = variable("terminalResidualChargedAsCostNs");
        let residual_total = variable("terminalResidualInclusiveCostBoundNs");
        let residual_observed = variable("terminalResidualCostObserved");
        let residual_paid = variable("terminalResidualFullCostPaid");
        let residual_known = variable("terminalResidualCostKnown");
        let terminal_debt = variable("terminalReconciliationDebtMs");
        let child_reap = variable("qualifiedUniqueGitReapedChildCountedOnce");
        let born_latch = variable("bornStoppedOriginalPhysicalLatch");
        let born_cost_seen = variable("bornStoppedNativeCostObserved");
        let born_cost_retained = variable("bornStoppedNativeCostRetainedFunded");
        let born_native_paid = variable("bornStoppedNativeCostPaidReceipt");
        let born_native_cost = variable("bornStoppedNativeInclusiveCostBoundNs");
        let born_native_debt = variable("bornStoppedNativeCreatedDebtMs");
        let born_raw_known = variable("bornStoppedOriginalNativeAdmissionCostKnown");
        let born_receipt_clocks = variable("bornStoppedReceiptClosingClocksKnown");
        let born_expected = variable("bornStoppedRetainedExpectedMembers");
        let born_activated = variable("bornStoppedImplicitReservedSlotActivated");
        let born_physical = variable("bornStoppedFreshPhysicalClosureValidated");
        let born_attempt_retained = variable("bornStoppedTerminalAttemptRetained");
        let born_attempt_clocks = variable("bornStoppedTerminalAttemptClosingClocksKnown");
        let adaptive_run = variable("adaptiveRunBudgetNs");
        let adaptive_cap = variable("adaptiveRunCapNs");
        let adaptive_target = variable("adaptiveAggregateTargetANs");
        let adaptive_c_reserve = variable("adaptiveMaxKnownControllerReserveNs");
        let adaptive_latency = variable("adaptiveMaxMatchingStopAckLatencyNs");
        let adaptive_slots = variable("adaptiveExpectedReservedSlots");
        let issued_run = variable("issuedAdaptiveRunBudgetNs");
        let issued_cap = variable("issuedAdaptiveRunCapNs");
        let current_c = variable("adaptiveCurrentCompleteControllerNs");
        let current_a = variable("adaptiveCurrentAggregateActiveNs");
        let feedbacks = variable("adaptiveAppliedClosedGrantReceipts");
        let scheduler_clock = variable("adaptiveElapsedClockKnown");
        let elapsed_stage = variable("adaptiveFreshElapsedNonePreBudgetStopDue");
        let elapsed = variable("adaptiveFreshElapsedNs");
        let optional_count = variable("adaptiveOptionalWorkCountInGrant");
        let optional_at = variable("adaptiveOptionalWorkPriorElapsedNs");
        let optional_fresh = variable("adaptiveOptionalWorkHadFreshClock");
        let feedback_kind = variable("adaptiveFeedbackNoneHalveGrowHold");
        let live_mask = variable("adaptiveRetainedValidLiveStopAckMask");
        let delay0 = variable("adaptiveRetainedSlot0StopAckDelayNs");
        let delay1 = variable("adaptiveRetainedSlot1StopAckOrExitDelayNs");
        let matching_stop = variable("adaptiveRetainedSuccessfulSameGrantStop");
        let latency_known = variable("adaptiveCurrentLiveLatencyKnown");
        let good = model.check(100_000).unwrap();
        for violation in good.violations.iter().take(3) {
            member_window_failure_rows(
                &model,
                &good.states,
                &violation.property,
                &violation.prefix,
                &violation.cycle,
                |row| {
                    good.edges[row]
                        .iter()
                        .map(|edge| (edge.to, model.actions[edge.action].name.clone()))
                        .collect()
                },
            );
        }
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states.iter().any(|s| s[law] == 1
                && s[0] == 7
                && s[basis] == 1
                && s[q] == 500_000
                && s[wall] == 20_000_000
                && s[26] == 1
                && s[27] == 9),
            "slow wall/low CPU measured grant pays fresh debt without borrowing wall credit"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 2
                && s[0] == 7
                && s[basis] == 1
                && s[q] == 1_000_000
                && s[wall] == 25_000_000
                && s[26] == 1
                && s[27] == 9),
            "both exact hard boundaries are accepted with full fresh9ms debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 1
                && s[49] == 2
                && s[0] == 7
                && s[burned] == 625_000
                && s[charged] == 625_000),
            "two separately paid grants accumulate CPU without resetting startup accounting"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 3
                && s[0] == 3
                && s[initial_ready] == 1
                && s[stage] == 5
                && s[49] == 0
                && s[42] == 0),
            "future within-grant uncertainty cannot pre-refuse the originally proved stopped startup"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 3
                && s[0] == 4
                && s[49] == 1
                && s[8] == 5
                && s[42] == 1
                && s[10] == 1
                && s[11] == 1
                && s[15] == 0
                && s[18] == 0
                && s[20] == 0
                && s[26] == 0),
            "unknown child arises within its already-open original window without backdated birth, ACK or seal"
        );
        for profile in [3, 4, 9, 10] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[0] == 7
                    && s[basis] == 0
                    && s[26] == 0
                    && s[q] > 0
                    && s[27] > 0),
                "unknown/preACK/missing/reversed profile{profile} retains window charge and refuses trust"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 10
                && s[0] == 7
                && s[37] == 6
                && s[27] == 9
                && s[28] == 9
                && s[41] == 1
                && s[51] == 1
                && s[26] == 0
                && s[34] == 0
                && s[checked] == 0),
            "fresh refusal debt does not block an owned unavailable disposition or fabricate settlement"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[law] == 4 && s[0] == 7 && s[read0] == 1),
            "early reads never turn into postACK evidence merely because ACK arrives later"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 7
                && s[0] == 7
                && s[basis] == 1
                && s[q] == 500_000
                && s[wall] == 30_000_000
                && s[26] == 0
                && s[37] == 6
                && s[27] == 9),
            "30ms remains a raw hard wall refusal even when measured CPU passes"
        );
        let a0 = variable("slot0ActualCpuNs");
        let a1 = variable("slot1ActualCpuNs");
        let w0 = variable("slot0WindowNs");
        assert!(
            good.states.iter().any(|s| s[law] == 8
                && s[0] == 7
                && s[a0] > s[w0]
                && s[a0] + s[a1] <= s[wall]
                && s[basis] == 0
                && s[26] == 0),
            "aggregate passing CPU cannot replace the per-slot one-thread coverage predicate"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 5
                && s[0] == 7
                && s[q] == 2_500_000
                && s[26] == 0
                && s[27] == 23
                && s[burned] == 2_375_000
                && s[charged] == 2_375_000),
            "actual2ms descendant rollup is retained and its overrun pays rounded-up full debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 11
                && s[48] == 27
                && s[46] == 1
                && s[burned] == 625_000
                && s[charged] == 625_000),
            "late member CPU and controller post-work each pay debt, but controller work adds no member credit"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 1
                && s[40] == 1
                && s[checked] == 1
                && s[terminal] == 1
                && s[34] == 1),
            "trusted measured accounting requires actual reap and passed terminal conservation"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 11
                && s[40] == 1
                && s[wait4] == 625_000
                && s[charged] == 625_000
                && s[residual_observed] == 1
                && s[residual_total] == 250_000
                && s[residual] == 0
                && s[residual_paid] == 0
                && s[checked] == 0
                && s[terminal_debt] == 9
                && s[27] == 9
                && s[28] == 9),
            "even zero helper residual retains a fresh ordinary terminal C unit and its full unpaid debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 11
                && s[40] == 1
                && s[residual_observed] == 1
                && s[residual_total] == 250_000
                && s[residual_paid] == 1
                && s[27] == 0
                && s[28] == 0
                && s[checked] == 1
                && s[terminal] == 1),
            "actual reap has a local full-payment/conservation response without another OS event or helper grant"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 14
                && s[40] == 1
                && s[checked] == 0
                && s[wait4] == 500_000
                && s[charged] == 375_000
                && s[n_reaped] == 1
                && s[teardown_limit] == 150_000
                && s[teardown_allowance] == 150_000
                && s[residual] == 125_000
                && s[residual_charged] == 125_000
                && s[residual_total] == 375_000
                && s[residual_observed] == 1
                && s[residual_paid] == 0
                && s[terminal_debt] == 9
                && s[27] == 9
                && s[28] == 9),
            "the newly authorized fixed ceiling never waives any of the original125us residual COST or its fresh debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 12
                && s[pending] == 1
                && s[read0] == 3
                && s[origin0] == 2
                && s[40] == 1
                && s[checked] == 0),
            "direct retained wait4 is acquired before conservation classification, never from RI exit"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 12
                && s[pending] == 1
                && s[40] == 0
                && s[read0] == 0
                && s[basis] == 0
                && s[q] == 0
                && s[26] == 0
                && s[33] == 0
                && s[34] == 0
                && s[41] == 1
                && s[committed] == 0),
            "direct exit parks accounting while original retirement/reap remains possible without a charge"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 12
                && s[pending] == 0
                && s[40] == 1
                && s[read0] == 3
                && s[basis] == 1
                && s[q] == 500_000
                && s[27] == 9
                && s[checked] == 0
                && s[committed] == 1),
            "own actual wait4 can commit the frozen terminal grant then pay its fresh debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 12
                && s[40] == 1
                && s[terminal] == 1
                && s[checked] == 1
                && s[34] == 1),
            "deferred direct-exit accounting has an actual settled trusted path without charge/reap deadlock"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 12
                && s[40] == 1
                && s[pending] == 0
                && s[basis] == 0
                && s[q] == 1_000_000
                && s[37] != 0
                && s[checked] == 1
                && s[27] == 0
                && s[41] == 0
                && s[34] == 0),
            "fault/ABA while pending retains window accounting and can settle without granting trust"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 13
                && s[0] == 7
                && s[origin1] == 2
                && s[read1] == 0
                && s[basis] == 0
                && s[q] == 1_000_000
                && s[26] == 1),
            "implicit exited child lacks own wait4 and keeps the proven conservative window basis"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 14
                && s[40] == 1
                && s[checked] == 1
                && s[terminal] == 1
                && s[wait4] == 500_000
                && s[charged] == 375_000
                && s[burned] == 500_000
                && s[teardown_allowance] == 150_000
                && s[residual_charged] == 125_000
                && s[residual_paid] == 1
                && s[27] == 0
                && s[28] == 0),
            "independent prefix remains375us while a separately fixed150us ceiling admits the paid125us terminal residual"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 28
                && s[40] == 1
                && s[checked] == 1
                && s[terminal] == 2
                && s[wait4] == 450_000
                && s[charged] == 200_000
                && s[teardown_allowance] == 150_000
                && s[residual] == 250_000
                && s[residual_charged] == 250_000
                && s[residual_total] == 500_000
                && s[residual_paid] == 1
                && s[terminal_debt] == 9
                && s[excess] == 100_000
                && s[27] == 0
                && s[28] == 0
                && s[37] == 6
                && s[34] == 0),
            "actual post-sample250us burn exceeds fixed150us and stays refused after its FULL residual cost/debt is paid"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 15
                && s[49] == 2
                && s[0] == 7
                && s[basis] == 1
                && s[read1] == 4
                && s[unique] == 1
                && s[mixed] == 0
                && s[exit_before_read] == 1
                && s[zero_children] == 1
                && s[ack_before_read] == 1
                && s[rollup_count] == 1
                && s[cumulative] == 250_000
                && s[baseline] == 125_000
                && s[prior_live] == 125_000
                && s[delta] == 125_000
                && s[duplicate] == 0
                && s[reported] == 375_000
                && s[q] == 625_000
                && s[charged] == 750_000
                && s[burned] == 750_000
                && s[provisional] == 1
                && s[validated] == 0
                && s[40] == 0
                && s[34] == 0),
            "unique post-exit rollup excludes prior live child credit and startup, and remains provisional"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 15
                && s[read1] == 4
                && s[40] == 1
                && s[wait4] == 750_000
                && s[charged] == 750_000
                && s[n_reaped] == 2
                && s[child_reap] == 1
                && s[teardown_allowance] == 300_000
                && s[provisional] == 0
                && s[validated] == 1
                && s[revoked] == 0
                && s[terminal] == 1
                && s[34] == 1),
            "qualified child rollup is distinct from own wait4 and trusts only after final leader conservation"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 15
                && s[49] == 2
                && s[read1] == 4
                && s[40] == 0
                && s[n_reaped] == 0
                && s[child_reap] == 0),
            "two grants and qualified source reads cannot count an exit ceiling before actual leader reap"
        );
        for profile in [13, 16, 17, 18] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[40] == 1
                    && s[n_reaped] == 1
                    && s[child_reap] == 0
                    && s[teardown_allowance] == 150_000),
                "unproven/exited/mixed/pre-exit/duplicate child profile{profile} adds no second teardown allowance"
            );
        }
        assert!(
            good.states.iter().all(|s| s[read1] != 3),
            "leader child rollup is never fabricated as controller-owned child wait4"
        );
        for profile in [16, 17, 18] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[0] == 7
                    && s[read1] == 4
                    && s[basis] == 0
                    && s[q] == 1_000_000
                    && s[37] == 6
                    && s[26] == 0
                    && s[34] == 0
                    && s[27] == 9),
                "mixed/pre-exit/duplicated rollup profile{profile} retains window cost and refuses authority"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 17
                && s[0] == 7
                && s[8] == 4
                && s[exit_before_read] == 0
                && s[zero_children] == 0
                && s[ack_before_read] == 1
                && s[read1] == 4
                && s[basis] == 0),
            "later child exit cannot backdate causal closure of an earlier rollup read"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 19
                && s[40] == 1
                && s[terminal] == 2
                && s[checked] == 1
                && s[revoked] == 1
                && s[basis] == 0
                && s[raw_basis] == 1
                && s[raw_q] == 625_000
                && s[reported] == 375_000
                && s[q] == 1_000_000
                && s[fallback_q] == 1_000_000
                && s[fallback_debt] == 9
                && s[wait4] == 900_000
                && s[charged] == 500_000
                && s[n_reaped] == 2
                && s[child_reap] == 1
                && s[teardown_allowance] == 300_000
                && s[residual_charged] == 400_000
                && s[residual_paid] == 1
                && s[excess] == 100_000
                && s[27] == 9
                && s[34] == 0),
            "terminal rollup mismatch revokes effective basis and retains raw shortfall plus fresh fallback billing"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 19
                && s[revoked] == 1
                && s[27] == 0
                && s[28] == 0
                && s[raw_q] == 625_000
                && s[wait4] == 900_000
                && s[residual] == 400_000
                && s[excess] == 100_000
                && s[terminal] == 2
                && s[34] == 0),
            "paid fallback debt never erases the original measured receipt or reclassifies conservation failure"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 1
                && s[49] == 1
                && s[stage] == 5
                && s[initial_ready] == 1
                && s[staged_cpu] == 1_164_833),
            "the >1ms total is separately paced stopped work before actual first CONT"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 20
                && s[40] == 0
                && s[initial_attempt] == 1
                && s[initial_known] == 1
                && s[initial_l] == 75_000
                && s[burned] == 200_000
                && s[charged] == 200_000
                && s[27] == 9
                && s[28] == 9),
            "eligible initial RI is independent before own wait4 and pays fresh debt without a grant"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 20
                && s[40] == 1
                && s[terminal] == 1
                && s[wait4] == 200_000
                && s[initial_l] == 75_000
                && s[initial_paid] == 1
                && s[49] == 0
                && s[34] == 0),
            "a covered startup-only actual reap can conserve without manufacturing helper/output authority"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 27
                && s[40] == 1
                && s[terminal] == 1
                && s[initial_attempt] == 1
                && s[initial_l] == 0
                && s[wait4] == 125_000),
            "mandatory attempt remains distinct from numerical need for any additional kernel credit"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 22
                && s[40] == 1
                && s[checked] == 1
                && s[terminal] == 1
                && s[wait4] == 325_000
                && s[charged] == 200_000
                && s[residual_charged] == 125_000
                && s[residual_paid] == 1
                && s[teardown_allowance] == 150_000
                && s[initial_known] == 1
                && s[49] == 0
                && s[34] == 0),
            "available initial RI plus fixed teardown bound has a genuinely paid residual path without runtime/output authority"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 21
                && s[40] == 0
                && s[initial_attempt] == 1
                && s[initial_known] == 0
                && s[initial_l] == 0
                && s[initial_paid] == 0
                && s[initial_bound] == 3_000_000
                && s[initial_attempt_debt] == 27
                && s[27] == 27
                && s[28] == 27
                && s[51] == 1
                && s[41] == 1),
            "unavailable initial RI retains UNKNOWN3ms/fresh27ms in the same held original owner before reap"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 21
                && s[40] == 1
                && s[initial_attempt] == 1
                && s[initial_known] == 0
                && s[initial_l] == 0
                && s[initial_paid] == 1
                && s[initial_attempt_debt] == 27
                && s[initial_unknown] == 1
                && s[initial_unknown_debt] == 27
                && s[checked] == 1
                && s[terminal] == 0
                && s[excess] == 0
                && s[wait4] == 200_000
                && s[charged] == 125_000
                && s[37] == 6
                && s[34] == 0
                && s[27] == 0
                && s[28] == 0
                && s[residual_known] == 0
                && s[residual_total] == 3_000_000
                && s[residual_paid] == 1
                && s[charged] + s[teardown_allowance] >= s[wait4]),
            "even a numerically fitting fixed ceiling never repairs unavailable RI: full unknown cost is paid, accounting remains unknown/refused"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 21
                && s[40] == 1
                && s[initial_paid] == 1
                && s[checked] == 0
                && s[residual_observed] == 1
                && s[residual_known] == 0
                && s[residual_total] == 3_000_000
                && s[initial_unknown_debt] == 27
                && s[27] == 27
                && s[28] == 27),
            "actual terminal observation retains its own full conservative unknown27ms unit before classification"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 21
                && s[initial_unknown] == 1
                && s[27] == 0
                && s[28] == 0
                && s[terminal] == 0
                && s[excess] == 0
                && s[34] == 0),
            "unknown conservation remains refused after all conservative fresh debt is paid"
        );
        for profile in 23..=26 {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[37] == 6
                    && s[51] == 1
                    && s[41] == 1
                    && s[40] == 0
                    && s[34] == 0),
                "unknown initial source/roster/timebase/payment keeps original pins in owned quarantine"
            );
            assert!(
                good.states.iter().all(|s| s[law] != profile
                    || (s[initial_attempt] == 0 && s[initial_l] == 0 && s[40] == 0)),
                "missing initial baseline/roster/paid evidence/timebase cannot acquire independent L"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 29
                && s[0] == 3
                && s[born_latch] == 1
                && s[born_cost_seen] == 0
                && s[born_expected] == 2
                && s[born_activated] == 0
                && s[49] == 0
                && s[52] == 0
                && s[37] == 5
                && s[41] == 1
                && s[40] == 0
                && s[charged] == 0),
            "Attach callback panic retains the original born-stopped source BEFORE native callback/creation settlement or first CONT"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 29
                && s[born_cost_retained] == 1
                && s[born_native_cost] == 2_000_000
                && s[born_native_debt] == 18
                && s[born_native_paid] == 0
                && s[27] == 18
                && s[28] == 18
                && s[40] == 0),
            "actual native birth cost remains separately retained with its full fresh18ms before physical-only settlement"
        );
        for profile in [29, 30, 32] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[40] == 0
                    && s[born_native_paid] == 1
                    && s[born_physical] == 1
                    && s[initial_attempt] == 1
                    && s[born_attempt_retained] == 1
                    && s[initial_bound] == 3_000_000
                    && s[initial_attempt_debt] == 27
                    && s[initial_paid] == 0
                    && s[27] == 27
                    && s[28] == 27
                    && s[initial_l] == 0
                    && s[charged] == 0),
                "early profile{profile} retains one actual terminal RI attempt and full UNKNOWN3ms/27ms; no startup/L credit"
            );
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[40] == 1
                    && s[born_latch] == 1
                    && s[born_native_paid] == 1
                    && s[initial_paid] == 1
                    && s[initial_known] == i64::from(profile != 30)
                    && s[initial_unknown] == 1
                    && s[checked] == 1
                    && s[terminal] == 0
                    && s[wait4] == 125_000
                    && s[charged] == 0
                    && s[initial_l] == 0
                    && s[teardown_allowance] == 150_000
                    && s[n_reaped] == 1
                    && s[child_reap] == 0
                    && s[residual_known] == 0
                    && s[residual_total] == 3_000_000
                    && s[residual_paid] == 1
                    && s[27] == 0
                    && s[28] == 0
                    && s[52] == 0
                    && s[49] == 0
                    && s[born_activated] == 0
                    && s[34] == 0),
                "early profile{profile} owns actual reap after full payment, but available RI/fixedT never establishes missing startup accounting or runtime trust"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 32
                && s[born_cost_seen] == 1
                && s[born_raw_known] == 0
                && s[25] == 0
                && s[born_native_cost] == 3_000_000
                && s[born_native_debt] == 27
                && s[born_cost_retained] == 1
                && s[born_receipt_clocks] == 1
                && s[born_native_paid] == 1
                && s[40] == 1
                && s[checked] == 1
                && s[terminal] == 0
                && s[initial_unknown] == 1
                && s[charged] == 0
                && s[34] == 0),
            "raw native clock failure stays UNKNOWN while a later actual known closing receipt/full27ms permits physical-only original reap"
        );
        for profile in [31, 33, 34, 35, 36, 37] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[born_cost_seen] == 1
                    && s[37] == 5
                    && s[51] == 1
                    && s[41] == 1
                    && s[40] == 0
                    && s[34] == 0),
                "early missing source/native-clock/physical-proof/attempt-clock/funding profile{profile} keeps original custody"
            );
            assert!(
                good.states.iter().all(|s| s[law] != profile || s[40] == 0),
                "an early panic or empty census never grants own-reap permission for profile{profile}"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 34
                && s[initial_attempt] == 1
                && s[born_attempt_retained] == 1
                && s[born_attempt_clocks] == 0
                && s[27] == 0
                && s[28] == 0
                && s[initial_paid] == 0
                && s[40] == 0
                && s[41] == 1),
            "debt-field equality cannot replace the unknown current attempt closing-clock receipt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 35
                && s[born_cost_seen] == 1
                && s[born_native_cost] == 2_000_000
                && s[born_cost_retained] == 0
                && s[born_native_paid] == 0
                && s[27] == 0
                && s[28] == 0
                && s[40] == 0
                && s[41] == 1),
            "unfunded raw native cost is retained in custody instead of normalized into a paid zero"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 37
                && s[born_cost_seen] == 1
                && s[born_raw_known] == 0
                && s[born_native_cost] == 3_000_000
                && s[born_native_debt] == 9
                && s[born_cost_retained] == 0
                && s[born_native_paid] == 0
                && s[27] == 0
                && s[28] == 0
                && s[40] == 0
                && s[41] == 1
                && s[34] == 0),
            "only9ms funding cannot stand in for the whole UNKNOWN3ms/27ms native unit even after its partial debt is paid"
        );
        for (profile, slots, run) in [(38, 1, 350_000), (39, 2, 175_000)] {
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[49] == 1
                    && s[issued_run] == run
                    && s[issued_cap] == run
                    && s[adaptive_slots] == slots
                    && s[current_a] == 600_000
                    && s[current_c] == 250_000
                    && s[q] == 850_000
                    && s[27] == 9
                    && s[26] == 1
                    && s[feedbacks] == 0),
                "one aggregate target reserves ALL expected slots plus known controller/STOP latency before CONT"
            );
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[49] == 2
                    && s[feedbacks] == 2
                    && s[feedback_kind] == 2
                    && s[burned] == 975_000
                    && s[charged] == 975_000
                    && s[adaptive_run] <= s[adaptive_cap]
                    && s[adaptive_run] <= 450_000),
                "two real original grants apply bounded decrease then slow growth without inventing a third grant"
            );
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[0] == 4
                    && s[optional_count] == 1
                    && s[optional_fresh] == 1
                    && s[optional_at] < s[issued_run]
                    && s[elapsed_stage] == 0),
                "optional work invalidates its consumed elapsed timestamp"
            );
            assert!(
                good.states.iter().any(|s| s[law] == profile
                    && s[0] == 4
                    && s[optional_count] == 1
                    && s[elapsed_stage] == 2
                    && s[elapsed] == s[issued_run]
                    && s[14] == 0),
                "a new current elapsed read observes STOP due before the positive ACK"
            );
        }
        assert!(
            good.states.iter().any(|s| s[law] == 38
                && s[feedbacks] == 1
                && s[feedback_kind] == 1
                && s[issued_run] == 350_000
                && s[adaptive_run] == 175_000
                && s[adaptive_cap] == 200_000
                && s[adaptive_latency] == 250_000
                && s[adaptive_target] == 550_000),
            "above-target aggregate A halves the ACTUAL prior run once, then clamps the newly observed latency cap"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 38
                && s[feedbacks] == 2
                && s[issued_run] == 175_000
                && s[adaptive_run] == 185_000
                && s[adaptive_c_reserve] == 350_000
                && s[adaptive_target] == 450_000
                && s[adaptive_cap] == 200_000
                && s[current_c] == 350_000
                && s[q] == 600_000),
            "slow growth is10us and complete newly measured C shrinks aggregate target while the capped run ceiling remains200us; all C stays in Q"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 39
                && s[feedbacks] == 2
                && s[adaptive_slots] == 2
                && s[issued_run] == 87_500
                && s[adaptive_run] == 97_500
                && s[adaptive_cap] == 150_000
                && s[adaptive_c_reserve] == 250_000
                && s[current_c] == 200_000
                && s[adaptive_latency] == 125_000
                && s[adaptive_target] == 550_000),
            "smaller later C/latency never decay the retained reserves or replace reserved2 with a visible-count divisor"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 40
                && s[elapsed_stage] == 2
                && s[elapsed] > s[issued_run]
                && s[optional_count] == 0
                && s[14] == 0),
            "fresh elapsed already beyond budget stops before any optional grant work"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 40
                && s[q] == 1_178_500
                && s[current_a] == 932_917
                && s[current_c] == 245_583
                && s[wall] == 1_114_000
                && s[27] == 11
                && s[28] == 11
                && s[37] == 6
                && s[26] == 0
                && s[34] == 0
                && s[optional_count] == 0),
            "adaptive allocation does not erase the actual builder over-Q refusal or its full fresh11ms debt"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 40
                && s[feedbacks] == 1
                && s[adaptive_latency] == 382_000
                && s[adaptive_run] == 25_000
                && s[adaptive_cap] == 25_000
                && s[26] == 0
                && s[34] == 0),
            "a larger observed STOP latency bounds future run to the25us floor but cannot reopen a refused grant"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 41
                && s[scheduler_clock] == 0
                && s[37] == 1
                && s[51] == 1
                && s[41] == 1
                && s[40] == 0
                && s[49] == 0
                && s[feedbacks] == 0
                && s[34] == 0),
            "unknown fresh elapsed clock holds the original effect without CONT/growth or optimistic qualification"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 42
                && s[feedbacks] == 1
                && s[live_mask] == 3
                && s[delay0] == 100_000
                && s[delay1] == 300_000
                && s[matching_stop] == 1
                && s[latency_known] == 1
                && s[adaptive_latency] == 300_000
                && s[adaptive_cap] == 25_000
                && s[adaptive_run] == 25_000
                && s[adaptive_target] == 500_000
                && s[current_a] == 525_000
                && s[current_c] == 300_000
                && s[q] == 825_000),
            "MAX across both live ACK receipts, not earliest ACK, updates reserves BEFORE new-target halving/cap"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 43
                && s[49] == 2
                && s[feedbacks] == 2
                && s[live_mask] == 1
                && s[delay0] == 100_000
                && s[delay1] == 300_000
                && s[latency_known] == 1
                && s[read0] == 2
                && s[origin0] == 1
                && s[read1] == 4
                && s[origin1] == 2
                && s[rollup_count] == 1
                && s[delta] == 50_000
                && s[prior_live] == 300_000
                && s[current_a] == 300_000
                && s[q] == 550_000
                && s[adaptive_latency] == 125_000
                && s[adaptive_run] == 97_500
                && s[feedback_kind] == 2),
            "qualified exited-child rollup retains CPU once while its later exit endpoint cannot inflate LIVE STOP latency"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 44
                && s[49] == 2
                && s[pending] == 1
                && s[40] == 0
                && s[committed] == 1
                && s[live_mask] == 0
                && s[latency_known] == 0
                && s[origin0] == 2
                && s[read0] == 0
                && s[34] == 0),
            "exited-only second grant remains settlement-only pending until original actual own wait4; no live ACK is invented"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 44
                && s[49] == 2
                && s[feedbacks] == 2
                && s[pending] == 0
                && s[40] == 1
                && s[committed] == 2
                && s[read0] == 3
                && s[origin0] == 2
                && s[live_mask] == 0
                && s[latency_known] == 0
                && s[matching_stop] == 1
                && s[feedback_kind] == 3
                && s[issued_run] == 175_000
                && s[adaptive_run] == 175_000
                && s[adaptive_latency] == 250_000
                && s[adaptive_cap] == 200_000
                && s[q] == 350_000
                && s[wait4] == 825_000
                && s[charged] == 825_000),
            "retained own wait4 is legal measured accounting but exited-only endpoints never regrow the predictive run"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 45
                && s[feedbacks] == 1
                && s[basis] == 0
                && s[read0] == 0
                && s[current_a] == 525_000
                && s[current_c] == 300_000
                && s[wall] == 625_000
                && s[q] == 925_000
                && s[adaptive_c_reserve] == 300_000
                && s[adaptive_target] == 500_000
                && s[issued_run] == 350_000
                && s[adaptive_run] == 175_000
                && s[feedback_kind] == 1
                && s[27] == 9
                && s[28] == 9),
            "finite observed A above NEW500us target shrinks WINDOW-basis run without granting measured authority or dropping max(W,A)+C"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 46
                && s[feedbacks] == 1
                && s[matching_stop] == 0
                && s[live_mask] == 0
                && s[latency_known] == 0
                && s[basis] == 0
                && s[feedback_kind] == 3
                && s[adaptive_run] == 350_000
                && s[37] == 6
                && s[26] == 0
                && s[34] == 0
                && s[q] == 700_000),
            "nonmatching/non-successful STOP supplies no predictive latency or growth even with retained finite stopped raw cost"
        );
        assert!(
            good.states.iter().any(|s| s[law] == 44
                && s[49] == 2
                && s[pending] == 1
                && s[40] == 0
                && s[committed] == 1
                && s[charged] == 725_000
                && s[burned] == 825_000
                && s[q] == 0
                && s[basis] == 0
                && s[checked] == 0
                && s[33] == 0
                && s[34] == 0),
            "second pending grant preserves prior real725us credit/count1 while CURRENT own-wait4 accounting is uncommitted"
        );
        assert!(
            good.states.iter().all(|s| s[law] != 44
                || s[pending] == 0
                || (s[49] == 2
                    && s[committed] == 1
                    && s[charged] == 725_000
                    && s[q] == 0
                    && s[basis] == 0
                    && s[checked] == 0
                    && s[33] == 0
                    && s[34] == 0)),
            "neither pending retirement nor retained actual reap erases prior credit or admits main proof/trust before current commit"
        );
        for (control, property) in [
            (29, "MeasuredNeedsClosure"),
            (30, "MeasuredNeedsClosure"),
            (31, "ChargedCoversBurned"),
            (32, "ChargedCoversBurned"),
            (33, "WallBounded"),
            (34, "PerSlotMeasuredCpuFitsWindow"),
            (35, "RollupRequiresUniqueAdmittedChild"),
            (36, "RollupNeedsCausalExitAndLeaderAck"),
            (37, "RollupCountedOnce"),
            (38, "InitialCorroborationStageRequiresFreshPaidRest"),
            (39, "InitialEligibleTerminalRiAttemptPrecedesOwnWait4"),
            (40, "ChargedCoversBurned"),
            (
                41,
                "TerminalResidualIsFullyChargedOutsideConservationCredit",
            ),
            (42, "TeardownCeilingIsIndependentFixedPerActualReap"),
            (43, "TeardownChildCountNeedsQualifiedRetainedReapProof"),
            (
                44,
                "BornStoppedCreationFaultReapNeedsPaidOriginalPhysicalLatch",
            ),
            (45, "IssuedAdaptiveGrantUsesMeasuredCostAndStopLatencyCap"),
            (46, "AdaptiveLatencyRetainsMaximumValidLiveAck"),
            (47, "AdaptiveGrowthNeedsMeasuredLiveSuccessfulStopAck"),
            (48, "WindowBasisAboveNewTargetHalvesPriorRun"),
            (49, "PendingDirectExitCannotAdmitExpensiveWorkOrTrust"),
        ] {
            let broken = member_window(control);
            let result = broken.check(100_000).unwrap();
            assert!(
                result.violations.iter().any(|v| v.property == property),
                "{} must violate intended {property}, not merely TypeOK",
                broken.name
            );
            assert!(
                !result.violations.iter().any(|v| v.property == "TypeOK"),
                "{} must retain every declared finite bound",
                broken.name
            );
            if control == 38 {
                assert!(
                    result.states.iter().any(|s| s[law] != 0
                        && s[stage] == 2
                        && s[prior_debt] == 9
                        && s[27] == 18
                        && s[28] == 18
                        && s[49] == 0
                        && s[initial_ready] == 0),
                    "skipping a stopped checkpoint rest admits fresh work over the retained prior obligation"
                );
            }
            if control == 39 {
                assert!(
                    result.states.iter().any(|s| s[law] == 27
                        && s[40] == 1
                        && s[initial_attempt] == 0
                        && s[wait4] == s[charged]
                        && s[terminal] == 0
                        && s[49] == 0
                        && s[34] == 0),
                    "skipped initial RI violates protocol order even when raw terminal CPU would be covered"
                );
                assert!(
                    result.states.iter().any(|s| s[law] == 21
                        && s[40] == 1
                        && s[initial_attempt] == 1
                        && s[initial_known] == 0
                        && s[initial_paid] == 0
                        && s[27] == 27
                        && s[28] == 27
                        && s[initial_attempt_debt] == 27
                        && s[terminal] == 0),
                    "the same omitted readiness guard also exposes actual own reap with retained UNKNOWN debt still unpaid"
                );
            }
            if control == 49 {
                assert!(
                    result.states.iter().any(|s| s[law] == 44
                        && s[49] == 2
                        && s[pending] == 1
                        && s[40] == 0
                        && s[committed] == 2
                        && s[charged] == 725_000
                        && s[burned] == 825_000
                        && s[read0] == 0
                        && s[origin0] == 2
                        && s[q] == 0
                        && s[basis] == 0
                        && s[33] == 0
                        && s[34] == 0),
                    "one premature CURRENT commit violates pending ownership while retaining all prior credit and no forged wait4/CPU/proof/trust"
                );
            }
            if control == 46 {
                assert!(
                    result.states.iter().any(|s| s[law] == 42
                        && s[feedbacks] == 1
                        && s[live_mask] == 3
                        && s[delay0] == 100_000
                        && s[delay1] == 300_000
                        && s[adaptive_latency] == 100_000
                        && s[adaptive_cap] == 150_000
                        && s[adaptive_run] == 87_500
                        && s[q] == 825_000
                        && s[charged] == s[burned]),
                    "one MIN-for-MAX fault selects wrong next87500 instead of25000 while all actual raw accounting stays intact"
                );
            }
            if control == 47 {
                assert!(
                    result.states.iter().any(|s| s[law] == 44
                        && s[49] == 2
                        && s[feedbacks] == 2
                        && s[40] == 1
                        && s[read0] == 3
                        && s[origin0] == 2
                        && s[live_mask] == 0
                        && s[latency_known] == 1
                        && s[feedback_kind] == 2
                        && s[issued_run] == 175_000
                        && s[adaptive_run] == 185_000
                        && s[q] == 350_000
                        && s[wait4] == 825_000
                        && s[charged] == 825_000),
                    "one exit-as-live source classification fault falsely regrows175->185us after legal own-wait4 accounting, not a forged wait4/CPU certificate"
                );
            }
            if control == 48 {
                assert!(
                    result.states.iter().any(|s| s[law] == 45
                        && s[feedbacks] == 1
                        && s[basis] == 0
                        && s[current_a] == 525_000
                        && s[current_c] == 300_000
                        && s[adaptive_target] == 500_000
                        && s[issued_run] == 350_000
                        && s[adaptive_run] == 350_000
                        && s[feedback_kind] == 3
                        && s[q] == 925_000),
                    "the old measured-only decrease guard misses finite WINDOW A above the NEW target without changing any raw Q/debt gate"
                );
            }
            if control == 45 {
                assert!(
                    result.states.iter().any(|s| s[law] == 39
                        && s[49] == 1
                        && s[issued_run] == 1_000_000
                        && s[issued_cap] == 175_000
                        && s[adaptive_slots] == 2
                        && s[current_a] == 2_200_000
                        && s[current_c] == 250_000
                        && s[q] == 2_450_000
                        && s[wall] == 2_200_000
                        && s[27] == 23
                        && s[28] == 23
                        && s[37] == 6
                        && s[26] == 0
                        && s[34] == 0
                        && s[charged] == s[burned]
                        && s[terminal] == 0),
                    "fixed>=1ms CONT run must reach actual oversize aggregate A+C/raw debt/refusal, not a fake certificate or TypeOK"
                );
            }
            if control == 44 {
                assert!(
                    result.states.iter().any(|s| s[law] == 29
                        && s[40] == 1
                        && s[born_expected] == 2
                        && s[born_activated] == 0
                        && s[born_latch] == 1
                        && s[born_native_paid] == 1
                        && s[born_physical] == 1
                        && s[initial_attempt] == 1
                        && s[initial_known] == 1
                        && s[born_attempt_retained] == 1
                        && s[initial_paid] == 0
                        && s[27] == 27
                        && s[28] == 27
                        && s[52] == 0
                        && s[49] == 0
                        && s[terminal] == 0
                        && s[charged] == 0
                        && s[34] == 0),
                    "known original physical exit plus available RI cannot bypass this CURRENT unknown attempt payment after an Attach panic"
                );
            }
            if control == 40 {
                assert!(
                    result.states.iter().any(|s| s[law] == 28
                        && s[terminal] == 1
                        && s[wait4] > s[charged] + s[teardown_allowance]
                        && s[residual_charged] == 250_000
                        && s[residual_paid] == 1),
                    "above-T bypass falsely certifies despite independent raw excess and fully paid residual cost"
                );
            }
            if control == 41 {
                assert!(
                    result.states.iter().any(|s| s[law] == 14
                        && s[residual_observed] == 1
                        && s[residual] == 125_000
                        && s[residual_charged] == 0
                        && s[charged] + s[teardown_allowance] >= s[wait4]
                        && s[charged] == 375_000
                        && s[residual_total] == 250_000),
                    "a legitimate fixed ceiling cannot excuse the missing raw residual COST"
                );
            }
            if control == 42 {
                assert!(
                    result.states.iter().any(|s| s[law] == 28
                        && s[n_reaped] == 1
                        && s[teardown_limit] == 250_000
                        && s[teardown_allowance] == 250_000
                        && s[residual] == 250_000
                        && s[residual_paid] == 1
                        && s[charged] + s[teardown_allowance] == s[wait4]),
                    "deriving T from the same wait4 makes the numerical comparison vacuous even when cost was properly paid"
                );
            }
            if control == 43 {
                assert!(
                    result.states.iter().any(|s| s[law] == 28
                        && s[40] == 1
                        && s[n_reaped] == 2
                        && s[child_reap] == 0
                        && s[8] == 0
                        && s[teardown_allowance] == 300_000
                        && s[residual_paid] == 1
                        && s[charged] + s[teardown_allowance] >= s[wait4]),
                    "an expected/unproven child cannot manufacture a second ceiling to cover otherwise excessive raw CPU"
                );
            }
            if matches!(control, 31 | 32) {
                assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[law] == if control == 31 { 5 } else { 6 }
                            && s[40] == 1
                            && s[terminal] == 1
                            && s[burned] > s[charged]),
                    "omitted rollup/conversion must falsely certify independently retained raw CPU"
                );
                assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[law] == if control == 31 { 5 } else { 6 }
                            && s[40] == 1
                            && s[terminal] == 2
                            && s[excess] > 0),
                    "the same raw shortfall also has a correct detected-refusal path"
                );
            }
            if control == 30 {
                assert!(
                    result.states.iter().any(|s| s[law] == 12
                        && s[40] == 0
                        && s[basis] == 1
                        && s[read0] == 2
                        && s[origin0] == 2),
                    "RI exit cannot be relabeled a stopped postACK counter before own actual wait4"
                );
            }
            if control == 37 {
                assert!(
                    result.states.iter().any(|s| s[law] == 18
                        && s[basis] == 1
                        && s[q] == 750_000
                        && s[rollup_count] == 2
                        && s[duplicate] == 125_000
                        && s[charged] >= s[burned]
                        && s[26] == 1),
                    "double rollup attribution must fail even when CPU fits and a conservation upper bound would pass"
                );
            }
            if control == 35 {
                assert!(
                    result.states.iter().any(|s| s[law] == 16
                        && s[basis] == 1
                        && s[mixed] == 1
                        && s[unique] == 0
                        && s[q] == 750_000
                        && s[charged] == s[burned]),
                    "mixed unadmitted child must fail its source predicate even when numeric accounting reconciles"
                );
            }
            if control == 36 {
                assert!(
                    result.states.iter().any(|s| s[law] == 17
                        && s[basis] == 1
                        && s[exit_before_read] == 0
                        && s[zero_children] == 0
                        && s[ack_before_read] == 1
                        && s[8] == 4),
                    "later retired identity cannot launder a rollup read made before proven causal exit"
                );
            }
        }
        for control in [31, 32] {
            // Remove ONLY the extra forged-certification fault. Pure omitted
            // rollup/conversion is contained by the unchanged hard gate.
            let pure = member_window_projection(control, false, true, false);
            let result = pure.check(100_000).unwrap();
            assert!(
                result.violations.is_empty(),
                "pure omission {control}: {:?}",
                result.violations
            );
            let profile = if control == 31 { 5 } else { 6 };
            let fresh_debt = if control == 31 { 21 } else { 9 };
            assert!(
                result.states.iter().any(|s| s[law] == profile
                    && s[40] == 1
                    && s[checked] == 1
                    && s[terminal] == 2
                    && s[wait4] > s[charged]
                    && s[excess] == s[wait4] - s[charged] - s[teardown_allowance]
                    && s[37] == 6
                    && s[26] == 0
                    && s[34] == 0
                    && s[terminal_debt] == fresh_debt
                    && s[residual_paid] == 1
                    && s[residual_charged] == s[wait4] - s[charged]
                    && s[27] == 0
                    && s[28] == 0
                    && s[41] == 1),
                "pure omission/conversion retains independent excess after fixed T and pays the FULL raw residual fresh debt before refusal"
            );
            assert!(
                result.states.iter().any(|s| s[law] == profile
                    && s[terminal] == 2
                    && s[excess] > 0
                    && s[27] == 0
                    && s[28] == 0
                    && s[34] == 0),
                "pure omission remains refused after all fresh debt is paid"
            );
            assert!(
                result
                    .states
                    .iter()
                    .all(|s| s[law] != profile || s[34] == 0),
                "pure omission cannot trust output even though the good hard gate contains the fault"
            );
        }
    }

    #[test]
    fn dream_read_member_windows_preserve_required_closure_and_additive_debt() {
        let model = member_window(0);
        let good = model.check(100_000).unwrap();
        println!("HarnessDreamReadMemberWindow: {} states", good.states.len());
        for violation in good.violations.iter().take(3) {
            member_window_failure_rows(&model, &good.states, &violation.property,
                &violation.prefix, &violation.cycle, |row| good.edges[row].iter()
                    .map(|edge| (edge.to, model.actions[edge.action].name.clone())).collect());
        }
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states.iter().any(|s| s[0] == 3
                && s[52] == 1
                && s[53] == 0
                && s[27] == 9
                && s[28] == 9),
            "settled creation can still have fresh controller debt before initial corroboration"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 3
                && s[52] == 1
                && s[53] == 1
                && s[27] == 9
                && s[49] == 0),
            "initial admitted corroboration creates its own fresh debt before any CONT"
        );
        assert!(
            good.states
                .iter()
                .all(|s| s[49] == 0 || (s[52] == 1 && s[53] == 1)),
            "all grants follow settled creation and admitted initial corroboration"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 4
                && s[37] != 0
                && s[10] == 1
                && s[11] == 1
                && s[41] == 1),
            "failed admission preserves both open original windows and owned effect"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 4
                && s[37] != 0
                && s[16] == 2
                && s[17] == 2
                && s[26] == 0),
            "late physical exec/token cleanup remains possible after failure without coverage"
        );
        assert!(
            good.states.iter().all(|s| s[1] != 9 || s[4] == 0),
            "unsupported whole-command basis never creates a native effect"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 9 && s[37] == 6 && s[4] == 0),
            "unsupported basis reaches an explicit no-effect refusal"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[12] == 2 && s[13] == 3 && s[21] == 3 && s[26] == 1),
            "unequal checked endpoints retain their exact union"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 2 && s[11] == 1 && s[13] == 3 && s[18] == 1),
            "unknown slot stays open from t0 through same-grant closure"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[16] == 2 && s[17] == 2 && s[11] == 1 && s[26] == 1),
            "exec keeps one logical window and requires current task capture"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 5 && s[22] > s[21] && s[26] == 0 && s[27] > 0),
            "A over W remains charged and unavailable"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 10 && s[24] == 2 && s[26] == 1),
            "exact1ms profile remains within the ordinary limit"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 6 && s[24] == 3 && s[26] == 0 && s[27] == 18),
            "1.5ms overrun refuses and retains conservative observed debt"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[30] == 1 && s[27] == 9 && s[28] == 9 && s[48] == 18),
            "late observation incurs fresh debt after preceding rest was paid"
        );
        assert!(
            good.states.iter().any(|s| s[48] == 27 && s[46] == 1),
            "grant, late kernel and post-seal corroboration each retain debt"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[34] == 1 && s[40] == 1 && s[41] == 0)
        );
        for (mode, property) in [
            (1, "WindowUnionRetainsEveryReservedIntervalCase0"),
            (2, "BothPossibleWindowsOpenAtOriginalCheckedT0"),
            (
                3,
                "TentativeEndpointsNeedPositiveCurrentTaskAcknowledgement",
            ),
            (4, "UsableSealFollowsEveryRequiredClosedRosterPredicate"),
            (
                5,
                "TentativeEndpointsNeedPositiveCurrentTaskAcknowledgement",
            ),
            (6, "UsableSealFollowsEveryRequiredClosedRosterPredicate"),
            (7, "UnexplainedActiveCpuOverWindowCannotCertify"),
            (8, "LateKernelAttributionIsExplainedAndChargedOnce"),
            (9, "AllActiveControllerAndPipeCpuCountExactlyOnceCase1"),
            (10, "AllActiveControllerAndPipeCpuCountExactlyOnceCase1"),
            (11, "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase1"),
            (12, "ObservedActiveCpuHasNoOmissionOrRollupDuplicationCase1"),
            (13, "CoverageKeepsOriginalEpochAndCompleteRoster"),
            (14, "DebtNeverDropsObservedOutstandingWork"),
            (15, "NoNewContinuationWhileClosedDebtIsPending"),
            (16, "MainProofNeverOverlapsOriginalLiveHelper"),
            (17, "FailureCannotReleaseUnreapedOriginalEffect"),
            (18, "FailureCannotReleaseUnreapedOriginalEffect"),
            (19, "FailureCannotReleaseUnreapedOriginalEffect"),
            (20, "FailureCannotReleaseUnreapedOriginalEffect"),
            (21, "FailureCannotReleaseUnreapedOriginalEffect"),
            (22, "CoverageKeepsOriginalEpochAndCompleteRoster"),
            (23, "KnownOverOneMsQuantumCannotCertify"),
            (24, "LateKernelAttributionIsExplainedAndChargedOnce"),
            (
                25,
                "StickyFailedAdmissionForbidsOrdinaryExpensiveObservation",
            ),
            (26, "InitialCorroborationRequiresSettledCreationOwnership"),
            (27, "InitialCorroborationRequiresBothPriorDebtsPaid"),
            (28, "DebtNeverDropsObservedOutstandingWork"),
        ] {
            let model = member_window(mode);
            let checked = model
                .check(100_000)
                .unwrap_or_else(|e| panic!("{}: {e}", model.name));
            println!("{}: {} states", model.name, checked.states.len());
            assert!(
                checked.violations.iter().any(|v| v.property == property),
                "{} must causally violate {property}",
                model.name
            );
        }
    }

    #[test]
    fn dream_read_exec_incarnation_has_causal_controls() {
        let good = helper_exec(0).check(100_000).unwrap();
        println!("HarnessDreamReadHelperExec: {} states", good.states.len());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states.iter().any(|s| s[0] == 2
                && s[1] == 2
                && s[2] == 1
                && s[6] == 1
                && s[21] == 1),
            "new GPG task may execute within the already admitted clone quantum"
        );
        assert!(good.states.iter().any(|s| s[11] == 1 && s[9] == 1));
        assert!(good.states.iter().any(|s| s[15] == 1 && s[14] == 1));
        assert!(good.states.iter().any(|s| s[16] == 1 && s[15] == 0));
        let expected = [
            "RestNeverUsesOldTaskStopReceipt",
            "RetainedOldTokenNeverControlsCurrentTask",
            "CurrentTaskTokenCapturedBeforeControl",
            "OnlyOneWayCloneToGpgExecIsAdmitted",
            "TrustedExecEvidenceRequiresActualGpg",
            "TrustedExecEvidenceRequiresCurrentStopOrRealExit",
            "ReleaseRequiresActualExitReapAndRetirement",
        ];
        for (index, invariant) in expected.into_iter().enumerate() {
            let model = helper_exec((index + 1) as u8);
            let checked = model.check(100_000).unwrap();
            println!("{}: {} states", model.name, checked.states.len());
            assert!(
                checked.violations.iter().any(|v| v.property == invariant),
                "{} must causally violate {invariant}",
                model.name
            );
        }
    }

    #[test]
    fn dream_read_helper_lifecycle_and_causal_controls() {
        let good = helper(0).check(100_000).unwrap();
        println!("HarnessDreamReadHelper: {} states", good.states.len());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states.iter().any(|s| s[13] == 1),
            "covered actually reaped output is reachable"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[0] == 7 && s[26] == 0 && s[5] > 0),
            "unknown work retains real helper ownership"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[3] == 2 && s[5] == 1 && s[15] == 1),
            "unused signature creation credit stays consumed"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 2 && s[2] == 0 && s[38] == 1 && s[14] == 1),
            "old A1 effect survives and refuses A3 replay"
        );
        assert!(
            good.states.iter().any(|s| s[8] == 2
                && s[9] == 2
                && s[32] == 2
                && s[30] == 1
                && s[31] == 1),
            "two-member whole charge is reachable"
        );
        for fault in 1..=3 {
            assert!(
                good.states
                    .iter()
                    .any(|s| s[25] == fault && s[15] == 1 && s[26] == 0 && s[5] > 0)
            );
        }
        for case in 0..=9 {
            assert!(
                good.states.iter().any(|s| s[22] == case && s[37] == 1),
                "original input case {case} stays explored"
            );
        }
        assert!(
            good.states
                .iter()
                .any(|s| s[18] == 1 && s[0] == 3 && s[17] == 0),
            "STOP is consumed before native effect/outcome and before physical proof"
        );
        assert!(
            good.states.iter().any(|s| s[18] == 3 && s[17] == 0),
            "ESRCH plus retained leader exit still proves no physical stopped seal"
        );
        for outcome in [4, 5] {
            assert!(
                good.states.iter().any(|s| s[18] == outcome
                    && s[0] == 7
                    && s[15] == 1
                    && s[26] == 0
                    && s[17] == 0),
                "refused native outcome {outcome} retains its first operation/effect without stop proof"
            );
        }
        let expected = [
            "NativeEffectRequiresReservedAttempt",
            "TrustedOutputHasOnlyAdmittedMembers",
            "NativeEffectRequiresExactExecutableCapability",
            "AttachedWorkKeepsExactIncarnation",
            "NoUserCodeBeforeInitialSuspension",
            "StoppedStateRequiresAllMemberConfirmation",
            "HelperChargeIncludesDescendantsAndController",
            "HelperChargeIncludesDescendantsAndController",
            "MainProofCannotOverlapRunnableGroup",
            "DebtCoversWholeObservedUnrestedSpan",
            "OrdinaryApplicationCannotUseNativeLabel",
            "CertifiedNativeCreationMeetsCpuWallInclusiveBounds",
            "CertifiedNativeCreationMeetsCpuWallInclusiveBounds",
            "CertifiedNativeCreationMeetsCpuWallInclusiveBounds",
            "CoveredRunnableChargeMeetsWholeGroupWindow",
            "OriginalNativeEffectNeverReplayed",
            "TrustedOutputHasCompleteCoverage",
            "TrustedOutputHasActualReapAndRetirement",
            "FaultCannotReleaseUnsettledOwnership",
            "FaultCannotReleaseUnsettledOwnership",
            "FaultCannotReleaseUnsettledOwnership",
            "StopAttemptIsNotRetriedAfterAnyNativeOutcome",
        ];
        for (index, invariant) in expected.into_iter().enumerate() {
            let model = helper((index + 1) as u8);
            let checked = model
                .check(100_000)
                .unwrap_or_else(|e| panic!("{}: {e}", model.name));
            println!("{}: {} states", model.name, checked.states.len());
            assert!(
                checked.violations.iter().any(|v| v.property == invariant),
                "{} must causally violate {invariant}",
                model.name
            );
            if (18..=20).contains(&index) {
                assert!(
                    checked
                        .states
                        .iter()
                        .any(|s| s[15] == 1 && s[26] == 1 && s[20] == 0 && s[5] > 0),
                    "fault control must release an actual unreaped helper"
                );
            }
        }
    }

    #[test]
    fn dream_read_models_have_real_success_refusal_and_causal_controls() {
        let good = eligibility(0).check(100_000).unwrap();
        assert!(good.violations.is_empty());
        assert!(good.states.iter().any(|s| s[0] == 1));
        assert!(good.states.iter().any(|s| s[0] == 2));
        for (control, invariant) in [
            (1, "CertifiedHasExactCurrentHead"),
            (2, "CertifiedHasAcceptedOuterSeal"),
            (3, "CertifiedHasCompleteSelectedClosure"),
        ] {
            assert!(
                eligibility(control)
                    .check(100_000)
                    .unwrap()
                    .violations
                    .iter()
                    .any(|v| v.property == invariant)
            );
        }
        let owners = ownership(0).check(100_000).unwrap();
        assert!(owners.violations.is_empty());
        assert!(
            owners
                .states
                .iter()
                .any(|s| s[0] == 2 && s[1] == 4 && s[2] == 4)
        );
        assert!(owners.states.iter().any(|s| s[0] == 3 && s[5] == 1));
        assert!(owners.states.iter().any(|s| s[0] == 6));
        assert!(
            owners
                .states
                .iter()
                .any(|s| s[3] == 1 && s[7] == 1 && s[8] == 0)
        );
        assert!(
            owners
                .states
                .iter()
                .any(|s| s[3] == 2 && s[12] == 1 && s[8] == 0)
        );
        assert!(
            owners
                .states
                .iter()
                .any(|s| s[3] == 3 && s[9] == 2 && s[10] == 0 && s[13] == 1)
        );
        for control in 1..=4 {
            assert!(
                !ownership(control)
                    .check(100_000)
                    .unwrap()
                    .violations
                    .is_empty()
            );
        }
        for (control, invariant) in [
            (1, "LeaderReapRequiresPriorControllerRetirement"),
            (3, "RetiredOrReapedGroupCannotBeSignaled"),
            (4, "ForegroundOwnedGroupCannotExecute"),
        ] {
            assert!(
                ownership(control)
                    .check(100_000)
                    .unwrap()
                    .violations
                    .iter()
                    .any(|v| v.property == invariant)
            );
        }
        let snapshots = snapshot(0).check(100_000).unwrap();
        assert!(snapshots.violations.is_empty());
        assert!(snapshots.states.iter().any(|s| s[6] == 1));
        assert!(
            snapshots
                .states
                .iter()
                .any(|s| s[0] == 2 && s[1] == 0 && s[6] == 2)
        );
        for (control, invariant) in [
            (1, "DescriptorFollowsEveryImmutablePage"),
            (2, "DescriptorBindsExactMonotonicSourceRevision"),
        ] {
            assert!(
                snapshot(control)
                    .check(100_000)
                    .unwrap()
                    .violations
                    .iter()
                    .any(|v| v.property == invariant)
            );
        }
    }
}

// Origin: https://github.com/bboxy450/bboxy450-harness
// Retain accurate reuse credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
/// One terminal envelope, one producer and one subscriber. Publication, queue
/// observation and the close release are separate atomic actions. Reader-only
/// weak fairness proves drainage; OS scheduling and byte/clock bounds are API
/// witnesses, not consequences of this finite model.
pub fn close_receive(control: u8) -> Model {
    let mut m = base(
        if control == 1 {
            "DreamReadCloseReceiveLateClosedRead"
        } else {
            "DreamReadCloseReceive"
        },
        control != 0,
        &[
            ("readerIdleCapturedEmptyEof", 3, 0),
            ("queuedTerminal", 1, 0),
            ("terminalPublished", 1, 0),
            ("producerClosedRelease", 1, 0),
            ("capturedClosedBeforeDrain", 1, 0),
            ("terminalDelivered", 1, 0),
            ("returnedEof", 1, 0),
            ("transportFault", 1, 0),
            ("explicitDeadline", 1, 0),
        ],
    );
    const PH: usize = 0;
    const Q: usize = 1;
    const PUB: usize = 2;
    const CLOSED: usize = 3;
    const SNAP: usize = 4;
    const GOT: usize = 5;
    const EOF: usize = 6;
    act(
        &mut m,
        "PublishOneTerminalBeforeProducerClose",
        all([eq(PUB, 0), eq(CLOSED, 0)]),
        &[(PUB, Int(1)), (Q, Int(1))],
    );
    act(
        &mut m,
        "ReleaseClosedAfterTerminalEnqueue",
        all([eq(PUB, 1), eq(CLOSED, 0)]),
        &[(CLOSED, Int(1))],
    );
    for (flag, name) in [(7, "ObserveTransportFault"), (8, "ObserveExplicitDeadline")] {
        act(
            &mut m,
            name,
            eq(CLOSED, 0),
            &[(flag, Int(1)), (CLOSED, Int(1))],
        );
    }
    let capture = m.actions.len();
    act(
        &mut m,
        "CaptureClosedBeforeAnyQueueObservation",
        all([eq(PH, 0), eq(EOF, 0)]),
        &[(SNAP, Var(CLOSED)), (PH, Int(1))],
    );
    let dequeue = m.actions.len();
    act(
        &mut m,
        "DequeueTerminalBeforeConsideringEof",
        all([eq(PH, 1), eq(Q, 1)]),
        &[(Q, Int(0)), (GOT, Int(1)), (PH, Int(0))],
    );
    let empty = m.actions.len();
    act(
        &mut m,
        "ObserveEmptyQueue",
        all([eq(PH, 1), eq(Q, 0)]),
        &[(PH, Int(2))],
    );
    let selected = if control == 1 { CLOSED } else { SNAP };
    let eof = m.actions.len();
    act(
        &mut m,
        "DecideEofFromObservedClose",
        all([eq(PH, 2), eq(selected, 1)]),
        &[(PH, Int(3)), (EOF, Int(1))],
    );
    let again = m.actions.len();
    act(
        &mut m,
        "ReturnEmptyAndObserveAgainAfterNotification",
        all([eq(PH, 2), eq(selected, 0)]),
        &[(PH, Int(0))],
    );
    safe(
        &mut m,
        "EofNeverHidesQueuedDurableTerminal",
        eq(EOF, 0).or(all([eq(Q, 0), Var(GOT).eq(Var(PUB))])),
    );
    safe(
        &mut m,
        "TerminalDeliveryRequiresItsSinglePublication",
        eq(GOT, 0).or(all([eq(PUB, 1), eq(Q, 0)])),
    );
    progress(
        &mut m,
        "ClosedSubscriberEventuallyDrainsAndReturnsEof",
        eq(CLOSED, 1),
        eq(EOF, 1),
        vec![capture, dequeue, empty, eof, again],
    );
    m
}

// Origin: https://github.com/bboxy450/bboxy450-harness
// Retain accurate reuse credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
/// One fixed session and one existing Unix writer. The local face can observe
/// the published close while the remote write is held. Admission sealing is
/// separate from publication/fault. The existing drain deadline is an explicit
/// action, not a numeric OS-time theorem; external writer release is not fair.
pub fn close_retirement(control: u8) -> Model {
    let name = match control {
        1 => "DreamReadCloseRetirementEagerCancel",
        2 => "DreamReadCloseRetirementSealedEof",
        _ => "DreamReadCloseRetirement",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("terminalPublished", 1, 0),
            ("admissionSealed", 1, 0),
            ("actorFaulted", 1, 0),
            ("lifecycleObserverEof", 1, 0),
            ("localFaceObservedTerminal", 1, 0),
            ("terminalQueuedForUnixWriter", 1, 0),
            ("unixTerminalWriteHeld", 1, 0),
            ("externalWriterReleased", 1, 0),
            ("terminalWritten", 1, 0),
            ("remoteObservedTerminal", 1, 0),
            ("callerRequestedClose", 1, 0),
            ("serverDraining", 1, 0),
            ("connectionCancelled", 1, 0),
            ("explicitDrainDeadlineExpired", 1, 0),
            ("serverRetired", 1, 0),
            ("lifecycleStartedRetirement", 1, 0),
        ],
    );
    const PUB: usize = 0;
    const SEAL: usize = 1;
    const FAULT: usize = 2;
    const LIFE: usize = 3;
    const LOCAL: usize = 4;
    const Q: usize = 5;
    const HELD: usize = 6;
    const RELEASE: usize = 7;
    const WRITTEN: usize = 8;
    const REMOTE: usize = 9;
    const CALL: usize = 10;
    const DRAIN: usize = 11;
    const CANCEL: usize = 12;
    const EXPIRED: usize = 13;
    const RETIRED: usize = 14;
    const LIFE_RETIRE: usize = 15;
    act(
        &mut m,
        "SealAdmissionBeforeJoiningServices",
        eq(SEAL, 0),
        &[(SEAL, Int(1))],
    );
    act(
        &mut m,
        "PublishTerminalExactlyOnceAfterServiceJoin",
        all([
            eq(SEAL, 1),
            eq(PUB, 0),
            eq(FAULT, 0),
            eq(CANCEL, 0),
            eq(RETIRED, 0),
        ]),
        &[(PUB, Int(1)), (Q, Int(1))],
    );
    act(
        &mut m,
        "ObserveActorFaultWithoutInventingPublication",
        all([eq(PUB, 0), eq(FAULT, 0), eq(RETIRED, 0)]),
        &[(FAULT, Int(1))],
    );
    act(
        &mut m,
        "LifecycleObserverEndsOrLags",
        all([eq(LIFE, 0), eq(RETIRED, 0)]),
        &[(LIFE, Int(1))],
    );
    let resubscribe = m.actions.len();
    act(
        &mut m,
        "ResubscribeEndedObserverAtCurrentHeadWhileStillLive",
        all([eq(LIFE, 1), eq(PUB, 0), eq(FAULT, 0), eq(RETIRED, 0)]),
        &[(LIFE, Int(0))],
    );
    act(
        &mut m,
        "LifecycleRetirementRequiresPublicationOrActualFault",
        all([
            eq(LIFE, 1),
            eq(LIFE_RETIRE, 0),
            eq(RETIRED, 0),
            eq(PUB, 1).or(eq(FAULT, 1)).or(if control == 2 {
                eq(SEAL, 1)
            } else {
                Bool(false)
            }),
        ]),
        &[(LIFE_RETIRE, Int(1)), (DRAIN, Int(1)), (LIFE, Int(0))],
    );
    let face = m.actions.len();
    act(
        &mut m,
        "LocalFaceObservesPublishedTerminal",
        all([eq(PUB, 1), eq(LOCAL, 0)]),
        &[(LOCAL, Int(1))],
    );
    act(
        &mut m,
        "UnixDequeuesTerminalButWriteRemainsHeld",
        all([eq(Q, 1), eq(CANCEL, 0)]),
        &[(Q, Int(0)), (HELD, Int(1))],
    );
    act(
        &mut m,
        "ExternalWriterReleaseHasNoFairnessAssumption",
        all([eq(HELD, 1), eq(RELEASE, 0), eq(CANCEL, 0)]),
        &[(RELEASE, Int(1))],
    );
    let write = m.actions.len();
    act(
        &mut m,
        "WriteWholeTerminalFrameBeforeConnectionEnd",
        all([eq(HELD, 1), eq(RELEASE, 1), eq(CANCEL, 0), eq(WRITTEN, 0)]),
        &[(WRITTEN, Int(1)), (HELD, Int(0))],
    );
    let remote = m.actions.len();
    act(
        &mut m,
        "RemoteObservesAlreadyWrittenTerminal",
        all([eq(WRITTEN, 1), eq(REMOTE, 0)]),
        &[(REMOTE, Int(1))],
    );
    act(
        &mut m,
        "CallerClosesStillUnpublishedServerImmediately",
        all([eq(CALL, 0), eq(PUB, 0), eq(RETIRED, 0)]),
        &[(CALL, Int(1)), (CANCEL, Int(1))],
    );
    act(
        &mut m,
        "CallerCloseAfterActualLocalTerminalObservation",
        all([eq(CALL, 0), eq(LOCAL, 1), eq(RETIRED, 0)]),
        &[
            (CALL, Int(1)),
            (DRAIN, Int(1)),
            (CANCEL, Int(i64::from(control == 1))),
        ],
    );
    let complete = m.actions.len();
    act(
        &mut m,
        "CompleteDrainAfterWrittenTerminalOrUnpublishedActorFault",
        all([
            eq(DRAIN, 1),
            eq(RETIRED, 0),
            eq(WRITTEN, 1).or(all([eq(PUB, 0), eq(FAULT, 1)])),
        ]),
        &[(DRAIN, Int(0)), (CANCEL, Int(1)), (RETIRED, Int(1))],
    );
    let expire = m.actions.len();
    act(
        &mut m,
        "ExistingDrainDeadlineExpiresWhileWriterMayRemainHeld",
        all([eq(DRAIN, 1), eq(RETIRED, 0)]),
        &[
            (EXPIRED, Int(1)),
            (DRAIN, Int(0)),
            (CANCEL, Int(1)),
            (RETIRED, Int(1)),
        ],
    );
    let immediate = m.actions.len();
    act(
        &mut m,
        "RetireExplicitlyCancelledUnpublishedServer",
        all([eq(CALL, 1), eq(PUB, 0), eq(CANCEL, 1), eq(RETIRED, 0)]),
        &[(DRAIN, Int(0)), (RETIRED, Int(1))],
    );
    safe(
        &mut m,
        "PublishedClosePreservesWriterUntilWriteFaultOrDrainDeadline",
        eq(PUB, 0)
            .or(eq(CANCEL, 0))
            .or(eq(WRITTEN, 1))
            .or(eq(FAULT, 1))
            .or(eq(EXPIRED, 1)),
    );
    safe(
        &mut m,
        "LifecycleEofNeedsPublicationOrActualActorFault",
        eq(LIFE_RETIRE, 0).or(eq(PUB, 1)).or(eq(FAULT, 1)),
    );
    safe(
        &mut m,
        "WireObservationRequiresCompletedTerminalWrite",
        eq(REMOTE, 0).or(eq(WRITTEN, 1)),
    );
    safe(
        &mut m,
        "RetiredServerHasRevokedConnectionGrants",
        eq(RETIRED, 0).or(eq(CANCEL, 1)),
    );
    progress(
        &mut m,
        "StartedDrainEventuallyRetires",
        eq(DRAIN, 1),
        eq(RETIRED, 1),
        vec![complete, expire],
    );
    progress(
        &mut m,
        "WrittenTerminalEventuallyReachesRemoteObserver",
        eq(WRITTEN, 1),
        eq(REMOTE, 1),
        vec![remote],
    );
    // Fairness covers only local observation/drain. It neither releases an
    // externally held writer nor requires the actor/environment to publish.
    m.weak_fair.push(vec![resubscribe]);
    m.weak_fair.push(vec![face]);
    m.weak_fair.push(vec![write]);
    m.weak_fair.push(vec![immediate]);
    m
}

#[cfg(test)]
mod close_delivery_boundary_tests {
    use super::*;

    fn checked(model: Model) -> bbt_tlc::Report {
        model.check(100_000).unwrap()
    }

    #[test]
    fn close_receive_preserves_the_held_empty_publish_close_order() {
        let good = checked(close_receive(0));
        println!(
            "HarnessDreamReadCloseReceive: {} states, {} labelled edges",
            good.states.len(),
            good.edges.iter().map(Vec::len).sum::<usize>()
        );
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states
                .iter()
                .any(|s| s[0] == 2 && s[1] == 1 && s[3] == 1 && s[4] == 0)
        );
        assert!(good.states.iter().any(|s| s[5] == 1 && s[6] == 1));
        assert!(good.states.iter().any(|s| s[7] == 1 && s[6] == 1));
        assert!(good.states.iter().any(|s| s[8] == 1 && s[6] == 1));
        let bad = checked(close_receive(1));
        println!(
            "BrokenHarnessDreamReadCloseReceiveLateClosedRead: {} states, {} labelled edges",
            bad.states.len(),
            bad.edges.iter().map(Vec::len).sum::<usize>()
        );
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == "EofNeverHidesQueuedDurableTerminal" && v.cycle.is_empty())
        );
        assert!(
            bad.states
                .iter()
                .any(|s| s[1] == 1 && s[4] == 0 && s[6] == 1)
        );
    }

    #[test]
    fn close_retirement_preserves_held_writer_and_distinguishes_admission_seal() {
        let good = checked(close_retirement(0));
        println!(
            "HarnessDreamReadCloseRetirement: {} states, {} labelled edges",
            good.states.len(),
            good.edges.iter().map(Vec::len).sum::<usize>()
        );
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(good.states.iter().any(|s| s[4] == 1
            && s[6] == 1
            && s[10] == 1
            && s[11] == 1
            && s[12] == 0));
        assert!(good.states.iter().any(|s| s[9] == 1 && s[14] == 1));
        assert!(
            good.states
                .iter()
                .any(|s| s[6] == 1 && s[13] == 1 && s[14] == 1)
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[0] == 0 && s[2] == 1 && s[14] == 1)
        );
        for (control, invariant) in [
            (
                1,
                "PublishedClosePreservesWriterUntilWriteFaultOrDrainDeadline",
            ),
            (2, "LifecycleEofNeedsPublicationOrActualActorFault"),
        ] {
            let model = close_retirement(control);
            let name = model.name.clone();
            let bad = checked(model);
            println!(
                "{name}: {} states, {} labelled edges",
                bad.states.len(),
                bad.edges.iter().map(Vec::len).sum::<usize>()
            );
            assert!(
                bad.violations
                    .iter()
                    .any(|v| v.property == invariant && v.cycle.is_empty())
            );
        }
    }
}


#[cfg(test)]
mod qualification_observer_value_equivalence {
    use super::*;

    #[test]
    fn original_and_selected_value_guard_keep_all_42_graphs_named_verdicts_and_complete_traces() {
        for control in 0..=41 {
            let original_model = qualification_observer(control, true);
            let value_model = qualification(control);
            assert_eq!(original_model.name, value_model.name);
            assert_eq!(
                format!("{:?}", original_model.variables),
                format!("{:?}", value_model.variables),
                "control{control} variables and bounds"
            );
            assert_eq!(
                format!("{:?}", original_model.invariants),
                format!("{:?}", value_model.invariants),
                "control{control} every safety clause"
            );
            assert_eq!(
                format!("{:?}", original_model.responses),
                format!("{:?}", value_model.responses),
                "control{control} every liveness clause"
            );
            assert_eq!(original_model.weak_fair, value_model.weak_fair);
            assert_eq!(original_model.check_deadlock, value_model.check_deadlock);
            assert_eq!(original_model.actions.len(), value_model.actions.len());
            for (original, value) in original_model.actions.iter().zip(&value_model.actions) {
                assert_eq!(original.name, value.name);
                assert_eq!(format!("{:?}", original.updates), format!("{:?}", value.updates));
            }
            let original = original_model.check(100_000).unwrap();
            let value = value_model.check(100_000).unwrap();
            assert_eq!(original.states, value.states, "control{control} complete states");
            assert_eq!(original.edges.len(), value.edges.len());
            for (a, b) in original.edges.iter().zip(&value.edges) {
                assert_eq!(
                    a.iter().map(|e| (e.action, e.to)).collect::<Vec<_>>(),
                    b.iter().map(|e| (e.action, e.to)).collect::<Vec<_>>(),
                    "control{control} complete labelled edges"
                );
            }
            assert_eq!(
                original.violations.iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle)).collect::<Vec<_>>(),
                value.violations.iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle)).collect::<Vec<_>>(),
                "control{control} named safety/liveness verdicts and complete prefix/cycle traces"
            );
            assert!(
                original.violations.iter().all(|v| v.property != "TypeOK"),
                "control{control} TypeOK is never a qualifying negative"
            );
            println!(
                "qualification_observer_equivalence control={control} model={} states={} labelled_edges={} named_verdicts={:?}",
                original_model.name,
                original.states.len(),
                original.edges.iter().map(Vec::len).sum::<usize>(),
                original.violations.iter().map(|v| &v.property).collect::<Vec<_>>()
            );
        }
    }
}
