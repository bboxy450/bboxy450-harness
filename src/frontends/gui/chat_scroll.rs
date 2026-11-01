// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

// Adapted from bboxy450-term crates/bboxy450-term/src/chat_scroll.rs.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
//! One session-bound transcript. Manual intent survives growth/reflow; a burst
//! accumulates against its pending target until the next solved layout.
#[derive(Debug)]
pub struct State {
    requested: f32,
    following: bool,
    pending: Option<f32>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            requested: 0.,
            following: true,
            pending: None,
        }
    }
}
impl State {
    pub fn following(&self) -> bool {
        self.following
    }
    pub fn layout_offset(&self) -> f32 {
        if self.following {
            f32::MAX
        } else {
            self.requested
        }
    }
    pub fn layout_solved(&mut self) {
        self.pending = None;
    }
    pub fn wheel(&mut self, actual: f32, max: f32, dy: f32) -> bool {
        if !actual.is_finite() || !max.is_finite() || !dy.is_finite() || dy == 0. {
            return false;
        }
        let max = max.max(0.);
        let before = self.pending.unwrap_or(actual).clamp(0., max);
        let offset = (before + dy).clamp(0., max);
        self.pending = Some(offset);
        let following = offset == max;
        let changed = before != offset || self.following != following;
        if changed {
            self.requested = offset;
            self.following = following;
        }
        changed
    }
    pub fn follow_latest(&mut self) {
        *self = Self::default();
    }
}
