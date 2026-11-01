// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::protocol::{Envelope, laws::check_transcript};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
fn command(temp: &Path, face: &str) -> tokio::process::Command {
    command_inner(temp, face, true)
}
fn command_inner(temp: &Path, face: &str, scripted: bool) -> tokio::process::Command {
    use std::os::unix::fs::PermissionsExt;
    let capability_file = temp.join("capability");
    std::fs::write(&capability_file, format!("{}\n", "ab".repeat(32))).unwrap();
    std::fs::set_permissions(&capability_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let mut c = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    c.env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", temp)
        .env("BBOXY_CAPABILITY_FILE", capability_file)
        .env("BBOXY_SEAT_ID", "throwaway-seat")
        .env("BBOXY_SOCK", temp.join("missing.sock"))
        .env("BBOXY_GENERATION", "7")
        .env("BBOXY_ROLE", "executor")
        .env("BBOXY_TEST_CUSTOM", "throwaway-marker")
        .arg(face)
        .arg("--cwd")
        .arg(temp)
        .arg("--sessions")
        .arg(temp.join("sessions"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    if scripted {
        c.arg("--scripted")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse"));
    }
    c
}
fn server(report: &Path) -> Value {
    json!({"command":env!("CARGO_BIN_EXE_mcp-fixture"),"args":["echo_env"],"env":{"MCP_ENV_REPORT":report,"BBOXY_TEST_CONFIGURED":"throwaway-configured"},"timeout_s":2,"trust_hints":true,"launcher":true})
}
fn profile(temp: &Path, servers: Value) {
    let dir = temp.join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&dir).unwrap();
    let value = json!({"name":"nemotron","model":"mock","endpoint":"http://127.0.0.1:9/v1","api_key":"unused-test","mcpServers":servers});
    std::fs::write(
        dir.join("nemotron.md"),
        format!("---\n{}\n---\n", serde_yaml::to_string(&value).unwrap()),
    )
    .unwrap();
}
async fn observation(path: &Path) -> Value {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if let Ok(bytes) = tokio::fs::read(path).await {
                break serde_json::from_slice(&bytes).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("MCP environment fixture did not start")
}
fn check_log(temp: &Path) {
    let file = std::fs::read_dir(temp.join("sessions"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|s| s == "jsonl"))
        .unwrap();
    let log = std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .collect::<Vec<_>>();
    check_transcript(&log).unwrap();
}
async fn finish_seat(mut child: tokio::process::Child, temp: &Path) -> String {
    let mut input = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    input.write_all(b"hello\n").await.unwrap();
    input.flush().await.unwrap();
    let mut output = String::new();
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(line) = lines.next_line().await.unwrap() {
            output.push_str(&line);
            output.push('\n');
            if line.contains("EndTurn") {
                return;
            }
        }
        panic!("seat exited before its turn ended");
    })
    .await
    .expect("seat turn hang guard");
    input.shutdown().await.unwrap();
    drop(input);
    assert!(
        tokio::time::timeout(Duration::from_secs(15), child.wait())
            .await
            .expect("seat close hang guard")
            .unwrap()
            .success()
    );
    check_log(temp);
    output
}
#[tokio::test]
async fn launcher_flags_deliver_context_tools_and_only_launcher_environment() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let root = temp.path();
    let launcher_report = root.join("launcher-env.json");
    let profile_report = root.join("profile-env.json");
    let loser = root.join("overridden-env.json");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let (sent, received) = tokio::sync::oneshot::channel();
    let http = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = vec![];
        let mut buf = [0u8; 4096];
        let body = loop {
            let n = socket.read(&mut buf).await.unwrap();
            assert!(n > 0, "provider request ended early");
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
                let length: usize = header
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                if bytes.len() >= end + 4 + length {
                    break serde_json::from_slice::<Value>(&bytes[end + 4..end + 4 + length])
                        .unwrap();
                }
            }
        };
        let reply = include_bytes!("../fixtures/sse/text.sse");
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            reply.len()
        );
        socket.write_all(head.as_bytes()).await.unwrap();
        socket.write_all(reply).await.unwrap();
        let _ = sent.send(body);
    });
    profile(
        root,
        json!({"profile":server(&profile_report),"bboxy":server(&loser)}),
    );
    let p = root.join(".bboxy450-harness/profiles/nemotron.md");
    let text = std::fs::read_to_string(&p)
        .unwrap()
        .replace("http://127.0.0.1:9/v1", &endpoint);
    std::fs::write(p, text).unwrap();
    let mut launcher = server(&launcher_report);
    launcher["launcher"] = json!(false);
    let file = root.join("mcp.json");
    std::fs::write(&file, json!({"mcpServers":{"bboxy":launcher}}).to_string()).unwrap();
    let context = "NAME: throwaway executor\nROLE: EXECUTOR\nCall bboxy's orient MCP first.";
    let mut child = command_inner(root, "seat", false)
        .arg("--append-system-prompt")
        .arg(context)
        .arg("--mcp-config")
        .arg(&file)
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut output = String::new();
    tokio::time::timeout(Duration::from_secs(15), async {
        let mut launcher = false;
        let mut profile = false;
        while let Some(line) = lines.next_line().await.unwrap() {
            launcher |= line.contains("mcp bboxy: ready");
            profile |= line.contains("mcp profile: ready");
            output.push_str(&line);
            output.push('\n');
            if launcher && profile {
                return;
            }
        }
        panic!("seat exited before MCP readiness");
    })
    .await
    .expect("launcher MCP server did not become ready");
    input.write_all(b"hello\n").await.unwrap();
    input.flush().await.unwrap();
    let body = tokio::time::timeout(Duration::from_secs(15), received)
        .await
        .expect("first provider request missing")
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(line) = lines.next_line().await.unwrap() {
            output.push_str(&line);
            output.push('\n');
            if line.contains("EndTurn") {
                return;
            }
        }
        panic!("seat exited before turn completion");
    })
    .await
    .expect("seat turn hang guard");
    input.shutdown().await.unwrap();
    drop(input);
    assert!(
        tokio::time::timeout(Duration::from_secs(15), child.wait())
            .await
            .expect("seat close hang guard")
            .unwrap()
            .success()
    );
    http.await.unwrap();
    check_log(root);
    assert_eq!(body["messages"][0]["role"], "system");
    assert!(
        body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .ends_with(context),
        "appended startup text absent from stable system prefix"
    );
    assert!(
        body["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["function"]["name"] == "mcp__bboxy__read"),
        "launcher tools absent from first request"
    );
    let launcher = observation(&launcher_report).await;
    let profile = observation(&profile_report).await;
    assert_eq!(
        launcher["capability_matches"], true,
        "launcher MCP lost its capability file"
    );
    assert_eq!(launcher["legacy_capability_present"], false);
    assert_eq!(
        launcher["seat_matches"], true,
        "launcher MCP lost seat identity"
    );
    assert!(
        launcher["names"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n == "BBOXY_TEST_CONFIGURED")
    );
    assert_eq!(
        profile["names"],
        json!([]),
        "profile MCP inherited BBOXY variables"
    );
    assert!(!loser.exists(), "profile overrode launcher provenance");
    assert_eq!(
        output.matches("configuration overrides").count(),
        1,
        "source collision must emit one Notice"
    );
}
#[tokio::test]
async fn unavailable_config_is_a_notice_and_the_turn_completes() {
    for directory in [false, true] {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let file = temp.path().join("unavailable.json");
        if directory {
            std::fs::create_dir(&file).unwrap();
        }
        let child = command(temp.path(), "seat")
            .arg("--mcp-config")
            .arg(file)
            .spawn()
            .unwrap();
        let output = finish_seat(child, temp.path()).await;
        assert!(
            output.contains("launcher MCP configuration unavailable or invalid"),
            "unavailable MCP config did not produce a Notice"
        );
    }
}
#[tokio::test]
async fn unknown_options_and_missing_flag_arguments_still_fail() {
    for flag in [
        "--not-a-real-option",
        "--append-system-prompt",
        "--mcp-config",
    ] {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(15),
            command(temp.path(), "seat").arg(flag).output(),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result.status.code(), Some(1));
        let error = String::from_utf8(result.stderr).unwrap();
        assert!(error.contains("unknown option") || error.contains("missing "));
    }
}
#[tokio::test]
async fn profile_server_cannot_inherit_bboxy_environment() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let report = temp.path().join("profile-env.json");
    profile(temp.path(), json!({"bboxy":server(&report)}));
    let child = command(temp.path(), "seat").spawn().unwrap();
    let observed = observation(&report).await;
    finish_seat(child, temp.path()).await;
    assert_eq!(
        observed["names"],
        json!([]),
        "profile MCP inherited BBOXY variables"
    );
    assert_eq!(observed["capability_matches"], false);
}
#[cfg(feature = "acp")]
#[tokio::test]
async fn acp_server_cannot_inherit_bboxy_environment() {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let report = temp.path().join("acp-env.json");
    let mut child = command(temp.path(), "acp").spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let request = json!({"jsonrpc":"2.0","id":1,"method":"session/new","params":{"cwd":temp.path(),"mcpServers":[{"name":"bboxy","command":env!("CARGO_BIN_EXE_mcp-fixture"),"args":["echo_env"],"env":[{"name":"MCP_ENV_REPORT","value":report}],"launcher":true}]}});
    input
        .write_all((request.to_string() + "\n").as_bytes())
        .await
        .unwrap();
    let reply: Value = serde_json::from_str(
        &tokio::time::timeout(Duration::from_secs(15), lines.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let session = reply["result"]["sessionId"].as_str().unwrap();
    let observed = observation(&report).await;
    let request = json!({"jsonrpc":"2.0","id":2,"method":"session/prompt","params":{"sessionId":session,"prompt":[{"type":"text","text":"hello"}]}});
    input
        .write_all((request.to_string() + "\n").as_bytes())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let v: Value =
                serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
            if v["id"] == 2 {
                assert_eq!(v["result"]["stopReason"], "end_turn");
                break;
            }
        }
    })
    .await
    .expect("ACP turn hang guard");
    input.shutdown().await.unwrap();
    drop(input);
    assert!(
        tokio::time::timeout(Duration::from_secs(15), child.wait())
            .await
            .expect("ACP close hang guard")
            .unwrap()
            .success()
    );
    check_log(temp.path());
    assert_eq!(
        observed["names"],
        json!([]),
        "ACP MCP inherited BBOXY variables"
    );
    assert_eq!(observed["capability_matches"], false);
}

// Each child has a private environment and reports booleans, never identity values.
fn session_text(root: &Path) -> String {
    std::fs::read_dir(root.join("sessions"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect()
}
async fn env_attempt(acp: bool, env: Value) -> (Option<Value>, String) {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let report = temp.path().join("env.json");
    let mut config = server(&report);
    config["env"]
        .as_object_mut()
        .unwrap()
        .extend(env.as_object().unwrap().clone());
    if !acp {
        profile(temp.path(), json!({"bboxy":config}));
    }
    let mut child = command(temp.path(), if acp { "acp" } else { "seat" })
        .spawn()
        .unwrap();
    let mut input = None;
    let mut lines = None;
    let mut session = Value::Null;
    if acp {
        let mut writer = child.stdin.take().unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap()).lines();
        let env = config["env"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, value)| json!({"name":name,"value":value}))
            .collect::<Vec<_>>();
        let new = json!({"jsonrpc":"2.0","id":1,"method":"session/new","params":{"cwd":temp.path(),"mcpServers":[{"name":"bboxy","command":config["command"],"args":config["args"],"env":env}]}});
        writer
            .write_all((new.to_string() + "\n").as_bytes())
            .await
            .unwrap();
        let reply: Value = serde_json::from_str(
            &tokio::time::timeout(Duration::from_secs(15), reader.next_line())
                .await
                .unwrap()
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        session = reply["result"]["sessionId"].clone();
        assert!(session.is_string());
        input = Some(writer);
        lines = Some(reader);
    }
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let log = session_text(temp.path());
            if log.contains("mcp bboxy: ready")
                || log.contains("mcp bboxy: failed")
                || log.contains("mcp bboxy: config")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("MCP environment attempt never resolved");
    if acp {
        let mut input = input.unwrap();
        let mut lines = lines.unwrap();
        let prompt = json!({"jsonrpc":"2.0","id":2,"method":"session/prompt","params":{"sessionId":session,"prompt":[{"type":"text","text":"hello"}]}});
        input
            .write_all((prompt.to_string() + "\n").as_bytes())
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let reply: Value =
                    serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
                if reply["id"] == 2 {
                    assert_eq!(reply["result"]["stopReason"], "end_turn");
                    break;
                }
            }
        })
        .await
        .expect("ACP environment turn hang guard");
        drop(input);
        assert!(
            tokio::time::timeout(Duration::from_secs(15), child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
        check_log(temp.path());
    } else {
        finish_seat(child, temp.path()).await;
    }
    let observed = std::fs::read(report)
        .ok()
        .map(|b| serde_json::from_slice(&b).unwrap());
    (observed, session_text(temp.path()))
}
#[cfg(feature = "acp")]
#[tokio::test]
async fn profile_and_acp_refuse_bboxy_environment_indirection() {
    for acp in [false, true] {
        for braced in [false, true] {
            let env = json!({"S":if braced {"${BBOXY_SOCK}"} else {"$BBOXY_SOCK"},"I":"$BBOXY_SEAT_ID","R":"!printenv BBOXY_ROLE"});
            let (observed, log) = env_attempt(acp, env).await;
            assert!(
                log.contains("configuration error: reserved environment reference"),
                "BBOXY reference lacked typed configuration refusal (ACP={acp}, braced={braced})"
            );
            assert!(
                observed.is_none(),
                "untrusted MCP started with a BBOXY alias (ACP={acp})"
            );
            assert!(log.contains("turn_ended"));
        }
    }
}
#[cfg(feature = "acp")]
#[tokio::test]
async fn command_resolver_cannot_copy_seat_environment_into_mcp_aliases() {
    for acp in [false, true] {
        // No dollar reference: this must be protected by the resolver's own scrub.
        let (observed, log) = env_attempt(acp, json!({"S":"!printenv BBOXY_SOCK"})).await;
        assert!(
            observed.is_none(),
            "command resolver leaked BBOXY_SOCK into an MCP alias (ACP={acp})"
        );
        assert!(log.contains("environment command failed"));
        let (observed, _) = env_attempt(acp, json!({"S":"!printenv BBOXY_SOCK || true"})).await;
        let observed = observed.expect("an empty successful resolver value is allowed");
        assert_eq!(
            observed["alias_empty"], true,
            "command resolver did not remove the BBOXY namespace (ACP={acp})"
        );
        assert_eq!(observed["names"], json!([]));
        assert_eq!(observed["alias_sock_matches"], false);
    }
}
