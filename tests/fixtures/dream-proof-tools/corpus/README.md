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

Generator/assembly: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness.
Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
