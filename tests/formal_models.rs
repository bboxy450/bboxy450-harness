// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![allow(clippy::disallowed_methods)] // Private fixture/model IO runs outside the UI thread.
#[path = "formal/export.rs"]
mod export;
#[path = "formal/models.rs"]
mod models;
#[test]
fn native_contracts_and_broken_controls() {
    let path = std::env::var("FORMAL_OUT").unwrap_or_else(|_| "target/formal/models".into());
    let root = std::path::Path::new(&path);
    std::fs::create_dir_all(root).unwrap();
    let mut manifest = String::new();
    for (m, expected) in models::corpus() {
        export::export(m, expected, root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn gui_polish_pending_native_models_and_controls() {
    let path = std::env::var("GUI_POLISH_FORMAL_OUT")
        .unwrap_or_else(|_| "target/gui-polish/formal".into());
    let root = std::path::Path::new(&path);
    std::fs::create_dir_all(root).unwrap();
    let mut manifest = String::new();
    for (control, expected) in [(0, "pass"), (1, "safety"), (2, "safety")] {
        let model = models::gui_pending(control);
        let checked = model.check(100_000).unwrap();
        if control == 0 {
            assert!(
                checked.states.iter().any(|s| s[1] == 1 && s[5] == 1),
                "both repeated prompts must coexist"
            );
            assert!(
                checked.states.iter().any(|s| s[1] == 2 && s[5] == 2),
                "both causal completions must be reachable"
            );
            assert!(
                checked.states.iter().any(|s| s[0] == 2 && s[1] == 4),
                "A-B-A must retain unknown old ownership"
            );
        }
        export::export(model, expected, root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn scroll_native_models_and_controls() {
    let root = std::path::PathBuf::from(std::env::var("SCROLL_FORMAL_OUT")
        .unwrap_or_else(|_| "target/scroll/formal".into()));
    std::fs::create_dir_all(&root).unwrap();
    let mut manifest = String::new();
    for (model, expected) in [(models::scroll_stick(false), "pass"),
        (models::scroll_stick(true), "safety"), (models::scroll_resume_control(), "safety")] {
        export::export(model, expected, &root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn acp_steering_native_model_and_broken_controls() {
    let root = std::path::PathBuf::from(
        std::env::var("ACP_STEERING_FORMAL_OUT")
            .unwrap_or_else(|_| "target/acp-steering/formal".into()),
    );
    std::fs::create_dir_all(&root).unwrap();
    let mut manifest = String::new();
    for (control, expected) in [(0, "pass"), (1, "safety"), (2, "safety")] {
        let model = models::acp_steering(control);
        if control == 0 {
            let checked = model.check(100_000).unwrap();
            assert!(checked.states.iter().any(|s| s[0] == 3 && s[2] == 1));
            assert!(checked.states.iter().any(|s| s[0] == 3 && s[2] == 0));
            assert!(checked.states.iter().any(|s| s[0] == 5 && s[3] == 1));
            assert!(checked.states.iter().any(|s| s[0] == 6 && s[6] == 1));
        }
        export::export(model, expected, &root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn acp_controls_native_model_and_broken_controls() {
    let root = std::path::PathBuf::from(
        std::env::var("ACP_CONTROLS_FORMAL_OUT")
            .unwrap_or_else(|_| "target/acp-controls/formal".into()),
    );
    std::fs::create_dir_all(&root).unwrap();
    let mut manifest = String::new();
    for (control, expected) in [(0, "pass"), (1, "safety"), (2, "safety")] {
        let model = models::acp_controls(control);
        if control == 0 {
            let checked = model.check(100_000).unwrap();
            assert!(
                checked
                    .states
                    .iter()
                    .any(|s| s[0] == 2 && s[2] == 0 && s[4] == 1),
                "old request can survive A-B-A"
            );
            assert!(
                checked.states.iter().any(|s| s[4] == 2),
                "unknown outcome reachable"
            );
        }
        export::export(model, expected, &root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn attachments_intake_native_model_and_control() {
    let path = std::env::var("ATTACHMENTS_FORMAL_OUT")
        .unwrap_or_else(|_| "target/harness-attachments/formal".into());
    let root = std::path::Path::new(&path);
    std::fs::create_dir_all(root).unwrap();
    let mut manifest = String::new();
    for (broken, expected) in [(false, "pass"), (true, "safety")] {
        let model = models::attachment_intake(broken);
        let check = model.check(100_000).unwrap();
        assert!(
            check
                .states
                .iter()
                .any(|s| s[0] == 2 && s[2] == 0 && s[4] == 1),
            "delayed A-B-A completion is reachable"
        );
        export::export(model, expected, root, &mut manifest).unwrap();
    }
    for (broken, expected) in [(false, "pass"), (true, "safety")] {
        let model = models::attachment_capability(broken);
        let checked = model.check(100_000).unwrap();
        assert!(
            checked
                .states
                .iter()
                .any(|s| s[0] == 2 && s[1] == 0 && s[2] == 1),
            "delayed response across same-session reconnects must be reachable"
        );
        export::export(model, expected, root, &mut manifest).unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

#[test]
fn onboarding_install_native_model_and_control() {
    let root = std::path::PathBuf::from(
        std::env::var("ONBOARDING_FORMAL_OUT")
            .unwrap_or_else(|_| "target/harness-onboarding/formal".into()),
    );
    std::fs::create_dir_all(&root).unwrap();
    let mut manifest = String::new();
    for broken in [false, true] {
        export::export(
            models::onboarding_install(broken),
            if broken { "safety" } else { "pass" },
            &root,
            &mut manifest,
        )
        .unwrap();
    }
    std::fs::write(root.join("manifest.tsv"), manifest).unwrap();
}

// Exhaustive compatibility witness for the four conditional assignments that
// published TLC 0.1.1 cannot express. The reference is the original integer
// conditional math; the checker executes the actual lowered guarded actions.
#[test]
fn conditional_lowering_preserves_full_finite_successors_and_fairness() {
    use std::collections::BTreeMap;
    let discovery = models::corpus()
        .into_iter()
        .find(|(m, _)| m.name == "BrokenHarnessAttachmentDiscovery")
        .unwrap()
        .0;
    let cases = [
        (0, false, models::attachment_intake(false)),
        (0, true, models::attachment_intake(true)),
        (1, false, models::attachment_capability(false)),
        (1, true, models::attachment_capability(true)),
        (2, false, discovery),
        (3, false, models::onboarding_install(false)),
        (3, true, models::onboarding_install(true)),
    ];
    let mut checked_states = 0;
    let mut checked_edges = 0;
    for (family, broken, mut model) in cases {
        let (variables, labels, fair) = match family {
            0 => (
                vec![
                    ("birth", 2, 0),
                    ("draft", 1, 0),
                    ("captured_birth", 2, 0),
                    ("captured_draft", 1, 0),
                    ("pending", 1, 0),
                    ("crossed", 1, 0),
                ],
                vec!["Capture", "Edit", "Switch", "Complete", "Complete"],
                vec![vec![3, 4]],
            ),
            1 | 2 => (
                vec![
                    ("birth", 2, 0),
                    ("captured_birth", 2, 0),
                    ("pending", 1, 0),
                    ("grant_birth", 3, 3),
                    ("unsupported_sent", 1, 0),
                    ("discovery_error", 1, 0),
                ],
                vec![
                    "Query",
                    "Reconnect",
                    "LiveResponse",
                    "LiveResponse",
                    "SendImage",
                    "SendImage",
                    "SendText",
                    "DiscoveryCompletionIgnored",
                ],
                vec![],
            ),
            3 => (
                vec![
                    ("generation", 3, 0),
                    ("owner", 3, 0),
                    ("active", 2, 0),
                    ("old", 3, 0),
                    ("cancelled", 1, 0),
                    ("crossed", 1, 0),
                ],
                vec!["Click", "Click", "Cancel", "Reap", "DelayedComplete"],
                vec![vec![3]],
            ),
            _ => unreachable!(),
        };
        let mut labels = labels;
        if family == 2 {
            labels.push("BrokenDiscoveryUserError");
        }
        assert_eq!(model.weak_fair, fair, "{} fairness", model.name);
        assert!(!model.check_deadlock);
        assert_eq!(
            model
                .actions
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>(),
            labels
        );
        assert_eq!(
            model
                .variables
                .iter()
                .map(|v| (v.name.as_str(), v.max, v.initial))
                .collect::<Vec<_>>(),
            variables
        );
        assert!(model.variables.iter().all(|v| v.min == 0));
        let domain = model
            .variables
            .iter()
            .map(|v| (v.max + 1) as usize)
            .product::<usize>();
        // All declared states, including unreachable combinations and both
        // conditional branches. Never infer equivalence only from happy paths.
        for mut ordinal in 0..domain {
            let state = model
                .variables
                .iter()
                .map(|v| {
                    let value = (ordinal % (v.max + 1) as usize) as i64;
                    ordinal /= (v.max + 1) as usize;
                    value
                })
                .collect::<Vec<_>>();
            for (v, value) in model.variables.iter_mut().zip(&state) {
                v.initial = *value;
            }
            let mut expected = BTreeMap::new();
            let mut add = |label: &str, updates: &[(usize, i64)]| {
                let mut next = state.clone();
                for (i, value) in updates {
                    next[*i] = *value;
                }
                assert!(expected.insert(label.to_string(), next).is_none());
            };
            match family {
                0 => {
                    if state[4] == 0 {
                        add("Capture", &[(2, state[0]), (3, state[1]), (4, 1)]);
                    }
                    if state[1] == 0 {
                        add("Edit", &[(1, 1)]);
                    }
                    if state[0] < 2 {
                        add("Switch", &[(0, state[0] + 1)]);
                    }
                    if state[4] == 1 {
                        let current = state[0] == state[2] && state[1] == state[3];
                        add(
                            "Complete",
                            &[(4, 0), (5, if broken && !current { 1 } else { state[5] })],
                        );
                    }
                }
                1 | 2 => {
                    if state[2] == 0 {
                        add("Query", &[(1, state[0]), (2, 1)]);
                    }
                    if state[0] < 2 {
                        add("Reconnect", &[(0, state[0] + 1), (3, 3)]);
                    }
                    if state[2] == 1 {
                        add(
                            "LiveResponse",
                            &[
                                (2, 0),
                                (
                                    3,
                                    if state[1] == state[0] {
                                        state[0]
                                    } else {
                                        state[3]
                                    },
                                ),
                            ],
                        );
                    }
                    let valid = state[3] == state[0];
                    if broken || valid {
                        add("SendImage", &[(4, if valid { state[4] } else { 1 })]);
                    }
                    add("SendText", &[]);
                    add("DiscoveryCompletionIgnored", &[]);
                    if family == 2 && state[2] == 1 {
                        add("BrokenDiscoveryUserError", &[(5, 1)]);
                    }
                }
                3 => {
                    if state[0] < 3 && if broken { state[2] < 2 } else { state[2] == 0 } {
                        add(
                            "Click",
                            &[
                                (0, state[0] + 1),
                                (1, state[0] + 1),
                                (2, state[2] + 1),
                                (
                                    3,
                                    if state[3] == 0 {
                                        state[0] + 1
                                    } else {
                                        state[3]
                                    },
                                ),
                            ],
                        );
                    }
                    if state[2] == 1 {
                        add("Cancel", &[(4, 1)]);
                        add("Reap", &[(2, 0), (4, 0)]);
                    }
                    if state[3] > 0 && state[2] == 1 && state[1] != state[3] {
                        add("DelayedComplete", &[(5, if broken { 1 } else { state[5] })]);
                    }
                }
                _ => unreachable!(),
            }
            let report = model.check(domain).unwrap();
            assert_eq!(report.states[0], state);
            let mut actual = BTreeMap::new();
            for edge in &report.edges[0] {
                let label = &model.actions[edge.action].name;
                assert!(
                    actual
                        .insert(label.clone(), report.states[edge.to].clone())
                        .is_none(),
                    "split branches overlap: {} {state:?} {label}",
                    model.name
                );
            }
            assert_eq!(actual, expected, "{} at {state:?}", model.name);
            checked_states += 1;
            checked_edges += actual.len();
        }
    }
    assert_eq!(checked_states, 2688);
    println!(
        "conditional lowering: {checked_states} declared states; {checked_edges} labeled successors match original math; fairness groups match"
    );
}
