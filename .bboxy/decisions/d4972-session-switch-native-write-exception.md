---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:25:27.450Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CKFDB177AY6CF7SV8DSPJN
number: 4972
provenance:
  digest: sha256:05c0cdb8f4256dcc37c5ad94a5ea38885305d43ccd5922188f5a82982f67ea90
  frontmatter_digest: sha256:d8396c450cce2c89b5a0efee516855b8749f5e03049a51fa7b3cdc191ad74c9d
  log: .bboxy/log/2026-11-01.jsonl#32232
  sig: NoXOXjXR4eKjDkJVFHUbJX2WqiYZbRqExI/gdsGW1y/noiQ6dhR9okCcaYV/BWNNUJaHx0nFE8Xn24HAAPfxDQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: Session switch may create (never modify) one native session file in ~/.claude or ~/.codex
type: decision
updated: 2026-11-01T22:25:27.450Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01. **Context.** interfaces/session-switch.md §6 (#4963) and bboxy450-term ICD #348 Versioning ("the app never writes ~/.claude or ~/.codex"). A switch into Claude or Codex needs a native session file the target CLI can resume.

**Decision (Oracle sign-off under d4071).** I approve a listed, caused exception:
- The switch-prepare child may create exactly one new session file per switch in the target client's session directory.
- It uses create-exclusive (O_EXCL), mode 0600, with a fresh native id. It never overwrites, appends to, renames or deletes any existing file.
- It never writes Codex's state_5.sqlite or any other client database.
- Each write is recorded in the switch log with its path and cause.

The exception is withdrawn per client as soon as P0 proves that client's own supported entry point works: Codex's session importer, or file-path resume. The exception must then be removed and the #348 row updated.

**Verification.** A switch test asserts that the client directory contains exactly the existing files plus one new file. A broken control lets the switch overwrite an existing file, and it must go red.

**Reason.** Cross-harness switching is an operator requirement. A create-only write of a fresh file can't corrupt the user's existing sessions.