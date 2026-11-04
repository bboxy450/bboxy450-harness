// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Private D1 fixtures; real signature and process boundaries, no operator state.
#![allow(dead_code)]
#[path = "../fixtures/dream/good/support.rs"]
mod good;
use bboxy450_harness::{
    dream::{
        d0,
        resolve::{Resolver, Root},
        trust::Trust,
    },
    protocol::*,
    redact::Set,
    store::Store,
};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
pub const REPO: &str = "fixture.invalid/dream/repo";
pub struct Fixture {
    pub repo: good::Repository,
    pub gpg: tempfile::TempDir,
    pub policy: PathBuf,
    pub sessions: PathBuf,
    pub episodes: Vec<d0::Episode>,
    pub fingerprint: String,
}
impl Fixture {
    // Adapted from tests/dream_trust.rs's independently signed, private-HOME fixture.
    pub fn gpg(&self, args: &[&str]) -> Vec<u8> {
        let out = Command::new("gpg")
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
            .env("HOME", self.repo.home.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "private gpg {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
    }
    pub async fn new() -> Self {
        let repo = good::Repository::new();
        let gpg = tempfile::Builder::new()
            .prefix("wp6-gpg-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(gpg.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut f = Self {
            policy: repo.home.path().join("trust.json"),
            sessions: repo.home.path().join("sessions"),
            repo,
            gpg,
            episodes: vec![],
            fingerprint: String::new(),
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
        let wrapper = f.repo.home.path().join("sign-gpg");
        std::fs::write(&wrapper,format!("#!/bin/sh\nexec /opt/homebrew/bin/gpg --no-options --homedir '{}' --batch --pinentry-mode loopback \"$@\"\n",f.gpg.path().display())).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
        f.repo.git(&["config", "gpg.format", "openpgp"]);
        f.repo
            .git(&["config", "gpg.program", wrapper.to_str().unwrap()]);
        f.repo.git(&["config", "user.signingkey", &fingerprint]);
        std::fs::create_dir_all(f.repo.path.join(".bboxy/tasks")).unwrap();
        for n in 0..3 {
            let id = format!("01M0000000000000000000010{n}");
            let mut front = json!({"id":id,"number":n+10,"type":"task","title":"Private fixture verdict","verify":"sh -c 'test -f fixed'","status":"accepted","verify_result":{"exit_code":0}});
            if n == 1 {
                front["status"] = json!("rejected");
                front["rejection"] = json!({"reason":"fixture absent"});
            }
            if n == 2 {
                front["status"] = json!("reported");
                front["verify_result"]["exit_code"] = json!(1);
            }
            let record = good::sign_record(
                front,
                "Synthetic task verdict. No live task or credentials.\n",
                &f.repo.key,
            );
            std::fs::write(
                f.repo.path.join(format!(".bboxy/tasks/synth-{n}.md")),
                record.bytes,
            )
            .unwrap();
        }
        f.repo.git(&["add", "."]);
        f.repo.git(&["commit", "-qm", "Private D1 task anchors"]);
        let commit = f.repo.git(&["rev-parse", "HEAD"]);
        f.repo
            .git(&["update-ref", "refs/remotes/origin/main", &commit]);
        let policy = json!({"schema":"bboxy.dream.trust/2","roots":[{"repo":REPO,"git_dir":f.repo.path,"branch":"refs/remotes/origin/main"}],"git_signers":[{"fingerprint":fingerprint,"public_key_file":key,"repos":[REPO],"revoked_after":null}],"kb_signer_kinds":["seat"],"producers":[]});
        std::fs::write(&f.policy, serde_json::to_vec(&policy).unwrap()).unwrap();
        std::fs::set_permissions(&f.policy, std::fs::Permissions::from_mode(0o600)).unwrap();
        for n in 0..3 {
            f.source(n).await;
        }
        let result = Box::pin(d0::replay_with_sessions(
            vec![f.sessions.clone()],
            f.repo.home.path().join("dream"),
            f.sessions.clone(),
            false,
            Set::default(),
        ))
        .await
        .unwrap();
        assert_eq!(result.episodes.len(), 3);
        f.episodes = result.episodes;
        f.fingerprint = f.episodes[0]
            .fingerprints
            .iter()
            .find(|fp| f.episodes.iter().all(|e| e.fingerprints.contains(*fp)))
            .unwrap()
            .clone();
        f
    }
    pub fn scope(&self) -> Scope {
        Scope {
            level: ScopeLevel::Repo,
            repos: vec![REPO.into()],
        }
    }
    pub async fn resolver(&self, trusted: bool) -> Resolver {
        let root = Root::open(REPO.into(), self.repo.path.clone())
            .await
            .unwrap();
        let mut resolver = Resolver::open(self.sessions.clone(), vec![root])
            .await
            .unwrap();
        if trusted {
            resolver = resolver.with_trust(
                Trust::load(
                    self.policy.clone(),
                    vec![self.repo.path.clone(), self.sessions.clone()],
                )
                .await
                .unwrap(),
            );
        }
        resolver
    }
    async fn source(&self, n: usize) {
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: self.repo.path.clone(),
            profile: Some("scripted".into()),
            fork: None,
            protocol: PROTOCOL,
            harness: "codex".into(),
            created_ms: 0,
            vcs: Some(Vcs {
                repo: Some(REPO.into()),
                commit: Some(self.repo.git(&["rev-parse", "HEAD"])),
                dirty: Some(false),
                ..Default::default()
            }),
            dream: None,
            source: None,
        };
        let mut store = Store::open(self.sessions.clone(), meta, None, Set::default())
            .await
            .unwrap();
        let turn = TurnId::new();
        store
            .append(Event::TurnQueued { turn }, None)
            .await
            .unwrap();
        store
            .append(
                Event::TurnStarted {
                    turn,
                    model: Some("synthetic".into()),
                    origin: None,
                },
                None,
            )
            .await
            .unwrap();
        for (name, input, text, status) in [
            (
                "Read",
                json!({"path":"fixture.txt"}),
                include_str!("../fixtures/dream/synth/tool-output.txt").to_owned(),
                ItemStatus::Failed {
                    reason: "missing".into(),
                    code: None,
                },
            ),
            (
                "bboxy_task_report",
                json!({"task":format!(".bboxy/tasks/synth-{n}.md")}),
                "recorded".into(),
                ItemStatus::Completed,
            ),
        ] {
            let item = ItemId::new();
            store
                .append(
                    Event::ItemStarted {
                        turn: Some(turn),
                        item,
                        kind: ItemKind::ToolCall,
                        parent: None,
                        head: Some(ItemHead::ToolCall {
                            call_id: item.to_string(),
                            name: name.into(),
                            tool_kind: ToolKind::Read,
                        }),
                    },
                    None,
                )
                .await
                .unwrap();
            store
                .append(
                    Event::ItemCompleted {
                        item,
                        status,
                        body: Item::ToolCall {
                            call_id: item.to_string(),
                            name: name.into(),
                            tool_kind: ToolKind::Read,
                            input_raw: input.to_string(),
                            output: vec![OutputBlock::Text { text }],
                            locations: vec![],
                            spill: None,
                            title: None,
                            server: None,
                            raw_output: None,
                        },
                    },
                    None,
                )
                .await
                .unwrap();
        }
        store
            .append(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::EndTurn,
                    usage: None,
                },
                None,
            )
            .await
            .unwrap();
        store.append(Event::SessionClosed, None).await.unwrap();
        bboxy450_harness::protocol::laws::check_authored(&store.events).unwrap();
    }
    pub fn candidate(&self, input: &bboxy450_harness::dream::prompt::Prompt) -> Value {
        let refs = input.references().keys().cloned().collect::<Vec<_>>();
        json!({"outcome":"candidates","candidates":[{"kind":"lesson","title":"Create the fixture before verification","description":"A missing fixture makes the check fail.","why":"The repeated missing fixture appears in the pinned accepted and failed episodes.","how_to_apply":"Create the private fixture, then run its verification command.","evidence":refs.into_iter().take(4).collect::<Vec<_>>(),"links":[],"scope_level":"repo","keywords":["fixture"]}]})
    }
    pub fn source_bytes(&self) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        std::fs::read_dir(&self.sessions)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .map(|p| {
                let bytes = std::fs::read(&p).unwrap();
                (p, bytes)
            })
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = Command::new("gpgconf")
            .arg("--homedir")
            .arg(self.gpg.path())
            .args(["--kill", "gpg-agent"])
            .env("HOME", self.repo.home.path())
            .output();
    }
}
pub fn sse(value: &Value) -> Vec<u8> {
    format!(": Synthetic bboxy450-harness fixture; policy https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md\ndata: {}\n\ndata: [DONE]\n\n",json!({"choices":[{"delta":{"content":value.to_string()},"finish_reason":"stop"}]})).into_bytes()
}

pub async fn proposal_store(f: &Fixture, scope: Scope) -> Store {
    let meta = SessionMeta {
        session: SessionId::new(),
        cwd: f.repo.path.clone(),
        profile: Some("scripted".into()),
        fork: None,
        protocol: PROTOCOL,
        harness: "dream".into(),
        created_ms: 0,
        vcs: None,
        source: None,
        dream: Some(DreamRun {
            phase: DreamPhase::Synthesis,
            trigger: Trigger::Operator { message: None },
            scope,
            parent: None,
            prompt_version: Some(bboxy450_harness::dream::prompt::VERSION.into()),
            thresholds: Some(bboxy450_harness::dream::prompt::THRESHOLDS.into()),
        }),
    };
    Store::open(f.sessions.clone(), meta, None, Set::default())
        .await
        .unwrap()
}
pub fn private_command(f: &Fixture) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("BBOXY_")
            || name.starts_with("GIT_")
            || name.contains("API_KEY")
            || name == "OPENAI_API_KEY"
        {
            cmd.env_remove(key);
        }
    }
    cmd.current_dir(&f.repo.path)
        .env("HOME", f.repo.home.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

pub fn sse_usage(value: &Value) -> Vec<u8> {
    format!(": Synthetic bboxy450-harness fixture; policy https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md\ndata: {}\n\ndata: [DONE]\n\n",json!({"choices":[{"delta":{"content":value.to_string()},"finish_reason":"stop"}],"usage":{"prompt_tokens":17,"completion_tokens":9}})).into_bytes()
}
