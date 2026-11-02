// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{protocol::*, redact::Set};
use arc_swap::{ArcSwap, ArcSwapOption};
use crossbeam_queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{Notify, mpsc};
pub mod host;
pub mod unix;
pub use host::Host;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ack {
    pub op_id: OpId,
    pub duplicate: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Refused {
    Busy,
    UnknownSession,
    Version { server: Version },
    Closed,
    SessionExists,
    BadFork,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscribe {
    pub session: SessionId,
    pub after: Option<(Epoch, Seq)>,
    pub ephemeral: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hello {
    pub v: Version,
    pub epoch: Epoch,
    pub head: Seq,
    pub meta: SessionMeta,
}
pub trait Gateway: Send + Sync {
    fn send(&self, op: OpEnvelope) -> Result<Ack, Refused>;
    fn subscribe(&self, req: Subscribe) -> Result<Subscription, Refused>;
}
// Kept inline so draining on the UI thread performs no per-event allocation.
#[allow(clippy::large_enum_variant)]
pub enum TryRecv {
    Envelope(Envelope),
    Empty,
    Closed,
}
struct Wake(Arc<dyn bbt_rt::Waker>);
struct Subscriber {
    queue: ArrayQueue<Envelope>,
    terminal: ArcSwapOption<Envelope>,
    notify: Notify,
    waker: ArcSwapOption<Wake>,
    last: AtomicU64,
    closed: AtomicBool,
    ephemeral: bool,
    approver: AtomicBool,
    drops: AtomicU64,
}
impl Subscriber {
    fn wake(&self) {
        self.notify.notify_one();
        if let Some(w) = self.waker.load_full() {
            w.0.wake();
        }
    }
    fn push(&self, e: Envelope) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        if e.seq.is_none() && self.queue.len() > self.queue.capacity() * 3 / 4 {
            self.drops.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if let Err(e) = self.queue.push(e) {
            if e.seq.is_some() {
                let mut terminal = e;
                terminal.raw = None;
                terminal.seq = None;
                terminal.event = Event::Lagged {
                    resume_after: Seq(self.last.load(Ordering::Acquire)),
                };
                self.terminal.store(Some(Arc::new(terminal)));
                self.closed.store(true, Ordering::Release);
            } else {
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.wake();
    }
}
pub struct Subscription {
    hello: Hello,
    replay: VecDeque<Envelope>,
    history: im::Vector<Envelope>,
    replay_pos: usize,
    snapshots: VecDeque<Envelope>,
    sub: Arc<Subscriber>,
    cutoff: Seq,
    hub: Weak<Hub>,
}
impl Subscription {
    pub fn hello(&self) -> &Hello {
        &self.hello
    }
    pub fn set_waker(&mut self, w: Arc<dyn bbt_rt::Waker>) {
        self.sub.waker.store(Some(Arc::new(Wake(w))));
    }
    pub fn set_approver(&mut self, yes: bool) {
        self.sub.approver.store(yes, Ordering::Release);
    }
    pub fn ephemeral_drops(&self) -> u64 {
        self.sub.drops.load(Ordering::Relaxed)
    }
    pub fn try_recv(&mut self) -> TryRecv {
        let replay = self
            .replay
            .pop_front()
            .or_else(|| {
                let e = self.history.get(self.replay_pos).cloned();
                if e.is_some() {
                    self.replay_pos += 1;
                }
                e
            })
            .or_else(|| self.snapshots.pop_front());
        if let Some(e) = replay {
            if let Some(s) = e.seq {
                self.sub.last.store(s.0, Ordering::Release);
            }
            return TryRecv::Envelope(e);
        }
        while let Some(e) = self.sub.queue.pop() {
            if let Some(s) = e.seq {
                if s <= self.cutoff {
                    continue;
                }
                self.sub.last.store(s.0, Ordering::Release);
            } else if e.at < self.cutoff {
                continue;
            }
            return TryRecv::Envelope(e);
        }
        if let Some(e) = self.sub.terminal.swap(None) {
            let mut e = (*e).clone();
            if let Event::Lagged { resume_after } = &mut e.event {
                *resume_after = Seq(self.sub.last.load(Ordering::Acquire));
            }
            return TryRecv::Envelope(e);
        }
        if self.sub.closed.load(Ordering::Acquire) {
            TryRecv::Closed
        } else {
            TryRecv::Empty
        }
    }
    pub async fn recv(&mut self) -> Option<Envelope> {
        loop {
            let sub = self.sub.clone();
            let notified = sub.notify.notified();
            match self.try_recv() {
                TryRecv::Envelope(e) => return Some(e),
                TryRecv::Closed => return None,
                TryRecv::Empty => notified.await,
            }
        }
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.sub.closed.store(true, Ordering::Release);
        if let Some(h) = self.hub.upgrade() {
            h.subs.rcu(|v| {
                Arc::new(
                    v.iter()
                        .filter(|w| w.upgrade().is_some_and(|s| !Arc::ptr_eq(&s, &self.sub)))
                        .cloned()
                        .collect(),
                )
            });
        }
    }
}
#[derive(Clone)]
pub(crate) struct Projection {
    pub hello: Hello,
    pub log: im::Vector<Envelope>,
    pub open: BTreeMap<ItemId, (Option<TurnId>, Option<ItemId>, Item)>,
    pub closed: bool,
    epochs: im::OrdMap<Epoch, Seq>,
}
pub(crate) struct Hub {
    state: ArcSwap<Projection>,
    subs: ArcSwap<Vec<Weak<Subscriber>>>,
    pub redactor: Set,
    pending: std::sync::Mutex<BTreeMap<ItemId, (Instant, Vec<Delta>)>>,
}
impl Hub {
    pub fn new(meta: SessionMeta, epoch: Epoch, events: &[Envelope], redactor: Set) -> Self {
        let mut p = Projection {
            hello: Hello {
                v: PROTOCOL,
                epoch,
                head: Seq(0),
                meta,
            },
            log: im::Vector::new(),
            epochs: im::OrdMap::new(),
            open: BTreeMap::new(),
            closed: false,
        };
        for e in events {
            Self::reduce(&mut p, e);
        }
        Self {
            state: ArcSwap::from_pointee(p),
            subs: ArcSwap::from_pointee(vec![]),
            redactor,
            pending: std::sync::Mutex::new(BTreeMap::new()),
        }
    }
    fn reduce(p: &mut Projection, e: &Envelope) {
        p.hello.epoch = e.epoch;
        if let Some(seq) = e.seq {
            p.hello.head = seq;
            p.epochs.entry(e.epoch).or_insert(seq);
            p.log.push_back(e.clone());
        }
        match &e.event {
            Event::ItemStarted {
                turn,
                item,
                kind,
                parent,
                head,
            } => {
                let mut partial = Item::interrupted_from_head(kind.clone(), *item, head.as_ref());
                if let Item::ToolCall { output, .. } = &mut partial {
                    output.clear();
                }
                p.open.insert(*item, (*turn, *parent, partial));
            }
            Event::ItemDelta { item, delta } => {
                if let Some((_, _, body)) = p.open.get_mut(item) {
                    body.apply(delta);
                }
            }
            Event::ItemSnapshot { item, partial } => {
                if let Some((_, _, body)) = p.open.get_mut(item) {
                    *body = partial.clone();
                }
            }
            Event::ItemCompleted { item, .. } => {
                p.open.remove(item);
            }
            Event::SessionClosed => p.closed = true,
            _ => {}
        }
    }
    pub fn publish(&self, e: Envelope) {
        let e = if e.raw.is_some() {
            e
        } else {
            self.redactor.sanitize(&e).expect("envelope sanitizes")
        };
        let mut pending = self.pending.lock().unwrap();
        if let Event::ItemDelta { item, delta } = &e.event {
            pending
                .entry(*item)
                .or_insert_with(|| (Instant::now(), vec![]))
                .1
                .push(delta.clone());
            return;
        }
        let mut state = (**self.state.load()).clone();
        Self::reduce(&mut state, &e);
        self.state.store(Arc::new(state));
        if let Event::ItemCompleted { item, .. } = e.event {
            pending.remove(&item);
        }
        drop(pending);
        for sub in self.subs.load().iter().filter_map(Weak::upgrade) {
            if e.seq.is_some() || sub.ephemeral {
                sub.push(e.clone());
            }
        }
    }
    pub fn flush_deltas(&self) {
        let mut pending = self.pending.lock().unwrap();
        let ready = pending
            .iter()
            .filter_map(|(i, (at, _))| (at.elapsed() >= Duration::from_millis(50)).then_some(*i))
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return;
        }
        // Copy the growing body once per coalescing interval, never per token.
        let mut state = (**self.state.load()).clone();
        let mut envelopes = vec![];
        for item in ready {
            let (_, deltas) = pending.remove(&item).unwrap();
            if let Some((_, _, body)) = state.open.get_mut(&item) {
                for delta in deltas {
                    body.apply(&delta);
                }
                let e = Envelope {
                    raw: None,
                    v: PROTOCOL,
                    session: state.hello.meta.session,
                    epoch: state.hello.epoch,
                    seq: None,
                    at: state.hello.head,
                    ts_ms: crate::store::now_ms(),
                    cause: None,
                    event: Event::ItemSnapshot {
                        item,
                        partial: self.redactor.sanitize(body).expect("snapshot redacts"),
                    },
                };
                envelopes.push(e);
            }
        }
        self.state.store(Arc::new(state));
        drop(pending);
        for e in envelopes {
            for sub in self
                .subs
                .load()
                .iter()
                .filter_map(Weak::upgrade)
                .filter(|s| s.ephemeral)
            {
                sub.push(e.clone());
            }
        }
    }
    pub fn approvers(&self) -> usize {
        self.subs
            .load()
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|s| !s.closed.load(Ordering::Acquire) && s.approver.load(Ordering::Acquire))
            .count()
    }
    pub fn close(&self) {
        for s in self.subs.load().iter().filter_map(Weak::upgrade) {
            s.closed.store(true, Ordering::Release);
            s.wake();
        }
    }
    pub fn projection(&self) -> Arc<Projection> {
        let pending = self.pending.lock().unwrap();
        let mut state = (**self.state.load()).clone();
        for (item, (_, deltas)) in pending.iter() {
            if let Some((_, _, body)) = state.open.get_mut(item) {
                for delta in deltas {
                    body.apply(delta);
                }
            }
        }
        Arc::new(state)
    }
}
pub(crate) struct Admitted {
    pub env: OpEnvelope,
    pub face: &'static str,
}
#[derive(Clone)]
pub struct InProc {
    pub(crate) hub: Arc<Hub>,
    work: mpsc::Sender<Admitted>,
    control: mpsc::Sender<Admitted>,
    recent: Arc<ArcSwap<Vec<(OpId, Ack)>>>,
    closed: Arc<AtomicBool>,
    faulted: Arc<AtomicBool>,
    face: &'static str,
    unattended: Arc<AtomicBool>,
    notices: Option<mpsc::Sender<crate::engine::Internal>>,
}
impl InProc {
    pub(crate) fn new(
        hub: Arc<Hub>,
        work: mpsc::Sender<Admitted>,
        control: mpsc::Sender<Admitted>,
        start: OpId,
    ) -> Self {
        Self {
            hub,
            work,
            control,
            recent: Arc::new(ArcSwap::from_pointee(vec![(
                start,
                Ack {
                    op_id: start,
                    duplicate: false,
                },
            )])),
            closed: Arc::new(AtomicBool::new(false)),
            faulted: Arc::new(AtomicBool::new(false)),
            face: "gateway",
            unattended: Arc::new(AtomicBool::new(false)),
            notices: None,
        }
    }
    pub(crate) fn set_unattended(&self, value: crate::provider::profile::Unattended) {
        self.unattended.store(
            matches!(value, crate::provider::profile::Unattended::Allow),
            Ordering::Release,
        );
    }
    #[cfg(feature = "acp")]
    pub(crate) fn unattended(&self) -> crate::provider::profile::Unattended {
        if self.unattended.load(Ordering::Acquire) {
            crate::provider::profile::Unattended::Allow
        } else {
            crate::provider::profile::Unattended::Deny
        }
    }
    pub(crate) fn with_notices(mut self, tx: mpsc::Sender<crate::engine::Internal>) -> Self {
        self.notices = Some(tx);
        self
    }
    #[cfg(feature = "acp")]
    pub(crate) async fn notice(&self, level: Level, text: String) {
        if let Some(tx) = &self.notices {
            let _ = tx
                .send(crate::engine::Internal::ServiceNotice { level, text })
                .await;
        }
    }
    #[cfg(feature = "acp")]
    pub(crate) fn for_face(&self, face: &'static str) -> Self {
        let mut gateway = self.clone();
        gateway.face = face;
        gateway
    }
    pub fn session(&self) -> SessionId {
        self.hub.state.load().hello.meta.session
    }
    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire) || self.hub.state.load().closed
    }
    pub fn hello(&self) -> Hello {
        self.hub.state.load().hello.clone()
    }
    pub fn subscribe_version(&self, req: Subscribe, v: Version) -> Result<Subscription, Refused> {
        if !v.is_readable() {
            return Err(Refused::Version { server: PROTOCOL });
        }
        self.subscribe(req)
    }
    pub(crate) fn mark_faulted(&self) {
        self.faulted.store(true, Ordering::Release);
    }
    pub(crate) fn seal(&self) {
        self.closed.store(true, Ordering::Release);
    }
    pub(crate) fn mark_closed(&self) {
        self.closed.store(true, Ordering::Release);
        self.hub.close();
    }
}
impl Gateway for InProc {
    fn send(&self, op: OpEnvelope) -> Result<Ack, Refused> {
        if !op.v.is_readable() {
            return Err(Refused::Version { server: PROTOCOL });
        }
        if op.session != self.hub.state.load().hello.meta.session {
            return Err(Refused::UnknownSession);
        }
        if self.closed.load(Ordering::Acquire) || self.hub.state.load().closed {
            return Err(Refused::Closed);
        }
        if matches!(op.op, Op::StartSession { .. }) {
            return Err(Refused::SessionExists);
        }
        loop {
            let old = self.recent.load_full();
            if let Some((_, ack)) = old.iter().find(|(id, _)| *id == op.op_id) {
                let mut ack = ack.clone();
                ack.duplicate = true;
                return Ok(ack);
            }
            if matches!(op.op, Op::StartSession { .. }) {
                return Err(Refused::SessionExists);
            }
            let lane = if matches!(
                op.op,
                Op::Interrupt { .. } | Op::ResolveApproval { .. } | Op::Close
            ) {
                &self.control
            } else {
                &self.work
            };
            let permit = lane.try_reserve().map_err(|e| {
                if matches!(e, mpsc::error::TrySendError::Closed(_)) {
                    Refused::Closed
                } else {
                    Refused::Busy
                }
            })?;
            let ack = Ack {
                op_id: op.op_id,
                duplicate: false,
            };
            let mut next = (*old).clone();
            next.push((op.op_id, ack.clone()));
            if next.len() > 1024 {
                next.remove(0);
            }
            let prev = self.recent.compare_and_swap(&old, Arc::new(next));
            if Arc::ptr_eq(&prev, &old) {
                permit.send(Admitted {
                    env: op,
                    face: self.face,
                });
                return Ok(ack);
            }
        }
    }
    fn subscribe(&self, req: Subscribe) -> Result<Subscription, Refused> {
        if req.session != self.hub.state.load().hello.meta.session {
            return Err(Refused::UnknownSession);
        }
        let sub = Arc::new(Subscriber {
            queue: ArrayQueue::new(4096),
            terminal: ArcSwapOption::empty(),
            notify: Notify::new(),
            waker: ArcSwapOption::empty(),
            last: AtomicU64::new(req.after.map(|(_, s)| s.0).unwrap_or(0)),
            closed: AtomicBool::new(false),
            ephemeral: req.ephemeral,
            approver: AtomicBool::new(false),
            drops: AtomicU64::new(0),
        });
        self.hub.subs.rcu(|v| {
            let mut next = v
                .iter()
                .filter(|w| w.strong_count() > 0)
                .cloned()
                .collect::<Vec<_>>();
            next.push(Arc::downgrade(&sub));
            Arc::new(next)
        });
        // Registration precedes the atomic projection read. Live seqs at/below this
        // captured head are filtered at receive, joining replay and live exactly once.
        let state = self.hub.projection();
        let after = req.after.map(|(_, s)| s).unwrap_or(Seq(0));
        let hello = state.hello.clone();
        let mut replay = VecDeque::new();
        let mut snapshots = VecDeque::new();
        let trusted = req
            .after
            .map(|(e, _)| {
                if e > hello.epoch {
                    return Seq(0);
                }
                state
                    .epochs
                    .range((std::ops::Bound::Excluded(e), std::ops::Bound::Unbounded))
                    .next()
                    .map(|(_, seq)| Seq(seq.0.saturating_sub(1)))
                    .unwrap_or(hello.head)
                    .min(hello.head)
            })
            .unwrap_or(hello.head);
        if self.faulted.load(Ordering::Acquire) || req.after.is_some_and(|(_, s)| s > trusted) {
            let e = Envelope {
                raw: None,
                v: PROTOCOL,
                session: req.session,
                epoch: hello.epoch,
                seq: None,
                at: hello.head,
                ts_ms: crate::store::now_ms(),
                cause: None,
                event: Event::Reset {
                    epoch: hello.epoch,
                    head: trusted,
                },
            };
            replay.push_back(e);
            sub.closed.store(true, Ordering::Release);
        } else {
            if req.ephemeral {
                for (&item, (_, _, body)) in &state.open {
                    snapshots.push_back(Envelope {
                        raw: None,
                        v: PROTOCOL,
                        session: req.session,
                        epoch: hello.epoch,
                        seq: None,
                        at: hello.head,
                        ts_ms: crate::store::now_ms(),
                        cause: None,
                        event: Event::ItemSnapshot {
                            item,
                            partial: self.hub.redactor.sanitize(body).expect("snapshot redacts"),
                        },
                    });
                }
            }
            if state.closed {
                sub.closed.store(true, Ordering::Release);
            }
        }
        let history = if replay.is_empty() {
            state.log.clone()
        } else {
            im::Vector::new()
        };
        Ok(Subscription {
            history,
            replay_pos: after.0.min(hello.head.0) as usize,
            cutoff: hello.head,
            hello,
            replay,
            snapshots,
            sub,
            hub: Arc::downgrade(&self.hub),
        })
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    #[test]
    fn held_capacity_returns_busy_without_an_actor_or_runtime() {
        let session = SessionId::new();
        let meta = SessionMeta {
            session,
            cwd: std::path::PathBuf::new(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: 0,
        };
        let hub = Arc::new(Hub::new(meta, Epoch(1), &[], Set::default()));
        let (work, mut work_rx) = mpsc::channel(1);
        let (control, mut control_rx) = mpsc::channel(256);
        let held = work.try_reserve().unwrap();
        let g = InProc::new(hub, work.clone(), control, OpId::new());
        let op = |op| OpEnvelope {
            v: PROTOCOL,
            session,
            op_id: OpId::new(),
            op,
        };
        let input = op(Op::UserTurn {
            blocks: vec![InputBlock::Text { text: "x".into() }],
            mode: TurnMode::Queue,
        });
        let send: fn(&InProc, OpEnvelope) -> Result<Ack, Refused> = Gateway::send;
        assert_eq!(send(&g, input.clone()), Err(Refused::Busy));
        // Independent control admission works while the work reservation is held.
        let interrupt = op(Op::Interrupt { turn: None });
        send(&g, interrupt.clone()).unwrap();
        assert_eq!(control_rx.try_recv().unwrap().env.op_id, interrupt.op_id);
        drop(held);
        send(&g, input.clone()).unwrap();
        assert_eq!(work_rx.try_recv().unwrap().env.op_id, input.op_id);
        let sub: Result<Subscription, Refused> = g.subscribe(Subscribe {
            session,
            after: None,
            ephemeral: false,
        });
        assert_eq!(sub.unwrap().hello().meta.session, session);
    }
}
