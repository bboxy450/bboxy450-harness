---
id: 01M6F4R15RSMBSJ9NC575QVH5T
number: 5975
type: task
title: 'Log IR v1: compatible by construction (byte pass-through, E9 durability from seq, 1.0, check_authored, corpus test), then the 1.1 additions incl. usage'
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

**Hard rule.** `make test-ir` must stay green on every commit. Never edit `verification/protocol-v1.lock.json` by hand or any existing `tests/fixtures/ir-v1/*` file. After an addition, refresh the lock with `make lock-ir` (it refuses anything that is not additive) and commit the result with the change. If the gate refuses your change, the change is wrong (spec E1/E2). Message the Architect only if you believe the gate is wrong, and give evidence.

## Part A: compatible by construction (no wire change)

1. **Fork copies bytes (E6).** `src/store/fork.rs:173` re-serializes each parent event, which turns future variants into `{"type":"unknown"}` and drops unknown fields. Copy parent lines `1..=at` verbatim. Rewriting the session header (`fork.rs:168`) is authorship and stays.
2. **Replay forwards bytes (E6).** Durable envelopes read from disk keep their original line. The Unix server (`src/gateway/unix.rs:63`) and `--json` output write that line when present; only freshly authored events are serialized. In-process faces keep using the parsed value.
   - **Recommended mechanism, by construction:** `Envelope` gains `raw: Option<Arc<str>>` (skipped by serde, ignored by `PartialEq`), filled when parsed from a line. A hand-written `Serialize` emits `raw` verbatim through `serde_json::value::RawValue` (feature `raw_value`) when it is present, and the derived form otherwise. Then *every* serialization path preserves unknown data, including ones nobody has written yet. Hazards you must handle: a parsed envelope that is mutated must clear `raw` (today the only rewrite is fork's header, which builds a fresh envelope), and the golden tests must not compare `raw`. Carrying the line beside the envelope is also acceptable if the corpus test passes; say which you chose and why.
3. **Readers take durability from `seq` (E9).** `src/store.rs:144` and `:215` apply `Event::is_durable()` to parsed events, so a log with one unknown durable event kind cannot be opened. On read, durable means `seq.is_some()`; `is_durable()` is used only when authoring. Audit every other read-side use (`grep is_durable`): `src/import/watch.rs:134` is authoring and stays.
4. **Version acceptance and 1.0.** Readers accept `major ∈ {0, 1}` and refuse only a major greater than their own: `src/frontends/gui/model.rs:110` and gateway `Refused::Version`. Then set `PROTOCOL = {1, 0}` (the lock, spec §2) and regenerate the protocol goldens with `UPDATE_GOLDENS=1`. The frozen `tests/fixtures/ir-v1/` corpus is untouched and must still open.
5. **`laws::check_authored(&[Envelope])`.** It runs `check_transcript` and also refuses any `Unknown` variant anywhere in an authored event (E5). Call it wherever tests check our own output (e2e, headless, ACP, import). `check_transcript` stays tolerant for reading.
6. **`tests/ir_v1_corpus.rs`:**
   - every line of `tests/fixtures/ir-v1/*.jsonl` parses as `Envelope`, with no top-level `Event::Unknown`, and `check_transcript` passes per file;
   - a future-data case: a temp log with one line whose event type is unknown **and durable (it carries the next `seq`)** and which has an unknown field is opened (E9), replayed over Unix and forked; the bytes of that line are identical in all three, and the replayed seq list is gapless;
   - **broken controls:** (a) the old re-serializing fork path behind a `#[cfg(test)]` switch must fail the byte-identity assertion; (b) `is_durable()` restored on the read path must fail the open.
8. **`Unknown` keeps its payload (E4).** Every `#[serde(other)] Unknown` becomes an untagged catch-all `Unknown(serde_json::Value)` (serde's variant-level `untagged`) holding the whole value, tag included; `kind()` of such an item is `ItemKind::Unknown`; `apply` ignores deltas to it. `future_variants_and_fields` asserts the payload round-trips. The schema export shows one catch-all per union and `make test-ir` must still pass (the gate accepts a named `unknown` or an untagged catch-all).
7. Record the contract in `verification/contracts.toml` (E6 pass-through: sources, witness `tests/ir_v1_corpus.rs`, assumptions) and list `src/store/fork.rs` coverage in `verification/coverage.toml`.

## Part B: 1.1 additions (spec §3, §4, §4a; each optional or a new variant)

1. Add the types exactly as named in spec §3–§4a, including `Item::Native { provider, record_type, raw: Value, blob: Option<BlobRef> }` (§7; the projector rules that write it are task #5979's, not yours). Every new union or enum has `#[serde(other)] Unknown`. Every new field is `Option` or `#[serde(default, skip_serializing_if = …)]`. Regenerate `protocol.schema.json` with `UPDATE_GOLDENS=1`. `make test-ir` must pass with no lock change.
2. Every in-repo face gets display arms (GUI, seat line mode, ACP export, web replay if present). Unknown kinds keep the neutral placeholder.
3. **Usage (§4a), writers:**
   - The engine writes `scope`, `backend{engine, provider id}`, `model`, `sources` and `cost` with `price_source`.
   - The ACP backend maps `usage_update` to `Item::Usage{scope: session, context_used, context_window}`. It is dropped today at `src/backend/acp_agent.rs:1209`; if p6 has already landed this, extend it to the spec shape.
   - The importer maps Claude `message.usage` and Codex `token_count` to call-scope `Item::Usage` with `backend{import, claude-code|codex, cli version}`. They are dropped today, since `TurnEnded.usage` is always `None`.
   - Unreported numbers follow the never-invented rule.
4. **`bboxy450-harness usage [--json] PATH…`** follows §4a. Add `tests/usage_report.rs`:
   - the report over `tests/fixtures/ir-v1/` equals the committed `tests/fixtures/usage/ir-v1-report.json`, with the 0.4 placeholders counted as missing, not zero;
   - a session with both call and turn records is counted once (finest scope);
   - a forked pair counts its prefix once;
   - **broken control:** a fork that drops `Item::Usage` changes the totals and must fail.
5. `laws::check_authored` enforces §4a: a writer at ≥ 1.1 writes `scope`, `backend` and `source`, and names every placeholder in `unreported`. Scopes are never summed together by the report.
6. Set `PROTOCOL = {1, 1}`.
7. Add one frozen corpus file of 1.1 output, `tests/fixtures/ir-v1/v1.1-all-items.jsonl`, containing every new variant and one Usage record per scope, then `make lock-ir` and commit the refreshed lock with it.

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
- the 1.1 corpus file path.
