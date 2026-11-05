// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Bounded builder/publication and external admission contracts, not native-code proofs.
//! Source authentication, byte custody, actual fsync/link, and canonical component
//! measurements remain independent process witnesses. In particular the frozen Q2
//! probe's Git mode currently returns 81 (unproven); these models certify no tool.
use super::*;

/// Two builders own distinct immutable candidates. Stages encode the finite
/// pipeline: absent/fetched/authenticated/built/reviewed/witnessed/qualification/
/// receipt/refused. IO may fail or never return; only local refusal is weakly fair.
pub fn publication(control: u8) -> Model {
    let suffix = match control {
        1 => "Auth",
        2 => "Review",
        3 => "Witness",
        4 => "ReceiptFirst",
        5 => "Clobber",
        _ => "",
    };
    let mut vars = Vec::new();
    for i in 0..2 {
        for (name, max, initial) in [
            ("stage", 8, 0),
            ("authenticated", 1, 0),
            ("reviewed", 1, 0),
            ("witnessed", 1, 0),
            ("qualification", 1, 0),
            ("receipt", 1, 0),
            ("owner", 2, 2),
            ("failed", 1, 0),
        ] {
            vars.push(Variable::new(&format!("{name}{i}"), 0, max, initial));
        }
    }
    let mut m = Model::new(
        &format!(
            "{}HarnessDreamProofToolsPublication{suffix}",
            if control == 0 { "" } else { "Broken" }
        ),
        vars,
    );
    m.check_deadlock = false;
    for i in 0..2 {
        let v = i * 8;
        let live = eq(v + 7, 0);
        act(&mut m, &format!("Fetch{i}"), eq(v, 0), &[(v, Int(1))]);
        act(
            &mut m,
            &format!("Authenticate{i}"),
            all([eq(v, 1), live.clone()]),
            &[(v, Int(2)), (v + 1, Int(1))],
        );
        act(
            &mut m,
            &format!("Build{i}"),
            all([
                if control == 1 {
                    eq(v, 1).or(eq(v, 2))
                } else {
                    eq(v, 2)
                },
                live.clone(),
            ]),
            &[(v, Int(3))],
        );
        act(
            &mut m,
            &format!("ReviewClosure{i}"),
            all([eq(v, 3), live.clone()]),
            &[(v, Int(4)), (v + 2, Int(i64::from(control != 2)))],
        );
        act(
            &mut m,
            &format!("CanonicalWitness{i}"),
            all([eq(v, 4), live.clone()]),
            &[(v, Int(5)), (v + 3, Int(1))],
        );
        act(
            &mut m,
            &format!("PublishQualification{i}"),
            all([
                if control == 3 {
                    eq(v, 4).or(eq(v, 5))
                } else {
                    eq(v, 5)
                },
                live.clone(),
            ]),
            &[(v, Int(6)), (v + 4, Int(1))],
        );
        act(
            &mut m,
            &format!("PublishReceiptLastNoReplace{i}"),
            all([
                if control == 4 {
                    eq(v, 5).or(eq(v, 6))
                } else {
                    eq(v, 6)
                },
                live,
                eq(v + 5, 0),
            ]),
            &[(v, Int(7)), (v + 5, Int(1)), (v + 6, Int(i as i64))],
        );
        act(
            &mut m,
            &format!("Fail{i}"),
            all([Int(0).lt(Var(v)), Var(v).lt(Int(7))]),
            &[(v + 7, Int(1))],
        );
        let refuse = m.actions.len();
        act(
            &mut m,
            &format!("RetainAndRefuse{i}"),
            all([eq(v + 7, 1), Var(v).lt(Int(8))]),
            &[(v, Int(8))],
        );
        progress(
            &mut m,
            &format!("FailedBuilderLocallyRefuses{i}"),
            eq(v + 7, 1),
            eq(v, 8),
            vec![refuse],
        );
        if control == 5 {
            act(
                &mut m,
                &format!("OtherBuilderReplacesPublished{i}"),
                eq(v + 5, 1),
                &[(v + 6, Int((1 - i) as i64))],
            );
        }
        safe(
            &mut m,
            "BuildRequiresAuthentication",
            Var(v).lt(Int(3)).or(eq(v, 8)).or(eq(v + 1, 1)),
        );
        safe(
            &mut m,
            "QualificationRequiresReviewAndWitness",
            eq(v + 4, 0).or(all([eq(v + 1, 1), eq(v + 2, 1), eq(v + 3, 1)])),
        );
        safe(
            &mut m,
            "ReceiptLastAfterQualification",
            eq(v + 5, 0).or(all([eq(v + 4, 1), eq(v + 7, 0)])),
        );
        safe(
            &mut m,
            "PublishedCandidateKeepsOriginalOwner",
            eq(v + 5, 0).or(eq(v + 6, i as i64)),
        );
    }
    m
}

/// Two opaque candidate tuples and three monotonic consumer incarnations A1/B2/A3.
/// Root admission is an external explicit action, never a builder feature claim.
/// `valid` abstracts the entire source/key/patch/config/library/compiler/OS/receipt
/// tuple; it does not claim that equal-looking bytes establish source provenance.
pub fn binding(control: u8) -> Model {
    let suffix = match control {
        1 => "SelfAdmission",
        2 => "Partial",
        3 => "ChangedTuple",
        4 => "StaleABA",
        5 => "OtherCandidate",
        _ => "",
    };
    let mut m = base(
        &format!("DreamProofToolsBinding{suffix}"),
        control != 0,
        &[
            ("complete0", 1, 0),
            ("complete1", 1, 0),
            ("valid0", 1, 1),
            ("valid1", 1, 1),
            ("epoch", 2, 0),
            ("selected", 1, 0),
            ("approvedCandidate", 2, 2),
            ("approvedEpoch", 3, 3),
            ("rootReviewed", 1, 0),
            ("builderFeatureClaim", 1, 0),
            ("bound", 1, 0),
            ("trusted", 1, 0),
        ],
    );
    for i in 0..2 {
        act(
            &mut m,
            &format!("CompleteCandidate{i}"),
            eq(i, 0),
            &[(i, Int(1))],
        );
        act(
            &mut m,
            &format!("SelectCandidate{i}"),
            Bool(true),
            &[(5, Int(i as i64)), (10, Int(0)), (11, Int(0))],
        );
        act(
            &mut m,
            &format!("MutateTuple{i}"),
            eq(i + 2, 1),
            &[(i + 2, Int(0)), (10, Int(0)), (11, Int(0))],
        );
        act(
            &mut m,
            &format!("RootAdmitExactCandidate{i}"),
            all([
                if control == 2 { Bool(true) } else { eq(i, 1) },
                eq(i + 2, 1),
            ]),
            &[
                (6, Int(i as i64)),
                (7, Var(4)),
                (8, Int(1)),
                (10, Int(0)),
                (11, Int(0)),
            ],
        );
        let current_epoch = if control == 4 {
            Var(4)
                .eq(Var(7))
                .or(all([eq(4, 0).or(eq(4, 2)), eq(7, 0).or(eq(7, 2))]))
        } else {
            Var(4).eq(Var(7))
        };
        let root = all([
            eq(8, 1),
            if control == 5 {
                Bool(true)
            } else {
                eq(6, i as i64)
            },
            current_epoch,
        ]);
        act(
            &mut m,
            &format!("BindPreparedCandidate{i}"),
            all([
                eq(5, i as i64),
                if control == 1 {
                    root.or(eq(9, 1))
                } else {
                    root
                },
                if control == 3 {
                    Bool(true)
                } else {
                    eq(i + 2, 1)
                },
            ]),
            &[(10, Int(1))],
        );
        safe(
            &mut m,
            "BoundRequiresCompleteQualification",
            eq(10, 0).or(eq(5, i as i64).negate()).or(eq(i, 1)),
        );
        safe(
            &mut m,
            "BoundRequiresCurrentExactTuple",
            eq(10, 0).or(eq(5, i as i64).negate()).or(eq(i + 2, 1)),
        );
    }
    act(&mut m, "BuilderClaimsFeatures", Bool(true), &[(9, Int(1))]);
    act(
        &mut m,
        "ChangeIncarnation",
        Var(4).lt(Int(2)),
        &[(4, Var(4).plus(Int(1))), (10, Int(0)), (11, Int(0))],
    );
    act(&mut m, "TrustBoundComponent", eq(10, 1), &[(11, Int(1))]);
    safe(
        &mut m,
        "BoundRequiresRootReviewedTuple",
        eq(10, 0).or(eq(8, 1)),
    );
    safe(
        &mut m,
        "BoundRequiresSameCandidate",
        eq(10, 0).or(Var(5).eq(Var(6))),
    );
    safe(
        &mut m,
        "BoundRequiresCurrentIncarnation",
        eq(10, 0).or(Var(4).eq(Var(7))),
    );
    safe(&mut m, "TrustRequiresBinding", eq(11, 0).or(eq(10, 1)));
    m
}

/// One retained original effect, three builder incarnations and debt 0..2.
/// Debt units stand for independently measured post-span rest, not milliseconds.
/// Exit/IO/clock recovery has no fairness. Local refusal can quarantine an owned
/// effect; quarantine neither proves reap nor releases the original identity.
pub fn custody(control: u8) -> Model {
    let suffix = match control {
        1 => "DebtReset",
        2 => "EffectABA",
        3 => "UnknownRetry",
        4 => "RetiredSignal",
        5 => "EarlyCleanup",
        6 => "TrustBeforeReap",
        _ => "",
    };
    let mut m = base(
        &format!("DreamProofToolsCustody{suffix}"),
        control != 0,
        &[
            ("epoch", 2, 0),
            ("effect", 1, 0),
            ("effectEpoch", 3, 3),
            ("originalEpoch", 3, 3),
            ("unknown", 1, 0),
            ("debt", 2, 0),
            ("paid", 2, 0),
            ("required", 2, 0),
            ("retired", 1, 0),
            ("reaped", 1, 0),
            ("terminal", 2, 0), // pending, released, owned quarantine
            ("trusted", 1, 0),
            ("fault", 1, 0),
            ("effects", 2, 0),
            ("released", 1, 0),
            ("clockKnown", 1, 1),
            ("retiredSignal", 1, 0),
        ],
    );
    act(
        &mut m,
        "CreateOriginalEffect",
        all([eq(1, 0), eq(13, 0), eq(10, 0)]),
        &[
            (1, Int(1)),
            (2, Var(0)),
            (3, Var(0)),
            (5, Int(2)),
            (7, Int(2)),
            (13, Int(1)),
        ],
    );
    act(
        &mut m,
        "FailureKeepsEffectAndDebt",
        all([eq(1, 1), eq(11, 0)]),
        &[
            (12, Int(1)),
            (5, if control == 1 { Int(0) } else { Var(5) }),
        ],
    );
    act(
        &mut m,
        "UnknownWriteOrNativeOutcome",
        all([eq(1, 1), eq(9, 0)]),
        &[(4, Int(1)), (12, Int(1))],
    );
    act(
        &mut m,
        "ClockLostKeepsOriginalEffect",
        all([eq(1, 1), eq(11, 0)]),
        &[(15, Int(0)), (4, Int(1)), (12, Int(1))],
    );
    act(
        &mut m,
        "RestartKeepsEffectIdentity",
        Var(0).lt(Int(2)),
        &[
            (0, Var(0).plus(Int(1))),
            (
                2,
                if control == 2 {
                    Var(0).plus(Int(1))
                } else {
                    Var(2)
                },
            ),
            (11, Int(0)),
        ],
    );
    if control == 3 {
        act(
            &mut m,
            "RetryUnknownEffect",
            all([eq(1, 1), eq(4, 1), eq(13, 1)]),
            &[(13, Int(2))],
        );
    }
    let pay = m.actions.len();
    act(
        &mut m,
        "PayObservedPostSpanDebt",
        all([Int(0).lt(Var(5)), eq(15, 1)]),
        &[(5, Var(5).minus(Int(1))), (6, Var(6).plus(Int(1)))],
    );
    act(&mut m, "RetireSignalingRights", eq(1, 1), &[(8, Int(1))]);
    act(
        &mut m,
        "SignalOwnedUnretiredIdentity",
        all([eq(1, 1), eq(8, 0), eq(9, 0)]),
        &[],
    );
    if control == 4 {
        act(
            &mut m,
            "SignalRetiredIdentity",
            all([eq(1, 1), eq(8, 1)]),
            &[(16, Int(1))],
        );
    }
    act(
        &mut m,
        "ObserveActualReapAndGroupSettlement",
        all([eq(1, 1), eq(8, 1)]),
        &[(9, Int(1)), (4, Int(0))],
    );
    let quarantine = m.actions.len();
    act(
        &mut m,
        "RetainOwnedQuarantine",
        all([eq(1, 1), eq(12, 1), eq(10, 0), eq(9, 0).or(eq(15, 0))]),
        &[(10, Int(2))],
    );
    let release = m.actions.len();
    act(
        &mut m,
        "ReleaseOnlySettledOriginalEffect",
        all([
            eq(1, 1),
            eq(8, 1),
            if control == 5 { Bool(true) } else { eq(9, 1) },
            eq(5, 0),
        ]),
        &[(1, Int(0)), (14, Int(1)), (10, Int(1))],
    );
    act(
        &mut m,
        "TrustAfterActualSettlementAndDebt",
        all([
            eq(1, 1),
            eq(8, 1),
            if control == 6 { Bool(true) } else { eq(9, 1) },
            eq(5, 0),
            eq(4, 0),
            eq(12, 0),
            eq(15, 1),
            Var(2).eq(Var(0)),
        ]),
        &[(11, Int(1))],
    );
    safe(&mut m, "DebtNeverForgiven", Var(5).plus(Var(6)).eq(Var(7)));
    safe(
        &mut m,
        "OriginalEffectSurvivesABA",
        eq(1, 0).or(Var(2).eq(Var(3))),
    );
    safe(&mut m, "NoReplacementForUnknownEffect", Var(13).le(Int(1)));
    safe(&mut m, "NoSignalAfterIdentityRetirement", eq(16, 0));
    safe(&mut m, "CleanupRequiresActualReap", eq(14, 0).or(eq(9, 1)));
    safe(
        &mut m,
        "TrustRequiresActualReapAndPaidDebt",
        eq(11, 0).or(all([eq(9, 1), eq(5, 0), eq(4, 0), eq(15, 1)])),
    );
    progress(
        &mut m,
        "FailedControlSettlesOrRetainsQuarantine",
        all([eq(1, 1), eq(12, 1)]),
        eq(10, 1).or(eq(10, 2)),
        vec![pay, quarantine, release],
    );
    m
}

/// An adversarial program may create and exit unseen process/thread/service work
/// between censuses. Only separately reviewed source/build guards can establish
/// the finite premise. This model explicitly permits the unseen event; it does
/// not infer its absence from runtime samples or claim universal detection.
pub fn hidden_work(control: u8) -> Model {
    let suffix = match control {
        1 => "CensusQualification",
        2 => "UnqualifiedTrust",
        _ => "",
    };
    let mut m = base(
        &format!("DreamProofToolsHiddenWork{suffix}"),
        control != 0,
        &[
            ("sourcePrepared", 1, 0),
            ("sourceGuard", 1, 1),
            ("audited", 1, 0),
            ("hiddenPhase", 2, 0),
            ("hiddenKind", 3, 0), // none, process, thread, service
            ("censusEmpty", 1, 0),
            ("qualified", 1, 0),
            ("trusted", 1, 0),
        ],
    );
    act(&mut m, "PrepareExactSource", Bool(true), &[(0, Int(1))]);
    act(&mut m, "ReviewExactSource", eq(0, 1), &[(2, Int(1))]);
    for kind in 1..=3 {
        act(
            &mut m,
            &format!("ReachableMutantKind{kind}"),
            all([eq(0, 1), eq(4, 0)]),
            &[(1, Int(0)), (4, Int(kind)), (6, Int(0)), (7, Int(0))],
        );
    }
    act(
        &mut m,
        "UnseenBirthBetweenSamples",
        all([eq(1, 0), eq(3, 0)]),
        &[(3, Int(1)), (5, Int(0))],
    );
    act(&mut m, "FastExitBeforeCensus", eq(3, 1), &[(3, Int(2))]);
    act(
        &mut m,
        "CensusReportsNoLiveExtraMember",
        eq(3, 0).or(eq(3, 2)),
        &[(5, Int(1))],
    );
    act(
        &mut m,
        "QualifyReviewedGuardedSource",
        all([
            eq(0, 1),
            eq(2, 1),
            if control == 1 {
                eq(1, 1).or(eq(5, 1))
            } else {
                eq(1, 1)
            },
        ]),
        &[(6, Int(1))],
    );
    act(
        &mut m,
        "TrustOnlySourceQualifiedLifetime",
        if control == 2 {
            eq(6, 1).or(all([eq(0, 1), eq(5, 1)]))
        } else {
            eq(6, 1)
        },
        &[(7, Int(1))],
    );
    safe(
        &mut m,
        "QualificationRequiresSourceGuard",
        eq(6, 0).or(all([eq(0, 1), eq(1, 1), eq(2, 1)])),
    );
    safe(
        &mut m,
        "TrustRequiresSourceQualifiedLifetime",
        eq(7, 0).or(all([eq(6, 1), eq(1, 1)])),
    );
    m
}

pub fn corpus() -> Vec<(Model, &'static str)> {
    let mut out = Vec::new();
    for (family, count) in [
        (publication as fn(u8) -> Model, 5),
        (binding as fn(u8) -> Model, 5),
        (custody as fn(u8) -> Model, 6),
        (hidden_work as fn(u8) -> Model, 2),
    ] {
        for control in 0..=count {
            out.push((
                family(control),
                if control == 0 { "pass" } else { "safety" },
            ));
        }
    }
    out
}

#[test]
fn dream_proof_tools_controls_reach_their_named_failure() {
    for (family, controls) in [
        (
            publication as fn(u8) -> Model,
            vec![
                (1, "BuildRequiresAuthentication"),
                (2, "QualificationRequiresReviewAndWitness"),
                (3, "QualificationRequiresReviewAndWitness"),
                (4, "ReceiptLastAfterQualification"),
                (5, "PublishedCandidateKeepsOriginalOwner"),
            ],
        ),
        (
            binding as fn(u8) -> Model,
            vec![
                (1, "BoundRequiresRootReviewedTuple"),
                (2, "BoundRequiresCompleteQualification"),
                (3, "BoundRequiresCurrentExactTuple"),
                (4, "BoundRequiresCurrentIncarnation"),
                (5, "BoundRequiresSameCandidate"),
            ],
        ),
        (
            custody as fn(u8) -> Model,
            vec![
                (1, "DebtNeverForgiven"),
                (2, "OriginalEffectSurvivesABA"),
                (3, "NoReplacementForUnknownEffect"),
                (4, "NoSignalAfterIdentityRetirement"),
                (5, "CleanupRequiresActualReap"),
                (6, "TrustRequiresActualReapAndPaidDebt"),
            ],
        ),
        (
            hidden_work as fn(u8) -> Model,
            vec![
                (1, "QualificationRequiresSourceGuard"),
                (2, "TrustRequiresSourceQualifiedLifetime"),
            ],
        ),
    ] {
        let good = family(0).check(100_000).unwrap();
        assert!(good.violations.is_empty());
        for (control, property) in controls {
            let bad = family(control).check(100_000).unwrap();
            assert!(
                bad.violations.iter().any(|v| v.property == property),
                "control {control} did not reach {property}"
            );
        }
    }
}

#[test]
fn dream_proof_tools_good_models_are_nonvacuous() {
    let publication = publication(0).check(100_000).unwrap();
    assert!(
        publication.states.iter().any(|s| s[0] == 7 && s[8] == 7),
        "both independent builders can publish complete candidates"
    );
    assert!(
        publication.states.iter().any(|s| s[0] == 1 && s[8] == 7),
        "one incomplete builder does not gate the peer"
    );
    let binding = binding(0).check(100_000).unwrap();
    assert!(
        binding
            .states
            .iter()
            .any(|s| s[4] == 2 && s[7] == 0 && s[10] == 0),
        "old A1 admission survives into A3 without rebinding"
    );
    assert!(
        binding
            .states
            .iter()
            .any(|s| s[5] == 1 && s[8] == 1 && s[11] == 1),
        "Root can admit and bind the second complete candidate"
    );
    let custody = custody(0).check(100_000).unwrap();
    assert!(
        custody.states.iter().any(|s| s[0] == 2
            && s[1] == 1
            && s[2] == 0
            && s[4] == 1
            && s[5] == 2),
        "unknown original A1 effect and full debt survive A3 restart"
    );
    assert!(
        custody
            .states
            .iter()
            .any(|s| s[10] == 2 && s[1] == 1 && s[9] == 0),
        "quarantine retains unreaped original custody"
    );
    assert!(
        custody
            .states
            .iter()
            .any(|s| s[11] == 1 && s[9] == 1 && s[6] == 2),
        "actual reap and full payment can precede a trusted component"
    );
    let hidden = hidden_work(0).check(100_000).unwrap();
    for kind in 1..=3 {
        assert!(
            hidden.states.iter().any(|s| s[3] == 2
                && s[4] == kind
                && s[5] == 1
                && s[6] == 0
                && s[7] == 0),
            "empty census after hidden kind {kind} must remain unqualified"
        );
    }
}
