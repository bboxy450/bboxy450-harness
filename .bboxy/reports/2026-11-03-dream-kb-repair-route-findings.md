---
type: report
title: 'KB number collision repair investigation: current import cannot preserve canonical identities'
task: '[[01M6J9X5943VYFN7QH5CT7WQSA]]'
commits:
- bb57401880c341a2078e551780c1861dbaacd50d
verify:
  command: git diff --check -- .bboxy/decisions .bboxy/reports
  exit_code: 0
author: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
created: 2026-11-04T03:40:57.184Z
id: 01M6JAAH12HXTB42HZ3YV5F5K6
updated: 2026-11-04T03:40:57.184Z
number: 20261103136281
created_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
updated_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
authorship:
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  kind: seat
  public_key: 6HRMC38XoZOA5TUGuPnerzKIZabBnbbBZSCqIQt225c=
  process_generation: '1'
provenance:
  digest: sha256:48df66b5d954925d531df9acffdd5af4ae5e1deba3eb8e610dff671ec2e2939c
  frontmatter_digest: sha256:99c32690675ba820b303d440333b50d67d5838881eb5f33daa9351fc7a0d73fd
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  sig: PG29zDBDSzD740/s7awNsfr4wCmpLMGiSyIlDgwiqgSLvH8XkGU7TQprqHRhJX0UMYVQgGFBIdWEKlfK9RtjBA==
  log: .bboxy/log/2026-11-04.jsonl#42899
---
# Authenticated KB repair route — findings

**Result:** the current authenticated catalogue exposes **no supported repair of the eleven existing canonical number collisions** under the required identity/signature/report/assignment constraints. This is a completed interface investigation, not successful mirror repair. The earlier refinement task [[01M6J7JVTEKEBV9F19DQ2GDEQQ]] remains rejected and its mirror gate was not rerun.

The full source-controlled determination is [[d20261103136278-task-import-cannot-repair-existing-number-collisions]], signed source commit `bb57401880c341a2078e551780c1861dbaacd50d`. It contains the exact schema, file:line matrix, inspected tests and missing-capability requirements.

## Evidence per acceptance line

| Requirement | Evidence and result |
|---|---|
| Inspect CURRENT authenticated tools | orient op `01M6J9Y8CV959X747THM8XTK6T`: seat `01M6J5ZDFRK47SHWQW7FM4PSR4`, Architect/oracle class, generation 1, grant epoch 1, root `0f3e1ffa2cdaed6cf6e164db05f51d146051213d6749b5cd2ba8a3aea584d8cc` revision 0, workspace revision 3. Current catalogue contains import/context/write/reconcile and typed ctl; no renumber-existing/repair operation. Live tool declarations were inspected, not inferred from a peer message. |
| Inspect exact import/context schema and actual reply | task_import requires audit_path, bundle_sha256, expected_root, manifest_path, owner_seat, source_dir; optional op_id is minted by the wrapper. No preserve-ID, existing-record reconciliation map or dry-run field. task_import_context op `01M6J9Y82F0FS0S3KR3QR429H8` returned selected expected_root `/Users/feral/mydev/bboxy450-harness/.bboxy` with the same root/workspace pins. No mutation followed that read. |
| Manifest/audit/root pins and collision handling | term pin `b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8`; `crates/bbt-mux/src/index/migration.rs:24-63,98-225` defines and validates manifest/audit, raw hashes, source identities, unique source numbers, path containment and complete dispositions. A bundle containing both claimants of the same source number is rejected. Source revision is 40-hex syntax, not an independent Git ancestry proof. `mcp/work_items.rs:827-848` pins selected root and bundle digest. |
| Determine actual effect rather than trusting its name | `migration.rs:323-345` explicitly generates fresh IDs and new numbers; `:397-438` archives exact raw bytes then removes original active identity, assignment, report and provenance fields from projections; `:439-480` applies historical statuses/custodian metadata; `:486-498` publishes under imports/digest/new-id.md. Existing canonical collision claimants are left intact. Import is unsuitable for this repair even though raw history is preserved. |
| Role/root authority and atomicity | `logic/task_import.rs:149-239,277-301` checks Oracle caller/open Oracle custodian, same swarm/selected root, write-ready binding, root inode, provider/workspace revision and current signing/grant/generation custody; per-root admission is nonblocking. `index/migration.rs:367-394,508-598` stages, signs and atomically publishes one import directory; `:601-705` validates admitted receipt and raw/projection identities. This is new-namespace atomicity, not existing-record repair. |
| Inspect ordinary knowledge writer constraints | `index.rs:848-880` refuses conflicted updates and makes an existing number immutable; omission retains it. `mcp/knowledge.rs:215-228` fixes create IDs to op_id and update IDs to ref. `knowledge/bboxy.rs:891-905` checks number uniqueness after ID/path resolution. `knowledge/effects.rs:199-241` preserves report append-only/task authority. `mcp/seats.rs:127-149` makes ctl an allowlisted same-actor alias, not private Index access. |
| Inspect real tests | `index/migration.rs:893-1002` contains interrupted staging, replay, changed-input/dedupe and admitted-receipt controls. `tests/mcp.rs:3696-3945` checks role/root/hash restrictions, concurrent ordinary writes, same-operation/restart replay, no wakes and byte-identical pre-existing records. `tests/storage_bus_index.rs:435-506,569-616` refuses renumber/conflicted rewrites and allocates unrelated records above collisions. These tests were read, **not executed**. |
| Record original operation custody without retrying | operation_status op `01M6JA3373T2R5AB4F4SCNZQAV` queried original write `01M6J7RY9NMDQQ1FQTB7KFRMMQ`: tool=knowledge_write, ledger=retained, status=rejected, terminal=true, retry_allowed=false. Original exact refusal: create id must equal op_id; update id cannot change. No effect replay. Earlier unknown IDs `01M6J7WK3HTJAR5QXJCEW9W85Z` and `01M6J7WK3JKV91V73HC915XRER` were failed knowledge_fetch reads, not writes. No other actor's original unknown-write ID was supplied or probed. |
| Precise absence and missing host capability | Decision line 80 describes a root-bound existing-identity number-conflict reconciliation capability with complete ID/path/digest/number collision pins, atomic deterministic mapping, signing and current authority checks, unchanged lifecycle/assignment fields, immutable original signed/report bytes, receipt and read-only operation recovery. A signed mapping/alias overlay or explicit versioned migration semantics is needed; current import has neither. No new callable API is asserted. |
| Stop within scope and preserve rejected gate | Investigation stopped at that determination. Only the new decision and this report were written through knowledge tools. No importer mutation, original-record write, shell renumber, provider/root switch, foreign write, implementation, recruitment, retired tooling, operator polling or repeated mirror-gate run occurred. |

Source references abbreviated to a basename in the table are all under `crates/bbt-mux/src` at the pinned commit; the decision gives full paths.

## Commands and real results

- `date -u '+%Y-%m-%dT%H:%M:%SZ'` — exit 0, start 2026-11-04T03:33:52Z; report checkpoint 03:39:28Z.
- `git -C /Users/feral/mydev/bboxy450-term rev-parse origin/main` — exit 0, `b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8`.
- Pinned source reads with `git -C /Users/feral/mydev/bboxy450-term show b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8:<path> | nl -ba | sed -n '<ranges>p'` — all returned exit 0 with source excerpts. The inspected paths were `crates/bbt-mux/src/{mcp/work_items.rs,index/migration.rs,logic/task_import.rs,index.rs,knowledge/effects.rs,knowledge/bboxy.rs,mcp/knowledge.rs,mcp/seats.rs}` and `crates/bbt-mux/tests/{mcp.rs,storage_bus_index.rs}`. Decisive actual ranges included migration 1–265,263–540,540–820,890–1020; logic import 1–350; index 755–900; work_items 440–535,780–858; writer effects 95–245; MCP tests 3696–3945 and transport 3535–3660; storage tests 430–505,569–621. Search/read output was inspected, not an agent's summary.
- `git diff --check -- .bboxy/decisions .bboxy/reports` — **exit 0**, no output.
- `git verify-commit bb57401880c341a2078e551780c1861dbaacd50d` — **exit 0**, Good signature from Boxy (bboxy450), approved key `9B07E849480C68328CA20684844ECC0587DE7AB0`.
- `git show --check --format=oneline bb57401880c341a2078e551780c1861dbaacd50d` — **exit 0**, no whitespace errors in the committed decision.
- Current skill discovery returned the same bboxy450 and knowledge-base digests as already read, with provider_error=null; no plugin/workspace change or retired tool activation.

The authenticated read replies above are successful data replies, not shell exit statuses. No importer or product/formal test was executed. The task's exit-0 document check establishes this investigation's gate only; it does not prove an import, repair, mirror application or implementation acceptance.

## Custody, limits and owner handoff

Source commit `bb57401880c341a2078e551780c1861dbaacd50d` contains only `.bboxy/decisions/d20261103136278-task-import-cannot-repair-existing-number-collisions.md` and its Report trailer. This report uses the task's unique ID wikilink and full source SHA. Existing dirty reports and all original specifications/collision claimants were preserved.

Runtime catalogue/root context and source definitions agree on the inspected interface. Exact running-daemon build-to-source attestation was not provided, and no mutating conformance test was authorized; this finding is scoped to the authenticated catalogue and pinned implementation, not hypothetical operator-only or future capabilities.

Nyx owns any mutation or routing to term's bbt-mux Index/knowledge/provenance/work-item owner. The handoff asks for the missing **existing-identity** repair capability, not another import. Unknown effects must stay bound to the original caller/root/operation and be queried without replay. Original refinement #5984 remains rejected until its unchanged mirror verification actually exits 0.
