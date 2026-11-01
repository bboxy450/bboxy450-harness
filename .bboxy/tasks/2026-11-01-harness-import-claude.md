---
allowed_paths:
- src/
- tests/
- scripts/
- verification/
- Cargo.toml
- Cargo.lock
- .bboxy/runs/
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:01:37.407Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJ3RQK3Q9S9MJ3VH9Y365J
number: 4935
provenance:
  digest: sha256:af7e966207a494c9e61b7fccdfb21e3482981503f0d8ee701bb06f1fa7c0a064
  frontmatter_digest: sha256:f6387c72007cd035cccc9e8f47a843e212b891e23417304dfab9debeb1d6942c
  log: .bboxy/log/2026-11-01.jsonl#32008
  sig: tR18/9EAM2xm+FxD3nT4bI8UHoHF/Lqws8PpKjCZW9rR5Ga6jJPBu3X7J/TJUkJ/YuhxGjhpfmbuLoh6IybLCQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E3g: import Claude Code session logs into harness protocol transcripts'
type: task
updated: 2026-11-01T22:01:40.848Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-harness. Work on a new branch from origin/main and open a PR. Do not merge.

Operator request: render real Claude Code sessions from this machine in our harness presentation, for a demo in bboxybook. This task is the converter. The bboxybook page is a separate task that consumes your library API.

Build:
1. A library module `import::claude` with `convert(reader) -> impl Iterator<Item = Result<Envelope, ImportError>>` (or a stream). It turns a Claude Code session JSONL (`~/.claude/projects/<dir>/<session>.jsonl`) into a valid harness protocol transcript: the same Envelope/Event/Item JSONL that `gui --replay` and check_transcript accept, at the current protocol version.
2. A CLI command: `bboxy450-harness import claude <session.jsonl> [-o out.jsonl]`, which writes to stdout by default. `bboxy450-harness gui --replay <out.jsonl>` must render it.

Mapping. Write it as a table in your report and in a doc comment:
- user text → UserMessage;
- assistant text → AgentMessage;
- thinking → Reasoning;
- tool_use / tool_result → ToolCall, with Terminal for Bash, FileChange/TurnDiff for Edit, Write and MultiEdit, and Plan for TodoWrite;
- images → Image (metadata only);
- sidechain / Task subagents → Subagent;
- compaction summaries → Compaction;
- usage → Usage;
- turn boundaries → TurnStarted / TurnEnded;
- timestamps preserved;
- anything unrecognised → an Unknown item or a Notice, never a panic or a silent drop.

Ids must be stable and deterministic, so the same input gives byte-identical output. Read the real format from the files on this machine, but commit nothing from them.

Privacy and secrets (hard rules):
- NEVER commit any content from a real local log. That covers fixtures, golden files, screenshots, reports and test output.
- Tests use a hand-written synthetic Claude-format fixture that covers every mapping row, tool errors, malformed lines and a large file.
- The converter runs the harness redactor over every text field: API-key shapes (the sk-… family and others the redactor knows), the seat capability, and values from `!cmd`-style key config when present. Add a test with planted secret shapes in the synthetic fixture; the output must contain none.
- Your report may describe the real-log run only as statistics (item counts per kind, unknown count, time). Include no content.

Robustness:
- Bounded memory: stream line by line. Measure a 100 MB synthetic file with a structural check, not a wall-clock percentile.
- A truncated last line is tolerated and reported.
- Every converted fixture passes check_transcript.

Verification:
- A golden test: synthetic in, expected protocol JSONL out, sorted canonical JSON as in PR #17.
- A GUI headless smoke test renders the converted synthetic transcript, with every mapped kind visible.
- Broken controls: drop the redaction pass (the planted-secret test goes red), and make an unknown entry panic (the robustness test goes red).
- Run the CLI once against the most recent real local session, and report only counts.

Acceptance:
- `make test` exits 0 unattended, with native/TLC parity.
- All commits are signed.
- The report goes in `.bboxy/runs/2026-11-01-import-claude-report.log`. Don't write "key:" before hex.
- Send one task_report with verify_result.command exactly `make test`.