---
id: "0DWZZJJRQDTNCNJNKK4D95MM21"
number: 5
type: spec
title: "interfaces and the verification matrix: requirement → evidence, and the formal model list"
status: baselined
links:
  - { to: "[[protocol]]", rel: index }
  - { to: "[[gateway]]", rel: index }
  - { to: "[[provider]]", rel: index }
  - { to: "[[embed-seat]]", rel: index }
  - { to: "[[budgets]]", rel: builds-on }
  - { to: "[[harness-zoomed-out]]", rel: verifies }
  - { to: "[[formal-reuse]]", rel: builds-on }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Interfaces and verification matrix

## Boundaries

| ICD | Sides | Status |
|---|---|---|
| [[protocol]] | engine/store (E1) ↔ every face (E2 GUI, headless, acp, seat) | baselined v0.1, **critical path** |
| [[gateway]] | engine (E1) ↔ faces; InProc and Unix | baselined v0.1 |
| [[provider]] | engine ↔ OpenAI-compatible endpoint (E1 on both sides) | baselined v0.1 |
| [[embed-seat]] | bboxy450-term (ICD #348 owner) ↔ `seat` face | proposed; Side A needs Corvin Duskmantle |
| toolkit widgets | E2 upstream in bboxy450-term ↔ the harness `gui` face | no ICD of its own. Each widget's API is part of its upstream task brief, and the face consumes only `protocol` plus `bbt_ui::Element` |

## Requirement → evidence

**PASS** means an exit-0 script. **SKIP** and **UNAVAILABLE** are never PASS. A row with no evidence yet is *unproven*, not done.

| # | Requirement (source) | Evidence | Gate |
|---|---|---|---|
| R1 | One ending (law 1) | invariant [[one-ending]] | `make test` |
| R2 | Pairing (law 2) | [[tool-pairing]] | `make test` |
| R3 | Never block (law 3) | [[never-block]] | `make test` |
| R4 | Cancel is cancel (law 4) | [[cancel-is-cancel]] | `make test` |
| R5 | The log is truth (law 5) | [[log-is-truth]] | `make test` |
| R6 | No side effect outside policy (law 6) | [[policy-gates-effects]] | `make test` |
| R7 | Secrets never persist (law 7) | [[secrets-never-persist]] | `make test` |
| R8 | Wire stability and forward compatibility | `tests/protocol_golden.rs` | `make test` |
| R9 | Strict stream assembly and pre-output retry | `tests/provider_stream.rs` over the SSE fixtures, plus the real-HTTP fixture server | `make test` |
| R10 | Tools: exact-unique edit, staged apply_patch, cap+spill, process-group exec, error-as-output | `tests/tools.rs`, one case per kittens tool behaviour ([[kittens-spec]] §4) | `make test` |
| R11 | Headless `-p --json` emits protocol JSONL that passes `check_transcript` | `tests/cli_headless.rs` runs the built binary against the scripted provider | `make test` |
| R12 | ACP export is lossless for items ACP can express | `tests/acp_export.rs`: fixture → ACP updates → item kinds/status match; kinds ACP lacks are listed in the test | `make test` |
| R13 | GUI renders every item kind from fixtures | `tests/gui_fixtures.rs`: headless `render_to_rgba` of each fixture. Assertions on the frame: non-blank regions per item, no overlap, tail stuck to the bottom. Goldens are not pixel-exact | `make test` (macOS) |
| R14 | Seat line mode, hangup → stop | [[embed-seat]] §6 | `make test` |
| R15 | Embedded build has one copy of each toolkit crate | inside bboxy450-term: `cargo tree -d -p bboxy450-harness \| grep -c bboxy450-` must print `0` | bboxy450-term `make test` (after membership) |
| R16 | Budgets | `tests/rt_budget.rs`, `tests/gui_budget.rs` ([[budgets]]) | `make test` |
| R17 | Formal: every model passes, every broken control is rejected, native = Java TLC | `make test-formal` (the reused pipeline, below) | `make test` |
| R18 | Live provider works | `make smoke-live`: one turn with one `read` tool call against the profile's model. **PASS** needs `TurnEnded{EndTurn}`, a paired tool call and `check_transcript` OK. A missing or timed-out key prints **SKIP**; 429 past the 90 s deadline prints **UNAVAILABLE** | manual and night, **not** `make test` |
| R19 | No test needs the operator | the harness `make test` runs with `HOME=$(mktemp -d)`, no network (the scripted provider), no Keychain | `make test` |

## Formal models

All of these reuse the bboxy450-term stack: `bboxy450-tlc` models written in its `Model/Variable/Action` API, or `reactor_model!` from `bboxy450-yarn` for the turn loop. Each is checked natively and against pinned Java TLC with `verification/tools.lock.json`. Each also has rows in `verification/{coverage,contracts,corpus}.toml`. No parallel stack is built.

The upstream pipeline hardcodes bboxy450-term paths (`scripts/formal.py:202-204` names `crates/bboxy450-term/tests/formal_contracts/models.rs`). Parameterising those paths is an upstream task. Until it lands, the harness runs the same scripts with its own `verification/` dir.

**Witness** means the real process or API test that maps the model to the code (d365).

| Model | State and atomic actions (bounds) | Safety / liveness | Broken control (must be rejected) | Upstream shape reused | Witness |
|---|---|---|---|---|---|
| **AgentTurnLifecycle** (`reactor_model!` of the turn loop) | turn ∈ {Queued, Streaming, ToolPending, Ending, Ended}; finish/cancel/error arrivals; 2 turns, 2 rounds | exactly one `TurnEnded` per turn; ended is absorbing; with weak fairness, every turn ends | cancel racing `Finish` emits a second end | LifecycleOwnership (explicit terminal), the yarn `#[terminal]` relation | `e2e_scripted` + `check_transcript` |
| **ToolPairing** | pending calls {c1,c2}; result, cancel, panic, recovery | at `TurnEnded`, every call has exactly one result | cancel path omits the synthetic result | DocumentFeed's at-most-once emission | `store_recovery` truncation sweep |
| **CancelTree** | generations A, B, A' (A → B → A); held completion; kill and join | no completion applied under a non-current generation; no live child after the turn ends | generation fence removed | AsyncRenderPreparation (late result retired), DocumentFeed (generation bypass control) | `cancel_process` A→B→A |
| **SteerQueue** (was InputPreemption) | firehose model stream; steer, interrupt and work-lane inbox (cap 2) | interrupt is admitted when the inbox is full; under `#[yields_to]` a pending steer is consumed within N stream items; an acked steer appears exactly once | interrupt on the work lane, starved by the firehose (liveness failure) | NativeInputOwnership; yarn `yields_to` / `drain(max)` | `gateway_ops::interrupt_when_full`, firehose fixture |
| **ApprovalLifecycle** | approval ∈ {Pending, Allowed, Denied, Cancelled}; 2 faces resolving; generation; unattended | resolved exactly once; the tool runs only on Allowed of the current generation; no face attached → resolves (no deadlock) | runs on a stale approval; a second variant waits forever when unattended (deadlock) | NotificationReplace (newer revision wins) | `policy_gate` |
| **SubscriberReplay** | log head ≤ 3; subscriber with buffer cap 1; register-vs-read order; lag; resubscribe; epoch reset | delivered durable seqs = `after+1..head`, with no gap and no duplicate; the engine is never disabled by a full buffer | register *after* the store read (gap) | VolatileBusSubscription (replay vs live policy) | `gateway_replay`, `gateway_lag` |
| **StreamToView** | item generations; deltas d1..d3; view open/drop; snapshot | deltas applied in order, exactly once, to the current item generation; a late chunk for a dropped view is retired | applies a delta to a replaced item generation | AsyncRenderPreparation | `gui_fixtures::live_equals_replay` |
| **IncrementalMarkdown** | doc = stable prefix blocks + tail; append that does or does not close a block; cache validity bit | the cached prefix is never stale: render(cache, tail) = render(full) | the cache is kept across an append that reopens the last block (a code fence) | RenderLinePlan (cache identity guard) | upstream bbt-doc proptest: incremental parse = full parse over random append splits of the corpus |
| **DirtyWakeFrame** | event arrival, dirty bit, wake, drain with a budget quantum and remainder | every arrival yields a frame; a budget-deferred remainder re-marks dirty and is never lost | the remainder is dropped without re-marking | **DirtyWake** itself, extended with the budgeted drain | `gui_budget` deferred-drain counter = 0 lost |
| **ScrollStick** | following bit; user wheel; content growth; jump-to-latest | if following, the viewport shows the tail after every growth; a manual offset is never overwritten by layout; jump restores following | layout resets a manual offset on growth | `chat_scroll.rs` scoped-intent rules (new model; upstream has none) | `gui_fixtures::stick_and_release` |
| **EmbedLifecycle** | host attach/detach (window close, Cmd-Q, PTY hangup), explicit stop, process birth | detach preserves the run; only an explicit stop (or hangup, which is defined as stop for a seat) ends it; stop signals only its own birth | detach kills the run | **LifecycleOwnership** | `seat_process` hangup; standalone window-close test |

Every model is a pass/broken pair registered in `corpus.toml` with `expected = pass | safety | deadlock | liveness`. A source/model hash mismatch fails (d365).

Each contract's `assumptions` line names what the model does not cover: real HTTP, wall-clock deadlines, and pixel output stay with the witness tests.

## Open (unproven until evidence exists)

- [[formal-reuse]] had not landed when this matrix was written. If its map assigns different upstream shapes, that map wins and this table is amended.
- R15 and the [[embed-seat]] Side A rows wait on bboxy450-term membership and the ICD #348 amendment.
- R13 is macOS-only: the toolkit is Metal-only (`bbt-render/src/device.rs:18`).

Owner: Nyx Copperwing <ARCHITECT>.
