---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:16:43.217Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJZD8KDBW3Q2S5FMBE5DTG
number: 4951
provenance:
  digest: sha256:cdd9d44eef078c43930eb80be15dd477b676b799c06bda9b445e702708209aab
  frontmatter_digest: sha256:5fdeb8684ee1c7bb6349887b063958ac412c1277cabc8db9c120019caf80b390
  log: .bboxy/log/2026-11-01.jsonl#32139
  sig: 3+/4isAGn91QMS/C98wdKFV2K9MnlfJavzu0nxAlhfLHnVLh03tj3AXKrscWBOCpM+nbPl3U8zgFKBFip1DoCw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: 'Agent session-log tools: survey for bboxy450-log-converter'
type: research
updated: 2026-11-01T22:16:43.217Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
Date 2026-11-01. This is a compressed survey made by an Oracle research agent. It records ideas only; d4949 forbids depending on or copying from any of these.

**ctxrs/ctx** (https://github.com/ctxrs/ctx, Rust, Apache-2.0)
- Covers 43 providers: an index with no write-back.
- IR: `crates/ctx-history-core/src/core_record.rs:287` CoreRecord, a flat event with:
  - event_id and session_id;
  - parent/root session lineage and a relationship;
  - event_copy, which carries fork provenance;
  - event_sequence and occurred_at;
  - event_type: message, tool_call, tool_output, command_*, artifact, summary or notice;
  - role;
  - parser_revision and normalization_revision;
  - the raw native JSON kept as structured_content.
- Fidelity enum: full, partial, imported, inferred, summary_only.
- Interchange: ctx-history-jsonl-v2, with record types manifest, source, session, event, file_reference and edge.
- Ideas to take: the fidelity enum, lineage, the parser revision, and keeping the raw record.

**casr** (https://github.com/Dicklesworthstone/cross_agent_session_resumer, Rust, MIT)
- 18 providers, with resume in either direction.
- `Provider{read_session, write_session, resume_command}` (src/providers/mod.rs:58-96), with a read(write(read(x))) fidelity check.
- Its CanonicalMessage flattens content to a string, which is lossy.
- Ideas to take: the provider trait shape and the round-trip check.

**Harbor ATIF v1.8** (https://github.com/laude-institute/harbor, rfcs/0001-trajectory-format.md, Apache-2.0)
- A Trajectory is made of steps: source, message, reasoning_content, tool_calls (tool_call_id, function_name, arguments) and observation.results (source_call_id).
- It also carries metrics, subagent_trajectories, continued_trajectory_ref and is_copied_context.
- An ATIF→Claude/Codex native lowering exists (installed/claude_code.py:416).
- It is lossy for resume: reasoning is a plain string with no signature or encrypted blob.
- Idea to take: align the step and tool-call shape and offer ATIF export.

**openai/codex external-agent-migration** (codex-rs/external-agent-migration, Apache-2.0)
- Claude/Cursor → Codex, one way only (`/import`, CLI 0.140).

**Small Claude↔Codex bridges** (session-bridge, session-porter, cct, agent-chorus)
- Known losses: Codex encrypted reasoning; Edit/MultiEdit ↔ apply_patch.
- session-bridge has no license, so don't read its code.

**claude-code-log** (daaain/claude-code-log, MIT)
- A Pydantic taxonomy of every Claude entry type (models.py:131-377).

**OTel GenAI semconv**
- No stable attributes as of 2026-07. Use it only as a secondary export.

**Binary format recommendation**
- zstd seekable format:
  - one frame per N events or per turn;
  - the seek table lives in a trailing skippable frame and is rewritten on append;
  - plain `zstd -d` still works;
  - fsync each frame before rewriting the seek table;
  - a torn table is rebuilt by scanning the frames.
- Records are CBOR (RFC 8949) with integer keys. CBOR is self-describing, keeps unknown fields and allows deterministic hashing.
- A trained zstd dictionary per provider, with its id in the header.
- An optional blake3 content-addressed table for repeated big strings: system prompts and tool definitions.
- Rejected: postcard and bincode (not self-describing), FlatBuffers and Cap'n Proto (rigid), MessagePack (acceptable, but CBOR is the standard).