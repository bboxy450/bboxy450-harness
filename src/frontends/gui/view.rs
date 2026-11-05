// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Retained toolkit transcript and composer. Effects stay with the host bridge.
#![deny(clippy::disallowed_methods)]
use super::{
    anim::{Clock, Ease},
    bridge::{Bridge, FrameState, ProfileInfo},
    chat_scroll, chrome,
    model::{Transcript, TurnState},
    stream_motion,
    zoom::Zoom,
};
use crate::protocol::*;
use bbt_render::chrome::materials as m;
use bbt_ui::{
    self as ui, components as c,
    modules::{composer, seat_row::named},
    widgets::{Button, ButtonKind, ButtonState},
};
use bbt_ui_model::{
    Density, Theme,
    draw::{Color, DrawList, TextStyle, TypeScale},
    layout::{Axis, Rect, TextMeasure, TextStyle as MeasureStyle},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::Arc,
};
#[path = "face_rows.rs"]
mod face_rows;
use face_rows::Folds;

// Adapted from bboxy450-term scene::Metrics (CoreText, shared paint metrics).
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
type TranscriptColumn = (c::Element, Option<f32>, HashMap<ItemId, f32>);
type FontKey = (u32, u16, bool, bool, u32);
fn font_key(s: &MeasureStyle) -> FontKey {
    (
        s.size.to_bits(),
        s.weight,
        s.mono,
        s.italic,
        s.tracking_em.to_bits(),
    )
}
#[derive(Default)]
pub struct Metrics {
    widths: RefCell<HashMap<(String, FontKey), f32>>,
    fonts: RefCell<HashMap<FontKey, [f32; 2]>>,
}
fn font(s: &MeasureStyle) -> TextStyle {
    TextStyle {
        scale: TypeScale::Custom(s.size),
        weight: s.weight,
        mono: s.mono,
        italic: s.italic,
        tracking_em: s.tracking_em,
        ..Default::default()
    }
}
impl TextMeasure for Metrics {
    fn width(&self, text: &str, s: &MeasureStyle) -> f32 {
        let key = (text.to_owned(), font_key(s));
        if let Some(width) = self.widths.borrow().get(&key) {
            return *width;
        }
        let width = bbt_render::chrome::text::measure(text, font(s), None)
            .expect("native metrics")
            .advance;
        self.widths.borrow_mut().insert(key, width);
        width
    }
}
impl c::Measure for Metrics {
    fn font_metrics(&self, s: &MeasureStyle) -> [f32; 2] {
        let key = font_key(s);
        if let Some(metrics) = self.fonts.borrow().get(&key) {
            return *metrics;
        }
        let m = bbt_render::chrome::text::measure("Hg", font(s), None).expect("native font");
        let metrics = [m.ascent, m.descent];
        self.fonts.borrow_mut().insert(key, metrics);
        metrics
    }
}
pub fn context() -> ui::Context {
    ui::Context::new(Theme::Dark, Density::Cozy)
}
pub(super) fn column(children: Vec<c::Item>, height: Option<f32>) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.padding = Some([ui::Length::Px(0.); 4]);
    style.overrides.height = height.map(ui::Length::Px);
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(0.),
        paint: ui::Paint::solid(ui::Color::Transparent),
        outside: false,
    });
    style.overrides.fill = Some(ui::Paint::solid(ui::Color::Transparent));
    let mut b = c::Box::new(style, children);
    b.direction = Axis::Column;
    b.gap = ui::Length::Px(0.);
    b.align = c::Alignment::Stretch;
    c::Element::Box(b)
}
pub fn label(value: impl Into<String>) -> c::Element {
    c::Element::Text(c::Text::new(
        value,
        ui::Text {
            step: ui::TypeStep::Caption,
            ..Default::default()
        },
    ))
}
pub fn find<'a>(n: &'a c::Layout, key: &str) -> Option<&'a c::Layout> {
    if matches!(&n.kind, c::LayoutKind::Named(name) if name == key) {
        Some(n)
    } else {
        n.children.iter().find_map(|c| find(c, key))
    }
}
fn contains(r: Rect, at: [f32; 2]) -> bool {
    at[0] >= r.x && at[0] < r.right() && at[1] >= r.y && at[1] < r.bottom()
}
fn scroll_panel(name: &str) -> bool {
    name.starts_with("mdout/") || name.starts_with("codeout/")
}
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Send,
    RemoveAttachment(usize),
    Interrupt,
    Stop,
    StopAt(Epoch),
    Approval(ItemId, ApprovalDecision),
    Jump,
    Toggle(ItemId, super::rich::Toggle),
    /// An error receipt's Retry: the failed turn's own prompt, sent again.
    Retry(TurnId),
    Complete(ui::text_edit::Completion, String),
    /// The header profile chip: opens or closes the profile switcher.
    Profiles,
    /// A profile switcher row: the same path as `/profile NAME`.
    SwitchProfile(String),
    AgentConfig(OpEnvelope),
    ConfigMenu(String),
    PanelScroll(String, [f32; 2]),
    /// Inside an open menu (true) keeps it; outside it (false) closes it.
    Menu(bool),
    /// The switcher's `+ New profile`: the native form is not built yet.
    NewProfile,
    Setup(bool),
    SetupDetails(crate::onboarding::Agent),
    InstallAdapter(crate::onboarding::Recipe),
    CancelInstall(crate::onboarding::Agent),
    /// An empty-session chip: 0 opens the "/" menu, 1 inserts `@`, 2 focuses the composer.
    Chip(u8),
}
/// Keys the open profile switcher takes before the composer.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuKey {
    Up,
    Down,
    Enter,
    Backspace,
    Text(String),
}
#[derive(Default)]
pub struct Frame {
    pub links: Arc<ui::modules::document_modal::controller::Regions>,
    pub theme: Theme,
    /// The face: chrome, header, transcript and composer, flat primitives.
    pub list: DrawList,
    /// The alive rims (the composer's), for the renderer's material pass between `list`
    /// and `over`. Empty in a flat composition that drops them.
    pub rims: Vec<m::Surface>,
    /// Over the face and its rims: the floating menus and the zoom pill.
    pub over: DrawList,
    pub layout: Option<c::Layout>,
    pub visible: Vec<(ItemId, ItemKind, Rect)>,
    pub actions: Vec<(Rect, Action)>,
    /// Pointer hover zones, front to back: the open menu's rows, then each message row's
    /// source toggle (`source/{id}/toggle`) and the row itself (`msg/{id}`).
    pub hover: Vec<(Rect, String)>,
    pub scroll: f32,
    pub max_scroll: f32,
    pub timings: [std::time::Duration; 5],
}
impl Frame {
    pub fn item_text(&self, id: ItemId) -> Option<String> {
        self.layout
            .as_ref()
            .and_then(|l| find(l, &format!("item/{id}")))
            .map(layout_text)
    }
}

// Retain identity ownership as well as content. A retired prepared snapshot's
// allocation cannot be reused for an A -> B -> A resize before replacement.
type CacheKey = (
    Arc<Transcript>,
    String,
    [f32; 2],
    f32,
    bool,
    Option<Arc<PreparedRows>>,
);
pub struct CachedRow {
    width: f32,
    /// The text zoom the row was laid out at, percent.
    zoom: u8,
    completed: Option<Seq>,
    deps: u64,
    face: super::rich::Face,
    layout: c::Layout,
}
struct Controls {
    key: String,
    size: [f32; 2],
    layout: c::Layout,
    rows_h: f32,
}
#[path = "input.rs"]
mod input;
pub struct HarnessView {
    theme: Theme,
    theme_revision: u64,
    inputs: input::Inputs,
    pub composer: super::editor::Input,
    approval: Option<(ItemId, ui::modules::approval::Approval)>,
    completion: Option<ui::text_edit::CompletionPopover>,
    profile_menu: Option<super::menu::ProfileMenu>,
    setup_open: bool,
    setup_dismissed: bool,
    setup_visible: bool,
    setup_details: std::collections::BTreeSet<crate::onboarding::Agent>,
    inactive_scroll: chat_scroll::State,
    config_open: Option<String>,
    config_selected: usize,
    config_scroll: f32,
    config_menu_rect: Option<Rect>,
    pub scroll: chat_scroll::State,
    /// Stay at the latest prepared tail through initial replay/reconnection.
    tail_pending: bool,
    /// View-local offsets into worker-prepared Markdown tool panels.
    markdown_scroll: HashMap<String, [f32; 2]>,
    metrics: Metrics,
    cached: Option<CacheKey>,
    controls: Option<Controls>,
    retained_prepared: Option<Arc<PreparedRows>>,
    pub frame: Frame,
    pub notice: Option<String>,
    /// The window holds keyboard focus: the composer draws its focus ring and caret.
    pub focused: bool,
    /// Profiles for `/profile` completion and the switcher, from the latest frame.
    profiles: Arc<Vec<ProfileInfo>>,
    /// The session's current profile name, from the latest frame.
    current_profile: Option<String>,
    /// The actor birth the latest frame showed: `/stop` is fenced to it, as the header
    /// Stop button was, so a stop typed against one actor never closes its replacement.
    birth: Option<Epoch>,
    /// The window's animation clock (`anim.rs`). Settled by default: headless paths see end
    /// states; the window installs a live clock.
    pub anim: Clock,
    /// Rows a frame of this session has shown; a row not in it enters with the rise.
    known: HashSet<ItemId>,
    known_scope: Option<(Option<SessionId>, Epoch)>,
    /// Rows whose entrance is in flight.
    entering: HashSet<ItemId>,
    /// Clock time of the last composer edit: the caret stays solid for 500 ms after it.
    last_edit: f32,
    /// Menu motion state: the card the last frame painted, the row under the
    /// pointer and the closed card still fading out.
    menus: Menus,
    /// Stream and card motion: what is new (chunks, folds, ended turns) and the frame pass.
    pub motion: stream_motion::Motion,
    /// The standalone window draws the term's title bar above the face (harness-spec.md
    /// Standalone window), also used by full-window captures. Embedded views omit it.
    pub chrome: bool,
    /// The title bar's hit regions, retained from the solved tree each frame.
    pub titlebar: chrome::Titlebar,
    /// Global text zoom (harness-spec.md Standalone window; `zoom.rs`).
    zoom: Zoom,
    /// The zoom pill: the clock time the zoom last changed and a count keying its fade.
    zoom_shown: Option<(f32, u32)>,
    /// The row at the top of the viewport when the zoom changed and its offset from the
    /// viewport's top; the first layout at the new zoom puts it back there.
    anchor: Option<(ItemId, f32)>,
    /// Focus swells fired so far (keys the swell tween); 0 while none is running.
    swell: u32,
    was_focused: bool,
}
/// Which floating menu is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuKind {
    Slash,
    Profiles,
    Config,
}
impl MenuKind {
    fn id(self) -> &'static str {
        match self {
            MenuKind::Slash => "menu/slash",
            MenuKind::Profiles => "menu/profiles",
            MenuKind::Config => "menu/config",
        }
    }
}
/// A painted card: its kind, solved layout, selection highlight and opacity.
struct Card {
    kind: MenuKind,
    layout: c::Layout,
    sel: Option<Rect>,
    opacity: f32,
}
#[derive(Default)]
struct Menus {
    /// The card the last frame painted.
    last: Option<Card>,
    /// The closed card, fading out over 120ms `std` until it is gone.
    ghost: Option<Card>,
    /// The hover zone under the pointer: a menu row, a message row or its source toggle.
    hover: Option<String>,
    /// The hover changed: the frame must be rebuilt although nothing else did.
    dirty: bool,
}
/// harness-spec.md "Refinement pass 2": the message source toggle fades over 150ms `std`.
const SOURCE_HOVER_MS: f32 = 150.;
/// harness-spec.md Motion table, the menu rows: open 170ms `out`, close
/// 120ms `std`, selection 120ms `std`, hover 150ms `std`.
/// scripts/test-motion-control.py zeroes these: every tween must then snap
/// and tests/gui_menu_motion.rs must go red.
const MENU_OPEN_MS: f32 = 170.;
const MENU_CLOSE_MS: f32 = 120.;
const MENU_SELECT_MS: f32 = 120.;
const MENU_HOVER_MS: f32 = 150.;
/// One frame of a menu's motion: the card's opacity, drop and scale, and the
/// sliding selection highlight.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuMotion {
    pub opacity: f32,
    pub dy: f32,
    pub scale: f32,
    pub sel: Option<Rect>,
}
impl Default for HarnessView {
    fn default() -> Self {
        let composer = super::editor::Input::new(composer::Composer {
            focus: true,
            to: vec![composer::Address {
                kind: "seat".into(),
                label: "Agent".into(),
                destination: "session".into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        Self {
            theme: Theme::Dark,
            theme_revision: 0,
            inputs: Default::default(),
            composer,
            approval: None,
            completion: None,
            profile_menu: None,
            setup_open: false,
            setup_dismissed: false,
            setup_visible: false,
            setup_details: Default::default(),
            inactive_scroll: {
                let mut s = chat_scroll::State::default();
                s.anchor(0., f32::MAX);
                s
            },
            config_open: None,
            config_selected: 0,
            config_scroll: 0.,
            config_menu_rect: None,
            scroll: Default::default(),
            tail_pending: true,
            markdown_scroll: HashMap::new(),
            metrics: Metrics::default(),
            cached: None,
            controls: None,
            retained_prepared: None,
            frame: Frame::default(),
            notice: None,
            focused: false,
            profiles: Arc::new(vec![]),
            current_profile: None,
            birth: None,
            anim: Clock::default(),
            known: HashSet::new(),
            known_scope: None,
            entering: HashSet::new(),
            last_edit: 0.,
            menus: Menus::default(),
            motion: stream_motion::Motion::default(),
            chrome: false,
            titlebar: chrome::Titlebar::default(),
            zoom: Zoom::default(),
            zoom_shown: None,
            anchor: None,
            swell: 0,
            was_focused: false,
        }
    }
}
/// The zoom pill's life on the window clock (lab.css .hx-zoom): drop in over 170 ms `out`,
/// hold 900 ms, fade out over 150 ms.
const PILL_IN_MS: f32 = 170.;
const PILL_HOLD_MS: f32 = 900.;
const PILL_OUT_MS: f32 = 150.;
impl HarnessView {
    pub fn theme(&self) -> Theme {
        self.theme
    }
    pub fn context(&self) -> ui::Context {
        ui::Context::new(self.theme, Density::Cozy)
    }
    /// Presentation identity is per view; a returning palette is a fresh preparation.
    pub fn set_theme(&mut self, theme: Theme) -> bool {
        if self.theme == theme {
            return false;
        }
        self.theme = theme;
        self.theme_revision = self.theme_revision.wrapping_add(1);
        self.cached = None;
        self.controls = None;
        self.retained_prepared = None;
        self.menus.last = None;
        self.menus.ghost = None;
        true
    }
    /// The live standalone window and offscreen captures share this chrome and text scale.
    /// Startup zoom creates no transient pill and performs no settings IO.
    pub fn standalone(zoom: Zoom) -> Self {
        Self {
            chrome: true,
            zoom,
            ..Self::default()
        }
    }
    /// A wheel over an inset first scrolls that document, then the transcript at its edge.
    pub fn wheel(&mut self, at: [f32; 2], dy: f32) -> bool {
        self.wheel_delta(at, [0., dy])
    }
    /// Both axes stay view-local; horizontal arrows also serve embedded hosts with a vertical-only wheel API.
    pub fn wheel_delta(&mut self, at: [f32; 2], mut delta: [f32; 2]) -> bool {
        if self.config_menu_rect.is_some_and(|r| contains(r, at)) {
            self.config_scroll = (self.config_scroll + delta[1]).max(0.);
            self.invalidate();
            return true;
        }
        fn panel(node: &c::Layout, at: [f32; 2]) -> Option<String> {
            if let c::LayoutKind::Named(name) = &node.kind
                && scroll_panel(name)
                && contains(node.rect, at)
            {
                return Some(name.clone());
            }
            node.children.iter().find_map(|n| panel(n, at))
        }
        if let Some(key) = self
            .frame
            .layout
            .as_ref()
            .and_then(|root| find(root, "transcript"))
            .filter(|n| contains(n.rect, at))
            .and_then(|n| panel(n, at))
        {
            // A single long line is horizontally scrollable even in an older embedded host.
            if let Some(n) = self
                .frame
                .layout
                .as_ref()
                .and_then(|r| find(r, &key))
                .and_then(|n| n.children.first())
                && let c::LayoutKind::ScrollView { content, .. } = n.kind
                && content[1] <= n.rect.h
                && content[0] > n.rect.w
                && delta[0] == 0.
            {
                delta = [delta[1], 0.];
            }
            if self.shift_panel(&key, delta) {
                return true;
            }
        }
        if self
            .scroll
            .wheel(self.frame.scroll, self.frame.max_scroll, delta[1])
        {
            self.invalidate();
            return true;
        }
        false
    }
    fn shift_panel(&mut self, key: &str, delta: [f32; 2]) -> bool {
        let Some(node) = self
            .frame
            .layout
            .as_ref()
            .and_then(|r| find(r, key))
            .and_then(|n| n.children.first())
        else {
            return false;
        };
        let c::LayoutKind::ScrollView { content, offset } = node.kind else {
            return false;
        };
        let next = [
            (offset[0] + delta[0]).clamp(0., (content[0] - node.rect.w).max(0.)),
            (offset[1] + delta[1]).clamp(0., (content[1] - node.rect.h).max(0.)),
        ];
        if next == offset {
            return false;
        }
        self.markdown_scroll.insert(key.to_owned(), next);
        self.invalidate();
        true
    }

    pub fn zoom(&self) -> Zoom {
        self.zoom
    }
    /// Cmd+= / Cmd+- / Cmd+0 (harness-spec.md Standalone window): take `zoom`, show the
    /// pill, anchor the scroll to the top visible row, and ask the bridge for rows at the
    /// new size. The pill shows even at the range's end; true when the zoom changed.
    pub fn set_zoom(&mut self, zoom: Zoom, bridge: &Bridge) -> bool {
        let n = self.zoom_shown.map_or(1, |(_, n)| n + 1);
        self.zoom_shown = Some((self.anim.now(), n));
        let changed = self.restore_zoom(zoom, bridge);
        self.invalidate();
        changed
    }
    /// The saved zoom at startup: applied with no pill and no anchor.
    pub fn restore_zoom(&mut self, zoom: Zoom, bridge: &Bridge) -> bool {
        if zoom == self.zoom {
            return false;
        }
        self.anchor = self.top_visible();
        self.zoom = zoom;
        bridge.set_zoom(zoom.percent());
        self.invalidate();
        true
    }
    /// The first row the last frame showed, with its top's offset from the viewport's top.
    fn top_visible(&self) -> Option<(ItemId, f32)> {
        let layout = self.frame.layout.as_ref()?;
        let viewport = find(layout, "transcript")?.children.first()?.rect;
        let (id, ..) = self.frame.visible.first()?;
        let row = find(layout, &format!("item/{id}"))?.rect;
        Some((*id, row.y - viewport.y))
    }
    /// Running with an empty draft, the send button is the stop square (.hx-send.stop).
    pub fn send_stops(&self, state: &FrameState) -> bool {
        !state.replay
            && !matches!(state.transcript.phase, Phase::Idle | Phase::Closed)
            && self.composer.view.text.is_empty()
            && self.composer.view.attach.is_empty()
    }
    pub fn invalidate(&mut self) {
        self.cached = None;
    }
    /// The transcript column: one fixed-height slot per prepared row (heights tween over
    /// 200 ms `out` near the viewport; rows new to this session enter with the rise), or
    /// the empty session, or a loading label. Returns the element, its content height when
    /// it is a row column, and the entrance progress of rows still rising.
    fn transcript(
        &mut self,
        state: &FrameState,
        prepared: Option<&Arc<PreparedRows>>,
        rows_h: f32,
        width: f32,
        first_frame: bool,
        ink: &super::face::Ink,
    ) -> Result<TranscriptColumn, String> {
        let model = &state.transcript;
        let has_rows = model.order.iter().any(|id| face_rows::drawn(model, id));
        // Discovery is independent of session loading. Only a settled, explicitly
        // unconfigured new session may select setup automatically; absent catalogue
        // or metadata is not evidence that an existing session needs onboarding.
        // A failed first profile resolution is a settled new-session welcome,
        // not attached history. Keep its diagnostic visible in the setup panel.
        let startup_error = state.created_here
            && model.profile.is_none()
            && has_rows
            && model
                .items
                .values()
                .all(|row| row.completed.is_some() && matches!(row.body, Item::Notice { .. }));
        let welcome = state.ready
            && !state.replay
            && model.meta.is_some()
            && (model
                .profile
                .as_ref()
                .is_some_and(|(_, name)| name == "unconfigured")
                || startup_error)
            && (!has_rows || startup_error)
            && model.turns.is_empty()
            && state.pending_prompts.is_empty();
        let show_setup = self.setup_open
            || (!self.setup_dismissed && welcome && !state.onboarding.entries.is_empty());
        if show_setup != self.setup_visible {
            std::mem::swap(&mut self.scroll, &mut self.inactive_scroll);
            self.setup_visible = show_setup;
        }
        if show_setup {
            let notices: Vec<_> = if startup_error {
                model
                    .items
                    .values()
                    .filter_map(|row| match &row.body {
                        Item::Notice { text, .. } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect()
            } else {
                vec![]
            };
            let element = super::setup::element(
                &state.onboarding,
                self.context(),
                width,
                &self.setup_details,
                &notices,
            );
            let height = element
                .layout(Rect::new(0., 0., width, 0.), self.context(), &self.metrics)?
                .rect
                .h;
            return Ok((element, Some(height), HashMap::new()));
        }
        let mut children = vec![];
        let mut total = 0.;
        let mut enters = HashMap::new();
        if let Some(prepared) = prepared {
            // animate only what is visible (plus a margin): offscreen rows take end states
            let animating = self.anim.animates();
            let (view_top, view_bottom) =
                (self.frame.scroll - 240., self.frame.scroll + rows_h + 240.);
            let mut anchor_top = None;
            for id in &model.order {
                let Some(row) = prepared.rows.get(id) else {
                    continue;
                };
                if self.anchor.is_some_and(|(a, _)| a == *id) {
                    anchor_top = Some(total);
                }
                let h0 = row.layout.rect.h;
                if h0 == 0. {
                    continue;
                }
                let near = total + h0 >= view_top && total <= view_bottom;
                let h = if animating && near {
                    self.anim.tween(&format!("h/{id}"), h0, 200., Ease::Out)
                } else {
                    h0
                };
                if !first_frame && animating && self.known.insert(*id) {
                    self.entering.insert(*id);
                } else {
                    self.known.insert(*id);
                }
                if self.entering.contains(id) {
                    let t = self
                        .anim
                        .enter(&format!("in/{id}"), 0., 1., 320., Ease::Out);
                    if t >= 1. || !near {
                        self.entering.remove(id);
                    } else {
                        enters.insert(*id, t);
                    }
                }
                children.push(
                    named(format!("cached/{id}"), c::Element::Spacer)
                        .fixed_height(h, id.to_string(), 0.)
                        .into(),
                );
                total += h;
            }
            // The zoom changed: once the rows are prepared at the new size, the row that was
            // at the top of the viewport goes back to the same offset (harness-spec.md
            // Standalone window: "the scroll position stays anchored to the top visible block").
            if let Some((_, dy)) = self.anchor
                && prepared.zoom == self.zoom.percent()
            {
                if let Some(top) = anchor_top {
                    self.scroll.anchor(top - dy, (total - rows_h).max(0.));
                }
                self.anchor = None;
            }
        }
        for prompt in state
            .pending_prompts
            .values()
            .filter(|p| Some(p.session) == model.session && p.epoch == model.epoch)
        {
            let solved = prepared.and_then(|p| p.pending.get(&prompt.op));
            let (element, height) = if let Some(solved) = solved {
                (
                    named(format!("pending/{}", prompt.op), c::Element::Spacer),
                    solved.rect.h,
                )
            } else {
                // Adapted from this project's pre-Markdown pending-row path:
                // https://github.com/bboxy450/bboxy450-harness/blob/a1ad9fbd/src/frontends/gui/view.rs
                // Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
                // A bounded plain preview remains visible before worker formatting.
                // Keep the complete prompt unchanged; never parse Markdown here.
                let mut chars = prompt.text.chars();
                let mut preview = chars.by_ref().take(160).collect::<String>();
                if chars.next().is_some() {
                    preview.push('…');
                }
                let content =
                    c::Element::Text(super::face::text(preview, 13.5, 400, ink.fg, false));
                let bubble = super::face::prompt_bubble(
                    content,
                    &format!("YOU · {}", prompt.status),
                    true,
                    ink,
                );
                let gutter = (14_f32 - 4.).max(self.context().density.tokens().gap as f32);
                let mut element = named(
                    format!("pending/{}", prompt.op),
                    super::face::frame(
                        super::face::Frame {
                            padding: [4., gutter + 4., 18., gutter + 4.],
                            ..Default::default()
                        },
                        vec![bubble.into()],
                    ),
                );
                super::face::zoom_text(&mut element, self.zoom.scale());
                let height = element
                    .layout(Rect::new(0., 0., width, 0.), self.context(), &self.metrics)?
                    .rect
                    .h;
                (element, height)
            };
            total += height;
            // Every sibling must declare its height. An Auto pending row makes
            // the toolkit fall back to natural measurement, where the cached
            // Spacer slots measure zero and clip all retained history paint.
            children.push(c::Item {
                element,
                extent: c::Extent::Fixed(height),
            });
        }
        let has_pending = !children.is_empty();
        if children.is_empty() {
            if state.ready && !has_rows && model.meta.is_some() && model.profile.is_some() {
                // harness-spec.md "Empty session": the page's `who` is the ACP agent line, else
                // the model, else the profile; `in ~/<cwd leaf>` when the session has a cwd
                let (profile, model_name) = model.profile.clone().unwrap_or_else(|| {
                    (
                        model
                            .meta
                            .as_ref()
                            .and_then(|m| m.profile.clone())
                            .unwrap_or_default(),
                        String::new(),
                    )
                });
                let who = match super::face::acp_model(&model_name) {
                    Some((agent, m)) => {
                        let p = if profile.is_empty() { agent } else { profile };
                        if m.is_empty() {
                            p
                        } else {
                            format!("{p} · {m}")
                        }
                    }
                    None if !model_name.is_empty() => model_name,
                    None => profile,
                };
                let leaf = model
                    .meta
                    .as_ref()
                    .and_then(|m| m.cwd.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty());
                let mut empty = super::face::empty_session(
                    (!who.is_empty()).then_some(who),
                    leaf,
                    (0.12 * rows_h).max(0.),
                    ink,
                    &mut self.anim,
                );
                super::face::zoom_text(&mut empty, self.zoom.scale());
                children.push(empty.into());
            } else {
                let mut message = super::face::loading_session(state.ready, ink, &mut self.anim);
                super::face::zoom_text(&mut message, self.zoom.scale());
                let centered = super::face::frame(
                    super::face::Frame {
                        row: true,
                        ..Default::default()
                    },
                    vec![
                        super::face::fill(c::Element::Spacer),
                        message.into(),
                        super::face::fill(c::Element::Spacer),
                    ],
                );
                let height = centered
                    .layout(Rect::new(0., 0., width, 0.), self.context(), &self.metrics)?
                    .rect
                    .h;
                children.push(c::Item {
                    element: c::Element::Spacer,
                    extent: c::Extent::Fixed(((rows_h - height) / 2.).max(0.)),
                });
                children.push(centered.into());
            }
        }
        if (prepared.is_some() && has_rows) || has_pending {
            Ok((
                column(children, None).fixed_height(total, "transcript-content", 0.),
                Some(total),
                enters,
            ))
        } else {
            Ok((column(children, None), None, enters))
        }
    }
    /// The scroll offset for this frame: the tail follow glides there with a critically
    /// damped ~180 ms spring that retargets as content grows; a manual position is exact.
    fn scroll_offset(&mut self, content_h: Option<f32>, rows_h: f32, first_frame: bool) -> f32 {
        if !self.scroll.following() {
            return self.scroll.layout_offset();
        }
        let Some(content_h) = content_h else {
            return 0.;
        };
        let max = (content_h - rows_h).max(0.);
        if first_frame || self.tail_pending {
            // a session's first frame shows its tail at once
            self.anim.forget_spring("scroll");
            return max;
        }
        self.anim
            .spring_from("scroll", self.frame.scroll.min(max), max, 180.)
    }
    /// The same Element is used by a standalone surface and the offscreen driver.
    pub fn render(&mut self, state: &FrameState, size: [f32; 2]) -> Result<&Frame, String> {
        self.poll_inputs(state);
        self.config_menu_rect = None;
        self.profiles = state.profiles.clone();
        self.current_profile = state.transcript.profile.as_ref().map(|(p, _)| p.clone());
        self.birth = Some(state.transcript.epoch);
        let offset_key = self.scroll.layout_offset();
        // the caret blink only shows while the composer is focused; otherwise it must not
        // keep busting the idle frame cache as the clock advances
        let blink = self.focused && self.anim.blink_at(1000., self.anim.now() - self.last_edit);
        // the zoom pill's phase: in or holding, fading, gone (a timed wake ends the hold)
        let pill = self.zoom_shown.map_or(0, |(t0, _)| {
            let since = self.anim.now() - t0;
            if since < PILL_HOLD_MS {
                1
            } else if since < PILL_HOLD_MS + PILL_OUT_MS {
                2
            } else {
                0
            }
        });
        let draft = format!(
            "{:?}{:?}{:?}{:?}{}{:?}{blink}{}{}{pill}",
            self.composer.view,
            self.notice,
            state.error,
            state.notice,
            self.focused,
            self.profile_menu,
            self.zoom.percent(),
            self.chrome,
        );
        let draft = format!(
            "{draft}{:p}{:p}{}{:?}{}",
            Arc::as_ptr(&state.pending_prompts),
            Arc::as_ptr(&state.onboarding),
            self.setup_open,
            self.setup_details,
            state.created_here
        );
        // a hover change rebuilds too: the menu row's fill starts its fade
        if !self.anim.in_motion()
            && !self.menus.dirty
            && self
                .cached
                .as_ref()
                .is_some_and(|(model, text, s, o, ready, prepared)| {
                    Arc::ptr_eq(model, &state.transcript)
                        && text == &draft
                        && s == &size
                        && *o == offset_key
                        && *ready == state.ready
                        && match (prepared, &state.prepared) {
                            (Some(prior), Some(next)) => Arc::ptr_eq(prior, next),
                            (None, None) => true,
                            _ => false,
                        }
                })
        {
            return Ok(&self.frame);
        }
        let timer = std::time::Instant::now();
        let ctx = self.context();
        let model = &state.transcript;
        let now = self.anim.now();
        self.motion.begin_frame(state, now);
        let width = size[0].max(240.);
        // Local/metadata publication may wrap the same log in another Arc, or
        // arrive while worker preparation is pending. Keep the last solved rows
        // of this birth until their replacement is ready; never retain across
        // a session/epoch reset or a cursor rewind.
        let compatible = |p: &&Arc<PreparedRows>| {
            p.theme == self.theme
                && p.theme_revision == self.theme_revision
                && p.source.session == model.session
                && p.source.epoch == model.epoch
                && p.source.cursor <= model.cursor
        };
        let prepared = state
            .prepared
            .as_ref()
            .filter(compatible)
            .or_else(|| self.retained_prepared.as_ref().filter(compatible))
            .cloned();
        self.retained_prepared = prepared.clone();
        let prepared = prepared.as_ref();
        // A session's first frame takes its end state: nothing enters, the tail is shown.
        let scope = (model.session, model.epoch);
        let first_frame = self.known_scope != Some(scope);
        if first_frame {
            self.follow_latest();
            self.known_scope = Some(scope);
            self.known.clear();
            self.entering.clear();
            self.markdown_scroll.clear();
        }
        let ink = super::face::ink(ctx);
        let row_slots = timer.elapsed();
        let status = self
            .notice
            .as_ref()
            .or(state.error.as_ref())
            .or(state.notice.as_ref())
            .cloned()
            .map(|s| (s, super::face::Tone::Idle))
            .unwrap_or_else(|| {
                if let Some(tag) = &model.last_unknown_tag {
                    (
                        format!("Unsupported event: {tag} ({} total)", model.unknown),
                        super::face::Tone::Idle,
                    )
                } else {
                    phase_now(model)
                }
            });
        let (status, tone) = status;
        let status = model
            .context
            .as_ref()
            .map_or_else(|| status.clone(), |context| format!("{status} · {context}"));
        // the composer's states (p2) follow the live phase; the header shows `tone`

        let live = !matches!(model.phase, Phase::Idle | Phase::Closed) && !state.replay;
        let ask = live && matches!(model.phase, Phase::Approval { .. });
        // the rim reads Errored while nothing runs after a failed turn (or a bridge error)
        let errored = !live
            && (state.error.is_some()
                || matches!(
                    model.turns.values().next_back(),
                    Some(TurnState::Ended {
                        reason: StopReason::Error { .. },
                        ..
                    })
                ));
        let focused = self.focused;
        // the header meter sums every usage item (harness-spec.md "Refinement pass 2")
        let usage = model
            .items
            .values()
            .fold((0u64, 0u64), |acc, r| match &r.body {
                Item::Usage { usage } => (
                    acc.0.saturating_add(usage.input_tokens),
                    acc.1.saturating_add(usage.output_tokens),
                ),
                _ => acc,
            });
        // focus fires the swell (rim-glow §7): one half sine over glow.swell_ms, keyed per
        // swell so a settled swell is forgotten and the next focus starts a new one
        if focused && !self.was_focused {
            self.swell = self.swell.wrapping_add(1).max(1);
        }
        self.was_focused = focused;
        let zoom = self.zoom.scale();
        let controls_key = format!(
            "{:?}",
            model
                .agent_config
                .as_ref()
                .map(|c| (c.instance, c.revision, c.pending, &c.status))
        ) + &format!(
            "{draft}{status}{tone:?}{live}{errored}{focused}{}{}{:?}{:?}{:?}{usage:?}",
            self.swell,
            self.profile_menu.is_some(),
            state.ready,
            model.profile,
            model.meta.as_ref().map(|m| &m.cwd)
        ) + &format!("{:?}", self.config_open);
        // While the clock moves something the controls are rebuilt every frame (the composer
        // reads its tweens as it is built); at rest they are reused by key.
        let reuse = !self.anim.in_motion()
            && self
                .controls
                .as_ref()
                .is_some_and(|c| c.key == controls_key && c.size == size);
        let (mut layout, rows_h, enters) = if reuse {
            let (mut layout, rows_h) = {
                let controls = self.controls.as_ref().unwrap();
                (controls.layout.clone(), controls.rows_h)
            };
            let (transcript, content_h, enters) =
                self.transcript(state, prepared, rows_h, width, first_frame, &ink)?;
            let offset = self.scroll_offset(content_h, rows_h, first_frame);
            let slot = find_mut(&mut layout, "transcript")
                .and_then(|n| n.children.first_mut())
                .ok_or("transcript slot missing")?;
            *slot = c::Element::ScrollView(c::ScrollView {
                viewport: [slot.rect.w, slot.rect.h],
                offset: [0., offset],
                content_width: Some(width),
                content: Box::new(transcript),
            })
            .layout(slot.rect, ctx, &self.metrics)?;
            (layout, rows_h, enters)
        } else {
            let swell = if self.swell > 0 {
                let p = self.anim.enter(
                    &format!("composer/swell/{}", self.swell),
                    0.,
                    1.,
                    super::face::SWELL_MS,
                    Ease::Linear,
                );
                if p >= 1. {
                    self.swell = 0;
                }
                super::face::swell_envelope(p)
            } else {
                0.
            };
            let composer = super::face::composer(
                &self.composer.view,
                super::face::ComposerPhase {
                    live: live && !ask,
                    ask,
                    errored,
                    focused,
                    swell,
                },
                ctx,
                width,
                zoom,
                &self.metrics,
                &mut self.anim,
            )?;
            let composer = self.attachment_panel(composer, width);
            let composer_h = composer
                .layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?
                .rect
                .h;
            let leaf = model
                .meta
                .as_ref()
                .and_then(|m| m.cwd.file_name())
                .map(|n| n.to_string_lossy().into_owned());
            let base_header = super::face::header(
                model.profile.clone(),
                leaf.clone(),
                &status,
                tone,
                self.profile_menu.is_some(),
                usage,
                &ink,
            );
            let mut selectors = Vec::new();
            if let Some(config) = &model.agent_config {
                for (category, title) in [("model", "Model"), ("thought_level", "Effort")] {
                    if let Some(option) = config.options.iter().find(|o| o.category == category) {
                        let value = option
                            .choices
                            .iter()
                            .find(|c| c.value == option.current)
                            .map(|c| c.name.as_str())
                            .unwrap_or(&option.current);
                        selectors.push(super::face::fixed(super::menu::select_chip(
                            &format!("config/{category}"),
                            if width < 760. { "" } else { title },
                            value,
                            self.config_open.as_deref() == Some(category),
                            config.pending.is_none() && !state.replay,
                            &ink,
                        )));
                    }
                }
            }
            let mut inline = base_header.clone();
            if let c::Element::Box(panel) = &mut inline {
                let at = if model.profile.is_some() { 3 } else { 2 };
                panel.children.splice(at..at, selectors.clone());
                panel.gap = ui::Length::Px(6.);
            }
            let mut measured = inline.clone();
            super::face::zoom_text(&mut measured, zoom);
            let required = measured.natural(f32::INFINITY, ctx, &self.metrics)?.0[0];
            let mut header = if selectors.is_empty() || required <= width {
                inline
            } else {
                let controls = super::face::frame(
                    super::face::Frame {
                        fill: super::face::alpha(ink.seat, 0.06),
                        padding: [0., 12., 8., 12.],
                        gap: 8.,
                        row: width >= 360.,
                        ..Default::default()
                    },
                    selectors,
                );
                column(vec![base_header.into(), controls.into()], None)
            };
            header = named("toolbar", header);
            super::face::zoom_text(&mut header, zoom);
            // 40 px at 100 %; larger type reflows the chips and the bar grows with them
            let header_h = header
                .layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?
                .rect
                .h
                .max(40.);
            // the standalone window's own title bar (chrome.rs) above the face
            let bar_h = if self.chrome { chrome::TITLE_BAR_H } else { 0. };
            let rows_h = (size[1] - composer_h - header_h - bar_h).max(1.);
            let (transcript, content_h, enters) =
                self.transcript(state, prepared, rows_h, width, first_frame, &ink)?;
            let offset = self.scroll_offset(content_h, rows_h, first_frame);
            let rows = named(
                "transcript",
                c::Element::ScrollView(c::ScrollView {
                    viewport: [width, rows_h],
                    offset: [0., offset],
                    content_width: Some(width),
                    content: Box::new(transcript),
                }),
            );
            let mut children = vec![];
            if self.chrome {
                let profile = model
                    .profile
                    .as_ref()
                    .map(|(p, _)| p.clone())
                    .or_else(|| model.meta.as_ref().and_then(|m| m.profile.clone()));
                children.push(c::Item {
                    element: chrome::title_bar(
                        &chrome::crumb(profile.as_deref(), leaf.as_deref()),
                        focused,
                        ctx,
                    ),
                    extent: c::Extent::Fixed(bar_h),
                });
            }
            children.extend([
                c::Item {
                    element: header,
                    extent: c::Extent::Fixed(header_h),
                },
                c::Item {
                    element: rows,
                    extent: c::Extent::Fixed(rows_h),
                },
                composer.into(),
            ]);
            let root = column(children, Some(size[1]));

            let mut layout = root.layout(Rect::new(0., 0., width, size[1]), ctx, &self.metrics)?;
            paint_shapes(&mut layout);
            self.controls = Some(Controls {
                key: controls_key,
                size,
                layout: layout.clone(),
                rows_h,
            });
            (layout, rows_h, enters)
        };
        let controls = timer.elapsed();
        let solved = timer.elapsed();
        // the orbit arcs and the sheen, painted over the rows inside the viewport
        let mut overlays: Option<(Rect, Vec<bbt_ui_model::draw::Prim>)> = None;
        if let Some(prepared) = prepared {
            let viewport = find(&layout, "transcript")
                .and_then(|n| n.children.first())
                .map(|n| n.rect)
                .ok_or("transcript viewport missing")?;
            // Streaming rows: each chunk's glyphs fade in over 140ms `out` (harness-spec.md
            // Motion). The tracker compares the row's glyphs with the last frame's, so text
            // already on screen never re-fades; the visitor splits runs at chunk boundaries.
            let animating = self.anim.animates();
            let frame_no = self.motion.frame;
            let (anim, motion, metrics, markdown_scroll) = (
                &mut self.anim,
                &mut self.motion,
                &self.metrics,
                &self.markdown_scroll,
            );
            restore_pending(&mut layout, &prepared.pending, viewport, markdown_scroll);
            restore_rows(
                &mut layout,
                &prepared.rows,
                viewport,
                &enters,
                markdown_scroll,
                &mut |id| {
                    if !animating || !motion.streaming(state, &id) {
                        return Box::new(stream_motion::Plain);
                    }
                    let Some(row) = prepared.rows.get(&id) else {
                        return Box::new(stream_motion::Plain);
                    };
                    let Some(text) = stream_motion::stream_glyphs(&row.layout) else {
                        return Box::new(stream_motion::Plain);
                    };
                    let chunks: Vec<(usize, f32)> = motion
                        .fades
                        .observe(id, &text, now, frame_no)
                        .into_iter()
                        .map(|(start, _)| {
                            let alpha = anim.enter(
                                &format!("fade/{id}/{start}"),
                                0.,
                                1.,
                                stream_motion::FADE_MS,
                                Ease::Out,
                            );
                            (start, alpha)
                        })
                        .collect();
                    if chunks.is_empty() {
                        Box::new(stream_motion::Plain)
                    } else {
                        Box::new(stream_motion::Fader::new(chunks, metrics))
                    }
                },
            );
            let prims = self.motion.pass(&mut self.anim, &mut layout, state, &ink);
            if !prims.is_empty() {
                overlays = Some((viewport, prims));
            }
        }
        // Hover-only controls, each over 150 ms `std`; a settled clock answers the end state,
        // so captures and the parity gate never show them. Hit zones (the named nodes'
        // rects) stay.
        // - The message source toggle (harness-spec.md "Refinement pass 2"): nothing at
        //   rest, .6 while its row is under the pointer, 1 over the control itself.
        // - A settled thought's chevron (harness-spec.md "Tight transcript"): nothing at
        //   rest, shown while the row is under the pointer or the thought is open.
        {
            let hovered = self.menus.hover.as_deref();
            let anim = &mut self.anim;
            let faces = &state.faces;
            each_named(&mut layout, &mut |key, node| {
                let target = if key.starts_with("source/") && key.ends_with("/toggle") {
                    let row = key
                        .strip_prefix("source/")
                        .and_then(|k| k.strip_suffix("/toggle"))
                        .unwrap_or(key);
                    if hovered == Some(key) {
                        1.
                    } else if hovered == Some(format!("msg/{row}").as_str()) {
                        0.6
                    } else {
                        0.
                    }
                } else if key.starts_with("reasoning/") && key.ends_with("/toggle/chev") {
                    let row = key.strip_suffix("/chev").unwrap_or(key);
                    let open = key
                        .strip_prefix("reasoning/")
                        .and_then(|k| k.split('/').next())
                        .and_then(|s| s.parse().ok())
                        .map(ItemId)
                        .is_some_and(|id| faces.get(&id).is_some_and(|f| f.reasoning_open));
                    if open || hovered == Some(row) { 1. } else { 0. }
                } else {
                    return false;
                };
                let a = anim.tween(&format!("{key}/hover"), target, SOURCE_HOVER_MS, Ease::Std);
                if a <= 0. {
                    node.children.clear();
                } else {
                    stream_motion::fade_node(node, a);
                }
                true
            });
        }
        // the empty session's rise and its chips' staggered rise (opacity is in the element)
        for (name, key) in [
            ("empty", "empty/in"),
            ("empty/chip/0", "empty/chip/0"),
            ("empty/chip/1", "empty/chip/1"),
            ("empty/chip/2", "empty/chip/2"),
        ] {
            if let Some(t) = self.anim.value(key)
                && t < 1.
                && let Some(node) = find_mut(&mut layout, name)
            {
                translate(node, 0., 6. * (1. - t));
            }
        }
        // the caret glides to its column over 80 ms, stays solid while typing and blinks
        // after 500 ms idle
        if let Some(node) = find_mut(&mut layout, "composer/text/caret") {
            let r = node.rect;
            let x = self.anim.tween("caret/x", r.x, 80., Ease::Out);
            let y = self.anim.tween("caret/y", r.y, 80., Ease::Out);
            translate(node, x - r.x, y - r.y);
            if !self.anim.blink(1000., self.anim.now() - self.last_edit) {
                node.children.clear();
            }
        }
        let restored = timer.elapsed();
        self.titlebar.retain(&layout);
        let mut frame = Frame {
            theme: self.theme,
            ..Default::default()
        };
        frame.list.rect(
            Rect::new(0., 0., width, size[1]),
            0.,
            Color::hex(ctx.theme.palette().bg),
        );
        for paint in c::paint(&layout, ctx) {
            append_paint(&mut frame.list, &mut frame.rims, paint)?;
        }
        if self.anim.animates() && !ask {
            let viewport = find(&layout, "transcript")
                .map(|n| n.rect)
                .unwrap_or(Rect::new(0., 0., width, size[1]));
            each_named(&mut layout, &mut |key, node| {
                let Some(raw) = key
                    .strip_prefix("tool/")
                    .and_then(|k| k.strip_suffix("/status"))
                    .or_else(|| {
                        key.strip_prefix("reasoning/")
                            .and_then(|k| k.strip_suffix("/toggle/icon"))
                    })
                else {
                    return false;
                };
                let Some(id) = raw.parse().ok().map(ItemId) else {
                    return false;
                };
                if !state
                    .transcript
                    .items
                    .get(&id)
                    .is_some_and(|row| row.status.is_none())
                {
                    return false;
                }
                frame.rims.push(m::Surface {
                    rect: node.rect,
                    radii: [9.; 4],
                    clips: vec![m::Clip {
                        rect: viewport,
                        radii: [0.; 4],
                    }],
                    material: m::Material::Rim(m::Rim {
                        color: Color(ink.seat),
                        waiting: Color(ink.ask),
                        errored: Color(ink.bad),
                        activity: m::Activity::Active,
                        focus: 1.,
                        size: 1.5,
                        depth: 3.,
                        phase_ms: 0.,
                        outer: 0.75,
                        alpha: 1.,
                        swell: 0.,
                        attention: false,
                    }),
                });
                false
            });
        }
        if let Some((viewport, prims)) = overlays {
            frame.list.clip(viewport, [0.; 4]);
            frame.list.primitives.extend(prims);
            frame.list.unclip();
        }
        frame.timings = [
            row_slots,
            controls - row_slots,
            solved - controls,
            restored - solved,
            timer.elapsed() - restored,
        ];
        let scroll_node = find(&layout, "transcript")
            .and_then(|n| n.children.first())
            .ok_or("transcript layout missing")?;
        if let c::LayoutKind::ScrollView { content, offset } = scroll_node.kind {
            frame.scroll = offset[1];
            frame.max_scroll = (content[1] - rows_h).max(0.);
        }
        for id in &model.order {
            if let Some(node) = find(&layout, &format!("item/{id}")) {
                let r = intersection(node.rect, scroll_node.rect);
                if r.w > 0. && r.h > 0. {
                    frame.visible.push((*id, model.items[id].kind.clone(), r));
                }
            }
            // a settled thought row is a hover zone: its chevron shows while hovered
            if model.items[id].kind == ItemKind::Reasoning {
                let key = format!("reasoning/{id}/toggle");
                if let Some(node) = find(&layout, &key) {
                    let r = intersection(node.rect, scroll_node.rect);
                    if r.w > 0. && r.h > 0. {
                        frame.hover.push((r, key));
                    }
                }
            }
            // a message row and its source toggle are hover zones; the toggle is listed
            // first so the pointer over it reads as the control, not only the row
            if model.items[id].kind == ItemKind::AgentMessage {
                let toggle = format!("source/{id}/toggle");
                for (node_key, zone) in [
                    (toggle.clone(), toggle),
                    (format!("item/{id}"), format!("msg/{id}")),
                ] {
                    if let Some(node) = find(&layout, &node_key) {
                        let r = intersection(node.rect, scroll_node.rect);
                        if r.w > 0. && r.h > 0. {
                            frame.hover.push((r, zone));
                        }
                    }
                }
            }
            for (key, toggle) in [
                (
                    format!("reasoning/{id}/toggle"),
                    super::rich::Toggle::Reasoning,
                ),
                (format!("source/{id}/toggle"), super::rich::Toggle::Source),
                (format!("toolhead/{id}"), super::rich::Toggle::Output),
                (format!("more/{id}"), super::rich::Toggle::Arguments),
                (
                    format!("tool/{id}/arguments/toggle"),
                    super::rich::Toggle::Arguments,
                ),
                (
                    format!("tool/{id}/output/toggle"),
                    super::rich::Toggle::Output,
                ),
            ] {
                if let Some(node) = find(&layout, &key) {
                    let r = intersection(node.rect, scroll_node.rect);
                    if r.w > 0. && r.h > 0. {
                        frame.actions.push((r, Action::Toggle(*id, toggle)));
                    }
                }
            }
            for h in 0.. {
                let Some(node) = find(&layout, &format!("diff/{id}/hunk/{h}/toggle")) else {
                    break;
                };
                let r = intersection(node.rect, scroll_node.rect);
                if r.w > 0. && r.h > 0. {
                    frame
                        .actions
                        .push((r, Action::Toggle(*id, super::rich::Toggle::Hunk(h))));
                }
            }
            let row = &model.items[id];
            if let Item::Approval {
                options,
                decision: None,
                ..
            } = &row.body
                && row.status.is_none()
            {
                for option in options {
                    let (text, decision) = approval(option.clone());
                    if let Some(node) = find(&layout, &format!("approval/{id}/{text}")) {
                        let r = intersection(node.rect, scroll_node.rect);
                        if r.w > 0. && r.h > 0. {
                            frame.actions.push((r, Action::Approval(*id, decision)));
                        }
                    }
                }
            }
        }
        fn arrows(node: &c::Layout, out: &mut Vec<(Rect, Action)>) {
            if let c::LayoutKind::Named(name) = &node.kind {
                for (prefix, dx) in [("codeleft/", -160.), ("coderight/", 160.)] {
                    if let Some(key) = name.strip_prefix(prefix) {
                        out.push((node.rect, Action::PanelScroll(key.into(), [dx, 0.])));
                    }
                }
            }
            for child in &node.children {
                arrows(child, out);
            }
        }
        arrows(&layout, &mut frame.actions);
        if !state.replay
            && state
                .transcript
                .agent_config
                .as_ref()
                .is_some_and(|c| c.pending.is_none())
        {
            for category in ["model", "thought_level"] {
                if let Some(node) = find(&layout, &format!("config/{category}")) {
                    frame
                        .actions
                        .push((node.rect, Action::ConfigMenu(category.into())));
                }
            }
        }
        for turn in state.transcript.turns.keys() {
            for (key, action) in [
                (format!("turn/{}/retry", turn.0), Action::Retry(*turn)),
                // an error receipt's Switch profile opens the header's profile switcher
                (format!("turn/{}/switch", turn.0), Action::Profiles),
            ] {
                if let Some(node) = find(&layout, &key) {
                    let r = intersection(node.rect, scroll_node.rect);
                    if r.w > 0. && r.h > 0. {
                        frame.actions.push((r, action));
                    }
                }
            }
        }
        for (key, action) in [
            ("profile", Action::Profiles),
            ("jump", Action::Jump),
            ("interrupt", Action::Interrupt),
            ("stop", Action::StopAt(state.transcript.epoch)),
            ("empty/chip/0", Action::Chip(0)),
            ("empty/chip/1", Action::Chip(1)),
            ("empty/chip/2", Action::Chip(2)),
        ] {
            if let Some(node) = find(&layout, key) {
                frame.actions.push((node.rect, action));
            }
        }
        for prompt in state.pending_prompts.values() {
            let id = ItemId(prompt.op.0);
            if let Some(node) = find(&layout, &format!("source/{id}/toggle")) {
                let r = intersection(node.rect, scroll_node.rect);
                if r.w > 0. && r.h > 0. {
                    frame
                        .actions
                        .push((r, Action::Toggle(id, super::rich::Toggle::Source)));
                    frame.hover.push((r, format!("source/{id}/toggle")));
                }
            }
        }
        for (key, action) in super::setup::actions(&state.onboarding) {
            if let Some(node) = find(&layout, &key) {
                let r = intersection(node.rect, scroll_node.rect);
                if r.w > 0. && r.h > 0. {
                    frame.actions.push((r, action));
                }
            }
        }
        frame.links = Arc::new(ui::modules::document_modal::controller::Regions::retained(
            &layout,
        ));
        frame.layout = Some(layout);
        self.scroll.layout_solved();
        if !self.scroll.following()
            || (state.ready
                && prepared.is_some_and(|p| p.source.cursor == model.cursor
                    && p.width == width && p.zoom == self.zoom.percent()))
        {
            self.tail_pending = false;
        }
        self.frame = frame;
        // the floating menus over everything, with this frame's motion
        self.paint_menus(state, size, rows_h)?;
        // the clock forgets what nobody asked for and decides whether a frame follows
        self.anim.end_frame();
        self.cached = Some((
            state.transcript.clone(),
            draft,
            size,
            offset_key,
            state.ready,
            state.prepared.clone(),
        ));
        Ok(&self.frame)
    }
    /// Paint the floating menus over the frame: the open card with this frame's
    /// motion, its hit zones (front of `actions`) and hover rows, and the card
    /// that closed, fading. Its tweens are read every frame a card shows, so
    /// the clock keeps them and a change tweens from where the value is.
    fn paint_menus(
        &mut self,
        state: &FrameState,
        size: [f32; 2],
        rows_h: f32,
    ) -> Result<(), String> {
        let ctx = self.context();
        let ink = super::face::ink(ctx);
        let width = size[0].max(240.);
        self.paint_zoom_pill(ctx, &ink, width)?;
        let mut hover = vec![];
        let mut front = vec![];
        if let Some(layout) = &self.frame.layout {
            for i in 0..self.inputs.attachments.len() {
                if let Some(node) = find(layout, &format!("attachment/remove/{i}")) {
                    front.push((node.rect, Action::RemoveAttachment(i)));
                }
            }
            if let Some(node) = find(layout, "composer/attach") {
                front.push((node.rect, Action::Chip(1)));
            }
        }
        let hovered = self.menus.hover.clone();
        let open = if let Some(popover) = &self.completion {
            // Spec: left 12, bottom 4px into the composer's top padding.
            let bottom = 40. + rows_h + 4.;
            // 32 px rows under ~84 px of labels, insets and footer; at most 8
            let max_rows = ((bottom - 44. - 84.) / 32.).floor().max(1.) as usize;
            let selected = super::menu::slash_selected_key(popover);
            let keys: Vec<String> = (0..popover.choices.options.len())
                .map(|i| format!("completion/{i}"))
                .collect();
            let look = look(
                &mut self.anim,
                &keys,
                selected.as_deref(),
                hovered.as_deref(),
            );
            // The card's surface_2 fill is opaque: transcript ink never shows through.
            let mut popup = named(
                "completion-overlay",
                if popover.request.trigger == '@' {
                    self.file_menu(popover, &ink)
                } else {
                    super::menu::slash_menu(popover, &self.profiles, max_rows, &ink, &look)
                },
            );
            flat(&mut popup);
            let mut popup = popup.layout(
                Rect::new(12., 0., super::menu::slash_menu_width(width), 0.),
                ctx,
                &self.metrics,
            )?;
            let top = (bottom - popup.rect.h).max(44.);
            translate(&mut popup, 0., top);
            for (i, choice) in popover.choices.options.iter().enumerate() {
                let key = format!("completion/{i}");
                if let Some(node) = find(&popup, &key) {
                    front.push((
                        node.rect,
                        Action::Complete(popover.request.clone(), choice.label.clone()),
                    ));
                    hover.push((node.rect, key));
                }
            }
            Some((MenuKind::Slash, popup, selected, false))
        } else if let Some(menu) = &self.profile_menu {
            let current = state
                .transcript
                .profile
                .as_ref()
                .map(|(p, m)| (p.as_str(), m.as_str()));
            let selected = menu.selected_key(&self.profiles);
            let keys: Vec<String> = menu
                .rows(&self.profiles)
                .iter()
                .filter(|p| !super::menu::off(p))
                .map(|p| format!("pmenu/row/{}", p.name))
                .collect();
            let look = look(
                &mut self.anim,
                &keys,
                selected.as_deref(),
                hovered.as_deref(),
            );
            let profile_popup = if let Some(error) = state
                .profile_error
                .as_ref()
                .filter(|_| self.profiles.is_empty())
            {
                label(error)
            } else {
                menu.element(&self.profiles, current, &ink, &look)
            };
            let mut children = vec![profile_popup.into()];
            if !self.profiles.is_empty()
                && let Some(error) = &state.profile_error
            {
                children.push(label(error).into());
            }
            let content = column(children, None);
            let popup_width = super::menu::profile_menu_width(width);
            let content_h = content
                .layout(Rect::new(0., 0., popup_width, 0.), ctx, &self.metrics)?
                .rect
                .h;
            let viewport_h = content_h.min((size[1] - 60.).max(80.));
            self.config_scroll = self.config_scroll.min((content_h - viewport_h).max(0.));
            let mut popup = c::Element::ScrollView(c::ScrollView {
                viewport: [popup_width, viewport_h],
                offset: [0., self.config_scroll],
                content_width: Some(popup_width),
                content: Box::new(content),
            });
            flat(&mut popup);
            let popup = popup.layout(
                Rect::new(
                    10.,
                    self.frame
                        .layout
                        .as_ref()
                        .and_then(|l| find(l, "profile"))
                        .map_or(40., |n| n.rect.bottom() + 6.),
                    super::menu::profile_menu_width(width),
                    0.,
                ),
                ctx,
                &self.metrics,
            )?;
            self.config_menu_rect = Some(popup.rect);
            front.push((popup.rect, Action::Menu(true)));
            if let Some(node) = find(&popup, "pmenu/new") {
                let rect = intersection(node.rect, popup.rect);
                if rect.w > 0. && rect.h > 0. {
                    front.push((rect, Action::NewProfile));
                }
            }
            for p in menu.rows(&self.profiles) {
                let key = format!("pmenu/row/{}", p.name);
                if let Some(node) = find(&popup, &key) {
                    let rect = intersection(node.rect, popup.rect);
                    if rect.w > 0. && rect.h > 0. {
                        front.push((rect, Action::SwitchProfile(p.name.clone())));
                        hover.push((rect, key));
                    }
                }
            }
            Some((MenuKind::Profiles, popup, selected, true))
        } else if let Some(category) = &self.config_open
            && let Some(option) = state
                .transcript
                .agent_config
                .as_ref()
                .and_then(|c| c.options.iter().find(|o| &o.category == category))
        {
            let prefix = format!("acp/{category}");
            let keys: Vec<_> = (0..option.choices.len())
                .map(|i| format!("{prefix}/{i}"))
                .collect();
            let selected = keys.get(self.config_selected).cloned();
            let look = look(
                &mut self.anim,
                &keys,
                selected.as_deref(),
                hovered.as_deref(),
            );
            let current = option
                .choices
                .iter()
                .position(|c| c.value == option.current)
                .unwrap_or(usize::MAX);
            let names = option
                .choices
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>();
            let content = super::menu::select_menu(&prefix, &names, current, &look, &ink);
            let anchor = self
                .frame
                .layout
                .as_ref()
                .and_then(|l| find(l, &format!("config/{category}")))
                .map(|n| n.rect)
                .unwrap_or(Rect::new(10., 0., 200., 40.));
            let popup_width = (width - 20.).clamp(180., 280.);
            let y = anchor.bottom() + 6.;
            let content_h = content
                .layout(Rect::new(0., 0., popup_width, 0.), ctx, &self.metrics)?
                .rect
                .h;
            let viewport_h = content_h.min((size[1] - y - 8.).clamp(32., 280.));
            self.config_scroll = self.config_scroll.min((content_h - viewport_h).max(0.));
            let popup = c::Element::ScrollView(c::ScrollView {
                viewport: [popup_width, viewport_h],
                offset: [0., self.config_scroll],
                content_width: Some(popup_width),
                content: Box::new(content),
            })
            .layout(
                Rect::new(
                    anchor.x.min(width - popup_width - 10.).max(10.),
                    y,
                    popup_width,
                    0.,
                ),
                ctx,
                &self.metrics,
            )?;
            self.config_menu_rect = Some(popup.rect);
            front.push((popup.rect, Action::Menu(true)));
            for (index, key) in keys.iter().enumerate() {
                if let Some(node) = find(&popup, key) {
                    let rect = intersection(node.rect, popup.rect);
                    if rect.w > 0. && rect.h > 0. {
                        front.push((
                            rect,
                            config_action(state, category, index).unwrap_or(Action::Menu(false)),
                        ));
                        hover.push((rect, key.clone()));
                    }
                }
            }
            Some((MenuKind::Config, popup, selected, true))
        } else {
            None
        };
        // hit-tested front to back: the last row wins first, as inserted at 0
        front.reverse();
        self.frame.actions.splice(0..0, front);
        if open.as_ref().is_some_and(|(.., outside)| *outside) {
            // a click anywhere else closes the switcher
            self.frame
                .actions
                .push((Rect::new(0., 0., width, size[1]), Action::Menu(false)));
        }
        // the menu zones sit in front of the rows' (message rows and their source toggles)
        hover.append(&mut self.frame.hover);
        self.frame.hover = hover;
        let shown = open.as_ref().map(|(kind, ..)| *kind);
        if shown.is_none()
            && self.menus.hover.as_deref().is_some_and(|k| {
                k.starts_with("completion/") || k.starts_with("pmenu/") || k.starts_with("acp/")
            })
        {
            // no menu to hover: a reopened menu starts clean
            self.menus.hover = None;
        }
        // The card that closed (or gave way to the other kind) fades out from
        // where it was: 120ms `std`, no movement. Settled clocks drop it at once.
        if let Some(last) = self.menus.last.take()
            && shown != Some(last.kind)
        {
            self.menus.ghost = Some(last);
        }
        if let Some(ghost) = &self.menus.ghost {
            let opacity = self.anim.enter(
                &format!("{}/out", ghost.kind.id()),
                ghost.opacity,
                0.,
                MENU_CLOSE_MS,
                Ease::Std,
            );
            if opacity > 0. {
                let motion = MenuMotion {
                    opacity,
                    dy: 0.,
                    scale: 1.,
                    sel: ghost.sel,
                };
                paint_overlay(
                    &mut self.frame.over,
                    &ghost.layout,
                    ctx,
                    &ink,
                    ghost.kind,
                    &motion,
                )?;
            } else {
                self.menus.ghost = None;
            }
        }
        if let Some((kind, popup, selected, _)) = open {
            // open: 170ms `out`, a 6px drop, scale .98 → 1 and a fade, one tween
            let p = self.anim.enter(
                &format!("{}/in", kind.id()),
                0.,
                1.,
                MENU_OPEN_MS,
                Ease::Out,
            );
            // the selection highlight slides between rows over 120ms `std`;
            // on first sight it sits on its row
            let sel = selected.and_then(|k| find(&popup, &k)).map(|n| {
                let r = n.rect;
                let id = kind.id();
                let y = self
                    .anim
                    .tween(&format!("{id}/sel/y"), r.y, MENU_SELECT_MS, Ease::Std);
                let h = self
                    .anim
                    .tween(&format!("{id}/sel/h"), r.h, MENU_SELECT_MS, Ease::Std);
                Rect::new(r.x, y, r.w, h)
            });
            let motion = MenuMotion {
                opacity: p,
                dy: -6. * (1. - p),
                scale: 0.98 + 0.02 * p,
                sel,
            };
            paint_overlay(&mut self.frame.over, &popup, ctx, &ink, kind, &motion)?;
            self.menus.last = Some(Card {
                kind,
                layout: popup,
                sel,
                opacity: p,
            });
        }
        self.menus.dirty = false;
        Ok(())
    }
    /// The zoom pill (harness-spec.md Standalone window): `110%` centred 12 px under the
    /// feed's top, dropping in 4 px over 170 ms `out`, held 900 ms, then fading over 150 ms
    /// `std`. Settled and reduced clocks show it whole and drop it at the hold's end; a
    /// timed wake ends the hold. A new press while it shows resets the hold without a
    /// second drop.
    fn paint_zoom_pill(
        &mut self,
        ctx: ui::Context,
        ink: &super::face::Ink,
        width: f32,
    ) -> Result<(), String> {
        let Some((t0, n)) = self.zoom_shown else {
            return Ok(());
        };
        let since = self.anim.now() - t0;
        if since >= PILL_HOLD_MS + PILL_OUT_MS {
            self.zoom_shown = None;
            return Ok(());
        }
        let p_in = self
            .anim
            .enter("zoom/pill/in", 0., 1., PILL_IN_MS, Ease::Out);
        let p_out = if since >= PILL_HOLD_MS {
            self.anim.enter(
                &format!("zoom/pill/out/{n}"),
                1.,
                0.,
                PILL_OUT_MS,
                Ease::Std,
            )
        } else {
            self.anim.wake_in(PILL_HOLD_MS - since);
            1.
        };
        // faded out (or an end-state clock past the hold): gone
        if p_out <= 0. {
            return Ok(());
        }
        // painted at every opacity while it lives, so the frame that starts the drop carries
        // the pill (transparent) and a reader sees the motion from its first frame
        let opacity = (p_in * p_out).clamp(0., 1.);
        let feed = self
            .frame
            .layout
            .as_ref()
            .and_then(|l| find(l, "transcript"))
            .map_or(Rect::new(0., 0., width, 0.), |n| n.rect);
        let mut pill = named("zoom/pill", super::face::zoom_pill(&self.zoom.label(), ink));
        flat(&mut pill);
        // shrink-wrap the pill to its label: measure the text, then fix the box's width
        fn ink_width(n: &c::Layout) -> Option<f32> {
            match &n.kind {
                c::LayoutKind::Text { runs, .. } if !runs.is_empty() => Some(
                    runs.iter().map(|r| r.rect.right()).fold(0., f32::max)
                        - runs.iter().map(|r| r.rect.x).fold(f32::MAX, f32::min),
                ),
                _ => n.children.iter().find_map(ink_width),
            }
        }
        // (a root layout is as wide as its bounds, so the second pass is given the label's
        // width plus the pill's padding and border)
        let probe = pill.layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?;
        let pill_w = ink_width(&probe).map_or(width, |w| (w + 26.).ceil().min(width));
        let mut laid = pill.layout(Rect::new(0., 0., pill_w, 0.), ctx, &self.metrics)?;
        let x = (feed.x + (feed.w - laid.rect.w) / 2.).round();
        let y = feed.y + 12. - 4. * (1. - p_in);
        translate(&mut laid, x, y);
        let mut prims = DrawList::default();
        let mut rims = vec![];
        for paint in c::paint(&laid, ctx) {
            append_paint(&mut prims, &mut rims, paint)?;
        }
        if opacity < 1. {
            self.frame.over.push(bbt_ui_model::draw::Prim::Group {
                primitives: prims.primitives,
                opacity,
                saturation: 1.,
                mask: None,
            });
        } else {
            self.frame.over.primitives.extend(prims.primitives);
        }
        Ok(())
    }
    /// The pointer moved to `at`: track the menu row under it. True when the
    /// hovered row changed, so the host repaints.
    pub fn pointer(&mut self, at: [f32; 2]) -> bool {
        let key = self
            .frame
            .hover
            .iter()
            .find(|(r, _)| at[0] >= r.x && at[0] < r.right() && at[1] >= r.y && at[1] < r.bottom())
            .map(|(_, k)| k.clone());
        if key == self.menus.hover {
            return false;
        }
        self.menus.hover = key;
        self.menus.dirty = true;
        true
    }
    /// The row the pointer rests on, when it is over an open menu.
    pub fn hovered(&self) -> Option<&str> {
        self.menus.hover.as_deref()
    }
    /// Overlay actions win before any link below them.
    pub fn link_at(&self, at: [f32; 2]) -> Option<String> {
        if self.frame.actions.iter().any(|(r, _)| contains(*r, at)) {
            return None;
        }
        match self.frame.links.hit(at) {
            Some(ui::modules::document_modal::controller::Action::Open(target)) => Some(target),
            _ => None,
        }
    }
    fn profile_names(&self) -> Vec<String> {
        self.profiles.iter().map(|p| p.name.clone()).collect()
    }
    pub fn edit(&mut self, command: ui::text_edit::Command) {
        self.inputs.changed();
        self.composer.handle(composer::Command::Edit(command));
        self.inputs.observed_text = self.composer.view.text.clone();
        self.completion = self.composer.popover(&self.profile_names());
        self.refresh_files();
        self.last_edit = self.anim.now();
        self.invalidate();
    }
    pub fn completion_open(&self) -> bool {
        self.completion.is_some()
    }
    pub fn profile_menu_open(&self) -> bool {
        self.profile_menu.is_some() || self.config_open.is_some()
    }
    /// Open (or close) the header profile switcher, as the chip click does.
    pub fn toggle_profile_menu(&mut self) {
        self.config_open = None;
        self.config_scroll = 0.;
        self.profile_menu = match self.profile_menu.take() {
            Some(_) => None,
            None => {
                self.completion = None;
                let mut menu = super::menu::ProfileMenu::default();
                // Start on the current profile, as the web marks it.
                let current = self.current_profile.clone();
                if let Some(i) = menu
                    .rows(&self.profiles)
                    .iter()
                    .position(|p| Some(&p.name) == current.as_ref())
                {
                    menu.selected = i;
                }
                Some(menu)
            }
        };
        self.invalidate();
    }
    /// Keys for the open profile switcher; false when it is closed.
    pub fn profile_menu_key(&mut self, key: MenuKey, bridge: &Bridge) -> bool {
        if let Some(category) = self.config_open.clone() {
            let option = bridge
                .state
                .transcript
                .agent_config
                .as_ref()
                .and_then(|c| c.options.iter().find(|o| o.category == category));
            let Some(option) = option.filter(|o| !o.choices.is_empty()) else {
                return false;
            };
            let len = option.choices.len();
            self.config_selected = self.config_selected.min(len - 1);
            match key {
                MenuKey::Up => self.config_selected = (self.config_selected + len - 1) % len,
                MenuKey::Down => self.config_selected = (self.config_selected + 1) % len,
                MenuKey::Enter => {
                    let action = config_action(&bridge.state, &category, self.config_selected)
                        .unwrap_or(Action::Menu(false));
                    self.activate(action, bridge);
                    return true;
                }
                MenuKey::Text(text) => {
                    if let Some(index) = option
                        .choices
                        .iter()
                        .position(|c| c.name.to_lowercase().starts_with(&text.to_lowercase()))
                    {
                        self.config_selected = index;
                    }
                }
                MenuKey::Backspace => {}
            }
            self.config_scroll = self.config_selected.saturating_sub(4) as f32 * 30.;
            self.invalidate();
            return true;
        }
        let profiles = self.profiles.clone();
        let Some(menu) = &mut self.profile_menu else {
            return false;
        };
        let len = menu.rows(&profiles).len();
        match key {
            MenuKey::Up => menu.move_by(-1, len),
            MenuKey::Down => menu.move_by(1, len),
            MenuKey::Backspace => {
                menu.query.pop();
                menu.selected = 0;
            }
            MenuKey::Text(text) => {
                menu.query.push_str(&text);
                menu.selected = 0;
            }
            MenuKey::Enter => {
                if let Some(p) = menu.chosen(&profiles) {
                    let name = p.name.clone();
                    self.activate(Action::SwitchProfile(name), bridge);
                    return true;
                }
            }
        }
        self.invalidate();
        true
    }
    /// Enter on the open "/" menu: complete the chosen command, then run it.
    pub fn run_completion(&mut self, bridge: &Bridge) -> bool {
        if let Some(p) = &self.completion
            && p.request.trigger == '@'
        {
            let request = p.request.clone();
            if let Some(label) = p
                .choices
                .selected()
                .and_then(|i| p.choices.options.get(i))
                .map(|c| c.label.clone())
            {
                self.choose_file(request, label, bridge);
            }
            return true;
        }
        let Some(popover) = &mut self.completion else {
            return false;
        };
        if let Some(edit) = popover.handle(ui::modules::approval::Key::Enter) {
            self.completion = None;
            if self.composer.handle(composer::Command::Edit(edit)) == composer::Outcome::Changed {
                self.activate(Action::Send, bridge);
            }
        }
        self.invalidate();
        true
    }
    pub fn complete(&mut self, key: ui::modules::approval::Key) -> bool {
        if self.completion.is_none() {
            self.completion = self.composer.popover(&self.profile_names());
        }
        let Some(popover) = &mut self.completion else {
            return false;
        };
        if popover.request.trigger == '@' && matches!(key, ui::modules::approval::Key::Enter) {
            return false;
        }
        if let Some(edit) = popover.handle(key) {
            self.composer.handle(composer::Command::Edit(edit));
            self.completion = None;
        }
        self.invalidate();
        true
    }
    fn sync_approval(&mut self, bridge: &Bridge) -> bool {
        let row = bridge.state.transcript.items.values().find(|r| {
            r.kind == ItemKind::Approval
                && r.status.is_none()
                && matches!(r.body, Item::Approval { decision: None, .. })
        });
        let Some(row) = row else {
            self.approval = None;
            return false;
        };
        if self.approval.as_ref().is_none_or(|(id, _)| *id != row.id)
            && let Item::Approval { options, .. } = &row.body
        {
            self.approval = Some((row.id, approval_choices(row.id, options)));
        }
        self.approval.is_some()
    }
    pub fn approval_key(&mut self, key: ui::modules::approval::Key, bridge: &Bridge) -> bool {
        if !self.sync_approval(bridge) {
            return false;
        }
        let (id, prompt) = self.approval.as_mut().unwrap();
        let id = *id;
        let selected = prompt.handle(key);
        bridge.toggle(
            id,
            super::rich::Toggle::Approval(prompt.selected().unwrap_or(0)),
        );
        if let Some(selected) = selected {
            let decision = match selected.as_str() {
                "Allow" => ApprovalDecision::Allow,
                "Always" => ApprovalDecision::AllowAlways,
                _ => ApprovalDecision::Deny { reason: None },
            };
            self.activate(Action::Approval(id, decision), bridge);
        }
        self.invalidate();
        true
    }
    pub fn cancel_prompt(&mut self, bridge: &Bridge) -> bool {
        if self.sync_approval(bridge) {
            let id = self.approval.as_ref().unwrap().0;
            self.completion = None;
            self.activate(
                Action::Approval(
                    id,
                    ApprovalDecision::Deny {
                        reason: Some("Cancelled".into()),
                    },
                ),
                bridge,
            );
            return true;
        }
        if self.completion.take().is_some()
            || self.profile_menu.take().is_some()
            || self.config_open.take().is_some()
        {
            self.invalidate();
            return true;
        }
        false
    }
    /// `/profile NAME` and the switcher: continue this conversation on NAME.
    fn switch_to(&mut self, name: &str, bridge: &Bridge) {
        self.profile_menu = None;
        self.config_open = None;
        self.notice = Some(
            if !matches!(
                bridge.state.gateway,
                Some(super::bridge::ViewGateway::Unix(_))
            ) && !bridge.state.profiles.iter().any(|p| p.name == name)
            {
                format!("No profile named {name} · /profiles lists them")
            } else if self.current_profile.as_deref() == Some(name) {
                format!("Already on {name}")
            } else {
                match bridge.switch_profile(name.to_owned()) {
                    Ok(()) => {
                        format!("Switching to {name}: the conversation continues in a new session")
                    }
                    Err(why) => format!("Could not switch: {why}"),
                }
            },
        );
        self.follow_latest();
        self.invalidate();
    }
    /// Resume or an explicit jump starts at the newest row, with no stale glide.
    pub fn follow_latest(&mut self) {
        self.scroll.follow_latest();
        self.tail_pending = true;
        self.anchor = None;
        self.anim.forget_spring("scroll");
        self.invalidate();
    }
    pub fn activate(&mut self, action: Action, bridge: &Bridge) {
        match action {
            Action::PanelScroll(key, delta) => {
                self.shift_panel(&key, delta);
                return;
            }
            Action::Jump => self.follow_latest(),
            Action::AgentConfig(envelope) => {
                self.config_open = None;
                self.notice = match bridge.send_envelope(envelope) {
                    Ok(_) => None,
                    Err(_) => Some("Agent change not admitted · refresh the controls".into()),
                };
                self.invalidate();
                return;
            }
            Action::ConfigMenu(category) => {
                let closing = self.config_open.as_deref() == Some(&category);
                self.config_open = if closing {
                    None
                } else {
                    Some(category.clone())
                };
                self.profile_menu = None;
                self.completion = None;
                self.config_scroll = 0.;
                self.config_selected = bridge
                    .state
                    .transcript
                    .agent_config
                    .as_ref()
                    .and_then(|c| c.options.iter().find(|o| o.category == category))
                    .and_then(|o| o.choices.iter().position(|c| c.value == o.current))
                    .unwrap_or(0);
            }
            Action::Profiles => return self.toggle_profile_menu(),
            Action::SwitchProfile(name) => {
                self.setup_open = false;
                return self.switch_to(&name, bridge);
            }
            Action::Menu(true) => return,
            Action::Menu(false) => {
                self.profile_menu = None;
                self.config_open = None;
            }
            Action::Setup(open) => {
                self.setup_open = open;
                self.setup_dismissed = !open;
            }
            Action::SetupDetails(agent) => {
                if !self.setup_details.remove(&agent) {
                    self.setup_details.insert(agent);
                }
            }
            Action::InstallAdapter(recipe) => {
                self.notice = bridge.install_adapter(recipe).err().map(str::to_owned);
            }
            Action::CancelInstall(agent) => bridge.cancel_install(agent),
            Action::NewProfile => {
                self.profile_menu = None;
                self.setup_open = true;
            }
            Action::Toggle(item, toggle) => bridge.toggle(item, toggle),
            Action::Chip(which) => {
                // the empty session's three ways in: "/" (opens the menu), "@", or focus
                self.composer
                    .handle(composer::Command::Focus(composer::Field::Body));
                match which {
                    0 => return self.edit(ui::text_edit::Command::Insert("/".into())),
                    1 => return self.edit(ui::text_edit::Command::Insert("@".into())),
                    _ => {}
                }
            }
            Action::RemoveAttachment(index) => {
                self.remove_attachment(index);
                return;
            }
            Action::Complete(request, text) => {
                if request.trigger == '@' {
                    self.choose_file(request, text, bridge);
                    return;
                }
                let outcome = self.composer.handle(composer::Command::Edit(
                    ui::text_edit::Command::Complete { request, text },
                ));
                if outcome == composer::Outcome::Changed {
                    // a click on a "/" row runs it, as Enter does
                    self.completion = None;
                    return self.activate(Action::Send, bridge);
                } else {
                    self.completion = self.composer.popover(&self.profile_names());
                }
            }
            Action::Send => {
                if let composer::Outcome::Submit(sent) =
                    self.composer.handle(composer::Command::Send)
                {
                    // Attachments are user content; a draft containing them never
                    // becomes a launcher/profile/approval command.
                    let text = if sent.attach.is_empty() {
                        sent.text.trim()
                    } else {
                        ""
                    };
                    // `/profiles` opens the switcher; `/profile NAME` continues this session on NAME
                    if text == "/profiles" || text == "/profile" {
                        self.composer.stored(&sent);
                        if bridge.state.profiles.is_empty() {
                            self.notice =
                                Some("No profiles in ~/.bboxy450-harness/profiles".into());
                        } else if self.profile_menu.is_none() {
                            self.toggle_profile_menu();
                        }
                        self.invalidate();
                        return;
                    }
                    if let Some(name) = text.strip_prefix("/profile ").map(str::trim) {
                        self.composer.stored(&sent);
                        let name = name.to_owned();
                        self.switch_to(&name, bridge);
                        return;
                    }
                    if text == "/stop" {
                        self.composer.stored(&sent);
                        let birth = self.birth.unwrap_or(bridge.state.transcript.epoch);
                        self.activate(Action::StopAt(birth), bridge);
                        self.follow_latest();
                        return;
                    }
                    let blocks = match self.attachment_blocks(&sent) {
                        Ok(blocks) => blocks,
                        Err(error) => {
                            self.notice = Some(error);
                            self.invalidate();
                            return;
                        }
                    };
                    let op = match text {
                        "/interrupt" => Op::Interrupt { turn: None },
                        _ => Op::UserTurn {
                            blocks,
                            mode: TurnMode::Queue,
                        },
                    };
                    if self.inputs.attachments.iter().any(|a| a.custody.is_some())
                        && bridge.state.gateway.is_some()
                        && !bridge.image_intake_available()
                    {
                        self.notice=Some("This actor has not advertised image intake support. Your image draft and custody are retained; no message was sent.".into());
                        self.invalidate();
                        return;
                    }
                    let op_id = OpId::new();
                    let Some(session) = bridge.state.transcript.session else {
                        self.notice = Some("Session unavailable; draft retained".into());
                        return;
                    };
                    let custody = self
                        .inputs
                        .attachments
                        .iter()
                        .filter_map(|a| a.custody.clone())
                        .collect();
                    match bridge.send_with_custody(
                        OpEnvelope {
                            v: PROTOCOL,
                            op_id,
                            session,
                            op,
                        },
                        custody,
                    ) {
                        Ok(_) => {
                            self.composer.stored(&sent);
                            if !sent.attach.is_empty() {
                                self.inputs.sent.push(input::Sent {
                                    owner: self.inputs.owner.clone(),
                                    op: op_id,
                                    pending_at_send: bridge.state.pending_prompts.clone(),
                                    attachments: std::mem::take(&mut self.inputs.attachments),
                                    draft: sent,
                                });
                            }
                            self.inputs.changed();
                            self.notice = None;
                            self.follow_latest();
                        }
                        Err(error) => {
                            self.notice = Some(format!(
                                "Message not admitted: {}. Your draft is retained.",
                                super::bridge::refusal_text(&error)
                            ))
                        }
                    }
                }
            }
            action => {
                let op = match action {
                    Action::Interrupt => Op::Interrupt { turn: None },
                    Action::Stop => Op::Close,
                    Action::StopAt(birth) => {
                        if bridge.state.transcript.epoch != birth {
                            self.notice = Some("Session replaced".into());
                            self.invalidate();
                            return;
                        }
                        Op::Close
                    }
                    Action::Retry(turn) => {
                        let model = &bridge.state.transcript;
                        let prompt = model.order.iter().find_map(|id| {
                            let row = &model.items[id];
                            match &row.body {
                                Item::UserMessage { blocks } if row.turn == Some(turn) => {
                                    Some(blocks.clone())
                                }
                                _ => None,
                            }
                        });
                        let Some(blocks) = prompt else {
                            self.notice = Some("Nothing to retry: the turn has no prompt".into());
                            self.invalidate();
                            return;
                        };
                        self.follow_latest();
                        Op::UserTurn {
                            blocks,
                            mode: TurnMode::Queue,
                        }
                    }
                    Action::Approval(approval, decision) => {
                        if !bridge
                            .state
                            .transcript
                            .items
                            .get(&approval)
                            .is_some_and(|row| {
                                row.status.is_none()
                                    && matches!(row.body, Item::Approval { decision: None, .. })
                            })
                        {
                            self.notice = Some("Approval expired".into());
                            self.invalidate();
                            return;
                        }
                        Op::ResolveApproval { approval, decision }
                    }
                    _ => unreachable!(),
                };
                self.notice = bridge
                    .send(op)
                    .err()
                    .map(|e| format!("Action not admitted: {}", super::bridge::refusal_text(&e)));
            }
        }
        self.invalidate();
    }
}
fn config_action(state: &FrameState, category: &str, index: usize) -> Option<Action> {
    let config = state.transcript.agent_config.as_ref()?;
    if state.replay || config.pending.is_some() {
        return None;
    }
    let option = config.options.iter().find(|o| o.category == category)?;
    let choice = option.choices.get(index)?;
    if choice.value == option.current {
        return None;
    }
    Some(Action::AgentConfig(OpEnvelope {
        v: PROTOCOL,
        session: state.transcript.session?,
        op_id: OpId::new(),
        op: Op::SetAgentConfig {
            epoch: state.transcript.epoch,
            instance: config.instance,
            revision: config.revision,
            config_id: option.id.clone(),
            value: Some(choice.value.clone()),
        },
    }))
}

pub fn intersection(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    Rect::new(
        x,
        y,
        (a.right().min(b.right()) - x).max(0.),
        (a.bottom().min(b.bottom()) - y).max(0.),
    )
}
/// The per-row look of a menu this frame: each row's selected weight (120ms
/// `std`) and hover weight (150ms `std`), read from the clock for every row
/// so a change tweens from where the row is. First sight is the end state.
fn look(
    anim: &mut Clock,
    keys: &[String],
    selected: Option<&str>,
    hovered: Option<&str>,
) -> super::menu::Look {
    let mut look = super::menu::Look::default();
    for key in keys {
        let target = |on: bool| if on { 1. } else { 0. };
        let sel = anim.tween(
            &format!("{key}/sel"),
            target(selected == Some(key.as_str())),
            MENU_SELECT_MS,
            Ease::Std,
        );
        let hover = anim.tween(
            &format!("{key}/hover"),
            target(hovered == Some(key.as_str())),
            MENU_HOVER_MS,
            Ease::Std,
        );
        if sel > 0. {
            look.sel.push((key.clone(), sel));
        }
        if hover > 0. {
            look.hover.push((key.clone(), hover));
        }
    }
    look
}
/// A floating menu over the frame. Spec flat fallback: no drop shadow, a 1px
/// bg line under the bottom edge. The card chrome paints first, then the
/// sliding selection highlight, then the rows over it; the whole card then
/// takes the frame's drop, scale (about its centre, as CSS) and opacity.
fn paint_overlay(
    list: &mut DrawList,
    popup: &c::Layout,
    ctx: ui::Context,
    ink: &super::face::Ink,
    kind: MenuKind,
    motion: &MenuMotion,
) -> Result<(), String> {
    use super::face::alpha;
    let mut chrome = popup.clone();
    if let Some(rows) = find_mut(&mut chrome, super::menu::LIST) {
        rows.children.clear();
    }
    let mut prims = DrawList::default();
    let mut rims = vec![];
    for paint in c::paint(&chrome, ctx) {
        append_paint(&mut prims, &mut rims, paint)?;
    }
    prims.clip(popup.rect, [7.; 4]);
    if let Some(r) = motion.sel {
        match kind {
            // seat-14, radius 8, and the 2px seat bar inset 8px top and bottom
            MenuKind::Slash => {
                prims.rect(r, 8., Color(alpha(ink.seat, 0.14)));
                prims.rect(
                    Rect::new(r.x - 3., r.y + 8., 2., (r.h - 16.).max(1.)),
                    1.,
                    Color(ink.seat),
                );
            }
            MenuKind::Profiles | MenuKind::Config => {
                prims.rect(r, 7., Color(alpha(ink.seat, 0.12)))
            }
        }
    }
    if let Some(rows) = find(popup, super::menu::LIST) {
        for paint in c::paint(rows, ctx) {
            append_paint(&mut prims, &mut rims, paint)?;
        }
    }
    prims.unclip();
    let r = popup.rect;
    prims.rect(
        Rect::new(r.x + 13., r.bottom(), (r.w - 26.).max(0.), 1.),
        0.,
        Color(ink.bg),
    );
    if motion.scale != 1. || motion.dy != 0. {
        transform(
            &mut prims.primitives,
            motion.scale,
            [r.x + r.w / 2., r.y + r.h / 2.],
            motion.dy,
        );
    }
    if motion.opacity < 1. {
        list.push(bbt_ui_model::draw::Prim::Group {
            primitives: prims.primitives,
            opacity: motion.opacity.max(0.),
            saturation: 1.,
            mask: None,
        });
    } else {
        list.primitives.extend(prims.primitives);
    }
    Ok(())
}
/// Scale primitives by `s` about `origin`, then shift them down by `dy`. Text
/// keeps its glyph size and width: at the .98 a menu opens from, a 2% larger
/// run is invisible, while a 2% narrower max width would ellipsize it.
fn transform(prims: &mut [bbt_ui_model::draw::Prim], s: f32, origin: [f32; 2], dy: f32) {
    use bbt_ui_model::draw::Prim;
    let pt = |p: [f32; 2]| {
        [
            origin[0] + (p[0] - origin[0]) * s,
            origin[1] + (p[1] - origin[1]) * s + dy,
        ]
    };
    let rect = |r: Rect| {
        let [x, y] = pt([r.x, r.y]);
        Rect::new(x, y, r.w * s, r.h * s)
    };
    for prim in prims {
        match prim {
            Prim::Rect { rect: r, .. }
            | Prim::Gloss { rect: r, .. }
            | Prim::Icon { rect: r, .. }
            | Prim::Image { rect: r, .. }
            | Prim::Clip { rect: r, .. } => *r = rect(*r),
            Prim::Hairline { from, to, .. } => {
                *from = pt(*from);
                *to = pt(*to);
            }
            Prim::Text {
                origin: o,
                baseline,
                ..
            }
            | Prim::TextShadow {
                origin: o,
                baseline,
                ..
            } => {
                *o = pt(*o);
                if let Some(b) = baseline {
                    *b = origin[1] + (*b - origin[1]) * s + dy;
                }
            }
            Prim::Path {
                origin: o,
                scale: ps,
                ..
            }
            | Prim::PathShadow {
                origin: o,
                scale: ps,
                ..
            } => {
                *o = pt(*o);
                *ps *= s;
            }
            Prim::Group { primitives, .. } => transform(primitives, s, origin, dy),
            Prim::Unclip => {}
        }
    }
}
/// Flatten paint ops into primitives; the alive rims (a material pass) are collected in
/// `rims` for the renderer to draw after the flat list, as the term's host does.
fn append_paint(
    list: &mut DrawList,
    rims: &mut Vec<m::Surface>,
    paint: c::Paint,
) -> Result<(), String> {
    match paint {
        c::Paint::Draw(draw) | c::Paint::Underlay(draw) => list.primitives.extend(draw.primitives),
        c::Paint::Group {
            ops,
            opacity,
            saturation,
            mask,
        } => {
            let mut inner = DrawList::default();
            for op in ops {
                append_paint(&mut inner, rims, op)?;
            }
            list.push(bbt_ui_model::draw::Prim::Group {
                primitives: inner.primitives,
                opacity,
                saturation,
                mask,
            });
        }
        c::Paint::Material {
            kind: ui::Material::Rim,
            rect,
            radii,
            color,
            rim,
            clips,
            ..
        } => rims.push(m::Surface {
            rect,
            radii,
            material: rim_material(color, rim),
            clips: clips
                .into_iter()
                .map(|c| m::Clip {
                    rect: c.rect,
                    radii: c.radii,
                })
                .collect(),
        }),
        c::Paint::Material { .. } => {
            return Err("Unsupported transcript material".into());
        }
    }
    Ok(())
}
/// The renderer's rim from a Style's `Rim` state.
// Adapted from bboxy450-term crates/bboxy450-term/src/gpu.rs rim_material.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
fn rim_material(color: Color, rim: Option<ui::Rim>) -> m::Material {
    let tokens = bbt_ui_model::TOKENS;
    let s = rim.unwrap_or_default();
    let size = s.size.unwrap_or(tokens.glow.rim as f32);
    m::Material::Rim(m::Rim {
        color: s
            .tint
            .map(Color)
            .unwrap_or(color)
            .alpha(s.inferred_multiplier),
        waiting: Color::hex(tokens.color.state.waiting),
        errored: Color::hex(tokens.color.state.errored),
        activity: match s.activity {
            ui::GlowActivity::Idle => m::Activity::Idle,
            ui::GlowActivity::Active => m::Activity::Active,
            ui::GlowActivity::Waiting => m::Activity::Waiting,
            ui::GlowActivity::Errored => m::Activity::Errored,
        },
        focus: if s.focused { 1. } else { 0. },
        depth: s.depth.unwrap_or(tokens.glow.depth as f32),
        size,
        phase_ms: (s.seed % 10_000) as f32,
        outer: s.outer.unwrap_or(f32::MAX).clamp(0., size / 2.),
        alpha: s.rim_alpha.unwrap_or(1.),
        swell: s.swell.clamp(0., 1.),
        attention: s.attention,
    })
}
fn approval(kind: ApprovalDecisionKind) -> (&'static str, ApprovalDecision) {
    match kind {
        ApprovalDecisionKind::Allow => ("Allow", ApprovalDecision::Allow),
        ApprovalDecisionKind::AllowAlways => ("Always", ApprovalDecision::AllowAlways),
        _ => ("Deny", ApprovalDecision::Deny { reason: None }),
    }
}
pub fn kind_label(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::UserMessage => "You",
        ItemKind::AgentMessage => "Agent",
        ItemKind::Reasoning => "Reasoning",
        ItemKind::ToolCall => "Tool call",
        ItemKind::FileChange => "File change",
        ItemKind::TurnDiff => "Turn diff",
        ItemKind::Terminal => "Terminal",
        ItemKind::Plan => "Plan",
        ItemKind::Approval => "Approval",
        ItemKind::Image => "Image",
        ItemKind::Compaction => "Compaction",
        ItemKind::ContextTrim => "Context trim",
        ItemKind::Usage => "Usage",
        ItemKind::Notice => "Notice",
        ItemKind::Subagent => "Subagent",
        ItemKind::Command => "Command",
        ItemKind::Hook => "Hook",
        ItemKind::SystemMessage => "System message",
        ItemKind::Native => "Native record",
        ItemKind::MemoryWrite => "Memory write",
        ItemKind::MemoryState => "Memory state",
        ItemKind::Trial => "Trial",
        ItemKind::Adoption => "Adoption",
        ItemKind::Recall => "Recall",
        ItemKind::Unknown(_) => "Unknown item",
    }
}
/// The header's phase, from the transcript as the web reference reads it (harness.js
/// phaseNow): closed, a tool waiting on its approval, the newest running item, else the
/// live phase.
fn phase_now(model: &super::model::Transcript) -> (String, super::face::Tone) {
    use super::face::Tone;
    if model.closed || matches!(model.phase, Phase::Closed) {
        return ("Session closed".into(), Tone::Idle);
    }
    let cwd = model
        .meta
        .as_ref()
        .map(|m| m.cwd.to_string_lossy().into_owned())
        .unwrap_or_default();
    let asking: std::collections::HashSet<ItemId> = model
        .items
        .values()
        .filter_map(|r| match &r.body {
            Item::Approval {
                subject,
                decision: None,
                ..
            } => Some(subject.tool_call),
            _ => None,
        })
        .collect();
    for id in model.order.iter().rev() {
        let r = &model.items[id];
        if r.status.is_some() {
            continue;
        }
        match &r.body {
            Item::Reasoning { .. } => return ("Thinking".into(), Tone::Live),
            Item::AgentMessage { .. } => return ("Writing".into(), Tone::Live),
            Item::ToolCall {
                name,
                tool_kind,
                input_raw,
                output,
                ..
            } => {
                let d = crate::frontends::face_core::describe(
                    name,
                    tool_kind.clone(),
                    input_raw,
                    output,
                    &cwd,
                );
                if asking.contains(id) {
                    return (format!("Waiting for you · {}", d.verb), Tone::Ask);
                }
                return (
                    format!("{} {}", d.verb, d.target).trim().to_owned(),
                    Tone::Live,
                );
            }
            _ => {}
        }
    }
    match model.phase {
        Phase::Idle | Phase::Unknown(_) => ("Ready".into(), Tone::Idle),
        ref p => (phase_text(p), Tone::Live),
    }
}
fn phase_text(phase: &Phase) -> String {
    match phase {
        Phase::Idle => "Ready".into(),
        Phase::Queued { turns } => format!("{turns} turns queued"),
        Phase::Thinking => "Thinking".into(),
        Phase::Streaming => "Writing".into(),
        Phase::Tool { .. } => "Running tool".into(),
        Phase::Approval { .. } => "Approval needed".into(),
        Phase::Retrying { attempt, wait_ms } => format!("Retry {attempt} in {wait_ms} ms"),
        Phase::Closed => "Session closed".into(),
        Phase::Unknown(raw) => format!("Unsupported status: {}", unknown_tag(raw)),
    }
}
fn arguments(raw: &str) -> String {
    fn readable(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(fields) => fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", readable(v)))
                .collect::<Vec<_>>()
                .join("; "),
            serde_json::Value::Array(values) => {
                values.iter().map(readable).collect::<Vec<_>>().join(", ")
            }
            serde_json::Value::Null => "none".into(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
        }
    }
    serde_json::from_str(raw)
        .map(|v| readable(&v))
        .unwrap_or_else(|_| {
            if raw.is_empty() {
                "Waiting for arguments".into()
            } else {
                format!("Arguments streaming ({} bytes)", raw.len())
            }
        })
}
/// Visible solved glyph text, used by headless semantic witnesses.
pub fn layout_text(layout: &c::Layout) -> String {
    fn collect(layout: &c::Layout, out: &mut Vec<String>) {
        if let c::LayoutKind::Text { runs, .. } = &layout.kind {
            let mut text = String::new();
            let mut previous: Option<(f32, f32)> = None;
            for run in runs {
                if previous.is_some_and(|(right, baseline)| {
                    (run.baseline - baseline).abs() > 0.5 || run.rect.x - right > 1.
                }) {
                    text.push(' ');
                }
                text.push_str(&run.text);
                previous = Some((run.rect.right(), run.baseline));
            }
            out.push(text);
        }
        for child in &layout.children {
            collect(child, out);
        }
    }
    let mut out = vec![];
    collect(layout, &mut out);
    out.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
// Adapted from bbt-ui components::translate_layout. Completed layouts are
// immutable; only their placement changes. ScrollView's fixed-height window
// elides offscreen slots before this walk, so only visible rows are restored.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
// A row whose slot is still growing (its height tween in flight) is clipped to the slot by
// a synthetic scroll node, so a card never paints over the row below it. A row in `enters`
// sits 6 px lower and fades with its entrance progress (block enters: 320 ms `out`).
// `runs` sees every text run of the row in paint order, elided or not, so a streaming
// fade can count glyphs from the row's start whatever the viewport shows.
fn restore_rows<'a>(
    node: &mut c::Layout,
    rows: &im::HashMap<ItemId, Arc<CachedRow>>,
    viewport: Rect,
    enters: &HashMap<ItemId, f32>,
    markdown_scroll: &HashMap<String, [f32; 2]>,
    runs: &mut dyn FnMut(ItemId) -> Box<dyn stream_motion::Runs + 'a>,
) {
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(id) = name
            .strip_prefix("cached/")
            .and_then(|s| s.parse().ok())
            .map(ItemId)
        && let Some(row) = rows.get(&id)
    {
        let slot = node.rect;
        let t = enters.get(&id).copied().unwrap_or(1.);
        let dx = slot.x - row.layout.rect.x;
        let dy = slot.y - row.layout.rect.y + 6. * (1. - t);
        let mut visitor = runs(id);
        if let Some(mut solved) = visible_layout(
            &row.layout,
            dx,
            dy,
            viewport,
            markdown_scroll,
            visitor.as_mut(),
        ) {
            if row.layout.rect.h > slot.h + 0.5 {
                solved = c::Layout {
                    rect: slot,
                    baseline: solved.baseline,
                    kind: c::LayoutKind::ScrollView {
                        content: [slot.w, row.layout.rect.h],
                        offset: [0., 0.],
                    },
                    children: vec![solved],
                };
            }
            if t < 1. {
                solved = c::Layout {
                    rect: solved.rect,
                    baseline: solved.baseline,
                    kind: c::LayoutKind::Filter {
                        saturation: 1.,
                        opacity: t.clamp(0., 1.),
                    },
                    children: vec![solved],
                };
            }
            *node = solved;
        }
        return;
    }
    for child in &mut node.children {
        restore_rows(child, rows, viewport, enters, markdown_scroll, runs);
    }
}
// Keep only glyphs/blocks intersecting the viewport. A giant paragraph remains
// fully scrollable without cloning/painting its entire cached solved layout.
// `runs` sees every text run of the row in paint order, elided or not, so a
// streaming fade can count glyphs from the row's start whatever the viewport shows.
fn visible_layout(
    node: &c::Layout,
    dx: f32,
    dy: f32,
    viewport: Rect,
    markdown_scroll: &HashMap<String, [f32; 2]>,
    runs: &mut dyn stream_motion::Runs,
) -> Option<c::Layout> {
    let rect = Rect::new(node.rect.x + dx, node.rect.y + dy, node.rect.w, node.rect.h);
    let visible = intersection(rect, viewport);
    if visible.w <= 0. || visible.h <= 0. {
        runs.skip(node);
        return None;
    }
    if let c::LayoutKind::Named(name) = &node.kind
        && scroll_panel(name)
        && let Some(scroll) = node.children.first()
        && let c::LayoutKind::ScrollView { content, offset } = scroll.kind
    {
        let saved = markdown_scroll.get(name).copied().unwrap_or([0., 0.]);
        let x = saved[0].clamp(0., (content[0] - rect.w).max(0.));
        let y = saved[1].clamp(0., (content[1] - rect.h).max(0.));
        return Some(c::Layout {
            rect,
            baseline: node.baseline,
            kind: node.kind.clone(),
            children: vec![c::Layout {
                rect,
                baseline: scroll.baseline,
                kind: c::LayoutKind::ScrollView {
                    content,
                    offset: [x, y],
                },
                children: scroll
                    .children
                    .iter()
                    .filter_map(|child| {
                        visible_layout(
                            child,
                            dx + offset[0] - x,
                            dy + offset[1] - y,
                            visible,
                            markdown_scroll,
                            runs,
                        )
                    })
                    .collect(),
            }],
        });
    }
    let kind = match &node.kind {
        c::LayoutKind::Text {
            runs: text_runs,
            selectable,
            ellipsis,
            shadow,
        } => c::LayoutKind::Text {
            runs: runs
                .visit(text_runs)
                .into_iter()
                .filter(|r| {
                    r.rect.bottom() + dy >= viewport.y && r.rect.y + dy <= viewport.bottom()
                })
                .map(|mut r| {
                    r.rect.x += dx;
                    r.rect.y += dy;
                    r.baseline += dy;
                    r
                })
                .collect(),
            selectable: *selectable,
            ellipsis: *ellipsis,
            shadow: *shadow,
        },
        other => other.clone(),
    };
    let name = match &node.kind {
        c::LayoutKind::Named(name) => Some(name.as_str()),
        _ => None,
    };
    if let Some(name) = name {
        runs.enter(name);
    }
    let children = node
        .children
        .iter()
        .filter_map(|child| visible_layout(child, dx, dy, viewport, markdown_scroll, runs))
        .collect();
    if let Some(name) = name {
        runs.leave(name);
    }
    Some(c::Layout {
        rect,
        baseline: node.baseline,
        kind,
        children,
    })
}
fn restore_pending(
    node: &mut c::Layout,
    rows: &HashMap<OpId, Arc<c::Layout>>,
    viewport: Rect,
    scroll: &HashMap<String, [f32; 2]>,
) {
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(id) = name
            .strip_prefix("pending/")
            .and_then(|s| s.parse().ok())
            .map(OpId)
        && let Some(row) = rows.get(&id)
    {
        if let Some(solved) = visible_layout(
            row,
            node.rect.x - row.rect.x,
            node.rect.y - row.rect.y,
            viewport,
            scroll,
            &mut stream_motion::Plain,
        ) {
            node.children = vec![solved];
        }
        return;
    }
    for child in &mut node.children {
        restore_pending(child, rows, viewport, scroll);
    }
}
pub fn translate(node: &mut c::Layout, dx: f32, dy: f32) {
    node.rect.x += dx;
    node.rect.y += dy;
    if let c::LayoutKind::Text { runs, .. } = &mut node.kind {
        for run in runs {
            run.rect.x += dx;
            run.rect.y += dy;
            run.baseline += dy;
        }
    }
    for child in &mut node.children {
        translate(child, dx, dy);
    }
}

/// Immutable worker-prepared transcript. The frame only positions visible rows.
pub struct PreparedRows {
    pub theme: Theme,
    pub theme_revision: u64,
    pub source: Arc<Transcript>,
    pub width: f32,
    /// The text zoom these rows were laid out at, percent (harness-spec.md Standalone window).
    pub zoom: u8,
    rows: im::HashMap<ItemId, Arc<CachedRow>>,
    /// The rows' solved heights summed: the content height once every height tween rests.
    pub height: f32,
    pub(crate) pending: HashMap<OpId, Arc<c::Layout>>,
    pub layout_builds: usize,
    pub parsed_markdown_bytes: usize,
    pub received_markdown_bytes: usize,
    pub markdown_block_layouts: usize,
    pub markdown_appends: usize,
    pub deferred_markdown_batches: usize,
}
impl PreparedRows {
    pub(super) fn layouts(&self) -> impl Iterator<Item = &c::Layout> {
        self.source
            .order
            .iter()
            .filter_map(|id| self.rows.get(id).map(|row| &row.layout))
    }
}
#[derive(Default)]
pub struct Preparer {
    theme: Theme,
    theme_revision: u64,
    metrics: Metrics,
    rows: im::HashMap<ItemId, Arc<CachedRow>>,
    scope: Option<(Option<SessionId>, Epoch)>,
    layout_builds: usize,
    terminal_outputs: std::collections::HashMap<ItemId, String>,
    documents: super::rich::Documents,
    /// Global text zoom for the rows it prepares; rows at another zoom are rebuilt.
    zoom: Zoom,
}
impl Preparer {
    pub(crate) fn prepare_pending(
        &mut self,
        prepared: &mut PreparedRows,
        prompts: &im::OrdMap<OpId, super::bridge::PendingPrompt>,
        faces: &super::rich::Faces,
    ) -> Result<(), String> {
        let ctx = self.context();
        let ink = super::face::ink(ctx);
        let gutter = (14_f32 - 4.).max(ctx.density.tokens().gap as f32);
        let width = prepared.width - 2. * gutter - 8.;
        for prompt in prompts.values().filter(|p| {
            Some(p.session) == prepared.source.session && p.epoch == prepared.source.epoch
        }) {
            let doc = self.documents.prepare(
                ItemId(prompt.op.0),
                &prompt.text,
                true,
                (width - 26.).max(1.),
                self.zoom.scale(),
                &self.metrics,
            )?;
            let id = ItemId(prompt.op.0);
            let source = faces.get(&id).is_some_and(|f| f.source);
            let content =
                super::face::prompt_content(doc.element.clone(), id, source, &prompt.text, &ink);
            let bubble = super::face::prompt_bubble(
                content,
                &format!("YOU · {}", prompt.status),
                true,
                &ink,
            );
            let mut element = super::face::frame(
                super::face::Frame {
                    padding: [4., gutter + 4., 18., gutter + 4.],
                    ..Default::default()
                },
                vec![bubble.into()],
            );
            flat(&mut element);
            super::face::zoom_text(&mut element, self.zoom.scale());
            let mut layout =
                element.layout(Rect::new(0., 0., prepared.width, 0.), ctx, &self.metrics)?;
            if !source {
                super::rich::restore(&mut layout, &doc.blocks);
            }
            prepared.pending.insert(prompt.op, Arc::new(layout));
        }
        Ok(())
    }
    pub fn context(&self) -> ui::Context {
        ui::Context::new(self.theme, Density::Cozy)
    }
    pub fn set_theme(&mut self, theme: Theme) -> bool {
        if self.theme == theme {
            return false;
        }
        self.set_theme_revision(theme, self.theme_revision.wrapping_add(1))
    }
    pub(crate) fn set_theme_revision(&mut self, theme: Theme, revision: u64) -> bool {
        if self.theme == theme && self.theme_revision == revision {
            return false;
        }
        self.theme = theme;
        self.theme_revision = revision;
        self.rows.clear();
        self.documents.set_theme(theme);
        true
    }
    pub fn set_zoom(&mut self, zoom: Zoom) {
        self.zoom = zoom;
    }
    /// Blob loading is completed before preparation, on the background path.
    pub fn has_terminal_output(&self, id: &ItemId) -> bool {
        self.terminal_outputs.contains_key(id)
    }
    pub fn terminal_output(&mut self, id: ItemId, text: String) {
        self.terminal_outputs.insert(id, text);
        self.rows.remove(&id);
    }

    pub(crate) fn same_scope(&self, model: &Transcript) -> bool {
        self.scope == Some((model.session, model.epoch))
    }
    pub(crate) fn scope(&mut self, model: &Transcript) {
        let scope = (model.session, model.epoch);
        if self.scope != Some(scope) {
            self.rows.clear();
            self.terminal_outputs.clear();
            self.documents = Default::default();
            self.documents.set_theme(self.theme);
            self.scope = Some(scope);
        }
    }
    /// Caller must use the bounded runtime::blocking preparation pool.
    pub fn prepare(&mut self, source: Arc<Transcript>, width: f32) -> Result<PreparedRows, String> {
        self.prepare_with_faces(source, width, &Default::default())
    }
    pub fn prepare_with_faces(
        &mut self,
        source: Arc<Transcript>,
        width: f32,
        faces: &super::rich::Faces,
    ) -> Result<PreparedRows, String> {
        let ctx = self.context();
        let model = &source;
        let width = width.max(240.);
        let zoom = self.zoom;
        let scale = zoom.scale();
        let gutter = ctx.density.tokens().gap as f32;
        self.scope(model);
        let mut total_height = 0.;
        bbt_rt::assert_not_ui_thread();
        let folds = Folds::of(model, faces);
        for (index, id) in model.order.iter().enumerate() {
            let row = &model.items[id];
            let face = faces.get(id).cloned().unwrap_or_default();
            if folds.parent.contains_key(id) {
                // shown inside its tool card; that card's signature tracks it
                self.rows.remove(id);
                continue;
            }
            if row.completed.is_some()
                && let Some(cached) = self.rows.get(id)
                && cached.width == width
                && cached.zoom == zoom.percent()
                && cached.completed == row.completed
                && cached.deps == folds.deps(model, id, self)
                && cached.face == face
            {
                let h = cached.layout.rect.h;
                total_height += h;
                continue;
            }
            let deps = folds.deps(model, id, self);
            // Transcript rows fill their pane with 14 px side padding; the composer
            // keeps its independent reading-width cap.
            let gutter = (14_f32 - 4.).max(gutter);
            let content_width = width - 2. * gutter - 8.;
            let (element, rich) = self.face_row(
                model,
                row,
                &face,
                &folds,
                index,
                width,
                gutter,
                content_width,
            )?;
            let mut element = named(format!("item/{id}"), element);
            flat(&mut element);
            super::face::zoom_text(&mut element, scale);
            let mut layout = element.layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?;
            if let Some(rich) = &rich {
                super::rich::restore(&mut layout, &rich.blocks);
            }
            let height = layout.rect.h;
            self.rows.insert(
                *id,
                Arc::new(CachedRow {
                    width,
                    zoom: zoom.percent(),
                    completed: row.completed,
                    deps,
                    face,
                    layout,
                }),
            );
            self.layout_builds += 1;
            total_height += height;
        }
        Ok(PreparedRows {
            theme: self.theme,
            theme_revision: self.theme_revision,
            source,
            width,
            zoom: zoom.percent(),
            rows: self.rows.clone(),
            height: total_height,
            pending: HashMap::new(),
            layout_builds: self.layout_builds,
            parsed_markdown_bytes: self.documents.parsed_bytes,
            received_markdown_bytes: self.documents.received_bytes,
            markdown_block_layouts: self.documents.block_layouts,
            markdown_appends: self.documents.appends,
            deferred_markdown_batches: self.documents.deferred_batches,
        })
    }
}

/// Contentless decorations, deliberately excluded from content-box semantics.
/// Keep this list explicit: adding another pattern requires changing its test.
pub const CONTENTLESS_SHAPE_PATTERNS: &[&str] = &[
    "*/rail",                 // StreamDock's role-colour rule has no content slot.
    "composer/text/caret",    // The insertion marker represents a text position.
    "composer/address/*/dot", // An address status dot encodes state with colour.
];
fn contentless_shape(name: &str) -> Option<usize> {
    CONTENTLESS_SHAPE_PATTERNS.iter().position(|pattern| {
        pattern
            .split_once('*')
            .map_or(name == *pattern, |(prefix, suffix)| {
                name.len() >= prefix.len() + suffix.len()
                    && name.starts_with(prefix)
                    && name.ends_with(suffix)
            })
    })
}
// Rules, status dots and the editor caret have no content slot. Retain their
// solved geometry/colour as drawing primitives; cards and controls keep Box.
fn paint_shapes(node: &mut c::Layout) {
    let shape = match &node.kind {
        c::LayoutKind::Named(name) => contentless_shape(name),
        _ => None,
    };
    if matches!(shape, Some(0 | 1))
        && let Some(rail) = node.children.first_mut()
        && let c::LayoutKind::Box { style, .. } = &rail.kind
        && rail.children.is_empty()
    {
        rail.kind = c::LayoutKind::Rects {
            rects: Arc::from([(Rect::new(0., 0., rail.rect.w, rail.rect.h), style.fill)]),
            scale: 1.,
            repeat: None,
        };
        return;
    }
    if shape == Some(2)
        && let Some(dot) = node.children.first_mut()
        && let c::LayoutKind::Box { style, .. } = &dot.kind
        && dot.children.is_empty()
    {
        // The address status dot is likewise a shape, with no content slot.
        use bbt_ui_model::draw::{PathCommand as P, PathStroke};
        let [x, y] = [dot.rect.w / 2., dot.rect.h / 2.];
        let k = 0.552_284_8;
        dot.kind = c::LayoutKind::Path {
            points: Arc::from([
                P::Move([x, 0.]),
                P::Cubic([[x + x * k, 0.], [2. * x, y - y * k], [2. * x, y]]),
                P::Cubic([[2. * x, y + y * k], [x + x * k, 2. * y], [x, 2. * y]]),
                P::Cubic([[x - x * k, 2. * y], [0., y + y * k], [0., y]]),
                P::Cubic([[0., y - y * k], [x - x * k, 0.], [x, 0.]]),
            ]),
            stroke: PathStroke {
                width: 0.,
                ..Default::default()
            },
            color: style.fill,
            fill: Some(style.fill),
            scale: 1.,
        };
        return;
    }
    for child in &mut node.children {
        paint_shapes(child);
    }
}

/// Calls `f` on every named node, nearest the root first; `f` answers whether it took the
/// node, in which case the walk does not descend into it.
fn each_named(node: &mut c::Layout, f: &mut impl FnMut(&str, &mut c::Layout) -> bool) {
    if let c::LayoutKind::Named(name) = &node.kind {
        let name = name.clone();
        if f(&name, node) {
            return;
        }
    }
    for child in &mut node.children {
        each_named(child, f);
    }
}
fn find_mut<'a>(node: &'a mut c::Layout, key: &str) -> Option<&'a mut c::Layout> {
    if matches!(&node.kind, c::LayoutKind::Named(name) if name == key) {
        Some(node)
    } else {
        node.children
            .iter_mut()
            .find_map(|child| find_mut(child, key))
    }
}

fn approval_choices(
    id: ItemId,
    options: &[ApprovalDecisionKind],
) -> ui::modules::approval::Approval {
    ui::modules::approval::Approval::new(
        format!("approval/{id}"),
        "",
        options
            .iter()
            .map(|kind| {
                let (label, _) = approval(kind.clone());
                ui::modules::approval::OptionItem {
                    key: label.into(),
                    label: label.into(),
                    enabled: !kind.is_unknown(),
                }
            })
            .collect(),
    )
}
fn approval_atom(
    id: ItemId,
    subject: &ApprovalSubject,
    options: &[ApprovalDecisionKind],
) -> ui::modules::approval::Approval {
    let mut prompt = approval_choices(id, options);
    let summary = subject
        .summary
        .split_once(' ')
        .and_then(|(name, raw)| {
            serde_json::from_str::<serde_json::Value>(raw)
                .ok()
                .map(|_| format!("{name}: {}", arguments(raw)))
        })
        .unwrap_or_else(|| subject.summary.clone());
    let paths = subject
        .paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    prompt.title = format!(
        "{summary}\n{paths}\n{}",
        subject.command.as_deref().unwrap_or("")
    );
    prompt
}
fn inline_row(children: Vec<c::Item>) -> c::Element {
    c::Element::Stack(c::Stack {
        direction: Axis::Row,
        gap: ui::Length::Px(8.),
        align: c::Alignment::Start,
        children,
    })
}
fn approval_element(prompt: &ui::modules::approval::Approval, ctx: ui::Context) -> c::Element {
    let mut atom = prompt.element(ctx);
    if let c::Element::Named { content, .. } = &mut atom
        && let c::Element::Stack(stack) = content.as_mut()
    {
        stack.children.truncate(1);
        let buttons = prompt
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let kind = match option.key.as_str() {
                    "Allow" => ButtonKind::Primary,
                    "Deny" => ButtonKind::Danger,
                    _ => ButtonKind::Ghost,
                };
                let state = if !option.enabled {
                    ButtonState::Disabled
                } else if prompt.selected() == Some(i) {
                    ButtonState::Focus
                } else {
                    ButtonState::Idle
                };
                let mut button = Button {
                    label: option.label.clone(),
                    kind,
                    state,
                    icon: None,
                    key: None,
                }
                .element(ctx)
                .expect("approval button style");
                if kind == ButtonKind::Ghost
                    && let c::Element::Box(b) = &mut button
                {
                    b.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface2));
                }
                named(format!("{}/{}", prompt.key, option.key), button).into()
            })
            .collect();
        stack.children.push(
            named(
                format!("inline/{}/choices", prompt.key),
                inline_row(buttons),
            )
            .into(),
        );
    }
    atom
}
// This host has a flat material pass. Preserve toolkit geometry, colours and
// actions, while choosing the flat surface override before solving the tree.
pub(super) fn flat(element: &mut c::Element) {
    match element {
        c::Element::Box(b) => {
            b.style.overrides.material = Some(ui::Material::Flat);
            for item in &mut b.children {
                flat(&mut item.element);
            }
        }
        c::Element::Stack(s) => {
            for item in &mut s.children {
                flat(&mut item.element);
            }
        }
        c::Element::Flow(f) => {
            for item in &mut f.children {
                flat(&mut item.element);
            }
        }
        c::Element::Named { content, .. }
        | c::Element::FixedHeight { content, .. }
        | c::Element::Filter { content, .. }
        | c::Element::InlineMiddle { content, .. }
        | c::Element::Baseline { content, .. } => flat(content),
        c::Element::ScrollView(s) => flat(&mut s.content),
        c::Element::Overlay(s) => {
            s.style.overrides.material = Some(ui::Material::Flat);
            flat(&mut s.content);
        }
        c::Element::Layers { base, overlays } => {
            flat(base);
            for p in overlays {
                flat(&mut p.content);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod onboarding_markdown_tests {
    use super::*;
    #[tokio::test]
    async fn transcript_fills_wide_and_three_column_panes() {
        crate::runtime::blocking(move || {
        let ctx = ui::Context::new(Theme::Light, Density::Cozy);
        let ink = super::super::face::ink(ctx);
        let metrics = Metrics::default();
        for width in [280., 440., 960., 1600.] {
            let mut model = Transcript::default();
            for line in include_str!("../../../tests/fixtures/protocol/all-items.jsonl").lines() {
                model.apply(&serde_json::from_str::<Envelope>(line).unwrap());
            }
            let prepared = Preparer::default().prepare(Arc::new(model), width).unwrap();
            for row in prepared.rows.values() {
                assert!(row.layout.rect.x <= 14., "centred row in {width}px pane: {:?}", row.layout.rect);
                assert!(row.layout.rect.right() >= width - 14.);
            }
            let bubble = super::super::face::prompt_bubble(named("copy", label("short")), "YOU", false, &ink)
                .layout(Rect::new(0., 0., width - 28., 80.), ctx, &metrics).unwrap();
            assert!((bubble.children[0].rect.w - (width - 28.)).abs() < 0.1);
        }
        }).await;
    }
    #[test]
    fn tool_header_and_reasoning_text_share_alignment() {
        let ctx = ui::Context::new(Theme::Light, Density::Cozy);
        let ink = super::super::face::ink(ctx);
        let metrics = Metrics::default();
        let head = |target: &str| {
            super::super::face::head(
                super::super::face::Head {
                    key: "head".into(),
                    tool_kind: "execute",
                    verb: "Run command".into(),
                    target: target.into(),
                    mono: true,
                    place: None,
                    stat: None,
                    meta: None,
                    badge: None,
                    dur: None,
                    status: super::super::face::Status::Done,
                },
                &ink,
            )
            .layout(Rect::new(0., 0., 640., 40.), ctx, &metrics)
            .unwrap()
        };
        fn baselines(node: &c::Layout, out: &mut Vec<f32>) {
            if let c::LayoutKind::Text { runs, .. } = &node.kind {
                out.extend(
                    runs.iter()
                        .filter(|r| {
                            r.text == "Run command"
                                || r.text == "mcp.bboxy.message_list"
                                || r.text == "date -u"
                        })
                        .map(|r| r.baseline),
                );
            }
            for child in &node.children {
                baselines(child, out);
            }
        }
        for target in ["mcp.bboxy.message_list", "date -u"] {
            let layout = head(target);
            let mut lines = Vec::new();
            baselines(&layout, &mut lines);
            assert_eq!(
                lines.len(),
                2,
                "tool label must remain visible for {target}"
            );
            assert!(
                (lines[0] - lines[1]).abs() < 0.01,
                "text baselines: {lines:?}"
            );
        }
        let row =
            super::super::face::thought_row("reasoning/test/toggle", true, None, "", true, &ink);
        let body =
            super::super::face::thought_body(named("reasoning-copy", label("Details")), &ink);
        let layout = column(vec![row.into(), body.into()], None)
            .layout(Rect::new(0., 0., 440., 100.), ctx, &metrics)
            .unwrap();
        let disc = find(&layout, "reasoning/test/toggle/icon").unwrap().rect;
        let rail = find(&layout, "reasoning/rail").unwrap().rect;
        assert!((disc.x + disc.w / 2. - rail.x - rail.w / 2.).abs() < 0.01);
        assert!(
            (rail.x + rail.w / 2. - 5.).abs() < 0.01,
            "same axis as tool-group rail"
        );
        assert_eq!(
            find(&layout, "reasoning/test/toggle/label").unwrap().rect.x,
            find(&layout, "reasoning-copy").unwrap().rect.x
        );
    }
    #[test]
    fn header_dropdowns_fit_narrow_panes_and_keep_revision_fences() {
        let model = Transcript {
            session: Some(SessionId::new()),
            profile: Some(("codex".into(), "acp:codex-acp".into())),
            agent_config: Some(AgentConfig {
                instance: OpId::new(),
                revision: 7,
                pending: None,
                status: "Confirmed by agent".into(),
                options: ["model", "thought_level"]
                    .into_iter()
                    .map(|category| AgentOption {
                        id: category.into(),
                        name: category.into(),
                        category: category.into(),
                        current: "3".into(),
                        legacy: false,
                        choices: (0..8)
                            .map(|i| AgentChoice {
                                value: i.to_string(),
                                name: format!("Choice {i}"),
                            })
                            .collect(),
                    })
                    .collect(),
            }),
            ..Default::default()
        };
        let mut state = FrameState {
            ready: true,
            transcript: Arc::new(model),
            ..Default::default()
        };
        for width in [240., 440., 640., 960.] {
            let mut view = HarnessView::default();
            view.render(&state, [width, 520.]).unwrap();
            for category in ["model", "thought_level"] {
                let rect = find(
                    view.frame.layout.as_ref().unwrap(),
                    &format!("config/{category}"),
                )
                .unwrap()
                .rect;
                assert!(
                    rect.x >= 0. && rect.right() <= width,
                    "{category}: {rect:?} in {width}"
                );
            }
            if width >= 640. {
                let layout = view.frame.layout.as_ref().unwrap();
                let profile = find(layout, "profile").unwrap().rect;
                let model = find(layout, "config/model").unwrap().rect;
                assert!(
                    (profile.y + profile.h / 2. - model.y - model.h / 2.).abs() < 0.1,
                    "header must fit one row at {width}"
                );
            }
            view.config_open = Some("model".into());
            view.config_selected = 3;
            view.invalidate();
            view.render(&state, [width, 520.]).unwrap();
            let popup = &view.menus.last.as_ref().unwrap().layout;
            assert!(popup.rect.right() <= width && popup.rect.bottom() <= 520.);
            assert!(popup.rect.h <= 280.);
        }
        let Some(Action::AgentConfig(op)) = config_action(&state, "model", 4) else {
            panic!("choice missing")
        };
        assert!(matches!(op.op, Op::SetAgentConfig { revision: 7, .. }));
        assert!(config_action(&state, "model", 3).is_none());
        state.replay = true;
        assert!(config_action(&state, "model", 4).is_none());
        state.replay = false;
        Arc::make_mut(&mut state.transcript)
            .agent_config
            .as_mut()
            .unwrap()
            .pending = Some(OpId::new());
        assert!(config_action(&state, "model", 4).is_none());
    }
    #[tokio::test]
    async fn onboarding_user_markdown_pending_and_history() {
        let device = bbt_render::device::DeviceContext::new().await.unwrap();
        crate::runtime::blocking(move || {
            let mut renderer=bbt_render::chrome::ChromeRenderer::with_context(device,super::super::assets::icons()).unwrap();
            let source_text = "# Heading\n\n- **bold** [link](https://example.com) `inline`\n\n```python\nprint('never executed')\n```";
            let base: Envelope = serde_json::from_str(include_str!("../../../tests/fixtures/protocol/all-items.jsonl").lines().next().unwrap()).unwrap();
            let mut model = Transcript::default();
            model.apply(&base);
            let id = ItemId::new();
            let mut event = base.clone();
            event.seq = Some(Seq(2)); event.at = Seq(2);
            event.event = Event::ItemStarted { turn: None, item: id, kind: ItemKind::UserMessage, parent: None, head: None };
            model.apply(&event);
            event.seq = Some(Seq(3)); event.at = Seq(3);
            event.event = Event::ItemCompleted { item: id, status: ItemStatus::Completed, body: Item::UserMessage { blocks: vec![InputBlock::Text { text: source_text.into() }, InputBlock::Mention { path: "caption.txt".into() }] } };
            model.apply(&event);
            let mut prompt = super::super::bridge::PendingPrompt::new(OpId::new(), base.session, base.epoch, source_text.into());
            prompt.status = "Queued".into();
            let prompts = im::ordmap! { prompt.op => prompt.clone() };
            for theme in [Theme::Light, Theme::Dark] {
                for width in [280., 440.] {
                    let mut p = Preparer::default(); p.set_theme(theme);
                    let mut prepared = p.prepare(Arc::new(model.clone()), width).unwrap();
                    p.prepare_pending(&mut prepared, &prompts, &Default::default()).unwrap();
                    let row = &prepared.rows[&id].layout;
                    let pending = &prepared.pending[&prompt.op];
                    fn text(n: &c::Layout) -> String {
                        let mut s = String::new();
                        if let c::LayoutKind::Text { runs, .. } = &n.kind { for r in runs { s.push_str(&r.text); } }
                        for child in &n.children { s.push_str(&text(child)); } s
                    }
                    for node in [row, pending.as_ref()] {
                        let t = text(node);
                        for want in ["Heading", "bold", "link", "inline", "never executed"] { assert!(t.contains(want), "missing {want}: {t}"); }
                        assert!(!t.contains("**bold**"));
                        assert!(t.contains("bold link"),"Markdown span separators must survive: {t}");
                        assert!(node.rect.h > 90.);
                        fn bounds(n: &c::Layout, width: f32) { if let c::LayoutKind::Text { runs, .. } = &n.kind { for r in runs { assert!(r.rect.right() <= width + 2., "{:?} > {width}", r.rect); } } for child in &n.children { bounds(child, width); } }
                        bounds(node, width);
                    }
                    assert!(text(row).contains("caption.txt"));
                    assert!(text(pending).contains("Queued"));
                    assert!(prepared.source.items[&id].body == model.items[&id].body);
                    let state=super::super::bridge::FrameState { transcript:Arc::new(model.clone()),prepared:Some(Arc::new(prepared)),pending_prompts:Arc::new(prompts.clone()),ready:true,replay:true,..Default::default() };
                    let mut view=HarnessView::default(); view.set_theme(theme);
                    let frame=view.render(&state,[width,1000.]).unwrap();
                    let visible=layout_text(frame.layout.as_ref().unwrap());
                    assert!(visible.contains("Queued") && visible.contains("Heading"));
                    assert!(frame.item_text(id).unwrap().contains("caption.txt"),"queued row must retain prior history paint");
                    let rgba=renderer.render_to_rgba(&frame.list,width as u32,1000,1.).unwrap();
                    if let Some(out)=std::env::var_os("ONBOARDING_CAPTURE_DIR") {
                        let out=std::path::PathBuf::from(out); std::fs::create_dir_all(&out).unwrap();
                        let mut encoder=png::Encoder::new(std::fs::File::create(out.join(format!("user-{theme:?}-{width}.png"))).unwrap(),width as u32,1000);
                        encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
                        encoder.write_header().unwrap().write_image_data(&rgba).unwrap();
                    }
                    let faces=im::ordmap! {id => super::super::rich::Face {source:true,..Default::default()}};
                    let source=p.prepare_with_faces(Arc::new(model.clone()),width,&faces).unwrap();
                    assert!(layout_text(&source.rows[&id].layout).contains("**bold**"),"selectable original source stays available");
                    let mut prepared=p.prepare(Arc::new(model.clone()),width).unwrap();
                    let empty = im::OrdMap::new(); p.prepare_pending(&mut prepared, &empty, &Default::default()).unwrap();
                    assert!(prepared.rows.contains_key(&id));
                }
            }
        }).await;
    }
}
