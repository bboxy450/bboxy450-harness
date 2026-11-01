---
allowed_paths:
- src/
- tests/
- Makefile
- scripts/
- verification/
- Cargo.toml
- Cargo.lock
- protocol.schema.json
- .bboxy/runs/
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T06:15:00.321Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6AVYEXPZB5SF1JCQ5V3A2S3
number: 1992
provenance:
  digest: sha256:9ecc6f9ce3b1ff769c26313a8e511c882ee8209f281b7c7638292574b9a1f94f
  frontmatter_digest: sha256:2010830eb4424936ba8303c5bbf8f9cf0e1e54b6be27c2ec355aa5ef314c8fd0
  log: .bboxy/log/2026-11-01.jsonl#11478
  sig: hHd1OsOy+8EcMjOAsJUO3WlNU33Pv35n7E8KOWUGwrMLGCJsymZ6tYS0A1VSLt1VHU34IXuEvisI1yP1IcEgDw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1c: ACP face, MCP client, automatic compaction (protocol v0.3)'
type: task
updated: 2026-11-01T06:15:03.252Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
# E1c: ACP face, MCP client and automatic compaction

ROLE: EXECUTOR (Codex).

**Worktree:** `/Users/feral/mydev/bboxy450-harness-core`.
1. Fetch.
2. Create branch `e1c-acp-mcp-compaction` from `origin/main` at or after `c745ad9`.
3. Push the branch and open a PR. Do not merge.

E3 is rebasing the GUI (`src/frontends/gui/`) in parallel. Do not touch that directory.

## Binding specs
The ICD wins over this brief.
- `.bboxy/interfaces/acp.md` (new)
- `.bboxy/interfaces/mcp.md` (new)
- `.bboxy/interfaces/protocol.md` §4d (v0.3)
- `.bboxy/interfaces/gateway.md` §1b
- `.bboxy/interfaces/interfaces.md` rows R22–R24, plus models HarnessAutoCompact and HarnessMcpServer
- `.bboxy/specs/budgets.md`
- the invariants

## Outcome
1. **ACP pin check first.** The ACP facts in `acp.md` were read from schema 1.9.1, but the crate we pin (3.4.0) uses schema 1.12.0.
   - Before coding, check every fact `acp.md` relies on against 1.12.0.
   - Write the differences to `.bboxy/runs/acp-1.12-diff.md`.
   - Message the Architect only if a difference changes the mapping. Otherwise proceed.
2. **ACP face** (feature `acp`): `bboxy450-harness acp` over stdio, per `acp.md`.
   - Map items to `session/update`.
   - Send approvals as `request_permission`.
   - Implement `session/load` through the store.
   - Unsupported methods degrade as the ICD specifies.
   - **Witness:** a scripted ACP client test drives a full turn against the mock provider, including tool, diff, terminal, plan and approval. `check_transcript` must hold.
3. **MCP client**, per `mcp.md`.
   - Servers live as long as the session.
   - Tools enter the registry, with their concurrency class taken from `readOnlyHint`.
   - Every call goes through `policy.decide`.
   - Each server has a timeout.
   - A dead or slow server produces a labelled Notice and never blocks a turn.
   - Secrets go through the redactor.
   - **Witness:** a fixture MCP stdio server, as a test binary. Cover: a read-only call running in parallel, a write call that needs approval, the server crashing mid-call, and the server hanging past its timeout.
4. **Automatic compaction**, per protocol §4d: the trigger, the ladder (clip, then drop, then summarize), and a cache-stable prefix. This is protocol v0.3; regenerate the schema.
5. **Formal models.** HarnessAutoCompact and HarnessMcpServer, each with the broken control named in `interfaces.md`.
   - The invariants must be stated over state, not over a flag that only broken actions set.
   - Previous reviews found exactly that failure mode. Reviewers will check for it.
6. **Live smoke.** Extend `make smoke-live` with one ACP-driven turn.
   - Use the same nemotron free profile.
   - At most 3 more requests.
   - Report the outcome as SKIP, UNAVAILABLE or PASS, honestly.

## Verify
- `make test` runs unattended and exits 0.
- No wall-clock gates.
- Show the red→green progression for every model control and for the MCP failure witnesses.

## Report
Send one `task_report`. Include files and mechanisms, commands with exit codes, and the branch with its signed head.
