// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
mod support;
use bboxy450_harness::{
    frontends::usage,
    protocol::*,
    store::{Store, log_path},
};
use std::path::Path;
fn call(input: u64, output: u64) -> Usage {
    Usage {
        source: Some(NumberSource::Reported),
        input_tokens: input,
        output_tokens: output,
        cost_micros_usd: Some(100),
        ..Default::default()
    }
    .authored(
        UsageScope::Call,
        Backend {
            kind: BackendKind::Engine,
            name: "fixture".into(),
            version: Some("1".into()),
        },
        Some("test-model".into()),
    )
}
async fn append(store: &mut Store, usage: Usage) {
    let item = ItemId::new();
    store
        .append(
            Event::ItemStarted {
                turn: None,
                item,
                kind: ItemKind::Usage,
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
                body: Item::Usage { usage },
            },
            None,
        )
        .await
        .unwrap();
}
#[test]
fn frozen_corpus_report_matches_expected_and_cli() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("tests/fixtures/ir-v1");
    let report = usage::report_paths(std::slice::from_ref(&dir)).unwrap();
    let value = serde_json::to_value(&report).unwrap();
    let golden = root.join("tests/fixtures/usage/ir-v1-report.json");
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
        std::fs::write(
            &golden,
            serde_json::to_string_pretty(&value).unwrap() + "\n",
        )
        .unwrap();
    }
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(golden).unwrap()).unwrap();
    assert_eq!(value, expected);
    assert!(report.rows.iter().any(|r| r.source.is_unknown()));
    assert!(
        report
            .rows
            .iter()
            .any(|r| r.source == NumberSource::Estimated && r.numbers["input_tokens"] == 100)
    );
    assert!(
        report
            .rows
            .iter()
            .any(|r| r.currency.as_deref() == Some("EUR") && r.numbers["cost_micros"] == 50)
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args(["usage", "--json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        expected
    );
}
#[tokio::test]
async fn fork_prefix_is_counted_once_and_dropping_usage_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("sessions");
    let mut parent = Store::open(
        root.clone(),
        support::meta(temp.path().into()),
        None,
        Default::default(),
    )
    .await
    .unwrap();
    // A cumulative gauge below the fork cannot be added to the detailed calls,
    // even when only the child log is supplied and inherited lines name the parent.
    let mut gauge = call(99, 99);
    gauge.scope = Some(UsageScope::Session);
    append(&mut parent, gauge).await;
    append(&mut parent, call(12, 3)).await;
    let mut meta = support::meta(temp.path().into());
    meta.fork = Some((parent.session, parent.head()));
    let mut child = Store::open(root.clone(), meta, None, Default::default())
        .await
        .unwrap();
    let expected = usage::report_logs(std::slice::from_ref(&parent.events)).unwrap();
    assert_eq!(
        usage::report_logs(std::slice::from_ref(&child.events)).unwrap(),
        expected
    );
    assert_eq!(
        usage::report_logs(&[parent.events.clone(), child.events.clone()]).unwrap(),
        expected
    );
    let broken = child
        .events
        .iter()
        .filter(|e| {
            !matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::Usage { .. },
                    ..
                }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let broken = usage::report_logs(&[broken]).unwrap();
    assert!(std::panic::catch_unwind(|| assert_eq!(broken, expected)).is_err());
    println!("broken fork dropping Item::Usage: total-identity assertion rejected");
    append(&mut child, call(7, 1)).await;
    let pair = usage::report_paths(&[
        log_path(&root, parent.session),
        log_path(&root, child.session),
    ])
    .unwrap();
    assert_eq!(pair.rows.len(), 1);
    assert_eq!(pair.rows[0].numbers["input_tokens"], 19);
    assert_eq!(pair.rows[0].numbers["output_tokens"], 4);
    assert_eq!(pair.rows[0].numbers["cost_micros"], 200);
    assert_eq!(pair.rows[0].records, 2);
    let grandchild_meta = SessionMeta {
        session: SessionId::new(),
        fork: Some((child.session, child.head())),
        ..support::meta(temp.path().into())
    };
    let grandchild = Store::open(root.clone(), grandchild_meta, None, Default::default())
        .await
        .unwrap();
    assert_eq!(usage::report_paths(&[root]).unwrap(), pair);
    laws::check_transcript(&grandchild.events).unwrap();
}
#[tokio::test]
async fn native_usage_keeps_backend_version_and_absence() {
    let temp = tempfile::tempdir().unwrap();
    for (kind, text, name, version, input, output) in [
        (
            bboxy450_harness::import::Native::ClaudeCode,
            r#"{"type":"assistant","sessionId":"native-claude","version":"2.1","message":{"id":"request-a","model":"claude-fixture","content":[{"type":"text","text":"ok"}],"usage":{"input_tokens":50,"output_tokens":4,"cache_read_input_tokens":5},"stop_reason":"end_turn"}}"#,
            "claude-code",
            "2.1",
            55,
            4,
        ),
        (
            bboxy450_harness::import::Native::Codex,
            r#"{"type":"session_meta","payload":{"id":"native-codex","cli_version":"0.8"}}
{"type":"turn_context","payload":{"model":"codex-fixture"}}
{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":9,"output_tokens":2,"reasoning_output_tokens":1}}}}"#,
            "codex",
            "0.8",
            9,
            2,
        ),
    ] {
        let file = temp.path().join(format!("{name}.jsonl"));
        std::fs::write(&file, format!("{text}\n")).unwrap();
        let root = temp.path().join(name);
        let id = bboxy450_harness::import::import_file(Some(kind), &file, root.clone())
            .await
            .unwrap();
        let log = std::fs::read_to_string(log_path(&root, id))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_authored(&log).unwrap();
        let usages = log
            .iter()
            .filter_map(|e| match &e.event {
                Event::ItemCompleted {
                    body: Item::Usage { usage },
                    ..
                } => Some(usage),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(usages.len(), 1);
        let usage = usages[0];
        assert_eq!((usage.input_tokens, usage.output_tokens), (input, output));
        let backend = usage.backend.as_ref().unwrap();
        assert_eq!(backend.kind, BackendKind::Import);
        assert_eq!(
            (&*backend.name, backend.version.as_deref()),
            (name, Some(version))
        );
        assert_eq!(usage.scope, Some(UsageScope::Call));
        assert!(usage.cost.is_none());
    }
}
#[test]
fn writer_rules_reject_missing_provenance_and_preserve_unknown_reading() {
    let mut usage = call(5, 2);
    let mut log = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .collect::<Vec<_>>();
    for e in &mut log {
        e.v = Version { major: 0, minor: 4 };
    }
    let e = log
        .iter_mut()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::Usage { .. },
                    ..
                }
            )
        })
        .unwrap();
    e.v = PROTOCOL;
    if let Event::ItemCompleted {
        body: Item::Usage { usage: stored },
        ..
    } = &mut e.event
    {
        *stored = usage.clone();
    }
    laws::check_authored(&log).unwrap();
    for missing in ["scope", "backend", "source", "invalid_placeholder"] {
        let mut bad = log.clone();
        let Event::ItemCompleted {
            body: Item::Usage { usage },
            ..
        } = &mut bad.iter_mut().find(|e| e.v.major == 1).unwrap().event
        else {
            panic!()
        };
        match missing {
            "scope" => usage.scope = None,
            "backend" => usage.backend = None,
            "source" => usage.source = None,
            _ => usage.unreported.push("input_tokens".into()),
        }
        laws::check_transcript(&bad).unwrap();
        assert_eq!(
            laws::check_authored(&bad).unwrap_err().law,
            "usage-provenance"
        );
    }
    usage.source = Some(NumberSource::Estimated);
    let total = Usage::sum_calls(&[usage, call(7, 3)]).unwrap().unwrap();
    assert_eq!(total.input_tokens, 12);
    assert_eq!(total.output_tokens, 5);
    assert_eq!(total.source, Some(NumberSource::Derived));
    let absent = Usage::default().authored(
        UsageScope::Call,
        Backend {
            kind: BackendKind::Import,
            name: "unknown-counters".into(),
            version: None,
        },
        None,
    );
    assert!(absent.unreported.iter().any(|s| s == "input_tokens"));
    assert!(!absent.has_reported_numbers());
    assert!(absent.cached_input_tokens.is_none());
    assert!(Usage::sum_calls(&[call(u64::MAX, 0), call(1, 0)]).is_err());
    let incomparable = ["EUR", "GBP"].map(|currency| Usage {
        unreported: vec!["input_tokens".into(), "output_tokens".into()],
        cost: Some(Box::new(Cost {
            amount_micros: 1,
            currency: currency.into(),
            source: NumberSource::Reported,
            price_source: Some("provider".into()),
        })),
        ..Usage::default()
    });
    assert!(Usage::sum_calls(&incomparable).unwrap().is_none());
}

#[tokio::test]
async fn engine_consolidates_snapshots_per_call_and_derives_the_whole_turn() {
    use bboxy450_harness::provider::{Caps, Finish, ModelEvent, ModelRequest, Provider};
    use std::{
        pin::Pin,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };
    struct ProviderFixture(AtomicUsize);
    impl Provider for ProviderFixture {
        fn id(&self) -> &str {
            "usage-fixture"
        }
        fn caps(&self) -> Caps {
            Caps {
                images: false,
                reasoning: false,
                tool_choice: true,
                context_tokens: 1000,
            }
        }
        fn stream(
            &self,
            _: ModelRequest,
            _: tokio_util::sync::CancellationToken,
        ) -> Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
            let round = self.0.fetch_add(1, Ordering::SeqCst);
            let mut events = vec![
                ModelEvent::Usage(call(10, 0)),
                ModelEvent::Usage(call(10, 5)),
            ];
            if round == 0 {
                events.push(ModelEvent::Retrying {
                    attempt: 1,
                    wait_ms: 0,
                    class: ErrorClass::Transient,
                });
                events.push(ModelEvent::Usage(call(7, 2)));
                let raw = r#"{"path":"input.txt"}"#;
                events.extend([
                    ModelEvent::ToolCallDelta {
                        index: 0,
                        id: Some("usage-read".into()),
                        name: Some("read".into()),
                        args: raw.into(),
                    },
                    ModelEvent::ToolCallReady {
                        index: 0,
                        id: "usage-read".into(),
                        name: "read".into(),
                        args_raw: raw.into(),
                        args: Ok(serde_json::from_str(raw).unwrap()),
                    },
                    ModelEvent::Finish(Finish::ToolCalls),
                ]);
            } else {
                events.push(ModelEvent::Text("done".into()));
                events.push(ModelEvent::Finish(Finish::Stop));
            }
            Box::pin(futures_util::stream::iter(events))
        }
    }
    let session =
        support::with_provider(Arc::new(ProviderFixture(AtomicUsize::new(0))), false).await;
    let mut sub = support::subscribe(&session.gateway, false);
    use bboxy450_harness::gateway::Gateway;
    session
        .gateway
        .send(support::prompt(&session.gateway, "read"))
        .unwrap();
    support::end(&mut sub).await;
    let log = support::close(session).await;
    laws::check_authored(&log).unwrap();
    let calls = log
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::Usage { usage },
                ..
            } => Some(usage),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls
            .iter()
            .map(|u| (u.input_tokens, u.output_tokens))
            .collect::<Vec<_>>(),
        [(10, 5), (7, 2), (10, 5)]
    );
    for usage in calls {
        assert_eq!(usage.scope, Some(UsageScope::Call));
        assert_eq!(usage.backend.as_ref().unwrap().name, "usage-fixture");
        assert_eq!(
            usage.cost.as_ref().unwrap().price_source.as_deref(),
            Some("provider")
        );
    }
    let total = log
        .iter()
        .find_map(|e| match &e.event {
            Event::TurnEnded {
                usage: Some(usage), ..
            } => Some(usage),
            _ => None,
        })
        .unwrap();
    assert_eq!((total.input_tokens, total.output_tokens), (27, 12));
    assert_eq!(total.scope, Some(UsageScope::Turn));
    assert_eq!(total.source, Some(NumberSource::Derived));
    let report = usage::report_logs(&[log]).unwrap();
    assert_eq!(report.rows[0].numbers["input_tokens"], 27);
    assert_eq!(report.rows[0].numbers["output_tokens"], 12);
}

#[tokio::test]
async fn cumulative_only_session_contributes_its_last_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(
        temp.path().into(),
        support::meta(temp.path().into()),
        None,
        Default::default(),
    )
    .await
    .unwrap();
    for n in [10, 20, 7] {
        let mut snapshot = call(n, n);
        snapshot.scope = Some(UsageScope::Session);
        append(&mut store, snapshot).await;
    }
    let report = usage::report_logs(std::slice::from_ref(&store.events)).unwrap();
    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].scope, UsageScope::Session);
    assert_eq!(report.rows[0].records, 1);
    assert_eq!(report.rows[0].numbers["input_tokens"], 7);
    assert_eq!(report.rows[0].numbers["output_tokens"], 7);
}
