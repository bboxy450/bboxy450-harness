// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Rows of the rich face. A resolved approval, a tool's terminal and its file change fold
//! into the tool call's card; each keeps its own `item/{id}` node inside that card, so
//! visibility and text witnesses still find every item. The card's cache signature
//! covers its folded children.
use super::super::{
    face as f,
    model::{Transcript, TurnState, ViewItem},
    rich::Face,
};
use super::{Preparer, approval_atom, approval_element};
use crate::protocol::*;
use bbt_ui::{self as ui, components as c, modules::seat_row::named};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub(super) struct Folds {
    /// child -> the tool call that shows it
    pub parent: HashMap<ItemId, ItemId>,
    kids: HashMap<ItemId, Vec<ItemId>>,
    /// tool -> its open approval, still a card of its own (it owns the buttons)
    asking: HashMap<ItemId, ItemId>,
    /// the last visible row of each turn carries the turn's receipt once it ends
    last: HashMap<ItemId, TurnId>,
    /// First visible item of each agent turn carries its origin heading.
    pub agent_heads: std::collections::HashSet<ItemId>,
    /// runaway loop: the repeated reply that opens the card -> everything folded after it
    loops: HashMap<ItemId, Vec<ItemId>>,
    /// turn -> replies folded into its loop (the receipt says so)
    folded: HashMap<TurnId, usize>,
    /// the transcript's last visible row carries .hx-feed's 18 px bottom padding
    tail: Option<ItemId>,
    /// the first visible row carries .hx-feed's 14 px top padding
    first: Option<ItemId>,
    /// parallel tool calls (harness.js `mount`): tool card -> (its index, the group size)
    pub group: HashMap<ItemId, (usize, usize)>,
}
/// Native records and usage never carry a receipt or feed padding (harness-spec.md
/// "What never reaches the transcript"). They remain in the replay, not the face.
pub(super) fn drawn(model: &Transcript, id: &ItemId) -> bool {
    model.items.get(id).is_some_and(|r| {
        !matches!(r.kind, ItemKind::Native | ItemKind::Usage)
            && (r.kind != ItemKind::UserMessage || r.completed.is_some())
    })
}
pub(super) use crate::decision::{QuestionField, question_fields};
/// A Question's answer in a few words, for its settled row.
fn answer_summary(answer: &serde_json::Value) -> String {
    match answer.get("action").and_then(|a| a.as_str()) {
        Some("accept") => {
            let values: Vec<String> = answer
                .get("content")
                .and_then(|c| c.as_object())
                .into_iter()
                .flatten()
                .filter_map(|(_, v)| match v {
                    serde_json::Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
                    serde_json::Value::Array(a) if !a.is_empty() => Some(
                        a.iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    serde_json::Value::Bool(b) => Some(b.to_string()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .collect();
            if values.is_empty() {
                "Answered".into()
            } else {
                values.join(" · ")
            }
        }
        Some("decline") => "Skipped".into(),
        _ => "Cancelled".into(),
    }
}
/// An MCP server's lifecycle notice, `mcp <server>: starting|ready`. A run of
/// them is startup chatter: it folds into one strip of server chips.
pub(super) fn mcp_lifecycle(text: &str) -> Option<(&str, &str)> {
    let (server, state) = text.strip_prefix("mcp ")?.split_once(": ")?;
    (!server.contains(' ') && matches!(state, "starting" | "ready")).then_some((server, state))
}
fn python_cell(row: &ViewItem) -> Option<crate::frontends::face_core::PythonCell> {
    match &row.body {
        Item::ToolCall {
            name,
            input_raw,
            output,
            raw_output,
            ..
        } => crate::frontends::face_core::python_cell(name, input_raw, output, raw_output.as_ref()),
        _ => None,
    }
}
/// harness-spec.md "Runaway loop card": first line, lowercased, letters only, 48 chars.
fn line_key(markdown: &str) -> String {
    let first = markdown.trim().lines().next().unwrap_or("").to_lowercase();
    let letters: String = first
        .chars()
        .filter(|c| c.is_ascii_lowercase() || *c == ' ')
        .collect();
    letters
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(48)
        .collect()
}
impl Folds {
    pub fn of(model: &Transcript, faces: &super::super::rich::Faces) -> Self {
        let mut f = Self::default();
        // A reply whose first line repeats an earlier reply of its turn opens a loop card;
        // later replies, thoughts, tool calls and notices of that turn fold into it.
        let mut said: HashMap<TurnId, std::collections::HashSet<String>> = HashMap::new();
        let mut anchor: HashMap<TurnId, ItemId> = HashMap::new();
        for id in &model.order {
            let r = &model.items[id];
            let Some(t) = r.turn else { continue };
            if let Some(a) = anchor.get(&t) {
                if matches!(
                    r.kind,
                    ItemKind::AgentMessage
                        | ItemKind::Reasoning
                        | ItemKind::ToolCall
                        | ItemKind::Notice
                ) {
                    f.loops.entry(*a).or_default().push(*id);
                    if r.kind == ItemKind::AgentMessage {
                        *f.folded.entry(t).or_default() += 1;
                    }
                }
                continue;
            }
            if let Item::AgentMessage { markdown } = &r.body
                && r.completed.is_some()
            {
                let key = line_key(markdown);
                let seen = said.entry(t).or_default();
                if !key.is_empty() && seen.contains(&key) {
                    anchor.insert(t, *id);
                    f.loops.insert(*id, vec![]);
                    f.folded.insert(t, 1);
                } else {
                    seen.insert(key);
                }
            }
        }
        for (a, members) in &f.loops {
            // open (its header toggled once) shows the folded rows in place again
            if !faces.get(a).is_some_and(|face| face.output_closed) {
                for m in members {
                    f.parent.insert(*m, *a);
                }
            }
        }
        let is_tool = |id: &ItemId| {
            model
                .items
                .get(id)
                .is_some_and(|t| t.kind == ItemKind::ToolCall)
        };
        let mut executions = HashMap::new();
        for id in &model.order {
            let r = &model.items[id];
            if f.parent.contains_key(id) {
                continue;
            }
            if let Some(cell) = python_cell(r)
                && let Some(execution) = cell.execution_id
            {
                if !cell.continued {
                    executions.insert(execution, *id);
                } else if cell.status.is_some()
                    && let Some(parent) = executions.get(&execution)
                {
                    f.parent.insert(*id, *parent);
                    f.kids.entry(*parent).or_default().push(*id);
                    continue;
                }
            }
            let to = match &r.body {
                Item::Approval {
                    subject, decision, ..
                } if is_tool(&subject.tool_call) => {
                    if decision.is_some() {
                        Some(subject.tool_call)
                    } else {
                        f.asking.insert(subject.tool_call, *id);
                        None
                    }
                }
                Item::Terminal { .. } | Item::FileChange { .. } => r.parent.filter(is_tool),
                _ => None,
            };
            if let Some(to) = to {
                f.parent.insert(*id, to);
                f.kids.entry(to).or_default().push(*id);
            }
        }
        // Consecutive MCP lifecycle notices fold into the first of their run.
        let mut strip: Option<ItemId> = None;
        for id in &model.order {
            if f.parent.contains_key(id) || !drawn(model, id) {
                continue;
            }
            match (&model.items[id].body, strip) {
                (Item::Notice { text, .. }, Some(head)) if mcp_lifecycle(text).is_some() => {
                    f.parent.insert(*id, head);
                    f.kids.entry(head).or_default().push(*id);
                }
                (Item::Notice { text, .. }, None) if mcp_lifecycle(text).is_some() => {
                    strip = Some(*id)
                }
                _ => strip = None,
            }
        }
        // harness.js mount(): a tool card mounted right after a tool card (or the group it
        // already sits in) joins its rail. Blocks are what the web draws: folded children and
        // native records and usage never mount, so they neither join nor break a run.
        // Any other row, a loop's members (they mount inside the loop card) and the turn's
        // receipt block end it.
        let in_loop: std::collections::HashSet<ItemId> =
            f.loops.values().flatten().copied().collect();
        let mut run: Vec<ItemId> = vec![];
        let mut run_turn = None;
        let flush = |run: &mut Vec<ItemId>, group: &mut HashMap<ItemId, (usize, usize)>| {
            let n = run.len();
            if n > 1 {
                for (i, id) in run.iter().enumerate() {
                    group.insert(*id, (i, n));
                }
            }
            run.clear();
        };
        for id in &model.order {
            if f.parent.contains_key(id) {
                continue;
            }
            let r = &model.items[id];
            if !drawn(model, id) {
                continue;
            }
            let tool = !in_loop.contains(id)
                && matches!(
                    r.kind,
                    ItemKind::ToolCall | ItemKind::Terminal | ItemKind::FileChange
                );
            if !(tool && (run.is_empty() || run_turn == r.turn)) {
                flush(&mut run, &mut f.group);
            }
            if tool {
                run.push(*id);
                run_turn = r.turn;
            }
        }
        flush(&mut run, &mut f.group);
        f.tail = model
            .order
            .iter()
            .rev()
            .find(|id| !f.parent.contains_key(id) && drawn(model, id))
            .copied();
        f.first = model
            .order
            .iter()
            .find(|id| !f.parent.contains_key(id) && drawn(model, id))
            .copied();
        let mut seen = std::collections::HashSet::new();
        for id in &model.order {
            let r = &model.items[id];
            if let Some(t) = r.turn
                && !f.parent.contains_key(id)
                && drawn(model, id)
                && seen.insert(t)
                && model.turn_origins.get(&t) == Some(&TurnOrigin::Agent)
            {
                f.agent_heads.insert(*id);
            }
        }
        seen.clear();
        for id in model.order.iter().rev() {
            let r = &model.items[id];
            if let Some(t) = r.turn
                && !f.parent.contains_key(id)
                && drawn(model, id)
                && seen.insert(t)
            {
                f.last.insert(*id, t);
            }
        }
        f
    }
    fn kid(&self, model: &Transcript, tool: &ItemId, kind: ItemKind) -> Option<ViewItem> {
        self.kids
            .get(tool)?
            .iter()
            .filter_map(|k| model.items.get(k))
            .find(|k| k.kind == kind)
            .cloned()
    }
    fn python(
        &self,
        model: &Transcript,
        row: &ViewItem,
    ) -> Option<crate::frontends::face_core::PythonCell> {
        let mut cell = python_cell(row)?;
        for child in self.kids.get(&row.id).into_iter().flatten() {
            if let Some(next) = model.items.get(child).and_then(python_cell) {
                cell.merge(next);
            }
        }
        Some(cell)
    }
    pub fn deps(&self, model: &Transcript, id: &ItemId, p: &Preparer) -> u64 {
        // rows without folded children or a receipt carry no signature (0)
        let mut any = false;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        if self.agent_heads.contains(id) {
            any = true;
            "agent continued".hash(&mut h);
        }
        if self.tail == Some(*id) {
            any = true;
            "tail".hash(&mut h);
        }
        if let Some(g) = self.group.get(id) {
            any = true;
            g.hash(&mut h);
        }
        if let Some(t) = self.last.get(id) {
            any = true;
            format!("{:?}", model.turns.get(t)).hash(&mut h);
        }
        for k in self
            .kids
            .get(id)
            .into_iter()
            .flatten()
            .chain(self.asking.get(id))
            .chain(self.loops.get(id).into_iter().flatten())
        {
            if let Some(r) = model.items.get(k) {
                any = true;
                (
                    k,
                    r.completed,
                    format!("{:?}", r.status),
                    r.terminal_tail.len(),
                    p.has_terminal_output(k),
                )
                    .hash(&mut h);
                if let Some(cell) = python_cell(r) {
                    format!("{cell:?}").hash(&mut h);
                }
            }
        }
        if any { h.finish() } else { 0 }
    }
}

impl Preparer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn face_row(
        &mut self,
        model: &Transcript,
        row: &ViewItem,
        face: &Face,
        folds: &Folds,
        index: usize,
        width: f32,
        gutter: f32,
        content_width: f32,
    ) -> Result<(c::Element, Option<super::super::rich::Body>), String> {
        let ctx = self.context();
        let ink = f::ink(ctx);
        let id = row.id;
        let cwd = model
            .meta
            .as_ref()
            .map(|m| m.cwd.display().to_string())
            .unwrap_or_default();
        let mut rich = None;
        // Raw edge records never draw, including while incomplete. Like usage, they
        // keep their id at zero height, with no body, gutter, padding or receipt.
        if !drawn(model, &id) {
            return Ok((c::Element::Spacer, None));
        }
        // a parallel-call group member is inset 14px behind the rail (.hx-group padding-left)
        let group = folds.group.get(&id).copied();
        let inset = if group.is_some() { 14. } else { 0. };
        let body: c::Element = match &row.body {
            Item::UserMessage { blocks } => {
                let text = blocks
                    .iter()
                    .map(|b| match b {
                        InputBlock::Text { text } => text.clone(),
                        InputBlock::Image { blob } => {
                            format!("Image: {} · {} bytes", blob.mime, blob.len)
                        }
                        InputBlock::Mention { path } => format!("@{}", path.display()),
                        InputBlock::Unknown(raw) => {
                            format!("Unsupported input: {}", unknown_tag(raw))
                        }
                        other => other.display_text(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let doc = self.documents.prepare(
                    id,
                    if text.is_empty() { "…" } else { &text },
                    true,
                    (content_width - 26.).max(1.),
                    self.zoom.scale(),
                    &self.metrics,
                )?;
                let (content, toggle) =
                    f::prompt_content(doc.element.clone(), id, face.source, &text, &ink);
                let bubble = f::prompt_bubble(content, "YOU", Some(toggle), false, &ink);
                if !face.source {
                    rich = Some(doc);
                }
                bubble
            }
            Item::AgentMessage { markdown } if folds.loops.contains_key(&id) => {
                let members = &folds.loops[&id];
                let count = |k: ItemKind| {
                    std::iter::once(&id)
                        .chain(members)
                        .filter(|m| model.items.get(m).is_some_and(|r| r.kind == k))
                        .count()
                };
                let bits: Vec<String> = [
                    (count(ItemKind::AgentMessage), "reply", "replies"),
                    (count(ItemKind::Reasoning), "thought", "thoughts"),
                    (count(ItemKind::ToolCall), "tool call", "tool calls"),
                ]
                .into_iter()
                .filter(|(n, ..)| *n > 0)
                .map(|(n, one, many)| format!("{n} {}", if n == 1 { one } else { many }))
                .collect();
                let open = face.output_closed;
                let mut kids: Vec<c::Item> = vec![
                    f::loop_head(&format!("toolhead/{id}"), &bits.join(" · "), open, &ink).into(),
                ];
                if open {
                    let doc = self.documents.prepare(
                        id,
                        markdown,
                        true,
                        content_width - 36.,
                        self.zoom.scale(),
                        &self.metrics,
                    )?;
                    kids.push(
                        named(
                            format!("loopbody/{id}"),
                            f::loop_body(doc.element.clone(), &ink),
                        )
                        .into(),
                    );
                    rich = Some(doc);
                } else {
                    // folded rows stay addressable while the card is closed
                    for m in members {
                        kids.push(named(format!("item/{m}"), c::Element::Spacer).into());
                    }
                }
                named(format!("loop/{id}"), f::loop_card(kids, &ink))
            }
            Item::AgentMessage { markdown } => {
                let doc = self.documents.prepare(
                    id,
                    markdown,
                    row.completed.is_some(),
                    content_width,
                    self.zoom.scale(),
                    &self.metrics,
                )?;
                // `stream/{id}`: the text a chunk fade counts glyphs in (stream_motion)
                let base = named(
                    format!("stream/{id}"),
                    if face.source {
                        let mut t = f::text(markdown.clone(), 12., 400, ink.fg, true);
                        t.preserve_whitespace = true;
                        c::Element::Text(t)
                    } else {
                        doc.element.clone()
                    },
                );
                // Hover-only (harness-spec.md "Refinement pass 2"): the view hides it at
                // rest and fades it to .6 on row hover, 1 on its own, so it is drawn in full
                // `dim` here.
                let toggle = named(
                    format!("source/{id}/toggle"),
                    f::line(
                        if face.source { "formatted" } else { "source" },
                        10.5,
                        500,
                        ink.dim,
                        true,
                    ),
                );
                // The web reference has no source line: the toggle sits at the right end
                // of the message's last line when that line leaves room, else on its own.
                let ([tw, th], tb) = toggle
                    .natural(f32::INFINITY, ctx, &self.metrics)
                    .map_err(|e| format!("{e:?}"))?;
                let seat = doc
                    .tail
                    .filter(|_| !face.source)
                    .filter(|[right, _]| right + tw + 16. <= content_width);
                let content = match (row.completed.is_some(), seat) {
                    (false, _) => base,
                    (true, Some([_, above])) => c::Element::Layers {
                        base: Box::new(base),
                        overlays: vec![c::Placed {
                            content: toggle,
                            left: content_width - tw,
                            bottom: (above - (th - tb)).max(0.),
                        }],
                    },
                    (true, None) => f::vstack(
                        vec![
                            base.into(),
                            f::hstack(
                                vec![
                                    c::Item {
                                        element: c::Element::Spacer,
                                        extent: c::Extent::Fill,
                                    },
                                    f::fixed(toggle),
                                ],
                                0.,
                            )
                            .into(),
                        ],
                        2.,
                    ),
                };
                rich = Some(doc);
                content
            }
            Item::Reasoning { summary, raw, .. } => {
                let summary = if summary.trim().is_empty() {
                    raw.as_deref().unwrap_or(summary)
                } else {
                    summary
                }
                .trim();
                let live = row.status.is_none();
                let open = live || face.reasoning_open;
                let gist = gist(summary);
                let mut kids: Vec<c::Item> = vec![
                    f::thought_row(
                        &format!("reasoning/{id}/toggle"),
                        live,
                        row.completed_ms.map(|c| c.saturating_sub(row.started_ms)),
                        // the gist stays beside the label open or closed, as .hx-gist does
                        &gist,
                        open,
                        &ink,
                    )
                    .into(),
                ];
                // Live and completed reasoning share the streaming Markdown preparer.
                let doc = self.documents.reasoning(
                    id,
                    summary,
                    !live,
                    content_width - 20.,
                    self.zoom.scale(),
                    &self.metrics,
                )?;
                if open {
                    kids.push(
                        named(
                            format!("reasoning/{id}/body"),
                            f::thought_body(doc.element.clone(), &ink),
                        )
                        .into(),
                    );
                    rich = Some(doc);
                }
                f::vstack(kids, 4.)
            }
            Item::ToolCall { .. } => {
                let (card, doc) =
                    self.tool_card(model, row, face, folds, content_width - inset, &ink)?;
                rich = doc;
                card
            }
            Item::Approval {
                subject,
                options,
                decision,
                by,
            } => match decision {
                None => {
                    let mut prompt = approval_atom(id, subject, options);
                    for _ in 0..face.approval_selected {
                        prompt.handle(ui::modules::approval::Key::Down);
                    }
                    let effect = format!("{:?}", subject.effect).to_lowercase();
                    f::card(
                        &f::Status::Asking,
                        vec![
                            f::frame(
                                f::Frame {
                                    padding: [9., 12., 0., 12.],
                                    gap: 8.,
                                    row: true,
                                    ..Default::default()
                                },
                                vec![
                                    f::fixed(f::icon("shield", ink.ask, 15., 1.6)),
                                    f::fixed(f::runs(
                                        vec![
                                            ("Allow ".into(), ink.fg, 400, false),
                                            (effect, ink.ask, 650, false),
                                            ("?".into(), ink.fg, 400, false),
                                        ],
                                        13.,
                                        false,
                                        false,
                                    )),
                                    f::fill(f::line(
                                        subject
                                            .command
                                            .clone()
                                            .unwrap_or_else(|| subject.summary.clone()),
                                        12.,
                                        400,
                                        ink.dim,
                                        true,
                                    )),
                                ],
                            )
                            .into(),
                            f::frame(
                                f::Frame {
                                    padding: [4., 12., 10., 12.],
                                    ..Default::default()
                                },
                                vec![approval_element(&prompt, ctx).into()],
                            )
                            .into(),
                        ],
                        &ink,
                    )
                }
                Some(d) => f::hstack(
                    vec![
                        f::fixed(
                            f::badge(Some(d), by.as_ref(), &ink).unwrap_or(c::Element::Spacer),
                        ),
                        f::fill(f::line(subject.summary.clone(), 12., 400, ink.dim, true)),
                    ],
                    8.,
                ),
            },
            Item::Question {
                message,
                schema,
                answer,
                by,
            } => match answer {
                None => {
                    let fields = question_fields(schema);
                    let selects: Vec<_> = fields.iter().filter(|f| !f.options.is_empty()).collect();
                    let free = fields.iter().any(|f| f.options.is_empty());
                    let mut kids: Vec<c::Item> = vec![
                        f::frame(
                            f::Frame {
                                padding: [9., 12., 0., 12.],
                                gap: 8.,
                                row: true,
                                ..Default::default()
                            },
                            vec![
                                f::fixed(f::icon("shield", ink.ask, 15., 1.6)),
                                f::fill(c::Element::Text(f::text(
                                    message.as_str(),
                                    13.,
                                    600,
                                    ink.fg,
                                    false,
                                ))),
                            ],
                        )
                        .into(),
                    ];
                    let pill = |key: String, label: &str, on: bool| {
                        let (fg, bg) = if on {
                            (ink.bg, ink.ask)
                        } else {
                            (ink.fg, f::mix(ink.surface, ink.ask, 0.14))
                        };
                        c::Element::Named {
                            name: key,
                            content: Box::new(f::pill(label, fg, bg, 12.)),
                        }
                    };
                    for field in &selects {
                        let mut body = vec![];
                        if let Some(caption) = field
                            .description
                            .as_ref()
                            .or(field.title.as_ref())
                            .filter(|_| selects.len() > 1)
                        {
                            body.push(f::fixed(f::line(caption.clone(), 12., 500, ink.dim, false)));
                        }
                        let pills = field
                            .options
                            .iter()
                            .enumerate()
                            .map(|(i, (value, label))| {
                                let on = face.picked.contains(&(field.key.clone(), value.clone()));
                                f::fixed(pill(
                                    format!("question/{id}/{}/{i}", field.key),
                                    label,
                                    on,
                                ))
                            })
                            .collect();
                        let mut flow = c::Flow::new(pills);
                        flow.row_gap = ui::Length::Px(6.);
                        flow.column_gap = ui::Length::Px(6.);
                        body.push(f::fixed(c::Element::Flow(flow)));
                        kids.push(
                            f::frame(
                                f::Frame {
                                    padding: [6., 12., 0., 35.],
                                    gap: 4.,
                                    ..Default::default()
                                },
                                body,
                            )
                            .into(),
                        );
                    }
                    let mut actions = vec![];
                    if selects.len() > 1 || selects.iter().any(|f| f.multi) {
                        actions.push(f::fixed(pill(
                            format!("question/{id}/submit"),
                            "Submit",
                            true,
                        )));
                    }
                    actions.push(f::fixed(pill(format!("question/{id}/skip"), "Skip", false)));
                    if free {
                        actions.push(f::fill(f::line(
                            "or type your answer below".to_owned(),
                            12.,
                            400,
                            ink.dim,
                            false,
                        )));
                    }
                    kids.push(
                        f::frame(
                            f::Frame {
                                padding: [8., 12., 10., 35.],
                                gap: 8.,
                                row: true,
                                ..Default::default()
                            },
                            actions,
                        )
                        .into(),
                    );
                    f::card(&f::Status::Asking, kids, &ink)
                }
                Some(answer) => f::hstack(
                    vec![
                        f::fixed(f::icon("shield", ink.dim, 13., 1.4)),
                        f::fixed(f::line(message.clone(), 12., 400, ink.dim, false)),
                        f::fill(f::line(
                            match by {
                                // Automation answered: say so, with the model's confidence.
                                Some(Resolver::Model { confidence, .. }) => format!(
                                    "{} · answered by automation ({confidence:.2})",
                                    answer_summary(answer)
                                ),
                                _ => answer_summary(answer),
                            },
                            12.,
                            600,
                            ink.fg,
                            false,
                        )),
                    ],
                    8.,
                ),
            },
            Item::Notice { text, .. } if mcp_lifecycle(text).is_some() => {
                // one chip per server, its latest state; restarts counted
                let mut servers: Vec<(String, String, usize)> = vec![];
                let kids = folds.kids.get(&id).into_iter().flatten();
                for r in std::iter::once(row).chain(kids.filter_map(|k| model.items.get(k))) {
                    let Item::Notice { text, .. } = &r.body else {
                        continue;
                    };
                    let Some((server, state)) = mcp_lifecycle(text) else {
                        continue;
                    };
                    let at = match servers.iter().position(|s| s.0 == server) {
                        Some(at) => at,
                        None => {
                            servers.push((server.into(), String::new(), 0));
                            servers.len() - 1
                        }
                    };
                    servers[at].1 = state.into();
                    servers[at].2 += usize::from(state == "starting");
                }
                f::mcp_strip(&servers, &ink)
            }
            Item::Notice { text, level } => f::notice(text, level, &ink),
            Item::Terminal { command, exit, .. } => {
                let status = term_status(row, *exit);
                let out = self.terminal_text(row, "");
                let panel_width = (content_width - inset - 48.).max(1.);
                let mut body = vec![];
                body.extend(command_panels(
                    &mut self.documents,
                    command,
                    id,
                    panel_width,
                    face,
                    &ink,
                    &self.metrics,
                )?);
                body.push(
                    named(
                        format!("term/{id}"),
                        code_panel(
                            f::term_output_full(
                                out.as_deref().unwrap_or(if row.status.is_some() {
                                    "no output"
                                } else {
                                    ""
                                }),
                                &ink,
                            ),
                            &format!("codeout/{id}/output"),
                            panel_width,
                            320.,
                            &self.metrics,
                            ctx,
                        )?,
                    )
                    .into(),
                );
                f::card(
                    &status,
                    vec![
                        f::head(
                            f::Head {
                                key: format!("toolhead/{id}"),
                                tool_kind: "execute",
                                verb: if super::super::rich::python_snippet(command).is_some() {
                                    "Run Python"
                                } else {
                                    "Run command"
                                }
                                .into(),
                                target: f::compact_heading(
                                    super::super::rich::python_snippet(command)
                                        .as_ref()
                                        .map_or(command.as_str(), |p| p.source),
                                    80,
                                ),
                                mono: true,
                                place: None,
                                stat: None,
                                // `exit N` only when N ≠ 0 (harness-spec.md "Tight transcript")
                                meta: exit
                                    .filter(|e| *e != 0)
                                    .map(|e| (format!("exit {e}"), Some(false))),
                                badge: None,
                                dur: None,
                                status: term_status(row, *exit),
                            },
                            &ink,
                        )
                        .into(),
                        named(format!("toolbody/{id}"), f::body_el(body)).into(),
                    ],
                    &ink,
                )
            }
            Item::FileChange { path, unified, .. } => {
                let p = path.display().to_string();
                let p = p.strip_prefix(&format!("{cwd}/")).unwrap_or(&p).to_owned();
                f::card(
                    &f::Status::Done,
                    vec![
                        f::head(
                            f::Head {
                                key: format!("toolhead/{id}"),
                                tool_kind: "edit",
                                verb: "Changed".into(),
                                target: p,
                                mono: false,
                                place: None,
                                stat: Some(f::diff_stat(unified)),
                                meta: None,
                                badge: None,
                                dur: None,
                                status: f::Status::Done,
                            },
                            &ink,
                        )
                        .into(),
                        named(
                            format!("toolbody/{id}"),
                            f::body_el(vec![
                                f::diff_view(&format!("diff/{id}"), unified, &ink).into(),
                            ]),
                        )
                        .into(),
                    ],
                    &ink,
                )
            }
            Item::TurnDiff { unified } => f::notice(unified, &Level::Info, &ink),
            Item::ContextTrim { clipped_through } => {
                let doc = self.documents.prepare(
                    id,
                    &format!(
                        "### Context trimmed through event {}\n\nTool output limited to **2 KiB**; reasoning and images omitted from model context.",
                        clipped_through.0
                    ),
                    row.completed.is_some(),
                    content_width,
                    self.zoom.scale(),
                    &self.metrics,
                )?;
                let element = doc.element.clone();
                rich = Some(doc);
                element
            }
            Item::Compaction { .. } => f::vstack(
                vec![
                    f::hrule(1., f::alpha(ink.fg, 0.08)).into(),
                    f::line("Context compacted", 11.5, 400, ink.dim, false).into(),
                ],
                6.,
            ),
            Item::Command { name, .. } => f::notice(&format!("Ran /{name}"), &Level::Info, &ink),
            Item::Hook { name, .. } => f::notice(&format!("Hook {name}"), &Level::Info, &ink),
            Item::SystemMessage { text, .. } => f::notice(text, &Level::Info, &ink),
            Item::Plan { entries } => {
                let text = entries
                    .iter()
                    .map(|entry| {
                        let status = match entry.status {
                            PlanStatus::Pending => "Pending",
                            PlanStatus::InProgress => "In progress",
                            PlanStatus::Completed => "Completed",
                            PlanStatus::Cancelled => "Cancelled",
                            PlanStatus::Unknown(_) => "Unknown",
                        };
                        format!("{status}: {}", entry.text)
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                f::notice(&text, &Level::Info, &ink)
            }
            Item::Image { mime, blob } => f::notice(
                &format!("Image: {mime} · {} bytes", blob.len),
                &Level::Info,
                &ink,
            ),
            Item::Subagent { session, .. } => {
                f::notice(&format!("Session: {session}"), &Level::Info, &ink)
            }
            Item::MemoryWrite { .. }
            | Item::MemoryState { .. }
            | Item::Trial { .. }
            | Item::Adoption { .. }
            | Item::Recall { .. }
            | Item::Receipt { .. } => f::notice(
                &row.body.extension_text().unwrap_or_default(),
                &Level::Info,
                &ink,
            ),
            Item::Unknown(raw) => f::notice(
                &format!("Unsupported item: {}", unknown_tag(raw)),
                &Level::Info,
                &ink,
            ),
            Item::Native { .. } | Item::Usage { .. } => {
                return Ok((c::Element::Spacer, None));
            }
        };
        let body = match group {
            Some((i, n)) => {
                let member = f::group_member(body, i, n, &ink);
                if i == 0 {
                    f::vstack(
                        vec![
                            f::group_caption(&format!("group/{id}/caption"), n, &ink).into(),
                            member.into(),
                        ],
                        0.,
                    )
                } else {
                    member
                }
            }
            None => body,
        };
        let mut content: Vec<c::Item> = vec![];
        if folds.agent_heads.contains(&id) {
            content.push(f::notice("agent continued", &Level::Info, &ink).into());
        }
        content.push(body.into());
        if let Some(turn) = folds.last.get(&id)
            && let Some(TurnState::Ended { reason, usage }) = model.turns.get(turn)
        {
            let items: Vec<&ViewItem> = model
                .order
                .iter()
                .map(|i| &model.items[i])
                .filter(|r| r.turn == Some(*turn) && drawn(model, &r.id))
                .collect();
            let count = |k: ItemKind| items.iter().filter(|r| r.kind == k).count();
            let start = items.iter().map(|r| r.started_ms).min();
            let end = items.iter().filter_map(|r| r.completed_ms).max();
            content.push(
                named(
                    format!("turn/{}", turn.0),
                    f::receipt(
                        f::Receipt {
                            ok: matches!(reason, StopReason::EndTurn | StopReason::Quiet { .. }),
                            soft: matches!(reason, StopReason::Interrupted | StopReason::Cancelled),
                            reason: stop_title(reason).into(),
                            why: match reason {
                                StopReason::Error { message, .. } => Some(message.clone()),
                                _ => None,
                            },
                            took: start.zip(end).map(|(a, b)| b.saturating_sub(a)),
                            tools: count(ItemKind::ToolCall),
                            edits: count(ItemKind::FileChange),
                            tokens: usage.as_ref().map(|u| (u.input_tokens, u.output_tokens)),
                            folded: folds.folded.get(turn).copied().unwrap_or(0),
                            key: format!("turn/{}", turn.0),
                        },
                        &ink,
                    ),
                )
                .into(),
            );
        }
        // Row region: symmetric gutters (CardGuttersSymmetric) and a 4 px inset body.
        let body = named(format!("body/{id}"), f::vstack(content, 10.));
        let region = named(
            format!("e{index}"),
            f::frame(
                f::Frame {
                    padding: [0., 4., 0., 4.],
                    ..Default::default()
                },
                vec![body.into()],
            ),
        );
        // Compact transcript: 3 + 3 px between rows.
        let gap = 3.;
        // .hx-feed's 14 px top padding sits on the first drawn row
        let top = if folds.first == Some(id) { 14. } else { gap };
        let bottom = if folds.tail == Some(id) { 18. } else { gap };
        // .hx-group: margin-top 12 above the caption's 15px box; the 6px member gap is inside
        // the rail column, so the members themselves touch
        let (top, bottom) = match group {
            Some((0, _)) => (top + 12. - 15., 0.),
            Some((i, n)) if i + 1 == n => (0., bottom),
            Some(_) => (0., 0.),
            None => (top, bottom),
        };
        let _ = width;
        Ok((
            f::frame(
                f::Frame {
                    padding: [top, gutter, bottom, gutter],
                    ..Default::default()
                },
                vec![region.into()],
            ),
            rich,
        ))
    }

    fn terminal_text(&self, term: &ViewItem, fallback: &str) -> Option<String> {
        self.terminal_outputs
            .get(&term.id)
            .cloned()
            .or_else(|| (!term.terminal_tail.is_empty()).then(|| term.terminal_tail.clone()))
            .or_else(|| (!fallback.is_empty()).then(|| fallback.to_owned()))
    }

    fn tool_card(
        &mut self,
        model: &Transcript,
        row: &ViewItem,
        face: &Face,
        folds: &Folds,
        content_width: f32,
        ink: &f::Ink,
    ) -> Result<(c::Element, Option<super::super::rich::Body>), String> {
        let ctx = self.context();
        let cwd = model
            .meta
            .as_ref()
            .map(|m| m.cwd.display().to_string())
            .unwrap_or_default();
        let cwd = cwd.as_str();
        let Item::ToolCall {
            name,
            tool_kind,
            input_raw,
            output,
            title,
            server,
            raw_output,
            ..
        } = &row.body
        else {
            return Err("not a tool call".into());
        };
        let id = row.id;
        let python = folds.python(model, row);
        let d = f::describe(name, tool_kind.clone(), input_raw, output, cwd);
        let approval = folds.kid(model, &id, ItemKind::Approval);
        let term = folds.kid(model, &id, ItemKind::Terminal);
        let change = folds.kid(model, &id, ItemKind::FileChange);
        let (decision, by) = match approval.as_ref().map(|a| &a.body) {
            Some(Item::Approval { decision, by, .. }) => (decision.clone(), by.clone()),
            _ => (None, None),
        };
        let denied = matches!(decision, Some(ApprovalDecision::Deny { .. }));
        let mut status = match &row.status {
            None if folds.asking.contains_key(&id) => f::Status::Asking,
            None => f::Status::Running,
            Some(_) if denied => f::Status::Denied,
            Some(ItemStatus::Completed) => f::Status::Done,
            Some(ItemStatus::Cancelled | ItemStatus::Interrupted) => f::Status::Stopped,
            Some(_) => f::Status::Failed,
        };
        // the terminal's own command is authoritative over the call's arguments
        let mut d = d;
        if crate::frontends::face_core::placeholder_output(output)
            && let Some(value) = raw_output.as_ref().filter(|v| !v.is_null())
        {
            d.out = crate::frontends::face_core::raw_tool_output(value);
        }
        if !row.terminal_tail.is_empty() {
            d.out = row.terminal_tail.clone();
        }
        if let Some(Item::Terminal { command, .. }) = term.as_ref().map(|t| &t.body)
            && !command.is_empty()
        {
            d.target = command.clone();
            d.mono = true;
        }
        if let Some(cell) = &python {
            d.out = cell.text();
            status = match cell.status.as_deref() {
                Some("starting" | "running" | "cancelling") => f::Status::Running,
                Some("completed") => f::Status::Done,
                Some("cancelled" | "interrupted") => f::Status::Stopped,
                Some(_) => f::Status::Failed,
                None => status,
            };
        }
        let original_command = if let Some(cell) = &python {
            cell.source.clone()
        } else if d.kind == "execute" {
            Some(if !d.target.is_empty() {
                d.target.clone()
            } else if name.chars().any(char::is_whitespace) {
                name.clone()
            } else {
                title
                    .as_ref()
                    .filter(|t| !t.is_empty())
                    .unwrap_or(name)
                    .clone()
            })
        } else if super::super::rich::python_snippet(name).is_some() {
            Some(name.clone())
        } else {
            title
                .as_ref()
                .filter(|t| super::super::rich::python_snippet(t).is_some())
                .cloned()
        };
        if let Some(cell) = &python {
            d.kind = "execute";
            d.mono = true;
            d.verb = if cell.continued {
                "Continue Python"
            } else {
                "Run Python"
            }
            .into();
            // The header names what the run does, not its import boilerplate.
            d.target = f::compact_heading(
                cell.source
                    .as_deref()
                    .map(crate::frontends::face_core::PythonCell::headline)
                    .filter(|line| !line.is_empty())
                    .or(cell.execution_id.as_deref())
                    .unwrap_or("Python"),
                80,
            );
        } else if let Some(command) = &original_command {
            let python = super::super::rich::python_snippet(command);
            d.kind = "execute";
            d.mono = true;
            d.verb = if python.is_some() {
                "Run Python"
            } else {
                "Run command"
            }
            .into();
            d.target =
                f::compact_heading(python.as_ref().map_or(command.as_str(), |p| p.source), 80);
            // `cd DIR && python3 …` reads as Run Python in DIR (its last component).
            if let Some(cwd) = python.as_ref().and_then(|p| p.cwd) {
                let leaf = cwd.trim_end_matches('/').rsplit('/').next().unwrap_or(cwd);
                d.place = Some(if leaf.is_empty() { cwd } else { leaf }.to_owned());
            }
        }
        let exit = term.as_ref().and_then(|t| match &t.body {
            Item::Terminal { exit, .. } => *exit,
            _ => None,
        });
        let unified = change
            .as_ref()
            .and_then(|c| match &c.body {
                Item::FileChange { unified, .. } => Some(unified.clone()),
                _ => None,
            })
            .or_else(|| (d.kind == "edit" && d.out.starts_with("---")).then(|| d.out.clone()));
        let lines = d.out.lines().filter(|l| !l.is_empty()).count();
        let meta = if python.is_some() {
            None
        } else {
            match d.kind {
                // `exit N` only when N ≠ 0: success says nothing, the check mark already did
                "execute" => exit
                    .filter(|e| *e != 0)
                    .map(|e| (format!("exit {e}"), Some(false))),
                "read" if row.status.is_some() && lines > 0 => {
                    Some((format!("{lines} lines"), None))
                }
                "search" if row.status.is_some() && lines > 0 => Some((
                    format!("{lines} hit{}", if lines == 1 { "" } else { "s" }),
                    None,
                )),
                _ if raw_output.is_some() && row.status.is_some() && lines > 0 => {
                    Some((format!("{lines} lines"), None))
                }
                _ => None,
            }
        };
        let failed = matches!(status, f::Status::Failed | f::Status::Denied);
        // the terminal's output, else the call's: what an execute body shows
        let out_text = match &term {
            Some(t) => self.terminal_text(t, &d.out),
            None => (!d.out.is_empty()).then(|| d.out.clone()),
        };
        let out_lines = out_text
            .as_deref()
            .map_or(0, |o| o.lines().filter(|l| !l.is_empty()).count());
        let running = matches!(status, f::Status::Running);
        // Python opens when finished and small: its structured rows, not raw JSON lines.
        let python_small = python.as_ref().is_some_and(|cell| {
            let rows: usize = python_emits(cell)
                .iter()
                .map(|v| f::emit_row_count(v, ink))
                .sum();
            let out = cell.output.values().flat_map(|t| t.lines()).count();
            !running && rows + out <= 8
        });
        // Running calls keep the compact header; finished edits and small results open.
        let auto = failed
            || python_small
            || matches!(status, f::Status::Asking)
            || (row.completed.is_some()
                && python.is_none()
                && ((d.kind == "execute" && out_lines <= 6)
                    || d.kind == "edit"
                    || (d.verb == "Search" && lines <= 6)));
        let open = auto != face.output_closed;
        let badge = f::badge(decision.as_ref(), by.as_ref(), ink).map(|b| match &approval {
            Some(a) => named(format!("item/{}", a.id), b),
            None => b,
        });
        // Python's state is a chip beside any approval badge.
        let badge = match &python {
            Some(cell) => {
                let label = cell.status.clone().unwrap_or_else(|| {
                    if cell.error.is_some() {
                        "failed"
                    } else {
                        "pending"
                    }
                    .into()
                });
                let chip = f::python_status(&label, cell.exit_code, ink)
                    .map(|chip| named(format!("python/{id}/status"), chip));
                match (chip, badge) {
                    (Some(chip), Some(b)) => Some(f::hstack(vec![f::fixed(chip), f::fixed(b)], 6.)),
                    (chip, b) => chip.or(b),
                }
            }
            None => badge,
        };
        // A Python run lasts from its call to its last folded poll.
        let finished = match &python {
            Some(_) if running => None,
            Some(_) => folds
                .kids
                .get(&id)
                .into_iter()
                .flatten()
                .filter_map(|k| model.items.get(k).and_then(|r| r.completed_ms))
                .chain(row.completed_ms)
                .max(),
            None => row.completed_ms,
        };
        let mut kids: Vec<c::Item> = vec![
            f::head(
                f::Head {
                    key: format!("toolhead/{id}"),
                    tool_kind: d.kind,
                    verb: d.verb.clone(),
                    target: d.target.clone(),
                    mono: d.mono,
                    place: d.place.clone(),
                    stat: unified.as_deref().map(f::diff_stat),
                    meta,
                    badge,
                    // the duration only from 1s (harness-spec.md "Tight transcript")
                    dur: finished
                        .map(|c| c.saturating_sub(row.started_ms))
                        .filter(|ms| *ms >= 1000)
                        .map(f::duration),
                    status,
                },
                ink,
            )
            .into(),
        ];
        if python.is_some() {
            // Continuations share the parent card but preserve their addressable
            // item nodes for visibility and replay witnesses.
            for child in folds.kids.get(&id).into_iter().flatten() {
                if model.items.get(child).and_then(python_cell).is_some() {
                    kids.push(named(format!("item/{child}"), c::Element::Spacer).into());
                }
            }
        }
        let mut inside: Vec<c::Item> = vec![];
        let mut rich = None;
        let caption = [
            title.as_deref().filter(|t| {
                !t.contains('\n')
                    && t.chars().count() <= 120
                    && Some(*t) != original_command.as_deref()
                    && *t != name
                    && *t != d.verb
            }),
            server.as_deref().filter(|_| python.is_none()),
        ]
        .into_iter()
        .flatten()
        .map(|s| f::compact_heading(s, 64))
        .collect::<Vec<_>>()
        .join(" · ");
        if open && !caption.is_empty() {
            inside
                .push(f::line(f::compact_heading(&caption, 80), 11.5, 400, ink.dim, false).into());
        }
        let wrap = |kid: &Option<ViewItem>, e: c::Element| match kid {
            Some(k) => named(format!("item/{}", k.id), e),
            None => e,
        };
        if open
            && python.is_none()
            && let Some(command) = &original_command
        {
            inside.extend(command_panels(
                &mut self.documents,
                command,
                id,
                (content_width - 48.).max(1.),
                face,
                ink,
                &self.metrics,
            )?);
        }
        if open {
            if denied {
                let reason = match &decision {
                    Some(ApprovalDecision::Deny { reason: Some(r) }) => r.clone(),
                    _ => match &row.status {
                        Some(ItemStatus::Failed { reason, .. }) => reason.clone(),
                        _ => "denied".into(),
                    },
                };
                inside.push(error_box(&reason, ink).into());
            } else if let Some(cell) = &python {
                inside.extend(python_body(
                    &mut self.documents,
                    cell,
                    id,
                    (content_width - 48.).max(1.),
                    face.arguments_closed,
                    ink,
                    &self.metrics,
                )?);
            } else if d.kind == "execute" {
                // a finished run without output says so in the terminal's own voice
                let out = out_text
                    .as_ref()
                    .map(|o| f::term_output(o, ink))
                    .or_else(|| {
                        row.status.is_some().then(|| {
                            vec![
                                f::term_line(
                                    vec![(
                                        if d.quiet {
                                            "output stayed in the agent\u{2019}s own terminal"
                                        } else {
                                            "no output"
                                        }
                                        .into(),
                                        f::hex("#7d8597"),
                                        400,
                                        false,
                                    )],
                                    12.,
                                    true,
                                )
                                .into(),
                            ]
                        })
                    });
                let output = if let Some(text) = out_text.as_deref() {
                    named(
                        format!("term/{id}"),
                        code_panel(
                            if super::super::rich::json_output(text) {
                                self.documents
                                    .code_element_language(text, "json", &self.metrics)
                            } else {
                                f::term_output_full(text, ink)
                            },
                            &format!("codeout/{id}/output"),
                            (content_width - 48.).max(1.),
                            320.,
                            &self.metrics,
                            ctx,
                        )?,
                    )
                } else {
                    f::terminal(&format!("term/{id}"), out, ink)
                };
                inside.push(wrap(&term, output).into());
            } else if let Some(u) = &unified {
                let mut panel = code_panel(
                    f::diff_view(
                        &format!("diff/{}", change.as_ref().map_or(id, |c| c.id)),
                        u,
                        ink,
                    ),
                    &format!("codeout/{id}/diff"),
                    (content_width - 48.).max(1.),
                    360.,
                    &self.metrics,
                    ctx,
                )?;
                // The file name heads its own panel, on the code's inset.
                if let Some(path) = u
                    .lines()
                    .find_map(|line| line.strip_prefix("+++ "))
                    .filter(|path| *path != "/dev/null")
                    && let c::Element::Box(b) = &mut panel
                {
                    let header = f::frame(
                        f::Frame {
                            padding: [0., 0., 6., 0.],
                            ..Default::default()
                        },
                        vec![
                            f::line(
                                path.strip_prefix("b/").unwrap_or(path),
                                11.5,
                                500,
                                ink.dim,
                                true,
                            )
                            .into(),
                        ],
                    );
                    b.children.insert(0, header.into());
                }
                inside.push(wrap(&change, super::column(vec![panel.into()], None)).into());
            } else if let Some(text) = {
                // describe() unwraps ACP's console envelope. Preserve actual Markdown
                // fences (especially a tool result consisting of one Mermaid figure).
                let raw = output
                    .iter()
                    .map(OutputBlock::display_text)
                    .collect::<Vec<_>>()
                    .join("\n");
                let text = if !row.terminal_tail.is_empty()
                    || raw.is_empty()
                    || raw.trim_start().starts_with("```console\n")
                {
                    d.out.as_str()
                } else {
                    raw.as_str()
                };
                super::super::rich::tool_markdown(d.kind, &d.target, failed, text)
            } {
                let doc = self.documents.prepare(
                    id,
                    &text,
                    row.completed.is_some(),
                    content_width - 48.,
                    self.zoom.scale(),
                    &self.metrics,
                )?;
                inside.push(
                    doc.inset(id, content_width - 48., &self.metrics, ctx)?
                        .into(),
                );
                rich = Some(doc);
            } else if d.verb == "Read" && !d.out.is_empty() {
                let limit = if face.arguments_closed {
                    usize::MAX
                } else {
                    14
                };
                let (src, total) = f::source_view(&d.out, limit, ink);
                inside.push(src.into());
                if total > 14 {
                    inside.push(
                        named(
                            format!("more/{id}"),
                            f::disclosure(face.arguments_closed, ink),
                        )
                        .into(),
                    );
                }
            } else if (d.verb == "Search" || d.verb == "Find") && !d.out.is_empty() {
                let pattern = d
                    .args
                    .get("pattern")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                inside.push(
                    f::hits(
                        &format!("hits/{id}"),
                        &d.out,
                        if d.verb == "Search" { pattern } else { "$^" },
                        cwd,
                        ink,
                    )
                    .into(),
                );
            } else if !d.out.is_empty() {
                inside.push(
                    if failed {
                        error_box(&d.out, ink)
                    } else {
                        plain_box(&d.out, id, content_width - 48., ink, &self.metrics, ctx)?
                    }
                    .into(),
                );
            } else if failed && let Some(ItemStatus::Failed { reason, .. }) = &row.status {
                inside.push(error_box(reason, ink).into());
            }
        } else {
            if let Some(cell) = &python
                && let Some(row) = python_preview(cell, id, ink)
            {
                inside.push(row.into());
            }
            // folded children stay addressable while the card is closed
            for k in [&term, &change].into_iter().flatten() {
                inside.push(named(format!("item/{}", k.id), c::Element::Spacer).into());
            }
        }
        if open || !inside.is_empty() {
            let mut body = if inside.is_empty() {
                c::Element::Spacer
            } else {
                f::body_el(inside)
            };
            if open {
                body = named(format!("tool/{id}/body"), body);
            }
            // Keep the motion hook around the capture's open-body marker.
            kids.push(named(format!("toolbody/{id}"), body).into());
        }
        let _ = d.name;
        Ok((
            named(format!("tool/{id}"), f::card(&status, kids, ink)),
            rich,
        ))
    }
}
/// The receipt title says what happened in words (harness-spec.md, Turn receipt).
fn stop_title(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::EndTurn => "Done",
        StopReason::Quiet { .. } => "Quiet",
        StopReason::Cancelled => "Cancelled",
        StopReason::Interrupted => "Interrupted",
        StopReason::MaxTokens => "Hit the output limit",
        StopReason::MaxRounds => "Stopped at the round limit",
        StopReason::Error { class, .. } => match class {
            ErrorClass::Transient => "Connection dropped",
            ErrorClass::RateLimited => "Rate limited",
            ErrorClass::Auth => "Not signed in",
            ErrorClass::BadRequest => "Request refused",
            ErrorClass::Malformed => "Unreadable reply",
            ErrorClass::MidStream => "Reply cut off",
            ErrorClass::Config => "Profile misconfigured",
            ErrorClass::Store => "Could not save the session",
            ErrorClass::Tool => "Tool failed",
            ErrorClass::Internal => "Harness error",
            ErrorClass::Cancelled => "Cancelled",
            ErrorClass::Unknown(_) => "Failed",
        },
        StopReason::Unknown(_) => "Stopped",
    }
}
/// A thought's gist: emphasis and code marks dropped, up to the first sentence or line.
fn gist(summary: &str) -> String {
    let clean = summary
        .trim()
        .replace("**", "")
        .replace("__", "")
        .replace('`', "");
    let mut end = clean.len();
    let mut chars = clean.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if ch == '\n' {
            end = i;
            break;
        }
        if matches!(ch, '.' | '!' | '?') && chars.peek().is_some_and(|(_, n)| n.is_whitespace()) {
            end = i + ch.len_utf8();
            break;
        }
    }
    clean[..end].trim().to_owned()
}
fn term_status(row: &ViewItem, exit: Option<i32>) -> f::Status {
    match (&row.status, exit) {
        (None, _) => f::Status::Running,
        (Some(ItemStatus::Completed), Some(0) | None) => f::Status::Done,
        _ => f::Status::Failed,
    }
}
/// The values a Python run emitted, one per `emit()` call.
fn python_emits(cell: &crate::frontends::face_core::PythonCell) -> Vec<serde_json::Value> {
    match &cell.emitted {
        Some(serde_json::Value::Array(values)) => values.clone(),
        Some(serde_json::Value::Null) | None => vec![],
        Some(value) => vec![value.clone()],
    }
}
/// A closed Python card's one line: the failure, else the first emitted value, else
/// the latest output. No execution id: it costs no row or column while closed.
/// Project-owned design; credit reuse with
/// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
fn python_preview(
    cell: &crate::frontends::face_core::PythonCell,
    id: ItemId,
    ink: &f::Ink,
) -> Option<c::Element> {
    let emits = python_emits(cell);
    let parts = if let Some((code, message)) = cell.error_parts() {
        let text = [code.unwrap_or_default(), message]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" \u{b7} ");
        vec![(text, f::mix(ink.bad, ink.fg, 0.8), 400, false)]
    } else if let Some(first) = emits.first() {
        f::emit_preview(first, ink)
    } else {
        cell.output
            .values()
            .flat_map(|t| t.lines())
            .rfind(|l| !l.trim().is_empty())
            .map(|l| vec![(l.to_owned(), ink.dim, 400, false)])
            .unwrap_or_default()
    };
    if parts.is_empty() {
        return None;
    }
    let mut preview = f::term_line(parts, 12., false);
    if let c::Element::Text(t) = &mut preview {
        t.ellipsis = true;
    }
    let mut kids = vec![f::fill(named(format!("python/{id}/preview"), preview))];
    if emits.len() > 1 && cell.error.is_none() {
        kids.push(f::fixed(f::line(
            format!("+{} more", emits.len() - 1),
            11.,
            400,
            ink.dim,
            false,
        )));
    }
    // Inset to the source text's left edge, as the open card's results are.
    Some(f::frame(
        f::Frame {
            padding: [0., 10., 0., 10.],
            gap: 10.,
            row: true,
            ..Default::default()
        },
        kids,
    ))
}
/// Source lines an open Python card shows before its expand control; a source only
/// one line longer is shown whole rather than hiding a single line.
const PYTHON_SOURCE_LINES: usize = 5;
/// An open Run Python card inside the one card container: the source on the code
/// tint (clamped), the failure strip, then emits and output on the card surface,
/// inset to the source text, and one footer row with the expand control and a small
/// execution tag (the full id while details are expanded). No empty placeholders.
/// Project-owned design; credit reuse with
/// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
fn python_body(
    documents: &mut super::super::rich::Documents,
    cell: &crate::frontends::face_core::PythonCell,
    id: ItemId,
    width: f32,
    full: bool,
    ink: &f::Ink,
    metrics: &impl c::Measure,
) -> Result<Vec<c::Item>, String> {
    let ctx = documents.context();
    let mut kids: Vec<c::Item> = vec![];
    let mut clamped = false;
    if let Some(source) = cell.source.as_deref().filter(|s| !s.trim().is_empty()) {
        let mut code = documents.code_element(source.trim_end(), metrics);
        bare(&mut code);
        let ([_, height], _) = code.natural(f32::INFINITY, ctx, metrics)?;
        let lines = source.trim_end().lines().count().max(1);
        clamped = lines > PYTHON_SOURCE_LINES + 1;
        let cap = if full {
            320.
        } else if clamped {
            (height / lines as f32 * PYTHON_SOURCE_LINES as f32).round()
        } else {
            height
        };
        kids.push(
            panel(
                code,
                &format!("codeout/{id}/command"),
                width,
                cap,
                Look::Source,
                metrics,
                ctx,
            )?
            .into(),
        );
    }
    if let Some(error) = cell.error_parts() {
        kids.push(named(format!("python/{id}/error"), python_error(&error, ink)).into());
    }
    let emits = python_emits(cell);
    let mut cut = false;
    let mut results: Vec<c::Item> = vec![];
    for (n, value) in emits.iter().enumerate() {
        let (block, shortened) = f::emit_block(value, full, ink);
        cut |= shortened || (full && f::emit_block(value, false, ink).1);
        results.push(named(format!("python/{id}/emit/{n}"), block).into());
    }
    if !cell.output.is_empty() {
        use crate::frontends::face_core::{Verdict, verdict};
        let lines = cell
            .output
            .iter()
            .flat_map(|(index, text)| {
                let stderr = cell.stderr.contains(index);
                text.trim_end_matches('\n')
                    .lines()
                    .map(move |l| (l.to_owned(), stderr))
                    .collect::<Vec<_>>()
            })
            .map(|(text, stderr)| {
                let color = if stderr {
                    f::mix(ink.bad, ink.fg, 0.7)
                } else {
                    match verdict(&text) {
                        Verdict::Ok => ink.ok,
                        Verdict::Bad => ink.bad,
                        Verdict::Warn => ink.ask,
                        Verdict::Dim => ink.dim,
                        Verdict::Plain => ink.fg,
                    }
                };
                f::term_line(vec![(text, color, 400, false)], 12., false).into()
            })
            .collect();
        // The emit rows' 4 px inset, so output and values share one left edge.
        results.push(
            named(
                format!("term/{id}"),
                f::frame(
                    f::Frame {
                        padding: [0., 4., 0., 4.],
                        ..Default::default()
                    },
                    vec![f::vstack(lines, 0.).into()],
                ),
            )
            .into(),
        );
    }
    if !results.is_empty() {
        kids.push(
            panel(
                f::vstack(results, 8.),
                &format!("codeout/{id}/results"),
                width,
                if full { 640. } else { 320. },
                Look::Results,
                metrics,
                ctx,
            )?
            .into(),
        );
    }
    // One footer row: the centred expand control when something is clamped, and the
    // execution tag at the right. The row itself toggles details (and the full id).
    let expandable = clamped || cut;
    let tag = cell.execution_id.as_deref().map(|e| {
        if full {
            e.to_owned()
        } else {
            crate::frontends::face_core::PythonCell::short_id(e)
        }
    });
    if expandable || tag.is_some() {
        let side = |e: c::Element| c::Item {
            element: e,
            extent: c::Extent::Fill,
        };
        let mut row = vec![side(c::Element::Spacer)];
        if expandable {
            row.push(f::fixed(f::icon_turned(
                "chev",
                ink.seat,
                12.,
                1.8,
                if full { 270. } else { 90. },
            )));
        }
        row.push(side(match tag {
            Some(tag) => f::hstack(
                vec![
                    side(c::Element::Spacer),
                    f::fixed(named(
                        format!("python/{id}/execution"),
                        f::line(tag, 10., 400, f::alpha(ink.dim, 0.75), true),
                    )),
                ],
                0.,
            ),
            None => c::Element::Spacer,
        }));
        kids.push(named(format!("more/{id}"), f::hstack(row, 8.)).into());
    }
    Ok(kids)
}
/// A Markdown code block without its own frame, for a section that supplies the tint.
fn bare(code: &mut c::Element) {
    match code {
        c::Element::Named { content, .. } => bare(content),
        c::Element::Box(b) => {
            let o = &mut b.style.overrides;
            o.fill = Some(f::paint([0.; 4]));
            o.border = Some(ui::Border {
                width: ui::Length::Px(0.),
                paint: f::paint([0.; 4]),
                outside: false,
            });
            o.padding = Some([ui::Length::Px(0.); 4]);
            o.shadow = None;
        }
        _ => {}
    }
}
/// A failed run's own words above its data: the code bold, the message wrapped.
fn python_error(error: &(Option<String>, String), ink: &f::Ink) -> c::Element {
    let (code, message) = error;
    let mut parts = vec![];
    if let Some(code) = code {
        parts.push((code.clone(), ink.bad, 600, false));
        if !message.is_empty() {
            parts.push(("  ".into(), ink.bad, 400, false));
        }
    }
    parts.push((message.clone(), f::mix(ink.bad, ink.fg, 0.8), 400, false));
    // A tint, not a box: the card already draws the only border.
    f::frame(
        f::Frame {
            fill: f::alpha(ink.bad, 0.10),
            radius: [8.; 4],
            padding: [6., 10., 6., 10.],
            ..Default::default()
        },
        vec![f::runs(parts, 12., true, true).into()],
    )
}
fn error_box(text: &str, ink: &f::Ink) -> c::Element {
    f::frame(
        f::Frame {
            fill: f::alpha(ink.bad, 0.10),
            radius: [8.; 4],
            padding: [8., 10., 8., 10.],
            ..Default::default()
        },
        vec![
            f::runs(
                vec![(text.to_owned(), f::mix(ink.bad, ink.fg, 0.75), 400, false)],
                12.,
                true,
                true,
            )
            .into(),
        ],
    )
}
fn plain_box(
    text: &str,
    id: ItemId,
    width: f32,
    ink: &f::Ink,
    metrics: &impl c::Measure,
    ctx: ui::Context,
) -> Result<c::Element, String> {
    code_panel(
        f::runs(
            vec![(text.to_owned(), ink.fg, 400, false)],
            12.,
            true,
            false,
        ),
        &format!("mdout/{id}"),
        width.max(1.),
        400.,
        metrics,
        ctx,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "Source language and bounded command-panel geometry are independent presentation inputs"
)]
fn command_panels(
    documents: &mut super::super::rich::Documents,
    command: &str,
    id: ItemId,
    width: f32,
    face: &Face,
    ink: &f::Ink,
    metrics: &impl c::Measure,
) -> Result<Vec<c::Item>, String> {
    let ctx = documents.context();
    let python = super::super::rich::python_snippet(command);
    let long = command.contains('\n')
        || command.chars().count() > 80
        || f::line(command, 12., 400, ink.fg, true)
            .natural(f32::INFINITY, ctx, metrics)?
            .0[0]
            > (width - 122.).max(1.);
    if !long && python.is_none() {
        return Ok(vec![]);
    }
    let content = match &python {
        Some(p) => documents.code_element(p.source, metrics),
        None => f::term_line(vec![(command.into(), ink.fg, 400, false)], 12., false),
    };
    let mut children = vec![
        code_panel(
            content,
            &format!("codeout/{id}/command"),
            width,
            if face.arguments_closed { 320. } else { 76. },
            metrics,
            ctx,
        )?
        .into(),
        named(
            format!("more/{id}"),
            f::disclosure(face.arguments_closed, ink),
        )
        .into(),
    ];
    if python.is_some() {
        children.push(
            named(
                format!("source/{id}/toggle"),
                f::line(
                    if face.source {
                        "Hide original command"
                    } else {
                        "Original command"
                    },
                    10.5,
                    500,
                    ink.dim,
                    true,
                ),
            )
            .into(),
        );
        if face.source {
            children.push(
                code_panel(
                    f::term_line(vec![(command.into(), ink.fg, 400, false)], 12., false),
                    &format!("codeout/{id}/original"),
                    width,
                    320.,
                    metrics,
                    ctx,
                )?
                .into(),
            );
        }
    }
    Ok(children)
}

/// Literal code/output keeps its full payload. Only the viewport enters card geometry.
/// Project-owned bboxy450-harness; credit reuse with
/// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
fn code_panel(
    content: c::Element,
    key: &str,
    width: f32,
    cap: f32,
    metrics: &impl c::Measure,
    ctx: ui::Context,
) -> Result<c::Element, String> {
    panel(content, key, width, cap, Look::Code, metrics, ctx)
}
/// How a bounded panel sits in its card.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    /// The code tint with the horizontal-scroll arrow row.
    Code,
    /// The code tint without the arrow row (the viewport still scrolls).
    Source,
    /// No tint, inset so emit rows and output meet the source text's left edge.
    Results,
}
fn panel(
    mut content: c::Element,
    key: &str,
    width: f32,
    cap: f32,
    look: Look,
    metrics: &impl c::Measure,
    ctx: ui::Context,
) -> Result<c::Element, String> {
    let ink = f::ink(ctx);
    super::super::rich::bound_native_runs(&mut content);
    let ([natural_width, height], _) = content.natural(f32::INFINITY, ctx, metrics)?;
    let mut children = vec![
        named(
            key.to_owned(),
            c::Element::ScrollView(c::ScrollView {
                viewport: [width, height.min(cap).max(1.)],
                offset: [0., 0.],
                content_width: Some(natural_width.max(width)),
                content: Box::new(content),
            }),
        )
        .into(),
    ];
    if natural_width > width && look == Look::Code {
        children.push(
            f::hstack(
                vec![
                    f::fixed(named(
                        format!("codeleft/{key}"),
                        f::line("←", 10., 500, ink.dim, false),
                    )),
                    f::fixed(named(
                        format!("coderight/{key}"),
                        f::line("→", 10., 500, ink.dim, false),
                    )),
                ],
                10.,
            )
            .into(),
        );
    }
    let mut panel = f::frame(
        f::Frame {
            fill: if look == Look::Results {
                [0.; 4]
            } else {
                f::alpha(ink.bg, 0.55)
            },
            radius: [8.; 4],
            padding: if look == Look::Results {
                [0., 6., 0., 6.]
            } else {
                [6., 10., 6., 10.]
            },
            ..Default::default()
        },
        children,
    );
    if let c::Element::Box(b) = &mut panel {
        b.clip = true;
    }
    // Retain the terminal block's named output/padding contract inside its new viewport.
    Ok(
        if let Some(id) = key
            .strip_prefix("codeout/")
            .and_then(|k| k.strip_suffix("/output"))
        {
            named(format!("term/{id}/output"), panel)
        } else {
            panel
        },
    )
}
