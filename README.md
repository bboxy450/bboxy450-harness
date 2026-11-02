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
`make app-unsigned` uses an ad-hoc signature. `make smoke-app` checks that test
bundle with an offline rendered replay and a three-second empty-HOME launch,
without credentials or operator input. `make release-dry` packages the ad-hoc
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
