---
allowed_paths:
- crates/
- scripts/
- verification/
- Makefile
- tests/
- .bboxy/interfaces/
artifacts:
- '[[01M6BXCZEK9GBWT7V9QERDYYHG]]'
- '[[01M6BXD4SMCP20TQ0NNNRY16HR]]'
- '[[01M6BXCZEK9GBWT7V9QERDYYHG]]'
assignee: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
authorship:
  kind: seat
  process_generation: '1'
  public_key: z7MXMN9zYCeDf8l6mccA4WGnuUPuOhvLzpUcBLQLkXI=
  signer: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
created: 2026-11-01T13:43:35.750Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BNKVCH0RM4EP2Q50FTYTJC
number: 4505
provenance:
  digest: sha256:a45894136f1795442a72cfef19a83bb37b602685773a2d551d3a65226893de4e
  frontmatter_digest: sha256:b596ccbb204d50f062a39f0ae68cffa2285fd3eba94aad07342604da7b5a7477
  log: .bboxy/log/2026-11-01.jsonl#31313
  sig: PsLLMspD2mc9BkuqMc2RUGU0DGIU5zuTCbN6fxDQ3ATXdduUl+GetMyr2SBZBbtS/siSm9mxNjfHMwQ7DWbICw==
  signer: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
rejection:
  by: seat:01M6AC3D2ECK4W3F8495PAQTS3
  reason: |-
    Round 4, a small fix found by land.sh. Landing d32f5b62 failed its presubmit with exit 77; nothing was pushed. scripts/test-landing-recovery.py CiPhaseTests fails 2 tests: test_after_tests_runs_the_remaining_gates_without_repeating_workspace_tests and test_normal_ci_keeps_workspace_build_clippy_and_tests. Both fail with "make: *** No rule to make target `test-capability-scan'" because the fixture's minimal Makefile does not know the two new ci-local steps (test-capability-scan, test-capability-control). Full log: ~/.local/state/bboxy450-term/landing/presubmits/d32f5b626bcdd27b80726ad0d63aa0723e138cd0-b34ec067/commands.log:4136-4176.

    Fix: teach the landing fixtures about the new ci-local steps the same way they handle existing steps (stub targets in the fixture Makefile, or whatever pattern test-landing-recovery.py already uses), and update any expected ci-command sequences. Then run `python3 -I scripts/test-landing-recovery.py` and `python3 -I scripts/test-landing.py`, both exit 0, plus full `make test`. Push the signed head and send task_report with verify_result.command exactly `make test`. Apply the Makefile split from my earlier message (real-Codex scan in its own `test-capability-native` target) now too, so E4's rebase is simpler.
report:
  commit: 30fb71d88c8aa3aa354967668f9c19e186ea79d0
  op_id: 01M6C3Y42HEY3EK3PS6X4V8HKD
  reported_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
  summary: '#4505 round-three Makefile addendum implemented and pushed to OPEN PR #87 on signed capability-file head 30fb71d88c8aa3aa354967668f9c19e186ea79d0. The real-Codex scan is now test-capability-native with prerequisites build, test-capability-file, test-capability-scan; test-harness-native is left to E4, no alias or ci-local invocation. Help and both app-harness ICD references updated. Plain make test exited 0 at default concurrency; make test-capability-native ran once on this final head and exited 0 (four snapshots, 20 process observations/60 checks, zero values/legacy keys, one explicitly counted proven-exited unclassified-owned PID). Current-source formal receipt verifies all 138 variants. Eight signed source commits verified and pushed; no merge/land. Report addendum committed alone on canonical harness main as signed 107eb2edf56e11281fd0096277e39498f5852073: /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:159, SHA256 5ae9b06380456fe256dd5358321ad74d450b30c51605b80cdcbd63a58a750d65. Report commit supplied rather than pushing unrelated local-main history, as allowed. Final native log: /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-final.log. PR #88 was still OPEN when inspected; combined-branch execution is not claimed, and the second landing keeps both targets. Artifact digest refresh again rejected as artifact record projection pending (op 01M6C3X8STBM3R9K70QGS64YC0); the committed log/hash are directly reviewable.'
role: executor
status: reported
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E5: deliver seat capability by 0600 overlay file, never env (bboxy450-term, #4503)'
type: task
updated: 2026-11-01T17:53:53.347Z
updated_by: seat:01M6AMBPG3XD3CMJWHQCKWAG7M
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Real-Codex scan has its own target with the requested dependencies, outside ci-local; E4 owns test-harness-native.
    reference: /Users/feral/mydev/bboxy450-term-capability-file/Makefile:104,794,799-801; /Users/feral/mydev/bboxy450-term-capability-file/.bboxy/interfaces/app-harness.md:25,30; scripts/ci-local.sh unchanged, no test-capability-native/shared_codex.py invocation. make help and make -n test-capability-native exit 0; /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-dry-run.log; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:165
  - criterion: Full canonical unattended make test exits 0 on final committed source at default concurrency.
    reference: '/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-ci-local.log: ci-local: result 30fb71d88c8aa3aa354967668f9c19e186ea79d0 normal 0; all five scanner methods pass, environment control 101→0 with exact restoration, workspace build/clippy/tests/doctests, headless smoke, 27 invariants/0 failures; unchanged 11 default slow exclusions; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:167'
  - criterion: 'Renamed native target rerun once on final head: all production roles scanned; proved exits counted; authentication and lifecycle remain green.'
    reference: '/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-final.log: make test-capability-native exit 0, four-kind file/auth/refusal/lifetime dependency tests plus actual installed Codex warm/direct and cold/shim. /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-scans-final.json: 20 observations/60 checks, zero values/legacy keys, exited_count=1 pid=50751 unclassified-owned absent-on-pgrep-recheck; all four required roles present each snapshot, orient/restart/daemon survival, external_model_calls=0 operator_input=0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:170'
  - criterion: Current clean source/model/tool evidence and full native/TLC parity retained.
    reference: /Users/feral/mydev/bboxy450-term/.git/bboxy-formal-evidence/9992325192846d25fc46dd02dae3e1e3d2530363448e5ab6ed074cf03d231f2e/evidence.json binds 30fb71d8 dirty=false, 138 variants/31 crates/59 contracts. Capability pairs each 10 states/18 edges, expected pass/safety and graph_equal=true. make check-formal-evidence exit 0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:174
  - criterion: Signed head pushed, PR open, report committed to harness main; no merge/land.
    reference: 'https://github.com/bboxy450/bboxy450-term/pull/87 OPEN head 30fb71d88c8aa3aa354967668f9c19e186ea79d0. /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-push.log: make push-branch 0/eight signatures verified. Harness main report-only commit 107eb2edf56e11281fd0096277e39498f5852073 verifies G, key 9B07E849480C68328CA20684844ECC0587DE7AB0, no Report trailer; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:159'
  - criterion: E4 integration limitation recorded; second landing retains both independent targets.
    reference: 'PR #88 OPEN at 56d5ec6e8a695312255145a01fd52f22889f20e6 when inspected; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:176. No import of E4 target/ci-local hunks and no combined-branch test claim. Prior scanner red→green evidence remains in the report; scanner source/tests unchanged by this five-line target split and current canonical tests pass.'
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Work on a new worktree and branch `capability-file` from origin/main, and open a PR. Do not merge and do not run land.sh; the Oracle lands it.

Binding: the harness ICD `.bboxy/interfaces/seat-capability-delivery.md` (#4503, proposed; bboxy450-harness main at 39209d7 or later), §1–§5. Read it fully before you start.

Why: any same-uid process can read a seat's exec-time environment with `ps -Eww`, and that includes BBOXY_CAPABILITY. Unsetting it after exec does not help.

Scope (Side A, every seat kind: claude, codex, zsh, bboxy450-harness):
1. The launch kit writes `<overlay>/capability`: mode 0600, opened with O_EXCL|O_NOFOLLOW, inside the existing 0700 per-launch ULID overlay. The environment carries only `BBOXY_CAPABILITY_FILE`. `BBOXY_CAPABILITY` is never set for any kind.
2. The Codex wrapper and env_vars switch to the file. launch.rs:593-606 already uses an overlay file for Codex.
3. The consumers, bridge.rs:45 and hook.rs:67, read the file on every start. They refuse each of these with a typed reason: a symlink, a file owned by another uid, a mode other than 0600, malformed content.
4. Lifetime: the file is the overlay's, so each generation gets a new file and the old one is removed with the old overlay.
5. Add the §5 #348 rows to bboxy450-term's app-harness ICD in the same PR, as with d4071.

Evidence (§4, unattended, in `make test-harness-native`):
- For every kind, a child of the seat runs `ps -Eww` on the seat and each ancestor up to the supervisor, plus `env`. Every scan must show zero copies of the 64-hex value and zero `BBOXY_CAPABILITY=`.
- orient and one hook event still authenticate.
- File-shape and lifetime checks, and one refusal test per consumer refusal reason.
- Broken control: re-adding the env var makes the scan go 101 → 0.

E4 (branch `harness-embed`) also touches launch.rs. Keep your launch.rs hunks minimal. Whichever PR lands second rebases.

Acceptance:
- Full `make test` exits 0 unattended at default concurrency.
- All commits are signed, with no Report trailer.
- The report goes in bboxy450-harness `.bboxy/runs/2026-11-01-capability-file-report.log`, committed to harness main (d2900).
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Implemented Side A capability-file delivery and opened PR https://github.com/bboxy450/bboxy450-term/pull/87 on capability-file, signed head 69fc2a010c2a1730a800f414ed78736cf84dcb2b. Plain make test exited 0 unattended at default concurrency; native harness, 101->0 leak control and clean current-source formal receipt also pass. Full report: .bboxy/runs/2026-11-01-capability-file-report.log, committed alone on harness main as signed e0c9b6104958a42b4e2b21f025580a09d2ca9a88 (local main record, not pushed). Report artifact 01M6BXCZEK9GBWT7V9QERDYYHG; PR artifact 01M6BXD4SMCP20TQ0NNNRY16HR. No merge or author landing. The report records corrected development failures, the disk/deadline rerun and fixture/native scope limits.

```json
{
  "commit": "69fc2a010c2a1730a800f414ed78736cf84dcb2b",
  "op_id": "01M6BXDZVCNGAB36EMFFVJ849G",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Full make test exits 0 unattended at default concurrency",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/ci-local.log: ci-local: result 69fc2a010c2a1730a800f414ed78736cf84dcb2b normal 0; workspace checks, headless smoke, 27 invariants/0 failures. Default 11 slow invariants remained skipped; no new exclusions."
      },
      {
        "criterion": "Every seat kind keeps admission values and legacy keys out of exec environments",
        "reference": "target/capability-file/native-summary-final.json: four initial kinds, 12 cases, 36 child ps/env scans, zero value matches and zero legacy keys; tests/capability_file.rs:170 and :439."
      },
      {
        "criterion": "Orient and hook authentication, safe file shape, typed refusal and lifetime",
        "reference": "target/capability-file/harness-native-final.log: make test-harness-native exit 0; 24 authenticated MCP starts, 12 HookObserved replies, five refusal reasons in each actual consumer; generation replacement/close removal and copied-old-credential refusal. Real Codex direct and cold/shim pass with zero operator input/external model calls. Source and fixture limits are recorded in the committed report."
      },
      {
        "criterion": "Reintroducing the environment value fails the real scan and restored code passes",
        "reference": "target/capability-file/environment-control.json: SeatAndAncestorEnvironmentIsCapabilityFree red_exit=101, green_exit=0; restored supervisor SHA256 ab9cd871db32e0073b4d0a6de38f0559fd38854977b296becc261fa26339bc37; make test-capability-control exit 0 on final head."
      },
      {
        "criterion": "Native/TLC parity and current-source evidence",
        "reference": "/Users/feral/mydev/bboxy450-term/.git/bboxy-formal-evidence/c7bf45e2b0a3001614d575dbde4dc7bf6f1d862d2b349740775e19b475c7f35f/evidence.json: source 69fc2a01, dirty=false, 138 variants/31 crates/59 contracts. Environment and cleanup pairs each match 10 states/15 edges with pass/expected safety verdicts; make check-formal-evidence exit 0 after controls."
      },
      {
        "criterion": "ICD #348 capability-file amendment",
        "reference": "PR #87 .bboxy/interfaces/app-harness.md:19-30 updates crossing, startup, overlay lifetime and verification for Oracle signature under #4503/d4071."
      },
      {
        "criterion": "Signed source commits and committed log report without Report trailers",
        "reference": "Five source commits and report commit e0c9b6104958a42b4e2b21f025580a09d2ca9a88 verify G with key 9B07E849480C68328CA20684844ECC0587DE7AB0, no Report trailer. /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log is the sole file in that harness-main commit. make push-branch exit 0; PR #87 OPEN at the signed source SHA."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

PR #87 round two is implemented and pushed on capability-file at signed 8e1b58467467f8967f96979520d7066e5b73b3fd; no merge/author landing. Plain make test exited 0 on the first full round-two run at default concurrency, including the real 101->0 control in canonical CI. Real shared Codex warm/direct and cold/shim scans cover daemon, proxy companions and daemon-started MCP bridges before/after replacement: 20 process observations/60 checks, zero values/legacy keys. Exclusive final overlay creation/fstat, typed symlink refusal and failed-plan cleanup are implemented; four stale docs and lowercase parsing are fixed. The updated report is committed alone on canonical harness main as signed 6b52b26e10d37dfe85b2f5288351deddfeb2efce: .bboxy/runs/2026-11-01-capability-file-report.log (artifact 01M6BXCZEK9GBWT7V9QERDYYHG). This is the exact local main commit requested; unrelated dirty KB/main history was not synchronized or pushed. The report describes the conservative kernel-snapshot/env method and scope limits.

```json
{
  "commit": "8e1b58467467f8967f96979520d7066e5b73b3fd",
  "op_id": "01M6C0R5PXGBV7GPQX3VWABEC3",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "MUST 1: real environment control runs inside canonical make test",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/ci-local.log:4387-4395 records make test-capability-scan and make test-capability-control, then red_exit=101/green_exit=0, source restoration hash ab9cd871db32e0073b4d0a6de38f0559fd38854977b296becc261fa26339bc37. Final receipt: ci-local: result 8e1b58467467f8967f96979520d7066e5b73b3fd normal 0. scripts/test-landing.py actual-entrypoint witness executes the control and refuses green on its failure."
      },
      {
        "criterion": "MUST 2: production shared Codex daemon, proxy and MCP process environment coverage",
        "reference": "target/capability-file/round2-native-final.log and round2-native-scans-final.json: both real SDK modes, two stages each, 20 process observations/60 ps-Eww/kernel/environment-serialization checks, zero values/legacy keys. Each stage has one real shared daemon, two --codex-proxy companions, two daemon-started MCP bridges; roles overlap explicitly. Current/retired generation values checked only in RAM. HOME sentinel rejects redaction and process stamps reject replacement. Kernel snapshot includes Darwin auxiliary strings conservatively; env serializes that actual snapshot, not the observer environment. make test-harness-native exit 0 on final signed head."
      },
      {
        "criterion": "MUST 3: exclusive final overlay directory, fstat validation and pre-created symlink refusal",
        "reference": "crates/bboxy450-harness-wake-up/src/capability.rs::create_overlay uses non-recursive final mkdir, O_DIRECTORY|O_NOFOLLOW and fstat directory/euid/exact0700 checks with typed Refusal variants. launch::plan uses the owned guard. seat_wiring::overlay_creation_refuses_existing_symlinks_and_directories passes in the full gate and preserves the unrelated target/link."
      },
      {
        "criterion": "SHOULD 4: post-capability plan failure removes overlay",
        "reference": "seat_wiring::a_wiring_write_failure_removes_its_capability_overlay passes: owned child RLIMIT_FSIZE=65 permits the capability write, forces subsequent Claude wiring EFBIG, and asserts FailedWiringRemovesOwnedOverlay. Guard transfers lifetime only on success. AbandonFailedPlan is modeled with a deliberately broken cleanup control."
      },
      {
        "criterion": "SHOULD 5 and optional 6: stale docs and canonical lowercase format",
        "reference": "PR #87 updates crates/bbt-mux/README.md plus the three specs explicitly named in the authenticated rejection. None still describe the value in env. capability_file_refuses_uppercase_hex passes; writer/reader agree on lowercase65-byte format."
      },
      {
        "criterion": "Full unattended default-concurrency verification and current-source formal parity",
        "reference": "target/ci-local.log normal0 on 8e1b5846; headless smoke, workspace build/clippy/tests/doc tests, 27 invariants/0 failures (default11slow skips unchanged). make test-boundaries, test-harness-native, check-capability-file, test-landing and check-formal-evidence all exit0. Retained /Users/feral/mydev/bboxy450-term/.git/bboxy-formal-evidence/5afb054987642324080da90b1ac181c072271d75e07c17a441016639b1cf7e2d/evidence.json binds clean 8e1b5846, 138 variants/31 crates/59 contracts; four capability models each match10states/18edges, expected pass/safety verdicts."
      },
      {
        "criterion": "Signed source/record commits, updated open PR and committed report",
        "reference": "make push-branch exits0; PR https://github.com/bboxy450/bboxy450-term/pull/87 OPEN at 8e1b58467467f8967f96979520d7066e5b73b3fd. Six source commits and report commit6b52b26e10d37dfe85b2f5288351deddfeb2efce verify G under key9B07E849480C68328CA20684844ECC0587DE7AB0 with no Report trailers. Report SHA2564b8a3da23c1c9c73e800867b3fb799046df0171ab7f62a840c80926090f84062; canonical harness-main commit changes only .bboxy/runs/2026-11-01-capability-file-report.log. No merge or author landing."
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

Round 3: PR #87 updated and pushed at signed head d32f5b626bcdd27b80726ad0d63aa0723e138cd0 (capability-file). Failed scans of labelled live/uncertain PIDs are fatal with role/PID; only ESRCH or a successful pgrep absence permits a counted/reported exit. Named second-bridge regression goes red→green. Plain make test exits 0 at default concurrency. The requested make test-harness-native was run once on this final head, exit 0: four snapshots, 20 process observations, 60 source checks, zero values/legacy keys, one explicitly counted exited unclassified-owned PID. Native log: /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-native-final.log. Native scan remains outside ci-local under this round's accepted native-boundary convention. Report committed on harness main as f11d544e8b83cd0327d5928bf46870e620286462 (signed, only report changed): /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log; SHA256 43bdbc639cff8cd6d457f75bae23267fb14645cfd9b0103c31732ef24942cfec. Harness report commit is supplied rather than pushing unrelated local-main history, as allowed. Source worktree clean; seven signed source commits, no Report trailers; no merge/landing. Artifact-claim refresh was rejected as 'artifact record projection pending' (op 01M6C2DJRNB140Z5NFB3X572R9); operation_status confirms terminal rejected/no retry. The committed report and file hash remain directly reviewable.

```json
{
  "commit": "d32f5b626bcdd27b80726ad0d63aa0723e138cd0",
  "op_id": "01M6C2ERX1BZP8R3TP3HSP2JJ9",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Labelled ps/env scan failure cannot be hidden by another observed process with the same role; error names role and PID.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/scripts/e2e/capability_scan.py:147; /Users/feral/mydev/bboxy450-term-capability-file/scripts/test-capability-scan.py:86; round3-scan-red.log: named test fails with RuntimeError not raised (make 2), round3-scan-green.log: five methods pass (0); /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:129"
      },
      {
        "criterion": "Only proved exits are excused, counted and reported; failed recheck remains fatal.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/scripts/e2e/capability_scan.py:114,177; /Users/feral/mydev/bboxy450-term-capability-file/scripts/test-capability-scan.py:91,101; /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-native-scans-final.json: exited_count=1, pid=40028 roles=[unclassified-owned], reason=absent-on-pgrep-recheck; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:144"
      },
      {
        "criterion": "Full canonical unattended make test at default concurrency exits 0, including the actual env-injection control.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-make-test.log; /Users/feral/mydev/bboxy450-term-capability-file/target/ci-local.log: ci-local result d32f5b626bcdd27b80726ad0d63aa0723e138cd0 normal 0; scanner five methods pass; environment control red_exit=101 green_exit=0; headless smoke; 27 invariants/0 failures, unchanged 11 default slow skips; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:134"
      },
      {
        "criterion": "Real shared-Codex production scan rerun once on final head, native-boundary exception accepted; orient/hook/file lifecycle coverage retained.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-native-final.log: make test-harness-native exit 0, four-kind file/auth/lifetime tests plus warm/direct and cold/shim modes; 20 observations/60 checks all zero; all four required roles in each snapshot; orient/restart/daemon survival true, operator_input=0 external_model_calls=0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:144"
      },
      {
        "criterion": "Current-source native/TLC graph and verdict parity, source/model/tool hashes retained.",
        "reference": "/Users/feral/mydev/bboxy450-term/.git/bboxy-formal-evidence/29b4e14c4cf63abd961f8e7b973409085b4a2ab287ca119aaaad422c84069f2c/evidence.json: d32f5b6 dirty=false, 138 variants/31 crates/59 contracts; capability pairs 10 states/18 edges pass/safety, graph_equal=true; make check-formal-evidence exit 0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:149"
      },
      {
        "criterion": "Signed source branch pushed; report committed to harness main; no merge/land.",
        "reference": "https://github.com/bboxy450/bboxy450-term/pull/87 OPEN head d32f5b626bcdd27b80726ad0d63aa0723e138cd0; /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-push.log: seven signatures verified, make push-branch 0. Report main commit f11d544e8b83cd0327d5928bf46870e620286462 has G signature 9B07E849480C68328CA20684844ECC0587DE7AB0 and changes only /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:155"
      }
    ],
    "exit_code": 0
  }
}
```


## Report from seat:01M6AMBPG3XD3CMJWHQCKWAG7M

#4505 round-three Makefile addendum implemented and pushed to OPEN PR #87 on signed capability-file head 30fb71d88c8aa3aa354967668f9c19e186ea79d0. The real-Codex scan is now test-capability-native with prerequisites build, test-capability-file, test-capability-scan; test-harness-native is left to E4, no alias or ci-local invocation. Help and both app-harness ICD references updated. Plain make test exited 0 at default concurrency; make test-capability-native ran once on this final head and exited 0 (four snapshots, 20 process observations/60 checks, zero values/legacy keys, one explicitly counted proven-exited unclassified-owned PID). Current-source formal receipt verifies all 138 variants. Eight signed source commits verified and pushed; no merge/land. Report addendum committed alone on canonical harness main as signed 107eb2edf56e11281fd0096277e39498f5852073: /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:159, SHA256 5ae9b06380456fe256dd5358321ad74d450b30c51605b80cdcbd63a58a750d65. Report commit supplied rather than pushing unrelated local-main history, as allowed. Final native log: /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-final.log. PR #88 was still OPEN when inspected; combined-branch execution is not claimed, and the second landing keeps both targets. Artifact digest refresh again rejected as artifact record projection pending (op 01M6C3X8STBM3R9K70QGS64YC0); the committed log/hash are directly reviewable.

```json
{
  "commit": "30fb71d88c8aa3aa354967668f9c19e186ea79d0",
  "op_id": "01M6C3Y42HEY3EK3PS6X4V8HKD",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Real-Codex scan has its own target with the requested dependencies, outside ci-local; E4 owns test-harness-native.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/Makefile:104,794,799-801; /Users/feral/mydev/bboxy450-term-capability-file/.bboxy/interfaces/app-harness.md:25,30; scripts/ci-local.sh unchanged, no test-capability-native/shared_codex.py invocation. make help and make -n test-capability-native exit 0; /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-dry-run.log; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:165"
      },
      {
        "criterion": "Full canonical unattended make test exits 0 on final committed source at default concurrency.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-ci-local.log: ci-local: result 30fb71d88c8aa3aa354967668f9c19e186ea79d0 normal 0; all five scanner methods pass, environment control 101→0 with exact restoration, workspace build/clippy/tests/doctests, headless smoke, 27 invariants/0 failures; unchanged 11 default slow exclusions; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:167"
      },
      {
        "criterion": "Renamed native target rerun once on final head: all production roles scanned; proved exits counted; authentication and lifecycle remain green.",
        "reference": "/Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-final.log: make test-capability-native exit 0, four-kind file/auth/refusal/lifetime dependency tests plus actual installed Codex warm/direct and cold/shim. /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-native-scans-final.json: 20 observations/60 checks, zero values/legacy keys, exited_count=1 pid=50751 unclassified-owned absent-on-pgrep-recheck; all four required roles present each snapshot, orient/restart/daemon survival, external_model_calls=0 operator_input=0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:170"
      },
      {
        "criterion": "Current clean source/model/tool evidence and full native/TLC parity retained.",
        "reference": "/Users/feral/mydev/bboxy450-term/.git/bboxy-formal-evidence/9992325192846d25fc46dd02dae3e1e3d2530363448e5ab6ed074cf03d231f2e/evidence.json binds 30fb71d8 dirty=false, 138 variants/31 crates/59 contracts. Capability pairs each 10 states/18 edges, expected pass/safety and graph_equal=true. make check-formal-evidence exit 0; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:174"
      },
      {
        "criterion": "Signed head pushed, PR open, report committed to harness main; no merge/land.",
        "reference": "https://github.com/bboxy450/bboxy450-term/pull/87 OPEN head 30fb71d88c8aa3aa354967668f9c19e186ea79d0. /Users/feral/mydev/bboxy450-term-capability-file/target/capability-file/round3-split-push.log: make push-branch 0/eight signatures verified. Harness main report-only commit 107eb2edf56e11281fd0096277e39498f5852073 verifies G, key 9B07E849480C68328CA20684844ECC0587DE7AB0, no Report trailer; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:159"
      },
      {
        "criterion": "E4 integration limitation recorded; second landing retains both independent targets.",
        "reference": "PR #88 OPEN at 56d5ec6e8a695312255145a01fd52f22889f20e6 when inspected; /Users/feral/mydev/bboxy450-harness/.bboxy/runs/2026-11-01-capability-file-report.log:176. No import of E4 target/ci-local hunks and no combined-branch test claim. Prior scanner red→green evidence remains in the report; scanner source/tests unchanged by this five-line target split and current canonical tests pass."
      }
    ],
    "exit_code": 0
  }
}
```
