---
allowed_paths:
- scripts/
- Makefile
- verification/
- tests/
- .bboxy/runs/
artifacts:
- '[[01M6CEWEZJN7W78EQTMZPGFN63]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: u0vQJ7ngwMS5yr2jtfd3HrkhSWQXq2tpEWFJBh5W+4A=
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
created: 2026-11-01T19:28:36.979Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6C9BKH81A65DH77B29TJ2RS
number: 4900
provenance:
  digest: sha256:536f6349bb76fafff1611b2dda0382842853afa43ac093cefc286ab374b924d5
  frontmatter_digest: sha256:623d173e345ddb054522982401740744b0317bb7a372d1cd6d5e86b705564711
  log: .bboxy/log/2026-11-01.jsonl#32775
  sig: F5s2/YulsU/paiKXwIqdcu4Ho1u0lIBZxbJOVKUhwzLuCDB9N9gHZI89UVCN/Tq1CsHTaESew6ZhImoidLKsBQ==
  signer: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Round 2 for PR #89. Fix on branch `landing-budget` together with R5 (#4915), so the PR lands once. The Oracle's review passed R1 (exact covered-test matching) and R4.

    MUST:
    1. R3 reuse key, `model_reuse_key` (scripts/formal.py ~362). The key does not include the TLC harness itself. Add the sha256 of `scripts/formal.py` (tlc() flags and JVM options at :214-233, dot_graph, graph equality), or the relevant input_hashes subset. Scenario: a PR adds `-deadlock` to tlc() and presubmit reuses every archived verdict without running it. Add a control: change formal.py, and reuse must be refused.
    2. R2 machinery set, `landing_machinery` (scripts/landing.py:42-52). It must include every file the fixture corpus copies or exercises, per d5002 R2 "or a fixture they read". That means formal.py, quarantine.sh, main_watch.py, main-watch.sh, main-red-bounded.sh, release-inputs.py, verify-signatures.py, verify-signatures-core.py and harness-submodule.py (see test-landing.py:129-133,378,522). Ruling on harness-submodule.py: include it, because the fixtures copy it. I'm asking the Architect to correct d5002's verification row. Prefer deriving the set from the fixture copy list rather than a second hand-maintained list, so the two can't drift. Add a test that a cut touching only main_watch.py runs the fixtures.
    3. Wire `scripts/test-landing-budget-controls.py` into the unattended stack: ci-local, and so make test. As it stands the nine controls never run.

    SHOULD:
    4. Make the R1-ignored control mutate the ignored-result discrimination itself, not add a `--no-cargo` flag. Make R1-filtered target the filter check rather than a bare `return True`.
    5. Record the residual risk in your report: the formal evidence store under git-common-dir is shared by every local branch and nothing signs it. Postsubmit never reuses, which bounds it.

    Then run make test, test-landing.py and test-landing-recovery.py, push the signed commits, append to the report (no "key:" before hex), and send task_report for #4900. Send #4915's report separately.
report:
  commit: 6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff
  op_id: 01M6CMH8TR1N67QFKDYKZAMCBC
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: 'Round 2 complete on landing-budget, PR #89: https://github.com/bboxy450/bboxy450-term/pull/89. Signed source 6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff is pushed; source worktree clean. R3 now binds reuse to checker_sha256 of formal.py, test-formal.py, bboxy450-tlc and bbt-yarn; a checker-byte change forces every model to recheck. R2 derives machinery directly from test-landing.py''s actual FIXTURE_SCRIPTS tuple, including main_watch.py and harness-submodule.py; tuple-only additions trigger automatically. Canonical ci-local/make test now executes all 18 isolated mutation controls. Ignored/filtered controls mutate their specific result discrimination. Exact make test exited 0 on the first full run of this head; native/TLC checked 201 models with zero reuse, then a separate opt-in warm run reused all 201 and strict receipt verification passed. Standalone landing 162 fixtures, recovery 46 tests and formal-driver 22 tests all exit 0. All 18 controls go 1 -> 0 at named assertions (Python unittest statuses). Full appended report is .bboxy/runs/2026-11-01-landing-budget-report.log in harness, signed local report-only commit 89e77964cac00dca35af3506dd0909c1b3a81f69, ref landing-budget-report-r2-4900, digest sha256:728518530326907f5c6cb81b36407f178210d851c5adebdd57665869caadc8bd; no unrelated harness history pushed. Report records unsigned shared formal-store trust boundary, postsubmit never reuses, and F1: earlier unchanged knowledge_provider.rs:239 complex-YAML failure with retained full/isolation commands and logs; no deterministic seed captured, no quarantine added. Real two-cut bbt-mux margin remains unmeasured. No merge or production land.sh.'
role: executor
status: reported
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E6: implement d5002 landing presubmit budget (R0-R4) in bboxy450-term'
type: task
updated: 2026-11-01T22:43:57.626Z
updated_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Exact full unattended gate; signed pushed source
    reference: target/landing-budget-r2/make-test.log and make-test.exit=0; ci-local.log ends 'result 6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff normal 0'; signature.exit=0; git ls-remote matches source; git status --short empty.
  - criterion: R3 checker changes refuse reuse; postsubmit fresh parity
    reference: 'scripts/formal.py:389,402; scripts/test-formal.py:378; canonical-control-outputs/R3-checker-red.log named ''changed checker harness reused evidence'' exit1, restored green exit0. Fresh archive 40ab719ff655359c2521043a08c3ce4694810926c1121fcda31886799e9dcb68: 201 graphs/verdicts match, 0 reused; checker digest 8225d069bbb7478217c1954ec79dee0f5f617534907937488eab1a3c947b3e32.'
  - criterion: R2 fixture copy set cannot drift
    reference: scripts/landing.py:43,57 reads actual tuple scripts/test-landing.py:35,135. Main_watch-only positive and tuple-addition tests at :2499,2505 pass; R2-copy-tuple control 'added fixture copy did not trigger' 1 -> 0. Brand-only skip and harness-submodule trigger pass.
  - criterion: Controls run inside canonical make test; targeted ignored/filtered controls
    reference: scripts/ci-local.sh:75; test-landing.py:923 actual-CI sentinel exit19 propagates. canonical-control-outputs/summary.json has 18 rows, every red_exit=1 and green_exit=0, including CI-controls/R1-filtered/R1-ignored; full ci-local.log records all 18.
  - criterion: Required standalone verification and actual reuse
    reference: target/landing-budget-r2/test-landing.exit=0 (162), test-landing-recovery.exit=0 (46), test-formal.exit=0 (22), controls.exit=0. formal-reuse.exit=0 reuses all 201 from fresh archive; check-formal-evidence.exit=0, warm archive 67e927a292b62d013ae178b5c219ec6ae85d7c503e87ba4344de489dd4c145b3.
  - criterion: Committed report, residual trust risk and finding F1
    reference: Harness signed report-only 89e77964cac00dca35af3506dd0909c1b3a81f69:.bboxy/runs/2026-11-01-landing-budget-report.log, Round 2 appendix; command/exit/hash inventory, named controls, unsigned shared git-common-dir store risk, knowledge_provider.rs:239 prior exit2 and passing isolated commands; secret scan exit0. verification/README.md:289 and contracts.toml:299 record store assumption.
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Work on a new worktree and branch `landing-budget` from origin/main (289218b6 or later), and open a PR. Do not merge or run land.sh; the Oracle lands it.

Binding: bboxy450-term `.bboxy/decisions/d5002*` (proposed) on main at 289218b6. Read it fully. The presubmit cap stays at 900 s. Allocation: formal 290, tests 400, fixtures 60, invariants/misc 60, margin 90.

Implement the following. Each rule needs one positive case and one broken control that goes red:
- R0: per-sub-step timing. The presubmit records `stage_seconds` per stage, plus the load average, in its result JSON.
- R1: no double runs. `invariants.sh --affected` passes `--no-cargo`, or an equivalent, for packages quarantine-test already ran in the same presubmit. The ctl binary must not run twice.
- R2: landing fixtures (`test-landing.py --all` plus resource-ledger) run in presubmit only when the landing machinery changes, as d5002 defines it. ci-local postsubmit still runs them always. Today landing.py:831 runs them for any scripts/ change.
- R3: formal verdict reuse per model, only when the .tla bytes, the expectation and the tool pins all match archived evidence. Postsubmit never reuses. The broken control: a changed .tla with a stale archive must re-check, not reuse.
- R4: a stage over its allocation is an advisory finding in the result JSON. Only the 900 s total blocks.

Acceptance:
- Full `make test` exits 0.
- `python3 -I scripts/test-landing.py` and `python3 -I scripts/test-landing-recovery.py` both exit 0.
- A presubmit dry run on a code-tier change (use land's existing fixture harness) shows stage_seconds and no duplicate ctl run.
- All commits are signed, with no Report trailer.
- The report goes in bboxy450-harness `.bboxy/runs/2026-11-01-landing-budget-report.log`. Tell me the commit; don't push unrelated history.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

Implemented d5002 R0–R4 in bboxy450-term PR #89 (https://github.com/bboxy450/bboxy450-term/pull/89), branch landing-budget, signed HEAD 680c0579bdd777e808542024f5bb64e3e98e56b4. Exact make test exited 0 unattended; full postsubmit compared 201 native/TLC models with 0 reused verdicts, and a subsequent opt-in formal run reused all 201 with a valid current-source receipt. Nine named mutation controls went 1 -> 0. Full command/exit/hash report: [[runs/2026-11-01-landing-budget-report.log]], SHA-256 dee342e882f0d01e751a831425563b75c2f4b257a0fb3f47c752246de5f76ed5, signed local harness report-only commit 2bb259987ae1150573abd5f54d4dffcbf9d4156d (refs/heads/landing-budget-report-4900, not pushed). Term source tree is clean; only the new term branch was pushed. Real two-cut bbt-mux margin measurement remains unmeasured. No merge or production land.sh run.

```json
{
  "commit": "680c0579bdd777e808542024f5bb64e3e98e56b4",
  "op_id": "01M6CEXB8ZJGVPDY7BYK9VTCAX",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Full unattended canonical gate",
        "reference": "/Users/feral/mydev/bboxy450-term-landing-budget/target/ci-local.log ends: ci-local: result 680c0579bdd777e808542024f5bb64e3e98e56b4 normal 0; full report records its SHA-256. Headless smoke passed; final invariants: 27 checks, 0 failures, 11 slow checks skipped."
      },
      {
        "criterion": "R0 per-stage/sub-step timings and load",
        "reference": "scripts/test-landing.py BudgetClockTests and BudgetPresubmitTests.test_observed_ctl_runs_once pass; target/landing-budget/dry-run.json records four stages, four formal sub-steps, start/end load, exit 0 and ctl_runs=[ctl]. Missing-phase mutation exits 1 then restored 0."
      },
      {
        "criterion": "R1 no duplicate ctl; filtered/ignored tests remain covered",
        "reference": "Real Rust ctl fixture runs once when covered and twice when filtered or ignored. Wrong-attempt/target receipts cannot suppress execution. R1, R1-filtered and R1-ignored named controls each exit 1 -> 0, source hashes match signed source."
      },
      {
        "criterion": "R2 fixture dependency scope",
        "reference": "BudgetPresubmitTests.test_harness_helper_cut_skips_landing_fixtures and test_landing_machinery_cut_runs_fixtures pass (positive changes landing.py). Broad scripts/ mutation exits 1 at unrelated script ran landing fixtures, restored 0. Full ci-local retains unconditional fixture runs."
      },
      {
        "criterion": "R3 per-model archive reuse with fresh receipts",
        "reference": "Postsubmit evidence 275951b78d9de047b0c09ef051333e0d04807d3ef21a303f1f18a348e3c1dbb3: 201 models, 0 reused. Opt-in evidence 831269341df5d0813ba64fae4e7094175d2d801791205d00d41269a6ccda6ec5: 201 reused_from that archive. make test-formal FORMAL_REUSE=1 and make check-formal-evidence exit 0. Stale TLA and postsubmit-reuse mutations exit 1 -> 0; changed tool pins recheck every model."
      },
      {
        "criterion": "R4 advisory stage overruns; unchanged hard 900 s bound",
        "reference": "BudgetPresubmitTests.test_allocation_overrun_is_advisory and BudgetClockTests.test_stage_allocation_is_advisory_but_900_seconds_still_blocks pass. Named controls exit 1 -> 0. Virtual time above 900 s yields 124; stage overrun yields a finding and exit 0."
      },
      {
        "criterion": "Required fixture commands",
        "reference": "python3 -I scripts/test-landing.py -> 0 (152); python3 -I scripts/test-landing-recovery.py -> 0 (46); python3 -I scripts/test-formal.py -> 0 (20); five existing extended cases -> 0; all raw log hashes and command lines are in the committed .log."
      },
      {
        "criterion": "Signed scoped branch, PR and report",
        "reference": "Term signed HEAD 680c0579bdd777e808542024f5bb64e3e98e56b4; git verify-commit -> 0; origin/landing-budget matches; PR #89 OPEN; 13 scoped files. Harness report-only signed commit 2bb259987ae1150573abd5f54d4dffcbf9d4156d changes only .bboxy/runs/2026-11-01-landing-budget-report.log. Neither commit has a Report trailer."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

Round 2 complete on landing-budget, PR #89: https://github.com/bboxy450/bboxy450-term/pull/89. Signed source 6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff is pushed; source worktree clean. R3 now binds reuse to checker_sha256 of formal.py, test-formal.py, bboxy450-tlc and bbt-yarn; a checker-byte change forces every model to recheck. R2 derives machinery directly from test-landing.py's actual FIXTURE_SCRIPTS tuple, including main_watch.py and harness-submodule.py; tuple-only additions trigger automatically. Canonical ci-local/make test now executes all 18 isolated mutation controls. Ignored/filtered controls mutate their specific result discrimination. Exact make test exited 0 on the first full run of this head; native/TLC checked 201 models with zero reuse, then a separate opt-in warm run reused all 201 and strict receipt verification passed. Standalone landing 162 fixtures, recovery 46 tests and formal-driver 22 tests all exit 0. All 18 controls go 1 -> 0 at named assertions (Python unittest statuses). Full appended report is .bboxy/runs/2026-11-01-landing-budget-report.log in harness, signed local report-only commit 89e77964cac00dca35af3506dd0909c1b3a81f69, ref landing-budget-report-r2-4900, digest sha256:728518530326907f5c6cb81b36407f178210d851c5adebdd57665869caadc8bd; no unrelated harness history pushed. Report records unsigned shared formal-store trust boundary, postsubmit never reuses, and F1: earlier unchanged knowledge_provider.rs:239 complex-YAML failure with retained full/isolation commands and logs; no deterministic seed captured, no quarantine added. Real two-cut bbt-mux margin remains unmeasured. No merge or production land.sh.

```json
{
  "commit": "6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff",
  "op_id": "01M6CMH8TR1N67QFKDYKZAMCBC",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Exact full unattended gate; signed pushed source",
        "reference": "target/landing-budget-r2/make-test.log and make-test.exit=0; ci-local.log ends 'result 6c9d9d7497eef3b6db74f6af53b9471e05a2c2ff normal 0'; signature.exit=0; git ls-remote matches source; git status --short empty."
      },
      {
        "criterion": "R3 checker changes refuse reuse; postsubmit fresh parity",
        "reference": "scripts/formal.py:389,402; scripts/test-formal.py:378; canonical-control-outputs/R3-checker-red.log named 'changed checker harness reused evidence' exit1, restored green exit0. Fresh archive 40ab719ff655359c2521043a08c3ce4694810926c1121fcda31886799e9dcb68: 201 graphs/verdicts match, 0 reused; checker digest 8225d069bbb7478217c1954ec79dee0f5f617534907937488eab1a3c947b3e32."
      },
      {
        "criterion": "R2 fixture copy set cannot drift",
        "reference": "scripts/landing.py:43,57 reads actual tuple scripts/test-landing.py:35,135. Main_watch-only positive and tuple-addition tests at :2499,2505 pass; R2-copy-tuple control 'added fixture copy did not trigger' 1 -> 0. Brand-only skip and harness-submodule trigger pass."
      },
      {
        "criterion": "Controls run inside canonical make test; targeted ignored/filtered controls",
        "reference": "scripts/ci-local.sh:75; test-landing.py:923 actual-CI sentinel exit19 propagates. canonical-control-outputs/summary.json has 18 rows, every red_exit=1 and green_exit=0, including CI-controls/R1-filtered/R1-ignored; full ci-local.log records all 18."
      },
      {
        "criterion": "Required standalone verification and actual reuse",
        "reference": "target/landing-budget-r2/test-landing.exit=0 (162), test-landing-recovery.exit=0 (46), test-formal.exit=0 (22), controls.exit=0. formal-reuse.exit=0 reuses all 201 from fresh archive; check-formal-evidence.exit=0, warm archive 67e927a292b62d013ae178b5c219ec6ae85d7c503e87ba4344de489dd4c145b3."
      },
      {
        "criterion": "Committed report, residual trust risk and finding F1",
        "reference": "Harness signed report-only 89e77964cac00dca35af3506dd0909c1b3a81f69:.bboxy/runs/2026-11-01-landing-budget-report.log, Round 2 appendix; command/exit/hash inventory, named controls, unsigned shared git-common-dir store risk, knowledge_provider.rs:239 prior exit2 and passing isolated commands; secret scan exit0. verification/README.md:289 and contracts.toml:299 record store assumption."
      }
    ],
    "exit_code": 0
  }
}
```
