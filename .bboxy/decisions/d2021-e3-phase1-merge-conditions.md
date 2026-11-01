---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T06:19:36.162Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
decided: 2026-10-31
id: 01M6AW6W6TAC6Z25P5SE9SAFB6
links:
- rel: amends
  to: '[[2026-10-31-harness-gui]]'
number: 2021
provenance:
  digest: sha256:410ad556a39e44ac2a35f5096f5bd15729158c10443e12f62a2ed85610465d1c
  frontmatter_digest: sha256:06f470a282ead9cc15e8965bfe26252bb2011d9ab2713d98978734af908e66b3
  log: .bboxy/log/2026-11-01.jsonl#11629
  sig: wNRRLr2B7GjVWCX0Qdyi8NzxnFhgRgGe2vHWAFnUYSifBVPnsu4PTC/qXquj3zsUEGZlAZBiYPqwaFNcPy/FBA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: decided
title: 'PR #2 (E3 GUI phase 1) merge disposition: residual gaps become phase 2 entry conditions'
type: decision
updated: 2026-11-01T06:19:36.162Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
# PR #2 merge disposition

The Oracle re-reviewed `d89094c`. Items 1, 2, 5 and 6 of the rejection are fixed and non-vacuous, and so is the source-of-truth for live terminal output. Two residual gaps remain. Each guards behaviour that does not exist in phase 1 yet, so each becomes a binding **phase 2 entry condition** and does not trigger a third phase-1 round.

1. **Budget §2, "a block over 4 KiB is never parsed on the UI thread".** `tests/gui_budget.rs` `completed_layouts_are_retained_and_large_blocks_prepare_off_ui` calls `Preparer::prepare` inside its own `runtime::blocking`, so it checks where the test put the work, not what production does. Phase 1 parses no markdown, so the property holds trivially in production today. Phase 2 introduces markdown parsing, and it must drive the check through `Bridge::replay` / `publish_frame` and read `state.prepared.prepared_on_ui`.
2. **The 4 KiB card body cap** (`view.rs:741-749`). In phase 2, streaming markdown with a virtualized transcript replaces the cap. A long reply must be fully readable, either expandable or virtualized, never silently truncated.

Also in phase 2:
- Correct the README wording about 100k charged deltas.
- Assert the card role in a test.
- Assert the live terminal tail as the view renders it mid-stream.
- Add an API witness that an old gateway's Close leaves a new actor running.

The pre-existing flake in `seat_process` is tracked in [[d2019-seat-process-pidfile-race]].
