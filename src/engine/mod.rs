// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    gateway::{Gateway, Hub, InProc},
    policy::{Action, Decision, EffectClass, Permit, Policy},
    protocol::*,
    provider::{
        Provider,
        profile::{Profile, Unattended},
    },
    redact::Set,
    store::Store,
    tools::Registry,
};
use futures_util::FutureExt;
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
pub mod history;
pub mod task_group;
pub mod turn_loop;
#[derive(Clone)]
pub struct Config {
    pub meta: SessionMeta,
    pub sessions: PathBuf,
    pub profiles: PathBuf,
    pub provider: Arc<dyn Provider>,
    pub model: String,
    pub policy: Policy,
    pub redactor: Set,
    pub max_rounds: u32,
    pub profile_error: Option<String>,
}
/// Trusted process-launch settings, separate from profile/ACP server data.
#[derive(Clone, Default)]
pub struct LaunchOptions {
    /// Explicit launcher approval intent; map to offered allow_once, never vendor flags.
    pub yolo: bool,
    pub append_system_prompt: String,
    pub mcp_servers: crate::mcp::Servers,
    pub notices: Vec<String>,
    /// Opaque trusted scratch capability; cannot be supplied by profile/IR/MCP.
    pub trial: Option<crate::tools::TrialCapability>,
}
// Additive SessionMeta/IR metadata enlarged Event. Keep this private by-value
// channel payload to avoid a new per-event heap allocation in WP1.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Internal {
    ServiceNotice {
        level: Level,
        text: String,
    },
    PrepareRound {
        generation: u64,
        turn: TurnId,
        tools: Vec<crate::provider::ToolDef>,
        reply: oneshot::Sender<std::io::Result<(Vec<serde_json::Value>, usize)>>,
    },

    ProfileReady {
        op: OpId,
        result: Result<(Profile, Arc<dyn Provider>), Rejection>,
    },
    Event {
        generation: u64,
        event: Event,
        cause: Option<OpId>,
    },
    Approval {
        generation: u64,
        turn: TurnId,
        item: ItemId,
        tool: String,
        effect: EffectClass,
        body: Item,
        reply: oneshot::Sender<Result<Permit, String>>,
    },
}
struct Pending {
    turn: TurnId,
    generation: u64,
    tool: String,
    effect: EffectClass,
    body: Item,
    reply: oneshot::Sender<Result<Permit, String>>,
}
enum Queued {
    Turn(TurnId, Vec<InputBlock>, OpId),
    Compact(OpId),
}
struct Running {
    turn: TurnId,
    initial_cause: OpId,
    generation: u64,
    cancel: CancellationToken,
    steer: mpsc::Sender<(OpId, Vec<InputBlock>)>,
    pending: VecDeque<(OpId, Vec<InputBlock>)>,
    cause: Option<OpId>,
}
pub struct SessionActor {
    store: Store,
    gateway: InProc,
    work: mpsc::Receiver<crate::gateway::Admitted>,
    control: mpsc::Receiver<crate::gateway::Admitted>,
    config: Config,
    tools: Registry,
    generation: Arc<AtomicU64>,
    queue: VecDeque<Queued>,
    running: Option<Running>,
    approvals: BTreeMap<ItemId, Pending>,
    resolved: BTreeMap<ItemId, ()>,
    interrupted: BTreeMap<TurnId, OpId>,
    stopping: Option<OpId>,
    internal: mpsc::Receiver<Internal>,
    output: mpsc::Sender<Internal>,
    tasks: JoinSet<(TurnId, u64)>,
    services: task_group::TaskGroup,
    profile_pending: Option<OpId>,
    auto: crate::context::auto::Options,
    window_warned: bool,
    mcp: crate::mcp::Manager,
    append_system_prompt: String,
    launch_yolo: bool,
}
impl SessionActor {
    pub async fn open(
        config: Config,
        start: OpId,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        Self::load(config, start, None).await
    }
    pub(crate) async fn load(
        config: Config,
        start: OpId,
        notice: Option<String>,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        Self::load_with_mcp(
            config,
            start,
            notice,
            Default::default(),
            Default::default(),
        )
        .await
    }
    pub(crate) async fn load_with_mcp(
        config: Config,
        start: OpId,
        notice: Option<String>,
        overrides: crate::mcp::Servers,
        launch: LaunchOptions,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        let actor = Self::new(config, start, notice, overrides, launch).await?;
        let gateway = actor.gateway.clone();
        let task = bbt_rt::handle().spawn(actor.run());
        Ok((gateway, task))
    }
    async fn new(
        mut config: Config,
        start: OpId,
        notice: Option<String>,
        overrides: crate::mcp::Servers,
        launch: LaunchOptions,
    ) -> std::io::Result<Self> {
        // Validate before VCS, Store, provider/profile mutation or MCP startup.
        let trial_profile = if let Some(capability) = launch.trial.clone() {
            if launch.yolo
                || !launch.append_system_prompt.is_empty()
                || !launch.mcp_servers.is_empty()
                || !overrides.is_empty()
                || config.profile_error.is_some()
            {
                return Err(std::io::Error::other("unsupported trial launch widening"));
            }
            let cwd = config.meta.cwd.clone();
            let sessions = config.sessions.clone();
            let profiles = config.profiles.clone();
            crate::runtime::blocking(move || {
                capability.validate_launch(&cwd, &sessions, &profiles)
            })
            .await?;
            let profile = if let Some(name) = &config.meta.profile {
                let path = Profile::file_in(&config.profiles, name)
                    .map_err(|_| std::io::Error::other("trial profile unavailable"))?;
                Some(
                    Profile::load(&path)
                        .await
                        .map_err(|_| std::io::Error::other("trial profile unavailable"))?,
                )
            } else {
                None
            };
            if profile.as_ref().is_some_and(|p| {
                p.is_acp_agent() || !p.mcp_servers.is_empty() || p.model != config.model
            }) {
                return Err(std::io::Error::other("unsupported trial profile/MCP"));
            }
            config.policy = Policy::dream_trial();
            Some(profile)
        } else {
            None
        };
        if launch.yolo {
            config.policy = Policy::allow_all(config.policy.on_unattended);
        }
        crate::vcs::capture_new_session(&mut config.meta, &config.sessions).await;
        let mut store = if let Some(capability) = &launch.trial {
            Store::open_trial(
                config.sessions.clone(),
                config.meta.clone(),
                Some(start),
                config.redactor.clone(),
                capability,
            )
            .await?
        } else {
            Store::open(
                config.sessions.clone(),
                config.meta.clone(),
                Some(start),
                config.redactor.clone(),
            )
            .await?
        };
        if let Some(Envelope {
            event: Event::SessionStarted { meta },
            ..
        }) = store.events.first()
        {
            config.meta = meta.clone();
        }
        if store.fresh {
            if let Some(message) = &config.profile_error {
                let item = ItemId::new();
                store
                    .append(
                        Event::ItemStarted {
                            turn: None,
                            item,
                            kind: ItemKind::Notice,
                            parent: None,
                            head: None,
                        },
                        Some(start),
                    )
                    .await?;
                store
                    .append(
                        Event::ItemCompleted {
                            item,
                            status: ItemStatus::Completed,
                            body: Item::Notice {
                                level: Level::Error,
                                text: message.clone(),
                            },
                        },
                        Some(start),
                    )
                    .await?;
            } else {
                store
                    .append(
                        Event::ProfileChanged {
                            profile: config
                                .meta
                                .profile
                                .clone()
                                .unwrap_or_else(|| "scripted".into()),
                            model: config.model.clone(),
                        },
                        Some(start),
                    )
                    .await?;
            }
        }
        for (level, text) in notice
            .into_iter()
            .map(|text| (Level::Error, text))
            .chain(launch.notices.into_iter().map(|text| (Level::Warn, text)))
            .chain(
                config
                    .redactor
                    .capability_notice()
                    .into_iter()
                    .map(|text| (Level::Warn, text)),
            )
        {
            let item = ItemId::new();
            store
                .append(
                    Event::ItemStarted {
                        turn: None,
                        item,
                        kind: ItemKind::Notice,
                        parent: None,
                        head: None,
                    },
                    None,
                )
                .await?;
            store
                .append(
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body: Item::Notice { level, text },
                    },
                    None,
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
        let (output, internal) = mpsc::channel(256);
        let gateway = InProc::new(hub, work_tx, control_tx, start).with_notices(output.clone());
        let profile = if let Some(profile) = trial_profile {
            profile
        } else if let Some(name) = &config.meta.profile {
            match Profile::file_in(&config.profiles, name) {
                Ok(path) => Profile::load(&path).await.ok(),
                Err(_) => None,
            }
        } else {
            None
        };
        let auto = crate::context::auto::Options {
            enabled: launch.trial.is_none() && profile.as_ref().is_none_or(|p| p.auto_compact),
            window: profile.as_ref().and_then(|p| p.context_tokens),
        };
        let servers = profile
            .as_ref()
            .map(|p| p.mcp_servers.clone())
            .unwrap_or_default();
        let (servers, collisions) =
            crate::mcp::select_sources(servers, overrides, launch.mcp_servers);
        let mut services = task_group::TaskGroup::new(CancellationToken::new());
        let mcp = crate::mcp::Manager::start(
            servers,
            config.redactor.clone(),
            output.clone(),
            &mut services,
        );
        for text in collisions {
            let tx = output.clone();
            let cancel = services.cancel.clone();
            services.spawn(async move {
                let _ = task_group::send(
                    &tx,
                    Internal::ServiceNotice {
                        level: Level::Info,
                        text,
                    },
                    &cancel,
                )
                .await;
            });
        }
        let window_warned=store.events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {status:ItemStatus::Completed,body:Item::Notice {text,..},..} if text=="context window unknown; automatic compaction off"));
        gateway.set_unattended(config.policy.on_unattended);
        let actor = Self {
            store,
            gateway: gateway.clone(),
            work,
            control,
            config,
            tools: launch
                .trial
                .clone()
                .map_or_else(Registry::new, Registry::trial),
            generation: Arc::new(AtomicU64::new(0)),
            queue: VecDeque::new(),
            running: None,
            approvals: BTreeMap::new(),
            resolved: BTreeMap::new(),
            interrupted: BTreeMap::new(),
            stopping: None,
            internal,
            output,
            tasks: JoinSet::new(),
            services,
            mcp,
            profile_pending: None,
            auto,
            window_warned,
            append_system_prompt: launch.append_system_prompt,
            launch_yolo: launch.yolo,
        };
        Ok(actor)
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
        if matches!(e.event, Event::SessionClosed) {
            self.gateway.hub.publish(e.clone());
            crate::dream::d0::session_closed(
                &self.config.sessions,
                &e,
                self.config.redactor.clone(),
            );
            crate::frontends::ir_scan::session_closed(
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
    async fn run(mut self) -> std::io::Result<()> {
        if self.store.closed {
            self.gateway.mark_closed();
            return Ok(());
        }
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(25));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut heartbeat = tokio::time::interval_at(
            tokio::time::Instant::now() + std::time::Duration::from_secs(15),
            std::time::Duration::from_secs(15),
        );
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut work_batch = 0usize;
        let result=async {
   loop {
    if self.running.is_none()&&self.profile_pending.is_none() {
     if let Some(op)=self.stopping {
       self.gateway.seal();self.work.close();self.control.close();
       let _=self.services.cancel_and_join().await;
       while let Some(env)=self.control.recv().await {self.op(env).await?;}
       while let Some(env)=self.work.recv().await {self.reject(env.env.op_id,Rejection::InvalidInput{detail:"session is closing".into()}).await?;}
       while let Some(work)=self.queue.pop_front(){match work {Queued::Turn(turn,_,_)=>self.publish(Event::TurnEnded{turn,reason:StopReason::Cancelled,usage:None},Some(op)).await?,Queued::Compact(id)=>self.reject(id,Rejection::NothingToCompact).await?}}
       self.publish(Event::SessionClosed,Some(op)).await?;break;
     }
     if let Some(work)=self.queue.pop_front(){match work{Queued::Turn(turn,blocks,cause)=>self.start(turn,blocks,cause).await?,Queued::Compact(id)=>{self.compact(id).await?;continue;}}}
    }
    tokio::select! {biased;
     Some(op)=self.control.recv()=>self.op(op).await?,
     Some(op)=self.work.recv(),if self.stopping.is_none()&&self.profile_pending.is_none()&&work_batch<8=>{work_batch+=1;self.op(op).await?;},
     Some(message)=self.internal.recv()=>{work_batch=0;self.internal(message).await?;},
     Some(result)=self.tasks.join_next(),if !self.tasks.is_empty()=> {if let Err(e)=result {self.crashed(&format!("runner failed: {}",if e.is_panic(){"panic"}else{"cancelled"})).await?;}},
     _=tick.tick()=>{work_batch=0;self.gateway.hub.flush_deltas();self.services.reap().await;},
     _=heartbeat.tick(),if self.running.is_none()&&self.profile_pending.is_none()=>self.publish(Event::Status{phase:Phase::Idle},None).await?,
    }
    tokio::task::yield_now().await;
   }Ok(())
  }.await;
        if result.is_err() {
            self.internal.close();
            let _ = self.services.cancel_and_join().await;
            self.gateway.mark_faulted();
            if let Some(r) = &self.running {
                r.cancel.cancel();
            }
            while self.tasks.join_next().await.is_some() {}
            let mut e = self.store.ephemeral(Event::Status {
                phase: Phase::Closed,
            });
            e.event = Event::Status {
                phase: Phase::Closed,
            };
            self.gateway.hub.publish(e);
        }
        self.gateway.mark_closed();
        result
    }
    async fn start(
        &mut self,
        turn: TurnId,
        blocks: Vec<InputBlock>,
        initial_cause: OpId,
    ) -> std::io::Result<()> {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let cancel = CancellationToken::new();
        let (steer, rx) = mpsc::channel(64);
        self.publish(
            Event::TurnStarted {
                turn,
                model: Some(self.config.model.clone()),
                origin: Some(TurnOrigin::User),
            },
            None,
        )
        .await?;
        let (messages, _) = crate::context::replay(&self.store.events, &self.store.blobs).await?;
        let (definitions, bindings, warnings) = self.mcp.snapshot();
        for text in warnings {
            self.record_item(
                Some(turn),
                Item::Notice {
                    level: Level::Warn,
                    text,
                },
            )
            .await?;
        }
        let tools = self.tools.with_mcp(definitions, bindings);
        let ctx = turn_loop::Context {
            turn,
            generation,
            current: self.generation.clone(),
            cancel: cancel.clone(),
            provider: self.config.provider.clone(),
            model: self.config.model.clone(),
            messages,
            tools,
            cwd: self.config.meta.cwd.clone(),
            blobs: self.store.blobs.clone(),
            redactor: self.config.redactor.clone(),
            output: self.output.clone(),
            max_rounds: self.config.max_rounds,
        };
        self.running = Some(Running {
            turn,
            initial_cause,
            generation,
            cancel,
            steer,
            pending: VecDeque::new(),
            cause: None,
        });
        self.tasks.spawn_on(
            async move {
                turn_loop::TurnRunner::new(ctx, blocks, rx).run().await;
                (turn, generation)
            },
            bbt_rt::handle(),
        );
        Ok(())
    }
    async fn queued(&mut self, blocks: Vec<InputBlock>, cause: OpId) -> std::io::Result<()> {
        let turn = TurnId::new();
        self.publish(Event::TurnQueued { turn }, Some(cause))
            .await?;
        self.queue.push_back(Queued::Turn(turn, blocks, cause));
        Ok(())
    }
    async fn op(&mut self, admitted: crate::gateway::Admitted) -> std::io::Result<()> {
        let crate::gateway::Admitted { env, face } = admitted;
        let id = env.op_id;
        if self.tools.is_trial()
            && match &env.op {
                Op::SetProfile { .. } | Op::Compact => true,
                Op::ResolveApproval {
                    decision: ApprovalDecision::AllowAlways,
                    ..
                } => true,
                Op::UserTurn { blocks, mode } => {
                    !matches!(mode, TurnMode::Queue)
                        || self.running.is_some()
                        || !self.queue.is_empty()
                        || blocks.iter().any(|b| !matches!(b, InputBlock::Text { .. }))
                }
                _ => false,
            }
        {
            return self
                .reject(
                    id,
                    Rejection::InvalidInput {
                        detail: "trial context/profile widening refused".into(),
                    },
                )
                .await;
        }
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
            Op::UserTurn { blocks, mode } => {
                let blocks = match self.store.blobs.admit_images(blocks).await {
                    Ok(blocks) => blocks,
                    Err(error) => {
                        return self
                            .reject(
                                id,
                                Rejection::InvalidInput {
                                    detail: error.to_string(),
                                },
                            )
                            .await;
                    }
                };
                if blocks.is_empty() || blocks.iter().any(|b| matches!(b, InputBlock::Unknown(_))) {
                    return self
                        .reject(
                            id,
                            Rejection::InvalidInput {
                                detail: "provide supported input blocks".into(),
                            },
                        )
                        .await;
                }
                if matches!(mode, TurnMode::Steer) && self.running.is_some() {
                    let r = self.running.as_mut().unwrap();
                    if r.steer.try_send((id, blocks.clone())).is_ok() {
                        r.pending.push_back((id, blocks));
                    } else {
                        self.reject(
                            id,
                            Rejection::InvalidInput {
                                detail: "steer queue is full".into(),
                            },
                        )
                        .await?;
                    }
                } else if self.queue.len() >= 64 {
                    self.reject(
                        id,
                        Rejection::InvalidInput {
                            detail: "turn queue is full".into(),
                        },
                    )
                    .await?;
                } else {
                    self.queued(blocks, id).await?;
                }
            }
            Op::Interrupt { turn } => {
                let target = turn.or_else(|| self.running.as_ref().map(|r| r.turn));
                let Some(target) = target else {
                    return self.reject(id, Rejection::NoSuchTurn).await;
                };
                if let Some(into) = self.interrupted.get(&target) {
                    return self.reject(id, Rejection::Coalesced { into: *into }).await;
                }
                if let Some(r) = self.running.as_mut().filter(|r| r.turn == target) {
                    self.interrupted.insert(target, id);
                    r.cause = Some(id);
                    r.cancel.cancel();
                    self.cancel_approvals(target).await?;
                } else if let Some(p) = self
                    .queue
                    .iter()
                    .position(|w| matches!(w,Queued::Turn(t,_,_) if *t==target))
                {
                    self.queue.remove(p);
                    self.interrupted.insert(target, id);
                    self.publish(
                        Event::TurnEnded {
                            turn: target,
                            reason: StopReason::Cancelled,
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
                if let Some(p) = self.approvals.remove(&approval) {
                    if self
                        .running
                        .as_ref()
                        .is_none_or(|r| r.generation != p.generation || r.cancel.is_cancelled())
                    {
                        self.resolve(
                            approval,
                            p,
                            ApprovalDecision::Deny {
                                reason: Some("cancelled".into()),
                            },
                            Resolver::Cancelled,
                            None,
                        )
                        .await?;
                        self.reject(id, Rejection::AlreadyResolved).await?;
                    } else {
                        self.resolve(
                            approval,
                            p,
                            decision,
                            Resolver::Face { name: face.into() },
                            Some(id),
                        )
                        .await?;
                    }
                } else {
                    self.reject(
                        id,
                        if self.resolved.contains_key(&approval) {
                            Rejection::AlreadyResolved
                        } else {
                            Rejection::NoSuchApproval
                        },
                    )
                    .await?;
                }
            }
            Op::Close => {
                if let Some(into) = self.stopping {
                    self.reject(id, Rejection::Coalesced { into }).await?;
                } else {
                    self.stopping = Some(id);
                    self.services.cancel.cancel();
                    if let Some(op) = self.profile_pending.take() {
                        self.reject(
                            op,
                            Rejection::ProfileInvalid {
                                detail: "profile resolution cancelled by Close".into(),
                            },
                        )
                        .await?;
                    }
                    if let Some(r) = self.running.as_mut() {
                        r.cause = Some(id);
                        r.cancel.cancel();
                        let turn = r.turn;
                        self.cancel_approvals(turn).await?;
                    }
                }
            }
            Op::SetProfile { profile } => self.set_profile(profile, id).await?,
            Op::Compact => self.queue.push_back(Queued::Compact(id)),
            _ => {
                self.reject(
                    id,
                    Rejection::InvalidInput {
                        detail: "unsupported operation".into(),
                    },
                )
                .await?
            }
        }
        Ok(())
    }
    async fn internal(&mut self, msg: Internal) -> std::io::Result<()> {
        match msg {
            Internal::ServiceNotice { level, text } => {
                self.record_item(None, Item::Notice { level, text }).await?
            }
            Internal::PrepareRound {
                generation,
                turn,
                tools,
                reply,
            } => {
                if self
                    .running
                    .as_ref()
                    .is_some_and(|r| r.generation == generation && !r.cancel.is_cancelled())
                {
                    // Store/blob failures are fatal, as in append and manual compaction.
                    // Recovery closes a ladder item whose blob could not be committed.
                    let request = self.prepare_round(turn, &tools).await?;
                    let _ = reply.send(Ok(request));
                }
            }

            Internal::ProfileReady { op, result } => {
                if self.profile_pending != Some(op) || self.stopping.is_some() {
                    return Ok(());
                }
                self.profile_pending = None;
                match result {
                    Err(reason) => self.reject(op, reason).await?,
                    Ok((profile, provider)) => {
                        self.auto = crate::context::auto::Options {
                            enabled: profile.auto_compact,
                            window: profile.context_tokens,
                        };

                        self.config.profile_error = None;
                        self.config.model = profile.model.clone();
                        self.config.policy = if self.launch_yolo {
                            Policy::allow_all(profile.on_unattended)
                        } else {
                            profile.policy()
                        };
                        self.gateway
                            .set_unattended(self.config.policy.on_unattended);
                        self.config.provider = provider;
                        self.publish(
                            Event::ProfileChanged {
                                profile: profile.name,
                                model: profile.model,
                            },
                            Some(op),
                        )
                        .await?;
                    }
                }
            }
            Internal::Event {
                generation,
                event,
                mut cause,
            } => {
                if self
                    .running
                    .as_ref()
                    .is_none_or(|r| r.generation != generation)
                {
                    return Ok(());
                }
                // Initial input keeps its immutable admitted cause even when an
                // interrupt races its first item. Steer events carry their own cause.
                if cause.is_none()
                    && matches!(
                        event,
                        Event::ItemStarted {
                            kind: ItemKind::UserMessage,
                            ..
                        }
                    )
                {
                    cause = self.running.as_ref().map(|r| r.initial_cause);
                }
                if let Event::ItemStarted {
                    kind: ItemKind::UserMessage,
                    ..
                } = event
                    && let Some(op) = cause
                {
                    self.running
                        .as_mut()
                        .unwrap()
                        .pending
                        .retain(|(id, _)| *id != op);
                }
                if let Event::TurnEnded {
                    turn,
                    mut reason,
                    usage,
                } = event
                {
                    let r = self.running.take().unwrap();
                    if r.cancel.is_cancelled() {
                        reason = StopReason::Cancelled;
                        cause = r.cause;
                    }
                    self.cancel_approvals(turn).await?;
                    self.publish(
                        Event::TurnEnded {
                            turn,
                            reason,
                            usage,
                        },
                        cause,
                    )
                    .await?;
                    for (op, blocks) in r.pending {
                        self.queued(blocks, op).await?;
                    }
                } else {
                    self.publish(event, cause).await?;
                }
            }
            Internal::Approval {
                generation,
                turn,
                item,
                tool,
                effect,
                body,
                reply,
            } => {
                if self
                    .running
                    .as_ref()
                    .is_none_or(|r| r.generation != generation || r.cancel.is_cancelled())
                {
                    let _ = reply.send(Err("cancelled".into()));
                    return Ok(());
                }
                let decision = self.config.policy.decide(&Action {
                    tool: tool.clone(),
                    effect: effect.clone(),
                    generation,
                });
                if effect == EffectClass::ReadOnly {
                    match &decision {
                        Decision::Allow { .. } => {
                            let _ = reply.send(Ok(Permit::approval(tool, generation)));
                            return Ok(());
                        }
                        Decision::Deny { reason } => {
                            let _ = reply.send(Err(reason.clone()));
                            return Ok(());
                        }
                        Decision::Ask => {}
                    }
                }
                self.publish(
                    Event::ItemStarted {
                        turn: Some(turn),
                        item,
                        kind: ItemKind::Approval,
                        parent: match &body {
                            Item::Approval { subject, .. } => Some(subject.tool_call),
                            _ => None,
                        },
                        head: None,
                    },
                    None,
                )
                .await?;
                if matches!(decision, Decision::Ask) {
                    self.publish(
                        Event::ItemSnapshot {
                            item,
                            partial: body.clone(),
                        },
                        None,
                    )
                    .await?;
                }
                let p = Pending {
                    turn,
                    generation,
                    tool,
                    effect,
                    body,
                    reply,
                };
                match decision {
                    Decision::Allow { rule } => {
                        self.resolve(
                            item,
                            p,
                            ApprovalDecision::Allow,
                            Resolver::Rule { id: rule },
                            None,
                        )
                        .await?
                    }
                    Decision::Deny { reason } => {
                        self.resolve(
                            item,
                            p,
                            ApprovalDecision::Deny {
                                reason: Some(reason),
                            },
                            Resolver::Rule { id: "deny".into() },
                            None,
                        )
                        .await?
                    }
                    Decision::Ask if self.gateway.hub.approvers() == 0 => {
                        let decision =
                            if matches!(self.config.policy.on_unattended, Unattended::Allow) {
                                ApprovalDecision::Allow
                            } else {
                                ApprovalDecision::Deny {
                                    reason: Some("no approver attached; on_unattended=deny".into()),
                                }
                            };
                        self.resolve(item, p, decision, Resolver::Unattended, None)
                            .await?;
                    }
                    Decision::Ask => {
                        self.approvals.insert(item, p);
                    }
                }
            }
        }
        Ok(())
    }
    async fn record_item(&mut self, turn: Option<TurnId>, body: Item) -> std::io::Result<()> {
        let item = ItemId::new();
        self.publish(
            Event::ItemStarted {
                turn,
                item,
                kind: body.kind(),
                parent: None,
                head: body.head(),
            },
            None,
        )
        .await?;
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body,
            },
            None,
        )
        .await
    }
    async fn request_context(&self) -> std::io::Result<(Vec<serde_json::Value>, usize)> {
        let (tail, prefix) = crate::context::replay(&self.store.events, &self.store.blobs).await?;
        let mut system = format!(
            "You are bboxy450-harness, a coding assistant. Use the provided tools. Working directory: {}",
            self.config.meta.cwd.display()
        );
        if !self.append_system_prompt.is_empty() {
            system.push('\n');
            system.push_str(&self.append_system_prompt);
        }
        let mut messages = vec![serde_json::json!({"role":"system","content":system})];
        messages.extend(tail);
        Ok((messages, prefix))
    }
    async fn prepare_round(
        &mut self,
        turn: TurnId,
        tools: &[crate::provider::ToolDef],
    ) -> std::io::Result<(Vec<serde_json::Value>, usize)> {
        use crate::context::{self, auto};
        let mut request = self.request_context().await?;
        if !self.auto.enabled {
            return Ok(request);
        }
        let window = self
            .auto
            .window
            .unwrap_or(self.config.provider.caps().context_tokens);
        if window == 0 {
            if !self.window_warned {
                self.window_warned = true;
                self.record_item(
                    Some(turn),
                    Item::Notice {
                        level: Level::Warn,
                        text: "context window unknown; automatic compaction off".into(),
                    },
                )
                .await?;
            }
            return Ok(request);
        }
        let max_tokens = window / 8;
        if !auto::trigger(
            auto::estimate(&self.store.events, &request.0, tools),
            max_tokens,
            window,
        ) {
            return Ok(request);
        }
        let r = auto::ended_through(&self.store.events);
        if r > context::clipped_through(&self.store.events) {
            self.record_item(Some(turn), Item::ContextTrim { clipped_through: r })
                .await?;
            request = self.request_context().await?;
            if auto::fits(
                auto::estimate(&self.store.events, &request.0, tools),
                max_tokens,
                window,
            ) {
                return Ok(request);
            }
        }
        if r > context::latest(&self.store.events)
            .map(|(_, r)| r)
            .unwrap_or(Seq(0))
        {
            self.summarize(Some(turn), None).await?;
            request = self.request_context().await?;
        }
        Ok(request)
    }
    async fn compact(&mut self, id: OpId) -> std::io::Result<()> {
        self.summarize(None, Some(id)).await
    }
    async fn summarize(
        &mut self,
        turn: Option<TurnId>,
        cause: Option<OpId>,
    ) -> std::io::Result<()> {
        let latest = crate::context::latest(&self.store.events);
        let r0 = latest.map(|(_, r)| r).unwrap_or(Seq(0));
        let r = crate::context::auto::ended_through(&self.store.events);
        let Some(replaced_through) = Some(r).filter(|r| *r > r0) else {
            return if let Some(id) = cause {
                self.reject(id, Rejection::NothingToCompact).await
            } else {
                Ok(())
            };
        };
        let prev = if let Some((blob, _)) = latest {
            Some(
                String::from_utf8(self.store.blobs.get(blob).await?)
                    .map_err(std::io::Error::other)?,
            )
        } else {
            None
        };
        let window = self
            .store
            .events
            .iter()
            .filter(|e| e.seq.is_some_and(|s| s > r0 && s <= replaced_through))
            .cloned()
            .collect::<Vec<_>>();
        let digest = crate::runtime::blocking(move || {
            Ok::<_, std::io::Error>(crate::context::compact::compact_summary(
                prev.as_deref(),
                &window,
            ))
        })
        .await?;
        let item = ItemId::new();
        self.publish(
            Event::ItemStarted {
                turn,
                item,
                kind: ItemKind::Compaction,
                parent: None,
                head: None,
            },
            cause,
        )
        .await?;
        let summary = self
            .store
            .blobs
            .put(digest.into_bytes(), "text/markdown")
            .await?;
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::Compaction {
                    summary,
                    replaced_through,
                },
            },
            cause,
        )
        .await
    }
    async fn set_profile(&mut self, name: String, id: OpId) -> std::io::Result<()> {
        if self.running.is_some() {
            return self.reject(id, Rejection::NotIdle).await;
        }
        self.profile_pending = Some(id);
        let root = self.config.profiles.clone();
        let redactor = self.config.redactor.clone();
        let output = self.output.clone();
        let cancel = self.services.cancel.child_token();
        self.services.spawn(async move{
            let resolve=std::panic::AssertUnwindSafe(async{
                let path=Profile::file_in(&root,&name).map_err(|e|Rejection::ProfileInvalid{detail:e.message})?;
                let text=tokio::select!{biased;_=cancel.cancelled()=>return Err(Rejection::ProfileInvalid{detail:"cancelled".into()}),text=tokio::fs::read_to_string(path)=>text.map_err(|_|Rejection::UnknownProfile{name:name.clone()})?};
                let mut profile=Profile::parse(&text).map_err(|e|Rejection::ProfileInvalid{detail:e.message})?;
                profile.name=name;
                let key=profile.key(&redactor,cancel.clone()).await.map_err(|e|Rejection::ProfileInvalid{detail:e.message})?;
                let options=crate::provider::Options{retry_deadline:profile.retry_deadline.map(std::time::Duration::from_secs),..Default::default()};
                let provider=profile.build_provider(key,redactor,options).map_err(|e|Rejection::ProfileInvalid{detail:e.message})?;
                Ok((profile,provider))
            }).catch_unwind().await.unwrap_or_else(|_|Err(Rejection::ProfileInvalid{detail:"profile worker panicked".into()}));
            let _=task_group::send(&output,Internal::ProfileReady{op:id,result:resolve},&cancel).await;
        });
        Ok(())
    }
    async fn resolve(
        &mut self,
        item: ItemId,
        mut p: Pending,
        decision: ApprovalDecision,
        by: Resolver,
        cause: Option<OpId>,
    ) -> std::io::Result<()> {
        let allowed = matches!(
            decision,
            ApprovalDecision::Allow | ApprovalDecision::AllowAlways
        ) && self
            .running
            .as_ref()
            .is_some_and(|r| r.generation == p.generation && !r.cancel.is_cancelled());
        if matches!(decision, ApprovalDecision::AllowAlways) && allowed {
            self.config.policy.always_allow(&p.tool, p.effect.clone());
        }
        let denied = match &decision {
            ApprovalDecision::Deny { reason } => {
                reason.clone().unwrap_or_else(|| "tool denied".into())
            }
            _ => "cancelled".into(),
        };
        if let Item::Approval {
            decision: d, by: b, ..
        } = &mut p.body
        {
            *d = Some(decision);
            *b = Some(by);
        }
        self.publish(
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: p.body,
            },
            cause,
        )
        .await?;
        self.resolved.insert(item, ());
        let _ = p.reply.send(if allowed {
            Ok(Permit::approval(p.tool, p.generation))
        } else {
            Err(denied)
        });
        Ok(())
    }
    async fn cancel_approvals(&mut self, turn: TurnId) -> std::io::Result<()> {
        let ids = self
            .approvals
            .iter()
            .filter_map(|(id, p)| (p.turn == turn).then_some(*id))
            .collect::<Vec<_>>();
        for id in ids {
            let p = self.approvals.remove(&id).unwrap();
            self.resolve(
                id,
                p,
                ApprovalDecision::Deny {
                    reason: Some("turn cancelled".into()),
                },
                Resolver::Cancelled,
                None,
            )
            .await?;
        }
        Ok(())
    }
    async fn crashed(&mut self, message: &str) -> std::io::Result<()> {
        if let Some(r) = self.running.take() {
            r.cancel.cancel();
            let state = self.gateway.hub.projection();
            let mut open = state.open.clone();
            while !open.is_empty() {
                let id = *open
                    .keys()
                    .find(|id| !open.values().any(|(_, p, _)| p.as_ref() == Some(id)))
                    .unwrap();
                let (_, _, body) = open.remove(&id).unwrap();
                self.publish(
                    Event::ItemCompleted {
                        item: id,
                        status: ItemStatus::Interrupted,
                        body: Item::interrupted_from_head(
                            body.kind(),
                            id,
                            self.store.events.iter().rev().find_map(|e| match &e.event {
                                Event::ItemStarted { item, head, .. } if *item == id => {
                                    head.as_ref()
                                }
                                _ => None,
                            }),
                        ),
                    },
                    None,
                )
                .await?;
            }
            self.publish(
                Event::TurnEnded {
                    turn: r.turn,
                    reason: StopReason::Error {
                        class: ErrorClass::Internal,
                        message: message.into(),
                    },
                    usage: None,
                },
                None,
            )
            .await?;
        }
        Ok(())
    }
}
pub async fn stop(gateway: &InProc) {
    let _ = gateway.send(OpEnvelope {
        v: PROTOCOL,
        session: gateway.hello().meta.session,
        op_id: OpId::new(),
        op: Op::Close,
    });
}

#[cfg(test)]
mod hardening_tests {
    use super::*;
    use crate::provider::{self, Caps, ModelEvent, ModelRequest};
    use std::{sync::Mutex, time::Duration};
    struct Recording {
        inner: provider::ScriptedProvider,
        requests: Arc<Mutex<Vec<ModelRequest>>>,
    }
    impl Provider for Recording {
        fn caps(&self) -> Caps {
            self.inner.caps()
        }
        fn stream(
            &self,
            r: ModelRequest,
            c: CancellationToken,
        ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
            self.requests.lock().unwrap().push(r.clone());
            self.inner.stream(r, c)
        }
    }
    fn envelope(g: &InProc, op: Op) -> OpEnvelope {
        OpEnvelope {
            v: PROTOCOL,
            session: g.session(),
            op_id: OpId::new(),
            op,
        }
    }
    async fn event(sub: &mut crate::gateway::Subscription) -> Envelope {
        tokio::time::timeout(Duration::from_secs(5), sub.recv())
            .await
            .unwrap()
            .unwrap()
    }
    #[tokio::test]
    async fn launch_context_is_in_the_first_scripted_request_stable_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let session = SessionId::new();
        let requests = Arc::new(Mutex::new(vec![]));
        let provider = Recording {
            inner: provider::ScriptedProvider::new(vec![
                include_bytes!("../../tests/fixtures/sse/text.sse").to_vec(),
            ]),
            requests: requests.clone(),
        };
        let config = Config {
            meta: SessionMeta {
                vcs: None,
                dream: None,
                source: None,
                session,
                cwd: temp.path().into(),
                profile: None,
                fork: None,
                protocol: PROTOCOL,
                harness: "test".into(),
                created_ms: 0,
            },
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(provider),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        };
        let (gateway, task) = SessionActor::load_with_mcp(
            config,
            OpId::new(),
            None,
            Default::default(),
            LaunchOptions {
                append_system_prompt: "NAME: test seat\nROLE: EXECUTOR".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let mut sub = gateway
            .subscribe(crate::gateway::Subscribe {
                session,
                after: None,
                ephemeral: false,
            })
            .unwrap();
        gateway
            .send(envelope(
                &gateway,
                Op::UserTurn {
                    blocks: vec![InputBlock::Text {
                        text: "hello".into(),
                    }],
                    mode: TurnMode::Queue,
                },
            ))
            .unwrap();
        loop {
            if matches!(event(&mut sub).await.event, Event::TurnEnded { .. }) {
                break;
            }
        }
        gateway.send(envelope(&gateway, Op::Close)).unwrap();
        task.await.unwrap().unwrap();
        let requests = requests.lock().unwrap();
        let request = requests.first().unwrap();
        assert!(request.stable_prefix > 0);
        assert!(
            request.messages[..request.stable_prefix]
                .iter()
                .any(|m| m["role"] == "system"
                    && m["content"]
                        .as_str()
                        .is_some_and(|s| s.ends_with("NAME: test seat\nROLE: EXECUTOR"))),
            "appended startup text absent from stable system prefix"
        );
    }
    #[tokio::test]
    async fn crashed_closes_from_the_durable_head_not_runtime_identity() {
        let temp = tempfile::tempdir().unwrap();
        let session = SessionId::new();
        let config = Config {
            meta: SessionMeta {
                vcs: None,
                dream: None,
                source: None,
                session,
                cwd: temp.path().into(),
                profile: Some("scripted".into()),
                fork: None,
                protocol: PROTOCOL,
                harness: env!("CARGO_PKG_VERSION").into(),
                created_ms: 0,
            },
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(provider::ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 1,
            profile_error: None,
        };
        let mut actor = SessionActor::new(
            config,
            OpId::new(),
            None,
            Default::default(),
            Default::default(),
        )
        .await
        .unwrap();
        let turn = TurnId::new();
        let item = ItemId::new();
        let head = ItemHead::ToolCall {
            call_id: "known-call".into(),
            name: "read".into(),
            tool_kind: ToolKind::Read,
        };
        actor
            .publish(Event::TurnQueued { turn }, None)
            .await
            .unwrap();
        actor
            .publish(
                Event::TurnStarted {
                    turn,
                    model: None,
                    origin: None,
                },
                None,
            )
            .await
            .unwrap();
        actor
            .publish(
                Event::ItemStarted {
                    turn: Some(turn),
                    item,
                    kind: ItemKind::ToolCall,
                    parent: None,
                    head: Some(head.clone()),
                },
                None,
            )
            .await
            .unwrap();
        let (steer, _) = mpsc::channel(1);
        actor.running = Some(Running {
            turn,
            initial_cause: OpId::new(),
            generation: 1,
            cancel: CancellationToken::new(),
            steer,
            pending: VecDeque::new(),
            cause: None,
        });
        actor.crashed("injected runner failure").await.unwrap();
        let body = actor
            .store
            .events
            .iter()
            .find_map(|e| {
                if let Event::ItemCompleted { item: i, body, .. } = &e.event {
                    (*i == item).then_some(body)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(body.head(), Some(head));
        if let Item::ToolCall {
            input_raw, output, ..
        } = body
        {
            assert!(input_raw.is_empty());
            assert!(
                matches!(&output[0],OutputBlock::Error{text} if text=="tool call was not completed")
            );
        } else {
            panic!("wrong recovery kind");
        }
        laws::check_transcript(&actor.store.events).unwrap();
    }
    #[tokio::test]
    async fn a_b_a_is_fenced_by_birth_not_identity() {
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let session = SessionId::new();
        let requests = Arc::new(Mutex::new(vec![]));
        let mut provider = provider::ScriptedProvider::new(vec![
            include_bytes!("../../tests/fixtures/sse/tools.sse").to_vec(),
            include_bytes!("../../tests/fixtures/sse/text.sse").to_vec(),
            include_bytes!("../../tests/fixtures/sse/text.sse").to_vec(),
        ]);
        provider.delay = Duration::from_millis(1);
        let config = Config {
            meta: SessionMeta {
                vcs: None,
                dream: None,
                source: None,
                session,
                cwd: temp.path().into(),
                profile: Some("scripted".into()),
                fork: None,
                protocol: PROTOCOL,
                harness: env!("CARGO_PKG_VERSION").into(),
                created_ms: 0,
            },
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(Recording {
                inner: provider,
                requests: requests.clone(),
            }),
            model: "scripted".into(),
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 50,
            profile_error: None,
        };
        let actor = SessionActor::new(
            config,
            OpId::new(),
            None,
            Default::default(),
            Default::default(),
        )
        .await
        .unwrap();
        let output = actor.output.clone();
        let generation = actor.generation.clone();
        let g = actor.gateway.clone();
        let task = bbt_rt::handle().spawn(actor.run());
        let mut sub = g
            .subscribe(crate::gateway::Subscribe {
                session,
                after: None,
                ephemeral: false,
            })
            .unwrap();
        g.send(envelope(
            &g,
            Op::UserTurn {
                blocks: vec![InputBlock::Text { text: "A".into() }],
                mode: TurnMode::Queue,
            },
        ))
        .unwrap();
        let (item, head) = loop {
            let e = event(&mut sub).await;
            if let Event::ItemStarted {
                item,
                kind: ItemKind::ToolCall,
                head,
                ..
            } = e.event
            {
                break (item, head);
            }
        };
        let old_generation = generation.load(Ordering::Acquire);
        let (release, held) = oneshot::channel();
        let held_tool = bbt_rt::handle().spawn(async move {
            held.await.unwrap();
            let mut body = Item::interrupted_from_head(ItemKind::ToolCall, item, head.as_ref());
            if let Item::ToolCall { output, .. } = &mut body {
                *output = vec![OutputBlock::Text {
                    text: "LATE_FROM_A".into(),
                }];
            }
            output
                .send(Internal::Event {
                    generation: old_generation,
                    event: Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body,
                    },
                    cause: None,
                })
                .await
                .unwrap();
        });
        g.send(envelope(&g, Op::Interrupt { turn: None })).unwrap();
        while !matches!(
            event(&mut sub).await.event,
            Event::TurnEnded {
                reason: StopReason::Cancelled,
                ..
            }
        ) {}
        g.send(envelope(
            &g,
            Op::UserTurn {
                blocks: vec![InputBlock::Text { text: "B".into() }],
                mode: TurnMode::Queue,
            },
        ))
        .unwrap();
        let b = loop {
            if let Event::TurnStarted { turn, .. } = event(&mut sub).await.event {
                break turn;
            }
        };
        release.send(()).unwrap();
        held_tool.await.unwrap();
        let mut b_events = vec![];
        loop {
            let e = event(&mut sub).await;
            let done = matches!(e.event,Event::TurnEnded{turn,..} if turn==b);
            b_events.push(e);
            if done {
                break;
            }
        }
        assert!(
            !serde_json::to_string(&b_events)
                .unwrap()
                .contains("LATE_FROM_A")
        );
        g.send(envelope(
            &g,
            Op::UserTurn {
                blocks: vec![InputBlock::Text { text: "A".into() }],
                mode: TurnMode::Queue,
            },
        ))
        .unwrap();
        while !matches!(event(&mut sub).await.event, Event::TurnEnded { .. }) {}
        assert_eq!(generation.load(Ordering::Acquire), 3);
        assert!(!format!("{:?}", requests.lock().unwrap()).contains("LATE_FROM_A"));
        g.send(envelope(&g, Op::Close)).unwrap();
        task.await.unwrap().unwrap();
        let log = std::fs::read_to_string(
            temp.path()
                .join("sessions")
                .join(format!("{session}.jsonl")),
        )
        .unwrap();
        assert!(!log.contains("LATE_FROM_A"));
        let events = log
            .lines()
            .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
            .collect::<Vec<_>>();
        laws::check_transcript(&events).unwrap();
    }
}

#[cfg(test)]
mod trial_launch_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[tokio::test]
    async fn trial_launch_prevents_all_mcp_origins_and_context_widening() {
        let home = std::env::var_os("HOME");
        let temp = tempfile::tempdir().unwrap();
        for name in ["repo", "sessions", "profiles", "home"] {
            std::fs::create_dir(temp.path().join(name)).unwrap();
            std::fs::set_permissions(
                temp.path().join(name),
                std::fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let root = temp.path().to_path_buf();
        let capability = crate::runtime::blocking(move || {
            crate::tools::TrialCapability::mint(
                root.join("repo"),
                root.join("sessions"),
                root.join("profiles"),
                root.join("home"),
            )
        })
        .await
        .unwrap();
        let mut meta = SessionMeta {
            session: SessionId::new(),
            cwd: temp.path().join("repo"),
            profile: Some("scripted".into()),
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: 0,
            vcs: Some(Vcs::default()),
            dream: None,
            source: None,
        };
        let config = Config {
            meta: meta.clone(),
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(crate::provider::ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: Policy::allow_all(crate::provider::profile::Unattended::Allow),
            redactor: Set::default(),
            max_rounds: 2,
            profile_error: None,
        };
        let sentinel = temp.path().join("mcp-must-not-start");
        let servers:crate::mcp::Servers=serde_json::from_value(serde_json::json!({"trap":{"command":"/bin/sh","args":["-c",format!("touch '{}'",sentinel.display())]}})).unwrap();
        let profile = serde_json::json!({"name":"scripted","model":"scripted","endpoint":"http://127.0.0.1:1/v1","auto_compact":true});
        for origin in 0..4 {
            let mut launch = LaunchOptions {
                trial: Some(capability.clone()),
                ..Default::default()
            };
            let mut overrides = Default::default();
            let mut p = profile.clone();
            match origin {
                0 => p["mcpServers"] = serde_json::to_value(&servers).unwrap(),
                1 => launch.mcp_servers = servers.clone(),
                2 => overrides = servers.clone(),
                3 => launch.yolo = true,
                _ => unreachable!(),
            }
            std::fs::write(
                config.profiles.join("scripted.md"),
                format!("---\n{}---\n", serde_yaml::to_string(&p).unwrap()),
            )
            .unwrap();
            assert!(
                SessionActor::load_with_mcp(config.clone(), OpId::new(), None, overrides, launch)
                    .await
                    .is_err()
            );
            assert!(!sentinel.exists());
            assert!(
                !crate::store::log_path(&config.sessions, meta.session).exists(),
                "refused before Store effects"
            );
        }
        std::fs::write(
            config.profiles.join("scripted.md"),
            format!("---\n{}---\n", serde_yaml::to_string(&profile).unwrap()),
        )
        .unwrap();
        let actor = SessionActor::new(
            config.clone(),
            OpId::new(),
            None,
            Default::default(),
            LaunchOptions {
                trial: Some(capability.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!actor.auto.enabled);
        assert!(actor.tools.is_trial());
        assert_eq!(actor.tools.definitions().len(), 2);
        let gateway = actor.gateway.clone();
        // Public gateway admissions and actual actor dispatch, rather than a
        // profile-name assertion, exercise post-start pinning.
        let task = bbt_rt::handle().spawn(async move { actor.run().await });
        let hello = gateway.hello();
        let mut sub = gateway
            .subscribe(crate::gateway::Subscribe {
                session: hello.meta.session,
                after: Some((hello.epoch, hello.head)),
                ephemeral: false,
            })
            .unwrap();
        for op in [
            Op::SetProfile {
                profile: "operator".into(),
            },
            Op::Compact,
            Op::UserTurn {
                blocks: vec![InputBlock::Text {
                    text: "steer".into(),
                }],
                mode: TurnMode::Steer,
            },
            Op::UserTurn {
                blocks: vec![InputBlock::Mention {
                    path: temp.path().join("outside"),
                }],
                mode: TurnMode::Queue,
            },
            Op::ResolveApproval {
                approval: ItemId::new(),
                decision: ApprovalDecision::AllowAlways,
            },
        ] {
            gateway
                .send(OpEnvelope {
                    v: PROTOCOL,
                    session: hello.meta.session,
                    op_id: OpId::new(),
                    op,
                })
                .unwrap();
            let event = tokio::time::timeout(std::time::Duration::from_secs(3), sub.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(matches!(event.event, Event::OpRejected { .. }));
        }
        stop(&gateway).await;
        task.await.unwrap().unwrap();
        capability.finish().await;
        assert!(!sentinel.exists());
        assert_eq!(home, std::env::var_os("HOME"));
        // Untrusted session/profile JSON cannot supply the opaque Rust capability.
        meta.session = SessionId::new();
        let mut forged = serde_json::to_value(meta).unwrap();
        forged["trial"] = serde_json::json!(true);
        let parsed: SessionMeta = serde_json::from_value(forged).unwrap();
        assert_eq!(parsed.harness, env!("CARGO_PKG_VERSION"));
        assert!(LaunchOptions::default().trial.is_none());
    }
}
