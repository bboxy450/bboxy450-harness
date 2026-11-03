// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The standalone window's chrome (harness-spec.md "Standalone window"): the term's own
//! title bar in place of OS decorations, and its retained hit regions. The window is created
//! with `with_decorations(false)`; this module draws `bbt_ui::modules::title_bar::TitleBar`
//! (dots, mark, `bboxy450-harness`, the `profile · cwd` crumb, the theme control, no sidebar
//! toggle) and turns presses on it into window intents. Geometry comes from the painted tree,
//! never from platform window dimensions.
//
// Adapted from bboxy450-term crates/bboxy450-term/src/desktop_window.rs (Titlebar, Hit,
// Intent and their tests). https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use bbt_desktop_core::{Action, TitlebarAction};
use bbt_ui::{self as ui, components as c};
use bbt_ui_model::layout::Rect;
use std::time::{Duration, Instant};

/// The bar's height, style.json `spacing.title_bar`.
pub const TITLE_BAR_H: f32 = bbt_ui_model::STYLE.spacing.title_bar as f32;
pub const TITLE: &str = "bboxy450-harness";

/// The term's title bar for the harness: `bboxy450-harness`, the `profile · cwd leaf` crumb
/// and the theme control; the sidebar toggle is removed as the web does
/// (`bar.querySelector('.tbtn:not(.theme)').remove()`). Flat fallback material: plain
/// `surface` with the 1px hair bottom line (no frosted blur in the harness composition).
pub fn title_bar(crumb: &str, focused: bool, ctx: ui::Context) -> c::Element {
    let mut bar = ui::modules::title_bar::TitleBar {
        title: TITLE.into(),
        crumb: crumb.into(),
        sidebar_open: false,
        blurred: !focused,
        theme: "dark".into(),
    }
    .element(ctx);
    if let c::Element::Named { content, .. } = &mut bar
        && let c::Element::Box(b) = content.as_mut()
    {
        b.children.retain(
            |item| !matches!(&item.element, c::Element::Named { name, .. } if name == "bar/tbtn"),
        );
        b.style.overrides.material = Some(ui::Material::Flat);
        b.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface));
    }
    bar
}

/// The crumb text: `profile · leaf`, `profile`, `leaf`, or nothing.
pub fn crumb(profile: Option<&str>, leaf: Option<&str>) -> String {
    match (
        profile.filter(|p| !p.is_empty()),
        leaf.filter(|l| !l.is_empty()),
    ) {
        (Some(p), Some(l)) => format!("{p} · {l}"),
        (Some(p), None) => p.into(),
        (None, Some(l)) => l.into(),
        (None, None) => String::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Close,
    Minimize,
    Fullscreen,
    Theme,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Window(Action),
    Titlebar(TitlebarAction),
    Theme,
    /// The caller may start an OS drag only for a physical mouse-down event.
    Drag,
}

/// Retained title-bar hit regions and the double-click gesture.
#[derive(Debug, Default)]
pub struct Titlebar {
    bounds: Option<Rect>,
    controls: Vec<(Hit, Rect)>,
    click: Option<(Instant, [f32; 2])>,
}

fn contains(rect: Rect, at: [f32; 2]) -> bool {
    at[0] >= rect.x && at[0] < rect.right() && at[1] >= rect.y && at[1] < rect.bottom()
}

fn named<'a>(layout: &'a c::Layout, key: &str) -> Option<&'a c::Layout> {
    if matches!(&layout.kind, c::LayoutKind::Named(name) if name == key) {
        return Some(layout);
    }
    layout.children.iter().find_map(|child| named(child, key))
}

fn near(a: [f32; 2], b: [f32; 2]) -> bool {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy <= 16.
}

impl Titlebar {
    /// Read the bar and its controls off the solved tree. No bar (a capture, a test without
    /// chrome) retains nothing and hits nothing.
    pub fn retain(&mut self, layout: &c::Layout) {
        self.bounds = named(layout, "bar").map(|l| l.rect);
        self.controls = [
            (Hit::Close, "bar/win/i"),
            (Hit::Minimize, "bar/win/i1"),
            (Hit::Fullscreen, "bar/win/i2"),
            (Hit::Theme, "bar/tbtn1"),
        ]
        .into_iter()
        .filter_map(|(hit, key)| named(layout, key).map(|l| (hit, l.rect)))
        .collect();
        if self.bounds.is_none() {
            self.click = None;
        }
    }
    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }
    pub fn hit(&self, at: [f32; 2]) -> Option<Hit> {
        if !self.bounds.is_some_and(|bounds| contains(bounds, at)) {
            return None;
        }
        Some(
            self.controls
                .iter()
                .find(|(_, rect)| contains(*rect, at))
                .map_or(Hit::Background, |(hit, _)| *hit),
        )
    }
    /// A drag, a click elsewhere, or invalid coordinates cannot complete a double-click.
    pub fn moved(&mut self, at: [f32; 2]) {
        if self.click.is_some_and(|(_, from)| !near(from, at)) {
            self.click = None;
        }
    }
    /// A press at `at`: a control's intent, the second half of a double-click on the
    /// background (`double_click`, the system preference), or a drag.
    pub fn press(
        &mut self,
        at: [f32; 2],
        now: Instant,
        interval: Duration,
        double_click: TitlebarAction,
    ) -> Option<Intent> {
        let Some(hit) = self.hit(at) else {
            self.click = None;
            return None;
        };
        if hit != Hit::Background {
            self.click = None;
            return Some(match hit {
                Hit::Close => Intent::Window(Action::CloseWindow),
                Hit::Minimize => Intent::Window(Action::Minimize),
                Hit::Fullscreen => Intent::Window(Action::ToggleFullscreen),
                Hit::Theme => Intent::Theme,
                Hit::Background => unreachable!(),
            });
        }
        if self.click.take().is_some_and(|(previous, from)| {
            !interval.is_zero()
                && now
                    .checked_duration_since(previous)
                    .is_some_and(|elapsed| elapsed <= interval)
                && near(from, at)
        }) {
            return Some(Intent::Titlebar(double_click));
        }
        self.click = Some((now, at));
        Some(Intent::Drag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titlebar() -> Titlebar {
        Titlebar {
            bounds: Some(Rect::new(0., 0., 400., 38.)),
            controls: vec![
                (Hit::Close, Rect::new(14., 14., 10., 10.)),
                (Hit::Minimize, Rect::new(30., 14., 10., 10.)),
                (Hit::Fullscreen, Rect::new(46., 14., 10., 10.)),
                (Hit::Theme, Rect::new(370., 8., 26., 26.)),
            ],
            click: None,
        }
    }

    // keeps: operator standard title-bar behavior — controls never begin window gestures.
    #[test]
    fn controls_follow_painted_regions_and_exclude_double_clicks_and_drag() {
        let mut bar = titlebar();
        let now = Instant::now();
        for (point, intent) in [
            ([19., 19.], Intent::Window(Action::CloseWindow)),
            ([35., 19.], Intent::Window(Action::Minimize)),
            ([51., 19.], Intent::Window(Action::ToggleFullscreen)),
            ([380., 19.], Intent::Theme),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    bar.press(point, now, Duration::from_millis(500), TitlebarAction::Zoom),
                    Some(intent)
                );
            }
        }
        assert_eq!(bar.hit([400., 10.]), None);
        assert_eq!(bar.hit([100., 38.]), None);
        assert_eq!(bar.hit([100., f32::NAN]), None);
        assert_eq!(bar.hit([200., 20.]), Some(Hit::Background));
    }

    // keeps: operator standard title-bar behavior — honor platform policy and timing.
    #[test]
    fn double_clicks_honor_interval_distance_policy_and_do_not_overlap() {
        let interval = Duration::from_millis(500);
        let now = Instant::now();
        for policy in [
            TitlebarAction::Fill,
            TitlebarAction::Zoom,
            TitlebarAction::Minimize,
            TitlebarAction::None,
        ] {
            let mut bar = titlebar();
            assert_eq!(
                bar.press([200., 20.], now, interval, policy),
                Some(Intent::Drag)
            );
            assert_eq!(
                bar.press([202., 21.], now + interval, interval, policy),
                Some(Intent::Titlebar(policy))
            );
            assert_eq!(
                bar.press([200., 20.], now + interval, interval, policy),
                Some(Intent::Drag)
            );
        }
        for (elapsed, at) in [(501, [200., 20.]), (100, [205., 20.])] {
            let mut bar = titlebar();
            bar.press([200., 20.], now, interval, TitlebarAction::Zoom);
            assert_eq!(
                bar.press(
                    at,
                    now + Duration::from_millis(elapsed),
                    interval,
                    TitlebarAction::Zoom
                ),
                Some(Intent::Drag)
            );
        }
    }

    #[test]
    fn drag_movement_and_non_titlebar_clicks_cancel_the_first_click() {
        let now = Instant::now();
        let interval = Duration::from_millis(500);
        let mut bar = titlebar();
        bar.press([200., 20.], now, interval, TitlebarAction::Zoom);
        bar.moved([220., 20.]);
        bar.moved([200., 20.]);
        assert_eq!(
            bar.press([200., 20.], now, interval, TitlebarAction::Zoom),
            Some(Intent::Drag)
        );
        assert_eq!(
            bar.press([200., 200.], now, interval, TitlebarAction::Zoom),
            None
        );
        assert_eq!(
            bar.press([200., 20.], now, interval, TitlebarAction::Zoom),
            Some(Intent::Drag)
        );
        let mut empty = Titlebar::default();
        empty.retain(
            &c::Element::Spacer
                .layout(
                    Rect::new(0., 0., 10., 10.),
                    super::super::view::context(),
                    &super::super::view::Metrics::default(),
                )
                .unwrap(),
        );
        assert_eq!(
            empty.hit([5., 5.]),
            None,
            "no bar in the tree: nothing hits"
        );
    }

    #[test]
    fn crumb_joins_the_known_parts() {
        assert_eq!(
            crumb(Some("claude-acp"), Some("acpdemo")),
            "claude-acp · acpdemo"
        );
        assert_eq!(crumb(Some("claude-acp"), None), "claude-acp");
        assert_eq!(crumb(None, Some("acpdemo")), "acpdemo");
        assert_eq!(crumb(Some(""), Some("")), "");
    }
}
