---
id: "01M6EW88Q4VKMX9HBC6YKW56CD"
number: 5174
type: report
title: "Claude Code final handoff with witnessed late cancellation"
task: "tasks/2026-11-02-harness-claude-code-agent.md"
supersedes: "reports/2026-11-02-claude-code-agent"
commits:
  - "4105eef505c93e6796ae8119949db4d6dbfacac4"
  - "1a7f0af11b850e90db8f89d09c56941b1cf4afe7"
verify:
  command: "make test && make test-formal"
  exit_code: 0
author: "seat:01M6ESNX4AS5VZV13S8K6VA3RT"
created: "2026-11-02"
---
# Final handoff

Provider: claude-code runs the official CLI's own agent loop, tools, permissions
and login through the existing registered OwnedChild on bbt_rt. The proposed
ICD records Oracle ruling `01M6ESRQ3W4WEEEZ3F0SWHEZ2R`, replacing spawn_command
because its published stdin is null. No credential resolver, HTTP completion
proxy, replacement system prompt or harness tool dispatch is used.

The complete mechanism, changed-file inventory and original acceptance table are
in [[reports/2026-11-02-claude-code-agent]]. This append-only correction strengthens
two process witnesses and preserves API-profile validation; it supersedes the
prior report's final verification claim and late-producer evidence.

The earlier complete gate on ae60f5d1e62046b7648b682bae687e354666d9e3 exited 0
(log: /tmp/claude-code-initial-candidate-gate.log). Review showed its parent
fixture ignored SIGINT but accepted SIGTERM, so it could die before emitting the
intended late events. That gate did not prove the late producer ran. The current
fixture ignores both signals, waits for its descendant's signal-ready handshake,
then emits late text and duplicate results during grace. A boolean trace must
prove those emissions occurred; the GUI event history must contain none of the
stale text, exactly one Cancelled ending and no surviving owned group within 2 s.

## Final changes and evidence

- src/provider/profile.rs:119 — model remains required for API providers before
  any credential resolution; only CLI profiles allow omission. The new assertion
  at tests/claude_code_agent.rs:36 guards the validation boundary.
- tests/fixtures/claude-code/claude:90 — signal-resistant leader/member and ready
  handshake; :102 records actual late/duplicate emission.
  tests/claude_code_agent.rs:392/:413 asserts the producer ran, output was retired,
  cancellation stayed below 2 s, group lookup returned ESRCH and three births
  used the stored resume ID.
- The external tool fixture now uses the harness's valid `write` name and
  `path/content` arguments, while the harness profile allows all effects. At
  tests/claude_code_agent.rs:305 the call is displayed with its result but the
  target file stays absent. This would fail if it were dispatched as a harness
  write; the earlier uppercase name/alternate argument could not prove that.
- tests/claude_code_agent.rs:14/:36 — bounded hostile framing, optional CLI model,
  credential-source refusal and API model validation; :305 — text, six turns in
  one process, tool/result/text, thinking, unknown notices, duplicate retirement,
  token/cache/cost mapping and inherited-environment boolean.
- :347/:380 — is_error/subtype, malformed/oversized stdout, crash status 17 with
  redacted stderr, result/startup Auth, missing binary and version mismatch.
  :446 — actual argv allowlist and monitored credential-file/Keychain access,
  with both deliberately broken probes first required to fail (91 and 92).
- :513 — two actual harness crashes and replay recover the CLI ID; :554 — Close
  at admission and during version startup ends Cancelled within 2 s;
  :579 — profile A/B/A resumes only A's matching durable ID.
- tests/formal/models.rs:1963 and tests/formal_nonvacuity.rs:504 — idle/turn/
  cancelling/exited/resuming, three births/turns, duplicate/late events, terminal
  conservation, resume identity and named weak fairness. Good graph: 1289 states,
  2362 edges; Fence control: 1613/3052, safety failure; Resume control: 1289/2362,
  safety failure. The 72-model native/TLC corpus and source inventory remain wired
  into make test-formal; no stateful production transition changed in this correction.
- .bboxy/interfaces/claude-code-agent.md remains proposed for Architect review;
  it covers owners, approved transport, argv, mappings, lifecycle, failure modes,
  version 2.1.304 checks, compliance and abstraction limits.

`cargo test --offline --locked --test claude_code_agent --test provider_stream
--test profile_lifecycle` exited 0 after this correction: 11 CLI tests, 6 provider
stream tests, 2 profile lifecycle tests. Log: /tmp/claude-code-strong-controls.log.

Final acceptance is a fresh single `make test && make test-formal` on the signed
report commit, after these source changes. Its actual combined exit status and
commit are supplied by task_report. Source unchanged throughout is enforced by
the canonical stack. Final receipts: .bboxy/runs/test-current.json, gui-gate.json,
formal-native.json, and target/formal/evidence.json; archived reference evidence
and generated models remain under verification/evidence/<digest> beyond target
cleanup. `make check-formal-evidence` verifies clean source, final HEAD and hashes.
The final terminal log is /tmp/claude-code-final-gate.log. Earlier receipts do not
stand in for this final-HEAD gate.

All commits are signed with the bboxy450 machine key; each includes a Report
trailer. Implementation changes stay in the assigned worktree/paths. The canonical
commands also update the three tracked .bboxy/runs JSON receipts; these are left
uncommitted as current verification evidence. No other implementation changes
are intentionally left uncommitted. A PR is opened to main and is not merged.

No real login, live subscription request, network completion or optional live
smoke was run. The runtime version probe is tested with the offline executable.
Text/file mentions are supported; images are refused. Steer queues the next
turn. Claude Code owns its own prompt, tools, permissions, MCP and context.
The healthy-CLI UUID ordering assumption, monitored-symbol limits and finite
model/OS/IO abstraction limits in the proposed ICD still apply. Root attribution
already satisfies the policy; every generated source retains the requested header.
