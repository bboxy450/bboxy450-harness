---
allowed_paths:
- src/frontends/gui/
- src/frontends/mod.rs
- src/main.rs
- tests/gui_*
- tests/formal/
- tests/fixtures/gui/
- verification/
- Cargo.toml
- Cargo.lock
- Makefile
- .bboxy/runs/
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T06:27:42.178Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6AWNPTK11DP11SX76AGC9E5
number: 2202
provenance:
  digest: sha256:a20fe302a716eaf21108ed23f06ef007abe9eb750298fc6aa4d269b4ecd88755
  frontmatter_digest: sha256:07fd726310681ccd4800b8ee688e4a810ee77d74d5ba9779f7c113fde5768436
  log: .bboxy/log/2026-11-01.jsonl#12198
  sig: oXY33z9+Jas0o99OJm5IM9s+sRO4FzMk84prg6oH9u2JyUpH+9WvJozSYdbUs+ii3TaaSPBVTqMgqqoYZloSAg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3b: GUI phase 2 — rich widgets (streaming markdown, highlighting, diff, tool cards, approval, editor) + d2021 conditions'
type: task
updated: 2026-11-01T06:27:42.178Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
# E3b: GUI phase 2

ROLE: EXECUTOR (Codex).

**Worktree:** `/Users/feral/mydev/bboxy450-harness-gui`.
1. Fetch.
2. Create branch `e3b-gui-rich` from `origin/main` at or after `637b10c`. Phase 1 (PR #2) is merged there.
3. Push the branch and open a PR. Do not merge.

E1c is working in parallel on ACP, MCP and compaction in core `src/`. Do not touch core files.

## Toolkit dependency
The rich widgets are in bboxy450-term PR #86, branch `harness-widgets`, which is in rework.

Until PR #86 merges and toolkit 0.1.2 is published:
- Pin `bboxy450-ui`, `bboxy450-doc` and `bboxy450-render` through `[patch.crates-io]`, using a git revision of `https://github.com/bboxy450/bboxy450-term` on that branch, per architect-review-v0 §6.
- Rebase onto each new head E2 pushes. The rework will change the internals, and may slightly change the APIs (see the rejection of task #50).
- When the Oracle signals that 0.1.2 is published, switch to the crates.io versions.

## Outcome
1. **Transcript.** `AgentMessage` and `Reasoning` render as streaming Markdown through `bbt-doc`'s streaming append, with syntax-highlighted code blocks prepared off the UI thread. Reasoning is collapsible.
2. **Changes.** `FileChange` and `TurnDiff` use the diff view.
3. **Tools.** `ToolCall` uses the tool-call card. `Terminal` shows the live tail while running and the full output from its blob when it completes.
4. **Approval.** Use the approval atom: keyboard navigation, Escape to cancel, and Allow visually distinct from Deny. Plan, Image, Usage and Subagent stay as readable plain text. Their dedicated widgets are a later task.
5. **Composer.** Use the upgraded editor: history on Up at the first line, plus `/` and `@` completion hooks. For now completion only offers slash commands; no file search yet.
6. **[[d2021-e3-phase1-merge-conditions]] are binding.**
   - (a) The ">4 KiB never parsed on the UI thread" gate runs through `Bridge::replay` / `publish_frame` and reads `state.prepared.prepared_on_ui`. It must fail if preparation is moved onto the UI thread.
   - (b) No 4 KiB card cap. A 64 KiB reply must be fully readable through virtualized or expandable rendering.
   - (c) Fix the README wording about 100k deltas.
   - (d) Add tests that assert the card role and the live terminal tail as rendered mid-stream.
   - (e) Add an API witness that an old gateway's Close leaves a new actor running.
7. **Budgets §2** hold under the new rendering. Completed item layouts are never rebuilt. Markdown preparation is incremental, never quadratic: add a test that counts parsed bytes over a streamed 64 KiB message.
8. **Smoke.** The smoke checks rendered meaning: a highlighted code block with more than one colour, diff add and remove lines, tool-card status, and approval buttons. Update the screenshots.

## Verify
- `make test` exits 0 unattended.
- No wall-clock gates.
- Show red→green for 6a, 6b and 7.

## Report
One `task_report`. The report file must be `.bboxy/runs/*.log`, never `.md` (AGENTS.md rule).
