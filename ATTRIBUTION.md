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

## Lineage and credits

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
- **Built on** the bboxy450-term toolkit crates (https://github.com/bboxy450/bboxy450-term), used as dependencies under their own terms.
