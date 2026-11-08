// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The rich face's blocks: one tool call is one card (approval badge, terminal and diff
//! folded in), reasoning is a one-line disclosure, the user speaks in a bubble.
//! Translated from bboxy450-term lab/web/harness.js and lab.css (the web reference the
//! parity script compares against). https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use super::anim::{Clock, Ease};
pub use crate::frontends::face_core::{
    DiffRow, Tool, describe, diff_stat, duration, kilo, minimal_diff,
};
use crate::frontends::face_core::{Tok, Verdict, hit_line, hit_spans, kind_hex, tokens, verdict};
use crate::protocol::*;
use bbt_ui::{self as ui, components as c, modules::seat_row::named};
use bbt_ui_model::draw::{Color as DrawColor, LineCap, LineJoin, PathCommand as P, PathStroke};
use bbt_ui_model::layout::Axis;
use std::sync::Arc;

pub type Rgba = [f32; 4];
/// Theme inks, resolved once per row. Tints mix toward these like CSS color-mix.
#[derive(Clone, Copy)]
pub struct Ink {
    pub fg: Rgba,
    pub dim: Rgba,
    pub surface: Rgba,
    pub bg: Rgba,
    pub seat: Rgba,
    pub ok: Rgba,
    pub bad: Rgba,
    pub ask: Rgba,
}
pub fn hex(s: &str) -> Rgba {
    DrawColor::hex(s).0
}
pub fn ink(ctx: ui::Context) -> Ink {
    let p = ctx.theme.palette();
    let state = |value| {
        let color = hex(value);
        if ctx.theme == ui::Theme::Light {
            mix(color, hex(p.fg), 0.55)
        } else {
            color
        }
    };
    Ink {
        fg: hex(p.fg),
        dim: hex(p.fg_dim),
        surface: hex(p.surface),
        bg: hex(p.bg),
        seat: hex(p.accent),
        ok: state("#2EE6A8"),
        bad: state("#FF5D5D"),
        ask: state("#FFC53D"),
    }
}
/// color-mix(in srgb, a t, b)
pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    [0, 1, 2, 3].map(|i| a[i] * t + b[i] * (1. - t))
}
pub fn alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], c[3] * a]
}
pub fn paint(c: Rgba) -> ui::Paint {
    ui::Paint::solid(ui::Color::Rgba(c.map(|v| v.clamp(0., 1.))))
}
pub fn kind_color(kind: &str) -> Rgba {
    hex(kind_hex(kind))
}
pub fn text(value: impl Into<String>, size: f32, weight: u16, color: Rgba, mono: bool) -> c::Text {
    let mut t = c::Text::new(
        value,
        ui::Text {
            step: ui::TypeStep::Body,
            size: Some(size),
            weight,
            paint: paint(color),
            mono,
            line_height: 1.5,
            ..Default::default()
        },
    );
    t.wrap = true;
    t
}
pub fn line(
    value: impl Into<String>,
    size: f32,
    weight: u16,
    color: Rgba,
    mono: bool,
) -> c::Element {
    let mut t = text(value, size, weight, color, mono);
    t.wrap = false;
    t.ellipsis = true;
    c::Element::Text(t)
}
/// A terminal line (.hx-term): mono at line-height 1.55, as the web reference sets it. The
/// 1.5 of `runs` is 0.6 px short per line, which the old `exit N` line used to mask.
pub fn term_line(parts: Vec<(String, Rgba, u16, bool)>, size: f32, wrap: bool) -> c::Element {
    let mut e = runs(parts, size, true, wrap);
    if let c::Element::Text(t) = &mut e {
        for r in &mut t.runs {
            r.style.line_height = 1.55;
        }
    }
    e
}
pub fn runs(
    parts: Vec<(String, Rgba, u16, bool)>,
    size: f32,
    mono: bool,
    wrap: bool,
) -> c::Element {
    let mut t = text("", size, 400, [0.; 4], mono);
    t.runs = parts
        .into_iter()
        .filter(|(s, ..)| !s.is_empty())
        .map(|(value, color, weight, italic)| c::TextRun {
            value,
            style: ui::Text {
                step: ui::TypeStep::Body,
                size: Some(size),
                weight,
                paint: paint(color),
                mono,
                italic,
                line_height: 1.5,
                ..Default::default()
            },
        })
        .collect();
    if t.runs.is_empty() {
        t.runs.push(c::TextRun {
            value: " ".into(),
            style: ui::Text {
                size: Some(size),
                mono,
                ..Default::default()
            },
        });
    }
    t.wrap = wrap;
    t.preserve_whitespace = true;
    c::Element::Text(t)
}
pub struct Frame {
    pub fill: Rgba,
    pub border: Option<Rgba>,
    pub radius: [f32; 4],
    pub padding: [f32; 4],
    pub gap: f32,
    pub row: bool,
    /// rows stretch children to the row height (rules beside text)
    pub stretch: bool,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            fill: [0.; 4],
            border: None,
            radius: [0.; 4],
            padding: [0.; 4],
            gap: 0.,
            row: false,
            stretch: false,
        }
    }
}
pub fn frame(f: Frame, children: Vec<c::Item>) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.material = Some(ui::Material::Flat);
    style.overrides.fill = Some(paint(f.fill));
    style.overrides.radii = Some(f.radius.map(ui::Length::Px));
    style.overrides.padding = Some(f.padding.map(ui::Length::Px));
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(if f.border.is_some() { 1. } else { 0. }),
        paint: paint(f.border.unwrap_or([0.; 4])),
        outside: false,
    });
    style.overrides.shadow = None;
    let mut b = c::Box::new(style, children);
    b.direction = if f.row { Axis::Row } else { Axis::Column };
    b.gap = ui::Length::Px(f.gap);
    b.align = if f.row && !f.stretch {
        c::Alignment::Center
    } else {
        c::Alignment::Stretch
    };
    c::Element::Box(b)
}
pub fn hstack(children: Vec<c::Item>, gap: f32) -> c::Element {
    frame(
        Frame {
            gap,
            row: true,
            ..Default::default()
        },
        children,
    )
}
pub fn vstack(children: Vec<c::Item>, gap: f32) -> c::Element {
    frame(
        Frame {
            gap,
            ..Default::default()
        },
        children,
    )
}
pub fn fill(e: c::Element) -> c::Item {
    c::Item {
        element: e,
        extent: c::Extent::Content {
            grow: 1.,
            shrink: 1.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    }
}
pub fn fixed(e: c::Element) -> c::Item {
    c::Item {
        element: e,
        extent: c::Extent::Content {
            grow: 0.,
            shrink: 0.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    }
}
fn spacer() -> c::Item {
    c::Item {
        element: c::Element::Spacer,
        extent: c::Extent::Fill,
    }
}
pub fn pill(label: &str, fg: Rgba, bg: Rgba, size: f32) -> c::Element {
    frame(
        Frame {
            fill: bg,
            radius: [999.; 4],
            padding: [1., 7., 1., 7.],
            row: true,
            ..Default::default()
        },
        vec![fixed(line(label, size, 600, fg, false))],
    )
}

// ---------- icons: 16-unit strokes, translated from harness.js ICON ----------
type Poly = Vec<[f32; 2]>;
fn circle(cx: f32, cy: f32, r: f32) -> Vec<P> {
    let k = 0.552_284_8 * r;
    vec![
        P::Move([cx + r, cy]),
        P::Cubic([[cx + r, cy + k], [cx + k, cy + r], [cx, cy + r]]),
        P::Cubic([[cx - k, cy + r], [cx - r, cy + k], [cx - r, cy]]),
        P::Cubic([[cx - r, cy - k], [cx - k, cy - r], [cx, cy - r]]),
        P::Cubic([[cx + k, cy - r], [cx + r, cy - k], [cx + r, cy]]),
    ]
}
fn polys(lines: &[Poly]) -> Vec<P> {
    lines
        .iter()
        .flat_map(|l| {
            l.iter()
                .enumerate()
                .map(|(i, p)| if i == 0 { P::Move(*p) } else { P::Line(*p) })
        })
        .collect()
}
pub fn icon(name: &str, color: Rgba, size: f32, weight: f32) -> c::Element {
    icon_turned(name, color, size, weight, 0.)
}
/// `icon` turned clockwise by `deg` about its centre (the chevron open at 90°).
pub fn icon_turned(name: &str, color: Rgba, size: f32, weight: f32, deg: f32) -> c::Element {
    let k = size / 16.;
    c::Element::Path(c::Path {
        size: [size, size],
        points: Arc::from(icon_points(name, size, deg)),
        stroke: PathStroke {
            width: weight * k,
            cap: LineCap::Round,
            join: LineJoin::Round,
            ..Default::default()
        },
        paint: paint(color),
        fill: None,
        opacity: 1.,
    })
}
/// The glyph's commands in a `size` slot, turned by `deg`: what a motion pass rebuilds
/// when it turns a chevron or spins the think icon.
pub fn icon_points(name: &str, size: f32, deg: f32) -> Vec<P> {
    let k = size / 16.;
    // harness.js: ICON[name] || ICON.other
    let cmds = icon_commands(name)
        .or_else(|| icon_commands("other"))
        .unwrap_or_default();
    let cmds = if deg == 0. {
        cmds
    } else {
        super::stream_motion::turned(&cmds, deg, [8., 8.])
    };
    cmds.into_iter()
        .map(|c| match c {
            P::Move(p) => P::Move([p[0] * k, p[1] * k]),
            P::Line(p) => P::Line([p[0] * k, p[1] * k]),
            P::Cubic(ps) => P::Cubic(ps.map(|p| [p[0] * k, p[1] * k])),
        })
        .collect()
}
/// The 16-unit stroke of a named icon, translated from harness.js `ICON`; None for a name
/// the web has no icon for. The tool kinds are read, search, execute, edit, delete, move,
/// fetch and other, each with its own glyph: fetch is the globe, other the sparkle.
pub fn icon_commands(name: &str) -> Option<Vec<P>> {
    Some(match name {
        "read" => polys(&[
            vec![
                [4., 1.75],
                [9.5, 1.75],
                [12.5, 4.75],
                [12.5, 14.25],
                [4., 14.25],
                [4., 1.75],
            ],
            vec![[9.5, 1.75], [9.5, 4.75], [12.5, 4.75]],
            vec![[6., 8.], [10.5, 8.]],
            vec![[6., 10.5], [10.5, 10.5]],
        ]),
        "search" => {
            let mut v = circle(7., 7., 4.25);
            v.extend(polys(&[vec![[10.25, 10.25], [14., 14.]]]));
            v
        }
        "execute" => polys(&[
            vec![
                [1.75, 2.75],
                [14.25, 2.75],
                [14.25, 13.25],
                [1.75, 13.25],
                [1.75, 2.75],
            ],
            vec![[4.5, 6.5], [6.5, 8.25], [4.5, 10.]],
            vec![[8.5, 10.5], [11.5, 10.5]],
        ]),
        "edit" => polys(&[
            vec![
                [10.5, 2.5],
                [13.5, 5.5],
                [6., 13.],
                [3., 13.],
                [3., 10.],
                [10.5, 2.5],
            ],
            vec![[9., 4.], [12., 7.]],
        ]),
        "think" => polys(&[
            vec![[8., 1.75], [8., 3.75]],
            vec![[8., 12.25], [8., 14.25]],
            vec![[1.75, 8.], [3.75, 8.]],
            vec![[12.25, 8.], [14.25, 8.]],
            vec![[3.6, 3.6], [5., 5.]],
            vec![[11., 11.], [12.4, 12.4]],
            vec![[3.6, 12.4], [5., 11.]],
            vec![[11., 5.], [12.4, 3.6]],
        ]),
        "check" => polys(&[vec![[3.5, 8.5], [6.5, 11.5], [12.5, 4.5]]]),
        "x" => polys(&[
            vec![[4.5, 4.5], [11.5, 11.5]],
            vec![[11.5, 4.5], [4.5, 11.5]],
        ]),
        "shield" => polys(&[
            vec![
                [8., 1.75],
                [13.25, 4.],
                [13.25, 8.],
                [11.5, 12.2],
                [8., 14.25],
                [4.5, 12.2],
                [2.75, 8.],
                [2.75, 4.],
                [8., 1.75],
            ],
            vec![[5.75, 8.], [7.35, 9.6], [10.25, 6.5]],
        ]),
        "shieldx" => polys(&[
            vec![
                [8., 1.75],
                [13.25, 4.],
                [13.25, 8.],
                [11.5, 12.2],
                [8., 14.25],
                [4.5, 12.2],
                [2.75, 8.],
                [2.75, 4.],
                [8., 1.75],
            ],
            vec![[6.25, 6.25], [9.75, 9.75]],
            vec![[9.75, 6.25], [6.25, 9.75]],
        ]),
        "chev" => polys(&[vec![[6., 4.], [10., 8.], [6., 12.]]]),
        "send" => polys(&[
            vec![[8., 13.], [8., 3.]],
            vec![[3.5, 7.5], [8., 3.], [12.5, 7.5]],
        ]),
        // web harness.js icons: plug (ACP agents, /profiles) and stop (/interrupt)
        "plug" => polys(&[
            vec![[6., 2.], [6., 5.]],
            vec![[10., 2.], [10., 5.]],
            vec![
                [4.5, 5.],
                [11.5, 5.],
                [11.5, 7.5],
                [10.5, 9.9],
                [8., 11.],
                [5.5, 9.9],
                [4.5, 7.5],
                [4.5, 5.],
            ],
            vec![[8., 11.], [8., 14.]],
        ]),
        "stop" => polys(&[vec![
            [4.5, 4.5],
            [11.5, 4.5],
            [11.5, 11.5],
            [4.5, 11.5],
            [4.5, 4.5],
        ]]),
        "loop" => {
            // two arcs, each with its arrowhead (web icon 'loop')
            let arc = |cx: f32, cy: f32, from: f32, to: f32| -> Vec<[f32; 2]> {
                (0..=10)
                    .map(|i| {
                        let a = (from + (to - from) * i as f32 / 10.).to_radians();
                        [cx + 4.5 * a.cos(), cy + 4.5 * a.sin()]
                    })
                    .collect()
            };
            polys(&[
                arc(7.5, 7., 180., 326.),
                arc(8.5, 9., 0., 146.),
                vec![[11.5, 2.], [11.5, 4.75], [8.75, 4.75]],
                vec![[4.5, 14.], [4.5, 11.25], [7.25, 11.25]],
            ])
        }
        // the filled stop square of soft receipts: one stroke as wide as the square
        "stopfill" => polys(&[vec![[1.45, 8.], [14.55, 8.]]]),
        "retry" => {
            // M13 8a5 5 0 1 1-1.5-3.55 M13 2.5v3h-3 (web icon 'retry')
            let arc: Vec<[f32; 2]> = (0..=16)
                .map(|i| {
                    let a = (314.6_f32 * i as f32 / 16.).to_radians();
                    [8. + 5. * a.cos(), 8. + 5. * a.sin()]
                })
                .collect();
            polys(&[arc, vec![[13., 2.5], [13., 5.5], [10., 5.5]]])
        }
        "chevdown" => polys(&[vec![[4., 6.], [8., 10.], [12., 6.]]]),
        "branch" => {
            let mut v = circle(4.5, 3.5, 1.5);
            v.extend(circle(4.5, 12.5, 1.5));
            v.extend(circle(11.5, 5.5, 1.5));
            v.extend(polys(&[
                vec![[4.5, 5.], [4.5, 11.]],
                vec![[11.5, 7.], [9., 9.5], [4.5, 11.]],
            ]));
            v
        }
        // the globe: circle r6, the equator, two meridians (M8 2c2 2 2 10 0 12 / c-2 2-2 10 0 12)
        "fetch" => {
            let mut v = circle(8., 8., 6.);
            v.extend([
                P::Move([2., 8.]),
                P::Line([14., 8.]),
                P::Move([8., 2.]),
                P::Cubic([[10., 4.], [10., 12.], [8., 14.]]),
                P::Move([8., 2.]),
                P::Cubic([[6., 4.], [6., 12.], [8., 14.]]),
            ]);
            v
        }
        // the bin: lid, handle, tapered body
        "delete" => polys(&[
            vec![[3., 4.5], [13., 4.5]],
            vec![[6.5, 4.5], [6.5, 3.], [9.5, 3.], [9.5, 4.5]],
            vec![[4.5, 4.5], [5.25, 13.5], [10.75, 13.5], [11.5, 4.5]],
        ]),
        // the arrow right
        "move" => polys(&[
            vec![[2.5, 8.], [13.5, 8.]],
            vec![[10., 4.5], [13.5, 8.], [10., 11.5]],
        ]),
        // the sparkle (eight-point star)
        "other" => polys(&[vec![
            [8., 2.],
            [9.6, 6.4],
            [14., 8.],
            [9.6, 9.6],
            [8., 14.],
            [6.4, 9.6],
            [2., 8.],
            [6.4, 6.4],
            [8., 2.],
        ]]),
        _ => return None,
    })
}
/// A centred icon in a square tile (tool kind, status disc).
pub fn tile(icon_el: c::Element, side: f32, radius: f32, bg: Rgba) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.material = Some(ui::Material::Flat);
    style.overrides.fill = Some(paint(bg));
    style.overrides.radii = Some([ui::Length::Px(radius); 4]);
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(0.),
        paint: paint([0.; 4]),
        outside: false,
    });
    style.overrides.width = Some(ui::Length::Px(side));
    style.overrides.height = Some(ui::Length::Px(side));
    style.overrides.padding = Some([ui::Length::Px(0.); 4]);
    let mut b = c::Box::new(style, vec![spacer(), fixed(icon_el), spacer()]);
    // the Card preset's gap would push the glyph right of centre
    b.gap = ui::Length::Px(0.);
    b.direction = Axis::Row;
    b.align = c::Alignment::Center;
    c::Element::Box(b)
}

// ---------- syntax: the web highlighter's token classes ----------
fn tok_color(tok: Tok, ink: &Ink) -> (Rgba, bool) {
    let tint = |h: &str, t: f32| mix(hex(h), ink.fg, t);
    match tok {
        Tok::Comment => (ink.dim, true),
        Tok::Attr => (tint("#D4A017", 0.75), false),
        Tok::Str => (tint("#2EC46A", 0.7), false),
        Tok::Num => (tint("#FF8A2B", 0.75), false),
        Tok::Macro => (tint("#1FA8E0", 0.75), false),
        Tok::Keyword => (tint("#FF4FA3", 0.72), false),
        Tok::Type => (tint("#9B5CFF", 0.72), false),
        Tok::Plain => (ink.fg, false),
    }
}
pub fn syntax(line: &str, ink: &Ink) -> Vec<(String, Rgba, u16, bool)> {
    let mut out: Vec<(String, Rgba, u16, bool)> = vec![];
    for (s, tok) in tokens(line) {
        let (color, italic) = tok_color(tok, ink);
        match out.last_mut() {
            Some(last) if last.1 == color && last.3 == italic => last.0.push_str(&s),
            _ => out.push((s, color, 400, italic)),
        }
    }
    out
}

fn gutter(n: Option<usize>, ink: &Ink) -> c::Item {
    c::Item {
        element: runs(
            vec![(
                n.map_or(String::new(), |n| n.to_string()),
                alpha(ink.dim, 0.55),
                400,
                false,
            )],
            12.,
            true,
            false,
        ),
        extent: c::Extent::Fixed(30.),
    }
}
fn code_frame(rows: Vec<c::Item>, ink: &Ink) -> c::Element {
    frame(
        Frame {
            fill: alpha(ink.bg, 0.55),
            border: Some(alpha(ink.fg, 0.10)),
            radius: [8.; 4],
            padding: [4., 0., 4., 0.],
            ..Default::default()
        },
        rows,
    )
}
/// A removed line followed by its added twin marks only the span that changed; common
/// prefix and suffix stay plain (web pairMarks). Char ranges [start, end) per row.
fn pair_marks(rows: &[DiffRow]) -> Vec<Option<(usize, usize)>> {
    let mut marks = vec![None; rows.len()];
    for i in 0..rows.len().saturating_sub(1) {
        if rows[i].op != '-'
            || rows[i + 1].op != '+'
            || (i > 0 && rows[i - 1].op == '-')
            || rows.get(i + 2).is_some_and(|r| r.op == '+')
        {
            continue;
        }
        let a: Vec<char> = rows[i].text.chars().collect();
        let b: Vec<char> = rows[i + 1].text.chars().collect();
        let p = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
        let q = a[p..]
            .iter()
            .rev()
            .zip(b[p..].iter().rev())
            .take_while(|(x, y)| x == y)
            .count();
        if p + q < 4 {
            continue;
        }
        marks[i] = Some((p, a.len() - q));
        marks[i + 1] = Some((p, b.len() - q));
    }
    marks
}
/// Diff text with its changed span tinted beneath the glyphs (.hx-diff .chg): the
/// base is blank mono of the same length, the tint and the text are layered over it.
fn marked_line(
    parts: Vec<(String, Rgba, u16, bool)>,
    text: &str,
    span: (usize, usize),
    tint: Rgba,
) -> c::Element {
    use bbt_ui_model::layout::TextMeasure;
    let m = super::view::Metrics::default();
    let style = bbt_ui_model::layout::TextStyle {
        size: 12.,
        weight: 400,
        mono: true,
        italic: false,
        tracking_em: 0.,
    };
    let chars: Vec<char> = text.chars().collect();
    let mark = parts.first().map_or(String::new(), |p| p.0.clone());
    let prefix: String = mark
        .chars()
        .chain(chars[..span.0].iter().copied())
        .collect();
    let changed: String = chars[span.0..span.1].iter().collect();
    let x = m.width(&prefix, &style);
    let w = m.width(&changed, &style);
    let blank = " ".repeat(mark.chars().count() + chars.len());
    let h = (12.0_f32 * 1.5 * 64.).floor() / 64.;
    c::Element::Layers {
        base: Box::new(runs(vec![(blank, [0.; 4], 400, false)], 12., true, false)),
        overlays: vec![
            c::Placed {
                content: c::Element::Rects(c::Rects {
                    size: [w + 2., h - 2.],
                    rects: Arc::from([(
                        bbt_ui_model::layout::Rect::new(0., 0., w + 2., h - 2.),
                        paint(tint),
                    )]),
                    repeat: None,
                }),
                left: x - 1.,
                bottom: 1.,
            },
            c::Placed {
                content: runs(parts, 12., true, false),
                left: 0.,
                bottom: 0.,
            },
        ],
    }
}
pub fn diff_view(key: &str, unified: &str, ink: &Ink) -> c::Element {
    const CTX: usize = 2;
    let mut out: Vec<c::Item> = vec![];
    for rows in minimal_diff(unified) {
        let marks = pair_marks(&rows);
        let near: Vec<bool> = (0..rows.len())
            .map(|i| {
                rows[i.saturating_sub(CTX)..(i + CTX + 1).min(rows.len())]
                    .iter()
                    .any(|r| r.op != ' ')
            })
            .collect();
        let mut skipped = 0;
        let fold = |out: &mut Vec<c::Item>, skipped: &mut usize| {
            if *skipped > 0 {
                out.push(
                    frame(
                        Frame {
                            fill: alpha(ink.fg, 0.04),
                            padding: [1., 0., 1., 64.],
                            ..Default::default()
                        },
                        vec![
                            line(
                                format!(
                                    "⋯ {} unchanged line{}",
                                    skipped,
                                    if *skipped == 1 { "" } else { "s" }
                                ),
                                11.,
                                400,
                                ink.dim,
                                false,
                            )
                            .into(),
                        ],
                    )
                    .into(),
                );
                *skipped = 0;
            }
        };
        for (i, r) in rows.iter().enumerate() {
            if !near[i] {
                skipped += 1;
                continue;
            }
            fold(&mut out, &mut skipped);
            let (bg, mark, mark_c) = match r.op {
                '+' => (alpha(ink.ok, 0.13), "+", ink.ok),
                '-' => (alpha(ink.bad, 0.12), "−", ink.bad),
                _ => ([0.; 4], " ", ink.dim),
            };
            let mut parts = vec![(format!("{mark} "), mark_c, 400, false)];
            parts.extend(syntax(&r.text, ink));
            let text_cell = match marks[i] {
                Some(span) if span.1 > span.0 => {
                    marked_line(parts, &r.text, span, alpha(mark_c, 0.38))
                }
                _ => runs(parts, 12., true, false),
            };
            out.push(
                named(
                    format!("{key}/line/{}", out.len()),
                    frame(
                        Frame {
                            fill: bg,
                            row: true,
                            stretch: true,
                            ..Default::default()
                        },
                        vec![
                            fixed(bar(3., if r.op == ' ' { [0.; 4] } else { mark_c }))
                                .with_width(3.),
                            gutter(r.old, ink),
                            gutter(r.new, ink),
                            fill(text_cell),
                        ],
                    ),
                )
                .into(),
            );
        }
        fold(&mut out, &mut skipped);
    }
    named(key.to_owned(), code_frame(out, ink))
}
trait Width {
    fn with_width(self, w: f32) -> c::Item;
}
impl Width for c::Item {
    fn with_width(mut self, w: f32) -> c::Item {
        self.extent = c::Extent::Fixed(w);
        self
    }
}
pub fn source_view(numbered: &str, limit: usize, ink: &Ink) -> (c::Element, usize) {
    let rows: Vec<(usize, &str)> = numbered
        .lines()
        .enumerate()
        .map(|(i, l)| match l.split_once(": ") {
            Some((n, t)) if n.parse::<usize>().is_ok() => (n.parse().unwrap(), t),
            _ => match l.strip_suffix(':').and_then(|n| n.parse().ok()) {
                Some(n) => (n, ""),
                None => (i + 1, l),
            },
        })
        .collect();
    let total = rows.len();
    let items = rows
        .into_iter()
        .take(limit)
        .map(|(n, t)| {
            hstack(
                vec![
                    gutter(Some(n), ink),
                    fill(runs(syntax(t, ink), 12., true, false)),
                ],
                8.,
            )
            .into()
        })
        .collect();
    (code_frame(items, ink), total)
}
// ---------- Run Python: status chip and emitted values ----------
// Project-owned bboxy450-harness design (structured emit() values as aligned key/value
// rows in the face's token colours). Credit reuse with
// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.
type Part = (String, Rgba, u16, bool);
/// Nesting shown as rows; deeper containers collapse to one inline summary.
const EMIT_DEPTH: usize = 3;
/// Rows and string characters shown before the card's expand control.
const EMIT_ROWS: usize = 40;
const EMIT_CHARS: usize = 96;

/// A Python execution's state for its header, only where the status disc beside it
/// does not already say it: `completed` and `running` add no chip; other states take a
/// chip tinted like the approval badge, then `exit N` when it failed or N ≠ 0.
pub fn python_status(status: &str, exit: Option<i64>, ink: &Ink) -> Option<c::Element> {
    let (c, failed, said) = match status {
        "completed" => (ink.ok, false, true),
        "running" => (ink.seat, false, true),
        "starting" | "cancelling" => (ink.seat, false, false),
        "cancelled" | "interrupted" | "pending" => (ink.dim, false, false),
        "failed" => (ink.bad, true, true),
        _ => (ink.bad, true, false),
    };
    let mut kids = vec![];
    if !said {
        kids.push(fixed(frame(
            Frame {
                fill: alpha(c, 0.13),
                radius: [999.; 4],
                padding: [1., 7., 1., 7.],
                row: true,
                ..Default::default()
            },
            vec![fixed(line(status, 10.5, 600, mix(c, ink.fg, 0.85), false))],
        )));
    }
    if let Some(code) = exit.filter(|code| *code != 0 || failed) {
        kids.push(fixed(line(
            format!("exit {code}"),
            11.5,
            500,
            ink.bad,
            true,
        )));
    }
    (!kids.is_empty()).then(|| hstack(kids, 6.))
}
fn key_ink(ink: &Ink) -> Rgba {
    mix(ink.fg, ink.dim, 0.3)
}
/// `err`/`error` holding anything but null is the failure a person scans for.
fn failure_key(key: &str, value: &serde_json::Value) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "err" | "error" | "errors"
    ) && !value.is_null()
        && value != &serde_json::Value::Bool(false)
}
fn clip(text: &str, limit: usize, cut: &mut bool) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    *cut = true;
    let mut out: String = text.chars().take(limit).collect();
    out.push('\u{2026}');
    out
}
/// One JSON value inline, in the code highlighter's literal colours; `null` dim.
fn inline(value: &serde_json::Value, ink: &Ink, limit: usize, cut: &mut bool, out: &mut Vec<Part>) {
    use serde_json::Value;
    let punct = alpha(ink.dim, 0.9);
    match value {
        Value::Null => out.push(("null".into(), alpha(ink.dim, 0.75), 400, true)),
        Value::Bool(_) | Value::Number(_) => {
            out.push((value.to_string(), tok_color(Tok::Num, ink).0, 400, false))
        }
        Value::String(s) => out.push((
            serde_json::to_string(&clip(s, limit, cut)).unwrap_or_default(),
            tok_color(Tok::Str, ink).0,
            400,
            false,
        )),
        Value::Array(items) => {
            out.push(("[".into(), punct, 400, false));
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push((", ".into(), punct, 400, false));
                }
                inline(item, ink, limit, cut, out);
            }
            out.push(("]".into(), punct, 400, false));
        }
        Value::Object(map) => {
            out.push(("{".into(), punct, 400, false));
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push((", ".into(), punct, 400, false));
                }
                out.push((format!("{k}: "), key_ink(ink), 400, false));
                inline(v, ink, limit, cut, out);
            }
            out.push(("}".into(), punct, 400, false));
        }
    }
}
fn inline_len(value: &serde_json::Value) -> usize {
    serde_json::to_string(value).map_or(usize::MAX, |s| s.chars().count())
}
fn scalar(value: &serde_json::Value) -> bool {
    !value.is_array() && !value.is_object()
}
fn emit_rows(
    value: &serde_json::Value,
    depth: usize,
    full: bool,
    ink: &Ink,
    cut: &mut bool,
    rows: &mut Vec<(Vec<Part>, bool)>,
) {
    use serde_json::Value;
    let entries: Vec<(String, &Value)> = match value {
        Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v)).collect(),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(i, v)| (format!("[{i}]"), v))
            .collect(),
        _ => {
            let mut parts = vec![];
            inline(
                value,
                ink,
                if full { usize::MAX } else { EMIT_CHARS },
                cut,
                &mut parts,
            );
            rows.push((parts, false));
            return;
        }
    };
    if entries.is_empty() {
        rows.push((
            vec![(
                if value.is_array() { "[]" } else { "{}" }.into(),
                ink.dim,
                400,
                false,
            )],
            false,
        ));
        return;
    }
    let pad = entries
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(0)
        .min(24);
    let indent = "  ".repeat(depth);
    for (key, v) in entries {
        let bad = failure_key(&key, v);
        let key_parts = vec![(
            format!("{indent}{key:<pad$}  "),
            if bad { ink.bad } else { key_ink(ink) },
            if bad { 600 } else { 400 },
            false,
        )];
        let short = inline_len(v) <= 56;
        let flat = v.as_array().is_some_and(|a| a.iter().all(scalar)) && short;
        if scalar(v)
            || flat
            || v.as_array().is_some_and(Vec::is_empty)
            || v.as_object().is_some_and(|o| o.is_empty())
            || depth + 1 >= EMIT_DEPTH
        {
            let mut parts = key_parts;
            if !scalar(v) && depth + 1 >= EMIT_DEPTH && !short && !full {
                // Collapsed beyond the depth cap; the expand control shows it inline.
                *cut = true;
                let n = v
                    .as_array()
                    .map_or_else(|| v.as_object().map_or(0, |o| o.len()), Vec::len);
                let label = if v.is_array() {
                    format!("[\u{2026}] {n} items")
                } else {
                    format!("{{\u{2026}}} {n} keys")
                };
                parts.push((label, alpha(ink.dim, 0.85), 400, true));
            } else {
                let start = parts.len();
                inline(
                    v,
                    ink,
                    if full { usize::MAX } else { EMIT_CHARS },
                    cut,
                    &mut parts,
                );
                if bad {
                    for part in &mut parts[start..] {
                        part.1 = mix(ink.bad, ink.fg, 0.8);
                    }
                }
            }
            rows.push((parts, bad));
        } else {
            rows.push((key_parts, bad));
            emit_rows(v, depth + 1, full, ink, cut, rows);
        }
    }
}
/// One `emit()` value as aligned `key  value` rows (nested values indented, capped at
/// `EMIT_DEPTH`). Returns the block and whether anything was shortened; `full` shows
/// every row and character, so no data is reachable only through raw JSON.
pub fn emit_block(value: &serde_json::Value, full: bool, ink: &Ink) -> (c::Element, bool) {
    let mut cut = false;
    let mut rows = vec![];
    emit_rows(value, 0, full, ink, &mut cut, &mut rows);
    let more = if full {
        0
    } else {
        rows.len().saturating_sub(EMIT_ROWS)
    };
    if more > 0 {
        cut = true;
        rows.truncate(EMIT_ROWS);
    }
    let mut items: Vec<c::Item> = rows
        .into_iter()
        .map(|(parts, bad)| {
            let row = term_line(parts, 12., false);
            if bad {
                frame(
                    Frame {
                        fill: alpha(ink.bad, 0.12),
                        radius: [4.; 4],
                        padding: [0., 4., 0., 4.],
                        ..Default::default()
                    },
                    vec![row.into()],
                )
                .into()
            } else {
                frame(
                    Frame {
                        padding: [0., 4., 0., 4.],
                        ..Default::default()
                    },
                    vec![row.into()],
                )
                .into()
            }
        })
        .collect();
    if more > 0 {
        items.push(
            runs(
                vec![(
                    format!("\u{22ef} {more} more rows"),
                    alpha(ink.dim, 0.8),
                    400,
                    true,
                )],
                11.5,
                false,
                false,
            )
            .into(),
        );
    }
    (vstack(items, 0.), cut)
}
/// Rows `emit_block` draws for a value before its expand control.
pub fn emit_row_count(value: &serde_json::Value, ink: &Ink) -> usize {
    let (mut cut, mut rows) = (false, vec![]);
    emit_rows(value, 0, false, ink, &mut cut, &mut rows);
    rows.len().min(EMIT_ROWS + 1)
}
/// The first emitted value on one line for a closed card: `key value  key value`.
pub fn emit_preview(value: &serde_json::Value, ink: &Ink) -> Vec<Part> {
    let mut cut = false;
    let mut out = vec![];
    match value.as_object() {
        Some(map) => {
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(("   ".into(), ink.dim, 400, false));
                }
                let bad = failure_key(k, v);
                out.push((
                    format!("{k} "),
                    if bad { ink.bad } else { key_ink(ink) },
                    if bad { 600 } else { 400 },
                    false,
                ));
                let start = out.len();
                inline(v, ink, 48, &mut cut, &mut out);
                if bad {
                    for part in &mut out[start..] {
                        part.1 = mix(ink.bad, ink.fg, 0.8);
                    }
                }
            }
        }
        None => inline(value, ink, 64, &mut cut, &mut out),
    }
    // Bound the line itself: a closed card never carries a payload's tail.
    let mut budget = 160usize;
    out.retain_mut(|part| {
        if budget == 0 {
            return false;
        }
        let n = part.0.chars().count();
        if n > budget {
            part.0 = part.0.chars().take(budget).collect::<String>() + "\u{2026}";
            budget = 0;
        } else {
            budget -= n;
        }
        true
    });
    out
}

/// Test/compiler output coloured by verdict, as a person scans it.
pub fn term_output(out: &str, ink: &Ink) -> Vec<c::Item> {
    let ok = ink.ok;
    let bad = ink.bad;
    const SHOWN: usize = 14;
    let lines: Vec<&str> = out.trim_matches('\n').lines().collect();
    let more = lines.len().saturating_sub(SHOWN + 2);
    let mut items: Vec<c::Item> = lines[..if more > 0 { SHOWN } else { lines.len() }]
        .iter()
        .map(|l| {
            let color = match verdict(l) {
                Verdict::Ok => ok,
                Verdict::Bad => bad,
                Verdict::Warn => ink.ask,
                Verdict::Dim => ink.dim,
                Verdict::Plain => ink.fg,
            };
            term_line(vec![((*l).to_owned(), color, 400, false)], 12., true).into()
        })
        .collect();
    if more > 0 {
        items.push(
            runs(
                vec![(
                    format!("⋯ {more} more lines"),
                    alpha(ink.dim, 0.8),
                    400,
                    true,
                )],
                11.5,
                false,
                false,
            )
            .into(),
        );
    }
    items
}
pub fn term_output_full(out: &str, ink: &Ink) -> c::Element {
    vstack(
        out.trim_matches('\n')
            .lines()
            .map(|line| {
                let color = match verdict(line) {
                    Verdict::Ok => ink.ok,
                    Verdict::Bad => ink.bad,
                    Verdict::Warn => ink.ask,
                    Verdict::Dim => ink.dim,
                    Verdict::Plain => ink.fg,
                };
                term_line(vec![(line.to_owned(), color, 400, false)], 12., false).into()
            })
            .collect(),
        0.,
    )
}
pub struct Receipt {
    pub ok: bool,
    /// interrupted / cancelled: a neutral receipt with a stop disc, never red
    pub soft: bool,
    pub reason: String,
    pub took: Option<u64>,
    pub tools: usize,
    pub edits: usize,
    pub tokens: Option<(u64, u64)>,
    /// .hx-end .why: the failure's own words on a full-width line under the facts
    pub why: Option<String>,
    /// replies folded into the turn's loop card
    pub folded: usize,
    /// names the Retry / Switch profile buttons `{key}/retry`, `{key}/switch` (error stops)
    pub key: String,
}
/// The turn receipt (.hx-end, harness-spec.md "Tight transcript"): no card. A hairline rule,
/// then one 11.5 px line: the mark, the title, the duration (from 1s, or always when the turn
/// did not end ok), tools and edits (zeros omitted, no rounds), tokens. A failure keeps its
/// red rule, its reason and the Retry / Switch profile actions.
pub fn receipt(r: Receipt, ink: &Ink) -> c::Element {
    let c = if r.ok {
        ink.ok
    } else if r.soft {
        ink.dim
    } else {
        ink.bad
    };
    let mut kids = vec![
        fixed(tile(
            if r.soft {
                stop_glyph(ink.surface)
            } else {
                icon(if r.ok { "check" } else { "x" }, ink.surface, 10., 2.4)
            },
            15.,
            7.5,
            c,
        )),
        fixed(named(
            format!("{}/title", r.key),
            line(r.reason, 12., 650, ink.fg, false),
        )),
    ];
    let mut facts = vec![];
    if let Some(t) = r.took.filter(|t| *t >= 1000 || !r.ok) {
        facts.push(duration(t));
    }
    for (n, w) in [(r.tools, "tool"), (r.edits, "edit")] {
        if n > 0 {
            facts.push(format!("{n} {w}{}", if n == 1 { "" } else { "s" }));
        }
    }
    if let Some((i, o)) = r.tokens.filter(|t| t.0 > 0) {
        facts.push(format!("{} in · {} out", kilo(i), kilo(o)));
    }
    // each fact is named: the motion pass counts its numbers up as the receipt arrives
    kids.extend(facts.into_iter().enumerate().map(|(i, t)| {
        fixed(named(
            format!("{}/fact/{i}", r.key),
            line(t, 11.5, 400, ink.dim, false),
        ))
    }));
    if r.folded > 0 {
        kids.push(fixed(hstack(
            vec![
                fixed(icon("loop", ink.ask, 12., 1.5)),
                fixed(line(
                    format!(
                        "{} repeated repl{} folded",
                        r.folded,
                        if r.folded == 1 { "y" } else { "ies" }
                    ),
                    11.5,
                    400,
                    ink.ask,
                    false,
                )),
            ],
            4.,
        )));
    }
    let facts = frame(
        Frame {
            gap: 10.,
            row: true,
            ..Default::default()
        },
        kids,
    );
    let mut rows = vec![facts.into()];
    if let Some(why) = r.why.filter(|w| !w.is_empty()) {
        let mut t = text(why, 12.5, 400, mix(ink.bad, ink.fg, 0.8), false);
        t.runs[0].style.line_height = 1.4;
        rows.push(c::Element::Text(t).into());
    }
    if !r.ok && !r.soft {
        // .hx-eact: Retry (bad fill, white) and Switch profile (card-hi)
        let button =
            |label: &str, glyph: Option<&str>, fill_c: Rgba, fg: Rgba, edge: Option<Rgba>| {
                let mut kids = vec![];
                if let Some(g) = glyph {
                    kids.push(fixed(icon(g, fg, 12., 1.6)));
                }
                kids.push(fixed(line(label, 12., 600, fg, false)));
                frame(
                    Frame {
                        fill: fill_c,
                        border: edge,
                        radius: [8.; 4],
                        padding: [4., 10., 4., 10.],
                        gap: 6.,
                        row: true,
                        ..Default::default()
                    },
                    kids,
                )
            };
        rows.push(
            hstack(
                vec![
                    fixed(named(
                        format!("{}/retry", r.key),
                        button("Retry", Some("retry"), ink.bad, hex("#ffffff"), None),
                    )),
                    fixed(named(
                        format!("{}/switch", r.key),
                        button(
                            "Switch profile",
                            None,
                            alpha(ink.fg, 0.07),
                            ink.fg,
                            Some(alpha(ink.fg, 0.10)),
                        ),
                    )),
                    spacer(),
                ],
                6.,
            )
            .into(),
        );
    }
    // the rule above: `hair`, or `bad` 35% for a failure (.hx-end border-top)
    let rule = if r.ok || r.soft {
        alpha(ink.fg, 0.10)
    } else {
        alpha(ink.bad, 0.35)
    };
    vstack(
        vec![
            hrule(1., rule).into(),
            frame(
                Frame {
                    padding: [8., 2., 2., 2.],
                    gap: 4.,
                    ..Default::default()
                },
                rows,
            )
            .into(),
        ],
        0.,
    )
}
/// The 9 px stop square of a soft receipt or a stopped tool, in an 11 px icon slot.
/// One butt-capped stroke as wide as the square: filled paths paint offset in this renderer.
pub fn stop_glyph(color: Rgba) -> c::Element {
    let mut e = icon("stopfill", color, 11., 9. * 16. / 11.);
    if let c::Element::Path(p) = &mut e {
        p.stroke.cap = LineCap::Butt;
    }
    e
}
/// The terminal block (.hx-term): the output alone, on 7 px top and 10 px bottom padding.
/// The head names the command and carries a non-zero exit; the body repeats neither
/// (harness-spec.md "Tight transcript"). No output at all draws nothing but the node.
pub fn terminal(key: &str, output: Option<Vec<c::Item>>, ink: &Ink) -> c::Element {
    let Some(out) = output else {
        return named(key.to_owned(), c::Element::Spacer);
    };
    named(
        key.to_owned(),
        frame(
            Frame {
                fill: mix(ink.bg, ink.surface, 0.55),
                border: Some(alpha(ink.fg, 0.07)),
                radius: [8.; 4],
                ..Default::default()
            },
            vec![
                named(
                    format!("{key}/output"),
                    frame(
                        Frame {
                            padding: [7., 10., 10., 10.],
                            ..Default::default()
                        },
                        out,
                    ),
                )
                .into(),
            ],
        ),
    )
}

// ---------- tool calls ----------
#[derive(Clone, Copy)]
pub enum Status {
    Running,
    Asking,
    Done,
    Failed,
    Denied,
    /// interrupted / cancelled: a dim stop disc, default border, not red
    Stopped,
}
/// Three 3px seat dots (.hx-dots): a running card's status, a live thought's tail. At rest
/// they step 1 → .75 → .5; a motion pass bounces them.
pub fn dots(seat: Rgba) -> c::Element {
    hstack(
        [1., 0.75, 0.5]
            .into_iter()
            .map(|a| fixed(dot(3., alpha(seat, a))))
            .collect(),
        2.,
    )
}
pub fn status_disc(st: &Status, ink: &Ink) -> c::Element {
    match st {
        Status::Running => tile(c::Element::Spacer, 18., 9., ink.seat),
        Status::Asking => icon("shield", ink.ask, 14., 1.6),
        Status::Done => tile(icon("check", ink.surface, 11., 2.4), 18., 9., ink.ok),
        Status::Failed | Status::Denied => tile(icon("x", ink.surface, 11., 2.4), 18., 9., ink.bad),
        Status::Stopped => tile(stop_glyph(ink.surface), 18., 9., ink.dim),
    }
}

/// The same centered control reveals command and output details.
pub fn disclosure(open: bool, ink: &Ink) -> c::Element {
    frame(
        Frame {
            row: true,
            ..Default::default()
        },
        vec![
            spacer(),
            fixed(icon_turned(
                "chev",
                ink.seat,
                12.,
                1.8,
                if open { 270. } else { 90. },
            )),
            spacer(),
        ],
    )
}
pub fn badge(
    decision: Option<&ApprovalDecision>,
    by: Option<&Resolver>,
    ink: &Ink,
) -> Option<c::Element> {
    let (name, label, c) = match decision? {
        ApprovalDecision::Deny { .. } => (
            "shieldx",
            if matches!(by, Some(Resolver::Unattended)) {
                "denied · no approver"
            } else {
                "denied"
            },
            ink.bad,
        ),
        // a rule's allow is routine (harness-spec.md "Tight transcript"): no badge; only a
        // person's or the unattended policy's decision shows
        ApprovalDecision::Allow | ApprovalDecision::AllowAlways
            if matches!(by, Some(Resolver::Rule { .. })) =>
        {
            return None;
        }
        ApprovalDecision::AllowAlways => ("shield", "always", ink.ok),
        _ => ("shield", "allowed", ink.ok),
    };
    Some(frame(
        Frame {
            fill: alpha(c, 0.13),
            radius: [999.; 4],
            padding: [1., 7., 1., 5.],
            gap: 4.,
            row: true,
            ..Default::default()
        },
        vec![
            fixed(icon(name, mix(c, ink.fg, 0.85), 12., 1.5)),
            fixed(line(label, 10.5, 600, mix(c, ink.fg, 0.85), false)),
        ],
    ))
}
pub struct Head {
    pub key: String,
    pub tool_kind: &'static str,
    pub verb: String,
    pub target: String,
    pub mono: bool,
    /// `.hx-where`: where a Search or Find looked, read `in <path>` right after the target
    pub place: Option<String>,
    pub stat: Option<(usize, usize)>,
    pub meta: Option<(String, Option<bool>)>,
    pub badge: Option<c::Element>,
    pub dur: Option<String>,
    pub status: Status,
}
/// One deterministic display line; exact source remains in the tool details.
pub fn compact_heading(value: &str, limit: usize) -> String {
    let first = value.lines().next().unwrap_or_default();
    let clean = first.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = clean.chars().take(limit).collect();
    if clean.chars().count() > limit || value.lines().count() > 1 {
        out.push('…');
    }
    out
}
pub fn head(h: Head, ink: &Ink) -> c::Element {
    let kc = kind_color(h.tool_kind);
    let mut kids = vec![
        fixed(tile(
            icon(h.tool_kind, kc, 13., 1.5),
            20.,
            6.,
            alpha(kc, 0.16),
        )),
        // The verb names the card; a long target ellipsises before it does.
        c::Item {
            element: line(compact_heading(&h.verb, 40), 13., 650, ink.fg, false),
            extent: c::Extent::Content {
                grow: 0.,
                shrink: 0.,
                min: 0.,
                max: Some(320.),
                max_fraction: None,
            },
        },
        // .hx-tgt is content-sized and ellipsised (min-width 0); .hx-sp takes the slack
        c::Item {
            element: line(
                compact_heading(&h.target, 80),
                if h.mono { 12. } else { 13. },
                400,
                alpha(ink.fg, 0.82),
                h.mono,
            ),
            extent: c::Extent::Content {
                grow: 0.,
                shrink: 1.,
                min: 0.,
                max: None,
                max_fraction: None,
            },
        },
    ];
    if let Some(p) = h.place {
        kids.push(fixed(line(
            compact_heading(&format!("in {p}"), 36),
            12.,
            400,
            ink.dim,
            true,
        )));
    }
    // Prose and monospace text share a baseline; the icon and status stay centered.
    let mut title = hstack(kids.drain(1..).collect(), 8.);
    if let c::Element::Box(row) = &mut title {
        row.align = c::Alignment::Baseline;
    }
    kids.push(c::Item {
        element: title,
        extent: c::Extent::Content {
            grow: 0.,
            shrink: 1.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    });
    kids.push(spacer());
    if let Some((a, d)) = h.stat {
        kids.push(fixed(runs(
            vec![
                (format!("+{a} "), ink.ok, 600, false),
                (format!("−{d}"), ink.bad, 600, false),
            ],
            11.5,
            true,
            false,
        )));
    }
    if let Some((m, good)) = h.meta {
        let c = match good {
            Some(true) => ink.ok,
            Some(false) => ink.bad,
            None => ink.dim,
        };
        kids.push(fixed(line(compact_heading(&m, 24), 11.5, 400, c, false)));
    }
    if let Some(b) = h.badge {
        kids.push(fixed(b));
    }
    // .hx-dur: a 32 px right-aligned slot, filled only from 1s (the callers pass none
    // below that; harness-spec.md "Tight transcript"). No duration, no empty slot.
    if let Some(dur) = &h.dur {
        kids.push(
            c::Item::from(hstack(
                vec![
                    spacer(),
                    fixed(line(compact_heading(dur, 12), 11., 400, ink.dim, false)),
                ],
                0.,
            ))
            .with_width(32.),
        );
    }
    let status_key = h.key.replacen("toolhead/", "tool/", 1) + "/status";
    kids.push(fixed(named(status_key, status_disc(&h.status, ink))));
    named(
        h.key,
        frame(
            Frame {
                // Compact tool header; status changes keep the same height.
                padding: [4., 10., 4., 10.],
                gap: 8.,
                row: true,
                ..Default::default()
            },
            kids,
        ),
    )
}
pub fn card(status: &Status, children: Vec<c::Item>, ink: &Ink) -> c::Element {
    let (fill_c, border) = match status {
        Status::Failed | Status::Denied => (alpha(ink.bad, 0.06), alpha(ink.bad, 0.40)),
        Status::Asking => (alpha(ink.fg, 0.04), alpha(ink.ask, 0.60)),
        Status::Running => (alpha(ink.fg, 0.04), alpha(ink.seat, 0.70)),
        _ => (alpha(ink.fg, 0.04), alpha(ink.fg, 0.10)),
    };
    let mut card = frame(
        Frame {
            fill: fill_c,
            border: Some(border),
            radius: [11.; 4],
            ..Default::default()
        },
        children,
    );
    if let c::Element::Box(panel) = &mut card {
        panel.clip = true;
    }
    card
}
pub fn body(children: Vec<c::Item>) -> c::Item {
    body_el(children).into()
}
/// The card body (.hx-tb): what a fold reveals.
pub fn body_el(children: Vec<c::Item>) -> c::Element {
    frame(
        Frame {
            padding: [0., 10., 8., 10.],
            gap: 6.,
            ..Default::default()
        },
        children,
    )
}
/// `.hx-hits`: one row per hit, `file:ln` then the text 10px on, each match in a `mark`
/// pill (search purple 35%, radius 3, padding 0 2) named `{key}/{row}/mark/{k}`.
pub fn hits(key: &str, out: &str, pattern: &str, cwd: &str, ink: &Ink) -> c::Element {
    let re = regex::Regex::new(pattern).ok();
    let mono = |value: String, color: Rgba| {
        let mut t = text(value, 12., 400, color, true);
        t.wrap = false;
        t.ellipsis = true;
        t.preserve_whitespace = true;
        c::Element::Text(t)
    };
    let shrink = |e: c::Element| c::Item {
        element: e,
        extent: c::Extent::Content {
            grow: 0.,
            shrink: 1.,
            min: 0.,
            max: None,
            max_fraction: None,
        },
    };
    let rows = out
        .lines()
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, l)| {
            let (f, n, t) = hit_line(l);
            let f = f.strip_prefix(&format!("{cwd}/")).unwrap_or(f);
            let mut kids = vec![fixed(runs(
                vec![
                    (f.to_owned(), ink.fg, 400, false),
                    (
                        n.map(|n| format!(":{n}")).unwrap_or_default(),
                        ink.seat,
                        400,
                        false,
                    ),
                ],
                12.,
                true,
                false,
            ))];
            let t = t.trim();
            if !t.is_empty() {
                let mut marks = 0;
                let spans = hit_spans(t, re.as_ref())
                    .into_iter()
                    .map(|(span, hit)| {
                        if hit {
                            let k = marks;
                            marks += 1;
                            fixed(named(
                                format!("{key}/{i}/mark/{k}"),
                                frame(
                                    Frame {
                                        fill: alpha(hex("#B07BFF"), 0.35),
                                        radius: [3.; 4],
                                        padding: [0., 2., 0., 2.],
                                        row: true,
                                        ..Default::default()
                                    },
                                    vec![fixed(mono(span, ink.fg))],
                                ),
                            ))
                        } else {
                            shrink(mono(span, ink.dim))
                        }
                    })
                    .collect();
                kids.push(shrink(hstack(spans, 0.)));
            }
            frame(
                Frame {
                    fill: alpha(ink.fg, 0.04),
                    radius: [6.; 4],
                    padding: [3., 8., 3., 8.],
                    gap: 10.,
                    row: true,
                    ..Default::default()
                },
                kids,
            )
            .into()
        })
        .collect();
    vstack(rows, 2.)
}

// ---------- parallel tool calls: harness.js mount() puts consecutive tool cards on one rail ----------
/// `.hx-group::after`: "N CALLS · ONE ROUND", 10px 600 +0.06em, seat 70% over fg-dim, in the
/// 15px line box that sits on the group's top edge.
pub fn group_caption(key: &str, n: usize, ink: &Ink) -> c::Element {
    let mut t = c::Text::new(
        format!("{n} CALLS · ONE ROUND"),
        ui::Text {
            step: ui::TypeStep::Body,
            size: Some(10.),
            weight: 600,
            paint: paint(mix(ink.seat, ink.dim, 0.7)),
            tracking_em: 0.06,
            line_height: 1.5,
            ..Default::default()
        },
    );
    t.wrap = false;
    named(key.to_owned(), c::Element::Text(t))
}
/// `.hx-group`: a member card inset 14px behind a 2px seat rail 4px in from the group's left
/// edge; the rail starts 8px into the first card and ends 8px before the last. The 6px member
/// gap is inside the rail column so the rail runs unbroken. The web's seat → seat 15% gradient
/// down the group is one flat step per member here. The caption sits on the group's left edge:
/// the row region's 4px inset (the smoke's inset-card rule) leaves no room for the web's -2px.
pub fn group_member(card: c::Element, index: usize, n: usize, ink: &Ink) -> c::Element {
    let (first, last) = (index == 0, index + 1 == n);
    let a = 1. - 0.85 * (index as f32 + 0.5) / n as f32;
    let rail = frame(
        Frame {
            padding: [
                if first { 8. } else { 0. },
                8.,
                if last { 8. } else { 0. },
                TIMELINE_CENTER - 1.,
            ],
            ..Default::default()
        },
        vec![c::Item {
            element: bar(2., alpha(ink.seat, a)),
            extent: c::Extent::Fill,
        }],
    );
    let gap = |h: f32| c::Item {
        element: c::Element::Spacer,
        extent: c::Extent::Fixed(h),
    };
    let mut column = vec![];
    if !first {
        column.push(gap(3.));
    }
    column.push(card.into());
    if !last {
        column.push(gap(3.));
    }
    frame(
        Frame {
            row: true,
            stretch: true,
            ..Default::default()
        },
        vec![fixed(rail).with_width(14.), fill(vstack(column, 0.))],
    )
}

// ---------- conversation blocks ----------
/// The source toggle preserves selectable exact input Markdown beside its rich form.
pub fn prompt_content(
    body: c::Element,
    id: ItemId,
    source: bool,
    raw: &str,
    ink: &Ink,
) -> (c::Element, c::Element) {
    let content = if source {
        let mut t = text(raw, 13.5, 400, ink.fg, true);
        t.preserve_whitespace = true;
        c::Element::Text(t)
    } else {
        body
    };
    // The toggle rides the caption row: no line of its own under the text.
    let toggle = named(
        format!("source/{id}/toggle"),
        line(
            if source { "formatted" } else { "source" },
            10.5,
            500,
            ink.dim,
            true,
        ),
    );
    (content, toggle)
}
/// Shared bubble chrome; Markdown is prepared off the UI thread by Preparer.
pub fn prompt_bubble(
    body: c::Element,
    caption: &str,
    toggle: Option<c::Element>,
    pending: bool,
    ink: &Ink,
) -> c::Element {
    let caption = line(
        caption,
        10.5,
        600,
        mix(ink_accent(pending, ink), ink.fg, 0.7),
        false,
    );
    let header = match toggle {
        Some(toggle) => hstack(vec![caption.into(), spacer(), toggle.into()], 6.),
        None => caption,
    };
    let accent = ink_accent(pending, ink);
    let bubble = frame(
        Frame {
            fill: mix(accent, ink.surface, 0.2),
            border: Some(alpha(accent, 0.35)),
            radius: [14., 14., 4., 14.],
            padding: [6., 12., 7., 12.],
            gap: 1.,
            ..Default::default()
        },
        vec![header.into(), body.into()],
    );
    hstack(
        vec![c::Item {
            element: bubble,
            extent: c::Extent::Content {
                grow: 1.,
                shrink: 1.,
                min: 0.,
                max: None,
                max_fraction: None,
            },
        }],
        0.,
    )
}
fn ink_accent(pending: bool, ink: &Ink) -> Rgba {
    if pending { ink.dim } else { ink.seat }
}
/// The runaway loop card (harness-spec.md): ask 30% border, ask 5% fill, radius 11.
pub fn loop_card(children: Vec<c::Item>, ink: &Ink) -> c::Element {
    frame(
        Frame {
            fill: alpha(ink.ask, 0.05),
            border: Some(alpha(ink.ask, 0.30)),
            radius: [11.; 4],
            ..Default::default()
        },
        children,
    )
}
/// Its header reads like a tool header: ask ↻ tile, title, counts, a `folded` chip, chevron.
pub fn loop_head(key: &str, counts: &str, open: bool, ink: &Ink) -> c::Element {
    named(
        key.to_owned(),
        frame(
            Frame {
                padding: [7., 10., 7., 10.],
                gap: 8.,
                row: true,
                ..Default::default()
            },
            vec![
                fixed(tile(
                    icon("loop", ink.ask, 14., 1.5),
                    24.,
                    7.,
                    alpha(ink.ask, 0.16),
                )),
                fixed(line(
                    "The agent kept repeating itself",
                    13.,
                    650,
                    ink.fg,
                    false,
                )),
                fill(line(counts, 12., 400, ink.dim, false)),
                fixed(frame(
                    Frame {
                        fill: alpha(ink.fg, 0.07),
                        border: Some(alpha(ink.fg, 0.10)),
                        radius: [999.; 4],
                        padding: [0., 6., 0., 6.],
                        row: true,
                        ..Default::default()
                    },
                    vec![fixed(line("folded", 10.5, 400, ink.dim, false))],
                )),
                fixed(named(
                    format!("{key}/chev"),
                    icon_turned(
                        "chev",
                        alpha(ink.dim, 0.6),
                        12.,
                        1.5,
                        if open { 90. } else { 0. },
                    ),
                )),
            ],
        ),
    )
}
/// The open loop body: an ask 25% rule on top, padding 10 12 12 22.
pub fn loop_body(child: c::Element, ink: &Ink) -> c::Element {
    vstack(
        vec![
            c::Item {
                element: c::Element::Rects(c::Rects {
                    size: [1., 1.],
                    rects: Arc::from([(
                        bbt_ui_model::layout::Rect::new(0., 0., 1., 1.),
                        paint(alpha(ink.ask, 0.25)),
                    )]),
                    repeat: Some((Axis::Row, 1.)),
                }),
                extent: c::Extent::Fixed(1.),
            },
            frame(
                Frame {
                    padding: [10., 12., 12., 22.],
                    ..Default::default()
                },
                vec![child.into()],
            )
            .into(),
        ],
        0.,
    )
}
const TIMELINE_CENTER: f32 = 5.;
pub fn thought_row(
    key: &str,
    live: bool,
    took: Option<u64>,
    gist: &str,
    open: bool,
    ink: &Ink,
) -> c::Element {
    // harness-spec.md "Tight transcript": a thought under 1s has no label, only its gist
    let label = if live {
        Some("Thinking".to_owned())
    } else {
        took.filter(|ms| *ms >= 1000)
            .map(|ms| format!("Thought for {}", duration(ms)))
    };
    // Reasoning uses the same running/completed indicator as tool calls.
    let mut kids = vec![fixed(c::Element::Layers {
        base: Box::new(meter_bar(14., 18., [0.; 4])),
        overlays: vec![c::Placed {
            content: named(
                format!("{key}/icon"),
                status_disc(&if live { Status::Running } else { Status::Done }, ink),
            ),
            left: TIMELINE_CENTER - 9.,
            bottom: 0.,
        }],
    })];
    if let Some(label) = label {
        kids.push(fixed(named(
            format!("{key}/label"),
            line(label, 12., 650, ink.fg, false),
        )));
    }
    if !live {
        // .hx-gist: ellipsised, the chevron right after it (.hx-tk is inline flex); the
        // view hides the chevron until the row is hovered or open
        kids.push(c::Item {
            element: line(gist, 12., 400, alpha(ink.dim, 0.75), false),
            extent: c::Extent::Content {
                grow: 0.,
                shrink: 1.,
                min: 0.,
                max: None,
                max_fraction: None,
            },
        });
        kids.push(fixed(named(
            format!("{key}/chev"),
            icon_turned(
                "chev",
                alpha(ink.dim, 0.6),
                12.,
                1.5,
                if open { 90. } else { 0. },
            ),
        )));
    }
    kids.push(spacer());
    named(key.to_owned(), hstack(kids, 6.))
}
pub fn thought_body(child: c::Element, ink: &Ink) -> c::Element {
    let gap = 14.;
    frame(
        Frame {
            gap,
            padding: [0., 0., 0., TIMELINE_CENTER - 1.],
            row: true,
            stretch: true,
            ..Default::default()
        },
        vec![
            fixed(named("reasoning/rail", bar(2., alpha(ink.seat, 0.45)))).with_width(2.),
            fill(child),
        ],
    )
}
pub fn notice(text_value: &str, level: &Level, ink: &Ink) -> c::Element {
    let color = match level {
        Level::Warn => ink.ask,
        Level::Error => ink.bad,
        _ => ink.dim,
    };
    // .hx-notice: the dot centred on the first line, long notices wrap beneath
    let mut t = text(text_value, 11.5, 400, ink.dim, false);
    t.runs[0].style.line_height = 1.4;
    frame(
        Frame {
            gap: 7.,
            row: true,
            stretch: true,
            ..Default::default()
        },
        vec![
            fixed(frame(
                Frame {
                    padding: [5., 0., 0., 0.],
                    ..Default::default()
                },
                vec![hstack(vec![fixed(dot(6., color))], 0.).into()],
            )),
            fill(c::Element::Text(t)),
        ],
    )
}

/// What the header says on its right: one phase, coloured by tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Idle,
    Live,
    Ask,
}
/// An ACP session's model string, `acp:<command> · <model>`, as (command, model).
pub fn acp_model(model: &str) -> Option<(String, String)> {
    let rest = model.strip_prefix("acp:")?;
    let (agent, model) = rest.split_once('·').unwrap_or((rest, ""));
    Some((agent.trim().to_owned(), model.trim().to_owned()))
}
/// An agent's tile: mark text and colour, by the first word of its command (harness.js AGENTS).
pub fn agent_mark(command: &str) -> (String, Rgba) {
    let c = command.split_whitespace().next().unwrap_or("");
    let (m, h) = match c {
        "claude-agent-acp" => ("C", "#D97757"),
        "codex-acp" => ("Cx", "#10A37F"),
        "opencode" => ("oc", "#5B6178"),
        "pi-acp" => ("π", "#FF4FA3"),
        "omp" => ("ω", "#B07BFF"),
        "goose" => ("G", "#C98A1B"),
        _ => {
            let first = c
                .chars()
                .next()
                .map_or("·".into(), |f| f.to_uppercase().to_string());
            return (first, hex("#2EE6A8"));
        }
    };
    (m.into(), hex(h))
}
/// The `ACP` tag: 9.5 bold on the seat-14 kind fill.
pub fn acp_tag(ink: &Ink) -> c::Element {
    frame(
        Frame {
            fill: mix(ink.seat, ink.surface, 0.14),
            radius: [4.; 4],
            padding: [0., 5., 0., 5.],
            row: true,
            ..Default::default()
        },
        vec![fixed(line(
            "ACP",
            9.5,
            700,
            mix(ink.ok, ink.fg, 0.85),
            false,
        ))],
    )
}

/// The session bar: mark, name, profile chip (`profile`, opens the switcher), folder chip, and the live
/// phase. Interrupt and stop live in the composer (esc, the stop button, `/interrupt`,
/// `/stop`); the header carries no controls.
pub fn header(
    profile: Option<(String, String)>,
    folder: Option<String>,
    status: &str,
    tone: Tone,
    menu_open: bool,
    usage: (u64, u64),
    ink: &Ink,
) -> c::Element {
    let chip = |kids: Vec<c::Item>| {
        frame(
            Frame {
                fill: alpha(ink.fg, 0.04),
                radius: [999.; 4],
                padding: [1., 8., 1., 8.],
                gap: 5.,
                row: true,
                ..Default::default()
            },
            kids,
        )
    };
    // the identity portrait at 20px, radius 6, inside a 1px seat-55 ring
    let mark = frame(
        Frame {
            fill: alpha(ink.seat, 0.55),
            radius: [7.; 4],
            padding: [1.; 4],
            ..Default::default()
        },
        vec![fixed(c::Element::Image {
            asset: super::assets::MARK.into(),
            size: ui::Length::Px(20.),
            radius: ui::Length::Px(6.),
        })],
    );
    let mut kids = vec![fixed(mark), fixed(line("bboxy", 13., 700, ink.fg, false))];
    if let Some((p, m)) = profile {
        let mut parts = vec![];
        let text_c = if menu_open { ink.fg } else { ink.dim };
        if let Some((agent, model)) = acp_model(&m) {
            let (mark, color) = agent_mark(&agent);
            parts.push(fixed(tile(
                runs(vec![(mark, hex("#ffffff"), 800, false)], 8.5, false, false),
                18.,
                9.,
                color,
            )));
            let mut t = vec![(if p.is_empty() { agent } else { p }, text_c, 400, false)];
            if !model.is_empty() {
                t.push((format!(" · {model}"), alpha(ink.dim, 0.75), 400, false));
            }
            parts.push(fixed(runs(t, 11.5, false, false)));
            parts.push(fixed(acp_tag(ink)));
        } else {
            parts.push(fixed(dot(6., ink.seat)));
            let label = if p.is_empty() || p == m {
                m
            } else {
                format!("{p} · {m}")
            };
            parts.push(fixed(line(label, 11.5, 400, text_c, false)));
        }
        parts.push(fixed(down_chevron(alpha(ink.dim, 0.7))));
        // web .hx-model: the chip opens the profile switcher; open, it takes the seat
        kids.push(fixed(named(
            "profile",
            frame(
                Frame {
                    fill: if menu_open {
                        alpha(ink.seat, 0.1)
                    } else {
                        alpha(ink.fg, 0.04)
                    },
                    border: Some(if menu_open {
                        alpha(ink.seat, 0.55)
                    } else {
                        [0.; 4]
                    }),
                    radius: [999.; 4],
                    padding: [1., 8., 1., 8.],
                    gap: 5.,
                    row: true,
                    ..Default::default()
                },
                parts,
            ),
        )));
    }
    if let Some(f) = folder {
        kids.push(fixed(chip(vec![
            fixed(icon("branch", ink.dim, 12., 1.5)),
            fixed(line(f, 11.5, 400, ink.dim, false)),
        ])));
    }
    kids.push(spacer());
    let (dot_c, text_c) = match tone {
        Tone::Idle => (ink.dim, ink.dim),
        Tone::Live => (ink.seat, ink.fg),
        Tone::Ask => (ink.ask, ink.ask),
    };
    kids.push(fixed(dot(7., dot_c)));
    let say: String = if status.chars().count() > 48 {
        status.chars().take(47).chain(['…']).collect()
    } else {
        status.to_owned()
    };
    kids.push(fixed(named("phase", line(say, 11.5, 400, text_c, false))));
    // .hx-meter: the usage items' sum (harness-spec.md "Refinement pass 2"), shown once
    // input tokens arrive: a 38×4 track filled by input / 2000 %, then the in + out total
    // at 11 `fg-dim`. A 0/0 total shows no meter. Flat: solid seat for the web's gradient.
    let (tokens_in, tokens_out) = usage;
    if tokens_in > 0 {
        let full = (tokens_in as f32 / 200_000.).min(1.);
        kids.push(fixed(named(
            "meter",
            hstack(
                vec![
                    fixed(c::Element::Layers {
                        base: Box::new(meter_bar(38., 4., alpha(ink.fg, 0.07))),
                        overlays: vec![c::Placed {
                            content: meter_bar((38. * full).round().max(4.), 4., ink.seat),
                            left: 0.,
                            bottom: 0.,
                        }],
                    }),
                    fixed(line(
                        kilo(tokens_in.saturating_add(tokens_out)),
                        11.,
                        400,
                        ink.dim,
                        false,
                    )),
                ],
                6.,
            ),
        )));
    }
    frame(
        Frame {
            fill: alpha(ink.seat, 0.06),
            padding: [8., 12., 8., 12.],
            gap: 8.,
            row: true,
            ..Default::default()
        },
        kids,
    )
}
/// A solid `w`×`h` bar with round ends (the meter's track and its fill).
pub fn meter_bar(w: f32, h: f32, color: Rgba) -> c::Element {
    let mut style = ui::Style::new(ui::Preset::Card);
    style.overrides.material = Some(ui::Material::Flat);
    style.overrides.fill = Some(paint(color));
    style.overrides.radii = Some([ui::Length::Px(h / 2.); 4]);
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(0.),
        paint: paint([0.; 4]),
        outside: false,
    });
    style.overrides.width = Some(ui::Length::Px(w));
    style.overrides.height = Some(ui::Length::Px(h));
    style.overrides.padding = Some([ui::Length::Px(0.); 4]);
    style.overrides.shadow = None;
    c::Element::Box(c::Box::new(style, vec![]))
}

/// Composer states from harness-spec.md (Composer table): the session phase decides `run`
/// and `ask`; otherwise the draft and focus decide `typing`, `focus` or `idle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerState {
    Idle,
    Focus,
    Typing,
    Run,
    Ask,
}
impl ComposerState {
    pub fn of(live: bool, ask: bool, text: bool, focused: bool) -> Self {
        if ask {
            Self::Ask
        } else if live {
            Self::Run
        } else if text {
            Self::Typing
        } else if focused {
            Self::Focus
        } else {
            Self::Idle
        }
    }
}

/// What the session and the window tell the composer: a running turn, a waiting approval,
/// a session whose last turn failed, keyboard focus, and the focus swell's envelope.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComposerPhase {
    pub live: bool,
    pub ask: bool,
    /// The last turn ended in an error and nothing runs: the rim reads Errored.
    pub errored: bool,
    pub focused: bool,
    /// The focus swell this frame, 0..1 (rim-glow §7): `sin(π t / swell_ms)` from the moment
    /// focus arrived, 0 at rest. The view computes it on the window clock.
    pub swell: f32,
}

/// The composer rim's activity (harness-spec.md Composer, alive rim): `run` → Active,
/// `ask` → Waiting (amber), error → Errored, everything else → Idle. The same mapping
/// `bbt_ui::modules::pane` takes from a seat's `Activity`. scripts/test-chrome-control.py
/// flattens it and requires tests/gui_chrome.rs to go red.
pub fn rim_activity(live: bool, ask: bool, errored: bool) -> ui::GlowActivity {
    if ask {
        ui::GlowActivity::Waiting
    } else if live {
        ui::GlowActivity::Active
    } else if errored {
        ui::GlowActivity::Errored
    } else {
        ui::GlowActivity::Idle
    }
}
/// The focus swell's whole life, design `glow.swell_ms` (900 ms).
pub const SWELL_MS: f32 = bbt_ui_model::TOKENS.glow.swell_ms as f32;
/// The composer rim's shader phase seed; one composer, one constant.
pub const RIM_SEED: u32 = 7;
/// The focus swell's envelope at progress `p` (0..1 of `SWELL_MS`): one half sine,
/// `reach(t) = swell · sin(π t / swell_ms)` (design tokens, glow). 0 once it has run.
pub fn swell_envelope(p: f32) -> f32 {
    if (0. ..1.).contains(&p) {
        (std::f32::consts::PI * p).sin().clamp(0., 1.)
    } else {
        0.
    }
}

/// The composer: one rounded box with the draft, two quiet chips, a state hint and the round
/// send button. Translated from harness.js composer() and lab.css .hx-comp/.hx-box/.hx-send
/// (bboxy450-term lab/web, harness-spec.md; https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md).
/// Node names keep the toolkit composer's hit contract (`composer`, `composer/text`,
/// `composer/body-viewport`, `composer/send`, `composer/attach`, `composer/priority`,
/// `composer/address`), so `composer::Regions` still drives clicks, the caret and the IME.
///
/// Motion (harness-spec.md Motion table) reads through `anim`: the focus fill over 200 ms
/// `std`, the box height over 160 ms `out`, the send fill and icon colour across
/// idle → ready → stop over 200 ms `std` with an arrow ↔ square crossfade. The box wears
/// the term's alive pane rim (`Preset::PaneGlow` with `overrides.rim`, exactly as
/// `bbt_ui::modules::pane` does): activity from the phase, the focus swell from `phase`,
/// a constant seat-35 border; the rim replaces the top-edge sweep and the focus ring.
/// A settled clock answers the end states, so the static look is unchanged.
///
/// `zoom` scales every text size and line height (harness-spec.md Standalone window).
pub fn composer(
    view: &ui::modules::composer::Composer,
    phase: ComposerPhase,
    ctx: ui::Context,
    width: f32,
    zoom: f32,
    m: &dyn c::Measure,
    anim: &mut Clock,
) -> Result<c::Element, String> {
    use ComposerState as S;
    use bbt_ui::widgets::text_area::TextArea;
    use bbt_ui_model::layout::Rect;
    let ComposerPhase {
        live,
        ask,
        errored,
        focused,
        swell,
    } = phase;
    let zoom = if zoom.is_finite() && zoom > 0. {
        zoom
    } else {
        1.
    };
    let line_h = 20. * zoom;
    const MAX_LINES: f32 = 8.;
    let ink = ink(ctx);
    let has_text = !view.text.is_empty();
    let state = ComposerState::of(live, ask, has_text, focused);
    let box_w = (width - 24.).clamp(1., 860.);
    let side = ((width - box_w) / 2.).max(0.);
    let inner = (box_w - 22.).max(1.);
    let empty = !has_text && view.edit.as_ref().is_none_or(|e| e.preedit.is_none());
    // state tweens: 0..1 each, retargeting from wherever they are
    let focus_t = anim.tween(
        "composer/focus",
        f32::from(u8::from(focused)),
        200.,
        Ease::Std,
    );
    let ready_t = anim.tween(
        "composer/send/ready",
        f32::from(u8::from(has_text)),
        200.,
        Ease::Std,
    );
    let stop_now = state == S::Run && !has_text;
    let stop_t = anim.tween(
        "composer/send/stop",
        f32::from(u8::from(stop_now)),
        200.,
        Ease::Std,
    );
    let ph_ink = if state == S::Ask {
        alpha(ink.ask, 0.8)
    } else {
        alpha(ink.dim, 0.7)
    };
    let mut body = TextArea {
        value: view.text.clone(),
        placeholder: match state {
            S::Run => "Queue a follow-up…",
            S::Ask => "Answer the approval above",
            _ => "Ask bboxy, @ a file, / a command",
        }
        .into(),
        style: ui::Text {
            size: Some(13. * zoom),
            paint: paint(if empty { ph_ink } else { ink.fg }),
            line_height: 20. / 13.,
            ..Default::default()
        },
        edit: (view.focus && focused).then(|| view.edit.clone().unwrap_or_default()),
        min_lines: 1,
        wrap: true,
        name: "composer/text".into(),
    }
    .element(inner, ctx, m)?;
    if empty {
        tint_placeholder(&mut body, paint(ph_ink));
    }
    // Grow with the draft up to MAX_LINES, then scroll so the caret stays in view. The box
    // height tweens over 160 ms; the caret stays in view at every intermediate height.
    let measured = body.layout(Rect::new(0., 0., inner, 0.), ctx, m)?;
    let target_h = measured.rect.h.clamp(line_h, line_h * MAX_LINES);
    let height = anim.tween("composer/height", target_h, 160., Ease::Out);
    fn caret(n: &c::Layout) -> Option<Rect> {
        if matches!(&n.kind, c::LayoutKind::Named(k) if k == "composer/text/caret") {
            return Some(n.rect);
        }
        n.children.iter().find_map(caret)
    }
    let bottom = caret(&measured).map_or(measured.rect.h, |r| r.bottom());
    let offset = view
        .body_scroll
        .unwrap_or((bottom - height).max(0.))
        .clamp(0., (measured.rect.h - height).max(0.));
    let viewport = named(
        "composer/body-viewport",
        c::Element::ScrollView(c::ScrollView {
            viewport: [inner, height],
            offset: [0., offset],
            content_width: Some(inner),
            content: Box::new(body),
        }),
    );
    let chip = |key: &str, label: &str| {
        named(
            key,
            frame(
                Frame {
                    fill: alpha(ink.fg, 0.07),
                    border: Some(alpha(ink.fg, 0.10)),
                    radius: [999.; 4],
                    padding: [0., 6., 0., 6.],
                    row: true,
                    ..Default::default()
                },
                vec![fixed(line(label, 10.5 * zoom, 400, ink.dim, false))],
            ),
        )
    };
    let mut tools = vec![
        fixed(chip("composer/attach", "@ context")),
        fixed(chip("composer/priority", "queue")),
        spacer(),
    ];
    let hint = match state {
        S::Run if has_text => "⏎ queue · esc interrupt",
        S::Run => "esc interrupt",
        S::Typing => "⏎ send · ⇧⏎ newline",
        S::Ask => "1 allow · 2 always · 3 deny",
        S::Idle | S::Focus => "",
    };
    if !hint.is_empty() {
        let color = if state == S::Ask {
            ink.ask
        } else {
            alpha(ink.dim, 0.7)
        };
        tools.push(fixed(named(
            "composer/hint",
            line(hint, 10.5 * zoom, 400, color, false),
        )));
    }
    // .hx-send: a 26 px circle. Empty: card-hi with a dim arrow; with text: seat with an
    // on-accent arrow and a glow; running and empty: a stop square on a hairline inset.
    // Fill and icon colour tween across idle → ready → stop; the arrow and the square
    // crossfade while `stop_t` is in flight.
    let stop = stop_now;
    let on_accent = hex("#FFFFFF");
    let arrow_ink = mix(on_accent, ink.dim, ready_t);
    let arrow = icon("send", arrow_ink, 14., 2.2);
    let glyph = if stop_t <= 0. {
        arrow
    } else if stop_t >= 1. {
        square(12., 6., 1.1, ink.fg)
    } else {
        c::Element::Layers {
            base: Box::new(c::Element::Filter {
                content: Box::new(arrow),
                saturation: 1.,
                opacity: 1. - stop_t,
            }),
            overlays: vec![c::Placed {
                content: c::Element::Filter {
                    content: Box::new(square(12., 6., 1.1, ink.fg)),
                    saturation: 1.,
                    opacity: stop_t,
                },
                left: 1.,
                bottom: 1.,
            }],
        }
    };
    let mut send = frame(
        Frame {
            fill: mix(ink.seat, alpha(ink.fg, 0.07), ready_t),
            border: (stop_t > 0.).then_some(alpha(ink.fg, 0.10 * stop_t)),
            radius: [13.; 4],
            padding: [if stop { 7. } else { 6. }; 4],
            ..Default::default()
        },
        vec![fixed(glyph)],
    );
    if let c::Element::Box(b) = &mut send {
        // Fixed 26 px whatever the inset border does to the padding box.
        b.style.overrides.width = Some(ui::Length::Px(26.));
        b.style.overrides.height = Some(ui::Length::Px(26.));
        if stop {
            b.style.overrides.padding = Some([ui::Length::Px(6.); 4]);
        }
        if ready_t > 0. {
            b.style.overrides.shadow = Some(ui::Shadow::Custom {
                blur: 11.,
                color: alpha(ink.seat, 0.55 * ready_t),
            });
        }
    }
    tools.push(fixed(named("composer/send", send)));
    let mut kids = vec![];
    if let Some(n) = view.notice.as_ref().filter(|n| n.kind != "sent") {
        kids.push(fixed(named(
            "composer/notice",
            line(n.text.clone(), 11.5 * zoom, 400, ink.ask, false),
        )));
    }
    // Zero-height address slot: routing stays in the draft model, the row is not shown.
    kids.push(c::Item {
        element: named("composer/address", c::Element::Spacer),
        extent: c::Extent::Fixed(0.),
    });
    kids.push(fixed(viewport));
    // The tools row sits 6 px under the draft (.hx-tools margin-top).
    kids.push(fixed(frame(
        Frame {
            padding: [6., 0., 0., 0.],
            gap: 6.,
            row: true,
            ..Default::default()
        },
        tools,
    )));
    // The alive rim (harness-spec.md Composer: "replaces the top-edge sweep and the focus
    // ring"): the term's pane rim on the composer box. The border is a constant seat-35 and
    // the fill is `card` → `focus-fill` with focus; everything else that moves lives in the
    // rim shader. Copied from bbt_ui::modules::pane (bboxy450-term crates/bbt-ui/src/modules/pane.rs,
    // the PaneGlow style and its Rim overrides; https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md).
    const PADDING: [f32; 4] = [8., 10., 6., 10.];
    let mut style = ui::Style::new(ui::Preset::PaneGlow);
    style.overrides.fill = Some(paint(alpha(ink.fg, 0.04 + 0.01 * focus_t)));
    style.overrides.border = Some(ui::Border {
        width: ui::Length::Px(1.),
        paint: paint(alpha(ink.seat, 0.35)),
        outside: false,
    });
    style.overrides.radii = Some([ui::Length::Px(13.); 4]);
    style.overrides.padding = Some(PADDING.map(ui::Length::Px));
    style.overrides.shadow = Some(ui::Shadow::None);
    let rim_size = bbt_ui_model::TOKENS.glow.rim as f32;
    style.overrides.rim = Some(ui::Rim {
        activity: rim_activity(live, ask, errored),
        swell: swell.clamp(0., 1.),
        focused,
        seed: RIM_SEED,
        inferred_multiplier: 1.,
        tint: Some(ink.seat),
        // The cell edge rule (rim-glow §3): the bloom stops at the box's smallest padding,
        // so no glyph of the draft is ever lit.
        depth: Some(
            (bbt_ui_model::TOKENS.glow.depth as f32)
                .min(PADDING.iter().copied().fold(f32::MAX, f32::min)),
        ),
        // A block draws its outer half only into the spacing it has (rim-glow §8): the
        // composer sits 8 px under the feed.
        outer: Some((rim_size / 2.).min(8.)),
        ..ui::Rim::default()
    });
    let mut b = c::Box::new(style, kids);
    b.direction = Axis::Column;
    b.gap = ui::Length::Px(0.);
    b.align = c::Alignment::Stretch;
    let panel = c::Element::Box(b);
    Ok(frame(
        Frame {
            padding: [8., side, 12., side],
            ..Default::default()
        },
        vec![fixed(named("composer", panel))],
    ))
}
/// Global text zoom (harness-spec.md "Standalone window"): every text size and line height
/// in the face scales by `scale`; paddings, radii, icons and tiles do not, so the layout
/// reflows around larger type. A run with no explicit size takes its type step's pixels
/// first. Applied to an element before it is laid out.
pub fn zoom_text(e: &mut c::Element, scale: f32) {
    if !(scale.is_finite() && scale > 0.) || (scale - 1.).abs() < 1e-6 {
        return;
    }
    fn style(s: &mut ui::Text, scale: f32) {
        s.size = Some(s.size.unwrap_or_else(|| s.step.pixels()) * scale);
    }
    match e {
        c::Element::Text(t) => {
            for run in &mut t.runs {
                style(&mut run.style, scale);
            }
        }
        c::Element::EditingText(t) => style(&mut t.style, scale),
        c::Element::InlineMiddle { content, parent } => {
            style(parent, scale);
            zoom_text(content, scale);
        }
        c::Element::Named { name, content } => {
            // Literal panes were measured before text zoom. Their horizontal extent
            // must grow with the glyphs, while the viewport stays inside the card.
            if (name.starts_with("codeout/") || name.starts_with("mdout/"))
                && let c::Element::ScrollView(scroll) = content.as_mut()
                && let Some(width) = &mut scroll.content_width
            {
                *width = (*width * scale).max(scroll.viewport[0]);
            }
            zoom_text(content, scale);
        }
        c::Element::FixedHeight { content, .. }
        | c::Element::Filter { content, .. }
        | c::Element::Baseline { content, .. } => zoom_text(content, scale),
        c::Element::Layers { base, overlays } => {
            // Overlays laid on text (an intraline diff mark) sit at offsets measured
            // at 100%; they scale with the glyphs or drift right along the line.
            let on_text = matches!(base.as_ref(), c::Element::Text(_));
            zoom_text(base, scale);
            for p in overlays {
                if on_text {
                    p.left *= scale;
                    p.bottom *= scale;
                    if let c::Element::Rects(r) = &mut p.content {
                        r.size = r.size.map(|v| v * scale);
                        r.rects = r
                            .rects
                            .iter()
                            .map(|(rect, paint)| {
                                let mut rect = *rect;
                                rect.x *= scale;
                                rect.y *= scale;
                                rect.w *= scale;
                                rect.h *= scale;
                                (rect, *paint)
                            })
                            .collect();
                    }
                }
                zoom_text(&mut p.content, scale);
            }
        }
        c::Element::Box(b) => {
            if let Some(t) = &mut b.style.overrides.text {
                style(t, scale);
            }
            for item in &mut b.children {
                zoom_text(&mut item.element, scale);
            }
        }
        c::Element::Stack(s) => {
            for item in &mut s.children {
                zoom_text(&mut item.element, scale);
            }
        }
        c::Element::Flow(f) => {
            for item in &mut f.children {
                zoom_text(&mut item.element, scale);
            }
        }
        c::Element::ScrollView(s) => zoom_text(&mut s.content, scale),
        c::Element::Overlay(o) => zoom_text(&mut o.content, scale),
        c::Element::Glyph { .. }
        | c::Element::Spacer
        | c::Element::Icon { .. }
        | c::Element::Image { .. }
        | c::Element::ImageRect { .. }
        | c::Element::Path(_)
        | c::Element::Rects(_) => {}
    }
}
/// The zoom pill (harness-spec.md Standalone window; lab.css .hx-zoom): `110%` in mono
/// 600 12/1.4, padding 4 12, pill radius, `card-hi` fill, `hair` border, `fg` text. The
/// view centres it 12 px under the feed's top and runs its drop and fade.
pub fn zoom_pill(label: &str, ink: &Ink) -> c::Element {
    let mut t = text(label, 12., 600, ink.fg, true);
    t.wrap = false;
    if let Some(run) = t.runs.first_mut() {
        run.style.line_height = 1.4;
    }
    frame(
        Frame {
            fill: alpha(ink.fg, 0.07),
            border: Some(alpha(ink.fg, 0.10)),
            radius: [999.; 4],
            padding: [4., 12., 4., 12.],
            row: true,
            ..Default::default()
        },
        vec![fixed(c::Element::Text(t))],
    )
}
pub(crate) fn identity_mark(ink: &Ink, halo_a: f32, halo_s: f32, key: &str) -> c::Element {
    // the mark: 52 round; the halo ring 6 px outside it, so a 64 box at rest
    const SIDE: f32 = 64.;
    let r = SIDE / 2.;
    let mut ring = circle(r, r, (r - 0.5) * halo_s);
    ring.shrink_to_fit();
    let halo = c::Element::Path(c::Path {
        size: [SIDE, SIDE],
        points: Arc::from(ring),
        stroke: PathStroke {
            width: 1.,
            ..Default::default()
        },
        paint: paint(alpha(ink.seat, 0.45)),
        fill: None,
        opacity: halo_a,
    });
    c::Element::Layers {
        base: Box::new(named(format!("{key}/halo"), halo)),
        overlays: vec![c::Placed {
            content: named(
                format!("{key}/mark"),
                c::Element::Image {
                    asset: super::assets::MARK.into(),
                    size: ui::Length::Px(52.),
                    radius: ui::Length::Px(26.),
                },
            ),
            left: 6.,
            bottom: 6.,
        }],
    }
}
fn breathing_halo(anim: &mut Clock) -> (f32, f32) {
    if anim.animates() {
        let v = 0.5 - 0.5 * (anim.cycle(3200.) * std::f32::consts::TAU).cos();
        (0.25 + 0.55 * v, 0.96 + 0.08 * v)
    } else {
        (0.5, 1.)
    }
}
/// The welcome portrait and one status label, with no welcome actions or copy.
pub fn loading_session(_preparing: bool, ink: &Ink, anim: &mut Clock) -> c::Element {
    let (halo_a, halo_s) = breathing_halo(anim);
    let mut group = vstack(
        vec![
            fixed(identity_mark(ink, halo_a, halo_s, "session-loader")),
            fixed(named(
                "session-loader/label",
                line("Loading…", 13.5, 400, ink.dim, false),
            )),
        ],
        14.,
    );
    if let c::Element::Box(b) = &mut group {
        b.align = c::Alignment::Center;
    }
    named("session-loader", group)
}

/// The empty session (harness-spec.md "Empty session"): the 52 px mark inside a breathing
/// seat halo, "What are we building?", `profile · model in ~/<cwd leaf>`, and three chips
/// named `empty/chip/0..2` (`/ commands`, `@ a file`, `Describe a change`). Translated
/// from harness.js empty() and lab.css .hx-empty (bboxy450-term lab/web;
/// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md).
///
/// Motion: the column rises 6 px and fades in over 420 ms `out`, the chips follow 60 ms
/// apart, and the halo breathes over 3.2 s (opacity .25–.8, scale .96–1.04). Flat: the
/// end state with a static ring at .5. `top` is the scroll height's 12 %.
pub fn empty_session(
    who: Option<String>,
    leaf: Option<String>,
    top: f32,
    ink: &Ink,
    anim: &mut Clock,
) -> c::Element {
    let rise = anim.enter("empty/in", 0., 1., 420., Ease::Out);
    let (halo_a, halo_s) = breathing_halo(anim);
    let mark = identity_mark(ink, halo_a, halo_s, "empty");
    let mut title = text("What are we building?", 17., 600, ink.fg, false);
    title.wrap = false;
    for run in &mut title.runs {
        run.style.tracking_em = -0.01;
    }
    let mut kids = vec![
        c::Item {
            // .hx-emark margin-bottom 6 on top of the column's 8 gap
            element: frame(
                Frame {
                    padding: [0., 0., 6., 0.],
                    ..Default::default()
                },
                vec![fixed(mark)],
            ),
            extent: c::Extent::Auto,
        },
        fixed(named("empty/title", c::Element::Text(title))),
    ];
    let sub = match (who, leaf) {
        (Some(w), Some(l)) => Some(format!("{w} in ~/{l}")),
        (Some(w), None) => Some(w),
        (None, Some(l)) => Some(format!("~/{l}")),
        (None, None) => None,
    };
    if let Some(sub) = sub {
        kids.push(fixed(named(
            "empty/sub",
            line(sub, 12., 400, ink.dim, false),
        )));
    }
    let chips = ["/ commands", "@ a file", "Describe a change"]
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let t = anim.enter_after(
                &format!("empty/chip/{i}"),
                0.,
                1.,
                420.,
                60. * (i + 1) as f32,
                Ease::Out,
            );
            let chip = named(
                format!("empty/chip/{i}"),
                frame(
                    Frame {
                        fill: alpha(ink.fg, 0.04),
                        border: Some(alpha(ink.fg, 0.10)),
                        radius: [999.; 4],
                        padding: [5., 11., 5., 11.],
                        row: true,
                        ..Default::default()
                    },
                    vec![fixed(line(*label, 12., 400, ink.dim, false))],
                ),
            );
            fixed(lift(chip, t))
        })
        .collect();
    kids.push(fixed(frame(
        Frame {
            // .hx-echips margin-top 10 on top of the column's 8 gap
            padding: [10., 0., 0., 0.],
            gap: 6.,
            row: true,
            ..Default::default()
        },
        chips,
    )));
    let mut column = frame(
        Frame {
            padding: [top + 14., 16., 0., 16.],
            gap: 8.,
            ..Default::default()
        },
        kids,
    );
    if let c::Element::Box(b) = &mut column {
        b.align = c::Alignment::Center;
    }
    named("empty", lift(column, rise))
}
/// An entrance at progress `t`: transparent at 0, opaque at 1. The 6 px rise is applied to
/// the solved layout by the view (`view::rise`), as a CSS transform leaves flow alone. At 1
/// the element is returned untouched, so settled layouts carry no extra nodes.
pub fn lift(element: c::Element, t: f32) -> c::Element {
    if t >= 1. {
        return element;
    }
    c::Element::Filter {
        content: Box::new(element),
        saturation: 1.,
        opacity: t.clamp(0., 1.),
    }
}
/// A filled rounded square centred in a `box_side` path (the web `stop` glyph).
fn square(box_side: f32, side: f32, r: f32, color: Rgba) -> c::Element {
    let (a, b) = ((box_side - side) / 2., (box_side + side) / 2.);
    let k = 0.552_284_8 * r;
    let mut path = vec![
        P::Move([a + r, a]),
        P::Line([b - r, a]),
        P::Cubic([[b - r + k, a], [b, a + r - k], [b, a + r]]),
        P::Line([b, b - r]),
        P::Cubic([[b, b - r + k], [b - r + k, b], [b - r, b]]),
        P::Line([a + r, b]),
        P::Cubic([[a + r - k, b], [a, b - r + k], [a, b - r]]),
        P::Line([a, a + r]),
        P::Cubic([[a, a + r - k], [a + r - k, a], [a + r, a]]),
    ];
    path.shrink_to_fit();
    c::Element::Path(c::Path {
        size: [box_side, box_side],
        points: Arc::from(path),
        stroke: PathStroke {
            width: 0.,
            ..Default::default()
        },
        paint: paint(color),
        fill: Some(paint(color)),
        opacity: 1.,
    })
}
/// TextArea paints its placeholder in the theme's muted ink; the web dims it further.
fn tint_placeholder(e: &mut c::Element, p: ui::Paint) {
    match e {
        c::Element::Text(t) => t.runs.iter_mut().for_each(|r| r.style.paint = p),
        c::Element::Named { content, .. } => tint_placeholder(content, p),
        c::Element::Layers { base, overlays } => {
            tint_placeholder(base, p);
            overlays
                .iter_mut()
                .for_each(|o| tint_placeholder(&mut o.content, p));
        }
        c::Element::Box(b) => b
            .children
            .iter_mut()
            .for_each(|i| tint_placeholder(&mut i.element, p)),
        _ => {}
    }
}

/// The web's rotated `chev` icon: a small down chevron.
fn down_chevron(color: Rgba) -> c::Element {
    c::Element::Path(c::Path {
        size: [10., 10.],
        points: Arc::from([
            P::Move([2.5, 3.75]),
            P::Line([5., 6.25]),
            P::Line([7.5, 3.75]),
        ]),
        stroke: PathStroke {
            width: 1.5,
            cap: LineCap::Round,
            join: LineJoin::Round,
            ..Default::default()
        },
        paint: paint(color),
        fill: None,
        opacity: 1.,
    })
}
/// A filled dot: a shape, not a painted box (it has no content slot).
pub fn dot(side: f32, color: Rgba) -> c::Element {
    let mut path = circle(side / 2., side / 2., side / 2.);
    path.shrink_to_fit();
    c::Element::Path(c::Path {
        size: [side, side],
        points: Arc::from(path),
        stroke: PathStroke {
            width: 0.,
            ..Default::default()
        },
        paint: paint(color),
        fill: Some(paint(color)),
        opacity: 1.,
    })
}
/// A horizontal rule that stretches with its column (.hx-cmd's 1px bottom border).
pub fn hrule(h: f32, color: Rgba) -> c::Element {
    c::Element::Rects(c::Rects {
        size: [1., h],
        rects: Arc::from([(bbt_ui_model::layout::Rect::new(0., 0., 1., h), paint(color))]),
        repeat: Some((Axis::Row, 1.)),
    })
}
/// A vertical rule that stretches with its row.
pub fn bar(w: f32, color: Rgba) -> c::Element {
    c::Element::Rects(c::Rects {
        size: [w, 1.],
        rects: Arc::from([(bbt_ui_model::layout::Rect::new(0., 0., w, 1.), paint(color))]),
        repeat: Some((Axis::Column, 1.)),
    })
}

#[cfg(test)]
mod python_emit_tests {
    use super::*;
    use serde_json::json;
    fn ink() -> Ink {
        let c = [0.5, 0.5, 0.5, 1.];
        Ink {
            fg: [1.; 4],
            dim: c,
            surface: c,
            bg: [0., 0., 0., 1.],
            seat: c,
            ok: [0., 1., 0., 1.],
            bad: [1., 0., 0., 1.],
            ask: c,
        }
    }
    fn texts(e: &c::Element, out: &mut Vec<String>) {
        match e {
            c::Element::Text(t) => out.push(t.runs.iter().map(|r| r.value.as_str()).collect()),
            c::Element::Box(b) => b.children.iter().for_each(|i| texts(&i.element, out)),
            c::Element::Named { content, .. } => texts(content, out),
            _ => {}
        }
    }
    /// Rows wrapped in a tinted frame: the failure highlight.
    fn tinted(e: &c::Element) -> Vec<String> {
        let c::Element::Box(b) = e else { return vec![] };
        b.children
            .iter()
            .filter(|i| matches!(&i.element, c::Element::Box(r) if r.style.overrides.fill.as_ref().is_some_and(|f| *f != paint([0.; 4]))))
            .map(|i| {
                let mut t = vec![];
                texts(&i.element, &mut t);
                t.concat()
            })
            .collect()
    }
    #[test]
    fn python_emit_rows_align_highlight_failures_and_bound_depth_and_length() {
        let ink = ink();
        let (block, cut) = emit_block(&json!({"path":"a.md","err":null}), false, &ink);
        let mut rows = vec![];
        texts(&block, &mut rows);
        assert_eq!(rows, ["path  \"a.md\"", "err   null"]);
        assert!(
            !cut && tinted(&block).is_empty(),
            "a null err is not a failure"
        );
        let (block, _) = emit_block(&json!({"path":"a.md","error":"denied"}), false, &ink);
        assert_eq!(tinted(&block), ["error  \"denied\""]);
        assert!(
            failure_key("ERR", &json!({"code":1}))
                && !failure_key("err", &json!(false))
                && !failure_key("errand", &json!(1))
        );
        // Depth cap: the third level collapses unless the card is expanded.
        let deep = json!({"a":{"b":{"c":{"d":{"e":"x".repeat(80)}}}}});
        let (block, cut) = emit_block(&deep, false, &ink);
        let mut rows = vec![];
        texts(&block, &mut rows);
        assert!(
            cut && rows.last().unwrap().contains("{\u{2026}} 1 keys"),
            "{rows:?}"
        );
        let (block, cut) = emit_block(&deep, true, &ink);
        let mut rows = vec![];
        texts(&block, &mut rows);
        assert!(
            !cut && rows.concat().contains(&"x".repeat(80)),
            "expanded keeps every byte"
        );
        // Long strings and long blocks shorten; expanded shows them whole.
        let (_, cut) = emit_block(&json!("y".repeat(200)), false, &ink);
        assert!(cut);
        let many = json!((0..60).collect::<Vec<_>>());
        assert_eq!(emit_row_count(&many, &ink), EMIT_ROWS + 1);
        let (block, cut) = emit_block(&many, true, &ink);
        let mut rows = vec![];
        texts(&block, &mut rows);
        assert!(!cut && rows.len() == 60);
        // Literal colours: strings and numbers take the highlighter's tokens, null is dim.
        let mut parts = vec![];
        inline(
            &json!([1, "s", null, true]),
            &ink,
            96,
            &mut false,
            &mut parts,
        );
        let color = |text: &str| parts.iter().find(|p| p.0 == text).unwrap().1;
        assert_eq!(color("1"), tok_color(Tok::Num, &ink).0);
        assert_eq!(color("true"), tok_color(Tok::Num, &ink).0);
        assert_eq!(color("\"s\""), tok_color(Tok::Str, &ink).0);
        assert_eq!(color("null"), alpha(ink.dim, 0.75));
        // The closed preview is one bounded line.
        let preview: String = emit_preview(&json!({"k":"z".repeat(500)}), &ink)
            .into_iter()
            .map(|p| p.0)
            .collect();
        assert!(
            preview.chars().count() <= 64 && preview.starts_with("k "),
            "{preview}"
        );
    }
    #[test]
    fn python_status_chip_shows_exit_only_when_it_says_something() {
        let ink = ink();
        let label = |status: &str, exit| {
            let mut t = vec![];
            if let Some(chip) = python_status(status, exit, &ink) {
                texts(&chip, &mut t);
            }
            t.join(" ")
        };
        // The status disc already says completed, running and failed.
        assert_eq!(label("completed", Some(0)), "");
        assert_eq!(label("completed", Some(7)), "exit 7");
        assert_eq!(label("failed", Some(0)), "exit 0");
        assert_eq!(label("failed", Some(1)), "exit 1");
        assert_eq!(label("running", None), "");
        assert_eq!(label("timed_out", Some(1)), "timed_out exit 1");
        assert_eq!(label("cancelling", None), "cancelling");
        assert_eq!(label("cancelled", None), "cancelled");
    }
}
