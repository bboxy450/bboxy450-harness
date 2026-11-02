---
id: "01M6EVGB3N0ZZPWA7VEGEKSDKJ"
number: 5173
type: report
title: "Claude Code agent session implementation and verification"
task: "tasks/2026-11-02-harness-claude-code-agent.md"
commits:
  - "4105eef505c93e6796ae8119949db4d6dbfacac4"
verify:
  command: "make test && make test-formal"
  exit_code: 0
author: "seat:01M6ESNX4AS5VZV13S8K6VA3RT"
created: "2026-11-02"
---
# Handoff

Implemented provider: claude-code as the official CLI's own long-lived agent.
The harness sends user stream-json input, renders its events and owns process
lifecycle; it does not supply a replacement prompt, run its tool calls or obtain
Claude credentials. The proposed ICD is [[interfaces/claude-code-agent]].

Implementation commit: `4105eef505c93e6796ae8119949db4d6dbfacac4`.
`git verify-commit` exited 0, with the bboxy450 signing key and Boxy identity.
This report is a separate signed commit so final verification can bind to a
stable HEAD containing both source and report. The authoritative acceptance
results are the generated final-HEAD receipts below; task_report is sent only
after both required commands exit 0. No merge or release is performed.

## Mechanism and changed files

- src/provider/profile.rs:90 — validates CLI model/permission options, allows an
  omitted endpoint/model, refuses api_key/api_key_env for this provider.
  :155 selects the external agent before any key resolver or HTTP client.
  :180 refuses direct credential resolution for CLI profiles.
- src/provider/claude_code.rs:46 — fixed argv allowlist; :77 bounded,
  cancellation-safe JSONL framing, with large parses off the IO workers; :141
  registered-secret/credential-syntax scrubbing; :150 usage conversion.
- src/provider/mod.rs:117 — default external-agent selection hook. Its CLI
  implementation refuses completion-stream use rather than proxying tools.
- src/engine/mod.rs:288 — each SessionActor owns its independent CLI worker;
  CLI sessions omit harness MCP startup. :448 bypasses the harness TurnRunner,
  system prompt, context assembly and tool registry. :658 refuses harness
  compaction. :711 closes the old agent on profile change and restores the
  matching profile's durable CLI ID.
- src/engine/claude_code.rs:29 — restores the UUID from the existing log's init
  Notice. :124 projects assistant text/thinking and external display-only
  tool calls/results; :234 closes all pending tools before one terminal.
  :404 drains idle events. :478 writes new user turns, parses init/result and
  unknown events, applies startup/stall bounds, and preserves usage/errors.
  :670 uses the existing registered OwnedChild on bbt_rt's IO runtime with
  inherited environment and workspace cwd. :729 observes leader exit without
  premature reap. :740 signals, joins, kills residual members and redacts the
  bounded stderr tail. :792 checks the exact 2.1.304 pin before each birth.
- src/process.rs:436 and :607 — crate visibility only for the existing leader
  observation and termination helpers. Oracle ruling
  `01M6ESRQ3W4WEEEZ3F0SWHEZ2R` approved OwnedChild because the requested
  bboxy450-rt 0.1.1 spawn_command hardcodes null stdin. The ICD records the
  original brief line and this replacement. No bboxy450-term write occurred.
- src/frontends/headless.rs:195, src/gateway/host.rs:247 — share the provider
  factory so headless, seat, GUI/Host and ACP selection preserve the boundary.
  src/frontends/smoke.rs — initializes the added optional profile field.
  Existing headless Clippy failures were fixed mechanically: collapse one if,
  move the unchanged seat_command helper before the test module.
- tests/fixtures/claude-code/claude — executable offline CLI, version response,
  scripted JSONL, errors, duplicate events and stubborn descendant fixture.
  tests/fixtures/claude-code/deny_credentials.c — macOS interposer and its
  deliberately rejected file-open/Keychain probes.
- tests/claude_code_agent.rs — 11 parser/profile/real-process tests.
  tests/formal/models.rs:1963 and tests/formal_nonvacuity.rs:504 — lifecycle
  abstraction and reachable A-B-A/failing-control witnesses.
- verification/contracts.toml:269, corpus.toml:343, coverage.toml — add the
  contract, two broken controls, every new Rust source and implementation-only
  limits. Existing make test-formal discovers the family through this corpus.
  verification/README.md records the bounds and current inventory (72 variants,
  26 contracts). verification/.gitignore retains generated evidence locally
  without staging run archives as source.

Root ATTRIBUTION.md already names the canonical project and policy. All new
source files carry its requested header; the root policy was left intact.

## Acceptance evidence

| Assigned criterion | Executable evidence |
|---|---|
| Profile provider: claude-code, own CLI/login/tools | tests/claude_code_agent.rs:36 and :301; provider factory precedes key lookup, observed argv is exact, fake Write is displayed and does not write the workspace |
| Text-only turn and two turns over one process | :301 checks first/last replies, six completed turns and exactly one long-lived CLI invocation |
| tool_use → tool_result → text | :301 checks the prefixed ToolCall, output and valid complete transcript |
| Thinking | :301 asserts the Reasoning summary |
| Usage/cost/is_error/subtype | :301 checks 12 input, 7 output, 3 cached tokens and 123 USD micros; :343 checks an error result |
| Malformed/oversized/truncated/untrusted lines | :14 exercises framing directly; :343 exercises malformed and oversized stdout in real child processes |
| Unknown event is noticed | :301 asserts a Notice and subsequent successful text |
| Mid-turn crash | :343 checks failure, exit code 17, tool pairing and stderr redaction |
| Missing/not logged in | :343 covers both result-level and startup-stderr Auth; :376 covers missing executable and wrong version |
| Cancellation and group join within 2 s | :388 checks Cancelled, elapsed time and ESRCH on owned groups, including late/duplicate output during the grace interval |
| Startup/Close race still ends once | :544 closes at TurnStarted, including a hanging version probe; Cancelled and <2 s are asserted |
| Resume A → B → A | :388 checks three CLI births with --resume; :503 kills and reloads the real harness twice from the durable log; :569 switches profiles A/B/A and checks A's matching ID |
| No replacement prompt/tools-off flags, credentials or Keychain access | :69 checks the argv allowlist; :436 observes the actual child argv, runs the instrumented harness and first requires both broken access probes to exit 91/92. The fixture confirms inherited OAuth env with a boolean only, never its value |
| Formal lifecycle, late/duplicate events, terminal conservation and resume ID | tests/formal/models.rs:1963; native/TLC parity checks the full graph. Good: 1289 states/2362 edges. Fence control: 1613/3052, safety failure. Resume control: 1289/2362, safety failure. The nonvacuity test requires the held original completion to survive through birth 2 |
| Proposed ICD | .bboxy/interfaces/claude-code-agent.md includes owners, argv, events/mapping, lifecycle, failures, version probe, compliance, assumptions, verification and the Oracle transport ruling |

## Commands and actual results

Preflight commands, before the final acceptance run:

- `cargo test --offline --locked --test claude_code_agent --test profile_lifecycle --test e2e_scripted --test cancel_process --test formal_models --test formal_nonvacuity` — exit 0. CLI 11 tests, profile 2, scripted e2e 5, process cancel 4, formal models 2, nonvacuity 15. Log: /tmp/claude-code-tested.log.
- `cargo clippy --offline --locked --all-targets --all-features -- -D warnings` — exit 0. Log: /tmp/claude-code-precommit-clippy.log.
- `python3 -I scripts/formal.py --target target/formal` — exit 0; all 72 variants' full state/transition sets and native/TLC verdicts agree. This warmed the checksum-pinned reference-tool cache before the canonical run. Log: /tmp/claude-code-parity-preflight.log. Its dirty precommit receipt is not final acceptance evidence.
- `python3 -I scripts/checks.py headers`, `kb-records`, `inventory-shape`, and `git diff --check` — exit 0.
- `git verify-commit 4105eef505c93e6796ae8119949db4d6dbfacac4` — exit 0, good bboxy450 signature.

The final acceptance command is `make test && make test-formal`, run once on the
report commit with source unchanged. Its exact HEAD, command exits and source
hashes are in .bboxy/runs/test-current.json, gui-gate.json and formal-native.json.
The pinned reference receipt is target/formal/evidence.json, archived with its
models under verification/evidence/<digest>/ beyond target cleanup. A final
`make check-formal-evidence` requires clean source and the same HEAD. The terminal
log is /tmp/claude-code-final-gate.log; the tool task_report supplies the final
commit and actual combined exit status. These script receipts, rather than the
report's claims, are the acceptance authority's evidence.

## Failures, limits and deliberate omissions

During development, targeted checks exposed and corrected: missing native Expr
comparison syntax; a test that sent a POSIX signal instead of the supported
Gateway Interrupt; an old recovery marker being mistaken for the newest replay
head; and the two existing Clippy findings above. Current focused checks pass.
No full canonical run was used for iteration before the final report commit.

The interface remains proposed pending Architect review. No real Claude login,
subscription request, external-network completion or live smoke was run. The
optional smoke-live-claude-code target was not added. The 2.1.304 pin is verified
at runtime and by the offline version fixture; this report makes no live-service
terms or subscription-success claim.

Supported user input is text and file mentions. CLI tool permissions, MCP,
context and prompt remain owned by Claude Code. Steer becomes the next queued
turn. The CLI is assumed to emit unique event UUIDs and no new-UUID prior-turn
terminal after a healthy result; cancelled/restarted groups are independently
fenced and joined. The macOS monitor covers credential-file opens, the monitored
Keychain API and security-tool spawn; it is a process witness, not a universal
proof about arbitrary linked code. Finite models do not prove arbitrary Rust
refinement, OS deadlines, wire parsing or the official CLI's internals.

Only the assigned worktree and allowed paths were changed. No merge, release,
credential mutation or bboxy450-term/main-worktree edit is part of this handoff.
