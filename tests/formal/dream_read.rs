// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Finite Q2 producer contracts. Evidence facts abstract existing verifiers;
//! bytes, signatures, bounded process IO and allocator behavior remain witnesses.
use super::*;

/// Decision66050/66074 raw failed foreground/final signal and SAME original
/// physical custody. No external helper window or per-grant CPU is modeled.
/// The controller/pipe/retirement fields describe its OWN65086 ordinary unit:
/// complete measured cutoff, inclusive CPU<=1ms and fresh9x debt remain hard.
/// External CPU/caps/debt are HelperShimBudget. No native success is fair.
pub fn failed_stop_terminal(control: u8) -> Model {
    let suffix = ["", "ErrnoAck", "RetryStop", "Resume", "LeaderOnly", "Roster",
        "Identity", "FailedEndpoint", "Controller", "Pipe", "Late", "Quantum",
        "Clock", "Debt", "Reap", "UnknownExit", "WorkWhilePending", "PayloadWhilePending"][control as usize];
    let mut m = base(&format!("DreamReadFailedStop{suffix}"), control != 0, &[
        ("phase", 6, 0), ("profile", 13, 0), ("rawErrnoEpermEsrch", 2, 0),
        ("consumedStopAttempts", 2, 0), ("leaderWnowaitOriginal", 1, 0),
        ("everyMemberPositiveExit", 1, 0), ("causallyClosedRoster", 1, 0),
        ("exactRetainedIncarnation", 1, 0), ("ownControllerUnitEndFailedOrMeasured", 2, 0),
        ("controllerIncluded", 1, 0), ("pipeIncluded", 1, 0),
        ("ownRetirementObservationCostIncluded", 1, 0), ("ownControllerUnitWithinOrOver1ms", 2, 0),
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
    // token/exec, unclosed own-unit cutoff, controller/pipe/retirement cost omissions, own-unit overrun, clock and unavailable reap.
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

/// One pre-final-receipt audit cutoff, independent from the existing wire schema.
/// Three bounded repeated-control counts and three persistence/tail outcomes are
/// observed independently. Final receipt IO/durability and process exit remain
/// implementation witnesses; this projection never releases custody or qualifies.
pub fn private_verdicts(control: u8) -> Model {
    let suffix = [
        "",
        "FirstErrorOnly",
        "Persistence",
        "EarlyTerminal",
        "HeldFdLocator",
        "TerminalWorkAdmission",
        "DeadlineSuppressReport",
    ][usize::from(control)];
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
    const INPUT_CASE: usize = 16;
    const HELD_FD: usize = 17;
    const LOCATOR: usize = 18;
    const FENCE: usize = 19;
    const WORK_DEADLINE: usize = 20;
    const REPORTING: usize = 21;
    const NEXT_ENGINE: usize = 22;
    const FORMAT_ADMISSION: usize = 23;
    const RESULT_ATTEMPT: usize = 24;
    const STDOUT_OUTCOME: usize = 25;
    let mut m = base(
        &format!("DreamReadPrivateVerdicts{suffix}"),
        control != 0,
        &[
            ("phase", 5, 0),
            ("actualControlFailureCount", 2, 0),
            ("actualPersistenceOutcome", 3, 0),
            ("actualTerminalOutcome", 3, 0),
            ("controlVerdict", 2, 0),
            ("persistenceVerdict", 3, 0),
            ("terminalCostVerdict", 3, 0),
            ("retainedControlFailureCount", 2, 0),
            ("retainedPersistenceFailure", 1, 0),
            ("retainedTerminalFailure", 1, 0),
            ("preFinalReceiptTailClosed", 1, 0),
            ("retainedTailCostBound", 3, 0),
            ("retainedRawUnknown", 1, 0),
            ("freshTailDebt", 27, 0),
            ("tailDebtActuallyPaid", 1, 0),
            ("finalAuditAttempted", 1, 0),
            ("immutableInputCaseCurrentLocatorChangedHeldMutated", 3, 0),
            ("retainedFdAndOriginalStampMatch", 1, 0),
            ("pathnameLocatorStillMatches", 1, 0),
            ("descriptorFenceUnsetPassRefuse", 2, 0),
            ("originalWorkDeadlineExpired", 1, 0),
            ("terminalReportingOnlyDisposition", 1, 0),
            ("nextEngineCommandAdmittedAfterTerminalDisposition", 1, 0),
            (
                "formatterAdmissionUnsetKnownBoundedPaidOverrunUnknown",
                3,
                0,
            ),
            ("terminalResultAttempted", 1, 0),
            ("stdoutOutcomeUnobservedCompleteFailedUnknown", 3, 0),
        ],
    );
    // Strict decode+expected hash have already succeeded. These immutable
    // cases separate retained descriptor authority from a later path lookup:
    // matching FD/path; same held FD with replaced locator after deadline;
    // in-place held-file mutation despite a matching locator after deadline.
    for input in 1..=3 {
        act(
            &mut m,
            &format!("RetainDecodedOriginalDescriptorCase{input}"),
            all([eq(PH, 0), eq(INPUT_CASE, 0)]),
            &[
                (INPUT_CASE, Int(input)),
                (HELD_FD, Int(i64::from(input != 3))),
                (LOCATOR, Int(i64::from(input != 2))),
                (WORK_DEADLINE, Int(i64::from(input != 1))),
            ],
        );
    }
    for current in 0..=1 {
        act(
            &mut m,
            &format!("FenceSameRetainedDescriptorStamp{current}"),
            all([
                eq(PH, 1),
                eq(FENCE, 0),
                eq(if control == 4 { LOCATOR } else { HELD_FD }, current),
            ]),
            &[(FENCE, Int(if current == 1 { 1 } else { 2 }))],
        );
    }
    let terminal_mode = m.actions.len();
    act(
        &mut m,
        "EnterTerminalOnlyReportingAfterDispatchDisposition",
        all([eq(PH, 2), eq(REPORTING, 0)]),
        &[(REPORTING, Int(1))],
    );
    for admission in 1..=3 {
        // Admission1 abstracts an actually measured <=1ms formatter whose
        // full current cost/rest is already paid;2/3 retain overrun/UNKNOWN.
        // No cost/clock success is fair and unknown never supplies this fact.
        act(
            &mut m,
            &format!("ObserveActualFormatterAdmission{admission}"),
            all([
                eq(PH, 2),
                eq(REPORTING, 1),
                eq(FORMAT_ADMISSION, 0),
                if admission == 1 {
                    Bool(true)
                } else {
                    Int(0).lt(Var(EXPECT_CONTROL))
                },
            ]),
            &[(FORMAT_ADMISSION, Int(admission))],
        );
    }
    let attempt = m.actions.len();
    act(
        &mut m,
        "AttemptBoundedPaidTerminalResultDespiteExpiredWorkDeadline",
        all([
            eq(PH, 2),
            eq(REPORTING, 1),
            eq(FORMAT_ADMISSION, 1),
            eq(RESULT_ATTEMPT, 0),
            if control == 6 {
                eq(WORK_DEADLINE, 0)
            } else {
                Bool(true)
            },
        ]),
        &[(RESULT_ATTEMPT, Int(1))],
    );
    for outcome in 1..=3 {
        act(
            &mut m,
            &format!("ObserveIndependentStdoutOutcome{outcome}"),
            all([
                eq(PH, 2),
                eq(RESULT_ATTEMPT, 1),
                eq(STDOUT_OUTCOME, 0),
                if outcome == 1 {
                    Bool(true)
                } else {
                    Int(0).lt(Var(EXPECT_CONTROL))
                },
            ]),
            &[(STDOUT_OUTCOME, Int(outcome))],
        );
    }
    if control == 5 {
        act(
            &mut m,
            "BrokenTerminalDispositionAdmitsNextEngineCommand",
            all([eq(REPORTING, 1), eq(NEXT_ENGINE, 0)]),
            &[(NEXT_ENGINE, Int(1))],
        );
    }
    for c in 0..=2 {
        for p in 1..=3 {
            for t in 1..=3 {
                act(
                    &mut m,
                    &format!("SelectIndependentOutcomes{c}_{p}_{t}"),
                    all([
                        eq(PH, 0),
                        Int(0).lt(Var(INPUT_CASE)),
                        if c == 0 {
                            eq(INPUT_CASE, 1)
                        } else {
                            Bool(true)
                        },
                    ]),
                    &[
                        (PH, Int(1)),
                        (EXPECT_CONTROL, Int(c)),
                        (EXPECT_PERSIST, Int(p)),
                        (EXPECT_TAIL, Int(t)),
                    ],
                );
            }
        }
        act(
            &mut m,
            &format!("RetainRepeatedControlFailures{c}"),
            all([eq(PH, 1), eq(EXPECT_CONTROL, c), Int(0).lt(Var(FENCE))]),
            &[
                (PH, Int(2)),
                (CONTROL, Int(if c == 0 { 1 } else { 2 })),
                (CONTROL_COUNT, Int(if control == 1 { c.min(1) } else { c })),
            ],
        );
        for p in 1..=3 {
            let persistence_error = i64::from(p != 1);
            act(
                &mut m,
                &format!("ObserveActualPublicationOutcome{c}_{p}"),
                all([
                    eq(PH, 2),
                    eq(EXPECT_CONTROL, c),
                    eq(EXPECT_PERSIST, p),
                    eq(REPORTING, 1),
                    all([
                        eq(FORMAT_ADMISSION, 1),
                        eq(RESULT_ATTEMPT, 1),
                        Int(0).lt(Var(STDOUT_OUTCOME)),
                    ])
                    .or(all([Int(1).lt(Var(FORMAT_ADMISSION)), Bool(p != 1)])),
                ]),
                &[
                    (PH, Int(3)),
                    (PERSIST, Int(if control == 2 { 1 } else { p })),
                    (
                        PERSIST_ERROR,
                        Int(if control == 1 && c != 0 {
                            0
                        } else {
                            persistence_error
                        }),
                    ),
                ],
            );
            for t in 1..=3 {
                // Outcome1 is a within-unit bound,2 a genuine overrun,3 raw
                // unknown with the conservative3/27 obligation and no paid claim.
                let cost = t;
                let unknown = i64::from(t == 3);
                let terminal_error = i64::from(t != 1);
                act(
                    &mut m,
                    &format!("CloseAllExitTailWork{c}_{p}_{t}"),
                    all([
                        eq(PH, 3),
                        eq(EXPECT_CONTROL, c),
                        eq(EXPECT_PERSIST, p),
                        eq(EXPECT_TAIL, t),
                    ]),
                    &[
                        (PH, Int(4)),
                        (TAIL, Int(t)),
                        (TAIL_CLOSED, Int(1)),
                        (COST, Int(cost)),
                        (RAW_UNKNOWN, Int(unknown)),
                        (DEBT, Int(9 * cost)),
                        (PAID, Int(0)),
                        (
                            TAIL_ERROR,
                            Int(if control == 1 && (c != 0 || p != 1) {
                                0
                            } else {
                                terminal_error
                            }),
                        ),
                    ],
                );
            }
        }
    }
    let pay = m.actions.len();
    act(
        &mut m,
        "ActuallyPayFullKnownClosedTailDebt",
        all([eq(PH, 4), eq(RAW_UNKNOWN, 0), eq(PAID, 0)]),
        &[(DEBT, Int(0)), (PAID, Int(1))],
    );
    let audit = m.actions.len();
    act(
        &mut m,
        "AttemptFinalRawAuditAfterClosedTailCutoff",
        all([eq(PH, 4), eq(PAID, 1).or(eq(RAW_UNKNOWN, 1))]),
        &[(PH, Int(5)), (AUDIT, Int(1))],
    );
    if control == 3 {
        act(
            &mut m,
            "BrokenFinalRawAuditBeforeExitTail",
            eq(PH, 3),
            &[(PH, Int(5)), (AUDIT, Int(1))],
        );
    }
    safe(
        &mut m,
        "EveryIndependentFailureRemainsRecorded",
        all([
            eq(CONTROL, 0).or(Var(CONTROL_COUNT).eq(Var(EXPECT_CONTROL))),
            eq(PERSIST, 0).or(eq(EXPECT_PERSIST, 1).and(eq(PERSIST_ERROR, 0)).or(eq(
                EXPECT_PERSIST,
                1,
            )
            .negate()
            .and(eq(PERSIST_ERROR, 1)))),
            eq(TAIL_CLOSED, 0).or(eq(EXPECT_TAIL, 1)
                .and(eq(TAIL_ERROR, 0))
                .or(eq(EXPECT_TAIL, 1).negate().and(eq(TAIL_ERROR, 1)))),
        ]),
    );
    safe(
        &mut m,
        "PersistenceVerdictMatchesActualAttempt",
        eq(PERSIST, 0).or(Var(PERSIST).eq(Var(EXPECT_PERSIST))),
    );
    safe(
        &mut m,
        "TerminalReceiptFollowsAllTailCost",
        eq(AUDIT, 0).or(all([eq(TAIL_CLOSED, 1), Int(0).lt(Var(TAIL))])),
    );
    for t in 1..=3 {
        safe(
            &mut m,
            &format!("ClosedTailRetainsFullCostAndFreshDebt{t}"),
            all([eq(TAIL_CLOSED, 1), eq(EXPECT_TAIL, t)])
                .negate()
                .or(all([
                    eq(COST, t),
                    eq(RAW_UNKNOWN, i64::from(t == 3)),
                    eq(PAID, 0)
                        .and(eq(DEBT, 9 * t))
                        .or(Bool(t != 3).and(eq(PAID, 1)).and(eq(DEBT, 0))),
                ])),
        );
    }
    safe(
        &mut m,
        "DescriptorFenceMatchesSameRetainedFd",
        eq(FENCE, 0)
            .or(eq(FENCE, 1).and(eq(HELD_FD, 1)))
            .or(eq(FENCE, 2).and(eq(HELD_FD, 0))),
    );
    safe(
        &mut m,
        "TerminalReportingCannotAdmitNextEngineWork",
        eq(NEXT_ENGINE, 0),
    );
    safe(
        &mut m,
        "ResultAttemptRequiresKnownBoundedPaidFormatter",
        eq(RESULT_ATTEMPT, 0).or(all([eq(REPORTING, 1), eq(FORMAT_ADMISSION, 1)])),
    );
    safe(
        &mut m,
        "StdoutCompletionRequiresActualResultAttempt",
        eq(STDOUT_OUTCOME, 1).negate().or(eq(RESULT_ATTEMPT, 1)),
    );
    progress(
        &mut m,
        "PaidTerminalRefusalAttemptsResultDespiteWorkDeadline",
        all([
            eq(PH, 2),
            eq(REPORTING, 1),
            eq(WORK_DEADLINE, 1),
            eq(FORMAT_ADMISSION, 1),
        ]),
        eq(RESULT_ATTEMPT, 1),
        vec![attempt],
    );
    m.weak_fair.push(vec![terminal_mode]);
    m.weak_fair.push(vec![pay]);
    m.weak_fair.push(vec![audit]);
    m
}

#[cfg(test)]
mod private_verdict_tests {
    use super::*;
    #[test]
    fn independent_failure_axes_and_final_tail_cutoff_export_all_controls() {
        let output =
            std::env::var_os("DREAM_PRIVATE_VERDICT_FORMAL_OUT").map(std::path::PathBuf::from);
        let mut manifest = String::new();
        for control in 0..=6 {
            let model = private_verdicts(control);
            let graph = model.check(100_000).unwrap();
            let properties: std::collections::BTreeSet<_> = graph
                .violations
                .iter()
                .map(|v| v.property.as_str())
                .collect();
            let expected: std::collections::BTreeSet<_> = match control {
                1 => ["EveryIndependentFailureRemainsRecorded"]
                    .into_iter()
                    .collect(),
                2 => ["PersistenceVerdictMatchesActualAttempt"]
                    .into_iter()
                    .collect(),
                3 => ["TerminalReceiptFollowsAllTailCost"].into_iter().collect(),
                4 => ["DescriptorFenceMatchesSameRetainedFd"]
                    .into_iter()
                    .collect(),
                5 => ["TerminalReportingCannotAdmitNextEngineWork"]
                    .into_iter()
                    .collect(),
                6 => ["PaidTerminalRefusalAttemptsResultDespiteWorkDeadline"]
                    .into_iter()
                    .collect(),
                _ => std::collections::BTreeSet::new(),
            };
            assert_eq!(properties, expected);
            if control == 6 {
                assert!(graph.violations.iter().any(|v| !v.cycle.is_empty()));
            } else {
                assert!(graph.violations.iter().all(|v| v.cycle.is_empty()));
            }
            if control == 0 {
                for c in 0..=2 {
                    for p in 1..=3 {
                        for t in 1..=3 {
                            assert!(
                                graph
                                    .states
                                    .iter()
                                    .any(|s| s[1] == c && s[2] == p && s[3] == t && s[15] == 1),
                                "each independent repeated-control/persistence/tail class reaches its final audit attempt"
                            );
                        }
                    }
                }
                assert!(
                    graph.states.iter().any(|s| s[15] == 1
                        && s[1] == 2
                        && s[7] == 2
                        && s[5] == 2
                        && s[8] == 1
                        && s[6] == 2
                        && s[9] == 1),
                    "an earlier control error cannot hide failed persistence or an over-budget exit tail"
                );
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[15] == 1 && s[12] == 1 && s[13] == 27 && s[14] == 0),
                    "unknown tail cost remains raw unknown and unpaid at the explicit audit cutoff"
                );
            }
            if control == 0 {
                assert!(
                    graph.states.iter().any(|s| s[16] == 2
                        && s[17] == 1
                        && s[18] == 0
                        && s[19] == 1
                        && s[20] == 1
                        && s[21] == 1
                        && s[22] == 0
                        && s[23] == 1
                        && s[24] == 1
                        && s[25] == 1
                        && s[5] == 1
                        && s[15] == 1),
                    "expired work deadline and replaced locator preserve same held-FD fence and paid refusal result/stdout/persistence attempt"
                );
                assert!(
                    graph.states.iter().any(|s| s[16] == 3
                        && s[17] == 0
                        && s[18] == 1
                        && s[19] == 2
                        && s[24] == 1
                        && s[15] == 1),
                    "in-place held descriptor mutation refuses source admission while terminal reporting retains the refusal"
                );
                for format in [2, 3] {
                    assert!(
                        graph
                            .states
                            .iter()
                            .any(|s| s[23] == format && s[24] == 0 && s[15] == 1),
                        "overrun/unknown formatter cannot claim a result even when final raw audit is attempted"
                    );
                }
                assert!(
                    graph
                        .states
                        .iter()
                        .any(|s| s[24] == 1 && s[25] == 2 && s[5] == 1),
                    "persisted file does not turn a failed stdout write into a completed result"
                );
            }
            println!(
                "private_verdicts control={control} model={} states={} labelled_edges={} named_verdicts={properties:?}",
                model.name,
                graph.states.len(),
                graph.edges.iter().map(Vec::len).sum::<usize>()
            );
            if let Some(root) = &output {
                let models = root.join("models");
                std::fs::create_dir_all(&models).unwrap();
                super::helper_observer_lowering_export::export(
                    model,
                    if control == 0 {
                        "pass"
                    } else if control == 6 {
                        "liveness"
                    } else {
                        "safety"
                    },
                    &models,
                    &mut manifest,
                )
                .unwrap();
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

/// Independent65086/65126 admission, native effect and original custody model.
/// The external helper never becomes an own-thread quantum. Native birth
/// 2ms CPU/250ms prepared-process wall/3ms inclusive under66331; every
/// other native class remains25ms. Ordinary controller/application1ms and
/// own9x debt stay; helper terminal CPU/debt is HelperShimBudget separately.
/// Attempt0/1/2 classes represent empty/near256/exhausted consumed capacity.
pub fn helper(control: u8) -> Model {
    helper_observer(control, false)
}
fn helper_observer(control: u8, full_observer: bool) -> Model {
    let suffix = match control {
        0 => "",
        1 => "Attempt",
        2 => "Member",
        3 => "Executable",
        4 => "Aba",
        9 => "MainOverlap",
        10 => "DebtReset",
        11 => "ApplicationLabel",
        12 => "NativeCpu",
        13 => "NativeWall",
        14 => "NativeInclusive",
        16 => "EffectError",
        17 => "Coverage",
        18 => "EarlyTrust",
        19 => "ClockRelease",
        20 => "PanicRelease",
        21 => "RevokeRelease",
        22 => "StopRetry",
        23 => "OldProcessCreateWall",
        _ => panic!("superseded external-grant control"),
    };
    const P: usize = 0;
    const INC: usize = 1;
    const CAP: usize = 2;
    const ATT: usize = 3;
    const RESERVED: usize = 4;
    const EFFECT: usize = 5;
    const EFFECTS: usize = 6;
    const EXEC: usize = 7;
    const CASE: usize = 8;
    const CERT: usize = 9;
    const DEBT: usize = 10;
    const OWED: usize = 11;
    const LABEL: usize = 12;
    const SPAN: usize = 13;
    const OWNER: usize = 14;
    const MEMBERS: usize = 15;
    const EXTRA: usize = 16;
    const COVER: usize = 17;
    const TRUST: usize = 18;
    const RETIRED: usize = 19;
    const REAP: usize = 20;
    const GONE: usize = 21;
    const RELEASE: usize = 22;
    const FAULT: usize = 23;
    const MAIN: usize = 24;
    const STOP: usize = 25;
    const RAW: usize = 26;
    const APP: usize = 27;
    const CONFIG: usize = 28;
    const RAW_COVER: usize = 29;
    // Disjoint, single-observation policy projection. Legacy custody fields
    // remain untouched, avoiding a cross product with every lifecycle history.
    const BOUND_CASE: usize = 30;
    const NATIVE_TAG: usize = 31; // 1 prepared process_create,2 any other native
    const WALL_MS: usize = 32;
    const NATIVE_CPU_MS: usize = 33;
    const INCLUSIVE_CPU_MS: usize = 34;
    const BOUND_CLOCK_KNOWN: usize = 35;
    const BOUND_VERDICT: usize = 36;
    const CREATE_EXCESS_MS: usize = 37;
    const CREATE_EXCESS_COUNT: usize = 38;
    const CREATE_EXCESS_KNOWN: usize = 39;
    let mut m = base(
        &format!("DreamReadHelper{suffix}"),
        control != 0,
        &[
            (
                "phaseAbsentReservedBornNativePaidAttachedRetiredReapedQuarantine",
                7,
                0,
            ),
            ("currentIncarnationA1B2A3", 2, 0),
            ("capturedEffectIncarnation", 2, 0),
            ("consumedCreationCapacityClass", 2, 0),
            ("actualCreationReserved", 1, 0),
            ("originalEffectRetained", 1, 0),
            ("actualEffects", 2, 0),
            ("exactExecutableCapability", 1, 1),
            ("inputProfile", 14, 0),
            ("nativeBirthCertified", 1, 0),
            ("ownThreadDebtClass", 3, 0),
            ("observedOwnThreadDebtClass", 3, 0),
            ("labelNoneApplicationNative", 2, 0),
            ("spanNoneNativeApplication", 2, 0),
            ("originalOwnerHeld", 1, 0),
            ("members", 2, 0),
            ("actualUnadmittedMember", 1, 0),
            ("sourceCoverageClaimed", 1, 0),
            ("trusted", 1, 0),
            ("retired", 1, 0),
            ("actualOwnReaped", 1, 0),
            ("actualGroupGone", 1, 0),
            ("ownershipReleased", 1, 0),
            ("faultNoneClockPanicRevokeResourceMemberCoverage", 6, 0),
            ("mainExpensiveWork", 1, 0),
            ("foregroundStopUnusedConsumedRetried", 2, 0),
            ("rawStopNoneOkEpermEsrch", 3, 0),
            ("ordinaryApplicationUsed", 1, 0),
            ("configured", 1, 0),
            ("actualSourceCoverageKnown", 1, 1),
            ("nativeBoundSingleObservationCase", 10, 0),
            ("nativeTagPreparedProcessCreateOther", 2, 0),
            ("nativeWallMillisecondsRepresentative", 251, 0),
            ("nativeCpuMillisecondsRepresentative", 3, 0),
            ("inclusiveCpuMillisecondsRepresentative", 4, 0),
            ("nativeBoundClockKnown", 1, 0),
            ("nativeBoundVerdictUnsetAdmitRefuse", 2, 0),
            ("processCreateWallExcessMilliseconds", 226, 0),
            ("processCreateWallExcessCount", 1, 0),
            ("processCreateWallExcessKnown", 1, 0),
        ],
    );
    for profile in 0..=14 {
        act(
            &mut m,
            &format!("ConfigureIndependentAdmission{profile}"),
            eq(CONFIG, 0),
            &[
                (CONFIG, Int(1)),
                (CASE, Int(profile)),
                (EXEC, Int(i64::from(profile != 9))),
                (
                    ATT,
                    Int(if profile == 10 {
                        1
                    } else if profile == 11 {
                        2
                    } else {
                        0
                    }),
                ),
                (RAW_COVER, Int(i64::from(profile != 12))),
                (EXTRA, Int(i64::from(profile == 8))),
            ],
        );
    }
    let needed = |two: bool| if two { 2 } else { 1 };
    for two in [false, true] {
        act(
            &mut m,
            if two {
                "ReserveSignatureAndImplicitChildBeforeBirth"
            } else {
                "ReserveOneBeforeBirth"
            },
            all([
                eq(CONFIG, 1),
                eq(P, 0),
                if two {
                    eq(CASE, 7).or(eq(CASE, 10))
                } else {
                    eq(CASE, 7).or(eq(CASE, 10)).negate()
                },
                Var(ATT).plus(Int(needed(two))).lt(Int(3)),
            ]),
            &[
                (ATT, Var(ATT).plus(Int(needed(two)))),
                (RESERVED, Int(1)),
                (P, Int(1)),
                (CAP, Var(INC)),
            ],
        );
    }
    act(
        &mut m,
        "NativeBirthRetainsOriginalEffectBeforeVerdict",
        all([eq(P, 1), eq(EXEC, 1), eq(RESERVED, 1), eq(EFFECT, 0)]),
        &[
            (P, Int(2)),
            (EFFECT, Int(1)),
            (EFFECTS, Int(1)),
            (OWNER, Int(1)),
            (MEMBERS, Int(1)),
        ],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenBirthWithoutConsumedReservation",
            all([eq(P, 0), eq(CONFIG, 1), eq(EFFECT, 0)]),
            &[
                (P, Int(2)),
                (EFFECT, Int(1)),
                (EFFECTS, Int(1)),
                (OWNER, Int(1)),
                (MEMBERS, Int(1)),
            ],
        );
    }
    if control == 3 {
        act(
            &mut m,
            "BrokenBirthForeignExecutable",
            all([eq(P, 1), eq(EXEC, 0)]),
            &[
                (P, Int(2)),
                (EFFECT, Int(1)),
                (EFFECTS, Int(1)),
                (OWNER, Int(1)),
                (MEMBERS, Int(1)),
            ],
        );
    }
    for profile in 0..=14 {
        let native_ok = !matches!(profile, 2..=5);
        let certified = native_ok
            || (control == 12 && profile == 2)
            || (control == 13 && profile == 3)
            || (control == 14 && profile == 4);
        let cost = if matches!(profile, 2..=5) { 3 } else { 1 };
        let fault = if profile == 5 {
            1
        } else if matches!(profile, 2..=4) {
            4
        } else if matches!(profile, 6 | 13) {
            2
        } else if profile == 14 {
            3
        } else {
            0
        };
        act(
            &mut m,
            &format!("CloseActualNativeUnitAfterRetainedEffectProfile{profile}"),
            all([eq(P, 2), eq(CASE, profile)]),
            &[
                (P, Int(3)),
                (CERT, Int(i64::from(certified))),
                (DEBT, Int(cost)),
                (OWED, Int(cost)),
                (SPAN, Int(1)),
                (LABEL, Int(2)),
                (FAULT, Int(fault)),
            ],
        );
    }
    let exact = Var(INC).eq(Var(CAP));
    act(
        &mut m,
        "AttachExactOriginalWithoutAnyExternalQuantum",
        all([
            eq(P, 3),
            eq(CERT, 1),
            eq(FAULT, 0),
            eq(DEBT, 0),
            eq(OWED, 0),
            exact.clone(),
        ]),
        &[(P, Int(4)), (COVER, Int(1))],
    );
    act(
        &mut m,
        "ObserveUnadmittedMemberRefuses",
        all([eq(P, 4), eq(CASE, 8), eq(FAULT, 0)]),
        &[(EXTRA, Int(1)), (COVER, Int(0)), (FAULT, Int(5))],
    );
    act(
        &mut m,
        "ObserveMissingCoverageRefuses",
        all([eq(P, 4), eq(CASE, 12), eq(COVER, 1)]),
        &[(COVER, Int(0)), (FAULT, Int(6))],
    );
    for inc in 0..2 {
        act(
            &mut m,
            &format!("ObserveOriginalAuthorityChange{inc}"),
            all([
                eq(OWNER, 1),
                eq(INC, inc),
                Int(2).lt(Var(P)),
                eq(RELEASE, 0),
            ]),
            &[
                (INC, Int(inc + 1)),
                (P, Int(7)),
                (COVER, Int(0)),
                (FAULT, Int(3)),
            ],
        );
    }
    if control == 4 {
        act(
            &mut m,
            "BrokenA3AttachesOldA1Effect",
            all([eq(OWNER, 1), eq(INC, 2), eq(CAP, 0)]),
            &[(P, Int(4))],
        );
    }
    if control == 16 {
        act(
            &mut m,
            "BrokenReplayAfterNativeReportedError",
            all([eq(CASE, 6), eq(EFFECTS, 1), eq(OWNER, 1)]),
            &[(EFFECTS, Int(2))],
        );
    }
    for cost in 1..=2 {
        let mut updates = vec![
            (APP, Int(1)),
            (SPAN, Int(2)),
            (LABEL, Int(if control == 11 { 2 } else { 1 })),
            (DEBT, Int(cost)),
            (OWED, Int(cost)),
        ];
        if cost == 2 && control != 11 {
            updates.extend([(FAULT, Int(4)), (COVER, Int(0))]);
        }
        act(
            &mut m,
            &format!("OneOrdinaryApplicationControllerUnit{cost}"),
            all([
                eq(P, 4),
                eq(APP, 0),
                eq(DEBT, 0),
                eq(OWED, 0),
                if cost == 2 {
                    eq(CASE, 1)
                } else {
                    eq(CASE, 1).negate()
                },
            ]),
            &updates,
        );
    }
    if control == 10 {
        act(
            &mut m,
            "BrokenResetOwnThreadDebt",
            Int(0).lt(Var(DEBT)),
            &[(DEBT, Int(0))],
        );
    }
    for debt in 1..=3 {
        act(
            &mut m,
            &format!("PayOwnNativeApplicationDebt{debt}"),
            all([eq(DEBT, debt), eq(OWED, debt)]),
            &[(DEBT, Int(debt - 1)), (OWED, Int(debt - 1))],
        );
    }
    act(
        &mut m,
        "ConsumeOneForegroundStopBeforeOutcome",
        all([eq(P, 4), eq(STOP, 0)]),
        &[(STOP, Int(1))],
    );
    for raw in 1..=3 {
        act(
            &mut m,
            &format!("RetainRawForegroundStopOutcome{raw}"),
            all([eq(STOP, 1), eq(RAW, 0)]),
            &[(RAW, Int(raw))],
        );
    }
    if control == 22 {
        act(
            &mut m,
            "BrokenRetryConsumedForegroundStop",
            all([eq(STOP, 1), Int(0).lt(Var(RAW))]),
            &[(STOP, Int(2))],
        );
    }
    if control == 9 {
        act(
            &mut m,
            "BrokenMainProofWhileOriginalHelperLives",
            all([eq(OWNER, 1), eq(REAP, 0)]),
            &[(MAIN, Int(1))],
        );
    }
    act(
        &mut m,
        "RetireSameOriginalEffect",
        all([eq(OWNER, 1), eq(RETIRED, 0), Int(2).lt(Var(P))]),
        &[(RETIRED, Int(1)), (P, Int(5))],
    );
    act(
        &mut m,
        "ObserveActualOwnWait4AndGroupGone",
        all([eq(OWNER, 1), eq(RETIRED, 1), eq(REAP, 0)]),
        &[
            (REAP, Int(1)),
            (GONE, Int(1)),
            (MEMBERS, Int(0)),
            (P, Int(6)),
        ],
    );
    act(
        &mut m,
        "ReleaseOnlyPhysicallySettledAndPaid",
        all([
            eq(REAP, 1),
            eq(GONE, 1),
            eq(RETIRED, 1),
            eq(DEBT, 0),
            eq(OWED, 0),
        ]),
        &[(OWNER, Int(0)), (RELEASE, Int(1))],
    );
    act(
        &mut m,
        "TrustOnlyOriginalCurrentCoveredReapedOutput",
        all([
            eq(RELEASE, 1),
            eq(REAP, 1),
            eq(GONE, 1),
            eq(FAULT, 0),
            eq(COVER, 1),
            exact.clone(),
            if control == 2 {
                Bool(true)
            } else {
                eq(EXTRA, 0)
            },
            if control == 17 {
                Bool(true)
            } else {
                eq(RAW_COVER, 1)
            },
        ]),
        &[(TRUST, Int(1))],
    );
    if control == 18 {
        act(
            &mut m,
            "BrokenTrustBeforeOwnReap",
            eq(P, 4),
            &[(TRUST, Int(1))],
        );
    }
    for (mutant, fault) in [(19, 1), (20, 2), (21, 3)] {
        if control == mutant {
            act(
                &mut m,
                "BrokenFaultReleasesOriginalUnreapedEffect",
                all([eq(OWNER, 1), eq(REAP, 0), eq(FAULT, fault)]),
                &[(OWNER, Int(0)), (RELEASE, Int(1))],
            );
        }
    }
    act(
        &mut m,
        "RetainOwnedQuarantineWhenSettlementUnavailable",
        all([eq(OWNER, 1), eq(REAP, 0), Int(2).lt(Var(P))]),
        &[(P, Int(7))],
    );
    safe(
        &mut m,
        "NativeEffectRequiresReservedAttempt",
        eq(EFFECT, 0).or(eq(RESERVED, 1)),
    );
    safe(
        &mut m,
        "NativeEffectRequiresExactExecutableCapability",
        eq(EFFECT, 0).or(eq(EXEC, 1)),
    );
    safe(
        &mut m,
        "AttachedWorkKeepsExactIncarnation",
        eq(P, 4).negate().or(exact),
    );
    safe(
        &mut m,
        "MainProofCannotOverlapRunnableGroup",
        eq(MAIN, 0).or(eq(REAP, 1)),
    );
    safe(
        &mut m,
        "DebtCoversWholeObservedUnrestedSpan",
        Var(DEBT).lt(Var(OWED)).negate(),
    );
    safe(
        &mut m,
        "OrdinaryApplicationCannotUseNativeLabel",
        eq(SPAN, 2).negate().or(eq(LABEL, 2).negate()),
    );
    safe(
        &mut m,
        "CertifiedNativeCreationMeetsCpuWallInclusiveBounds",
        eq(CERT, 0).or(all([
            eq(CASE, 2).negate(),
            eq(CASE, 3).negate(),
            eq(CASE, 4).negate(),
            eq(CASE, 5).negate(),
        ])),
    );
    safe(
        &mut m,
        "OriginalNativeEffectNeverReplayed",
        Var(EFFECTS).lt(Int(2)),
    );
    safe(
        &mut m,
        "TrustedOutputHasOnlyAdmittedMembers",
        eq(TRUST, 0).or(eq(EXTRA, 0)),
    );
    safe(
        &mut m,
        "TrustedOutputHasCompleteCoverage",
        eq(TRUST, 0).or(eq(RAW_COVER, 1)),
    );
    safe(
        &mut m,
        "TrustedOutputHasActualReapAndRetirement",
        eq(TRUST, 0).or(all([eq(REAP, 1), eq(GONE, 1), eq(RETIRED, 1)])),
    );
    safe(
        &mut m,
        "FaultCannotReleaseUnsettledOwnership",
        eq(RELEASE, 0).or(all([eq(REAP, 1), eq(GONE, 1), eq(DEBT, 0), eq(OWED, 0)])),
    );
    safe(
        &mut m,
        "StopAttemptIsNotRetriedAfterAnyNativeOutcome",
        eq(STOP, 2).negate(),
    );
    // Decision66331 changes only prepared posix_spawn. Existing C1 compute
    // owner THREAD creation and publication remain other-native25ms. The old
    // profile3 is now process_create above250ms; CPU/inclusive/clock/custody
    // profiles retain their original meaning and actions.
    for action in &mut m.actions {
        action.guard = eq(BOUND_CASE, 0).and(action.guard.clone());
    }
    // Ten correlated supplied-observation classes, never 0..251 independent
    // choices. One original observation means count0/1; source u64 telemetry
    // is ns (these exact ms representatives multiply by1_000_000). Known
    // creation excess is retained BEFORE verdict, even on251/CPU refusal.
    let bounds = [
        (1, 25, 2, 3, true),
        (1, 34, 2, 3, true),
        (1, 250, 2, 3, true),
        (1, 251, 2, 3, true),
        (2, 25, 2, 3, true),
        (2, 34, 2, 3, true),
        (1, 34, 3, 3, true),
        (1, 34, 2, 4, true),
        (1, 34, 2, 3, false),
        (2, 250, 2, 3, true),
    ];
    for (index, &(tag, wall, cpu, inclusive, known)) in bounds.iter().enumerate() {
        let case = index as i64 + 1;
        let excess_known = tag == 1 && known;
        let excess = if excess_known { (wall - 25).max(0) } else { 0 };
        act(
            &mut m,
            &format!("RetainTaggedNativeBoundObservation{case}"),
            all([eq(CONFIG, 0), eq(P, 0), eq(EFFECT, 0), eq(BOUND_CASE, 0)]),
            &[
                (BOUND_CASE, Int(case)),
                (NATIVE_TAG, Int(tag)),
                (WALL_MS, Int(wall)),
                (NATIVE_CPU_MS, Int(cpu)),
                (INCLUSIVE_CPU_MS, Int(inclusive)),
                (BOUND_CLOCK_KNOWN, Int(i64::from(known))),
                (CREATE_EXCESS_MS, Int(excess)),
                (CREATE_EXCESS_COUNT, Int(i64::from(excess > 0))),
                (CREATE_EXCESS_KNOWN, Int(i64::from(excess_known))),
            ],
        );
        let wall_bound = if tag == 1 && control != 23 { 250 } else { 25 };
        let admitted = known && cpu <= 2 && inclusive <= 3 && wall <= wall_bound;
        act(
            &mut m,
            &format!("VerdictExactTaggedNativeBoundObservation{case}"),
            all([eq(BOUND_CASE, case), eq(BOUND_VERDICT, 0)]),
            &[(BOUND_VERDICT, Int(if admitted { 1 } else { 2 }))],
        );
        let expected =
            known && cpu <= 2 && inclusive <= 3 && wall <= if tag == 1 { 250 } else { 25 };
        safe(
            &mut m,
            &format!("TaggedNativeWallExcessIsExactCase{case}"),
            eq(BOUND_CASE, case).negate().or(all([
                eq(CREATE_EXCESS_MS, excess),
                eq(CREATE_EXCESS_COUNT, i64::from(excess > 0)),
                eq(CREATE_EXCESS_KNOWN, i64::from(excess_known)),
            ])),
        );
        safe(
            &mut m,
            &format!("TaggedNativeVerdictMatchesBoundCase{case}"),
            all([eq(BOUND_CASE, case), eq(BOUND_VERDICT, 0).negate()])
                .negate()
                .or(eq(BOUND_VERDICT, if expected { 1 } else { 2 })),
        );
    }
    if control != 0 {
        let names: &[&str] = match control {
            1 => &["NativeEffectRequiresReservedAttempt"],
            2 => &["TrustedOutputHasOnlyAdmittedMembers"],
            3 => &["NativeEffectRequiresExactExecutableCapability"],
            4 => &["AttachedWorkKeepsExactIncarnation"],
            9 => &["MainProofCannotOverlapRunnableGroup"],
            10 => &["DebtCoversWholeObservedUnrestedSpan"],
            11 => &["OrdinaryApplicationCannotUseNativeLabel"],
            12..=14 => &["CertifiedNativeCreationMeetsCpuWallInclusiveBounds"],
            16 => &["OriginalNativeEffectNeverReplayed"],
            17 => &["TrustedOutputHasCompleteCoverage"],
            18 => &["TrustedOutputHasActualReapAndRetirement"],
            19..=21 => &["FaultCannotReleaseUnsettledOwnership"],
            22 => &["StopAttemptIsNotRetriedAfterAnyNativeOutcome"],
            23 => &[
                "TaggedNativeVerdictMatchesBoundCase2",
                "TaggedNativeVerdictMatchesBoundCase3",
            ],
            _ => unreachable!(),
        };
        let valid = all(m
            .invariants
            .iter()
            .filter(|i| full_observer || names.contains(&i.name.as_str()))
            .map(|i| i.condition.clone()));
        for a in &mut m.actions {
            let guard = a.guard.clone().and(valid.clone());
            a.guard = if full_observer {
                guard
            } else {
                guard.eq(Bool(true))
            };
        }
        let failed = valid.negate();
        act(
            &mut m,
            "ObserverStopsAtFailedAdmissionBoundary",
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



/// Task66364 adds a DISJOINT retained-child terminal/rollup projection to
/// VerifierEvents. The registered knote denotes one process, not a PID lookup;
/// an approved clone-to-GPG exec changes its token chain, not that process.
/// RI-only TotalBudget trajectories remain the unchanged legacy source slice.
/// This projection checks the alternate bit4 source and unchanged other bits,
/// source-bound read ordering/coverage, sticky refusal and one subtree bill.
fn extend_verifier_child_exit(m: &mut Model, control: u8) {
    const CASE: usize = 32;
    const P: usize = 33;
    const WATCH: usize = 34;
    const NOTE: usize = 35;
    const RI_EXIT: usize = 36;
    const RI_ERROR: usize = 37;
    const CURRENT: usize = 38;
    const GONE: usize = 39;
    const RECEIPT: usize = 40;
    const FIRST: usize = 41;
    const READ: usize = 42;
    const COVER: usize = 43;
    const OTHER: usize = 44;
    const VERDICT: usize = 45;
    const WAIT: usize = 46;
    const CPU: usize = 47;
    const DUE: usize = 48;
    const PAID: usize = 49;
    const RELEASE: usize = 50;
    const UNPROVEN: usize = 51;
    const EXTRA: usize = 52;
    const WITNESS: usize = 53;
    const PENDING: usize = 54;
    const DRAINED: usize = 55;
    const OWNER: usize = 56;
    const PRIOR: usize = 57;
    const BILLS: usize = 58;
    const EXEC: usize = 59;
    for (name, max) in [
        ("childExitCorrelatedCase", 16),
        ("childExitProtocolPhase", 10),
        ("registeredChildKnoteNoneSameProcessForeign", 2),
        ("childEventNoneExecExit", 2),
        ("childRiExitAbstimePositive", 1),
        ("childRiFailureNoneEsrchOtherUnknown", 3),
        ("currentValidatedChildTokenChain", 1),
        ("childCurrentTokenGone", 1),
        ("childTerminalObservedReceipt", 1),
        ("firstChildTerminalReceipt", 1),
        ("sameLeaderRollupReadTime", 2),
        ("componentwiseChildRollupCovers", 1),
        ("unchangedRollupOtherBitsSourceSoleKnownEdge", 1),
        ("childRollupPendingAcceptedRefused", 2),
        ("actualLeaderOwnWait4AndGroupGone", 1),
        ("authoritativeWholeSubtreeWait4Cpu", 2),
        ("fullWholeSubtreeFreshDebt", 18),
        ("wholeSubtreeDebtActuallyPaid", 18),
        ("coveredChildCostRelease", 1),
        ("stickyChildCostUnproven", 1),
        ("extraChildWait4OrDuplicateDebit", 1),
        ("child_exit_witnessNoneRiKqueueNoteExit", 2),
        ("queuedChildExitNotification", 1),
        ("childEventsDrainedBeforeRollupVerdict", 1),
        ("retainedOriginalLeaderOwner", 1),
        ("priorChildCostAlreadyUnproven", 1),
        ("originalLeaderTerminalBillCount", 1),
        ("approvedChildExecWithinWatchedProcess", 1),
    ] {
        m.variables.push(Variable::new(name, 0, max, 0));
    }
    // No independent cross-product: existing discovery actions stay in CASE0;
    // every appended profile leaves all32 existing fields at their initial0.
    for action in &mut m.actions {
        action.guard = eq(CASE, 0).and(action.guard.clone());
    }
    for profile in 1..=16 {
        act(
            m,
            &format!("RetainOriginalWatchedVerifierProfile{profile}"),
            all([eq(0, 0), eq(CASE, 0)]),
            &[
                (CASE, Int(profile)),
                (P, Int(1)),
                (OWNER, Int(1)),
                (
                    WATCH,
                    Int(if profile == 4 {
                        0
                    } else if profile == 5 {
                        2
                    } else {
                        1
                    }),
                ),
                (CURRENT, Int(i64::from(profile != 16))),
                (PRIOR, Int(i64::from(profile == 13))),
                (UNPROVEN, Int(i64::from(profile == 13))),
                (EXEC, Int(i64::from(profile == 15))),
            ],
        );
        // Profiles:1 RI exit,2 knote+RI_ESRCH,3 absent event,4 registration
        // failed,5 foreign/reused process watch,6 EXEC-only,7 RI_EIO,
        // 8 token not gone,9 unknown receipt clock,10 stale leader read,
        // 11 componentwise undercoverage,12 other mask premise missing,
        // 13 already-sticky unknown,14 RI result unknown,15 approved exec
        // on SAME watched process,16 invalid current token/exec chain.
        act(
            m,
            &format!("QueueChildRecordWithoutInventingTerminalOrCpu{profile}"),
            all([eq(CASE, profile), eq(P, 1)]),
            &[
                (P, Int(2)),
                (
                    NOTE,
                    Int(if profile == 1 || profile == 3 || profile == 4 {
                        0
                    } else if profile == 6 {
                        1
                    } else {
                        2
                    }),
                ),
                (PENDING, Int(i64::from(!matches!(profile, 1 | 3 | 4)))),
            ],
        );
        act(
            m,
            &format!("DrainRetainedChildRecordsBeforeAnyUnprovenLatch{profile}"),
            all([eq(CASE, profile), eq(P, 2)]),
            &[
                (P, Int(3)),
                (PENDING, Int(0)),
                (DRAINED, Int(1)),
                (
                    RECEIPT,
                    Int(i64::from(!matches!(profile, 1 | 3 | 4 | 6 | 9))),
                ),
                (FIRST, Int(i64::from(!matches!(profile, 1 | 3 | 4 | 6 | 9)))),
            ],
        );
        act(
            m,
            &format!("ReadChildRiAndRetainExactOutcomeWithoutResidualCredit{profile}"),
            all([eq(CASE, profile), eq(P, 3)]),
            &[
                (P, Int(4)),
                (RI_EXIT, Int(i64::from(profile == 1))),
                (
                    RI_ERROR,
                    Int(if profile == 1 {
                        0
                    } else if profile == 7 {
                        2
                    } else if profile == 14 {
                        3
                    } else {
                        1
                    }),
                ),
                (GONE, Int(i64::from(profile != 8))),
                (RECEIPT, if profile == 1 { Int(1) } else { Var(RECEIPT) }),
                (FIRST, if profile == 1 { Int(1) } else { Var(FIRST) }),
            ],
        );
        act(
            m,
            &format!("FreshSameLeaderRollupRetainsOtherBitsAndLastChildSample{profile}"),
            all([eq(CASE, profile), eq(P, 5)]),
            &[
                (P, Int(6)),
                (READ, Int(if profile == 10 { 0 } else { 2 })),
                (COVER, Int(i64::from(profile != 11))),
                (OTHER, Int(i64::from(profile != 12))),
            ],
        );
    }
    // A repeat notification is observed but never moves the receipt beyond a
    // previously covering same-leader read. No native-delivery fairness.
    act(
        m,
        "DuplicateChildExitKeepsFirstPositiveReceiptStable",
        all([
            eq(CASE, 0).negate(),
            eq(NOTE, 2),
            eq(RECEIPT, 1),
            eq(RELEASE, 0),
        ]),
        &[(RECEIPT, Var(FIRST))],
    );
    let ri = all([eq(RI_EXIT, 1), eq(CURRENT, 1), eq(GONE, 1), eq(RECEIPT, 1)]);
    let note = all([
        eq(WATCH, 1),
        eq(CURRENT, 1),
        eq(NOTE, 2),
        eq(RI_EXIT, 0),
        eq(RI_ERROR, 1),
        eq(GONE, 1),
        eq(RECEIPT, 1),
        eq(DRAINED, 1),
    ]);
    let accepted_note = if control == 6 {
        Bool(false)
    } else {
        all([
            if control == 7 {
                Bool(true)
            } else {
                eq(WATCH, 1)
            },
            eq(CURRENT, 1),
            eq(NOTE, 2),
            eq(RI_EXIT, 0),
            eq(RI_ERROR, 1),
            eq(GONE, 1),
            eq(RECEIPT, 1),
            eq(DRAINED, 1),
        ])
    };
    for (kind, guard) in [
        (1, ri.clone()),
        (2, ri.clone().negate().and(accepted_note.clone())),
        (0, ri.clone().negate().and(accepted_note.negate())),
    ] {
        act(
            m,
            &format!("ClassifyRetainedChildExitWitness{kind}"),
            all([eq(CASE, 0).negate(), eq(P, 4), eq(DRAINED, 1), guard]),
            &[(P, Int(5)), (WITNESS, Int(kind))],
        );
    }
    let correct = all([
        ri.clone().or(note.clone()),
        eq(OTHER, 1),
        eq(COVER, 1),
        Var(READ).lt(Var(RECEIPT)).negate(),
        eq(PRIOR, 0),
    ]);
    let admitted = all([
        eq(WITNESS, 0).negate(),
        eq(OTHER, 1),
        eq(COVER, 1),
        Var(READ).lt(Var(RECEIPT)).negate(),
        eq(PRIOR, 0),
    ]);
    for verdict in 1..=2 {
        act(
            m,
            &format!("DecideChildRollupWithoutClearingEarlierUnknown{verdict}"),
            all([
                eq(CASE, 0).negate(),
                eq(P, 6),
                eq(DRAINED, 1),
                if verdict == 1 {
                    admitted.clone()
                } else {
                    admitted.clone().negate()
                },
            ]),
            &[
                (P, Int(7)),
                (VERDICT, Int(verdict)),
                (UNPROVEN, if verdict == 2 { Int(1) } else { Var(UNPROVEN) }),
            ],
        );
    }
    act(
        m,
        "ActualOriginalLeaderWait4BooksWholeSubtreeExactlyOnce",
        all([eq(CASE, 0).negate(), eq(P, 7), eq(WAIT, 0), eq(BILLS, 0)]),
        &[
            (P, Int(8)),
            (WAIT, Int(1)),
            (BILLS, Int(1)),
            (CPU, Int(2)),
            (DUE, Int(18)),
            (PAID, Int(0)),
        ],
    );
    act(
        m,
        "ObserveFullFreshSubtreeRestEndpoint",
        all([eq(CASE, 0).negate(), eq(P, 8), eq(WAIT, 1), eq(PAID, 0)]),
        &[(P, Int(9)), (PAID, Var(DUE))],
    );
    act(
        m,
        "KnownCoveredChildCostAfterWholeLeaderBillPaid",
        all([eq(CASE, 0).negate(), eq(P, 9), eq(VERDICT, 1), eq(PAID, 18)]),
        &[(P, Int(10)), (RELEASE, Int(1)), (OWNER, Int(0))],
    );
    act(
        m,
        "UnprovenChildKeepsSameOwnerAfterKnownLeaderCostPaid",
        all([eq(CASE, 0).negate(), eq(P, 9), eq(VERDICT, 2), eq(PAID, 18)]),
        &[(P, Int(10))],
    );
    safe(
        m,
        "ChildKnoteExitNeedsRetainedSameProcessCurrentChainAndEsrch",
        eq(CASE, 0).or(all([
            eq(WITNESS, 1).negate().or(ri),
            eq(WITNESS, 2).negate().or(note),
        ])),
    );
    safe(
        m,
        "ChildRollupVerdictMatchesActualTaggedTerminalWitness",
        eq(CASE, 0).or(eq(VERDICT, 0)
            .or(eq(VERDICT, 1).and(correct.clone()))
            .or(eq(VERDICT, 2).and(correct.negate()))),
    );
    safe(
        m,
        "FirstChildExitReceiptNeverMovesOnDuplicate",
        eq(CASE, 0).or(Var(RECEIPT).eq(Var(FIRST))),
    );
    safe(
        m,
        "Bit4KnoteSourcePreservesUnknownAndRollupCoverage",
        eq(CASE, 0).or(all([
            eq(PRIOR, 0).or(eq(UNPROVEN, 1)),
            eq(VERDICT, 2)
                .negate()
                .or(all([eq(UNPROVEN, 1), eq(OWNER, 1), eq(RELEASE, 0)])),
            eq(VERDICT, 1).negate().or(all([
                eq(OTHER, 1),
                eq(COVER, 1),
                eq(RECEIPT, 1),
                Var(READ).lt(Var(RECEIPT)).negate(),
            ])),
        ])),
    );
    safe(
        m,
        "ChildExitEventNeverBecomesCpuReapOrDuplicateChildBill",
        eq(CASE, 0).or(all([
            eq(EXTRA, 0),
            eq(BILLS, 0).or(all([eq(WAIT, 1), eq(CPU, 2), eq(DUE, 18)])),
            eq(WAIT, 1).or(all([eq(CPU, 0), eq(DUE, 0), eq(PAID, 0), eq(BILLS, 0)])),
        ])),
    );
    safe(
        m,
        "CoveredChildReleaseNeedsActualWholeLeaderBillAndFullDebt",
        eq(CASE, 0).or(eq(RELEASE, 0).or(all([
            eq(VERDICT, 1),
            eq(WAIT, 1),
            eq(BILLS, 1),
            eq(CPU, 2),
            eq(DUE, 18),
            eq(PAID, 18),
            eq(UNPROVEN, 0),
        ]))),
    );
    // All native completion/delivery/clock observations remain unfair.
}

#[cfg(test)]
mod verifier_child_exit_tests {
    use super::*;

    #[test]
    fn retained_child_knote_exit_covers_esrch_without_invented_cost_or_reap() {
        let model = verifier_event_discovery(0);
        let good = model.check(100_000).unwrap();
        println!(
            "{} child-exit projection: {} states",
            model.name,
            good.states.len()
        );
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        for (profile, witness) in [(1, 1), (2, 2), (15, 2)] {
            assert!(
                good.states.iter().any(|s| s[32] == profile
                    && s[45] == 1
                    && s[53] == witness
                    && s[46] == 0
                    && s[47] == 0
                    && s[58] == 0),
                "RI or retained same-process knote permits bit4; neither observation is wait4/CPU"
            );
            assert!(
                good.states.iter().any(|s| s[32] == profile
                    && s[50] == 1
                    && s[46] == 1
                    && s[47] == 2
                    && s[48] == 18
                    && s[49] == 18
                    && s[52] == 0
                    && s[58] == 1),
                "whole leader subtree wait4 is billed once and full9x paid before cost release"
            );
        }
        assert!(
            good.states.iter().any(|s| s[32] == 15
                && s[59] == 1
                && s[34] == 1
                && s[38] == 1
                && s[53] == 2
                && s[50] == 1),
            "approved exec advances validated token chain within the same process-bound watch"
        );
        for profile in [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16] {
            assert!(
                good.states.iter().any(|s| s[32] == profile
                    && s[33] == 10
                    && s[45] == 2
                    && s[50] == 0
                    && s[51] == 1
                    && s[56] == 1
                    && s[46] == 1
                    && s[47] == 2
                    && s[48] == 18
                    && s[49] == 18),
                "invalid source/outcome/time/rollup premise{profile} retains full leader cost/debt and same-owner refusal"
            );
        }
        assert!(
            good.states.iter().any(|s| s[32] == 2
                && s[54] == 1
                && s[55] == 0
                && s[40] == 0
                && s[45] == 0
                && s[51] == 0),
            "pending exit record must drain before an irreversible unproven verdict"
        );
        assert!(
            good.states.iter().any(|s| s[32] == 13
                && s[53] == 2
                && s[57] == 1
                && s[51] == 1
                && s[45] == 2),
            "later valid event evidence never clears already-sticky child_cost_unproven"
        );
        let duplicate = model
            .actions
            .iter()
            .position(|a| a.name == "DuplicateChildExitKeepsFirstPositiveReceiptStable")
            .unwrap();
        assert!(
            good.edges
                .iter()
                .enumerate()
                .any(|(from, edges)| good.states[from][32] == 2
                    && good.states[from][42] == 2
                    && edges
                        .iter()
                        .any(|edge| edge.action == duplicate && edge.to == from)),
            "duplicate child NOTE_EXIT preserves the earlier receipt covered by leader rollup"
        );
        for (control, property) in [
            (6, "ChildRollupVerdictMatchesActualTaggedTerminalWitness"),
            (
                7,
                "ChildKnoteExitNeedsRetainedSameProcessCurrentChainAndEsrch",
            ),
        ] {
            let model = verifier_event_discovery(control);
            let checked = model.check(100_000).unwrap();
            println!("{}: {} states", model.name, checked.states.len());
            assert!(
                checked.violations.iter().any(|v| v.property == property),
                "{} must causally fail {property}",
                model.name
            );
            if control == 6 {
                assert!(
                    checked.states.iter().any(|s| s[32] == 2
                        && s[37] == 1
                        && s[35] == 2
                        && s[34] == 1
                        && s[45] == 2
                        && s[53] == 0),
                    "old ignored-NOTE_EXIT refuses the otherwise valid leader-reaped child"
                );
            } else {
                assert!(
                    checked
                        .states
                        .iter()
                        .any(|s| s[32] == 5 && s[34] == 2 && s[53] == 2 && s[45] == 1),
                    "foreign/reused watch cannot certify a retained child's exit"
                );
            }
        }
    }
}

/// Task66356: a disjoint finite protocol projection of the SAME helper owner.
/// Profiles correlate a queued Darwin notification with a subsequent census;
/// NOTE_FORK carries no child PID and notification counts are not birth counts.
/// Native delivery/census/GS1/GS2/exit/wait4/clock success is not fair. The
/// ordinary observation/FD unit retains its full abstract9x endpoint debt.
/// Numeric budgets, native timing and exact GS1/GS2 byte/identity predicates
/// remain the unchanged TotalBudget, HelperExec and GitSigTemp contracts.
pub fn verifier_event_discovery(control: u8) -> Model {
    let suffix = [
        "",
        "PollOnly",
        "LateRegistration",
        "EventAsProof",
        "EarlyClose",
        "SkipTerminalDrain",
        "IgnoreChildExit",
        "UnboundChildExit",
    ][control as usize];
    let mut m = base(
        &format!("DreamReadVerifierEvents{suffix}"),
        control != 0,
        &[
            ("phase", 12, 0),
            ("correlatedTimeline", 11, 0),
            ("ownedKqueueFd", 1, 0),
            ("kqueueClosed", 1, 0),
            ("leaderRegistrationNoneSuccessFailure", 2, 0),
            ("firstControlledRun", 1, 0),
            ("queuedForkRecord", 1, 0),
            ("queuedLeaderExecRecord", 1, 0),
            ("leader_fork_events", 1, 0),
            ("leader_exec_events", 1, 0),
            ("censusUncheckedEmptyOneMultipleUnknown", 4, 0),
            ("child_discovered_via", 2, 0),
            ("immutableFirstDiscoveryCause", 2, 0),
            ("childExecExitRegistration", 2, 0),
            ("observedChildNotificationNoneExecExit", 2, 0),
            ("retainedSourceBoundGs1", 1, 0),
            ("actualOriginalExit", 1, 0),
            ("actualOriginalOwnWait4", 1, 0),
            ("positiveOriginalGroupGone", 1, 0),
            ("retainedSourceBoundGs2", 1, 0),
            ("verifierEvidenceAccepted", 1, 0),
            ("refusalNoneCoverageNoVerifierMissedGs1", 4, 0),
            ("fork_seen_child_missed", 1, 0),
            ("freshOrdinaryDebtPending", 1, 0),
            ("fullRetainedOrdinaryDebt", 9, 0),
            ("ordinaryDebtActuallyPaid", 9, 0),
            ("ownershipReleased", 1, 0),
            ("sameOwnerQuarantine", 1, 0),
            ("availableRecordsDrained", 1, 0),
            ("forkRecordCensusPerformed", 1, 0),
            ("kqueueCloseOnExec", 1, 0),
            ("sourceQueuedForkRecord", 1, 0),
        ],
    );
    const P: usize = 0;
    const CASE: usize = 1;
    const FD: usize = 2;
    const CLOSED: usize = 3;
    const REG: usize = 4;
    const RUN: usize = 5;
    const PENDING: usize = 6;
    const EXEC_PENDING: usize = 7;
    const FORKS: usize = 8;
    const EXECS: usize = 9;
    const CENSUS: usize = 10;
    const VIA: usize = 11;
    const FIRST: usize = 12;
    const CHILD_REG: usize = 13;
    const CHILD_NOTE: usize = 14;
    const GS1: usize = 15;
    const EXIT: usize = 16;
    const REAP: usize = 17;
    const GONE: usize = 18;
    const GS2: usize = 19;
    const PROOF: usize = 20;
    const REFUSAL: usize = 21;
    const MISSED: usize = 22;
    const DEBT: usize = 23;
    const DUE: usize = 24;
    const PAID: usize = 25;
    const RELEASE: usize = 26;
    const QUAR: usize = 27;
    const DRAINED: usize = 28;
    const FORK_CENSUS: usize = 29;
    const CLOEXEC: usize = 30;
    const SOURCE_FORK: usize = 31;
    // 1 fast fork+exit between polls, including leader exit before draining;
    // 2 event discovers one child, 3 poll discovers first then fork notification;
    // 4 never forks, 5 leader registration fails, 6 child registration fails;
    // 7 GS1 fails, 8 unavailable native observation retains FD in quarantine;
    // 9 leader EXEC record without fork, 10 child EXIT record without GS1;
    // 11 multiple/unclosed child custody refuses and retains original owner.
    for profile in 1..=11 {
        act(
            &mut m,
            &format!("OwnCloseOnExecKqueueForTimeline{profile}"),
            eq(P, 0),
            &[
                (P, Int(1)),
                (CASE, Int(profile)),
                (FD, Int(1)),
                (CLOEXEC, Int(1)),
                (DEBT, Int(1)),
                (DUE, Int(9)),
                (PAID, Int(0)),
            ],
        );
        act(
            &mut m,
            &format!("RegisterLeaderForkExecExitBeforeRunning{profile}"),
            all([eq(CASE, profile), eq(P, 1), eq(DEBT, 0)]),
            &[
                (P, Int(2)),
                (REG, Int(if profile == 5 { 2 } else { 1 })),
                (DEBT, Int(1)),
                (DUE, Int(9)),
                (PAID, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "ObserveEntireFreshOrdinaryRestEndpoint",
        eq(DEBT, 1),
        &[(DEBT, Int(0)), (PAID, Var(DUE))],
    );
    act(
        &mut m,
        "FirstControlledRunNeedsSuccessfulOriginalRegistration",
        all([
            eq(P, 2),
            eq(DEBT, 0),
            if control == 2 { Bool(true) } else { eq(REG, 1) },
        ]),
        &[(P, Int(3)), (RUN, Int(1)), (DEBT, Int(1)), (PAID, Int(0))],
    );
    act(
        &mut m,
        "RegistrationFailureRefusesWithoutControlledRun",
        all([eq(P, 2), eq(REG, 2), eq(DEBT, 0)]),
        &[(P, Int(8)), (REFUSAL, Int(1)), (DRAINED, Int(1))],
    );
    for profile in [1, 2, 3, 4, 6, 7, 9, 10, 11] {
        let fork = i64::from(!matches!(profile, 4 | 9));
        act(
            &mut m,
            &format!("SourceTimelineQueuesRecordsWithoutSupplyingChildPid{profile}"),
            all([eq(CASE, profile), eq(P, 3), eq(DEBT, 0)]),
            &[
                (P, Int(4)),
                (PENDING, Int(fork)),
                (SOURCE_FORK, Int(fork)),
                (EXEC_PENDING, Int(i64::from(profile == 9))),
                (EXIT, Int(i64::from(profile == 1))),
                (VIA, Int(i64::from(profile == 3))),
                (FIRST, Int(i64::from(profile == 3))),
                (CENSUS, Int(if profile == 3 { 2 } else { 0 })),
                (DEBT, Int(1)),
                (PAID, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "UnavailableEventObservationRetainsSameOwnerAndFd",
        all([eq(CASE, 8), eq(P, 3), eq(DEBT, 0)]),
        &[
            (P, Int(12)),
            (QUAR, Int(1)),
            (REFUSAL, Int(1)),
            (DEBT, Int(1)),
            (PAID, Int(0)),
        ],
    );
    act(
        &mut m,
        "DrainAvailableRecordsBeforeTerminalEarlyReturnOrCensus",
        all([
            eq(P, 4),
            eq(DEBT, 0),
            if control == 5 {
                eq(CASE, 1).negate()
            } else {
                Bool(true)
            },
        ]),
        &[
            (P, Int(5)),
            (FORKS, if control == 1 { Int(0) } else { Var(PENDING) }),
            (EXECS, Var(EXEC_PENDING)),
            (PENDING, Int(0)),
            (EXEC_PENDING, Int(0)),
            (DRAINED, Int(1)),
            (DEBT, Int(1)),
            (PAID, Int(0)),
        ],
    );
    if control == 5 {
        act(
            &mut m,
            "BrokenLeaderExitBypassesQueuedForkDrain",
            all([eq(P, 4), eq(CASE, 1), eq(EXIT, 1), eq(DEBT, 0)]),
            &[(P, Int(5)), (DEBT, Int(1)), (PAID, Int(0))],
        );
    }
    for profile in [1, 2, 3, 4, 6, 7, 9, 10, 11] {
        let one = matches!(profile, 2 | 3 | 6 | 7 | 10);
        let empty = matches!(profile, 1 | 4 | 9);
        for fork_seen in 0..=1 {
            let cause = if profile == 3 {
                1
            } else if one && fork_seen == 1 {
                2
            } else if one {
                1
            } else {
                0
            };
            act(
                &mut m,
                &format!("ImmediateExistingCensusTimeline{profile}ForkRecord{fork_seen}"),
                all([
                    eq(CASE, profile),
                    eq(P, 5),
                    eq(DEBT, 0),
                    eq(FORKS, fork_seen),
                ]),
                &[
                    (P, Int(if one { 6 } else { 8 })),
                    (
                        CENSUS,
                        Int(if one {
                            2
                        } else if empty {
                            1
                        } else {
                            3
                        }),
                    ),
                    (VIA, Int(cause)),
                    (FIRST, Int(cause)),
                    (FORK_CENSUS, Int(fork_seen)),
                    (MISSED, Int(i64::from(empty && fork_seen == 1))),
                    (
                        REFUSAL,
                        Int(if empty {
                            if fork_seen == 1 { 3 } else { 2 }
                        } else if one {
                            0
                        } else {
                            1
                        }),
                    ),
                    (DEBT, Int(1)),
                    (PAID, Int(0)),
                ],
            );
        }
    }
    for profile in [2, 3, 6, 7, 10] {
        act(
            &mut m,
            &format!("RegisterRetainedChildExecExitOnFirstAttachment{profile}"),
            all([eq(CASE, profile), eq(P, 6), eq(DEBT, 0)]),
            &[
                (P, Int(7)),
                (CHILD_REG, Int(if profile == 6 { 2 } else { 1 })),
                (
                    CHILD_NOTE,
                    Int(if profile == 2 {
                        1
                    } else if profile == 10 {
                        2
                    } else {
                        0
                    }),
                ),
                (DEBT, Int(1)),
                (PAID, Int(0)),
            ],
        );
        act(
            &mut m,
            &format!("ExistingGs1StillNeedsActualRetainedVerifierEvidence{profile}"),
            all([eq(CASE, profile), eq(P, 7), eq(DEBT, 0)]),
            &[
                (P, Int(8)),
                (GS1, Int(i64::from(matches!(profile, 2 | 3)))),
                (
                    REFUSAL,
                    Int(if profile == 6 {
                        1
                    } else if matches!(profile, 7 | 10) {
                        4
                    } else {
                        0
                    }),
                ),
                (DEBT, Int(1)),
                (PAID, Int(0)),
            ],
        );
    }
    if control == 3 {
        act(
            &mut m,
            "BrokenChildNotificationSubstitutesForGs1Gs2AndOwnReap",
            all([eq(P, 7), eq(DEBT, 0), eq(CHILD_NOTE, 0).negate()]),
            &[(PROOF, Int(1))],
        );
    }
    if control == 4 {
        act(
            &mut m,
            "BrokenCloseKqueueWhileOriginalEffectStillUnsettled",
            all([eq(P, 3), eq(FD, 1), eq(DEBT, 0)]),
            &[
                (FD, Int(0)),
                (CLOSED, Int(1)),
                (DEBT, Int(1)),
                (PAID, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "MultipleOrUnclosedChildCustodyKeepsOriginalQuarantine",
        all([eq(P, 8), eq(CASE, 11), eq(DEBT, 0)]),
        &[(P, Int(12)), (QUAR, Int(1)), (REFUSAL, Int(1))],
    );
    act(
        &mut m,
        "IndependentActualOriginalExitNotNotification",
        all([eq(P, 8), eq(DEBT, 0), eq(CASE, 11).negate()]),
        &[(P, Int(9)), (EXIT, Int(1)), (DEBT, Int(1)), (PAID, Int(0))],
    );
    act(
        &mut m,
        "ActualOriginalOwnWait4AndPositiveGroupSettlement",
        all([
            eq(P, 9),
            eq(DEBT, 0),
            eq(EXIT, 1),
            if control == 5 {
                Bool(true)
            } else {
                all([eq(DRAINED, 1), eq(PENDING, 0)])
            },
        ]),
        &[
            (P, Int(10)),
            (REAP, Int(1)),
            (GONE, Int(1)),
            (DEBT, Int(1)),
            (PAID, Int(0)),
        ],
    );
    for gs1 in 0..=1 {
        act(
            &mut m,
            &format!("ExistingGs2AfterActualLeaderSettlementGs1{gs1}"),
            all([
                eq(P, 10),
                eq(DEBT, 0),
                eq(GS1, gs1),
                eq(REAP, 1),
                eq(GONE, 1),
            ]),
            &[(P, Int(11)), (GS2, Int(gs1)), (PROOF, Int(gs1))],
        );
    }
    act(
        &mut m,
        "ChargedKqueueCloseAfterOriginalEffectCannotAct",
        all([eq(P, 11), eq(DEBT, 0), eq(REAP, 1), eq(GONE, 1)]),
        &[
            (P, Int(12)),
            (FD, Int(0)),
            (CLOSED, Int(1)),
            (DEBT, Int(1)),
            (PAID, Int(0)),
        ],
    );
    act(
        &mut m,
        "ReleaseOnlyAfterPaidCloseAndActualSettlement",
        all([
            eq(P, 12),
            eq(QUAR, 0),
            eq(CLOSED, 1),
            eq(DEBT, 0),
            eq(REAP, 1),
            eq(GONE, 1),
            eq(RELEASE, 0),
        ]),
        &[(RELEASE, Int(1))],
    );
    safe(
        &mut m,
        "LeaderRegisteredBeforeFirstControlledRun",
        eq(RUN, 0).or(eq(REG, 1)),
    );
    safe(
        &mut m,
        "FastForkIsMissedRefusalRatherThanNoVerifier",
        all([
            Var(P)
                .lt(Int(8))
                .or(eq(CENSUS, 1).negate())
                .or(eq(SOURCE_FORK, 0))
                .or(all([eq(MISSED, 1), eq(REFUSAL, 3)])),
            eq(MISSED, 0).or(all([eq(VIA, 0), eq(GS1, 0), eq(PROOF, 0)])),
        ]),
    );
    safe(
        &mut m,
        "QueuedForkDrainedAndCensusedBeforeActualReap",
        eq(REAP, 0).or(all([
            eq(DRAINED, 1),
            eq(PENDING, 0),
            eq(SOURCE_FORK, 0).or(eq(FORK_CENSUS, 1)),
        ])),
    );
    safe(
        &mut m,
        "EventsNeverReplaceActualVerifierGs1Gs2OrSettlement",
        eq(PROOF, 0).or(all([
            eq(CENSUS, 2),
            eq(CHILD_REG, 1),
            eq(GS1, 1),
            eq(GS2, 1),
            eq(REAP, 1),
            eq(GONE, 1),
        ])),
    );
    safe(
        &mut m,
        "KqueueRetainedUntilActualOriginalSettlement",
        eq(P, 0).or(eq(FD, 1).or(all([eq(REAP, 1), eq(GONE, 1)]))),
    );
    safe(
        &mut m,
        "FirstDiscoveryCauseIsImmutable",
        Var(VIA).eq(Var(FIRST)),
    );
    safe(
        &mut m,
        "ActualGs1RequiresRegisteredRetainedSingleVerifier",
        eq(GS1, 0).or(all([eq(CENSUS, 2), eq(CHILD_REG, 1), eq(VIA, 0).negate()])),
    );
    safe(
        &mut m,
        "KqueueIsCloseOnExecAndObservationDebtIsWhole",
        all([
            eq(P, 0).or(eq(CLOEXEC, 1)),
            eq(DEBT, 0).or(all([eq(DUE, 9), eq(PAID, 0)])),
        ]),
    );
    safe(
        &mut m,
        "ReleaseNeedsFullPaidCloseAndActualSettlement",
        eq(RELEASE, 0).or(all([
            eq(CLOSED, 1),
            eq(FD, 0),
            eq(REAP, 1),
            eq(GONE, 1),
            eq(DEBT, 0),
            eq(PAID, 9),
            eq(QUAR, 0),
        ])),
    );
    extend_verifier_child_exit(&mut m, control);
    m
}

#[cfg(test)]
mod verifier_event_discovery_tests {
    use super::*;

    #[test]
    fn queued_fork_discovery_preserves_custody_diagnostics_and_all_causal_controls() {
        let good = verifier_event_discovery(0).check(100_000).unwrap();
        println!(
            "HarnessDreamReadVerifierEvents: {} states",
            good.states.len()
        );
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        for profile in [2, 3] {
            assert!(
                good.states.iter().any(|s| s[1] == profile
                    && s[20] == 1
                    && s[15] == 1
                    && s[19] == 1
                    && s[17] == 1
                    && s[26] == 1
                    && s[9] == 0
                    && s[11] == if profile == 3 { 1 } else { 2 }),
                "retained verifier uses existing GS1/GS2; initial leader EXEC event is unnecessary"
            );
        }
        assert!(
            good.states.iter().any(|s| s[1] == 1
                && s[16] == 1
                && s[6] == 1
                && s[8] == 0
                && s[28] == 0
                && s[2] == 1
                && s[17] == 0),
            "positive original exit cannot bypass the queued fast-fork record"
        );
        assert!(
            good.states.iter().any(|s| s[1] == 1
                && s[22] == 1
                && s[21] == 3
                && s[17] == 1
                && s[18] == 1
                && s[26] == 1
                && s[20] == 0
                && s[27] == 0),
            "missed-verifier refusal may physically settle without inventing custody or trust"
        );
        for profile in [4, 9] {
            assert!(
                good.states.iter().any(|s| s[1] == profile
                    && s[21] == 2
                    && s[22] == 0
                    && s[20] == 0
                    && s[26] == 1),
                "never-fork and leader-EXEC-only give created-no-verifier, not missed"
            );
        }
        assert!(
            good.states.iter().any(|s| s[1] == 5
                && s[4] == 2
                && s[5] == 0
                && s[21] == 1
                && s[26] == 1),
            "failed registration refuses before controlled run"
        );
        assert!(
            good.states.iter().any(|s| s[1] == 6
                && s[13] == 2
                && s[15] == 0
                && s[21] == 1
                && s[26] == 1),
            "child registration failure never supplies GS1"
        );
        assert!(
            good.states.iter().any(|s| s[1] == 7
                && s[15] == 0
                && s[21] == 4
                && s[20] == 0
                && s[26] == 1),
            "event discovery does not waive failed GS1"
        );
        assert!(
            good.states.iter().any(|s| s[1] == 10
                && s[14] == 2
                && s[16] == 0
                && s[17] == 0
                && s[15] == 0
                && s[2] == 1),
            "child EXIT notification supplies neither actual original exit/reap nor GS1"
        );
        for profile in [8, 11] {
            assert!(
                good.states.iter().any(|s| s[1] == profile
                    && s[27] == 1
                    && s[2] == 1
                    && s[3] == 0
                    && s[26] == 0
                    && s[20] == 0),
                "unavailable observation or unclosed child custody retains same owner and FD"
            );
        }
        let expected = [
            "FastForkIsMissedRefusalRatherThanNoVerifier",
            "LeaderRegisteredBeforeFirstControlledRun",
            "EventsNeverReplaceActualVerifierGs1Gs2OrSettlement",
            "KqueueRetainedUntilActualOriginalSettlement",
            "QueuedForkDrainedAndCensusedBeforeActualReap",
        ];
        for (index, property) in expected.into_iter().enumerate() {
            let model = verifier_event_discovery((index + 1) as u8);
            let result = model.check(100_000).unwrap();
            println!("{}: {} states", model.name, result.states.len());
            assert!(
                result.violations.iter().any(|v| v.property == property),
                "{} must causally violate {property}",
                model.name
            );
            match index + 1 {
                1 => assert!(result.states.iter().any(|s| s[1] == 1
                    && s[31] == 1
                    && s[8] == 0
                    && s[21] == 2
                    && s[22] == 0)),
                2 => assert!(result.states.iter().any(|s| s[4] == 2 && s[5] == 1)),
                3 => assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[14] != 0 && s[20] == 1 && s[15] == 0 && s[17] == 0)
                ),
                4 => assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[2] == 0 && s[3] == 1 && s[17] == 0 && s[5] == 1)
                ),
                5 => assert!(
                    result
                        .states
                        .iter()
                        .any(|s| s[1] == 1 && s[6] == 1 && s[28] == 0 && s[17] == 1)
                ),
                _ => unreachable!(),
            }
        }
    }
}

/// Conditional foreground-pause/current-incarnation refinement. It starts
/// from an actually observed paused clone, not mandatory suspended birth.
/// Resume and exec invalidate old control receipts; retained old tokens are
/// read-only. Debt here is OWN controller observation work, never helper CPU.
/// Unsliced helper lifetime/caps/terminal debt are HelperShimBudget.
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
            ("sourceHelperRunning", 1, 0),
            ("stopRequestVersion", 2, 0),
            ("ownControllerObservationDebt", 1, 0),
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
            ("versionAtForegroundResume", 2, 0),
            ("receiptAtForegroundResume", 2, 0),
            ("stoppedAtForegroundResume", 1, 0),
            ("versionAtRest", 2, 0),
            ("receiptAtRest", 2, 0),
            ("stoppedAtRest", 1, 0),
            ("capturedAtForegroundResume", 2, 0),
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
        "ResumeForegroundPauseFromCurrentStopReceipt",
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
    // This transition requires no prior GPG stop: the original running source owns execution through this one-way native exec boundary.
    act(
        &mut m,
        "KernelExecCloneToExactGpgWhileRunning",
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
            "BrokenResumeUsesReadOnlyOldStopSeal",
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
        "ForegroundResumeUsesCurrentSealAtAdmission",
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

/// Decision66459 including Darwin correction and amendment2. Fixed pinned tools
/// supply the fork/exec shape premise; notifications/censuses cannot prove it.
/// CPU units abstract <=250ms/over, <=3s job/over; debt is exact 9x in these
/// representatives. Pause owns one STOP, same-owner positive ACK and one CONT;
/// raw USER=0 and SYSTEM<=50us are separate gates. No OS-success fairness.
pub fn total_budget(control: u8) -> Model {
    let names = [
        "",
        "Argv",
        "Environment",
        "Gs1",
        "SelfQos",
        "ObservedFork",
        "Wait4Unknown",
        "CommandCpu",
        "JobCpu",
        "Rss",
        "Debt",
        "Reap",
        "Census",
        "Gs2",
        "PauseUser",
        "PauseSystem",
        "DoubleContinue",
        "LiveOverCap",
        "FailedLiveReadRefuses",
        "Deadline",
    ];
    assert!(control < names.len() as u8);
    let mut m = base(
        "HelperShimBudget",
        control != 0,
        &[
            ("phaseAbsentPreparedExecObservedReapedSettled", 5, 0),
            ("scenario", 19, 0),
            ("executed", 1, 0),
            ("killed", 1, 0),
            ("refused", 1, 0),
            ("actualOwnWait4", 1, 0),
            ("trusted", 1, 0),
            ("rawTerminalCpu", 3, 0),
            ("debtDue", 27, 0),
            ("restPaid", 27, 0),
            ("liveReadFailed", 1, 0),
            ("liveReadOverCap", 1, 0),
            ("stopCount", 1, 0),
            ("positiveAck", 1, 0),
            ("continueCount", 2, 0),
            ("pauseUserGrew", 1, 0),
            ("pauseSystemOver50us", 1, 0),
            ("nextExpensive", 1, 0),
        ],
    );
    m.name = format!(
        "{}HelperShimBudget{}",
        if control == 0 { "" } else { "BrokenHarness" },
        names[control as usize]
    );
    const P: usize = 0;
    const C: usize = 1;
    const EXEC: usize = 2;
    const KILL: usize = 3;
    const REFUSE: usize = 4;
    const REAP: usize = 5;
    const TRUST: usize = 6;
    const CPU: usize = 7;
    const DUE: usize = 8;
    const PAID: usize = 9;
    const FAILED: usize = 10;
    const OVER: usize = 11;
    const STOP: usize = 12;
    const ACK: usize = 13;
    const CONT: usize = 14;
    const USER: usize = 15;
    const SYSTEM: usize = 16;
    const NEXT: usize = 17;
    for case in 0..=19 {
        act(
            &mut m,
            &format!("PrepareBoundFormScenario{case}"),
            eq(P, 0),
            &[
                (P, Int(1)),
                (C, Int(case)),
                (CPU, Int(if case == 7 { 3 } else { 1 })),
                (FAILED, Int(i64::from(case == 18))),
                (OVER, Int(i64::from(case == 17))),
                (USER, Int(i64::from(case == 14))),
                (SYSTEM, Int(i64::from(case == 15))),
            ],
        );
    }
    let pre = all((1..=4).map(|bad| eq(C, bad).negate().or(Bool(control == bad as u8))));
    act(
        &mut m,
        "ShimChecksExactArgvEnvironmentGs1AndOwnQosThenExec",
        eq(P, 1).and(pre.clone()),
        &[(P, Int(2)), (EXEC, Int(1))],
    );
    act(
        &mut m,
        "ShimMismatchExits81BeforeExec",
        eq(P, 1).and(pre.negate()),
        &[(P, Int(5)), (REFUSE, Int(1))],
    );
    let fork_or_live = eq(C, 5)
        .and(Bool(control != 5))
        .or(eq(C, 17).and(Bool(control != 17)));
    act(
        &mut m,
        "ObservedUnexpectedForkOrSuccessfulOverCapReadKills",
        eq(P, 2).and(fork_or_live.clone()),
        &[(P, Int(3)), (KILL, Int(1)), (REFUSE, Int(1))],
    );
    act(
        &mut m,
        "OrdinaryLiveReadFailedIgnoredOrWithinCap",
        eq(P, 2).and(fork_or_live.negate()),
        &[
            (P, Int(3)),
            (REFUSE, if control == 18 { Var(FAILED) } else { Int(0) }),
        ],
    );
    for case in 0..=19 {
        let raw = if case == 7 { 3 } else { 1 };
        let due = 9 * raw;
        act(
            &mut m,
            &format!("SameOwnerPauseReapAndFullRawBill{case}"),
            all([eq(P, 3), eq(C, case)]),
            &[
                (P, Int(4)),
                (STOP, Int(1)),
                (ACK, Int(1)),
                (CONT, Int(if case == 16 && control == 16 { 2 } else { 1 })),
                (REAP, Int(i64::from(case != 11))),
                (DUE, Int(due)),
                (PAID, Int(if case == 10 && control == 10 { 0 } else { due })),
            ],
        );
    }
    let terminal = all([6, 7, 8, 9, 11, 12, 13, 14, 15, 19]
        .into_iter()
        .map(|bad| eq(C, bad).negate().or(Bool(control == bad as u8))));
    let bill = Var(PAID).eq(Var(DUE));
    let accept = all([
        eq(REFUSE, 0),
        terminal.clone(),
        if control == 10 {
            Bool(true)
        } else {
            bill.clone()
        },
    ]);
    act(
        &mut m,
        "TrustOnlyAfterExactWait4CapsDebtCensusGs2AndPauseVerdict",
        eq(P, 4).and(accept.clone()),
        &[(P, Int(5)), (TRUST, Int(1)), (NEXT, Int(1))],
    );
    act(
        &mut m,
        "RefuseRetainingFullTerminalBillAndExcess",
        eq(P, 4).and(accept.negate()),
        &[(P, Int(5)), (REFUSE, Int(1))],
    );
    for (case, name) in [
        (1, "ExactArgvBeforeExec"),
        (2, "ExactEnvironmentBeforeExec"),
        (3, "InProcessGs1BeforeExec"),
        (4, "SelfBackgroundQosBeforeExec"),
    ] {
        safe(&mut m, name, eq(EXEC, 0).or(eq(C, case).negate()));
    }
    safe(
        &mut m,
        "ObservedUnexpectedForkRefuses",
        eq(P, 3)
            .or(eq(P, 4))
            .or(eq(P, 5))
            .negate()
            .or(eq(C, 5).negate())
            .or(eq(EXEC, 0))
            .or(all([eq(KILL, 1), eq(REFUSE, 1)])),
    );
    safe(
        &mut m,
        "SuccessfulOverCapReadKills",
        Var(P)
            .lt(Int(3))
            .or(eq(C, 17).negate())
            .or(eq(EXEC, 0))
            .or(eq(KILL, 1)),
    );
    safe(
        &mut m,
        "FailedLiveReadIsIgnored",
        Var(P).lt(Int(3)).or(eq(FAILED, 0)).or(eq(REFUSE, 0)),
    );
    for (case, name) in [
        (6, "KnownOwnWait4"),
        (7, "Command250ms"),
        (8, "Job3s"),
        (9, "TerminalRss16MiB"),
        (11, "ActualReap"),
        (12, "FinalKnownCensus"),
        (13, "Gs2AfterReap"),
        (14, "PauseUserZero"),
        (15, "PauseSystem50us"),
        (19, "DeadlineRefuses"),
    ] {
        safe(&mut m, name, eq(TRUST, 0).or(eq(C, case).negate()));
    }
    safe(&mut m, "DebtBeforeNext", eq(NEXT, 0).or(bill));
    safe(&mut m, "OneStopAckAndOneContinue", Var(CONT).lt(Int(2)));
    safe(
        &mut m,
        "Full9xRawTerminalBill",
        eq(P, 4)
            .or(eq(P, 5))
            .negate()
            .or(eq(EXEC, 0))
            .or(eq(CPU, 3).and(eq(DUE, 27)).or(eq(CPU, 1).and(eq(DUE, 9)))),
    );
    let fairness = (0..m.actions.len()).collect();
    progress(
        &mut m,
        "EventuallySettledWithControlAndSettlementFair",
        eq(P, 1),
        eq(P, 5),
        fairness,
    );
    m
}
#[cfg(test)]
mod total_budget_tests {
    use super::*;
    #[test]
    fn helper_shim_budget_and_nineteen_gate_controls() {
        let good = total_budget(0).check(10000).unwrap();
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        for control in 1..=19 {
            let result = total_budget(control).check(10000).unwrap();
            assert!(!result.violations.is_empty(), "control {control} escaped");
        }
    }
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
            ("sourceLifetimeCoverage", 1, 0),
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
    const SOURCE_COVERED: usize = 19;
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
        &[(PH, Int(4)), (SOURCE_COVERED, Int(1))],
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
            eq(SOURCE_COVERED, 1),
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
            eq(SOURCE_COVERED, 1),
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
        "PrivateForkExecUsesExistingReservedTwoMemberOwner",
        all([
            eq(MODE, 2),
            eq(PH, 3),
            eq(EFFECTS, 1),
            eq(OWN, 1),
            eq(DEBT, 0),
            eq(REFUSED, 0),
            current.clone(),
        ]),
        &[(PH, Int(4)), (SOURCE_COVERED, Int(1))],
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
        for control in [
            0, 1, 2, 3, 4, 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 21, 22, 23,
        ] {
            let full_model = helper_observer(control, true);
            let lowered_model = helper(control);
            assert_eq!(full_model.name, lowered_model.name);
            assert_eq!(
                format!("{:?}", full_model.variables),
                format!("{:?}", lowered_model.variables)
            );
            assert_eq!(
                format!("{:?}", full_model.invariants),
                format!("{:?}", lowered_model.invariants)
            );
            assert_eq!(
                format!("{:?}", full_model.responses),
                format!("{:?}", lowered_model.responses)
            );
            assert_eq!(full_model.weak_fair, lowered_model.weak_fair);
            assert_eq!(full_model.check_deadlock, lowered_model.check_deadlock);
            assert_eq!(full_model.actions.len(), lowered_model.actions.len());
            for (full, lowered) in full_model.actions.iter().zip(&lowered_model.actions) {
                assert_eq!(full.name, lowered.name);
                assert_eq!(
                    format!("{:?}", full.updates),
                    format!("{:?}", lowered.updates)
                );
            }
            let full = full_model.check(100_000).unwrap();
            let lowered = lowered_model.check(100_000).unwrap();
            assert_eq!(
                full.states, lowered.states,
                "control{control} complete states"
            );
            assert_eq!(full.edges.len(), lowered.edges.len());
            for (full, lowered) in full.edges.iter().zip(&lowered.edges) {
                assert_eq!(
                    full.iter()
                        .map(|edge| (edge.action, edge.to))
                        .collect::<Vec<_>>(),
                    lowered
                        .iter()
                        .map(|edge| (edge.action, edge.to))
                        .collect::<Vec<_>>(),
                    "control{control} complete labelled edges"
                );
            }
            assert_eq!(
                full.violations
                    .iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle))
                    .collect::<Vec<_>>(),
                lowered
                    .violations
                    .iter()
                    .map(|v| (&v.property, &v.prefix, &v.cycle))
                    .collect::<Vec<_>>(),
                "control{control} named verdicts and complete prefix/cycle traces"
            );
            println!(
                "helper_observer_equivalence control={control} model={} states={} edges={} named_verdicts={:?}",
                full_model.name,
                full.states.len(),
                full.edges.iter().map(Vec::len).sum::<usize>(),
                full.violations
                    .iter()
                    .map(|v| &v.property)
                    .collect::<Vec<_>>()
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
                )
                .unwrap();
                helper_observer_lowering_export::export(
                    lowered_model,
                    expected,
                    &root.join("lowered"),
                    &mut lowered_manifest,
                )
                .unwrap();
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
            "new GPG task may execute while the source helper runs"
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
    fn dream_read_helper_admission_native_and_original_custody_controls() {
        let good = helper(0).check(100_000).unwrap();
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(
            good.states
                .iter()
                .any(|s| s[18] == 1 && s[20] == 1 && s[21] == 1)
        );
        assert!(
            good.states.iter().any(|s| s[3] == 2 && s[6] == 1),
            "signature consumes both reservations before one actual leader"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[0] == 7 && s[14] == 1 && s[20] == 0),
            "unavailable settlement keeps original custody"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[8] == 5 && s[5] == 1 && s[10] == 3 && s[11] == 3),
            "unknown native cost retains full conservative native debt"
        );
        assert!(
            good.states
                .iter()
                .any(|s| s[1] == 2 && s[2] == 0 && s[5] == 1 && s[14] == 1),
            "A1 effect is retained across A1/B2/A3"
        );
        for (case, verdict, excess, count, known) in [
            (1, 1, 0, 0, 1),
            (2, 1, 9, 1, 1),
            (3, 1, 225, 1, 1),
            (4, 2, 226, 1, 1),
            (5, 1, 0, 0, 0),
            (6, 2, 0, 0, 0),
            (7, 2, 9, 1, 1),
            (8, 2, 9, 1, 1),
            (9, 2, 0, 0, 0),
            (10, 2, 0, 0, 0),
        ] {
            assert!(
                good.states.iter().any(|s| s[30] == case
                    && s[36] == verdict
                    && s[37] == excess
                    && s[38] == count
                    && s[39] == known),
                "exact tagged bound verdict and unclamped telemetry case{case}"
            );
        }
        let old_wall = helper(23).check(100_000).unwrap();
        assert!(
            old_wall
                .violations
                .iter()
                .any(|v| v.property == "TaggedNativeVerdictMatchesBoundCase2"
                    && !v.prefix.is_empty()
                    && old_wall.states[*v.prefix.last().unwrap()][30] == 2),
            "old25ms control must over-refuse the otherwise valid34ms process_create positive"
        );
        assert!(
            old_wall
                .states
                .iter()
                .any(|s| s[30] == 2 && s[36] == 2 && s[37] == 9 && s[38] == 1),
            "old-bound refusal still retains exact9ms excess rather than a TypeOK failure"
        );
        for (control, property) in [
            (1, "NativeEffectRequiresReservedAttempt"),
            (2, "TrustedOutputHasOnlyAdmittedMembers"),
            (3, "NativeEffectRequiresExactExecutableCapability"),
            (4, "AttachedWorkKeepsExactIncarnation"),
            (9, "MainProofCannotOverlapRunnableGroup"),
            (10, "DebtCoversWholeObservedUnrestedSpan"),
            (11, "OrdinaryApplicationCannotUseNativeLabel"),
            (12, "CertifiedNativeCreationMeetsCpuWallInclusiveBounds"),
            (13, "CertifiedNativeCreationMeetsCpuWallInclusiveBounds"),
            (14, "CertifiedNativeCreationMeetsCpuWallInclusiveBounds"),
            (16, "OriginalNativeEffectNeverReplayed"),
            (17, "TrustedOutputHasCompleteCoverage"),
            (18, "TrustedOutputHasActualReapAndRetirement"),
            (19, "FaultCannotReleaseUnsettledOwnership"),
            (20, "FaultCannotReleaseUnsettledOwnership"),
            (21, "FaultCannotReleaseUnsettledOwnership"),
            (22, "StopAttemptIsNotRetriedAfterAnyNativeOutcome"),
            (23, "TaggedNativeVerdictMatchesBoundCase2"),
        ] {
            let m = helper(control);
            let checked = m.check(100_000).unwrap();
            assert!(!checked.violations.iter().any(|v| v.property == "TypeOK"));
            assert!(
                checked.violations.iter().any(|v| v.property == property),
                "{} must violate {property}",
                m.name
            );
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

// Task66427: standalone finite companion of the foreground STOP receipt.
// Factored from the disjoint Init branch; legacy TotalBudget stays byte-identical.
// Origin/credit: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// It adds no ACK, exit, own-wait4, group absence or numeric cost from errno.
// Native success and rest observations remain unfair; missing facts retain
// the original owner. Numeric timestamps/OS errno causes remain C witnesses.
pub fn foreground_stop_terminal(control: u8) -> Model {
    let mut m = base("DreamReadForegroundStopTerminal", control != 0, &[]);
    if control != 0 {
        m.name = "BrokenHarnessHelperShimBudgetMissingForegroundTerminal".into();
    }
    const CASE: usize = 0;
    const PHASE: usize = 1;
    const ERR: usize = 2; //0 none,1 ESRCH,2 EPERM,3 EINVAL,4 other
    const ZOMBIE_ONLY: usize = 3;
    const ATTEMPT: usize = 4;
    const ACK: usize = 5;
    const OWN_REAP: usize = 6;
    const GONE: usize = 7;
    const RAW_CPU: usize = 8;
    const CPU_KNOWN: usize = 9;
    const BILL: usize = 10;
    const DUE: usize = 11;
    const PAID: usize = 12;
    const PENDING: usize = 13;
    const RESULT: usize = 14; //0 undecided,1 complete verdict,2 refusal
    const OWNER: usize = 15;
    const RESUME: usize = 16;
    const TERMINAL: usize = 17;
    for (name, maximum) in [
        ("foregroundStopRaceCase", 13),
        ("foregroundStopRacePhase", 7),
        ("foregroundStopRawNoneEsrchEpermEinvalOther", 4),
        ("foregroundStopPositiveZombieOnlyGroup", 1),
        ("foregroundStopOriginalAttemptCount", 1),
        ("foregroundStopPositiveAck", 1),
        ("foregroundStopActualOriginalOwnWait4", 1),
        ("foregroundStopIndependentGroupGone", 1),
        ("foregroundStopRawWait4CpuOrUnknown", 4),
        ("foregroundStopWait4CpuKnown", 1),
        ("foregroundStopAuthoritativeCpuBill", 3),
        ("foregroundStopFullFreshDebtDue", 27),
        ("foregroundStopActualFreshRestPaid", 27),
        ("foregroundStopPendingTerminal", 1),
        ("foregroundStopFinalDisposition", 2),
        ("foregroundStopSameOriginalOwnerHeld", 1),
        ("foregroundStopResumeCount", 1),
        ("foregroundStopIndependentTerminalObserved", 1),
    ] {
        m.variables.push(Variable::new(name, 0, maximum, 0));
    }
    // (raw errno, positive zombie-only, actual own reap, group gone,
    //  raw CPU class, counter known, observed full payment).
    // Unknown CPU uses raw class4 and the existing conservative27 pacing
    // representative, never a finite authoritative bill or release credit.
    let cases = [
        (1, 0, 1, 1, 1, 1, 1), //ESRCH, later original ownwait4/full bill
        (2, 1, 1, 1, 1, 1, 1), //EPERM plus positive zombie-only proof
        (2, 0, 1, 1, 1, 1, 1), //EPERM without zombie-only proof refuses
        (1, 0, 0, 0, 0, 0, 0), //ESRCH without actual ownwait4 stays pending
        (3, 1, 1, 1, 1, 1, 1), //EINVAL is never terminal-before-stop
        (4, 1, 1, 1, 1, 1, 1), //all other errors remain refusal
        (1, 0, 1, 0, 1, 1, 1), //ownwait4 alone is not group-gone
        (1, 0, 1, 1, 1, 1, 0), //unpaid full known debt retains owner
        (1, 0, 1, 1, 3, 1, 1), //raw over-cap CPU fully billed then refused
        (1, 0, 1, 1, 4, 0, 1), //unknown raw CPU never numeric credit
        (1, 0, 1, 1, 0, 1, 1), //zero CPU still owes full ordinary floor9
        (2, 1, 0, 0, 0, 0, 0), //zombie-only EPERM still needs ownwait4
        (1, 0, 1, 1, 2, 1, 1), //exact abstract CPU cap/full18 debt
    ];
    for (index, &(error, zombie, reaped, gone, raw, known, paid)) in cases.iter().enumerate() {
        let case = index as i64 + 1;
        let eligible = error == 1 || (error == 2 && zombie == 1);
        let due = if reaped == 0 {
            0
        } else if known == 0 {
            27
        } else {
            9 * raw.max(1)
        };
        act(
            &mut m,
            &format!("SelectOriginalForegroundStopRaceCase{case}"),
            eq(CASE, 0),
            &[(CASE, Int(case)), (OWNER, Int(1))],
        );
        act(
            &mut m,
            &format!("RequestForegroundPauseBeforeOriginalStopAttempt{case}"),
            all([eq(CASE, case), eq(PHASE, 0), eq(OWNER, 1), eq(OWN_REAP, 0)]),
            &[(PHASE, Int(1))],
        );
        act(
            &mut m,
            &format!("RecordActualFailedOriginalForegroundStopErrno{error}Case{case}"),
            all([
                eq(CASE, case),
                eq(PHASE, 1),
                eq(OWN_REAP, 0),
                eq(ATTEMPT, 0),
            ]),
            &[
                (PHASE, Int(2)),
                (ERR, Int(error)),
                (ATTEMPT, Int(1)),
                (PENDING, Int(i64::from(error == 1 || error == 2))),
            ],
        );
        act(
            &mut m,
            &format!("ObserveIndependentOriginalTerminalAfterStopReceipt{case}"),
            all([eq(CASE, case), eq(PHASE, 2)]),
            &[
                (PHASE, Int(3)),
                (TERMINAL, Int(1)),
                (ZOMBIE_ONLY, Int(zombie)),
            ],
        );
        act(
            &mut m,
            &format!("ObserveActualOriginalOwnWait4OrItsAbsenceCase{case}"),
            all([eq(CASE, case), eq(PHASE, 3)]),
            &[
                (PHASE, Int(4)),
                (OWN_REAP, Int(reaped)),
                (RAW_CPU, Int(raw)),
                (CPU_KNOWN, Int(known)),
                (BILL, Int(if known == 1 { raw } else { 0 })),
                (DUE, Int(due)),
            ],
        );
        act(
            &mut m,
            &format!("ObserveIndependentOriginalGroupGoneOrMissingCase{case}"),
            all([eq(CASE, case), eq(PHASE, 4)]),
            &[(PHASE, Int(5)), (GONE, Int(gone))],
        );
        act(
            &mut m,
            &format!("ObserveFullFreshWait4DebtPaymentOrUnavailableCase{case}"),
            all([eq(CASE, case), eq(PHASE, 5)]),
            &[
                (PHASE, Int(6)),
                (PAID, Int(if paid == 1 { due } else { 0 })),
            ],
        );
        let proof = eligible && reaped == 1 && gone == 1 && known == 1 && paid == 1;
        let resolve = proof && control != 1;
        act(
            &mut m,
            &format!("ResolveForegroundStopOnlyFromOriginalPaidTerminalProofCase{case}"),
            all([eq(CASE, case), eq(PHASE, 6)]),
            &[
                (PHASE, Int(7)),
                (RESULT, Int(if resolve && raw < 3 { 1 } else { 2 })),
                (PENDING, if resolve { Int(0) } else { Var(PENDING) }),
                (OWNER, Int(i64::from(!resolve))),
            ],
        );
    }
    let eligible = eq(ERR, 1).or(all([eq(ERR, 2), eq(ZOMBIE_ONLY, 1)]));
    let physical_paid = all([
        eligible.clone(),
        eq(ATTEMPT, 1),
        eq(TERMINAL, 1),
        eq(OWN_REAP, 1),
        eq(GONE, 1),
        eq(CPU_KNOWN, 1),
        Var(BILL).eq(Var(RAW_CPU)),
        Var(PAID).eq(Var(DUE)),
        Int(0).lt(Var(DUE)),
    ]);
    let accepted = physical_paid.clone().and(Var(RAW_CPU).lt(Int(3)));
    safe(
        &mut m,
        "ForegroundStopErrnoNeverSuppliesAckOrResume",
        all([eq(ACK, 0), eq(RESUME, 0)]),
    );
    safe(
        &mut m,
        "ForegroundStopPendingRetainsRawReceiptAndOriginalOwner",
        eq(PENDING, 0).or(all([
            eq(OWNER, 1),
            eq(ATTEMPT, 1),
            eq(ERR, 1).or(eq(ERR, 2)),
        ])),
    );
    safe(
        &mut m,
        "ForegroundStopAuthoritativeBillAndFullDebtNeverClamp",
        eq(OWN_REAP, 0).or(eq(CPU_KNOWN, 0)).or(all([
            Var(BILL).eq(Var(RAW_CPU)),
            eq(DUE, 9)
                .and(eq(RAW_CPU, 0).or(eq(RAW_CPU, 1)))
                .or(all([eq(RAW_CPU, 2), eq(DUE, 18)]))
                .or(all([eq(RAW_CPU, 3), eq(DUE, 27)])),
        ])),
    );
    safe(
        &mut m,
        "ForegroundStopCompletionNeedsOriginalReapGroupAndPaidBill",
        eq(RESULT, 1).negate().or(accepted.clone()),
    );
    safe(
        &mut m,
        "ForegroundStopDispositionMatchesOriginalPaidTerminalProof",
        eq(PHASE, 7).negate().or(accepted
            .clone()
            .and(eq(RESULT, 1))
            .or(accepted.negate().and(eq(RESULT, 2)))),
    );
    // Preserve the original broken observer's stopped-at-first-failure loop.
    // This changes no reachable outcome; it keeps its labelled failing graph.
    if control != 0 {
        let valid = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid.clone()).eq(Bool(true));
        }
        act(
            &mut m,
            "ObserverStopsAtFirstFailedContract",
            valid.negate().eq(Bool(true)),
            &[],
        );
    }
    m
}

#[cfg(test)]
mod foreground_stop_terminal_tests {
    use super::*;
    #[test]
    fn foreground_stop_terminal_keeps_all_thirteen_cases_and_missing_path_control() {
        let good_model = foreground_stop_terminal(0);
        let good = good_model.check(100_000).unwrap();
        assert_eq!(good_model.variables.len(), 18);
        assert_eq!(good_model.actions.len(), 104);
        assert_eq!(good_model.invariants.len(), 5);
        assert!(good_model.responses.is_empty());
        assert!(good_model.weak_fair.is_empty());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert_eq!(good.states.len(), 105);
        assert_eq!(good.edges.iter().map(Vec::len).sum::<usize>(), 104);
        // All thirteen correlated outcomes survive factoring without added fairness.
        for case in 1..=13 {
            let accepted = matches!(case, 1 | 2 | 11 | 13);
            assert!(
                good.states.iter().any(|s| s[0] == case
                    && s[1] == 7
                    && s[14] == (if accepted { 1 } else { 2 })
                    && s[4] == 1
                    && s[5] == 0
                    && s[16] == 0),
                "failed foreground STOP case{case} reaches exact paid-terminal verdict or refusal without ACK/CONT"
            );
            assert!(
                good.states.iter().any(|s| s[0] == case
                    && s[1] == 2
                    && s[2] != 0
                    && s[4] == 1
                    && s[5] == 0
                    && s[6] == 0
                    && s[7] == 0
                    && s[11] == 0
                    && s[15] == 1),
                "raw failed STOP case{case} supplies neither ownwait4/groupgone nor numeric bill"
            );
        }
        for case in [1, 2] {
            assert!(
                good.states.iter().any(|s| s[0] == case
                    && s[1] == 7
                    && s[6] == 1
                    && s[7] == 1
                    && s[8] == 1
                    && s[10] == 1
                    && s[11] == 9
                    && s[12] == 9
                    && s[13] == 0
                    && s[14] == 1
                    && s[15] == 0
                    && s[17] == 1),
                "ESRCH or zombie-only EPERM resolves only after actual ownreap/groupgone/full raw bill and fresh payment"
            );
        }
        for case in [4, 12] {
            assert!(
                good.states.iter().any(|s| s[0] == case
                    && s[1] == 7
                    && s[6] == 0
                    && s[13] == 1
                    && s[14] == 2
                    && s[15] == 1),
                "independent terminal observation without original ownwait4 retains pending owner"
            );
        }
        assert!(
            good.states.iter().any(|s| s[0] == 3
                && s[1] == 7
                && s[2] == 2
                && s[3] == 0
                && s[6] == 1
                && s[7] == 1
                && s[12] == 9
                && s[14] == 2),
            "EPERM without positive zombie-only group proof never becomes terminal-before-stop"
        );
        for case in [5, 6] {
            assert!(
                good.states.iter().any(|s| s[0] == case
                    && s[1] == 7
                    && s[2] >= 3
                    && s[6] == 1
                    && s[7] == 1
                    && s[10] == 1
                    && s[12] == 9
                    && s[14] == 2),
                "EINVAL/other error remains refusal despite independently completed cost/physical proof"
            );
        }
        assert!(
            good.states.iter().any(|s| s[0] == 7
                && s[1] == 7
                && s[6] == 1
                && s[7] == 0
                && s[12] == 9
                && s[15] == 1),
            "ownwait4 without groupgone cannot resolve failed foreground STOP"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 8
                && s[1] == 7
                && s[6] == 1
                && s[7] == 1
                && s[11] == 9
                && s[12] == 0
                && s[13] == 1
                && s[14] == 2
                && s[15] == 1),
            "full unpaid known debt remains retained after physical proof"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 9
                && s[1] == 7
                && s[8] == 3
                && s[10] == 3
                && s[11] == 27
                && s[12] == 27
                && s[13] == 0
                && s[14] == 2),
            "raw CPU overcap is fully billed and paid without converting terminal race resolution to budget success"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 10
                && s[1] == 7
                && s[8] == 4
                && s[9] == 0
                && s[10] == 0
                && s[11] == 27
                && s[12] == 27
                && s[14] == 2
                && s[15] == 1),
            "unknown authoritative CPU keeps raw unknown and no finite credit despite conservative pacing"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 11
                && s[1] == 7
                && s[8] == 0
                && s[10] == 0
                && s[11] == 9
                && s[12] == 9
                && s[14] == 1),
            "zero CPU retains full fresh floor-times9"
        );
        assert!(
            good.states.iter().any(|s| s[0] == 13
                && s[1] == 7
                && s[8] == 2
                && s[10] == 2
                && s[11] == 18
                && s[12] == 18
                && s[14] == 1),
            "exact abstract CPU cap retains full unrounded bill and debt"
        );

        let bad_model = foreground_stop_terminal(1);
        let bad = bad_model.check(100_000).unwrap();
        assert!(!bad.violations.iter().any(|v| v.property == "TypeOK"));
        assert_eq!(bad.states.len(), 105);
        assert_eq!(bad.edges.iter().map(Vec::len).sum::<usize>(), 108);
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == "ForegroundStopDispositionMatchesOriginalPaidTerminalProof")
        );
        assert!(
            bad.states.iter().any(|s| s[0] == 1
                && s[1] == 7
                && s[2] == 1
                && s[4] == 1
                && s[5] == 0
                && s[6] == 1
                && s[7] == 1
                && s[8] == 1
                && s[10] == 1
                && s[11] == 9
                && s[12] == 9
                && s[13] == 1
                && s[14] == 2
                && s[15] == 1),
            "omitting terminal-before-stop path falsely refuses fully paid actual original settlement, not TypeOK"
        );
        let output = std::env::var_os("DREAM_FOREGROUND_STOP_TERMINAL_FORMAL_OUT")
            .map(std::path::PathBuf::from);
        let mut manifest = String::new();
        for (model, checked, expected) in [(good_model, good, "pass"), (bad_model, bad, "safety")] {
            println!(
                "{} states={} labelled_edges={} named={:?}",
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
                std::fs::create_dir_all(root).unwrap();
                super::helper_observer_lowering_export::export(
                    model,
                    expected,
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

// Task66445: separate failed-observation protocol; every legacy model above is
// byte-preserved. Origin/credit policy:
// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Names/IDs match the C fixed enum. Native-domain normalization is an explicit
// source premise: Unix ESRCH and gs1_procargs-only EINVAL differ from Mach
// TASK_NAME KERN_FAILURE/null port and TASK_INFO invalidated-port errors.
// Unix ESRCH is not a Mach-site error. ZOMBIE_ROW/raw0 is a distinct candidate
// only at live TASKALL opening/closing or prekill BSD after validated lineage.
// Numeric equality across domains supplies no proof. This model observes
// failed live-observation predicates only; it never
// manufactures a successful syscall, census, registered watch, GS1 or ACK.
pub const DREAM_READ_OBSERVATION_SITES: [&str; 27] = [
    "live_taskall_open",
    "audit_taskall_before",
    "audit_task_name",
    "audit_task_name_null",
    "audit_task_info",
    "audit_taskall_after",
    "live_image_path",
    "live_rusage",
    "live_taskall_close",
    "gs1_image_path",
    "gs1_procargs",
    "child_stop_signal",
    "child_continue_signal",
    "initial_continue_signal",
    "foreground_stop_signal",
    "foreground_continue_signal",
    "prekill_bsd",
    "final_group_kill",
    "children_census",
    "group_census",
    "event_register",
    "event_poll",
    "event_delivery",
    "handoff_rusage",
    "escaped_cleanup_bsd",
    "escaped_cleanup_signal",
    "owned_terminal_waitid",
];

// (normalized error class, independent terminal kind, original owned pin,
//  unreaped at call, current identity class, prior sticky failure, later actual
//  ownwait4, later groupgone, raw CPU class, known CPU, actual full payment).
// Error:1 domain-correct gone candidate,2 other native/domain error,
// 3 existing signal EPERM pending,4 successful-read validation/stale errno.
// Terminal:0 live/absent,1 RIexit,2 process-bound knote,3 same-original SZOMB
// (for EPERM, this COMPOSITE fact additionally includes independent positive
// zombie-only proof for the WHOLE original group, not just the leader),4 unknown.
// Identity:0 incomplete,1 exact PID/birth/start/full-current-token,
// 2 foreign birth,3 foreign start,4 wrong token/current row.
const OBSERVATION_CASES: [[i64; 11]; 25] = [
    [1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1], //same-original RI exit
    [1, 2, 1, 1, 1, 0, 1, 1, 1, 1, 1], //same-process retained NOTE_EXIT
    [1, 3, 1, 1, 1, 0, 1, 1, 1, 1, 1], //same-original SZOMB
    [2, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1], //other error never rescued
    [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0], //still live; no exit from error
    [1, 4, 1, 1, 1, 0, 0, 0, 0, 0, 0], //unknown terminal proof
    [1, 1, 1, 1, 2, 0, 0, 0, 0, 0, 0], //foreign birth
    [1, 1, 1, 1, 3, 0, 0, 0, 0, 0, 0], //foreign start
    [1, 1, 1, 1, 4, 0, 0, 0, 0, 0, 0], //full token/current row mismatch
    [1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0], //unowned effect
    [1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0], //already reaped: no live authority
    [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0], //terminal takeover, ownwait4 absent
    [1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1], //ownwait4 alone not groupgone
    [1, 1, 1, 1, 1, 0, 1, 1, 4, 0, 1], //unknown raw CPU, no finite credit
    [1, 1, 1, 1, 1, 0, 1, 1, 3, 1, 1], //full overcap bill/debt, then refusal
    [1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1], //zero CPU keeps fresh floor9
    [1, 1, 1, 1, 1, 0, 1, 1, 2, 1, 1], //exact abstract cap/full18
    [1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0], //full known debt remains unpaid
    [4, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1], //success+invalid read is not stale ESRCH
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], //independent prior failure never clears
    [3, 3, 1, 1, 1, 0, 1, 1, 1, 1, 1], //EPERM: whole-group zombie proof, pending signal sites only
    [3, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0], //EPERM without zombie-only group proof
    [1, 4, 1, 1, 1, 0, 0, 0, 0, 0, 0], //unavailable independent recheck
    [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0], //incomplete retained identity
    [2, 3, 1, 1, 1, 0, 1, 1, 1, 1, 1], //equal numeric codes in wrong native domain
];

pub fn systematic_observe_terminal(control: u8) -> Model {
    let mut m = base("DreamReadObserveOrTerminal", control != 0, &[]);
    if control != 0 {
        m.name = "BrokenHarnessDreamReadObserveOrTerminalBypassSite".into();
    }
    const SITE: usize = 0;
    const CASE: usize = 1;
    const P: usize = 2;
    const ERR: usize = 3;
    const OWNED: usize = 4;
    const UNREAPED: usize = 5;
    const ID: usize = 6;
    const PRIOR: usize = 7;
    const TERMINAL: usize = 8;
    const ROUTE: usize = 9;
    const PENDING: usize = 10;
    const REAP: usize = 11;
    const GONE: usize = 12;
    const RAW: usize = 13;
    const KNOWN: usize = 14;
    const BILL: usize = 15;
    const DUE: usize = 16;
    const PAID: usize = 17;
    const OWNER: usize = 18;
    const ACK: usize = 19;
    const CONT: usize = 20;
    const WORK: usize = 21;
    const SUCCESS: usize = 22;
    const REFUSAL: usize = 23;
    const RESULT: usize = 24;
    for (name, max) in [
        ("observationSite", 27),
        ("observationCase", 25),
        ("observationPhase", 8),
        ("siteNormalizedRawFailureClass", 4),
        ("originalOwnedPin", 1),
        ("originalUnreapedAtCall", 1),
        ("originalCurrentIdentityClass", 4),
        ("priorStickyFailure", 1),
        ("independentlyObservedTerminalKind", 4),
        ("failedObservationRoute", 3),
        ("existingSignalEffectPending", 1),
        ("actualOriginalOwnWait4", 1),
        ("independentOriginalGroupGone", 1),
        ("rawAuthoritativeCpuOrUnknown", 4),
        ("authoritativeCpuKnown", 1),
        ("fullUnclampedCpuBill", 3),
        ("fullFreshDebtDue", 27),
        ("actualFullRestPaid", 27),
        ("sameOriginalCustodyHeld", 1),
        ("positiveStopAck", 1),
        ("newContinueCount", 1),
        ("newProofWork", 1),
        ("fabricatedNativeSuccess", 1),
        ("stickyRefusal", 1),
        ("finalAcceptedOrRefused", 2),
    ] {
        m.variables.push(Variable::new(name, 0, max, 0));
    }
    for (index, name) in DREAM_READ_OBSERVATION_SITES.iter().enumerate() {
        act(
            &mut m,
            &format!("SelectOwnedObservationSite_{}_{name}", index + 1),
            eq(P, 0),
            &[(P, Int(1)), (SITE, Int(index as i64 + 1))],
        );
    }
    for (index, c) in OBSERVATION_CASES.iter().enumerate() {
        act(
            &mut m,
            &format!(
                "CaptureSiteFailureAndRetainedOriginalFactsCase{}",
                index + 1
            ),
            eq(P, 1),
            &[
                (P, Int(2)),
                (CASE, Int(index as i64 + 1)),
                (ERR, Int(c[0])),
                (OWNED, Int(c[2])),
                (UNREAPED, Int(c[3])),
                (ID, Int(c[4])),
                (PRIOR, Int(c[5])),
                (REFUSAL, Int(c[5])),
                (OWNER, Int(1)),
            ],
        );
    }
    // Pinned bboxy450-tlc0.1.1 has no conditional expression. Lower each
    // piecewise update into mutually exclusive guarded actions, preserving
    // every profile/phase/state value without changing the tool or bounds.
    let selected_cases = |field: usize, value: i64| {
        OBSERVATION_CASES
            .iter()
            .enumerate()
            .filter(|(_, c)| c[field] == value)
            .fold(Bool(false), |guard, (i, _)| {
                guard.or(eq(CASE, i as i64 + 1))
            })
    };
    for kind in 0..=4 {
        act(
            &mut m,
            &format!("ObserveIndependentSameOriginalTerminalKind{kind}"),
            all([eq(P, 2), selected_cases(1, kind)]),
            &[(P, Int(3)), (TERMINAL, Int(kind))],
        );
    }
    let pending_site = eq(SITE, 15).or(eq(SITE, 18));
    let identity = all([eq(OWNED, 1), eq(UNREAPED, 1), eq(ID, 1)]);
    let positive_terminal = Int(0).lt(Var(TERMINAL)).and(Var(TERMINAL).lt(Int(4)));
    let domain_gone = eq(ERR, 1).or(all([eq(ERR, 3), pending_site.clone(), eq(TERMINAL, 3)]));
    let eligible = all([identity.clone(), domain_gone, positive_terminal]);
    let pending = all([identity, pending_site, eq(ERR, 1).or(eq(ERR, 3))]);
    let route_matches = eligible
        .clone()
        .and(eq(ROUTE, 2))
        .or(all([
            eligible.clone().negate(),
            pending.clone(),
            eq(ROUTE, 3),
        ]))
        .or(all([
            eligible.clone().negate(),
            pending.clone().negate(),
            eq(ROUTE, 0),
        ]));
    let take = if control == 0 {
        eligible.clone()
    } else {
        Bool(false)
    };
    for existing_pending in 0..=1 {
        act(
            &mut m,
            &format!("ApplySharedTerminalTakeoverWithExistingPending{existing_pending}"),
            all([
                eq(P, 3),
                take.clone(),
                if existing_pending == 1 {
                    pending.clone()
                } else {
                    pending.clone().negate()
                },
            ]),
            &[
                (P, Int(4)),
                (ROUTE, Int(2)),
                (PENDING, Int(existing_pending)),
                (REFUSAL, Var(PRIOR)),
            ],
        );
    }
    act(
        &mut m,
        "PreserveOnlyExistingSignalPendingForSelectedSite",
        all([eq(P, 3), take.clone().negate(), pending.clone()]),
        &[
            (P, Int(4)),
            (ROUTE, Int(3)),
            (PENDING, Int(1)),
            (REFUSAL, Var(PRIOR)),
        ],
    );
    act(
        &mut m,
        "RefuseUnprovedOrOtherErrorAtSelectedFixedSite",
        all([eq(P, 3), take.negate(), pending.negate()]),
        &[
            (P, Int(4)),
            (ROUTE, Int(0)),
            (PENDING, Int(0)),
            (REFUSAL, Int(1)),
        ],
    );
    let mut cost_classes = std::collections::BTreeMap::<[i64; 3], Vec<i64>>::new();
    for (i, c) in OBSERVATION_CASES.iter().enumerate() {
        cost_classes
            .entry([c[6], c[8], c[9]])
            .or_default()
            .push(i as i64 + 1);
    }
    for ([reaped, raw, known], cases) in cost_classes {
        let due = if reaped == 0 {
            0
        } else if known == 0 {
            27
        } else {
            9 * raw.max(1)
        };
        let profile = cases
            .iter()
            .fold(Bool(false), |guard, case| guard.or(eq(CASE, *case)));
        act(
            &mut m,
            &format!("ObserveActualOwnWait4Reap{reaped}Raw{raw}Known{known}"),
            all([eq(P, 4), profile]),
            &[
                (P, Int(5)),
                (REAP, Int(reaped)),
                (KNOWN, Int(known)),
                (RAW, Int(raw)),
                (BILL, Int(if known == 1 { raw } else { 0 })),
                (DUE, Int(due)),
            ],
        );
    }
    for gone in 0..=1 {
        act(
            &mut m,
            &format!("ObserveIndependentOriginalGroupGone{gone}"),
            all([eq(P, 5), selected_cases(7, gone)]),
            &[(P, Int(6)), (GONE, Int(gone))],
        );
    }
    for paid in 0..=1 {
        act(
            &mut m,
            &format!("ObserveActualFullFreshDebtPayment{paid}"),
            all([eq(P, 6), selected_cases(10, paid)]),
            &[
                (P, Int(7)),
                (PAID, if paid == 1 { Var(DUE) } else { Int(0) }),
            ],
        );
    }
    let physical_paid = all([
        eq(REAP, 1),
        eq(GONE, 1),
        eq(KNOWN, 1),
        Var(BILL).eq(Var(RAW)),
        Int(0).lt(Var(DUE)),
        Var(PAID).eq(Var(DUE)),
    ]);
    let accepted = all([
        eq(ROUTE, 2),
        eq(PRIOR, 0),
        physical_paid.clone(),
        Var(RAW).lt(Int(3)),
    ]);
    act(
        &mut m,
        "CompleteKnownPaidTerminalProtocolVerdict",
        all([eq(P, 7), accepted.clone()]),
        &[
            (P, Int(8)),
            (OWNER, Int(0)),
            (PENDING, Int(0)),
            (REFUSAL, Int(0)),
            (RESULT, Int(1)),
        ],
    );
    act(
        &mut m,
        "PhysicallySettleKnownPaidCostWhileKeepingRefusal",
        all([eq(P, 7), physical_paid.clone(), accepted.clone().negate()]),
        &[
            (P, Int(8)),
            (OWNER, Int(0)),
            (PENDING, Int(0)),
            (REFUSAL, Int(1)),
            (RESULT, Int(2)),
        ],
    );
    act(
        &mut m,
        "RetainOriginalOwnerForUnknownOrUnpaidSettlement",
        all([eq(P, 7), physical_paid.clone().negate()]),
        &[
            (P, Int(8)),
            (OWNER, Int(1)),
            (REFUSAL, Int(1)),
            (RESULT, Int(2)),
        ],
    );
    safe(
        &mut m,
        "ObservationFailureNeverSuppliesAckContinueWorkOrNativeSuccess",
        all([eq(ACK, 0), eq(CONT, 0), eq(WORK, 0), eq(SUCCESS, 0)]),
    );
    safe(
        &mut m,
        "ObservationTerminalTakeoverNeedsExactOriginalAndIndependentProof",
        eq(ROUTE, 2).negate().or(eligible),
    );
    safe(
        &mut m,
        "ObservationDispositionMatchesEveryFixedSite",
        Var(P).lt(Int(4)).or(route_matches),
    );
    safe(
        &mut m,
        "RawFailureAndTerminalTakeoverNeverInventOwnReapOrCpu",
        Var(P).lt(Int(2)).or(Int(4).lt(Var(P))).or(all([
            eq(REAP, 0),
            eq(GONE, 0),
            eq(KNOWN, 0),
            eq(RAW, 0),
            eq(BILL, 0),
            eq(DUE, 0),
            eq(PAID, 0),
        ])),
    );
    safe(
        &mut m,
        "ObservationAuthoritativeCostAndFullDebtNeverClamp",
        eq(REAP, 0).or(eq(KNOWN, 0)).or(all([
            Var(BILL).eq(Var(RAW)),
            eq(DUE, 9)
                .and(eq(RAW, 0).or(eq(RAW, 1)))
                .or(all([eq(RAW, 2), eq(DUE, 18)]))
                .or(all([eq(RAW, 3), eq(DUE, 27)])),
        ])),
    );
    safe(
        &mut m,
        "ObservationUnknownCostNeverBecomesFiniteCreditOrRelease",
        eq(REAP, 0)
            .or(eq(KNOWN, 1))
            .or(all([eq(RAW, 4), eq(BILL, 0), eq(DUE, 27), eq(OWNER, 1)])),
    );
    safe(
        &mut m,
        "ObservationReleaseNeedsActualOriginalReapGroupAndFullPaidBill",
        eq(OWNER, 1).or(eq(P, 0)).or(eq(P, 1)).or(physical_paid),
    );
    safe(
        &mut m,
        "ObservationPriorFailureNeverClears",
        eq(PRIOR, 0).or(eq(REFUSAL, 1)),
    );
    safe(
        &mut m,
        "ObservationFinalVerdictMatchesTerminalAndBudgetProof",
        eq(P, 8).negate().or(accepted
            .clone()
            .and(eq(RESULT, 1))
            .or(accepted.negate().and(eq(RESULT, 2)))),
    );
    if control != 0 {
        let valid = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid.clone()).eq(Bool(true));
        }
        act(
            &mut m,
            "ObserverStopsAtFirstFailedContract",
            valid.negate().eq(Bool(true)),
            &[],
        );
    }
    m
}

#[cfg(test)]
mod systematic_observe_terminal_tests {
    use super::*;
    #[test]
    fn every_fixed_observation_site_keeps_terminal_identity_full_cost_and_bypass_control() {
        let gm = systematic_observe_terminal(0);
        let good = gm.check(100_000).unwrap();
        assert_eq!(gm.variables.len(), 25);
        assert_eq!(gm.actions.len(), 74);
        assert_eq!(gm.invariants.len(), 9);
        assert!(gm.responses.is_empty() && gm.weak_fair.is_empty());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert_eq!(good.states.len(), 4753);
        assert_eq!(good.edges.iter().map(Vec::len).sum::<usize>(), 4752);
        let bm = systematic_observe_terminal(1);
        let bad = bm.check(100_000).unwrap();
        assert!(!bad.violations.iter().any(|v| v.property == "TypeOK"));
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == "ObservationDispositionMatchesEveryFixedSite")
        );
        for (site_index, name) in DREAM_READ_OBSERVATION_SITES.iter().enumerate() {
            let site = site_index as i64 + 1;
            let signal_pending = matches!(site, 15 | 18);
            for (index, c) in OBSERVATION_CASES.iter().enumerate() {
                let case = index as i64 + 1;
                let identity = c[2] == 1 && c[3] == 1 && c[4] == 1;
                let candidate = c[0] == 1 || (c[0] == 3 && signal_pending && c[1] == 3);
                let proof = (1..=3).contains(&c[1]);
                let route = if identity && candidate && proof {
                    2
                } else if identity && signal_pending && matches!(c[0], 1 | 3) {
                    3
                } else {
                    0
                };
                let physical = c[6] == 1 && c[7] == 1 && c[9] == 1 && c[10] == 1;
                let accepted = route == 2 && physical && c[5] == 0 && c[8] < 3;
                let end = good
                    .states
                    .iter()
                    .find(|s| s[0] == site && s[1] == case && s[2] == 8)
                    .unwrap();
                assert_eq!(
                    end[9], route,
                    "{name}/case{case} keeps distinct terminal-only/pending/refusal outcome"
                );
                assert_eq!(
                    end[24],
                    if accepted { 1 } else { 2 },
                    "{name}/case{case} exact verdict"
                );
                assert_eq!(
                    end[18],
                    i64::from(!physical),
                    "{name}/case{case} original ownership"
                );
                assert_eq!(
                    &end[19..23],
                    &[0, 0, 0, 0],
                    "{name}/case{case} no ACK/CONT/work/native-success invention"
                );
                let cut = good
                    .states
                    .iter()
                    .find(|s| s[0] == site && s[1] == case && s[2] == 4)
                    .unwrap();
                assert_eq!(
                    &cut[11..18],
                    &[0, 0, 0, 0, 0, 0, 0],
                    "{name}/case{case} error/terminal recheck supplies no physical or numeric cost proof"
                );
                if c[6] == 1 {
                    let due = if c[9] == 0 { 27 } else { 9 * c[8].max(1) };
                    assert_eq!(end[13], c[8], "{name}/case{case} raw CPU retained");
                    assert_eq!(
                        end[15],
                        if c[9] == 1 { c[8] } else { 0 },
                        "{name}/case{case} authoritative credit"
                    );
                    assert_eq!(end[16], due, "{name}/case{case} full debt");
                    assert_eq!(
                        end[17],
                        if c[10] == 1 { due } else { 0 },
                        "{name}/case{case} actual payment"
                    );
                }
            }
            for case in 1..=3 {
                assert!(
                    bad.states.iter().any(|s| s[0] == site
                        && s[1] == case
                        && s[2] == 4
                        && s[9] != 2
                        && s[11] == 0
                        && s[12] == 0
                        && s[15] == 0
                        && s[16] == 0
                        && s[17] == 0),
                    "per-site bypass {name}/case{case} falsely refuses positive exact-original terminal proof"
                );
            }
        }
        let output =
            std::env::var_os("DREAM_OBSERVE_OR_TERMINAL_FORMAL_OUT").map(std::path::PathBuf::from);
        let mut manifest = String::new();
        for (model, checked, expected) in [(gm, good, "pass"), (bm, bad, "safety")] {
            println!(
                "{} states={} labelled_edges={} named={:?}",
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
                std::fs::create_dir_all(root).unwrap();
                super::helper_observer_lowering_export::export(
                    model,
                    expected,
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
