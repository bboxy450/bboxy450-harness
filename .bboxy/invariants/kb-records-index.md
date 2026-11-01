---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
check: python3 -I scripts/checks.py kb-records && python3 -I scripts/checks.py kb-records-control
cost: the KB index refuses every write while one file is malformed, so knowledge_write stops for every seat; three outages on 2026-11-01 (runs/e3-gui-report.md unnumbered, runs/e3-pr-body.md without frontmatter)
created: 2026-11-01T06:28:57.802Z
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
id: 01M6AWR0PYENB2BANYS2T6BA24
links:
- rel: constrains
  to: '[[interfaces]]'
number: 2224
provenance:
  digest: sha256:e8ceefc4507a41c4070eff6a750a0e09e687dd15a75ba37ee578e8cc66d2894c
  frontmatter_digest: sha256:455e7568af07d6c155d25c3d15afb170485b0ae87228c65a15dcc136b76cc083
  log: .bboxy/log/2026-11-01.jsonl#12363
  sig: B6WjARSwR8xzp6xMkdbv4dZtms8ODuZW2A2T9PbZAz+pQNYp0Sjj6/1G+OC6CAw7GDdHPZ8jN9TjridQHHG8CQ==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: active
title: 'every .bboxy/**/*.md is a loadable index record: frontmatter with a unique id and a unique positive number'
type: invariant
updated: 2026-11-01T06:28:57.802Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Invariant: KB records index

**Property.** Every file matching `.bboxy/**/*.md` on disk, tracked or not, meets all of the following:
- it starts with a `---` frontmatter block that is closed by `---`;
- the block has a top-level `id:` with a value;
- the block has a top-level `number:` whose value is a positive integer;
- no two records share an `id`;
- no two records share a `number`.

Run output that is not a record goes in `runs/` as `.log` or `.json`, never `.md` (AGENTS.md).

**Why on disk and not tracked files.** The index reads the working tree. An untracked `.md` stops it just as well as a committed one.

**Check.**
- `python3 -I scripts/checks.py kb-records` checks the property. Exit 0 means it held; the message names the first bad file and the rule it breaks.
- `python3 -I scripts/checks.py kb-records-control` is the broken control. It writes a frontmatter-less `runs/report.md` into a temporary `.bboxy` and must see the check reject it. It exits non-zero if the check accepts it.
- Both run in `make test` (`scripts/test.py`), with no network and no operator. They become a gate when PR #4 merges; until then the invariant is **unenforced**.

**Not checked.** The check does not test whether the frontmatter is valid YAML beyond these keys, or whether a record's `type` is known. If the index refuses a file for one of those reasons, that is a gap: extend the check, and do not rename the file around it.

**Cost of a violation.** `knowledge_write` returns `write_not_confirmed` for every seat until the file is fixed.

Owner: Nyx Copperwing <ARCHITECT>.
