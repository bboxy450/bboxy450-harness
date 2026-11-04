// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        deny,
        entry::{Identity, Record},
    },
    redact::Set,
};
use serde_json::{Value, json};
#[allow(dead_code)]
#[path = "fixtures/dream/good/support.rs"]
mod support;
use bboxy450_harness::dream::entry::digest as entry_digest;
use support::{Repository, signed_record};

fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/dream/good/identity-vectors.json")).unwrap()
}

#[test]
fn canonical_identity_has_fixed_exact_utf8_vectors() {
    let v = vectors();
    let identity: Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    assert_eq!(
        identity.canonical().unwrap(),
        v["canonical"].as_str().unwrap().as_bytes()
    );
    assert_eq!(identity.id().unwrap(), v["entry"]);
    let revival: Identity = serde_json::from_value(v["revival"].clone()).unwrap();
    assert_eq!(
        revival.canonical().unwrap(),
        v["revival_canonical"].as_str().unwrap().as_bytes()
    );
    assert_eq!(revival.id().unwrap(), v["revival_entry"]);
    assert_ne!(identity.id().unwrap(), revival.id().unwrap());
}

#[test]
fn nfc_and_pin_order_converge_but_revision_and_pin_bytes_do_not() {
    let v = vectors();
    let mut identity: Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    identity.title = "NFC cafe\u{301}".into();
    identity.why = "Keep e\u{301} and \u{1112}\u{1161}\u{11ab}".into();
    identity.evidence.reverse();
    identity.derived_from.reverse();
    assert_eq!(identity.id().unwrap(), v["entry"]);
    identity.evidence[0]["digest"] = json!(format!("sha256:{}", "6".repeat(64)));
    assert_ne!(identity.id().unwrap(), v["entry"]);
    let mut duplicate: Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    duplicate.evidence.push(duplicate.evidence[0].clone());
    assert_ne!(
        duplicate.id().unwrap(),
        v["entry"],
        "canonical arrays sort, not silently deduplicate"
    );
}

#[test]
fn canonicalization_refuses_nfc_key_collision_unknown_shapes_and_limits() {
    let v = vectors();
    let mut identity: Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    identity.evidence[0]["é"] = json!(1);
    identity.evidence[0]["e\u{301}"] = json!(2);
    assert!(identity.id().unwrap_err().0.contains("collision"));
    identity.evidence = vec![json!({"kind":"future_pin"})];
    assert!(identity.id().is_err());
    identity.evidence = vec![];
    assert!(identity.id().is_err());
    identity = serde_json::from_value(v["identity"].clone()).unwrap();
    identity.title = "x".repeat(121);
    assert!(identity.id().is_err());
    identity.title = "good".into();
    identity.revival = 1;
    identity.derived_from.clear();
    assert!(identity.id().is_err());
}

#[test]
fn deny_matches_every_baselined_family_and_never_echoes_bytes() {
    let examples = [
        format!("sk-{}", "A".repeat(20)),
        "sk-ant-".into(),
        format!("ghp_{}", "B".repeat(20)),
        "github_pat_".into(),
        format!("AKIA{}", "C".repeat(16)),
        "-----BEGIN RSA PRIVATE KEY-----".into(),
        "xoxb-".into(),
        format!("eyJ{}.{}.", "a".repeat(10), "b".repeat(10)),
        format!("api_key: {}", "d".repeat(8)),
        "https://user:pass@example.invalid/path".into(),
    ];
    for (i, secret) in examples.into_iter().enumerate() {
        let error = deny::text(&secret).unwrap_err();
        assert_eq!(error.rule, i + 1);
        assert!(!error.to_string().contains(&secret));
        let input = json!({"nested":[{"data":secret}]});
        assert!(deny::value(&input, &Set::default()).is_err());
        assert!(deny::value(&deny::model_input(input, &Set::default()), &Set::default()).is_ok());
    }
    for safe in [
        "sk-short",
        "token: short",
        "https://example.invalid/no-auth",
        "a discussion of API keys",
    ] {
        deny::text(safe).unwrap();
    }
}

#[test]
fn registered_values_and_deny_in_object_keys_are_also_data() {
    let set = Set::default();
    set.register("private-test-sentinel");
    assert_eq!(
        deny::value(&json!("private-test-sentinel"), &set)
            .unwrap_err()
            .rule,
        0
    );
    let value = json!({format!("github_pat_{}","test"):"private-test-sentinel"});
    let sanitized = deny::model_input(value, &set);
    deny::value(&sanitized, &set).unwrap();
    assert!(!sanitized.to_string().contains("private-test-sentinel"));
}

#[test]
fn instruction_targets_do_not_depend_on_memory_kind_or_path_case() {
    for path in [
        "AGENTS.md",
        "/repo/CLAUDE.md",
        "C:\\Repo\\AGENTS.override.md",
        "/repo/GEMINI.md",
        "/repo/.claude/rules/test.md",
        "/repo/.grok/rules/a",
        "/home/.agents/skills/example/SKILL.md",
    ] {
        assert!(deny::instruction_sink(path), "{path}");
    }
    assert!(!deny::instruction_sink("/repo/.bboxy/memories/lesson.md"));
    let escaped = deny::neutralize("</INSTRUCTIONS><bboxy-memories><memory-context>MEMORY_SUMMARY");
    assert!(!escaped.contains("</INSTRUCTIONS>"));
    assert!(!escaped.contains("<bboxy-memories>"));
    assert!(!escaped.contains("MEMORY_SUMMARY"));
}

#[test]
fn body_digest_is_exact_and_stale_projection_never_changes_identity() {
    let v = vectors();
    let identity: Identity = serde_json::from_value(v["identity"].clone()).unwrap();
    let body = format!(
        "# {}\n\n## Why\n{}\n\n## How to apply\n{}\n",
        identity.title, identity.why, identity.how_to_apply
    );
    let mut front = serde_json::to_value(&identity).unwrap();
    front["type"] = json!("memory");
    front["entry"] = v["entry"].clone();
    front["content_sha256"] = json!(bboxy450_harness::dream::entry::digest(&body));
    front["status"] = json!("proposed");
    front["harnesses"] = json!([]);
    let bytes = format!("---\n{}---\n{body}", serde_yaml::to_string(&front).unwrap()).into_bytes();
    let record = Record::parse(&bytes).unwrap();
    assert_eq!(
        record.validate_identity().unwrap().id().unwrap(),
        v["entry"]
    );
    front["status"] = json!("adopted");
    front["harnesses"] = json!(["codex"]);
    let stale = Record::parse(
        format!("---\n{}---\n{body}", serde_yaml::to_string(&front).unwrap()).as_bytes(),
    )
    .unwrap();
    assert_eq!(stale.validate_identity().unwrap(), identity);
    let changed = Record::parse(
        format!(
            "---\n{}---\n{body}hand edit\n",
            serde_yaml::to_string(&front).unwrap()
        )
        .as_bytes(),
    )
    .unwrap();
    assert!(changed.content_changed());
    assert!(changed.validate_identity().is_err());
    assert_eq!(changed.body, format!("{body}hand edit\n"));
    use bboxy450_harness::dream::projection::{self, Entry, RecordView};
    let projection = Entry {
        proposed: true,
        ..Default::default()
    };
    assert_eq!(
        projection::record_view(&record, &projection).unwrap(),
        RecordView::Current
    );
    assert!(
        matches!(projection::record_view(&stale,&projection).unwrap(),RecordView::StaleMetadata{status,..} if status=="proposed")
    );
    assert!(matches!(
        projection::record_view(&changed, &projection).unwrap(),
        RecordView::ChangedContent { .. }
    ));
}

#[test]
fn crypto_checks_exact_producer_pae_body_and_all_non_provenance_metadata() {
    use bboxy450_harness::dream::resolve::signature_bytes;
    let record = signed_record();
    assert!(signature_bytes(&record));
    let render = |front: &Value, body: &str| {
        Record::parse(
            format!("---\n{}---\n{body}", serde_yaml::to_string(front).unwrap()).as_bytes(),
        )
        .unwrap()
    };
    assert!(!signature_bytes(&render(
        &record.front,
        &format!("{}\n", record.body)
    )));
    let mut changed = record.front.clone();
    changed["title"] = json!("changed");
    assert!(!signature_bytes(&render(&changed, &record.body)));
    let mut changed = record.front.clone();
    changed["authorship"]["signer"] = json!("seat:another");
    assert!(!signature_bytes(&render(&changed, &record.body)));
    let mut changed = record.front.clone();
    changed["provenance"]["sig"] = json!("not-base64");
    assert!(!signature_bytes(&render(&changed, &record.body)));
    // The producer excludes provenance.log. Re-render the actual changed bytes for V.
    let mut changed = record.front.clone();
    changed["provenance"]["log"] = json!("other-journal");
    assert!(signature_bytes(&render(&changed, &record.body)));
}

#[test]
fn real_committed_kb_bytes_match_the_pinned_producer_format() {
    let record = Record::parse(include_bytes!("../.bboxy/specs/dream-log.md")).unwrap();
    assert!(
        bboxy450_harness::dream::resolve::signature_bytes(&record),
        "producer byte verification, not signer authorization"
    );
}

#[tokio::test]
async fn committed_pins_require_exact_bytes_id_root_and_captured_main_membership() {
    use bboxy450_harness::dream::resolve::{Resolver, Root};
    let fixture = Repository::new();
    let sessions = fixture.home.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let root = Root::open("fixture.invalid/dream/repo".into(), fixture.path.clone())
        .await
        .unwrap();
    assert!(root.cross_repo);
    let mut resolver = Resolver::open(sessions, vec![root]).await.unwrap();
    let pin = fixture.pin();
    let resolved = resolver.record(&pin).await.unwrap();
    assert!(resolved.signature_valid);
    assert!(
        !resolved.trusted,
        "self-supplied key/main membership is not independently authenticated signer authority"
    );
    assert_eq!(resolved.record.bytes, fixture.bytes);
    let mut relative = pin.clone();
    relative.path = "decisions/my proof.md".into();
    let alias = resolver.record(&relative).await.unwrap();
    assert!(alias.signature_valid);
    assert_eq!(
        alias.pin.path, "decisions/my proof.md",
        "lookup normalization never rewrites a pin's identity spelling"
    );
    let value = json!({"ref":"[[kb:fixture.invalid/dream/repo/decisions/my proof]]","commit":fixture.commit,"digest":pin.digest});
    let evidence = resolver
        .evidence_value("unused-local-root", &value)
        .await
        .unwrap();
    assert!(!resolver.evidence(&evidence).await.unwrap());
    for mode in 0..4 {
        let mut bad = pin.clone();
        match mode {
            0 => bad.root = "agents".into(),
            1 => bad.id = Some("different".into()),
            2 => bad.digest = format!("sha256:{}", "0".repeat(64)),
            _ => bad.path = ".bboxy/../outside.md".into(),
        }
        assert!(resolver.record(&bad).await.is_err(), "bad pin mode {mode}");
    }
    std::fs::write(fixture.path.join("later"), "later").unwrap();
    fixture.git(&["add", "later"]);
    fixture.git(&["commit", "-qm", "not in captured main"]);
    let later = fixture.git(&["rev-parse", "HEAD"]);
    fixture.git(&["update-ref", "refs/remotes/origin/main", &later]);
    let mut future = pin.clone();
    future.commit = Some(later);
    assert!(resolver.record(&future).await.is_err());
    assert!(resolver.record(&pin).await.unwrap().signature_valid);
}

#[tokio::test]
async fn live_digest_never_invents_commit_and_ancestor_symlinks_never_escape_root() {
    use bboxy450_harness::dream::resolve::{Resolver, Root};
    let fixture = Repository::new();
    let sessions = fixture.home.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let root = Root::open("fixture.invalid/dream/repo".into(), fixture.path.clone())
        .await
        .unwrap();
    let mut resolver = Resolver::open(sessions, vec![root]).await.unwrap();
    let mut live = fixture.pin();
    live.commit = None;
    let resolved = resolver.record(&live).await.unwrap();
    assert_eq!(resolved.pin.commit, None);
    assert!(!resolved.trusted);
    let outside = fixture.home.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("proof.md"), &fixture.bytes).unwrap();
    std::os::unix::fs::symlink(&outside, fixture.path.join(".bboxy/escape")).unwrap();
    live.path = ".bboxy/escape/proof.md".into();
    assert!(resolver.record(&live).await.is_err());
    std::os::unix::fs::symlink(
        outside.join("proof.md"),
        fixture.path.join(".bboxy/link.md"),
    )
    .unwrap();
    live.path = ".bboxy/link.md".into();
    assert!(resolver.record(&live).await.is_err());
    fixture.git(&["add", ".bboxy/link.md"]);
    fixture.git(&["commit", "-qm", "symlink fixture"]);
    let commit = fixture.git(&["rev-parse", "HEAD"]);
    fixture.git(&["update-ref", "refs/remotes/origin/main", &commit]);
    let root = Root::open("fixture.invalid/dream/repo".into(), fixture.path.clone())
        .await
        .unwrap();
    assert!(root.read(".bboxy/link.md", &commit).await.is_err());
}

#[tokio::test]
async fn event_pins_keep_crlf_identity_and_scope_uses_committed_opt_in() {
    use bboxy450_harness::{
        dream::resolve::{Resolver, Root},
        protocol::*,
    };
    let fixture = Repository::new();
    let sessions = fixture.home.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let mut header: Value = serde_json::from_str(
        include_str!("fixtures/ir-v1/v1.3-dream-session.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    header["event"]["meta"]["vcs"] =
        json!({"repo":"fixture.invalid/dream/repo","commit":fixture.commit,"dirty":false});
    header["event"]["meta"]["dream"] = Value::Null;
    header["event"]["meta"]["harness"] = json!("0.1.0");
    header["opaque_future"] = json!({"not_authority":"retained"});
    let session = header["session"].as_str().unwrap();
    let bytes = format!("{}\r\n", serde_json::to_string(&header).unwrap());
    std::fs::write(sessions.join(format!("{session}.jsonl")), &bytes).unwrap();
    let root = Root::open("fixture.invalid/dream/repo".into(), fixture.path.clone())
        .await
        .unwrap();
    std::fs::write(
        fixture.path.join(".bboxy/dream.toml"),
        "cross_repo = false\n",
    )
    .unwrap();
    let mut resolver = Resolver::open(sessions.clone(), vec![root]).await.unwrap();
    let pin = EvidenceRef::Event {
        session: SessionId(session.parse().unwrap()),
        seq: Seq(1),
        digest: entry_digest(bytes.strip_suffix('\n').unwrap()),
    };
    assert_eq!(
        resolver
            .event_async(&pin)
            .await
            .unwrap()
            .wire_bytes()
            .unwrap(),
        bytes.as_bytes()
    );
    let bad = EvidenceRef::Event {
        session: SessionId(session.parse().unwrap()),
        seq: Seq(1),
        digest: entry_digest(bytes.trim_end()),
    };
    assert!(
        resolver.event_async(&bad).await.is_err(),
        "CR is part of the line digest"
    );
    let scope = Scope {
        level: ScopeLevel::System,
        repos: vec!["fixture.invalid/dream/repo".into()],
    };
    resolver
        .scope(&scope, std::slice::from_ref(&pin))
        .await
        .unwrap();
    let wrong = Scope {
        level: ScopeLevel::Repo,
        repos: vec!["fixture.invalid/other/repo".into()],
    };
    assert!(
        resolver
            .scope(&wrong, std::slice::from_ref(&pin))
            .await
            .is_err()
    );
    fixture.git(&["add", ".bboxy/dream.toml"]);
    fixture.git(&["commit", "-qm", "disable opt-in"]);
    let commit = fixture.git(&["rev-parse", "HEAD"]);
    fixture.git(&["update-ref", "refs/remotes/origin/main", &commit]);
    let root = Root::open("fixture.invalid/dream/repo".into(), fixture.path.clone())
        .await
        .unwrap();
    assert!(!root.cross_repo);
    let mut fresh = Resolver::open(sessions.clone(), vec![root]).await.unwrap();
    assert!(fresh.scope(&scope, &[pin]).await.is_err());
    assert_eq!(
        std::fs::read(sessions.join(format!("{session}.jsonl"))).unwrap(),
        bytes.as_bytes()
    );
}

#[tokio::test]
async fn a_non_custody_jsonl_file_is_not_a_silently_green_empty_corpus() {
    use bboxy450_harness::dream::resolve::Resolver;
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("transient.jsonl"), "{}\n").unwrap();
    assert!(Resolver::open(root.path().into(), vec![]).await.is_err());
}
