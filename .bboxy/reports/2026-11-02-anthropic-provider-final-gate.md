---
author: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
authorship: {"kind":"seat","process_generation":"1","public_key":"pnKNvmWPkRl5rD2uVnyFbQX/wwmq3gxVJCQyoeoGEG0=","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
commits: ["eedbe44aa9a92e3ab597a4a1b7f077628333889a"]
created: "2026-11-02T20:37:06.728Z"
created_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
id: "01M6EZNP26Z6PH8ABM3YNYCB8A"
number: 5841
provenance: {"digest":"sha256:7a86a2f00df63150999534e3768c58932b8d6d700ef2916012e732fbd43e6d2e","frontmatter_digest":"sha256:5692a98ddad594dffdb17f026ea8127c3c97c607ea0da01fc07de5b30f72a21b","log":".bboxy/log/2026-11-02.jsonl#39006","sig":"bWWxvZT6YMZeHmY/v9BroL53j4VciIbS4WBg3lZI2qMzJjjbUUI4U7Sn0Kigx37EKDbQn0ZXc2bG1kb0W0Q4BQ==","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
status: "blocked"
supersedes: "[[reports/2026-11-02-anthropic-provider-completion]]"
task: "[[tasks/2026-11-02-harness-anthropic-provider-v2]]"
tasks: ["[[tasks/2026-11-02-harness-anthropic-provider-v2]]","[[tasks/2026-11-02-harness-anthropic-provider]]"]
title: "Anthropic provider handoff — upstream shutdown verification remains blocked"
type: "report"
updated: "2026-11-02T20:37:06.728Z"
updated_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
verify: {"command":"cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_shutdown:: -- --test-threads=1","exit_code":101}
---
# Anthropic provider handoff

Task #5282 supersedes #5203. Source head: eedbe44aa9a92e3ab597a4a1b7f077628333889a, rebased onto origin/main d6e504d69265465145cce208fcb695cc3bcb73ff. Worktree: /Users/feral/mydev/bboxy450-harness-anthropic. Draft PR: https://github.com/bboxy450/bboxy450-harness/pull/26. No merge was performed.

**Implementation is ready for review; required verification remains blocked by an upstream seat-shutdown failure. No green canonical gate is claimed.** This report supersedes #5378. Earlier implementation and development evidence is retained in [[reports/2026-11-02-anthropic-provider]], [[reports/2026-11-02-anthropic-provider-final]] and [[reports/2026-11-02-anthropic-provider-completion]]; their commit references precede rebases.

## Files and mechanisms

- src/provider/anthropic.rs — native API-key Messages client, base-/v1 endpoint normalization, system hoisting, native tool pairing and choices, request/profile/default (4096) max_tokens, three stable cache breakpoints and exact native reasoning replay.
- src/provider/anthropic_parser.rs — incremental SSE/UTF-8 and block assembly; thinking/signatures/redacted data; cumulative cache read/creation usage; strict stop/error mapping and one terminal event. Only tool_use makes calls ready. Length/refusal retire incomplete tool input without dispatch.
- src/provider/http.rs and src/provider/mod.rs — shared cancellable retry/read-stall/budget handling. Anthropic uses x-api-key and anthropic-version, refuses redirects and retries all 5xx. Legacy request bytes, legacy retryable statuses and its redirect policy are preserved and checked.
- src/provider/profile.rs, src/frontends/headless.rs, src/gateway/host.rs, src/engine/mod.rs — shared profile factory for initial/deferred, Host and idle profile changes, reusing literal/environment/!cmd key resolution. src/frontends/smoke.rs initializes the additive profile field.
- src/protocol/mod.rs, protocol.schema.json and src/provider/parser.rs — optional cache_creation_input_tokens, defaulting to None and omitted when absent so old usage/log bytes replay unchanged.
- tests/anthropic_provider.rs, 23 tests/fixtures/sse/anthropic/*.sse, tests/support/sse_server.rs — 19 real-client HTTP/process/shape/replay/secrecy/compatibility tests, exact events and explicit UTF-8 boundaries; test server captures complete requests and supplies redirects.
- tests/formal/models.rs, tests/formal_nonvacuity.rs, verification/{contracts,corpus,coverage}.toml and verification/README.md — shared provider model, three actual broken controls, source inventory, assumptions and implementation-only boundaries.
- .bboxy/interfaces/provider.md section 7 — amendment remains proposed for Oracle/Architect review.

No product process/MCP implementation was changed. tests/support/seat_shutdown.rs and scripts/test.py are byte-identical to the current upstream versions. Experimental local changes to these files were restored per the delegator. When rebase skipped the already-merged original witness fix, a following historical revert would have removed it; commit eedbe44 restores the accepted upstream file, leaving no net witness/script diff.

## Acceptance evidence

| Requirement | Witness |
|---|---|
| Key-only auth, no impersonation; hoisting, pairing, cache markers and stable bytes | tests/anthropic_provider.rs:390 inspects headers and request shape; :504 uses literal/env/command sources through the real binary and checks redaction. |
| Endpoint, tool choice and max_tokens | :460 and :765 inspect all choices, request/profile/default values, and base/slash/v1/v1-slash endpoints. |
| SSE, stop/error classes, retries/cancel and exact terminal behavior | :99, :240, :267, :315, :669 and :717 run raw fixtures through the real HTTP client. |
| Signed/redacted replay | :214, :504 and :922 compare opaque snapshots and actual next-round native content. |
| Truncated/refused tools remain paired and never execute | :966 witnesses the engine finalizer and transcript laws; the model reaches length/refusal with pending calls. |
| Existing OpenAiCompat behavior | :874 checks exact request bytes; :1015 checks legacy 501/529 classes and redirect behavior, plus Anthropic's redirect refusal. |
| Usage/schema/backward replay | old_usage_replays_and_cache_creation_is_additive, exact usage fixture assertions, regenerated schema/golden transcripts. |
| Formal | HarnessProviderStream: 242 states/453 edges; retry, finish and ending controls: 278/561, 242/453 and 423/634, with their named safety failures. Pre-rebase current-provider parity passed all 75 models (27 contracts). Final report-commit source bindings are refreshed separately below. |

## Final requested delta checks: actual exits

The delegator prescribed one serial delta run after #5702 merged, not another full gate (decisions 01M6EXY6N851EKCVKPQTPAEM12 and 01M6EYWM5XZ67XJF900NJV6PND). The run admitted with zero other scripts/test.py processes, recorded in target/provider-final-delta.json:

1. cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_shutdown:: -- --test-threads=1 — **101**. Five tests passed; the HUP case failed. No retry was made.
2. cargo test --offline --locked --features acp,test-fixtures --test anthropic_provider --test provider_stream — **0**, 19 new provider tests and 6 existing provider tests.
3. cargo clippy --offline --locked --all-targets --all-features -- -D warnings — **0**.

The identifiable remaining failure is:

    seat_shutdown::hup_kills_owned_groups_before_supervisor_escalation_and_resume_recovers
    tests/support/seat_shutdown.rs:223
    owned group survived supervisor escalation: [50839, 50835]

Logs: target/provider-final-seat.log, target/provider-final-providers.log, target/provider-final-clippy.log. The driver exits 1 because the first command failed. This is the intermittent baseline class warned about by the delegator, and is reported rather than silently retried or waived.

## Earlier full-gate and baseline evidence

- make test && make test-formal on 5870fde7792f3073e649f0f95405a985e518543c — **2**, only seat_shutdown cases failed after the default suite and earlier feature suites passed. target/provider-final-gate-5870fde.log and target/provider-gate-admission.log retain the full output and zero-other-full-gates admission.
- Exact origin/main b503f35acc40eccf42c07c5633970c81b0f744f6, extracted by git archive into target/provider-baseline-b503f35 without touching the main worktree — the unmodified seat-process target exits **101**, TERM case observes graceful exit 0 then fails “fixture must exercise abrupt recovery” at :233. target/provider-main-shutdown.log and the snapshot's BASELINE_REVISION retain this proof. The separate baseline task #5702 adopted the graceful-exit fixture correction.
- Earlier full attempts and their actual failures are retained in target/provider-final-gate-{4de363d,df82a59,0583953,91d4053}.log. No failed invocation is presented as success.
- make test-formal-parity on e55fbda8aa5142dd7560e2ca31d05a7a9e749dab — **0**, all 75 native/Java complete state/edge graphs and verdicts matched; evidence archive b60e51bb5b8737e5edae7b5964f8e48c7850a1adc0436eab05c2a481bb7984f8.
- Independent prior GUI preflight exited **0**, including headless Metal smoke, before the latest toolkit rebase. That is historical evidence, not a claim that this final source passed a new full GUI gate.

## Provenance, gaps and disposition

All delivered branch commits are signed by Boxy <bboxy450@gmail.com>; git log reports GPG status G. Rebase had no product-code conflict on the final update. A generated formal-native receipt autostash conflicted; its pre-rebase bytes were retained under target/provider-pre-rebase-receipts and restored, without discarding source.

The earlier report operation 01M6EW3X2E7FPRTXMADHSW0AZV remains unknown (workspace_pending, then missing ledger/retry_allowed=false). It was not redispatched. The delegator recorded the host defect and explicitly authorized this distinct final-gate report handoff in decision 01M6EW7V7S8NH9C4NE9PPTC379. This record covers the subsequent rebase and actual new delta results.

After this metadata-only report commit, refresh make test-formal-parity to bind final HEAD/source/model/tool hashes, retaining target/provider-final-formal.log and verification/evidence/. task_report carries its actual result and the final signed report commit. The failing seat-shutdown result remains **101** regardless of formal success.

The PR stays draft pending disposition of the remaining upstream group-retirement failure. Optional live smoke was not added/run, no real profile key was read, and images remain unsupported. Finite model parity does not prove arbitrary Rust refinement, kernel scheduling or full TLC completeness. Generated receipt files remain dirty and retained for independent review.
