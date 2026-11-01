---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:02:27.147Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJ59AXM94NWAP75ACFMGXJ
number: 4945
provenance:
  digest: sha256:301639a1ec41a6ee34dce1ed34f44a9c358923f567323a86e58ec3bb96c3e57e
  frontmatter_digest: sha256:ea63bf1032d0c4ac6c5321b488d53675deb9c50f996ba16577ca212346b5c4cb
  log: .bboxy/log/2026-11-01.jsonl#32055
  sig: qAVSlX0PZjp8QsLTs+wMQ4SUR6TprJ0qIF3KjI6NoicwDpB14rbQNoesoRkqhGbcjCrpA1ppwjP/hAsq4MsoBQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: 'Task #4933 (E7) scope includes generated release-input mirrors'
type: decision
updated: 2026-11-01T22:02:27.147Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01. **Task.** #4933 (tasks/2026-11-01-term-harness-menu-branding.md).

**Context.** bboxy450-term `scripts/release-inputs.py` requires `crates/bboxybook/release-inputs/lab/web/components.js` to be byte-identical to `lab/web/components.js`. The branding change edits `lab/web/components.js`, which is in scope. The mirror was not in allowed_paths.

**Decision.** #4933's allowed paths include `crates/bboxybook/release-inputs/**` mirrors of any file the task may change. A mirror changes only through `make sync-release-inputs`, never by hand, and must stay byte-identical to its source.

**Reason.** A generated copy that a gate requires to match is part of the change itself, not a scope expansion. This record binds the ruling to the task, because a DM can't amend task scope.