---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
artifacts:
- '[[01M6CGKX3S72MGAQ94Z3D9F1WK]]'
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T21:11:35.908Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CF85M6MAJT9NMJEW79VK5Y
number: 4921
provenance:
  digest: sha256:835ffa0ba858bd3a09fb6c14504c189908327c58b806380914d648974d6cd2cf
  frontmatter_digest: sha256:91a82283d7e4cb5a830d72c6e0962c1b8a7622f1310204279e78a4fb471a3cf8
  log: .bboxy/log/2026-11-01.jsonl#31971
  sig: pjSp2HAm/v8FbhynkQpgxVJLZKdw6SoVbA/NQh7zc1izdNcIARnS2Q8BgPjt9La3EyRuhC8yi4+MwdH368w7Bg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: ea468352b348f2ea629e9479a2115ec36d96ae7d
  op_id: 01M6CGNB2124VA461D4WR51PHA
  reported_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
  summary: 'Completed #4921 on capability-side-b-followups from origin/main 2ce1917 after PR #18 merge. PR #20 is OPEN and MERGEABLE: https://github.com/bboxy450/bboxy450-harness/pull/20; remote head verified ea468352b348f2ea629e9479a2115ec36d96ae7d. Both new commits are G-signed by fingerprint 9B07E849480C68328CA20684844ECC0587DE7AB0. Full make test exited 0 unattended on source 1201697f31120f7e5324017b8fb9f88ff3069d0c (17 core +4 GUI commands); evidence-only final head has identical gate input hashes. All eight canonical controls go 101 ->0. Final-head formal parity and check-formal-evidence exit0, 69 variants. Committed report .bboxy/runs/2026-11-01-capability-side-b-followups-report.log, SHA256 ccf4ba6ae3e4afe05739eec049adaaff33fa709d805897d6847b9a023ac5330f (artifact 01M6CGKX3S72MGAQ94Z3D9F1WK). ICD §4 harness row should cite tests/capability_delivery.rs for the new proofs and refresh pending labels; Architect owns that edit. Tracked worktree clean; verifier archives are local untracked evidence. No merge.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3f: Side B follow-ups (live GUI witness, per-face controls, case (c), quiet when unset)'
type: task
updated: 2026-11-01T21:47:42.914Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: 1. Live GUI registration is witnessed immediately before Host startup with a real headless bridge and a deleting-loader control
    reference: src/frontends/gui/bridge.rs:161-188 has a test-fixtures-only synchronous observer at the actual Bridge::create worker boundary immediately before Host::new. tests/capability_delivery.rs:588 runs that real live bridge in a private child process, captures Set::scrub(private file value)==<redacted> at the boundary, asserts GuiLiveCapabilityRegisteredBeforeHost, waits for ready and closes. Deleting only the live registration causes exit101 at that named assertion; restoring passes0. No window or shared-process env mutation.
  - criterion: 2. Per-face broken controls fail at named assertions and restore green inside the canonical stack
    reference: '.bboxy/runs/2026-11-01-capability-side-b-followups-controls.json (SHA256 b901acab3714a9fd81b33565403354fe41c090ebb51cf1ce3981322dbfdabb25) records all8: legacy-reader, headless, seat, ACP, Bridge replay, Bridge live, GUI smoke, optional live smoke. Each exact real test has red101 at its named assertion, exact source-byte restoration/hash and green0; generic child wrapper has a distinct assertion. Headless/seat/ACP inspect actual session log/blob bytes as well as wire output. scripts/test.py invokes the extended script in make test; GUI gate enables test-fixtures to run the live witness normally. Local complete logs target/capability-control/<case>-{red,green}.log.'
  - criterion: 3. ICD case (c) explicitly checks the file variable in profile/ACP/launcher MCP servers and tools
    reference: tests/capability_delivery.rs:407 starts actual echo_env MCP processes for all3 origins. tests/bin/mcp_fixture.rs reports capability_file_present without values. Profile and ACP require BBOXY_CAPABILITY_FILE absent, even when the reserved configured key is attempted; launcher --mcp-config requires it present. All MCP processes have legacy variable absent. Each origin also runs a real tool whose printenv and file-variable cat/printenv probes are empty; literal-file output is redacted and storage checked. Canonical ACP/fixture suite passes this case.
  - criterion: 4. Unset configuration is silent; supplied invalid files emit typed Notices and continue
    reference: src/capability.rs:99 and src/redact.rs:63 distinguish unset initialization from refusal. tests/capability_delivery.rs:321 asserts quiet unset headless startup with completed turn, and set missing/symlink/mode/content/type cases emit safe typed Notices and complete. GUI Bridge replay test :537 covers valid, unset and supplied-invalid cases, requiring a Notice only for invalid input and redaction for valid input. HarnessMcpServer adds unset/valid/invalid input classes, notice_emitted and QuietWhenUnset; BrokenHarnessMcpServerE emits an incorrect Notice and violates that property.
  - criterion: 5. Report names the ICD §4 harness citation and leaves the Architect-owned interface edit untouched
    reference: Committed .bboxy/runs/2026-11-01-capability-side-b-followups-report.log section5 explicitly says the ICD §4 harness row should cite tests/capability_delivery.rs for case(c), live pre-Host/replay witnesses and per-face control evidence. The base ICD already cites this file for PR18 but has pending labels; the report requests their update from the Architect. No ICD changes in this PR.
  - criterion: Unattended full make test, headless smoke and native/TLC parity all pass
    reference: '.bboxy/runs/test-current.json PASS source1201697: all17 core commands0; .bboxy/runs/gui-gate.json all4 GUI commands0. Full log target/followups-make-test.log SHA256 2c6235ce95f322771afa9d5217b491b512db947a30bcd6bf73b1e42c89864f74. No test filter/concurrency override passed to make; existing separately executed seat/PTY isolation unchanged. Smoke: 22items/15kinds, 3approval buttons, structural/semantic/scrolled checks and refreshed screenshots, no operator/display input. Committed code-head formal summary SHA256 e37d6dfaecf8a4c5accf4b85781b53490bce7d8aeb5f47830965cf2c54644d59; all69 native/TLC graphs and verdicts match. Good MCP534states/1701edges TLC0, D/E same graph sizes TLC12 at NoUnredactedCapability/QuietWhenUnset. Nonvacuity reaches valid/degraded/silent states. No wall-clock gate added.'
  - criterion: Signed pushed final head, current-source formal receipt, committed report and honest worktree state
    reference: PR20 OPEN/MERGEABLE head ea468352b348f2ea629e9479a2115ec36d96ae7d; source1201697 and report ea46835 both G-signed with required fingerprint. Final-head make test-formal-parity and make check-formal-evidence exit0; source-clean69-model receipt target/followups-final-head-formal-receipt.log, full local archive verification/evidence/69bd2e61d38444f2e8e32b93acd0afc9cc79574a2aeba0554a242e4ab1e896e8/evidence.json. Gate input hashes match the full tested source. Report https://github.com/bboxy450/bboxy450-harness/blob/ea468352b348f2ea629e9479a2115ec36d96ae7d/.bboxy/runs/2026-11-01-capability-side-b-followups-report.log; forbidden hex-label regex checked absent. Tracked files clean, only untracked verifier graph archives remain. No merge or publication.
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main, after PR #18 has merged, and open a PR. Do not merge. These come from the Oracle's security review of PR #18; no leak was found.

1. **Live GUI face witness.** Nothing tests live-GUI registration at gui/bridge.rs:157; deleting that line would go undetected. Add a headless test that starts the live GUI bridge with a fixture capability file and asserts the redaction set contains the value before Host starts. Add a broken control that deletes the registration and must go red.
2. **Per-face broken controls.** scripts/test-capability-control.py covers only the headless face. Extend it so that, for each face (headless, seat, ACP, GUI replay, GUI live, smoke), reverting that face's loader goes red at its named assertion.
3. **Case (c), file-path-specific.** Add an echo_env test asserting that profile and ACP MCP servers and tools never see `BBOXY_CAPABILITY_FILE`, while launcher `--mcp-config` servers do.
4. **Quiet when unset (Oracle ruling).** When `BBOXY_CAPABILITY_FILE` is unset (a non-seat run such as `-p` or the GUI), emit no Notice. Emit the typed Notice only when the variable is set and the file fails validation. Test both cases. I'm asking the Architect to align the ICD.
5. **ICD citation.** In your report, say the ICD §4 harness row should cite tests/capability_delivery.rs. The Architect edits the ICD.

Acceptance:
- `make test` exits 0 unattended.
- The controls go 101 → 0.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-capability-side-b-followups-report.log`. Don't write "key:" directly before any hex value.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Completed #4921 on capability-side-b-followups from origin/main 2ce1917 after PR #18 merge. PR #20 is OPEN and MERGEABLE: https://github.com/bboxy450/bboxy450-harness/pull/20; remote head verified ea468352b348f2ea629e9479a2115ec36d96ae7d. Both new commits are G-signed by fingerprint 9B07E849480C68328CA20684844ECC0587DE7AB0. Full make test exited 0 unattended on source 1201697f31120f7e5324017b8fb9f88ff3069d0c (17 core +4 GUI commands); evidence-only final head has identical gate input hashes. All eight canonical controls go 101 ->0. Final-head formal parity and check-formal-evidence exit0, 69 variants. Committed report .bboxy/runs/2026-11-01-capability-side-b-followups-report.log, SHA256 ccf4ba6ae3e4afe05739eec049adaaff33fa709d805897d6847b9a023ac5330f (artifact 01M6CGKX3S72MGAQ94Z3D9F1WK). ICD §4 harness row should cite tests/capability_delivery.rs for the new proofs and refresh pending labels; Architect owns that edit. Tracked worktree clean; verifier archives are local untracked evidence. No merge.

```json
{
  "commit": "ea468352b348f2ea629e9479a2115ec36d96ae7d",
  "op_id": "01M6CGNB2124VA461D4WR51PHA",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1. Live GUI registration is witnessed immediately before Host startup with a real headless bridge and a deleting-loader control",
        "reference": "src/frontends/gui/bridge.rs:161-188 has a test-fixtures-only synchronous observer at the actual Bridge::create worker boundary immediately before Host::new. tests/capability_delivery.rs:588 runs that real live bridge in a private child process, captures Set::scrub(private file value)==<redacted> at the boundary, asserts GuiLiveCapabilityRegisteredBeforeHost, waits for ready and closes. Deleting only the live registration causes exit101 at that named assertion; restoring passes0. No window or shared-process env mutation."
      },
      {
        "criterion": "2. Per-face broken controls fail at named assertions and restore green inside the canonical stack",
        "reference": ".bboxy/runs/2026-11-01-capability-side-b-followups-controls.json (SHA256 b901acab3714a9fd81b33565403354fe41c090ebb51cf1ce3981322dbfdabb25) records all8: legacy-reader, headless, seat, ACP, Bridge replay, Bridge live, GUI smoke, optional live smoke. Each exact real test has red101 at its named assertion, exact source-byte restoration/hash and green0; generic child wrapper has a distinct assertion. Headless/seat/ACP inspect actual session log/blob bytes as well as wire output. scripts/test.py invokes the extended script in make test; GUI gate enables test-fixtures to run the live witness normally. Local complete logs target/capability-control/<case>-{red,green}.log."
      },
      {
        "criterion": "3. ICD case (c) explicitly checks the file variable in profile/ACP/launcher MCP servers and tools",
        "reference": "tests/capability_delivery.rs:407 starts actual echo_env MCP processes for all3 origins. tests/bin/mcp_fixture.rs reports capability_file_present without values. Profile and ACP require BBOXY_CAPABILITY_FILE absent, even when the reserved configured key is attempted; launcher --mcp-config requires it present. All MCP processes have legacy variable absent. Each origin also runs a real tool whose printenv and file-variable cat/printenv probes are empty; literal-file output is redacted and storage checked. Canonical ACP/fixture suite passes this case."
      },
      {
        "criterion": "4. Unset configuration is silent; supplied invalid files emit typed Notices and continue",
        "reference": "src/capability.rs:99 and src/redact.rs:63 distinguish unset initialization from refusal. tests/capability_delivery.rs:321 asserts quiet unset headless startup with completed turn, and set missing/symlink/mode/content/type cases emit safe typed Notices and complete. GUI Bridge replay test :537 covers valid, unset and supplied-invalid cases, requiring a Notice only for invalid input and redaction for valid input. HarnessMcpServer adds unset/valid/invalid input classes, notice_emitted and QuietWhenUnset; BrokenHarnessMcpServerE emits an incorrect Notice and violates that property."
      },
      {
        "criterion": "5. Report names the ICD §4 harness citation and leaves the Architect-owned interface edit untouched",
        "reference": "Committed .bboxy/runs/2026-11-01-capability-side-b-followups-report.log section5 explicitly says the ICD §4 harness row should cite tests/capability_delivery.rs for case(c), live pre-Host/replay witnesses and per-face control evidence. The base ICD already cites this file for PR18 but has pending labels; the report requests their update from the Architect. No ICD changes in this PR."
      },
      {
        "criterion": "Unattended full make test, headless smoke and native/TLC parity all pass",
        "reference": ".bboxy/runs/test-current.json PASS source1201697: all17 core commands0; .bboxy/runs/gui-gate.json all4 GUI commands0. Full log target/followups-make-test.log SHA256 2c6235ce95f322771afa9d5217b491b512db947a30bcd6bf73b1e42c89864f74. No test filter/concurrency override passed to make; existing separately executed seat/PTY isolation unchanged. Smoke: 22items/15kinds, 3approval buttons, structural/semantic/scrolled checks and refreshed screenshots, no operator/display input. Committed code-head formal summary SHA256 e37d6dfaecf8a4c5accf4b85781b53490bce7d8aeb5f47830965cf2c54644d59; all69 native/TLC graphs and verdicts match. Good MCP534states/1701edges TLC0, D/E same graph sizes TLC12 at NoUnredactedCapability/QuietWhenUnset. Nonvacuity reaches valid/degraded/silent states. No wall-clock gate added."
      },
      {
        "criterion": "Signed pushed final head, current-source formal receipt, committed report and honest worktree state",
        "reference": "PR20 OPEN/MERGEABLE head ea468352b348f2ea629e9479a2115ec36d96ae7d; source1201697 and report ea46835 both G-signed with required fingerprint. Final-head make test-formal-parity and make check-formal-evidence exit0; source-clean69-model receipt target/followups-final-head-formal-receipt.log, full local archive verification/evidence/69bd2e61d38444f2e8e32b93acd0afc9cc79574a2aeba0554a242e4ab1e896e8/evidence.json. Gate input hashes match the full tested source. Report https://github.com/bboxy450/bboxy450-harness/blob/ea468352b348f2ea629e9479a2115ec36d96ae7d/.bboxy/runs/2026-11-01-capability-side-b-followups-report.log; forbidden hex-label regex checked absent. Tracked files clean, only untracked verifier graph archives remain. No merge or publication."
      }
    ],
    "exit_code": 0
  }
}
```
