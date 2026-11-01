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
fn fork_materialize(b: bool) -> Model {
    let mut m = base::<5>(
        "ForkMaterialize",
        b,
        &[
            ("phase", 3, 0),
            ("blobs_ready", 1, 0),
            ("copied_at", 1, 0),
            ("visible", 1, 0),
            ("parent_head", 2, 1),
        ],
    );
    act(&mut m, "CopyBlobs", eq(0, 0), &[(0, Int(1)), (1, Int(1))]);
    act(
        &mut m,
        "WriteFixedPrefix",
        eq(0, 1),
        &[(0, Int(2)), (2, Int(1))],
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
        &[(0, Int(0)), (1, Int(0)), (2, Int(0))],
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
fn process_lifecycle(mode: u8) -> Model {
    let mut m = base::<7>(
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
        &[(0, Int(2))],
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
    // This monitor is set by an actual signal action in an empty-group state,
    // not by the selected mode. The broken Drop removes the ownership guard.
    act(
        &mut m,
        "SignalEmptyGroup",
        all([
            eq(0, 2),
            eq(1, 0),
            if mode == 2 {
                Bool(true)
            } else {
                eq(0, 2).negate()
            },
        ]),
        &[(6, Int(1))],
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
        eq(2, 0).negate(),
        eq(2, 2),
        vec![exit, signal, skip, reap, drain, end],
    );
    match mode {
        1 => m.name = "BrokenHarnessProcessDrain".into(),
        2 => m.name = "BrokenHarnessProcessOwnership".into(),
        3 => m.name = "BrokenHarnessProcessReapedMembers".into(),
        _ => {}
    }
    m
}
pub fn corpus() -> Vec<(Model, &'static str)> {
    let mut out = vec![];
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
            profile_load(b),
            fork_materialize(b),
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
    out.push((steer_mode(false, true), "safety"));
    let mut registration = replay_base(false);
    registration.name = "HarnessSubscriberReplayRegistration".into();
    out.push((registration, "pass"));
    let mut registration = replay_base(true);
    registration.name = "BrokenHarnessSubscriberReplayRegistration".into();
    out.push((registration, "safety"));
    out.push((replay_tail(2), "safety"));
    out.push((process_lifecycle(0), "pass"));
    out.push((process_lifecycle(1), "liveness"));
    out.push((process_lifecycle(2), "safety"));
    out.push((process_lifecycle(3), "safety"));
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
    out.push((projected, "pass"));
    out
}
