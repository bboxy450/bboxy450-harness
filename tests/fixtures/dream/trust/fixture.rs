// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::support::{Repository, sign_record};
use bboxy450_harness::{
    dream::{
        entry,
        trust::{self, Trust},
    },
    protocol::*,
};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
pub struct Fixture {
    pub repo: Repository,
    pub gpg: tempfile::TempDir,
    pub policy: PathBuf,
    pub value: Value,
    pub pin: EvidenceRef,
}
impl Fixture {
    pub fn gpg(&self, args: &[&str]) -> Vec<u8> {
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
    pub fn new() -> Self {
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
    pub fn write(&self) {
        std::fs::write(&self.policy, serde_json::to_vec(&self.value).unwrap()).unwrap();
        std::fs::set_permissions(&self.policy, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    pub async fn load(&self) -> Result<Trust, entry::Invalid> {
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

impl Fixture {
    /// Isolated policy only: source -> artifact identity -> gate -> full report -> task -> policy.
    /// This does not install/approve a real harness producer.
    pub fn approve_fixture(&mut self) -> trust::Producer {
        let source = self.repo.git(&["rev-parse", "HEAD"]);
        let tree = self.repo.git(&["rev-parse", "HEAD^{tree}"]);
        let exe =
            entry::digest(std::fs::read(std::env::current_exe().unwrap()).unwrap())[7..].to_owned();
        let producer = json!({"repo":"fixture.invalid/dream/repo","commit":source,"tree":tree,"version":"test/1","build":"test/private","exe_sha256":exe,"capabilities":["receipt/1","effect-wrapper/1","no-replace/1","constructive-seal/1"]});
        let mut gate = producer.clone();
        gate["schema"] = json!("bboxy.dream.producer-gate/1");
        gate["witnesses"] = json!([
            {"id":"dream_trust::receipt_header_body_roundtrip","exit_code":0},
            {"id":"dream_sinks::receipt_ops_match_independent_observer","exit_code":0},
            {"id":"dream_sinks::hand_write_between_read_and_link_preserved","exit_code":0},
            {"id":"dream_sinks::overwrite_rename_control_fails_audit","exit_code":0},
            {"id":"dream_trust::receipt_seal_constructs_without_self_hash","exit_code":0},
            {"id":"dream_sinks::late_owned_source_write_after_seal_rejected","exit_code":0},
            {"id":"dream_sinks::broken_seal_exception_fails_audit","exit_code":0}]);
        std::fs::create_dir_all(self.repo.path.join(".bboxy/runs")).unwrap();
        std::fs::create_dir_all(self.repo.path.join(".bboxy/reports")).unwrap();
        std::fs::create_dir_all(self.repo.path.join(".bboxy/tasks")).unwrap();
        let bytes = serde_json::to_vec(&gate).unwrap();
        std::fs::write(self.repo.path.join(".bboxy/runs/gate.json"), &bytes).unwrap();
        self.repo.git(&["add", "."]);
        self.repo
            .git(&["commit", "-qm", "private measured artifact gate"]);
        let g = self.repo.git(&["rev-parse", "HEAD"]);
        let report = sign_record(
            json!({"id":"01M00000000000000000000005","number":5,"type":"report","task":"[[01M00000000000000000000004]]","commits":[source,g]}),
            "isolated gate report; no live approval\n",
            &self.repo.key,
        );
        std::fs::write(self.repo.path.join(".bboxy/reports/gate.md"), &report.bytes).unwrap();
        self.repo.git(&["add", "."]);
        self.repo
            .git(&["commit", "-qm", "private full gate report"]);
        let r = self.repo.git(&["rev-parse", "HEAD"]);
        let task = sign_record(
            json!({"id":"01M00000000000000000000004","number":4,"type":"task","title":"fixture approval","status":"accepted","verify_result":{"exit_code":0},"report":{"commit":r,"report":"[[reports/gate]]"}}),
            "accepted private artifact fixture\n",
            &self.repo.key,
        );
        std::fs::write(self.repo.path.join(".bboxy/tasks/approval.md"), &task.bytes).unwrap();
        self.repo.git(&["add", "."]);
        self.repo
            .git(&["commit", "-qm", "private accepted gate task"]);
        let t = self.repo.git(&["rev-parse", "HEAD"]);
        self.repo
            .git(&["update-ref", "refs/remotes/origin/main", &t]);
        let mut approval = producer.clone();
        approval["gate"] = json!({"commit":g,"path":".bboxy/runs/gate.json","sha256":&entry::digest(&bytes)[7..],"task":{"kind":"record","root":"fixture.invalid/dream/repo","id":"01M00000000000000000000004","path":".bboxy/tasks/approval.md","commit":t,"digest":entry::digest(&task.bytes)}});
        self.value["producers"] = json!([approval]);
        self.write();
        serde_json::from_value(producer).unwrap()
    }
}
