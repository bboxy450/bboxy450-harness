// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::import::{Native, edge::remainder};
use bboxy450_harness::{import::import_file_with_redactor, protocol::*, redact::Set, store::Blobs};
use serde_json::Value;
use serde_json::json;
use std::path::Path;

fn read(path: &Path) -> Vec<Envelope> {
    std::fs::read_to_string(path)
        .unwrap()
        .split_inclusive('\n')
        .map(|l| Envelope::from_line(l).unwrap())
        .collect()
}
fn write(path: &Path, records: &[Value]) {
    std::fs::write(
        path,
        records.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
}
fn has_native(events: &[Envelope], raw: &Value) -> bool {
    events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Native {raw:value,..},..} if value==raw))
}

#[tokio::test]
async fn native_projectors_round_trip_unknown_records_and_known_extra_fields_after_redaction() {
    for (provider, unknown, known) in [
        (
            Native::ClaudeCode,
            json!({"type":"never_seen_claude","uuid":"future-c","future":{"payload":"needle"}}),
            json!({"type":"assistant","uuid":"known-c","parentUuid":"future-c","timestamp":"2026-11-02T10:00:00Z","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]},"future_payload":{"keep":"needle"}}),
        ),
        (
            Native::Codex,
            json!({"type":"event_msg","payload":{"type":"never_seen_codex","opaque":["needle",7]}}),
            json!({"type":"response_item","timestamp":"2026-11-02T10:00:00Z","payload":{"type":"message","id":"known-x","role":"assistant","content":[{"type":"output_text","text":"hello"}],"new_inner":false},"future_payload":{"keep":"needle"}}),
        ),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("native.jsonl");
        write(&file, &[unknown.clone(), known]);
        let redactor = Set::default();
        redactor.register("needle");
        let session = import_file_with_redactor(
            Some(provider),
            &file,
            tmp.path().join("sessions"),
            redactor.clone(),
        )
        .await
        .unwrap();
        let path = tmp.path().join(format!("sessions/{session}.jsonl"));
        let events = read(&path);
        bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
        let expected = redactor.value(unknown);
        assert!(has_native(&events, &expected));
        let kept = events
            .iter()
            .find_map(|e| e.ext.get(provider.label()))
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&kept["future_payload"]).unwrap(),
            serde_json::to_vec(&json!({"keep":"<redacted>"})).unwrap()
        );
        assert!(events.iter().any(|e| {
            e.native
                .as_ref()
                .is_some_and(|n| n.id.starts_with("known-"))
        }));
        assert!(!std::fs::read_to_string(path).unwrap().contains("needle"));
    }
}

#[tokio::test]
async fn sidechains_commands_nonmessages_and_partial_unknown_blocks_are_native() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("native.jsonl");
    let records = vec![
        json!({"type":"system","subtype":"future","value":7}),
        json!({"type":"assistant","isSidechain":true,"message":{"content":[{"type":"text","text":"side"}]}}),
        json!({"type":"user","message":{"content":"<command-name>/clear</command-name>"}}),
        json!({"type":"user","isMeta":true,"message":{"content":"meta"}}),
        json!({"type":"assistant","message":{"content":[{"type":"text","text":"known"},{"type":"future_image","data":[1,2]}]}}),
    ];
    write(&file, &records);
    let id = import_file_with_redactor(
        Some(Native::ClaudeCode),
        &file,
        tmp.path().join("sessions"),
        Set::default(),
    )
    .await
    .unwrap();
    let events = read(&tmp.path().join(format!("sessions/{id}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
    for record in records {
        assert!(has_native(&events, &record), "missing {record}");
    }
}

#[tokio::test]
async fn oversized_native_is_a_verbatim_redacted_json_blob_before_its_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("huge.jsonl");
    let raw = json!({"type":"future_huge","data":"界needle\n".repeat(15000)});
    write(&file, std::slice::from_ref(&raw));
    let set = Set::default();
    set.register("needle");
    let id = import_file_with_redactor(
        Some(Native::ClaudeCode),
        &file,
        tmp.path().join("sessions"),
        set.clone(),
    )
    .await
    .unwrap();
    let events = read(&tmp.path().join(format!("sessions/{id}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
    let (preview, blob) = events
        .iter()
        .find_map(|e| {
            if let Event::ItemCompleted {
                body:
                    Item::Native {
                        raw,
                        blob: Some(blob),
                        ..
                    },
                ..
            } = &e.event
            {
                Some((raw, blob))
            } else {
                None
            }
        })
        .unwrap();
    assert!(preview.as_str().unwrap().len() <= 4096);
    let blobs = Blobs {
        dir: tmp.path().join(format!("sessions/{id}/blobs")),
        redactor: set.clone(),
    };
    let exact = blobs.get(blob).await.unwrap();
    assert_eq!(exact, serde_json::to_vec(&set.value(raw)).unwrap());
    let scan = bboxy450_harness::frontends::ir_scan::scan_paths(
        vec![tmp.path().join(format!("sessions/{id}.jsonl"))],
        bboxy450_harness::protocol::schema_value(),
        set,
        None,
    )
    .await
    .unwrap();
    assert!(
        scan.entries
            .values()
            .any(|e| e.carrier == "native" && e.shape.iter().any(|p| p == "/data:string"))
    );
    let derived = bboxy450_harness::import::reproject::run(
        tmp.path().join(format!("sessions/{id}.jsonl")),
        tmp.path().join("derived"),
        Set::default(),
    )
    .await
    .unwrap();
    let derived_session = match &read(&derived)[0].event {
        Event::SessionStarted { meta } => meta.session,
        _ => panic!("missing derived header"),
    };
    let copied = Blobs {
        dir: tmp.path().join(format!("derived/{derived_session}/blobs")),
        redactor: Set::default(),
    };
    assert_eq!(copied.get(blob).await.unwrap(), exact);
}

#[tokio::test]
async fn reproject_promotes_native_and_ext_in_a_derived_log_without_rewriting_source() {
    let tmp = tempfile::tempdir().unwrap();
    let native = tmp.path().join("native.jsonl");
    write(&native, &[json!({"type":"future_container","value":1})]);
    let id = import_file_with_redactor(
        Some(Native::ClaudeCode),
        &native,
        tmp.path().join("sessions"),
        Set::default(),
    )
    .await
    .unwrap();
    let path = tmp.path().join(format!("sessions/{id}.jsonl"));
    let mut rows = read(&path);
    assert!(matches!(&rows[0].event, Event::SessionStarted { meta }
        if meta.source.as_ref().is_some_and(|s| s.parser_revision == Some(bboxy450_harness::import::PARSER_REVISION))));
    for row in &mut rows {
        row.raw = None;
        if let Event::ItemCompleted {
            body: Item::Native {
                raw, record_type, ..
            },
            ..
        } = &mut row.event
        {
            *raw = json!({"type":"assistant","uuid":"promoted","message":{"content":[{"type":"text","text":"now typed"}]}});
            *record_type = "assistant".into();
            row.ext.insert("codex".into(),json!({"type":"response_item","payload":{"type":"message","id":"ext-promoted","role":"assistant","content":[{"type":"output_text","text":"typed from ext"}]}}));
        }
    }
    let bytes = rows
        .iter()
        .map(|r| String::from_utf8(r.wire_bytes().unwrap()).unwrap())
        .collect::<String>();
    std::fs::write(&path, &bytes).unwrap();
    let derived = bboxy450_harness::import::reproject::run(
        path.clone(),
        tmp.path().join("derived"),
        Set::default(),
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes.as_bytes());
    let output = read(&derived);
    bboxy450_harness::protocol::laws::check_transcript(&output).unwrap();
    assert!(
        matches!(&output[0].event,Event::SessionStarted {meta} if meta.fork.is_some_and(|(source,_)|source==id))
    );
    for text in ["now typed", "typed from ext"] {
        assert!(output.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::AgentMessage {markdown},..} if markdown==text)));
    }
    let derived_bytes = std::fs::read_to_string(&derived).unwrap();
    assert!(matches!(&output[0].event, Event::SessionStarted { meta }
        if meta.source.as_ref().is_some_and(|s| s.parser_revision == Some(bboxy450_harness::import::PARSER_REVISION))));
    let repeated = bboxy450_harness::import::reproject::run(
        derived.clone(),
        tmp.path().join("derived-again"),
        Set::default(),
    )
    .await
    .unwrap();
    let repeated_output = read(&repeated);
    bboxy450_harness::protocol::laws::check_transcript(&repeated_output).unwrap();
    assert!(
        matches!(&repeated_output[0].event, Event::SessionStarted { meta }
        if meta.source.as_ref().is_some_and(|s| s.parser_revision == Some(bboxy450_harness::import::PARSER_REVISION)))
    );
    assert_eq!(std::fs::read_to_string(&derived).unwrap(), derived_bytes);
    let repeated_bytes = std::fs::read_to_string(&repeated).unwrap();
    for row in derived_bytes
        .lines()
        .skip(1)
        .take_while(|l| !l.contains("session_closed"))
    {
        assert!(repeated_bytes.contains(row));
    }
    for row in bytes
        .lines()
        .skip(1)
        .take_while(|l| !l.contains("session_closed"))
    {
        assert!(derived_bytes.contains(row));
    }
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .arg("--sessions")
        .arg(tmp.path().join("sessions"))
        .arg("reproject")
        .arg(id.to_string())
        .arg("--out")
        .arg(tmp.path().join("cli-out"))
        .env("HOME", tmp.path())
        .env_remove("BBOXY_CAPABILITY_FILE")
        .env_remove("BBOXY_CAPABILITY")
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    let emitted = std::path::PathBuf::from(String::from_utf8(child.stdout).unwrap().trim());
    assert_eq!(emitted.parent(), Some(tmp.path().join("cli-out").as_path()));
    assert!(emitted.is_file());
}

#[cfg(feature = "test-fixtures")]
#[test]
fn restored_drop_arm_is_rejected_by_the_round_trip_assertion() {
    let record = json!({"type":"never_seen","payload":{"opaque":7}});
    let mut p = bboxy450_harness::import::Projector::new(Native::ClaudeCode);
    p.restore_drop_control();
    let broken = p.project(&record);
    let verdict = std::panic::catch_unwind(|| {
        assert!(broken.iter().any(|p|matches!(&p.event,Event::ItemCompleted {body:Item::Native {raw,..},..} if raw==&record)),"edge round-trip lost native value")
    });
    assert!(verdict.is_err());
    println!("broken restored _ => {{}} arm: round-trip assertion rejected");
}

#[cfg(feature = "acp-agent")]
#[tokio::test]
async fn acp_projector_preserves_never_seen_update_meta_and_untyped_content_through_the_real_process()
 {
    use bboxy450_harness::{
        engine::Config,
        gateway::{Gateway, Host, Subscribe},
        policy::Policy,
        provider::{ScriptedProvider, profile::Unattended},
    };
    use std::{sync::Arc, time::Duration};
    let tmp = tempfile::tempdir().unwrap();
    let profiles = tmp.path().join("profiles");
    std::fs::create_dir(&profiles).unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/acp-agent/fake_agent.py");
    std::fs::write(profiles.join("edge.md"),format!("---\nname: edge\nprovider: acp-agent\nagent:\n  command: [python3, -I, {}, ir-edge]\n---\n",script.display())).unwrap();
    let session = SessionId::new();
    let set = Set::default();
    set.register("needle");
    let meta = SessionMeta {
        vcs: None,
        dream: None,
        session,
        cwd: tmp.path().into(),
        profile: Some("edge".into()),
        fork: None,
        protocol: PROTOCOL,
        harness: env!("CARGO_PKG_VERSION").into(),
        created_ms: 0,
        source: None,
    };
    let host = Host::new(Config {
        meta,
        sessions: tmp.path().join("sessions"),
        profiles,
        provider: Arc::new(ScriptedProvider::new(vec![])),
        model: "scripted".into(),
        policy: Policy::new(Unattended::Deny),
        redactor: set.clone(),
        max_rounds: 2,
        profile_error: None,
    });
    let op = |op| OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op,
    };
    let (gateway, _) = host
        .create(op(Op::StartSession {
            cwd: tmp.path().into(),
            profile: Some("edge".into()),
            fork: None,
        }))
        .await
        .unwrap();
    let mut sub = gateway
        .subscribe(Subscribe {
            session,
            after: None,
            ephemeral: true,
        })
        .unwrap();
    gateway
        .send(op(Op::UserTurn {
            blocks: vec![InputBlock::Text { text: "go".into() }],
            mode: TurnMode::Queue,
        }))
        .unwrap();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(10), sub.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(e.event, Event::TurnEnded { .. }) {
            break;
        }
    }
    gateway.send(op(Op::Close)).unwrap();
    tokio::time::timeout(Duration::from_secs(10), host.join())
        .await
        .unwrap()
        .unwrap();
    let events = read(&tmp.path().join(format!("sessions/{session}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
    let unknown = set
        .value(json!({"sessionUpdate":"never_seen_update","value":{"secret":"needle","count":7}}));
    assert!(has_native(&events, &unknown));
    assert!(events.iter().any(|e| {
        e.ext
            .get("acp")
            .is_some_and(|v| v.get("_meta") == Some(&json!({"keep":"<redacted>"})))
    }));
    assert!(events.iter().any(|e| {
        e.ext
            .get("acp")
            .is_some_and(|v| v.get("future_field") == Some(&json!({"opaque":[1,2]})))
    }));
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Native {raw,..},..} if raw.pointer("/content/type").and_then(Value::as_str)==Some("future_media"))));
    assert!(
        events
            .iter()
            .any(|e| e.ext.get("acp").is_some_and(|v| v["title"] == "changed"
                && v["kind"] == "execute"
                && v["rawOutput"] == json!({"future":"<redacted>"})))
    );
    assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::ToolCall {raw_output:Some(value),..},..} if value==&json!({"future":"<redacted>"}))));
}

#[tokio::test]
async fn codex_tool_result_keeps_unknown_fields_beside_its_consumed_content() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("codex.jsonl");
    write(
        &source,
        &[
            json!({"type":"response_item","payload":{"type":"function_call","call_id":"c","name":"shell","arguments":"{}"}}),
            json!({"type":"response_item","payload":{"type":"function_call_output","call_id":"c","output":{"content":"okay","extra":{"kept":7}}}}),
        ],
    );
    let id = import_file_with_redactor(
        Some(Native::Codex),
        &source,
        tmp.path().join("sessions"),
        Set::default(),
    )
    .await
    .unwrap();
    let log = read(&tmp.path().join(format!("sessions/{id}.jsonl")));
    bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
    assert!(log.iter().any(|e| {
        e.ext
            .get("codex")
            .is_some_and(|v| v.pointer("/payload/output/extra") == Some(&json!({"kept":7})))
    }));
}

#[test]
fn recursive_consumption_retains_native_extra_keys_and_array_positions() {
    let claude = json!({"type":"assistant","uuid":"a","parentUuid":"p","timestamp":"2026-11-02T10:00:00Z",
        "message":{"role":"assistant","content":[{"type":"text","text":"known","future":{"nested":7}},
            {"type":"new_block","opaque":[1,2]}],"new_message_key":false},"vendor":true});
    let kept = remainder(Native::ClaudeCode, &claude);
    assert_eq!(
        kept["message"]["content"][0]["future"],
        claude["message"]["content"][0]["future"]
    );
    assert_eq!(kept["message"]["content"][0]["type"], "text");
    assert_eq!(
        kept["message"]["content"][1],
        claude["message"]["content"][1]
    );
    assert_eq!(kept["message"]["new_message_key"], false);
    assert_eq!(kept["vendor"], true);
    assert!(kept.get("uuid").is_none());
    assert!(kept.get("timestamp").is_none());
    let codex = json!({"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"known","citations":[{"future_url":"https://example.invalid"}]}],"future_flag":3}});
    let kept = remainder(Native::Codex, &codex);
    assert_eq!(
        kept["payload"]["content"][0]["citations"],
        codex["payload"]["content"][0]["citations"]
    );
    assert_eq!(kept["payload"]["future_flag"], 3);
}

#[test]
fn seat_native_is_silent_and_unknown_renders_one_notice_without_raw_payload() {
    use bboxy450_harness::frontends::seat::LineProjection;
    let mut face = LineProjection::default();
    let session = SessionId::new();
    let mut seq = 0;
    let mut output = Vec::new();
    for body in [
        Item::Native {
            provider: "vendor\nname".into(),
            record_type: "novel".into(),
            raw: json!({"secret":"DO_NOT_RENDER_BLOB"}),
            blob: None,
        },
        Item::Unknown(json!({"kind":"future_renderer","payload":"DO_NOT_RENDER_BLOB"})),
    ] {
        let item = ItemId::new();
        for event in [
            Event::ItemStarted {
                turn: None,
                item,
                kind: body.kind(),
                parent: None,
                head: None,
            },
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body,
            },
        ] {
            seq += 1;
            let e = Envelope {
                raw: None,
                ext: Default::default(),
                native: None,
                v: PROTOCOL,
                session,
                epoch: Epoch(1),
                seq: Some(Seq(seq)),
                at: Seq(seq),
                ts_ms: 0,
                cause: None,
                event,
            };
            output.extend(face.render(&e));
        }
    }
    let text = String::from_utf8(output).unwrap();
    assert!(!text.contains("vendor") && !text.contains("novel"));
    assert!(text.contains("future_renderer"));
    assert!(!text.contains("DO_NOT_RENDER_BLOB"));
    assert_eq!(
        text.lines()
            .filter(|line| line.contains("vendor name") || line.contains("future_renderer"))
            .count(),
        1
    );
}
