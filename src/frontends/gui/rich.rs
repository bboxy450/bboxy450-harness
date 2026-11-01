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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Face {
    pub reasoning_closed: bool,
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
            Toggle::Reasoning => self.reasoning_closed = !self.reasoning_closed,
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
    blocks: Vec<(Range<usize>, Arc<c::Layout>)>,
    width: f32,
    received: usize,
    parsed: usize,
    // A credit bounds reparsing even for one unfinished paragraph/fence or
    // document-wide reference definitions. Unparsed bytes remain visible as
    // literal tail text; completion always flushes through streaming append.
}
#[derive(Default)]
pub struct Documents {
    docs: HashMap<ItemId, Document>,
    highlighter: Option<RustHighlighter>,
    pub parsed_bytes: usize,
    pub received_bytes: usize,
    pub block_layouts: usize,
}
pub struct Body {
    pub element: c::Element,
    pub blocks: Vec<Arc<c::Layout>>,
}
impl Documents {
    pub fn prepare(
        &mut self,
        id: ItemId,
        text: &str,
        complete: bool,
        width: f32,
        metrics: &impl c::Measure,
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
        if parse {
            blocking_hook(BlockingWork::MarkdownPrepare);
            doc.stream.append(&text[prior..]);
            let range = doc.stream.reparsed_range();
            self.parsed_bytes += range.len();
            doc.parsed += range.len();
            changed = Some(range.start);
        }
        if doc.width != width {
            doc.blocks.clear();
            doc.width = width;
        }
        if let Some(start) = changed {
            doc.blocks.retain(|(range, _)| range.end <= start);
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
                fragment.highlighted_body_element(&highlights, context(), metrics, width);
            super::view::flat(&mut element);
            let layout = element.layout(Rect::new(0., 0., width, 0.), context(), metrics)?;
            doc.blocks.push((block.source.clone(), Arc::new(layout)));
            self.block_layouts += 1;
        }
        let mut blocks: Vec<_> = doc.blocks.iter().map(|(_, l)| l.clone()).collect();
        let pending = &text[doc.stream.document().source.len()..];
        if !pending.is_empty() {
            // Literal streaming tail is fully readable while a pathological
            // unfinished block waits for enough parse credit. No bytes vanish.
            let mut text = c::Text::new(pending, ui::Text::default());
            text.preserve_whitespace = true;
            blocks.push(Arc::new(c::Element::Text(text).layout(
                Rect::new(0., 0., width, 0.),
                context(),
                metrics,
            )?));
        }
        let slots = blocks
            .iter()
            .enumerate()
            .map(|(i, l)| named(format!("mdblock/{i}"), column(vec![], Some(l.rect.h))).into())
            .collect();
        Ok(Body {
            element: column(slots, None),
            blocks,
        })
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
        *node = solved;
        return;
    }
    for child in &mut node.children {
        restore(child, blocks);
    }
}
