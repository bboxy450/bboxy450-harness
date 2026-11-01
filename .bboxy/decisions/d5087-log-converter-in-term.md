---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T23:04:02.662Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CNP27VW05JMXAKE85JHZD8
number: 5087
provenance:
  digest: sha256:d844dcb6e1c704a8e24edb7d1dd2179596c8ade200f2b3fe81d66986d316d15c
  frontmatter_digest: sha256:2029fda22d58afe2136f08e5ae2dfbb93fe27d6f1ca5a6c1f0c75fadc9bb8da0
  log: .bboxy/log/2026-11-01.jsonl#33264
  sig: /Rq7Pubqqr40L5imz0Uk28IF7ysQMMWR9JDOCr18dZTUnCvpH5XPp6I7VOLxVNq43g1eXZaDJllpfzrLGVlNCw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: bboxy450-log-converter is a crate in bboxy450-term; no new repositories (supersedes d4964)
type: decision
updated: 2026-11-01T23:04:02.662Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01.

**Operator instruction (verbatim):** "no new repos, all crates in bboxy450-term repository, everything is a crate in there!"

**Decision.**
- The converter lives at `bboxy450-term/crates/bboxy450-log-converter` as a workspace member. This supersedes d4964, which made it a separate git-pinned repository.
- It joins bboxy450-term's `make test`, its formal inventory (Rust TLC with Java parity), its secret scan and its landing gates. It lands through `scripts/land.sh` under bboxy450-term's recorded Oracle authorization.
- The bboxy450-harness submodule may depend on the converter by path inside bboxy450-term only through the existing feature-guard rules. The harness stays a separately buildable crate; its standalone build then needs the converter through a git pin or path, which is a P5 detail.
- Work already done in bboxy450/bboxy450-log-converter moves into bboxy450-term as PRs: P0 probes (PR #2, 7945ad92), P1 crate and IR (PR #1, bb87743b), and the in-flight P2 and P4. Their verified content is ported as is; it is re-verified under bboxy450-term's gates.
- The bboxy450/bboxy450-log-converter repository is retired once the port lands. Deleting or archiving it waits for the operator's confirmation.
- No new repositories are created for future components either. New components are crates in bboxy450-term.

**Reason.** Operator instruction: one repository for the suite.