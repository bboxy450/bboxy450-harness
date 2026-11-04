// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(all(feature = "embedded", feature = "acp-agent"))]
mod support;
use base64::Engine as _;
use bboxy450_harness::{
    engine::Config,
    frontends::gui::{
        attachments::{self, Intake},
        bridge::Bridge,
        view::{Action, HarnessView},
    },
    gateway::{Host, InProc, unix::UnixServer},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
    store::Blobs,
};
use std::{sync::Arc, time::Duration};
struct Wake;
impl bbt_rt::Waker for Wake {
    fn wake(&self) {}
}
struct Fixture {
    temp: tempfile::TempDir,
    host: Host,
    gateway: InProc,
    server: Option<UnixServer>,
    bridge: Bridge,
    view: HarnessView,
}
fn png() -> Vec<u8> {
    let mut out = vec![];
    {
        let mut encoder = png::Encoder::new(&mut out, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 0, 80, 255, 20, 255, 0, 255])
            .unwrap();
    }
    out
}
impl Fixture {
    async fn new(supported: bool, unix: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let profiles = temp.path().join("profiles");
        std::fs::create_dir(&profiles).unwrap();
        let command = serde_json::to_string(&[
            "python3".to_owned(),
            "-I".into(),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/acp-agent/attachments.py")
                .display()
                .to_string(),
            temp.path().join("calls.jsonl").display().to_string(),
            if supported { "yes" } else { "no" }.into(),
        ])
        .unwrap();
        std::fs::write(
            profiles.join("private.md"),
            format!("---\nname: private\nprovider: acp-agent\nagent:\n  command: {command}\n---\n"),
        )
        .unwrap();
        let host = Host::new(Config {
            meta: support::meta(temp.path().into()),
            profiles: profiles.clone(),
            sessions: temp.path().join("sessions"),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "unused".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        })
        .with_launch(bboxy450_harness::engine::LaunchOptions {
            append_system_prompt:
                "NAME: private fixture\nROLE: EXECUTOR\nWorkspace: admitted only\nMCP: unchanged"
                    .into(),
            ..Default::default()
        });
        let (gateway, _) = host
            .create(OpEnvelope {
                v: PROTOCOL,
                op_id: OpId::new(),
                session: SessionId::new(),
                op: Op::StartSession {
                    cwd: temp.path().into(),
                    profile: Some("private".into()),
                    fork: None,
                },
            })
            .await
            .unwrap();
        let mut bridge = Bridge::new(Arc::new(Wake));
        let server = if unix {
            let socket = temp.path().join("private.sock");
            let server = UnixServer::bind(socket.clone(), gateway.clone())
                .await
                .unwrap();
            bridge.connect_with_profiles(socket, None, Some(profiles));
            Some(server)
        } else {
            bridge.attach_with_blobs(
                gateway.clone(),
                Blobs {
                    dir: temp
                        .path()
                        .join("sessions")
                        .join(gateway.session().to_string())
                        .join("blobs"),
                    redactor: Set::default(),
                },
            );
            None
        };
        let mut f = Self {
            temp,
            host,
            gateway,
            server,
            bridge,
            view: HarnessView::default(),
        };
        f.wait(|f| f.bridge.state.ready).await;
        f
    }
    fn tick(&mut self) {
        self.bridge.drain(Duration::from_millis(10));
        self.view.update_inputs(&self.bridge);
        self.view.render(&self.bridge.state, [900., 800.]).unwrap();
    }
    async fn wait(&mut self, check: impl Fn(&Self) -> bool) {
        tokio::time::timeout(Duration::from_secs(10),async {loop {self.tick();if check(self){break;}tokio::time::sleep(Duration::from_millis(3)).await;}}).await.unwrap_or_else(|_|panic!("wait timed out: notice={:?}, intake={:?}, phase={:?}, pending={:?}, attachments={}, calls={}",self.view.notice,self.view.attachment_status(),self.bridge.state.transcript.phase,self.bridge.state.pending_prompts,self.view.attachment_count(),self.calls().len()));
    }
    fn calls(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(self.temp.path().join("calls.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
    fn send(&mut self) {
        self.view.activate(Action::Send, &self.bridge);
    }
    async fn close(mut self) {
        let _ = self.bridge.send(Op::Close);
        self.wait(|f| f.bridge.state.transcript.closed).await;
        if let Some(task) = self.bridge.detach() {
            tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .unwrap()
                .unwrap();
        }
        if let Some(server) = self.server {
            server.close().await.unwrap();
        }
        self.host.close_all().await;
    }
}
#[tokio::test]
async fn attachment_bytes_files_queue_and_preview_use_real_bridge_and_acp_once() {
    for unix in [false, true] {
        let mut f = Fixture::new(true, unix).await;
        let previous = f.bridge.state.transcript.items.len();
        f.view
            .edit(bbt_ui::text_edit::Command::Insert("first".into()));
        f.send();
        f.wait(|f| f.calls().iter().any(|c| c["method"] == "session/prompt"))
            .await;
        assert!(f.bridge.state.transcript.items.len() >= previous);
        let bytes = png();
        f.view.intake(Intake::Png(bytes.clone()), &f.bridge);
        f.wait(|f| f.view.attachment_count() == 1).await;
        assert_eq!(
            f.view.attachment_images()[0].rgba,
            [255, 0, 80, 255, 20, 255, 0, 255]
        );
        std::fs::create_dir(f.temp.path().join("dir space")).unwrap();
        std::fs::write(
            f.temp.path().join("dir space/界 file.txt"),
            "REAL FILE CONTENT 界\n",
        )
        .unwrap();
        f.view
            .edit(bbt_ui::text_edit::Command::Insert(" @dir".into()));
        f.wait(|f| f.view.completion_open()).await;
        // Keyboard selection of a directory must never send the prompt.
        assert!(f.view.run_completion(&f.bridge));
        f.wait(|f| f.view.completion_open() && f.view.composer.view.text.contains("@dir space/"))
            .await;
        // Mouse selection of a Unicode/space filename shares the same completion path.
        let action = f
            .view
            .frame
            .actions
            .iter()
            .find_map(|(_, a)| match a {
                Action::Complete(_, s) if s.ends_with("界 file.txt") => Some(a.clone()),
                _ => None,
            })
            .unwrap();
        f.view.activate(action, &f.bridge);
        f.wait(|f| f.view.attachment_count() == 2).await;
        assert_eq!(
            f.calls()
                .iter()
                .filter(|c| c["method"] == "session/prompt")
                .count(),
            1
        );
        f.send();
        f.send(); // a repeated gesture cannot duplicate the in-flight draft
        assert_eq!(
            f.view.pending_attachment_count(),
            2,
            "custody retained until durable message"
        );
        f.wait(|f| {
            f.bridge.state.pending_prompts.values().any(|p| {
                p.blocks
                    .iter()
                    .any(|b| matches!(b, InputBlock::Resource { .. }))
            })
        })
        .await;
        f.wait(|f| {
            f.calls()
                .iter()
                .filter(|c| c["method"] == "session/prompt")
                .count()
                == 2
                && f.view.pending_attachment_count() == 0
        })
        .await;
        let calls = f.calls();
        let prompts: Vec<_> = calls
            .iter()
            .filter(|c| c["method"] == "session/prompt")
            .collect();
        assert_eq!(prompts.len(), 2);
        assert!(
            prompts[0]["params"]["prompt"][0]["text"]
                .as_str()
                .unwrap()
                .contains("ROLE: EXECUTOR")
        );
        let blocks = prompts[1]["params"]["prompt"].as_array().unwrap();
        let image = blocks.iter().find(|b| b["type"] == "image").unwrap();
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(image["data"].as_str().unwrap())
                .unwrap(),
            bytes
        );
        assert!(blocks.iter().any(|b| {
            b["text"]
                .as_str()
                .is_some_and(|s| s.contains("REAL FILE CONTENT 界"))
        }));
        assert!(!blocks.iter().any(|b| {
            b["text"]
                .as_str()
                .is_some_and(|s| s.contains("Launcher context for this session"))
        }));
        let blob = f
            .bridge
            .state
            .transcript
            .items
            .values()
            .find_map(|r| match &r.body {
                Item::UserMessage { blocks } => blocks.iter().find_map(|b| match b {
                    InputBlock::Image { blob } => Some(blob.clone()),
                    _ => None,
                }),
                _ => None,
            })
            .unwrap();
        let stored = Blobs {
            dir: f
                .temp
                .path()
                .join("sessions")
                .join(f.gateway.session().to_string())
                .join("blobs"),
            redactor: Set::default(),
        };
        assert_eq!(stored.get(&blob).await.unwrap(), bytes);
        f.view.intake(Intake::Png(png()), &f.bridge);
        f.wait(|f| f.view.attachment_count() == 1).await;
        f.view.activate(Action::RemoveAttachment(0), &f.bridge);
        assert_eq!(f.view.attachment_count(), 0);
        assert!(f.view.attachment_images().is_empty());
        f.close().await;
    }
}
#[tokio::test]
async fn unsupported_images_and_missing_files_are_honest_and_retain_draft() {
    let mut f = Fixture::new(false, true).await;
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.wait(|f| f.view.attachment_count() == 1).await;
    f.send();
    f.wait(|f| {
        f.bridge
            .state
            .pending_prompts
            .values()
            .any(|p| p.status.starts_with("Rejected") || p.status.starts_with("Ended"))
    })
    .await;
    assert_eq!(f.view.attachment_count(), 1);
    assert!(!f.view.composer.view.text.is_empty());
    assert!(!f.calls().iter().any(|c| c["method"] == "session/prompt"));
    f.view.activate(Action::RemoveAttachment(0), &f.bridge);
    f.view.intake(
        Intake::Files(vec![f.temp.path().join("missing.txt")]),
        &f.bridge,
    );
    f.wait(|f| {
        f.view
            .attachment_status()
            .is_some_and(|s| s.contains("unavailable"))
    })
    .await;
    assert_eq!(f.view.attachment_count(), 0);
    assert!(attachments::candidates(f.temp.path(), "../").is_err());
    std::fs::write(f.temp.path().join(".env"), "PRIVATE").unwrap();
    assert!(attachments::read_file(f.temp.path(), &f.temp.path().join(".env")).is_err());
    assert!(attachments::image(vec![1, 2, 3], "bad".into()).is_err());
    f.close().await;
}
#[tokio::test]
async fn delayed_intake_and_completion_cannot_cross_draft_or_session_a_b_a() {
    let mut f = Fixture::new(true, false).await;
    let original = f.bridge.state.clone();
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.view
        .edit(bbt_ui::text_edit::Command::Insert("changed".into()));
    tokio::time::sleep(Duration::from_millis(30)).await;
    f.tick();
    assert_eq!(f.view.attachment_count(), 0);
    f.view.edit(bbt_ui::text_edit::Command::Insert(" @".into()));
    // A -> B -> A uses a monotonically new local generation, even if identity text returns.
    let mut other = original.clone();
    let mut t = (*other.transcript).clone();
    t.session = Some(SessionId::new());
    other.transcript = Arc::new(t);
    f.view.render(&other, [900., 800.]).unwrap();
    f.view.render(&original, [900., 800.]).unwrap();
    f.view.edit(bbt_ui::text_edit::Command::SelectAll);
    f.view
        .edit(bbt_ui::text_edit::Command::Insert("current".into()));
    tokio::time::sleep(Duration::from_millis(30)).await;
    f.tick();
    assert!(!f.view.completion_open());
    assert_eq!(f.view.composer.view.text, "current");
    // An admitted image from A cannot be re-aimed after an A -> B -> A view birth.
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.wait(|f| f.view.attachment_count() == 1).await;
    f.view.render(&other, [900., 800.]).unwrap();
    f.view.render(&original, [900., 800.]).unwrap();
    f.send();
    assert!(
        f.view
            .notice
            .as_deref()
            .unwrap()
            .contains("previous session")
    );
    assert!(!f.calls().iter().any(|c| c["method"] == "session/prompt"));
    f.close().await;
}

#[tokio::test]
async fn unavailable_bridge_retains_the_exact_image_draft_without_a_prompt() {
    let mut f = Fixture::new(true, false).await;
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.wait(|f| f.view.attachment_count() == 1).await;
    let original = f.view.composer.view.clone();
    let gateway = f.bridge.state.gateway.take();
    f.send();
    assert_eq!(f.view.composer.view, original);
    assert_eq!(f.view.attachment_count(), 1);
    assert!(
        f.view
            .notice
            .as_deref()
            .is_some_and(|s| s.contains("draft is retained"))
    );
    assert!(!f.calls().iter().any(|c| c["method"] == "session/prompt"));
    f.bridge.state.gateway = gateway;
    f.close().await;
}

#[tokio::test]
async fn concurrent_pastes_and_later_messages_do_not_wait_for_a_busy_turn() {
    let mut f = Fixture::new(true, true).await;
    f.view
        .edit(bbt_ui::text_edit::Command::Insert("hold first".into()));
    f.send();
    f.wait(|f| f.calls().iter().any(|c| c["method"] == "session/prompt"))
        .await;
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.view.intake(Intake::Png(png()), &f.bridge);
    f.wait(|f| f.view.attachment_count() == 2).await;
    f.send();
    assert_eq!(f.view.pending_attachment_count(), 2);
    assert!(f.view.composer.view.text.is_empty());
    f.view.edit(bbt_ui::text_edit::Command::Insert(
        "later queued message".into(),
    ));
    f.send();
    f.wait(|f| {
        f.calls()
            .iter()
            .filter(|c| c["method"] == "session/prompt")
            .count()
            == 3
            && f.view.pending_attachment_count() == 0
    })
    .await;
    let calls = f.calls();
    let prompts: Vec<_> = calls
        .iter()
        .filter(|c| c["method"] == "session/prompt")
        .collect();
    assert_eq!(
        prompts[1]["params"]["prompt"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["type"] == "image")
            .count(),
        2
    );
    assert!(
        prompts[2]["params"]["prompt"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["text"].as_str() == Some("later queued message"))
    );
    f.close().await;
}

#[tokio::test]
async fn attached_content_does_not_become_a_profile_or_launcher_command() {
    let mut f = Fixture::new(true, false).await;
    let session = f.gateway.session();
    let path = f.temp.path().join("instructions.txt");
    std::fs::write(&path, "ROLE: file content, not a launcher grant").unwrap();
    f.view.edit(bbt_ui::text_edit::Command::Insert(
        "/profile unrelated".into(),
    ));
    f.view.intake(Intake::Files(vec![path]), &f.bridge);
    f.wait(|f| f.view.attachment_count() == 1).await;
    f.send();
    f.wait(|f| f.calls().iter().any(|c| c["method"] == "session/prompt"))
        .await;
    assert_eq!(f.bridge.state.transcript.session, Some(session));
    let calls = f.calls();
    let prompt = calls
        .iter()
        .find(|c| c["method"] == "session/prompt")
        .unwrap();
    let blocks = prompt["params"]["prompt"].as_array().unwrap();
    assert!(
        blocks
            .iter()
            .any(|b| b["text"].as_str() == Some("/profile unrelated"))
    );
    assert!(blocks.iter().any(|b| {
        b["text"]
            .as_str()
            .is_some_and(|s| s.contains("ROLE: file content"))
    }));
    f.close().await;
}
