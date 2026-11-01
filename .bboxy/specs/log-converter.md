---
id: 01M6CK65TF8ERXTK6ZVJEP6S6M
number: 4965
type: spec
title: "Spec: bboxy450-log-converter — system, invariants, budgets, models, verification matrix, phases"
status: proposed
links:
  - { to: "[[d4949-log-converter-from-scratch]]", rel: constrained-by }
  - { to: "[[d4947-demo-bundle-sanitized-claude-logs]]", rel: constrained-by }
  - { to: "[[d4964-log-converter-own-repo]]", rel: decided-by }
  - { to: "[[log-converter-sota]]", rel: fed-by }
  - { to: "[[native-session-formats]]", rel: fed-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# Spec: bboxy450-log-converter

**Status: proposed.** Operator request via the Oracle (message 01M6CJZSEMEF8MFSYJQ7B33CKP). Binding inputs: d4949 (written from scratch; TLA+ checked by bboxy450-tlc with Java TLC parity; close the Rust TLC gaps), d4947 (only sanitized content is committed), #4951 and #4953.

## 1. What it is for

A conversation outlives its harness. Any Claude Code, Codex or Bboxy session can be:
1. read into one IR;
2. stored compactly as it happens;
3. resumed in any of the three.

"Bboxy" is the user-facing name of our harness.

## 2. Components and boundaries

```
 native logs ──parse──▶ IR ──lower──▶ native session ──resume──▶ target CLI
 (Claude, Codex,          │  ▲
  Bboxy)                  ▼  │ read/tail
            watch ──▶ Sink ──▶ bxl log (file now; remote later)
 bboxy450-term ctl seat switch ──▶ switch-prepare (child process)
```

| Boundary | ICD |
|---|---|
| parsers ↔ IR ↔ lowerers, loss table, sanitizer | [[lc-ir]] |
| bxl writer ↔ tailing readers, file format | [[lc-bxl]] |
| native log ↔ watcher ↔ sink, cursor, delivery | [[lc-watch-sink]] |
| bboxy450-term ↔ converter: the live switch (amends #348) | [[session-switch]] |
| Bboxy codec ↔ harness protocol v0.4 | [[protocol]] §3–§4, read-only for us |

**CLI**: `bboxy450-log-converter`, with these subcommands:
- `parse <provider> <file> -o x.bxl.d`;
- `lower <bxl> --to <provider> [-o]`;
- `convert <file> --to <provider>`, which composes parse and lower;
- `sanitize`;
- `cat` (bxl → canonical JSON);
- `watch <provider> <file> --sink file:<dir>`;
- `switch-prepare` ([[session-switch]] §1);
- `export atif`.

The repo is `bboxy450/bboxy450-log-converter` ([[d4964-log-converter-own-repo]]).

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
| presubmit (d5002) | the converter has its own repo gate, so it adds 0 s to term's and harness's 900 s presubmits. A pin bump runs only the consumer's integration test | the d5002 `stage_seconds` record |

## 5. TLA+ models (d4949: bboxy450-tlc, with Java TLC parity of full state/edge sets and verdicts)

| Model | Contract | Broken controls |
|---|---|---|
| `LcAppendLog` | [[lc-bxl]] §6: writer, crash anywhere, recovery, index sidecar, 2 tailing readers, rotation | (a) yield before checksum; (b) index before data sync; (c) in-place seek-table overwrite |
| `LcWatcher` | [[lc-watch-sink]] §4: partial lines, truncate, replace, crash/restart, sink ack, in-flight bound | (a) cursor before ack; (b) partial line consumed; (c) inode ignored |
| `LcSessionSwitch` | [[session-switch]] §5: quiesce, stop, convert, launch, rollback, A→B→A | (a) open turn; (b) B before A exits; (c) append to source; (d) queued input to B |

- **Not modelled:** parsing, lowering, CBOR and zstd byte layout. These are covered by golden, round-trip and torn-byte tests.
- **Gaps:** any construct bboxy450-tlc cannot check is a tracked gap. It gets a bboxy450-tlc task in bboxy450-term (where the crate lives), and the model is never left Java-only silently. Each model's coverage entry names its assumptions and bounds.

## 6. Verification matrix

| Requirement (operator) | Evidence | Phase |
|---|---|---|
| (1) parsers and lowerers for all three; the target resumes | `roundtrip`, `loss` and fixpoint tests per provider; real-CLI resume against a mock endpoint (P0 → P6) | P0, P3–P6 |
| (2) compact streaming and append binary format | `torn`, `tail`, `size` and compatibility tests; `LcAppendLog` | P2 |
| (3) a watcher that tails a live log, with a sink and a resumable cursor | `watch` test with SIGKILL restarts; `LcWatcher` | P7 |
| (4) live switch in term in all directions, A→B→A | term `make test-session-switch`; `LcSessionSwitch`; #348 amendment signed | P8 |
| (5) "Bboxy" as the user-facing name | CLI help and loss-report golden output say `bboxy`; a check refuses `bboxy450-harness` in user-facing strings except the binary name | P1 |
| (6) #4935 becomes the Claude parser and sanitizer | P3 diff moves `import::claude` into `parse::claude` and `sanitize`; the harness `import claude` calls the pinned crate; the #4935 goldens still pass | P3, P5 |
| d4947 sanitized bundles | the sanitizer planted-secret test + control; bundle checks | P3 |
| d4949 from scratch, credited | no dependency on ctx/casr/Harbor/session-bridge (a `cargo tree` deny check); ATTRIBUTION.md "Inspired by" rows | P1 |
| validation, on real clients | the night run: convert and resume real local sessions (sanitized stats only) | after P8 |

## 7. Phases (each one is a task-sized Executor assignment; parallel where marked)

| Phase | Outcome | Depends on | Verify |
|---|---|---|---|
| **P0 resume probe** | An unattended script, `probes/resume.sh` (in the new repo, or in the harness `scripts/` if P1 isn't up yet). It works in a throwaway `HOME`/`CODEX_HOME` with **no real credentials**. It starts a local mock Anthropic Messages endpoint and a mock OpenAI Responses endpoint, each returning a canned reply and recording the request. It hand-writes a minimal two-turn Claude session and a minimal two-turn Codex rollout, then runs `claude -p --resume <id> "next"` (with `ANTHROPIC_BASE_URL` pointing at the mock and a dummy key) and `codex exec resume <id> "next"` (a mock `model_provider` `base_url`). It also probes: Claude resume by file path; whether `api-request` lines are needed; Codex resume with no `state_5.sqlite` row (backfill); a non-interactive Codex importer entry point; and whether Codex accepts on resume a `reasoning` item that has `summary` text and no `encrypted_content` (this decides the Claude → Codex reasoning row of [[lc-ir]] §5). Output: a decision record of which entry points work, per [[session-switch]] §3 | — | for each case, the CLI exits 0 **and** the mock's recorded request contains the first turn's text. The verdict is the script's exit code |
| **P1 bootstrap** | Repo `bboxy450/bboxy450-log-converter` (the operator creates it and records merge authorization). Crate skeleton, ATTRIBUTION, headers, KB, `make test`/`make test-formal`, the dependency deny check, IR types + CBOR codec with integer-key golden | — (∥ P0) | `make test` exit 0 |
| **P2 bxl** | Writer, reader, tail, recovery, index sidecar, seal and rotation; `LcAppendLog` + 3 controls | P1 | torn/tail/size/compat tests; formal parity |
| **P3 Claude** | Port #4935 into `parse::claude` + `lower::claude` (raw same-provider) + `sanitize` | P1 (∥ P2, P4) | roundtrip, determinism, sanitizer and control; 100 MB memory budget |
| **P4 Codex** | `parse::codex` + `lower::codex` (raw same-provider) | P1 (∥ P2, P3) | roundtrip, determinism |
| **P5 Bboxy** | `parse::bboxy` + `lower::bboxy` (protocol v0.4); the harness `import <provider>` uses the pinned crate (a harness PR) | P1, P3 | roundtrip; harness `check_transcript` on lowered fixtures; harness `make test` |
| **P6 cross-provider** | The loss table, implemented with LossReport; the Claude id rewrite; Codex `session_meta` synthesis | P0, P3–P5 | `loss` tests; the fixpoint test; real-CLI resume of every cross direction against mocks (P0 harness) |
| **P7 watch** | Watcher + `FileSink` + the `Sink` trait; `LcWatcher` + 3 controls | P2, P3, P4 | `watch` test with SIGKILL; formal parity |
| **P8 term switch** | bboxy450-term: `ctl seat switch`, input refusal, rollback, the child-process converter, #348 rows; `LcSessionSwitch` + 4 controls | P0, P5, P6 | `make test-session-switch`; formal parity; Oracle signs the #348 rows |
| P9 extras | ATIF export; per-provider dictionaries trained on the synthetic corpus | P2, P6 | ATIF golden; size budget |

**Critical path:** P0 + P1 → P3 → P6 → P8. P2, P4 and P7 run beside it.

**Highest risk:** P0. No synthesized session is proven to resume yet (#4953). If P0 fails for a client, the matching rows of [[session-switch]] §3 change before P6 starts.

Owner: Nyx Copperwing <ARCHITECT>.
