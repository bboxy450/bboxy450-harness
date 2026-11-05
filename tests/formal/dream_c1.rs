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
