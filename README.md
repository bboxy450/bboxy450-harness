# bboxy450-harness

One Rust package for the bboxy450-term coding-agent core. Protocol v0.1, the
session actor, provider, tools, JSONL store and gateway are implemented here.
The GUI face and toolkit widgets are separate assignments. `standalone`,
`embedded` and `acp` are declared integration features; this core change does
not supply their window or ACP adapter.

```sh
cargo run -- -p "Inspect this project" --json --profile nemotron
cargo run -- seat --profile nemotron --cwd .
make test
make smoke-live
```

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
