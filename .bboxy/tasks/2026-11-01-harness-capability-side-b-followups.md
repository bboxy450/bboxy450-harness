---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- .bboxy/runs/
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T21:11:35.908Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CF85M6MAJT9NMJEW79VK5Y
number: 4921
provenance:
  digest: sha256:cd3f2e82b0d5cad315e0a4069c61b1b0a2d317cd5e6c96d0ef33cd9e78a91e3e
  frontmatter_digest: sha256:790fd9a164a17716f19ac72d9663c6cd9000457981169140442cd1f62e44f5bd
  log: .bboxy/log/2026-11-01.jsonl#31837
  sig: oMhBE0+MZyRVzF5S2A5gmPosOV5MqqS2IREz+SgvBLSwjwQ+QbjX27tK5weqrLOGVh9AZByY4BEB5tDhxz7uCA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3f: Side B follow-ups (live GUI witness, per-face controls, case (c), quiet when unset)'
type: task
updated: 2026-11-01T21:18:51.866Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main, after PR #18 has merged, and open a PR. Do not merge. These come from the Oracle's security review of PR #18; no leak was found.

1. **Live GUI face witness.** Nothing tests live-GUI registration at gui/bridge.rs:157; deleting that line would go undetected. Add a headless test that starts the live GUI bridge with a fixture capability file and asserts the redaction set contains the value before Host starts. Add a broken control that deletes the registration and must go red.
2. **Per-face broken controls.** scripts/test-capability-control.py covers only the headless face. Extend it so that, for each face (headless, seat, ACP, GUI replay, GUI live, smoke), reverting that face's loader goes red at its named assertion.
3. **Case (c), file-path-specific.** Add an echo_env test asserting that profile and ACP MCP servers and tools never see `BBOXY_CAPABILITY_FILE`, while launcher `--mcp-config` servers do.
4. **Quiet when unset (Oracle ruling).** When `BBOXY_CAPABILITY_FILE` is unset (a non-seat run such as `-p` or the GUI), emit no Notice. Emit the typed Notice only when the variable is set and the file fails validation. Test both cases. I'm asking the Architect to align the ICD.
5. **ICD citation.** In your report, say the ICD §4 harness row should cite tests/capability_delivery.rs. The Architect edits the ICD.

Acceptance:
- `make test` exits 0 unattended.
- The controls go 101 → 0.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-capability-side-b-followups-report.log`. Don't write "key:" directly before any hex value.
- Send one task_report with verify_result.command exactly `make test`.