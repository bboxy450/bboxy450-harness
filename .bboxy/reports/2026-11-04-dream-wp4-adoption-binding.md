---
type: report
title: 'Dream WP4: containing-session Adoption binding and fresh verification'
task: '[[01M6J8TV4EVP7GZG875FC5J4P5]]'
supersedes: '[[01M6KQH16NND639THWAJAE4NQ2]]'
commits:
- 0fea1767e931953b2a3caeaaa9b8fc32ddd25fc7
- 75486df6ad3082f8c145b2e54ea2171e26b37001
- ba7f829d89dc03a5ace3b2da603b4e10f586e985
- d3aab3b08bb0be24518b66b44fed83d9209d61e5
verify:
  command: cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
  exit_code: 0
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T17:25:07.135Z
id: 01M6KSFKXQWTGWQXX68N9K3356
updated: 2026-11-04T17:25:07.135Z
number: 20261103136505
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:c429b282ae42253922036bfd794f94d879d716ec40b38a6392c93791b37aeac3
  frontmatter_digest: sha256:39c7ff832144faeb54b077f61f480aed8333785506560cc27b10960af74abfce
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: 3HRAQhXIg/EqVwSlI0x25sGdaUP/qPjODfVnHqhHO7oUM1P/kBU5Ft1pgkghwz1fFpFLHv6ZI64RtFhMXm9wDQ==
  log: .bboxy/log/2026-11-04.jsonl#45309
---
# Dream WP4 Adoption binding correction

Implemented the rejected P2 boundary in signed source **d3aab3b08bb0be24518b66b44fed83d9209d61e5**. The exact assigned verification exited **0** with **43 primary tests, 138 full native/Java graph/verdict comparisons and 45 contracts**; fresh app/native smoke, all-feature clippy, headers and secrets checks also exited0. This report supersedes [[01M6KQH16NND639THWAJAE4NQ2]] / report commitba7f829. Its earlier complete-gate0 and source75486df results remain historical and did not establish the missing global Adoption binding. Canonical KB5974 remains unwaived; this is an isolated Executor handoff, not acceptance or publication.

## Current brief, source and scope

Task01M6J8TV4EVP7GZG875FC5J4P5 was fetched after signed rejection message2B9HZMW2VKH1CRP2WWBB8DZJBT: etag4ed68fa65a80e20d6a777e502e3a510ffd8ef6e288ca562a04daea0e82667801, verify revision3, status rejected, same assignee, worktree /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp4. The rejection requests this in-scope repair and fresh report/evidence, with unchanged exact verify. Governing §13.5 P2(d) requires that publication session S own exactly its post-Receipt Adoption set; the operative constructive seal adds the contiguous suffix. Missing/unresolvable receipt remains UNPROVEN; known binding violations are FAIL. No stateful/concurrency, threshold, IR/schema or permission expansion was requested or made.

Original prepared base7d3e22cbc4442cee8ea8bdef6e23f5225708680c and checked dependency integration base10c722781bcdaaaaac2fa0266f0787fdf93034f2 are unchanged. The earlier own implementation chain b48d44c/0fea176/75486df and append-only reports remain intact. Compare this repair against **ba7f829d89dc03a5ace3b2da603b4e10f586e985**: source d3aab3b changes exactly seven scoped files—src/dream/audit.rs, src/dream/receipt.rs, tests/dream_audit_broken.rs, tests/dream_seal.rs, tests/fixtures/dream/good/audit_fixture.rs, verification/contracts.toml and verification/coverage.toml. Source commit d3aab3b is signed Boxy <bboxy450@gmail.com>; actual git verify-commit exited0. Headers/origin/policy remain attached, including credit beside the regression adapted from the Oracle instrument.

No other worktree, shared checkout source, dependency protocol/schema/corpus/frontend/tooling source, package/dependency/feature, model property/control, source budget, trust policy or generated evidence from a peer was edited. No rebase/reset, recruitment, merge, push, publication, live policy/bootstrap or real executable approval occurred. Required formal-native.json/generated evidence is retained outside source commits. The new full-report commit follows tested source; final-report HEAD gets fresh parity/evidence validation after commit, with its actual receipt linked by task_report rather than transferring a source-head receipt.

## Counterexample, mechanism and repair

The Oracle retained its actual public-API probe under shared .bboxy/runs/oracle-wp4-foreign-adoption-20261104/. I read probe.rs/result.json and independently hashed probe.log: **22ba3f32f91d8407be8fde8e72517fc67752487185ba449419c81df9e77dd0e5**. Its recorded baseline audit0 and foreign-session audit0/findings[] produced probe101, expected audit1. The rejected complete assigned gate0 remains historical, as does our previous source gate at75486df and final-head parity atba7f829.

The defect was a global digest-only join: after a publication's own S passed check_publication, another session's Adoption could reuse its successful receipt digest. The publication-local field/count checks never saw that foreign event.

- **receipt.rs:1571, check_adoption:** one shared pure check binds an actual durable completed Adoption to a qualifying Published/Idempotent publication. It checks containing session equality **before any sequence arithmetic**, then selected entry, exact harness/sink/block/RecordPin and receipt digest. Within that same log, its completion must lie in the publication's contiguous R4 Adoption suffix: positive even local offset, at most twice the selected count. No timestamps or cross-log ordinal ordering are used.
- **receipt::check_publication:** uses that same check for each of S's digest-bound Adoptions, retaining exact selected-set count and entry uniqueness. No duplicate or displaced completion can substitute for another selected entry. Skipped publications still authorize none.
- **audit.rs:849–885:** every global Adoption fact resolves its actual retained envelope and runs the shared check against a qualifying publication, instead of accepting digest membership. A known accepted receipt whose publication failed qualification is a known P2 violation; absent/unresolvable receipt remains lost_witness/unproven. Foreign facts are rejected, not filtered out.
- **Existing behavior retained:** same-session published and idempotent positives, skipped-Recall correction, exact current-body/header/blob/cut/source checks, low-trust rendering, stdout held-writer/awaited-flush transport, byte identity and unknown roundtripping remain in place. No writer/ownership/concurrency primitive or product producer was added.

## Real API/CLI and pure controls

tests/dream_audit_broken.rs:461, adoption_cannot_borrow_a_publication_from_another_session, uses sufficient::AuditFixture::new(false), actual private signed Git/KB/artifact-policy fixtures and public audit APIs. Its baseline API and real isolated CLI both return0. It clones one successful Adoption into fresh ordinary dream session42, creates that session's valid final run receipt with fixture.finish/Owner::default, then runs both actual API and CLI. After repair both return1 with **P2 violation adoption_session**. The foreign log is retained byte-identically by the audit.

**Before repair**, the permanent regression itself exited101 and printed baseline0/foreign API0/foreign CLI0/findings[]: /tmp/dream-wp4-adoption-red.log SHA25611c2fc4256460ca58dfeaa475b7c6b9dfe5f33424991060d51228c1294080122. **After repair**, it exited0 and printed API1/CLI1 with adoption_session: /tmp/dream-wp4-adoption-green.log SHA256e5a4699bb279447409a2a9738a30cfb2356764f29f2e1bf4a8e2e0eb0f756cb1. Red101 and the prior green complete gate are both preserved, not substituted for one another.

tests/dream_audit_broken.rs:525 builds seven genuinely sealed retained controls: altered entry, harness, sink, block digest, record digest, missing record and duplicate Adoption. The fixture transform runs **before final sealing**, so valid retained run/header/prefix evidence cannot supply an accidental own_prefix_mismatch rejection. Each control retains the same referenced good digest, calls actual audit API and CLI, returns1 with a named P2 violation, and leaves all retained logs byte-identical. Original published/idempotent same-session positives remain audit0/CLI0.

tests/dream_seal.rs:357 additionally checks the shared interpreter against an actual accepted publication/Adoption: foreign sessions with seq0 and u64::MAX both yield adoption_session, proving session mismatch is handled before ordinal comparison. Same-session before/equal/odd/after-suffix positions yield adoption_order; the actual completion passes.

The focused two-suite run exited0: seven audit tests (including all seven altered/duplicate cases), seven seal tests plus its explicitly invoked private child; /tmp/dream-wp4-adoption-suites.log SHA2567e09e3f8a7be9d395d678195fecead0739588493f569fba23057d458e55ca711. The fresh full primary below reruns them with all other assigned suites.

## Full WP4 acceptance mapping

| Criterion | Mechanism and current executable evidence |
|---|---|
| P1 finite exact closure and pins | audit::run/Resolver::record/evidence/projection::union; fixed identity/CRLF/root/main/nofollow and conflicting durable identity tests; named P1 broken API/CLI rejection |
| P2 publications/Adoption/Recall | R1–R4/E/cut/projection/rank/budget/generated pins and outer run checks; shared global/local Adoption membership/session/full-field/position binding; sufficient publication/idempotent/stale/skip/Recall positives and all new actual API/CLI negatives |
| P3 origin | known DreamRun trigger and MemoryWrite-in-dream checks; named P3 broken fixture |
| P4 append-only and identity | seven-field NFC EntryId and exact read-only source prefixes; actual own-custody conflict preservation; real private delayed-child WRITE/RESTORE ABA observer still yields P4 FAIL despite equal bytes; active/unobserved work remains unproven |
| P5 same-pair passed trials | Projection::build/qualifying_trial known thresholds, resolving same-entry/harness pins, held-out/control/run/usage/spend gates, terminal/revival fences; graph and P5 negatives |
| P6 trust/instruction barrier | exact DSSE V plus independently authorized trust/2 T/root/signer/ancestry/artifact/gate/task/report checks; K2/rotation/executable/report negative fixtures; non-Proposal instruction sinks rejected |
| P7 scope | actual committed root/repo/cross-repo opt-in and actual sink configuration; missing native scope unproven; scope and P7 controls |
| P8 secrets | shared deny/registered-value scans of draft/record/rendered block/authored receipt, quoted data and scrubbed diagnostics; every family/key/value/registered pattern plus P8 control |
| P9 content versus projection | exact body/content identity separated from derived status/harness metadata; changed bytes preserved, never overwritten as repair; body/stale metadata and P9 controls |
| why/graph, additive custody | actual read-only audit/why sink:line chains, finite cross-root links and optional strength; skipped receipt has no materialized-line authority; all15 frozen JSONL bytes still match original7d3e22cb |
| bounded/unattended operation | per-invocation nonblocking seal owner and exact private processes, homes/keyrings and actual CLI; fresh release/ad-hoc replay/hidden native/Metal/process/check-transcript smoke0 |

The full pure validator/trust/seal implementation, committed vectors and earlier substrate regression logs remain documented in [[reports/2026-11-04-dream-wp4-recall-final]] and [[reports/2026-11-04-dream-wp4-audit-seal]]. Their superseded completion claims and source-bound receipts do not certify this repair. Isolated synthetic policy/gate rows exercise reader trust/receipt joins; they do not approve a live binary or prove that seven product gate witnesses ran.

## Model boundary and fresh source-bound verification

This is a correction to pure interpretation of retained facts, not a new stateful contract. Existing HarnessDreamSink represents publication/receipt confirmation with opaque identities; it does **not** encode concrete log-session IDs, line sequences, full RecordPins or digest/body joins. The independent foreign-session counterexample passed the old model gate. It is now rejected by real consumer API/CLI tests, not by claiming the quotient proves Rust refinement. contracts/coverage now explicitly name that abstraction and actual shared-check/API/process tests, including dream_seal's ordinal controls. No model/action/property/control/bound was weakened or omitted.

The earlier required model-before-owner evidence and real delayed-child controls remain valid history. The current full parity refresh checks all **138 reachable-state sets, labeled-transition sets and verdicts**, **45 contracts**, one crate, including HarnessDreamSeal pass35states/61edges and expected child/late-write controls59/100 and50/101 (TLC12 safety). Hashes, signatures, syscall bytes/durability, arbitrary Rust/OS behavior and truthful complete producer observation remain implementation/W assumptions and actual witnesses; native loader/consolidation/forgetting is excluded. WP8/9 retain real producer wrappers, independent sink observer/live executable gates and final combined mapping. Missing/unapproved proof remains unproven; no self-approval/bootstrap or global queue is introduced.

Exact assigned command:
```
cargo test --test dream_audit_broken --test dream_graph --test dream_identity --test dream_trust --test dream_seal && make test-formal && make test-formal-parity && make check-formal-evidence
```
**Exit0 at d3aab3b**, /tmp/dream-wp4-adoption-final-primary.log SHA256f50e0942d6689e3a19b10c1e11bce4906cb3cacbf4f8345063e3af372d1c18a8. **43 primary tests:** audit7, graph8, identity13, seal7 plus invoked private child, trust8. Native formal/nonvacuity/reactor and all138 native/Java comparisons passed. Current-source receipt **62698bdbcd3b3a24769b8b31c5581970e03ee93f8d52ff85460f79ef1f55ead1**, source d3aab3b08bb0be24518b66b44fed83d9209d61e5, dirty=false,138models/45contracts/1crate, retained at verification/evidence/<digest>/evidence.json and target/formal/evidence.json. Exact check-formal-evidence exited0.

Additional fresh commands at the same source:
| Command | Actual exit | Log |
|---|---|---|
| cargo fmt --check; git diff --check |0 each| collected before source commit |
| cargo clippy --offline --locked --all-targets --all-features -- -D warnings |0| /tmp/dream-wp4-adoption-final-clippy.log |
| make smoke-app |0| /tmp/dream-wp4-adoption-final-smoke.log |
| python3 -I scripts/checks.py headers |0| /tmp/dream-wp4-adoption-final-headers.log |
| python3 -I scripts/checks.py secrets |0| /tmp/dream-wp4-adoption-final-secrets.log |

Actual command/exit/source rows /tmp/dream-wp4-adoption-final-verify.json SHA25606ac7230dfcfbd797b99f03b7b8eb3815bed5976439d64ea6898ff3cf0074b44. Clippy log SHA256655ee0e1cd040ebd44de242abbc1ea1b307a981ad57a44bc1db443ab0f3478b5. Smoke log SHA256bdb851507aaf7f677b777912b05578b56bab988f1e3ab85f3a9565504607c730. Actual target/app-smoke/evidence.json SHA2566c99cb1e5caef7aef2b620afa441ed8096c0f645daa71002914cffc6f6ba2240 names d3aab3b, private_home=true/operator_input=false and replay/native launcher/codesign/actual CLI check-transcript exits0. This exercises actual release/ad-hoc bundle, PNG replay, hidden NSWindow/Metal surface/private live frame and owned process exit/reap. No desktop input/capture, visible-presentation claim, Apple signing identity, Keychain prompt or operator credential.

check_transcript means protocol::laws::check_transcript/check_authored in audit/fixtures/replay/corpus and the actual bundled CLI checker, never a fabricated shell target. All15 prior frozen JSONL files were re-compared byte-for-byte with7d3e22cb after this source gate. Earlier test-ir/golden/corpus/fork/edge/replay/library regressions remain accurately attributed to their older unchanged substrate source; no fresh full canonical/default+ACP/GUI/packaging gate is claimed here.

## Failures and unwaived acceptance limits

The actual red foreign-session API/CLI0 counterexample invalidates the earlier completion claim; both its Oracle probe101 and our permanent red101 are retained. The old41-test/138-model complete-gate0 remains historical and was insufficient to establish this boundary. New focused controls and fresh43-test source gate pass, with the model's concrete-identity gap explicitly documented.

Canonical make test remains **not green** and was not rerun in an unchanged red loop. Retained actual /tmp/dream-wp3-make-test.log SHA256651c9eace96f8d01c7378655c00203e3bb1825b1e03f71662d0cdfae90d193d3 records make2 after kb-records1 for duplicate KB5974. The two conflicting records remain unchanged (28f3442ce7dbfa406518135828bf5f063f1b06b573f3fa2b7987ef4c832ba450 and d2eb97c855304806f9da7abfaa6ff6b2d003a1320aea582982fbe963c378a4d4). No number repair, filtering/waiver, threshold/schema change, main acceptance or publication occurred. Original mirror conflicts remain under dream-log precedence.

Source is clean; required tracked generated formal-native.json remains modified and retained, alongside ignored source-bound evidence. Final-report HEAD parity/evidence is refreshed after committing this append-only report and supplied separately in task_report. The Oracle independently verifies this repair; canonical/coherent-main full gates and downstream real producer/observer/serve-time approval obligations remain outstanding as assigned, without an operator step.
