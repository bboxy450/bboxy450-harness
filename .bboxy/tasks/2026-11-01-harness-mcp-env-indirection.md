---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
artifacts:
- '[[01M6BMWCM6SJX08DWVE1A5X6EG]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T13:12:29.478Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BKTWWKQGW85XN2D4EYVQDV
number: 4344
provenance:
  digest: sha256:21a5fade752ea343baf8e2bb90fb5e6e6b4092e5f411578c8bc1ef08c00137ba
  frontmatter_digest: sha256:5e28387864f2e331ee3268a381b73f1c1114652d867e4f9efe134cdfed77b8a1
  log: .bboxy/log/2026-11-01.jsonl#28199
  sig: uDPSdTNdSIMP2efQMx8iJNtarls8vrxIZYpLFYMZl06m25gxnyVprPpKFc13/lUO0ZopVvQRfnROzaQrVPAsAw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: 3014e49b4851c878294b6d8660e08abedc59c562
  op_id: 01M6BN4HYS2X9MMFQYR3W1S1P2
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: |-
    E1f security complete on e1f-mcp-env-indirection. PR #11 OPEN: https://github.com/bboxy450/bboxy450-harness/pull/11 . Signed head3014e49b4851c878294b6d8660e08abedc59c562; all four commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
    Full committed report: .bboxy/runs/2026-11-01-mcp-env-indirection-report.log; Report: [[2026-11-01-mcp-env-indirection-report]]. Non-launcher reserved variable references are refused as Config before any resolver/spawn, with one Notice and a continuing turn. Every !cmd resolver strips the entire BBOXY namespace. Launcher server inheritance is unchanged.
    Exactly make test=0 unattended, rerun on integrated main1b1d39e at e2890a1. All134 recorded inputs match final records head. Both new controls101→0 at named assertions; actual profile/ACP echo_env cases cover bare/braced exploit maps and failed/empty command resolution; existing launcher witness passes. Immutable env/provenance transforms remain explicitly implementation-only.
    Final-head make test-formal-parity=0 and make check-formal-evidence=0:64 models/25 contracts, clean receiptde18ae25b21550dff8f85e8cefe0df58112f8a1fb6715c15262466b54823b06b. Initial Clippy failure retained and fixed; one rebased console chunk is labelled truncated, structured per-layer records are complete. Tracked tree clean; local proof archives retained. No real credential bytes or live-provider requests. Continuing separately assigned hangup task#4340.
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1f (security): block $BBOXY_* / !cmd indirection into untrusted MCP server env'
type: task
updated: 2026-11-01T13:40:06.395Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Reserved references refused as typed Config before resolution; Notice and continued turns
    reference: src/mcp.rs::validate_environment/source_tests/supervise; tests/support/seat_launch.rs::profile_and_acp_refuse_bboxy_environment_indirection; bboxy-indirection-red.log101→green0; check_transcript
  - criterion: All command resolvers strip full namespace; trusted launcher inheritance unchanged
    reference: src/process.rs::run_inner(capture_only); provider/profile.rs::resolve_command; command_resolver_cannot_copy_seat_environment_into_mcp_aliases; resolver-namespace-red.log101→green0; existing launcher witness
  - criterion: Exact unattended verify, complete inventory and native/TLC parity
    reference: '.bboxy/runs/mcp-env-indirection/post-rebase-test-current.json, post-rebase-gui-gate.json: make test0; all134 inputs match;64 models/25 contracts; final-head receiptde18ae25b21550dff8f85e8cefe0df58112f8a1fb6715c15262466b54823b06b; coverage.toml'
  - criterion: Signed pushed PR and committed report, no assigned gaps
    reference: PR11 head3014e49; allfourcommitsG; .bboxy/runs/2026-11-01-mcp-env-indirection-report.log; controls.json and real red/green outputs
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main, after PR #8 has merged, and open a PR. Do not merge. Binding: `.bboxy/interfaces/mcp.md` §5/§6: a non-launcher server sees no BBOXY_* value under any name. Do this before or alongside #4340; it is security.

The defect, from the Oracle's review of PR #8: a profile or ACP `session/new` can copy seat identity values in under a non-`BBOXY_` name.
- Exploit: `"env":{"S":"$BBOXY_SOCK","I":"$BBOXY_SEAT_ID","R":"!printenv BBOXY_ROLE"}`.
- Cause, part 1: the literal check at `src/mcp.rs:532` matches only the capability.
- Cause, part 2: `src/process.rs:168` removes only `BBOXY_CAPABILITY` from the `!cmd` shell environment.
- ACP env values also go through `$` and `!` resolution (`acp.rs:322-327`).

Fix:
1. For non-launcher servers, refuse any env value that references a `BBOXY_`-prefixed variable through `$NAME` or `${NAME}`. Report it as a typed configuration error and Notice. The session continues and the server does not start.
2. In `!cmd` resolution, remove every `BBOXY_`-prefixed variable from the child's environment, not only the capability. Do this for every caller of that resolver: profile keys, MCP env and ACP env.
3. Launcher (`--mcp-config`) servers are unchanged.

Tests (real spawned `mcp-fixture echo_env`, recording names and match booleans only, never values):
- A profile server and an ACP server, each configured with the exploit map, end up with none of BBOXY_SOCK, BBOXY_SEAT_ID or BBOXY_ROLE.
- `!printenv BBOXY_SOCK` resolves to empty or fails.
- The launcher server still receives them.

Broken controls:
- Remove the `$BBOXY_` refusal. This must go red.
- Narrow the `!cmd` scrub back to the capability only. This must go red.

Acceptance:
- `make test` exits 0 unattended.
- The controls go 101 → 0.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-mcp-env-indirection-report.log`.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1f security complete on e1f-mcp-env-indirection. PR #11 OPEN: https://github.com/bboxy450/bboxy450-harness/pull/11 . Signed head3014e49b4851c878294b6d8660e08abedc59c562; all four commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
Full committed report: .bboxy/runs/2026-11-01-mcp-env-indirection-report.log; Report: [[2026-11-01-mcp-env-indirection-report]]. Non-launcher reserved variable references are refused as Config before any resolver/spawn, with one Notice and a continuing turn. Every !cmd resolver strips the entire BBOXY namespace. Launcher server inheritance is unchanged.
Exactly make test=0 unattended, rerun on integrated main1b1d39e at e2890a1. All134 recorded inputs match final records head. Both new controls101→0 at named assertions; actual profile/ACP echo_env cases cover bare/braced exploit maps and failed/empty command resolution; existing launcher witness passes. Immutable env/provenance transforms remain explicitly implementation-only.
Final-head make test-formal-parity=0 and make check-formal-evidence=0:64 models/25 contracts, clean receiptde18ae25b21550dff8f85e8cefe0df58112f8a1fb6715c15262466b54823b06b. Initial Clippy failure retained and fixed; one rebased console chunk is labelled truncated, structured per-layer records are complete. Tracked tree clean; local proof archives retained. No real credential bytes or live-provider requests. Continuing separately assigned hangup task#4340.

```json
{
  "commit": "3014e49b4851c878294b6d8660e08abedc59c562",
  "op_id": "01M6BN4HYS2X9MMFQYR3W1S1P2",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Reserved references refused as typed Config before resolution; Notice and continued turns",
        "reference": "src/mcp.rs::validate_environment/source_tests/supervise; tests/support/seat_launch.rs::profile_and_acp_refuse_bboxy_environment_indirection; bboxy-indirection-red.log101→green0; check_transcript"
      },
      {
        "criterion": "All command resolvers strip full namespace; trusted launcher inheritance unchanged",
        "reference": "src/process.rs::run_inner(capture_only); provider/profile.rs::resolve_command; command_resolver_cannot_copy_seat_environment_into_mcp_aliases; resolver-namespace-red.log101→green0; existing launcher witness"
      },
      {
        "criterion": "Exact unattended verify, complete inventory and native/TLC parity",
        "reference": ".bboxy/runs/mcp-env-indirection/post-rebase-test-current.json, post-rebase-gui-gate.json: make test0; all134 inputs match;64 models/25 contracts; final-head receiptde18ae25b21550dff8f85e8cefe0df58112f8a1fb6715c15262466b54823b06b; coverage.toml"
      },
      {
        "criterion": "Signed pushed PR and committed report, no assigned gaps",
        "reference": "PR11 head3014e49; allfourcommitsG; .bboxy/runs/2026-11-01-mcp-env-indirection-report.log; controls.json and real red/green outputs"
      }
    ],
    "exit_code": 0
  }
}
```
