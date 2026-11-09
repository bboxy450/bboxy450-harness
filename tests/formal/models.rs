// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Adapted from bboxy450-term contract helpers and shapes: LifecycleOwnership, DocumentFeed,
// AsyncRenderPreparation, NativeInputOwnership, NotificationReplace, VolatileBusSubscription.
// https://github.com/bboxy450/bboxy450-term/blob/main/crates/bboxy450-term/tests/formal_contracts/models.rs
use Expr::{Bool, Int, Var};
use bbt_tlc::{Action, Expr, Invariant, Model, Response, Variable};
#[path = "python_execution.rs"]
mod python_execution;
fn eq(v: usize, n: i64) -> Expr {
    Var(v).eq(Int(n))
}
fn all(xs: impl IntoIterator<Item = Expr>) -> Expr {
    xs.into_iter().fold(Bool(true), Expr::and)
}
fn base<const N: usize>(name: &str, b: bool, vars: &[(&str, i64, i64); N]) -> Model {
    let mut m = Model::new(
        &format!("{}Harness{name}", if b { "Broken" } else { "" }),
        vars.iter()
            .map(|(n, max, init)| Variable::new(n, 0, *max, *init))
            .collect(),
    );
    m.check_deadlock = false;
    m
}
fn act(m: &mut Model, n: &str, g: Expr, u: &[(usize, Expr)]) {
    m.actions.push(Action::new(n, g, u.to_vec()));
}
fn safe(m: &mut Model, n: &str, c: Expr) {
    m.invariants.push(Invariant {
        name: n.into(),
        condition: c,
    });
}
fn progress(m: &mut Model, n: &str, r: Expr, s: Expr, fair: Vec<usize>) {
    m.responses.push(Response {
        name: n.into(),
        request: r,
        response: s,
    });
    m.weak_fair.push(fair);
}
// Adapted from bboxy450-term PluginHostApi and OperationRecovery: one
// terminal outcome, paired synthetic results, and replay without redispatch.
fn lifecycle_mode(b: bool, pair_control: bool) -> Model {
    let mut m = base::<11>(
        if pair_control {
            "TurnLifecyclePairing"
        } else {
            "TurnLifecycle"
        },
        b,
        &[
            ("phase_a", 4, 0),
            ("ends_a", 2, 0),
            ("round_a", 2, 0),
            ("phase_b", 4, 0),
            ("ends_b", 2, 0),
            ("round_b", 2, 0),
            ("result_a1", 1, 1),
            ("result_a2", 1, 1),
            ("result_b1", 1, 1),
            ("result_b2", 1, 1),
            ("recovered", 1, 0),
        ],
    );
    for (t, c) in [(0, 6), (3, 8)] {
        act(&mut m, "StartQueued", eq(t, 0), &[(t, Int(1))]);
        act(
            &mut m,
            "ToolPending",
            all([eq(t, 1), Var(t + 2).lt(Int(2))]),
            &[(t, Int(2)), (c, Int(0)), (c + 1, Int(0))],
        );
        act(&mut m, "ResultOne", eq(t, 2), &[(c, Int(1))]);
        act(&mut m, "ResultTwo", eq(t, 2), &[(c + 1, Int(1))]);
        act(
            &mut m,
            "ToolResult",
            all([eq(t, 2), eq(c, 1), eq(c + 1, 1)]),
            &[(t, Int(1)), (t + 2, Var(t + 2).plus(Int(1)))],
        );
        let finish = m.actions.len();
        act(
            &mut m,
            "FinishCancelOrError",
            Var(t).lt(Int(3)),
            &[
                (t, Int(3)),
                (c, Int(1)),
                (
                    c + 1,
                    if b && pair_control {
                        Var(c + 1)
                    } else {
                        Int(1)
                    },
                ),
            ],
        );
        let end = m.actions.len();
        act(
            &mut m,
            "EmitOneEnding",
            eq(t, 3),
            &[(t, Int(4)), (t + 1, Var(t + 1).plus(Int(1)))],
        );
        act(
            &mut m,
            "CrashRecoveryPairsWithoutRedispatch",
            Var(t).lt(Int(3)),
            &[(10, Int(1)), (t, Int(3)), (c, Int(1)), (c + 1, Int(1))],
        );
        act(&mut m, "ReplayIsReadOnly", eq(t, 4), &[]);
        if b && !pair_control {
            act(
                &mut m,
                "LateCompletionSecondEnding",
                all([eq(t, 4), eq(t + 1, 1)]),
                &[(t + 1, Int(2))],
            );
        }
        safe(
            &mut m,
            "ExactlyOneEnding",
            Var(t + 1)
                .le(Int(1))
                .and(eq(t, 4).negate().or(eq(t + 1, 1))),
        );
        safe(
            &mut m,
            "PairedAtEnd",
            eq(t, 4).negate().or(all([eq(c, 1), eq(c + 1, 1)])),
        );
        progress(
            &mut m,
            "EveryQueuedTurnEnds",
            Bool(true),
            eq(t, 4),
            vec![finish, end],
        );
    }
    m
}
fn cancel_tree(b: bool) -> Model {
    // A completion captures a birth, independently of the reusable display identity.
    let mut m = base::<7>(
        "CancelTree",
        b,
        &[
            ("generation", 2, 0),
            ("identity", 1, 0),
            ("held_generation", 2, 0),
            ("held", 1, 0),
            ("live_child", 1, 1),
            ("phase", 2, 0),
            ("bad", 2, 0),
        ],
    );
    act(
        &mut m,
        "CaptureCompletion",
        all([eq(3, 0), eq(5, 0)]),
        &[(3, Int(1)), (2, Var(0))],
    );
    act(&mut m, "CancelOldBirth", eq(5, 0), &[(5, Int(1))]);
    act(&mut m, "KillAndJoin", eq(5, 1), &[(4, Int(0)), (5, Int(2))]);
    act(
        &mut m,
        "NextBirth",
        all([eq(5, 2), Var(0).lt(Int(2))]),
        &[
            (0, Var(0).plus(Int(1))),
            (1, Int(1).minus(Var(1))),
            (5, Int(0)),
            (4, Int(1)),
        ],
    );
    act(
        &mut m,
        "HeldCompletion",
        all([
            eq(3, 1),
            eq(5, 0),
            if b { Bool(true) } else { Var(2).eq(Var(0)) },
        ]),
        &[(3, Int(0)), (6, Var(0).minus(Var(2)))],
    );
    act(
        &mut m,
        "RetireHeld",
        all([eq(3, 1), eq(5, 0).negate().or(Var(2).eq(Var(0)).negate())]),
        &[(3, Int(0))],
    );
    safe(&mut m, "NoStaleCompletion", eq(6, 0));
    safe(&mut m, "NoChildAfterEnd", eq(5, 2).negate().or(eq(4, 0)));
    m
}
fn cancel_group(b: bool) -> Model {
    let mut m = cancel_tree(false);
    m.name = format!("{}HarnessCancelTreeGroup", if b { "Broken" } else { "" });
    if b {
        m.actions
            .iter_mut()
            .find(|a| a.name == "KillAndJoin")
            .unwrap()
            .updates[0]
            .1 = Var(4);
    }
    m
}
fn steer(b: bool) -> Model {
    steer_mode(b, false)
}
fn steer_mode(block_control: bool, duplicate: bool) -> Model {
    let mut m = base::<8>(
        "SteerQueue",
        block_control,
        &[
            ("work_inbox", 64, 64),
            ("pending_interrupt", 1, 1),
            ("admitted", 1, 0),
            ("consumed", 1, 0),
            ("running", 1, 1),
            ("delivered", 2, 0),
            ("queued", 2, 0),
            ("acked", 1, 1),
        ],
    );
    let admit = m.actions.len();
    act(
        &mut m,
        "AdmitControl",
        all([
            eq(1, 1),
            eq(2, 0),
            if block_control {
                Var(0).lt(Int(64))
            } else {
                Bool(true)
            },
        ]),
        &[(2, Int(1))],
    );
    let consume = m.actions.len();
    act(
        &mut m,
        "ConsumeControl",
        all([eq(2, 1), eq(3, 0)]),
        &[(3, Int(1)), (1, Int(0))],
    );
    let deliver = m.actions.len();
    act(
        &mut m,
        "DeliverSteer",
        all([eq(7, 1), eq(4, 1), eq(5, 0), eq(6, 0)]),
        &[(5, Int(1))],
    );
    act(&mut m, "TurnEndsBeforeDelivery", eq(4, 1), &[(4, Int(0))]);
    let queue = m.actions.len();
    act(
        &mut m,
        "ConvertSteerToQueuedTurn",
        all([
            eq(7, 1),
            eq(4, 0),
            eq(6, 0),
            if duplicate { Bool(true) } else { eq(5, 0) },
        ]),
        &[(6, Int(1))],
    );
    safe(
        &mut m,
        "SteerAppearsAtMostOnce",
        Var(5).plus(Var(6)).le(Var(7)),
    );
    progress(
        &mut m,
        "AckedSteerEventuallyAppears",
        eq(7, 1),
        Var(5).plus(Var(6)).eq(Int(1)),
        vec![deliver, queue],
    );
    progress(
        &mut m,
        "InterruptAlwaysProgresses",
        Bool(true),
        eq(3, 1),
        vec![admit, consume],
    );
    if duplicate {
        m.name = "BrokenHarnessSteerQueueDelivery".into();
        m.responses.clear();
        m.weak_fair.clear();
    }
    m
}
fn control_bound(b: bool) -> Model {
    // One slot per outstanding interrupt (64 queued turns + running), approval
    // (64 tools), and Close. Consumers return the same type's slot.
    let mut m = base::<9>(
        "ControlLaneBound",
        b,
        &[
            ("interrupt_slots", 65, 65),
            ("interrupt_pending", 257, 0),
            ("approval_slots", 64, 64),
            ("approval_pending", 64, 0),
            ("close_slots", 1, 1),
            ("close_pending", 1, 0),
            ("control_depth", 257, 0),
            ("work_inbox", 64, 64),
            ("duplicate_producer", 1, 0),
        ],
    );
    for (slot, pending, bound, label) in [
        (0, 1, 65, "Interrupt"),
        (2, 3, 64, "Approval"),
        (4, 5, 1, "Close"),
    ] {
        act(
            &mut m,
            &format!("Produce{label}"),
            all([
                eq(8, 0),
                Var(slot).eq(Int(0)).negate(),
                Var(pending).lt(Int(bound)),
            ]),
            &[
                (slot, Var(slot).minus(Int(1))),
                (pending, Var(pending).plus(Int(1))),
                (6, Var(6).plus(Int(1))),
            ],
        );
        act(
            &mut m,
            &format!("Consume{label}"),
            all([
                eq(8, 0),
                Var(pending).eq(Int(0)).negate(),
                Var(slot).lt(Int(bound)),
            ]),
            &[
                (slot, Var(slot).plus(Int(1))),
                (pending, Var(pending).minus(Int(1))),
                (6, Var(6).minus(Int(1))),
            ],
        );
        safe(
            &mut m,
            &format!("{label}SlotsConserved"),
            Var(slot).plus(Var(pending)).eq(Int(bound)),
        );
    }
    // A producer that enqueues without owning a type slot duplicates an op.
    // At saturation, a producer that loses its reservation floods duplicates.
    // Freeze consumers for this finite worst-case schedule, not by fairness.
    if b {
        act(
            &mut m,
            "LoseProducerReservation",
            all([eq(8, 0), eq(1, 65), eq(3, 64), eq(5, 1)]),
            &[(8, Int(1))],
        );
        act(
            &mut m,
            "DuplicateInterruptWithoutSlot",
            all([eq(8, 1), Var(6).lt(Int(257))]),
            &[(1, Var(1).plus(Int(1))), (6, Var(6).plus(Int(1)))],
        );
    }
    safe(
        &mut m,
        "DepthMatchesOutstanding",
        Var(6).eq(Var(1).plus(Var(3)).plus(Var(5))),
    );
    safe(&mut m, "ControlLaneBounded", Var(6).le(Int(256)));
    safe(&mut m, "LegitimateOutstandingFits", Var(6).le(Int(130)));
    m
}
// Adapted from bboxy450-term AttentionLifecycle, PluginHookGate and HarnessToken.
fn approval(b: bool, unattended: bool) -> Model {
    let mut m = base::<5>(
        if unattended {
            "ApprovalUnattended"
        } else {
            "ApprovalLifecycle"
        },
        b,
        &[
            ("approval", 3, 0),
            ("generation", 2, 0),
            ("answer_generation", 2, 0),
            ("effect", 1, 0),
            ("resolutions", 2, 0),
        ],
    );
    act(
        &mut m,
        "ReplaceBirthAtoBtoA",
        all([Var(1).lt(Int(2)), eq(3, 0)]),
        &[(1, Var(1).plus(Int(1)))],
    );
    if !unattended {
        for (n, decision) in [("FaceOneAllow", 1), ("FaceTwoDeny", 2), ("Cancel", 3)] {
            act(&mut m, n, eq(0, 0), &[(0, Int(decision)), (4, Int(1))]);
        }
        act(
            &mut m,
            "CurrentApprovedEffect",
            all([
                eq(0, 1),
                eq(3, 0),
                if b { Bool(true) } else { Var(1).eq(Var(2)) },
            ]),
            &[(3, Int(1))],
        );
        act(&mut m, "ReplayNeverReasks", eq(0, 0).negate(), &[]);
        safe(&mut m, "AnswerOnlyCurrent", eq(3, 0).or(Var(1).eq(Var(2))));
        safe(&mut m, "NoEffectWithoutAllow", eq(3, 0).or(eq(0, 1)));
        progress(
            &mut m,
            "AskedEventuallyResolved",
            eq(0, 0),
            eq(0, 0).negate(),
            vec![1, 2, 3],
        );
    } else {
        let resolve = m.actions.len();
        act(
            &mut m,
            "UnattendedDeny",
            if b { Bool(false) } else { eq(0, 0) },
            &[(0, Int(2)), (4, Int(1))],
        );
        act(&mut m, "OtherSessionContinues", Bool(true), &[]);
        progress(
            &mut m,
            "UnattendedEventuallyResolves",
            eq(0, 0),
            eq(0, 0).negate(),
            vec![resolve],
        );
    }
    safe(&mut m, "ExactlyOneResolver", Var(4).le(Int(1)));
    m
}
fn replay_base(b: bool) -> Model {
    // Read head=1; one concurrent append can race registration. Subscriber
    // buffer=1; loss closes with lag and replay repairs the cursor.
    let mut m = base::<9>(
        "SubscriberReplay",
        b,
        &[
            ("head", 3, 1),
            ("registered", 1, 0),
            ("read_head", 3, 0),
            ("cursor", 3, 0),
            ("buffer", 3, 0),
            ("phase", 3, 0),
            ("lagged", 1, 0),
            ("epoch", 1, 0),
            ("reset", 1, 0),
        ],
    );
    act(
        &mut m,
        "RegisterBeforeRead",
        eq(5, 0),
        &[(1, if b { Int(0) } else { Int(1) }), (5, Int(1))],
    );
    act(
        &mut m,
        "ReadReplayHead",
        eq(5, 1),
        &[(2, Var(0)), (5, Int(2))],
    );
    act(
        &mut m,
        "ConcurrentAppend",
        all([eq(5, 2), Var(0).lt(Int(3))]),
        &[
            (0, Var(0).plus(Int(1))),
            (4, if b { Int(0) } else { Var(0).plus(Int(1)) }),
        ],
    );
    act(
        &mut m,
        "FinishReplayRegister",
        eq(5, 2),
        &[(3, Var(2)), (1, Int(1)), (5, Int(3))],
    );
    act(
        &mut m,
        "LiveDeliverOrLag",
        all([eq(5, 3), Var(3).lt(Var(0))]),
        &[(6, Int(1))],
    );
    act(
        &mut m,
        "ReplayAfterLag",
        eq(6, 1),
        &[(3, Var(0)), (6, Int(0)), (4, Int(0))],
    );
    act(
        &mut m,
        "KernelTailLoss",
        all([eq(7, 0), eq(5, 3)]),
        &[(7, Int(1)), (0, Int(1)), (8, Int(1))],
    );
    safe(
        &mut m,
        "NoSilentDurableGap",
        all([eq(5, 3), eq(7, 0), eq(6, 0), eq(4, 0)])
            .negate()
            .or(Var(0).eq(Var(3))),
    );
    safe(
        &mut m,
        "EpochLossRequiresReset",
        Var(3).le(Var(0)).or(eq(8, 1)),
    );
    m
}
fn replay_drop(b: bool) -> Model {
    let mut m = replay_base(false);
    m.name = format!(
        "{}HarnessSubscriberReplayDrop",
        if b { "Broken" } else { "" }
    );
    if b {
        m.actions
            .iter_mut()
            .find(|a| a.name == "ReplayAfterLag")
            .unwrap()
            .updates[0]
            .1 = Var(3);
    }
    m
}
fn replay_healthy(b: bool) -> Model {
    let mut m = base::<3>(
        "SubscriberReplayHealthy",
        b,
        &[
            ("head", 3, 0),
            ("slow_full", 1, 1),
            ("healthy_cursor", 3, 0),
        ],
    );
    act(
        &mut m,
        "AppendNeverWaitsForSlowPeer",
        Var(0).lt(Int(3)),
        &[(0, Var(0).plus(Int(1)))],
    );
    act(
        &mut m,
        "HealthyDeliver",
        all([Var(2).lt(Var(0)), if b { eq(1, 0) } else { Bool(true) }]),
        &[(2, Var(2).plus(Int(1)))],
    );
    safe(&mut m, "NoDuplicateOrFutureDelivery", Var(2).le(Var(0)));
    progress(
        &mut m,
        "HealthyPeerProgresses",
        Var(2).lt(Var(0)),
        Var(2).eq(Var(0)),
        vec![1],
    );
    m
}
// Adapted from bboxy450-term DirtyWake: https://github.com/bboxy450/bboxy450-term/blob/main/crates/bboxy450-term/tests/formal_contracts/models.rs
fn stream_to_view(b: bool) -> Model {
    let mut m = base::<4>(
        "StreamToView",
        b,
        &[
            ("bits", 1, 0),
            ("pending", 1, 0),
            ("producer_pc", 1, 0),
            ("drain_pc", 2, 0),
        ],
    );
    act(&mut m, "MarkBits", eq(2, 0), &[(0, Int(1)), (2, Int(1))]);
    act(&mut m, "PostWake", eq(2, 1), &[(1, Int(1)), (2, Int(0))]);
    act(&mut m, "TakeBits", eq(3, 0), &[(0, Int(0)), (3, Int(1))]);
    act(
        &mut m,
        "ClearPending",
        eq(3, 1),
        &[(1, Int(0)), (3, Int(2))],
    );
    act(
        &mut m,
        "RecheckEmpty",
        all([eq(3, 2), eq(0, 0)]),
        &[(3, Int(0))],
    );
    act(
        &mut m,
        "RecheckPublished",
        all([eq(3, 2), eq(0, 1)]),
        &[(3, Int(0)), (1, if b { Var(1) } else { Int(1) })],
    );
    safe(
        &mut m,
        "NoLostImmediateWake",
        all([eq(2, 0), eq(3, 0), eq(0, 1)]).negate().or(eq(1, 1)),
    );
    m
}
// Pending profile IO is a cancellable session-owned service, not work on the
// actor's admission path. A completion is only applied to its current slot.
fn profile_load(b: bool) -> Model {
    let mut m = base::<6>(
        "ProfileLoad",
        b,
        &[
            ("job", 4, 0),
            ("closed", 1, 0),
            ("generation", 2, 0),
            ("captured", 2, 0),
            ("applied", 1, 0),
            ("held", 1, 1),
        ],
    );
    act(
        &mut m,
        "Start",
        all([eq(0, 0), eq(1, 0)]),
        &[(0, Int(1)), (3, Var(2))],
    );
    act(
        &mut m,
        "Resolve",
        all([eq(0, 1), eq(5, 1)]),
        &[(0, Int(2)), (5, Int(0))],
    );
    act(&mut m, "Close", eq(1, 0), &[(1, Int(1)), (0, Int(3))]);
    act(&mut m, "Join", eq(0, 3), &[(0, Int(4))]);
    act(
        &mut m,
        "LateCompletion",
        all([eq(0, 3), eq(5, 1)]),
        &[(0, Int(2)), (5, Int(0))],
    );
    act(
        &mut m,
        "ReplaceBirth",
        all([Var(2).lt(Int(2)), eq(4, 0)]),
        &[(2, Var(2).plus(Int(1)))],
    );
    let current = all([eq(1, 0), Var(2).eq(Var(3))]);
    act(
        &mut m,
        "ApplyCurrent",
        all([eq(0, 2), if b { Bool(true) } else { current.clone() }]),
        &[(4, Int(1)), (0, Int(4))],
    );
    act(
        &mut m,
        "Retire",
        all([eq(0, 2), current.clone().negate()]),
        &[(0, Int(4))],
    );
    safe(&mut m, "NoLateProfileEffect", eq(4, 0).or(current));
    // Once a profile is installed, later Close does not retroactively invalidate it.
    m.actions[2].guard = all([eq(1, 0), eq(4, 0)]);
    progress(&mut m, "ClosedServicesJoin", eq(1, 1), eq(0, 4), vec![3, 7]);
    m
}
// Materialized fork: a finite fixed prefix, with parent appends and crashes
// before the atomic publication. Actual rename/fsync/blob bytes are API witnesses.
// d20261103136541: one old guard, one replacement, one bare duplicate.
// Job ownership survives caller cancellation. Clone close/exec has NO fairness.
fn store_writer_lease(mode: u8) -> Model {
    let mut m = base::<12>(
        "StoreWriterLease",
        false,
        &[
            ("old_phase", 5, 0), // absent, opening job, Store, ending, released guard, gone
            ("caller_cancelled", 1, 0),
            ("duplicate", 2, 0),      // absent, open, closed
            ("duplicate_kind", 2, 0), // none, fsync, inherited
            ("kernel_owner", 2, 0),   // none, old, replacement
            ("replacement", 3, 0),    // absent, live, dropped, unavailable
            ("released", 1, 0),
            ("unlock_failed", 1, 0),
            ("outcome", 4, 0), // none, handoff, constructor error, abandoned result, Store drop
            ("live_refused", 1, 0),
            ("false_busy", 1, 0),
            ("bad_mutation", 1, 0),
        ],
    );
    let live_old = || eq(0, 1).or(eq(0, 2));
    act(
        &mut m,
        "AcquireForOpeningJob",
        all([eq(0, 0), eq(4, 0)]),
        &[(0, Int(1)), (4, Int(1))],
    );
    act(&mut m, "CancelOpeningCaller", eq(1, 0), &[(1, Int(1))]);
    for (kind, name) in [(1, "CloneForFsync"), (2, "InheritBeforeExec")] {
        act(
            &mut m,
            name,
            all([live_old(), eq(2, 0)]),
            &[(2, Int(1)), (3, Int(kind))],
        );
    }
    act(
        &mut m,
        "RefuseWhileGuardLive",
        all([live_old(), eq(4, 1), eq(9, 0)]),
        &[(9, Int(1))],
    );
    act(
        &mut m,
        "HandoffGuardToStore",
        all([eq(0, 1), eq(1, 0)]),
        &[(0, Int(2)), (8, Int(1))],
    );
    act(
        &mut m,
        "ConstructorFailsAfterAcquisition",
        eq(0, 1),
        &[(0, Int(3)), (8, Int(2))],
    );
    act(
        &mut m,
        "AbandonedOpeningJobFinishes",
        all([eq(0, 1), eq(1, 1)]),
        &[(0, Int(3)), (8, Int(3))],
    );
    act(
        &mut m,
        "DropStoreGuard",
        eq(0, 2),
        &[(0, Int(3)), (8, Int(4))],
    );
    act(
        &mut m,
        "CheckedUnlockFailsKeepsLiveGuard",
        all([live_old(), eq(7, 0)]),
        &[(7, Int(1))],
    );
    let success = m.actions.len();
    // Broken missing-unlock control claims release but leaves its OFD locked.
    act(
        &mut m,
        "SuccessfulExplicitUnlock",
        all([eq(0, 3), if mode == 1 { eq(2, 1) } else { Bool(true) }]),
        &[
            (0, Int(if mode == 1 { 5 } else { 4 })),
            (4, if mode == 1 { Var(4) } else { Int(0) }),
            (6, Int(1)),
            (7, Int(0)),
        ],
    );
    let success_without_duplicate = m.actions.len();
    act(
        &mut m,
        "MissingUnlockButLastFileCloses",
        all([Bool(mode == 1), eq(0, 3), eq(2, 1).negate()]),
        &[(0, Int(5)), (4, Int(0)), (6, Int(1)), (7, Int(0))],
    );
    let failure = m.actions.len();
    act(
        &mut m,
        "DropUnlockFailsUnavailable",
        eq(0, 3),
        &[(0, Int(5)), (4, Int(0)), (6, Int(0)), (7, Int(1))],
    );
    // On failure, last-close clears the old lock only if no duplicate remains.
    // Split actions retain the exact finite descriptor condition.
    m.actions[failure].guard = all([eq(0, 3), eq(2, 1).negate()]);
    act(
        &mut m,
        "DropUnlockFailsWithDuplicate",
        all([eq(0, 3), eq(2, 1)]),
        &[(0, Int(5)), (6, Int(0)), (7, Int(1))],
    );
    let failure_duplicate = m.actions.len() - 1;
    act(
        &mut m,
        "DestroyOldReleasedGuard",
        eq(0, 4),
        &[(0, Int(5)), (4, if mode == 3 { Int(0) } else { Var(4) })],
    );
    act(
        &mut m,
        "IdempotentOldRelease",
        all([eq(0, 4), eq(5, 1)]),
        &[(4, if mode == 3 { Int(0) } else { Var(4) })],
    );
    act(
        &mut m,
        "CloseDuplicateWhileOldLive",
        all([live_old(), eq(2, 1)]),
        &[(2, Int(2)), (4, if mode == 2 { Int(0) } else { Var(4) })],
    );
    act(
        &mut m,
        "LastOldDescriptorCloses",
        all([eq(0, 5), eq(2, 1), eq(4, 1)]),
        &[(2, Int(2)), (4, Int(0))],
    );
    act(
        &mut m,
        "OldDescriptorClosesWhileReplacementLives",
        all([eq(0, 5), eq(2, 1), eq(4, 1).negate()]),
        &[(2, Int(2)), (4, if mode == 3 { Int(0) } else { Var(4) })],
    );
    act(
        &mut m,
        "ReplacementAcquiresBeforeOldExec",
        all([eq(0, 0).negate(), eq(5, 0), eq(4, 0)]),
        &[(5, Int(1)), (4, Int(2))],
    );
    act(
        &mut m,
        "ObserveFalseBusyAfterSuccessfulRelease",
        all([eq(6, 1), eq(4, 1), eq(5, 0)]),
        &[(5, Int(3)), (10, Int(1))],
    );
    act(
        &mut m,
        "ObserveUnavailableFailedUnlock",
        all([eq(7, 1), eq(0, 5), eq(4, 1), eq(5, 0)]),
        &[(5, Int(3))],
    );
    act(
        &mut m,
        "MutateThroughMatchingGuard",
        all([live_old(), eq(4, 1)]),
        &[(11, Int(0))],
    );
    act(
        &mut m,
        "MutationAfterPrematureRelease",
        all([live_old(), eq(4, 1).negate()]),
        &[(11, Int(1))],
    );
    act(
        &mut m,
        "ReplacementDropsItsOwnGuard",
        eq(5, 1),
        &[(5, Int(2)), (4, Int(0))],
    );
    safe(
        &mut m,
        "NoConcurrentLiveWriters",
        live_old().and(eq(5, 1)).negate(),
    );
    safe(&mut m, "MutationRequiresMatchingGuardAndLease", eq(11, 0));
    safe(&mut m, "NoFalseBusyAfterSuccessfulRelease", eq(10, 0));
    safe(
        &mut m,
        "OldReleaseCannotClearReplacementLease",
        eq(5, 1).negate().or(eq(4, 2)),
    );
    progress(
        &mut m,
        "EndingGuardSettlesAsReleasedOrUnavailable",
        eq(0, 3),
        eq(0, 4).or(eq(0, 5)),
        vec![
            success,
            success_without_duplicate,
            failure,
            failure_duplicate,
        ],
    );
    if mode == 1 {
        m.name = "BrokenHarnessStoreWriterLeaseMissing".into();
    }
    if mode == 2 {
        m.name = "BrokenHarnessStoreWriterLeasePremature".into();
    }
    if mode == 3 {
        m.name = "BrokenHarnessStoreWriterLeaseStale".into();
    }
    m
}
fn fork_materialize(b: bool) -> Model {
    let mut m = base::<8>(
        "ForkMaterialize",
        b,
        &[
            ("phase", 3, 0),
            ("blobs_ready", 1, 0),
            ("copied_at", 1, 0),
            ("visible", 1, 0),
            ("parent_head", 2, 1),
            ("opaque_bytes_preserved", 1, 0),
            ("source_prefix_preserved", 1, 1),
            ("typed_suffix", 1, 0),
        ],
    );
    act(&mut m, "CopyBlobs", eq(0, 0), &[(0, Int(1)), (1, Int(1))]);
    act(
        &mut m,
        "WriteFixedPrefix",
        eq(0, 1),
        &[(0, Int(2)), (2, Int(1)), (5, Int(1))],
    );
    act(
        &mut m,
        "AtomicPublish",
        if b { eq(0, 0) } else { eq(0, 2) },
        &[(0, Int(3)), (3, Int(1))],
    );
    act(
        &mut m,
        "CrashBeforePublish",
        Var(0).lt(Int(3)),
        &[(0, Int(0)), (1, Int(0)), (2, Int(0)), (5, Int(0))],
    );
    act(
        &mut m,
        "ParentAppends",
        Var(4).lt(Int(2)),
        &[(4, Var(4).plus(Int(1)))],
    );
    safe(
        &mut m,
        "VisibleChildHasBlobsAndFixedPrefix",
        eq(3, 0).or(all([eq(0, 3), eq(1, 1), eq(2, 1)])),
    );
    if b {
        act(
            &mut m,
            "ReserializeLosesFutureData",
            eq(0, 2),
            &[(5, Int(0))],
        );
    }
    safe(
        &mut m,
        "CopiedPrefixPreservesOpaqueBytes",
        eq(2, 0).or(eq(5, 1)),
    );
    act(&mut m, "ReprojectTypedSuffix", eq(3, 1), &[(7, Int(1))]);
    if b {
        act(
            &mut m,
            "BrokenPromotionRewritesSource",
            eq(3, 1),
            &[(6, Int(0))],
        );
    }
    safe(&mut m, "ReprojectNeverRewritesSource", eq(6, 1));
    m
}
fn replay(b: bool) -> Model {
    replay_tail(if b { 1 } else { 0 })
}
fn replay_tail(mode: u8) -> Model {
    let mut m = base::<19>(
        "SubscriberReplay",
        mode == 1,
        &[
            ("head", 3, 3),
            ("epoch", 3, 1),
            ("hwm", 3, 1),
            ("birth", 2, 0),
            ("face_epoch", 3, 1),
            ("cursor", 3, 3),
            ("phase", 3, 1),
            ("resets", 2, 0),
            ("e1", 3, 1),
            ("e2", 3, 1),
            ("e3", 3, 1),
            ("b1", 3, 1),
            ("b2", 3, 1),
            ("b3", 3, 1),
            ("f1", 3, 1),
            ("f2", 3, 1),
            ("f3", 3, 1),
            ("log_epoch_max", 3, 1),
            ("prior_epoch", 3, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessSubscriberReplayEpochReuse".into();
    }
    for cut in 0..3 {
        let mut updates = vec![
            (0, Int(cut as i64)),
            (6, Int(2)),
            (17, if cut == 0 { Int(0) } else { Var(8 + cut - 1) }),
        ];
        for i in cut..3 {
            updates.push((8 + i, Int(0)));
            updates.push((11 + i, Int(0)));
        }
        act(
            &mut m,
            &format!("TailLossTo{cut}"),
            all([eq(6, 1), Var(3).lt(Int(2))]),
            &updates,
        );
    }
    let next = if mode == 2 {
        Var(17).plus(Int(1))
    } else {
        Var(2).plus(Int(1))
    };
    let restart = m.actions.len();
    act(
        &mut m,
        "RestartWithDurableEpoch",
        eq(6, 2),
        &[
            (18, Var(1)),
            (1, next.clone()),
            (2, next),
            (3, Var(3).plus(Int(1))),
            (6, Int(3)),
        ],
    );
    let mut progress_actions = vec![restart];
    for i in 0..3 {
        progress_actions.push(m.actions.len());
        act(
            &mut m,
            &format!("AppendReassigned{}", i + 1),
            all([eq(6, 3), eq(0, i as i64)]),
            &[
                (0, Int(i as i64 + 1)),
                (8 + i, Var(1)),
                (11 + i, Var(3).plus(Int(1))),
            ],
        );
    }
    let mut unsafe_cursor = Var(5).le(Var(0)).negate().or(Var(4).le(Var(1)).negate());
    for i in 0..3 {
        unsafe_cursor = unsafe_cursor.or(Var(5)
            .le(Int(i as i64))
            .negate()
            .and(Var(8 + i).le(Var(4)).negate()));
    }
    let should_reset = if mode == 1 {
        Var(4).eq(Var(1)).negate().and(Var(5).le(Var(0)).negate())
    } else {
        unsafe_cursor
    };
    for cut in 0..=3 {
        let no_later = all((0..cut).map(|i| Var(8 + i).le(Var(4))));
        let trusted = if cut == 0 {
            Var(4).le(Var(1)).negate().or(Var(8).le(Var(4)).negate())
        } else {
            Var(4).le(Var(1)).and(no_later).and(if cut == 3 {
                Bool(true)
            } else {
                Var(8 + cut).le(Var(4)).negate()
            })
        };
        let mut updates = vec![
            (5, Int(cut as i64)),
            (4, Var(1)),
            (6, Int(1)),
            (7, Var(7).plus(Int(1))),
        ];
        for i in cut..3 {
            updates.push((14 + i, Int(0)));
        }
        progress_actions.push(m.actions.len());
        act(
            &mut m,
            &format!("ResetToTrusted{cut}"),
            all([eq(6, 3), eq(0, 3), should_reset.clone(), trusted]),
            &updates,
        );
    }
    progress_actions.push(m.actions.len());
    act(
        &mut m,
        "ResumeWithoutReset",
        all([eq(6, 3), eq(0, 3), should_reset.negate()]),
        &[(4, Var(1)), (6, Int(1))],
    );
    for i in 0..3 {
        act(
            &mut m,
            &format!("Deliver{}", i + 1),
            all([eq(6, 1), eq(5, i as i64), Var(0).le(Int(i as i64)).negate()]),
            &[(5, Int(i as i64 + 1)), (14 + i, Var(11 + i))],
        );
    }
    let prefix = all((0..3).map(|i| Var(5).le(Int(i as i64)).or(Var(14 + i).eq(Var(11 + i)))));
    safe(&mut m, "FaceViewIsLogPrefix", eq(6, 1).negate().or(prefix));
    safe(
        &mut m,
        "EpochNeverReused",
        eq(6, 3).negate().or(Var(1).le(Var(18)).negate()),
    );
    progress(
        &mut m,
        "RecoveryEventuallyReconnects",
        eq(6, 3),
        eq(6, 1),
        progress_actions,
    );
    m
}
// Shutdown's bounded scheduling abstraction: one fork can be in progress at
// the first killpg. It publishes before a second watch opportunity; signalled
// processes exit before escalation. Real 5ms/250ms timing stays a process
// witness, not a claim that fairness supplies a wall-clock deadline.
fn seat_shutdown(mode: u8) -> Model {
    let mut m = base::<7>(
        "SeatShutdown",
        false,
        &[
            ("leader", 3, 0), // alive, signalled, exited/waitable, reaped
            ("member", 3, 0),
            ("fork", 2, 0), // absent, in progress, published
            ("closing", 1, 0),
            ("source", 3, 0), // none, signal, physical EOF, terminal I/O error
            ("output_blocked", 1, 0),
            ("phase", 5, 0), // first watch, fork settles, second watch, exit, escalation
        ],
    );
    act(
        &mut m,
        "BeginFork",
        all([eq(4, 0), eq(2, 0)]),
        &[(2, Int(1))],
    );
    act(&mut m, "BlockOutput", eq(4, 0), &[(5, Int(1))]);
    act(
        &mut m,
        "SignalHandlerKillsPublishedMembers",
        eq(4, 0),
        &[(4, Int(1)), (3, Int(1)), (0, Int(1)), (1, Int(1))],
    );
    act(&mut m, "PhysicalEOFWithoutSignal", eq(4, 0), &[(4, Int(2))]);
    act(&mut m, "TerminalIOError", eq(4, 0), &[(4, Int(3))]);
    let mut fair = Vec::new();
    for phase in [0, 2] {
        for source in [1, 2, 3] {
            for blocked in [0, 1] {
                let observes = !(mode == 2 && source != 1 && blocked == 1);
                let signals = observes && (phase == 0 || mode != 1);
                fair.push(m.actions.len());
                act(
                    &mut m,
                    &format!("Watch{phase}Source{source}Blocked{blocked}"),
                    all([eq(6, phase), eq(4, source), eq(5, blocked)]),
                    &[
                        (6, Int(phase + 1)),
                        (3, if observes { Int(1) } else { Var(3) }),
                        (0, if signals { Int(1) } else { Var(0) }),
                        (1, if signals { Int(1) } else { Var(1) }),
                    ],
                );
            }
        }
    }
    fair.push(m.actions.len());
    act(
        &mut m,
        "InFlightForkPublishesAfterFirstKill",
        all([eq(6, 1), eq(2, 1)]),
        &[(2, Int(2)), (1, Int(0))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "ForkHasSettledBeforeNextWatch",
        all([eq(6, 1), eq(2, 1).negate()]),
        &[(6, Int(2))],
    );
    for leader in 0..=1 {
        for member in 0..=1 {
            fair.push(m.actions.len());
            act(
                &mut m,
                &format!("KernelExitsSignalled{leader}{member}"),
                all([eq(6, 3), eq(0, leader), eq(1, member)]),
                &[(6, Int(4)), (0, Int(leader * 2)), (1, Int(member * 2))],
            );
        }
    }
    fair.push(m.actions.len());
    act(&mut m, "SupervisorEscalates", eq(6, 4), &[(6, Int(5))]);
    for (index, name) in [(0, "Leader"), (1, "Member")] {
        let reap = m.actions.len();
        act(
            &mut m,
            &format!("Reap{name}"),
            eq(index, 2),
            &[(index, Int(3))],
        );
        progress(
            &mut m,
            &format!("{name}EventuallyReaped"),
            eq(index, 2),
            eq(index, 3),
            vec![reap],
        );
    }
    safe(
        &mut m,
        "ShutdownObservationIndependentOfOutput",
        eq(6, 0).or(eq(3, 1)),
    );
    safe(
        &mut m,
        "NoExecutableMembersAfterSupervisorKill",
        eq(6, 5)
            .negate()
            .or(all([Int(2).le(Var(0)), Int(2).le(Var(1))])),
    );
    progress(
        &mut m,
        "WatchEventuallyReachesEscalation",
        eq(4, 0).negate(),
        eq(6, 5),
        fair,
    );
    match mode {
        1 => m.name = "BrokenHarnessSeatShutdownSingleKill".into(),
        2 => m.name = "BrokenHarnessSeatShutdownBlockedOutput".into(),
        _ => {}
    }
    m
}
// Decision d20261103136528: one queued spawn, caller/drop/deadline races,
// one exclusive owner through worker -> result -> caller/retirement -> reap.
// Existing ProcessLifecycle separately owns member/signalling/PID fencing.
fn process_spawn_admission(mode: u8) -> Model {
    let mut m = base::<8>(
        "ProcessSpawnAdmission",
        false,
        &[
            ("caller", 2, 0),    // live, locally cancelled, dropped
            ("admission", 8, 8), // fresh=8; queued, admitted, result, caller, retiring, retired, reaped, refused
            ("child", 3, 0),     // none, running, killed/waitable, reaped
            ("owner", 4, 0),     // none, worker, result, caller, retirement
            ("reservation", 1, 0),
            ("deadline", 1, 0),
            ("returned", 1, 0), // return/drop is explicitly NOT a joined-cleanup assertion
            ("late_admission", 1, 0),
        ],
    );
    let cancelled = || eq(0, 0).negate().or(eq(5, 1));
    act(
        &mut m,
        "QueueLiveRequest",
        all([eq(1, 8), eq(0, 0), eq(5, 0)]),
        &[(1, Int(0))],
    );
    let refuse_unstarted = m.actions.len();
    act(
        &mut m,
        "RefuseBeforeQueue",
        all([eq(1, 8), cancelled()]),
        &[(1, Int(7)), (6, Int(1))],
    );

    act(&mut m, "CancelLocalOnly", eq(0, 0), &[(0, Int(1))]);
    act(
        &mut m,
        "DropCaller",
        eq(0, 2).negate(),
        &[(0, Int(2)), (6, Int(1))],
    );
    act(&mut m, "DeadlineElapses", eq(5, 0), &[(5, Int(1))]);
    let refuse = m.actions.len();
    act(
        &mut m,
        "RefuseCancelledQueue",
        all([eq(1, 0), cancelled()]),
        &[(1, Int(7)), (6, Int(1))],
    );
    // No fairness of permit availability/service. Active queued work may wait.
    act(
        &mut m,
        "AdmitLiveQueue",
        all([eq(1, 0), eq(0, 0), eq(5, 0)]),
        &[(1, Int(1)), (3, Int(1))],
    );
    act(
        &mut m,
        "AdmitCancelledQueueControl",
        all([Bool(mode == 1), eq(1, 0), cancelled()]),
        &[(1, Int(1)), (3, Int(1)), (7, Int(1))],
    );
    let spawn = m.actions.len();
    act(
        &mut m,
        "SpawnFinishesOwned",
        eq(1, 1),
        &[(1, Int(2)), (2, Int(1)), (3, Int(2)), (4, Int(1))],
    );
    let failed = m.actions.len();
    act(
        &mut m,
        "SpawnFailsWithoutChild",
        eq(1, 1),
        &[(1, Int(7)), (3, Int(0)), (6, Int(1))],
    );
    act(
        &mut m,
        "HandoffLiveResult",
        all([eq(1, 2), eq(0, 0), eq(5, 0)]),
        &[(1, Int(3)), (3, Int(3))],
    );
    let abandon = m.actions.len();
    act(
        &mut m,
        "DropAbandonedResult",
        all([eq(1, 2), cancelled()]),
        &[(1, Int(4)), (3, Int(if mode == 2 { 0 } else { 4 }))],
    );
    let begin = m.actions.len();
    act(
        &mut m,
        "RetireCancelledCaller",
        all([eq(1, 3), cancelled()]),
        &[(1, Int(4)), (3, Int(4))],
    );
    act(
        &mut m,
        "FinishNormalCaller",
        all([eq(1, 3), eq(0, 0), eq(5, 0)]),
        &[(1, Int(4)), (3, Int(4))],
    );
    act(
        &mut m,
        "ReturnBeforeCleanupJoins",
        all([Var(1).lt(Int(6)), eq(1, 0).negate(), cancelled(), eq(6, 0)]),
        &[(6, Int(1))],
    );
    let retire = m.actions.len();
    act(
        &mut m,
        "KillAndRetireReservation",
        all([eq(1, 4), eq(3, 4)]),
        &[(1, Int(5)), (2, Int(2)), (4, Int(0))],
    );
    let reap = m.actions.len();
    act(
        &mut m,
        "ReapThroughExistingOwner",
        all([eq(1, 5), eq(3, 4)]),
        &[(1, Int(6)), (2, Int(3)), (3, Int(0))],
    );
    act(&mut m, "ReturnSuccessfulJoin", eq(1, 6), &[(6, Int(1))]);
    safe(&mut m, "NoLateQueuedAdmission", eq(7, 0));
    safe(
        &mut m,
        "NoLostOwnedChild",
        eq(2, 1).or(eq(2, 2)).negate().or(eq(3, 0).negate()),
    );
    safe(
        &mut m,
        "RunningChildKeepsReservation",
        eq(2, 1).negate().or(eq(4, 1)),
    );
    safe(
        &mut m,
        "RetiredChildCannotBeSignalledByReservation",
        eq(2, 2).or(eq(2, 3)).negate().or(eq(4, 0)),
    );
    progress(
        &mut m,
        "UnstartedCancellationReturnsWithoutService",
        eq(1, 8).and(cancelled()),
        eq(6, 1),
        vec![refuse_unstarted],
    );
    progress(
        &mut m,
        "QueuedCancellationReturnsWithoutPermitFairness",
        eq(1, 0).and(cancelled()),
        eq(6, 1),
        vec![refuse],
    );
    progress(
        &mut m,
        "CancelledAdmittedSpawnEventuallyReaps",
        all([eq(1, 0).negate(), Var(1).lt(Int(6)), cancelled()]),
        eq(1, 6).or(eq(1, 7)),
        vec![spawn, failed, abandon, begin, retire, reap],
    );
    // Group fairness: one finishing action, one retirement action and reap each
    // progress only under the stated eligible-service/cleanup assumptions.
    m.weak_fair.push(vec![spawn, failed]);
    m.weak_fair.push(vec![abandon, begin]);
    m.weak_fair.push(vec![retire]);
    m.weak_fair.push(vec![reap]);
    if mode == 1 {
        m.name = "BrokenHarnessProcessSpawnAdmissionLate".into();
    }
    if mode == 2 {
        m.name = "BrokenHarnessProcessSpawnAdmissionOwner".into();
    }
    m
}
fn process_lifecycle(mode: u8) -> Model {
    let mut m = base::<10>(
        "ProcessLifecycle",
        false,
        &[
            ("leader", 2, 0), // 0 running, 1 exited/waitable, 2 reaped
            ("member_alive", 1, 1),
            ("cancel", 2, 0), // 0 none, 1 requested, 2 completed
            ("kill_attempted", 1, 0),
            ("pipe_open", 1, 1),
            ("ended", 1, 0),
            ("signal_to_empty", 1, 0),
            ("seat_alive", 1, 1),
            ("hangup", 1, 0),
            ("registered", 1, 1),
        ],
    );
    let exit = m.actions.len();
    act(&mut m, "LeaderExits", eq(0, 0), &[(0, Int(1))]);
    act(&mut m, "MemberExits", eq(1, 1), &[(1, Int(0))]);
    act(&mut m, "Cancel", eq(2, 0), &[(2, Int(1))]);
    let signal = m.actions.len();
    act(
        &mut m,
        "SignalReservedGroup",
        all([eq(2, 1), eq(3, 0), eq(0, 2).negate()]),
        &[(1, Int(0)), (3, Int(1))],
    );
    let skip = m.actions.len();
    act(
        &mut m,
        "SkipDisownedGroup",
        all([eq(2, 1), eq(3, 0), eq(0, 2)]),
        &[(3, Int(1))],
    );
    let reap = m.actions.len();
    act(
        &mut m,
        "ReapLeader",
        all([
            eq(0, 1),
            if mode == 3 {
                Bool(true)
            } else {
                all([eq(3, 1), eq(1, 0)])
            },
        ]),
        &[(0, Int(2)), (9, if mode == 5 { Var(9) } else { Int(0) })],
    );
    let drain = m.actions.len();
    act(
        &mut m,
        "GraceClosesPipe",
        all([eq(2, 1), eq(3, 1), eq(4, 1), Bool(mode != 1)]),
        &[(4, Int(0))],
    );
    let end = m.actions.len();
    act(
        &mut m,
        "CompleteCancel",
        all([eq(2, 1), eq(3, 1), eq(0, 2), eq(4, 0)]),
        &[(2, Int(2)), (5, Int(1))],
    );
    // A stale registry entry enables the actual handler action after release.
    // The monitor records a signal to that now-foreign identity.
    act(
        &mut m,
        "SignalEmptyGroup",
        all([
            eq(0, 2),
            eq(1, 0),
            if mode == 2 {
                Bool(true)
            } else {
                eq(9, 1) // The shutdown handler acts on registration, not leader state.
            },
        ]),
        &[(6, Int(1))],
    );
    // SIGHUP/TERM/PTY EOF run the signal-safe kill before the async close
    // path is notified. A later supervisor KILL cannot run Rust Drop/join.
    act(
        &mut m,
        "HangupReservedGroup",
        all([eq(8, 0), eq(0, 2).negate()]),
        &[
            (8, Int(1)),
            (1, if mode == 4 { Var(1) } else { Int(0) }),
            (3, if mode == 4 { Var(3) } else { Int(1) }),
        ],
    );
    act(
        &mut m,
        "HangupAfterGroupRetired",
        all([eq(8, 0), eq(0, 2)]),
        &[(8, Int(1))],
    );
    act(
        &mut m,
        "BeginAsyncHangupClose",
        all([eq(8, 1), eq(2, 0)]),
        &[(2, Int(1))],
    );
    for action in &mut m.actions {
        action.guard = action.guard.clone().and(eq(7, 1));
    }
    act(
        &mut m,
        "SupervisorEscalatesToKill",
        all([eq(8, 1), eq(7, 1)]),
        &[(7, Int(0))],
    );
    safe(
        &mut m,
        "NoOwnedMemberAfterSupervisorKill",
        eq(7, 1).or(eq(1, 0)),
    );
    safe(
        &mut m,
        "NoMemberAfterCancel",
        eq(2, 2).negate().or(eq(1, 0)),
    );
    safe(&mut m, "NoSignalToEmptyGroup", eq(6, 0));
    safe(&mut m, "NoPipeWaitAfterEnd", eq(5, 0).or(eq(4, 0)));
    progress(
        &mut m,
        "CancelEventuallyEnds",
        eq(2, 0).negate().and(eq(7, 1)),
        eq(2, 2).or(eq(7, 0)),
        vec![exit, signal, skip, reap, drain, end],
    );
    match mode {
        1 => m.name = "BrokenHarnessProcessDrain".into(),
        2 => m.name = "BrokenHarnessProcessOwnership".into(),
        3 => m.name = "BrokenHarnessProcessReapedMembers".into(),
        4 => m.name = "BrokenHarnessProcessHangup".into(),
        5 => m.name = "BrokenHarnessProcessReapLease".into(),
        _ => {}
    }
    m
}
// Adapted from bboxy450-term DirtyWake: a budgeted drain retires only one
// item, then complete_drain rechecks and marks the unconsumed remainder.
fn dirty_wake_frame(b: bool) -> Model {
    let mut m = base::<8>(
        "DirtyWakeFrame",
        b,
        &[
            ("produced", 3, 0),
            ("consumed", 3, 0),
            ("wake", 1, 0),
            ("draining", 1, 0),
            ("scan_remaining", 3, 3),
            ("scan_pending", 1, 0),
            ("scan_poll_units", 2, 0),
            ("session_closed", 1, 0),
        ],
    );
    // Session-close scanning is a separate owner. Close never depends on scan
    // state; each poll enqueues at most one record before an await boundary.
    act(&mut m, "CloseWithoutJoiningScan", eq(7, 0), &[(7, Int(1))]);
    let dispatch = m.actions.len();
    act(
        &mut m,
        "ScanDispatchOne",
        all([eq(7, 1), eq(5, 0), Int(0).lt(Var(4))]),
        &[(5, Int(1)), (6, if b { Int(2) } else { Int(1) })],
    );
    let resume = m.actions.len();
    act(
        &mut m,
        "ScanAwaitWorker",
        eq(5, 1),
        &[(5, Int(0)), (6, Int(0)), (4, Var(4).minus(Int(1)))],
    );
    safe(&mut m, "ScanPollBudget", Var(6).le(Int(1)));
    act(
        &mut m,
        "Publish",
        Var(0).lt(Int(3)),
        &[(0, Var(0).plus(Int(1))), (2, Int(1))],
    );
    act(
        &mut m,
        "BudgetOne",
        all([eq(3, 0), eq(2, 1), Var(1).lt(Var(0))]),
        &[(1, Var(1).plus(Int(1))), (2, Int(0)), (3, Int(1))],
    );
    act(
        &mut m,
        "CompleteEmpty",
        all([eq(3, 1), Var(1).eq(Var(0))]),
        &[(3, Int(0))],
    );
    act(
        &mut m,
        "CompleteRemainder",
        all([eq(3, 1), Var(1).lt(Var(0))]),
        &[(3, Int(0)), (2, if b { Int(0) } else { Int(1) })],
    );
    safe(
        &mut m,
        "NoLostRemainder",
        all([eq(3, 0), Var(1).lt(Var(0))]).negate().or(eq(2, 1)),
    );
    if !b {
        progress(
            &mut m,
            "ClosedScanEventuallyFinishes",
            eq(7, 1),
            eq(4, 0),
            vec![dispatch, resume],
        );
        progress(
            &mut m,
            "ArrivalYieldsFrame",
            Var(1).lt(Var(0)),
            Var(1).eq(Var(0)),
            vec![4, 5, 6],
        );
    } // broken control isolates safety
    m
}
// Adapted from bboxy450-term chat_scroll.rs: layout follows a finite tail
// sentinel, but never overwrites manual intent; Jump explicitly restores it.
pub fn scroll_stick(b: bool) -> Model {
    let mut m = base::<5>(
        "ScrollStick",
        b,
        &[
            ("tail", 3, 1),
            ("following", 1, 1),
            ("offset", 3, 1),
            ("requested", 3, 0),
            ("panel", 3, 0),
        ],
    );
    act(
        &mut m,
        "AppendFollowing",
        all([eq(1, 1), Var(0).lt(Int(3))]),
        &[(0, Var(0).plus(Int(1))), (2, Var(0).plus(Int(1)))],
    );
    act(
        &mut m,
        "ScrollAway",
        eq(1, 1),
        &[(1, Int(0)), (2, Int(0)), (3, Int(0))],
    );
    act(
        &mut m,
        "AppendManual",
        all([eq(1, 0), Var(0).lt(Int(3))]),
        &[
            (0, Var(0).plus(Int(1))),
            (2, if b { Var(0).plus(Int(1)) } else { Var(2) }),
        ],
    );
    act(
        &mut m,
        "WheelBackToBottom",
        eq(1, 0),
        &[(1, Int(1)), (2, Var(0)), (3, Var(0))],
    );
    act(
        &mut m,
        "JumpToLatest",
        eq(1, 0),
        &[(1, Int(1)), (2, Var(0))],
    );
    act(
        &mut m,
        "ResumeView",
        Bool(true),
        &[(1, Int(1)), (2, Var(0))],
    );
    act(
        &mut m,
        "ScrollInsidePanel",
        Var(4).lt(Int(3)),
        &[(4, Var(4).plus(Int(1)))],
    );
    safe(
        &mut m,
        "TailShownWhileFollowing",
        eq(1, 0).or(Var(2).eq(Var(0))),
    );
    safe(
        &mut m,
        "ManualOffsetPreserved",
        eq(1, 1).or(Var(2).eq(Var(3))),
    );
    m
}
pub fn scroll_resume_control() -> Model {
    let mut m = scroll_stick(false);
    m.name = "BrokenHarnessScrollResume".into();
    act(&mut m, "ResumeWithoutTail", eq(1, 0), &[(1, Int(1))]);
    m
}
// Adapted from bboxy450-term LifecycleOwnership: detaching a face never
// signals the actor; only an explicit Stop ends its own birth.
fn embed_lifecycle(b: bool) -> Model {
    let mut m = base::<10>(
        "EmbedLifecycle",
        b,
        &[
            ("attached", 1, 1),
            ("consume", 1, 1),
            ("host_owned", 1, 1),
            ("actor_alive", 1, 1),
            ("birth", 2, 0),
            ("stop_pending", 2, 0),
            ("held_stop_birth", 2, 0),
            ("applied_stop_birth", 2, 0),
            ("stopped", 1, 0),
            ("owned_turn", 1, 1),
        ],
    );
    act(
        &mut m,
        "AdmitStopForBoundGateway",
        eq(5, 0),
        &[(5, Int(1)), (6, Var(4))],
    );
    act(&mut m, "DetachFace", eq(0, 1), &[(0, Int(0))]);
    act(&mut m, "OwnedTurnFinishes", eq(9, 1), &[(9, Int(0))]);
    act(
        &mut m,
        "ConsumeReturnsAndCreateDropsHost",
        all([eq(0, 0), eq(9, 0), eq(1, 1)]),
        &[(1, Int(0)), (2, Int(0))],
    );
    // Public label A -> B -> A; birth remains monotonic. An old gateway can
    // stop its old actor, never the actor now displayed by the replacement.
    act(
        &mut m,
        "NewBirthAtoBtoA",
        all([eq(1, 0), Var(4).lt(Int(2))]),
        &[
            (4, Var(4).plus(Int(1))),
            (0, Int(1)),
            (1, Int(1)),
            (2, Int(1)),
            (3, Int(1)),
            (8, Int(0)),
            (9, Int(1)),
        ],
    );
    act(
        &mut m,
        "DeliverHeldStop",
        all([eq(5, 1), if b { Bool(true) } else { Var(6).eq(Var(4)) }]),
        &[
            (5, Int(2)),
            (3, Int(0)),
            (8, Int(1)),
            (7, Var(6)),
            (9, Int(0)),
        ],
    );
    act(
        &mut m,
        "OldGatewayDoesNotSignalCurrentActor",
        all([eq(5, 1), Var(6).eq(Var(4)).negate()]),
        &[(5, Int(2))],
    );
    safe(&mut m, "DetachPreservesRun", eq(8, 1).or(eq(3, 1)));
    safe(
        &mut m,
        "StopSignalsOwnBirth",
        eq(8, 0).or(Var(7).eq(Var(4))),
    );
    safe(&mut m, "HostRetiresWithConsume", Var(1).eq(Var(2)));
    safe(
        &mut m,
        "DetachedConsumeReturnsAfterOwnedWork",
        eq(1, 1).or(all([eq(0, 0), eq(9, 0)])),
    );
    m
}
// Harness-specific models: durable reductions and session-lived, birth-fenced MCP.
fn acp_flow(mode: u8) -> Model {
    let mut m = base::<13>(
        "AcpFlow",
        false,
        &[
            ("inbox", 2, 0),
            ("prompt_admitted", 1, 0),
            ("cancel_admitted", 1, 0),
            ("turn", 2, 0),
            ("log_head", 2, 0),
            ("projected", 2, 0),
            ("writer_queue", 2, 0),
            ("reading", 1, 0),
            ("conn", 1, 1),
            ("peer", 1, 1),
            ("response", 2, 0),
            ("cancel_target", 2, 0),
            ("metadata_left", 2, 2),
        ],
    );
    if mode > 0 {
        m.name = format!(
            "BrokenHarnessAcpFlow{}",
            ['A', 'B', 'C'][(mode - 1) as usize]
        );
    }
    act(
        &mut m,
        "ReadPrompt",
        all([eq(0, 0), eq(8, 1)]),
        &[(0, Int(1))],
    );
    act(
        &mut m,
        "ReadCancel",
        all([eq(0, 1), eq(8, 1)]),
        &[(0, Int(2))],
    );
    let admit = m.actions.len();
    act(
        &mut m,
        "DurablePromptAdmission",
        all([Int(0).lt(Var(0)), eq(1, 0), eq(8, 1)]),
        &[(1, Int(1)), (3, Int(1))],
    );
    let cancel = m.actions.len();
    act(
        &mut m,
        "CancelItsRunningPrompt",
        all([eq(0, 2), eq(2, 0), eq(1, 1), eq(3, 1), eq(8, 1)]),
        &[(2, Int(1)), (11, Var(3)), (3, Int(2))],
    );
    act(
        &mut m,
        "CancelAfterEnding",
        all([eq(0, 2), eq(2, 0), eq(1, 1), eq(3, 2), eq(8, 1)]),
        &[(2, Int(1)), (11, Var(3))],
    );
    act(
        &mut m,
        "UnorderedCancelOvertakesPrompt",
        all([Bool(mode == 1), eq(0, 2), eq(2, 0), eq(1, 0), eq(8, 1)]),
        &[(2, Int(1)), (11, Var(3))],
    );
    let emit = m.actions.len();
    // EngineNeverWaits means ENABLED EngineEmit whenever another log event is
    // due. Share its real guard, so coupling emit to stdout capacity is checked
    // over writer_queue, rather than an independently maintained "bad" flag.
    let emit_due = all([eq(1, 1), eq(3, 1), Var(4).lt(Int(2)), eq(8, 1)]);
    let emit_ready = emit_due.clone().and(if mode == 3 {
        Var(6).lt(Int(2))
    } else {
        Bool(true)
    });
    act(
        &mut m,
        "EngineEmit",
        emit_ready.clone(),
        &[(4, Var(4).plus(Int(1)))],
    );
    let end = m.actions.len();
    act(
        &mut m,
        "EngineEnd",
        all([eq(3, 1), eq(4, 2), eq(8, 1)]),
        &[(3, Int(2))],
    );
    let project = m.actions.len();
    act(
        &mut m,
        "SubscriptionProjects",
        all([Var(5).lt(Var(4)), Var(6).lt(Int(2)), eq(8, 1)]),
        &[(5, Var(5).plus(Int(1))), (6, Var(6).plus(Int(1)))],
    );
    act(
        &mut m,
        "ConnectionReply",
        all([Int(0).lt(Var(12)), Var(6).lt(Int(2)), eq(8, 1)]),
        &[(12, Var(12).minus(Int(1))), (6, Var(6).plus(Int(1)))],
    );
    let reply = m.actions.len();
    act(
        &mut m,
        "QueuePromptResponse",
        all([
            eq(3, 2),
            eq(1, 1),
            eq(10, 0),
            Var(5).eq(Var(4)),
            Var(6).lt(Int(2)),
            eq(8, 1),
        ]),
        &[(10, Int(1)), (6, Var(6).plus(Int(1)))],
    );
    let writer = m.actions.len();
    act(
        &mut m,
        "WriterMessage",
        all([
            eq(7, 1),
            eq(8, 1),
            Int(0).lt(Var(6)),
            eq(10, 1).negate().or(Int(1).lt(Var(6))),
        ]),
        &[(6, Var(6).minus(Int(1)))],
    );
    act(
        &mut m,
        "WriterResponse",
        all([eq(7, 1), eq(8, 1), eq(6, 1), eq(10, 1)]),
        &[(6, Int(0)), (10, Int(2))],
    );
    let resume = m.actions.len();
    act(
        &mut m,
        "ReaderResumes",
        all([eq(7, 0), eq(8, 1)]),
        &[(7, Int(1))],
    );
    // Every nonempty readable interval drains under WF{writer}; pausing on an
    // empty queue abstracts infinitely-often reading with a drain opportunity.
    act(
        &mut m,
        "ReaderPauses",
        all([eq(7, 1), eq(6, 0)]),
        &[(7, Int(0))],
    );
    act(
        &mut m,
        "EofOrBrokenStdout",
        eq(9, 1),
        &[(9, Int(0)), (8, Int(0))],
    );
    act(
        &mut m,
        "FullQueueClosesConnection",
        all([Bool(mode == 2), eq(6, 2), eq(8, 1)]),
        &[(8, Int(0))],
    );
    safe(
        &mut m,
        "CancelHitsItsPrompt",
        eq(2, 0).or(Int(0).lt(Var(11))),
    );
    safe(&mut m, "CloseOnlyOnEof", eq(8, 1).or(eq(9, 0)));
    let emit_enabled = m.actions[emit].guard.clone();
    safe(
        &mut m,
        "EngineNeverWaits",
        emit_due.negate().or(emit_enabled),
    );
    progress(
        &mut m,
        "PromptEventuallyResponds",
        Int(0).lt(Var(0)).and(eq(9, 1)),
        eq(10, 2).or(eq(9, 0)),
        vec![admit],
    );
    for fair in [
        vec![cancel, cancel + 1],
        vec![emit],
        vec![end],
        vec![project],
        vec![reply],
        vec![writer, writer + 1],
        vec![resume],
    ] {
        m.weak_fair.push(fair);
    }
    m
}
fn auto_compact(mode: u8) -> Model {
    // phase: idle=0, ladder=1, sending=2; item: none=0, trim=1, summary=2.
    let mut m = base::<13>(
        "AutoCompact",
        false,
        &[
            ("est", 4, 4),
            ("turn_open", 1, 1),
            ("ended_through", 2, 1),
            ("clipped", 3, 0),
            ("replaced", 3, 0),
            ("phase", 2, 0),
            ("steps", 2, 0),
            ("item", 2, 0),
            ("durable_clipped", 3, 0),
            ("durable_replaced", 3, 0),
            ("queued", 1, 0),
            ("latest_ending", 3, 1),
            ("trim_estimate", 3, 3),
        ],
    );
    if mode > 0 {
        m.name = format!(
            "BrokenHarnessAutoCompact{}",
            ['A', 'B', 'C', 'D'][(mode - 1) as usize]
        );
    }
    // Slots 1..2 are completed prefixes; slot 3 covers the open turn.
    // A queued cancellation may end in slot 3 without closing that running turn.
    act(&mut m, "TrimWillFit", eq(5, 0), &[(12, Int(2))]);
    act(&mut m, "TrimNeedsSummary", eq(5, 0), &[(12, Int(3))]);
    act(
        &mut m,
        "CompleteAnotherPrefixTurn",
        all([eq(5, 0), eq(2, 1), eq(11, 1)]),
        &[(2, Int(2)), (11, Int(2))],
    );
    act(
        &mut m,
        "CompletePrefixBelowQueuedEnding",
        all([eq(5, 0), eq(2, 1), eq(11, 3)]),
        &[(2, Int(2))],
    );
    act(&mut m, "QueueTurn", eq(10, 0), &[(10, Int(1))]);
    act(
        &mut m,
        "CancelQueuedTurn",
        eq(10, 1),
        &[(10, Int(0)), (11, Int(3))],
    );
    let frontier = if mode == 4 { Var(11) } else { Var(2) };
    act(
        &mut m,
        "Boundary",
        eq(5, 0),
        &[(5, Int(1)), (0, Int(4)), (6, Int(0))],
    );
    let mut fair = vec![];
    fair.push(m.actions.len());
    act(
        &mut m,
        "BeginTrim",
        all([eq(5, 1), eq(7, 0), eq(6, 0), Var(3).lt(frontier.clone())]),
        &[(7, Int(1))],
    );
    fair.push(m.actions.len());
    let target = if mode == 1 { Int(3) } else { frontier.clone() };
    act(
        &mut m,
        "CompleteTrim",
        eq(7, 1),
        &[
            (3, target.clone()),
            (8, target),
            (7, Int(0)),
            (6, Int(1)),
            (0, Var(12)),
        ],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "BeginSummary",
        all([
            eq(5, 1),
            eq(7, 0),
            Var(4).lt(frontier.clone()),
            Int(2).lt(Var(0)),
            eq(6, 1).or(all([eq(6, 0), frontier.clone().le(Var(3))])),
        ]),
        &[(7, Int(2))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "CompleteSummary",
        eq(7, 2),
        &[
            (4, frontier.clone()),
            (9, frontier.clone()),
            (7, Int(0)),
            (6, Int(2)),
            (0, Int(1)),
        ],
    );
    let nothing = Var(2).le(Var(3)).and(Var(2).le(Var(4)));
    fair.push(m.actions.len());
    act(
        &mut m,
        "Send",
        all([
            eq(5, 1),
            eq(7, 0),
            Var(0).le(Int(2)).or(if mode == 3 {
                Bool(false)
            } else {
                eq(6, 2).or(nothing)
            }),
        ]),
        &[(5, Int(2))],
    );
    act(&mut m, "NextRound", eq(5, 2), &[(5, Int(0))]);
    act(
        &mut m,
        "Restart",
        all([eq(5, 2), eq(7, 0)]),
        &[(3, if mode == 2 { Int(0) } else { Var(8) }), (4, Var(9))],
    );
    safe(
        &mut m,
        "NeverReducesOpenTurn",
        Var(3).le(Var(2)).and(Var(4).le(Var(2))),
    );
    safe(
        &mut m,
        "BoundariesMonotone",
        Var(8).le(Var(3)).and(Var(9).le(Var(4))),
    );
    safe(&mut m, "ItemBeforeRequest", eq(5, 2).negate().or(eq(7, 0)));
    progress(&mut m, "BoundaryEventuallySends", eq(5, 1), eq(5, 2), fair);
    m
}
fn mcp_server(mode: u8) -> Model {
    // Starting=0, ready=1, failed=2, dead=3; round absent/open/finished.
    // Three births stand for the five-attempt product bound; identities are gen mod 2.
    let mut m = base::<16>(
        "McpServer",
        false,
        &[
            ("state", 3, 0),
            ("gen", 2, 0),
            ("failures", 2, 0),
            ("call", 1, 0),
            ("round_open", 2, 0),
            ("held", 1, 0),
            ("held_generation", 2, 0),
            ("accepted_tag", 2, 0),
            ("accepted_birth", 2, 0),
            ("silent", 1, 0),
            ("retired", 1, 0),
            // Input: 0=unset, 1=valid supplied file, 2=invalid supplied file.
            ("file_available", 2, 1),
            ("capability_registered", 1, 0),
            // 0=pending, 1=registered, 2=degraded Notice, 3=silent non-seat.
            ("face_prepared", 3, 0),
            ("output_projected", 1, 0),
            ("notice_emitted", 1, 0),
        ],
    );
    if mode > 0 {
        m.name = format!(
            "BrokenHarnessMcpServer{}",
            ['A', 'B', 'C', 'D', 'E'][(mode - 1) as usize]
        );
    }
    m.check_deadlock = mode == 1;
    act(
        &mut m,
        "StartRound",
        if mode == 1 {
            all([eq(4, 0), eq(0, 1)])
        } else {
            eq(4, 0)
        },
        &[(4, Int(1))],
    );
    act(
        &mut m,
        "Ready",
        all([eq(0, 0), eq(13, 0).negate()]),
        &[(0, Int(1))],
    );
    act(
        &mut m,
        "FailedStartOrExit",
        all([Var(0).le(Int(1)), Var(2).lt(Int(2))]),
        &[(0, Int(2)), (2, Var(2).plus(Int(1)))],
    );
    act(
        &mut m,
        "Restart",
        all([eq(0, 2), Var(1).lt(Int(2))]),
        &[(0, Int(0)), (1, Var(1).plus(Int(1)))],
    );
    act(
        &mut m,
        "Exhausted",
        all([Var(0).le(Int(1)), eq(2, 2)]),
        &[(0, Int(3)), (10, Int(1))],
    );
    let call = m.actions.len();
    act(
        &mut m,
        "Call",
        all([eq(4, 1), eq(3, 0), eq(0, 1)]),
        &[(3, Int(1))],
    );
    act(
        &mut m,
        "ServerHangs",
        all([eq(3, 1), eq(5, 0)]),
        &[(9, Int(1))],
    );
    act(
        &mut m,
        "HoldResult",
        all([eq(3, 1), eq(5, 0), eq(9, 0)]),
        &[(5, Int(1)), (6, Var(1))],
    );
    let fail = m.actions.len();
    act(
        &mut m,
        "UnavailableFailsImmediately",
        all([eq(4, 1), eq(3, 0), eq(0, 1).negate()]),
        &[(4, Int(2))],
    );
    let apply = m.actions.len();
    act(
        &mut m,
        "Respond",
        all([
            eq(3, 1),
            eq(5, 1),
            if mode == 2 {
                Bool(true)
            } else {
                Var(6).eq(Var(1))
            },
        ]),
        &[
            (3, Int(0)),
            (5, Int(0)),
            (4, Int(2)),
            (7, Var(6)),
            (8, Var(1)),
        ],
    );
    let discard = m.actions.len();
    act(
        &mut m,
        "DropStale",
        all([eq(3, 1), eq(5, 1), Var(6).eq(Var(1)).negate()]),
        &[(3, Int(0)), (5, Int(0)), (4, Int(2))],
    );
    let timeout = m.actions.len();
    act(
        &mut m,
        "Timeout",
        if mode == 3 { Bool(false) } else { eq(3, 1) },
        &[(3, Int(0)), (5, Int(0)), (4, Int(2))],
    );
    act(&mut m, "Finished", eq(4, 2), &[]);
    // File validity is an input abstraction. The broken reader consults only
    // the absent legacy environment, reports degradation and misses a valid file.
    act(
        &mut m,
        "ReadCapabilityFile",
        eq(13, 0),
        &[
            (12, Int(if mode == 4 { 0 } else { 1 })),
            (13, Int(if mode == 4 { 2 } else { 1 })),
            (15, Int(if mode == 4 { 1 } else { 0 })),
        ],
    );
    act(
        &mut m,
        "UnavailableFileNotice",
        eq(13, 0),
        &[(11, Int(2)), (13, Int(2)), (15, Int(1))],
    );
    act(
        &mut m,
        "UnsetWithoutNotice",
        eq(13, 0),
        &[
            (11, Int(0)),
            (13, Int(3)),
            (15, Int(if mode == 5 { 1 } else { 0 })),
        ],
    );
    act(
        &mut m,
        "ProjectToolOrProviderOutput",
        all([eq(13, 0).negate(), eq(14, 0)]),
        &[(14, Int(1))],
    );
    safe(
        &mut m,
        "NoUnredactedCapability",
        all([eq(11, 1), eq(14, 1)]).negate().or(eq(12, 1)),
    );
    safe(
        &mut m,
        "QuietWhenUnset",
        all([eq(11, 0), eq(13, 0).negate()]).negate().or(eq(15, 0)),
    );
    safe(
        &mut m,
        "ReadyAfterCapabilityAttempt",
        eq(0, 1).negate().or(eq(13, 0).negate()),
    );
    safe(&mut m, "NoStaleResult", Var(7).eq(Var(8)));
    progress(
        &mut m,
        "RoundNeverWaits",
        eq(4, 0),
        eq(4, 0).negate(),
        vec![0],
    );
    safe(&mut m, "DeadIsFinal", eq(10, 0).or(eq(0, 3)));
    progress(
        &mut m,
        "CallEventuallyEnds",
        eq(3, 1),
        eq(3, 0),
        vec![apply, discard, timeout],
    );
    progress(
        &mut m,
        "RoundMakesProgress",
        eq(4, 1).and(eq(3, 0)),
        eq(4, 2).or(eq(3, 1)),
        vec![fail, call],
    );
    m
}
fn mcp_group_close(leader_only: bool) -> Model {
    mcp_group_close_mode(leader_only, false)
}
fn mcp_group_close_mode(leader_only: bool, skip_lost: bool) -> Model {
    // Close owns a leader and two members. SIGTERM is ignored; the grace
    // expires before SIGKILL, and leader wait completes only after that signal.
    let mut m = base::<7>(
        "McpGroupClose",
        leader_only,
        &[
            ("leader_alive", 1, 1),
            ("members_alive", 2, 2),
            ("close", 3, 0),
            ("term_sent", 1, 0),
            ("leader_reaped", 1, 0),
            ("notification", 2, 0), // pending, delivered, sender lost
            ("registered", 1, 1),
        ],
    );
    act(
        &mut m,
        "CloseSendsIgnoredTerm",
        all([
            eq(2, 0),
            eq(5, 0).negate(),
            if skip_lost { eq(5, 1) } else { Bool(true) },
        ]),
        &[(2, Int(1)), (3, Int(1))],
    );
    act(
        &mut m,
        "GraceExpiresAndKill",
        eq(2, 1),
        &[
            (0, Int(0)),
            (1, if leader_only { Var(1) } else { Int(0) }),
            (2, Int(2)),
            (6, Int(0)),
        ],
    );
    act(
        &mut m,
        "ReapAndCompleteClose",
        all([eq(2, 2), eq(0, 0)]),
        &[(2, Int(3)), (4, Int(1))],
    );
    act(
        &mut m,
        "MemberExits",
        all([Var(2).lt(Int(3)), Int(0).lt(Var(1))]),
        &[(1, Var(1).minus(Int(1)))],
    );
    act(&mut m, "TransportClosed", eq(5, 0), &[(5, Int(1))]);
    act(&mut m, "SenderLost", eq(5, 0), &[(5, Int(2))]);
    // The typed owner can have an already-reaped child only after this fence.
    act(
        &mut m,
        "FencedWaitBeforeFinish",
        all([eq(2, 0), eq(4, 0)]),
        &[(0, Int(0)), (1, Int(0)), (4, Int(1)), (6, Int(0))],
    );
    if skip_lost {
        m.name = "BrokenHarnessMcpLostChild".into();
        act(
            &mut m,
            "SkipCleanupOnReceiverError",
            all([eq(2, 0), eq(5, 2)]),
            &[(2, Int(3))],
        );
    }
    safe(&mut m, "NoRegistrationAfterReap", eq(4, 0).or(eq(6, 0)));
    safe(
        &mut m,
        "NoMemberAfterClose",
        eq(2, 3).negate().or(all([eq(0, 0), eq(1, 0)])),
    );
    safe(
        &mut m,
        "ReapAfterKill",
        eq(4, 0).or(all([eq(0, 0), eq(6, 0)])),
    );
    progress(
        &mut m,
        "CloseEventuallyCompletes",
        Int(0).lt(Var(2)),
        eq(2, 3),
        vec![1, 2],
    );
    m
}
// Byte units abstract the upper bound reported by StreamingMarkdown's parse
// range. An unfolded ready row includes its parsed prefix and literal tail.
fn gui_markdown(mode: u8) -> Model {
    let mut m = base::<6>(
        "GuiMarkdownBudget",
        false,
        &[
            ("received", 8, 0),
            ("parsed_prefix", 8, 0),
            ("parse_work", 64, 0),
            ("completed", 1, 0),
            ("rendered_current", 1, 0),
            ("visible_bytes", 8, 0),
        ],
    );
    if mode != 0 {
        m.name = if mode == 1 {
            "BrokenHarnessGuiMarkdownQuadratic"
        } else if mode == 2 {
            "BrokenHarnessGuiMarkdownHiddenTail"
        } else {
            "BrokenHarnessGuiMarkdownSuppressedRender"
        }
        .into();
    }
    act(
        &mut m,
        "Append",
        all([eq(3, 0), Var(0).lt(Int(7))]),
        &[(0, Var(0).plus(Int(1))), (4, Int(0))],
    );
    let credit = Var(0).plus(Var(0)).plus(Var(0));
    act(
        &mut m,
        "PrepareAppend",
        all([
            eq(3, 0),
            Var(1).lt(Var(0)),
            if mode == 1 {
                Bool(true)
            } else {
                Var(2).plus(Var(0)).le(credit)
            },
        ]),
        &[(1, Var(0)), (2, Var(2).plus(Var(0)))],
    );
    // Completion introduces the final byte. A pre-completion render cannot
    // already satisfy the visibility response just by setting a mode flag.
    act(
        &mut m,
        "CompletionDeliversFinalByte",
        all([eq(3, 0), eq(0, 7)]),
        &[
            (0, Int(8)),
            (1, Int(8)),
            (2, Var(2).plus(Int(8))),
            (3, Int(1)),
            (4, Int(0)),
        ],
    );
    let render = m.actions.len();
    act(
        &mut m,
        "RenderPreparedRow",
        all([eq(4, 0), if mode == 3 { eq(3, 0) } else { Bool(true) }]),
        &[(4, Int(1)), (5, if mode == 2 { Var(1) } else { Var(0) })],
    );
    safe(
        &mut m,
        "ParseWorkIsLinear",
        Var(2).le(Var(0).plus(Var(0)).plus(Var(0)).plus(Var(0))),
    );
    safe(&mut m, "NoHiddenBytes", eq(4, 0).or(Var(5).eq(Var(0))));
    progress(
        &mut m,
        "CompletedReplyEventuallyVisible",
        eq(3, 1),
        eq(5, 8),
        vec![render],
    );
    m
}
fn gui_choice(b: bool) -> Model {
    let mut m = base::<6>(
        "GuiApprovalChoice",
        b,
        &[
            ("prompt_birth", 2, 0),
            ("captured_birth", 2, 0),
            ("captured", 1, 0),
            ("open", 1, 1),
            ("dispatched", 1, 0),
            ("dispatched_for", 2, 0),
        ],
    );
    act(
        &mut m,
        "CaptureRenderedButton",
        eq(3, 1),
        &[(1, Var(0)), (2, Int(1))],
    );
    act(
        &mut m,
        "RetireAndReplacePrompt",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (3, Int(1)), (4, Int(0))],
    );
    act(
        &mut m,
        "ResolveCapturedChoice",
        all([
            eq(2, 1),
            if b {
                Bool(true)
            } else {
                all([eq(3, 1), Var(1).eq(Var(0))])
            },
        ]),
        &[(4, Int(1)), (5, Var(1)), (3, Int(0))],
    );
    safe(
        &mut m,
        "NoRetiredPromptDispatch",
        eq(4, 0).or(Var(5).eq(Var(0))),
    );
    m
}
// The rich face folds a tool's terminal/diff/approval into the tool's card. A completed
// card is cached; its signature must cover the folded child, or a child that advances
// after the card completed is never redrawn. (gui/face_rows.rs Folds::deps, view.rs
// Preparer cache check.)
fn folded_card(b: bool) -> Model {
    let mut m = base::<5>(
        "FoldedCard",
        b,
        &[
            ("child", 2, 0),
            ("cached", 2, 0),
            ("valid", 1, 0),
            ("shown", 2, 0),
            ("fresh", 1, 1),
        ],
    );
    act(
        &mut m,
        "ChildAdvances",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (4, Int(0))],
    );
    let stale = Var(1).eq(Var(0)).negate();
    act(
        &mut m,
        "PrepareRebuild",
        if b {
            eq(2, 0)
        } else {
            eq(2, 0).or(stale.clone())
        },
        &[(1, Var(0)), (2, Int(1)), (3, Var(0)), (4, Int(1))],
    );
    act(
        &mut m,
        "PrepareReuse",
        if b {
            eq(2, 1)
        } else {
            all([eq(2, 1), stale.negate()])
        },
        &[(3, Var(1)), (4, Int(1))],
    );
    safe(
        &mut m,
        "FreshCardShowsChild",
        eq(4, 0).or(Var(3).eq(Var(0))),
    );
    m
}
// Seat state: initial idle, one turn/permission, profile generations A-B-A.
// Applying an event and reporting its state is atomic. Probe receipt/reply are
// separate actions. While a turn is open a weak-fair one-second clock step
// refreshes the report every second tick, regardless of protocol activity.
// Both user and agent starts own the timer. Explicit work phases are reported
// atomically, without advancing the clock. Bytes, tool/approval strings and
// wall-clock scheduling are implementation witnesses.
fn seat_report(mode: u8) -> Model {
    let mut m = base::<13>(
        "SeatReport",
        false,
        &[
            ("phase", 3, 0),
            ("reported", 3, 0),
            ("profile", 2, 0),
            ("title", 2, 0),
            ("probe", 1, 0),
            ("probe_as_input", 1, 0),
            ("report_age", 2, 0),
            ("heartbeat", 1, 0),
            ("origin", 2, 0),
            ("work_phase", 2, 0),
            ("reported_phase", 2, 0),
            ("explicit_phase", 1, 0),
            ("child", 1, 0),
        ],
    );
    m.name = match mode {
        0 => "HarnessSeatReport",
        1 => "BrokenHarnessSeatReportState",
        2 => "BrokenHarnessSeatReportProbe",
        3 => "BrokenHarnessSeatReportHeartbeat",
        4 => "BrokenHarnessSeatReportPhase",
        _ => "BrokenHarnessSeatReportChild",
    }
    .into();
    for (name, origin) in [("UserTurnStarts", 1), ("AgentTurnStarts", 2)] {
        act(
            &mut m,
            name,
            eq(0, 0).or(eq(0, 3)),
            &[
                (0, Int(1)),
                (1, Int(1)),
                (6, Int(0)),
                (8, Int(origin)),
                (9, Int(0)),
                (10, Int(0)),
                (11, Int(0)),
                (12, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "PermissionPending",
        eq(0, 1),
        &[(0, Int(2)), (1, Int(if mode == 1 { 1 } else { 2 }))],
    );
    act(
        &mut m,
        "PermissionResolved",
        eq(0, 2),
        &[(0, Int(1)), (1, Int(1))],
    );
    for (name, phase) in [("Thinking", 0), ("Writing", 1), ("Tool", 2)] {
        act(
            &mut m,
            name,
            eq(0, 1).and(eq(9, phase).negate().or(eq(11, 0))),
            &[
                (9, Int(phase)),
                (10, if mode == 4 { Var(10) } else { Int(phase) }),
                (11, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "ChildArrivesInferred",
        all([eq(0, 1), eq(11, 0), eq(12, 0)]),
        &[(12, Int(1)), (9, Int(2)), (10, Int(2))],
    );
    act(
        &mut m,
        "ChildArrivesExplicit",
        all([eq(0, 1), eq(11, 1), eq(12, 0)]),
        &[(12, Int(1)), (10, if mode == 5 { Int(2) } else { Var(10) })],
    );
    act(&mut m, "ChildCompletes", eq(12, 1), &[(12, Int(0))]);
    act(
        &mut m,
        "Retrying",
        eq(0, 1),
        &[(9, Int(0)), (10, Int(0)), (11, Int(0))],
    );
    act(
        &mut m,
        "TurnEnds",
        eq(0, 1).or(eq(0, 2)),
        &[
            (0, Int(0)),
            (1, Int(0)),
            (6, Int(0)),
            (8, Int(0)),
            (9, Int(0)),
            (10, Int(0)),
            (11, Int(0)),
            (12, Int(0)),
        ],
    );
    act(
        &mut m,
        "TurnErrors",
        eq(0, 1).or(eq(0, 2)),
        &[
            (0, Int(3)),
            (1, Int(3)),
            (6, Int(0)),
            (8, Int(0)),
            (9, Int(0)),
            (10, Int(0)),
            (11, Int(0)),
            (12, Int(0)),
        ],
    );
    act(
        &mut m,
        "ProfileChanges",
        Var(2).lt(Int(2)),
        &[(2, Var(2).plus(Int(1))), (3, Var(2).plus(Int(1)))],
    );
    act(&mut m, "ProbeReceived", eq(4, 0), &[(4, Int(1))]);
    let reply = m.actions.len();
    act(
        &mut m,
        "ProbeAnswered",
        eq(4, 1),
        &[(4, Int(0)), (5, Int(i64::from(mode == 2)))],
    );
    let tick = m.actions.len();
    act(
        &mut m,
        "SecondElapses",
        eq(0, 1).or(eq(0, 2)).and(eq(6, 0)),
        &[(6, Int(1))],
    );
    act(
        &mut m,
        "HeartbeatDue",
        eq(0, 1).or(eq(0, 2)).and(eq(6, 1)),
        &[
            (6, Int(if mode == 3 { 2 } else { 0 })),
            (
                7,
                if mode == 3 {
                    Var(7)
                } else {
                    Int(1).minus(Var(7))
                },
            ),
        ],
    );
    safe(&mut m, "ReportMatchesState", Var(0).eq(Var(1)));
    safe(&mut m, "TitleMatchesProfile", Var(2).eq(Var(3)));
    safe(&mut m, "ProbeNeverBecomesInput", eq(5, 0));
    safe(&mut m, "WorkingReportFresh", Var(6).lt(Int(2)));
    safe(&mut m, "IdleHasNoTimer", eq(0, 1).or(eq(0, 2)).or(eq(6, 0)));
    safe(&mut m, "PhaseReportedImmediately", Var(9).eq(Var(10)));
    safe(
        &mut m,
        "OpenTurnHasOrigin",
        eq(0, 1)
            .or(eq(0, 2))
            .negate()
            .or(Var(8).lt(Int(1)).negate()),
    );
    progress(
        &mut m,
        "ProbeEventuallyAnswered",
        eq(4, 1),
        eq(4, 0),
        vec![reply],
    );
    for bit in [0, 1] {
        progress(
            &mut m,
            &format!("WorkingTurnRefreshes{bit}"),
            eq(0, 1).or(eq(0, 2)).and(eq(7, bit)),
            eq(0, 0).or(eq(0, 3)).or(eq(7, 1 - bit)),
            vec![tick, tick + 1],
        );
    }
    m
}
// ACP agent backend (src/backend/acp_agent.rs): one prompt in flight. After
// session/cancel the client must answer every pending request_permission
// `cancelled`; the agent then ends the prompt itself. One TurnEnded per prompt.
// phase: 0 idle, 1 prompting, 2 awaiting permission, 3 cancelling, 4 ended
// perm: 0 none, 1 pending, 2 answered
fn acp_agent_turn(mode: u8) -> Model {
    let b = mode == 1;
    let mut m = base::<16>(
        "AcpAgentTurn",
        b,
        &[
            ("phase", 4, 0),
            ("perm", 2, 0),
            ("cancel", 1, 0),
            ("agent_done", 1, 0),
            ("ends", 2, 0),
            ("agent_alive", 1, 1),
            ("stubborn", 1, 0),
            ("native_updates", 2, 0),
            ("retained_updates", 2, 0),
            ("origin", 2, 0), // 0 none, 1 user, 2 agent
            ("tool_open", 1, 0),
            ("quiet", 1, 0), // abstract silence deadline; no wall-clock claim
            ("queued_prompt", 1, 0),
            ("work_item", 1, 0),
            ("cancelled_reply", 1, 0),
            ("tool_started", 1, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessAcpAgentGrace".into();
    }
    if mode == 3 {
        m.name = "BrokenHarnessAcpAgentIdleCancel".into();
    }
    act(
        &mut m,
        "Prompt",
        eq(0, 0),
        &[(0, Int(1)), (9, Int(1)), (12, Int(0))],
    );
    act(
        &mut m,
        "PermissionRequested",
        all([eq(0, 1), eq(1, 0), eq(3, 0)]),
        &[(0, Int(2)), (1, Int(1)), (10, Int(1)), (13, Int(1))],
    );
    act(
        &mut m,
        "FaceAnswers",
        all([eq(0, 2), eq(1, 1), eq(2, 0)]),
        &[(0, Int(1)), (1, Int(2))],
    );
    act(
        &mut m,
        "InterruptPrompting",
        all([eq(0, 1), eq(2, 0), eq(1, 1).negate()]),
        &[(0, Int(3)), (2, Int(1))],
    );
    // the protocol: a pending permission is answered `cancelled` with the cancel
    act(
        &mut m,
        "InterruptAwaitingPermission",
        all([eq(0, 2), eq(2, 0)]),
        &[
            (0, Int(3)),
            (2, Int(1)),
            (1, if b { Int(1) } else { Int(2) }),
            (14, Int(1)),
        ],
    );
    // the agent ends its prompt only when it is not waiting on a permission answer
    act(
        &mut m,
        "AgentEndsPrompt",
        all([
            eq(0, 1).or(eq(0, 3)),
            eq(1, 1).negate(),
            eq(3, 0),
            eq(6, 0),
            eq(9, 1).or(eq(0, 3)),
        ]),
        &[(3, Int(1))],
    );
    act(
        &mut m,
        "TurnEnded",
        all([
            eq(3, 1),
            eq(0, 4).negate(),
            if b { Bool(true) } else { Var(7).eq(Var(8)) },
        ]),
        // end_turn settles every open permission (cancelled) before TurnEnded
        &[
            (0, Int(4)),
            (1, Int(0)),
            (10, Int(0)),
            (13, Int(0)),
            (4, Var(4).plus(Int(1))),
        ],
    );
    act(
        &mut m,
        "NextPrompt",
        eq(0, 4),
        &[
            (0, Int(0)),
            (1, Int(0)),
            (2, Int(0)),
            (3, Int(0)),
            (4, Int(0)),
            (5, Int(1)),
            (6, Int(0)),
            (9, Int(0)),
            (10, Int(0)),
            (11, Int(0)),
            (13, Int(0)),
            (14, Int(0)),
            (15, Int(0)),
        ],
    );
    act(
        &mut m,
        "AgentIgnoresCancel",
        all([eq(0, 3), eq(3, 0), eq(6, 0)]),
        &[(6, Int(1))],
    );
    let grace = m.actions.len();
    act(
        &mut m,
        "CancelGraceExpires",
        all([eq(0, 3), eq(3, 0)]),
        &[(3, Int(1)), (5, Int(i64::from(mode == 2)))],
    );
    safe(&mut m, "OneTurnEndedPerPrompt", Var(4).lt(Int(2)));
    let retain = m.actions.len();
    act(
        &mut m,
        "PreserveUpdateAsTypedExtOrNative",
        Var(8).lt(Var(7)),
        &[(8, Var(7))],
    );
    act(
        &mut m,
        "ReceiveUnmappedOrExtraFields",
        all([
            eq(0, 1).or(eq(0, 3)),
            eq(3, 0),
            Var(7).eq(Var(8)),
            Var(7).lt(Int(2)),
        ]),
        &[(7, Var(7).plus(Int(1)))],
    );
    safe(
        &mut m,
        "NoUpdateLossAtTurnEnd",
        eq(0, 4).negate().or(Var(7).eq(Var(8))),
    );
    if mode == 0 {
        m.weak_fair.push(vec![retain]);
    }
    safe(
        &mut m,
        "NoPermissionPendingAfterCancel",
        eq(2, 0).or(eq(1, 1).negate()),
    );
    safe(
        &mut m,
        "StubbornAgentKilledBeforeEnding",
        all([eq(0, 4), eq(6, 1)]).negate().or(eq(5, 0)),
    );
    if mode == 0 {
        progress(
            &mut m,
            "CancelledTurnEnds",
            eq(2, 1),
            eq(0, 4),
            vec![5, grace],
        );
        m.weak_fair.push(vec![6]);
    }
    // Idle work opens a fresh turn atomically; a session gauge does not.
    act(
        &mut m,
        "IdleWorkOpensAgentTurn",
        eq(0, 0),
        &[(0, Int(1)), (9, Int(2)), (13, Int(1)), (11, Int(0))],
    );
    act(
        &mut m,
        "IdlePermissionOpensAgentTurn",
        eq(0, 0),
        &[
            (0, Int(if mode == 3 { 0 } else { 2 })),
            (1, Int(if mode == 3 { 2 } else { 1 })),
            (9, Int(if mode == 3 { 0 } else { 2 })),
            (10, Int(i64::from(mode != 3))),
            (13, Int(i64::from(mode != 3))),
            (14, Int(i64::from(mode == 3))),
            (15, Int(i64::from(mode != 3))),
        ],
    );
    act(&mut m, "IdleSessionGauge", eq(0, 0), &[]);
    act(
        &mut m,
        "AgentToolOpens",
        all([eq(0, 1), eq(9, 2), eq(3, 0), eq(10, 0), eq(13, 0)]),
        &[(10, Int(1)), (13, Int(1)), (11, Int(0)), (15, Int(1))],
    );
    let tool_done = m.actions.len();
    act(
        &mut m,
        "AgentToolCompletes",
        all([eq(0, 1), eq(10, 1), eq(1, 1).negate()]),
        &[(10, Int(0)), (11, Int(0))],
    );
    act(
        &mut m,
        "AgentWireRearmsQuiet",
        all([eq(0, 1), eq(9, 2), eq(3, 0), Var(7).lt(Int(2))]),
        &[(11, Int(0)), (7, Var(7).plus(Int(1)))],
    );
    let timer = m.actions.len();
    act(
        &mut m,
        "QuietTimerFires",
        all([eq(0, 1), eq(9, 2), eq(3, 0)]),
        &[(11, Int(1))],
    );
    act(
        &mut m,
        "UserPromptQueuesBehindAgent",
        all([eq(9, 2), eq(0, 1).or(eq(0, 2))]),
        &[(12, Int(1))],
    );
    let quiet_end = m.actions.len();
    act(
        &mut m,
        "QuietOrQueuedPromptEndsAgentTurn",
        all([
            eq(0, 1),
            eq(9, 2),
            eq(3, 0),
            eq(2, 0),
            eq(10, 0),
            eq(1, 1).negate(),
            eq(11, 1).or(eq(12, 1)),
            Var(7).eq(Var(8)),
        ]),
        &[(3, Int(1))],
    );
    safe(
        &mut m,
        "NoWorkItemOutsideTurn",
        eq(13, 0).or(eq(0, 1).or(eq(0, 2)).or(eq(0, 3))),
    );
    safe(
        &mut m,
        "PermissionCancelledOnlyAfterCancel",
        eq(14, 0).or(eq(2, 1)),
    );
    safe(
        &mut m,
        "QuietEndsOnlyWithoutOpenWork",
        all([eq(9, 2), eq(3, 1), eq(2, 0)])
            .negate()
            .or(all([eq(10, 0), eq(1, 1).negate()])),
    );
    if mode == 0 {
        // A finite burst, answered permissions and completed tools eventually
        // permit the abstract silence timer or a waiting user to close the turn.
        m.weak_fair.push(vec![2, tool_done]);
        m.weak_fair.push(vec![timer]);
        m.weak_fair.push(vec![quiet_end]);
        progress(
            &mut m,
            "AgentTurnEventuallyEnds",
            all([eq(9, 2), eq(2, 0)]),
            eq(0, 4),
            vec![6],
        );
    }
    m
}
#[test]
fn acp_agent_idle_quiet_and_queue_states_are_reachable_and_the_control_is_rejected() {
    let model = acp_agent_turn(0);
    let checked = model.check(100_000).unwrap();
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
    for predicate in [
        |s: &Vec<i64>| s[9] == 2 && s[0] == 1 && s[13] == 1,
        |s: &Vec<i64>| s[9] == 2 && s[0] == 2 && s[1] == 1,
        |s: &Vec<i64>| s[9] == 2 && s[3] == 1 && s[11] == 0 && s[12] == 1,
        |s: &Vec<i64>| s[9] == 2 && s[0] == 4 && s[4] == 1,
    ] {
        assert!(checked.states.iter().any(predicate));
    }
    let broken = acp_agent_turn(3).check(100_000).unwrap();
    assert!(
        broken
            .violations
            .iter()
            .any(|v| v.property == "PermissionCancelledOnlyAfterCancel")
    );
}
// Shared HTTP retry/terminal ownership and Anthropic stop-reason translation.
// Wire framing and token arithmetic are separate real-HTTP witnesses.
fn provider_stream(mode: u8) -> Model {
    let mut m = base::<12>(
        "ProviderStream",
        false,
        &[
            ("phase", 3, 0), // idle, reading, backoff, terminal
            ("attempt", 2, 0),
            ("output", 1, 0),
            ("calls", 2, 0),
            ("reason", 5, 0),   // absent, stop, tools, length, refusal, unknown
            ("outcome", 11, 0), // four finishes, malformed, midstream, cancel, auth, bad, transient, rate
            ("endings", 2, 0),
            ("usage_snapshot", 2, 0),
            ("usage_writes", 2, 0),
            ("usage_written_value", 2, 0),
            ("usage_reported_total", 4, 0),
            ("usage_flushed_total", 4, 0),
        ],
    );
    m.name = match mode {
        0 => "HarnessProviderStream",
        1 => "BrokenHarnessProviderRetry",
        2 => "BrokenHarnessProviderFinish",
        _ => "BrokenHarnessProviderEnding",
    }
    .into();
    act(
        &mut m,
        "StartRequest",
        eq(0, 0),
        &[(0, Int(1)), (1, Int(1))],
    );
    act(
        &mut m,
        "TextThinkingSignatureOrOpaque",
        all([eq(0, 1), eq(4, 0)]),
        &[(2, Int(1))],
    );
    act(
        &mut m,
        "ToolUse",
        all([eq(0, 1), eq(4, 0), Var(3).lt(Int(2))]),
        &[(2, Int(1)), (3, Var(3).plus(Int(1)))],
    );
    act(
        &mut m,
        "ReceiveCumulativeUsageSnapshot",
        all([eq(0, 1), Var(7).lt(Int(2))]),
        &[(7, Var(7).plus(Int(1))), (10, Var(10).plus(Int(1)))],
    );
    let flush = m.actions.len();
    act(
        &mut m,
        "FlushLastUsageAtResponseEnd",
        all([eq(0, 3).or(eq(0, 2)), eq(8, 0), Int(0).lt(Var(7))]),
        &[(8, Int(1)), (9, Var(7)), (11, Var(11).plus(Var(7)))],
    );
    if mode == 3 {
        act(
            &mut m,
            "BrokenWriteEverySnapshot",
            all([eq(0, 1), eq(7, 2)]),
            &[(8, Int(2))],
        );
    }
    safe(&mut m, "OneUsageRecordPerResponse", Var(8).le(Int(1)));
    safe(
        &mut m,
        "UsageNeverDoubleCountedAcrossRetries",
        Var(11).le(Var(10)),
    );
    safe(
        &mut m,
        "NoUsageLostAcrossRetries",
        eq(8, 0).negate().or(Var(10).eq(Var(11).plus(Var(7)))),
    );
    safe(
        &mut m,
        "FlushedUsageIsLastSnapshot",
        eq(8, 0).or(Var(9).eq(Var(7))),
    );
    progress(
        &mut m,
        "ReportedUsageEventuallyPreserved",
        all([eq(0, 3), Int(0).lt(Var(10))]),
        Var(11).eq(Var(10)),
        vec![flush],
    );
    for reason in 1..=5 {
        act(
            &mut m,
            "MessageDeltaReason",
            all([eq(0, 1), eq(4, 0)]),
            &[(4, Int(reason))],
        );
    }
    for reason in 0..=5 {
        for calls in 0..=2 {
            let consistent = reason != 0
                && reason != 5
                && (reason != 1 || calls == 0)
                && (reason != 2 || calls > 0);
            let outcome = if consistent || mode == 2 && reason == 1 {
                reason
            } else {
                5
            };
            act(
                &mut m,
                "MessageStop",
                all([eq(0, 1), eq(4, reason), eq(3, calls)]),
                &[(0, Int(3)), (5, Int(outcome)), (6, Int(1))],
            );
        }
    }
    for class in [10, 11] {
        act(
            &mut m,
            "RetryTransientOrRateLimited",
            all([
                eq(0, 1),
                Var(1).lt(Int(2)),
                if mode == 1 { Bool(true) } else { eq(2, 0) },
            ]),
            &[(0, Int(2)), (5, Int(class))],
        );
    }
    act(
        &mut m,
        "BackoffCompletes",
        all([
            eq(0, 2),
            if mode == 3 {
                Bool(true)
            } else {
                eq(7, 0).or(eq(8, 1))
            },
        ]),
        &[
            (7, Int(0)),
            (8, Int(0)),
            (9, Int(0)),
            (0, Int(1)),
            (1, Var(1).plus(Int(1))),
            (4, Int(0)),
            (5, Int(0)),
        ],
    );
    for class in [8, 9, 10, 11] {
        act(
            &mut m,
            "ErrorBeforeOutputOrRetryExhausted",
            all([eq(0, 1), eq(2, 0)]),
            &[(0, Int(3)), (5, Int(class)), (6, Int(1))],
        );
    }
    act(
        &mut m,
        "ErrorAfterOutput",
        all([eq(0, 1), eq(2, 1)]),
        &[(0, Int(3)), (5, Int(6)), (6, Int(1))],
    );
    let cancel = m.actions.len();
    act(
        &mut m,
        "CancelReadingOrWaiting",
        Var(0).lt(Int(3)),
        &[(0, Int(3)), (5, Int(7)), (6, Int(1))],
    );
    act(
        &mut m,
        "TerminalIgnoresLaterFrames",
        eq(0, 3),
        if mode == 3 { &[(6, Int(2))] } else { &[] },
    );
    safe(
        &mut m,
        "RetryOnlyBeforeOutput",
        eq(0, 2).negate().or(eq(2, 0)),
    );
    safe(
        &mut m,
        "OneTerminal",
        Var(6).le(Int(1)).and(eq(0, 3).negate().or(eq(6, 1))),
    );
    safe(
        &mut m,
        "StopAgreesWithCalls",
        all([eq(0, 3), eq(5, 1)]).negate().or(eq(3, 0)),
    );
    safe(
        &mut m,
        "ToolsAgreesWithCalls",
        all([eq(0, 3), eq(5, 2)]).negate().or(Int(0).lt(Var(3))),
    );
    safe(
        &mut m,
        "AfterOutputErrorsAreMidStream",
        all([eq(0, 3), Int(8).le(Var(5))]).negate().or(eq(2, 0)),
    );
    progress(
        &mut m,
        "RequestEventuallyTerminates",
        Bool(true),
        eq(0, 3),
        vec![cancel],
    );
    m
}

// Finder PATH recovery: two concurrent ACP sessions share one bounded probe.
// OS process cleanup, actual PATH bytes and executable permissions are process
// witnesses in tests/acp_agent.rs, not claims about arbitrary shell behavior.
fn agent_path(mode: u8) -> Model {
    let mut m = base::<6>(
        "AgentPath",
        mode != 0,
        &[
            ("cache", 3, 0), // empty, probing, shell PATH, fallback PATH
            ("probes", 2, 0),
            ("session_a", 2, 0), // absent, waiting, spawned
            ("session_b", 2, 0),
            ("path_a", 2, 0), // inherited, shell, fallback
            ("path_b", 2, 0),
        ],
    );
    if mode != 0 {
        m.name = match mode {
            1 => "BrokenHarnessAgentPathDuplicateProbe",
            2 => "BrokenHarnessAgentPathNoDeadline",
            _ => "BrokenHarnessAgentPathChildEnv",
        }
        .into();
    }
    for session in [2, 3] {
        act(&mut m, "RequestAgent", eq(session, 0), &[(session, Int(1))]);
    }
    let probe = m.actions.len();
    act(
        &mut m,
        "BeginProbe",
        all([eq(0, 0), eq(2, 1).or(eq(3, 1))]),
        &[(0, Int(1)), (1, Var(1).plus(Int(1)))],
    );
    if mode == 1 {
        act(
            &mut m,
            "DuplicateProbe",
            all([eq(0, 1), eq(1, 1), eq(2, 1), eq(3, 1)]),
            &[(1, Int(2))],
        );
    }
    act(&mut m, "ShellSucceeded", eq(0, 1), &[(0, Int(2))]);
    act(&mut m, "ShellFailedOrEmpty", eq(0, 1), &[(0, Int(3))]);
    let deadline = m.actions.len();
    act(
        &mut m,
        "ProbeDeadline",
        if mode == 2 { Bool(false) } else { eq(0, 1) },
        &[(0, Int(3))],
    );
    for (session, child_path) in [(2, 4), (3, 5)] {
        let spawn = m.actions.len();
        act(
            &mut m,
            "SpawnAgent",
            all([eq(session, 1), eq(0, 2).or(eq(0, 3))]),
            &[
                (session, Int(2)),
                (
                    child_path,
                    if mode == 3 {
                        Int(0)
                    } else {
                        Var(0).minus(Int(1))
                    },
                ),
            ],
        );
        safe(
            &mut m,
            &format!("Child{session}UsesResolvedPath"),
            eq(session, 2)
                .negate()
                .or(Var(child_path).eq(Var(0).minus(Int(1)))),
        );
        progress(
            &mut m,
            &format!("Session{session}StartupProgress"),
            eq(session, 1),
            eq(session, 2),
            vec![probe, deadline, spawn],
        );
    }
    safe(&mut m, "LoginShellProbedOnce", Var(1).lt(Int(2)));
    m
}

// Native-log accounting: two identified calls (one and two tokens), repeated in
// either order across two turn boundaries. Seen identities outlive turn totals.
// Missing usage has no accounting action. No fairness is assumed of native IO.
fn native_usage(mode: u8) -> Model {
    let mut m = base::<10>(
        "NativeUsage",
        mode != 0,
        &[
            ("turn", 2, 0),
            ("seen_a", 1, 0),
            ("seen_b", 1, 0),
            ("items_a", 2, 0),
            ("items_b", 2, 0),
            ("turn_usage", 6, 0),
            ("finished_usage", 6, 0),
            ("total", 9, 0),
            ("session_step", 3, 0),
            ("session_tokens", 3, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessNativeUsageScopes".into();
    }
    // Native cumulative A -> B -> A snapshots persist separately and are not
    // added to calls or turn totals. The control incorrectly adds both scopes.
    for (step, tokens) in [(0, 1), (1, 3), (2, 1)] {
        act(
            &mut m,
            "SessionSnapshot",
            eq(8, step),
            &[
                (8, Int(step + 1)),
                (9, Int(tokens)),
                (
                    7,
                    if mode == 2 {
                        Var(7).plus(Int(tokens))
                    } else {
                        Var(7)
                    },
                ),
            ],
        );
    }
    for (seen, items, tokens) in [(1, 3, 1), (2, 4, 2)] {
        act(
            &mut m,
            "AccountNewCall",
            all([Var(0).lt(Int(2)), eq(seen, 0)]),
            &[
                (seen, Int(1)),
                (items, Int(1)),
                (5, Var(5).plus(Int(tokens))),
                (7, Var(7).plus(Int(tokens))),
            ],
        );
        act(
            &mut m,
            "RepeatedBlockOrSnapshot",
            all([Var(0).lt(Int(2)), eq(seen, 1), Var(items).lt(Int(2))]),
            if mode == 1 {
                vec![
                    (items, Var(items).plus(Int(1))),
                    (5, Var(5).plus(Int(tokens))),
                    (7, Var(7).plus(Int(tokens))),
                ]
            } else {
                vec![]
            }
            .as_slice(),
        );
    }
    let end = m.actions.len();
    act(
        &mut m,
        "EndTurn",
        Var(0).lt(Int(2)),
        &[
            (0, Var(0).plus(Int(1))),
            (6, Var(6).plus(Var(5))),
            (5, Int(0)),
        ],
    );
    safe(
        &mut m,
        "OneUsagePerCall",
        Var(3).eq(Var(1)).and(Var(4).eq(Var(2))),
    );
    safe(
        &mut m,
        "TurnTotalsConserveUsage",
        Var(7).eq(Var(5).plus(Var(6))),
    );
    safe(
        &mut m,
        "OnlyReportedTokens",
        Var(7).eq(Var(1).plus(Var(2)).plus(Var(2))),
    );
    progress(
        &mut m,
        "FiniteImportEnds",
        Var(0).lt(Int(2)),
        eq(0, 2),
        vec![end],
    );
    m
}

// The import/watch writer owns a pending event. A large text blob must be
// durable before an event names it; a failed write publishes no reference.
fn native_spill(b: bool) -> Model {
    let mut m = base::<6>(
        "NativeSpill",
        b,
        &[
            ("phase", 4, 0), // idle, pending, blob ready, published, failed
            ("large", 1, 0),
            ("blob", 1, 0),
            ("reference", 1, 0),
            ("wire_size", 2, 0), // empty, bounded preview, oversized native text
            ("native_or_ext_preserved", 1, 0),
        ],
    );
    for large in 0..=1 {
        act(
            &mut m,
            "ReceiveNativeText",
            eq(0, 0),
            &[(0, Int(1)), (1, Int(large)), (5, Int(1))],
        );
    }
    let put = m.actions.len();
    act(
        &mut m,
        "PersistBlob",
        all([eq(0, 1), eq(1, 1)]),
        &[(0, Int(2)), (2, Int(1))],
    );
    act(
        &mut m,
        "BlobWriteFails",
        all([eq(0, 1), eq(1, 1)]),
        &[(0, Int(4))],
    );
    let publish_small = m.actions.len();
    act(
        &mut m,
        "AppendSmall",
        all([eq(0, 1), eq(1, 0)]),
        &[
            (0, Int(3)),
            (4, Int(1)),
            (5, if b { Int(0) } else { Int(1) }),
        ],
    );
    let publish_large = m.actions.len();
    act(
        &mut m,
        "AppendLarge",
        if b {
            eq(0, 1).or(eq(0, 2)).and(eq(1, 1))
        } else {
            eq(0, 2)
        },
        &[(0, Int(3)), (3, Int(1)), (4, Int(1))],
    );
    safe(&mut m, "ReferenceOnlyAfterBlob", eq(3, 0).or(eq(2, 1)));
    safe(&mut m, "EnvelopeBounded", Var(4).lt(Int(2)));
    safe(
        &mut m,
        "EdgeNeverDropsPublishedData",
        eq(0, 3).negate().or(eq(5, 1)),
    );
    progress(
        &mut m,
        "PendingSettles",
        eq(0, 1),
        eq(0, 3).or(eq(0, 4)),
        vec![put, publish_small, publish_large],
    );
    m
}

// Global text zoom (gui/zoom.rs, view.rs set_zoom, window.rs save_zoom; harness-spec.md
// Standalone window): Cmd+= / Cmd+- / Cmd+0 move the zoom on a 10 % grid between 90 and
// 160 (indices 0..=7 here, 100 % is 1), every change shows the pill and schedules a save,
// and the persisted value equals the shown one once no save is in flight. The broken
// control drops the clamp: ZoomIn walks past the top step.
fn text_zoom(b: bool) -> Model {
    let mut m = base::<4>(
        "TextZoom",
        b,
        &[
            ("zoom", 8, 1),
            ("saved", 8, 1),
            ("pending", 1, 0),
            ("shown", 1, 0),
        ],
    );
    act(
        &mut m,
        "ZoomIn",
        Var(0).lt(Int(if b { 8 } else { 7 })),
        &[(0, Var(0).plus(Int(1))), (2, Int(1)), (3, Int(1))],
    );
    act(
        &mut m,
        "ZoomOut",
        Int(0).lt(Var(0)),
        &[(0, Var(0).minus(Int(1))), (2, Int(1)), (3, Int(1))],
    );
    act(
        &mut m,
        "Reset",
        Bool(true),
        &[(0, Int(1)), (2, Int(1)), (3, Int(1))],
    );
    act(&mut m, "Save", eq(2, 1), &[(1, Var(0)), (2, Int(0))]);
    act(&mut m, "HidePill", eq(3, 1), &[(3, Int(0))]);
    safe(&mut m, "ZoomWithinRange", Var(0).le(Int(7)));
    safe(
        &mut m,
        "PersistedFollowsZoom",
        eq(2, 1).or(Var(1).eq(Var(0))),
    );
    progress(&mut m, "SaveCompletes", eq(2, 1), eq(2, 0), vec![3]);
    m
}
// rich-pane §5: one message retains its op_id and admission session. Births
// 0/1/2 stand for distinct A/B/A sessions, never a reused socket identity.
fn socket_delivery(mode: u8) -> Model {
    let mut m = base::<13>(
        "SocketDelivery",
        false,
        &[
            ("session", 2, 0),
            ("announced", 3, 3),
            ("msg", 3, 0), // new, sent, acked, refused
            ("bound", 2, 0),
            ("target", 2, 0),
            ("op_key", 1, 0),
            ("resent", 1, 0),
            ("turns_A", 2, 0),
            ("turns_B", 2, 0),
            ("turns_A2", 2, 0),
            ("seen_A", 1, 0),
            ("seen_B", 1, 0),
            ("seen_A2", 1, 0),
        ],
    );
    m.name = match mode {
        0 => "HarnessSocketDelivery",
        1 => "BrokenHarnessSocketDeliveryFreshId",
        _ => "BrokenHarnessSocketDeliveryRetarget",
    }
    .into();
    act(
        &mut m,
        "Announce",
        Var(1).eq(Var(0)).negate(),
        &[(1, Var(0))],
    );
    act(
        &mut m,
        "SendBoundMessage",
        all([eq(2, 0), Var(1).eq(Var(0))]),
        &[(2, Int(1)), (3, Var(0)), (4, Var(0))],
    );
    act(
        &mut m,
        "SwitchToNewSession",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (1, Int(3))],
    );
    act(
        &mut m,
        "ResendSameIntent",
        all([eq(2, 0).negate(), eq(6, 0)]),
        &[
            (2, Int(1)),
            (4, if mode == 2 { Var(0) } else { Var(3) }),
            (5, Int(if mode == 1 { 1 } else { 0 })),
            (6, Int(1)),
        ],
    );
    let mut ack = vec![];
    for session in 0..=2 {
        let turns = 7 + session;
        let seen = 10 + session;
        ack.push(m.actions.len());
        act(
            &mut m,
            "AckFirstEffect",
            all([
                eq(2, 1),
                eq(0, session as i64),
                eq(4, session as i64),
                eq(seen, 0).or(eq(5, 1)),
                Var(turns).lt(Int(2)),
            ]),
            &[
                (2, Int(2)),
                (turns, Var(turns).plus(Int(1))),
                (seen, Int(1)),
            ],
        );
        ack.push(m.actions.len());
        act(
            &mut m,
            "AckDuplicateWithoutEffect",
            all([
                eq(2, 1),
                eq(0, session as i64),
                eq(4, session as i64),
                eq(seen, 1),
                eq(5, 0),
            ]),
            &[(2, Int(2))],
        );
        safe(
            &mut m,
            &format!("NoTurnOnUnannouncedSession{session}"),
            eq(3, session as i64).or(eq(turns, 0)),
        );
    }
    ack.push(m.actions.len());
    act(
        &mut m,
        "AckRefusedClosedSocket",
        all([eq(2, 1), Var(4).eq(Var(0)).negate()]),
        &[(2, Int(3))],
    );
    safe(
        &mut m,
        "OneTurnPerMessage",
        Var(7).plus(Var(8)).plus(Var(9)).le(Int(1)),
    );
    progress(
        &mut m,
        "SentEventuallyAckedOrRefused",
        eq(2, 1),
        eq(2, 2).or(eq(2, 3)),
        ack,
    );
    m
}

// Two epochs per birth, three births (A-B-A), one job and two completions.
// All delays/reordering are arbitrary; Connect is weakly fair while live.
fn socket_lifecycle(mode: u8) -> Model {
    let mut m = base::<14>(
        "SocketLifecycle",
        false,
        &[
            ("birth", 2, 0),
            ("epoch", 1, 0),
            ("live", 1, 1),
            ("connected", 1, 0),
            ("job", 3, 0), // none, running, cancelled, done
            ("job_birth", 2, 0),
            ("job_epoch", 1, 0),
            ("applied", 2, 0),
            ("stale_applied", 1, 0),
            ("cancel_applied", 1, 0),
            ("detached", 1, 0),
            ("explicit_close", 1, 0),
            ("pending", 1, 0),
            ("retired", 1, 0),
        ],
    );
    m.name = match mode {
        0 => "HarnessSocketLifecycle",
        1 => "BrokenHarnessSocketLifecycleStale",
        2 => "BrokenHarnessSocketLifecycleDuplicate",
        3 => "BrokenHarnessSocketLifecycleCancelled",
        4 => "BrokenHarnessSocketLifecycleDetach",
        _ => "BrokenHarnessSocketLifecycleStalePending",
    }
    .into();
    let connect = m.actions.len();
    act(
        &mut m,
        "ConnectHello",
        all([eq(2, 1), eq(3, 0), eq(10, 0)]),
        &[(3, Int(1))],
    );
    act(
        &mut m,
        "StartJob",
        all([eq(2, 1), eq(3, 1), eq(4, 0)]),
        &[(4, Int(1)), (5, Var(0)), (6, Var(1)), (12, Int(1))],
    );
    act(&mut m, "Cancel", eq(4, 1), &[(4, Int(2)), (12, Int(0))]);
    act(
        &mut m,
        "Detach",
        eq(10, 0),
        &[
            (10, Int(1)),
            (3, Int(0)),
            (2, if mode == 4 { Int(0) } else { Var(2) }),
        ],
    );
    act(&mut m, "Reattach", eq(10, 1), &[(10, Int(0)), (13, Int(0))]);
    act(
        &mut m,
        "Close",
        eq(2, 1),
        &[(2, Int(0)), (3, Int(0)), (11, Int(1)), (12, Int(0))],
    );
    act(
        &mut m,
        "RestartEpoch",
        all([eq(2, 0), eq(1, 0)]),
        &[(1, Int(1)), (2, Int(1)), (3, Int(0)), (11, Int(0))],
    );
    act(
        &mut m,
        "SwitchBirth",
        Var(0).lt(Int(2)),
        &[
            (0, Var(0).plus(Int(1))),
            (1, Int(0)),
            (2, Int(1)),
            (3, Int(0)),
            (11, Int(0)),
            (12, if mode == 5 { Var(12) } else { Int(0) }),
        ],
    );
    let fresh = Var(0).eq(Var(5)).and(Var(1).eq(Var(6)));
    for job in 1..=3 {
        if job == 2 && mode != 3 || job == 3 && mode != 2 {
            continue;
        }
        for stale in [false, true] {
            if stale && mode != 1 {
                continue;
            }
            act(
                &mut m,
                "Completion",
                all([
                    eq(4, job),
                    eq(2, 1),
                    if stale {
                        fresh.clone().negate()
                    } else {
                        fresh.clone()
                    },
                    Var(7).lt(Int(2)),
                ]),
                &[
                    (4, Int(3)),
                    (12, Int(0)),
                    (7, Var(7).plus(Int(1))),
                    (8, Int(i64::from(stale))),
                    (9, Int(i64::from(job == 2))),
                ],
            );
        }
    }
    let retire = m.actions.len();
    act(
        &mut m,
        "RetireDetached",
        all([eq(10, 1), eq(12, 0), eq(13, 0)]),
        &[(13, Int(1))],
    );
    act(&mut m, "Wait", Bool(true), &[(12, Var(12))]);
    progress(
        &mut m,
        "DetachedOldIntentEventuallyRetired",
        all([
            eq(10, 1),
            eq(13, 0),
            eq(4, 0)
                .or(eq(4, 2))
                .or(eq(4, 3))
                .or(fresh.negate())
                .or(eq(2, 0)),
        ]),
        eq(13, 1).or(eq(10, 0)),
        vec![retire],
    );
    safe(&mut m, "NoStaleCompletion", eq(8, 0));
    safe(&mut m, "NoCancelledCompletion", eq(9, 0));
    safe(&mut m, "NoDuplicateCompletion", Var(7).le(Int(1)));
    safe(
        &mut m,
        "DetachPreservesSession",
        eq(10, 0).or(eq(11, 1)).or(eq(2, 1)),
    );
    progress(
        &mut m,
        "LiveAttachedEventuallyConnected",
        all([eq(2, 1), eq(10, 0), eq(3, 0)]),
        eq(3, 1).or(eq(2, 0)).or(eq(10, 1)),
        vec![connect],
    );
    m
}

// Four preloaded immutable views, each with one notification and one render.
// One unit is 100us: drain share 3, render cost 2 and projection share 6.
// Worker preparation and wall-clock execution are implementation witnesses.
// bboxy450-harness; credit reuse: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn four_view_budget(broken: bool) -> Model {
    let mut m = base::<18>(
        "FourViewBudget",
        false,
        &[
            ("drain_cursor", 3, 0),
            ("projection_cursor", 3, 0),
            ("phase", 2, 0),
            ("visits", 4, 0),
            ("drain_spent", 4, 0),
            ("projection_spent", 8, 0),
            ("todo0", 1, 1),
            ("todo1", 1, 1),
            ("todo2", 1, 1),
            ("todo3", 1, 1),
            ("drained0", 1, 0),
            ("drained1", 1, 0),
            ("drained2", 1, 0),
            ("drained3", 1, 0),
            ("painted0", 1, 0),
            ("painted1", 1, 0),
            ("painted2", 1, 0),
            ("painted3", 1, 0),
        ],
    );
    if broken {
        m.name = "BrokenHarnessFourViewBudgetTail".into();
    }
    for i in 0..4 {
        let next = Int(((i + 1) % 4) as i64);
        act(
            &mut m,
            "DrainNotification",
            all([
                eq(2, 0),
                eq(0, i as i64),
                Var(3).lt(Int(4)),
                Var(4).lt(Int(3)),
                eq(6 + i, 1),
            ]),
            &[
                (0, next.clone()),
                (3, Var(3).plus(Int(1))),
                (4, Var(4).plus(Int(1))),
                (6 + i, Int(0)),
                (10 + i, Int(1)),
            ],
        );
        act(
            &mut m,
            "VisitEmptyDrain",
            all([
                eq(2, 0),
                eq(0, i as i64),
                Var(3).lt(Int(4)),
                Var(4).lt(Int(3)),
                eq(6 + i, 0),
            ]),
            &[(0, next.clone()), (3, Var(3).plus(Int(1)))],
        );
        let allowed = if broken {
            Bool(true)
        } else {
            Var(5).lt(Int(6))
        };
        act(
            &mut m,
            "RenderPreparedView",
            all([
                eq(2, 1),
                eq(1, i as i64),
                Var(3).lt(Int(4)),
                allowed.clone(),
                eq(14 + i, 0),
            ]),
            &[
                (1, next.clone()),
                (3, Var(3).plus(Int(1))),
                (5, Var(5).plus(Int(2))),
                (14 + i, Int(1)),
            ],
        );
        act(
            &mut m,
            "VisitUnchangedView",
            all([
                eq(2, 1),
                eq(1, i as i64),
                Var(3).lt(Int(4)),
                allowed,
                eq(14 + i, 1),
            ]),
            &[(1, next), (3, Var(3).plus(Int(1)))],
        );
        safe(
            &mut m,
            &format!("NotificationConserved{i}"),
            Var(6 + i).plus(Var(10 + i)).eq(Int(1)),
        );
    }
    act(
        &mut m,
        "BeginProjection",
        all([eq(2, 0), eq(3, 4).or(eq(4, 3))]),
        &[(2, Int(1)), (3, Int(0))],
    );
    act(
        &mut m,
        "EndFrame",
        all([
            eq(2, 1),
            eq(3, 4).or(if broken { Bool(false) } else { eq(5, 6) }),
        ]),
        &[(2, Int(2))],
    );
    act(
        &mut m,
        "NextFrame",
        eq(2, 2),
        &[(2, Int(0)), (3, Int(0)), (4, Int(0)), (5, Int(0))],
    );
    let fair = (0..m.actions.len()).collect();
    act(&mut m, "Wait", Bool(true), &[(2, Var(2))]);
    safe(&mut m, "AggregateDrainBudget", Var(4).le(Int(3)));
    safe(&mut m, "AggregateProjectionBudget", Var(5).le(Int(6)));
    progress(
        &mut m,
        "AllAdmittedWorkRetires",
        Bool(true),
        all((0..4).flat_map(|i| [eq(6 + i, 0), eq(14 + i, 1)])),
        fair,
    );
    m
}

// Existing idle Status at tick 15 reaches both subscriber preferences. Delivery
// is atomic here; wall-clock scheduling and <=20s receiver freshness are real
// socket witnesses. A heartbeat changes transport age, never semantic age/head.
// bboxy450-harness; credit reuse: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn socket_heartbeat(broken: bool) -> Model {
    let mut m = base::<8>(
        "SocketHeartbeat",
        broken,
        &[
            ("clock", 21, 0),
            ("transport_age", 21, 0),
            ("semantic_age", 21, 0),
            ("ephemeral", 1, 0),
            ("heartbeat", 1, 0),
            ("delivered", 1, 0),
            ("item_delivered", 1, 0),
            ("durable_head", 0, 0),
        ],
    );
    act(&mut m, "AllowItemEphemeral", eq(0, 0), &[(3, Int(1))]);
    act(&mut m, "ItemSnapshot", Bool(true), &[(6, Var(3))]);
    let tick = m.actions.len();
    act(
        &mut m,
        "ClockTick",
        all([
            Var(0).lt(Int(21)),
            Var(0).lt(Int(14)).or(Int(14).lt(Var(0))),
        ]),
        &[
            (0, Var(0).plus(Int(1))),
            (1, Var(1).plus(Int(1))),
            (2, Var(2).plus(Int(1))),
        ],
    );
    let heartbeat = m.actions.len();
    for ephemeral in 0..=1 {
        let routed = !broken || ephemeral == 1;
        act(
            &mut m,
            "ExistingIdleStatusTick",
            all([eq(0, 14), eq(3, ephemeral)]),
            &[
                (0, Int(15)),
                (1, if routed { Int(0) } else { Var(1).plus(Int(1)) }),
                (2, Int(15)),
                (4, Int(1)),
                (5, Int(i64::from(routed))),
            ],
        );
    }
    safe(
        &mut m,
        "UnconditionalTransportStatus",
        eq(4, 0).or(eq(5, 1)),
    );
    safe(&mut m, "SilentConsumerStaysFresh", Var(1).le(Int(20)));
    safe(
        &mut m,
        "HeartbeatNeverClaimsSemanticProgress",
        Var(2).eq(Var(0)).and(eq(7, 0)),
    );
    safe(&mut m, "ItemPreferencePreserved", eq(3, 1).or(eq(6, 0)));
    progress(
        &mut m,
        "SilentIntervalObserved",
        Bool(true),
        eq(0, 21),
        vec![tick, heartbeat, heartbeat + 1],
    );
    m
}

// Immutable launch context and relay ownership across three A/B/A births.
// Real ACP/MCP bytes and priority semantics remain process witnesses.
// Project policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn launch_context(mode: usize) -> Model {
    let mut m = base::<13>(
        "LaunchContext",
        false,
        &[
            ("birth", 2, 0),
            ("stage", 3, 0),
            ("want_context", 1, 0),
            ("want_mcp", 1, 0),
            ("held_birth", 3, 3),
            ("sent_birth", 3, 3),
            ("context_sent", 1, 0),
            ("mcp_sent", 1, 0),
            ("prompt_count", 2, 0),
            ("startup_age", 2, 0),
            ("launcher_allow", 1, 0),
            ("profile_allow", 1, 0),
            ("effective_allow", 1, 0),
        ],
    );
    m.name = match mode {
        1 => "BrokenHarnessLaunchContextPrompt",
        2 => "BrokenHarnessLaunchContextMcp",
        3 => "BrokenHarnessLaunchContextStale",
        4 => "BrokenHarnessLaunchContextDeadline",
        5 => "BrokenHarnessLaunchContextSynthetic",
        6 => "BrokenHarnessLaunchContextApproval",
        _ => "HarnessLaunchContext",
    }
    .into();
    for context in 0..=1 {
        for mcp in 0..=1 {
            for launcher in 0..=1 {
                for profile in 0..=1 {
                    act(
                        &mut m,
                        "Admit",
                        eq(1, 0),
                        &[
                            (1, Int(1)),
                            (2, Int(context)),
                            (3, Int(mcp)),
                            (10, Int(launcher)),
                            (11, Int(profile)),
                            (
                                12,
                                Int(if mode == 6 {
                                    profile
                                } else {
                                    i64::from(launcher == 1 || profile == 1)
                                }),
                            ),
                        ],
                    );
                }
            }
        }
    }
    act(
        &mut m,
        "HoldCompletion",
        all([eq(1, 1), eq(4, 3)]),
        &[(4, Var(0))],
    );
    act(
        &mut m,
        "StartupReady",
        eq(1, 1),
        &[
            (1, Int(2)),
            (5, Var(0)),
            (6, if mode == 1 { Int(0) } else { Var(2) }),
            (7, if mode == 2 { Int(0) } else { Var(3) }),
        ],
    );
    act(
        &mut m,
        "HeldCompletion",
        all([
            eq(1, 1),
            Var(4).lt(Int(3)),
            if mode == 3 {
                Bool(true)
            } else {
                Var(4).eq(Var(0))
            },
        ]),
        &[(1, Int(2)), (5, Var(4)), (6, Var(2)), (7, Var(3))],
    );
    act(&mut m, "ServerFailed", eq(1, 1), &[(1, Int(3))]);
    let tick = m.actions.len();
    act(
        &mut m,
        "StartupTick",
        all([eq(1, 1), Var(9).lt(Int(2))]),
        &[(9, Var(9).plus(Int(1)))],
    );
    let deadline = m.actions.len();
    act(
        &mut m,
        "StartupDeadline",
        all([eq(1, 1), eq(9, 2), Bool(mode != 4)]),
        &[(1, Int(3))],
    );
    act(
        &mut m,
        "RealPrompt",
        all([eq(1, 2), eq(8, 0)]),
        &[(8, Int(if mode == 5 { 2 } else { 1 }))],
    );
    act(
        &mut m,
        "ReopenSwitchGeneration",
        Var(0).lt(Int(2)),
        &[
            (0, Var(0).plus(Int(1))),
            (1, Int(0)),
            (5, Int(3)),
            (6, Int(0)),
            (7, Int(0)),
            (8, Int(0)),
            (9, Int(0)),
        ],
    );
    act(&mut m, "Wait", Bool(true), &[(0, Var(0))]);
    safe(
        &mut m,
        "AdmittedContextForwarded",
        eq(1, 2).negate().or(Var(6).eq(Var(2))),
    );
    safe(
        &mut m,
        "AdmittedMcpForwarded",
        eq(1, 2).negate().or(Var(7).eq(Var(3))),
    );
    safe(
        &mut m,
        "CurrentGenerationOnly",
        eq(1, 2).negate().or(Var(5).eq(Var(0))),
    );
    safe(&mut m, "NoSyntheticPrompt", Var(8).le(Int(1)));
    safe(
        &mut m,
        "LauncherApprovalMapped",
        eq(1, 0).or(Var(12)
            .eq(Var(10).plus(Var(11)))
            .or(all([eq(10, 1), eq(11, 1), eq(12, 1)]))),
    );
    progress(
        &mut m,
        "StartupSettlesWithoutServerFairness",
        eq(1, 1),
        eq(1, 1).negate(),
        vec![tick, deadline],
    );
    m
}

/// Two identical-text local intents have different op identities. Births 0/1/2
/// represent A/B/A; a reconnect does not change a birth. No server fairness is
/// assumed: an absent durable reply remains visibly pending/unknown forever.
pub fn gui_pending(control: u8) -> Model {
    let mut m = base(
        "GuiPendingPrompt",
        control != 0,
        &[
            ("birth", 2, 0),
            ("state0", 4, 0),
            ("owner0", 2, 0),
            ("ack0", 1, 0),
            ("blue0", 1, 0),
            ("state1", 4, 0),
            ("owner1", 2, 0),
            ("ack1", 1, 0),
            ("blue1", 1, 0),
        ],
    );
    if control != 0 {
        m.name = format!(
            "BrokenHarnessGuiPendingPrompt{}",
            if control == 1 { "Ack" } else { "Stale" }
        );
    }
    for state in [1, 5] {
        let owner = state + 1;
        let ack = state + 2;
        let blue = state + 3;
        // 0 unsent; 1 gray queued; 2 blue durable; 3 rejected; 4 unknown.
        act(
            &mut m,
            "ScheduleLocal",
            eq(state, 0),
            &[(state, Int(1)), (owner, Var(0))],
        );
        act(
            &mut m,
            "AckAdmissionOnly",
            all([eq(state, 1), eq(ack, 0)]),
            &[
                (ack, Int(1)),
                (state, Int(if control == 1 { 0 } else { 1 })),
            ],
        );
        act(
            &mut m,
            "DisconnectUnknownNoResend",
            eq(state, 1),
            &[(state, Int(4))],
        );
        act(&mut m, "RejectedVisible", eq(state, 1), &[(state, Int(3))]);
        let generation = if control == 2 {
            Bool(true)
        } else {
            Var(owner).eq(Var(0))
        };
        act(
            &mut m,
            "CorrelatedDurableUser",
            all([eq(state, 1).or(eq(state, 4)), generation]),
            &[(state, Int(2)), (blue, Int(1))],
        );
        // Replay does not increment a count, re-dispatch or create a new item.
        act(&mut m, "DuplicateDurableReplay", eq(state, 2), &[]);
        safe(
            &mut m,
            "BlueRequiresCurrentCausalBirth",
            eq(blue, 0).or(all([eq(state, 2), Var(owner).eq(Var(0))])),
        );
        safe(
            &mut m,
            "AckCannotRemovePending",
            eq(ack, 0).or(Int(0).lt(Var(state))),
        );
    }
    act(
        &mut m,
        "SwitchBirthABA",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (4, Int(0)), (8, Int(0))],
    );
    m
}

/// Finite ACP selection contract: child births A/B/A, option revisions 0..2,
/// one admitted request, notification/reply reordering and unknown outcomes.
/// No vendor/IO fairness; only the local deadline is weakly fair.
pub fn acp_controls(control: u8) -> Model {
    let mut m = base(
        "AcpControls",
        false,
        &[
            ("birth", 2, 0),
            ("revision", 2, 0),
            ("request_birth", 2, 0),
            ("request_revision", 2, 0),
            ("phase", 3, 0),
            ("applied", 1, 0),
            ("requests", 2, 0),
            ("prompt_effects", 0, 0),
        ],
    );
    if control != 0 {
        m.name = format!(
            "BrokenHarnessAcpControls{}",
            if control == 1 { "Stale" } else { "Replay" }
        );
    }
    act(
        &mut m,
        "SelectAdvertised",
        eq(4, 0),
        &[(2, Var(0)), (3, Var(1)), (4, Int(1)), (6, Int(1))],
    );
    act(
        &mut m,
        "SwitchBirthABA",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (1, Int(0)), (5, Int(0))],
    );
    act(
        &mut m,
        "AgentOptionUpdate",
        Var(1).lt(Int(2)),
        &[(1, Var(1).plus(Int(1))), (5, Int(0))],
    );
    let same = all([Var(0).eq(Var(2)), Var(1).eq(Var(3))]);
    let waiting = eq(4, 1).or(eq(4, 2));
    act(
        &mut m,
        "ConfirmedResponse",
        all([
            waiting.clone(),
            if control == 1 {
                Bool(true)
            } else {
                same.clone()
            },
        ]),
        &[(4, Int(3)), (5, Int(1))],
    );
    act(
        &mut m,
        "StaleResponseIgnored",
        all([waiting.clone(), same.negate()]),
        &[(4, Int(3))],
    );
    act(&mut m, "RefusedRetainsConfirmed", waiting, &[(4, Int(3))]);
    let deadline = m.actions.len();
    act(&mut m, "DeadlineUnknown", eq(4, 1), &[(4, Int(2))]);
    if control == 2 {
        act(
            &mut m,
            "BrokenReplayUnknown",
            all([eq(4, 2), eq(6, 1)]),
            &[(6, Int(2))],
        );
    }
    safe(
        &mut m,
        "ConfirmedHasCurrentChildAndOptionIdentity",
        eq(5, 0).or(all([Var(0).eq(Var(2)), Var(1).eq(Var(3))])),
    );
    safe(&mut m, "UnknownNeverRedispatches", Var(6).lt(Int(2)));
    safe(&mut m, "ConfigHasNoPromptEffect", eq(7, 0));
    progress(
        &mut m,
        "LocalDeadlineSettlesPending",
        eq(4, 1),
        eq(4, 1).negate(),
        vec![deadline],
    );
    m
}

/// One input racing an active ACP prompt. Unknown transport outcomes may already
/// have reached the agent, so only an explicit unconsumed result permits replay.
pub fn acp_steering(control: u8) -> Model {
    let mut m = base::<8>(
        "AcpSteering",
        control != 0,
        &[
            ("stage", 6, 0),
            ("supported", 1, 0),
            ("busy", 1, 1),
            ("deliveries", 2, 0),
            ("unknown", 1, 0),
            ("cancelled", 1, 0),
            ("steer_calls", 1, 0),
            ("prompt_calls", 1, 0),
        ],
    );
    if control != 0 {
        m.name
            .push_str(if control == 1 { "Replay" } else { "Cancel" });
    }
    act(
        &mut m,
        "NegotiateSupported",
        eq(0, 0),
        &[(0, Int(1)), (1, Int(1))],
    );
    act(&mut m, "NegotiateUnsupported", eq(0, 0), &[(0, Int(1))]);
    act(&mut m, "ActivePromptEnds", eq(2, 1), &[(2, Int(0))]);
    let route = m.actions.len();
    act(
        &mut m,
        "SendSteering",
        all([eq(0, 1), eq(1, 1), eq(2, 1)]),
        &[(0, Int(2)), (6, Int(1)), (5, Int(i64::from(control == 2)))],
    );
    act(
        &mut m,
        "QueueUnsupportedOrIdle",
        all([eq(0, 1), eq(1, 0).or(eq(2, 0))]),
        &[(0, Int(4))],
    );
    let replies = m.actions.len();
    act(&mut m, "Injected", eq(0, 2), &[(0, Int(3)), (3, Int(1))]);
    act(&mut m, "ExplicitlyUnconsumed", eq(0, 2), &[(0, Int(4))]);
    for consumed in 0..=1 {
        act(
            &mut m,
            if consumed == 0 {
                "UnknownUnconsumed"
            } else {
                "UnknownAlreadyConsumed"
            },
            eq(0, 2),
            &[
                (0, Int(if control == 1 { 4 } else { 5 })),
                (3, Int(consumed)),
                (4, Int(1)),
            ],
        );
    }
    let dispatch = m.actions.len();
    act(
        &mut m,
        "NormalPrompt",
        all([eq(0, 4), eq(2, 0)]),
        &[(0, Int(6)), (3, Var(3).plus(Int(1))), (7, Int(1))],
    );
    safe(&mut m, "AtMostOneDelivery", Var(3).lt(Int(2)));
    safe(&mut m, "NoCancelForSteering", eq(5, 0));
    safe(&mut m, "NoReplayOfUnknown", eq(4, 0).or(eq(7, 0)));
    progress(
        &mut m,
        "InputRouted",
        eq(0, 1),
        Int(1).lt(Var(0)),
        vec![route, route + 1],
    );
    progress(
        &mut m,
        "SteeringSettles",
        eq(0, 2),
        Int(2).lt(Var(0)),
        (replies..dispatch).collect(),
    );
    progress(
        &mut m,
        "IdleQueueDispatched",
        all([eq(0, 4), eq(2, 0)]),
        eq(0, 6),
        vec![dispatch],
    );
    m
}

// Project-owned dream-log §8 / memory-graph §4 contract models.
// Origin: https://github.com/bboxy450/bboxy450-harness
// Reuse credit policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn dream_lifecycle(mode: u8) -> Model {
    // Refined contract: Proposed, Superseded and Expired are entry-global;
    // Tested/Adopted/Rejected/Retired belong to (entry,harness). Trial pins
    // contain the entry birth; harness identity is represented by its slot.
    let mut m = base::<25>(
        "DreamLifecycle",
        mode != 0,
        &[
            ("entry", 1, 0),
            ("entry_id", 1, 0),
            ("crash", 2, 0),
            ("proposed", 1, 0),
            ("global_terminal", 2, 0),
            ("state_a", 4, 0),
            ("trial_a", 3, 0),
            ("pass_a", 4, 0),
            ("adopt_pin_a", 4, 0),
            ("previous_a", 4, 0),
            ("sink_a", 2, 0),
            ("dirty_a", 1, 1),
            ("state_b", 4, 0),
            ("trial_b", 3, 0),
            ("pass_b", 4, 0),
            ("adopt_pin_b", 4, 0),
            ("previous_b", 4, 0),
            ("sink_b", 2, 0),
            ("dirty_b", 1, 1),
            ("held_entry", 2, 0),
            ("held_pass", 4, 0),
            ("archived_terminal", 1, 0),
            ("adopt_global_at_apply", 2, 0),
            ("served_a", 2, 0),
            ("served_b", 2, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessDreamLifecycleFence".into();
    }
    if mode == 3 {
        m.name = "BrokenHarnessDreamLifecyclePair".into();
    }
    // Pair states: proposed0, tested1, adopted2, rejected3, retired4.
    // The entry-global terminal overlays both harness projections.
    let live = eq(2, 1).negate();
    let open = all([live.clone(), eq(3, 1), eq(4, 0)]);
    act(
        &mut m,
        "Propose",
        all([live.clone(), eq(3, 0)]),
        &[(3, Int(1))],
    );
    for s in [5, 12] {
        let h = if s == 5 { "A" } else { "B" };
        let other_pass = if s == 5 { 14 } else { 7 };
        let pair_pin = Var(0).plus(Var(0)).plus(Int(if s == 5 { 1 } else { 2 }));
        let mut trials = Vec::new();
        for (verdict, name) in [(1, "Pass"), (2, "Fail"), (3, "Inconclusive")] {
            trials.push(m.actions.len());
            act(
                &mut m,
                &format!("Trial{name}{h}"),
                all([open.clone(), eq(s, 0)]),
                &[
                    (s, Int(1)),
                    (s + 1, Int(verdict)),
                    (
                        s + 2,
                        if verdict == 1 {
                            pair_pin.clone()
                        } else {
                            Int(0)
                        },
                    ),
                    (s + 4, Var(s)),
                    (s + 6, Int(1)),
                ],
            );
        }
        let admission = match mode {
            1 => eq(s, 0).or(eq(s, 1)),
            3 => all([eq(s, 1), eq(s + 1, 1).or(Int(0).lt(Var(other_pass)))]),
            _ => all([eq(s, 1), eq(s + 1, 1)]),
        };
        act(
            &mut m,
            &format!("Adopt{h}"),
            all([open.clone(), admission]),
            &[
                (s, Int(2)),
                (s + 3, Var(if mode == 3 { other_pass } else { s + 2 })),
                (s + 4, Var(s)),
                (s + 6, Int(1)),
                (22, Var(4)),
            ],
        );
        act(
            &mut m,
            &format!("Reject{h}"),
            all([open.clone(), eq(s, 1), eq(s + 1, 2)]),
            &[(s, Int(3)), (s + 4, Var(s)), (s + 6, Int(1))],
        );
        act(
            &mut m,
            &format!("Retire{h}"),
            all([open.clone(), eq(s, 2)]),
            &[(s, Int(4)), (s + 4, Var(s)), (s + 6, Int(1))],
        );
        for adopted in [false, true] {
            let effective = all([eq(3, 1), eq(4, 0), eq(s, 2)]);
            act(
                &mut m,
                &format!("Lower{h}Adopted{adopted}"),
                all([
                    live.clone(),
                    if adopted {
                        effective.clone()
                    } else {
                        effective.clone().negate()
                    },
                ]),
                &[
                    (s + 5, if adopted { Var(0).plus(Int(1)) } else { Int(0) }),
                    (s + 6, Int(0)),
                ],
            );
            if adopted {
                safe(
                    &mut m,
                    &format!("AdoptedHasSamePairPass{h}"),
                    effective.negate().or(all([
                        Var(s + 2).eq(pair_pin.clone()),
                        Var(s + 3).eq(Var(s + 2)),
                    ])),
                );
            }
        }
        safe(&mut m, &format!("StateMonotone{h}"), Var(s + 4).le(Var(s)));
        safe(
            &mut m,
            &format!("SinkSubsetAdopted{h}"),
            eq(s + 6, 1).or(
                all([eq(4, 0), eq(s, 2), Var(s + 5).eq(Var(0).plus(Int(1)))])
                    .or(all([eq(4, 0).negate().or(eq(s, 2).negate()), eq(s + 5, 0)])),
            ),
        );
        progress(
            &mut m,
            &format!("ProposedLeaves{h}"),
            all([eq(3, 1), eq(4, 0), eq(s, 0)]),
            eq(3, 0).or(eq(4, 0).negate()).or(eq(s, 0).negate()),
            trials,
        );
    }
    act(
        &mut m,
        "HoldPassingAdoptionA",
        all([open.clone(), eq(5, 1), eq(6, 1), eq(19, 0)]),
        &[(19, Var(0).plus(Int(1))), (20, Var(7))],
    );
    act(
        &mut m,
        "ApplyHeldAdoptionA",
        all([
            live.clone(),
            Int(0).lt(Var(19)),
            if mode == 2 {
                Bool(true)
            } else {
                all([open.clone(), eq(5, 1), Var(19).eq(Var(0).plus(Int(1)))])
            },
        ]),
        &[
            (5, Int(2)),
            (8, Var(20)),
            (9, Var(5)),
            (11, Int(1)),
            (19, Int(0)),
            (22, Var(4)),
        ],
    );
    act(
        &mut m,
        "DiscardStaleAdoption",
        all([
            Int(0).lt(Var(19)),
            eq(4, 0)
                .negate()
                .or(Var(19).eq(Var(0).plus(Int(1))).negate()),
        ]),
        &[(19, Int(0))],
    );
    for (terminal, name) in [(1, "Supersede"), (2, "Expire")] {
        act(
            &mut m,
            name,
            open.clone(),
            &[(4, Int(terminal)), (11, Int(1)), (18, Int(1))],
        );
    }
    act(&mut m, "Crash", eq(2, 0), &[(2, Int(1))]);
    let restart = m.actions.len();
    act(&mut m, "Restart", eq(2, 1), &[(2, Int(2))]);
    progress(
        &mut m,
        "CrashEventuallyRestarts",
        eq(2, 1),
        eq(2, 2),
        vec![restart],
    );
    act(
        &mut m,
        "ReviveNewId",
        all([
            live,
            eq(0, 0),
            eq(3, 1),
            eq(4, 0)
                .negate()
                .or(all([Int(3).le(Var(5)), Int(3).le(Var(12))])),
        ]),
        &[
            (0, Int(1)),
            (1, Int(1)),
            (3, Int(0)),
            (4, Int(0)),
            (21, Int(1)),
            (5, Int(0)),
            (6, Int(0)),
            (7, Int(0)),
            (8, Int(0)),
            (9, Int(0)),
            (11, Int(1)),
            (12, Int(0)),
            (13, Int(0)),
            (14, Int(0)),
            (15, Int(0)),
            (16, Int(0)),
            (18, Int(1)),
        ],
    );
    for (pair, sink, served, h) in [(5, 10, 23, "A"), (12, 17, 24, "B")] {
        let eligible = all([eq(4, 0), eq(pair, 2), Var(sink).eq(Var(0).plus(Int(1)))]);
        act(
            &mut m,
            &format!("LoadCurrent{h}"),
            all([eq(2, 1).negate(), eligible.clone()]),
            &[(served, Var(0).plus(Int(1)))],
        );
        act(
            &mut m,
            &format!("OmitStale{h}"),
            eligible.clone().negate(),
            &[(served, Int(0))],
        );
        safe(
            &mut m,
            &format!("ServedSubsetCurrentlyAdopted{h}"),
            eq(served, 0).or(all([eligible, Var(served).eq(Var(0).plus(Int(1)))])),
        );
    }
    // A request's view is invalidated by lifecycle changes; an old physical
    // artifact may remain, but the next load must revalidate the active pair.
    for action in &mut m.actions {
        for (pair, served) in [(5, 23), (12, 24)] {
            if action
                .updates
                .iter()
                .any(|(field, _)| [0, 4, pair].contains(field))
            {
                action.updates.push((served, Int(0)));
            }
        }
    }
    safe(
        &mut m,
        "RevivalNewId",
        eq(0, 0).or(all([eq(1, 1), eq(21, 1)])),
    );
    safe(&mut m, "GlobalTerminalFencesDelayedAdoption", eq(22, 0));
    m
}

fn dream_merge(mode: u8) -> Model {
    // The union's lossless sufficient statistic for this projection is the
    // component-wise max of validated (entry,harness) states and global
    // terminal facts. Duplicate/reordered envelopes cannot change that join.
    // No timestamp, wall clock or arrival-order winner exists in the state.
    let mut m = base::<13>(
        "DreamMerge",
        mode != 0,
        &[
            ("writes_a", 3, 0),
            ("writes_b", 3, 0),
            ("log_a_pair_a", 4, 0),
            ("log_a_pair_b", 4, 0),
            ("log_a_global", 2, 0),
            ("log_b_pair_a", 4, 0),
            ("log_b_pair_b", 4, 0),
            ("log_b_global", 2, 0),
            ("projection_a", 6, 0),
            ("projection_b", 6, 0),
            ("projection_global", 2, 0),
            ("observed", 3, 0),
            ("merged", 1, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessDreamMergeHarness".into();
    }
    for writer in 0..2 {
        let offset = 2 + writer * 3;
        act(
            &mut m,
            &format!("WriteProposed{writer}"),
            Var(writer).lt(Int(3)),
            &[(writer, Var(writer).plus(Int(1)))],
        );
        for component in 0..3 {
            for state in 1..=if component == 2 { 2 } else { 4 } {
                for advances in [false, true] {
                    let field = offset + component;
                    act(
                        &mut m,
                        &format!("Write{writer}Component{component}State{state}Advance{advances}"),
                        all([
                            Var(writer).lt(Int(3)),
                            if advances {
                                Var(field).lt(Int(state))
                            } else {
                                Int(state).le(Var(field))
                            },
                        ]),
                        &[
                            (writer, Var(writer).plus(Int(1))),
                            (field, if advances { Int(state) } else { Var(field) }),
                        ],
                    );
                }
            }
        }
        act(
            &mut m,
            &format!("StopWriter{writer}"),
            Var(writer).lt(Int(3)),
            &[(writer, Int(3))],
        );
    }
    let stopped = all([eq(0, 3), eq(1, 3)]);
    let mut observe = Vec::new();
    for (before, after, name) in [
        (0, 1, "ObserveA"),
        (2, 3, "ObserveAAfterB"),
        (0, 2, "ObserveB"),
        (1, 3, "ObserveBAfterA"),
    ] {
        observe.push(m.actions.len());
        act(
            &mut m,
            name,
            all([stopped.clone(), eq(11, before)]),
            &[(11, Int(after))],
        );
    }
    act(&mut m, "DuplicateEnvelope", eq(11, 3), &[]);
    act(&mut m, "ReorderObservedEnvelopes", eq(11, 3), &[]);
    let mut merges = Vec::new();
    for (first, second) in [(2, 5), (5, 2)] {
        for winners in 0..8 {
            let mut guard = vec![eq(11, 3)];
            let mut selected = [0; 3];
            for (component, selected_field) in selected.iter_mut().enumerate() {
                let (left, right) = (first + component, second + component);
                let right_wins = winners & (1 << component) != 0;
                guard.push(if right_wins {
                    Var(left).lt(Var(right))
                } else {
                    Var(right).le(Var(left))
                });
                *selected_field = if mode == 1 || right_wins { right } else { left };
            }
            for global in 0..=2 {
                for cross in [false, true] {
                    merges.push(m.actions.len());
                    let mut g = guard.clone();
                    g.push(eq(selected[2], global));
                    g.push(if cross {
                        Int(3).le(Var(selected[1]))
                    } else {
                        Var(selected[1]).lt(Int(3))
                    });
                    act(
                        &mut m,
                        &format!(
                            "Merge{first}Then{second}Winners{winners}Global{global}Cross{cross}"
                        ),
                        all(g),
                        &[
                            (
                                8,
                                if global != 0 {
                                    Int(4 + global)
                                } else {
                                    Var(if mode == 2 && cross {
                                        selected[1]
                                    } else {
                                        selected[0]
                                    })
                                },
                            ),
                            (
                                9,
                                if global != 0 {
                                    Int(4 + global)
                                } else {
                                    Var(selected[1])
                                },
                            ),
                            (10, Var(selected[2])),
                            (12, Int(1)),
                        ],
                    );
                }
            }
        }
    }
    let maximum = |out, left, right| {
        all([Var(right).le(Var(left)), Var(out).eq(Var(left))])
            .or(all([Var(left).lt(Var(right)), Var(out).eq(Var(right))]))
    };
    safe(
        &mut m,
        "CommutativeIdempotentProjection",
        eq(12, 0).or(all([
            maximum(10, 4, 7),
            eq(10, 0)
                .negate()
                .or(all([maximum(8, 2, 5), maximum(9, 3, 6)])),
            eq(10, 1).negate().or(all([eq(8, 5), eq(9, 5)])),
            eq(10, 2).negate().or(all([eq(8, 6), eq(9, 6)])),
        ])),
    );
    progress(
        &mut m,
        "BothLogsEventuallyObserved",
        stopped.clone(),
        eq(11, 3),
        observe,
    );
    progress(
        &mut m,
        "ConvergesOnceWritesStop",
        stopped,
        eq(12, 1),
        merges,
    );
    m
}

fn dream_sink(mode: u8) -> Model {
    // Safe mode publishes a new dream-owned immutable artifact with an
    // atomic no-replace operation. It never renames over the shared index.
    // Broken mode performs the old shared-file read/check/rename sequence.
    // Hand bytes are finite content identities, separate from write versions:
    // the second hand edit can return to the original bytes (ABA).
    let mut m = base::<23>(
        "DreamSink",
        mode != 0,
        &[
            ("hand_writes", 2, 0),
            ("hand_bytes", 2, 0),
            ("expected_hand", 2, 0),
            ("adopted", 3, 0),
            ("phase", 4, 0),
            ("attempt", 3, 0),
            ("read_version", 2, 0),
            ("read_hand", 2, 0),
            ("read_adopted", 3, 0),
            ("artifact_owner", 2, 0),
            ("artifact_bytes", 3, 0),
            ("artifact_pin", 3, 0),
            ("valid", 1, 0),
            ("ownership_changed", 1, 0),
            ("notice", 1, 0),
            ("published", 1, 0),
            ("active", 3, 0),
            ("recorded", 1, 0),
            ("reachable", 1, 0),
            ("crash", 2, 0),
            ("served", 3, 0),
            ("record_active", 3, 0),
            ("shared_index", 1, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessDreamSinkAdoption".into();
    }
    if mode == 3 {
        m.name = "BrokenHarnessDreamSinkLoad".into();
    }
    // Finite membership relation for two-entry selected/active sets.
    let subset = |small, large| {
        all([eq(small, 0)
            .or(eq(small, 1).and(eq(large, 1).or(eq(large, 3))))
            .or(eq(small, 2).and(eq(large, 2).or(eq(large, 3))))
            .or(eq(small, 3).and(eq(large, 3)))])
    };
    // adopted is a two-entry bitset, not a count: either adoption can arrive first.
    for (before, after) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        act(
            &mut m,
            &format!("Adopt{before}To{after}"),
            eq(3, before),
            &[
                (3, Int(after)),
                (16, Var(16).plus(Int(after - before))),
                (20, Int(0)),
            ],
        );
    }
    for (before, after) in [(1, 0), (2, 0), (3, 1), (3, 2)] {
        act(
            &mut m,
            &format!("Retire{before}To{after}"),
            eq(16, before),
            &[(16, Int(after)), (20, Int(0))],
        );
    }
    act(
        &mut m,
        "SharedIndexAlreadyExists",
        eq(4, 0),
        &[(22, Int(1))],
    );
    for (step, bytes, name) in [
        (0, 1, "HandAppend"),
        (1, 0, "HandEditABA"),
        (1, 2, "HandEditNew"),
    ] {
        act(
            &mut m,
            name,
            eq(0, step),
            &[(0, Int(step + 1)), (1, Int(bytes)), (2, Int(bytes))],
        );
    }
    let mut worker = Vec::new();
    for selected in 0..=3 {
        worker.push(m.actions.len());
        let eligible = match selected {
            0 => Bool(true),
            1 => eq(16, 1).or(eq(16, 3)),
            2 => eq(16, 2).or(eq(16, 3)),
            _ => eq(16, 3),
        };
        act(
            &mut m,
            &format!("ReadSnapshotSubset{selected}"),
            all([eq(4, 0), Var(5).lt(Int(3)), eligible]),
            &[
                (4, Int(1)),
                (5, Var(5).plus(Int(1))),
                (6, Var(0)),
                (7, Var(1)),
                (8, Int(selected)),
            ],
        );
    }
    worker.push(m.actions.len());
    act(
        &mut m,
        "CheckDigest",
        all([eq(4, 1), Var(7).eq(Var(1))]),
        &[(4, Int(2))],
    );
    for exhausted in [false, true] {
        worker.push(m.actions.len());
        act(
            &mut m,
            &format!("RetryExhausted{exhausted}"),
            all([
                eq(4, 1),
                Var(7).eq(Var(1)).negate(),
                if exhausted {
                    eq(5, 3)
                } else {
                    Var(5).lt(Int(3))
                },
            ]),
            &[
                (4, Int(if exhausted { 4 } else { 0 })),
                (14, Int(i64::from(exhausted))),
            ],
        );
    }
    // This is the critical interleaving: the hand writer runs AFTER CheckDigest.
    // No-replace creates a different immutable path; the shared bytes stay live.
    worker.push(m.actions.len());
    let mut publication = vec![
        (4, Int(3)),
        (9, Int(1)),
        (10, Var(8)),
        (11, Var(8)),
        (12, Int(1)),
        (15, Int(1)),
    ];
    if mode == 1 {
        publication.push((1, Var(7)));
    }
    act(
        &mut m,
        "PublishNoReplace",
        all([eq(4, 2), eq(9, 0), eq(22, 0)]),
        &publication,
    );
    worker.push(m.actions.len());
    act(
        &mut m,
        "SharedIndexSkip",
        all([eq(4, 2), eq(22, 1)]),
        &[(4, Int(4)), (14, Int(1))],
    );
    act(
        &mut m,
        "HandCreatesTargetAfterCheck",
        all([eq(4, 2), eq(9, 0), Var(0).lt(Int(2))]),
        &[
            (9, Int(2)),
            (10, Int(3)),
            (13, Int(1)),
            (0, Var(0).plus(Int(1))),
        ],
    );
    worker.push(m.actions.len());
    act(
        &mut m,
        "NoReplaceConflictNotice",
        all([eq(4, 2), eq(9, 2)]),
        &[(4, Int(4)), (14, Int(1)), (12, Int(0))],
    );
    // Hand editing a published artifact is legitimate. The pinned IR remains
    // immutable, but the file is now foreign; never claim its digest is valid.
    for bytes in 0..=3 {
        act(
            &mut m,
            &format!("HandTakesOwnershipBytes{bytes}"),
            all([eq(9, 1), Var(0).lt(Int(2))]),
            &[
                (9, Int(2)),
                (10, Int(bytes)),
                (12, Int(0)),
                (13, Int(1)),
                (0, Var(0).plus(Int(1))),
                (20, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "LoaderReachesPublication",
        all([eq(15, 1), eq(9, 1), eq(12, 1), eq(22, 0)]),
        &[(18, Int(1))],
    );
    act(
        &mut m,
        "RecordAdoption",
        if mode == 2 {
            eq(4, 1)
        } else {
            all([
                eq(15, 1),
                eq(18, 1),
                eq(9, 1),
                eq(12, 1),
                Int(0).lt(Var(11)),
                subset(11, 16),
            ])
        },
        &[(17, Int(1)), (21, Var(16))],
    );
    m.actions.last_mut().unwrap().guard = m.actions.last().unwrap().guard.clone().and(eq(17, 0));
    act(&mut m, "ReplayAdoptionReceipt", eq(17, 1), &[]);
    act(
        &mut m,
        "LoadEligiblePins",
        all([eq(17, 1), eq(9, 1), eq(12, 1), subset(11, 16)]),
        &[(20, Var(11))],
    );
    act(
        &mut m,
        "LoadMissingOrStale",
        eq(17, 0)
            .or(eq(9, 1).negate())
            .or(eq(12, 0))
            .or(subset(11, 16).negate()),
        &[(20, if mode == 3 { Var(11) } else { Int(0) })],
    );
    safe(
        &mut m,
        "NoFalseAdoption",
        eq(17, 0).or(all([eq(15, 1), eq(18, 1), subset(11, 21)])),
    );
    safe(
        &mut m,
        "ServedPinsValid",
        eq(20, 0).or(all([
            eq(17, 1),
            eq(9, 1),
            eq(12, 1),
            subset(20, 16),
            Var(20).eq(Var(11)),
        ])),
    );
    // The dream worker can crash at every writer stage. Published artifacts
    // survive; recording is idempotent and only follows loader reachability.
    for action in &mut m.actions {
        if !action.name.starts_with("Hand")
            && !action.name.starts_with("Adopt")
            && !action.name.starts_with("Retire")
        {
            action.guard = action.guard.clone().and(eq(19, 1).negate());
        }
    }
    act(&mut m, "Crash", eq(19, 0), &[(19, Int(1))]);
    let restart = m.actions.len();
    act(&mut m, "Restart", eq(19, 1), &[(19, Int(2))]);
    progress(
        &mut m,
        "SinkCrashEventuallyRestarts",
        eq(19, 1),
        eq(19, 2),
        vec![restart],
    );
    safe(&mut m, "HandLinesPreserved", Var(1).eq(Var(2)));
    safe(
        &mut m,
        "PublishedBlockEqualsAdoptedSnapshot",
        eq(15, 0).or(Var(11).eq(Var(8))),
    );
    safe(
        &mut m,
        "PinnedOutputUncorrupted",
        eq(12, 0).or(all([eq(9, 1), Var(10).eq(Var(11)), eq(15, 1)])),
    );
    safe(
        &mut m,
        "ChangedOwnershipLabeled",
        eq(9, 2).negate().or(all([eq(13, 1), eq(12, 0)])),
    );
    progress(
        &mut m,
        "RegenerateSettlesWithoutHandWriterFairness",
        Bool(true),
        eq(4, 3).or(eq(4, 4)),
        worker.clone(),
    );
    // A no-replace collision is an explicit Notice, never a successful claim.
    // With at most two hand writes there cannot be three digest conflicts.
    progress(
        &mut m,
        "RegenerateSucceedsOrOwnershipConflict",
        Bool(true),
        eq(4, 3).or(eq(14, 1)),
        worker,
    );
    m
}

// D0: two workers, two shards, two opaque pins; A targets shard0 and B
// chooses shard0/1 (symmetry reduction). One crash and one explicit replay.
// IO may stall forever. Only caller return and post-publication release are WF;
// no promise that skipped/stalled observations are eventually cached. OS flock,
// atomic rename/fsync and byte identity need the real process witnesses.
// Origin/policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn dream_d0(mode: u8) -> Model {
    let mut m = base::<19>(
        "DreamD0",
        mode != 0,
        &[
            ("cache0", 3, 0),
            ("cache1", 3, 0),
            ("count0", 2, 0),
            ("count1", 2, 0),
            ("committed0", 3, 0),
            ("committed1", 3, 0),
            ("owner0", 2, 0),
            ("owner1", 2, 0),
            ("phaseA", 9, 0),
            ("phaseB", 9, 0),
            ("shardB", 1, 0),
            ("readA", 3, 0),
            ("readB", 3, 0),
            ("unionA", 3, 0),
            ("unionB", 3, 0),
            ("callerA", 1, 0),
            ("callerB", 1, 0),
            ("crashes", 1, 0),
            ("replays", 1, 0),
        ],
    );
    if mode == 2 {
        m.name = "BrokenHarnessDreamD0Blocking".into();
    }
    let held = |phase| all([Int(1).lt(Var(phase)), Var(phase).lt(Int(6))]);
    act(&mut m, "DispatchA", eq(8, 0), &[(8, Int(1))]);
    for shard in 0..=1 {
        act(
            &mut m,
            &format!("DispatchB{shard}"),
            eq(9, 0),
            &[(9, Int(1)), (10, Int(shard))],
        );
    }
    for worker in 0..2 {
        let phase = 8 + worker;
        let pin = 1_i64 << worker;
        let caller = 15 + worker;
        let returned = m.actions.len();
        act(
            &mut m,
            &format!("ReturnCaller{worker}"),
            all([
                Int(0).lt(Var(phase)),
                eq(caller, 0),
                if mode == 2 {
                    Var(phase).lt(Int(9))
                } else {
                    Bool(true)
                },
            ]),
            &[(caller, Int(1))],
        );
        progress(
            &mut m,
            &format!("DetachedCaller{worker}"),
            Int(0).lt(Var(phase)),
            eq(caller, 1),
            vec![returned],
        );
        for shard in 0..=1 {
            if worker == 0 && shard == 1 {
                continue;
            }
            let target = if worker == 0 {
                Bool(true)
            } else {
                eq(10, shard as i64)
            };
            let owner = 6 + shard;
            act(
                &mut m,
                &format!("Acquire{worker}Shard{shard}"),
                all([target.clone(), eq(phase, 1), eq(owner, 0)]),
                &[(owner, Int(worker as i64 + 1)), (phase, Int(2))],
            );
            act(
                &mut m,
                &format!("Busy{}{}", worker, shard),
                all([target.clone(), eq(phase, 1), Int(0).lt(Var(owner))]),
                &[(phase, Int(if mode == 2 { 9 } else { 7 }))],
            );
            if mode == 2 {
                act(
                    &mut m,
                    "BlockingAcquireUnsticksOnlyAfterOwnerExits",
                    all([target.clone(), eq(phase, 9), eq(owner, 0)]),
                    &[(phase, Int(1))],
                );
            }
            act(
                &mut m,
                &format!("Read{worker}Shard{shard}"),
                all([target.clone(), eq(phase, 2)]),
                &[(11 + worker, Var(shard)), (phase, Int(3))],
            );
            for prior in 0_i64..=3 {
                act(
                    &mut m,
                    &format!("Union{worker}Shard{shard}Prior{prior}"),
                    all([target.clone(), eq(phase, 3), eq(11 + worker, prior)]),
                    &[
                        (13 + worker, Int(if mode == 1 { pin } else { prior | pin })),
                        (phase, Int(4)),
                    ],
                );
                for ready in 0_i64..=3 {
                    act(
                        &mut m,
                        &format!("Publish{worker}Shard{shard}Ledger{prior}Ready{ready}"),
                        all([
                            target.clone(),
                            eq(phase, 4),
                            eq(4 + shard, prior),
                            eq(13 + worker, ready),
                        ]),
                        &[
                            (shard, Int(ready)),
                            (2 + shard, Int(ready.count_ones() as i64)),
                            (4 + shard, Int(prior | pin)),
                            (phase, Int(5)),
                        ],
                    );
                }
            }
            let released = m.actions.len();
            act(
                &mut m,
                &format!("Release{worker}Shard{shard}"),
                all([target.clone(), eq(phase, 5)]),
                &[(owner, Int(0)), (phase, Int(6))],
            );
            m.weak_fair.push(vec![released]);
            act(
                &mut m,
                &format!("CrashRelease{worker}Shard{shard}"),
                all([target.clone(), held(phase), eq(17, 0)]),
                &[(owner, Int(0)), (phase, Int(8)), (17, Int(1))],
            );
            safe(
                &mut m,
                &format!("ClaimOwnsWorker{worker}Shard{shard}"),
                target
                    .clone()
                    .and(held(phase))
                    .eq(Bool(false))
                    .or(eq(owner, worker as i64 + 1)),
            );
        }
        act(
            &mut m,
            &format!("ExplicitDuplicateReplay{worker}"),
            all([eq(18, 0), eq(phase, 6).or(eq(phase, 7)).or(eq(phase, 8))]),
            &[
                (phase, Int(1)),
                (11 + worker, Int(0)),
                (13 + worker, Int(0)),
                (18, Int(1)),
            ],
        );
    }
    safe(
        &mut m,
        "SameShardExclusive",
        held(8).and(held(9)).eq(Bool(false)).or(eq(10, 1)),
    );
    for shard in 0..=1 {
        safe(
            &mut m,
            &format!("CommittedPinsPreserved{shard}"),
            Var(shard).eq(Var(4 + shard)),
        );
        safe(
            &mut m,
            &format!("CountIsUniquePins{shard}"),
            (0_i64..=3)
                .map(|mask| eq(shard, mask).and(eq(2 + shard, mask.count_ones() as i64)))
                .fold(Bool(false), Expr::or),
        );
    }
    act(&mut m, "StalledIOOrScheduler", Bool(true), &[(17, Var(17))]);
    m
}

// Native custody: two materializers for one opaque, versioned namespace;
// one crash/replay, two noncooperating no-replace target births. Staged bytes
// never resolve as evidence. Hash/ULID/clock/byte and kernel semantics are real
// implementation obligations, not supplied by this finite abstraction.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn dream_native_custody(broken: bool) -> Model {
    let mut m = base::<12>(
        "DreamNativeCustody",
        broken,
        &[
            ("owner", 2, 0),
            ("phaseA", 7, 0),
            ("phaseB", 7, 0),
            ("blobs", 2, 0),
            ("log", 2, 0),
            ("pinsA", 1, 0),
            ("pinsB", 1, 0),
            ("crashes", 1, 0),
            ("replays", 1, 0),
            ("foreignLog", 1, 0),
            ("foreignBlob", 1, 0),
            ("publications", 2, 0),
        ],
    );
    act(
        &mut m,
        "ForeignLogBeforeNoReplace",
        eq(4, 0),
        &[(4, Int(2)), (9, Int(1))],
    );
    act(
        &mut m,
        "ForeignBlobBeforeNoReplace",
        eq(3, 0),
        &[(3, Int(2)), (10, Int(1))],
    );
    for worker in 0..2 {
        let phase = 1 + worker;
        let id = worker as i64 + 1;
        let pins = 5 + worker;
        act(
            &mut m,
            &format!("AcquireNative{worker}"),
            all([eq(phase, 0), eq(0, 0)]),
            &[(0, Int(id)), (phase, Int(1))],
        );
        act(
            &mut m,
            &format!("BusyNativeSkip{worker}"),
            all([eq(phase, 0), Int(0).lt(Var(0))]),
            &[(phase, Int(7))],
        );
        act(
            &mut m,
            &format!("StagePrivate{worker}"),
            all([eq(phase, 1), eq(4, 0)]),
            &[(phase, Int(2))],
        );
        act(
            &mut m,
            &format!("ValidateStagedTranscriptAndBlobs{worker}"),
            eq(phase, 2),
            &[(phase, Int(3))],
        );
        act(
            &mut m,
            &format!("PublishBlobsNoReplace{worker}"),
            all([eq(phase, 3), Var(3).lt(Int(2))]),
            &[(3, Int(1)), (phase, Int(4))],
        );
        act(
            &mut m,
            &format!("PublishCompleteLogNoReplace{worker}"),
            all([eq(phase, 4), eq(3, 1), eq(4, 0)]),
            &[(4, Int(1)), (11, Var(11).plus(Int(1))), (phase, Int(5))],
        );
        act(
            &mut m,
            &format!("ValidateReuseCompletedWinner{worker}"),
            all([eq(phase, 1), eq(3, 1), eq(4, 1)]),
            &[(phase, Int(5))],
        );
        act(
            &mut m,
            &format!("NativeConflictRefusal{worker}"),
            all([
                Int(0).lt(Var(phase)),
                Var(phase).lt(Int(5)),
                eq(4, 2).or(eq(3, 2)),
            ]),
            &[(phase, Int(7)), (0, Int(0))],
        );
        let returned = m.actions.len();
        act(
            &mut m,
            &format!("ReturnConfirmedPins{worker}"),
            all([eq(phase, 5), eq(3, 1), eq(4, 1)]),
            &[(phase, Int(6)), (pins, Int(1)), (0, Int(0))],
        );
        m.weak_fair.push(vec![returned]);
        progress(
            &mut m,
            &format!("ConfirmedWorkerReturns{worker}"),
            eq(phase, 5),
            eq(pins, 1).or(eq(phase, 7)),
            vec![returned],
        );
        act(
            &mut m,
            &format!("NativeCrashRelease{worker}"),
            all([Int(0).lt(Var(phase)), Var(phase).lt(Int(6)), eq(7, 0)]),
            &[(phase, Int(7)), (0, Int(0)), (7, Int(1))],
        );
        act(
            &mut m,
            &format!("NativeExplicitReplay{worker}"),
            all([eq(phase, 6).or(eq(phase, 7)), eq(8, 0)]),
            &[(phase, Int(0)), (8, Int(1))],
        );
        if broken {
            act(
                &mut m,
                &format!("ReturnTransientStagingPins{worker}"),
                eq(phase, 2),
                &[(phase, Int(7)), (pins, Int(1)), (0, Int(0))],
            );
        }
        safe(
            &mut m,
            &format!("NativeClaimOwnsWorker{worker}"),
            Int(0)
                .lt(Var(phase))
                .and(Var(phase).lt(Int(6)))
                .eq(Bool(false))
                .or(eq(0, id)),
        );
        safe(
            &mut m,
            &format!("NoPrematureNativePin{worker}"),
            eq(pins, 0).or(eq(3, 1).and(eq(4, 1))),
        );
    }
    safe(
        &mut m,
        "CompleteLogHasCompleteBlobs",
        eq(4, 1).eq(Bool(false)).or(eq(3, 1)),
    );
    safe(&mut m, "ForeignNativeLogPreserved", eq(9, 0).or(eq(4, 2)));
    safe(&mut m, "ForeignNativeBlobPreserved", eq(10, 0).or(eq(3, 2)));
    safe(&mut m, "OneNativePublication", Var(11).le(Int(1)));
    act(&mut m, "NativeIOStalls", Bool(true), &[(7, Var(7))]);
    m
}

// Constructive seal: one owner, zero/one child, one in-flight protected effect.
// phase 0 recording,1 frozen,2 sealed,3 closed; child 0 absent,1 active,2 joined,3 reaped;
// coverage 0 unobserved,1 observed,2 enforced-read-only. Atomic actions are admission,
// completion, spawn/observe/join/reap, freeze, own-custody seal and close. No fairness
// or arbitrary-IO liveness claim. Hashes, capabilities and syscalls need real witnesses.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
fn dream_seal(mode: u8) -> Model {
    let mut m = base::<7>(
        "DreamSeal",
        mode != 0,
        &[
            ("phase", 3, 0),
            ("child", 3, 0),
            ("coverage", 2, 0),
            ("inflight", 1, 0),
            ("spawned", 1, 0),
            ("late_protected", 1, 0),
            ("custody", 1, 0),
        ],
    );
    if mode == 1 {
        m.name = "BrokenHarnessDreamSealLateWrite".into();
    }
    if mode == 2 {
        m.name = "BrokenHarnessDreamSealChild".into();
    }
    act(
        &mut m,
        "AdmitProtectedEffect",
        all([eq(0, 0), eq(3, 0)]),
        &[(3, Int(1))],
    );
    act(
        &mut m,
        "CompleteObservedEffect",
        all([eq(0, 0), eq(3, 1)]),
        &[(3, Int(0))],
    );
    act(
        &mut m,
        "SpawnOwnedChild",
        all([eq(0, 0), eq(4, 0)]),
        &[(1, Int(1)), (4, Int(1))],
    );
    for coverage in 1..=2 {
        act(
            &mut m,
            &format!("ObserveOrEnforceChild{coverage}"),
            all([eq(0, 0), eq(1, 1)]),
            &[(2, Int(coverage))],
        );
    }
    for terminal in 2..=3 {
        act(
            &mut m,
            &format!("JoinOrReapChild{terminal}"),
            all([eq(0, 0), eq(1, 1)]),
            &[(1, Int(terminal))],
        );
    }
    let quiescent = eq(1, 0).or(all([Int(1).lt(Var(1)), Int(0).lt(Var(2))]));
    act(
        &mut m,
        "Freeze",
        all([
            eq(0, 0),
            eq(3, 0),
            if mode == 2 {
                Bool(true)
            } else {
                quiescent.clone()
            },
        ]),
        &[(0, Int(1))],
    );
    act(
        &mut m,
        "SealOwnCustody",
        eq(0, 1),
        &[(0, Int(2)), (6, Int(1))],
    );
    act(&mut m, "Close", eq(0, 2), &[(0, Int(3))]);
    if mode == 1 {
        act(
            &mut m,
            "BrokenLateProtectedWrite",
            Int(0).lt(Var(0)),
            &[(5, Int(1))],
        );
    }
    if mode == 2 {
        act(
            &mut m,
            "BrokenUnjoinedChildWrite",
            all([Int(0).lt(Var(0)), eq(1, 1)]),
            &[(5, Int(1))],
        );
    }
    safe(&mut m, "NoProtectedEffectsAfterFreeze", eq(5, 0));
    safe(&mut m, "FrozenOwnsNoMutableChild", eq(0, 0).or(quiescent));
    safe(&mut m, "FreezeHasNoInflightEffect", eq(0, 0).or(eq(3, 0)));
    m
}

// Decision d20261103136567: frozen caller observations at receive/return.
// Timer readiness is independent of the already elapsed absolute deadline.
// Concrete packets/ownership/reap remain the unchanged process families.
fn process_admission_result(broken: bool) -> Model {
    let mut m = base::<6>(
        "ProcessAdmissionResult",
        broken,
        &[
            ("cancelled", 1, 0),
            ("elapsed", 1, 0),
            ("caller_timer_ready", 1, 0),
            ("worker_deadline_observed", 1, 0),
            ("received", 3, 0), // pending, closed, success, explicit error
            ("returned", 4, 0), // pending, Interrupted, TimedOut, success, explicit error
        ],
    );
    // All inputs stop changing when returned is set: no later observation may
    // retroactively invalidate a correctly classified earlier result.
    act(
        &mut m,
        "ObserveCancellation",
        all([eq(5, 0), eq(0, 0)]),
        &[(0, Int(1))],
    );
    act(
        &mut m,
        "ObserveElapsedDeadline",
        all([eq(5, 0), eq(1, 0)]),
        &[(1, Int(1))],
    );
    act(
        &mut m,
        "CallerTimerBecomesReady",
        all([eq(5, 0), eq(1, 1), eq(2, 0)]),
        &[(2, Int(1))],
    );
    act(
        &mut m,
        "WorkerDeadlineClosesChannel",
        all([eq(5, 0), eq(1, 1), eq(4, 0)]),
        &[(3, Int(1)), (4, Int(1))],
    );
    for (name, kind) in [
        ("ChannelUnavailable", 1),
        ("ReceiveOwnedSuccess", 2),
        ("ReceiveExplicitError", 3),
    ] {
        act(&mut m, name, all([eq(5, 0), eq(4, 0)]), &[(4, Int(kind))]);
    }
    for (kind, class) in [(1, 1), (2, 3), (3, 4)] {
        if broken {
            act(
                &mut m,
                &format!("OldMapping{kind}"),
                all([eq(5, 0), eq(4, kind)]),
                &[(5, Int(class))],
            );
        } else {
            act(
                &mut m,
                &format!("ReturnCancelled{kind}"),
                all([eq(5, 0), eq(4, kind), eq(0, 1)]),
                &[(5, Int(1))],
            );
            act(
                &mut m,
                &format!("ReturnTimedOut{kind}"),
                all([eq(5, 0), eq(4, kind), eq(0, 0), eq(1, 1)]),
                &[(5, Int(2))],
            );
            act(
                &mut m,
                &format!("ReturnBeforeCutoff{kind}"),
                all([eq(5, 0), eq(4, kind), eq(0, 0), eq(1, 0)]),
                &[(5, Int(class))],
            );
        }
    }
    safe(
        &mut m,
        "ObservedCancellationHasPriority",
        eq(5, 0).or(eq(0, 0)).or(eq(5, 1)),
    );
    safe(
        &mut m,
        "ObservedElapsedResultIsTimedOut",
        eq(5, 0).or(eq(0, 1)).or(eq(1, 0)).or(eq(5, 2)),
    );
    let before = eq(4, 1)
        .and(eq(5, 1))
        .or(eq(4, 2).and(eq(5, 3)))
        .or(eq(4, 3).and(eq(5, 4)));
    safe(
        &mut m,
        "BeforeCutoffPreservesOutcome",
        eq(5, 0).or(eq(0, 1)).or(eq(1, 1)).or(before),
    );
    m
}
// WP7: two independent candidate claims; a claim outlives all owned arm cleanup.
// One child abstracts the serial with/without schedule; concrete counts are math tests.
fn dream_trial_claims(mode: u8) -> Model {
    let mut m = base::<10>(
        "DreamTrialClaims",
        false,
        &[
            ("phase_a", 4, 0),
            ("phase_b", 4, 0),
            ("claim_a", 1, 0),
            ("claim_b", 1, 0),
            ("owned_a", 1, 0),
            ("owned_b", 1, 0),
            ("caller_a", 2, 0),
            ("caller_b", 2, 0),
            ("busy_a", 1, 0),
            ("busy_b", 1, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialClaims{}",
            if mode == 1 { "Global" } else { "EarlyRelease" }
        );
    }
    for j in 0..2 {
        let (p, c, o, caller, busy) = (j, 2 + j, 4 + j, 6 + j, 8 + j);
        let claim_guard = all([
            eq(p, 0),
            eq(caller, 0),
            if mode == 1 {
                eq(2 + (1 - j), 0)
            } else {
                Bool(true)
            },
        ]);
        act(
            &mut m,
            &format!("Claim{j}"),
            claim_guard.clone(),
            &[(p, Int(1)), (c, Int(1))],
        );
        safe(
            &mut m,
            &format!("IndependentClaimAvailable{j}"),
            all([eq(p, 0), eq(caller, 0)]).negate().or(claim_guard),
        );
        act(
            &mut m,
            &format!("BusyReturnsWithoutWaiting{j}"),
            eq(c, 1),
            &[(busy, Int(1))],
        );
        act(
            &mut m,
            &format!("StartOneArm{j}"),
            all([eq(p, 1), eq(caller, 0), eq(o, 0)]),
            &[(p, Int(2)), (o, Int(1))],
        );
        act(
            &mut m,
            &format!("ArmReturnedStillOwnsCleanup{j}"),
            eq(p, 2),
            &[(p, Int(3))],
        );
        for (name, value) in [("Cancel", 1), ("Drop", 2), ("Deadline", 1)] {
            act(
                &mut m,
                &format!("{name}{j}"),
                eq(p, 1).or(eq(p, 2)),
                &[(caller, Int(value)), (p, Int(3))],
            );
        }
        let join = m.actions.len();
        act(
            &mut m,
            &format!("JoinOwnedCleanup{j}"),
            all([eq(p, 3), eq(o, 1)]),
            &[(o, Int(0))],
        );
        let release = m.actions.len();
        act(
            &mut m,
            &format!("ReleaseSettledClaim{j}"),
            all([eq(p, 3), if mode == 2 { Bool(true) } else { eq(o, 0) }]),
            &[(p, Int(4)), (c, Int(0))],
        );
        safe(
            &mut m,
            &format!("OwnedArmKeepsClaim{j}"),
            eq(o, 0).or(eq(c, 1)),
        );
        safe(
            &mut m,
            &format!("SerialArmHasClaim{j}"),
            eq(p, 2).negate().or(eq(c, 1)),
        );
        progress(
            &mut m,
            &format!("SettledCleanupReleasesClaim{j}"),
            eq(p, 3),
            eq(p, 4),
            vec![join, release],
        );
    }
    m
}
// WP7: new run bookkeeping is separate from existing lifecycle/owner models.
// A pass is active only for its exact pin/pair with observed isolation/usage/end/join.
fn dream_trial_outcome(mode: u8) -> Model {
    let mut m = base::<12>(
        "DreamTrialOutcome",
        false,
        &[
            ("phase", 4, 0),
            ("isolation", 2, 0),
            ("usage", 2, 0),
            ("ended", 1, 0),
            ("cleanup", 1, 0),
            ("entry_pin", 1, 0),
            ("harness_pin", 1, 0),
            ("terminal", 2, 0),
            ("caller", 2, 0),
            ("over_cap", 1, 0),
            ("score_pass", 1, 1),
            ("result", 4, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialOutcome{}",
            [
                "",
                "Isolation",
                "Harness",
                "Terminal",
                "Usage",
                "Entry",
                "Cleanup"
            ][usize::from(mode)]
        );
    }
    act(&mut m, "Prepare", eq(0, 0), &[(0, Int(1))]);
    act(
        &mut m,
        "EstablishIsolation",
        all([eq(0, 1), eq(1, 0)]),
        &[(1, Int(1))],
    );
    act(
        &mut m,
        "IsolationUnavailable",
        all([eq(0, 1), eq(1, 0)]),
        &[(1, Int(2))],
    );
    act(
        &mut m,
        "StartIsolatedArm",
        all([eq(0, 1), eq(1, 1), eq(7, 0), eq(8, 0)]),
        &[(0, Int(2))],
    );
    act(
        &mut m,
        "BrokenStartWithoutIsolation",
        all([Bool(mode == 1), eq(0, 1), eq(1, 2), eq(7, 0), eq(8, 0)]),
        &[(0, Int(2))],
    );
    act(
        &mut m,
        "UnavailableBeforeExecution",
        all([
            eq(0, 1),
            eq(1, 2).or(eq(7, 0).negate()).or(eq(8, 0).negate()),
        ]),
        &[(0, Int(4)), (11, Int(3))],
    );
    for (name, index, max) in [
        ("IsolationFailed", 1, 2),
        ("UsageKnown", 2, 1),
        ("UsageIncomparable", 2, 2),
        ("TurnEnded", 3, 1),
        ("ChangedEntryPin", 5, 1),
        ("WrongHarnessPin", 6, 1),
        ("SpendExceeded", 9, 1),
        ("ScoreFails", 10, 0),
    ] {
        let observation = if index == 2 && max == 1 {
            eq(2, 0)
        } else {
            eq(index, max).negate()
        };
        act(
            &mut m,
            name,
            all([eq(0, 2), observation]),
            &[(index, Int(max))],
        );
    }
    act(
        &mut m,
        "CleanupActuallyJoined",
        all([eq(0, 2), eq(3, 1).or(eq(8, 0).negate())]),
        &[(4, Int(1))],
    );
    act(
        &mut m,
        "CaptureReturnedObservations",
        eq(0, 2),
        &[(0, Int(3))],
    );
    for (name, value) in [
        ("CancelCaller", 1),
        ("DropCaller", 2),
        ("RunOrVerifyDeadline", 1),
    ] {
        act(
            &mut m,
            name,
            eq(0, 1).or(eq(0, 2)).or(eq(0, 3)),
            &[(8, Int(value))],
        );
    }
    // Result 1 is an active pass; 4 is a historical pass fenced by later terminal.
    for (name, value) in [("PairTerminal", 1), ("GlobalTerminal", 2)] {
        let allowed = if value == 1 {
            eq(7, 0)
        } else {
            eq(7, 2).negate()
        };
        act(
            &mut m,
            &format!("{name}BeforeOrAfterNonPass"),
            all([allowed.clone(), eq(11, 1).negate()]),
            &[(7, Int(value))],
        );
        act(
            &mut m,
            &format!("{name}FencesActivePass"),
            all([allowed, eq(11, 1)]),
            &[(7, Int(value)), (11, Int(if mode == 3 { 1 } else { 4 }))],
        );
    }
    let gates = |omit: usize| {
        all([
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (5, 0),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
        ]
        .into_iter()
        .filter(|(i, _)| *i != omit)
        .map(|(i, n)| eq(i, n)))
    };
    let full = gates(usize::MAX);
    act(
        &mut m,
        "PassQualifiedSamePair",
        all([eq(0, 3), full.clone(), eq(10, 1)]),
        &[(0, Int(4)), (11, Int(1))],
    );
    act(
        &mut m,
        "FailQualifiedTrial",
        all([eq(0, 3), full.clone(), eq(10, 0)]),
        &[(0, Int(4)), (11, Int(2))],
    );
    act(
        &mut m,
        "InconclusiveMissingGate",
        all([eq(0, 3), full.clone().negate()]),
        &[(0, Int(4)), (11, Int(3))],
    );
    let omitted = match mode {
        1 => 1,
        2 => 6,
        3 => 7,
        4 => 2,
        5 => 5,
        6 => 4,
        _ => usize::MAX,
    };
    act(
        &mut m,
        "BrokenAcceptIncompleteObservation",
        all([
            Bool(mode != 0),
            eq(0, 3),
            gates(omitted),
            full.clone().negate(),
            eq(10, 1),
        ]),
        &[(0, Int(4)), (11, Int(1))],
    );
    safe(
        &mut m,
        "ActivePassRequiresObservedGates",
        eq(11, 1).negate().or(full),
    );
    safe(
        &mut m,
        "NoTerminalActivePass",
        eq(11, 1).negate().or(eq(7, 0)),
    );
    m
}
// WP7 finite syscall-capability quotient: no kernel/Hasher/OS proof. A checked
// descriptor survives pathname substitution; root, leaf type, generation and
// execution denial remain separate facts. No external same-user root relocation.
fn dream_trial_capability(mode: u8) -> Model {
    let mut m = base::<10>(
        "DreamTrialCapability",
        mode != 0,
        &[
            ("phase", 4, 0),
            ("operand", 6, 0),
            ("root", 1, 1),
            ("live", 1, 1),
            ("fd_safe", 1, 0),
            ("substituted", 1, 0),
            ("forged_allow", 1, 0),
            ("outside", 1, 0),
            ("executed", 1, 0),
            ("effect", 1, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialCapability{}",
            ["", "Reopen", "Permit", "Exec"][mode as usize]
        );
    }
    for operand in 1..=6 {
        act(
            &mut m,
            "ChooseOperand",
            eq(0, 0),
            &[(0, Int(1)), (1, Int(operand))],
        );
    }
    act(&mut m, "WrongRoot", eq(0, 1), &[(2, Int(0))]);
    act(&mut m, "CancelOrStale", eq(0, 1), &[(3, Int(0))]);
    act(&mut m, "ForgedPermit", eq(0, 1), &[(6, Int(1))]);
    let valid = all([eq(1, 1), eq(2, 1), eq(3, 1)]);
    act(
        &mut m,
        "OpenCheckedDescriptor",
        eq(0, 1).and(valid.clone()),
        &[(0, Int(2)), (4, Int(1))],
    );
    if mode == 2 {
        act(
            &mut m,
            "ForgedOpen",
            all([eq(0, 1), eq(6, 1), valid.clone().negate()]),
            &[(0, Int(2)), (4, Int(0))],
        );
    }
    act(&mut m, "CancelAfterOpen", eq(0, 2), &[(3, Int(0))]);
    act(
        &mut m,
        "RefuseStaleEffect",
        all([eq(0, 2), eq(3, 0)]),
        &[(0, Int(4))],
    );
    act(&mut m, "SubstitutePathAfterOpen", eq(0, 2), &[(5, Int(1))]);
    let bad = eq(4, 0).or(if mode == 1 { eq(5, 1) } else { Bool(false) });
    act(
        &mut m,
        "ConfinedEffect",
        all([eq(0, 2), eq(3, 1), bad.clone().negate()]),
        &[(0, Int(3)), (9, Int(1))],
    );
    act(
        &mut m,
        "EscapedEffect",
        all([eq(0, 2), eq(3, 1), bad]),
        &[(0, Int(3)), (7, Int(1)), (9, Int(1))],
    );
    act(
        &mut m,
        "DenyExecMcp",
        all([eq(0, 1), eq(1, 6)]),
        &[(0, Int(3)), (8, Int(i64::from(mode == 3)))],
    );
    act(
        &mut m,
        "RejectUnsupported",
        eq(0, 1).and(valid.negate()),
        &[(0, Int(4))],
    );
    act(&mut m, "JoinTools", eq(0, 3), &[(0, Int(4))]);
    safe(&mut m, "NoOutsideReadWrite", eq(7, 0));
    safe(&mut m, "NoExecOrMcp", eq(8, 0));
    safe(&mut m, "NoStaleEffects", eq(9, 0).or(eq(3, 1)));
    m
}
// The opaque proof binds retained target-turn leaves, homogeneous known basis,
// exact aggregate and the current run. No arithmetic/byte/call-completeness proof.
fn dream_trial_usage(mode: u8) -> Model {
    let mut m = base::<7>(
        "DreamTrialUsage",
        mode != 0,
        &[
            ("phase", 3, 0),
            ("rows", 3, 0),
            ("basis", 3, 0),
            ("aggregate_matches", 1, 0),
            ("proof", 1, 0),
            ("run_matches", 1, 1),
            ("qualified", 1, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialUsage{}",
            ["", "Join", "Blind", "Basis"][mode as usize]
        );
    }
    for rows in 1..=3 {
        for basis in 1..=3 {
            for matches in 0..=1 {
                act(
                    &mut m,
                    "RetainCut",
                    eq(0, 0),
                    &[
                        (0, Int(1)),
                        (1, Int(rows)),
                        (2, Int(basis)),
                        (3, Int(matches)),
                    ],
                );
            }
        }
    }
    let valid = all([
        if mode == 1 { Bool(true) } else { eq(1, 1) },
        if mode == 3 { Bool(true) } else { eq(2, 1) },
        eq(3, 1),
    ]);
    act(
        &mut m,
        "CheckTargetLeavesAndSum",
        eq(0, 1).and(valid),
        &[(0, Int(2)), (4, Int(1))],
    );
    act(&mut m, "CopiedProofChangedRun", eq(0, 2), &[(5, Int(0))]);
    act(
        &mut m,
        "Qualify",
        eq(0, 2).and(if mode == 2 {
            Bool(true)
        } else {
            all([eq(4, 1), eq(5, 1)])
        }),
        &[(0, Int(3)), (6, Int(1))],
    );
    if mode == 2 {
        act(
            &mut m,
            "BlindDerivedTag",
            eq(0, 1),
            &[(0, Int(3)), (6, Int(1))],
        );
    }
    safe(
        &mut m,
        "QualificationRequiresBoundTargetProof",
        eq(6, 0).or(all([eq(4, 1), eq(1, 1), eq(2, 1), eq(3, 1), eq(5, 1)])),
    );
    m
}
// Result responsiveness is independent of settlement. No fairness of native
// spawn/wait is assumed; explicit no-child or successful owned reap is needed.
fn dream_trial_settlement(mode: u8) -> Model {
    let mut m = base::<7>(
        "DreamTrialSettlement",
        mode != 0,
        &[
            ("phase", 5, 0),
            ("result", 2, 0),
            ("caller_dropped", 1, 0),
            ("observer", 2, 0),
            ("child", 2, 0),
            ("resources", 1, 1),
            ("admission_closed", 1, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialSettlement{}",
            ["", "Err", "Channel", "Late"][mode as usize]
        );
    }
    act(&mut m, "AdmitNativeSpawn", eq(0, 0), &[(0, Int(1))]);
    act(
        &mut m,
        "ExplicitQueuedRefusal",
        eq(0, 0),
        &[(0, Int(5)), (1, Int(1)), (6, Int(1))],
    );
    act(
        &mut m,
        "ResultDeadline",
        all([Var(0).lt(Int(4)), eq(1, 0)]),
        &[(1, Int(1))],
    );
    act(&mut m, "CallerDrop", eq(2, 0), &[(2, Int(1))]);
    act(
        &mut m,
        "LateSpawnPacket",
        eq(0, 1),
        &[(0, Int(2)), (4, Int(1))],
    );
    act(
        &mut m,
        "DeliverResult",
        all([eq(0, 2), eq(1, 0)]),
        &[(1, Int(2))],
    );
    act(
        &mut m,
        "RetireSameOwner",
        eq(0, 2),
        &[(0, Int(3)), (6, Int(1))],
    );
    act(
        &mut m,
        "SuccessfulFencedWaitAndIoJoin",
        eq(0, 3),
        &[(0, Int(4)), (4, Int(2))],
    );
    act(&mut m, "LostAcknowledgement", eq(3, 0), &[(3, Int(2))]);
    act(
        &mut m,
        "ConfirmExplicitCompletion",
        all([eq(3, 0), eq(6, 1), eq(0, 4).or(eq(0, 5))]),
        &[(3, Int(1))],
    );
    act(&mut m, "ReleaseOwnedScratchClaim", eq(3, 1), &[(5, Int(0))]);
    if mode == 1 {
        act(&mut m, "ReleaseOnErr", eq(1, 1), &[(5, Int(0))]);
    }
    if mode == 2 {
        act(
            &mut m,
            "ChannelClosedIsNotAcknowledgement",
            eq(3, 2),
            &[(3, Int(1))],
        );
    }
    if mode == 3 {
        act(&mut m, "ReleaseBeforeLateSpawn", eq(0, 1), &[(5, Int(0))]);
    }
    let settled = all([eq(6, 1), eq(0, 4).or(eq(0, 5)), eq(4, 1).negate()]);
    safe(
        &mut m,
        "ConfirmedMeansJoinedOwnership",
        eq(3, 1).negate().or(settled.clone()),
    );
    safe(
        &mut m,
        "PendingOwnsResources",
        eq(5, 1).or(settled.and(eq(3, 1))),
    );
    m
}

// Internal opaque-capability fork: quiescent or never-started queued frontier.
// Exact bytes, syscalls and crash points require the real Store witnesses.
fn dream_trial_frontier(mode: u8) -> Model {
    let mut m = base::<12>(
        "DreamTrialFrontier",
        mode != 0,
        &[
            ("phase", 10, 0),
            ("frontier", 5, 0),
            ("pending", 2, 0),
            ("prefix", 1, 0),
            ("same_bytes", 1, 1),
            ("lease", 1, 0),
            ("epoch", 3, 0),
            ("ended_a", 2, 0),
            ("ended_b", 1, 0),
            ("crashed", 1, 0),
            ("fresh", 1, 0),
            ("success", 1, 0),
        ],
    );
    if mode != 0 {
        m.name = format!(
            "BrokenHarnessDreamTrialFrontier{}",
            ["", "Active", "Early", "Bytes", "Inherited", "Duplicate"][mode as usize]
        );
    }
    for frontier in 1..=5 {
        for pending in if frontier == 2 { 1..=2 } else { 0..=0 } {
            act(
                &mut m,
                "ObservePinnedCut",
                eq(0, 0),
                &[(0, Int(1)), (1, Int(frontier)), (2, Int(pending))],
            );
        }
    }
    let valid = if mode == 1 {
        Var(1).le(Int(4))
    } else {
        Var(1).le(Int(2))
    };
    act(
        &mut m,
        "ClassifyWithCompleteValidationView",
        eq(0, 1).and(valid),
        &[(0, Int(2)), (5, Int(1))],
    );
    act(
        &mut m,
        "PublishExactRawPrefix",
        eq(0, 2),
        &[
            (0, Int(3)),
            (3, Int(1)),
            (6, Int(1)),
            (4, Int(i64::from(mode != 3))),
        ],
    );
    let mut fair = vec![];
    fair.push(m.actions.len());
    act(
        &mut m,
        "QuiescentReady",
        all([eq(0, 3), eq(2, 0)]),
        &[(0, Int(5))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "ReserveChildEpochTwo",
        all([eq(0, 3), Int(0).lt(Var(2))]),
        &[(0, Int(4)), (6, Int(2))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "InterruptQueueAOnce",
        all([eq(0, 4), Int(0).lt(Var(2)), eq(7, 0)]),
        &[(7, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "InterruptQueueBOnce",
        all([eq(0, 4), eq(2, 2), eq(8, 0)]),
        &[(8, Int(1))],
    );
    let recovered = all([eq(2, 0).or(eq(7, 1)), Var(2).lt(Int(2)).or(eq(8, 1))]);
    fair.push(m.actions.len());
    act(
        &mut m,
        "PersistRecoveryBeforeExposure",
        eq(0, 4).and(recovered.clone()),
        &[(0, Int(5))],
    );
    act(
        &mut m,
        "CrashAfterPrefixOrPartialSuffix",
        all([eq(9, 0), eq(0, 3).or(eq(0, 4)).or(eq(0, 5))]),
        &[(0, Int(9)), (5, Int(0)), (9, Int(1))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "OrdinaryReopenReservesNextEpoch",
        eq(0, 9),
        &[(0, Int(4)), (5, Int(1)), (6, Var(6).plus(Int(1)))],
    );
    fair.push(m.actions.len());
    act(
        &mut m,
        "PersistenceUnavailable",
        eq(0, 3).or(eq(0, 4)),
        &[(0, Int(10))],
    );
    act(&mut m, "ExposeActor", eq(0, 5), &[(0, Int(6))]);
    act(
        &mut m,
        "FreshContinuationOnly",
        eq(0, 6),
        &[(0, Int(7)), (10, Int(1))],
    );
    act(
        &mut m,
        "MatchFreshTurnUsage",
        eq(0, 7),
        &[(0, Int(8)), (11, Int(1))],
    );
    if mode == 2 {
        act(
            &mut m,
            "ExposePublishedPrefixEarly",
            eq(0, 3).and(eq(2, 2)),
            &[(0, Int(6))],
        );
    }
    if mode == 4 {
        act(
            &mut m,
            "CountInheritedRecoveryAsSuccess",
            all([eq(0, 4), eq(7, 1)]),
            &[(11, Int(1))],
        );
    }
    if mode == 5 {
        act(
            &mut m,
            "DuplicateInterruptedEnding",
            all([eq(0, 4), eq(7, 1)]),
            &[(7, Int(2))],
        );
    }
    safe(
        &mut m,
        "OnlyAcceptedFrontier",
        eq(3, 0).or(Var(1).le(Int(2))),
    );
    safe(&mut m, "InheritedBytesUnchanged", eq(4, 1));
    safe(
        &mut m,
        "NoActorBeforeRecovery",
        Var(0).lt(Int(6)).or(eq(0, 9)).or(eq(0, 10)).or(all([
            recovered,
            eq(5, 1),
            eq(2, 0).or(Int(1).lt(Var(6))),
        ])),
    );
    safe(&mut m, "EachQueueEndsOnce", Var(7).le(Int(1)));
    safe(
        &mut m,
        "FreshSuccessHasFreshIdentity",
        eq(11, 0).or(all([eq(10, 1), eq(0, 8)])),
    );
    progress(
        &mut m,
        "LocalRecoverySettles",
        eq(0, 3).or(eq(0, 4)).or(eq(0, 9)),
        eq(0, 5)
            .or(eq(0, 6))
            .or(eq(0, 7))
            .or(eq(0, 8))
            .or(eq(0, 10)),
        fair,
    );
    m
}

// WP8 finite interfaces, before stateful implementation. One invocation/route,
// one target, one external hand writer and zero/one delayed effect; no OS or IO
// fairness is assumed. Runtime/transport/process witnesses remain separate.
fn wp8_flush(mode: u8) -> Model {
    let name = [
        "DreamFlush",
        "DreamFlushReplay",
        "DreamFlushUncommitted",
        "DreamFlushRoot",
    ][mode as usize];
    let mut m = base(
        name,
        mode != 0,
        &[
            ("phase", 6, 0),
            ("write_calls", 2, 0),
            ("committed", 1, 0),
            ("wrong_root", 1, 0),
            ("pinned", 1, 0),
        ],
    );
    act(&mut m, "WriteOnce", eq(0, 0), &[(0, Int(1)), (1, Int(1))]);
    act(&mut m, "QueryOriginalWrite", eq(0, 1), &[(0, Int(2))]);
    act(&mut m, "CommitOnce", eq(0, 2), &[(0, Int(3))]);
    act(
        &mut m,
        "QueryOriginalCommit",
        eq(0, 3),
        &[(0, Int(4)), (2, Int(1))],
    );
    act(
        &mut m,
        "CommitFailed",
        eq(0, 3),
        &[(0, Int(if mode == 2 { 4 } else { 6 }))],
    );
    act(&mut m, "UnavailableNoOp", eq(0, 0), &[(0, Int(6))]);
    if mode == 1 {
        act(
            &mut m,
            "ReplayUnknownWrite",
            all([eq(0, 1), Var(1).lt(Int(2))]),
            &[(1, Var(1).plus(Int(1)))],
        );
    }
    if mode == 3 {
        act(&mut m, "WrongRootReply", eq(0, 4), &[(3, Int(1))]);
    }
    let mut pin = eq(0, 4);
    if mode != 2 {
        pin = pin.and(eq(2, 1));
    }
    if mode != 3 {
        pin = pin.and(eq(3, 0));
    }
    act(&mut m, "EmitCommittedPin", pin, &[(0, Int(5)), (4, Int(1))]);
    safe(&mut m, "UnknownEffectsNeverRedispatched", Var(1).le(Int(1)));
    safe(
        &mut m,
        "PinRequiresExactConfirmedCommit",
        eq(4, 0).or(all([eq(2, 1), eq(3, 0), eq(0, 5)])),
    );
    m
}
fn wp8_publication(mode: u8) -> Model {
    let name = [
        "DreamPublication",
        "DreamPublicationClobber",
        "DreamPublicationLate",
        "DreamPublicationSync",
        "DreamPublicationAdoption",
        "DreamPublicationAbandon",
    ][mode as usize];
    let mut m = base(
        name,
        mode != 0,
        &[
            ("phase", 8, 0),
            ("target", 2, 0),
            ("hand_written", 1, 0),
            ("hand_preserved", 1, 1),
            ("published", 1, 0),
            ("dir_synced", 1, 0),
            ("frozen", 1, 0),
            ("source_mutated", 1, 0),
            ("effective", 1, 0),
            ("current", 1, 1),
            ("approved", 1, 1),
            ("inflight", 1, 0),
            ("caller", 1, 1),
            ("cancelled", 1, 0),
            ("owner_retained", 1, 1),
        ],
    );
    for (from, label) in [
        (0, "Read"),
        (1, "CreateOwnTemp"),
        (2, "WriteOnce"),
        (3, "FsyncTemp"),
    ] {
        act(&mut m, label, eq(0, from), &[(0, Int(from + 1))]);
    }
    act(
        &mut m,
        "HandWriteBetweenReadAndLink",
        all([Int(0).lt(Var(0)), Var(0).le(Int(4)), eq(2, 0)]),
        &[(1, Int(2)), (2, Int(1))],
    );
    act(
        &mut m,
        "LinkToAbsent",
        all([eq(0, 4), eq(1, 0)]),
        &[(0, Int(5)), (1, Int(1)), (4, Int(1))],
    );
    act(
        &mut m,
        "ConflictSkip",
        all([eq(0, 4), eq(1, 2)]),
        &[(0, Int(6))],
    );
    if mode == 1 {
        act(
            &mut m,
            "BrokenOverwriteRename",
            all([eq(0, 4), eq(1, 2)]),
            &[(0, Int(5)), (1, Int(1)), (3, Int(0)), (4, Int(1))],
        );
    }
    act(
        &mut m,
        "DirectorySync",
        eq(0, 5),
        &[(0, Int(6)), (5, Int(1))],
    );
    if mode == 3 {
        act(&mut m, "OmitDirectorySync", eq(0, 5), &[(0, Int(6))]);
    }
    act(
        &mut m,
        "StartObservedWork",
        all([Var(0).lt(Int(6)), eq(11, 0)]),
        &[(11, Int(1))],
    );
    act(&mut m, "JoinObservedWork", eq(11, 1), &[(11, Int(0))]);
    let mut freeze = all([eq(0, 6), eq(11, 0)]);
    if mode != 3 {
        freeze = freeze.and(eq(4, 0).or(eq(5, 1)));
    }
    act(&mut m, "FreezeReceipt", freeze, &[(0, Int(7)), (6, Int(1))]);
    act(
        &mut m,
        "TerminalFence",
        eq(9, 1),
        &[(9, Int(0)), (8, Int(0))],
    );
    act(
        &mut m,
        "ApprovalUnavailable",
        eq(10, 1),
        &[(10, Int(0)), (8, Int(0))],
    );
    act(
        &mut m,
        "AdoptAfterDurableReceipt",
        all([eq(0, 7), eq(4, 1), eq(5, 1), eq(9, 1), eq(10, 1)]),
        &[(0, Int(8)), (8, Int(1))],
    );
    act(&mut m, "LabelSkipOrUnproven", eq(0, 7), &[(0, Int(8))]);
    if mode == 4 {
        act(
            &mut m,
            "BrokenFalseAdoption",
            eq(0, 7),
            &[(0, Int(8)), (8, Int(1))],
        );
    }
    if mode == 2 {
        act(&mut m, "BrokenLateSourceWrite", eq(6, 1), &[(7, Int(1))]);
    }
    act(
        &mut m,
        "CallerDropRequestsCancellation",
        eq(12, 1),
        &[(12, Int(0)), (13, Int(1))],
    );
    act(
        &mut m,
        "ReleaseOnlySettledOwner",
        all([eq(0, 8), eq(11, 0)]),
        &[(14, Int(0))],
    );
    if mode == 5 {
        act(
            &mut m,
            "BrokenDropDestroysInflightOwner",
            all([eq(12, 0), eq(11, 1)]),
            &[(14, Int(0))],
        );
    }
    safe(
        &mut m,
        "RetainOwnerUntilSettlement",
        eq(11, 0).or(eq(14, 1)),
    );
    safe(&mut m, "NoHandClobber", eq(3, 1));
    safe(&mut m, "FrozenHasNoLateSourceMutation", eq(7, 0));
    safe(
        &mut m,
        "SuccessRequiresDirectorySync",
        eq(6, 0).or(eq(4, 0)).or(eq(5, 1)),
    );
    safe(
        &mut m,
        "EffectiveAdoptionRequiresReceiptAndCurrency",
        eq(8, 0).or(all([
            eq(0, 8),
            eq(4, 1),
            eq(5, 1),
            eq(6, 1),
            eq(9, 1),
            eq(10, 1),
        ])),
    );
    m
}
fn wp8_recall(mode: u8) -> Model {
    let name = [
        "DreamRecall",
        "DreamRecallAppend",
        "DreamRecallPayload",
        "DreamRecallTrial",
        "DreamRecallTerminal",
    ][mode as usize];
    let mut m = base(
        name,
        mode != 0,
        &[
            ("phase", 4, 0),
            ("pin_ok", 1, 1),
            ("quoted_exact", 1, 1),
            ("trial", 1, 0),
            ("current", 1, 1),
            ("logged", 1, 0),
            ("injected", 1, 0),
        ],
    );
    act(&mut m, "TrialLaunch", eq(0, 0), &[(3, Int(1))]);
    act(&mut m, "InvalidatePin", eq(0, 0), &[(1, Int(0))]);
    act(
        &mut m,
        "TerminalBeforeInsertion",
        Var(0).lt(Int(2)),
        &[(4, Int(0))],
    );
    act(
        &mut m,
        "AppendRecall",
        eq(0, 0),
        &[(0, Int(1)), (5, Int(1))],
    );
    act(&mut m, "AppendFailureOmits", eq(0, 0), &[(0, Int(4))]);
    if mode == 1 {
        act(&mut m, "BrokenAssumeAppend", eq(0, 0), &[(0, Int(1))]);
    }
    if mode == 2 {
        act(&mut m, "ForgedPayload", eq(0, 0), &[(2, Int(0))]);
    }
    let mut guard = all([eq(0, 1), eq(1, 1)]);
    if mode != 1 {
        guard = guard.and(eq(5, 1));
    }
    if mode != 2 {
        guard = guard.and(eq(2, 1));
    }
    if mode != 3 {
        guard = guard.and(eq(3, 0));
    }
    if mode != 4 {
        guard = guard.and(eq(4, 1));
    }
    act(
        &mut m,
        "InsertQuotedUserData",
        guard,
        &[(0, Int(2)), (6, Int(1))],
    );
    act(&mut m, "DispatchActualRequest", eq(0, 2), &[(0, Int(3))]);
    act(&mut m, "OmitUnavailable", eq(0, 1), &[(0, Int(4))]);
    safe(
        &mut m,
        "InsertionRequiresDurableExactCurrentNonTrialPin",
        eq(6, 0).or(all([eq(1, 1), eq(2, 1), eq(3, 0), eq(4, 1), eq(5, 1)])),
    );
    m
}

#[path = "dream_c1.rs"]
mod dream_c1;
#[path = "dream_read.rs"]
mod dream_read;
pub fn corpus() -> Vec<(Model, &'static str)> {
    let mut out = vec![
        (dream_c1::bootstrap_entry(0), "pass"),
        (dream_c1::bootstrap_entry(1), "safety"),
        (dream_c1::bootstrap_entry(2), "safety"),
        (dream_c1::bootstrap_entry(3), "safety"),
        (dream_c1::bootstrap_entry(4), "safety"),
        (dream_c1::bootstrap_handoff(0), "pass"),
        (dream_c1::bootstrap_handoff(1), "safety"),
        (dream_c1::bootstrap_handoff(2), "safety"),
        (dream_c1::bootstrap_handoff(3), "safety"),
        (dream_c1::bootstrap_handoff(4), "safety"),
        (dream_c1::bootstrap_handoff(5), "safety"),
        (dream_c1::bootstrap_handoff(6), "safety"),
        (dream_c1::bootstrap_handoff(7), "safety"),
        (dream_c1::bootstrap_handoff(8), "safety"),
        (dream_c1::bootstrap_handoff(9), "safety"),
        (dream_c1::bootstrap_handoff(10), "safety"),
        (dream_c1::bootstrap_handoff(11), "safety"),
        (dream_c1::bootstrap_handoff(12), "safety"),
        (dream_c1::bootstrap_handoff(13), "safety"),
        (dream_c1::bootstrap_handoff(14), "safety"),
        (dream_c1::bootstrap_handoff(15), "safety"),
        (dream_c1::bootstrap_handoff(16), "safety"),
        (dream_c1::bootstrap_handoff(17), "safety"),
        (dream_c1::quantum_debt(0), "pass"),
        (dream_c1::quantum_debt(1), "safety"),
        (dream_c1::quantum_debt(2), "safety"),
        (dream_c1::quantum_debt(3), "safety"),
        (dream_c1::quantum_debt(4), "safety"),
        (dream_c1::quantum_debt(5), "safety"),
        (dream_c1::quantum_debt(6), "safety"),
        (dream_c1::quantum_debt(7), "safety"),
        (dream_c1::quantum_debt(8), "safety"),
        (dream_c1::quantum_debt(9), "safety"),
        (dream_c1::quantum_debt(10), "safety"),
        (dream_c1::quantum_debt(11), "safety"),
        (dream_read::private_verdicts(0), "pass"),
        (dream_read::private_verdicts(1), "safety"),
        (dream_read::private_verdicts(2), "safety"),
        (dream_read::private_verdicts(3), "safety"),
        (dream_read::private_verdicts(4), "safety"),
        (dream_read::private_verdicts(5), "safety"),
        (dream_read::private_verdicts(6), "liveness"),
        (dream_read::failed_stop_terminal(0), "pass"),
        (dream_read::failed_stop_terminal(1), "safety"),
        (dream_read::failed_stop_terminal(2), "safety"),
        (dream_read::failed_stop_terminal(3), "safety"),
        (dream_read::failed_stop_terminal(4), "safety"),
        (dream_read::failed_stop_terminal(5), "safety"),
        (dream_read::failed_stop_terminal(6), "safety"),
        (dream_read::failed_stop_terminal(7), "safety"),
        (dream_read::failed_stop_terminal(8), "safety"),
        (dream_read::failed_stop_terminal(9), "safety"),
        (dream_read::failed_stop_terminal(10), "safety"),
        (dream_read::failed_stop_terminal(11), "safety"),
        (dream_read::failed_stop_terminal(12), "safety"),
        (dream_read::failed_stop_terminal(13), "safety"),
        (dream_read::failed_stop_terminal(14), "safety"),
        (dream_read::failed_stop_terminal(15), "safety"),
        (dream_read::failed_stop_terminal(16), "safety"),
        (dream_read::failed_stop_terminal(17), "safety"),
        (dream_read::quarantine_owner(0), "pass"),
        (dream_read::quarantine_owner(1), "safety"),
        (dream_read::quarantine_owner(2), "safety"),
        (dream_read::close_receive(0), "pass"),
        (dream_read::close_receive(1), "safety"),
        (dream_read::close_retirement(0), "pass"),
        (dream_read::close_retirement(1), "safety"),
        (dream_read::close_retirement(2), "safety"),
        (dream_read::closure_codec(0), "pass"),
        (dream_read::closure_codec(1), "safety"),
        (dream_read::closure_codec(2), "safety"),
        (dream_read::closure_codec(3), "safety"),
        (dream_read::closure_codec(4), "safety"),
        (dream_read::closure_codec(5), "safety"),
        (dream_read::closure_codec(6), "safety"),
        (dream_read::closure_codec(7), "safety"),
        (dream_read::closure_codec(8), "safety"),
        (dream_read::git_sig_temp(0), "pass"),
        (dream_read::git_sig_temp(1), "safety"),
        (dream_read::git_sig_temp(2), "safety"),
        (dream_read::git_sig_temp(3), "safety"),
        (dream_read::git_sig_temp(4), "safety"),
        (dream_read::git_sig_temp(5), "safety"),
        (dream_read::git_sig_temp(6), "safety"),
        (dream_read::git_sig_temp(7), "safety"),
        (dream_read::git_sig_temp(8), "safety"),
        (dream_read::git_sig_temp(9), "safety"),
        (dream_read::helper(0), "pass"),
        (dream_read::helper(1), "safety"),
        (dream_read::helper(2), "safety"),
        (dream_read::helper(3), "safety"),
        (dream_read::helper(4), "safety"),
        (dream_read::helper(9), "safety"),
        (dream_read::helper(10), "safety"),
        (dream_read::helper(11), "safety"),
        (dream_read::helper(12), "safety"),
        (dream_read::helper(13), "safety"),
        (dream_read::helper(14), "safety"),
        (dream_read::helper(16), "safety"),
        (dream_read::helper(17), "safety"),
        (dream_read::helper(18), "safety"),
        (dream_read::helper(19), "safety"),
        (dream_read::helper(20), "safety"),
        (dream_read::helper(21), "safety"),
        (dream_read::helper(22), "safety"),
        (dream_read::helper(23), "safety"),
        (dream_read::verifier_event_discovery(0), "pass"),
        (dream_read::verifier_event_discovery(1), "safety"),
        (dream_read::verifier_event_discovery(2), "safety"),
        (dream_read::verifier_event_discovery(3), "safety"),
        (dream_read::verifier_event_discovery(4), "safety"),
        (dream_read::verifier_event_discovery(5), "safety"),
        (dream_read::verifier_event_discovery(6), "safety"),
        (dream_read::verifier_event_discovery(7), "safety"),
        (dream_read::helper_exec(0), "pass"),
        (dream_read::helper_exec(1), "safety"),
        (dream_read::helper_exec(2), "safety"),
        (dream_read::helper_exec(3), "safety"),
        (dream_read::helper_exec(4), "safety"),
        (dream_read::helper_exec(5), "safety"),
        (dream_read::helper_exec(6), "safety"),
        (dream_read::helper_exec(7), "safety"),
        (dream_read::total_budget(0), "pass"),
        (dream_read::total_budget(1), "safety"),
        (dream_read::total_budget(2), "safety"),
        (dream_read::total_budget(3), "safety"),
        (dream_read::total_budget(4), "safety"),
        (dream_read::total_budget(5), "safety"),
        (dream_read::total_budget(6), "safety"),
        (dream_read::total_budget(7), "safety"),
        (dream_read::total_budget(8), "safety"),
        (dream_read::total_budget(9), "safety"),
        (dream_read::total_budget(10), "safety"),
        (dream_read::total_budget(11), "safety"),
        (dream_read::total_budget(12), "safety"),
        (dream_read::total_budget(13), "safety"),
        (dream_read::total_budget(14), "safety"),
        (dream_read::total_budget(15), "safety"),
        (dream_read::total_budget(16), "safety"),
        (dream_read::total_budget(17), "safety"),
        (dream_read::total_budget(18), "safety"),
        (dream_read::total_budget(19), "safety"),
        (dream_read::foreground_stop_terminal(0), "pass"),
        (dream_read::foreground_stop_terminal(1), "safety"),
        (dream_read::systematic_observe_terminal(0), "pass"),
        (dream_read::systematic_observe_terminal(1), "safety"),
        (dream_read::qualification(0), "pass"),
        (dream_read::qualification(1), "pass"),
        (dream_read::qualification(2), "safety"),
        (dream_read::qualification(3), "safety"),
        (dream_read::qualification(4), "safety"),
        (dream_read::qualification(5), "safety"),
        (dream_read::qualification(6), "safety"),
        (dream_read::qualification(7), "safety"),
        (dream_read::qualification(8), "safety"),
        (dream_read::qualification(9), "safety"),
        (dream_read::qualification(10), "safety"),
        (dream_read::qualification(11), "safety"),
        (dream_read::qualification(12), "safety"),
        (dream_read::qualification(13), "safety"),
        (dream_read::qualification(14), "safety"),
        (dream_read::qualification(15), "safety"),
        (dream_read::qualification(16), "safety"),
        (dream_read::qualification(17), "safety"),
        (dream_read::qualification(18), "safety"),
        (dream_read::qualification(19), "safety"),
        (dream_read::qualification(20), "safety"),
        (dream_read::qualification(21), "safety"),
        (dream_read::qualification(22), "safety"),
        (dream_read::qualification(23), "safety"),
        (dream_read::qualification(24), "safety"),
        (dream_read::qualification(25), "safety"),
        (dream_read::qualification(26), "safety"),
        (dream_read::qualification(27), "safety"),
        (dream_read::qualification(28), "safety"),
        (dream_read::qualification(29), "safety"),
        (dream_read::qualification(30), "safety"),
        (dream_read::qualification(31), "safety"),
        (dream_read::qualification(32), "safety"),
        (dream_read::qualification(33), "safety"),
        (dream_read::qualification(34), "safety"),
        (dream_read::qualification(35), "safety"),
        (dream_read::qualification(36), "safety"),
        (dream_read::qualification(37), "safety"),
        (dream_read::qualification(38), "safety"),
        (dream_read::qualification(39), "safety"),
        (dream_read::qualification(40), "safety"),
        (dream_read::qualification(41), "safety"),
        (dream_read::eligibility(0), "pass"),
        (dream_read::eligibility(1), "safety"),
        (dream_read::eligibility(2), "safety"),
        (dream_read::eligibility(3), "safety"),
        (dream_read::ownership(0), "pass"),
        (dream_read::ownership(1), "safety"),
        (dream_read::ownership(2), "safety"),
        (dream_read::ownership(3), "safety"),
        (dream_read::ownership(4), "safety"),
        (dream_read::snapshot(0), "pass"),
        (dream_read::snapshot(1), "safety"),
        (dream_read::snapshot(2), "safety"),
        (dream_c1::returned_owner(0), "pass"),
        (dream_c1::returned_owner(1), "safety"),
        (dream_c1::returned_owner(2), "safety"),
        (dream_c1::returned_owner(3), "safety"),
        (dream_c1::public_metadata(0), "pass"),
        (dream_c1::public_metadata(1), "safety"),
        (dream_c1::public_metadata(2), "safety"),
        (dream_c1::retained_host(false), "pass"),
        (dream_c1::retained_host(true), "safety"),
        (dream_c1::storage(false), "pass"),
        (dream_c1::storage(true), "safety"),
        (dream_c1::inspection(0), "pass"),
        (dream_c1::inspection(1), "safety"),
        (dream_c1::inspection(2), "safety"),
        (dream_c1::inspection(3), "safety"),
        (dream_c1::inspection(4), "safety"),
        (dream_c1::worker(0), "pass"),
        (dream_c1::worker(1), "safety"),
        (dream_c1::worker(2), "safety"),
        (dream_c1::worker(3), "safety"),
        (python_execution::model(0), "pass"),
        (python_execution::model(1), "safety"),
        (python_execution::model(2), "safety"),
        (python_execution::model(3), "safety"),
        (python_execution::model(4), "safety"),
        (python_execution::model(5), "safety"),
        (python_execution::model(6), "safety"),
        (python_execution::bounded_wait(0), "pass"),
        (python_execution::bounded_wait(1), "safety"),
        (python_execution::bounded_wait(2), "safety"),
        (python_execution::catalogue(0), "pass"),
        (python_execution::catalogue(1), "safety"),
        (python_execution::catalogue(2), "safety"),
        (python_execution::capacity(false), "pass"),
        (python_execution::capacity(true), "liveness"),
        (unrestricted_startup(false), "pass"),
        (unrestricted_startup(true), "safety"),
        (draft_handoff(false), "pass"),
        (draft_handoff(true), "safety"),
        (wp8_flush(0), "pass"),
        (wp8_flush(1), "safety"),
        (wp8_flush(2), "safety"),
        (wp8_flush(3), "safety"),
        (wp8_publication(0), "pass"),
        (wp8_publication(1), "safety"),
        (wp8_publication(2), "safety"),
        (wp8_publication(3), "safety"),
        (wp8_publication(4), "safety"),
        (wp8_publication(5), "safety"),
        (wp8_recall(0), "pass"),
        (wp8_recall(1), "safety"),
        (wp8_recall(2), "safety"),
        (wp8_recall(3), "safety"),
        (wp8_recall(4), "safety"),
        (dream_trial_frontier(0), "pass"),
        (dream_trial_frontier(1), "safety"),
        (dream_trial_frontier(2), "safety"),
        (dream_trial_frontier(3), "safety"),
        (dream_trial_frontier(4), "safety"),
        (dream_trial_frontier(5), "safety"),
        (dream_trial_capability(0), "pass"),
        (dream_trial_capability(1), "safety"),
        (dream_trial_capability(2), "safety"),
        (dream_trial_capability(3), "safety"),
        (dream_trial_usage(0), "pass"),
        (dream_trial_usage(1), "safety"),
        (dream_trial_usage(2), "safety"),
        (dream_trial_usage(3), "safety"),
        (dream_trial_settlement(0), "pass"),
        (dream_trial_settlement(1), "safety"),
        (dream_trial_settlement(2), "safety"),
        (dream_trial_settlement(3), "safety"),
        (dream_trial_claims(0), "pass"),
        (dream_trial_claims(1), "safety"),
        (dream_trial_claims(2), "safety"),
        (dream_trial_outcome(0), "pass"),
        (dream_trial_outcome(1), "safety"),
        (dream_trial_outcome(2), "safety"),
        (dream_trial_outcome(3), "safety"),
        (dream_trial_outcome(4), "safety"),
        (dream_trial_outcome(5), "safety"),
        (dream_trial_outcome(6), "safety"),
        (process_admission_result(false), "pass"),
        (process_admission_result(true), "safety"),
        (dream_seal(0), "pass"),
        (dream_seal(1), "safety"),
        (dream_seal(2), "safety"),
        (dream_native_custody(false), "pass"),
        (dream_native_custody(true), "safety"),
        (dream_d0(0), "pass"),
        (dream_d0(1), "safety"),
        (dream_d0(2), "liveness"),
        (dream_sink(0), "pass"),
        (dream_sink(1), "safety"),
        (dream_sink(2), "safety"),
        (dream_sink(3), "safety"),
        (dream_lifecycle(0), "pass"),
        (dream_lifecycle(1), "safety"),
        (dream_lifecycle(2), "safety"),
        (dream_lifecycle(3), "safety"),
        (dream_merge(0), "pass"),
        (dream_merge(1), "safety"),
        (dream_merge(2), "safety"),
        (scroll_resume_control(), "safety"),
        (acp_steering(0), "pass"),
        (acp_steering(1), "safety"),
        (acp_steering(2), "safety"),
        (acp_controls(0), "pass"),
        (acp_controls(1), "safety"),
        (acp_controls(2), "safety"),
        (attachment_capability(false), "pass"),
        (attachment_capability(true), "safety"),
        (attachment_discovery_broken(), "safety"),
        (onboarding_install(false), "pass"),
        (onboarding_install(true), "safety"),
        (attachment_intake(false), "pass"),
        (attachment_intake(true), "safety"),
        (gui_pending(0), "pass"),
        (gui_pending(1), "safety"),
        (gui_pending(2), "safety"),
        (socket_heartbeat(false), "pass"),
        (socket_heartbeat(true), "safety"),
        (four_view_budget(false), "pass"),
        (four_view_budget(true), "safety"),
        (socket_lifecycle(0), "pass"),
        (socket_lifecycle(1), "safety"),
        (socket_lifecycle(2), "safety"),
        (socket_lifecycle(3), "safety"),
        (socket_lifecycle(4), "safety"),
        (socket_lifecycle(5), "liveness"),
        (socket_delivery(0), "pass"),
        (socket_delivery(1), "safety"),
        (socket_delivery(2), "safety"),
        (agent_path(0), "pass"),
        (agent_path(1), "safety"),
        (agent_path(2), "liveness"),
        (agent_path(3), "safety"),
        (gui_markdown(0), "pass"),
        (gui_markdown(1), "safety"),
        (gui_markdown(2), "safety"),
        (gui_markdown(3), "liveness"),
        (gui_choice(false), "pass"),
        (gui_choice(true), "safety"),
        (seat_report(0), "pass"),
        (seat_report(1), "safety"),
        (seat_report(2), "safety"),
        (seat_report(3), "safety"),
        (seat_report(4), "safety"),
        (seat_report(5), "safety"),
        (acp_agent_turn(2), "safety"),
        (acp_agent_turn(3), "safety"),
    ];
    for b in [false, true] {
        for m in [
            lifecycle_mode(b, false),
            lifecycle_mode(b, true),
            cancel_tree(b),
            cancel_group(b),
            steer(b),
            control_bound(b),
            approval(b, false),
            approval(b, true),
            replay(b),
            replay_drop(b),
            replay_healthy(b),
            stream_to_view(b),
            dirty_wake_frame(b),
            scroll_stick(b),
            acp_agent_turn(u8::from(b)),
            native_usage(if b { 1 } else { 0 }),
            native_spill(b),
            folded_card(b),
            embed_lifecycle(b),
            profile_load(b),
            fork_materialize(b),
            text_zoom(b),
        ] {
            let expected = if !b {
                "pass"
            } else if [
                "BrokenHarnessSteerQueue",
                "BrokenHarnessApprovalUnattended",
                "BrokenHarnessSubscriberReplayHealthy",
            ]
            .contains(&m.name.as_str())
            {
                "liveness"
            } else {
                "safety"
            };
            out.push((m, expected));
        }
    }
    for mode in 0..=3 {
        out.push((
            auto_compact(mode),
            match mode {
                0 => "pass",
                3 => "liveness",
                _ => "safety",
            },
        ));
        out.push((
            mcp_server(mode),
            match mode {
                0 => "pass",
                1 => "deadlock",
                2 => "safety",
                _ => "liveness",
            },
        ));
    }
    out.push((auto_compact(4), "safety"));
    out.push((mcp_server(4), "safety"));
    out.push((mcp_server(5), "safety"));
    out.push((acp_flow(0), "pass"));
    out.push((acp_flow(1), "safety"));
    out.push((acp_flow(2), "safety"));
    out.push((acp_flow(3), "safety"));
    out.push((mcp_group_close(false), "pass"));
    out.push((mcp_group_close(true), "safety"));
    out.push((steer_mode(false, true), "safety"));
    let mut registration = replay_base(false);
    registration.name = "HarnessSubscriberReplayRegistration".into();
    out.push((registration, "pass"));
    let mut registration = replay_base(true);
    registration.name = "BrokenHarnessSubscriberReplayRegistration".into();
    out.push((registration, "safety"));
    out.push((replay_tail(2), "safety"));
    out.push((store_writer_lease(0), "pass"));
    out.push((store_writer_lease(1), "safety"));
    out.push((store_writer_lease(2), "safety"));
    out.push((store_writer_lease(3), "safety"));
    out.push((process_spawn_admission(0), "pass"));
    out.push((process_spawn_admission(1), "safety"));
    out.push((process_spawn_admission(2), "safety"));
    out.push((process_lifecycle(0), "pass"));
    out.push((process_lifecycle(1), "liveness"));
    out.push((process_lifecycle(2), "safety"));
    out.push((process_lifecycle(3), "safety"));
    out.push((process_lifecycle(4), "safety"));
    out.push((process_lifecycle(5), "safety"));
    out.push((seat_shutdown(0), "pass"));
    out.push((seat_shutdown(1), "safety"));
    out.push((seat_shutdown(2), "safety"));
    out.push((mcp_group_close_mode(false, true), "safety"));
    for mode in 0..=3 {
        out.push((
            provider_stream(mode),
            if mode == 0 { "pass" } else { "safety" },
        ));
    }
    out.push((native_usage(2), "safety"));
    use bbt_yarn::reactor_model;
    let mut projected = bboxy450_harness::harness_turn_topology!(
        reactor_model,
        sources,
        event,
        { Ok::<(), String>(()) },
        { Ok(bbt_yarn::reactor::Control::Continue) },
        { Ok(bbt_yarn::reactor::Control::Continue) },
        { Ok(bbt_yarn::reactor::Control::Continue) }
    );
    projected.name = "HarnessTurnLoop".into();
    for mode in 0..=6 {
        out.push((
            launch_context(mode),
            if mode == 0 {
                "pass"
            } else if mode == 4 {
                "liveness"
            } else {
                "safety"
            },
        ));
    }
    out.push((projected, "pass"));
    out
}

#[test]
fn embed_stop_control_reaches_a_stale_a_to_b_to_a_delivery() {
    let good = embed_lifecycle(false);
    let bad = embed_lifecycle(true);
    let g = good.check(100_000).unwrap();
    let b = bad.check(100_000).unwrap();
    assert!(g.violations.is_empty());
    assert!(
        b.violations
            .iter()
            .any(|v| v.property == "StopSignalsOwnBirth"),
        "broken Stop must actually violate its named property"
    );
    let index = |name: &str| good.variables.iter().position(|v| v.name == name).unwrap();
    assert!(
        b.states.iter().any(|s| s[index("birth")] == 2
            && s[index("applied_stop_birth")] == 0
            && s[index("stopped")] == 1),
        "broken delivery must stop the replacement A with the first A's Stop"
    );
    assert!(
        g.states.iter().any(|s| s[index("birth")] == 2
            && s[index("held_stop_birth")] == 0
            && s[index("stop_pending")] == 1),
        "the stale A Stop must survive through B into a new A birth"
    );
}

#[test]
fn launch_controls_reach_named_forwarding_and_generation_failures() {
    for (mode, property) in [
        (1, "AdmittedContextForwarded"),
        (2, "AdmittedMcpForwarded"),
        (3, "CurrentGenerationOnly"),
        (4, "StartupSettlesWithoutServerFairness"),
        (5, "NoSyntheticPrompt"),
        (6, "LauncherApprovalMapped"),
    ] {
        let result = launch_context(mode).check(100_000).unwrap();
        assert!(
            result.violations.iter().any(|v| v.property == property),
            "control {mode} did not reach {property}"
        );
    }
    let model = launch_context(0);
    let result = model.check(100_000).unwrap();
    assert!(result.violations.is_empty());
    assert!(
        result
            .states
            .iter()
            .any(|s| s[0] == 2 && s[4] == 0 && s[1] == 1),
        "missing delayed A to B to A completion"
    );
    assert!(
        result
            .states
            .iter()
            .any(|s| s[1] == 2 && s[2] == 1 && s[3] == 1 && s[6] == 1 && s[7] == 1),
        "no nonempty forwarding state"
    );
}

/// One captured completion, two draft revisions, three view births A/B/A.
/// Actor-owned generations and drafts advance atomically. Completion may arrive
/// at any time; weak fairness of Complete settles it (including stale discard).
pub fn attachment_intake(b: bool) -> Model {
    let mut m = base::<6>(
        "AttachmentIntake",
        b,
        &[
            ("birth", 2, 0),
            ("draft", 1, 0),
            ("captured_birth", 2, 0),
            ("captured_draft", 1, 0),
            ("pending", 1, 0),
            ("crossed", 1, 0),
        ],
    );
    act(
        &mut m,
        "Capture",
        eq(4, 0),
        &[(2, Var(0)), (3, Var(1)), (4, Int(1))],
    );
    act(&mut m, "Edit", eq(1, 0), &[(1, Int(1))]);
    act(
        &mut m,
        "Switch",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1)))],
    );
    let current = Var(0).eq(Var(2)).and(Var(1).eq(Var(3)));
    // The two disjoint cases retain Complete's human label. Weak fairness is
    // on their union, exactly the original Complete action's enabling set.
    act(
        &mut m,
        "Complete",
        eq(4, 1).and(current.clone()),
        &[(4, Int(0)), (5, Var(5))],
    );
    act(
        &mut m,
        "Complete",
        eq(4, 1).and(current.negate()),
        &[(4, Int(0)), (5, if b { Int(1) } else { Var(5) })],
    );
    safe(&mut m, "NoCrossDraftOrBirth", eq(5, 0));
    progress(&mut m, "IntakeSettles", eq(4, 1), eq(4, 0), vec![3, 4]);
    m
}

/// Fresh connection challenges are represented by three monotonically distinct
/// births, including A/B/A. Old/absent actors may never advertise; no response
/// fairness is assumed. Sending text remains enabled without an image grant.
pub fn attachment_capability(b: bool) -> Model {
    let mut m = base::<6>(
        "AttachmentCapability",
        b,
        &[
            ("birth", 2, 0),
            ("captured_birth", 2, 0),
            ("pending", 1, 0),
            ("grant_birth", 3, 3),
            ("unsupported_sent", 1, 0),
            ("discovery_error", 1, 0),
        ],
    );
    act(&mut m, "Query", eq(2, 0), &[(1, Var(0)), (2, Int(1))]);
    act(
        &mut m,
        "Reconnect",
        Var(0).lt(Int(2)),
        &[(0, Var(0).plus(Int(1))), (3, Int(3))],
    );
    let current = Var(1).eq(Var(0));
    act(
        &mut m,
        "LiveResponse",
        eq(2, 1).and(current.clone()),
        &[(2, Int(0)), (3, Var(0))],
    );
    act(
        &mut m,
        "LiveResponse",
        eq(2, 1).and(current.negate()),
        &[(2, Int(0)), (3, Var(3))],
    );
    let valid = Var(3).eq(Var(0));
    act(&mut m, "SendImage", valid.clone(), &[(4, Var(4))]);
    act(
        &mut m,
        "SendImage",
        Bool(b).and(valid.negate()),
        &[(4, Int(1))],
    );
    act(&mut m, "SendText", Bool(true), &[]);
    // RPC outcomes are not user intents or custody grants, even after reconnect.
    act(&mut m, "DiscoveryCompletionIgnored", Bool(true), &[]);
    safe(&mut m, "NoImageWithoutCurrentActorProof", eq(4, 0));
    safe(&mut m, "DiscoveryCannotRejectUserIntent", eq(5, 0));
    m
}

fn attachment_discovery_broken() -> Model {
    let mut m = attachment_capability(false);
    m.name = "BrokenHarnessAttachmentDiscovery".into();
    act(&mut m, "BrokenDiscoveryUserError", eq(2, 1), &[(5, Int(1))]);
    m
}

/// One agent, three monotonic install attempts (A/B/A), one delayed duplicate
/// completion. Click/cancel/reap/completion are atomic short-lock operations.
/// Reap includes the existing OwnedChild wait fence; weak fairness abstracts the
/// 180s deadline, which is witnessed by the real process implementation.
pub fn onboarding_install(b: bool) -> Model {
    use Expr::{Int, Var};
    use bbt_yarn::model::{Action, Expr, Invariant, Model, Response, Variable};
    let mut m = Model::new(
        if b {
            "BrokenHarnessOnboardingInstall"
        } else {
            "HarnessOnboardingInstall"
        },
        vec![
            Variable::new("generation", 0, 3, 0),
            Variable::new("owner", 0, 3, 0),
            Variable::new("active", 0, 2, 0),
            Variable::new("old", 0, 3, 0),
            Variable::new("cancelled", 0, 1, 0),
            Variable::new("crossed", 0, 1, 0),
        ],
    );
    m.check_deadlock = false;
    let clickable = Var(0).lt(Int(3)).and(if b {
        Var(2).lt(Int(2))
    } else {
        Var(2).eq(Int(0))
    });
    for (empty, next_old) in [(true, Var(0).plus(Int(1))), (false, Var(3))] {
        let old_empty = Var(3).eq(Int(0));
        m.actions.push(Action::new(
            "Click",
            clickable
                .clone()
                .and(if empty { old_empty } else { old_empty.negate() }),
            vec![
                (0, Var(0).plus(Int(1))),
                (1, Var(0).plus(Int(1))),
                (2, Var(2).plus(Int(1))),
                (3, next_old),
            ],
        ));
    }
    m.actions
        .push(Action::new("Cancel", Var(2).eq(Int(1)), vec![(4, Int(1))]));
    m.actions.push(Action::new(
        "Reap",
        Var(2).eq(Int(1)),
        vec![(2, Int(0)), (4, Int(0))],
    ));
    m.actions.push(Action::new(
        "DelayedComplete",
        Int(0)
            .lt(Var(3))
            .and(Var(2).eq(Int(1)))
            .and(Var(1).eq(Var(3)).negate()),
        vec![(5, if b { Int(1) } else { Var(5) })],
    ));
    m.invariants.push(Invariant {
        name: "AtMostOneOwnedInstaller".into(),
        condition: Var(2).lt(Int(2)),
    });
    m.invariants.push(Invariant {
        name: "OnlyCurrentCompletionSettles".into(),
        condition: Var(5).eq(Int(0)),
    });
    m.responses.push(Response {
        name: "DeadlineEventuallyReaps".into(),
        request: Var(2).eq(Int(1)),
        response: Var(2).eq(Int(0)),
    });
    // Click split at index 0 shifts Cancel to 2 and Reap to 3.
    m.weak_fair.push(vec![3]);
    m
}

/// A replacement face acquires the seat's draft only after queued edits drain.
pub fn draft_handoff(b: bool) -> Model {
    let mut m = base::<5>(
        "DraftHandoff",
        b,
        &[
            ("old_phase", 2, 0), // visible, retired, released
            ("queued_edit", 1, 1),
            ("draft_text", 1, 0),
            ("replacement_open", 1, 0),
            ("replacement_text", 1, 0),
        ],
    );
    act(&mut m, "Detach", eq(0, 0), &[(0, Int(1))]);
    act(
        &mut m,
        "DrainAcceptedEdit",
        all([eq(0, 1), eq(1, 1)]),
        &[(1, Int(0)), (2, Int(1))],
    );
    act(
        &mut m,
        "ReleaseCustody",
        all([eq(0, 1), if b { Bool(true) } else { eq(1, 0) }]),
        &[(0, Int(2))],
    );
    act(
        &mut m,
        "Reopen",
        all([eq(0, 2), eq(3, 0)]),
        &[(3, Int(1)), (4, Var(2))],
    );
    safe(&mut m, "AcceptedEditSurvivesReopen", eq(3, 0).or(eq(4, 1)));
    m
}
pub fn unrestricted_startup(b: bool) -> Model {
    let mut m = base::<3>(
        "UnrestrictedStartup",
        b,
        &[
            ("phase", 3, 0),
            ("mode_confirmed", 1, 0),
            ("prompt_sent", 1, 0),
        ],
    );
    act(
        &mut m,
        "RequestAdvertisedFullAccess",
        eq(0, 0),
        &[(0, Int(1))],
    );
    act(
        &mut m,
        "AgentConfirmsFullAccess",
        eq(0, 1),
        &[(0, Int(2)), (1, Int(1))],
    );
    act(
        &mut m,
        "SendPrompt",
        if b { eq(0, 1).or(eq(0, 2)) } else { eq(0, 2) },
        &[(0, Int(3)), (2, Int(1))],
    );
    safe(
        &mut m,
        "FullAccessConfirmedBeforePrompt",
        eq(2, 0).or(eq(1, 1)),
    );
    m
}
