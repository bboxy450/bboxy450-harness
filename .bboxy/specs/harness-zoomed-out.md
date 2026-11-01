---
id: "08D625V1E7JNPS29P0P4Y4YZKN"
number: 19
type: spec
title: "bboxy450-harness — zoomed-out spec"
status: draft
created: 2026-10-31
owner: "Vesper Greywind (Oracle); review Nyx Copperwing (Architect)"
links:
  - { to: "[[kittens-spec]]", rel: evidence }
  - { to: "[[sota-harnesses]]", rel: evidence }
  - { to: "[[bboxy450-term-toolkit]]", rel: evidence }
  - { to: "[[toolkit-lowering]]", rel: lowered-by }
---
# bboxy450-harness — zoomed-out spec

## What it is

A coding-agent harness whose first-class face is a **native GUI with rich content**: streaming markdown, tool-call cards, diffs, live terminal output, plans, images and tables, approvals, and nested subagents. It is built on the bboxy450-term toolkit and lives as a **git submodule of bboxy450-term**. It runs either:

- **standalone**, in its own window or headless, or
- **embedded**, as a view or seat inside bboxy450-term.

There is one core, and every face is a client of it.

Lineage: Gonzih/kittens. We keep its rule that every turn has exactly one ending, its tool-call pairing, its cancellation tree, its retry-before-output policy, its `!cmd` key profiles and its ACP layer over a single event stream. We drop its scope sprawl, its per-token journal, its uncacheable prompt rebuilt each call, its double store and its 5k-line TUI loop. See [[kittens-spec]].

## Non-goals (v0)

- Process-wide sandbox backends. v0 has an approval policy gate only; Seatbelt comes later.
- Running other harnesses inside panes. That is bboxy450-term's job.
- Knowledge-base and workgraph code. bboxy450-term owns those, reachable through MCP.
- A TUI. Headless JSONL and ACP cover terminals and editors.
- Supporting more than one wire format at v0.

## Shape

```
 GUI (bbt-ui, standalone window | embedded view) ─┐
 headless JSONL (tests, scripts) ─────────────────┤  Gateway: send(Op)->Ack|Busy (never awaits)
 ACP stdio (Zed, bboxy450-term seat later) ───────┘           subscribe(after_seq)->Envelope stream
                                   │ bounded op inbox        ▲ per-subscriber bounded buffer, Lagged→replay
                     SessionActor (one task per session, owns state)
                                   │ run_turn(ctx, CancellationToken)
                     TurnRunner (bbt-yarn reactor; JoinSet of tools)
                       ├─ Provider stream (OpenAI-compatible; OpenRouter first)
                       ├─ ToolTask[k]  (ReadOnly ‖ parallel, Exclusive alone, per-path write locks)
                       └─ Policy.decide(action) → Allow | Ask(Approval item) | Deny
                     Store: single-writer append-only event log (JSONL tree) = source of truth
```

## Moving parts (v0 owners are modules in one package)

| Module | Owns |
|---|---|
| `protocol` | `Op`, `Event`, `Envelope{seq, session, item_id?, durable, payload}`, `Item` kinds. It has no dependencies. Wire schema is versioned. |
| `engine` | SessionActor, TurnRunner, steer/follow-up queues, cancellation tree, turn-end guarantee, tool pairing. |
| `provider` | `trait Provider { stream(req, cancel) -> Stream<ModelEvent> }`, OpenAI-compatible chat completions with streamed tool-call assembly, retry before first output, profiles with `!cmd` keys. |
| `context` | Prompt assembly once per turn with a stable cacheable prefix, token accounting, compaction ladder (clip → drop → summarize). |
| `tools` | Registry and schemas, concurrency classes, timeouts, output caps that spill to a file. Tools: read, write, edit (exact match; on a miss, report what was close), apply_patch, bash/exec (process-group kill), grep, glob. |
| `policy` | Rules as data; ask by default for writes and exec; "always allow" adds a rule. |
| `store` | Event log under `~/.bboxy450-harness/sessions/<id>.jsonl` with id/parent for forks, replay `after_seq`, fsync at turn and approval boundaries. |
| `mcp` | MCP client. Servers live for the session, and their tools enter the registry. |
| `gateway` | One trait for in-process and Unix-socket transports. Fan-out, lag, replay. |
| `frontends` | `headless`, `gui`, `acp`. Each is a pure consumer of `protocol`. |

## Items (what the GUI renders; ACP gets a lossless export)

Every item goes started → delta* → completed by item id. Deltas are ephemeral and coalesced at 30–100 ms per subscriber; completed items are durable.

The item kinds are:

- `UserMessage`, `AgentMessage{markdown}`, `Reasoning{summary, raw?}`
- `ToolCall{kind, status, input, output_blocks, locations}`
- `FileChange{path, unified}`, `TurnDiff`
- `Terminal{id, stream, exit}`
- `Plan{entries}`
- `Approval{subject, options}`
- `Image{mime, blob}`
- `Compaction{summary_ref}`
- `Usage`
- `Notice{level}` for retries and errors
- `Subagent{session_id}`

## Laws (to become invariants with checks; the Architect owns them)

1. **One ending.** Every accepted turn ends in exactly one `TurnEnd` or `Error`. Every item that started reaches a terminal state, including after cancel or crash replay.
2. **Pairing.** Every tool call in saved history has exactly one result, with a synthetic result if the call was cancelled.
3. **Never block.** `send` never awaits. The engine never waits on a frontend; a slow subscriber gets `Lagged` and replays.
4. **Cancel is cancel.** Cancelling a turn kills its process groups. No late tool result is applied to a newer generation (A → B → A).
5. **The log is truth.** Every view is a projection of the event log; a reconnect at `after_seq` yields no durable gap.
6. **No side effect outside policy.** Every write, exec or network-capable tool passes `policy.decide`.
7. **Secrets never persist.** Keys are resolved at use and redacted from logs, events and errors.

## Faces and hosting

- **Standalone:** `bboxy450-harness` runs the GUI window (feature `standalone`, the winit pattern from bboxybook). `bboxy450-harness -p "..." --json` runs headless. `bboxy450-harness acp` runs over stdio.
- **Embedded (phase 1, no bboxy450-term core change):** bboxy450-term launches it as a process seat using a new approved profile `bboxy450-harness` (`--acp` or `--json`). It talks to the bboxy MCP over its seat socket like any harness.
- **Embedded (phase 2, needs a decision superseding d20 "a seat is a PTY"):** an in-process view hosted as a LocalPage or pane content, using feature `embedded` and the same `Element` view code.

## Formal-first (operator, 2026-10-31: "TLA+ very important for async harness correctness and rendering")

Every stateful contract is modelled before it is coded. We reuse bboxy450-term's whole formal stack and do not build a parallel one: Yarn (`bboxy450-yarn`), the existing TLA+ models, the native `bboxy450-tlc`, and Java TLC parity. Evidence: [[formal-reuse]].

Each model below comes paired with a deliberately broken control, which the checker must reject, and a real process witness.

**Async core**
- **AgentTurnLifecycle** (Idle → Streaming → ToolPending → Streaming → Done | Cancelled | Error) proves law 1, one ending.
- **ToolPairing** proves law 2.
- **CancelTree with generation fencing A → B → A** proves law 4: no late result is applied to a newer generation.
- **SteerQueue** proves that user steer/interrupt always makes progress under a firehose stream.
- **ApprovalLifecycle** proves each approval resolves exactly once, as Resolved, TimedOut or Cancelled, and that nothing executes without an approval from the current generation.
- **SubscriberReplay** proves laws 3 and 5: lag and reconnect at `after_seq` leave no durable gap and never stall the engine.

**Rendering**
- **StreamToView** proves deltas are applied in order, exactly once, to the current item generation, and that late chunks for a dropped view are retired.
- **IncrementalMarkdown** proves the stable-prefix cache is never stale: re-parsing only the tail gives the same result as a full parse at every step. A property test witnesses this against the full parser.
- **DirtyWakeFrame** proves an event arrival always yields a frame, and that draining under the frame budget never loses an item. This reuses the toolkit's DirtyWake model.
- **ScrollStick** proves stick-to-bottom holds unless the user scrolled away, and is restored on a jump to the latest content.
- **EmbedLifecycle** proves a standalone or embedded detach preserves the run and only an explicit stop ends it.

The turn loop is a `reactor!`, so its arbitration rules (`yields_to`, `drain`, `shutdown`) are the same relations the models state. A source/model hash mismatch fails the build.

## Test model (no operator input)

- **Unit and formal:** Yarn/TLA+ models with broken controls for AgentTurnLifecycle, CancelTree, ApprovalLifecycle, SubscriberReplay and StreamToView.
- **Mock-provider e2e:** scripted SSE fixtures for tool calls, cancel mid-tool, retries and malformed streams.
- **Live smoke:** OpenRouter free NVIDIA model (`nvidia/nemotron-3.5-super-vl:free`) through a profile in `~/.bboxy450-harness/profiles/`. The key comes from a `!cmd`, never from disk in the repo. The smoke skips with a labelled SKIP (exit 0, not PASS) when no key is available.
- **GUI:** headless smoke renders recorded event fixtures to a framebuffer and makes assertions about the frame.
