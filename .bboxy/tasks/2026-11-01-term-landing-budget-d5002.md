---
allowed_paths:
- scripts/
- Makefile
- verification/
- tests/
- .bboxy/runs/
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T19:28:36.979Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6C9BKH81A65DH77B29TJ2RS
number: 4900
provenance:
  digest: sha256:be7ec9a4af9d0424711b89fc827c2851a55319865480efb01af4a09396758e0d
  frontmatter_digest: sha256:d356b188e9e1eccc221e195fe319ecedb25b4520a0d86dc8fce8a252dbb5e518
  log: .bboxy/log/2026-11-01.jsonl#31510
  sig: gC0nDByiNbEo+FLaL9+y3vau8uVfkrZIOcm5LEnqgU2OlUHeEcTNWNpg2xlrMTbxkJ7NMw/4/quKdyGohzttCQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E6: implement d5002 landing presubmit budget (R0-R4) in bboxy450-term'
type: task
updated: 2026-11-01T19:28:40.207Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
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