---
id: 01M6CK65TFHBYQHHCKNGHABBHH
number: 4963
type: interface
title: "ICD: live session switch between harnesses — bboxy450-term ↔ log-converter (proposed; amends #348)"
status: proposed
links:
  - { to: "[[interfaces]]", rel: index }
  - { to: "[[log-converter]]", rel: part-of }
  - { to: "bboxy450-term:[[app-harness]]", rel: amends }
  - { to: "[[lc-ir]]", rel: uses }
  - { to: "[[embed-seat]]", rel: fed-by }
created: "2026-11-01T00:00:00Z"
updated: "2026-11-01T00:00:00Z"
---
# ICD: live session switch, proposed

**Status: proposed.** This ICD switches a live bboxy450-term seat from harness A to harness B, keeping the conversation. Supported directions are claude→codex, codex→claude, either→Bboxy and Bboxy→either. It amends bboxy450-term ICD #348 (§6). Under d4071, the #348 rows land in a bboxy450-term PR and the Oracle signs them.

| | |
|---|---|
| Side A | bboxy450-term: `bbt-mux` seat lifecycle (generation, `harness_kind`, `harness_session_id` persistence at `logic.rs:6162-6170`), `ctl`, the wake-up launch kit |
| Side B | `bboxy450-log-converter` CLI, run as a child process: `switch-prepare --from <provider> --native <id or path> --ir <seat.bxl.d> --to <provider> --cwd <dir>` |
| Constraint | the converter never runs on term's UI or logic thread, so term spends no frame time on it |

## 1. What crosses

- **Term → converter**: the source provider and its native session id/path, the seat's IR log directory `<state>/seats/<seat>/ir.bxl.d`, the target provider and the cwd.
- **Converter → term**: one JSON line on stdout, then exit 0:
  `{"native_id": "<new id>", "resume": ["<argv>" …], "placed": "<path>", "loss": LossReport, "ir_through": EventId}`.
  On failure: a non-zero exit and `{"error": "<typed reason>"}`. The reasons are `not_at_turn_boundary`, `source_unreadable`, `lower_failed`, `place_failed` and `unsupported_direction`.
- **The seat's IR log is the seat's history across harnesses.** The converter appends the source session's events, then a lineage record `SwitchedFrom{A, native A id}`.

## 2. Protocol

States: `Running(A)` → `Quiescing` → `Stopped(A)` → `Converting` → `Launching(B)` → `Running(B)`. Any failure goes to `RollingBack` → `Running(A)`.

1. **Quiesce.** `ctl seat switch <seat> --to <B>` is accepted only at a turn boundary: adapter `turn_open == false` and no pending approval. Otherwise it is refused immediately with `switch_turn_open`, and nothing waits (never-block). From acceptance until `Running(B)` or rollback, seat input is **refused** with `seat_switching`. It is not queued, so no input can reach the wrong harness.
2. **Stop A** through its normal close path (the embed-seat §3 budgets for Bboxy; the client's own exit for Claude and Codex). The native file is final when A's process group is gone.
3. **Convert.** Run `switch-prepare`. It:
   - parses A's final native session into the seat IR log;
   - checks the IR turn boundary ([[lc-ir]] §2), refusing `not_at_turn_boundary` if it is not one;
   - lowers to B with a **fresh native id** (never reused);
   - places the result through B's entry point (§3).
4. **Launch B** at `generation + 1` with `harness_kind = B`, `harness_session_id = <new id>` and the returned `resume` argv. Running means B's first observation shows the new session loaded (wake readiness where it exists; otherwise the first adapter observation that names the id).
5. **Rollback.** If step 3 or 4 fails, term relaunches A at `generation + 1` resuming A's original native id. A's native file was never written, so the resume is exactly the pre-switch state. The failure is reported to the seat and to the operator as one Notice with the typed reason.

## 3. Entry points (placement)

Phase 0 ([[log-converter]] §7 P0) proves or replaces each row. Until then each row is **unproven**.

| Target | Placement | Resume |
|---|---|---|
| Claude | create-exclusive (`O_EXCL`) new file `~/.claude/projects/<cwd-slug>/<new-uuid>.jsonl`. It never touches an existing file | `claude --resume <new-uuid>` |
| Codex | preferred: Codex's own importer, if P0 shows a non-interactive entry point; otherwise a new rollout file at `~/.codex/sessions/YYYY/MM/DD/rollout-…-<new-uuid>.jsonl` with `session_meta` first, relying on `codex resume`'s filename-scan backfill. **Never write `state_5.sqlite`** | `codex resume <new-uuid>` |
| Bboxy | `bboxy450-harness import <bxl>`. The harness writes its own session, and term still never writes `~/.bboxy450-harness` (#348 Versioning) | `bboxy450-harness seat --resume <id>` |

## 4. A→B→A

Switching back lowers the whole seat IR to A:
- A's original events use retained `raw` ([[lc-ir]] §3), so they come back **byte-identical**, thinking signatures and encrypted reasoning included;
- B's turns are lowered by the loss table ([[lc-ir]] §5).

The result is a **new** native id for A, never an append to the original file. Repeated switches are bounded only by the IR log size.

## 5. Model: `LcSessionSwitch` (TLA+, bboxy450-tlc + Java parity)

- **State:** seat phase (§2), `procs` (the set of live harness processes for the seat), `turn_open`, `input_inbox`, `native` (map id → `{owner, contents_version}`), `harness_session_id`, convert outcome, launch outcome.
- **Actions:** UserInput, TurnStart/End, RequestSwitch, StopA, ExitA, ConvertOk/Fail, LaunchOk/Fail, Rollback, RequestSwitchBack.
- **Bounds:** 2 providers + Bboxy, 2 switches (A→B→A), 1 failure each.

**Safety:**
- `OneHarness`: `Cardinality(procs) ≤ 1`.
- `SwitchAtBoundary`: Converting implies the turn was closed.
- `NoMisdelivery`: no input is delivered to a harness other than the one Running when it was accepted.
- `SourceUntouched`: a native session's `contents_version` never changes after a switch starts from it.
- `SessionIdResumes`: `harness_session_id` always names a placed native session.

**Liveness:** `SwitchEnds` (WF on Stop, Exit, Convert, Launch): every accepted switch ends in Running(B) or Running(A).

**Broken controls:**
- (a) a switch accepted with the turn open → `SwitchAtBoundary` fails;
- (b) B launched before A exits → `OneHarness` fails;
- (c) lowering appends to A's existing file → `SourceUntouched` fails;
- (d) input queued during the switch and delivered to B → `NoMisdelivery` fails.
- (e) placement overwrites an existing native file → `SourceUntouched` fails. The same control exists as a test: a placement without `O_EXCL` over an existing path must make `make test-session-switch` fail (d4972).

## 6. #348 rows that change (for the ICD #348 owner; the Oracle signs under d4071)

| #348 row | Change |
|---|---|
| Amended by | add: `bboxy450-harness:[[session-switch]]` |
| States and lifecycle | add: a seat may change `harness_kind` at `generation + 1` through `ctl seat switch`, only at a turn boundary. Seat input is refused with `seat_switching` while the switch runs. Failure rolls back to the source harness resuming its untouched native session |
| Versioning | **change**: "the app never writes the user's `~/.claude` or `~/.codex`" gains one listed, caused exception. On `ctl seat switch` into Claude or Codex, the converter creates one **new** session file (create-exclusive, fresh id) at the client's session path, plus nothing else. It never modifies or deletes an existing file and never writes `state_5.sqlite`. If P0 shows the client's own importer or file-path route works non-interactively, that route is used instead and this exception is withdrawn for that client. Signed by the Oracle as [[d4972-session-switch-native-write-exception]]; the row is added to #348 when P8 lands |
| Verified by | add: `make test-session-switch` (bboxy450-term), which runs claude→codex→claude and either→Bboxy→either against mock model endpoints in a throwaway HOME/CODEX_HOME, plus the `LcSessionSwitch` model |

## 7. Verified by

| Clause | Evidence |
|---|---|
| §2 boundary and refusal | term test: a switch with `turn_open` is refused with `switch_turn_open` and zero side effects; input during a switch is refused with `seat_switching` |
| §2 rollback | an injected `lower_failed` makes term resume A's original id, and A's file hash is unchanged |
| §3 placement | P0, then `make test-session-switch`: each direction resumes in the real CLI against a mock endpoint, and the mock checks that the request carries the prior turns' text |
| §4 A→B→A | the A-native lines of the final A file are byte-identical to the original lines |
| §5 | `make test-formal` with `LcSessionSwitch` and controls (a)–(d) |

## Change log

- **2026-11-01.** Accepted by the Oracle as the build basis (message 01M6CKHR65S4AQB5744JAV2GG9). The §6 Versioning exception is signed as d4972. Broken control (e), overwrite, is added. The repo is `bboxy450/bboxy450-log-converter` at `f509e8e`.

Owner: Nyx Copperwing <ARCHITECT> (Side B and the protocol). Side A: bboxy450-term, ICD #348.
