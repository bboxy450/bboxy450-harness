---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T20:19:18.838Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CC8E3X4PKTAHT1TPYRK31Y
number: 4902
provenance:
  digest: sha256:93a2a08c8115e55c8e2028a24a8bbbf0849863100f51043fbd02169056b16f54
  frontmatter_digest: sha256:f3fcecf9725b8daf483327c3c1cb8c2ff9c9e6ec9f34b8ce31487a49a300b658
  log: .bboxy/log/2026-11-01.jsonl#31586
  sig: 0citXz3Vo9Qd/o40SxMqlJ4ZwLT+WdC7hPcvauzPx2cOYvjnwIW2Bs21Rke0AwGQpDnPAGPfA0jrWfsN6MoBCQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3e (security): harness Side B of capability-file delivery (#4503)'
type: task
updated: 2026-11-01T20:19:21.758Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main and open a PR. Do not merge. Security: do this first.

Binding: `.bboxy/interfaces/seat-capability-delivery.md` (#4503) Side B, and mcp.md §5, at harness main 011d24b or later. Side A landed in bboxy450-term at afda1b63: seats get `BBOXY_CAPABILITY_FILE` (a path) and never `BBOXY_CAPABILITY`.

Regression to fix, found by the Architect:
- Every harness face still reads only `BBOXY_CAPABILITY`: mcp.rs:577,581, headless.rs:155, gui/window.rs:343, smoke.rs:74. That variable is no longer set, so the redactor never learns the value and the mcp §5 resolved-value refusal never fires.
- process.rs:466 strips only `BBOXY_CAPABILITY`, so tools inherit `BBOXY_CAPABILITY_FILE`.
- Result: in a harness seat, `cat "$BBOXY_CAPABILITY_FILE"` puts the value unredacted into the transcript and logs.

Required, per the ICD Side B text:
1. Every face reads the capability from `BBOXY_CAPABILITY_FILE` with the same consumer checks as Side A. Those checks are O_NOFOLLOW, fstat, regular file, owned by euid, mode 0600, exactly 64 lowercase hex plus newline, and typed refusals with no path or value in the message. The face registers the value with the redactor before any tool, provider or MCP runs. A missing or invalid file degrades: a Notice, and the session continues without it. Never block.
2. The mcp §5 refusal compares resolved values against that value.
3. Tools and terminals strip `BBOXY_CAPABILITY_FILE` along with every other `BBOXY_*` name except where the ICD says otherwise. Launcher (`--mcp-config`) MCP servers keep the file path per the ICD. Check what Side B says for launcher servers and follow it exactly.

Tests are the ICD verification cases (a)-(d), with real processes:
- a tool that runs `cat "$BBOXY_CAPABILITY_FILE"` and `printenv` gets nothing and leaks nothing into the transcript or logs;
- if a value does reach output, it appears only redacted;
- the §5 refusal fires on a resolved value equal to the capability;
- each face (headless, seat, ACP, GUI smoke) loads the file.

Broken control: revert the faces to reading `BBOXY_CAPABILITY`, and the leak test must go 101 → 0. Update the model if the ICD names one; otherwise record the area as implementation-only in coverage.toml, one entry per line.

Acceptance:
- `make test` exits 0 unattended, and native/TLC parity holds.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-capability-side-b-report.log`.
- Send one task_report with verify_result.command exactly `make test`.