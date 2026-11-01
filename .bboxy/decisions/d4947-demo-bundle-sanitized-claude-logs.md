---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:04:00.062Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJ842YRF887AFA4FGFW3C5
number: 4947
provenance:
  digest: sha256:62350745c277009f094f5856798bad9e292c95951df195c32d3926e990a09c12
  frontmatter_digest: sha256:7137d39c7727bf5802716f98a364b15ca75647e5cfb5484e52e969c4deecf6db
  log: .bboxy/log/2026-11-01.jsonl#32091
  sig: PS8lb45zCYbquVBZApNGgrU32jUv2jNCaR8VIGIMSAjn4WhbrCsqBZ7Jz0GffVWn7lmv0F767qUl2ABlkk/AAg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: 'Sanitized real Claude sessions may be bundled into the harness demo (#4935, #4939)'
type: decision
updated: 2026-11-01T22:04:00.062Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01. **Tasks.** #4935 (import::claude), #4939 (bboxybook harness demo).

**Operator instruction (verbatim):** "clean up logs and shit sanitize them but you can bundle them into a demo, for shits and giggles".

**Decision.** This replaces the "never commit real log content" rule in #4935 and #4939 with "commit only sanitized content".
1. **#4935 adds a sanitizer**, `bboxy450-harness import claude --sanitize`, which runs after conversion. It must:
   - remove secrets: every redactor pattern, API-key shapes, the seat capability, bearer and authorization headers, and private keys or PEM blocks;
   - replace the home directory `/Users/<name>` with `~`;
   - replace email addresses, hostnames and IPs with placeholders;
   - replace UUIDs, ULIDs and session ids with stable fakes;
   - drop image payloads, keeping metadata only;
   - truncate tool outputs over 4 KiB.
   It must be deterministic, and it gets a planted-secret test plus a broken control.
2. **#4939 bundles one to three sanitized sessions** as demo fixtures under the bboxybook harness section, chosen for a variety of item kinds. Every bundled file must pass:
   - `--sanitize`;
   - the harness `checks.py secrets` and bboxy450-term's secret scans;
   - a grep showing no `/Users/`, `@`-emails or `sk-` shapes;
   - check_transcript.
3. **Oracle review.** Before merge, the Oracle reads the bundled fixtures in full and refuses anything personal or sensitive. Files outside this repo's work, such as other projects' sessions, are excluded unless obviously harmless. Synthetic fixtures still drive the unit tests.

**Reason.** The operator owns the data and explicitly allows this. Sanitizing in code plus a human-equivalent review keeps "secrets never persist" intact.