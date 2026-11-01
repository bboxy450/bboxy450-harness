---
allowed_paths:
- crates/bboxybook/
- crates/bboxy450-harness
- Cargo.toml
- Cargo.lock
- scripts/
- tests/
- Makefile
- verification/
- lab/
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:01:52.387Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJ47DCVW0HF57NV1AB65MR
number: 4939
provenance:
  digest: sha256:e224e442f8aad8b870f4f81412bf9d0989e1661fc0b06e79d14f0d0eb371be66
  frontmatter_digest: sha256:29c1f2d5594586fa909ea27408de4e49ab306bdae22df3c7a0e7f0111c73c657
  log: .bboxy/log/2026-11-01.jsonl#32019
  sig: bBExss4yoC+DwFtv8iuCBwhe/g7LHr+4uFG0EQwTzHnN04amfmEVuGEWASlKBQmRzzjK/h5hjaI0QCxyh/l4Dw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
status: open
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E8: bboxybook harness demo page rendering converted Claude sessions via the real harness view'
type: task
updated: 2026-11-01T22:01:52.387Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Work on a new worktree and branch `bboxybook-harness` from origin/main, and open a PR. Do not merge or run land.sh. Start only once harness task #4935 (import::claude) is merged on harness main, and pin the submodule to that commit.

Operator request: bboxybook (crates/bboxybook, `bboxy450-book`, the native component gallery) is where we demo the harness. Integrate it fully.

Build:
1. A "Harness" section in bboxybook that renders a harness protocol transcript with the harness's own GUI view code (bboxy450-harness `frontends::gui` under the `embedded` feature: view, rich cards, prepared layout). Do not reimplement the cards. Show every item kind; the page scrolls and the cards expand and collapse as they do in the harness GUI.
2. Sources for the page:
   - (a) The committed synthetic demo transcript. It is the default, and it is what tests and screenshots use.
   - (b) At runtime only: `bboxy450-book --claude-log <path>`, or "latest local session" (the newest `~/.claude/projects/**/*.jsonl`), converted on the fly with `bboxy450_harness::import::claude::convert`. Nothing from a real log is ever written into the repo, fixtures, screenshots or reports.
3. Depend on bboxy450-harness by in-tree path with `default-features = false, features = ["embedded"]`. NEVER enable `acp`: the parent feature guard (serde_json preserve_order) must stay green. Add bboxybook to the guard's coverage if it isn't already covered.

Evidence:
- A headless bboxybook render or screenshot of the synthetic demo, committed under .bboxy/runs/. It shows the rich harness cards: messages, reasoning, tool call, terminal, diff, plan and subagent.
- A test that the page uses the harness view entry point, so it can't silently fork. A broken control swaps in a stub renderer and must go red.
- A test that `--claude-log` on a synthetic Claude-format file renders N items equal to the converter's output count.
- A privacy guard: a test that the committed screenshots and fixtures come only from the synthetic source.
- The feature guard and `make test-harness-native` still pass.

Acceptance:
- Full `make test` exits 0 at default concurrency, unattended.
- All commits are signed, with no Report trailer.
- The report goes in bboxy450-harness `.bboxy/runs/2026-11-01-bboxybook-harness-report.log`. Tell me the commit. Don't write "key:" before hex.
- Send one task_report with verify_result.command exactly `make test`.