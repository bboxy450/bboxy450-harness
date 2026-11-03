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
    gateway::{Gateway, Host, InProc, Refused, Subscribe},
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

#[derive(Clone, Default)]
pub struct FrameState {
    pub revision: u64,
    pub transcript: Arc<Transcript>,
    pub gateway: Option<InProc>,
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
struct Mailbox {
    latest: Latest<FrameState>,
    notices: UiSender,
    pending_turns: arc_swap::ArcSwap<im::OrdSet<OpId>>,
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
    fn publish(&self, mut state: FrameState) {
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
        let tx = self.switch.as_ref().ok_or("no live session to switch")?;
        tx.send(profile).map_err(|_| "the session host has stopped")
    }
    pub fn create(&mut self, config: Config) {
        self.start(config, None)
    }
    /// Render another harness's native session live (read-only until `/profile` forks it).
    pub fn watch(&mut self, config: Config, native: Option<crate::import::Native>, path: PathBuf) {
        self.start(config, Some((native, path)))
    }
    fn start(&mut self, config: Config, watch: Option<(Option<crate::import::Native>, PathBuf)>) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        let (switch_tx, mut switches) = tokio::sync::mpsc::unbounded_channel::<String>();
        self.switch = Some(switch_tx);
        #[cfg(feature = "test-fixtures")]
        let before_host = self.before_host.take();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            crate::capability::register(&config.redactor).await;
            let session = config.meta.session;
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
            let redactor = config.redactor.clone();
            mailbox.profiles.store(Arc::new(profile_names(&config.profiles).await));
            let cwd = config.meta.cwd.clone();
            let host = Host::new(config);
            let created = match watch {
                Some((native, path)) => host.watch(native, path, cwd).await.map(|g| (g, crate::gateway::Ack { op_id: OpId::new(), duplicate: false })),
                None => host.create(OpEnvelope { v: PROTOCOL, op_id: OpId::new(), session, op }).await,
            };
            match created {
                Ok((mut gateway, _)) => {
                    loop {
                        tokio::select! {
                            _ = consume(gateway.clone(), mailbox.clone(), detach.clone()) => break,
                            Some(profile) = switches.recv() => match host.switch(&gateway, profile.clone()).await {
                                Ok(next) => {
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
        self.worker = Some(bbt_rt::handle().spawn(consume(gateway, mailbox, detach)));
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
        changed
    }
    pub fn send(&self, op: Op) -> Result<crate::gateway::Ack, Refused> {
        let gateway = self.state.gateway.as_ref().ok_or(Refused::Closed)?;
        let turn = matches!(op, Op::UserTurn { .. });
        let op_id = OpId::new();
        if turn {
            self.mailbox.pending(op_id, true);
        }
        let result = gateway.send(OpEnvelope {
            v: PROTOCOL,
            op_id,
            session: gateway.session(),
            op,
        });
        if turn && result.is_err() {
            self.mailbox.pending(op_id, false);
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
    model.apply(event)
}
async fn consume(gateway: InProc, mailbox: Arc<Mailbox>, detach: CancellationToken) {
    let mut model = Transcript::default();
    let mut after = None;
    let mut detached = false;
    let mut owned_turns = std::collections::BTreeMap::new();
    let mut preparer = Preparer::default();
    loop {
        let mut sub = match gateway.subscribe(Subscribe {
            session: gateway.session(),
            after,
            ephemeral: true,
        }) {
            Ok(sub) => sub,
            Err(error) => {
                mailbox.publish(FrameState {
                    transcript: Arc::new(model),
                    error: Some(format!("Subscription ended: {}", refusal_text(&error))),
                    ..Default::default()
                });
                return;
            }
        };
        sub.set_approver(!detached);
        model.hello(gateway.session(), sub.hello().epoch);
        mailbox.publish(FrameState {
            transcript: Arc::new(model.clone()),
            gateway: Some(gateway.clone()),
            ready: true,
            ..Default::default()
        });
        loop {
            if detached && mailbox.pending_turns.load().is_empty() {
                return;
            }
            let event = tokio::select! {
                biased;
                _ = detach.cancelled(), if !detached => { detached = true; after = Some((model.epoch, model.cursor)); break; }
                _ = mailbox.resized.notified(), if !detached => {
                    publish_frame(FrameState { transcript: Arc::new(model.clone()), gateway: Some(gateway.clone()), ready: true, ..Default::default() }, &mailbox, &mut preparer).await;
                    continue;
                }
                event = sub.recv() => event,
            };
            let Some(event) = event else {
                return;
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
                        ready: true,
                        ..Default::default()
                    },
                    &mailbox,
                    &mut preparer,
                )
                .await;
            }
            if let Effect::Resubscribe { after: resume } = effect {
                after = resume;
                break;
            }
            if closed {
                return;
            }
            tokio::task::yield_now().await;
        }
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
