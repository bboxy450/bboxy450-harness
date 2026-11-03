// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The standalone window's chrome (harness-spec.md "Standalone window" and the composer's
//! alive rim): the term's title bar and its hit regions, global text zoom steps, clamps,
//! reflow with the anchored row, the zoom pill, and the rim's activity mapping. No window,
//! no wall clock: the view is rendered by hand. scripts/test-chrome-control.py breaks the
//! clamp and the activity mapping and requires the marked tests to go red.
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        anim::{Clock, Ease},
        bridge::{Bridge, FrameState},
        chrome::{self, Hit},
        face,
        model::{Transcript, TurnState, ViewItem},
        view::{HarnessView, PreparedRows, Preparer, find},
        zoom::{self, Zoom},
    },
    protocol::*,
};
use bbt_ui::{GlowActivity, components::Layout};
use std::sync::Arc;

struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
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
fn message(id: ItemId, markdown: &str) -> ViewItem {
    ViewItem {
        id,
        turn: None,
        parent: None,
        started: Seq(1),
        completed: Some(Seq(2)),
        kind: ItemKind::AgentMessage,
        body: Item::AgentMessage {
            markdown: markdown.into(),
        },
        status: Some(ItemStatus::Completed),
        terminal_tail: String::new(),
        started_ms: 0,
        completed_ms: Some(1),
    }
}
fn transcript(rows: &[ViewItem]) -> Transcript {
    let mut t = Transcript {
        session: Some(SessionId::new()),
        profile: Some(("claude-acp".into(), "acp:claude-agent-acp · opus".into())),
        meta: Some(SessionMeta {
            source: None,
            session: SessionId::new(),
            cwd: "/tmp/acpdemo".into(),
            profile: Some("claude-acp".into()),
            fork: None,
            protocol: PROTOCOL,
            harness: "test".into(),
            created_ms: 0,
        }),
        ..Default::default()
    };
    for row in rows {
        t.order.push_back(row.id);
        t.items.insert(row.id, row.clone());
    }
    t
}
fn style(n: &Layout) -> Option<&bbt_ui::ResolvedStyle> {
    match &n.kind {
        bbt_ui::components::LayoutKind::Box { style, .. } => Some(style),
        _ => n.children.iter().find_map(style),
    }
}
fn composer_rim(l: &Layout) -> bbt_ui::Rim {
    style(find(l, "composer").expect("composer"))
        .expect("a box")
        .rim
        .expect("the composer wears the alive rim")
}
fn render(view: &mut HarnessView, state: &FrameState) -> Layout {
    view.render(state, SIZE).unwrap().layout.clone().unwrap()
}

// control: scripts/test-chrome-control.py (zoom clamp)
#[test]
fn zoom_steps_and_clamps_on_the_ten_percent_grid() {
    let mut z = Zoom::default();
    assert_eq!(z.percent(), 100);
    let mut seen = vec![];
    for _ in 0..9 {
        z = z.step(1);
        seen.push(z.percent());
    }
    assert_eq!(seen, [110, 120, 130, 140, 150, 160, 160, 160, 160]);
    let mut seen = vec![];
    for _ in 0..9 {
        z = z.step(-1);
        seen.push(z.percent());
    }
    assert_eq!(seen, [150, 140, 130, 120, 110, 100, 90, 90, 90]);
    assert_eq!(Zoom::from_percent(45).percent(), zoom::MIN);
    assert_eq!(Zoom::from_percent(999).percent(), zoom::MAX);
    assert_eq!(Zoom::from_percent(125).percent(), 130, "snaps to the grid");
    for p in (zoom::MIN..=zoom::MAX).step_by(usize::from(zoom::STEP)) {
        let z = Zoom::from_percent(i32::from(p));
        assert_eq!(z.percent(), p);
        assert!((z.scale() - f32::from(p) / 100.).abs() < 1e-6);
        assert_eq!(z.label(), format!("{p}%"));
    }
}

/// Cmd+= / Cmd+- / Cmd+0 through the view: the composer's draft text, the header and the
/// prepared rows all scale, nothing else does, and the layout reflows.
#[tokio::test]
async fn zoom_scales_every_text_size_and_the_layout_reflows() {
    let bridge = Bridge::new(Arc::new(Wake));
    let mut preparer = Preparer::default();
    let rows: Vec<ViewItem> = (0..3)
        .map(|i| {
            message(
                ItemId::new(),
                &format!("Reply {i} with a few words of prose."),
            )
        })
        .collect();
    let model = Arc::new(transcript(&rows));
    let mut state = FrameState {
        transcript: model.clone(),
        ready: true,
        replay: true,
        ..Default::default()
    };
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, model.clone(), SIZE[0]).await,
    ));
    let mut view = HarnessView::default();
    view.edit(bbt_ui::text_edit::Command::Insert("hello".into()));
    let l100 = render(&mut view, &state);
    fn sizes(n: &Layout, out: &mut Vec<f32>) {
        if let bbt_ui::components::LayoutKind::Text { runs, .. } = &n.kind {
            out.extend(runs.iter().map(|r| r.style.size));
        }
        n.children.iter().for_each(|c| sizes(c, out));
    }
    let composer_sizes = |l: &Layout| {
        let mut out = vec![];
        sizes(find(l, "composer").unwrap(), &mut out);
        out
    };
    let header_sizes = |l: &Layout| {
        let mut out = vec![];
        sizes(find(l, "toolbar").unwrap(), &mut out);
        out
    };
    let row_height = |l: &Layout| find(l, &format!("item/{}", rows[0].id)).unwrap().rect.h;
    let send = |l: &Layout| find(l, "composer/send").unwrap().rect;
    let before = (
        composer_sizes(&l100),
        header_sizes(&l100),
        row_height(&l100),
        send(&l100),
    );
    assert!(before.0.contains(&13.), "the draft is 13 px at 100 %");

    assert!(view.set_zoom(Zoom::from_percent(130), &bridge));
    assert_eq!(view.zoom().percent(), 130);
    // the rows come back from the preparer at the new zoom, as the bridge does
    preparer.set_zoom(view.zoom());
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, model.clone(), SIZE[0]).await,
    ));
    assert_eq!(state.prepared.as_ref().unwrap().zoom, 130);
    let l130 = render(&mut view, &state);
    let after = (
        composer_sizes(&l130),
        header_sizes(&l130),
        row_height(&l130),
        send(&l130),
    );
    assert_eq!(after.0.len(), before.0.len());
    for (a, b) in before.0.iter().zip(&after.0) {
        assert!((b / a - 1.3).abs() < 0.01, "composer text {a} -> {b}");
    }
    assert_eq!(after.1.len(), before.1.len());
    for (a, b) in before.1.iter().zip(&after.1) {
        assert!((b / a - 1.3).abs() < 0.01, "header text {a} -> {b}");
    }
    assert!(
        after.2 > before.2 + 4.,
        "a row reflows taller: {} -> {}",
        before.2,
        after.2
    );
    assert_eq!(
        (after.3.w, after.3.h),
        (26., 26.),
        "the send circle keeps its size"
    );
    assert!(
        find(&l130, "composer").unwrap().rect.h > find(&l100, "composer").unwrap().rect.h,
        "the composer box grows with its line height"
    );
    assert!(find(&l130, "toolbar").unwrap().rect.h >= 40.);
    assert_eq!(
        find(&l130, "transcript").unwrap().rect.bottom(),
        find(&l130, "composer").unwrap().rect.y - 8.,
        "the feed ends 8 px above the box at every zoom"
    );
    // Cmd+0 resets
    view.set_zoom(Zoom::default(), &bridge);
    preparer.set_zoom(view.zoom());
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, model.clone(), SIZE[0]).await,
    ));
    let back = render(&mut view, &state);
    assert_eq!(composer_sizes(&back), before.0);
    assert_eq!(row_height(&back), before.2);
}

/// Zooming keeps the row at the top of the viewport where it was (harness-spec.md
/// Standalone window: "The scroll position stays anchored to the top visible block").
#[tokio::test]
async fn reflow_keeps_the_anchored_row() {
    let bridge = Bridge::new(Arc::new(Wake));
    let mut preparer = Preparer::default();
    let rows: Vec<ViewItem> = (0..40)
        .map(|i| {
            message(
                ItemId::new(),
                &format!("Reply {i}. A paragraph long enough to take a line or two of the feed."),
            )
        })
        .collect();
    let model = Arc::new(transcript(&rows));
    let mut state = FrameState {
        transcript: model.clone(),
        ready: true,
        replay: true,
        ..Default::default()
    };
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, model.clone(), SIZE[0]).await,
    ));
    let mut view = HarnessView::default();
    render(&mut view, &state);
    let max = view.frame.max_scroll;
    assert!(max > 400., "a scrollable feed: {max}");
    // scroll into the middle: row k sits a little under the viewport's top
    view.scroll.wheel(view.frame.scroll, max, -(max / 2.));
    view.invalidate();
    let l = render(&mut view, &state);
    let viewport = |l: &Layout| find(l, "transcript").unwrap().children[0].rect;
    let (top_id, ..) = view.frame.visible[0];
    let top_row = |l: &Layout| find(l, &format!("item/{top_id}")).unwrap().rect;
    let dy_before = top_row(&l).y - viewport(&l).y;
    assert!(
        dy_before <= 0. && dy_before > -400.,
        "row {top_id} straddles the top: {dy_before}"
    );

    assert!(view.set_zoom(Zoom::from_percent(150), &bridge));
    // until the rows arrive at the new zoom the old rows and offset stay (no jump)
    let stale = render(&mut view, &state);
    assert_eq!(top_row(&stale).y - viewport(&stale).y, dy_before);
    preparer.set_zoom(view.zoom());
    state.prepared = Some(Arc::new(
        prepare(&mut preparer, model.clone(), SIZE[0]).await,
    ));
    let l150 = render(&mut view, &state);
    assert!(view.frame.max_scroll > max, "taller rows, a longer feed");
    let dy_after = top_row(&l150).y - viewport(&l150).y;
    assert!(
        (dy_after - dy_before).abs() < 0.5,
        "the anchored row stays put: {dy_before} -> {dy_after}"
    );
    assert!(
        !view.scroll.following(),
        "a mid-feed anchor is a manual position"
    );
    assert_eq!(
        view.anim.next_frame(),
        None,
        "a settled view asks for no frame"
    );
}

/// The pill: `110%` centred 12 px under the feed's top, shown for 900 ms then gone; a
/// settled clock paints the end states.
#[test]
fn zoom_pill_shows_the_percent_then_fades() {
    let bridge = Bridge::new(Arc::new(Wake));
    let state = FrameState {
        transcript: Arc::new(transcript(&[message(ItemId::new(), "A reply.")])),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut view = HarnessView::default();
    view.anim = Clock::live(false);
    view.anim.tick(0.);
    let frame = view.render(&state, SIZE).unwrap();
    assert!(frame.over.primitives.is_empty(), "no pill before a zoom");
    let feed = find(frame.layout.as_ref().unwrap(), "transcript")
        .unwrap()
        .rect;
    view.anim.tick(1000.);
    view.set_zoom(Zoom::from_percent(110), &bridge);
    let frame = view.render(&state, SIZE).unwrap();
    let texts = |prims: &[bbt_ui_model::draw::Prim]| -> Vec<(String, [f32; 2])> {
        fn walk(p: &bbt_ui_model::draw::Prim, out: &mut Vec<(String, [f32; 2])>) {
            match p {
                bbt_ui_model::draw::Prim::Text { text, origin, .. } => {
                    out.push((text.to_string(), *origin))
                }
                bbt_ui_model::draw::Prim::Group { primitives, .. } => {
                    primitives.iter().for_each(|p| walk(p, out))
                }
                _ => {}
            }
        }
        let mut out = vec![];
        prims.iter().for_each(|p| walk(p, &mut out));
        out
    };
    let shown = texts(&frame.over.primitives);
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert_eq!(shown[0].0, "110%");
    let pill = find(frame.layout.as_ref().unwrap(), "zoom/pill");
    assert!(
        pill.is_none(),
        "the pill is an overlay, not part of the face's tree"
    );
    assert!(
        matches!(frame.over.primitives.first(), Some(bbt_ui_model::draw::Prim::Group { opacity, .. }) if *opacity < 0.05),
        "it drops in from transparent"
    );
    assert_eq!(view.anim.next_frame(), Some(0.), "the drop is motion");
    view.anim.tick(1170.);
    let frame = view.render(&state, SIZE).unwrap();
    assert!(
        !matches!(
            frame.over.primitives.first(),
            Some(bbt_ui_model::draw::Prim::Group { .. })
        ),
        "fully in: painted plain"
    );
    let (_, origin) = texts(&frame.over.primitives)[0];
    assert!(
        (origin[0] - (feed.x + feed.w / 2.)).abs() < 60.,
        "centred over the feed: {origin:?}"
    );
    assert!(
        origin[1] > feed.y + 12. && origin[1] < feed.y + 40.,
        "12 px under the feed's top"
    );
    assert!(
        matches!(view.anim.next_frame(), Some(ms) if ms > 0. && ms <= 730.),
        "a timed wake ends the hold: {:?}",
        view.anim.next_frame()
    );
    // the timed wake fires at the hold's end and starts the fade; 75 ms later it is halfway
    view.anim.tick(1900.);
    view.render(&state, SIZE).unwrap();
    assert_eq!(view.anim.next_frame(), Some(0.), "the fade is motion");
    view.anim.tick(1975.);
    let frame = view.render(&state, SIZE).unwrap();
    let expected = 1. - Ease::Std.at(0.5);
    assert!(
        matches!(frame.over.primitives.first(), Some(bbt_ui_model::draw::Prim::Group { opacity, .. }) if (*opacity - expected).abs() < 0.02),
        "fading at the fade's middle: std(.5) from 1 toward 0"
    );
    view.anim.tick(2100.);
    let frame = view.render(&state, SIZE).unwrap();
    assert!(frame.over.primitives.is_empty(), "gone after 1050 ms");
    // a settled clock: whole, then gone at the hold's end
    let mut settled = HarnessView::default();
    settled.render(&state, SIZE).unwrap();
    settled.set_zoom(Zoom::from_percent(120), &bridge);
    let frame = settled.render(&state, SIZE).unwrap();
    assert_eq!(texts(&frame.over.primitives)[0].0, "120%");
    assert!(!matches!(
        frame.over.primitives.first(),
        Some(bbt_ui_model::draw::Prim::Group { .. })
    ));
    settled.anim.tick(899.);
    assert!(
        !settled
            .render(&state, SIZE)
            .unwrap()
            .over
            .primitives
            .is_empty()
    );
    settled.anim.tick(901.);
    assert!(
        settled
            .render(&state, SIZE)
            .unwrap()
            .over
            .primitives
            .is_empty()
    );
}

// control: scripts/test-chrome-control.py (rim activity)
#[test]
fn rim_activity_follows_run_ask_error_and_idle() {
    use GlowActivity as A;
    assert_eq!(face::rim_activity(false, false, false), A::Idle);
    assert_eq!(face::rim_activity(true, false, false), A::Active);
    assert_eq!(
        face::rim_activity(true, true, false),
        A::Waiting,
        "ask over run"
    );
    assert_eq!(face::rim_activity(false, true, false), A::Waiting);
    assert_eq!(face::rim_activity(false, false, true), A::Errored);
    assert_eq!(
        face::rim_activity(true, false, true),
        A::Active,
        "a new turn clears the error"
    );
    assert_eq!(face::swell_envelope(0.), 0.);
    assert!((face::swell_envelope(0.5) - 1.).abs() < 1e-6);
    assert_eq!(face::swell_envelope(1.), 0., "a run swell is 0");
    assert_eq!(face::swell_envelope(-0.1), 0.);
}

// control: scripts/test-chrome-control.py (rim activity)
#[test]
fn composer_rim_reads_the_session() {
    let mut view = HarnessView::default();
    let turn = TurnId::new();
    let mut render_phase = |phase: Phase, ended: Option<StopReason>| {
        let mut t = transcript(&[message(ItemId::new(), "A reply.")]);
        t.phase = phase;
        if let Some(reason) = ended {
            t.turns.insert(
                turn,
                TurnState::Ended {
                    reason,
                    usage: None,
                },
            );
        }
        let state = FrameState {
            transcript: Arc::new(t),
            ready: true,
            ..Default::default()
        };
        view.invalidate();
        composer_rim(&render(&mut view, &state))
    };
    let idle = render_phase(Phase::Idle, Some(StopReason::EndTurn));
    assert_eq!(idle.activity, GlowActivity::Idle);
    assert_eq!(idle.seed, face::RIM_SEED);
    assert_eq!(idle.swell, 0.);
    assert!(!idle.focused);
    assert_eq!(
        idle.tint,
        Some(face::ink(bboxy450_harness::frontends::gui::view::context()).seat)
    );
    assert_eq!(
        idle.depth,
        Some(6.),
        "the bloom stops at the box's smallest padding"
    );
    assert_eq!(
        idle.outer,
        Some(2.),
        "the outer half fits the 8 px above the box"
    );
    assert_eq!(
        render_phase(Phase::Streaming, None).activity,
        GlowActivity::Active
    );
    assert_eq!(
        render_phase(
            Phase::Approval {
                item: ItemId::new()
            },
            None
        )
        .activity,
        GlowActivity::Waiting
    );
    assert_eq!(
        render_phase(
            Phase::Idle,
            Some(StopReason::Error {
                class: ErrorClass::Transient,
                message: "dropped".into(),
            }),
        )
        .activity,
        GlowActivity::Errored
    );
    assert_eq!(
        render_phase(
            Phase::Thinking,
            Some(StopReason::Error {
                class: ErrorClass::Transient,
                message: "dropped".into(),
            }),
        )
        .activity,
        GlowActivity::Active,
        "a running turn after an error is Active"
    );
    // the border stays constant at seat-35; no sweep, no ring
    let state = FrameState {
        transcript: Arc::new(transcript(&[])),
        ready: true,
        ..Default::default()
    };
    view.invalidate();
    let l = render(&mut view, &state);
    let s = style(find(&l, "composer").unwrap()).unwrap();
    assert!((s.border.as_ref().unwrap().color[3] - 0.35).abs() < 0.001);
    assert!(s.rings.is_empty());
    assert!(find(&l, "composer/sweep").is_none());
    assert_eq!(s.material, bbt_ui::Material::Rim);
    // the frame carries the rim for the material pass
    let frame = view.render(&state, SIZE).unwrap();
    assert_eq!(frame.rims.len(), 1, "one alive rim: the composer's");
    let composer = find(frame.layout.as_ref().unwrap(), "composer")
        .unwrap()
        .rect;
    assert_eq!(frame.rims[0].rect, composer);
    assert!(matches!(
        frame.rims[0].material,
        bbt_render::chrome::materials::Material::Rim(r)
            if r.activity == bbt_render::chrome::materials::Activity::Idle && r.outer == 2. && r.depth == 6.
    ));
}

/// The standalone window's title bar: dots, mark, title, crumb, theme control, no sidebar
/// toggle; its hit regions come from the painted tree. Captures and tests draw no bar.
#[test]
fn title_bar_is_drawn_with_the_crumb_and_hits_follow_the_tree() {
    let state = FrameState {
        transcript: Arc::new(transcript(&[message(ItemId::new(), "A reply.")])),
        ready: true,
        replay: true,
        ..Default::default()
    };
    let mut plain = HarnessView::default();
    let l = render(&mut plain, &state);
    assert!(find(&l, "bar").is_none(), "no chrome by default");
    assert_eq!(find(&l, "toolbar").unwrap().rect.y, 0.);
    assert_eq!(plain.titlebar.hit([10., 10.]), None);

    let mut view = HarnessView::default();
    view.chrome = true;
    view.focused = true;
    let l = render(&mut view, &state);
    let bar = find(&l, "bar").expect("the title bar").rect;
    assert_eq!(
        (bar.x, bar.y, bar.w, bar.h),
        (0., 0., SIZE[0], chrome::TITLE_BAR_H)
    );
    assert_eq!(
        find(&l, "toolbar").unwrap().rect.y,
        bar.h,
        "the face starts under the bar"
    );
    let text = bboxy450_harness::frontends::gui::view::layout_text(find(&l, "bar").unwrap());
    assert!(text.contains("bboxy450-harness"), "{text}");
    assert!(text.contains("claude-acp · acpdemo"), "{text}");
    assert!(find(&l, "bar/tbtn").is_none(), "no sidebar toggle");
    assert!(find(&l, "bar/tbtn1").is_some(), "the theme control");
    assert!(find(&l, "bar/logo").is_some());
    for key in ["bar/win/i", "bar/win/i1", "bar/win/i2"] {
        let dot = find(&l, key).unwrap().rect;
        assert_eq!((dot.w, dot.h), (10., 10.), "{key}");
    }
    let centre = |r: bbt_ui_model::layout::Rect| [r.x + r.w / 2., r.y + r.h / 2.];
    assert_eq!(
        view.titlebar
            .hit(centre(find(&l, "bar/win/i").unwrap().rect)),
        Some(Hit::Close)
    );
    assert_eq!(
        view.titlebar
            .hit(centre(find(&l, "bar/win/i1").unwrap().rect)),
        Some(Hit::Minimize)
    );
    assert_eq!(
        view.titlebar
            .hit(centre(find(&l, "bar/win/i2").unwrap().rect)),
        Some(Hit::Fullscreen)
    );
    assert_eq!(
        view.titlebar
            .hit(centre(find(&l, "bar/tbtn1").unwrap().rect)),
        Some(Hit::Theme)
    );
    assert_eq!(
        view.titlebar.hit([SIZE[0] / 2., bar.h / 2.]),
        Some(Hit::Background)
    );
    assert_eq!(
        view.titlebar.hit([SIZE[0] / 2., bar.h + 1.]),
        None,
        "the face is not the bar"
    );
    let now = std::time::Instant::now();
    let interval = std::time::Duration::from_millis(500);
    assert_eq!(
        view.titlebar.press(
            [SIZE[0] / 2., 19.],
            now,
            interval,
            bbt_desktop_core::TitlebarAction::Zoom
        ),
        Some(chrome::Intent::Drag)
    );
    assert_eq!(
        view.titlebar.press(
            [SIZE[0] / 2., 19.],
            now + interval,
            interval,
            bbt_desktop_core::TitlebarAction::Zoom
        ),
        Some(chrome::Intent::Titlebar(
            bbt_desktop_core::TitlebarAction::Zoom
        ))
    );
    assert_eq!(
        view.titlebar.press(
            centre(find(&l, "bar/win/i").unwrap().rect),
            now,
            interval,
            bbt_desktop_core::TitlebarAction::Zoom
        ),
        Some(chrome::Intent::Window(
            bbt_desktop_core::Action::CloseWindow
        ))
    );
    // the bar's text is chrome: the zoom leaves it alone
    let bridge = Bridge::new(Arc::new(Wake));
    view.set_zoom(Zoom::from_percent(160), &bridge);
    let l160 = render(&mut view, &state);
    assert_eq!(find(&l160, "bar").unwrap().rect.h, chrome::TITLE_BAR_H);
    fn sizes(n: &Layout, out: &mut Vec<f32>) {
        if let bbt_ui::components::LayoutKind::Text { runs, .. } = &n.kind {
            out.extend(runs.iter().map(|r| r.style.size));
        }
        n.children.iter().for_each(|c| sizes(c, out));
    }
    let (mut a, mut b) = (vec![], vec![]);
    sizes(find(&l, "bar").unwrap(), &mut a);
    sizes(find(&l160, "bar").unwrap(), &mut b);
    assert_eq!(a, b, "title bar type is not zoomed");
}
