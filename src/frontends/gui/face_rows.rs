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
use super::{Preparer, approval_atom, approval_element, context};
use crate::protocol::*;
use bbt_ui::{self as ui, components as c, modules::seat_row::named};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Kinds drawn by the rich face; the rest keep the toolkit message card.
pub(super) fn handles(_model: &Transcript, row: &ViewItem) -> bool {
    matches!(
        row.kind,
        ItemKind::UserMessage
            | ItemKind::AgentMessage
            | ItemKind::Reasoning
            | ItemKind::ToolCall
            | ItemKind::Approval
            | ItemKind::Notice
            | ItemKind::Usage
            | ItemKind::Terminal
            | ItemKind::FileChange
    )
}

#[derive(Default)]
pub(super) struct Folds {
    /// child -> the tool call that shows it
    pub parent: HashMap<ItemId, ItemId>,
    kids: HashMap<ItemId, Vec<ItemId>>,
    /// tool -> its open approval, still a card of its own (it owns the buttons)
    asking: HashMap<ItemId, ItemId>,
    /// the last visible row of each turn carries the turn's receipt once it ends
    last: HashMap<ItemId, TurnId>,
    /// runaway loop: the repeated reply that opens the card -> everything folded after it
    loops: HashMap<ItemId, Vec<ItemId>>,
    /// turn -> replies folded into its loop (the receipt says so)
    folded: HashMap<TurnId, usize>,
    /// the transcript's last visible row carries .hx-feed's 18 px bottom padding
    tail: Option<ItemId>,
    /// the first visible row carries .hx-feed's 14 px top padding
    first: Option<ItemId>,
}
/// Usage items never draw a row (harness-spec.md "Refinement pass 2"): their figures feed
/// the header meter and nothing else, so they carry neither the receipt nor the feed padding.
fn drawn(model: &Transcript, id: &ItemId) -> bool {
    model
        .items
        .get(id)
        .is_some_and(|r| r.kind != ItemKind::Usage)
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
        for id in &model.order {
            let r = &model.items[id];
            if f.parent.contains_key(id) {
                continue;
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
    pub fn deps(&self, model: &Transcript, id: &ItemId, p: &Preparer) -> u64 {
        // rows without folded children or a receipt carry no signature (0)
        let mut any = false;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        if self.tail == Some(*id) {
            any = true;
            "tail".hash(&mut h);
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
        let ctx = context();
        let ink = f::ink(ctx);
        let id = row.id;
        let cwd = model
            .meta
            .as_ref()
            .map(|m| m.cwd.display().to_string())
            .unwrap_or_default();
        let mut rich = None;
        // A usage item draws nothing (harness-spec.md "Refinement pass 2"): the header meter
        // sums its figures. The row keeps its id at zero height, with no region or padding.
        if matches!(row.body, Item::Usage { .. }) {
            return Ok((c::Element::Spacer, None));
        }
        let content: c::Element = match &row.body {
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
                f::user_bubble(if text.is_empty() { "…" } else { &text }, &ink)
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
                if live {
                    let mut text = f::text(format!("{summary} ▍"), 12.5, 400, ink.dim, false);
                    text.preserve_whitespace = true;
                    kids.push(
                        named(
                            format!("reasoning/{id}/body"),
                            f::thought_body(
                                named(format!("stream/{id}"), c::Element::Text(text)),
                                &ink,
                            ),
                        )
                        .into(),
                    );
                } else {
                    // Completed thoughts are prepared once even while collapsed.
                    let doc = self.documents.reasoning(
                        id,
                        summary,
                        true,
                        content_width - 20.,
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
                }
                f::vstack(kids, 4.)
            }
            Item::ToolCall { .. } => {
                let (card, doc) = self.tool_card(model, row, face, folds, content_width, &ink)?;
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
                f::card(
                    &status,
                    vec![
                        f::head(
                            f::Head {
                                key: format!("toolhead/{id}"),
                                tool_kind: "execute",
                                verb: "Run".into(),
                                target: command.trim_start_matches("bash -c ").to_owned(),
                                mono: true,
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
                        named(
                            format!("toolbody/{id}"),
                            f::body_el(vec![
                                f::terminal(
                                    &format!("term/{id}"),
                                    // a finished run without output says so (.hx-out dim)
                                    out.map(|o| f::term_output(&o, &ink)).or_else(|| {
                                        row.status.is_some().then(|| {
                                            vec![
                                                f::term_line(
                                                    vec![(
                                                        "no output".into(),
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
                                    }),
                                    &ink,
                                )
                                .into(),
                            ]),
                        )
                        .into(),
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
            Item::Command { .. } | Item::Hook { .. } | Item::SystemMessage { .. } => f::notice(
                &row.body.extension_text().unwrap_or_default(),
                &Level::Info,
                &ink,
            ),
            Item::Native { .. } => f::notice(
                &crate::frontends::native_label(&row.body).unwrap_or_default(),
                &Level::Info,
                &ink,
            ),
            Item::Unknown(raw) => f::notice(
                &format!("Unsupported item: {}", unknown_tag(raw)),
                &Level::Info,
                &ink,
            ),
            _ => return Err("face row for an unhandled kind".into()),
        };
        let mut content: Vec<c::Item> = vec![content.into()];
        if let Some(turn) = folds.last.get(&id)
            && let Some(TurnState::Ended { reason, usage }) = model.turns.get(turn)
        {
            let items: Vec<&ViewItem> = model
                .order
                .iter()
                .map(|i| &model.items[i])
                .filter(|r| r.turn == Some(*turn))
                .collect();
            let count = |k: ItemKind| items.iter().filter(|r| r.kind == k).count();
            let start = items.iter().map(|r| r.started_ms).min();
            let end = items.iter().filter_map(|r| r.completed_ms).max();
            content.push(
                named(
                    format!("turn/{}", turn.0),
                    f::receipt(
                        f::Receipt {
                            ok: matches!(reason, StopReason::EndTurn),
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
        // .hx-feed gap 8 (harness-spec.md "Tight transcript"): 4 + 4 between rows
        let gap = 4.;
        // .hx-feed's 14 px top padding sits on the first drawn row
        let top = if folds.first == Some(id) { 14. } else { gap };
        let _ = width;
        Ok((
            f::frame(
                f::Frame {
                    padding: [
                        top,
                        gutter,
                        if folds.tail == Some(id) { 18. } else { gap },
                        gutter,
                    ],
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
        let d = f::describe(name, tool_kind.clone(), input_raw, output, cwd);
        let approval = folds.kid(model, &id, ItemKind::Approval);
        let term = folds.kid(model, &id, ItemKind::Terminal);
        let change = folds.kid(model, &id, ItemKind::FileChange);
        let (decision, by) = match approval.as_ref().map(|a| &a.body) {
            Some(Item::Approval { decision, by, .. }) => (decision.clone(), by.clone()),
            _ => (None, None),
        };
        let denied = matches!(decision, Some(ApprovalDecision::Deny { .. }));
        let status = match &row.status {
            None if folds.asking.contains_key(&id) => f::Status::Asking,
            None => f::Status::Running,
            Some(_) if denied => f::Status::Denied,
            Some(ItemStatus::Completed) => f::Status::Done,
            Some(ItemStatus::Cancelled | ItemStatus::Interrupted) => f::Status::Stopped,
            Some(_) => f::Status::Failed,
        };
        // the terminal's own command is authoritative over the call's arguments
        let mut d = d;
        if output.is_empty()
            && let Some(value) = raw_output
        {
            d.out = value.to_string();
        }
        if let Some(Item::Terminal { command, .. }) = term.as_ref().map(|t| &t.body)
            && !command.is_empty()
        {
            d.target = command
                .strip_prefix("bash -c ")
                .unwrap_or(command)
                .to_owned();
            d.mono = true;
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
        let meta = match d.kind {
            // `exit N` only when N ≠ 0: success says nothing, the check mark already did
            "execute" => exit
                .filter(|e| *e != 0)
                .map(|e| (format!("exit {e}"), Some(false))),
            "read" if row.status.is_some() && lines > 0 => Some((format!("{lines} lines"), None)),
            "search" if row.status.is_some() => Some((
                format!("{lines} hit{}", if lines == 1 { "" } else { "s" }),
                None,
            )),
            _ => None,
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
        // harness-spec.md "Tight transcript": a finished, successful command opens only when
        // its output is 6 lines or fewer; running, asking, failed and edit cards stay open
        let auto = failed
            || matches!(status, f::Status::Asking | f::Status::Running)
            || (d.kind == "execute" && out_lines <= 6)
            || d.kind == "edit"
            || (d.verb == "Search" && lines <= 6);
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
        let mut inside: Vec<c::Item> = vec![];
        let mut rich = None;
        let caption = [title.as_deref(), server.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        if !caption.is_empty() {
            inside.push(f::notice(&caption, &Level::Info, ink).into());
        }
        let wrap = |kid: &Option<ViewItem>, e: c::Element| match kid {
            Some(k) => named(format!("item/{}", k.id), e),
            None => e,
        };
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
                let out = out_text.map(|o| f::term_output(&o, ink)).or_else(|| {
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
                inside.push(wrap(&term, f::terminal(&format!("term/{id}"), out, ink)).into());
            } else if let Some(u) = &unified {
                inside.push(
                    wrap(
                        &change,
                        f::diff_view(
                            &format!("diff/{}", change.as_ref().map_or(id, |c| c.id)),
                            u,
                            ink,
                        ),
                    )
                    .into(),
                );
            } else if let Some(text) = {
                // describe() unwraps ACP's console envelope. Preserve actual Markdown
                // fences (especially a tool result consisting of one Mermaid figure).
                let raw = output
                    .iter()
                    .map(OutputBlock::display_text)
                    .collect::<Vec<_>>()
                    .join("\n");
                let text = if raw.is_empty() || raw.trim_start().starts_with("```console\n") {
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
                    &self.metrics,
                )?;
                inside.push(doc.inset(id, content_width - 48., &self.metrics)?.into());
                rich = Some(doc);
            } else if d.verb == "Read" && !d.out.is_empty() {
                let limit = if face.arguments_closed {
                    usize::MAX
                } else {
                    14
                };
                let (src, total) = f::source_view(&d.out, limit, ink);
                inside.push(src.into());
                if total > limit {
                    inside.push(
                        named(
                            format!("more/{id}"),
                            f::line(
                                format!("Show all {total} lines"),
                                11.5,
                                500,
                                ink.seat,
                                false,
                            ),
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
                        plain_box(&d.out, ink)
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
fn plain_box(text: &str, ink: &f::Ink) -> c::Element {
    let t: String = text.chars().take(1600).collect();
    f::frame(
        f::Frame {
            fill: f::alpha(ink.bg, 0.55),
            radius: [8.; 4],
            padding: [8., 10., 8., 10.],
            ..Default::default()
        },
        vec![f::runs(vec![(t, ink.fg, 400, false)], 12., true, true).into()],
    )
}
