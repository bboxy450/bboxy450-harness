// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Catalogue observation and first-page capture across two nonreused births.
//! No scheduler fairness or notification-delivery guarantee is assumed: Observe
//! represents a completed watcher observation, List a completed first-page list.
macro_rules! catalogue {
    ($name:ident, $forget_restore:expr, $retain_capture:expr) => {
        bbt_yarn::protocol! {
            pub mod $name {
                domain Birth { First, Second }
                state {
                    issued: Set<Birth> = Set::empty();
                    ready: Option<Birth> = None;
                    observed: Option<Birth> = None;
                    initialized: bool = false;
                    capture: Option<Birth> = None;
                    missed: bool = false;
                    stale: bool = false;
                }
                events { Start(birth: Birth); Down; Observe; List; }
                on Start(birth) {
                    case fresh when ready.is_none() && !issued.contains(birth) => {
                        set issued = issued.with(birth);
                        set ready = Some(birth);
                        preserve observed, initialized, capture, missed, stale;
                    }
                    case duplicate when !(ready.is_none() && !issued.contains(birth)) => {
                        preserve issued, ready, observed, initialized, capture, missed, stale;
                        reject BirthReusedOrLive;
                    }
                }
                on Down {
                    case down when true => {
                        set ready = None;
                        preserve issued, observed, initialized, capture, missed, stale;
                    }
                }
                on Observe {
                    case changed when initialized && observed != ready => {
                        set observed = ready;
                        set missed = missed || ($forget_restore && observed.is_none() && !ready.is_none());
                        preserve issued, ready, initialized, capture, stale;
                    }
                    case initial_or_same when !(initialized && observed != ready) => {
                        set observed = ready;
                        set initialized = true;
                        preserve issued, ready, capture, missed, stale;
                    }
                }
                on List {
                    case refresh when !ready.is_none() && (!$retain_capture || capture.is_none()) => {
                        set capture = ready;
                        preserve issued, ready, observed, initialized, missed, stale;
                    }
                    case pinned when !ready.is_none() && $retain_capture && !capture.is_none() => {
                        set stale = stale || capture != ready;
                        preserve issued, ready, observed, initialized, capture, missed;
                    }
                    case down when ready.is_none() => {
                        preserve issued, ready, observed, initialized, capture, missed, stale;
                        ignore Unavailable;
                    }
                }
                #[invariant(id = "MCP-RELAY-001")]
                fn changed_observations_notify() { !missed }
                #[invariant(id = "MCP-RELAY-002")]
                fn first_page_uses_current_birth() { !stale }
            }
        }
    };
}
catalogue!(HarnessMcpRelay, false, false);
catalogue!(BrokenHarnessMcpRelayNotify, true, false);
catalogue!(BrokenHarnessMcpRelayCapture, false, true);
