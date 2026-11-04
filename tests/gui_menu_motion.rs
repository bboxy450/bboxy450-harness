// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Menu motion on the animation clock (harness-spec.md Motion: menu open /
//! close, menu selection, hover). The clock is stepped by hand in fixed
//! increments; every assertion is a tween value read off the painted frame.
//! The clock, empty session, blocks, tail follow and composer have their own
//! witnesses in tests/gui_motion.rs.
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        anim::{Clock, Ease},
        bridge::{Bridge, FrameState, ProfileInfo},
        face,
        model::{Transcript, ViewItem},
        view::{Action, Frame, HarnessView, MenuKey, Preparer, context},
    },
    protocol::*,
};
use bbt_ui_model::{draw::Prim, layout::Rect};
use std::sync::Arc;

struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
const SIZE: [f32; 2] = [900., 700.];
/// A prepared, non-interactive row keeps loading/empty-state motion and message
/// source hover zones out of these menu-only clock and hit-map witnesses.
async fn state() -> FrameState {
    let profile = |name: &str, text: &str| ProfileInfo::parse(name, text);
    let id = ItemId::new();
    let mut transcript = Transcript {
        profile: Some(("haiku".into(), "claude-haiku-5-5".into())),
        ..Default::default()
    };
    transcript.order.push_back(id);
    transcript.items.insert(
        id,
        ViewItem {
            id,
            turn: None,
            parent: None,
            started: Seq(1),
            completed: Some(Seq(2)),
            kind: ItemKind::Notice,
            body: Item::Notice {
                level: Level::Info,
                text: "A settled row.".into(),
            },
            status: Some(ItemStatus::Completed),
            terminal_tail: String::new(),
            started_ms: 0,
            completed_ms: Some(1),
        },
    );
    let transcript = Arc::new(transcript);
    let source = transcript.clone();
    let prepared = bboxy450_harness::runtime::blocking(move || {
        Preparer::default().prepare(source, SIZE[0]).unwrap()
    })
    .await;
    FrameState {
        transcript,
        prepared: Some(Arc::new(prepared)),
        ready: true,
        profiles: Arc::new(vec![
            profile(
                "codex-acp",
                "---\nname: codex-acp\nprovider: acp-agent\nagent:\n  command: [codex-acp]\n---\n",
            ),
            profile(
                "haiku",
                "---\nname: haiku\nprovider: anthropic\nmodel: claude-haiku-5-5\n---\n",
            ),
            profile(
                "opus",
                "---\nname: opus\nprovider: anthropic\nmodel: claude-opus-5-5\n---\n",
            ),
        ]),
        ..Default::default()
    }
}
fn live() -> HarnessView {
    let mut view = HarnessView::default();
    view.anim = Clock::live(false);
    view
}
/// Step the clock to `ms` and build the frame.
fn at<'v>(view: &'v mut HarnessView, state: &FrameState, ms: f32) -> &'v Frame {
    view.anim.tick(ms);
    view.render(state, SIZE).unwrap()
}
/// The menu's primitives over the base frame (`Frame::over`, painted after the face and
/// its rims): the fading group's opacity (None when painted plain, at full opacity) and
/// the card's primitives.
fn overlay(frame: &Frame) -> (Option<f32>, Vec<Prim>) {
    let over = &frame.over.primitives[..];
    match over {
        [
            Prim::Group {
                primitives,
                opacity,
                ..
            },
        ] => (Some(*opacity), primitives.clone()),
        _ => (None, over.to_vec()),
    }
}
fn rects<'a>(prims: &'a [Prim]) -> impl Iterator<Item = (Rect, [f32; 4])> + 'a {
    prims.iter().filter_map(|p| match p {
        Prim::Rect { rect, fill, .. } => Some((*rect, fill.0)),
        _ => None,
    })
}
fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}
fn same_rect(a: Rect, b: Rect) -> bool {
    close(a.x, b.x) && close(a.y, b.y) && close(a.w, b.w) && close(a.h, b.h)
}
fn same_color(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}
/// The card's outer ring: the widest rect the menu paints. `left` is where it
/// sits when settled (the spec's left 12 / left 10); mid-flight the scale
/// about the centre moves it right, so only `settled` rings are checked.
fn ring(prims: &[Prim], left: f32) -> Rect {
    let r = rects(prims)
        .map(|(r, _)| r)
        .filter(|r| r.w > 300.)
        .max_by(|a, b| a.w.total_cmp(&b.w))
        .expect("menu ring");
    assert!(r.x >= left - 1., "{r:?} left of {left}");
    r
}
fn settled(prims: &[Prim], left: f32) -> Rect {
    let r = ring(prims, left);
    assert!(r.x <= left + 1.5, "{r:?} is not at rest at left {left}");
    r
}
/// The rect painted with exactly `fill`.
fn filled(prims: &[Prim], fill: [f32; 4]) -> Option<Rect> {
    rects(prims)
        .find(|(_, c)| same_color(*c, fill))
        .map(|(r, _)| r)
}
/// The fill painted on exactly `rect`, if any.
fn fill_of(prims: &[Prim], rect: Rect) -> Option<[f32; 4]> {
    rects(prims)
        .find(|(r, _)| same_rect(*r, rect))
        .map(|(_, c)| c)
}
fn row(frame: &Frame, key: &str) -> Rect {
    frame
        .hover
        .iter()
        .find(|(_, k)| k == key)
        .map(|(r, _)| *r)
        .unwrap_or_else(|| panic!("row {key} has a hover zone"))
}
/// CSS-style scale about the card's centre plus the drop.
fn transformed(end: Rect, centre: [f32; 2], s: f32, dy: f32) -> Rect {
    Rect::new(
        centre[0] + (end.x - centre[0]) * s,
        centre[1] + (end.y - centre[1]) * s + dy,
        end.w * s,
        end.h * s,
    )
}

#[tokio::test]
async fn slash_menu_opens_with_a_drop_scale_and_fade_then_sits_idle() {
    let state = state().await;
    let mut view = live();
    at(&mut view, &state, 0.);
    assert_eq!(view.anim.next_frame(), None, "no menu: nothing to animate");
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    // the settled geometry, measured once from a settled clock on the same input
    let mut flat = HarnessView::default();
    flat.render(&state, SIZE).unwrap();
    flat.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    let (group, end) = overlay(flat.render(&state, SIZE).unwrap());
    assert_eq!(group, None, "a settled clock paints the card plain");
    let end_ring = settled(&end, 12.);
    let centre = [end_ring.x + end_ring.w / 2., end_ring.y + end_ring.h / 2.];
    // t0: invisible, 6px up, at .98
    let (group, prims) = overlay(at(&mut view, &state, 1000.));
    assert_eq!(group, Some(0.));
    assert!(same_rect(
        ring(&prims, 12.),
        transformed(end_ring, centre, 0.98, -6.)
    ));
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the open tween wants frames"
    );
    // half way: the `out` curve's value at .5 drives opacity, drop and scale together
    let p = Ease::Out.at(0.5);
    let (group, prims) = overlay(at(&mut view, &state, 1085.));
    assert!(close(group.expect("fading in"), p), "{group:?} vs {p}");
    let want = transformed(end_ring, centre, 0.98 + 0.02 * p, -6. * (1. - p));
    let got = ring(&prims, 12.);
    assert!(same_rect(got, want), "{got:?} vs {want:?}");
    assert!(got.y > end_ring.y - 6. && got.y < end_ring.y);
    assert!(got.w > end_ring.w * 0.98 && got.w < end_ring.w);
    // the end: plain primitives at the settled geometry
    let (group, prims) = overlay(at(&mut view, &state, 1170.));
    assert_eq!(group, None);
    assert!(same_rect(ring(&prims, 12.), end_ring));
    // the composer's own send-button fade (200ms `std` from the "/" edit) ends at
    // 1200; from then on nothing moves and the clock goes idle
    at(&mut view, &state, 1200.);
    assert_eq!(view.anim.next_frame(), None, "settled: no frame requested");
    let requested = view.anim.frames_requested;
    for ms in [1300., 1500., 2000., 5000., 60_000.] {
        let (group, prims) = overlay(at(&mut view, &state, ms));
        assert_eq!(group, None);
        assert!(same_rect(ring(&prims, 12.), end_ring));
        assert_eq!(view.anim.next_frame(), None, "idle at {ms}ms");
    }
    assert_eq!(
        view.anim.frames_requested, requested,
        "an open, settled menu asks for zero redraws"
    );
}

#[tokio::test]
async fn slash_selection_slides_between_rows_and_the_tile_tint_crossfades() {
    let state = state().await;
    let ink = face::ink(context());
    let sel_fill = face::alpha(ink.seat, 0.14);
    let tile_on = face::alpha(ink.seat, 0.16);
    let tile_off = face::alpha(ink.fg, 0.07);
    let mut view = live();
    at(&mut view, &state, 0.);
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    let frame = at(&mut view, &state, 1000.);
    let (r0, r1) = (row(frame, "completion/0"), row(frame, "completion/1"));
    assert!(r1.y > r0.y);
    // settled on row 0 after the open tween: the highlight sits on it
    let (_, prims) = overlay(at(&mut view, &state, 1170.));
    assert!(same_rect(filled(&prims, sel_fill).expect("highlight"), r0));
    let tile = |prims: &[Prim], r: Rect| {
        rects(prims)
            .find(|(t, _)| {
                close(t.w, 18.) && close(t.h, 18.) && t.y > r.y && t.bottom() < r.bottom()
            })
            .map(|(_, c)| c)
            .expect("the row's 18px tile")
    };
    assert!(same_color(tile(&prims, r0), tile_on));
    assert!(same_color(tile(&prims, r1), tile_off));
    // ArrowDown at 1200: the highlight leaves row 0 for row 1 over 120ms `std`
    assert!(view.complete(bbt_ui::modules::approval::Key::Down));
    let (_, prims) = overlay(at(&mut view, &state, 1200.));
    assert!(
        same_rect(filled(&prims, sel_fill).expect("highlight"), r0),
        "the move starts from the current row, it never jumps"
    );
    assert_eq!(view.anim.next_frame(), Some(0.));
    let q = Ease::Std.at(0.5);
    let (_, prims) = overlay(at(&mut view, &state, 1260.));
    let mid = filled(&prims, sel_fill).expect("highlight");
    assert!(
        close(mid.y, r0.y + (r1.y - r0.y) * q),
        "{} vs {}",
        mid.y,
        r0.y + (r1.y - r0.y) * q
    );
    assert!(close(mid.h, r0.h));
    // the 2px seat bar rides with it, inset 8px top and bottom
    let bar = filled(&prims, ink.seat).expect("seat bar");
    assert!(close(bar.x, mid.x - 3.) && close(bar.y, mid.y + 8.) && close(bar.w, 2.));
    // the tiles crossfade with the same weight
    let t1 = tile(&prims, r1);
    for i in 0..4 {
        let want = tile_on[i] * q + tile_off[i] * (1. - q);
        assert!(
            (t1[i] - want).abs() < 1e-3,
            "tile channel {i}: {} vs {want}",
            t1[i]
        );
    }
    let (_, prims) = overlay(at(&mut view, &state, 1320.));
    assert!(same_rect(filled(&prims, sel_fill).expect("highlight"), r1));
    assert!(same_color(tile(&prims, r1), tile_on));
    assert!(same_color(tile(&prims, r0), tile_off));
    assert_eq!(view.anim.next_frame(), None);
    // a retarget mid-flight: Down again at 1340, then Up at 1400 turns back from
    // wherever the highlight is, never from `from`
    let frame = at(&mut view, &state, 1330.);
    let r2 = row(frame, "completion/2");
    assert!(view.complete(bbt_ui::modules::approval::Key::Down));
    at(&mut view, &state, 1340.);
    let (_, prims) = overlay(at(&mut view, &state, 1400.));
    let here = filled(&prims, sel_fill).expect("highlight").y;
    assert!(here > r1.y && here < r2.y);
    assert!(view.complete(bbt_ui::modules::approval::Key::Up));
    let (_, prims) = overlay(at(&mut view, &state, 1400.));
    assert!(close(filled(&prims, sel_fill).expect("highlight").y, here));
    let (_, prims) = overlay(at(&mut view, &state, 1430.));
    let back = filled(&prims, sel_fill).expect("highlight").y;
    assert!(
        back < here && back > r1.y,
        "{back} between {} and {here}",
        r1.y
    );
    let (_, prims) = overlay(at(&mut view, &state, 1600.));
    assert!(same_rect(filled(&prims, sel_fill).expect("highlight"), r1));
}

#[tokio::test]
async fn hover_fills_fade_in_and_out_over_150ms_from_the_pointer() {
    let state = state().await;
    let ink = face::ink(context());
    let mut view = live();
    at(&mut view, &state, 0.);
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    at(&mut view, &state, 1000.);
    // 1200: the menu (170ms) and the composer's send fade (200ms) are both done
    let frame = at(&mut view, &state, 1200.);
    let r2 = row(frame, "completion/2");
    assert_eq!(view.anim.next_frame(), None);
    let fill = |prims: &[Prim]| fill_of(prims, r2).map_or(0., |c| c[3]);
    // not hovered: no fill on the row
    let (_, prims) = overlay(at(&mut view, &state, 1200.));
    assert_eq!(fill(&prims), 0.);
    // the pointer enters row 2: card-hi (fg 7%) fades in over 150ms `std`
    assert!(view.pointer([r2.x + 20., r2.y + r2.h / 2.]));
    assert_eq!(view.hovered(), Some("completion/2"));
    assert!(
        !view.pointer([r2.x + 30., r2.y + r2.h / 2.]),
        "same row: no change"
    );
    let (_, prims) = overlay(at(&mut view, &state, 1200.));
    assert_eq!(fill(&prims), 0., "the fade starts at zero");
    assert_eq!(view.anim.next_frame(), Some(0.));
    let q = Ease::Std.at(0.5);
    let (_, prims) = overlay(at(&mut view, &state, 1275.));
    assert!(
        close(fill(&prims), 0.07 * q),
        "{} vs {}",
        fill(&prims),
        0.07 * q
    );
    let (_, prims) = overlay(at(&mut view, &state, 1350.));
    assert!(close(fill(&prims), 0.07));
    let c = fill_of(&prims, r2).unwrap();
    assert!(same_color(c, face::alpha(ink.fg, 0.07)));
    assert_eq!(view.anim.next_frame(), None);
    // the pointer leaves: the fill fades back out
    assert!(view.pointer([0., 0.]));
    assert_eq!(view.hovered(), None);
    at(&mut view, &state, 1400.);
    let (_, prims) = overlay(at(&mut view, &state, 1475.));
    assert!(close(fill(&prims), 0.07 * (1. - q)));
    let (_, prims) = overlay(at(&mut view, &state, 1550.));
    assert_eq!(fill(&prims), 0.);
    assert_eq!(view.anim.next_frame(), None);
}

#[tokio::test]
async fn a_closed_menu_fades_out_over_120ms_then_the_clock_is_idle() {
    let state = state().await;
    let bridge = Bridge::new(Arc::new(Wake));
    let mut view = live();
    at(&mut view, &state, 0.);
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    at(&mut view, &state, 1000.);
    let (_, end) = overlay(at(&mut view, &state, 1170.));
    let end_ring = settled(&end, 12.);
    assert!(view.cancel_prompt(&bridge));
    assert!(!view.completion_open());
    // the card is gone from the hit map but still on screen, fading
    let frame = at(&mut view, &state, 2000.);
    assert!(frame.hover.is_empty());
    assert!(
        !frame
            .actions
            .iter()
            .any(|(_, a)| matches!(a, Action::Complete(..)))
    );
    let (group, prims) = overlay(frame);
    assert_eq!(group, None, "starts from full opacity");
    assert!(
        same_rect(ring(&prims, 12.), end_ring),
        "no movement on close"
    );
    assert_eq!(view.anim.next_frame(), Some(0.));
    let q = Ease::Std.at(0.5);
    let (group, prims) = overlay(at(&mut view, &state, 2060.));
    assert!(close(group.expect("fading out"), 1. - q));
    assert!(same_rect(ring(&prims, 12.), end_ring));
    let (group, prims) = overlay(at(&mut view, &state, 2120.));
    assert_eq!((group, prims.len()), (None, 0), "gone");
    assert_eq!(view.anim.next_frame(), None);
    let requested = view.anim.frames_requested;
    for ms in [2200., 3000., 10_000.] {
        let (_, prims) = overlay(at(&mut view, &state, ms));
        assert!(prims.is_empty());
        assert_eq!(view.anim.next_frame(), None);
    }
    assert_eq!(view.anim.frames_requested, requested);
}

#[tokio::test]
async fn profile_switcher_drops_in_slides_its_selection_and_fades_out() {
    let state = state().await;
    let bridge = Bridge::new(Arc::new(Wake));
    let ink = face::ink(context());
    let sel_fill = face::alpha(ink.seat, 0.12);
    let mut view = live();
    at(&mut view, &state, 0.);
    view.toggle_profile_menu();
    let mut flat = HarnessView::default();
    flat.render(&state, SIZE).unwrap();
    flat.toggle_profile_menu();
    let (_, end) = overlay(flat.render(&state, SIZE).unwrap());
    let end_ring = settled(&end, 10.);
    assert!(
        (40. ..=41.).contains(&end_ring.y),
        "top 40, left 10: {end_ring:?}"
    );
    let centre = [end_ring.x + end_ring.w / 2., end_ring.y + end_ring.h / 2.];
    let (group, prims) = overlay(at(&mut view, &state, 500.));
    assert_eq!(group, Some(0.));
    assert!(same_rect(
        ring(&prims, 10.),
        transformed(end_ring, centre, 0.98, -6.)
    ));
    let p = Ease::Out.at(0.5);
    let (group, prims) = overlay(at(&mut view, &state, 585.));
    assert!(close(group.unwrap(), p));
    assert!(same_rect(
        ring(&prims, 10.),
        transformed(end_ring, centre, 0.98 + 0.02 * p, -6. * (1. - p))
    ));
    let frame = at(&mut view, &state, 670.);
    let (haiku, opus) = (row(frame, "pmenu/row/haiku"), row(frame, "pmenu/row/opus"));
    let (group, prims) = overlay(frame);
    assert_eq!(group, None);
    assert!(
        same_rect(filled(&prims, sel_fill).expect("highlight"), haiku),
        "the switcher opens on the current profile"
    );
    assert_eq!(view.anim.next_frame(), None);
    // Down: the seat-12 highlight slides to opus over 120ms `std`
    view.profile_menu_key(MenuKey::Down, &bridge);
    at(&mut view, &state, 700.);
    let q = Ease::Std.at(0.5);
    let (_, prims) = overlay(at(&mut view, &state, 760.));
    let mid = filled(&prims, sel_fill).expect("highlight");
    assert!(close(mid.y, haiku.y + (opus.y - haiku.y) * q));
    let (_, prims) = overlay(at(&mut view, &state, 820.));
    assert!(same_rect(
        filled(&prims, sel_fill).expect("highlight"),
        opus
    ));
    // a hover on haiku fades seat-12 in over the current row's seat-9
    assert!(view.pointer([haiku.x + 10., haiku.y + 5.]));
    let (_, prims) = overlay(at(&mut view, &state, 900.));
    assert!(same_color(
        fill_of(&prims, haiku).unwrap(),
        face::alpha(ink.seat, 0.09)
    ));
    let (_, prims) = overlay(at(&mut view, &state, 975.));
    let a = fill_of(&prims, haiku).unwrap()[3];
    assert!(close(a, 0.09 + 0.03 * q), "{a}");
    let (_, prims) = overlay(at(&mut view, &state, 1050.));
    assert!(same_color(
        fill_of(&prims, haiku).unwrap(),
        face::alpha(ink.seat, 0.12)
    ));
    // a click outside closes it: a 120ms fade, no actions, then idle
    view.activate(Action::Menu(false), &bridge);
    assert!(!view.profile_menu_open());
    let frame = at(&mut view, &state, 1100.);
    assert!(
        !frame
            .actions
            .iter()
            .any(|(_, a)| matches!(a, Action::Menu(_)))
    );
    let (group, prims) = overlay(at(&mut view, &state, 1160.));
    assert!(close(group.expect("fading"), 1. - q));
    assert!(same_rect(ring(&prims, 10.), end_ring));
    let (group, prims) = overlay(at(&mut view, &state, 1220.));
    assert_eq!((group, prims.len()), (None, 0));
    assert_eq!(view.anim.next_frame(), None);
}

#[tokio::test]
async fn settled_and_reduced_motion_clocks_paint_end_states_without_frames() {
    let state = state().await;
    for mut view in [HarnessView::default(), {
        let mut v = HarnessView::default();
        v.anim = Clock::live(true);
        v
    }] {
        at(&mut view, &state, 0.);
        view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
        let frame = at(&mut view, &state, 1.);
        let (r1, r2) = (row(frame, "completion/1"), row(frame, "completion/2"));
        let (group, prims) = overlay(frame);
        assert_eq!(group, None);
        settled(&prims, 12.);
        assert_eq!(view.anim.next_frame(), None);
        // Down selects row 1 and the pointer rests on row 2: both at once, no frames
        view.complete(bbt_ui::modules::approval::Key::Down);
        view.pointer([r2.x + 5., r2.y + 5.]);
        let (group, prims) = overlay(at(&mut view, &state, 2.));
        assert_eq!(group, None);
        let ink = face::ink(context());
        assert!(
            same_rect(filled(&prims, face::alpha(ink.seat, 0.14)).unwrap(), r1),
            "jumped"
        );
        assert!(
            same_color(fill_of(&prims, r2).unwrap(), face::alpha(ink.fg, 0.07)),
            "end state"
        );
        assert_eq!(view.anim.next_frame(), None);
        assert_eq!(view.anim.frames_requested, 0);
    }
}

/// While a menu tweens the clock is in motion, so every frame rebuilds (the
/// view's rule for any motion); this prints what a menu frame costs.
#[tokio::test]
async fn a_menu_animation_frame_is_built_every_frame_while_it_moves() {
    let state = state().await;
    let mut view = live();
    at(&mut view, &state, 0.);
    view.edit(bbt_ui::text_edit::Command::Insert("/".into()));
    let frame = at(&mut view, &state, 1000.);
    assert!(!frame.list.primitives.is_empty() && !frame.over.primitives.is_empty());
    let mut cost = vec![];
    for i in 1..=10 {
        assert_eq!(view.anim.next_frame(), Some(0.), "opening: a frame follows");
        let t = std::time::Instant::now();
        let frame = at(&mut view, &state, 1000. + 16. * i as f32);
        cost.push(t.elapsed().as_secs_f64() * 1000.);
        assert!(!frame.over.primitives.is_empty(), "the card is painted");
    }
    // 1200: the card (170ms) and the composer's send fade (200ms) are both done
    at(&mut view, &state, 1200.);
    assert_eq!(view.anim.next_frame(), None, "open: idle");
    cost.sort_by(|a, b| a.total_cmp(b));
    println!(
        "menu animation frame (debug build, full rebuild): median {:.3}ms, max {:.3}ms",
        cost[cost.len() / 2],
        cost[cost.len() - 1]
    );
}
