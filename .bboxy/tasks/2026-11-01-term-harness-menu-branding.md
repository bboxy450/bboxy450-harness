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
artifacts:
- '[[01M6CMHGXPE887W6MBEHWF1TMJ]]'
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
  digest: sha256:8945b47e23c196be0c4987da95bb3635814d0e2bbfbda72129f2db6dc0a24394
  frontmatter_digest: sha256:43b3c0877bc04b38d384e9155d892250767845efaebb615e41badeeda43c59d9
  log: .bboxy/log/2026-11-01.jsonl#32840
  sig: VU64Kb/4Br5M6p5KQ0KoMtxOjai+nxUVEA7Mc7tKn4uPd50eW85eydhyjygvw7kpdUEZQAgIHFpi3DB3GrgiAA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Round 2, the operator rename only. The fox logo and placement in PR #90 look right. But the user-facing label is still "bboxy harness" (bbt-ui modules/harness.rs:29,41, bbt-mux adapters/mod.rs:86, and the test at harness_branding_tests.rs:267). The menu screenshot at .bboxy/runs/2026-11-01-harness-branding-menu.png shows "bboxy harness".

    The operator ruled, in my message 01M6CJK97H: the user-facing name is just "Bboxy" everywhere. That covers menus, settings, seat rows, the HUD, tooltips and the missing-binary message. The kind id stays `bboxy450-harness`.

    Fix:
    - Change every user-facing string to "Bboxy".
    - Make the table-driven test assert "Bboxy", and add a check that no user-facing surface shows "bboxy harness" or `bboxy450-harness`.
    - Regenerate the screenshot.
    - Run full `make test`, push the signed commits, update the report, and send task_report.
report:
  commit: 533b6641a946e9defe836c5a9db519a500c065d4
  op_id: 01M6CMM0J2EDXB54JBJVXFMK8G
  reported_by: seat:01M6ADTVVZS36VS43P2PG15HRQ
  summary: 'E7 completed: PR #90 is OPEN/CLEAN at signed 533b6641a946e9defe836c5a9db519a500c065d4 (https://github.com/bboxy450/bboxy450-term/pull/90). Full make test exits 0 at default concurrency, unattended; make test-boundaries and existing geometry parity also exit 0. Full report .bboxy/runs/2026-11-01-harness-branding-report.log is committed alone on harness main as signed 177fcd3e491546391eb7f96981dbf9f5431d6061 (SHA256 12c20b4a3f903177acc28e130b70aa2aa87919a3c1dc42940ed2f8ca0c985df6). The report gives every display surface, mechanisms, exact commands/exits, evidence and limits. Native/TLC receipt covers 205 models/32 crates/84 contracts. Binding mirror scope: d4945. No author merge, land.sh landing, release or Report trailers.'
role: executor
status: rejected
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E7: built-in harness as a first-class seat in bboxy450-term menus, with the bboxy logo'
type: task
updated: 2026-11-01T22:45:57.922Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Builtin harness is present wherever harness kinds are offered, with the requested display label and existing fox.
    reference: '533b6641: bbt-ui modules/harness.rs:10,20,39,46,60; app settings/roles.rs:18 re-exports HARNESSES; main.rs:1204,1916,2118,2913 projects launcher/split/machine/Settings lists; settings/controller.rs:102 accepts all members. Settings cards and notification rows say bboxy harness. Sidebar/pane/closed/pending/graph use the same existing bboxy PNG; full per-surface text is in the committed report.'
  - criterion: Table-driven tests and a deliberately failing omission control.
    reference: crates/bboxy450-term/src/harness_branding_tests.rs:47,154,163,332; full CI lines1720-1725 pass actual controller, split, remote resolver, Settings and seat layouts for every HARNESSES member. Final make test-harness-branding-control exits0 only after cargo exits101 with broken picker omits bboxy450-harness. /tmp/branding-control-final.log.
  - criterion: Headless menu shows the original fox and is committed in bboxy450-term.
    reference: https://github.com/bboxy450/bboxy450-term/blob/533b6641a946e9defe836c5a9db519a500c065d4/.bboxy/runs/2026-11-01-harness-branding-menu.png ; SHA256 6a82fadf3dce65377081172e54013976f6087731c240e566d818bc8c2d66361c. harness_branding_tests.rs:262 renders actual Scene/VerificationFrame and checks bitmap pixels; 250 colours/201 blue pixels in the17px mark. Visual inspection completed; normal full unit suite passes this render test.
  - criterion: Menu launch remains a working phase-1a process seat and missing executable names a build/PATH route.
    reference: spawn_params.rs:173 preserves builtin seat argument; :213 resolves all machine kinds. The menu test compares resolved executable/argv against each builtin. adapters/launch.rs:135,268 private missing-executable process test exits127 with make build-harness/PATH instructions. Full CI line4941 Ctrl-D code0/18.772125ms; line4943 owner exit Killed:9 observed/268.613833ms, every group ESRCH, completed turn, guarded refusal zero writes, manual recovery intact and check-transcript0. Existing startup-prompt/MCP wiring exercised by the scripted native case.
  - criterion: Full unattended gate, formal inventory and process/visual regressions are green.
    reference: target/harness-branding-ci-green.log SHA256 adb7ed877277fa1f11a92a7f398873a72b016724e82791e920728855ac027009 ends exact533b6641 normal0 (lines5864-5866);27 invariants0 failures11 standard slow skips. make test-boundaries0 (/tmp/branding-boundaries.log). PARITY_OUT=target/parity-branding-final make parity-settings0:472 cases;416 required/measured geometry;0 mismatches/unproven;48 legacy harness-icon pixel advisories. Frozen web b0565b65 has identical UI/book/lab inputs through final533b6641.
  - criterion: Formal checks describe the current clean source; negative controls retain expected failures.
    reference: make test-formal0 in full gate; make check-formal-evidence0. Clean source533b6641 receipt SHA256 ddd85e029c6a5583fd4a46f7613b3cfd28a7e5abbd155e075ef83ace6df6eb6c:205 models/32 crates/84 contracts, native/TLC graph/verdict parity. Pure catalogue/PNG checks are recorded as implementation coverage; no new stateful protocol was introduced.
  - criterion: Signed branch/PR and one committed report in the required repository, without merging.
    reference: make push-branch0 verified3 signatures, formal receipt and exact-HEAD CI; origin/harness-branding matches533b6641; PR90 OPEN/CLEAN. Signer fingerprint 9B07E849480C68328CA20684844ECC0587DE7AB0. Report commit177fcd3e491546391eb7f96981dbf9f5431d6061 changes only .bboxy/runs/2026-11-01-harness-branding-report.log and is signed. No Report trailers, AGENTS/schema edits or author landing. d4945 authorizes generated release-input mirrors; make sync-release-inputs keeps10 copies identical.
  - criterion: Limits and artifact handoff are recorded honestly.
    reference: No required acceptance gap remains; standard slow/scale checks and external provider/remote-host execution are not claimed, phase1b unchanged. PR artifact claim01M6CMHGXPE887W6MBEHWF1TMJ stored. Image/report responsibility claims were refused workspace_pending (ops01M6CMHJ64C9S7XZ65KKY8WS40 and01M6CMHJ6ZJ212EV96A9XZPRZH); operation_status reports ledger missing/unknown and retry_allowed=false, so they were not redispatched. The screenshot/report bytes and signed commits are verified independently above.
  exit_code: 0
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

## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

E7 completed: PR #90 is OPEN/CLEAN at signed 533b6641a946e9defe836c5a9db519a500c065d4 (https://github.com/bboxy450/bboxy450-term/pull/90). Full make test exits 0 at default concurrency, unattended; make test-boundaries and existing geometry parity also exit 0. Full report .bboxy/runs/2026-11-01-harness-branding-report.log is committed alone on harness main as signed 177fcd3e491546391eb7f96981dbf9f5431d6061 (SHA256 12c20b4a3f903177acc28e130b70aa2aa87919a3c1dc42940ed2f8ca0c985df6). The report gives every display surface, mechanisms, exact commands/exits, evidence and limits. Native/TLC receipt covers 205 models/32 crates/84 contracts. Binding mirror scope: d4945. No author merge, land.sh landing, release or Report trailers.

```json
{
  "commit": "533b6641a946e9defe836c5a9db519a500c065d4",
  "op_id": "01M6CMM0J2EDXB54JBJVXFMK8G",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Builtin harness is present wherever harness kinds are offered, with the requested display label and existing fox.",
        "reference": "533b6641: bbt-ui modules/harness.rs:10,20,39,46,60; app settings/roles.rs:18 re-exports HARNESSES; main.rs:1204,1916,2118,2913 projects launcher/split/machine/Settings lists; settings/controller.rs:102 accepts all members. Settings cards and notification rows say bboxy harness. Sidebar/pane/closed/pending/graph use the same existing bboxy PNG; full per-surface text is in the committed report."
      },
      {
        "criterion": "Table-driven tests and a deliberately failing omission control.",
        "reference": "crates/bboxy450-term/src/harness_branding_tests.rs:47,154,163,332; full CI lines1720-1725 pass actual controller, split, remote resolver, Settings and seat layouts for every HARNESSES member. Final make test-harness-branding-control exits0 only after cargo exits101 with broken picker omits bboxy450-harness. /tmp/branding-control-final.log."
      },
      {
        "criterion": "Headless menu shows the original fox and is committed in bboxy450-term.",
        "reference": "https://github.com/bboxy450/bboxy450-term/blob/533b6641a946e9defe836c5a9db519a500c065d4/.bboxy/runs/2026-11-01-harness-branding-menu.png ; SHA256 6a82fadf3dce65377081172e54013976f6087731c240e566d818bc8c2d66361c. harness_branding_tests.rs:262 renders actual Scene/VerificationFrame and checks bitmap pixels; 250 colours/201 blue pixels in the17px mark. Visual inspection completed; normal full unit suite passes this render test."
      },
      {
        "criterion": "Menu launch remains a working phase-1a process seat and missing executable names a build/PATH route.",
        "reference": "spawn_params.rs:173 preserves builtin seat argument; :213 resolves all machine kinds. The menu test compares resolved executable/argv against each builtin. adapters/launch.rs:135,268 private missing-executable process test exits127 with make build-harness/PATH instructions. Full CI line4941 Ctrl-D code0/18.772125ms; line4943 owner exit Killed:9 observed/268.613833ms, every group ESRCH, completed turn, guarded refusal zero writes, manual recovery intact and check-transcript0. Existing startup-prompt/MCP wiring exercised by the scripted native case."
      },
      {
        "criterion": "Full unattended gate, formal inventory and process/visual regressions are green.",
        "reference": "target/harness-branding-ci-green.log SHA256 adb7ed877277fa1f11a92a7f398873a72b016724e82791e920728855ac027009 ends exact533b6641 normal0 (lines5864-5866);27 invariants0 failures11 standard slow skips. make test-boundaries0 (/tmp/branding-boundaries.log). PARITY_OUT=target/parity-branding-final make parity-settings0:472 cases;416 required/measured geometry;0 mismatches/unproven;48 legacy harness-icon pixel advisories. Frozen web b0565b65 has identical UI/book/lab inputs through final533b6641."
      },
      {
        "criterion": "Formal checks describe the current clean source; negative controls retain expected failures.",
        "reference": "make test-formal0 in full gate; make check-formal-evidence0. Clean source533b6641 receipt SHA256 ddd85e029c6a5583fd4a46f7613b3cfd28a7e5abbd155e075ef83ace6df6eb6c:205 models/32 crates/84 contracts, native/TLC graph/verdict parity. Pure catalogue/PNG checks are recorded as implementation coverage; no new stateful protocol was introduced."
      },
      {
        "criterion": "Signed branch/PR and one committed report in the required repository, without merging.",
        "reference": "make push-branch0 verified3 signatures, formal receipt and exact-HEAD CI; origin/harness-branding matches533b6641; PR90 OPEN/CLEAN. Signer fingerprint 9B07E849480C68328CA20684844ECC0587DE7AB0. Report commit177fcd3e491546391eb7f96981dbf9f5431d6061 changes only .bboxy/runs/2026-11-01-harness-branding-report.log and is signed. No Report trailers, AGENTS/schema edits or author landing. d4945 authorizes generated release-input mirrors; make sync-release-inputs keeps10 copies identical."
      },
      {
        "criterion": "Limits and artifact handoff are recorded honestly.",
        "reference": "No required acceptance gap remains; standard slow/scale checks and external provider/remote-host execution are not claimed, phase1b unchanged. PR artifact claim01M6CMHGXPE887W6MBEHWF1TMJ stored. Image/report responsibility claims were refused workspace_pending (ops01M6CMHJ64C9S7XZ65KKY8WS40 and01M6CMHJ6ZJ212EV96A9XZPRZH); operation_status reports ledger missing/unknown and retry_allowed=false, so they were not redispatched. The screenshot/report bytes and signed commits are verified independently above."
      }
    ],
    "exit_code": 0
  }
}
```
