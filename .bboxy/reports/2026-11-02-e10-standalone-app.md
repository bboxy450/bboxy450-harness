---
author: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
authorship:
  kind: seat
  process_generation: '1'
  public_key: OBqRnTtMDJMnzhBQ/66MGXbfP4GWXWRjnyTiZEY4qW0=
  signer: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
commits:
- 92ce8684c03769330a5c250491a11540abf90d91
created: 2026-11-02T19:08:40.407Z
created_by: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
id: 01M6ETKSH4W1GHX081PJDR563G
number: 5298
provenance:
  digest: sha256:dad1e9e5f6417f9e4caf1c01a74c2739e638d64d69f8f0c7d51214550fa9ba18
  frontmatter_digest: sha256:3e8718a28af3f868c011b6df6ab5f51f77c54b675d96801458d738ccf7057b3f
  log: .bboxy/log/2026-11-02.jsonl#35234
  sig: O2H6WHdcnpFRnFwns4pbXZJ2NGsqSCUDJISnxtv+hmqBrSf6i7UAs0XA220x8asIsY25DvU14iNa1il812ZTAQ==
  signer: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
task: '[[tasks/2026-11-02-harness-standalone-app-release]]'
title: E10 standalone macOS app packaging and release handoff
type: report
updated: 2026-11-02T19:08:40.407Z
updated_by: seat:01M6ESXQ3BJ3PQZ0NTQCA647KY
verify:
  command: make smoke-app && make release-dry
  exit_code: 0
---
Task: [[tasks/2026-11-02-harness-standalone-app-release]] (#5217, 01M6ESXNVR6AA2QKCYGR73V5T3).
Worktree: /Users/feral/mydev/bboxy450-harness-app; branch standalone-app.
Implementation commit: 92ce8684c03769330a5c250491a11540abf90d91, signed by Boxy (bboxy450) <bboxy450@gmail.com> (git signature status G).

## Changes and acceptance evidence

- Makefile:68 builds the locked release binary with standalone and assembles target/Bboxy Harness.app. Makefile:78 stamps Cargo's 0.1.0, git rev-list --count HEAD and BBOXYSourceRevision. Dirty detection covers release inputs, excluding generated run receipts.
- packaging/macos/Info.plist:13 declares com.bboxy450.harness, the launcher executable and bboxy.icns. Makefile:77 includes LICENSE and ATTRIBUTION.md; Makefile:83 lints the final plist.
- packaging/macos/launcher.sh:10 opens gui with --cwd HOME and the default profile (environment, default-profile file, nemotron fallback). Explicit arguments pass through unchanged. tests/app_packaging.py:34 checks precedence, blank values, paths with spaces and CLI passthrough.
- tests/gui_bridge.rs:41 uses the actual live startup boundary to prove an installed profile resolves, a missing profile produces the existing notice, and both sessions close. No production Rust behavior or stateful contract changed; existing formal models and broken controls remain applicable.
- packaging/macos/bboxy.png is copied byte-for-byte from ~/.agents/identity/bboxy.png; source SHA256 bc5cc774612f29a452967a5a4fa993292092a59b58dfd03dfb7ddb6b3286c370. bboxy.icns was derived with term's sips/iconutil recipe; its SHA256 is 64f56a52a0bc39b89496322f7f94105fa6422eed9d1dba399a731961efa412db. ATTRIBUTION.md credits both the artwork and the adapted term packaging recipe.
- Makefile:85 checks exactly term's approved Apple Development identity 3943AF130A015B36824B7FBC86AA672E7B25F3C4 (team UQB3368A84), signs the binary and launcher bundle, and verifies strictly. Missing identity and signing failure stop with no fallback (tests/app_packaging.py:47).
- Makefile:91 signs ad-hoc without invoking security or an Apple private key. scripts/smoke-app.py:24 verifies that bundle strictly; :32 runs the bundled offline replay and validates PNG magic/nonzero length; :40 starts its launcher under an isolated environment, proves it survives three seconds, checks HOME cwd/default profile/the notice and the real transcript, then sends SIGTERM and reaps it within five seconds. No operator input, credentials or Keychain access.
- Makefile:105 uses ditto --sequesterRsrc --keepParent, writes a portable SHA256 manifest, signs the zip with the pinned public fingerprint using gpg --batch, then verifies signature and checksum. release-dry uses ad-hoc Apple signing and performs no tag/push/publication.
- Makefile:119 implements the separately invoked release: approved GitHub identity, clean main, signed HEAD, Apple-signed app, signed zip, signed v<version> tag, push --follow-tags, and gh release create with the three assets. This publishing target was deliberately not run.
- README.md:19 documents standalone download, pinned public key import/fingerprint, signature/checksum verification before extraction/open, default profiles, an explicit project launch and the build/test/release commands.

## Commands actually run

| Command | Exit / evidence |
|---|---|
| cargo test --offline --locked --features standalone,test-fixtures --test gui_bridge standalone_start_resolves_profile_and_missing_profile_keeps_notice -- --nocapture | 0; installed/missing profile startup and Close passed |
| make test-app-packaging | 0; launcher contracts and two signing refusal controls passed |
| cargo fmt --check | 0 |
| python3 -I scripts/checks.py headers | 0 |
| python3 -I scripts/checks.py kb-records | 0; 2853 existing records before this report |
| make app | 0; real Apple Development signature, runtime flag, com.bboxy450.harness requirement, team UQB3368A84; no prompt/input (pre-commit build, log target/e10-preflight/signed-app.log) |
| codesign --verify --strict --deep target/Bboxy\ Harness.app | 0 on both the actual Apple-signed build and an extracted dry-run archive |
| make smoke-app && make release-dry | 0 on signed implementation commit 92ce8684c03769330a5c250491a11540abf90d91; replay exit 0, 246388-byte PNG, live GUI for 3 seconds, checked transcript exit 0, launcher exit -15 (SIGTERM) and reaped, strict signature exit 0, GPG verify and shasum -c both 0 |

The initial preflight exposed Make's whitespace splitting of abspath on the app name; the committed smoke target resolves paths in Python. The startup test's expected text was corrected to the real existing "profile could not be resolved" notice. Neither remains an open failure.

## Artifact receipts

Canonical outputs are target/Bboxy Harness.app and target/release-artifacts/bboxy450-harness-0.1.0-macos-arm64.zip{,.sha256,.asc}.
The exact pre-report artifacts from implementation commit 92ce8684c03769330a5c250491a11540abf90d91 are retained under target/e10-preflight/92ce8684c03769330a5c250491a11540abf90d91/:

| File in that directory | SHA256 |
|---|---|
| bboxy450-harness-0.1.0-macos-arm64.zip | dfeb468f28bc2b5c230e8b87d5610bf82a9ad466a70ea296fdb2d942d296ee20 |
| bboxy450-harness-0.1.0-macos-arm64.zip.sha256 | df8b690ed72a6b64ea2616081b39a9eff14848011271ef96a1887867355f51cb |
| bboxy450-harness-0.1.0-macos-arm64.zip.asc | cd67de742784e761298ad009252cbc6af40d0c2ba8b2e460b808f322a2093526 |

gpg --batch --verify output (exit 0, retained as gpg-verify.log):

```text
gpg: Signature made Mon Nov  2 11:07:35 2026 PST
gpg:                using EDDSA key 9B07E849480C68328CA20684844ECC0587DE7AB0
gpg: Good signature from "Boxy (bboxy450) <bboxy450@gmail.com>" [ultimate]
```

shasum -a 256 -c output: bboxy450-harness-0.1.0-macos-arm64.zip: OK (exit 0).

## Final-commit gate and limits

This append-only report is committed before the required single full final-commit gate:
make test && make smoke-app && make release-dry.
The task_report handoff records that gate's actual exit, final signed commit, PR, logs and the refreshed canonical artifact hashes. Those hashes necessarily change when the report commit changes CFBundleVersion/source revision; the pre-report receipts above refer only to the retained, explicitly versioned directory.

Final gate logs and exit records are retained under target/e10-final-evidence/. make test runs the repository's unit/integration/process/e2e gates, native Yarn/formal models and broken controls, Java TLC graph/verdict parity, GUI rendering, secrets/header/KB checks and packaging controls. Its generated tracked run receipts are preserved there before restoring the run-output files, so the worktree can be handed off cleanly without another source commit.

No change was made to bboxy450-term, another harness worktree, core GUI styling, concurrency models, dependencies, version or credentials. No merge, tag, push of tags, GitHub release or cargo publication was performed. The app is Apple Development signed, not notarized; Gatekeeper download policy and real provider inference are not exercised by this packaging smoke. The live smoke uses SIGTERM with an idle session, not a window-close gesture or an admitted model turn.
