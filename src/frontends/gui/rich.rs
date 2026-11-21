// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Worker-owned streaming documents and prepared block layouts.
use super::view::{column, translate};
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

/// A conservative, borrowed Python view. Parsing never starts a shell or changes source bytes.
/// Origin/credit: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#[derive(Debug, PartialEq, Eq)]
pub struct PythonSnippet<'a> {
    pub source: &'a str,
    pub original: &'a str,
    /// A literal `cd DIR &&` (or `;`) before the interpreter: where it runs.
    pub cwd: Option<&'a str>,
}
/// Agents often run Python from a directory: `cd DIR && python3 - <<'PY'`. Only a
/// plain or single-quoted path is admitted; any expansion stays a shell command.
fn cd_prefix(command: &str) -> Option<(&str, &str)> {
    let rest = command.trim_start().strip_prefix("cd ")?.trim_start();
    let (dir, after) = if let Some(quoted) = rest.strip_prefix('\'') {
        let end = quoted.find('\'')?;
        (&quoted[..end], &quoted[end + 1..])
    } else {
        let end = rest.find(|c: char| c.is_whitespace() || c == ';' || c == '&')?;
        let dir = &rest[..end];
        if !dir
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_./-~@+".contains(c))
        {
            return None;
        }
        (dir, &rest[end..])
    };
    if dir.is_empty() || dir.contains('\n') {
        return None;
    }
    let after = after.trim_start();
    let after = after
        .strip_prefix("&&")
        .or_else(|| after.strip_prefix(';'))?;
    Some((dir, after.trim_start()))
}
pub fn python_snippet(command: &str) -> Option<PythonSnippet<'_>> {
    if let Some((dir, rest)) = cd_prefix(command) {
        let mut snippet = interpreter_snippet(rest)?;
        snippet.original = command;
        snippet.cwd = Some(dir);
        return Some(snippet);
    }
    interpreter_snippet(command)
}
fn interpreter_snippet(command: &str) -> Option<PythonSnippet<'_>> {
    fn executable(word: &str) -> bool {
        if !word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_./-".contains(c))
        {
            return false;
        }
        let name = word.rsplit('/').next().unwrap_or("");
        matches!(name, "python" | "python3")
            || name
                .strip_prefix("python3.")
                .is_some_and(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
    }
    fn invocation(prefix: &str) -> bool {
        let mut words = prefix.split_whitespace();
        words.next().is_some_and(executable)
            && words.all(|w| matches!(w, "-" | "-u" | "-B" | "-I" | "-E" | "-s" | "-S"))
    }
    if let Some((header, rest)) = command.split_once('\n')
        && let Some((prefix, delimiter)) = header.trim_end_matches('\r').split_once("<<")
    {
        if !invocation(prefix) {
            return None;
        }
        let delimiter = delimiter.trim();
        let (delimiter, quoted) =
            if let Some(q) = delimiter.chars().next().filter(|c| matches!(c, '\'' | '"')) {
                (delimiter.strip_prefix(q)?.strip_suffix(q)?, true)
            } else {
                (delimiter, false)
            };
        if delimiter.is_empty()
            || !delimiter
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return None;
        }
        let mut offset = 0;
        for line in rest.split_inclusive('\n') {
            if line.trim_end_matches(['\r', '\n']) == delimiter {
                if !rest[offset + line.len()..].trim().is_empty() {
                    return None;
                }
                let source = &rest[..offset];
                if !quoted && source.contains(['$', '`', '\\']) {
                    return None;
                }
                return Some(PythonSnippet {
                    source,
                    original: command,
                    cwd: None,
                });
            }
            offset += line.len();
        }
        return None;
    }
    // Only a single literal shell argument is admitted. Interpolation, adjacent
    // shell strings, escapes in double quotes and trailing commands stay shell.
    let mut rest = command.trim_start();
    let n = rest.find(char::is_whitespace)?;
    if !executable(&rest[..n]) {
        return None;
    }
    rest = rest[n..].trim_start();
    loop {
        let n = rest.find(char::is_whitespace)?;
        let word = &rest[..n];
        rest = rest[n..].trim_start();
        if word == "-c" {
            break;
        }
        if !matches!(word, "-u" | "-B" | "-I" | "-E" | "-s" | "-S") {
            return None;
        }
    }
    let quote = rest.chars().next().filter(|c| matches!(c, '\'' | '"'))?;
    let rest = rest.strip_prefix(quote)?;
    let end = rest.find(quote)?;
    let source = &rest[..end];
    if !rest[end + 1..].trim().is_empty() || quote == '"' && source.contains(['$', '`', '\\']) {
        return None;
    }
    Some(PythonSnippet {
        source,
        original: command,
        cwd: None,
    })
}

/// Bound individual native text producers without dropping or reordering UTF-8.
/// The shared raster backend owns its own device limits; this is a small producer bound.
pub const NATIVE_RUN_BYTES: usize = 128;
pub fn bound_native_runs(element: &mut c::Element) {
    match element {
        c::Element::Text(text) => {
            let mut runs = vec![];
            for run in std::mem::take(&mut text.runs) {
                let mut rest = run.value.as_str();
                while !rest.is_empty() {
                    let mut n = rest.len().min(NATIVE_RUN_BYTES);
                    while !rest.is_char_boundary(n) {
                        n -= 1;
                    }
                    runs.push(c::TextRun {
                        value: rest[..n].into(),
                        style: run.style,
                    });
                    rest = &rest[n..];
                }
            }
            text.runs = runs;
        }
        c::Element::Box(b) => {
            for item in &mut b.children {
                bound_native_runs(&mut item.element)
            }
        }
        c::Element::Stack(b) => {
            for item in &mut b.children {
                bound_native_runs(&mut item.element)
            }
        }
        c::Element::Flow(b) => {
            for item in &mut b.children {
                bound_native_runs(&mut item.element)
            }
        }
        c::Element::Named { content, .. }
        | c::Element::FixedHeight { content, .. }
        | c::Element::Filter { content, .. }
        | c::Element::InlineMiddle { content, .. }
        | c::Element::Baseline { content, .. } => bound_native_runs(content),
        c::Element::ScrollView(v) => bound_native_runs(&mut v.content),
        c::Element::Overlay(v) => bound_native_runs(&mut v.content),
        c::Element::Layers { base, overlays } => {
            bound_native_runs(base);
            for p in overlays {
                bound_native_runs(&mut p.content);
            }
        }
        _ => {}
    }
}

/// Read the typed display results of an actual Python tool reply. The runtime
/// keeps stdout and ordinary emitted values separate. Never search either for embeds.
pub(super) fn python_displayed(
    execution: &str,
    raw: Option<&serde_json::Value>,
    output: &[crate::protocol::OutputBlock],
) -> Vec<String> {
    use serde_json::Value;
    fn read(execution: &str, value: &Value, depth: u8) -> Option<Vec<String>> {
        if depth == 0 {
            return None;
        }
        if value["execution_id"].as_str() == Some(execution) {
            return Some(
                value["displayed"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .take(256)
                    .filter_map(Value::as_str)
                    .filter(|value| crate::python_execute::display_reference(value))
                    .map(str::to_owned)
                    .collect(),
            );
        }
        let inner = crate::frontends::face_core::codex_outcome(value)
            .and_then(Result::ok)
            .or_else(|| value.get("structuredContent"))
            .or_else(|| {
                value.get("result").filter(|inner| {
                    inner.get("content").is_some() || inner.get("structuredContent").is_some()
                })
            });
        inner
            .and_then(|v| read(execution, v, depth - 1))
            .or_else(|| {
                value["content"].as_array()?.iter().find_map(|v| {
                    let raw: Value = serde_json::from_str(v["text"].as_str()?).ok()?;
                    read(execution, &raw, depth - 1)
                })
            })
    }

    raw.and_then(|value| read(execution, value, 6))
        .or_else(|| {
            output.iter().find_map(|block| {
                use crate::protocol::OutputBlock;
                match block {
                    OutputBlock::Json { value } => read(execution, value, 6),
                    OutputBlock::Text { text } | OutputBlock::Error { text } => {
                        let value: Value = serde_json::from_str(text).ok()?;
                        read(execution, &value, 6)
                    }
                    _ => None,
                }
            })
        })
        .unwrap_or_default()
}

/// Structured prose and valid JSON use the shared document worker.
pub(super) fn tool_markdown(kind: &str, path: &str, failed: bool, text: &str) -> Option<String> {
    if json_output(text) {
        return Some(format!("```json\n{text}\n```"));
    }
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

pub(super) fn json_output(text: &str) -> bool {
    matches!(text.trim_start().chars().next(), Some('{' | '['))
        && serde_json::from_str::<serde_json::Value>(text).is_ok()
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
    /// A Question's picked options, as (field, value).
    pub picked: std::collections::BTreeSet<(String, String)>,
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
    /// Pick a Question option; a single-select field keeps one value.
    Pick {
        field: String,
        value: String,
        multi: bool,
    },
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
            Toggle::Pick {
                field,
                value,
                multi,
            } => {
                let pick = (field, value);
                if self.picked.remove(&pick) {
                    return;
                }
                if !multi {
                    self.picked.retain(|(f, _)| *f != pick.0);
                }
                self.picked.insert(pick);
            }
        }
    }
}
type Image = Arc<bbt_render::chrome::icons::Image>;
type LoadImages = dyn Fn(&mut Markdown) -> Vec<Image> + Send + Sync;

/// Host-owned resolution. Replacing the handle invalidates unchanged Markdown.
/// Standalone surfaces never install a loader.
pub struct ImagePreparation(Box<LoadImages>);

impl ImagePreparation {
    pub fn new(load: impl Fn(&mut Markdown) -> Vec<Image> + Send + Sync + 'static) -> Arc<Self> {
        Arc::new(Self(Box::new(load)))
    }

    pub(super) fn same(a: &Option<Arc<Self>>, b: &Option<Arc<Self>>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Default)]
struct Document {
    stream: StreamingMarkdown,
    prepared_images: Option<Markdown>,
    images: Vec<Image>,
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
    theme: ui::Theme,
    docs: HashMap<ItemId, Document>,
    image_preparation: Option<Arc<ImagePreparation>>,
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
        ctx: ui::Context,
    ) -> Result<c::Element, String> {
        let ink = super::face::ink(ctx);
        let ([_, height], _) = self
            .element
            .natural(width, ctx, metrics)
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
    pub(super) fn set_image_preparation(
        &mut self,
        preparation: Option<Arc<ImagePreparation>>,
    ) -> bool {
        if ImagePreparation::same(&self.image_preparation, &preparation) {
            return false;
        }
        self.image_preparation = preparation;
        for doc in self.docs.values_mut() {
            if doc.prepared_images.take().is_some() {
                doc.blocks.clear();
                doc.images.clear();
            }
        }
        true
    }

    pub(super) fn reset_session(&mut self) {
        *self = Self {
            theme: self.theme,
            image_preparation: self.image_preparation.take(),
            ..Default::default()
        };
    }

    pub(super) fn images(&self) -> impl Iterator<Item = &Image> {
        self.docs.values().flat_map(|doc| &doc.images)
    }

    pub fn set_theme(&mut self, theme: ui::Theme) -> bool {
        if self.theme == theme {
            return false;
        }
        self.theme = theme;
        self.highlighter = None;
        for doc in self.docs.values_mut() {
            doc.blocks.clear();
        }
        true
    }
    pub(super) fn context(&self) -> ui::Context {
        ui::Context::new(self.theme, ui::Density::Cozy)
    }
    /// Reuse the shared Markdown Code block and syntect-backed highlighter.
    /// Only the literal code subtree is returned; the harness owns its outer viewport.
    pub fn code_element(&mut self, source: &str, metrics: &impl c::Measure) -> c::Element {
        self.code_element_language(source, "python", metrics)
    }
    pub fn code_element_language(
        &mut self,
        source: &str,
        language: &str,
        metrics: &impl c::Measure,
    ) -> c::Element {
        bbt_rt::assert_not_ui_thread();
        let ctx = self.context();
        let document = Markdown {
            source: source.into(),
            blocks: vec![bbt_doc::Block {
                source: 0..source.len(),
                kind: bbt_doc::BlockKind::Code {
                    info: language.into(),
                    text: source.into(),
                },
            }],
            ..Default::default()
        };
        let highlighter = self
            .highlighter
            .get_or_insert_with(|| RustHighlighter::new(self.theme == ui::Theme::Light));
        let highlights = Highlights::prepare(&document, highlighter);
        let element = document.transcript_body_element(&highlights, ctx, metrics, 576.);
        // The code subtree is found by name, never by the shape around it:
        // bbt-doc's frame (2026-11-16) wrapped framed code in `Layers` with no
        // `block0` node, and the `expect` here panicked the pane's preparer on
        // every Python source and shell command, freezing the view.
        fn find(e: &c::Element, wanted: &str) -> Option<c::Element> {
            match e {
                c::Element::Named { name, content } if name == wanted => Some((**content).clone()),
                c::Element::Named { content, .. } => find(content, wanted),
                c::Element::Box(b) => b.children.iter().find_map(|i| find(&i.element, wanted)),
                c::Element::Layers { base, .. } => find(base, wanted),
                c::Element::ScrollView(s) => find(&s.content, wanted),
                _ => None,
            }
        }
        fn unscroll(e: &mut c::Element) {
            if let c::Element::ScrollView(s) = e {
                *e = *s.content.clone();
                unscroll(e);
                return;
            }
            match e {
                c::Element::Box(b) => {
                    for i in &mut b.children {
                        unscroll(&mut i.element)
                    }
                }
                c::Element::Named { content, .. } => unscroll(content),
                _ => {}
            }
        }
        let mut code = match find(&element, "block0").or_else(|| {
            find(&element, "block0/text").map(|text| c::Element::Named {
                name: "block0/text".into(),
                content: Box::new(text),
            })
        }) {
            Some(code) => code,
            None => {
                bbt_rt::error_journal::report(bbt_rt::error_journal::Failure::new(
                    "gui",
                    "code_block_shape",
                    format!(
                        "the shared {language} code block had no block0 node; shown as plain text"
                    ),
                ));
                let mut text = c::Text::new(
                    source,
                    ui::Text {
                        mono: true,
                        line_height: 1.6,
                        ..Default::default()
                    },
                );
                text.wrap = false;
                text.preserve_whitespace = true;
                c::Element::Text(text)
            }
        };
        unscroll(&mut code);
        super::view::flat(&mut code);
        bound_native_runs(&mut code);
        code
    }
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
        let ctx = self.context();
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
        if let Some(preparation) = &self.image_preparation
            && (parse || doc.prepared_images.is_none())
            && doc.stream.document().source.contains("![")
        {
            let mut markdown = doc.stream.document().clone();
            doc.images = (preparation.0)(&mut markdown);
            doc.prepared_images = Some(markdown);
        }
        let markdown = doc
            .prepared_images
            .as_ref()
            .unwrap_or_else(|| doc.stream.document());
        let highlighter = self
            .highlighter
            .get_or_insert_with(|| RustHighlighter::new(self.theme == ui::Theme::Light));
        for block in &markdown.blocks[doc.blocks.len()..] {
            let fragment = Markdown {
                blocks: vec![block.clone()],
                ..Default::default()
            };
            let highlights = Highlights::prepare(&fragment, highlighter);
            let mut element = fragment.transcript_body_element(&highlights, ctx, metrics, width);
            super::view::flat(&mut element);
            transcript_type(
                &mut element,
                (width - MEASURE).max(0.),
                false,
                reasoning,
                ctx,
            );
            super::face::zoom_text(&mut element, zoom);
            bound_native_runs(&mut element);
            let mut layout = element.layout(Rect::new(0., 0., width, 0.), ctx, metrics)?;
            chips_and_bullets(&mut layout, metrics, ctx)?;
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
            bound_native_runs(&mut text);
            blocks.push(Arc::new(text.layout(
                Rect::new(0., 0., width, 0.),
                ctx,
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
        let mut element = column(slots, None);
        if let c::Element::Box(b) = &mut element {
            b.style.overrides.width = Some(ui::Length::Px(width));
        }
        Ok(Body {
            element,
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

fn transcript_type(
    e: &mut c::Element,
    delta: f32,
    in_flow: bool,
    reasoning: bool,
    ctx: ui::Context,
) {
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
                transcript_type(&mut item.element, delta, false, reasoning, ctx);
            }
        }
        c::Element::Stack(s) => {
            for item in &mut s.children {
                transcript_type(&mut item.element, delta, false, reasoning, ctx);
            }
        }
        c::Element::Flow(f) => {
            for item in &mut f.children {
                transcript_type(&mut item.element, delta, true, reasoning, ctx);
            }
        }
        c::Element::ScrollView(s) => {
            widen(&mut s.viewport[0], delta);
            transcript_type(&mut s.content, delta, false, reasoning, ctx);
        }
        c::Element::Named { content, .. }
        | c::Element::FixedHeight { content, .. }
        | c::Element::Filter { content, .. }
        | c::Element::InlineMiddle { content, .. }
        | c::Element::Baseline { content, .. } => {
            transcript_type(content, delta, in_flow, reasoning, ctx)
        }
        c::Element::Overlay(o) => transcript_type(&mut o.content, delta, false, reasoning, ctx),
        c::Element::Layers { base, overlays } => {
            transcript_type(base, delta, false, reasoning, ctx);
            for p in overlays {
                transcript_type(&mut p.content, delta, false, reasoning, ctx);
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
            // Inline fragments share one line: retain the original separators
            // between emphasis, links and code rather than trimming each fragment.
            if in_flow {
                t.preserve_whitespace = true;
            }
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
                            s.paint = super::face::paint(super::face::ink(ctx).dim);
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
fn chips_and_bullets(
    node: &mut c::Layout,
    metrics: &impl c::Measure,
    ctx: ui::Context,
) -> Result<(), String> {
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
            let ink = super::face::ink(ctx);
            for run in runs
                .iter()
                .filter(|r| r.style.mono && (r.style.size - CHIP_TEXT).abs() < 0.001)
                .filter(|r| !r.text.trim().is_empty())
            {
                let h = (CHIP_TEXT * 1.25 + 3.).round();
                // Native thin-space advances can be less than the web's 5px
                // chip padding, especially at a wrapped line edge. Keep paint
                // inside the measured line instead of clipping into its neighbor.
                let x = (run.rect.x - 5.).max(child.rect.x);
                let right = (run.rect.right() + 5.).min(child.rect.right());
                let w = (right - x).max(0.);
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
                out.push(path.layout(Rect::new(x, y, w, h), ctx, metrics)?);
            }
        }
        chips_and_bullets(&mut child, metrics, ctx)?;
        out.push(child);
    }
    node.children = out;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::tool_markdown;

    #[tokio::test]
    async fn image_preparation_replaces_the_layout_for_unchanged_text() {
        use super::{Documents, ImagePreparation};
        use crate::protocol::ItemId;
        use bbt_doc::{BlockKind, Inline, markdown::ImageAsset};
        use bbt_ui::components::{Layout, LayoutKind};
        use std::sync::Arc;

        fn has_image(layout: &Layout, name: &str) -> bool {
            matches!(&layout.kind, LayoutKind::Image { asset, .. } if asset == name)
                || layout.children.iter().any(|child| has_image(child, name))
        }

        crate::runtime::blocking(|| {
            let mut docs = Documents::default();
            let metrics = super::super::view::Metrics::default();
            let id = ItemId::new();
            let text = "Before ![sample](images/probe.png) after.";
            for name in ["image/a", "image/b", "image/a"] {
                let prepare = ImagePreparation::new(move |markdown| {
                    let BlockKind::Paragraph(inlines) = &mut markdown.blocks[0].kind else {
                        panic!("fixture needs a paragraph");
                    };
                    let asset = inlines
                        .iter_mut()
                        .find_map(|inline| match inline {
                            Inline::Image { asset, .. } => Some(asset),
                            _ => None,
                        })
                        .expect("fixture needs an image");
                    *asset = Some(
                        serde_json::from_value::<ImageAsset>(serde_json::json!({
                            "key": name, "width": 20, "height": 12,
                        }))
                        .expect("bitmap metadata"),
                    );
                    vec![Arc::new(bbt_render::chrome::icons::Image {
                        name: name.into(),
                        width: 20,
                        height: 12,
                        rgba: vec![255; 20 * 12 * 4],
                    })]
                });
                assert!(docs.set_image_preparation(Some(prepare)));
                let body = docs.prepare(id, text, true, 480., 1., &metrics).unwrap();
                assert!(body.blocks.iter().any(|layout| has_image(layout, name)));
                assert_eq!(docs.images().count(), 1);
            }
            assert!(docs.set_image_preparation(None));
            let body = docs.prepare(id, text, true, 480., 1., &metrics).unwrap();
            assert!(
                body.blocks
                    .iter()
                    .all(|layout| !has_image(layout, "image/a"))
            );
            assert_eq!(docs.images().count(), 0);
        })
        .await;
    }

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
    fn json_tool_results_keep_literal_source_and_error_fields() {
        let source = r#"{"status":"rejected","error":{"message":"Keep **literal** text"}}"#;
        assert_eq!(
            tool_markdown("execute", "", true, source),
            Some(format!("```json\n{source}\n```"))
        );
        assert!(tool_markdown("other", "", false, "{invalid json}").is_none());
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

#[cfg(test)]
mod gui_polish_geometry {
    use super::*;
    #[tokio::test]
    async fn gui_polish_inline_code_chips_stay_inside_their_text_line() {
        fn check(node: &c::Layout, chips: &mut usize) {
            for pair in node.children.windows(2) {
                if matches!(pair[0].kind, c::LayoutKind::Path { .. })
                    && let c::LayoutKind::Text { runs, .. } = &pair[1].kind
                    && runs
                        .iter()
                        .any(|r| r.style.mono && (r.style.size - CHIP_TEXT).abs() < 0.001)
                {
                    *chips += 1;
                    let chip = pair[0].rect;
                    let line = pair[1].rect;
                    assert!(
                        chip.x >= line.x - 0.5 && chip.right() <= line.right() + 0.5,
                        "inline code paint escaped its line: chip={chip:?} line={line:?}"
                    );
                }
            }
            // Newer toolkits (term's in-tree bboxy450-doc) draw inline code themselves as a
            // named `…/capsule`; the harness then paints no chip of its own. Either witness
            // must stay inside the line it sits on.
            for child in &node.children {
                if matches!(&child.kind, c::LayoutKind::Named(n) if n.ends_with("/capsule")) {
                    *chips += 1;
                    let (chip, line) = (child.rect, node.rect);
                    assert!(
                        chip.x >= line.x - 0.5 && chip.right() <= line.right() + 0.5,
                        "inline code capsule escaped its line: chip={chip:?} line={line:?}"
                    );
                }
            }
            for child in &node.children {
                check(child, chips);
            }
        }
        crate::runtime::blocking(|| {
        let metrics = super::super::view::Metrics::default();
        let mut chips = 0;
        for width in [200., 290., 640.] {
            let mut docs = Documents::default();
            let text = "`bboxy450-harness` then a wrapped `bboxy450-harness-bboxy450-harness-bboxy450-harness` near the edge.";
            let body = docs
                .prepare(ItemId::new(), text, true, width, 1., &metrics)
                .unwrap();
            for block in body.blocks {
                check(&block, &mut chips);
            }
        }
        assert!(chips > 0, "chip witness absent");
        }).await;
    }
}

#[cfg(test)]
mod quiet_code_tests {
    use super::*;
    #[test]
    fn quiet_code_parser_preserves_literal_python_bytes_and_rejects_shell_mixtures() {
        for command in [
            "python3 - <<'PY'\nfrom pathlib import Path\nprint('雪 🦊')\nPY\n",
            "python -u <<\"END\"\r\nprint('雪')\r\nEND\r\n",
            "python3 <<PY\nprint(42)\nPY",
            "python3 -I -c 'import json; print(\"雪\")'",
            "python -c \"print('snow')\"",
            "cd /Users/me/twig/ui && python3 - <<'PY'\nprint(open('index.html').read())\nPY",
            "cd '/tmp/a b'; python3 -c 'print(1)'",
        ] {
            let parsed = python_snippet(command).expect(command);
            assert_eq!(parsed.original.as_bytes(), command.as_bytes());
            assert!(
                command
                    .as_bytes()
                    .windows(parsed.source.len())
                    .any(|w| w == parsed.source.as_bytes())
            );
            assert!(parsed.source.contains("print"));
        }
        for command in [
            "python3 - <<'PY'\nprint(1)\nPY\ngit add .",
            "echo x; python3 -c 'print(1)'",
            "python3 -c 'print(1)' && git status",
            "python3 -c \"print('$HOME')\"",
            "python3 <<PY\nprint('$HOME')\nPY",
            "python3 - <<-PY\n\tprint(1)\n\tPY",
            "node -e 'console.log(1)'",
            "git add file\ngit commit",
            "python3 -m pip install x",
            "cd $HOME && python3 -c 'print(1)'",
            "cd a && b && python3 -c 'print(1)'",
            "cd `pwd` && python3 -c 'print(1)'",
            "cd x && python3 - <<'PY'\nprint(1)\nPY\ngit add .",
        ] {
            assert!(
                python_snippet(command).is_none(),
                "ambiguous/non-Python command: {command}"
            );
        }
    }
    #[test]
    fn quiet_code_native_chunks_preserve_all_unicode_bytes() {
        let source = format!("first:{}:last", "α雪🦊\\\\\"".repeat(10_000));
        let mut element = super::super::face::runs(
            vec![(source.clone(), [1.; 4], 400, false)],
            12.,
            true,
            false,
        );
        bound_native_runs(&mut element);
        let c::Element::Text(text) = element else {
            panic!("literal text")
        };
        assert!(text.runs.iter().all(|r| r.value.len() <= NATIVE_RUN_BYTES));
        assert_eq!(
            text.runs
                .iter()
                .map(|r| r.value.as_str())
                .collect::<String>()
                .as_bytes(),
            source.as_bytes()
        );
    }
    #[tokio::test]
    async fn quiet_code_python_uses_shared_markdown_highlight_spans() {
        crate::runtime::blocking(|| {
            use bbt_doc::highlight::Highlighter;
            let source = "from pathlib import Path\nvalue = '雪 🦊'\nprint(value, 42)\n";
            let spans = RustHighlighter::new(false).highlight("python", source);
            assert_eq!(
                spans.iter().map(|s| s.text.as_str()).collect::<String>(),
                source
            );
            let colors: std::collections::BTreeSet<_> =
                spans.iter().map(|s| s.color.map(f32::to_bits)).collect();
            assert!(
                colors.len() >= 3,
                "Python was rendered without syntax colors"
            );
            fn text(e: &c::Element, out: &mut String) {
                match e {
                    c::Element::Text(t) => {
                        for r in &t.runs {
                            out.push_str(&r.value)
                        }
                    }
                    c::Element::Box(b) => {
                        for i in &b.children {
                            text(&i.element, out)
                        }
                    }
                    c::Element::Named { content, .. } => text(content, out),
                    _ => {}
                }
            }
            let element =
                Documents::default().code_element(source, &super::super::view::Metrics::default());
            let mut rendered = String::new();
            text(&element, &mut rendered);
            assert_eq!(rendered, source.strip_suffix('\n').unwrap());
            let parsed = bbt_doc::parse(&format!("```python\n{source}```\n"));
            let highlights = Highlights::prepare(&parsed, &RustHighlighter::new(false));
            let block = parsed
                .blocks
                .iter()
                .find(|b| matches!(b.kind, bbt_doc::BlockKind::Code { .. }))
                .unwrap();
            let ordinary = highlights.spans(block).unwrap();
            assert_eq!(
                ordinary,
                spans.as_slice(),
                "ordinary fenced Python must use the same highlighter"
            );
        })
        .await;
    }
}
#[cfg(test)]
mod code_element_tests {
    use super::*;
    fn has_text(e: &c::Element, want: &str) -> bool {
        match e {
            c::Element::Text(t) => t
                .runs
                .iter()
                .map(|r| r.value.as_str())
                .collect::<String>()
                .contains(want),
            c::Element::Named { content, .. } => has_text(content, want),
            c::Element::Box(b) => b.children.iter().any(|i| has_text(&i.element, want)),
            c::Element::Layers { base, .. } => has_text(base, want),
            c::Element::ScrollView(s) => has_text(&s.content, want),
            _ => false,
        }
    }
    // Live 2026-11-18: every Python source and shell command panicked the
    // pane's preparer ("shared Markdown code block") under the workspace
    // bbt-doc, whose framed code has no `block0` node. Each language returns
    // the literal code, whatever bbt-doc wraps it in.
    #[test]
    fn every_code_language_returns_its_literal_text() {
        let metrics = super::super::view::Metrics::default();
        for theme in [ui::Theme::Light, ui::Theme::Dark] {
            let mut docs = Documents::default();
            docs.set_theme(theme);
            for (language, source) in [
                (
                    "python",
                    "import asyncio\nprint(await tools.search('twig'))",
                ),
                ("sh", "cat .kb/AGENTS.md | head -40"),
                ("json", "{\"ok\": true}"),
                (
                    "python",
                    &"x = 1 # wide\n".repeat(3).replace('\n', &" ".repeat(300)),
                ),
            ] {
                let e = docs.code_element_language(source, language, &metrics);
                assert!(
                    has_text(
                        &e,
                        source.split_whitespace().next().unwrap().trim_matches('"')
                    ),
                    "{language} lost its code: {e:?}"
                );
            }
        }
    }
}

// keeps: specs/python-charts — stdout and emitted JSON remain literal.
#[cfg(test)]
#[test]
fn python_artifact_rows_ignore_stdout_and_nested_json() {
    use serde_json::json;
    let execution = "0000000000000000000000000B";
    let image = "![[artifact:0000000000000000000000000A:2]]";
    let record = json!({"execution_id":execution,"status":"completed",
        "output":image,"result":{"displayed":[image]}});
    assert!(python_displayed(execution, Some(&record), &[]).is_empty());
    let part = json!({"result":{"structuredContent":{
        "execution_id":execution,"displayed":[image,"ordinary text", "![bad]"]}}});
    assert_eq!(python_displayed(execution, Some(&part), &[]), vec![image]);
    assert!(python_displayed("other", Some(&part), &[]).is_empty());
}
