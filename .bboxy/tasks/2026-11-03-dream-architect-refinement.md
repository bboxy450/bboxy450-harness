---
type: task
title: 'Dreaming: refine IR, sinks, units and work-package interfaces'
role: architect
worktree: /Users/feral/mydev/bboxy450-harness
allowed_paths:
- .bboxy/specs/dream-log.md
- .bboxy/specs/memory-graph.md
- .bboxy/interfaces/dream-sinks.md
- .bboxy/specs/log-ir-v1.md
- .bboxy/decisions/
- .bboxy/reports/
verify: git diff --check && rg -q dream-log .bboxy/specs/log-ir-v1.md
id: 01M6J7JVTEKEBV9F19DQ2GDEQQ
status: claimed
delegated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
updated: 2026-11-04T02:53:10.928Z
created: 2026-11-04T02:53:04.293Z
number: 5984
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
assignee: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
provenance:
  digest: sha256:4e30773d08e8b411f99587ec05db80bc07603a5a93f5d4ba1fcd2f5f548b82c3
  frontmatter_digest: sha256:ce17d7d016e561e14cb9bca4792c829eeb557f31252e48863d9ba7a518041d4b
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: kNyAjZIfAG6W5LT+8tJDLZaP4SBXeM92TKA2y1vcplI6TqjoHkatGr3d0u0IkSO+IJEZYCE34bRyy6AK+cwmDQ==
  log: .bboxy/log/2026-11-04.jsonl#40262
---
ROLE: ARCHITECT
Parent: [[2026-11-03-swarm-dreaming-succession]] (#5981).
Worktree: /Users/feral/mydev/bboxy450-harness (shared KB; documents only through knowledge_write/knowledge_commit).
Read [[swarm-dreaming]], [[d5963-dream-adoption-and-cross-repo-scope]], [[d5974-memories-are-kb-records]], [[kb-weighted-links]], [[memory-formats-sota]], [[memory-graph]], [[dream-log]], [[dream-sinks]].

Outcome: refine and baseline implementable contracts with Nyx. Add log-ir-v1 §4c referencing dream-log §2 and change log. The local root lacks log-ir-v1; read its exact origin/main version first and coordinate with Nyx before importing it. Confirm T3 usage.json used units from producer source with file:line evidence, and correct the consumer ICD. Identify term owners for T1 cross-root link resolution and T4 agents root mount; provide precise owner handoffs to Nyx (no outside-repo writes).
Resolve contract defects before briefing dependent Executors: digest recheck followed by rename leaves a TOCTOU lost-update race with hand edits; P2 requires preservation but handwritten edits are legitimate; ensure low-trust Recall semantics and concurrent lifecycle per harness are coherent. Review no-global-chokepoint rules against one trial per machine. Numbers change only with a new thresholds revision.
Review WP1-9 path ownership and missing test/wiring paths; propose exact minimal amendments for integration and formal checks, including broken controls and real witnesses. Do not implement code, recruit or merge.

Allowed edits: .bboxy/specs/dream-log.md, .bboxy/specs/memory-graph.md, .bboxy/interfaces/dream-sinks.md, .bboxy/specs/log-ir-v1.md, .bboxy/decisions/, .bboxy/reports/.
Verify: git diff --check; plus exact origin/source evidence for T3 and grep assertion that log-ir-v1 references dream-log. Record actual statuses; no self-report substitutes for evidence.
Report via committed .bboxy/reports/ with Report trailer and task_report. Coordinate source KB commits with Nyx, who publishes by cherry-pick to origin/main and reconciles checkout. Do not reset, stash or overwrite shared dirty files.