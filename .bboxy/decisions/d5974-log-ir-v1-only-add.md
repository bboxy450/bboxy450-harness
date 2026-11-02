---
id: 01M6F4R15RY136PXM68TSXV7Y9
number: 5974
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
- **Starting fresh at 1.0 with a cleaned-up shape.** Rejected. It breaks every existing log for cosmetic gains. The few awkward shapes, such as the `fork` tuple, stay forever.

**Consequences.**
- Every future protocol change is either a pure addition or a new major with a reader for all older ones. The second needs an Architect decision and is not planned.
- [[protocol]]'s change rule ("additive = minor") is now enforced, not advisory.
- The code currently violates E5/E6 in two places (fork re-serializes; GUI rejects major ≠ 0). Brief [[2026-11-02-log-ir-v1-compat]] makes the code compatible by construction before any v1.0 addition lands.
