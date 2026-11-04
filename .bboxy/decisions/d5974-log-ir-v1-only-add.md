---
id: 01M6F4R15RY136PXM68TSXV7Y9
number: 93377961361017
type: decision
title: The bboxy log IR is locked at v1 and only grows by addition; 0.4 logs are v1 logs
status: accepted
links:
  - { to: "[[log-ir-v1]]", rel: constrains }
  - { to: "[[protocol]]", rel: constrains }
  - { to: "[[d5972-our-log-format-is-the-center]]", rel: builds-on }
  - { to: "[[lc-ir]]", rel: narrows }
created: 2026-11-02T23:00:00Z
updated: 2026-11-02T23:00:00Z
---
**Date.** 2026-11-02. **Recorded by** Lumen Foxglove <ARCHITECT>.

**Operator direction (relayed by the Oracle, "crucial", "very important").** The UI only renders our log, and any harness can run behind ours, so the log IR is the universal, long-lived contract. It must be future-proof like protobuf: only add, never change or remove, backward compatible and tolerant of newer data.

**Decision.**
1. The IR is the [[protocol]] envelope log. [[log-ir-v1]] is its law (E1–E8), and `verification/protocol-v1.lock.json` is its field-level baseline.
2. The baseline is the current 0.4 surface, unchanged. Version `0.x` is declared to be v1, so every existing log stays valid with no migration.
3. The gaps found for rendering any harness are closed by **additions** ([[log-ir-v1]] §4), never by reshaping existing types.
4. Forward tolerance comes from two things. Every union carries `unknown` (reader side). Only an envelope's author serializes it, and everything else passes bytes through (E6).
5. Harness-specific data that has no field yet goes in `Envelope.ext` under a namespace. A typed field is added when the data proves general.
6. `make test` fails on any removal, rename, retype, tag change or new required field (`scripts/ir-compat.py`, with a broken control).

**Alternatives.**
- **CBOR with integer keys** ([[lc-ir]], [[log-converter-sota]]). Rejected as the store format. It would put a second format beside the JSONL every face already reads. JSON gives the same evolution properties under E1–E8, and the gate enforces them.
- **Protobuf/IDL as the source of truth.** Rejected. Generating Rust from an IDL means a toolchain in the build, and serde plus the schemars export plus a lock gives the same "only add" check with no new dependency.
- **A field-level `extra: Map` on every struct** for unknown-field preservation. Rejected. It is invasive on 37 definitions; byte pass-through (E6) preserves everything at the unit that is actually stored and forwarded, the line.
- **Starting fresh at 1.0 with a cleaned-up shape.** Rejected. It breaks every existing log for cosmetic gains. The few awkward shapes, such as the `fork` tuple, stay for the life of major 1; only a major bump (below) retires them.

**Consequences.**
- Every future protocol change is either a pure addition within the major or an explicit major bump. **The bump is the escape hatch** (operator, 2026-11-02): when a drastic change is needed, increase the major, let it be incompatible, and update the entire system together; readers refuse a higher major, and old logs get a one-way converter only if they are worth keeping. The design is never contorted to avoid a bump.
- [[protocol]]'s change rule ("additive = minor") is now enforced, not advisory.
- The code violated E5/E6/E9 in three places (fork re-serialized; the GUI rejected major ≠ 0; the store applied `is_durable()` on read). [[2026-11-02-log-ir-v1-compat]] fixed them before the 1.1 additions landed (Part A 5826403, Part B f73a034).
