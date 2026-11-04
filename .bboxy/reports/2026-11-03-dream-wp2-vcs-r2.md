---
type: report
title: 'Dream WP2 revision 2: bounded VCS capture and native provenance'
task: '[[tasks/2026-11-03-dream-wp2-vcs-r2]]'
commits:
- da78cbda91b5de454c38152de8c44b5d587f2086
verify:
  command: cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent
  exit_code: 0
author: seat:01M6J7N5V103SG1KS0THEXB9R1
created: 2026-11-04T04:58:11.565Z
id: 01M6JEQYVNM8M5G8D7R4122764
updated: 2026-11-04T04:58:11.565Z
number: 20261103136324
created_by: seat:01M6J7N5V103SG1KS0THEXB9R1
updated_by: seat:01M6J7N5V103SG1KS0THEXB9R1
authorship:
  signer: seat:01M6J7N5V103SG1KS0THEXB9R1
  kind: seat
  public_key: v6Dhh0hEt+bB6M2k/4ibRUpZFwCZYd7J3USyUIR1XFs=
  process_generation: '1'
provenance:
  digest: sha256:bc0e8bce2f21a986cfaf16ff6651089baddd5e2d956cafd8cbd687347c0bf1a2
  frontmatter_digest: sha256:02a0d3b7a579dfedb1724ce48d93141a880513a70f801ff07ed9843a5869ebcc
  signer: seat:01M6J7N5V103SG1KS0THEXB9R1
  sig: 3+x/ph2e72dXr9ht032QFrVfwMWz97rb4SIfnna0AE9cA9mQrqzCbAaSX+FsJbMwPFVwb56OCXsZhc6OcHboBQ==
  log: .bboxy/log/2026-11-04.jsonl#43318
---
# Dream WP2 revision 2: bounded VCS observations and native provenance

Primary verification exits 0 (23 tests); the additionally required ir_edge suite exits 0 (8 tests). The unchanged canonical make test exits 2 at duplicate KB number 5974 after all preceding unit/integration, control and formal commands pass. GUI, packaging and application smoke pass unattended. This is a candidate handoff, not acceptance or a gate waiver.

Task: [[tasks/2026-11-03-dream-wp2-vcs-r2]]. Author: seat:01M6J7N5V103SG1KS0THEXB9R1. Writing worktree: /Users/feral/mydev/bboxy450-harness-dream-wp2, branch dream/wp2. Prepared comparison base: a279e0fdc5866068db567561e24ffa2bc0897d6b. Signed implementation: da78cbda91b5de454c38152de8c44b5d587f2086; Boxy <bboxy450@gmail.com>; git verify-commit exits 0 with machine key 9B07E849480C68328CA20684844ECC0587DE7AB0. The implementation commit carries this report's locator. No publication or merge occurred.

The prepared base contains independently reviewed WP1/WP5 candidates, not accepted/main prerequisites. The signed task expressly permits bounded work on that base while retaining every acceptance gate. Governing text is dream-log b4894fb6dd4e9731256760683ae2548a03f72bad and [[d20261103136206-dream-contract-refinement]] (066a84e88d17d9f187d97c8326ae0555c417306e), with dream-log §12 precedence over conflicting-number original mirrors. All commands and edits used the WP2 worktree. WP1 source and its retained evidence were not changed.

## Changes and acceptance evidence

- Actual Git metadata and shared identity: src/vcs.rs:45 normalizes supported HTTP/HTTPS/SSH/Git and SCP origins into host/owner/name without authentication, scheme or trailing .git. src/vcs.rs:150 locates actual Git toplevel and uses its canonical realpath only when origin is absent. src/vcs.rs:176 captures actual branch, commit and clean/dirty result. Failures leave fields absent. tests/dream_vcs.rs:176 proves clean, modified and untracked states from a private real repository; :196 proves no-origin realpath and no-repository absence; :140 covers URL forms, ports, unsupported origins and oversize input.
- Session-start integration only: src/lib.rs:13 adds only the module declaration; src/engine/mod.rs:179 and src/backend/acp_agent.rs:235 add only metadata capture before Store::open. src/vcs.rs:210 preserves provided metadata and existing-log headers without another Git probe. tests/dream_vcs.rs:211 observes actual engine and ACP session headers and persisted credential-free logs. tests/cli_headless.rs:9 covers no-repo scripted execution; :66 proves closed replay leaves bytes unchanged without invoking a private Git marker.
- Bounded unattended subprocesses: src/vcs.rs:15 shares one three-second deadline across five queries; :92 reuses process::capture_stdout with a 16KiB output cap and its existing owned-group timeout/kill/drain/reap cleanup (up to one additional second under that helper's OS assumptions). No shell, network operation, credential helper, hook, fsmonitor or operator prompt is used. Git system/global configuration is disabled; inherited GIT_* variables are removed before safe defaults. Canonicalization runs through the existing permit-backed runtime::blocking helper rather than a runtime worker. Git runs only in async session startup, never the active turn/UI loop. tests/dream_vcs.rs:254 proves missing/failing Git still permits a turn and does not echo private stderr; :274 runs a stalled Git process and TERM-ignoring child, then asserts a successful turn and disappearance of the exact recorded owned PIDs.
- Native availability without current-checkout guesses: src/import/edge.rs:15 promotes Claude branch only and Codex recorded repository_url/branch/commit_hash only. Unknown or absent dirty stays absent. src/import/mod.rs:121 collects native observations; :950 attaches them to one-shot import SessionMeta. tests/import_native.rs:872 runs partial/full native cases beside an unrelated private dirty checkout and proves historical gaps remain absent; unknown Git leaves and malformed/oversized branch data remain in raw projection.
- Credential-safe native raw/ext paths: src/import/edge.rs:45 sanitizes only known Codex repository_url leaves, and :110 applies the same sanitization to extension remainders. src/import/mod.rs:223 retains sanitized raw fallback. tests/import_native.rs:966 proves credentials/query/fragment do not reach typed/raw/ext persisted output, unknown Git fields remain, and the original input file remains unchanged.
- Parser provenance: src/import/mod.rs:16 defines PARSER_REVISION=3 and :961 uses it for one-shot native import. src/import/reproject.rs:70 uses that same constant only for recognized native harnesses; other mappers retain their prior behavior. tests/ir_edge.rs:183 verifies one-shot/reproject/repeated reproject headers without losing any source/prefix immutability assertions. My initial boundary query incorrectly called the base hardcoded Some(2); the Oracle corrected it: the base used previous_revision.saturating_add(1). The authorized fix prevents repeated native reprojection from claiming a new mapping revision.
- Additive IR and byte identity: no Cargo manifest/lock, protocol source/schema/lock, or frozen IR fixture changed relative to prepared base (git diff --exit-code over those paths exits 0). Protocol golden/corpus tests pass, including unknown preservation/refused authorship and replay/fork byte identity. The actual check_transcript/check_authored functions run in tests/protocol_golden.rs:432, tests/ir_v1_corpus.rs:40, tests/import_native.rs:121 and tests/ir_edge.rs:227/:245; no invented shell checker target is substituted.
- Attribution and scope: every source retains the exact project/license/policy header from ATTRIBUTION.md; the new module and tests carry it. Implementation commit changes exactly 12 allowed files (848 insertions, 9 deletions). verification/coverage.toml:79 adds only the required src/vcs.rs inventory entry, and its implementation_only entry adds the authorized implementation-only coverage/limits note. The task's two signed amendments supplied those inventory and native reproject paths.

## Verification results

| Command / witness | Exit | Evidence |
| --- | --- | --- |
| cargo test --test dream_vcs --test import_native --test cli_headless --features acp-agent | 0 | /tmp/dream-wp2-verify-final.log: 6 VCS + 15 import + 2 CLI tests, all passed |
| cargo test --test ir_edge | 0 | /tmp/dream-wp2-ir-edge-final.log: 8 passed |
| make test-ir && cargo test --test protocol_golden --test ir_v1_corpus | 0 | 64 locked definitions; compatibility controls reject 10/accept 4; 11 corpus + 8 golden tests passed |
| cargo check --offline --locked --all-targets --features acp-agent | 0 | /tmp/dream-wp2-check.log |
| make test | 2 | /tmp/dream-wp2-make-test-final.log; source da78cbda; all 17 preceding commands exit 0; KB checker exits 1 |
| make test-formal-native; make test-formal-parity (canonical commands) | 0 each | 130 models / 42 contracts / 1 inventoried crate; full native/TLC state/edge and verdict parity; receipt below |
| make test-gui test-app-packaging smoke-app | 0 | /tmp/dream-wp2-remaining-gates.log; GUI suites/control, 21 visible items/14 kinds/3 approval buttons, hidden native window/start/frame/exit/reap, ad-hoc codesign and private install tests |
| python3 -I scripts/checks.py kb-records-control; inventory-shape; inventory-shape-control | 0 each | rejected bad record; inventory PASS (1); rejected 2 malformed inventories |
| cargo run --offline --locked -- ir-scan --propose <private-home>/ir-proposals --check-pending tests/fixtures/ir-v1/pending tests/fixtures/ | 0 | /tmp/dream-wp2-ir-scan-final.log; private HOME/TMPDIR, toolchain homes explicitly preserved, seat/capability variables removed |
| git diff --check; git verify-commit da78cbda91b5de454c38152de8c44b5d587f2086 | 0 each | no whitespace defects; Good signature from Boxy |
| python3 -I scripts/checks.py secrets; headers | 0 each | PASS |

Canonical make test's successful rows include both clippy modes with -D warnings; default and ACP/test-fixtures all-target suites; separately isolated seat launch and non-PTY cases; hardening, capability, heartbeat, launch parity and approval controls; native formal and parity; secrets, secrets-control and headers. Its private-home, operator_input=false receipt is .bboxy/runs/test-current.json. The GUI receipt is .bboxy/runs/gui-gate.json with private_home=true and operator_input=false. Application smoke uses ad-hoc signing, private homes and hidden windows; no desktop injection, Apple identity or Keychain request.

A real-process negative control first demonstrated inherited Git Trace2 could write a synthetic environment secret directly outside captured stdout. With the old helper, the focused engine_and_acp_headers_capture_without_persisting_remote_credentials test exits 101 (/tmp/dream-wp2-trace-red.log), failing “VcsMetadataTraceDisabled: inherited Git tracing wrote outside the capture”. Clearing the full inherited GIT_* namespace fixes it; the final primary suite checks no trace file is created for both engine and ACP, and passes. A separate earlier experiment did not reproduce remote-URL config trace leakage; that is not claimed. Only synthetic data was used.

The first standalone private-HOME scanner attempt exited 1 before running the scanner because rustup had no default in the new HOME. Preserving the existing CARGO_HOME/RUSTUP_HOME exactly as the canonical gate does corrected this fixture setup; the scanner then exited 0. No product/toolchain setting was changed.

## Formal coverage and limits

Current-source canonical receipt: verification/evidence/a43642e363138d98873f6ef51c201c8e48fefd105833e90c8585e69ae31a335d/evidence.json; SHA256 a43642e363138d98873f6ef51c201c8e48fefd105833e90c8585e69ae31a335d; source da78cbda91b5de454c38152de8c44b5d587f2086; dirty=false; 130 models, 42 contracts. It was generated after the final Trace2 fix; no inherited dependency receipt is reused. Evidence remains retained outside temporary build cleanup.

WP2 introduces no process owner, registry, retry queue or lifecycle: it reuses the existing process helper unchanged. Its finite process lifecycle/ownership/drain/reap controls are rerun with native/TLC parity; the stalled-Git witness exercises the actual process path. These models do not prove arbitrary Git parsing, URL semantics, syscall scheduling, external configuration behavior, bytes or wall-clock deadlines. Consecutive Git commands are observations, not an atomic changing-checkout transaction. The three-second budget plus inherited cleanup constrains awaited work; it is not a universal OS/filesystem timing guarantee. Unsupported local/file origins stay unknown. Live-watch/reproject VCS header promotion remains outside this brief; native observations/unknown raw data are retained. WP9 owns final combined implementation mappings for the dream pipeline. No sink/injection/adoption behavior or dream model is changed.

## Outstanding gate and retained state

make test remains red solely at the unchanged duplicate-number baseline:

    kb record number 5974 repeats: .bboxy/decisions/d5974-log-ir-v1-only-add.md, .bboxy/decisions/d5974-memories-are-kb-records.md
    make: *** [test] Error 1

Both records are byte-unchanged against prepared base a279e0fd (git diff --exit-code exits 0). No renumbering, filtering, gate waiver or speculative cleanup was performed. A second full canonical run was necessary only because source changed to fix the demonstrated Trace2 leak; no same-premise third retry was made. Acceptance still requires canonical make test to become green after the authorized KB repair.

At handoff the implementation is committed; the report is committed separately with the Report trailer. The five generated retained evidence files are intentionally uncommitted: .bboxy/runs/formal-native.json, gui-gate.json, gui-smoke.png, gui-smoke-scrolled.png and test-current.json. Build/app artifacts stay in ignored target directories. They are observations, not source changes. The source/index scope is clean; the worktree is not globally clean because these generated receipts remain. No secrets or raw operator transcripts are committed.
