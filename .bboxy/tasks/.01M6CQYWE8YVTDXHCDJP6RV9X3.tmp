---
allowed_paths:
- site/
- lab/web/
- crates/bboxybook/release-inputs/
- Makefile
- scripts/
- tests/
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T23:43:47.188Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CQYTSSCPVEXXC8MSFAXCDP
number: 5110
provenance:
  digest: sha256:e08220dca6a8665add33166540dbc60e8d3bfcc794bb9b2184a08014422c1a26
  frontmatter_digest: sha256:ab4717ee1c50852ace5d5f5884bbe697f0b563c1d94b0a21858e2c46660b4692
  log: .bboxy/log/2026-11-01.jsonl#33654
  sig: 5AcGsiqBZetSaUKT0oJSRpj8ZzwS57chG8KZpkx1I07+j5pWtg9BnOhEKSjnAGl2hgG7o3e5+V/ahr2FXKkABA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E9: website demo — sanitized refresh and three panes (bboxy450-harness, Claude, Codex)'
type: task
updated: 2026-11-01T23:43:47.188Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Start after the converter's P3 (task #5104, Claude parse plus sanitize) is on origin/log-converter. Worktree: `git -C ~/mydev/bboxy450-term worktree add ~/mydev/bboxy450-term-site -b site-panes origin/main`, rebased onto the converter branch as needed. Open a PR. Do not merge, do not deploy, do not run land.sh.

Operator request (2026-11-01): "deploy from this machine locally coz logs, make website refresh script sanitize logs then one of the panes on webiste shoud be boxy harness, one claude and one codex". Deployment stays local and manual; the Oracle runs `sh site/refresh.sh && cd site && npx wrangler deploy`. Never add a CI deploy.

Scope:
1. `site/refresh.sh` and `site/extract.py`:
   - Every session that goes into `site/real.js` is first sanitized with the converter (`bboxy450-log-converter` CLI: parse then `--sanitize`, per d4947: secrets, home paths, emails, hosts and IPs, ids, images, 4 KiB truncation).
   - Real content never reaches real.js unsanitized.
   - Add a check that refresh.sh fails if the sanitizer is unavailable, so nothing leaks through.
   - Add a scan of the generated real.js for secret shapes, `/Users/`, emails and `sk-` that fails the refresh.
2. The site shows exactly three session panes: one **bboxy450-harness** session (a real local harness session, sanitized; the label is "bboxy450-harness", per d5089, with the fox logo), one **Claude** session and one **Codex** session. Each pane uses that harness's own branding. refresh.sh picks the sessions explicitly by configurable path, not hard-coded old paths, with sensible defaults (the latest session of each kind).
3. Tests in make test, all on synthetic fixtures:
   - extract plus sanitize produces three panes;
   - the leak scan rejects a planted secret (broken control: skip the sanitizer and the scan must go red);
   - site/build.sh output contains the three panes.
4. Keep the copy of site files under bboxybook release-inputs in sync through `make sync-release-inputs` if it is affected.

Acceptance:
- Full `make test` exits 0.
- All commits are signed.
- The report goes in bboxy450-harness `.bboxy/runs/2026-11-01-site-panes-report.log` (tell me the commit).
- Send one task_report with verify_result.command exactly `make test`.

The Oracle reviews the regenerated real.js content in full before deploying.