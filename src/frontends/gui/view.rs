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
fn column(children: Vec<c::Item>, height: Option<f32>) -> c::Element {
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
    Approval(ItemId, ApprovalDecision),
    Jump,
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
    layout: c::Layout,
}
struct Controls {
    key: String,
    size: [f32; 2],
    layout: c::Layout,
    rows_h: f32,
}
pub struct HarnessView {
    pub composer: composer::Controller,
    pub scroll: chat_scroll::State,
    metrics: Metrics,
    cached: Option<CacheKey>,
    controls: Option<Controls>,
    pub frame: Frame,
    pub notice: Option<String>,
}
impl Default for HarnessView {
    fn default() -> Self {
        let composer = composer::Controller::new(composer::Composer {
            focus: true,
            body_height: Some(64.),
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
        let draft = format!("{:?}{:?}{:?}", self.composer.view, self.notice, state.error);
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
            .cloned()
            .unwrap_or_else(|| {
                if state.replay {
                    "Offline replay".into()
                } else {
                    phase_text(&model.phase)
                }
            });
        let controls_key = format!("{draft}{status}{}", state.ready);
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
            let mut controls = vec![label(format!("bboxy450 harness · {status}")).into()];
            for (key, text) in [
                ("jump", "Latest"),
                ("interrupt", "Interrupt"),
                ("stop", "Stop"),
            ] {
                let button = Button {
                    label: text.into(),
                    kind: ButtonKind::Ghost,
                    state: ButtonState::Idle,
                    icon: None,
                    key: None,
                }
                .element(ctx)?;
                controls.push(named(key, button).into());
            }
            let header = c::Element::Box(c::Box::new(ui::Style::new(ui::Preset::Card), controls));
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

            let layout = root.layout(Rect::new(0., 0., width, size[1]), ctx, &self.metrics)?;
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
            restore_rows(&mut layout, &prepared.rows);
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
            ("stop", Action::Stop),
        ] {
            if let Some(node) = find(&layout, key) {
                frame.actions.push((node.rect, action));
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
    pub fn activate(&mut self, action: Action, bridge: &Bridge) {
        match action {
            Action::Jump => self.scroll.follow_latest(),
            Action::Send => {
                if let composer::Outcome::Submit(sent) =
                    self.composer.handle(composer::Command::Send)
                {
                    match bridge.send(Op::UserTurn {
                        blocks: vec![InputBlock::Text {
                            text: sent.text.clone(),
                        }],
                        mode: TurnMode::Queue,
                    }) {
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
                    Action::Approval(approval, decision) => {
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
    let mut end = text.len().min(4096);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    if end < text.len() {
        format!("{}\n… Full body retained in the session log.", &text[..end])
    } else if text.is_empty() {
        "…".into()
    } else {
        text
    }
}

// Adapted from bbt-ui components::translate_layout. Completed layouts are
// immutable; only their placement changes. ScrollView's fixed-height window
// elides offscreen slots before this walk, so only visible rows are restored.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
fn restore_rows(node: &mut c::Layout, rows: &im::HashMap<ItemId, Arc<CachedRow>>) {
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(id) = name
            .strip_prefix("cached/")
            .and_then(|s| s.parse().ok())
            .map(ItemId)
        && let Some(row) = rows.get(&id)
    {
        let mut solved = row.layout.clone();
        let dx = node.rect.x - solved.rect.x;
        let dy = node.rect.y - solved.rect.y;
        translate(&mut solved, dx, dy);
        *node = solved;
        return;
    }
    for child in &mut node.children {
        restore_rows(child, rows);
    }
}
fn translate(node: &mut c::Layout, dx: f32, dy: f32) {
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
    pub large_markdown_blocks: usize,
    pub prepared_on_ui: bool,
}
#[derive(Default)]
pub struct Preparer {
    metrics: Metrics,
    rows: im::HashMap<ItemId, Arc<CachedRow>>,
    scope: Option<(Option<SessionId>, Epoch)>,
    layout_builds: usize,
    terminal_outputs: std::collections::HashMap<ItemId, String>,
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

    pub(crate) fn scope(&mut self, model: &Transcript) {
        let scope = (model.session, model.epoch);
        if self.scope != Some(scope) {
            self.rows.clear();
            self.terminal_outputs.clear();
            self.scope = Some(scope);
        }
    }
    /// Caller must use the bounded runtime::blocking preparation pool.
    pub fn prepare(&mut self, source: Arc<Transcript>, width: f32) -> Result<PreparedRows, String> {
        let ctx = context();
        let model = &source;
        let width = width.max(240.);
        self.scope(model);
        let mut total_height = 0.;
        bbt_rt::assert_not_ui_thread();
        for (index, id) in model.order.iter().enumerate() {
            let row = &model.items[id];
            if row.completed.is_some()
                && let Some(cached) = self.rows.get(id)
                && cached.width == width
                && cached.completed == row.completed
            {
                let h = cached.layout.rect.h;
                total_height += h;
                continue;
            }
            let message = Message {
                from: kind_label(row.kind).into(),
                role: role(row.kind).into(),
                text: display_body(row, self.terminal_outputs.get(id)),
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
            let mut body = dock.items_at(ctx, index, &self.metrics);
            if let Item::Approval {
                options,
                decision: None,
                ..
            } = &row.body
                && row.status.is_none()
            {
                let mut buttons = vec![];
                for option in options {
                    let (text, _) = approval(*option);
                    let state = ButtonState::Idle;
                    let button = Button {
                        label: text.into(),
                        kind: ButtonKind::Ghost,
                        state,
                        icon: None,
                        key: None,
                    }
                    .element(ctx)?;
                    buttons.push(named(format!("approval/{id}/{text}"), button).into());
                }
                let b = c::Box::new(ui::Style::new(ui::Preset::Card), buttons);
                body.push(c::Element::Box(b).into());
            }
            let element = named(format!("item/{id}"), column(body, None));
            let layout = element.layout(Rect::new(0., 0., width, 0.), ctx, &self.metrics)?;
            let height = layout.rect.h;
            self.rows.insert(
                *id,
                Arc::new(CachedRow {
                    width,
                    completed: row.completed,
                    layout,
                }),
            );
            self.layout_builds += 1;
            total_height += height;
        }
        let large_markdown_blocks = source
            .items
            .values()
            .filter(
                |row| matches!(&row.body, Item::AgentMessage { markdown } if markdown.len() > 4096),
            )
            .count();
        Ok(PreparedRows {
            source,
            width,
            rows: self.rows.clone(),
            height: total_height,
            layout_builds: self.layout_builds,
            large_markdown_blocks,
            prepared_on_ui: bbt_rt::is_ui_thread(),
        })
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
