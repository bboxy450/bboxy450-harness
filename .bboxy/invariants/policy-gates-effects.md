---
type: invariant
title: "no write, exec or network-capable tool runs without a current-generation policy decision"
status: active
check: cargo test --test policy_gate && make test-formal MODELS=ApprovalLifecycle
cost: "a model-chosen command runs that nobody, and no rule, allowed"
links:
  - { to: "[[protocol]]", rel: constrains }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 6: no side effect outside policy

**Property.**
- **Effect classes.** Every registered tool declares `effect: ReadOnly | Write | Exec | Network`. MCP tools are `Exec` unless they carry `readOnlyHint`.
- **The gate.** The only path from a `ToolCallReady` to `Tool::run` for a non-ReadOnly tool goes through `policy.decide(&Action) -> Allow | Ask | Deny`.
- **Ask.** `Ask` creates an `Approval` item. The tool runs only after that item completes with `Allow` or `AllowAlways`, and only for the same `(turn, generation)`.
- **Resolution.** Each approval resolves exactly once: by a face, by a rule, by `Unattended`, or as `Cancelled` with its turn.

The default policy is: ReadOnly is allowed, and everything else is asked. A profile may set `policy: yolo`, which is the operator's seat convention. Yolo is a rule-set (`Allow *`), so the gate and the `Approval` item are still recorded.

**Check.**
- `tests/policy_gate.rs`:
  - A registry walk asserts every tool declares an effect.
  - A test-only counting `Tool` run with policy `Deny *` asserts zero runs.
  - A stale approval from the previous turn is refused.
- Model ApprovalLifecycle must pass. BrokenApprovalLifecycle runs on a stale approval and must be rejected.

Owner: Nyx Copperwing <ARCHITECT>.
