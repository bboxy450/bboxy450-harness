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
created: 2026-11-01T21:07:38.309Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CF0XJFJ4X0CTNWZZXFSMH7
number: 4915
provenance:
  digest: sha256:fc5b97638ad27d32fe655f328fb4a6214352f451be5fbc8606cac9383bb1c07a
  frontmatter_digest: sha256:efb8f6a6b4a81c941ea912f5242792342b217fc7a376ff16fa80f042117b9889
  log: .bboxy/log/2026-11-01.jsonl#31693
  sig: /BlkqO11TTRgEm7kYLmZC1jxljkXWGSKYLTtDm3+CwkvCjgrKbQPsMgwd+kdgryfF0o3dmYCHb2j6i19bniKCw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E6b: d5002 R5 records-only rebase acceptance for exact-SHA gate logs'
type: task
updated: 2026-11-01T21:07:41.376Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Continue on branch `landing-budget` (PR #89, after 680c0579). Add R5 as new signed commits on the same PR, so it lands once. Do not merge or run land.sh.

Binding: bboxy450-term `.bboxy/decisions/d5002*` rule R5, as amended at f001755b. Read its exact text.

Implement R5: when `land --gate-log` is given a green full ci-local log for an exact SHA, and the candidate after rebase differs from that SHA only in `.bboxy/` record files (paths exactly as R5 defines them), land accepts the log. It reruns only the records checks named by R5 (kb records, links resolving, secrets) on the rebased candidate, and does not reject for a SHA mismatch. Any non-`.bboxy/` delta keeps today's behaviour: the log is refused.

Tests in the landing fixtures:
(a) a records-only delta is accepted, and only the records checks run;
(b) a delta touching any non-records path is refused;
(c) a records-only delta whose records check fails is refused;
(d) a rename or mode change outside `.bboxy/` counts as non-records.
Broken control: treat any delta as records-only. Case (b) must go red.

Acceptance:
- `make test`, `python3 -I scripts/test-landing.py` and `python3 -I scripts/test-landing-recovery.py` all exit 0.
- All commits are signed.
- Append the R5 section to your existing report `.bboxy/runs/2026-11-01-landing-budget-report.log`. Do not write "key:" directly before any hex value, because the secret scan flags it.
- Send one task_report for this task with verify_result.command exactly `make test`.