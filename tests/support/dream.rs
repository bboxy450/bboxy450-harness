// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Private D1 fixtures; real signature and process boundaries, no operator state.
#![allow(dead_code)]
#[path = "../fixtures/dream/good/support.rs"]
pub(crate) mod good;
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
        f.episodes = replay_explicit_sources(&f, f.repo.home.path().join("dream")).await;
        assert_eq!(f.episodes.len(), 3);
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
        if std::thread::panicking() {
            // Retain ONLY this test's synthetic public repo/log/cache custody.
            // Private disposable GPG keys are outside this directory and still drop.
            if let Ok(empty) = tempfile::Builder::new()
                .prefix("wp6-empty-")
                .tempdir_in("/tmp")
            {
                let home = std::mem::replace(&mut self.repo.home, empty).keep();
                let _ = std::fs::remove_file(home.join("git-signing"));
                let _ = std::fs::remove_file(home.join("git-signing.pub"));
                eprintln!("synthetic D1 failure custody: {}", home.display());
            }
        }
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

/// Build retained fixture inputs through the real bounded D0 API, once
/// per explicit source log. Aggregate replay latency is tested separately by
/// trial_d0_twelve_log_replay_keeps_pins_cache_bytes_and_source_identity.
async fn replay_explicit_sources(f: &Fixture, root: PathBuf) -> Vec<d0::Episode> {
    let before = f.source_bytes();
    let mut episodes = vec![];
    for (index, path) in before.keys().enumerate() {
        let replay = Box::pin(d0::replay_with_sessions(
            vec![path.clone()],
            root.clone(),
            f.sessions.clone(),
            index == 0,
            Set::default(),
        ))
        .await
        .unwrap();
        assert_eq!(replay.busy, 0);
        assert_eq!(replay.skipped, 0);
        assert!(replay.max_poll_cpu_ns <= 1_000_000);
        episodes.extend(replay.episodes);
    }
    episodes.sort_by_cached_key(|episode| (episode.repo.clone(), serde_json::to_string(&episode.origin).unwrap()));
    assert_eq!(f.source_bytes(), before);
    episodes
}

/// Synthetic retained source episodes and real signed task observations. Trial
/// continuations themselves use the ordinary engine and confined actual tools.
pub async fn trial_fixture() -> (Fixture, EvidenceRef) {
    trial_fixture_timing(|_| {}).await
}
pub async fn trial_fixture_timing(
    change: impl Fn(&mut serde_json::Value),
) -> (Fixture, EvidenceRef) {
    use bboxy450_harness::{
        dream::{fingerprint, prompt, synth},
        provider::ScriptedProvider,
    };
    let mut f = Box::pin(Fixture::new()).await;
    std::fs::create_dir_all(f.repo.path.join(".bboxy/runs")).unwrap();
    for n in 0..9 {
        let began = std::time::Instant::now();
        let exit = Command::new("/bin/sh")
            .args(["-c", "test -f fixed"])
            .current_dir(&f.repo.path)
            .env_clear()
            .status()
            .unwrap()
            .code()
            .unwrap();
        let elapsed: u64 = began.elapsed().as_millis().try_into().unwrap();
        assert_eq!(exit, 1);
        let task_id = format!("01M0000000000000000000020{n}");
        let task_path = format!(".bboxy/tasks/trial-{n}.md");
        let log_path = format!(".bboxy/runs/trial-original-{n}.json");
        let log=serde_json::to_vec(&json!({"schema":"bboxy.dream.original-verify/1","task":{"id":task_id,"path":task_path},"command":"sh -c 'test -f fixed'","exit_code":exit,"elapsed_ms":elapsed})).unwrap();
        let log_digest = bboxy450_harness::dream::entry::digest(&log);
        std::fs::write(f.repo.path.join(&log_path), log).unwrap();
        let mut front = json!({"type":"task","id":format!("01M0000000000000000000020{n}"),"number":100+n,"title":"Private original verifier failure","verify":"sh -c 'test -f fixed'","status":"reported","verify_result":{"exit_code":exit,"elapsed_ms":elapsed,"run_log":{"path":log_path,"digest":log_digest}}});
        change(&mut front);
        let record = good::sign_record(
            front,
            "Synthetic original observation. No live job or operator identity.\n",
            &f.repo.key,
        );
        std::fs::write(
            f.repo.path.join(format!(".bboxy/tasks/trial-{n}.md")),
            record.bytes,
        )
        .unwrap();
    }
    f.repo.git(&["add", ".bboxy/tasks", ".bboxy/runs"]);
    f.repo
        .git(&["commit", "-qm", "Private trial task observations"]);
    let commit = f.repo.git(&["rev-parse", "HEAD"]);
    f.repo
        .git(&["update-ref", "refs/remotes/origin/main", &commit]);
    assert!(
        f.repo
            .git(&["--no-optional-locks", "status", "--porcelain"])
            .is_empty()
    );
    for n in 0..9 {
        let meta = SessionMeta {
            session: SessionId::new(),
            cwd: f.repo.path.clone(),
            profile: Some("scripted".into()),
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: 0,
            vcs: Some(Vcs {
                repo: Some(REPO.into()),
                commit: Some(commit.clone()),
                dirty: Some(false),
                ..Default::default()
            }),
            dream: None,
            source: None,
        };
        let mut store = Store::open(f.sessions.clone(), meta, None, Set::default())
            .await
            .unwrap();
        store
            .append(
                Event::ProfileChanged {
                    profile: "scripted".into(),
                    model: "scripted".into(),
                },
                None,
            )
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
                    model: Some("scripted".into()),
                    origin: Some(TurnOrigin::User),
                },
                None,
            )
            .await
            .unwrap();
        let body = Item::UserMessage {
            blocks: vec![InputBlock::Text {
                text: if n < 7 {
                    "Restore the missing private fixture."
                } else {
                    "CONTROL: keep the private fixture available."
                }
                .into(),
            }],
        };
        trial_item(&mut store, Some(turn), body, ItemStatus::Completed).await;
        for (name, input, text, status) in [
            (
                "read",
                json!({"path":"fixed"}),
                if n < 7 {
                    "missing private fixture"
                } else {
                    "distinct control clue"
                }
                .to_owned(),
                ItemStatus::Failed {
                    reason: "missing".into(),
                    code: None,
                },
            ),
            (
                "bboxy_task_report",
                json!({"task_path":format!(".bboxy/tasks/trial-{n}.md")}),
                "synthetic retained task observation".into(),
                ItemStatus::Completed,
            ),
        ] {
            let body = Item::ToolCall {
                call_id: ItemId::new().to_string(),
                name: name.into(),
                tool_kind: ToolKind::Read,
                input_raw: input.to_string(),
                output: vec![OutputBlock::Text { text }],
                locations: vec![],
                spill: None,
                title: None,
                raw_output: None,
                server: None,
            };
            trial_item(&mut store, Some(turn), body, status).await;
        }
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 10,
            source: Some(NumberSource::Reported),
            ..Default::default()
        }
        .authored(
            UsageScope::Call,
            Backend {
                kind: BackendKind::Engine,
                name: "scripted".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            },
            Some("scripted".into()),
        );
        trial_item(
            &mut store,
            Some(turn),
            Item::Usage {
                usage: usage.clone(),
            },
            ItemStatus::Completed,
        )
        .await;
        store
            .append(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::EndTurn,
                    usage: Usage::sum_calls(&[usage]).unwrap(),
                },
                None,
            )
            .await
            .unwrap();
        store.append(Event::SessionClosed, None).await.unwrap();
    }
    f.episodes = replay_explicit_sources(&f, f.repo.home.path().join("trial-replay")).await;
    f.fingerprint = fingerprint::tool_fail("read", "missing private fixture");
    assert_eq!(
        f.episodes
            .iter()
            .filter(|e| e.fingerprints.contains(&f.fingerprint))
            .count(),
        7
    );
    let mut resolver = f.resolver(true).await;
    let input = prompt::prepare(
        &mut resolver,
        &f.episodes,
        &f.fingerprint,
        f.scope(),
        &Set::default(),
    )
    .await
    .unwrap()
    .unwrap();
    let mut candidate = f.candidate(&input);
    let refs = input
        .references()
        .iter()
        .filter(|(_, p)| matches!(p, EvidenceRef::Event { .. }))
        .take(1)
        .chain(
            input
                .references()
                .iter()
                .filter(|(_, p)| matches!(p, EvidenceRef::Record { .. }))
                .take(1),
        )
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    candidate["candidates"][0]["evidence"] = json!(refs);
    let mut store = proposal_store(&f, f.scope()).await;
    synth::run(
        &mut store,
        std::sync::Arc::new(ScriptedProvider::new(vec![sse(&candidate)])),
        "scripted".into(),
        input,
        &mut resolver,
        Set::default(),
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .unwrap();
    let event = store
        .events
        .iter()
        .find(|e| {
            matches!(
                e.event,
                Event::ItemCompleted {
                    body: Item::MemoryWrite { .. },
                    ..
                }
            )
        })
        .unwrap();
    let bytes = event.wire_bytes().unwrap();
    let pin = EvidenceRef::Event {
        session: event.session,
        seq: event.seq.unwrap(),
        digest: bboxy450_harness::dream::entry::digest(bytes.strip_suffix(b"\n").unwrap()),
    };
    store.append(Event::SessionClosed, None).await.unwrap();
    drop(store);
    (f, pin)
}
pub async fn trial_item(store: &mut Store, turn: Option<TurnId>, body: Item, status: ItemStatus) {
    let item = ItemId::new();
    let head = if let Item::ToolCall {
        call_id,
        name,
        tool_kind,
        ..
    } = &body
    {
        Some(ItemHead::ToolCall {
            call_id: call_id.clone(),
            name: name.clone(),
            tool_kind: tool_kind.clone(),
        })
    } else {
        None
    };
    store
        .append(
            Event::ItemStarted {
                turn,
                item,
                kind: body.kind(),
                parent: None,
                head,
            },
            None,
        )
        .await
        .unwrap();
    store
        .append(Event::ItemCompleted { item, status, body }, None)
        .await
        .unwrap();
}

// d20261103136610: each phase has a fresh process/OnceLock. No warmup child,
// selected minimum or test-order dependence substitutes for the ready witness.
pub async fn process_poll_modes() {
    use bboxy450_harness::process::{Request, SettlementStatus, observe_capture};
    let executable = std::env::current_exe().unwrap();
    let mut observations = vec![];
    for mode in ["initialization-only", "cold-api", "ready"] {
        let private = tempfile::Builder::new()
            .prefix("wp7-poll-")
            .tempdir_in("/tmp")
            .unwrap();
        let args = vec![
            "-i".into(),
            format!("HOME={}", private.path().display()),
            format!("TMPDIR={}", private.path().display()),
            "PATH=/usr/bin:/bin".into(),
            format!("DREAM_WP7_CPU_MODE={mode}"),
            executable.to_string_lossy().into_owned(),
            "--exact".into(),
            "synth_d0_process_spawn_poll_bound_witness".into(),
            "--nocapture".into(),
        ];
        let mut operation = observe_capture(
            Request {
                program: "/usr/bin/env".into(),
                args,
                stdin: None,
                cwd: private.path().into(),
                timeout: std::time::Duration::from_secs(30),
                spill_dir: private.path().join("unused"),
                cap: 64 * 1024,
                redactor: Set::default(),
            },
            tokio_util::sync::CancellationToken::new(),
        );
        let settlement = operation.settlement();
        let result = operation.result().await;
        let status = settlement.wait().await;
        let row = match result {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.bytes);
                let measurement = text.lines().find_map(|line| {
                    line.split_once("dream-wp7-poll:")
                        .and_then(|(_, j)| serde_json::from_str::<Value>(j).ok())
                });
                json!({"mode":mode,"child_exit_code":output.code,"cancelled":output.cancelled,"settled":status==SettlementStatus::Confirmed,"measurement":measurement})
            }
            Err(error) => {
                json!({"mode":mode,"child_exit_code":null,"settled":status==SettlementStatus::Confirmed,"error":error.to_string()})
            }
        };
        println!("dream-wp7-poll-child:{row}");
        observations.push(row);
    }
    let record = json!({"verify_label":std::env::var("DREAM_WP7_VERIFY_LABEL").ok(),"executable_sha256":bboxy450_harness::dream::entry::digest(std::fs::read(executable).unwrap()),"observations":observations});
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".bboxy/runs")
        .join(format!("dream-wp7-poll-modes-{}.json", ulid::Ulid::new()));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    file.write_all(&serde_json::to_vec_pretty(&record).unwrap())
        .unwrap();
    file.write_all(b"\n").unwrap();
    assert!(
        observations.iter().all(|r| r["child_exit_code"] == 0
            && r["settled"] == true
            && r["cancelled"].as_bool() != Some(true)
            && r["measurement"]["mode"] == r["mode"]),
        "fresh mode failed; ALL observations retained at {}: {}",
        path.display(),
        record
    );
}
