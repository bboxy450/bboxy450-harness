// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! ACP agent backend: the harness as the UI for any ACP agent (Claude Agent, Codex,
//! Gemini, goose, opencode). The agent owns its loop, tools and permissions, so this
//! replaces the engine's session actor behind the same Gateway; every face renders the
//! same `Item`s and drives the session with the same `Op`s.
//!
//! The client side is newline-delimited JSON-RPC over the child's stdio, like the face in
//! `frontends/acp.rs`; the schema crate only validates. Protocol rules held here
//! (ICD `.bboxy/interfaces/acp-agent.md`):
//! - every agent→client request is answered; unsupported ones get -32601;
//! - after `session/cancel`, every pending permission is answered `cancelled` and updates
//!   are still accepted until the prompt returns (`stopReason: cancelled`);
//! - one `TurnEnded` per prompt; no item stays open at a turn's end;
//! - yolo prefers `allow_once`, falling back to `allow_always`; ask/unattended stay explicit;
//! - the harness never offers, selects or triggers an agent's claude.ai login method.
use super::acp_permissions;
use crate::{
    engine::{Config, Internal, LaunchOptions, task_group::TaskGroup},
    gateway::{Admitted, Hub, InProc},
    process::OwnedChild,
    protocol::*,
    provider::profile::{AgentSpec, Unattended},
    store::Store,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    ffi::{OsStr, OsString},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::ChildStdin,
    sync::mpsc,
    time::Instant,
};

/// One JSON-RPC line from the agent may not exceed this; a longer line ends the agent.
pub const MAX_LINE: usize = 8 * 1024 * 1024;
/// After `session/cancel`, the agent has this long to end the prompt before its group is killed.
pub const CANCEL_GRACE: Duration = Duration::from_secs(5);
/// ACP v1 has no unsolicited-turn end signal. Silence closes only an agent turn
/// with no open tool or permission. Overrides are sampled once at actor load.
pub const AGENT_TURN_QUIET: Duration = Duration::from_secs(30);
fn agent_turn_quiet() -> Duration {
    std::env::var("BBOXY450_ACP_AGENT_TURN_QUIET_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(AGENT_TURN_QUIET)
}
// Test-only broken control: BBOXY450_ACP_IDLE_CONTROL=hide restores the old
// hidden idle updates and cancelled idle permissions. Never enable in profiles.
fn cancel_grace() -> Duration {
    std::env::var("BBOXY450_ACP_CANCEL_GRACE_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(CANCEL_GRACE)
}
const STDERR_TAIL: usize = 2048;
static LOGIN_PATH: tokio::sync::OnceCell<OsString> = tokio::sync::OnceCell::const_new();

/// Finder does not inherit the terminal's PATH. Probe once, off the UI thread,
/// with the same bounded, owned process-group capture used by profile resolvers.
pub(crate) async fn login_path() -> &'static OsStr {
    LOGIN_PATH
        .get_or_init(|| async {
            if let Some(shell) = std::env::var_os("SHELL") {
                let output = crate::process::capture_stdout(
                    crate::process::Request {
                        program: shell.to_string_lossy().into_owned(),
                        args: vec!["-ilc".into(), "printf %s \"$PATH\"".into()],
                        stdin: None,
                        cwd: std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| "/".into()),
                        timeout: Duration::from_secs(3),
                        spill_dir: PathBuf::new(),
                        cap: 64 * 1024,
                        redactor: Default::default(),
                    },
                    tokio_util::sync::CancellationToken::new(),
                )
                .await;
                if let Ok(output) = output
                    && output.code == Some(0)
                    && !output.cancelled
                    && output.bytes.len() < 64 * 1024
                    && let Ok(path) = String::from_utf8(output.bytes)
                    // A shell startup banner may precede printf's final line.
                    && let Some(path) = path.lines().last().filter(|p| !p.is_empty() && !p.contains('\0'))
                {
                    return OsString::from(path);
                }
            }
            let inherited = std::env::var_os("PATH")
                .unwrap_or_else(|| "/usr/bin:/bin:/usr/sbin:/sbin".into());
            let mut dirs: Vec<_> = std::env::split_paths(&inherited).collect();
            dirs.extend([PathBuf::from("/opt/homebrew/bin"), "/usr/local/bin".into()]);
            if let Some(home) = std::env::var_os("HOME") {
                dirs.extend([Path::new(&home).join(".local/bin"), Path::new(&home).join(".cargo/bin")]);
            }
            std::env::join_paths(dirs).unwrap_or(inherited)
        })
        .await
}

pub(crate) async fn find_agent(
    program: &str,
    path: &OsStr,
    cwd: &Path,
) -> std::io::Result<PathBuf> {
    for dir in std::env::split_paths(path) {
        let candidate = cwd.join(dir).join(program);
        if let Ok(meta) = tokio::fs::metadata(&candidate).await
            && meta.is_file()
            && meta.permissions().mode() & 0o111 != 0
        {
            return Ok(candidate);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("ACP agent executable {program:?} was not found in the login-shell PATH"),
    ))
}

enum Wire {
    Message(Value),
    Bad(String),
    Eof,
}
enum Call {
    Initialize,
    NewSession,
    SetPermissions {
        session_result: Value,
    },
    SetModel {
        revision: u64,
    },
    Prompt,
    Steer {
        op: OpId,
        blocks: Vec<InputBlock>,
        deadline: Instant,
    },
    Configure {
        op: OpId,
        revision: u64,
        id: String,
        value: String,
        legacy: bool,
    },
}
struct Agent {
    child: OwnedChild,
    /// whole lines, written in order by the writer task
    out: mpsc::UnboundedSender<String>,
    session: Option<String>,
    backend: Backend,
    model: Option<String>,
    pending_model: Option<String>,
    controls: AgentConfig,
    config_deadline: Option<Instant>,
    bootstrap_pending: bool,
    /// the agent takes launcher instructions as a session system prompt (ACP `systemPrompt`)
    system_prompt: bool,
    images_supported: bool,
    steering_supported: bool,
    startup_deadline: Option<Instant>,
    next_id: u64,
    calls: HashMap<u64, Call>,
    stderr: Arc<Mutex<String>>,
    /// set when the agent's stderr pipe closes: its last words are in `stderr`
    stderr_done: Arc<std::sync::atomic::AtomicBool>,
}
struct Tool {
    item: ItemId,
    call_id: String,
    name: String,
    kind: ToolKind,
    input: String,
    output: Vec<OutputBlock>,
    locations: Vec<Location>,
    raw_output: Option<Value>,
}
struct Permission {
    rpc: Value,
    tool: ItemId,
    options: Vec<(String, String)>,
    body: Item,
}
struct Running {
    turn: TurnId,
    blocks: Vec<InputBlock>,
    cause: Option<OpId>,
    prompt_sent: bool,
    cancel_at: Option<Instant>,
    quiet_at: Option<Instant>,
    last_wire_at: Instant,
    interrupted: bool,
    message: Option<(ItemId, String)>,
    thought: Option<(ItemId, String)>,
    tools: BTreeMap<String, Tool>,
    plan: Option<(ItemId, Vec<PlanEntry>)>,
    permissions: BTreeMap<ItemId, Permission>,
    /// Open ACP form elicitations: the request id to answer and the item.
    questions: BTreeMap<ItemId, (Value, Item)>,
}

pub struct AcpAgentActor {
    store: Store,
    gateway: InProc,
    work: mpsc::Receiver<Admitted>,
    control: mpsc::Receiver<Admitted>,
    config: Config,
    spec: AgentSpec,
    yolo: bool,
    /// The profile's model, chosen through the agent's `model` config option when offered.
    model: Option<String>,
    agent: Option<Agent>,
    wire: Option<mpsc::UnboundedReceiver<Wire>>,
    queue: VecDeque<(TurnId, Vec<InputBlock>, OpId)>,
    running: Option<Running>,
    resolved: HashSet<ItemId>,
    projected_ext: BTreeMap<String, Value>,
    /// Earlier turns of this log (a reopened or forked session), handed to the next
    /// fresh agent with its first prompt: the agent's own context starts empty.
    handoff: Option<String>,
    recall: Option<crate::dream::recall::Context>,
    trial_recall_suppressed: bool,
    dream_owner: crate::dream::c1::DreamOwner,
    cancel_grace: Duration,
    agent_turn_quiet: Duration,
    hide_idle: bool,
    launch_context: String,
    mcp_servers: Vec<Value>,
    services: TaskGroup,
    internal: mpsc::Receiver<Internal>,
    relay: crate::mcp::relay::Relay,
}

impl AcpAgentActor {
    pub(crate) async fn load(
        config: Config,
        start: OpId,
        profile: crate::provider::profile::Profile,
        overrides: crate::mcp::Servers,
        launch: LaunchOptions,
        notice: Option<String>,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        Self::load_inner(config, start, profile, overrides, launch, notice, None).await
    }
    /// Explicit caller-prepared reference; live loader discovery remains outside this adapter.
    pub async fn load_with_recall(
        config: Config,
        start: OpId,
        profile: crate::provider::profile::Profile,
        launch: LaunchOptions,
        recall: crate::dream::recall::Context,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        let recall = (recall.kind() == &SinkKind::AcpContext).then_some(recall);
        Self::load_inner(
            config,
            start,
            profile,
            Default::default(),
            launch,
            None,
            recall,
        )
        .await
    }
    async fn load_inner(
        mut config: Config,
        start: OpId,
        profile: crate::provider::profile::Profile,
        overrides: crate::mcp::Servers,
        launch: LaunchOptions,
        notice: Option<String>,
        recall: Option<crate::dream::recall::Context>,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        let spec = profile.agent.clone().unwrap_or_default();
        let yolo = launch.yolo || profile.policy.as_deref() == Some("yolo");
        if yolo {
            config.policy = crate::policy::Policy::allow_all(profile.on_unattended);
        }
        let model = Some(profile.model.clone()).filter(|m| !m.is_empty());
        crate::vcs::capture_new_session(&mut config.meta, &config.sessions).await;
        let mut store = Store::open(
            config.sessions.clone(),
            config.meta.clone(),
            Some(start),
            config.redactor.clone(),
        )
        .await?;
        if let Some(Envelope {
            event: Event::SessionStarted { meta },
            ..
        }) = store.events.first()
        {
            config.meta = meta.clone();
        }
        if store.fresh {
            store
                .append(
                    Event::ProfileChanged {
                        profile: config.meta.profile.clone().unwrap_or_default(),
                        model: match &model {
                            Some(m) => format!("{} · {m}", agent_label(&spec)),
                            None => agent_label(&spec),
                        },
                    },
                    Some(start),
                )
                .await?;
        }
        let hub = Arc::new(Hub::new(
            config.meta.clone(),
            store.epoch,
            &store.events,
            config.redactor.clone(),
        ));
        let (work_tx, work) = mpsc::channel(64);
        let (control_tx, control) = mpsc::channel(256);
        let gateway = InProc::new(hub, work_tx, control_tx, start);
        gateway.set_unattended(config.policy.on_unattended);
        // a reopened or forked session restarts its agent: hand it the conversation so far
        let handoff = handoff(&store.events);
        let (mut servers, collisions) =
            crate::mcp::select_sources(profile.mcp_servers, overrides, launch.mcp_servers);
        let (output, internal) = mpsc::channel(256);
        let mut services = TaskGroup::new(tokio_util::sync::CancellationToken::new());
        let endpoint = crate::python_execute::Endpoint::new().await?;
        crate::mcp::Manager::with_python(&mut servers, &endpoint.socket());
        let manager = crate::mcp::Manager::start_in(
            servers,
            config.redactor.clone(),
            output,
            &mut services,
            Some(config.meta.cwd.clone()),
        );
        crate::python_execute::Service::start(
            endpoint,
            config
                .sessions
                .join(config.meta.session.to_string())
                .join("python"),
            config.meta.cwd.clone(),
            manager.clone(),
            config.redactor.clone(),
            &mut services,
        )
        .await?;
        let relay = crate::mcp::relay::Relay::shared(manager, &mut services).await?;
        let mcp_servers = relay.servers.clone();
        let mut actor = Self {
            launch_context: config.redactor.scrub(&launch.append_system_prompt),
            mcp_servers,
            services,
            internal,
            relay,
            store,
            gateway: gateway.clone(),
            work,
            control,
            config,
            spec,
            yolo,
            model,
            agent: None,
            wire: None,
            queue: VecDeque::new(),
            running: None,
            resolved: HashSet::new(),
            projected_ext: BTreeMap::new(),
            handoff,
            recall,
            trial_recall_suppressed: launch.trial.is_some(),
            dream_owner: launch.dream_owner,
            cancel_grace: cancel_grace(),
            agent_turn_quiet: agent_turn_quiet(),
            hide_idle: std::env::var("BBOXY450_ACP_IDLE_CONTROL").as_deref() == Ok("hide"),
        };
        for text in notice
            .into_iter()
            .chain(launch.notices)
            .chain(actor.config.redactor.capability_notice())
        {
            actor.notice(Level::Warn, text).await?;
        }
        for text in collisions {
            actor.notice(Level::Info, text).await?;
        }
        let task = bbt_rt::handle().spawn(actor.run());
        Ok((gateway, task))
    }

    async fn publish(&mut self, event: Event, cause: Option<OpId>) -> std::io::Result<()> {
        let e = if event.is_durable() {
            let ext = if matches!(
                &event,
                Event::ItemStarted { .. } | Event::ItemCompleted { .. }
            ) {
                std::mem::take(&mut self.projected_ext)
            } else {
                Default::default()
            };
            crate::import::append_projected(
                &mut self.store,
                event,
                cause,
                crate::store::now_ms(),
                None,
                ext,
            )
            .await?
        } else {
            self.config
                .redactor
                .sanitize(&self.store.ephemeral(event))
                .map_err(std::io::Error::other)?
        };
        if matches!(e.event, Event::SessionClosed) {
            self.gateway.hub.publish(e.clone());
            crate::dream::session_closed(
                self.dream_owner,
                &self.config.sessions,
                &e,
                self.config.redactor.clone(),
                self.gateway.hub.clone(),
            );
        } else {
            self.gateway.hub.publish(e);
        }
        Ok(())
    }
    async fn reject(&mut self, op: OpId, reason: Rejection) -> std::io::Result<()> {
        self.publish(Event::OpRejected { op, reason }, Some(op))
            .await
    }
    async fn record(&mut self, turn: Option<TurnId>, body: Item) -> std::io::Result<()> {
        let item = ItemId::new();
        self.start_item(turn, item, &body, None).await?;
        self.complete(item, ItemStatus::Completed, body).await
    }
    async fn notice(&mut self, level: Level, text: String) -> std::io::Result<()> {
        let turn = self.running.as_ref().map(|r| r.turn);
        self.record(turn, Item::Notice { level, text }).await
    }
    async fn start_item(
        &mut self,
        turn: Option<TurnId>,
        item: ItemId,
        body: &Item,
        parent: Option<ItemId>,
    ) -> std::io::Result<()> {
        let cause = if matches!(body, Item::UserMessage { .. }) {
            self.running
                .as_ref()
                .filter(|r| Some(r.turn) == turn)
                .and_then(|r| r.cause)
        } else {
            None
        };
        self.publish(
            Event::ItemStarted {
                turn,
                item,
                kind: body.kind(),
                parent,
                head: body.head(),
            },
            cause,
        )
        .await
    }
    async fn complete(
        &mut self,
        item: ItemId,
        status: ItemStatus,
        body: Item,
    ) -> std::io::Result<()> {
        self.publish(Event::ItemCompleted { item, status, body }, None)
            .await
    }

    async fn run(mut self) -> std::io::Result<()> {
        if self.store.closed {
            self.gateway.mark_closed();
            return Ok(());
        }
        let mut tick = tokio::time::interval(Duration::from_millis(25));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut heartbeat = tokio::time::interval_at(
            Instant::now() + Duration::from_secs(15),
            Duration::from_secs(15),
        );
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let result = async {
            loop {
                if self.running.is_none()
                    && let Some((turn, blocks, cause)) = self.queue.pop_front()
                {
                    self.start(turn, blocks, cause).await?;
                }
                // A waiting user prompt shortens the quiet wait, but cannot
                // overtake a tool or a permission owned by the agent turn.
                if !self.queue.is_empty() && self.agent_turn_ready() {
                    self.end_agent_turn().await?;
                    continue;
                }
                let cancel_at = self.running.as_ref().and_then(|r| r.cancel_at);
                let quiet_at = self.running.as_ref().and_then(|r| r.quiet_at);
                tokio::select! {biased;
                    Some(op) = self.control.recv() => { if self.op(op).await? { break; } }
                    Some(op) = self.work.recv() => { if self.op(op).await? { break; } }
                    Some(Internal::ServiceNotice { level, text }) = self.internal.recv() => self.notice(level, text).await?,
                    Some(wire) = recv(&mut self.wire) => self.wire(wire).await?,
                    _ = sleep_until(cancel_at) => self.cancel_expired().await?,
                    _ = sleep_until(quiet_at) => self.quiet_expired().await?,
                    _ = tick.tick() => {
                        self.gateway.hub.flush_deltas();
                        self.expire_steering(Some(Instant::now())).await?;
                        if self.agent.as_ref().and_then(|a| a.config_deadline).is_some_and(|d| Instant::now() >= d) {
                            let a = self.agent.as_mut().unwrap();
                            a.config_deadline = None;
                            a.controls.status = "Unknown outcome · request will not be replayed".into();
                            self.publish_controls(None).await?;
                        }
                        if self.agent.as_ref().and_then(|a| a.startup_deadline).is_some_and(|d| Instant::now() >= d) {
                            self.agent_gone("ACP agent startup deadline exceeded".into()).await?;
                        }
                    },
                    _ = heartbeat.tick(), if self.running.is_none() =>
                        self.publish(Event::Status { phase: Phase::Idle }, None).await?,
                }
            }
            Ok::<_, std::io::Error>(())
        }
        .await;
        self.kill();
        let _ = self.services.cancel_and_join().await;
        self.relay.remove();
        if result.is_err() {
            self.gateway.mark_faulted();
        }
        self.gateway.mark_closed();
        result
    }

    // ---------- ops from faces ----------
    /// Returns true once the session has closed.
    async fn op(&mut self, admitted: Admitted) -> std::io::Result<bool> {
        let Admitted { env, face } = admitted;
        let id = env.op_id;
        match env.op {
            Op::QueryInputCapabilities { request } => {
                self.publish(
                    Event::InputCapabilities {
                        request,
                        image_custody_version: 1,
                    },
                    None,
                )
                .await?;
            }
            Op::UserTurn { blocks, .. } => {
                let blocks = match self.store.blobs.admit_images(blocks).await {
                    Ok(blocks) => blocks,
                    Err(error) => {
                        self.reject(
                            id,
                            Rejection::InvalidInput {
                                detail: error.to_string(),
                            },
                        )
                        .await?;
                        return Ok(false);
                    }
                };
                // Runtime capability negotiation covers Codex, Claude and other ACP adapters.
                if blocks.is_empty() || blocks.iter().any(|b| matches!(b, InputBlock::Unknown(_))) {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "provide supported input blocks".into(),
                        },
                    )
                    .await?;
                } else if self.queue.len()
                    + self.agent.as_ref().map_or(0, |a| {
                        a.calls
                            .values()
                            .filter(|c| matches!(c, Call::Steer { .. }))
                            .count()
                    })
                    >= 64
                {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "turn queue is full".into(),
                        },
                    )
                    .await?;
                } else if self.running.is_some()
                    && self.agent.as_ref().is_some_and(|a| {
                        a.steering_supported && a.session.is_some() && !a.bootstrap_pending
                    })
                {
                    self.steer(blocks, id).await?;
                } else {
                    self.queue_input(blocks, id).await?;
                }
            }
            Op::Interrupt { turn } => {
                let target = turn.or_else(|| self.running.as_ref().map(|r| r.turn));
                if let Some(t) = target
                    && self.running.as_ref().is_some_and(|r| r.turn == t)
                {
                    self.interrupt().await?;
                } else if let Some(pos) =
                    target.and_then(|t| self.queue.iter().position(|q| q.0 == t))
                {
                    let (turn, ..) = self.queue.remove(pos).expect("queued turn");
                    self.publish(
                        Event::TurnStarted {
                            turn,
                            model: self.model.clone(),
                            origin: Some(TurnOrigin::User),
                        },
                        Some(id),
                    )
                    .await?;
                    self.publish(
                        Event::TurnEnded {
                            turn,
                            reason: StopReason::Interrupted,
                            usage: None,
                        },
                        Some(id),
                    )
                    .await?;
                } else {
                    self.reject(id, Rejection::NoSuchTurn).await?;
                }
            }
            Op::ResolveApproval { approval, decision } => {
                let face = face.to_owned();
                self.resolve_by_face(id, approval, decision, face).await?;
            }
            Op::AnswerQuestion { question, answer } => {
                let open = self
                    .running
                    .as_ref()
                    .is_some_and(|r| r.questions.contains_key(&question));
                if !matches!(
                    answer.get("action").and_then(Value::as_str),
                    Some("accept" | "decline" | "cancel")
                ) {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "an answer's action is accept, decline or cancel".into(),
                        },
                    )
                    .await?;
                } else if open {
                    self.settle_question(question, answer, Some(id)).await?;
                } else if self.resolved.contains(&question) {
                    self.reject(id, Rejection::AlreadyResolved).await?;
                } else {
                    self.reject(id, Rejection::NoSuchApproval).await?;
                }
            }
            Op::SetProfile { .. } => {
                self.reject(
                    id,
                    Rejection::InvalidInput {
                        detail:
                            "an ACP agent session keeps its agent; start a new session to switch"
                                .into(),
                    },
                )
                .await?;
            }
            Op::SetAgentConfig {
                epoch,
                instance,
                revision,
                config_id,
                value,
            } => {
                self.configure(id, epoch, instance, revision, config_id, value)
                    .await?;
            }
            Op::Compact => self.reject(id, Rejection::NothingToCompact).await?,
            Op::Close => {
                self.close(id).await?;
                return Ok(true);
            }
            Op::StartSession { .. } | Op::Unknown(_) => {
                self.reject(
                    id,
                    Rejection::InvalidInput {
                        detail: "unsupported operation".into(),
                    },
                )
                .await?;
            }
        }
        Ok(false)
    }

    async fn session_ready(&mut self, result: Value) -> std::io::Result<()> {
        let Some(session) = result.get("sessionId").and_then(Value::as_str) else {
            return self
                .agent_gone("ACP agent returned no sessionId".into())
                .await;
        };
        let session = session.to_owned();
        self.agent.as_mut().expect("agent").session = Some(session.clone());
        self.agent.as_mut().expect("agent").model = model_option(&result)
            .and_then(|option| option.get("currentValue"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        {
            let a = self.agent.as_mut().unwrap();
            a.controls.replace(&result);
            a.controls.status = "Confirmed by agent".into();
            a.model = a
                .controls
                .options
                .iter()
                .find(|o| o.category == "model")
                .map(|o| o.current.clone())
                .or_else(|| a.model.clone());
        }
        self.publish_controls(None).await?;
        if let Some(wanted) = self.model.clone() {
            match pick_model(&result, &wanted) {
                ModelChoice::Current => {}
                ModelChoice::Set(value) => {
                    let agent = self.agent.as_mut().expect("agent");
                    agent.pending_model = Some(value.clone());
                    let option = agent
                        .controls
                        .options
                        .iter()
                        .find(|o| o.category == "model");
                    let legacy = option.is_some_and(|o| o.legacy);
                    // Preserve the existing profile bootstrap for older v1 advertisements
                    // missing optional UI metadata. Such rows remain unavailable as UI selectors.
                    let config_id = option
                        .map(|o| o.id.as_str())
                        .or_else(|| {
                            model_option(&result)
                                .and_then(|o| o.get("id"))
                                .and_then(Value::as_str)
                        })
                        .unwrap_or("model");
                    let params = if legacy {
                        json!({ "sessionId": session, "modelId": value })
                    } else {
                        json!({ "sessionId": session, "configId": config_id, "value": value })
                    };
                    let revision = agent.controls.revision;
                    agent.controls.pending = Some(OpId::new());
                    agent.controls.status = "Pending · applying profile model intent".into();
                    let sent = call(
                        agent,
                        Call::SetModel { revision },
                        if legacy {
                            "session/set_model"
                        } else {
                            "session/set_config_option"
                        },
                        params,
                    );
                    self.publish_controls(None).await?;
                    return sent;
                }
                ModelChoice::NotOffered(current) => {
                    self.notice(
                        Level::Warn,
                        format!(
                            "the agent does not offer model {wanted}; it keeps {}",
                            current.unwrap_or_else(|| "its default".into())
                        ),
                    )
                    .await?;
                }
            }
        }
        self.send_prompt().await
    }

    async fn publish_controls(&mut self, cause: Option<OpId>) -> std::io::Result<()> {
        let state = self
            .agent
            .as_ref()
            .map(|a| a.controls.clone())
            .unwrap_or_default();
        self.publish(
            Event::ContextChanged {
                mode: None,
                permission_mode: None,
                config: Some(BTreeMap::from([(
                    "acp_controls".into(),
                    serde_json::to_value(state).unwrap(),
                )])),
            },
            cause,
        )
        .await
    }
    async fn configure(
        &mut self,
        op: OpId,
        epoch: Epoch,
        instance: OpId,
        revision: u64,
        id: String,
        value: Option<String>,
    ) -> std::io::Result<()> {
        let value = value.filter(|value| {
            epoch == self.gateway.hello().epoch
                && self.agent.as_ref().is_some_and(|a| {
                    a.session.is_some() && a.controls.admits(instance, revision, &id, value)
                })
        });
        let Some(value) = value else {
            return self.reject(op, Rejection::InvalidInput { detail: "Agent configuration unavailable, pending or stale; refresh the controls".into() }).await;
        };
        let agent = self.agent.as_mut().unwrap();
        let legacy = agent
            .controls
            .options
            .iter()
            .find(|o| o.id == id)
            .unwrap()
            .legacy;
        let params = if legacy {
            json!({"sessionId": agent.session, "modelId": value})
        } else {
            json!({"sessionId": agent.session, "configId": id, "value": value})
        };
        agent.controls.pending = Some(op);
        agent.controls.status = "Pending · awaiting agent confirmation".into();
        agent.config_deadline = Some(Instant::now() + Duration::from_secs(5));
        let sent = call(
            agent,
            Call::Configure {
                op,
                revision,
                id,
                value,
                legacy,
            },
            if legacy {
                "session/set_model"
            } else {
                "session/set_config_option"
            },
            params,
        );
        if sent.is_err() {
            agent.controls.status = "Unknown outcome · request will not be replayed".into();
            agent.config_deadline = None;
        }
        self.publish_controls(Some(op)).await
    }
    async fn close(&mut self, id: OpId) -> std::io::Result<()> {
        self.gateway.seal();
        if self.running.is_some() {
            self.send_cancel();
            self.end_turn(StopReason::Cancelled, ItemStatus::Cancelled)
                .await?;
        }
        while let Some((turn, _, _)) = self.queue.pop_front() {
            self.publish(
                Event::TurnStarted {
                    turn,
                    model: self.model.clone(),
                    origin: Some(TurnOrigin::User),
                },
                Some(id),
            )
            .await?;
            self.publish(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::Cancelled,
                    usage: None,
                },
                Some(id),
            )
            .await?;
        }
        self.kill();
        self.publish(Event::SessionClosed, Some(id)).await
    }

    // ---------- turns ----------
    async fn start(
        &mut self,
        turn: TurnId,
        blocks: Vec<InputBlock>,
        cause: OpId,
    ) -> std::io::Result<()> {
        self.publish(
            Event::TurnStarted {
                turn,
                model: self.model.clone(),
                origin: Some(TurnOrigin::User),
            },
            None,
        )
        .await?;
        self.running = Some(Running {
            turn,
            blocks: blocks.clone(),
            cause: Some(cause),
            prompt_sent: false,
            cancel_at: None,
            quiet_at: None,
            last_wire_at: Instant::now(),
            interrupted: false,
            message: None,
            thought: None,
            tools: BTreeMap::new(),
            plan: None,
            permissions: BTreeMap::new(),
            questions: BTreeMap::new(),
        });
        let non_text: Vec<Value> = blocks
            .iter()
            .filter(|b| !matches!(b, InputBlock::Text { .. }))
            .map(|b| serde_json::to_value(b).expect("input serializes"))
            .collect();
        if !non_text.is_empty() {
            self.projected_ext
                .insert("acp".into(), json!({"input_blocks":non_text}));
        }
        if !blocks.iter().any(|b| matches!(b, InputBlock::Image { .. })) {
            self.record(Some(turn), Item::UserMessage { blocks })
                .await?;
        }
        self.publish(
            Event::Status {
                phase: Phase::Thinking,
            },
            None,
        )
        .await?;
        if self.agent.is_none() {
            if let Err(error) = self.spawn().await {
                let message = format!("ACP agent could not start: {error}");
                self.notice(Level::Error, message.clone()).await?;
                return self
                    .end_turn(
                        StopReason::Error {
                            class: ErrorClass::Config,
                            message,
                        },
                        ItemStatus::Failed {
                            code: None,
                            reason: "agent unavailable".into(),
                        },
                    )
                    .await;
            }
            if let Some(text) = &self.handoff {
                let turns =
                    text.matches("\n[user]").count() + usize::from(text.starts_with("[user]"));
                self.notice(
                    Level::Info,
                    format!("ACP agent started with a handoff of {turns} earlier turn(s) from this session"),
                )
                .await?;
            }
        } else {
            self.send_prompt().await?;
        }
        Ok(())
    }
    async fn start_agent_turn(&mut self) -> std::io::Result<()> {
        let turn = TurnId::new();
        self.publish(
            Event::TurnStarted {
                turn,
                model: self
                    .agent
                    .as_ref()
                    .and_then(|a| a.model.clone())
                    .or_else(|| self.model.clone()),
                origin: Some(TurnOrigin::Agent),
            },
            None,
        )
        .await?;
        let now = Instant::now();
        self.running = Some(Running {
            turn,
            blocks: vec![],
            cause: None,
            // An unsolicited turn must never cause session/prompt to be sent.
            prompt_sent: true,
            cancel_at: None,
            quiet_at: Some(now + self.agent_turn_quiet),
            last_wire_at: now,
            interrupted: false,
            message: None,
            thought: None,
            tools: BTreeMap::new(),
            plan: None,
            permissions: BTreeMap::new(),
            questions: BTreeMap::new(),
        });
        self.publish(
            Event::Status {
                phase: Phase::Thinking,
            },
            None,
        )
        .await
    }
    fn agent_turn_ready(&self) -> bool {
        self.running.as_ref().is_some_and(|r| {
            r.quiet_at.is_some()
                && r.cancel_at.is_none()
                && r.tools.is_empty()
                && r.permissions.is_empty()
                && r.questions.is_empty()
        })
    }
    async fn end_agent_turn(&mut self) -> std::io::Result<()> {
        let ms = self
            .running
            .as_ref()
            .expect("agent turn")
            .last_wire_at
            .elapsed()
            .as_millis()
            .min(u64::MAX as u128) as u64;
        self.end_turn(StopReason::Quiet { ms }, ItemStatus::Completed)
            .await
    }
    async fn quiet_expired(&mut self) -> std::io::Result<()> {
        if self.agent_turn_ready() {
            self.end_agent_turn().await
        } else {
            if let Some(r) = self.running.as_mut() {
                // Avoid a ready timer spinning while a tool/permission is open.
                r.quiet_at =
                    Some(Instant::now() + self.agent_turn_quiet.max(Duration::from_millis(1)));
            }
            Ok(())
        }
    }
    async fn queue_input(&mut self, blocks: Vec<InputBlock>, op: OpId) -> std::io::Result<()> {
        let turn = TurnId::new();
        self.publish(Event::TurnQueued { turn }, Some(op)).await?;
        self.queue.push_back((turn, blocks, op));
        Ok(())
    }
    // Protocol: https://github.com/agentclientprotocol/codex-acp/blob/main/docs/steering.md
    // Also implemented by claude-agent-acp; this is capability based, not vendor based.
    async fn steer(&mut self, blocks: Vec<InputBlock>, op: OpId) -> std::io::Result<()> {
        if blocks.iter().any(|b| matches!(b, InputBlock::Image { .. }))
            && !self.agent.as_ref().is_some_and(|a| a.images_supported)
        {
            return self
                .reject(
                    op,
                    Rejection::InvalidInput {
                        detail: "Agent does not support image input".into(),
                    },
                )
                .await;
        }
        let mut prompt = Vec::new();
        for block in &blocks {
            if let InputBlock::Image { blob } = block {
                use base64::Engine as _;
                prompt.push(json!({"type":"image", "data":base64::engine::general_purpose::STANDARD.encode(self.store.blobs.get(blob).await?), "mimeType":blob.mime}));
            } else {
                prompt.push(content_block(block));
            }
        }
        let agent = self.agent.as_mut().expect("negotiated agent");
        let params = json!({"sessionId":agent.session,"prompt":prompt,"_meta":{"steering":{"idleBehavior":"promptRequired"}}});
        let sent = call(
            agent,
            Call::Steer {
                op,
                blocks,
                deadline: Instant::now() + Duration::from_secs(10),
            },
            "_session/steering",
            params,
        );
        if sent.is_err() {
            self.agent_gone("ACP steering transport closed".into())
                .await?;
        }
        Ok(())
    }
    async fn steering_receipt(
        &mut self,
        op: OpId,
        blocks: Vec<InputBlock>,
        delivered: bool,
    ) -> std::io::Result<()> {
        let body = if delivered {
            Item::UserMessage {
                blocks: blocks.clone(),
            }
        } else {
            Item::Notice {
                level: Level::Warn,
                text: "Steering delivery unknown · inspect the agent before resending".into(),
            }
        };
        let mut meta = json!({"steering":{"outcome":if delivered {"injected"} else {"unknown"}}});
        if !delivered {
            meta["steering"]["input"] = serde_json::to_value(blocks).expect("input serializes");
        }
        let item = ItemId::new();
        // Unowned by a turn: its response may race the original turn's ending.
        self.projected_ext.insert("acp".into(), meta.clone());
        self.publish(
            Event::ItemStarted {
                turn: None,
                item,
                kind: body.kind(),
                parent: None,
                head: body.head(),
            },
            Some(op),
        )
        .await?;
        self.projected_ext.insert("acp".into(), meta);
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body,
            },
            Some(op),
        )
        .await
    }
    async fn expire_steering(&mut self, now: Option<Instant>) -> std::io::Result<()> {
        let expired: Vec<_> = self.agent.as_ref().map(|a| a.calls.iter().filter_map(|(id, call)|
            matches!(call, Call::Steer { deadline, .. } if now.is_none_or(|now| now >= *deadline)).then_some(*id)
        ).collect()).unwrap_or_default();
        for id in expired {
            if let Some(Call::Steer { op, blocks, .. }) =
                self.agent.as_mut().and_then(|a| a.calls.remove(&id))
            {
                self.steering_receipt(op, blocks, false).await?;
            }
        }
        Ok(())
    }
    async fn send_prompt(&mut self) -> std::io::Result<()> {
        if self
            .agent
            .as_ref()
            .is_some_and(|a| a.session.is_some() && !a.images_supported)
            && self.running.as_ref().is_some_and(|r| {
                !r.prompt_sent
                    && r.blocks
                        .iter()
                        .any(|b| matches!(b, InputBlock::Image { .. }))
            })
        {
            let message = "This ACP agent does not advertise image input; the draft is retained and no prompt was sent".to_owned();
            if let Some(cause) = self.running.as_ref().and_then(|r| r.cause) {
                self.reject(
                    cause,
                    Rejection::InvalidInput {
                        detail: message.clone(),
                    },
                )
                .await?;
            }
            self.notice(Level::Error, message.clone()).await?;
            return self
                .end_turn(
                    StopReason::Error {
                        class: ErrorClass::Config,
                        message,
                    },
                    ItemStatus::Failed {
                        code: None,
                        reason: "image input unsupported".into(),
                    },
                )
                .await;
        }
        if self.agent.as_ref().is_some_and(|a| a.session.is_some())
            && let Some(r) = self.running.as_ref().filter(|r| {
                !r.prompt_sent
                    && r.blocks
                        .iter()
                        .any(|b| matches!(b, InputBlock::Image { .. }))
            })
        {
            let (turn, blocks) = (r.turn, r.blocks.clone());
            self.record(Some(turn), Item::UserMessage { blocks })
                .await?;
        }
        let Some(r) = self.running.as_mut() else {
            return Ok(());
        };
        let Some(agent) = self.agent.as_mut() else {
            return Ok(());
        };
        let Some(session) = agent.session.clone() else {
            return Ok(());
        };
        if r.prompt_sent {
            return Ok(());
        }
        r.prompt_sent = true;
        let mut prompt: Vec<Value> = vec![];
        if agent.bootstrap_pending {
            agent.bootstrap_pending = false;
            agent.startup_deadline = None;
            if !self.launch_context.is_empty() && !agent.system_prompt {
                prompt.push(json!({"type":"text", "text":format!(
                    "Launcher context for this session (ACP v1 bootstrap context; supplied instructions, not a system-role transport):\n{}\n--- end launcher context ---", self.config.redactor.scrub(&self.launch_context)
                )}));
            }
        }
        if let Some(text) = self.handoff.take() {
            prompt.push(json!({ "type": "text", "text": format!(
                "Context: the conversation so far in this session, handed over from an earlier agent. \
                 Use it as background; the new request follows.\n\n{text}\n\n--- new request ---") }));
        }
        if let Some(recall) = self.recall.take() {
            let from = self.store.events.len();
            if let Ok(Some(block)) = recall
                .append(
                    &mut self.store,
                    self.trial_recall_suppressed,
                    &self.config.redactor,
                )
                .await
            {
                for event in &self.store.events[from..] {
                    self.gateway.hub.publish(event.clone());
                }
                prompt.push(json!({"type":"text","text":block}));
            }
        }
        for block in &r.blocks {
            if let InputBlock::Image { blob } = block {
                use base64::Engine as _;
                let bytes = self.store.blobs.get(blob).await?;
                prompt.push(json!({"type":"image","data":base64::engine::general_purpose::STANDARD.encode(bytes),"mimeType":blob.mime}));
            } else {
                prompt.push(content_block(block));
            }
        }
        let params = json!({ "sessionId": session, "prompt": prompt });
        debug_assert!(
            serde_json::from_value::<agent_client_protocol::schema::v1::PromptRequest>(
                params.clone()
            )
            .is_ok()
        );
        call(agent, Call::Prompt, "session/prompt", params)
    }
    async fn interrupt(&mut self) -> std::io::Result<()> {
        let Some(r) = self.running.as_mut() else {
            return Ok(());
        };
        if r.cancel_at.is_some() {
            return Ok(());
        }
        r.interrupted = true;
        r.cancel_at = Some(Instant::now() + self.cancel_grace);
        let sent = r.prompt_sent;
        let pending: Vec<ItemId> = r.permissions.keys().copied().collect();
        let asked: Vec<ItemId> = r.questions.keys().copied().collect();
        if !sent {
            // nothing reached the agent yet: the turn ends here
            return self
                .end_turn(StopReason::Interrupted, ItemStatus::Interrupted)
                .await;
        }
        self.send_cancel();
        // the protocol: every pending permission is answered `cancelled`
        for item in pending {
            self.settle_permission(item, None, Resolver::Cancelled, None)
                .await?;
        }
        for item in asked {
            self.settle_question(item, json!({"action": "cancel"}), None)
                .await?;
        }
        Ok(())
    }
    fn send_cancel(&mut self) {
        if let Some(agent) = self.agent.as_mut()
            && let Some(session) = agent.session.clone()
        {
            let _ = notify(agent, "session/cancel", json!({ "sessionId": session }));
        }
    }
    async fn cancel_expired(&mut self) -> std::io::Result<()> {
        // the agent did not end the prompt in time: its process group goes
        self.kill();
        self.notice(
            Level::Warn,
            format!(
                "ACP agent did not stop within {}ms of cancel; its process group was killed",
                self.cancel_grace.as_millis()
            ),
        )
        .await?;
        self.end_turn(StopReason::Interrupted, ItemStatus::Interrupted)
            .await
    }
    /// Close every open item of the running turn (children before parents), then end it.
    async fn end_turn(&mut self, reason: StopReason, open: ItemStatus) -> std::io::Result<()> {
        let Some(mut r) = self.running.take() else {
            return Ok(());
        };
        for (item, p) in std::mem::take(&mut r.permissions) {
            self.resolved.insert(item);
            let body = decided(p.body, None, Resolver::Cancelled);
            self.complete(item, ItemStatus::Cancelled, body).await?;
        }
        for (item, (rpc, body)) in std::mem::take(&mut r.questions) {
            self.resolved.insert(item);
            let cancel = json!({"action": "cancel"});
            if let Some(agent) = self.agent.as_mut() {
                let _ = reply(
                    agent,
                    json!({"jsonrpc": "2.0", "id": rpc, "result": cancel}),
                );
            }
            self.complete(item, ItemStatus::Cancelled, answered(body, cancel))
                .await?;
        }
        self.running = Some(r);
        self.flush_text().await?;
        let mut r = self.running.take().expect("running turn");
        for (_, t) in std::mem::take(&mut r.tools) {
            let status = if matches!(open, ItemStatus::Completed) {
                ItemStatus::Interrupted
            } else {
                open.clone()
            };
            self.complete(
                t.item,
                status,
                tool_body(t, "(the agent did not report a result)"),
            )
            .await?;
        }
        if let Some((item, entries)) = r.plan.take() {
            self.complete(item, ItemStatus::Completed, Item::Plan { entries })
                .await?;
        }
        self.publish(
            Event::TurnEnded {
                turn: r.turn,
                reason,
                usage: None,
            },
            r.cause,
        )
        .await?;
        self.publish(Event::Status { phase: Phase::Idle }, None)
            .await
    }
    async fn flush_text(&mut self) -> std::io::Result<()> {
        let Some(r) = self.running.as_mut() else {
            return Ok(());
        };
        let message = r.message.take();
        let thought = r.thought.take();
        if let Some((item, text)) = thought {
            self.complete(
                item,
                ItemStatus::Completed,
                Item::Reasoning {
                    summary: text,
                    raw: None,
                    provider_opaque: None,
                },
            )
            .await?;
        }
        if let Some((item, text)) = message {
            self.complete(
                item,
                ItemStatus::Completed,
                Item::AgentMessage { markdown: text },
            )
            .await?;
        }
        Ok(())
    }

    // ---------- the agent process ----------
    async fn spawn(&mut self) -> std::io::Result<()> {
        let mut env = vec![];
        for (name, value) in &self.spec.env {
            let value = match value.strip_prefix('!') {
                Some(cmd) => crate::provider::profile::resolve_command(
                    cmd,
                    tokio_util::sync::CancellationToken::new(),
                )
                .await
                .map_err(|e| std::io::Error::other(format!("agent.env {name}: {}", e.message)))?,
                None => value.clone(),
            };
            let value = value.trim().to_owned();
            self.config.redactor.register(&value);
            env.push((name.clone(), value));
        }
        // Launch controls are public configuration, not secret profile env.
        // Apply them after resolving/redacting the caller's actual env values.
        let mut spec = AgentSpec {
            command: self.spec.command.clone(),
            env: BTreeMap::new(),
        };
        if self.yolo {
            acp_permissions::prepare(&mut spec);
        }
        let mut argv = spec.command.iter();
        let program = argv
            .next()
            .ok_or_else(|| std::io::Error::other("agent.command is empty"))?;
        let path = if program.contains('/') {
            None
        } else {
            Some(login_path().await)
        };
        let resolved = match path {
            Some(path) => find_agent(program, path, &self.config.meta.cwd).await?,
            None => PathBuf::from(program),
        };
        let mut command = tokio::process::Command::new(resolved);
        command
            .args(argv)
            .envs(env)
            .envs(spec.env)
            .current_dir(&self.config.meta.cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if let Some(path) = path {
            command.env("PATH", path);
        }
        let mut child = OwnedChild::spawn(command)?;
        let stdin = child
            .stdin()
            .ok_or_else(|| std::io::Error::other("agent stdin unavailable"))?;
        let (out, lines) = mpsc::unbounded_channel();
        bbt_rt::handle().spawn(write_lines(stdin, lines));
        let stdout = child
            .stdout()
            .ok_or_else(|| std::io::Error::other("agent stdout unavailable"))?;
        let stderr_pipe = child.stderr();
        let (tx, rx) = mpsc::unbounded_channel();
        bbt_rt::handle().spawn(read_lines(stdout, tx));
        let stderr = Arc::new(Mutex::new(String::new()));
        let stderr_done = Arc::new(std::sync::atomic::AtomicBool::new(stderr_pipe.is_none()));
        if let Some(pipe) = stderr_pipe {
            bbt_rt::handle().spawn(read_tail(pipe, stderr.clone(), stderr_done.clone()));
        }
        let mut agent = Agent {
            child,
            out,
            session: None,
            backend: Backend {
                kind: BackendKind::AcpAgent,
                name: self.spec.command.first().cloned().unwrap_or_default(),
                version: None,
            },
            model: None,
            pending_model: None,
            controls: AgentConfig::default(),
            config_deadline: None,
            bootstrap_pending: true,
            system_prompt: false,
            images_supported: false,
            steering_supported: false,
            startup_deadline: Some(
                Instant::now()
                    + std::env::var("BBOXY450_ACP_STARTUP_MS")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .map(Duration::from_millis)
                        .unwrap_or(Duration::from_secs(30)),
            ),
            next_id: 0,
            calls: HashMap::new(),
            stderr,
            stderr_done,
        };
        let params = json!({
            "protocolVersion": 1,
            // phase 1 serves no files or terminals; the agent uses its own tools
            "clientCapabilities": { "fs": { "readTextFile": false, "writeTextFile": false }, "terminal": false, "elicitation": { "form": {} } },
            "clientInfo": { "name": "bboxy450-harness", "version": env!("CARGO_PKG_VERSION") }
        });
        call(&mut agent, Call::Initialize, "initialize", params)?;
        self.agent = Some(agent);
        self.wire = Some(rx);
        Ok(())
    }
    fn kill(&mut self) {
        if let Some(agent) = self.agent.take() {
            agent.child.send(libc::SIGKILL);
        }
        self.wire = None;
    }
    async fn agent_gone(&mut self, why: String) -> std::io::Result<()> {
        self.expire_steering(None).await?;
        // stdout can close before stderr is drained: give the agent's last words a moment
        if let Some(done) = self.agent.as_ref().map(|a| a.stderr_done.clone()) {
            let until = Instant::now() + Duration::from_millis(500);
            while !done.load(std::sync::atomic::Ordering::Acquire) && Instant::now() < until {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        let tail = self
            .agent
            .as_ref()
            .map(|a| a.stderr.lock().map(|s| s.clone()).unwrap_or_default())
            .unwrap_or_default();
        self.kill();
        self.publish_controls(None).await?;
        let tail = self.config.redactor.scrub(tail.trim());
        let message = if tail.is_empty() {
            why
        } else {
            format!("{why}; stderr: {tail}")
        };
        if self.running.is_some() {
            self.notice(Level::Error, message.clone()).await?;
            self.end_turn(
                StopReason::Error {
                    class: ErrorClass::MidStream,
                    message,
                },
                ItemStatus::Failed {
                    code: None,
                    reason: "agent exited".into(),
                },
            )
            .await
        } else {
            self.notice(Level::Warn, message).await
        }
    }

    // ---------- the wire ----------
    async fn wire(&mut self, wire: Wire) -> std::io::Result<()> {
        match wire {
            Wire::Eof => self.agent_gone("ACP agent exited".into()).await,
            Wire::Bad(why) => self.agent_gone(format!("ACP agent sent {why}")).await,
            Wire::Message(m) => {
                if let Some(r) = self.running.as_mut()
                    && r.quiet_at.is_some()
                {
                    r.last_wire_at = Instant::now();
                    r.quiet_at = Some(r.last_wire_at + self.agent_turn_quiet);
                }
                let method = m.get("method").and_then(Value::as_str).map(str::to_owned);
                match (method, m.get("id").cloned()) {
                    (Some(method), Some(id)) => {
                        self.request(&method, id, m.get("params").cloned().unwrap_or(Value::Null))
                            .await
                    }
                    (Some(method), None) => {
                        if method == "session/update" {
                            if m.pointer("/params/sessionId").and_then(Value::as_str)
                                != self.agent.as_ref().and_then(|a| a.session.as_deref())
                            {
                                return Ok(());
                            }
                            let update =
                                m.pointer("/params/update").cloned().unwrap_or(Value::Null);
                            self.update(update).await
                        } else {
                            self.native_record_type(&method, &m).await
                        }
                    }
                    (None, Some(id)) => self.response(id, m).await,
                    (None, None) => {
                        self.agent_gone(
                            "ACP agent sent a message that is neither request nor response".into(),
                        )
                        .await
                    }
                }
            }
        }
    }
    async fn response(&mut self, id: Value, m: Value) -> std::io::Result<()> {
        let Some(agent) = self.agent.as_mut() else {
            return Ok(());
        };
        let Some(call_kind) = id.as_u64().and_then(|n| agent.calls.remove(&n)) else {
            return Ok(());
        };
        if let Call::Steer { op, blocks, .. } = call_kind {
            return match m.pointer("/result/outcome").and_then(Value::as_str) {
                Some("injected" | "startedNewTurn") => {
                    self.steering_receipt(op, blocks, true).await
                }
                Some("promptRequired")
                    if m.pointer("/result/reason").and_then(Value::as_str)
                        == Some("noRunningTurn") =>
                {
                    self.queue_input(blocks, op).await
                }
                _ if m.pointer("/error/code").and_then(Value::as_i64) == Some(-32601) => {
                    self.agent.as_mut().unwrap().steering_supported = false;
                    self.queue_input(blocks, op).await
                }
                _ => self.steering_receipt(op, blocks, false).await,
            };
        }
        if let Call::Configure {
            op,
            revision,
            id,
            value,
            legacy,
        } = call_kind
        {
            let a = self.agent.as_mut().unwrap();
            if a.controls.pending != Some(op) {
                return Ok(());
            }
            a.config_deadline = None;
            a.controls.pending = None;
            if m.get("error").is_some() {
                a.controls.status = "Refused by agent · confirmed values retained".into();
            } else if a.controls.revision != revision {
                // A newer notification is authoritative; an older response cannot roll it back.
                a.controls.status = "Agent updated configuration · older response ignored".into();
            } else if legacy && m.get("result").is_some_and(Value::is_object) {
                if let Some(o) = a
                    .controls
                    .options
                    .iter_mut()
                    .find(|o| o.id == id && o.legacy)
                {
                    o.current = value;
                }
                // Legacy model acknowledgements have no effort catalogue: retire the old choices.
                a.controls.options.retain(|o| o.category == "model");
                a.controls.revision += 1;
                a.controls.status = "Confirmed by agent · legacy model capability".into();
            } else if m
                .pointer("/result/configOptions")
                .is_some_and(Value::is_array)
            {
                a.controls.replace(&m["result"]);
                a.controls.status = "Confirmed by agent".into();
            } else {
                a.controls.status =
                    "Unknown outcome · agent omitted configuration; request will not be replayed"
                        .into();
                a.controls.options.clear();
                a.controls.revision += 1;
            }
            a.model = a
                .controls
                .options
                .iter()
                .find(|o| o.category == "model")
                .map(|o| o.current.clone());
            return self.publish_controls(Some(op)).await;
        }
        if let Some(error) = m.get("error") {
            let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
            let text = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_owned();
            let hint = if code == -32000 || text.to_lowercase().contains("auth") {
                " · authenticate the agent in a terminal (API key, console login, Bedrock or Vertex)"
            } else {
                ""
            };
            let message = format!("ACP agent refused {}: {text}{hint}", call_name(&call_kind));
            return match call_kind {
                Call::SetModel { .. } if self.running.is_some() => {
                    let a = self.agent.as_mut().expect("agent");
                    a.pending_model = None;
                    a.controls.pending = None;
                    a.controls.status = "Refused by agent · confirmed values retained".into();
                    self.publish_controls(None).await?;
                    // the turn still runs, on the agent's own model
                    self.notice(Level::Warn, message).await?;
                    self.send_prompt().await
                }
                Call::Prompt
                | Call::Initialize
                | Call::NewSession
                | Call::SetPermissions { .. }
                    if self.running.is_some() =>
                {
                    self.notice(Level::Error, message.clone()).await?;
                    if !matches!(call_kind, Call::Prompt) {
                        self.kill();
                    }
                    self.end_turn(
                        StopReason::Error {
                            class: if hint.is_empty() {
                                ErrorClass::BadRequest
                            } else {
                                ErrorClass::Auth
                            },
                            message,
                        },
                        ItemStatus::Failed {
                            code: None,
                            reason: "agent error".into(),
                        },
                    )
                    .await
                }
                _ => self.notice(Level::Warn, message).await,
            };
        }
        let result = m.get("result").cloned().unwrap_or(Value::Null);
        match call_kind {
            Call::Initialize => {
                let version = result.get("protocolVersion").and_then(Value::as_u64);
                if version != Some(1) {
                    return self
                        .agent_gone(format!("ACP agent speaks protocol {version:?}, not 1"))
                        .await;
                }
                let cwd = self.config.meta.cwd.display().to_string();
                let agent = self.agent.as_mut().expect("agent");
                agent.steering_supported = result
                    .pointer("/_meta/steering/supported")
                    .and_then(Value::as_bool)
                    == Some(true);
                agent.images_supported = result
                    .pointer("/agentCapabilities/promptCapabilities/image")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if let Some(name) = result
                    .pointer("/agentInfo/name")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                {
                    agent.backend.name = name.to_owned();
                }
                agent.backend.version = result
                    .pointer("/agentInfo/version")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                // Append mode is the baseline every advertising agent accepts
                // (claude-agent-acp: preset + append; codex-acp: developer instructions).
                agent.system_prompt = result
                    .pointer("/agentCapabilities/sessionCapabilities/systemPrompt")
                    .is_some_and(Value::is_object);
                let mut params = json!({ "cwd": cwd, "mcpServers": self.mcp_servers });
                if self.yolo {
                    acp_permissions::session_options(&mut params, &agent.backend.name);
                }
                let fallback = !self.launch_context.is_empty() && !agent.system_prompt;
                if agent.system_prompt && !self.launch_context.is_empty() {
                    params["systemPrompt"] = json!({"mode": "append", "content": [
                        {"type": "text", "text": self.config.redactor.scrub(&self.launch_context)}
                    ]});
                }
                call(agent, Call::NewSession, "session/new", params)?;
                if fallback {
                    self.notice(Level::Info, "Launcher context rides with the first prompt (no system prompt on this agent).".into()).await?;
                }
                Ok(())
            }
            Call::NewSession => {
                if self.yolo
                    && let Some((config_id, mode)) = acp_permissions::selection(&result)
                {
                    let Some(session) = result
                        .get("sessionId")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                    else {
                        return self
                            .agent_gone("ACP agent returned no sessionId".into())
                            .await;
                    };
                    let mut result = result;
                    if let Some(modes) = result.get_mut("modes").and_then(Value::as_object_mut) {
                        modes.insert("currentModeId".into(), json!(mode));
                    }
                    if let Some(options) = result
                        .get_mut("configOptions")
                        .and_then(Value::as_array_mut)
                    {
                        for option in options {
                            if config_id.as_deref().is_some_and(|id| {
                                option.get("id").and_then(Value::as_str) == Some(id)
                            }) {
                                option["currentValue"] = json!(mode);
                            }
                        }
                    }
                    let agent = self.agent.as_mut().expect("agent");
                    agent.session = Some(session.clone());
                    let (method, params) = match config_id {
                        Some(id) => (
                            "session/set_config_option",
                            json!({"sessionId":session,"configId":id,"value":mode}),
                        ),
                        None => (
                            "session/set_mode",
                            json!({"sessionId":session,"modeId":mode}),
                        ),
                    };
                    return call(
                        agent,
                        Call::SetPermissions {
                            session_result: result,
                        },
                        method,
                        params,
                    );
                }
                self.session_ready(result).await
            }
            Call::SetPermissions { session_result } => self.session_ready(session_result).await,
            Call::SetModel { revision } => {
                let agent = self.agent.as_mut().expect("agent");
                let pending = agent.pending_model.take();
                agent.controls.pending = None;
                agent.controls.status = "Confirmed by agent".into();
                if agent.controls.revision != revision {
                    agent.controls.status =
                        "Agent updated configuration · older response ignored".into();
                } else if result.get("configOptions").is_some_and(Value::is_array) {
                    agent.controls.replace(&result);
                    agent.model = agent
                        .controls
                        .options
                        .iter()
                        .find(|o| o.category == "model")
                        .map(|o| o.current.clone());
                } else {
                    // Old adapters acknowledged with {}; they confirmed only the selected model.
                    agent.model = pending;
                    agent.controls.options.retain(|o| o.category == "model");
                    for o in &mut agent.controls.options {
                        if o.category == "model"
                            && let Some(m) = &agent.model
                        {
                            o.current = m.clone();
                        }
                    }
                    agent.controls.revision += 1;
                }
                self.publish_controls(None).await?;
                self.send_prompt().await
            }
            Call::Configure { .. } | Call::Steer { .. } => unreachable!(),
            Call::Prompt => {
                if result
                    .as_object()
                    .is_some_and(|o| o.keys().any(|k| k != "stopReason"))
                    || result
                        .get("stopReason")
                        .and_then(Value::as_str)
                        .is_some_and(|s| {
                            !matches!(
                                s,
                                "end_turn"
                                    | "max_tokens"
                                    | "max_turn_requests"
                                    | "cancelled"
                                    | "refusal"
                            )
                        })
                {
                    self.native_record_type("prompt_result", &result).await?;
                }
                let stop = result
                    .get("stopReason")
                    .and_then(Value::as_str)
                    .unwrap_or("end_turn");
                let interrupted = self.running.as_ref().is_some_and(|r| r.interrupted);
                let reason = match stop {
                    "end_turn" => StopReason::EndTurn,
                    "max_tokens" => StopReason::MaxTokens,
                    "max_turn_requests" => StopReason::MaxRounds,
                    "cancelled" if interrupted => StopReason::Interrupted,
                    "cancelled" => StopReason::Cancelled,
                    "refusal" => StopReason::Error {
                        class: ErrorClass::BadRequest,
                        message: "the agent refused this prompt".into(),
                    },
                    other => StopReason::Error {
                        class: ErrorClass::Malformed,
                        message: format!("unknown stop reason {other}"),
                    },
                };
                let open = if interrupted {
                    ItemStatus::Interrupted
                } else {
                    ItemStatus::Completed
                };
                self.end_turn(reason, open).await
            }
        }
    }
    async fn request(&mut self, method: &str, id: Value, params: Value) -> std::io::Result<()> {
        // Only form elicitations are advertised; a URL one falls to "not supported".
        if method == "elicitation/create"
            && params.get("mode").and_then(Value::as_str).unwrap_or("form") == "form"
        {
            return self.question(id, params).await;
        }
        if method != "session/request_permission" {
            // phase 1 advertises no fs, terminal or elicitation: answer, never wedge the turn
            if let Some(agent) = self.agent.as_mut() {
                let _ = reply(
                    agent,
                    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("{method} is not supported by this client") } }),
                );
            }
            return Ok(());
        }
        if self.running.is_none() && !self.hide_idle {
            self.start_agent_turn().await?;
        }
        if self.running.as_ref().is_none_or(|r| r.cancel_at.is_some()) {
            if let Some(agent) = self.agent.as_mut() {
                let _ = reply(agent, permission_reply(&id, None));
            }
            return Ok(());
        }
        let tool_call = params.get("toolCall").cloned().unwrap_or(Value::Null);
        let tool = self.tool_for(&tool_call).await?;
        let options: Vec<(String, String)> = params
            .get("options")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|o| {
                Some((
                    o.get("kind")?.as_str()?.to_owned(),
                    o.get("optionId")?.as_str()?.to_owned(),
                ))
            })
            .collect();
        let mut kinds = vec![];
        for (kind, _) in &options {
            let k = match kind.as_str() {
                "allow_once" => ApprovalDecisionKind::Allow,
                "allow_always" => ApprovalDecisionKind::AllowAlways,
                _ => ApprovalDecisionKind::Deny,
            };
            if !kinds.contains(&k) {
                kinds.push(k);
            }
        }
        let r = self.running.as_ref().expect("running");
        let t = r.tools.values().find(|t| t.item == tool).expect("tool");
        let title = tool_call
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(&t.name)
            .to_owned();
        let body = Item::Approval {
            subject: ApprovalSubject {
                tool_call: tool,
                effect: match t.kind {
                    ToolKind::Edit | ToolKind::Delete | ToolKind::Move => Effect::Write,
                    ToolKind::Execute => Effect::Exec,
                    ToolKind::Fetch => Effect::Network,
                    ToolKind::Read | ToolKind::Search | ToolKind::Think => Effect::Read,
                    _ => Effect::Exec,
                },
                summary: title,
                paths: t.locations.iter().map(|l| l.path.clone()).collect(),
                command: tool_call
                    .pointer("/rawInput/command")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            },
            options: kinds,
            decision: None,
            by: None,
        };
        let item = ItemId::new();
        let turn = r.turn;
        self.start_item(Some(turn), item, &body, Some(tool)).await?;
        self.publish(
            Event::ItemSnapshot {
                item,
                partial: body.clone(),
            },
            None,
        )
        .await?;
        self.running.as_mut().expect("running").permissions.insert(
            item,
            Permission {
                rpc: id,
                tool,
                options,
                body,
            },
        );
        self.publish(
            Event::Status {
                phase: Phase::Approval { item },
            },
            None,
        )
        .await?;
        // Full-access profiles do not wait for a face to approve a tool.
        if self.yolo {
            return self
                .settle_permission(
                    item,
                    Some(ApprovalDecision::Allow),
                    Resolver::Rule {
                        id: "profile-yolo".into(),
                    },
                    None,
                )
                .await;
        }
        if self.gateway.hub.approvers() == 0 {
            let decision = match self.config.policy.on_unattended {
                Unattended::Allow => ApprovalDecision::Allow,
                Unattended::Deny => ApprovalDecision::Deny {
                    reason: Some("no approver attached; on_unattended=deny".into()),
                },
            };
            return self
                .settle_permission(item, Some(decision), Resolver::Unattended, None)
                .await;
        }
        Ok(())
    }
    async fn resolve_by_face(
        &mut self,
        op: OpId,
        approval: ItemId,
        decision: ApprovalDecision,
        face: String,
    ) -> std::io::Result<()> {
        if self
            .running
            .as_ref()
            .is_some_and(|r| r.permissions.contains_key(&approval))
        {
            self.settle_permission(
                approval,
                Some(decision),
                Resolver::Face { name: face },
                Some(op),
            )
            .await
        } else {
            let reason = if self.resolved.contains(&approval) {
                Rejection::AlreadyResolved
            } else {
                Rejection::NoSuchApproval
            };
            self.reject(op, reason).await
        }
    }
    /// An ACP form elicitation becomes a Question the human answers in a face. The
    /// session waits on it like an approval (Phase::Approval); full access never
    /// answers one, because only the human can.
    async fn question(&mut self, id: Value, params: Value) -> std::io::Result<()> {
        if self.running.is_none() && !self.hide_idle {
            self.start_agent_turn().await?;
        }
        if self.running.as_ref().is_none_or(|r| r.cancel_at.is_some()) {
            if let Some(agent) = self.agent.as_mut() {
                let _ = reply(
                    agent,
                    json!({"jsonrpc": "2.0", "id": id, "result": {"action": "cancel"}}),
                );
            }
            return Ok(());
        }
        let body = Item::Question {
            message: params
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            schema: params
                .get("requestedSchema")
                .cloned()
                .unwrap_or(Value::Null),
            answer: None,
        };
        let item = ItemId::new();
        let turn = self.running.as_ref().expect("running").turn;
        self.start_item(Some(turn), item, &body, None).await?;
        self.publish(
            Event::ItemSnapshot {
                item,
                partial: body.clone(),
            },
            None,
        )
        .await?;
        self.running
            .as_mut()
            .expect("running")
            .questions
            .insert(item, (id, body));
        self.publish(
            Event::Status {
                phase: Phase::Approval { item },
            },
            None,
        )
        .await
    }
    /// Send the human's answer to the agent and complete the question.
    async fn settle_question(
        &mut self,
        item: ItemId,
        answer: Value,
        cause: Option<OpId>,
    ) -> std::io::Result<()> {
        let Some((rpc, body)) = self
            .running
            .as_mut()
            .and_then(|r| r.questions.remove(&item))
        else {
            return Ok(());
        };
        if let Some(agent) = self.agent.as_mut() {
            let _ = reply(
                agent,
                json!({"jsonrpc": "2.0", "id": rpc, "result": answer}),
            );
        }
        self.resolved.insert(item);
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: answered(body, answer),
            },
            cause,
        )
        .await
    }
    /// Answer one permission request: a decision picks the matching option; `None` is `cancelled`.
    async fn settle_permission(
        &mut self,
        item: ItemId,
        decision: Option<ApprovalDecision>,
        by: Resolver,
        cause: Option<OpId>,
    ) -> std::io::Result<()> {
        let Some(p) = self
            .running
            .as_mut()
            .and_then(|r| r.permissions.remove(&item))
        else {
            return Ok(());
        };
        let wanted: &[&str] = match &decision {
            Some(ApprovalDecision::Allow) if self.yolo => &["allow_once", "allow_always"],
            Some(ApprovalDecision::Allow) => &["allow_once"],
            Some(ApprovalDecision::AllowAlways) => &["allow_always"],
            Some(ApprovalDecision::Deny { .. }) => &["reject_once", "reject_always"],
            _ => &[],
        };
        let option = wanted
            .iter()
            .find_map(|k| p.options.iter().find(|(kind, _)| kind == k).cloned());
        if let Some(agent) = self.agent.as_mut() {
            let _ = reply(
                agent,
                permission_reply(&p.rpc, option.as_ref().map(|(_, id)| id.as_str())),
            );
        }
        let decision = if option
            .as_ref()
            .is_some_and(|(kind, _)| kind == "allow_always")
        {
            Some(ApprovalDecision::AllowAlways)
        } else if option.is_some() {
            decision
        } else {
            Some(ApprovalDecision::Deny {
                reason: Some("cancelled".into()),
            })
        };
        self.resolved.insert(item);
        let _ = p.tool;
        let body = decided(p.body, decision, by);
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body,
            },
            cause,
        )
        .await
    }

    // ---------- session/update ----------
    async fn update(&mut self, u: Value) -> std::io::Result<()> {
        if u.get("sessionUpdate").and_then(Value::as_str) == Some("config_option_update") {
            if let Some(a) = self.agent.as_mut() {
                a.controls.replace(&u);
                a.model = a
                    .controls
                    .options
                    .iter()
                    .find(|o| o.category == "model")
                    .map(|o| o.current.clone());
            }
            return self.publish_controls(None).await;
        }
        let extra = update_remainder(&u);
        if extra.as_object().is_some_and(|o| !o.is_empty()) {
            self.projected_ext.insert("acp".into(), extra);
        }
        let result = self.update_inner(u.clone()).await;
        if result.is_ok() && !self.projected_ext.is_empty() {
            self.native_update(&u).await?;
        }
        self.projected_ext.clear();
        result
    }
    async fn native_update(&mut self, u: &Value) -> std::io::Result<()> {
        self.native_record_type(
            u.get("sessionUpdate")
                .and_then(Value::as_str)
                .unwrap_or("notification"),
            u,
        )
        .await
    }
    async fn native_record_type(&mut self, kind: &str, u: &Value) -> std::io::Result<()> {
        let provider = self
            .agent
            .as_ref()
            .map(|a| a.backend.name.clone())
            .unwrap_or_else(|| agent_label(&self.spec));
        let provider = if provider.starts_with("acp:") {
            provider
        } else {
            format!("acp:{provider}")
        };
        let body = Item::Native {
            provider,
            record_type: kind.into(),
            raw: u.clone(),
            blob: None,
        };
        self.record(self.running.as_ref().map(|r| r.turn), body)
            .await
    }
    async fn update_inner(&mut self, u: Value) -> std::io::Result<()> {
        let kind = u
            .get("sessionUpdate")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        // Usage is a session gauge and can arrive after the prompt response.
        if kind == "usage_update" {
            // A costed usage closes the agent's query segment. Claude ACP drops the
            // completion of a call in flight when steering lands, so a call still
            // open here never reports: settle it instead of spinning forever.
            if u.get("cost").is_some()
                && let Some(r) = self.running.as_mut()
            {
                for (_, t) in std::mem::take(&mut r.tools) {
                    self.complete(
                        t.item,
                        ItemStatus::Interrupted,
                        tool_body(t, "(the agent did not report a result)"),
                    )
                    .await?;
                }
            }
            if let Some(agent) = &self.agent
                && let Some(usage) = session_usage(&u, agent.backend.clone(), agent.model.clone())
            {
                self.record(self.running.as_ref().map(|r| r.turn), Item::Usage { usage })
                    .await?;
            } else {
                self.native_update(&u).await?;
            }
            return Ok(());
        }
        if self.running.is_none() {
            if !self.hide_idle
                && matches!(
                    kind.as_str(),
                    "agent_message_chunk"
                        | "agent_thought_chunk"
                        | "tool_call"
                        | "tool_call_update"
                        | "plan"
                )
            {
                self.start_agent_turn().await?;
            } else {
                return self.native_update(&u).await;
            }
        }
        match kind.as_str() {
            "agent_message_chunk" | "agent_thought_chunk" => {
                let text = u
                    .pointer("/content/text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                if text.is_empty() {
                    return Ok(());
                }
                let thought = kind == "agent_thought_chunk";
                self.chunk(thought, text).await
            }
            "tool_call" | "tool_call_update" => {
                self.flush_text().await?;
                let item = self.tool_for(&u).await?;
                self.publish(
                    Event::Status {
                        phase: Phase::Tool { item },
                    },
                    None,
                )
                .await?;
                let status = u.get("status").and_then(Value::as_str).unwrap_or("");
                if matches!(status, "completed" | "failed") {
                    let id = u
                        .get("toolCallId")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned();
                    let r = self.running.as_mut().expect("running");
                    // a permission still open for this call settles first: a child before its parent
                    let open: Vec<ItemId> = r
                        .permissions
                        .iter()
                        .filter(|(_, p)| p.tool == item)
                        .map(|(i, _)| *i)
                        .collect();
                    for i in open {
                        self.settle_permission(i, None, Resolver::Cancelled, None)
                            .await?;
                    }
                    let t = self
                        .running
                        .as_mut()
                        .expect("running")
                        .tools
                        .remove(&id)
                        .expect("tool");
                    let st = if status == "failed" {
                        ItemStatus::Failed {
                            code: None,
                            reason: "tool failed".into(),
                        }
                    } else {
                        ItemStatus::Completed
                    };
                    self.complete(item, st, tool_body(t, "(no output)")).await?;
                    self.publish(
                        Event::Status {
                            phase: Phase::Thinking,
                        },
                        None,
                    )
                    .await?;
                }
                Ok(())
            }
            "plan" => {
                if u.get("entries")
                    .and_then(Value::as_array)
                    .is_some_and(|entries| {
                        entries.iter().any(|e| {
                            !matches!(
                                e.get("status").and_then(Value::as_str),
                                Some("pending" | "in_progress" | "completed")
                            ) || e
                                .get("priority")
                                .and_then(Value::as_str)
                                .is_some_and(|p| !matches!(p, "low" | "medium" | "high"))
                        })
                    })
                {
                    return self.native_update(&u).await;
                }
                let entries: Vec<PlanEntry> = u
                    .get("entries")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|e| PlanEntry {
                        id: None,
                        text: e
                            .get("content")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_owned(),
                        status: match e.get("status").and_then(Value::as_str) {
                            Some("completed") => PlanStatus::Completed,
                            Some("in_progress") => PlanStatus::InProgress,
                            _ => PlanStatus::Pending,
                        },
                        priority: match e.get("priority").and_then(Value::as_str) {
                            Some("high") => Some(Priority::High),
                            Some("medium") => Some(Priority::Medium),
                            Some("low") => Some(Priority::Low),
                            _ => None,
                        },
                    })
                    .collect();
                let r = self.running.as_mut().expect("running");
                let turn = r.turn;
                let item = match &mut r.plan {
                    Some((item, old)) => {
                        *old = entries.clone();
                        *item
                    }
                    None => {
                        let item = ItemId::new();
                        r.plan = Some((item, entries.clone()));
                        self.start_item(Some(turn), item, &Item::Plan { entries: vec![] }, None)
                            .await?;
                        item
                    }
                };
                self.publish(
                    Event::ItemSnapshot {
                        item,
                        partial: Item::Plan { entries },
                    },
                    None,
                )
                .await
            }
            _ => self.native_update(&u).await,
        }
    }
    async fn chunk(&mut self, thought: bool, text: String) -> std::io::Result<()> {
        let r = self.running.as_mut().expect("running");
        let turn = r.turn;
        // a switch between thought and message closes the other stream
        let other_open = if thought {
            r.message.is_some()
        } else {
            r.thought.is_some()
        };
        if other_open {
            let (message, reasoning) = (r.message.take(), r.thought.take());
            if thought {
                r.thought = reasoning;
                if let Some((item, t)) = message {
                    self.complete(
                        item,
                        ItemStatus::Completed,
                        Item::AgentMessage { markdown: t },
                    )
                    .await?;
                }
            } else {
                r.message = message;
                if let Some((item, t)) = reasoning {
                    self.complete(
                        item,
                        ItemStatus::Completed,
                        Item::Reasoning {
                            summary: t,
                            raw: None,
                            provider_opaque: None,
                        },
                    )
                    .await?;
                }
            }
        }
        let r = self.running.as_mut().expect("running");
        let slot = if thought {
            &mut r.thought
        } else {
            &mut r.message
        };
        let item = match slot {
            Some((item, buf)) => {
                buf.push_str(&text);
                *item
            }
            None => {
                let item = ItemId::new();
                *slot = Some((item, text.clone()));
                let body = if thought {
                    Item::Reasoning {
                        summary: String::new(),
                        raw: None,
                        provider_opaque: None,
                    }
                } else {
                    Item::AgentMessage {
                        markdown: String::new(),
                    }
                };
                self.start_item(Some(turn), item, &body, None).await?;
                if !thought {
                    self.publish(
                        Event::Status {
                            phase: Phase::Streaming,
                        },
                        None,
                    )
                    .await?;
                }
                item
            }
        };
        let delta = if thought {
            Delta::Reasoning { append: text }
        } else {
            Delta::Text { append: text }
        };
        self.publish(Event::ItemDelta { item, delta }, None).await
    }
    /// The tool item for an ACP tool call, started on first sight and updated in place.
    async fn tool_for(&mut self, u: &Value) -> std::io::Result<ItemId> {
        let id = u
            .get("toolCallId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let r = self.running.as_mut().expect("running");
        let turn = r.turn;
        let fresh = !r.tools.contains_key(&id);
        if fresh {
            let kind = tool_kind(u.get("kind").and_then(Value::as_str).unwrap_or("other"));
            let name = u
                .get("title")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("tool")
                .to_owned();
            r.tools.insert(
                id.clone(),
                Tool {
                    item: ItemId::new(),
                    call_id: if id.is_empty() {
                        ItemId::new().to_string()
                    } else {
                        id.clone()
                    },
                    name,
                    kind,
                    input: String::new(),
                    output: vec![],
                    locations: vec![],
                    raw_output: None,
                },
            );
        }
        let t = r.tools.get_mut(&id).expect("tool");
        if let Some(raw) = u.get("rawOutput") {
            t.raw_output = Some(raw.clone());
        }
        if let Some(raw) = u.get("rawInput").filter(|v| !v.is_null()) {
            t.input = raw.to_string();
        }
        if let Some(locs) = u.get("locations").and_then(Value::as_array) {
            t.locations = locs
                .iter()
                .filter_map(|l| {
                    Some(Location {
                        path: l.get("path")?.as_str()?.into(),
                        line: l
                            .get("line")
                            .and_then(Value::as_u64)
                            .and_then(|n| u32::try_from(n).ok()),
                        end_line: l
                            .get("endLine")
                            .and_then(Value::as_u64)
                            .and_then(|n| u32::try_from(n).ok()),
                    })
                })
                .collect();
        }
        if let Some(content) = u.get("content").and_then(Value::as_array) {
            t.output = content
                .iter()
                .map(|value| {
                    output_block(value).unwrap_or_else(|| OutputBlock::Json {
                        value: value.clone(),
                    })
                })
                .collect();
        }
        let item = t.item;
        if fresh {
            let head = Item::ToolCall {
                title: None,
                server: None,
                raw_output: None,
                call_id: t.call_id.clone(),
                name: t.name.clone(),
                tool_kind: t.kind.clone(),
                input_raw: String::new(),
                output: vec![],
                locations: vec![],
                spill: None,
            };
            self.start_item(Some(turn), item, &head, None).await?;
        }
        Ok(item)
    }
}

// ---------- pure mapping ----------
/// Conservative source-key consumption. Nested tool content stays opaque until
/// every native field has a typed equivalent; _meta and rawOutput always survive.
pub fn update_remainder(update: &Value) -> Value {
    let mut extra = update.clone();
    if let Some(o) = extra.as_object_mut() {
        o.remove("sessionUpdate");
        match update.get("sessionUpdate").and_then(Value::as_str) {
            Some("agent_message_chunk" | "agent_thought_chunk")
                if update.pointer("/content/type").and_then(Value::as_str) == Some("text") =>
            {
                if let Some(content) = o.get_mut("content").and_then(Value::as_object_mut) {
                    if content.get("text").is_some_and(Value::is_string) {
                        content.remove("text");
                        content.remove("type");
                    }
                    if content.is_empty() {
                        o.remove("content");
                    }
                }
            }
            Some("tool_call" | "tool_call_update") => {
                if o.get("toolCallId").is_some_and(Value::is_string) {
                    o.remove("toolCallId");
                }
                if o.get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|s| matches!(s, "completed" | "failed"))
                {
                    o.remove("status");
                }
                if o.get("rawInput").is_some_and(|v| !v.is_null()) {
                    o.remove("rawInput");
                }
            }
            _ => {}
        }
        if !o.is_empty()
            && let Some(kind) = update.get("sessionUpdate")
        {
            o.insert("sessionUpdate".into(), kind.clone());
        }
    }
    extra
}
/// Offline projection uses the same content and usage primitives as the live
/// actor, without launching an agent or reusing its lifecycle identifiers.
pub fn reproject_update(provider: &str, u: &Value) -> Option<Item> {
    match u.get("sessionUpdate")?.as_str()? {
        "agent_message_chunk" => Some(Item::AgentMessage {
            markdown: u.pointer("/content/text")?.as_str()?.into(),
        }),
        "agent_thought_chunk" => Some(Item::Reasoning {
            summary: u.pointer("/content/text")?.as_str()?.into(),
            raw: None,
            provider_opaque: None,
        }),
        "usage_update" => session_usage(
            u,
            Backend {
                kind: BackendKind::AcpAgent,
                name: provider.trim_start_matches("acp:").into(),
                version: None,
            },
            None,
        )
        .map(|usage| Item::Usage { usage }),
        "tool_call" | "tool_call_update"
            if matches!(
                u.get("status").and_then(Value::as_str),
                Some("completed" | "failed")
            ) =>
        {
            Some(Item::ToolCall {
                call_id: u.get("toolCallId")?.as_str()?.into(),
                name: u
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("tool")
                    .into(),
                tool_kind: tool_kind(u.get("kind").and_then(Value::as_str).unwrap_or("other")),
                title: u.get("title").and_then(Value::as_str).map(str::to_owned),
                server: None,
                raw_output: u.get("rawOutput").cloned(),
                input_raw: u.get("rawInput").map(Value::to_string).unwrap_or_default(),
                output: u
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(output_block)
                    .collect(),
                locations: vec![],
                spill: None,
            })
        }
        _ => None,
    }
}
/// ACP usage_update is cumulative session context/cost, not call billing.
/// Log IR v1 §4a keeps provenance and labels required-token placeholders.
pub fn session_usage(update: &Value, backend: Backend, model: Option<String>) -> Option<Usage> {
    let context_used = update.get("used").and_then(Value::as_u64);
    let context_window = update.get("size").and_then(Value::as_u64);
    let cost = update.get("cost").and_then(|cost| {
        let currency = cost.get("currency")?.as_str()?;
        if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return None;
        }
        let amount = cost.get("amount")?.as_f64()?;
        // Convert the reported amount to the IR's nearest micro-unit. Reject
        // invalid/overflowing numbers rather than saturating a bill to u64::MAX.
        let micros = (amount * 1_000_000.).round();
        if amount < 0. || !micros.is_finite() || micros >= u64::MAX as f64 {
            return None;
        }
        Some(Cost {
            amount_micros: micros as u64,
            currency: currency.to_owned(),
            source: NumberSource::Reported,
            price_source: Some("provider".into()),
        })
    });
    if context_used.is_none() && context_window.is_none() && cost.is_none() {
        return None;
    }
    Some(Usage {
        scope: Some(UsageScope::Session),
        backend: Some(Box::new(backend)),
        model,
        context_used,
        context_window,
        cost_micros_usd: cost
            .as_ref()
            .filter(|c| c.currency == "USD")
            .map(|c| c.amount_micros),
        cost: cost.map(Box::new),
        source: Some(NumberSource::Reported),
        unreported: vec!["input_tokens".into(), "output_tokens".into()],
        ..Default::default()
    })
}
fn model_option(new_session: &Value) -> Option<&Value> {
    new_session
        .get("configOptions")
        .and_then(Value::as_array)?
        .iter()
        .find(|o| {
            o.get("category").and_then(Value::as_str) == Some("model")
                || o.get("id").and_then(Value::as_str) == Some("model")
        })
}
pub enum ModelChoice {
    Current,
    Set(String),
    NotOffered(Option<String>),
}
/// Match a profile model against the agent's `model` config option (session/new result):
/// Exact value, provider-prefixed suffix, then an offered alias matching a whole
/// '-', '/' or '.' separated token of the wanted model (e.g. haiku).
pub fn pick_model(new_session: &Value, wanted: &str) -> ModelChoice {
    let normalized;
    let option = if let Some(option) = model_option(new_session) {
        option
    } else {
        let mut controls = AgentConfig::default();
        controls.replace(new_session);
        let Some(o) = controls.options.iter().find(|o| o.category == "model") else {
            return ModelChoice::NotOffered(None);
        };
        normalized = json!({"currentValue":o.current,"options":o.choices.iter().map(|c| json!({"value":c.value})).collect::<Vec<_>>()});
        &normalized
    };
    let current = option
        .get("currentValue")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let values: Vec<&str> = option
        .get("options")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|e| {
            e.get("options")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(e))
        })
        .filter_map(|e| e.get("value").and_then(Value::as_str))
        .collect();
    let suffix = format!("/{wanted}");
    let found = values
        .iter()
        .find(|v| **v == wanted)
        .or_else(|| values.iter().find(|v| v.ends_with(&suffix)))
        .or_else(|| {
            values
                .iter()
                .find(|v| !v.is_empty() && wanted.split(['-', '/', '.']).any(|token| token == **v))
        });
    match found {
        Some(v) if current.as_deref() == Some(*v) => ModelChoice::Current,
        Some(v) => ModelChoice::Set((*v).to_owned()),
        None => ModelChoice::NotOffered(current),
    }
}
/// The conversation in a log as plain text for a fresh agent: user and assistant
/// messages and one line per tool call, newest kept when over budget.
pub fn handoff(events: &[Envelope]) -> Option<String> {
    const BUDGET: usize = 24 * 1024;
    let mut lines: Vec<String> = vec![];
    for e in events {
        if let Event::ItemCompleted { body, .. } = &e.event {
            match body {
                Item::UserMessage { blocks } => {
                    let text: Vec<&str> = blocks
                        .iter()
                        .filter_map(|b| match b {
                            InputBlock::Text { text } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();
                    lines.push(format!("[user] {}", text.join("\n")));
                }
                Item::AgentMessage { markdown } if !markdown.trim().is_empty() => {
                    lines.push(format!("[assistant] {}", markdown.trim()))
                }
                Item::ToolCall {
                    name,
                    input_raw,
                    output,
                    ..
                } => {
                    let first = output
                        .iter()
                        .find_map(|o| match o {
                            OutputBlock::Text { text } | OutputBlock::Error { text } => {
                                text.lines().next()
                            }
                            _ => None,
                        })
                        .unwrap_or("");
                    let input: String = input_raw.chars().take(200).collect();
                    lines.push(format!(
                        "[tool {name}] {input} -> {}",
                        first.chars().take(200).collect::<String>()
                    ));
                }
                _ => {}
            }
        }
    }
    if !lines.iter().any(|l| l.starts_with("[user]")) {
        return None;
    }
    let mut out: Vec<String> = vec![];
    let mut used = 0;
    for line in lines.into_iter().rev() {
        if used + line.len() > BUDGET {
            out.push("[earlier turns omitted]".into());
            break;
        }
        used += line.len() + 1;
        out.push(line);
    }
    out.reverse();
    Some(out.join("\n"))
}
fn agent_label(spec: &AgentSpec) -> String {
    let program = spec.command.first().map(String::as_str).unwrap_or("agent");
    let program = program.rsplit('/').next().unwrap_or(program);
    format!(
        "acp:{}",
        spec.command
            .iter()
            .skip(1)
            .fold(program.to_owned(), |a, s| format!("{a} {s}"))
    )
}
fn call_name(c: &Call) -> &'static str {
    match c {
        Call::Initialize => "initialize",
        Call::NewSession => "session/new",
        Call::SetPermissions { .. } => "unrestricted permissions",
        Call::SetModel { .. } => "the model choice",
        Call::Prompt => "the prompt",
        Call::Steer { .. } => "steering",
        Call::Configure { .. } => "session configuration",
    }
}
pub fn tool_kind(k: &str) -> ToolKind {
    match k {
        "read" => ToolKind::Read,
        "edit" => ToolKind::Edit,
        "delete" => ToolKind::Delete,
        "move" => ToolKind::Move,
        "search" => ToolKind::Search,
        "execute" => ToolKind::Execute,
        "think" => ToolKind::Think,
        "fetch" => ToolKind::Fetch,
        _ => ToolKind::Other,
    }
}
fn tool_body(t: Tool, empty: &str) -> Item {
    let output = if t.output.is_empty() {
        match t.raw_output.as_ref().filter(|value| !value.is_null()) {
            Some(value) => vec![OutputBlock::Json {
                value: value.clone(),
            }],
            None => vec![OutputBlock::Text { text: empty.into() }],
        }
    } else {
        t.output
    };
    Item::ToolCall {
        title: None,
        server: None,
        raw_output: t.raw_output,
        call_id: t.call_id,
        name: t.name,
        tool_kind: t.kind,
        input_raw: t.input,
        output,
        locations: t.locations,
        spill: None,
    }
}

fn decided(body: Item, decision: Option<ApprovalDecision>, by: Resolver) -> Item {
    match body {
        Item::Approval {
            subject, options, ..
        } => Item::Approval {
            subject,
            options,
            decision: Some(decision.unwrap_or(ApprovalDecision::Deny {
                reason: Some("cancelled".into()),
            })),
            by: Some(by),
        },
        other => other,
    }
}
/// ACP tool-call content → harness output blocks. A diff becomes unified text.
pub fn output_block(c: &Value) -> Option<OutputBlock> {
    match c.get("type")?.as_str()? {
        "content" => {
            let inner = c.get("content")?;
            match inner.get("type")?.as_str()? {
                "text" => Some(OutputBlock::Text {
                    text: inner.get("text")?.as_str()?.to_owned(),
                }),
                _ => None,
            }
        }
        "diff" => {
            let path = c.get("path")?.as_str()?;
            let old = c.get("oldText").and_then(Value::as_str).unwrap_or("");
            let new = c.get("newText").and_then(Value::as_str).unwrap_or("");
            Some(OutputBlock::Diff {
                path: path.into(),
                unified: unified(path, old, new),
                old_text: c.get("oldText").and_then(Value::as_str).map(str::to_owned),
                new_text: c.get("newText").and_then(Value::as_str).map(str::to_owned),
                change: Some(if c.get("oldText").is_none_or(Value::is_null) {
                    ChangeKind::Add
                } else if c.get("newText").is_none_or(Value::is_null) {
                    ChangeKind::Delete
                } else {
                    ChangeKind::Modify
                }),
            })
        }
        "terminal" => Some(OutputBlock::Terminal {
            terminal: c
                .get("terminalId")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .into(),
            exit: c
                .get("exitCode")
                .and_then(Value::as_i64)
                .and_then(|n| i32::try_from(n).ok()),
        }),
        _ => None,
    }
}
pub fn unified(path: &str, old: &str, new: &str) -> String {
    let (a, b): (Vec<&str>, Vec<&str>) = (old.lines().collect(), new.lines().collect());
    let mut out = format!(
        "--- a/{path}\n+++ b/{path}\n@@ -1,{} +1,{} @@\n",
        a.len(),
        b.len()
    );
    for l in a {
        out.push_str(&format!("-{l}\n"));
    }
    for l in b {
        out.push_str(&format!("+{l}\n"));
    }
    out
}
/// Harness input → ACP prompt content. Mentions become resource links; images are not sent in phase 1.
pub fn content_block(b: &InputBlock) -> Value {
    match b {
        InputBlock::Text { text } => json!({ "type": "text", "text": text }),
        InputBlock::Mention { path } => json!({
            "type": "resource_link",
            "uri": format!("file://{}", path.display()),
            "name": path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string()),
        }),
        InputBlock::Image { blob } => {
            json!({ "type": "resource_link", "uri":format!("bboxy-blob://{}",blob.sha256), "name":blob.mime,"mimeType":blob.mime })
        }
        InputBlock::ResourceLink { uri, title } => {
            json!({"type":"resource_link","uri":uri,"name":title.as_deref().unwrap_or(uri)})
        }
        InputBlock::Resource { uri, mime, .. } => {
            json!({"type":"resource_link","uri":uri,"name":uri,"mimeType":mime})
        }
        InputBlock::Audio { .. } => json!({"type":"text","text":b.display_text()}),
        InputBlock::Unknown(_) => json!({ "type": "text", "text": "Unsupported input" }),
    }
}
fn answered(body: Item, answer: Value) -> Item {
    match body {
        Item::Question {
            message, schema, ..
        } => Item::Question {
            message,
            schema,
            answer: Some(answer),
        },
        other => other,
    }
}
fn permission_reply(id: &Value, option: Option<&str>) -> Value {
    let outcome = match option {
        Some(o) => json!({ "outcome": "selected", "optionId": o }),
        None => json!({ "outcome": "cancelled" }),
    };
    json!({ "jsonrpc": "2.0", "id": id, "result": { "outcome": outcome } })
}
fn call(agent: &mut Agent, kind: Call, method: &str, params: Value) -> std::io::Result<()> {
    agent.next_id += 1;
    let id = agent.next_id;
    agent.calls.insert(id, kind);
    write(
        agent,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    )
}
fn notify(agent: &mut Agent, method: &str, params: Value) -> std::io::Result<()> {
    write(
        agent,
        json!({ "jsonrpc": "2.0", "method": method, "params": params }),
    )
}
fn reply(agent: &mut Agent, message: Value) -> std::io::Result<()> {
    write(agent, message)
}
/// `BBOXY450_ACP_TRACE=<file>`: append every JSON-RPC line, both directions, for debugging
/// an agent. Off by default; the file may contain prompts, so it is never on by itself.
fn trace(direction: &str, line: &str) {
    if let Some(path) = std::env::var_os("BBOXY450_ACP_TRACE") {
        use std::io::Write;
        #[allow(clippy::disallowed_methods)]
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{direction} {}", line.trim_end());
        }
    }
}
fn write(agent: &mut Agent, message: Value) -> std::io::Result<()> {
    let mut line = message.to_string();
    trace(">", &line);
    line.push('\n');
    agent
        .out
        .send(line)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "agent stdin closed"))
}
async fn write_lines(mut stdin: ChildStdin, mut lines: mpsc::UnboundedReceiver<String>) {
    while let Some(line) = lines.recv().await {
        if stdin.write_all(line.as_bytes()).await.is_err() || stdin.flush().await.is_err() {
            return;
        }
    }
}
async fn read_lines(stdout: tokio::process::ChildStdout, tx: mpsc::UnboundedSender<Wire>) {
    let mut reader = BufReader::with_capacity(64 * 1024, stdout);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let mut limited = (&mut reader).take(MAX_LINE as u64 + 1);
        match limited.read_until(b'\n', &mut buf).await {
            Ok(0) => {
                let _ = tx.send(Wire::Eof);
                return;
            }
            Ok(_) if buf.len() > MAX_LINE => {
                let _ = tx.send(Wire::Bad(format!("a line over {MAX_LINE} bytes")));
                return;
            }
            Ok(_) => {
                let line = String::from_utf8_lossy(&buf);
                let line = line.trim();
                trace("<", line);
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(line) {
                    Ok(v) => {
                        if tx.send(Wire::Message(v)).is_err() {
                            return;
                        }
                    }
                    Err(_) => {
                        let _ = tx.send(Wire::Bad("malformed JSON-RPC".into()));
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Wire::Bad(format!("an unreadable stream ({e})")));
                return;
            }
        }
    }
}
async fn read_tail(
    mut pipe: tokio::process::ChildStderr,
    tail: Arc<Mutex<String>>,
    done: Arc<std::sync::atomic::AtomicBool>,
) {
    let mut chunk = [0u8; 4096];
    while let Ok(n) = pipe.read(&mut chunk).await {
        if n == 0 {
            break;
        }
        if let Ok(mut t) = tail.lock() {
            t.push_str(&String::from_utf8_lossy(&chunk[..n]));
            if t.len() > STDERR_TAIL {
                let mut cut = t.len() - STDERR_TAIL;
                while !t.is_char_boundary(cut) {
                    cut += 1;
                }
                t.drain(..cut);
            }
        }
    }
    done.store(true, std::sync::atomic::Ordering::Release);
}
async fn recv(rx: &mut Option<mpsc::UnboundedReceiver<Wire>>) -> Option<Wire> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}
async fn sleep_until(at: Option<Instant>) {
    match at {
        Some(at) => tokio::time::sleep_until(at).await,
        None => std::future::pending().await,
    }
}
