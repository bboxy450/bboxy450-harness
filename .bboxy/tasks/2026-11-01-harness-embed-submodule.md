---
allowed_paths:
- .gitmodules
- crates/bboxy450-harness
- Cargo.toml
- Cargo.lock
- Makefile
- scripts/
- verification/
- crates/bbt-mux/src/mcp/seats.rs
- crates/bbt-mux/src/adapters/
- crates/bbt-mux/tests/
- crates/bboxy450-term/src/settings/roles.rs
- crates/bbt-proto/src/gui/roles.rs
- tests/
artifacts:
- '[[01M6C1N43VCM1DYSNX2EC9SP8R]]'
- '[[01M6C4E5QH07Q35PZEDPSJQ8HB]]'
assignee: seat:01M6ADTVVZS36VS43P2PG15HRQ
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T12:12:28.301Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6BGD04REW5PM1QB1TED987Y
number: 3913
provenance:
  digest: sha256:cb9a5aa60db6533dba92944b54ced0535aff9a6f95a6c0b8a04ad1adc77e6461
  frontmatter_digest: sha256:11a8993ea44382a97855170cda26f663a8de4717256fe628ebcf534c408371fb
  log: .bboxy/log/2026-11-01.jsonl#31367
  sig: bezfvk+uT7ZcCa42C3Vzq3/niT5MEmbI3tVNGeg/1FmUcvNgV81zBUxejUOdqxkPXbK+6uBUwO8Gvf1xcu6mAw==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: d6fa3fd5c0f738b9a65e93646e750289914ff722
  op_id: 01M6C4JGB5A2AA0F180AGK09RT
  reported_by: seat:01M6ADTVVZS36VS43P2PG15HRQ
  summary: 'E4 completed and reviewed. PR #88: https://github.com/bboxy450/bboxy450-term/pull/88. Final signed/tested head d6fa3fd5c0f738b9a65e93646e750289914ff722, harness pin 47b140fa9bd2bec99054394c206deb1b36d7b3b0. Parent make test exits 0 at default concurrency, unattended; exact-pin child make test exits 0. Combined native/TLC parity: 201 models, 32 crates, 82 contracts. Report: .bboxy/runs/2026-11-01-embed-submodule-report.log, committed alone on harness main as signed 09aedbcbe533c85f74d9a05441990d55eaf5a197 (SHA256 22cc93068b1e455eba1d854ef71e625c8ec3117cf6f0f5b83c2802435feefead). Per instruction 01M6C3WS387JV0NY2WBNFCTXHV, no further branch push was made; the Oracle is landing reviewed d6fa3fd5 first, and PR #87 will rebase onto E4. No author merge/land.sh/publication.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E4: harness as bboxy450-term submodule + phase-1a process seat profile'
type: task
updated: 2026-11-01T18:24:58.578Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: Canonical submodule joins the workspace and all seven toolkit dependencies resolve in tree; initialization never skips a missing checkout.
    reference: d6fa3fd5 .gitmodules:1, Cargo.toml:34,55; scripts/harness-submodule.py:21,85; final CI make test-harness-native resolution check0; fresh-clone init0 on 653ce725 (init/Cargo/gitlink bytes identical to final head).
  - criterion: Complete formal inventory, imported harness models and broken controls, native/TLC parity and source-bound evidence.
    reference: 'verification/coverage.toml:223; scripts/formal.py:83,92,248,317; receipt bb860d7b4ed94bd5228a9435d74939f04539b0bd7fac22242963729f97d94e9c, clean source d6fa3fd5: 201 models/32 crates/82 contracts; all 67 harness models included.'
  - criterion: Approved Pty/Cadence/hooks=false process seat with executor role and shared startup/MCP wiring.
    reference: mcp/seats.rs:22, adapters/mod.rs:81, settings/roles.rs:18, proto gui/roles.rs:63, wake-up launch.rs:421,516,1064; existing settings integration test:604 verifies catalog and executor/Pty/output-cadence/hooks=false; d4071 and app-harness.md:35 carry exact phase1a ICD rows.
  - criterion: Real scripted-provider seat starts with session ULID, completes one line, closes within bound, and leaves no owned group.
    reference: 'mcp.rs:4964,4988; final retained CI lines4800/4802: Ctrl-D code0 in12.215792ms, owner Killed:9 recorded in292.514875ms, all groups ESRCH; tool ignores HUP+TERM; manual resume preserves completed items and check-transcript0; unknown profile refused; guarded input composer-readiness-missing has zero writes/no retry.'
  - criterion: Parent JSON feature isolation is enforced and has a real rejecting control; ACP remains child-only.
    reference: scripts/harness-submodule.py:56,63,80; six submodule controls0. Parent resolved serde_json features alloc/default/float_roundtrip/std exclude preserve_order; --check-json-features --acp-control exits1 naming harness/acp and serde_json/preserve_order. Original ctl ordering and MCP reordered-argument retry tests unchanged, green.
  - criterion: Standalone harness make test remains unattended and green on exact pinned main.
    reference: make test-harness-standalone0; /tmp/embed-harness-47b-standalone.log; source47b140f, clean child receipt3b9847e910dc7ec0f8b451d6bb05922a1ef60a57af95bcc4e0a52c03a996ebe3:67 models/25 contracts; default/ACP tests, both Clippy modes, GUI tests and headless Metal smoke22 items/15 kinds pass.
  - criterion: Full parent verification at default concurrency, including integration, formal, UI lint, headless smoke and normal invariants.
    reference: target/embed-reviewed-pre-capability-green.log SHA256 91967cc99ff92bca7c9ef98ce85ed3d245503c807f1bca196fee08be315caa1c ends exact d6fa3fd5 normal0;27 invariants0 failures. Deliberate-CLOEXEC case runs separately and passes at line4814; every case runs, no RUST_TEST_THREADS override. make test-boundaries0; /tmp/embed-parent-boundaries.log.
  - criterion: Signed source and one complete committed report; branch held for Oracle landing.
    reference: verify-signatures.py b9015a4..d6fa3fd5 exits0,8 signatures; make check-formal-evidence0; report commit09aedbc changes only .bboxy/runs/2026-11-01-embed-submodule-report.log, signed9B07E849480C68328CA20684844ECC0587DE7AB0; no Report trailers. PR/file artifact claims01M6C1N43VCM1DYSNX2EC9SP8R and01M6C4E5QH07Q35PZEDPSJQ8HB.
  exit_code: 0
---
ROLE: EXECUTOR. Repo bboxy450/bboxy450-term. Work in a new worktree on branch `harness-embed` from origin/main (59dedc64 or later). Open a PR, do not merge, and do not run land.sh. The Oracle lands it.

Binding: the harness ICD `.bboxy/interfaces/embed-seat.md` (in bboxy450/bboxy450-harness), §1–§6. It amends bboxy450-term ICD #348, and the Architect is getting that amendment agreed in parallel. Build to it now. The Oracle won't land until #348 is agreed, and if the agreement changes a clause, you'll get a delta brief.

Scope:
1. **Submodule.**
   - Add `https://github.com/bboxy450/bboxy450-harness` as a git submodule at `crates/bboxy450-harness`, pinned to harness main (cdde43d or the latest at start).
   - It joins the workspace. The harness's `[patch.crates-io]` git-rev pins for the toolkit crates must resolve to the in-tree paths when built inside bboxy450-term: use workspace path deps or the workspace `[patch]`, whichever keeps the standalone `cd crates/bboxy450-harness && make test` working unchanged.
   - Fresh clones need `--recurse-submodules`. Make the ci-local and land gates initialise the submodule unattended. A missing submodule must fail the gate loudly, never skip it.
2. **Verification inventory.**
   - Register the harness crate and its models (it has its own 58 Harness* models) in bboxy450-term's verification inventory, so the "omission must fail" rule holds.
   - Either invoke the harness's `make test-formal` from the workspace formal target, or import its models. Native/TLC parity must still be checked.
3. **Side A seat profile (ICD §4).**
   - Add `"bboxy450-harness"` to `APPROVED_PROFILES` (bbt-mux/src/mcp/seats.rs).
   - Add the adapter record arm mirroring pi/omp: `command = ["bboxy450-harness","seat"]`, `WakeMethod::Pty`, `StateSource::Cadence`, `hooks: false`.
   - Add the role-list kinds in settings/roles.rs and bbt-proto gui/roles.rs.
   - No new WakeMethod and no d20 change.
4. **Evidence (ICD §6, Side A).** Add a `make test-harness-native` seat-spawn case that does all of the following unattended:
   - spawns a `bboxy450-harness` seat with the scripted provider (no network, no key);
   - asserts the first-line `session <ulid>`;
   - submits one line and sees a completed turn;
   - closes the seat;
   - asserts exit within the ICD bound and no orphaned process group (`kill(-pgid,0)==ESRCH`).

   Add a broken control: an unknown profile must be refused with a reason. If the EmbedLifecycle model lives in the harness, reference it; don't duplicate it.

Acceptance:
- In bboxy450-term, full `make test` exits 0 at default concurrency, unattended, including the new harness case and the formal inventory.
- In `crates/bboxy450-harness`, standalone `make test` still exits 0.
- All commits are signed. Put no Report trailer on the commits. The full report goes in bboxy450-harness `.bboxy/runs/2026-11-01-embed-submodule-report.log`, committed to harness main, as with d2900.
- Send one task_report with verify_result.command exactly `make test`.

## Report from seat:01M6ADTVVZS36VS43P2PG15HRQ

E4 completed and reviewed. PR #88: https://github.com/bboxy450/bboxy450-term/pull/88. Final signed/tested head d6fa3fd5c0f738b9a65e93646e750289914ff722, harness pin 47b140fa9bd2bec99054394c206deb1b36d7b3b0. Parent make test exits 0 at default concurrency, unattended; exact-pin child make test exits 0. Combined native/TLC parity: 201 models, 32 crates, 82 contracts. Report: .bboxy/runs/2026-11-01-embed-submodule-report.log, committed alone on harness main as signed 09aedbcbe533c85f74d9a05441990d55eaf5a197 (SHA256 22cc93068b1e455eba1d854ef71e625c8ec3117cf6f0f5b83c2802435feefead). Per instruction 01M6C3WS387JV0NY2WBNFCTXHV, no further branch push was made; the Oracle is landing reviewed d6fa3fd5 first, and PR #87 will rebase onto E4. No author merge/land.sh/publication.

```json
{
  "commit": "d6fa3fd5c0f738b9a65e93646e750289914ff722",
  "op_id": "01M6C4JGB5A2AA0F180AGK09RT",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "Canonical submodule joins the workspace and all seven toolkit dependencies resolve in tree; initialization never skips a missing checkout.",
        "reference": "d6fa3fd5 .gitmodules:1, Cargo.toml:34,55; scripts/harness-submodule.py:21,85; final CI make test-harness-native resolution check0; fresh-clone init0 on 653ce725 (init/Cargo/gitlink bytes identical to final head)."
      },
      {
        "criterion": "Complete formal inventory, imported harness models and broken controls, native/TLC parity and source-bound evidence.",
        "reference": "verification/coverage.toml:223; scripts/formal.py:83,92,248,317; receipt bb860d7b4ed94bd5228a9435d74939f04539b0bd7fac22242963729f97d94e9c, clean source d6fa3fd5: 201 models/32 crates/82 contracts; all 67 harness models included."
      },
      {
        "criterion": "Approved Pty/Cadence/hooks=false process seat with executor role and shared startup/MCP wiring.",
        "reference": "mcp/seats.rs:22, adapters/mod.rs:81, settings/roles.rs:18, proto gui/roles.rs:63, wake-up launch.rs:421,516,1064; existing settings integration test:604 verifies catalog and executor/Pty/output-cadence/hooks=false; d4071 and app-harness.md:35 carry exact phase1a ICD rows."
      },
      {
        "criterion": "Real scripted-provider seat starts with session ULID, completes one line, closes within bound, and leaves no owned group.",
        "reference": "mcp.rs:4964,4988; final retained CI lines4800/4802: Ctrl-D code0 in12.215792ms, owner Killed:9 recorded in292.514875ms, all groups ESRCH; tool ignores HUP+TERM; manual resume preserves completed items and check-transcript0; unknown profile refused; guarded input composer-readiness-missing has zero writes/no retry."
      },
      {
        "criterion": "Parent JSON feature isolation is enforced and has a real rejecting control; ACP remains child-only.",
        "reference": "scripts/harness-submodule.py:56,63,80; six submodule controls0. Parent resolved serde_json features alloc/default/float_roundtrip/std exclude preserve_order; --check-json-features --acp-control exits1 naming harness/acp and serde_json/preserve_order. Original ctl ordering and MCP reordered-argument retry tests unchanged, green."
      },
      {
        "criterion": "Standalone harness make test remains unattended and green on exact pinned main.",
        "reference": "make test-harness-standalone0; /tmp/embed-harness-47b-standalone.log; source47b140f, clean child receipt3b9847e910dc7ec0f8b451d6bb05922a1ef60a57af95bcc4e0a52c03a996ebe3:67 models/25 contracts; default/ACP tests, both Clippy modes, GUI tests and headless Metal smoke22 items/15 kinds pass."
      },
      {
        "criterion": "Full parent verification at default concurrency, including integration, formal, UI lint, headless smoke and normal invariants.",
        "reference": "target/embed-reviewed-pre-capability-green.log SHA256 91967cc99ff92bca7c9ef98ce85ed3d245503c807f1bca196fee08be315caa1c ends exact d6fa3fd5 normal0;27 invariants0 failures. Deliberate-CLOEXEC case runs separately and passes at line4814; every case runs, no RUST_TEST_THREADS override. make test-boundaries0; /tmp/embed-parent-boundaries.log."
      },
      {
        "criterion": "Signed source and one complete committed report; branch held for Oracle landing.",
        "reference": "verify-signatures.py b9015a4..d6fa3fd5 exits0,8 signatures; make check-formal-evidence0; report commit09aedbc changes only .bboxy/runs/2026-11-01-embed-submodule-report.log, signed9B07E849480C68328CA20684844ECC0587DE7AB0; no Report trailers. PR/file artifact claims01M6C1N43VCM1DYSNX2EC9SP8R and01M6C4E5QH07Q35PZEDPSJQ8HB."
      }
    ],
    "exit_code": 0
  }
}
```
