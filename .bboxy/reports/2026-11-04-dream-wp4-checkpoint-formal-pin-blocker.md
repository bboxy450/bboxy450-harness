---
type: report
title: 'Dream WP4 checkpoint: post-report formal gate blocked by changed upstream source pins'
task: '[[01M6J8TV4EVP7GZG875FC5J4P5]]'
supersedes: '[[01M6KFSB6JQA1VRP4XCA7FZ26Q]]'
commits:
- b48d44cd5123a9842dd96b25cdf1a1313c07810b
- 29afe26a320d40649938a02e10e6b1e68f13a39b
verify:
  command: make test-formal-parity && make check-formal-evidence
  exit_code: 2
author: seat:01M6KDR90VJTRA1S722FZFECRZ
created: 2026-11-04T14:40:29.227Z
id: 01M6KG25JS2DZE6ECB640S2DDB
updated: 2026-11-04T14:40:29.227Z
number: 20261103136430
created_by: seat:01M6KDR90VJTRA1S722FZFECRZ
updated_by: seat:01M6KDR90VJTRA1S722FZFECRZ
authorship:
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  kind: seat
  public_key: ehJK2YEPauOppEESuGiWGjLMLDo566y1WrkV6fcY2oE=
  process_generation: '1'
provenance:
  digest: sha256:40dc331a239952f5da5df2579ae3d9dbb5a36905aeb282e8165efd8bdaa78302
  frontmatter_digest: sha256:bb23c189f450098ddb4409ac062324729d86b3050330544748cbcd2d8c909e0f
  signer: seat:01M6KDR90VJTRA1S722FZFECRZ
  sig: bhiWCWUkPayrTiFp37N05RrwdelsDIO9aZRa9U+6b07pA2rb5weMcgmmOJL7iWax3n/MKFErnWb3LZOXE7vXDA==
  log: .bboxy/log/2026-11-04.jsonl#44521
---
# WP4 checkpoint correction: current formal source pins moved

This append-only correction supersedes [[reports/2026-11-04-dream-wp4-pure-verifiers-checkpoint]] for the **latest formal gate result**. Its historical exact primary exit 0 at source b48d44c remains accurate; it is not evidence of a green gate after upstream main changed.

Source `b48d44cd5123a9842dd96b25cdf1a1313c07810b` and report `29afe26a320d40649938a02e10e6b1e68f13a39b` are both signed Boxy <bboxy450@gmail.com>, verified exit 0. Their source/test bytes are equal. Worktree is /Users/feral/mydev/bboxy450-harness-dream-wp4, branch dream/wp4, original comparison base 7d3e22cb.

The actual post-report command `make test-formal-parity && make check-formal-evidence` exited **2**. Parity itself exited 0: 135 native/Java reachable-state, labeled-edge and verdict comparisons, 44 contracts, one crate. Generated receipt `a75e50de0e4af68a4f2b08bd342d0ccccb0a2d79234b07a6dbaecf127c932772` names 29afe26, dirty=false. The immediately following evidence gate refused:

```
formal: FAILED: upstream main SOURCE hash changed; re-copy scripts/formal.py
make: *** [check-formal-evidence] Error 1
```

The combined/current gate is **not green**. Log /tmp/dream-wp4-report-head-formal.log SHA256 `78baaab398fc193ec99a3247ae05ad706d4f311aaf3cace2558dad9fe1e25c0c`. That receipt is retained as actual generated output, not validated current-tool evidence.

A read-only diagnostic at /Users/feral/mydev/bboxy450-term main `fe8454217ba8ae43325f6d69964c1a1de15e8779` confirms all three pins differ:

| source | pinned SHA256 | actual upstream main SHA256 |
|---|---|---|
| scripts/formal.py | 7ac68f32996c5a3ec051dce6886a2419431a264f763924d36796d3d2b401100d | 09ca7496419d59f0a41245cff8297b7fdb79dba2a9b81abf5df9e4a6597aa937 |
| scripts/test-formal.py | 808ceaff7994903925defd8b08ac45eaeecec7d46d7d6ee7d711901d6571a7b3 | 7627077e427cecf992e00bba1c0ed55f13843f897502f54545cd2e5dd9a4aa85 |
| crates/bboxy450-tlc/examples/verify.rs | 88b6400d6a883b25f2eca777cb936ba568f9e9ef702f51aef4f7b83e022c8fb8 | f45b999db8be8da4a18afd36ef24558c9654f9d58d642d5c30bb1ff7abbdc278 |

Those snapshots/pins and scripts are outside WP4's allowed write scope. No checker/snapshot/pin or toolkit source was edited, no environment override bypassed the live source check, and no unchanged red loop was repeated. The exact decision needed is an authorized independently reviewed toolkit-source integration/migration amendment covering these pins/snapshots, in addition to the already-missing final constructive receipt seal and independently checked typed Receipt carrier integration amendment.

The complete P2/P4 audit, receipt/header roundtrip and successful historical good fixture remain unimplemented as recorded in the first report. Every current dream run stays P4 UNPROVEN missing_run_receipt; Adoption history remains P2 UNPROVEN. The canonical KB5974 blocker remains unwaived and was not rerun. No completion task_report, acceptance, merge, publication or live-policy bootstrap is claimed. The first report's 30 scoped tests, 47 regressions, clippy, exact bytes and earlier primary command remain historical checkpoint evidence with their stated limits.

This correction adds only its own report. Mandated formal-native/generated evidence is preserved outside commits. No earlier receipt covers the next report HEAD or the future combined carrier/seal implementation; a later coherent source/tool integration needs fresh exact checks.
