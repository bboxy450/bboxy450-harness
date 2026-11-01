---
allowed_paths:
- src/frontends/gui/
- tests/
- verification/
- scripts/
- .bboxy/runs/
artifacts:
- '[[01M6BM4BVP4G7RTZK682PFGBK5]]'
- '[[01M6BM4HHHZ0JNKT0ET59T061Q]]'
- '[[01M6BM50X8YWZGDFX115XDT5AF]]'
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T13:04:20.292Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BKBZ7Z5P83XZAM05SQ8FEM
number: 4299
provenance:
  digest: sha256:f06b3901ab2532d49f11ce343379ce03fe40026fad44de193f7aadaaa7806ce1
  frontmatter_digest: sha256:009e42bc098e1e587af5441992d1a6766cf1584470a05cca50201c21e01708d9
  log: .bboxy/log/2026-11-01.jsonl#27551
  sig: vzouPQX/4xKu/565l0m5UCeA5iX/D7t9GejqSUsn5nRwAyLY+QGRFjymzlaNpQNmoE2tYC2+C8s7hblGqlivDA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: 31e9af2668db26e9ae4d1e0467d28707b17487f9
  op_id: 01M6BM66V7X1YR7FQBY71FCZ21
  reported_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
  summary: 'E3d complete. PR #10 https://github.com/bboxy450/bboxy450-harness/pull/10 is OPEN on e3d-gui-nits at signed head 31e9af2668db26e9ae4d1e0467d28707b17487f9. Header-sized filled disclosure glyphs, symmetric token gutters and the exact documented three-pattern registry are implemented. Committed report: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/2026-11-01-gui-nits-report.log'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3d: disclosure glyph size, right gutter, record paint_shapes exemptions'
type: task
updated: 2026-11-01T13:23:21.106Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Full unattended unfiltered make test exited 0 on first full run. All 10 canonical and 4 GUI stages passed, including all 15 rich tests and headless smoke. Verified source 7a4cc7a; final commit is records-only and all checked source/model/tool hashes match.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-make-test.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-verification.json
  - criterion: Disclosure glyphs derive their 10.5 px filled geometry from the header's resolved text size. Structural smoke checks actual path ink and slot against the label em, a conservative cap-height bound. Production size=0.2 control fails 101 at DisclosureAtLeastCapHeight and restores to 0.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1627 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/layout_checks.rs#L160 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/disclosure-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/disclosure-green.log
  - criterion: Both card gutters use one density token; worker rich preparation accounts for the inner width. All 22 full prepared cards have equal positive gutters (5 px at 1120 px). Removing only the right inset fails 101 at CardGuttersSymmetric, then passes 0 after restoration.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1092 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/layout_checks.rs#L80 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/gutters-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/gutters-green.log
  - criterion: CONTENTLESS_SHAPE_PATTERNS is the sole documented list for rail/caret/address-dot conversion. Exact-list assertion makes additions explicit. Appending approval/*/badge fails 101 at ContentlessShapePatternsStayExplicit and restores to 0.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1362 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/tests/gui_render.rs#L347 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/shape-list-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/shape-list-green.log
  - criterion: Committed final full and scrolled screenshots show the visible disclosure and symmetric gutters; existing rendered meaning, highlighting, diffs, approval and inset checks remain.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/gui-smoke.png ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/gui-smoke-scrolled.png ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/results.json
  - criterion: All 64 native/TLC complete graphs and verdicts match across 25 contracts/one crate. Committed source/model/tool evidence. Final-head parity refresh and check-formal-evidence exit 0 locally. Both commits verify G with key 9B07E849480C68328CA20684844ECC0587DE7AB0.
    reference: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-formal-summary.json ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-source-signatures.log ; local .bboxy/runs/e3d-final-head-formal-local.log and e3d-final-head-checked-local.log (untracked current-head receipts); git log origin/main..HEAD --format='%H %G? %GF'
  - criterion: New branch from f9dc177, signed head pushed, PR open and not merged. Tracked worktree/source clean; previous untracked logs/local formal archives preserved. Required report is committed as .log.
    reference: https://github.com/bboxy450/bboxy450-harness/pull/10 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/2026-11-01-gui-nits-report.log
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main (f9dc177 or later) and open a PR. Do not merge. These come from the PR #9 review.

1. The ▾ disclosure glyph renders at about 3 px, so it is nearly invisible on the ARGUMENTS/OUTPUT accordion headers. Size it from the header's text size, the same way other glyphs are sized. Add a structural smoke check: the glyph rect is at least the label's cap height. Add a broken control for it.
2. Cards run to x=1120 with no right gutter, while the left side has about 2 px. Make the gutters symmetric from one theme or layout token. Extend the existing MarkdownBlocksInsideCard-style geometry check so that every card has a right gutter at least the left one.
3. `paint_shapes` turns `*/rail`, `composer/text/caret` and `composer/address/*/dot` into shapes, so NoEmptyPaintedBoxes skips them by name. Move that list into one named constant with a comment saying why each entry has no content. Assert in a test that the list is exactly those three patterns, so any new exemption is a visible diff.

Acceptance:
- `make test` exits 0 unattended.
- Each new control goes 101 → 0.
- The refreshed `.bboxy/runs/gui-smoke.png` shows the glyph and the gutter.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-gui-nits-report.log`.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

E3d complete. PR #10 https://github.com/bboxy450/bboxy450-harness/pull/10 is OPEN on e3d-gui-nits at signed head 31e9af2668db26e9ae4d1e0467d28707b17487f9. Header-sized filled disclosure glyphs, symmetric token gutters and the exact documented three-pattern registry are implemented. Committed report: https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/2026-11-01-gui-nits-report.log

```json
{
  "commit": "31e9af2668db26e9ae4d1e0467d28707b17487f9",
  "op_id": "01M6BM66V7X1YR7FQBY71FCZ21",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Full unattended unfiltered make test exited 0 on first full run. All 10 canonical and 4 GUI stages passed, including all 15 rich tests and headless smoke. Verified source 7a4cc7a; final commit is records-only and all checked source/model/tool hashes match.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-make-test.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-verification.json"
      },
      {
        "criterion": "Disclosure glyphs derive their 10.5 px filled geometry from the header's resolved text size. Structural smoke checks actual path ink and slot against the label em, a conservative cap-height bound. Production size=0.2 control fails 101 at DisclosureAtLeastCapHeight and restores to 0.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1627 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/layout_checks.rs#L160 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/disclosure-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/disclosure-green.log"
      },
      {
        "criterion": "Both card gutters use one density token; worker rich preparation accounts for the inner width. All 22 full prepared cards have equal positive gutters (5 px at 1120 px). Removing only the right inset fails 101 at CardGuttersSymmetric, then passes 0 after restoration.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1092 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/layout_checks.rs#L80 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/gutters-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/gutters-green.log"
      },
      {
        "criterion": "CONTENTLESS_SHAPE_PATTERNS is the sole documented list for rail/caret/address-dot conversion. Exact-list assertion makes additions explicit. Appending approval/*/badge fails 101 at ContentlessShapePatternsStayExplicit and restores to 0.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/src/frontends/gui/view.rs#L1362 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/tests/gui_render.rs#L347 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/shape-list-red.log ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/shape-list-green.log"
      },
      {
        "criterion": "Committed final full and scrolled screenshots show the visible disclosure and symmetric gutters; existing rendered meaning, highlighting, diffs, approval and inset checks remain.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/gui-smoke.png ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/gui-smoke-scrolled.png ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-controls/results.json"
      },
      {
        "criterion": "All 64 native/TLC complete graphs and verdicts match across 25 contracts/one crate. Committed source/model/tool evidence. Final-head parity refresh and check-formal-evidence exit 0 locally. Both commits verify G with key 9B07E849480C68328CA20684844ECC0587DE7AB0.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-formal-summary.json ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/e3d-source-signatures.log ; local .bboxy/runs/e3d-final-head-formal-local.log and e3d-final-head-checked-local.log (untracked current-head receipts); git log origin/main..HEAD --format='%H %G? %GF'"
      },
      {
        "criterion": "New branch from f9dc177, signed head pushed, PR open and not merged. Tracked worktree/source clean; previous untracked logs/local formal archives preserved. Required report is committed as .log.",
        "reference": "https://github.com/bboxy450/bboxy450-harness/pull/10 ; https://github.com/bboxy450/bboxy450-harness/blob/31e9af2668db26e9ae4d1e0467d28707b17487f9/.bboxy/runs/2026-11-01-gui-nits-report.log"
      }
    ],
    "exit_code": 0
  }
}
```
