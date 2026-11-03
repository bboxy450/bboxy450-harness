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
    chat_scroll,
    model::{Transcript, ViewItem},
    stream_motion,
};
use crate::protocol::*;
use bbt_ui::{
    self as ui, components as c,
    modules::{
        composer,
        seat_row::named,
        stream_dock::{Entry, Message, StreamDock},
    },
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
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Send,
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
    /// Inside an open menu (true) keeps it; outside it (false) closes it.
    Menu(bool),
    /// The switcher's `+ New profile`: the native form is not built yet.
    NewProfile,
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
    pub list: DrawList,
    pub layout: Option<c::Layout>,
    pub visible: Vec<(ItemId, ItemKind, Rect)>,
    pub actions: Vec<(Rect, Action)>,
    /// Pointer hover zones of the open menu: the row name under each rect.
    pub hover: Vec<(Rect, String)>,
    pub scroll: f32,
    pub max_scroll: f32,
    pub timings: [std::time::Duration; 5],
    /// Primitive count of the frame under the menus; the menus' primitives follow it.
    pub base: usize,
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
pub struct HarnessView {
    pub composer: super::editor::Input,
    approval: Option<(ItemId, ui::modules::approval::Approval)>,
    completion: Option<ui::text_edit::CompletionPopover>,
    profile_menu: Option<super::menu::ProfileMenu>,
    pub scroll: chat_scroll::State,
    /// View-local offsets into worker-prepared Markdown tool panels.
    markdown_scroll: HashMap<ItemId, f32>,
    metrics: Metrics,
    cached: Option<CacheKey>,
    controls: Option<Controls>,
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
}
/// Which floating menu is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuKind {
    Slash,
    Profiles,
}
impl MenuKind {
    fn id(self) -> &'static str {
        match self {
            MenuKind::Slash => "menu/slash",
            MenuKind::Profiles => "menu/profiles",
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
    /// The row name under the pointer.
    hover: Option<String>,
    /// The hover changed: the frame must be rebuilt although nothing else did.
    dirty: bool,
}
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
            composer,
            approval: None,
            completion: None,
            profile_menu: None,
            scroll: Default::default(),
            markdown_scroll: HashMap::new(),
            metrics: Metrics::default(),
            cached: None,
            controls: None,
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
        }
    }
}
impl HarnessView {
    /// A wheel over an inset first scrolls that document, then the transcript at its edge.
    pub fn wheel(&mut self, at: [f32; 2], dy: f32) -> bool {
        fn contains(r: Rect, at: [f32; 2]) -> bool {
            at[0] >= r.x && at[0] < r.right() && at[1] >= r.y && at[1] < r.bottom()
        }
        fn panel(node: &c::Layout, at: [f32; 2]) -> Option<(ItemId, f32, f32)> {
            if let c::LayoutKind::Named(name) = &node.kind
                && let Some(id) = name.strip_prefix("mdout/").and_then(|s| s.parse().ok())
                && contains(node.rect, at)
                && let Some(scroll) = node.children.first()
                && let c::LayoutKind::ScrollView { content, offset } = scroll.kind
            {
                return Some((ItemId(id), offset[1], (content[1] - scroll.rect.h).max(0.)));
            }
            node.children.iter().find_map(|n| panel(n, at))
        }
        if let Some((id, offset, max)) = self
            .frame
            .layout
            .as_ref()
            .and_then(|root| find(root, "transcript"))
            .filter(|n| contains(n.rect, at))
            .and_then(|n| panel(n, at))
        {
            let next = (offset + dy).clamp(0., max);
            if next != offset {
                self.markdown_scroll.insert(id, next);
                self.invalidate();
                return true;
            }
        }
        if self
            .scroll
            .wheel(self.frame.scroll, self.frame.max_scroll, dy)
        {
            self.invalidate();
            return true;
        }
        false
    }

    /// Running with an empty draft, the send button is the stop square (.hx-send.stop).
    pub fn send_stops(&self, state: &FrameState) -> bool {
        !state.replay
            && !matches!(state.transcript.phase, Phase::Idle | Phase::Closed)
            && self.composer.view.text.is_empty()
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
        first_frame: bool,
        ink: &super::face::Ink,
    ) -> (c::Element, Option<f32>, HashMap<ItemId, f32>) {
        let model = &state.transcript;
        let mut children = vec![];
        let mut total = 0.;
        let mut enters = HashMap::new();
        if let Some(prepared) = prepared {
            // animate only what is visible (plus a margin): offscreen rows take end states
            let animating = self.anim.animates();
            let (view_top, view_bottom) =
                (self.frame.scroll - 240., self.frame.scroll + rows_h + 240.);
            for id in &model.order {
                let Some(row) = prepared.rows.get(id) else {
                    continue;
                };
                let h0 = row.layout.rect.h;
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
        }
        if children.is_empty() {
            if state.ready && model.order.is_empty() {
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
                children.push(
                    super::face::empty_session(
                        (!who.is_empty()).then_some(who),
                        leaf,
                        (0.12 * rows_h).max(0.),
                        ink,
                        &mut self.anim,
                    )
                    .into(),
                );
            } else {
                children.push(
                    label(if !state.ready {
                        "Loading session…"
                    } else {
                        "Preparing transcript…"
                    })
                    .into(),
                );
            }
        }
        if prepared.is_some() && !model.order.is_empty() {
            (
                column(children, None).fixed_height(total, "transcript-content", 0.),
                Some(total),
                enters,
            )
        } else {
            (column(children, None), None, enters)
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
        if first_frame {
            // a session's first frame shows its tail at once
            return max;
        }
        self.anim
            .spring_from("scroll", self.frame.scroll.min(max), max, 180.)
    }
    /// The same Element is used by a standalone surface and the offscreen driver.
    pub fn render(&mut self, state: &FrameState, size: [f32; 2]) -> Result<&Frame, String> {
        self.profiles = state.profiles.clone();
        self.current_profile = state.transcript.profile.as_ref().map(|(p, _)| p.clone());
        self.birth = Some(state.transcript.epoch);
        let offset_key = self.scroll.layout_offset();
        // the caret blink only shows while the composer is focused; otherwise it must not
        // keep busting the idle frame cache as the clock advances
        let blink = self.focused && self.anim.blink_at(1000., self.anim.now() - self.last_edit);
        let draft = format!(
            "{:?}{:?}{:?}{:?}{}{:?}{blink}",
            self.composer.view,
            self.notice,
            state.error,
            state.notice,
            self.focused,
            self.profile_menu
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
        let ctx = context();
        let model = &state.transcript;
        let now = self.anim.now();
        self.motion.begin_frame(state, now);
        let width = size[0].max(240.);
        let prepared = state
            .prepared
            .as_ref()
            .filter(|p| Arc::ptr_eq(&p.source, &state.transcript));
        // A session's first frame takes its end state: nothing enters, the tail is shown.
        let scope = (model.session, model.epoch);
        let first_frame = self.known_scope != Some(scope);
        if first_frame {
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
        let focused = self.focused;
        let controls_key = format!(
            "{draft}{status}{tone:?}{live}{focused}{}{:?}{:?}{:?}",
            self.profile_menu.is_some(),
            state.ready,
            model.profile,
            model.meta.as_ref().map(|m| &m.cwd)
        );
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
                self.transcript(state, prepared, rows_h, first_frame, &ink);
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
            let composer = super::face::composer(
                &self.composer.view,
                super::face::ComposerPhase {
                    live: live && !ask,
                    ask,
                    focused,
                },
                ctx,
                width,
                &self.metrics,
                &mut self.anim,
            )?;
            let composer_h = composer
                .layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?
                .rect
                .h;
            let header_h = 40.;
            let rows_h = (size[1] - composer_h - header_h).max(1.);
            let (transcript, content_h, enters) =
                self.transcript(state, prepared, rows_h, first_frame, &ink);
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
            let header = named(
                "toolbar",
                super::face::header(
                    model.profile.clone(),
                    model
                        .meta
                        .as_ref()
                        .and_then(|m| m.cwd.file_name())
                        .map(|n| n.to_string_lossy().into_owned()),
                    &status,
                    tone,
                    self.profile_menu.is_some(),
                    &super::face::ink(ctx),
                ),
            );
            let root = column(
                vec![
                    c::Item {
                        element: header,
                        extent: c::Extent::Fixed(header_h),
                    },
                    c::Item {
                        element: rows,
                        extent: c::Extent::Fixed(rows_h),
                    },
                    composer.into(),
                ],
                Some(size[1]),
            );

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
        let mut frame = Frame::default();
        frame.list.rect(
            Rect::new(0., 0., width, size[1]),
            0.,
            Color::hex(ctx.theme.palette().bg),
        );
        for paint in c::paint(&layout, ctx) {
            append_paint(&mut frame.list, paint)?;
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
        frame.layout = Some(layout);
        self.scroll.layout_solved();
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
        let ctx = context();
        let ink = super::face::ink(ctx);
        let width = size[0].max(240.);
        self.frame.base = self.frame.list.primitives.len();
        let mut hover = vec![];
        let mut front = vec![];
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
                super::menu::slash_menu(popover, &self.profiles, max_rows, &ink, &look),
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
            let mut popup = menu.element(&self.profiles, current, &ink, &look);
            flat(&mut popup);
            let popup = popup.layout(
                Rect::new(10., 40., super::menu::profile_menu_width(width), 0.),
                ctx,
                &self.metrics,
            )?;
            front.push((popup.rect, Action::Menu(true)));
            if let Some(node) = find(&popup, "pmenu/new") {
                front.push((node.rect, Action::NewProfile));
            }
            for p in menu.rows(&self.profiles) {
                let key = format!("pmenu/row/{}", p.name);
                if let Some(node) = find(&popup, &key) {
                    front.push((node.rect, Action::SwitchProfile(p.name.clone())));
                    hover.push((node.rect, key));
                }
            }
            Some((MenuKind::Profiles, popup, selected, true))
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
        self.frame.hover = hover;
        let shown = open.as_ref().map(|(kind, ..)| *kind);
        if shown.is_none() {
            // nothing to hover: a reopened menu starts clean
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
                    &mut self.frame.list,
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
            paint_overlay(&mut self.frame.list, &popup, ctx, &ink, kind, &motion)?;
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
    fn profile_names(&self) -> Vec<String> {
        self.profiles.iter().map(|p| p.name.clone()).collect()
    }
    pub fn edit(&mut self, command: ui::text_edit::Command) {
        self.composer.handle(composer::Command::Edit(command));
        self.completion = self.composer.popover(&self.profile_names());
        self.last_edit = self.anim.now();
        self.invalidate();
    }
    pub fn completion_open(&self) -> bool {
        self.completion.is_some()
    }
    pub fn profile_menu_open(&self) -> bool {
        self.profile_menu.is_some()
    }
    /// Open (or close) the header profile switcher, as the chip click does.
    pub fn toggle_profile_menu(&mut self) {
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
        if self.completion.take().is_some() || self.profile_menu.take().is_some() {
            self.invalidate();
            return true;
        }
        false
    }
    /// `/profile NAME` and the switcher: continue this conversation on NAME.
    fn switch_to(&mut self, name: &str, bridge: &Bridge) {
        self.profile_menu = None;
        self.notice = Some(if !bridge.state.profiles.iter().any(|p| p.name == name) {
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
        });
        self.scroll.follow_latest();
        self.invalidate();
    }
    pub fn activate(&mut self, action: Action, bridge: &Bridge) {
        match action {
            Action::Jump => self.scroll.follow_latest(),
            Action::Profiles => return self.toggle_profile_menu(),
            Action::SwitchProfile(name) => return self.switch_to(&name, bridge),
            Action::Menu(true) => return,
            Action::Menu(false) => self.profile_menu = None,
            Action::NewProfile => {
                self.profile_menu = None;
                self.notice = Some(
                    "New profile: add NAME.md to ~/.bboxy450-harness/profiles (no native form yet)"
                        .into(),
                );
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
            Action::Complete(request, text) => {
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
                    let text = sent.text.trim();
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
                        self.scroll.follow_latest();
                        return;
                    }
                    let op = match text {
                        "/interrupt" => Op::Interrupt { turn: None },
                        _ => Op::UserTurn {
                            blocks: vec![InputBlock::Text {
                                text: sent.text.clone(),
                            }],
                            mode: TurnMode::Queue,
                        },
                    };
                    match bridge.send(op) {
                        Ok(_) => {
                            self.composer.stored(&sent);
                            self.notice = None;
                            self.scroll.follow_latest();
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
                        self.scroll.follow_latest();
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
    for paint in c::paint(&chrome, ctx) {
        append_paint(&mut prims, paint)?;
    }
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
            MenuKind::Profiles => prims.rect(r, 9., Color(alpha(ink.seat, 0.12))),
        }
    }
    if let Some(rows) = find(popup, super::menu::LIST) {
        for paint in c::paint(rows, ctx) {
            append_paint(&mut prims, paint)?;
        }
    }
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
fn append_paint(list: &mut DrawList, paint: c::Paint) -> Result<(), String> {
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
                append_paint(&mut inner, op)?;
            }
            list.push(bbt_ui_model::draw::Prim::Group {
                primitives: inner.primitives,
                opacity,
                saturation,
                mask,
            });
        }
        // Adapted from bboxybook components::append_flat: a decorative rim
        // is optional in a flat offscreen/standalone composition.
        // https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
        c::Paint::Material {
            kind: ui::Material::Rim,
            ..
        } => {}
        c::Paint::Material { .. } => {
            return Err("Unsupported transcript material".into());
        }
    }
    Ok(())
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
fn status_text(status: &ItemStatus) -> String {
    match status {
        ItemStatus::Completed => "Completed".into(),
        ItemStatus::Failed { reason, .. } => format!("Failed: {reason}"),
        ItemStatus::Cancelled => "Cancelled".into(),
        ItemStatus::Interrupted => "Interrupted".into(),
        ItemStatus::Unknown(raw) => format!("Unsupported status: {}", unknown_tag(raw)),
    }
}
fn role(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::UserMessage => "user",
        ItemKind::AgentMessage | ItemKind::Reasoning | ItemKind::Plan => "assistant",
        ItemKind::ToolCall | ItemKind::Terminal | ItemKind::FileChange | ItemKind::TurnDiff => {
            "tool"
        }
        ItemKind::Subagent => "subagent",
        _ => "system",
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
fn display_body(row: &ViewItem, terminal_output: Option<&String>) -> String {
    let text = match &row.body {
        Item::AgentMessage { markdown } => markdown.clone(),
        Item::Reasoning { summary, .. } => summary.clone(),
        Item::UserMessage { blocks } => blocks
            .iter()
            .map(|b| match b {
                InputBlock::Text { text } => text.clone(),
                InputBlock::Image { blob } => format!("Image: {} · {} bytes", blob.mime, blob.len),
                InputBlock::Mention { path } => format!("Mention: {}", path.display()),
                InputBlock::Unknown(raw) => format!("Unsupported input: {}", unknown_tag(raw)),
                other => other.display_text(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Item::ToolCall {
            name,
            input_raw,
            output,
            ..
        } => {
            let output = output
                .iter()
                .map(|b| match b {
                    OutputBlock::Text { text } => text.clone(),
                    OutputBlock::Image { blob } => {
                        format!("Image: {} · {} bytes", blob.mime, blob.len)
                    }
                    OutputBlock::Json { value } => arguments(&value.to_string()),
                    OutputBlock::Diff { path, unified, .. } => {
                        format!("{}\n{unified}", path.display())
                    }
                    OutputBlock::Error { text } => format!("Error: {text}"),
                    OutputBlock::Unknown(raw) => {
                        format!("Unsupported tool output: {}", unknown_tag(raw))
                    }
                    other => other.display_text(),
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{name}\nArguments: {}\n{output}", arguments(input_raw))
        }
        Item::Terminal {
            command,
            exit,
            signal,
            output,
            ..
        } => {
            let result = exit.map_or_else(
                || "Exit code: pending".into(),
                |c| format!("Exit code: {c}"),
            );
            let signal = signal.map_or(String::new(), |s| format!(" · Signal: {s}"));
            let tail = terminal_output.map(String::as_str).unwrap_or_else(|| {
                if !row.terminal_tail.is_empty() {
                    &row.terminal_tail
                } else if output.as_ref().is_some_and(|b| b.len == 0) {
                    "(no output)"
                } else {
                    "Output pending"
                }
            });
            format!("{command}\n{result}{signal}\n{tail}")
        }
        Item::FileChange { path, unified, .. } => format!("{}\n{unified}", path.display()),
        Item::TurnDiff { unified } => unified.clone(),
        Item::Plan { entries } => entries
            .iter()
            .map(|e| {
                let status = match e.status {
                    PlanStatus::Pending => "Pending",
                    PlanStatus::InProgress => "In progress",
                    PlanStatus::Completed => "Completed",
                    PlanStatus::Cancelled => "Cancelled",
                    PlanStatus::Unknown(_) => "Unknown",
                };
                format!("{status}: {}", e.text)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Item::Approval {
            subject, decision, ..
        } => {
            let result = match decision {
                None => "Awaiting decision",
                Some(ApprovalDecision::Allow) => "Allowed",
                Some(ApprovalDecision::AllowAlways) => "Always allowed",
                Some(ApprovalDecision::Deny { .. }) => "Denied",
                _ => "Decision unavailable",
            };
            let paths = subject
                .paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}\n{}\n{paths}\n{result}",
                subject.summary,
                subject.command.as_deref().unwrap_or("")
            )
        }
        Item::Image { mime, blob } => format!("Image: {mime} · {} bytes", blob.len),
        Item::Compaction {
            summary,
            replaced_through,
        } => format!(
            "Compacted through event {} · summary {} bytes",
            replaced_through.0, summary.len
        ),
        Item::ContextTrim { clipped_through } => {
            format!("Context trimmed through event {}", clipped_through.0)
        }
        Item::Usage { usage } => format!(
            "Input: {} tokens · Output: {} tokens\nCached: {} tokens · Reasoning: {} tokens",
            usage.input_tokens,
            usage.output_tokens,
            usage.cached_input_tokens.unwrap_or(0),
            usage.reasoning_tokens.unwrap_or(0)
        ),
        Item::Notice { text, .. } => text.clone(),
        Item::Subagent { session, .. } => format!("Session: {session}"),
        Item::Command { .. } | Item::Hook { .. } | Item::SystemMessage { .. } => {
            row.body.extension_text().unwrap_or_default()
        }
        Item::Native { .. } => crate::frontends::native_label(&row.body).unwrap_or_default(),
        Item::Unknown(raw) => format!("Unsupported item: {}", unknown_tag(raw)),
    };
    if text.is_empty() { "…".into() } else { text }
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
    markdown_scroll: &HashMap<ItemId, f32>,
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
    markdown_scroll: &HashMap<ItemId, f32>,
    runs: &mut dyn stream_motion::Runs,
) -> Option<c::Layout> {
    let rect = Rect::new(node.rect.x + dx, node.rect.y + dy, node.rect.w, node.rect.h);
    let visible = intersection(rect, viewport);
    if visible.w <= 0. || visible.h <= 0. {
        runs.skip(node);
        return None;
    }
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(id) = name.strip_prefix("mdout/").and_then(|s| s.parse().ok())
        && let Some(scroll) = node.children.first()
        && let c::LayoutKind::ScrollView { content, offset } = scroll.kind
    {
        let y = markdown_scroll
            .get(&ItemId(id))
            .copied()
            .unwrap_or(0.)
            .clamp(0., (content[1] - rect.h).max(0.));
        return Some(c::Layout {
            rect,
            baseline: node.baseline,
            kind: node.kind.clone(),
            children: vec![c::Layout {
                rect,
                baseline: scroll.baseline,
                kind: c::LayoutKind::ScrollView {
                    content,
                    offset: [offset[0], y],
                },
                children: scroll
                    .children
                    .iter()
                    .filter_map(|child| {
                        visible_layout(
                            child,
                            dx,
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
pub(super) fn translate(node: &mut c::Layout, dx: f32, dy: f32) {
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
    pub source: Arc<Transcript>,
    pub width: f32,
    rows: im::HashMap<ItemId, Arc<CachedRow>>,
    /// The rows' solved heights summed: the content height once every height tween rests.
    pub height: f32,
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
    metrics: Metrics,
    rows: im::HashMap<ItemId, Arc<CachedRow>>,
    scope: Option<(Option<SessionId>, Epoch)>,
    layout_builds: usize,
    terminal_outputs: std::collections::HashMap<ItemId, String>,
    documents: super::rich::Documents,
}
impl Preparer {
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
        let ctx = context();
        let model = &source;
        let width = width.max(240.);
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
                && cached.completed == row.completed
                && cached.deps == folds.deps(model, id, self)
                && cached.face == face
            {
                let h = cached.layout.rect.h;
                total_height += h;
                continue;
            }
            let trim_callout = if let Item::ContextTrim { clipped_through } = &row.body {
                Some(format!(
                    "### Context trimmed through event {}\n\nTool output limited to **2 KiB**; reasoning and images omitted from model context.",
                    clipped_through.0
                ))
            } else {
                None
            };
            if face_rows::handles(model, row) {
                let deps = folds.deps(model, id, self);
                // .hx-feed: a centred 860 px column with 14 px padding; the row region
                // adds its own 4 px inset, so the frame gutter carries the rest
                let gutter = (((width - 860.) / 2.).max(0.) + 14. - 4.).max(gutter);
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
                let mut layout =
                    element.layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?;
                if let Some(rich) = &rich {
                    super::rich::restore(&mut layout, &rich.blocks);
                }
                let height = layout.rect.h;
                self.rows.insert(
                    *id,
                    Arc::new(CachedRow {
                        width,
                        completed: row.completed,
                        deps,
                        face,
                        layout,
                    }),
                );
                self.layout_builds += 1;
                total_height += height;
                continue;
            }
            let sourceable = matches!(row.kind, ItemKind::AgentMessage | ItemKind::Reasoning);
            let markdown = match &row.body {
                Item::AgentMessage { markdown } => Some(markdown.as_str()),
                Item::Reasoning { summary, .. } => Some(summary.as_str()),
                Item::ContextTrim { .. } => trim_callout.as_deref(),
                _ => None,
            };
            let message = Message {
                from: kind_label(row.kind.clone()).into(),
                role: role(row.kind.clone()).into(),
                text: if markdown.is_some()
                    || matches!(
                        row.kind,
                        ItemKind::ToolCall
                            | ItemKind::FileChange
                            | ItemKind::TurnDiff
                            | ItemKind::Approval
                    ) {
                    String::new()
                } else {
                    display_body(row, self.terminal_outputs.get(id))
                },
                time: row.status.as_ref().map_or("Streaming".into(), status_text),
                message_ids: vec![id.to_string()],
                ..Default::default()
            };
            let dock = StreamDock {
                dock: false,
                framed: false,
                entries: vec![Entry::Message {
                    data: Box::new(message),
                    show_channel: Some(false),
                }],
            };
            let mut cards = dock.items_at(ctx, index, &self.metrics);
            let skeleton = cards[0].element.layout(
                Rect::new(0., 0., width - 2. * gutter, 0.),
                ctx,
                &self.metrics,
            )?;
            let content_key = format!("e{index}/body");
            let content_width = find(&skeleton, &content_key)
                .ok_or("missing StreamDock body")?
                .rect
                .w;
            let rich = markdown
                .map(|text| {
                    self.documents.prepare(
                        *id,
                        text,
                        row.completed.is_some(),
                        content_width,
                        &self.metrics,
                    )
                })
                .transpose()?;
            let mut body = vec![];
            let mut toggles = vec![];
            if row.kind == ItemKind::Reasoning {
                toggles.push(
                    inline_control(
                        format!("reasoning/{id}/toggle"),
                        if face.reasoning_open {
                            "Show reasoning"
                        } else {
                            "Hide reasoning"
                        },
                        ctx,
                    )
                    .into(),
                );
            }
            if let Some(rich) = &rich {
                if !sourceable {
                    body.push(rich.element.clone().into());
                } else {
                    toggles.push(
                        inline_control(
                            format!("source/{id}/toggle"),
                            if face.source {
                                "View formatted"
                            } else {
                                "View source"
                            },
                            ctx,
                        )
                        .into(),
                    );
                    body.push(
                        named(
                            format!("inline/{id}/toggles"),
                            inline_row(std::mem::take(&mut toggles)),
                        )
                        .into(),
                    );
                    if !face.reasoning_open {
                        if face.source {
                            let mut text = c::Text::new(
                                markdown.unwrap_or_default(),
                                ui::Text {
                                    mono: true,
                                    ..Default::default()
                                },
                            );
                            text.preserve_whitespace = true;
                            body.push(c::Element::Text(text).into());
                        } else {
                            body.push(rich.element.clone().into());
                        }
                    }
                }
            }
            match &row.body {
                Item::ToolCall {
                    name,
                    input_raw,
                    output,
                    ..
                } => {
                    use ui::modules::tool_call::{Status, ToolCall};
                    let status = match row.status {
                        None => Status::Running,
                        Some(ItemStatus::Completed) => Status::Succeeded,
                        Some(ItemStatus::Failed { .. }) => Status::Failed,
                        _ => Status::Cancelled,
                    };
                    let output = output
                        .iter()
                        .map(|b| match b {
                            OutputBlock::Text { text } | OutputBlock::Error { text } => {
                                text.clone()
                            }
                            OutputBlock::Diff { unified, .. } => unified.clone(),
                            OutputBlock::Json { value } => arguments(&value.to_string()),
                            OutputBlock::Image { blob } => {
                                format!("Image: {} · {} bytes", blob.mime, blob.len)
                            }
                            _ => "Unsupported output".into(),
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    let mut tool = ToolCall {
                        key: format!("tool/{id}"),
                        title: name.clone(),
                        status,
                        arguments: format!("Arguments: {}", arguments(input_raw)),
                        output,
                        arguments_open: !face.arguments_closed,
                        output_open: !face.output_closed,
                    }
                    .element(ctx, content_width, &self.metrics);
                    subdued_tool_parts(&mut tool, &face);
                    body.push(tool.into());
                }
                Item::FileChange { unified, .. } | Item::TurnDiff { unified } => {
                    use ui::modules::diff::{Diff, DiffView, Folds};
                    crate::runtime::blocking_hook(crate::runtime::BlockingWork::Diff);
                    let diff = Diff::parse(unified).map_err(str::to_string)?;
                    let mut folds = Folds::default();
                    for h in &face.closed_hunks {
                        folds.toggle(*h);
                    }
                    body.push(
                        DiffView {
                            key: &format!("diff/{id}"),
                            diff: &diff,
                            folds: &folds,
                        }
                        .element(ctx, content_width, &self.metrics)
                        .into(),
                    );
                }
                Item::Approval {
                    subject,
                    options,
                    decision,
                    ..
                } => {
                    if decision.is_some() {
                        body.push(label(display_body(row, None)).into());
                    } else {
                        let mut prompt = approval_atom(*id, subject, options);
                        for _ in 0..face.approval_selected {
                            prompt.handle(ui::modules::approval::Key::Down);
                        }
                        body.push(approval_element(&prompt, ctx).into());
                    }
                }
                _ => {}
            }
            let content = find_element_mut(&mut cards[0].element, &content_key)
                .ok_or("missing StreamDock content")?;
            let c::Element::Box(content) = content else {
                return Err("unexpected StreamDock body".into());
            };
            let mut inside = content.children.split_off(1);
            inside.extend(body);
            let mut inset = column(inside, None);
            if let c::Element::Box(b) = &mut inset {
                b.gap = ui::Length::Px(6.);
            }
            content
                .children
                .push(named(format!("body/{id}"), inset).into());
            let mut row_content = column(cards, None);
            if let c::Element::Box(b) = &mut row_content {
                b.style.overrides.padding = Some([0., gutter, 0., gutter].map(ui::Length::Px));
            }
            let mut element = named(format!("item/{id}"), row_content);
            flat(&mut element);
            let mut layout = element.layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?;
            if let Some(rich) = &rich {
                super::rich::restore(&mut layout, &rich.blocks);
            }
            paint_shapes(&mut layout);
            let height = layout.rect.h;
            self.rows.insert(
                *id,
                Arc::new(CachedRow {
                    width,
                    completed: row.completed,
                    deps: 0,
                    face,
                    layout,
                }),
            );
            self.layout_builds += 1;
            total_height += height;
        }
        Ok(PreparedRows {
            source,
            width,
            rows: self.rows.clone(),
            height: total_height,
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
fn inline_control(key: String, text: &str, ctx: ui::Context) -> c::Element {
    named(
        key,
        Button {
            label: text.into(),
            kind: ButtonKind::Ghost,
            state: ButtonState::Idle,
            icon: None,
            key: None,
        }
        .element(ctx)
        .expect("inline control style"),
    )
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
// The toolkit tool card owns disclosure geometry. Status colour belongs to the
// status chip; argument/output bands use the host's subdued theme surfaces.
fn subdued_tool_parts(element: &mut c::Element, face: &super::rich::Face) {
    match element {
        c::Element::Named { name, content } => {
            if (name.ends_with("/arguments") || name.ends_with("/output"))
                && let c::Element::Box(frame) = content.as_mut()
                && let c::Element::Box(inner) = &mut frame.children[0].element
            {
                // Natural height includes the body's inset and wrapped lines.
                // The toolkit's fixed height was measured before host padding.
                frame.style.overrides.height = None;
                frame.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface));
                inner.style.overrides.padding = Some([ui::Length::Px(0.); 4]);
                if inner.children.len() > 2 {
                    let body = &mut inner.children[2];
                    let mut inset = column(vec![body.element.clone().into()], None);
                    if let c::Element::Box(b) = &mut inset {
                        b.style.overrides.padding = Some([6., 8., 6., 8.].map(ui::Length::Px));
                    }
                    body.element = named(format!("{name}/content"), inset);
                    body.extent = c::Extent::MinContent;
                    // Spacing is layout, not an empty painted card.
                    inner.children[1] = c::Item {
                        element: c::Element::Spacer,
                        extent: c::Extent::Fixed(3.),
                    };
                }
                frame.children.truncate(1); // no decorative empty bumper
                frame.children[0].extent = c::Extent::MinContent;
            }
            if name.ends_with("/head")
                && let c::Element::Box(header) = content.as_mut()
            {
                header.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface2));
                header.style.overrides.padding = Some([0., 8., 0., 8.].map(ui::Length::Px));
            }
            if name.ends_with("/head/right/swatch") {
                // A visible disclosure glyph replaces the empty status swatch.
                let closed = if name.contains("/arguments/") {
                    face.arguments_closed
                } else {
                    face.output_closed
                };
                **content = disclosure_glyph(closed, context());
            }
            if name.ends_with("/head/sname")
                && let c::Element::Text(text) = content.as_mut()
            {
                for run in &mut text.runs {
                    run.style.paint = ui::Paint::solid(ui::Color::Foreground);
                }
            }
            subdued_tool_parts(content, face);
        }
        c::Element::Box(b) => {
            b.children.retain(|item| !matches!(&item.element, c::Element::Named { name, .. } if name.ends_with("/head/count")));
            for item in &mut b.children {
                subdued_tool_parts(&mut item.element, face);
            }
        }
        c::Element::Stack(stack) => {
            for item in &mut stack.children {
                subdued_tool_parts(&mut item.element, face);
            }
        }
        _ => {}
    }
}
fn disclosure_glyph(closed: bool, ctx: ui::Context) -> c::Element {
    use bbt_ui_model::draw::{PathCommand as P, PathStroke};
    // One header em bounds its cap height. A path uses the whole ink rectangle,
    // unlike the font-dependent 3 px ink inside a nominal 13 px triangle span.
    let side = ui::Style::new(ui::Preset::Band)
        .resolve(ctx)
        .expect("header style")
        .text
        .size;
    let points = if closed {
        [[0., 0.], [side, side / 2.], [0., side], [0., 0.]]
    } else {
        [[0., 0.], [side, 0.], [side / 2., side], [0., 0.]]
    };
    c::Element::Path(c::Path {
        size: [side, side],
        points: Arc::from([
            P::Move(points[0]),
            P::Line(points[1]),
            P::Line(points[2]),
            P::Line(points[3]),
        ]),
        stroke: PathStroke {
            width: 0.,
            ..Default::default()
        },
        paint: ui::Paint::solid(ui::Color::Foreground),
        fill: Some(ui::Paint::solid(ui::Color::Foreground)),
        opacity: 1.,
    })
}
fn find_element_mut<'a>(element: &'a mut c::Element, key: &str) -> Option<&'a mut c::Element> {
    match element {
        c::Element::Named { name, content } => {
            if name == key {
                Some(content)
            } else {
                find_element_mut(content, key)
            }
        }
        c::Element::Box(b) => b
            .children
            .iter_mut()
            .find_map(|i| find_element_mut(&mut i.element, key)),
        c::Element::Stack(s) => s
            .children
            .iter_mut()
            .find_map(|i| find_element_mut(&mut i.element, key)),
        _ => None,
    }
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
