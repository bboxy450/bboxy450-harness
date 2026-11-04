// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![cfg(feature = "acp-agent")]
mod support;
use bboxy450_harness::{
    engine::Config,
    frontends::gui::{bridge::Bridge, model::Transcript},
    gateway::{Gateway, Host, InProc, Subscribe, Subscription},
    policy::Policy,
    protocol::*,
    provider::{ScriptedProvider, profile::Unattended},
    redact::Set,
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
    sub: Subscription,
}
impl Fixture {
    async fn new(scenario: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let profiles = temp.path().join("profiles");
        std::fs::create_dir(&profiles).unwrap();
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/acp-agent/config_agent.py");
        let command = serde_json::to_string(&vec![
            "python3".to_owned(),
            "-I".into(),
            script.display().to_string(),
            scenario.into(),
            temp.path().join("calls.jsonl").display().to_string(),
        ])
        .unwrap();
        std::fs::write(
            profiles.join("private.md"),
            format!("---\nname: private\nprovider: acp-agent\nagent:\n  command: {command}\n---\n"),
        )
        .unwrap();
        if scenario == "bootstrap-reorder" {
            let path = profiles.join("private.md");
            let text = std::fs::read_to_string(&path)
                .unwrap()
                .replace("provider: acp-agent", "provider: acp-agent\nmodel: beta");
            std::fs::write(path, text).unwrap();
        }
        let host = Host::new(Config {
            meta: support::meta(temp.path().into()),
            profiles,
            sessions: temp.path().join("sessions"),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        });
        let (gateway, _) = host
            .create(OpEnvelope {
                v: PROTOCOL,
                session: SessionId::new(),
                op_id: OpId::new(),
                op: Op::StartSession {
                    cwd: temp.path().into(),
                    profile: Some("private".into()),
                    fork: None,
                },
            })
            .await
            .unwrap();
        let sub = gateway
            .subscribe(Subscribe {
                session: gateway.session(),
                after: None,
                ephemeral: true,
            })
            .unwrap();
        gateway
            .send(support::op(
                &gateway,
                Op::UserTurn {
                    blocks: vec![InputBlock::Text {
                        text: "only prompt".into(),
                    }],
                    mode: TurnMode::Queue,
                },
            ))
            .unwrap();
        let mut f = Self {
            temp,
            host,
            gateway,
            sub,
        };
        loop {
            if matches!(f.next().await.event, Event::TurnEnded { .. }) {
                break;
            }
        }
        f
    }
    async fn next(&mut self) -> Envelope {
        tokio::time::timeout(Duration::from_secs(8), self.sub.recv())
            .await
            .unwrap()
            .unwrap()
    }
    fn state(&self) -> AgentConfig {
        let mut t = Transcript::default();
        let path = self
            .temp
            .path()
            .join("sessions")
            .join(format!("{}.jsonl", self.gateway.session()));
        for line in std::fs::read_to_string(path).unwrap().lines() {
            t.apply(&serde_json::from_str::<Envelope>(line).unwrap());
        }
        t.agent_config.unwrap()
    }
    fn select(&self, c: &AgentConfig, id: &str, value: &str) -> OpEnvelope {
        support::op(
            &self.gateway,
            Op::SetAgentConfig {
                epoch: self.gateway.hello().epoch,
                instance: c.instance,
                revision: c.revision,
                config_id: id.into(),
                value: Some(value.into()),
            },
        )
    }
    async fn settle(&mut self, op: OpEnvelope) -> AgentConfig {
        let id = op.op_id;
        self.gateway.send(op).unwrap();
        loop {
            let e = self.next().await;
            if e.cause == Some(id)
                && matches!(&e.event, Event::ContextChanged { config: Some(c), .. } if c.get("acp_controls").is_some_and(|v| v["pending"].is_null()))
            {
                return self.state();
            }
        }
    }
    fn calls(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(self.temp.path().join("calls.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
    async fn close(self) {
        self.close_with_turns(1).await;
    }
    async fn close_with_turns(self, expected: usize) {
        self.gateway
            .send(support::op(&self.gateway, Op::Close))
            .unwrap();
        tokio::time::timeout(Duration::from_secs(8), self.host.join())
            .await
            .unwrap()
            .unwrap();
        let path = self
            .temp
            .path()
            .join("sessions")
            .join(format!("{}.jsonl", self.gateway.session()));
        let log = std::fs::read_to_string(path).unwrap();
        assert!(!log.contains("PRIVATE-UNPROJECTED"));
        let events: Vec<Envelope> = log
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        bboxy450_harness::protocol::laws::check_authored(&events).unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.event, Event::TurnStarted { .. }))
                .count(),
            expected
        );
        assert_eq!(
            self.calls()
                .iter()
                .filter(|c| c["method"] == "session/prompt")
                .count(),
            expected
        );
        assert!(!self.calls().iter().any(|c| {
            c["method"]
                .as_str()
                .is_some_and(|s| s.starts_with("fs/") || s.starts_with("terminal/"))
        }));
    }
}
fn current(c: &AgentConfig, category: &str) -> String {
    c.options
        .iter()
        .find(|o| o.category == category)
        .unwrap()
        .current
        .clone()
}
#[tokio::test]
async fn acp_controls_real_config_dynamic_refusal_reconnect_and_stale_selections() {
    let mut f = Fixture::new("standard").await;
    let first = f.state();
    assert_eq!(current(&first, "model"), "alpha");
    let op = f.select(&first, "vendor-model", "beta");
    let id = op.op_id;
    f.gateway.send(op).unwrap();
    loop {
        let e = f.next().await;
        if e.cause == Some(id) && matches!(e.event, Event::ContextChanged { .. }) {
            break;
        }
    }
    let pending = f.state();
    assert_eq!(pending.pending, Some(id));
    assert_eq!(current(&pending, "model"), "alpha");
    loop {
        let e = f.next().await;
        if e.cause == Some(id)
            && matches!(&e.event, Event::ContextChanged { config: Some(c), .. } if c["acp_controls"]["pending"].is_null())
        {
            break;
        }
    }
    let beta = f.state();
    assert_eq!(current(&beta, "model"), "beta");
    assert_eq!(
        beta.options[1]
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect::<Vec<_>>(),
        vec!["balanced", "deep"]
    );
    let refused = f.settle(f.select(&beta, "compute", "deep")).await;
    assert!(refused.status.starts_with("Refused"));
    assert_eq!(current(&refused, "thought_level"), "balanced");
    let alpha = f.settle(f.select(&refused, "vendor-model", "alpha")).await;
    assert_eq!(current(&alpha, "model"), "alpha");
    let count = f.calls().len();
    // Same model name after A→B→A is a new option identity.
    let stale = f.select(&first, "compute", "high");
    let stale_id = stale.op_id;
    f.gateway.send(stale).unwrap();
    loop {
        if matches!(f.next().await.event, Event::OpRejected { op, .. } if op == stale_id) {
            break;
        }
    }
    assert_eq!(f.calls().len(), count);
    // A fresh view replays confirmed state without another child, prompt or set call.
    let socket = f.temp.path().join("s.sock");
    let server =
        bboxy450_harness::gateway::unix::UnixServer::bind(socket.clone(), f.gateway.clone())
            .await
            .unwrap();
    for _ in 0..2 {
        let mut bridge = Bridge::new(Arc::new(Wake));
        bridge.connect_with_profiles(socket.clone(), None, Some(f.temp.path().join("profiles")));
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                bridge.drain(Duration::from_millis(1));
                if bridge
                    .state
                    .transcript
                    .agent_config
                    .as_ref()
                    .is_some_and(|c| current(c, "model") == "alpha")
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .unwrap();
        if let Some(worker) = bridge.detach() {
            worker.await.unwrap();
        }
    }
    assert_eq!(f.calls().len(), count);
    server.close().await.unwrap();
    f.close().await;
}
#[tokio::test]
async fn acp_controls_newer_update_wins_over_old_response_and_wrong_session_update() {
    let mut f = Fixture::new("standard").await;
    let state = f.state();
    let state = f.settle(f.select(&state, "vendor-model", "gamma")).await;
    assert_eq!(current(&state, "model"), "alpha");
    assert!(state.status.contains("older response ignored"));
    assert_eq!(state.options[1].choices[0].value, "low");
    f.close().await;
}
#[tokio::test]
async fn acp_controls_legacy_and_unsupported_are_advertisement_driven() {
    let mut f = Fixture::new("legacy").await;
    let state = f.state();
    assert!(state.options[0].legacy);
    let state = f.settle(f.select(&state, "model", "beta")).await;
    assert_eq!(current(&state, "model"), "beta");
    assert_eq!(state.options.len(), 1);
    assert!(
        f.calls()
            .iter()
            .any(|c| c["method"] == "session/set_model" && c["params"]["modelId"] == "beta")
    );
    assert!(
        !f.calls()
            .iter()
            .any(|c| c["method"] == "session/set_config_option")
    );
    f.close().await;
    let mut f = Fixture::new("unsupported").await;
    let state = f.state();
    assert!(state.options.is_empty());
    let op = f.select(&state, "model", "fabricated");
    let id = op.op_id;
    f.gateway.send(op).unwrap();
    loop {
        if matches!(f.next().await.event, Event::OpRejected { op, .. } if op == id) {
            break;
        }
    }
    assert!(
        !f.calls()
            .iter()
            .any(|c| c["method"] == "session/set_config_option")
    );
    f.close().await;
}
#[tokio::test]
async fn acp_controls_unknown_outcome_is_never_replayed() {
    let mut f = Fixture::new("unknown").await;
    let c = f.state();
    let op = f.select(&c, "compute", "high");
    f.gateway.send(op).unwrap();
    loop {
        let e = f.next().await;
        if matches!(&e.event, Event::ContextChanged { config: Some(c), .. } if c["acp_controls"]["status"].as_str().is_some_and(|s| s.starts_with("Unknown outcome")))
        {
            break;
        }
    }
    let c = f.state();
    assert_eq!(current(&c, "thought_level"), "low");
    assert!(c.pending.is_some());
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        f.calls()
            .iter()
            .filter(|c| c["method"] == "session/set_config_option")
            .count(),
        1
    );
    f.close().await;
}

#[test]
fn acp_controls_default_home_path_is_selected_in_private_child() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(profiles.join("isolated.md"),"---\nname: isolated\nprovider: acp-agent\nagent:\n  command: [private-agent]\n  env:\n    PRIVATE_NAME: PRIVATE-VALUE\n---\n").unwrap();
    std::fs::write(
        profiles.parent().unwrap().join("default-profile"),
        "isolated\n",
    )
    .unwrap();
    for mode in ["present", "absent", "empty", "missing", "unreadable"] {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "acp_controls_home_child_entry", "--nocapture"])
            .env("PATH", "") // Discovery must not seed operator-installed adapters.
            .env("ACP_CONTROLS_HOME_CHILD", mode);
        match mode {
            "absent" => {
                command.env_remove("HOME");
            }
            "empty" => {
                command.env("HOME", "");
            }
            "missing" => {
                command.env("HOME", temp.path().join("missing"));
            }
            "unreadable" => {
                let home = temp.path().join("bad-home");
                std::fs::create_dir_all(home.join(".bboxy450-harness")).unwrap();
                std::fs::write(home.join(".bboxy450-harness/profiles"), "not a directory").unwrap();
                command.env("HOME", home);
            }
            _ => {
                command.env("HOME", temp.path());
            }
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "private child {mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("PRIVATE-VALUE"));
    }
}
#[tokio::test]
async fn acp_controls_home_child_entry() {
    let Ok(mode) = std::env::var("ACP_CONTROLS_HOME_CHILD") else {
        return;
    };
    let mut bridge = Bridge::new(Arc::new(Wake));
    // The real installed-host entry point: no explicit catalogue override.
    bridge.connect(
        std::path::PathBuf::from("/private/tmp/acp-controls-nonexistent.sock"),
        None,
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            bridge.drain(Duration::from_millis(1));
            if bridge.state.error.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    if mode == "present" {
        assert!(bridge.state.profile_error.is_none());
        assert_eq!(bridge.state.profiles.len(), 1);
        let p = &bridge.state.profiles[0];
        assert_eq!(p.name, "isolated");
        assert!(p.default);
        assert_eq!(p.agent_env, 1);
        assert!(!format!("{p:?}").contains("PRIVATE-VALUE"));
    } else {
        assert!(
            bridge
                .state
                .profile_error
                .as_ref()
                .unwrap()
                .contains("unavailable")
        );
        assert!(bridge.state.profiles.is_empty());
    }
    if let Some(worker) = bridge.detach() {
        worker.await.unwrap();
    }
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn acp_controls_native_popover_routes_real_selection_and_unavailable_labels() {
    use bboxy450_harness::frontends::gui::{
        bridge::FrameState,
        view::{Action, HarnessView},
    };
    let mut f = Fixture::new("standard").await;
    let config = f.state();
    let mut transcript = Transcript::default();
    transcript.hello(f.gateway.session(), f.gateway.hello().epoch);
    transcript.agent_config = Some(config.clone());
    let state = FrameState {
        transcript: Arc::new(transcript),
        ready: true,
        ..Default::default()
    };
    let mut view = HarnessView::default();
    view.toggle_profile_menu();
    view.render(&state, [700., 900.]).unwrap();
    let action=view.frame.actions.iter().find_map(|(_,a)| match a { Action::AgentConfig(e) if matches!(&e.op,Op::SetAgentConfig { value,.. } if value.as_deref()==Some("beta")) => Some(a.clone()), _=>None }).expect("advertised model button has a real op");
    let stale_action = view.frame.actions.iter().find_map(|(_,a)| match a { Action::AgentConfig(e) if matches!(&e.op,Op::SetAgentConfig { value,.. } if value.as_deref()==Some("gamma")) => Some(a.clone()), _=>None }).unwrap();
    let Action::AgentConfig(op) = action.clone() else {
        unreachable!()
    };
    let c = f.settle(op).await;
    assert_eq!(current(&c, "model"), "beta");
    let mut bridge = Bridge::new(Arc::new(Wake));
    bridge.attach(f.gateway.clone());
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            bridge.drain(Duration::from_millis(1));
            if bridge
                .state
                .transcript
                .agent_config
                .as_ref()
                .is_some_and(|c| current(c, "model") == "beta")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    // A retained action from alpha cannot be re-aimed at the new option revision.
    view.activate(stale_action, &bridge);
    loop {
        if matches!(f.next().await.event, Event::OpRejected { .. }) {
            break;
        }
    }
    let calls = f.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|c| c["method"] == "session/set_config_option")
            .count(),
        1
    );
    let empty = FrameState {
        ready: true,
        profile_error: Some("Profile catalogue unavailable · permission denied".into()),
        ..Default::default()
    };
    view.render(&empty, [700., 900.]).unwrap();
    assert!(
        !view
            .frame
            .actions
            .iter()
            .any(|(_, a)| matches!(a, Action::AgentConfig(_)))
    );
    // Popover pixels are generated by the same native toolkit layout on both surfaces.
    assert!(!view.frame.over.primitives.is_empty());
    if let Some(worker) = bridge.detach() {
        worker.await.unwrap();
    }
    f.close().await;
}

#[tokio::test]
async fn acp_controls_profile_bootstrap_is_revision_fenced() {
    let f = Fixture::new("bootstrap-reorder").await;
    let c = f.state();
    assert_eq!(current(&c, "model"), "alpha");
    assert!(c.status.contains("older response ignored"));
    assert_eq!(
        f.calls()
            .iter()
            .filter(|c| c["method"] == "session/set_config_option")
            .count(),
        1
    );
    f.close().await;
}

#[tokio::test]
async fn acp_controls_epoch_child_and_session_identity_reject_before_wire() {
    let mut f = Fixture::new("standard").await;
    let c = f.state();
    let count = f.calls().len();
    for wrong_child in [true, false] {
        let mut op = f.select(&c, "compute", "high");
        let id = op.op_id;
        if let Op::SetAgentConfig {
            instance, epoch, ..
        } = &mut op.op
        {
            if wrong_child {
                *instance = OpId::new();
            } else {
                epoch.0 += 1;
            }
        }
        f.gateway.send(op).unwrap();
        loop {
            if matches!(f.next().await.event,Event::OpRejected {op,..} if op==id) {
                break;
            }
        }
    }
    let mut op = f.select(&c, "compute", "high");
    op.session = SessionId::new();
    assert!(matches!(
        f.gateway.send(op),
        Err(bboxy450_harness::gateway::Refused::UnknownSession)
    ));
    assert_eq!(f.calls().len(), count);
    f.close().await;
}

#[tokio::test]
async fn acp_controls_missing_fields_reject_before_wire() {
    let mut f = Fixture::new("standard").await;
    let c = f.state();
    assert!(f.gateway.hello().epoch.0 > 0 && c.revision > 0);
    assert_ne!(c.instance.0, ulid::Ulid::nil());
    let count = f.calls().len();
    for missing in [
        vec!["epoch"],
        vec!["instance"],
        vec!["revision"],
        vec!["config_id"],
        vec!["value"],
        vec!["epoch", "instance", "revision", "config_id", "value"],
    ] {
        let mut wire = serde_json::to_value(f.select(&c, "compute", "high")).unwrap();
        for field in &missing {
            wire["op"].as_object_mut().unwrap().remove(*field).unwrap();
        }
        let op: OpEnvelope = serde_json::from_value(wire).unwrap();
        assert!(matches!(op.op, Op::SetAgentConfig { .. }));
        let id = op.op_id;
        f.gateway.send(op).unwrap();
        loop {
            if matches!(f.next().await.event, Event::OpRejected { op, .. } if op == id) {
                break;
            }
        }
        assert_eq!(f.calls().len(), count, "missing {missing:?} reached child");
        let now = f.state();
        assert_eq!(
            (&now.instance, now.revision, &now.options, now.pending),
            (&c.instance, c.revision, &c.options, c.pending),
            "missing {missing:?} changed control identity or selections"
        );
        assert_eq!(
            now.status,
            "Agent configuration unavailable, pending or stale; refresh the controls"
        );
    }
    f.close().await;

    // A provider may advertise an empty string. Absence/null must not select it,
    // while an explicit empty value must retain its valid wire semantics.
    let mut f = Fixture::new("empty-choice").await;
    let c = f.state();
    let count = f.calls().len();
    for null in [false, true] {
        let mut wire = serde_json::to_value(f.select(&c, "compute", "")).unwrap();
        if null {
            wire["op"]["value"] = serde_json::Value::Null;
        } else {
            wire["op"].as_object_mut().unwrap().remove("value").unwrap();
        }
        let op: OpEnvelope = serde_json::from_value(wire).unwrap();
        let id = op.op_id;
        f.gateway.send(op).unwrap();
        loop {
            if matches!(f.next().await.event, Event::OpRejected { op, .. } if op == id) {
                break;
            }
        }
        assert_eq!(f.calls().len(), count, "absent/null value selected empty");
        assert_eq!(current(&f.state(), "thought_level"), "low");
    }
    let op = f.select(&c, "compute", "");
    let confirmed = f.settle(op).await;
    assert_eq!(current(&confirmed, "thought_level"), "");
    f.close().await;
}

#[cfg(feature = "embedded")]
#[tokio::test]
async fn acp_controls_native_raster_displays_confirmed_and_unavailable_states() {
    use bboxy450_harness::frontends::gui::{bridge::FrameState, view::HarnessView};
    let f = Fixture::new("standard").await;
    let config = f.state();
    f.close().await;
    let context = bbt_render::device::DeviceContext::new().await.unwrap();
    bboxy450_harness::runtime::blocking(move || {
        let mut view = HarnessView::default();
        view.toggle_profile_menu();
        let transcript = Transcript {
            agent_config: Some(config),
            ..Transcript::default()
        };
        let state = FrameState {
            transcript: Arc::new(transcript),
            ready: true,
            ..Default::default()
        };
        let mut renderer = bbt_render::chrome::ChromeRenderer::with_context(
            context,
            bboxy450_harness::frontends::gui::assets::icons(),
        )
        .unwrap();
        let mut captures = vec![];
        for (name, state, expected) in [
            ("agent-options", state, "Agent model: alpha"),
            (
                "agent-unavailable",
                FrameState {
                    ready: true,
                    profile_error: Some("Profile catalogue unavailable · permission denied".into()),
                    ..Default::default()
                },
                "Reasoning effort: unavailable",
            ),
        ] {
            let frame = view.render(&state, [700., 900.]).unwrap();
            let caption = frame
                .over
                .primitives
                .iter()
                .filter_map(|p| match p {
                    bbt_ui_model::draw::Prim::Text { text, .. } => Some(text.as_ref()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            let caption = caption.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(
                caption.contains(expected),
                "{name}: expected {expected:?}; painted {caption:?}"
            );
            let mut list = frame.list.clone();
            list.primitives
                .extend(frame.over.primitives.iter().cloned());
            let rgba = renderer.render_to_rgba(&list, 700, 900, 1.).unwrap();
            assert!(
                rgba.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|p| [p[0], p[1], p[2]])
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    > 20
            );
            if let Some(dir) = std::env::var_os("PROFILES_PENDING_CAPTURE_DIR") {
                let dir = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
                let mut encoder = png::Encoder::new(file, 700, 900);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&rgba)
                    .unwrap();
            }
            captures.push(rgba);
        }
        assert_ne!(
            captures[0], captures[1],
            "confirmed/unavailable states must change actual pixels"
        );
    })
    .await;
}

#[tokio::test]
async fn acp_controls_real_child_restart_fences_the_prior_incarnation() {
    let mut f = Fixture::new("restart").await;
    let path = f
        .temp
        .path()
        .join("sessions")
        .join(format!("{}.jsonl", f.gateway.session()));
    let old = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .find_map(|line| {
            let e: Envelope = serde_json::from_str(line).unwrap();
            match e.event {
                Event::ContextChanged {
                    config: Some(c), ..
                } => c
                    .get("acp_controls")
                    .and_then(|v| serde_json::from_value::<AgentConfig>(v.clone()).ok())
                    .filter(|c| !c.options.is_empty()),
                _ => None,
            }
        })
        .unwrap();
    while !f.state().options.is_empty() {
        f.next().await;
    }
    f.gateway
        .send(support::op(
            &f.gateway,
            Op::UserTurn {
                blocks: vec![InputBlock::Text {
                    text: "second explicit turn".into(),
                }],
                mode: TurnMode::Queue,
            },
        ))
        .unwrap();
    loop {
        if matches!(f.next().await.event, Event::TurnEnded { .. }) {
            break;
        }
    }
    let new = f.state();
    assert_ne!(old.instance, new.instance);
    assert_eq!(current(&new, "model"), "alpha");
    let count = f.calls().len();
    let op = f.select(&old, "compute", "high");
    let id = op.op_id;
    f.gateway.send(op).unwrap();
    loop {
        if matches!(f.next().await.event,Event::OpRejected{op,..} if op==id) {
            break;
        }
    }
    assert_eq!(f.calls().len(), count);
    f.close_with_turns(2).await;
}
