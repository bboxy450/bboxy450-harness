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
}
impl Folds {
    pub fn of(model: &Transcript) -> Self {
        let mut f = Self::default();
        let is_tool = |id: &ItemId| {
            model
                .items
                .get(id)
                .is_some_and(|t| t.kind == ItemKind::ToolCall)
        };
        for id in &model.order {
            let r = &model.items[id];
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
        let mut seen = std::collections::HashSet::new();
        for id in model.order.iter().rev() {
            let r = &model.items[id];
            if let Some(t) = r.turn
                && !f.parent.contains_key(id)
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
                        InputBlock::Unknown => "Unsupported input".into(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                f::user_bubble(if text.is_empty() { "…" } else { &text }, &ink)
            }
            Item::AgentMessage { markdown } => {
                let doc = self.documents.prepare(
                    id,
                    markdown,
                    row.completed.is_some(),
                    content_width,
                    &self.metrics,
                )?;
                let mut kids: Vec<c::Item> = vec![];
                if face.source {
                    let mut t = f::text(markdown.clone(), 12., 400, ink.fg, true);
                    t.preserve_whitespace = true;
                    kids.push(c::Element::Text(t).into());
                } else {
                    kids.push(doc.element.clone().into());
                }
                if row.completed.is_some() {
                    kids.push(
                        f::hstack(
                            vec![
                                c::Item {
                                    element: c::Element::Spacer,
                                    extent: c::Extent::Fill,
                                },
                                f::fixed(named(
                                    format!("source/{id}/toggle"),
                                    f::line(
                                        if face.source { "formatted" } else { "source" },
                                        10.5,
                                        500,
                                        f::alpha(ink.dim, 0.7),
                                        true,
                                    ),
                                )),
                            ],
                            0.,
                        )
                        .into(),
                    );
                }
                rich = Some(doc);
                f::vstack(kids, 2.)
            }
            Item::Reasoning { summary, .. } => {
                let live = row.status.is_none();
                let open = live || face.reasoning_open;
                let gist = summary
                    .split_inclusive(['.', '!', '?'])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_owned();
                let mut kids: Vec<c::Item> = vec![
                    f::thought_row(
                        &format!("reasoning/{id}/toggle"),
                        live,
                        row.completed_ms.map(|c| c.saturating_sub(row.started_ms)),
                        if open { "" } else { &gist },
                        open,
                        &ink,
                    )
                    .into(),
                ];
                // parsed once whether or not it is shown: a disclosure never reparses
                let doc = self.documents.prepare(
                    id,
                    summary,
                    row.completed.is_some(),
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
                f::vstack(kids, 4.)
            }
            Item::ToolCall { .. } => self.tool_card(model, row, face, folds, &cwd, &ink)?,
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
            Item::Usage { usage } => f::notice(
                &format!(
                    "{} in · {} out{}",
                    kfmt(usage.input_tokens),
                    kfmt(usage.output_tokens),
                    usage
                        .cached_input_tokens
                        .filter(|c| *c > 0)
                        .map_or(String::new(), |c| format!(" · {} cached", kfmt(c)))
                ),
                &Level::Info,
                &ink,
            ),
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
                                meta: exit.map(|e| (format!("exit {e}"), Some(e == 0))),
                                badge: None,
                                dur: None,
                                status: term_status(row, *exit),
                            },
                            &ink,
                        )
                        .into(),
                        f::body(vec![
                            f::terminal(
                                &format!("term/{id}"),
                                command.trim_start_matches("bash -c "),
                                out.map(|o| f::term_output(&o, &ink)),
                                *exit,
                                &ink,
                            )
                            .into(),
                        ]),
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
                        f::body(vec![
                            f::diff_view(&format!("diff/{id}"), unified, &ink).into(),
                        ]),
                    ],
                    &ink,
                )
            }
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
                            reason: match reason {
                                StopReason::Error { message, .. } => format!("Error: {message}"),
                                other => format!("{other:?}").to_lowercase().replace('_', " "),
                            },
                            took: start.zip(end).map(|(a, b)| b.saturating_sub(a)),
                            rounds: count(ItemKind::Reasoning),
                            tools: count(ItemKind::ToolCall),
                            edits: count(ItemKind::FileChange),
                            tokens: usage.as_ref().map(|u| (u.input_tokens, u.output_tokens)),
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
        let gap = if matches!(row.kind, ItemKind::Notice | ItemKind::Usage) {
            3.
        } else {
            5.
        };
        let _ = width;
        Ok((
            f::frame(
                f::Frame {
                    padding: [gap, gutter, gap, gutter],
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
        cwd: &str,
        ink: &f::Ink,
    ) -> Result<c::Element, String> {
        let Item::ToolCall {
            name,
            tool_kind,
            input_raw,
            output,
            ..
        } = &row.body
        else {
            return Err("not a tool call".into());
        };
        let id = row.id;
        let d = f::describe(name, *tool_kind, input_raw, output, cwd);
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
            Some(_) => f::Status::Failed,
        };
        // the terminal's own command is authoritative over the call's arguments
        let mut d = d;
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
            "execute" => exit.map(|e| (format!("exit {e}"), Some(e == 0))),
            "read" if row.status.is_some() && lines > 0 => Some((format!("{lines} lines"), None)),
            "search" if row.status.is_some() => Some((
                format!("{lines} hit{}", if lines == 1 { "" } else { "s" }),
                None,
            )),
            _ => None,
        };
        let failed = matches!(status, f::Status::Failed | f::Status::Denied);
        let auto = failed
            || matches!(status, f::Status::Asking)
            || d.kind == "execute"
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
                    dur: row
                        .completed_ms
                        .map(|c| f::duration(c.saturating_sub(row.started_ms))),
                    status,
                },
                ink,
            )
            .into(),
        ];
        let mut inside: Vec<c::Item> = vec![];
        let wrap = |kid: &Option<ViewItem>, e: c::Element| match kid {
            Some(k) => named(format!("item/{}", k.id), e),
            None => e,
        };
        if open {
            if denied {
                let reason = match &decision {
                    Some(ApprovalDecision::Deny { reason: Some(r) }) => r.clone(),
                    _ => match &row.status {
                        Some(ItemStatus::Failed { reason }) => reason.clone(),
                        _ => "denied".into(),
                    },
                };
                inside.push(error_box(&reason, ink).into());
            } else if d.kind == "execute" {
                let command = d.target.clone();
                let out = match &term {
                    Some(t) => self.terminal_text(t, &d.out),
                    None => (!d.out.is_empty()).then(|| d.out.clone()),
                };
                inside.push(
                    wrap(
                        &term,
                        f::terminal(
                            &format!("term/{id}"),
                            &command,
                            out.map(|o| f::term_output(&o, ink)),
                            exit,
                            ink,
                        ),
                    )
                    .into(),
                );
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
            } else if failed && let Some(ItemStatus::Failed { reason }) = &row.status {
                inside.push(error_box(reason, ink).into());
            }
        } else {
            // folded children stay addressable while the card is closed
            for k in [&term, &change].into_iter().flatten() {
                inside.push(named(format!("item/{}", k.id), c::Element::Spacer).into());
            }
        }
        if !inside.is_empty() {
            kids.push(f::body(inside));
        }
        let _ = d.name;
        Ok(named(format!("tool/{id}"), f::card(&status, kids, ink)))
    }
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
fn kfmt(n: u64) -> String {
    match n {
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1e6),
        n if n >= 10_000 => format!("{}k", n / 1000),
        n if n >= 1000 => format!("{:.1}k", n as f64 / 1e3),
        n => n.to_string(),
    }
}
