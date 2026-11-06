# Public-only presigned corpus

Copied from Git2.54.0's t/t7510-signed-commit.sh and ONLY the PUBLIC armor in
t/lib-gpg/keyring.gpg, authenticated through the pinned release archive:
https://github.com/git/git/tree/v2.54.0/t.
Original Git GPL-2.0 terms/notices remain; these are public test signatures and
certificates, never operator keys. No signing/key generation or private-packet
decoding/import/export produced this corpus.

The commit, ancestor, trees and blobs reconstruct the upstream test's exact
deterministic SHA-1 object bytes. Both original signature headers remain intact.
The corrupt control changes one signature byte and recomputes only its armor CRC;
real Git/GPG must decide its cryptographic outcome. Hashes and CRCs do not certify
either signature. Actual verification remains pending the Root-frozen canonical
private real-pair probe; corpus.lock.json records that limitation.

The builder checks SHA-1 Git header unfolding without changing any byte: each
commit's selected `gpgsig` equals its case's signature Ref (228 bytes, including
the final LF), and both payloads equal `valid-payload.txt` (218 bytes). Alternate
`gpgsig-sha256` headers are excluded from the SHA-1 payload and selected signature.
These are source/corpus equivalence checks, not cryptographic verification.

The implicit verifier operand is the matcher `{git_signature_temp}`. Git creates
its own `TMPDIR/.git_vtag_tmpXXXXXX`, writes the extracted bytes and closes it
before invoking GPG; it deletes the file after reap. The matcher is never
expanded to a fixture path or directly launched by the builder. Q2 alone owns
the actual argv, temporary identity/content/exclusivity and GS1/GS2 checks.

Generator/assembly: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness.
Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
