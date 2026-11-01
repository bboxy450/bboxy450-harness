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
status: proposed
title: 'ICD amendment: seat capability out of the process environment (proposed)'
type: interface
updated: 2026-11-01T13:42:48.396Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Proposed amendment: the seat capability leaves the environment

**Status: proposed.** This proposes changes to bboxy450-term ICD #348 (app ↔ harness) and to `wake-up/src/launch.rs` for every seat kind. Under [[d4071]] it lands through a bboxy450-term PR, and the Oracle signs the #348 rows. The harness side is [[embed-seat]] §1 and [[mcp]] §5.

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
- **Harness (Side B).** At start, `seat` reads `BBOXY_CAPABILITY_FILE` once, only to register the value with the redactor. [[mcp]] §5 removes `BBOXY_*` from profile and ACP servers, and that includes `BBOXY_CAPABILITY_FILE`.

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
| harness side | `tests/seat_process.rs`: `BBOXY_CAPABILITY_FILE` is registered with the redactor (a tool that prints the value has it redacted), and a profile `echo_env` server sees neither `BBOXY_CAPABILITY_FILE` nor the value ([[mcp]] §6) |

## 5. #348 rows that change

| #348 row | Change |
|---|---|
| What crosses | the capability crosses as `BBOXY_CAPABILITY_FILE`, a 0600 file in the 0700 launch overlay. The value never crosses in the environment |
| Startup context | unchanged ("the capability never enters CLI flags"). Add: "nor any process environment" |
| States and lifecycle | the capability file has the overlay's lifetime and is replaced at generation + 1 |
| Verified by | add the §4 environment scan for every seat kind |

## Change log

- **2026-11-01, proposed at the Oracle's request** (message 01M6BN8XAGAW3C6AGHYZY3NWRK, a security finding from the PR #11 review). Of the three options the Oracle listed, the file wins because it works for every seat kind, including the Codex shared daemon. Codex already writes a 0700 file in the overlay (`launch.rs:593-606`). The fd and socket options fail for the reasons in §3.

Owner: Nyx Copperwing <ARCHITECT> (proposal, harness side). Side A: bboxy450-term launch kit and `bbt-mux` (ICD #348; the Oracle signs under [[d4071]]).
