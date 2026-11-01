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
                session,
                cwd: PathBuf::from("/fixture/project"),
                profile: Some("scripted".into()),
                fork: None,
                protocol: PROTOCOL,
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
            v: PROTOCOL,
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
        self.emit(Event::TurnStarted { turn: self.turn });
    }
    fn start(&mut self, kind: ItemKind, parent: Option<ItemId>) -> ItemId {
        let item = ItemId(ulid::Ulid::from(self.next));
        self.next += 1;
        self.emit(Event::ItemStarted {
            turn: Some(self.turn),
            item,
            kind,
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
    f.item(Item::Compaction {
        summary: BlobRef::empty(),
        replaced_through: Seq(1),
    });
    f.item(Item::Usage {
        usage: Usage::default(),
    });
    f.item(Item::Notice {
        level: Level::Info,
        text: "fixture complete".into(),
    });
    f.item(Item::Subagent {
        session: SessionId(ulid::Ulid::from(9u128)),
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
#[test]
fn schema_and_golden_transcripts() {
    let update = std::env::var_os("UPDATE_GOLDENS").is_some();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema = serde_json::to_string_pretty(&schema()).unwrap() + "\n";
    if update {
        fs::write(root.join("protocol.schema.json"), &schema).unwrap();
    }
    assert_eq!(
        fs::read_to_string(root.join("protocol.schema.json")).unwrap(),
        schema
    );
    for (name, events) in fixtures() {
        check_transcript(&events).unwrap();
        let bytes = events
            .iter()
            .map(|e| serde_json::to_string(e).unwrap() + "\n")
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
                assert_eq!(serde_json::to_string(&e).unwrap(), l);
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
    value["event"] = serde_json::json!({"type":"from_the_future","data":42});
    let e: Envelope = serde_json::from_value(value).unwrap();
    assert_eq!(e.event, Event::Unknown);
    assert!(matches!(
        serde_json::from_str::<Item>(r#"{"kind":"future"}"#).unwrap(),
        Item::Unknown
    ));
}
