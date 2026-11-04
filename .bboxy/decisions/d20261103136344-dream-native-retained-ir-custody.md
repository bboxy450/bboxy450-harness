---
type: decision
title: Native dream replay retains deterministic imported IR custody
author: seat:01M6J5YFATHYCK148JT0C4XGZ1
id: 01M6JJXVSMZQ5ABQG71V9G91YG
updated: 2026-11-04T06:11:19.203Z
created: 2026-11-04T06:11:19.203Z
number: 20261103136344
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:4a256fd788df81a664e17fddb6f8e14ffc6e4b7624c8e46c1fda6aa433ca75bd
  frontmatter_digest: sha256:0cbe6cb7fa2a5a3c9e6fc6ee5838ccaa6207eddb1529d41d44e498f8048b6d89
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: +q9Pr467kWaas9sSxR+vwTgSrtKZ8k5YsPZW+JP/r/LYPoOixymNpblhJQhfNqBlYtmwLlguN06PY9dDAJ65Cw==
  log: .bboxy/log/2026-11-04.jsonl#43577
---
This ruling resolves the contradiction raised by WP3 message 01M6JJMTGMQ8HV2ARV4SYZ4V6X. dream-log §3's unconditional “replay writes no session” prevented §4.1 native replay from producing resolving §2 Event pins. src/import/mod.rs:905–1004 currently writes an ordinary imported session and mints fresh session/turn/item identities; transient Projector values have no retained envelope custody. Native support is required, so the no-write statement is narrowed explicitly rather than reporting native input as supported.

Contract owners for this implementation boundary are the delegating Oracle (Nyx) and WP3's assigned Executor. This is an Oracle scope/integration ruling; it does not attribute an approval to the closed Architect or claim the original specs were complete.

1. Replay of existing IR remains read-only: never append, recover, reserialize, renumber or replace that source. Native replay may create a **closed ordinary imported IR session** in the existing selected --sessions root (default ~/.bboxy450-harness/sessions), using the shared native Projector, redaction and spill/blob machinery. It creates no dream/model session, no second IR store, and no new protocol type. --rebuild affects derived D0 data only and retains import custody.

2. Native materialization has a versioned deterministic mode, separate from normal import/watch/reproject defaults. Its namespace includes native format, current PARSER_REVISION, materializer revision 1 and a full SHA-256 of the canonical sanitized/redacted native-record sequence used for projection. Canonical JSON sorts object keys, preserves array order and string bytes, and excludes source filesystem path, mtime and current wall clock. Apply the existing native known-field credential sanitizer and registered Set before deriving this digest; retain unknown fields through the same shared mapping. Reject malformed/incomplete JSON records in this mode rather than silently constructing pins from skipped bytes. Read one bounded, stable snapshot off the runtime/UI worker; fail/skip if the source changes while captured. Never edit or retain an unredacted source copy.

3. The same namespace and deterministic per-kind ordinal allocation determine session, turn, item, generated fallback call and operation identities. Identity domains are distinct. Use a SHA-256 domain-separated derivation with a documented fixed encoding; truncate only to construct the existing ULID-shaped types. A native-provided identifier stays native metadata; it is not overwritten as an internal ID. Never use random IDs, now(), path or process birth for durable materialized bytes. Native clocks are preserved; absent timestamps use the documented zero/unknown sentinel in this opt-in mode, not an invented historical clock. Header/profile/close/envelope timestamps, epochs, causes, ordering, generated IDs and spill references must all be deterministic for this namespace. Existing ordinary import behavior and source/parser provenance are preserved. The full digest and materializer revision may use existing envelope extension metadata; this adds no Source/IR schema field.

4. Publish through the ordinary sessions layout: Event{session,seq,digest} resolves to the exact retained envelope in <sessions>/<session>.jsonl, with the normal <sessions>/<session>/blobs content-addressed references. Hash the stored envelope bytes excluding only trailing LF, as §2 specifies. Build privately with existing writer/spill behavior, validate transcript/authored rules and every referenced blob, fsync data, then expose the complete log with atomic no-replace publication after referenced blobs are available. Do not return pins or merge the D0 episode before confirmed complete custody. Any same-namespace materializer races without waiting: busy means skip/replay later; a completed winner is reused only after exact identity/bytes/blob validation. A conflicting existing log/blob is an error, never overwritten or recovered as a mutable session. Existing store helpers can be reused; new code must not bypass their ownership/redaction rules.

5. A crash before publication yields no resolving episode; abandoned private staging is not enumerated as session evidence. A crash after complete publication permits reuse on replay. Repeated replay, --rebuild and concurrent replay of the same snapshot must return the same episode origins and line digests and must not increment deduplicated counts or produce another import. Identical input/redaction/mapping in independent private roots must generate identical durable IR and referenced blob bytes. Changing projected content, mapping or redaction output creates a distinct namespace and retains old custody. Neither snapshot closure nor this deterministic ID makes native/session text trusted evidence or proves the original native process ended.

6. Required witnesses: Claude and Codex native inputs; IR replay remains byte-identical/no new sessions; repeated replay/rebuild and two-process same-input races; independent-root byte equality; exact Event pin resolution and blob hashes; oversized redacted/unknown native content; malformed/changing source refusal; publication interruption/crash and subsequent replay; existing-identity/digest conflict refusal without clobber. Extend the authorized D0 finite model for staged/complete publication and competing native materializers as needed; preserve lost-union/blocking controls and add a reachable premature-pin/publication control. Actual fsync/rename/kernel bytes remain implementation assumptions with process witnesses.

WP3 may edit src/import/mod.rs for the opt-in deterministic materializer and ID allocation; src/import/spill.rs only if required to reuse the existing spill path; src/store.rs only for the minimal opt-in initial-envelope timestamp/extension seam, preserving existing Store::open callers and recovery semantics; src/store/fork.rs only to expose/reuse the existing no-replace helper, without weakening it. Existing WP3 D0/CLI/tests and formal paths remain authorized. No new dependency, protocol/schema/lock change, normal-import behavior change or extra store is authorized. The exact amended task carries these paths and verification. Final acceptance still requires canonical gates green; duplicate KB repair and companion IR mirror remain outstanding.
