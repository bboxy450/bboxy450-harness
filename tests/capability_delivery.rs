// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(any(feature = "test-fixtures", feature = "standalone"))]
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

struct Fixture {
    own: tempfile::TempDir,
    value: String,
    file: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let own = tempfile::Builder::new()
            .prefix("capability-side-b-")
            .tempdir_in("/tmp")
            .unwrap();
        let value = format!("{:032x}{:032x}", ulid::Ulid::new().0, ulid::Ulid::new().0);
        let file = own.path().join("capability");
        std::fs::write(&file, format!("{value}\n")).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        Self { own, value, file }
    }
    fn root(&self) -> &Path {
        self.own.path()
    }
}

#[cfg(feature = "test-fixtures")]
mod processes {
    use super::*;
    use bboxy450_harness::protocol::{Envelope, Event, Item, OutputBlock, laws::check_transcript};
    use serde_json::{Value, json};
    use std::{process::Stdio, time::Duration};
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines};

    fn quote(path: &Path) -> String {
        format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
    }
    fn round(delta: Value, finish: &str) -> String {
        format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({"choices":[{"index":0,"delta":delta,"finish_reason":null}]}),
            json!({"choices":[{"index":0,"delta":{},"finish_reason":finish}]})
        )
    }
    fn command(f: &Fixture, face: &str, tool: bool) -> tokio::process::Command {
        let text = f.root().join("text.sse");
        std::fs::write(&text, round(json!({"content":"all done"}), "stop")).unwrap();
        let mut c = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        c.env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("HOME", f.root())
            .env("TMPDIR", f.root())
            .env("BBOXY_CAPABILITY_FILE", &f.file)
            .env("BBOXY_TEST_FORBIDDEN", "namespace-marker")
            .env("BBOXY_SEAT_ID", "private-seat")
            .arg("--cwd")
            .arg(f.root())
            .arg("--sessions")
            .arg(f.root().join("sessions"))
            .arg("--yolo")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        match face {
            "json" => {
                c.args(["-p", "go", "--json"]);
            }
            "seat" | "acp" => {
                c.arg(face);
            }
            _ => panic!("unknown private face"),
        }
        if tool {
            let program = format!(
                "printf 'ENV_BEGIN\\n'; printenv; printf 'ENV_END\\nPATH_BEGIN\\n'; cat \"$BBOXY_CAPABILITY_FILE\" 2>/dev/null || true; printenv BBOXY_CAPABILITY_FILE || true; printf '\\nPATH_END\\nLITERAL_BEGIN\\n'; cat {}; printf 'LITERAL_END\\n'",
                quote(&f.file)
            );
            let scripted = f.root().join("tool.sse");
            std::fs::write(&scripted, round(json!({"tool_calls":[{"index":0,"id":"capability-probe","type":"function","function":{"name":"bash","arguments":json!({"command":program}).to_string()}}]}), "tool_calls")).unwrap();
            c.arg("--scripted").arg(scripted);
        }
        c.arg("--scripted").arg(text);
        c
    }
    fn events(f: &Fixture) -> Vec<Envelope> {
        let file = std::fs::read_dir(f.root().join("sessions"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .unwrap();
        let events = std::fs::read_to_string(file)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect::<Vec<_>>();
        check_transcript(&events).unwrap();
        events
    }
    fn session_text(f: &Fixture) -> String {
        let mut text = String::new();
        if let Ok(files) = std::fs::read_dir(f.root().join("sessions")) {
            for file in files
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|e| e == "jsonl"))
            {
                text.push_str(&std::fs::read_to_string(file.path()).unwrap_or_default());
            }
        }
        text
    }
    fn assert_no_stored_value(f: &Fixture, wire: &str) {
        assert!(
            !wire.contains(&f.value),
            "CapabilityFileValueIsRedacted: face output contains the value"
        );
        let mut pending = vec![f.root().join("sessions")];
        while let Some(path) = pending.pop() {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    let bytes = std::fs::read(path).unwrap();
                    assert!(
                        !bytes
                            .windows(f.value.len())
                            .any(|b| b == f.value.as_bytes()),
                        "CapabilityFileValueIsRedacted: session file contains the value"
                    );
                }
            }
        }
        assert!(
            std::fs::read_to_string(&f.file).unwrap() == format!("{}\n", f.value),
            "capability reader changed the owned file"
        );
    }
    fn assert_tool(f: &Fixture, wire: &str) {
        assert_no_stored_value(f, wire);
        let text = events(f)
            .into_iter()
            .filter_map(|e| match e.event {
                Event::ItemCompleted {
                    body: Item::ToolCall { output, .. },
                    ..
                } => Some(output),
                _ => None,
            })
            .flatten()
            .filter_map(|b| match b {
                OutputBlock::Text { text } => Some(text),
                _ => None,
            })
            .collect::<String>();
        assert!(
            text.contains("LITERAL_BEGIN\n<redacted>\nLITERAL_END"),
            "CapabilityFileValueIsRedacted: direct file output was not scrubbed"
        );
        assert!(
            text.contains("PATH_BEGIN\n\nPATH_END"),
            "tool inherited its capability path"
        );
        assert!(
            !text.contains("BBOXY_"),
            "tool inherited a reserved namespace entry"
        );
        assert!(!text.contains("namespace-marker"));
    }
    async fn json_face(c: &mut tokio::process::Command) -> String {
        c.stdin(Stdio::null());
        let output = tokio::time::timeout(Duration::from_secs(30), c.output())
            .await
            .expect("headless hang guard")
            .unwrap();
        assert!(output.status.success(), "private headless process failed");
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
    struct Interactive {
        child: tokio::process::Child,
        input: tokio::process::ChildStdin,
        lines: Lines<BufReader<tokio::process::ChildStdout>>,
        stderr: tokio::task::JoinHandle<String>,
        wire: String,
        session: Option<Value>,
    }
    impl Interactive {
        async fn start(f: &Fixture, face: &str, tool: bool, servers: Vec<Value>) -> Self {
            let mut child = command(f, face, tool).spawn().unwrap();
            let input = child.stdin.take().unwrap();
            let lines = BufReader::new(child.stdout.take().unwrap()).lines();
            let mut stderr = child.stderr.take().unwrap();
            let stderr = tokio::spawn(async move {
                let mut text = String::new();
                stderr.read_to_string(&mut text).await.unwrap();
                text
            });
            let mut run = Self {
                child,
                input,
                lines,
                stderr,
                wire: String::new(),
                session: None,
            };
            if face == "acp" {
                run.send(json!({"jsonrpc":"2.0","id":1,"method":"session/new","params":{"cwd":f.root(),"mcpServers":servers}})).await;
                loop {
                    let line = run.line().await.expect("ACP new ended early");
                    let value: Value = serde_json::from_str(&line).unwrap();
                    if value["id"] == 1 {
                        run.session = Some(value["result"]["sessionId"].clone());
                        assert!(run.session.as_ref().unwrap().is_string());
                        break;
                    }
                }
            }
            run
        }
        async fn send(&mut self, value: Value) {
            self.input
                .write_all((value.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            self.input.flush().await.unwrap();
        }
        async fn line(&mut self) -> Option<String> {
            let line = tokio::time::timeout(Duration::from_secs(30), self.lines.next_line())
                .await
                .expect("face hang guard")
                .unwrap();
            if let Some(line) = &line {
                self.wire.push_str(line);
                self.wire.push('\n');
            }
            line
        }
        async fn finish(mut self) -> String {
            if let Some(session) = self.session.clone() {
                self.send(json!({"jsonrpc":"2.0","id":2,"method":"session/prompt","params":{"sessionId":session,"prompt":[{"type":"text","text":"go"}]}})).await;
                loop {
                    let value: Value =
                        serde_json::from_str(&self.line().await.expect("ACP prompt ended early"))
                            .unwrap();
                    if value["id"] == 2 {
                        assert_eq!(value["result"]["stopReason"], "end_turn");
                        break;
                    }
                }
            } else {
                self.input.write_all(b"go\n").await.unwrap();
                self.input.flush().await.unwrap();
                loop {
                    if self
                        .line()
                        .await
                        .expect("seat ended early")
                        .contains("EndTurn")
                    {
                        break;
                    }
                }
            }
            self.input.shutdown().await.unwrap();
            drop(self.input);
            tokio::time::timeout(Duration::from_secs(30), async {
                while let Some(line) = self.lines.next_line().await.unwrap() {
                    self.wire.push_str(&line);
                    self.wire.push('\n');
                }
                assert!(self.child.wait().await.unwrap().success());
                self.wire.push_str(&self.stderr.await.unwrap());
            })
            .await
            .expect("face close hang guard");
            self.wire
        }
    }
    #[tokio::test]
    async fn capability_file_value_is_redacted_in_real_headless_tool_and_log() {
        let f = Fixture::new();
        let wire = json_face(&mut command(&f, "json", true)).await;
        assert_tool(&f, &wire);
    }
    #[tokio::test]
    async fn seat_loads_capability_file_before_tool_output() {
        let f = Fixture::new();
        let run = Interactive::start(&f, "seat", true, vec![]).await;
        assert_tool(&f, &run.finish().await);
    }
    #[cfg(feature = "acp")]
    #[tokio::test]
    async fn acp_loads_capability_file_before_tool_output() {
        let f = Fixture::new();
        let run = Interactive::start(&f, "acp", true, vec![]).await;
        assert_tool(&f, &run.finish().await);
    }
    #[tokio::test]
    async fn missing_and_unsafe_files_emit_safe_notices_and_continue() {
        for (case, reason) in [
            ("missing", "missing"),
            ("symlink", "symlink"),
            ("mode", "wrong_mode"),
            ("malformed", "malformed"),
            ("directory", "not_regular"),
        ] {
            let f = Fixture::new();
            let rejected = f.root().join("rejected-path-marker");
            match case {
                "missing" => {}
                "symlink" => std::os::unix::fs::symlink(&f.file, &rejected).unwrap(),
                "mode" => {
                    std::fs::copy(&f.file, &rejected).unwrap();
                    std::fs::set_permissions(&rejected, std::fs::Permissions::from_mode(0o644))
                        .unwrap();
                }
                "malformed" => {
                    std::fs::write(&rejected, "invalid-admission-content").unwrap();
                    std::fs::set_permissions(&rejected, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                }
                "directory" => std::fs::create_dir(&rejected).unwrap(),
                _ => unreachable!(),
            }
            let wire =
                json_face(command(&f, "json", false).env("BBOXY_CAPABILITY_FILE", &rejected)).await;
            assert!(wire.contains("all done"));
            let notices = events(&f)
                .into_iter()
                .filter_map(|e| match e.event {
                    Event::ItemCompleted {
                        body: Item::Notice { text, .. },
                        ..
                    } => Some(text),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert!(
                notices
                    .iter()
                    .any(|text| text.contains(&format!("seat_admission_file_{reason}")))
            );
            assert!(
                notices
                    .iter()
                    .all(|text| !text.contains("rejected-path-marker")
                        && !text.contains(&f.value)
                        && !text.contains("invalid-admission-content"))
            );
        }
        let f = Fixture::new();
        let wire = json_face(command(&f, "json", false).env_remove("BBOXY_CAPABILITY_FILE")).await;
        assert!(wire.contains("seat_admission_file_missing") && wire.contains("all done"));
    }
    #[tokio::test]
    async fn optional_live_smoke_checks_the_file_before_its_key_skip() {
        let f = Fixture::new();
        std::fs::set_permissions(&f.file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", f.root())
            .env("BBOXY_CAPABILITY_FILE", &f.file)
            .arg("smoke-live")
            .output()
            .await
            .unwrap();
        assert!(output.status.success());
        let wire = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(wire.contains("SKIP") && wire.contains("seat_admission_file_wrong_mode"));
        assert!(!wire.contains(&f.value) && !wire.contains(f.file.to_str().unwrap()));
    }
    #[tokio::test]
    async fn profile_and_acp_refuse_a_resolved_value_equal_to_the_capability() {
        for face in ["seat", "acp"]
            .into_iter()
            .filter(|face| *face != "acp" || cfg!(feature = "acp"))
        {
            let f = Fixture::new();
            let report = f.root().join("server-must-not-start.json");
            let config = json!({"command":env!("CARGO_BIN_EXE_mcp-fixture"),"args":["echo_env"],"env":{"MCP_ENV_REPORT":report,"MCP_SECRET":format!("!cat {}", quote(&f.file))},"timeout_s":2});
            let mut acp = vec![];
            if face == "seat" {
                let dir = f.root().join(".bboxy450-harness/profiles");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("nemotron.md"), format!("---\n{}---\n", serde_yaml::to_string(&json!({"name":"nemotron","model":"scripted","endpoint":"http://127.0.0.1:9/v1","api_key":"unused-test","mcpServers":{"probe":config}})).unwrap())).unwrap();
            } else {
                acp.push(json!({"name":"probe","command":config["command"],"args":config["args"],"env":config["env"].as_object().unwrap().iter().map(|(name,value)|json!({"name":name,"value":value})).collect::<Vec<_>>()}));
            }
            let run = Interactive::start(&f, face, false, acp).await;
            tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    if session_text(&f).contains("reserved capability in environment") {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("reserved resolved capability was not refused");
            assert!(
                !report.exists(),
                "reserved resolved value reached a server process"
            );
            let wire = run.finish().await;
            assert_no_stored_value(&f, &wire);
        }
    }
}

#[cfg(feature = "standalone")]
mod gui {
    use super::*;
    #[test]
    fn gui_smoke_loads_file_and_renders_the_same_frame_as_explicit_redaction() {
        let f = Fixture::new();
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gui/rich-all-items.jsonl"),
        )
        .unwrap();
        assert!(source.contains("I’ll read both files."));
        let original = source.replace(
            "I’ll read both files.",
            &format!("I’ll read both files. {}", f.value),
        );
        let expected = original.replace(&f.value, "<redacted>");
        let mut screenshots = vec![];
        for (name, text) in [("loaded", original), ("redacted", expected)] {
            let root = f.root().join(name);
            std::fs::create_dir(&root).unwrap();
            let replay = root.join("fixture.jsonl");
            std::fs::write(&replay, text).unwrap();
            let screenshot = root.join("frame.png");
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
                .env_clear()
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .env("HOME", f.root())
                .env("TMPDIR", f.root())
                .env("BBOXY_CAPABILITY_FILE", &f.file)
                .args(["gui", "--replay"])
                .arg(replay)
                .args(["--headless", "--screenshot"])
                .arg(&screenshot)
                .output()
                .unwrap();
            assert!(output.status.success(), "private GUI smoke process failed");
            assert!(!String::from_utf8_lossy(&output.stdout).contains(&f.value));
            assert!(!String::from_utf8_lossy(&output.stderr).contains(&f.value));
            screenshots.push(std::fs::read(screenshot).unwrap());
        }
        assert!(
            screenshots[0] == screenshots[1],
            "GuiSmokeRegistersCapabilityBeforeRendering: loaded and explicitly redacted frames differ"
        );
    }
}
