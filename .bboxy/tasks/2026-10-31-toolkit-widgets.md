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
  digest: sha256:bf80a392afcdd918e5c58b23e68a662806ac184aa2fda4eb33307210e2f6c309
  frontmatter_digest: sha256:6fd02f524035b9654ab53be45755302da65672212cfdf0c48fe00ee9b1d7cdf6
  log: .bboxy/log/2026-11-01.jsonl#191
  sig: cNnbq+XFXT4W/mQQRS6DynoaNT8SMiepLvb1qZmJgpE6x9ERbjv3rZxxrEVlBhNXHQNCniLQVV67z574BuvJCA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E2: upstream bboxy450-term toolkit widgets for the harness GUI + rendering formal models'
type: task
updated: 2026-11-01T02:08:11.298Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
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
