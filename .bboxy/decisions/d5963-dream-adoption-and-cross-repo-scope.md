---
type: decision
title: 'Dreaming: auto-adopt into low-trust context sinks; cross-repository evidence is opt-in'
status: accepted
links:
- to: '[[swarm-dreaming]]'
  rel: decides
id: 01M6J5RBTNYGZMSJQ0WF40DP6B
updated: 2026-11-04T02:21:07.550Z
created: 2026-11-04T02:21:07.550Z
number: 5963
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:c443571b5b50d6e7a024cd3ee7477f8785f5d1b64d0cc952e86a18018c6862ac
  frontmatter_digest: sha256:0a7a994f4a453a4126bbc22e2c81bf4ff8f21df11b73010996e5297027dbd401
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: fWFEuQz/EVUJruasJLx4nc8ncluediATChkzBEnaIFM0oraaUuI+OnPh2yJapRccTOnQAprBbgTOSxWPifW4Cw==
  log: .bboxy/log/2026-11-04.jsonl#40142
---
# Dreaming: auto-adopt yes, cross-repository evidence opt-in

**Date:** 2026-11-03. **Decided by:** the operator ("cross repo evidence - opt in, auto-adopt yes"). **Recorded by:** Vesper Greywind <ARCHITECT>.

## Decision

1. **Auto-adopt: yes.** A dream candidate that passes its scripted fork-replay A/B ([[swarm-dreaming]] §3.4) is adopted into the low-trust context sinks with no human step. Those sinks are Claude Code memory, Codex memories, and engine/ACP start context.
   - Instruction sinks still take proposals only, under taint barrier I2. Instruction sinks are `~/.agents` skills, `AGENTS.md`, and `.bboxy` invariants and decisions.
2. **Cross-repository evidence: opt-in.**
   - By default, a dream cites evidence only from sessions whose `cwd` is inside the repository it dreams for.
   - A repository joins the shared pool only through an explicit, recorded opt-in. Evidence pools only among opted-in repositories.
   - Redaction (I5) applies everywhere.

## Alternatives considered

- **Proposals only, everywhere.** Rejected. It puts a human gate on every memory, which the never-block policy forbids, and output nobody consumes is cost.
- **Cross-repository by default.** Rejected. The Codex logs (6.2 GB) span unrelated projects, and leaking context between them is a privacy and poisoning risk.

## Consequences

- The `DreamLifecycle` model must include the transition from tested to adopted with no human step, and must carry the cross-repository scope as state.
- The thresholds for "passing" are spec numbers, set before D1 runs, not judged case by case.
