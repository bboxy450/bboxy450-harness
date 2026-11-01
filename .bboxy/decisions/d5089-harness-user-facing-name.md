---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T23:04:49.079Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CNQFKEXS9XS9YWZYF255HX
number: 5089
provenance:
  digest: sha256:f0c7fb97c301401ad7296e4a5535dcf6a08a322fc26fb575a4e2d667e20589e5
  frontmatter_digest: sha256:0afbc9698b4851079e8aef271d7cbb5a80db992fe670da1e7e1447f8fc582bcd
  log: .bboxy/log/2026-11-01.jsonl#33290
  sig: PmapuiuBn6+GECTISabS5DNtRS6FkCtVY8kJ27p2/E2ZPsbzeuekjf9xTgPtfbLE5y1EGK5Fg8nu+SYwfy4KDg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: User-facing name of the harness is bboxy450-harness
type: decision
updated: 2026-11-01T23:04:49.079Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01.

**Operator instruction (verbatim):** "currently we call tert bboxy450-term now that i think about it, harness should be called bboxy450-harness, you were correctl". The operator then confirmed: "all is correct".

**Decision.** Every user-facing surface shows "bboxy450-harness": bboxy450-term menus, settings, seat rows, HUD, tooltips, the missing-binary message, bboxybook, and docs and specs. This matches the "bboxy450-term" naming. It supersedes the earlier interim labels "bboxy harness" and "Bboxy". The kind id was already `bboxy450-harness`, so no display mapping exists. The logo stays the bboxy fox.

**Verification.** A test in task #4933 asserts the label and that neither "bboxy harness" nor "Bboxy" appears on any user-facing surface.