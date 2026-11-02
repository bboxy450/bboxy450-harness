// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::frontends::seat::{ColorMode, LineProjection, Style};
use bboxy450_harness::protocol::*;

fn fixtures() -> Vec<(String, Vec<Envelope>)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![];
    for file in std::fs::read_dir(root.join("tests/fixtures/protocol")).unwrap() {
        let file = file.unwrap().path();
        let events = std::fs::read_to_string(&file)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_transcript(&events).unwrap();
        out.push((
            file.file_stem().unwrap().to_string_lossy().into_owned(),
            events,
        ));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}
fn project(style: Style, events: &[Envelope]) -> Vec<u8> {
    let mut projection = LineProjection::new(style);
    events.iter().flat_map(|e| projection.render(e)).collect()
}
fn golden(name: &str, bytes: &[u8]) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/fixtures/seat").join(name);
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
    }
    assert_eq!(
        String::from_utf8_lossy(&std::fs::read(&path).unwrap()),
        String::from_utf8_lossy(bytes),
        "{name}"
    );
}
/// ICD embed-seat §2: the only escapes are SGR (CSI … m) and OSC 8 links.
fn only_sgr_and_osc8(bytes: &[u8]) {
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let csi = regex::Regex::new(r"\x1b\[[0-9;]*[^0-9;m]").unwrap();
    assert!(csi.find(&text).is_none(), "non-SGR CSI in output");
    let mut rest = text.as_str();
    while let Some(i) = rest.find('\x1b') {
        let tail = &rest[i + 1..];
        assert!(
            tail.starts_with('[') || tail.starts_with("]8;;") || tail.starts_with('\\'),
            "escape other than SGR/OSC 8: {:?}",
            &tail[..tail.len().min(12)]
        );
        rest = tail;
    }
}

#[test]
fn plain_fixtures_are_escape_free_goldens() {
    for (name, events) in fixtures() {
        let bytes = project(Style::default(), &events);
        assert!(
            !bytes.contains(&0x1b),
            "{name}: plain output carries an escape"
        );
        golden(&format!("{name}.txt"), &bytes);
    }
}

#[test]
fn colour_fixtures_use_only_sgr_and_osc8() {
    for (name, events) in fixtures() {
        for color in [ColorMode::Truecolor, ColorMode::Ansi256] {
            let style = Style {
                color,
                width: 100,
                echo: false,
            };
            let bytes = project(style, &events);
            only_sgr_and_osc8(&bytes);
            if color == ColorMode::Truecolor {
                golden(&format!("{name}.ansi"), &bytes);
            }
        }
    }
}

fn envelope(seq: u64, ts: u64, event: Event) -> Envelope {
    Envelope {
        v: PROTOCOL,
        session: SessionId(ulid::Ulid(1)),
        epoch: Epoch(1),
        seq: Some(Seq(seq)),
        at: Seq(seq),
        ts_ms: ts,
        cause: None,
        event,
    }
}
fn message_events(markdown: &str, deltas: bool) -> Vec<Envelope> {
    let turn = TurnId(ulid::Ulid(2));
    let item = ItemId(ulid::Ulid(3));
    let mut seq = 0;
    let mut next = |event| {
        seq += 1;
        envelope(seq, seq, event)
    };
    let mut events = vec![
        next(Event::TurnStarted { turn }),
        next(Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: ItemKind::AgentMessage,
            parent: None,
            head: None,
        }),
    ];
    if deltas {
        for c in markdown.chars() {
            events.push(next(Event::ItemDelta {
                item,
                delta: Delta::Text {
                    append: c.to_string(),
                },
            }));
        }
    }
    events.push(next(Event::ItemCompleted {
        item,
        status: ItemStatus::Completed,
        body: Item::AgentMessage {
            markdown: markdown.into(),
        },
    }));
    events
}

#[test]
fn streamed_text_prints_each_prefix_once() {
    let md = "## Title\n\nSome **bold** and `code` text that runs long enough to wrap across the narrow width we give it here.\n\n- one\n- two\n\n```rust\nfn main() { let x = 1; }\n```\nEnd.";
    for color in [ColorMode::None, ColorMode::Truecolor] {
        let style = Style {
            color,
            width: 40,
            echo: false,
        };
        let streamed = project(style.clone(), &message_events(md, true));
        let whole = project(style, &message_events(md, false));
        assert_eq!(
            String::from_utf8_lossy(&streamed),
            String::from_utf8_lossy(&whole)
        );
    }
}

#[test]
fn model_text_cannot_inject_controls() {
    let hostile = "ok \x1b[2J\x1b]52;c;AAAA\x07 \x1b[1;1H done \u{9b}31m";
    for color in [ColorMode::None, ColorMode::Truecolor] {
        let style = Style {
            color,
            width: 80,
            echo: false,
        };
        let bytes = project(style, &message_events(hostile, true));
        only_sgr_and_osc8(&bytes);
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("\x1b[2J") && !text.contains("\x1b]52") && !text.contains('\u{9b}'));
    }
}

#[test]
fn rows_fit_the_width() {
    for (name, events) in fixtures() {
        for width in [40, 72, 120] {
            let style = Style {
                color: ColorMode::None,
                width,
                echo: false,
            };
            let text = String::from_utf8(project(style, &events)).unwrap();
            for line in text.lines() {
                let cols = line.chars().count();
                assert!(cols < width, "{name}@{width}: {cols} cols: {line:?}");
            }
        }
    }
}
