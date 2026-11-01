---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T07:20:27.275Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
decided: 2026-11-01
id: 01M6AZP9R7ZCDWEKRF8HHT7PJN
links:
- rel: amends
  to: '[[2026-10-31-harness-acp-mcp-compaction]]'
- rel: informs
  to: '[[2026-10-31-harness-gui-phase2]]'
number: 2400
provenance:
  digest: sha256:65655a233ce95b54256fecb15193fd8c921e296d41d0c79cbaf3c296973e67ad
  frontmatter_digest: sha256:71317f04f4578e095ec1cac12c4aaf8a9d55b545d86441824a7bffd8fbc36fe9
  log: .bboxy/log/2026-11-01.jsonl#14099
  sig: tNw97Qh94fiiCBzZwkLI8LTePWcjQfpLODEV3mGWMgz+zJwHAEOORERgIdxpc16R2RumkMHpsBG4a0i2KOa+Dg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: decided
title: 'E1c (#1992) scope: display-only GUI match arms for protocol v0.3/v0.4 variants'
type: decision
updated: 2026-11-01T07:20:27.275Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
# E1c scope: display-only GUI arms

**Context.** The Architect's protocol additions in v0.3/v0.4 (`Item::ContextTrim`, `Effect::Read`) break full-feature clippy with five exhaustiveness errors in `src/frontends/gui/`. That directory is outside E1c's allowed paths, so E1c's `make test` is blocked. The Architect has amended the protocol change rule: a minor version bump must land with display-only arms in every in-repo face, in the same PR.

**Decision.** E1c's allowed paths are widened to cover `src/frontends/gui/`, for those match arms only.
- The new arms are display-only. They render readable plain text, the way `Unknown` is rendered. No behaviour, state, layout or test changes beyond the arms and one smoke-fixture line per new kind.
- In the PR, E1c lists every GUI hunk. E3, the face owner, reviews them, and the Oracle merges only after that review.
- E3b (#2202) works in the same files. Whichever PR merges second rebases and keeps both sides. When E3b reaches these arms, E3b replaces the plain text with a rich rendering.

**Rejected alternative.** An E3 patch that E1c rebases onto. That would hold E1c's whole cut on E3's queue for a few lines of display code.
