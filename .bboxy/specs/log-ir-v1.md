---
id: 01M6F4R15R4FFCRBCT162Q7CDQ
number: 5973
type: spec
title: "Spec: bboxy log IR v1 — the universal render IR; evolution law, versioning, extension slot, v1.0 additions, gate"
status: baselined
links:
  - { to: "[[protocol]]", rel: constrains }
  - { to: "[[d5974-log-ir-v1-only-add]]", rel: constrained-by }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: builds-on }
  - { to: "[[lc-ir]]", rel: builds-on }
  - { to: "[[log-converter-sota]]", rel: builds-on }
  - { to: "[[native-session-formats]]", rel: builds-on }
  - { to: "[[log-is-truth]]", rel: constrains }
  - { to: "[[interfaces]]", rel: index }
created: 2026-11-02T23:00:00Z
updated: 2026-11-02T23:00:00Z
---
# Bboxy log IR v1

**What it is.** The [[protocol]] `Envelope` JSONL is the one contract between every backend (engine, ACP agents, native-log import) and every face (GUI, web, seat, term pane, ACP export). It is both the store format and the render IR ([[d5972-our-log-format-is-the-center]]). Logs live for years and are read by builds that were never compiled against them, so the contract is protobuf-shaped: **only add**.

**The field-level baseline is a file, not prose:** `verification/protocol-v1.lock.json`. It holds 37 definitions, every field with its type signature and required flag, every variant, the retired-name list and the frozen corpus hashes. It was locked from `protocol.schema.json` at 3c78fcf (protocol 0.4). This spec is the law; the lock is the baseline it is checked against.

## 1. Evolution law

| # | Rule | Checked by |
|---|---|---|
| E1 | **Only add.** A change may add a definition, a field that is optional (`Option<T>`, or `#[serde(default)]` with `skip_serializing_if`), a variant to a union or enum, or a value to a string enum | `ir-compat check` |
| E2 | **Never change or remove.** No removed definition, field or variant. No renamed name. No changed type, tag key or definition shape. No optional field made required. No new required field, including inside a new variant | `ir-compat check` |
| E3 | **Meaning is fixed too.** An existing field keeps its unit, range and semantics (e.g. `ts_ms` is Unix milliseconds, `cost_micros_usd` is micro-dollars). A new meaning needs a new field | review: [[protocol]] change log entry naming the field; the Architect signs |
| E4 | **Every union and enum has `unknown`, and `Unknown` keeps its payload.** A reader maps a variant it does not know to `Unknown(raw)` holding the whole value, tag included; it counts it, renders a neutral placeholder naming the tag, and never fails the stream. Nothing a reader cannot type is lost in memory either | `ir-compat check` (one named `unknown` or one untagged catch-all per union); `tests/protocol_golden.rs::future_variants_and_fields` asserts the payload round-trips |
| E5 | **Writers never write `unknown`.** It is a reader-side value | `laws::check_authored` (brief [[2026-11-02-log-ir-v1-compat]]) |
| E6 | **Only the author serializes.** A component that stores, forwards, forks or replays an envelope it did not create passes on the original line bytes. Unknown fields and variants therefore survive store → replay → fork → socket in any build | `tests/ir_v1_corpus.rs` future-data case (brief) |
| E7 | **Retired names are never reused.** Removing a name is impossible (E2). A name that is deprecated stays readable forever; writers may stop writing it only if it is optional. Its name goes into `retired` in the lock, and the gate refuses any new use of it | `ir-compat check` |
| E8 | **Old logs are immutable.** `tests/fixtures/ir-v1/*.jsonl` is the frozen corpus. A file is never edited. Each minor adds a new `v<maj>.<min>-*.jsonl` file | `ir-compat check` (sha256 per file); `tests/ir_v1_corpus.rs` parses every line |
| E9 | **Readers take durability from `seq`.** A read envelope is durable iff `seq` is `Some`. `Event::is_durable()` is a writer-side function and is never applied to a parsed event. Otherwise one new durable event kind makes every older build refuse to open newer logs, which is what `src/store.rs:144` and `:215` do today | `tests/ir_v1_corpus.rs` future-data case: a log with an unknown durable event (with `seq`) opens, replays and forks (brief) |

**Unknown fields** are ignored by readers (serde default), and preserved by E6 rather than by a field-level map on every struct.

## 2. Versioning

- `Version { major, minor }` on every envelope is the writer's version.
- **Minor.** Bumped once per release that adds anything. A reader at any minor reads every log of its major: newer data arrives as `Unknown` or as ignored fields (E4), older data has `None` or defaults.
- **Major** means a different IR. It is created only by an Architect decision that names a reader for every older major, with no loss. There is no planned major 2.
- **0.x is v1.** The 0.4 surface is exactly the locked baseline, so 0.4 logs are v1 logs. Readers accept `major ∈ {0, 1}`. A reader refuses (`Refused::Version`) only a major greater than its own.
- **1.0 is the lock, not a feature.** `PROTOCOL` becomes `{1, 0}` with no schema change, in the same cut that makes readers compatible by construction (brief Part A). The §4 additions are **1.1**. From then on, each minor is one release of additions, and "1.x" reads as "locked".
- **Re-lock.** The Architect re-locks only after `ir-compat check` passes on the new schema. Additions become part of the baseline; nothing already locked can leave it. A re-lock is recorded in this spec's change log with the commit.

## 3. Extension slot and provenance

- **`Envelope.ext: BTreeMap<String, Value>`** (default empty, not serialized when empty). Keys are a namespace: `claude-code`, `codex`, `acp:<agent id>`, `engine`, or `x-<vendor>`. The value is opaque.
  - A face never needs `ext` to render.
  - A face may render a namespace it knows.
  - `ext` is counted in the 256 KiB envelope limit and passes through the redactor like any other value.
  - When data proves general, a typed field is added (E1). The `ext` key may keep being written, but no reader may depend on it.
- **`Envelope.native: Option<Native { id: String, parent: Option<String>, ts_ms: Option<u64> }>`**: the source record's identity, for import and watch.
- **`SessionMeta.source: Option<Source { harness: String, native_id: Option<String>, version: Option<String>, parser_revision: Option<u32> }>`**: which harness wrote the original session.

## 4. v1.1 additions (all additive; the gate already admits them)

These come from the render-gap survey (2026-11-02, file:line evidence in [[2026-11-02-log-ir-v1-compat]]). Each is optional or a new variant.

| Area | Addition |
|---|---|
| Usage and model | §4a; `TurnStarted.model: Option<String>` |
| Media | `InputBlock::{Audio{blob}, Resource{uri, mime, blob?}, ResourceLink{uri, title?}}`; `OutputBlock::{Audio{blob}, Resource{uri, mime, blob?}}` |
| Terminal | `OutputBlock::Terminal{terminal, exit?}`, referencing the `Terminal` item, so ACP terminal content is not text |
| Diffs | `OutputBlock::Diff.{old_text?, new_text?, change?: ChangeKind}` (ACP `oldText`/`newText` kept) |
| Tool identity | `ToolCall.{title?, server?, raw_output?: Value}` (MCP server or namespace; title separate from name) |
| Citations | `OutputBlock::Citations{entries: [Citation{url, title?, snippet?}]}` |
| Subagents | `Subagent.{name?, call?: ItemId}`; sidechains import as their own session plus this item |
| Context and modes | `Event::ContextChanged{mode?, permission_mode?, config?: map}` (durable) |
| Commands, hooks, system | `Item::Command{name, args}`, `Item::Hook{event, name, outcome, output?}`, `Item::SystemMessage{role: system\|developer, text}` |
| Nothing dropped | `Item::Native{provider, record_type, raw: Value, blob?: BlobRef}` (§7): the verbatim record a projector could not map. `Envelope.ext[provider]` carries the fields of a mapped record the projector did not consume |
| Errors | `ItemStatus::Failed.code?: String` |
| Sealed reasoning | `Reasoning.provider_opaque` keeps its type (`any`). Its value is fixed as `{provider, form, data}` and is never rendered or sent to another provider |
| Plans | `PlanEntry.id?: String` |

### 4a. Usage: first-class, never invented, always preserved

The operator requires cost analysis across backends from the logs alone. Today's `Usage { input_tokens, output_tokens, cached_input_tokens, cache_creation_input_tokens, reasoning_tokens, cost_micros_usd }`, `Item::Usage` and `TurnEnded.usage` stay as they are. Everything below is added.

**Where usage is written.**

| Scope | Carrier | Writer |
|---|---|---|
| one model call | `Item::Usage { usage }` with `usage.scope = call` | engine per provider response; importer per Claude `message.usage` / Codex `token_count`; ACP when the agent reports per call |
| one turn | `TurnEnded.usage`, `scope = turn` | the backend. When nobody reports a turn total it is the sum of that turn's call items, and every summed number is marked `derived` |
| session so far | `Item::Usage`, `scope = session` | ACP `usage_update` (cumulative `used`/`size`), native cumulative counters |

Totals across sessions are computed offline, never written.

**Added to `Usage`** (all optional):
- `scope: Option<UsageScope>`, where `UsageScope` is `call | turn | session | unknown`.
- `model: Option<String>`: the exact model id that was billed.
- `backend: Option<Backend>`, where `Backend { kind: BackendKind, name: String, version: Option<String> }` and `BackendKind` is `engine | acp_agent | import | unknown`. For the engine, `name` is the provider id. For an ACP agent it is the agent name and version. For an import it is the native harness (`claude-code`, `codex`) and its version.
- `request_id: Option<String>`.
- `context_used: Option<u64>`, `context_window: Option<u64>`: as reported (ACP `usage_update.used`/`size`).
- `cost: Option<Cost>`, where `Cost { amount_micros: u64, currency: String, source: NumberSource, price_source: Option<String> }`. The currency is ISO 4217. `price_source` names the price table and its date, e.g. `anthropic-pricing@2026-11-01`, or `provider` when the bill came with the response. When the currency is USD, `cost_micros_usd` is still written with the same value, for 0.x readers.
- `source: Option<NumberSource>`: where the token numbers of this record came from. `NumberSource` is `reported | derived | estimated | unknown`. One record has one source: a provider response is `reported`, a turn total summed from calls is `derived`, a number from a price table or a tokenizer is `estimated`. Cost has its own `source` (above) because it is the number most often estimated while the tokens are reported.
- `unreported: Vec<String>` (skip if empty): the names of required numbers that were written `0` because nothing was reported.

**Never invented.** An optional number that was not reported is written as `None`. `input_tokens` and `output_tokens` cannot be `None` (E2), so an unreported one is written `0` and named in `unreported`. A record with nothing reported at all is not written.

| Writer version | Requirement |
|---|---|
| ≥ 1.1 | must write `scope`, `backend` and `source`; `unreported` whenever a required number is a placeholder |
| ≤ 1.0 logs | read with `source = unknown`. The corpus has `input_tokens: 0, output_tokens: 0` with nothing reported (`tests/fixtures/ir-v1/v0.4-all-items.jsonl`); that is exactly what `unreported` names |

**Preservation.** Usage is ordinary durable data, so it travels with the log:
- Store, replay and socket carry it by E6.
- A fork copies it byte for byte.
- A switch or handoff forks the log, so the new session carries the old session's usage below the fork point and adds its own above it. The cost report counts a forked prefix once, by `(fork session, seq)`.
- Import writes it from the native records. It is never dropped as "metadata".

**Offline query.** `bboxy450-harness usage [--json] PATH…` reads session directories or JSONL files and prints totals by `backend.kind/name`, `model`, `currency` and `source`. Rules:
- **Scopes are never added together.** Per turn, use the finest scope present: the sum of its `call` records if any, else the `turn` record, else nothing. `session` records are cumulative, so a session contributes the last one only when it has no `call` or `turn` records at all.
- Forked prefixes are counted once, by `(fork session, seq)`.
- Placeholders (`unreported`) count as missing, not as zero. Rows with `source ≠ reported` are reported separately, never folded into reported totals.

**Checks** (brief [[2026-11-02-log-ir-v1-compat]]):
- `laws::check_authored` enforces the ≥ 1.1 writer rules.
- `tests/ir_v1_corpus.rs` checks that usage survives fork: the totals of a forked log's prefix equal the parent's at the fork point.
- The usage report over the frozen corpus equals a committed expected JSON.
- A broken control (a fork that drops `Item::Usage`) must fail.

The render rule for every addition: a face that predates it shows the item kind's neutral placeholder (E4). A face that knows it renders it. Parity fixtures (R27) gain one replay per new kind.

## 5. Gate

| Command | Exit 0 means |
|---|---|
| `python3 -I scripts/ir-compat.py check` | the schema keeps every locked definition, field, variant, type and tag; every union has `unknown`; no retired name is used; the frozen corpus is byte-identical |
| `python3 -I scripts/ir-compat.py control` | broken control: 9 forbidden mutations of the current schema are each refused, and 3 allowed additions are accepted |
| `make test-ir` | both; also in `make test` (`scripts/test.py`), after `tests/protocol_golden.rs` has proven the schema file is freshly generated |

**What the gate does not prove.**
- Semantics (E3), which is a review rule.
- serde behaviour that the schema does not express, e.g. `#[serde(default)]` without `skip_serializing_if`. Byte-level round trip is `tests/protocol_golden.rs`.
- E5, E6 and E9 until [[2026-11-02-log-ir-v1-compat]] lands. **Known violations** today: fork re-serializes (`src/store/fork.rs:173`); the GUI drops major ≠ 0 (`src/frontends/gui/model.rs:110`); the store refuses to open a log with an unknown event kind (`src/store.rs:144`, `:215`, `is_durable()` applied on read).

## 6. Where the IR lives

The IR is the contract of the whole suite, not of one consumer. Its home is a crate in the bboxy450-term workspace (d5087), consumed by the harness the way it consumes `bbt-ui`. The move is sequenced in [[d5977-rich-view-dependency-direction]]; until then the types, the lock, the corpus and the gate stay in this repository and move together.

## 7. Nothing is dropped at the edge

The only data a mapping can ever be built from is the data the edge kept. From 1.1 every projector (native import, ACP backend, any future backend) obeys:

1. **Unmapped record → `Item::Native`.** A native record, an ACP `session/update` kind or a notification the projector has no arm for becomes `Item::Native { provider, record_type, raw, blob }`: `raw` is the verbatim value after the redactor; over 64 KiB the value goes to a blob and `raw` keeps its first 4 KiB. It is a durable item of the turn it arrived in, or `turn: None`. It replaces today's `_ => {}` arms (`src/import/mod.rs:309, 355, 431`) and the ACP Warn-notice arm (`src/backend/acp_agent.rs:1210`). A face renders it collapsed: provider, kind, size.
2. **Mapped record, unconsumed fields → `ext`.** A projector records which keys it read; the rest of the object, minus a per-provider allowlist of fields already carried elsewhere (ids now in `native`, timestamps now in `ts_ms`), goes to `Envelope.ext[<provider>]` on the item it produced. ACP `_meta` goes there whole.
3. **Never silently narrowed.** A value the typed field cannot hold (a stop reason with no variant, a content block with no arm) is not coerced to the nearest value: the typed field gets its `Unknown`/`Other` **and** the original goes to `ext`. The placeholder-text arms (`acp_agent.rs:1580, 1594, 1626`) are retired.
4. **Writers still never emit `unknown`** (E5): `Native` and `ext` are typed; `Unknown` stays a reader-side value.

Consequence: promotion is retroactive. Once a mapping lands, `bboxy450-harness reproject <session>` derives a new session log from the old one's `Native` and `ext` data, as a fork-like derived log. The original is never rewritten (E8, [[log-is-truth]]).

## 8. The unknown scanner: the IR grows from observed data

`bboxy450-harness ir-scan [--json] [--propose DIR] PATH…` (brief [[2026-11-02-ir-unknown-scanner]]).

- **Catalogue.** Walks logs and collects every `Item::Native` (provider + record_type), every `Unknown` payload (enum + tag), every `ext` namespace and key, and every unknown field of a known type (raw object keys absent from the schema for that type). An entry is keyed by a shape fingerprint (sorted key paths with value kinds) and holds count, sessions, first and last seen, and one redacted sample ≤ 4 KiB. It is written to `~/.bboxy450-harness/ir-unknowns/<fingerprint>.json` and merged across runs.
- **Proposal.** With `--propose`, every fingerprint yields an additive proposal: a schema fragment (a new `Item`/`OutputBlock`/`InputBlock` variant, or optional fields on an existing one, names taken from the source's own names in snake_case), a projector-arm skeleton for that provider, a fixture line from the sample into `tests/fixtures/ir-v1/pending/`, and a task record `.bboxy/tasks/<date>-ir-extend-<fingerprint>.md` in the brief format, ready for the Oracle to dispatch. A proposal that `ir-compat check` would refuse is a scanner bug.
- **When it runs.** (a) At session close, in-process on that session's log, off the hot path: a new fingerprint gives one Info notice, "N unmapped kinds from <provider>; run ir-scan --propose". (b) In `make test`, over `tests/fixtures/`, as a dry run: a fixture whose unknowns have no pending proposal fails. (c) In the night run over real sessions, with `--propose`.
- **Landing an extension.** The proposal is an ordinary executor task: implement the arm, move the fixture into the frozen corpus as `v1.<n>-<provider>-<kind>.jsonl`, pass `make test-ir`. Because the gate proves additivity, `scripts/land.sh` refreshes the lock itself (`ir-compat lock` after `check` passes); no Architect sits in the loop. The Architect reviews names and meaning (E3) afterwards and can correct only by adding (E7).
- **Bounds.** Samples pass the redactor. One sample per fingerprint. Scan cost stays under 1 ms CPU between awaits ([[budgets]] §1), never on the UI thread, and is bounded by the log's size.

## Change log

- 2026-11-02, at the operator's direction ("log into the IR what we could not parse, mark it unknown, and have an automated scanner extend the IR the moment we meet an unknown"): E4 strengthened (payload-carrying `Unknown`), §7 edge rule, §8 scanner and self-extension pipeline, [[d5978-unknown-is-preserved-and-promoted]].
- 2026-11-02, Lumen Foxglove <ARCHITECT>, on the operator's direction relayed by the Oracle: v1 law and gate; locked from 0.4 at 3c78fcf; corpus = the 5 protocol fixtures + 5 real ACP sessions.
- 2026-11-02, second pass at the operator's request ("rethink your choices"). Found by reading the store: E9 (readers take durability from `seq`; `store.rs:144`/`:215` violate it). 1.0 is now the lock and the additions are 1.1. Usage provenance simplified from a per-field map to one `source` per record plus `unreported`; the report never adds scopes together. §6 names the IR's future home.
