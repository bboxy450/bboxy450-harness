// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Finite Q2 producer contracts. Evidence facts abstract existing verifiers;
//! bytes, signatures, bounded process IO and allocator behavior remain witnesses.
use super::*;

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
        _ => "DreamReadHelper",
    };
    const ST:usize=0; const INC:usize=1; const CAP:usize=2; const ATT:usize=3;
    const TICKET:usize=4; const MEMBERS:usize=5; const EXTRA:usize=6;
    const OWNER:usize=7; const DEBT:usize=8; const OWED:usize=9;
    const FG:usize=10; const DEADLINE:usize=11; const COVER:usize=12;
    const TRUST:usize=13; const CONSUMED:usize=14; const EFFECTS:usize=15;
    const PLAN:usize=16; const STOP:usize=17; const REQUEST:usize=18;
    const RETIRED:usize=19; const REAPED:usize=20; const SPAN:usize=21;
    const CASE:usize=22; const LABEL:usize=23; const GRANTS:usize=24;
    const FAULT:usize=25; const RELEASED:usize=26; const RESERVED:usize=27;
    const NATIVE:usize=28; const CHARGED:usize=29; const CONTROLLER:usize=30;
    const DESCENDANT:usize=31; const SPAN_MEMBERS:usize=32; const KNOWN:usize=33;
    const READY:usize=34; const OUTPUT:usize=35; const APP_USED:usize=36;
    const CONFIG:usize=37; const REFUSED:usize=38; const INVALID:usize=39;
    let mut m = base(name, control != 0, &[
        ("state",7,0), ("incarnationA1B2A3",2,0), ("effectIncarnation",2,0),
        ("creationAttempts",2,0), ("creationTicket",2,0), ("members",2,0),
        ("unadmittedMember",1,0), ("workOwnerMainHelperNone",2,0),
        ("debt",2,0), ("observedUnrestedCharge",2,0), ("foreground",1,0),
        ("deadline",1,0), ("coverage",1,0), ("outputTrusted",1,0),
        ("originalEffectConsumed",1,0), ("actualOriginalEffects",2,0),
        ("approvedMemberLimit",2,1), ("allStoppedOrExited",1,1),
        ("stopRequested",1,0), ("controllerRetired",1,0), ("allActuallyReaped",1,0),
        ("spanNoneNativeHelperApplication",3,0), ("inputCase",9,0),
        ("labelNoneApplicationNative",2,0), ("hasGranted",1,0),
        ("faultNoneClockPanicRevokeEffectResourceMember",6,0), ("ownershipReleased",1,0),
        ("lastCreationReserved",1,0), ("nativeCreationCertified",1,0),
        ("spanCharged",1,0), ("controllerCharged",1,0), ("descendantCharged",1,0),
        ("membersAtSpan",2,0), ("accountingKnown",1,1), ("actualReapReady",1,0),
        ("outputObserved",1,0), ("applicationSpanUsed",1,0), ("configured",1,0),
        ("staleReplyRefused",1,0), ("invalidated",1,0),
    ]);
    let exact = Var(INC).eq(Var(CAP));
    let admitted = if control == 4 {
        exact.clone().or(all([eq(INC,2),eq(CAP,0)]))
    } else { exact.clone() };
    let native_ok = all([eq(CASE,3).negate(),eq(CASE,4).negate(),eq(CASE,5).negate()]);
    let native_guard = native_ok.clone().or(match control {
        12 => eq(CASE,3), 13 => eq(CASE,4), 14 => eq(CASE,5), _ => Bool(false),
    });
    let quiet = eq(MEMBERS,0).or(eq(STOP,1));
    let idle = all([eq(DEBT,0),eq(OWED,0),eq(SPAN,0)]);
    // One input class at a time keeps the complete graph finite and tractable.
    // Classes cover wrong executable/suspend capability, native CPU/wall/raw,
    // ordinary application overrun, runnable-window overrun and attempt edges.
    for case in 0..=9 {
        act(&mut m,&format!("ConfigureCase{case}"),all([eq(CONFIG,0),eq(ST,0)]),&[
            (CASE,Int(case)),(CONFIG,Int(1)),
            (ATT,Int(if case==8 {2} else if case==9 {1} else {0})),
        ]);
    }
    for need in 1..=2 {
        for before in 0..=2 {
            let valid = before + need <= 2;
            act(&mut m,&format!("ReserveAttempt{before}Need{need}"),all([
                eq(CONFIG,1),eq(ST,0),eq(TICKET,0),eq(CONSUMED,0),eq(FAULT,0),
                eq(ATT,before),eq(FG,0),eq(DEADLINE,0),idle.clone(),
                if valid || control==1 {Bool(true)} else {Bool(false)},
            ]),&[(ATT,Int((before+need).min(2))),(TICKET,Int(need)),(PLAN,Int(need)),
                (CAP,Var(INC)),(RESERVED,Int(i64::from(valid)))]);
        }
    }
    for cost in 1..=2 {
        for initially_stopped in 0..=1 {
            act(&mut m,&format!("CreateSuspendedCost{cost}Stopped{initially_stopped}"),all([
                eq(ST,0),eq(CONFIG,1),eq(TICKET,0).negate(),eq(CONSUMED,0),eq(MEMBERS,0),
                eq(DEADLINE,0),eq(FAULT,0),eq(FG,0),admitted.clone(),idle.clone(),
                if control==3 {Bool(true)} else {eq(CASE,1).negate()},
                if control==5 {Bool(true)} else {eq(CASE,2).negate()},
                if initially_stopped==0 {eq(CASE,2)} else {eq(CASE,2).negate()},
                if cost==1 {native_ok.clone()} else {native_ok.clone().negate()},
                Var(EFFECTS).lt(Int(2)),
            ]),&[(ST,Int(1)),(MEMBERS,Int(1)),(OWNER,Int(0)),(STOP,Int(initially_stopped)),
                (CONSUMED,Int(1)),(EFFECTS,Var(EFFECTS).plus(Int(1))), (TICKET,Int(0)),
                (SPAN,Int(1)),(LABEL,Int(2)),(OWED,Int(cost)),(CHARGED,Int(0)),
                (COVER,Int(0)),(NATIVE,Int(0))]);
        }
    }
    act(&mut m,"AttachExactSuspendedIdentityBeforeGrant",all([
        eq(ST,1),eq(STOP,1),admitted.clone(),eq(FAULT,0),eq(DEADLINE,0),
    ]),&[(ST,Int(2))]);
    for cost in 1..=2 {
        act(&mut m,&format!("ChargeNativeInclusiveScope{cost}"),all([
            eq(ST,2),eq(SPAN,1),eq(CHARGED,0),eq(OWED,cost),eq(KNOWN,1),
            native_guard.clone(),eq(STOP,1),
        ]),&[(DEBT,Int(cost)),(CHARGED,Int(1)),(OWNER,Int(2)),
            (COVER,Int(1)),(NATIVE,Int(1)),(CONTROLLER,Int(1)),(DESCENDANT,Int(1))]);
    }
    act(&mut m,"GrantOneBoundedGroupQuantum",all([
        eq(ST,2).or(eq(ST,4)),eq(STOP,1),eq(OWNER,2),idle.clone(),
        eq(FG,0),eq(DEADLINE,0),eq(FAULT,0),eq(KNOWN,1),eq(NATIVE,1),
        admitted.clone(),
    ]),&[(ST,Int(3)),(STOP,Int(0)),(REQUEST,Int(0)),(OWNER,Int(1)),
        (GRANTS,Int(1)),(SPAN,Int(2)),(LABEL,Int(1)),(KNOWN,Int(0)),
        (CHARGED,Int(0)),(CONTROLLER,Int(0)),(DESCENDANT,Int(0))]);
    act(&mut m,"ApprovedImplicitGpgAppears",all([eq(ST,3),eq(STOP,0),eq(PLAN,2),eq(MEMBERS,1)]),&[(MEMBERS,Int(2))]);
    for count in 1..=2 {
        let mut updates = vec![(MEMBERS,Int(2)),(EXTRA,Int(1))];
        if control != 2 { updates.extend([(ST,Int(7)),(FAULT,Int(6)),(KNOWN,Int(0)),(COVER,Int(0))]); }
        act(&mut m,&format!("ObserveMemberBeyondAdmittedLimit{count}"),all([
            eq(ST,3),eq(MEMBERS,count),eq(PLAN,count),eq(EXTRA,0),
        ]),&updates);
    }
    act(&mut m,"RequestGroupStop",all([eq(ST,3),eq(REQUEST,0)]),&[(REQUEST,Int(1))]);
    act(&mut m,"ObserveAllMembersStoppedOrExited",all([eq(ST,3),eq(REQUEST,1),eq(STOP,0)]),&[(STOP,Int(1))]);
    for count in 1..=2 {
        act(&mut m,&format!("ConfirmStoppedMembers{count}"),all([
            eq(ST,3),eq(REQUEST,1),eq(MEMBERS,count),
            if control==6 {Bool(true)} else {eq(STOP,1)},
        ]),&[(ST,Int(4)),(OWNER,Int(2)),(OWED,Int(count)),(SPAN_MEMBERS,Int(count)),(KNOWN,Int(1)),
            (SPAN,Int(2)),(LABEL,Int(1)),(CHARGED,Int(0))]);
    }
    for count in 1..=2 {
        let omit_descendant = control==7 && count==2;
        let omit_controller = control==8;
        act(&mut m,&format!("ChargeFullRunnableWallMembersAndController{count}"),all([
            eq(ST,4),eq(STOP,1),eq(SPAN,2),eq(CHARGED,0),eq(KNOWN,1),
            eq(OWED,count),if control==15 {Bool(true)} else {eq(CASE,7).negate()},
        ]),&[(DEBT,Int(count-i64::from(omit_descendant||omit_controller))),
            (CHARGED,Int(1)),(COVER,Int(1)),(CONTROLLER,Int(i64::from(!omit_controller))),
            (DESCENDANT,Int(i64::from(!omit_descendant)))]);
    }
    for cost in 1..=2 {
        act(&mut m,&format!("OrdinaryApplicationSpan{cost}"),all([
            eq(CONFIG,1),eq(APP_USED,0),eq(FAULT,0),eq(DEADLINE,0),eq(FG,0),
            if control==9 {idle.clone().or(all([eq(ST,3),eq(DEBT,0),eq(OWED,0)]))} else {idle.clone()},
            if control==9 {eq(ST,2).or(eq(ST,3)).or(eq(ST,4))}
                else {all([eq(ST,2).or(eq(ST,4)),quiet.clone(),eq(OWNER,1).negate()])},
            if cost==2 {eq(CASE,6)} else {eq(CASE,6).negate()},
        ]),&[(OWNER,Int(0)),(OWED,Int(cost)),(SPAN,Int(3)),(LABEL,Int(1)),
            (CHARGED,Int(0)),(APP_USED,Int(1))]);
        act(&mut m,&format!("ChargeOrdinaryApplication{cost}"),all([
            eq(SPAN,3),eq(CHARGED,0),eq(OWED,cost),eq(KNOWN,1),
            if control==11 {Bool(true)} else {eq(CASE,6).negate()},
        ]),&[(DEBT,Int(cost)),(CHARGED,Int(1)),(OWNER,Int(2)),
            (LABEL,Int(if control==11 && cost==2 {2} else {1})),
            (CONTROLLER,Int(1)),(DESCENDANT,Int(1))]);
    }
    let mut settle = vec![];
    for debt in 1..=2 {
        settle.push(m.actions.len());
        act(&mut m,&format!("RestAfterConfirmedWork{debt}"),all([
            eq(DEBT,debt),eq(OWED,debt),eq(CHARGED,1),eq(OWNER,2),eq(KNOWN,1),quiet.clone(),
        ]),&[(DEBT,Int(debt-1)),(OWED,Int(debt-1))]);
    }
    act(&mut m,"ReconcilePaidSpanWithoutResettingAnotherDebt",all([
        eq(DEBT,0),eq(OWED,0),eq(CHARGED,1),eq(OWNER,2),quiet.clone(),
    ]),&[(SPAN,Int(0)),(LABEL,Int(0)),(CHARGED,Int(0)),(CONTROLLER,Int(0)),
        (DESCENDANT,Int(0)),(SPAN_MEMBERS,Int(0)),(REQUEST,Int(0)),(APP_USED,Int(0))]);
    for foreground in 0..=1 {
        act(&mut m,&format!("ObserveForeground{foreground}WithoutDebtReset"),all([
            eq(RELEASED,0),eq(FG,foreground).negate(),
            eq(ST,2).or(eq(ST,3)).or(eq(ST,4)).or(eq(ST,5).and(eq(DEBT,0).negate())),
        ]),&[(FG,Int(foreground))]);
    }
    act(&mut m,"IoCompletionKeepsDebt",all([eq(RELEASED,0),eq(DEBT,0).negate(),eq(ST,2).or(eq(ST,4))]),&[]);
    if control==10 {
        act(&mut m,"BrokenForegroundOrIoDebtReset",all([eq(CHARGED,1),eq(DEBT,0).negate()]),&[(DEBT,Int(0)),(FG,Int(1))]);
    }
    act(&mut m,"ObserveOriginalOutputBeforeSettlement",all([
        eq(ST,3).or(eq(ST,4)),Var(GRANTS).lt(Int(1)).negate(),eq(FAULT,0),
    ]),&[(OUTPUT,Int(1))]);
    act(&mut m,"ObserveDeadline",all([eq(RELEASED,0),eq(DEADLINE,0)]),&[(DEADLINE,Int(1))]);
    for fault in 1..=3 {
        let mut updates = vec![(ST,Int(7)),(FAULT,Int(fault)),(COVER,Int(0)),(TRUST,Int(0)),
            (KNOWN,Int(if fault==3 {1} else {0}))];
        if fault!=3 {
            // Unknown accounting is terminal for content. Quotient only dead
            // span annotations; retain every outstanding debt/ownership fact.
            updates.extend([(SPAN,Int(0)),(LABEL,Int(0)),(CHARGED,Int(0)),
                (CONTROLLER,Int(0)),(DESCENDANT,Int(0)),(SPAN_MEMBERS,Int(0)),
                (NATIVE,Int(0)),(OUTPUT,Int(0)),(GRANTS,Int(0)),(APP_USED,Int(0))]);
        }
        act(&mut m,&format!("ObserveClockPanicRevoke{fault}"),all([
            Var(ST).lt(Int(6)),eq(RELEASED,0),eq(FAULT,0),eq(CONFIG,1),eq(EFFECTS,1),
        ]),&updates);
    }
    act(&mut m,"NativeEffectThenReportedError",all([
        eq(ST,2),eq(DEBT,0),eq(OWED,0),eq(FAULT,0),eq(CONSUMED,1),eq(GRANTS,0),eq(APP_USED,0),
    ]),&[(ST,Int(7)),(FAULT,Int(4)),(COVER,Int(0)),(TRUST,Int(0)),
        (CONSUMED,Int(if control==16 {0} else {1}))]);
    let overrun = eq(SPAN,1).and(native_ok.clone().negate())
        .or(eq(SPAN,2).and(eq(CASE,7))).or(eq(SPAN,3).and(eq(CASE,6)));
    settle.push(m.actions.len());
    act(&mut m,"RefuseMeasuredOverrunWithoutCertifyingOutput",all([
        overrun,eq(CHARGED,0),eq(FAULT,0),eq(RELEASED,0),
    ]),&[(ST,Int(7)),(FAULT,Int(5)),(COVER,Int(0)),(TRUST,Int(0))]);
    for cost in 1..=2 {
        act(&mut m,&format!("ChargeKnownRejectedSpan{cost}"),all([
            eq(ST,7),eq(FAULT,5),eq(KNOWN,1),eq(CHARGED,0),eq(OWED,cost),quiet.clone(),
        ]),&[(DEBT,Int(cost)),(CHARGED,Int(1)),(OWNER,Int(2)),
            (COVER,Int(0)),(CONTROLLER,Int(1)),(DESCENDANT,Int(1))]);
    }
    settle.push(m.actions.len());
    act(&mut m,"DeadlineRetainsOwnedQuarantine",all([
        eq(DEADLINE,1),Var(ST).lt(Int(5)),eq(RELEASED,0),
    ]),&[(ST,Int(7)),(COVER,Int(0)),(TRUST,Int(0))]);
    // Invalidate/restart preserve the original effect and the old pinned owner.
    // A3 repeats the label A, never its incarnation or original authority.
    act(&mut m,"InvalidateWithoutReleasingOldOwner",all([
        eq(INVALID,0),Var(INC).lt(Int(2)),eq(RELEASED,0),eq(CONFIG,1),
        eq(FAULT,0).or(all([eq(FAULT,3),eq(ST,7),eq(INC,1),eq(KNOWN,1)])),
    ]),&[(INVALID,Int(1)),(ST,Int(7)),(FAULT,Int(3)),(COVER,Int(0)),(TRUST,Int(0))]);
    act(&mut m,"RestartA1B2A3",all([eq(INVALID,1),Var(INC).lt(Int(2))]),&[(INC,Var(INC).plus(Int(1))),(INVALID,Int(0))]);
    act(&mut m,"RefuseOldIncarnationReply",all([exact.clone().negate(),eq(ST,7)]),&[(REFUSED,Int(1))]);
    if control==4 {
        act(&mut m,"ReplayOldAReplyIntoA3",all([
            eq(ST,7),eq(INC,2),eq(CAP,0),eq(MEMBERS,0).negate(),eq(STOP,1),
            eq(NATIVE,1),eq(DEBT,0),eq(OWED,0),
        ]),&[(ST,Int(2)),(FAULT,Int(0)),(COVER,Int(1)),(OWNER,Int(2))]);
    }
    settle.push(m.actions.len());
    act(&mut m,"RetireControllerBeforePinnedLeaderFinalSignal",all([
        eq(ST,1).or(eq(ST,2)).or(eq(ST,4)).or(eq(ST,7)),eq(RETIRED,0),
    ]),&[(ST,Int(5)),(RETIRED,Int(1))]);
    // OS completion is observed, not a fairness assumption. A never-returning
    // observation has an explicit owned quarantine disposition below.
    act(&mut m,"ObserveActualAllMemberReapReady",all([eq(ST,5),eq(RETIRED,1),eq(READY,0)]),&[(READY,Int(1))]);
    settle.push(m.actions.len());
    act(&mut m,"ActuallyReapAllMembersAfterControllerRetirement",all([
        eq(ST,5),eq(RETIRED,1),eq(READY,1),
    ]),&[(ST,Int(6)),(MEMBERS,Int(0)),(REAPED,Int(1)),(STOP,Int(1)),(OWNER,Int(2))]);
    settle.push(m.actions.len());
    act(&mut m,"UnavailableSettlementKeepsOwnedQuarantine",all([
        eq(ST,5),eq(READY,0),eq(DEADLINE,1),
    ]),&[(ST,Int(7)),(COVER,Int(0)),(TRUST,Int(0))]);
    act(&mut m,"RetryOnlyUnconsumedOriginalEffect",all([
        eq(ST,6),eq(FAULT,4),eq(CONSUMED,0),eq(DEBT,0),eq(OWED,0),eq(RELEASED,0),
    ]),&[(ST,Int(0)),(FAULT,Int(0)),(RETIRED,Int(0)),(REAPED,Int(0)),
        (READY,Int(0)),(NATIVE,Int(0)),(COVER,Int(0)),(SPAN,Int(0)),(LABEL,Int(0))]);
    act(&mut m,"CoverageReceiptUnavailable",all([eq(ST,6),eq(COVER,1),eq(RELEASED,0)]),&[(COVER,Int(0)),(TRUST,Int(0))]);
    let fully_reaped = all([eq(ST,6),eq(MEMBERS,0),eq(RETIRED,1),eq(REAPED,1)]);
    act(&mut m,"TrustOnlyCoveredSettledOriginalOutput",all([
        if control==18 {Bool(true)} else {fully_reaped.clone()},
        if control==17 {Bool(true)} else {eq(COVER,1)},
        if control==2 {Bool(true)} else {eq(EXTRA,0)},
        eq(NATIVE,1),eq(KNOWN,1),eq(FAULT,0),eq(DEBT,0),eq(OWED,0),eq(OWNER,2),
        eq(OUTPUT,1),eq(FG,0),eq(DEADLINE,0),admitted.clone(),eq(CONSUMED,1),eq(TRUST,0),
    ]),&[(TRUST,Int(1))]);
    settle.push(m.actions.len());
    act(&mut m,"ReleaseOnlyActuallySettledKnownOwnership",all([
        fully_reaped.clone(),eq(KNOWN,1),eq(DEBT,0),eq(OWED,0),eq(OWNER,2),eq(RELEASED,0),
    ]),&[(RELEASED,Int(1))]);
    for (mode,fault) in [(19,1),(20,2),(21,3)] {
        if control==mode {
            act(&mut m,&format!("BrokenReleaseOnFault{fault}"),all([
                eq(ST,7),eq(FAULT,fault),eq(RELEASED,0),
            ]),&[(RELEASED,Int(1)),(OWNER,Int(2))]);
        }
    }
    safe(&mut m,"NativeEffectRequiresReservedAttempt",eq(EFFECTS,0).or(eq(RESERVED,1)));
    safe(&mut m,"NativeEffectRequiresExactExecutableCapability",eq(EFFECTS,0).or(eq(CASE,1).negate()));
    safe(&mut m,"NoUserCodeBeforeInitialSuspension",eq(EFFECTS,0).or(eq(CASE,2).negate()));
    safe(&mut m,"AttachedWorkKeepsExactIncarnation",all([eq(ST,1).negate(),eq(ST,2).negate(),eq(ST,3).negate(),eq(ST,4).negate()]).or(exact.clone()));
    safe(&mut m,"StoppedStateRequiresAllMemberConfirmation",eq(ST,4).negate().or(eq(STOP,1)));
    safe(&mut m,"MainProofCannotOverlapRunnableGroup",eq(MEMBERS,0).or(eq(STOP,1)).or(eq(OWNER,0).negate()));
    safe(&mut m,"DebtCoversWholeObservedUnrestedSpan",eq(CHARGED,0).or(Var(DEBT).lt(Var(OWED)).negate()));
    safe(&mut m,"HelperChargeIncludesDescendantsAndController",eq(CHARGED,0).or(eq(SPAN,2).negate()).or(all([
        eq(CONTROLLER,1),Var(SPAN_MEMBERS).lt(Int(2)).or(eq(DESCENDANT,1)),
    ])));
    safe(&mut m,"OrdinaryApplicationCannotUseNativeLabel",eq(LABEL,2).negate().or(eq(SPAN,1)));
    safe(&mut m,"CertifiedNativeCreationMeetsCpuWallInclusiveBounds",eq(NATIVE,0).or(native_ok));
    safe(&mut m,"CoveredRunnableChargeMeetsWholeGroupWindow",eq(SPAN,2).negate().or(eq(CHARGED,0)).or(eq(COVER,0)).or(eq(CASE,7).negate()));
    safe(&mut m,"OriginalNativeEffectNeverReplayed",Var(EFFECTS).lt(Int(2)));
    safe(&mut m,"TrustedOutputHasCompleteCoverage",eq(TRUST,0).or(eq(COVER,1)));
    safe(&mut m,"TrustedOutputHasActualReapAndRetirement",eq(TRUST,0).or(fully_reaped.clone()));
    safe(&mut m,"TrustedOutputHasOnlyAdmittedMembers",eq(TRUST,0).or(eq(EXTRA,0)));
    safe(&mut m,"TrustedOutputKeepsOriginalIncarnation",eq(TRUST,0).or(exact));
    safe(&mut m,"FaultCannotReleaseUnsettledOwnership",eq(RELEASED,0).or(all([
        fully_reaped,eq(MEMBERS,0),eq(DEBT,0),eq(OWED,0),eq(KNOWN,1),
    ])));
    progress(&mut m,"ControlHasOwnedSettlementOrQuarantineDisposition",eq(DEADLINE,1),
        Var(ST).lt(Int(5)).negate(),settle);
    if control != 0 {
        // A control fixture asserts at its first invalid boundary. Model that
        // observer explicitly instead of exploring unrelated execution after a
        // proved violation. This changes no good-model transition or bound and
        // retains every valid prefix plus the causal failing transition. Native
        // and reference TLC still compare the complete declared control graph.
        let valid_prefix = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid_prefix.clone());
        }
        act(&mut m,"ControlObserverStopsAtFailedBoundary",valid_prefix.negate(),&[]);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dream_read_helper_lifecycle_and_causal_controls() {
        let good = helper(0).check(100_000).unwrap();
        println!("HarnessDreamReadHelper: {} states", good.states.len());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
        assert!(good.states.iter().any(|s| s[13]==1), "covered actually reaped output is reachable");
        assert!(good.states.iter().any(|s| s[0]==7 && s[26]==0 && s[5]>0), "unknown work retains real helper ownership");
        assert!(good.states.iter().any(|s| s[3]==2 && s[5]==1 && s[15]==1), "unused signature creation credit stays consumed");
        assert!(good.states.iter().any(|s| s[1]==2 && s[2]==0 && s[38]==1 && s[14]==1), "old A1 effect survives and refuses A3 replay");
        assert!(good.states.iter().any(|s| s[8]==2 && s[9]==2 && s[32]==2 && s[30]==1 && s[31]==1), "two-member whole charge is reachable");
        for fault in 1..=3 {
            assert!(good.states.iter().any(|s| s[25]==fault && s[15]==1 && s[26]==0 && s[5]>0));
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
        ];
        for (index, invariant) in expected.into_iter().enumerate() {
            let model = helper((index+1) as u8);
            let checked = model.check(100_000).unwrap_or_else(|e| panic!("{}: {e}",model.name));
            println!("{}: {} states",model.name,checked.states.len());
            assert!(checked.violations.iter().any(|v| v.property==invariant), "{} must causally violate {invariant}", model.name);
            if index>=18 {
                assert!(checked.states.iter().any(|s| s[15]==1 && s[26]==1 && s[20]==0 && s[5]>0), "fault control must release an actual unreaped helper");
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
