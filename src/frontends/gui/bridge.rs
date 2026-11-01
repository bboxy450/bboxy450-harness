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
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub struct FrameState {
    pub transcript: Arc<Transcript>,
    pub gateway: Option<InProc>,
    pub error: Option<String>,
    pub replay: bool,
    pub ready: bool,
    #[cfg(feature = "embedded")]
    pub prepared: Option<Arc<super::view::PreparedRows>>,
}
struct Mailbox {
    latest: Latest<FrameState>,
    notices: UiSender,
    pending_turns: arc_swap::ArcSwap<im::OrdSet<OpId>>,
    width: AtomicU32,
    resized: tokio::sync::Notify,
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
    fn publish(&self, state: FrameState) {
        self.latest.publish(state);
        // A saturated notification queue is harmless: Latest holds the complete
        // current state and its own dirty source, including all durable items.
        let _ = self.notices.try_send(UiMsg::Changed { source: 0 });
    }
}
pub struct Bridge {
    mailbox: Arc<Mailbox>,
    messages: UiReceiver,
    pub dirty: DirtySet,
    pub state: FrameState,
    detach: CancellationToken,
    worker: Option<tokio::task::JoinHandle<()>>,
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
                resized: tokio::sync::Notify::new(),
            }),
            messages,
            dirty,
            state: FrameState::default(),
            detach: CancellationToken::new(),
            worker: None,
        }
    }
    /// IO is scheduled, never awaited by the caller/frame owner.
    pub fn replay(&mut self, path: PathBuf) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            match read_replay(path).await {
                Ok(transcript) => {
                    let state = FrameState { transcript: Arc::new(transcript), replay: true, ready: true, ..Default::default() };
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
    pub fn create(&mut self, config: Config) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker = Some(bbt_rt::handle().spawn(async move {
            let session = config.meta.session;
            let op = Op::StartSession {
                cwd: config.meta.cwd.clone(),
                profile: config.meta.profile.clone(),
                fork: None,
            };
            let host = Host::new(config);
            match host
                .create(OpEnvelope {
                    v: PROTOCOL,
                    op_id: OpId::new(),
                    session,
                    op,
                })
                .await
            {
                Ok((gateway, _)) => {
                    consume(gateway, mailbox.clone(), detach).await;
                    // Keep the actor host alive through detached work. A detach
                    // sends no Close. The idle log remains resumable on disk.
                    drop(host);
                }
                Err(error) => mailbox.publish(FrameState {
                    error: Some(format!("Session could not start: {error:?}")),
                    ..Default::default()
                }),
            }
        }));
    }
    pub fn attach(&mut self, gateway: InProc) {
        let mailbox = self.mailbox.clone();
        let detach = self.detach.clone();
        self.worker = Some(bbt_rt::handle().spawn(consume(gateway, mailbox, detach)));
    }
    /// A real frame drains under 0.3ms. complete_drain re-marks any remainder,
    /// including a publication racing the just-completed frame.
    pub fn drain(&mut self, budget: Duration) -> bool {
        self.dirty.drain();
        let start = Instant::now();
        let mut changed = false;
        if let Some(state) = self.mailbox.latest.take() {
            self.state = state;
            changed = true;
        }
        while start.elapsed() < budget && self.messages.try_recv().is_some() {}
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
struct Preparer;
async fn publish_frame(mut state: FrameState, mailbox: &Mailbox, preparer: &mut Preparer) {
    #[cfg(feature = "embedded")]
    {
        let mut owned = std::mem::take(preparer);
        let source = state.transcript.clone();
        let width = f32::from_bits(mailbox.width.load(Ordering::Acquire));
        let (next, result) = crate::runtime::blocking(move || {
            let result = owned.prepare(source, width);
            (owned, result)
        })
        .await;
        *preparer = next;
        match result {
            Ok(prepared) => state.prepared = Some(Arc::new(prepared)),
            Err(error) => state.error = Some(error),
        }
    }
    #[cfg(not(feature = "embedded"))]
    {
        let _ = preparer;
        let _ = &mut state;
    }
    mailbox.publish(state);
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
                    error: Some(format!("Subscription ended: {error:?}")),
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
#[allow(
    clippy::disallowed_methods,
    reason = "file reading and projection run inside runtime::blocking, never a frame"
)]
pub async fn read_replay(path: PathBuf) -> Result<Transcript, String> {
    crate::runtime::blocking(move || {
        let bytes = std::fs::read(path).map_err(|_| "Replay file unavailable".to_string())?;
        let events = bytes
            .split(|c| *c == b'\n')
            .filter(|line| !line.is_empty())
            .map(serde_json::from_slice)
            .collect::<Result<Vec<Envelope>, _>>()
            .map_err(|_| "Invalid replay JSONL".to_string())?;
        crate::protocol::laws::check_transcript(&events).map_err(|e| e.to_string())?;
        let mut model = Transcript::default();
        for e in &events {
            if let Effect::Resubscribe { after: None } = model.apply(e) {
                // Offline Reset rebuilds from the surviving durable prefix.
                let Event::Reset { head, .. } = e.event else {
                    unreachable!()
                };
                for kept in events
                    .iter()
                    .filter(|v| v.seq.is_some_and(|seq| seq <= head))
                {
                    model.apply(kept);
                }
            }
        }
        Ok(model)
    })
    .await
}
