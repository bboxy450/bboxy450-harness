// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use base64::Engine;
use bboxy450_harness::dream::entry::{Record, digest as entry_digest};
use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};
use serde_json::{Value, json};

pub const SIGNER: &str = "seat:01M00000000000000000000001";
pub fn key_pair() -> Ed25519KeyPair {
    let generated = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    Ed25519KeyPair::from_pkcs8(generated.as_ref()).unwrap()
}
// Adapted from bboxy450-term b2f4b0c index.rs:1044-1073.
// Policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
pub fn sign_record(mut front: Value, body: &str, key: &Ed25519KeyPair) -> Record {
    front.as_object_mut().unwrap().remove("provenance");
    front["created_by"] = json!(SIGNER);
    front["updated_by"] = json!(SIGNER);
    front["authorship"] = json!({"signer":SIGNER,"kind":"seat","process_generation":"1",
        "public_key":base64::engine::general_purpose::STANDARD.encode(key.public_key().as_ref())});
    let body_digest = entry_digest(body.as_bytes());
    let front_digest = entry_digest(serde_yaml::to_string(&front).unwrap().as_bytes());
    let payload = format!(
        "{}{}",
        front_digest.strip_prefix("sha256:").unwrap(),
        body_digest.strip_prefix("sha256:").unwrap()
    );
    let domain = b"bboxy/item/v1";
    let mut pae = format!("DSSEv1 {} ", domain.len()).into_bytes();
    pae.extend(domain);
    pae.extend(format!(" {} ", payload.len()).as_bytes());
    pae.extend(payload.as_bytes());
    front["provenance"] = json!({"digest":body_digest,"frontmatter_digest":front_digest,"signer":SIGNER,
        "sig":base64::engine::general_purpose::STANDARD.encode(key.sign(&pae).as_ref()),"log":".bboxy/log/toy.jsonl#1"});
    Record::parse(format!("---\n{}---\n{body}", serde_yaml::to_string(&front).unwrap()).as_bytes())
        .unwrap()
}
fn anchor_record(key: &Ed25519KeyPair) -> Record {
    sign_record(
        json!({"id":"01M00000000000000000000002","number":1,"type":"decision","title":"Toy proof"}),
        "signed fixture body\n",
        key,
    )
}
pub fn signed_record() -> Record {
    anchor_record(&key_pair())
}

// Reuses WP3's private signed-Git fixture pattern; no operator identity or key is opened.
pub struct Repository {
    pub home: tempfile::TempDir,
    pub path: std::path::PathBuf,
    pub commit: String,
    pub bytes: Vec<u8>,
    pub key: Ed25519KeyPair,
}
impl Repository {
    pub fn git(&self, args: &[&str]) -> String {
        let mut command = std::process::Command::new("git");
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                command.env_remove(key);
            }
        }
        let out = command
            .args(["--no-pager", "-c", "core.hooksPath=/dev/null"])
            .args(args)
            .current_dir(&self.path)
            .env("HOME", self.home.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "private git fixture failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().into()
    }
    pub fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join("repo");
        std::fs::create_dir(&path).unwrap();
        let kb_key = key_pair();
        let bytes = anchor_record(&kb_key).bytes;
        let mut fixture = Self {
            home,
            path,
            commit: String::new(),
            bytes,
            key: kb_key,
        };
        let key = fixture.home.path().join("git-signing");
        let status = std::process::Command::new("/usr/bin/ssh-keygen")
            .args([
                "-q",
                "-t",
                "ed25519",
                "-N",
                "",
                "-C",
                "Boxy <bboxy450@gmail.com>",
                "-f",
            ])
            .arg(&key)
            .status()
            .unwrap();
        assert!(status.success());
        fixture.git(&["init", "-q"]);
        fixture.git(&["config", "user.name", "Boxy"]);
        fixture.git(&["config", "user.email", "bboxy450@gmail.com"]);
        fixture.git(&["config", "commit.gpgsign", "true"]);
        fixture.git(&["config", "gpg.format", "ssh"]);
        fixture.git(&["config", "user.signingkey", key.to_str().unwrap()]);
        fixture.git(&[
            "remote",
            "add",
            "origin",
            "https://fixture.invalid/dream/repo.git",
        ]);
        std::fs::create_dir_all(fixture.path.join(".bboxy/decisions")).unwrap();
        std::fs::write(
            fixture.path.join(".bboxy/decisions/my proof.md"),
            &fixture.bytes,
        )
        .unwrap();
        std::fs::write(
            fixture.path.join(".bboxy/dream.toml"),
            "cross_repo = true\n",
        )
        .unwrap();
        fixture.git(&["add", "."]);
        fixture.git(&["commit", "-qm", "fixture anchor"]);
        fixture.commit = fixture.git(&["rev-parse", "HEAD"]);
        fixture.git(&["update-ref", "refs/remotes/origin/main", &fixture.commit]);
        fixture
    }
    pub fn pin(&self) -> bboxy450_harness::protocol::RecordPin {
        bboxy450_harness::protocol::RecordPin {
            root: "fixture.invalid/dream/repo".into(),
            id: Some("01M00000000000000000000002".into()),
            path: ".bboxy/decisions/my proof.md".into(),
            commit: Some(self.commit.clone()),
            digest: entry_digest(&self.bytes),
        }
    }
}
