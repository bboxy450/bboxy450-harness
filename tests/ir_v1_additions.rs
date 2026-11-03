// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::*;
use std::collections::BTreeMap;
fn fixture() -> Vec<Envelope> {
    let session = SessionId(ulid::Ulid::from(900u128));
    let turn = TurnId(ulid::Ulid::from(901u128));
    let call = ItemId(ulid::Ulid::from(1001u128));
    let backend = Backend {
        kind: BackendKind::Engine,
        name: "fixture-provider".into(),
        version: Some("1.0".into()),
    };
    let mut call_usage = Usage {
        input_tokens: 100,
        output_tokens: 20,
        cached_input_tokens: Some(10),
        cost_micros_usd: Some(1200),
        request_id: Some("fixture-request".into()),
        ..Default::default()
    }
    .authored(
        UsageScope::Call,
        backend.clone(),
        Some("fixture-model".into()),
    );
    call_usage.source = Some(NumberSource::Estimated);
    let mut events = vec![];
    let mut emit = |event: Event| {
        let seq = Seq(events.len() as u64 + 1);
        events.push(Envelope {
            raw: None,
            ext: BTreeMap::from([("engine".into(), serde_json::json!({"fixture":true}))]),
            native: Some(Native {
                id: format!("record-{}", seq.0),
                parent: None,
                ts_ms: Some(seq.0),
            }),
            v: PROTOCOL,
            session,
            epoch: Epoch(1),
            seq: Some(seq),
            at: seq,
            ts_ms: seq.0,
            cause: None,
            event,
        });
    };
    emit(Event::SessionStarted {
        meta: SessionMeta {
            source: Some(Source {
                harness: "fixture".into(),
                native_id: Some("native-fixture".into()),
                version: Some("1.0".into()),
                parser_revision: Some(1),
            }),
            session,
            cwd: "/fixture/project".into(),
            profile: Some("fixture".into()),
            fork: None,
            protocol: PROTOCOL,
            harness: "1.0".into(),
            created_ms: 0,
        },
    });
    emit(Event::TurnQueued { turn });
    emit(Event::TurnStarted {
        turn,
        model: Some("fixture-model".into()),
    });
    emit(Event::ContextChanged {
        mode: Some("code".into()),
        permission_mode: Some("ask".into()),
        config: Some(BTreeMap::from([(
            "theme".into(),
            serde_json::json!("dark"),
        )])),
    });
    let blob = BlobRef {
        sha256: "a".repeat(64),
        len: 4,
        mime: "audio/wav".into(),
    };
    let bodies = vec![
        Item::UserMessage {
            blocks: vec![
                InputBlock::Audio { blob: blob.clone() },
                InputBlock::Resource {
                    uri: "file:///fixture/a".into(),
                    mime: "text/plain".into(),
                    blob: Some(blob.clone()),
                },
                InputBlock::ResourceLink {
                    uri: "https://example.test/reference".into(),
                    title: Some("Reference".into()),
                },
            ],
        },
        Item::ToolCall {
            call_id: "fixture-tool".into(),
            name: "read".into(),
            title: Some("Read reference".into()),
            server: Some("fixture-mcp".into()),
            raw_output: Some(serde_json::json!({"ok":true})),
            tool_kind: ToolKind::Read,
            input_raw: "{}".into(),
            locations: vec![],
            spill: None,
            output: vec![
                OutputBlock::Audio { blob: blob.clone() },
                OutputBlock::Resource {
                    uri: "file:///fixture/b".into(),
                    mime: "text/plain".into(),
                    blob: Some(blob),
                },
                OutputBlock::Terminal {
                    terminal: "fixture-terminal".into(),
                    exit: Some(0),
                },
                OutputBlock::Diff {
                    path: "a.txt".into(),
                    unified: "-old\n+new".into(),
                    old_text: Some("old".into()),
                    new_text: Some("new".into()),
                    change: Some(ChangeKind::Modify),
                },
                OutputBlock::Citations {
                    entries: vec![Citation {
                        url: "https://example.test/source".into(),
                        title: Some("Source".into()),
                        snippet: Some("A cited fact".into()),
                    }],
                },
            ],
        },
        Item::Subagent {
            session: SessionId(ulid::Ulid::from(903u128)),
            name: Some("Research".into()),
            call: Some(call),
        },
        Item::Command {
            name: "help".into(),
            args: "usage".into(),
        },
        Item::Hook {
            event: "pre_tool".into(),
            name: "fixture-hook".into(),
            outcome: "passed".into(),
            output: Some("ok".into()),
        },
        Item::SystemMessage {
            role: SystemRole::System,
            text: "System instructions".into(),
        },
        Item::SystemMessage {
            role: SystemRole::Developer,
            text: "Developer instructions".into(),
        },
        Item::Native {
            provider: "fixture".into(),
            record_type: "unmapped".into(),
            raw: serde_json::json!({"type":"future_native","kept":true}),
            blob: None,
        },
        Item::Reasoning {
            summary: "Readable reasoning".into(),
            raw: None,
            provider_opaque: Some(
                serde_json::json!({"provider":"fixture-provider","form":"sealed","data":"opaque fixture"}),
            ),
        },
        Item::Plan {
            entries: vec![PlanEntry {
                id: Some("step-1".into()),
                text: "Check usage".into(),
                status: PlanStatus::Completed,
                priority: None,
            }],
        },
        Item::Usage {
            usage: call_usage.clone(),
        },
        Item::Usage {
            usage: Usage {
                context_used: Some(500),
                context_window: Some(1000),
                ..Default::default()
            }
            .authored(
                UsageScope::Session,
                Backend {
                    kind: BackendKind::AcpAgent,
                    name: "fixture-agent".into(),
                    version: Some("2.0".into()),
                },
                None,
            ),
        },
        Item::Usage {
            usage: Usage {
                input_tokens: 7,
                output_tokens: 3,
                cost: Some(Box::new(Cost {
                    amount_micros: 50,
                    currency: "EUR".into(),
                    source: NumberSource::Reported,
                    price_source: Some("provider".into()),
                })),
                ..Default::default()
            }
            .authored(
                UsageScope::Call,
                Backend {
                    kind: BackendKind::Import,
                    name: "codex".into(),
                    version: Some("1.0".into()),
                },
                Some("fixture-codex".into()),
            ),
        },
    ];
    let calls = bodies
        .iter()
        .filter_map(|b| match b {
            Item::Usage { usage } if usage.scope == Some(UsageScope::Call) => Some(usage.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (i, body) in bodies.into_iter().enumerate() {
        let item = ItemId(ulid::Ulid::from(1000u128 + i as u128));
        emit(Event::ItemStarted {
            turn: Some(turn),
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        });
        let status = if matches!(body, Item::Native { .. }) {
            ItemStatus::Failed {
                reason: "unmapped record retained".into(),
                code: Some("NATIVE_GAP".into()),
            }
        } else {
            ItemStatus::Completed
        };
        emit(Event::ItemCompleted { item, status, body });
    }
    emit(Event::TurnEnded {
        turn,
        reason: StopReason::EndTurn,
        usage: Usage::sum_calls(&calls).unwrap(),
    });
    emit(Event::SessionClosed);
    events
}
#[test]
fn v1_additions_frozen_output_and_faces() {
    let events = fixture();
    laws::check_authored(&events).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ir-v1/v1.1-all-items.jsonl");
    let text = events
        .iter()
        .map(|e| {
            let mut value = serde_json::to_value(e).unwrap();
            value.sort_all_objects();
            serde_json::to_string(&value).unwrap() + "\n"
        })
        .collect::<String>();
    // A newly authored corpus file is written once, never edited as a golden.
    if !path.exists() && std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::write(&path, &text).unwrap();
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
    let mut model = bboxy450_harness::frontends::gui::model::Transcript::default();
    for event in &events {
        model.apply(event);
    }
    assert_eq!(model.unknown, 0);
    assert!(model.context.as_ref().unwrap().contains("code"));
    let mut projection = bboxy450_harness::frontends::seat::LineProjection::new(
        bboxy450_harness::frontends::seat::Style {
            color: bboxy450_harness::frontends::seat::ColorMode::None,
            width: 120,
            echo: false,
        },
    );
    let seat = events
        .iter()
        .flat_map(|e| projection.render(e))
        .collect::<Vec<_>>();
    let seat = String::from_utf8(seat).unwrap();
    for text in [
        "Ran /help",
        "Hook fixture-hook",
        "Developer instructions",
        "Audio:",
        "Terminal: fixture-terminal",
        "Source:",
    ] {
        assert!(seat.contains(text), "missing {text}");
    }
    assert!(!seat.contains("unmapped") && !seat.contains("NATIVE_GAP"));
}

#[tokio::test]
async fn projected_envelopes_redact_extras_and_enforce_the_total_wire_limit() {
    use bboxy450_harness::{redact::Set, store::Store};
    let temp = tempfile::tempdir().unwrap();
    let redactor = Set::default();
    redactor.register("fixture-secret-123");
    let mut meta = match &fixture()[0].event {
        Event::SessionStarted { meta } => meta.clone(),
        _ => unreachable!(),
    };
    meta.session = SessionId::new();
    let mut store = Store::open(temp.path().into(), meta, None, redactor)
        .await
        .unwrap();
    let cause = OpId::new();
    let event = Event::ProfileChanged {
        profile: "fixture".into(),
        model: "model".into(),
    };
    let envelope = store
        .append_projected_at(
            event.clone(),
            Some(cause),
            42,
            Some(Native {
                id: "fixture-secret-123".into(),
                parent: Some("parent".into()),
                ts_ms: Some(41),
            }),
            BTreeMap::from([(
                "engine".into(),
                serde_json::json!({"nested":["fixture-secret-123"]}),
            )]),
        )
        .await
        .unwrap();
    assert_eq!((envelope.ts_ms, envelope.cause), (42, Some(cause)));
    assert_eq!(envelope.native.as_ref().unwrap().id, "<redacted>");
    assert_eq!(envelope.ext["engine"]["nested"][0], "<redacted>");
    let head = store.head();
    let rejected = store
        .append_projected_at(
            event,
            None,
            43,
            None,
            BTreeMap::from([(
                "engine".into(),
                serde_json::json!({"large":"x".repeat(MAX_ENVELOPE_BYTES)}),
            )]),
        )
        .await
        .unwrap_err();
    assert!(rejected.to_string().contains("256 KiB"));
    assert_eq!(store.head(), head);
    let blob = store
        .blobs
        .put(br#"{"unmapped":true}"#.to_vec(), "application/json")
        .await
        .unwrap();
    let item = ItemId::new();
    store
        .append(
            Event::ItemStarted {
                turn: None,
                item,
                kind: ItemKind::Native,
                parent: None,
                head: None,
            },
            None,
        )
        .await
        .unwrap();
    store
        .append(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::Native {
                    provider: "fixture".into(),
                    record_type: "unmapped".into(),
                    raw: serde_json::json!({"preview":true}),
                    blob: Some(blob.clone()),
                },
            },
            None,
        )
        .await
        .unwrap();
    let mut child_meta = match &store.events[0].event {
        Event::SessionStarted { meta } => meta.clone(),
        _ => unreachable!(),
    };
    child_meta.session = SessionId::new();
    child_meta.fork = Some((store.session, store.head()));
    let child = Store::open(temp.path().into(), child_meta, None, Default::default())
        .await
        .unwrap();
    assert_eq!(
        child.blobs.get(&blob).await.unwrap(),
        br#"{"unmapped":true}"#
    );
    assert_eq!(child.events[1].ext, envelope.ext);
    store.append(Event::SessionClosed, None).await.unwrap();
    let replay = Store::replay_closed(temp.path().into(), store.session)
        .await
        .unwrap();
    assert_eq!(replay[1], envelope);
    assert!(
        !String::from_utf8(replay[1].wire_bytes().unwrap())
            .unwrap()
            .contains("fixture-secret-123")
    );
}
