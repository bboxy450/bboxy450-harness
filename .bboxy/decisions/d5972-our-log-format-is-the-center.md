---
id: 01M6F45VPBP7AGX33087HZ3FKX
number: 93377961361018
type: decision
title: Our log format is the center — every backend writes it, every face renders it, native logs import into it, ACP is the wire between processes
status: accepted
links:
  - { to: "[[protocol]]", rel: constrains }
  - { to: "[[acp-agent]]", rel: constrains }
  - { to: "[[acp]]", rel: constrains }
  - { to: "[[session-switch]]", rel: supersedes-in-part }
  - { to: "[[lc-ir]]", rel: supersedes-in-part }
  - { to: "[[d4972-session-switch-native-write-exception]]", rel: narrows }
  - { to: "[[d5087-log-converter-in-term]]", rel: narrows }
created: 2026-11-02T22:00:00Z
updated: 2026-11-02T22:00:00Z
---
**Date.** 2026-11-02. **Recorded by** Lumen Foxglove <ARCHITECT>, on the Oracle's brief carrying the operator's direction.

**Operator direction (via the Oracle).** "OUR LOG FORMAT IS THE CENTER (every backend writes it, every face renders it, ACP at process edges both ways)."

**Decision.**
1. **One canonical log.** The session log is the [[protocol]] `Envelope` JSONL (v0.4) written by `Store`. It is the only history the harness trusts, renders, forks, compacts and hands off. Laws [[log-is-truth]] and [[one-ending]] apply to it whatever produced the events.
2. **Every backend writes it.** The engine, the ACP agent backend ([[acp-agent]]) and the native-log watcher (`src/import/watch.rs`) are interchangeable session actors behind one `Gateway`. A new backend is accepted only if its output passes `check_transcript`.
3. **Native logs import, they are not written back.** Claude Code and Codex files are *read* and projected, line by line, into our events (`src/import/mod.rs`, pure streaming `Projector`). Unknown records add nothing. Continuing such a session is `Host::switch` (`src/gateway/host.rs:237`): fork our log at its head onto any profile, close the old session, and give a fresh agent the conversation as handoff text ([[acp-agent]] §5). The harness writes no file under `~/.claude` or `~/.codex`.
4. **ACP is the wire between processes, both ways.** In: the harness serves ACP v1 to clients ([[acp]]). Out: the harness drives ACP agents ([[acp-agent]]). A future remote seat (ssh, cloud) is an ACP stdio stream over that transport; it carries `session/update`s, never our log file, and the receiving side writes its own log. Inside one process, faces use `Gateway` and `protocol` directly, not ACP.
5. **Face parity is measured on the log.** Every face renders the same envelopes; native-vs-web rich-face parity is gated on replays of committed logs (matrix R27, [[interfaces]]).

**Alternatives.**
- *A neutral IR (lc-ir/bxl) as center, lowered to each native format* ([[lc-ir]], [[session-switch]]). Rejected as the center: it adds a second format and native write-back with per-client-version probes (F1–F10), and the converter crate was never built (no `crates/bboxy450-log-converter` in bboxy450-term at e624d888). Our protocol log already carries every item kind the faces render.
- *Native resume (`claude --resume`, `codex resume`) to switch.* Rejected as default: it ties continuity to a client's private format and version; handoff through our log works across any backend.
- *ACP as the in-process face API.* Rejected: it cannot express our items losslessly (R12 lists the gaps), and `Gateway` already gives lanes, replay and backpressure.

**Consequences.**
- [[session-switch]] and [[lc-ir]] stay *proposed* and are off the critical path. Native write-back under [[d4972-session-switch-native-write-exception]] is allowed only as an explicit export, never as how a switch works. [[d5087-log-converter-in-term]] still governs where a converter would live if one is built.
- Handoff is lossy by design (text, 24 KiB, newest kept). The loss is visible: the new agent's first notice counts the handed turns, and the full history stays in our log.
- A projector or backend that emits an event `check_transcript` rejects is a defect in that projector, not a reason to relax the checker.

**Verification.** `tests/import_native.rs` (Claude and Codex projection; a watched log continued on an ACP agent with the handoff), `tests/acp_agent.rs` (switch forks and hands over), model `HarnessAcpAgentTurn`, and `check_transcript` on every produced log. Unmodelled: file growth and truncation of a watched log (`verification/coverage.toml`).
