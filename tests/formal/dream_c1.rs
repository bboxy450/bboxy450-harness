// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Finite producer contracts. Bytes, CPU/heap, pipe IO and syscalls are process witnesses.
use super::*;
pub fn inspection(control: u8) -> Model {
    let name = match control {
        1 => "DreamC1InspectionClose",
        2 => "DreamC1InspectionRaw",
        3 => "DreamC1InspectionClobber",
        4 => "DreamC1InspectionLegacyHook",
        _ => "DreamC1Inspection",
    };
    // Two selected source proofs; a raw/canonical digest distinction; one immutable output.
    let mut m = base(
        name,
        control != 0,
        &[
            ("source1", 2, 0),
            ("source2", 2, 0),
            ("rawDigest", 1, 0),
            ("canonicalDigest", 1, 1),
            ("validated", 1, 0),
            ("output", 2, 0),
            ("result", 1, 0),
            ("finished", 1, 0),
            ("termOwner", 1, 1),
            ("legacyEffects", 1, 0),
        ],
    );
    act(
        &mut m,
        "LegacyCloseHook",
        if control == 4 { Bool(true) } else { eq(8, 0) },
        &[(9, Int(1))],
    );
    safe(
        &mut m,
        "ManagedCloseHasNoLegacyEffects",
        eq(8, 0).or(eq(9, 0)),
    );
    for i in 0..2 {
        act(
            &mut m,
            &format!("ObserveClosed{i}"),
            all([eq(i, 0), eq(4, 0), eq(7, 0)]),
            &[(i, Int(1))],
        );
        act(
            &mut m,
            &format!("ObserveMalformed{i}"),
            all([eq(i, 0), eq(4, 0), eq(7, 0)]),
            &[(i, Int(2))],
        );
    }
    act(
        &mut m,
        "RawBytesMatch",
        all([eq(2, 0), eq(4, 0), eq(7, 0)]),
        &[(2, Int(1))],
    );
    let close = if control == 1 {
        Bool(true)
    } else {
        all([eq(0, 1), eq(1, 1)])
    };
    let bytes = if control == 2 { eq(3, 1) } else { eq(2, 1) };
    act(
        &mut m,
        "ValidateExactSnapshot",
        all([close, bytes, eq(4, 0), eq(7, 0)]),
        &[(4, Int(1))],
    );
    act(
        &mut m,
        "PublishPageNoReplace",
        all([eq(4, 1), eq(5, 0), eq(7, 0)]),
        &[(5, Int(1))],
    );
    if control == 3 {
        act(&mut m, "ReplacePublishedPage", eq(5, 1), &[(5, Int(2))]);
    }
    act(
        &mut m,
        "PublishResultLast",
        all([eq(4, 1), eq(5, 1), eq(6, 0), eq(7, 0)]),
        &[(6, Int(1)), (7, Int(1))],
    );
    let fail = m.actions.len();
    act(&mut m, "FailOrDeadline", eq(7, 0), &[(7, Int(1))]);
    safe(
        &mut m,
        "ValidatedRequiresBothClosedAndRaw",
        eq(4, 0).or(all([eq(0, 1), eq(1, 1), eq(2, 1)])),
    );
    safe(&mut m, "ImmutablePage", Var(5).le(Int(1)));
    safe(
        &mut m,
        "ResultLast",
        eq(6, 0).or(all([eq(4, 1), eq(5, 1), eq(7, 1)])),
    );
    progress(
        &mut m,
        "ValidationTerminates",
        Bool(true),
        eq(7, 1),
        vec![fail],
    );
    m
}
pub fn worker(control: u8) -> Model {
    let name = match control {
        1 => "DreamC1WorkerForeground",
        2 => "DreamC1WorkerEarlyDone",
        3 => "DreamC1WorkerCancelledPublish",
        _ => "DreamC1Worker",
    };
    // Two units, three activity revisions (A1/B2/A3), one compute owner and one Done.
    let mut m = base(
        name,
        control != 0,
        &[
            ("started", 1, 0),
            ("foreground", 1, 1),
            ("revision", 2, 0),
            ("units", 2, 0),
            ("cancel", 1, 0),
            ("live", 1, 0),
            ("settled", 1, 0),
            ("done", 1, 0),
            ("foregroundWork", 1, 0),
            ("published", 1, 0),
            ("cancelledPublish", 1, 0),
        ],
    );
    act(
        &mut m,
        "MatchingStart",
        eq(0, 0),
        &[(0, Int(1)), (5, Int(1))],
    );
    for rev in 0..2 {
        for fg in 0..2 {
            act(
                &mut m,
                &format!("Activity{rev}_{fg}"),
                all([eq(0, 1), eq(2, rev), eq(7, 0)]),
                &[(1, Int(fg)), (2, Int(rev + 1))],
            );
        }
    }
    let fg = if control == 1 { Bool(true) } else { eq(1, 0) };
    act(
        &mut m,
        "BoundedUnit",
        all([eq(0, 1), fg, eq(4, 0), eq(5, 1), Var(3).lt(Int(2))]),
        &[(3, Var(3).plus(Int(1))), (8, Var(1))],
    );
    let cancel = m.actions.len();
    act(
        &mut m,
        "CancelDeadlineOrEof",
        all([eq(0, 1), eq(4, 0), eq(7, 0)]),
        &[(4, Int(1))],
    );
    let publish_guard = if control == 3 { Bool(true) } else { eq(4, 0) };
    act(
        &mut m,
        "ImmutableResult",
        all([eq(3, 2), eq(5, 1), publish_guard, eq(9, 0)]),
        &[(9, Int(1)), (10, Var(4))],
    );
    let settle = m.actions.len();
    act(
        &mut m,
        "ObserveComputeExit",
        all([eq(5, 1), eq(4, 1).or(eq(9, 1))]),
        &[(5, Int(0)), (6, Int(1))],
    );
    let ready = if control == 2 { eq(0, 1) } else { eq(6, 1) };
    let done = m.actions.len();
    act(&mut m, "SingleDone", all([ready, eq(7, 0)]), &[(7, Int(1))]);
    safe(&mut m, "NoForegroundUnit", eq(8, 0));
    safe(
        &mut m,
        "DoneRequiresRealSettlement",
        eq(7, 0).or(all([eq(6, 1), eq(5, 0)])),
    );
    // A result cannot be first published by an already-cancelled unit. Cancellation
    // after a prior immutable publication is allowed; the owner still reports cancelled.
    // Track the edge causally rather than forbidding that valid later cancellation.
    safe(&mut m, "CancelledCannotPublish", eq(10, 0));
    progress(
        &mut m,
        "StartedEventuallySettles",
        eq(0, 1),
        eq(7, 1),
        vec![cancel, settle, done],
    );
    m
}

/// Declared requested storage only. Native allocator/serde/regex overhead is
/// deliberately outside this finite credit abstraction and is measured in processes.
pub fn storage(bypass: bool) -> Model {
    let mut m = base(
        "DreamC1Storage",
        bypass,
        &[
            ("declared", 4, 0),
            ("reserved", 4, 0),
            ("phase", 4, 0),
            ("cancel", 1, 0),
            ("settled", 1, 0),
            ("control", 2, 2),
            ("status", 1, 0),
            ("owner", 1, 0),
        ],
    );
    act(
        &mut m,
        "ReserveBeforeCreate",
        all([eq(2, 0), eq(3, 0)]),
        &[(0, Int(2)), (1, Int(2)), (2, Int(1))],
    );
    act(
        &mut m,
        "ReserveOldAndNewOverlap",
        all([eq(2, 1), eq(3, 0)]),
        &[(1, if bypass { Int(2) } else { Int(4) }), (2, Int(2))],
    );
    act(
        &mut m,
        "InstallGrowth",
        all([eq(2, 2), eq(3, 0)]),
        &[(0, Int(4)), (2, Int(3))],
    );
    act(
        &mut m,
        "FreeOldThenReleaseCredit",
        all([eq(2, 3), eq(3, 0)]),
        &[(0, Int(2)), (1, Int(2)), (2, Int(4))],
    );
    act(
        &mut m,
        "TransferOneOwner",
        all([eq(0, 2), eq(7, 0)]),
        &[(7, Int(1))],
    );
    act(
        &mut m,
        "IndependentStatusAtExhaustion",
        eq(1, 4),
        &[(6, Int(1))],
    );
    let cancel = m.actions.len();
    act(&mut m, "Cancel", eq(3, 0), &[(3, Int(1))]);
    let settle = m.actions.len();
    act(
        &mut m,
        "PacedDropThenRelease",
        all([eq(3, 1), eq(4, 0)]),
        &[(0, Int(0)), (1, Int(0)), (4, Int(1))],
    );
    safe(&mut m, "EveryDeclaredCapacityHasCredit", Var(0).le(Var(1)));
    safe(
        &mut m,
        "WorkCannotBorrowControl",
        all([Var(1).le(Int(4)), eq(5, 2)]),
    );
    safe(
        &mut m,
        "SettlementReleasesOwnedCredits",
        eq(4, 0).or(all([eq(0, 0), eq(1, 0)])),
    );
    progress(
        &mut m,
        "CancelledStorageSettles",
        Bool(true),
        eq(4, 1),
        vec![cancel, settle],
    );
    m
}

/// Private fixture only: actual Host/IR/UDS/EOF/join are process witnesses.
pub fn retained_host(early: bool) -> Model {
    let mut m = base(
        "DreamC1RetainedHost",
        early,
        &[
            ("phase", 4, 0),
            ("created", 2, 0),
            ("closedA", 1, 0),
            ("closedB", 1, 0),
            ("consumed", 1, 0),
            ("joined", 1, 0),
        ],
    );
    act(
        &mut m,
        "CreateActualA",
        eq(0, 0),
        &[(0, Int(1)), (1, Int(1))],
    );
    let close_a = m.actions.len();
    act(
        &mut m,
        "FlushDurableCloseA",
        eq(0, 1),
        &[(0, Int(2)), (2, Int(1))],
    );
    act(
        &mut m,
        "CreateFreshBOnce",
        all([
            if early {
                eq(0, 1).or(eq(0, 2))
            } else {
                eq(0, 2)
            },
            eq(4, 0),
        ]),
        &[(0, Int(3)), (1, Int(2)), (4, Int(1))],
    );
    let close_b = m.actions.len();
    act(
        &mut m,
        "CloseActualB",
        eq(0, 3),
        &[(0, Int(4)), (3, Int(1))],
    );
    let eof = m.actions.len();
    act(
        &mut m,
        "EofDisablesTransition",
        eq(0, 2),
        &[(0, Int(4)), (4, Int(1))],
    );
    let join = m.actions.len();
    act(&mut m, "ActualHostJoin", eq(0, 4), &[(5, Int(1))]);
    safe(
        &mut m,
        "BBeginsAfterDurableFlushedA",
        Var(1).lt(Int(2)).or(eq(2, 1)),
    );
    safe(
        &mut m,
        "ExactlyOneFreshTransition",
        Var(1).lt(Int(2)).or(eq(4, 1)),
    );
    safe(
        &mut m,
        "JoinAfterAllCreatedSessionsClose",
        eq(5, 0).or(all([eq(2, 1), Var(1).lt(Int(2)).or(eq(3, 1))])),
    );
    progress(
        &mut m,
        "RetainedOwnerEventuallyJoins",
        Var(1).lt(Int(1)).negate(),
        eq(5, 1),
        vec![close_a, close_b, eof, join],
    );
    m
}

/// One numeric public label and one typed identifier ending07. Classification
/// shapes/provenance are helper tests; this models the registration effects.
pub fn public_metadata(control: u8) -> Model {
    let name = match control {
        1 => "DreamC1PublicMetadataOverScrub",
        2 => "DreamC1PublicMetadataBlanket",
        _ => "DreamC1PublicMetadata",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("phase", 3, 0),
            ("public", 1, 0),
            ("independentSecret", 1, 0),
            ("registered", 1, 0),
            ("typedIdValid", 1, 1),
        ],
    );
    for public in 0..2 {
        for known in 0..2 {
            act(
                &mut m,
                &format!("ResolvePublic{public}Known{known}"),
                eq(0, 0),
                &[
                    (0, Int(1)),
                    (1, Int(public)),
                    (2, Int(known)),
                    (3, Int(known)),
                ],
            );
        }
    }
    act(
        &mut m,
        "RegisterTypedPublicEnvironment",
        all([eq(0, 1), eq(1, 1)]),
        &[(0, Int(2)), (3, if control == 1 { Int(1) } else { Var(2) })],
    );
    act(
        &mut m,
        "RegisterOtherEnvironment",
        all([eq(0, 1), eq(1, 0)]),
        &[(0, Int(2)), (3, if control == 2 { Var(2) } else { Int(1) })],
    );
    let corrupt = eq(3, 1);
    act(
        &mut m,
        "SanitizeTypedOperation",
        all([eq(0, 2), corrupt.clone()]),
        &[(0, Int(3)), (4, Int(0))],
    );
    act(
        &mut m,
        "PreserveTypedOperation",
        all([eq(0, 2), corrupt.negate()]),
        &[(0, Int(3)), (4, Int(1))],
    );
    safe(
        &mut m,
        "PublicRoutingMetadataPreservesTypedIdentity",
        eq(1, 0).or(eq(2, 1)).or(eq(4, 1)),
    );
    safe(
        &mut m,
        "ActualAndIndependentSecretsRemainRegistered",
        Var(0).lt(Int(2)).or(all([eq(1, 1), eq(2, 0)]).or(eq(3, 1))),
    );
    m
}

/// Returned ownership crosses the verdict (CPU/clock/cancel), then bounded
/// disposal precedes credit release and the externally visible Done receipt.
/// Pieces/credits are abstract; CPU, wait intervals and actual serde are tests.
pub fn returned_owner(control: u8) -> Model {
    let name = match control {
        1 => "DreamC1ReturnedOwnerUnpaced",
        2 => "DreamC1ReturnedOwnerCredit",
        3 => "DreamC1ReturnedOwnerDone",
        _ => "DreamC1ReturnedOwner",
    };
    let mut m = base(
        name,
        control != 0,
        &[
            ("phase", 4, 0),
            ("pieces", 2, 0),
            ("credit", 1, 0),
            ("verdict", 3, 0),
            ("unpaced", 1, 0),
        ],
    );
    let produce = m.actions.len();
    act(
        &mut m,
        "ProduceIntoExternalSlot",
        eq(0, 0),
        &[(0, Int(1)), (1, Int(2)), (2, Int(1))],
    );
    let mut fair = vec![produce];
    for verdict in 1..=3 {
        fair.push(m.actions.len());
        act(
            &mut m,
            &format!("RejectCpuClockCancel{verdict}"),
            eq(0, 1),
            &[(0, Int(2)), (3, Int(verdict))],
        );
    }
    for pieces in 1..=2 {
        fair.push(m.actions.len());
        act(
            &mut m,
            &format!("PacedDropPiece{pieces}"),
            all([eq(0, 2), eq(1, pieces)]),
            &[(1, Int(pieces - 1))],
        );
    }
    if control == 1 {
        act(
            &mut m,
            "ImplicitUnmeasuredDropOnRejectedReturn",
            all([eq(0, 2), eq(1, 2)]),
            &[(1, Int(0)), (4, Int(1))],
        );
    }
    fair.push(m.actions.len());
    act(
        &mut m,
        "ReleaseCreditAfterLastOwnedPiece",
        all([eq(0, 2), if control == 2 { Bool(true) } else { eq(1, 0) }]),
        &[(0, Int(3)), (2, Int(0))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "DoneAfterRealSettlement",
        if control == 3 {
            eq(0, 2).or(eq(0, 3))
        } else {
            eq(0, 3)
        },
        &[(0, Int(4))],
    );
    safe(
        &mut m,
        "EveryReturnedOwnerKeepsItsCredit",
        eq(1, 0).or(eq(2, 1)),
    );
    safe(&mut m, "EveryRejectedPieceIsPaced", eq(4, 0));
    safe(
        &mut m,
        "DoneAfterDrainedOwnershipAndCredit",
        eq(0, 4).negate().or(all([eq(1, 0), eq(2, 0)])),
    );
    progress(
        &mut m,
        "RejectedOwnerEventuallySettles",
        Bool(true),
        eq(0, 4),
        fair,
    );
    m
}


/// Two named jobs/owners, projected onto one two-call run and a peer whose native
/// completion may be withheld forever. CPU buckets are abstract milliseconds:
/// app 0..2, native 0..3, inclusive 0..3, and post-end debt 0..27. The projection
/// explores each selected job against the other held owner; it is not an
/// implementation refinement or an OS-return/CPU/wall-time theorem.
///
/// Native outcomes enter owned state before clock/CPU/wall/cancel verdicts.
/// Rest is an observed monotonic interval AFTER ReturnNative, never prior idle or
/// IO. Weak fairness covers local control, rest, verdict and settlement actions;
/// ReturnNative/ObserveExit are external observations, without OS fairness.
/// Once rest is paid, span-local CPU/timing scratch is canonicalized; cumulative
/// actual/reported cost, effect/op, currentness and owner custody remain intact.
/// Foreground changes are relevant through work/native admission, while context
/// changes remain available through final observed exit and settlement.
pub fn quantum_debt(control: u8) -> Model {
    let suffix = match control {
        1 => "UnpaidRest",
        2 => "FakeNativeApp",
        3 => "RepeatedEffect",
        4 => "EarlyOwnerRelease",
        5 => "HiddenNativeCpu",
        6 => "MissingCost",
        7 => "ClockEffect",
        8 => "CancelEffect",
        9 => "AbaEffect",
        10 => "WithheldCompletion",
        11 => "EarlyPresent",
        _ => "",
    };
    let mut m = base(
        &format!("DreamC1QuantumDebt{suffix}"),
        control != 0,
        &[
            ("phase", 10, 0),        // 0 ready,1/5 native,2/6 returned,3/7 debt,4 parked,8 exited,9 settled,10 presented
            ("calls", 2, 0),
            ("kind", 2, 0),          // 1 sole compute creation,2 no-replace publication
            ("in_flight", 1, 0),
            ("effect", 2, 0),        // absent/present/unknown
            ("captured_effect", 2, 0),
            ("captured_op", 2, 0),
            ("invocations", 2, 0),
            ("owner", 1, 1),         // reservation, then actual owned handle/custody
            ("exited", 1, 0),
            ("settled", 1, 0),
            ("presented", 1, 0),
            ("debt", 27, 0),
            ("inclusive_cpu", 3, 0),
            ("native_cpu", 3, 0),
            ("app_cpu", 2, 0),
            ("total_cpu", 9, 0),
            ("actual_cpu", 9, 0),
            ("sources_closed", 1, 0),
            ("started", 1, 0),
            ("foreground", 1, 0),
            ("epoch", 2, 0),         // A1/B2/A3: binding remains A1
            ("cancel", 1, 0),
            ("verdict", 2, 0),       // pending/accepted/rejected
            ("clock_wall", 2, 0),    // valid/over25ms/missing clock
            ("quarantined", 1, 0),
            ("hello", 1, 0),
            ("app_done", 1, 0),
            ("unpaid_admission", 1, 0),
            ("fake_app_class", 4, 0), // parser/control/cleanup/whole control tick
            ("peer_native", 1, 0),
            ("peer_control", 1, 0),
            ("job", 1, 0),           // both symmetric job/owner identities are explored
            ("selected", 1, 0),
            ("rest_observed", 1, 0),
            ("completed", 1, 0),
            ("admitted_epoch", 2, 0),
            ("admitted_foreground", 1, 0),
            ("admitted_closed", 1, 0),
            ("peer_owner", 1, 0),
            ("actual_compute_owner", 2, 0), // absent/present/unknown, before timing verdict
            ("caller_owner", 2, 0), // startup caller/sole compute/shared hot reactor
        ],
    );
    for job in 0..2 {
        act(&mut m, &format!("SelectJob{job}AgainstHeldPeer"), eq(33, 0),
            &[(32, Int(job)), (33, Int(1))]);
    }
    // Pick the held peer before the focused run. The two job selections cover
    // either physical identity; later placement of this unrelated admission only
    // duplicates states and cannot affect the focused run's transitions.
    act(&mut m, "AdmitIndependentPeerNative", all([eq(33, 1), eq(30, 0), eq(0, 0)]),
        &[(30, Int(1)), (39, Int(1))]);
    let mut fair = vec![];
    let peer_control = m.actions.len();
    act(&mut m, "IndependentPeerControlDuringDebtOrNative", eq(31, 0), &[(31, Int(1))]);
    m.weak_fair.push(vec![peer_control]);
    m.responses.push(Response { name: "PeerControlsNeverAwaitWork".into(),
        request: Bool(true), response: eq(31, 1) });

    let cancel = m.actions.len();
    act(&mut m, "CancelOrDeadline", all([eq(33, 1), eq(22, 0), eq(10, 0)]), &[(22, Int(1))]);
    m.weak_fair.push(vec![cancel]);
    for phase in [1, 5] {
        fair.push(m.actions.len());
        act(&mut m, &format!("QuarantineWithheldCompletion{phase}"),
            all([eq(0, phase), eq(22, 1), eq(25, 0)]), &[(25, Int(1))]);
    }
    for epoch in 0..2 {
        act(&mut m, &format!("ChangeContextA1B2A3_{epoch}"),
            all([eq(19, 1), eq(21, epoch), eq(10, 0)]), &[(21, Int(epoch + 1))]);
    }
    for fg in 0..2 {
        act(&mut m, &format!("Foreground{fg}"), all([eq(19, 1), eq(10, 0), eq(0, 4).or(eq(0, 5))]), &[(20, Int(fg))]);
    }
    for (kind, ready, flight, returned, debt_phase) in [(1, 0, 1, 2, 3), (2, 4, 5, 6, 7)] {
        let source_gate = if kind == 1 { Bool(true) } else {
            all([eq(18, 1), eq(19, 1), eq(20, 0), eq(21, 0), eq(27, 1)])
        };
        act(&mut m, &format!("AdmitNative{kind}"),
            all([eq(33, 1), eq(30, 1), eq(0, ready), eq(1, kind - 1), eq(12, 0),
                eq(3, 0), eq(22, 0), source_gate]),
            &[(0, Int(flight)), (1, Int(kind)), (2, Int(kind)), (3, Int(1)),
                (4, Int(2)), (5, Int(2)), (6, Int(kind)), (7, Int(1)),
                (23, Int(0)), (13, Int(0)), (14, Int(0)), (15, Int(0)),
                (34, Int(0)), (36, Var(21)), (37, Var(20)), (38, Var(18)),
                (41, Int(kind - 1)), (29, Int(0))]);
        // At most one native call per span. Return records its exact owned
        // effect/op before every retrospective timing/clock/cancel decision.
        for (app, native, clock_wall) in [(0, 0, 0), (1, 0, 0), (0, 1, 0),
            (1, 1, 0), (1, 2, 0), (0, 3, 0), (0, 1, 1), (0, 3, 2)] {
            let inclusive = app + native;
            for effect in 0..3 {
                let mut updates = vec![(0, Int(returned)), (3, Int(0)),
                    (4, Int(effect)), (5, Int(effect)), (13, Int(inclusive)),
                    (14, Int(native)), (15, Int(app)),
                    (16, Var(16).plus(Int(if control == 5 { app } else { inclusive }))),
                    (17, Var(17).plus(Int(inclusive))), (24, Int(clock_wall))];
                if kind == 1 { updates.push((40, Int(effect))); }
                act(&mut m, &format!("ReturnNative{kind}App{app}Cpu{native}Timing{clock_wall}Effect{effect}"),
                    all([eq(0, flight), eq(3, 1)]), &updates);
            }
        }
        for (app, native, clock_wall) in [(0, 0, 0), (1, 0, 0), (0, 1, 0),
            (1, 1, 0), (1, 2, 0), (0, 3, 0), (0, 1, 1), (0, 3, 2)] {
            let inclusive: i64 = app + native;
            for effect in 0..3 {
                let accept = effect == 1 && clock_wall == 0 && native <= 2;
                let lose = (control == 7 && clock_wall == 2)
                    || control == 8 || (control == 9 && kind == 2);
                let cause = if control == 8 { eq(22, 1) }
                    else if control == 9 && kind == 2 { eq(21, 2) }
                    else { Bool(true) };
                fair.push(m.actions.len());
                let mut updates = vec![(0, Int(debt_phase)),
                    (12, Int(if control == 6 { 0 } else { 9 * inclusive.max(1) })),
                    (23, Int(if accept { 1 } else { 2 }))];
                if !accept { updates.push((22, Int(1))); }
                if lose { updates.push((4, Int(0))); }
                act(&mut m, &format!("VerdictNative{kind}Cost{inclusive}Effect{effect}Timing{clock_wall}Cpu{native}"),
                    all([eq(0, returned), eq(13, inclusive), eq(14, native),
                        eq(4, effect), eq(24, clock_wall), cause]), &updates);
            }
        }
        for inclusive in 0..=3 {
            let debt = 9 * inclusive.max(1);
            fair.push(m.actions.len());
            act(&mut m, &format!("PayRestAfterSpanEnd{kind}Cost{inclusive}"),
                all([eq(0, debt_phase), eq(12, debt), eq(13, inclusive)]),
                &[(12, Int(0)), (0, Int(if kind == 1 { 4 } else { 7 })), (34, Int(1)),
                    (13, Int(0)), (14, Int(0)), (15, Int(0)), (24, Int(0))]);
        }
        if control == 1 {
            act(&mut m, &format!("UsePriorIdleOrSkipRequiredRest{kind}"),
                all([eq(0, debt_phase), Int(0).lt(Var(12))]),
                &[(12, Int(0)), (0, Int(if kind == 1 { 4 } else { 7 })), (28, Int(1))]);
        }
        if control == 3 {
            for phase in [flight, returned, debt_phase] {
                act(&mut m, &format!("RepeatSameNativeOperation{kind}Phase{phase}"),
                    all([eq(0, phase), eq(7, 1)]), &[(7, Int(2))]);
            }
        }
    }
    act(&mut m, "HelloAfterBootstrapDebt", all([eq(0, 4), eq(12, 0), eq(26, 0), eq(22, 0)]), &[(26, Int(1))]);
    act(&mut m, "MatchingStart", all([eq(26, 1), eq(19, 0), eq(22, 0)]), &[(19, Int(1))]);
    act(&mut m, "Close", all([eq(19, 1), eq(18, 0), eq(22, 0)]), &[(18, Int(1))]);
    act(&mut m, "AdmitApp", all([eq(0, 4), eq(12, 0), eq(18, 1), eq(19, 1),
        eq(20, 0), eq(21, 0), eq(22, 0), eq(27, 0)]),
        &[(27, Int(1)), (2, Int(0)), (15, Int(1)), (12, Int(9)), (34, Int(0)),
            (36, Var(21)), (37, Var(20)), (38, Var(18)),
            (16, Var(16).plus(Int(1))), (17, Var(17).plus(Int(1)))]);
    fair.push(m.actions.len());
    act(&mut m, "PayRestAfterAppEnd", all([eq(0, 4), eq(27, 1), eq(12, 9)]),
        &[(12, Int(0)), (34, Int(1)), (13, Int(0)), (14, Int(0)), (15, Int(0)), (24, Int(0))]);
    act(&mut m, "RejectTwoMsOrdinaryApp", all([eq(0, 4), eq(12, 0), eq(18, 1), eq(19, 1),
        eq(20, 0), eq(21, 0), eq(22, 0), eq(27, 0)]),
        &[(27, Int(1)), (2, Int(0)), (15, Int(2)), (12, Int(18)), (34, Int(0)),
            (23, Int(2)), (22, Int(1)), (36, Var(21)), (37, Var(20)), (38, Var(18)),
            (16, Var(16).plus(Int(2))), (17, Var(17).plus(Int(2)))]);
    fair.push(m.actions.len());
    act(&mut m, "PayRestAfterRejectedAppEnd", all([eq(0, 4), eq(27, 1), eq(12, 18)]),
        &[(12, Int(0)), (34, Int(1)), (13, Int(0)), (14, Int(0)), (15, Int(0)), (24, Int(0))]);
    if control == 2 {
        for class in 1..=4 {
            act(&mut m, &format!("MisclassifyTwoMsAppAsNativeClass{class}"),
                all([eq(0, 4), eq(12, 0), eq(19, 1), eq(22, 0), eq(27, 0)]),
                &[(27, Int(1)), (15, Int(2)), (29, Int(class)), (12, Int(18)),
                    (16, Var(16).plus(Int(2))), (17, Var(17).plus(Int(2)))]);
        }
    }
    // Exit is never inferred from a timeout, channel closure, or a failed verdict.
    // Unknown effects retain the owner in quarantine and cannot release credits.
    act(&mut m, "ObserveExit", all([eq(0, 4).or(eq(0, 7)).or(eq(0, 0)),
        eq(3, 0), eq(12, 0), Var(4).lt(Int(2)), eq(22, 1).or(eq(1, 2))]),
        &[(0, Int(8)), (9, Int(1)), (40, Int(0)), (20, Int(0)),
            (13, Int(0)), (14, Int(0)), (15, Int(0)), (24, Int(0))]);
    for phase in [0, 3, 4, 7] {
        fair.push(m.actions.len());
        act(&mut m, &format!("QuarantineAwaitingEffectOrExit{phase}"),
            all([eq(0, phase), eq(22, 1), eq(25, 0)]), &[(25, Int(1))]);
    }
    fair.push(m.actions.len());
    act(&mut m, "SettleAfterObservedExit", all([eq(0, 8), eq(9, 1), eq(3, 0), eq(12, 0)]),
        &[(0, Int(9)), (8, Int(0)), (10, Int(1)), (4, Int(0))]);
    fair.push(m.actions.len());
    act(&mut m, "Present", all([eq(0, 9), eq(10, 1), eq(9, 1), eq(12, 0),
        eq(1, 2), eq(23, 1), eq(18, 1), eq(21, 0), eq(22, 0)]),
        &[(0, Int(10)), (11, Int(1)), (35, Int(1))]);
    fair.push(m.actions.len());
    act(&mut m, "PresentRefusal", all([eq(0, 9), eq(10, 1), eq(9, 1), eq(12, 0),
        eq(22, 1).or(eq(23, 2)).or(eq(21, 0).negate())]),
        &[(0, Int(10)), (11, Int(1))]);
    if control == 4 {
        act(&mut m, "ReleaseOwnerBeforeEffectOrExit", all([eq(8, 1), eq(10, 0), eq(1, 1)]), &[(8, Int(0))]);
    }
    if control == 10 {
        act(&mut m, "InferSettlementFromWithheldNative", all([eq(3, 1), eq(22, 1)]), &[(10, Int(1)), (8, Int(0))]);
    }
    if control == 11 {
        act(&mut m, "PresentBeforeCurrentValidationOrExit", eq(1, 2), &[(11, Int(1))]);
    }
    safe(&mut m, "NoNativeOnHotOwner", eq(3, 0).or(Var(41).lt(Int(2))));
    safe(&mut m, "ActualAndPeerOwnersKeepCustody", eq(40, 0).or(eq(8, 1)).and(eq(30, 0).or(eq(39, 1))));
    safe(&mut m, "NoSourceAccessBeforeHelloAndStart", eq(18, 0).or(all([eq(26, 1), eq(19, 1)])));
    safe(&mut m, "NativeCountAndOriginalOperationStayBound", Var(1).le(Int(2)).and(Var(6).eq(Var(1))));
    safe(&mut m, "NoAdmissionWithUnpaidRestOrSavedIdle", eq(28, 0));
    safe(&mut m, "OnlyEnumeratedNativeCalls", eq(29, 0));
    safe(&mut m, "NoRepeatedUnresolvedNativeOperation", Var(7).le(Int(1)));
    safe(&mut m, "FullCallingThreadCostIsNeverHidden", Var(16).eq(Var(17)));
    safe(&mut m, "CostCreatesPostEndDebt", all([eq(0, 3).or(eq(0, 7)), eq(34, 0)])
        .negate().or(Int(0).lt(Var(12))));
    for cost in 0..=3 {
        safe(&mut m, &format!("ExactFullCostCreatesDebt{cost}"),
            all([eq(0, 3).or(eq(0, 7)), eq(34, 0), eq(13, cost)]).negate()
                .or(eq(12, 9 * cost.max(1))));
    }
    safe(&mut m, "BootstrapDebtBeforeHello", eq(0, 3).negate().or(Int(0).lt(Var(12))));
    safe(&mut m, "AdmissionRequiresClosedCurrentQuiescentSources", eq(1, 0).or(eq(2, 1))
        .or(all([eq(36, 0), eq(37, 0), eq(38, 1)])));
    safe(&mut m, "CompletionRequiresCurrentValidation", eq(35, 0).or(all([eq(18, 1), eq(21, 0), eq(22, 0), eq(23, 1), eq(9, 1), eq(10, 1)])));
    safe(&mut m, "CapturedEffectSurvivesAllVerdicts", eq(10, 1).or(eq(4, 0).negate()).or(eq(5, 0)));
    safe(&mut m, "OwnerReleaseRequiresRealSettlement", eq(8, 1).or(all([eq(10, 1), eq(9, 1), eq(3, 0)])));
    safe(&mut m, "PresentedRequiresRealExitAndSettlement", eq(11, 0).or(all([eq(9, 1), eq(10, 1), eq(3, 0), eq(12, 0)])));
    safe(&mut m, "NativeAcceptanceUsesExactCpuAndTiming", eq(23, 1).negate().or(all([Var(14).le(Int(2)), Var(15).le(Int(1)), eq(24, 0)])));
    m.responses.push(Response { name: "ObservedSettlementEventuallyPresents".into(),
        request: eq(10, 1), response: eq(11, 1) });
    m.responses.push(Response { name: "CancelledOwnerSettlesOrHonestlyQuarantines".into(),
        request: eq(22, 1), response: eq(10, 1).or(eq(25, 1)) });
    // Native return and exit observation have no fairness. Local quarantine
    // remains available when either external completion is withheld.

    m.weak_fair.push(fair);
    m
}


#[test]
fn quantum_debt_outcomes_and_controls_are_causally_reachable() {
    let good = quantum_debt(0);
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
    for job in 0..2 {
        assert!(checked.states.iter().any(|s| s[32] == job && s[35] == 1),
            "both named job projections must have real completed paths");
    }
    for timing in 0..3 {
        assert!(checked.states.iter().any(|s| s[0] == 3 && s[24] == timing
            && s[23] == 2 && s[4] == 1 && s[5] == 1 && s[8] == 1 && s[40] == 1),
            "CPU/wall/clock rejection must retain an actual returned effect");
    }
    assert!(checked.states.iter().any(|s| s[0] == 3 && s[13] == 3
        && s[14] == 2 && s[15] == 1 && s[12] == 27 && s[26] == 0),
        "legal native+app cost3 must owe27 after bootstrap, before Hello");
    assert!(checked.states.iter().any(|s| s[0] == 1 && s[22] == 1
        && s[25] == 1 && s[8] == 1 && s[31] == 1 && s[30] == 1),
        "withheld own and peer completions must preserve custody with independent control");
    assert!(checked.states.iter().any(|s| s[0] == 6 && s[21] == 2 && s[4] == 1),
        "a returned effect from A1 must remain owned across B2/A3");
    for (mode, property) in [
        (1, "NoAdmissionWithUnpaidRestOrSavedIdle"),
        (2, "OnlyEnumeratedNativeCalls"),
        (3, "NoRepeatedUnresolvedNativeOperation"),
        (4, "OwnerReleaseRequiresRealSettlement"),
        (5, "FullCallingThreadCostIsNeverHidden"),
        (6, "CostCreatesPostEndDebt"),
        (7, "CapturedEffectSurvivesAllVerdicts"),
        (8, "CapturedEffectSurvivesAllVerdicts"),
        (9, "CapturedEffectSurvivesAllVerdicts"),
        (10, "OwnerReleaseRequiresRealSettlement"),
        (11, "PresentedRequiresRealExitAndSettlement"),
    ] {
        let bad = quantum_debt(mode).check(100_000).unwrap();
        assert!(bad.violations.iter().any(|v| v.property == property),
            "control{mode} must violate its own causal property {property}");
        if mode == 2 {
            for class in 1..=4 {
                assert!(bad.states.iter().any(|s| s[29] == class && s[15] == 2),
                    "parser/control/cleanup/whole-tick2ms controls must all execute");
            }
        }
        if mode == 3 {
            for kind in 1..=2 {
                assert!(bad.states.iter().any(|s| s[2] == kind && s[7] == 2),
                    "compute creation and publication repeats must each be reachable");
            }
        }
    }
}

/// Earliest private bootstrap handoff. One parent/child birth, one fixed permit,
/// three controlled child spans (argv, channel, Builder), and A1/B2/A3 identities.
/// Span-local costs are forgotten only after their post-end debt is paid; effect,
/// incarnation, grant-attempt and custody state survive. CPU/debt are abstract ms.
/// Native return, permit delivery and actual exit have no fairness assumption.
/// The 128-byte record, FD/token/parser/deadline and process markers are witnesses,
/// not refinements established by this finite model.
pub fn bootstrap_handoff(control: u8) -> Model {
    let suffix = match control {
        1 => "EarliestGate", 2 => "ParentOverlap", 3 => "UnpaidGrant",
        4 => "UnknownCostGrant", 5 => "DebtReset", 6 => "StaleGrantABA",
        7 => "DuplicateGrant", 8 => "RevokedLease", 9 => "UnknownBirthRelease",
        10 => "UnknownBirthRetry", 11 => "UnpaidHello", 12 => "UnpaidStart",
        13 => "ProbeStart", 14 => "NullWork", 15 => "ControlCost",
        16 => "ControlPayload", 17 => "AdmissionRetry", _ => "",
    };
    let mut m = base(&format!("DreamC1BootstrapHandoff{suffix}"), control != 0, &[
        ("phase", 17, 0), ("birth", 2, 0), ("retained_birth", 2, 0),
        ("child_attempts", 2, 0), ("permit", 3, 0), ("permit_attempts", 2, 0),
        ("birth_epoch", 2, 0), ("epoch", 2, 0), ("permit_epoch", 2, 0),
        ("permit_pipe", 2, 0), ("permit_nonce", 2, 0), ("pipe", 2, 0), ("nonce", 2, 0),
        ("cost_known", 1, 0), ("parent_debt", 27, 0), ("child_debt", 27, 0),
        ("span_cpu", 3, 0),
        // parent/childBootstrap/parentPrepare/childWork/settlement/none
        ("expensive_owner", 5, 0), ("slot", 1, 1), ("waiting", 1, 0),
        ("bootstrapped", 1, 0), ("hello", 1, 0), ("start", 1, 0),
        ("exited", 1, 0), ("settled", 1, 0), ("foreground", 1, 0),
        ("cancel", 1, 0), ("lease", 1, 0), ("bootstrap_span", 3, 0),
        ("compute_owner", 2, 0), ("parent_active", 1, 0), ("child_active", 1, 0),
        ("quarantined", 1, 0), ("admission_valid", 1, 1),
        ("grant_parent_debt", 27, 0), ("grant_cost_known", 1, 1),
        ("reset_without_rest", 1, 0), ("early_class", 3, 0), ("control_seen", 1, 0),
        ("hello_child_debt", 27, 0), ("start_parent_debt", 27, 0),
        ("birth_inflight", 1, 0), ("child_inflight", 1, 0),
        ("profile", 2, 0), // unselected/probe=true/work probe=false
        ("full_context", 1, 0), // null/full; probe remains null permanently
        ("prepared_actor_proof", 1, 0), // external frozen-proof assumption, not authentication proof
        ("admission_attempts", 3, 0), ("admission_stage", 3, 0), // none/permit/Start/refused
        ("admission_cost_known", 1, 0), ("admission_cpu", 2, 0),
        ("admission_payload", 1, 0), ("prepared_proof_chunks", 7, 0),
        ("source_access", 1, 0),
    ]);
    act(&mut m, "SelectSourceFreeProbeNullContext", all([eq(0, 0), eq(43, 0)]),
        &[(43, Int(1)), (44, Int(0))]);
    act(&mut m, "SelectRealJobFullContext", all([eq(0, 0), eq(43, 0)]),
        &[(43, Int(2)), (44, Int(1))]);
    if control == 14 {
        act(&mut m, "SelectRealJobWithNullContext", all([eq(0, 0), eq(43, 0)]),
            &[(43, Int(2)), (44, Int(0))]);
    }
    let profile_authorized = eq(43, 1).and(eq(44, 0)).and(eq(45, 0))
        .or(all([eq(43, 2), eq(44, 1), eq(45, 1)]));
    let work_authorized = all([eq(43, 2), eq(44, 1), eq(45, 1)]);
    let mut local_fair = vec![];
    let controls = m.actions.len();
    act(&mut m, "IndependentControl", eq(38, 0), &[(38, Int(1))]);
    m.weak_fair.push(vec![controls]);
    m.responses.push(Response { name: "ControlsDoNotWaitForBootstrapOrDebt".into(),
        request: Bool(true), response: eq(38, 1) });
    act(&mut m, "Prepare", all([eq(0, 0), eq(3, 0), eq(26, 0), eq(43, 0).negate()]),
        &[(0, Int(1)), (3, Int(1)), (27, Int(1)), (30, Int(1)), (41, Int(1)), (6, Var(7))]);
    // The actual OS outcome enters retained owned state before a clock/debt or
    // cancellation verdict. Missing cost cannot be turned into a valid permit.
    for birth in 0..=2 {
        for cost in 0..=3 {
            act(&mut m, &format!("BirthRetainEffect{birth}Cost{cost}"), eq(0, 1),
                &[(0, Int(2)), (1, Int(birth)), (2, Int(birth)),
                    (13, Int(1)), (16, Int(cost)), (30, Int(0)), (41, Int(0))]);
        }
        act(&mut m, &format!("BirthRetainEffect{birth}MissingCost"), eq(0, 1),
            &[(0, Int(2)), (1, Int(birth)), (2, Int(birth)),
                (13, Int(0)), (16, Int(3)), (30, Int(0)), (41, Int(0))]);
    }
    for cost in 0..=3 {
        local_fair.push(m.actions.len());
        act(&mut m, &format!("ParentVerdictCost{cost}"), all([eq(0, 2), eq(16, cost), eq(13, 1)]),
            &[(0, Int(3)), (14, Int(9 * cost.max(1))), (17, Int(5))]);
    }
    local_fair.push(m.actions.len());
    act(&mut m, "ParentVerdictMissingCost", all([eq(0, 2), eq(13, 0)]),
        &[(0, Int(3)), (14, Int(27)), (17, Int(5)), (26, Int(1)), (32, Int(1))]);
    local_fair.push(m.actions.len());
    act(&mut m, "PayParent", all([eq(0, 3), Int(0).lt(Var(14))]),
        &[(0, Int(4)), (14, Int(0)), (16, Int(0)), (19, Int(1))]);
    let current = all([Var(6).eq(Var(7)), Var(11).eq(Var(7)), Var(12).eq(Var(7))]);
    // This observes availability of the already prepared, public-only frozen
    // proof. It does not model SQLite/raw-proof IO or establish actor authority.
    // Exact provenance, <=7*16KiB preparation and its measured callback are real
    // consumer fixtures; probe never requests or receives an actor proof.
    act(&mut m, "ObservePreparedPublicActorProof", all([eq(0, 4), eq(43, 2), eq(44, 1),
        eq(45, 0), eq(46, 0), eq(26, 0)]), &[(45, Int(1)), (51, Int(7))]);
    for (known, cpu) in [(1, 0), (1, 1), (1, 2), (0, 0)] {
        let accepted = control == 15 || (known == 1 && cpu <= 1);
        let mut updates = vec![(46, Int(1)), (47, Int(if accepted { 1 } else { 3 })),
            (48, Int(known)), (49, Int(cpu)), (50, Int(0))];
        if !accepted { updates.push((26, Int(1))); }
        act(&mut m, &format!("FinalPermitAdmissionKnown{known}Cpu{cpu}"),
            all([eq(0, 4), eq(1, 1), eq(13, 1), eq(14, 0), eq(15, 0), eq(46, 0),
                eq(27, 1), eq(26, 0), eq(25, 0), current.clone(), profile_authorized.clone()]), &updates);
    }
    let grant_guard = all([eq(0, 4), eq(1, 1), eq(13, 1), eq(14, 0), eq(15, 0),
        eq(4, 0), eq(5, 0), eq(17, 5), eq(27, 1), eq(26, 0), eq(25, 0),
        eq(46, 1), eq(47, 1), current.clone(), profile_authorized.clone()]);
    for permit in 1..=2 {
        act(&mut m, if permit == 1 { "GrantPartialOnce" } else { "Grant" }, grant_guard.clone(),
            &[(4, Int(permit)), (5, Int(1)), (8, Var(7)), (9, Var(11)), (10, Var(12)),
                (34, Var(14)), (35, Var(13)), (48, Int(1)), (49, Int(0))]);
    }
    let permit_current = all([eq(4, 2), Var(8).eq(Var(7)), Var(9).eq(Var(11)),
        Var(10).eq(Var(12)), current.clone()]);
    act(&mut m, "ReadGrant", all([eq(0, 4), eq(5, 1), eq(13, 1), eq(14, 0),
        eq(17, 5), eq(27, 1), eq(26, 0), eq(25, 0), permit_current.clone(), profile_authorized.clone()]),
        &[(0, Int(5)), (17, Int(1)), (19, Int(0))]);
    // The lease is rechecked at EVERY controlled span, including after each rest.
    // A closed/extra/HUP/EOF lease never becomes valid merely because cost was paid.
    for span in 0..3 {
        act(&mut m, &format!("ChildSpan{}", ["Argv", "Channel", "Builder"][span]),
            all([eq(0, 5), eq(17, 1), eq(28, span as i64), eq(14, 0), eq(15, 0), eq(13, 1),
                eq(27, 1), eq(26, 0), eq(25, 0), permit_current.clone(), profile_authorized.clone()]),
            &[(0, Int(6)), (31, Int(1)), (42, Int(1)), (33, Int(1))]);
        let costs: &[i64] = if span == 2 { &[1, 3] } else { &[1] };
        let effects: &[i64] = if span == 2 { &[0, 1, 2] } else { &[0] };
        for &cost in costs {
            for &effect in effects {
                let mut updates = vec![(0, Int(7)), (16, Int(cost)), (31, Int(0)), (42, Int(0))];
                if span == 2 { updates.push((29, Int(effect))); }
                act(&mut m, &format!("RetainChildSpan{span}Effect{effect}Cost{cost}"),
                    all([eq(0, 6), eq(28, span as i64)]), &updates);
            }
        }
        for cost in 1..=3 {
            local_fair.push(m.actions.len());
            let mut updates = vec![(0, Int(8)), (15, Int(9 * cost)), (28, Int(span as i64 + 1))];
            if span == 2 { updates.push((20, Int(1))); }
            act(&mut m, &format!("ChildVerdict{span}Cost{cost}"),
                all([eq(0, 7), eq(28, span as i64), eq(16, cost),
                    if span == 2 { eq(29, 1) } else { Bool(true) }]), &updates);
            if span == 2 {
                local_fair.push(m.actions.len());
                act(&mut m, &format!("ChildVerdictRejectedEffectCost{cost}"),
                    all([eq(0, 7), eq(28, 2), eq(16, cost), eq(29, 1).negate()]),
                    &[(0, Int(8)), (15, Int(9 * cost)), (28, Int(3)), (26, Int(1))]);
            }
        }
    }
    local_fair.push(m.actions.len());
    act(&mut m, "PayChild", all([eq(0, 8), Int(0).lt(Var(15))]),
        &[(0, Int(5)), (15, Int(0)), (16, Int(0))]);
    act(&mut m, "Hello", all([eq(0, 5), eq(17, 1), eq(28, 3), eq(20, 1), eq(29, 1),
        eq(14, 0), eq(15, 0), eq(27, 1), eq(26, 0), eq(25, 0), permit_current.clone(), profile_authorized.clone()]),
        &[(0, Int(9)), (21, Int(1)), (17, Int(2)), (39, Var(15))]);
    for cost in 0..=3 {
        act(&mut m, &format!("ParentPrepareCost{cost}"),
            all([eq(0, 9), eq(17, 2), eq(21, 1), eq(14, 0), eq(15, 0), eq(26, 0), eq(25, 0), current.clone(), work_authorized.clone()]),
            &[(0, Int(10)), (14, Int(9 * cost.max(1))), (16, Int(cost))]);
    }
    local_fair.push(m.actions.len());
    act(&mut m, "PayParentPrepare", all([eq(0, 10), Int(0).lt(Var(14))]),
        &[(0, Int(11)), (14, Int(0)), (16, Int(0))]);
    for (known, cpu) in [(1, 0), (1, 1), (1, 2), (0, 0)] {
        let accepted = control == 15 || (known == 1 && cpu <= 1);
        let mut updates = vec![(46, Int(2)), (47, Int(if accepted { 2 } else { 3 })),
            (48, Int(known)), (49, Int(cpu)), (50, Int(0))];
        if !accepted { updates.push((26, Int(1))); }
        act(&mut m, &format!("FinalStartAdmissionKnown{known}Cpu{cpu}"),
            all([eq(0, 11), eq(14, 0), eq(15, 0), eq(46, 1), eq(47, 1), eq(26, 0),
                eq(25, 0), eq(27, 1), permit_current.clone(), work_authorized.clone()]), &updates);
    }
    act(&mut m, "PaidStart", all([eq(0, 11), eq(17, 2), eq(21, 1), eq(14, 0), eq(15, 0),
        eq(26, 0), eq(25, 0), eq(46, 2), eq(47, 2), permit_current.clone(), work_authorized.clone()]),
        &[(0, Int(12)), (22, Int(1)), (17, Int(3)), (40, Var(14)), (48, Int(1)), (49, Int(0))]);
    act(&mut m, "ChildWork", all([eq(0, 12), eq(17, 3), eq(14, 0), eq(15, 0), eq(26, 0),
        eq(25, 0), current.clone(), work_authorized.clone()]),
        &[(0, Int(17)), (31, Int(1)), (42, Int(1)), (52, Int(1))]);
    // A bounded application span has local completion fairness; this does not
    // add fairness to process/Builder native returns or external delivery/exit.
    local_fair.push(m.actions.len());
    act(&mut m, "ReturnChildWork", eq(0, 17),
        &[(0, Int(13)), (31, Int(0)), (42, Int(0)), (15, Int(9))]);
    local_fair.push(m.actions.len());
    act(&mut m, "PayChildWork", all([eq(0, 13), eq(15, 9)]),
        &[(0, Int(14)), (15, Int(0)), (17, Int(4))]);
    // Revoke/EOF are independent of native return, delivery or either debt lane.
    for event in ["RevokeClose", "RevokeExtraByte", "RevokeHup", "EOF"] {
        act(&mut m, event, all([eq(23, 0), eq(24, 0), eq(27, 1)]),
            &[(4, Int(3)), (27, Int(0)), (26, Int(1))]);
    }
    let cancel = m.actions.len();
    act(&mut m, "CancelOrDeadline", all([eq(26, 0), eq(23, 0), eq(24, 0)]), &[(26, Int(1))]);
    m.weak_fair.push(vec![cancel]);
    for epoch in 0..2 {
        let updates = vec![(7, Int(epoch + 1)), (11, Int(epoch + 1)), (12, Int(epoch + 1)),
            (4, Int(3)), (27, Int(0)), (26, Int(1))];
        let restart = all([eq(7, epoch), Int(0).lt(Var(0)), eq(23, 0), eq(24, 0)]);
        act(&mut m, &format!("RestartA1B2A3_{epoch}"),
            restart.clone().and(if control == 5 { all([eq(14, 0), eq(15, 0)]) } else { Bool(true) }), &updates);
        if control == 5 {
            let mut reset = updates;
            reset.extend([(14, Int(0)), (15, Int(0)), (36, Int(1))]);
            act(&mut m, &format!("RestartDropsActualUnpaidDebt{epoch}"),
                restart.and(Int(0).lt(Var(14)).or(Int(0).lt(Var(15)))), &reset);
        }
    }
    for fg in 0..2 {
        act(&mut m, &format!("BootstrapForeground{fg}"),
            all([Int(3).lt(Var(0)), Var(0).lt(Int(13))]), &[(25, Int(fg))]);
    }
    local_fair.push(m.actions.len());
    act(&mut m, "RefuseAndSettle", all([eq(26, 1), eq(41, 0), eq(42, 0),
        eq(14, 0), eq(15, 0), eq(30, 0), eq(31, 0), Var(0).lt(Int(14))]),
        &[(0, Int(14)), (17, Int(4)), (19, Int(0))]);
    local_fair.push(m.actions.len());
    act(&mut m, "QuarantineWithheldBirthOrComputeOrExit", all([eq(26, 1), eq(32, 0), eq(23, 0), eq(24, 0)]),
        &[(32, Int(1))]);
    // External fenced evidence, never a timeout or a closed pipe, establishes exit.
    act(&mut m, "ObserveExit", all([eq(0, 14), eq(1, 1), eq(14, 0), eq(15, 0),
        eq(41, 0), eq(42, 0)]), &[(0, Int(15)), (23, Int(1)), (29, Int(0)), (17, Int(5))]);
    local_fair.push(m.actions.len());
    act(&mut m, "SettleAbsentBirth", all([eq(0, 14), eq(1, 0), eq(41, 0)]),
        &[(0, Int(16)), (24, Int(1)), (18, Int(0)), (17, Int(5))]);
    local_fair.push(m.actions.len());
    act(&mut m, "SettleAfterActualExit", all([eq(0, 15), eq(23, 1)]),
        &[(0, Int(16)), (24, Int(1)), (18, Int(0))]);
    if control == 1 {
        for class in 1..=3 {
            let mut updates = vec![(0, Int(6)), (37, Int(class)), (33, Int(0)),
                (31, Int(1)), (42, Int(1)), (28, Int(class - 1)), (17, Int(1))];
            if class == 3 { updates.push((29, Int(1))); }
            act(&mut m, &format!("BypassEarliest{}Gate", ["Argv", "Channel", "Builder"][(class - 1) as usize]),
                all([eq(0, 4), eq(1, 1), eq(4, 0), eq(14, 0)]), &updates);
        }
    }
    if control == 2 {
        act(&mut m, "OverlapParentPrepareWithChildBootstrap", eq(31, 1),
            &[(30, Int(1)), (14, Int(9)), (17, Int(2))]);
    }
    if control == 3 || control == 4 {
        let cost_gate = if control == 4 { eq(13, 0) } else { eq(13, 1) };
        act(&mut m, "GrantBeforeKnownParentCostAndDebt", all([eq(0, 3), eq(1, 1), eq(5, 0), cost_gate]),
            &[(0, Int(4)), (4, Int(2)), (5, Int(1)), (8, Var(7)), (9, Var(11)), (10, Var(12)),
                (34, Var(14)), (35, Var(13))]);
    }
    if control == 6 {
        act(&mut m, "ReuseA1GrantPipeNonceAfterA3", all([eq(7, 2), eq(1, 1), eq(5, 1), eq(8, 0)]),
            &[(4, Int(2)), (0, Int(6)), (31, Int(1)), (42, Int(1)), (33, Int(0))]);
    }
    if control == 7 {
        act(&mut m, "RepeatSinglePermitWriteAttempt", eq(5, 1), &[(5, Int(2))]);
    }
    if control == 8 {
        act(&mut m, "IgnoreClosedExtraOrHupLeaseAfterRest", all([eq(0, 5), eq(27, 0),
            Var(28).lt(Int(3)), eq(15, 0)]), &[(0, Int(6)), (31, Int(1)), (42, Int(1)), (33, Int(0))]);
    }
    if control == 9 {
        act(&mut m, "ReleaseUnknownBirthWithoutExit", all([eq(1, 2), eq(18, 1)]), &[(18, Int(0))]);
    }
    if control == 10 {
        act(&mut m, "RetryPossiblyCompletedUnknownBirth", all([eq(1, 2), eq(3, 1)]),
            &[(3, Int(2)), (0, Int(1)), (41, Int(1)), (30, Int(1)), (17, Int(0))]);
    }
    if control == 11 {
        act(&mut m, "HelloBeforeBootstrapRestPaid", all([eq(0, 8), eq(28, 3), eq(20, 1), Int(0).lt(Var(15))]),
            &[(0, Int(9)), (21, Int(1)), (39, Var(15)), (17, Int(2))]);
    }
    if control == 12 {
        act(&mut m, "StartBeforeParentPrepareRestPaid", all([eq(0, 10), Int(0).lt(Var(14))]),
            &[(0, Int(12)), (22, Int(1)), (40, Var(14)), (17, Int(3))]);
    }
    if control == 13 {
        act(&mut m, "StartProbeDespiteMatchingIncarnationIds", all([eq(0, 9), eq(43, 1),
            eq(44, 0), eq(14, 0), eq(15, 0), eq(26, 0), permit_current.clone()]),
            &[(0, Int(12)), (22, Int(1)), (17, Int(3))]);
        act(&mut m, "ReadSourceAfterForbiddenProbeStart", all([eq(0, 12), eq(43, 1), eq(22, 1)]),
            &[(52, Int(1))]);
    }
    if control == 14 {
        act(&mut m, "GrantNullWorkByAmbientFallback", all([eq(0, 4), eq(43, 2), eq(44, 0),
            eq(1, 1), eq(13, 1), eq(14, 0), eq(5, 0), eq(26, 0), current.clone()]),
            &[(4, Int(2)), (5, Int(1)), (8, Var(7)), (9, Var(11)), (10, Var(12)),
                (34, Int(0)), (35, Int(1)), (46, Int(1)), (47, Int(1)), (48, Int(1))]);
    }
    if control == 16 {
        act(&mut m, "RunPayloadOrNativeInsideControlAdmission", all([eq(0, 4), eq(1, 1),
            eq(13, 1), eq(14, 0), eq(15, 0), eq(46, 0), eq(26, 0), eq(27, 1),
            work_authorized.clone(), current.clone()]),
            &[(46, Int(1)), (47, Int(1)), (48, Int(1)), (49, Int(1)), (50, Int(1))]);
    }
    if control == 17 {
        act(&mut m, "RetryFinalAdmissionAfterTwoAttempts", eq(46, 2), &[(46, Int(3))]);
    }
    safe(&mut m, "ProbePermanentlyForbidsStartSourceAndActorProof", eq(43, 1).negate()
        .or(all([eq(44, 0), eq(45, 0), eq(22, 0), eq(52, 0)])));
    safe(&mut m, "RealWorkNeverFallsBackFromNullContext", all([eq(43, 2), Int(0).lt(Var(5))]).negate()
        .or(all([eq(44, 1), eq(45, 1)])));
    safe(&mut m, "WorkSourceRequiresFullContextAndStart", eq(52, 0).or(all([eq(43, 2), eq(44, 1), eq(45, 1), eq(22, 1)])));
    safe(&mut m, "FinalParentAdmissionIsKnownOneMsControl", eq(47, 1).or(eq(47, 2)).negate()
        .or(all([eq(48, 1), Var(49).le(Int(1))])));
    safe(&mut m, "FinalParentAdmissionHasNoPayloadOrNativeWork", eq(50, 0));
    safe(&mut m, "FinalParentAdmissionHasAtMostTwoAttempts", Var(46).le(Int(2)));
    safe(&mut m, "EveryControlledBootstrapSpanRequiresCurrentPaidPermit", eq(33, 1));
    safe(&mut m, "ParentAndChildNeverOverlapExpensiveSpans", eq(30, 0).or(eq(31, 0)));
    safe(&mut m, "GrantRequiresPaidParentDebt", eq(5, 0).or(eq(34, 0)));
    safe(&mut m, "GrantRequiresKnownParentCost", eq(5, 0).or(eq(35, 1)));
    safe(&mut m, "RestartNeverResetsUnpaidDebt", eq(36, 0));
    safe(&mut m, "ValidGrantIsBoundToIncarnationPipeAndNonce", eq(4, 2).negate().or(all([
        Var(8).eq(Var(7)), Var(9).eq(Var(11)), Var(10).eq(Var(12)), Var(6).eq(Var(7))])));
    safe(&mut m, "SinglePermitAttempt", Var(5).le(Int(1)));
    safe(&mut m, "SingleChildAttemptNoUnknownEffectRetry", Var(3).le(Int(1)));
    safe(&mut m, "UnknownBirthRetainsItsSlot", eq(1, 2).negate().or(eq(18, 1)));
    safe(&mut m, "BirthEffectSurvivesParentVerdict", Var(1).eq(Var(2)));
    safe(&mut m, "HelloOnlyAfterPaidBootstrapDebt", eq(21, 0).or(all([eq(39, 0), eq(20, 1)])));
    safe(&mut m, "StartOnlyAfterPaidParentPreparation", eq(22, 0).or(all([eq(40, 0), eq(21, 1)])));
    safe(&mut m, "CustodyReleaseRequiresAbsentBirthOrActualExit", eq(18, 1).or(all([
        eq(24, 1), eq(1, 0).or(eq(23, 1)), eq(41, 0), eq(42, 0)])));
    m.responses.push(Response { name: "CancelledBootstrapSettlesOrHonestlyQuarantines".into(),
        request: eq(26, 1), response: eq(24, 1).or(eq(32, 1)) });
    m.weak_fair.push(local_fair);
    m
}

#[test]
fn bootstrap_handoff_earliest_gate_effects_and_controls_are_causal() {
    let good = bootstrap_handoff(0);
    let checked = good.check(100_000).unwrap();
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
    assert!(checked.states.iter().any(|s| s[22] == 1 && s[14] == 0 && s[15] == 0 && s[29] == 1),
        "a paid current grant must reach Start with an actual compute owner");
    assert!(checked.states.iter().any(|s| s[24] == 1 && s[23] == 1), "actual child exit must settle");
    assert!(checked.states.iter().any(|s| s[43] == 1 && s[44] == 0 && s[45] == 0
        && s[21] == 1 && s[22] == 0 && s[52] == 0 && s[23] == 1 && s[24] == 1),
        "a null-context source-free probe must emit Hello and settle by actual exit without actor proof");
    let eof = good.actions.iter().position(|action| action.name == "EOF").unwrap();
    assert!(checked.edges.iter().enumerate().any(|(from, edges)| checked.states[from][43] == 1
        && checked.states[from][21] == 1 && edges.iter().any(|edge| edge.action == eof)),
        "physical EOF must remain an enabled probe transition after Hello");
    assert!(checked.states.iter().any(|s| s[43] == 2 && s[44] == 1 && s[45] == 1
        && s[22] == 1 && s[52] == 1 && s[46] == 2 && s[51] == 7),
        "real source work requires full prepared context and both final admission attempts");
    for attempts in 1..=2 {
        for failure in 0..=1 {
            assert!(checked.states.iter().any(|s| s[46] == attempts && s[47] == 3 && s[22] == 0
                && s[26] == 1 && if failure == 0 { s[48] == 0 } else { s[49] == 2 }),
                "unknown clocks and control CPU>1ms must refuse independently at permit and Start");
        }
    }
    for birth in 0..=2 {
        assert!(checked.states.iter().any(|s| s[0] == 2 && s[1] == birth && s[2] == birth),
            "all typed birth effects must be retained before parent verdict");
    }
    assert!(checked.states.iter().any(|s| s[0] == 2 && s[1] == 1 && s[6] == 0 && s[7] == 2 && s[26] == 1),
        "a delayed native birth from A1 must retain its original incarnation after B2/A3");
    assert!(checked.states.iter().any(|s| s[1] == 2 && s[32] == 1 && s[18] == 1 && s[38] == 1),
        "unknown birth must quarantine with its slot while independent control progresses");
    assert!(checked.states.iter().any(|s| s[4] == 1 && s[5] == 1 && s[21] == 0),
        "a partial one-shot permit cannot bootstrap or emit Hello");
    for span in 0..3 {
        assert!(checked.states.iter().any(|s| s[0] == 5 && s[28] == span && s[27] == 0 && s[26] == 1),
            "lease revocation must remain reachable before every span, including after rest");
    }
    for (mode, property) in [
        (1, "EveryControlledBootstrapSpanRequiresCurrentPaidPermit"),
        (2, "ParentAndChildNeverOverlapExpensiveSpans"),
        (3, "GrantRequiresPaidParentDebt"), (4, "GrantRequiresKnownParentCost"),
        (5, "RestartNeverResetsUnpaidDebt"), (6, "ValidGrantIsBoundToIncarnationPipeAndNonce"),
        (7, "SinglePermitAttempt"), (8, "EveryControlledBootstrapSpanRequiresCurrentPaidPermit"),
        (9, "UnknownBirthRetainsItsSlot"), (10, "SingleChildAttemptNoUnknownEffectRetry"),
        (11, "HelloOnlyAfterPaidBootstrapDebt"), (12, "StartOnlyAfterPaidParentPreparation"),
        (13, "ProbePermanentlyForbidsStartSourceAndActorProof"),
        (14, "RealWorkNeverFallsBackFromNullContext"),
        (15, "FinalParentAdmissionIsKnownOneMsControl"),
        (16, "FinalParentAdmissionHasNoPayloadOrNativeWork"),
        (17, "FinalParentAdmissionHasAtMostTwoAttempts"),
    ] {
        let bad = bootstrap_handoff(mode).check(100_000).unwrap();
        assert!(bad.violations.iter().any(|v| v.property == property),
            "bootstrap control{mode} must violate its own causal property {property}");
        if mode == 1 {
            for class in 1..=3 {
                assert!(bad.states.iter().any(|s| s[37] == class && s[33] == 0),
                    "argv/channel/Builder gates must each be bypassed by the control");
            }
            assert!(bad.states.iter().any(|s| s[37] == 3 && s[29] == 1 && s[4] == 0),
                "a mistimed Builder still leaves an actual owned compute effect");
        }
        if mode == 2 {
            for phase in [6, 17] {
                assert!(bad.states.iter().any(|s| s[0] == phase && s[30] == 1 && s[31] == 1),
                    "parent overlap must execute during both bootstrap and admitted child work");
            }
        }
        if mode == 5 {
            assert!(bad.states.iter().any(|s| s[36] == 1 && s[14] == 0 && s[15] == 0),
                "the restart control must actually erase an unpaid cost, not only label one");
        }
        if mode == 6 {
            assert!(bad.states.iter().any(|s| s[7] == 2 && s[8] == 0 && s[9] == 0
                && s[10] == 0 && s[31] == 1 && s[33] == 0),
                "the stale grant must actually start a controlled span using A1 pipe/nonce at A3");
        }
        if mode == 10 {
            assert!(bad.states.iter().any(|s| s[1] == 2 && s[3] == 2 && s[41] == 1),
                "unknown-effect retry must actually re-enter native birth while retaining old uncertainty");
        }
        if mode == 13 {
            assert!(bad.states.iter().any(|s| s[43] == 1 && s[44] == 0 && s[45] == 0
                && s[22] == 1 && s[52] == 1 && s[6] == s[7] && s[9] == s[11] && s[10] == s[12]),
                "matching incarnation/pipe/nonce must not authorize probe Start or source fallback");
        }
        if mode == 14 {
            assert!(bad.states.iter().any(|s| s[43] == 2 && s[44] == 0 && s[45] == 0 && s[4] == 2),
                "the null-work control must actually issue a grant without inventing actor proof");
        }
        if mode == 15 {
            for failure in 0..=1 {
                assert!(bad.states.iter().any(|s| (s[47] == 1 || s[47] == 2)
                    && if failure == 0 { s[48] == 0 } else { s[49] == 2 }),
                    "the control must actually approve unknown clock and CPU>1ms outcomes");
            }
        }
        if mode == 16 {
            assert!(bad.states.iter().any(|s| s[50] == 1 && s[49] == 1 && s[47] == 1),
                "payload/native work remains forbidden even when hidden within a fast control callback");
        }
        if mode == 8 {
            for span in 0..3 {
                assert!(bad.states.iter().any(|s| s[28] == span && s[27] == 0 && s[33] == 0),
                    "revoked-lease control must execute every bootstrap span");
            }
        }
    }
}

// One child owns phases exec(0), entry(1), entry_emitted(2), waiting_emitted(3),
// permit_read(4), Hello(5), refusal(6). Counts0..2, owned allocation0..1,
// actual/observed CPU0..2 (1 is the unchanged budget) start at zero. Sampling,
// formatting/write, waiting, observation and permit read are separate atomic
// actions. WF covers only local waiting/observation/refusal, never OS entry,
// syscall completion, parent permit delivery, clocks or heap implementation.
pub fn bootstrap_entry(control: u8) -> Model {
    let name = match control {
        1 => "DreamC1BootstrapEntryLate",
        2 => "DreamC1BootstrapEntryAllocation",
        3 => "DreamC1BootstrapEntryDuplicate",
        4 => "DreamC1BootstrapEntryNoObserve",
        _ => "DreamC1BootstrapEntry",
    };
    let mut m = base(name, control != 0, &[
        ("phase", 6, 0), ("entries", 2, 0), ("waiting", 1, 0),
        ("permitReads", 1, 0), ("formatterOwnedAlloc", 1, 0),
        ("recordCpu", 2, 0), ("observedCpu", 2, 0), ("observed", 1, 0),
        ("permitReadsAtEntry", 1, 0),
    ]);
    act(&mut m, "EarlyEntry", eq(0, 0), &[(0, Int(1))]);
    for cpu in 1..=2 {
        act(&mut m, &format!("EmitEntryCpu{cpu}"), eq(0, 1), &[
            (0, Int(2)), (1, Int(1)), (5, Int(cpu)),
            (4, Int(if control == 2 { 1 } else { 0 })), (8, Var(3)),
        ]);
    }
    let waiting = m.actions.len();
    act(&mut m, "EmitUnchangedWaiting", eq(0, 2), &[(0, Int(3)), (2, Int(1))]);
    let observe = m.actions.len();
    act(&mut m, "ObserveWholePrelude", all([eq(0, 3), eq(7, 0)]), &[
        (6, if control == 4 { Int(0) } else { Var(5) }), (7, Int(1)),
    ]);
    let refuse = m.actions.len();
    act(&mut m, "RefuseOverBudget", all([eq(0, 3), eq(7, 1), Int(1).lt(Var(6))]), &[(0, Int(6))]);
    act(&mut m, "PermitRead", all([eq(0, 3), eq(7, 1), Var(6).le(Int(1))]), &[(0, Int(4)), (3, Int(1))]);
    act(&mut m, "PaidHelloAbstracted", eq(0, 4), &[(0, Int(5))]);
    if control == 1 {
        act(&mut m, "WaitingWithoutEntry", eq(0, 1), &[
            (0, Int(3)), (2, Int(1)), (5, Int(1)),
        ]);
        act(&mut m, "EmitEntryAfterPermitRead", all([eq(0, 4), eq(1, 0)]), &[
            (1, Int(1)), (8, Var(3)),
        ]);
    }
    if control == 3 {
        act(&mut m, "EmitEntryTwice", eq(0, 2), &[(1, Int(2))]);
    }
    safe(&mut m, "AtMostOneEntry", Var(1).le(Int(1)));
    safe(&mut m, "EntryBeforeWaitingBeforePermitRead", all([
        eq(8, 0), eq(2, 0).or(eq(1, 1)), eq(3, 0).or(eq(2, 1)),
    ]));
    safe(&mut m, "EntryFormatterOwnsNoAllocation", eq(4, 0));
    safe(&mut m, "FullRecordCpuInsidePrepermitUsage", eq(7, 0).or(Var(6).eq(Var(5))));
    progress(&mut m, "EmittedEntryLocallyObserved", eq(0, 2), eq(7, 1).or(eq(0, 6)), vec![waiting, observe, refuse]);
    m
}
#[test]
fn bootstrap_entry_controls_reach_the_intended_violation() {
    let good = bootstrap_entry(0).check(100_000).unwrap();
    assert!(good.violations.is_empty(), "{:?}", good.violations);
    assert!(good.states.iter().any(|s| s[0] == 5 && s[1] == 1 && s[6] == 1));
    assert!(good.states.iter().any(|s| s[0] == 6 && s[5] == 2 && s[6] == 2));
    for (mode, property) in [(1, "EntryBeforeWaitingBeforePermitRead"),
        (2, "EntryFormatterOwnsNoAllocation"), (3, "AtMostOneEntry"),
        (4, "FullRecordCpuInsidePrepermitUsage")] {
        let bad = bootstrap_entry(mode).check(100_000).unwrap();
        assert!(bad.violations.iter().any(|v| v.property == property), "control {mode}: {:?}", bad.violations);
        if mode == 1 {
            assert!(bad.states.iter().any(|s| s[0] == 4 && s[1] == 1 && s[3] == 1 && s[8] == 1), "late control must actually emit after a permit read");
        }
    }
}
