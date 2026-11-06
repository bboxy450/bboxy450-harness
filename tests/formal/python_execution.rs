// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! One execution, one nested effect and three identity epochs (including A-B-A).
use super::*;

pub(super) fn model(control: u8) -> Model {
    let suffix = [
        "PythonExecution",
        "PythonExecutionReplay",
        "PythonExecutionStale",
        "PythonExecutionCustody",
        "PythonExecutionCancel",
        "PythonExecutionAdmission",
        "PythonExecutionRetarget",
    ][control as usize];
    let mut m = base::<15>(
        suffix,
        control > 0,
        &[
            ("phase", 4, 0),
            ("epoch", 2, 0),
            ("captured", 2, 0),
            ("dispatches", 2, 0),
            ("receipt", 1, 0),
            ("visible", 1, 0),
            ("published_epoch", 2, 0),
            ("owned", 1, 0),
            ("pid_reused", 1, 0),
            ("foreign_kill", 1, 0),
            ("temporary", 1, 0),
            ("cancelled", 1, 0),
            ("after_cancel_dispatch", 1, 0),
            ("complete_admission", 1, 0),
            ("cross_scope_effect", 1, 0),
        ],
    );
    act(
        &mut m,
        "StageAdmission",
        all([eq(0, 0), eq(10, 0)]),
        &[(10, Int(1))],
    );
    act(
        &mut m,
        "InterruptedTemporary",
        all([eq(0, 0), eq(10, 1)]),
        &[(10, Int(0))],
    );
    act(
        &mut m,
        "PublishCompleteAdmission",
        all([eq(0, 0), eq(10, 1)]),
        &[(0, Int(1)), (2, Var(1)), (10, Int(0)), (13, Int(1))],
    );
    act(
        &mut m,
        "StartOwnedChild",
        all([eq(0, 1), eq(11, 0), Var(2).eq(Var(1))]),
        &[(0, Int(2)), (7, Int(1))],
    );
    act(
        &mut m,
        "NestedDispatch",
        all([eq(0, 2), eq(3, 0), eq(11, 0), Var(2).eq(Var(1))]),
        &[(3, Int(1))],
    );
    act(
        &mut m,
        "RecordEffectResult",
        all([eq(0, 2), eq(3, 1), eq(4, 0)]),
        &[(4, Int(1))],
    );
    let publish = m.actions.len();
    act(
        &mut m,
        "PublishResult",
        all([
            eq(0, 2),
            eq(4, 1),
            eq(11, 0),
            if control == 2 {
                Bool(true)
            } else {
                Var(2).eq(Var(1))
            },
        ]),
        &[
            (0, Int(3)),
            (5, Int(1)),
            (6, Var(2)),
            (7, Int(0)),
            (8, Int(1)),
        ],
    );
    act(
        &mut m,
        "AdvanceEpoch",
        Var(1).lt(Int(2)),
        &[(1, Var(1).plus(Int(1))), (5, Int(0))],
    );
    act(
        &mut m,
        "Cancel",
        all([eq(0, 1).or(eq(0, 2)), eq(11, 0)]),
        &[(11, Int(1))],
    );
    act(
        &mut m,
        "RetireExactChild",
        all([eq(0, 2), eq(11, 1), eq(7, 1)]),
        &[(0, Int(4)), (7, Int(0)), (8, Int(1))],
    );
    let deadline = m.actions.len();
    act(
        &mut m,
        "DeadlineOrOwnerLoss",
        eq(0, 1).or(eq(0, 2)),
        &[(0, Int(4)), (7, Int(0)), (8, Int(1))],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenRetryRedispatch",
            all([eq(3, 1), eq(0, 3).or(eq(0, 4))]),
            &[(3, Int(2))],
        );
    }
    if control == 3 {
        act(
            &mut m,
            "BrokenKillReusedPid",
            all([eq(7, 0), eq(8, 1)]),
            &[(9, Int(1))],
        );
    }
    if control == 4 {
        act(
            &mut m,
            "BrokenDispatchAfterCancel",
            all([eq(0, 2), eq(11, 1), eq(3, 0)]),
            &[(3, Int(1)), (12, Int(1))],
        );
    }
    if control == 5 {
        act(
            &mut m,
            "BrokenStartPartialAdmission",
            all([eq(0, 0), eq(10, 1)]),
            &[(0, Int(2)), (7, Int(1))],
        );
    }
    if control == 6 {
        act(
            &mut m,
            "BrokenRetargetAfterPreflight",
            all([eq(0, 2), eq(3, 0), eq(11, 0), Var(2).eq(Var(1)).negate()]),
            &[(3, Int(1)), (14, Int(1))],
        );
    }
    safe(&mut m, "NoRepeatedEffect", Var(3).le(Int(1)));
    safe(&mut m, "CurrentResultOnly", eq(5, 0).or(Var(6).eq(Var(1))));
    safe(&mut m, "NeverKillReusedIdentity", eq(9, 0));
    safe(&mut m, "NoNewEffectAfterCancel", eq(12, 0));
    safe(&mut m, "NoPartialAdmissionExecutes", eq(0, 0).or(eq(13, 1)));
    safe(&mut m, "NativeEffectKeepsCapturedScope", eq(14, 0));
    progress(
        &mut m,
        "EventuallySettled",
        eq(0, 1).or(eq(0, 2)),
        eq(0, 3).or(eq(0, 4)),
        vec![publish, deadline],
    );
    m
}

/// Two owners, two host-work slots; a stalled owner's quota is one. Control
/// admission is separate. The old shared connection budget can starve both.
pub(super) fn capacity(broken: bool) -> Model {
    let mut m = base::<5>(
        "PythonControlCapacity",
        broken,
        &[
            ("a_active", 2, 0),
            ("b_requested", 1, 0),
            ("b_done", 1, 0),
            ("cancel_requested", 1, 0),
            ("cancel_done", 1, 0),
        ],
    );
    act(
        &mut m,
        "StallA",
        all([eq(4, 0), Var(0).lt(Int(if broken { 2 } else { 1 }))]),
        &[(0, Var(0).plus(Int(1)))],
    );
    act(&mut m, "RequestIndependentB", eq(1, 0), &[(1, Int(1))]);
    let other = m.actions.len();
    act(
        &mut m,
        "ServeIndependentB",
        all([eq(1, 1), eq(2, 0), Var(0).lt(Int(2))]),
        &[(2, Int(1))],
    );
    act(&mut m, "RequestCancelA", eq(3, 0), &[(3, Int(1))]);
    let control = m.actions.len();
    act(
        &mut m,
        "ReplyAndCancelOwnedA",
        all([
            eq(3, 1),
            eq(4, 0),
            if broken {
                Var(0).lt(Int(2))
            } else {
                Bool(true)
            },
        ]),
        &[(4, Int(1)), (0, Int(0))],
    );
    progress(
        &mut m,
        "IndependentRunRemainsResponsive",
        eq(1, 1),
        eq(2, 1),
        vec![other],
    );
    progress(
        &mut m,
        "ControlRemainsResponsive",
        eq(3, 1),
        eq(4, 1),
        vec![control],
    );
    m
}
