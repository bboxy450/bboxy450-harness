// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::{laws::check_transcript, *};
use std::{fs, path::PathBuf};
struct Fixture {
    events: Vec<Envelope>,
    head: u64,
    epoch: Epoch,
    turn: TurnId,
    next: u128,
}
impl Fixture {
    fn new() -> Self {
        let session = SessionId(ulid::Ulid::from(1u128));
        let mut f = Self {
            events: vec![],
            head: 0,
            epoch: Epoch(1),
            turn: TurnId(ulid::Ulid::from(2u128)),
            next: 100,
        };
        f.emit(Event::SessionStarted {
            meta: SessionMeta {
                source: None,
                vcs: None,
                dream: None,
                session,
                cwd: PathBuf::from("/fixture/project"),
                profile: Some("scripted".into()),
                fork: None,
                protocol: Version { major: 1, minor: 0 },
                harness: "0.1.0".into(),
                created_ms: 0,
            },
        });
        f
    }
    fn emit(&mut self, event: Event) {
        if let Event::ItemCompleted { item, body, .. } = &event {
            for e in &mut self.events {
                if let Event::ItemStarted { item: id, head, .. } = &mut e.event
                    && id == item
                {
                    *head = body.head();
                }
            }
        }
        let seq = if event.is_durable() {
            self.head += 1;
            Some(Seq(self.head))
        } else {
            None
        };
        self.events.push(Envelope {
            raw: None,
            ext: Default::default(),
            native: None,
            v: Version { major: 1, minor: 0 },
            session: SessionId(ulid::Ulid::from(1u128)),
            epoch: self.epoch,
            seq,
            at: Seq(self.head),
            ts_ms: self.events.len() as u64,
            cause: None,
            event,
        });
    }
    fn start_turn(&mut self) {
        self.emit(Event::TurnQueued { turn: self.turn });
        self.emit(Event::TurnStarted {
            turn: self.turn,
            model: None,
            origin: None,
        });
    }
    fn start(&mut self, kind: ItemKind, parent: Option<ItemId>) -> ItemId {
        let item = ItemId(ulid::Ulid::from(self.next));
        self.next += 1;
        self.emit(Event::ItemStarted {
            turn: Some(self.turn),
            item,
            kind: kind.clone(),
            parent,
            head: Item::interrupted(kind, item).head(),
        });
        item
    }
    fn complete(&mut self, id: ItemId, body: Item) {
        self.emit(Event::ItemCompleted {
            item: id,
            status: ItemStatus::Completed,
            body,
        });
    }
    fn item(&mut self, body: Item) {
        let id = self.start(body.kind(), None);
        self.complete(id, body);
    }
    fn idle_item(&mut self, body: Item) {
        let item = ItemId(ulid::Ulid::from(self.next));
        self.next += 1;
        self.emit(Event::ItemStarted {
            turn: None,
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        });
        self.complete(item, body);
    }
    fn end(&mut self, reason: StopReason) {
        self.emit(Event::TurnEnded {
            turn: self.turn,
            reason,
            usage: Some(Usage {
                input_tokens: 42,
                output_tokens: 21,
                ..Usage::default()
            }),
        });
    }
}
fn tool(id: &str, name: &str, kind: ToolKind) -> Item {
    Item::ToolCall {
        title: None,
        server: None,
        raw_output: None,
        call_id: id.into(),
        name: name.into(),
        tool_kind: kind,
        input_raw: "{\"path\":\"hello.txt\"}".into(),
        output: vec![OutputBlock::Text {
            text: "hello\n".into(),
        }],
        locations: vec![Location {
            path: "hello.txt".into(),
            line: Some(1),
            end_line: Some(1),
        }],
        spill: None,
    }
}
fn fixtures() -> Vec<(&'static str, Vec<Envelope>)> {
    let mut f = Fixture::new();
    f.start_turn();
    f.item(Item::UserMessage {
        blocks: vec![InputBlock::Text {
            text: "Read two files and update hello.txt".into(),
        }],
    });
    let id = f.start(ItemKind::AgentMessage, None);
    f.emit(Event::ItemDelta {
        item: id,
        delta: Delta::Text {
            append: "I’ll read ".into(),
        },
    });
    f.emit(Event::ItemSnapshot {
        item: id,
        partial: Item::AgentMessage {
            markdown: "I’ll read ".into(),
        },
    });
    f.emit(Event::ItemDelta {
        item: id,
        delta: Delta::Text {
            append: "both files.".into(),
        },
    });
    f.complete(
        id,
        Item::AgentMessage {
            markdown: "I’ll read both files.".into(),
        },
    );
    let id = f.start(ItemKind::Reasoning, None);
    f.emit(Event::ItemDelta {
        item: id,
        delta: Delta::Reasoning {
            append: "Compare the current contents.".into(),
        },
    });
    f.complete(
        id,
        Item::Reasoning {
            summary: "Compare the current contents.".into(),
            raw: None,
            provider_opaque: Some(
                serde_json::json!({"id":"opaque-1","type":"reasoning.encrypted"}),
            ),
        },
    );
    let a = f.start(ItemKind::ToolCall, None);
    let b = f.start(ItemKind::ToolCall, None);
    f.complete(b, tool("read-b", "read", ToolKind::Read));
    f.complete(a, tool("read-a", "read", ToolKind::Read));
    f.item(Item::Plan {
        entries: vec![PlanEntry {
            id: None,
            text: "Update greeting".into(),
            status: PlanStatus::InProgress,
            priority: Some(Priority::High),
        }],
    });
    let t = f.start(ItemKind::ToolCall, None);
    let approval = f.start(ItemKind::Approval, Some(t));
    f.complete(
        approval,
        Item::Approval {
            subject: ApprovalSubject {
                tool_call: t,
                effect: Effect::Write,
                summary: "Edit hello.txt".into(),
                paths: vec!["hello.txt".into()],
                command: None,
            },
            options: vec![ApprovalDecisionKind::Allow, ApprovalDecisionKind::Deny],
            decision: Some(ApprovalDecision::Allow),
            by: Some(Resolver::Rule {
                id: "fixture-rule".into(),
            }),
        },
    );
    let diff = f.start(ItemKind::FileChange, Some(t));
    f.complete(
        diff,
        Item::FileChange {
            path: "hello.txt".into(),
            change: ChangeKind::Modify,
            unified: "--- a/hello.txt\n+++ b/hello.txt\n@@ -1 +1 @@\n-hello\n+hello world\n".into(),
        },
    );
    f.complete(t, tool("edit-1", "edit", ToolKind::Edit));
    let t = f.start(ItemKind::ToolCall, None);
    let term = f.start(ItemKind::Terminal, Some(t));
    f.emit(Event::ItemDelta {
        item: term,
        delta: Delta::Output {
            stream: Stream::Stdout,
            append: "ok\n".into(),
        },
    });
    f.complete(
        term,
        Item::Terminal {
            terminal: "fixture-terminal".into(),
            command: "printf ok".into(),
            exit: Some(0),
            signal: None,
            output: Some(BlobRef::empty()),
            live: None,
        },
    );
    f.complete(t, tool("bash-1", "bash", ToolKind::Execute));
    f.item(Item::TurnDiff {
        unified: "--- a/hello.txt\n+++ b/hello.txt\n".into(),
    });
    f.item(Item::Image {
        mime: "image/png".into(),
        blob: BlobRef::empty(),
    });
    f.item(Item::Usage {
        usage: Usage::default(),
    });
    f.item(Item::Notice {
        level: Level::Info,
        text: "fixture complete".into(),
    });
    f.item(Item::Subagent {
        name: None,
        call: None,
        session: SessionId(ulid::Ulid::from(9u128)),
    });
    f.end(StopReason::EndTurn);
    let first = Seq(f.head);
    f.idle_item(Item::Compaction {
        summary: BlobRef::empty(),
        replaced_through: first,
    });
    f.turn = TurnId(ulid::Ulid::from(3u128));
    f.start_turn();
    f.item(Item::UserMessage {
        blocks: vec![InputBlock::Text {
            text: "Read two files again.".into(),
        }],
    });
    f.item(Item::AgentMessage {
        markdown: "I’ll read both files. Continuing.".into(),
    });
    f.end(StopReason::EndTurn);
    let second = Seq(f.head);
    f.turn = TurnId(ulid::Ulid::from(4u128));
    f.start_turn();
    f.item(Item::UserMessage {
        blocks: vec![InputBlock::Text {
            text: "Read two files to finish.".into(),
        }],
    });
    f.item(Item::ContextTrim {
        clipped_through: second,
    });
    f.item(Item::Compaction {
        summary: BlobRef::empty(),
        replaced_through: second,
    });
    f.end(StopReason::EndTurn);
    let all = f.events.clone();
    f.emit(Event::Lagged {
        resume_after: Seq(f.head),
    });
    let lag = f.events.clone();
    f.events.pop();
    f.epoch = Epoch(2);
    f.emit(Event::Reset {
        epoch: Epoch(2),
        head: Seq(f.head),
    });
    let reset = f.events;
    let mut c = Fixture::new();
    c.start_turn();
    let t = c.start(ItemKind::ToolCall, None);
    c.emit(Event::ItemDelta {
        item: t,
        delta: Delta::ToolInput {
            append: "{\"command\":\"sleep 600\"}".into(),
        },
    });
    c.emit(Event::ItemCompleted {
        item: t,
        status: ItemStatus::Cancelled,
        body: tool("cancelled-call", "bash", ToolKind::Execute),
    });
    c.end(StopReason::Cancelled);
    let mut r = Fixture::new();
    r.start_turn();
    let t = r.start(ItemKind::ToolCall, None);
    r.epoch = Epoch(2);
    r.emit(Event::ItemCompleted {
        item: t,
        status: ItemStatus::Interrupted,
        body: Item::interrupted(ItemKind::ToolCall, t),
    });
    r.end(StopReason::Interrupted);
    r.emit(Event::SessionRecovered {
        prior_epoch: Epoch(1),
        closed_items: 1,
        closed_turns: 1,
    });
    vec![
        ("all-items", all),
        ("interrupt-mid-tool", c.events),
        ("recovery", r.events),
        ("lagged", lag),
        ("reset", reset),
    ]
}
// ACP enables serde_json/preserve_order upstream. Normalize snapshot values
// explicitly so neither schema nor nested opaque objects depend on that feature.
fn canonical_value(value: &impl serde::Serialize) -> serde_json::Value {
    let mut value = serde_json::to_value(value).unwrap();
    value.sort_all_objects();
    value
}

fn canonical_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(&canonical_value(value)).unwrap()
}

#[test]
fn canonical_json_sorts_nested_objects_in_arrays() {
    let value: serde_json::Value = serde_json::from_str(r#"{"z":[{"b":1,"a":0}],"a":1}"#).unwrap();
    assert_eq!(canonical_json(&value), r#"{"a":1,"z":[{"a":0,"b":1}]}"#);
}

// Either ACP feature unifies serde_json's preserve_order feature.
#[cfg(not(any(feature = "acp", feature = "acp-agent")))]
#[test]
fn default_json_maps_iterate_sorted() {
    let mut map = serde_json::Map::new();
    map.insert("z".into(), serde_json::Value::Null);
    map.insert("a".into(), serde_json::Value::Null);
    assert_eq!(
        map.keys().map(String::as_str).collect::<Vec<_>>(),
        ["a", "z"]
    );
}

#[test]
fn schema_and_golden_transcripts() {
    let update = std::env::var_os("UPDATE_GOLDENS").is_some();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema = serde_json::to_string_pretty(&schema_value()).unwrap() + "\n";
    if update {
        fs::write(root.join("protocol.schema.json"), &schema).unwrap();
    }
    assert_eq!(
        fs::read_to_string(root.join("protocol.schema.json")).unwrap(),
        schema
    );
    for (name, events) in fixtures() {
        laws::check_authored(&events).unwrap();
        let bytes = events
            .iter()
            .map(|e| canonical_json(e) + "\n")
            .collect::<String>();
        let path = root.join(format!("tests/fixtures/protocol/{name}.jsonl"));
        if update {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &bytes).unwrap();
        }
        assert_eq!(fs::read_to_string(path).unwrap(), bytes);
    }
    let mut count = 0;
    for path in fs::read_dir(root.join("tests/fixtures/protocol")).unwrap() {
        let path = path.unwrap().path();
        if path.extension().is_none_or(|x| x != "jsonl") {
            continue;
        }
        let bytes = fs::read_to_string(path).unwrap();
        let events = bytes
            .lines()
            .map(|l| {
                let e: Envelope = serde_json::from_str(l).unwrap();
                assert_eq!(canonical_json(&e), l);
                e
            })
            .collect::<Vec<_>>();
        check_transcript(&events).unwrap();
        count += 1;
    }
    assert_eq!(count, 5);
}
#[test]
fn future_variants_and_fields() {
    let mut value = serde_json::to_value(&fixtures()[0].1[0]).unwrap();
    value["future_field"] = true.into();
    let raw = serde_json::json!({"type":"from_the_future","data":{"retained":[42]}});
    value["event"] = raw.clone();
    let e: Envelope = serde_json::from_value(value).unwrap();
    assert!(matches!(&e.event,Event::Unknown(v) if v==&raw));
    assert_eq!(serde_json::to_value(&e.event).unwrap(), raw);
    macro_rules! retained {
        ($ty:ty,$raw:tt) => {{
            let raw = serde_json::json!($raw);
            let value: $ty = serde_json::from_value(raw.clone()).unwrap();
            assert_eq!(serde_json::to_value(value).unwrap(), raw, stringify!($ty));
        }};
    }
    retained!(Op,{"type":"future_op","payload":42});
    retained!(InputBlock,{"type":"future_input","payload":42});
    retained!(ApprovalDecision,{"decision":"future_decision","payload":42});
    retained!(ItemStatus,{"status":"future_status","payload":42});
    retained!(TurnOrigin,{"kind":"future_origin","payload":42});
    retained!(StopReason,{"reason":"future_reason","payload":42});
    retained!(Delta,{"type":"future_delta","payload":42});
    retained!(Item,{"kind":"future_item","payload":42});
    retained!(ItemHead,{"kind":"future_head","payload":42});
    retained!(OutputBlock,{"type":"future_output","payload":42});
    retained!(ChangeKind,{"op":"future_change","payload":42});
    retained!(Resolver,{"by":"future_resolver","payload":42});
    retained!(Phase,{"phase":"future_phase","payload":42});
    retained!(Rejection,{"reason":"future_rejection","payload":42});
    retained!(UsageScope, "future_scope");
    retained!(SystemRole, "future_role");
    retained!(BackendKind, "future_backend");
    retained!(NumberSource, "future_source");
    retained!(TurnMode, "future_mode");
    retained!(ToolKind, "future_tool");
    retained!(ItemKind, "future_item_kind");
    retained!(Stream, "future_stream");
    retained!(PlanStatus, "future_plan");
    retained!(Priority, "future_priority");
    retained!(Effect, "future_effect");
    retained!(ApprovalDecisionKind, "future_option");
    retained!(Level, "future_level");
    retained!(ErrorClass, "future_error");
    let raw = serde_json::json!({"kind":"future_item","payload":42});
    let mut item: Item = serde_json::from_value(raw.clone()).unwrap();
    assert!(item.kind().is_unknown());
    item.apply(&Delta::Text {
        append: "ignored".into(),
    });
    assert_eq!(serde_json::to_value(item).unwrap(), raw);
}

#[test]
fn recovery_does_not_author_an_unknown_body() {
    let body = Item::interrupted(
        ItemKind::Unknown(serde_json::json!("future_item_kind")),
        ItemId::new(),
    );
    assert!(matches!(
        body,
        Item::Notice {
            level: Level::Warn,
            ..
        }
    ));
}

#[test]
fn old_turn_starts_keep_an_absent_origin_and_new_origins_roundtrip() {
    let turn = TurnId::new();
    let legacy = serde_json::json!({"type":"turn_started","turn":turn});
    let event: Event = serde_json::from_value(legacy.clone()).unwrap();
    assert!(matches!(event, Event::TurnStarted { origin: None, .. }));
    assert_eq!(serde_json::to_value(event).unwrap(), legacy);
    for origin in [TurnOrigin::User, TurnOrigin::Agent] {
        let event = Event::TurnStarted {
            turn,
            model: None,
            origin: Some(origin),
        };
        assert_eq!(
            serde_json::from_value::<Event>(serde_json::to_value(&event).unwrap()).unwrap(),
            event
        );
    }
    assert_eq!(
        serde_json::from_value::<StopReason>(serde_json::json!({"reason":"quiet"})).unwrap(),
        StopReason::Quiet { ms: 0 }
    );
    let reason = StopReason::Quiet { ms: 30000 };
    assert_eq!(
        serde_json::to_value(&reason).unwrap(),
        serde_json::json!({"reason":"quiet","ms":30000})
    );
}

#[test]
fn dream_enums_roundtrip_every_known_value_and_unknown_payload() {
    fn roundtrip<T>(values: &[serde_json::Value], is_unknown: fn(&T) -> bool)
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        for raw in values {
            let parsed: T = serde_json::from_value(raw.clone()).unwrap();
            assert!(!is_unknown(&parsed), "known value parsed as Unknown: {raw}");
        }
        for raw in values.iter().cloned().chain([
            serde_json::json!("future_value"),
            serde_json::json!({"kind":"future_value","payload":{"nested":[42]}}),
        ]) {
            let parsed: T = serde_json::from_value(raw.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), raw);
        }
    }
    macro_rules! strings {
        ($ty:ty, $($value:literal),+ $(,)?) => {
            roundtrip::<$ty>(&[$(serde_json::json!($value)),+], <$ty>::is_unknown);
        };
    }
    strings!(
        DreamPhase,
        "replay",
        "synthesis",
        "trial",
        "lower",
        "flush",
        "audit"
    );
    strings!(ScopeLevel, "repo", "system");
    strings!(MemoryOp, "pending", "created", "revised");
    strings!(
        EntryState,
        "proposed",
        "tested",
        "adopted",
        "rejected",
        "superseded",
        "expired",
        "retired"
    );
    strings!(Arm, "with", "without");
    strings!(Verdict, "pass", "fail", "inconclusive");
    strings!(
        SinkKind,
        "claude_memory",
        "codex_extension",
        "grok_inbox",
        "engine_context",
        "acp_context",
        "proposal"
    );
    strings!(RecallBasis, "reported", "inferred", "cited");
    let session = SessionId::new();
    roundtrip::<Trigger>(
        &[
            serde_json::json!({"kind":"session_closed","session":session}),
            serde_json::json!({"kind":"quota","provider":"fixture","resets_at":42}),
            serde_json::json!({"kind":"operator","message":null}),
            serde_json::json!({"kind":"dream","entry":"sha256:fixture"}),
            serde_json::json!({"kind":"schedule","name":"night"}),
        ],
        Trigger::is_unknown,
    );
    roundtrip::<EvidenceRef>(
        &[
            serde_json::json!({"kind":"event","session":session,"seq":42,"digest":"sha256:fixture"}),
            serde_json::json!({"kind":"record","root":"repo","id":"record","path":"tasks/fixture.md","commit":"abc123","digest":"sha256:fixture"}),
        ],
        EvidenceRef::is_unknown,
    );
}

#[test]
fn dream_recovery_keeps_known_kinds_and_never_authors_unknown() {
    for kind in [
        ItemKind::MemoryWrite,
        ItemKind::MemoryState,
        ItemKind::Trial,
        ItemKind::Adoption,
        ItemKind::Recall,
    ] {
        let id = ItemId::new();
        let body = Item::interrupted(kind.clone(), id);
        assert_eq!(body.kind(), kind);
        let mut f = Fixture::new();
        f.emit(Event::ItemStarted {
            turn: None,
            item: id,
            kind,
            parent: None,
            head: None,
        });
        f.emit(Event::ItemCompleted {
            item: id,
            body,
            status: ItemStatus::Interrupted,
        });
        laws::check_authored(&f.events).unwrap();
    }
}

#[test]
fn dream_additions_default_every_field_without_minting_source_identity() {
    let schema = schema_value();
    for name in [
        "Vcs",
        "DreamRun",
        "Scope",
        "RecordPin",
        "TrialEpisode",
        "TrialRun",
        "SinkRef",
    ] {
        assert!(
            schema["$defs"][name]
                .get("required")
                .is_none_or(|v| v.as_array().unwrap().is_empty()),
            "{name} requires a new field"
        );
    }
    for name in ["Trigger", "EvidenceRef"] {
        for variant in schema["$defs"][name]["anyOf"].as_array().unwrap() {
            if let Some(required) = variant.get("required") {
                assert_eq!(required, &serde_json::json!(["kind"]), "{name}");
            }
        }
    }
    let dream: DreamRun = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(
        dream.phase.is_unknown() && dream.trigger.is_unknown() && dream.scope.level.is_unknown()
    );
    let missing = serde_json::json!({"kind":"event"});
    let a: EvidenceRef = serde_json::from_value(missing.clone()).unwrap();
    let b: EvidenceRef = serde_json::from_value(missing).unwrap();
    assert_eq!(a, b);
    assert!(
        matches!(a, EvidenceRef::Event { session, seq: Seq(0), .. } if session.0 == ulid::Ulid::from(0u128))
    );
    let run: TrialRun = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(run.arm.is_unknown());
    assert_eq!(run.session.0, ulid::Ulid::from(0u128));
    let pin: RecordPin = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(pin.commit.is_none() && pin.id.is_none());
}
