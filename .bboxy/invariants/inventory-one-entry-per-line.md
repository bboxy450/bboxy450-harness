---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
check: python3 -I scripts/checks.py inventory-shape && python3 -I scripts/checks.py inventory-shape-control
cost: 'every concurrent PR that adds a model, source or exemption conflicts on the same line: PR #8 vs #9, then #10 vs #8 (2026-11-01), so each landing needs a manual rebase'
created: 2026-11-01T13:24:33.111Z
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
id: 01M6BMGZKBHZTWW6PP16QF5K27
links:
- rel: constrains
  to: '[[interfaces]]'
number: 4409
provenance:
  digest: sha256:183bd5c53b57f317b9226e8a5be677d97df5479070206174ecb82dfaf5e09d85
  frontmatter_digest: sha256:80f68958fe8b24ec0c9917694eaee7c1ee8003b0de6fba2d6cfa750f0eb111ee
  log: .bboxy/log/2026-11-01.jsonl#27571
  sig: L9EJWkKGiQ5MSsdPgKvNZSExH2r1A+M3fhaHCIWWPKmUx+1cKelM0gOOZT2d8yNZY+IroRMFe5L/fv/vfmK2Cw==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: active
title: shared inventory arrays are one entry per line with a trailing comma
type: invariant
updated: 2026-11-01T13:24:33.111Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Invariant: shared inventories are one entry per line

**Property.** In every file listed in `INVENTORIES` (`scripts/checks.py`, currently `verification/coverage.toml`):
- an array opens with `key = [` alone on its line;
- each entry is on its own line and ends with `,`, including the last one;
- the array closes with `]` alone on its line;
- comment lines and blank lines are allowed inside the array.

**Why.** Every PR adds a model, a source or an implementation-only entry to these arrays. When the whole array is one line, any two additions change that same line, and git reports a conflict. With one entry per line, two additions at different places merge cleanly. If both add at the end, git sees two adjacent insertions, which a rebase resolves without editing either entry.

**Order.** Append new entries where they belong logically. This invariant does not impose a sort order, because sorting would move existing lines and cause the same conflicts.

**Check.** `python3 -I scripts/checks.py inventory-shape` exits 0 when the property holds. It names `file:line` on the first violation. The broken control `inventory-shape-control` must reject both a one-line array and an entry without a trailing comma. Both run in `make test` (`scripts/test.py`).

**Scope.** A file joins `INVENTORIES` when concurrent PRs keep appending to its arrays. `verification/contracts.toml` holds per-contract arrays that PRs add as whole new blocks; it is not listed until it conflicts in the same way.

Decided 2026-11-01 at the Oracle's request (message 01M6BMETK95X7N8PST8B6FTJYJ). Alternative rejected: a merge driver or `.gitattributes union`. A union merge can silently duplicate or drop entries, and a duplicate model breaks the formal coverage inventory.
