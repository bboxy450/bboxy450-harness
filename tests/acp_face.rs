// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "acp")]
mod support;
use bboxy450_harness::{
    engine::Config, frontends::acp, policy::Policy, protocol::*, provider::ScriptedProvider,
    provider::profile::Unattended, redact::Set,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
struct Client {
    read: tokio::io::Lines<tokio::io::BufReader<tokio::io::ReadHalf<tokio::io::DuplexStream>>>,
    write: tokio::io::WriteHalf<tokio::io::DuplexStream>,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
    config: Config,
}
async fn client(temp: &tempfile::TempDir, rounds: Vec<Vec<u8>>) -> Client {
    client_policy(temp, rounds, Unattended::Deny).await
}
async fn client_policy(
    temp: &tempfile::TempDir,
    rounds: Vec<Vec<u8>>,
    unattended: Unattended,
) -> Client {
    let config = Config {
        meta: support::meta(temp.path().into()),
        sessions: temp.path().join("sessions"),
        profiles: temp.path().join("profiles"),
        provider: Arc::new(ScriptedProvider::new(rounds)),
        model: "scripted".into(),
        policy: Policy::new(unattended),
        redactor: Set::default(),
        max_rounds: 8,
        profile_error: None,
    };
    let (client, server) = tokio::io::duplex(1024 * 1024);
    let (read, write) = tokio::io::split(client);
    let (server_read, server_write) = tokio::io::split(server);
    let cfg = config.clone();
    let task = tokio::spawn(async move { acp::serve(server_read, server_write, cfg).await });
    Client {
        read: tokio::io::BufReader::new(read).lines(),
        write,
        task,
        config,
    }
}
impl Client {
    async fn send(&mut self, value: Value) {
        let mut bytes = serde_json::to_vec(&value).unwrap();
        bytes.push(b'\n');
        self.write.write_all(&bytes).await.unwrap();
    }
    async fn request(&mut self, id: i32, method: &str, params: Value) {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await;
    }
    async fn next(&mut self) -> Value {
        let line = tokio::time::timeout(std::time::Duration::from_secs(8), self.read.next_line())
            .await
            .expect("ACP dispatcher stalled")
            .unwrap()
            .unwrap();
        assert!(line.len() <= 256 * 1024);
        serde_json::from_str(&line).unwrap()
    }
    async fn close(mut self) {
        self.write.shutdown().await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), self.task)
            .await
            .expect("ACP EOF did not join")
            .unwrap()
            .unwrap();
    }
}
fn validate(value: &Value) {
    if value["method"] == "session/update" {
        serde_json::from_value::<agent_client_protocol::schema::v1::SessionNotification>(
            value["params"].clone(),
        )
        .unwrap();
    } else if value["method"] == "session/request_permission" {
        serde_json::from_value::<agent_client_protocol::schema::v1::RequestPermissionRequest>(
            value["params"].clone(),
        )
        .unwrap();
    }
}
#[tokio::test]
async fn scripted_client_full_turn_and_permission_keep_the_dispatcher_live() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("input.txt"), "before\n").unwrap();
    std::fs::write(temp.path().join("other.txt"), "other\n").unwrap();
    let mut c = client(
        &temp,
        vec![
            support::sse("tools"),
            support::sse("edit"),
            support::tool_sse("terminal", "bash", json!({"command":"printf terminal"})),
            support::sse("text"),
        ],
    )
    .await;
    c.request(1, "initialize", json!({"protocolVersion":99}))
        .await;
    let init = c.next().await;
    assert_eq!(init["result"]["protocolVersion"], 1);
    assert_eq!(
        init["result"]["agentCapabilities"],
        json!({"loadSession":true,"promptCapabilities":{"image":true,"audio":false,"embeddedContext":true},"mcpCapabilities":{"http":false,"sse":false},"sessionCapabilities":{"close":{}}})
    );
    assert_eq!(init["result"]["authMethods"], json!([]));
    c.request(2, "session/new", json!({"cwd":temp.path(),"mcpServers":[]}))
        .await;
    let sid = c.next().await["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();
    c.request(
        3,
        "session/prompt",
        json!({"sessionId":sid,"prompt":[{"type":"text","text":"read edit execute"}]}),
    )
    .await;
    let mut messages = vec![];
    let mut permission = false;
    loop {
        let value = c.next().await;
        validate(&value);
        if value["method"] == "session/request_permission" {
            permission = true;
            // An outstanding permission and prompt must not hold the JSON-RPC dispatcher.
            c.request(
                8,
                "session/set_mode",
                json!({"sessionId":sid,"modeId":"unknown"}),
            )
            .await;
            c.send(json!({"jsonrpc":"2.0","id":value["id"],"result":{"outcome":{"outcome":"selected","optionId":"allow"}}})).await;
        }
        let done = value["id"] == 3;
        messages.push(value);
        if done {
            break;
        }
    }
    golden("scripted-turn", &canonical(&messages, temp.path()));
    assert!(permission);
    assert!(
        messages
            .iter()
            .any(|m| m["id"] == 8 && m["error"]["code"] == -32601)
    );
    let types = messages
        .iter()
        .filter_map(|m| m["params"]["update"]["sessionUpdate"].as_str())
        .collect::<Vec<_>>();
    for expected in [
        "agent_message_chunk",
        "agent_thought_chunk",
        "tool_call",
        "tool_call_update",
    ] {
        assert!(types.contains(&expected), "missing {expected}");
    }
    let all = serde_json::to_string(&messages).unwrap();
    assert!(all.contains("```diff"));
    assert!(all.contains("exit 0"));
    assert_eq!(messages.last().unwrap()["result"]["stopReason"], "end_turn");
    assert_eq!(
        std::fs::read_to_string(temp.path().join("input.txt")).unwrap(),
        "after\n"
    );
    let sessions = c.config.sessions.clone();
    c.close().await;
    let log = std::fs::read_to_string(bboxy450_harness::store::log_path(
        &sessions,
        SessionId(sid.parse().unwrap()),
    ))
    .unwrap();
    let log = log
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect::<Vec<_>>();
    bboxy450_harness::protocol::laws::check_transcript(&log).unwrap();
    assert!(log.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Approval {by:Some(Resolver::Face {name}),..},..} if name=="acp")));
}
#[tokio::test]
async fn load_replays_all_items_including_plan_before_response() {
    let temp = tempfile::tempdir().unwrap();
    let mut c = client(&temp, vec![]).await;
    let mut log = include_str!("fixtures/protocol/all-items.jsonl")
        .lines()
        .map(|s| serde_json::from_str::<Envelope>(s).unwrap())
        .filter(|e| e.seq.is_some() && !matches!(e.event, Event::SessionClosed))
        .collect::<Vec<_>>();
    let meta = match &log[0].event {
        Event::SessionStarted { meta } => meta.clone(),
        _ => panic!(),
    };
    std::fs::create_dir_all(&c.config.sessions).unwrap();
    let path = bboxy450_harness::store::log_path(&c.config.sessions, meta.session);
    let bytes = log
        .drain(..)
        .map(|e| serde_json::to_string(&e).unwrap() + "\n")
        .collect::<String>();
    std::fs::write(path, bytes).unwrap();
    c.request(
        1,
        "session/load",
        json!({"sessionId":meta.session.to_string(),"cwd":meta.cwd,"mcpServers":[]}),
    )
    .await;
    let mut updates = vec![];
    loop {
        let value = c.next().await;
        validate(&value);
        if value["id"] == 1 {
            assert!(value.get("result").is_some(), "{value}");
            break;
        }
        updates.push(value);
    }
    golden("loaded-all-items", &updates);
    assert!(
        updates
            .iter()
            .any(|v| v["params"]["update"]["sessionUpdate"] == "plan")
    );
    assert!(
        updates
            .iter()
            .any(|v| v["params"]["update"]["sessionUpdate"] == "user_message_chunk")
    );
    c.close().await;
}
#[tokio::test]
async fn malformed_line_does_not_end_the_face() {
    let temp = tempfile::tempdir().unwrap();
    let mut c = client(&temp, vec![]).await;
    c.write.write_all(b"{invalid}\n").await.unwrap();
    assert_eq!(c.next().await["error"]["code"], -32700);
    c.request(2, "authenticate", json!({})).await;
    assert_eq!(c.next().await["error"]["code"], -32601);
    c.close().await;
}
fn golden(name: &str, values: &[Value]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/acp/{name}.jsonl"));
    let bytes = values
        .iter()
        .map(|v| serde_json::to_string(v).unwrap() + "\n")
        .collect::<String>();
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), bytes);
}
/// Canonical client-visible transcript: coalesce suffix chunks, omit input-only
/// refreshes and unfinished terminal replacements, sort parallel completions.
fn canonical(values: &[Value], cwd: &std::path::Path) -> Vec<Value> {
    let ids = values
        .iter()
        .filter(|v| v["params"]["update"]["sessionUpdate"] == "tool_call")
        .filter_map(|v| v["params"]["update"]["toolCallId"].as_str())
        .enumerate()
        .map(|(i, id)| (id.to_owned(), format!("tool-{i}")))
        .collect::<std::collections::BTreeMap<_, _>>();
    fn rewrite(
        v: &mut Value,
        ids: &std::collections::BTreeMap<String, String>,
        cwd: &str,
        real: &str,
    ) {
        match v {
            Value::String(s) => {
                *s = s.replace(real, "<cwd>").replace(cwd, "<cwd>");
            }
            Value::Object(o) => {
                for (k, v) in o {
                    if k == "sessionId" {
                        *v = json!("session");
                    } else if k == "toolCallId" {
                        if let Some(id) = v.as_str().and_then(|s| ids.get(s)) {
                            *v = json!(id);
                        }
                    } else {
                        rewrite(v, ids, cwd, real);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    rewrite(v, ids, cwd, real);
                }
            }
            _ => {}
        }
    }
    let mut out = vec![];
    let mut chunks = std::collections::BTreeMap::<String, String>::new();
    for value in values {
        let update = &value["params"]["update"];
        let kind = update["sessionUpdate"].as_str().unwrap_or("");
        if kind.ends_with("_chunk") {
            chunks
                .entry(kind.into())
                .or_default()
                .push_str(update["content"]["text"].as_str().unwrap_or(""));
            continue;
        }
        if kind == "tool_call_update" && update.get("status").is_none() {
            if update.get("content").is_none() {
                continue;
            }
            let text = update["content"][0]["content"]["text"]
                .as_str()
                .unwrap_or("");
            if text.starts_with("$ ") && !text.contains("exit ") {
                continue;
            }
        }
        let mut value = value.clone();
        rewrite(
            &mut value,
            &ids,
            &cwd.display().to_string(),
            &std::fs::canonicalize(cwd).unwrap().display().to_string(),
        );
        out.push(value);
    }
    for (kind, text) in chunks {
        out.push(json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"session","update":{"sessionUpdate":kind,"content":{"type":"text","text":text}}}}));
    }
    out.sort_by_key(|v| serde_json::to_string(v).unwrap());
    out
}
#[tokio::test]
async fn queued_prompts_cancel_and_other_sessions_keep_running() {
    let temp = tempfile::tempdir().unwrap();
    let mut c = client(
        &temp,
        vec![
            support::sse("bash"),
            support::sse("text"),
            support::sse("text"),
        ],
    )
    .await;
    c.request(1, "session/new", json!({"cwd":temp.path(),"mcpServers":[]}))
        .await;
    let sid = c.next().await["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_owned();
    c.request(
        2,
        "session/prompt",
        json!({"sessionId":sid,"prompt":[{"type":"text","text":"wait"}]}),
    )
    .await;
    let permission = loop {
        let value = c.next().await;
        if value["method"] == "session/request_permission" {
            break value["id"].clone();
        }
    };
    c.request(
        3,
        "session/prompt",
        json!({"sessionId":sid,"prompt":[{"type":"text","text":"queued"}]}),
    )
    .await;
    c.request(4, "session/new", json!({"cwd":temp.path(),"mcpServers":[]}))
        .await;
    let other = loop {
        let value = c.next().await;
        if value["id"] == 4 {
            break value["result"]["sessionId"].as_str().unwrap().to_owned();
        }
    };
    c.request(
        5,
        "session/prompt",
        json!({"sessionId":other,"prompt":[{"type":"text","text":"other session"}]}),
    )
    .await;
    c.send(json!({"jsonrpc":"2.0","method":"session/cancel","params":{"sessionId":sid}}))
        .await;
    let mut done = std::collections::BTreeMap::new();
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while !done.contains_key(&2) {
            let value = c.next().await;
            validate(&value);
            if let Some(id) = value["id"].as_i64().filter(|id| [2, 3, 5].contains(id)) {
                done.insert(id, value["result"]["stopReason"].clone());
            }
        }
    })
    .await
    .expect("ACP cancel did not settle its prompt");
    c.send(json!({"jsonrpc":"2.0","id":permission,"result":{"outcome":{"outcome":"selected","optionId":"allow"}}})).await;
    while done.len() < 3 {
        let value = c.next().await;
        if let Some(id) = value["id"].as_i64().filter(|id| [2, 3, 5].contains(id)) {
            done.insert(id, value["result"]["stopReason"].clone());
        }
    }
    assert_eq!(done[&2], "cancelled");
    assert_eq!(done[&3], "end_turn");
    assert_eq!(done[&5], "end_turn");
    c.close().await;
}
#[tokio::test]
async fn every_unsupported_method_has_method_not_found() {
    let temp = tempfile::tempdir().unwrap();
    let mut c = client(&temp, vec![]).await;
    c.send(json!({"jsonrpc":"2.0","method":"unknown/notification"}))
        .await;
    for (index, method) in [
        "authenticate",
        "session/set_mode",
        "session/set_config_option",
        "session/list",
        "session/resume",
        "session/delete",
        "logout",
        "session/fork",
        "providers/list",
        "nes/predict",
        "mcp/list",
        "document/edit",
        "unknown",
    ]
    .iter()
    .enumerate()
    {
        c.request(index as i32, method, json!({})).await;
        assert_eq!(c.next().await["error"]["code"], -32601);
    }
    c.close().await;
}
#[tokio::test]
async fn acp_binary_uses_only_jsonrpc_on_real_stdio() {
    let temp = tempfile::tempdir().unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse");
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    command
        .arg("acp")
        .arg("--scripted")
        .arg(path)
        .arg("--cwd")
        .arg(temp.path())
        .arg("--sessions")
        .arg(temp.path().join("sessions"))
        .env("HOME", temp.path())
        .env_remove("BBOXY_CAPABILITY")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().unwrap();
    let mut write = child.stdin.take().unwrap();
    let mut lines = tokio::io::BufReader::new(child.stdout.take().unwrap()).lines();
    async fn send(write: &mut tokio::process::ChildStdin, value: Value) {
        write
            .write_all((serde_json::to_string(&value).unwrap() + "\n").as_bytes())
            .await
            .unwrap();
    }
    send(&mut write,json!({"jsonrpc":"2.0","id":1,"method":"session/new","params":{"cwd":temp.path(),"mcpServers":[]}})).await;
    let value: Value = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
    let sid = value["result"]["sessionId"].as_str().unwrap();
    send(&mut write,json!({"jsonrpc":"2.0","id":2,"method":"session/prompt","params":{"sessionId":sid,"prompt":[{"type":"text","text":"hello"}]}})).await;
    loop {
        let line = tokio::time::timeout(std::time::Duration::from_secs(8), lines.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let value: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["jsonrpc"], "2.0");
        if value["id"] == 2 {
            assert_eq!(value["result"]["stopReason"], "end_turn");
            break;
        }
    }
    write.shutdown().await.unwrap();
    drop(write);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(2), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
}
#[tokio::test]
#[cfg(feature = "test-fixtures")]
async fn eof_cancels_a_pending_profile_factory_without_waiting_for_its_key_command() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    let marker = temp.path().join("key-command-started");
    std::fs::write(profiles.join("slow.md"),format!("---\nname: slow\nmodel: mock\nendpoint: https://example.invalid/v1\napi_key: '!{} key_pipe_escape {}'\n---\n",env!("CARGO_BIN_EXE_mcp-fixture"),marker.display())).unwrap();
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .args(["acp", "--profile", "slow"])
        .env("HOME", temp.path())
        .env_remove("BBOXY_CAPABILITY")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut write = child.stdin.take().unwrap();
    write.write_all((json!({"jsonrpc":"2.0","id":1,"method":"session/new","params":{"cwd":temp.path(),"mcpServers":[]}}).to_string()+"\n").as_bytes()).await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        while !marker.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("profile command never started");
    write.shutdown().await.unwrap();
    drop(write);
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), child.wait()).await;
    let escaped = std::fs::read_to_string(&marker)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    unsafe {
        libc::kill(-escaped, libc::SIGKILL);
    }
    if result.is_err() {
        let _ = child.kill().await;
        let mut bytes = vec![];
        use tokio::io::AsyncReadExt;
        child
            .stderr
            .take()
            .unwrap()
            .read_to_end(&mut bytes)
            .await
            .unwrap();
        panic!(
            "EOF waited for the key command: {}",
            String::from_utf8_lossy(&bytes)
        );
    }
    assert!(result.unwrap().unwrap().success());
}
#[tokio::test]
async fn permission_rpc_errors_use_the_sessions_unattended_policy() {
    for allow in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("input.txt"), "before\n").unwrap();
        let mut c = client_policy(
            &temp,
            vec![support::sse("edit"), support::sse("text")],
            if allow {
                Unattended::Allow
            } else {
                Unattended::Deny
            },
        )
        .await;
        c.request(1, "session/new", json!({"cwd":temp.path(),"mcpServers":[]}))
            .await;
        let sid = c.next().await["result"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();
        c.request(
            2,
            "session/prompt",
            json!({"sessionId":sid,"prompt":[{"type":"text","text":"edit"}]}),
        )
        .await;
        loop {
            let value = c.next().await;
            if value["method"] == "session/request_permission" {
                c.send(json!({"jsonrpc":"2.0","id":value["id"],"error":{"code":-32603,"message":"client unavailable"}})).await;
            }
            if value["id"] == 2 {
                assert_eq!(value["result"]["stopReason"], "end_turn");
                break;
            }
        }
        assert_eq!(
            std::fs::read_to_string(temp.path().join("input.txt")).unwrap(),
            if allow { "after\n" } else { "before\n" }
        );
        c.close().await;
    }
}
