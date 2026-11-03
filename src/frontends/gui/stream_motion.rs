// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Stream and card motion (harness-spec.md "Motion" table): each streamed chunk's glyphs
//! fade in; a fold tweens its height while the chevron turns; a running tool card's
//! border orbits and its dots bounce; the live "Thinking" label sheens; the turn receipt
//! rises in and its numbers count up.
//!
//! The window's animation clock (`anim.rs`) keeps time and eases values. This module
//! turns those values into changes on the visible rows: it never keeps time itself, and
//! its trackers only decide *what* is new (a chunk, a fold, an ended turn) so that text
//! already on screen never re-fades and offscreen rows take their end state.
//! Translated from bboxy450-term lab/web/lab.css (hx-sheen, hx-bounce, hx-orbit, hx-rise)
//! and harness-spec.md. https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use super::{
    anim::{Clock, Ease},
    bridge::FrameState,
    face::{Ink, icon_points},
    model::TurnState,
    rich::Faces,
};
use crate::protocol::{Item, ItemId, ItemKind, TurnId};
use bbt_ui::components::{Layout, LayoutKind, PositionedRun};
use bbt_ui_model::{
    draw::{Color, LinearAlphaMask, PathCommand as P, PathStroke, Prim, TextStyle, TypeScale},
    layout::{Rect, TextMeasure, TextStyle as MeasureStyle},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// Streaming text: each chunk's glyphs fade over this long (spec: 140ms `out`).
pub const FADE_MS: f32 = 140.;
/// Fold / unfold and block growth (spec: 200ms `out`).
pub const FOLD_MS: f32 = 200.;
/// The receipt rises in over 320ms; its numbers count up over 400ms.
pub const RISE_MS: f32 = 320.;
pub const COUNT_MS: f32 = 400.;
/// Loops: the tool border orbit, the dots' bounce, the thinking sheen, the think icon's spin.
pub const ORBIT_MS: f32 = 1600.;
pub const BOUNCE_MS: f32 = 1000.;
pub const SHEEN_MS: f32 = 1400.;
pub const SPIN_MS: f32 = 2400.;

// ---------- what is new: trackers ----------

/// A streaming row's glyphs as last seen, and the chunk starts still fading.
#[derive(Debug, Default, Clone)]
struct Fade {
    glyphs: String,
    /// (first glyph of the chunk, the frame time it arrived)
    chunks: Vec<(usize, f32)>,
    frame: u64,
}
/// Decides which glyphs are new. Settled text never re-fades: the glyph string of a row is
/// compared with the last frame's, and only what follows the common prefix is a chunk.
/// A row not seen in the previous frame (it was offscreen, or it is the first frame) is
/// taken as settled: offscreen blocks take their end state.
#[derive(Debug, Default)]
pub struct Fades {
    items: HashMap<ItemId, Fade>,
}
impl Fades {
    /// Observe `glyphs` of a visible row at `now` (ms) in frame `frame`. Returns the chunk
    /// starts still fading, ascending, with their arrival times.
    pub fn observe(&mut self, id: ItemId, glyphs: &str, now: f32, frame: u64) -> Vec<(usize, f32)> {
        let f = self.items.entry(id).or_insert_with(|| Fade {
            glyphs: glyphs.to_owned(),
            chunks: vec![],
            frame: frame.wrapping_sub(1),
        });
        if f.frame.wrapping_add(1) != frame {
            f.glyphs = glyphs.to_owned();
            f.chunks.clear();
        } else if glyphs != f.glyphs {
            let common = f
                .glyphs
                .chars()
                .zip(glyphs.chars())
                .take_while(|(a, b)| a == b)
                .count();
            // text that changed behind the prefix restarts from here; settled text before it stays
            f.chunks.retain(|(start, _)| *start < common);
            if glyphs.chars().count() > common {
                f.chunks.push((common, now));
            }
            f.glyphs = glyphs.to_owned();
        }
        f.frame = frame;
        f.chunks.retain(|(_, t0)| now - t0 < FADE_MS);
        f.chunks.clone()
    }
    /// Whether `id` still has a chunk in flight.
    pub fn fading(&self, id: &ItemId) -> bool {
        self.items.get(id).is_some_and(|f| !f.chunks.is_empty())
    }
    /// Drop rows that are complete and settled.
    pub fn forget(&mut self, keep: impl Fn(&ItemId) -> bool) {
        self.items.retain(|id, f| keep(id) || !f.chunks.is_empty());
    }
}

/// Turn receipts that ended while the view was watching rise in and count up; the ones
/// present when the view first saw the transcript are settled.
#[derive(Debug, Default)]
pub struct Receipts {
    seen: HashSet<TurnId>,
    fresh: HashMap<TurnId, f32>,
    primed: bool,
}
impl Receipts {
    /// `ended` lists every ended turn of the frame's transcript. Returns the turns whose
    /// receipt is still arriving, with their arrival time.
    pub fn observe(
        &mut self,
        ended: impl Iterator<Item = TurnId>,
        now: f32,
    ) -> &HashMap<TurnId, f32> {
        for turn in ended {
            if self.seen.insert(turn) && self.primed {
                self.fresh.insert(turn, now);
            }
        }
        self.primed = true;
        self.fresh.retain(|_, t0| now - *t0 < RISE_MS.max(COUNT_MS));
        &self.fresh
    }
}

// ---------- text: glyph order, run splitting, fades ----------

/// Every non-whitespace char of a subtree in paint order. Reflow moves line breaks and
/// spaces; glyphs keep their order, so a chunk boundary is a glyph index.
pub fn glyphs(node: &Layout, out: &mut String) {
    if let LayoutKind::Text { runs, .. } = &node.kind {
        for run in runs {
            out.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        }
    }
    for child in &node.children {
        glyphs(child, out);
    }
}
fn measure(style: &bbt_ui::style::ResolvedText) -> MeasureStyle {
    MeasureStyle {
        size: style.size,
        weight: style.weight,
        mono: style.mono,
        italic: style.italic,
        tracking_em: style.tracking_em,
    }
}
/// Split a run before its `at`-th char: the prefix keeps its place, the suffix starts where
/// the prefix's advance ends. (The renderer shapes each run alone; kerning across the cut is
/// lost for the fade's 140ms, which no eye sees.)
pub fn split_run(
    run: &PositionedRun,
    at: usize,
    metrics: &impl TextMeasure,
) -> (PositionedRun, PositionedRun) {
    let k = run
        .text
        .char_indices()
        .nth(at)
        .map_or(run.text.len(), |(i, _)| i);
    let w = metrics.width(&run.text[..k], &measure(&run.style));
    let mut a = run.clone();
    a.text = run.text[..k].to_owned();
    a.rect.w = w;
    let mut b = run.clone();
    b.text = run.text[k..].to_owned();
    b.rect.x += w;
    b.rect.w = (run.rect.w - w).max(0.);
    (a, b)
}
/// Applies chunk alphas to a row's runs as `view::visible_layout` visits them. `chunks` are
/// (first glyph, alpha) ascending; a glyph takes the alpha of the newest chunk at or before
/// it, and glyphs before every chunk are settled. The counter runs over every run of the
/// row, visible or not, so a boundary is the same glyph whatever the viewport shows.
pub struct Fader<'a, M: TextMeasure> {
    chunks: Vec<(usize, f32)>,
    metrics: &'a M,
    at: usize,
    /// Named nodes deep inside the `stream/` subtree; 0 outside it.
    depth: usize,
}
impl<'a, M: TextMeasure> Fader<'a, M> {
    pub fn new(chunks: Vec<(usize, f32)>, metrics: &'a M) -> Self {
        Self {
            chunks,
            metrics,
            at: 0,
            depth: 0,
        }
    }
    fn alpha_at(&self, glyph: usize) -> f32 {
        self.chunks
            .iter()
            .rev()
            .find(|(start, _)| *start <= glyph)
            .map_or(1., |(_, a)| *a)
    }
}
/// Visits a row's text runs during viewport restoration (every run, visible or not).
pub trait Runs {
    /// The runs of one Text node, in order; returns the runs to keep laying out.
    fn visit(&mut self, runs: &[PositionedRun]) -> Vec<PositionedRun>;
    /// A subtree the viewport elides; its runs still count.
    fn skip(&mut self, node: &Layout);
    /// Descending into / leaving a named node (balanced).
    fn enter(&mut self, _name: &str) {}
    fn leave(&mut self, _name: &str) {}
}
/// The plain pass: runs unchanged.
pub struct Plain;
impl Runs for Plain {
    fn visit(&mut self, runs: &[PositionedRun]) -> Vec<PositionedRun> {
        runs.to_vec()
    }
    fn skip(&mut self, _: &Layout) {}
}
/// The streamed text of a row: the subtree named `stream/{id}` (the reply's document or
/// the thought's body). Labels, controls and the receipt around it are not a stream.
pub const STREAM: &str = "stream/";
/// The glyphs of a row's streamed text, if it has any.
pub fn stream_glyphs(row: &Layout) -> Option<String> {
    fn stream(node: &Layout) -> Option<&Layout> {
        if matches!(&node.kind, LayoutKind::Named(n) if n.starts_with(STREAM)) {
            return Some(node);
        }
        node.children.iter().find_map(stream)
    }
    let mut out = String::new();
    glyphs(stream(row)?, &mut out);
    Some(out)
}
impl<M: TextMeasure> Runs for Fader<'_, M> {
    fn enter(&mut self, name: &str) {
        if self.depth > 0 || name.starts_with(STREAM) {
            self.depth += 1;
        }
    }
    fn leave(&mut self, _: &str) {
        self.depth = self.depth.saturating_sub(1);
    }
    fn visit(&mut self, runs: &[PositionedRun]) -> Vec<PositionedRun> {
        if self.depth == 0 {
            return runs.to_vec();
        }
        let mut out = Vec::with_capacity(runs.len());
        for run in runs {
            let n = run.text.chars().filter(|c| !c.is_whitespace()).count();
            let (start, end) = (self.at, self.at + n);
            self.at = end;
            let mut piece = run.clone();
            let mut piece_start = start;
            for &(boundary, _) in &self.chunks {
                if boundary <= piece_start {
                    continue;
                }
                if boundary >= end {
                    break;
                }
                // the char index in `piece` of its (boundary - piece_start)-th glyph
                let mut seen = 0;
                let cut = piece
                    .text
                    .chars()
                    .position(|c| {
                        if c.is_whitespace() {
                            return false;
                        }
                        seen += 1;
                        seen > boundary - piece_start
                    })
                    .unwrap_or(piece.text.chars().count());
                let (mut a, b) = split_run(&piece, cut, self.metrics);
                a.style.color[3] *= self.alpha_at(piece_start);
                out.push(a);
                piece = b;
                piece_start = boundary;
            }
            piece.style.color[3] *= self.alpha_at(piece_start);
            out.push(piece);
        }
        out
    }
    fn skip(&mut self, node: &Layout) {
        let counted = if self.depth > 0 {
            let mut s = String::new();
            glyphs(node, &mut s);
            Some(s)
        } else {
            stream_glyphs(node)
        };
        self.at += counted.map_or(0, |s| s.chars().count());
    }
}

// ---------- layout edits on the visible clone ----------

/// Wraps `node` in an opacity filter (the toolkit paints it as one group). Returns whether
/// it did: at full opacity the node stays as it is.
pub fn fade_node(node: &mut Layout, opacity: f32) -> bool {
    if opacity >= 1. {
        return false;
    }
    let inner = std::mem::replace(
        node,
        Layout {
            rect: Rect::new(0., 0., 0., 0.),
            baseline: 0.,
            kind: LayoutKind::Spacer,
            children: vec![],
        },
    );
    *node = Layout {
        rect: inner.rect,
        baseline: inner.baseline,
        kind: LayoutKind::Filter {
            saturation: 1.,
            opacity: opacity.max(0.),
        },
        children: vec![inner],
    };
    true
}
/// Moves `node` and everything in it by `dy` (the receipt's 6px rise).
pub fn lift(node: &mut Layout, dy: f32) {
    super::view::translate(node, 0., dy);
}
/// Turns a path layout to `deg` about the centre of its slot: the chevron (0 → 90°) and
/// the spinning think icon. `base` are the unturned commands in slot units.
pub fn turn_path(node: &mut Layout, base: &[P], deg: f32) {
    fn path(node: &mut Layout) -> Option<&mut Layout> {
        if matches!(node.kind, LayoutKind::Path { .. }) {
            return Some(node);
        }
        node.children.iter_mut().find_map(path)
    }
    let Some(node) = path(node) else {
        return;
    };
    if let LayoutKind::Path { points, scale, .. } = &mut node.kind {
        let s = if *scale > 0. { *scale } else { 1. };
        let c = [node.rect.w / 2. / s, node.rect.h / 2. / s];
        *points = Arc::from(turned(base, deg, c));
    }
}
/// `points` rotated clockwise by `deg` about `c`.
pub fn turned(points: &[P], deg: f32, c: [f32; 2]) -> Vec<P> {
    let (sin, cos) = deg.to_radians().sin_cos();
    let r = |p: [f32; 2]| {
        let (x, y) = (p[0] - c[0], p[1] - c[1]);
        [c[0] + x * cos - y * sin, c[1] + x * sin + y * cos]
    };
    points
        .iter()
        .map(|cmd| match cmd {
            P::Move(p) => P::Move(r(*p)),
            P::Line(p) => P::Line(r(*p)),
            P::Cubic(ps) => P::Cubic(ps.map(r)),
        })
        .collect()
}
/// hx-bounce at `phase` (0..1): (dy, alpha). 0%, 80%, 100%: rest at .5; 40%: up 3px at 1.
/// Each keyframe leg eases in-out.
pub fn bounce(phase: f32) -> (f32, f32) {
    let phase = phase.rem_euclid(1.);
    let e = if phase < 0.4 {
        Ease::InOut.at(phase / 0.4)
    } else if phase < 0.8 {
        1. - Ease::InOut.at((phase - 0.4) / 0.4)
    } else {
        0.
    };
    (-3. * e, 0.5 + 0.5 * e)
}
/// Lifts a dot and sets its alpha for the bounce.
pub fn bounce_dot(node: &mut Layout, (dy, alpha): (f32, f32)) {
    super::view::translate(node, 0., dy);
    if let LayoutKind::Path { color, fill, .. } = &mut node.kind {
        color[3] = alpha;
        if let Some(f) = fill {
            f[3] = alpha;
        }
    }
}
/// The three dots under a `dots` node, staggered by `lag` of the cycle each.
fn bounce_dots(node: &mut Layout, phase: f32, lag: f32) {
    fn paths<'a>(node: &'a mut Layout, out: &mut Vec<&'a mut Layout>) {
        if matches!(node.kind, LayoutKind::Path { .. }) {
            out.push(node);
            return;
        }
        for child in &mut node.children {
            paths(child, out);
        }
    }
    let mut dots = vec![];
    paths(node, &mut dots);
    for (k, dot) in dots.into_iter().enumerate() {
        bounce_dot(dot, bounce(phase - lag * k as f32));
    }
}
/// Scales the alpha of a box's border (the running card dims its ring under the orbit).
pub fn dim_border(node: &mut Layout, alpha: f32) {
    if let LayoutKind::Box { style, .. } = &mut node.kind
        && let Some(border) = &mut style.border
    {
        border.color[3] *= alpha;
    }
}
/// Recolours every run under `node`.
pub fn recolor(node: &mut Layout, color: [f32; 4]) {
    if let LayoutKind::Text { runs, .. } = &mut node.kind {
        for run in runs {
            run.style.color = color;
        }
    }
    for child in &mut node.children {
        recolor(child, color);
    }
}
/// Rewrites every run under `node` through `f` (the receipt's count-up).
pub fn rewrite(node: &mut Layout, f: &impl Fn(&str) -> String) {
    if let LayoutKind::Text { runs, .. } = &mut node.kind {
        for run in runs {
            run.text = f(&run.text);
        }
    }
    for child in &mut node.children {
        rewrite(child, f);
    }
}
/// Every number in `text` scaled by `p`, keeping its decimals: "1.2k in · 300 out" at .5
/// reads "0.6k in · 150 out". The words and units stay.
pub fn count_up(text: &str, p: f32) -> String {
    let p = p.clamp(0., 1.) as f64;
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || (chars[i] == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
            {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            let decimals = token.find('.').map_or(0, |d| token.len() - d - 1);
            let value: f64 = token.parse().unwrap_or(0.);
            let scaled = value * p;
            if decimals == 0 {
                out.push_str(&format!("{}", scaled.round() as u64));
            } else {
                out.push_str(&format!("{scaled:.decimals$}"));
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

// ---------- paint: the orbit and the sheen ----------

/// A point on the 1px border line of a rounded rect at perimeter fraction `s` (0 at the
/// top centre, clockwise). The line sits half a pixel inside the box, as an inset border.
pub fn perimeter_point(rect: Rect, radius: f32, s: f32) -> [f32; 2] {
    let (x0, y0) = (rect.x + 0.5, rect.y + 0.5);
    let (w, h) = ((rect.w - 1.).max(0.), (rect.h - 1.).max(0.));
    let r = (radius - 0.5).clamp(0., w.min(h) / 2.);
    let arc = std::f32::consts::FRAC_PI_2 * r;
    let (ew, eh) = (w - 2. * r, h - 2. * r);
    let total = 2. * (ew + eh) + 4. * arc;
    if total <= 0. {
        return [x0, y0];
    }
    // the path starts after the top-left corner; the top centre is half the top edge in
    let mut d = (s.rem_euclid(1.) * total + ew / 2.).rem_euclid(total);
    let a = arc.max(1e-6);
    // top edge, then clockwise: a corner arc after every edge
    for (i, len) in [ew, arc, eh, arc, ew, arc, eh, arc].into_iter().enumerate() {
        if d <= len || i == 7 {
            let t = d;
            return match i {
                0 => [x0 + r + t, y0],
                1 => corner(x0 + w - r, y0 + r, r, -90., t / a),
                2 => [x0 + w, y0 + r + t],
                3 => corner(x0 + w - r, y0 + h - r, r, 0., t / a),
                4 => [x0 + w - r - t, y0 + h],
                5 => corner(x0 + r, y0 + h - r, r, 90., t / a),
                6 => [x0, y0 + h - r - t],
                _ => corner(x0 + r, y0 + r, r, 180., t / a),
            };
        }
        d -= len;
    }
    [x0 + r, y0]
}
fn corner(cx: f32, cy: f32, r: f32, from_deg: f32, t: f32) -> [f32; 2] {
    let a = (from_deg + 90. * t.clamp(0., 1.)).to_radians();
    [cx + r * a.cos(), cy + r * a.sin()]
}
/// The conic orbit of a running tool card (lab.css .hx-tool.run): transparent to 55%,
/// `seat` at 75%, seat-30%-on-`fg` at 82%, transparent at 92%, turned by `phase` (0..1,
/// one lap per 1.6s). Drawn as short strokes along the border line, each with its own
/// colour and alpha, so the highlight has a head and a tail.
pub fn orbit(rect: Rect, radius: f32, phase: f32, seat: [f32; 4], fg: [f32; 4]) -> Vec<Prim> {
    const SEGMENTS: usize = 24;
    let bright = [0, 1, 2].map(|i| seat[i] * 0.3 + fg[i] * 0.7);
    let shade = |f: f32| -> [f32; 4] {
        if f < 0.75 {
            [seat[0], seat[1], seat[2], ((f - 0.55) / 0.2).clamp(0., 1.)]
        } else if f < 0.82 {
            let t = (f - 0.75) / 0.07;
            let [r, g, b] = [0, 1, 2].map(|i| seat[i] * (1. - t) + bright[i] * t);
            [r, g, b, 1.]
        } else {
            [
                bright[0],
                bright[1],
                bright[2],
                (1. - (f - 0.82) / 0.1).clamp(0., 1.),
            ]
        }
    };
    let (w, h) = ((rect.w - 1.).max(0.), (rect.h - 1.).max(0.));
    let r = (radius - 0.5).clamp(0., w.min(h) / 2.);
    let total = 2. * (w + h - 4. * r) + 2. * std::f32::consts::PI * r;
    let mut out = Vec::with_capacity(SEGMENTS);
    for k in 0..SEGMENTS {
        let f0 = 0.55 + 0.37 * k as f32 / SEGMENTS as f32;
        let f1 = 0.55 + 0.37 * (k + 1) as f32 / SEGMENTS as f32;
        let color = shade((f0 + f1) / 2.);
        if color[3] <= 0.002 {
            continue;
        }
        // sample every ~1.5px so the corners stay round
        let steps = (((f1 - f0) * total) / 1.5).ceil().max(1.) as usize;
        let mut points = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let f = f0 + (f1 - f0) * i as f32 / steps as f32;
            let p = perimeter_point(rect, radius, f + phase);
            points.push(if i == 0 { P::Move(p) } else { P::Line(p) });
        }
        out.push(Prim::Path {
            origin: [0., 0.],
            scale: 1.,
            points: Arc::from(points),
            stroke: PathStroke {
                width: 1.,
                cap: bbt_ui_model::draw::LineCap::Butt,
                ..Default::default()
            },
            color: Color(color),
            fill: None,
        });
    }
    out
}
/// The toolkit's text primitive for a run, in `color`.
pub fn text_prim(run: &PositionedRun, color: [f32; 4]) -> Prim {
    Prim::Text {
        origin: [
            run.rect.x,
            run.rect.y + (run.line_height - run.style.size * 1.4) * 0.5,
        ],
        baseline: Some(run.baseline),
        text: run.text.clone().into(),
        style: TextStyle {
            scale: TypeScale::Custom(run.style.size),
            weight: run.style.weight,
            mono: run.style.mono,
            italic: run.style.italic,
            tracking_em: run.style.tracking_em,
            uppercase: false,
        },
        max_width: None,
        color: Color(color),
    }
}
/// `prims` through a tent of alpha: 0 at `x0`, 1 from `x1` to `x2`, 0 at `x3`.
fn tent(prims: Vec<Prim>, y: f32, x0: f32, x1: f32, x2: f32, x3: f32) -> Prim {
    let rising = Prim::Group {
        primitives: prims,
        opacity: 1.,
        saturation: 1.,
        mask: Some(LinearAlphaMask {
            start: [x0, y],
            end: [x1.max(x0 + 0.01), y],
            alpha0: 0.,
            alpha1: 1.,
        }),
    };
    Prim::Group {
        primitives: vec![rising],
        opacity: 1.,
        saturation: 1.,
        mask: Some(LinearAlphaMask {
            start: [x2, y],
            end: [x3.max(x2 + 0.01), y],
            alpha0: 1.,
            alpha1: 0.,
        }),
    }
}
/// The sheen across a live "Thinking" label (lab.css .hx-shimmer): over a dim base, `fg`
/// glyphs under a wide tent and `seat` glyphs under a narrow one cross the label from left
/// to right once per cycle. `runs` are the label's runs; `phase` is 0..1.
pub fn sheen(runs: &[PositionedRun], phase: f32, fg: [f32; 4], seat: [f32; 4]) -> Vec<Prim> {
    let Some(first) = runs.first() else {
        return vec![];
    };
    let x = runs.iter().map(|r| r.rect.x).fold(f32::INFINITY, f32::min);
    let right = runs
        .iter()
        .map(|r| r.rect.right())
        .fold(f32::NEG_INFINITY, f32::max);
    let w = (right - x).max(1.);
    let y = first.baseline;
    // background-position 100% → -150% on a 250%-wide gradient: the highlight's centre
    // travels from a quarter width before the label to well past it
    let c = x - 0.25 * w + 3.75 * w * phase.rem_euclid(1.);
    let text = |color: [f32; 4]| runs.iter().map(|r| text_prim(r, color)).collect::<Vec<_>>();
    vec![
        tent(
            text(fg),
            y,
            c - 1.25 * w,
            c - 0.25 * w,
            c + 0.25 * w,
            c + 1.25 * w,
        ),
        tent(
            text(seat),
            y,
            c - 0.3 * w,
            c - 0.05 * w,
            c + 0.05 * w,
            c + 0.3 * w,
        ),
    ]
}

// ---------- the frame pass ----------

/// Per-view motion state for the stream and the cards. `pass` runs once per built frame
/// over the restored (visible) rows, after `view::restore_rows` applied the chunk fades.
#[derive(Default)]
pub struct Motion {
    pub fades: Fades,
    receipts: Receipts,
    /// The faces of the last frame: a changed face is a fold that opened or closed.
    faces: Option<Arc<Faces>>,
    /// Folds in flight: item -> the frame time it toggled.
    folds: HashMap<ItemId, f32>,
    /// Built frames, for the fade tracker's "seen last frame" rule.
    pub frame: u64,
}
impl Motion {
    /// Note a built frame and what changed in it before the rows are restored.
    pub fn begin_frame(&mut self, state: &FrameState, now: f32) {
        self.frame = self.frame.wrapping_add(1);
        let model = &state.transcript;
        self.receipts.observe(
            model
                .turns
                .iter()
                .filter_map(|(t, s)| matches!(s, TurnState::Ended { .. }).then_some(*t)),
            now,
        );
        if let Some(old) = &self.faces
            && !Arc::ptr_eq(old, &state.faces)
        {
            for (id, face) in state.faces.iter() {
                if old.get(id) != Some(face) {
                    self.folds.insert(*id, now);
                }
            }
        }
        self.faces = Some(state.faces.clone());
        self.folds.retain(|_, t0| now - *t0 < FOLD_MS);
        self.fades
            .forget(|id| model.items.get(id).is_some_and(|r| r.completed.is_none()));
    }
    /// Whether `id` streams text that may still fade: an open reply or thought, or one whose
    /// last chunk is still arriving.
    pub fn streaming(&self, state: &FrameState, id: &ItemId) -> bool {
        self.fades.fading(id)
            || state.transcript.items.get(id).is_some_and(|r| {
                r.completed.is_none()
                    && matches!(r.kind, ItemKind::AgentMessage | ItemKind::Reasoning)
            })
    }
    /// Moves the visible rows for this frame and returns the overlay primitives (the orbit
    /// arcs and the sheen) to paint over them, clipped to the transcript viewport.
    pub fn pass(
        &mut self,
        anim: &mut Clock,
        layout: &mut Layout,
        state: &FrameState,
        ink: &Ink,
    ) -> Vec<Prim> {
        let mut out = vec![];
        if !anim.animates() {
            return out;
        }
        let model = &state.transcript;
        let asking: HashSet<ItemId> = model
            .items
            .values()
            .filter_map(|r| match &r.body {
                Item::Approval {
                    subject,
                    decision: None,
                    ..
                } if r.status.is_none() => Some(subject.tool_call),
                _ => None,
            })
            .collect();
        let live = |id: &ItemId| model.items.get(id).is_some_and(|r| r.status.is_none());
        // returns whether it wrapped the node (its original children are then one level down)
        let mut walk = |node: &mut Layout, anim: &mut Clock, out: &mut Vec<Prim>| -> bool {
            let LayoutKind::Named(name) = &node.kind else {
                return false;
            };
            let name = name.clone();
            let Some((head, rest)) = name.split_once('/') else {
                return false;
            };
            let id = |s: &str| s.split('/').next().and_then(|s| s.parse().ok()).map(ItemId);
            match head {
                // the running card: a dim ring under the orbit; its dots bounce
                "tool" if rest.ends_with("/status") => {
                    if let Some(id) = id(rest)
                        && live(&id)
                        && !asking.contains(&id)
                    {
                        bounce_dots(node, anim.cycle(BOUNCE_MS), 0.15);
                    }
                }
                "tool" if !rest.contains('/') => {
                    if let Some(id) = id(rest)
                        && live(&id)
                        && !asking.contains(&id)
                    {
                        if let Some(card) = node.children.first_mut() {
                            dim_border(card, 0.3);
                        }
                        out.extend(orbit(
                            node.rect,
                            11.,
                            anim.cycle(ORBIT_MS),
                            ink.seat,
                            ink.fg,
                        ));
                    }
                }
                // a live thought: the icon spins, the label sheens, the dots bounce
                "reasoning" if rest.ends_with("/toggle/icon") => {
                    if id(rest).is_some_and(|id| live(&id)) {
                        let base = icon_points("think", node.rect.w, 0.);
                        turn_path(node, &base, 360. * anim.cycle(SPIN_MS));
                    }
                }
                "reasoning" if rest.ends_with("/toggle/label") => {
                    if id(rest).is_some_and(|id| live(&id)) {
                        let phase = anim.cycle(SHEEN_MS);
                        recolor(node, ink.dim);
                        let mut runs = vec![];
                        label_runs(node, &mut runs);
                        out.extend(sheen(&runs, phase, ink.fg, ink.seat));
                    }
                }
                "reasoning" if rest.ends_with("/toggle/dots") => {
                    bounce_dots(node, anim.cycle(BOUNCE_MS), 0.16);
                }
                // folds: the chevron turns 0 → 90°, the body fades in
                "reasoning" | "toolhead" if rest.ends_with("/chev") => {
                    if let Some(id) = id(rest) {
                        let face = state.faces.get(&id).cloned().unwrap_or_default();
                        let open = if head == "reasoning" {
                            face.reasoning_open
                        } else {
                            face.output_closed
                        };
                        let deg = anim.tween(
                            &format!("chev/{id}"),
                            if open { 90. } else { 0. },
                            FOLD_MS,
                            Ease::Out,
                        );
                        let base = icon_points("chev", node.rect.w, 0.);
                        turn_path(node, &base, deg);
                    }
                }
                "toolbody" | "loopbody" => {
                    if let Some(id) = id(rest)
                        && self.folds.contains_key(&id)
                    {
                        let t = anim.enter(&format!("fold/{id}"), 0., 1., FOLD_MS, Ease::Out);
                        return fade_node(node, t);
                    }
                }
                "reasoning" if rest.ends_with("/body") => {
                    if let Some(id) = id(rest)
                        && self.folds.contains_key(&id)
                    {
                        let t = anim.enter(&format!("fold/{id}"), 0., 1., FOLD_MS, Ease::Out);
                        return fade_node(node, t);
                    }
                }
                // the receipt rises in; its numbers count up
                "turn" if rest.contains("/fact/") => {
                    if let Some(turn) = rest
                        .split('/')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .map(TurnId)
                        && self.receipts.fresh.contains_key(&turn)
                    {
                        let t =
                            anim.enter(&format!("count/{}", turn.0), 0., 1., COUNT_MS, Ease::Out);
                        rewrite(node, &|s| count_up(s, t));
                    }
                }
                "turn" if !rest.contains('/') => {
                    if let Some(turn) = rest.parse().ok().map(TurnId)
                        && self.receipts.fresh.contains_key(&turn)
                    {
                        let t = anim.enter(&format!("rise/{}", turn.0), 0., 1., RISE_MS, Ease::Out);
                        lift(node, 6. * (1. - t));
                        return fade_node(node, t);
                    }
                }
                _ => {}
            }
            false
        };
        fn visit(
            node: &mut Layout,
            walk: &mut impl FnMut(&mut Layout, &mut Clock, &mut Vec<Prim>) -> bool,
            anim: &mut Clock,
            out: &mut Vec<Prim>,
        ) {
            let wrapped = walk(node, anim, out);
            let children = if wrapped {
                // the wrapper's one child is the node itself: go on from its children
                &mut node.children[0].children
            } else {
                &mut node.children
            };
            for child in children {
                visit(child, walk, anim, out);
            }
        }
        visit(layout, &mut walk, anim, &mut out);
        out
    }
}
/// The runs of a label node, in order.
fn label_runs(node: &Layout, out: &mut Vec<PositionedRun>) {
    if let LayoutKind::Text { runs, .. } = &node.kind {
        out.extend(runs.iter().cloned());
    }
    for child in &node.children {
        label_runs(child, out);
    }
}
