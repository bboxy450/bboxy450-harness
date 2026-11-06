// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Finite Q2 producer contracts. Evidence facts abstract existing verifiers;
//! bytes, signatures, bounded process IO and allocator behavior remain witnesses.
use super::*;

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
        // A control fixture asserts at its first invalid boundary. Model that
        // observer explicitly instead of exploring unrelated execution after a
        // proved violation. This changes no good-model transition or bound and
        // retains every valid prefix plus the causal failing transition. Native
        // and reference TLC still compare the complete declared control graph.
        let valid_prefix = all(m.invariants.iter().map(|i| i.condition.clone()));
        for action in &mut m.actions {
            action.guard = action.guard.clone().and(valid_prefix.clone());
        }
        act(
            &mut m,
            "ControlObserverStopsAtFailedBoundary",
            valid_prefix.negate(),
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

/// Decision d93377961365489: exact representative window geometries, including
/// unequal post-call endpoints and a reserved slot whose birth stays unknown.
/// Costs0..3 use an immutable profile's250us/500us tick. Accepted ordinary
/// charges owe9ms; the1.5ms overrun profile conservatively retains18ms. Three
/// accepted observed charges fit additive debt0..27ms. No OS timing is proved.
pub fn member_window(control: u8) -> Model {
    member_window_observer(control, false)
}

fn member_window_observer(control: u8, full_observer: bool) -> Model {
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
    fn member_window_observer_guard_keeps_every_original_state_edge_and_named_verdict() {
        for control in 1..=28 {
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
    fn dream_read_member_windows_preserve_required_closure_and_additive_debt() {
        let good = member_window(0).check(100_000).unwrap();
        println!("HarnessDreamReadMemberWindow: {} states", good.states.len());
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
