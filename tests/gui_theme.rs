// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "embedded")]
#![allow(
    clippy::disallowed_methods,
    reason = "private offscreen native appearance witness"
)]
use bboxy450_harness::{
    frontends::gui::{
        bridge::{FrameState, PendingPrompt},
        model::{Transcript, ViewItem},
        rich::Documents,
        view::{HarnessView, Metrics, Preparer, find, layout_text},
    },
    protocol::*,
};
use bbt_render::chrome::{ChromeRenderer, materials as m};
use bbt_ui::{self as ui, components as c};
use std::{path::Path, sync::Arc};

fn fixture() -> Arc<Transcript> {
    // Adapted from tests/gui_chrome.rs retained item fixture, same project/policy above.
    let session = SessionId::new();
    let mut t = Transcript {
        session: Some(session),
        epoch: Epoch(7),
        cursor: Seq(22),
        ..Default::default()
    };
    for i in 0..12 {
        let id = ItemId::new();
        let row = ViewItem {
            id,
            turn: None,
            parent: None,
            started: Seq(1),
            completed: Some(Seq(2)),
            kind: ItemKind::AgentMessage,
            body: Item::AgentMessage {
                markdown: format!(
                    "Theme prose {i}: `make verify-harness-theme`.\n\n```python\nvalue = 42\nprint(\"native colors\", value)\n```\n"
                ),
            },
            status: Some(ItemStatus::Completed),
            terminal_tail: String::new(),
            started_ms: 0,
            completed_ms: Some(1),
        };
        t.order.push_back(id);
        t.items.insert(id, row);
    }
    Arc::new(t)
}
fn save(path: &Path, width: u32, height: u32, pixels: &[u8]) {
    let mut e = png::Encoder::new(std::fs::File::create(path).unwrap(), width, height);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(pixels).unwrap();
}
fn capture(
    renderer: &mut ChromeRenderer,
    view: &mut HarnessView,
    state: &FrameState,
    name: &str,
) -> Vec<u8> {
    let frame = view.render(state, [640., 480.]).unwrap();
    let pixels = m::render_composed_to_rgba(
        renderer,
        m::Scene {
            backdrop: m::Backdrop::DrawList(&frame.list),
            items: &frame.rims,
            width: 640,
            height: 480,
            scale: 1.,
            time_ms: 0.,
            theme: frame.theme,
        },
        &frame.over,
    )
    .unwrap();
    let bg = bbt_ui_model::draw::Color::hex(view.theme().palette().bg).0;
    assert_eq!(
        &pixels[(100 * 640 + 2) * 4..(100 * 640 + 2) * 4 + 3],
        &bg.map(|v| (v * 255.).round() as u8)[..3],
        "native background palette"
    );
    if let Some(out) = std::env::var_os("BBOXY_HARNESS_THEME_OUT") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        save(&out.join(format!("{name}.png")), 640, 480, &pixels);
    }
    pixels
}
fn private(name: &str) -> bool {
    if std::env::var_os("BBOXY_THEME_PRIVATE").is_some() {
        return false;
    }
    let home = tempfile::Builder::new()
        .prefix("harness-theme-")
        .tempdir()
        .unwrap();
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--nocapture"])
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .env("BBOXY_THEME_PRIVATE", "1");
    if let Some(out) = std::env::var_os("BBOXY_HARNESS_THEME_OUT") {
        command.env("BBOXY_HARNESS_THEME_OUT", out);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
// keeps: harness-theme — independent palettes, delayed and ABA preparation never recolor ownership.
#[test]
fn harness_theme_native_independent_views_and_delayed_aba() {
    if private("harness_theme_native_independent_views_and_delayed_aba") {
        return;
    }
    bbt_rt::handle().block_on(bboxy450_harness::runtime::blocking(|| {
        let model = fixture();
        let mut preparer = Preparer::default();
        let mut view = HarnessView::default();
        let old_dark = Arc::new(preparer.prepare(model.clone(), 640.).unwrap());
        let mut state = FrameState {
            transcript: model.clone(),
            prepared: Some(old_dark.clone()),
            ready: true,
            replay: true,
            ..Default::default()
        };
        let mut pending = (*state.pending_prompts).clone();
        let mut prompt = PendingPrompt::new(
            OpId::new(),
            model.session.unwrap(),
            model.epoch,
            "queued gray tail".into(),
        );
        prompt.status = "Queued".into();
        pending.insert(prompt.op, prompt);
        state.pending_prompts = Arc::new(pending);
        view.edit(ui::text_edit::Command::Insert("retained draft".into()));
        let mut renderer =
            ChromeRenderer::new(bboxy450_harness::frontends::gui::assets::icons()).unwrap();
        capture(&mut renderer, &mut view, &state, "dark-current");
        assert!(view.frame.max_scroll > 0.);
        view.scroll.anchor(36., view.frame.max_scroll);
        view.invalidate();
        view.render(&state, [640., 480.]).unwrap();
        let scroll = view.frame.scroll;
        let pending = state.pending_prompts.clone();
        let source = state.transcript.clone();
        let draft = view.composer.view.clone();
        assert!(view.set_theme(ui::Theme::Light));
        assert!(!view.set_theme(ui::Theme::Light));
        preparer.set_theme(ui::Theme::Light);
        let light = Arc::new(preparer.prepare(model.clone(), 640.).unwrap());
        // Dark completion arriving after the light request is incompatible.
        capture(&mut renderer, &mut view, &state, "light-delayed-loading");
        assert!(view.frame.visible.is_empty());
        state.prepared = Some(light.clone());
        let light_pixels = capture(&mut renderer, &mut view, &state, "light-current");
        assert!((view.frame.scroll - scroll).abs() < 0.01);
        assert_eq!(view.composer.view, draft);
        view.scroll.follow_latest();
        view.invalidate();
        capture(&mut renderer, &mut view, &state, "light-queued-tail");
        let text = layout_text(view.frame.layout.as_ref().unwrap());
        assert!(text.contains("queued gray tail"));
        view.scroll.anchor(scroll, view.frame.max_scroll);
        view.invalidate();
        let mut independent = HarnessView::default();
        independent.set_theme(ui::Theme::Light);
        independent.edit(ui::text_edit::Command::Insert("retained draft".into()));
        let mut independent_state = state.clone();
        let mut independent_p = Preparer::default();
        independent_p.set_theme(ui::Theme::Light);
        independent_state.prepared = Some(Arc::new(
            independent_p.prepare(model.clone(), 640.).unwrap(),
        ));
        let stable = capture(
            &mut renderer,
            &mut independent,
            &independent_state,
            "light-independent",
        );
        view.set_theme(ui::Theme::Dark);
        preparer.set_theme(ui::Theme::Dark);
        state.prepared = Some(old_dark);
        capture(&mut renderer, &mut view, &state, "dark-aba-loading");
        assert!(view.frame.visible.is_empty());
        state.prepared = Some(Arc::new(preparer.prepare(model, 640.).unwrap()));
        let dark = capture(&mut renderer, &mut view, &state, "dark-returned");
        assert_ne!(dark, light_pixels);
        assert_eq!(view.composer.view, draft);
        assert!(Arc::ptr_eq(&pending, &state.pending_prompts));
        assert!(Arc::ptr_eq(&source, &state.transcript));
        assert_eq!(
            stable,
            capture(
                &mut renderer,
                &mut independent,
                &independent_state,
                "light-independent-after"
            )
        );
        independent.toggle_profile_menu();
        let menu_pixels = capture(
            &mut renderer,
            &mut independent,
            &independent_state,
            "light-menu",
        );
        let surface = ui::Paint::solid(ui::Color::Surface2)
            .resolve(ui::Context::new(ui::Theme::Light, ui::Density::Cozy))
            .unwrap()
            .map(|v| (v * 255.).round() as u8);
        assert_eq!(
            &menu_pixels[(100 * 640 + 400) * 4..(100 * 640 + 400) * 4 + 3],
            &surface[..3],
            "menu uses the light palette, not a dark literal"
        );
        assert!(find(independent.frame.layout.as_ref().unwrap(), "composer").is_some());
        assert!(!independent.frame.over.primitives.is_empty());
    }));
}
fn colors(node: &c::Element, out: &mut Vec<[f32; 4]>, ctx: ui::Context) {
    match node {
        c::Element::Text(t) => {
            for run in &t.runs {
                out.push(run.style.paint.resolve(ctx).unwrap());
            }
        }
        c::Element::Named { content, .. } => colors(content, out, ctx),
        c::Element::Stack(s) => {
            for child in &s.children {
                colors(&child.element, out, ctx);
            }
        }
        c::Element::Flow(f) => {
            for child in &f.children {
                colors(&child.element, out, ctx);
            }
        }
        c::Element::ScrollView(s) => colors(&s.content, out, ctx),
        c::Element::Box(b) => {
            for child in &b.children {
                colors(&child.element, out, ctx)
            }
        }
        _ => {}
    }
}
#[test]
fn harness_theme_python_highlights_and_plain_output_use_matching_palette() {
    bbt_rt::handle().block_on(bboxy450_harness::runtime::blocking(|| {
        let mut docs = Documents::default();
        let metrics = Metrics::default();
        let mut palettes = vec![];
        for theme in [ui::Theme::Dark, ui::Theme::Light, ui::Theme::Dark] {
            docs.set_theme(theme);
            let ctx = ui::Context::new(theme, ui::Density::Cozy);
            let element = docs.code_element("value = 42\nprint(\"color\", value)\n", &metrics);
            let mut colors_ = vec![];
            colors(&element, &mut colors_, ctx);
            assert!(colors_.len() > 3);
            assert!(colors_.windows(2).any(|p| p[0] != p[1]));
            palettes.push(colors_);
            let ink = bboxy450_harness::frontends::gui::face::ink(ctx);
            let output =
                bboxy450_harness::frontends::gui::face::term_output_full("literal output", &ink);
            let mut output_colors = vec![];
            colors(&output, &mut output_colors, ctx);
            assert!(!output_colors.is_empty());
            assert!(output_colors.iter().all(|c| *c == ink.fg));
        }
        assert_ne!(palettes[0], palettes[1]);
        assert_eq!(palettes[0], palettes[2]);
    }));
}
#[cfg(feature = "standalone")]
#[test]
fn harness_theme_native_system_event_adapter_and_existing_override() {
    use bboxy450_harness::frontends::gui::window::Appearance;
    use winit::{event::WindowEvent, window::Theme};
    let mut appearance = Appearance::new(Some(Theme::Light), None);
    assert_eq!(appearance.theme(), ui::Theme::Light);
    assert!(appearance.system_event(&WindowEvent::ThemeChanged(Theme::Dark)));
    assert_eq!(appearance.theme(), ui::Theme::Dark);
    appearance.set_override(Some(ui::Theme::Light));
    assert!(!appearance.system_event(&WindowEvent::ThemeChanged(Theme::Light)));
    assert!(!appearance.system_event(&WindowEvent::ThemeChanged(Theme::Dark)));
    assert_eq!(appearance.theme(), ui::Theme::Light);
    appearance.cycle();
    assert_eq!(appearance.theme(), ui::Theme::Dark);
    appearance.cycle();
    assert_eq!(appearance.theme(), ui::Theme::Dark);
    assert!(appearance.system_event(&WindowEvent::ThemeChanged(Theme::Light)));
    assert_eq!(appearance.theme(), ui::Theme::Light);
}

#[tokio::test]
async fn harness_theme_reads_existing_settings_override_without_touching_zoom() {
    let root = tempfile::Builder::new()
        .prefix("theme-settings-")
        .tempdir()
        .unwrap();
    for (name, expected) in [
        ("light", Some(ui::Theme::Light)),
        ("dark", Some(ui::Theme::Dark)),
        ("auto", None),
    ] {
        let bytes = format!("{{\"theme\":\"{name}\",\"zoom\":120}}\n");
        tokio::fs::write(root.path().join("settings.json"), &bytes)
            .await
            .unwrap();
        assert_eq!(
            bboxy450_harness::frontends::gui::zoom::load_theme(root.path()).await,
            expected
        );
        assert_eq!(
            tokio::fs::read_to_string(root.path().join("settings.json"))
                .await
                .unwrap(),
            bytes
        );
    }
}
