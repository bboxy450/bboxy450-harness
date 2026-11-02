# Harness formal verification

The native backend is crates.io `bboxy450-tlc =0.1.1`, using its finite integer
and Boolean Model/Variable/Action API. Yarn 0.1.1 projects the production
`harness_turn_topology!` declaration. This repository contains no checker fork.

The corpus has 77 model variants mapped to 28 contracts, including the shared
Yarn scheduling projection.
It explores two queued turns, two tools/rounds, three birth generations (A-B-A),
a two-slot stand-in for the bounded work lane, and a three-entry replay log.
Explicit weak-fair action groups discharge turn, steer, approval and healthy
subscriber response obligations. No fairness is assumed for arbitrary OS IO.

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
