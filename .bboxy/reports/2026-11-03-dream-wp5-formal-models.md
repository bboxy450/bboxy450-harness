---
type: report
title: 'Dream WP5: refined lifecycle, merge and immutable sink models with exact TLC parity'
task: '[[tasks/2026-11-03-dream-wp5-formal-models]]'
commits:
- a3c5ef086d3d40265cae6cddda865892e07b9c91
- 689dca090b1a52a23f5167972333b8d32ea64538
verify:
  command: make test-formal && make test-formal-parity
  exit_code: 0
author: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
created: 2026-11-04T03:44:12.614Z
id: 01M6JAGG3R7XDECB6YBH3CDYGD
updated: 2026-11-04T03:44:12.614Z
number: 20261103136285
created_by: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
updated_by: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
authorship:
  signer: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
  kind: seat
  public_key: X+HwtTkXh0xN8p8edm1pk0qDQzJChM4ed8AzzWmBZIM=
  process_generation: '1'
provenance:
  digest: sha256:d5a38696b9a0b5d853dfdd48b9abc88aa88d01c10b4e4e675bcefff02f262888
  frontmatter_digest: sha256:45b31cbddcfdac5a4906a0371f9469fdbcd1ad301a76c697e35d775e0afe784d
  signer: seat:01M6J7N5WZ9XEJXWRF45SA6FQ9
  sig: rVg7POv+UvYzlNffx7VC2Mycj7IthJEuPqSLZD3FhrUNDGLaRfMtUtfhzXP2hD13jMhMVME1yRdqANzm+eVeDg==
  log: .bboxy/log/2026-11-04.jsonl#42937
---
ROLE: EXECUTOR. Task [[tasks/2026-11-03-dream-wp5-formal-models]] (#5989), verify revision 1. Worktree `/Users/feral/mydev/bboxy450-harness-dream-wp5`, branch `dream/wp5`, base `6957019f8e1d9c03364cab7f0b1efae0386c5a8c`.

Implemented the three independent dream contract families and eight reachable broken controls. The exact amended verification command, `make test-formal && make test-formal-parity`, exited **0** on signed source commit `689dca090b1a52a23f5167972333b8d32ea64538`: 130 models, 42 contracts, one inventoried crate; exact native/Java reachable states, labeled transitions and expected safety/deadlock/liveness verdicts agree. This is model evidence; dream implementation conformance remains with the implementing WPs.

**Canonical stack limitation:** `make test` exited **2**, because its unchanged `scripts/checks.py kb-records` command exited **1** on baseline duplicate number 5974. All preceding commands, including both feature suites, process controls, formal parity, headers and secret checks, exited 0. The remaining GUI, packaging, smoke, IR and inventory checks were run separately and exited 0. The KB failure is not waived or represented as a green full stack.

## Contract baseline and scope

Consumed signed dream-log `b4894fb6dd4e9731256760683ae2548a03f72bad`, sections 8 and 12, record `01M6J5WGRKN1SDGC6R0F79N5N7`, digest `1fa101d85b19fb0db99f9e22e2c5a2f63db556e4a6de2bcac79cc6b7672650da`. Also fetched the readable superseding [[d20261103136206-dream-contract-refinement]] (`01M6J8T8NS8AZGD811VWH4NX8N`, signed source `066a84e88d17d9f187d97c8326ae0555c417306e`, digest `0513b6349eff97c16abf82f9f027a750b404c5de6c4479502a7e9d27ee140bd3`). The original mirror lookups refused `number_conflict`; they were not bypassed or edited.

The signed task amendment `29Z1HBAAQPK1YRVSNYA2785RK0` was fetched and applied: the assertions now live in `tests/formal_nonvacuity.rs`, and the report uses the exact expanded verify command. No export change was needed. No production source, protocol bytes, license, shared checkout source, merge, publication or trusted installation was changed.

## Acceptance evidence

| Criterion | Source and mechanism |
|---|---|
| Lifecycle | `tests/formal/models.rs:3867` — 25 bounded integer/Boolean variables; two entry revisions, two harnesses, one crash/restart, one held completion and one revival. Proposed/global Superseded/Expired overlay independent harness states. Trial pins encode entry birth and harness. Adoption cannot borrow another harness pass or a prior revival pin. Lower/Load fence terminal and stale artifacts. |
| Merge | `tests/formal/models.rs:4149` — two logs, two harnesses, zero to three writes each. Component joins implement the projection of validated event-set union; fixed terminal precedence replaces arrival/wall-clock winners. Observation and merge are separate actions; Duplicate/Reorder are explicit quotient stutters. |
| Sink | `tests/formal/models.rs:4325` — two hand writes, two adoption arrivals, three attempts, one immutable target and one crash. Read, digest Check, HandEdit and no-replace Publish remain separate. Foreign targets/shared indexes skip; legitimate generated edits change ownership. Loader reachability precedes a single grouped Adoption receipt. Load omits stale/unrecorded pins. |
| Nonvacuity and controls | `tests/formal_nonvacuity.rs:1027`, `:1116`, `:1185` — positive race, ABA, no-replace conflict, shared-index skip, ownership takeover, crash-before-record/restart, separate harness outcomes, old completion surviving revival, and named negative-property assertions. Wrong harness pins fail even when both harnesses passed. |
| Join algebra | `tests/formal_nonvacuity.rs:1210` — extract the actual model join table, assert all 729 normalized input pairs are reachable, then exhaust idempotence, commutativity and all 27^3 associativity triples. The table comes from the model graph rather than a second max implementation. |
| Inventories | `verification/corpus.toml:7`, `verification/contracts.toml:8`, `verification/coverage.toml:10` — all three positive families/eight controls registered with independent verdicts. Initial values/bounds, safety, progress, named fairness and source mappings are exported. Future production/test paths are explicitly planned/pending, never listed as existing conformance witnesses. |

Fairness is explicit: lifecycle assumes admitted eligible trial completions and Restart; merge assumes eventual observation of both stopped logs and Merge; sink assumes local worker steps and Restart. No unconditional fairness is assigned to unavailable quota/root, providers, hand writers, native loaders or external IO. Model deadlock checking is disabled for these three waiting contracts; their named response properties are still checked by both backends. The existing corpus retains its registered deadlock controls.

## Exact graph and verdict parity

TLC 1.7.4 and Temurin 21.0.13+9 remain the checksum-pinned reference. Positive TLC property runs exited 0; each new safety control exited 12. Graph generation and full labeled-edge equality succeeded for every row.

| Model | States | Edges | Verdict |
|---|---:|---:|---|
| HarnessDreamLifecycle | 23235 | 120154 | pass |
| BrokenHarnessDreamLifecycle | 83715 | 431512 | safety |
| BrokenHarnessDreamLifecycleFence | 46617 | 235840 | safety |
| BrokenHarnessDreamLifecyclePair | 52623 | 266956 | safety |
| HarnessDreamMerge | 39400 | 239100 | pass |
| BrokenHarnessDreamMerge | 43750 | 256500 | safety |
| BrokenHarnessDreamMergeHarness | 39400 | 239100 | safety |
| HarnessDreamSink | 18474 | 70440 | pass |
| BrokenHarnessDreamSink | 20892 | 79814 | safety |
| BrokenHarnessDreamSinkAdoption | 57999 | 237073 | safety |
| BrokenHarnessDreamSinkLoad | 24666 | 92953 | safety |

Retained clean-source receipt: `verification/evidence/23dca4442ce3683fd8a47da4115e48a0a4665d1908f79398f40d92f7355a5a1d/evidence.json`, source `689dca090b1a52a23f5167972333b8d32ea64538`, dirty=false. Its `models/` directory retains generated TLA/config, native states/edges/traces, Java DOT/property logs, metadata and pinned hashes. The report commit changes HEAD; the exact amended command will be rerun after this report is committed, and that current-HEAD receipt/result will be carried by task_report.

## Commands and actual results

- `make test-formal && make test-formal-parity` — **0** at 689dca0. Each native invocation: 4 formal-model tests, 29 nonvacuity tests and one reactor test; eight Python gate controls also pass. All 130 reference comparisons complete.
- `cargo clippy --offline --locked --test formal_nonvacuity --test formal_models -- -D warnings` — **0**. `cargo fmt --check` and `git diff --check` — **0**.
- `cargo test --offline --locked --test formal_nonvacuity dream -- --nocapture` — **0**, all three focused witnesses. Log `/tmp/dream-wp5-nonvacuity.log`.
- `cargo test --offline --locked --lib` — **0**, 28 tests. `cargo test --offline --locked --test protocol_laws --test store_recovery --test store_fork` — **0**, 11 tests. These invoke the real `protocol::laws::check_transcript`; there is no invented shell target named check_transcript. Logs `/tmp/dream-wp5-unit.log`, `/tmp/dream-wp5-log-witnesses.log`.
- `make test` — **2** at a3c5ef0; nested KB checker **1**, all 17 preceding canonical command rows **0**, private HOME/operator_input=false. Log `/tmp/dream-wp5-test-final.log`, SHA256 `616925b68b8a699a4ccef21f256ad68da2cccc4e2b3d8ab7b71381aba1c975c1`.
- `make test-gui test-app-packaging smoke-app` — **0**. Actual Metal smoke: 21 visible items/14 kinds; private ad-hoc installs/signature checks; hidden native window/surface/live frame and exact process exit/reap. No Apple identity/Keychain or desktop input. Log `/tmp/dream-wp5-post-core.log`, SHA256 `f43cf98f6b266a3867fd31088c10da0d3a0fc89539ff71de687e9f9bc014ac1b`.
- `make test-ir` — **0**, 47 locked definitions; control rejects 10 and accepts 4 cases. Log `/tmp/dream-wp5-ir.log`, SHA256 `059804d4afc74022edb73fdf38068746850be2c8efc1d337e34110ba208957d5`.
- Canonical `cargo run --offline --locked -- ir-scan --propose <private-temp>/proposals --check-pending tests/fixtures/ir-v1/pending tests/fixtures/` with private HOME — **0**. Log `/tmp/dream-wp5-ir-scan.log`.
- `scripts/checks.py` kb-records-control, inventory-shape and inventory-shape-control — **0**. Canonical headers, secrets and secrets-control were **0**.
- `git verify-commit a3c5ef086d3d40265cae6cddda865892e07b9c91 689dca090b1a52a23f5167972333b8d32ea64538` — **0**, required Boxy Ed25519 key `9B07E849480C68328CA20684844ECC0587DE7AB0`.

The failed full-stack gate is baseline evidence: both `.bboxy/decisions/d5974-log-ir-v1-only-add.md` and `.bboxy/decisions/d5974-memories-are-kb-records.md` already carry number 5974 in base 6957019; `git show` confirms both, and `git diff 6957019 HEAD -- .bboxy/decisions` is empty. Renumbering/index repair is outside this assignment. An earlier development make-test attempt also exited 2 on an intermediate model compile error; it is superseded by the final run above, not called a baseline defect. Local model compile/state-limit experiments were corrected before the signed result.

## Implementation boundary and handoff

These contracts do not prove arbitrary Rust refinement, raw envelope identity/digest rejection, canonical EntryId bytes, retained terminal-reason sets/scalar metadata, real trial reruns/numeric thresholds, quota fairness, ranking/byte budgets, committed RecordPin custody, no-replace/link/fsync kernel behavior, native loader reachability/forgetting, actual memory prompt roles or real dream API/process conformance. One eligible trial completion per pair/revision and a normalized validated-log quotient are deliberate finite abstractions.

`src/dream` and its new API/process witnesses do not exist in this isolated base. Planned paths and pending `tests/dream_{pipeline,audit_broken,graph,sinks,recall}.rs` are recorded explicitly; existing store/protocol/smoke tests prove substrate only. Implementing WPs own those witnesses, and WP9 owns final source mappings/hashes after integration/rebase. No claim of dream implementation validation or night-run success is made.

Signed implementation commits: a3c5ef086d3d40265cae6cddda865892e07b9c91 and 689dca090b1a52a23f5167972333b8d32ea64538. Changed source: `tests/formal/models.rs`, `tests/formal_nonvacuity.rs` and the three TOML inventories. Generated run receipts/screenshots will be restored after preserving the evidence/logs above; retained formal archives stay local. The report is the only additional committed path.