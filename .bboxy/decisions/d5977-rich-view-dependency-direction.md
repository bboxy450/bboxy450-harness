---
id: 01M6F5C7W3D43GKXPWGDX677HN
number: 5977
type: decision
title: Term links the harness crate for the rich pane now; the IR and the transcript view move into the term workspace after the parity pass
status: accepted
links:
  - { to: "[[rich-pane]]", rel: constrains }
  - { to: "[[log-ir-v1]]", rel: constrains }
  - { to: "[[d5087-log-converter-in-term]]", rel: builds-on }
  - { to: "[[upstream-toolkit]]", rel: builds-on }
created: 2026-11-02T23:45:00Z
updated: 2026-11-02T23:45:00Z
---
**Date.** 2026-11-02. **Recorded by** Lumen Foxglove <ARCHITECT>.

**Problem.** The harness consumes term's toolkit (`bbt-ui`, `bbt-render`, …) by git pin. [[rich-pane]] has term render the harness's `HarnessView`, so term would consume the harness by git pin too. That is a cycle between repositories: term → harness → toolkit-at-rev-X, while term is at rev Y. `[patch]` resolves it to one copy (R15), at the price of a lockstep: every toolkit change in term must build the pinned harness, and every harness pin bump must match term's toolkit. A lockstep between two repositories is a chokepoint for the swarm.

**Decision.**
1. **Phase 1 (now):** term links `bboxy450-harness` with `default-features = false, features = ["embedded"]`, git-pinned, with `[patch]` entries that force the toolkit crates to term's in-tree paths. R15 (`cargo tree -d … | grep -c bboxy450-` = 0) is the gate. Executors on the parity pass keep working in the harness's `src/frontends/gui/`.
2. **Exit, triggered when the parity pass is accepted (R27 `--target` exit 0):** two crates move into the term workspace under d5087, and the harness consumes them by pin the way it consumes `bbt-ui`:
   - `bboxy450-log-ir`: `src/protocol/` (types, laws, schema export), `protocol.schema.json`, `verification/protocol-v1.lock.json`, `tests/fixtures/ir-v1/`, `scripts/ir-compat.py` and `make test-ir`. The gate runs in term's `make test`; the harness keeps a re-export (`crate::protocol`) so call sites do not change.
   - `bboxy450-transcript`: the log-IR → `Element` projection (`gui/view.rs`, `face.rs`, `face_rows.rs`, `rich.rs`, `chat_scroll.rs`, `editor.rs`, the transcript part of `model.rs`), taking a read-only frame state. The `Bridge` (tokio, gateway, mailbox) stays in the harness.
   After the move, term's rich pane depends on in-tree crates only; the `[patch]` block and the harness pin leave term's `Cargo.toml`.
3. If the operator instead folds the harness repository into bboxy450-term (d5087: "everything is a crate in there"), the cycle disappears without step 2's split; the IR crate boundary is still drawn, inside one workspace. That is the operator's call, not the Architect's.

**Alternatives rejected.**
- *Move first, pane later.* It stalls the parity executors mid-flight and delays the pane the operator asked for.
- *Keep the pin forever.* The lockstep lands on every toolkit change; the swarm rule is no chokepoints.
- *Term re-implements the transcript view from its `composer`/`conversation` modules.* Two renderers of one IR drift; R27 exists to prevent exactly that.

**Verification.** Phase 1: R15 in term. Exit: `make test-ir` green in term over the moved lock and corpus; the harness `make test` green on the pin bump; `tests/gui_fixtures.rs` goldens unchanged across the move (same `layout_text` per fixture).
