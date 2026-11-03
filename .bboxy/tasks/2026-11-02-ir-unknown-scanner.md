---
id: 01M6F6XPKG5CXV6BPFHCXND4A3
number: 5979
type: task
title: 'IR edge rule and unknown scanner: projectors never drop (Native + ext), ir-scan catalogues unknowns and proposes additions, land.sh refreshes the lock, reproject'
status: accepted
role: executor
links:
  - { to: "[[log-ir-v1]]", rel: implements }
  - { to: "[[d5978-unknown-is-preserved-and-promoted]]", rel: implements }
  - { to: "[[2026-11-02-log-ir-v1-compat]]", rel: depends-on }
allowed_paths:
- src/import/
- src/backend/acp_agent.rs
- src/ir_scan.rs
- src/ir_scan/
- src/main.rs
- src/frontends/
- src/engine/
- scripts/land.sh
- scripts/test.py
- scripts/ir-compat.py
- Makefile
- tests/
- verification/contracts.toml
- verification/coverage.toml
verify: make test
created: 2026-11-02T23:59:00Z
updated: 2026-11-02T23:59:00Z
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness, own worktree off origin/main. Spec: `.bboxy/specs/log-ir-v1.md` §7 and §8 (binding), decision d5978. **Depends on #5975**: it owns `src/protocol/` and adds `Item::Native{provider, record_type, raw, blob}` and payload-carrying `Unknown(raw)` in its Part B. Start after #5975 Part B lands, or coordinate through the Oracle if it is still open. You do not edit `src/protocol/`, the lock by hand, or any existing frozen corpus file.

## Part A: the edge never drops (spec §7)

1. **Importer** (`src/import/mod.rs`): every `_ => {}` arm (309, 355, 431) and every skip (215 sidechains, 222–224 non-message records, 230 meta, 252 content blocks, 316–318 command lines, 365, 379, 396) becomes either a typed mapping or `Item::Native` with the verbatim record. Mapped records put their unconsumed keys into `Envelope.ext["claude-code"]` / `["codex"]`; keep a per-provider allowlist for keys already carried (ids → `native`, timestamps → `ts_ms`). Sidechains: a `Native` item now; full sub-session projection is a later addition.
2. **ACP backend** (`src/backend/acp_agent.rs`): the unknown-update arm (1210) writes `Item::Native{provider: "acp:<agent name>", record_type: <update kind>, raw}` instead of a Warn notice; `_meta` of any update goes to `ext["acp"]`; the placeholder arms (1580 non-text content, 1594 terminal, 1626 image input) keep the original value in `ext` beside the typed `Unknown`/`Other`; `rawOutput` is stored in `ext` until #5975 Part B types it.
3. **Faces**: `Native` renders collapsed (provider · kind · size); an `Unknown` item renders its tag. Seat line mode prints one line each.
4. **`bboxy450-harness reproject <session> [--out DIR]`**: derive a new session log from an old one by re-running the current projector over its `Native`/`ext` data, as a fork-like derived log with `SessionMeta.fork` pointing at the source. The source is never rewritten.
5. **`tests/ir_edge.rs`**: for each projector, feed a never-seen record kind (and a known record with an extra field) and assert the raw value round-trips through a `Native` item / `ext` byte-for-byte after redaction; `check_authored` passes. **Broken control:** restore one `_ => {}` arm behind a test switch; the test must fail.

## Part B: the scanner (spec §8)

6. **`bboxy450-harness ir-scan [--json] [--propose DIR] PATH…`** in `src/ir_scan.rs`: catalogue by fingerprint (sorted key paths with value kinds) over `Native`, `Unknown` payloads, `ext` keys and unknown fields of known types (compare raw object keys with `protocol.schema.json`). One redacted sample ≤ 4 KiB per fingerprint; counts, sessions, first/last seen. Store under `~/.bboxy450-harness/ir-unknowns/`, merged across runs.
7. **`--propose DIR`**: per fingerprint, write a schema fragment (additive only), a projector-arm skeleton, a fixture line into `tests/fixtures/ir-v1/pending/`, and a task record in the brief format (`.bboxy/tasks/<date>-ir-extend-<fingerprint>.md`, frontmatter with id and number). Run `python3 -I scripts/ir-compat.py check` against the fragment applied to a copy of the schema; a refusal is a scanner bug and the test for it fails.
8. **Session close hook**: after `SessionClosed`, scan that session's log on `bbt_rt`, ≤ 1 ms CPU between awaits, bounded by log size; a new fingerprint emits one Info notice. Never on the UI thread; never delays close (law never-block).
9. **`make test`**: `scripts/test.py` runs `ir-scan --propose <tmp> tests/fixtures/` as a dry run and fails when a fixture holds an unknown with no pending proposal in `tests/fixtures/ir-v1/pending/`.
10. **Lock refresh is self-service.** The harness has no `land.sh`; the merge gate is `make test`. `make lock-ir` (exists) refuses anything non-additive, so the executor who lands an addition runs it and commits the lock with the change. Make `make test-ir` fail with the hint `run make lock-ir` when the schema has additions the lock does not yet carry, so a stale lock cannot land.
11. **`tests/ir_scan.rs`**: a fixture with two unknown shapes yields two fingerprints, two proposals that pass `ir-compat`, and a task record that passes `scripts/checks.py kb-records`. **Broken controls:** (a) a proposal generator that renames an existing field must be refused by the gate and the test must fail; (b) the session-close hook with the budget removed must fail the budget assertion.
12. Record contracts in `verification/contracts.toml` (edge rule: implementation witness `tests/ir_edge.rs`; scanner: `tests/ir_scan.rs`) and coverage for the new sources.

**Report** in `.bboxy/reports/`: file:line per step, `make test` exit, both broken-control outputs, the fingerprints found over the current fixtures (there will be some: the corpus has ACP sessions), and the size growth of one re-imported Claude Code session before/after Part A.

**Accepted (Oracle, Vesper Greywind, 2026-11-02).** Landed as three commits on harness main (edge preservation, ir-scan catalogue and proposals, reproject). Oracle full gate exit 0 on the stacked branch; parity no regression.
