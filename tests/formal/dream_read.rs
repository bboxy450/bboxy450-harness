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

/// Decision d93377961365489: exact representative window geometries, including
/// unequal post-call endpoints and a reserved slot whose birth stays unknown.
/// Costs0..3 use an immutable profile's250us/500us tick. Accepted ordinary
/// charges owe9ms; the1.5ms overrun profile conservatively retains18ms. Three
/// accepted observed charges fit additive debt0..27ms. No OS timing is proved.
pub fn member_window(control: u8) -> Model {
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
            ("cumulativeDebtMs", 27, 0),
            ("grantCount", 2, 0),
            ("provisionedWholeCommandSupport", 1, 1),
            ("ownedUnavailableQuarantine", 1, 0),
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
        all([eq(PH, 3), available.clone(), eq(DEBT, 0)]),
        &first,
    );
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
                    Int(if c == 0 {
                        0
                    } else if control == 9 {
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
        "GenuinelyPostSealCorroborationIsSeparateOrdinaryUnit",
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
            (DEBT, Int(9)),
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
    if control != 0 {
        // As with the existing helper controls, stop at the first causal failed
        // assertion. No post-invalid-record arithmetic is certified.
        let valid = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid.clone());
        }
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            valid.negate(),
            &[],
        );
    }
    m
}

/// Decision d93377961365654: a reviewed finite program/build premise is distinct
/// from runtime observations. The approved graph assumes that premise; the
/// adversarial graph permits unaccounted work after its explicit withdrawal.
/// Assumption-drop controls deliberately forge the approval/premise relation,
/// retain identical censuses, and must reach actual unaccounted-work trust.
pub fn qualification(control: u8) -> Model {
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
    if control >= 2 {
        let valid = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid.clone());
        }
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            valid.negate(),
            &[],
        );
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
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
        }
    }

    #[test]
    fn dream_read_member_windows_preserve_required_closure_and_additive_debt() {
        let good = member_window(0).check(100_000).unwrap();
        println!("HarnessDreamReadMemberWindow: {} states", good.states.len());
        assert!(good.violations.is_empty(), "{:?}", good.violations);
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
            good.states
                .iter()
                .any(|s| s[0] == 2 && s[1] == 2 && s[2] == 1 && s[6] == 1 && s[21] == 1),
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
