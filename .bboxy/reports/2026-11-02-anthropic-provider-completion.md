---
author: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
authorship: {"kind":"seat","process_generation":"1","public_key":"pnKNvmWPkRl5rD2uVnyFbQX/wwmq3gxVJCQyoeoGEG0=","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
commits: ["3186dc3fb0e743761c9f1d7b2079876cf2a09db3","76e7d511fe3820b8f4d2da827a856ded3a3138d3","dc18b3aed795c6815da43e8b6b99e5e5e41f0d1b"]
created: "2026-11-02T19:28:39.617Z"
created_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
id: "01M6EVRBEXHKXRRE8Y525BYJ1Q"
number: 5378
provenance: {"digest":"sha256:435c34480068b6173cfc191c4f634d756f173c95d3b4d43447cc6ac8e21fed04","frontmatter_digest":"sha256:7ee6c76d60d73125a73cd50516332293f2f2d90c6ffae1391ce6fd995b2d73f6","log":".bboxy/log/2026-11-02.jsonl#35805","sig":"lPkLjlNaXZhj51HeEJFkjE5s8bsL4n7WeOpuncfltFcQYXuMMqaHef94FT2vNJvHFTv9PlumcBMmsOIT3UKkBA==","signer":"seat:01M6ESNVQCH697AK1FE3S9Q52D"}
supersedes: "[[reports/2026-11-02-anthropic-provider-final]]"
task: "[[tasks/2026-11-02-harness-anthropic-provider-v2]]"
tasks: ["[[tasks/2026-11-02-harness-anthropic-provider-v2]]","[[tasks/2026-11-02-harness-anthropic-provider]]"]
title: "Anthropic provider completion evidence and canonical gate receipt"
type: "report"
updated: "2026-11-02T19:28:39.617Z"
updated_by: "seat:01M6ESNVQCH697AK1FE3S9Q52D"
verify: {"command":"cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_shutdown::","exit_code":0}
---
# Anthropic provider handoff

Task #5282 supersedes #5203. Worktree: /Users/feral/mydev/bboxy450-harness-anthropic, branch anthropic-provider. This append-only report supersedes [[reports/2026-11-02-anthropic-provider-final]] (#5334). The full implementation file list, mechanisms, acceptance matrix, abstraction limits and scope-authorization trail remain in [[reports/2026-11-02-anthropic-provider]] (#5318), with the ACP byte-fixture correction in #5334.

## Final source changes and evidence

- 3186dc3fb0e743761c9f1d7b2079876cf2a09db3 — native API-key Anthropic client, strict streamed blocks, signed/redacted replay, stable cache markers, cache creation usage, profile factories, schema, fixtures and shared retry ownership.
- 76e7d511fe3820b8f4d2da827a856ded3a3138d3 — freeze legacy request bytes in their original insertion order under both serde_json map backends. The first canonical attempt on 4de363d exited 2: ACP's preserve_order feature exposed the test expectation's ordering error. The product's legacy request construction was unchanged. target/provider-final-gate-4de363d.log retains that failure; all 17 provider tests then passed under ACP (target/provider-acp-final.log), and the exact byte test passed under default features.
- dc18b3aed795c6815da43e8b6b99e5e5e41f0d1b — repair a pre-existing supervisor test premise, entirely in tests/support/seat_shutdown.rs. The second canonical attempt on df82a59 exited 2 although all provider tests passed: hup_kills_owned_groups_before_supervisor_escalation_and_resume_recovers observed a valid graceful SessionClosed and failed its unconditional abrupt-exit assumption. Evidence: target/provider-final-gate-df82a59.log; .bboxy/interfaces/embed-seat.md:59 explicitly permits trying the graceful path after synchronously killing groups.

The supervisor witness now preserves the original group-ESRCH and escalation assertions. A clean log is checked and retained; a separate real SIGKILL of a pending headless core supplies the mandatory abrupt-recovery probe. The same seat --resume path must still emit SessionRecovered, satisfy check_transcript and close gracefully. A dedicated test always runs the crash probe. No log is truncated, no deadline is relaxed and no recovery/group assertion is skipped. The failing synchronous-kill control would still fail before the fallback runs.

Two developmental PTY versions of the new crash probe exited 101 on their exit guard; the probe now uses a headless process to avoid another PTY/session-leader teardown dependency. It owns and retires all orphan groups before waiting. Product signal/lifecycle code and models are unchanged.

## Actual verification before this report commit

- cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_shutdown:: — exit 0; all 6 supervisor/writer/crash-recovery tests passed (target/provider-seat-shutdown.log).
- cargo clippy --offline --locked --features acp,test-fixtures --test seat_process -- -D warnings — exit 0 (target/provider-seat-clippy.log).
- Provider default/ACP tests, native model/nonvacuity tests, schema/goldens and all-feature clippy preflights — exit 0, with exact commands and log locators in #5318/#5334.
- git verify-commit — exit 0 on every implementation/correction commit, signed by the required Boxy key.
- git diff --check — exit 0.

## Canonical final-HEAD receipt

This report is committed before the required full final-HEAD run, so it does not claim a result that has not happened. Run exactly make test && make test-formal after the report commit and retain target/provider-final-gate.log. task_report is sent only after actual exit 0 and includes final HEAD, the unmerged PR URL and this report locator.

Machine receipts are .bboxy/runs/test-current.json, .bboxy/runs/gui-gate.json, .bboxy/runs/formal-native.json and target/formal/evidence.json, with retained source/model/tool-bound archives under verification/evidence/. They bind the final commit and record native/Java complete graph and verdict parity. No source change follows a successful final run.

All new provider tests use loopback and synthetic keys. No optional live smoke, real key read, release or merge was performed. Images remain unsupported. The proposed provider ICD section 7 awaits the Oracle/Architect review before merge. Source is committed; generated gate receipts may dirty tracked run files and remain available for the Oracle's independent verification.
