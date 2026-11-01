---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- Makefile
- .bboxy/runs/
artifacts:
- '[[01M6BDA6DK5SDYZ9T4YTB4XXBE]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T10:54:48.881Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BBYSZ542PGBJ39XKNZEH63
number: 3815
provenance:
  digest: sha256:e6438602dd518f988452dd5be5b27a58281de0d30ca08ad1c09a48ca62773133
  frontmatter_digest: sha256:37dfc6a733e82068bafa256413008f83c31eec868a1d1d39712a320acbe8eca1
  log: .bboxy/log/2026-11-01.jsonl#24313
  sig: 0SSpEgqf4c1vJZaKWMdoRiGwxJeMqQFgGMojnjAIQfGtfeIe06TRh+sQf91sUxS7+1BxH15P9+Gy64Oa2kmmBQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: 52f5419f0602d69e0877ba7682bada1b047b06ef
  op_id: 01M6BDG63A8T1FENSW9S7BXMYC
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: |-
    All four E1c follow-ups addressed on e1c-acp-mcp-followups. PR #7 OPEN/CLEAN: https://github.com/bboxy450/bboxy450-harness/pull/7 . Signed head52f5419f0602d69e0877ba7682bada1b047b06ef; both commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
    Full committed report: .bboxy/runs/2026-11-01-e1c-followups-report.log (Report: [[2026-11-01-e1c-followups-report]]). Actual make test=0 unattended on b4116b3, without env prefix; all126 inputs are byte-identical at the final records head. Current-head make test-formal-parity=0 and make check-formal-evidence=0:58 models/23 contracts, receipt74f930bf9c1254c9ca15a5fd3219b3e025e11b0729fd6a80f0a26fe697b70ddd.
    Each of four new controls exits101 at its named assertion then0 restored: actual emit enabledness/EngineNeverWaits, leader/member close/NoMemberAfterClose, admission lag/hang guard, positive-pid signals/orphaned member after Close joins. Retained no-signal control also101→0 at the separate Close join guard. Shared close helper now has a hang guard. The lag test fills the actual4096-slot queue and holds EOF after Lagged; its limit is explicit in the report. Tracked worktree clean; generated proof archives retained. No GUI/wire/schema changes, no new live-provider requests, no assigned gap.
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1c follow-ups: AcpFlow invariant, MCP group-kill model, admission lag, leader-only control'
type: task
updated: 2026-11-01T11:26:52.238Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: 1 EngineNeverWaits is actual emit enabledness; writer-coupled control fails safety
    reference: tests/formal/models.rs::acp_flow and BrokenHarnessAcpFlowC; tests/formal_nonvacuity.rs::acp_engine_emit_stays_enabled_with_a_full_writer checks actual full-queue emit edges; .bboxy/runs/e1c-followups/acp-engine-writer-{red,green}.log:101→0;58-model native/TLC parity
  - criterion: 2 MCP Close models leader and members ignoring TERM; leader-only control fails NoMemberAfterClose
    reference: HarnessMcpGroupClose/BrokenHarnessMcpGroupClose in tests/formal/models.rs; tests/formal_nonvacuity.rs::mcp_close_joins_members_even_when_term_is_ignored; verification/{contracts,coverage,corpus}.toml maps finish_child; .bboxy/runs/e1c-followups/mcp-close-model-{red,green}.log:101→0; correct8states/11edges, broken12states/15edges match TLC
  - criterion: 3 Lagged admission replays from resume_after and observes the correct caused durable outcome
    reference: src/frontends/acp.rs::await_prompt_admission and ::admission_lag_replays_the_durable_prompt_before_next_input; actual4096-slot overflow, held EOF, precise TurnQueued cause, check_transcript; admission-baseline-red.log and admission-lag-{red,green}.log:101 at named10s hang guard→0; held-EOF limit documented in report
  - criterion: 4 Positive-pid mutation exposes orphaned member rather than Close timeout; shared helper bounded
    reference: scripts/test-e1c-rework-controls.py::leader_only; tests/mcp_client.rs::close 15s hang guard and close_kills_a_server_and_member_that_ignore_eof_and_term; tests/bin/mcp_fixture.rs member stdio isolation; leader-only-red.log says orphaned member after Close joined, then green0; retained group-kill-red.log separately says Close join hang guard
  - criterion: Canonical unattended acceptance, exact verify command, signed head and current-source formal proof
    reference: .bboxy/runs/e1c-followups/make-test.log, verification.json, test-current.json, gui-gate.json:make test and every layer0; fmt/clippy/mock/unit/integration/hardening/native then TLC/secret/header/KB+negative/GUI smoke; all126 input hashes match final head; receipt74f930bf9c1254c9ca15a5fd3219b3e025e11b0729fd6a80f0a26fe697b70ddd; git verify-commit b4116b3 HEAD=0
  - criterion: Four new controls plus retained no-signal control all show named101→0, with commands and actual statuses
    reference: .bboxy/runs/e1c-followups/controls.json and controls.log; command python3 -I scripts/test-e1c-rework-controls.py --case group-kill --case acp-engine-writer --case mcp-close-model --case admission-lag --case leader-only --out .bboxy/runs/e1c-followups exited0; source restored in finally; full committed report .bboxy/runs/2026-11-01-e1c-followups-report.log
  exit_code: 0
---
ROLE: EXECUTOR. Worktree: your existing harness-core worktree, on a new branch from origin/main (ccce095 or later). PR to bboxy450/bboxy450-harness; do not merge.

These are the non-blocking findings from the Oracle's adversarial review of PR #6 (merged as eef1153). Fix all four.

1. **AcpFlow `EngineNeverWaits` is vacuous** (tests/formal/models.rs, `acp_flow`). The engine-emit guard never reads `writer_queue`, so the invariant can't fail on the engine waiting on the writer. Model the engine→writer coupling (a bounded queue, with the engine's emit blocked or not) so that the invariant reads that state. Add a broken control where emit waits on a full writer queue, which must violate it. Otherwise rename the invariant to what it really checks and record the gap in verification/coverage.toml. The model is preferred.
2. **No model for MCP process-group close.** Add Close and process-group state to the HarnessMcpServer model, or add a small new model: leader plus members, with SIGTERM ignored. Safety: after Close completes, no group member is alive. Broken control: kill the leader only. Map it to `finish_child` in verification.
3. **Admission-lag hang** (src/…/acp.rs:226-237). If the admission subscription lags past the prompt's first durable event, the loop never exits and the session worker hangs. Handle Lagged: resubscribe or replay from `resume_after`, or treat the turn as admitted. Add a test that forces the lag. A broken control without the fix must hang until a test guard trips (structural, with no wall-clock percentile).
4. **Leader-only kill control.** In scripts/test-e1c-rework-controls.py, add a mutation that replaces `kill(-pgid)` with `kill(pid)`. It must go red on the orphaned-member assertion, not on a Close timeout. Also add a hang guard to the shared `close()` helper (tests/mcp_client.rs:209).

Acceptance: `make test` exits 0 unattended. Each new control shows 101 before the fix and 0 after, with its named assertion. Native/TLC parity holds. Commits are signed. Put the report in .bboxy/runs/2026-11-01-e1c-followups-report.log (a .log file, not .md). Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

All four E1c follow-ups addressed on e1c-acp-mcp-followups. PR #7 OPEN/CLEAN: https://github.com/bboxy450/bboxy450-harness/pull/7 . Signed head52f5419f0602d69e0877ba7682bada1b047b06ef; both commits verify with Boxy key9B07E849480C68328CA20684844ECC0587DE7AB0. No merge.
Full committed report: .bboxy/runs/2026-11-01-e1c-followups-report.log (Report: [[2026-11-01-e1c-followups-report]]). Actual make test=0 unattended on b4116b3, without env prefix; all126 inputs are byte-identical at the final records head. Current-head make test-formal-parity=0 and make check-formal-evidence=0:58 models/23 contracts, receipt74f930bf9c1254c9ca15a5fd3219b3e025e11b0729fd6a80f0a26fe697b70ddd.
Each of four new controls exits101 at its named assertion then0 restored: actual emit enabledness/EngineNeverWaits, leader/member close/NoMemberAfterClose, admission lag/hang guard, positive-pid signals/orphaned member after Close joins. Retained no-signal control also101→0 at the separate Close join guard. Shared close helper now has a hang guard. The lag test fills the actual4096-slot queue and holds EOF after Lagged; its limit is explicit in the report. Tracked worktree clean; generated proof archives retained. No GUI/wire/schema changes, no new live-provider requests, no assigned gap.

```json
{
  "commit": "52f5419f0602d69e0877ba7682bada1b047b06ef",
  "op_id": "01M6BDG63A8T1FENSW9S7BXMYC",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1 EngineNeverWaits is actual emit enabledness; writer-coupled control fails safety",
        "reference": "tests/formal/models.rs::acp_flow and BrokenHarnessAcpFlowC; tests/formal_nonvacuity.rs::acp_engine_emit_stays_enabled_with_a_full_writer checks actual full-queue emit edges; .bboxy/runs/e1c-followups/acp-engine-writer-{red,green}.log:101→0;58-model native/TLC parity"
      },
      {
        "criterion": "2 MCP Close models leader and members ignoring TERM; leader-only control fails NoMemberAfterClose",
        "reference": "HarnessMcpGroupClose/BrokenHarnessMcpGroupClose in tests/formal/models.rs; tests/formal_nonvacuity.rs::mcp_close_joins_members_even_when_term_is_ignored; verification/{contracts,coverage,corpus}.toml maps finish_child; .bboxy/runs/e1c-followups/mcp-close-model-{red,green}.log:101→0; correct8states/11edges, broken12states/15edges match TLC"
      },
      {
        "criterion": "3 Lagged admission replays from resume_after and observes the correct caused durable outcome",
        "reference": "src/frontends/acp.rs::await_prompt_admission and ::admission_lag_replays_the_durable_prompt_before_next_input; actual4096-slot overflow, held EOF, precise TurnQueued cause, check_transcript; admission-baseline-red.log and admission-lag-{red,green}.log:101 at named10s hang guard→0; held-EOF limit documented in report"
      },
      {
        "criterion": "4 Positive-pid mutation exposes orphaned member rather than Close timeout; shared helper bounded",
        "reference": "scripts/test-e1c-rework-controls.py::leader_only; tests/mcp_client.rs::close 15s hang guard and close_kills_a_server_and_member_that_ignore_eof_and_term; tests/bin/mcp_fixture.rs member stdio isolation; leader-only-red.log says orphaned member after Close joined, then green0; retained group-kill-red.log separately says Close join hang guard"
      },
      {
        "criterion": "Canonical unattended acceptance, exact verify command, signed head and current-source formal proof",
        "reference": ".bboxy/runs/e1c-followups/make-test.log, verification.json, test-current.json, gui-gate.json:make test and every layer0; fmt/clippy/mock/unit/integration/hardening/native then TLC/secret/header/KB+negative/GUI smoke; all126 input hashes match final head; receipt74f930bf9c1254c9ca15a5fd3219b3e025e11b0729fd6a80f0a26fe697b70ddd; git verify-commit b4116b3 HEAD=0"
      },
      {
        "criterion": "Four new controls plus retained no-signal control all show named101→0, with commands and actual statuses",
        "reference": ".bboxy/runs/e1c-followups/controls.json and controls.log; command python3 -I scripts/test-e1c-rework-controls.py --case group-kill --case acp-engine-writer --case mcp-close-model --case admission-lag --case leader-only --out .bboxy/runs/e1c-followups exited0; source restored in finally; full committed report .bboxy/runs/2026-11-01-e1c-followups-report.log"
      }
    ],
    "exit_code": 0
  }
}
```
