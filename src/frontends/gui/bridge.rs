// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Background-only loading/subscription, with replace-latest projection delivery.
#![deny(clippy::disallowed_methods)]
use super::model::{Effect, Transcript};
use crate::{
    engine::Config,
    gateway::{
        Ack, Gateway, Host, InProc, Refused, Subscribe, Subscription,
        unix::{ClientError, UnixClient},
    },
    protocol::*,
};
use bbt_rt::{DirtySet, Latest, UiMsg, UiReceiver, UiSender, Waker};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

/// Local presentation only: Ack is admission, never durable chat confirmation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPrompt {
    pub op: OpId,
    pub session: SessionId,
    pub epoch: Epoch,
    pub text: String,
    pub status: String,
    item: Option<ItemId>,
    turn: Option<TurnId>,
}
impl PendingPrompt {
    pub fn new(op: OpId, session: SessionId, epoch: Epoch, text: String) -> Self {
        Self {
            op,
            session,
            epoch,
            text,
            status: "Sending…".into(),
            item: None,
            turn: None,
        }
    }
    fn confirmed(&self, model: &Transcript) -> bool {
        Some(self.session) == model.session
            && self.epoch == model.epoch
            && self.item.is_some_and(|item| {
                model
                    .items
                    .get(&item)
                    .is_some_and(|row| row.completed.is_some() && row.kind == ItemKind::UserMessage)
            })
    }
    fn observe(&mut self, event: &Envelope) -> bool {
        if event.session != self.session || event.epoch != self.epoch || event.seq.is_none() {
            return false;
        }
        match &event.event {
            Event::TurnQueued { turn } if event.cause == Some(self.op) => {
                self.turn = Some(*turn);
                if self.status == "Sending…" {
                    self.status = "Queued…".into();
                }
            }
            Event::ItemStarted {
                item,
                kind: ItemKind::UserMessage,
                ..
            } if event.cause == Some(self.op) => self.item = Some(*item),
            Event::ItemCompleted {
                item,
                body: Item::UserMessage { .. },
                ..
            } if self.item == Some(*item) => return true,
            Event::TurnEnded { turn, .. } if self.turn == Some(*turn) && self.item.is_none() => {
                self.status = "Ended · no durable message was recorded".into();
            }
            Event::OpRejected { op, .. } if *op == self.op => {
                self.status = "Rejected · message was not sent".into()
            }
            _ => {}
        }
        false
    }
}

#[derive(Clone, Default)]
pub struct FrameState {
    pub revision: u64,
    pub pending_prompts: Arc<im::OrdMap<OpId, PendingPrompt>>,
    pub transcript: Arc<Transcript>,
    pub gateway: Option<ViewGateway>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub replay: bool,
    pub ready: bool,
    /// Profiles in the machine-global profiles directory, for the switcher.
    pub profiles: Arc<Vec<ProfileInfo>>,
    #[cfg(feature = "embedded")]
    pub prepared: Option<Arc<super::view::PreparedRows>>,
    #[cfg(feature = "embedded")]
    pub faces: Arc<super::rich::Faces>,
}

/// Local admission is synchronous. A Unix dispatch is explicitly pending until
/// its correlated server response arrives on the background worker.
#[derive(Debug)]
pub enum Submission {
    Admitted(Ack),
    Pending { op_id: OpId },
}
#[derive(Clone)]
pub enum ViewGateway {
    InProc(InProc),
    Unix(RemoteGateway),
}
#[derive(Clone)]
pub struct RemoteGateway {
    client: UnixClient,
    session: SessionId,
    epoch: Epoch,
    work: Arc<tokio::sync::Semaphore>,
    outcomes:
        tokio::sync::mpsc::UnboundedSender<(SessionId, Epoch, OpId, Result<Ack, ClientError>)>,
}
struct ConnectionGuard(Option<UnixClient>);
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(client) = &self.0 {
            client.close();
        }
    }
}
impl ViewGateway {
    fn session(&self) -> SessionId {
        match self {
            Self::InProc(g) => g.session(),
            Self::Unix(g) => g.session,
        }
    }
    fn send(&self, op: OpEnvelope) -> Result<Submission, Refused> {
        if op.session != self.session() {
            return Err(Refused::UnknownSession);
        }
        match self {
            Self::InProc(g) => g.send(op).map(Submission::Admitted),
            Self::Unix(g) => {
                // Bound in-flight work without making Stop/approval wait for it.
                let permit = if matches!(
                    op.op,
                    Op::Interrupt { .. } | Op::ResolveApproval { .. } | Op::Close
                ) {
                    None
                } else {
                    Some(
                        g.work
                            .clone()
                            .try_acquire_owned()
                            .map_err(|_| Refused::Busy)?,
                    )
                };
                let client = g.client.clone();
                let outcomes = g.outcomes.clone();
                let op_id = op.op_id;
                let session = g.session;
                let epoch = g.epoch;
                bbt_rt::handle().spawn(async move {
                    let result = client.send(op).await;
                    let _ = outcomes.send((session, epoch, op_id, result));
                    drop(permit);
                });
                Ok(Submission::Pending { op_id })
            }
        }
    }
}
enum Source {
    InProc(InProc),
    Unix { socket: PathBuf },
}
impl Source {
    async fn subscribe(
        &self,
        after: Option<(Epoch, Seq)>,
        approver: bool,
        outcomes: tokio::sync::mpsc::UnboundedSender<(
            SessionId,
            Epoch,
            OpId,
            Result<Ack, ClientError>,
        )>,
    ) -> Result<(ViewGateway, Subscription), String> {
        match self {
            Self::InProc(g) => g
                .subscribe(Subscribe {
                    session: g.session(),
                    after,
                    ephemeral: true,
                })
                .map(|sub| (ViewGateway::InProc(g.clone()), sub))
                .map_err(|e| refusal_text(&e).to_string()),
            Self::Unix { socket, .. } => {
                let (client, sub) = UnixClient::discover(socket, after, true, approver)
                    .await
                    .map_err(|e| client_error_text(&e))?;
                let session = sub.hello().meta.session;
                Ok((
                    ViewGateway::Unix(RemoteGateway {
                        client,
                        session,
                        epoch: sub.hello().epoch,
                        work: Arc::new(tokio::sync::Semaphore::new(64)),
                        outcomes,
                    }),
                    sub,
                ))
            }
        }
    }
}
struct Mailbox {
    latest: Latest<FrameState>,
    notices: UiSender,
    pending_turns: arc_swap::ArcSwap<im::OrdSet<OpId>>,
    prompts: arc_swap::ArcSwap<im::OrdMap<OpId, PendingPrompt>>,
    width: AtomicU32,
    /// Global text zoom, percent (harness-spec.md Standalone window); rows re-prepare.
    zoom: AtomicU32,
    resized: tokio::sync::Notify,
    published: AtomicU64,
    blobs: arc_swap::ArcSwapOption<crate::store::Blobs>,
    profiles: arc_swap::ArcSwap<Vec<ProfileInfo>>,
    #[cfg(feature = "embedded")]
    faces: arc_swap::ArcSwap<super::rich::Faces>,
}
impl Mailbox {
    fn pending(&self, op: OpId, insert: bool) {
        self.pending_turns.rcu(|current| {
            let mut next = current.as_ref().clone();
            if insert {
                next.insert(op);
            } else {
                next.remove(&op);
            }
            Arc::new(next)
        });
    }
    fn prompts(&self, edit: impl Fn(&mut im::OrdMap<OpId, PendingPrompt>)) {
        let mut changed = false;
        self.prompts.rcu(|current| {
            let mut next = current.as_ref().clone();
            edit(&mut next);
            // RCU can retry. Only its successful attempt decides whether this
            // publisher changed the projection and owes a notification.
            changed = next != **current;
            if changed {
                Arc::new(next)
            } else {
                current.clone()
            }
        });
        if changed {
            let _ = self.notices.try_send(UiMsg::Changed { source: 0 });
        }
    }
    fn prompt_status(&self, op: OpId, status: &str) {
        self.prompts(|prompts| {
            if let Some(prompt) = prompts.get_mut(&op)
                && !prompt.status.starts_with("Rejected")
                && !prompt.status.starts_with("Ended")
                && (!status.starts_with("Queued") || prompt.status == "Sending…")
            {
                prompt.status = status.to_owned();
            }
        });
    }
    fn publish(&self, mut state: FrameState) {
        state.pending_prompts = self.prompts.load_full();
        state.revision = self.published.load(Ordering::Acquire) + 1;
        state.profiles = self.profiles.load_full();
        let revision = state.revision;
        self.latest.publish(state);
        // A saturated notification queue is harmless: Latest holds the complete
        // current state and its own dirty source, including all durable items.
        let _ = self.notices.try_send(UiMsg::Changed { source: 0 });
        self.published.store(revision, Ordering::Release);
    }
}
#[cfg(feature = "test-fixtures")]
type BeforeHost = Box<dyn FnOnce(&crate::redact::Set) + Send>;
pub struct Bridge {
    mailbox: Arc<Mailbox>,
    messages: UiReceiver,
    pub dirty: DirtySet,
    pub state: FrameState,
    detach: CancellationToken,
    worker: Option<tokio::task::JoinHandle<()>>,
    /// Profile switches for the live session's host worker.
    switch: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    #[cfg(feature = "test-fixtures")]
    before_host: Option<BeforeHost>,
}
impl Bridge {
    pub fn new(waker: Arc<dyn Waker>) -> Self {
        let dirty = DirtySet::new(2, waker);
        let (notices, messages) = bbt_rt::ui_channel(16, dirty.paint_source(1));
        Self {
            mailbox: Arc::new(Mailbox {
                latest: Latest::new(dirty.paint_source(0)),
                notices,
                pending_turns: arc_swap::ArcSwap::from_pointee(im::OrdSet::new()),
                prompts: arc_swap::ArcSwap::from_pointee(im::OrdMap::new()),
                width: AtomicU32::new(1120f32.to_bits()),
                zoom: AtomicU32::new(u32::from(super::zoom::DEFAULT)),
                resized: tokio::sync::Notify::new(),
                published: AtomicU64::new(0),
                blobs: arc_swap::ArcSwapOption::empty(),
                profiles: arc_swap::ArcSwap::from_pointee(vec![]),
                #[cfg(feature = "embedded")]
                faces: arc_swap::ArcSwap::from_pointee(Default::default()),
            }),
            messages,
            dirty,
            state: FrameState::default(),
            detach: CancellationToken::new(),
            worker: None,
            switch: None,
            #[cfg(feature = "test-fixtures")]
            before_host: None,
        }
    }
    pub fn published_revision(&self) -> u64 {
        self.mailbox.published.load(Ordering::Acquire)
    }
    pub fn pending_notices(&self) -> bool {
        !self.messages.is_empty()
    }
    /// IO is scheduled, never awaited by the caller/frame owner.
    pub fn replay(&mut self, path: PathBuf) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            let redactor = crate::redact::Set::default();
            crate::capability::register(&redactor).await;
            let root = path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            match read_replay_with_redactor(path, redactor.clone()).await {
                Ok(transcript) => {
                    if let Some(session) = transcript.session { mailbox.blobs.store(Some(Arc::new(crate::store::Blobs { dir: root.join(session.to_string()).join("blobs"), redactor: redactor.clone() }))); }
                    let state = FrameState { transcript: Arc::new(transcript), notice: redactor.capability_notice(), replay: true, ready: true, ..Default::default() };
                    let mut preparer = Preparer::default();
                    loop {
                        publish_frame(state.clone(), &mailbox, &mut preparer).await;
                        tokio::select! { _ = detach.cancelled() => return, _ = mailbox.resized.notified() => {} }
                    }
                }
                Err(error) => mailbox.publish(FrameState { error: Some(error), replay: true, ..Default::default() }),
            }
        }));
    }
    /// Resizing schedules preparation; the previous solved rows remain visible.
    pub fn set_width(&self, width: f32) {
        if width.is_finite() {
            let bits = width.max(240.).to_bits();
            if self.mailbox.width.swap(bits, Ordering::AcqRel) != bits {
                self.mailbox.resized.notify_one();
            }
        }
    }
    /// A zoom change schedules preparation at the new text size, like a resize.
    pub fn set_zoom(&self, percent: u8) {
        let percent = u32::from(percent);
        if self.mailbox.zoom.swap(percent, Ordering::AcqRel) != percent {
            self.mailbox.resized.notify_one();
        }
    }
    #[cfg(feature = "embedded")]
    pub fn toggle(&self, item: ItemId, toggle: super::rich::Toggle) {
        self.mailbox.faces.rcu(|current| {
            let mut next = current.as_ref().clone();
            let mut face = next.get(&item).cloned().unwrap_or_default();
            face.apply(toggle.clone());
            next.insert(item, face);
            Arc::new(next)
        });
        self.mailbox.resized.notify_one();
    }
    /// Test-only observation at the actual live startup boundary, with no UI.
    #[cfg(feature = "test-fixtures")]
    pub fn observe_host_start(
        &mut self,
        observe: impl FnOnce(&crate::redact::Set) + Send + 'static,
    ) {
        self.before_host = Some(Box::new(observe));
    }
    /// Continue the live session on another profile (any backend): the host forks it
    /// at its head onto `profile` and the view follows the new session.
    pub fn switch_profile(&self, profile: String) -> Result<(), &'static str> {
        if matches!(self.state.gateway, Some(ViewGateway::Unix(_))) {
            return self
                .send(Op::SetProfile { profile })
                .map(|_| ())
                .map_err(|_| "profile switch not admitted");
        }
        let tx = self.switch.as_ref().ok_or("no live session to switch")?;
        tx.send(profile).map_err(|_| "the session host has stopped")
    }
    pub fn create(&mut self, config: Config) {
        self.start(config, None, None)
    }
    pub fn resume(&mut self, config: Config, session: SessionId) {
        self.start(config, None, Some(session))
    }
    /// Render another harness's native session live (read-only until `/profile` forks it).
    pub fn watch(&mut self, config: Config, native: Option<crate::import::Native>, path: PathBuf) {
        self.start(config, Some((native, path)), None)
    }
    fn start(
        &mut self,
        config: Config,
        watch: Option<(Option<crate::import::Native>, PathBuf)>,
        resume: Option<SessionId>,
    ) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        let (switch_tx, mut switches) = tokio::sync::mpsc::unbounded_channel::<String>();
        self.switch = Some(switch_tx);
        #[cfg(feature = "test-fixtures")]
        let before_host = self.before_host.take();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            crate::capability::register(&config.redactor).await;
            let session = resume.unwrap_or(config.meta.session);
            mailbox.blobs.store(Some(Arc::new(crate::store::Blobs {
                dir: config.sessions.join(session.to_string()).join("blobs"),
                redactor: config.redactor.clone(),
            })));
            let op = Op::StartSession {
                cwd: config.meta.cwd.clone(),
                profile: config.meta.profile.clone(),
                fork: None,
            };
            #[cfg(feature = "test-fixtures")]
            if let Some(observe) = before_host {
                observe(&config.redactor);
            }
            let sessions = config.sessions.clone();
            let root = config.profiles.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            let redactor = config.redactor.clone();
            mailbox.profiles.store(Arc::new(profile_names(&config.profiles).await));
            let cwd = config.meta.cwd.clone();
            let host = Host::new(config);
            let created = if resume.is_some() {
                host.open(session).await.map(|g| (g, crate::gateway::Ack { op_id: OpId::new(), duplicate: false }))
            } else { match watch {
                Some((native, path)) => host.watch(native, path, cwd).await.map(|g| (g, crate::gateway::Ack { op_id: OpId::new(), duplicate: false })),
                None => host.create(OpEnvelope { v: PROTOCOL, op_id: OpId::new(), session, op }).await,
            }};
            match created {
                Ok((mut gateway, _)) => {
                    let (remote_tx, mut remote_switches) = tokio::sync::mpsc::channel::<crate::gateway::Admitted>(64);
                    gateway = gateway.with_profile_switch(remote_tx.clone());
                    let mut socket = match crate::frontends::seat::bind_session(&root, gateway.clone(), resume.is_some()).await {
                        Ok(socket) => socket,
                        Err(error) => {
                            mailbox.publish(FrameState { error: Some(error), ..Default::default() });
                            host.close_all().await;
                            return;
                        }
                    };
                    loop {
                        tokio::select! {
                            _ = consume(Source::InProc(gateway.clone()), None, mailbox.clone(), detach.clone()) => break,
                            Some(profile) = async {
                                tokio::select! {
                                    profile = switches.recv() => profile,
                                    request = remote_switches.recv() => request.and_then(|request| {
                                        if request.env.session != gateway.session() { return None; }
                                        match request.env.op { Op::SetProfile { profile } => Some(profile), _ => None }
                                    }),
                                }
                            } => match host.switch(&gateway, profile.clone()).await {
                                Ok(next) => {
                                    let next = next.with_profile_switch(remote_tx.clone());
                                    match crate::frontends::seat::bind_session(&root, next.clone(), false).await {
                                        Ok(next_socket) => { let _ = std::mem::replace(&mut socket, next_socket).retire().await; }
                                        Err(error) => {
                                            mailbox.publish(FrameState { error: Some(error), ..Default::default() });
                                            host.close_all().await;
                                            break;
                                        }
                                    }
                                    mailbox.blobs.store(Some(Arc::new(crate::store::Blobs {
                                        dir: sessions.join(next.session().to_string()).join("blobs"),
                                        redactor: redactor.clone(),
                                    })));
                                    gateway = next;
                                }
                                Err(error) => mailbox.publish(FrameState {
                                    error: Some(format!("Could not switch to {profile}: {}", refusal_text(&error))),
                                    ..Default::default()
                                }),
                            },
                        }
                    }
                    let _ = socket.close().await;
                    // Keep the actor host alive through detached work. A detach
                    // sends no Close. The idle log remains resumable on disk.
                    drop(host);
                }
                Err(error) => mailbox.publish(FrameState {
                    error: Some(format!("Session could not start: {}", refusal_text(&error))),
                    ..Default::default()
                }),
            }
        }));
    }
    pub fn attach_with_blobs(&mut self, gateway: InProc, blobs: crate::store::Blobs) {
        self.mailbox.blobs.store(Some(Arc::new(blobs)));
        self.attach(gateway);
    }
    pub fn attach(&mut self, gateway: InProc) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker =
            Some(bbt_rt::handle().spawn(consume(Source::InProc(gateway), None, mailbox, detach)));
    }
    /// Connect to a session or host-chosen seat socket. All socket IO, decoding, retries and row
    /// preparation run on bbt_rt; the caller only schedules and drains Latest.
    pub fn connect(&mut self, socket: PathBuf, after: Option<(Epoch, Seq)>) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker =
            Some(bbt_rt::handle().spawn(consume(Source::Unix { socket }, after, mailbox, detach)));
    }
    /// A real frame drains under 0.3ms. complete_drain re-marks any remainder,
    /// including a publication racing the just-completed frame.
    pub fn drain(&mut self, budget: Duration) -> bool {
        self.drain_with_clock(budget, &super::budget::RealClock::default())
    }
    pub fn drain_with_clock(
        &mut self,
        budget: Duration,
        clock: &impl super::budget::Clock,
    ) -> bool {
        #[allow(
            clippy::large_enum_variant,
            reason = "Moves one prepared frame at a time; boxing adds a heap allocation to every UI drain"
        )]
        enum Update {
            State(FrameState),
            Notice,
        }
        self.dirty.drain();
        let mailbox = &self.mailbox;
        let messages = &self.messages;
        let state = &mut self.state;
        let mut changed = false;
        super::budget::drain(
            clock,
            budget,
            || {
                mailbox
                    .latest
                    .take()
                    .map(Update::State)
                    .or_else(|| messages.try_recv().map(|_| Update::Notice))
            },
            |update| {
                if let Update::State(next) = update {
                    *state = next;
                    changed = true;
                }
            },
        );
        self.messages.complete_drain();
        self.mailbox.latest.complete_drain();
        // Retire on the UI owner's consumed projection, after worker preparation.
        // Clearing at event arrival could briefly show neither gray nor blue.
        if self
            .mailbox
            .prompts
            .load()
            .values()
            .any(|p| p.confirmed(&state.transcript))
        {
            self.mailbox.prompts(|prompts| {
                let confirmed: Vec<_> = prompts
                    .iter()
                    .filter(|(_, p)| p.confirmed(&state.transcript))
                    .map(|(op, _)| *op)
                    .collect();
                for op in confirmed {
                    prompts.remove(&op);
                }
            });
        }
        let prompts = self.mailbox.prompts.load_full();
        changed |= !Arc::ptr_eq(&state.pending_prompts, &prompts);
        state.pending_prompts = prompts;
        changed
    }
    pub fn send(&self, op: Op) -> Result<Submission, Refused> {
        let session = self
            .state
            .gateway
            .as_ref()
            .ok_or(Refused::Closed)?
            .session();
        self.send_envelope(OpEnvelope {
            v: PROTOCOL,
            op_id: OpId::new(),
            session,
            op,
        })
    }
    /// Resends must carry the original session and op_id, including after a
    /// timeout or a switch. Scheduling a remote request is not admission.
    pub fn send_envelope(&self, envelope: OpEnvelope) -> Result<Submission, Refused> {
        let gateway = self.state.gateway.as_ref().ok_or(Refused::Closed)?;
        let turn = matches!(envelope.op, Op::UserTurn { .. });
        let op_id = envelope.op_id;
        if let Op::UserTurn { blocks, .. } = &envelope.op {
            let text = blocks
                .iter()
                .map(InputBlock::display_text)
                .collect::<Vec<_>>()
                .join("\n");
            let prompt =
                PendingPrompt::new(op_id, envelope.session, self.state.transcript.epoch, text);
            self.mailbox.prompts(|prompts| {
                prompts.entry(op_id).or_insert(prompt.clone());
            });
            self.mailbox.pending(op_id, true);
        }
        let result = gateway.send(envelope);
        if turn && result.is_err() {
            self.mailbox.pending(op_id, false);
            self.mailbox.prompts(|prompts| {
                prompts.remove(&op_id);
            });
        }
        result
    }
    /// Window-close and embedded detach share this path. Only Stop sends Close.
    pub fn request_detach(&self) {
        self.detach.cancel();
    }
    pub fn detach(&mut self) -> Option<tokio::task::JoinHandle<()>> {
        self.detach.cancel();
        self.worker.take()
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.detach.cancel();
    }
}
#[cfg(feature = "embedded")]
type Preparer = super::view::Preparer;
#[cfg(not(feature = "embedded"))]
#[derive(Default)]
struct Preparer {}
async fn publish_frame(mut state: FrameState, mailbox: &Arc<Mailbox>, preparer: &mut Preparer) {
    #[cfg(feature = "embedded")]
    {
        if let Some(blobs) = mailbox.blobs.load_full() {
            load_terminal_outputs(&state.transcript, &blobs, preparer).await;
        }
        state.faces = mailbox.faces.load_full();
        let faces = state.faces.clone();
        let mut owned = std::mem::take(preparer);
        let source = state.transcript.clone();
        let width = f32::from_bits(mailbox.width.load(Ordering::Acquire));
        let zoom = mailbox.zoom.load(Ordering::Acquire);
        let publisher = mailbox.clone();
        *preparer = crate::runtime::blocking(move || {
            owned.set_zoom(super::zoom::Zoom::from_percent(
                i32::try_from(zoom).unwrap_or(i32::from(super::zoom::DEFAULT)),
            ));
            match owned.prepare_with_faces(source, width, &faces) {
                Ok(prepared) => state.prepared = Some(Arc::new(prepared)),
                Err(error) => state.error = Some(error),
            }
            // Replacing Latest can retire a large old layout. Retire it on the
            // preparation worker as well, never on either shared IO worker.
            publisher.publish(state);
            owned
        })
        .await;
    }
    #[cfg(not(feature = "embedded"))]
    {
        let _ = preparer;
        let _ = &mut state;
        mailbox.publish(state);
    }
}
fn apply_event(
    model: &mut Transcript,
    event: &Envelope,
    owned: &mut std::collections::BTreeMap<TurnId, OpId>,
    mailbox: &Mailbox,
) -> Effect {
    // Causal op IDs keep historical/replayed endings from retiring a new turn.
    if event.seq.is_some_and(|s| s > model.cursor) {
        match &event.event {
            Event::TurnQueued { turn } => {
                if let Some(op) = event.cause
                    && mailbox.pending_turns.load().contains(&op)
                {
                    owned.insert(*turn, op);
                }
            }
            Event::TurnEnded { turn, .. } => {
                if let Some(op) = owned.remove(turn) {
                    mailbox.pending(op, false);
                }
            }
            Event::OpRejected { op, .. } => mailbox.pending(*op, false),
            _ => {}
        }
    }
    // Accept only events admitted by this model's current session/epoch/cursor.
    if event.session == model.session.unwrap_or(event.session)
        && event.epoch == model.epoch
        && event.seq.is_some_and(|seq| seq > model.cursor)
        && !mailbox.prompts.load().is_empty()
        && matches!(
            &event.event,
            Event::TurnQueued { .. }
                | Event::TurnEnded { .. }
                | Event::OpRejected { .. }
                | Event::ItemStarted {
                    kind: ItemKind::UserMessage,
                    ..
                }
                | Event::ItemCompleted {
                    body: Item::UserMessage { .. },
                    ..
                }
        )
    {
        mailbox.prompts(|prompts| {
            let ids: Vec<_> = prompts.keys().copied().collect();
            for id in ids {
                if let Some(prompt) = prompts.get_mut(&id) {
                    prompt.observe(event);
                }
            }
        });
    }
    model.apply(event)
}
async fn consume(
    source: Source,
    initial_after: Option<(Epoch, Seq)>,
    mailbox: Arc<Mailbox>,
    detach: CancellationToken,
) {
    let mut model = Transcript::default();
    let mut after = initial_after;
    let mut detached = false;
    let mut owned_turns = std::collections::BTreeMap::new();
    let mut preparer = Preparer::default();
    let (outcomes, mut responses) = tokio::sync::mpsc::unbounded_channel();
    let mut dispatch_error = None;
    let mut last_connect: Option<tokio::time::Instant> = None;
    loop {
        if detached && mailbox.pending_turns.load().is_empty() {
            return;
        }
        if matches!(source, Source::Unix { .. }) {
            if let Some(last) = last_connect {
                tokio::select! {
                    _ = detach.cancelled(), if !detached => return,
                    _ = tokio::time::sleep_until(last + Duration::from_secs(2)) => {}
                }
            }
            last_connect = Some(tokio::time::Instant::now());
        }
        let (gateway, mut sub) = match tokio::select! {
            _ = detach.cancelled(), if !detached => { detached = true; continue; }
            result = source.subscribe(after, !detached, outcomes.clone()) => result,
        } {
            Ok(connected) => connected,
            Err(error) => {
                mailbox.publish(FrameState {
                    transcript: Arc::new(model.clone()),
                    error: Some(format!("Subscription unavailable: {error}")),
                    ..Default::default()
                });
                if matches!(source, Source::InProc(_)) || detached {
                    return;
                }
                tokio::select! {
                    _ = detach.cancelled() => return,
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                }
                continue;
            }
        };
        let connection_guard = ConnectionGuard(match &gateway {
            ViewGateway::Unix(remote) => Some(remote.client.clone()),
            ViewGateway::InProc(_) => None,
        });
        if model
            .session
            .is_some_and(|session| session != gateway.session())
        {
            // These intents belong to the retired session. Waiting for their
            // endings on its replacement would keep detach alive forever.
            for op in mailbox.pending_turns.load_full().iter() {
                mailbox.pending(*op, false);
            }
            model = Transcript::default();
            owned_turns.clear();
            dispatch_error = None;
            preparer = Preparer::default();
            after = None;
            continue;
        }
        if model.session.is_none() && after.is_some_and(|(_, seq)| seq > model.cursor) {
            // A fresh view has no retained prefix for an external cursor. Probe
            // the requested cursor, then rebuild the missing prefix from the log.
            // Internal lag/reconnect retains its model and needs only the suffix.
            after = None;
            continue;
        }
        sub.set_approver(!detached);
        model.hello(gateway.session(), sub.hello().epoch);
        publish_frame(
            FrameState {
                transcript: Arc::new(model.clone()),
                gateway: Some(gateway.clone()),
                ready: true,
                ..Default::default()
            },
            &mailbox,
            &mut preparer,
        )
        .await;
        loop {
            if detached && mailbox.pending_turns.load().is_empty() {
                return;
            }
            let event = tokio::select! {
                biased;
                _ = detach.cancelled(), if !detached => {
                    detached = true;
                    // Reconnect without the server's approver lease while an
                    // owned turn finishes. Detaching sends no Close.
                    after = Some((model.epoch, model.cursor));
                    break;
                }
                Some((session, epoch, op, result)) = responses.recv() => {
                    if session != gateway.session() || epoch != model.epoch {
                        mailbox.pending(op, false);
                        continue;
                    }
                    if result.is_ok() {
                        mailbox.prompt_status(op, "Queued · awaiting durable message…");
                    }
                    if let Err(error) = result {
                        mailbox.prompt_status(op, if matches!(error, ClientError::Refused(_)) {
                            "Rejected · message was not sent"
                        } else { "Outcome unknown · no automatic resend" });
                        mailbox.pending(op, false);
                        dispatch_error = Some(format!("Dispatch {op}: {}", client_error_text(&error)));
                        if !detached {
                            publish_frame(FrameState { transcript: Arc::new(model.clone()), gateway: Some(gateway.clone()), error: dispatch_error.clone(), ready: true, ..Default::default() }, &mailbox, &mut preparer).await;
                        }
                    }
                    continue;
                }
                _ = mailbox.resized.notified(), if !detached => {
                    publish_frame(FrameState { transcript: Arc::new(model.clone()), gateway: Some(gateway.clone()), error: dispatch_error.clone(), ready: true, ..Default::default() }, &mailbox, &mut preparer).await;
                    continue;
                }
                event = sub.recv() => event,
            };
            let Some(event) = event else {
                if matches!(source, Source::InProc(_)) {
                    return;
                }
                after = Some((model.epoch, model.cursor));
                break;
            };
            let mut effect = apply_event(&mut model, &event, &mut owned_turns, &mailbox);
            let start = Instant::now();
            let mut closed = false;
            while effect == Effect::None && start.elapsed() < Duration::from_micros(500) {
                match sub.try_recv() {
                    crate::gateway::TryRecv::Envelope(event) => {
                        effect = apply_event(&mut model, &event, &mut owned_turns, &mailbox)
                    }
                    crate::gateway::TryRecv::Empty => break,
                    crate::gateway::TryRecv::Closed => {
                        closed = true;
                        break;
                    }
                }
            }
            if !detached {
                publish_frame(
                    FrameState {
                        transcript: Arc::new(model.clone()),
                        gateway: Some(gateway.clone()),
                        error: dispatch_error.clone(),
                        ready: !model.closed,
                        ..Default::default()
                    },
                    &mailbox,
                    &mut preparer,
                )
                .await;
            }
            if model.closed && matches!(source, Source::Unix { .. }) {
                after = None;
                break;
            }
            if let Effect::Resubscribe { after: resume } = effect {
                after = resume;
                break;
            }
            if closed {
                if matches!(source, Source::InProc(_)) {
                    return;
                }
                after = Some((model.epoch, model.cursor));
                break;
            }
            tokio::task::yield_now().await;
        }
        if detached && mailbox.pending_turns.load().is_empty() {
            return;
        }
        // Lag/Reset reconnects and transport failure share the same cursor path.
        // Never spin on a malformed peer or move an intent to another session.
        if matches!(source, Source::Unix { .. }) {
            if !detached {
                publish_frame(
                    FrameState {
                        transcript: Arc::new(model.clone()),
                        error: (!model.closed).then(|| "Socket reconnecting".into()),
                        ..Default::default()
                    },
                    &mailbox,
                    &mut preparer,
                )
                .await;
            }
            drop(gateway);
            drop(connection_guard);
            tokio::select! {
                _ = detach.cancelled(), if !detached => return,
                _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            }
        }
    }
}
fn client_error_text(error: &ClientError) -> String {
    match error {
        ClientError::Refused(error) => refusal_text(error).into(),
        ClientError::Timeout { .. } => {
            "ack timeout; outcome unknown; retry only the same op_id".into()
        }
        ClientError::Disconnected => {
            "socket unavailable; outcome unknown; retry only the same op_id".into()
        }
        ClientError::Protocol(_) => "invalid socket protocol".into(),
    }
}
/// Validation and projection of large fixture files use the same four-permit
/// preparation pool as other harness blocking/CPU work, never a frame callback.
pub async fn read_replay(path: PathBuf) -> Result<Transcript, String> {
    let redactor = crate::redact::Set::default();
    crate::capability::register(&redactor).await;
    read_replay_with_redactor(path, redactor).await
}
#[allow(
    clippy::disallowed_methods,
    reason = "file reading and projection run inside runtime::blocking, never a frame"
)]
pub(crate) async fn read_replay_with_redactor(
    path: PathBuf,
    redactor: crate::redact::Set,
) -> Result<Transcript, String> {
    crate::runtime::blocking(move || {
        let bytes = std::fs::read(path).map_err(|_| "Replay file unavailable".to_string())?;
        let wire = bytes
            .split(|c| *c == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).map(|value| redactor.value(value)))
            .collect::<Result<Vec<serde_json::Value>, _>>()
            .map_err(|_| "Invalid replay JSONL".to_string())?;
        let events: Vec<Envelope> = wire
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect::<Result<_, _>>()
            .map_err(|_| "Invalid replay envelope".to_string())?;
        crate::protocol::laws::check_transcript(&events).map_err(|e| e.to_string())?;
        let mut model = Transcript::default();
        for (e, value) in events.iter().zip(&wire) {
            if let Effect::Resubscribe { after: None } =
                model.apply_wire(value).map_err(|e| e.to_string())?
            {
                // Offline Reset rebuilds from the surviving durable prefix.
                let Event::Reset { head, .. } = e.event else {
                    unreachable!()
                };
                for (_, kept) in events
                    .iter()
                    .zip(&wire)
                    .filter(|(v, _)| v.seq.is_some_and(|seq| seq <= head))
                {
                    model.apply_wire(kept).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(model)
    })
    .await
}

/// Terminal blobs are checked and read on runtime workers before row layout.
#[cfg(feature = "embedded")]
pub async fn load_terminal_outputs(
    model: &Transcript,
    blobs: &crate::store::Blobs,
    preparer: &mut super::view::Preparer,
) {
    if !preparer.same_scope(model) {
        let mut owned = std::mem::take(preparer);
        let source = model.clone();
        *preparer = crate::runtime::blocking(move || {
            owned.scope(&source);
            owned
        })
        .await;
    }
    for row in model.items.values() {
        if row.completed.is_some()
            && !preparer.has_terminal_output(&row.id)
            && let Item::Terminal {
                output: Some(blob), ..
            } = &row.body
        {
            let text = if blob.len == 0 {
                "(no output)".into()
            } else {
                match blobs.get(blob).await {
                    Ok(bytes) => {
                        crate::runtime::blocking(move || {
                            String::from_utf8_lossy(&bytes).into_owned()
                        })
                        .await
                    }
                    Err(_) => "Output unavailable: blob could not be read".into(),
                }
            };
            preparer.terminal_output(row.id, text);
        }
    }
}
pub fn refusal_text(error: &Refused) -> &'static str {
    match error {
        Refused::Busy => "session busy",
        Refused::UnknownSession => "session unavailable",
        Refused::Version { .. } => "protocol version mismatch",
        Refused::Closed => "session closed",
        Refused::SessionExists => "session already exists",
        Refused::BadFork => "fork unavailable",
    }
}

/// What the profile switcher may show about a profile: never a key, a key
/// source or an agent environment value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileInfo {
    /// File stem: what `/profile NAME` and `Host::switch` take.
    pub name: String,
    pub kind: ProfileKind,
    /// `provider:` as written (openrouter, anthropic, acp-agent, …).
    pub provider: Option<String>,
    /// The model, or empty when an ACP agent brings its own.
    pub model: String,
    /// An ACP agent's command words (program file name first); never its env.
    pub agent: Vec<String>,
    /// How many env names an ACP agent sets (never their values).
    pub agent_env: usize,
    /// Where a model's key comes from: the kind of source only.
    pub auth: ProfileAuth,
    /// False when `api_key_env` names a variable absent from this process.
    pub auth_ok: bool,
    pub context_tokens: Option<u64>,
    /// Named by `~/.bboxy450-harness/default-profile`.
    pub default: bool,
}
/// The kind of a model profile's key source (web AUTH_SAY), never the key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfileAuth {
    Env,
    Keychain,
    Command,
    /// A literal key in the profile file.
    InFile,
    #[default]
    None,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfileKind {
    #[default]
    Model,
    Agent,
    /// The file exists but its frontmatter does not parse as a profile.
    Unreadable,
}
impl ProfileInfo {
    pub fn parse(name: &str, text: &str) -> Self {
        let mut info = Self {
            name: name.to_owned(),
            ..Default::default()
        };
        match crate::provider::profile::Profile::parse(text) {
            Ok(p) => {
                info.kind = if p.is_acp_agent() {
                    ProfileKind::Agent
                } else {
                    ProfileKind::Model
                };
                info.provider = p.provider.clone();
                info.model = p.model.clone();
                info.context_tokens = p.context_tokens;
                info.auth_ok = true;
                info.auth = match (&p.api_key, &p.api_key_env) {
                    (Some(k), _) if k.starts_with("!security ") => ProfileAuth::Keychain,
                    (Some(k), _) if k.starts_with('!') => ProfileAuth::Command,
                    (Some(_), _) => ProfileAuth::InFile,
                    (None, Some(name)) => {
                        // presence only: the value is never read into the face
                        info.auth_ok = std::env::var_os(name).is_some();
                        ProfileAuth::Env
                    }
                    (None, None) => ProfileAuth::None,
                };
                if let Some(agent) = &p.agent {
                    info.agent = agent.command.clone();
                    info.agent_env = agent.env.len();
                    if let Some(first) = info.agent.first_mut()
                        && let Some(file) = std::path::Path::new(first.as_str()).file_name()
                    {
                        *first = file.to_string_lossy().into_owned();
                    }
                }
            }
            Err(_) => info.kind = ProfileKind::Unreadable,
        }
        info
    }
}

/// Profiles in the profiles directory, sorted by name. Read on the host worker,
/// never the UI thread; only the switcher-safe fields leave this function.
pub async fn profile_names(dir: &std::path::Path) -> Vec<ProfileInfo> {
    let mut out = vec![];
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(e)) = entries.next_entry().await {
            let path = e.path();
            if path.extension().is_some_and(|x| x == "md")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                let text = tokio::fs::read_to_string(&path).await.unwrap_or_default();
                out.push(ProfileInfo::parse(stem, &text));
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if let Some(parent) = dir.parent()
        && let Ok(name) = tokio::fs::read_to_string(parent.join("default-profile")).await
        && let Some(p) = out.iter_mut().find(|p| p.name == name.trim())
    {
        p.default = true;
    }
    out
}

#[cfg(test)]
mod gui_polish_tests {
    use super::*;
    #[test]
    fn gui_polish_pending_requires_causal_durable_user_in_current_birth() {
        let mut event: Envelope = serde_json::from_str(
            include_str!("../../../tests/fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let op = OpId::new();
        let item = ItemId::new();
        let mut prompt = PendingPrompt::new(op, event.session, event.epoch, "repeat".into());
        let mut other =
            PendingPrompt::new(OpId::new(), event.session, event.epoch, "repeat".into());
        event.cause = Some(op);
        event.seq = Some(Seq(100));
        event.event = Event::TurnQueued {
            turn: TurnId::new(),
        };
        assert!(!prompt.observe(&event));
        assert_eq!(prompt.status, "Queued…");
        event.event = Event::ItemStarted {
            turn: None,
            item,
            kind: ItemKind::UserMessage,
            parent: None,
            head: None,
        };
        assert!(!other.observe(&event));
        let mut stale = event.clone();
        stale.epoch = Epoch(event.epoch.0 + 1);
        assert!(!prompt.observe(&stale));
        assert!(prompt.item.is_none());
        stale = event.clone();
        stale.session = SessionId::new();
        assert!(!prompt.observe(&stale));
        assert!(prompt.item.is_none());
        assert!(!prompt.observe(&event));
        event.event = Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: Item::UserMessage {
                blocks: vec![InputBlock::Text {
                    text: "repeat".into(),
                }],
            },
        };
        event.cause = None; // completions inherit the immutable item's recorded cause
        assert!(prompt.observe(&event));
        assert!(!other.observe(&event));
        let mut bridge = Bridge::new(Arc::new(TestWake));
        bridge.state.transcript = Arc::new({
            let mut t = Transcript::default();
            t.hello(event.session, event.epoch);
            t
        });
        bridge.mailbox.prompts(|p| {
            p.insert(op, prompt.clone());
        });
        let mut model = (*bridge.state.transcript).clone();
        model.cursor = Seq(100);
        // Duplicate/stale durable records cannot retire a newly owned prompt.
        apply_event(&mut model, &event, &mut Default::default(), &bridge.mailbox);
        assert_eq!(bridge.mailbox.prompts.load().len(), 1);
        let completion = event.event.clone();
        event.event = Event::ItemStarted {
            turn: None,
            item,
            kind: ItemKind::UserMessage,
            parent: None,
            head: None,
        };
        event.seq = Some(Seq(101));
        apply_event(&mut model, &event, &mut Default::default(), &bridge.mailbox);
        event.seq = Some(Seq(102));
        event.event = completion;
        apply_event(&mut model, &event, &mut Default::default(), &bridge.mailbox);
        assert!(
            !bridge.mailbox.prompts.load().is_empty(),
            "pending retires only when prepared model is consumed"
        );
        bridge.state.transcript = Arc::new(model.clone());
        bridge.drain(Duration::from_millis(1));
        assert!(bridge.mailbox.prompts.load().is_empty());
        bridge.mailbox.prompts(|p| {
            p.insert(other.op, other.clone());
        });
        bridge
            .mailbox
            .prompt_status(other.op, "Queued · awaiting durable message…");
        assert_eq!(
            bridge.mailbox.prompts.load().len(),
            1,
            "Ack alone is still a gray local intent"
        );
        bridge
            .mailbox
            .prompt_status(other.op, "Outcome unknown · no automatic resend");
        assert!(
            bridge.mailbox.prompts.load()[&other.op]
                .status
                .contains("unknown")
        );
        event.seq = Some(Seq(103));
        event.event = Event::OpRejected {
            op: other.op,
            reason: Rejection::InvalidInput {
                detail: "fixture".into(),
            },
        };
        apply_event(&mut model, &event, &mut Default::default(), &bridge.mailbox);
        assert!(
            bridge.mailbox.prompts.load()[&other.op]
                .status
                .contains("Rejected")
        );
    }
    #[test]
    fn gui_polish_pending_noop_events_preserve_identity_and_notifications() {
        fn apply(model: &mut Transcript, bridge: &Bridge, event: &mut Envelope, body: Event) {
            event.seq = Some(Seq(model.cursor.0 + 1));
            event.at = event.seq.unwrap();
            event.event = body;
            apply_event(model, event, &mut Default::default(), &bridge.mailbox);
        }
        fn quiet_records(model: &mut Transcript, bridge: &Bridge, event: &mut Envelope) {
            let before = bridge.mailbox.prompts.load_full();
            event.cause = None;
            for body in [
                Item::Native {
                    provider: "fixture".into(),
                    record_type: "token".into(),
                    raw: serde_json::json!({"tokens":1}),
                    blob: None,
                },
                Item::Usage {
                    usage: Usage::default(),
                },
            ] {
                let item = ItemId::new();
                apply(
                    model,
                    bridge,
                    event,
                    Event::ItemStarted {
                        turn: None,
                        item,
                        kind: body.kind(),
                        parent: None,
                        head: None,
                    },
                );
                apply(
                    model,
                    bridge,
                    event,
                    Event::ItemCompleted {
                        item,
                        status: ItemStatus::Completed,
                        body,
                    },
                );
                assert!(
                    Arc::ptr_eq(&before, &bridge.mailbox.prompts.load_full()),
                    "unrelated records replaced the pending projection"
                );
                assert!(
                    !bridge.pending_notices(),
                    "unrelated records sent an extra pending notification"
                );
            }
        }
        let mut event: Envelope = serde_json::from_str(
            include_str!("../../../tests/fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let mut model = Transcript::default();
        model.hello(event.session, event.epoch);
        let mut bridge = Bridge::new(Arc::new(TestWake));
        quiet_records(&mut model, &bridge, &mut event);
        let op = OpId::new();
        let prompt = PendingPrompt::new(op, event.session, event.epoch, "pending".into());
        let empty = bridge.mailbox.prompts.load_full();
        bridge.mailbox.prompts(|prompts| {
            prompts.insert(op, prompt.clone());
        });
        assert!(!Arc::ptr_eq(&empty, &bridge.mailbox.prompts.load_full()));
        assert!(bridge.pending_notices(), "local submit must notify");
        bridge.drain(Duration::from_millis(1));
        quiet_records(&mut model, &bridge, &mut event);
        // A relevant event for another immutable op is also a no-op.
        event.cause = Some(OpId::new());
        let before = bridge.mailbox.prompts.load_full();
        apply(
            &mut model,
            &bridge,
            &mut event,
            Event::TurnQueued {
                turn: TurnId::new(),
            },
        );
        assert!(Arc::ptr_eq(&before, &bridge.mailbox.prompts.load_full()));
        assert!(!bridge.pending_notices());
        event.cause = Some(op);
        apply(
            &mut model,
            &bridge,
            &mut event,
            Event::TurnQueued {
                turn: TurnId::new(),
            },
        );
        assert!(!Arc::ptr_eq(&before, &bridge.mailbox.prompts.load_full()));
        assert!(bridge.pending_notices(), "queued status must notify");
        bridge.drain(Duration::from_millis(1));
        let before = bridge.mailbox.prompts.load_full();
        bridge.mailbox.prompt_status(op, "Queued…");
        assert!(Arc::ptr_eq(&before, &bridge.mailbox.prompts.load_full()));
        assert!(
            !bridge.pending_notices(),
            "unchanged status must stay quiet"
        );
        let item = ItemId::new();
        apply(
            &mut model,
            &bridge,
            &mut event,
            Event::ItemStarted {
                turn: None,
                item,
                kind: ItemKind::UserMessage,
                parent: None,
                head: None,
            },
        );
        assert!(
            bridge.pending_notices(),
            "causal ownership update must notify"
        );
        bridge.drain(Duration::from_millis(1));
        event.cause = None;
        apply(
            &mut model,
            &bridge,
            &mut event,
            Event::ItemCompleted {
                item,
                status: ItemStatus::Completed,
                body: Item::UserMessage {
                    blocks: vec![InputBlock::Text {
                        text: "pending".into(),
                    }],
                },
            },
        );
        bridge.state.transcript = Arc::new(model);
        bridge.drain(Duration::from_millis(1));
        assert!(bridge.mailbox.prompts.load().is_empty());
        assert!(
            bridge.pending_notices(),
            "retiring confirmed local prompt must notify"
        );
    }
    struct TestWake;
    impl Waker for TestWake {
        fn wake(&self) {}
    }
}
