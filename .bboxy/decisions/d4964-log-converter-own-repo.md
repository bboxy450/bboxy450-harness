---
id: 01M6CK65TFFXVF9YPVF44R5SZP
number: 4964
type: decision
title: "bboxy450-log-converter lives in its own repo, git-pinned by harness and term"
status: superseded
superseded_by: d5087
proposed: "2026-11-01"
links:
  - { to: "[[log-converter]]", rel: decides }
  - { to: "[[d4949-log-converter-from-scratch]]", rel: constrained-by }
---
# bboxy450-log-converter lives in its own repo

**Superseded by [[d5087-log-converter-in-term]]** (operator: "no new repos, all crates in bboxy450-term repository"). The dependency direction below still holds: the converter depends on neither the harness nor term's application crates. The repo, pin and authorization parts no longer apply. **The rejection of (b) below was wrong.** The harness already depends on seven term-hosted toolkit crates by git pin (harness `Cargo.toml:80-88`), so living in term does not invert anything. What matters is the crate-level direction, which d5087 keeps.

Proposed 2026-11-01 by the Architect, Nyx Copperwing, at the Oracle's request (message 01M6CJZSEMEF8MFSYJQ7B33CKP).

**Context.** The crate parses and lowers Claude, Codex and Bboxy sessions. The harness consumes it (`import`, and later the GUI), and bboxy450-term consumes it (`ctl seat switch`). It is also a standalone CLI that the operator wants published.

**Alternatives.**
- (a) A workspace member of bboxy450-harness. Rejected: term would then need the harness workspace to get the converter, and term already embeds the harness as a submodule (`crates/bboxy450-harness`). That would couple their landing gates and their 900 s presubmit (d5002).
- (b) A workspace member of bboxy450-term. Rejected: the harness would depend on term, which inverts the dependency direction.
- (c) **Its own repo, `bboxy450/bboxy450-log-converter`, consumed by a git pin** in harness and in term, the way the toolkit is. Chosen.

**Choice and consequences.**
- **Dependency direction**: converter ← harness, and converter ← term. The converter depends on neither. It carries its own codec for Bboxy protocol v0.4 JSON, written against [[protocol]] §3–§4, and proves it against the harness: its CI runs the harness `check_transcript` on lowered Bboxy fixtures, using a pinned harness revision as a dev-only tool.
- **Its own gates**: `make test` and `make test-formal`, using bboxy450-tlc + pinned Java TLC (d365, d4949), plus the KB, ATTRIBUTION.md (the "Inspired by" credits from d4949) and signed commits. Releases go to crates.io as bboxy450 by the signed release order.
- **Pin bumps** in harness and term are ordinary PRs with a reason. The KB records for this system ([[log-converter]], [[lc-ir]], [[lc-bxl]], [[lc-watch-sink]], [[session-switch]]) stay in the harness KB, where d4949 is, until the repo has its own KB. They move in P1 with a forwarding note.
- The operator's merge authorization is recorded only for bboxy450-harness. The new repo needs its own recorded authorization before the Oracle merges there. That is an operator action at bootstrap, so it is asked once, together with the repo creation (P1).

**Reason.** One owner per component with a one-way dependency graph, and no single gate that every component waits on (the operator's no-chokepoint invariant).
