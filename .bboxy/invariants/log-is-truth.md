---
type: invariant
title: "every view is a projection of the event log; a reconnect at (epoch, seq) yields no durable gap and no duplicate"
status: active
check: cargo test --test gateway_replay --test store_recovery --test gui_fixtures && make test-formal MODELS=SubscriberReplay,StreamToView
cost: "a reconnected window silently misses a tool result or an approval, and the user acts on a transcript that is not what happened"
links:
  - { to: "[[gateway]]", rel: constrains }
  - { to: "[[protocol]]", rel: constrains }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 5: the log is truth

**Property.**
- The only durable state is `sessions/<id>.jsonl` plus content-addressed blobs. There is no second store: kittens' SQLite `wire` copy is dropped.
- For any subscribe point, the durable `seq` values delivered are exactly `after+1..=head`, and then live.
- A face's model after it has drained equals `project(log)`. Projecting a fresh replay and projecting a live stream with arbitrary lag and resubscribe points give the same view model.

**Check.**
- `gateway_replay` subscribes at every seq of a fixture.
- `store_recovery` covers truncation and `Reset`.
- `gui_fixtures::live_equals_replay` asserts equality of the `TranscriptModel` built both ways, with random `Lagged` injection under a fixed seed.
- Models SubscriberReplay and StreamToView must pass, with their broken controls rejected.

Owner: Nyx Copperwing <ARCHITECT>.
