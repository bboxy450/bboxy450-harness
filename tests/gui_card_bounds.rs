// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "embedded")]
use bboxy450_harness::{
    frontends::gui::{
        assets, face,
        view::{Metrics, context},
    },
    runtime,
};
use bbt_ui::components as c;
use bbt_ui_model::{
    draw::{Color, DrawList},
    layout::Rect,
};

fn draw(layout: Option<&c::Layout>) -> DrawList {
    let mut list = DrawList::default();
    list.rect(Rect::new(0., 0., 640., 240.), 0., Color::hex("#0b0d15"));
    if let Some(layout) = layout {
        for paint in c::paint(layout, context()) {
            match paint {
                c::Paint::Draw(p) | c::Paint::Underlay(p) => list.primitives.extend(p.primitives),
                _ => panic!("flat card witness must not silently discard a material or group"),
            }
        }
    }
    list
}
fn save(name: &str, rgba: &[u8]) {
    if let Some(root) = std::env::var_os("HARNESS_CARD_BOUNDS_CAPTURE_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        let mut encoder = png::Encoder::new(
            std::fs::File::create(root.join(format!("{name}.png"))).unwrap(),
            640,
            240,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(rgba)
            .unwrap();
    }
}

#[tokio::test]
async fn acp_controls_tool_cards_pixels_reject_unclipped_control() {
    let device = bbt_render::device::DeviceContext::new().await.unwrap();
    runtime::blocking(move || {
        let metrics = Metrics::default();
        let ink = face::ink(context());
        let mut renderer =
            bbt_render::chrome::ChromeRenderer::with_context(device, assets::icons()).unwrap();
        let baseline = renderer.render_to_rgba(&draw(None), 640, 240, 1.).unwrap();
        for width in [180., 420.] {
            // A child wider than its real production card challenges the card guard itself.
            let card = face::card(
                &face::Status::Done,
                vec![
                    face::term_line(
                        vec![(
                            format!(
                                "{}\n{}",
                                "LONG-COMMAND-".repeat(100),
                                "HEREDOC-LINE-".repeat(100)
                            ),
                            [1., 0.2, 0.2, 1.],
                            400,
                            false,
                        )],
                        12.,
                        false,
                    )
                    .into(),
                ],
                &ink,
            );
            let normal = card
                .layout(Rect::new(24., 40., width, 0.), context(), &metrics)
                .unwrap();
            let mut broken = card.clone();
            let c::Element::Box(panel) = &mut broken else {
                panic!("production card guard")
            };
            assert!(panel.clip, "production clipping guard is required");
            panel.clip = false; // Deliberately broken control, without editing production source.
            let broken = broken
                .layout(Rect::new(24., 40., width, 0.), context(), &metrics)
                .unwrap();
            assert_eq!(
                normal.rect, broken.rect,
                "the control changes clipping only"
            );
            let good = renderer
                .render_to_rgba(&draw(Some(&normal)), 640, 240, 1.)
                .unwrap();
            let bad = renderer
                .render_to_rgba(&draw(Some(&broken)), 640, 240, 1.)
                .unwrap();
            let escaped = |image: &[u8]| {
                let mut count = 0;
                for y in 40..normal.rect.bottom().ceil().min(240.) as usize {
                    for x in (normal.rect.right().ceil() as usize + 4)..630 {
                        let i = (y * 640 + x) * 4;
                        if image[i..i + 4] != baseline[i..i + 4] {
                            count += 1;
                        }
                    }
                }
                count
            };
            assert_eq!(escaped(&good), 0, "normal card paints beyond its boundary");
            assert!(
                escaped(&bad) > 100,
                "broken clipping control failed to expose overflow"
            );
            println!(
                "card width {width}: normal escaped pixels {}, broken escaped pixels {}",
                escaped(&good),
                escaped(&bad)
            );
            save(&format!("clipped-{width}"), &good);
            save(&format!("broken-{width}"), &bad);
        }
    })
    .await;
}
