---
type: decision
title: Original verify timing binds a committed run log to its exact task pin
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[dream-log]]'
  rel: clarifies
  weight: 3
id: 01M6MC356HPAM75R7EDFXR5ZKX
updated: 2026-11-04T22:50:21.690Z
created: 2026-11-04T22:50:21.690Z
number: 20261103136607
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:26b8193810832f1f237b8a74264b6f3d8a5bd2d30716d3b201cfc7709999b4f9
  frontmatter_digest: sha256:04b6a47e6596b57266f8831e67ce840a5069853b74016d81c916d65dbde64c55
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: NDdEh5I1uFqC5/3vml4nxUeLp2L3Q5AvOa2clYmmtpoX2CX8RUKJMUfn7vmNlAiLZqqglaiTJgFtpWaF/q+LBA==
  log: .bboxy/log/2026-11-04.jsonl#46229
---
Oracle ruling for WP7 blocker01M6MBZBRNQZQQYE2K03B28KCA. Owners: Nyx Copperwing and Executor01M6KDR90VJTRA1S722FZFECRZ. This specifies dream-log §4.3's original-run elapsed observation; it changes neither the600000ms limit nor the absent-timing fallback.

Evidence: join::Task has a complete committed Record evidence pin, verify string and verify_result.exit_code, but no run locator or timing contract. trial.rs currently reads /verify_result/elapsed_ms from the task alone. Existing Root::read(path,commit) already reads regular committed objects at a pinned ancestor of the mounted origin/main with bounded owned Git capture; no new reader or join/IR field is needed.

BINDING (optional task-frontmatter convention, consumed only by trial.rs)
The exact pinned task may contain:
verify_result:
  exit_code: 0
  run_log:
    path: .bboxy/runs/original-verify.json
    digest: sha256:<64 lowercase hex digits>

run_log is ONE object, not an array, inferred link, filename search or mutable latest pointer. It has exactly path and digest. The path is a literal normalized repository-relative path below .bboxy/runs/ with .json suffix; reject absolute paths, empty/dot/parent components, backslashes and NUL. Read it from the SAME root and SAME commit as the fully resolved task pin, through the existing mounted Root::read. Never use the working tree, origin/main's moving tip, another root or a separately supplied commit. Verify SHA-256 of the exact file bytes before JSON parsing. The task's own Git pin binds both records at once; the run log must not contain a self-referential task/commit digest.

LOG BODY
One strict UTF-8 JSON object, no duplicate or extra keys:
{"schema":"bboxy.dream.original-verify/1","task":{"id":"<exact task id>","path":"<exact pinned repository-relative task path>"},"command":"<exact task.verify string>","exit_code":0,"elapsed_ms":1234}
The nested task object has exactly id/path. Both match the resolved task pin verbatim; command is byte-exact, with no shell normalization; exit_code is an integer equal to the recorded task verify_result.exit_code. elapsed_ms is an unsigned integer0..u64::MAX measured in MILLISECONDS. It is the monotonic wall duration from immediately before admitting/invoking that original verification command until its actual owned completion, including admission/spawn/wait/owned-IO completion. Not CPU time, timestamps, a model estimate, a sum of overlapping subcommands or the later trial's duration. Exact600000 is eligible by timing;600001 and above excludes the episode. Zero is a real measured value, never a missing-value default.

Trust remains the existing pinned task/root/signature/evidence policy; a matching log is a recorded measurement, not independent proof that arbitrary original producers told the truth. No new authority is inferred from JSON or its digest. No existing accepted task/report is rewritten here. Test fixtures can commit their measured log and binding together, with no circular pin.

DISPOSITIONS
- With NO run_log binding and NO explicit legacy timing field, timing is absent. Apply the written fallback: eligibility only when verify is not make test, plus every other existing eligibility/isolation check. This is not a claim that no undeclared log exists.
- A declared null/malformed/missing/unreadable/uncommitted/oversized/wrong-root/path/digest/schema/identity/command/exit log, absent or invalid elapsed_ms, duplicate keys or conflicting bindings is UNAVAILABLE: exclude that episode with an explicit Inconclusive reason, not absent-timing fallback. No directory/report scan chooses the fastest/latest/passing alternative.
- Legacy task verify_result.elapsed_ms by itself is unbound timing and UNAVAILABLE, regardless of whether its number is small. Do not use it as positive evidence or silently treat it as absence. If it coexists with the valid bound log, require an unsigned integer exactly equal to log.elapsed_ms; a mismatch/malformed/null is unavailable. Other explicit legacy timing spellings under verify_result (elapsed, elapsed_seconds, duration_ms) have no approved conversion and are unavailable, never ignored into the fallback. Thus known declared >10min observations cannot become eligible by dropping or overriding timing.
- This initial contract binds directly from the task. A report citation, Markdown link or report prose does not establish the binding and is not parsed for units or implicit task joins. No generic report resolver or report-to-log inference is authorized. A future producer can adopt the same explicit binding deliberately; this change does not manufacture historical measurements or scan unrelated runs.

IMPLEMENTATION / CHECKS
Implement the pure parser/binding resolution and disposition in already allowed src/dream/trial.rs; reuse resolved task bytes/frontmatter and Root::read. Do not change join::Task, resolver/cache/trust APIs, IR/schema, thresholds, global index or existing records. Existing tests/dream_pipeline.rs, tests/support/dream.rs and trial fixture paths cover fixture creation and the actual prepare path. Parse bounded bytes off the runtime worker as existing budgets require. Commit the measured private original-run fixture log with its task binding; stop relying on the old inline-only elapsed fixture.

Tests must exercise actual prepare/pinned Git resolution as well as pure parsing:600000/600001, absent non-make fallback and absent make test refusal, legacy-only timing, valid equal mirror and conflicting/malformed mirrors, wrong id/path/root/digest/command/exit/commit membership, duplicate keys, missing or uncommitted log, symlink Git object, working-tree/tip mutation after pinning, unsupported units/floats/negative/overflow. A present slow bound log and conflicting fast/slow logs must not be eligible. Keep actual20-arm engine/fork/Derived/verifier success and all existing assertions green. This is pure evidence parsing/lookup: specialized tests, no gratuitous new concurrency model or changes to the147 pre-WP7 models.

WP7 verify revision4 stays unchanged and covers the new dream units/pipeline checks; full source/report-HEAD formal evidence and the already required composed/default/ACP/clippy/IR/native smoke still apply. Preserve old timing/positive diagnostics and report this amendment in the signed handoff. No canonical gate waiver, source merge/push, live spend or policy/bootstrap.