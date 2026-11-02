---
author: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
authorship: {"kind":"seat","process_generation":"1","public_key":"pnKNvmWPkRl5rD2uVnyFbQX/wwmq3gxVJCQyoeoGEG0=","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
commits: ["3186dc3fb0e743761c9f1d7b2079876cf2a09db3"]
created: "2026-11-02T19:13:49.955Z"
created_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
id: "01M6ETX4H3BSQDEX8RGTM4D2B1"
number: 5318
provenance: {"digest":"sha256:22d4538e2758aae193c014c0439f8d78e941140a30cdc977be83368737aab4a0","frontmatter_digest":"sha256:64ef8c368b2ae1b64a52a9cb87bd8590a336792cf56429abc1c8373f423c183c","log":".bboxy/log/2026-11-02.jsonl#35372","sig":"HvFP8qBgXuIvgcGJ3uuHXMcO/WrpyBxg5Qsy6GCyn5BtWfjv2WSwtrUbh93fJZWXDzc++pVPqHPPE1UFlYggAg==","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
task: "[[tasks/2026-11-02-harness-anthropic-provider-v2]]"
tasks: ["[[tasks/2026-11-02-harness-anthropic-provider-v2]]","[[tasks/2026-11-02-harness-anthropic-provider]]"]
title: "Anthropic Messages provider handoff"
type: "report"
updated: "2026-11-02T19:13:49.955Z"
updated_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
verify: {"command":"cargo clippy --offline --locked --all-targets --all-features -- -D warnings","exit_code":0}
---
# Anthropic Messages provider

Implemented #5282, superseding #5203, in /Users/feral/mydev/bboxy450-harness-anthropic, branch anthropic-provider. Signed implementation commit: 3186dc3fb0e743761c9f1d7b2079876cf2a09db3. git verify-commit returned 0 with the required Boxy key. No real profile keys were read.

## Files and mechanisms

- src/provider/anthropic.rs:20 — API-key Messages client; base or /v1 endpoints normalize to one /v1/messages. Request max_tokens overrides profile value, then 4096. System is hoisted; tool schemas, choices, calls and paired results use native blocks. Cache markers stay on the last tool/system block and final eligible stable content block. :192 replays native opaque content in original order, preserving signatures and redacted data while checking tool pairing.
- src/provider/anthropic_parser.rs:31 — incremental UTF-8/SSE framing, owned content blocks, strict finish validation, one terminal, opaque snapshots and safe error classes. :364 merges cumulative usage and retains cache read/creation counts.
- src/provider/http.rs:43 — shared HTTP budgets, exponential backoff, Retry-After, read stalls, cancellation and retry-before-output. Format-specific authentication; raw bodies and reqwest diagnostics are never formatted. src/provider/mod.rs retains legacy request bytes.
- src/provider/profile.rs:149, src/frontends/headless.rs, src/gateway/host.rs, src/engine/mod.rs — one factory selects the backend for initial/deferred/Host/idle profile construction. Key resolution is reused. src/frontends/smoke.rs initializes the additive Profile field.
- src/protocol/mod.rs:408, protocol.schema.json, src/provider/parser.rs — optional cache_creation_input_tokens defaults to None and is omitted when absent, preserving old usage/log bytes.
- tests/fixtures/sse/anthropic/ — 21 offline fixtures; tests/support/sse_server.rs captures full requests and honors explicit fragment boundaries.
- tests/anthropic_provider.rs — 17 real-loopback HTTP, profile/process, request-byte, transcript, retry/cancel and replay tests. The parser unit witness feeds every single byte and every two-chunk split through UTF-8 and signed thinking.
- tests/formal/models.rs:1963, tests/formal_nonvacuity.rs:24, verification/{contracts,corpus,coverage}.toml and verification/README.md — shared provider lifecycle/finish model, three broken controls, source inventory and abstraction limits.
- .bboxy/interfaces/provider.md:112 — proposed section 7 covering mapping, errors, cache, usage, reasoning replay and fixtures. Frontmatter amendment remains proposed for Oracle/Architect review before merge.

Clippy exposed two pre-existing warnings in the scoped headless file: a nested if and a function after a test module. The condition was collapsed and the unchanged function moved before the test module.

## Acceptance evidence

| Requirement | Executable witness / source |
|---|---|
| Selection; key-only headers; no impersonation | tests/anthropic_provider.rs:384 checks x-api-key/version and absence of Authorization/beta/client-id; :498/:609/:799 exercise initial literal/env/command, idle switch and Host factory. |
| Base and /v1 endpoints | tests/anthropic_provider.rs:759 checks base, slash, /v1 and /v1/ all POST to exactly /v1/messages. |
| System/tool/choice/max-token mapping and pairing | tests/anthropic_provider.rs:384/:454 inspect requests, a stable boundary inside merged tool results, all choices and request/profile/default token values. Orphan pairing fails before HTTP. |
| Required SSE fixtures and stop/error classes | tests/anthropic_provider.rs:99 compares exact events; :234/:261/:309/:663/:711 cover Retry-After, disconnect/stall, cancel while reading/backing off, malformed frames, HTTP/SSE errors and no second terminal. |
| Signed/redacted replay | tests/anthropic_provider.rs:208/:498/:913 check opaque events, actual signed tool-round replay and exact signed/redacted blocks beside cache markers. |
| Usage and old logs | tests/anthropic_provider.rs:99/:898 check read/creation counts and old Usage serialization; protocol_golden regenerated the schema and checks transcript laws. |
| Stable bytes and OpenAI compatibility | tests/anthropic_provider.rs:384 compares serialized stable blocks across changed suffixes; :868 compares exact legacy request bytes and headers. |
| Secrets | tests/anthropic_provider.rs:498 checks stdout, stderr, logs and next-round bodies using a synthetic key. Error fixtures check fixed safe diagnostics. Existing secrets gate remains in make test. |
| Formal and broken controls | HarnessProviderStream has 242 native states, no violation; retry/finish/ending controls have 278/242/423 states and violate RetryOnlyBeforeOutput / StopAgreesWithCalls / OneTerminal. tests/formal_nonvacuity.rs:24 checks named violations and reachable two-call/backoff states. |
| Signed final gate and unmerged PR | The final report commit is followed by the canonical command and PR creation. Receipts and task_report bind the final HEAD, actual canonical exit and PR URL. |

## Commands and actual results at report creation

- UPDATE_GOLDENS=1 cargo test --offline --locked --test protocol_golden schema_and_golden_transcripts — exit 0; schema regenerated, golden transcripts and law checks passed.
- cargo test --offline --locked --test anthropic_provider --test provider_stream --test formal_nonvacuity — exit 0; 16 Anthropic, 6 existing HTTP and 15 formal nonvacuity tests at that development point (target/provider-focused.log).
- cargo test --offline --locked --test anthropic_provider --test formal_models — exit 0; final 17 Anthropic and 2 model tests (target/provider-targeted-final.log).
- cargo test --offline --locked --test anthropic_provider --test provider_stream --test formal_nonvacuity --lib provider:: — exit 0; the unit parser witness ran. Integration tests were filtered by provider:: and are not counted as integration evidence for this invocation.
- cargo clippy --offline --locked --all-targets --all-features -- -D warnings — exit 0 (target/provider-clippy.log). Earlier exits 101 identified the headless warnings and a DoubleEndedIterator::last test lint; all were fixed.
- Source-header, KB-frontmatter and inventory-shape scripts, git diff --check — exit 0.
- Earlier compile, before v2 delegation — exit 101 on the out-of-scope Profile literal and Usage field. They remained untouched until the fresh task widened scope. Trail: blocker 01M6ESQN2Z1997CYKEWZ5XP5YB, signed decision 01M6EST427KTGPEV9E9T1N4QYN, superseding blocker 01M6ETF48VZWZE1B75SG812X0G, fresh task 01M6ETFFVENY5RAX4DGAC4BW50 (#5282). The signed DM alone was not used to bypass scope.

## Final gate receipt

This report is committed before the required single full final-HEAD run. At document creation that canonical run is not yet claimed. After the report commit, run exactly make test && make test-formal, retaining target/provider-final-gate.log. Report success only after its actual exit 0.

Canonical receipts: .bboxy/runs/test-current.json, .bboxy/runs/gui-gate.json, .bboxy/runs/formal-native.json and target/formal/evidence.json, with retained archives under verification/evidence/. They bind the final HEAD, source/model hashes and native/Java graph/verdict parity. task_report supplies the final actual exit and PR URL. No source change follows a successful final run.

## Limits and state

No optional live smoke was added or run. Tests use loopback and synthetic keys. Images are not mapped; images=false is advertised. The formal abstraction covers two attempts, two calls and terminal classes; bytes, arithmetic, signatures, real timers, kernel IO and secret matching remain implementation witnesses. Finite native/Java parity does not establish arbitrary Rust refinement or full TLC completeness.

The ICD amendment remains proposed. Generated gate receipts may dirty tracked run files after the final commit; source is committed. No merge, release or publication is authorized by this handoff.
