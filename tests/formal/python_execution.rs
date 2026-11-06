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
            ("after_retirement_dispatch", 1, 0),
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
        "CompleteWithoutEffects",
        all([eq(0, 2), eq(3, 0), eq(11, 0), Var(2).eq(Var(1))]),
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
        act(
            &mut m,
            "BrokenDispatchAfterTerminal",
            all([eq(0, 3).or(eq(0, 4)), eq(3, 0)]),
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
    safe(&mut m, "NoNewEffectAfterRetirement", eq(12, 0));
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

/// One durable execution and one bounded request. Two logical ticks abstract
/// the 25s MCP wait ceiling; request completion is atomic with the cap tick.
/// Worker completion may arrive twice or after cancellation/the request reply.
pub(super) fn bounded_wait(control: u8) -> Model {
    let suffix = ["PythonWait", "PythonWaitCap", "PythonWaitDuplicate"][control as usize];
    let mut m = base::<6>(
        suffix,
        control > 0,
        &[
            ("phase", 2, 0), // running, completed, cancelled/deadline
            ("settlements", 2, 0),
            ("completions", 2, 0),
            ("wait", 2, 0), // absent, waiting, replied
            ("elapsed", 3, 0),
            ("reply", 2, 0), // running handle, completed, cancelled
        ],
    );
    act(
        &mut m,
        "BeginRunningWait",
        all([eq(0, 0), eq(3, 0)]),
        &[(3, Int(1))],
    );
    for phase in [1, 2] {
        act(
            &mut m,
            "ReadRetainedTerminal",
            all([eq(0, phase), eq(3, 0)]),
            &[(3, Int(2)), (5, Int(phase))],
        );
    }
    let tick = m.actions.len();
    act(
        &mut m,
        "WaitTick",
        all([eq(3, 1), eq(4, 0)]),
        &[(4, Int(1))],
    );
    let cap = m.actions.len();
    act(
        &mut m,
        "ReplyAtWaitCap",
        all([eq(3, 1), eq(4, 1)]),
        &[(3, Int(2)), (4, Int(2)), (5, Var(0))],
    );
    let mut terminal = Vec::new();
    for phase in [1, 2] {
        for waiting in [false, true] {
            terminal.push(m.actions.len());
            let mut updates = vec![(0, Int(phase)), (1, Int(1))];
            if phase == 1 {
                updates.push((2, Int(1)));
            }
            if waiting {
                updates.extend([(3, Int(2)), (5, Int(phase))]);
            }
            act(
                &mut m,
                if phase == 1 {
                    "CompleteAndWakeWaiter"
                } else {
                    "CancelOrDeadlineAndWakeWaiter"
                },
                all([eq(0, 0), if waiting { eq(3, 1) } else { eq(3, 1).negate() }]),
                &updates,
            );
        }
    }
    let mut late = vec![(2, Var(2).plus(Int(1)))];
    if control == 2 {
        late.push((1, Var(1).plus(Int(1))));
    }
    act(
        &mut m,
        "DuplicateOrLateWorkerCompletion",
        all([eq(0, 0).negate(), Var(2).lt(Int(2)), Var(1).lt(Int(2))]),
        &late,
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenKeepWaitingAtCap",
            all([eq(3, 1), eq(4, 1)]),
            &[(4, Int(2))],
        );
        act(
            &mut m,
            "BrokenWaitExceedsCap",
            all([eq(3, 1), eq(4, 2)]),
            &[(4, Int(3))],
        );
    }
    safe(&mut m, "WaitNeverExceedsCap", Var(4).le(Int(2)));
    safe(
        &mut m,
        "CapReturnsDurableHandle",
        eq(4, 2).negate().or(eq(3, 2)),
    );
    safe(&mut m, "TerminalSettlesOnce", Var(1).le(Int(1)));
    safe(
        &mut m,
        "TerminalWakesWaiter",
        eq(0, 0).or(eq(3, 1).negate()),
    );
    progress(
        &mut m,
        "WaitAlwaysReplies",
        eq(3, 1),
        eq(3, 2),
        vec![tick, cap],
    );
    progress(
        &mut m,
        "ExecutionEventuallySettles",
        eq(0, 0),
        eq(0, 1).or(eq(0, 2)),
        terminal,
    );
    m
}

/// One live connection, three catalogue revisions whose public tool sets are
/// A-B-A. Exact revision identity fences delayed refresh results; catalogue
/// changes do not turn an already dispatched unknown effect into a retry.
pub(super) fn catalogue(control: u8) -> Model {
    let suffix = [
        "PythonCatalogue",
        "PythonCatalogueAba",
        "PythonCatalogueReplay",
    ][control as usize];
    let mut m = base::<7>(
        suffix,
        control > 0,
        &[
            ("catalogue_epoch", 2, 0),
            ("cached_epoch", 2, 0),
            ("dirty", 1, 1),
            ("refreshing", 1, 0),
            ("refresh_epoch", 2, 0),
            ("dispatches", 2, 0),
            ("unknown", 1, 0),
        ],
    );
    act(
        &mut m,
        "ListChanged",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (2, Int(1))],
    );
    let refresh = m.actions.len();
    act(
        &mut m,
        "RefreshOnNotificationOrUnsentMiss",
        all([eq(2, 1), eq(3, 0)]),
        &[(3, Int(1)), (4, Var(0))],
    );
    let publish = m.actions.len();
    act(
        &mut m,
        "PublishCurrentCatalogue",
        all([eq(3, 1), Var(4).eq(Var(0))]),
        &[(1, Var(4)), (2, Int(0)), (3, Int(0))],
    );
    let stale = m.actions.len();
    act(
        &mut m,
        "DiscardLateCatalogue",
        all([eq(3, 1), Var(4).eq(Var(0)).negate()]),
        &[(2, Int(1)), (3, Int(0))],
    );
    act(
        &mut m,
        "DispatchUsingRefreshedCatalogue",
        all([eq(2, 0), Var(1).eq(Var(0)), eq(5, 0)]),
        &[(5, Int(1))],
    );
    act(
        &mut m,
        "NestedTimeoutEffectUnknown",
        all([eq(5, 1), eq(6, 0)]),
        &[(6, Int(1))],
    );
    if control == 1 {
        act(
            &mut m,
            "BrokenPublishSamePublicShapeAfterABA",
            all([eq(3, 1), eq(4, 0), eq(0, 2)]),
            &[(1, Var(4)), (2, Int(0)), (3, Int(0))],
        );
    }
    if control == 2 {
        act(
            &mut m,
            "BrokenReplayTimedOutEffect",
            all([eq(5, 1), eq(6, 1)]),
            &[(5, Int(2))],
        );
    }
    safe(
        &mut m,
        "CleanCatalogueHasCurrentRevision",
        eq(2, 1).or(Var(1).eq(Var(0))),
    );
    safe(&mut m, "UnknownEffectNeverReplayed", Var(5).le(Int(1)));
    progress(
        &mut m,
        "CatalogueEventuallyRefreshes",
        eq(2, 1),
        eq(2, 0),
        vec![refresh, publish, stale],
    );
    m
}
