---
id: 01M6CK65TF1C4S46TDWY98RHQ4
number: 4960
type: interface
title: "ICD: log-converter IR — parsers ↔ IR ↔ lowerers (v0.1, proposed)"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[log-converter]]", rel: part-of }
  - { to: "[[d4949-log-converter-from-scratch]]", rel: constrained-by }
  - { to: "[[d4947-demo-bundle-sanitized-claude-logs]]", rel: constrained-by }
  - { to: "[[protocol]]", rel: fed-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: log-converter IR, v0.1

**Status: proposed.** This is the common intermediate representation of `bboxy450-log-converter` ([[log-converter]]). Parsers produce it, and lowerers, the binary log ([[lc-bxl]]), the watcher ([[lc-watch-sink]]) and the session switch ([[session-switch]]) consume it. The sources are [[log-converter-sota]] (#4951, ideas only, under d4949) and [[native-session-formats]] (#4953, field names on this machine: Claude Code 2.1.304, codex-cli 0.164.0).

| | |
|---|---|
| Side A | parsers `parse::{claude, codex, bboxy}`, plus the sanitizer pass `sanitize` |
| Side B | lowerers `lower::{claude, codex, bboxy}`, ATIF export, `bxl` writer, watcher sinks |
| Provider names | `claude`, `codex`, `bboxy`. **"Bboxy" is the user-facing name of our harness** in the CLI, docs and loss reports. `bboxy450-harness` stays only as the binary and crate name |

## 1. Shape

The IR is aligned to ATIF v1.8 steps and tool calls, so ATIF export is a projection. It extends ATIF with typed content blocks, sealed reasoning, compaction, a DAG, context changes, lineage, per-event fidelity and the raw record.

```rust
pub struct Session {
    pub id: SessionId,                  // IR-owned ULID; never a native id
    pub source: Source,                 // where it was parsed from
    pub lineage: Vec<Lineage>,          // how it relates to other sessions
    pub cwd: Option<PathBuf>, pub git: Option<Git>,   // Git { branch, commit, repo_url }
    pub created_at: Option<Timestamp>,
}
pub struct Source { pub provider: Provider, pub native_id: String, pub cli_version: Option<String>,
                    pub parser_revision: u32 }               // bumped on any mapping change
pub enum Provider { Claude, Codex, Bboxy, #[serde(other)] Unknown }
pub struct Lineage { pub relation: Relation, pub session: SessionId, pub native: Option<(Provider, String)>,
                     pub at: Option<EventId> }
pub enum Relation { ForkOf, ResumeOf, SwitchedFrom, SubagentOf, CompactedFrom, ImportedFrom, #[serde(other)] Unknown }

pub struct Event {
    pub seq: u64,                       // dense from 0 within the session; assigned by the parser
    pub id: EventId,                    // deterministic; see §2
    pub parent: Option<EventId>,        // DAG edge: Claude parentUuid; the previous event elsewhere
    pub turn: Option<TurnId>,           // None outside a turn (session setup, settings)
    pub at: Option<Timestamp>,          // RFC 3339 with ms, as recorded natively; never invented
    pub fidelity: Fidelity,
    pub raw: Option<Raw>,               // the native record, verbatim bytes (§3)
    pub kind: EventKind,
}
pub enum Fidelity {
    Full,         // every native field is represented in `kind`
    Partial,      // some native fields are only in `raw`
    Inferred,     // synthesized by the parser (e.g. Claude TurnStart); no native record of its own
    SummaryOnly,  // the content was replaced by a summary (compaction)
    Opaque,       // meaning not parsed: sealed reasoning, Native records
}
pub enum EventKind {
    TurnStart  { origin: TurnOrigin },                       // User | Resume | System | Unknown
    TurnEnd    { outcome: TurnOutcome },                     // Completed | Interrupted | Failed | Unknown
    Message    { role: Role, blocks: Vec<Block> },          // Role: User | Assistant | System | Developer
    ToolCall   { call_id: String, name: String, namespace: Option<String>,
                 tool_kind: ToolKind, input: ToolInput },    // ToolKind = protocol §4 ToolKind
    ToolResult { call_id: String, is_error: bool, blocks: Vec<Block> },
    Reasoning  { text: Option<String>, summary: Option<String>, sealed: Option<Sealed> },
    Compaction { summary: Option<Vec<Block>>, replaced_through: Option<EventId>,
                 replacement: Vec<EventId>, sealed: Option<Sealed> },
    Context    { changes: BTreeMap<String, Value> },        // Codex turn_context, Claude cwd/permissionMode/model
    Usage      { input: u64, output: u64, cache_read: u64, cache_write: u64, reasoning: u64 },
    Notice     { level: Level, text: String },
    Subagent   { session: SessionId, relation: Relation },
    Native     { provider: Provider, record_type: String },  // retained native record with no IR meaning
    #[serde(other)] Unknown,
}
pub enum ToolInput { Json(Value), Text(String) }             // Codex function_call `arguments` is a JSON string → Json
pub enum Block { Text { text: String }, Image { mime: String, data: BlobRef }, Data { mime: String, data: BlobRef },
                 #[serde(other)] Unknown }
pub struct BlobRef { pub blake3: [u8; 32], pub len: u64 }   // body in the bxl blob table (lc-bxl §2); may be absent
pub struct Sealed { pub provider: Provider, pub form: SealedForm, pub bytes: Vec<u8> }
pub enum SealedForm { ThinkingSignature, RedactedThinking, EncryptedReasoning, EncryptedCompaction, #[serde(other)] Unknown }
pub struct Raw { pub provider: Provider, pub bytes: Vec<u8> /* one native line, no trailing newline */ }
```

## 2. Identity, order and turns

- **EventId** = `blake3(provider ‖ native_id ‖ native_key)[..16]`. `native_key` is the record's own id where one exists (Claude `uuid`, Codex `call_id`/`rs_…`/`ordinal`, Bboxy `seq`). Otherwise it is the native line ordinal. An Inferred event uses its anchor's key plus a role tag (`#turn-start`). The same input therefore always gives the same ids. This is the watcher's idempotency key ([[lc-watch-sink]] §3).
- **Order.** `seq` is native file order. `parent` carries the Claude DAG: streamed blocks chain by `parentUuid`, a sidechain is a branch, and `last-prompt.leafUuid` names the live leaf. For Codex and Bboxy, `parent` is the previous event.
- **Turns.**
  - Codex: explicit, from `event_msg` `task_started` and `task_complete`/`turn_aborted`.
  - Bboxy: explicit, from `TurnStarted`/`TurnEnded`.
  - Claude: inferred. A `user` line whose content is a string or text blocks, and that is not `isMeta`, `isCompactSummary` or a `tool_result`, starts a turn. The turn ends at the last assistant or tool event before the next turn start, or at a `system` `turn_duration`.
- **Turn boundary** (the switch point, [[session-switch]] §2): the last event is a TurnEnd with outcome Completed or Interrupted, and no ToolCall in the turn lacks a ToolResult.

## 3. Raw retention and same-provider lowering

- Every parsed native record is kept as `raw`, unless the run says `--no-raw`. Only then is `raw` dropped, and every such event is downgraded to Partial.
- **Same-provider lowering emits retained `raw` bytes verbatim**, in `seq` order. Events without `raw` (Inferred, or produced by another provider) go through the cross-provider path (§5).
- Consequences:
  - Claude thinking signatures and Codex encrypted reasoning and compaction survive a same-provider round trip byte for byte.
  - A→B→A keeps all of A's native events exactly ([[session-switch]] §4).
- **Native** records keep provider-internal lines that have no IR meaning: Claude `attachment`, `api-request*`, `queue-operation`, `file-history-*`, `cost-state`, titles and modes; Codex `world_state`, `token_usage_record`, `inter_agent_communication_metadata`. They are lowered only to their own provider. Cross-provider, they are dropped and counted (§5).

## 4. Mapping (parsers)

| Native | IR |
|---|---|
| Claude `user` with string or text blocks | TurnStart (Inferred) + Message{User} |
| Claude `assistant` `text` / `thinking{thinking, signature}` / `redacted_thinking` / `tool_use{id, name, input}` | Message{Assistant} / Reasoning{text, sealed: ThinkingSignature} / Reasoning{sealed: RedactedThinking} / ToolCall{call_id: id, input: Json} |
| Claude `user` with `tool_result{tool_use_id, content, is_error}` | ToolResult |
| Claude `system` `compact_boundary`, then the `user` line with `isCompactSummary` | Compaction{summary, replaced_through: logicalParentUuid's event} |
| Claude `isSidechain` chain / Task subagent | Subagent + lineage SubagentOf |
| Claude `assistant.message.usage` | Usage |
| Codex `session_meta` | Session (source, cwd, git); `base_instructions` → Message{System} (Partial) |
| Codex `response_item` `message{role, content[input_text, output_text, input_image]}` | Message (Developer stays Developer) |
| Codex `reasoning{summary[], encrypted_content}` | Reasoning{summary, sealed: EncryptedReasoning} |
| Codex `function_call{name, namespace, arguments}` / `custom_tool_call{name, input}` (incl. code-mode `exec`) | ToolCall{input: Json / Text} |
| Codex `*_call_output` | ToolResult |
| Codex `compacted{replacement_history, compaction.encrypted_content}` | Compaction{replacement: parsed messages, sealed: EncryptedCompaction}, Fidelity Opaque for the sealed part |
| Codex `turn_context` | Context |
| Codex `event_msg` `token_count` | Usage |
| Bboxy Envelope items (protocol v0.4 §4) | 1:1: UserMessage/AgentMessage → Message; Reasoning → Reasoning (`provider_opaque` → `sealed` with its provider); ToolCall → ToolCall + ToolResult; Compaction/ContextTrim → Compaction; Usage, Notice, Subagent 1:1; FileChange/TurnDiff/Terminal/Plan/Approval/Image → Native{Bboxy} with Full fidelity for the Bboxy target, and lowered cross-provider as text (§5) |
| anything unrecognised | Native{provider, record_type} (Opaque); a line that is not JSON → Notice{Warn}, plus a parse-error count. Never a panic, never a silent drop |

E3's #4935 `import::claude` (harness) is the first implementation of the Claude rows and of the sanitizer. Its code moves into this crate in phase 3 ([[log-converter]] §7).

## 5. Cross-provider lowering and the loss table

`lower::<Q>(session) -> (native lines, LossReport)`. `LossReport` counts every input event as exactly one of `kept`, `transformed` or `dropped{reason}`, so `kept + transformed + dropped = events`. Nothing is dropped unless a row below drops it.

| Direction | Dropped (counted) | Transformed | Kept |
|---|---|---|---|
| Claude → Codex | Claude Native records; Usage; reasoning per the table below | ToolCall/ToolResult → `function_call`/`function_call_output`, `call_id` kept, `arguments` = JSON string; Claude tool names without a Codex equivalent → `function_call` with the original name and namespace `claude`; Compaction → a Developer message with the summary | Message text, tool I/O |
| Codex → Claude | `sealed` EncryptedCompaction; Codex Native records; reasoning per the table below | `exec`/custom tools → `tool_use` with input `{"code": …}` and the original name; Developer → Claude `user` with `isMeta`; Compaction → `compact_boundary` + `isCompactSummary` user line from `replacement` | Message text, tool outputs as `tool_result` text |
| Claude/Codex → Bboxy | foreign Native records (counted) | reasoning per the table below; Bash → Terminal, Edit/Write/MultiEdit/apply_patch → FileChange, TodoWrite/plan → Plan | all text, tool I/O, images |
| Bboxy → Claude/Codex | Usage; reasoning per the table below | FileChange/TurnDiff → ToolCall + ToolResult text with the unified diff; Terminal → a Bash/`exec` call with output; Plan → text; Approval → Notice text; Image → `input_image` (Codex) or image block (Claude) when the blob is present, else text | Message text, tool I/O |
| same provider | nothing (`raw` verbatim) | — | everything |

**Reasoning.** This follows the operator ruling relayed by the Oracle in message 01M6CKBFRBGGVBDT05CRCV6H35, and it is binding.
- **The IR always keeps reasoning** whenever the source exposes it: the readable text in `text`/`summary`, and the opaque part in `sealed`, tagged with its provider (Claude `signature` or `redacted_thinking`, Codex `encrypted_content`). Neither `--no-raw` nor `sanitize` removes `sealed` or the reasoning text. Sanitizing applies to the text as it does to any other text. `sealed` is opaque bytes and is kept as is.
- **The sealed part is replayed only when `sealed.provider == target`**, verbatim. In every other case it is dropped and counted as `dropped{sealed_reasoning_foreign}`.

| Direction | Readable reasoning text | Sealed part |
|---|---|---|
| Claude → Claude, Codex → Codex | verbatim from `raw` | replayed verbatim |
| Claude → Codex | carried as a `reasoning` item with `summary[{summary_text}]` and **no** `encrypted_content`, **only if** P0/P6 shows Codex accepts that on resume. Otherwise dropped and counted as `dropped{reasoning_text_not_carriable}`. It is never folded into assistant or user text, because the model would then read it as something it said aloud | dropped, counted |
| Codex → Claude | dropped and counted as `dropped{reasoning_text_not_carriable}`: Claude rejects a `thinking` block without a valid signature (`invalid_thinking_signature`, #4953), and an unsigned block cannot be made legal. It is never folded into text | dropped, counted |
| Claude/Codex → Bboxy | kept as `Reasoning{summary: text, raw: text}`, rendered and available to Bboxy on any model, including OpenRouter models (real reasoning examples from Claude) | kept as `Reasoning.provider_opaque = {provider, form, bytes}`; never sent to a model of another provider |
| Bboxy → Claude/Codex | as the two cross-provider rows above, by target | replayed only if `provider_opaque.provider == target`; otherwise dropped, counted |

**Cross-provider constraints for the target to accept the result** (from #4953, to be proven in phase 0):
- Claude requires `tool_use_id` matching the pattern, no duplicate ids, no unmatched `tool_result` and no empty text block. Lowering rewrites foreign call ids into the Claude pattern, deterministically from `call_id`.
- Codex requires `session_meta` as the first line.

## 6. Sanitizer (d4947)

`sanitize(Session) -> Session` is an IR pass, so it covers all three providers. It applies the d4947 rules to `kind` and to `raw`, in place: secret shapes and the seat capability, `/Users/<name>` → `~`, emails/hosts/IPs → placeholders, UUIDs/ULIDs/session ids → stable fakes, image payloads → metadata, and tool output > 4 KiB → truncated. A rewritten raw record is re-serialized and keeps Full fidelity only when its JSON still parses. Otherwise raw is dropped and the event becomes Partial. The pass is deterministic.

## 7. Versioning

- The IR is versioned with `bxl` ([[lc-bxl]] §5). The CBOR encoding uses integer keys per field (the table lives in `src/ir/keys.rs` and is golden-tested), unknown keys are preserved, and every enum ends in `Unknown`.
- `parser_revision` bumps on any mapping change. A consumer may re-parse from `raw` when it sees an older revision.

## 8. Verified by

| Clause | Evidence |
|---|---|
| §3 same-provider identity | `tests/roundtrip.rs`: for each provider's synthetic fixture, `lower::P(parse::P(x))` is byte-identical to `x` |
| IR fixpoint (casr-style) | `parse::P(lower::P(ir)) == ir`, ignoring `raw` and fidelity, for IR built from each other provider |
| §5 no silent loss | for every direction, `kept + transformed + dropped == events`, and each `dropped` reason is one row of the table. Broken control: a lowerer that skips an event without counting it fails |
| §5 target acceptance | phase 0 and phase 6: the lowered session resumes in the real CLI against a mock endpoint ([[log-converter]] §7 P0) |
| §2 determinism | two runs on the same input give byte-identical `bxl` |
| §4 robustness | malformed lines, a truncated last line and unknown types give Notice/Native, never a panic. Broken control: an unknown type that panics fails the test |
| §5 reasoning | for each direction, a fixture with readable and sealed reasoning gives exactly the table's outcome. Bboxy targets keep both; cross-provider targets count `sealed_reasoning_foreign`. Broken control: a lowerer that folds reasoning into assistant text fails |
| §6 sanitizer | planted secrets in all three fixtures; the output contains none, and `checks.py secrets` passes. Broken control: skipping the pass fails |

Owner: Nyx Copperwing <ARCHITECT>. Implementation: Executors under the phases of [[log-converter]].
