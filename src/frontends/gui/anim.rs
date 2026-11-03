// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! The window's one animation clock (harness-spec.md "Motion", "Clock rules").
//!
//! Every value that moves on screen reads through this clock. A face builder asks for a
//! value by element id every frame; the clock answers with the value *now* and remembers
//! where the value is going. Nothing else keeps time.
//!
//! # API for face builders
//!
//! ```ignore
//! let a: &mut Clock = &mut view.anim;             // one per window, ticked by the host
//! a.tween("composer/focus", 1., 200., Ease::Std)   // -> f32 now; retargets from the current value
//! a.enter("block/42", 0., 1., 320., Ease::Out)     // first sight starts at `from`; then as `tween`
//! a.enter_after("chip/1", 0., 1., 420., 60., Ease::Out) // the same, delayed 60 ms
//! a.spring("scroll", max, 180.)                    // critically damped glide; retargets mid-glide
//! a.spring_from("scroll", here, max, 180.)         // the same, first sight starts at `here`
//! a.cycle(3200.)                                   // 0..1 phase of a loop; keeps the clock alive
//! a.blink(1000., since_edit_ms)                    // true while the caret is on; the only loop
//!                                                  //   that survives reduced motion
//! a.snap("h/42", 120.)                             // set without motion (also forgets history)
//! a.animates()                                     // false: settled, reduced, flat or paused;
//!                                                  //   builders then draw the flat column
//! ```
//!
//! Frame protocol, in order, once per frame:
//! 1. the host calls [`Clock::tick`] with the frame time (the window: its uptime in ms;
//!    tests step it by hand; the clock holds no `Instant`);
//! 2. the face builders ask for their values (above) while the frame is built;
//! 3. the view calls [`Clock::end_frame`]: ids nobody touched are forgotten, the loop count
//!    resets, and [`Clock::next_frame`] is decided;
//! 4. the host reads [`Clock::next_frame`]: `Some(0.)` means request the next frame now,
//!    `Some(ms)` means wake in `ms` (a caret blink edge), `None` means idle, no frame;
//! 5. the host reports the frame's cost with [`Clock::frame_cost`].
//!
//! Rules the clock enforces:
//! - A tween is `(from, to, t0, duration, curve)` keyed by id. A new target mid-flight
//!   retargets from the current value; it never snaps back to `from`.
//! - An element that leaves the screen (offscreen block, closed menu) is forgotten at
//!   `end_frame` and takes its end state when it returns.
//! - [`Clock::settled`] (the default, used by tests, captures and the parity gate) and reduced
//!   motion (`BBOXY_REDUCED_MOTION=1` or the OS setting, passed to [`Clock::live`]) answer end
//!   states at once; only [`Clock::blink`] still runs under reduced motion.
//! - Two frames in a row over the 16.6 ms budget fall back to end states until the clock is
//!   next idle ([`Clock::flat`]).
//! - [`Clock::in_motion`] is true after a frame that moved something; the view's frame cache
//!   must rebuild then even when nothing else changed.
//!
//! Curves (CSS cubic-bezier, solved on x): `out` (.2,.8,.2,1) for entrances and glides,
//! `std` (.4,0,.2,1) for colour and state, `pop` (.34,1.56,.64,1) for overshoot,
//! `in_out` (.42,0,.58,1) for loops.
use std::collections::HashMap;

/// The 60 fps frame budget the spec holds at 1000 blocks.
pub const FRAME_BUDGET_MS: f32 = 16.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ease {
    /// cubic-bezier(.2,.8,.2,1): entrances and glides.
    Out,
    /// cubic-bezier(.4,0,.2,1): colour and state changes.
    Std,
    /// cubic-bezier(.34,1.56,.64,1): overshoot.
    Pop,
    /// cubic-bezier(.42,0,.58,1): the loops' ease-in-out.
    InOut,
    Linear,
}
impl Ease {
    fn control(self) -> [f32; 4] {
        match self {
            Ease::Out => [0.2, 0.8, 0.2, 1.],
            Ease::Std => [0.4, 0., 0.2, 1.],
            Ease::Pop => [0.34, 1.56, 0.64, 1.],
            Ease::InOut => [0.42, 0., 0.58, 1.],
            Ease::Linear => [0., 0., 1., 1.],
        }
    }
    /// The curve's y at progress `t` in 0..=1 (CSS semantics: x is time, y is value).
    pub fn at(self, t: f32) -> f32 {
        if t <= 0. {
            return 0.;
        }
        if t >= 1. {
            return 1.;
        }
        if self == Ease::Linear {
            return t;
        }
        let [x1, y1, x2, y2] = self.control();
        let bez = |a: f32, b: f32, s: f32| {
            // one-dimensional cubic Bézier from 0 through a, b to 1
            let u = 1. - s;
            3. * u * u * s * a + 3. * u * s * s * b + s * s * s
        };
        // Newton on x with a bisection fallback, as browsers do.
        let mut s = t;
        for _ in 0..8 {
            let x = bez(x1, x2, s) - t;
            if x.abs() < 1e-5 {
                return bez(y1, y2, s);
            }
            let u = 1. - s;
            let dx = 3. * u * u * x1 + 6. * u * s * (x2 - x1) + 3. * s * s * (1. - x2);
            if dx.abs() < 1e-6 {
                break;
            }
            s -= x / dx;
        }
        let (mut lo, mut hi) = (0f32, 1f32);
        s = t;
        for _ in 0..24 {
            let x = bez(x1, x2, s);
            if (x - t).abs() < 1e-5 {
                break;
            }
            if x < t {
                lo = s;
            } else {
                hi = s;
            }
            s = (lo + hi) / 2.;
        }
        bez(y1, y2, s)
    }
}

#[derive(Clone, Copy, Debug)]
struct Tween {
    from: f32,
    to: f32,
    t0: f32,
    ms: f32,
    ease: Ease,
    touched: bool,
}
impl Tween {
    fn at(&self, now: f32) -> f32 {
        if self.ms <= 0. {
            return self.to;
        }
        let t = ((now - self.t0) / self.ms).clamp(0., 1.);
        self.from + (self.to - self.from) * self.ease.at(t)
    }
    fn done(&self, now: f32) -> bool {
        self.ms <= 0. || now - self.t0 >= self.ms
    }
}
#[derive(Clone, Copy, Debug)]
struct Spring {
    x: f32,
    v: f32,
    target: f32,
    last: f32,
    touched: bool,
}
impl Spring {
    /// Settled: the web's `|gap| < .5 && |v| < 4` (px, px/s).
    fn done(&self) -> bool {
        (self.target - self.x).abs() < 0.5 && self.v.abs() < 4.
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// End states at once, no frames: tests, captures, the parity gate.
    Settled,
    /// Animate.
    Live,
    /// The OS or `BBOXY_REDUCED_MOTION=1`: end states; the caret still blinks.
    Reduced,
}

/// One per window. See the module header.
#[derive(Debug)]
pub struct Clock {
    mode: Mode,
    now: f32,
    tweens: HashMap<String, Tween>,
    springs: HashMap<String, Spring>,
    /// Loops asked for this frame (cleared by `end_frame`); the clock stays awake for them.
    loops: usize,
    /// The nearest caret blink edge asked for this frame, in ms from now.
    blink_edge: Option<f32>,
    /// What the last `end_frame` decided; see `next_frame`.
    next: Option<f32>,
    /// Consecutive frames over budget; two put the clock in flat mode until idle.
    missed: u8,
    flat: bool,
    paused: bool,
    /// Immediate frames the host was told to request, for tests and the report.
    pub frames_requested: u64,
}
impl Default for Clock {
    fn default() -> Self {
        Self::settled()
    }
}
impl Clock {
    fn with(mode: Mode) -> Self {
        Self {
            mode,
            now: 0.,
            tweens: HashMap::new(),
            springs: HashMap::new(),
            loops: 0,
            blink_edge: None,
            next: None,
            missed: 0,
            flat: false,
            paused: false,
            frames_requested: 0,
        }
    }
    /// End states, never a frame. The default for every headless path.
    pub fn settled() -> Self {
        Self::with(Mode::Settled)
    }
    /// The window's clock. `reduced` draws end states and keeps only the caret blink.
    pub fn live(reduced: bool) -> Self {
        Self::with(if reduced { Mode::Reduced } else { Mode::Live })
    }
    /// `BBOXY_REDUCED_MOTION=1` (any non-empty value but `0`), else the OS setting.
    pub fn reduced_motion_preferred() -> bool {
        match std::env::var("BBOXY_REDUCED_MOTION") {
            Ok(v) if !v.is_empty() => v != "0",
            _ => os_reduce_motion(),
        }
    }
    /// Whether values move at all right now (live, not reduced, not in the flat fallback).
    pub fn animates(&self) -> bool {
        self.mode == Mode::Live && !self.flat && !self.paused
    }
    pub fn reduced(&self) -> bool {
        self.mode == Mode::Reduced
    }
    /// In the two-missed-frames fallback (until the next idle).
    pub fn flat(&self) -> bool {
        self.flat
    }
    /// The frame's time in milliseconds. Monotonic; a step backwards is ignored.
    pub fn tick(&mut self, now_ms: f32) {
        if now_ms.is_finite() && now_ms > self.now {
            self.now = now_ms;
        }
    }
    pub fn now(&self) -> f32 {
        self.now
    }
    /// Loops pause while the window is occluded; they resume where the clock is.
    pub fn pause(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// The value of `id` now, moving toward `target` over `ms` on `ease`. First sight is
    /// `target` itself (an element that appears takes its end state). A new target in
    /// mid-flight retargets from the current value.
    pub fn tween(&mut self, id: &str, target: f32, ms: f32, ease: Ease) -> f32 {
        self.tween_impl(id, None, target, ms, ease)
    }
    /// As `tween`, but an id seen for the first time starts at `from`: an entrance.
    pub fn enter(&mut self, id: &str, from: f32, target: f32, ms: f32, ease: Ease) -> f32 {
        self.tween_impl(id, Some((from, 0.)), target, ms, ease)
    }
    /// An entrance that waits `delay_ms` at `from` before it moves (staggered chips).
    pub fn enter_after(
        &mut self,
        id: &str,
        from: f32,
        target: f32,
        ms: f32,
        delay_ms: f32,
        ease: Ease,
    ) -> f32 {
        self.tween_impl(id, Some((from, delay_ms.max(0.))), target, ms, ease)
    }
    /// The current value of a tween asked for this frame, without touching it. None when
    /// nobody asked (the element is at rest and carries no row).
    pub fn value(&self, id: &str) -> Option<f32> {
        self.tweens.get(id).map(|t| t.at(self.now))
    }
    /// Set `id` to `value` with no motion and no history.
    pub fn snap(&mut self, id: &str, value: f32) {
        self.tweens.insert(
            id.into(),
            Tween {
                from: value,
                to: value,
                t0: self.now,
                ms: 0.,
                ease: Ease::Linear,
                touched: true,
            },
        );
    }
    fn tween_impl(
        &mut self,
        id: &str,
        from: Option<(f32, f32)>,
        target: f32,
        ms: f32,
        ease: Ease,
    ) -> f32 {
        if !self.animates() {
            // end state at once; keep a settled row so `wants_frame` stays honest
            self.snap(id, target);
            return target;
        }
        let now = self.now;
        match self.tweens.get_mut(id) {
            Some(t) => {
                t.touched = true;
                if t.to != target {
                    let current = t.at(now);
                    *t = Tween {
                        from: current,
                        to: target,
                        t0: now,
                        ms,
                        ease,
                        touched: true,
                    };
                }
                t.at(now)
            }
            None => {
                let (start, delay) = from.unwrap_or((target, 0.));
                let t = Tween {
                    from: start,
                    to: target,
                    t0: now + delay,
                    ms: if from.is_some() { ms } else { 0. },
                    ease,
                    touched: true,
                };
                let v = t.at(now);
                self.tweens.insert(id.into(), t);
                v
            }
        }
    }
    /// Critically damped follow of `target` with the web's ~`ms` settle (ω = 2π/ms). First
    /// sight is `target`. Integrates from the last frame's time, capped at 50 ms a step so a
    /// stalled window does not overshoot.
    pub fn spring(&mut self, id: &str, target: f32, ms: f32) -> f32 {
        self.spring_impl(id, None, target, ms)
    }
    /// As `spring`, but an id seen for the first time starts at `from` (a jump to the tail
    /// glides from where the view is).
    pub fn spring_from(&mut self, id: &str, from: f32, target: f32, ms: f32) -> f32 {
        self.spring_impl(id, Some(from), target, ms)
    }
    fn spring_impl(&mut self, id: &str, from: Option<f32>, target: f32, ms: f32) -> f32 {
        if !self.animates() {
            self.springs.remove(id);
            return target;
        }
        let now = self.now;
        let w = 2. * std::f32::consts::PI / (ms.max(1.) / 1000.);
        let s = self.springs.entry(id.into()).or_insert(Spring {
            x: from.unwrap_or(target),
            v: 0.,
            target,
            last: now,
            touched: true,
        });
        // A settled spring given a new goal starts its glide here: the clock may have
        // skipped frames while the window idled, so reset the integration step to one frame
        // instead of a stale gap that would overshoot and snap to the clamp.
        if s.done() && s.target != target {
            s.last = now - 16.;
            s.v = 0.;
        }
        s.touched = true;
        s.target = target;
        // Cap the step so one slow or resumed frame stays within the explicit integrator's
        // stable range (w·dt below ~1), never a jump.
        let dt = ((now - s.last) / 1000.).clamp(0., 0.025);
        s.last = now;
        if dt > 0. {
            let gap = s.target - s.x;
            s.v += (w * w * gap - 2. * w * s.v) * dt;
            s.x += s.v * dt;
        }
        if s.done() {
            s.x = s.target;
            s.v = 0.;
        }
        s.x
    }
    /// The current value of a spring without advancing it (for example a scroll offset the
    /// user starts from). None when no spring is in flight.
    pub fn spring_value(&self, id: &str) -> Option<f32> {
        self.springs.get(id).map(|s| s.x)
    }
    /// Phase 0..1 of a loop with `period_ms`. Asking keeps the clock awake this frame. Under
    /// reduced motion, flat fallback or pause the phase is 0 and no frame is wanted.
    pub fn cycle(&mut self, period_ms: f32) -> f32 {
        if !self.animates() {
            return 0.;
        }
        self.loops += 1;
        (self.now / period_ms.max(1.)).rem_euclid(1.)
    }
    /// Caret rule: solid for 500 ms after the last edit (`since_ms`), then a 1 s step blink.
    /// The only loop that runs under reduced motion; always on when settled or paused. It
    /// costs a frame only at each edge (`next_frame` reports the wait), never a continuous
    /// loop.
    pub fn blink(&mut self, period_ms: f32, since_ms: f32) -> bool {
        if self.mode == Mode::Settled || self.paused {
            return true;
        }
        let (on, until_edge) = Self::blink_phase(period_ms, since_ms);
        self.blink_edge = Some(
            self.blink_edge
                .map_or(until_edge, |e: f32| e.min(until_edge)),
        );
        on
    }
    /// A timed frame in `ms` for a hold that ends without a tween (the zoom pill's 900 ms
    /// before it fades). Like a blink edge it costs one frame, not a loop; settled and paused
    /// clocks schedule nothing.
    pub fn wake_in(&mut self, ms: f32) {
        if self.mode == Mode::Settled || self.paused || !ms.is_finite() {
            return;
        }
        let ms = ms.max(1.);
        self.blink_edge = Some(self.blink_edge.map_or(ms, |e: f32| e.min(ms)));
    }
    /// What `blink` would answer, without scheduling anything: for frame cache keys.
    pub fn blink_at(&self, period_ms: f32, since_ms: f32) -> bool {
        if self.mode == Mode::Settled || self.paused {
            return true;
        }
        Self::blink_phase(period_ms, since_ms).0
    }
    fn blink_phase(period_ms: f32, since_ms: f32) -> (bool, f32) {
        let period = period_ms.max(1.);
        if since_ms < 500. {
            (true, 500. - since_ms)
        } else {
            let phase = ((since_ms - 500.) / period).rem_euclid(1.);
            let half = 0.5;
            let left = if phase < half {
                half - phase
            } else {
                1. - phase
            };
            (phase < half, (left * period).max(1.))
        }
    }

    /// Whether anything is moving right now: a tween in flight, a spring not settled, or a
    /// loop asked for this frame. (A pending caret blink edge is not motion.)
    pub fn wants_frame(&self) -> bool {
        if self.paused {
            return false;
        }
        self.loops > 0
            || (self.animates()
                && (self.tweens.values().any(|t| !t.done(self.now))
                    || self.springs.values().any(|s| !s.done())))
    }
    /// What the last `end_frame` decided: `Some(0.)` request the next frame now, `Some(ms)`
    /// wake in `ms`, `None` idle.
    pub fn next_frame(&self) -> Option<f32> {
        self.next
    }
    /// The last frame moved something, so the next frame must be built even if nothing else
    /// changed.
    pub fn in_motion(&self) -> bool {
        self.next == Some(0.)
    }
    /// Ends the frame: forgets ids nobody touched, clears the loop count and decides
    /// `next_frame`. Returns it.
    pub fn end_frame(&mut self) -> Option<f32> {
        let wants = self.wants_frame();
        self.tweens.retain(|_, t| std::mem::take(&mut t.touched));
        self.springs.retain(|_, s| std::mem::take(&mut s.touched));
        self.loops = 0;
        let edge = self.blink_edge.take();
        if self.flat && !wants {
            // idle again: the next motion animates
            self.flat = false;
            self.missed = 0;
        }
        self.next = if wants {
            self.frames_requested += 1;
            Some(0.)
        } else {
            edge.filter(|_| !self.paused)
        };
        self.next
    }
    /// The host's measured cost of the frame just presented, against the 16.6 ms budget.
    /// Two misses in a row fall back to end states until the clock is next idle.
    pub fn frame_cost(&mut self, cost_ms: f32) {
        if self.mode != Mode::Live {
            return;
        }
        if cost_ms > FRAME_BUDGET_MS {
            self.missed = self.missed.saturating_add(1);
            if self.missed >= 2 {
                self.flat = true;
            }
        } else {
            self.missed = 0;
        }
    }
}

/// macOS "Reduce motion" (System Settings → Accessibility → Display), read from the
/// `com.apple.universalaccess` preference domain through CoreFoundation. Off when unset
/// or unreadable.
#[cfg(target_os = "macos")]
fn os_reduce_motion() -> bool {
    use std::ffi::c_void;
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFTypeID = usize;
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const std::ffi::c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFPreferencesCopyAppValue(key: CFStringRef, application: CFStringRef) -> CFTypeRef;
        fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
        fn CFBooleanGetTypeID() -> CFTypeID;
        fn CFBooleanGetValue(boolean: CFTypeRef) -> u8;
        fn CFRelease(cf: CFTypeRef);
    }
    const UTF8: u32 = 0x0800_0100;
    let key = c"reduceMotion";
    let app = c"com.apple.universalaccess";
    // SAFETY: both strings are NUL-terminated literals; every CF object created here is
    // released on this path, and CFPreferencesCopyAppValue's null return is checked
    // before use.
    unsafe {
        let key = CFStringCreateWithCString(std::ptr::null(), key.as_ptr(), UTF8);
        let app = CFStringCreateWithCString(std::ptr::null(), app.as_ptr(), UTF8);
        if key.is_null() || app.is_null() {
            if !key.is_null() {
                CFRelease(key);
            }
            if !app.is_null() {
                CFRelease(app);
            }
            return false;
        }
        let value = CFPreferencesCopyAppValue(key, app);
        let on = !value.is_null()
            && CFGetTypeID(value) == CFBooleanGetTypeID()
            && CFBooleanGetValue(value) != 0;
        if !value.is_null() {
            CFRelease(value);
        }
        CFRelease(key);
        CFRelease(app);
        on
    }
}
#[cfg(not(target_os = "macos"))]
fn os_reduce_motion() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn curves_pass_through_the_corners_and_pop_overshoots() {
        for e in [Ease::Out, Ease::Std, Ease::Pop, Ease::InOut, Ease::Linear] {
            assert_eq!(e.at(0.), 0.);
            assert_eq!(e.at(1.), 1.);
            assert!((e.at(0.5) - e.at(0.5)).abs() < 1e-6);
        }
        assert!((Ease::Linear.at(0.25) - 0.25).abs() < 1e-6);
        assert!(Ease::Out.at(0.25) > 0.6, "out is fast early");
        assert!(Ease::Std.at(0.25) < 0.3, "std eases in");
        assert!((0.0..=1.0).contains(&Ease::InOut.at(0.5)));
        assert!((Ease::InOut.at(0.5) - 0.5).abs() < 0.02);
        assert!(Ease::Pop.at(0.7) > 1., "pop overshoots past the target");
    }
    #[test]
    fn tween_retargets_from_the_current_value_and_never_snaps_back() {
        let mut c = Clock::live(false);
        c.tick(0.);
        assert_eq!(c.enter("a", 0., 100., 200., Ease::Linear), 0.);
        c.tick(100.);
        assert!((c.tween("a", 100., 200., Ease::Linear) - 50.).abs() < 1e-3);
        // retarget mid-flight: from 50 toward 0
        assert!((c.tween("a", 0., 200., Ease::Linear) - 50.).abs() < 1e-3);
        c.tick(200.);
        assert!((c.tween("a", 0., 200., Ease::Linear) - 25.).abs() < 1e-3);
        c.tick(400.);
        assert_eq!(c.tween("a", 0., 200., Ease::Linear), 0.);
        assert!(!c.wants_frame());
    }
    #[test]
    fn settled_and_reduced_answer_end_states_without_frames() {
        for mut c in [Clock::settled(), Clock::live(true)] {
            c.tick(0.);
            assert_eq!(c.enter("a", 0., 1., 320., Ease::Out), 1.);
            assert_eq!(c.spring("s", 400., 180.), 400.);
            assert_eq!(c.cycle(1600.), 0.);
            assert!(!c.wants_frame());
            assert_eq!(c.end_frame(), None);
            assert_eq!(c.frames_requested, 0);
        }
        let mut c = Clock::live(true);
        c.tick(0.);
        assert!(c.blink(1000., 0.), "reduced motion keeps the caret blink");
        assert!(!c.wants_frame(), "a blink edge is a timed wake, not motion");
        assert_eq!(c.end_frame(), Some(500.), "solid while typing, then blink");
        c.tick(700.);
        assert!(c.blink(1000., 700.), "on for the first half step");
        assert_eq!(c.end_frame(), Some(300.));
        c.tick(1200.);
        assert!(!c.blink(1000., 1200.), "off half of the step blink");
        assert_eq!(c.end_frame(), Some(300.));
        let mut c = Clock::settled();
        assert!(c.blink(1000., 5000.), "settled never hides the caret");
        assert_eq!(c.end_frame(), None);
    }
    #[test]
    fn delayed_entrance_waits_at_from() {
        let mut c = Clock::live(false);
        c.tick(0.);
        assert_eq!(c.enter_after("chip", 0., 1., 100., 60., Ease::Linear), 0.);
        c.tick(60.);
        assert_eq!(c.enter_after("chip", 0., 1., 100., 60., Ease::Linear), 0.);
        c.tick(110.);
        assert!((c.enter_after("chip", 0., 1., 100., 60., Ease::Linear) - 0.5).abs() < 1e-3);
    }
    #[test]
    fn spring_settles_near_the_web_time_constant_without_overshoot() {
        let mut c = Clock::live(false);
        c.tick(0.);
        assert_eq!(c.spring_from("s", 0., 400., 180.), 0.);
        let mut last = 0.;
        let mut settled_at = None;
        let mut at_180 = 0.;
        for frame in 1..200 {
            let now = frame as f32 * 16.;
            c.tick(now);
            let x = c.spring("s", 400., 180.);
            assert!(x >= last - 1e-3 && x <= 400., "monotone, no overshoot: {x}");
            last = x;
            if now <= 180. {
                at_180 = x;
            }
            if x == 400. && settled_at.is_none() {
                settled_at = Some(now);
            }
        }
        assert!(
            at_180 > 360.,
            "most of the glide is done by 180 ms: {at_180}"
        );
        let settled = settled_at.expect("settles");
        assert!((200. ..=600.).contains(&settled), "settled at {settled} ms");
    }
    #[test]
    fn two_missed_frames_fall_flat_until_idle() {
        let mut c = Clock::live(false);
        c.tick(0.);
        c.enter("a", 0., 1., 320., Ease::Out);
        assert_eq!(c.end_frame(), Some(0.));
        c.frame_cost(20.);
        assert!(!c.flat());
        c.tick(20.);
        c.enter("a", 0., 1., 320., Ease::Out);
        assert_eq!(c.end_frame(), Some(0.));
        c.frame_cost(20.);
        assert!(c.flat(), "two over-budget frames fall back to end states");
        c.tick(40.);
        assert_eq!(c.tween("a", 1., 320., Ease::Out), 1.);
        assert_eq!(c.end_frame(), None);
        c.frame_cost(1.);
        assert!(!c.flat(), "idle clears the fallback");
        c.tick(60.);
        assert_eq!(c.enter("b", 0., 1., 320., Ease::Out), 0.);
        assert!(c.wants_frame());
    }
    #[test]
    fn untouched_ids_are_forgotten_and_return_at_their_end_state() {
        let mut c = Clock::live(false);
        c.tick(0.);
        c.enter("row", 0., 1., 320., Ease::Out);
        c.end_frame();
        c.tick(16.);
        c.end_frame(); // nobody asked for `row`
        c.tick(32.);
        assert_eq!(c.tween("row", 1., 320., Ease::Out), 1.);
        assert_eq!(c.end_frame(), None);
    }
    #[test]
    fn loops_keep_the_clock_awake_only_while_asked() {
        let mut c = Clock::live(false);
        c.tick(0.);
        let p0 = c.cycle(1600.);
        assert_eq!(c.end_frame(), Some(0.));
        c.tick(800.);
        assert!((c.cycle(1600.) - 0.5).abs() < 1e-3 && p0 == 0.);
        assert_eq!(c.end_frame(), Some(0.));
        c.tick(816.);
        assert_eq!(c.end_frame(), None, "the loop left the screen");
        c.pause(true);
        c.cycle(1600.);
        assert_eq!(c.end_frame(), None, "occluded: no frames");
    }
}
