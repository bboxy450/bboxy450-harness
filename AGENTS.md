# bboxy450-harness — agent instructions

This repository uses a GUI-first coding-agent harness built for the bboxy450-term suite. Read `.bboxy/README.md` first, then `.bboxy/specs/harness-zoomed-out.md`. The interface control documents (ICDs) in `.bboxy/interfaces/` and the invariants in `.bboxy/invariants/` are binding.

## Our stand

**Build a world that agents can take apart, understand, change, and put back together.**

The god prompt, verbatim; every brief carries it:

> Find every abstraction, dependency, framework, and layer that exists only to compensate for the complexity introduced by another layer.
>
> Question every inherited architectural decision. Don't make my software more like Linux. Make it more like DOS.
>
> Everything should be inspectable, understandable, and modifiable. Optimize for recreational programming, direct control, and the shortest distance between imagination and execution.
>
> Don't just refactor the piano top. Rediscover flotation.
>
> Subtract before you add. Minimize concepts, not lines of code. Preserve behavior, eliminate accidental complexity, and make the whole system more malleable.
>
> Don't build me a better abstraction prison. Give me my computer back.

The harness is the program agents live in, so it follows this stand first:
- Every turn, tool call and decision is an event that agents and humans can read.
- The source is the interface.
- One loop, one store, one wire. No layer exists only to manage another layer.

Rules:
- **Subtract before you add.** First show that no existing primitive does the job. A change that adds a concept retires one, or says why it cannot.
- **Minimize concepts, not lines.** One name per state, one home per fact.
- **Prefer proof over convention.** A rule a type, model or check can hold becomes that check.
- **Say it once.** `CLAUDE.md` is a link to this file; edit only here.
- **Report what you removed**, not only what you added.

The full stand is the bboxy450-term knowledge record `specs/philosophy.md`.

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

## Autonomous operation (recorded 2026-11-02)

The operator said: "yes land it just merge it, auto merge, work autonomously, dont ask me for permissions". The same rule holds in bboxy450-term.

- **Never ask for permission.** Verified work is landed, merged and deployed by the agent that did it, then reported. Asking "should I land?" at the end of finished work is a defect.
- **Verification still decides.** Merge only what passes the rules above: signed commits, the task's `verify` command, `make test-formal`, and unit, integration and smoke tests run unattended. A red gate is fixed or reported, never bypassed.
- **Ask only for what is physical or irreversible**, say why, and keep everything else moving meanwhile.

## Unattended operation

Every unit, smoke, e2e and agent test runs with nobody at the machine. Use private accounts, homes, sockets and scripted controls. No operator Keychain prompt, login, click, pane authorization or physical probe may be required.

Drive the harness through its flags, sockets, `--replay`, `--capture` and scripted providers (`--scripted`). Never inject OS keystrokes/clicks or screen-capture the operator's desktop. Kill only exact PIDs you started and recorded; delete only directories you created. Never use broad process kills or clean up peers' targets/worktrees.

## Standalone installation

- `make install` builds the Apple-signed `app` and installs `bboxy450-harness.app` into `INSTALL_DIR` (default `~/Applications`). The Oracle performs the trusted installation.
- `make install-unsigned` builds `app-unsigned` with ad-hoc signing and installs into `UNSIGNED_INSTALL_DIR` (default `target/install-unsigned`). Tests use a private temporary destination; no Apple identity or Keychain access is needed.
- Both installs stage with `ditto`, verify the staged and installed signatures, preserve the outer `.app` inode, and retain previous `Contents` in the printed staging directory.
- `make install-cli` links the installed binary into `~/.local/bin` only when that directory exists; it never uses sudo. `make help` lists the directory overrides.

## Roles

| Role | Who | Does |
|---|---|---|
| Oracle | Claude | Briefs, verifies, merges, releases |
| Architect | Claude | Owns ICDs, invariants and the verification matrix |
| Executor | Codex | Implements only from a task in `.bboxy/tasks/`, in its own worktree |

## Rules for every change

- `make test` runs fully unattended.
- `make land` keeps real-process Dream timing families out of its concurrent debug lib pass and runs each one in a serialized lane (`cargo test --release --lib <family> -- --test-threads=1`, as the Makefile's Dream targets do); the families are `SERIALIZED_LIB_FAMILIES` in `scripts/land.py`, and a family excluded from the concurrent pass but not run serialized, or one that runs zero tests, fails the gate.
- Every source file carries the header from `ATTRIBUTION.md`.
- Keys are never written to the repository or to logs.
- Every `.md` file under `.bboxy/` is an indexed record. It needs frontmatter with an `id` and a `number`, or the knowledge index refuses every write. Put run reports and logs in `.bboxy/runs/` as `.log` or `.json`, never as `.md`.
