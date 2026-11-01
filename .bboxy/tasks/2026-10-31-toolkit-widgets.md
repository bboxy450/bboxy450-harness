---
allowed_paths:
- crates/bbt-ui/
- crates/bbt-doc/
- crates/bbt-render/
- crates/bboxybook/
- tests/formal_contracts/
- verification/
- scripts/deps-licences.sh
- Cargo.toml
- Cargo.lock
artifacts:
- '[[01M6AQR54F4TBT16B6EJSZG50F]]'
- '[[01M6AQR7T6DMCX29MTYFMBCHHV]]'
- '[[01M6AZRA74WYMK5MHNEBMQHHMT]]'
- '[[01M6B3Z46NXTCP7P0ZTDCY0YMQ]]'
- '[[01M6B71XE74M655P2RS6DMVQ64]]'
- '[[01M6B71XE74M655P2RS6DMVQ64]]'
assignee: seat:01M6ADTVVZS36VS43P2PG15HRQ
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T02:08:11.298Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6ADTGYAQ5VJ7GZ05FWT7Q93
number: 50
provenance:
  digest: sha256:ac6a6065705544842de0707ee2c78f198eb1ad1c86fa538cac56cb055072c953
  frontmatter_digest: sha256:87b3a3300c1017271e0a135a5f8094257f8e2cc8051f34b957a2339966d266ee
  log: .bboxy/log/2026-11-01.jsonl#24362
  sig: JglZKYtAngQSFjRXHkPOekjJkFbwelB3dGCrKlUA4CibQJ0UcQzDxk4GGdrwZii59Ei2msW3cFoYdBudh2m4Cw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Round 4, focused. Reviewed at 4889321f; your merge d5e19904 is not on origin, so push it. Verified fixed and kept: container and multiline definition detection, ScrollStick (liveness dropped; the witness now calls the real `jump_to_latest`), the four local-degradation diff cases, and allowed paths. The landing plan is unchanged: I land through one signed squash commit, so there is no history rewrite and no Report trailer.

    MUST
    1. **REGRESSION in `bbt-ui/src/diff.rs:88`.** `---` and `+++` are tested as headers before the hunk counters. In a valid hunk, removing a line `-- foo` (SQL, Lua, Haskell comments, email `-- `) renders as `--- foo`; adding `++ x` renders as `+++ x`; removing a bare `--` gives `---`. Each of these now ends the hunk early and pushes everything after it into the headers. 18ff123b handled them correctly.
       - Always break on `diff --git`.
       - Break on `---`/`+++` only when the hunk counters are exhausted, or when `---` is immediately followed by `+++` at a hunk boundary.
       - Tests: removing a `-- comment` line, adding `++ x`, and removing `--`, each inside a valid hunk with lines after it. Each must be red on 4889321f and green after the fix.
    2. **Escaped bracket in a definition label** (`streaming.rs:50-73`). `[a\]b]: u` is a false negative: `find(']')` stops at the escaped `]`. Skip backslash-escaped brackets, and add a corpus piece plus a red seed.
    3. **Make sure `crates/bbt-ui/reports/` is deleted at the pushed head**, per d2900. The 4889321f tree still has it.

    SHOULD
    - Add back a model assertion that a jump resets the requested offset and the manual intent to 0.
    - The generated multiline seed (`seed%32==24`) should reference the label it defines.

    Verify:
    - Run the full `make test` on your merge with main, at default concurrency.
    - Run `make test-kb-reports` on that same merge. It is expected to fail only on the old Report trailers, which the squash landing removes.
    - Push, then send a `task_report` naming the final head and the merge commit.
report:
  commit: 92fb33df6060a18442270c2487f311147fad934c
  op_id: 01M6BGC2QJ1DTMHBJC15TJWM6R
  reported_by: seat:01M6ADTVVZS36VS43P2PG15HRQ
  summary: 'Round 4 complete. Final head and merge commit: signed 92fb33df6060a18442270c2487f311147fad934c, pushed to harness-widgets. Report: [.bboxy/runs/2026-11-01-toolkit-widgets-report.log](../runs/2026-11-01-toolkit-widgets-report.log). All three MUST items and both SHOULD items are addressed with exact 4889321f red-to-green evidence. Full make test passed at default concurrency: 124 models, 31 crates, 52 contracts and 128 headless stories. The separate report check failed only on the four expected historical trailers, handled by the Oracle''s squash landing. The Oracle confirms PR #86 landed as signed squash 59dedc64 through land.sh with all gates and round-4 review passed.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E2: upstream bboxy450-term toolkit widgets for the harness GUI + rendering formal models'
type: task
updated: 2026-11-01T12:12:02.309Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Full report
    reference: .bboxy/runs/2026-11-01-toolkit-widgets-report.log — complete files/mechanisms, per-item evidence, commands/exits, red-to-green, bounds, gaps and release follow-up. Signed harness report commit 969a9e940841b006caf73f11c495c70a15a3b530; SHA256 0728aa84c9ac5a03d835406a2df39afdebdee5b758ec752301382a1c76b614c8.
  - criterion: Exact-head verification
    reference: make test CI_LOG=target/ci-round4.log exited 0 on 92fb33df6060a18442270c2487f311147fad934c, no RUST_TEST_THREADS override. Exact-source normal0 at target/ci-round4.log:4885; CI SHA256 0a4b5f9a72a659780b6b6be2df547d8b62825421d7f77ffc5f7442a3bb62f900. Fresh formal receipt b724a59e47cfa5adcda6abd01c9a9591fe0b969ee857ce415ef3586026fdb56b:124 models/31 crates/52 contracts. All128stories, headless smoke and27normal invariants passed;11existing slow checks skipped.
  - criterion: Expected report check and landing disposition
    reference: SIGNED_RANGE=origin/main..HEAD make test-kb-reports exited2 only on old Report trailers in4889321f/18ff123b/6ee43422/b81d3c38;7other corpus tests passed. No gate/history workaround. Oracle message01M6BGACSPRFT8J2EVPF0K12AP confirms signed squash59dedc64 landed through land.sh with all gates and round4review passed.
  exit_code: 0
---
# E2: upstream toolkit widgets for a rich agent GUI

ROLE: EXECUTOR (Codex). The Oracle is Vesper Greywind and the Architect is Nyx Copperwing.

**Worktree:** `/Users/feral/mydev/bboxy450-term-harness-widgets`, branch `harness-widgets`, repo `bboxy450/bboxy450-term`. Before starting, read bboxy450-term's `AGENTS.md`/`CLAUDE.md` and follow its landing rules. Push the branch and open a PR. Do not merge.

## Why
bboxy450-harness is a GUI-first coding-agent harness: repo https://github.com/bboxy450/bboxy450-harness, local at `/Users/feral/mydev/bboxy450-harness`. Its GUI needs widgets that the toolkit lacks, and per the spec they land upstream, never forked. Read these harness files for context:

- `/Users/feral/mydev/bboxy450-harness/.bboxy/specs/toolkit-lowering.md`, the gap table
- `/Users/feral/mydev/bboxy450-harness/.bboxy/research/bboxy450-term-toolkit.md`, sections 2 and 7, which give `file:line` pointers
- `/Users/feral/mydev/bboxy450-harness/.bboxy/interfaces/protocol.md` §4, the Item kinds these widgets render
- `/Users/feral/mydev/bboxy450-harness/.bboxy/interfaces/interfaces.md`, the rendering rows of the matrix
- `/Users/feral/mydev/bboxy450-harness/.bboxy/specs/budgets.md`, frame and preparation budgets

## Outcome
Each item below is generic. None of them knows about the harness or its protocol.

1. **bbt-doc: streaming markdown append.** Re-parse only the open tail block, using a stable-prefix cache.
   - Witness: a property test that incremental output equals a full parse after every append, at random split points, including splits inside code fences and tables.
2. **bbt-doc: syntax highlighting hook** that turns `Code{info}` into styled spans.
   - Use a pure-Rust highlighter whose licence passes `scripts/deps-licences.sh`.
   - Heavy preparation runs off the UI thread, following the `documents.rs` `Work::prepare` / AsyncRenderPreparation pattern.
3. **bbt-ui: markdown inside StreamDock chat messages**, or a `Transcript` module that renders bbt-doc blocks per message.
4. **bbt-ui: diff view.** Unified diff with per-hunk folding, line numbers and add/remove styling from theme tokens.
5. **bbt-ui: tool-call card atom.** Status chip, title, collapsible args and output, built on `Accordion`.
6. **bbt-ui: approval prompt atom.** Options navigable by keyboard.
7. **bbt-ui `text_edit::Editor`.** Up/Down, word and line motions, undo/redo, input history, and a completion popover hook for `/` and `@`.
8. **Formal models**, registered in bboxy450-term's verification inventory. Each needs a Broken control that must fail plus a real witness test. Reuse existing upstream models (DirtyWake, AsyncRenderPreparation, RenderLinePlan) where they fit instead of duplicating them.
   - **StreamToView:** deltas are applied in order and exactly once to the current generation; late chunks for a dropped view are retired.
   - **IncrementalMarkdown:** the prefix cache is never stale.
   - **ScrollStick:** stick-to-bottom holds unless the user scrolled away.
   - **DirtyWakeFrame:** event arrival always yields a frame.
9. **bboxybook stories** for every new widget, so they render headless in the existing story and parity flow.

## Verify
- `make test` in the worktree: layering, clippy, unit, `make test-formal`, ui-lint and the headless GUI smoke. All of it unattended.
- Every new widget has a bboxybook story that renders headless without panicking.

## Report (one final report)
- files and mechanism
- commands with exit codes
- evidence per item
- which toolkit crate versions need bumping for a 0.1.2 release
- gaps
- the branch and head commit, which must be signed

Blockers that stop all work: at most 3 lines to the Oracle.


## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

Implemented the nine upstream toolkit outcomes. [PR #86](https://github.com/bboxy450/bboxy450-term/pull/86) is open, unmerged, from clean branch harness-widgets at signed b81d3c384c48185e5f85fa3ce997d297ab361f46. Full committed report: [toolkit-widgets-report](https://github.com/bboxy450/bboxy450-term/blob/b81d3c384c48185e5f85fa3ce997d297ab361f46/crates/bbt-ui/reports/toolkit-widgets-report.md). Final make test exited 0; 116 models/31 crates/48 contracts passed native/TLC parity, and all 128 new headless stories passed. The report covers mechanisms, earlier failures, gaps and the UI/doc/book 0.1.2 release follow-up.

```json
{
  "commit": "b81d3c384c48185e5f85fa3ce997d297ab361f46",
  "op_id": "01M6AQSRWRVE096C5F3GW0KBE6",
  "verify_result": {
    "command": "make test CI_LOG=target/ci-final.log",
    "evidence": [
      {
        "criterion": "1. Stable-prefix streaming Markdown, random split/full-parse equivalence",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-doc/tests/streaming.rs:21 — every split and seeded partitions; target/ci-final.log:499 — passed. Full mapping: https://github.com/bboxy450/bboxy450-term/blob/b81d3c384c48185e5f85fa3ce997d297ab361f46/crates/bbt-ui/reports/toolkit-widgets-report.md#files-and-acceptance-evidence"
      },
      {
        "criterion": "2. Pure-Rust syntax hook and worker preparation",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-doc/tests/highlight.rs:13 — owned worker spans; :46 — generation retirement; :64 — UI guard. target/ci-final.log:422 — all three passed; dependency licence invariant passed."
      },
      {
        "criterion": "3. Prepared Markdown per transcript message",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-ui/src/modules/transcript.rs:24 — keyed prepared bodies; crates/bboxybook/src/toolkit.rs — Markdown transcript stories. target/ci-final.log:358 — full story matrix passed."
      },
      {
        "criterion": "4. Unified diff, independent folds, line numbers and themed adds/removes",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-ui/tests/toolkit.rs:55 and :222 — hunk numbering/folding and multi-file headers; target/ci-final.log:3915 — passed. Open/folded headless stories passed."
      },
      {
        "criterion": "5. Accordion tool-call card",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-ui/tests/toolkit.rs:116 — independent argument/output toggles; target/ci-final.log:3919 — passed. All status/fold stories passed."
      },
      {
        "criterion": "6. Keyboard approval options",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-ui/tests/toolkit.rs:75 — disabled skipping, wrap, endpoints and Enter; target/ci-final.log:3913 — passed. Enabled/disabled stories passed."
      },
      {
        "criterion": "7. Editor motions, undo/redo, history and slash/mention completion",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bbt-ui/tests/toolkit.rs:143 and :191 — Unicode motions/history/undo and stale completion refusal; target/ci-final.log:3914 and :3920 — passed."
      },
      {
        "criterion": "8. Four inventoried formal contracts, broken controls and real witnesses",
        "reference": "target/ci-final.log:4669 — StreamToView; :4671 — IncrementalMarkdown; :4673 — ScrollStick; :4675 — DirtyWakeFrame, each followed by its expected broken failure. Clean b81d3c38 receipt SHA-256 305d157afe6958f074f5c940f3a6845453c333c8dc4ee9d515d744d40a66ba11; 116 models/31 crates/48 contracts. https://github.com/bboxy450/bboxy450-term/blob/b81d3c384c48185e5f85fa3ce997d297ab361f46/crates/bbt-ui/reports/toolkit-widgets-report.md#files-and-acceptance-evidence maps real witnesses and bounds."
      },
      {
        "criterion": "9. Every new widget renders headless in bboxybook",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/crates/bboxybook/tests/toolkit.rs:10 — all 128 theme/density cases; target/ci-final.log:358 — passed. make stories-toolkit exited 0; native captures in target/bboxybook/toolkit/toolkit/."
      },
      {
        "criterion": "Final exact-source verification and signed PR handoff",
        "reference": "/Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-final.log:4920 — normal 0 for b81d3c384c48185e5f85fa3ce997d297ab361f46; log SHA-256 419b70302d0fe3d869d9749369ffb93c6b036b373c82434d5a5dab10552d7136. make push-branch CI_LOG=target/ci-final.log exited 0 (signature/formal/source gates); PR #86 head and origin/harness-widgets match, approved key 9B07E849480C68328CA20684844ECC0587DE7AB0 verified G; clean tree. 27 invariant checks passed, 11 existing slow checks skipped; both issue #85 regressions passed without exception."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

Rework submitted: all nine MUST items addressed. [PR #86](https://github.com/bboxy450/bboxy450-term/pull/86) is OPEN/CLEAN at signed 18ff123b6dc68d2a4f578417359c910a3ce933b3, pushed with clean worktree. Full committed report: [toolkit-widgets-report](https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md), including red→green evidence for1/2/6/7/8, package/lock proof, bounds and gaps. make test with RUST_TEST_THREADS=1 exited0 on this exact head;116models/31crates/48contracts matched native/TLC and all128 headless stories passed. Earlier same-head #85-only exit2 is archived and disclosed. Remaining SHOULD items are approval Escape/descriptions/allow-deny tones; UI/doc/book0.1.2 follow-up remains unpublished. No merge performed.

```json
{
  "commit": "18ff123b6dc68d2a4f578417359c910a3ce933b3",
  "op_id": "01M6AZW4R2VNF9N2RH6ET7VQQH",
  "verify_result": {
    "command": "make test CI_LOG=target/ci-ready.log RUST_TEST_THREADS=1",
    "evidence": [
      {
        "criterion": "MUST 1 — definition-only fallback and meaningful incremental work/property coverage (red→green)",
        "reference": "streaming.rs:23 classifies definition lines; tests/streaming.rs:137 measures 16,461 reparsed vs 785,890 full bytes across100 appends; :76 proves full-parse equality and96/96 primarily-incremental seeds (7/8 omit definitions). make test-toolkit-rework: red2 on b81d3c38 ('lost prefix at append1:0..308'), green0. Logs /tmp/e2-rework-red-widgets.log and /tmp/e2-rework-test-toolkit-rework.log. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green"
      },
      {
        "criterion": "MUST 2 — state-derived applied sequence/origin ledger, not a bad flag (red→green)",
        "reference": "toolkit_models.rs:11 tracks each sequence count/origin; exactly-once prefix and current-generation invariants. :387 removes the good generation guard and specifically requires AppliedOriginsBelongToCurrentGeneration violation. make test-toolkit-models red2 (old model mutation undetected), green0. Native/TLC good104states285edges, broken safety704/2164. /tmp/e2-rework-red-models.log; /tmp/e2-rework-test-toolkit-models.log; https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#rework-acceptance-evidence"
      },
      {
        "criterion": "MUST 3 — reports excluded from published UI package",
        "reference": "crates/bbt-ui/Cargo.toml exclude=[\"reports/\"]. make list-toolkit-package exit0 executes cargo package --list --allow-dirty --locked -p bboxy450-ui --target-dir /Users/feral/mydev/bboxy450-term-harness-widgets/target and asserts no reports/ path. target/toolkit-package-list.txt; /tmp/e2-rework-package-list-checked.log. make test-licenses exit0."
      },
      {
        "criterion": "MUST 4 — exact syntect feature reduction and lock diff",
        "reference": "bbt-doc/Cargo.toml: parsing, default-syntaxes, default-themes, regex-fancy only. make lock exit0 removed11 packages including yaml-rust/plist/time/quick-xml0.42. Existing quick-xml0.41 belongs to wayland-scanner, not syntect. Review Cargo.lock diff6ee43422^..6ee43422; /tmp/e2-rework-lock.diff and /tmp/e2-rework-lock.log."
      },
      {
        "criterion": "MUST 5 — shared SyntaxSet/ThemeSet OnceLock",
        "reference": "bbt-doc/src/highlight.rs:33 shared immutable Resources initialized behind worker guard; :87 pointer-identity test across light/dark highlighters. make test-toolkit exit0 and exact18ff full workspace tests passed."
      },
      {
        "criterion": "MUST 6 — nonpanicking diff render and local malformed-line degradation (red→green)",
        "reference": "bbt-ui/src/modules/diff.rs:40 accepts blank context, retains damaged raw line as Note and subsequent counters; DiffView handles natural() Err without expect/panic. tests/toolkit.rs:124. make test-toolkit-rework red2 (unprefixed hunk line), green0 with both blank and damaged cases. /tmp/e2-rework-red-widgets.log; /tmp/e2-rework-test-toolkit-rework.log."
      },
      {
        "criterion": "MUST 7 — ScrollStick model/API endpoint and reset agreement plus jump liveness (red→green)",
        "reference": "toolkit_models.rs:155 re-sticks at offset>=maximum, resets intent on JumpToLatest, registers jump~>following. :405 compares reachable endpoint/jump edges with actual API; transcript.rs:103 checks actual private reset state. README fixed. make test-toolkit-models red2 (jump intent retained), green0. Native/TLC good32/288; broken safety38/342. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green"
      },
      {
        "criterion": "MUST 8 — upstream bits/pending/drain coalescing, real new-controller witness (red→green)",
        "reference": "toolkit_models.rs:233 extends DirtyWake with admitted/consumed queue ledger, coalesced frame-ready and remainder rearm. tests/toolkit.rs:38 uses actual StreamToView/DirtySet/ViewRing:3 deltas in2 frames; omitted complete_drain loses final delta. Model structural witness red2 (missing upstream fields), green0. Native/TLC good142/245; broken liveness129/212. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green"
      },
      {
        "criterion": "MUST 9 — explicit definition classification/invalidation action",
        "reference": "toolkit_models.rs:96 distinguishes ordinary/inline-bracket from reference/footnote definition appends; definition changes identity, InvalidateDefinitionPrefix clears cache before return. Broken skips invalidation. Native/TLC good33/45; broken safety48/63. Atomic boundary, finite bounds and grammar exclusions in verification/contracts.toml."
      },
      {
        "criterion": "SHOULD and gaps",
        "reference": "All128 headless stories pass with folded-hunk line absence, multiple rendered code colours and scroll pixel inequality. UI paint test proves approval selection changes focus paint. Remaining SHOULD: Escape cancellation, option descriptions, distinct allow/deny tones. Deferred APIs, independent new-control browser references and downstream frame/history budgets remain gaps. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#bounds-remaining-gaps-and-disposition"
      },
      {
        "criterion": "Signed, pushed, conflict-free PR and final verification",
        "reference": "PR86 OPEN/CLEAN at signed18ff123b6dc68d2a4f578417359c910a3ce933b3; origin/harness-widgets identical, tree clean. make push-branch CI_LOG=target/ci-ready.log exit0 verified3 signatures with key9B07E849480C68328CA20684844ECC0587DE7AB0, fresh formal receipt and exactHEAD normal0. Registration-only follow-up resolves newer-main insertion conflict without merge/rebase/history rewrite; merge-tree vs2e7a6791 exit0. Final CI target/ci-ready.log:4808, SHA25646747a3ee7f82126e509d8b5432b1bdc5ee8bf6d285358bc2595bc05edae662a; formal receipt502b8ea6b0a2ec3bb68508937c50f1e1576877dd40995995c0a03ca401b989a6 (clean18ff,116models/31crates/48contracts)."
      },
      {
        "criterion": "Known mux flake history and retry disposition",
        "reference": "First18ff make test exit2 ONLY capture_fairness.rs:352 after exact serial Screens; d824/#85. Archive/tmp/e2-ready-first-18ff-issue85.log, SHA2567b1f4cd235eaf2a6044d54b116edf17eb19964f37f04b2099c81ea90e546da52. One full retry with RUST_TEST_THREADS=1 exited0 with both#85 tests passing, unchanged assertions/quarantine. Parent6ee full gate also exited0. All runs unattended;27invariant checks pass,11existing slow checks skipped."
      },
      {
        "criterion": "Final gate layer 1",
        "reference": "[\"sh\",\"scripts/layering.sh\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 2",
        "reference": "[\"python3\",\"-I\",\"scripts/test-landing.py\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:6. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 3",
        "reference": "[\"python3\",\"-I\",\"scripts/test-resource-ledger.py\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:46. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 4",
        "reference": "[\"cargo\",\"build\",\"--workspace\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:52. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 5",
        "reference": "[\"cargo\",\"clippy\",\"--workspace\",\"--all-targets\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\",\"--\",\"-D\",\"warnings\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:55. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 6",
        "reference": "[\"sh\",\"scripts/quarantine-test.sh\",\"--workspace\",\"--locked\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:58. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 7",
        "reference": "[\"make\",\"test-formal\",\"TARGET_DIR=/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4256. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 8",
        "reference": "[\"sh\",\"scripts/ui-lint.sh\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4581. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 9",
        "reference": "[\"cargo\",\"build\",\"-p\",\"bboxy450-term\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4598. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 10",
        "reference": "private headless argv retained at log line 4600 — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4600. Parent invocation exported RUST_TEST_THREADS=1."
      },
      {
        "criterion": "Final gate layer 11",
        "reference": "[\"sh\",\"scripts/invariants.sh\"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4617. Parent invocation exported RUST_TEST_THREADS=1."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

Second rework implemented and pushed to [PR #86](https://github.com/bboxy450/bboxy450-term/pull/86) at signed4889321f13e509dc7ca18ce7ed3f0217e76cdd1b, clean worktree. [Committed report](https://github.com/bboxy450/bboxy450-term/blob/4889321f13e509dc7ca18ce7ed3f0217e76cdd1b/crates/bbt-ui/reports/toolkit-widgets-report.md) maps mechanisms, original outcomes, red→green and gaps. make test exited0 with default concurrency and no serial retry;116models/31crates/48contracts and128stories pass, both#85witnesses pass. Three focused fixes are complete. Integration remains blocked by conflicts with newer main ee6b7d5e and its new historical-report gate while authenticated report-record tools are unavailable; attempted integration was aborted. PR remains unmerged/CONFLICTING. Oracle disposition needed for report migration/current-main integration.

```json
{
  "commit": "4889321f13e509dc7ca18ce7ed3f0217e76cdd1b",
  "op_id": "01M6B40N0XESJ4X72GVH4Z55S5",
  "verify_result": {
    "command": "make test CI_LOG=target/ci-round2.log",
    "evidence": [
      {
        "criterion": "Round2 #1 streaming correctness / red→green",
        "reference": "bbt-doc/src/markdown/streaming.rs:25,51,86 removes container markers and retains ambiguous multiline-label candidates. tests/streaming.rs:23 quoted-definition seed and :44 all-split/random corpus cover > [r]: u, - [r]: u, - [^f]: x, [foo\\nbar]: u. make test-toolkit-rework exit2 on unchanged18ff123b (/tmp/e2-round2-red.log SHA25661a8b5ac3ed191e44069441b16be0dc2709533d38502e320bf0221c5941d549a), then exit0 (/tmp/e2-round2-green.log):6doc+15UI tests,95/96 primarily incremental seeds,16,461/785,890 work bytes. :31 proves ordinary-link candidate disambiguation resumes caching."
      },
      {
        "criterion": "Round2 #2 local diff recovery / red→green",
        "reference": "bbt-ui/src/modules/diff.rs:42 boundaries outrank counters; incomplete/malformed rows stay visible as notes. tests/toolkit.rs:144,151,157,172 test next-hunk/EOF truncation, orphan note, bad header, and new-file boundaries with zero/short/long counts, including bare/tab headers. Four new tests failed on18ff in same exit2 red run and pass in exit0 green; separate bad-header input is asserted too. No expect/panic render path."
      },
      {
        "criterion": "Round2 #3 ScrollStick claim and real API witness",
        "reference": "toolkit_models.rs:155 removes JumpRestoresFollowing response and marker; contracts.toml:472 drops asynchronous liveness claim. :390 witness calls real jump_to_latest at:416 for every reachable jump and compares following/layout projection. make test-toolkit-models exit0, all4witnesses; good28states/252edges and Broken34/306safety match TLC. Existing private reset-state unit witness retained."
      },
      {
        "criterion": "Accepted fixes retained: prior MUST2/3/4/5/8/9",
        "reference": "Ledger generation/order counts and origin invariants, report exclusion, exact four syntect features, OnceLock resources, upstream DirtyWake bits/pending/drain_pc+coalescing, and explicit definition invalidation remain. Good/Broken graphs: Stream104/285 vs704/2164 safety; Incremental33/45 vs48/63 safety; Dirty142/245 vs129/212 liveness. All match TLC. Prior required red→green1/2/6/7/8 logs and mechanisms remain in committed report. make list-toolkit-package exit0 (/tmp/e2-round2-package.log); target/toolkit-package-list.txt omits reports/."
      },
      {
        "criterion": "Original nine outcomes and stories",
        "reference": "Committed report's nine-row matrix maps streaming/highlighting/Transcript/diff/tool/approval/Editor/formal/stories to source and tests. target/ci-round2.log:355 every_rich_widget_story_renders_headless_across_themes_and_densities passes, asserting all128 cases. Folding, multi-colour syntax, transcript scroll and approval focus paint assertions remain."
      },
      {
        "criterion": "Full-gate layer 1 (actual exit0)",
        "reference": "[\"sh\",\"scripts/layering.sh\"] — exit0, target/ci-round2.log:4; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 2 (actual exit0)",
        "reference": "[\"python3\",\"-I\",\"scripts/test-landing.py\"] — exit0, target/ci-round2.log:6; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 3 (actual exit0)",
        "reference": "[\"python3\",\"-I\",\"scripts/test-resource-ledger.py\"] — exit0, target/ci-round2.log:65; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 4 (actual exit0)",
        "reference": "[\"cargo\",\"build\",\"--workspace\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit0, target/ci-round2.log:71; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 5 (actual exit0)",
        "reference": "[\"cargo\",\"clippy\",\"--workspace\",\"--all-targets\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\",\"--\",\"-D\",\"warnings\"] — exit0, target/ci-round2.log:79; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 6 (actual exit0)",
        "reference": "[\"sh\",\"scripts/quarantine-test.sh\",\"--workspace\",\"--locked\"] — exit0, target/ci-round2.log:101; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 7 (actual exit0)",
        "reference": "[\"make\",\"test-formal\",\"TARGET_DIR=/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit0, target/ci-round2.log:4313; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 8 (actual exit0)",
        "reference": "[\"sh\",\"scripts/ui-lint.sh\"] — exit0, target/ci-round2.log:4640; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 9 (actual exit0)",
        "reference": "[\"cargo\",\"build\",\"-p\",\"bboxy450-term\",\"--locked\",\"--target-dir\",\"/Users/feral/mydev/bboxy450-term-harness-widgets/target\"] — exit0, target/ci-round2.log:4658; run by canonical make test."
      },
      {
        "criterion": "Full-gate layer 10 (actual exit0)",
        "reference": "Complete exact private headless env/argv at target/ci-round2.log:4663; binary --headless --smoke --ctl --unattended, private account/home/run/repository; exit0, stopped Ok at:4679. Full argv also /tmp/e2-round2-final-evidence.json."
      },
      {
        "criterion": "Full-gate layer 11 (actual exit0)",
        "reference": "[\"sh\",\"scripts/invariants.sh\"] — exit0, target/ci-round2.log:4680; run by canonical make test."
      },
      {
        "criterion": "Exact-head proof / no serial override / issue85",
        "reference": "make test CI_LOG=target/ci-round2.log exit0; Make default RUST_TEST_THREADS=2, environment unset, no override/retry/exception. Source4889321f line1, normal0 result:4875; all27 invariants0failures,11policy slow skips:4873. Both issue85 mux tests pass, including watched replay invariant. CI SHA2567905ed57e14752410f138daa2813752a80dd668d83d49085bb9ca6d2b4966afa. Fresh formal receipt4fa8940c3c7872d06717d2516f8798e3459689c220c8a362ac69b44c588539be:116models/31crates/48contracts, clean4889321f. Archived /tmp/e2-round2-ci-4889321f.log."
      },
      {
        "criterion": "Branch / signature / push / report",
        "reference": "harness-widgets head4889321f13e509dc7ca18ce7ed3f0217e76cdd1b; signatureG key9B07E849480C68328CA20684844ECC0587DE7AB0 Boxy<bboxy450@gmail.com>. make push-branch CI_LOG=target/ci-round2.log exit0 verifies4signedcommits0exceptions+freshformal. Remote branch and PR86 head match; worktree clean. Committed report artifact01M6B3Z46NXTCP7P0ZTDCY0YMQ SHA256e23cefe5bed3e41ceff2681fd96f45ef125deeb47fc6fe7864545ab777829c6a. PR unmerged."
      },
      {
        "criterion": "Integration gap requiring Oracle disposition",
        "reference": "Main advanced to ee6b7d5e during gate. git merge-tree --write-tree HEAD origin/main exit1: Makefile,bbt-doc/src/lib.rs,verification/contracts.toml,corpus.toml,coverage.toml. gh pr view86:OPEN/CONFLICTING/DIRTY. Attempted own-branch main merge was aborted; no incoming AGENTS/schema changes retained. Source inspection of main bbt-doc/tests/corpus/reports.rs:61,139-180 indicates new gate rejects four historical Report:[[toolkit-widgets-report]] trailers in candidate commit trees. Current authenticated knowledge_write enum has no report and task_report schema has no report locator. Latest brief explicitly accepts excluded crate report. Need Oracle disposition for legacy-trailer migration/current-main integration; no history rewrite/schema edits performed. PR body records this gap."
      },
      {
        "criterion": "Scope / gaps / release",
        "reference": "d824 scope/issue85 decision cited in report+PR;31 attribution copies required by license-packages.py COPIES (including accepted plugins/gitkb). AGENTS/KBschema untouched by this branch. Remaining SHOULD approval Escape/descriptions/allow-deny tones; explicit deferred virtualization,Plan/Image,ANSI/tail truncation,intra-line diff. UI/doc/book need0.1.2 release follow-up; no versions/publish/main merge. No downstream p99 frame-budget proof or slow/night coverage claimed."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

Round 4 complete. Final head and merge commit: signed 92fb33df6060a18442270c2487f311147fad934c, pushed to harness-widgets. Report: [.bboxy/runs/2026-11-01-toolkit-widgets-report.log](../runs/2026-11-01-toolkit-widgets-report.log). All three MUST items and both SHOULD items are addressed with exact 4889321f red-to-green evidence. Full make test passed at default concurrency: 124 models, 31 crates, 52 contracts and 128 headless stories. The separate report check failed only on the four expected historical trailers, handled by the Oracle's squash landing. The Oracle confirms PR #86 landed as signed squash 59dedc64 through land.sh with all gates and round-4 review passed.

```json
{
  "commit": "92fb33df6060a18442270c2487f311147fad934c",
  "op_id": "01M6BGC2QJ1DTMHBJC15TJWM6R",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Full report",
        "reference": ".bboxy/runs/2026-11-01-toolkit-widgets-report.log — complete files/mechanisms, per-item evidence, commands/exits, red-to-green, bounds, gaps and release follow-up. Signed harness report commit 969a9e940841b006caf73f11c495c70a15a3b530; SHA256 0728aa84c9ac5a03d835406a2df39afdebdee5b758ec752301382a1c76b614c8."
      },
      {
        "criterion": "Exact-head verification",
        "reference": "make test CI_LOG=target/ci-round4.log exited 0 on 92fb33df6060a18442270c2487f311147fad934c, no RUST_TEST_THREADS override. Exact-source normal0 at target/ci-round4.log:4885; CI SHA256 0a4b5f9a72a659780b6b6be2df547d8b62825421d7f77ffc5f7442a3bb62f900. Fresh formal receipt b724a59e47cfa5adcda6abd01c9a9591fe0b969ee857ce415ef3586026fdb56b:124 models/31 crates/52 contracts. All128stories, headless smoke and27normal invariants passed;11existing slow checks skipped."
      },
      {
        "criterion": "Expected report check and landing disposition",
        "reference": "SIGNED_RANGE=origin/main..HEAD make test-kb-reports exited2 only on old Report trailers in4889321f/18ff123b/6ee43422/b81d3c38;7other corpus tests passed. No gate/history workaround. Oracle message01M6BGACSPRFT8J2EVPF0K12AP confirms signed squash59dedc64 landed through land.sh with all gates and round4review passed."
      }
    ],
    "exit_code": 0
  }
}
```
