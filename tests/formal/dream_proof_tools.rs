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
//! Recipe mapping: build/previous_build retain original intent and inode on
//! failure/restart; refuse_unknown_build_effects fences unresolved commands;
//! publish_candidate seals qualification before the no-replace receipt;
//! bind_current_candidate rechecks the entire source/platform/tool tuple.
//! Whole-source snapshot checks bind the reviewed guard bytes, while real hidden
//! native mutants and compiler/platform fidelity remain separate obligations.
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

/// One prepared source/build evidence snapshot has either six or seven sources.
/// The current ordered tuple preserves the original six pins and appends exactly
/// Libksba 1.8.1 as source seven. Boolean predicates abstract actual cryptographic
/// verdicts and byte comparisons; they do not authenticate source by themselves.
/// Recipe mapping: source_snapshot binds the exact source bytes; build and
/// previous_build require matching source-lock/auth/generated/link evidence;
/// publish_candidate cannot promote an obsolete six-source basis by relabeling it.
/// SourceSeven is a finite admission constraint, not runtime approval, compiler
/// correctness, whole-native-code coverage or a claim to detect unseen events.
/// No IO completion, successful authentication, review or approval is fair.
pub fn source_seven(control: u8) -> Model {
    let suffix = match control {
        1 => "SixSource",
        2 => "StaleLock",
        3 => "MissingAuth",
        4 => "MismatchedAuth",
        5 => "Generated",
        6 => "Linked",
        7 => "Source",
        8 => "OrderedPins",
        _ => "",
    };
    let mut m = base(
        &format!("DreamProofToolsSourceSeven{suffix}"),
        control != 0,
        &[
            ("phase", 3, 0), // absent, prepared, auth observed, qualified basis
            ("sourceCount", 7, 0),
            ("lockCurrent", 1, 0),
            ("orderedPins", 1, 0),
            ("authenticatedCount", 7, 0),
            ("authenticatedBytesMatch", 1, 0),
            ("generatedCurrent", 1, 0),
            ("linkedCurrent", 1, 0),
            ("sourceCurrent", 1, 0),
            ("qualified", 1, 0),
            ("reused", 1, 0),
        ],
    );
    // The relabeled six-source input is deliberately distinct from a genuinely
    // current seven-source lock: a claimed current label cannot repair omission.
    for (name, count, current_lock, ordered) in [
        ("PrepareExactSevenSourceBasis", 7, 1, 1),
        ("PrepareRelabeledSixSourceBasis", 6, 1, 1),
        ("PrepareStaleSevenSourceLock", 7, 0, 1),
        ("PrepareChangedOriginalOrSeventhPin", 7, 1, 0),
    ] {
        act(
            &mut m,
            name,
            eq(0, 0),
            &[
                (0, Int(1)),
                (1, Int(count)),
                (2, Int(current_lock)),
                (3, Int(ordered)),
                (6, Int(1)),
                (7, Int(1)),
                (8, Int(1)),
            ],
        );
    }
    act(
        &mut m,
        "ObserveAuthenticationOfPreparedSourceSet",
        eq(0, 1),
        &[(0, Int(2)), (4, Var(1)), (5, Int(1))],
    );
    act(
        &mut m,
        "ObserveOnlySixAuthenticationVerdicts",
        all([eq(0, 1), eq(1, 7)]),
        &[(0, Int(2)), (4, Int(6)), (5, Int(1))],
    );
    act(
        &mut m,
        "ObserveMismatchedAuthenticationBytes",
        eq(0, 1),
        &[(0, Int(2)), (4, Var(1)), (5, Int(0))],
    );
    for (field, name) in [
        (6, "ChangedGeneratedConfig"),
        (7, "ChangedLinkedClosure"),
        (8, "ChangedSourceSnapshot"),
    ] {
        act(
            &mut m,
            name,
            eq(0, 2).or(eq(0, 3)),
            &[(field, Int(0)), (0, Int(2)), (9, Int(0)), (10, Int(0))],
        );
    }
    let exact_basis = all([
        eq(1, 7),
        eq(2, 1),
        eq(3, 1),
        Var(4).eq(Var(1)),
        eq(5, 1),
        eq(6, 1),
        eq(7, 1),
        eq(8, 1),
    ]);
    let controlled_basis = all([
        if control == 1 { Bool(true) } else { eq(1, 7) },
        if control == 2 { Bool(true) } else { eq(2, 1) },
        if control == 8 { Bool(true) } else { eq(3, 1) },
        if control == 3 {
            Bool(true)
        } else {
            Var(4).eq(Var(1))
        },
        if control == 4 { Bool(true) } else { eq(5, 1) },
        if control == 5 { Bool(true) } else { eq(6, 1) },
        if control == 6 { Bool(true) } else { eq(7, 1) },
        if control == 7 { Bool(true) } else { eq(8, 1) },
    ]);
    act(
        &mut m,
        "QualifyExactSevenSourceBasis",
        all([eq(0, 2), controlled_basis.clone()]),
        &[(0, Int(3)), (9, Int(1))],
    );
    act(
        &mut m,
        "ReuseOnlyCurrentAuthenticatedSourceBuild",
        all([eq(0, 3), eq(9, 1), controlled_basis]),
        &[(10, Int(1))],
    );
    let idle = all([eq(9, 0), eq(10, 0)]);
    for (name, required) in [
        ("SourceBasisRequiresExactlySeven", eq(1, 7)),
        ("SourceBasisRequiresCurrentLock", eq(2, 1)),
        ("SourceBasisAuthenticatesEverySource", Var(4).eq(Var(1))),
        ("SourceBasisRequiresMatchingAuthenticatedBytes", eq(5, 1)),
        ("SourceBasisRequiresGeneratedCurrentness", eq(6, 1)),
        ("SourceBasisRequiresLinkedCurrentness", eq(7, 1)),
        ("SourceBasisRequiresSourceCurrentness", eq(8, 1)),
        ("SourceBasisPreservesOrderedPins", eq(3, 1)),
    ] {
        safe(&mut m, name, idle.clone().or(required));
    }
    safe(
        &mut m,
        "ReuseRequiresQualifiedSourceBasis",
        eq(10, 0).or(eq(9, 1)),
    );
    // Retaining the exact predicate makes the abstraction's complete premise
    // explicit even when the causal controls remove one admission conjunct.
    safe(
        &mut m,
        "QualifiedSevenSourceBasisIsExact",
        idle.or(exact_basis),
    );
    m
}

/// ICD 98a3023e: complete typed pre-test basis, one immutable manifest and six
/// identical outer FilePins selecting fixed projections. Two candidates and
/// epochs A1/B2/A3 are opaque identities, never reusable approval labels.
/// Predicates represent strict canonical decoding, seven-source/auth/generated,
/// eight-finding/five-phase/sixteen-command and reachable closure/span coverage,
/// exact projection hashes, fixed config/fixture/probe bindings, aarch64 platform
/// identity and charged metadata custody. Their real byte/cryptographic/native
/// checks remain implementation obligations, not truths inferred from booleans.
/// Recipe mapping preserves build/previous_build/source_snapshot/publication;
/// Q2 alone owns the private decoder, canonical controller and runtime rejection.
/// Only Reject is weakly fair; parsing, IO, birth, actual settlement and external
/// Root admission have no success fairness. The result is permanently private.
pub fn pretest_basis(control: u8) -> Model {
    let suffix = match control {
        1 => "Malformed",
        2 => "ProjectionBinding",
        3 => "Coverage",
        4 => "Architecture",
        5 => "Currentness",
        6 => "SelfHashCycle",
        7 => "Custody",
        8 => "MetadataBudget",
        9 => "Fixture",
        10 => "StaleABA",
        11 => "CrossCandidate",
        12 => "UnapprovedRuntime",
        13 => "BirthBeforeBinding",
        14 => "UnchargedCopy",
        _ => "",
    };
    let mut m = base(
        &format!("DreamProofToolsPretestBasis{suffix}"),
        control != 0,
        &[
            ("phase", 6, 0), // absent, parsed, bound, admitted, rejected, settling, settled
            ("candidate", 1, 0),
            ("epoch", 2, 0),
            ("capturedCandidate", 2, 2),
            ("capturedEpoch", 3, 3),
            ("shapeValid", 1, 0),
            ("projectionsBound", 1, 0),
            ("coverage", 1, 0),
            ("arch", 1, 0),
            ("currentness", 1, 0),
            ("acyclic", 1, 0),
            ("custody", 1, 0),
            ("metadataBudget", 1, 0),
            ("fixtureBound", 1, 0),
            ("innerBound", 1, 0),
            ("helperBirths", 1, 0),
            ("privateResult", 1, 0),
            ("runtimeAdmitted", 1, 0),
            ("defect", 17, 0),
            ("basisCopies", 2, 0),
            ("chargedCopies", 2, 0),
            ("actualSettled", 1, 0),
        ],
    );
    // Each mutation is independently reachable; defect labels identify the
    // implementation witnesses represented by a failing finite predicate.
    for (defect, name, invalid_field) in [
        (0, "ParseCompleteTypedBasis", None),
        (1, "ParseMissingInnerField", Some(5)),
        (2, "ParseUnknownOrDuplicateInnerField", Some(5)),
        (3, "ParseNoncanonicalOrMistypedInteger", Some(5)),
        (4, "ParseWrongProjectionDigest", Some(6)),
        (5, "ParseSixUnequalManifestPins", Some(6)),
        (
            6,
            "ParseDroppedSourceAuthPatchNodeSpanCommandOrPhase",
            Some(7),
        ),
        (7, "ParseArm64AsCanonicalOuterLabel", Some(8)),
        (8, "ParseX86FatOrWrongMachO", Some(8)),
        (9, "ParseWrongCompilerTarget", Some(8)),
        (
            10,
            "ParseStaleSameVersionBinarySdkLibraryOrCacheUuid",
            Some(9),
        ),
        (11, "ParseCurrentSourceOrFutureResultHashCycle", Some(10)),
        (12, "ParseForeignRootOrUnsafeCustody", Some(11)),
        (13, "ParseExceededCombinedMetadataReservation", Some(12)),
        (14, "ParseForeignFixtureKeysOrCommit", Some(13)),
        (15, "ParseSubstitutedFrozenProbe", Some(9)),
        (16, "ParseCrossCandidateDescriptor", None),
        (17, "ParseIncompleteSevenSourceOrReviewCoverage", Some(7)),
    ] {
        let mut updates = vec![
            (0, Int(1)),
            (
                3,
                if defect == 16 {
                    Int(1).minus(Var(1))
                } else {
                    Var(1)
                },
            ),
            (4, Var(2)),
            (18, Int(defect)),
            (19, Int(1)),
            (20, Int(1)),
        ];
        for field in 5..=13 {
            updates.push((field, Int(i64::from(invalid_field != Some(field)))));
        }
        act(&mut m, name, eq(0, 0), &updates);
    }
    let exact_epoch = Var(2).eq(Var(4));
    let exact_candidate = Var(1).eq(Var(3));
    let copies_charged = Var(19).le(Var(20));
    let complete = all((5..=13).map(|field| eq(field, 1)))
        .and(exact_epoch.clone())
        .and(exact_candidate.clone())
        .and(copies_charged.clone());
    let controlled_fields = all((5..=13).map(|field| {
        let omitted = match field {
            5 => control == 1,
            6 => control == 2,
            7 => control == 3,
            8 => control == 4,
            9 => control == 5,
            10 => control == 6,
            11 => control == 7,
            12 => control == 8,
            13 => control == 9,
            _ => false,
        };
        if omitted {
            Bool(true)
        } else {
            eq(field, 1)
        }
    }));
    let controlled_epoch = if control == 10 {
        exact_epoch
            .clone()
            .or(all([eq(2, 0).or(eq(2, 2)), eq(4, 0).or(eq(4, 2))]))
    } else {
        exact_epoch.clone()
    };
    act(
        &mut m,
        "BindStrictTypedBasisAndSixFixedProjections",
        all([
            eq(0, 1),
            controlled_fields,
            controlled_epoch,
            if control == 11 {
                Bool(true)
            } else {
                exact_candidate.clone()
            },
            copies_charged.clone(),
        ]),
        &[(0, Int(2)), (14, Int(1))],
    );
    act(
        &mut m,
        "AdmitOriginalPrivateHelperOnlyAfterBinding",
        all([
            if control == 13 {
                eq(0, 1).or(eq(0, 2))
            } else {
                eq(0, 2).and(eq(14, 1))
            },
            exact_epoch.clone(),
            exact_candidate.clone(),
            eq(9, 1),
            eq(15, 0),
        ]),
        &[(0, Int(3)), (15, Int(1))],
    );
    // A-B-A never changes a captured snapshot's original epoch. After birth the
    // same change retires into settlement rather than erasing/replacing its effect.
    for phase in 0..=3 {
        act(
            &mut m,
            &format!("ChangeCandidateIncarnationAtPhase{phase}"),
            all([eq(0, phase), Var(2).lt(Int(2))]),
            &[
                (
                    0,
                    Int(if phase == 0 {
                        0
                    } else if phase == 3 {
                        5
                    } else {
                        1
                    }),
                ),
                (1, Int(1).minus(Var(1))),
                (2, Var(2).plus(Int(1))),
                (14, Int(0)),
            ],
        );
    }
    act(
        &mut m,
        "ChangeActualTupleBeforeBirth",
        eq(0, 1).or(eq(0, 2)),
        &[(0, Int(1)), (9, Int(0)), (14, Int(0))],
    );
    act(
        &mut m,
        "ChangeActualTupleAfterBirthKeepsOriginalEffect",
        eq(0, 3),
        &[(0, Int(5)), (9, Int(0)), (14, Int(0))],
    );
    if control == 14 {
        act(
            &mut m,
            "DuplicateBasisWithoutWorkingStorageCredit",
            all([eq(0, 1).or(eq(0, 2)), eq(19, 1)]),
            &[(19, Int(2))],
        );
    }
    let reject = m.actions.len();
    let invalid_pending = eq(0, 1).or(eq(0, 2)).and(complete.clone().negate());
    act(
        &mut m,
        "RejectMalformedOrUnboundBasisBeforeBirth",
        invalid_pending.clone(),
        &[(0, Int(4)), (14, Int(0))],
    );
    act(
        &mut m,
        "ObserveActualPrivateGroupAndRetiredSettlement",
        eq(0, 3).or(eq(0, 5)),
        &[(0, Int(6)), (14, Int(0)), (16, Int(1)), (21, Int(1))],
    );
    act(
        &mut m,
        "RuntimeRejectsPrivateCandidateResultKind",
        eq(16, 1),
        &[],
    );
    if control == 12 {
        act(
            &mut m,
            "TreatPrivateResultAsRuntimeApproval",
            eq(16, 1),
            &[(17, Int(1))],
        );
    }
    for (name, required) in [
        ("InnerBindingRequiresCanonicalTypedBasis", eq(5, 1)),
        ("InnerBindingRequiresOneManifestSixProjections", eq(6, 1)),
        ("InnerBindingRequiresCompleteCoverage", eq(7, 1)),
        ("InnerBindingRequiresCanonicalArchitecture", eq(8, 1)),
        ("InnerBindingRequiresCurrentTuple", eq(9, 1)),
        ("InnerBindingIsAcyclic", eq(10, 1)),
        ("InnerBindingRequiresOwnedCustody", eq(11, 1)),
        ("InnerBindingRequiresAggregateMetadataBudget", eq(12, 1)),
        ("InnerBindingRequiresBoundFixture", eq(13, 1)),
        ("InnerBindingRequiresCurrentIncarnation", exact_epoch),
        ("InnerBindingRequiresSameCandidate", exact_candidate),
    ] {
        safe(&mut m, name, eq(14, 0).or(required));
    }
    safe(&mut m, "EveryBasisCopyHasWorkingCredit", copies_charged);
    safe(
        &mut m,
        "PrivateBirthRequiresBoundBasis",
        eq(0, 3).negate().or(eq(14, 1)),
    );
    safe(
        &mut m,
        "RejectedBasisHasNoHelperBirth",
        eq(0, 4).negate().or(eq(15, 0)),
    );
    safe(
        &mut m,
        "PrivateResultRequiresActualSettlement",
        eq(16, 0).or(eq(21, 1)),
    );
    safe(&mut m, "PrivateResultCannotAuthorizeRuntime", eq(17, 0));
    progress(
        &mut m,
        "InvalidBasisLocallyRefuses",
        invalid_pending,
        eq(0, 4).or(eq(0, 5)).or(eq(0, 6)),
        vec![reject],
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
        (source_seven as fn(u8) -> Model, 8),
        (pretest_basis as fn(u8) -> Model, 14),
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

#[test]
fn dream_proof_tools_source_seven_controls_are_causal() {
    for (control, property) in [
        (1, "SourceBasisRequiresExactlySeven"),
        (2, "SourceBasisRequiresCurrentLock"),
        (3, "SourceBasisAuthenticatesEverySource"),
        (4, "SourceBasisRequiresMatchingAuthenticatedBytes"),
        (5, "SourceBasisRequiresGeneratedCurrentness"),
        (6, "SourceBasisRequiresLinkedCurrentness"),
        (7, "SourceBasisRequiresSourceCurrentness"),
        (8, "SourceBasisPreservesOrderedPins"),
    ] {
        let bad = source_seven(control).check(100_000).unwrap();
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == property && v.cycle.is_empty()),
            "source-seven control {control} did not reach safety invariant {property}"
        );
    }
}

#[test]
fn dream_proof_tools_source_seven_good_basis_is_nonvacuous() {
    let good = source_seven(0).check(100_000).unwrap();
    assert!(good.violations.is_empty());
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 3 && s[1] == 7 && s[4] == 7 && s[9] == 1 && s[10] == 1),
        "an exact authenticated seven-source current basis can qualify and be reused"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 2 && s[1] == 6 && s[2] == 1 && s[4] == 6 && s[9] == 0),
        "a fully authenticated relabeled six-source basis is reachable but unqualified"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 2 && s[1] == 7 && s[2] == 0 && s[4] == 7 && s[9] == 0),
        "a stale seven-source lock remains unqualified despite complete authentication"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 2 && s[1] == 7 && s[4] == 6 && s[9] == 0),
        "missing seventh authentication is reachable without qualifying the seven-source basis"
    );
    for field in [3, 5, 6, 7, 8] {
        assert!(
            good.states.iter().any(|s| s[0] == 2
                && s[1] == 7
                && s[4] == 7
                && s[field] == 0
                && s[9] == 0
                && s[10] == 0),
            "mismatched pin/auth/generated/link/source field {field} is reachable and refused"
        );
    }
}

#[test]
fn dream_proof_tools_typed_basis_controls_are_causal() {
    for (control, property) in [
        (1, "InnerBindingRequiresCanonicalTypedBasis"),
        (2, "InnerBindingRequiresOneManifestSixProjections"),
        (3, "InnerBindingRequiresCompleteCoverage"),
        (4, "InnerBindingRequiresCanonicalArchitecture"),
        (5, "InnerBindingRequiresCurrentTuple"),
        (6, "InnerBindingIsAcyclic"),
        (7, "InnerBindingRequiresOwnedCustody"),
        (8, "InnerBindingRequiresAggregateMetadataBudget"),
        (9, "InnerBindingRequiresBoundFixture"),
        (10, "InnerBindingRequiresCurrentIncarnation"),
        (11, "InnerBindingRequiresSameCandidate"),
        (12, "PrivateResultCannotAuthorizeRuntime"),
        (13, "PrivateBirthRequiresBoundBasis"),
        (14, "EveryBasisCopyHasWorkingCredit"),
    ] {
        let bad = pretest_basis(control).check(100_000).unwrap();
        assert!(
            bad.violations
                .iter()
                .any(|v| v.property == property && v.cycle.is_empty()),
            "typed-basis control {control} did not reach safety invariant {property}"
        );
    }
}

#[test]
fn dream_proof_tools_typed_basis_good_model_is_nonvacuous() {
    let good = pretest_basis(0).check(100_000).unwrap();
    assert!(good.violations.is_empty());
    for candidate in 0..=1 {
        assert!(
            good.states.iter().any(|s| s[0] == 6
                && s[3] == candidate
                && s[16] == 1
                && s[17] == 0
                && s[21] == 1),
            "candidate {candidate} can yield a settled permanently private result"
        );
    }
    for field in 5..=13 {
        assert!(
            good.states
                .iter()
                .any(|s| s[0] == 4 && s[field] == 0 && s[14] == 0 && s[15] == 0),
            "invalid typed-basis field {field} reaches refusal before any helper birth"
        );
    }
    assert!(
        good.states.iter().any(|s| s[0] == 1
            && s[1] == 0
            && s[2] == 2
            && s[3] == 0
            && s[4] == 0
            && s[14] == 0
            && s[15] == 0),
        "A1 typed input survives A-B-A without binding A3"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 5 && s[15] == 1 && s[14] == 0 && s[16] == 0),
        "changed admitted tuple keeps its original effect for actual settlement"
    );
    assert!(
        good.states
            .iter()
            .any(|s| s[0] == 2 && s[14] == 1 && s[19] == 1 && s[20] == 1),
        "six logical projections bind one charged physical basis manifest"
    );
}
