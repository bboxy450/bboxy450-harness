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
    assert!(!env.profiles.join("omp.md").exists());
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
                let state = bboxy450_harness::frontends::gui::bridge::FrameState {
                    onboarding: Arc::new(snapshot.clone()),
                    ready: true,
                    replay: true,
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

                let text = bboxy450_harness::frontends::gui::view::layout_text(layout);
                assert!(
                    text.contains("Codex")
                        && text.contains("adapter missing")
                        && text.contains("npm install --global")
                        && text
                            .replace(' ', "")
                            .contains("@agentclientprotocol/codex-acp@2.5.0")
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
            }
        }
    })
    .await;
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn private_setup_survives_unconfigured_startup_notice() {
    use bboxy450_harness::{
        frontends::gui::{
            bridge::{FrameState, ProfileInfo},
            model::Transcript,
            view::{HarnessView, layout_text},
        },
        protocol::*,
        runtime,
    };
    runtime::blocking(|| {
        let root = tempfile::tempdir().unwrap();
        let env = env(root.path());
        executable(&env, "codex", "exit 0");
        let snapshot = discover(&env);
        let base: Envelope = serde_json::from_str(
            include_str!("fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let mut model = Transcript::default();
        model.apply(&base);
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
                level: Level::Error,
                text: "profile could not be resolved".into(),
            },
        };
        model.apply(&event);
        let text = std::fs::read_to_string(env.profiles.join("codex.md")).unwrap();
        let mut state = FrameState {
            transcript: Arc::new(model),
            profiles: Arc::new(vec![ProfileInfo::parse("codex", &text)]),
            onboarding: Arc::new(snapshot),
            ready: true,
            ..Default::default()
        };
        let mut view = HarnessView::default();
        let frame = view.render(&state, [800., 900.]).unwrap();
        assert!(layout_text(frame.layout.as_ref().unwrap()).contains("Choose your coding agent"));
        Arc::make_mut(&mut state.transcript).turns.insert(
            TurnId::new(),
            bboxy450_harness::frontends::gui::model::TurnState::Running,
        );
        state.profiles = Arc::new(vec![]);
        let mut active = HarnessView::default();
        let frame = active.render(&state, [800., 900.]).unwrap();
        assert!(
            !layout_text(frame.layout.as_ref().unwrap()).contains("Choose your coding agent"),
            "catalogue discovery must not replace active chat"
        );
    })
    .await;
}
