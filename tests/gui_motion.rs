// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Motion witnesses (harness-spec.md "Motion"): the window clock is stepped in fixed
//! increments and tween values are asserted at sample times through the solved layout.
//! No window, no wall clock: `Clock::live(false)` ticked by hand.
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        anim::{Clock, Ease},
        bridge::FrameState,
        model::{Transcript, ViewItem},
        view::{HarnessView, PreparedRows, Preparer, find, layout_text},
    },
    protocol::*,
};
use bbt_ui::components::{Layout, LayoutKind};
use std::sync::Arc;

const SIZE: [f32; 2] = [900., 700.];

async fn prepare(preparer: &mut Preparer, source: Arc<Transcript>, width: f32) -> PreparedRows {
    let mut owned = std::mem::take(preparer);
    let (next, result) = bboxy450_harness::runtime::blocking(move || {
        let result = owned.prepare(source, width);
        (owned, result)
    })
    .await;
    *preparer = next;
    result.unwrap()
}
fn message(id: ItemId, markdown: &str, completed: bool) -> ViewItem {
    ViewItem {
        id,
        turn: None,
        parent: None,
        started: Seq(1),
        completed: completed.then_some(Seq(2)),
        kind: ItemKind::AgentMessage,
        body: Item::AgentMessage {
            markdown: markdown.into(),
        },
        status: completed.then_some(ItemStatus::Completed),
        terminal_tail: String::new(),
        started_ms: 0,
        completed_ms: completed.then_some(1),
    }
}
fn transcript(rows: &[ViewItem]) -> Transcript {
    let mut t = Transcript {
        session: Some(SessionId::new()),
        profile: Some(("nemotron".into(), "nemotron-3-nano".into())),
        ..Default::default()
    };
    for row in rows {
        t.order.push_back(row.id);
        t.items.insert(row.id, row.clone());
    }
    t
}
async fn state(preparer: &mut Preparer, rows: &[ViewItem]) -> FrameState {
    let mut s = FrameState {
        transcript: Arc::new(transcript(rows)),
        ready: true,
        replay: true,
        ..Default::default()
    };
    s.prepared = Some(Arc::new(
        prepare(preparer, s.transcript.clone(), SIZE[0]).await,
    ));
    s
}
fn live_view() -> HarnessView {
    let mut view = HarnessView::default();
    view.anim = Clock::live(false);
    view
}
/// Render at clock time `ms` and return the solved layout.
fn at(view: &mut HarnessView, state: &FrameState, ms: f32) -> Layout {
    at_size(view, state, ms, SIZE)
}
fn at_size(view: &mut HarnessView, state: &FrameState, ms: f32, size: [f32; 2]) -> Layout {
    view.anim.tick(ms);
    view.render(state, size).unwrap().layout.clone().unwrap()
}
fn filter_opacity(node: &Layout) -> Option<f32> {
    match &node.kind {
        LayoutKind::Filter { opacity, .. } => Some(*opacity),
        _ => node.children.iter().find_map(filter_opacity),
    }
}
/// The opacity of the entrance filter wrapping the keyed node (the filter is an ancestor of
/// `item/{id}`, so a descendant search misses it). None when the node is at rest.
fn entrance_opacity(root: &Layout, key: &str) -> Option<f32> {
    if let LayoutKind::Filter { opacity, .. } = root.kind
        && find(root, key).is_some()
    {
        return Some(opacity);
    }
    root.children.iter().find_map(|c| entrance_opacity(c, key))
}
fn box_style(node: &Layout) -> Option<&bbt_ui::ResolvedStyle> {
    match &node.kind {
        LayoutKind::Box { style, .. } => Some(style),
        _ => node.children.iter().find_map(box_style),
    }
}
fn near(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

/// Empty session: the column rises 6 px and fades over 420 ms `out`, the chips follow 60 ms
/// apart, and the halo breathes over 3.2 s between opacity .25 and .8. The loop keeps the
/// clock asking for frames; a settled clock draws the end state and asks for none.
#[test]
fn empty_session_rises_in_and_the_halo_breathes() {
    let state = FrameState {
        transcript: Arc::new(transcript(&[])),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut view = live_view();
    let l0 = at(&mut view, &state, 0.);
    let empty0 = find(&l0, "empty").expect("the empty session");
    assert!(
        layout_text(empty0).contains("What are we building?")
            && layout_text(empty0).contains("nemotron-3-nano"),
        "{}",
        layout_text(empty0)
    );
    // the column's entrance is keyed on the title (a stable non-chip element inside it, so
    // its wrapping filter is the column's own and never a chip's)
    assert_eq!(
        entrance_opacity(&l0, "empty/title"),
        Some(0.),
        "transparent at t=0"
    );
    let y0 = empty0.rect.y;
    let chip2_y0 = find(&l0, "empty/chip/2").unwrap().rect.y;
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the entrance asks for a frame"
    );

    let l1 = at(&mut view, &state, 210.);
    let empty1 = find(&l1, "empty").unwrap();
    let half = Ease::Out.at(0.5);
    assert!(
        near(entrance_opacity(&l1, "empty/title").unwrap(), half, 0.01),
        "opacity at 210 ms is out(.5) = {half}: {:?}",
        entrance_opacity(&l1, "empty/title")
    );
    assert!(
        near(empty1.rect.y, y0 - 6. * half, 0.5),
        "risen 6·out(.5) px: {} -> {}",
        y0,
        empty1.rect.y
    );
    // chip 2 waits 180 ms, so at 210 ms its own 420 ms rise has run only 30 ms (the chips
    // compound their stagger over the column's fade, so read the chip tween from the clock)
    let chip2_t = Ease::Out.at(30. / 420.);
    assert!(
        near(view.anim.value("empty/chip/2").unwrap(), chip2_t, 0.02),
        "chip 2 stagger at 210 ms: {:?} vs {chip2_t}",
        view.anim.value("empty/chip/2")
    );
    assert!(
        view.anim.value("empty/chip/0").unwrap() > view.anim.value("empty/chip/2").unwrap(),
        "chip 0 leads chip 2"
    );

    let l2 = at(&mut view, &state, 420.);
    let empty2 = find(&l2, "empty").unwrap();
    assert_eq!(
        entrance_opacity(&l2, "empty/title"),
        None,
        "the column is in place: no wrapping filter on the title"
    );
    assert!(near(empty2.rect.y, y0 - 6., 0.5));
    assert!(
        view.anim.value("empty/chip/2").unwrap() < 1.,
        "chip 2 is still rising at 420 ms"
    );
    let l3 = at(&mut view, &state, 600.);
    let chip2_rest = find(&l3, "empty/chip/2").unwrap().rect.y;
    assert_eq!(
        view.anim.value("empty/chip/2"),
        Some(1.),
        "chip 2 is in by 600 ms"
    );
    assert!(
        chip2_y0 > chip2_rest + 5.,
        "chip 2 rose into place: {chip2_y0} -> {chip2_rest}"
    );

    // the halo: opacity .25 at phase 0, .8 at phase .5 (1600 ms), scale .96 -> 1.04
    let halo = |l: &Layout| match &find(l, "empty/halo").unwrap().children[0].kind {
        LayoutKind::Path { color, points, .. } => {
            let bbt_ui_model::draw::PathCommand::Move(p) = points[0] else {
                panic!("circle starts with a move")
            };
            (color[3], p[0] - 32.)
        }
        other => panic!("halo is a path: {other:?}"),
    };
    let (a0, r0) = halo(&l0);
    assert!(
        near(a0, 0.45 * 0.25, 0.005) && near(r0, 31.5 * 0.96, 0.05),
        "{a0} {r0}"
    );
    let l4 = at(&mut view, &state, 1600.);
    let (a4, r4) = halo(&l4);
    assert!(
        near(a4, 0.45 * 0.8, 0.005) && near(r4, 31.5 * 1.04, 0.05),
        "{a4} {r4}"
    );
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the halo loop keeps the clock awake"
    );
    let frames = view.anim.frames_requested;
    at(&mut view, &state, 1616.);
    assert_eq!(view.anim.frames_requested, frames + 1);

    // the settled clock: end state, static ring at .5, no frame
    let mut settled = HarnessView::default();
    let ls = at(&mut settled, &state, 0.);
    let es = find(&ls, "empty").unwrap();
    assert_eq!(filter_opacity(es), None);
    assert!(
        near(es.rect.y, y0 - 6., 0.5),
        "settled sits where the entrance ends"
    );
    let (a, r) = halo(&ls);
    assert!(near(a, 0.45 * 0.5, 0.005) && near(r, 31.5, 0.05));
    assert_eq!(settled.anim.next_frame(), None);
    assert_eq!(settled.anim.frames_requested, 0);
    // the chips act
    let chips = settled
        .frame
        .actions
        .iter()
        .filter(|(_, a)| matches!(a, bboxy450_harness::frontends::gui::view::Action::Chip(_)))
        .count();
    assert_eq!(chips, 3);
}

/// Block enters: a row new to the session sits 6 px low and transparent, then rises and
/// fades in over 320 ms `out`. A session's first frame enters nothing.
#[tokio::test]
async fn blocks_enter_with_a_rise_and_fade() {
    let mut preparer = Preparer::default();
    let first = ItemId::new();
    let s0 = state(&mut preparer, &[message(first, "First reply.", true)]).await;
    let mut view = live_view();
    let l0 = at(&mut view, &s0, 0.);
    let row0 = find(&l0, &format!("item/{first}")).unwrap();
    assert_eq!(
        entrance_opacity(&l0, &format!("item/{first}")),
        None,
        "the first frame of a session enters nothing"
    );
    let rest_y = row0.rect.y;

    let second = ItemId::new();
    let mut s1 = s0.clone();
    let mut t = (*s0.transcript).clone();
    let row = message(second, "Second reply.", true);
    t.order.push_back(second);
    t.items.insert(second, row);
    s1.transcript = Arc::new(t);
    s1.prepared = Some(Arc::new(
        prepare(&mut preparer, s1.transcript.clone(), SIZE[0]).await,
    ));
    let _ = rest_y;
    let key = format!("item/{second}");
    // a tall viewport: the two short rows never fill it, so there is no tail-follow scroll
    let tall = [900., 3000.];
    let l1 = at_size(&mut view, &s1, 100., tall);
    assert_eq!(
        entrance_opacity(&l1, &format!("item/{first}")),
        None,
        "a known row does not re-enter"
    );
    // the entrance fades the row in over 320 ms `out`: its opacity, asserted at sample times,
    // matches the curve exactly (the solved filter and the clock tween agree). The 6 px rise
    // runs underneath and is checked by direction below, since the fast ease-out all but
    // finishes it within the first frames while the row above still reflows.
    let low_y = find(&l1, &key).unwrap().rect.y;
    assert_eq!(entrance_opacity(&l1, &key), Some(0.), "transparent at t=0");
    assert_eq!(view.anim.value(&format!("in/{second}")), Some(0.));
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the entrance asks for a frame"
    );

    for (ms, progress) in [(160., 60. / 320.), (260., 160. / 320.), (380., 280. / 320.)] {
        let l = at_size(&mut view, &s1, ms, tall);
        let want = Ease::Out.at(progress);
        assert!(
            near(entrance_opacity(&l, &key).unwrap(), want, 0.01),
            "opacity at {ms} ms is out({progress}) = {want}: {:?}",
            entrance_opacity(&l, &key)
        );
        assert!(
            near(
                view.anim.value(&format!("in/{second}")).unwrap(),
                want,
                0.01
            ),
            "the clock tween agrees with the solved opacity"
        );
    }

    let l3 = at_size(&mut view, &s1, 420., tall);
    assert_eq!(
        entrance_opacity(&l3, &key),
        None,
        "fully in after 320 ms: no filter"
    );
    let settled_y = find(&l3, &key).unwrap().rect.y;
    assert!(
        low_y > settled_y + 4.,
        "the row rose into place: entered {low_y} px down, settled at {settled_y}"
    );
    assert_eq!(view.anim.next_frame(), None, "settled: no frame");
}

/// Block grows: a streaming card's slot height tweens over 200 ms `out` as content arrives,
/// the rows below slide with it, the content height follows, and the growing row is clipped
/// to its slot so it never paints over the next card.
#[tokio::test]
async fn card_heights_tween_and_rows_below_slide() {
    let mut preparer = Preparer::default();
    let a = ItemId::new();
    let b = ItemId::new();
    let s0 = state(
        &mut preparer,
        &[message(a, "One line.", false), message(b, "Below.", true)],
    )
    .await;
    let mut view = live_view();
    let l0 = at(&mut view, &s0, 0.);
    let h0 = find(&l0, &format!("item/{a}")).unwrap().rect.h;
    let below0 = find(&l0, &format!("item/{b}")).unwrap().rect.y;

    let mut s1 = s0.clone();
    let mut t = (*s0.transcript).clone();
    t.items.insert(
        a,
        message(
            a,
            "One line.\n\nA second paragraph arrives.\n\nAnd a third one, too.",
            false,
        ),
    );
    s1.transcript = Arc::new(t);
    s1.prepared = Some(Arc::new(
        prepare(&mut preparer, s1.transcript.clone(), SIZE[0]).await,
    ));
    let l1 = at(&mut view, &s1, 100.);
    let grown = find(&l1, &format!("item/{a}")).unwrap().rect.h;
    assert!(grown > h0 + 20., "the row grew: {h0} -> {grown}");
    assert!(
        near(
            find(&l1, &format!("item/{b}")).unwrap().rect.y,
            below0,
            0.01
        ),
        "on the frame the growth arrives, the row below has not moved yet"
    );
    fn clip_for(l: &Layout, id: ItemId) -> Option<&Layout> {
        // the synthetic scroll node wrapping a growing row: its first child is the row
        fn walk<'a>(n: &'a Layout, key: &str) -> Option<&'a Layout> {
            if let LayoutKind::ScrollView { .. } = n.kind
                && n.children.len() == 1
                && find(&n.children[0], key).is_some()
                && !matches!(&n.children[0].kind, LayoutKind::Named(name) if name == "transcript")
                && n.rect.h + 0.5 < n.children[0].rect.h
            {
                return Some(n);
            }
            n.children.iter().find_map(|c| walk(c, key))
        }
        walk(l, &format!("item/{id}"))
    }
    assert!(
        clip_for(&l1, a).is_some(),
        "a growing row is clipped to its slot"
    );

    let l2 = at(&mut view, &s1, 200.);
    let half = Ease::Out.at(0.5);
    let expect = h0 + (grown - h0) * half;
    let clip = clip_for(&l2, a).expect("still growing at 100 ms");
    assert!(
        near(clip.rect.h, expect, 0.5),
        "slot height at 100 ms is h0 + Δ·out(.5): {} vs {expect}",
        clip.rect.h
    );
    let below2 = find(&l2, &format!("item/{b}")).unwrap().rect.y;
    assert!(
        near(below2, below0 + (grown - h0) * half, 0.5),
        "the row below slides by the same amount: {below0} -> {below2}"
    );
    let l3 = at(&mut view, &s1, 320.);
    assert!(clip_for(&l3, a).is_none(), "at rest the row is not clipped");
    assert!(near(
        find(&l3, &format!("item/{b}")).unwrap().rect.y,
        below0 + (grown - h0),
        0.5
    ));
    assert_eq!(view.anim.next_frame(), None);
}

/// Tail follow: the scroll offset glides to the bottom with a critically damped ~180 ms
/// spring, monotone and without overshoot; a new chunk retargets mid-glide; a manual scroll
/// up stops the follow at once.
#[tokio::test]
async fn tail_follow_glides_and_retargets() {
    let mut preparer = Preparer::default();
    let long = "A paragraph of transcript text that wraps and takes up room.\n\n".repeat(12);
    let rows: Vec<ViewItem> = (0..6)
        .map(|_| message(ItemId::new(), &long, true))
        .collect();
    let s0 = state(&mut preparer, &rows).await;
    let mut view = live_view();
    view.render(&s0, SIZE).unwrap();
    let max0 = view.frame.max_scroll;
    assert!(max0 > 0., "taller than the viewport");
    assert_eq!(
        view.frame.scroll, max0,
        "a session's first frame shows its tail"
    );

    let grow = |s: &FrameState| {
        let mut next = s.clone();
        let mut t = (*s.transcript).clone();
        let id = ItemId::new();
        t.order.push_back(id);
        t.items.insert(id, message(id, &long, true));
        next.transcript = Arc::new(t);
        next
    };
    let mut s1 = grow(&s0);
    s1.prepared = Some(Arc::new(
        prepare(&mut preparer, s1.transcript.clone(), SIZE[0]).await,
    ));
    at(&mut view, &s1, 16.);
    let max1 = view.frame.max_scroll;
    assert!(max1 > max0 + 100.);
    assert!(
        view.frame.scroll < max1 && view.frame.scroll >= max0,
        "the glide starts from where the view was: {} in {max0}..{max1}",
        view.frame.scroll
    );
    assert_eq!(view.anim.next_frame(), Some(0.));
    let mut last = view.frame.scroll;
    let mut samples = vec![];
    for frame in 2..40 {
        at(&mut view, &s1, frame as f32 * 16.);
        let now = view.frame.scroll;
        assert!(
            now >= last - 0.001 && now <= max1 + 0.001,
            "monotone, no overshoot"
        );
        samples.push(now);
        last = now;
    }
    let at_192 = samples[10]; // frame 12 = 192 ms after the growth
    assert!(
        at_192 > max0 + 0.85 * (max1 - max0),
        "most of the glide is done after ~180 ms: {at_192} of {max0}..{max1}"
    );
    assert_eq!(view.frame.scroll, max1, "settled on the tail");
    assert_eq!(view.anim.next_frame(), None);

    // a chunk mid-glide retargets without a jump: add a row, step two frames into the
    // glide, add another, and confirm the offset keeps climbing continuously (no snap back
    // to a `from`, no jump to the new tail)
    let mut s2 = grow(&s1);
    s2.prepared = Some(Arc::new(
        prepare(&mut preparer, s2.transcript.clone(), SIZE[0]).await,
    ));
    at(&mut view, &s2, 640.);
    let mid = view.frame.scroll;
    let max2 = view.frame.max_scroll;
    assert!(
        mid > max1 && mid < max2,
        "one frame in, the glide is partway, not a jump: {mid} in {max1}..{max2}"
    );
    // retarget mid-glide: another row before the first glide finishes
    let mut s3 = grow(&s2);
    s3.prepared = Some(Arc::new(
        prepare(&mut preparer, s3.transcript.clone(), SIZE[0]).await,
    ));
    at(&mut view, &s3, 656.);
    let after = view.frame.scroll;
    let max3 = view.frame.max_scroll;
    assert!(
        after > mid && after < max3,
        "retargeted from the current value, no snap back and no jump: {mid} -> {after} (max {max3})"
    );

    // the user scrolls up: following stops, the offset is exact
    let max3 = view.frame.max_scroll;
    assert!(view.scroll.wheel(view.frame.scroll, max3, -200.));
    view.invalidate();
    at(&mut view, &s3, 764.);
    assert!(!view.scroll.following());
    assert!(near(view.frame.scroll, after - 200., 0.01));
    at(&mut view, &s3, 1500.);
    assert_eq!(view.anim.next_frame(), None, "a manual position is still");
}

/// Composer: focus border and ring over 200 ms `std`; the box height over 160 ms `out`; the
/// send colour idle → ready over 200 ms `std`; the caret glides to its column over 80 ms and
/// stays solid while typing, then blinks after 500 ms idle.
#[tokio::test]
async fn composer_focus_height_send_and_caret_tween() {
    use bbt_ui::text_edit::Command as Edit;
    // a non-empty transcript: no empty-session halo loop, so the only motion is the
    // composer's own
    let mut preparer = Preparer::default();
    let state = self::state(&mut preparer, &[message(ItemId::new(), "A reply.", true)]).await;
    let mut view = live_view();
    let border = |l: &Layout| {
        box_style(find(l, "composer").unwrap())
            .unwrap()
            .border
            .as_ref()
            .map(|b| b.color[3])
            .unwrap()
    };
    let ring = |l: &Layout| {
        box_style(find(l, "composer").unwrap())
            .unwrap()
            .rings
            .first()
            .map(|r| r.color[3])
    };
    let l0 = at(&mut view, &state, 0.);
    assert!(near(border(&l0), 0.10, 0.001), "hair at rest");
    assert_eq!(ring(&l0), None);
    view.focused = true;
    view.invalidate();
    let l1 = at(&mut view, &state, 1000.);
    assert!(near(border(&l1), 0.10, 0.001), "focus starts from hair");
    let l2 = at(&mut view, &state, 1100.);
    let half = Ease::Std.at(0.5);
    assert!(
        near(border(&l2), 0.10 + (0.55 - 0.10) * half, 0.003),
        "border at 100 ms: {} vs std(.5)",
        border(&l2)
    );
    assert!(
        near(ring(&l2).unwrap(), 0.14 * half, 0.003),
        "ring fades in with focus"
    );
    let l3 = at(&mut view, &state, 1200.);
    assert!(near(border(&l3), 0.55, 0.001) && near(ring(&l3).unwrap(), 0.14, 0.001));
    assert!(
        matches!(view.anim.next_frame(), Some(ms) if ms > 0.),
        "the focus tween settled; only the caret's timed blink remains, not motion: {:?}",
        view.anim.next_frame()
    );

    // the send button: card-hi -> seat as text arrives, the glow with it
    let send_fill = |l: &Layout| box_style(find(l, "composer/send").unwrap()).unwrap().fill;
    let idle_fill = send_fill(&l3);
    view.anim.tick(2000.);
    view.edit(Edit::Insert("hi".into()));
    let l4 = at(&mut view, &state, 2000.);
    assert_eq!(
        send_fill(&l4),
        idle_fill,
        "a tween starts from the current value"
    );
    let l5 = at(&mut view, &state, 2100.);
    let mid_fill = send_fill(&l5);
    let l6 = at(&mut view, &state, 2200.);
    let ready_fill = send_fill(&l6);
    assert_ne!(idle_fill, ready_fill);
    for i in 0..4 {
        let expect = ready_fill[i] * half + idle_fill[i] * (1. - half);
        assert!(
            near(mid_fill[i], expect, 0.01),
            "send fill channel {i} at 100 ms"
        );
    }

    // the caret glides: it is still at the old column on the frame of the edit
    let caret_x = |l: &Layout| find(l, "composer/text/caret").map(|n| n.rect.x);
    let x_hi = caret_x(&l6).expect("focused draft paints its caret");
    view.anim.tick(3000.);
    view.edit(Edit::Insert(" there".into()));
    let l7 = at(&mut view, &state, 3000.);
    assert!(
        near(caret_x(&l7).unwrap(), x_hi, 0.01),
        "glide starts from the old column"
    );
    let l8 = at(&mut view, &state, 3040.);
    let l9 = at(&mut view, &state, 3080.);
    let x_end = caret_x(&l9).unwrap();
    assert!(x_end > x_hi + 10.);
    assert!(
        near(
            caret_x(&l8).unwrap(),
            x_hi + (x_end - x_hi) * Ease::Out.at(0.5),
            0.5
        ),
        "caret at 40 ms is out(.5) of the way"
    );
    // solid for 500 ms after the edit, then a 1 s step blink
    let caret_on = |l: &Layout| !find(l, "composer/text/caret").unwrap().children.is_empty();
    assert!(caret_on(&l9));
    assert_eq!(
        view.anim.next_frame(),
        Some(420.),
        "wake at the first blink edge"
    );
    let l10 = at(&mut view, &state, 3499.);
    assert!(caret_on(&l10), "solid while typing");
    let l11 = at(&mut view, &state, 3500.);
    assert!(caret_on(&l11));
    let l12 = at(&mut view, &state, 4001.);
    assert!(!caret_on(&l12), "off in the second half of the step");
    assert_eq!(view.anim.next_frame(), Some(499.));
    let l13 = at(&mut view, &state, 4500.);
    assert!(caret_on(&l13));

    // the box height: a second line grows it 20 px over 160 ms out
    let box_h = |l: &Layout| find(l, "composer/body-viewport").unwrap().rect.h;
    let h1 = box_h(&l13);
    view.anim.tick(5000.);
    view.edit(Edit::Insert("\nmore".into()));
    let l14 = at(&mut view, &state, 5000.);
    assert!(near(box_h(&l14), h1, 0.01));
    let l15 = at(&mut view, &state, 5080.);
    assert!(
        near(box_h(&l15), h1 + 20. * Ease::Out.at(0.5), 0.5),
        "{}",
        box_h(&l15)
    );
    let l16 = at(&mut view, &state, 5160.);
    assert!(near(box_h(&l16), h1 + 20., 0.01));
    // and the feed above shifted with it
    let feed = |l: &Layout| find(l, "transcript").unwrap().rect.h;
    assert!(near(feed(&l16), feed(&l13) - 20., 0.01));
}

/// The send button crossfades arrow ↔ square as a turn starts with an empty draft.
#[test]
fn send_button_crossfades_arrow_and_square() {
    let idle = FrameState {
        transcript: Arc::new(transcript(&[])),
        ready: true,
        replay: false,
        ..Default::default()
    };
    let mut running = idle.clone();
    running.transcript = Arc::new(Transcript {
        phase: Phase::Streaming,
        ..(*idle.transcript).clone()
    });
    let mut view = live_view();
    let l0 = at(&mut view, &idle, 0.);
    fn paths(n: &Layout) -> usize {
        matches!(n.kind, LayoutKind::Path { .. }) as usize
            + n.children.iter().map(paths).sum::<usize>()
    }
    fn opacities(n: &Layout, out: &mut Vec<f32>) {
        if let LayoutKind::Filter { opacity, .. } = n.kind {
            out.push(opacity);
        }
        n.children.iter().for_each(|c| opacities(c, out));
    }
    let send0 = find(&l0, "composer/send").unwrap();
    assert_eq!(paths(send0), 1, "the arrow alone");
    let l1 = at(&mut view, &running, 100.);
    let l2 = at(&mut view, &running, 200.);
    let send2 = find(&l2, "composer/send").unwrap();
    assert_eq!(paths(send2), 2, "arrow and square during the crossfade");
    let mut o = vec![];
    opacities(send2, &mut o);
    let half = Ease::Std.at(0.5);
    assert_eq!(o.len(), 2);
    assert!(
        near(o[0], 1. - half, 0.01) && near(o[1], half, 0.01),
        "{o:?}"
    );
    assert!(
        find(&l1, "composer/sweep").is_some(),
        "the running sweep is on"
    );
    let l3 = at(&mut view, &running, 300.);
    let send3 = find(&l3, "composer/send").unwrap();
    assert_eq!(paths(send3), 1, "the square alone");
    let mut o = vec![];
    opacities(send3, &mut o);
    assert!(o.is_empty(), "no filter nodes at rest");
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the sweep loops while running"
    );
    // the sweep moves: its slices sit elsewhere 800 ms later
    let sweep = |l: &Layout| match &find(l, "composer/sweep").unwrap().children[0].kind {
        LayoutKind::Rects { rects, .. } => rects.iter().map(|(r, _)| r.x).fold(f32::MAX, f32::min),
        other => panic!("{other:?}"),
    };
    let l4 = at(&mut view, &running, 1100.);
    assert!(
        sweep(&l4) > sweep(&l3) + 50.,
        "{} -> {}",
        sweep(&l3),
        sweep(&l4)
    );
}

/// Idle: once every tween has settled and no loop is visible, further frames with the clock
/// advancing ask for no redraw and rebuild nothing.
#[tokio::test]
async fn an_idle_window_requests_no_frames() {
    let mut preparer = Preparer::default();
    let rows: Vec<ViewItem> = (0..3)
        .map(|i| message(ItemId::new(), &format!("Reply {i}."), true))
        .collect();
    let s0 = state(&mut preparer, &rows).await;
    let mut view = live_view();
    at(&mut view, &s0, 0.);
    // something moved: a row arrives
    let mut s1 = s0.clone();
    let mut t = (*s0.transcript).clone();
    let id = ItemId::new();
    t.order.push_back(id);
    t.items.insert(id, message(id, "Late reply.", true));
    s1.transcript = Arc::new(t);
    s1.prepared = Some(Arc::new(
        prepare(&mut preparer, s1.transcript.clone(), SIZE[0]).await,
    ));
    let mut frame = 1;
    while view.anim.next_frame().is_some() || frame == 1 {
        at(&mut view, &s1, frame as f32 * 16.);
        frame += 1;
        assert!(frame < 60, "settles within a second");
    }
    assert!(frame > 10, "the entrance took frames: {frame}");
    let requested = view.anim.frames_requested;
    let timings = view.frame.timings;
    for i in 0..120 {
        at(&mut view, &s1, 2000. + i as f32 * 16.);
        assert_eq!(view.anim.next_frame(), None, "idle at frame {i}");
    }
    assert_eq!(
        view.anim.frames_requested, requested,
        "zero redraw requests once settled"
    );
    assert_eq!(
        view.frame.timings, timings,
        "the cached frame was returned, not rebuilt"
    );
}

/// Reduced motion and the settled default land on the same end state as a live clock that
/// has finished moving, so static parity holds for every clock.
#[test]
fn every_clock_shares_the_end_state() {
    let state = FrameState {
        transcript: Arc::new(transcript(&[])),
        ready: true,
        replay: true,
        ..Default::default()
    };
    // step the live clock past the 600 ms entrance so every element but the halo has
    // settled (the halo loops forever, so the view never fully idles here)
    let mut live = live_view();
    let mut l = at(&mut live, &state, 0.);
    for f in 1..60 {
        l = at(&mut live, &state, f as f32 * 16.);
    }
    let mut reduced = HarnessView::default();
    reduced.anim = Clock::live(true);
    let r = at(&mut reduced, &state, 0.);
    let mut settled = HarnessView::default();
    let s = at(&mut settled, &state, 0.);
    assert_eq!(reduced.anim.next_frame(), None);
    assert_eq!(settled.anim.next_frame(), None);
    for key in [
        "empty",
        "empty/title",
        "empty/chip/0",
        "empty/chip/2",
        "composer",
        "composer/send",
    ] {
        let a = find(&l, key).unwrap().rect;
        let b = find(&r, key).unwrap().rect;
        let c = find(&s, key).unwrap().rect;
        assert!(
            near(a.x, b.x, 0.01) && near(a.y, b.y, 0.01) && near(a.h, b.h, 0.01),
            "{key}: live {a:?} vs reduced {b:?}"
        );
        assert!(near(b.x, c.x, 0.01) && near(b.y, c.y, 0.01) && near(b.h, c.h, 0.01));
    }
    assert_eq!(layout_text(&l), layout_text(&s));
}

/// The nearest opacity filter wrapping the keyed node, walking down from the root (an
/// entrance filter further up is passed over). None when the node is at rest.
fn nearest_filter(root: &Layout, key: &str) -> Option<f32> {
    let below = root.children.iter().find_map(|c| nearest_filter(c, key));
    if below.is_some() {
        return below;
    }
    if let LayoutKind::Filter { opacity, .. } = root.kind
        && find(root, key).is_some()
    {
        return Some(opacity);
    }
    None
}

/// The message source toggle is hover-only (harness-spec.md "Refinement pass 2"): nothing at
/// rest, .6 while its row is under the pointer, 1 over the control itself, each over 150 ms
/// `std`; back to nothing when the pointer leaves. Its hit zone stays. A settled clock never
/// shows it and asks for no frame.
#[tokio::test]
async fn source_toggle_shows_on_hover_only() {
    let mut preparer = Preparer::default();
    let id = ItemId::new();
    let s = state(&mut preparer, &[message(id, "A reply.", true)]).await;
    let key = format!("source/{id}/toggle");
    let mut view = live_view();
    let l0 = at(&mut view, &s, 0.);
    let toggle = find(&l0, &key).expect("the toggle keeps its hit zone");
    assert!(toggle.rect.w > 0., "the hit zone has a width");
    assert!(layout_text(toggle).is_empty(), "hidden at rest");
    assert_eq!(nearest_filter(&l0, &key), None);
    assert_eq!(view.anim.next_frame(), None, "nothing moves at rest");

    // the pointer enters the row: 0 → .6 over 150 ms `std`
    let row = find(&l0, &format!("item/{id}")).unwrap().rect;
    assert!(view.pointer([row.x + 10., row.y + 5.]));
    assert_eq!(view.hovered(), Some(format!("msg/{id}").as_str()));
    let l1 = at(&mut view, &s, 100.);
    assert_eq!(
        view.anim.next_frame(),
        Some(0.),
        "the fade asks for a frame"
    );
    assert!(
        layout_text(find(&l1, &key).unwrap()).is_empty(),
        "still nothing on the first frame of the fade"
    );
    let l2 = at(&mut view, &s, 175.);
    let want = 0.6 * Ease::Std.at(0.5);
    assert!(
        near(nearest_filter(&l2, &key).unwrap(), want, 0.01),
        "half way: {:?} vs {want}",
        nearest_filter(&l2, &key)
    );
    assert!(layout_text(find(&l2, &key).unwrap()).contains("source"));
    let l3 = at(&mut view, &s, 260.);
    assert!(
        near(nearest_filter(&l3, &key).unwrap(), 0.6, 0.001),
        "row hover rests at .6"
    );
    assert_eq!(view.anim.next_frame(), None, "settled at .6");

    // the pointer reaches the control: .6 → 1 over 150 ms `std`, then no filter at all
    let r = find(&l3, &key).unwrap().rect;
    assert!(view.pointer([r.x + 2., r.y + r.h / 2.]));
    assert_eq!(view.hovered(), Some(key.as_str()));
    let l4 = at(&mut view, &s, 300.);
    assert!(
        near(nearest_filter(&l4, &key).unwrap(), 0.6, 0.001),
        "retargets from .6"
    );
    let l5 = at(&mut view, &s, 375.);
    let want = 0.6 + 0.4 * Ease::Std.at(0.5);
    assert!(
        near(nearest_filter(&l5, &key).unwrap(), want, 0.01),
        "half way up: {:?} vs {want}",
        nearest_filter(&l5, &key)
    );
    let l6 = at(&mut view, &s, 460.);
    assert_eq!(nearest_filter(&l6, &key), None, "fully shown: no filter");
    assert!(layout_text(find(&l6, &key).unwrap()).contains("source"));
    assert_eq!(view.anim.next_frame(), None);

    // the pointer leaves: 1 → 0, then nothing again and no frame
    assert!(view.pointer([0., 0.]));
    assert_eq!(view.hovered(), None);
    let _ = at(&mut view, &s, 500.);
    let l7 = at(&mut view, &s, 575.);
    assert!(
        near(
            nearest_filter(&l7, &key).unwrap(),
            1. - Ease::Std.at(0.5),
            0.01
        ),
        "half way out: {:?}",
        nearest_filter(&l7, &key)
    );
    let l8 = at(&mut view, &s, 660.);
    assert!(
        layout_text(find(&l8, &key).unwrap()).is_empty(),
        "hidden again"
    );
    assert_eq!(view.anim.next_frame(), None);

    // settled (captures, the parity gate) and reduced motion: nothing, no frame
    for mut other in [HarnessView::default(), {
        let mut v = HarnessView::default();
        v.anim = Clock::live(true);
        v
    }] {
        let l = at(&mut other, &s, 0.);
        assert!(layout_text(find(&l, &key).unwrap()).is_empty());
        assert_eq!(other.anim.next_frame(), None);
    }
}

/// harness-spec.md "Tight transcript": a settled thought's chevron shows only while its row
/// is under the pointer (150 ms `std`) or the thought is open; at rest it draws nothing.
#[tokio::test]
async fn thought_chevron_shows_on_hover_or_open() {
    use bboxy450_harness::frontends::gui::rich::{Face, Faces};
    let mut preparer = Preparer::default();
    let id = ItemId::new();
    let mut row = message(id, "", true);
    row.kind = ItemKind::Reasoning;
    row.body = Item::Reasoning {
        summary: "Compare the files.".into(),
        raw: None,
        provider_opaque: None,
    };
    row.completed_ms = Some(400);
    let s = state(&mut preparer, &[row]).await;
    let key = format!("reasoning/{id}/toggle/chev");
    let zone = format!("reasoning/{id}/toggle");
    let mut view = live_view();
    let l0 = at(&mut view, &s, 0.);
    let shown = |l: &Layout| find(l, &key).is_some_and(|n| !n.children.is_empty());
    assert!(!shown(&l0), "hidden at rest");
    assert_eq!(view.anim.next_frame(), None);
    let r = find(&l0, &zone).expect("the thought row").rect;
    assert!(view.pointer([r.x + 5., r.y + r.h / 2.]));
    let _ = at(&mut view, &s, 100.);
    let l = at(&mut view, &s, 175.);
    assert!(
        near(nearest_filter(&l, &key).unwrap(), Ease::Std.at(0.5), 0.01),
        "half way in: {:?}",
        nearest_filter(&l, &key)
    );
    let l = at(&mut view, &s, 300.);
    assert!(
        shown(&l) && nearest_filter(&l, &key).is_none(),
        "shown while hovered"
    );
    assert!(view.pointer([0., 0.]));
    let _ = at(&mut view, &s, 400.);
    let l = at(&mut view, &s, 600.);
    assert!(!shown(&l), "hidden again");
    assert_eq!(view.anim.next_frame(), None);
    // open: shown without a pointer
    let mut open = s.clone();
    let mut faces = Faces::default();
    faces.insert(
        id,
        Face {
            reasoning_open: true,
            ..Default::default()
        },
    );
    // a toggle re-prepares the rows with the face, as Bridge::toggle does
    let (_preparer, prepared) = bboxy450_harness::runtime::blocking({
        let mut owned = std::mem::take(&mut preparer);
        let source = s.transcript.clone();
        let faces = faces.clone();
        move || {
            let result = owned.prepare_with_faces(source, SIZE[0], &faces);
            (owned, result)
        }
    })
    .await;
    open.prepared = Some(Arc::new(prepared.unwrap()));
    open.faces = Arc::new(faces);
    // the reveal starts on the first open frame and takes 150 ms
    let _ = at(&mut view, &open, 700.);
    let l = at(&mut view, &open, 900.);
    assert!(shown(&l), "open: shown");
    // settled: nothing at rest
    let mut settled = HarnessView::default();
    let l = at(&mut settled, &s, 0.);
    assert!(!shown(&l));
}
