---
allowed_paths:
- src/
- tests/
- Makefile
- scripts/
- verification/
- Cargo.toml
- Cargo.lock
- protocol.schema.json
- .bboxy/runs/
artifacts:
- '[[01M6B4VRVDQ78D0RTR1293ECB7]]'
- '[[01M6B4VRVDQ78D0RTR1293ECB7]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: u0vQJ7ngwMS5yr2jtfd3HrkhSWQXq2tpEWFJBh5W+4A=
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
created: 2026-11-01T06:15:00.321Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6AVYEXPZB5SF1JCQ5V3A2S3
number: 1992
provenance:
  digest: sha256:b337799a2ce9c860091907c3656d41906ad34c1df98a9d73170966feccfcf83f
  frontmatter_digest: sha256:9546e9c0891ce30fa5c21bb918c0aca45d588c5728b1f972cce799712b9392bc
  log: .bboxy/log/2026-11-01.jsonl#19239
  sig: 2alWVti/bOKN1va+9vLvkq7PCs6+a7vF8BcabbqU2cpq2WMIxh4Pfrl6YVjNyjlPYe4gOaMXMQU4lgSCQ7ijBQ==
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    PR #6 (4834a35) is rejected for one focused round. Verified good, keep as is: MCP policy gating (readOnlyHint never bypasses decide; the liar test), timeouts, generation guard, the auto-compaction trigger, Usage voiding and frontier law, the 7 formal controls (stated over state), the acp-1.12 diff, and the d2400 GUI hunks (E3 approved them). Rebase onto main first; main is now past cc7cab1.

    MUST
    1. **BLOCKER: ACP backpressure tears down the connection.** At `acp.rs:56-65,748`, `emit()` uses a 256-entry queue with `try_send`; when the queue is full it cancels, and the connection closes. A slow client during a streaming turn therefore kills the session. acp.md §5 allows teardown only on EOF or a broken stdout.
       - Fix: output must never cause teardown. Either wait (bounded) on the writer task only, never on the reader or engine path, or coalesce updates.
       - Add a test with a client that stops reading for a while, then resumes. The turn must complete, and `check_transcript` must hold.
    2. **ACP per-session ordering.** At `acp.rs:784`, every line is spawned as its own task, so a `session/cancel` can overtake its `session/prompt`: the Interrupt reaches no turn, and the prompt then runs to completion. Requests for one session must be processed in order. Add a test that sends prompt and then cancel back to back with no wait, and assert `TurnEnded{Cancelled}`.
    3. **`tool_call_update` overwrites earlier content.** At `acp.rs:618` vs `:621/627`, the completion update replaces the diff and terminal content. Merge the content instead of replacing it, per acp.md. Fix the vacuous assertion at `acp_face.rs:173`. Stop sorting in `canonical()` (`acp_face.rs:341`), so the golden comparison checks order.
    4. **Finish d2019.** Fix the pidfile waits at `store_recovery.rs:198/219-233` and `profile_lifecycle.rs:16/25-33` with the same temp-file-and-rename pattern. Attach the logs: 200 consecutive passing runs each of `seat_process`, `store_recovery` and `profile_lifecycle`.
    5. **The MCP server group kill is untested.** Deleting `killpg` (`mcp.rs:394-408`) keeps all tests green. Add a fixture server that ignores stdin EOF and forks a group member, plus a control with `killpg` removed that must fail.
    6. **The cache-prefix test cannot fail.** At `context_auto.rs:166-175`, assert that between reduction items the prefix is byte-identical and longer than the system prompt. Also add a test where trimming alone fits and no summary follows (the `auto::fits` early return).
    7. **MCP close margin.** At `mcp_client.rs:269/286`, the deadline is 2 s, but `finish_child` sleeps 1 s. Gate on observed group exit (ESRCH polling with a generous hang guard), not on a 2 s wall clock.

    SHOULD: do the cheap ones; list the rest as gaps
    - ICD §6 witnesses: ACP MaxTokens, MaxRounds and Error stop reasons; permission `cancelled`; disconnect while a permission is pending; late response after Interrupt (assert something); EOF mid-turn followed by `check_transcript`. MCP: Interrupt during a slow call; a stale-generation response is dropped; the crash Notice is asserted; name truncation and collision; stderr excerpt scrubbing; a secret grep over blobs and the session directory.
    - `acp.rs:277-280`: on a cwd failure, close the started session.
    - `mcp.rs:230-234`: a stale call's reason. `turn_loop.rs:840`: a Deny must carry the rule's reason.
    - The HarnessAutoCompact model is too narrow: `ended_through` is never advanced, and a trim is never enough on its own. Widen it so the trim-only path and the summary-without-trim path are both explored.
    - `seat_process.rs:71-75` no longer exercises the PTY-master-close path. Restore it, or record the gap.

    Verify: `make test` exits 0, unattended. Show red→green for 1, 2, 3, 5 and 6. Run `make smoke-live` once and record the real outcome. Report via `task_report`.
report:
  commit: f3aa965abc5a1e04a74c601e1dc3e76dfee0a801
  op_id: 01M6B6E0ME41T5T1E2BC0JJJ56
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: 'E1c item 7 / d2019 completed on the same e1c-acp-mcp-compaction branch. PR #6 remains OPEN at signed head f3aa965abc5a1e04a74c601e1dc3e76dfee0a801; no merge. The seat PID producer was already atomic in 52dd070. Required 200 consecutive cargo test --test seat_process runs now exit 0: 600 passed, 0 failed, 0 ignored; exact command, log, source and binary hashes are committed. The audit found and fixed the profile, recovery and escaped ACP-child PID producers in ca59d8e; existing parse polling and lifecycle fences are documented for the remaining readers. CARGO_BUILD_JOBS=2 make test exited 0 unattended on ca59d8e. The final records-only head changes no verification inputs. Final-head native/TLC parity and make check-formal-evidence each exit 0, 52 models, receipt e98035daa2af332e2609f7b1e43f465cc50c4cba962a995b64dc0b1efd3cb1d7. Full report updated in place: Report: [[e1c-report]] (#3022), item 7 section. Product code is unchanged; prior observed headless/ACP live PASS remains in the report, with no extra live requests. E3 review of the previously listed GUI hunks remains required before merge.'
role: executor
status: reported
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1c: ACP face, MCP client, automatic compaction (protocol v0.3)'
type: task
updated: 2026-11-01T09:18:16.251Z
updated_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
verify: make test
verify_result:
  command: CARGO_BUILD_JOBS=2 make test
  evidence:
  - criterion: 200 consecutive complete seat_process runs, all three tests each time, no skips or failures
    reference: .bboxy/runs/e1c/seat-200.json; .bboxy/runs/e1c/seat-200.log — for i in $(seq 200); do CARGO_BUILD_JOBS=2 cargo test --test seat_process || exit 1; done exited 0; 200 success summaries, 600 passed; seat source matches the committed atomic fix
  - criterion: Atomic seat PID publication from d2019
    reference: tests/seat_process.rs:130; signed fix 52dd070; .bboxy/runs/e1c/make-test-initial.log retains the original ParseIntError Empty
  - criterion: Audit and repair other PID-file waits
    reference: .bboxy/runs/e1c/pidfile-audit.json; tests/profile_lifecycle.rs:16; tests/store_recovery.rs:198; tests/bin/mcp_fixture.rs:29 (atomic write/rename); tests/cancel_process.rs:11/:71/:76 and tests/mcp_client.rs:281/:569 retain documented parse/lifecycle fences
  - criterion: Full unattended canonical gate with affected fixtures, models and GUI
    reference: .bboxy/runs/e1c/item7-make-test.log; .bboxy/runs/test-current.json; .bboxy/runs/gui-gate.json — all layer exits 0 on ca59d8e; .bboxy/runs/e1c/item7-formal-code.json
  - criterion: Final signed head, source-clean current-head formal parity and strict receipt
    reference: f3aa965abc5a1e04a74c601e1dc3e76dfee0a801; git verify-commit HEAD exit 0, Boxy key 9B07E849480C68328CA20684844ECC0587DE7AB0; make test-formal-parity exit 0; make check-formal-evidence exit 0; verification/evidence/e98035daa2af332e2609f7b1e43f465cc50c4cba962a995b64dc0b1efd3cb1d7/evidence.json
  - criterion: Committed report and PR handoff
    reference: '.bboxy/runs/e1c-report.md:174, note #3022; https://github.com/bboxy450/bboxy450-harness/pull/6 — OPEN at f3aa965; push and PR description update exit 0'
  exit_code: 0
---
# E1c: ACP face, MCP client and automatic compaction

ROLE: EXECUTOR (Codex).

**Worktree:** `/Users/feral/mydev/bboxy450-harness-core`.
1. Fetch.
2. Create branch `e1c-acp-mcp-compaction` from `origin/main` at or after `c745ad9`.
3. Push the branch and open a PR. Do not merge.

E3 is rebasing the GUI (`src/frontends/gui/`) in parallel. Do not touch that directory.

## Binding specs
The ICD wins over this brief.
- `.bboxy/interfaces/acp.md` (new)
- `.bboxy/interfaces/mcp.md` (new)
- `.bboxy/interfaces/protocol.md` §4d (v0.3)
- `.bboxy/interfaces/gateway.md` §1b
- `.bboxy/interfaces/interfaces.md` rows R22–R24, plus models HarnessAutoCompact and HarnessMcpServer
- `.bboxy/specs/budgets.md`
- the invariants

## Outcome
1. **ACP pin check first.** The ACP facts in `acp.md` were read from schema 1.9.1, but the crate we pin (3.4.0) uses schema 1.12.0.
   - Before coding, check every fact `acp.md` relies on against 1.12.0.
   - Write the differences to `.bboxy/runs/acp-1.12-diff.md`.
   - Message the Architect only if a difference changes the mapping. Otherwise proceed.
2. **ACP face** (feature `acp`): `bboxy450-harness acp` over stdio, per `acp.md`.
   - Map items to `session/update`.
   - Send approvals as `request_permission`.
   - Implement `session/load` through the store.
   - Unsupported methods degrade as the ICD specifies.
   - **Witness:** a scripted ACP client test drives a full turn against the mock provider, including tool, diff, terminal, plan and approval. `check_transcript` must hold.
3. **MCP client**, per `mcp.md`.
   - Servers live as long as the session.
   - Tools enter the registry, with their concurrency class taken from `readOnlyHint`.
   - Every call goes through `policy.decide`.
   - Each server has a timeout.
   - A dead or slow server produces a labelled Notice and never blocks a turn.
   - Secrets go through the redactor.
   - **Witness:** a fixture MCP stdio server, as a test binary. Cover: a read-only call running in parallel, a write call that needs approval, the server crashing mid-call, and the server hanging past its timeout.
4. **Automatic compaction**, per protocol §4d: the trigger, the ladder (clip, then drop, then summarize), and a cache-stable prefix. This is protocol v0.3; regenerate the schema.
5. **Formal models.** HarnessAutoCompact and HarnessMcpServer, each with the broken control named in `interfaces.md`.
   - The invariants must be stated over state, not over a flag that only broken actions set.
   - Previous reviews found exactly that failure mode. Reviewers will check for it.
6. **Live smoke.** Extend `make smoke-live` with one ACP-driven turn.
   - Use the same nemotron free profile.
   - At most 3 more requests.
   - Report the outcome as SKIP, UNAVAILABLE or PASS, honestly.

## Verify
- `make test` runs unattended and exits 0.
- No wall-clock gates.
- Show the red→green progression for every model control and for the MCP failure witnesses.

## Report
Send one `task_report`. Include files and mechanisms, commands with exit codes, and the branch with its signed head.


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1c implemented and pushed on e1c-acp-mcp-compaction. PR #6 OPEN: https://github.com/bboxy450/bboxy450-harness/pull/6 ; no merge. Signed head 4834a3564dba706373f1d9b78c9b65d226719603 (Boxy, key 9B07E849480C68328CA20684844ECC0587DE7AB0). Full committed report: Report: [[e1c-report]] (#3022), .bboxy/runs/e1c-report.md. make test exited 0 unattended on a757963; the final records-only commit changes no verification inputs. Final-head native/TLC parity and strict receipt each exited 0: 52 models, 21 contracts, one crate, receipt 798cd045cde97946193ed86e0c61f5ccadef3dfd2c35b918ce0eb0d38de24192. All seven new model controls fail the named properties; five production process controls each show 101→0. One make smoke-live invocation exited 0: PASS for headless and ACP, exactly three requests each, one read/one edit/file changed/transcript laws held. Source tree clean; refreshed formal-native receipt and local proof/log archives are run outputs. E3 review of the three listed GUI display locations and derived smoke expectations is required by d2400 before merge. Initial gate failures and their fixes are retained in the report.

```json
{
  "commit": "4834a3564dba706373f1d9b78c9b65d226719603",
  "op_id": "01M6B5NDHD98H9PMN02WEFAS7H",
  "verify_result": {
    "command": "CARGO_BUILD_JOBS=2 make test",
    "evidence": [
      {
        "criterion": "1 ACP pin checked against schema 1.12.0 before coding; no unstable features",
        "reference": ".bboxy/runs/acp-1.12-diff.md; signed pre-code commit ed99393; Cargo.toml/Cargo.lock"
      },
      {
        "criterion": "2 ACP stdio face, independent sessions, permission provenance, load replay, full tool/diff/terminal and Plan witness, transcript laws",
        "reference": "src/frontends/acp.rs; src/gateway/host.rs; tests/acp_face.rs (8 passing); tests/fixtures/acp/*.jsonl; forced-drop repair test in src/frontends/acp.rs; .bboxy/runs/e1c/make-test.log"
      },
      {
        "criterion": "3 Session-lived MCP clients, concurrent reads, policy/trust separation, timeout/crash/restart/re-list shutdown and secret scrubbing",
        "reference": "src/mcp.rs; tests/bin/mcp_fixture.rs; tests/mcp_client.rs (11 passing); .bboxy/runs/e1c/process-controls.json and adjacent red/green logs (five controls 101→0)"
      },
      {
        "criterion": "4 Protocol 0.4 automatic compaction, exact wire accounting, cache prefix, durable recovery and safe queued-cancellation frontier",
        "reference": "src/context/auto.rs; src/context/mod.rs; src/engine/mod.rs; protocol.schema.json; tests/context_auto.rs; tests/protocol_laws.rs; .bboxy/runs/e1c/queued-cancel-{red,green}.log (101→0); signed ICD 237f441/72d6549"
      },
      {
        "criterion": "5 State-based HarnessAutoCompact and HarnessMcpServer; all seven broken controls fail correctly under native and TLC",
        "reference": ".bboxy/runs/e1c/model-controls.json; .bboxy/runs/e1c/formal-code.json; tests/formal_nonvacuity.rs; verification/{corpus,contracts,coverage}.toml"
      },
      {
        "criterion": "6 Live smoke really PASS on both faces, exit 0; ACP at most three extra requests",
        "reference": ".bboxy/runs/smoke-live-2026-11-01.json; .bboxy/runs/e1c/smoke-live.log; headless=3 requests and ACP=3, both read_calls=1/edit_calls=1/file_changed=true/check_transcript=true"
      },
      {
        "criterion": "Canonical unattended stack, GUI owner-review scope and exact per-layer exits",
        "reference": ".bboxy/runs/e1c/make-test.log; .bboxy/runs/test-current.json; .bboxy/runs/gui-gate.json; .bboxy/runs/e1c-report.md; decision d2400; GUI smoke PASS with 22 visible items/15 kinds/3 approval buttons/scrolled viewport"
      },
      {
        "criterion": "Final signed HEAD and current-source native/TLC proof receipt",
        "reference": "4834a3564dba706373f1d9b78c9b65d226719603; git verify-commit HEAD exit 0; make test-formal-parity exit 0; make check-formal-evidence exit 0; verification/evidence/798cd045cde97946193ed86e0c61f5ccadef3dfd2c35b918ce0eb0d38de24192/evidence.json; .bboxy/runs/e1c/final-head-formal.log"
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1c item 7 / d2019 completed on the same e1c-acp-mcp-compaction branch. PR #6 remains OPEN at signed head f3aa965abc5a1e04a74c601e1dc3e76dfee0a801; no merge. The seat PID producer was already atomic in 52dd070. Required 200 consecutive cargo test --test seat_process runs now exit 0: 600 passed, 0 failed, 0 ignored; exact command, log, source and binary hashes are committed. The audit found and fixed the profile, recovery and escaped ACP-child PID producers in ca59d8e; existing parse polling and lifecycle fences are documented for the remaining readers. CARGO_BUILD_JOBS=2 make test exited 0 unattended on ca59d8e. The final records-only head changes no verification inputs. Final-head native/TLC parity and make check-formal-evidence each exit 0, 52 models, receipt e98035daa2af332e2609f7b1e43f465cc50c4cba962a995b64dc0b1efd3cb1d7. Full report updated in place: Report: [[e1c-report]] (#3022), item 7 section. Product code is unchanged; prior observed headless/ACP live PASS remains in the report, with no extra live requests. E3 review of the previously listed GUI hunks remains required before merge.

```json
{
  "commit": "f3aa965abc5a1e04a74c601e1dc3e76dfee0a801",
  "op_id": "01M6B6E0ME41T5T1E2BC0JJJ56",
  "verify_result": {
    "command": "CARGO_BUILD_JOBS=2 make test",
    "evidence": [
      {
        "criterion": "200 consecutive complete seat_process runs, all three tests each time, no skips or failures",
        "reference": ".bboxy/runs/e1c/seat-200.json; .bboxy/runs/e1c/seat-200.log — for i in $(seq 200); do CARGO_BUILD_JOBS=2 cargo test --test seat_process || exit 1; done exited 0; 200 success summaries, 600 passed; seat source matches the committed atomic fix"
      },
      {
        "criterion": "Atomic seat PID publication from d2019",
        "reference": "tests/seat_process.rs:130; signed fix 52dd070; .bboxy/runs/e1c/make-test-initial.log retains the original ParseIntError Empty"
      },
      {
        "criterion": "Audit and repair other PID-file waits",
        "reference": ".bboxy/runs/e1c/pidfile-audit.json; tests/profile_lifecycle.rs:16; tests/store_recovery.rs:198; tests/bin/mcp_fixture.rs:29 (atomic write/rename); tests/cancel_process.rs:11/:71/:76 and tests/mcp_client.rs:281/:569 retain documented parse/lifecycle fences"
      },
      {
        "criterion": "Full unattended canonical gate with affected fixtures, models and GUI",
        "reference": ".bboxy/runs/e1c/item7-make-test.log; .bboxy/runs/test-current.json; .bboxy/runs/gui-gate.json — all layer exits 0 on ca59d8e; .bboxy/runs/e1c/item7-formal-code.json"
      },
      {
        "criterion": "Final signed head, source-clean current-head formal parity and strict receipt",
        "reference": "f3aa965abc5a1e04a74c601e1dc3e76dfee0a801; git verify-commit HEAD exit 0, Boxy key 9B07E849480C68328CA20684844ECC0587DE7AB0; make test-formal-parity exit 0; make check-formal-evidence exit 0; verification/evidence/e98035daa2af332e2609f7b1e43f465cc50c4cba962a995b64dc0b1efd3cb1d7/evidence.json"
      },
      {
        "criterion": "Committed report and PR handoff",
        "reference": ".bboxy/runs/e1c-report.md:174, note #3022; https://github.com/bboxy450/bboxy450-harness/pull/6 — OPEN at f3aa965; push and PR description update exit 0"
      }
    ],
    "exit_code": 0
  }
}
```
