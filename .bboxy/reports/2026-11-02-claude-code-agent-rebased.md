---
id: "01M6EXMST69ZDYAV02MYEZK4QP"
number: 5176
type: report
title: "Claude Code handoff on the current rich GUI baseline"
task: "tasks/2026-11-02-harness-claude-code-agent.md"
supersedes: "reports/2026-11-02-claude-code-agent-final"
commits:
  - "e774cb259101b396a03b6288da2b6e28098d5a60"
  - "dbc7bc86b8d92b101182c47d45982f93088061e5"
verify:
  command: "make test && make test-formal"
  exit_code: 0
author: "seat:01M6ESNX4AS5VZV13S8K6VA3RT"
created: "2026-11-02"
---
# Handoff after integration with current main

The implementation and complete acceptance matrix remain in
[[reports/2026-11-02-claude-code-agent-controls]] and the full mechanisms in
[[reports/2026-11-02-claude-code-agent]]. The resource-failure disposition remains
in [[reports/2026-11-02-claude-code-agent-final]]. Those records are retained as
history; this record supersedes their source and final verification reference.

Oracle decisions 01M6EVHHG595P4GRG6QJRFV1S8 and
01M6EWR9TJWHJGQA09N8ZZ08PA require rebasing onto current main before the final
gate and avoiding overlap with another scripts/test.py gate. The branch is now
rebased, with signed commits, onto origin/main b503f35 (base Clippy fixes,
rich GUI face and folded-card model). No merge was made. The two conflicts were
resolved in the assigned worktree: src/frontends/headless.rs retains main's
baseline helper layout plus only the CLI provider-factory selection; the formal
corpus retains main's folded-card model unchanged and adds the CLI family.

A byte comparison removed only the added CLI function and its three corpus
entries from tests/formal/models.rs and required exact equality with main.
Baseline witnesses and scripts/test.py were not changed. verification/README.md
now records 74 variants/27 contracts, including the inherited folded-card pair.
The CLI family still has 1289 states/2362 edges in the good graph, plus the
nonvacuous Fence and Resume safety controls.

Current implementation commits are e774cb259101b396a03b6288da2b6e28098d5a60
and dbc7bc86b8d92b101182c47d45982f93088061e5. They replace the old 4105eef and
1a7f0af references in the historical records. The five rebased commits all
reported good bboxy450 signatures. New report commits are signed as well.

Acceptance evidence at current source: src/provider/profile.rs:90/:161 selects
the native agent before any credential resolver or HTTP client;
src/engine/mod.rs:448 bypasses the harness tools/context loop;
src/engine/claude_code.rs:124/:478/:670/:740 projects bounded stream-json,
spawns the registered owned process group and joins cancellation. The 11 tests
in tests/claude_code_agent.rs cover text, thinking, external tool/result/text,
usage/cost, multiple turns in one process, errors, hostile framing, unknown
notices, crash status/redacted stderr, Auth/missing binary, version pin,
actual late emissions during cancel grace, <2 s/group teardown, credential and
Keychain negative controls, real harness restart and profile A-B-A recovery.
The allow-all valid harness write remains display-only. The formal family is
now at tests/formal/models.rs:2013, with tests/formal_nonvacuity.rs:504 and the
paired inventory. The ICD remains proposed, with the approved OwnedChild
transport ruling, exact 2.1.304 probe, mappings, lifecycle and compliance line.

The complete pre-rebase gate on 3d70ffe exited 0 and its clean-source reference
receipt was 5df0fc1818642268234b326d88b43dff14e60532b263c38fc7afe2a99d2ca542.
It is retained, with all three run JSONs and terminal log, beneath that
verification/evidence digest as pre-rebase evidence. It is not acceptance for
the rebased source. The earlier ENOSPC linker failure remains recorded in the
prior disposition and /tmp/claude-code-enospc-gate.log; no tool failure was
counted as a property verdict.

A fresh `make test && make test-formal` is run to completion on this report
commit with CARGO_BUILD_JOBS=2, after pgrep confirms no other scripts/test.py.
Actual final SHA/exits are supplied in task_report. The canonical source-hash
check, JSON command receipts, GUI structural smoke and native/TLC full graph
comparison decide the result. make check-formal-evidence must also exit 0 on
this exact HEAD. Updated receipts are .bboxy/runs/test-current.json,
gui-gate.json, formal-native.json, and target/formal/evidence.json; retained
reference evidence is under verification/evidence/<digest>. Terminal log:
/tmp/claude-code-rebased-gate.log. Three generated run JSONs remain uncommitted
as current evidence; all implementation and report source is committed.

PR #24 is updated to this verified signed head and remains open to main,
without merging. Architect review of the proposed ICD is still required.
No real login, subscription request or optional live smoke was run. CLI tools,
permissions, prompt, MCP and context stay owned by Claude Code; text and file
mentions are supported, images are refused and Steer queues the next turn.
The documented CLI UUID-ordering, monitored-symbol and finite-model/OS/IO
limits still apply. No credentials, baseline witnesses, other worktree or
bboxy450-term files were changed by this assignment.
