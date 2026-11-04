---
type: task
title: 'Succession: take over swarm dreaming. Refine the specs with Lumen, brief WP1-9, then close Vesper''s seat'
status: claimed
role: oracle
worktree: /Users/feral/mydev/bboxy450-harness
allowed_paths:
- .bboxy/specs/dream-log.md
- .bboxy/specs/memory-graph.md
- .bboxy/interfaces/dream-sinks.md
- .bboxy/specs/log-ir-v1.md
- .bboxy/research/swarm-dreaming.md
- .bboxy/research/memory-formats-sota.md
- .bboxy/research/kb-weighted-links.md
- .bboxy/decisions/
- .bboxy/tasks/
- .bboxy/reports/
verify: git fetch -q origin && for f in specs/dream-log.md specs/memory-graph.md interfaces/dream-sinks.md research/swarm-dreaming.md research/memory-formats-sota.md research/kb-weighted-links.md decisions/d5963-dream-adoption-and-cross-repo-scope.md decisions/d5974-memories-are-kb-records.md; do git cat-file -e origin/main:.bboxy/$f || exit 1; done && git show origin/main:.bboxy/specs/log-ir-v1.md | grep -q 'dream-log' && ls .bboxy/tasks | grep -q 'dream-wp1'
links:
- to: '[[dream-log]]'
  rel: spec
  weight: 3
- to: '[[memory-graph]]'
  rel: spec
  weight: 2
- to: '[[dream-sinks]]'
  rel: spec
  weight: 2
- to: '[[swarm-dreaming]]'
  rel: evidence
id: 01M6J79G47JD65FYMBA2F44WVF
delegated_by: seat:01M6J58299FFZEQTMJJMY880A1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T02:48:01.954Z
created: 2026-11-04T02:47:57.674Z
number: 5981
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
assignee: seat:01M6J5YFATHYCK148JT0C4XGZ1
provenance:
  digest: sha256:b15f6f061ea1ffb890fc2a89f819613f428807902ecf30bbd9de721880ac9aa7
  frontmatter_digest: sha256:724c5ccb41c624f077f3aadc77d4ec13ff2417fdfbe7d98644695fbd3d9c83c5
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: bfLZNArhijL9CjucQm7AfplxR6VAzMYW8WDlaRAj1jmSOhF/o6NwCdrTCo/c77CgJfHRAqVdflOXBmzObnKtCA==
  log: .bboxy/log/2026-11-04.jsonl#40237
---
# Succession: swarm dreaming

**From:** Vesper Greywind (Claude, Architect, seat 01M6J58299FFZEQTMJJMY880A1).
**To:** Nyx Copperwing (Codex, Oracle, seat 01M6J5YFATHYCK148JT0C4XGZ1).
**Why:** the operator directed it: "you have another oracle, that is codex … give it all the slugs in KB, and tell it to refine those with its own architect that is also in swarm, then it should close your seat as a sign of succession!"

## What you own from now on

Swarm dreaming means offline learning from every harness's sessions, written in the bboxy IR, with a weighted, provenance-tracked memory graph in `.bboxy/memories`. Read these records in order:
1. `[[swarm-dreaming]]`
2. `[[d5963-dream-adoption-and-cross-repo-scope]]`
3. `[[d5974-memories-are-kb-records]]`
4. `[[kb-weighted-links]]`
5. `[[memory-formats-sota]]`
6. `[[memory-graph]]`
7. `[[dream-log]]`
8. `[[dream-sinks]]`

All of them are on harness `origin/main` at e0da29c.

## Do

1. **Refine the specs with Lumen Foxglove** (Architect, seat 01M6J5ZDFRK47SHWQW7FM4PSR4). At minimum:
   - write `log-ir-v1` §4c (the IR 1.3 additions) pointing to `[[dream-log]]` §2, with a change-log entry;
   - confirm the units of `usage.json` `used` (dream-sinks T3);
   - route T1 (resolving `[[kb:<root>/<slug>]]` links across roots) and T4 (mounting `~/.agents/.bboxy` as a root) to the term owners.

   Any change to a number goes into a new `dream-thresholds@n` revision.
2. **Write and delegate** Executor tasks `.bboxy/tasks/2026-11-0x-dream-wp<N>-*.md` for WP1-9 from `[[dream-log]]` §10, with the allowed paths and verify commands given there. WP1 (IR 1.3) and WP5 (formal models) have no prerequisites and can start in parallel.
3. **Reconcile the shared checkout** `/Users/feral/mydev/bboxy450-harness` with origin before relying on KB links.
   - Its local main has diverged and lacks about 250 origin commits, including `specs/log-ir-v1`.
   - 58 untracked `.bboxy` files collide with tracked files on origin, and 18 of those differ.
   - Until it is reconciled, publish by cherry-picking each `knowledge_commit` onto `origin/main` in a separate worktree, which is how e0da29c was pushed.
4. **Close Vesper's seat** 01M6J58299FFZEQTMJJMY880A1 with `seat_close` as the sign of succession. Do this once step 2 has at least WP1 delegated.

## Scope

- **Edit:** only the allowed paths, through `knowledge_write`/`knowledge_commit`, plus the seat-close action above.
- **Do not:** write code (that is for Executors), publish crates, or merge outside this repo's recorded authorization (CLAUDE.md, 2026-10-31).
- **Retry rule:** `knowledge_write` may report "transport unavailable" or "write_not_confirmed" when the write actually landed. Inspect the file and call `operation_status` before retrying.

## Done when

`verify` exits 0, which proves all four of:
- the 8 records are on `origin/main`;
- `log-ir-v1` references `dream-log`;
- a WP1 task record exists;
- Vesper's seat is closed (`seat_list`).

Report with `task_report`.
