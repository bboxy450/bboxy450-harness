// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    dream::{
        entry::{self, Record},
        receipt::{self, Generated, Outcome},
        trust::{self, Policy, Trust},
    },
    protocol::*,
};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
#[allow(dead_code)]
#[path = "fixtures/dream/good/support.rs"]
mod support;
use support::{Repository, sign_record};

#[test]
fn exact_v_rejects_byte_serialization_crypto_and_binding_controls() {
    let record = support::signed_record();
    let id = record.front["id"].as_str().unwrap();
    let kinds = vec!["seat".into()];
    assert!(trust::verify_bytes(&record.bytes, id, &kinds).is_ok());
    let bytes = String::from_utf8(record.bytes.clone()).unwrap();
    let mut controls = vec![
        format!("\u{feff}{bytes}"),
        bytes.replace('\n', "\r\n"),
        bytes.replace("signed fixture body", "different body"),
        bytes.replace("Toy proof", "other title"),
        bytes.replacen("---\n", "---\n ", 1),
        bytes.replace("---\nsigned fixture body", "signed fixture body"),
        bytes.replace("title: Toy proof", "title: 'Toy proof'"),
    ];
    let mut short = record.front.clone();
    short["provenance"]["sig"] = json!("YQ==");
    controls.push(format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(&short).unwrap(),
        record.body
    ));
    let mut after = record.front.clone();
    after["after_provenance"] = json!(1);
    controls.push(format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(&after).unwrap(),
        record.body
    ));
    let mut mismatch = record.front.clone();
    mismatch["provenance"]["signer"] = json!("seat:01M00000000000000000000009");
    controls.push(format!(
        "---\n{}---\n{}",
        serde_yaml::to_string(&mismatch).unwrap(),
        record.body
    ));
    for (n, control) in controls.iter().enumerate() {
        assert!(
            trust::verify_bytes(control.as_bytes(), id, &kinds).is_err(),
            "control {n}"
        );
    }
    assert!(trust::verify_bytes(&record.bytes, "01M00000000000000000000009", &kinds).is_err());
    assert!(trust::verify_bytes(&record.bytes, id, &["operator".into()]).is_err());
    let key = support::key_pair();
    let replaced = sign_record(record.front.clone(), &record.body, &key);
    assert!(
        trust::verify_bytes(&replaced.bytes, id, &kinds).is_ok(),
        "K2 passes byte integrity only"
    );
    let mut log = record.front.clone();
    log["provenance"]["log"] = json!("unsigned locator");
    let log = Record::parse(
        format!(
            "---\n{}---\n{}",
            serde_yaml::to_string(&log).unwrap(),
            record.body
        )
        .as_bytes(),
    )
    .unwrap();
    assert!(
        trust::verify_bytes(&log.bytes, id, &kinds).is_ok(),
        "the pinned producer excludes provenance.log"
    );
}

struct Fixture {
    repo: Repository,
    gpg: tempfile::TempDir,
    policy: PathBuf,
    value: Value,
    pin: EvidenceRef,
}
impl Fixture {
    fn gpg(&self, args: &[&str]) -> Vec<u8> {
        let output = Command::new("gpg")
            .args(["--no-options", "--homedir"])
            .arg(self.gpg.path())
            .args([
                "--batch",
                "--no-tty",
                "--pinentry-mode",
                "loopback",
                "--passphrase",
                "",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "private gpg: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }
    fn new() -> Self {
        let repo = Repository::new();
        let gpg = tempfile::Builder::new()
            .prefix("wp4-gpg-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(gpg.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut f = Self {
            policy: repo.home.path().join("trust.json"),
            repo,
            gpg,
            value: Value::Null,
            pin: EvidenceRef::Unknown(json!({})),
        };
        f.gpg(&[
            "--quick-generate-key",
            "Boxy <bboxy450@gmail.com>",
            "ed25519",
            "sign",
            "0",
        ]);
        let listing = String::from_utf8(f.gpg(&["--with-colons", "--list-keys"])).unwrap();
        let fingerprint = listing
            .lines()
            .find(|l| l.starts_with("fpr:"))
            .unwrap()
            .split(':')
            .nth(9)
            .unwrap()
            .to_owned();
        let key = f.repo.home.path().join("public.asc");
        std::fs::write(&key, f.gpg(&["--armor", "--export", &fingerprint])).unwrap();
        // Only the test's exact private keyring signs. The product independently imports public keys.
        let wrapper = f.repo.home.path().join("sign-gpg");
        std::fs::write(&wrapper,format!("#!/bin/sh\nexec /opt/homebrew/bin/gpg --no-options --homedir '{}' --batch --pinentry-mode loopback \"$@\"\n",f.gpg.path().display())).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
        f.repo.git(&["config", "gpg.format", "openpgp"]);
        f.repo
            .git(&["config", "gpg.program", wrapper.to_str().unwrap()]);
        f.repo.git(&["config", "user.signingkey", &fingerprint]);
        let record = sign_record(
            json!({"id":"01M00000000000000000000002","number":1,"type":"decision","title":"independently committed"}),
            "private root anchor\n",
            &f.repo.key,
        );
        std::fs::write(
            f.repo.path.join(".bboxy/decisions/my proof.md"),
            &record.bytes,
        )
        .unwrap();
        f.repo.git(&["add", "."]);
        f.repo.git(&["commit", "-qm", "private OpenPGP root"]);
        let commit = f.repo.git(&["rev-parse", "HEAD"]);
        f.repo
            .git(&["update-ref", "refs/remotes/origin/main", &commit]);
        f.pin = EvidenceRef::Record {
            root: "fixture.invalid/dream/repo".into(),
            id: "01M00000000000000000000002".into(),
            path: ".bboxy/decisions/my proof.md".into(),
            commit,
            digest: entry::digest(&record.bytes),
        };
        f.value = json!({"schema":"bboxy.dream.trust/2","roots":[{"repo":"fixture.invalid/dream/repo","git_dir":f.repo.path,"branch":"refs/remotes/origin/main"}],
            "git_signers":[{"fingerprint":fingerprint,"public_key_file":key,"repos":["fixture.invalid/dream/repo"],"revoked_after":null}],"kb_signer_kinds":["seat"],"producers":[]});
        f.write();
        f
    }
    fn write(&self) {
        std::fs::write(&self.policy, serde_json::to_vec(&self.value).unwrap()).unwrap();
        std::fs::set_permissions(&self.policy, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    async fn load(&self) -> Result<Trust, entry::Invalid> {
        Trust::load(self.policy.clone(), vec![self.repo.path.clone()]).await
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = Command::new("gpgconf")
            .arg("--homedir")
            .arg(self.gpg.path())
            .args(["--kill", "gpg-agent"])
            .output();
    }
}

#[tokio::test]
async fn independent_git_authority_and_rotation_are_ancestry_based() {
    let mut f = Fixture::new();
    let trust = f.load().await.unwrap();
    trust.anchor(&f.pin).await.unwrap();
    let sessions = f.repo.home.path().join("sessions");
    std::fs::create_dir(&sessions).unwrap();
    let root = bboxy450_harness::dream::resolve::Root::open(
        "fixture.invalid/dream/repo".into(),
        f.repo.path.clone(),
    )
    .await
    .unwrap();
    let mut resolver = bboxy450_harness::dream::resolve::Resolver::open(sessions, vec![root])
        .await
        .unwrap();
    assert!(
        !resolver.evidence(&f.pin).await.unwrap(),
        "no policy cannot self-authorize"
    );
    resolver = resolver.with_trust(f.load().await.unwrap());
    assert!(
        resolver.evidence(&f.pin).await.unwrap(),
        "real resolver uses independent T"
    );
    let mut alias = f.pin.clone();
    if let EvidenceRef::Record { root, .. } = &mut alias {
        *root = "alias".into();
    }
    assert_eq!(trust.anchor(&alias).await.unwrap_err().0, "unknown_root");
    f.repo.git(&[
        "tag",
        "-a",
        "private-annotation",
        "-m",
        "not a commit object",
    ]);
    let tag = f.repo.git(&["rev-parse", "private-annotation"]);
    let mut tagged = f.pin.clone();
    if let EvidenceRef::Record { commit, .. } = &mut tagged {
        *commit = tag;
    }
    assert!(
        trust.anchor(&tagged).await.is_err(),
        "a tag object's hex spelling is not a commit pin"
    );
    let mut wrong = f.pin.clone();
    if let EvidenceRef::Record { digest, .. } = &mut wrong {
        *digest = entry::digest(b"other");
    }
    assert_eq!(trust.anchor(&wrong).await.unwrap_err().0, "pin_digest");
    f.value["git_signers"][0]["revoked_after"] = json!(f.repo.commit);
    f.write();
    assert!(
        f.load().await.unwrap().anchor(&f.pin).await.is_err(),
        "later touching commit is revoked without clocks"
    );
    if let EvidenceRef::Record { commit, .. } = &f.pin {
        f.value["git_signers"][0]["revoked_after"] = json!(commit);
    }
    f.write();
    assert!(
        f.load().await.unwrap().anchor(&f.pin).await.is_ok(),
        "cut itself is authorized"
    );
    // K2: bytes validly re-signed but never committed by an authorized Git signer.
    let record = sign_record(
        json!({"id":"01M00000000000000000000002","number":1,"type":"decision","title":"K2"}),
        "unapproved writer\n",
        &support::key_pair(),
    );
    assert!(
        trust::verify_bytes(
            &record.bytes,
            "01M00000000000000000000002",
            &["seat".into()]
        )
        .is_ok()
    );
    std::fs::write(
        f.repo.path.join(".bboxy/decisions/my proof.md"),
        &record.bytes,
    )
    .unwrap();
    // This disposable SSH key is not the allowlisted OpenPGP signer.
    f.repo.git(&["config", "gpg.format", "ssh"]);
    f.repo.git(&[
        "config",
        "user.signingkey",
        f.repo.home.path().join("git-signing").to_str().unwrap(),
    ]);
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "unapproved K2"]);
    let commit = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    let mut pin = f.pin.clone();
    if let EvidenceRef::Record {
        commit: c, digest, ..
    } = &mut pin
    {
        *c = commit;
        *digest = entry::digest(&record.bytes);
    }
    assert!(f.load().await.unwrap().anchor(&pin).await.is_err());
}

#[tokio::test]
async fn policy_rejects_unknown_schema_aliases_permissions_and_root_rebinding() {
    let mut f = Fixture::new();
    let original = f.value.clone();
    for mutation in [json!("future"), json!("bboxy.dream.trust/1")] {
        f.value["schema"] = mutation;
        f.write();
        assert!(f.load().await.is_err());
    }
    f.value = original.clone();
    f.value["extra"] = json!(true);
    f.write();
    assert!(f.load().await.is_err());
    f.value = original.clone();
    f.value["roots"]
        .as_array_mut()
        .unwrap()
        .push(original["roots"][0].clone());
    f.write();
    assert!(f.load().await.is_err());
    f.value = original.clone();
    f.value["roots"][0]["repo"] = json!("https://fixture.invalid/dream/repo.git");
    f.write();
    assert!(f.load().await.is_err());
    f.value = original.clone();
    f.value["git_signers"][0]["fingerprint"] = json!("0".repeat(40));
    f.write();
    assert!(f.load().await.is_err());
    f.value = original.clone();
    f.write();
    std::fs::set_permissions(&f.policy, std::fs::Permissions::from_mode(0o620)).unwrap();
    assert!(f.load().await.is_err());
    f.write();
    let link = f.repo.home.path().join("policy-link");
    std::os::unix::fs::symlink(&f.policy, &link).unwrap();
    assert!(Trust::load(link, vec![]).await.is_err());
    assert!(
        Trust::load(f.policy.clone(), vec![f.repo.home.path().into()])
            .await
            .is_err()
    );
    let inside = f.repo.path.join("trust.json");
    std::fs::copy(&f.policy, &inside).unwrap();
    assert!(
        Trust::load(inside, vec![]).await.is_err(),
        "policy in a repository is never an independent authority"
    );
    f.repo.git(&[
        "remote",
        "set-url",
        "origin",
        "https://fixture.invalid/other/repo.git",
    ]);
    assert!(
        f.load().await.unwrap().anchor(&f.pin).await.is_err(),
        "one unbound root degrades separately"
    );
    assert!(
        Policy::parse(br#"{"schema":"bboxy.dream.trust/2","schema":"bboxy.dream.trust/2"}"#)
            .is_err()
    );
}

fn trace() -> (Vec<Value>, Vec<Generated>) {
    let hash = "a".repeat(64);
    let target = "/private/dream/context.md";
    let temp = "/private/dream/.dream-tmp-01M00000000000000000000009";
    let ops = vec![
        json!({"i":0,"op":"read","path":target,"result":"absent"}),
        json!({"i":1,"op":"create_temp","path":temp,"result":"ok"}),
        json!({"i":2,"op":"write_temp","path":temp,"sha256":hash,"len":42,"result":"ok"}),
        json!({"i":3,"op":"fsync","path":temp,"result":"ok"}),
        json!({"i":4,"op":"link_noreplace","from":temp,"to":target,"result":"ok"}),
        json!({"i":5,"op":"fsync_dir","path":"/private/dream","result":"ok"}),
        json!({"i":6,"op":"unlink_own_temp","path":temp,"result":"ok"}),
    ];
    (
        ops,
        vec![Generated {
            path: target.into(),
            sha256: hash,
            len: 42,
        }],
    )
}
fn interpret(
    ops: &[Value],
    generated: &[Generated],
    outcome: Option<Outcome>,
) -> Result<receipt::Interpretation, entry::Invalid> {
    receipt::interpret(
        &receipt::parse_effects(ops)?,
        generated,
        SessionId(ulid::Ulid::from(1u128)),
        &["/private/dream/cache".into()],
        outcome,
    )
}
#[test]
fn deterministic_effect_trace_positive_and_negative_controls() {
    let (ops, generated) = trace();
    assert_eq!(
        interpret(&ops, &generated, Some(Outcome::Published))
            .unwrap()
            .outcome,
        Outcome::Published
    );
    let mut cases = vec![];
    let mut bad = ops.clone();
    bad[4]["from"] = json!("/private/dream/unowned");
    cases.push((bad, "trace:unowned_source"));
    let mut bad = ops.clone();
    bad[2]["sha256"] = json!("b".repeat(64));
    cases.push((bad, "trace:temp_digest"));
    let mut bad = ops.clone();
    bad.remove(3);
    for (i, v) in bad.iter_mut().enumerate() {
        v["i"] = json!(i);
    }
    cases.push((bad, "trace:unsynced"));
    let mut bad = ops.clone();
    bad.remove(5);
    for (i, v) in bad.iter_mut().enumerate() {
        v["i"] = json!(i);
    }
    cases.push((bad, "trace:no_dir_sync"));
    let mut bad = ops.clone();
    bad[4]["result"] = json!("error");
    cases.push((bad, "outcome_mismatch"));
    let mut bad = ops.clone();
    bad[4]["i"] = json!(40);
    cases.push((bad, "truncated_witness"));
    let mut bad = ops.clone();
    bad[0] = json!({"i":0,"op":"read","path":generated[0].path,"result":"present","sha256":"c".repeat(64),"len":42});
    cases.push((bad, "trace:inconsistent"));
    for (bad, expected) in cases {
        assert_eq!(
            interpret(&bad, &generated, Some(Outcome::Published))
                .unwrap_err()
                .0,
            expected
        );
    }
    let mut extra = ops.clone();
    extra[0]["unchecked"] = json!(true);
    assert!(interpret(&extra, &generated, None).is_err());
    let mut missing = ops.clone();
    missing[1].as_object_mut().unwrap().remove("result");
    assert_eq!(
        interpret(&missing, &generated, None).unwrap_err().0,
        "malformed_effect"
    );
    for mutation in ["rename", "unlink", "truncate", "chmod"] {
        let bad = vec![json!({"i":0,"op":mutation,"path":generated[0].path,"result":"ok"})];
        assert_eq!(
            interpret(&bad, &generated, None).unwrap_err().0,
            "trace:clobber"
        );
    }
    let same = vec![
        json!({"i":0,"op":"read","path":generated[0].path,"result":"present","sha256":generated[0].sha256,"len":42}),
    ];
    assert_eq!(
        interpret(&same, &generated, Some(Outcome::Idempotent))
            .unwrap()
            .outcome,
        Outcome::Idempotent
    );
    let mut race = ops.clone();
    race[4]["result"] = json!("eexist");
    race.insert(5,json!({"i":5,"op":"read","path":generated[0].path,"result":"present","sha256":"c".repeat(64),"len":42}));
    for (i, v) in race.iter_mut().enumerate() {
        v["i"] = json!(i);
    }
    assert_eq!(
        interpret(&race, &generated, None).unwrap().outcome,
        Outcome::Skipped
    );
    assert!(interpret(&race, &generated, Some(Outcome::Published)).is_err());
    // An ABA rewrite is a forbidden mutation even when the retained digest agrees.
    let mut aba = same;
    aba.push(json!({"i":1,"op":"unlink","path":generated[0].path,"result":"ok"}));
    assert_eq!(
        interpret(&aba, &generated, None).unwrap_err().0,
        "trace:clobber"
    );
    let source = vec![
        json!({"i":0,"op":"append_own_log","session":"00000000000000000000000002","seq_from":1,"seq_to":2,"result":"ok"}),
    ];
    assert_eq!(
        interpret(&source, &[], None).unwrap_err().0,
        "writer_touched_source"
    );
}

#[tokio::test]
async fn approved_artifact_binds_executable_capabilities_gate_and_accepted_task() {
    let mut f = Fixture::new();
    let source = f.repo.git(&["rev-parse", "HEAD"]);
    let tree = f.repo.git(&["rev-parse", "HEAD^{tree}"]);
    let producer = json!({"repo":"fixture.invalid/dream/repo","commit":source,"tree":tree,"version":"test/1","build":"test/private","exe_sha256":"a".repeat(64),"capabilities":["receipt/1","effect-wrapper/1","no-replace/1"]});
    let mut gate = producer.clone();
    gate["schema"] = json!("bboxy.dream.producer-gate/1");
    gate["witnesses"] = json!([
        {"id":"dream_trust::receipt_header_body_roundtrip","exit_code":0},
        {"id":"dream_sinks::receipt_ops_match_independent_observer","exit_code":0},
        {"id":"dream_sinks::hand_write_between_read_and_link_preserved","exit_code":0},
        {"id":"dream_sinks::overwrite_rename_control_fails_audit","exit_code":0}]);
    let task = sign_record(
        json!({"id":"01M00000000000000000000004","number":4,"type":"task","title":"private artifact approval","status":"accepted","verify_result":{"exit_code":0},"report":{"commit":source}}),
        "synthetic gate evidence; not a live executable approval\n",
        &f.repo.key,
    );
    let log = serde_json::to_vec(&gate).unwrap();
    std::fs::create_dir_all(f.repo.path.join(".bboxy/runs")).unwrap();
    std::fs::create_dir_all(f.repo.path.join(".bboxy/tasks")).unwrap();
    std::fs::write(f.repo.path.join(".bboxy/runs/gate.json"), &log).unwrap();
    std::fs::write(f.repo.path.join(".bboxy/tasks/approval.md"), &task.bytes).unwrap();
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "private retained approval"]);
    let commit = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    let mut approval = producer.clone();
    approval["gate"] = json!({"commit":commit,"path":".bboxy/runs/gate.json","sha256":entry::digest(&log)[7..],"task":{"kind":"record","root":"fixture.invalid/dream/repo","id":"01M00000000000000000000004","path":".bboxy/tasks/approval.md","commit":commit,"digest":entry::digest(&task.bytes)}});
    f.value["producers"] = json!([approval]);
    f.write();
    let expected: trust::Producer = serde_json::from_value(producer.clone()).unwrap();
    let trust = f.load().await.unwrap();
    trust
        .approve(
            &expected,
            &["receipt/1", "effect-wrapper/1", "no-replace/1"],
        )
        .await
        .unwrap();
    let sessions = f.repo.home.path().join("accepted-sessions");
    std::fs::create_dir(&sessions).unwrap();
    let root = bboxy450_harness::dream::resolve::Root::open(
        "fixture.invalid/dream/repo".into(),
        f.repo.path.clone(),
    )
    .await
    .unwrap();
    let resolver = bboxy450_harness::dream::resolve::Resolver::open(sessions, vec![root])
        .await
        .unwrap()
        .with_trust(f.load().await.unwrap());
    let tasks = resolver.accepted_tasks().await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].verify_exit, Some(0));
    let mut changed = expected.clone();
    changed.exe_sha256 = "b".repeat(64);
    assert!(trust.approve(&changed, &["receipt/1"]).await.is_err());
    changed = expected.clone();
    changed.commit = f.repo.commit.clone();
    assert!(
        trust.approve(&changed, &["receipt/1"]).await.is_err(),
        "old signed code is not an approved artifact"
    );
    assert!(trust.approve(&expected, &["unknown/1"]).await.is_err());
    let valid = f.value.clone();
    f.value["producers"][0]["gate"]["sha256"] = json!("b".repeat(64));
    f.write();
    assert!(
        f.load()
            .await
            .unwrap()
            .approve(&expected, &["receipt/1"])
            .await
            .is_err()
    );
    f.value = valid.clone();
    f.value["producers"][0]["unknown"] = json!(true);
    f.write();
    assert!(f.load().await.is_err(), "strict flattened policy fields");
    f.value = valid.clone();
    f.value["producers"][0]["capabilities"] = json!(["receipt/1"]);
    f.write();
    assert!(
        f.load()
            .await
            .unwrap()
            .approve(&expected, &["effect-wrapper/1"])
            .await
            .is_err()
    );
    // A signed gate with a failed required witness is independently rejected.
    f.value = valid;
    gate["witnesses"][1]["exit_code"] = json!(1);
    let bytes = serde_json::to_vec(&gate).unwrap();
    std::fs::write(f.repo.path.join(".bboxy/runs/gate.json"), &bytes).unwrap();
    f.repo.git(&["add", "."]);
    f.repo.git(&["commit", "-qm", "private failed witness"]);
    let commit = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    f.value["producers"][0]["gate"]["commit"] = json!(commit);
    f.value["producers"][0]["gate"]["sha256"] = json!(&entry::digest(&bytes)[7..]);
    f.write();
    assert!(
        f.load()
            .await
            .unwrap()
            .approve(&expected, &["effect-wrapper/1"])
            .await
            .is_err()
    );
}

#[test]
fn pinned_real_producer_record_is_a_byte_vector_only() {
    let bytes = include_bytes!("fixtures/dream/trust/producer-decision.blob");
    assert_eq!(
        entry::digest(bytes),
        "sha256:51e052c1f90d3322351848263d9ce1097a9f31e30860bf00d6bd95577c5f5a36"
    );
    let record = Record::parse(bytes).unwrap();
    trust::verify_bytes(
        bytes,
        record.front["id"].as_str().unwrap(),
        &["seat".into()],
    )
    .unwrap();
    // This vector carries no local policy or producer-artifact approval.
}
