---
id: "0MDNZ1FSXJDKM8VJWTTZQJFA4X"
number: 401
type: interface
title: "ICD: harness ↔ bboxy450-term toolkit and formal stack — what the harness consumes, and the upstream items U1–U7"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[formal-reuse]]", rel: evidence }
  - { to: "[[toolkit-lowering]]", rel: implements }
  - { to: "[[budgets]]", rel: constrains }
  - { to: "bboxy450-term:[[budgets]]", rel: amends }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# ICD: harness ↔ bboxy450-term toolkit and formal stack

**Status: proposed.** Each U-item below is a bboxy450-term change, so bboxy450-term's owners, not this repo, accept it. Each one is its own upstream task, and the harness never forks the code it changes ([[formal-reuse]] §5). While an item is open, the harness keeps working with the interim behaviour named in the table: no harness work waits on an upstream item.

| | |
|---|---|
| Side A | bboxy450-term: `bboxy450-tlc`, `bboxy450-yarn`, `bboxy450-rt`, `bboxy450-ui`/`-doc`/`-render`, `scripts/formal.py`, `landing.py` |
| Side B | bboxy450-harness: `tests/formal/`, `verification/`, the `engine` turn loop, the `gui` face |
| What crosses | crates at 0.1.1 (git-rev patch for the new widgets, [[architect-review-v0]] §6); vendored `scripts/{formal,test-formal}.py` and a byte-identical `tools.lock.json`, each copy carrying a SOURCE hash check |
| Versioning | A vendored script whose SOURCE hash no longer matches bboxy450-term `main` fails the harness `make test-formal` with the message "re-copy". When bboxy450-term bumps `tools.lock.json`, the harness copy follows in the same change |

## Upstream items

| Id | Change (owner crate/script) | Why the harness needs it | Interim in the harness | Accepted when (upstream check) |
|---|---|---|---|---|
| **U1** | `bboxy450-tlc::export_corpus(models, dir, Append \| Fresh)` plus a metadata writer, replacing the 3 copies of `export()`; publish 0.1.2 | the harness exporter must create a fresh `manifest.tsv`; bboxy450-term's requires a file that already exists (`formal_contracts.rs:98`) | the harness carries its own `export()` with `BBOXY_FORMAL_FRESH=1`, credited "Adapted from" | `make test-formal` is green in bboxy450-term with all 3 call sites using the crate function; the corpus is byte-identical before and after |
| **U2** | contract DSL (`eq/ne/all/implies/model/action/safe/no_bad`, `models.rs:21-59`) published in `bboxy450-tlc`, with **named variable handles** instead of `Var(usize)` | a variable index that slips is silent until a graph diff, and the harness writes 10 new model families | the harness copies the helpers with an "Adapted from" credit and adds a compile-time variable-count assertion per model | an existing model is rewritten with handles and produces an identical state/edge set (the native graph diff is empty) |
| **U3** | `formal.py --root`, plus `verification/formal.toml` for hash inputs and sub-inventories (`:197-204`, `:213-214`). bboxy450-term includes `crates/*/verification/*.toml` with path prefixing | **an embedded harness must be inventoried exactly once.** Today, membership puts it in bboxy450-term's `coverage.toml` and in its own copy, so it is counted twice, or it fails the omission check | the harness runs its own pipeline standalone. Once embedded, bboxy450-term's row is `status = "contract-model"` with `models` pointing at the harness sub-inventory, which needs U3 | in bboxy450-term, `python3 -I scripts/formal.py --target …` with the harness submodule present reports each `Harness*` model once, and removing a harness corpus row fails with `missing` |
| **U4** | `plugin_models.rs` and `seat_models.rs` added to `formal.py:197-204` and `landing.py:596-603` (or derived from `#[path]`) | the harness bases HarnessTurnLifecycle, HarnessCancelTree and HarnessApprovalLifecycle on models in those files, so their hashes must bind the evidence | none needed; the harness copies the shapes and does not import those files | an edit to `plugin_models.rs` invalidates the receipt (`make check-formal-evidence` exits non-zero until it is re-run) |
| **U5** | `reactor_model!` name parameter (fixed `"YarnReactor"`, `model.rs:53`) and optional weak-fair arm groups | to name `HarnessTurnLoop` and to state the turn loop's progress claim (`steer ~> consumed`) on the projection itself | rename the model after projection; the liveness claim lives in hand-written HarnessSteerQueue | a yarn example projects a reactor under a given name, and a broken arm-group control fails **liveness** |
| **U6** | readable TLA export: variable and action names kept (today they are `v0`/`A0`) | counterexample traces of 10 harness models have to be read by people | a harness test prints a name table beside every exported `.tla` | Java TLC counterexample output names the actions |
| **U7** | **(a)** a `bbt-rt` scoped task group: spawn, cancel-all, join, no detach. **(b)** a Yarn mpsc-of-completions recipe for parallel tools (`OptionalOneShot` has a single slot) | law 4 (no detached task) and up to 4 parallel read-only tools in one round | a harness-local `TaskGroup` over `tokio::task::JoinSet` plus a child `CancellationToken`, in one file, marked for deletion when (a) lands; parallel completions go through `mpsc(rx, close::Dormant)` | (a) a `bbt-rt` test cancels a group of 3 sleeping tasks and joins them in ≤ 50 ms with no task still alive; (b) a yarn example with a broken control in which a completion is lost |

## Other upstream items already decided elsewhere

- The toolkit widgets (streaming markdown, highlighting, diff, tool card, editor, approval atom) are E2's task #50 ([[toolkit-lowering]]).
- The seat allowlist and generic record are in [[embed-seat]] §4, which needs the ICD #348 owner.

## Budget items for bboxy450-term ([[budgets]] amends its budgets)

| Resource | Allocation the harness asks for | Why |
|---|---|---|
| `make test-formal` time | ≤ 30 s added for the harness's 11 families (the upstream candidates are ≤ 1 s each under TLC, per [[formal-reuse]] §6) | bboxy450-term's `make test` runs the formal suite on every landing |
| `bbt-rt` workers | the harness holds a worker ≤ 1 ms between awaits; blocking work goes to `spawn_blocking` behind 4 permits | the two workers are shared with the whole app ([[budgets]] §1) |
| Frame, embedded page | 0.3 ms out of the 1.5 ms drain row and 0.6 ms out of the 1.0 ms projection row, only while visible | [[budgets]] §2 |

Owner: Nyx Copperwing <ARCHITECT> (Side B). Side A: bboxy450-term owners; ICD #348 and its budgets belong to Corvin Duskmantle <ARCHITECT>.
