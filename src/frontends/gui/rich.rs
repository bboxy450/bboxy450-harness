// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Worker-owned streaming documents and prepared block layouts.
use super::view::{column, context, translate};
use crate::{
    protocol::ItemId,
    runtime::{BlockingWork, blocking_hook},
};
use bbt_doc::{
    Markdown, StreamingMarkdown,
    highlight::{Highlights, RustHighlighter},
};
use bbt_ui::{self as ui, components as c, modules::seat_row::named};
use bbt_ui_model::layout::Rect;
use std::{collections::HashMap, ops::Range, sync::Arc};

/// Only prose bodies enter the document worker; terminal and failed output stay literal.
pub(super) fn tool_markdown(kind: &str, path: &str, failed: bool, text: &str) -> Option<String> {
    if kind == "execute" || failed || text.is_empty() {
        return None;
    }
    let markdown_read = kind == "read"
        && std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| {
                ["md", "markdown", "mdx"]
                    .iter()
                    .any(|e| s.eq_ignore_ascii_case(e))
            });
    let text = if markdown_read {
        text.lines()
            .map(|line| {
                let digits = line.bytes().take_while(u8::is_ascii_digit).count();
                if digits > 0
                    && let Some(rest) = line[digits..].strip_prefix(':')
                {
                    rest.strip_prefix(' ').unwrap_or(rest)
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        text.to_owned()
    };
    (markdown_read || looks_markdown(&text)).then_some(text)
}

fn looks_markdown(text: &str) -> bool {
    let mut structured = 0;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if line.len() - trimmed.len() <= 3
            && (trimmed.starts_with("```") || trimmed.starts_with("~~~"))
        {
            return true;
        }
        let heading = line.bytes().take_while(|b| *b == b'#').count();
        let ordered = trimmed.bytes().take_while(u8::is_ascii_digit).count();
        if (1..=6).contains(&heading) && line[heading..].starts_with(' ')
            || ["- ", "* ", "+ "].iter().any(|p| trimmed.starts_with(p))
            || ordered > 0 && trimmed[ordered..].starts_with(". ")
            || line.starts_with('|') && line.trim_end().ends_with('|')
            || line.starts_with("> ")
        {
            structured += 1;
            if structured >= 2 {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Face {
    pub reasoning_open: bool,
    pub source: bool,
    pub arguments_closed: bool,
    pub output_closed: bool,
    pub closed_hunks: std::collections::BTreeSet<usize>,
    pub approval_selected: usize,
}
pub type Faces = im::OrdMap<ItemId, Face>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Toggle {
    Reasoning,
    Source,
    Arguments,
    Output,
    Hunk(usize),
    Approval(usize),
}
impl Face {
    pub fn apply(&mut self, toggle: Toggle) {
        match toggle {
            Toggle::Reasoning => self.reasoning_open = !self.reasoning_open,
            Toggle::Source => self.source = !self.source,
            Toggle::Arguments => self.arguments_closed = !self.arguments_closed,
            Toggle::Output => self.output_closed = !self.output_closed,
            Toggle::Hunk(i) => {
                if !self.closed_hunks.remove(&i) {
                    self.closed_hunks.insert(i);
                }
            }
            Toggle::Approval(i) => self.approval_selected = i,
        }
    }
}
#[derive(Default)]
struct Document {
    stream: StreamingMarkdown,
    blocks: Vec<Prepared>,
    width: f32,
    /// The text zoom the blocks were laid out at; another zoom relays them out.
    zoom: f32,
    received: usize,
    parsed: usize,
    reasoning: bool,
    // A credit bounds reparsing even for one unfinished paragraph/fence or
    // document-wide reference definitions. Unparsed bytes remain visible as
    // literal tail text; completion always flushes through streaming append.
}
struct Prepared {
    source: Range<usize>,
    layout: Arc<c::Layout>,
    /// web block margins [top, bottom]
    margins: [f32; 2],
    /// paragraph, heading or list: a control may sit beside its last line
    prose: bool,
}
#[derive(Default)]
pub struct Documents {
    docs: HashMap<ItemId, Document>,
    highlighter: Option<RustHighlighter>,
    pub parsed_bytes: usize,
    pub received_bytes: usize,
    pub block_layouts: usize,
    pub appends: usize,
    pub deferred_batches: usize,
}
pub struct Body {
    pub element: c::Element,
    pub blocks: Vec<Arc<c::Layout>>,
    /// The last text line: its right edge and its baseline's distance above the body's
    /// bottom. A row can seat a small control beside it instead of on a line of its own.
    pub tail: Option<[f32; 2]>,
}
impl Body {
    pub(super) fn inset(
        &self,
        id: ItemId,
        width: f32,
        metrics: &impl c::Measure,
    ) -> Result<c::Element, String> {
        let ink = super::face::ink(context());
        let ([_, height], _) = self
            .element
            .natural(width, context(), metrics)
            .map_err(|e| format!("{e:?}"))?;
        Ok(named(
            format!("mdpanel/{id}"),
            super::face::frame(
                super::face::Frame {
                    fill: super::face::alpha(ink.bg, 0.35),
                    border: Some(super::face::alpha(ink.fg, 0.10)),
                    radius: [8.; 4],
                    padding: [8., 12., 10., 12.],
                    ..Default::default()
                },
                vec![
                    named(
                        format!("mdout/{id}"),
                        c::Element::ScrollView(c::ScrollView {
                            viewport: [width, height.min(400.)],
                            offset: [0., 0.],
                            content_width: None,
                            content: Box::new(self.element.clone()),
                        }),
                    )
                    .into(),
                ],
            ),
        ))
    }
}
impl Documents {
    /// `zoom` scales every text size in the blocks (harness-spec.md Standalone window).
    pub fn prepare(
        &mut self,
        id: ItemId,
        text: &str,
        complete: bool,
        width: f32,
        zoom: f32,
        metrics: &impl c::Measure,
    ) -> Result<Body, String> {
        self.prepare_styled(id, text, complete, width, zoom, metrics, false)
    }
    pub(super) fn reasoning(
        &mut self,
        id: ItemId,
        text: &str,
        complete: bool,
        width: f32,
        zoom: f32,
        metrics: &impl c::Measure,
    ) -> Result<Body, String> {
        self.prepare_styled(id, text, complete, width, zoom, metrics, true)
    }
    #[allow(clippy::too_many_arguments)] // The zoom rides beside the measure; one body serves both entry points.
    fn prepare_styled(
        &mut self,
        id: ItemId,
        text: &str,
        complete: bool,
        width: f32,
        zoom: f32,
        metrics: &impl c::Measure,
        reasoning: bool,
    ) -> Result<Body, String> {
        let doc = self.docs.entry(id).or_default();
        if !text.starts_with(&doc.stream.document().source) {
            *doc = Document::default();
        }
        self.received_bytes += text.len().saturating_sub(doc.received);
        doc.received = doc.received.max(text.len());
        let prior = doc.stream.document().source.len();
        let parse = prior < text.len()
            && (complete
                || prior == 0
                || doc.parsed.saturating_add(text.len()) <= text.len().saturating_mul(3));
        let mut changed = None;
        if prior < text.len() && !parse {
            self.deferred_batches += 1;
        }
        if parse {
            self.appends += 1;
            blocking_hook(BlockingWork::MarkdownPrepare);
            doc.stream.append(&text[prior..]);
            let range = doc.stream.reparsed_range();
            self.parsed_bytes += range.len();
            doc.parsed += range.len();
            changed = Some(range.start);
        }
        if doc.width != width || doc.reasoning != reasoning || doc.zoom != zoom {
            doc.blocks.clear();
            doc.width = width;
            doc.reasoning = reasoning;
            doc.zoom = zoom;
        }
        if let Some(start) = changed {
            doc.blocks.retain(|b| b.source.end <= start);
        }
        let highlighter = self
            .highlighter
            .get_or_insert_with(|| RustHighlighter::new(false));
        for block in &doc.stream.document().blocks[doc.blocks.len()..] {
            let fragment = Markdown {
                blocks: vec![block.clone()],
                ..Default::default()
            };
            let highlights = Highlights::prepare(&fragment, highlighter);
            let mut element =
                fragment.transcript_body_element(&highlights, context(), metrics, width);
            super::view::flat(&mut element);
            transcript_type(&mut element, (width - MEASURE).max(0.), false, reasoning);
            super::face::zoom_text(&mut element, zoom);
            let mut layout = element.layout(Rect::new(0., 0., width, 0.), context(), metrics)?;
            chips_and_bullets(&mut layout, metrics)?;
            let prose = matches!(
                block.kind,
                bbt_doc::BlockKind::Paragraph(_)
                    | bbt_doc::BlockKind::Heading { .. }
                    | bbt_doc::BlockKind::List { .. }
            );
            doc.blocks.push(Prepared {
                source: block.source.clone(),
                layout: Arc::new(layout),
                margins: margins(&block.kind),
                prose,
            });
            self.block_layouts += 1;
        }
        let mut blocks: Vec<_> = doc.blocks.iter().map(|b| b.layout.clone()).collect();
        let mut spacing: Vec<[f32; 2]> = doc.blocks.iter().map(|b| b.margins).collect();
        let mut prose = doc.blocks.last().is_some_and(|b| b.prose);
        let pending = &text[doc.stream.document().source.len()..];
        if !pending.is_empty() {
            // Literal streaming tail is fully readable while a pathological
            // unfinished block waits for enough parse credit. No bytes vanish.
            let mut text = c::Text::new(pending, ui::Text::default());
            text.preserve_whitespace = true;
            let mut text = c::Element::Text(text);
            super::face::zoom_text(&mut text, zoom);
            blocks.push(Arc::new(text.layout(
                Rect::new(0., 0., width, 0.),
                context(),
                metrics,
            )?));
            spacing.push([0., 0.]);
            prose = false;
        }
        // Block rhythm of the web reference (.hx-msg p/h/ul margins): adjacent margins
        // collapse to the larger one; the message keeps its first top and last bottom.
        let mut slots: Vec<c::Item> = vec![];
        let mut previous = 0_f32;
        for (i, l) in blocks.iter().enumerate() {
            let gap = if i == 0 {
                spacing[i][0]
            } else {
                previous.max(spacing[i][0])
            };
            if gap > 0. {
                slots.push(column(vec![], Some(gap)).into());
            }
            slots.push(named(format!("mdblock/{i}"), column(vec![], Some(l.rect.h))).into());
            previous = spacing[i][1];
        }
        if previous > 0. {
            slots.push(column(vec![], Some(previous)).into());
        }
        let tail = blocks.last().filter(|_| prose).and_then(|l| {
            let mut last: Option<[f32; 2]> = None;
            last_line(l, &mut last);
            last.map(|[right, baseline]| [right, l.rect.bottom() - baseline + previous])
        });
        Ok(Body {
            element: column(slots, None),
            blocks,
            tail,
        })
    }
}
/// [right edge, baseline] of the lowest text line in a block layout.
fn last_line(node: &c::Layout, out: &mut Option<[f32; 2]>) {
    if let c::LayoutKind::Text { runs, .. } = &node.kind {
        for r in runs.iter().filter(|r| !r.text.trim().is_empty()) {
            *out = match *out {
                Some([_, b]) if r.baseline > b + 0.5 => Some([r.rect.right(), r.baseline]),
                Some([x, b]) if (r.baseline - b).abs() <= 0.5 => Some([x.max(r.rect.right()), b]),
                None => Some([r.rect.right(), r.baseline]),
                keep => keep,
            };
        }
    }
    for child in &node.children {
        last_line(child, out);
    }
}
pub fn restore(node: &mut c::Layout, blocks: &[Arc<c::Layout>]) {
    if let c::LayoutKind::Named(name) = &node.kind
        && let Some(index) = name
            .strip_prefix("mdblock/")
            .and_then(|s| s.parse::<usize>().ok())
        && let Some(block) = blocks.get(index)
    {
        let mut solved = (**block).clone();
        let dx = node.rect.x - solved.rect.x;
        let dy = node.rect.y - solved.rect.y;
        translate(&mut solved, dx, dy);
        // Keep the block boundary around the restored layout: smoke and hit
        // inspection must see the geometry we actually paint, not empty slots.
        node.rect = solved.rect;
        node.children = vec![solved];
        return;
    }
    for child in &mut node.children {
        restore(child, blocks);
    }
}

// ---------- transcript typography ----------
// The web reference (bboxy450-term lab/web/lab.css .hx-msg) sets agent prose at 13.5 px on
// the full feed column, headings at 15 px, and `code` as a chip: .88em mono on a hairline
// rounded card. The toolkit's document renderer keeps a 576 px reading measure and plain
// mono runs, so the transcript adapts its prepared element here, before layout.
// Translated from https://github.com/bboxy450/bboxy450-term (lab/web/lab.css); policy
// https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

/// The toolkit's reading measure (style.json spacing.doc_measure).
const MEASURE: f32 = bbt_ui_model::STYLE.spacing.doc_measure as f32;
const BODY: f32 = 13.5;
const BODY_LINE: f32 = 1.45;
const HEADING: f32 = 15.;
/// .88em of the body: a 1/64 px value no other transcript text uses, so the chip pass
/// recognises inline code after layout without a side table.
const CHIP_TEXT: f32 = 11.875;
/// Two thin spaces each side stand in for the chip's 5 px padding and 1 px border.
const THIN: &str = "\u{2009}\u{2009}";

/// Web block margins [top, bottom] (.hx-msg p, h3..h6, ul/ol, .hx-code, .dgfig).
fn margins(kind: &bbt_doc::BlockKind) -> [f32; 2] {
    use bbt_doc::BlockKind as K;
    match kind {
        K::Paragraph(_) => [0., 6.],
        K::Heading { .. } => [8., 6.],
        K::List { .. } => [2., 8.],
        K::Code { info, .. } if info.split_whitespace().next() == Some("mermaid") => [8., 8.],
        K::Code { .. } => [6., 6.],
        K::Rule => [8., 8.],
        _ => [6., 6.],
    }
}

fn widen(v: &mut f32, delta: f32) {
    // Only quantities derived from the reading measure (nested lists and quotes step
    // inward from it); table and figure widths already use the full column.
    if delta > 0. && *v <= MEASURE + 0.01 && *v >= MEASURE - 120. {
        *v += delta;
    }
}

fn transcript_type(e: &mut c::Element, delta: f32, in_flow: bool, reasoning: bool) {
    match e {
        c::Element::Box(b) => {
            if let Some(ui::Length::Px(w)) = &mut b.style.overrides.width {
                widen(w, delta);
            }
            // a list: its items are marker layers; .hx-msg li's 2 px margins collapse to a
            // 2 px gap between items (harness-spec.md "Refinement pass 2")
            if !b.children.is_empty()
                && b.children
                    .iter()
                    .all(|i| matches!(i.element, c::Element::Layers { .. }))
            {
                b.gap = ui::Length::Px(2.);
            }
            for item in &mut b.children {
                if let c::Extent::Fixed(w) = &mut item.extent {
                    widen(w, delta);
                }
                transcript_type(&mut item.element, delta, false, reasoning);
            }
        }
        c::Element::Stack(s) => {
            for item in &mut s.children {
                transcript_type(&mut item.element, delta, false, reasoning);
            }
        }
        c::Element::Flow(f) => {
            for item in &mut f.children {
                transcript_type(&mut item.element, delta, true, reasoning);
            }
        }
        c::Element::ScrollView(s) => {
            widen(&mut s.viewport[0], delta);
            transcript_type(&mut s.content, delta, false, reasoning);
        }
        c::Element::Named { content, .. }
        | c::Element::FixedHeight { content, .. }
        | c::Element::Filter { content, .. }
        | c::Element::InlineMiddle { content, .. }
        | c::Element::Baseline { content, .. } => {
            transcript_type(content, delta, in_flow, reasoning)
        }
        c::Element::Overlay(o) => transcript_type(&mut o.content, delta, false, reasoning),
        c::Element::Layers { base, overlays } => {
            transcript_type(base, delta, false, reasoning);
            for p in overlays {
                transcript_type(&mut p.content, delta, false, reasoning);
                // li::marker takes the seat colour
                if let c::Element::Named { name, content } = &mut p.content
                    && name.contains("/bullet")
                    && let c::Element::Text(t) = content.as_mut()
                {
                    for r in &mut t.runs {
                        r.style.paint = ui::Paint::solid(ui::Color::Accent);
                    }
                }
            }
        }
        c::Element::Text(t) => {
            let mono = t.runs.iter().any(|r| r.style.mono);
            let prose = t.runs.iter().any(|r| !r.style.mono);
            // a code block is all mono; inline code shares its text with prose, or is a
            // lone word of a wrapped flow line
            let inline = mono && (prose || in_flow);
            let mut runs = Vec::with_capacity(t.runs.len() + 2);
            for mut run in std::mem::take(&mut t.runs) {
                let s = &mut run.style;
                if !s.mono && s.step == ui::TypeStep::Body {
                    match s.size {
                        _ if reasoning => {
                            s.size = Some(12.5);
                            s.line_height = BODY_LINE;
                            s.paint = super::face::paint(super::face::ink(context()).dim);
                        }
                        None => {
                            s.size = Some(BODY);
                            s.line_height = BODY_LINE;
                        }
                        Some(px) if s.weight >= 700 && px > BODY => {
                            s.size = Some(HEADING);
                            s.line_height = 1.4;
                        }
                        _ => {}
                    }
                    // .hx-msg .arw: → and ← take the seat colour, bold
                    if run.value.contains(['→', '←']) {
                        let mut rest = run.value.as_str();
                        while let Some(i) = rest.find(['→', '←']) {
                            let arrow = rest[i..].chars().next().unwrap_or('→');
                            if i > 0 {
                                runs.push(c::TextRun {
                                    value: rest[..i].into(),
                                    style: run.style,
                                });
                            }
                            runs.push(c::TextRun {
                                value: arrow.into(),
                                style: ui::Text {
                                    weight: 700,
                                    paint: ui::Paint::solid(ui::Color::Accent),
                                    ..run.style
                                },
                            });
                            rest = &rest[i + arrow.len_utf8()..];
                        }
                        if !rest.is_empty() {
                            runs.push(c::TextRun {
                                value: rest.into(),
                                style: run.style,
                            });
                        }
                        continue;
                    }
                    runs.push(run);
                } else if inline && s.mono {
                    s.size = Some(CHIP_TEXT);
                    let pad = ui::Text {
                        mono: false,
                        size: Some(BODY),
                        line_height: BODY_LINE,
                        ..*s
                    };
                    runs.push(c::TextRun {
                        value: THIN.into(),
                        style: pad,
                    });
                    runs.push(run);
                    runs.push(c::TextRun {
                        value: THIN.into(),
                        style: pad,
                    });
                } else {
                    runs.push(run);
                }
            }
            t.runs = runs;
        }
        _ => {}
    }
}

fn rounded(w: f32, h: f32, r: f32) -> Vec<bbt_ui_model::draw::PathCommand> {
    use bbt_ui_model::draw::PathCommand as P;
    let k = r * 0.552_284_8;
    let (x0, y0, x1, y1) = (0.5, 0.5, w - 0.5, h - 0.5);
    vec![
        P::Move([x0 + r, y0]),
        P::Line([x1 - r, y0]),
        P::Cubic([[x1 - r + k, y0], [x1, y0 + r - k], [x1, y0 + r]]),
        P::Line([x1, y1 - r]),
        P::Cubic([[x1, y1 - r + k], [x1 - r + k, y1], [x1 - r, y1]]),
        P::Line([x0 + r, y1]),
        P::Cubic([[x0 + r - k, y1], [x0, y1 - r + k], [x0, y1 - r]]),
        P::Line([x0, y0 + r]),
        P::Cubic([[x0, y0 + r - k], [x0 + r - k, y0], [x0 + r, y0]]),
    ]
}

/// After layout: paint a chip behind each inline-code run, and set each list marker on
/// its item's first line (the toolkit places it from the reading-measure height).
fn chips_and_bullets(node: &mut c::Layout, metrics: &impl c::Measure) -> Result<(), String> {
    if matches!(node.kind, c::LayoutKind::Stack)
        && node.children.len() == 2
        && matches!(&node.children[1].kind, c::LayoutKind::Named(n) if n.contains("/bullet"))
    {
        let dy = node.children[0].rect.y - node.children[1].rect.y;
        translate(&mut node.children[1], 0., dy);
    }
    let mut out = Vec::with_capacity(node.children.len());
    for mut child in std::mem::take(&mut node.children) {
        if let c::LayoutKind::Text { runs, .. } = &child.kind {
            let ink = super::face::ink(context());
            for run in runs
                .iter()
                .filter(|r| r.style.mono && (r.style.size - CHIP_TEXT).abs() < 0.001)
                .filter(|r| !r.text.trim().is_empty())
            {
                let h = (CHIP_TEXT * 1.25 + 3.).round();
                let w = run.rect.w + 10.;
                let y = (run.baseline - CHIP_TEXT * 0.36 - h / 2.).round();
                let path = c::Element::Path(c::Path {
                    size: [w, h],
                    points: Arc::from(rounded(w, h, 5.)),
                    stroke: bbt_ui_model::draw::PathStroke {
                        width: 1.,
                        ..Default::default()
                    },
                    paint: super::face::paint(super::face::alpha(ink.fg, 0.10)),
                    fill: Some(super::face::paint(super::face::alpha(ink.fg, 0.07))),
                    opacity: 1.,
                });
                out.push(path.layout(Rect::new(run.rect.x - 5., y, w, h), context(), metrics)?);
            }
        }
        chips_and_bullets(&mut child, metrics)?;
        out.push(child);
    }
    node.children = out;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::tool_markdown;

    #[test]
    fn tool_markdown_detection_accepts_structure_and_markdown_reads() {
        for text in [
            "```mermaid\ngraph LR\nA --> B\n```",
            "~~~rust\nfn main() {}\n~~~",
            "# Heading\n## Next",
            "- one\n  + two",
            "1. one\n2. two",
            "| a | b |\n|---|---|",
            "> one\n> two",
            "# Heading\n- item",
        ] {
            assert_eq!(
                tool_markdown("other", "", false, text).as_deref(),
                Some(text)
            );
        }
        for path in ["README.md", "notes.markdown", "page.MDX"] {
            assert_eq!(
                tool_markdown(
                    "read",
                    path,
                    false,
                    "12: # Title\n13:\n14: - one\n15: - two"
                ),
                Some("# Title\n\n- one\n- two".into())
            );
        }
    }

    #[test]
    fn tool_markdown_detection_rejects_plain_failed_and_execute_output() {
        for text in [
            "plain output\nsecond line",
            "# one heading\nplain body",
            "- one item",
            "value: # heading\nnot > a quote",
            "####### too deep\n####### still too deep",
            "x ``` embedded fence",
            "",
        ] {
            assert!(
                tool_markdown("other", "", false, text).is_none(),
                "{text:?}"
            );
        }
        let markdown = "# Title\n- item\n```mermaid\ngraph LR\nA --> B\n```";
        assert!(tool_markdown("execute", "README.md", false, markdown).is_none());
        assert!(tool_markdown("read", "README.md", true, markdown).is_none());
        assert!(tool_markdown("other", "", true, markdown).is_none());
        assert!(tool_markdown("read", "README.md.txt", false, "plain").is_none());
        assert!(tool_markdown("other", "README.md", false, "plain").is_none());
    }
}
