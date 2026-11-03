// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Stream and card motion (harness-spec.md "Motion"), proven with numbers: a live clock
//! stepped in fixed increments, tween values asserted at sample times, and no frame
//! requested once everything has settled.
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        anim::{Clock, Ease},
        bridge::FrameState,
        model::{Transcript, TurnState, ViewItem},
        rich::{Face, Faces},
        stream_motion::{self, count_up, perimeter_point, turned},
        view::{Frame, HarnessView, Preparer, find},
    },
    protocol::*,
    runtime,
};
use bbt_ui::components::{Layout, LayoutKind, PositionedRun};
use bbt_ui_model::{draw::Prim, layout::Rect};
use std::sync::Arc;

const SIZE: [f32; 2] = [900., 600.];

fn item(kind: ItemKind, body: Item, turn: TurnId, live: bool) -> ViewItem {
    ViewItem {
        id: ItemId::new(),
        turn: Some(turn),
        parent: None,
        started: Seq(1),
        completed: if live { None } else { Some(Seq(2)) },
        kind,
        body,
        status: if live {
            None
        } else {
            Some(ItemStatus::Completed)
        },
        terminal_tail: String::new(),
        started_ms: 0,
        completed_ms: if live { None } else { Some(1500) },
    }
}
fn transcript(items: Vec<ViewItem>) -> (Transcript, TurnId) {
    let mut model = Transcript::default();
    let turn = items[0].turn.unwrap();
    for row in items {
        model.order.push_back(row.id);
        model.items.insert(row.id, row);
    }
    model.turns.insert(turn, TurnState::Running);
    (model, turn)
}
fn state(p: &mut Preparer, model: &Transcript, faces: Faces) -> FrameState {
    let faces = Arc::new(faces);
    let source = Arc::new(model.clone());
    let prepared = p
        .prepare_with_faces(source.clone(), SIZE[0], &faces)
        .unwrap();
    FrameState {
        transcript: source,
        ready: true,
        replay: true,
        prepared: Some(Arc::new(prepared)),
        faces,
        ..Default::default()
    }
}
fn live_view() -> HarnessView {
    let mut view = HarnessView::default();
    view.anim = Clock::live(false);
    view.anim.tick(0.);
    view
}
fn runs<'a>(n: &'a Layout, out: &mut Vec<&'a PositionedRun>) {
    if let LayoutKind::Text { runs, .. } = &n.kind {
        out.extend(runs.iter());
    }
    for c in &n.children {
        runs(c, out);
    }
}
/// The alpha of the run holding `needle`.
fn alpha_of(frame: &Frame, needle: &str) -> f32 {
    let mut all = vec![];
    runs(frame.layout.as_ref().unwrap(), &mut all);
    all.iter()
        .find(|r| r.text.contains(needle))
        .unwrap_or_else(|| panic!("no run holds {needle:?}"))
        .style
        .color[3]
}
/// The opacity filter wrapped around the node named `name`, if any.
fn filter_over(n: &Layout, name: &str) -> Option<f32> {
    if let LayoutKind::Filter { opacity, .. } = &n.kind
        && n.children.len() == 1
        && matches!(&n.children[0].kind, LayoutKind::Named(k) if k == name)
    {
        return Some(*opacity);
    }
    n.children.iter().find_map(|c| filter_over(c, name))
}
fn paths<'a>(n: &'a Layout, out: &mut Vec<&'a Layout>) {
    if matches!(n.kind, LayoutKind::Path { .. }) {
        out.push(n);
    }
    for c in &n.children {
        paths(c, out);
    }
}
fn points(n: &Layout) -> Vec<[f32; 2]> {
    let LayoutKind::Path { points, .. } = &n.kind else {
        panic!("not a path");
    };
    points
        .iter()
        .filter_map(|p| match p {
            bbt_ui_model::draw::PathCommand::Move(p) | bbt_ui_model::draw::PathCommand::Line(p) => {
                Some(*p)
            }
            _ => None,
        })
        .collect()
}
fn near(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}
/// The orbit arcs: 1px butt-capped strokes at the frame's origin, after the rows.
fn orbit_prims(frame: &Frame) -> usize {
    frame
        .list
        .primitives
        .iter()
        .filter(|p| matches!(p, Prim::Path { origin: [0., 0.], stroke, fill: None, .. } if stroke.width == 1. && stroke.cap == bbt_ui_model::draw::LineCap::Butt))
        .count()
}
fn sheen_groups(frame: &Frame) -> usize {
    frame
        .list
        .primitives
        .iter()
        .filter(|p| matches!(p, Prim::Group { mask: Some(_), .. }))
        .count()
}

#[tokio::test]
async fn a_streamed_chunk_fades_over_140ms_and_settled_text_never_refades() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (mut model, _) = transcript(vec![item(
            ItemKind::AgentMessage,
            Item::AgentMessage {
                markdown: "Hello world".into(),
            },
            turn,
            true,
        )]);
        let id = model.order[0];
        let mut p = Preparer::default();
        let mut view = live_view();
        // the first frame settles what is already on screen
        let s0 = state(&mut p, &model, Faces::default());
        let frame = view.render(&s0, SIZE).unwrap();
        assert_eq!(alpha_of(frame, "Hello"), 1.);
        assert_eq!(
            view.anim.next_frame(),
            None,
            "a settled stream asks for no frame"
        );
        // a chunk arrives
        model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
            markdown: "Hello world and more".into(),
        };
        let s1 = state(&mut p, &model, Faces::default());
        let mut samples = vec![];
        for t in [0., 35., 70., 105., 140.] {
            view.anim.tick(t);
            let frame = view.render(&s1, SIZE).unwrap();
            samples.push((t, alpha_of(frame, "more"), alpha_of(frame, "Hello")));
        }
        for (t, new, old) in &samples {
            let want = Ease::Out.at(t / stream_motion::FADE_MS);
            assert!(
                near(*new, want, 1e-3),
                "t={t}ms: chunk alpha {new}, want out({}) = {want}",
                t / stream_motion::FADE_MS
            );
            assert_eq!(*old, 1., "t={t}ms: settled text re-faded");
        }
        assert_eq!(samples[0].1, 0., "a chunk starts invisible");
        assert_eq!(samples[4].1, 1., "and is solid at 140ms");
        assert!(view.anim.frames_requested >= 4, "the clock drove the fade");
        println!("chunk fade samples (t, chunk alpha, settled alpha): {samples:?}");
        // idle: once settled nothing asks for a frame, and the frame is served from cache
        assert_eq!(view.anim.next_frame(), None);
        let requested = view.anim.frames_requested;
        for t in [200., 400., 1000.] {
            view.anim.tick(t);
            let frame = view.render(&s1, SIZE).unwrap();
            assert_eq!(alpha_of(frame, "more"), 1.);
            assert_eq!(
                view.anim.next_frame(),
                None,
                "t={t}ms: idle must not request a frame"
            );
        }
        assert_eq!(
            view.anim.frames_requested, requested,
            "zero redraw requests once settled"
        );
        // the next chunk fades from where it starts; the earlier chunk stays solid
        model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
            markdown: "Hello world and more words".into(),
        };
        let s2 = state(&mut p, &model, Faces::default());
        view.anim.tick(1000.);
        let frame = view.render(&s2, SIZE).unwrap();
        assert_eq!(alpha_of(frame, "words"), 0.);
        assert_eq!(alpha_of(frame, "more"), 1.);
        view.anim.tick(1070.);
        let frame = view.render(&s2, SIZE).unwrap();
        assert!(near(alpha_of(frame, "words"), Ease::Out.at(0.5), 1e-3));
    })
    .await;
}

#[tokio::test]
async fn a_settled_clock_draws_end_states_and_never_asks_for_a_frame() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (mut model, _) = transcript(vec![item(
            ItemKind::AgentMessage,
            Item::AgentMessage {
                markdown: "Hello".into(),
            },
            turn,
            true,
        )]);
        let id = model.order[0];
        let mut p = Preparer::default();
        let mut view = HarnessView::default();
        view.render(&state(&mut p, &model, Faces::default()), SIZE)
            .unwrap();
        model.items.get_mut(&id).unwrap().body = Item::AgentMessage {
            markdown: "Hello again".into(),
        };
        let frame = view
            .render(&state(&mut p, &model, Faces::default()), SIZE)
            .unwrap();
        assert_eq!(alpha_of(frame, "again"), 1.);
        assert_eq!(view.anim.next_frame(), None);
        assert_eq!(view.anim.frames_requested, 0);
    })
    .await;
}

#[tokio::test]
async fn a_fold_turns_the_chevron_and_fades_the_body_over_200ms() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (model, _) = transcript(vec![item(
            ItemKind::Reasoning,
            Item::Reasoning {
                summary: "Compare the files. Then decide what to change.".into(),
                raw: None,
                provider_opaque: None,
            },
            turn,
            false,
        )]);
        let id = model.order[0];
        let chev = format!("reasoning/{id}/toggle/chev");
        let body = format!("reasoning/{id}/body");
        let mut p = Preparer::default();
        let mut view = live_view();
        let closed = state(&mut p, &model, Faces::default());
        let frame = view.render(&closed, SIZE).unwrap();
        assert!(find(frame.layout.as_ref().unwrap(), &body).is_none());
        let chevron = |frame: &Frame| {
            let node = find(frame.layout.as_ref().unwrap(), &chev).expect("chevron");
            let mut ps = vec![];
            paths(node, &mut ps);
            points(ps[0])
        };
        let at = |deg: f32| -> Vec<[f32; 2]> {
            turned(
                &bboxy450_harness::frontends::gui::face::icon_points("chev", 12., 0.),
                deg,
                [6., 6.],
            )
            .iter()
            .filter_map(|p| match p {
                bbt_ui_model::draw::PathCommand::Move(p)
                | bbt_ui_model::draw::PathCommand::Line(p) => Some(*p),
                _ => None,
            })
            .collect()
        };
        let same = |a: &[[f32; 2]], b: &[[f32; 2]]| {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(p, q)| near(p[0], q[0], 1e-3) && near(p[1], q[1], 1e-3))
        };
        assert!(same(&chevron(frame), &at(0.)), "closed: chevron at 0°");
        // open it
        let mut faces = Faces::default();
        faces.insert(
            id,
            Face {
                reasoning_open: true,
                ..Default::default()
            },
        );
        let open = state(&mut p, &model, faces);
        let mut samples = vec![];
        for t in [0., 50., 100., 150., 200.] {
            view.anim.tick(t);
            let frame = view.render(&open, SIZE).unwrap();
            let want = Ease::Out.at(t / stream_motion::FOLD_MS);
            assert!(
                same(&chevron(frame), &at(90. * want)),
                "t={t}ms: chevron must be at {}°",
                90. * want
            );
            let opacity = filter_over(frame.layout.as_ref().unwrap(), &body);
            samples.push((t, 90. * want, opacity));
            if want < 1. {
                assert!(
                    near(opacity.expect("body fades"), want, 1e-3),
                    "t={t}ms: body opacity {opacity:?}, want {want}"
                );
            } else {
                assert_eq!(opacity, None, "settled: no filter around the body");
            }
            assert!(find(frame.layout.as_ref().unwrap(), &body).is_some());
        }
        println!("fold samples (t, chevron °, body opacity): {samples:?}");
        assert_eq!(view.anim.next_frame(), None, "the fold has settled");
        // close it again: the chevron turns back from 90°
        let closed_again = state(&mut p, &model, Faces::default());
        view.anim.tick(300.);
        let frame = view.render(&closed_again, SIZE).unwrap();
        assert!(same(&chevron(frame), &at(90.)), "closing starts from 90°");
        view.anim.tick(400.);
        let frame = view.render(&closed_again, SIZE).unwrap();
        assert!(same(&chevron(frame), &at(90. * (1. - Ease::Out.at(0.5)))));
        view.anim.tick(600.);
        view.render(&closed_again, SIZE).unwrap();
        assert_eq!(view.anim.next_frame(), None);
    })
    .await;
}

#[tokio::test]
async fn a_running_card_orbits_and_a_live_thought_sheens_until_they_finish() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (mut model, _) = transcript(vec![
            item(
                ItemKind::ToolCall,
                Item::ToolCall {
                    call_id: "c1".into(),
                    name: "bash".into(),
                    tool_kind: ToolKind::Execute,
                    input_raw: r#"{"command":"cargo test"}"#.into(),
                    output: vec![],
                    locations: vec![],
                    spill: None,
                    title: None,
                    server: None,
                    raw_output: None,
                },
                turn,
                true,
            ),
            item(
                ItemKind::Reasoning,
                Item::Reasoning {
                    summary: "Looking at the tests".into(),
                    raw: None,
                    provider_opaque: None,
                },
                turn,
                true,
            ),
        ]);
        let tool = model.order[0];
        let thought = model.order[1];
        let mut p = Preparer::default();
        let mut view = live_view();
        let live = state(&mut p, &model, Faces::default());
        let frame = view.render(&live, SIZE).unwrap();
        let arcs0 = orbit_prims(frame);
        assert!(
            arcs0 >= 20,
            "the running card's border orbits: {arcs0} arcs"
        );
        assert_eq!(
            sheen_groups(frame),
            2,
            "the thinking label sheens in two masked layers"
        );
        let first_arc = |frame: &Frame| {
            frame
                .list
                .primitives
                .iter()
                .find_map(|p| match p {
                    Prim::Path {
                        origin: [0., 0.],
                        points,
                        stroke,
                        fill: None,
                        ..
                    } if stroke.width == 1. => points.iter().find_map(|c| match c {
                        bbt_ui_model::draw::PathCommand::Move(p) => Some(*p),
                        _ => None,
                    }),
                    _ => None,
                })
                .unwrap()
        };
        let dot = |frame: &Frame, k: usize| -> (f32, f32) {
            let status = find(
                frame.layout.as_ref().unwrap(),
                &format!("tool/{tool}/status"),
            )
            .unwrap();
            let mut ps = vec![];
            paths(status, &mut ps);
            let LayoutKind::Path { color, .. } = &ps[k].kind else {
                unreachable!()
            };
            (ps[k].rect.y, color[3])
        };
        let head0 = first_arc(frame);
        let (y0, a0) = dot(frame, 0);
        assert!(near(a0, 0.5, 1e-3), "a dot rests at .5");
        assert_eq!(
            view.anim.next_frame(),
            Some(0.),
            "loops keep the clock awake"
        );
        // half a lap later the arc's head is on the far side of the card
        view.anim.tick(800.);
        let frame = view.render(&live, SIZE).unwrap();
        let head1 = first_arc(frame);
        let card = find(frame.layout.as_ref().unwrap(), &format!("tool/{tool}"))
            .unwrap()
            .rect;
        assert!(
            (head0[0] - head1[0]).abs() > card.w * 0.5
                || (head0[1] - head1[1]).abs() > card.h * 0.5,
            "orbit moved: {head0:?} -> {head1:?} on {card:?}"
        );
        // the first dot: phase .8 of its 1s bounce is rest; at 400ms it was up 3px at 1
        let (y8, a8) = dot(frame, 0);
        assert!(near(y8, y0, 1e-3) && near(a8, 0.5, 1e-3));
        view.anim.tick(1400.);
        let frame = view.render(&live, SIZE).unwrap();
        let (y4, a4) = dot(frame, 0);
        assert!(near(y4, y0 - 3., 1e-3), "dot up 3px at 40%: {y4} vs {y0}");
        assert!(near(a4, 1., 1e-3));
        // the think icon spins: its path differs from the settled glyph at this phase
        let icon = find(
            frame.layout.as_ref().unwrap(),
            &format!("reasoning/{thought}/toggle/icon"),
        )
        .unwrap();
        let mut ps = vec![];
        paths(icon, &mut ps);
        let spun = points(ps[0]);
        let rest: Vec<[f32; 2]> =
            bboxy450_harness::frontends::gui::face::icon_points("think", 14., 0.)
                .iter()
                .filter_map(|p| match p {
                    bbt_ui_model::draw::PathCommand::Move(p)
                    | bbt_ui_model::draw::PathCommand::Line(p) => Some(*p),
                    _ => None,
                })
                .collect();
        assert!(spun.iter().zip(&rest).any(|(a, b)| !near(a[0], b[0], 1e-3)));
        // both finish: no arcs, no sheen, no frame
        for id in [tool, thought] {
            let row = model.items.get_mut(&id).unwrap();
            row.status = Some(ItemStatus::Completed);
            row.completed = Some(Seq(2));
            row.completed_ms = Some(900);
        }
        let done = state(&mut p, &model, Faces::default());
        view.anim.tick(1500.);
        let frame = view.render(&done, SIZE).unwrap();
        assert_eq!(orbit_prims(frame), 0);
        assert_eq!(sheen_groups(frame), 0);
        // the finished rows change height (status disc, "Thought for" label): that is the
        // 200ms height tween's frame, not a loop's. Past it, nothing asks for a frame.
        view.anim.tick(1750.);
        let frame = view.render(&done, SIZE).unwrap();
        assert_eq!(orbit_prims(frame), 0);
        assert_eq!(
            view.anim.next_frame(),
            None,
            "finished cards ask for no frame"
        );
    })
    .await;
}

#[tokio::test]
async fn the_turn_receipt_rises_in_and_its_numbers_count_up() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (mut model, _) = transcript(vec![
            item(
                ItemKind::UserMessage,
                Item::UserMessage {
                    blocks: vec![InputBlock::Text { text: "go".into() }],
                },
                turn,
                false,
            ),
            item(
                ItemKind::AgentMessage,
                Item::AgentMessage {
                    markdown: "Done it.".into(),
                },
                turn,
                false,
            ),
        ]);
        let mut p = Preparer::default();
        let mut view = live_view();
        let running = state(&mut p, &model, Faces::default());
        let frame = view.render(&running, SIZE).unwrap();
        let key = format!("turn/{}", turn.0);
        assert!(find(frame.layout.as_ref().unwrap(), &key).is_none());
        model.turns.insert(
            turn,
            TurnState::Ended {
                reason: StopReason::EndTurn,
                usage: Some(Box::new(Usage {
                    input_tokens: 1200,
                    output_tokens: 300,
                    ..Default::default()
                })),
            },
        );
        let ended = state(&mut p, &model, Faces::default());
        let fact = |frame: &Frame, i: usize| {
            let n = find(frame.layout.as_ref().unwrap(), &format!("{key}/fact/{i}")).expect("fact");
            let mut rs = vec![];
            runs(n, &mut rs);
            rs.iter().map(|r| r.text.as_str()).collect::<String>()
        };
        let mut samples = vec![];
        for t in [0., 100., 200., 320., 400.] {
            view.anim.tick(t);
            let frame = view.render(&ended, SIZE).unwrap();
            let layout = frame.layout.as_ref().unwrap();
            let receipt = find(layout, &key).expect("receipt").rect;
            let rise = Ease::Out.at(t / stream_motion::RISE_MS);
            let count = Ease::Out.at(t / stream_motion::COUNT_MS);
            let opacity = filter_over(layout, &key);
            samples.push((t, receipt.y, opacity, fact(frame, 0), fact(frame, 1)));
            if rise < 1. {
                assert!(
                    near(opacity.expect("the receipt fades in"), rise, 1e-3),
                    "t={t}"
                );
            } else {
                assert_eq!(opacity, None);
            }
            assert_eq!(
                fact(frame, 0),
                count_up("1.5s", count),
                "t={t}: duration counts up"
            );
            assert_eq!(
                fact(frame, 1),
                count_up("1.2k in · 300 out", count),
                "t={t}: tokens count up"
            );
        }
        println!("receipt samples (t, y, opacity, duration, tokens): {samples:?}");
        // it rose 6px: the first frame sat 6px lower than the settled one
        assert!(
            near(samples[0].1 - samples[4].1, 6., 1e-3),
            "rise: {samples:?}"
        );
        assert_eq!(samples[0].3, "0.0s");
        assert_eq!(samples[4].3, "1.5s");
        assert_eq!(samples[4].4, "1.2k in · 300 out");
        assert_eq!(view.anim.next_frame(), None, "the receipt has settled");
        // a receipt already there when the view first looks is settled, not animated
        let mut fresh = live_view();
        let frame = fresh.render(&ended, SIZE).unwrap();
        assert_eq!(filter_over(frame.layout.as_ref().unwrap(), &key), None);
        assert_eq!(fact(frame, 1), "1.2k in · 300 out");
        assert_eq!(fresh.anim.next_frame(), None);
    })
    .await;
}

#[test]
fn count_up_scales_every_number_and_keeps_the_words() {
    assert_eq!(count_up("1.2k in · 300 out", 0.5), "0.6k in · 150 out");
    assert_eq!(count_up("3 rounds · 2 tools", 0.), "0 rounds · 0 tools");
    assert_eq!(count_up("840ms", 1.), "840ms");
    assert_eq!(count_up("$0.0042", 0.5), "$0.0021");
    assert_eq!(count_up("Done", 0.3), "Done");
}

#[test]
fn the_orbit_runs_the_border_line_clockwise_from_the_top_centre() {
    let r = Rect::new(10., 20., 200., 60.);
    let top = perimeter_point(r, 11., 0.);
    assert!(
        near(top[0], 10. + 0.5 + 199. / 2., 1e-3) && near(top[1], 20.5, 1e-3),
        "{top:?}"
    );
    let right = perimeter_point(r, 11., 0.25);
    assert!(near(right[0], 10. + 0.5 + 199., 1e-3), "{right:?}");
    let bottom = perimeter_point(r, 11., 0.5);
    assert!(
        near(bottom[1], 20. + 0.5 + 59., 1e-3) && near(bottom[0], top[0], 1e-3),
        "{bottom:?}"
    );
    let left = perimeter_point(r, 11., 0.75);
    assert!(near(left[0], 10.5, 1e-3), "{left:?}");
    // the corner arcs stay on the rounded outline, never cut across it
    for s in [0.22, 0.23, 0.24, 0.26] {
        let p = perimeter_point(r, 11., s);
        assert!(
            p[0] <= 10.5 + 199. + 1e-3 && p[1] >= 20.5 - 1e-3,
            "{s}: {p:?}"
        );
    }
    // a lap's worth of arcs: 24 segments across 37% of the perimeter, each a short stroke
    let arcs = stream_motion::orbit(r, 11., 0.3, [0., 1., 0.5, 1.], [1.; 4]);
    assert!(arcs.len() >= 20 && arcs.len() <= 24, "{}", arcs.len());
}

#[test]
fn a_chevron_turned_90_degrees_is_the_down_chevron() {
    let chev = bboxy450_harness::frontends::gui::face::icon_points("chev", 16., 0.);
    let down = bboxy450_harness::frontends::gui::face::icon_points("chevdown", 16., 0.);
    let got: Vec<[f32; 2]> = turned(&chev, 90., [8., 8.])
        .iter()
        .filter_map(|p| match p {
            bbt_ui_model::draw::PathCommand::Move(p) | bbt_ui_model::draw::PathCommand::Line(p) => {
                Some(*p)
            }
            _ => None,
        })
        .collect();
    let mut want: Vec<[f32; 2]> =
        down.iter()
            .filter_map(|p| match p {
                bbt_ui_model::draw::PathCommand::Move(p)
                | bbt_ui_model::draw::PathCommand::Line(p) => Some(*p),
                _ => None,
            })
            .collect();
    want.reverse();
    assert_eq!(got.len(), 3);
    for (g, w) in got.iter().zip(&want) {
        assert!(
            near(g[0], w[0], 1e-4) && near(g[1], w[1], 1e-4),
            "{got:?} vs {want:?}"
        );
    }
}

/// The overlays paint: an offscreen Metal frame of a live card differs from the settled
/// frame along the card's border (the orbit) and over the "Thinking" label (the sheen),
/// and nowhere else of note. `HARNESS_RENDER_EVIDENCE=dir` keeps both PNGs.
#[tokio::test]
async fn a_live_frame_paints_the_orbit_and_the_sheen_in_pixels() {
    let context = bbt_render::device::DeviceContext::new().await.unwrap();
    runtime::blocking(move || {
        let turn = TurnId::new();
        let (model, _) = transcript(vec![
            item(
                ItemKind::ToolCall,
                Item::ToolCall {
                    call_id: "c1".into(),
                    name: "bash".into(),
                    tool_kind: ToolKind::Execute,
                    input_raw: r#"{"command":"cargo test"}"#.into(),
                    output: vec![],
                    locations: vec![],
                    spill: None,
                    title: None,
                    server: None,
                    raw_output: None,
                },
                turn,
                true,
            ),
            item(
                ItemKind::Reasoning,
                Item::Reasoning {
                    summary: "Looking at the tests".into(),
                    raw: None,
                    provider_opaque: None,
                },
                turn,
                true,
            ),
        ]);
        let tool = model.order[0];
        let thought = model.order[1];
        let mut p = Preparer::default();
        let live = state(&mut p, &model, Faces::default());
        let (w, h) = (SIZE[0] as u32, SIZE[1] as u32);
        let mut renderer = bbt_render::chrome::ChromeRenderer::with_context(
            context,
            bboxy450_harness::frontends::gui::assets::icons(),
        )
        .unwrap();
        let mut settled = HarnessView::default();
        let frame = settled.render(&live, SIZE).unwrap();
        let card = find(frame.layout.as_ref().unwrap(), &format!("tool/{tool}"))
            .unwrap()
            .rect;
        let label = find(
            frame.layout.as_ref().unwrap(),
            &format!("reasoning/{thought}/toggle/label"),
        )
        .unwrap()
        .rect;
        let still = renderer.render_to_rgba(&frame.list, w, h, 1.).unwrap();
        let mut view = live_view();
        view.anim.tick(400.);
        let frame = view.render(&live, SIZE).unwrap();
        let moving = renderer.render_to_rgba(&frame.list, w, h, 1.).unwrap();
        if let Some(out) = std::env::var_os("HARNESS_RENDER_EVIDENCE") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            for (name, bytes) in [
                ("motion-settled.png", &still),
                ("motion-live-400ms.png", &moving),
            ] {
                let mut enc =
                    png::Encoder::new(std::fs::File::create(out.join(name)).unwrap(), w, h);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(bytes).unwrap();
            }
        }
        let changed = |r: Rect| -> usize {
            let mut n = 0;
            for y in r.y.max(0.) as u32..r.bottom().min(h as f32) as u32 {
                for x in r.x.max(0.) as u32..r.right().min(w as f32) as u32 {
                    let i = ((y * w + x) * 4) as usize;
                    if still[i..i + 3] != moving[i..i + 3] {
                        n += 1;
                    }
                }
            }
            n
        };
        // the border band: the card's outline, 3px in and out
        let outer = changed(Rect::new(
            card.x - 3.,
            card.y - 3.,
            card.w + 6.,
            card.h + 6.,
        ));
        let inner = changed(Rect::new(
            card.x + 3.,
            card.y + 3.,
            card.w - 6.,
            card.h - 6.,
        ));
        let border = outer - inner;
        assert!(
            border > 60,
            "the orbit must paint along the border: {border} px changed"
        );
        let sheen = changed(label);
        assert!(
            sheen > 40,
            "the sheen must paint over the label: {sheen} px changed"
        );
        println!("live pixels: border band {border}, label {sheen}, card interior {inner}");
    })
    .await;
}

/// A thought's label changes from "Thinking" to "Thought for …" when it completes; the
/// body it streamed is settled text and must not fade again. Only the stream counts.
#[tokio::test]
async fn a_completing_thought_relabels_without_refading_its_body() {
    runtime::blocking(|| {
        let turn = TurnId::new();
        let (mut model, _) = transcript(vec![item(
            ItemKind::Reasoning,
            Item::Reasoning {
                summary: "Looking at the tests".into(),
                raw: None,
                provider_opaque: None,
            },
            turn,
            true,
        )]);
        let id = model.order[0];
        let mut p = Preparer::default();
        let mut view = live_view();
        view.render(&state(&mut p, &model, Faces::default()), SIZE)
            .unwrap();
        // a chunk streams into the body
        model.items.get_mut(&id).unwrap().body = Item::Reasoning {
            summary: "Looking at the tests carefully".into(),
            raw: None,
            provider_opaque: None,
        };
        let frame = view
            .render(&state(&mut p, &model, Faces::default()), SIZE)
            .unwrap();
        assert_eq!(alpha_of(frame, "carefully"), 0.);
        assert_eq!(alpha_of(frame, "Looking"), 1.);
        view.anim.tick(200.);
        view.render(&state(&mut p, &model, Faces::default()), SIZE)
            .unwrap();
        // it completes: the label changes, the body (open while it was live) must not re-fade
        let row = model.items.get_mut(&id).unwrap();
        row.status = Some(ItemStatus::Completed);
        row.completed = Some(Seq(2));
        row.completed_ms = Some(1500);
        let mut faces = Faces::default();
        faces.insert(
            id,
            Face {
                reasoning_open: true,
                ..Default::default()
            },
        );
        view.anim.tick(210.);
        let frame = view.render(&state(&mut p, &model, faces), SIZE).unwrap();
        assert_eq!(alpha_of(frame, "Thought"), 1.);
        assert_eq!(
            alpha_of(frame, "Looking"),
            1.,
            "settled body re-faded on completion"
        );
        assert_eq!(alpha_of(frame, "carefully"), 1.);
    })
    .await;
}
