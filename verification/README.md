# Harness formal verification

The native backend is crates.io `bboxy450-tlc =0.1.1`, using its finite integer
and Boolean Model/Variable/Action API. Yarn 0.1.1 projects the production
`harness_turn_topology!` declaration. This repository contains no checker fork.

The corpus has 108 model variants mapped to 36 contracts, including the shared
Yarn scheduling projection.
It explores two queued turns, two tools/rounds, three birth generations (A-B-A),
a two-slot stand-in for the bounded work lane, and a three-entry replay log.
Explicit weak-fair action groups discharge turn, steer, approval and healthy
subscriber response obligations. No fairness is assumed for arbitrary OS IO.

`HarnessSocketDelivery` binds one message to its announced session and op_id
across three distinct A/B/A births. Under weak fairness of acknowledgement or
closed-session refusal, every sent message settles and produces at most one
turn. Its controls mint a fresh resend ID or redirect a resend to the new
session; both must fail safety. Unattended headless and GUI host workers
check socket discovery, socket mode/lifetime, switch and crash/resume epochs.
Unix/InProc bridge tests project every IR v1 fixture through the shared worker
and compare the complete transcript and rendered `layout_text`. These are
implementation witnesses; Side A delivery and aggregate multi-pane budgets
remain outside this B1/B2 change.
`HarnessSocketLifecycle` adds two epochs per birth, cancellation, detach,
delayed and duplicate completions. Four controls violate stale, duplicate,
cancelled or detach safety. Connect is weakly fair while attached and live;
real socket operations remain bounded witnesses (5 s Hello/ack, 2 s backoff,
1 s server close drain), not a timing theorem. A fifth lifecycle control retains
an unknown dispatch from the retired session and fails detached-worker progress
under weak fairness of retirement; a socket disconnect/replacement API witness
proves detach releases that intent without polluting the new face. No PTY
discovery frame is emitted.

Broken controls cover double endings, omitted synthetic results, stale
completions, leader-only cancellation, a blocked interrupt, stale approvals,
unattended waiting, late registration, dropped lag repair, a shared subscriber
queue, lost dirty wakes and late profile effects after Close and early publication of an incomplete fork. A negative passes the gate only when the native and
Java backends both reject it with the registered safety/liveness verdict.

`coverage.toml` inventories the sole package and every Rust source file.
`contracts.toml` maps each family to source and real API/process witnesses;
`corpus.toml` independently fixes expected outcomes. Omitted packages, sources,
models or paired controls fail. Rust const array counts fix variable arity.

Run `make test-formal-native` (also `make test-formal`) without Java.
`make test-formal-parity` runs native/Java parity. `make test` runs native first,
then parity.
TLC 1.7.4 and Temurin 21.0.13+9 are checksum-pinned in `tools.lock.json`.
The reference driver compares exact reachable state and labeled-transition
sets, then checks property verdicts independently. Unsupported semantics,
partial exploration, timeout, tool errors and disagreements fail.

The credited upstream exporter and Python gate are adapted locally for U1-U7.
`upstream-sources.json` binds retained source snapshots; if a local toolkit
checkout is available, its main-branch sources must still match those pins.
An isolated clone verifies the pinned snapshots. Re-copy is required on drift.

`target/formal/evidence.json` binds HEAD, source/model/tool hashes and bounds.
The gate archives the same evidence and generated artifacts under
`verification/evidence/<digest>/` so target cleanup does not erase it. Generated
archives are local run artifacts, outside the source-dirty predicate. Use
`make check-formal-evidence` after a clean commit; rebase invalidates receipts.

These models abstract contracts. They do not prove arbitrary Rust refinement,
HTTP/TCP parsing, actual kernel/process behavior, filesystem durability,
wall-clock bounds, secret scrubbing or pixels. Those remain covered by the
integration/process tests named in the inventory. GUI rendering and embedded
workspace inventory integration belong to the separate frontend/upstream work.

The hardening corpus tags held completions with actual births, explores both
acked-steer routes, derives control capacity from typed producer slots, and checks pipe-drain
progress plus ownership after reap. Subscriber models carry content births,
tail loss, a durable epoch high-water mark and a face-prefix invariant; the old
Reset rule and log-only epoch allocator are separate failing safety controls.

Focused rework: ProcessLifecycle separates an exited waitable leader from a
reaped leader and an independently live member. Its two ownership controls
reproduce early-reap cancellation leakage and signalling an empty group.
ControlLaneBound models typed producer slots and consumers returning them;
the duplicate-producer control reaches depth 257. No invariant substitutes a
Rust constant or a mode-generated failure flag for these reachable states.

The GUI adds finite DirtyWakeFrame (three publications, one drain quantum),
ScrollStick (tail/offset 0..3) and EmbedLifecycle (three births) families and their
safety controls. The existing DirtyWake StreamToView pair is retained. These
models are abstractions, with no claim that the checker proves delta content or
pixels: `gui_fixtures` checks ordered projection, generation fencing and random
delta loss/coalescing against completion; `gui_budget` reduces 100k protocol
deltas on the replay worker, then charges the actual UI drain's publications
and notifications with an injected clock (300us drain, deferral, no loss); `gui_render` and `test-gui-smoke` witness the real retained tree,
CoreText and offscreen Metal frame. Native surface presentation/physical IME
and embedding into the separate term workspace remain implementation-only.

`make test` runs `tests/gui_gate.py` after the core gate. It uses its own HOME,
checks the exact upstream disallowed-method catalogue at UI module boundaries,
runs the feature tests and asserts every all-items kind has a nonblank,
nonoverlapping framebuffer region. Screenshot: `.bboxy/runs/gui-smoke.png`.

Protocol v0.2's optional ItemStarted.head and Terminal.live are projected from
wire values as additive fields, so the frontend can build while E1b's typed
core changes land independently. The in-process path reads the same fields
from serialized typed events once that core is linked. The v0.2 GUI fixture
witnesses durable identity before deltas, snapshot repair after a dropped
terminal delta, Unicode-safe 64KiB tail retention, and authoritative completion.
No core/protocol source is forked by this frontend.

Budget amendment 82f2edd: `make test` uses deterministic structural GUI gates.
The shared production drain accepts an injected clock; the 100k-delta witness
charges 50us per received UI publication/notification and state replacement, and asserts a 300us drain with no loss. The 100k protocol deltas are reduced on the replay worker; they are not 100k UI drain charges.
A real worker witness checks retained completed layouts and >4KiB preparation
away from the UI thread. `make bench-budgets` includes `bench-gui-budget`; its
real-clock p99 is observational and prints UNMEASURED when load exceeds ncpu.

Anthropic uses the same HTTP retry/read-stall/cancellation implementation as
OpenAiCompat (`src/provider/http.rs`). `HarnessProviderStream` adds the previously
unmodeled provider boundary: two attempts, zero to two calls, an output bit,
six absent/known/unknown stop-reason states and eleven terminal classes. Its
three controls allow a retry after output, a Stop with tool calls, or a second
terminal. The good native graph has 242 states; controls reach 278, 242 and 423
states respectively. Weak fairness on cancellation guarantees eventual
termination when external IO never completes; no fairness of external IO is
assumed. `tests/anthropic_provider.rs` supplies byte-level HTTP, signed/redacted
replay, cache-placement, token-count, real profile and transcript witnesses.

Finder-launched ACP agents use the process-wide `HarnessAgentPath` cache model:
two concurrent startups, one login-shell probe, shell/fallback PATH identities,
and a deadline that needs no shell fairness. Controls duplicate the probe, omit
the deadline or fail to export the resolved PATH. `tests/acp_agent.rs` runs two
real sessions per isolated process under a PATH missing the agent directory,
including shell failure, empty output, timeout/reaping and absolute commands.
The completion overlay is a pure layout/paint change; `gui_render` compares
actual Metal pixels with and without the ACP error notice and red receipt under
the opaque popover. Pixels and OS behavior remain implementation witnesses.

Native imports add `HarnessNativeUsage` (two identified calls, repeated across
two turn boundaries) and `HarnessNativeSpill` (small/large text, persistence
success/failure, blob before reference), each with a rejected safety control.
`tests/import_native.rs` witnesses the actual Claude message IDs, Codex
cumulative snapshots/resets, cache/reasoning counts, model changes, live file
append, failed writes, redaction, exact blob bytes and JSON/UTF-8 preview bounds.
Neither model proves parsing, byte arithmetic or filesystem durability.

Native usage also separates three cumulative session snapshots (1-3-1) from
call and turn totals; `BrokenHarnessNativeUsageScopes` deliberately sums both
scopes. Provenance and absent-number labels are parser/API witnesses.

`HarnessSeatReport` also explores a two-tick heartbeat clock while working or
awaiting approval. Protocol activity and probes cannot postpone its deadline;
turn end removes the timer. User and agent starts share that ownership. Explicit
Thinking/Writing/Tool changes publish immediately without waiting for a tick;
`BrokenHarnessSeatReportPhase` keeps the old report and fails safety.
An arriving child may infer a phase only before explicit Status selection;
`BrokenHarnessSeatReportChild` overwrites an explicit phase and fails safety.
`BrokenHarnessSeatReportHeartbeat` lets the report
expire without refreshing. Weak fairness on clock steps abstracts scheduling;
fake Tokio time verifies the two-second interval, and a silent tool under a
private PTY witnesses repeated live reports beyond five seconds. Phase text,
ACP identity, redaction and terminal bytes remain implementation witnesses.
