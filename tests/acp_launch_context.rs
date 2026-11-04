// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Real Host + downstream JSON-RPC + real throwaway MCP calls; no vendor inference.
#![cfg(feature = "acp-agent")]
mod support;
use bboxy450_harness::{
    engine::{Config, LaunchOptions},
    gateway::{Gateway, Host, InProc},
    mcp::{ServerConfig, Servers},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
const CONTEXT: &str = "NAME: Private launch\nROLE: EXECUTOR\nTask: private-parity\nWorkspace: quoted directory\nWorkflow: bboxy450 scoped delegation";
fn script() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/acp-agent/launch_parity.py")
        .display()
        .to_string()
}
// Each test owns a whole process environment; never resolves the real seat's capability.
fn isolated(name: &str) -> bool {
    if std::env::var_os("PARITY_CHILD").is_some() {
        return false;
    }
    let home = tempfile::Builder::new()
        .prefix("launch-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    let cap = home.path().join("capability");
    std::fs::write(
        &cap,
        format!(
            "{:032x}{:032x}\n",
            u128::from(ulid::Ulid::new()),
            u128::from(ulid::Ulid::new())
        ),
    )
    .unwrap();
    std::fs::set_permissions(&cap, std::fs::Permissions::from_mode(0o600)).unwrap();
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    for (key, _) in std::env::vars_os() {
        if key.as_encoded_bytes().starts_with(b"BBOXY_") {
            command.env_remove(key);
        }
    }
    let output = command
        .args(["--exact", name, "--nocapture"])
        .env("PARITY_CHILD", "1")
        .env("BBOXY450_ACP_STARTUP_MS", "5000")
        .env("HOME", home.path())
        .env("BBOXY_SEAT_ID", "private-launch-seat")
        .env("BBOXY_ROLE", "executor")
        .env("BBOXY_CAPABILITY_FILE", &cap)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "isolated witness failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
struct Fixture {
    temp: tempfile::TempDir,
    config: Config,
    profiles: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let profiles = temp.path().join("profiles");
        std::fs::create_dir(&profiles).unwrap();
        let cwd = temp.path().join("quoted workspace ' $ literal");
        std::fs::create_dir(&cwd).unwrap();
        let config = Config {
            meta: support::meta(cwd),
            sessions: temp.path().join("sessions"),
            profiles: profiles.clone(),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        };
        Self {
            temp,
            config,
            profiles,
        }
    }
    fn server(&self, origin: &str) -> ServerConfig {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(
            std::fs::read(std::env::var_os("BBOXY_CAPABILITY_FILE").unwrap()).unwrap(),
        );
        ServerConfig {
            command: "/usr/bin/python3".into(),
            args: vec!["-I".into(), script(), "server".into()],
            env: [
                ("PARITY_ORIGIN".into(), format!("fixture-origin:{origin}")),
                ("PARITY_CAP_HASH".into(), format!("{digest:x}")),
                (
                    "PARITY_EFFECTS".into(),
                    self.temp.path().join("effects").display().to_string(),
                ),
            ]
            .into(),
            timeout_s: 1,
            trust_hints: false,
        }
    }
    fn profile(&self, name: &str, model: &str, servers: Servers) {
        let command = vec![
            "/usr/bin/python3".to_owned(),
            "-I".into(),
            script(),
            "agent".into(),
            self.temp.path().join(name).display().to_string(),
            name.into(),
            "--vendor-owned-permission=ask".into(),
            "literal ' \" $ args".into(),
        ];
        let p = json!({"name":name,"provider":"acp-agent","model":model,"agent":{"command":command},"on_unattended":"deny","policy":if name=="claude-exec" {"yolo"}else{"ask"},"mcpServers":servers});
        std::fs::write(
            self.profiles.join(format!("{name}.md")),
            format!("---\n{}---\n", serde_yaml::to_string(&p).unwrap()),
        )
        .unwrap();
    }
    fn host(&self, context: &str, servers: Servers) -> Host {
        Host::new(self.config.clone()).with_launch(LaunchOptions {
            yolo: false,
            append_system_prompt: context.into(),
            mcp_servers: servers,
            notices: vec![],
            trial: None,
        })
    }
    async fn create(&self, host: &Host, name: &str, overrides: Servers) -> InProc {
        host.create_with_mcp(
            OpEnvelope {
                v: PROTOCOL,
                session: SessionId::new(),
                op_id: OpId::new(),
                op: Op::StartSession {
                    cwd: self.config.meta.cwd.clone(),
                    profile: Some(name.into()),
                    fork: None,
                },
            },
            overrides,
        )
        .await
        .unwrap()
        .0
    }
    fn journal(&self, name: &str) -> Vec<Value> {
        std::fs::read_to_string(self.temp.path().join(name))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
}
async fn turn(g: &InProc, request: &str) -> Vec<Envelope> {
    let hello = g.hello();
    let mut sub = g
        .subscribe(bboxy450_harness::gateway::Subscribe {
            session: g.session(),
            after: Some((hello.epoch, hello.head)),
            ephemeral: true,
        })
        .unwrap();
    g.send(support::prompt(g, request)).unwrap();
    let mut events = vec![];
    loop {
        let e = tokio::time::timeout(Duration::from_secs(40), sub.recv())
            .await
            .unwrap()
            .unwrap();
        let end = matches!(e.event, Event::TurnEnded { .. });
        events.push(e);
        if end {
            break;
        }
    }
    events
}
fn reply(events: &[Envelope]) -> Value {
    let text = events
        .iter()
        .find_map(|e| {
            if let Event::ItemCompleted {
                body: Item::AgentMessage { markdown },
                ..
            } = &e.event
            {
                Some(markdown)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("downstream process did not complete: {events:?}"));
    serde_json::from_str(text).unwrap()
}
fn assert_context(journal: &[Value], expected: &str, turns: usize) {
    let prompts: Vec<_> = journal
        .iter()
        .filter(|r| r["method"] == "session/prompt")
        .collect();
    assert_eq!(
        prompts.len(),
        turns,
        "synthetic or duplicate downstream prompt"
    );
    let blocks = prompts[0]["params"]["prompt"].as_array().unwrap();
    assert!(
        blocks
            .iter()
            .any(|b| b["text"].as_str().is_some_and(|t| t.contains(expected))),
        "missing admitted launcher context"
    );
}
#[tokio::test]
async fn downstream_context_and_callable_mcp() {
    if isolated("downstream_context_and_callable_mcp") {
        return;
    }
    let f = Fixture::new();
    bboxy450_harness::capability::register(&f.config.redactor).await;
    for name in ["codex-exec", "codex-acp", "claude-exec", "claude-acp"] {
        f.profile(
            name,
            "model-A",
            [("seat".into(), f.server("profile"))].into(),
        );
        let mut launcher = f.server("launcher");
        launcher.env.insert(
            "PARITY_REFLECT".into(),
            "fixture-reflection-sentinel".into(),
        );
        let host = f.host(CONTEXT, [("seat".into(), launcher)].into());
        let g = f
            .create(&host, name, [("seat".into(), f.server("ACP"))].into())
            .await;
        let events = turn(&g, "actual request").await;
        let result = reply(&events);
        assert_eq!(
            result["tools"]["seat"]["seat"], "private-launch-seat",
            "missing callable admitted MCP server"
        );
        assert_eq!(
            result["tools"]["seat"]["launcher"], true,
            "launcher precedence lost"
        );
        assert_eq!(
            result["permission"],
            if name == "claude-exec" { "yes" } else { "no" },
            "approval semantics changed"
        );
        assert_eq!(
            result["tools"]["seat"]["probe"], "<redacted>",
            "capability reached downstream model content"
        );
        assert_eq!(
            result["tools"]["seat"]["custody"], true,
            "lost private file capability custody"
        );
        let j = f.journal(name);
        assert_context(&j, CONTEXT, 1);
        assert_eq!(
            j[0]["cwd"],
            f.config
                .meta
                .cwd
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
        );
        assert_eq!(j[0]["argv"][2], "literal ' \" $ args");
        assert!(
            j.iter().any(|r| r["method"] == "session/set_config_option"
                && r["params"]["value"] == "fixture/model-A"),
            "model semantics dropped"
        );
        let new = j.iter().find(|r| r["method"] == "session/new").unwrap();
        assert_eq!(
            new["params"]["mcpServers"].as_array().unwrap().len(),
            1,
            "empty downstream mcpServers"
        );
        assert!(new.to_string().contains("/usr/bin/nc"));
        assert!(!new.to_string().contains("BBOXY_CAPABILITY"));
        assert!(
            std::fs::read_to_string(f.config.sessions.join(format!("{}.jsonl", g.session())))
                .unwrap()
                .contains("launcher configuration overrides ACP")
        );
        let events2 = turn(&g, "second actual request").await;
        reply(&events2);
        let j = f.journal(name);
        assert_context(&j, CONTEXT, 2);
        assert!(
            !j.iter()
                .filter(|r| r["method"] == "session/prompt")
                .nth(1)
                .unwrap()
                .to_string()
                .contains(CONTEXT),
            "bootstrap duplicated in live session"
        );
        host.close_all().await;
        let path = f.config.sessions.join(format!("{}.jsonl", g.session()));
        let bytes = std::fs::read_to_string(path).unwrap();
        let log: Vec<Envelope> = bytes
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        bboxy450_harness::protocol::laws::check_authored(&log).unwrap();
        assert_eq!(
            log.iter()
                .filter(|e| matches!(e.event, Event::TurnStarted { .. }))
                .count(),
            2
        );
        assert_eq!(
            log.iter()
                .filter(|e| matches!(
                    e.event,
                    Event::ItemCompleted {
                        body: Item::UserMessage { .. },
                        ..
                    }
                ))
                .count(),
            2
        );
        let cap =
            std::fs::read_to_string(std::env::var_os("BBOXY_CAPABILITY_FILE").unwrap()).unwrap();
        assert!(!bytes.contains(cap.trim()));
        assert!(!serde_json::to_string(&j).unwrap().contains(cap.trim()));
    }
    assert_eq!(
        std::fs::read_to_string(f.temp.path().join("effects"))
            .unwrap()
            .lines()
            .count(),
        8,
        "duplicate tool effect"
    );
}
#[tokio::test]
async fn reopen_switch_and_generation_context() {
    if isolated("reopen_switch_and_generation_context") {
        return;
    }
    let f = Fixture::new();
    for name in ["A", "B"] {
        f.profile(
            name,
            if name == "A" { "model-A" } else { "model-B" },
            Servers::new(),
        );
    }
    let host = f.host(
        "generation one",
        [("seat".into(), f.server("launcher"))].into(),
    );
    let a = f.create(&host, "A", Servers::new()).await;
    reply(&turn(&a, "first").await);
    let b = host.switch(&a, "B".into()).await.unwrap();
    reply(&turn(&b, "second").await);
    let a2 = host.switch(&b, "A".into()).await.unwrap();
    reply(&turn(&a2, "third").await);
    assert_ne!(a.session(), a2.session());
    let old_new = f
        .journal("A")
        .into_iter()
        .find(|r| r["method"] == "session/new")
        .unwrap();
    let old_path = PathBuf::from(
        old_new["params"]["mcpServers"][0]["args"][1]
            .as_str()
            .unwrap(),
    );
    for _ in 0..100 {
        if !old_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        !old_path.exists(),
        "retired generation kept its relay/grants"
    );
    let b_model = f.journal("B");
    assert!(
        b_model
            .iter()
            .any(|r| r["method"] == "session/set_config_option"
                && r["params"]["value"] == "fixture/model-B")
    );
    let session = a2.session();
    // Abrupt runtime teardown is exercised in existing acp_agent/seat tests.
    // Drop the actor task via process termination would also terminate this test.
    // Reopen from a private copy of the live durable log under a new Host generation.
    let sessions = f.temp.path().join("reopened");
    std::fs::create_dir(&sessions).unwrap();
    std::fs::copy(
        f.config.sessions.join(format!("{session}.jsonl")),
        sessions.join(format!("{session}.jsonl")),
    )
    .unwrap();
    host.close_all().await;
    let mut config = f.config.clone();
    config.sessions = sessions;
    let next = Host::new(config).with_launch(LaunchOptions {
        yolo: false,
        append_system_prompt: "generation two".into(),
        mcp_servers: Servers::new(),
        notices: vec![],
        trial: None,
    });
    let reopened = next.open(session).await.unwrap();
    let result = reply(&turn(&reopened, "fourth").await);
    assert_eq!(
        result["tools"],
        json!({}),
        "old grants crossed launch generation"
    );
    let j = f.journal("A");
    let last = j
        .iter()
        .rev()
        .find(|r| r["method"] == "session/prompt")
        .unwrap();
    assert!(last.to_string().contains("generation two"));
    assert!(
        !last.to_string().contains("generation one"),
        "stale launcher context in history"
    );
    next.close_all().await;
}
#[tokio::test]
async fn invalid_unavailable_and_absent_context_continue() {
    if isolated("invalid_unavailable_and_absent_context_continue") {
        return;
    }
    let f = Fixture::new();
    let mut invalid = f.server("profile");
    invalid.env.insert("alias".into(), "$BBOXY_SEAT_ID".into());
    f.profile("absent", "", [("invalid".into(), invalid)].into());
    let mut missing = f.server("launcher");
    missing.command = "/missing/private-mcp".into();
    let host = f.host("", [("missing".into(), missing)].into());
    let g = f.create(&host, "absent", Servers::new()).await;
    let events = turn(&g, "one request").await;
    let result = reply(&events);
    assert_eq!(result["tools"]["missing"], Value::Null);
    assert!(result["tools"].get("invalid").is_none());
    let j = f.journal("absent");
    let prompt = j.iter().find(|r| r["method"] == "session/prompt").unwrap();
    assert_eq!(
        prompt["params"]["prompt"],
        json!([{"type":"text","text":"one request"}])
    );
    assert!(
        std::fs::read_to_string(f.config.sessions.join(format!("{}.jsonl", g.session())))
            .unwrap()
            .contains("reserved environment reference")
    );
    reply(&turn(&g, "continued request").await);
    host.close_all().await;
}

#[tokio::test]
async fn bounded_startup_and_nonlaunch_environment() {
    if isolated("bounded_startup_and_nonlaunch_environment") {
        return;
    }
    let f = Fixture::new();
    let mut profile_server = f.server("profile");
    profile_server
        .env
        .insert("BBOXY_SEAT_ID".into(), "spoofed-seat".into());
    f.profile("env", "", [("profile".into(), profile_server)].into());
    let host = f.host("", Servers::new());
    let g = f
        .create(&host, "env", [("acp".into(), f.server("ACP"))].into())
        .await;
    let result = reply(&turn(&g, "environment probe").await);
    for name in ["profile", "acp"] {
        assert_eq!(
            result["tools"][name]["seat"],
            Value::Null,
            "non-launch server inherited seat"
        );
        assert_eq!(
            result["tools"][name]["custody"], false,
            "non-launch server inherited capability"
        );
    }
    host.close_all().await;
    f.profile("hang", "", Servers::new());
    let path = f.profiles.join("hang.md");
    let profile = std::fs::read_to_string(&path).unwrap();
    // Set the deliberately unresponsive fixture flag in its exact command array.
    let mut p = bboxy450_harness::provider::profile::Profile::parse(&profile).unwrap();
    p.agent
        .as_mut()
        .unwrap()
        .command
        .push("--hang-startup".into());
    std::fs::write(
        &path,
        format!("---\n{}---\n", serde_yaml::to_string(&p).unwrap()),
    )
    .unwrap();
    let host = f.host("bounded", Servers::new());
    let g = f.create(&host, "hang", Servers::new()).await;
    let before = std::time::Instant::now();
    let events = turn(&g, "deadline probe").await;
    assert!(
        before.elapsed() < Duration::from_secs(10),
        "unbounded ACP startup"
    );
    assert!(events.iter().any(|e| matches!(&e.event,Event::TurnEnded{reason:StopReason::Error{message,..},..} if message.contains("startup deadline"))));
    host.close_all().await;
}

#[tokio::test]
async fn child_restart_bootstraps_without_an_added_turn() {
    if isolated("child_restart_bootstraps_without_an_added_turn") {
        return;
    }
    let f = Fixture::new();
    f.profile("restart", "model-A", Servers::new());
    let path = f.profiles.join("restart.md");
    let mut p = bboxy450_harness::provider::profile::Profile::parse(
        &std::fs::read_to_string(&path).unwrap(),
    )
    .unwrap();
    p.agent
        .as_mut()
        .unwrap()
        .command
        .push("--crash-first".into());
    std::fs::write(
        &path,
        format!("---\n{}---\n", serde_yaml::to_string(&p).unwrap()),
    )
    .unwrap();
    let host = f.host(CONTEXT, [("seat".into(), f.server("launcher"))].into());
    let g = f.create(&host, "restart", Servers::new()).await;
    let first = turn(&g, "first real turn").await;
    assert!(first.iter().any(|e| matches!(
        e.event,
        Event::TurnEnded {
            reason: StopReason::Error { .. },
            ..
        }
    )));
    let second = reply(&turn(&g, "second real turn").await);
    assert_eq!(second["tools"]["seat"]["custody"], true);
    let j = f.journal("restart");
    assert_context(&j, CONTEXT, 2);
    for prompt in j.iter().filter(|r| r["method"] == "session/prompt") {
        assert!(
            prompt["params"]["prompt"][0]["text"]
                .as_str()
                .unwrap()
                .contains(CONTEXT),
            "restart lost launcher context"
        );
    }
    assert_eq!(
        std::fs::read_to_string(f.temp.path().join("effects"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    host.close_all().await;
}

#[tokio::test]
async fn unavailable_slow_server_does_not_hang_delivery() {
    if isolated("unavailable_slow_server_does_not_hang_delivery") {
        return;
    }
    let f = Fixture::new();
    f.profile("slow", "", Servers::new());
    let mut slow = f.server("launcher");
    slow.env
        .insert("PARITY_SLOW_CALL".into(), "fixture-wait-for-timeout".into());
    let host = f.host(
        CONTEXT,
        [("fast".into(), f.server("launcher")), ("slow".into(), slow)].into(),
    );
    let g = f.create(&host, "slow", Servers::new()).await;
    let before = std::time::Instant::now();
    let events = turn(&g, "bounded tool calls").await;
    let result = reply(&events);
    assert_eq!(result["tools"]["fast"]["custody"], true);
    assert_eq!(result["tools"]["slow"], Value::Null);
    assert!(
        before.elapsed() < Duration::from_secs(5),
        "slow MCP server hung delivery"
    );
    let bytes =
        std::fs::read_to_string(f.config.sessions.join(format!("{}.jsonl", g.session()))).unwrap();
    assert!(bytes.contains("relay unavailable or request deadline exceeded"));
    host.close_all().await;
}

#[tokio::test]
async fn outer_cli_maps_yolo_and_invalid_config_to_downstream() {
    if isolated("outer_cli_maps_yolo_and_invalid_config_to_downstream") {
        return;
    }
    let f = Fixture::new();
    f.profile("cli", "model-A", Servers::new());
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let profiles = home.join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::copy(f.profiles.join("cli.md"), profiles.join("cli.md")).unwrap();
    let mcp = f.temp.path().join("mcp.json");
    std::fs::write(
        &mcp,
        serde_json::to_vec(&json!({"mcpServers":{"seat":f.server("launcher")}})).unwrap(),
    )
    .unwrap();
    for (index, config) in [mcp.clone(), f.temp.path().join("unavailable.json")]
        .into_iter()
        .enumerate()
    {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .args([
                "--profile",
                "cli",
                "--json",
                "--yolo",
                "-p",
                "one real CLI request",
                "--append-system-prompt",
                CONTEXT,
                "--cwd",
            ])
            .arg(&f.config.meta.cwd)
            .arg("--sessions")
            .arg(f.temp.path().join(format!("cli-sessions-{index}")))
            .arg("--socket")
            .arg(home.join(format!("cli-{index}.sock")))
            .arg("--mcp-config")
            .arg(config)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "outer CLI failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let wire = String::from_utf8(output.stdout).unwrap();
        if index == 1 {
            assert!(wire.contains("launcher MCP configuration unavailable or invalid"));
        }
        let j = f.journal("cli");
        let permission = j.iter().rev().find(|r| r["id"] == "permission").unwrap();
        assert_eq!(
            permission["result"]["outcome"]["optionId"], "yes",
            "launcher --yolo approval semantics dropped"
        );
        assert_context(&j, CONTEXT, index + 1);
    }
}
