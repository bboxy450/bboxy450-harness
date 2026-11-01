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
title: 'ICD amendment: seat capability out of the process environment (baselined)'
type: interface
updated: 2026-11-01T13:42:48.396Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Proposed amendment: the seat capability leaves the environment

**Status: baselined, both sides.** Side A (bboxy450-term launch kit, supervisor, bridge, hook) landed on bboxy450-term main at `afda1b63` via `land --gate-log`, and the #348 rows of §5 are in `app-harness.md:20,23`. Changing a Side A clause now needs both owners and a recorded reason. Side B (the harness) was merged to harness main as `4deca92` (PR #18; tested merge `64a457f`). Follow-ups for the pending §4 cases are task #4921. The harness side is [[embed-seat]] §1 and [[mcp]] §5.

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
- **Harness (Side B).** At start, every face (`seat`, headless, ACP, GUI, smoke) reads `BBOXY_CAPABILITY_FILE` once, through the same refusals as the consumers above (`src/capability.rs`). It registers the value with the redactor before any provider, MCP server or tool output. The [[mcp]] §5 refusal of a resolved value that contains the capability compares against that value. [[mcp]] §5 removes `BBOXY_*` from profile and ACP servers, and that includes `BBOXY_CAPABILITY_FILE`. Tool processes have every `BBOXY_*` variable removed, which includes both of these (`src/process.rs:478`).
  - **Unset** (a non-seat run, such as `-p` or a GUI launched by hand): silent. No Notice, nothing registered. Oracle ruling, message 01M6CF89EXEQFFKDZ152B7648A.
  - **Set but invalid** (missing file, symlink, wrong owner or mode, malformed content): one typed Notice, `seat_admission_file_unavailable`, with safe text that never contains the path's content. The run continues without the value. A capability-gated action then fails at the bboxy side, not in the harness.
  - Implemented in PR #18, merged as `4deca92`. Before it, every face read only `BBOXY_CAPABILITY`, so the value went unredacted (finding of 2026-11-01, harness main `637cebb`).

## 3. What this claims, and what it does not

- **It claims no passive exposure.** The capability value appears in no process's environment. So `ps -E`, `env` output, crash reports and inherited environments of third-party tools do not contain it.
- **Redaction is contiguous-byte matching.** A tool that deliberately transforms the value (splits it, reverses it, changes its case, encodes it) defeats redaction. That is the same-uid limit below seen from the inside: a process that wants the capability can read the file. Redaction covers accidental display, not a determined reader.
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
| harness side | `tests/capability_delivery.rs` (PR #18), capability delivered only as `BBOXY_CAPABILITY_FILE`. **Proven:** (a) a tool that runs `cat <path>` has the value redacted in its output and in the session log, for headless, seat and ACP (`capability_file_value_is_redacted_in_real_headless_tool_and_log`, `seat_loads_capability_file_before_tool_output`, `acp_loads_capability_file_before_tool_output`); (b) a tool's environment carries neither variable (same tests, `assert_tool`); (d) a profile or ACP server whose resolved value equals the capability is refused with `reserved capability in environment` (`profile_and_acp_refuse_a_resolved_value_equal_to_the_capability`); unset is silent and set-but-invalid gives the typed Notice and continues (`missing_and_unsafe_files_emit_safe_notices_and_continue`). Model `HarnessMcpServer` property `NoUnredactedCapability`, with a broken control that reads only the legacy variable. **Proven by PR #20 (merged `74e2174`, task #4921):** (c) profile and ACP `echo_env` servers, and tools, see neither the variable nor the value, while a launcher server keeps it (`profile_and_acp_servers_and_tools_strip_capability_file_but_launcher_keeps_it`, `:407-450`); the live GUI registers the value before the host starts (`live_gui_registers_capability_before_host_starts`, `:588-645`); per-face broken controls in `scripts/test-capability-control.py`, run by `scripts/test.py:46`, with 8 cases: legacy-reader, headless, seat, acp, gui-replay, gui-live, gui-smoke and smoke. Each breaks registration for its face and requires the named assertion to fail. **Known gaps:** (1) an empty-string `BBOXY_CAPABILITY_FILE` has no test, though the code treats it as set-but-invalid (Notice). (2) The launcher case puts `BBOXY_CAPABILITY_FILE` in the server's explicit `env`, so a launcher server *inheriting* it from the face is unwitnessed |

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

- **2026-11-01, at the Oracle's ruling** (message 01M6CF89EXEQFFKDZ152B7648A, PR #18 Side B review). Unset `BBOXY_CAPABILITY_FILE` is silent; only set-but-invalid emits the typed Notice. Before this, every `-p` and GUI run showed "seat capability unavailable". The §4 harness row now cites `tests/capability_delivery.rs`: (a), (b) and (d) are proven, and (c), the live GUI and per-face controls are pending as E3f. Added the stated non-claim that deliberately transforming the value defeats redaction.
- **2026-11-01, at the Oracle's report** (message 01M6CFHE46WHXHCDFYKZB4FRTA). Side B baselined. I verified that `4deca92` is a signed merge on harness main. The tool-environment clause now cites the mechanism, a strip of every `BBOXY_*` variable (`process.rs:478`). The Oracle's gate on `64a457f`: `make test` exit 0 with 68 models in parity, smoke-live passed, 0 key hits.

- **2026-11-01, at the Oracle's report** (message 01M6CH1JKPPQQBX7SGH3EN4ZZE). The §4 pending items are proven by PR #20, merged as `74e2174`, signed. I read the cited tests and the 8 control cases. Two gaps are recorded: an empty-string path, and launcher inheritance, which the explicit config does not exercise.

Owner: Nyx Copperwing <ARCHITECT> (proposal, harness side). Side A: bboxy450-term launch kit and `bbt-mux` (ICD #348; the Oracle signs under [[d4071]]).
