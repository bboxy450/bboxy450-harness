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
//! - `*_always` options are never chosen automatically; unattended deny is `reject_once`;
//! - the harness never offers, selects or triggers an agent's claude.ai login method.
use crate::{
    engine::Config,
    gateway::{Admitted, Hub, InProc},
    process::OwnedChild,
    protocol::*,
    provider::profile::{AgentSpec, Unattended},
    store::Store,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
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
const STDERR_TAIL: usize = 2048;

enum Wire {
    Message(Value),
    Bad(String),
    Eof,
}
enum Call {
    Initialize,
    NewSession,
    Prompt,
}
struct Agent {
    child: OwnedChild,
    /// whole lines, written in order by the writer task
    out: mpsc::UnboundedSender<String>,
    session: Option<String>,
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
    cause: OpId,
    prompt_sent: bool,
    cancel_at: Option<Instant>,
    interrupted: bool,
    message: Option<(ItemId, String)>,
    thought: Option<(ItemId, String)>,
    tools: BTreeMap<String, Tool>,
    plan: Option<(ItemId, Vec<PlanEntry>)>,
    permissions: BTreeMap<ItemId, Permission>,
}

pub struct AcpAgentActor {
    store: Store,
    gateway: InProc,
    work: mpsc::Receiver<Admitted>,
    control: mpsc::Receiver<Admitted>,
    config: Config,
    spec: AgentSpec,
    yolo: bool,
    agent: Option<Agent>,
    wire: Option<mpsc::UnboundedReceiver<Wire>>,
    queue: VecDeque<(TurnId, Vec<InputBlock>, OpId)>,
    running: Option<Running>,
    resolved: HashSet<ItemId>,
    warned: HashSet<String>,
    restarted: bool,
}

impl AcpAgentActor {
    pub(crate) async fn load(
        mut config: Config,
        start: OpId,
        spec: AgentSpec,
        yolo: bool,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
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
                        model: agent_label(&spec),
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
        // a reopened session restarts its agent: the agent's own context starts fresh
        let restarted = !store.fresh
            && store
                .events
                .iter()
                .any(|e| matches!(e.event, Event::TurnStarted { .. }));
        let actor = Self {
            store,
            gateway: gateway.clone(),
            work,
            control,
            config,
            spec,
            yolo,
            agent: None,
            wire: None,
            queue: VecDeque::new(),
            running: None,
            resolved: HashSet::new(),
            warned: HashSet::new(),
            restarted,
        };
        let task = bbt_rt::handle().spawn(actor.run());
        Ok((gateway, task))
    }

    async fn publish(&mut self, event: Event, cause: Option<OpId>) -> std::io::Result<()> {
        let e = if event.is_durable() {
            self.store.append(event, cause).await?
        } else {
            self.config
                .redactor
                .sanitize(&self.store.ephemeral(event))
                .map_err(std::io::Error::other)?
        };
        self.gateway.hub.publish(e);
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
        self.publish(
            Event::ItemStarted {
                turn,
                item,
                kind: body.kind(),
                parent,
                head: body.head(),
            },
            None,
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
                let cancel_at = self.running.as_ref().and_then(|r| r.cancel_at);
                tokio::select! {biased;
                    Some(op) = self.control.recv() => { if self.op(op).await? { break; } }
                    Some(op) = self.work.recv() => { if self.op(op).await? { break; } }
                    Some(wire) = recv(&mut self.wire) => self.wire(wire).await?,
                    _ = sleep_until(cancel_at) => self.cancel_expired().await?,
                    _ = tick.tick() => self.gateway.hub.flush_deltas(),
                    _ = heartbeat.tick(), if self.running.is_none() =>
                        self.publish(Event::Status { phase: Phase::Idle }, None).await?,
                }
            }
            Ok::<_, std::io::Error>(())
        }
        .await;
        self.kill();
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
            Op::UserTurn { blocks, .. } => {
                // ACP has no steering: a turn sent while another runs waits its turn
                if blocks.is_empty() || blocks.iter().any(|b| matches!(b, InputBlock::Unknown)) {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "provide supported input blocks".into(),
                        },
                    )
                    .await?;
                } else if self.queue.len() >= 64 {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "turn queue is full".into(),
                        },
                    )
                    .await?;
                } else {
                    let turn = TurnId::new();
                    self.publish(Event::TurnQueued { turn }, Some(id)).await?;
                    self.queue.push_back((turn, blocks, id));
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
                    self.publish(Event::TurnStarted { turn }, Some(id)).await?;
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
            Op::Compact => self.reject(id, Rejection::NothingToCompact).await?,
            Op::Close => {
                self.close(id).await?;
                return Ok(true);
            }
            Op::StartSession { .. } | Op::Unknown => {
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

    async fn close(&mut self, id: OpId) -> std::io::Result<()> {
        self.gateway.seal();
        if self.running.is_some() {
            self.send_cancel();
            self.end_turn(StopReason::Cancelled, ItemStatus::Cancelled)
                .await?;
        }
        while let Some((turn, _, _)) = self.queue.pop_front() {
            self.publish(Event::TurnStarted { turn }, Some(id)).await?;
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
        self.publish(Event::TurnStarted { turn }, None).await?;
        self.running = Some(Running {
            turn,
            blocks: blocks.clone(),
            cause,
            prompt_sent: false,
            cancel_at: None,
            interrupted: false,
            message: None,
            thought: None,
            tools: BTreeMap::new(),
            plan: None,
            permissions: BTreeMap::new(),
        });
        self.record(Some(turn), Item::UserMessage { blocks })
            .await?;
        self.publish(
            Event::Status {
                phase: Phase::Thinking,
            },
            None,
        )
        .await?;
        if self.agent.is_none() {
            if let Err(error) = self.spawn() {
                let message = format!("ACP agent could not start: {error}");
                self.notice(Level::Error, message.clone()).await?;
                return self
                    .end_turn(
                        StopReason::Error {
                            class: ErrorClass::Config,
                            message,
                        },
                        ItemStatus::Failed {
                            reason: "agent unavailable".into(),
                        },
                    )
                    .await;
            }
            if self.restarted {
                self.restarted = false;
                self.notice(
                    Level::Info,
                    "ACP agent started fresh: earlier turns are in this log, not in the agent's context".into(),
                )
                .await?;
            }
        } else {
            self.send_prompt()?;
        }
        Ok(())
    }
    fn send_prompt(&mut self) -> std::io::Result<()> {
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
        let prompt: Vec<Value> = r.blocks.iter().map(content_block).collect();
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
        r.cancel_at = Some(Instant::now() + CANCEL_GRACE);
        let sent = r.prompt_sent;
        let pending: Vec<ItemId> = r.permissions.keys().copied().collect();
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
                "ACP agent did not stop within {}s of cancel; its process group was killed",
                CANCEL_GRACE.as_secs()
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
            Some(r.cause),
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
    fn spawn(&mut self) -> std::io::Result<()> {
        let mut argv = self.spec.command.iter();
        let program = argv
            .next()
            .ok_or_else(|| std::io::Error::other("agent.command is empty"))?;
        let mut command = tokio::process::Command::new(program);
        command
            .args(argv)
            .current_dir(&self.config.meta.cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
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
            next_id: 0,
            calls: HashMap::new(),
            stderr,
            stderr_done,
        };
        let params = json!({
            "protocolVersion": 1,
            // phase 1 serves no files or terminals; the agent uses its own tools
            "clientCapabilities": { "fs": { "readTextFile": false, "writeTextFile": false }, "terminal": false },
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
                let method = m.get("method").and_then(Value::as_str).map(str::to_owned);
                match (method, m.get("id").cloned()) {
                    (Some(method), Some(id)) => {
                        self.request(&method, id, m.get("params").cloned().unwrap_or(Value::Null))
                            .await
                    }
                    (Some(method), None) => {
                        if method == "session/update" {
                            let update =
                                m.pointer("/params/update").cloned().unwrap_or(Value::Null);
                            self.update(update).await
                        } else {
                            Ok(())
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
                Call::Prompt | Call::Initialize | Call::NewSession if self.running.is_some() => {
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
                call(
                    agent,
                    Call::NewSession,
                    "session/new",
                    json!({ "cwd": cwd, "mcpServers": [] }),
                )
            }
            Call::NewSession => {
                let Some(session) = result.get("sessionId").and_then(Value::as_str) else {
                    return self
                        .agent_gone("ACP agent returned no sessionId".into())
                        .await;
                };
                self.agent.as_mut().expect("agent").session = Some(session.to_owned());
                self.send_prompt()
            }
            Call::Prompt => {
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
                    _ => Effect::Unknown,
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
        // no face can answer: decide by the profile, never with an *_always option
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
            Some(ApprovalDecision::Allow) => &["allow_once"],
            Some(ApprovalDecision::AllowAlways) => &["allow_always"],
            Some(ApprovalDecision::Deny { .. }) => &["reject_once", "reject_always"],
            _ => &[],
        };
        let option = wanted.iter().find_map(|k| {
            p.options
                .iter()
                .find(|(kind, _)| kind == k)
                .map(|(_, id)| id.clone())
        });
        if let Some(agent) = self.agent.as_mut() {
            let _ = reply(agent, permission_reply(&p.rpc, option.as_deref()));
        }
        let decision = if option.is_some() {
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
        let kind = u
            .get("sessionUpdate")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if self.running.is_none() {
            // between turns only metadata may arrive; content without a turn is not ours to show
            return Ok(());
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
                let entries: Vec<PlanEntry> = u
                    .get("entries")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .map(|e| PlanEntry {
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
            // session metadata: kept by the agent, nothing to draw in phase 1
            "available_commands_update"
            | "current_mode_update"
            | "config_option_update"
            | "session_info_update"
            | "usage_update"
            | "user_message_chunk" => Ok(()),
            other => {
                if self.warned.insert(other.to_owned()) {
                    self.notice(
                        Level::Warn,
                        format!("ACP agent sent an update this client does not show: {other}"),
                    )
                    .await?;
                }
                Ok(())
            }
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
                },
            );
        }
        let t = r.tools.get_mut(&id).expect("tool");
        if let Some(raw) = u.get("rawInput").filter(|v| !v.is_null()) {
            t.input = raw.to_string();
        }
        if let Some(locs) = u.get("locations").and_then(Value::as_array) {
            t.locations = locs
                .iter()
                .filter_map(|l| {
                    Some(Location {
                        path: l.get("path")?.as_str()?.into(),
                        line: l.get("line").and_then(Value::as_u64).map(|n| n as u32),
                        end_line: None,
                    })
                })
                .collect();
        }
        if let Some(content) = u.get("content").and_then(Value::as_array) {
            t.output = content.iter().filter_map(output_block).collect();
        }
        let item = t.item;
        if fresh {
            let head = Item::ToolCall {
                call_id: t.call_id.clone(),
                name: t.name.clone(),
                tool_kind: t.kind,
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
        Call::Prompt => "the prompt",
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
    Item::ToolCall {
        call_id: t.call_id,
        name: t.name,
        tool_kind: t.kind,
        input_raw: t.input,
        output: if t.output.is_empty() {
            vec![OutputBlock::Text { text: empty.into() }]
        } else {
            t.output
        },
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
                other => Some(OutputBlock::Text {
                    text: format!("[{other} content]"),
                }),
            }
        }
        "diff" => {
            let path = c.get("path")?.as_str()?;
            let old = c.get("oldText").and_then(Value::as_str).unwrap_or("");
            let new = c.get("newText").and_then(Value::as_str).unwrap_or("");
            Some(OutputBlock::Diff {
                path: path.into(),
                unified: unified(path, old, new),
            })
        }
        "terminal" => Some(OutputBlock::Text {
            text: format!(
                "[terminal {}]",
                c.get("terminalId").and_then(Value::as_str).unwrap_or("?")
            ),
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
            json!({ "type": "text", "text": format!("[image {} · {} bytes not sent]", blob.mime, blob.len) })
        }
        InputBlock::Unknown => json!({ "type": "text", "text": "" }),
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
fn write(agent: &mut Agent, message: Value) -> std::io::Result<()> {
    let mut line = message.to_string();
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
