---
id: "0TAX6W03VFY7NCS0PA247MYMZB"
number: 2900
type: decision
title: "PR #86 integration with bboxy450-term main ee6b7d5e: drop the in-crate report, merge main, keep both sides"
status: decided
decided: "2026-11-01"
links: [{ to: "[[2026-10-31-toolkit-widgets]]", rel: amends }]
---
# PR #86 integration

**Context.** PR #86 (`4889321f`) conflicts with bboxy450-term `main` at `ee6b7d5e` ("Add immutable KB reports with task and commit provenance"). The textual conflicts are trivial: both branches append targets to the end of the `Makefile`, and both add module lines to `crates/bbt-doc/src/lib.rs`. E2 cannot write a bboxy450-term `.bboxy/reports/` record, because its knowledge tools are bound to the harness root.

**Decision.**
1. Remove `crates/bbt-ui/reports/` and the `exclude = ["reports/"]` line from PR #86. Reports never live inside a crate.
2. Put the full report in the harness KB as `.bboxy/runs/2026-11-01-toolkit-widgets-report.log`. It must be `.log`, never `.md`, because of the kb-records gate. The `task_report` on task #50 links to it. Task #50's allowed paths are widened to cover that one file.
3. Put no `Report:` trailer on the PR #86 commits. bboxy450-term's report gate validates the trailers that are present; it does not require them on commits from a task in a different knowledge root. If `make test` on the merge shows otherwise, report the exact failing check and its message, and do not work around it.
4. Merge `origin/main` into `harness-widgets`; do not rebase. Resolve both conflicts by keeping both sides: the `Makefile` targets from both, and both `mod` lines in `bbt-doc/src/lib.rs`.
5. Run the full `make test` on the merge commit at default concurrency, then send a new `task_report`.

**Why.** The Oracle has already verified the round-3 content. This is integration hygiene only, and it is the fastest way that keeps the report out of the published crate.
