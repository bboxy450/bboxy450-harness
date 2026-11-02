// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Retained toolkit transcript and composer. Effects stay with the host bridge.
#![deny(clippy::disallowed_methods)]
use super::{
    bridge::{Bridge, FrameState},
    chat_scroll,
    model::{Transcript, ViewItem},
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
use std::{cell::RefCell, collections::HashMap, sync::Arc};
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
    Complete(ui::text_edit::Completion, String),
}
#[derive(Default)]
pub struct Frame {
    pub list: DrawList,
    pub layout: Option<c::Layout>,
    pub visible: Vec<(ItemId, ItemKind, Rect)>,
    pub actions: Vec<(Rect, Action)>,
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
    pub scroll: chat_scroll::State,
    metrics: Metrics,
    cached: Option<CacheKey>,
    controls: Option<Controls>,
    pub frame: Frame,
    pub notice: Option<String>,
}
impl Default for HarnessView {
    fn default() -> Self {
        let composer = super::editor::Input::new(composer::Composer {
            focus: true,
            body_height: Some(40.),
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
            scroll: Default::default(),
            metrics: Metrics::default(),
            cached: None,
            controls: None,
            frame: Frame::default(),
            notice: None,
        }
    }
}
impl HarnessView {
    pub fn invalidate(&mut self) {
        self.cached = None;
    }
    /// The same Element is used by a standalone surface and the offscreen driver.
    pub fn render(&mut self, state: &FrameState, size: [f32; 2]) -> Result<&Frame, String> {
        let offset = self.scroll.layout_offset();
        let draft = format!(
            "{:?}{:?}{:?}{:?}",
            self.composer.view, self.notice, state.error, state.notice
        );
        if self
            .cached
            .as_ref()
            .is_some_and(|(model, text, s, o, ready, prepared)| {
                Arc::ptr_eq(model, &state.transcript)
                    && text == &draft
                    && s == &size
                    && *o == offset
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
        let width = size[0].max(240.);
        let prepared = state
            .prepared
            .as_ref()
            .filter(|p| Arc::ptr_eq(&p.source, &state.transcript));
        let mut children = vec![];
        if let Some(prepared) = prepared {
            for id in &model.order {
                if let Some(row) = prepared.rows.get(id) {
                    let height = row.layout.rect.h;
                    children.push(
                        named(format!("cached/{id}"), c::Element::Spacer)
                            .fixed_height(height, id.to_string(), 0.)
                            .into(),
                    );
                }
            }
        }
        if children.is_empty() {
            children.push(
                label(if !state.ready {
                    "Loading session…"
                } else if model.order.is_empty() {
                    "Send a message to begin."
                } else {
                    "Preparing transcript…"
                })
                .into(),
            );
        }
        let transcript = if let Some(prepared) = prepared
            && !model.order.is_empty()
        {
            column(children, None).fixed_height(prepared.height, "transcript-content", 0.)
        } else {
            column(children, None)
        };
        let row_slots = timer.elapsed();
        let status = self
            .notice
            .as_ref()
            .or(state.error.as_ref())
            .or(state.notice.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                if state.replay {
                    "Offline replay".into()
                } else {
                    phase_text(&model.phase)
                }
            });
        let controls_key = format!(
            "{draft}{status}{}{:?}{:?}",
            state.ready,
            model.profile,
            model.meta.as_ref().map(|m| &m.cwd)
        );
        let (mut layout, rows_h) = if let Some(controls) = &self.controls
            && controls.key == controls_key
            && controls.size == size
        {
            let mut layout = controls.layout.clone();
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
            (layout, controls.rows_h)
        } else {
            let composer = self.composer.view.element(ctx, width, &self.metrics)?;
            let composer_h = composer
                .layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?
                .rect
                .h;
            let header_h = 40.;
            let rows_h = (size[1] - composer_h - header_h).max(1.);
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
                    model.profile.as_ref().map(|(p, m)| {
                        if p == m {
                            m.clone()
                        } else {
                            format!("{p} · {m}")
                        }
                    }),
                    model
                        .meta
                        .as_ref()
                        .and_then(|m| m.cwd.file_name())
                        .map(|n| n.to_string_lossy().into_owned()),
                    &status,
                    !matches!(model.phase, Phase::Idle | Phase::Closed) && !state.replay,
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
            (layout, rows_h)
        };
        let controls = timer.elapsed();
        let solved = timer.elapsed();
        if let Some(prepared) = prepared {
            let viewport = find(&layout, "transcript")
                .and_then(|n| n.children.first())
                .map(|n| n.rect)
                .ok_or("transcript viewport missing")?;
            restore_rows(&mut layout, &prepared.rows, viewport);
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
                    frame.visible.push((*id, model.items[id].kind, r));
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
                    let (text, decision) = approval(*option);
                    if let Some(node) = find(&layout, &format!("approval/{id}/{text}")) {
                        let r = intersection(node.rect, scroll_node.rect);
                        if r.w > 0. && r.h > 0. {
                            frame.actions.push((r, Action::Approval(*id, decision)));
                        }
                    }
                }
            }
        }
        for (key, action) in [
            ("jump", Action::Jump),
            ("interrupt", Action::Interrupt),
            ("stop", Action::StopAt(state.transcript.epoch)),
        ] {
            if let Some(node) = find(&layout, key) {
                frame.actions.push((node.rect, action));
            }
        }
        if let Some(popover) = &self.completion {
            let popup =
                popover
                    .element(ctx)
                    .layout(Rect::new(12., 40., 320., 0.), ctx, &self.metrics)?;
            for (i, choice) in popover.choices.options.iter().enumerate() {
                if let Some(node) = find(&popup, &format!("completion/{i}")) {
                    frame.actions.insert(
                        0,
                        (
                            node.rect,
                            Action::Complete(popover.request.clone(), choice.label.clone()),
                        ),
                    );
                }
            }
            for paint in c::paint(&popup, ctx) {
                append_paint(&mut frame.list, paint)?;
            }
        }
        frame.layout = Some(layout);
        self.scroll.layout_solved();
        self.frame = frame;
        self.cached = Some((
            state.transcript.clone(),
            draft,
            size,
            offset,
            state.ready,
            state.prepared.clone(),
        ));
        Ok(&self.frame)
    }
    pub fn edit(&mut self, command: ui::text_edit::Command) {
        self.composer.handle(composer::Command::Edit(command));
        self.completion = self.composer.popover();
        self.invalidate();
    }
    pub fn completion_open(&self) -> bool {
        self.completion.is_some()
    }
    pub fn complete(&mut self, key: ui::modules::approval::Key) -> bool {
        if self.completion.is_none() {
            self.completion = self.composer.popover();
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
        if self.completion.take().is_some() {
            self.invalidate();
            return true;
        }
        false
    }
    pub fn activate(&mut self, action: Action, bridge: &Bridge) {
        match action {
            Action::Jump => self.scroll.follow_latest(),
            Action::Toggle(item, toggle) => bridge.toggle(item, toggle),
            Action::Complete(request, text) => {
                let outcome = self.composer.handle(composer::Command::Edit(
                    ui::text_edit::Command::Complete { request, text },
                ));
                if outcome == composer::Outcome::Changed {
                    self.completion = None;
                } else {
                    self.completion = self.composer.popover();
                }
            }
            Action::Send => {
                if let composer::Outcome::Submit(sent) =
                    self.composer.handle(composer::Command::Send)
                {
                    let op = match sent.text.trim() {
                        "/interrupt" => Op::Interrupt { turn: None },
                        "/stop" => Op::Close,
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
        ItemKind::Unknown => "Unknown item",
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
        Phase::Unknown => "Status unavailable".into(),
    }
}
fn status_text(status: &ItemStatus) -> String {
    match status {
        ItemStatus::Completed => "Completed".into(),
        ItemStatus::Failed { reason } => format!("Failed: {reason}"),
        ItemStatus::Cancelled => "Cancelled".into(),
        ItemStatus::Interrupted => "Interrupted".into(),
        ItemStatus::Unknown => "Status unavailable".into(),
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
                InputBlock::Unknown => "Unsupported input".into(),
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
                    OutputBlock::Diff { path, unified } => format!("{}\n{unified}", path.display()),
                    OutputBlock::Error { text } => format!("Error: {text}"),
                    OutputBlock::Unknown => "Unsupported tool output".into(),
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
                    PlanStatus::Unknown => "Unknown",
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
        Item::Subagent { session } => format!("Session: {session}"),
        Item::Unknown => "Unsupported item".into(),
    };
    if text.is_empty() { "…".into() } else { text }
}

// Adapted from bbt-ui components::translate_layout. Completed layouts are
// immutable; only their placement changes. ScrollView's fixed-height window
// elides offscreen slots before this walk, so only visible rows are restored.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
fn restore_rows(node: &mut c::Layout, rows: &im::HashMap<ItemId, Arc<CachedRow>>, viewport: Rect) {
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(id) = name
            .strip_prefix("cached/")
            .and_then(|s| s.parse().ok())
            .map(ItemId)
        && let Some(row) = rows.get(&id)
    {
        let dx = node.rect.x - row.layout.rect.x;
        let dy = node.rect.y - row.layout.rect.y;
        if let Some(solved) = visible_layout(&row.layout, dx, dy, viewport) {
            *node = solved;
        }
        return;
    }
    for child in &mut node.children {
        restore_rows(child, rows, viewport);
    }
}
// Keep only glyphs/blocks intersecting the viewport. A giant paragraph remains
// fully scrollable without cloning/painting its entire cached solved layout.
fn visible_layout(node: &c::Layout, dx: f32, dy: f32, viewport: Rect) -> Option<c::Layout> {
    let rect = Rect::new(node.rect.x + dx, node.rect.y + dy, node.rect.w, node.rect.h);
    let visible = intersection(rect, viewport);
    if visible.w <= 0. || visible.h <= 0. {
        return None;
    }
    let kind = match &node.kind {
        c::LayoutKind::Text {
            runs,
            selectable,
            ellipsis,
            shadow,
        } => c::LayoutKind::Text {
            runs: runs
                .iter()
                .filter(|r| {
                    r.rect.bottom() + dy >= viewport.y && r.rect.y + dy <= viewport.bottom()
                })
                .map(|r| {
                    let mut r = r.clone();
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
    Some(c::Layout {
        rect,
        baseline: node.baseline,
        kind,
        children: node
            .children
            .iter()
            .filter_map(|child| visible_layout(child, dx, dy, viewport))
            .collect(),
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
    height: f32,
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
        let folds = Folds::of(model);
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
                from: kind_label(row.kind).into(),
                role: role(row.kind).into(),
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
                let (label, _) = approval(*kind);
                ui::modules::approval::OptionItem {
                    key: label.into(),
                    label: label.into(),
                    enabled: *kind != ApprovalDecisionKind::Unknown,
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
