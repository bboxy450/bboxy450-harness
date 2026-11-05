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
#[derive(Clone, Debug, PartialEq)]
pub struct PendingPrompt {
    pub op: OpId,
    pub session: SessionId,
    pub epoch: Epoch,
    pub text: String,
    pub blocks: Vec<InputBlock>,
    pub custody: Vec<Arc<crate::store::intake::ImageCustody>>,
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
            blocks: vec![],
            custody: vec![],
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
    fn confirmed_in(&self, state: &FrameState) -> bool {
        if !self.confirmed(&state.transcript) {
            return false;
        }
        #[cfg(feature = "embedded")]
        {
            state
                .prepared
                .as_ref()
                .is_some_and(|rows| self.confirmed(&rows.source))
        }
        #[cfg(not(feature = "embedded"))]
        {
            true
        }
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
            Event::TurnEnded { turn, .. }
                if self.turn == Some(*turn)
                    && self.item.is_none()
                    && !self.status.starts_with("Rejected") =>
            {
                self.status = "Ended · no durable message was recorded".into();
            }
            Event::ItemCompleted {
                body: Item::Notice { text, .. },
                ..
            } if event.cause == Some(self.op) && text.starts_with("Steering delivery unknown") => {
                self.status = text.clone();
            }
            Event::OpRejected { op, .. } if *op == self.op => {
                self.status = "Rejected · message was not sent".into()
            }
            _ => {}
        }
        false
    }
}

/// A live-connection proof. Replayed records, a bare Ack and old challenges
/// cannot authorize the local custody Resource protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorIntake {
    session: SessionId,
    epoch: Epoch,
    request: OpId,
    version: u32,
}
#[derive(Clone, Default)]
pub struct FrameState {
    pub actor_intake: Option<ActorIntake>,
    pub revision: u64,
    pub pending_prompts: Arc<im::OrdMap<OpId, PendingPrompt>>,
    pub transcript: Arc<Transcript>,
    pub gateway: Option<ViewGateway>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub replay: bool,
    pub ready: bool,
    /// The UI created this exact session; attached/replayed history never has this intent.
    pub created_here: bool,
    /// Profiles in the machine-global profiles directory, for the switcher.
    pub profiles: Arc<Vec<ProfileInfo>>,
    pub onboarding: Arc<crate::onboarding::Snapshot>,
    pub profile_error: Option<String>,
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
                let report = !matches!(op.op, Op::QueryInputCapabilities { .. });
                bbt_rt::handle().spawn(async move {
                    let result = client.send(op).await;
                    // Discovery refusal/timeout leaves custody unavailable; it
                    // is not a failed user intent, including after reconnect.
                    if report {
                        let _ = outcomes.send((session, epoch, op_id, result));
                    }
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
    actor_intake: arc_swap::ArcSwapOption<ActorIntake>,
    latest: Latest<FrameState>,
    notices: UiSender,
    pending_turns: arc_swap::ArcSwap<im::OrdSet<OpId>>,
    prompts: arc_swap::ArcSwap<im::OrdMap<OpId, PendingPrompt>>,
    width: AtomicU32,
    /// Global text zoom, percent (harness-spec.md Standalone window); rows re-prepare.
    zoom: AtomicU32,
    /// Low bit is light; upper bits identify every local palette transition.
    #[cfg(feature = "embedded")]
    appearance: AtomicU64,
    resized: tokio::sync::Notify,
    published: AtomicU64,
    blobs: arc_swap::ArcSwapOption<crate::store::Blobs>,
    profiles: arc_swap::ArcSwap<Vec<ProfileInfo>>,
    onboarding: arc_swap::ArcSwap<crate::onboarding::Snapshot>,
    profile_error: arc_swap::ArcSwapOption<String>,
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
            self.resized.notify_one();
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
        state.actor_intake = self
            .actor_intake
            .load_full()
            .filter(|c| {
                Some(c.session) == state.transcript.session && c.epoch == state.transcript.epoch
            })
            .map(|c| (*c).clone());
        state.pending_prompts = self.prompts.load_full();
        state.revision = self.published.load(Ordering::Acquire) + 1;
        state.profiles = self.profiles.load_full();
        state.profile_error = self.profile_error.load_full().map(|s| (*s).clone());
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
    input_waker: Arc<dyn Waker>,
    mailbox: Arc<Mailbox>,
    messages: UiReceiver,
    pub dirty: DirtySet,
    pub state: FrameState,
    detach: CancellationToken,
    worker: Option<tokio::task::JoinHandle<()>>,
    /// Profile switches for the live session's host worker.
    switch: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    setup: Option<Arc<crate::onboarding::Controller>>,
    created_session: Option<SessionId>,
    #[cfg(feature = "test-fixtures")]
    before_host: Option<BeforeHost>,
}
impl Bridge {
    pub fn new(waker: Arc<dyn Waker>) -> Self {
        let dirty = DirtySet::new(2, waker.clone());
        let (notices, messages) = bbt_rt::ui_channel(16, dirty.paint_source(1));
        Self {
            input_waker: waker,
            mailbox: Arc::new(Mailbox {
                actor_intake: arc_swap::ArcSwapOption::empty(),
                latest: Latest::new(dirty.paint_source(0)),
                notices,
                pending_turns: arc_swap::ArcSwap::from_pointee(im::OrdSet::new()),
                prompts: arc_swap::ArcSwap::from_pointee(im::OrdMap::new()),
                width: AtomicU32::new(1120f32.to_bits()),
                zoom: AtomicU32::new(u32::from(super::zoom::DEFAULT)),
                #[cfg(feature = "embedded")]
                appearance: AtomicU64::new(0),
                resized: tokio::sync::Notify::new(),
                published: AtomicU64::new(0),
                blobs: arc_swap::ArcSwapOption::empty(),
                profiles: arc_swap::ArcSwap::from_pointee(vec![]),
                onboarding: arc_swap::ArcSwap::from_pointee(Default::default()),
                profile_error: arc_swap::ArcSwapOption::empty(),
                #[cfg(feature = "embedded")]
                faces: arc_swap::ArcSwap::from_pointee(Default::default()),
            }),
            messages,
            dirty,
            state: FrameState::default(),
            detach: CancellationToken::new(),
            worker: None,
            switch: None,
            setup: None,
            created_session: None,
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
        self.created_session = None;
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
    #[cfg(feature = "embedded")]
    pub fn set_theme(&self, theme: bbt_ui::Theme) -> bool {
        let light = u64::from(theme == bbt_ui::Theme::Light);
        let changed = self
            .mailbox
            .appearance
            .try_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                ((old & 1) != light).then(|| ((old >> 1).wrapping_add(1) << 1) | light)
            })
            .is_ok();
        if changed {
            self.mailbox.resized.notify_one();
        }
        changed
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
        self.created_session = Some(config.meta.session);
        self.start(config, None, None, false)
    }
    /// Standalone new-session startup shares headless discovery and precedence.
    pub fn create_with_defaults(&mut self, config: Config) {
        self.created_session = Some(config.meta.session);
        self.start(config, None, None, true)
    }
    pub fn resume(&mut self, config: Config, session: SessionId) {
        self.created_session = None;
        self.start(config, None, Some(session), false)
    }
    /// Render another harness's native session live (read-only until `/profile` forks it).
    pub fn watch(&mut self, config: Config, native: Option<crate::import::Native>, path: PathBuf) {
        self.created_session = None;
        self.start(config, Some((native, path)), None, false)
    }
    fn attach_setup(&mut self, profiles: Option<PathBuf>) {
        let dir = profiles.clone();
        let Some(env) = profiles.and_then(crate::onboarding::Environment::host) else {
            return;
        };
        let controller = crate::onboarding::Controller::shared(env);
        let mut updates = controller.subscribe();
        self.setup = Some(controller);
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        bbt_rt::handle().spawn(async move {
            loop {
                let snapshot = updates.borrow_and_update().clone();
                let old=mailbox.onboarding.load_full();
                let changed = old.entries != snapshot.entries || old.catalogue_revision != snapshot.catalogue_revision;
                mailbox.onboarding.store(snapshot);
                if changed { load_catalogue(&mailbox, dir.clone()).await; }
                let _ = mailbox.notices.try_send(UiMsg::Changed { source: 0 });
                tokio::select! { _=detach.cancelled()=>break, result=updates.changed()=>if result.is_err() { break } }
            }
        });
    }
    pub fn install_adapter(&self, recipe: crate::onboarding::Recipe) -> Result<(), &'static str> {
        self.setup
            .as_ref()
            .ok_or("Setup unavailable in replay")?
            .install(recipe)
    }
    pub fn cancel_install(&self, agent: crate::onboarding::Agent) {
        if let Some(setup) = &self.setup {
            setup.cancel(agent);
        }
    }
    fn start(
        &mut self,
        mut config: Config,
        watch: Option<(Option<crate::import::Native>, PathBuf)>,
        resume: Option<SessionId>,
        bootstrap: bool,
    ) {
        self.attach_setup(Some(config.profiles.clone()));
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        let (switch_tx, mut switches) = tokio::sync::mpsc::unbounded_channel::<String>();
        self.switch = Some(switch_tx);
        #[cfg(feature = "test-fixtures")]
        let before_host = self.before_host.take();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            if bootstrap {
                config.meta.profile = Some(crate::onboarding::bootstrap(
                    config.profiles.clone(), config.meta.profile.take(), true,
                ).await);
            }
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
            load_catalogue(&mailbox, Some(config.profiles.clone())).await;
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
    pub fn input_waker(&self) -> Arc<dyn Waker> {
        self.input_waker.clone()
    }
    pub fn attach_with_blobs(&mut self, gateway: InProc, blobs: crate::store::Blobs) {
        self.mailbox.blobs.store(Some(Arc::new(blobs)));
        self.attach(gateway);
    }
    pub fn attach(&mut self, gateway: InProc) {
        self.created_session = None;
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker =
            Some(bbt_rt::handle().spawn(consume(Source::InProc(gateway), None, mailbox, detach)));
    }
    /// Connect to a session or host-chosen seat socket. All socket IO, decoding, retries and row
    /// preparation run on bbt_rt; the caller only schedules and drains Latest.
    pub fn connect(&mut self, socket: PathBuf, after: Option<(Epoch, Seq)>) {
        let profiles = machine_profiles(std::env::var_os("HOME"));
        self.connect_with_profiles(socket, after, profiles);
    }
    /// A caller with a private harness home supplies its catalogue directory.
    /// Both entry points use the same safe projection as standalone startup.
    pub fn connect_with_profiles(
        &mut self,
        socket: PathBuf,
        after: Option<(Epoch, Seq)>,
        profiles: Option<PathBuf>,
    ) {
        self.created_session = None;
        self.attach_setup(profiles.clone());
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            load_catalogue(&mailbox, profiles).await;
            consume(Source::Unix { socket }, after, mailbox, detach).await;
        }));
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
        state.created_here =
            self.created_session.is_some() && self.created_session == state.transcript.session;
        // Retire on the UI owner's consumed projection, after worker preparation.
        // Clearing at event arrival could briefly show neither gray nor blue.
        if self
            .mailbox
            .prompts
            .load()
            .values()
            .any(|p| p.confirmed_in(state))
        {
            self.mailbox.prompts(|prompts| {
                let confirmed: Vec<_> = prompts
                    .iter()
                    .filter(|(_, p)| p.confirmed_in(state))
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
        let setup = self.mailbox.onboarding.load_full();
        changed |= !Arc::ptr_eq(&state.onboarding, &setup);
        state.onboarding = setup;
        let profiles = self.mailbox.profiles.load_full();
        changed |= !Arc::ptr_eq(&state.profiles, &profiles);
        state.profiles = profiles;
        state.profile_error = self.mailbox.profile_error.load_full().map(|s| (*s).clone());
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
        self.send_with_custody(envelope, vec![])
    }
    pub fn image_intake_available(&self) -> bool {
        let Some(current) = self.mailbox.actor_intake.load_full() else {
            return false;
        };
        current.version == 1
            && self.state.actor_intake.as_ref() == Some(current.as_ref())
            && self.state.transcript.session == Some(current.session)
            && self.state.transcript.epoch == current.epoch
            && self
                .state
                .gateway
                .as_ref()
                .is_some_and(|g| g.session() == current.session)
    }
    /// The transport worker retains staged images even if its disposable view detaches.
    pub fn send_with_custody(
        &self,
        envelope: OpEnvelope,
        custody: Vec<Arc<crate::store::intake::ImageCustody>>,
    ) -> Result<Submission, Refused> {
        let gateway = self.state.gateway.as_ref().ok_or(Refused::Closed)?;
        if matches!(&envelope.op,Op::UserTurn {blocks,..} if blocks.iter().any(is_local_image))
            && !self.image_intake_available()
        {
            // Local non-admission. The view names the missing capability; no
            // pending intent or custody transfer has happened yet.
            return Err(Refused::Busy);
        }
        let turn = matches!(envelope.op, Op::UserTurn { .. });
        let op_id = envelope.op_id;
        if let Op::UserTurn { blocks, .. } = &envelope.op {
            let text = blocks
                .iter()
                .map(InputBlock::display_text)
                .collect::<Vec<_>>()
                .join("\n");
            let mut prompt =
                PendingPrompt::new(op_id, envelope.session, self.state.transcript.epoch, text);
            prompt.blocks = blocks.clone();
            prompt.custody = custody;
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
        let prompts = mailbox.prompts.load_full();
        let width = f32::from_bits(mailbox.width.load(Ordering::Acquire));
        let zoom = mailbox.zoom.load(Ordering::Acquire);
        let appearance = mailbox.appearance.load(Ordering::Acquire);
        let publisher = mailbox.clone();
        *preparer = crate::runtime::blocking(move || {
            owned.set_zoom(super::zoom::Zoom::from_percent(
                i32::try_from(zoom).unwrap_or(i32::from(super::zoom::DEFAULT)),
            ));
            owned.set_theme_revision(
                if appearance & 1 == 1 {
                    bbt_ui::Theme::Light
                } else {
                    bbt_ui::Theme::Dark
                },
                appearance >> 1,
            );
            match owned.prepare_with_faces(source, width, &faces) {
                Ok(mut prepared) => {
                    if let Err(error) = owned.prepare_pending(&mut prepared, &prompts, &faces) {
                        state.error = Some(error);
                    }
                    state.prepared = Some(Arc::new(prepared));
                }
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
    if let Event::InputCapabilities {
        request,
        image_custody_version,
    } = event.event
        && event.seq.is_none()
        && model.session == Some(event.session)
        && model.epoch == event.epoch
    {
        mailbox.actor_intake.rcu(|current| {
            current.as_ref().map(|c| {
                if c.session == event.session && c.epoch == event.epoch && c.request == request {
                    Arc::new(ActorIntake {
                        version: image_custody_version,
                        ..(**c).clone()
                    })
                } else {
                    c.clone()
                }
            })
        });
    }
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
            Event::ItemCompleted {
                body: Item::UserMessage { .. } | Item::Notice { .. },
                ..
            } if event
                .cause
                .is_some_and(|op| !owned.values().any(|owner| *owner == op)) =>
            {
                mailbox.pending(event.cause.unwrap(), false);
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
                    body: Item::UserMessage { .. } | Item::Notice { .. },
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
        mailbox.actor_intake.store(None);
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
        let request = OpId::new();
        mailbox.actor_intake.store(Some(Arc::new(ActorIntake {
            session: gateway.session(),
            epoch: sub.hello().epoch,
            request,
            version: 0,
        })));
        // Old actors may acknowledge/reject this unknown control operation. Only
        // their matching live capability response can enable image UserTurns.
        let _ = gateway.send(OpEnvelope {
            v: PROTOCOL,
            op_id: request,
            session: gateway.session(),
            op: Op::QueryInputCapabilities { request },
        });
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
            mailbox.actor_intake.store(None);
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
pub fn machine_profiles(home: Option<std::ffi::OsString>) -> Option<PathBuf> {
    home.filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(".bboxy450-harness/profiles"))
}
async fn load_catalogue(mailbox: &Mailbox, dir: Option<PathBuf>) {
    let (profiles, error) = match dir {
        Some(dir) => profile_catalogue(&dir).await,
        None => (
            vec![],
            Some("Profile catalogue unavailable · HOME is absent".into()),
        ),
    };
    mailbox.profiles.store(Arc::new(profiles));
    mailbox.profile_error.store(error.map(Arc::new));
}
pub async fn profile_names(dir: &std::path::Path) -> Vec<ProfileInfo> {
    profile_catalogue(dir).await.0
}
pub async fn profile_catalogue(dir: &std::path::Path) -> (Vec<ProfileInfo>, Option<String>) {
    let mut out = vec![];
    let mut error = None;
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(e) => {
            return (
                out,
                Some(format!("Profile catalogue unavailable · {}", e.kind())),
            );
        }
    };
    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(e) => {
                error = Some(format!("Profile catalogue incomplete · {}", e.kind()));
                break;
            }
        };
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "md")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            match tokio::fs::read_to_string(&path).await {
                Ok(text) => out.push(ProfileInfo::parse(stem, &text)),
                Err(e) => error = Some(format!("Profile catalogue incomplete · {}", e.kind())),
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if let Some(parent) = dir.parent() {
        match tokio::fs::read_to_string(parent.join("default-profile")).await {
            Ok(name) => {
                if let Some(p) = out.iter_mut().find(|p| p.name == name.trim()) {
                    p.default = true;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => error = Some(format!("Default profile unavailable · {}", e.kind())),
        }
    }
    (out, error)
}

#[cfg(test)]
mod gui_polish_tests {
    use super::*;
    #[tokio::test]
    async fn gui_polish_pending_requires_causal_durable_user_in_current_birth() {
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
        #[cfg(feature = "embedded")]
        {
            assert!(
                !bridge.mailbox.prompts.load().is_empty(),
                "durable arrival must wait for blue row preparation"
            );
            let source = bridge.state.transcript.clone();
            bridge.state.prepared = Some(Arc::new(
                crate::runtime::blocking(move || {
                    super::Preparer::default().prepare(source, 900.).unwrap()
                })
                .await,
            ));
            bridge.drain(Duration::from_millis(1));
        }
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
    #[tokio::test]
    async fn gui_polish_pending_noop_events_preserve_identity_and_notifications() {
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
        #[cfg(feature = "embedded")]
        {
            let source = bridge.state.transcript.clone();
            bridge.state.prepared = Some(Arc::new(
                crate::runtime::blocking(move || {
                    super::Preparer::default().prepare(source, 900.).unwrap()
                })
                .await,
            ));
        }
        bridge.drain(Duration::from_millis(1));
        assert!(bridge.mailbox.prompts.load().is_empty());
        assert!(
            bridge.pending_notices(),
            "retiring confirmed local prompt must notify"
        );
    }
    #[cfg(feature = "acp-agent")]
    #[tokio::test]
    async fn gui_polish_pending_remote_profiles_switch_through_the_admitted_host() {
        tokio::time::timeout(Duration::from_secs(40), async {

        use crate::{
            engine::LaunchOptions,
            gateway::unix::UnixServer,
            mcp::ServerConfig,
            policy::Policy,
            provider::{ScriptedProvider, profile::Unattended},
            redact::Set,
        };
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::PermissionsExt;
        async fn until(
            root: &std::path::Path,
            bridge: &mut Bridge,
            predicate: impl Fn(&FrameState) -> bool,
        ) {
            tokio::time::timeout(Duration::from_secs(18), async {
                loop {
                    bridge.drain(Duration::from_millis(2));
                    if predicate(&bridge.state) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap_or_else(|_| {
                let debug=std::path::Path::new("/tmp/profiles-pending-private-debug");let _=std::fs::create_dir_all(debug);
                for name in ["alpha.jsonl","beta.jsonl","effects"] {let _=std::fs::copy(root.join(name),debug.join(name));}
                if let Ok(files)=std::fs::read_dir(root.join("sessions")){for file in files.flatten(){if file.path().extension().is_some_and(|e|e=="jsonl"){let _=std::fs::copy(file.path(),debug.join(file.file_name()));}}}
                panic!("private remote profile projection did not settle: ready={} profile={:?} cursor={:?} phase={:?} turns={:?} error={:?}", bridge.state.ready, bridge.state.transcript.meta.as_ref().and_then(|m|m.profile.as_ref()),bridge.state.transcript.cursor,bridge.state.transcript.phase,bridge.state.transcript.turns,bridge.state.error) });
        }
        let temp = tempfile::tempdir_in("/tmp").unwrap();
        let profiles = temp.path().join("profiles");
        tokio::fs::create_dir(&profiles).await.unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/acp-agent/launch_parity.py");
        for index in 0..11 {
            let name = if index == 0 {
                "alpha".to_string()
            } else if index == 1 {
                "beta".to_string()
            } else {
                format!("unused-{index}")
            };
            let data = serde_json::json!({"name":name,"provider":"acp-agent","model":if index==1 {"fixture/model-B"}else{"fixture/model-A"},"agent":{"command":["/usr/bin/python3","-I",fixture,"agent",temp.path().join(format!("{name}.jsonl")),name],"env":{"PRIVATE_PROFILE_SECRET":"never show this fixture value"}}});
            tokio::fs::write(
                profiles.join(format!("{name}.md")),
                format!("---\n{data}\n---\n"),
            )
            .await
            .unwrap();
        }
        tokio::fs::write(temp.path().join("default-profile"), "alpha\n")
            .await
            .unwrap();
        let expected = profile_names(&profiles).await;
        assert_eq!(expected.len(), 11);
        assert!(expected.iter().any(|p| p.name == "alpha" && p.default));
        assert!(!format!("{expected:?}").contains("never show this fixture value"));
        // Share this fixture's discovery controller with connect_with_profiles:
        // operator-installed adapters must not seed extra private test profiles.
        let _setup = crate::onboarding::Controller::shared(crate::onboarding::Environment {
            profiles: profiles.clone(),
            home: temp.path().to_path_buf(),
            path: Default::default(),
            os: std::env::consts::OS.into(),
        });
        _setup.refresh_now().await;
        let capability = temp.path().join("private-capability");
        let bytes = format!("{}\n", "f".repeat(64));
        tokio::fs::write(&capability, &bytes).await.unwrap();
        tokio::fs::set_permissions(&capability, std::fs::Permissions::from_mode(0o600))
            .await
            .unwrap();
        let effects = temp.path().join("effects");
        let server = ServerConfig {
            command: "/usr/bin/python3".into(),
            args: vec![
                "-I".into(),
                fixture
                    .with_file_name("profile_switch_server.py")
                    .display()
                    .to_string(),
            ],
            env: [
                (
                    "BBOXY_CAPABILITY_FILE".into(),
                    capability.display().to_string(),
                ),
                ("PARITY_ORIGIN".into(), "fixture-origin:launcher".into()),
                (
                    "PARITY_CAP_HASH".into(),
                    format!("{:x}", Sha256::digest(bytes.as_bytes())),
                ),
                ("PARITY_EFFECTS".into(), effects.display().to_string()),
            ]
            .into(),
            timeout_s: 2,
            trust_hints: false,
        };
        let first: Envelope = serde_json::from_str(
            include_str!("../../../tests/fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let Event::SessionStarted { mut meta } = first.event else {
            panic!("session fixture")
        };
        meta.session = SessionId::new();
        meta.protocol = PROTOCOL;
        meta.cwd = temp.path().to_path_buf();
        meta.profile = Some("alpha".into());
        let context = "NAME: Private catalogue\nROLE: EXECUTOR\nWorkspace: private workspace\nAssignment: private profile switch\nWorkflow: scoped fixture";
        let host = Host::new(Config {
            meta: meta.clone(),
            profiles: profiles.clone(),
            sessions: temp.path().join("sessions"),
            provider: Arc::new(ScriptedProvider::new(vec![])),
            model: "unused".into(),
            profile_error: None,
            policy: Policy::new(Unattended::Deny),
            redactor: Set::default(),
            max_rounds: 2,
        })
        .with_launch(LaunchOptions {
            dream_owner: crate::dream::c1::DreamOwner::Legacy,
            yolo: true,
            append_system_prompt: context.into(),
            mcp_servers: [("seat".into(), server)].into(),
            notices: vec![],
            trial: None,
        });
        let (gateway, _) = host
            .create(OpEnvelope {
                v: PROTOCOL,
                op_id: OpId::new(),
                session: meta.session,
                op: Op::StartSession {
                    cwd: meta.cwd.clone(),
                    profile: Some("alpha".into()),
                    fork: None,
                },
            })
            .await
            .unwrap();
        let (switches, mut requests) = tokio::sync::mpsc::channel(4);
        let gateway = gateway.with_profile_switch(switches.clone());
        let socket = temp.path().join("seat.sock");
        let listener = UnixServer::bind(socket.clone(), gateway.clone())
            .await
            .unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let control = tokio::spawn(async move {
            let request = requests.recv().await.unwrap();
            assert_eq!(request.env.session, gateway.session());
            let Op::SetProfile { profile } = request.env.op else {
                panic!("admitted switch")
            };
            assert_eq!(profile, "beta");
            let next = host
                .switch(&gateway, profile)
                .await
                .unwrap()
                .with_profile_switch(switches);
            listener.retire().await.unwrap();
            let listener = UnixServer::bind(socket, next).await.unwrap();
            let _ = stopped.await;
            host.close_all().await;
            listener.retire().await.unwrap();
        });
        let mut bridge = Bridge::new(Arc::new(TestWake));
        bridge.connect_with_profiles(temp.path().join("seat.sock"), None, Some(profiles));
        until(temp.path(), &mut bridge, |s| {
            s.ready
                && s.transcript
                    .meta
                    .as_ref()
                    .is_some_and(|m| m.profile.as_deref() == Some("alpha"))
        })
        .await;
        assert_eq!(*bridge.state.profiles, expected);
        let send = |bridge: &Bridge| {
            bridge
                .send(Op::UserTurn {
                    blocks: vec![InputBlock::Text {
                        text: "one private prompt".into(),
                    }],
                    mode: TurnMode::Queue,
                })
                .unwrap()
        };
        send(&bridge);
        until(temp.path(), &mut bridge, |s| {
            s.transcript.turns.len() == 1
                && s.transcript
                    .turns
                    .values()
                    .all(|t| matches!(t, super::super::model::TurnState::Ended { .. }))
        })
        .await;
        bridge.switch_profile("beta".into()).unwrap();
        until(temp.path(), &mut bridge, |s| {
            s.ready
                && s.transcript
                    .meta
                    .as_ref()
                    .is_some_and(|m| m.profile.as_deref() == Some("beta"))
        })
        .await;
        assert_ne!(bridge.state.transcript.session, Some(meta.session));
        assert_eq!(*bridge.state.profiles, expected);
        let prior = bridge.state.transcript.turns.len();
        send(&bridge);
        until(temp.path(), &mut bridge, |s| {
            s.transcript.turns.len() > prior
                && s.transcript
                    .turns
                    .values()
                    .all(|t| matches!(t, super::super::model::TurnState::Ended { .. }))
        })
        .await;
        for name in ["alpha", "beta"] {
            let journal = tokio::fs::read_to_string(temp.path().join(format!("{name}.jsonl")))
                .await
                .unwrap();
            assert!(
                journal.contains("NAME: Private catalogue")
                    && journal.contains("ROLE: EXECUTOR")
                    && journal.contains("Assignment: private profile switch"),
                "bootstrap lost on {name}"
            );
            assert!(
                journal.contains(&temp.path().display().to_string()),
                "cwd lost on {name}"
            );
            let wanted = if name == "beta" {
                "fixture/model-B"
            } else {
                "fixture/model-A"
            };
            assert!(
                journal
                    .lines()
                    .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                    .any(|wire| wire["method"] == "session/set_config_option"
                        && wire["params"]["value"] == wanted),
                "model intent lost on {name}"
            );
        }
        let responses: Vec<_> = bridge
            .state
            .transcript
            .items
            .values()
            .filter_map(|row| {
                if let Item::AgentMessage { markdown } = &row.body {
                    serde_json::from_str::<serde_json::Value>(markdown).ok()
                } else {
                    None
                }
            })
            .collect();
        assert!(responses.len() >= 2);
        for response in responses {
            assert_eq!(response["permission"], "yes");
            assert_eq!(response["tools"]["seat"]["seat"], "private-profile-seat");
            assert_eq!(response["tools"]["seat"]["custody"], true);
            assert_eq!(response["tools"]["seat"]["launcher"], true);
        }
        assert_eq!(
            tokio::fs::read_to_string(effects)
                .await
                .unwrap()
                .lines()
                .count(),
            2
        );
        bridge.request_detach();
        stop.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(8),control).await.expect("private host cleanup deadline").unwrap();
        if let Some(worker) = bridge.detach() {
            tokio::time::timeout(Duration::from_secs(4), worker)
                .await
                .unwrap()
                .unwrap();
        }

        }).await.expect("private profile witness deadline");
    }
    struct TestWake;
    impl Waker for TestWake {
        fn wake(&self) {}
    }
}

#[cfg(all(test, feature = "embedded"))]
mod attachment_tests {
    use super::*;
    struct Wake;
    impl Waker for Wake {
        fn wake(&self) {}
    }
    #[tokio::test]
    async fn attachment_unknown_delivery_keeps_real_blocks_and_custody() {
        let temp = tempfile::tempdir().unwrap();
        let custody = crate::runtime::blocking(|| {
            let mut bytes = vec![];
            {
                let mut e = png::Encoder::new(&mut bytes, 1, 1);
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                e.write_header()
                    .unwrap()
                    .write_image_data(&[4, 5, 6, 255])
                    .unwrap();
            }
            Arc::new(crate::store::intake::ImageCustody::create(&bytes, "image/png").unwrap())
        })
        .await;
        let block = custody.block.clone();
        let op = OpId::new();
        let session = SessionId::new();
        let mut prompt = PendingPrompt::new(op, session, Epoch(1), "image".into());
        prompt.blocks = vec![block.clone()];
        prompt.custody = vec![custody.clone()];
        let mut bridge = Bridge::new(Arc::new(Wake));
        bridge.mailbox.prompts(|p| {
            p.insert(op, prompt.clone());
        });
        drop(prompt);
        drop(custody);
        bridge
            .mailbox
            .prompt_status(op, "Outcome unknown · no automatic resend");
        bridge.drain(Duration::from_millis(1));
        let pending = &bridge.state.pending_prompts[&op];
        assert_eq!(pending.blocks, vec![block]);
        assert_eq!(pending.custody.len(), 1);
        assert!(pending.status.contains("unknown"));
        let blobs = crate::store::Blobs {
            dir: temp.path().join("blobs"),
            redactor: Default::default(),
        };
        let admitted = blobs.admit_images(pending.blocks.clone()).await.unwrap();
        assert!(matches!(admitted.as_slice(), [InputBlock::Image { .. }]));
    }
}

fn is_local_image(block: &InputBlock) -> bool {
    matches!(block,InputBlock::Resource {uri,..} if uri.starts_with("bboxy-intake:"))
}

#[cfg(test)]
mod actor_intake_tests {
    use super::*;
    #[test]
    fn image_intake_proof_rejects_durable_replay_old_epoch_and_reconnect_challenge() {
        struct Wake;
        impl Waker for Wake {
            fn wake(&self) {}
        }
        let mut e: Envelope = serde_json::from_str(
            include_str!("../../../tests/fixtures/protocol/all-items.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let Event::SessionStarted { meta } = e.event.clone() else {
            panic!("session");
        };
        let hub = Arc::new(crate::gateway::Hub::new(
            meta,
            e.epoch,
            &[],
            crate::redact::Set::default(),
        ));
        let (work, _work) = tokio::sync::mpsc::channel(1);
        let (control, _control) = tokio::sync::mpsc::channel(1);
        let gateway = InProc::new(hub, work, control, OpId::new());
        let mut bridge = Bridge::new(Arc::new(Wake));
        let mut model = Transcript::default();
        model.hello(e.session, e.epoch);
        let request = OpId::new();
        bridge
            .mailbox
            .actor_intake
            .store(Some(Arc::new(ActorIntake {
                session: e.session,
                epoch: e.epoch,
                request,
                version: 0,
            })));
        e.event = Event::InputCapabilities {
            request,
            image_custody_version: 1,
        };
        e.seq = Some(Seq(1));
        apply_event(&mut model, &e, &mut Default::default(), &bridge.mailbox);
        assert_eq!(
            bridge.mailbox.actor_intake.load().as_ref().unwrap().version,
            0,
            "durable replay is not an actor grant"
        );
        e.seq = None;
        for missing in ["request", "image_custody_version"] {
            let mut wire = serde_json::to_value(&e.event).unwrap();
            wire.as_object_mut().unwrap().remove(missing).unwrap();
            let mut incomplete = e.clone();
            incomplete.event = serde_json::from_value(wire).unwrap();
            assert!(matches!(incomplete.event, Event::InputCapabilities { .. }));
            apply_event(
                &mut model,
                &incomplete,
                &mut Default::default(),
                &bridge.mailbox,
            );
            assert_eq!(
                bridge.mailbox.actor_intake.load().as_ref().unwrap().version,
                0,
                "missing {missing} is not an actor grant"
            );
        }
        e.epoch = Epoch(e.epoch.0 + 1);
        apply_event(&mut model, &e, &mut Default::default(), &bridge.mailbox);
        assert_eq!(
            bridge.mailbox.actor_intake.load().as_ref().unwrap().version,
            0
        );
        e.epoch = model.epoch;
        e.event = Event::InputCapabilities {
            request: OpId::new(),
            image_custody_version: 1,
        };
        apply_event(&mut model, &e, &mut Default::default(), &bridge.mailbox);
        assert_eq!(
            bridge.mailbox.actor_intake.load().as_ref().unwrap().version,
            0
        );
        e.event = Event::InputCapabilities {
            request,
            image_custody_version: 1,
        };
        apply_event(&mut model, &e, &mut Default::default(), &bridge.mailbox);
        bridge.mailbox.publish(FrameState {
            transcript: Arc::new(model.clone()),
            gateway: Some(ViewGateway::InProc(gateway)),
            ready: true,
            ..Default::default()
        });
        bridge.drain(Duration::from_millis(10));
        assert!(bridge.image_intake_available());
        let next = OpId::new();
        bridge
            .mailbox
            .actor_intake
            .store(Some(Arc::new(ActorIntake {
                session: e.session,
                epoch: e.epoch,
                request: next,
                version: 0,
            })));
        assert!(
            !bridge.image_intake_available(),
            "old rendered proof cannot survive reconnect"
        );
        apply_event(&mut model, &e, &mut Default::default(), &bridge.mailbox);
        assert_eq!(
            bridge.mailbox.actor_intake.load().as_ref().unwrap().version,
            0,
            "old same-session response is stale"
        );
    }
}
