// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{frontends::ir_scan::*, protocol::schema_value, redact::Set};
use serde_json::{Value, json};
use std::{path::Path, process::Command};

fn envelope(body: Value) -> Value {
    json!({"v":{"major":1,"minor":1},"session":"01M6F1VQBMY7AZZTA1SGJFPT6E", "seq":1,"epoch":1,"at":1,"ts_ms":7,"cause":null,
        "event":{"type":"item_completed","item":"01M6F1VQC21RZZ0RK2Y1PHEDQC","status":{"status":"completed"},"body":body}})
}
fn lines(path: &Path, values: &[Value]) {
    std::fs::write(
        path,
        values.iter().map(|v| format!("{v}\n")).collect::<String>(),
    )
    .unwrap();
}

#[tokio::test]
async fn two_shapes_generate_additive_proposals_and_indexable_tasks() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("observed.jsonl");
    let a = envelope(json!({"kind":"future_widget","width":4}));
    let b = envelope(json!({"kind":"future_graph","nodes":[{"title":"x"}]}));
    lines(&file, &[a.clone(), b, a]);
    let catalogue = tmp.path().join("catalogue");
    let scan = scan_paths(
        vec![file.clone()],
        schema_value(),
        Set::default(),
        Some(catalogue.clone()),
    )
    .await
    .unwrap();
    assert_eq!(scan.entries.len(), 2);
    assert_eq!(scan.new_fingerprints.len(), 2);
    assert_eq!(scan.entries.values().map(|e| e.count).sum::<u64>(), 3);
    let proposals = tmp.path().join("proposed");
    propose(&scan, &schema_value(), &proposals).await.unwrap();
    check_pending(&scan, &proposals.join("tests/fixtures/ir-v1/pending"))
        .await
        .unwrap();
    assert!(
        check_pending(&scan, &tmp.path().join("missing"))
            .await
            .is_err()
    );
    let fragments: Vec<_> = std::fs::read_dir(proposals.join("proposals"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".schema.json"))
        .collect();
    assert_eq!(fragments.len(), 2);
    for fragment in &fragments {
        let out = Command::new("python3")
            .args(["-I", "scripts/ir-compat.py", "check", "--fragment"])
            .arg(fragment)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    let script = format!(
        "import runpy\nm=runpy.run_path('scripts/checks.py')\nm['kb_records'](m['Path']({:?})/'.bboxy')\n",
        proposals.to_str().unwrap()
    );
    let out = Command::new("python3")
        .args(["-I", "-c", &script])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let scan2 = scan_paths(
        vec![file],
        schema_value(),
        Set::default(),
        Some(catalogue.clone()),
    )
    .await
    .unwrap();
    assert!(scan2.new_fingerprints.is_empty());
    for e in scan2.entries.values() {
        let persisted: Entry = serde_json::from_slice(
            &std::fs::read(catalogue.join(format!("{}.json", e.fingerprint))).unwrap(),
        )
        .unwrap();
        assert_eq!(persisted.count, e.count * 2);
        assert_eq!(persisted.sessions.len(), 1);
        assert_eq!(persisted.first_seen, 7);
        assert_eq!(persisted.last_seen, 7);
    }
    // Broken generator control: a fragment renames a field that is already locked.
    let mut schema = schema_value();
    let props = schema["$defs"]["Usage"]["properties"]
        .as_object_mut()
        .unwrap();
    let value = props.remove("input_tokens").unwrap();
    props.insert("renamed_tokens".into(), value);
    let broken = tmp.path().join("renamed.schema.json");
    std::fs::write(&broken, schema.to_string()).unwrap();
    let out = Command::new("python3")
        .args(["-I", "scripts/ir-compat.py", "check", "--fragment"])
        .arg(broken)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("field removed"));
    println!(
        "broken proposal rename: refused (exit {})",
        out.status.code().unwrap()
    );
}

#[tokio::test]
async fn unknown_fields_ext_enums_redaction_and_sample_cap() {
    let mut empty = Scan::default();
    let mut nullable = envelope(json!({"kind":"agent_message","markdown":"hello"}));
    nullable["event"] = json!({"type":"item_started","turn":null,"item":"01M6F1VQC21RZZ0RK2Y1PHEDQC","kind":"agent_message","parent":null,"head":null});
    empty.inspect(&nullable, &schema_value(), &Set::default(), "test");
    assert!(
        empty.entries.is_empty(),
        "nullable known fields are not unknowns: {empty:#?}"
    );
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("log.jsonl");
    let mut known = envelope(json!({"kind":"agent_message","markdown":"hi","future_flag":true}));
    known["ext"] = json!({"acp":{"vendor": {"authorization":"private", "body":"needle"}}});
    let huge = envelope(json!({"kind":"future_huge","content":"needle".repeat(20_000)}));
    let enumeration = envelope(
        json!({"kind":"tool_call","call_id":"x","name":"x","tool_kind":"future_tool","input_raw":"{}","output":[],"locations":[],"spill":null}),
    );
    lines(&file, &[known, huge, enumeration]);
    let redactor = Set::default();
    redactor.register("needle");
    let scan = scan_paths(vec![file], schema_value(), redactor, None)
        .await
        .unwrap();
    assert_eq!(scan.entries.len(), 4, "{scan:#?}");
    assert!(scan.entries.values().any(|e| e.carrier == "field"));
    assert!(
        scan.entries.values().any(|e| e.provider == "ToolKind"),
        "{scan:#?}"
    );
    for e in scan.entries.values() {
        let sample = serde_json::to_vec(&e.sample).unwrap();
        assert!(sample.len() <= SAMPLE_BYTES);
        assert!(!String::from_utf8_lossy(&sample).contains("needle"));
        assert!(!String::from_utf8_lossy(&sample).contains("private"));
    }
    assert_eq!(
        shape(&json!({"b":[1,2],"a":"one"})),
        shape(&json!({"a":"two","b":[9]}))
    );
    assert_ne!(shape(&json!({"a/b":1})), shape(&json!({"a":{"b":1}})));
}

#[tokio::test]
async fn close_scan_worker_budget_and_removed_budget_control() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("large.jsonl");
    let fields: serde_json::Map<String, Value> = (0..15000)
        .map(|i| (format!("field_{i}"), json!(i)))
        .collect();
    let value = envelope(json!({"kind":"future_large","data":fields}));
    lines(&file, &[value]);
    let good = scan_paths(vec![file.clone()], schema_value(), Set::default(), None)
        .await
        .unwrap();
    assert!(good.max_poll_cpu_ns <= CPU_BUDGET_NS);
    assert_eq!(good.lines, 1);
    let bad = unbounded_close_control(file, tmp.path().join("catalogue"))
        .await
        .unwrap()
        .unwrap_err();
    assert!(bad.to_string().contains("1 ms CPU"));
    println!("broken close scan without worker budget: refused ({bad})");
    // Close must return even if the queued background job never makes progress.
    let source = std::fs::read_to_string("src/ir_scan.rs").unwrap();
    let hook = source
        .split("pub fn on_close(")
        .nth(1)
        .unwrap()
        .split("pub async fn cli")
        .next()
        .unwrap();
    assert!(hook.contains("bbt_rt::handle().spawn"));
    assert!(!hook.contains("block_on"));
}

#[tokio::test]
async fn close_hook_is_detached_emits_only_new_kinds_and_keeps_source_immutable() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("closed.jsonl");
    lines(
        &file,
        &[envelope(json!({"kind":"future_close","amount":1}))],
    );
    let original = std::fs::read(&file).unwrap();
    let notices = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let target = notices.clone();
    let catalogue = tmp.path().join("catalogue");
    let job = on_close_in(
        file.clone(),
        Set::default(),
        catalogue.clone(),
        move |notice| target.lock().unwrap().push(notice),
    );
    // The caller has already regained ownership before the worker completes.
    let scan = tokio::time::timeout(std::time::Duration::from_secs(10), job)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(scan.new_fingerprints.len(), 1);
    assert_eq!(notices.lock().unwrap().len(), 1);
    assert!(notices.lock().unwrap()[0].contains("unmapped kinds from Item; run ir-scan --propose"));
    let target = notices.clone();
    on_close_in(file.clone(), Set::default(), catalogue, move |notice| {
        target.lock().unwrap().push(notice)
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(notices.lock().unwrap().len(), 1);
    assert_eq!(original, std::fs::read(file).unwrap());
}

#[test]
fn stale_lock_requires_self_service_refresh() {
    let tmp = tempfile::tempdir().unwrap();
    let mut schema = schema_value();
    schema["$defs"]["Usage"]["properties"]["new_optional"] = json!({"type":"string"});
    let path = tmp.path().join("additive.json");
    std::fs::write(&path, schema.to_string()).unwrap();
    let run = |current: bool| {
        let mut cmd = Command::new("python3");
        cmd.args(["-I", "scripts/ir-compat.py", "check", "--schema"])
            .arg(&path);
        if current {
            cmd.arg("--require-current");
        }
        cmd.output().unwrap()
    };
    assert!(run(false).status.success());
    let out = run(true);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("run make lock-ir"));
}

#[test]
fn cli_scans_fixtures_in_private_home_and_rejects_missing_pending_proposals() {
    let tmp = tempfile::tempdir().unwrap();
    let run = |pending: &Path| {
        Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .args(["ir-scan", "--json", "--propose"])
            .arg(tmp.path().join("proposed"))
            .arg("--check-pending")
            .arg(pending)
            .arg("tests/fixtures/ir-scan")
            .env("HOME", tmp.path())
            .env_remove("BBOXY_CAPABILITY_FILE")
            .env_remove("BBOXY_CAPABILITY")
            .output()
            .unwrap()
    };
    let output = run(Path::new("tests/fixtures/ir-v1/pending"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let scan: Scan = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(scan.entries.len(), 2);
    assert_eq!(
        std::fs::read_dir(tmp.path().join(".bboxy450-harness/ir-unknowns"))
            .unwrap()
            .filter(|p| p
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "json"))
            .count(),
        2
    );
    let missing = run(&tmp.path().join("missing"));
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("has no pending proposal"));
}
