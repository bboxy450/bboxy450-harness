// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![allow(clippy::disallowed_methods)] // Private fixture/model IO runs outside the UI thread.
use bboxy450_harness::{onboarding::*, provider::profile::Profile};
use std::{os::unix::fs::PermissionsExt, path::Path, sync::Arc, time::Duration};
fn env(root: &Path) -> Environment {
    std::fs::create_dir_all(root.join("bin")).unwrap();
    Environment {
        profiles: root.join("profiles"),
        home: root.into(),
        path: root.join("bin").into_os_string(),
        os: "macos".into(),
    }
}
fn executable(env: &Environment, name: &str, body: &str) {
    let path = env.home.join("bin").join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
#[test]
fn private_discovery_later_addition_and_profile_bytes() {
    let root = tempfile::tempdir().unwrap();
    let env = env(root.path());
    assert!(
        discover(&env)
            .entries
            .iter()
            .all(|e| !e.native && !e.adapter && !e.profile)
    );
    assert!(!env.profiles.join("grok.md").exists());
    assert!(!env.profiles.join("goose.md").exists());
    executable(&env, "codex", "exit 0");
    executable(&env, "npm", "exit 0");
    let snapshot = discover(&env);
    let entry = &snapshot.entries[0];
    assert!(entry.native && !entry.adapter && entry.profile);
    assert!(entry.status().contains("adapter missing"));
    let path = env.profiles.join("codex.md");
    let profile = Profile::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(profile.is_acp_agent());
    assert_eq!(profile.agent.unwrap().command, ["codex-acp"]);
    assert!(profile.api_key.is_none());
    let edited=b"---\nname: Customized\nprovider: acp-agent\nmodel: chosen-model\nagent:\n  command: [my-custom-command, --custom]\n---\nUser notes.\n";
    std::fs::write(&path, edited).unwrap();
    let custom = env.profiles.join("custom.md");
    std::fs::write(&custom, b"user custom bytes\n").unwrap();
    executable(&env, "grok", "exit 0");
    executable(&env, "goose", "exit 0");
    executable(&env, "pi", "exit 0");
    executable(&env, "omp", "exit 0");
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let env = env.clone();
            std::thread::spawn(move || discover(&env))
        })
        .collect();
    for thread in threads {
        let snapshot = thread.join().unwrap();
        assert!(snapshot.error.is_none());
    }
    assert_eq!(std::fs::read(path).unwrap(), edited);
    assert_eq!(std::fs::read(custom).unwrap(), b"user custom bytes\n");
    let grok =
        Profile::parse(&std::fs::read_to_string(env.profiles.join("grok.md")).unwrap()).unwrap();
    assert_eq!(grok.agent.unwrap().command, ["grok", "agent", "stdio"]);
    let goose = env.profiles.join("goose.md");
    let bytes = std::fs::read_to_string(&goose).unwrap();
    assert_eq!(
        Profile::parse(&bytes).unwrap().agent.unwrap().command,
        ["goose", "acp"]
    );
    std::fs::write(&goose, edited).unwrap();
    discover(&env);
    assert_eq!(std::fs::read(&goose).unwrap(), edited);
    let omp =
        Profile::parse(&std::fs::read_to_string(env.profiles.join("omp.md")).unwrap()).unwrap();
    assert_eq!(omp.policy.as_deref(), Some("yolo"));
    assert_eq!(omp.agent.unwrap().command, ["omp", "--mode", "acp"]);
    let snapshot = discover(&env);
    assert!(
        snapshot
            .entries
            .iter()
            .find(|e| e.agent == Agent::Omp)
            .unwrap()
            .recipes
            .is_empty()
    );
    executable(&env, "codex-acp", "exit 0");
    assert!(discover(&env).entries[0].adapter);
    assert!(std::fs::read_dir(&env.profiles).unwrap().all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".discover")
    }));
}
#[test]
fn private_manager_recipes_are_exact_and_platform_appropriate() {
    let root = tempfile::tempdir().unwrap();
    let mut env = env(root.path());
    for cmd in ["npm", "brew", "mise", "node"] {
        executable(&env, cmd, "exit 0");
    }
    let shim = env.home.join(".local/share/mise/shims");
    std::fs::create_dir_all(&shim).unwrap();
    env.path = std::env::join_paths([env.home.join("bin"), shim]).unwrap();
    let snapshot = discover(&env);
    let codex = &snapshot.entries[0];
    assert_eq!(codex.recipes.len(), 3);
    assert_eq!(
        codex.recipes[0].argv(),
        [
            "mise",
            "use",
            "--global",
            "npm:@agentclientprotocol/codex-acp@2.5.0"
        ]
    );
    assert_eq!(codex.recipes[1].argv(), ["brew", "install", "codex-acp"]);
    assert_eq!(
        codex.recipes[2].argv(),
        [
            "npm",
            "install",
            "--global",
            "@agentclientprotocol/codex-acp@2.5.0"
        ]
    );
    assert!(
        snapshot.entries[1]
            .recipes
            .iter()
            .all(|r| r.manager != Manager::Brew)
    );
    env.os = "windows".into();
    assert!(
        discover(&env).entries[0]
            .recipes
            .iter()
            .all(|r| r.manager != Manager::Brew)
    );
}
async fn settled(c: &Controller, agent: Agent) -> Install {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(i) = c.snapshot().installs.get(&agent)
                && !i.running
            {
                return i.clone();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}
#[tokio::test]
async fn private_installer_explicit_click_failure_retry_cancel_and_runtime_refresh() {
    let root = tempfile::tempdir().unwrap();
    let env = env(root.path());
    executable(&env, "codex", "exit 0");
    executable(
        &env,
        "npm",
        "echo \"$@\" >> \"$HOME/argv\"; echo private-failure; exit 7",
    );
    let c = Controller::shared(env.clone());
    let second = Controller::shared(env.clone());
    assert!(Arc::ptr_eq(&c, &second));
    c.refresh_now().await;
    assert!(
        !root.path().join("argv").exists(),
        "Discovery must never run the manager"
    );
    let recipe = c.snapshot().entries[0].recipes[0].clone();
    c.install(recipe.clone()).unwrap();
    assert!(second.install(recipe.clone()).is_err());
    let failed = settled(&c, Agent::Codex).await;
    assert!(failed.result.unwrap().contains("failed"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("argv"))
            .unwrap()
            .trim(),
        "install --global @agentclientprotocol/codex-acp@2.5.0"
    );
    executable(
        &env,
        "npm",
        "echo $$ > \"$HOME/pid\"; /bin/sleep 60 & echo $! > \"$HOME/child\"; wait",
    );
    c.install(recipe.clone()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !root.path().join("child").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let pid: i32 = std::fs::read_to_string(root.path().join("pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    c.cancel(Agent::Codex);
    assert!(
        c.install(recipe.clone()).is_err(),
        "cancel retains ownership until reap"
    );
    let cancelled = settled(&c, Agent::Codex).await;
    assert!(cancelled.result.unwrap().contains("cancelled"));
    assert_eq!(
        unsafe { libc::kill(pid, 0) },
        -1,
        "owned installer must be reaped"
    );
    assert_eq!(
        unsafe { libc::kill(-pid, 0) },
        -1,
        "owned descendants must be killed"
    );
    executable(
        &env,
        "npm",
        "printf '#!/bin/sh\\nexit 0\\n' > \"$HOME/bin/codex-acp\"; /bin/chmod 700 \"$HOME/bin/codex-acp\"; echo ready; exit 0",
    );
    c.install(recipe.clone()).unwrap();
    let success = settled(&c, Agent::Codex).await;
    assert!(success.result.unwrap().contains("complete"));
    tokio::time::timeout(Duration::from_secs(10), async {
        while !c.snapshot().entries[0].adapter {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        c.install(recipe).is_err(),
        "a stale UI choice cannot reinstall an available adapter"
    );
    assert!(c.snapshot().entries[0].profile);
    executable(&env, "claude-agent-acp", "exit 0");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !c.snapshot().entries[1].adapter {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        root.path().join("profiles/claude.md").exists(),
        "later additions discovered without restarting"
    );
}
#[tokio::test]
async fn private_installer_allowlist_and_bounded_output() {
    let root = tempfile::tempdir().unwrap();
    let env = env(root.path());
    executable(
        &env,
        "npm",
        "if [ -n \"$BBOXY_CAPABILITY_FILE\" ]; then exit 91; fi; i=0; while [ \"$i\" -lt 5000 ]; do printf 'bounded-output-1234567890\\n'; i=$((i+1)); done; exit 0",
    );
    let c = Controller::new(env);
    c.refresh_now().await;
    let invalid = Recipe {
        agent: Agent::Grok,
        manager: Manager::Brew,
        native: false,
    };
    assert!(c.install(invalid).is_err());
    assert!(c.snapshot().installs.is_empty());
    let recipe = c.snapshot().entries[0].recipes[0].clone();
    c.install(recipe).unwrap();
    let result = settled(&c, Agent::Codex).await;
    assert!(result.result.unwrap().contains("complete"));
    assert!(!result.output.is_empty());
    assert!(result.output.len() <= 8192);
}
#[cfg(feature = "embedded")]
#[tokio::test]
async fn onboarding_native_setup_cards_and_markdown_raster() {
    use bboxy450_harness::{frontends::gui::assets, runtime};
    use bbt_ui::Theme;
    let device = bbt_render::device::DeviceContext::new().await.unwrap();
    let root = tempfile::tempdir().unwrap();
    let env = env(root.path());
    executable(&env, "npm", "exit 0");
    executable(&env, "codex", "exit 0");
    let snapshot = discover(&env);
    runtime::blocking(move || {
        let mut renderer =
            bbt_render::chrome::ChromeRenderer::with_context(device, assets::icons()).unwrap();
        for (theme, name) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
            for width in [320u32, 800] {
                let mut view = bboxy450_harness::frontends::gui::view::HarnessView::default();
                view.set_theme(theme);
                view.scroll.anchor(123., 1000.);
                let base: bboxy450_harness::protocol::Envelope = serde_json::from_str(
                    include_str!("fixtures/protocol/all-items.jsonl")
                        .lines()
                        .next()
                        .unwrap(),
                )
                .unwrap();
                let mut model = bboxy450_harness::frontends::gui::model::Transcript::default();
                model.apply(&base);
                model.profile = Some(("missing".into(), "unconfigured".into()));
                let state = bboxy450_harness::frontends::gui::bridge::FrameState {
                    onboarding: Arc::new(snapshot.clone()),
                    transcript: Arc::new(model),
                    ready: true,
                    ..Default::default()
                };
                let frame = view.render(&state, [width as f32, 1000.]).unwrap();
                let layout = frame.layout.as_ref().unwrap();
                let viewport = bboxy450_harness::frontends::gui::view::find(layout, "transcript")
                    .unwrap()
                    .children
                    .first()
                    .unwrap()
                    .rect;
                let heading = bboxy450_harness::frontends::gui::view::find(layout, "setup")
                    .unwrap()
                    .rect;
                assert!(heading.y >= viewport.y - 1., "welcome must open at the top");

                let panel = bboxy450_harness::frontends::gui::view::find(layout, "setup/panel")
                    .unwrap()
                    .rect;
                assert!(
                    (panel.x + panel.w / 2. - width as f32 / 2.).abs() < 1.,
                    "panel {panel:?}"
                );
                assert!(panel.w <= 600. && panel.x >= 15.);
                let portrait =
                    bboxy450_harness::frontends::gui::view::find(layout, "setup/portrait/mark")
                        .unwrap()
                        .rect;
                assert!((portrait.x + portrait.w / 2. - width as f32 / 2.).abs() < 1.);
                for slug in ["codex", "claude", "pi"] {
                    let row = bboxy450_harness::frontends::gui::view::find(
                        layout,
                        &format!("setup/{slug}/row"),
                    )
                    .unwrap()
                    .rect;
                    assert!(
                        row.h < 125. && row.bottom() < viewport.bottom(),
                        "compact visible row: {row:?}"
                    );
                }
                let text = bboxy450_harness::frontends::gui::view::layout_text(layout);
                assert!(
                    text.contains("Codex")
                        && text.contains("adapter missing")
                        && !text.contains("npm install --global")
                        && !text.contains("Authentication and model selection")
                );
                assert!(frame.actions.iter().any(|(_, a)| matches!(
                    a,
                    bboxy450_harness::frontends::gui::view::Action::InstallAdapter(_)
                )));
                let rgba = renderer
                    .render_to_rgba(&frame.list, width, 1000, 1.)
                    .unwrap();
                assert!(rgba.windows(4).any(|p| p[0] != p[1]));
                if let Some(out) = std::env::var_os("ONBOARDING_CAPTURE_DIR") {
                    let out = std::path::PathBuf::from(out);
                    std::fs::create_dir_all(&out).unwrap();
                    let mut encoder = png::Encoder::new(
                        std::fs::File::create(out.join(format!("setup-{name}-{width}.png")))
                            .unwrap(),
                        width,
                        1000,
                    );
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()
                        .unwrap()
                        .write_image_data(&rgba)
                        .unwrap();
                }
                // Operate the actual retained disclosure action, then restore both scrolls.
                use bboxy450_harness::frontends::gui::{
                    bridge::Bridge,
                    view::{Action, find, layout_text},
                };
                struct Quiet;
                impl bbt_rt::Waker for Quiet {
                    fn wake(&self) {}
                }
                let bridge = Bridge::new(Arc::new(Quiet));
                let details = frame
                    .actions
                    .iter()
                    .find_map(|(_, a)| match a {
                        Action::SetupDetails(Agent::Codex) => Some(a.clone()),
                        _ => None,
                    })
                    .unwrap();
                view.activate(details.clone(), &bridge);
                let expanded = view.render(&state, [width as f32, 1000.]).unwrap();
                let text = layout_text(expanded.layout.as_ref().unwrap());
                assert!(
                    text.contains("npm install --global")
                        && text
                            .replace(' ', "")
                            .contains("@agentclientprotocol/codex-acp@2.5.0")
                );
                view.scroll.anchor(57., 1000.);
                view.activate(Action::Setup(false), &bridge);
                view.render(&state, [width as f32, 1000.]).unwrap();
                assert_eq!(
                    view.scroll.layout_offset(),
                    123.,
                    "conversation scroll restored"
                );
                view.activate(Action::Setup(true), &bridge);
                view.render(&state, [width as f32, 1000.]).unwrap();
                assert_eq!(view.scroll.layout_offset(), 57., "setup scroll restored");
                view.activate(details, &bridge);
                let collapsed = view.render(&state, [width as f32, 1000.]).unwrap();
                assert!(find(collapsed.layout.as_ref().unwrap(), "setup/panel").is_some());
                assert!(
                    !layout_text(collapsed.layout.as_ref().unwrap())
                        .contains("npm install --global")
                );
            }
        }
    })
    .await;
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn onboarding_loader_welcome_routing() {
    use bboxy450_harness::{
        frontends::gui::{
            bridge::FrameState,
            model::Transcript,
            view::{HarnessView, Preparer, find, layout_text},
        },
        protocol::*,
        runtime,
    };
    runtime::blocking(|| {
        let root = tempfile::tempdir().unwrap();
        let env = env(root.path());
        executable(&env, "codex", "exit 0");
        let mut state = FrameState {
            onboarding: Arc::new(discover(&env)),
            ..Default::default()
        };
        let render = |state: &FrameState, expected: &str| {
            let mut view = HarnessView::default();
            let frame = view.render(state, [800., 900.]).unwrap();
            let layout = frame.layout.as_ref().unwrap();
            assert!(
                find(layout, expected).is_some(),
                "expected {expected}: {}",
                layout_text(layout)
            );
            assert!(find(layout, "setup").is_none() || expected == "setup");
        };
        // Discovery can finish before connection, catalogue, metadata or replay.
        render(&state, "session-loader");
        state.ready = true;
        render(&state, "session-loader");
        let base: Envelope = serde_json::from_str(
            include_str!("fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let mut model = Transcript::default();
        model.apply(&base);
        state.transcript = Arc::new(model.clone());
        render(&state, "session-loader");
        model.profile = Some(("missing".into(), "unconfigured".into()));
        state.transcript = Arc::new(model.clone());
        state.ready = false;
        render(&state, "session-loader");
        state.ready = true;
        render(&state, "setup");
        state.replay = true;
        render(&state, "empty");
        state.replay = false;
        // Out-of-turn history has no turns entry, but remains history while preparing.
        let id = ItemId::new();
        let mut event = base.clone();
        event.seq = Some(Seq(2));
        event.at = Seq(2);
        event.event = Event::ItemStarted {
            turn: None,
            item: id,
            kind: ItemKind::Notice,
            parent: None,
            head: None,
        };
        model.apply(&event);
        event.seq = Some(Seq(3));
        event.at = Seq(3);
        event.event = Event::ItemCompleted {
            item: id,
            status: ItemStatus::Completed,
            body: Item::Notice {
                level: Level::Info,
                text: "Preserved history".into(),
            },
        };
        model.apply(&event);
        assert!(model.turns.is_empty());
        state.transcript = Arc::new(model);
        render(&state, "session-loader");
        state.prepared = Some(Arc::new(
            Preparer::default()
                .prepare(state.transcript.clone(), 800.)
                .unwrap(),
        ));
        render(&state, &format!("item/{id}"));
        let mut fresh = state.clone();
        fresh.created_here = true;
        Arc::make_mut(&mut fresh.transcript).profile = None;
        render(&fresh, "setup");
        let mut startup_view = HarnessView::default();
        assert!(
            layout_text(
                startup_view
                    .render(&fresh, [800., 900.])
                    .unwrap()
                    .layout
                    .as_ref()
                    .unwrap()
            )
            .contains("Preserved history")
        );
        state.ready = false;
        let mut view = HarnessView::default();
        assert!(
            layout_text(
                view.render(&state, [800., 900.])
                    .unwrap()
                    .layout
                    .as_ref()
                    .unwrap()
            )
            .contains("Preserved history")
        );
    })
    .await;
}

#[cfg(feature = "standalone")]
#[path = "support/onboarding_startup.rs"]
mod startup_fixture;
#[cfg(feature = "standalone")]
#[test]
fn private_empty_home_native_app_reaches_setup() {
    let home = startup_fixture::home();
    assert!(!home.path().join(".bboxy450-harness").exists());
    let mut command = startup_fixture::command(
        std::path::Path::new(env!("CARGO_BIN_EXE_bboxy450-harness")),
        home.path(),
    );
    command
        .args(["gui"])
        .env("BBOXY450_HARNESS_SMOKE_WINDOW", "1");
    let output = startup_fixture::run(&mut command);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "actual app exited before setup: {stderr}"
    );
    assert!(
        stderr.contains("visible=false; setup=true; profiles=0"),
        "{stderr}"
    );
    assert!(
        !home.path().join("installer-argv").exists(),
        "startup never executes a manager"
    );
    startup_fixture::evidence(
        "empty-home-native-app.json",
        serde_json::json!({"command":["bboxy450-harness","gui"],"private_home":true,"private_path":true,"exit_code":output.status.code(),"stderr":stderr}),
    );
}
