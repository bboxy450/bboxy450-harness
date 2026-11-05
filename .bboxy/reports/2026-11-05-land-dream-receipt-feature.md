---
type: report
title: Passive Receipt carrier and blob custody on current main
task: '[[01M6MP8BAE48MQYR8RG2K5BWZP]]'
commits:
- f41a99bf458e08c76b9c92a02d6350f227af4af7
- 65c68d94c20ec48fee3195bb9f31f50db7d5faa1
verify:
  command: make test-ir && cargo test --features acp-agent --test protocol_golden --test ir_v1_corpus --test store_fork && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-05T01:58:42.077Z
id: 01M6MPW0SKM5F3Y13A9683DQSS
updated: 2026-11-05T01:58:42.077Z
number: 20261103138015
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:9e27aead0526d1c3756d691d4c92eaad7939deb74b741b1988a4b4408464fa85
  frontmatter_digest: sha256:da39836adca72732fa970ef30da4f8e5c3f937fdb0bf3b73679c75adf5f86b64
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: tixnCsD5oEOAxMTikom3X6ZQBwn3TQwP/Z49DENMIO93uAfgs53HMiqnIqhMATv1mSTXBFTm09WjnY+t5FedCA==
  log: .bboxy/log/2026-11-05.jsonl#50899
---
The passive Receipt feature is prepared on current main in local branch feature/dream-receipt-land-20261105. The exact revision0 verification exits0, including IR compatibility, 16 corpus tests, 9 golden tests, 4 ordinary fork tests, all-feature all-target clippy, 136 full native/Java graph/transition/verdict comparisons, 45 contracts and private native app/check-transcript smoke. The actual 16-fixture Unix/InProc GUI parity test also exits0. This is a local signed handoff; no push, main merge, release or live model use was performed.

Task [[01M6MP8BAE48MQYR8RG2K5BWZP]] was fetched through the authenticated knowledge tools at etag 286243f83b96e31b7163b95468c6bd9b38caa8551d2267cf9b6a22e48d990d3f. The authenticated Executor assignment, [[d20261103136614-dream-incremental-feature-landings]] and accepted original carrier task [[01M6JTM3T7RB0TQNDARJVY6PAS]] govern this bounded landing. Every command and edit uses /Users/feral/mydev/bboxy450-harness-dream-wp4.

The fetched signed base is exactly fc6c0ef9754ea4f530111f54a23096f3434d151c (merged PR28), with approved Boxy signature. Only approved source 4ebde7d542b2414422680d07c0695a7f5ff6a752 was cherry-picked with -x/signing, producing f41a99bf458e08c76b9c92a02d6350f227af4af7. Its message preserves the original SHA/Original-Report and carries this task's Report trailer. Evidence commit 65c68d94c20ec48fee3195bb9f31f50db7d5faa1 retains the actual logs and byte proofs. Both new commits and the original source verify with Boxy <bboxy450@gmail.com>, key 9B07E849480C68328CA20684844ECC0587DE7AB0. No older carrier report or tooling snapshot commit was cherry-picked.

The only conflict was verification/coverage.toml. Resolution retains current main's entire inventory and adds only the original passive Receipt implementation-only note. Unrelated incoming WP2/WP3/planned-runtime notes were discarded because those sources are absent on this base. All 15 affected files have exactly the same added/removed lines as the approved original patch, despite newer surrounding main code. No whole old source file replaced current main. Source/registration for every current model, the PR28 conditional compatibility/check declarations, canonical snapshots, dependencies, license/ATTRIBUTION, agent configuration and existing scrolling/ACP implementations remain unchanged.

src/protocol/mod.rs adds ReceiptKind {Publication,Run,Unknown}, ReceiptOutcome {Published,Idempotent,Skipped,Completed,Failed,Unknown}, Item::Receipt and ItemKind::Receipt, plus optional Adoption.receipt. Item's kind discriminator remains receipt; the separate field uses receipt_kind with no alias that could duplicate kind. Fields retain optional/default reader behavior and nested unknown values. Existing interrupted-item handling and neutral extension labels cover the passive variant. The named frontend edits are exhaustive read-only labels/arms only.

src/protocol/laws.rs:265,432,538,664,687 — authored receipts require an existing dream writer session, current schema, a nonempty correctly typed receipt blob and structural optional harness/sink data. Recursive Unknown values are refused for new authorship. Adoption.receipt, when present, follows the current lowercase sha256 digest shape; absence retains valid historical Adoption authoring. These structural checks do not validate receipt body semantics, trusted producer authority or historical causal cuts. tests/protocol_golden.rs:693 checks the exact single discriminator/default/schema behavior; tests/ir_v1_corpus.rs:811,854 covers typed fixture pins, incomplete/future fields and nested Unknown authoring controls.

src/store/fork.rs:17 only includes Receipt.blob in the existing referenced-blob set. tests/ir_v1_corpus.rs:973 uses actual private Store logs/blobs, retained future/unknown envelope data, forks, Unix transport and a real headless replay process to assert exact retained envelope lines and real blob bytes/hashes. The original source can be removed without losing the child's custody. tests/ir_v1_corpus.rs:1137 removes/corrupts an actual referenced blob and requires fork refusal before child publication. No public fork classifier, ownership, no-replace or recovery semantics changed. These are actual byte/IO/process witnesses, not a semantic audit or producer.

Only tests/fixtures/ir-v1/v1.3-receipt.jsonl is added, SHA256 101e9b668eb2df169ccbf3698cbece93679b4e0c221d5e5712f86d73ffc85b8b. All 120 previous fixture files, including all 15 direct frozen IR JSONL files, are byte-identical to fetched main. Schema and lock changes are the additive Receipt/default/enum/Adoption-field union; current main's other 1.3 and agent configuration fields remain. The existing tests/gui_bridge.rs:148 test derives its entire file set from the actual lock and checks it against directory enumeration before rendering both transports; it ran 1 real test over all 16 locked fixtures, with no hardcoded 15-file shortcut.

Before switching branches, source/index were clean except the known generated native JSON. dream/wp7-r3 remains exactly 1c86843ed128e54232a678006bb78effcab3428a. Its175-model generated bytes were copied and hash-checked to feature-land-receipt-base-native175.json, SHA256 80698c710478e5f5e61f799ddd669529ce1c473d45debf1322ab98cff3b9efb7, before narrowly restoring that one generated path. All 22 then-untracked proof files were hashed and remain exact. No reset, clean, peer-tree/cache deletion or new worktree was used. The tracked native snapshot supplied by main is historical 81-model source b3ef7468, retained separately; it is explicitly not fresh fc6 evidence.

feature-land-receipt-preservation.json records 3,762 unchanged base file hashes, all 120 old fixture hashes, all 15 exact original-patch deltas and retained WP7/untracked proofs; SHA256 2a3daa625c410fc0296b55773b7a7bbfb70cd791a44f6e4ee6e140de5d527f11. The first read-only preservation instrument exited1 at CLAUDE.md because it compared followed content with a Git symlink-target blob. The corrected instrument compares exact readlink target bytes, passes and retains that diagnostic separately; no source was changed to satisfy it.

Exact revision0 command:

```sh
make test-ir && cargo test --features acp-agent --test protocol_golden --test ir_v1_corpus --test store_fork && cargo clippy --all-targets --all-features -- -D warnings && make test-formal && make test-formal-parity && make check-formal-evidence && make smoke-app
```

At clean source f41a99bf458e08c76b9c92a02d6350f227af4af7 this command exits0 in 155.161s. Log .bboxy/runs/feature-land-receipt-primary.log SHA256 0d92bef2cf49314bd5d74419005c266df242c657f9669ec65f4a189fcbbc60d5. Fresh full formal receipt 86024aef00a2dc4e50ed99f7b7abc420ace299f4ffb993edd2c8d1d5054a7296 binds that source, dirty=false and 340 current inputs; 136 models contain 85,693 states/306,680 labeled edges. Complete native/Java reachable sets and safety/deadlock/liveness verdicts match, including deliberate broken controls. The carrier introduces no lifecycle family. Fork-model abstraction does not prove bytes, cryptography, kernel/IO scheduling or receipt authority; the actual specialized tests cover this feature's implementation boundary.

Additional exact commands and results:

| Command | Exit | Raw log SHA256 |
|---|---:|---|
| cargo test --offline --locked --features acp,test-fixtures --test gui_bridge unix_and_inproc_views_equal_every_ir_v1_fixture -- --exact | 0 | b14be488beb60183c8a2b0f8fe07487d844001b02e9012a2271c2404d7106310 |
| python3 -I scripts/checks.py headers && python3 -I scripts/checks.py secrets && python3 -I scripts/checks.py kb-records && python3 -I scripts/checks.py kb-records-control && python3 -I scripts/checks.py inventory-shape && python3 -I scripts/checks.py inventory-shape-control && git diff --check fc6c0ef9754ea4f530111f54a23096f3434d151c HEAD -- protocol.schema.json src tests verification | 0 | a4fcaaf0cff5a7fbeae9814a905222458180baaf5b8898bea9cac4c7d93c3121 |

The native smoke creates a hidden real NSWindow/Metal surface, submits a private frame, checks bundled replay/transcript, strict ad-hoc signing and actual launcher exit/reap; all exit0. Visible desktop presentation is explicitly not tested. Private HOME/TMPDIR, offline cached dependencies and 2 jobs are used in the existing assigned target. Compiler wrappers are disabled only for these commands; no shared daemon/process/cache is controlled. Command JSON records retain actual exits, timestamps, source and 304 source/test/script hashes, checked against current bytes.

The supplemental cargo fmt --check exited1 on seven inherited current-main files (anim.rs, view.rs, headless.rs, acp_agent.rs, formal/models.rs, formal_models.rs and gui_scroll.rs). feature-land-receipt-fmt-baseline.json uses read-only base/current stdin formatting and proves identical formatting edits on both sources, SHA256 910b4eafbca6f46210718691a8dfeb5f437d395e30692d4f4deb05a52a63a855. No formatting/GUI/model scope was expanded and this failure is not called green. Its original source-check log SHA256 6d4cf028fb21e566c366975818a345fb331f5cf72e12e9a3a7c98da4050bb5d2 remains; the independent remaining checks above really executed and passed.

The old KB5974 hold is no longer current: normal kb-records exits0 on 2,926 records, with its broken control also passing. No identity, number or checker was rewritten here. The inherited GUI zoom timer101 is documented in [[reports/2026-11-05-land-dream-ir-feature]]; this passive carrier does not repair or claim that case. Full make test/all-target product acceptance is not claimed. The exact bounded task gate and requested parity/native checks are green.

feature-land-receipt-manifest.json indexes the actual source-current command results, preservation and remaining limits, SHA256 a6b0a8ac7d77e4d45aec7b16d243438f2369e2dc7b21dd1b86c04df4ef8aa06d. After committing this exact host-produced report, a new complete exact verify/formal/native run binds the report HEAD, with final hashes and source/index versus generated/post-report proof state supplied in task_report. Earlier receipts do not certify that later HEAD. No full Dream audit/trial/synthesis/lifecycle, live policy/bootstrap, publication/adoption producer or WP7–9 implementation is landed by this task. Stop is the signed handoff for Oracle review/merge.