---
id: "01M6FGCPJH9EVWQ2Z5DYDAFD4S"
number: 202611021800
type: report
title: "Named Space dispatch in the composer and profile search"
created: "2026-11-02"
owner: "Codex Executor, Oracle Vesper Greywind's space-fix brief"
---

Worktree: `/Users/feral/mydev/bboxy450-harness-space`; branch `fix/composer-space`;
base `082239d588e0624d212b64b2051a1f8e8877843c`. Direct Oracle brief: fix named
Space in both GUI text-entry arms, audit other paths, add dispatcher regressions
and the old-match control, run GUI tests, both Clippy configurations, then one
full `make test`. No push or merge is authorized by this assignment.

## Implementation and audit

- `src/frontends/gui/window.rs:443` — the open profile switcher accepts
  `Key::Character(_) | Key::Named(NamedKey::Space)` and forwards the supplied
  event text through the existing control-character filter. `!shortcut` remains.
- `src/frontends/gui/window.rs:514` — the composer accepts the same logical-key
  alternatives, still inserts the supplied event text, and retains both
  `!shortcut` and `!view.composer.composing()`.
- `src/frontends/gui/window.rs:334` — pressed winit events supply their logical
  key and `event.text` to this same public dispatcher. These are the only native
  character-admission arms (`rg -n 'Key::Character|NamedKey::Space|event\.text|KeyboardInput|Ime::' src tests --glob '*.rs'`).
- `src/frontends/gui/editor.rs:43` — `/` completion is derived from the composer
  draft, so it shares the corrected insertion path. The pinned toolkit editor
  ends its completion token at whitespace; arguments then remain ordinary draft
  text. There is no separate slash-filter key match to repair.
- `src/frontends/gui/window.rs:337` — IME commits already forward their string
  through `Edit::Commit`. Completion edits and `MenuKey::Text` also take strings
  directly. `src/frontends/headless.rs:460,536` and
  `src/frontends/acp.rs:246` submit text blocks, without winit key classification.
  No additional character-only text-entry gate was found.
- `ATTRIBUTION.md:42` — native text-entry witnesses and controls retain the
  project's existing attribution policy; all changed/generated source headers
  retain the canonical project, license and absolute policy URLs.

## Regression witnesses and broken control

`tests/gui_rich.rs:1083` dispatches Character(a), Named(Space) with text `" "`,
then Character(b) through production `route_key`; the draft must equal `"a b"`.
`:1092` does the same with the profile switcher open: its painted search text
must be `"a b"`, its no-match message includes that exact query, and the composer
remains empty. `:1118` checks the slash filter before whitespace, then types and
dispatches `/profile opus` through the ordinary Enter command path.
`:1165` checks Control/Super shortcuts, missing event text, control characters
and the composer's active preedit guard.

`scripts/test-space-control.py:33` replaces both real production matches with
the old `Key::Character(_)`, runs the three text regressions, requires exit 101
and each named failure, restores the original bytes in `finally`, verifies their
SHA256, and reruns all four Space tests. Final run: old match exit 101, all three
intended regressions fail; restored match exit 0, 4 passed. In the old match the
draft and profile query are `ab`, and the slash argument separator is missing.
Logs and structured receipt: `target/space-fix/controls/{old-match.log,restored.log,results.json}`.

An initial slash witness assumed completion continues across whitespace. Reading
the pinned tokenizer showed otherwise; the witness was corrected to verify the
existing token boundary and command dispatch before the final green control run.

The finite synchronous witness owns draft/search strings, menu-open state,
shortcut modifiers and preedit state on the UI thread, initially empty and
inactive. Each key dispatch is atomic. Accepted event text must retain its
separator; guarded input must not alter the draft. No scheduling/fairness or
concurrency, ownership, persistence or lifecycle contract changes. Logical-key
classification remains implementation-only in `verification/coverage.toml:139`;
the existing mandatory native/TLC stack still runs in the full gate.

## Verification and final handoff

The following completed on the final source bytes before the signed commit:

- `python3 -I scripts/test-space-control.py` — exit 0, old-match exit 101 and
  restored exit 0, with all 4 dispatcher regressions passing.
- `make test-gui` — exit 0, 57 feature tests passed, including all 24 rich GUI
  cases; headless Metal smoke passed for 22 items / 15 kinds and 3 approval
  buttons. Private HOME, no operator input.
- `cargo clippy --offline --locked --all-targets -- -D warnings` — exit 0.
- `cargo clippy --offline --locked --all-targets --all-features -- -D warnings`
  — exit 0.
- `cargo fmt --check`, `git diff --check`, source headers, KB records and
  inventory-shape checks — exit 0.

Exact commands, source hashes, timings and logs are
`target/space-fix/{gui,clippy-default,clippy-all-features}.{json,log}`.

One full `make test` runs after this report is committed so its native/Java
formal evidence can bind to a clean, signed source commit. This report makes no
advance claim about that run. Its actual outcome is recorded in
`target/space-fix/full.{json,log}` and the final handoff
`/Users/feral/mydev/bboxy450-harness-parity/reports/space-fix.md`.
The runner checks `pgrep -fl scripts/test.py` immediately before starting; another
full runner or a failed process probe prevents a second suite. The final
`make check-formal-evidence` receipt and a retained archive outside `target/`
preserve current-source model/tool/hash evidence beyond build cleanup.

No push, merge, release, installation, credential change, desktop input or
operator desktop capture is part of this task. Native physical keyboard/IME
delivery is not claimed; the tests drive the production dispatcher headlessly.
