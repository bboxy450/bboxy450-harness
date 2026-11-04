---
type: report
title: Oracle verification of Dream WP3 candidate
task: '[[01M6J8TTP3DJMYK0V9F336THNC]]'
commits:
- 90e9c0bbeda6c3b8ebdcbcbd4dfda2b32a356726
- 7c4f1108cfd56da0bd28cb0761ec965e2fe064df
verify:
  command: 'cargo test dream:: && cargo test --test dream_replay --features acp-agent && make test-formal && make test-formal-parity && make check-formal-evidence'
  exit_code: 0
author: seat:01M6J5YFATHYCK148JT0C4XGZ1
created: 2026-11-04T07:40:07.595Z
id: 01M6JR0FAQ5VDJQDDJATYQK8SS
updated: 2026-11-04T07:40:07.595Z
number: 20261103136354
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:5d6fca4b8b3eb82611666a6f4011af5e8b80d90f9cff9f8cd824e11afafe8429
  frontmatter_digest: sha256:9aeaeb82b50b7ec9f276b0d0f82c0d43c4f4ac8f33421fe0656769e978a70169
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: rvnfALGuOj6Wdew66Y9op0m15kmc8dYrOK/9bMZRrw++obJ4J/qWxbHQqX+TcBxr4lLakyHmr+y6wSamIVQpDA==
  log: .bboxy/log/2026-11-04.jsonl#43678
---
WP3 is independently checked for downstream candidate development, not accepted for canonical integration. Source commit 90e9c0bbeda6c3b8ebdcbcbd4dfda2b32a356726 and report HEAD 7c4f1108cfd56da0bd28cb0761ec965e2fe064df both verify with approved signed Boxy key 9B07E849480C68328CA20684844ECC0587DE7AB0; git diff --check against prepared 92521aa93ebcb8c20daab84fa92008dbd298ca1f exits 0. All changed paths are within the amended task.

Inspected the actual D0/task-join/fingerprint sources, native materializer and shared spill/store changes, close hooks, model builders/nonvacuity, key process tests and full report. src/dream/d0.rs:69/122 holds a nonblocking per-fingerprint lease over read/union/fsynced publication, preserves the claim inode and derives count from unique complete pins. :195 hashes retained envelope bytes. src/dream/join.rs:140 resolves origin/main once before literal path enumeration and pinned reads. Engine/ACP/watch changes only enqueue detached D0 alongside the scanner. The measured-poll guard rejects the actual 2ms inline control; CPU-heavy work goes through the existing bounded blocking helper. These observations do not prove arbitrary syscall latency.

Native source capture and projection at src/import/mod.rs:1135–1293 apply canonical sanitized/redacted snapshots, versioned full namespaces, domain-separated deterministic IDs and explicit zero unknown clocks. src/store.rs:114 is an opt-in initial timestamp/extension seam; normal callers preserve behavior. src/store/fork.rs:73 changes only existing no-replace helper visibility. src/import/mod.rs:1306/1409/1492 checks authored closure and blobs, privately stages, publishes blobs before the no-replace complete log, then validates exact winner bytes before returning a resolving path. The shared spill refactor preserves redaction-before-cut order. Existing IR is opened read-only. Source clocks/EOF remain observations, not proof of native process completion.

Independently reran the task's exact verify on report HEAD:
cargo test dream:: && cargo test --test dream_replay --features acp-agent && make test-formal && make test-formal-parity && make check-formal-evidence
Exit 0. Five dream units; thirteen replay tests plus private child fixtures; 135 native/Java full reachable-state, labeled-transition and expected-verdict comparisons; 44 contracts. Retained log /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/nyx-wp3-review-408pvvrc/verify.log SHA256 18515788e6c7a28158e9a0e07015ab1111a9db386dbad9e5444ffa723e89653f, with result.json beside it. Exact receipt SHA256 afc8bf0e1242ab7e73d29e54f45a4f96e31bd244061799233b0992581b4e5634 binds source7c4f110, dirty=false. New finite D0 and native-custody controls are reachable and fail with their expected safety/liveness classes; existing models are retained.

Additional independent command cargo test --test import_native --test ir_edge --test protocol_golden --test ir_v1_corpus --features acp-agent exits 0: 15+8+11+8=42 tests. The current-source exact task rerun exercises native independent-root bytes, repeat/rebuild/races, exact pin/blob resolution, conflict refusal and private process crash witnesses. A disposable CLI IR fixture using the actual repository cwd exercised catalogue loading in 2.078s, exit0, no unavailable-catalogue notice and max poll CPU796291ns; retained /tmp/nyx-d0-catalogue-zz754kpt/result.json. That no-turn fixture does not claim a trusted anchor or completed task join.

Read the actual worker canonical log/result and 18-command run receipt: preceding seventeen rows exit0, then kb-records exit1 for unchanged duplicate5974; make exits2. /tmp/dream-wp3-make-test.log SHA256 651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3. Also inspected the actual GUI/packaging/native smoke log and command receipts, including hidden Metal window/frame and exit/reap PASS; SHA256 e45f7559f435c4cbeb0820ec3f3e7f0d81adf1456c576265fe41b5d59ffe7dcd. GUI/full canonical gates were not independently rerun in this review. No unchanged red loop or waiver.

Prepared WP4 /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp4, clean comparison HEAD 7d3e22cbc4442cee8ea8bdef6e23f5225708680c. It retains exact WP3 HEAD and dependency ancestry plus verified signed document-only cherry-picks through origin/main155044619e5c563df69665be3a12337af2745315. src/tests/Cargo/verification preparation diff is empty. WP4's signed task amends only candidate-start prerequisite, exact workdir/base, validator/mapping scope and fresh final formal verification. Neither dependency acceptance nor implementation publication follows from this preparation.

WP3 source/index are clean; five generated run evidence files remain retained dirty. Finite opaque-pin models do not prove hashes, redaction, signatures, arbitrary filesystem races or timing; real tests substantiate the listed cases only. WP4 must independently validate provenance signatures/root membership and all evidence/conflicts: D0 record presence is not trust. Canonical duplicate-number repair and the original IR mirror remain unresolved. Final acceptance, term submodule update and swarm shutdown remain pending.
