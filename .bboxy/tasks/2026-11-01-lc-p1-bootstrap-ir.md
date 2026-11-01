---
allowed_paths:
- Cargo.toml
- Cargo.lock
- src/
- tests/
- scripts/
- Makefile
- verification/
- deny.toml
- .bboxy/
- README.md
assignee: seat:01M6CKGYEPHYP25GFF6H7B1X23
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:26:13.369Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CKGT5A1EWXG399B6ZQ5ZP5
number: 4976
provenance:
  digest: sha256:bbd39afa7fc1ca607e5abc4ebfc2511cac8e44f0918840de86b81ff9321f2e0a
  frontmatter_digest: sha256:1fa95891bbb05916bfb4e4f63a2fc22a4a5b9e5422b9fe8c99867bd22c4d7770
  log: .bboxy/log/2026-11-01.jsonl#32255
  sig: IT7KIthRpkDsoaYaLmEmI5eK9AXGRJ1unYsTkvqVMNJE6xFOMq40W5MyvdEelGmiQxejoNW57rekSMRv7wbSCg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'LC-P1: crate bootstrap, IR types, CBOR golden, deny check, formal stack'
type: task
updated: 2026-11-01T22:26:24.445Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-log-converter (clone at ~/mydev/bboxy450-log-converter). Work on a new branch `p1-bootstrap-ir` and open a PR. Do not merge.

Binding: bboxy450-harness `.bboxy/specs/log-converter.md` (#4965) phase P1 and §7, `interfaces/lc-ir.md` (#4960), and decisions d4949 and d4964. Read them first.

Build:
1. The Rust crate `bboxy450-log-converter`, library plus CLI skeleton. Every file carries the ATTRIBUTION.md header.
2. The IR types exactly as lc-ir.md specifies:
   - ATIF-aligned steps and tool calls;
   - content blocks;
   - provider-tagged reasoning: readable text plus the sealed part, never folded into message text;
   - compaction;
   - the Claude parentUuid DAG;
   - Context (turn_context);
   - lineage;
   - fidelity per event;
   - the retained raw record;
   - the deterministic EventId.
3. CBOR encoding with integer keys and deterministic encoding, with golden tests that cover every variant and a round trip. Unknown keys must be preserved.
4. A deny check, enforced in `make test`, that forbids ctx, casr and the other d4949 crates or copied code. Include a broken control.
5. A Makefile `test` target in the style of the harness: fmt, clippy (default and all features), test, headers, secrets, and the formal stack wired the way bboxy450-harness and bboxy450-term run it, with native Rust TLC (bboxy450-tlc) plus Java parity. It starts empty but is enforced, so later phases' models plug in.
6. A verification inventory listing the crate's models and their status.

Acceptance:
- `make test` exits 0 unattended.
- All commits are signed.
- The report goes in `.bboxy/runs/p1-bootstrap-ir-report.log` in this repo.
- Send one task_report with verify_result.command exactly `make test`.