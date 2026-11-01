// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! A bounded drain with an injected clock; the same loop serves frames and the
//! deterministic 100k-envelope witness. An item projection is one atomic unit.
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    TryRecv,
    Projection,
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
