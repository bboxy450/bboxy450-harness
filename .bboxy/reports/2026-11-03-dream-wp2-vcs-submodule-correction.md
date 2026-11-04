---
type: report
title: 'Dream WP2: correct false-clean submodule observations'
task: '[[tasks/2026-11-03-dream-wp2-vcs-r2]]'
supersedes: '[[reports/2026-11-03-dream-wp2-vcs-r2]]'
commits:
- da78cbda91b5de454c38152de8c44b5d587f2086
- 8b4be671e2955b59edd53ff55f8e1645ec4f146f
verify:
  command: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
  exit_code: 0
author: seat:01M6J7N5V103SG1KS0THEXB9R1
created: 2026-11-04T05:24:53.199Z
id: 01M6JG8TSAJ77BYMHX8T69TSCJ
updated: 2026-11-04T05:24:53.199Z
number: 20261103136331
created_by: seat:01M6J7N5V103SG1KS0THEXB9R1
updated_by: seat:01M6J7N5V103SG1KS0THEXB9R1
authorship:
  signer: seat:01M6J7N5V103SG1KS0THEXB9R1
  kind: seat
  public_key: v6Dhh0hEt+bB6M2k/4ibRUpZFwCZYd7J3USyUIR1XFs=
  process_generation: '1'
provenance:
  digest: sha256:96c98bbeca4457f0ec94278d9a58d557ff62712fc090b8d1f02d4cc38bec3b8e
  frontmatter_digest: sha256:c77534db3bb6624ebe1e31e0f58281b3c36c6e7ce4d902a0e0decdea2f83209f
  signer: seat:01M6J7N5V103SG1KS0THEXB9R1
  sig: iEfPUBPoDz958U7cDhKxp5UqEhM8QOQqhNwVIzJD9poG586YmFS9/Nbl8wLTcw9I2yoAF7IzgR6p/XR6PWlEBw==
  log: .bboxy/log/2026-11-04.jsonl#43427
---
# Dream WP2: submodule dirty-state correction

The Oracle's rejection was correct: the previous --ignore-submodules=all query could store dirty=false for a changed tracked submodule. The corrected query uses --ignore-submodules=none and overrides both repository and .gitmodules ignore policies. A real signed Git/harness regression fails against the old implementation and passes against the correction. Primary verification now passes 25 tests. Canonical make test still exits 2 at the separate, unchanged duplicate KB number 5974.

This report supersedes [[reports/2026-11-03-dream-wp2-vcs-r2]] for task [[tasks/2026-11-03-dream-wp2-vcs-r2]]. Rejection: 03DER5AWHM471TBMZQ4FJ49GAM. Author: seat:01M6J7N5V103SG1KS0THEXB9R1. Worktree: /Users/feral/mydev/bboxy450-harness-dream-wp2, branch dream/wp2. Prepared comparison base remains a279e0fdc5866068db567561e24ffa2bc0897d6b. Initial signed implementation da78cbda91b5de454c38152de8c44b5d587f2086 is retained; signed correction 8b4be671e2955b59edd53ff55f8e1645ec4f146f changes only src/vcs.rs and tests/dream_vcs.rs (139 insertions, 3 deletions). git verify-commit exits 0, Boxy <bboxy450@gmail.com>, key 9B07E849480C68328CA20684844ECC0587DE7AB0. The correction commit carries this report's Report trailer. The original report remains append-only.

## Correction and direct evidence

src/vcs.rs:192 changes the status flag to --ignore-submodules=none. The shared three-second query budget, 16KiB stdout cap, process owner, bounded termination/drain/reap, URL redaction and inherited GIT_* clearing are unchanged. The existing successful-status filter still maps nonzero exit, cancellation/timeout and capped output to absent dirty, never false.

tests/dream_vcs.rs:205 creates a private signed parent and child repository and registers an actual local submodule without network access. Both .gitmodules and repository configuration request ignore=all. It checks a clean baseline, modifies the child's tracked file, then makes a signed child commit so the gitlink differs. At each changed state, default/ignore-all status is empty while explicit ignore-none status contains M child. The actual scripted engine and ACP processes must report dirty=true in emitted and persisted SessionStarted headers; the public capture API must agree. Authored transcript laws run over their output.

Before changing src/vcs.rs, the focused regression was run against the original implementation:

    cargo test --test dream_vcs --features acp-agent changed_submodule_worktree_and_gitlink_are_dirty_in_actual_headers -- --exact
    exit 101
    VcsSubmoduleDirty: changed worktree was reported clean (acp=false)
    left: Some(false), right: Some(true)

Evidence: /tmp/dream-wp2-submodule-red.log. The fixture and exact same assertion pass after the flag change. This is a real API/process counterexample, not a model-only negative control.

tests/dream_vcs.rs:386 adds a successful-metadata/failed-status witness: status exits 72 or fills the output cap, and the real harness retains branch while dirty stays absent. The existing stalled-Git process witness at :403 now lets the preceding four queries succeed and stalls specifically at status. It checks the successful turn retains known branch, reports dirty=None (:470), and reaps the exact recorded Git leader/member PIDs under the existing deadline. Its child lease, timeout fallback and ownership assertions remain. The final primary suite also reruns the engine/ACP Trace2 leak regression and all URL credential assertions; those protections remain green.

## Full WP2 acceptance evidence

- Actual VCS capture/shared normalization: src/vcs.rs:45/150/176 use actual local Git data, normalize supported remote identities and use canonical toplevel realpath only when origin is absent. src/lib.rs:13 remains only the module declaration. src/engine/mod.rs:179 and src/backend/acp_agent.rs:235 remain only session-start metadata calls. Queries stay outside active turns/UI work. Existing/provided headers are preserved by capture_new_session. Real Git tests cover clean/dirty, no origin and no repo; missing/failing Git still permits an unattended turn. Submodule worktree/gitlink changes now have the explicit real-process coverage above.
- Native field availability: src/import/edge.rs:15 maps recorded Claude branch and Codex repository_url/branch/commit_hash only. Unknown or missing fields remain absent; no current checkout fills historical gaps. tests/import_native.rs:872 checks partial/full native cases beside an unrelated dirty checkout, plus malformed/oversized and unknown leaves.
- Redaction/unknown preservation: src/import/edge.rs:45/110 and src/import/mod.rs:223 preserve sanitized native raw/ext paths. tests/import_native.rs:966 checks known remote credentials/query/fragment do not reach persisted typed/raw/ext output, unknown fields survive and native input bytes remain unchanged. The existing Trace2 regression verifies no inherited tracing output for engine or ACP.
- Native provenance: PARSER_REVISION=3 in src/import/mod.rs is shared by one-shot import and recognized-native reprojection; distinct non-native behavior is unchanged. tests/ir_edge.rs:183/227/245 verifies repeated reprojection cannot manufacture a new mapping revision and retains all source/prefix immutability and transcript-law assertions.
- Additive IR, replay and attribution: Cargo manifest/lock, protocol source/schema/lock and all frozen IR fixtures are unchanged against prepared base (git diff --exit-code exits 0). The 19 golden/corpus tests exercise actual check_transcript/check_authored functions, unknown round-tripping/authored rejection and replay/fork/headless byte identity. Every source retains the absolute attribution/license/policy header. No dependency, schema, protocol or model change was introduced by the correction.
- Formal coverage: the authorized src/vcs.rs inventory/implementation-only note is retained. The same existing process helper and ownership/lifecycle contracts are reused; no new queue, owner, retry or lifecycle was introduced. Current corrected-source native/TLC parity covers 130 finite models, 42 contracts and 1 inventoried crate. Receipt verification/evidence/d641a5f66d97be51306a3bb1c7c76faab787f1c45252c5cfcc268b3bfb616aa2/evidence.json has SHA256 d641a5f66d97be51306a3bb1c7c76faab787f1c45252c5cfcc268b3bfb616aa2, source 8b4be671e2955b59edd53ff55f8e1645ec4f146f, dirty=false.

## Commands and outcomes

| Command | Exit | Evidence |
| --- | --- | --- |
| Focused submodule regression before source correction (exact command above) | 101 | /tmp/dream-wp2-submodule-red.log; named false-clean assertion |
| cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent | 0 | /tmp/dream-wp2-submodule-verify.log; 8 VCS + 15 import + 2 CLI tests |
| cargo test --test ir_edge | 0 | /tmp/dream-wp2-submodule-ir-edge.log; 8 tests |
| cargo clippy --offline --locked --test dream_vcs --features acp-agent -- -D warnings | 0 | /tmp/dream-wp2-submodule-clippy.log |
| make test | 2 | /tmp/dream-wp2-submodule-make-test.log; 17 preceding commands exit 0; KB checker exit 1 |
| Canonical make test-formal-native and make test-formal-parity | 0 each | Fresh corrected-source 130-model native/TLC state/edge/verdict parity; receipt above |
| make test-ir && cargo test --test protocol_golden --test ir_v1_corpus | 0 | 64 definitions; controls reject 10/accept 4; 11 corpus + 8 golden tests |
| python3 -I scripts/checks.py kb-records-control; inventory-shape; inventory-shape-control | 0 each | Bad-record control rejected; inventory PASS (1); rejected 2 malformed inventories |
| cargo run --offline --locked -- ir-scan --propose <private-home>/ir-proposals --check-pending tests/fixtures/ir-v1/pending tests/fixtures/ | 0 | /tmp/dream-wp2-submodule-ir-scan.log; private HOME/TMPDIR with explicit existing toolchain homes |
| make test-gui test-app-packaging smoke-app, isolated rerun | 0 | /tmp/dream-wp2-submodule-remaining-isolated.log |
| Initial overlapping GUI/IR invocation | 2 | /tmp/dream-wp2-submodule-remaining.log; feature artifact collision described below |
| git diff --check; git verify-commit 8b4be671e2955b59edd53ff55f8e1645ec4f146f | 0 each | Clean patch; Good signature from Boxy |

The canonical gate ran alone and recorded source 8b4be671, private_home=true and operator_input=false in .bboxy/runs/test-current.json. Its passing rows include both clippy modes, both all-target suites, isolated seat/non-PTY cases, hardening, capability/heartbeat/launch controls, native formal/parity, secrets/controls and headers. Actual protocol laws run inside tests, including persisted harness output; no invented checker target replaces check_transcript.

I initially overlapped the separate GUI gate with the default-feature IR scanner/build. They share target/debug/bboxy450-harness: three GUI CLI assertions failed, and a direct private-home probe of that binary returned “bboxy450-harness gui requires --features standalone”. The default-feature build had replaced the standalone artifact during GUI execution. After those jobs finished, rerunning the complete GUI/packaging/smoke command alone passed. No product source or test assertion was changed to obtain that result. Both failed and corrected logs are retained. GUI/application checks use private homes, scripted/hidden windows and ad-hoc signing; no desktop input, operator login or Keychain prompt.

## Limits, outstanding baseline and handoff state

The inherited finite process models do not prove Git semantics, submodule traversal, arbitrary filesystem/syscall timing, remote URL parsing, bytes or snapshot consistency. The new real-Git and process witnesses cover those specific implementation cases; Git observations across sequential commands are still not an atomic changing-checkout transaction. The shared three-second budget plus inherited bounded cleanup was not increased. Any incomplete status stays unknown. Unsupported local/file origins remain unknown; live-watch/reproject VCS header promotion stays outside this brief. WP9 owns final combined dream implementation mappings.

The independent canonical KB failure remains unchanged:

    kb record number 5974 repeats: .bboxy/decisions/d5974-log-ir-v1-only-add.md, .bboxy/decisions/d5974-memories-are-kb-records.md
    make: *** [test] Error 1

Both decision files are unchanged against prepared base a279e0fd. No repair, filtering, waiver or repeated same-premise canonical loop was performed. Acceptance still requires a green canonical make test after the authorized KB repair; this report does not request acceptance on a red gate.

The implementation and this append-only correction report are committed with signed Boxy commits and Report trailers. Source/index are clean at handoff; five retained generated outputs remain intentionally uncommitted: .bboxy/runs/formal-native.json, gui-gate.json, gui-smoke.png, gui-smoke-scrolled.png and test-current.json. Build artifacts/evidence remain retained in their ignored target/evidence directories. No secrets or raw operator transcripts were committed. All work stayed in WP2; WP1 and its evidence were untouched. No publication, merge or WP3 work occurred.
