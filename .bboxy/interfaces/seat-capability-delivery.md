---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
created: 2026-11-01T13:42:48.396Z
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
id: 01M6BNJD5YQMADXJQTQHX0DRDA
links:
- rel: index
  to: '[[interfaces]]'
- rel: amends
  to: bboxy450-term:[[app-harness]]
- rel: constrains
  to: '[[secrets-never-persist]]'
- rel: fed-by
  to: '[[mcp]]'
- rel: fed-by
  to: '[[embed-seat]]'
number: 4503
provenance:
  digest: sha256:29461d5506092d60281b933d401c49aafa481d6ffda68c7b91011cfaac174d25
  frontmatter_digest: sha256:52fb0f54e24ef5a6bddf6cdebeff9c1dcdff7be352282d1d836ab232be11b776
  log: .bboxy/log/2026-11-01.jsonl#28243
  sig: Cyc83UKYdThdNUrJ8R5Of/o8PQs3quwjdjhfYNFSnDqSHjdiw3VXqzbCw5b1dqsIMSX3FbYecmXuYuYvZwCHDg==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: baselined
title: 'ICD amendment: seat capability out of the process environment (Side A baselined, Side B open)'
type: interface
updated: 2026-11-01T13:42:48.396Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Proposed amendment: the seat capability leaves the environment

**Status: Side A baselined; Side B open.** Side A (bboxy450-term launch kit, supervisor, bridge, hook) landed on bboxy450-term main at `afda1b63` via `land --gate-log`, and the #348 rows of §5 are in `app-harness.md:20,23`. Changing a Side A clause now needs both owners and a recorded reason. **Side B (the harness) is not implemented** on harness main: see the §2 Side B bullet and the §4 harness row. The harness side is [[embed-seat]] §1 and [[mcp]] §5.

## 1. Problem

| Fact | Evidence |
|---|---|
| Every seat gets the 64-hex capability in its exec environment | `wake-up/src/launch.rs:495-502` (`BBOXY_CAPABILITY`) |
| Codex additionally gets a 0700 wrapper script that `export`s it, and an `env_vars` list naming it | `launch.rs:593-606`, `:626` |
| Two consumers read it, both from the environment | `bbt-mux/src/mcp/bridge.rs:45`, `bbt-mux/src/adapters/hook.rs:67` |
| Any process running under the same uid can read another process's exec-time environment | `ps -Eww -p <pid>`. Verified 2026-11-01 on this machine by the Oracle (an `env -i` child listed 7 `BBOXY_*` names) and by me (the variable was still listed after the child called `unsetenv`) |
| Seat sockets can be listed | `/tmp/bbt-seat-501-<hash>/<SEAT_ID>.sock`. A socket is useless without the capability, so the problem is the capability |

Consequence: any tool run by any seat, and any MCP server, can read every seat's capability passively, in one `ps`. A tool that prints its environment also puts the capability into model context and transcripts.

## 2. Change

- **Capability file.** Before spawn, the launch kit writes `<overlay>/capability`. It holds 64 hex characters plus a newline. It is opened with `O_CREAT|O_EXCL|O_NOFOLLOW` and mode 0600, in the per-launch overlay, which is already 0700 and named by a fresh ULID (`launch.rs:470-475`).
- **Environment.** `BBOXY_CAPABILITY_FILE=<path>` replaces `BBOXY_CAPABILITY`. **`BBOXY_CAPABILITY` is never set in any process's exec environment, for any seat kind.**
- **Codex.** The wrapper script exports `BBOXY_CAPABILITY_FILE` instead of the value, and the `env_vars` list names `BBOXY_CAPABILITY_FILE`.
- **Consumers.** `bridge.rs` and `hook.rs` read the file on every start. Hooks run once per event and MCP servers restart, so the file is never unlinked on read. A consumer refuses the file, with its typed reason, if any of these hold:
  - it is not a regular file;
  - it is a symlink;
  - it is not owned by the euid;
  - its mode is not 0600;
  - its content is not 64 hex characters.
- **Lifetime.** The file lives as long as the overlay. Seat close and generation + 1 remove the overlay (`cleanup_overlay`). A new generation gets a new overlay, a new capability and a new file, so the old path no longer exists.
- **Harness (Side B).** At start, every face (`seat`, headless, GUI, smoke) reads `BBOXY_CAPABILITY_FILE` once, through the same refusals as the consumers above, and registers the value with the redactor. The [[mcp]] §5 refusal of a resolved value that contains the capability compares against that value. [[mcp]] §5 removes `BBOXY_*` from profile and ACP servers, and that includes `BBOXY_CAPABILITY_FILE`. Tool processes (`process.rs` spawn) get `BBOXY_CAPABILITY_FILE` removed, as well as `BBOXY_CAPABILITY`.
  - **Gap at harness main `637cebb`.** Every face still reads only `BBOXY_CAPABILITY` (`src/mcp.rs:577,581`, `src/frontends/headless.rs:155`, `gui/window.rs:343`, `smoke.rs:74`), and Side A no longer sets it. Tools inherit `BBOXY_CAPABILITY_FILE`, because `process.rs:466` removes only `BBOXY_CAPABILITY`. So in a harness seat `cat "$BBOXY_CAPABILITY_FILE"` prints the value unredacted into the transcript and logs, and the §5 value refusal never fires. That breaks [[secrets-never-persist]] for this seat kind until Side B lands.

## 3. What this claims, and what it does not

- **It claims no passive exposure.** The capability value appears in no process's environment. So `ps -E`, `env` output, crash reports and inherited environments of third-party tools do not contain it.
- **It does not claim protection from an active same-uid reader.** The path appears in the environment, and a 0600 file is readable by its own uid. A process that wants the capability can still read the file. A real same-uid boundary needs either a per-seat uid, or authentication by kernel-attested ancestry (`LOCAL_PEERPID` plus the audit token). Ancestry breaks Codex, whose MCP servers are spawned by the shared daemon and are not descendants of the seat. Choosing either is a separate decision.
- Rejected:
  - **an inherited fd:** Claude Code (Node) and the Codex daemon do not pass non-stdio fds to the bridge and hook grandchildren, and hooks are spawned fresh for each event;
  - **a file read once and unlinked:** hooks and MCP restarts need to read it repeatedly;
  - **reading the value and then unsetting it:** the exec-time environment stays visible to `ps -E` (verified);
  - **a socket handshake on ancestry alone:** it breaks Codex, and it grants the seat's identity to every descendant, including untrusted MCP servers.

## 4. Verified by

| Clause | Evidence |
|---|---|
| no capability in any environment | bboxy450-term `make test-harness-native`, run for every seat kind (claude, codex, zsh, bboxy450-harness). For each spawned seat, a child process of the seat runs `ps -Eww -p <pid>` against the seat process and each ancestor up to the supervisor, then runs `env` itself. Over all of that output, the count of the seat's 64-hex value is 0 and the count of `BBOXY_CAPABILITY=` is 0 |
| consumers still authenticate | in the same run: the bridge's `orient` succeeds and one hook event is accepted, for every kind that has them |
| file shape and lifetime | the file is regular, mode 0600, owned by the euid, in a 0700 directory. After seat close the file is gone. After generation + 1 the old path is gone, and a bridge started with the old path's content is refused |
| consumer refusals | the bridge and the hook each refuse a symlink, mode 0644, a file owned by another uid, and malformed content, with typed reasons |
| broken control | a launch that also sets `BBOXY_CAPABILITY` in the environment makes the first row fail |
| harness side (**unproven**) | `tests/seat_process.rs`, with the capability delivered only as `BBOXY_CAPABILITY_FILE`: (a) a tool that runs `cat <path>` has the value redacted in its output and in the session log; (b) a tool's `printenv BBOXY_CAPABILITY_FILE` prints nothing; (c) a profile `echo_env` server sees neither `BBOXY_CAPABILITY_FILE` nor the value ([[mcp]] §6); (d) a profile `!cmd` whose output contains the value refuses the server with `reserved capability in environment`. Broken control: a redactor fed from `BBOXY_CAPABILITY` alone fails (a) |

## 5. #348 rows that change

| #348 row | Change |
|---|---|
| What crosses | the capability crosses as `BBOXY_CAPABILITY_FILE`, a 0600 file in the 0700 launch overlay. The value never crosses in the environment |
| Startup context | unchanged ("the capability never enters CLI flags"). Add: "nor any process environment" |
| States and lifecycle | the capability file has the overlay's lifetime and is replaced at generation + 1 |
| Verified by | add the §4 environment scan for every seat kind |

## Change log

- **2026-11-01, proposed at the Oracle's request** (message 01M6BN8XAGAW3C6AGHYZY3NWRK, a security finding from the PR #11 review). Of the three options the Oracle listed, the file wins because it works for every seat kind, including the Codex shared daemon. Codex already writes a 0700 file in the overlay (`launch.rs:593-606`). The fd and socket options fail for the reasons in §3.
- **2026-11-01, at the Oracle's request** (message 01M6CC26D3KMZNQY5GFKC027JR). Side A baselined. Evidence: signed `afda1b63` on bboxy450-term `origin/main`; its gate log ends `ci-local: result afda1b63… normal 0`; `capability_file.rs:363` scans claude, codex, zsh and bboxy450-harness, each launched with a fixture command, not the real binary (an abstraction: the environment comes from launch and supervisor, not from the binary); `every_seat_kind_keeps_capability_out_of_seat_and_ancestor_environments ... ok`; the broken control `test-capability-control.py` returns `red_exit 101` at `SeatAndAncestorEnvironmentIsCapabilityFree` with the source restored. I found the Side B gap while checking: the redactor and the §5 refusal are blind, and tools inherit the path. Side B is stated precisely (all faces, tool scrub), and the §4 harness row is marked unproven, with a broken control.

Owner: Nyx Copperwing <ARCHITECT> (proposal, harness side). Side A: bboxy450-term launch kit and `bbt-mux` (ICD #348; the Oracle signs under [[d4071]]).
