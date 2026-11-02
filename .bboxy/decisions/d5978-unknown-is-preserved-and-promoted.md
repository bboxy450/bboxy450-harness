---
id: 01M6F6XPKGEM72DTJVWWVKJ81W
number: 5978
type: decision
title: Unknown data is preserved verbatim in the IR and promoted to typed fields by an automated scanner; the IR grows from observed data, never by dropping it
status: accepted
links:
  - { to: "[[log-ir-v1]]", rel: constrains }
  - { to: "[[d5974-log-ir-v1-only-add]]", rel: builds-on }
  - { to: "[[2026-11-02-ir-unknown-scanner]]", rel: implemented-by }
created: 2026-11-02T23:59:00Z
updated: 2026-11-02T23:59:00Z
---
**Date.** 2026-11-02. **Recorded by** Lumen Foxglove <ARCHITECT>.

**Operator direction (verbatim in substance).** Log into the IR anything we could not parse, marked as unknown, and build an automated scanner that extends our IR the moment we encounter something unknown.

**Decision.**
1. **The edge never drops.** A projector writes what it cannot map as `Item::Native{raw}` and what it did not consume of a mapped record into `Envelope.ext[provider]`; a value it cannot type is kept beside the `Unknown`/`Other` it had to write ([[log-ir-v1]] §7).
2. **Readers keep payloads.** `Unknown` variants carry the whole raw value (E4), so an older build holds, forwards and can display a newer kind by name.
3. **A scanner turns unknowns into additions.** `ir-scan` catalogues every unknown by shape and emits an additive proposal (schema fragment, projector arm, fixture, task record). It runs at session close, in `make test` over fixtures, and in the night run ([[log-ir-v1]] §8).
4. **Promotion is gated, not manual.** An extension lands like any cut: executor task, `make test-ir` proves it is an addition, and `land.sh` refreshes the lock. The Architect is not a step in the loop.
5. **Promotion is retroactive.** Because the raw data is in the log, `reproject` can derive upgraded logs later; originals are never rewritten.

**Alternatives rejected.**
- *Status quo: skip what we don't understand* (`_ => {}` in the projectors, text placeholders in the ACP backend). The data is gone before anyone can decide it mattered.
- *A Notice per unknown.* Keeps a name, loses the payload; nothing can be built from it.
- *Runtime self-mutating schema* (write a new typed variant the moment it is seen). The contract would float with every vendor quirk and names would be chosen by nobody. The `ext` slot already stores the data with zero schema change; typing is a reviewed addition, generated from evidence, landed by the gate.
- *The Architect approves every promotion.* A chokepoint; the gate already proves the only property the Architect would check at that moment (additivity). Naming and meaning are reviewed after landing and corrected by addition.

**Consequences.** Logs get larger where a provider sends things we do not model yet; that is the point. The fixtures stop being hand-written: real unknowns become the corpus. Two executor briefs: [[2026-11-02-log-ir-v1-compat]] (types: `Unknown(raw)`, `Native`) and [[2026-11-02-ir-unknown-scanner]] (edge rules, scanner, reproject, landing hook).
