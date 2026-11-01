// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    policy::{Action, Policy},
    protocol::*,
    provider::profile::Unattended,
    redact::Set,
    store::Blobs,
    tools::*,
};
use serde_json::json;
use std::sync::{Arc, atomic::AtomicU64};
fn context(temp: &tempfile::TempDir) -> Context {
    let set = Set::default();
    Context {
        cwd: temp.path().into(),
        blobs: Blobs {
            dir: temp.path().join("blobs"),
            redactor: set.clone(),
        },
        redactor: set,
        generation: 1,
        current_generation: Arc::new(AtomicU64::new(1)),
        cancel: tokio_util::sync::CancellationToken::new(),
    }
}
async fn run(reg: &Registry, name: &str, args: serde_json::Value, ctx: Context) -> ToolOutput {
    let p = Policy::allow_all(Unattended::Deny);
    let permit = p.permit(&Action {
        tool: name.into(),
        effect: reg.get(name).unwrap().effect.clone(),
        generation: ctx.generation,
    });
    reg.run(name, args, ctx, permit).await
}
#[tokio::test]
async fn exact_unique_edit_and_staged_patch() {
    let t = tempfile::tempdir().unwrap();
    let ctx = context(&t);
    let reg = Registry::new();
    std::fs::write(t.path().join("a"), "hello\nhello\n").unwrap();
    let e = run(
        &reg,
        "edit",
        json!({"path":"a","oldText":"hello","newText":"bye"}),
        ctx.clone(),
    )
    .await;
    assert!(e.failed);
    assert!(matches!(&e.output[0],OutputBlock::Error{text} if text.contains("did you mean")));
    assert_eq!(
        std::fs::read_to_string(t.path().join("a")).unwrap(),
        "hello\nhello\n"
    );
    std::fs::write(t.path().join("b"), "before").unwrap();
    let e=run(&reg,"apply_patch",json!({"edits":[{"path":"b","oldText":"before","newText":"after"},{"path":"a","oldText":"absent","newText":"x"}]}),ctx.clone()).await;
    assert!(e.failed);
    assert_eq!(
        std::fs::read_to_string(t.path().join("b")).unwrap(),
        "before"
    );
    let e = run(
        &reg,
        "edit",
        json!({"path":"b","oldText":"before","newText":"after"}),
        ctx,
    )
    .await;
    assert!(!e.failed);
    assert!(matches!(&e.changes[0],Item::FileChange{unified,..} if unified.contains("+after")));
}
#[tokio::test]
async fn argv_cap_spill_search_and_path_locks() {
    let t = tempfile::tempdir().unwrap();
    let ctx = context(&t);
    let reg = Registry::new();
    let out = run(
        &reg,
        "exec",
        json!({"program":"printf","args":["%s","$(touch BAD)"]}),
        ctx.clone(),
    )
    .await;
    assert!(!t.path().join("BAD").exists());
    assert!(matches!(&out.output[0],OutputBlock::Text{text} if text.contains("$(touch BAD)")));
    let out = run(
        &reg,
        "bash",
        json!({"command":"head -c 70000 /dev/zero | tr '\\0' x"}),
        ctx.clone(),
    )
    .await;
    let blob = out.spill.unwrap();
    assert_eq!(blob.len, 70000);
    assert_eq!(ctx.blobs.get(&blob).await.unwrap().len(), 70000);
    std::fs::write(t.path().join("a.txt"), "find me\n").unwrap();
    let out = run(
        &reg,
        "grep",
        json!({"pattern":"find","path":"a.txt"}),
        ctx.clone(),
    )
    .await;
    assert!(matches!(&out.output[0],OutputBlock::Text{text} if text.contains("find me")));
    let out = run(&reg, "glob", json!({"pattern":"*.txt"}), ctx.clone()).await;
    assert!(matches!(&out.output[0],OutputBlock::Text{text} if text.contains("a.txt")));
    let a = run(
        &reg,
        "write",
        json!({"path":"same","content":"a".repeat(10000)}),
        ctx.clone(),
    );
    let b = run(
        &reg,
        "write",
        json!({"path":"same","content":"b".repeat(10000)}),
        ctx,
    );
    let (a, b) = tokio::join!(a, b);
    assert!(!a.failed && !b.failed);
    let text = std::fs::read_to_string(t.path().join("same")).unwrap();
    assert!(text == "a".repeat(10000) || text == "b".repeat(10000));
}
