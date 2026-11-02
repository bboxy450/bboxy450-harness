---
author: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
authorship: {"kind":"seat","process_generation":"1","public_key":"pnKNvmWPkRl5rD2uVnyFbQX/wwmq3gxVJCQyoeoGEG0=","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
commits: ["3186dc3fb0e743761c9f1d7b2079876cf2a09db3","76e7d511fe3820b8f4d2da827a856ded3a3138d3"]
created: "2026-11-02T19:18:56.060Z"
created_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
id: "01M6EV6GQNR2YTE3E6P4Y2YK5C"
number: 5334
provenance: {"digest":"sha256:553ff99ba8c0a8319951f404f08fea0a54f7ba01a7c841c925ffbad5b0c7788e","frontmatter_digest":"sha256:5c3318de908d71de5b25732f8c22e1c55ac5074fd8eacba00409cd96e2f06517","log":".bboxy/log/2026-11-02.jsonl#35509","sig":"dSkWAodAlzp4KkTdGix+ALULFIG8Z3H4ph1O/tdp72r2lqfmdyv6k1PHc9f3Cu1JZv1xAOiwuCwld544uPoNAw==","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
supersedes: "[[reports/2026-11-02-anthropic-provider]]"
task: "[[tasks/2026-11-02-harness-anthropic-provider-v2]]"
tasks: ["[[tasks/2026-11-02-harness-anthropic-provider-v2]]","[[tasks/2026-11-02-harness-anthropic-provider]]"]
title: "Anthropic provider final gate handoff after ACP byte-fixture correction"
type: "report"
updated: "2026-11-02T19:18:56.060Z"
updated_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
verify: {"command":"cargo test --offline --locked --features acp,test-fixtures --test anthropic_provider","exit_code":0}
---
# Final gate correction

This append-only report supersedes [[reports/2026-11-02-anthropic-provider]] (#5318) for the final verification handoff. The first canonical attempt on 4de363d7b51b3df874bd09417650977b54d29cd7 exited **2** (make; the underlying cargo test exited 101). The default feature suite passed. The ACP suite exposed an error in the byte-compatibility test expectation: ACP enables serde_json preserve_order, and its expected JSON literal listed optional tools before the legacy base fields. Actual OpenAiCompat bytes still followed the unchanged legacy construction order. Evidence: target/provider-final-gate-4de363d.log, existing_profiles_keep_openai_request_bytes assertion.

Signed correction commit 76e7d511fe3820b8f4d2da827a856ded3a3138d3 freezes the original request literal in legacy insertion order before deserialization. The expectation therefore exercises both serde_json map backends. Product source and wire construction are unchanged by this correction. cargo test --offline --locked --features acp,test-fixtures --test anthropic_provider then returned **0**, all 17 tests passed (target/provider-acp-final.log). The exact byte case also returned **0** under default features. No failed check was waived.

The required full canonical command runs on the new final report commit. Its post-commit receipt, actual exit and PR URL are carried in task_report, and target/provider-final-gate.log retains its output. The failed older run is retained separately. Generated run receipts are not source changes and remain outside the final source-dirty predicate.

---

# Implementation evidence

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
| Signed/redacted replay | tests/anthropic_provider.rs:208/:498/:916 check opaque events, actual signed tool-round replay and exact signed/redacted blocks beside cache markers. |
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
