---
id: "01M6EV1PEFM485NZKMPSCRMRR3"
number: 5172
type: interface
title: "ICD: official Claude Code agent session"
status: proposed
created: "2026-11-02"
owner: "Architect (review pending); Oracle routes before merge"
---
# Claude Code agent session

Side A is the harness SessionActor and its Claude Code session worker (Executor E9).
Side B is the official external `claude` executable and its stream-json interface.
The Oracle owns profile selection and acceptance; the Architect owns baselining
this proposed interface. This extends the provider selection boundary without
routing CLI tool calls into the harness tool registry.

## Mechanism and version

The assigned task's line “Spawn through `bbt_rt::spawn_command`” is replaced by
Oracle ruling `01M6ESRQ3W4WEEEZ3F0SWHEZ2R` in delegation thread
`4MCXFPRCFNW49N34WCZR62NFTF`. The Oracle verified that bboxy450-rt 0.1.1
`commands.rs:207` hardcodes null stdin and exposes no writer. The approved
mechanism is `crate::process::OwnedChild`, scheduled on `bbt_rt::handle()`, with
piped stdin/stdout/stderr, `process_group(0)`, kill-on-drop and the existing
registered process-group ownership fence. No toolkit change is required.

The executable is resolved as `claude` on the inherited PATH, with the session
workspace as cwd. There is one long-lived CLI process per loaded harness session;
independent sessions have independent workers and groups. Arguments are a fixed
allowlist, with no shell:

```text
claude -p --input-format stream-json --output-format stream-json --verbose
       [--model M] [--permission-mode MODE] [--resume SESSION_ID]
```

Before each birth, a short-lived `claude --version` probe must exit 0 and report
exactly **2.1.304** as its first whitespace-delimited field. Output is bounded at
4 KiB and time at 10 seconds. A version mismatch is Config, with a version-check
fix. No auto-update runs. Updating this pin requires renewed fixture/process
verification and Architect review. The CLI flags and non-interactive operation
are documented by [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference)
and [programmatic operation](https://code.claude.com/docs/en/headless).

A minimal profile under `~/.bboxy450-harness/profiles/code.md` is:

```yaml
---
name: code
provider: claude-code
model: sonnet
permission_mode: dontAsk
---
```

Model and permission_mode are optional; the CLI's own defaults apply if omitted.
Permission modes are validated against the pinned CLI's known modes. Model
values cannot start with `-` or exceed 256 bytes. An endpoint is unnecessary;
api_key and api_key_env are refused for this provider. Harness `policy: yolo`,
context assembly, compaction and MCP/tool configuration do not replace the
CLI's tools, permissions or prompt. Explicit harness Compact is refused;
Steer is retained and queued as the next turn.

## Turns and projection

A turn writes one newline-terminated JSON user message to the existing stdin:

```json
{"type":"user","message":{"role":"user","content":[{"type":"text","text":"prompt"}]}}
```

Only the new user blocks are sent. File mentions become `@path` text; images and
unknown input are refused with BadRequest. The serialized input is capped at
1 MiB and a blocked stdin write times out after 10 seconds.

Stdout is untrusted UTF-8 JSONL. Framing retains at most 1 MiB including newline,
including under cancelled reads; malformed, oversized, truncated and non-object
lines fail with a fixed diagnostic, without echoing their contents. Parsing
above 4 KiB uses the harness blocking permits. Event UUIDs are capped at 128
bytes; a bounded 4096-entry UUID window retires duplicates within a process.

| CLI event/block | Existing protocol projection |
|---|---|
| system/init | Info Notice with profile, session_id, actual model and pinned version |
| assistant/text | AgentMessage, started then completed |
| assistant/thinking | Reasoning summary, started then completed; signature not persisted |
| assistant/tool_use | ToolCall with `claude-code:` name, Other kind, durable head and input |
| user/tool_result | Completes that external ToolCall with text/JSON output; is_error marks Failed |
| result | Usage (tokens, cache-read tokens, cost in USD micros), and one TurnEnded |
| unknown type, subtype or block | A Notice; never silently dropped |

Tool IDs are namespaced by harness TurnId. They are display-only: no registry
lookup, policy request or harness tool execution occurs. Duplicate calls/results
are noticed and retired. Pending calls are capped at 64, completed call IDs at
4096 per turn. Every open call receives a synthetic Cancelled/Failed completion
before the turn's single terminal. The existing transcript checker verifies
pairing and one ending.

The recognized init Notice has prefix `Claude Code init: ` and a JSON payload
containing `profile`, `session_id`, `model`, `version`. It is sequenced durably
through the existing event log before TurnEnded. UUID-shaped CLI IDs only are
accepted. On recovery or a profile returning A → B → A, the most recent matching
profile's init Notice supplies `--resume`; no second credential or session store
is introduced. A changed CLI session ID within a resumed process is Malformed.

## Lifecycle and failure

- Start: lazy at the first turn, version checked, cwd applied, inherited environment.
- Turn: init must arrive within 10 seconds of process birth. Thereafter an output
  stall of 600 seconds fails. Neither bound needs operator input.
- Success: result/is_error=false completes once and retains the process for the
  next user turn. The worker also drains stdout while idle.
- Result error: usage and subtype are retained in a bounded diagnostic. Auth
  errors become Auth with `claude /login` or `claude setup-token`; other result
  errors become BadRequest. Raw error bodies are not reflected.
- Cancel: signal the owned group with SIGINT, then use the approved existing
  terminate path (SIGTERM, 500 ms grace, SIGKILL). Join the leader and stderr
  reader, retire that process birth, synthesize open tool completions and emit
  Cancelled within the existing 2-second bound. Cancellation wins a result race.
- Exit/crash: observe the leader without early reap, preserve buffered final
  frames for at most 50 ms, kill residual group members, then reap. Active turns
  fail with exit code/signal and a redacted bounded stderr tail. An idle exit
  becomes a Notice and the next turn resumes. No event from a cancelled process
  can reach the replacement birth.
- Restart/profile switch/Close: stop and join the old worker/group. A later birth
  uses the durable resume ID. Dropping a GUI subscriber leaves the session alive.
- Missing binary or missing login: Auth Notice and terminal, with installation
  or `claude /login` / `claude setup-token` instructions. No login is initiated.

Stderr retains 8 KiB only in memory. Known credential syntax and already-registered
harness secrets are scrubbed before any display or persistence; version/error
parser bodies and environment values are never logged.

The pinned CLI is assumed to emit unique event UUIDs, with no newly-UUID'd
prior-turn terminal after a healthy result. UUID duplicates are retired; events
received idle are noticed and discarded. Cancel/restart does not depend on this
assumption: the entire old process and its pipes are joined and dropped. Formal
bounds do not prove arbitrary CLI behavior, Rust refinement, kernel scheduling,
wire bytes, filesystem durability or credentials inside the CLI.

## Compliance boundary

The official CLI owns its own agent loop, tools, permissions and login. The
harness never reads, copies, refreshes or sends Claude credentials, never opens
`~/.claude` credentials or calls Keychain to obtain them, and never calls
`api.anthropic.com` with a subscription token. It never passes --system-prompt,
--system-prompt-file, --tools, tools-off/allowed/disallowed tool flags, or --bare.
It never converts subscription OAuth into a raw completion API for harness tools.
The environment is inherited unchanged; no token is resolved or injected by the
adapter. Only the official CLI may consume the operator's existing login or
CLAUDE_CODE_OAUTH_TOKEN. This boundary is the assigned task's policy constraint;
this ICD does not make a general terms-of-service guarantee.

## Verification

`tests/fixtures/claude-code/claude` is an offline executable selected via the test
child's PATH. `tests/claude_code_agent.rs` covers text, thinking, external tools,
usage, multiple turns/one process, error results, hostile framing, unknown events,
crash, auth/missing binary, pin mismatch, cancellation with a timer/group probe,
and A → B → A both within a session and across harness crashes. macOS DYLD
interposition rejects harness credential opens and Keychain API calls; deliberate
file-open and Keychain controls must fail before the production witness passes.
The observed CLI argv must equal the allowlist and its environment sentinel must
remain inherited. No real login, network, Keychain permission or operator input
is used by these witnesses.

HarnessClaudeCodeSession and its Fence/Resume broken controls are inventoried in
verification/contracts.toml, corpus.toml and coverage.toml. They explore idle /
turn / cancelling / exited / resuming, three births and three turns, duplicate or
late completions, terminal conservation, durable resume identity and progress
under explicit weak fairness. The native checker exports TLA+; pinned Java TLC
must match reachable states, labeled transitions and verdicts. The canonical
gates are `make test && make test-formal`. Real subscription/network operation
and an optional live smoke are deliberately outside that unattended gate.
