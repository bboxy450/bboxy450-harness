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
            Item::Notice { text, level } => f::notice(text, level, &ink),
            Item::Terminal { command, exit, .. } => {
                let status = term_status(row, *exit);
                let out = self.terminal_text(row, "");
                let panel_width = (content_width - inset - 48.).max(1.);
                let mut body = vec![];
                body.extend(command_panels(
                    &mut self.documents,
                    command,
                    false,
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
            d.target = f::compact_heading(
                cell.source
                    .as_deref()
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
        let meta = if let Some(cell) = &python {
            let mut label = cell.status.clone().unwrap_or_else(|| {
                if cell.error.is_some() {
                    "failed"
                } else {
                    "pending"
                }
                .into()
            });
            if let Some(exit) = cell.exit_code {
                label.push_str(&format!(" · exit {exit}"));
            }
            Some((label, cell.exit_code.map(|exit| exit == 0)))
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
        // Running calls keep the compact header; finished edits and small results open.
        let auto = failed
            || matches!(status, f::Status::Asking)
            || (row.completed.is_some()
                && ((d.kind == "execute" && out_lines <= 6)
                    || d.kind == "edit"
                    || (d.verb == "Search" && lines <= 6)));
        let open = auto != face.output_closed;
        let badge = f::badge(decision.as_ref(), by.as_ref(), ink).map(|b| match &approval {
            Some(a) => named(format!("item/{}", a.id), b),
            None => b,
        });
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
                    dur: row
                        .completed_ms
                        .map(|c| c.saturating_sub(row.started_ms))
                        .filter(|ms| *ms >= 1000)
                        .map(f::duration),
                    status,
                },
                ink,
            )
            .into(),
        ];
        if let Some(cell) = &python {
            if let Some(execution) = &cell.execution_id {
                kids.push(
                    named(
                        format!("python/{id}/execution"),
                        f::line(format!("Execution {execution}"), 10.5, 400, ink.dim, true),
                    )
                    .into(),
                );
            }
            let preview = cell.preview();
            if !preview.is_empty() {
                kids.push(
                    named(
                        format!("python/{id}/emitted"),
                        f::line(preview, 11.5, 400, ink.fg, true),
                    )
                    .into(),
                );
            }
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
            server.as_deref(),
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
        if open && let Some(command) = &original_command {
            inside.extend(command_panels(
                &mut self.documents,
                command,
                python.is_some(),
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
    python_source: bool,
    id: ItemId,
    width: f32,
    face: &Face,
    ink: &f::Ink,
    metrics: &impl c::Measure,
) -> Result<Vec<c::Item>, String> {
    let ctx = documents.context();
    let python = super::super::rich::python_snippet(command);
    let long = python_source
        || command.contains('\n')
        || command.chars().count() > 80
        || f::line(command, 12., 400, ink.fg, true)
            .natural(f32::INFINITY, ctx, metrics)?
            .0[0]
            > (width - 122.).max(1.);
    if !long && python.is_none() {
        return Ok(vec![]);
    }
    let content = match &python {
        _ if python_source => documents.code_element(command, metrics),
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
    mut content: c::Element,
    key: &str,
    width: f32,
    cap: f32,
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
    if natural_width > width {
        children.push(
            f::hstack(
                vec![
                    f::fixed(named(
                        format!("codeleft/{key}"),
                        f::line("←", 12., 500, ink.dim, false),
                    )),
                    f::fixed(named(
                        format!("coderight/{key}"),
                        f::line("→", 12., 500, ink.dim, false),
                    )),
                ],
                12.,
            )
            .into(),
        );
    }
    let mut panel = f::frame(
        f::Frame {
            fill: f::alpha(ink.bg, 0.55),
            radius: [8.; 4],
            padding: [7., 10., 10., 10.],
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
