---
id: "02PF55B2WHTZEBSEJYAVSQN0SZ"
number: 400
type: research
title: "Reusing bboxy450-term's formal stack (Yarn, bboxy450-tlc, Java TLC parity) in bboxy450-harness"
status: draft
created: 2026-10-31
owner: Vesper Greywind (Oracle) — research subagent, read-only on bboxy450-term
links:
  - { to: "[[harness-zoomed-out]]", rel: evidence-for }
  - { to: "[[toolkit-lowering]]", rel: evidence-for }
  - { to: "[[bboxy450-term-toolkit]]", rel: deepens }
---
# Formal reuse: Yarn + bboxy450-tlc + Java TLC parity

Source: bboxy450-term `e624d888`, read only. The working tree had edits from someone else (`.bboxy/README.md`, `resume.md`, one new artifact). None of them touch the formal code. This research changed nothing in the tree. Pointers are `file:line — mechanism`,
relative to `/Users/feral/mydev/bboxy450-term` unless prefixed. Scratch target:
`$S=/private/tmp/claude-501/-Users-feral-mydev-bboxy450-harness/01a194c1-b460-45ac-87fe-4f3b6d2c93ea/scratchpad`.

## 0. Bottom line (read this first)

- **There are no `.tla` source files anywhere.** Every model is a Rust value
  (`bboxy450_tlc::Model`) built in Rust; `.tla`/`.cfg` are *generated* by `Model::tla()`
  and fed to Java TLC only for parity. "Rust-native TLC" = a ~700-line explicit-state
  checker over a tiny integer/Boolean action language, not a TLA+ parser.
- **Yarn is two things:** (a) `reactor!`, a no_std compile-checked async select loop;
  (b) `reactor_model!`, which reuses the *same parser/validator* to emit a
  `bboxy450_tlc::Model` of the **scheduling only** (priority, buffered yields, guards,
  drain bounds). Handler bodies — i.e. all turn semantics — are opaque and *not* modeled.
- Therefore the harness's five law models (AgentTurnLifecycle, CancelTree, ...) are
  **hand-written `Model`s with Broken pairs**, exactly like bboxy450-term's 46 project
  families. `reactor!` ties code to model only for arbitration order, via a shared
  `macro_rules!` declaration + a real-runtime comparison test.
- **External reuse works today from crates.io 0.1.1 without Java** (verified, §3a).
  Java parity reuse needs the Python gate; it is reusable *as the bboxy450-tlc mirror
  uses it* but has bboxy450-term-specific hardcodes (§3d). Project models are private
  test code — reuse = copy with "Adapted from" credit, or upstream extraction.
- No product code in bboxy450-term uses `reactor!` yet (`grep -rl 'reactor!' crates` →
  only bbt-yarn/tlc docs; `crates/bbt-yarn/README.md:214` "No existing application loop
  adopts the macro"). The harness turn loop would be the **first adopter**.

## Commands run (all exit codes real)

| Command (cwd bboxy450-term, `CARGO_TARGET_DIR=$S/target`) | Exit | Result |
|---|---|---|
| `cargo test --locked -p bboxy450-tlc -j2` | 0 | 3 unit + 1 doctest pass, 2.5 s |
| `cargo run --locked -p bboxy450-tlc` (the brief's `-- --help`) | 101 | "a bin target must be available" — **no CLI exists**; entry points are lib + `examples/verify.rs` |
| `cargo run -p bboxy450-tlc --example verify -- $S/formal/models` | 0 | 12-model checker corpus exported (manifest below) |
| `cargo run -p bboxy450-yarn --features model --example verify -- $S/formal/models` | 0 | appended `YarnReactor 48 states`, `YarnBufferedYield 88 states` |
| `cargo test -p bboxy450-yarn --features model --test model_projection` | 0 | shared declaration: model first-choice == runtime for all 7 masks |
| `python3 -I $S/parity_one.py $S/formal Counter BrokenSafety UnfairResponse BrokenCheckpointFence YarnBufferedYield` (imports `scripts/formal.py` `tools/tlc/verdict/dot_graph/native_graph`) | 0 | first run downloaded pinned jar+Temurin to `$S/formal/tools`; 12.6 s total. TLC exits 0/12/13/12/0, graphs equal (3/3, 3/3, 4/4, 64/64 states; 124/124 and 208/208 edges) |
| external probe `$S/extprobe` (crates.io `bboxy450-yarn =0.1.1 features=["model"]`, `bboxy450-tlc =0.1.1`), `cargo test` | 0 | `reactor!` + `reactor_model!` from one `macro_rules!`; buffered steer beat model stream at runtime; model checked, `tla()` emitted; **no Java, no workspace** |
| `BBOXY_FORMAL_OUTPUT=$S/formal/models cargo test -p bboxy450-term --test formal_contracts` | see §6 | full bboxy450-term build; result recorded at the end |
| crates.io API `bboxy450-{tlc,yarn,yarn-macros}` | — | all max_version `0.1.1`; yarn 0.1.1 has features `macros, tokio, model` (0.1.0 lacked `model`) |

Toolchain: rustc/cargo 1.99.0; workspace `rust-version = "1.99"` (`Cargo.toml:42`) is inherited by the published crates, so the harness needs `rust-version >= 1.99`.

## 1. Yarn (`crates/bbt-yarn`, `crates/bbt-yarn-macros`; crates.io `bboxy450-yarn`, lib name `bbt_yarn`)

**What it is.** Vendored from Gonzih/kittens-old `293e1345…` (`crates/bbt-yarn/Cargo.toml:12-14`).
- `src/lib.rs:31-36` — `reactor!` re-exported under feature `macros`; `reactor_model!` only under `macros`+`model`.
- `Cargo.toml:16-21` (bbt-yarn) — features: `default=[macros,tokio]`; `model = ["dep:bboxy450-tlc"]` "never part of the runtime"; `--no-default-features` is `no_std`.
- `crates/bbt-yarn-macros/src/lib.rs:28-31` — `reactor` = `Backend::Core, Transfer::Event`; `:36-66` hidden comparison backends (`reactor_event/_slots/_tokio_event/_tokio_slots`).
- `crates/bbt-yarn-macros/src/lib.rs:72-100` — `reactor_model!`: `syn::parse2::<Reactor>` → **same `validate(&reactor)?`** → per arm emits `ReactorArm{name, terminal, guarded=when.is_some(), drain=max|1, yields_to=index}` → `model::reactor_model(&[..])`.
- `crates/bbt-yarn-macros/src/lib.rs:1464-1478` — crate path via `proc-macro-crate` on `"bboxy450-yarn"`; renamed dep `bbt-yarn = {package="bboxy450-yarn"}` resolves to `::bbt_yarn` (verified in probe).

**Grammar** (`crates/bbt-yarn/README.md:18-47`): `policy{selection: biased; required_phases:[..]}`, phases `initialize/before_poll/after_event`, arms `#[source(id)] #[readiness(quiescent|may_remain_ready)] [relations] item = persistent.place => {..}`.

**Relation attributes** (`README.md:34-47`, diagnostics `:220-237`):
| Attribute | Semantics | Model projection (`src/model.rs`) |
|---|---|---|
| `#[shutdown]` | terminal, unguarded, undrained, protected, leading prefix; returns `Result<Exit,E>` (KTR005) | `terminal=true` → `stopped=1` (`:127-129`) |
| `#[terminal]` | successful handler exits | same as above |
| `#[before(x)]` | freezes one source-order edge (KTR003 cycle, KTR016 lexical) | implicit in lexical index |
| `#[last]` | globally final (KTR004) | lexical index |
| `#[when(bool)]` | snapshot once per arbitration (KTR019 non-bool, KTR014 all-guarded) | guard var nondeterministically 0/1 at arbitration (`:54-75`) |
| `#[yields_to(t, when=buffered)]` | disable this (higher) arm while `t` has backlog, rechecked between drained items (KTR010) | `eligible &= backlog[t]==0`, also in drain (`:81-83,:109-111`) |
| `#[drain(max=N)]` | service selected + up to N items, literal 1..=4096, no alloc (KTR008/009) | `handled<=drain` invariant `bounded_service_window` (`:133-145`) |
| `#[starvation(allowed, reason="..")]` | weakens default KTR007 protection; compiler checks non-empty reason only | not projected |

**Compile-time checks.** Parser/validator KTR000–KTR020 (`bbt-yarn-macros/src/lib.rs:206-1062`); type-level asserts emitted into expansion: `bbt-yarn/src/lib.rs:57` SRC001 source admitted, `:66` KTR006 readiness matches adapter, `:75` KTR009 drainable, `:79` KTR010 backlog probe, `:83` KTR019 guard bool, `:89-103` KTR013 handler/phase result shapes. Executable repair reference: 18 compile-fail fixtures `crates/bbt-yarn/tests/ui/*.rs/.stderr` + 3 compile-pass negative controls `tests/ui-pass/` (`README.md:158-162`).

**Sources available** (`src/source/mod.rs`, `src/source/tokio_impl.rs`): core `Latched`, `FixedQueue<T,N>`; tokio `mpsc(rx, close::Emit|Dormant)` `:98`, `OptionalMpsc` `:202` (arm/disarm/replace), `OneShot`/`one_shot` `:303-310`, `OptionalOneShot` `:341` with `cancel_and_replace :376` / `cancel_and_disarm :383`, `Cancellation`/`cancellation(token)` `:427-434`, `OptionalDeadline` `:470` (`set/set_at/disarm`).
Design note for the harness: `OptionalOneShot` holds **one** future. Parallel tool calls (spec: `JoinSet of tools`) must feed completions through an owned task group into an `mpsc` source, not one `OptionalOneShot` per tool.

**What the projection covers and doesn't** (`src/model.rs:28-36`, `verification/README.md:20-28`): one pending item per source, nondeterministic arrivals, terminating handlers returning Continue; **no** handler effects, errors, source close, wakers, fairness (`check_deadlock=false`, no `weak_fair`, `model.rs:132`). Fixed module name `"YarnReactor"` (`model.rs:53`) — callers rename (`examples/verify.rs:113`). So a reactor model can prove "cancel/steer are selected before the model stream and drains are bounded", never "the turn ends".

**Minimal end-to-end example in the repo (code → model → check):**
1. Code + model from one declaration: `crates/bbt-yarn/tests/model_projection.rs:13-25` (`macro_rules! topology { ($backend:ident, $sources:ident) => { $backend!{ ... } } }`), runtime `topology!(reactor, sources)` `:50`, model `topology!(reactor_model, sources)` `:51`, `model.check(10_000)` `:55`, assert model's first selection == runtime result for each readiness mask `:52-63`.
2. Export for TLC: `crates/bbt-yarn/examples/verify.rs:93-117` (`reactor_model!` → `export(model,"pass",root,&mut manifest)` → `.tla/.cfg/Graph.cfg/.states/.edges/.native-trace` + `manifest.tsv` row).
3. Check: `make test-formal` → `scripts/formal.py` runs TLC twice per model (graph dump, then property check) and compares (§3).
Commands: `cargo test -p bboxy450-yarn --features model --test model_projection` (exit 0), `cargo run -p bboxy450-yarn --features model --example verify -- DIR` (exit 0), parity run above (exit 0).
Generated TLA (observed `$S/formal/models/YarnBufferedYield.tla`): variables `v0..v9`, actions `A0..`, `TypeOK`, `Init`, `Spec`; cfg `SPECIFICATION Spec / INVARIANT TypeOK / CHECK_DEADLOCK FALSE / INVARIANT Inv0`. **Names are erased** — TLC counterexamples are positional.

## 2. bboxy450-tlc (`crates/bboxy450-tlc`, crates.io `bboxy450-tlc 0.1.1`, mirror github.com/bboxy450/bboxy450-tlc public, pushed 2026-10-29)

**Layout.** `src/lib.rs` (701 lines, zero deps, std), `tests/model_checker.rs` (fail-closed tests), `examples/verify.rs` (12-model checker corpus exporter), README/LICENSE/NOTICE/ATTRIBUTION. **No binary target.**

**API** (`src/lib.rs`): `State=Vec<i64>` `:27`; `Expr` `:31` with `Var(idx)/Int/Bool` + `eq/lt/le/plus/minus/and/or/negate` `:65-93`; `Variable::new(name,min,max,init)` `:205`; `Action::new(name,guard,updates)` `:226` (simultaneous assignments reading pre-state); `Invariant{name,condition}` `:236`; `Response{name,request,response}` `:244` (= `request ~> response`); `Model{name,variables,actions,invariants,responses,weak_fair: Vec<Vec<action idx>>,check_deadlock}` `:254-268`; `Model::check(max_states)->Result<Report,String>` `:314` (BFS, exact states, SCC fair-response analysis `:451`, counterexample prefix + fair cycle `Violation` `:280`); `Model::tla()` `:541`; `Model::config(graph_only)` `:597`.

**Supported subset** (`README.md:57-72`, `verification/README.md:30-40`): finite signed-int vector; int/bool literals; `= < <= + -`, `/\ \/ ~`; guarded simultaneous assignment; invariants; explicit-action deadlock; `P ~> Q`; weak fairness over named action groups; self-loop is not fair progress. **Not:** `.tla` parsing, modules/INSTANCE, sets/functions/sequences/records, quantifiers, arbitrary temporal formulas, strong fairness, symmetry, refinement mapping. Variables are referenced **by index**.

**How models are written.** Plain Rust functions returning `Model`; never `.tla`. Project families: `crates/bboxy450-term/tests/formal_contracts/models.rs` (1828 lines) + `notification_models.rs`, `plugin_models.rs`, `seat_models.rs`. Local helper DSL `models.rs:21-59` — `eq/ne/all/implies`, `model(name, broken, &[(var,max,init)])` (name prefix `"Broken"` when `broken`, min=0, `check_deadlock:false`), `action`, `safe`, `no_bad(idx)` ("bad" flag var must stay 0). **Private to the test crate; not published.**

**Pass/broken pairs.** Each family is `fn f(b: bool) -> Model`; `b` removes one guard (e.g. `dirty_wake` `:63-100` turns the recheck's `pending:=1` into `pending:=pending` `:92`). `models.rs:1697-1757 cases()` maps 35 families to `(f(false),"pass"), (f(true), "liveness" if responses non-empty else "safety")` + explicit pairs + `plugin_models::cases()`. Name pairing is enforced by `scripts/formal.py:104-106` (`models == {X, "Broken"+X}`).

**Comparison to Java TLC** (`scripts/formal.py`):
- Native side writes artifacts in `export()` — **copied 3×**: `crates/bboxy450-tlc/examples/verify.rs:190-259`, `crates/bbt-yarn/examples/verify.rs:17-86`, `crates/bboxy450-term/tests/formal_contracts.rs:23-92`. Native verdict must equal expected *before* export (`:206-208`), bound `check(100_000)`.
- `formal.py:167-182 tlc()` — `java -XX:ActiveProcessorCount=2 -Xmx256m -XX:+UseSerialGC -cp tla2tools.jar tlc2.TLC -workers 1 -cleanup -config X[Graph].cfg -metadir ..`, graph run adds `-dump dot,actionlabels X.dot`; **timeout 60 s** → `ValueError` "incomplete is not a pass".
- `:135-153 dot_graph` parses TLC DOT to exact int tuples + `(from, action, to)`; rejects unknown/empty. `:156-164 native_graph` reads `.states/.edges`.
- `:313-332` per model: graph run must verdict `pass` (graph cfg disables properties), exact state-set and edge-set equality and count equality, then property run verdict == expected.
- `:185-194 verdict` — pass only on exit 0 **and** "Model checking completed. No error has been found."; safety=12+"is violated", deadlock=11+"Deadlock reached", liveness=13+"Temporal properties were violated"; anything else = tool failure, never an expected negative.

**Java/Temurin pin.** `verification/tools.lock.json` — TLC `1.7.4` `tla2tools.jar` sha256 `936a2620…0e88`; Temurin JRE `21.0.13+9` for `Darwin-arm64`, `Darwin-x86_64`, `Linux-x86_64`, `Linux-aarch64`, each url+sha256. Download: `formal.py:26-37 artifact()` (`curl -fsSL --retry 2 --max-time 180`, staged, sha256-verified, atomic rename, re-verified on every use); `:46-67 tools()` cache `TARGET/formal/tools/`, extracts with `tarfile filter="data"`, unknown host → error. Never in product runtime or source.

**Exit codes.** Native: `Result<_, String>` (no process). `formal.py` exits `1` on any `ValueError/OSError/SubprocessError` with `formal: FAILED:` (`:339-344`), else 0 with `formal: receipt {json}`. TLC exit codes as above (0/11/12/13).

**Fails closed.** Native: state limit `lib.rs:316,353-354`; checked-i64 overflow; `TypeOK` domain escape; bool/int type mismatch, repeated assignment, bad fairness group `:394-446`; witnessed in `tests/model_checker.rs:9-35`. Gate: checksum `formal.py:43`; host without pin `:52`; inventory missing/stale `:82`; scope/models/sources `:84-91`; contract pair/fields/paths/checks `:104-117`; metadata bounds and `implementation_refinement_proven != false` `:121-132`; TLC timeout `:180`; tool failure `:194`; graph mismatch `:321-322`; verdict mismatch `:325-326`; corpus ≠ `corpus.toml` `:297-300`; contract not in coverage `:304-306`.

## 3. Verification pipeline

**Registry schemas (real excerpts).**
```toml
# verification/coverage.toml:104-115 — one row per workspace package; omission fails (formal.py:70-92)
[[crate]]
name = "bboxy450-yarn"
status = "contract-model"            # or "implementation-tests"
models = ["YarnReactor", "FairResponse", "WeakFairFlicker", "StutteringIsNotFairProgress"]
scope = "Finite reactor scheduling and native checker semantics; arbitrary handler bodies and executor wake behavior remain outside the model."
sources = ["crates/bbt-yarn/src/model.rs"]
# verification/contracts.toml:25-32 — one row per Broken-paired family
[[contract]]
name = "AsyncRenderPreparation"
title = "Background renderer preparation never mutates a retired owner"
models = ["AsyncRenderPreparation", "BrokenAsyncRenderPreparation"]
sources = ["crates/bboxy450-term/src/gpu.rs"]
implementation_tests = ["crates/bboxy450-term/src/gpu.rs"]
assumptions = "One UI waiter, one pending worker and one completion; ..."
property = "No value installs after its UI waiter is dropped. The broken control installs a late result into the absent owner."
checks = ["make test-formal", "make test"]   # must be exactly this (formal.py:114)
# verification/corpus.toml:3-6 — 98 rows: checker 12, yarn 2, project 84
[[model]]
name = "NativeSurfaceResize"
expected = "pass"                    # pass | safety | deadlock | liveness
frontend = "project"                 # checker | yarn | project
```
Per-model `X.metadata.json` (written by `formal_contracts.rs:108-124`): variables(name,min,max,initial), actions, invariants, responses, weak_fair groups, check_deadlock, `implementation_refinement_proven:false`.

**`make test-formal`** (`Makefile:1419-1425`, prerequisite `test-yarn-model` `:1403-1409`):
1. `test-yarn-model`: `cargo test -p bboxy450-tlc`; `cargo test -p bboxy450-yarn --features model`; `cargo check` yarn no_std with/without `model`; strict clippy over tlc+yarn+macros.
2. `python3 -I scripts/test-formal.py` — 11 negative controls on the gate itself (tool errors aren't property failures `:206-210`, tampered tool `:212-217`, coverage omission `:219-228`, DOT parser `:230-239`, contract pairs `:42-63`, metadata refinement claim `:65-79`, plus publisher tests).
3. `cargo run -p bboxy450-tlc --example verify -- TARGET/formal/models` (writes **fresh** `manifest.tsv`).
4. `cargo run -p bboxy450-yarn --features model --example verify -- …` (reads + appends manifest; fails if absent, `examples/verify.rs:92`).
5. `BBOXY_FORMAL_OUTPUT=… cargo test -p bboxy450-term --test formal_contracts` (appends; `formal_contracts.rs:96-128`).
6. `python3 -I scripts/formal.py --target TARGET/formal` → parity, `evidence.json`, archive, receipt.
`make test` → `scripts/ci-local.sh:78` runs `make test-formal`. `make test-formal-contracts` (`:1411-1414`) = native project models + clippy, no Java. `make check-formal-evidence` (`:1400-1401`) = `formal.py --verify-evidence`; also `make push`, `scripts/landing.py:819,991,1027-1047`.

**Evidence / receipt.** `evidence.json` (`formal.py:308-333`): `source` HEAD, `dirty`, `tlc`/`java` pins, `crates`, `contracts`, per model `{name, expected, states, edges, tlc_exit, graph_equal:true, contract_metadata, tla_sha256}`, `complete_tlc_replacement:false`, `input_sha256`. Archived content-addressed under `$(git rev-parse --git-common-dir)/bboxy-formal-evidence/<sha256>` (`:259-280`). Receipt validation `:218-256`: HEAD equality, clean source, corpus exact, inventories equal, input hashes equal, pins equal, per-model exit/graph/TLA hash/metadata.
**Hash inputs** `formal.py:197-207`: hardcoded set (yarn model/verify, macros, tlc lib/verify, 4 verification files, formal.py, `formal_contracts.rs`, `models.rs`, `witnesses.rs`) ∪ every coverage `sources` ∪ every contract `sources`+`implementation_tests`. Gap: `plugin_models.rs` and `seat_models.rs` are in none of these (grep: 0 hits in coverage/contracts; not in the hardcoded set, nor in `scripts/landing.py:596-603` scope list). HEAD binding still catches committed edits; landing scope reuse may not.

**(a) External crate, native only, crates.io, no Java — WORKS.** Verified `$S/extprobe` (exit 0):
```toml
bbt-yarn = { package = "bboxy450-yarn", version = "=0.1.1", features = ["model"] }  # dev-dependency in the harness
bboxy450-tlc = "=0.1.1"                                                              # dev-dependency
```
Product depends on `bbt-yarn` with `default` (no `model`); tests/examples enable `model`. Checker is std, dependency-free, no process, no network (`bboxy450-tlc/README.md:12-14`). Put `model` in `[dev-dependencies]` only so Java/tlc can never enter the product graph.

**(b) Java parity in the harness's own CI target — reuse the mirror pattern.** `scripts/prepare-tlc.py:47-77` already produces a standalone repo that runs the same gate: copies `formal.py`, `test-formal.py`, `crates-publisher.py`, `tools.lock.json`, writes a single-package `Cargo.toml` + `[workspace]`, filtered `corpus.toml` (no `frontend`), `coverage.toml`, and a 3-step Makefile (`test-formal.py` → `cargo run --example verify` → `formal.py`). The published mirror github.com/bboxy450/bboxy450-tlc has `scripts/{formal.py,test-formal.py,crates-publisher.py}`. `formal.py` already handles a non-workspace root (`:74` single `[package]`) and optional `contracts.toml` (`:98-99`, but only when *missing*; if the harness ships one it is enforced — good).
Harness copy works as-is **provided** it (i) has a git HEAD, (ii) writes `manifest.tsv` fresh from its own exporter, (iii) ships `crates-publisher.py` (imported unconditionally by `test-formal.py:25`).

**(c) Registering in bboxy450-term when embedded** (submodule at `crates/bboxy450-harness`, in `workspace.members`):
1. `verification/coverage.toml` row `name="bboxy450-harness"` — mandatory once it's a member (`formal.py:75-82`).
2. Corpus/contract rows with BT-root-relative paths (`crates/bboxy450-harness/...`), names unique across the 98-model manifest (`formal.py:294` duplicate check) — prefix families `Harness*`.
3. One new Makefile line in BT `test-formal` (between `:1424` and `:1425`): `BBOXY_FORMAL_OUTPUT=… cargo test --locked -p bboxy450-harness --test formal` (appending to the manifest).
4. Layering/brand/licensing rows per [[bboxy450-term-toolkit]] §4.
Problem: rows are then **duplicated** between `bboxy450-harness/verification/*.toml` and BT's — drift risk. Fix upstream (gap U3).

**(d) Reusable as-is? — Hardcoded spots.**
| Spot | Hardcode | Effect for the harness |
|---|---|---|
| `scripts/formal.py:23` | `ROOT = parents[1]` of the script | root = wherever the script lives; fine for a copy, unusable as a shared tool pointed at another repo |
| `formal.py:47,242` | `ROOT/verification/tools.lock.json` | copy the lock file (keep identical pins) |
| `formal.py:98` | `crates/bboxy450-term` existence makes contracts mandatory | harness: always ship contracts.toml |
| `formal.py:114` | `checks == ["make test-formal","make test"]` | harness Makefile must expose exactly those names |
| `formal.py:197-204` | bboxy450-term/yarn/tlc paths in `input_hashes` | silently skipped if missing (`is_file`); harness model files only hashed if listed as contract `sources` |
| `formal.py:213-214` | untracked dirs `crates scripts verification lab design .cargo …` | harness `src/`, `tests/` untracked files are invisible to `dirty` |
| `formal.py:263` | archive at git-common-dir `bboxy-formal-evidence` | works in submodule (`.git/modules/...`) |
| `test-formal.py:25` | imports `crates-publisher.py` | must be copied too |
| `test-formal.py:35-40` | `crates/bboxy450-term` gate; exactly 12 `checker` rows | skipped when not BT; fine |
| `test-formal.py:170-204` | `make publish-crates` fixture | skipped when not BT |
| `Makefile:1422-1425` | package names, manifest order | harness writes its own 3-line target |
| 3× `export()` | copied Rust exporter | harness would make a 4th copy |
| `models.rs:21-59` | private DSL | 5th copy if harness copies it |
Verdict: **copy-with-credit works now** (mirror precedent); the right end state is extraction (U1–U3).

## 4. Existing models relevant to a coding-agent harness

All in `crates/bboxy450-term/tests/formal_contracts/` unless noted; contract text in `verification/contracts.toml:<line>`.
| Model (file:line) | Proves | Harness law |
|---|---|---|
| `PluginHostApi` `plugin_models.rs:316` (contract `:125`) | each admitted request ends in **exactly one** outcome (`AnsweredCallbackHasOneOutcome`: effect+refused=1); outer call eventually answered under WF though a callback interleaves; broken reader serves callbacks only between calls → liveness fail | **one-turn-ending**, **tool-call pairing** |
| `OperationRecovery` `seat_models.rs:13` (`:275`) | lookup never redispatches an unknown write, never leaks another root/actor, missing ≠ terminal; A-B-A context | **pairing after crash replay** (synthetic result, never re-run) |
| `HarnessToken` `models.rs:281` (`:135`) | single-use proof + input ownership; fresh proof never clears an ambiguous outcome | approval single-use; retry-before-output fencing |
| `InputReceipt` / `BrokenInputRetry` `bboxy450-tlc/examples/verify.rs:108` | lease fencing, unknown outcome not resent as a new id; applied ≤1 | **provider retry before first output** / at-most-once side effect |
| `PluginConnection` `plugin_models.rs:33` (`:85`) | result delivered only if completion generation == current enabled generation; generations 1..4 incl. A→B→A; stale closes connection | **cancel/generation fencing A→B→A** |
| `AsyncRenderPreparation` `models.rs:1092` (`:25`) | dropped waiter ⇒ late completion never installs (3 vars, smallest template) | **cancel is cancel** (late tool result after cancel) |
| `CheckpointFence` `bboxy450-tlc/examples/verify.rs:61` | stale-generation save cannot confirm | generation fencing on store fsync |
| `PluginSetup` `plugin_models.rs:560` (`:105`) | `NoProcessOutlivesTheLogin`; broken kills leader not group | **cancel kills process groups** |
| `NotificationReplace` `models.rs:102` (`:35`) | newer plan reconciled before posting obsolete after slow authorization | approval superseded by newer request |
| `AttentionLifecycle` `notification_models.rs:138` (`:55`) | detach preserves owner+question; **replay cannot create a question**; old context can't answer newer A→B→A | **approval lifecycle** across reconnect/replay |
| `PluginHookGate` `plugin_models.rs:399` (`:115`) | unlisted ⇒ never waits/vetoes; veto only from deny or fail-closed policy; waiting shim always answers (bounded); broken waits unbounded → liveness | **policy gate** + bounded approval wait |
| `PluginActivation` `models.rs:229` (`:75`) | changed digest/generation/disabled inspections never become active | policy rule / "always allow" bound to identity |
| `PluginHook` `plugin_models.rs:129` (`:95`) | queue offer beyond capacity dropped+counted, never waited; generation-checked delivery | **never-block send** |
| `CustodyDelivery` `models.rs:338` (`:145`) | delivery progresses without assuming dependency recovers (WF on admit only) | never-block on unavailable dependency |
| `SeatIngestion` `models.rs:671` (`:416`) | per-seat bounded next-seq slot; never skip/reorder; healthy seat drains despite held peer; broken shared head queue → liveness | **slow subscriber can't block others; ordered seq** |
| `VolatileBusSubscription` `models.rs:1569` (`:406`) | replay vs live-only subscription policies; which paths touch retained IO | **SubscriberReplay** (`after_seq` policy) |
| `TerminalReplayQuiet` `models.rs:1370` (`:336`), `CheckpointRestore` (`:195`) | replay/restore emits no effects | **log replay never re-runs tools** |
| `DirtyWake` `models.rs:63` (`:5`) | recheck after clearing pending ⇒ no lost wake | **StreamToView** dirty-mark/Latest wake |
| `LifecycleOwnership` `models.rs:422` (`:165`) | UI detach preserves owner; stop signals only captured birth | GUI/embedded view close keeps SessionActor |
| `YarnReactor`/`YarnBufferedYield` (`bbt-yarn/examples/verify.rs:93-112`) | priority + buffered yield + bounded drain | turn-loop arbitration (cancel > steer > stream) |
| `FairResponse`/`WeakFairFlicker`/`StutteringIsNotFairProgress` (tlc verify `:36,:176`) | WF semantics controls | sanity controls for harness liveness claims |

No existing model covers: append-only log with `after_seq` replay producing **no durable gap** under Lagged (closest: SeatIngestion ordering + VolatileBusSubscription policy), turn-end with nested subagents, secrets redaction (not a state property; implementation tests).

## 5. Reuse plan per harness model

All five: hand-written `bboxy450_tlc::Model` families with `b: bool` broken controls in harness `tests/formal/models.rs`; native check in `cargo test`; Java parity via the copied gate. Names prefixed for BT uniqueness.
| Harness model | Decision | Base (Adapted from) | State sketch / invariants | Broken control |
|---|---|---|---|---|
| `HarnessTurnLifecycle` (AgentTurnLifecycle) | **adapt** | PluginHostApi (one outcome + WF answer), OperationRecovery (crash replay) | turn∈{idle,run,cancelling,ended}, tool∈{none,started,result,synthetic}, endings 0..2, crashed; `ExactlyOneEnding` (endings≤1, ended⇒endings=1), `NoOpenItemAtEnd`, `PairedAtEnd` (ended⇒tool≠started); response `run ~> ended` WF{provider,tool,end} | cancel ends turn without synthetic result (safety) / second ending after late completion |
| `HarnessCancelTree` | **adapt** | PluginConnection (gen 1..3 A→B→A) + AsyncRenderPreparation + PluginSetup (process group) | gen, tool_gen, applied_gen, group_alive; `NoStaleApply` (applied⇒tool_gen=gen), `NoGroupOutlivesCancel` | apply without generation check; kill leader only |
| `HarnessApprovalLifecycle` | **adapt** | AttentionLifecycle + PluginHookGate + HarnessToken | asked, answer_gen, turn_gen, effect, replaying; `NoEffectWithoutAllow`, `AnswerOnlyCurrent`, `ReplayNeverReasks`; response `asked ~> (answered ∨ cancelled)` | old-turn answer approves new request (A→B→A) |
| `HarnessSubscriberReplay` | **new**, structure from SeatIngestion + VolatileBusSubscription | log_head 0..3, sub_cursor, buf 0..1, lagged; `NoDurableGap` (delivered seq contiguous), append always enabled (`SendNeverBlocks` as an enabled-ness invariant); response `lagged ~> cursor=head` WF{replay} | drop on lag without replay → gap; shared queue → healthy-sub liveness fail |
| `HarnessStreamToView` | **reuse DirtyWake directly** + AsyncRenderPreparation for closed-item deltas | as DirtyWake (bits/pending/pcs) + item_open | `NoLostImmediateWake`, `NoDeltaAfterCompleted` | omit recheck (as DirtyWake broken) |
| turn-loop arbitration | **reuse Yarn projection** | `reactor_model!` | `bounded_service_window` + priority | KTR compile-fail is the negative; add a runtime-vs-model mask test as model_projection.rs |

**How `reactor!` ties code to model.** In `src/engine/turn_loop.rs` define `macro_rules! turn_topology { ($backend:ident, $s:ident) => { $backend! { policy{selection: biased; required_phases:[..]} #[source(cancel)] #[readiness(quiescent)] #[shutdown] … #[source(stream)] #[yields_to(steer, when=buffered)] #[drain(max=N)] … } } }`; production calls `turn_topology!(reactor, sources)`; `tests/formal.rs` calls `turn_topology!(reactor_model, sources)`, renames the model `HarnessTurnLoop`, and asserts first-selection equality against the runtime for every readiness mask (copy of `model_projection.rs:34-64`). Pattern verified end-to-end from crates.io in `$S/extprobe/tests/probe.rs`. Turn semantics (laws 1,2,4,6) are connected only by **witnesses**: mock-provider e2e tests named in each contract's `implementation_tests`. No refinement proof (`implementation_refinement_proven:false`).

**Harness Makefile** (exact names required by `formal.py:114`):
```make
test-formal-native:   # no Java; part of plain cargo test too
	cargo test --locked --test formal --target-dir '$(TARGET_DIR)'
	cargo clippy --locked --all-targets --target-dir '$(TARGET_DIR)' -- -D warnings
test-formal: test-formal-native   # Java TLC parity; first run downloads pinned jar+JRE into TARGET_DIR/formal/tools
	python3 -I scripts/test-formal.py
	BBOXY_FORMAL_OUTPUT='$(abspath $(TARGET_DIR))/formal/models' BBOXY_FORMAL_FRESH=1 cargo test --locked --test formal --target-dir '$(TARGET_DIR)'
	python3 -I scripts/formal.py --target '$(abspath $(TARGET_DIR))/formal'
check-formal-evidence:
	python3 -I scripts/formal.py --target '$(abspath $(TARGET_DIR))/formal' --verify-evidence
test: … test-formal
```
(`BBOXY_FORMAL_FRESH` = harness exporter creates `manifest.tsv` instead of appending; bboxy450-term's exporter requires a pre-existing file, `formal_contracts.rs:98`.) The harness's `verification/{coverage,contracts,corpus}.toml` + `tools.lock.json` (byte-identical to BT's) live in-repo; vendored `scripts/{formal,test-formal,crates-publisher}.py` carry "Copied from bboxy450-term scripts/… @e624d888" headers and a SOURCE hash check.

**Upstream gaps to fix in bboxy450-term (each its own task; harness must not fork):**
- **U1** Move `export()` (3 copies) into `bboxy450-tlc` as a public `export_corpus(models, dir, Append|Fresh)` plus metadata writer; publish 0.1.2.
- **U2** Publish the contract DSL (`eq/ne/all/implies/model/action/safe/no_bad`, `models.rs:21-59`) in `bboxy450-tlc` with **named variable handles** (today `Var(usize)` — index slips are silent until a graph diff).
- **U3** Parameterize `formal.py`: `--root`, read extra hash inputs and sub-inventories from `verification/formal.toml` instead of `:197-204` and `:213-214`; let BT include `crates/*/verification/*.toml` with path prefixing so the embedded harness is inventoried once. Ship it from the bboxy450-tlc mirror as the canonical copy (or a `bboxy450-tlc` `[[bin]]` that does export + parity driving).
- **U4** Add `plugin_models.rs`/`seat_models.rs` to `formal.py:197-204` and `landing.py:596-603` (or derive from `#[path]`).
- **U5** `reactor_model!` name parameter (fixed `"YarnReactor"`, `model.rs:53`) and optional weak-fair arm groups so turn-loop progress claims are expressible.
- **U6** Readable TLA export: keep variable/action names as comments or identifiers (today `v0`/`A0`).
- **U7** bbt-rt scoped task group (already in [[toolkit-lowering]]) + a Yarn mpsc-of-completions recipe for parallel tools (`OptionalOneShot` is single-slot).
- Missing semantics that the harness models will hit: no sets/sequences ⇒ log replay must be modeled with small counters (≤3 entries); no strong fairness; no quantifiers ⇒ unrolled per-tool vars.

## 6. Project contract run (bboxy450-term)

- `BBOXY_FORMAL_OUTPUT=$S/formal/models cargo test --locked -p bboxy450-term --test formal_contracts -j2 project_contracts_and_broken_controls` — **exit 0** (full app build, then 0.57 s of checking); manifest now 98 rows = `corpus.toml` 98.
- Native sizes of the reuse candidates (states, vars): AsyncRenderPreparation 6/3 (Broken 7), DirtyWake 20/4 (21), PluginHostApi 27/7 (Broken liveness 9), PluginConnection 59/8 (80), SeatIngestion 98/8 (Broken liveness 62), AttentionLifecycle 168/12 (Broken 1028). All tiny; TLC per model is about 1 s.
- Java parity on 8 of them with `python3 -I $S/parity_one.py $S/formal PluginHostApi BrokenPluginHostApi PluginConnection BrokenPluginConnection DirtyWake BrokenDirtyWake BrokenSeatIngestion BrokenAttentionLifecycle` — **exit 0**; TLC exits 0/13/0/12/0/12/13/12 match expected; state and edge sets are identical (e.g. BrokenAttentionLifecycle 1028/1028 states, 6466/6466 edges).
- **Not run on purpose:** the full `scripts/formal.py --target …`, because `archive_evidence` (`formal.py:259-280`) writes into bboxy450-term's `.git/bboxy-formal-evidence`, and this brief is read-only. `$S/parity_one.py` calls the same functions without the evidence and archive steps. So these runs check parity only; they do not reproduce the receipt or evidence binding.
