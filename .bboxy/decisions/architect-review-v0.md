---
id: "091XAEFN2502WJG03QC5VK80NT"
number: 2
type: decision
title: "Architect review of the v0 specs: nine amendments, five decisions upheld"
status: decided
decided: "2026-10-31"
operator: []
links:
  - { to: "[[harness-zoomed-out]]", rel: amends }
  - { to: "[[toolkit-lowering]]", rel: amends }
  - { to: "[[protocol]]", rel: decides }
  - { to: "[[embed-seat]]", rel: decides }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Decision: Architect review of the v0 specs

By Nyx Copperwing <ARCHITECT>, 2026-10-31, at the Oracle's request. One round. Each amendment states what it replaces, the choice, the reason and its evidence. The Oracle's spec files are not edited; this record amends them.

## Upheld

The Oracle's five decisions stand:
- one package
- crates.io 0.1.1 with `[patch]`, refined in §6
- the phase-1 process seat, refined in §5
- toolkit gaps landing upstream
- no TUI

The evidence agrees on the first and fourth:
- One package: `toolkit-lowering` packaging; bboxy450-term `scripts/formal.py:70-92` sees only workspace members, so membership is required.
- Gaps upstream: d274 is the precedent for reusable kits.

## Amendments

**§1. A turn has one terminal event type.** The spec's law 1 says "`TurnEnd` or `Error`".
- *Choice.* `TurnEnded{reason}`, where `StopReason::Error{..}` is a variant, plus `TurnQueued` so that queued-then-cancelled turns also end exactly once.
- *Reason.* "Exactly one of two event types" needs a cross-type check, and kittens shipped two endings anyway (`src/agent.rs:221-224`). One type makes the law a count.
- *Alternative rejected.* Keeping the kittens pair for ACP fidelity. The ACP export maps `reason` to `StopReason` 1:1 anyway (`src/acp.rs:681-689`).

**§2. `ItemStarted` is durable, and the envelope carries `epoch`.** The spec says completed items are durable and implies started ones are not.
- *Choice.* Started and completed are both durable. `seq: Option<Seq>` replaces the `durable` bool, and `epoch` plus `Reset` handle a tail lost to an OS crash.
- *Reason.* Recovery cannot close an item it never saw start, so law 1 after a crash needs the start in the log. "Fsync at boundaries" means a published seq can vanish on power loss; without an epoch, a reconnecting face would accept reused seq numbers. The cost is one small line per item.

**§3. A control lane that is never `Busy`.** The spec has "a bounded op inbox → `Busy`" for all ops.
- *Choice.* Interrupt, ResolveApproval and Close go to latched slots and are never refused.
- *Reason.* An interrupt refused because the inbox is full of follow-ups breaks law 4 and the operator's never-block rule. This is the SteerQueue broken control.

**§4. `ItemSnapshot` on subscribe.** The spec has no snapshot.
- *Choice.* On subscribe, the gateway sends a snapshot of each open item's partial body.
- *Reason.* Without it, a window opened mid-answer shows an empty item until `ItemCompleted`. That is correct but looks broken, and it is cheap to fix with a snapshot.

**§5. The phase-1 seat runs line mode, not `--acp` or `--json`, and phase 1 is a core change.** The spec says "launches it as a process seat … (`--acp` or `--json`) … no bboxy450-term core change".
- *Choice.* `bboxy450-harness seat`, an append-only line projection with no cursor addressing ([[embed-seat]] §2). Phase 1a uses the generic `Pty`/`Cadence` record.
- *Reason, part 1: the profile flags.* d20 says a seat is a PTY with a screen and rejects pipe-mode seats because "a seat without a screen breaks every seat invariant: observation, guard, scrollback, resume". Nothing on the bboxy450-term side of the PTY speaks ACP, and JSON on a screen is unreadable.
- *Reason, part 2: "no core change".* That claim is not true. The allowlist is a hard-coded array (`bbt-mux/src/mcp/seats.rs:22`), and generic records are a hard-coded match arm (`bbt-mux/src/adapters/mod.rs:80-93`). Both need edits that amend ICD #348, so its owner must agree.
- *Grade.* The honest grade is Inferred. OSC 133 earns Reported only for `kind == "zsh"` (`wake-up/src/observe.rs:381`).
- *Not a TUI.* Line mode is a pure `Envelope → bytes` function of about 300 lines.

**§6. GUI widgets do not exist in 0.1.1.** The spec says "crates.io 0.1.1" for every toolkit dependency.
- *The conflict.* E2 lands the new widgets upstream (streaming markdown, diff, tool card, editor), but published 0.1.1 does not contain them. A standalone build against crates.io therefore cannot compile the GUI face until a toolkit release.
- *Choice.*
  - The core features (`default`, `acp`) depend on crates.io 0.1.1 as decided.
  - The harness root `Cargo.toml` adds `[patch.crates-io] bboxy450-{ui,doc,render} = { git = "https://github.com/bboxy450/bboxy450-term", rev = "<sha>" }` for standalone builds. When the harness is embedded, Cargo ignores a member's `[patch]` and bboxy450-term's root path patch wins.
  - A crates.io release of the harness waits for a toolkit release that contains the widgets.
- *Check.* R15, one copy of each toolkit crate, in [[interfaces]].
- *Unverified.* Whether Cargo prints a warning for the ignored member patch. It is a Cargo warning, not a rustc warning, so `clippy -D warnings` is unaffected. E1 confirms this on the first embedded build.

**§7. Approvals never wait unattended.** The spec says "ask by default" with no rule for when no face can answer.
- *Choice.* The profile sets `on_unattended: deny | allow`, default `deny`. An `Ask` with no approver-capable subscriber resolves at once with `by: Unattended`, as a model-visible denial with its reason.
- *Reason.* Headless, the seat and the night runs have no human. An approval that waits is an operator-input test (operator rule: "should not require MY INPUT"). This is the ApprovalLifecycle deadlock control.

**§8. The live smoke is not in `make test`, and its key source must not touch the Keychain.** The spec says "the key comes from a `!cmd` … the smoke skips with a labelled SKIP".
- *Choice.*
  - `make smoke-live` is separate from `make test`.
  - The `!cmd` runs under the 10 s timeout. A timeout is SKIP, and 429 past `retry_deadline` (90 s) is UNAVAILABLE. Neither is PASS.
  - At most 3 requests per run.
- *Reason, part 1: unattended runs.* bboxy450-term invariant #283 `e2e-runs-unattended` forbids login-Keychain access in unattended runs. A `!cmd` such as `security find-generic-password` can raise a dialog, and the 10 s timeout turns that hang into SKIP.
- *Reason, part 2: retries.* Kittens' retry-forever policy would hang the smoke on a quota (incidents `102-free-provider-quota`, `generation-retry-exhaustion`).
- *Operator action.* For an unattended night PASS, the operator puts the OpenRouter key in a source that needs no dialog. That is the one physical step, named here.
- *Not read.* I did not read the kittens profile values.

**§9. Budgets for the shared IO runtime.** The spec puts "all I/O on `bbt_rt::handle()`".
- *The conflict.* That runtime has 2 workers for the whole process (`bbt-rt/src/lib.rs:51`). In-process grep, glob, diffs and markdown preparation would starve bboxy450-term's IO when the harness is embedded.
- *Choice.* That work runs on `spawn_blocking` behind a semaphore of 4, with ≤ 1 ms of CPU between awaits on the workers ([[budgets]] §1). The harness view's frame share is ≤ 0.3 ms drain and ≤ 0.6 ms projection, taken from upstream rows, not added to them.

## Model list change

The spec lists 11 models. I keep all 11 and rename InputPreemption (toolkit map) to SteerQueue (the spec). Each model's broken control and upstream shape are in [[interfaces]].

## What would reverse these

- **§5:** a decision superseding d20 (the phase-2 in-process seat).
- **§6:** a toolkit 0.1.2 release that contains the widgets; the git patch is then deleted.
- **§8:** the operator stating that the live smoke belongs in the unattended gate, together with a dialog-free key source.
