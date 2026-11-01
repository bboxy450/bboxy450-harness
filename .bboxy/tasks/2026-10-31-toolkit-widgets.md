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
  digest: sha256:87906d1a20cd7faa10e1eac25ab40e0c68594196938bdfd85e0dedf51ceecd73
  frontmatter_digest: sha256:e82c166f0c13e46045f89a01860c4c31757c0d1c8085edcabe1265fae63c2fcb
  log: .bboxy/log/2026-11-01.jsonl#14228
  sig: PC0f2trXiMu1CwoNpF9mT5dOqMR93ym6XXxDvT5tjKXfrTtqGyjVzcleNf/KN1nKNHI5Qmy2q1epAi6tMK4VCA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Rework round 2 of PR #86 (18ff123b). The Oracle verified 2, 3, 4, 5, 8 and 9 as fixed and non-vacuous: the state-ledger StreamToView, package exclusion, syntect trim, OnceLock, the DirtyWake shape and the invalidation action. Keep them. Three focused fixes remain. Push to the same branch and report again.

    1. **Streaming correctness** (`bbt-doc/src/markdown/streaming.rs:23-34`, `:56-59`). `definition_line` only matches at the start of a line. Pulldown-cmark also accepts link and footnote definitions that change resolution document-wide inside blockquotes and list items: `> [r]: u`, `- [r]: u`, `- [^f]: x`. It also accepts labels split across lines (`[foo\nbar]: u`). None of these sets the full-parse flag, so the incremental output differs from a full parse: a link stays unresolved.
       - Fix: match after `>` and list markers, and treat a line-leading `[` without a closing `]` as a possible definition.
       - Add pieces for each form to the equivalence property test.
       - The `contracts.toml` IncrementalMarkdown assumption about "multiline definitions" must be backed by a test, or removed.
       - Red→green required: a seed with `> [r]: u` that fails on 18ff123b.
    2. **Diff robustness** (`bbt-ui/src/diff.rs`). These cases still return `Err` for the whole diff: a short hunk followed by `@@` (`:47-48`), a short hunk at EOF (`:118`), `\ No newline` before any hunk (`:74`), and a bad hunk header. An over-long hunk swallows the following `diff --git` or `---` as Notes.
       - Make every one of them degrade locally. A header must always start a new file or hunk.
       - Add a test for each.
    3. **ScrollStick liveness can't fail.** `JumpRestoresFollowing` holds in the same step as the jump (`:225-229`), and no broken model targets it.
       - Either drop the liveness claim from the model and contract, or add a broken model whose jump does not restore following, which must fail.
       - The witness at `:405-435` must compare its jump step against the real `jump_to_latest`, not against constants.

    Note: `plugins/gitkb/ATTRIBUTION.md` is accepted as a synchronized attribution copy.

    Verify:
    - `make test` with no `RUST_TEST_THREADS` override. The review found no new test that depends on running alone. If the full gate still needs `=1`, name the failing test and explain why. A #85 mux flake is not a blocker.
    - Report through `task_report`. The report goes outside the crate or stays excluded from it, as now.
report:
  commit: 18ff123b6dc68d2a4f578417359c910a3ce933b3
  op_id: 01M6AZW4R2VNF9N2RH6ET7VQQH
  reported_by: seat:01M6ADTVVZS36VS43P2PG15HRQ
  summary: 'Rework submitted: all nine MUST items addressed. [PR #86](https://github.com/bboxy450/bboxy450-term/pull/86) is OPEN/CLEAN at signed 18ff123b6dc68d2a4f578417359c910a3ce933b3, pushed with clean worktree. Full committed report: [toolkit-widgets-report](https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md), including red→green evidence for1/2/6/7/8, package/lock proof, bounds and gaps. make test with RUST_TEST_THREADS=1 exited0 on this exact head;116models/31crates/48contracts matched native/TLC and all128 headless stories passed. Earlier same-head #85-only exit2 is archived and disclosed. Remaining SHOULD items are approval Escape/descriptions/allow-deny tones; UI/doc/book0.1.2 follow-up remains unpublished. No merge performed.'
role: executor
status: rejected
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E2: upstream bboxy450-term toolkit widgets for the harness GUI + rendering formal models'
type: task
updated: 2026-11-01T07:27:53.594Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test CI_LOG=target/ci-ready.log RUST_TEST_THREADS=1
  evidence:
  - criterion: MUST 1 — definition-only fallback and meaningful incremental work/property coverage (red→green)
    reference: 'streaming.rs:23 classifies definition lines; tests/streaming.rs:137 measures 16,461 reparsed vs 785,890 full bytes across100 appends; :76 proves full-parse equality and96/96 primarily-incremental seeds (7/8 omit definitions). make test-toolkit-rework: red2 on b81d3c38 (''lost prefix at append1:0..308''), green0. Logs /tmp/e2-rework-red-widgets.log and /tmp/e2-rework-test-toolkit-rework.log. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green'
  - criterion: MUST 2 — state-derived applied sequence/origin ledger, not a bad flag (red→green)
    reference: toolkit_models.rs:11 tracks each sequence count/origin; exactly-once prefix and current-generation invariants. :387 removes the good generation guard and specifically requires AppliedOriginsBelongToCurrentGeneration violation. make test-toolkit-models red2 (old model mutation undetected), green0. Native/TLC good104states285edges, broken safety704/2164. /tmp/e2-rework-red-models.log; /tmp/e2-rework-test-toolkit-models.log; https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#rework-acceptance-evidence
  - criterion: MUST 3 — reports excluded from published UI package
    reference: crates/bbt-ui/Cargo.toml exclude=["reports/"]. make list-toolkit-package exit0 executes cargo package --list --allow-dirty --locked -p bboxy450-ui --target-dir /Users/feral/mydev/bboxy450-term-harness-widgets/target and asserts no reports/ path. target/toolkit-package-list.txt; /tmp/e2-rework-package-list-checked.log. make test-licenses exit0.
  - criterion: MUST 4 — exact syntect feature reduction and lock diff
    reference: 'bbt-doc/Cargo.toml: parsing, default-syntaxes, default-themes, regex-fancy only. make lock exit0 removed11 packages including yaml-rust/plist/time/quick-xml0.42. Existing quick-xml0.41 belongs to wayland-scanner, not syntect. Review Cargo.lock diff6ee43422^..6ee43422; /tmp/e2-rework-lock.diff and /tmp/e2-rework-lock.log.'
  - criterion: MUST 5 — shared SyntaxSet/ThemeSet OnceLock
    reference: bbt-doc/src/highlight.rs:33 shared immutable Resources initialized behind worker guard; :87 pointer-identity test across light/dark highlighters. make test-toolkit exit0 and exact18ff full workspace tests passed.
  - criterion: MUST 6 — nonpanicking diff render and local malformed-line degradation (red→green)
    reference: bbt-ui/src/modules/diff.rs:40 accepts blank context, retains damaged raw line as Note and subsequent counters; DiffView handles natural() Err without expect/panic. tests/toolkit.rs:124. make test-toolkit-rework red2 (unprefixed hunk line), green0 with both blank and damaged cases. /tmp/e2-rework-red-widgets.log; /tmp/e2-rework-test-toolkit-rework.log.
  - criterion: MUST 7 — ScrollStick model/API endpoint and reset agreement plus jump liveness (red→green)
    reference: toolkit_models.rs:155 re-sticks at offset>=maximum, resets intent on JumpToLatest, registers jump~>following. :405 compares reachable endpoint/jump edges with actual API; transcript.rs:103 checks actual private reset state. README fixed. make test-toolkit-models red2 (jump intent retained), green0. Native/TLC good32/288; broken safety38/342. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green
  - criterion: MUST 8 — upstream bits/pending/drain coalescing, real new-controller witness (red→green)
    reference: toolkit_models.rs:233 extends DirtyWake with admitted/consumed queue ledger, coalesced frame-ready and remainder rearm. tests/toolkit.rs:38 uses actual StreamToView/DirtySet/ViewRing:3 deltas in2 frames; omitted complete_drain loses final delta. Model structural witness red2 (missing upstream fields), green0. Native/TLC good142/245; broken liveness129/212. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#red-to-green
  - criterion: MUST 9 — explicit definition classification/invalidation action
    reference: toolkit_models.rs:96 distinguishes ordinary/inline-bracket from reference/footnote definition appends; definition changes identity, InvalidateDefinitionPrefix clears cache before return. Broken skips invalidation. Native/TLC good33/45; broken safety48/63. Atomic boundary, finite bounds and grammar exclusions in verification/contracts.toml.
  - criterion: SHOULD and gaps
    reference: 'All128 headless stories pass with folded-hunk line absence, multiple rendered code colours and scroll pixel inequality. UI paint test proves approval selection changes focus paint. Remaining SHOULD: Escape cancellation, option descriptions, distinct allow/deny tones. Deferred APIs, independent new-control browser references and downstream frame/history budgets remain gaps. https://github.com/bboxy450/bboxy450-term/blob/18ff123b6dc68d2a4f578417359c910a3ce933b3/crates/bbt-ui/reports/toolkit-widgets-report.md#bounds-remaining-gaps-and-disposition'
  - criterion: Signed, pushed, conflict-free PR and final verification
    reference: PR86 OPEN/CLEAN at signed18ff123b6dc68d2a4f578417359c910a3ce933b3; origin/harness-widgets identical, tree clean. make push-branch CI_LOG=target/ci-ready.log exit0 verified3 signatures with key9B07E849480C68328CA20684844ECC0587DE7AB0, fresh formal receipt and exactHEAD normal0. Registration-only follow-up resolves newer-main insertion conflict without merge/rebase/history rewrite; merge-tree vs2e7a6791 exit0. Final CI target/ci-ready.log:4808, SHA25646747a3ee7f82126e509d8b5432b1bdc5ee8bf6d285358bc2595bc05edae662a; formal receipt502b8ea6b0a2ec3bb68508937c50f1e1576877dd40995995c0a03ca401b989a6 (clean18ff,116models/31crates/48contracts).
  - criterion: Known mux flake history and retry disposition
    reference: First18ff make test exit2 ONLY capture_fairness.rs:352 after exact serial Screens; d824/#85. Archive/tmp/e2-ready-first-18ff-issue85.log, SHA2567b1f4cd235eaf2a6044d54b116edf17eb19964f37f04b2099c81ea90e546da52. One full retry with RUST_TEST_THREADS=1 exited0 with both#85 tests passing, unchanged assertions/quarantine. Parent6ee full gate also exited0. All runs unattended;27invariant checks pass,11existing slow checks skipped.
  - criterion: Final gate layer 1
    reference: '["sh","scripts/layering.sh"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 2
    reference: '["python3","-I","scripts/test-landing.py"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:6. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 3
    reference: '["python3","-I","scripts/test-resource-ledger.py"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:46. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 4
    reference: '["cargo","build","--workspace","--locked","--target-dir","/Users/feral/mydev/bboxy450-term-harness-widgets/target"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:52. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 5
    reference: '["cargo","clippy","--workspace","--all-targets","--locked","--target-dir","/Users/feral/mydev/bboxy450-term-harness-widgets/target","--","-D","warnings"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:55. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 6
    reference: '["sh","scripts/quarantine-test.sh","--workspace","--locked"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:58. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 7
    reference: '["make","test-formal","TARGET_DIR=/Users/feral/mydev/bboxy450-term-harness-widgets/target"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4256. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 8
    reference: '["sh","scripts/ui-lint.sh"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4581. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 9
    reference: '["cargo","build","-p","bboxy450-term","--locked","--target-dir","/Users/feral/mydev/bboxy450-term-harness-widgets/target"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4598. Parent invocation exported RUST_TEST_THREADS=1.'
  - criterion: Final gate layer 10
    reference: private headless argv retained at log line 4600 — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4600. Parent invocation exported RUST_TEST_THREADS=1.
  - criterion: Final gate layer 11
    reference: '["sh","scripts/invariants.sh"] — exit 0; /Users/feral/mydev/bboxy450-term-harness-widgets/target/ci-ready.log:4617. Parent invocation exported RUST_TEST_THREADS=1.'
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
