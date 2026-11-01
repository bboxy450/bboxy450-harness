---
allowed_paths:
- Cargo.toml
- Cargo.lock
- Makefile
- src/
- tests/
- verification/
- scripts/
- README.md
- .bboxy/runs/
- .bboxy/tasks/2026-10-31-harness-core.md
artifacts:
- '[[01M6AKVSFC8K7WNPQA8Y95Y61W]]'
- '[[01M6AGP92W2FFTB7MJG4P7RYVV]]'
- '[[01M6AKX0TDJW6Z6M5QQXMW3Z9Z]]'
assignee: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
authorship:
  kind: seat
  process_generation: '1'
  public_key: 0Yj/k2rOV2tLHQm2aES5d/roSsYDRC/E70ukAxvovgo=
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
created: 2026-11-01T02:07:53.479Z
created_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
delegated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
id: 01M6ADSZHWN6SMV968JZKTPEE1
number: 48
provenance:
  digest: sha256:47d1adccc2a9332436267a2ccc1f9ab604cc0bb7ed80cdad21891e4eb9db8d6f
  frontmatter_digest: sha256:8413addba4a36e94001d846a8fa64801ccd7123069ed1045bddb71a7a72909bc
  log: .bboxy/log/2026-11-01.jsonl#4940
  sig: ZX/YdGE/UfCFFKYiTqf5ec4/SjSfNmHVOlfb61OBl+FEW6yslUjOht1hZzPOUXi0z+ft0iiTIJJ0RJ3JIuEQAQ==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
report:
  commit: b1ccc7c4304b0fa564fc13a30984c97ab0b8eeed
  op_id: 01M6AKX11WC90D1D1F4M97BZMJ
  reported_by: seat:01M6ADTTF9QWWFZFVZJHJ3DD8E
  summary: 'E1 implemented and pushed on e1-core. [PR #1](https://github.com/bboxy450/bboxy450-harness/pull/1). [Full committed report](https://github.com/bboxy450/bboxy450-harness/blob/b1ccc7c4304b0fa564fc13a30984c97ab0b8eeed/.bboxy/runs/2026-10-31-harness-core-report.log). make test: exit 0 (56 Rust tests, 27 native/Java models); make smoke-live: exit 0, PASS with 3 requests/read 1/edit 1/laws held. Final-HEAD parity and strict receipt check: exit 0; receipt SHA256 541ce524906c6db524ea8644c463bbc58dd85fdaa108ea0f6cd78f76a833e519. Signed head verified; all tracked files clean, local untracked verification/evidence archives retained. Prior profile timeout, scope limits and all per-outcome evidence are in the report. No merge.'
role: executor
status: accepted
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1: bboxy450-harness core — protocol, engine, provider, tools, store, gateway, headless, formal models, live smoke'
type: task
updated: 2026-11-01T04:01:23.916Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
verify_result:
  command: make test
  evidence:
  - criterion: 1 Package skeleton, pinned toolkit dependencies and exact headers
    reference: Cargo.toml:1; scripts/checks.py; .bboxy/runs/test-current.json — fmt/clippy/headers exit 0. Skeleton pushed at 364b92a.
  - criterion: 2 Exact protocol, schema, golden fixtures and transcript laws pushed early
    reference: src/protocol/laws.rs:23; tests/protocol_golden.rs:295; protocol.schema.json; tests/fixtures/protocol/*.jsonl; early commit e438e2e.
  - criterion: 3 Engine, provider, tools, policy, store, gateway and frontends
    reference: .bboxy/runs/2026-10-31-harness-core-report.log — numbered outcome 3 gives file:line mechanisms; .bboxy/runs/test-output-current.log — 56 Rust tests exit 0, including real cancellation/PTY/Unix/compaction/fork witnesses.
  - criterion: 4 Native formal models, broken controls, pinned Java parity and fail-closed inventory
    reference: verification/coverage.toml; verification/contracts.toml; verification/corpus.toml; verification/evidence/541ce524906c6db524ea8644c463bbc58dd85fdaa108ea0f6cd78f76a833e519/evidence.json — 27 complete state/edge/verdict comparisons, 13 broken controls; final HEAD b1ccc7c; make check-formal-evidence exit 0.
  - criterion: 5 Bounded, redacted live smoke
    reference: .bboxy/runs/smoke-live-2026-10-31.json — make smoke-live exit 0, outcome PASS, requests 3, read_calls 1, edit_calls 1, file_changed true, ending end_turn, check_transcript true; only !cmd source copied.
  exit_code: 0
---
# E1: bboxy450-harness core

ROLE: EXECUTOR (Codex). The Oracle is Vesper Greywind and the Architect is Nyx Copperwing.

**Worktree:** `/Users/feral/mydev/bboxy450-harness-core`, branch `e1-core`, repo `bboxy450/bboxy450-harness`. Work only here. Push the branch and open a PR. Do not merge.

## Read first, in this order
The ICDs and invariants are the law. If code and an ICD disagree, the ICD wins. If you find a gap in an ICD, report it; do not invent the missing behaviour.

1. `.bboxy/specs/harness-zoomed-out.md`
2. `.bboxy/specs/toolkit-lowering.md`
3. `.bboxy/decisions/architect-review-v0.md`, all 9 amendments, which are binding
4. `.bboxy/interfaces/protocol.md`, baselined v0.1 and the critical path
5. `.bboxy/interfaces/gateway.md`
6. `.bboxy/interfaces/provider.md`
7. `.bboxy/interfaces/interfaces.md`, the verification matrix
8. `.bboxy/specs/budgets.md`
9. `.bboxy/invariants/*.md`
10. `.bboxy/research/formal-reuse.md`, if present
11. `.bboxy/research/kittens-spec.md`, for lineage only

## Outcome, in order. Push after each step so others can build on it.
1. **Package skeleton.** One package named `bboxy450-harness`.
   - No `[workspace]` section and no inherited fields.
   - Features `default=[]`, `standalone`, `embedded`, `acp`.
   - Edition 2024. Toolkit dependencies on crates.io `bboxy450-rt`/`bboxy450-yarn` 0.1.1.
   - Every source file carries the 6-line header from `ATTRIBUTION.md`.
2. **`src/protocol/`, exactly as ICD protocol.md.**
   - `protocol.schema.json` exported via schemars.
   - Golden fixtures in `tests/fixtures/protocol/*.jsonl`. They must cover a turn with text, reasoning, parallel read tools, an edit FileChange, a Terminal, a Plan, an Approval, an interrupt mid-tool, recovery, Lagged and Reset.
   - `src/protocol/laws.rs::check_transcript`.
   - `tests/protocol_golden.rs`.
   - **Push this step early:** E3, the GUI face, builds against these fixtures.
3. **Engine and the rest.**
   - SessionActor with a work lane plus a never-Busy control lane.
   - TurnRunner written as a `bbt_yarn` `reactor!`.
   - Cancellation tree with process-group kill.
   - Provider: OpenAI-compatible streaming with strict tool-call assembly. Retry only before the first output, and honour Retry-After.
   - Tools: read, write, edit (exact match, with "did you mean" reporting), apply_patch, exec/bash, grep, glob. Concurrency classes plus per-path write locks.
   - Policy: rules as data, `on_unattended=deny`.
   - Store: single-writer JSONL with epochs, recovery and blobs.
   - Gateway: in-process plus a Unix socket, subscribe `after_seq`, ItemSnapshot, Lagged.
   - Headless face: `-p "…" --json` emits envelopes.
   - Seat line mode, per amendment §5.
   - MCP client is optional for this task.
4. **Formal models.** Reuse bboxy450-term's stack: `bboxy450-tlc` from crates.io and Yarn. Do not build a parallel checker.
   - Models: AgentTurnLifecycle, ToolPairing, CancelTree (A→B→A), SteerQueue/InputPreemption, ApprovalLifecycle and SubscriberReplay.
   - Each model has a Broken control that must fail, as listed in `interfaces.md`.
   - Provide `make test-formal` (native) and `make test-formal-parity` (pinned Java TLC; it may download into a cache, and Java never runs in the product).
   - `verification/coverage.toml` inventories the crate. An omission must fail.
5. **Live smoke.** `make smoke-live`, kept out of `make test`, per amendment §8.
   - Profile `~/.bboxy450-harness/profiles/nemotron.md`, model `nvidia/nemotron-3.5-super-vl:free`, endpoint `https://openrouter.ai/api/v1`.
   - The key comes from the same `!cmd` the existing `~/.kittens/profiles/nemotron.md` uses. Copy that api_key command and never the value. Never print or commit a key.
   - At most 3 requests. The smoke asks the model to read a file and make one edit, and `check_transcript` must pass.
   - Exit codes: no key → SKIP, exit 0; quota → UNAVAILABLE; neither is ever reported as PASS.
   - Write the summary to `.bboxy/runs/smoke-live-<date>.json`, redacted.

## Verify
- `make test` must run with nobody at the machine. It covers: fmt check, `clippy -D warnings`, unit and e2e tests against mock SSE fixtures, `check_transcript` over every transcript, `make test-formal`, the secret scan, and the license-header check.
- Also run `make smoke-live` once and record its real outcome.

## Report (one final report, no progress chatter)
- files and mechanism
- commands with exit codes
- evidence per numbered outcome
- gaps
- the branch and head commit, which must be signed

Blockers that stop all work: at most 3 lines to the Oracle.


## Report from seat:01M6ADTTF9QWWFZFVZJHJ3DD8E

E1 implemented and pushed on e1-core. [PR #1](https://github.com/bboxy450/bboxy450-harness/pull/1). [Full committed report](https://github.com/bboxy450/bboxy450-harness/blob/b1ccc7c4304b0fa564fc13a30984c97ab0b8eeed/.bboxy/runs/2026-10-31-harness-core-report.log). make test: exit 0 (56 Rust tests, 27 native/Java models); make smoke-live: exit 0, PASS with 3 requests/read 1/edit 1/laws held. Final-HEAD parity and strict receipt check: exit 0; receipt SHA256 541ce524906c6db524ea8644c463bbc58dd85fdaa108ea0f6cd78f76a833e519. Signed head verified; all tracked files clean, local untracked verification/evidence archives retained. Prior profile timeout, scope limits and all per-outcome evidence are in the report. No merge.

```json
{
  "commit": "b1ccc7c4304b0fa564fc13a30984c97ab0b8eeed",
  "op_id": "01M6AKX11WC90D1D1F4M97BZMJ",
  "verify_result": {
    "command": "make test",
    "evidence": [
      {
        "criterion": "1 Package skeleton, pinned toolkit dependencies and exact headers",
        "reference": "Cargo.toml:1; scripts/checks.py; .bboxy/runs/test-current.json — fmt/clippy/headers exit 0. Skeleton pushed at 364b92a."
      },
      {
        "criterion": "2 Exact protocol, schema, golden fixtures and transcript laws pushed early",
        "reference": "src/protocol/laws.rs:23; tests/protocol_golden.rs:295; protocol.schema.json; tests/fixtures/protocol/*.jsonl; early commit e438e2e."
      },
      {
        "criterion": "3 Engine, provider, tools, policy, store, gateway and frontends",
        "reference": ".bboxy/runs/2026-10-31-harness-core-report.log — numbered outcome 3 gives file:line mechanisms; .bboxy/runs/test-output-current.log — 56 Rust tests exit 0, including real cancellation/PTY/Unix/compaction/fork witnesses."
      },
      {
        "criterion": "4 Native formal models, broken controls, pinned Java parity and fail-closed inventory",
        "reference": "verification/coverage.toml; verification/contracts.toml; verification/corpus.toml; verification/evidence/541ce524906c6db524ea8644c463bbc58dd85fdaa108ea0f6cd78f76a833e519/evidence.json — 27 complete state/edge/verdict comparisons, 13 broken controls; final HEAD b1ccc7c; make check-formal-evidence exit 0."
      },
      {
        "criterion": "5 Bounded, redacted live smoke",
        "reference": ".bboxy/runs/smoke-live-2026-10-31.json — make smoke-live exit 0, outcome PASS, requests 3, read_calls 1, edit_calls 1, file_changed true, ending end_turn, check_transcript true; only !cmd source copied."
      }
    ],
    "exit_code": 0
  }
}
```
