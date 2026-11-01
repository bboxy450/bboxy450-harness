---
id: 01M6CK65TF8ERXTK6ZVJEP6S6M
number: 4965
type: spec
title: "Spec: bboxy450-log-converter — system, invariants, budgets, models, verification matrix, phases"
status: proposed
links:
  - { to: "[[d4949-log-converter-from-scratch]]", rel: constrained-by }
  - { to: "[[d4947-demo-bundle-sanitized-claude-logs]]", rel: constrained-by }
  - { to: "[[d5087-log-converter-in-term]]", rel: decided-by }
  - { to: "[[log-converter-sota]]", rel: fed-by }
  - { to: "[[native-session-formats]]", rel: fed-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# Spec: bboxy450-log-converter

**Status: proposed.** Operator request via the Oracle (message 01M6CJZSEMEF8MFSYJQ7B33CKP). Binding inputs: d4949 (written from scratch; TLA+ checked by bboxy450-tlc with Java TLC parity; close the Rust TLC gaps), d4947 (only sanitized content is committed), #4951 and #4953.

## 1. What it is for

A conversation outlives its harness. Any Claude Code, Codex or bboxy450-harness session can be:
1. read into one IR;
2. stored compactly as it happens;
3. resumed in any of the three.

"bboxy450-harness" is the user-facing name of our harness.

## 2. Components and boundaries

```
 native logs ──parse──▶ IR ──lower──▶ native session ──resume──▶ target CLI
 (Claude, Codex,          │  ▲
  harness)                ▼  │ read/tail
            watch ──▶ Sink ──▶ bxl log (file now; remote later)
 bboxy450-term ctl seat switch ──▶ switch-prepare (child process)
```

| Boundary | ICD |
|---|---|
| parsers ↔ IR ↔ lowerers, loss table, sanitizer | [[lc-ir]] |
| bxl writer ↔ tailing readers, file format | [[lc-bxl]] |
| native log ↔ watcher ↔ sink, cursor, delivery | [[lc-watch-sink]] |
| bboxy450-term ↔ converter: the live switch (amends #348) | [[session-switch]] |
| bboxy450-harness codec ↔ harness protocol v0.4 | [[protocol]] §3–§4, read-only for us |

**CLI**: `bboxy450-log-converter`, with these subcommands:
- `parse <provider> <file> -o x.bxl.d`;
- `lower <bxl> --to <provider> [-o]`;
- `convert <file> --to <provider>`, which composes parse and lower;
- `sanitize`;
- `cat` (bxl → canonical JSON);
- `watch <provider> <file> --sink file:<dir>`;
- `switch-prepare` ([[session-switch]] §1);
- `export atif`.

**Location**: `bboxy450-term/crates/bboxy450-log-converter`, a member of the bboxy450-term workspace ([[d5087-log-converter-in-term]], which supersedes d4964). It uses term's `make test`, formal inventory, secret scan and `scripts/land.sh`, and lands under term's recorded authorization. From here on, "the crate" means that path and `C` abbreviates it.

**Dependency direction**: the converter depends on no bboxy450-term application crate (`bbt-mux`, `bboxy450-term`) and not on bboxy450-harness. Term's application and the harness depend on it. This is checked by a `cargo metadata` assertion in `scripts/harness-submodule.py --check-resolution`, extended in P1, which refuses any edge from `bboxy450-log-converter` to those crates.

**How the harness depends on it (P5)**, using the same pattern as the seven toolkit crates:
- the harness `Cargo.toml` declares `bboxy450-log-converter = "0.1"`, with `[patch.crates-io] bboxy450-log-converter = { git = "https://github.com/bboxy450/bboxy450-term", rev = <pin> }` for its standalone build;
- term's root `Cargo.toml` adds `bboxy450-log-converter = { path = "crates/bboxy450-log-converter" }` to its `[patch.crates-io]`, so inside term the harness uses the in-tree crate;
- `check_resolution` (`scripts/harness-submodule.py:85-95`) adds `bboxy450-log-converter` to its map of crates that must resolve once to the in-tree path. Without that, a second git copy could resolve silently;
- a pin bump in the harness is an ordinary harness PR.

## 3. Invariants

Each one is a gate once its phase lands. The command is named, and exit 0 means held.

| Invariant | Check | Violation costs |
|---|---|---|
| **same-provider identity**: `lower::P∘parse::P` is byte-identical | `cargo test --test roundtrip` | a resumed session differs from the original; signatures are rejected |
| **reasoning is never lost from the IR**: readable text and the provider-tagged sealed part are kept; sealed is replayed only to its own provider (operator ruling, message 01M6CKBFRBGGVBDT05CRCV6H35) | `cargo test --test loss` reasoning rows | lost research data, or a foreign signature sent to a model |
| **no silent loss**: `kept + transformed + dropped == events`, and every drop is a loss-table row | `cargo test --test loss` | a conversation silently loses content |
| **determinism**: the same input gives byte-identical bxl | `cargo test --test determinism` | the watcher's idempotency breaks (duplicates) |
| **no torn read / synced survives** | `tests/torn.rs`, plus model `LcAppendLog` | readers see garbage, or durable data is lost |
| **exactly-once into the file sink** | `tests/watch.rs`, plus model `LcWatcher` | duplicated or missing history |
| **one harness per seat; source untouched** | model `LcSessionSwitch`, plus term `make test-session-switch` | two harnesses write one conversation, or the user's original session is corrupted |
| **only sanitized real content is committed**; dictionaries are trained only on synthetic or sanitized corpora | `checks secrets`, plus the `dicts/SOURCES.md` path check, plus the d4947 grep | personal data is published |
| **no operator input** for any test | everything runs in throwaway HOME/CODEX_HOME against mock model endpoints | blocks the swarm |

## 4. Budgets: structural only, no wall-clock percentiles

| Resource | Allocation | Check |
|---|---|---|
| parse/lower memory | O(1) in session length: one native line (cap 64 MiB) + one bxl frame (≤ 1 MiB uncompressed) + the dictionary + open tool calls awaiting results (bounded by the turn) | a counting allocator in tests: peak on a 100 MB synthetic file ≤ peak on a 1 MB file + 2 MiB |
| bytes read | each input byte read once; re-reads only on recovery or a detected truncation/replacement | a read counter in tests == file size (plus the documented re-read cases) |
| bxl size | ≤ 0.35 × native JSONL bytes on the synthetic corpus, raw included (zstd level 3 + dictionary; JSON usually compresses 5–10×, which leaves margin) | `tests/size.rs` ratio assertion |
| tail latency | a frame closes ≤ 250 ms after its first event, on an injected clock | a deterministic test with a mock clock |
| watcher in flight | ≤ 4 unacknowledged batches | `BoundedInFlight` (model) + test counter |
| term frame time | **0**: conversion runs in a child process, never on term's UI or logic thread | term code review row in [[session-switch]] §1, plus a test that `ctl seat switch` returns its acceptance before the child exits |
| presubmit (d5002) | the crate's unit, round-trip and torn-byte tests run in term's presubmit only when the crate or a dependent is affected: a share of ≤ 60 s inside the 400 s affected-test allocation. The real-CLI probes (`make test-log-converter-native`) and the 100 MB budget test run only in ci-local's full run, never in presubmit. The models join `make test-formal` and are covered by d5002 R3 reuse when unchanged | the d5002 `stage_seconds` record. A presubmit over its share is an R4 finding |

## 5. TLA+ models (d4949: bboxy450-tlc, with Java TLC parity of full state/edge sets and verdicts)

| Model | Contract | Broken controls |
|---|---|---|
| `LcAppendLog` | [[lc-bxl]] §6: writer, crash anywhere, recovery, index sidecar, 2 tailing readers, rotation | (a) yield before checksum; (b) index before data sync; (c) in-place seek-table overwrite |
| `LcWatcher` | [[lc-watch-sink]] §4: partial lines, truncate, replace, crash/restart, sink ack, in-flight bound | (a) cursor before ack; (b) partial line consumed; (c) inode ignored |
| `LcSessionSwitch` | [[session-switch]] §5: quiesce, stop, convert, launch, rollback, A→B→A | (a) open turn; (b) B before A exits; (c) append to source; (d) queued input to B; (e) overwrite an existing native file (d4972) |

- **Not modelled:** parsing, lowering, CBOR and zstd byte layout. These are covered by golden, round-trip and torn-byte tests.
- **Gaps:** any construct bboxy450-tlc cannot check is a tracked gap. It gets a bboxy450-tlc task in bboxy450-term (where the crate lives), and the model is never left Java-only silently. Each model's coverage entry names its assumptions and bounds.

## 6. Verification matrix

| Requirement (operator) | Evidence | Phase |
|---|---|---|
| (1) parsers and lowerers for all three; the target resumes | `roundtrip`, `loss` and fixpoint tests per provider; real-CLI resume against a mock endpoint (P0 → P6) | P0, P3–P6 |
| (2) compact streaming and append binary format | `torn`, `tail`, `size` and compatibility tests; `LcAppendLog` | P2 |
| (3) a watcher that tails a live log, with a sink and a resumable cursor | `watch` test with SIGKILL restarts; `LcWatcher` | P7 |
| (4) live switch in term in all directions, A→B→A | term `make test-session-switch`; `LcSessionSwitch`; #348 amendment signed | P8 |
| (5) "bboxy450-harness" as the user-facing name (operator, superseding "Bboxy"; Oracle message 01M6CNQ8RNYCQ4YDA1C2GZSPQR) | CLI help and loss-report goldens say `bboxy450-harness`. The provider tag `bboxy` is internal only (CLI `--from bboxy` is accepted, and so is `--from bboxy450-harness`). A golden check refuses the bare word "Bboxy" in user-facing strings | P1 |
| (6) #4935 becomes the Claude parser and sanitizer | P3 diff moves `import::claude` into `parse::claude` and `sanitize`; the harness `import claude` calls the crate through the P5 pattern; the #4935 goldens still pass | P3, P5 |
| d4947 sanitized bundles | the sanitizer planted-secret test + control; bundle checks | P3 |
| d4949 from scratch, credited | no dependency on ctx/casr/Harbor/session-bridge (a `cargo metadata` deny check in the crate's tests); "Inspired by" rows in term's ATTRIBUTION.md | P1 |
| validation, on real clients | the night run: convert and resume real local sessions (sanitized stats only) | after P8 |

## 7. Phases (each one is a task-sized Executor assignment; parallel where marked)

Everything lands in bboxy450-term through `scripts/land.sh`. The gate for every phase is term's `make test` (ci-local), plus the phase's own targets below. `C` = `crates/bboxy450-log-converter`.

| Phase | Outcome | Depends on | Verify |
|---|---|---|---|
| **P0 resume probe** | **Done** in the retired repo (PR #2 `7945ad92`, facts F1–F10 in [[session-switch]] §3). Port it into term as `make test-log-converter-native`: `C/probes/` driven by a `scripts/e2e/` entry, in a throwaway `HOME`/`CODEX_HOME`, with mock endpoints and no credentials. It joins ci-local's full run, like `test-harness-native`, and stays out of presubmit. Absent clients produce a labelled SKIP, never a pass | — | `make test-log-converter-native` exit 0. Each positive witness has CLI exit 0 and turn one in the mock's captured request; the F4 and F10 negatives fail as required |
| **P1 crate + IR** | Port PR #1 (`bb87743b`) into `C`: IR types, canonical CBOR (RFC 8949 §4.2.1, [[lc-bxl]] §5), framed EventId ([[lc-ir]] §2), goldens, the dependency-direction and deny checks, workspace member, coverage and formal-inventory entries | — (∥ P0) | `cargo test --locked -p bboxy450-log-converter`; `make test-formal` (inventory accepts the new crate); `python3 -I scripts/harness-submodule.py --check-resolution` |
| **P2 bxl** | Writer, reader, tail, recovery, index sidecar, seal and rotation; `LcAppendLog` + 3 controls | P1 | `cargo test -p bboxy450-log-converter --test torn --test tail --test size --test compat`; `make test-formal` with native and Java parity |
| **P3 Claude** | Port #4935 into `parse::claude`, `lower::claude` (raw same-provider) and `sanitize` | P1 (∥ P2, P4) | `cargo test -p bboxy450-log-converter --test roundtrip --test determinism --test sanitize`; the 100 MB memory budget test in ci-local |
| **P4 Codex** | `parse::codex` and `lower::codex` (raw same-provider) | P1 (∥ P2, P3) | `cargo test -p bboxy450-log-converter --test roundtrip --test determinism` |
| **P5 bboxy450-harness codec + harness dependency** | `parse::bboxy` and `lower::bboxy` (protocol v0.4); the harness `import <provider>` calls the crate through the pattern in §2. This is two PRs: the harness PR, then the term submodule bump with the root patch and the `check_resolution` entry | P1, P3 | `cargo test -p bboxy450-log-converter --test roundtrip`; harness `make test` (including `check_transcript` on lowered fixtures); term `make test-harness-native` and `--check-resolution` |
| **P6 cross-provider** | The loss table with LossReport; the Claude id rewrite; Codex `session_meta` synthesis; the importer route | P0, P3–P5 | `cargo test -p bboxy450-log-converter --test loss --test fixpoint`; `make test-log-converter-native` extended to every cross direction |
| **P7 watch** | Watcher, `FileSink` and the `Sink` trait; `LcWatcher` + 3 controls | P2, P3, P4 | `cargo test -p bboxy450-log-converter --test watch`; `make test-formal` |
| **P8 term switch** | `ctl seat switch`, input refusal, rollback, the converter as a child process (the crate's binary), and the #348 rows; `LcSessionSwitch` + 5 controls | P0, P5, P6 | `make test-session-switch` (native, ci-local); `make test-formal`; the Oracle signs the #348 rows |
| P9 extras | ATIF export; per-provider dictionaries trained on the synthetic corpus | P2, P6 | ATIF golden; size budget |

**Critical path:** P0 + P1 → P3 → P6 → P8. P2, P4 and P7 run beside it.

**Highest risk, now:** P6. P0 proved two-turn text sessions only. Tools, images, signed thinking, encrypted reasoning, compaction and TUI resume are still unproven ([[session-switch]] §3).

## Change log

- **2026-11-01** (Oracle messages 01M6CNPD4JHWX35THP7GR5Z16A and 01M6CNQ8RNYCQ4YDA1C2GZSPQR). Moved into bboxy450-term per d5087, which supersedes d4964. §2 adds the location, dependency direction and the P5 harness dependency pattern. The §4 presubmit share is now inside d5002. §7 verify commands are term commands, and P0 becomes `make test-log-converter-native`. The user-facing name is now "bboxy450-harness" (it was "Bboxy"); the provider tag `bboxy` stays internal.

Owner: Nyx Copperwing <ARCHITECT>.
