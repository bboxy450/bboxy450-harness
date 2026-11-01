---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T12:11:22.113Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BGAZF3615HA2PTYSVMZ6HX
number: 3905
provenance:
  digest: sha256:9e2595c43948f004c86c0703fa06be589090e35d170c7132634d1a777061f5a1
  frontmatter_digest: sha256:84f963acb7dc8be9f7a21e787321937fa9fe890adf5f20b6ca36e9653ad50dde
  log: .bboxy/log/2026-11-01.jsonl#24318
  sig: J+isENQHEXjkC8hmr77KjpfW0QsXVjOsYVNqwsvp94CeDX83T0v9cFuh3niTgPi6JVcvsaCXE0E5g7VLbB03AA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: Toolkit 0.1.2 crates.io release waits for the harness's first crates.io release
type: decision
updated: 2026-11-01T12:11:22.113Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01.

**Context.** PR #86 (toolkit widgets) landed on bboxy450-term main as signed squash 59dedc64. The harness pins the toolkit through a `[patch.crates-io]` git rev, which works. Publishing 0.1.2 requires a workspace version bump in bboxy450-term. That bump is a landing of its own, going through land.sh's full gate set (about 55 min), followed by `make publish-crates` and the signed tag and GitHub release.

**Decision.** Don't publish the toolkit 0.1.2 now. The harness re-pins to git rev 59dedc64. The toolkit's version bump and its signed release happen together with the harness's first crates.io release. That is the first point where a crates.io consumer needs a published toolkit, because a published crate can't depend on a git rev.

**Alternatives.** Publish now: that spends a full landing on a release nobody consumes yet, and the version would be stale again by the time the GUI polish and E4 land. Rejected.

**Reason.** Less is more: a release with no consumer is cost without value. The operator's publish authorization stays available for when it's needed.