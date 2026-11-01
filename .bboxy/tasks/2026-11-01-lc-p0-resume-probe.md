---
allowed_paths:
- probes/
- tests/
- scripts/
- Makefile
- .bboxy/
assignee: seat:01M6CKGX6ME342X90F0N1YA5TE
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:26:05.696Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CKGJHWD73N4ZT1H7SY2GJM
number: 4974
provenance:
  digest: sha256:69284423b1e23453111efbc0864309965b501544f3077cfa779e62735438f70f
  frontmatter_digest: sha256:68164849e022a11beec99ce10e440e4ba623aa3d37bcb32b9a55313e2973ab73
  log: .bboxy/log/2026-11-01.jsonl#32250
  sig: KnnATZ56IDlCxcgau42QtNTBAinCSB27JRM0M/bcEXfK8+ryfyAtS/m0dn8XTXj3SovrJF4WsGXYsDFeKsbkBQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'LC-P0: unattended native resume probe (Claude, Codex) with mock endpoints'
type: task
updated: 2026-11-01T22:26:21.381Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-log-converter (clone at ~/mydev/bboxy450-log-converter, which has only records). Work on a new branch `p0-resume-probe` and open a PR. Do not merge.

Binding: bboxy450-harness `.bboxy/specs/log-converter.md` (#4965) phase P0 and §7 verify, `interfaces/session-switch.md` (#4963), research #4953 and decision d4972. Read them first.

Goal: prove with exit codes which synthesized native sessions each client actually resumes. Run fully unattended and with NO real credentials:
- a throwaway HOME, CLAUDE_CONFIG_DIR and CODEX_HOME;
- local mock Anthropic and Responses endpoints;
- dummy keys.

The installed CLIs are Claude Code 2.1.304 and codex-cli 0.164.0. Never touch the real ~/.claude or ~/.codex, and never print a credential.

Probes, each a script with a verdict. The verdict is CLI exit 0 AND the mock's captured request contains turn 1:
1. A Claude two-turn synthesized session, resumed by id and by file path.
2. Whether Claude needs api-request records.
3. Codex: a hand-written rollout resumed with no state_5.sqlite entry, through the filename fallback or backfill.
4. Codex's own Claude session importer (app-server external_agent_migration). Find whether a CLI or app-server entry point exists and whether it imports our synthesized Claude session.
5. Whether Codex accepts reasoning{summary} with no encrypted_content.

Each probe also gets a negative case that must fail: a malformed session (Claude: broken parentUuid chain; Codex: missing session_meta first line), so the verdict is not vacuous.

Deliverables:
- `probes/` scripts.
- `make test-probes`, which skips cleanly with a labelled reason if a CLI is absent and never hangs.
- A results table in the report. Each finding becomes a fact for P3, P4 and P6, and says whether d4972's file-write exception can be withdrawn for that client.

Acceptance:
- `make test` runs the probes plus any repo checks and exits 0 unattended.
- All commits are signed.
- Every file carries the ATTRIBUTION.md header.
- The report goes in `.bboxy/runs/p0-resume-probe-report.log` in this repo.
- Send one task_report with verify_result.command exactly `make test`.