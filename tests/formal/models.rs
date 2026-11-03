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
fn scroll_stick(b: bool) -> Model {
    let mut m = base::<4>(
        "ScrollStick",
        b,
        &[
            ("tail", 3, 1),
            ("following", 1, 1),
            ("offset", 3, 1),
            ("requested", 3, 0),
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
    let mut m = base::<10>(
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
        ],
    );
    m.name = match mode {
        1 => "BrokenHarnessLaunchContextPrompt",
        2 => "BrokenHarnessLaunchContextMcp",
        3 => "BrokenHarnessLaunchContextStale",
        4 => "BrokenHarnessLaunchContextDeadline",
        5 => "BrokenHarnessLaunchContextSynthetic",
        _ => "HarnessLaunchContext",
    }
    .into();
    for context in 0..=1 {
        for mcp in 0..=1 {
            act(
                &mut m,
                "Admit",
                eq(1, 0),
                &[(1, Int(1)), (2, Int(context)), (3, Int(mcp))],
            );
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
    progress(
        &mut m,
        "StartupSettlesWithoutServerFairness",
        eq(1, 1),
        eq(1, 1).negate(),
        vec![tick, deadline],
    );
    m
}

pub fn corpus() -> Vec<(Model, &'static str)> {
    let mut out = vec![
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
    for mode in 0..=5 {
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
