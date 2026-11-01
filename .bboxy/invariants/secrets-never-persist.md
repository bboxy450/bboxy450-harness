---
type: invariant
title: "keys and seat capabilities are resolved at use and never appear in the log, events, blobs, errors, Debug output or the screen"
status: active
check: cargo test --test secrets && sh scripts/secret-scan.sh
cost: "an API key or a bboxy seat capability lands in a session log that is shared, or is echoed to a model and leaves the machine"
links:
  - { to: "[[provider]]", rel: constrains }
  - { to: "[[embed-seat]]", rel: constrains }
  - { to: "bboxy450-term:[[ctl-never-prints-secrets]]", rel: sibling }
created: "2026-10-31T00:00:00Z"
updated: "2026-10-31T00:00:00Z"
---
# Law 7: secrets never persist

**Property.**
- Secret values are the resolved `api_key`, `BBOXY_CAPABILITY` and any profile `*_env` value. Each lives in a `Secret(Zeroizing<String>)` whose `Debug` prints `<redacted>` and which implements no `Serialize`.
- Every resolved secret is registered with `redact::Set`.
- Every byte written by the store, emitted in an `Envelope`, written to stdout or stderr, or sent to a provider as message content passes `redact::scrub`, which replaces exact occurrences of a secret with `<redacted>`.
- Provider error bodies are never echoed. This follows kittens `provider.rs` and its credential-reflection rule.

**Check.**
- `tests/secrets.rs` resolves a sentinel key `sk-SENTINEL-<random>` through a `!printf` command. It then runs a scripted session in which the tool output, the provider error body and the model text all contain the sentinel. It asserts the sentinel occurs in none of: the log, the blobs, captured stdout/stderr, the `--json` stream, or `format!("{:?}", config)`.
- `scripts/secret-scan.sh` greps `.bboxy/`, `tests/fixtures/` and the tracked tree for key shapes (`sk-or-`, `sk-`, 32+ hex characters next to `key`) and exits 1 on a hit.

Owner: Nyx Copperwing <ARCHITECT>.
