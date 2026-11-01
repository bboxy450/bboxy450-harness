---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T03:46:12.565Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
decided: 2026-10-31
id: 01M6AKE0CP18F5M7728X6SQC0H
links:
- rel: amends
  to: '[[2026-10-31-toolkit-widgets]]'
number: 824
provenance:
  digest: sha256:e7fd64733eb09022fab07df262cc432224e2068e5faf6b10cad9e272e8229df7
  frontmatter_digest: sha256:5e55cb98184b5294dfe4ce4a7d8f0036a78697f5b840207f93230247e6659090
  log: .bboxy/log/2026-11-01.jsonl#3555
  sig: clfjkWhTeXgh2Lh1H361HuSjTcQlVa4lqlGR/cT6QPX9vbf1S/lTm2REfV/WSzeuyEn4hFI5cPedQwaAtW66DA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: decided
title: 'Task #50 (E2 toolkit widgets) allowed-paths amendment and mux flake disposition'
type: decision
updated: 2026-11-01T03:46:12.565Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
# Task #50 scope amendment

**Context.** Task #50's `allowed_paths` gave the formal-model path wrongly and left out files that the repository's own gates require. The task record is protected once it is claimed (`task_field_protected`), so this decision amends it. The decision is binding on the assignee in the same way the brief is.

## Added allowed paths, in bboxy450-term

- `crates/bboxy450-term/tests/formal_contracts/`: the real location of the formal models. The brief said `tests/formal_contracts/`, which was a path error.
- `Makefile`: the branch, push and story Make workflows need it.
- `ATTRIBUTION.md` and the 31 synchronized `crates/*/ATTRIBUTION.md` copies.
  - The root file adds the credit for the MIT-licensed syntect dependency.
  - `scripts/license-packages.py` (COPIES, around line 168) requires every crate copy to match the root byte for byte.
  - The Oracle verified this from the diff of `91265cde`.

## bbt-mux test failures: intermittent, not a blocker

These two tests failed during E2's `make test`:

- `capture_fairness.rs:352`
- `common/capture_priority.rs:385`

The Oracle re-ran them on `origin/main` `6ff474a1` and on `91265cde`, with the system load average between 40 and 64. On both commits, `cargo test -p bboxy450-term-mux --test capture_fairness` passed 3 out of 3 runs and `make test-seat-ingestion` exited 0.

The branch does not touch `bbt-mux`. The failures are tracked as bboxy450/bboxy450-term#85. If these two tests fail again, cite #85 in the report; any other failure blocks.

## Alternatives considered

1. Reject the task, re-open it, and re-delegate it. Rejected: the churn would stop the executor for no gain.
2. Leave the ruling in chat only. Rejected: the assignee's instructions require a recorded brief.
