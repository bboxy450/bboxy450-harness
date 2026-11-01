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
  digest: sha256:c2e87681f557baa5e5f4428006c26e44357e51905be2e8ebe2d12bda7227dd4a
  frontmatter_digest: sha256:acc97e61f1b06a7ec19582b007e6edf766d197af1b74ee06735f3609165deed0
  log: .bboxy/log/2026-11-01.jsonl#209
  sig: qlZ+5McxgOWJyn0Odww+HzIGw12jXkURMyutjr/RrJIX0QvLizKdAM4Kfq9c15pibXoqtZff/DMB6cmPCha2AA==
  signer: seat:01M6AC3D2ECK4W3F8495PAQTS3
role: executor
status: claimed
swarm: 01M6ABW0CX6MY0D5WGCSQ3ADTG
title: 'E1: bboxy450-harness core — protocol, engine, provider, tools, store, gateway, headless, formal models, live smoke'
type: task
updated: 2026-11-01T02:08:24.158Z
updated_by: seat:01M6AC3D2ECK4W3F8495PAQTS3
verify: make test
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
