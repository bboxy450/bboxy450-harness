// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::frontends::seat::{
    ColorMode, InputAction, InputFilter, LineProjection, STATE_PROBE, StateReporter, Style,
};
use bboxy450_harness::protocol::*;
#[path = "support/quiet_replay.rs"]
mod quiet_replay;

#[test]
fn native_chunks_do_not_change_replayed_rows() {
    for count in [1, 16] {
        let with_native = quiet_replay::replay(count);
        let without_native = quiet_replay::without_native(&with_native);
        for color in [ColorMode::None, ColorMode::Ansi256, ColorMode::Truecolor] {
            let style = Style {
                color,
                width: 100,
                echo: false,
            };
            let mut before = LineProjection::new(style.clone());
            let mut after = LineProjection::new(style);
            let mut expected = vec![];
            let mut actual = vec![];
            for e in &without_native {
                expected.extend(before.render(e));
            }
            for e in &with_native {
                actual.extend(after.render(e));
                assert!(expected.starts_with(&actual), "NativeNeverDraws");
            }
            assert_eq!(
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&actual),
                "NativeReplayRowsUnchanged: N={count}, color={color:?}"
            );
        }
    }
}

#[test]
fn unstyled_items_are_quiet_notice_lines() {
    let bytes = project(Style::default(), &quiet_replay::notices());
    let text = String::from_utf8(bytes).unwrap();
    let notices: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("  · "))
        .collect();
    assert_eq!(
        notices,
        [
            "  · Ran /help",
            "  · Hook fixture-hook",
            "  · Session instructions",
            "  · Context compacted",
            "  · Unsupported item: future_renderer",
        ]
    );
    assert!(!text.contains("STAYS_IN_LOG") && !text.contains("Completed"));
}

#[test]
fn pending_native_record_does_not_hold_up_visible_output() {
    let mut events = quiet_replay::replay(1);
    events.retain(|e| {
        !matches!(
            &e.event,
            Event::ItemCompleted {
                body: Item::Native { .. },
                ..
            }
        )
    });
    quiet_replay::resequence(&mut events);
    let without = quiet_replay::without_native(&events);
    assert_eq!(
        project(Style::default(), &events),
        project(Style::default(), &without),
        "PendingNativeNeverBlocks"
    );
}

// A missing origin is a user turn in both old logs and IR 1.2. Deserialize
// fixtures so #5981 can add its optional typed field without constructor churn.
fn turn_started(turn: TurnId) -> Event {
    serde_json::from_value(serde_json::json!({"type":"turn_started","turn":turn,"model":null}))
        .unwrap()
}

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
        raw: None,
        ext: Default::default(),
        native: None,
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
        next(turn_started(turn)),
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

#[test]
fn state_reports_have_exact_transition_bytes() {
    let mut state = StateReporter::default();
    assert_eq!(state.report(), b"\x1b]7501;state=idle\x07");
    assert_eq!(
        state
            .apply(&Event::ProfileChanged {
                profile: "p".into(),
                model: "m".into()
            })
            .unwrap(),
        b"\x1b]7501;state=idle:title=cCDCtyBt\x07"
    );
    let turn = TurnId::new();
    let item = ItemId::new();
    assert_eq!(
        state.apply(&turn_started(turn)).unwrap(),
        b"\x1b]7501;state=working:title=cCDCtyBt:msg=VGhpbmtpbmc=\x07"
    );
    assert_eq!(
        state
            .apply(&Event::Status {
                phase: Phase::Approval { item }
            })
            .unwrap(),
        b"\x1b]7501;state=blocked:title=cCDCtyBt:msg=:kind=permission\x07"
    );
    let approval = Item::Approval {
        subject: ApprovalSubject {
            tool_call: ItemId::new(),
            effect: Effect::Exec,
            summary: "run?".into(),
            paths: vec![],
            command: None,
        },
        options: vec![ApprovalDecisionKind::Deny],
        decision: None,
        by: None,
    };
    assert_eq!(
        state
            .apply(&Event::ItemSnapshot {
                item,
                partial: approval.clone()
            })
            .unwrap(),
        b"\x1b]7501;state=blocked:title=cCDCtyBt:msg=cnVuPw==:kind=permission\x07"
    );
    assert_eq!(
        state
            .apply(&Event::ItemCompleted {
                item,
                status: ItemStatus::Cancelled,
                body: approval
            })
            .unwrap(),
        b"\x1b]7501;state=working:title=cCDCtyBt:msg=VGhpbmtpbmc=\x07"
    );
    assert_eq!(
        state
            .apply(&Event::TurnEnded {
                turn,
                reason: StopReason::EndTurn,
                usage: None
            })
            .unwrap(),
        b"\x1b]7501;state=idle:title=cCDCtyBt\x07"
    );
    state.apply(&turn_started(turn));
    assert_eq!(
        state
            .apply(&Event::TurnEnded {
                turn,
                reason: StopReason::Error {
                    class: ErrorClass::Config,
                    message: "bad".into()
                },
                usage: None
            })
            .unwrap(),
        b"\x1b]7501;state=error:title=cCDCtyBt:msg=YmFk\x07"
    );
    assert!(
        state.apply(&Event::Status { phase: Phase::Idle }).is_none(),
        "an idle heartbeat must not hide an error"
    );
    assert_eq!(
        state.report(),
        b"\x1b]7501;state=error:title=cCDCtyBt:msg=YmFk\x07"
    );
    assert_eq!(
        state.apply(&turn_started(turn)).unwrap(),
        b"\x1b]7501;state=working:title=cCDCtyBt:msg=VGhpbmtpbmc=\x07"
    );
    // A -> B -> A changes the title even though the state is unchanged.
    state.apply(&Event::ProfileChanged {
        profile: "q".into(),
        model: "m".into(),
    });
    assert_eq!(
        state
            .apply(&Event::ProfileChanged {
                profile: "p".into(),
                model: "m".into()
            })
            .unwrap(),
        b"\x1b]7501;state=working:title=cCDCtyBt:msg=VGhpbmtpbmc=\x07"
    );
}

fn report_field(bytes: &[u8], field: &str) -> Option<String> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let text = std::str::from_utf8(bytes).unwrap();
    text.trim_end_matches('\x07').split(':').find_map(|part| {
        part.strip_prefix(&format!("{field}="))
            .map(|value| String::from_utf8(STANDARD.decode(value).unwrap()).unwrap())
    })
}

#[tokio::test(start_paused = true)]
async fn state_heartbeat_uses_two_seconds_of_fake_time_only_while_a_turn_is_open() {
    use std::time::Duration;
    let mut state = StateReporter::default();
    tokio::time::advance(Duration::from_secs(20)).await;
    assert!(state.heartbeat().is_none());
    assert!(state.heartbeat_deadline().is_none());
    let turn = TurnId::new();
    state.apply(&turn_started(turn));
    for _ in 0..4 {
        let start = tokio::time::Instant::now();
        assert_eq!(
            state.heartbeat_deadline().unwrap() - start,
            Duration::from_secs(2)
        );
        tokio::time::advance(Duration::from_millis(1999)).await;
        assert!(state.heartbeat().is_none());
        // A phase update and a probe reply cannot reset the deadline.
        state.apply(&Event::Status {
            phase: Phase::Thinking,
        });
        state.report();
        tokio::time::advance(Duration::from_millis(1)).await;
        let bytes = state
            .heartbeat()
            .expect("silent working turn lost its heartbeat");
        assert_eq!(report_field(&bytes, "msg").as_deref(), Some("Thinking"));
        assert_eq!(tokio::time::Instant::now() - start, Duration::from_secs(2));
    }
    // A late poll emits once, then schedules from now rather than catching up.
    tokio::time::advance(Duration::from_secs(11)).await;
    assert!(state.heartbeat().is_some());
    assert!(state.heartbeat().is_none());
    let bytes = state
        .apply(&Event::TurnEnded {
            turn,
            reason: StopReason::EndTurn,
            usage: None,
        })
        .unwrap();
    assert!(bytes.starts_with(b"\x1b]7501;state=idle"));
    assert!(report_field(&bytes, "msg").is_none());
    assert!(state.heartbeat_deadline().is_none());
    tokio::time::advance(Duration::from_secs(20)).await;
    assert!(state.heartbeat().is_none());
    state.apply(&turn_started(turn));
    let bytes = state
        .apply(&Event::TurnEnded {
            turn,
            reason: StopReason::Error {
                class: ErrorClass::Config,
                message: "failed".into(),
            },
            usage: None,
        })
        .unwrap();
    assert!(bytes.starts_with(b"\x1b]7501;state=error"));
    tokio::time::advance(Duration::from_secs(20)).await;
    assert!(state.heartbeat().is_none());
    assert_eq!(
        state.report(),
        bytes,
        "idle must preserve the failed-turn detail"
    );
}

#[test]
fn state_phase_follows_live_items_and_acp_identity_without_changing_the_title() {
    let mut state = StateReporter::default();
    state.apply(&Event::ProfileChanged {
        profile: "claude".into(),
        model: "acp:claude-agent-acp · sonnet".into(),
    });
    let turn = TurnId::new();
    state.apply(&turn_started(turn));
    let title = report_field(&state.report(), "title").unwrap();
    assert_eq!(title, "claude · acp:claude-agent-acp · sonnet");
    let cases = [
        (
            serde_json::json!({"kind":"reasoning", "summary":"private", "raw":null,"provider_opaque":null}),
            "Thinking",
        ),
        (
            serde_json::json!({"kind":"agent_message", "markdown":"answer"}),
            "Writing",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"1","name":"bash","tool_kind":"execute","input_raw":"{\"command\":\"sh test.sh\"}","output":[],"locations":[],"spill":null}),
            "Run sh test.sh",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"2","name":"exec","tool_kind":"execute","input_raw":"{\"program\":\"sh\",\"args\":[\"test.sh\"]}","output":[],"locations":[],"spill":null}),
            "Run sh test.sh",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"3","name":"edit","tool_kind":"edit","input_raw":"{\"path\":\"src/main.rs\"}","output":[],"locations":[],"spill":null}),
            "Edit src/main.rs",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"write","name":"write","tool_kind":"edit","input_raw":"{\"path\":\"src/new.rs\"}","output":[],"locations":[],"spill":null}),
            "Write src/new.rs",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"search","name":"grep","tool_kind":"search","input_raw":"{\"pattern\":\"needle\",\"path\":\"src\"}","output":[],"locations":[],"spill":null}),
            "Search needle in src",
        ),
        (
            serde_json::json!({"kind":"tool_call","call_id":"4","name":"sh test.sh","tool_kind":"execute","input_raw":"","output":[],"locations":[],"spill":null}),
            "Run sh test.sh",
        ),
        (
            serde_json::json!({"kind":"terminal","terminal":"t","command":"cargo test","exit":null,"signal":null,"output":null}),
            "Run cargo test",
        ),
        (
            serde_json::json!({"kind":"file_change","path":"src/lib.rs","change":"modify","unified":""}),
            "Changed src/lib.rs",
        ),
    ];
    for (body, expected) in cases {
        let body: Item = serde_json::from_value(body).unwrap();
        let item = ItemId::new();
        state.apply(&Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        });
        state.apply(&Event::ItemSnapshot {
            item,
            partial: body.clone(),
        });
        assert_eq!(
            report_field(&state.report(), "msg").unwrap(),
            format!("acp:claude-agent-acp · {expected}")
        );
        assert_eq!(
            report_field(&state.report(), "title").as_deref(),
            Some(title.as_str())
        );
        state.apply(&Event::ItemCompleted {
            item,
            body,
            status: ItemStatus::Completed,
        });
        assert_eq!(
            report_field(&state.report(), "msg").as_deref(),
            Some("acp:claude-agent-acp · Thinking")
        );
    }
    // A -> B -> A must clear and restore the ACP prefix as well as the title.
    state.apply(&Event::ProfileChanged {
        profile: "native".into(),
        model: "m".into(),
    });
    assert_eq!(
        report_field(&state.report(), "msg").as_deref(),
        Some("Thinking")
    );
    state.apply(&Event::ProfileChanged {
        profile: "claude".into(),
        model: "acp:claude-agent-acp · sonnet".into(),
    });
    assert_eq!(
        report_field(&state.report(), "msg").as_deref(),
        Some("acp:claude-agent-acp · Thinking")
    );
}

#[tokio::test(start_paused = true)]
async fn status_phases_report_immediately_and_agent_turns_keep_the_same_heartbeat() {
    use serde_json::json;
    use std::time::Duration;
    // #5981 owns the typed origin/Quiet additions. These wire events exercise
    // both old logs and its additive shapes without requiring that parallel
    // implementation to be present: every TurnStarted/TurnEnded counts alike.
    for origin in [None, Some("user"), Some("agent")] {
        let mut state = StateReporter::default();
        let turn = TurnId::new();
        let mut wire = json!({"type":"turn_started","turn":turn,"model":null});
        if let Some(origin) = origin {
            wire["origin"] = json!({"kind":origin});
        }
        let event: Event = serde_json::from_value(wire).unwrap();
        let started = state.apply(&event).unwrap();
        assert_eq!(report_field(&started, "msg").as_deref(), Some("Thinking"));
        let deadline = state.heartbeat_deadline().unwrap();
        let tool = ItemId::new();
        let body: Item = serde_json::from_value(json!({"kind":"tool_call","call_id":"run","name":"bash","tool_kind":"execute","input_raw":"{\"command\":\"sh test.sh\"}","output":[],"locations":[],"spill":null})).unwrap();
        state.apply(&Event::ItemStarted {
            turn: Some(turn),
            item: tool,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        });
        state.apply(&Event::ItemSnapshot {
            item: tool,
            partial: body.clone(),
        });
        for (phase, expected) in [
            (Phase::Tool { item: tool }, "Run sh test.sh"),
            (Phase::Thinking, "Thinking"),
            (Phase::Streaming, "Writing"),
        ] {
            let now = tokio::time::Instant::now();
            let bytes = state
                .apply(&Event::Status { phase })
                .expect("phase change must emit immediately");
            assert_eq!(report_field(&bytes, "msg").as_deref(), Some(expected));
            assert_eq!(
                tokio::time::Instant::now(),
                now,
                "phase change waited for a tick"
            );
            assert_eq!(state.heartbeat_deadline(), Some(deadline));
        }
        // The old tool remains open. Its late snapshot must not replace Writing.
        assert!(
            state
                .apply(&Event::ItemSnapshot {
                    item: tool,
                    partial: body.clone()
                })
                .is_none()
        );
        tokio::time::advance(Duration::from_millis(1999)).await;
        assert!(state.heartbeat().is_none());
        tokio::time::advance(Duration::from_millis(1)).await;
        let bytes = state.heartbeat().expect("agent-origin turn lost its timer");
        assert_eq!(report_field(&bytes, "msg").as_deref(), Some("Writing"));
        let bytes = state
            .apply(&Event::Status {
                phase: Phase::Tool { item: tool },
            })
            .unwrap();
        assert_eq!(
            report_field(&bytes, "msg").as_deref(),
            Some("Run sh test.sh")
        );
        let approval = ItemId::new();
        let approval_body = Item::Approval {
            subject: ApprovalSubject {
                tool_call: tool,
                effect: Effect::Exec,
                summary: "Allow sh test.sh?".into(),
                paths: vec![],
                command: None,
            },
            options: vec![],
            decision: None,
            by: None,
        };
        let bytes = state
            .apply(&Event::ItemSnapshot {
                item: approval,
                partial: approval_body.clone(),
            })
            .unwrap();
        assert_eq!(
            report_field(&bytes, "msg").as_deref(),
            Some("Allow sh test.sh?")
        );
        tokio::time::advance(Duration::from_secs(2)).await;
        let bytes = state.heartbeat().unwrap();
        assert_eq!(
            report_field(&bytes, "msg").as_deref(),
            Some("Allow sh test.sh?")
        );
        state.apply(&Event::ItemCompleted {
            item: approval,
            status: ItemStatus::Completed,
            body: approval_body,
        });
        state.apply(&Event::ItemCompleted {
            item: tool,
            status: ItemStatus::Completed,
            body,
        });
        let end: Event = serde_json::from_value(json!({"type":"turn_ended","turn":turn,"reason":{"reason":"quiet","ms":200},"usage":null})).unwrap();
        let bytes = state.apply(&end).unwrap();
        assert!(bytes.starts_with(b"\x1b]7501;state=idle"));
        assert!(state.heartbeat_deadline().is_none());
        tokio::time::advance(Duration::from_secs(20)).await;
        assert!(state.heartbeat().is_none());
    }
}

#[test]
fn state_details_are_scrubbed_before_clipping_and_escape_encoding() {
    let redactor = bboxy450_harness::redact::Set::default();
    redactor.register("SECRET-SENTINEL");
    let mut state = StateReporter::new(redactor);
    let turn = TurnId::new();
    state.apply(&Event::ProfileChanged {
        profile: "p SECRET-SENTINEL".into(),
        model: "m".into(),
    });
    state.apply(&turn_started(turn));
    let item = ItemId::new();
    let subject = format!(
        "SECRET-SENTINEL\x1b]7501;state=idle\x07\n{}",
        "🦊".repeat(1000)
    );
    state.apply(&Event::ItemSnapshot {
        item,
        partial: Item::Approval {
            subject: ApprovalSubject {
                tool_call: ItemId::new(),
                effect: Effect::Exec,
                summary: subject,
                paths: vec![],
                command: None,
            },
            options: vec![],
            decision: None,
            by: None,
        },
    });
    let bytes = state.report();
    assert_eq!(bytes.iter().filter(|b| **b == 0x1b).count(), 1);
    assert_eq!(bytes.iter().filter(|b| **b == 7).count(), 1);
    let detail = report_field(&bytes, "msg").unwrap();
    assert!(detail.starts_with("<redacted>"));
    assert!(detail.len() <= 2048);
    assert!(!detail.chars().any(char::is_control));
    assert!(!detail.contains("SECRET-SENTINEL"));
    assert!(
        !report_field(&bytes, "title")
            .unwrap()
            .contains("SECRET-SENTINEL")
    );
    let bytes = state
        .apply(&Event::TurnEnded {
            turn,
            reason: StopReason::Error {
                class: ErrorClass::Config,
                message: format!("SECRET-SENTINEL {}", "🦊".repeat(1000)),
            },
            usage: None,
        })
        .unwrap();
    assert!(bytes.starts_with(b"\x1b]7501;state=error"));
    let detail = report_field(&bytes, "msg").unwrap();
    assert!(detail.starts_with("<redacted>"));
    assert!(detail.len() <= 256);
}

#[tokio::test(start_paused = true)]
async fn child_items_cannot_override_an_explicit_status_phase() {
    use std::time::Duration;
    let mut state = StateReporter::default();
    let turn = TurnId::new();
    state.apply(&turn_started(turn));
    let parent = ItemId::new();
    let parent_body: Item = serde_json::from_value(serde_json::json!({"kind":"tool_call","call_id":parent,"name":"bash","tool_kind":"execute","input_raw":"{\"command\":\"parent command\"}","output":[],"locations":[],"spill":null})).unwrap();
    state.apply(&Event::ItemStarted {
        turn: Some(turn),
        item: parent,
        kind: parent_body.kind(),
        parent: None,
        head: parent_body.head(),
    });
    state.apply(&Event::ItemSnapshot {
        item: parent,
        partial: parent_body,
    });
    for (phase, expected) in [(Phase::Thinking, "Thinking"), (Phase::Streaming, "Writing")] {
        state.apply(&Event::Status { phase });
        let child = ItemId::new();
        let body = Item::Terminal {
            terminal: "child".into(),
            command: "nested command".into(),
            exit: None,
            signal: None,
            output: None,
            live: None,
        };
        assert!(
            state
                .apply(&Event::ItemStarted {
                    turn: Some(turn),
                    item: child,
                    kind: body.kind(),
                    parent: Some(parent),
                    head: body.head(),
                })
                .is_none(),
            "a child start replaced the explicit phase"
        );
        assert!(
            state
                .apply(&Event::ItemSnapshot {
                    item: child,
                    partial: body.clone()
                })
                .is_none()
        );
        tokio::time::advance(Duration::from_secs(2)).await;
        let bytes = state.heartbeat().unwrap();
        assert_eq!(report_field(&bytes, "msg").as_deref(), Some(expected));
        assert!(
            state
                .apply(&Event::ItemCompleted {
                    item: child,
                    status: ItemStatus::Completed,
                    body
                })
                .is_none()
        );
        assert_eq!(
            report_field(&state.report(), "msg").as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn probes_are_filtered_at_every_split_without_losing_the_user_line() {
    for split in 0..=STATE_PROBE.len() {
        let mut filter = InputFilter::default();
        let mut actions = filter.feed(b"hel", false);
        actions.extend(filter.feed(&STATE_PROBE[..split], false));
        actions.extend(filter.feed(&STATE_PROBE[split..], false));
        actions.extend(filter.feed(b"lo\n", false));
        assert!(
            matches!(&actions[..], [InputAction::Probe, InputAction::Line(text)] if text == "hello"),
            "split {split}"
        );
    }
    let mut filter = InputFilter::default();
    let actions = filter.feed(b"\x1b]7501;?\x07\x1b]7501;?\x07\n", true);
    assert!(
        matches!(&actions[..], [InputAction::Probe, InputAction::Probe, InputAction::Line(text), InputAction::Echo(_)] if text.is_empty())
    );
    let mut filter = InputFilter::default();
    filter.feed(b"\x1b]7501;", false);
    assert!(
        matches!(&filter.finish()[..], [InputAction::Line(text), InputAction::Eof] if text == "\x1b]7501;")
    );
    let mut filter = InputFilter::default();
    let actions = filter.feed("hé\x7flp\n\x04".as_bytes(), true);
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, InputAction::Line(text) if text == "hlp"))
    );
    assert!(matches!(actions.last(), Some(InputAction::Eof)));
}

#[test]
fn report_text_is_bounded_base64_and_cannot_inject_controls() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let mut state = StateReporter::default();
    let report = state
        .apply(&Event::ProfileChanged {
            profile: "x\x1b]7501;state=idle\x07:".repeat(20),
            model: "🦊".repeat(200),
        })
        .unwrap();
    let report = String::from_utf8(report).unwrap();
    assert_eq!(report.matches('\x1b').count(), 1);
    let title = report
        .split(":title=")
        .nth(1)
        .unwrap()
        .trim_end_matches('\x07');
    let decoded = STANDARD.decode(title).unwrap();
    assert!(decoded.len() <= 192);
    assert!(String::from_utf8(decoded).is_ok());
}

#[test]
fn transcript_text_cannot_inject_state_reports_or_probes() {
    let events = message_events(
        "before \x1b]7501;state=idle\x07 \x1b]7501;?\x07 after",
        true,
    );
    for color in [ColorMode::None, ColorMode::Truecolor] {
        let bytes = project(
            Style {
                color,
                width: 80,
                echo: false,
            },
            &events,
        );
        assert!(
            !bytes
                .windows(b"\x1b]7501;".len())
                .any(|w| w == b"\x1b]7501;")
        );
        if color == ColorMode::None {
            assert!(!bytes.contains(&0x1b));
        } else {
            only_sgr_and_osc8(&bytes);
        }
    }
}
