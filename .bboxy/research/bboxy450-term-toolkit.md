---
type: research
title: "---"
created: 2026-10-31
---
---
id: "01M6CHRS0TKM00LK1TMAP0001"
type: "research"
title: "bboxy450-term toolkit map for bboxy450-harness"
status: "reported"
source: "https://github.com/bboxy450/bboxy450-term @ e624d888 (main, 2026-10-31)"
links:
  - { to: "https://github.com/bboxy450/bboxy450-term", rel: reference }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# bboxy450-term toolkit map for bboxy450-harness

All paths are relative to `/Users/feral/mydev/bboxy450-term`, at commit `e624d888`.
The format is `file:line — mechanism`. "BT" means bboxy450-term.

Read first:
- `AGENTS.md`: the Makefile is canonical; formal verification is mandatory; one worktree per Executor.
- `~/.agents/AGENTS.md`

## 1. Workspace map

The root `Cargo.toml` has resolver 3, edition 2024 and rust-version 1.99. `[workspace.dependencies]` sets version 0.1.1 for each in-repo crate and uses path deps. Package names differ from directory names: directory `bbt-ui` publishes as `bboxy450-ui`.

The required layer order is the `ORDER` line in `scripts/layering.sh:8`. Crates appear below in that order.

| Dir → package | Purpose | Key deps |
|---|---|---|
| `bboxy450-keychain{,-file,-windows}` | File custody for identity keys | getrandom, zeroize, windows-sys |
| `bboxy450-desktop-notifications{,-macos,-linux,-windows}` | OS notifications with callbacks | objc2-user-notifications |
| `bboxy450-tlc` | Pure Rust explicit-state model checker; exports TLA+ | none (std only) |
| `bbt-yarn-macros` → `bboxy450-yarn-macros` | `reactor!` / `reactor_model!` parser and validator | syn, quote |
| `bbt-yarn` → `bboxy450-yarn` | `no_std` reactor kernel: persistent sources, compile-checked arbitration | tokio =1.54.1 (opt), tokio-util, tlc (feature `model`) |
| `bbt-rt` → `bboxy450-rt` | Shared 2-worker Tokio IO runtime; non-blocking UI handoff | tokio, crossbeam-queue |
| `bbt-core` → `bboxy450-term-core` | Forked alacritty/vte terminal model | parking_lot, unicode-width |
| `bboxy450-harness-wake-up` | Harness launch/observe/wake protocols for claude, codex, zsh | tokio, tokio-tungstenite, mio |
| `bbt-mermaid` → `bboxy450-mermaid` | Mermaid parse and layout to geometry (flowchart, sequence, state) | none |
| `bboxy-firehose` → `bboxy450-term-firehose` | Synthetic PTY load scenes | libc |
| `bbt-ui-model` → `bboxy450-ui-core` | DrawList/prims, layout solve, events, focus, generated design tokens | toml (build) |
| `bbt-render` → `bboxy450-render` | wgpu/Metal retained renderer, CoreText shaping, atlases | wgpu =30.0.1, lyon, resvg, objc2-core-text |
| `bbt-ui` → `bboxy450-ui` | Element tree, widgets, atoms, modules (composer, stream_dock, ...) | ui-core |
| `bbt-doc` → `bboxy450-doc` | Markdown → `Element` | pulldown-cmark 0.13, mermaid |
| `bboxybook` → `bboxy450-book` | Standalone component gallery window and parity tools | winit 0.30, wgpu, pollster |
| `bbt-supervisor` → `bboxy450-harness-host` | PTY supervisor actor, capture rings, durability; no Tokio | mio, portable-pty, zstd |
| `bbt-plugin` → `bboxy450-plugin` | Plugin contract types | jsonschema |
| `bbt-plugin-sdk` → `bboxy450-term-plugin-sdk` | Rust stdio-MCP plugin runtime and testkit | serde_json |
| `bbt-plugin-fixture`, `plugins/gitkb` | Test plugins; GitKB data_source plugin | plugin-sdk |
| `bbt-proto` → `bboxy450-term-proto` | Wire types: seats, registry, messaging, GUI | core, ui-core, plugin |
| `bbt-mux` → `bboxy450-term-mux` | Logic: registry, bus, messaging, MCP, adapters, providers, store | rusqlite, tokio, pgp, ed25519, notify |
| `bboxy450-term-desktop{,-core,-macos}` | AppKit desktop facade and lifecycle contracts | objc2-app-kit, raw-window-handle |
| `bboxy450-term` | The app: winit, GPU, scene, ctl, MCP bridge (about 70k lines) | everything above, wgpu, winit, objc2-metal |

- `lab/web` holds the HTML/JS reference lab (`components.html`, `tokens`), which drives native parity (`.bboxy/runbook-web-to-native.md`).
- There are no submodules: `.gitmodules` is absent.
- Worktrees are sibling directories `../bboxy450-term-<topic>`, plus temporary ones in scratchpads (`git worktree list`).

## 2. GUI library

**Backend**
- `crates/bbt-render/Cargo.toml` uses wgpu `=30.0.1` with the `metal` and `wgsl` features only.
- `crates/bbt-render/src/device.rs:10` `DeviceContext{device,queue,adapter_info}`.
- `crates/bbt-render/src/device.rs:18` rejects any target other than macOS + Metal. The GUI is currently **macOS-only**.
- `crates/bbt-render/src/chrome/mod.rs:44` `ChromeRenderer` is the UI renderer. It does one instanced pass on the caller's device and owns no window.
  - `:28` `ChromeTarget{width,height,scale,view,format}`
  - `:67` `add_image`, `:68` `begin_frame`, `:73` `flush`
  - `:126` `draw`, `:140` `draw_overlay`, `:151` `draw_clearing`
  - `:799` `render_to_rgba`, used for headless/golden tests
- `crates/bbt-render/src/headless.rs:43` `HeadlessRenderer` (`:407` `render_to_rgba`, `:417` `write_png`) renders the terminal grid.
- `crates/bboxy450-term/src/headless.rs:57` runs the whole app with no window. Smoke tests use this.

**Frame loop**
- `crates/bboxy450-term/src/display_link.rs:62` `Mode{Fifo,Metal,Cv}`; `:205` `DisplayLink` on `CAMetalDisplayLink`; `:313` `request_frame`.
- `crates/bboxy450-term/src/gpu.rs:246` `Gpu`.
- `crates/bboxy450-term/src/gpu.rs:1203-1215` sends `Paint::Draw` to `draw_clearing` / `draw_overlay`.
- `crates/bboxy450-term/src/main.rs:7806` `redraw()`.

**Model: retained, rebuilt from data**
1. The app builds an `Element` tree.
2. `Element::layout` (`crates/bbt-ui/src/components.rs:1107`) produces `Layout` (`:609`).
3. `paint` (`:2231`) produces `Vec<Paint>` (`:2181` `Paint{Group,Draw,Underlay,Material}`).
4. That becomes a `DrawList`.

`crates/bboxy450-term/src/conversation_cache.rs:7` reuses the solved tree when its inputs are unchanged.

**Primitives (bbt-ui-model)**
- `src/draw/list.rs:12` `DrawList`
- `src/draw/prim.rs:207` `Prim{Path,PathShadow,Group,Rect,Hairline,Gloss,Text,TextShadow,Icon,Image,Clip,Unclip}`
- `src/draw/ctx.rs:26` `DrawCtx::new(Theme,Density)`
- `src/layout/solve.rs:29` `trait TextMeasure`; `:60` `solve_box`
- `src/event.rs:19` `Event`; `:97` `Update`
- `src/input/mod.rs`: `Focus`, `KeyRoute`, `Modifiers`

**Widgets (bbt-ui)**
- `src/components.rs:466` `enum Element{FixedHeight,Filter,InlineMiddle,Baseline,Named,Layers,Glyph,Box,Text,EditingText,Stack,Flow,ScrollView,Overlay,Spacer,Icon,Image,ImageRect,Path,Rects}`
- `src/components.rs:40` `trait Measure: TextMeasure` must be implemented by the host.
  - Real implementation: `crates/bboxy450-term/src/scene.rs:333` `Metrics`, which calls `bbt_render::chrome::text::measure`.
  - Stub: `crates/bboxybook/src/components.rs:30`.
- `src/style.rs:501` `Style::new(Preset)`; `:72` `Material`; `:649` `Context`
- `src/widgets.rs`: `:99` `Button`, `:205` `Viewport`, `:218` `row_window()` (virtualization), `:288` `List`, `:396` `Table`, `:543` `Input`
- `src/widgets/text_area.rs:12` `TextArea`
- `src/atoms.rs:74` `trait Atom`: Badge `:233`, Chip `:268`, Pill `:300`, Spinner `:368`
- `src/overlays.rs:12` `Modal`, plus `overlays/{context_menu,dialog}.rs`
- `src/modules/mod.rs:8-28`: accordion, composer, stream_dock, document_modal, graph_tab, pane, settings, sidebar, launcher, swarm_card, tab_row, title_bar
- Invariant `.bboxy/invariants/modules-through-components.md`: modules may not hand-draw rows.

**Text**
- `crates/bbt-render/src/chrome/text.rs:7` uses CoreText for shaping and fallback; `:64` `measure`, `:245` `rasterize`; `:69` errors on non-mac.
- `chrome/text_cache.rs:7-9` caches glyphs and shaped lines (contract RenderLinePlan).
- `src/atlas.rs:18` `GlyphAtlas` (mask/BGRA).
- There is no in-house shaper.

**Rich content (bbt-doc)**
- `src/markdown/parse.rs:8-16` runs pulldown-cmark with tables, tasklists, strikethrough, footnotes and wikilinks.
- `src/markdown/mod.rs:60` `BlockKind{Paragraph,Heading,List,Quote,Code{info,text},Table,Footnote,Rule,Html}`; `:90` `Inline{…,Image}`.
- `src/markdown/layout.rs:221` `element(ctx,m,width)`; `:254` `body_element`; `:337` `hosted_body_element_with_controller`.
- Code blocks: `layout.rs:1139` renders monospace text in a horizontal `ScrollView` with **no highlighting**. No syntect or tree-sitter anywhere in the workspace.
- Tables: `layout.rs:730`, `:1184`.
- Images: `Inline::Image{ImageAsset}` (`mod.rs:112`) becomes `Element::ImageRect` (`layout.rs:907`).
  - The host decodes PNG (`crates/bboxy450-term/src/documents/images.rs:39`).
  - It installs them with `Gpu::install_document_images` (`gpu.rs:329`), which calls `add_image`.
- Mermaid: `layout.rs:626` hands mermaid code blocks to `bbt-doc/src/mermaid.rs:89`.

**Chat views**
- `crates/bboxy450-term/src/chat_scroll.rs:7` provides scoped tail-following: `WheelIntent :18`, `State :66`, `bind :74`, `following :115`, `wheel :138`, `follow_latest :157`, `MAX_VIEWS = 128`. Layout never overwrites a manual offset.
- `crates/bbt-ui/src/modules/stream_dock.rs:210` `StreamDock{entries}`.
  - `:173` `Entry{Message,Epic,Task,Report,External,Tabs,Filter,Note}`
  - `:20` `Message` (author, role, time, text, receipts, attachments, reply_to)
  - `:1441` `items_at` (windowed)
  - `:1644` `element`
- `crates/bbt-ui/src/modules/composer/conversation.rs:49` `Conversation{stream,composer,scroll,tasks,task_scroll}` keeps the composer pinned at the foot.
  - Hosted by `crates/bboxy450-term/src/scene.rs:99` `ConversationLayout` and `sidebar_chat.rs`.
- Virtualization: `Element::FixedHeight` (`components.rs:469`), `row_window`, and `ScrollAnchor` (`:423`, with `anchor :1660` and `restore_anchor :1678`).
- Related commits:
  - 3fbe71f4: scoped scroll intent
  - a4658316: conversation layout reuse
  - eb73589b: transcript drag selection
  - f29c6fec: Enter sends
  - d92ed496: bounded recent ledger read
- The sibling worktrees `../bboxy450-term-chat-scroll` and `../bboxy450-term-chat-visual` hold that history.

**Input**
- `crates/bboxy450-term/src/keyboard.rs:7` is the winit boundary: `:30` `Input`, `:255` `encode` (PTY bytes).
- Routing: `main.rs:1555` `route_local_page`; `main.rs:1174` `RoutedKey`.
- IME: `main.rs:9034` turns `WindowEvent::Ime` into `ime_input` (`:6939`); see also `app_overlays.rs:743-1113`.
- `crates/bbt-ui/src/text_edit.rs:41` `Editor` uses UTF-16 offsets.
  - `:20` `Command{Insert,Commit,Delete,Move,SelectAll,SetCaret,Preedit,CancelPreedit}`
  - `:12` `Motion{Left,Right,Start,End}`: no Up/Down/word movement and **no undo**
- `src/text_selection.rs:87` `Selection` selects read-only retained text.
- `src/components.rs:155` `Edit{caret,select,preedit}`.
- Composer: `src/modules/composer.rs:145` `Controller`, with `:113` `Command` (`Edit`, `Attach`, `Priority`), `:103` `Submission`, and the view at `composer/view.rs:540`.
- `crates/bboxy450-term/src/composer_images.rs:27` `OwnedImage` handles pasted images.

**Theming**
- `crates/bbt-ui-model/build.rs:152-158` generates `OUT_DIR/tokens.rs` from `design/tokens.toml` and `design/style.json`.
- `src/tokens.rs:10` exposes `TOKENS`, `STYLE`, `Theme :15`, `Density :31` (touch, comfortable, cozy, compact).
- The web mirror is `lab/web/tokens.*`.

**Hosting a standalone window**

`crates/bboxybook/src/window.rs` is the minimal template:
- `:297` `impl ApplicationHandler` (winit 0.30.13)
- `:57` `init` creates the `wgpu::Surface` and `ChromeRenderer::with_context` (`:96`)
- `:250-284` per-frame sequence: get the texture, `begin_frame`, `draw(&DrawList, ChromeTarget)`, `flush`, `submit`, `present`
- `:420` `EventLoop::new`

**Hosting inside BT**
- **Panes are terminal-only.**
  - `crates/bbt-ui-model/src/model/split.rs:53` `SplitNode<Id=SeatId>{Leaf(SeatId),Split}`
  - `model/tab.rs:10` `Tab{tree}`
  - `crates/bbt-ui/src/modules/pane.rs:7` renders terminal spans
- **There is no pane-content trait.** Non-PTY views use one of two precedents:
  1. **Local page** (best fit for a full-area chat). `crates/bboxy450-term/src/scene.rs:130` `LocalPageLayout{layout,viewport,close,max_scroll,settings}` replaces the stage beside the sidebar. Examples: `graph_page_layout :1663` and the settings page. Keys arrive as `RoutedKey::LocalPage(PageCommand)`.
  2. **Modal overlay.** `scene.rs:2287` `document_layout()` uses `DocumentModal` (`crates/bbt-ui/src/modules/document_modal.rs:89`; controller `:232`).
     - Host: `crates/bboxy450-term/src/documents.rs:293` `Host`, with `Reader :190` and `Work::prepare :203` preparing on the IO thread.
     - Painting: `main.rs:7363` `document_paint`, then `gpu.rs:1205` `draw_overlay`.

**Gaps for a rich agent UI**

| Feature | State | Pointer / note |
|---|---|---|
| Diff view | MISSING | — |
| Syntax-highlighted code | PARTIAL | monospace only, `layout.rs:1139` |
| Tool-call cards | PARTIAL | patterns: `stream_dock::Entry::{Task,Report,External}`, `composer/report.rs:30` |
| Tables | EXISTS | — |
| Streaming text | PARTIAL | tail-follow and layout reuse exist; no append-delta API; no incremental markdown |
| Markdown in chat messages | MISSING | `Message.text` is plain `Text` |
| Collapsible sections | PARTIAL | `modules/accordion.rs:16`, `:137` `Reveal` |
| Multi-line editor | PARTIAL | `TextArea` + `Editor`; lacks vertical/word motion, undo, history |
| Selection/copy | EXISTS | `text_selection.rs` |
| Inline images | EXISTS | — |
| Mermaid | EXISTS | — |
| Virtualization | EXISTS | — |
| Cross-platform | MISSING | Metal/CoreText only |

## 3. Async library

There is **no custom executor**. Two layers sit on Tokio.

**bbt-rt (`bboxy450-rt`): the runtime and the UI handoff**
- `crates/bbt-rt/src/lib.rs:51` `handle()` is a process-wide `OnceLock` multi-thread Tokio runtime with **2 workers**, named `bbt-io`.
- Startup: `lib.rs:121` `init_ui(waker) -> UiInbox{sender,messages,output,dirty}` (`:104`). It registers the UI thread (`:68`).
  - Dirty source 0 is messages; source 1 is command output.
- `lib.rs:89` `assert_not_ui_thread()`: in debug builds, blocking IO on the UI thread panics; `ui_blocking_violations()` (`:98`) counts these.
- `src/queues.rs`:
  - `:49` `ui_channel(cap)`; `:65` `try_send` returns the message on a full queue and never waits.
  - `:96` `Latest<T>` is a replace-unread slot (`publish`/`take`/`complete_drain`).
  - `:136` `ViewRing<T>` drops the oldest entry and counts drops.
  - `:18` `UiMsg`.
- `src/wake.rs:13` `trait Waker` (event-loop proxy); `:38` `DirtySet` admits one wake until drained, with the recheck covered by contract DirtyWake.
- `src/commands.rs:272` `spawn_command(argv,cwd,env)->TaskId` and `:280` `cancel` provide async child processes, chunks of at most 16 KiB, and `UiMsg::Exited`.
- `src/error_journal.rs:191` `report(Failure)`.
- `src/diagnostics.rs` provides spans and probes; `meter.rs` provides `HitchMeter`.
- Feature `runtime` is on by default. `--no-default-features` gives a std-only registry and guards, used by the supervisor.
- `crates/bbt-rt/README.md` sets the convention: drain dirty bits at frame start, then drain messages under a **1 ms budget**.

**Enforcement**
- `clippy.toml` disallows `std::fs::*` and `std::process::Command::{spawn,status,output}`, with the reason "UI must use bbt-rt".
- `scripts/ui-lint.sh` runs `clippy -D clippy::disallowed_methods` on ui-core, ui, render and term.
- Invariants `.bboxy/invariants/{ui-never-waits,never-block-swarm,messaging-never-held,no-chokepoints}.md`.

**bbt-yarn (`bboxy450-yarn`): structured reactor loops** (see section 4 for the model side)
- `crates/bbt-yarn/README.md` describes `reactor!{ policy{selection: biased; required_phases:[…]} … }`.
  - Relations: `#[shutdown]`, `#[terminal]`, `#[before]`, `#[last]`, `#[when]`, `#[yields_to(...,when=buffered)]`, `#[drain(max=N)]`, `#[starvation(allowed,reason)]`.
- `src/source/mod.rs:194` `Latched<T>`, `:271` `FixedQueue<T,N>` (no_std).
- Tokio adapters in `src/source/tokio_impl.rs`:
  - `:98` `mpsc(rx, close::Emit|Dormant)`, `:202` `OptionalMpsc`
  - `:310` `one_shot(fut)`, `:341` `OptionalOneShot` (`cancel_and_replace`)
  - `:434` `cancellation(CancellationToken)`
  - `:470` `OptionalDeadline` (timers)
- `src/reactor.rs:8` `Control{Continue,Stop}`.
- The runtime crate is `forbid(unsafe_code)` and `no_std`. Readiness is a sealed trait, and arbitration never preempts.
- **No product crate consumes `bbt_yarn` yet.** Its only users are its own tests and examples. bboxy450-harness would be the first real reactor consumer.
- Missing:
  - no actor framework
  - no structured task-scope/nursery type, beyond `CancellationToken` plus `OptionalOneShot`
  - backpressure is by bounded queue only; `try_send` hands the message back

**GUI ↔ async seam**
- IO tasks run on `bbt_rt::handle()`.
- Results go back to the UI through `UiSender::try_send`, `Latest::publish` or `ViewRing::push`, each of which marks a `DirtySource`; the `Waker` then wakes winit.
- The UI drains on the next frame under its budget.
- The precedent for preparation on the IO thread is `documents.rs:203` `Work::prepare`, guarded by contract AsyncRenderPreparation: a result arriving after its waiter is dropped is retired.

## 4. Yarn / formal verification

**Sources**
- `verification/README.md`.
- `.bboxy/decisions/d365-mandatory-formal-verification.md` (number 365, decided 2026-10-29); it builds on `d33-yarn-macro`.

**Pipeline**
- `reactor_model!` reuses the `reactor!` grammar to produce a finite model.
- `crates/bboxy450-tlc/src/lib.rs` defines the model API: `Model::new :301`, `Variable::new(name,min,max,init) :205`, `Action::new(name,guard,updates) :226`, `Expr` ops, `check(max_states) :314`, `tla() :541`.
- The native checker runs beside pinned Java TLC (`verification/tools.lock.json`). The gate requires identical reachable states and action-labelled edges, plus matching verdicts.

**Project models**
- Defined as plain Rust in `crates/bboxy450-term/tests/formal_contracts/models.rs`. Examples: `fn model(name,broken,vars) :33`, `fn dirty_wake(b) :63`. Each model is a pass/broken pair.
- Implementation witnesses live in `tests/formal_contracts/witnesses.rs`.

**Registry files**
1. `verification/coverage.toml` has one `[[crate]]` per **workspace package**:
   - `name`
   - `status = "contract-model" | "implementation-tests"`
   - `models`
   - `scope` (text; must be non-empty)
   - `sources` (paths that must exist)
2. `verification/contracts.toml` has one `[[contract]]` per model family:
   - `name`, `title`
   - `models = [X, BrokenX]`
   - `sources`, `implementation_tests`
   - `assumptions`, `property`
   - `checks = ["make test-formal","make test"]`
3. `verification/corpus.toml` has one `[[model]]` per model:
   - `name`
   - `expected = pass | safety | deadlock | liveness`
   - `frontend = project | yarn`
   - Omitted members fail.

**Omission check**
- `scripts/formal.py:70-92` `coverage()` globs root `Cargo.toml` `workspace.members`.
- It raises `formal coverage inventory differs: missing/stale` when the coverage names differ from those packages.
- A `contract-model` row without models fails, as does a missing source file or an unexecuted model (`:300-306` ties contracts to coverage).
- **Caveat:** the check sees only members of BT's root workspace. A submodule crate that is not a member is invisible to it.

**Make targets** (`Makefile:1398-1425`)
- `make test-yarn-model`: tlc tests, yarn `--features model`, no_std checks, clippy.
- `make test-formal`: test-yarn-model, then `scripts/test-formal.py`, tlc and yarn `verify` examples, `formal_contracts`, `formal.py`. Output goes to `$(TARGET_DIR)/formal`.
- `make test-formal-contracts`.
- `make check-formal-evidence`: receipt validation, also run by `make push` and `scripts/land.sh`.

`make test` runs `scripts/ci-local.sh`, which runs `make test-formal` at `:78`.

**Precedent for a separately published crate:** `make prepare-tlc` / `test-tlc-standalone` / `publish-tlc-github` (`Makefile:1432-1450`) export the in-tree `bboxy450-tlc` to a standalone signed mirror repo, `bboxy450/bboxy450-tlc`. That is the inverse of a submodule.

**Registering a new crate** (all three gates fail on omission when the crate is a workspace member):
1. Add it to `workspace.members`.
2. Add a `coverage.toml` row.
3. Place it in the `scripts/layering.sh` ORDER.
4. Add a `.bboxy/specs/licensing.md` row, enforced by `scripts/brand.sh`, which requires a `bboxy450-*` name or an internal row with `publish=false`.
5. If it has a model, add the model to `models.rs` (or a `reactor_model!`) with its Broken pair, plus `contracts.toml` and `corpus.toml` rows.

## 5. Hosting harnesses today: seats, adapters, MCP

**Profiles**
- `crates/bboxy450-harness-wake-up/src/lib.rs:77` `HarnessRecord{kind:String,name,command,wake_method,state_source,hooks}`.
  - `builtin() :97` covers claude (`ClaudeSocket`), codex (`CodexRpc`) and zsh (`Pty`).
- `crates/bbt-mux/src/adapters/mod.rs:32` adds tile, icon and default_role.
  - `:80-93` adds pi and omp as **generic** records: `WakeMethod::Pty`, `StateSource::Cadence`, no hooks. They are not real adapters (see d274).
- `crates/bbt-mux/src/mcp/seats.rs:22` `APPROVED_PROFILES=["zsh","claude","codex","pi","omp"]` is the allowlist.
- UI lists: `crates/bboxy450-term/src/settings/roles.rs:18`; `crates/bbt-proto/src/gui/roles.rs:61`.

**Launch**
1. `crates/bbt-mux/src/logic.rs:1633` `spawn_harness` sends `Command::Launch`.
2. `:6188` `Worker::launch` validates the role catalog, then calls `ensure_endpoint :6039`, which mints a 32-byte capability per generation.
3. `adapters/launch.rs:77` calls the kit's `launch::plan` (`wake-up/src/launch.rs:458`).
4. `Worker::spawn :6609` sends supervisor `Action::Spawn{LaunchSpec}`.
5. `attach :5689` builds the adapter `Session`.

The kit sets the env vars `BBOXY_SEAT_ID/GENERATION/ROLE/SOCK/CAPABILITY` (`wake-up/src/launch.rs:487-500`).
- Claude gets a private `mcp.json` (`bboxy → <exe> mcp`) and settings with hooks (`:520-580`).
- Codex gets a per-seat wrapper and a `--remote` per-seat proxy in front of the shared app-server daemon (`:589-700`).

**Grades**
- `wake-up/src/observe.rs:17` `Grade{Inferred<Shim<Reported}`.
- `Session::apply :260`: a lower grade cannot override a higher one within 5000 ms (`:297`).
- Sources: cadence/OSC 21337 → Inferred; hook shim/OSC 7502 → Shim; OSC 7501/133, the Claude sessions file and Codex app-server events → Reported.
- Wire mirror: `crates/bbt-ui-model/src/model/seat.rs:41`.

**Seat model**
- `crates/bbt-proto/src/registry.rs:96` `SeatRecord{id,swarm,name,harness_kind:String,harness_session_id,role,location,muted,generation,lifecycle,…}`; `Lifecycle :73`.
- There is **no seat-kind enum**.
- `seat_state.rs:60` `ProcessStatus{Running{pid},Exited,ClosedGrace,Retained}`.
- Lifecycle functions in `logic.rs`: restart `:6454` (generation+1, resume by session id); recover `:6302`; relaunch `:6326`; input `:6854`; exit `:7637`.
- Supervisor: `crates/bbt-supervisor/src/supervisor.rs:7` is a single PTY-owner actor. Its protocol is `protocol.rs:261` `Action`, framed as length-prefixed JSON (`transport.rs:7`).
- `crates/bbt-mux/src/provider/mod.rs:77` `trait Provider{link(host)->SupervisorConnection}` covers local, ssh and cloud.

**MCP server**
- Each harness runs the stdio bridge `bboxy450-term mcp` (`main.rs:9820`, `crates/bbt-mux/src/mcp/bridge.rs:436`).
- The bridge forwards newline JSON-RPC to the per-seat Unix socket.
- `seat_socket.rs:1844` checks `{version,capability,seat,generation}` on every request.
- Tools are registered in `mcp/tools.rs:22` `catalogue(actor,plugins)`.
  - Role filter: `role.class==Oracle`.
  - Every tool gets an `op_id` for idempotent replay (`mcp/journal.rs`).
- `ctl_door.rs:7`: the operator door refuses seats.

**Bus**
- rusqlite tables `msg_*` (`messaging/ledger.rs:187-202`).
- `bus.rs:835` `Broker` handles publish, poll and ack.
- `crates/bbt-proto/src/messaging.rs`:
  - `:62` `MessageKind`
  - `:103` `WakeMode`
  - `:242` signed `Envelope`
  - `:419` `Evidence{…NativeQueued,NativeWritten,NativeUnknown,ApplicationAcknowledged}`
  - `:983` `TaskRequest`
- `logic.rs:8327` `pump_messages` dispatches on `wake_method` at `:8580`:
  - **Claude:** NDJSON to `CLAUDE_CODE_MESSAGING_SOCKET`.
  - **Codex:** WebSocket JSON-RPC `thread/queue/add` / `turn/steer` (`wake-up/src/native.rs:457`, `:1159`).
  - **Pty:** `GuardedInput` (`:8748`).
  - **Operator:** handoff.
- Ambiguous native writes are never retried (`wake-up lib.rs:283`).

**Plugin SDK**
- `crates/bbt-plugin/src/contracts.rs:20` `Contract{DataSource,TaskSystem,EventSink,Session,RepoGraph,DecisionModel,Provider}`. Session and Provider are describe-only.
- One stdio-MCP child per plugin (`bbt-plugin-sdk/src/runtime.rs:95` `run::<P>()`; trait `lib.rs:645`).
- Discovery: `.bboxy/plugins` and `~/.bboxy/plugins` (`bbt-mux/src/plugins/discovery.rs:36`).
- Plugin tools are exposed as `plugin_<contract>_<provider>_<method>`.

**An in-process native harness is blocked by decision #20.** `.bboxy/decisions/d20-seats-and-jobs.md` says "A seat is a PTY; a pipe-mode harness run is a job." The PTY assumption is hard-wired in these places:
- `seat_socket.rs:1847` calls `snapshot()`, which needs `views[id]` (`logic.rs:2223` `ViewState{info:SeatInfo,…}`).
- `Worker::spawn` always uses the supervisor (`logic.rs:6659`).
- `Binding` requires a pid and process_start (`wake-up lib.rs:136-187`).
- `WakeMethod` (`lib.rs:53`) and `Endpoint` (`:192`) are closed enums.
- `attach` hard-codes the kind strings (`lib.rs:439-447`).
- The UI pane renders only a terminal `View`.

These seams are already transport-agnostic:
- `trait Adapter` (`wake-up lib.rs:363`; `adapters/mod.rs:144`), with `on_native`, `attach(Endpoint)` and `wake()`
- `StateSource::Reported`
- `LogicClient::native_event` (around `logic.rs:1747`)
- the `Messaging`/`Hook` socket actions (`seat_socket.rs:38`)

## 6. Repo conventions

**KB layout** (`.bboxy/README.md`)
- Directories: `decisions/` (d<N>-slug.md), `interfaces/` (ICDs), `invariants/`, `tasks/` (YYYY-MM-DD-slug.md), `research/`, `specs/`, `journey/`, `proposals/`, `notes/`, `runbooks/`, `runs/` (only `.log`/`.json` summaries), `artifacts/` (ULID.md), `comments/`, `log/*.jsonl`.
- Top-level files: `graph.md`, `vision.md`, `plan.md`, `roadmap.md`.
- The schema is in `.bboxy/specs/work-items.md:24-50`:
  - `id` (ULID), `type`, `title`, `status`
  - `parent`, `children`, `depends_on`
  - `links:[{to,rel}]`
  - `assignee`, `role`, `allowed_paths`, `verify`
  - `check`/`cost` (invariant)
  - `on`/`anchor`/`kind` (comment)
  - `created`, `updated`
- Invariant `frontmatter-parses`.

Real frontmatter examples:

```yaml
# decision — .bboxy/decisions/d1-crate-topology.md
id: "01M5JFXASG4FY1C4D4N2MYGFVD"
type: "decision"
number: 1
title: "Crate topology: split by process boundary"
status: "decided"
decided: "2026-10-22"
operator: []
links: [{ to: "[[2026-10-22-architecture-round-1#1]]", rel: "supersedes-as-index" }]
```

```yaml
# interface — .bboxy/interfaces/app-harness.md (body: Side A/B, What crosses, Timing, Lifecycle, Failure, Versioning, Verified by; Owner line)
id: 01M5WG2WRXB79K84R58QP93KT8
type: interface
number: 348
title: "ICD: app ↔ harness"
status: baselined
links: [{ to: "[[interfaces]]", rel: index }]
```

```yaml
# invariant — .bboxy/invariants/signed-main.md
id: 01M5JQE3ZY6KYN1P0NA2M75X0E
number: 140
type: invariant
title: every commit on main carries a pinned verified signature
status: active
check: python3 -I scripts/verify-signatures.py "${SIGNED_RANGE:-HEAD}"
cost: an unsigned commit breaks the chain from artifact to key; releases inherit the gap
```

```yaml
# task — .bboxy/tasks/2026-10-30-plugin-sdk-rust.md
id: "01M66NR73B287NMCJCN34MR5Q1"
type: "task"
number: 377
title: "Rust plugin SDK; …"
status: "done"
role: "oracle"
owner: "Vesper Greywind"
verify: "make test"
reconciliation: [{ by, disposition: implemented, evidence: [urls], reason }]
```

```yaml
# research — .bboxy/research/agent-messaging.md
id: "1VCB1Q6T5JZWQDH2MQTBZ27GPR"
type: "research"
number: 69
title: "Embedded agent messaging — source evidence"
```

Seat-written records also carry signed blocks:
```yaml
authorship: { kind, process_generation, public_key, signer }
provenance: { digest, frontmatter_digest, log: .bboxy/log/<date>.jsonl#N, sig, signer }
```
See `journey/implementation-journey.md` for an example. `number` is assigned by the tool. Write tasks through `ctl task` or the MCP `task_*`/`knowledge_*` tools, never GitKB.

**ATTRIBUTION.md**
- Header lines: Project, Canonical source, Copyright, Policy URL, License URL, and "Slop respectfully. Keep the credit attached."
- Followed by "Terms and adoption" (Respectful Slop License 1.0-Draft, `LicenseRef-Respectful-Slop-1.0-Draft`, `license-file = "LICENSE"`) and "Keep credit beside the material".
- Every source file starts with a 6-line header (`crates/bbt-rt/src/lib.rs:1-6`): Copyright, SPDX, Project+URL, License URL, credit request, Policy URL.
- `make test-licenses` checks this.

**Test stack**
- `make test` runs `scripts/ci-local.sh`:
  1. layering
  2. landing tests
  3. `cargo build --workspace`
  4. `clippy --workspace -D warnings`
  5. quarantine-aware unit tests
  6. `make test-formal`
  7. ui-lint
  8. a headless GUI smoke run: `--headless --smoke --ctl --unattended --test-account` with a private HOME (`ci-local.sh:69-95`)
- Other targets: `make smoke` (unsigned install plus headless smoke), `make test-boundaries`, `test-chat-*` (`Makefile:296-335`), `test-parity` (visual), `test-gui-events`.
- Everything runs unattended (invariant `e2e-runs-unattended`, `scripts/e2e-unattended.sh`).

**Signing**
- Invariant `signed-main` runs `scripts/verify-signatures.py`, with pinned keys in `scripts/verification-keys`.
- `make push` verifies the exact-HEAD CI line plus the formal receipt.
- Crates: `make package-crate CRATE=`, then `scripts/crates-publisher.py`.

**Executors**
- One writing worktree per Executor (sibling `../bboxy450-term-<topic>`).
- Stage exact paths only.
- Land through `scripts/land.sh`.

## 7. Recommendation

**A. Layout: submodule as a single package, member of BT's workspace.**
- Path: `crates/bboxy450-harness` (submodule → `bboxy450/bboxy450-harness`).
- Its root `Cargo.toml` is a **package with no `[workspace]`**. A nested workspace root would make Cargo refuse membership in BT's workspace.
- List it in BT `workspace.members`. That brings it under layering, brand, coverage, clippy and `cargo build --workspace`, so every omission gate applies.
- Do not use `version.workspace = true` or other inheritance; keep the fields explicit so a standalone clone still builds.
- Dependencies on the toolkit: `bboxy450-ui = { version = "0.1.1", ... }` and the like, as crates.io versions or `git = "https://github.com/bboxy450/bboxy450-term"`.
  - BT adds `[patch.crates-io]` (or `[patch."https://github.com/bboxy450/bboxy450-term"]`) path overrides, so the embedded build uses local sources.
  - **Unverified:** whether the `bboxy450-ui/render/doc/rt/yarn` 0.1.1 crates are on crates.io. Check that before choosing between crates.io and git deps.
- For standalone use, the harness repo carries its own `Cargo.lock`, Makefile, `verification/` (coverage + contracts for itself) and `.bboxy/`.
  - It still needs the pinned TLC tooling. Reuse BT's `scripts/formal.py` approach, or depend on `bboxy450-tlc` and keep native-only verification plus a Java parity target.
- Layer placement: after `bboxy450-doc` / `bboxy450-rt` / `bboxy450-yarn` / `bboxy450-plugin` / `bboxy450-term-proto`, and before `bboxy450-term` (if BT embeds it).
  - If it needs `bboxy450-term-mux` types, put it after mux. Prefer a thin mapping in BT, which is the d274 pattern.
- Alternative precedent: the `prepare-tlc` mirror export (in-tree source, published mirror). It avoids submodule friction in `land.sh` and the signature checks, but inverts ownership.

**B. Features**
- `default = []` keeps a library core: the conversation model, agent loop and view builders returning `bbt_ui::Element`.
- `standalone` adds winit and wgpu plus a `main` bin, following the `bboxybook/src/window.rs` pattern.
- `embedded` exposes `HarnessView` for BT to host. No winit in this mode; BT supplies `Measure`, the `ChromeRenderer` and the input routing.
- Keep the `Element`/`DrawList` boundary as the embedding API, so both modes share one view code path.

**C. Toolkit additions**

Additions are listed by owner crate; each should land upstream in BT, not be forked inside the harness.

`bbt-ui` / `bbt-doc`:
- markdown in `StreamDock` messages, or a new `Transcript` module
- streaming append API and incremental markdown, re-parsing only the tail block
- syntax highlighting: a `Code{info}` hook taking caller-supplied spans; check the license against `scripts/deps-licences.sh`
- diff view (unified/split, hunk folding)
- tool-call card atom: status, args, collapsible output, using `Accordion`
- `text_edit::Editor`: Up/Down, word and line motions, undo/redo, history, slash-command/@ completion popover

BT host:
- a pane-content abstraction, either `SplitNode::Leaf` carrying a content enum (`Terminal(SeatId) | View(ViewId)`) or a `LocalPage` variant for the harness
- precedent to copy: `LocalPageLayout` plus `RoutedKey::LocalPage`

Async (`bbt-rt` / `bbt-yarn`):
- the agent loop as the first product `reactor!`, with these sources:
  - `#[shutdown]` cancellation
  - interactive user input above the model stream: `yields_to` / `drain(max)`
  - tool completions: `OptionalOneShot` with `cancel_and_replace`
  - deadlines: `OptionalDeadline`
- model deltas to the UI through `Latest<T>`; the transcript through `ViewRing`/`UiSender`
- add a scoped task group (spawn + cancel-all + join over `bbt_rt::handle()`) if `OptionalOneShot` is not enough

Seat integration (crosses decision #20 and ICD #348, so it needs the Architect plus both owners):
- *Option 1, no core change:* run the harness as a process seat that is an MCP client of its own seat socket through the `BBOXY_*` env, reporting via `Action::Hook` and receiving wakes on a native socket. This needs a new `WakeMethod`/`Endpoint` variant for its socket and an adapter in `bboxy450-harness-wake-up` at grade Reported.
- *Option 2, in-process seat:*
  - make `ViewState.info` optional
  - add `ProcessStatus::InProcess`
  - relax the `Binding` pid
  - add `WakeMethod::InProcess` (channel)
  - add a non-terminal `snapshot()`
  - add a `"bboxy"` profile to `APPROVED_PROFILES` and `builtin()`
  - Needs a superseding decision for d20.

**D. Yarn models to add, each with a Broken pair, before the code**
- **AgentTurnLifecycle**: Idle → Streaming → ToolPending → Streaming → Done/Cancelled. Cancel mid-tool never applies a late tool result; this is generation-fenced, A→B→A.
- **StreamToView**: deltas applied in order, exactly once, to the current message generation; a dropped view retires late chunks (reuse the AsyncRenderPreparation shape).
- **InputPreemption**: user interrupt/steer always progresses under a firehose stream (`yields_to`).
- **EmbedLifecycle**: detaching from BT (close window, Cmd-Q) preserves the agent run; an explicit stop ends it, mirroring LifecycleOwnership.
- **ToolApproval**, if approvals exist: no tool executes without a current-generation approval.

## Unverified / gaps in this map

- Whether the toolkit crates are on crates.io (no network check was made).
- Behaviour of `scripts/land.sh` and `verify-signatures.py` with a submodule gitlink (not read).
- `Makefile` `check-licenses` handling of submodule paths (not read).
- Non-mac GUI support: none exists today.
