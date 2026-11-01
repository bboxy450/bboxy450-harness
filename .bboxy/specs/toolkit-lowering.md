---
id: "02GK66JGYT5A8W07CXK1JFVQT9"
number: 20
type: spec
title: "Lowering bboxy450-harness onto the bboxy450-term toolkit"
status: draft
created: 2026-10-31
owner: "Vesper Greywind (Oracle); review Nyx Copperwing (Architect)"
links:
  - { to: "[[harness-zoomed-out]]", rel: lowers }
  - { to: "[[bboxy450-term-toolkit]]", rel: evidence }
---
# Lowering onto the bboxy450-term toolkit

Line pointers below are from [[bboxy450-term-toolkit]], taken at bboxy450-term `e624d888`.

## Packaging

- **Repository.** `bboxy450/bboxy450-harness` is public. Its root is a **single package with no `[workspace]`**, and every field is written out (nothing inherited), so a plain clone builds on its own.
- **Inside bboxy450-term.** It is a submodule at `crates/bboxy450-harness` and is listed in `workspace.members`. Membership is what brings it under the layering, brand, coverage, clippy and formal-omission gates. Without membership, `scripts/formal.py:70` would miss it.
- **Toolkit dependencies.** Use crates.io `0.1.1`: `bboxy450-ui`, `-render`, `-doc`, `-rt` and `-yarn`, all verified published on 2026-10-31. bboxy450-term overrides them with local sources via `[patch.crates-io]` path entries, so an embedded build always uses in-tree toolkit code.
- **Features.**
  - `default = []` is the library core: protocol, engine, provider, tools, store, gateway and headless.
  - `standalone` adds the winit window, using the `bboxybook/src/window.rs` pattern.
  - `embedded` exposes `HarnessView -> bbt_ui::Element` for bboxy450-term to host.
  - `acp` adds the ACP stdio face.
- **Layering.** In `scripts/layering.sh:8` it goes after doc/rt/yarn/plugin/term-proto and before `bboxy450-term`.

## Async: `bbt-rt` and `bbt-yarn`

- **I/O.** All I/O runs on `bbt_rt::handle()`. UI crates keep the clippy `disallowed_methods` rules, so there is no `std::fs` and no `Command` on the UI thread.
- **The turn loop is the first product `reactor!`.** Its sources, in order:
  1. `#[shutdown] cancellation(token)`
  2. user steer/interrupt via `mpsc` with `#[yields_to]` over the model stream
  3. the model stream via `mpsc` with `#[drain(max=N)]`
  4. tool completions via `OptionalOneShot` with `cancel_and_replace`
  5. timeouts via `OptionalDeadline`
- **Engine to GUI.** Item deltas go through `Latest<T>` per open item; durable items go through a `ViewRing`/`UiSender` that marks a `DirtySource`. The GUI drains under the 1 ms frame budget (`bbt-rt/README.md`).
- **To add upstream in `bbt-rt`:** a scoped task group (spawn + cancel-all + join), so that no tool or subagent task is ever detached. This lands in bboxy450-term, not in the harness.

## GUI: `bbt-ui`, `bbt-doc`, `bbt-render`

What the harness reuses:

- `chat_scroll.rs` (stick-to-bottom scrolling)
- the `StreamDock` transcript and the pinned composer
- IME
- list virtualization
- `Accordion`
- the `bbt-doc` markdown renderer, which handles tables, images and mermaid

What it needs added upstream. Each item below is a separate upstream task and lands in bboxy450-term, not forked inside the harness:

| Gap | Owner crate | Notes |
|---|---|---|
| Streaming markdown append (re-parse only the tail block) | bbt-doc | stable-prefix cache |
| Markdown inside chat messages | bbt-ui | StreamDock message body = bbt-doc |
| Syntax highlighting hook `Code{info}` → spans | bbt-doc | check the dependency licence (`scripts/deps-licences.sh`) |
| Diff view (unified, hunk fold) | bbt-ui | split view later |
| Tool-call card atom (status, args, collapsible output) | bbt-ui | built on Accordion |
| Editor: Up/Down, word/line motion, undo/redo, history, `/` and `@` completion | bbt-ui `text_edit` | |
| Approval prompt atom (options, keyboard) | bbt-ui | |

## Seat integration with bboxy450-term

- **Phase 1** needs no change to bboxy450-term core.
  - Add an approved profile `bboxy450-harness` to the profile set (which today is `zsh|claude|codex|pi|omp`).
  - The harness reads the `BBOXY_*` env, is an MCP client of its seat socket, and reports state through hooks.
  - It needs one new wake endpoint variant plus an adapter at grade Reported.
- **Phase 2** is an in-process seat or view, and needs a decision superseding d20. Out of scope until phase 1 works.

## Formal verification (mandatory, d365)

- The harness carries its own `verification/coverage.toml` and contract registry, plus the pinned TLC parity target.
- Inside bboxy450-term, the harness is listed in bboxy450-term's `verification/{coverage,contracts,corpus}.toml`.
- Models come first, each with a broken control:
  - AgentTurnLifecycle
  - CancelTree
  - ApprovalLifecycle
  - SubscriberReplay
  - StreamToView
