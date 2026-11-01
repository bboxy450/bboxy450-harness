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
  - { to: "[[upstream-toolkit]]", rel: index }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Interfaces and verification matrix

## Boundaries

| ICD | Sides | Status |
|---|---|---|
| [[protocol]] | engine/store (E1) ↔ every face (E2 GUI, headless, acp, seat) | baselined v0.3, **critical path** |
| [[gateway]] | engine (E1) ↔ faces; InProc and Unix | baselined v0.1 |
| [[provider]] | engine ↔ OpenAI-compatible endpoint (E1 on both sides) | baselined v0.1 |
| [[acp]] | engine via `Host`/`InProc` (E1) ↔ ACP v1 clients over stdio | baselined v0.1 |
| [[mcp]] | engine registry and policy (E1) ↔ user-configured MCP servers (untrusted child processes) | baselined v0.1 |
| [[embed-seat]] | bboxy450-term (ICD #348 owner) ↔ `seat` face | proposed; Side A needs Corvin Duskmantle |
| [[upstream-toolkit]] | harness ↔ bboxy450-term toolkit and formal stack; upstream items U1–U7, each with an interim and an acceptance check | proposed; bboxy450-term owners accept each item |
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
| R16 | Budgets | structural gates `tests/rt_budget.rs`, `tests/gui_budget.rs` ([[budgets]]); wall-clock p99s in `make bench-budgets` (UNMEASURED when load > ncpu, never PASS) | gates: `make test`; p99: bench, night |
| R17 | Formal: every model passes, every broken control is rejected, native = Java TLC | `make test-formal-native` (in `cargo test`), `make test-formal` (Java parity), `make check-formal-evidence` | `make test` |
| R18 | Live provider works | `make smoke-live`: one turn with one `read` tool call against the profile's model. **PASS** needs `TurnEnded{EndTurn}`, a paired tool call and `check_transcript` OK. A missing or timed-out key prints **SKIP**; 429 past the 90 s deadline prints **UNAVAILABLE** | manual and night, **not** `make test` |
| R19 | No test needs the operator | the harness `make test` runs with `HOME=$(mktemp -d)`, no network (the scripted provider), no Keychain | `make test` |
| R20 | Compaction is deterministic and the model never sees replaced items | `tests/context_compact.rs` ([[protocol]] §4b); implementation-only coverage (a pure function), named in `verification/coverage.toml` | `make test` |
| R21 | A fork is a self-contained log identical to the parent prefix | `tests/store_fork.rs` ([[protocol]] §4c); implementation-only coverage (single-writer atomic create over an append-only prefix), named in `verification/coverage.toml` | `make test` |
| R22 | Automatic compaction keeps requests inside the window, is cache-stable and is rebuilt identically after restart | model HarnessAutoCompact + 3 broken controls; `tests/context_auto.rs` (a)–(e) ([[protocol]] §4d) | `make test` |
| R23 | ACP v1 face: mapping, stop reasons, approvals, load replay, unsupported methods | `tests/acp_face.rs` and the `tests/fixtures/acp/*.jsonl` goldens ([[acp]] §6) | `make test` |
| R24 | MCP servers never block a turn, every call is policy-gated, secrets never persist | model HarnessMcpServer + 3 broken controls; `tests/mcp_client.rs` with the fixture server ([[mcp]] §6) | `make test` |

## Formal models

The harness reuses the bboxy450-term formal stack, following [[formal-reuse]] §4 and §5. It builds no stack of its own.

- **Models.** Each model is a hand-written `bboxy450_tlc::Model` family in `tests/formal/models.rs`, built with a `b: bool` that selects the broken control. The model names carry a `Harness` prefix so they stay unique once the harness is inventoried in bboxy450-term.
- **The turn loop.** Its arbitration is the Yarn projection. `turn_topology!` is called once with `reactor` in `src/engine/turn_loop.rs` and once with `reactor_model!` in `tests/formal.rs`. A test checks that the runtime and the model pick the same first source for every readiness mask (copied from `model_projection.rs:34-64`).
- **What the projection proves.** It proves *scheduling only*. Turn semantics (laws 1, 2, 4 and 6) connect to code through witnesses: the tests named in each contract's `implementation_tests`. `implementation_refinement_proven = false`.
- **Pipeline.** `make test-formal-native` runs the native checker inside `cargo test`. `make test-formal` adds pinned Java TLC parity through vendored copies of `scripts/{formal,test-formal}.py`; each copy has a "Copied from bboxy450-term … @e624d888" header and a SOURCE hash check. `make check-formal-evidence` checks the evidence. The target names are the ones `formal.py:114` requires.
- **Registration.** The repo carries `verification/{coverage,contracts,corpus}.toml`, plus `tools.lock.json` byte-identical to bboxy450-term's.
- **Checker limits.** The checker has small integers and Booleans, no sets or sequences, no quantifiers and no strong fairness. The log is therefore modelled as counters up to 3, per-tool state is unrolled into separate variables, and liveness uses weak fairness only.

**Witness** means the real process or API test that maps the model to the code (d365). A model whose upstream base is marked **new** has no upstream shape to adapt.

| Model | Base (Adapted from) | State and bounds | Safety / liveness | Broken control: expected verdict | Witness |
|---|---|---|---|---|---|
| **HarnessTurnLifecycle** (laws 1, 2) | PluginHostApi (one outcome + WF answer), OperationRecovery (crash replay never redispatches), TerminalReplayQuiet (replay emits no effects) | turn ∈ {idle, run, cancelling, ended}; tool ∈ {none, started, result, synthetic}; endings 0..2; crashed | `ExactlyOneEnding`; `NoOpenItemAtEnd`; `PairedAtEnd`; `ReplayNeverReruns`; `run ~> ended` under WF{provider, tool, end} | (a) a second ending after a late completion: **safety**; (b) cancel ends the turn without a synthetic result: **safety** | `e2e_scripted` + `check_transcript`; `store_recovery` truncation sweep |
| **HarnessToolPairing** | covered inside HarnessTurnLifecycle (`PairedAtEnd`, control b). It is not a separate family because the checker cannot quantify over calls, and two unrolled tools are already in that model | — | — | — | `tool-pairing` checks |
| **HarnessCancelTree** (law 4) | PluginConnection (generations 1..3, A→B→A), AsyncRenderPreparation (late result retired), PluginSetup (`NoProcessOutlivesTheLogin`) | gen, tool_gen, applied_gen, group_alive | `NoStaleApply`; `NoGroupOutlivesCancel` | (a) applies without the generation check: **safety**; (b) kills the leader only, not the group: **safety** | `cancel_process` A→B→A and `ESRCH` on every group |
| **HarnessSteerQueue** (law 3, lanes) | Yarn `reactor_model!` projection, named `HarnessTurnLoop` (needs U5); PluginHook (over-capacity offer dropped and counted, never waited) | control-lane depth 0..2 (cap 2 stands in for 256), work lane 0..2, firehose stream, steer pending | `InterruptAlwaysAdmitted`; `SteerExactlyOnce`; `steer ~> consumed` under WF | interrupt placed on the work lane behind the firehose: **liveness** | `gateway_ops::interrupt_when_full`; firehose fixture |
| **HarnessApprovalLifecycle** (law 6) | AttentionLifecycle (replay never creates a question; an old context cannot answer A→B→A), PluginHookGate (a bounded waiter always answers), HarnessToken (single-use proof) | asked, answer_gen, turn_gen, effect, replaying, approver_attached | `NoEffectWithoutAllow`; `AnswerOnlyCurrent`; `ReplayNeverReasks`; `asked ~> (answered ∨ cancelled ∨ unattended)` | (a) an old turn's answer approves a new request: **safety**; (b) waits when no approver is attached: **liveness** | `policy_gate` |
| **HarnessSubscriberReplay** (laws 3, 5) | **new**, built from the structure of SeatIngestion (per-subscriber seq slot) and VolatileBusSubscription (replay vs live policy), with CheckpointFence for the epoch | log_head 0..3, sub_cursor, buf 0..1, lagged, registered_before_read, epoch 0..1 | `NoDurableGap`; `NoDuplicate`; `SendNeverBlocks` (append is always enabled); `lagged ~> cursor = head` under WF{replay} | (a) drop on lag without replay: **safety**; (b) register after the store read: **safety**; (c) shared queue: **liveness** of the healthy subscriber | `gateway_replay`, `gateway_lag`, `store_recovery` `Reset` |
| **HarnessStreamToView** (law 5, render) | DirtyWake directly, plus AsyncRenderPreparation for deltas after completion | the DirtyWake variables + item_open | `NoLostImmediateWake`; `NoDeltaAfterCompleted` | recheck omitted (as in BrokenDirtyWake): **safety** | `gui_fixtures::live_equals_replay` |
| **HarnessDirtyWakeFrame** (render budget) | DirtyWake, extended with a budget quantum and a deferred remainder | bits, pending, remainder 0..1, drain_pc | `NoLostRemainder`; `arrival ~> frame` under WF{drain} | the remainder is dropped without re-marking dirty: **safety** | `gui_budget`: the lost-remainder counter = 0 |
| **HarnessIncrementalMarkdown** (render) | RenderLinePlan (cache identity guard). [[formal-reuse]] maps none; this is the closest upstream shape | prefix_valid, tail_kind ∈ {closed, open-fence}, append kind | `CacheNeverStale`: a reused prefix implies its identity is still valid | the cache is kept across an append that reopens a fence: **safety** | upstream bbt-doc property test: incremental parse = full parse over random append splits |
| **HarnessScrollStick** (render) | **new**. The rules come from `chat_scroll.rs` scoped intent | following, manual_offset, growth, jump | `TailShownWhileFollowing`; `ManualOffsetPreserved`; `jump ~> following` | layout resets a manual offset on growth: **safety** | `gui_fixtures::stick_and_release` |
| **HarnessEmbedLifecycle** | LifecycleOwnership | attach, detach kind ∈ {window, cmd-q, hangup}, stop, birth | `DetachPreservesRun`; `StopSignalsOwnBirth` | detach kills the run: **safety** | `seat_process` hangup; standalone window-close test |
| **HarnessAutoCompact** (law 5, cache stability) | **new**, built on HarnessTurnLifecycle's turn states | `est` 0..4 (abstract load), `turn_open`, `ended_through` 0..2, `clipped` 0..2, `replaced` 0..2, `phase` ∈ {idle, ladder, sending}, `steps` 0..2 per round | `NeverReducesOpenTurn` (`clipped ≤ ended_through ∧ replaced ≤ ended_through`); `BoundariesMonotone` (an action never lowers `clipped` or `replaced`); `ItemBeforeRequest` (no `sending` while a ladder item is open); `boundary ~> sending` under WF{ladder, send} | (a) the window includes the open turn: **safety**; (b) after restart the boundary is recomputed from non-durable state and may move back: **safety**; (c) no "nothing left to reduce" exit, so the ladder retries forever: **liveness** | `context_auto` (a)–(e) ([[protocol]] §4d) |
| **HarnessMcpServer** (laws 3, 4, 6) | PluginConnection (generations A→B→A, as HarnessCancelTree) | `state` ∈ {starting, ready, failed, dead}, `gen` 0..2, `failures` 0..2, `call` ∈ {none, inflight(g)}, `round_open` | `NoStaleResult`; `RoundNeverWaits`; `DeadIsFinal`; `inflight ~> none` under WF{timeout, respond} | (a) a round waits for `ready`: **deadlock**; (b) a result from any generation is accepted: **safety**; (c) no timeout: **liveness** | `mcp_client` ([[mcp]] §6) |

**Sanity controls.** FairResponse, WeakFairFlicker and StutteringIsNotFairProgress (tlc `verify.rs:36,176`) run unchanged, so a liveness verdict means what the harness claims it means.

**Not modelled.** These stay with implementation tests:
- secrets redaction, which is not a state property ([[secrets-never-persist]])
- turn ending with nested subagents (a v0 non-goal)
- real HTTP, wall-clock deadlines and pixels

Every model is a pass/broken pair in `corpus.toml`, with `expected = pass | safety | deadlock | liveness`. A source/model hash mismatch fails the check (d365).

## Open (unproven until evidence exists)

- **Upstream gaps.** U1–U7 from [[formal-reuse]] are recorded with their acceptance checks in [[upstream-toolkit]]. Until U5 lands, HarnessSteerQueue's liveness claim uses a hand-written model beside the projection.
- **Seat.** R15 and the [[embed-seat]] Side A rows wait on bboxy450-term membership and the ICD #348 amendment.
- **Platform.** R13 is macOS-only, because the toolkit is Metal-only (`bbt-render/src/device.rs:18`).

Owner: Nyx Copperwing <ARCHITECT>.
