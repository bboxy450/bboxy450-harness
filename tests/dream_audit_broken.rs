// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        audit,
        entry::{self, Identity},
        resolve::{Resolver, Root},
    },
    redact::Set,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::{Command, Output},
};
#[allow(dead_code)]
#[path = "fixtures/dream/good/support.rs"]
mod support;
use support::{Repository, sign_record};

struct Fixture {
    repo: Repository,
    sessions: PathBuf,
    sink: PathBuf,
    entry: String,
    secret: String,
}
impl Fixture {
    fn new(case: &str) -> Self {
        let repo = Repository::new();
        let anchor = repo.pin();
        let mut identity = Identity {
            kind: "lesson".into(),
            title: "Use the private verifier".into(),
            why: "Record an observed outcome.".into(),
            how_to_apply: "Run the private verifier before publishing.".into(),
            evidence: vec![json!({"kind":"record","root":anchor.root,"id":anchor.id,
            "path":anchor.path,"commit":anchor.commit,"digest":anchor.digest})],
            derived_from: vec![],
            revival: 0,
        };
        let secret = format!("sk-{}", "privatefixtureonly".repeat(3));
        if case == "P8" {
            identity.why = secret.clone();
        }
        let entry = identity.id().unwrap();
        let body = format!(
            "# {}\n\n## Why\n{}\n\n## How to apply\n{}\n",
            identity.title, identity.why, identity.how_to_apply
        );
        let mut front = serde_json::to_value(&identity).unwrap();
        front["id"] = json!("01M00000000000000000000003");
        front["number"] = json!(2);
        front["type"] = json!("memory");
        front["description"] = json!("A synthetic private audit fixture.");
        front["scope"] = json!({"level":"repo","repos":[if case=="P7" {"fixture.invalid/other/repo"}else{"fixture.invalid/dream/repo"}]});
        front["entry"] = json!(entry);
        front["content_sha256"] = json!(entry::digest(&body));
        front["status"] = json!(if case == "P9" { "adopted" } else { "proposed" });
        front["harnesses"] = if case == "P9" {
            json!(["engine"])
        } else {
            json!([])
        };
        front["origin"] = json!({"kind":"dream","session":"00000000000000000000000001","seq":3});
        let memory = sign_record(front, &body, &repo.key);
        std::fs::create_dir_all(repo.path.join(".bboxy/memories")).unwrap();
        std::fs::write(repo.path.join(".bboxy/memories/lesson.md"), &memory.bytes).unwrap();
        repo.git(&["add", ".bboxy/memories/lesson.md"]);
        repo.git(&["commit", "-qm", "private fixture memory"]);
        let commit = repo.git(&["rev-parse", "HEAD"]);
        repo.git(&["update-ref", "refs/remotes/origin/main", &commit]);
        let sessions = repo.home.path().join("sessions");
        std::fs::create_dir(&sessions).unwrap();
        let sink = repo.home.path().join("context.md");
        let block = format!(
            "<bboxy-memories>\n- private entry <!-- dream:{entry} -->\n</bboxy-memories>\n"
        );
        std::fs::write(&sink, &block).unwrap();
        let replacements = BTreeMap::from([
            ("__REPO__", repo.path.to_str().unwrap().to_owned()),
            ("__ENTRY__", entry.clone()),
            ("__COMMIT__", commit),
            ("__DIGEST__", entry::digest(&memory.bytes)),
            ("__ANCHOR_COMMIT__", repo.commit.clone()),
            ("__ANCHOR_DIGEST__", anchor.digest),
            ("__SINK__", sink.to_str().unwrap().into()),
            ("__BLOCK_DIGEST__", entry::digest(&block)),
        ]);
        fn replace(value: &mut Value, replacements: &BTreeMap<&str, String>) {
            match value {
                Value::String(text) => {
                    if let Some(value) = replacements.get(text.as_str()) {
                        *text = value.clone();
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        replace(value, replacements);
                    }
                }
                Value::Object(values) => {
                    for value in values.values_mut() {
                        replace(value, replacements);
                    }
                }
                _ => {}
            }
        }
        let path = if case == "good" {
            "good/proposed.jsonl".into()
        } else {
            format!("broken/{case}.jsonl")
        };
        let template = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/dream")
                .join(path),
        )
        .unwrap();
        let mut bytes = Vec::new();
        for line in template.lines() {
            let mut row: Value = serde_json::from_str(line).unwrap();
            replace(&mut row, &replacements);
            bytes.extend(serde_json::to_vec(&row).unwrap());
            bytes.push(b'\n');
        }
        std::fs::write(sessions.join("00000000000000000000000001.jsonl"), bytes).unwrap();
        Self {
            repo,
            sessions,
            sink,
            entry,
            secret,
        }
    }
    fn cli(&self, subcommand: &str, extra: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        command
            .args(["dream", subcommand, "--sessions"])
            .arg(&self.sessions)
            .args([
                "--root",
                &format!("fixture.invalid/dream/repo={}", self.repo.path.display()),
                "--json",
            ])
            .args(extra)
            .current_dir(&self.repo.path)
            .env("HOME", self.repo.home.path());
        for (name, _) in std::env::vars_os() {
            if name.to_str().is_some_and(|n| n.starts_with("BBOXY_")) {
                command.env_remove(name);
            }
        }
        command.output().unwrap()
    }
    fn hashes(&self) -> BTreeMap<PathBuf, String> {
        let mut out = BTreeMap::new();
        for path in [
            self.sessions.join("00000000000000000000000001.jsonl"),
            self.repo.path.join(".bboxy/memories/lesson.md"),
            self.sink.clone(),
        ] {
            out.insert(path.clone(), entry::digest(std::fs::read(path).unwrap()));
        }
        out
    }
}

#[test]
fn incomplete_run_stays_unproven_and_real_cli_queries_are_read_only() {
    let fixture = Fixture::new("good");
    let before = fixture.hashes();
    for (command, args) in [
        ("audit", vec![]),
        ("graph", vec!["--strength"]),
        ("why", vec![fixture.entry.as_str()]),
        ("why", vec!["lesson"]),
    ] {
        let output = fixture.cli(command, &args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        if command == "audit" {
            assert!(
                value["findings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|f| f["check"] == "P4"
                        && f["kind"] == "unproven"
                        && f["detail"] == "missing_run_receipt")
            );
        }
        assert!(!String::from_utf8_lossy(&output.stdout).contains(&fixture.secret));
    }
    assert_eq!(fixture.hashes(), before);
    assert_eq!(std::fs::read_dir(&fixture.sessions).unwrap().count(), 1);
    let output = fixture.cli("why", &["missing-entry"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn every_retained_broken_case_exits_one_and_names_its_check() {
    for n in 1..=9 {
        let check = format!("P{n}");
        let fixture = Fixture::new(&check);
        let before = fixture.hashes();
        let output = fixture.cli("audit", &[]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{check}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{check}: {e}; stderr={}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        assert!(
            value["findings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["check"] == check),
            "{check}: {value}"
        );
        assert_eq!(fixture.hashes(), before);
        assert!(!String::from_utf8_lossy(&output.stdout).contains(&fixture.secret));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(&fixture.secret));
    }
}

#[tokio::test]
async fn matching_current_bytes_never_replace_missing_publication_history() {
    let fixture = Fixture::new("P2");
    let root = Root::open(
        "fixture.invalid/dream/repo".into(),
        fixture.repo.path.clone(),
    )
    .await
    .unwrap();
    let mut resolver = Resolver::open(fixture.sessions.clone(), vec![root])
        .await
        .unwrap();
    let report = audit::run(&mut resolver, Set::default(), false, 0)
        .await
        .unwrap();
    assert!(fixture.sink.is_file());
    assert_eq!(report.exit_code(), 1);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.check == "P2" && f.kind == "unproven")
    );
}

#[allow(dead_code)]
#[path = "fixtures/dream/trust/fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "fixtures/dream/good/receipt_support.rs"]
mod r;
#[allow(dead_code)]
#[path = "fixtures/dream/good/audit_fixture.rs"]
mod sufficient;

#[tokio::test]
async fn sufficient_retained_publication_run_idempotence_and_stale_exposure_accept() {
    let fixture = sufficient::AuditFixture::new(true);
    let before = fixture
        .logs
        .iter()
        .map(|(s, l)| {
            (
                *s,
                entry::digest(
                    l.iter()
                        .flat_map(|e| e.wire_bytes().unwrap())
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut resolver = fixture.resolver().await;
    let result = audit::run(&mut resolver, Set::default(), true, 50000)
        .await
        .unwrap();
    assert_eq!(
        result.exit_code(),
        0,
        "{}",
        serde_json::to_string_pretty(&result).unwrap()
    );
    assert!(result.labels.iter().any(|l| l["kind"] == "stale_exposure"));
    assert!(result.graph.nodes.values().any(|n| n.kind == "sink_line"));
    let sink = result
        .graph
        .nodes
        .values()
        .find(|n| n.kind == "sink_line")
        .unwrap()
        .id
        .clone();
    for (command, query) in [("audit", None), ("why", Some(sink.as_str()))] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        cmd.args(["dream", command, "--sessions"])
            .arg(&fixture.sessions)
            .args([
                "--root",
                &format!(
                    "fixture.invalid/dream/repo={}",
                    fixture.fixture.repo.path.display()
                ),
                "--trust-policy",
            ])
            .arg(&fixture.fixture.policy)
            .arg("--json");
        if let Some(query) = query {
            cmd.arg(query);
        }
        for (name, _) in std::env::vars_os() {
            if name.to_str().is_some_and(|n| n.starts_with("BBOXY_")) {
                cmd.env_remove(name);
            }
        }
        let out = cmd
            .current_dir(&fixture.fixture.repo.path)
            .env("HOME", fixture.fixture.repo.home.path())
            .env_remove("BBOXY_CAPABILITY_FILE")
            .env_remove("BBOXY_CAPABILITY")
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    let path = &fixture.publications[0]
        .2
        .selection
        .as_ref()
        .unwrap()
        .generated[0]
        .path;
    std::fs::write(path, b"a legitimate later hand edit\n").unwrap();
    let mut resolver = fixture.resolver().await;
    let changed = audit::run(&mut resolver, Set::default(), false, 0)
        .await
        .unwrap();
    assert_eq!(
        changed.exit_code(),
        0,
        "{}",
        serde_json::to_string(&changed).unwrap()
    );
    assert!(changed.labels.iter().any(|l| l["kind"] == "drift"));
    assert_eq!(
        std::fs::read(path).unwrap(),
        b"a legitimate later hand edit\n"
    );
    for (s, digest) in before {
        assert_eq!(
            entry::digest(std::fs::read(fixture.sessions.join(format!("{s}.jsonl"))).unwrap()),
            digest
        );
    }
}

#[tokio::test]
async fn reported_recall_requires_successful_publication_not_a_valid_skip() {
    for skipped in [false, true] {
        let mut fixture = if skipped {
            sufficient::AuditFixture::skipped()
        } else {
            sufficient::AuditFixture::new(false)
        };
        let mut resolver = fixture.resolver().await;
        let before = audit::run(&mut resolver, Set::default(), false, 0)
            .await
            .unwrap();
        assert_eq!(
            before.exit_code(),
            0,
            "{}",
            serde_json::to_string(&before).unwrap()
        );
        assert_eq!(
            before.graph.nodes.values().any(|n| n.kind == "sink_line"),
            !skipped
        );
        fixture.add_reported_recall();
        let mut resolver = fixture.resolver().await;
        let report = audit::run(&mut resolver, Set::default(), false, 0)
            .await
            .unwrap();
        assert_eq!(
            report.exit_code(),
            i32::from(skipped),
            "skipped={skipped}: {}",
            serde_json::to_string(&report.findings).unwrap()
        );
        if skipped {
            assert!(
                report
                    .findings
                    .iter()
                    .any(|f| f.check == "P2" && f.detail == "recall_publication_binding")
            );
        }
    }
}
