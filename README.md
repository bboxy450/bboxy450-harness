# bboxy450-harness

One Rust package for the bboxy450-term coding-agent core. Protocol v0.1, the
session actor, provider, tools, JSONL store and gateway are implemented here.
The standalone macOS GUI uses the same dark bboxy450-ui toolkit as
bboxy450-term and also runs on its own. Headless, embedded and ACP clients
consume the same protocol.

```sh
cargo run -- -p "Inspect this project" --json --profile nemotron
cargo run -- seat --profile nemotron --cwd .
make test
make smoke-live
```

## Run standalone

Download the Apple Silicon app and its checksum/signature from
[GitHub Releases](https://github.com/bboxy450/bboxy450-harness/releases).
For example, from a directory where you want to unpack the app:

```sh
VERSION=0.1.0
ARCHIVE="bboxy450-harness-$VERSION-macos-arm64.zip"
for FILE in "$ARCHIVE" "$ARCHIVE.sha256" "$ARCHIVE.asc"; do
    curl -fLO "https://github.com/bboxy450/bboxy450-harness/releases/download/v$VERSION/$FILE"
done
curl -fLO https://raw.githubusercontent.com/bboxy450/bboxy450-harness/main/packaging/macos/release-signing-key.asc
gpg --import release-signing-key.asc
gpg --fingerprint 9B07E849480C68328CA20684844ECC0587DE7AB0
gpg --verify "$ARCHIVE.asc" "$ARCHIVE" &&
shasum -a 256 -c "$ARCHIVE.sha256" &&
ditto -x -k "$ARCHIVE" . &&
open "bboxy450-harness.app"
```

The signing fingerprint must be `9B07 E849 480C 6832 8CA2 0684 844E CC05 87DE 7AB0`.
Run each verification successfully before opening the app. The bundle uses
the approved Apple Development identity (team `UQB3368A84`); it is not notarized.
It needs no bboxy450-term installation.

Finder/Dock launch opens the GUI in your home directory. The default profile
is `BBOXY450_HARNESS_PROFILE`, then the name in
`~/.bboxy450-harness/default-profile`, then `nemotron`. With no profile installed,
the window stays open and shows a profile notice. To configure a profile, create
`~/.bboxy450-harness/profiles/nemotron.md`, for example:

```yaml
---
name: nemotron
model: nvidia/nemotron-3.5-super-vl:free
endpoint: https://openrouter.ai/api/v1/chat/completions
api_key_env: OPENROUTER_API_KEY
on_unattended: deny
---
```

With `OPENROUTER_API_KEY` set in your shell, launch explicitly in a project:

```sh
"bboxy450-harness.app/Contents/MacOS/launcher" gui --profile nemotron --cwd "$PWD"
```

The GUI loads the selected profile in the background. Shell environment
variables are available on the explicit launch; Finder uses its own environment.
Profiles also support an `api_key: '!command'` source for an unattended credential
helper. After adding a missing profile, relaunch the app.

From source, `make app` builds `target/bboxy450-harness.app` in release mode with
the approved Apple signature and fails if that identity is unavailable.
`make install` builds that signed app and installs it as
`~/Applications/bboxy450-harness.app`; override the destination with
`INSTALL_DIR=/path/to/Applications`. Reinstalling preserves the outer app inode
for Dock/Finder aliases and retains the previous `Contents` in the printed
private staging directory.

`make app-unsigned` uses an ad-hoc signature. `make install-unsigned` installs
that test app into `target/install-unsigned` by default; set
`UNSIGNED_INSTALL_DIR=/private/test/path` for an isolated destination. Its
default destination is separate from the trusted signed app. After installing,
`make install-cli` links the installed app's binary as
`~/.local/bin/bboxy450-harness` if `~/.local/bin` already exists, without sudo.
`make help` lists the targets and directory overrides.

`make smoke-app` checks that test bundle with an offline rendered replay and a
hidden native-window startup, without credentials or operator input. The smoke
creates a real window and Metal surface, configures the surface, and submits the
live chrome frame into a private texture because macOS occludes hidden windows.
It never shows a test window or captures the desktop; visible presentation is
outside this unattended check. `make test-app-startup-control` restores the
missing title-bar asset registration from 0c40316 and requires native startup to
fail before restoring the fix.
`make release-dry` packages the ad-hoc
test bundle and verifies its GPG signature/checksum in `target/release-artifacts/`;
it creates no tag or release. The Oracle uses `make release` on verified, clean
`main` to build the Apple-signed bundle, sign the archive, push a signed tag and
publish the three assets.

Profiles are YAML frontmatter in `~/.bboxy450-harness/profiles/<name>.md`.
They select `model`, `endpoint` and an `api_key` command beginning with `!`,
or `api_key_env`. Keys are resolved once, held with zeroizing storage and
registered for redaction. Keychain commands are refused unattended. Writes and
exec pass through policy; without an approver, `on_unattended` defaults to deny.
`policy: yolo` is an explicit allow rule, still recorded as an Approval item.

For an entirely offline process run, pass `--scripted fixture.sse` once per
round. `--sessions DIR`, `--cwd DIR` and `--yolo` support isolated e2e runs.
`--export-schema` emits the checked-in schema; `check-transcript FILE` validates
a completed JSONL transcript. `--json --resume SESSION` without a prompt replays a
closed log read-only, without loading a profile or changing its epoch.

Every face consumes the same envelopes. The work inbox holds 64 operations;
controls use the separate 256-entry FIFO from the completed protocol ICD.
Subscription registration precedes replay, which is read lazily from an
immutable log projection. Slow subscribers receive Lagged and reconnect.
Unix sockets are mode 0600 and check the peer uid. The GUI uses synchronous
InProc admission; a UnixClient awaits the server's correlated response.
`Host::create` and `Host::open` finish disk loading/recovery before exposing a
gateway or binding the session socket. Idle subscriptions get a 15-second
heartbeat. Seat text streams as snapshots arrive and prints each prefix once.

`Compact` writes a deterministic, capped Markdown digest without inference;
subsequent requests replace the old prefix with that stored summary. Forks
materialize a quiescent parent prefix and its blobs before publishing the child
log, so deleting the parent does not break the child. Neither rewrites history.

`make test` creates a private HOME and runs fmt, strict clippy, unit/process/e2e
checks, formal verification, a credential-shape scan and source-header checks.
Inference is mocked. Java TLC is a pinned development reference, downloaded
into the target cache if absent; it is not a product dependency.

`make test-formal-native` uses crates.io `bboxy450-tlc =0.1.1` and Yarn.
`make test-formal` / `make test-formal-parity` additionally compare complete
reachable states, labeled transitions and property verdicts with pinned Java
TLC. `make check-formal-evidence` rejects stale source/model/tool receipts.
See `verification/README.md` for bounds and the implementation-only coverage.

`make smoke-live` is separate. It copies only the existing kittens Nemotron
profile's key command into the harness profile and allows at most three HTTP
requests per invocation. Its redacted run summary distinguishes PASS, FAIL,
SKIP and UNAVAILABLE. SKIP and UNAVAILABLE are never PASS.

Credit and reuse policy: [ATTRIBUTION.md](ATTRIBUTION.md).

## Closed-source Dream C1 worker

The provisioned sibling executable exposes `dream inspect-c1 --job-dir ABSOLUTE_DIR`.
It dispatches before the shared async runtime and emits a capability Hello before
opening the job directory, then accepts one
matching Start over bounded JSONL pipes. The owner supplies a private 0700 job
with 0600 snapshots at `input/<session>.jsonl`, referenced blobs at
`input/<session>/blobs/<sha256>`, `input/manifest.json` committed last, and an
existing private `output/` directory. Launch with a cleared environment, LANG
and a private TMPDIR; no HOME, profile, seat capability or provider is needed.

`src/dream/c1_wire.rs` owns the transport, generated schema and locked corpus in
`tests/fixtures/dream-c1/`. Canonical hashing sorts keys recursively even when
ACP enables serde_json's insertion-order feature. UTF-8 scalar byte limits and
cross-field/order/cardinality rules require the DTO/worker validator as well as
the structural JSON Schema (JSON Schema string lengths count characters).

Selected immutable, terminal SessionClosed IR is validated incrementally,
including exact original line pins, non-fork transcript laws and typed blob inventories.
Native imports and SessionMeta.fork headers return unsupported_ir before replaying
inherited events; ordinary canonical fork support remains outside this C1 subset.
Inspection shares D0 fingerprints and selected-corpus cost ranking; it performs
no discovery, task join, shared cache write or inference. Results are canonical
bounded pages, published without replacement before result.json. The worker
uses nice +10/utility scheduling, measured ≤1 ms application/control/cleanup
units, an independent control owner, foreground pause and a wall deadline.
Exactly one 2 MiB compute owner is provisioned and parked before Hello, with
no source access until matching Start and quiescent foreground admission.
Only that `Builder::spawn` and each prechecked no-replace artifact rename get
the `c1-native-debt/1` native profile: observed ≤2 ms caller CPU/25 ms wall,
≤1 ms surrounding application CPU and ≤3 ms raw inclusive span. These are
observational admission allocations, not universal OS response bounds.
Each expensive span owes `9 * max(1 ms, inclusive CPU)` monotonic wall rest
AFTER its end; IO time cannot pay it. Bootstrap debt is paid before Hello;
controls remain independent. Timing failure retains actual handles or the
original publication nonce/operation/inode until joined settlement.
Resource overruns fail the run. Child declared working storage has atomic 78 MiB
credits, acquired before controllable growth and held through transfer/drop;
2 MiB controls and 16 MiB tested helper/startup allowance stay inside 96 MiB.
JSON preflight bounds depth 32, members 4096/container and total 65536 values/keys.
Old/new buffers, retained typed entries/payloads and serialization overlaps count.
Hidden serde/BTree/regex/native overhead is measured; the ledger proves declared
storage admission, not an absolute native allocator cap. Full observed child
heap exceeding 96 MiB fails. Diagnostics separately label actual live/high-water
heap, RSS, declared live/peak/cumulative credits and unmeasured native cumulative
allocation. Successful typed returns stay in an external stepped owner until the
CPU/heap/control verdict accepts them. Rejected returns, including unavailable
end clocks, drain nested JSON and typed collections in paced units before their
credits are released. Cancel/error teardown also uses paced units. Done follows actual
compute settlement and successful exit remains part of
the consumer's completion check.

Managed C1 additionally requires the private `dream-bootstrap/1` admission
before the first owned argv/path/Arc allocation. A non-inlined ordinary entry
keeps the ordinary CLI future/GUI frame behind the thin worker dispatch; exact
compiler/artifact prologues are profile evidence, not a loader latency guarantee.
Its sole extra cleared env entry
is `BBOXY_DREAM_BOOTSTRAP=1:N:birth:binding`, where N is the uniquely inherited
read-only anonymous pipe. The producer-owned fixed128-byte authority and independent
Rust/Python binding vectors live in `src/dream/bootstrap.rs` and
`tests/fixtures/dream-bootstrap/`. No worker JSON/schema/version changed.

Parent prepares the exact original binding/command/pipe, retains the returned
Child before clock verdicts, and repays launch/setup debt before its ONE permit
write. Its writer stays open as the revocation lease through paid-bootstrap Hello.
Child's fixed source-free control prelude uses at most1KiB declared state inside
the existing protected2MiB; it performs no hashing/JSON/source/provider discovery.
Only after a valid live permit can argv, channels and the sole compute owner run.
Lease/identity/deadline checks bracket each admission and follow every debt wait;
EOF/HUP/extra bytes refuse another span while retaining/joining any actual owner.
The read end retires only after bootstrap debt is paid. Parent then prepares/pays
and sends the existing bound Start. Probe=true binds explicit context:null and
permanently rejects Start; real jobs require complete public context before birth.
The opaque child binding comparison does not recompute or replace host authority.

Readiness stays min(child entry+250ms, original parent spawn-return readiness,
original job deadline), never restarted at grant or partial reads. At most4 permit
reads/4 interrupted waits are admitted. Nonblocking descriptor I/O prevents a
second reader/full stderr pipe from hanging the prelude. Darwin pipe reader and
writer inode stamps differ; retain each endpoint's own fullu64 stamp and the
constructor-linked custody, never assume endpoint inode equality or put them in
public wire numbers. Darwin is the implemented worker entry; GNU Linux recognizes
and refuses it before runtime through borrowed startup arguments. No additional
platform execution guarantee is claimed; late async C1 entry refuses.

Immediately before the unchanged waiting record, ordinary workers write one
fixed entry line (maximum211 bytes including LF) from stack storage:
`dream-bootstrap entry s=<u64> g=<u64> e=<u64> m=<u64> u=<u64> k=<u64> r=<u64> p=<u64> ok=<0|1>`.
The existing early entry sample e and then m use CLOCK_MONOTONIC; next comes
stack `proc_pid_rusage(getpid(), RUSAGE_INFO_V4)`, then g=`mach_absolute_time`,
then one unbuffered write. Only e/m support clock-domain comparisons, with
e<=m<=parent observation; e may precede parent spawn return. s/g/u/k/r/p remain
verbatim UNVERIFIED auxiliary data: no units, subtraction, phase cause or cost
authority is inferred. A failed rusage call sets only s/u/k/r/p to0 and ok=0;
clock and pipe failures still refuse. Sampling, formatting, descriptor checks
and both writes remain inside the existing single prepermit usage observation
and unchanged1ms control limit. No new profile, public wire or admission exists.

Bounded bootstrap diagnostic profile (duration units ns): `dream-bootstrap
state=waiting profile=dream-bootstrap/1` has `argv_admissions=0`,
`channel_admissions=0`, `owner_admissions=0` before the first controlled span.
`state=paid_hello` has those three counts plus `frame_admissions`, `permit_reads`,
`control_cpu_ns`, `control_max_cpu_ns`, after all expensive bootstrap debt and
before its own diagnostic emission. Final C1 control/process totals cover that
emission and read-end retirement. Four fixed admissions is the current profile;
no repeated-native or source work is hidden as control. Private approved test
fixtures alone expose an actual prepermit allocation mutation; the same marker
checker rejects it. This deliberately invalid artifact is revoked without a grant;
its actual allocation marker is required through bounded settlement/reap, even
if original readiness expired. Silence is not the causal witness and no later
marker certifies readiness. The ordinary positive worker never enables that mutation.

The unchanged `ResultDescriptor.max_poll_cpu_ns` captures the raw inclusive
maximum **before descriptor serialization/hash/write/publication**. It includes
native calls already observed and cannot certify its own later publication.
Terminal metrics include descriptor work, cleanup, joined ownership and the
final measured control tick. Fallback parked-owner sender disposal/join is a
measured ordinary settlement unit, folded into application/inclusive/main-owner
maxima with its post-span debt paid before terminal telemetry. Captured maximum
must not exceed terminal maximum.
Legacy 1 ms-only consumers may refuse; ≤3 ms alone does not prove this profile.
The ordinary-release/no-test-fixtures worker is retained in
`target/dream-c1-native-profile/worker` by the focused gate, beside exact source
commit, source/binary hashes and `rustc -Vv`. Fixture artifacts remain distinct.

Private bounded diagnostics freeze these metric names (all durations are ns):

| Record | Fields and cutoff |
|---|---|
| `native tag=compute_owner_create` / `publish_noreplace` | `count`, `cpu_ns`, `wall_ns`, `max_cpu_ns`, `max_wall_ns`, `inclusive_max_cpu_ns`, `inclusive_max_wall_ns`, `app_max_cpu_ns`, `effects`, `failures_with_effect`; exact callable observations; ≤1 and ≤4097 calls respectively |
| `terminal_metrics profile=c1-native-debt/1` | `captured_max_cpu_ns`, `inclusive_max_cpu_ns`, `app_max_cpu_ns`, `control_max_cpu_ns`, `inclusive_max_wall_ns`, `span_cpu_ns`, `owner_thread_cpu_ns`, `process_user_cpu_ns`, `process_system_cpu_ns`, `wall_ns`, `debt_total_ns`, `rest_observed_ns`, `remaining_debt_ns`; joined final-control cutoff before telemetry formatting/process teardown |
| `control_parts` | `input_max_cpu_ns`, `protocol_max_cpu_ns`, `settlement_max_cpu_ns`, `output_max_cpu_ns`, `total_cpu_ns`; whole ticks remain ≤1 ms independently of subspan measurements |
| `publication_custody` | original `run_id`, `nonce`, `operation` (page index or 4096 for result), `dev`, `ino`, `effect` (0 absent, 1 present, 2 unknown), `errno`, `cleanup_uncertain`; deterministic original private temp/output paths; one latest operation, retained through compute settlement |
| `metrics` | existing `compute_cpu_ns`, `max_unit_cpu_ns`, `cleanup_cpu_ns`, `max_cleanup_unit_cpu_ns`, heap/credit/normalization counters; compute cutoff, before owner-thread exit |
| Private test-parent `parent_reap` | `reap_user_cpu_ns`, `reap_system_cpu_ns`, `parent_elapsed_wall_ns`, `cutoff=actual_wait4_exit`; full reaped process usage includes telemetry/exit tail; parent elapsed wall includes launch/collection overhead |
| `cost_known` | false on unavailable/reversed clocks; unknown cost fails admission and is never a wrapping/zero CPU total |

`span_cpu_ns` sums measured expensive calling-thread spans only. Owner-thread CPU
and full process user/system CPU are separate overlapping observations and must
not be added together. Final parent post-reap usage covers telemetry/exit tail.
An unavailable inclusive CPU sample retains a conservative 27 ms settlement
rest. Every invalid measurement remains a sticky failure and never authorizes
another expensive span; a finite raw outer interval keeps its actual cost. No unbounded per-call logging or
additional wire frames are introduced. Formal C1QuantumDebt is a finite per-run
projection against a held peer; byte identity, OS timing, native heap, actual
process exit and complete consumer integration remain implementation witnesses.

Managed seats pass `--dream-owner term-c1`. This typed ownership setting preserves
durable SessionClosed and suppresses both legacy automatic D0 and automatic IR advice in engine, ACP and
native-watch close paths. Standalone/manual replay retains its existing owner.
`make test-dream-c1` runs private unit, wire/hash, process, large-input, legacy
replay and seat witnesses; native/Java evidence uses `make test-formal-parity`.
Consumer admission/authority, source retention, root/account slot limits and
native frame/input performance are verified in bboxy450-term.

The existing `test-fixtures` build feature additionally exposes
`seat --json --dream-owner term-c1 --dream-c1-fixture-next-session ULID`
with explicit private scripted inputs. It creates exactly one fresh nonfork B
through the retained Host after A's real durable close, preserving PID and using
the actual socket/Hello transport. This is a private API/transport witness;
ordinary CLI SessionClosed still exits. Physical EOF disables the transition.
`make test-dream-c1` runs the named private witnesses separately from ordinary
close/exit, Legacy, and generation-change tests.

Private managed fixtures preserve validated profile/model, `auto_compact:false`,
MCP, appended context and explicit `--yolo` intent; absence retains normal denial.
Launcher-origin same-name references for the three
public seat/generation/role labels are validated before excluding them from
substring-secret registration. Literal, command, alias and credential handling
keep their existing redaction, and an independently known secret stays registered.

## Private Dream proof tools (unapproved prerequisite)

`make fetch-dream-proof-tools` is the only provisioning step with network access.
It authenticates the seven exact release archives in
`tests/fixtures/dream-proof-tools/sources.lock.json` with the independently pinned
public-only bootstrap verifier before extraction. `make build-dream-proof-tools`
uses two niced jobs and private source/build/log directories under
`$TARGET_DIR/dream-proof-tools`; it installs nothing globally.

Root authorized pinned Libksba 1.8.1 as the seventh source after upstream GnuPG
2.5.24 required it even with GPGSM disabled. The builder retains that failed
six-source baseline, authenticates the new source and preserves the mandatory
library check. It substitutes no Homebrew binary libraries. Joined qualification
still requires Root's frozen canonical private-candidate probe; normal Git mode
remains `verification_unproven`.
The verifier input selection is currently unset. The offline adapter checks a
future Root-selected source, supplied immutable executable and preparation
receipt, including compiler, SDK, headers and selected build closure. It never
recompiles a probe or discovers a replacement. Historical probe results and the
signed C preflight remain separate evidence. Large preparation/header receipts
stay offline; missing selection or incomplete bindings refuse before a new build
or component launch. Ingestion success grants no execution or runtime authority.
The typed `provenance/pretest-basis.json` binds compiler, generated configuration,
candidate Mach-O images, ordered static selections, the fixed command profile,
public-only presigned fixture and conditional source review. Six descriptor
projections share that file. The single signed `closure-boundary.schema.json`
and its byte vectors define complete candidate loads with their modes, order and
repetitions. Selected-member hashes use exact length-prefixed raw names per
tool/output. OS internals use the explicit `trusted_platform_boundary/1` premise;
only candidate destinations are checked for bound shared-cache membership.
Before-birth/after-settle observations cannot establish absence of unseen changes.
The earlier recursive-platform graph exceeded 128 edges; that failed build and
all original evidence remain retained. `check --closure-projection-only` through
the recipe records an explicitly incomplete C projection from the retained run,
without executing its tools or creating a Basis, qualification or receipt.
`make test-dream-proof-tools` retains canonical native/ledger results and refuses
to certify a real pair. A repeated component check reuses exact retained results,
including failures, rather than retrying timing gates.

No Root admission is produced by this private build.
Q2's separate current-source approval, real Git→GPG component test, qualifying
publication, corrupt-proof negative and four-job product gates remain required.
The real complete Basis fit and joined qualification remain unproven. See the
fixture README for source review limits and upstream license mapping.
