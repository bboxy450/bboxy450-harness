// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
// Copied from bboxy450-term lab/web/components.js SVG dictionary, using the
// bboxybook build.rs bake pattern. Compiled in; no filesystem IO at frame time.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
/// The header mark: the bboxy identity portrait (~/.agents/identity/bboxy.png), baked at 64px.
pub const MARK: &str = "bboxy-mark";
fn mark() -> Vec<bbt_render::chrome::icons::Image> {
    let bytes: &[u8] = include_bytes!("bboxy-mark.png");
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let Ok(mut reader) = decoder.read_info() else {
        return vec![];
    };
    let mut rgba = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let Ok(info) = reader.next_frame(&mut rgba) else {
        return vec![];
    };
    rgba.truncate(info.buffer_size());
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return vec![];
    }
    vec![bbt_render::chrome::icons::Image {
        name: MARK.into(),
        width: info.width,
        height: info.height,
        rgba,
    }]
}
pub fn icons() -> bbt_render::chrome::icons::Assets {
    bbt_render::chrome::icons::Assets::with_svgs(
        mark(),
        &[
            (
                "claude",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.4\" stroke-linecap=\"round\"><path d=\"M12 3v18M3 12h18M5.6 5.6l12.8 12.8M18.4 5.6 5.6 18.4\"/></svg>",
            ),
            (
                "codex",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.2\" stroke-linejoin=\"round\"><path d=\"M12 2.5 20.5 7.3v9.4L12 21.5 3.5 16.7V7.3z\"/><path d=\"M12 8v8M8.5 10l3.5 2 3.5-2\"/></svg>",
            ),
            (
                "pi",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.4\" stroke-linecap=\"round\"><path d=\"M4 7h16M8 7v11M16 7v11\"/></svg>",
            ),
            (
                "omp",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.2\"><circle cx=\"12\" cy=\"12\" r=\"8.5\"/><circle cx=\"12\" cy=\"12\" r=\"2.5\" fill=\"#ffffff\" stroke=\"none\"/></svg>",
            ),
            (
                "zsh",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.4\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M5 6l7 6-7 6M13 18h6\"/></svg>",
            ),
            (
                "gear",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\"><circle cx=\"12\" cy=\"12\" r=\"3\"/><path d=\"M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1\"/></svg>",
            ),
            (
                "bug",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><rect x=\"7\" y=\"7\" width=\"10\" height=\"13\" rx=\"5\"/><path d=\"M9.5 7.5 8 4.5M14.5 7.5 16 4.5M7 11H3.5M17 11h3.5M7 15.5H4M17 15.5h3M12 7v13\"/></svg>",
            ),
            (
                "keyx",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"6.5\" cy=\"12\" r=\"3.5\"/><path d=\"M10 12h11M17 12v3M20 12v2.5M3 20L21 4\"/></svg>",
            ),
            (
                "plug",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M9 3v5M15 3v5M6 8h12v3a6 6 0 0 1-12 0zM12 17v4\"/></svg>",
            ),
            (
                "sidebarfox",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\"><rect x=\"3\" y=\"8\" width=\"18\" height=\"12\" rx=\"2.5\"/><path d=\"M9.5 8v12\"/><path d=\"M3.2 8.5 3.8 2.2 8.6 7.6Z\" fill=\"#ffffff\" stroke=\"none\"/></svg>",
            ),
            (
                "sidebarfoxopen",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\"><rect x=\"3\" y=\"8\" width=\"18\" height=\"12\" rx=\"2.5\"/><path d=\"M3 8h6.5v12H3z\" fill=\"#ffffff\" stroke=\"none\"/><path d=\"M3.2 8.5 3.8 2.2 8.6 7.6Z\" fill=\"#ffffff\" stroke=\"none\"/></svg>",
            ),
            (
                "archive",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M3 7h18v3H3zM5 10v9h14v-9M10 14h4\"/></svg>",
            ),
            (
                "chevdown",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.4\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M6.5 9.5 12 15l5.5-5.5\"/></svg>",
            ),
            (
                "sun",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\"><circle cx=\"12\" cy=\"12\" r=\"4\"/><path d=\"M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4\"/></svg>",
            ),
            (
                "moon",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\"><path d=\"M20 14.5A8.5 8.5 0 0 1 9.5 4a7 7 0 1 0 10.5 10.5z\"/></svg>",
            ),
            (
                "muted",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M11 5 6 9H3v6h3l5 4zM16 9l5 6M21 9l-5 6\"/></svg>",
            ),
            (
                "unmuted",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13\"/></svg>",
            ),
            (
                "cloud",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M7 18a4 4 0 0 1-.6-7.95A6 6 0 0 1 18 9a4.5 4.5 0 0 1-.5 9z\"/></svg>",
            ),
            (
                "ssh",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><rect x=\"3\" y=\"4\" width=\"18\" height=\"6\" rx=\"1.5\"/><rect x=\"3\" y=\"14\" width=\"18\" height=\"6\" rx=\"1.5\"/><path d=\"M7 7h.01M7 17h.01\"/></svg>",
            ),
            (
                "key",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"8\" cy=\"14\" r=\"4\"/><path d=\"M11 11 20 2M16 6l2 2M13 9l2 2\"/></svg>",
            ),
            (
                "google",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"#ffffff\"><path d=\"M21.6 12.2c0-.7-.1-1.3-.2-1.9H12v3.7h5.4a4.6 4.6 0 0 1-2 3v2.5h3.2c1.9-1.7 3-4.3 3-7.3z\"/><path d=\"M12 22c2.7 0 5-.9 6.6-2.4l-3.2-2.5c-.9.6-2 1-3.4 1-2.6 0-4.8-1.8-5.6-4.1H3.1v2.6A10 10 0 0 0 12 22z\"/><path d=\"M6.4 14a6 6 0 0 1 0-3.9V7.5H3.1a10 10 0 0 0 0 9z\"/><path d=\"M12 6c1.5 0 2.8.5 3.8 1.5l2.8-2.8A10 10 0 0 0 3.1 7.5l3.3 2.6C7.2 7.8 9.4 6 12 6z\"/></svg>",
            ),
            (
                "roles",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M5 19h14M5.5 16 3.5 7l5 3.5L12 5l3.5 5.5 5-3.5-2 9z\"/></svg>",
            ),
            (
                "book",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 21V5M8 7h7\"/></svg>",
            ),
            (
                "bell",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15z\"/><path d=\"M10 20.5a2 2 0 0 0 4 0\"/></svg>",
            ),
            (
                "tray",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><path d=\"M4 13h4l1.5 3h5L16 13h4\"/><path d=\"M4 13 6.5 5h11L20 13v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1z\"/></svg>",
            ),
            (
                "keys",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"><rect x=\"3\" y=\"6\" width=\"18\" height=\"12\" rx=\"2\"/><path d=\"M7 10h.01M11 10h.01M15 10h.01M7.5 14h9\"/></svg>",
            ),
            (
                "search",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2.2\" stroke-linecap=\"round\"><circle cx=\"10.5\" cy=\"10.5\" r=\"6\"/><path d=\"M15.5 15.5 20 20\"/></svg>",
            ),
            (
                "copy",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linejoin=\"round\"><rect x=\"9\" y=\"9\" width=\"11\" height=\"11\" rx=\"2\"/><path d=\"M5 15V6a1 1 0 0 1 1-1h9\"/></svg>",
            ),
            (
                "link",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\"><path d=\"M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1.5 1.5M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1.5-1.5\"/></svg>",
            ),
            (
                "open",
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#ffffff\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M14 4h6v6M20 4l-9 9M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5\"/></svg>",
            ),
        ],
    )
}
