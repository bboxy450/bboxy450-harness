// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    protocol::*,
    provider::{claude_code::*, profile::Profile},
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn json_lines_are_bounded_and_reject_untrusted_shapes() {
    for (bytes, expected) in [
        (b"{broken\n".to_vec(), "malformed"),
        (vec![b'x'; MAX_LINE + 1], "exceeds"),
        (b"{}".to_vec(), "truncated"),
        (b"[]\n".to_vec(), "object"),
    ] {
        let mut input = tokio::io::BufReader::with_capacity(17, bytes.as_slice());
        let error = JsonLines::default().next(&mut input).await.unwrap_err();
        assert_eq!(error.class, ErrorClass::Malformed);
        assert!(error.message.contains(expected), "{}", error.message);
    }
    let mut input = tokio::io::BufReader::with_capacity(1, b"{\"type\":\"future\"}\n".as_slice());
    let mut lines = JsonLines::default();
    assert_eq!(
        lines.next(&mut input).await.unwrap().unwrap()["type"],
        "future"
    );
    assert!(lines.next(&mut input).await.unwrap().is_none());
}

#[tokio::test]
async fn claude_profile_has_no_endpoint_or_credential_resolver() {
    let p =
        Profile::parse("---\nname: cli\nprovider: claude-code\npermission_mode: dontAsk\n---\n")
            .unwrap();
    let provider = p
        .build_provider(&Default::default(), CancellationToken::new())
        .await
        .unwrap();
    let opts = provider.claude_code().unwrap();
    assert!(opts.model.is_empty());
    assert_eq!(opts.permission_mode.as_deref(), Some("dontAsk"));
    assert_eq!(
        p.key(&Default::default(), CancellationToken::new())
            .await
            .unwrap_err()
            .class,
        ErrorClass::Auth
    );
    assert!(
        Profile::parse("---\nname: api\nendpoint: https://example.invalid/v1\n---\n").is_err(),
        "API profiles still require their model before resolving credentials"
    );
    for field in [
        "api_key: '!touch must-not-execute'",
        "api_key_env: CLAUDE_CODE_OAUTH_TOKEN",
        "permission_mode: injected",
    ] {
        assert!(
            Profile::parse(&format!(
                "---\nname: cli\nprovider: claude-code\n{field}\n---\n"
            ))
            .is_err()
        );
    }
}

#[test]
fn argv_is_an_allowlist_and_does_not_replace_claude_tools_or_prompt() {
    let options = Options {
        model: "sonnet".into(),
        permission_mode: Some("dontAsk".into()),
    };
    assert_eq!(
        options.argv(Some("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")),
        vec![
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--model",
            "sonnet",
            "--permission-mode",
            "dontAsk",
            "--resume",
            "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
        ]
    );
    for arg in options.argv(None) {
        assert!(
            ![
                "--system-prompt",
                "--system-prompt-file",
                "--tools",
                "--disallowedTools",
                "--allowedTools",
                "--bare"
            ]
            .contains(&arg.as_str())
        );
    }
    let set = Default::default();
    assert!(!scrub(&set, "access_token=fixture-secret-value").contains("fixture-secret-value"));
    assert!(!scrub(&set, "Bearer fixture-bearer-value").contains("fixture-bearer-value"));
}

use serde_json::Value;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
struct Seat {
    child: Child,
    input: Option<ChildStdin>,
    lines: tokio::io::Lines<BufReader<ChildStdout>>,
    temp: tempfile::TempDir,
    events: Vec<Envelope>,
}
impl Seat {
    async fn start(env: &[(&str, &str)]) -> Self {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let profiles = temp.path().join(".bboxy450-harness/profiles");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::write(profiles.join("code.md"), "---\nname: code\nprovider: claude-code\nmodel: sonnet\npermission_mode: dontAsk\npolicy: yolo\n---\n").unwrap();
        // A credential fixture is never readable by the instrumented harness.
        std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
        std::fs::write(
            temp.path().join(".claude/.credentials.json"),
            "must never be opened",
        )
        .unwrap();
        Self::spawn(temp, env, None).await
    }
    async fn spawn(
        temp: tempfile::TempDir,
        env: &[(&str, &str)],
        resume: Option<SessionId>,
    ) -> Self {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude-code");
        let mut paths = vec![fixture];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let mut command = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        command
            .args(["seat", "--json", "--profile", "code", "--cwd"])
            .arg(temp.path())
            .env("HOME", temp.path())
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("FAKE_CLAUDE_TRACE", temp.path().join("trace.jsonl"))
            .env("CLAUDE_CODE_OAUTH_TOKEN", "fixture-inherited-value")
            .env_remove("BBOXY_CAPABILITY")
            .env_remove("BBOXY_CAPABILITY_FILE")
            .envs(env.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(resume) = resume {
            command.args(["--resume", &resume.to_string()]);
        }
        let mut child = command.spawn().unwrap();
        let input = child.stdin.take();
        let lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut seat = Self {
            child,
            input,
            lines,
            temp,
            events: vec![],
        };
        loop {
            if matches!(seat.event().await.event, Event::SessionStarted { .. }) {
                break;
            }
        }
        if resume.is_some() {
            let log = temp_log(&seat);
            let head = std::fs::read_to_string(log)
                .unwrap()
                .lines()
                .last()
                .map(|line| serde_json::from_str::<Envelope>(line).unwrap().seq.unwrap())
                .unwrap();
            while seat.events.last().and_then(|e| e.seq).unwrap_or(Seq(0)) < head {
                seat.event().await;
            }
        }
        seat
    }
    async fn event(&mut self) -> Envelope {
        let line = tokio::time::timeout(Duration::from_secs(8), self.lines.next_line())
            .await
            .expect("harness event deadline")
            .unwrap();
        let line = if let Some(line) = line {
            line
        } else {
            use tokio::io::AsyncReadExt;
            let mut error = String::new();
            if let Some(mut pipe) = self.child.stderr.take() {
                let _ =
                    tokio::time::timeout(Duration::from_secs(1), pipe.read_to_string(&mut error))
                        .await;
            }
            panic!(
                "harness stdout closed: {:?}: {error}; trace={:?}",
                self.child.try_wait(),
                self.trace()
            );
        };
        let event: Envelope = serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"));
        self.events.push(event.clone());
        event
    }
    async fn request(&self, op: Op) {
        use bboxy450_harness::gateway::{Subscribe, unix::UnixClient};
        let session = self.events[0].session;
        let path = self
            .temp
            .path()
            .join(".bboxy450-harness/run")
            .join(format!("{session}.sock"));
        let (client, _sub) = UnixClient::connect(
            &path,
            Subscribe {
                session,
                after: None,
                ephemeral: false,
            },
        )
        .await
        .unwrap();
        client
            .send(OpEnvelope {
                v: PROTOCOL,
                session,
                op_id: OpId::new(),
                op,
            })
            .await
            .unwrap();
        client.close();
    }
    async fn prompt(&mut self, text: &str) {
        self.input
            .as_mut()
            .unwrap()
            .write_all(format!("{text}\n").as_bytes())
            .await
            .unwrap();
    }
    async fn end(&mut self) -> StopReason {
        loop {
            if let Event::TurnEnded { reason, .. } = self.event().await.event {
                return reason;
            }
        }
    }
    async fn turn(&mut self, text: &str) -> StopReason {
        self.prompt(text).await;
        self.end().await
    }
    fn trace(&self) -> Vec<Value> {
        std::fs::read_to_string(self.temp.path().join("trace.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    async fn close(mut self) -> Vec<Envelope> {
        self.input.take();
        loop {
            if matches!(self.event().await.event, Event::SessionClosed) {
                break;
            }
        }
        let status = tokio::time::timeout(Duration::from_secs(2), self.child.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(status.success(), "seat exited {status}");
        laws::check_transcript(&self.events).unwrap();
        self.events
    }
}
fn temp_log(s: &Seat) -> PathBuf {
    s.temp
        .path()
        .join(".bboxy450-harness/sessions")
        .join(format!("{}.jsonl", s.events[0].session))
}
fn has_text(events: &[Envelope], expected: &str) -> bool {
    events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::AgentMessage { markdown }, .. } if markdown.contains(expected)))
}

#[tokio::test]
async fn official_cli_session_projects_text_tools_thinking_and_two_turns() {
    let mut s = Seat::start(&[]).await;
    assert_eq!(s.turn("hello").await, StopReason::EndTurn);
    assert_eq!(s.turn("tools").await, StopReason::EndTurn);
    assert_eq!(s.turn("thinking").await, StopReason::EndTurn);
    assert_eq!(s.turn("unknown").await, StopReason::EndTurn);
    assert_eq!(s.turn("duplicate").await, StopReason::EndTurn);
    assert_eq!(s.turn("last").await, StopReason::EndTurn);
    assert!(has_text(&s.events, "reply 1: hello"));
    assert!(has_text(&s.events, "reply 6: last"));
    assert!(s.events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::Reasoning { summary, .. }, .. } if summary == "a short thought")));
    assert!(s.events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::ToolCall { name, output, .. }, .. } if name.starts_with("claude-code:") && !output.is_empty())));
    assert!(s.events.iter().any(|e| matches!(&e.event, Event::ItemCompleted { body: Item::Notice { text, .. }, .. } if text.contains("unknown Claude Code event"))));
    assert!(s.events.iter().any(|e| matches!(&e.event, Event::TurnEnded { usage: Some(u), .. } if u.input_tokens == 12 && u.output_tokens == 7 && u.cached_input_tokens == Some(3) && u.cost_micros_usd == Some(123))));
    assert!(!s.temp.path().join("must-not-be-written").exists());
    let trace = s.trace();
    assert_eq!(
        trace
            .iter()
            .filter(|v| v["argv"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v == "-p")))
            .count(),
        1
    );
    assert!(
        trace
            .iter()
            .filter(|v| v.get("input").is_some())
            .all(|v| v["oauth_inherited"] == true)
    );
    let events = s.close().await;
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e.event, Event::TurnEnded { .. }))
            .count(),
        6
    );
}

#[tokio::test]
async fn errors_auth_hostile_lines_and_crashes_end_and_pair_without_hanging() {
    let mut s = Seat::start(&[]).await;
    for (prompt, class) in [
        ("error", ErrorClass::BadRequest),
        ("malformed", ErrorClass::Malformed),
        ("oversized", ErrorClass::Malformed),
        ("crash", ErrorClass::MidStream),
        ("not-logged-in", ErrorClass::Auth),
    ] {
        let reason = s.turn(prompt).await;
        assert!(
            matches!(reason, StopReason::Error { class: c, .. } if c == class),
            "{prompt}: {reason:?}"
        );
    }
    let text = serde_json::to_string(&s.events).unwrap();
    assert!(text.contains("17"));
    assert!(text.contains("claude /login"));
    assert!(!text.contains("fixture-secret-value"));
    assert!(!text.contains("fixture-inherited-value"));
    s.close().await;
    let mut missing = Seat::start(&[("FAKE_CLAUDE_AUTH", "missing")]).await;
    assert!(matches!(
        missing.turn("hello").await,
        StopReason::Error {
            class: ErrorClass::Auth,
            ..
        }
    ));
    missing.close().await;
}

#[tokio::test]
async fn missing_cli_and_version_drift_fail_with_a_terminal_notice() {
    for (env, class) in [
        ([("PATH", "/nonexistent")], ErrorClass::Auth),
        ([("FAKE_CLAUDE_VERSION", "0.0.0")], ErrorClass::Config),
    ] {
        let mut s = Seat::start(&env).await;
        assert!(matches!(s.turn("hello").await, StopReason::Error { class: c, .. } if c == class));
        s.close().await;
    }
}

#[tokio::test]
async fn cancellation_joins_the_group_in_two_seconds_and_a_to_b_to_a_resumes() {
    let mut s = Seat::start(&[]).await;
    for prompt in ["cancel", "late"] {
        s.prompt(prompt).await;
        loop {
            if matches!(
                s.event().await.event,
                Event::ItemStarted {
                    kind: ItemKind::ToolCall,
                    ..
                }
            ) {
                break;
            }
        }
        let before = tokio::time::Instant::now();
        s.request(Op::Interrupt { turn: None }).await;
        assert_eq!(s.end().await, StopReason::Cancelled);
        assert!(before.elapsed() < Duration::from_secs(2));
        if prompt == "late" {
            assert!(
                s.trace().iter().any(|v| v["late_emitted"] == true),
                "the late-event producer did not run during cancel grace"
            );
        }
        for member in s.trace().iter().filter_map(|v| v["leader"].as_i64()) {
            assert_eq!(
                unsafe { libc::kill(-(member as i32), 0) },
                -1,
                "owned group survived cancel"
            );
        }
    }
    assert_eq!(s.turn("after restart").await, StopReason::EndTurn);
    let trace = s.trace();
    let commands = trace
        .iter()
        .filter_map(|v| v["argv"].as_array())
        .filter(|args| args.iter().any(|v| v == "-p"))
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 3);
    for args in &commands[1..] {
        let i = args
            .iter()
            .position(|v| v == "--resume")
            .expect("restart lost the resume id");
        assert_eq!(args[i + 1], "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    }
    assert!(!has_text(&s.events, "stale output"));
    s.close().await;
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn harness_never_opens_claude_credentials_or_calls_keychain() {
    let t = tempfile::tempdir_in("/tmp").unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude-code/deny_credentials.c");
    let library = t.path().join("deny_credentials.dylib");
    let probe = t.path().join("credential_probe");
    for (artifact, flag) in [(&library, "-dynamiclib"), (&probe, "-DPROBE")] {
        let result = Command::new("cc")
            .arg(flag)
            .arg(&source)
            .args(["-framework", "Security", "-o"])
            .arg(artifact)
            .output()
            .await
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    // A functioning monitor must reject both deliberately broken controls.
    for (arg, expected) in [("file", 91), ("keychain", 92)] {
        let result = Command::new(&probe)
            .arg(arg)
            .env("DYLD_INSERT_LIBRARIES", &library)
            .status()
            .await
            .unwrap();
        assert_eq!(
            result.code(),
            Some(expected),
            "credential monitor was inert"
        );
    }
    let library = library.to_str().unwrap();
    let mut s = Seat::start(&[("DYLD_INSERT_LIBRARIES", library)]).await;
    assert_eq!(s.turn("tools").await, StopReason::EndTurn);
    let trace = s.trace();
    let commands = trace
        .iter()
        .filter_map(|v| v["argv"].as_array())
        .filter(|a| a.iter().any(|v| v == "-p"))
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0],
        &serde_json::json!([
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--model",
            "sonnet",
            "--permission-mode",
            "dontAsk"
        ])
        .as_array()
        .unwrap()
        .clone()
    );
    s.close().await;
}

#[tokio::test]
async fn a_to_b_to_a_harness_process_restarts_recover_the_durable_cli_id() {
    let mut s = Seat::start(&[]).await;
    assert_eq!(s.turn("first process").await, StopReason::EndTurn);
    let session = s.events[0].session;
    for prompt in ["second process", "third process"] {
        // An explicit test crash leaves the log unclosed. Clean up the one
        // still-live descendant owned by this test before restarting the face.
        let pgid = s
            .trace()
            .iter()
            .rev()
            .find(|v| {
                v["argv"]
                    .as_array()
                    .is_some_and(|args| args.iter().any(|v| v == "-p"))
            })
            .unwrap()["pid"]
            .as_i64()
            .unwrap() as i32;
        s.child.start_kill().unwrap();
        s.child.wait().await.unwrap();
        assert_eq!(unsafe { libc::kill(-pgid, libc::SIGKILL) }, 0);
        let temp = s.temp;
        s = Seat::spawn(temp, &[], Some(session)).await;
        assert_eq!(s.turn(prompt).await, StopReason::EndTurn);
    }
    let trace = s.trace();
    let commands = trace
        .iter()
        .filter_map(|v| v["argv"].as_array())
        .filter(|args| args.iter().any(|v| v == "-p"))
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 3);
    for args in &commands[1..] {
        let i = args.iter().position(|v| v == "--resume").unwrap();
        assert_eq!(args[i + 1], "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    }
    s.close().await;
}

#[tokio::test]
async fn close_during_admission_and_version_startup_still_ends_cancelled() {
    for env in [vec![], vec![("FAKE_CLAUDE_VERSION_HANG", "1")]] {
        let mut s = Seat::start(&env).await;
        s.prompt("hello").await;
        loop {
            if matches!(s.event().await.event, Event::TurnStarted { .. }) {
                break;
            }
        }
        let before = tokio::time::Instant::now();
        s.request(Op::Close).await;
        assert_eq!(s.end().await, StopReason::Cancelled);
        assert!(before.elapsed() < Duration::from_secs(2));
        // Close already closed admission, so drain without submitting another op.
        loop {
            if matches!(s.event().await.event, Event::SessionClosed) {
                break;
            }
        }
        assert!(s.child.wait().await.unwrap().success());
        laws::check_transcript(&s.events).unwrap();
    }
}

#[tokio::test]
async fn profile_a_to_b_to_a_resumes_only_the_matching_cli_session() {
    let mut s = Seat::start(&[]).await;
    assert_eq!(s.turn("profile A").await, StopReason::EndTurn);
    std::fs::write(
        s.temp.path().join(".bboxy450-harness/profiles/other.md"),
        "---\nname: other\nprovider: claude-code\nmodel: opus\n---\n",
    )
    .unwrap();
    for (profile, prompt) in [("other", "profile B"), ("code", "profile A again")] {
        s.request(Op::SetProfile {
            profile: profile.into(),
        })
        .await;
        loop {
            if matches!(&s.event().await.event, Event::ProfileChanged { profile: p, .. } if p == profile)
            {
                break;
            }
        }
        assert_eq!(s.turn(prompt).await, StopReason::EndTurn);
    }
    let trace = s.trace();
    let commands = trace
        .iter()
        .filter_map(|v| v["argv"].as_array())
        .filter(|args| args.iter().any(|v| v == "-p"))
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 3);
    assert!(!commands[1].iter().any(|v| v == "--resume"));
    let i = commands[2].iter().position(|v| v == "--resume").unwrap();
    assert_eq!(commands[2][i + 1], "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    s.close().await;
}
