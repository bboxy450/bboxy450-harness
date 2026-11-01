---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- Makefile
- .bboxy/runs/
artifacts:
- '[[01M6BKHK2VX1JC42YMK26ZTKQS]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T12:16:29.494Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BGMBQVZF1WN4FX5C3X5E9M
number: 4069
provenance:
  digest: sha256:9093dec87b89d700879cd4ea0277a393a3c4a6952637e8c94b79194b62ef8698
  frontmatter_digest: sha256:5d0d9941488648d1d98f1d2ba592231b5596a72e2da5040679a59fcdc837f0c2
  log: .bboxy/log/2026-11-01.jsonl#27243
  sig: xxobYHz2kpvcsB53YlJVQJ8gv34uYDA6V5ClRTzn+J8pfhAG8OFmC26z5XYUsT2euY83csO6nzGVNz5+5NoMBA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: 83bb5559d3dfecad904717f280c7f430b80d73f6
  op_id: 01M6BKQC2JXS4205SSJP3RQG0C
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: |-
    E1d complete on e1d-seat-flags-mcp-env. PR #8 is ready: https://github.com/bboxy450/bboxy450-harness/pull/8 . Signed head83bb5559d3dfecad904717f280c7f430b80d73f6; source/records commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge. The draft was published early after the flag tests passed for E4.
    Full committed report: .bboxy/runs/2026-11-01-seat-flags-mcp-env-report.log (Report: [[2026-11-01-seat-flags-mcp-env-report]]). Profile and ACP servers lose all inherited/configured BBOXY_*; launcher provenance cannot come from server JSON, and only the --mcp-config source inherits the namespace. Startup text is inside the stable system prefix; launcher tools retain mcp__bboxy__ names; missing/unreadable config produces a Notice and a completed turn; unknown flags remain errors.
    Exactly make test exited0 unattended on96ce2d3 after rebasing onto current rich-GUI main. All132 source inputs match final records head. Four new controls each101→0 at their named assertion; scrub control requires both profile and ACP failures. Immutable origin/environment projection is explicitly implementation-only in coverage.toml, not a new MCP lifecycle transition.
    Current-head make test-formal-parity=0 and make check-formal-evidence=0:64 models/25 contracts, clean receipt4a4bf788fc183dee8c87a66b4bab334915a6b2c2defbae7f98d4c4e5161ac39c. Initial parallel PTY deadline failures are retained; the gate runs launcher fixtures separately from PTY cases, preserving every test and the original deadlines. No real credential value was printed/committed; tests clear the environment and use throwaway values. No new live-provider requests. Tracked worktree clean, local proof archives retained. Side A phase1a integration remains E4; no automatic wake/persistence claim.
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1d: seat --append-system-prompt/--mcp-config flags + scrub BBOXY_* from MCP server env'
type: task
updated: 2026-11-01T13:15:21.944Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: 1 SECURITY-first full BBOXY namespace scrub with launcher-only exception
    reference: src/mcp.rs::scrub_bboxy_env, Origin, select_sources and Slot; tests/support/seat_launch.rs profile_server_cannot_inherit_bboxy_environment + acp_server_cannot_inherit_bboxy_environment, launcher_flags_deliver_context_tools_and_only_launcher_environment; scrub-baseline-red.log101→scrub-green.log0; controls.json bboxy-scrub requires both named failures101→0, launcher-inherit101→0
  - criterion: 2 Both launcher flags, stable context, bboxy tool namespace, safe unavailable config, strict arguments
    reference: src/frontends/headless.rs flags/read_launch; engine::LaunchOptions and request_context; Host::with_launch create/open; ACP serve_with_host; first actual HTTP/SSE request contains startup text and mcp__bboxy__read; direct Recording ModelRequest checks messages[..stable_prefix]; flags-green.log5pass; append-system and missing-mcp-config controls each101→0
  - criterion: 3 Honest model classification and unchanged stateful contracts
    reference: 'verification/coverage.toml implementation_only row: source selection and env are immutable per-slot input transforms, covered by unit/process tests; existing MCP lifecycle transitions unchanged; full64-model native/TLC graph/verdict parity and crate/source inventory hold'
  - criterion: Exact unattended canonical acceptance on integrated main and identical final inputs
    reference: '.bboxy/runs/seat-flags-mcp-env/make-test.log, test-current.json, gui-gate.json, verification.json: make test=0, every child exit0, private HOME, no operator; separate old3PTY and new5launcher cases; rich-GUI tests/smoke PASS; all132 recorded input hashes match83bb555'
  - criterion: Each new control has named101→0 evidence
    reference: 'python3 -I scripts/test-e1c-rework-controls.py --case bboxy-scrub --case launcher-inherit --case append-system --case missing-mcp-config --out .bboxy/runs/seat-flags-mcp-env exited0; controls.json and red/green logs: profile+ACP inherited vars, launcher lost capability, missing stable-prefix text, missing Notice; sources restored in finally'
  - criterion: Signed pushed PR, full report, current-head formal proof and explicit test-isolation history
    reference: .bboxy/runs/2026-11-01-seat-flags-mcp-env-report.log; PR8 head83bb5559d3dfecad904717f280c7f430b80d73f6; git verify-commit2b4ad7c96ce2d3HEAD=0; final-head formal/evidence checks0, receipt4a4bf788fc183dee8c87a66b4bab334915a6b2c2defbae7f98d4c4e5161ac39c; targeted-initial-failure.log + targeted-second-failure.log retained, pty-isolated.log and pty-group.log0, fullcanonical0
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main (cdde43d or later) and open a PR. Do not merge.

Binding: `.bboxy/interfaces/embed-seat.md` §1/§2/§6 and `.bboxy/interfaces/mcp.md` §2/§5/§6 (record #1988, re-signed), at main 9fdddce or later. Read both before coding.

1. **SECURITY first: MCP server environment** (`src/mcp.rs:442`). Servers currently inherit the full environment, so every profile or ACP MCP server receives `BBOXY_CAPABILITY`.
   - Per mcp §5, strip every `BBOXY_*` variable from a spawned server's environment.
   - The one exception is servers that come from the launcher's `--mcp-config` file.
   - Tests per mcp §6:
     - A profile server and an ACP-supplied server each see no `BBOXY_*`.
     - A launcher `--mcp-config` server does see them.
     - A broken control that removes the scrub must go red.
2. **Seat flags** (`src/…/headless.rs:59-110`). Add `--append-system-prompt <text>` and `--mcp-config <path>` per embed-seat §1:
   - The text is appended to the system prompt.
   - The file is loaded as launcher MCP servers, which are exempt from the scrub in item 1.
   - Tool names stay `mcp__bboxy__*`, per mcp §2.
   - Unknown options are still errors.
   - Tests per embed-seat §6:
     - With both flags, the first turn's system prompt contains the text.
     - The launcher server's tools are listed under `mcp__bboxy__`.
     - A missing or unreadable `--mcp-config` prints a Notice and the turn still completes (never block).
3. If the scrub or launcher-trust distinction is a stateful contract in an existing model (HarnessMcpServer), update the model and add a broken control. Otherwise record it as covered by implementation tests only, in verification/coverage.toml.

Acceptance:
- `make test` exits 0 unattended.
- Each new control goes 101 → 0 at its named assertion.
- Native/TLC parity holds, and all commits are signed.
- Report in `.bboxy/runs/2026-11-01-seat-flags-mcp-env-report.log`.
- One task_report with verify_result.command exactly `make test`.

E4 (bboxy450-term seat spawn) depends on item 2, so put item 2 on the PR as soon as it is green.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1d complete on e1d-seat-flags-mcp-env. PR #8 is ready: https://github.com/bboxy450/bboxy450-harness/pull/8 . Signed head83bb5559d3dfecad904717f280c7f430b80d73f6; source/records commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge. The draft was published early after the flag tests passed for E4.
Full committed report: .bboxy/runs/2026-11-01-seat-flags-mcp-env-report.log (Report: [[2026-11-01-seat-flags-mcp-env-report]]). Profile and ACP servers lose all inherited/configured BBOXY_*; launcher provenance cannot come from server JSON, and only the --mcp-config source inherits the namespace. Startup text is inside the stable system prefix; launcher tools retain mcp__bboxy__ names; missing/unreadable config produces a Notice and a completed turn; unknown flags remain errors.
Exactly make test exited0 unattended on96ce2d3 after rebasing onto current rich-GUI main. All132 source inputs match final records head. Four new controls each101→0 at their named assertion; scrub control requires both profile and ACP failures. Immutable origin/environment projection is explicitly implementation-only in coverage.toml, not a new MCP lifecycle transition.
Current-head make test-formal-parity=0 and make check-formal-evidence=0:64 models/25 contracts, clean receipt4a4bf788fc183dee8c87a66b4bab334915a6b2c2defbae7f98d4c4e5161ac39c. Initial parallel PTY deadline failures are retained; the gate runs launcher fixtures separately from PTY cases, preserving every test and the original deadlines. No real credential value was printed/committed; tests clear the environment and use throwaway values. No new live-provider requests. Tracked worktree clean, local proof archives retained. Side A phase1a integration remains E4; no automatic wake/persistence claim.

```json
{
  "commit": "83bb5559d3dfecad904717f280c7f430b80d73f6",
  "op_id": "01M6BKQC2JXS4205SSJP3RQG0C",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1 SECURITY-first full BBOXY namespace scrub with launcher-only exception",
        "reference": "src/mcp.rs::scrub_bboxy_env, Origin, select_sources and Slot; tests/support/seat_launch.rs profile_server_cannot_inherit_bboxy_environment + acp_server_cannot_inherit_bboxy_environment, launcher_flags_deliver_context_tools_and_only_launcher_environment; scrub-baseline-red.log101→scrub-green.log0; controls.json bboxy-scrub requires both named failures101→0, launcher-inherit101→0"
      },
      {
        "criterion": "2 Both launcher flags, stable context, bboxy tool namespace, safe unavailable config, strict arguments",
        "reference": "src/frontends/headless.rs flags/read_launch; engine::LaunchOptions and request_context; Host::with_launch create/open; ACP serve_with_host; first actual HTTP/SSE request contains startup text and mcp__bboxy__read; direct Recording ModelRequest checks messages[..stable_prefix]; flags-green.log5pass; append-system and missing-mcp-config controls each101→0"
      },
      {
        "criterion": "3 Honest model classification and unchanged stateful contracts",
        "reference": "verification/coverage.toml implementation_only row: source selection and env are immutable per-slot input transforms, covered by unit/process tests; existing MCP lifecycle transitions unchanged; full64-model native/TLC graph/verdict parity and crate/source inventory hold"
      },
      {
        "criterion": "Exact unattended canonical acceptance on integrated main and identical final inputs",
        "reference": ".bboxy/runs/seat-flags-mcp-env/make-test.log, test-current.json, gui-gate.json, verification.json: make test=0, every child exit0, private HOME, no operator; separate old3PTY and new5launcher cases; rich-GUI tests/smoke PASS; all132 recorded input hashes match83bb555"
      },
      {
        "criterion": "Each new control has named101→0 evidence",
        "reference": "python3 -I scripts/test-e1c-rework-controls.py --case bboxy-scrub --case launcher-inherit --case append-system --case missing-mcp-config --out .bboxy/runs/seat-flags-mcp-env exited0; controls.json and red/green logs: profile+ACP inherited vars, launcher lost capability, missing stable-prefix text, missing Notice; sources restored in finally"
      },
      {
        "criterion": "Signed pushed PR, full report, current-head formal proof and explicit test-isolation history",
        "reference": ".bboxy/runs/2026-11-01-seat-flags-mcp-env-report.log; PR8 head83bb5559d3dfecad904717f280c7f430b80d73f6; git verify-commit2b4ad7c96ce2d3HEAD=0; final-head formal/evidence checks0, receipt4a4bf788fc183dee8c87a66b4bab334915a6b2c2defbae7f98d4c4e5161ac39c; targeted-initial-failure.log + targeted-second-failure.log retained, pty-isolated.log and pty-group.log0, fullcanonical0"
      }
    ],
    "exit_code": 0
  }
}
```
