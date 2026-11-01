---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:17:24.629Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CK0NPSMFYQJTKKJ1CH2BK0
number: 4953
provenance:
  digest: sha256:818cc5533debd2372bc7200420e38edc7af53d16d9a559b90431f594beb3be2f
  frontmatter_digest: sha256:7b026cc6c3999a8baa79876289e865ab3f4a7c8f4494f0aaf2285fc41401d06c
  log: .bboxy/log/2026-11-01.jsonl#32163
  sig: NR9L6YQXxzBCtCYTUIcLDpXmRWagCb99rkMopHI0EGmJdpMWSSwC+GeAJwKreFH7FnzcPD55dI4DYDXXcEL7Bg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: Claude Code and Codex native session formats, resume mechanics, cross-switch loss
type: research
updated: 2026-11-01T22:17:24.629Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
Date 2026-11-01. Oracle research agent, read-only, on this machine with Claude Code 2.1.304 and codex-cli 0.164.0. Field names only; no user content. Writing a synthesized session and resuming it is NOT yet proven.

## Claude Code
**Path.** `~/.claude/projects/<cwd with "/" → "-">/<session-uuid>.jsonl`.

**Line types.** user, assistant, system, attachment, api-request, api-request-blob, api-request-shape, queue-operation, last-prompt (leafUuid), custom-title, ai-title, agent-name, mode, permission-mode, file-history-snapshot/delta, pr-link, cost-state. None of the 31 local files has a `summary` type.

**user/assistant fields.**
- Shared: uuid, parentUuid (null at the root), sessionId, cwd, gitBranch, version, timestamp, isSidechain, userType, entrypoint, slug, message.
- assistant adds: requestId and message{id, model, role, type, content, stop_reason, usage}.
- user adds: toolUseResult, sourceToolAssistantUUID, isMeta, isCompactSummary.

**Content blocks.** text{text}, thinking{thinking, signature}, tool_use{id, name, input, caller}, tool_result{tool_use_id, content, is_error}. A plain prompt's content is a string.

**Streaming and chaining.** Every streamed block is its own assistant line, chained by parentUuid. A tool_result comes back as a user line whose parent is the tool_use line.

**Compaction.** A system line with subtype compact_boundary (logicalParentUuid, compactMetadata), then a user line with isCompactSummary:true.

**api-request records.** "Every later request and resume sends the record as-is … until compacted." They appear in 27 of 27 files from 2.1.304 but only 1 of 4 from 2.1.267, so they look optional for a synthesized file. Not proven.

**Resume.** -r/--resume [id|name], -c/--continue, --fork-session, --session-id <uuid>. The binary has the string "Unable to load transcript from file", so a file path probably works too.

**Validation.** The binary contains:
- invalid_thinking_signature, with a strip-all-thinking-and-retry path (tengu_thinking_signature_strip_retry);
- tool_use_id_invalid (pattern), duplicate_tool_use_id, unexpected_tool_result, tool_use_mismatch, empty_text_block.

## Codex CLI
**Path.** `~/.codex/sessions/YYYY/MM/DD/rollout-<ISO>-<uuid>.jsonl`; 405 files.

**Line shape.** {timestamp, ordinal, type, payload, metadata}. Types:
- session_meta, which must be first;
- response_item, event_msg, turn_context, compacted;
- token_usage_record, world_state, inter_agent_communication_metadata.

**session_meta payload.** id, session_id, cwd, cli_version, originator, source (cli/vscode/exec/subagent), model_provider, base_instructions, git{branch, commit_hash, repository_url}, history_mode, dynamic_tools, context_window.

**response_item kinds.**
- message{role user|assistant|developer, phase, content[input_text|output_text|input_image]};
- reasoning{id rs_…, summary[] (empty in practice), encrypted_content};
- custom_tool_call{call_id, name, input} with custom_tool_call_output;
- function_call{name, namespace, arguments:str} with function_call_output.

**Tools.** Freeform `exec` in code mode (JS input) dominates the corpus. There are no local_shell_call items today.

**compacted.** replacement_history[] holds messages plus an opaque compaction{encrypted_content}.

**Resume.** `codex resume [ID|name]`, --last, `codex fork`, `codex exec resume`. The lookup goes through ~/.codex/state_5.sqlite table `threads` (id, rollout_path, cwd, …), with a fallback that scans filenames and a backfill. The first line must be session_meta.

**Built-in importer.** `app-server/src/external_agent_migration/session_importer.rs` and the table `external_agent_config_imports`. Codex already imports Claude sessions.

## Our harness (Bboxy)
Envelope{seq, epoch, cause, event}. Items:
- user_message, agent_message;
- reasoning{summary, raw, provider_opaque};
- tool_call{call_id, name, tool_kind, input_raw, output[]};
- file_change, turn_diff, terminal, plan, approval, image{blob};
- compaction, context_trim, usage, notice, subagent, unknown.

Resume uses --resume on a closed session (headless.rs:70,108) or in seat mode.

## Loss table
| Direction | Drop | Transform | Keep |
|---|---|---|---|
| Claude→Codex | thinking and signature; api-request | tool_use/result → function_call/_output with call_id preserved; Bash/Edit/Read have no Codex tool, so they become text or a generic function call | text, tool results as text |
| Codex→Claude | encrypted reasoning; encrypted compaction | exec JS → tool_use with the JS kept, or text; developer messages → system/meta | message text, tool outputs |
| either→Bboxy | none material | opaque reasoning → provider_opaque | all |
| Bboxy→either | provider_opaque, foreign usage | tool_kind/name → target names | text, tool I/O, images |

## Hard blockers
1. Reasoning signatures and encrypted reasoning are provider-bound. Drop them on cross-provider lowering; only readable text survives.
2. Switch only at a turn boundary, where the last line is a user turn. Thinking-enabled continuation after a tool_use expects a leading thinking block. This comes from API knowledge and is not verified locally.
3. Codex encrypted compaction can't be read. Start from replacement_history or rebuild from the pre-compaction history.
4. Tool vocabularies differ (Codex code-mode exec). Lower foreign tool calls as text summaries or outputs.
5. Codex resume depends on the state_5.sqlite threads index. Prefer a supported entry point (Codex's importer or `codex resume` with a file it can backfill) over writing its database.

## Next proof (unattended)
Synthesize a two-turn session and run `claude -p --resume <id|path>` and `codex exec resume <id>` against it, in a throwaway HOME/CODEX_HOME with test credentials. The exit code is the verdict.