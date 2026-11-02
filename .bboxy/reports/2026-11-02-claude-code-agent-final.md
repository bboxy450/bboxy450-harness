---
id: "01M6EWPR3K525RHW9P48RNR4MX"
number: 5175
type: report
title: "Claude Code final verification disposition"
task: "tasks/2026-11-02-harness-claude-code-agent.md"
supersedes: "reports/2026-11-02-claude-code-agent-controls"
commits:
  - "4105eef505c93e6796ae8119949db4d6dbfacac4"
  - "1a7f0af11b850e90db8f89d09c56941b1cf4afe7"
verify:
  command: "make test && make test-formal"
  exit_code: 0
author: "seat:01M6ESNX4AS5VZV13S8K6VA3RT"
created: "2026-11-02"
---
# Final verification disposition

The implementation, all acceptance evidence, strengthened process controls and
remaining limits are in [[reports/2026-11-02-claude-code-agent-controls]], with the
full changed-file mechanisms in [[reports/2026-11-02-claude-code-agent]]. Those
records are retained unchanged. This final append-only record supplies the
verification failure disposition and supersedes their acceptance run locator.

The gate on 596ffb1ac26c9d9e8834e7bab0f50c727ca34f0e exited 2 during the GUI-smoke
capability negative control. target/capability-control/gui-smoke-red.log records
`ld: write() failed, errno=28 (No space left on device)` and no executed test.
The control gate correctly rejected that linker failure as evidence; it was not
counted as a safety counterexample. Captured gate log:
/tmp/claude-code-enospc-gate.log. The earlier ae60f5d candidate's complete gate
had exited 0, before strengthening the late producer and API-model validation.

Source was restored by the control's finally block; only the three generated
.bboxy/runs JSON receipts were dirty. Inspection after the failure reported
15 GiB free on the volume, target/debug/deps 7.2 GiB, incremental 0 B, and the
formal reference tool cache 232 MiB. No other worktree was read or cleaned to
recover space. The final acceptance run limits Cargo build concurrency to two
jobs, reducing simultaneous linker output while retaining every required test.
No product source or test condition was weakened.

The required command is run once to completion on this final report commit:
`CARGO_BUILD_JOBS=2` in the environment, `make test && make test-formal`.
Its actual combined exit status and exact final commit are recorded by the
host task_report after completion, with /tmp/claude-code-final-gate.log and
.bboxy/runs/test-current.json, gui-gate.json and formal-native.json as the command
receipts. The reference proof is target/formal/evidence.json and its retained
verification/evidence/<digest> archive, checked with make check-formal-evidence.
Earlier candidate receipts are not substitutes for final-HEAD verification.

The handoff has signed commits and a PR to main, without merging. The proposed
ICD still awaits Architect review. No real Claude login, subscription request
or optional live smoke was run. Eleven offline CLI tests include witnessed late
emissions, a valid harness write under allow-all remaining display-only,
credential/Keychain access controls and actual process/profile A-B-A recovery;
72 native/TLC variants include two deliberately broken CLI lifecycle controls.
The source tree is committed; the three canonical JSON run receipts remain
uncommitted evidence from verification. No credentials, release, other worktree
or bboxy450-term change is part of this handoff.
