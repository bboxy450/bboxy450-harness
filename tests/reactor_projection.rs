// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::harness_turn_topology;
use bbt_yarn::{
    reactor,
    reactor::Control,
    reactor_model,
    source::{FixedQueue, Latched},
};
struct Sources {
    cancel: Latched<()>,
    timeout: Latched<()>,
    events: FixedQueue<(), 2>,
    steer: FixedQueue<(), 2>,
}
#[tokio::test]
async fn every_readiness_mask_matches_shared_projection() {
    for mask in 1..16 {
        let mut s = Sources {
            cancel: Latched::new(),
            timeout: Latched::new(),
            events: FixedQueue::new(),
            steer: FixedQueue::new(),
        };
        if mask & 1 != 0 {
            s.cancel.arm(()).unwrap();
        }
        if mask & 2 != 0 {
            s.timeout.arm(()).unwrap();
        }
        if mask & 4 != 0 {
            s.events.push(()).unwrap();
        }
        if mask & 8 != 0 {
            s.steer.push(()).unwrap();
        }
        let actual = harness_turn_topology!(
            reactor,
            s,
            _event,
            { Ok::<u8, String>(1) },
            { Ok(Control::Stop(2)) },
            { Ok(Control::Stop(3)) },
            { Ok(Control::Stop(4)) }
        )
        .unwrap();
        let mut model = harness_turn_topology!(
            reactor_model,
            s,
            _event,
            { Ok::<u8, String>(1) },
            { Ok(Control::Stop(2)) },
            { Ok(Control::Stop(3)) },
            { Ok(Control::Stop(4)) }
        );
        model.name = "HarnessTurnLoop".into();
        for i in 0..4 {
            model.variables[4 + i].initial = i64::from(mask & (1 << i) != 0);
        }
        let graph = model.check(10000).unwrap();
        let choices = graph.edges[0]
            .iter()
            .filter_map(|edge| {
                let state = &graph.states[edge.to];
                (state[0] == 1).then_some(state[1])
            })
            .collect::<Vec<_>>();
        assert_eq!(choices, vec![actual as i64], "mask={mask}");
    }
}
