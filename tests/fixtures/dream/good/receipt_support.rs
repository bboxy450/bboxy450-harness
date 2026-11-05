// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        receipt::{Body, Run, SCHEMA},
        trust::Producer,
    },
    protocol::*,
};
use serde_json::json;
pub fn sid(n: u128) -> SessionId {
    SessionId(ulid::Ulid::from(n))
}
pub fn item(n: u128) -> ItemId {
    ItemId(ulid::Ulid::from(n))
}
pub fn push(log: &mut Vec<Envelope>, session: SessionId, event: Event) {
    let n = log.len() as u64 + 1;
    let mut v = json!({"v":{"major":1,"minor":3},"session":session,"epoch":1,"seq":n,"at":n,"ts_ms":(u128::from(session.0) as u64)*1000+n,"cause":null,"event":event});
    if n == 1 {
        v["_attribution"] = json!({"source":"https://github.com/bboxy450/bboxy450-harness","policy":"https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md"});
    }
    log.push(Envelope::from_line(&(serde_json::to_string(&v).unwrap() + "\n")).unwrap());
}
pub fn start(session: SessionId, cwd: &std::path::Path, dream: bool) -> Vec<Envelope> {
    let v = json!({"session":session,"cwd":cwd,"harness":if dream{"dream"}else{"scripted"},"profile":"scripted","created_ms":0,"fork":null,"protocol":{"major":1,"minor":3},"vcs":{"repo":"fixture.invalid/dream/repo","dirty":false},"dream":if dream{json!({"phase":"audit","trigger":{"kind":"operator","message":"private fixture"},"scope":{"level":"repo","repos":["fixture.invalid/dream/repo"]}})}else{serde_json::Value::Null}});
    let mut log = vec![];
    push(
        &mut log,
        session,
        Event::SessionStarted {
            meta: serde_json::from_value(v).unwrap(),
        },
    );
    log
}
pub fn begin(log: &mut Vec<Envelope>, session: SessionId, id: ItemId, kind: ItemKind) {
    push(
        log,
        session,
        Event::ItemStarted {
            item: id,
            kind: kind.clone(),
            turn: None,
            parent: None,
            head: if kind == ItemKind::Terminal {
                Some(ItemHead::Terminal {
                    terminal: "private-verify".into(),
                    command: "test -f fixed".into(),
                })
            } else {
                None
            },
        },
    );
}
pub fn completed(log: &mut Vec<Envelope>, session: SessionId, id: ItemId, body: Item) {
    push(
        log,
        session,
        Event::ItemCompleted {
            item: id,
            status: ItemStatus::Completed,
            body,
        },
    );
}
pub fn run_body(session: SessionId, producer: Producer) -> Body {
    Body {
        schema: SCHEMA.into(),
        kind: ReceiptKind::Run,
        outcome: ReceiptOutcome::Completed,
        harness: None,
        sink: None,
        run: Run { session },
        producer,
        coverage: None,
        ops: vec![],
        seal: None,
        cut: None,
        inventory: None,
        projection_digest: None,
        selection: None,
        sources: Some(vec![]),
        skip_reason: None,
    }
}
pub fn write_log(root: &std::path::Path, session: SessionId, log: &[Envelope]) {
    let bytes = log
        .iter()
        .flat_map(|e| e.wire_bytes().unwrap())
        .collect::<Vec<_>>();
    std::fs::write(root.join(format!("{session}.jsonl")), bytes).unwrap();
}
pub fn write_blob(root: &std::path::Path, session: SessionId, body: &Item, bytes: &[u8]) {
    let Item::Receipt {
        blob: Some(blob), ..
    } = body
    else {
        panic!("carrier");
    };
    let path = root.join(session.to_string()).join("blobs");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join(&blob.sha256), bytes).unwrap();
}
