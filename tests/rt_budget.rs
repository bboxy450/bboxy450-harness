// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    redact::Set,
    store::Blobs,
    tools::{Context, Registry},
};
use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};
#[tokio::test]
async fn ten_thousand_file_grep_leaves_io_ticker_responsive() {
    let temp = tempfile::tempdir().unwrap();
    for n in 0..10000 {
        std::fs::write(temp.path().join(format!("f{n}")), "needle\n").unwrap();
    }
    let set = Set::default();
    let ctx = Context {
        cwd: temp.path().into(),
        blobs: Blobs {
            dir: temp.path().join("blobs"),
            redactor: set.clone(),
        },
        redactor: set,
        generation: 1,
        current_generation: Arc::new(AtomicU64::new(1)),
        cancel: tokio_util::sync::CancellationToken::new(),
    };
    let task = bbt_rt::handle().spawn(async move {
        Registry::new()
            .run("grep", serde_json::json!({"pattern":"needle"}), ctx, None)
            .await
    });
    let ticker = bbt_rt::handle().spawn(async {
        let mut late = vec![];
        for _ in 0..200 {
            let start = tokio::time::Instant::now();
            tokio::time::sleep(Duration::from_millis(1)).await;
            late.push(start.elapsed().saturating_sub(Duration::from_millis(1)));
        }
        late.sort();
        late[197]
    });
    let result = task.await.unwrap();
    assert!(!result.failed);
    let p99 = ticker.await.unwrap();
    assert!(p99 <= Duration::from_millis(5), "p99 lateness {p99:?}");
}
