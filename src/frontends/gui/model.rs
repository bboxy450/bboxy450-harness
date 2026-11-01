// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Pure, retained projection of the protocol. No UI or runtime dependency.
//!
//! A subscription is ordered and delivers each ephemeral envelope once. The wire
//! has no delta ID: identical adjacent text deltas can be legitimate, and cannot
//! be deduplicated by content. Durable seqs are deduplicated defensively. The
//! bridge owns exactly one subscription and publishes complete projection values,
//! so replacing/coalescing UI notifications cannot replay an individual delta.
use crate::protocol::*;
use im::{OrdMap, Vector};

#[derive(Clone, Debug, PartialEq)]
pub struct ViewItem {
    pub id: ItemId,
    pub turn: Option<TurnId>,
    pub parent: Option<ItemId>,
    pub started: Seq,
    pub completed: Option<Seq>,
    pub kind: ItemKind,
    pub body: Item,
    pub status: Option<ItemStatus>,
    /// Transient terminal output; the completed body's blob is authoritative.
    pub terminal_tail: String,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TurnState {
    Queued,
    Running,
    Ended {
        reason: StopReason,
        usage: Option<Usage>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    Resubscribe { after: Option<(Epoch, Seq)> },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Transcript {
    pub session: Option<SessionId>,
    pub epoch: Epoch,
    pub cursor: Seq,
    pub meta: Option<SessionMeta>,
    pub profile: Option<(String, String)>,
    pub items: OrdMap<ItemId, ViewItem>,
    pub order: Vector<ItemId>,
    pub turns: OrdMap<TurnId, TurnState>,
    pub phase: Phase,
    pub closed: bool,
    pub unknown: u64,
    pub duplicate_durable: u64,
}
impl Default for Transcript {
    fn default() -> Self {
        Self {
            session: None,
            epoch: Epoch(0),
            cursor: Seq(0),
            meta: None,
            profile: None,
            items: OrdMap::new(),
            order: Vector::new(),
            turns: OrdMap::new(),
            phase: Phase::Idle,
            closed: false,
            unknown: 0,
            duplicate_durable: 0,
        }
    }
}
impl Transcript {
    pub fn hello(&mut self, session: SessionId, epoch: Epoch) {
        if self.session.is_some_and(|s| s != session) {
            *self = Self::default();
        }
        self.session = Some(session);
        self.epoch = epoch;
    }
    pub fn apply(&mut self, e: &Envelope) -> Effect {
        if e.v.major != PROTOCOL.major || self.session.is_some_and(|s| s != e.session) {
            return Effect::None;
        }
        self.session = Some(e.session);
        if let Event::Reset { epoch, .. } = e.event {
            if epoch < self.epoch {
                return Effect::None;
            }
            *self = Self {
                session: Some(e.session),
                epoch,
                ..Self::default()
            };
            return Effect::Resubscribe { after: None };
        }
        if let Some(seq) = e.seq {
            if seq <= self.cursor {
                self.duplicate_durable += 1;
                return Effect::None;
            }
            self.cursor = seq;
            self.epoch = self.epoch.max(e.epoch);
        } else if e.epoch != self.epoch {
            return Effect::None;
        }
        match &e.event {
            Event::SessionStarted { meta } => self.meta = Some(meta.clone()),
            Event::ProfileChanged { profile, model } => {
                self.profile = Some((profile.clone(), model.clone()))
            }
            Event::TurnQueued { turn } => {
                self.turns.insert(*turn, TurnState::Queued);
            }
            Event::TurnStarted { turn } => {
                self.turns.insert(*turn, TurnState::Running);
            }
            Event::TurnEnded {
                turn,
                reason,
                usage,
            } => {
                self.turns.insert(
                    *turn,
                    TurnState::Ended {
                        reason: reason.clone(),
                        usage: usage.clone(),
                    },
                );
            }
            Event::ItemStarted {
                turn,
                item,
                kind,
                parent,
            } => {
                // IDs are never reused. A reconnect cannot reopen a completed item.
                if !self.items.contains_key(item) {
                    self.order.push_back(*item);
                    self.items.insert(
                        *item,
                        ViewItem {
                            id: *item,
                            turn: *turn,
                            parent: *parent,
                            started: e.seq.unwrap_or(e.at),
                            completed: None,
                            kind: *kind,
                            body: empty_body(*kind, *item),
                            status: None,
                            terminal_tail: String::new(),
                        },
                    );
                }
            }
            Event::ItemCompleted { item, status, body } => {
                if let Some(row) = self.items.get_mut(item) {
                    row.body = body.clone();
                    row.status = Some(status.clone());
                    row.completed = e.seq;
                    row.terminal_tail.clear();
                }
            }
            Event::ItemDelta { item, delta } => {
                if let Some(row) = self.open_at(*item, e.at) {
                    if let (Item::Terminal { .. }, Delta::Output { append, .. }) =
                        (&row.body, delta)
                    {
                        row.terminal_tail.push_str(append);
                    } else {
                        row.body.apply(delta);
                    }
                }
            }
            Event::ItemSnapshot { item, partial } => {
                if let Some(row) = self.open_at(*item, e.at)
                    && partial.kind() == row.kind
                {
                    row.body = partial.clone();
                    row.terminal_tail.clear();
                }
            }
            Event::Status { phase } => self.phase = phase.clone(),
            Event::Lagged { resume_after } => {
                return Effect::Resubscribe {
                    after: Some((self.epoch, *resume_after)),
                };
            }
            Event::SessionClosed => {
                self.closed = true;
                self.phase = Phase::Closed;
            }
            Event::Unknown => self.unknown += 1,
            Event::SessionRecovered { .. } | Event::OpRejected { .. } | Event::Reset { .. } => {}
        }
        Effect::None
    }
    fn open_at(&mut self, item: ItemId, at: Seq) -> Option<&mut ViewItem> {
        self.items
            .get_mut(&item)
            .filter(|row| row.status.is_none() && at >= row.started)
    }
}
fn empty_body(kind: ItemKind, id: ItemId) -> Item {
    // Reuse the protocol's kind-complete constructor, stripping recovery-only
    // placeholders. Starting an item is not an interrupted/failed completion.
    let mut body = Item::interrupted(kind, id);
    match &mut body {
        Item::ToolCall {
            name,
            input_raw,
            output,
            ..
        } => {
            name.clear();
            input_raw.clear();
            output.clear();
        }
        Item::Approval {
            subject,
            decision,
            by,
            options,
        } => {
            subject.summary = "Approval pending".into();
            *decision = None;
            *by = None;
            // The ItemSnapshot supplies the actual subject and offered options.
            options.clear();
        }
        Item::Notice { text, .. } => text.clear(),
        _ => {}
    }
    body
}
