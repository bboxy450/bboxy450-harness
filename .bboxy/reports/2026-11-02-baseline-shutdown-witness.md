---
author: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
authorship:
  kind: seat
  process_generation: '1'
  public_key: OBqRnTtMDJMnzhBQ/66MGXbfP4GWXWRjnyTiZEY4qW0=
  signer: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
commits:
- dcb89dfb510c85eb606867a40976197dc8d467b7
created: 2026-11-02T20:16:24.312Z
created_by: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
id: 01M6EYFS4PYQ355YDGXWR86G0R
number: 5720
provenance:
  digest: sha256:02527090c796f7b99a81543e82a9043b9f15e65dd3bf004182341aba1dbb4c06
  frontmatter_digest: sha256:ddca780db73f1bb1d689cbfbe5ebba6c0499729ecfb9f34ecfc32a596cc8d76e
  log: .bboxy/log/2026-11-02.jsonl#38170
  sig: QcthAi9iwBVo2Ez+WJxqAxYrwVXsZPVMyyDwQXuZ2ZbxTp98NbIVahgak8ZhXI/B5klmsewFzio4keP/YYqbBA==
  signer: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
task: '[[tasks/2026-11-02-harness-baseline-shutdown-witness]]'
title: Baseline shutdown witness accepts graceful completion and always proves SIGKILL recovery
type: report
updated: 2026-11-02T20:16:24.312Z
updated_by: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
verify:
  command: 'cargo clippy --offline --locked --all-targets --all-features -- -D warnings && for i in 1 2 3 4 5; do cargo test --offline --locked --features acp,test-fixtures --test seat_process seat_shutdown:: || exit 1; done'
  exit_code: 0
---
Task: [[tasks/2026-11-02-harness-baseline-shutdown-witness]] (#5702).
Worktree: /Users/feral/mydev/bboxy450-harness-shutdownfix; branch baseline-shutdown-witness, based on f75c8cf24415811acebf0f1ab74edc195826bc5a.

Cherry-picked unchanged from E8's signed commit fc7e9ff3550bedf70e720fa42387bf847ed9ad3c, "Make supervisor recovery witness independent of graceful exit timing." The source credit is retained in the commit's cherry-pick line and Source-Report trailer. Signed implementation commit: dcb89dfb510c85eb606867a40976197dc8d467b7 (Boxy <bboxy450@gmail.com>, git signature status G).

tests/support/seat_shutdown.rs:231 accepts a SessionClosed result from the original supervisor witness, as embed-seat ICD §3 (:59) permits. It checks that completed transcript, creates a separate headless crash fixture, and resumes that fixture. An already abrupt original exit still resumes its own session.

:243 starts the fallback with a private HOME, scripted provider and fixture MCP; :281 sends real SIGKILL to the recorded child, cleans only its recorded groups, and requires non-successful exit plus group death. :300 still requires an unclosed log, actual SessionRecovered, successful resumed Ctrl-D within 2 seconds, a closed final log and check_transcript success. :338 unconditionally exercises that real crash/recovery helper, so this coverage never depends on the original exit timing.

The original owned-group assertions remain at :223. The 250ms escalation (:207) and 2s group/Ctrl-D budgets (:219/:320) are unchanged; the ICD's 100ms synchronous-kill contract and all production code remain unchanged. No skip, waiver, deadline relaxation or scripts/test.py edit. The fixture file is byte-identical to E8 fc7e9ff: SHA256 c4edd74e66e1393f93de84185a0082e841eedb3d9fed3e9bb17ce15514998f6e.

Verification on signed implementation dcb89df (20:13:22–20:14:44 UTC), with RUST_TEST_THREADS=1 and the existing /Users/feral/mydev/bboxy450-harness-app/target cache:
- Required Clippy command — exit 0.
- Five consecutive serial runs of the exact seat_shutdown:: command — all exit 0; each reports 6 passed, 0 failed, 0 ignored (30 passes total).
- cargo fmt --check — exit 0.
- git diff --check — exit 0.
- Static audit of source identity and exact scope — exit 0.

Before starting verification, pgrep found no full gate and the anthropic worktree had no Cargo process. Actual preflight and results are retained locally in target/shutdown-evidence/{preflight.json,verify.json,verify.log,audit.json}. The report-only commit changes no verification input; task_report names that final signed head and the PR.

Only tests/support/seat_shutdown.rs and this report changed. No production contract or model changed; no full make test or formal run is claimed for this scoped Clippy/process-witness assignment. No merge, tag, release, credential or peer-source changes.