# Respectful Slopping — Source Attribution and Reuse

Project: **bboxy450-harness**, a GUI-first coding-agent harness for the bboxy450-term suite.
Canonical source: https://github.com/bboxy450/bboxy450-harness
Copyright 2026 Boxy (bboxy450) and contributors, for their respective contributions.

Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE

**Slop respectfully. Keep the credit attached.**

## Terms

Project-owned contributions are under the Respectful Slop License 1.0-Draft
(`LicenseRef-Respectful-Slop-1.0-Draft`, see `LICENSE`). When you reuse material from this
project, credit it accurately as one of the following:

- **Copied from**
- **Adapted from**
- **Translated from**
- **Inspired by**

Each credit names **bboxy450-harness**, this canonical URL, and the affected file, symbol or block.
Independent inspiration credit is a request, not a condition.

## Keep credit beside the material

Every source file starts with this header:

```
// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
```

A distinctive implementation adapted from elsewhere carries its own `// Adapted from <project>: <url>` line beside it.
The source-header rule includes test fixtures and executable formal models;
keep the credit when extracting a regression witness or broken control.
Native text-entry dispatch witnesses and their broken controls follow this policy too.

## Lineage and credits

- ACP full-access defaults, per-adapter launch settings and confirmation before
  prompting in `src/backend/acp_permissions.rs` and their private process
  witnesses are project-owned under the policy above. Adapter command/API facts
  follow each provider's published interface; no provider implementation is copied.

- Centered setup, loader/welcome routing, independent view scrolls and Goose ACP
  discovery are project-owned contributions; existing toolkit and portrait assets
  retain their origin. Goose command facts come from the official ACP registry
  and Goose documentation; no upstream implementation was copied.

- Conservative Python command extraction, concise ACP heading projection and
  bounded native text producers in `src/frontends/gui/` are project-owned.
  Python colors and code blocks reuse bboxy450-term's existing `bbt-doc`
  Markdown renderer and syntect-backed `RustHighlighter`; no syntax engine was copied.
  The focused Unicode/source/geometry/MCP raster witnesses and gate retain
  this policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Bounded command/heredoc/output panes, view-local two-axis scrolling and native
  geometry and raster broken-control witnesses in the shared GUI are project-owned implementations. Keep
  this policy with reuse: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.


- Advertised ACP model/reasoning controls, child/revision fences, safe catalogue
  diagnostics and their private process/native/formal witnesses are project-owned.
  Wire shapes follow the ACP v1 session configuration specification:
  https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/protocol/v1/session-config-options.mdx.


- MCP result presentation, local causal pending-prompt bubbles, shared welcome-loader
  portrait and their focused native/formal witnesses in `src/frontends/gui/`,
  `src/frontends/face_core.rs`, `src/backend/acp_agent.rs`, `tests/` and
  `scripts/verify-gui-polish.py` are project-owned under the policy above.

- Launch bootstrap context, generation-private MCP stdio relays, process fixtures
  and formal controls in `src/backend/acp_agent.rs`, `src/mcp/relay.rs` and
  `tests/acp_launch_context.rs` are project-owned implementations under this policy.
  Bounded startup-order evidence and independent private-proof control,
  persistence and terminal-cost verdicts retain the same project origin and policy.


- The hidden native startup process witness and missing-title-bar-image control
  in `src/frontends/gui/window.rs` and `scripts/` are project-owned; they drive
  the shared native window, renderer and detach path without desktop input.

- Full-window captures reuse the live standalone chrome tree and text zoom in
  `src/frontends/gui/`; face-only captures keep the parity baseline comparable.
  **Adapted from bboxy450-term** `lab/web/harness.html`'s standalone-window state,
  the `CHROME=1` web composition and CSS zoom in `scripts/parity-web.sh` use
  `C.titleBar` and `Hx.view`. Upstream policy:
  https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md.

- Native-record suppression and quiet transcript notices in the GUI and seat
  are project-owned implementations of bboxy450-term's `lab/web/harness-spec.md`
  "What never reaches the transcript" (d68f505e), with replay-equivalence witnesses
  and production mutation controls under this policy.

- Seat socket lifecycle, deterministic message operation IDs and the shared
  InProc/Unix rich-view bridge in `src/frontends/` and `src/gateway/` are
  project-owned implementations of `.bboxy/interfaces/rich-pane.md`.
  The shared round-robin frame budget and executable production-guard control
  follow the same project-owned policy.

- Capture expansion in `src/frontends/gui/smoke.rs` uses the click-controlled
  face state; `scripts/parity-web.sh` invokes the bboxy450-term web reference's
  own disclosure handlers for its matching expanded view.

- Markdown tool-output detection, numbered Read projection and inset scrolling in
  `src/frontends/gui/` are project-owned implementations of the bboxy450-term
  `lab/web/harness-spec.md` Markdown-everywhere contract, using its `bbt_doc` renderer.

- Native-log usage accounting and blob previews in `src/import/` are project-owned
  implementations under the license and attribution policy above.

- **Inspired by Gonzih/kittens** (https://github.com/Gonzih/kittens), the predecessor Rust harness by the same authors. Ideas carried over:
  - the turn-end guarantee
  - tool-call pairing
  - the cancellation tree
  - `!cmd` key profiles
  - ACP over one event stream
- **Inspired by** these projects. The design survey copied no code from them; evidence is in `.bboxy/research/sota-harnesses.md`.
  - openai/codex
  - xai-org/grok-build
  - badlogic/pi-mono
  - oh-my-pi
  - sst/opencode
  - block/goose
  - Aider
  - the Zed Agent Client Protocol
  - the Model Context Protocol
- **Built on** the bboxy450-term toolkit crates (https://github.com/bboxy450/bboxy450-term), used as dependencies under their own terms. The standalone UI and native Mermaid layout share revision [775527db](https://github.com/bboxy450/bboxy450-term/commit/775527db16ffe33ee5cafaa0ff8566e3ea7512da); embedded builds use term's workspace crates.
- **Inspired by bboxy450-term** OSC 7501 seat observation grammar and text limits
  (`crates/bboxy450-harness-wake-up/src/observe.rs`); the harness emits compatible
  state reports. https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md.
  The origin-independent open-turn heartbeat, shared transcript tool heads,
  immediate phase projection and their fake-clock/process/formal
  witnesses in `src/frontends/seat.rs`, `src/frontends/headless.rs` and `tests/`
  are project-owned implementations under this policy.
- **Adapted from bboxy450-term** (https://github.com/bboxy450/bboxy450-term),
  `Makefile` bundle/signing/install/archive recipes and `packaging/macos/Info.plist`.
  Upstream reuse policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md.
- **Copied from bboxy450's identity artwork**, `~/.agents/identity/bboxy.png`
  (canonical source identifier: `bboxy450/dotagents:identity/bboxy.png`), as
  `packaging/macos/bboxy.png`. SHA256:
  `bc5cc774612f29a452967a5a4fa993292092a59b58dfd03dfb7ddb6b3286c370`.
  `packaging/macos/bboxy.icns` is adapted from that exact image using macOS
  `sips` and `iconutil`; no replacement artwork was generated. Keep this credit
  and the project's absolute attribution policy URL with reuse of the icon.

- Shared composer attachment custody, workspace completion, async ownership fences and private native/ACP/formal witnesses are project-owned; retain this policy with extracted implementations.

Local harness discovery, create-new editable profiles, fixed-argv setup installation
and shared user Markdown bubbles are project-authored contributions. Adapter
commands and recipes are factual contracts reviewed from the ACP registry,
https://github.com/agentclientprotocol/registry, and the linked adapter projects;
no registry code or dynamically downloaded installer command is embedded.

- Shared clean-start profile selection and private CLI/GUI witnesses are project-owned; retain the canonical origin and policy above with extracts.

## Shared suite license verification

The license text is byte-identical to bboxy450-term's Respectful Slop License
1.0-Draft. `make test-licenses` checks the entire license, local Cargo metadata,
source-attached credit, preserved prior terms and deliberate broken controls.
`make check-license-packages` checks Cargo's real package file lists where present.

Copied from bboxy450-term, with original source notices retained:
- https://github.com/bboxy450/bboxy450-term/blob/main/scripts/suite-licenses.py
- https://github.com/bboxy450/bboxy450-term/blob/main/scripts/test-suite-licenses.py

The accompanying `licensing.json` records source scope, preserved-file hashes
and the exact shared checker copies. These checks do not replace upstream terms.
- **Built on hashbrown 0.17.1** (https://github.com/rust-lang/hashbrown),
  used through its safe `HashTable` API for the private provenance record cache
  in `src/dream/resolve.rs`. Upstream is licensed **MIT OR Apache-2.0**;
  this is dependency/API use, with no copied upstream implementation. Its own
  origin and license notices retain their original terms.

- Dream C1 wire authority, canonical schema/hash corpus and isolated inspection worker
  are project-owned implementations; keep this policy with extracted transport,
  custody, scheduling, declared storage-credit accounting, incremental typed-result
  teardown and paced settlement
  witnesses. The credit model does not certify opaque native allocation. The split normalization masks
  retain this project's original expression as an independent regression oracle.

- C1 parked compute-owner provisioning, exact native-call/effect accounting,
  monotonic post-span rest debt, bounded private metrics and the paired runtime
  and C1QuantumDebt causal witnesses are project-owned. Keep the canonical
  source and this absolute policy URL with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

## Private Dream proof-tool provisioning

The recipe, byte/custody validators, regression witnesses and finite models in
`scripts/dream-proof-tools.py`, `tests/dream_proof_tools.rs` and
`tests/formal/dream_proof_tools.rs` are project-owned under this policy.

**Adapted from Git 2.54.0** (https://github.com/git/git/tree/v2.54.0),
**GnuPG 2.5.24**, **Libgpg-error 1.61**, **Libassuan 3.0.2** and **nPth 1.8**
(https://gnupg.org/): the private source guards in
`tests/fixtures/dream-proof-tools/*.patch` retain their original upstream
copyright and GPL/LGPL terms. They are not relicensed under the project license.
The adjacent `license-map.json` maps every patch and release to its upstream
terms and preserved license files. Libgcrypt 1.12.4 is a source dependency;
its original LGPL terms and bundled licenses remain attached.

**Built on Libksba1.8.1**, https://gnupg.org/ftp/gcrypt/libksba/, as the explicitly
pinned seventh source. Its library retains LGPL-3.0-or-later OR GPL-2.0-or-later
terms and original notices; other components retain their per-file licenses.
Its source signatures, license-map entries and actual linked/build-only evidence
are provenance, not runtime authority.

Public maintainer certificates and detached release signatures in that fixture
directory are **Copied from** the exact primary URLs in `sources.lock.json`,
with bytes preserved. They identify public release material, not operator keys
or runtime approval. Generated archives, source trees, scratch executables and
build logs remain private outside Git. Source-built does not mean qualified;
the current partial build and frozen Git component remain unsupported.

- The private dream-bootstrap/1 fixed permit, bounded earliest-entry/lease checks,
  canonical binding/layout vectors, independent checker and causal process/formal
  witnesses are project-owned. Keep this origin and policy beside extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- The ordinary fixed Dream bootstrap entry record, stack-only raw-counter
  sampling/formatting, finite entry-order/usage models and causal private process
  controls are project-owned. Preserve this origin and credit policy with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Bounded Python result waits, optional progress, live nested MCP catalogues,
  unknown-effect call deadlines, structured worker failures, lean byte output
  and linked command-style execution cards are project-owned implementations.
  Keep this source and policy with extracted runtime, rendering and formal witnesses:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Dream prepared-read DTOs, selected immutable evidence bindings, shared C1 owner
  adapters and private publication/process/formal conformance witnesses are
  project-owned. Retain this origin and the absolute policy URL with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Signature temporary scratch settlement, strict private input controls, paced
  source hashing, helper tail accounting and their source/model mutants are
  project-owned. Keep this policy with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- The shared Dream closure decoder, offline vector/context runner, signed C preparation input snapshot and terminal-delivery regression witnesses are project-owned. The schema/vector bytes are copied from the signed builder authority b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9 in this same project; retain their origin and this policy.

- Dream private command/result completion and independently observed failed-STOP
  settlement, including their source-known controls and finite parity witnesses,
  are project-owned. The fixed public presigned corpus relations are adapted
  from Git's `t/t7510-signed-commit.sh`; Git retains its own origin and license.
  Keep this policy with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Dream's same-controller quarantine custody, refusal wire, owner-loss record
  and live FD/child settlement controls are project-owned. Preserve this origin
  and policy with extracts:
  https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md.

- Dream's in-process exec shim, exact command/environment receipts, SCM_RIGHTS
  signature-inode custody and conditional supported-tool budget model are
  project-owned. Retain the canonical origin and absolute policy URL with extracts.
