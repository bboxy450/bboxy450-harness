---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T06:19:06.776Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
decided: 2026-10-31
id: 01M6AW5ZHPKYAXC6VYFT12W6YE
links:
- rel: amends
  to: '[[2026-10-31-harness-acp-mcp-compaction]]'
number: 2019
provenance:
  digest: sha256:6e1c14cc122b851f4deeb80d2abd9c10ccd55b7d718972fbeecdb4b44dcf41cc
  frontmatter_digest: sha256:406959daf8f6832d433c0e7c988832b0a735804541ef9549c9fb70541b691287
  log: .bboxy/log/2026-11-01.jsonl#11623
  sig: bRw8I1/lsYeq+GV/aTQF1KSCn4WXj4K5TUlM/IWTADGXGZ+uAP6/uPSz66UbYWAG4rYhoS7MVhm9igesfq3MAA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: decided
title: 'seat_process pidfile race: pre-existing flake, assigned to E1c (#1992) item 7'
type: decision
updated: 2026-11-01T06:19:06.776Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
# seat_process pidfile race

**Evidence.** On the merge of main and PR #2, `make test` exited 2. The failure was at `tests/seat_process.rs:145`: `ParseIntError { kind: Empty }`. The Oracle reran the test 30 times; it failed once.

**Cause.** The fixture runs `echo $$ > key.pid`. The shell creates the file before `echo` writes to it. The test waits only until `pid.exists()`, so it can read an empty file. The bug is in the test, not the product. The test arrived in PR #1 (`88b2ff7`).

**Disposition.**
- This failure does not block PR #2. E3 does not own the code, and the same code is on `main`.
- It is assigned to E1c (#1992) as item 7. The fixture writes to a temporary file and renames it into place, or the test waits until the file has a non-empty, parseable pid.
- Evidence required: 200 consecutive passing runs of `cargo test --test seat_process`.
- Also check the other pidfile waits in `tests/` for the same race.
