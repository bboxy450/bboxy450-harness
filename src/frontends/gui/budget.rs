// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! A bounded UI publication drain with an injected clock. Protocol deltas are
//! projected on the worker; replacing one complete view state is atomic.
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    TryRecv,
    Projection,
    /// A prepared native view's layout/paint projection, separate from drain.
    Render,
}
pub trait Clock {
    fn now(&self) -> Duration;
    /// Deterministic clocks charge the work; the real clock measures it directly.
    fn charge(&self, _work: Work) {}
}
pub struct RealClock(Instant);
impl Default for RealClock {
    fn default() -> Self {
        Self(Instant::now())
    }
}
impl Clock for RealClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}
#[derive(Debug)]
pub struct Drain {
    pub received: usize,
    pub spent: Duration,
}
pub fn drain<T>(
    clock: &impl Clock,
    budget: Duration,
    mut next: impl FnMut() -> Option<T>,
    mut project: impl FnMut(T),
) -> Drain {
    let start = clock.now();
    let mut received = 0;
    while clock.now().saturating_sub(start) < budget {
        let item = next();
        clock.charge(Work::TryRecv);
        let Some(item) = item else {
            break;
        };
        project(item);
        clock.charge(Work::Projection);
        received += 1;
    }
    Drain {
        received,
        spent: clock.now().saturating_sub(start),
    }
}

/// Visit visible views against one shared frame deadline, retaining the cursor
/// at the first deferred view. Each callback owns its bounded work quantum and
/// receives the remaining share; no per-view budget is multiplied by pane count.
/// A returned received count below views.len() tells the host to rearm a frame
/// for the retained tail; cursors belong to that host, never a global gate.
/// Real-clock results measure elapsed frame-path time; injected charges prove the
/// scheduling allocation, not arbitrary renderer execution time.
/// bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
/// Credit reuse accurately: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
pub fn round_robin<T>(
    views: &mut [T],
    cursor: &mut usize,
    clock: &impl Clock,
    budget: Duration,
    mut visit: impl FnMut(&mut T, Duration),
) -> Drain {
    let start = clock.now();
    let mut received = 0;
    for _ in 0..views.len() {
        let spent = clock.now().saturating_sub(start);
        if spent >= budget {
            break;
        }
        let index = *cursor % views.len();
        *cursor = (index + 1) % views.len();
        visit(&mut views[index], budget.saturating_sub(spent));
        received += 1;
    }
    Drain {
        received,
        spent: clock.now().saturating_sub(start),
    }
}
