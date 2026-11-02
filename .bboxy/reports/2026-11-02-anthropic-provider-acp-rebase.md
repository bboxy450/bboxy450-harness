---
author: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
authorship: {"kind":"seat","process_generation":"1","public_key":"pnKNvmWPkRl5rD2uVnyFbQX/wwmq3gxVJCQyoeoGEG0=","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
commits: ["1855afd66542641e6f4d3566d2a25799eb1549f8"]
created: "2026-11-02T20:55:03.186Z"
created_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
id: "01M6F0PHHZCPF5F3WSEYCKE2GQ"
number: 5941
provenance: {"digest":"sha256:2d4d167b60664ad2afafd7ed71a6841cda02a8b86a8debc82c12f5d2c9ef4ba9","frontmatter_digest":"sha256:af5fa5e0dbffb7454a1c867f2dcfa15f459b7cca68744b16bf9cd6443340db77","log":".bboxy/log/2026-11-02.jsonl#39637","sig":"+10iP3RaXiptBMjsT59Ioyty6LmPA1vNfEvLS/wIewp9ruNIi+XxTPpr2TRs0/LBmiI93t6Rif4La+kQAb15AQ==","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
supersedes: "[[reports/2026-11-02-anthropic-provider-final-gate]]"
task: "[[tasks/2026-11-02-harness-anthropic-provider-v2]]"
tasks: ["[[tasks/2026-11-02-harness-anthropic-provider-v2]]","[[tasks/2026-11-02-harness-anthropic-provider]]"]
title: "Anthropic provider and ACP actor coexist after rebase"
type: "report"
updated: "2026-11-02T20:55:03.186Z"
updated_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
verify: {"command":"make test-formal","exit_code":0}
---
Task #5282 supersedes #5203. All four checks requested in Oracle ruling 01M6F081WNV8YEXMSMSMC8QVB2 passed, once and serially, on source 1855afd66542641e6f4d3566d2a25799eb1549f8. The branch is rebased onto origin/main f8de77ae7dcbb5c064180268e491b9ed52fc72b0. Worktree: /Users/feral/mydev/bboxy450-harness-anthropic. PR: https://github.com/bboxy450/bboxy450-harness/pull/26; no merge performed.

The Oracle's ruling identifies the remaining shutdown witness as a load-dependent baseline flake, records serial A/B results of 0/5 failures on both 0c33b61 and main, and takes responsibility for the canonical full gate. This handoff does not claim a new passing make test. The earlier exit 101 and full-gate exit 2 remain accurately recorded in the superseded report.

## Integration and files

- src/provider/profile.rs:106 preserves ACP profiles' optional model/endpoint and required agent.command. The Anthropic default endpoint applies only in the model-provider branch at :121. Profile::build_provider at :182 still selects Anthropic only for provider: anthropic; other model backends retain OpenAiCompat.
- src/gateway/host.rs:233 and :250 retain upstream ACP-agent detection and the AcpAgentActor path. The model path at :297 uses the shared profile factory. Thus provider: acp-agent selects the ACP actor, provider: anthropic selects the native Messages provider, and other model profiles select OpenAiCompat.
- src/frontends/smoke.rs preserves both agent: None and max_tokens: None. Initial/deferred headless and idle engine construction continue using the same model-provider factory.
- tests/formal/models.rs:2016 retains acp_agent_turn and its broken control; :2097 retains provider_stream and its three broken controls. Both remain registered in corpus(). verification/{contracts,corpus,coverage}.toml retain both families and every source entry. verification/README.md now records 77 models and 28 contracts.
- Only src/provider/profile.rs and tests/formal/models.rs had source conflicts. The remaining overlapping files merged automatically. target/provider-acp-range-diff.log records the rebase delta; all other replayed commits were unchanged. A generated formal-native receipt autostash conflict was restored from this rebase's own stash df43c7f, with no source lost.
- src/backend/acp_agent.rs and the other new ACP backend files are inherited unchanged from main. tests/support/seat_shutdown.rs and scripts/test.py have no diff against main. No shutdown timing, skip or waiver was introduced.

## Requested checks and real exits

The serial driver recorded source/base SHAs, start/end times, individual commands and exit codes in target/provider-acp-checks.json (20:53:02–20:53:38 UTC). It exited 0.

| Command | Exit and evidence |
|---|---|
| cargo clippy --offline --locked --all-targets --all-features -- -D warnings | 0; target/provider-acp-clippy.log |
| cargo test --offline --locked --all-features --test anthropic_provider --test provider_stream | 0; 19 Anthropic and 6 existing provider tests; target/provider-acp-providers.log |
| cargo test --offline --locked --all-features --test acp_agent | 0; all 6 real-Host/fake-agent tests; target/provider-acp-acp-agent.log |
| make test-formal | 0; 77 model variants, 15 nonvacuity witnesses and the reactor projection; target/provider-acp-formal.log |

The ACP witnesses exercise streamed text/reasoning/plan, face permission resolution, unattended denial, cancellation, agent crashes/restarts, malformed lines and transcript laws. The existing Anthropic Host witness at tests/anthropic_provider.rs:805 proves the model selection still reaches the native Messages client.

## Acceptance evidence retained

| Requirement | Files and executable evidence |
|---|---|
| Native API-key-only auth and existing key sources | src/provider/{anthropic,http,profile}.rs; request-shape and real-CLI literal/env/!cmd tests inspect x-api-key/version and reject authorization/beta impersonation headers, with fake keys and redaction assertions. |
| System/tool mapping, choices, max_tokens, endpoints and caching | tests/anthropic_provider.rs:390 and tool_choices_profile_defaults_and_invalid_pairing; base, slash, /v1 and /v1/ all resolve to one /v1/messages. Stable content bytes and tool pairing are asserted; default max_tokens is 4096. |
| SSE, thinking/signature/redacted replay, usage, endings and errors | 23 tests/fixtures/sse/anthropic/*.sse and exact real-client event sequences, parser byte/UTF-8 split witnesses, actual next-round opaque block replay, retries/cancel/stall/HTTP and post-output error assertions. |
| Length/refusal never dispatch partial calls | truncated_or_refused_calls_remain_paired_without_execution checks the actual engine finalizer and transcript laws; the formal good model reaches these endings with pending calls. |
| Backward compatibility | Exact OpenAiCompat request bytes, legacy status/redirect behavior, old Usage replay with additive defaulted cache_creation_input_tokens and protocol.schema.json. All 6 existing provider tests remain green. |
| Stateful contract verification | HarnessProviderStream and three named safety controls coexist with HarnessAcpAgentTurn/BrokenHarnessAcpAgentTurn; coverage includes both implementations and their real API/process witnesses. |
| ICD and provenance | .bboxy/interfaces/provider.md section 7 remains amendment: proposed. All rebased commits and the integration commit have valid GPG signatures. No credentials or real profile keys were read. |

Earlier implementation details and full-gate logs are retained in [[reports/2026-11-02-anthropic-provider-final-gate]] and its linked reports. In particular, target/provider-final-gate-5870fde.log records the earlier canonical exit 2; it is not relabeled as success. Scope amendment authorization 01M6ESQN2Z1997CYKEWZ5XP5YB and superseding task #5282 remain the authority for startup/schema changes.

After committing this report, run the reference parity driver against the unchanged, freshly exported model artifacts and verify its receipt, binding the final report-commit HEAD, source/model/tool hashes and all 77 native/Java graphs and verdicts. The final task_report supplies the actual command exits and archive locator. No second full gate is run here, as the Oracle explicitly owns that check.

The worktree retains generated-only changes in .bboxy/runs/{formal-native,gui-gate,test-current}.json and verification/evidence/. Optional live smoke was not run; images remain unsupported. Finite model parity does not claim arbitrary Rust refinement, kernel scheduling guarantees or full TLC completeness. The unrelated unknown operation 01M6EW3X2E7FPRTXMADHSW0AZV remains undispatched.
