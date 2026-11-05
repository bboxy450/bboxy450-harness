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
