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
