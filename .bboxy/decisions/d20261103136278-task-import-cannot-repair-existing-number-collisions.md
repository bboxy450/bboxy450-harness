---
type: decision
title: Current task_import is lossless archival import, not canonical KB number-conflict repair
status: accepted
links:
- to: '[[01M6J9X5943VYFN7QH5CT7WQSA]]'
  rel: for
- to: '[[01M6J7JVTEKEBV9F19DQ2GDEQQ]]'
  rel: constrains
- to: '[[d20261103136206-dream-contract-refinement]]'
  rel: builds-on
id: 01M6JA76NF1P600FCE6XDQEHWK
updated: 2026-11-04T03:39:08.297Z
created: 2026-11-04T03:39:08.297Z
number: 20261103136278
created_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
updated_by: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
authorship:
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  kind: seat
  public_key: 6HRMC38XoZOA5TUGuPnerzKIZabBnbbBZSCqIQt225c=
  process_generation: '1'
provenance:
  digest: sha256:243157e98c60346989da3f0a42a996d8c78dfc78300acbbb4cc185b79197f017
  frontmatter_digest: sha256:e57c0dd44dcdf94808fff9a5a3657874029c14995a8562096ba2808a4c6f300c
  signer: seat:01M6J5ZDFRK47SHWQW7FM4PSR4
  sig: CztiVVJ8Vf0t1h1LYvixlOWDLezYpO4G/wuoVttI5CJtTyizXNSgYY9RY0pFbitRXvXfZKXh/PJy6P+xGoIGAA==
  log: .bboxy/log/2026-11-04.jsonl#42869
---
# Task import cannot repair existing KB number collisions

**Date:** 2026-11-03. **Author:** Lumen Foxglove, Architect. **Finding:** no supported route in this seat's current authenticated catalogue repairs the eleven canonical number collisions while preserving their record identities, signed history, append-only reports and task assignments.

This is the bounded result of [[01M6J9X5943VYFN7QH5CT7WQSA]], not a mutation authorization. [[01M6J7JVTEKEBV9F19DQ2GDEQQ]] remains rejected; its original mirror verification is unchanged.

## Scope and actual tool evidence

At `orient` operation `01M6J9Y8CV959X747THM8XTK6T`, the authenticated seat is `01M6J5ZDFRK47SHWQW7FM4PSR4`, generation 1, Architect/oracle class, grant epoch 1. The selected root is `0f3e1ffa2cdaed6cf6e164db05f51d146051213d6749b5cd2ba8a3aea584d8cc`, revision 0; swarm workspace revision 3.

The catalogue exposes task_import, task_import_context, knowledge_write, task_reconcile and the typed ctl alias. It exposes no repair/renumber-existing operation. The live `task_import` schema requires `audit_path, bundle_sha256, expected_root, manifest_path, owner_seat, source_dir` and permits an optional tool-minted/retry op_id. It has no existing-record identity/digest reconciliation map, preserve-ID mode or dry-run flag. The live `task_import_context({})` reply (op `01M6J9Y82F0FS0S3KR3QR429H8`) returned exactly `/Users/feral/mydev/bboxy450-harness/.bboxy` plus those root/workspace pins. That read establishes selection, not permission to mutate.

No task_import, reconcile, original-record write, provider/workspace switch, test import, shell repair or retired tooling was invoked.

## Pinned implementation: why importing is not repair

All paths below are in term commit **b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8**, read with git show, not the stale term working tree.

| Boundary | Exact evidence and consequence |
|---|---|
| Advertised role/schema | `crates/bbt-mux/src/mcp/work_items.rs:513-533` — import/context are Oracle-class catalogue entries; schema forbids undeclared arguments. `:781-839` requires Oracle class, canonical expected_root equal to selected native root, and Oracle owner. `:841-848` reloads and checks the bundle digest. |
| Manifest/audit pins | `crates/bbt-mux/src/index/migration.rs:24-63` — manifest has source_revision, seed/dependency/all counts, rows and links; audit has indexed dispositions. `:98-123` validates counts, ≤512 records and the 40-hex source_revision syntax (not a git ancestry/signature proof); `:139-187` requires unique source IDs, source numbers, paths and dedupe keys and matching raw source hash/type/identity. Thus a single manifest containing both claimants of an existing duplicated source number is rejected. `:154-175` checks relative paths/no symlinks and 32 MiB total; `:193-225` requires complete allowed seed dispositions and link mapping. |
| Fresh identities, not canonical repair | `crates/bbt-mux/src/index/migration.rs:277-283` chooses `imports/<bundle digest>` and verifies an existing receipt for replay. `:285-320` rejects a prior imported source under a different bundle. `:323-345` allocates above the current maximum and explicitly generates fresh ULIDs excluding source IDs and occupied IDs. `:486-498` writes only new paths under that imports namespace. No existing claimant is renumbered or removed. |
| Signatures and assignments change in the projection | `crates/bbt-mux/src/index/migration.rs:397-438` preserves exact bytes in raw/<new-id>.source, then removes id, number, assignee, delegated_by, report, verify_result, provenance/authorship and authorship timestamps from the projection and installs the fresh id/number/import_source. `:439-480` applies audited historical task status and custodian delegation or converts dependency tasks to notes. `:487-497` authors new signed projection records. Raw archival preservation is not preservation of the canonical active identity/signature/assignment. |
| Allocation/collision semantics | `crates/bbt-mux/src/index.rs:781-827` retains every current number claimant. `crates/bbt-mux/src/index/migration.rs:323-345` allocates above maximum, rather than repairing a collision. `:545-580` rechecks the allocation, IDs and source-dedupe claims; a concurrent allocation returns import_allocation_stale. This permits unrelated import publication around collisions but leaves the original conflicts intact. |
| Atomicity and receipt | `crates/bbt-mux/src/index/migration.rs:367-394` stages outside the active root; `:508-540` records hashes, admitted activity intent and signature; `:542-598` fsyncs and publishes one imports directory under a short root lock after validation. `:601-705` verifies root/digest/count, signature bound to host-admitted intent, raw hashes, imported IDs and original manifest/audit. This atomicity covers a new namespace, not a transaction over existing canonical records. |
| Root and current authority | `crates/bbt-mux/src/logic/task_import.rs:149-179` requires write-ready captured root, open Oracle custodian, same seat swarm and same selected root. `:180-239` captures signing custody, provider revision, root inode and nonblocking per-root capacity. `:24-73,277-301` recheck caller/custodian grants/signers/issuer, root inode, provider revision and workspace/seat generation before publication. Full/absent admission or changed authority fails closed; no global import gate is granted. |
| Generic writer cannot substitute | `crates/bbt-mux/src/index.rs:848-880` rejects an existing conflicting number and makes an existing number immutable; omitting number retains it. `crates/bbt-mux/src/mcp/knowledge.rs:215-228` fixes create IDs to the operation ID and update IDs to the original reference. `crates/bbt-mux/src/knowledge/bboxy.rs:891-905` applies fresh number uniqueness even to an otherwise resolved ID/path. `crates/bbt-mux/src/knowledge/effects.rs:199-241` keeps reports append-only and task-authorized. |
| ctl is not another privilege route | `crates/bbt-mux/src/mcp/seats.rs:127-149` canonicalizes only its explicit command allowlist, under the same actor and operation ID. It cannot call arbitrary private Index functions or convert a seat into the operator. |

## What the inspected tests establish in source

Tests were **read, not run**; no runtime pass is inferred.

- `crates/bbt-mux/src/index/migration.rs:893-953`: interrupted staging remains inactive; replay publishes once; changed source/audit/dedupe cannot create another imported identity.
- `crates/bbt-mux/src/index/migration.rs:955-1002`: a self-signed replacement receipt or re-signed mapping fails admitted-intent verification.
- `crates/bbt-mux/tests/mcp.rs:3696-3834`: root context and worker restrictions, wrong root, worker custodian and wrong digest are controlled.
- `crates/bbt-mux/tests/mcp.rs:3840-3907`: ordinary writes progress during staging; changed arguments with the same op_id are refused; replay/restart preserve the import publication.
- `crates/bbt-mux/tests/mcp.rs:3934-3945`: published hashes are unchanged, no historical inbox wakes, and the existing task171/unrelated files remain byte-identical. Preservation of existing files is evidence that import does not repair them.
- `crates/bbt-mux/tests/storage_bus_index.rs:435-506`: attempted renumber fails and leaves source bytes unchanged; omitted number is retained.
- `crates/bbt-mux/tests/storage_bus_index.rs:569-616`: different record types sharing a number remain conflicted; a new unrelated record allocates above them; rewriting a conflicted record is refused.
- `crates/bbt-mux/tests/mcp.rs:6449-6463`: a collided number returns number_conflict while unrelated fetch remains available.
- `crates/bbt-mux/tests/mcp.rs:3537-3653`: lost-completed-reply control returns unknown while a read-only original-operation lookup retains imported status; the control never retries the unknown effect.

## Original operation custody

The earlier attempted original-ID create is **not** an unknown write: `operation_status` for `01M6J7RY9NMDQQ1FQTB7KFRMMQ` returned ledger=retained, status=rejected, terminal=true, retry_allowed=false, tool=knowledge_write, with the exact ID-immutability rejection. Lookup op: `01M6JA3373T2R5AB4F4SCNZQAV`. No effect was retried.

The two earlier transport-unknown IDs `01M6J7WK3HTJAR5QXJCEW9W85Z` and `01M6J7WK3JKV91V73HC915XRER` were knowledge_fetch **reads**, not unknown writes. No original unknown-write ID for another actor was supplied; this investigation neither invents one nor probes foreign operation custody. An unknown mutation must be queried by its original owner/root/op ID, never converted into permission to import again.

## Missing host capability and owner handoff

**Missing capability:** authenticated, root-bound, existing-identity number-conflict reconciliation, distinct from import. It must pin the complete observed collision groups by canonical record ID/path/current digest/number and root/provider/workspace revision, preserve IDs and task lifecycle/assignment fields, leave signed originals and append-only report bytes auditable, publish one deterministic mapping/result atomically, recheck custody and concurrent changes, and provide a signed receipt plus read-only original-operation recovery without historical wakes.

Because the number participates in signed frontmatter, a claim to preserve old signature bytes cannot simply edit that field and keep the old signature. The host must explicitly define an authoritative signed mapping/alias overlay (with consistent resolver, graph, allocator and writer semantics), or a deliberate versioned migration retaining prior signed versions and an explicit rule for append-only reports. No such API or semantics is exposed here. This is a capability requirement for the host owner, not an implemented design or authorization to invent a tool call.

**Owner:** Nyx routes the finding to term's bbt-mux Index/knowledge/provenance/work-item owner. Any future repair must prove duplicate-group race handling, A→B→A/root/provider/grant fencing, existing ID/assignment/report/signature preservation, crash atomicity and unknown-outcome recovery with formal broken controls and real API/process witnesses. This task stops at this determination. It does not start implementation, create a replacement KB, renumber anything, or waive refinement #5984.
