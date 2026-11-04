// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Floating menus: the "/" command menu and the header profile switcher.
//! Adapted from bboxy450-term lab/web harness.js (slashMenu, profileMenu) and
//! lab.css (.hx-cmenu, .hx-crow, .hx-pmenu, .hx-prow), to harness-spec.md's
//! flat fallbacks. https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
#![deny(clippy::disallowed_methods)]
use super::{
    bridge::{ProfileAuth, ProfileInfo, ProfileKind},
    face::{Frame, Ink, Rgba, alpha, fill, fixed, frame, hex, icon, kilo, mix, paint, runs, tile},
};
use bbt_ui::{self as ui, components as c, modules::seat_row::named};

/// .hx-pmenu: min(440, pane − 20), 10 from the left.
pub fn profile_menu_width(width: f32) -> f32 {
    (width - 20.).clamp(200., 440.)
}
/// .hx-cmenu-host: min(460, pane − 24), 12 from the left.
pub fn slash_menu_width(width: f32) -> f32 {
    (width - 24.).clamp(200., 460.)
}
/// The "/" menu shows at most eight rows.
pub const SLASH_ROWS: usize = 8;
/// The card's row list, split from the card chrome so the view can paint the
/// sliding selection highlight between them (harness-spec.md Motion).
pub const LIST: &str = "menu/list";

/// Compact native select trigger, shared by the header's model and effort controls.
pub fn select_chip(
    key: &str,
    title: &str,
    value: &str,
    open: bool,
    enabled: bool,
    ink: &Ink,
) -> c::Element {
    let control = named(
        key,
        frame(
            Frame {
                fill: alpha(ink.seat, if open { 0.14 } else { 0.06 }),
                border: Some(alpha(ink.seat, if open { 0.5 } else { 0.12 })),
                radius: [7.; 4],
                padding: [5., 8., 5., 8.],
                gap: 6.,
                row: true,
                ..Default::default()
            },
            vec![
                fixed(tight(title, 11., 400, ink.dim, false)),
                fixed(tight(
                    super::face::compact_heading(value, 24),
                    11.5,
                    600,
                    ink.fg,
                    false,
                )),
                fixed(super::face::icon_turned(
                    "chev",
                    ink.dim,
                    10.,
                    1.6,
                    if open { 270. } else { 90. },
                )),
            ],
        ),
    );
    if enabled { control } else { dimmed(control) }
}

/// One bounded menu style for every advertised single-choice agent control.
pub fn select_menu(
    key: &str,
    options: &[String],
    current: usize,
    look: &Look,
    ink: &Ink,
) -> c::Element {
    let rows = options
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let row_key = format!("{key}/{index}");
            let hover = look.hover(&row_key);
            fixed(named(
                row_key,
                frame(
                    Frame {
                        fill: alpha(ink.fg, 0.05 * hover),
                        radius: [7.; 4],
                        padding: [7., 10., 7., 10.],
                        gap: 10.,
                        row: true,
                        ..Default::default()
                    },
                    vec![
                        fill(tight(
                            text,
                            12.,
                            if index == current { 600 } else { 400 },
                            ink.fg,
                            false,
                        )),
                        fixed(if index == current {
                            icon("check", ink.seat, 14., 2.)
                        } else {
                            frame(
                                Frame {
                                    padding: [0., 14., 0., 0.],
                                    ..Default::default()
                                },
                                vec![],
                            )
                        }),
                    ],
                ),
            ))
        })
        .collect();
    card(
        vec![fixed(named(
            LIST,
            frame(
                Frame {
                    padding: [5.; 4],
                    gap: 2.,
                    ..Default::default()
                },
                rows,
            ),
        ))],
        ink,
    )
}

/// Per-row animated weights in 0..1, keyed by row name: `sel` is how far a
/// row wears the selected look (the "/" tile tint), `hover` its pointer fill.
/// The sliding selection fill itself is painted by the view under [`LIST`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Look {
    pub sel: Vec<(String, f32)>,
    pub hover: Vec<(String, f32)>,
}
impl Look {
    /// The settled look: one row fully selected, nothing hovered.
    pub fn settled(selected: Option<String>) -> Self {
        Self {
            sel: selected.into_iter().map(|k| (k, 1.)).collect(),
            hover: vec![],
        }
    }
    fn weight(list: &[(String, f32)], key: &str) -> f32 {
        list.iter()
            .find(|(k, _)| k == key)
            .map_or(0., |(_, w)| w.clamp(0., 1.))
    }
    pub fn sel(&self, key: &str) -> f32 {
        Self::weight(&self.sel, key)
    }
    pub fn hover(&self, key: &str) -> f32 {
        Self::weight(&self.hover, key)
    }
}

const LAB_SEAT: &str = "#2EE6A8";

fn hair(ink: &Ink) -> Rgba {
    alpha(ink.fg, 0.1)
}
fn card_hi(ink: &Ink) -> Rgba {
    alpha(ink.fg, 0.07)
}

/// One-line text runs; each part is (text, colour, weight, mono).
fn spans(parts: Vec<(String, Rgba, u16, bool)>, size: f32) -> c::Element {
    let mut t = c::Text::new(
        "",
        ui::Text {
            size: Some(size),
            line_height: 1.3,
            ..Default::default()
        },
    );
    t.runs = parts
        .into_iter()
        .filter(|(s, ..)| !s.is_empty())
        .map(|(value, color, weight, mono)| c::TextRun {
            value,
            style: ui::Text {
                step: ui::TypeStep::Body,
                size: Some(size),
                weight,
                paint: paint(color),
                mono,
                line_height: 1.3,
                ..Default::default()
            },
        })
        .collect();
    if t.runs.is_empty() {
        t.runs.push(c::TextRun {
            value: " ".into(),
            style: ui::Text {
                size: Some(size),
                line_height: 1.3,
                ..Default::default()
            },
        });
    }
    t.wrap = false;
    t.ellipsis = true;
    t.preserve_whitespace = true;
    c::Element::Text(t)
}
fn tight(value: impl Into<String>, size: f32, weight: u16, color: Rgba, mono: bool) -> c::Element {
    spans(vec![(value.into(), color, weight, mono)], size)
}
fn caps(value: &str, size: f32, tracking: f32, color: Rgba) -> c::Element {
    let mut t = c::Text::new(
        value.to_uppercase(),
        ui::Text {
            step: ui::TypeStep::Body,
            size: Some(size),
            weight: 700,
            paint: paint(color),
            tracking_em: tracking,
            line_height: 1.3,
            ..Default::default()
        },
    );
    t.wrap = false;
    c::Element::Text(t)
}
fn hairline(ink: &Ink) -> c::Element {
    c::Element::Rects(c::Rects {
        size: [1., 1.],
        rects: std::sync::Arc::from([(
            bbt_ui_model::layout::Rect::new(0., 0., 1., 1.),
            paint(hair(ink)),
        )]),
        repeat: Some((bbt_ui_model::layout::Axis::Row, 1.)),
    })
}
fn gap(w: f32) -> c::Item {
    fixed(frame(
        Frame {
            padding: [0., w, 0., 0.],
            ..Default::default()
        },
        vec![],
    ))
}
fn dimmed(e: c::Element) -> c::Element {
    c::Element::Filter {
        content: Box::new(e),
        opacity: 0.5,
        saturation: 1.,
    }
}

/// Card material (flat fallback): surface_2, a hair border and an outer
/// 1px seat-16 ring. The drop shadow becomes a 1px bg line (see the view).
fn card(children: Vec<c::Item>, ink: &Ink) -> c::Element {
    let mut surface = frame(
        Frame {
            border: Some(hair(ink)),
            radius: [13.; 4],
            ..Default::default()
        },
        children,
    );
    if let c::Element::Box(b) = &mut surface {
        b.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface2));
    }
    frame(
        Frame {
            border: Some(alpha(ink.seat, 0.16)),
            radius: [14.; 4],
            ..Default::default()
        },
        vec![fixed(surface)],
    )
}

/// `kbd`: 16 high, min width 16, radius 4, mono 10, card-hi, hair border.
fn kbd(key: &str, ink: &Ink) -> c::Element {
    frame(
        Frame {
            fill: card_hi(ink),
            border: Some(hair(ink)),
            radius: [4.; 4],
            padding: [1., 4., 0., 4.],
            row: true,
            ..Default::default()
        },
        vec![fixed(tight(key, 10., 400, ink.dim, true))],
    )
}
/// .hx-kind / .hx-def: a small caps tag.
fn tag(label: &str, fg: Rgba, bg: Rgba, tracking: f32) -> c::Element {
    frame(
        Frame {
            fill: bg,
            radius: [4.; 4],
            padding: [0., 5., 0., 5.],
            row: true,
            ..Default::default()
        },
        vec![fixed(caps(label, 9.5, tracking, fg))],
    )
}

fn agent_mark(program: &str) -> (String, Rgba) {
    let (m, color) = match program {
        "claude-agent-acp" => ("C", "#D97757"),
        "codex-acp" => ("Cx", "#10A37F"),
        "opencode" => ("oc", "#5B6178"),
        "pi-acp" => ("π", "#FF4FA3"),
        "omp" => ("ω", "#B07BFF"),
        "goose" => ("G", "#C98A1B"),
        other => {
            let first = other
                .chars()
                .next()
                .map_or("·".into(), |c| c.to_uppercase().to_string());
            return (first, hex(LAB_SEAT));
        }
    };
    (m.into(), hex(color))
}
/// (mark, colour, label) of a model provider (web PROVIDERS).
fn provider(p: &ProfileInfo) -> (&'static str, &'static str, String) {
    match p.provider.as_deref() {
        Some("openrouter") => ("OR", "#7C6CFF", "OpenRouter".into()),
        Some("anthropic") => ("A", "#D97757", "Anthropic API".into()),
        Some("anthropic-subscription") => ("A", "#D97757", "Anthropic subscription".into()),
        Some("openai") => ("O", "#10A37F", "OpenAI".into()),
        Some("local") => ("L", "#9AA3C7", "Local".into()),
        other => ("?", "#9AA3C7", other.unwrap_or("custom").into()),
    }
}
fn provider_rank(p: &ProfileInfo) -> usize {
    [
        "openrouter",
        "anthropic",
        "anthropic-subscription",
        "openai",
        "local",
    ]
    .iter()
    .position(|x| p.provider.as_deref() == Some(x))
    .unwrap_or(5)
}
/// The provider or agent mark of a profile, and whether it is an agent.
pub fn mark(p: &ProfileInfo) -> (String, Rgba, bool) {
    if p.kind == ProfileKind::Agent {
        let (m, color) = agent_mark(p.agent.first().map(String::as_str).unwrap_or(""));
        (m, color, true)
    } else {
        let (m, color, _) = provider(p);
        (m.into(), hex(color), false)
    }
}
/// Agents are circles, models radius-7 squares (radius 5 at 18px). Flat: no
/// gradient, no coloured shadow.
fn mark_tile(p: &ProfileInfo, side: f32) -> c::Element {
    let (m, color, agent) = mark(p);
    let radius = if agent {
        side / 2.
    } else if side < 24. {
        5.
    } else {
        7.
    };
    tile(
        runs(
            vec![(m, hex("#FFFFFF"), 800, false)],
            if side < 24. { 9. } else { 10.5 },
            false,
            false,
        ),
        side,
        radius,
        color,
    )
}
/// The "/" menu detail: the model, or for an agent its command and model.
pub fn profile_detail(p: &ProfileInfo) -> String {
    match p.kind {
        ProfileKind::Unreadable => "unreadable profile".into(),
        ProfileKind::Agent if p.model.is_empty() => p.agent.join(" "),
        ProfileKind::Agent => format!("{} · {}", p.agent.join(" "), p.model),
        ProfileKind::Model => p.model.clone(),
    }
}
/// A row that cannot be chosen: unreadable, or its key env is missing.
pub fn off(p: &ProfileInfo) -> bool {
    p.kind == ProfileKind::Unreadable || (p.kind == ProfileKind::Model && !p.auth_ok)
}
fn auth_chip(p: &ProfileInfo, ink: &Ink) -> c::Element {
    let ok = (mix(ink.ok, ink.fg, 0.85), alpha(ink.ok, 0.12));
    let na = (ink.dim, card_hi(ink));
    let (label, (fg, bg)) = match p.kind {
        ProfileKind::Unreadable => ("unreadable".to_owned(), na),
        ProfileKind::Agent if p.agent_env > 0 => (format!("env {}", p.agent_env), na),
        ProfileKind::Agent => ("own login".into(), na),
        ProfileKind::Model => match p.auth {
            ProfileAuth::Env if !p.auth_ok => {
                ("missing env".into(), (ink.bad, alpha(ink.bad, 0.12)))
            }
            ProfileAuth::Env => ("env".into(), ok),
            ProfileAuth::Keychain => ("Keychain".into(), ok),
            ProfileAuth::Command => ("command".into(), ok),
            ProfileAuth::InFile => ("key in file".into(), ok),
            ProfileAuth::None => ("no key".into(), ok),
        },
    };
    frame(
        Frame {
            fill: bg,
            radius: [999.; 4],
            padding: [1., 7., 1., 7.],
            row: true,
            ..Default::default()
        },
        vec![fixed(tight(label, 10.5, 600, fg, false))],
    )
}

/// Header profile switcher state: a filter query and the keyboard selection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileMenu {
    pub query: String,
    pub selected: usize,
}
impl ProfileMenu {
    /// Rows in display order: ACP agents first, then models by provider.
    pub fn rows<'a>(&self, profiles: &'a [ProfileInfo]) -> Vec<&'a ProfileInfo> {
        let q = self.query.to_lowercase();
        let hit = |p: &&ProfileInfo| {
            q.is_empty()
                || format!(
                    "{} {} {} {}",
                    p.name,
                    p.model,
                    provider(p).2,
                    p.agent.join(" ")
                )
                .to_lowercase()
                .contains(&q)
        };
        let mut out: Vec<_> = profiles
            .iter()
            .filter(|p| p.kind == ProfileKind::Agent)
            .filter(hit)
            .collect();
        let mut models: Vec<_> = profiles
            .iter()
            .filter(|p| p.kind != ProfileKind::Agent)
            .filter(hit)
            .collect();
        models.sort_by_key(|p| provider_rank(p));
        out.extend(models);
        out
    }
    pub fn move_by(&mut self, delta: isize, len: usize) {
        if len > 0 {
            self.selected = (self.selected as isize + delta).rem_euclid(len as isize) as usize;
        }
    }
    /// The profile Enter switches to, when it can be chosen.
    pub fn chosen<'a>(&self, profiles: &'a [ProfileInfo]) -> Option<&'a ProfileInfo> {
        self.rows(profiles)
            .get(self.selected)
            .copied()
            .filter(|p| !off(p))
    }
    /// The name of the keyboard-selected row, when it can be chosen.
    pub fn selected_key(&self, profiles: &[ProfileInfo]) -> Option<String> {
        self.chosen(profiles)
            .map(|p| format!("pmenu/row/{}", p.name))
    }
    /// Choosable rows are named `pmenu/row/{name}`; the footer link `pmenu/new`.
    pub fn element(
        &self,
        profiles: &[ProfileInfo],
        current: Option<(&str, &str)>,
        ink: &Ink,
        look: &Look,
    ) -> c::Element {
        let rows = self.rows(profiles);
        let mut list: Vec<c::Item> = vec![];
        if let Some((name, model)) = current
            && !profiles.iter().any(|p| p.name == name)
        {
            list.push(profile_row(
                "pmenu/current".into(),
                (true, 0., false),
                tile(
                    runs(
                        vec![("·".into(), hex("#FFFFFF"), 800, false)],
                        10.5,
                        false,
                        false,
                    ),
                    24.,
                    7.,
                    hex("#9AA3C7"),
                ),
                vec![
                    fixed(tight(name, 13., 650, ink.fg, false)),
                    fixed(tight(
                        format!("{model} · not in profiles/"),
                        11.,
                        400,
                        ink.dim,
                        true,
                    )),
                ],
                vec![],
                ink,
            ));
        }
        let agents = rows.iter().filter(|p| p.kind == ProfileKind::Agent).count();
        for (i, p) in rows.iter().enumerate() {
            if i == 0 && agents > 0 {
                list.push(kind_group("plug", "ACP agents", agents, ink));
            }
            if i == agents {
                list.push(kind_group("think", "Models", rows.len() - agents, ink));
            }
            let on = current.is_some_and(|(name, _)| name == p.name);
            let sub = if p.kind == ProfileKind::Agent {
                let mut parts = vec![
                    ("$ ".to_owned(), ink.seat, 400, true),
                    (p.agent.join(" "), ink.dim, 400, true),
                ];
                if !p.model.is_empty() {
                    parts.push((format!(" · {}", p.model), ink.dim, 400, true));
                }
                parts
            } else if p.kind == ProfileKind::Unreadable {
                vec![("unreadable profile".into(), ink.dim, 400, true)]
            } else {
                vec![
                    (p.model.clone(), ink.dim, 400, true),
                    (
                        format!(" · {}", provider(p).2),
                        alpha(ink.dim, 0.75),
                        400,
                        false,
                    ),
                ]
            };
            let mut name_line = vec![fixed(tight(p.name.clone(), 13., 650, ink.fg, false))];
            if p.default {
                name_line.push(fixed(tag("default", ink.seat, alpha(ink.seat, 0.14), 0.05)));
            }
            let mut tail = vec![];
            if let Some(n) = p.context_tokens {
                tail.push(fixed(tight(kilo(n), 10.5, 400, ink.dim, false)));
            }
            tail.push(fixed(auth_chip(p, ink)));
            let key = if off(p) {
                format!("pmenu/off/{}", p.name)
            } else {
                format!("pmenu/row/{}", p.name)
            };
            let hover = if off(p) { 0. } else { look.hover(&key) };
            list.push(profile_row(
                key,
                (on, hover, off(p)),
                mark_tile(p, 24.),
                vec![
                    fixed(frame(
                        Frame {
                            gap: 6.,
                            row: true,
                            ..Default::default()
                        },
                        name_line,
                    )),
                    fixed(spans(sub, 11.)),
                ],
                tail,
                ink,
            ));
        }
        if rows.is_empty() {
            list.push(fixed(frame(
                Frame {
                    padding: [10., 8., 10., 8.],
                    ..Default::default()
                },
                vec![fixed(tight(
                    if profiles.is_empty() {
                        "No profiles yet".to_owned()
                    } else {
                        format!("No profile matches “{}”", self.query)
                    },
                    12.5,
                    400,
                    ink.dim,
                    false,
                ))],
            )));
        }
        // focused search field: seat border and a 3px seat-14 ring
        let field = frame(
            Frame {
                fill: alpha(ink.fg, 0.04),
                border: Some(alpha(ink.seat, 0.6)),
                radius: [9.; 4],
                padding: [7., 10., 7., 10.],
                row: true,
                ..Default::default()
            },
            vec![fill(if self.query.is_empty() {
                tight("Switch profile…", 13., 400, alpha(ink.dim, 0.8), false)
            } else {
                tight(self.query.clone(), 13., 400, ink.fg, false)
            })],
        );
        let search = frame(
            Frame {
                padding: [5., 5., 5., 5.],
                ..Default::default()
            },
            vec![fixed(frame(
                Frame {
                    fill: alpha(ink.seat, 0.14),
                    radius: [12.; 4],
                    padding: [3.; 4],
                    ..Default::default()
                },
                vec![fixed(field)],
            ))],
        );
        let foot = frame(
            Frame {
                padding: [7., 12., 7., 12.],
                gap: 8.,
                row: true,
                ..Default::default()
            },
            vec![
                fixed(named(
                    "pmenu/new",
                    tight("Set up agents / profiles", 11., 650, ink.seat, false),
                )),
                fill(c::Element::Spacer),
                fixed(tight(
                    "~/.bboxy450-harness/profiles",
                    10.5,
                    400,
                    ink.dim,
                    true,
                )),
            ],
        );
        card(
            vec![
                fixed(search),
                fixed(named(
                    LIST,
                    frame(
                        Frame {
                            padding: [0., 6., 6., 6.],
                            ..Default::default()
                        },
                        list,
                    ),
                )),
                fixed(hairline(ink)),
                fixed(foot),
            ],
            ink,
        )
    }
}
/// .hx-pgrp with a 12px icon and a right-aligned count at 70%.
fn kind_group(glyph: &str, label: &str, count: usize, ink: &Ink) -> c::Item {
    fixed(frame(
        Frame {
            padding: [8., 8., 3., 8.],
            gap: 6.,
            row: true,
            ..Default::default()
        },
        vec![
            fixed(icon(glyph, ink.dim, 12., 1.5)),
            fixed(caps(label, 10., 0.08, ink.dim)),
            fill(c::Element::Spacer),
            fixed(tight(
                count.to_string(),
                10.,
                600,
                alpha(ink.dim, 0.7),
                false,
            )),
        ],
    ))
}
/// .hx-prow: padding 6 8, radius 9, gap 9; tile, name block, spacer, tail, check.
/// The current row is seat-9; a hover fades to seat-12 over it (the web's
/// `:hover:not(.off)` wins over `.on`). The keyboard selection's seat-12 is the
/// view's sliding highlight, painted under the list.
fn profile_row(
    key: String,
    (on, hover, off): (bool, f32, bool),
    tile_el: c::Element,
    block: Vec<c::Item>,
    tail: Vec<c::Item>,
    ink: &Ink,
) -> c::Item {
    let base = if on { 0.09 } else { 0. };
    let tint = alpha(ink.seat, base + (0.12 - base) * hover);
    let mut block = frame(Frame::default(), block);
    let mut tile_el = tile_el;
    if off && !on {
        block = dimmed(block);
        tile_el = dimmed(tile_el);
    }
    let mut kids = vec![fixed(tile_el), fill(block)];
    kids.extend(tail);
    kids.push(fixed(if on {
        icon("check", ink.seat, 16., 1.8)
    } else {
        frame(
            Frame {
                padding: [0., 16., 0., 0.],
                ..Default::default()
            },
            vec![],
        )
    }));
    fixed(named(
        key,
        frame(
            Frame {
                fill: tint,
                radius: [9.; 4],
                padding: [6., 8., 6., 8.],
                gap: 9.,
                row: true,
                ..Default::default()
            },
            kids,
        ),
    ))
}

const COMMANDS: [(&str, &str, &str); 3] = [
    ("/interrupt", "Stop the running turn", "stop"),
    ("/stop", "Close this session", "x"),
    ("/profiles", "Open the profile switcher", "plug"),
];

/// The first row the "/" menu shows: the selection stays in the window.
pub fn slash_window(selected: Option<usize>, rows: usize) -> usize {
    selected.map_or(0, |s| (s + 1).saturating_sub(rows.max(1)))
}

/// The name of the "/" menu's selected row.
pub fn slash_selected_key(popover: &ui::text_edit::CompletionPopover) -> Option<String> {
    popover
        .choices
        .selected()
        .map(|i| format!("completion/{i}"))
}

/// The "/" menu card. Rows are named `completion/{i}`; at most `rows` show.
/// The selected row's seat-14 fill is the view's sliding highlight under
/// [`LIST`]; here its tile wears the seat tint by `look.sel`, and a hovered
/// row fades in `card-hi`.
pub fn slash_menu(
    popover: &ui::text_edit::CompletionPopover,
    profiles: &[ProfileInfo],
    rows: usize,
    ink: &Ink,
    look: &Look,
) -> c::Element {
    let selected = popover.choices.selected();
    let rows = rows.clamp(1, SLASH_ROWS);
    let start = slash_window(selected, rows);
    let typed = popover.request.query.chars().count() + 1;
    let options = &popover.choices.options;
    let mut list = vec![];
    let mut last_group = None;
    for (i, choice) in options.iter().enumerate().skip(start).take(rows) {
        let label = choice.label.as_str();
        let command = COMMANDS.iter().find(|(c, ..)| *c == label);
        let profile = label
            .strip_prefix("/profile ")
            .and_then(|n| profiles.iter().find(|p| p.name == n));
        let group = command.is_some();
        if last_group != Some(group) {
            last_group = Some(group);
            list.push(fixed(frame(
                Frame {
                    padding: [6., 8., 2., 8.],
                    ..Default::default()
                },
                vec![fixed(caps(
                    if group { "Commands" } else { "Switch profile" },
                    10.,
                    0.08,
                    ink.dim,
                ))],
            )));
        }
        let key = format!("completion/{i}");
        let sel = look.sel(&key);
        let hover = look.hover(&key);
        let lead = match (command, profile) {
            (Some((.., glyph)), _) => tile(
                icon(glyph, mix(ink.seat, ink.dim, sel), 11., 1.5),
                18.,
                5.,
                mix(alpha(ink.seat, 0.16), card_hi(ink), sel),
            ),
            (None, Some(p)) => mark_tile(p, 18.),
            (None, None) => tile(c::Element::Spacer, 18., 5., card_hi(ink)),
        };
        let split = label
            .char_indices()
            .nth(typed)
            .map_or(label.len(), |(at, _)| at);
        let detail = command
            .map(|(_, say, _)| say.to_string())
            .or(profile.map(profile_detail))
            .unwrap_or_default();
        let mut kids = vec![
            fixed(lead),
            fixed(spans(
                vec![
                    (label[..split].to_owned(), ink.seat, 700, true),
                    (label[split..].to_owned(), ink.fg, 400, true),
                ],
                12.5,
            )),
            fill(tight(detail, 12., 400, ink.dim, false)),
        ];
        if label == "/interrupt" {
            kids.push(fixed(kbd("esc", ink)));
        } else if profile.is_some_and(|p| p.kind == ProfileKind::Agent) {
            let seat = hex(LAB_SEAT);
            kids.push(fixed(tag(
                "ACP",
                mix(seat, ink.fg, 0.85),
                alpha(seat, 0.14),
                0.06,
            )));
        }
        list.push(fixed(named(
            key,
            frame(
                Frame {
                    fill: alpha(ink.fg, 0.07 * hover),
                    radius: [8.; 4],
                    padding: [7., 8., 7., 8.],
                    gap: 9.,
                    row: true,
                    ..Default::default()
                },
                kids,
            ),
        )));
    }
    let n = options.len();
    let foot = frame(
        Frame {
            padding: [6., 12., 6., 12.],
            gap: 4.,
            row: true,
            ..Default::default()
        },
        vec![
            fixed(kbd("↑", ink)),
            fixed(kbd("↓", ink)),
            fixed(tight("select", 10.5, 400, ink.dim, false)),
            gap(2.),
            fixed(kbd("⏎", ink)),
            fixed(tight("run", 10.5, 400, ink.dim, false)),
            gap(2.),
            fixed(kbd("esc", ink)),
            fixed(tight("close", 10.5, 400, ink.dim, false)),
            fill(c::Element::Spacer),
            fixed(tight(
                format!("{n} {}", if n == 1 { "match" } else { "matches" }),
                10.5,
                400,
                ink.dim,
                false,
            )),
        ],
    );
    card(
        vec![
            fixed(named(
                LIST,
                frame(
                    Frame {
                        padding: [4., 6., 4., 6.],
                        ..Default::default()
                    },
                    list,
                ),
            )),
            fixed(hairline(ink)),
            fixed(foot),
        ],
        ink,
    )
}
