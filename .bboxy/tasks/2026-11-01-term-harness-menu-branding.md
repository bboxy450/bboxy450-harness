---
allowed_paths:
- crates/bbt-ui/
- crates/bbt-render/
- crates/bboxy450-term/
- crates/bbt-proto/
- crates/bbt-mux/src/adapters/
- lab/
- scripts/
- tests/
- Makefile
- verification/
- .bboxy/interfaces/
assignee: seat:01M6ADTVVZS36VS43P2PG15HRQ
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T21:45:01.945Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CH5CMQJQV6NXPEV2K3VTCT
number: 4933
provenance:
  digest: sha256:1e1c16ce9cb017e378364e9422af3e7505f25bba332bc4feb6b5b983deee15ac
  frontmatter_digest: sha256:7186ad160864855881643a5646ff546c4a9b0c7f3fc1463f2019121549e53625
  log: .bboxy/log/2026-11-01.jsonl#31976
  sig: 46mH+q/GHaUVeDGBZHseFXR+nPWFsPHNmyJCeFdHLOq4FL14mQsqFVpfc3aHIhVofI6hQvIsmc32knFL8JCGCQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E7: built-in harness as a first-class seat in bboxy450-term menus, with the bboxy logo'
type: task
updated: 2026-11-01T21:47:45.951Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Work on a new worktree and branch `harness-branding` from origin/main, and open a PR. Do not merge or run land.sh.

Operator request: in bboxy450-term, the built-in harness must appear wherever Claude and Codex appear, in the seat or new-seat menu, the settings harness list and seat rows. It uses OUR logo, the bboxy fox, not a generic one, and is named correctly. This is full integration.

Facts:
- The kind id stays `bboxy450-harness` (settings/roles.rs:18 `HARNESSES`; adapter at bbt-mux adapters/mod.rs:81).
- The fox is already embedded: assets.rs:14 loads `lab/web/bboxy.png` as image `bboxy`. The canonical face is `~/.agents/identity/bboxy.png`; reuse the embedded asset and do not generate or substitute a new image. If menus need an SVG or icon glyph, as the Claude/Codex marks use via `BAKED_ICONS` (build.rs:34), derive it from the existing bboxy asset or fall back to the PNG. Don't draw a new logo.
- Hard-coded five-kind lists that omit the harness: bbt-ui modules/settings.rs:2675 and settings/controller.rs:102. Per-kind colour and logo: bbt-ui modules/seat_row.rs:421. Find every other place where claude/codex get a logo, label or colour (launcher and new-seat menus, the spawn picker, seat rows, HUD, sidebar) and give bboxy450-harness the same treatment.

Naming: show the display label "bboxy harness", in the same case style as its siblings in each menu. Never show the raw id `bboxy450-harness` or "generic harness". If the siblings show a product name (for example "Claude Code"), use "bboxy harness" in the same style. State in the report exactly what each surface shows.

Behaviour: picking it from the menu launches a working harness seat (phase 1a from E4: `bboxy450-harness seat` with the --append-system-prompt and --mcp-config wiring already in launch.rs). If the menu needs the harness binary on PATH, make the missing-binary refusal message say how to install it.

Evidence:
- Unit tests asserting that every surface listing claude/codex also lists bboxy450-harness, with the bboxy image and the label. Make this table-driven from `HARNESSES`, so a new kind can't be silently omitted again. The broken control removes the harness from one surface and must go red.
- A headless render or screenshot test of the menu showing the fox mark (use the repo's existing GUI screenshot or scene tooling). Commit the screenshot under .bboxy/runs/ (in bboxy450-term).
- The existing native harness seat test still passes.

Acceptance:
- Full `make test` exits 0 at default concurrency.
- All commits are signed, with no Report trailer.
- The report goes in bboxy450-harness `.bboxy/runs/2026-11-01-harness-branding-report.log`. Tell me the commit. Don't write "key:" before hex.
- Send one task_report with verify_result.command exactly `make test`.