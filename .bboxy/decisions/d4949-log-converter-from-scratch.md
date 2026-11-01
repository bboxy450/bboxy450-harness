---
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T22:15:38.437Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6CJXE2PA8CT81E64DWVPTMY
number: 4949
provenance:
  digest: sha256:a53a078dde67a6864cc8851f7e02467c086130b2f6aaab1c9b12e4aab24b5b2c
  frontmatter_digest: sha256:c6f5a6e03a4b7567061f34905a1a03837fb2c62a7c5d7d1c304abb2e50d79544
  log: .bboxy/log/2026-11-01.jsonl#32115
  sig: 7nrS8rihFgsGDGMlURpuQBS5egGLqf695/W6o3TjiZPtOqeHKDPgC+q72nVhGlm5lhp5MnoZy06Dw4maVgUqCg==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
title: bboxy450-log-converter is built from scratch; formal models via Rust TLC and Java TLC parity
type: decision
updated: 2026-11-01T22:15:38.437Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
---
**Date.** 2026-11-01. **Scope.** The new crate bboxy450-log-converter: Claude, Codex and Bboxy logs parse into an IR stored in a streamable append-only binary format, then lower back into any of the three, with cross-harness session switching in bboxy450-term.

**Operator instruction (verbatim):** "dont use ctx or casr crates, do our shit from scratch, be inspired by it, use TLA+ where it makes sense for harnedding, use our rust implementation of TLC and canonical java version and keep closing gaps in our rust TLC ariant".

**Decision.**
1. No dependency on, or copying from, ctxrs/ctx, casr, Harbor, session-bridge or any similar project. Ideas only, credited in ATTRIBUTION.md as "Inspired by" with their URLs: ctx (fidelity enum, lineage, raw-record retention), casr (provider read/write/resume shape, round-trip fidelity check), Harbor ATIF v1.8 (step and tool-call shape; we may offer ATIF export as an interop target, written by us), claude-code-log (Claude entry taxonomy). session-bridge has no license: do not even read its code further.
2. TLA+ for the stateful contracts: the append writer and the seek-table rewrite (torn-write recovery), the watcher (tail offset, rotation, truncation, restart from cursor, at-least-once versus exactly-once delivery to the sink), and the session-switch handoff in bboxy450-term (old seat quiesce, convert, new seat resume, A→B→A). Each model has a broken control. Pure parsing and lowering get golden and round-trip tests, not models.
3. Every model is checked by bboxy450-tlc, the native Rust checker, with parity against the pinned Java TLC (full state and edge sets plus verdicts). Any construct the Rust checker can't yet handle becomes a tracked gap and a task to close it in bboxy450-tlc. We never skip the model or keep it Java-only silently.

**Reason.** Operator ownership and policy: formal verification is mandatory (d365), Rust TLC should converge on Java TLC, and credit stays attached.