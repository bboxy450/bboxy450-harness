---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
artifacts:
- '[[01M6CF2CS6SY6EJ4T5Y4GH60N1]]'
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
  digest: sha256:290253a9149d687fdf13a2af2c5b277e438b7abc4d6a3832a544c0938748b560
  frontmatter_digest: sha256:7af29725a9c610ce0e5445df104306fe7026ce5f0166946ec07950e7cb4e889e
  log: .bboxy/log/2026-11-01.jsonl#31832
  sig: KycPBVGY9Wctx71UtI08k1P/0kMV/HeXZUS491eVPZHZIWz2KW/sR8RIVDKVOSaXX7uG14tICGW8ig0n0+f2CA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: b7736074c41028fd10bf12536c9dae646fde50c2
  op_id: 01M6CF3GWKN4Q87WYP0TF5HWHN
  reported_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
  summary: 'Implemented Side B on capability-file-side-b and opened PR #18: https://github.com/bboxy450/bboxy450-harness/pull/18 (OPEN, remote head verified b7736074c41028fd10bf12536c9dae646fde50c2). Both commits are signed G by 9B07E849480C68328CA20684844ECC0587DE7AB0. Full unattended make test exited 0 on source b2c078c414a409d033c58431aef022595645cbb2; final commit adds only .bboxy/runs evidence and has identical gate input hashes. Final-head native/TLC parity and check-formal-evidence also exited 0 (68 variants, source-clean). Committed report: .bboxy/runs/2026-11-01-capability-side-b-report.log, SHA256 09560d4e53c5b1c7069788ee5172017c516de3bfb617da563df1aa9e6058d80f; artifact claim 01M6CF2CS6SY6EJ4T5Y4GH60N1 accepted. No merge. Worktree tracked files clean; verifier graph archives remain untracked under verification/evidence/.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3e (security): harness Side B of capability-file delivery (#4503)'
type: task
updated: 2026-11-01T21:18:48.812Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Validated file loading in every face before providers, tools and MCP; typed path/value-free refusals and degraded continuation
    reference: 'src/capability.rs:21,57,73,99: O_NOFOLLOW|O_NONBLOCK, metadata plus descriptor fstat, regular/euid/exact0600 and bounded 64 lowercase hex+newline; Zeroizing storage. headless.rs:132 covers headless/seat/ACP; gui/bridge.rs:117,157 and gui/smoke.rs:18 cover worker-owned GUI; frontends/smoke.rs:35 runs before optional-key skip. Three reader unit tests, six core capability_delivery cases and one real GUI-render case pass. Invalid/missing-file process cases assert safe Notice and completed turn.'
  - criterion: MCP §5 compares resolved values to the captured capability and preserves only the launcher exception
    reference: 'src/redact.rs:48,70 and src/mcp.rs:578-581 replace raw-env comparison with captured validated value. tests/capability_delivery.rs:390: profile and ACP !cmd resolving the exact private capability are refused before MCP process startup, no report file, Notice and continuing turn. tests/support/seat_launch.rs asserts profile/ACP echo_env strips BBOXY_* while launcher MCP retains BBOXY_CAPABILITY_FILE and has no BBOXY_CAPABILITY. Resolver-namespace and launcher-inherit compatibility controls each 101 -> 0; target/side-b-compat-controls/controls.json.'
  - criterion: Real-process ICD cases (a)-(d), namespace stripping and rendered GUI redaction
    reference: src/process.rs:477 removes every BBOXY_* for tools/terminals/command resolvers. tests/capability_delivery.rs:292,298,305 execute printenv and cat "$BBOXY_CAPABILITY_FILE" (no namespace/path/value), then cat literal path (only <redacted>), inspecting actual ToolCall output, stdout/stderr, transcript laws and every session file/blob. GUI test at :431 runs two CLI headless renders and requires identical PNGs for file-value versus explicitly redacted fixture, with clean stdout/stderr. Profile/ACP echo_env and exact resolved-value witnesses cover (c)/(d).
  - criterion: Canonical legacy-reader broken control fails at the leak assertion and restores green
    reference: 'scripts/test.py:46 runs scripts/test-capability-control.py within make test; hardening asserts this command remains in the stack. target/capability-control/receipt.json: red_exit=101 at CapabilityFileValueIsRedacted, green_exit=0, restored src/capability.rs SHA256 0f449d5b018021295fcb977c271512ccc2dada75721b160770e50e1143cb868f. Source hash verified unchanged after the full gate.'
  - criterion: Updated ICD-named model, meaningful broken control and native/TLC parity
    reference: HarnessMcpServer now models file availability, face preparation, registration and output projection; ReadyAfterCapabilityAttempt/NoUnredactedCapability plus nonvacuity witnesses. BrokenHarnessMcpServerD violates named NoUnredactedCapability (TLC12); good passes TLC0; each 362 states/1145 edges, native/TLC graphs equal. All 68 variants/1 crate/25 contracts pass parity. Committed code-head summary .bboxy/runs/2026-11-01-capability-side-b-formal.json SHA256 1038a06c0c9ba8aa18ed216bd36b2f356e4af247c14aabf294724dbbc4ca2fdb. Final-head make test-formal-parity and make check-formal-evidence both 0; local source-clean receipt/archive verification/evidence/29790870f23922fae2d530e75602055370dae35a2af66e260d892e933a98d505/evidence.json binds b773607. coverage.toml:86-88 explicitly separates syscall/byte/environment/UI implementation witnesses; no arbitrary Rust refinement claim.
  - criterion: Full unattended canonical make test exits 0, including GUI smoke and no added wall-clock gates
    reference: '.bboxy/runs/test-current.json PASS source b2c078c: all17 core commands exit0, including both feature suites, separately executed existing seat/PTY isolation, hardening, canonical leak control, native/TLC, secret/header/KB/inventory checks. .bboxy/runs/gui-gate.json: all4 GUI commands exit0, including real-render test and make test-gui-smoke. Full log target/side-b-evidence/make-test.log SHA256 10b9f5208f43b4899d8445dedf4c3ab619ec25ce9226b1aaaded6eab8cf791bb. Test launch removes this real seat''s two capability env names; no test filter/concurrency override supplied to make test. Final-head gate source hashes equal the recorded full-gate inputs.'
  - criterion: Signed pushed branch, open PR, committed report and headless screenshots
    reference: 'PR https://github.com/bboxy450/bboxy450-harness/pull/18 OPEN on capability-file-side-b; remote head b7736074c41028fd10bf12536c9dae646fde50c2. Source b2c078c and evidence b773607 both G-signed by the required key. Report https://github.com/bboxy450/bboxy450-harness/blob/b7736074c41028fd10bf12536c9dae646fde50c2/.bboxy/runs/2026-11-01-capability-side-b-report.log. Refreshed .bboxy/runs/gui-smoke.png and gui-smoke-scrolled.png: 22 items/15 kinds, approval and structural/scrolled-viewport checks pass with no operator/display interaction. Tracked files clean; complete formal graph archives are local untracked evidence, accurately distinguished from the committed summary.'
  exit_code: 0
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

## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Implemented Side B on capability-file-side-b and opened PR #18: https://github.com/bboxy450/bboxy450-harness/pull/18 (OPEN, remote head verified b7736074c41028fd10bf12536c9dae646fde50c2). Both commits are signed G by 9B07E849480C68328CA20684844ECC0587DE7AB0. Full unattended make test exited 0 on source b2c078c414a409d033c58431aef022595645cbb2; final commit adds only .bboxy/runs evidence and has identical gate input hashes. Final-head native/TLC parity and check-formal-evidence also exited 0 (68 variants, source-clean). Committed report: .bboxy/runs/2026-11-01-capability-side-b-report.log, SHA256 09560d4e53c5b1c7069788ee5172017c516de3bfb617da563df1aa9e6058d80f; artifact claim 01M6CF2CS6SY6EJ4T5Y4GH60N1 accepted. No merge. Worktree tracked files clean; verifier graph archives remain untracked under verification/evidence/.

```json
{
  "commit": "b7736074c41028fd10bf12536c9dae646fde50c2",
  "op_id": "01M6CF3GWKN4Q87WYP0TF5HWHN",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Validated file loading in every face before providers, tools and MCP; typed path/value-free refusals and degraded continuation",
        "reference": "src/capability.rs:21,57,73,99: O_NOFOLLOW|O_NONBLOCK, metadata plus descriptor fstat, regular/euid/exact0600 and bounded 64 lowercase hex+newline; Zeroizing storage. headless.rs:132 covers headless/seat/ACP; gui/bridge.rs:117,157 and gui/smoke.rs:18 cover worker-owned GUI; frontends/smoke.rs:35 runs before optional-key skip. Three reader unit tests, six core capability_delivery cases and one real GUI-render case pass. Invalid/missing-file process cases assert safe Notice and completed turn."
      },
      {
        "criterion": "MCP §5 compares resolved values to the captured capability and preserves only the launcher exception",
        "reference": "src/redact.rs:48,70 and src/mcp.rs:578-581 replace raw-env comparison with captured validated value. tests/capability_delivery.rs:390: profile and ACP !cmd resolving the exact private capability are refused before MCP process startup, no report file, Notice and continuing turn. tests/support/seat_launch.rs asserts profile/ACP echo_env strips BBOXY_* while launcher MCP retains BBOXY_CAPABILITY_FILE and has no BBOXY_CAPABILITY. Resolver-namespace and launcher-inherit compatibility controls each 101 -> 0; target/side-b-compat-controls/controls.json."
      },
      {
        "criterion": "Real-process ICD cases (a)-(d), namespace stripping and rendered GUI redaction",
        "reference": "src/process.rs:477 removes every BBOXY_* for tools/terminals/command resolvers. tests/capability_delivery.rs:292,298,305 execute printenv and cat \"$BBOXY_CAPABILITY_FILE\" (no namespace/path/value), then cat literal path (only <redacted>), inspecting actual ToolCall output, stdout/stderr, transcript laws and every session file/blob. GUI test at :431 runs two CLI headless renders and requires identical PNGs for file-value versus explicitly redacted fixture, with clean stdout/stderr. Profile/ACP echo_env and exact resolved-value witnesses cover (c)/(d)."
      },
      {
        "criterion": "Canonical legacy-reader broken control fails at the leak assertion and restores green",
        "reference": "scripts/test.py:46 runs scripts/test-capability-control.py within make test; hardening asserts this command remains in the stack. target/capability-control/receipt.json: red_exit=101 at CapabilityFileValueIsRedacted, green_exit=0, restored src/capability.rs SHA256 0f449d5b018021295fcb977c271512ccc2dada75721b160770e50e1143cb868f. Source hash verified unchanged after the full gate."
      },
      {
        "criterion": "Updated ICD-named model, meaningful broken control and native/TLC parity",
        "reference": "HarnessMcpServer now models file availability, face preparation, registration and output projection; ReadyAfterCapabilityAttempt/NoUnredactedCapability plus nonvacuity witnesses. BrokenHarnessMcpServerD violates named NoUnredactedCapability (TLC12); good passes TLC0; each 362 states/1145 edges, native/TLC graphs equal. All 68 variants/1 crate/25 contracts pass parity. Committed code-head summary .bboxy/runs/2026-11-01-capability-side-b-formal.json SHA256 1038a06c0c9ba8aa18ed216bd36b2f356e4af247c14aabf294724dbbc4ca2fdb. Final-head make test-formal-parity and make check-formal-evidence both 0; local source-clean receipt/archive verification/evidence/29790870f23922fae2d530e75602055370dae35a2af66e260d892e933a98d505/evidence.json binds b773607. coverage.toml:86-88 explicitly separates syscall/byte/environment/UI implementation witnesses; no arbitrary Rust refinement claim."
      },
      {
        "criterion": "Full unattended canonical make test exits 0, including GUI smoke and no added wall-clock gates",
        "reference": ".bboxy/runs/test-current.json PASS source b2c078c: all17 core commands exit0, including both feature suites, separately executed existing seat/PTY isolation, hardening, canonical leak control, native/TLC, secret/header/KB/inventory checks. .bboxy/runs/gui-gate.json: all4 GUI commands exit0, including real-render test and make test-gui-smoke. Full log target/side-b-evidence/make-test.log SHA256 10b9f5208f43b4899d8445dedf4c3ab619ec25ce9226b1aaaded6eab8cf791bb. Test launch removes this real seat's two capability env names; no test filter/concurrency override supplied to make test. Final-head gate source hashes equal the recorded full-gate inputs."
      },
      {
        "criterion": "Signed pushed branch, open PR, committed report and headless screenshots",
        "reference": "PR https://github.com/bboxy450/bboxy450-harness/pull/18 OPEN on capability-file-side-b; remote head b7736074c41028fd10bf12536c9dae646fde50c2. Source b2c078c and evidence b773607 both G-signed by the required key. Report https://github.com/bboxy450/bboxy450-harness/blob/b7736074c41028fd10bf12536c9dae646fde50c2/.bboxy/runs/2026-11-01-capability-side-b-report.log. Refreshed .bboxy/runs/gui-smoke.png and gui-smoke-scrolled.png: 22 items/15 kinds, approval and structural/scrolled-viewport checks pass with no operator/display interaction. Tracked files clean; complete formal graph archives are local untracked evidence, accurately distinguished from the committed summary."
      }
    ],
    "exit_code": 0
  }
}
```
