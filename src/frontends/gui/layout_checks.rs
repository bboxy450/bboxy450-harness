// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Structural smoke contracts over solved, restored geometry, before painting.
use super::view::{PreparedRows, context};
use bbt_ui::{
    self as ui,
    components::{Layout, LayoutKind},
};
use bbt_ui_model::layout::Rect;

#[derive(Debug, Default)]
pub struct Checked {
    pub markdown_blocks: usize,
    pub accordion_headers: usize,
    pub inline_rows: usize,
    pub painted_boxes: usize,
}
fn inside(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x - 0.5
        && inner.y >= outer.y - 0.5
        && inner.right() <= outer.right() + 0.5
        && inner.bottom() <= outer.bottom() + 0.5
}
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.right() - 0.5
        && b.x < a.right() - 0.5
        && a.y < b.bottom() - 0.5
        && b.y < a.bottom() - 0.5
}
fn content(n: &Layout) -> bool {
    match &n.kind {
        LayoutKind::Text { runs, .. } => runs.iter().any(|r| !r.text.trim().is_empty()),
        LayoutKind::Icon { name, .. } => !name.is_empty(),
        LayoutKind::Image { asset, .. } => !asset.is_empty(),
        LayoutKind::Path { points, .. } => !points.is_empty(),
        LayoutKind::Rects { rects, .. } => !rects.is_empty(),
        _ => n.children.iter().any(content),
    }
}
fn descendants<'a>(n: &'a Layout, out: &mut Vec<&'a Layout>) {
    out.push(n);
    for child in &n.children {
        descendants(child, out);
    }
}
fn row(n: &Layout) -> &Layout {
    if matches!(n.kind, LayoutKind::Named(_)) && n.children.len() == 1 {
        row(&n.children[0])
    } else {
        n
    }
}

pub fn check(prepared: &PreparedRows, frame: &Layout) -> Result<Checked, String> {
    fn walk(
        n: &Layout,
        card: Option<Rect>,
        path: &str,
        checked: &mut Checked,
    ) -> Result<(), String> {
        let mut card = card;
        let mut path = path;
        if let LayoutKind::Named(name) = &n.kind {
            // Transcript rows are checked in full before viewport elision.
            if name == "transcript" {
                return Ok(());
            }
            path = name;
            if name
                .strip_prefix('e')
                .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            {
                card = Some(n.rect);
            }
            if name.starts_with("mdblock/") {
                checked.markdown_blocks += 1;
                if !card.is_some_and(|card| inside(n.rect, card)) || !content(n) {
                    return Err(format!(
                        "MarkdownBlocksInsideCard: {name} {:?} outside {card:?} or unrestored",
                        n.rect
                    ));
                }
            }
            if name.starts_with("tool/") && name.ends_with("/toggle") {
                checked.accordion_headers += 1;
                let surface = ui::Paint::solid(ui::Color::Surface2).resolve(context())?;
                let mut nodes = vec![];
                descendants(n, &mut nodes);
                if !nodes.iter().any(|node| matches!(&node.kind, LayoutKind::Box {style, ..} if style.fill == surface && inside(n.rect, node.rect))) {
                    return Err(format!("AccordionHeaderFillCoversRow: {name} {:?}", n.rect));
                }
            }
            if name.starts_with("inline/") || name == "toolbar" {
                checked.inline_rows += 1;
                let row = row(n);
                if !card.is_none_or(|card| inside(row.rect, card)) {
                    return Err(format!("InlineControlsInsideRow: {name} row outside card"));
                }
                for (i, control) in row.children.iter().enumerate() {
                    let mut nodes = vec![];
                    descendants(control, &mut nodes);
                    let text_outside = nodes.iter().any(|node| matches!(&node.kind, LayoutKind::Text {runs, ..} if runs.iter().any(|r| !r.text.trim().is_empty() && !inside(r.rect, control.rect))));
                    if !inside(control.rect, row.rect)
                        || text_outside
                        || row.children[i + 1..]
                            .iter()
                            .any(|other| overlaps(control.rect, other.rect))
                    {
                        return Err(format!(
                            "InlineControlsInsideRow: {name} control {i} {:?} row {:?}; text outside={text_outside}",
                            control.rect, row.rect
                        ));
                    }
                }
            }
        }
        if let LayoutKind::Box { style, .. } = &n.kind {
            let drawn = style.fill[3] > 0.
                || style
                    .border
                    .as_ref()
                    .is_some_and(|b| b.width > 0. && b.color[3] > 0.)
                || style
                    .rings
                    .iter()
                    .chain(style.focus_ring.iter())
                    .any(|b| b.width > 0. && b.color[3] > 0.)
                || style.shadow.as_ref().is_some_and(|s| s.color[3] > 0.);
            if drawn && n.rect.w > 0. && n.rect.h > 0. {
                checked.painted_boxes += 1;
                if !content(n) {
                    return Err(format!("NoEmptyPaintedBoxes: {path} {:?}", n.rect));
                }
            }
        }
        for child in &n.children {
            walk(child, card, path, checked)?;
        }
        Ok(())
    }
    let mut checked = Checked::default();
    for layout in prepared.layouts() {
        walk(layout, None, "prepared", &mut checked)?;
    }
    walk(frame, None, "frame", &mut checked)?;
    if checked.markdown_blocks == 0
        || checked.accordion_headers == 0
        || checked.inline_rows == 0
        || checked.painted_boxes == 0
    {
        return Err(format!("structural smoke lacks witnesses: {checked:?}"));
    }
    Ok(checked)
}
