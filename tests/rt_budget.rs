// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    policy::{Action, Policy},
    provider::profile::Unattended,
    redact::Set,
    runtime::{self, BlockingWork},
    store::{Blobs, Store},
    tools::{Context, Registry},
};
use std::{
    sync::{Arc, Barrier, atomic::AtomicU64},
    time::Duration,
};
mod support;

fn fixture() -> (tempfile::TempDir, Context) {
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
    (temp, ctx)
}

async fn greps(ctx: Context) {
    let registry = Registry::new();
    let tasks = (0..16)
        .map(|_| {
            let (registry, ctx) = (registry.clone(), ctx.clone());
            bbt_rt::handle().spawn(async move {
                registry
                    .run("grep", serde_json::json!({"pattern":"absent"}), ctx, None)
                    .await
            })
        })
        .collect::<Vec<_>>();
    for task in tasks {
        assert!(!task.await.unwrap().failed);
    }
}

#[tokio::test]
async fn expensive_entries_hold_four_permits_off_the_io_workers() {
    let (temp, ctx) = fixture();
    let before = runtime::blocking_stats();
    // Four real grep bodies meet at each wave. This proves saturation without
    // assuming how fast a loaded machine schedules the blocking pool.
    let wave = Arc::new(Barrier::new(4));
    let observation = runtime::observe_blocking(move |work| {
        if work == BlockingWork::Grep {
            wave.wait();
        }
    });
    greps(ctx.clone()).await;
    drop(observation);
    let registry = Registry::new();
    for (name, args) in [
        ("glob", serde_json::json!({"pattern":"f*"})),
        ("read", serde_json::json!({"path":"."})),
        (
            "edit",
            serde_json::json!({"path":"f0","oldText":"needle","newText":"changed"}),
        ),
    ] {
        let permit = Policy::allow_all(Unattended::Deny).permit(&Action {
            tool: name.into(),
            effect: registry.get(name).unwrap().effect.clone(),
            generation: 1,
        });
        assert!(!registry.run(name, args, ctx.clone(), permit).await.failed);
    }
    let store = Store::open(
        temp.path().join("sessions"),
        support::meta(temp.path().into()),
        None,
        Set::default(),
    )
    .await
    .unwrap();
    store.sync().await.unwrap();
    let after = runtime::blocking_stats();
    assert_eq!(after.max_held, 4);
    assert_eq!(
        after.calls(BlockingWork::Grep) - before.calls(BlockingWork::Grep),
        16
    );
    assert_eq!(
        after.calls(BlockingWork::Walk) - before.calls(BlockingWork::Walk),
        17
    );
    for work in [
        BlockingWork::Glob,
        BlockingWork::DirectoryRead,
        BlockingWork::Diff,
        BlockingWork::Fsync,
    ] {
        assert!(
            after.calls(work) > before.calls(work),
            "missing entry hook: {work:?}"
        );
    }
    // The lease check is also an executable negative witness on an IO worker.
    assert!(
        bbt_rt::handle()
            .spawn(async { runtime::assert_off_rt_worker() })
            .await
            .unwrap_err()
            .is_panic()
    );
}

#[tokio::test]
#[ignore = "wall-clock measurement: make bench-budgets"]
async fn ten_thousand_file_grep_leaves_io_ticker_responsive() {
    let (_temp, ctx) = fixture();
    let task = bbt_rt::handle().spawn(greps(ctx));
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
    task.await.unwrap();
    let p99 = ticker.await.unwrap();
    println!("grep ticker p99 lateness: {p99:?}; allocation: 5 ms");
    assert!(p99 <= Duration::from_millis(5), "p99 lateness {p99:?}");
}
