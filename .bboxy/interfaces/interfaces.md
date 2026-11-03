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
  - { to: "[[acp-agent]]", rel: index }
  - { to: "[[log-ir-v1]]", rel: index }
  - { to: "[[rich-pane]]", rel: index }
created: "2026-10-31T00:00:00Z"
updated: "2026-11-02T22:00:00Z"
---
# Interfaces and verification matrix

## Boundaries

| ICD | Sides | Status |
|---|---|---|
| [[protocol]] | engine/store (E1) ↔ every face (E2 GUI, headless, acp, seat) | baselined v0.4, **critical path** |
| [[gateway]] | engine (E1) ↔ faces; InProc and Unix | baselined v0.1 |
| [[provider]] | engine ↔ OpenAI-compatible endpoint (E1 on both sides) | baselined v0.1 |
| [[acp]] | engine via `Host`/`InProc` (E1) ↔ ACP v1 clients over stdio | baselined v0.1 |
| [[acp-agent]] | `AcpAgentActor` behind the Gateway ↔ external ACP v1 agents (Claude Agent, codex-acp, …) as untrusted children | baselined v0.1 (3d76710); center: [[d5972-our-log-format-is-the-center]] |
| [[rich-pane]] | bboxy450-term pane/scene/logic ↔ harness `seat` session socket + `HarnessView` (embedded) | proposed v0.1; Side A rows need the #348 owner |
| [[mcp]] | engine registry and policy (E1) ↔ user-configured MCP servers (untrusted child processes) | baselined v0.1 |
| [[embed-seat]] | bboxy450-term (ICD #348 owner) ↔ `seat` face | phase 1a baselined (bboxy450-term `d6fa3fd5`); 1b proposed |
| [[seat-capability-delivery]] | bboxy450-term launch kit, bridge and hook (ICD #348) ↔ every seat kind, including `seat` | baselined: Side A bboxy450-term `afda1b63`, Side B harness `4deca92`; the capability is a 0600 overlay file, never in an environment |
| [[lc-ir]] | log-converter parsers (Claude, Codex, Bboxy) ↔ IR ↔ lowerers; loss table; sanitizer | proposed v0.1 ([[log-converter]]) |
| [[lc-bxl]] | bxl append writer ↔ tailing readers; binary format | proposed v0.1 |
| [[lc-watch-sink]] | native Claude/Codex logs ↔ watcher ↔ sink; cursor, delivery | proposed v0.1 |
| [[session-switch]] | bboxy450-term ↔ log-converter: live harness switch; amends #348 | proposed; #348 rows need the Oracle's signature under d4071 |
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
| R22 | Automatic compaction keeps requests inside the window, is cache-stable and is rebuilt identically after restart | model HarnessAutoCompact + 4 broken controls; `tests/context_auto.rs` (a)–(f) ([[protocol]] §4d) | `make test` |
| R23 | ACP v1 face: mapping, stop reasons, approvals, load replay, unsupported methods, per-session FIFO admission, backpressure that never closes a connected face | model HarnessAcpFlow + 2 broken controls; `tests/acp_face.rs` and the `tests/fixtures/acp/*.jsonl` goldens ([[acp]] §1c, §6) | `make test` |
| R24 | MCP servers never block a turn, every call is policy-gated, secrets never persist | model HarnessMcpServer + 3 broken controls; `tests/mcp_client.rs` with the fixture server ([[mcp]] §6) | `make test` |
| R25 | ACP agent backend: one ending per prompt, cancel answers every pending permission, every agent request answered, `*_always` never automatic, no claude.ai login | model HarnessAcpAgentTurn + broken control; `tests/acp_agent.rs` ([[acp-agent]] §7). Kill-after-grace has no witness yet: **unproven**. `usage_update` is mapped since f73a034 (session-scope `Usage`) | `make test` |
| R26 | Our log is the center: native Claude/Codex logs project into logs that pass `check_transcript`; switch forks and hands over | `tests/import_native.rs`, `tests/acp_agent.rs` switch test ([[d5972-our-log-format-is-the-center]]) | `make test` |
| R27 | Rich-face parity: the native GUI renders a replay like the web reference | `python3 -I scripts/parity-gate.py OUT [--target]` over `tests/fixtures/parity/*.jsonl` (§Rich-face parity) | executor acceptance (macOS, Chrome); **not** `make test` |
| R28 | The log IR only grows by addition; 0.x logs stay valid; unknown data survives store, replay and fork; usage is never invented ([[log-ir-v1]], [[d5974-log-ir-v1-only-add]]) | `make test-ir` (`scripts/ir-compat.py` check + control, frozen corpus hashes); `tests/ir_v1_corpus.rs` (byte identity through open/replay/fork, unknown durable event opens, broken controls) and `tests/usage_report.rs` (fork prefix counted once; dropped Usage fails), landed with [[2026-11-02-log-ir-v1-compat]] at f73a034 | `make test` |
| R29 | Nothing is dropped at the edge: an unmapped record becomes `Item::Native` with its raw value, unconsumed fields go to `ext`, `Unknown` keeps its payload ([[log-ir-v1]] §7, E4, [[d5978-unknown-is-preserved-and-promoted]]) | `tests/ir_edge.rs` (brief [[2026-11-02-ir-unknown-scanner]]): a never-seen record through each projector round-trips raw; broken control: a restored `_ => {}` arm must fail. **unproven** | `make test` |
| R30 | The IR grows from observed data: `ir-scan` catalogues every unknown and emits an additive proposal that passes `ir-compat`; `make test` fails on a fixture unknown with no pending proposal; `land.sh` refreshes the lock ([[log-ir-v1]] §8) | `tests/ir_scan.rs` (same brief); broken control: a proposal that renames a field is refused. **unproven** | `make test` |

## Rich-face parity (R27)

**What parity means.** For one committed replay log, the native `gui --replay --capture 900x1100` frame and the web reference (bboxy450-term `lab/web/harness.js`, headless Chrome, dark, cozy) show the same rows in the same places: the same items, in the same order, at the same vertical positions, with the same chrome (header, status, composer). Colour and anti-aliasing differences inside a row are not parity failures.

**Instrument.** `scripts/parity-web.sh` writes `web.png`, `native.png`, `side.png` and `score.json`. Both renders are deterministic (two runs of `acp-claude-acp` at 3d76710 gave byte-identical PNGs). Metrics:
- `row_structure_mismatch`: the share of pixel rows where one render has ink and the other has none. It measures layout and is the primary metric.
- `mean_abs_diff`: the mean greyscale difference over the frame. It is secondary and is dominated by background.

**Gate.** `scripts/parity-gate.py` runs the instrument on every fixture and decides. Fixtures are `tests/fixtures/parity/*.jsonl`, adopted from `~/mydev/bboxy450-harness-parity/fixtures` and pinned by sha256 in `.bboxy/runs/parity-baseline-3d76710.json`. That file also records the web reference used, bboxy450-term `55f50175`.

| Exit | Meaning |
|---|---|
| 0 | no regression: every score ≤ baseline + tolerance (mean +0.003, rows +0.02), both PNGs 900×1100, fixture hashes match |
| 2 | (`--target` only) no regression, but a target is not met |
| 1 | regression, size mismatch, stale fixture hash or instrument failure |

**Acceptance for Executor work.**
- Every GUI cut: exit 0. A parity task (p1–p4) also: an improvement in `row_structure_mismatch` on the fixtures it names.
- "Parity done": `--target` exit 0. The targets are:
  - chat fixtures `acp-claude-acp`, `acp-codex-acp`, `acp-omp`, `acp-pi`: rows ≤ 0.08, mean ≤ 0.012;
  - rich fixtures `acp-opencode`, `glyph-atlas`, `mermaid`: rows ≤ 0.25, mean ≤ 0.030.
- The report carries the gate's stdout and the `side.png` paths. Looking at `side.png` is judgement: it advises and escalates, and never replaces the exit code.

**Baseline at 3d76710** (rows / mean): claude-acp .153/.019 · codex-acp .125/.016 · omp .125/.017 · pi .128/.017 · opencode .531/.046 · glyph-atlas .462/.049 · mermaid .497/.033. Gate exit 0; `--target` exit 2. **Broken control:** a native render replaced by a blank frame exits 1 with 11 FAIL lines.

**Rules.**
- Changing a fixture or a target is an Architect re-baseline with a recorded reason. A moved web reference is not: the gate prints `REFERENCE MOVED` and writes `OUT/baseline-candidate.json` from that run; whoever lands the run may adopt it (copy over the baseline file, commit message naming the lab hash), provided the run's TARGET lines are no worse than before. `--ratchet` then tightens from there.
- A tolerance is never widened to pass a cut. After a landed improvement, `parity-gate.py OUT --ratchet` lowers the baseline scores to the measured ones (never raises them, never on a moved reference), so slack does not accumulate; anyone may run it.
- The instrument resizes a wrongly sized native frame before scoring, so the gate checks the PNG sizes itself.
- Not covered: interaction states (hover, popover, scroll), light theme, other sizes. Those stay manual until fixtures exist for them.

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
| **HarnessAutoCompact** (law 5, cache stability) | **new**, built on HarnessTurnLifecycle's turn states | `est` 0..4 (abstract load), `turn_open`, `queued` 0..1, `ended_through` 0..2 (the §4b frontier, not the latest `TurnEnded`), `clipped` 0..2, `replaced` 0..2, `phase` ∈ {idle, ladder, sending}, `steps` 0..2 per round | `NeverReducesOpenTurn` (`clipped ≤ ended_through ∧ replaced ≤ ended_through`); `BoundariesMonotone` (an action never lowers `clipped` or `replaced`); `ItemBeforeRequest` (no `sending` while a ladder item is open); `boundary ~> sending` under WF{ladder, send} | (a) the window includes the open turn: **safety**; (b) after restart the boundary is recomputed from non-durable state and may move back: **safety**; (c) no "nothing left to reduce" exit, so the ladder retries forever: **liveness**; (d) `r` is the literal latest `TurnEnded`, so cancelling a queued turn while one runs lets the ladder reduce the open turn: **safety** | `context_auto` (a)–(f) ([[protocol]] §4d) |
| **HarnessAcpFlow** (law 3, face ordering) | **new**; prompt pairing stays in HarnessTurnLifecycle | per-session inbox (prompt, cancel), admitted, writer queue 0..2 (stands in for 256), `reading`, `conn` ∈ {open, closed}, turn ∈ {none, run, ended} | `CancelHitsItsPrompt`; `CloseOnlyOnEof`; `EngineNeverWaits`; `prompt ~> response` under WF{writer} and `reading` infinitely often | (a) one task per line, so a cancel overtakes its prompt: **safety**; (b) a full queue cancels and closes the connection: **safety** | `acp_face` FIFO and backpressure ([[acp]] §6) |
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
