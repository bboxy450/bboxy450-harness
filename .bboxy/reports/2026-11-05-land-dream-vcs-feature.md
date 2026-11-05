---
type: report
title: 'Dream VCS feature landing: current-main integration, private process controls and source-bound verification'
task: '[[01M6MNT4H1YN6DXA9RYA2ZQ1ZX]]'
commits:
- 7466caae377b02a1e16996cf4a2d83a78a08031b
- 9eef3a0fb966b1d20b600390d085fb3760692428
- f137fc78e4dcce71585263dcd4e2e2d70c33caea
verify:
  command: cargo test --features acp --test dream_vcs && cargo test --test import_native --test ir_edge && make test-ir && cargo test --test protocol_golden --test ir_v1_corpus && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
  exit_code: 0
verify_revision: 0
author: seat:01M6KDR96J84HR56MCXQ2JSW5Y
created: 2026-11-05T02:00:31.426Z
source: f137fc78e4dcce71585263dcd4e2e2d70c33caea
id: 01M6MPZBQBF367WMFV9PFV6AD2
updated: 2026-11-05T02:00:31.426Z
number: 20261103138017
created_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
updated_by: seat:01M6KDR96J84HR56MCXQ2JSW5Y
authorship:
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  kind: seat
  public_key: +go7EGHCYyMM/7hLFRzzAOFaKqWCHKAaxEdDnpzj8+I=
  process_generation: '1'
provenance:
  digest: sha256:3e15f0249bd300d56d332fc2f1b151b442583cabc7ef5c0dce7675a7f5a75cb0
  frontmatter_digest: sha256:ecd41387f9860d78b9d47dd00ab1eaaf78a78b933ce0a39ea7c8cf1e0ebc8084
  signer: seat:01M6KDR96J84HR56MCXQ2JSW5Y
  sig: Swtyny6DCNKr7mFIk66XF6mVdK9xLiIk7+CWp2gfcnVskzc+K9krBDlRgOsXkaJc1SGHHyDXVA55cfUnErbeCQ==
  log: .bboxy/log/2026-11-05.jsonl#50917
---
Prepared the accepted WP2 VCS feature on signed published main `fc6c0ef9754ea4f530111f54a23096f3434d151c`, without merging or pushing. The exact task verify revision 0 exits 0 at signed implementation HEAD `f137fc78e4dcce71585263dcd4e2e2d70c33caea`. Fresh native/Java complete reachable-state, action-labeled-transition and verdict comparisons match for all 136 models, 45 contracts and one crate (dirty=false). No Dream pipeline, Receipt, D0, audit, trial, synthesis, policy/bootstrap implementation or dependency/tooling change is included.

The assigned writing checkout/cache is /Users/feral/mydev/bboxy450-harness-dream-wp6. At entry, branch landing/dream-ir13-20261105 and HEAD were exactly fc6c0ef9754ea4f530111f54a23096f3434d151c; source/index were clean except the known generated formal-native.json. That file was copied byte-exact to .bboxy/runs/feature-land-vcs-retained-native.json, hash-checked (SHA256 fd5a099e99968f99965128bfb36eaa33beaf5e99c9dcb650f8132cafe1c87c6a), then only the generated original was restored before branching. Fetch returned the same signed origin/main. Main's signature and both original WP2 source signatures verify with approved key 9B07E849480C68328CA20684844ECC0587DE7AB0, Boxy <bboxy450@gmail.com>.

Protected branches remain exact: landing/dream-ir13-20261105=fc6c0ef9754ea4f530111f54a23096f3434d151c, feature/dream-ir13-land-20261105=5a6d3789f12d3308ba04baf2fcb5110a102ce825, dream/wp6=c98d185bc2182f3a37ceee936031081f8c7cb2bd. All pre-existing final IR proof files and Oracle report-head native output remain unchanged. Their hashes are in feature-land-vcs-preservation.json. No worktree, cache, report or old proof was deleted, reset or cleaned.

| Original signed source | New signed cherry-pick (-x) |
|---|---|
| da78cbda91b5de454c38152de8c44b5d587f2086 | 7466caae377b02a1e16996cf4a2d83a78a08031b |
| 8b4be671e2955b59edd53ff55f8e1645ec4f146f | 9eef3a0fb966b1d20b600390d085fb3760692428 |

The only cherry-pick conflict was verification/coverage.toml. Resolution retained the complete current-main inventory and added only src/vcs.rs plus the exact accepted WP2 implementation-only note. The old candidate's unrelated Dream-model note/planned_sources were not imported. All other hunks merged mechanically, preserving current engine/backend changes. feature-land-vcs-conflicts.json records this disposition.

A necessary fixture integration is signed `f137fc78e4dcce71585263dcd4e2e2d70c33caea`: tests/dream_vcs.rs pins the scripted leg to an explicit private vcs-scripted profile and adds child status/terminal-error diagnostics. A preceding ACP fixture leg discovers adapter profiles in the same private HOME; the legacy scripted leg's default codex profile could then route through Host's ACP selection despite the supplied ScriptedProvider. The retained failure identifies exit1/acp=false with Auth at session/new, rather than a VCS dirty-state assertion. Reading existing src/onboarding.rs:333 and src/gateway/host.rs:287 establishes this selection path. No auth interaction was performed; the failed leg produced no successful model turn. Explicit fixture profile selection fixes this test isolation defect without changing product onboarding, bootstrap, Host dispatch, budgets or assertions. All original VCS/submodule/credential assertions remain intact.

Acceptance mechanisms:

| Path/line | Mechanism and actual witness |
|---|---|
| src/vcs.rs:15,92,176 | Three-second shared deadline for five sequential Git queries; 16KiB stdout cap and existing bounded capture_stdout cancellation/group termination/drain/reap. Query failures, invalid output, cap and timeout remain unavailable, including dirty=None. Canonicalization uses existing permit-backed blocking helper. |
| src/vcs.rs:20,45,150 | Credential/query/fragment stripping and shared host/owner/name normalization; actual top-level path fallback only when origin is absent. No repository remains None. No invented host or native fields. |
| src/vcs.rs:212; src/engine/mod.rs:180; src/backend/acp_agent.rs:256 | Capture only before a new session's Store::open; skip supplied VCS or an existing/unreadable log. Engine and ACP emitted plus persisted SessionStarted headers observe actual Git data. Closed replay keeps original bytes without probing Git. |
| src/vcs.rs:190 | Status forces --ignore-submodules=none despite both local and .gitmodules ignore=all. Signed private parent/child fixtures exercise changed child worktree and changed gitlink through public capture and actual scripted/ACP/persisted headers. |
| src/vcs.rs:99 | Remove the entire inherited GIT_* namespace before fixed local-only Git options. No pager, helper, prompt, fsmonitor, hook or transport. Trace2 cannot write inherited environment secrets around stdout capture. |
| src/import/edge.rs:15,45; src/import/mod.rs:16,149,950,961 | Promote only actual native Claude branch or Codex repository/branch/commit data. Partial/absent data remains absent; dirty is not inferred from today's checkout. Known credential-bearing Codex URLs are sanitized in raw and ext projections; unfamiliar Git fields remain available under existing redaction. One-shot native parser revision is shared constant3. |
| src/import/reproject.rs:70; tests/ir_edge.rs:183 | Native reprojection uses the same PARSER_REVISION instead of inventing a new version per run; non-native mapper behavior remains. Derived replay/source immutability assertions remain; no VCS header promotion or source-prefix rewrite added. |

Tests/dream_vcs.rs covers eight actual cases: normalization; clean/dirty origin capture; no-origin/no-repository; engine+ACP credential-free emitted/persisted headers; missing/failing Git; capped/failed status preserving known branch; tracked submodule worktree/gitlink; and a TERM-ignoring stalled Git leader/member. The stall test records only its own exact PIDs and checks both are gone/reaped after bounded startup. Fixture commits use disposable SSH signing keys, never operator keys/Keychain. tests/import_native.rs:872,966 verifies native availability, unguessed absence, unknown field custody and sanitized raw/ext credentials. tests/cli_headless.rs retains two real process witnesses for no-repo metadata and byte-identical closed replay without credential or Git resolution.

Independent deliberately broken implementation controls were run against this composed source and the real process tests, then src/vcs.rs was restored byte-exact after each:

| Control and actual command | Result/log SHA256 |
|---|---|
| Change only status to --ignore-submodules=all; cargo test --features acp --test dream_vcs changed_submodule_worktree_and_gitlink_are_dirty_in_actual_headers -- --exact --nocapture | Expected101; named VcsSubmoduleDirty, actual Some(false) versus Some(true), at the actual scripted header. feature-land-vcs-control-ignore-all.log SHA256 c61dde83aa7acccf35f354f4f4e089817926fc72614e5dfb1c9e273ed6a8986c |
| Disable only inherited GIT_* removal; cargo test --features acp --test dream_vcs engine_and_acp_headers_capture_without_persisting_remote_credentials -- --exact --nocapture | Expected101; named VcsMetadataTraceDisabled, Git tracing created a file outside capture. feature-land-vcs-control-trace-inherited.log SHA256 b3e56bf0a45d352a7d07b03fe431e2418857eb685ddfc256acdf7e4bedcbf633 |

feature-land-vcs-controls.json records each command, real/expected exit, required marker, good/control source hashes and exact restoration. Good src/vcs.rs SHA256 a4f28febe102e2736e85a7ff70547b510a84c6134b9b04ca823068463da7dec3 equals the accepted correction's file byte-for-byte. No broken control was committed or used for passing formal evidence.

Current-main preservation is asserted for 205 protected files/hashes in feature-land-vcs-base.json and feature-land-vcs-preservation-checked.json: protocol/schema/IR lock, frozen/native/GUI fixtures, GUI/scrolling, licenses/ATTRIBUTION, Cargo files, all models/controls/corpus/contracts, adapted checkers and pinned snapshots are unchanged. No original WP2 report was copied. The sole formal inventory delta is the new module and one implementation-only note; existing owned-process contracts are reused unchanged. No new owner, global queue, retry or lifecycle was introduced. Git parsing, non-atomic changing-checkout observations, URL interpretation, raw bytes, filesystem/syscall scheduling and actual wall-clock guarantees remain implementation coverage, not a Rust/Git refinement proof.

Exact source verification revision0:

```sh
cargo test --features acp --test dream_vcs && cargo test --test import_native --test ir_edge && make test-ir && cargo test --test protocol_golden --test ir_v1_corpus && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
```

The complete && chain exits0 at f137fc78e4dcce71585263dcd4e2e2d70c33caea, so every listed command exits0. Durable log .bboxy/runs/feature-land-vcs-source-verify.log SHA256 `51db19739b8c85d39c4e347855153595f82ca5ca6f7046c908a5b812f5e70604`; matching JSON records 159.067 seconds. Counts: VCS8, import15, IR edge8, corpus11, golden8, formal model11, nonvacuity26, reactor1, infrastructure8. All-target/all-feature Clippy passes. make test-ir includes its compatibility/broken controls. All 136 fresh native/Java full state/transition/verdict checks match, 45 contracts/one crate, dirty=false; source receipt `49a371f544731609c1fccd791af85bdd6c67c4124f974baec36a2ccb351497ca` is retained at verification/evidence/<receipt>/evidence.json.

Actual make smoke-app passes header/license checks, standalone release bundle/plist, strict ad-hoc signing, Metal replay (21 visible items/14 kinds/readable payloads/three approval buttons), hidden native window/surface/live frame, child exit/reap and the actual check-transcript subprocess. feature-land-vcs-source-smoke-evidence.json records replay_exit_code=launcher_exit_code=transcript_exit_code=0; no visible desktop presentation is claimed. No operator input or Apple signing identity was required. All builds/features were serialized in the assigned target with lean profiles, offline Cargo and two jobs; fixtures used private HOME/TMPDIR/repositories/keys/sockets.

Supplemental exact commands and retained failures:

| Command | Exit and durable log hash |
|---|---|
| Initial cargo test --features acp --test dream_vcs && cargo test --test cli_headless | 101 before CLI leg; seven VCS cases passed, reused-home submodule fixture failed before VCS assertion. feature-land-vcs-focused.log f414b9a02521508b0967936628bcef02231b823bd2772b3600d0a104bbc30fd4 |
| Focused submodule case after adding status diagnostics | 101, actual child exit1/acp=false; feature-land-vcs-submodule-diagnostic.log cfd8ed43b6b2bc68c3b81b37addb872a30836104eca2407c62c3f0da11665161 |
| Focused submodule case after terminal-error diagnostics | 101, Auth/session-new selection defect described above; feature-land-vcs-submodule-terminal.log 522424316f15b91fb153902cb6e88c7134a7b3524173c4ee58bb8bf674a44f7e |
| cargo test --test import_native --test ir_edge --test cli_headless | 0, 15+8+2 cases; feature-land-vcs-inherited-focused.log e4679ee61bef233cc6133dc4f0c1b534c9c35c82b939880c2e539be0cd5d83c9 |
| cargo test --features acp --test dream_vcs && cargo test --test cli_headless after explicit fixture profile fix | 0, 8+2 cases; feature-land-vcs-fixture-fixed.log 8d115d52d853efb98481813a6aac0101c9cd82914712ceb2f4499067503de3f7 |

The failed logs remain actual failures, not retrospective passes. The fixture fix was based on captured terminal evidence and the actual selector code, not a retry/lock/budget change. Obsolete IR baseline failures, Ite/dependency changes, unrelated GUI fixes, benchmarks and full-suite loops were not rerun. Prior external target cleanup remains unproven and is not attributed to this task; all previous durable reports/proofs and protected branch/untracked hashes remain intact.

All three new source commits verify as author/committer Boxy <bboxy450@gmail.com>, approved key9B07E849480C68328CA20684844ECC0587DE7AB0, with Report trailers. At the report's implementation snapshot, source/index are clean; generated formal-native output is separate. This append-only host report and task-owned evidence will be committed with Report trailer. That advances HEAD, so the unchanged exact revision0 command is then rerun at report HEAD, producing fresh .bboxy/runs/feature-land-vcs-final-head.json/.log and its own full native/Java archive. The final signed SHA, actual final command exit/log hash and receipt are supplied in task_report. The f137fc7 receipt above is not rebound as final-HEAD evidence, and a final-HEAD pass is not asserted before it runs. Stop after that green handoff for Oracle PR/merge; no push, merge, release, live policy/bootstrap or recruitment is performed.
