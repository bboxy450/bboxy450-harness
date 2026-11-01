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

P0 settled these routes: log-converter PR #2 at `7945ad92`, report `.bboxy/runs/p0-resume-probe-report.log`, facts F1–F10, Claude Code 2.1.304 and codex-cli 0.164.0, against mock endpoints. The facts are pinned to those client versions. A client upgrade re-runs `make test-probes` before switching is trusted again.

| Target | Placement | Resume | Evidence |
|---|---|---|---|
| Claude | write the lowered session to a **private staging file** `<state>/seats/<seat>/native/claude-<new-uuid>.jsonl`, created with `O_EXCL` and mode 0600. That is outside `~/.claude`, so the converter writes nothing there. Claude appends to this file while the session lives, so it is kept until the seat closes. Node uuids stay unique: duplicate uuids make Claude drop history silently with exit 0 (F4), so lowering is tested on the model input, not the exit code | `claude --resume <staging path>` | F2, F3; F4/F5 controls |
| Codex, IR with no Codex-sealed events | **Codex's importer**: write the session as Claude-format text into a disposable staging tree `<tmp>/home/.claude/projects/<slug>/<uuid>.jsonl`. Run `codex app-server` with `HOME=<tmp>/home` and the real `CODEX_HOME`, then: initialize with `experimentalApi`, detect, import SESSIONS, wait for the completed import history (an importId alone is not success, F8), and resume the returned id. Codex writes its own rollout and database. `CLAUDE_CONFIG_DIR` does not select the tree (F8) | `codex resume <returned id>` | F7, F8 |
| Codex, IR with Codex-sealed reasoning or compaction (a session that ends in Codex after A→B→A) | **open**. The importer carries only Claude text, so Codex's own encrypted reasoning would be lost. The fallback is a direct rollout placement (F6: `session_meta` first, then filename lookup backfills `state_5.sqlite`), still under [[d4972-session-switch-native-write-exception]]. That needs proof of same-provider opaque fidelity first. Until then, use the importer route and count the loss as `dropped{sealed_reasoning_route}` | `codex resume <id>` | F6, F10 control |
| Bboxy | `bboxy450-harness import <bxl>`. The harness writes its own session, and term still never writes `~/.bboxy450-harness` (#348 Versioning) | `bboxy450-harness seat --resume <id>` | P5 |

**Coverage limits** carried to P6: P0 proved two-turn text sessions only. Tools, images, signed thinking, encrypted reasoning and compaction, long cwd slugs and interactive TUI resume are untested. The F9 reasoning summary is accepted by the mock only, not the real Responses service. P6 measures each of these per route and records any loss in the LossReport.

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
- (e) placement overwrites an existing native file → `SourceUntouched` fails. The same control exists as a test: a staging or rollout placement without `O_EXCL` over an existing path must make `make test-session-switch` fail (d4972).
- Retained client controls from P0: F4 (duplicate uuids) and F10 (`session_meta` not first) must keep failing on the model input.

## 6. #348 rows that change (for the ICD #348 owner; the Oracle signs under d4071)

| #348 row | Change |
|---|---|
| Amended by | add: `bboxy450-harness:[[session-switch]]` |
| States and lifecycle | add: a seat may change `harness_kind` at `generation + 1` through `ctl seat switch`, only at a turn boundary. Seat input is refused with `seat_switching` while the switch runs. Failure rolls back to the source harness resuming its untouched native session |
| Versioning | **unchanged for Claude**: the converter stages Claude sessions outside `~/.claude` (F2). **For Codex**, Codex's own importer writes `~/.codex` (a client write, listed with cause `ctl seat switch`) from a disposable staging HOME (F7). The [[d4972-session-switch-native-write-exception]] create-only exception stays only for the open Codex direct-rollout route (§3), and lapses when that route is either proven or dropped |
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
- **2026-11-01, P0 facts** (Oracle message 01M6CMNMTH3NV2PAY970J8NSXD; log-converter PR #2 `7945ad92`). §3 now rests on evidence: Claude is staged privately and resumed by path, and Codex goes through its importer from a disposable HOME. The d4972 exception is withdrawn except for the open Codex direct-rollout route, which same-provider sealed fidelity needs. The facts are pinned to the client versions.

Owner: Nyx Copperwing <ARCHITECT> (Side B and the protocol). Side A: bboxy450-term, ICD #348.
