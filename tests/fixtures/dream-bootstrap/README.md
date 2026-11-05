# Dream bootstrap fixture authority

Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
Copyright 2026 Boxy (bboxy450) and contributors.
License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

These immutable vectors define the fixed `dream-bootstrap/1` record and binding.
They contain fixture identities and a synthetic worker digest; they authorize no
real worker, process or account. The full C1 Context uses the existing canonical
`dream-c1/valid/manifest.json` fixture values. No source inputs or credentials are
included. Keep the binary bytes unchanged: the attribution travels in this file
and the checker because inserting a source header would change the 128-byte record.

| Byte range | Encoding |
|---|---|
| 0..8 | ASCII `BBDBOOT1` |
| 8 | Purpose: 1 = `inspect-c1`, 2 = `read-prepare` |
| 9 | Probe: exactly 0 or 1 |
| 10..16 | Six zero reserved bytes |
| 16..32 | Launch birth ULID, canonical 128-bit big-endian representation |
| 32..48 | Run ULID, same representation |
| 48..64 | Nonce ULID, same representation |
| 64..96 | SHA256 binding, 32 raw bytes |
| 96..104 | Nonzero incarnation, unsigned 64-bit little-endian |
| 104..112 | Sequence, unsigned 64-bit little-endian, exactly 1 |
| 112..120 | Absolute `CLOCK_MONOTONIC` deadline in ns, unsigned 64-bit little-endian |
| 120..128 | Eight zero reserved bytes |

The environment token is ASCII, at most 128 bytes:
`1:<fd decimal 3..65535>:<canonical uppercase birth ULID>:<64 lowercase hex binding>`.
No signs, whitespace or extra fields are accepted. Leading zeroes in an otherwise
valid decimal descriptor are allowed by this fixture grammar. A 16-byte ULID
always has a canonical uppercase textual representation; overflow and case errors
are tested on textual environment IDs.

The binding is SHA256 of the exact compact, sorted-key, UTF-8 JSON object with
keys `profile`, `purpose`, `probe`, `op_id`, `run_id`, `nonce`, `incarnation`,
`context`, and `worker_sha256`; its `profile` is `dream-bootstrap/1`.
Every `probe:true` binding contains explicit `context:null`, never an omitted key,
empty object, fixture Context or prior-seat Context. Every `probe:false` binding
requires the complete current public Context; missing/null Context is refused by
the parent before birth. Static work vectors contain the complete fixture Context
for canonicalization only and do not establish live authority. Sidecars
`valid/*.binding.json` have no trailing LF. The hashes in `*.binding.sha256`
and `vectors.json` cover those exact canonical bytes, excluding any LF.

`vectors.json` inventories every binary and its exact SHA256. Each case carries
self-contained binding inputs, canonical JSON/hash, launch birth/descriptor,
environment token, and independent expected binary/environment/identity verdicts.
`valid/` includes inspect, inspect probe and read-prepare. A fourth accepted
case reuses the inspect binary with `1:0007:...`, proving that decimal descriptor
leading zeroes denote the same fd 7. `invalid/` includes both
structurally malformed records and structurally valid identity mismatches; a
mismatch must not be treated as malformed binary. Invalid environment cases retain
a valid record so the environment check cannot pass from a binary parser alone.
The separate `binding_cases` array retains two valid probe/work inputs and six
causal context negatives with exact canonical bytes/hashes and expected input
verdicts. Invalid input hashes document bytes; they do not authorize birth.

All records use the fixed deadline `0x0102030405060708`, decimal
`72623859790382856`. This deliberately tests little-endian byte order; it is **not**
a current live deadline or evidence of runtime admission. Runtime deadline tests
must inject their monotonic clock (for example deadline−1, deadline and deadline+1)
and separately enforce expiry. The independent checker verifies static format and
binding identity only, without launching a worker or accessing a source.

Run the independent Python standard-library conformance checker:

```sh
python3 -I tests/fixtures/dream-bootstrap/check.py
```

The checker reads immutable bytes and compares independent ULID anchors, binding
canonicalization/hashes, field/identity verdicts and required negative counts.
It never regenerates or repairs a vector. Rust consumer/producer parsers must run
against these files; a checker pass is not a runtime process/bootstrap claim.
