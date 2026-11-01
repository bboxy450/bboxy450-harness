# bboxy450-harness — agent instructions

This repository uses a GUI-first coding-agent harness built for the bboxy450-term suite. Read `.bboxy/README.md` first, then `.bboxy/specs/harness-zoomed-out.md`. The interface control documents (ICDs) in `.bboxy/interfaces/` and the invariants in `.bboxy/invariants/` are binding.

## Operator authorization (recorded 2026-10-31)

The operator said: "you have auto merge authorization in this repository, fully autonomous workflow, you also have permission to publish crates to cargo as bboxy450 user, all autonomously, merge, install release and test and everything autonomously".

What this authorizes in `bboxy450/bboxy450-harness`:

- **Merge.** The Oracle merges a PR to `main` without waiting for the operator, once it has verified the PR.
  - Verified means the Oracle read the diff and ran the task's `verify` command, which exited 0.
  - `check_transcript` and `make test-formal` must be green.
  - Every commit must carry a signature.
  - The operator added on 2026-10-31: "you can merge if executor did enough automatic testing, its important to write correct specs, write correct code, do formal verification, do unit - integration - smoke testing". So a merge also needs all of the following:
    - the change matches its spec or ICD
    - any changed stateful contract has its model updated, together with a broken control
    - unit, integration and smoke tests are all green and run unattended

    A PR that is missing any of these goes back to its Executor.
- **Publish.** Crates are published to crates.io as **bboxy450**, following the signed release order in `~/.agents/AGENTS.md`:
  1. `cargo package`
  2. detached `.asc` signature
  3. signed tag
  4. `cargo publish`
  5. push tags
  6. `gh release` with the `.crate` and `.asc` attached

  Run `cargo login` with a bboxy450 token first. Never publish under any other account.
- **Install and test.** Install releases locally and run release tests, smoke tests and e2e tests autonomously.

What it does not authorize:

- Unsigned commits.
- Merges that skip verification, for example a merge that rests only on an agent's own report of success.
- Committing credentials.
- Publishing under another identity.
- Merging in other repositories, unless that repository records its own authorization. The toolkit crates live in bboxy450-term, and publishing them as bboxy450 is covered by the publish permission above. Merges there follow bboxy450-term's own rules.

## Roles

| Role | Who | Does |
|---|---|---|
| Oracle | Claude | Briefs, verifies, merges, releases |
| Architect | Claude | Owns ICDs, invariants and the verification matrix |
| Executor | Codex | Implements only from a task in `.bboxy/tasks/`, in its own worktree |

## Rules for every change

- `make test` runs fully unattended.
- Every source file carries the header from `ATTRIBUTION.md`.
- Keys are never written to the repository or to logs.
