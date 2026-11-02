---
id: 01M6F4R15RSMBSJ9NC575QVH5T
number: 5975
type: task
title: 'Log IR v1: compatible by construction (byte pass-through, major 0≡1, check_authored, corpus test), then the v1.0 additions incl. usage'
status: open
role: executor
links:
  - { to: "[[log-ir-v1]]", rel: implements }
  - { to: "[[d5974-log-ir-v1-only-add]]", rel: implements }
allowed_paths:
- src/protocol/
- src/store.rs
- src/store/
- src/gateway/
- src/frontends/
- src/import/
- src/backend/acp_agent.rs
- src/engine/
- src/provider/
- src/main.rs
- tests/
- protocol.schema.json
- verification/contracts.toml
- verification/coverage.toml
verify: make test
created: 2026-11-02T23:00:00Z
updated: 2026-11-02T23:00:00Z
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness, own worktree off origin/main. Spec: `.bboxy/specs/log-ir-v1.md` (binding). Decision: `.bboxy/decisions/d5974-log-ir-v1-only-add.md`. The Oracle dispatches and may split this into Part A and Part B; **Part A lands first**.

**Hard rule.** `make test-ir` must stay green on every commit. Never edit `verification/protocol-v1.lock.json` or `tests/fixtures/ir-v1/*`; those belong to the Architect. If the gate refuses your change, the change is wrong (spec E1/E2). Message the Architect only if you believe the gate is wrong, and give evidence.

## Part A: compatible by construction (no wire change)

1. **Fork copies bytes (E6).** `src/store/fork.rs:173` re-serializes each parent event, which turns future variants into `{"type":"unknown"}` and drops unknown fields. Copy parent lines `1..=at` verbatim. Rewriting the session header (`fork.rs:168`) is authorship and stays.
2. **Replay forwards bytes (E6).** Durable envelopes read from disk keep their original line, e.g. an `Arc<str>` beside the parsed `Envelope`. The Unix server (`src/gateway/unix.rs:63`) and `--json` output write that line when present; only freshly authored events are serialized. In-process faces keep using the parsed value.
3. **Version acceptance.** Readers accept `major ∈ {0, 1}` and refuse only a major greater than their own: `src/frontends/gui/model.rs:110` and gateway `Refused::Version`. `PROTOCOL` stays `0.4` in Part A.
4. **`laws::check_authored(&[Envelope])`.** It runs `check_transcript` and also refuses any `Unknown` variant anywhere in an authored event (E5). Call it wherever tests check our own output (e2e, headless, ACP, import). `check_transcript` stays tolerant for reading.
5. **`tests/ir_v1_corpus.rs`:**
   - every line of `tests/fixtures/ir-v1/*.jsonl` parses as `Envelope`, with no top-level `Event::Unknown`, and `check_transcript` passes per file;
   - a future-data case: a temp log that has a line with an unknown event type and an unknown field is opened, replayed over Unix and forked; the bytes of that line are identical in all three;
   - **broken control:** the old re-serializing fork path behind a `#[cfg(test)]` switch must fail the byte-identity assertion.
6. Record the contract in `verification/contracts.toml` (E6 pass-through: sources, witness `tests/ir_v1_corpus.rs`, assumptions) and list `src/store/fork.rs` coverage in `verification/coverage.toml`.

## Part B: v1.0 additions (spec §3, §4, §4a; each optional or a new variant)

1. Add the types exactly as named in spec §3–§4a. Every new union or enum has `#[serde(other)] Unknown`. Every new field is `Option` or `#[serde(default, skip_serializing_if = …)]`. Regenerate `protocol.schema.json` with `UPDATE_GOLDENS=1`. `make test-ir` must pass with no lock change.
2. Every in-repo face gets display arms (GUI, seat line mode, ACP export, web replay if present). Unknown kinds keep the neutral placeholder.
3. **Usage (§4a), writers:**
   - The engine writes `scope`, `backend{engine, provider id}`, `model`, `sources` and `cost` with `price_source`.
   - The ACP backend maps `usage_update` to `Item::Usage{scope: session, context_used, context_window}`. It is dropped today at `src/backend/acp_agent.rs:1209`; if p6 has already landed this, extend it to the spec shape.
   - The importer maps Claude `message.usage` and Codex `token_count` to call-scope `Item::Usage` with `backend{import, claude-code|codex, cli version}`. They are dropped today, since `TurnEnded.usage` is always `None`.
   - Unreported numbers follow the never-invented rule.
4. **`bboxy450-harness usage [--json] PATH…`** follows §4a. Add `tests/usage_report.rs`:
   - the report over `tests/fixtures/ir-v1/` equals the committed `tests/fixtures/usage/ir-v1-report.json`;
   - a forked pair counts its prefix once;
   - **broken control:** a fork that drops `Item::Usage` changes the totals and must fail.
5. `laws::check_authored` enforces §4a: a writer at ≥ 1.0 writes `scope`, `backend` and `sources` for each number it writes.
6. Set `PROTOCOL = {1, 0}`.
7. Add one frozen corpus file of v1.0 output, `tests/fixtures/ir-v1/v1.0-all-items.jsonl`, containing every new variant. The Architect then re-locks; hand the file over in your report and do **not** edit the lock.

**Render-gap evidence**, from the 2026-11-02 survey at 3c78fcf. These are importer and ACP projector follow-ups. They are in scope only where Part B needs a writer.
- **Importer, `src/import/mod.rs`:**
  - sidechains skipped (215); non-message records skipped (222–224); image blocks (252); thinking signature (288, 300);
  - command lines (316–318); compact summary misread as a prompt (312–330);
  - Codex session_meta/turn_context fields (338–347); token_count and exec events (355); compacted (431);
  - developer role (379); input_image (365); encrypted_content (396); namespace (402–416); failed=false (427);
  - native id discarded (499); usage None (136).
- **ACP, `src/backend/acp_agent.rs`:**
  - non-turn updates (1085); non-text chunks (1091); title as name (1319–1324); end_line (1351);
  - placeholders (1580, 1594, 1626); whole-file diff (1584–1590, 1605); dropped updates (1204–1210).

**Report.** A committed report in `.bboxy/reports/`. It gives:
- each step's file:line and mechanism;
- `make test` and `make test-ir` exits;
- the broken-control outputs;
- the v1.0 corpus file path.
