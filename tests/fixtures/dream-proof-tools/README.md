# Private proof-tool source inputs

Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

`sources.lock.json` preserves the exact six-source ICD pins, signed-byte forms,
public roots, certificate/signature hashes and frozen Q2 probe. Certificates and
signatures are unmodified primary-source bytes. `patches.lock.json` pins complete
patch bytes and every affected upstream preimage/postimage. Patches apply with
zero fuzz; their postimages are checked before compilation. Original source
notices and licenses stay under their upstream terms (`license-map.json`).

The Git guards authorize one real, fixed absolute GPG verification child from
the existing signature path, consume its attempt before effects, reject generic
commands and async fork/thread work, and disable ENOEXEC shell fallback. Building
with NO_PTHREADS also selects upstream synchronous paths; it is not a lifetime
proof. GnuPG guards reject secret packet dispatch, unsupported commands/keyboxd
and service connections. The public-import change omits only the advisory
secret-key preference query in a public-only home; actual public import,
revocation checks and signature verification remain upstream implementations.
Libgpg-error/Libassuan/nPth guards refuse process/thread creation at their source
seams. Libgcrypt selects upstream portable code with jitterentropy disabled.

These patches are incomplete qualification work. Current-source tests check
pins, custody and finite models; the complete command/error/constructor/platform
closure review and actually executed real-tool mutants remain outstanding.
Greps, symbol inventories, versions, absence of installed daemons and successful
samples do not establish lifetime coverage.

Actual four-library builds succeeded with the pinned Xcode compiler/SDK. Upstream
GnuPG configure requires Libksba ≥1.6.3 unconditionally, including
`--disable-gpgsm`; no seventh source or configure waiver is authorized. Its real
failure is retained under the private build ID. No GPG or candidate receipt can
be produced until Root amends that boundary. The frozen canonical probe is
unchanged: native and ledger modes can supply component evidence, while its Git
mode unconditionally refuses with exit 81. No builder result fills Q2's absent
Root-reviewed admission literal or certifies a lesson.
