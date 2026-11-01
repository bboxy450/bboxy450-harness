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
pub(crate) enum Internal {
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
        reply: oneshot::Sender<Option<Permit>>,
    },
}
struct Pending {
    turn: TurnId,
    generation: u64,
    tool: String,
    effect: EffectClass,
    body: Item,
    reply: oneshot::Sender<Option<Permit>>,
}
enum Queued {
    Turn(TurnId, Vec<InputBlock>),
    Compact(OpId),
}
struct Running {
    turn: TurnId,
    generation: u64,
    cancel: CancellationToken,
    steer: mpsc::Sender<(OpId, Vec<InputBlock>)>,
    pending: VecDeque<(OpId, Vec<InputBlock>)>,
    cause: Option<OpId>,
}
pub struct SessionActor {
    store: Store,
    gateway: InProc,
    work: mpsc::Receiver<OpEnvelope>,
    control: mpsc::Receiver<OpEnvelope>,
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
}
impl SessionActor {
    pub async fn open(
        config: Config,
        start: OpId,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        Self::load(config, start, None).await
    }
    pub(crate) async fn load(
        mut config: Config,
        start: OpId,
        notice: Option<String>,
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
            if let Some(message) = &config.profile_error {
                let item = ItemId::new();
                store
                    .append(
                        Event::ItemStarted {
                            turn: None,
                            item,
                            kind: ItemKind::Notice,
                            parent: None,
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
        if let Some(text) = notice {
            let item = ItemId::new();
            store
                .append(
                    Event::ItemStarted {
                        turn: None,
                        item,
                        kind: ItemKind::Notice,
                        parent: None,
                    },
                    None,
                )
                .await?;
            store
                .append(
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body: Item::Notice {
                            level: Level::Error,
                            text,
                        },
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
        let gateway = InProc::new(hub, work_tx, control_tx, start);
        let actor = Self {
            store,
            gateway: gateway.clone(),
            work,
            control,
            config,
            tools: Registry::new(),
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
            services: task_group::TaskGroup::new(CancellationToken::new()),
            profile_pending: None,
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
       while let Some(env)=self.work.recv().await {self.reject(env.op_id,Rejection::InvalidInput{detail:"session is closing".into()}).await?;}
       while let Some(work)=self.queue.pop_front(){match work {Queued::Turn(turn,_)=>self.publish(Event::TurnEnded{turn,reason:StopReason::Cancelled,usage:None},Some(op)).await?,Queued::Compact(id)=>self.reject(id,Rejection::NothingToCompact).await?}}
       self.publish(Event::SessionClosed,Some(op)).await?;break;
     }
     if let Some(work)=self.queue.pop_front(){match work{Queued::Turn(turn,blocks)=>self.start(turn,blocks).await?,Queued::Compact(id)=>{self.compact(id).await?;continue;}}}
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
    async fn start(&mut self, turn: TurnId, blocks: Vec<InputBlock>) -> std::io::Result<()> {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let cancel = CancellationToken::new();
        let (steer, rx) = mpsc::channel(64);
        self.publish(Event::TurnStarted { turn }, None).await?;
        let (messages, stable_prefix) =
            crate::context::replay(&self.store.events, &self.store.blobs).await?;
        let ctx = turn_loop::Context {
            turn,
            generation,
            current: self.generation.clone(),
            cancel: cancel.clone(),
            provider: self.config.provider.clone(),
            model: self.config.model.clone(),
            messages,
            stable_prefix,
            tools: self.tools.clone(),
            cwd: self.config.meta.cwd.clone(),
            blobs: self.store.blobs.clone(),
            redactor: self.config.redactor.clone(),
            output: self.output.clone(),
            max_rounds: self.config.max_rounds,
        };
        self.running = Some(Running {
            turn,
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
        self.queue.push_back(Queued::Turn(turn, blocks));
        Ok(())
    }
    async fn op(&mut self, env: OpEnvelope) -> std::io::Result<()> {
        let id = env.op_id;
        match env.op {
            Op::UserTurn { blocks, mode } => {
                if blocks.is_empty() || blocks.iter().any(|b| matches!(b, InputBlock::Unknown)) {
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
                    .position(|w| matches!(w,Queued::Turn(t,_) if *t==target))
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
                            Resolver::Face {
                                name: "gateway".into(),
                            },
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
            Internal::ProfileReady { op, result } => {
                if self.profile_pending != Some(op) || self.stopping.is_some() {
                    return Ok(());
                }
                self.profile_pending = None;
                match result {
                    Err(reason) => self.reject(op, reason).await?,
                    Ok((profile, provider)) => {
                        self.config.profile_error = None;
                        self.config.model = profile.model.clone();
                        self.config.policy = profile.policy();
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
                    let _ = reply.send(None);
                    return Ok(());
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
                    },
                    None,
                )
                .await?;
                self.publish(
                    Event::ItemSnapshot {
                        item,
                        partial: body.clone(),
                    },
                    None,
                )
                .await?;
                let decision = self.config.policy.decide(&Action {
                    tool: tool.clone(),
                    effect: effect.clone(),
                    generation,
                });
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
    async fn compact(&mut self, id: OpId) -> std::io::Result<()> {
        let latest = crate::context::latest(&self.store.events);
        let r0 = latest.map(|(_, r)| r).unwrap_or(Seq(0));
        let r = self.store.events.iter().rev().find_map(|e| {
            matches!(e.event, Event::TurnEnded { .. })
                .then_some(e.seq)
                .flatten()
        });
        let Some(replaced_through) = r.filter(|r| *r > r0) else {
            return self.reject(id, Rejection::NothingToCompact).await;
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
                turn: None,
                item,
                kind: ItemKind::Compaction,
                parent: None,
            },
            Some(id),
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
            Some(id),
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
                let provider=crate::provider::OpenAiCompat::new(&profile.endpoint,key,redactor,options).map_err(|e|Rejection::ProfileInvalid{detail:e.message})?;
                Ok((profile,Arc::new(provider) as Arc<dyn Provider>))
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
        let _ = p
            .reply
            .send(allowed.then(|| Permit::approval(p.tool, p.generation)));
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
                        body: if body.kind() == ItemKind::ToolCall {
                            Item::interrupted(ItemKind::ToolCall, id)
                        } else {
                            body
                        },
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
