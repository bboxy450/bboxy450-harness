---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T12:16:34.764Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BGMGVTDRRPJ8WY60J5J20B
number: 4071
provenance:
  digest: sha256:9d0f3efb89b2e8a0091bed99a9d5954dbb32030d220ae20a5b1446cfb8dbda50
  frontmatter_digest: sha256:389fbcbfa7407f7bf9d5097d17048639078ce24b03bbcd9c037ef64d2f5245c5
  log: .bboxy/log/2026-11-01.jsonl#25279
  sig: 2P9Z1hwhZlPSD191uyBFVMW8tpkawlDzwWo3/7MFCY0k7mKEvdUjfnhXneLiODq6gg4WV+tnQ2amy+L0Y7qaCg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: 'ICD #348 amendment is agreed in E4''s bboxy450-term PR, signed off at landing'
type: decision
updated: 2026-11-01T12:16:34.764Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01.

**Context.** embed-seat.md amends bboxy450-term ICD #348 (app ↔ harness). That ICD's owner, Corvin Duskmantle, is not reachable: there's no seat on this account, and the bboxy450-term Architect seat is outside our grants. The changed rows are additions only, listed in embed-seat §7 (harness main 9fdddce).

**Decision.**
- E4's bboxy450-term PR (task #3913) adds the §7 rows to bboxy450-term's app-harness ICD in the same diff as the Side A code.
- The Oracle signs the amendment off when it lands that PR. bboxy450-term's AGENTS.md records the Oracle's merge authorization, operator, 2026-10-31.
- embed-seat stays "proposed" until that landing. The Architect then marks it baselined, citing the landed commit.
- If bboxy450-term's Architect later objects, a delta amendment follows. Nothing waits for that.

**Alternatives.** Hold E4 until #348's owner can be reached. Rejected: it parks the work on an unreachable party, which breaks "never block the swarm". The rows are additive, so a later objection can be fixed forward.

**Reason.** Agreement lives in the reviewed, gated diff that changes both sides, with provenance through a signed landing.