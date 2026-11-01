# Harness formal verification

The native backend is crates.io `bboxy450-tlc =0.1.1`, using its finite integer
and Boolean Model/Variable/Action API. Yarn 0.1.1 projects the production
`harness_turn_topology!` declaration. This repository contains no checker fork.

The corpus has eleven positive/broken families and one scheduling projection.
It explores two queued turns, two tools/rounds, three birth generations (A-B-A),
a two-slot stand-in for the bounded work lane, and a three-entry replay log.
Explicit weak-fair action groups discharge turn, steer, approval and healthy
subscriber response obligations. No fairness is assumed for arbitrary OS IO.

Broken controls cover double endings, omitted synthetic results, stale
completions, leader-only cancellation, a blocked interrupt, stale approvals,
unattended waiting, late registration, dropped lag repair, a shared subscriber
queue and lost dirty wakes. A negative passes the gate only when the native and
Java backends both reject it with the registered safety/liveness verdict.

`coverage.toml` inventories the sole package and every Rust source file.
`contracts.toml` maps each family to source and real API/process witnesses;
`corpus.toml` independently fixes expected outcomes. Omitted packages, sources,
models or paired controls fail. Rust const array counts fix variable arity.

Run `make test-formal-native`, or `make test-formal` for full native/Java parity.
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
